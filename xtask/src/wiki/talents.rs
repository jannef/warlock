//! Talents (spec §8 "Talents"): every tree with its talents, their prerequisites and their rules
//! in full (the game says them in words), and the coverage matrix by system.

use super::text::signed;
use std::fmt::Write as _;
use warlock_core::content::{Content, Need, TalentDef, TalentEffect, TreeId};

/// Every tree in display order: a heading, its description, and its talents (name, ranks, what
/// each rank needs, what it does as the rule, and the systems it touches).
pub(super) fn talents(content: &Content) -> String {
    let book = content.talents();
    let mut out = String::new();
    for (i, tree) in (0u8..).zip(&book.trees) {
        let _ = writeln!(out, "### {}\n\n{}\n", tree.name, tree.description);
        let rows: Vec<&TalentDef> = book.of_tree(TreeId(i)).map(|(_, t)| t).collect();
        if rows.is_empty() {
            out.push_str("No talents yet.\n\n");
            continue;
        }
        out.push_str("| Talent | Ranks | Needs | Effect | Systems |\n|---|---:|---|---|---|\n");
        for def in rows {
            let systems: Vec<&str> = (def.systems.iter())
                .map(|s| book.system(*s).name.as_str())
                .collect();
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} |",
                name(def),
                def.rank_count(),
                needs_text(content, def),
                effects_text(def),
                systems.join(", ")
            );
        }
        out.push('\n');
    }
    out
}

/// The coverage matrix: every system of the closed list with the talents that touch it, and
/// whether the bar (`[coverage]` in `data/talents.toml`) already holds it.
pub(super) fn by_system(content: &Content) -> String {
    let book = content.talents();
    let bar = book.coverage;
    let mut out = String::from("| System | Talents | Held to the bar |\n|---|---|---|\n");
    for (i, system) in (0u8..).zip(&book.systems) {
        // A birth sign's gift is marked: it is given, never picked.
        let names: Vec<String> = (book.of_system(warlock_core::content::SystemId(i)))
            .map(|(_, t)| {
                if t.granted {
                    format!("{} (birth sign)", t.name)
                } else {
                    t.name.clone()
                }
            })
            .collect();
        let held = if system.pending {
            "not yet (pending)".to_string()
        } else {
            format!("yes ({} talents at least)", bar.min_talents)
        };
        let talents = if names.is_empty() {
            "—".to_string()
        } else {
            names.join(", ")
        };
        let _ = writeln!(out, "| {} | {talents} | {held} |", system.name);
    }
    out
}

/// "Weapon focus (a weapon skill)" for a talent with a choice, "Slight (flaw)" for a birth
/// sign's flaw, else the name.
fn name(def: &TalentDef) -> String {
    match def.choice {
        Some(kind) => format!("{} ({})", def.name, kind.words()),
        None if def.flaw => format!("{} (flaw)", def.name),
        None => def.name.clone(),
    }
}

/// The prerequisites: the talent's own, then each rank's where they differ ("Toughness 11 / 14
/// / 17"); "—" for none; a granted talent is never picked.
fn needs_text(content: &Content, def: &TalentDef) -> String {
    if def.granted {
        return "given by a birth sign, never picked".into();
    }
    let words = |needs: &[Need]| -> String {
        (needs.iter())
            .map(|n| n.words(content))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let common: Vec<Need> = def.needs.iter().collect();
    let per_rank: Vec<Vec<Need>> = (def.ranks.iter())
        .map(|r| r.needs.iter().collect())
        .collect();
    let mut parts = Vec::new();
    if !common.is_empty() {
        parts.push(words(&common));
    }
    if per_rank.iter().any(|r| !r.is_empty()) {
        let ranks: Vec<String> = (per_rank.iter().enumerate())
            .map(|(i, r)| {
                if r.is_empty() {
                    format!("{}: —", i + 1)
                } else {
                    format!("{}: {}", i + 1, words(r))
                }
            })
            .collect();
        parts.push(ranks.join("; "));
    }
    if parts.is_empty() {
        "—".to_string()
    } else {
        parts.join("; ")
    }
}

/// What the talent does, as the rule: its per-rank effects ("per rank" when it has several
/// ranks), then any rank's own.
pub(super) fn effects_text(def: &TalentDef) -> String {
    let ranked = def.rank_count() > 1;
    let mut parts: Vec<String> = (def.effects.iter())
        .map(|e| {
            let text = effect_text(*e);
            if ranked {
                format!("{text} per rank")
            } else {
                text
            }
        })
        .collect();
    for (i, rank) in def.ranks.iter().enumerate() {
        for effect in &rank.effects {
            parts.push(format!("rank {}: {}", i + 1, effect_text(*effect)));
        }
    }
    parts.join("; ")
}

/// One effect as the rule.
fn effect_text(effect: TalentEffect) -> String {
    let percent = |v: i32| format!("{} %", signed(v));
    match effect {
        TalentEffect::Dodge { value } => format!("Dodge {}", signed(value)),
        TalentEffect::Natural { value } => format!("natural armour {}", signed(value)),
        TalentEffect::MaxDex { value } => {
            format!("the burden's cap on Dexterity in Defence {}", signed(value))
        }
        TalentEffect::Protection { value } => format!("Protection {}", signed(value)),
        TalentEffect::MeleeDamage { value } => format!("melee damage {}", signed(value)),
        TalentEffect::Threat { value } => {
            format!("threat range {} with every attack", signed(value))
        }
        TalentEffect::RerollFumble => "a natural 2 on the attack roll is rolled again, once".into(),
        TalentEffect::MaxHpPerLevels {
            per_levels,
            at_least,
        } => format!("+1 maximum hit point per {per_levels} levels, at least +{at_least}"),
        TalentEffect::HealingPercent { value } => format!("healing pace {}", percent(value)),
        TalentEffect::Ease { of, times } => format!("{} eases {times}× as fast", of.words()),
        TalentEffect::PoisonSparesScores => "poison never lowers your scores".into(),
        TalentEffect::DiseaseChecks { value } => {
            format!("disease checks {}", signed(value))
        }
        TalentEffect::FootstepsPercent { value } => format!("footsteps {}", percent(value)),
        TalentEffect::SilentDoors => "the doors you open and close are silent".to_string(),
        TalentEffect::Hearing { value } => format!("hears sounds {value} fainter"),
        TalentEffect::CarryPercent { value } => format!("carrying limit {}", percent(value)),
        TalentEffect::TravelPercent { value } => {
            format!("overworld journey time {}", percent(value))
        }
        TalentEffect::Swimmer => {
            "wading (water, marsh) costs a plain step and never gives a disease".into()
        }
        TalentEffect::SpeedPercent { value } => format!("speed {}", percent(value)),
        TalentEffect::HungerPercent { value } => format!("hunger pace {}", percent(value)),
        TalentEffect::RotHarmPercent { value } => {
            format!(
                "sickness from rotten food {} (rounded down)",
                percent(value)
            )
        }
        TalentEffect::NoExertion => "blows, shots, and throws burn no nutrition".to_string(),
        TalentEffect::XpPercent { value } => format!("experience gained {}", percent(value)),
        TalentEffect::Notice { value } => {
            format!(
                "notices hidden things in passing, {} to the check",
                signed(value)
            )
        }
        TalentEffect::StoneSense => "notices hidden things on stone in passing".into(),
        TalentEffect::Search { value } => {
            format!("{} to every check to find hidden things", signed(value))
        }
        TalentEffect::SearchReach { value } => format!(
            "searching reaches {value} cell{} farther",
            if value == 1 { "" } else { "s" }
        ),
        TalentEffect::Kick { value } => format!("door kicks as if Strength {}", signed(value)),
        TalentEffect::DigPercent { value } => format!("digging work {}", percent(value)),
        TalentEffect::FavourPercent { value } => format!("shop favour {}", percent(value)),
        TalentEffect::FreeBlow { trigger } => format!("free blow: {trigger:?}").to_lowercase(),
    }
}
