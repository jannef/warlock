//! A hall of mirrors' `mirrors` (spec §7 "Special rooms"): parsed, left out by default, refused
//! out of range, and never over a whole level.

use super::room_effect::IMAGES;
use super::{RoomEffectId, RoomEffects};

fn effect(extra: &str) -> String {
    format!("[[effect]]\nid = \"m\"\nname = \"m\"\nmark = \"m\"\nenter = \"In.\"\n{extra}")
}

#[test]
fn mirrors_parse_and_default_to_none() {
    let plain = RoomEffects::parse("e.toml", &effect(""), |_| None).expect("valid");
    assert_eq!(plain.get(RoomEffectId(0)).mirrors, None);
    for n in [*IMAGES.start(), 2, *IMAGES.end()] {
        let text = effect(&format!("mirrors = {{ images = {n} }}\n"));
        let effects = RoomEffects::parse("e.toml", &text, |_| None).expect("valid");
        assert_eq!(
            effects.get(RoomEffectId(0)).mirrors.map(|m| m.images),
            Some(n)
        );
    }
}

#[test]
fn mirrors_out_of_range_or_over_a_whole_level_are_refused() {
    for n in [0, 9] {
        let text = effect(&format!("mirrors = {{ images = {n} }}\n"));
        let e = RoomEffects::parse("e.toml", &text, |_| None).unwrap_err();
        assert!(e.message.contains("mirrors.images"), "{}", e.message);
    }
    let text = effect("mirrors = { images = 2 }\nlevel_chance = 5\npervade = \"Glass.\"\n");
    let e = RoomEffects::parse("e.toml", &text, |_| None).unwrap_err();
    assert!(e.message.contains("holds mirrors"), "{}", e.message);
    let text = effect("mirrors = { images = 2, more = 1 }\n");
    assert!(RoomEffects::parse("e.toml", &text, |_| None).is_err());
}
