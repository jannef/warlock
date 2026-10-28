//! The weapon abilities tables (spec §8 "Weapon skills", "Fighting styles"; the combat
//! progression design §4, §6), from `data/weapon_abilities/`: what each rank teaches, the weapon
//! skills' on the Weapon skills page and the styles' on the Fighting styles page, with the rank,
//! the kind, the trigger, exactly what it does, and its costs (a triggered ability has none).

use super::text::{Term, anchored, link, table_text};
use std::fmt::Write as _;
use warlock_core::content::{
    ActiveAim, Content, ExtraDice, FreeBlowKind, On, WeaponAbility, WeaponDoes, WeaponSkillId,
};

/// The weapon skills' abilities.
pub(super) fn weapon_abilities(content: &Content) -> String {
    table(content, false)
}

/// The fighting styles' abilities.
pub(super) fn style_abilities(content: &Content) -> String {
    table(content, true)
}

/// The abilities of the styles (`styles`) or of the weapon skills, by their first skill in file
/// order, then rank, then name.
fn table(content: &Content, styles: bool) -> String {
    let skills = content.weapon_skills();
    let mut rows: Vec<(WeaponSkillId, u8, &WeaponAbility)> = (content.weapon_abilities().iter())
        .filter(|a| (a.skills.first()).is_some_and(|&s| skills.skill(s).is_style() == styles))
        .map(|a| (a.skills[0], a.rank, a))
        .collect();
    rows.sort_by(|a, b| (a.0, a.1, &a.2.name).cmp(&(b.0, b.1, &b.2.name)));
    let mut out = String::from(
        "| Ability | Taught by | Rank | Kind | Fires on | What it does | Stamina | Time |\n\
         |---|---|---:|---|---|---|---:|---|\n",
    );
    let (term, skill_term) = if styles {
        (Term::StyleAbility, Term::Style)
    } else {
        (Term::WeaponAbility, Term::WeaponSkill)
    };
    for (_, rank, a) in rows {
        let taught: Vec<String> = (a.skills.iter())
            .map(|&s| skills.skill(s))
            .map(|s| link(skill_term, &s.id, &s.name))
            .collect();
        let name = anchored(term, &a.id, &a.name);
        if let Some(active) = a.active {
            let aim = match active.aim {
                ActiveAim::Nothing => "used (aimed at nothing)",
                ActiveAim::Direction => "used (aimed in a direction)",
                ActiveAim::Cell => "used (aimed at a cell)",
            };
            let action = match active.action.id() {
                "melee" => "a blow",
                "shoot" => "a shot",
                "wait" => "a wait",
                other => other,
            };
            let _ = writeln!(
                out,
                "| {name} | {} | {rank} | active | {aim} | {} | {} | {}% of {action} |",
                taught.join(", "),
                table_text(&does(content, a)),
                active.stamina,
                active.percent,
            );
            continue;
        }
        let fires = if a.triggers.is_empty() {
            switch_trigger(a.effect)
        } else {
            let when: Vec<&str> = a.triggers.iter().map(|t| t.describe()).collect();
            when.join("; or ")
        };
        let _ = writeln!(
            out,
            "| {name} | {} | {rank} | triggered | {} | {} | — | none (free and instant) |",
            taught.join(", "),
            table_text(&fires),
            table_text(&does(content, a)),
        );
    }
    out
}

/// When a switch's free blow (or parry) comes, in words.
fn switch_trigger(effect: WeaponDoes) -> String {
    match effect {
        WeaponDoes::FreeBlow(kind) => match kind {
            FreeBlowKind::Riposte => "a melee attack on you misses by your Parry",
            FreeBlowKind::Cleave => "a blow of your melee attack kills",
            FreeBlowKind::GreatCleave => {
                "a blow of your melee attack kills, and again while they kill"
            }
            FreeBlowKind::Opportunist => "a hostile steps away from beside you",
            FreeBlowKind::Gatekeeper => "a hostile steps up beside you",
        },
        WeaponDoes::ParryMissiles => "always (a shot or a throw at you)",
        _ => "",
    }
    .into()
}

/// What ability `a`'s effect does, exactly.
fn does(content: &Content, a: &WeaponAbility) -> String {
    let defending = a.triggers.iter().any(|t| t.defending());
    match a.effect {
        WeaponDoes::FreeBlow(kind) => {
            let what = match kind {
                FreeBlowKind::Riposte => "a free blow back at the attacker once its attack is over",
                FreeBlowKind::Cleave => "a free blow at the next hostile beside you, clockwise",
                FreeBlowKind::GreatCleave => {
                    "a free blow at the next hostile beside you, clockwise, as long as they kill"
                }
                FreeBlowKind::Opportunist => "a free blow at it as it turns to go",
                FreeBlowKind::Gatekeeper => {
                    "a free blow at it as it comes (Brace of steel makes it surer, and a hit \
                     staggers: see Talents)"
                }
            };
            format!("{what}; a free blow brings none of its own")
        }
        WeaponDoes::Status {
            status,
            severity,
            turns,
            on,
        } => {
            let name = &content.status(status).name;
            let whom = match on {
                On::Target if defending => "your attacker",
                On::Target => "the one you struck",
                On::Own => "you",
            };
            let degrees = match turns {
                Some(dice) => format!("{dice} degrees (one roll)"),
                None if severity == 1 => "1 degree".into(),
                None => format!("{severity} degrees"),
            };
            format!("{name} on {whom}, {degrees}")
        }
        WeaponDoes::ExtraDamage { dice, kind } => {
            let dice = match dice {
                ExtraDice::MainHand => "the main hand's damage dice".to_string(),
                ExtraDice::OffHand => "the off hand's damage dice".to_string(),
                ExtraDice::Fixed(d) => d.to_string(),
            };
            let kind = kind.map_or_else(
                || "of the blow's kind".to_string(),
                |k| format!("of {}", content.damage_type(k).name.to_lowercase()),
            );
            format!(
                "{dice} rolled once more (no bonus) on the target, {kind}: resistances count, \
                 Protection does not (with Second blade's rend, the off hand's dice too)"
            )
        }
        WeaponDoes::FreeShot => "a free shot with your missile weapon and the ammunition you have \
                                 ready, else a free throw of what you have ready to throw; nothing \
                                 without"
            .into(),
        WeaponDoes::ParryMissiles => {
            "your Parry counts against shots and throws too (without it, Parry counts against \
             blows only)"
                .into()
        }
        WeaponDoes::ReflectSpell { per_turn } => {
            let times = if per_turn == 1 {
                "once".to_string()
            } else {
                format!("{per_turn} times")
            };
            format!(
                "the spell is cast back at its caster from your cell, as they cast it (their \
                 caster level, cap, and to-hit), rolling its own attack against their Defence; \
                 it costs them no more mana and you nothing; at most {times} a turn of yours, and \
                 a spell turned back is never turned back again. Bursts, cones, lines, and hexes \
                 roll no attack and are never turned back, nor a spell that missed you by your \
                 Parry or Dexterity"
            )
        }
        WeaponDoes::ReachBlow { reach } => format!(
            "a blow at the first foe within {reach} cells along the direction, every cell before \
             it empty and passable (a closed door blocks; no trap is sprung); a foe beyond the \
             first cell brings no free blows"
        ),
        WeaponDoes::SureCritical => {
            "a blow that is a critical if it hits, with no confirmation roll".into()
        }
        WeaponDoes::IgnoreDefence { categories } => {
            let names: Vec<&str> = categories.iter().map(|c| c.name()).collect();
            format!(
                "a blow rolled against the foe's Defence without its {} bonuses",
                names.join(" and ")
            )
        }
        WeaponDoes::IgnoreProtection => {
            "Protection soaks none of it (the profile's penetration is above any Protection)".into()
        }
        WeaponDoes::Sweep { to_hit } => format!(
            "a blow at every hostile beside you, clockwise from the north (or the direction \
             given), each at {to_hit:+} to hit and with the free blows the end of an attack \
             brings; with a weapon in each hand the main and off hand take turns"
        ),
        WeaponDoes::PierceShot => {
            "a shot that, when it hits, flies on along its line to the next one there, which it \
             attacks as a missed shot would (once)"
                .into()
        }
        WeaponDoes::Knockback {
            cells,
            bash,
            status,
        } => {
            let blow = if bash { "a shield bash" } else { "a blow" };
            let then = status.map_or_else(String::new, |s| {
                format!(", then {} (1 degree)", content.status(s).name)
            });
            format!(
                "{blow}; a foe it hits is driven {cells} cell back if that cell is free \
                 (walkable, no trap, no ice, nobody there){then}; into a wall or another it \
                 stays, and the blow lands all the same"
            )
        }
        WeaponDoes::Buff { status, severity } => {
            let def = content.status(status);
            format!("{} on you, {severity} degrees", def.name)
        }
        WeaponDoes::DoubleDice { to_hit } => format!(
            "a blow at {to_hit:+} to hit whose damage dice are rolled twice (its bonus and riders \
             once; it is no critical)"
        ),
    }
}
