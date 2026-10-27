//! A special room's `collapse` (spec §7 "Special rooms"): parsed, refused without `needs_below`
//! or over 100, never over a whole level, and read by [`Here::collapse`] of the room only.

use super::{Here, RoomEffectId, RoomEffects};

fn effect(extra: &str) -> String {
    format!(
        "[[effect]]\nid = \"collapsed\"\nname = \"collapsed floor\"\nmark = \"k\"\n\
         enter = \"Creak.\"\n{extra}\n"
    )
}

#[test]
fn collapse_parses_and_is_none_by_default() {
    let text = effect("needs_below = true\ncollapse = { chance = 15, damage = \"2d6\" }");
    let effects = RoomEffects::parse("e.toml", &text, |_| None).expect("valid");
    let c = effects.get(RoomEffectId(0)).collapse.expect("collapse");
    assert_eq!(c.chance, 15);
    assert_eq!(c.damage.to_string(), "2d6");
    let plain = RoomEffects::parse("e.toml", &effect(""), |_| None).expect("valid");
    assert!(plain.get(RoomEffectId(0)).collapse.is_none());
}

#[test]
fn collapse_needs_a_level_below_and_a_percent() {
    let e = RoomEffects::parse(
        "e.toml",
        &effect("collapse = { chance = 15, damage = \"2d6\" }"),
        |_| None,
    )
    .unwrap_err();
    assert!(e.message.contains("needs_below"), "{}", e.message);
    let e = RoomEffects::parse(
        "e.toml",
        &effect("needs_below = true\ncollapse = { chance = 101, damage = \"2d6\" }"),
        |_| None,
    )
    .unwrap_err();
    assert!(e.message.contains("collapse.chance 101"), "{}", e.message);
}

#[test]
fn collapse_never_covers_a_whole_level() {
    let text = effect(
        "needs_below = true\ncollapse = { chance = 15, damage = \"2d6\" }\nlevel_chance = 5\n\
         pervade = \"All of it.\"",
    );
    assert!(RoomEffects::parse("e.toml", &text, |_| None).is_err());
}

#[test]
fn here_reads_the_rooms_collapse_only() {
    let text = effect("needs_below = true\ncollapse = { chance = 15, damage = \"2d6\" }");
    let effects = RoomEffects::parse("e.toml", &text, |_| None).expect("valid");
    let def = effects.get(RoomEffectId(0));
    assert!(Here::default().collapse().is_none());
    let room = Here {
        room: Some(def),
        level: None,
    };
    assert_eq!(room.collapse().map(|c| c.chance), Some(15));
    let level = Here {
        room: None,
        level: Some(def),
    };
    assert!(level.collapse().is_none());
}
