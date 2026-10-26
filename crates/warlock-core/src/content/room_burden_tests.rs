//! A special room's `burden` (spec §7 "Special rooms"): parsed, defaulted, refused out of range,
//! and multiplied by [`Here::burden`].

use super::room_effect::BURDEN;
use super::{Here, RoomEffectId, RoomEffects};

const PLAIN: &str = "[[effect]]\nid = \"plain\"\nname = \"plain room\"\nmark = \"p\"\n\
                     enter = \"Plain.\"\n";

fn effect(id: &str, mark: char, burden: i32) -> String {
    format!(
        "[[effect]]\nid = \"{id}\"\nname = \"{id}\"\nmark = \"{mark}\"\nenter = \"In.\"\n\
         burden = {burden}\n"
    )
}

#[test]
fn burden_parses_and_defaults_to_the_whole_limit() {
    let text = format!("{PLAIN}{}", effect("heavy", 'w', 50));
    let effects = RoomEffects::parse("e.toml", &text, |_| None).expect("valid");
    let burden = |i: u8| effects.get(RoomEffectId(i)).burden;
    assert_eq!((burden(0), burden(1)), (100, 50));
    for b in [*BURDEN.start(), *BURDEN.end()] {
        let text = effect("x", 'x', i32::from(b));
        assert!(RoomEffects::parse("e.toml", &text, |_| None).is_ok(), "{b}");
    }
}

#[test]
fn burden_out_of_range_is_refused() {
    for b in [0, 9, 1001] {
        let e = RoomEffects::parse("e.toml", &effect("x", 'x', b), |_| None).unwrap_err();
        assert!(e.message.contains(&format!("burden {b}")), "{}", e.message);
    }
}

#[test]
fn burden_multiplies_the_room_and_the_level() {
    let text = format!("{}{}", effect("heavy", 'w', 50), effect("light", 'l', 200));
    let effects = RoomEffects::parse("e.toml", &text, |_| None).expect("valid");
    let (heavy, light) = (effects.get(RoomEffectId(0)), effects.get(RoomEffectId(1)));
    assert_eq!(Here::default().burden(), 100);
    let room = Here {
        room: Some(heavy),
        level: None,
    };
    assert_eq!(room.burden(), 50);
    let both = Here {
        room: Some(heavy),
        level: Some(light),
    };
    assert_eq!(both.burden(), 100);
}
