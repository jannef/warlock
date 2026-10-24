//! Talents (spec §8 "Talents"): every tree with its talents, their prerequisites and their rules
//! in full (the game says them in words), and the coverage matrix by system.

use super::text::signed;
use std::fmt::Write as _;
use warlock_core::content::{Content, Need, Role, TalentDef, TalentEffect, TreeId};

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
        TalentEffect::ThresholdPercent { value } => {
            format!("the light your eyes need {}", percent(value))
        }
        TalentEffect::Darkvision { value } => format!("darkvision {} cells", signed(value)),
        TalentEffect::SeesUv => "your eyes see ultraviolet light".to_string(),
        TalentEffect::Scent { status: id, radius } => format!(
            "sees creatures that are {} in line of sight within {radius} cells, however dark",
            status(&id).to_lowercase()
        ),
        TalentEffect::NightSight { value } => {
            format!("overworld sight at night {} cells", signed(value))
        }
        TalentEffect::LanternLight { light } => {
            let reach = |id: &str| (content.light_id(id)).map_or(0, |l| content.light(l).radius);
            format!(
                "your lantern burns brighter: its light reaches {} cells",
                reach(&light)
            )
        }
        TalentEffect::Ambush { dice } => format!(
            "+{dice} damage, after Protection, on a hit at a creature that has not seen you \
             (it hunts no one it has seen, and does not see you now)"
        ),
        TalentEffect::CarryPercent { value } => format!("carrying limit {}", percent(value)),
        TalentEffect::TravelPercent { value } => {
            format!("overworld journey time {}", percent(value))
        }
        TalentEffect::TerrainAs { terrains, like } => {
            let name = |id: &str| {
                (content.terrain_id(id))
                    .map_or_else(|| id.to_string(), |t| content.terrain(t).name.clone())
            };
            let names: Vec<String> = terrains.iter().map(|t| name(t)).collect();
            format!(
                "a step onto {} costs what one onto {} does",
                names.join(", "),
                name(&like)
            )
        }
        TalentEffect::PlungeProof => "a plunge through the ice does you no harm".to_string(),
        TalentEffect::LightOnIce => {
            "ice holds you as if you carried nothing (your own weight only)".to_string()
        }
        TalentEffect::ClimbPercent { value } => {
            format!("climbing out onto ice costs {}", percent(value))
        }
        TalentEffect::WadingProof => "wading never gives you a disease".to_string(),
        TalentEffect::BeastFriend => "creatures of animal factions are neutral to you until you \
                                      attack one of that type"
            .to_string(),
        TalentEffect::Forecast { hours } => format!(
            "under the open sky you know the first change of weather within {hours} hours \
             (the status bar's weather)"
        ),
        TalentEffect::SunHealing { value } => format!(
            "healing pace {} under the open sun (sun up, no roof, whatever the clouds)",
            percent(value)
        ),
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
        TalentEffect::Weapon {
            role,
            to_hit,
            damage,
            threat,
            ranks,
            speed,
            reload,
            ease,
            bash,
            shield,
        } => {
            let with = match role {
                None => "the chosen skill".to_string(),
                Some(Role::Melee) => "every melee weapon skill".to_string(),
                Some(Role::Launcher) => "every missile weapon skill".to_string(),
                Some(Role::Thrown) => "throws".to_string(),
                Some(Role::Unarmed) => "unarmed fighting".to_string(),
                Some(Role::ShieldBlock) => "shield blocking".to_string(),
                Some(Role::ShieldBash) => "shield bashing".to_string(),
                Some(Role::Style) => "every fighting style".to_string(),
            };
            let mut gains = Vec::new();
            for (value, what) in [(to_hit, "to hit"), (damage, "damage"), (shield, "Shield")] {
                if value != 0 {
                    gains.push(format!("{what} {}", signed(value)));
                }
            }
            if threat > 0 {
                gains.push(format!("threat range +{threat}"));
            }
            if ranks > 0 {
                gains.push(format!("benefits as {ranks} rank higher (at most 10)"));
            }
            if speed > 0 {
                gains.push(format!("attacks {speed} % cheaper"));
            }
            if reload > 0 {
                gains.push(format!("reloading {reload} % cheaper"));
            }
            if ease > 0 {
                gains.push(format!("both two-weapon penalties eased by {ease}"));
            }
            if bash > 0 {
                gains.push(format!(
                    "a free shield bash's chance +{bash} % where there is one"
                ));
            }
            format!("{} with {with}", gains.join(", "))
        }
        TalentEffect::LauncherRange { value } => {
            format!("missile weapon range {} cells", signed(value))
        }
        TalentEffect::ThrowRange { value } => format!("throwing range {} cells", signed(value)),
        TalentEffect::MaxDex { value } => {
            format!("worn armour's Dexterity cap {}", signed(value))
        }
        TalentEffect::Deflection { value } => format!("Deflection {}", signed(value)),
        TalentEffect::Protection {
            value,
            body_armour,
            shield,
        } => {
            let mut text = format!("Protection {}", signed(value));
            if let Some(at) = body_armour {
                text += &format!(
                    " while the armour piece worn with the most Protection has {at} or more"
                );
            }
            if shield {
                text += " while holding a shield";
            }
            text
        }
        TalentEffect::StanceToHit { stance, percent } => {
            format!(
                "{} stance's to-hit penalty {percent} % smaller",
                stance.id()
            )
        }
        TalentEffect::FleePercent { value } => format!("fleeing steps' cost {}", percent(value)),
        TalentEffect::Pain { value } => {
            format!(
                "every hit on you {} damage after the soak, never below 1",
                signed(-value)
            )
        }
        TalentEffect::Resist { value } => format!("resistance to the chosen element +{value} %"),
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
