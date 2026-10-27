//! A special room's `warp` (spec §7 "Special rooms"): parsed, defaulted, refused over 100, and
//! folded by [`Here::warp`].

use super::{Here, RoomEffectId, RoomEffects};

fn effect(id: &str, mark: char, warp: i32) -> String {
    format!(
        "[[effect]]\nid = \"{id}\"\nname = \"{id}\"\nmark = \"{mark}\"\nenter = \"In.\"\n\
         warp = {warp}\n"
    )
}

const PLAIN: &str = "[[effect]]\nid = \"plain\"\nname = \"plain room\"\nmark = \"p\"\n\
                     enter = \"Plain.\"\n";

#[test]
fn warp_parses_and_defaults_to_never() {
    let text = format!("{PLAIN}{}", effect("unstable", 'u', 10));
    let effects = RoomEffects::parse("e.toml", &text, |_| None).expect("valid");
    let warp = |i: u8| effects.get(RoomEffectId(i)).warp;
    assert_eq!((warp(0), warp(1)), (0, 10));
    assert!(RoomEffects::parse("e.toml", &effect("x", 'x', 100), |_| None).is_ok());
}

#[test]
fn warp_over_a_hundred_is_refused() {
    let e = RoomEffects::parse("e.toml", &effect("x", 'x', 101), |_| None).unwrap_err();
    assert!(e.message.contains("warp 101"), "{}", e.message);
    assert!(RoomEffects::parse("e.toml", &effect("x", 'x', -1), |_| None).is_err());
}

#[test]
fn warp_holds_if_either_effect_warps() {
    let text = format!("{}{}", effect("a", 'a', 10), effect("b", 'b', 50));
    let effects = RoomEffects::parse("e.toml", &text, |_| None).expect("valid");
    let (a, b) = (effects.get(RoomEffectId(0)), effects.get(RoomEffectId(1)));
    assert_eq!(Here::default().warp(), 0);
    let room = Here {
        room: Some(a),
        level: None,
    };
    assert_eq!(room.warp(), 10);
    // 90 % of 50 % stay: 55 % go.
    let both = Here {
        room: Some(a),
        level: Some(b),
    };
    assert_eq!(both.warp(), 55);
}
