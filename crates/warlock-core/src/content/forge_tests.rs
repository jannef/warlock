//! The forge as data (spec §7 "Special rooms"; #418): the room, its anvil and what it offers,
//! the outcome table with its `work` hooks, and what is refused.

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
    let (offer, table) = content.offer_at(anvil).expect("the anvil offers");
    assert_eq!((offer, table.id.as_str()), (Offer::Temper, "forge"));
    let room = content.room_effects().id("forge").expect("the forge");
    let def = content.room_effect(room);
    assert_eq!((def.weight, def.min_cells, def.mark), (3, 9, 'F'));
    assert_eq!(def.feature, Some(anvil));
    assert!(def.sides.is_none(), "an anvil has no side");
    assert!(!def.can_pervade());
}

#[test]
fn the_forges_table_works_the_item_and_is_never_spent() {
    let content = content();
    let table = content.outcomes().get("forge").expect("the forge's table");
    assert_eq!((table.seconds, table.lean, table.exhaust), (600, 20, 0));
    assert_eq!(table.spent, None);
    assert_eq!(table.kinds, [35, 30, 35]);
    let work = |id: &str| {
        let o = table.outcomes.iter().find(|o| o.id == id).expect(id);
        (o.kind, o.weight, o.effect.work)
    };
    assert_eq!(
        work("improved"),
        (OutcomeKind::Boon, 35, Some(Work::Improve))
    );
    assert_eq!(work("nothing"), (OutcomeKind::Nothing, 30, None));
    assert_eq!(work("flawed"), (OutcomeKind::Bane, 25, Some(Work::Flaw)));
    assert_eq!(work("ruined"), (OutcomeKind::Bane, 10, Some(Work::Ruin)));
}

#[test]
fn an_unknown_work_is_refused() {
    let content = content();
    let text = "seconds = 60\nkinds = { boon = 1 }\nexhaust = 0\n[[outcome]]\nkind = \"boon\"\n\
                tier = \"minor\"\nid = \"x\"\nweight = 1\nmessage = \"A.\"\n\
                effect = { work = \"gild\" }\n";
    let e = OutcomeTables::parse(std::iter::once(("data/outcomes/t.toml", text)), &content)
        .expect_err("unknown work");
    assert!(e.to_string().contains("gild"), "{e}");
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
    let ok = parse(&room("{ temper = \"forge\" }")).expect("one offer");
    assert_eq!(
        ok.defs()[0].offers,
        Some((Offer::Temper, "forge".to_string()))
    );
    let both = parse(&room("{ pray = \"forgotten_altar\", temper = \"forge\" }"));
    assert!(both.expect_err("two").to_string().contains("one thing"));
    let none = parse(&room("{ }"));
    assert!(
        none.expect_err("none")
            .to_string()
            .contains("names nothing")
    );
}
