//! A special room's `temperature` (spec §7 "Special rooms"): parsed, defaulted, refused out of
//! range, and added up by [`Here::warmth`].

use super::room_effect::TEMPERATURE;
use super::{Here, RoomEffectId, RoomEffects};
use crate::temperature::Temperature;

const PLAIN: &str = "[[effect]]\nid = \"plain\"\nname = \"plain room\"\nmark = \"p\"\n\
                     enter = \"Plain.\"\n";

fn effect(id: &str, mark: char, temperature: i32) -> String {
    format!(
        "[[effect]]\nid = \"{id}\"\nname = \"{id}\"\nmark = \"{mark}\"\nenter = \"In.\"\n\
         temperature = {temperature}\n"
    )
}

#[test]
fn temperature_parses_and_defaults_to_none() {
    let text = format!(
        "{PLAIN}{}{}",
        effect("hot", 'h', 25),
        effect("cold", 'c', -25)
    );
    let effects = RoomEffects::parse("e.toml", &text, |_| None).expect("valid");
    let degrees = |i: u8| effects.get(RoomEffectId(i)).temperature;
    assert_eq!((degrees(0), degrees(1), degrees(2)), (0, 25, -25));
    // The extremes are allowed.
    for t in [*TEMPERATURE.start(), *TEMPERATURE.end()] {
        let text = effect("x", 'x', i32::from(t));
        assert!(RoomEffects::parse("e.toml", &text, |_| None).is_ok(), "{t}");
    }
}

#[test]
fn temperature_out_of_range_is_refused() {
    for t in [-41, 41] {
        let e = RoomEffects::parse("e.toml", &effect("x", 'x', t), |_| None).unwrap_err();
        assert!(
            e.message.contains(&format!("temperature {t}")),
            "{}",
            e.message
        );
    }
    // Beyond what the field holds at all.
    let e = RoomEffects::parse("e.toml", &effect("x", 'x', 300), |_| None).unwrap_err();
    assert!(e.message.contains("temperature"), "{}", e.message);
}

#[test]
fn warmth_adds_the_room_and_the_level() {
    let text = format!("{}{}", effect("hot", 'h', 25), effect("cold", 'c', -10));
    let effects = RoomEffects::parse("e.toml", &text, |_| None).expect("valid");
    let (hot, cold) = (effects.get(RoomEffectId(0)), effects.get(RoomEffectId(1)));
    assert_eq!(Here::default().warmth(), Temperature::ZERO);
    let room = Here {
        room: Some(hot),
        level: None,
    };
    assert_eq!(room.warmth(), Temperature::degrees(25));
    let both = Here {
        room: Some(hot),
        level: Some(cold),
    };
    assert_eq!(both.warmth(), Temperature::degrees(15));
}
