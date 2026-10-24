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
                effects_text(content, def),
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
        let names: Vec<String> = (book.of_system(warlock_core::content::SystemId(i)))
            .map(|(_, t)| t.name.clone())
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

/// "Weapon focus (a weapon skill)" for a talent with a choice, else the name.
fn name(def: &TalentDef) -> String {
    match def.choice {
        Some(kind) => format!("{} ({})", def.name, kind.words()),
        None => def.name.clone(),
    }
}

/// The prerequisites: the talent's own, then each rank's where they differ ("Toughness 11 / 14
/// / 17"); "—" for none.
fn needs_text(content: &Content, def: &TalentDef) -> String {
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
fn effects_text(content: &Content, def: &TalentDef) -> String {
    let ranked = def.rank_count() > 1;
    let mut parts: Vec<String> = (def.effects.iter())
        .map(|e| {
            let text = effect_text(content, e);
            if ranked {
                format!("{text} per rank")
            } else {
                text
            }
        })
        .collect();
    for (i, rank) in def.ranks.iter().enumerate() {
        for effect in &rank.effects {
            parts.push(format!("rank {}: {}", i + 1, effect_text(content, effect)));
        }
    }
    parts.join("; ")
}

/// One effect as the rule.
fn effect_text(content: &Content, effect: &TalentEffect) -> String {
    let percent = |v: i32| format!("{} %", signed(v));
    let status = |id: &str| {
        (content.status_id(id)).map_or_else(|| id.to_string(), |s| content.status(s).name.clone())
    };
    match effect.clone() {
        TalentEffect::Dodge { value } => format!("Dodge {}", signed(value)),
        TalentEffect::Natural { value } => format!("natural armour {}", signed(value)),
        TalentEffect::MeleeDamage { value } => format!("melee damage {}", signed(value)),
        TalentEffect::MaxHpPerLevels {
            per_levels,
            at_least,
        } => format!("+1 maximum hit point per {per_levels} levels, at least +{at_least}"),
        TalentEffect::HealingPercent { value } => format!("healing pace {}", percent(value)),
        TalentEffect::FootstepsPercent { value } => format!("footsteps {}", percent(value)),
        TalentEffect::SilentDoors => "the doors you open and close are silent".to_string(),
        TalentEffect::Hearing { value } => format!("hears sounds {value} fainter"),
        TalentEffect::CarryPercent { value } => format!("carrying limit {}", percent(value)),
        TalentEffect::TravelPercent { value } => {
            format!("overworld journey time {}", percent(value))
        }
        TalentEffect::HungerPercent { value } => format!("hunger pace {}", percent(value)),
        TalentEffect::NoExertion => "blows, shots, and throws burn no nutrition".to_string(),
        TalentEffect::XpPercent { value } => format!("experience gained {}", percent(value)),
        TalentEffect::Notice { value } => {
            format!(
                "notices hidden things in passing, {} to the check",
                signed(value)
            )
        }
        TalentEffect::FreeBlow { trigger } => {
            use warlock_core::content::FreeBlowTrigger as T;
            match trigger {
                T::Cleave => {
                    "free blow: a melee blow that kills gives one at another hostile beside you"
                }
                T::GreatCleave => {
                    "free blow: cleaving goes on while the blows kill (once at each hostile beside \
                     you a turn)"
                }
                T::Riposte => "free blow: a blow that misses you by your Parry gives one back",
                T::Opportunist => "free blow: a hostile that steps away from beside you takes one",
            }
            .to_string()
        }
        TalentEffect::RestingPercent { value } => {
            format!("healing pace while resting {}", percent(value))
        }
        TalentEffect::SecondWind {
            per_level,
            at_least,
            cooldown,
        } => format!(
            "active (the time of a wait): {per_level} hit point per level back at once, at \
             least {at_least}; then {} (no second wind while it lasts)",
            status(&cooldown)
        ),
        TalentEffect::IronStomach => "rotten food never makes you vomit and sickens you half \
             as much (degrees halved, rounded down); overeating never makes you vomit"
            .to_string(),
        TalentEffect::StatusEases { status: id, value } => {
            format!("{} eases {value} % faster", status(&id))
        }
        TalentEffect::NoPenalty { status: id } => {
            format!("{} never lowers your scores", status(&id))
        }
        TalentEffect::Clotting => {
            "a big hit wounds you no more; a confirmed critical one degree less".to_string()
        }
        TalentEffect::PenaltyPercent { value } => format!(
            "what statuses take off your scores {} (each, rounded toward 0)",
            percent(value)
        ),
        TalentEffect::DiseaseChecks { value } => {
            format!("disease checks (catching, progressions) {}", signed(value))
        }
        TalentEffect::DiseaseRecovery { value } => format!(
            "a disease goes after {value} fewer checks won in a row (at least 1; never \
             one that does not recover)"
        ),
        TalentEffect::KickDie { sides } => format!("kicks at creatures roll 1d{sides}"),
        TalentEffect::DoorKick { value } => {
            format!("door kicks as if Strength {}", signed(value))
        }
        TalentEffect::DigPercent { value } => format!("digging work a turn {}", percent(value)),
        TalentEffect::TwoHandedPrice { value } => {
            format!("blows with a two-handed weapon cost {}", percent(value))
        }
        TalentEffect::CriticalStatus { status: id } => format!(
            "a critical you confirm in melee leaves the target {} (one degree)",
            status(&id)
        ),
        TalentEffect::Penetration { damage, value } => {
            let name = (content.damage_type_id(&damage)).map_or(damage.clone(), |d| {
                content.damage_type(d).name.to_lowercase()
            });
            format!(
                "blows that can do {name} damage: penetration {}",
                signed(value)
            )
        }
        TalentEffect::BurdenDex { value } => {
            format!("the burden's cap on Dexterity in Defence {}", signed(value))
        }
        TalentEffect::SureStep => "a trap you step onto first gets a Dexterity check against \
             its difficulty: made, it does not go off, and you find it"
            .to_string(),
        TalentEffect::PitProof => "pits never catch you (you find them)".to_string(),
        TalentEffect::SpeedPercent { value } => format!("speed {}", percent(value)),
        TalentEffect::MissileDodge { value } => {
            format!(
                "Dodge {} against shots, throws, and trap missiles",
                signed(value)
            )
        }
        TalentEffect::SpellDamagePercent { value } => format!(
            "damage from bursts and bolts (traps, scrolls) {} (rounded down)",
            percent(value)
        ),
    }
}
