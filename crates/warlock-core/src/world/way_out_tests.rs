//! Climbing out through a hidden way reveals it (spec §7 "Ways up", #1117): Ūtgang's top floor
//! climbs out onto the slopes of Fyrbeorg, where its way in lies hidden until the player first
//! comes out there; then it is marked on their map, named, and a way back in. Arriving on the
//! overworld any other way reveals nothing.

use super::*;
use crate::action::parse_actions;
use crate::dump::{self, Layer};
use crate::replay::Replay;

/// Ūtgang's way in on the Ascmearc, on the volcano's south foot.
const UTGANG: Pos = Pos::new(103, 17);

fn act(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.text.clone()).collect()
}

fn locations(world: &World) -> String {
    dump::layer_rows(world, Layer::Locations).join("\n")
}

#[test]
fn the_way_in_is_hidden_until_the_player_climbs_out_of_it() {
    let content = Content::builtin().expect("content");
    let (at, hidden) = content
        .hidden_location("ascmearc", "utgang")
        .expect("hidden");
    assert_eq!(at, UTGANG);
    assert_eq!(hidden.to.as_deref(), Some("utgang:1"));
    let hills = content.terrain_id("hills").expect("hills");
    let overworld = content.overworld_of("ascmearc").expect("the overworld");
    assert_eq!(overworld.terrain[UTGANG], hills);

    let mut w = World::new(Arc::new(content), "utgang", None).expect("a run");
    let mut replay = Replay::new(&w);
    assert_eq!(w.perception().level_id(), "utgang:1");
    assert!(!w.state.revealed.contains("utgang"));
    let action = parse_actions("ascend").expect("actions").remove(0);
    w.apply(&action);
    replay.push(&action, &w);
    assert_eq!(w.perception().level_id(), "ascmearc");
    assert_eq!(w.player_pos(), UTGANG, "out on the way in");
    assert!(w.state.revealed.contains("utgang"));
    let lines = said(&w);
    assert!(
        lines.iter().any(|l| l == "You mark Ūtgang on your map."),
        "{lines:?}"
    );
    assert!(
        locations(&w).contains("(103, 17) Ūtgang"),
        "{}",
        locations(&w)
    );
    assert_eq!(w.perception().leads_to(UTGANG).as_deref(), Some("Ūtgang"));
    // From now on a way in like any other, and said only the once.
    let action = parse_actions("descend").expect("actions").remove(0);
    w.apply(&action);
    replay.push(&action, &w);
    assert_eq!(w.perception().level_id(), "utgang:1");
    let before = said(&w).len();
    let action = parse_actions("ascend").expect("actions").remove(0);
    w.apply(&action);
    replay.push(&action, &w);
    assert_eq!(w.player_pos(), UTGANG);
    assert!(!said(&w)[before..].iter().any(|l| l.contains("mark")));
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(w.content().clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}

#[test]
fn arriving_on_the_overworld_any_other_way_reveals_nothing() {
    let content = Arc::new(Content::builtin().expect("content"));
    // Out of Fyrdelf, by its own way in.
    let mut w = World::new(content.clone(), "ascmearc", None).expect("a run");
    act(&mut w, "reveal:fyrdelf");
    w.travel("fyrdelf:1", None).expect("down into Fyrdelf");
    act(&mut w, "ascend");
    assert_eq!(w.perception().level_id(), "ascmearc");
    assert!(!w.state.revealed.contains("utgang"));
    assert!(!locations(&w).contains("Ūtgang"));
    // Out of Grundlēas, open to all.
    let mut w = World::new(content, "grundleas", None).expect("a run");
    act(&mut w, "ascend");
    assert_eq!(w.perception().level_id(), "ascmearc");
    assert!(!w.state.revealed.contains("utgang"));
}
