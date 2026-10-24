//! Damage types (spec §8 "Damage types"): the kinds, what each weapon does, and what armour
//! stands against or resists. Never a creature's resistances: the player learns those by play.

use super::order;
use super::text::{capital, yes};
use std::fmt::Write as _;
use warlock_core::content::{Content, DamageTypeId};

/// The damage types, in content order: whether armour (Protection) soaks the kind, whether its
/// big hits and criticals bleed, and how a blow of it is said.
pub(super) fn damage_types(content: &Content) -> String {
    let mut out = String::from(
        "| Kind | Armour soaks it | Wounds bleed | A blow of it |\n|---|:---:|:---:|---|\n",
    );
    let bleeds: Vec<DamageTypeId> = (content.status_defs().iter())
        .filter_map(|s| s.wound.as_ref())
        .flat_map(|w| w.kinds.iter().copied())
        .collect();
    let any_wound_bleeds_all = (content.status_defs().iter())
        .filter_map(|s| s.wound.as_ref())
        .any(|w| w.kinds.is_empty());
    for (i, t) in content.damage_types().iter().enumerate() {
        let id = DamageTypeId(i as u8);
        let bleed = any_wound_bleeds_all || bleeds.contains(&id);
        let _ = writeln!(
            out,
            "| {} | {} | {} | \"The kobold {} you.\" |",
            capital(&t.name),
            yes(t.physical),
            yes(bleed),
            t.verb
        );
    }
    if let Some(unarmed) = content.unarmed_damage() {
        let _ = writeln!(
            out,
            "\nBare hands, kicks, and shield bashes do {} damage.",
            content.damage_type(unarmed).name
        );
    }
    out
}

/// The kinds each weapon, missile weapon, and piece of ammunition does, and what it adds:
/// "slashing or piercing" for one that does the better of two.
pub(super) fn weapon_kinds(content: &Content) -> String {
    let mut out = String::from("| Weapon | Damage | Adds |\n|---|---|---|\n");
    for (_, o) in order::objects(content) {
        let kinds = match &o.attack {
            Some(attack) => {
                let names: Vec<&str> = (attack.kinds.iter())
                    .map(|k| content.damage_type(k).name.as_str())
                    .collect();
                names.join(" or ")
            }
            None => String::new(),
        };
        let added = o
            .added
            .map(|r| format!("{} {}", r.dice, content.damage_type(r.kind).name));
        if kinds.is_empty() && added.is_none() {
            continue;
        }
        let verb = (o.verb.as_deref()).map_or(String::new(), |v| format!(" (\"{v}\")"));
        let _ = writeln!(
            out,
            "| {} | {}{verb} | {} |",
            capital(&o.name),
            if kinds.is_empty() {
                "—".to_string()
            } else {
                kinds
            },
            added.unwrap_or_else(|| "—".into())
        );
    }
    out
}

/// What armour stands better or worse against, and what worn things resist, in percent: only
/// the pieces that say anything.
pub(super) fn armour_kinds(content: &Content) -> String {
    let mut out =
        String::from("| Armour | Protection | Against | Resists |\n|---|:---:|---|---|\n");
    let name = |k: DamageTypeId| content.damage_type(k).name.as_str();
    for (_, o) in order::objects(content) {
        if o.protects.is_empty() && o.resist.is_empty() {
            continue;
        }
        let against: Vec<String> = (o.protects.iter())
            .map(|&(k, more)| {
                let sign = if more < 0 { "\u{2212}" } else { "+" };
                format!("{sign}{} {}", more.abs(), name(k))
            })
            .collect();
        let resists: Vec<String> = (o.resist.iter())
            .map(|&(k, p)| format!("{} {p}%", name(k)))
            .collect();
        let protection = if o.protection > 0 {
            o.protection.to_string()
        } else {
            "—".into()
        };
        let dash = |v: Vec<String>| {
            if v.is_empty() {
                "—".into()
            } else {
                v.join(", ")
            }
        };
        let _ = writeln!(
            out,
            "| {} | {protection} | {} | {} |",
            capital(&o.name),
            dash(against),
            dash(resists)
        );
    }
    out
}

/// Damage dice with their kind, for the traps' and scrolls' tables: "3d6 fire".
pub(super) fn typed_dice(
    content: &Content,
    dice: impl std::fmt::Display,
    kind: Option<DamageTypeId>,
) -> String {
    match kind {
        Some(k) => format!("{dice} {}", content.damage_type(k).name),
        None => dice.to_string(),
    }
}
