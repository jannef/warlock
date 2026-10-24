//! The lantern's hood (spec §5 "Light sources"; #235): its states, what each costs and says, saves,
//! and the content checks. The light it gives is tested with the rest of the light
//! (`light_tests.rs`).

use super::*;
use crate::content::ContentFiles;

fn showcase() -> World {
    let content = Arc::new(Content::builtin().expect("content"));
    World::new(content, "showcase", None).expect("showcase")
}

fn last_message(w: &World) -> String {
    w.messages()
        .iter()
        .last()
        .map(|m| m.display())
        .unwrap_or_default()
}

#[test]
fn the_hood_lowers_and_rises_and_lights_a_lantern_that_is_out() {
    let mut w = showcase();
    assert_eq!(w.lantern(), Some(Lantern::Out));
    // Out: hooding lights it with the hood lowered, in the time of both (6 s and 3 s).
    let outcome = w.apply(&Action::HoodLantern);
    assert_eq!(outcome.elapsed, Duration::seconds(9));
    assert_eq!(w.lantern(), Some(Lantern::Hooded));
    assert_eq!(
        last_message(&w),
        "You light your lantern with its hood lowered."
    );
    let hooded = w.content().light_id("lantern_hooded");
    assert_eq!(w.state().lantern_light(w.content()), hooded);
    // Raised and lowered again: a flick of the shutter.
    let outcome = w.apply(&Action::HoodLantern);
    assert_eq!(outcome.elapsed, Duration::seconds(3));
    assert_eq!(w.lantern(), Some(Lantern::Lit));
    assert_eq!(last_message(&w), "You raise your lantern's hood.");
    let open = w.content().light_id("lantern");
    assert_eq!(w.state().lantern_light(w.content()), open);
    w.apply(&Action::HoodLantern);
    assert_eq!(w.lantern(), Some(Lantern::Hooded));
    assert_eq!(last_message(&w), "You lower your lantern's hood.");
    // Doused, it is out, and lit again it burns open.
    w.apply(&Action::ToggleLantern);
    assert_eq!(w.lantern(), Some(Lantern::Out));
    assert_eq!(w.state().lantern_light(w.content()), None);
    w.apply(&Action::ToggleLantern);
    assert_eq!(w.lantern(), Some(Lantern::Lit));
}

#[test]
fn hooded_in_the_dark_the_player_gets_too_little_light_for_any_eyes() {
    // The darkness chamber's inner room, unlit: hooded, the lantern lights the player's
    // neighbours for their eyes, and leaves their own cell below every eye's threshold (only the
    // bounce off the ring, at most an eighth of it).
    let mut w = showcase();
    let walk = "move:N*3 open:N move:N*2 move:E*30 open:N move:N*3 move:W*6 move:N*5 move:E*6 \
                move:S*2 hood";
    for action in crate::action::parse_actions(walk).expect("actions") {
        w.apply(&action);
    }
    assert_eq!(w.lantern(), Some(Lantern::Hooded));
    let view = w.perception();
    let me = view.viewer();
    let faintest = (w.content().vision_profiles().iter())
        .map(|v| v.threshold)
        .min()
        .expect("profiles");
    let own = view.light(me).expect("own cell").light();
    assert!(own.brightness() < faintest, "{own:?}");
    for d in [Dir::E, Dir::W, Dir::NE] {
        assert!(view.is_visible(me.step(d)), "{d:?}");
    }
}

#[test]
fn the_hood_is_saved_and_left_out_of_saves_while_raised() {
    let mut w = showcase();
    let json = |w: &World| serde_json::to_value(w.state()).expect("state");
    assert!(
        json(&w).get("lantern_hooded").is_none(),
        "old saves and hashes stay"
    );
    w.apply(&Action::ToggleLantern);
    assert!(json(&w).get("lantern_hooded").is_none());
    let lit = w.state_hash();
    w.apply(&Action::HoodLantern);
    assert_eq!(json(&w)["lantern_hooded"], true);
    assert_ne!(w.state_hash(), lit);
    let restored = World::from_state(w.content().clone(), w.state().clone()).expect("restored");
    assert_eq!(restored.lantern(), Some(Lantern::Hooded));
    assert_eq!(restored.perception, w.perception, "the same light, cold");
}

#[test]
fn a_lantern_without_a_hood_cannot_be_hooded() {
    let mut files = ContentFiles::builtin();
    files.edit_table("lights", |t| t.replace("hooded = \"lantern_hooded\"\n", ""));
    let content = Arc::new(Content::load(&files).expect("content"));
    let mut w = World::new(content, "showcase", None).expect("showcase");
    let outcome = w.apply(&Action::HoodLantern);
    assert!(!outcome.took_turn);
    assert_eq!(last_message(&w), "Your lantern has no hood.");
    assert_eq!(w.lantern(), Some(Lantern::Out));
}

#[test]
fn a_hood_must_name_a_light_without_a_hood() {
    let mut files = ContentFiles::builtin();
    files.edit_table("lights", |t| {
        t.replace("hooded = \"lantern_hooded\"", "hooded = \"shutter\"")
    });
    let error = Content::load(&files).expect_err("unknown hood").to_string();
    assert!(error.contains("unknown light `shutter`"), "{error}");
    let mut files = ContentFiles::builtin();
    files.edit_table("lights", |t| {
        t.replace("hollow = true", "hollow = true\nhooded = \"lantern\"")
    });
    let error = Content::load(&files)
        .expect_err("a hood's hood")
        .to_string();
    assert!(error.contains("has a hood of its own"), "{error}");
}
