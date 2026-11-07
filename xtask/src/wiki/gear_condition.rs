//! The condition creature gear spawns in (spec §8 "Creature gear"; #1193): the odds by tribe from
//! `data/gear_condition.toml`, exactly, and what damaged gear of each material is called.

use std::fmt::Write as _;
use warlock_core::content::{Content, ItemHarm, Odds, PER_MILLE};

/// A per mille chance as a percentage, without trailing zeros.
fn percent(per_mille: u32) -> String {
    let text = format!("{:.1}", f64::from(per_mille) / 10.0);
    let text = text.strip_suffix(".0").unwrap_or(&text);
    format!("{text}%")
}

/// One row of odds.
fn row(out: &mut String, who: &str, odds: &Odds) {
    let bands = [odds.damaged, odds.fine, odds.marvellous, odds.masterwork].map(u32::from);
    let made = PER_MILLE - bands.iter().sum::<u32>();
    let _ = write!(out, "| {who} |");
    for n in bands.into_iter().chain([made]) {
        let _ = write!(out, " {} |", percent(n));
    }
    out.push('\n');
}

/// The odds of each tribe (by creature tag, sorted by it) and of everyone else, then the order a
/// creature of several tribes takes them in, then the words by material.
pub(super) fn gear_condition(content: &Content) -> String {
    let rules = content.gear_condition();
    let mut out = String::from(
        "| Creatures | Damaged | Fine | Marvellous | Masterwork | As made |\n\
         |---|:---:|:---:|:---:|:---:|:---:|\n",
    );
    let mut tribes: Vec<_> = rules.tribes.iter().collect();
    tribes.sort_by(|a, b| a.tag.cmp(&b.tag));
    for tribe in tribes {
        row(&mut out, &format!("`{}`", tribe.tag), &tribe.odds);
    }
    row(&mut out, "Everyone else", &rules.default);
    let order: Vec<String> = (rules.tribes.iter())
        .map(|t| format!("`{}`", t.tag))
        .collect();
    let _ = writeln!(
        out,
        "\nA creature with more than one of these tags takes the first in this order: {}.",
        order.join(", ")
    );
    let mut groups: Vec<_> = rules.kinds.iter().collect();
    groups.sort_by(|a, b| a.0.cmp(&b.0));
    for (group, kinds) in groups {
        let words: Vec<String> = (kinds.iter())
            .map(|k| format!("*{}* ({})", k.word(), kind_of(*k)))
            .collect();
        let _ = writeln!(
            out,
            "\nDamaged gear of {group} is {}, whichever its material withstands less (the first \
             on a tie), never one it always withstands; one that always withstands them all is \
             plainly *damaged*. Gear of anything else is plainly *damaged*.",
            words.join(" or "),
        );
    }
    out
}

/// What the kind is, in words.
fn kind_of(harm: ItemHarm) -> &'static str {
    match harm {
        ItemHarm::Acid => "corrosion",
        ItemHarm::Rust => "rust",
        ItemHarm::Fire => "heat",
        ItemHarm::Work => "a failed attempt",
    }
}
