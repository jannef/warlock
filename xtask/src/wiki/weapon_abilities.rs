//! The weapon abilities tables (spec §8 "Weapon skills", "Fighting styles"; the combat
//! progression design §4, §6), from `data/weapon_abilities/`: what each rank teaches, the weapon
//! skills' on the Weapon skills page and the styles' on the Fighting styles page, with the rank,
//! the kind, the trigger, exactly what it does, and its costs (a triggered ability has none).

use super::text::table_text;
use std::fmt::Write as _;
use warlock_core::content::{
    Content, ExtraDice, FreeBlowKind, On, WeaponAbility, WeaponDoes, WeaponSkillId,
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
    for (_, rank, a) in rows {
        let taught: Vec<&str> = (a.skills.iter())
            .map(|&s| skills.skill(s).name.as_str())
            .collect();
        let fires = if a.triggers.is_empty() {
            switch_trigger(a.effect)
        } else {
            let when: Vec<&str> = a.triggers.iter().map(|t| t.describe()).collect();
            when.join("; or ")
        };
        let _ = writeln!(
            out,
            "| {} | {} | {rank} | triggered | {} | {} | — | none (free and instant) |",
            a.name,
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
    }
}
