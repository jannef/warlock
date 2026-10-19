//! Magic items (spec §8 "Magic items"): rarity, base items, prefixes, suffixes, and rare names.

use super::combat::percent;
use super::order;
use super::text::capital;
use super::vision::glow_word;
use std::fmt::Write as _;
use warlock_core::content::Effect as ItemEffect;
use warlock_core::content::{AffixDef, AffixKind, Content, ItemKind, Rarity};
use warlock_core::magic::describe;
use warlock_core::object::format_weight;

/// The prefixes table.
pub(super) fn prefixes(content: &Content) -> String {
    affix_table(content, AffixKind::Prefix)
}

/// The suffixes table.
pub(super) fn suffixes(content: &Content) -> String {
    affix_table(content, AffixKind::Suffix)
}

/// How often loot is normal, magic, or rare, and how many affixes magic and rare items roll
/// (spec §8 "Magic items").
pub(super) fn magic_rarity(content: &Content) -> String {
    let affixes = content.affixes();
    let total: u32 = Rarity::ALL.iter().map(|&r| affixes.rarity_weight(r)).sum();
    let mut out = String::from("| Loot is | How often |\n|---|---:|\n");
    for rarity in Rarity::ALL {
        let _ = writeln!(
            out,
            "| {} | {} |",
            capital(rarity.word()),
            percent(affixes.rarity_weight(rarity), total.max(1))
        );
    }
    let both = affixes.magic_both;
    let _ = writeln!(
        out,
        "\n| A magic item has | How often |\n|---|---:|\n\
         | a prefix | {} |\n| a suffix | {} |\n| a prefix and a suffix | {} |",
        percent(100 - both, 200),
        percent(100 - both, 200),
        percent(both, 100)
    );
    let counts: u32 = affixes.rare_counts.iter().map(|(_, w)| *w).sum();
    out.push_str("\n| A rare item has | How often |\n|---|---:|\n");
    for (count, weight) in &affixes.rare_counts {
        let _ = writeln!(
            out,
            "| {count} affixes | {} |",
            percent(*weight, counts.max(1))
        );
    }
    out
}

/// The base items magic items are made of, by kind: what they do, from which item level loot
/// holds them, and how often.
pub(super) fn magic_bases(content: &Content) -> String {
    let mut out = String::from(
        "| Base item | Kind | Stats | Weight | From item level | How often |\n\
         |---|---|---|---:|:---:|:---:|\n",
    );
    let all = order::objects(content);
    for kind in ItemKind::ALL {
        let bases: Vec<_> = (all.iter())
            .map(|(_, o)| *o)
            .filter(|o| o.kind == Some(kind))
            .collect();
        let total: u32 = bases.iter().map(|o| o.frequency).sum();
        for o in bases {
            let stats = describe::base_stats(o, content)
                .into_iter()
                .filter(|s| !s.starts_with("Worn") && !s.starts_with("Held"))
                .collect::<Vec<_>>();
            let stats = if stats.is_empty() {
                "-".to_string()
            } else {
                stats.join("; ")
            };
            let often = if o.frequency == 0 {
                "never".to_string()
            } else {
                percent(o.frequency, total.max(1))
            };
            let _ = writeln!(
                out,
                "| {} | {} | {stats} | {} | {} | {often} |",
                capital(&o.name),
                kind.plural(),
                format_weight(u64::from(o.weight)),
                o.level,
            );
        }
    }
    out.push_str(
        "\n\"How often\" is the base's share among the bases of its kind; every kind weighs the \
         sum of its bases' frequencies when loot picks a base.\n",
    );
    out
}

/// Every affix of `kind`: what it does, what it rolls on, and its tiers (from which item level,
/// and the values they roll).
fn affix_table(content: &Content, kind: AffixKind) -> String {
    let what = kind.name();
    let mut out = format!(
        "| {what} | Effect | Rolls on | Tiers (item level: value) | Weight |\n\
         |---|---|---|---|:---:|\n"
    );
    for a in order::affixes(content, kind) {
        let bases: Vec<&str> = a.bases.iter().map(|k| k.plural()).collect();
        let tiers: Vec<String> = (a.tiers.iter())
            .map(|t| format!("{}: {}", t.level, tier_value(a.effect, t, content)))
            .collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            a.name,
            effect_text(content, a),
            bases.join(", "),
            tiers.join(" · "),
            a.weight
        );
    }
    out
}

/// What affix `a` does, in words, without its value: its effect's words, or the skill it adds to.
fn effect_text(content: &Content, a: &AffixDef) -> String {
    match a.skill {
        Some(id) => format!("{} (a skill)", content.skill(id).name),
        None => effect_words(a.effect),
    }
}

/// What an effect does, in words, without its value.
fn effect_words(effect: ItemEffect) -> String {
    match effect {
        ItemEffect::ToHit => "to hit".into(),
        ItemEffect::Damage => "damage, added to every roll".into(),
        ItemEffect::ExtraDamage => "adds a die of damage".into(),
        ItemEffect::Threat => "criticals threaten on more rolls".into(),
        ItemEffect::Crit => "critical damage multiplier".into(),
        ItemEffect::AttackSpeed => "attacks faster".into(),
        ItemEffect::Range => "range, in cells".into(),
        ItemEffect::Draw => "less Strength needed to draw".into(),
        ItemEffect::Durability => "breaks less often".into(),
        ItemEffect::Parry => "Parry".into(),
        ItemEffect::Armour => "Armour".into(),
        ItemEffect::Shield => "Shield".into(),
        ItemEffect::MaxDex => "Dexterity may add more to Defence".into(),
        ItemEffect::Weight => "lighter".into(),
        ItemEffect::Dodge => "Dodge".into(),
        ItemEffect::Deflection => "Deflection".into(),
        ItemEffect::Natural => "natural armour".into(),
        ItemEffect::MaxHp => "maximum hit points".into(),
        ItemEffect::Regeneration => "hit points come back faster".into(),
        ItemEffect::Stat(stat) => stat.name().into(),
        ItemEffect::MoveSpeed => "you move faster".into(),
        ItemEffect::Carry => "carrying capacity".into(),
        ItemEffect::Stealth => "quieter footsteps".into(),
        ItemEffect::Light => "the wearer glows".into(),
        ItemEffect::Skill => "a skill".into(),
    }
}

/// What a tier rolls, in words: `+1–2`, `1d3–1d4`, `20–35%`; speed in words only.
fn tier_value(effect: ItemEffect, tier: &warlock_core::content::Tier, content: &Content) -> String {
    let range = |prefix: &str, suffix: &str| {
        if tier.min == tier.max {
            format!("{prefix}{}{suffix}", tier.min)
        } else {
            format!("{prefix}{}–{}{suffix}", tier.min, tier.max)
        }
    };
    let faster = |p: i32| match p {
        ..=7 => "a little faster",
        8..=12 => "faster",
        _ => "much faster",
    };
    match effect {
        ItemEffect::ExtraDamage if tier.min == tier.max => format!("1d{}", tier.min),
        ItemEffect::ExtraDamage => format!("1d{}–1d{}", tier.min, tier.max),
        ItemEffect::Draw => range("", " less"),
        ItemEffect::AttackSpeed | ItemEffect::MoveSpeed => {
            let (lo, hi) = (faster(tier.min), faster(tier.max));
            if lo == hi {
                lo.to_string()
            } else {
                format!("{lo} to {hi}")
            }
        }
        ItemEffect::Durability
        | ItemEffect::Weight
        | ItemEffect::Stealth
        | ItemEffect::Regeneration => range("", "%"),
        ItemEffect::Carry => range("+", " kg"),
        ItemEffect::Light => {
            let radius = tier.light.map_or(0, |id| content.light(id).radius);
            format!("{} ({radius} cells)", glow_word(radius))
        }
        _ => range("+", ""),
    }
}

/// The words rare items are named with: the first words, and the second words by kind.
pub(super) fn rare_names(content: &Content) -> String {
    let affixes = content.affixes();
    let mut out = format!(
        "First words: {}.\n\n| Kind | Second words |\n|---|---|\n",
        affixes.first_names.join(", ")
    );
    for (kind, words) in &affixes.second_names {
        let _ = writeln!(out, "| {} | {} |", capital(kind.plural()), words.join(", "));
    }
    out
}
