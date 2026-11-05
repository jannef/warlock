//! The forge as data (spec §7 "Special rooms"; #418, #1022): the room, its anvil and what it
//! offers (Temper names no outcome table), and what is refused.

use super::*;

fn content() -> Content {
    Content::builtin().expect("content")
}

#[test]
fn the_forge_is_a_room_with_an_anvil_that_offers_temper() {
    let content = content();
    let anvil = content.terrain_id("anvil").expect("anvil");
    let def = content.terrain(anvil);
    assert!(!def.walkable && def.transparent && def.description.is_some());
    assert_eq!(content.offer(anvil), Some(Offer::Temper));
    assert!(content.offer_at(anvil).is_none(), "no outcome table");
    assert!(content.outcomes().get("forge").is_none());
    let room = content.room_effects().id("forge").expect("the forge");
    let def = content.room_effect(room);
    assert_eq!((def.weight, def.min_cells, def.mark), (3, 9, 'F'));
    assert_eq!(def.feature, Some(anvil));
    assert!(def.sides.is_none(), "an anvil has no side");
    assert!(!def.can_pervade());
}

#[test]
fn an_effect_offers_exactly_one_ability() {
    let content = content();
    let anvil = content.terrain_id("anvil").expect("anvil");
    let def = content.terrain(anvil).clone();
    let room = |offers: &str| {
        format!(
            "[[effect]]\nid = \"x\"\nname = \"x\"\nmark = \"x\"\nenter = \"X.\"\n\
             feature = \"anvil\"\noffers = {offers}\n"
        )
    };
    let parse = |text: &str| RoomEffects::parse("e.toml", text, |_| Some((anvil, &def)));
    let ok = parse(&room("{ temper = true }")).expect("one offer");
    assert_eq!(ok.defs()[0].offers, Some((Offer::Temper, None)));
    let both = parse(&room("{ pray = \"forgotten_altar\", temper = true }"));
    assert!(both.expect_err("two").to_string().contains("one thing"));
    let none = parse(&room("{ }"));
    assert!(
        none.expect_err("none")
            .to_string()
            .contains("names nothing")
    );
    let not = parse(&room("{ temper = false }"));
    assert!(not.expect_err("none").to_string().contains("names nothing"));
    assert!(parse(&room("{ temper = \"forge\" }")).is_err(), "a table");
}
