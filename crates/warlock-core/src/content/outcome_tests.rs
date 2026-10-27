//! Outcome tables as data (`data/outcomes/`): what parses, what is refused, the leaned kinds.

use super::*;
use crate::content::{Content, Offer};

fn content() -> Content {
    Content::builtin().expect("content")
}

fn parse(text: &str) -> Result<OutcomeTables, ContentError> {
    let content = content();
    OutcomeTables::parse(std::iter::once(("data/outcomes/test.toml", text)), &content)
}

const HEAD: &str = "seconds = 60\nkinds = { boon = 35, nothing = 30, bane = 35 }\nlean = 15\n\
                    spent = \"spent_altar\"\n";

const ONE_EACH: &str = "[[outcome]]\nkind = \"boon\"\ntier = \"minor\"\nid = \"a\"\nweight = 1\n\
                        message = \"A.\"\n\
                        [[outcome]]\nkind = \"nothing\"\ntier = \"minor\"\nid = \"b\"\nweight = 1\n\
                        message = \"B.\"\n\
                        [[outcome]]\nkind = \"bane\"\ntier = \"minor\"\nid = \"c\"\nweight = 1\n\
                        message = \"C.\"\n";

#[test]
fn the_forgotten_altars_table_loads_with_its_minor_and_moderate_outcomes() {
    let content = content();
    let table = content
        .outcomes()
        .get("forgotten_altar")
        .expect("the altar's table");
    assert_eq!((table.seconds, table.turns()), (60, 6));
    assert_eq!(table.kinds, [35, 30, 35]);
    assert_eq!((table.lean, table.exhaust), (15, 50));
    assert_eq!(table.spent, content.terrain_id("spent_altar"));
    // Six "nothing happens" lines, the omen of four.
    assert_eq!(table.of(OutcomeKind::Nothing).count(), 6);
    assert_eq!(table.total(OutcomeKind::Nothing), 100);
    let omen = table
        .of(OutcomeKind::Nothing)
        .find(|o| o.id == "omen")
        .expect("omen");
    assert_eq!(omen.messages.len(), 4);
    // Within each kind: the minor and moderate tiers as the design gives them (the great ones
    // come with #407).
    let tier = |kind, tier| -> u32 {
        (table.of(kind).filter(|o| o.tier == tier))
            .map(|o| o.weight)
            .sum()
    };
    assert_eq!(tier(OutcomeKind::Boon, OutcomeTier::Minor), 65);
    assert_eq!(tier(OutcomeKind::Boon, OutcomeTier::Moderate), 29);
    assert_eq!(tier(OutcomeKind::Bane, OutcomeTier::Minor), 60);
    assert_eq!(tier(OutcomeKind::Bane, OutcomeTier::Moderate), 34);
    assert_eq!(tier(OutcomeKind::Boon, OutcomeTier::Great), 0);
    // The explosion: a fireball of radius 2, and rubble.
    let burst = table
        .outcomes
        .iter()
        .find(|o| o.id == "burst")
        .expect("burst");
    let explosion = burst.effect.explode.as_ref().expect("it explodes");
    assert_eq!(explosion.burst.radius, 2);
    assert_eq!(burst.effect.becomes, content.terrain_id("rubble"));
    // Every outcome of the table in the design's order of hooks has a line.
    assert!(table.outcomes.iter().all(|o| !o.messages.is_empty()));
}

#[test]
fn a_room_offers_pray_beside_its_feature_and_nothing_once_spent() {
    let content = content();
    let altar = content.terrain_id("forgotten_altar").expect("altar");
    let spent = content.terrain_id("spent_altar").expect("spent");
    let (offer, table) = content.offer_at(altar).expect("the altar offers");
    assert_eq!((offer, table.id.as_str()), (Offer::Pray, "forgotten_altar"));
    assert!(content.offer_at(spent).is_none());
    assert!(
        content
            .offer_at(content.terrain_id("altar").expect("altar"))
            .is_none()
    );
    let room = content.room_effects().id("altar").expect("the altar room");
    let def = content.room_effect(room);
    assert_eq!((def.weight, def.min_cells), (8, 9));
    assert_eq!(def.feature, Some(altar));
    assert!(def.side_light(crate::alignment::Alignment::Good).is_some());
    assert!(
        def.side_light(crate::alignment::Alignment::Neutral)
            .is_none()
    );
    assert!(!def.can_pervade());
}

#[test]
fn a_small_table_parses_and_its_fields_default() {
    let tables = parse(&format!("{HEAD}{ONE_EACH}")).expect("valid");
    let table = tables.get("test").expect("named by its file");
    assert_eq!(table.exhaust, 50, "the owner's default");
    assert_eq!(table.outcomes.len(), 3);
    assert_eq!(table.outcomes[0].effect, OutcomeEffect::default());
}

#[test]
fn the_lean_moves_weight_between_boon_and_bane() {
    let tables = parse(&format!("{HEAD}{ONE_EACH}")).expect("valid");
    let table = tables.get("test").expect("table");
    assert_eq!(table.leaned(0), [35, 30, 35]);
    assert_eq!(table.leaned(15), [50, 30, 20]);
    assert_eq!(table.leaned(-15), [20, 30, 50]);
    assert_eq!(
        table.leaned(100),
        [70, 30, 0],
        "never more than the bane has"
    );
}

#[test]
fn bad_tables_are_refused() {
    let cases = [
        (
            "seconds = 0\nkinds = { boon = 1 }\nexhaust = 0\n",
            "seconds",
        ),
        ("seconds = 60\nkinds = { }\nexhaust = 0\n", "not all 0"),
        (
            "seconds = 60\nkinds = { boon = 10, bane = 5 }\nlean = 6\nexhaust = 0\n",
            "lean",
        ),
        (
            "seconds = 60\nkinds = { boon = 1 }\nexhaust = 101\n",
            "percent",
        ),
        ("seconds = 60\nkinds = { boon = 1 }\n", "needs `spent`"),
        (
            "seconds = 60\nkinds = { boon = 1 }\nexhaust = 0\n",
            "no outcome",
        ),
    ];
    for (text, why) in cases {
        let e = parse(text).expect_err(text).to_string();
        assert!(e.contains(why), "{text}: {e}");
    }
}

#[test]
fn bad_outcomes_are_refused() {
    let outcome = |extra: &str| {
        format!(
            "seconds = 60\nkinds = {{ boon = 1 }}\nexhaust = 0\n[[outcome]]\nkind = \"boon\"\n\
             tier = \"minor\"\nid = \"x\"\nweight = 1\n{extra}"
        )
    };
    let cases = [
        ("", "message"),
        ("message = \" \"\n", "message"),
        (
            "message = \"A.\"\neffect = { status = { id = \"nonesuch\" } }\n",
            "unknown status",
        ),
        (
            "message = \"A.\"\neffect = { status = { id = \"inspired\", degrees = 2 } }\n",
            "degrees",
        ),
        (
            "message = \"A.\"\neffect = { heal = \"0\" }\n",
            "at least 1",
        ),
        (
            "message = \"A.\"\neffect = { know_traps = 99 }\n",
            "know_traps",
        ),
        (
            "message = \"A.\"\neffect = { lose_coins = 120 }\n",
            "percents",
        ),
        (
            "message = \"A.\"\neffect = { becomes = \"nonesuch\" }\n",
            "unknown terrain",
        ),
        (
            "message = \"A.\"\neffect = { objects = [{ object = \"nonesuch\", count = \"1\" }] }\n",
            "unknown object",
        ),
        (
            "message = \"A.\"\neffect = { explode = { spell = \"fireball\", radius = 99, \
             damage = \"1d6\" } }\n",
            "radius",
        ),
        (
            "message = \"A.\"\neffect = { wish = true }\n",
            "unknown field",
        ),
    ];
    for (extra, why) in cases {
        let text = outcome(extra);
        let e = parse(&text).expect_err(&text).to_string();
        assert!(e.contains(why), "{extra}: {e}");
    }
}
