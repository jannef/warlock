//! The rank tables of the abjuration, enchantment, and restoration spells (#1269; spec §8
//! "Spells": "Ranks"): every rank from 2 to 10 adds something, no degree a rank adds is lost to
//! its status's most for a caster at the base severity, and rank 10 gains what each spell's
//! role wants (a roll on a heal or a damage spell, reach on an aimed one, a longer stay on a
//! call).

use warlock_core::content::{Content, MAX_SPELL_RANK, SpellData};

const SPELLS: [&str; 21] = [
    // Abjuration.
    "blessing",
    "fire_cloak",
    "glamour",
    "rebuke",
    "silence",
    "unbinding",
    // Enchantment.
    "beguile",
    "bind",
    "call_of_the_forest",
    "call_of_the_wild",
    "call_the_elements",
    "evil_eye",
    "fear",
    "mesmerise",
    "sleep",
    "slumber",
    // Restoration.
    "benison",
    "cure_wounds",
    "mend",
    "remove_disease",
    "renewal",
];

fn spell<'a>(content: &'a Content, id: &str) -> &'a SpellData {
    content.spell(content.spell_id(id).unwrap_or_else(|| panic!("{id}")))
}

#[test]
fn every_rank_from_two_to_ten_adds_something() {
    let content = Content::builtin().expect("content");
    for id in SPELLS {
        let ranks: Vec<u8> = spell(&content, id)
            .ranks
            .steps()
            .iter()
            .map(|(r, _)| *r)
            .collect();
        assert_eq!(ranks, (2..=MAX_SPELL_RANK).collect::<Vec<_>>(), "{id}");
    }
}

#[test]
fn no_degree_a_rank_adds_is_lost_to_the_status_most() {
    let content = Content::builtin().expect("content");
    for id in SPELLS {
        let data = spell(&content, id);
        let Some((status, base)) = data.effect.status else {
            continue;
        };
        let most = content.status(status).max;
        let added = data.ranks.at(MAX_SPELL_RANK).degrees;
        let at_ten = i32::from(base) + added;
        assert!(at_ten <= i32::from(most), "{id}: {at_ten} > {most}");
    }
}

#[test]
fn rank_ten_gains_what_the_spell_is_for_and_is_never_free() {
    let content = Content::builtin().expect("content");
    for id in SPELLS {
        let data = spell(&content, id);
        let ten = data.ranks.at(MAX_SPELL_RANK);
        // Cheaper and quicker, never by more than half the mana or 60 percent of the time.
        assert!((-50..=-10).contains(&ten.cost), "{id}: cost {}", ten.cost);
        assert!((-60..=-20).contains(&ten.time), "{id}: time {}", ten.time);
        if data.effect.heal.is_some() {
            assert!(ten.bonus == 3 && ten.dice == 1 && ten.reroll == 1, "{id}");
        }
        if data.effect.summon.is_some() || data.effect.charm.is_some() {
            assert!(ten.degrees >= 8, "{id}: a call or charm stays longer");
        }
    }
    // The aimed ones reach farther; Rebuke burns harder.
    for id in [
        "beguile",
        "bind",
        "evil_eye",
        "fear",
        "mesmerise",
        "silence",
        "sleep",
    ] {
        assert_eq!(
            spell(&content, id).ranks.at(MAX_SPELL_RANK).cells,
            3,
            "{id}"
        );
    }
    let rebuke = spell(&content, "rebuke").ranks.at(MAX_SPELL_RANK);
    assert_eq!((rebuke.bonus, rebuke.dice, rebuke.reroll), (2, 1, 1));
    assert_eq!((rebuke.cells, rebuke.radius_cells), (2, 1));
}
