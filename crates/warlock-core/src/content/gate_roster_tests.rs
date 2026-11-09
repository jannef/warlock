//! The gates (#1295, docs/dev/creatures/gates.md; design `docs/design/miscasts-review.md` §6.2):
//! nine rooted, `loosed` summoners with no attack and no mana, met only through a miscast, each
//! opened by the one miscast row of its id.

use super::{Content, MishapEffect};

/// The gates by id with the fiends each lets through, as the design's table has them.
const GATES: &[(&str, &[&str])] = &[
    ("rift", &["dretch", "quasit"]),
    ("hellmouth", &["lemure", "imp"]),
    ("gloom_gate", &["hell_hound", "nightmare", "mezzoloth"]),
    ("abyssal_gate", &["balgura", "chasme", "hezrou", "vrock"]),
    (
        "iron_gate",
        &["bearded_devil", "barbed_devil", "bone_devil", "erinyes"],
    ),
    (
        "black_gate",
        &["incubus", "succubus", "canoloth", "night_hag", "nycaloth"],
    ),
    (
        "dire_gate",
        &[
            "nycaloth",
            "erinyes",
            "cambion",
            "marilith",
            "chain_devil",
            "glabrezu",
        ],
    ),
    (
        "arch_gate",
        &[
            "arcanaloth",
            "horned_devil",
            "nalfeshnee",
            "rakshasa",
            "ice_devil",
            "ultroloth",
        ],
    ),
    ("lords_gate", &["balor", "pit_fiend"]),
];

#[test]
fn every_gate_is_a_rooted_loosed_summoner_of_its_fiends_at_no_cost() {
    let content = Content::builtin().expect("content");
    let loosed = content.factions().id("loosed").expect("the loosed");
    let tagged: Vec<_> = content.tagged("gate").collect();
    assert_eq!(tagged.len(), GATES.len(), "the nine gates carry the tag");
    for &(id, pool) in GATES {
        let c = content.creature_id(id).expect(id);
        assert!(tagged.contains(&c), "{id}: tagged `gate`");
        let def = content.creature(c);
        assert!(def.rooted, "{id}: rooted");
        assert!(def.left_alone, "{id}: left alone by other creatures");
        assert_eq!(def.faction, Some(loosed), "{id}: loosed");
        assert!(
            def.attack.is_none() && def.more_attacks.is_empty(),
            "{id}: no attack"
        );
        assert!(!def.stray && !def.unique && !def.corpse, "{id}");
        let summon = def.summon.as_ref().expect("a summoning");
        assert_eq!(summon.mana, 0, "{id}: it costs nothing");
        let named: Vec<_> = pool
            .iter()
            .map(|p| content.creature_id(p).expect(p))
            .collect();
        assert_eq!(summon.pool, named, "{id}: its pool");
        // Every one it lets through a fiend.
        for &f in &summon.pool {
            let fiend = content.creature(f);
            assert!(
                fiend.tags.iter().any(|t| t == "fiend"),
                "{id}: {}",
                fiend.id
            );
        }
    }
}

#[test]
fn each_gate_is_opened_by_its_one_unprotected_miscast_row() {
    let content = Content::builtin().expect("content");
    let safe_below = content
        .miscast_rules()
        .expect("the miscast rules")
        .safe_below;
    for &(id, _) in GATES {
        let gate = content.creature_id(id).expect(id);
        let rows: Vec<_> = (content.miscasts().iter())
            .filter(|m| matches!(m.effect, MishapEffect::Summon { creature } if creature == gate))
            .collect();
        assert_eq!(rows.len(), 1, "{id}: one row opens it");
        assert_eq!(rows[0].id, id, "{id}: the row of its id");
        assert!(rows[0].from >= safe_below, "{id}: above the protected half");
    }
}
