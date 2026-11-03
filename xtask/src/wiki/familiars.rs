//! The witch's familiar (spec §8 "Allies"; `data/familiars.toml`): the starting beasts by race and
//! the rares with their odds, the growth table of the small beasts only a familiar is, and the
//! rules in words.

use super::companions::{beast, growth_table};
use super::text::{Term, link};
use std::fmt::Write as _;
use warlock_core::content::{Content, CreatureId};

/// Every race's starting beasts, then the rares with the odds.
pub(super) fn starts(content: &Content) -> String {
    let Some(rules) = content.familiars() else {
        return String::new();
    };
    let mut out = String::from("| Race | Starting beasts |\n|---|---|\n");
    for (race, list) in &rules.by_race {
        let name = content.race(race).map_or(race.clone(), |r| r.name.clone());
        let beasts: Vec<String> = list.iter().map(|&b| beast(content, b)).collect();
        let _ = writeln!(
            out,
            "| {} | {} |",
            link(Term::Race, race, &name),
            beasts.join(", ")
        );
    }
    let rares: Vec<String> = rules.rare.iter().map(|&b| beast(content, b)).collect();
    let _ = write!(
        out,
        "\nAt {} in a thousand ({:.1} %), any race's witch begins with one of the rares instead, \
         each as likely: {}. The beast's level is in brackets.\n",
        rules.rare_per_mille,
        f64::from(rules.rare_per_mille) / 10.0,
        rares.join(", ")
    );
    out
}

/// The growth table of the beasts a familiar may be that no one tames: the rest are in
/// the companions' table.
pub(super) fn growth(content: &Content) -> String {
    let Some(rules) = content.familiars() else {
        return String::new();
    };
    let mut beasts: Vec<CreatureId> = (rules.every_start().into_iter())
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
    let rows = [
        (
            "The start",
            format!(
                "a witch begins with a beast of her race's list, or at {} in a thousand one of \
                 the rares",
                r.rare_per_mille
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
