//! The witch's familiar (spec §8 "Allies"; `data/familiars.toml`): the starting beasts by race and
//! the rares with their odds, the growth table of the small beasts only a familiar is, and the
//! rules in words.

use super::companions::{growth_table, starts_table};
use super::util::{Term, at_normal_pace, link};
use std::fmt::Write as _;
use warlock_core::content::{ActionKind, BASE_TURN, Content, CreatureId, price};

/// Every race's starting beasts, then the rares with the odds.
pub(super) fn starts(content: &Content) -> String {
    let Some(rules) = content.familiars() else {
        return String::new();
    };
    starts_table(content, &rules.start, "witch", "")
}

/// The growth table of the beasts a familiar may be that no one tames: the rest are in
/// the companions' table.
pub(super) fn growth(content: &Content) -> String {
    let Some(rules) = content.familiars() else {
        return String::new();
    };
    let mut beasts: Vec<CreatureId> = (rules.every_kind().into_iter())
        .filter(|&id| !content.creature(id).tameable)
        .collect();
    beasts.sort_by_key(|&id| {
        let def = content.creature(id);
        (def.level, def.name.clone())
    });
    growth_table(content, &beasts)
}

/// The familiar's rules in words and numbers.
pub(super) fn rules(content: &Content) -> String {
    let Some(r) = content.familiars() else {
        return String::new();
    };
    let spell = content.spell(r.touch);
    let status = spell
        .effect
        .status
        .map(|(status, _)| content.status(status));
    let bargain: Vec<String> = (r.bargain_kinds.iter())
        .map(|&id| {
            let def = content.creature(id);
            link(Term::Creature, &def.id, &def.name)
        })
        .collect();
    let turn = at_normal_pace(price(
        content.action_costs().cost(ActionKind::CallFamiliar),
        BASE_TURN,
    ));
    let rows = [
        (
            "The start",
            format!(
                "a witch begins with a beast of her race's list, or at {} in a thousand one of \
                 the rares",
                r.start.rare_per_mille
            ),
        ),
        (
            "The touch",
            format!(
                "its melee hit on a creature that lives lays {} at the degrees she would cast {} \
                 now, and she is its giver",
                status.map_or(String::new(), |s| link(Term::Status, &s.id, &s.name)),
                spell.name,
            ),
        ),
        (
            "Her other hexes",
            "each on the creature it hit starts afresh at its own degree, when it is one that \
             stacks by refreshing"
                .to_string(),
        ),
        (
            "The calling",
            format!(
                "{} turns of sitting, each {turn}, and nothing else; the beast is drawn as at \
                 the start, by chance",
                r.call_turns
            ),
        ),
        (
            "The bargain's",
            format!(
                "from level {} of her class a calling's draw holds {} as well, one more beast \
                 of her race's list; a start never does",
                r.bargain_level,
                bargain.join(", ")
            ),
        ),
        (
            "Her familiar's nose",
            format!(
                "with [Familiar's sight](#familiars-sight), it finds foes for her within {} cells, \
                 or its kind's Nose where that is farther",
                r.nose
            ),
        ),
        (
            "Names",
            format!("one of {}: {}", r.names.len(), r.names.join(", ")),
        ),
    ];
    let mut out = String::from("| Rule | What it is |\n|---|---|\n");
    for (name, text) in rows {
        let _ = writeln!(out, "| {name} | {text} |");
    }
    out
}
