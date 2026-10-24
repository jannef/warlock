//! Class level tables (spec §8 "Class abilities"): what each class's levels give, as the rules
//! say it, and the levels of its bonus talent picks.

use super::talents::effect_text;
use std::fmt::Write as _;
use warlock_core::content::{ClassDef, Content};

/// Every class with a level table, by name: its bonus picks, then its abilities by level (name,
/// whether it is used from the abilities menu, and what it does as the rule).
pub(super) fn class_levels(content: &Content) -> String {
    let book = content.talents();
    let mut classes: Vec<&ClassDef> = (content.classes().iter())
        .filter(|c| !c.abilities.is_empty() || !c.bonus_levels.is_empty())
        .collect();
    classes.sort_by(|a, b| a.name.cmp(&b.name));
    if classes.is_empty() {
        return "No class has a level table yet.\n".into();
    }
    let mut out = String::new();
    for class in classes {
        let _ = writeln!(out, "### {}\n", class.name);
        if !class.bonus_levels.is_empty() {
            let trees: Vec<&str> = (class.bonus_trees.iter())
                .map(|&t| book.tree(t).name.as_str())
                .collect();
            let levels: Vec<String> = class.bonus_levels.iter().map(u8::to_string).collect();
            let _ = writeln!(
                out,
                "Bonus talent from {} at levels {}.\n",
                trees.join(" or "),
                levels.join(", ")
            );
        }
        if class.abilities.is_empty() {
            continue;
        }
        out.push_str("| Level | Ability | Kind | What it does |\n|---:|---|---|---|\n");
        let mut abilities: Vec<_> = class.abilities.iter().collect();
        abilities.sort_by(|a, b| (a.level, &a.name).cmp(&(b.level, &b.name)));
        for a in abilities {
            let kind = if a.active().is_some() {
                "ability"
            } else {
                "passive"
            };
            let mut rule: Vec<String> = a.effects.iter().map(|e| effect_text(content, e)).collect();
            if let Some(required) = a.requires_alignment {
                rule.push(format!("only while you are {}", required.words()));
            }
            let _ = writeln!(
                out,
                "| {} | {} | {kind} | {} |",
                a.level,
                a.name,
                rule.join("; ")
            );
        }
        out.push('\n');
    }
    out
}
