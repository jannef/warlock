//! Carried lights (spec §5 "Light sources"; #643): a lantern ready in the Tool slot, its modes,
//! what each costs and says, saves, and the content checks. The light it gives is tested with
//! the rest of the light (`light_tests.rs`).

use super::*;
use crate::content::ContentFiles;
use crate::object::LightMode;

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

/// Gives fixture content a lantern a scenario can ready (`wield = ["lantern"]`): a Tool slot,
/// the item classes, and a lantern of light preset `lantern`, which the fixture brings.
pub(super) fn with_lantern(files: &mut ContentFiles) {
    let slots = files.table("slots").map(|(_, t)| t.to_string()).next();
    let tool = "\n[[slot]]\nid = \"tool\"\nname = \"Tool\"\naction = \"ready\"\nmark = \"ready\"\n";
    files.insert(
        "data/slots.toml",
        &format!("{}{tool}", slots.unwrap_or_default()),
    );
    files.insert(
        "data/item_classes.toml",
        crate::content::BUILTIN_ITEM_CLASSES,
    );
    files.insert(
        "data/materials/zz_lantern_iron.toml",
        "[[material]]\nid = \"lantern_iron\"\nname = \"iron\"\n",
    );
    files.insert(
        "data/objects/zz_lantern.toml",
        "[[object]]\nid = \"lantern\"\nname = \"lantern\"\nglyph = \"(\"\nfg = \"#7a7d80\"\n\
         class = \"tools\"\nmaterials = [\"lantern_iron\"]\nweight = 1200\nvalue = 6\nslot = \"tool\"\nlight = \"lantern\"\n",
    );
}

/// The mode of the light the player has ready, if any.
pub(super) fn light_mode(w: &World) -> Option<LightMode> {
    let item = w.perception().ready_light()?;
    Some(w.state().player().inventory[item].light_mode())
}

#[test]
fn the_scenario_readies_an_unlit_lantern_in_the_tool_slot() {
    let w = showcase();
    let item = w.perception().ready_light().expect("a light ready");
    let player = w.state().player();
    let tool = w.content().slot_id("tool").expect("tool slot");
    assert_eq!(player.equipment.get(tool), Some(item));
    let lantern = &player.inventory[item];
    assert_eq!(lantern.describe(w.content()), "a lantern");
    assert_eq!(lantern.light_mode(), LightMode::Unlit);
    assert_eq!(w.state().carried_light(w.content()), None);
}

#[test]
fn using_the_tool_turns_a_lantern_lit_hooded_and_out() {
    let mut w = showcase();
    let open = w.content().light_id("lantern");
    let hooded = w.content().light_id("lantern_hooded");
    // Unlit to lit: the time of `light` (6 s).
    let outcome = w.apply(&Action::UseTool);
    assert_eq!(outcome.elapsed, Duration::seconds(6));
    assert_eq!(light_mode(&w), Some(LightMode::Lit));
    assert_eq!(last_message(&w), "You light your lantern.");
    assert_eq!(w.state().carried_light(w.content()), open);
    // Lit to hooded: a flick of the shutter (3 s).
    let outcome = w.apply(&Action::UseTool);
    assert_eq!(outcome.elapsed, Duration::seconds(3));
    assert_eq!(light_mode(&w), Some(LightMode::Hooded));
    assert_eq!(last_message(&w), "You lower the hood of your lantern.");
    assert_eq!(w.state().carried_light(w.content()), hooded);
    let item = w.perception().ready_light().expect("ready");
    let name = w.state().player().inventory[item].describe(w.content());
    assert_eq!(name, "a lantern (hooded)");
    // Hooded to unlit.
    let outcome = w.apply(&Action::UseTool);
    assert_eq!(outcome.elapsed, Duration::seconds(6));
    assert_eq!(light_mode(&w), Some(LightMode::Unlit));
    assert_eq!(last_message(&w), "You douse your lantern.");
    assert_eq!(w.state().carried_light(w.content()), None);
}

#[test]
fn a_lantern_gives_light_only_ready_in_the_tool_slot() {
    let mut w = showcase();
    w.apply(&Action::UseTool);
    assert!(w.state().carried_light(w.content()).is_some());
    w.apply(&Action::Remove("tool".into()));
    assert_eq!(w.perception().ready_light(), None);
    assert_eq!(
        w.state().carried_light(w.content()),
        None,
        "lit, but put away"
    );
    let lantern = &w.state().player().inventory[0];
    assert_eq!(lantern.light_mode(), LightMode::Lit, "it keeps its mode");
    let outcome = w.apply(&Action::UseTool);
    assert!(!outcome.took_turn);
    assert_eq!(last_message(&w), "You have no light ready to use.");
    w.apply(&Action::Equip {
        slot: "tool".into(),
        item: 0,
    });
    assert_eq!(
        w.state().carried_light(w.content()),
        w.content().light_id("lantern"),
        "readied again, it burns as it was left"
    );
}

#[test]
fn hooded_in_the_dark_the_player_gets_too_little_light_for_any_eyes() {
    // The darkness chamber's inner room, unlit: hooded, the lantern lights the player's
    // neighbours for their eyes, and leaves their own cell below every eye's threshold (only the
    // bounce off the ring, at most an eighth of it).
    let mut w = showcase();
    let walk = "move:N*3 open:N move:N*2 move:E*30 open:N move:N*3 move:W*6 move:N*5 move:E*6 \
                move:S*2 use_tool*2";
    for action in crate::action::parse_actions(walk).expect("actions") {
        w.apply(&action);
    }
    assert_eq!(light_mode(&w), Some(LightMode::Hooded));
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
fn the_mode_is_saved_with_the_item_and_left_out_while_unlit() {
    let mut w = showcase();
    let item = |w: &World| {
        let at = w.perception().ready_light().expect("ready");
        serde_json::to_value(&w.state().player().inventory[at]).expect("stack")
    };
    assert!(item(&w).get("light").is_none(), "unlit: left out");
    let unlit = w.state_hash();
    w.apply(&Action::UseTool);
    assert_eq!(item(&w)["light"], "lit");
    w.apply(&Action::UseTool);
    assert_eq!(item(&w)["light"], "hooded");
    assert_ne!(w.state_hash(), unlit);
    let restored = World::from_state(w.content().clone(), w.state().clone()).expect("restored");
    assert_eq!(light_mode(&restored), Some(LightMode::Hooded));
    assert_eq!(restored.perception, w.perception, "the same light, cold");
}

#[test]
fn a_lantern_without_a_hood_goes_from_lit_to_unlit() {
    let mut files = ContentFiles::builtin();
    files.edit_table("lights", |t| t.replace("hooded = \"lantern_hooded\"\n", ""));
    let content = Arc::new(Content::load(&files).expect("content"));
    let mut w = World::new(content, "showcase", None).expect("showcase");
    w.apply(&Action::UseTool);
    assert_eq!(light_mode(&w), Some(LightMode::Lit));
    w.apply(&Action::UseTool);
    assert_eq!(light_mode(&w), Some(LightMode::Unlit));
    assert_eq!(last_message(&w), "You douse your lantern.");
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
