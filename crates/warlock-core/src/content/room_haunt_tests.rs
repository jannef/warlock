//! A special room's `haunt` (spec §7 "Special rooms"): parsed, refused without `needs_danger` or
//! out of its range, never over a whole level, and read by [`Here::haunt`] of the room only.

use super::{Here, RoomEffectId, RoomEffects};
use crate::time::Duration;

fn effect(extra: &str) -> String {
    format!(
        "[[effect]]\nid = \"haunted\"\nname = \"haunted room\"\nmark = \"h\"\n\
         enter = \"Hush.\"\n{extra}\n"
    )
}

const HAUNT: &str = "needs_danger = true\nhaunt = { faction = \"undead\", every = \"1h\" }";

#[test]
fn haunt_parses_and_is_none_by_default() {
    let effects = RoomEffects::parse("e.toml", &effect(HAUNT), |_| None).expect("valid");
    let h = effects.get(RoomEffectId(0)).haunt.clone().expect("haunt");
    assert_eq!(h.faction, "undead");
    assert_eq!(h.every, Duration::seconds(3600));
    let plain = RoomEffects::parse("e.toml", &effect(""), |_| None).expect("valid");
    assert!(plain.get(RoomEffectId(0)).haunt.is_none());
}

#[test]
fn haunt_needs_a_danger_and_a_sane_period() {
    let refused = |extra: &str| {
        RoomEffects::parse("e.toml", &effect(extra), |_| None)
            .unwrap_err()
            .message
    };
    let e = refused("haunt = { faction = \"undead\", every = \"1h\" }");
    assert!(e.contains("needs_danger"), "{e}");
    let e = refused("needs_danger = true\nhaunt = { faction = \"undead\", every = \"5m\" }");
    assert!(e.contains("haunt.every"), "{e}");
    let e = refused("needs_danger = true\nhaunt = { faction = \"undead\", every = \"25h\" }");
    assert!(e.contains("haunt.every"), "{e}");
    let e = refused("needs_danger = true\nhaunt = { faction = \"undead\" }");
    assert!(e.contains("every"), "{e}");
}

#[test]
fn haunt_never_covers_a_whole_level() {
    let text = effect(&format!(
        "{HAUNT}\nlevel_chance = 5\npervade = \"All of it.\""
    ));
    assert!(RoomEffects::parse("e.toml", &text, |_| None).is_err());
}

#[test]
fn here_reads_the_rooms_haunt_only() {
    let effects = RoomEffects::parse("e.toml", &effect(HAUNT), |_| None).expect("valid");
    let def = effects.get(RoomEffectId(0));
    assert!(Here::default().haunt().is_none());
    let room = Here {
        room: Some(def),
        level: None,
    };
    assert_eq!(room.haunt().map(|h| h.every), Some(Duration::seconds(3600)));
    let level = Here {
        room: None,
        level: Some(def),
    };
    assert!(level.haunt().is_none());
}
