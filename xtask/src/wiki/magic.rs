//! Magic items (spec §8 "Magic items"): rarity, base items, prefixes, suffixes, and rare names.

use super::combat::percent;
use super::order;
use super::text::capital;
use super::vision::glow_word;
use std::fmt::Write as _;
use warlock_core::content::Effect as ItemEffect;
use warlock_core::content::{
    AffixDef, AffixKind, Content, ItemKind, MAX_ITEM_LEVEL, MAX_TIERS, Rarity,
};
use warlock_core::magic::describe;
use warlock_core::magic::loot::TIER_WINDOW;
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
    let [prefix, suffix, both] = affixes.magic_affixes;
    let shapes = (prefix + suffix + both).max(1);
    let _ = writeln!(
        out,
        "\n| A magic item has | How often |\n|---|---:|\n\
         | a prefix | {} |\n| a suffix | {} |\n| a prefix and a suffix | {} |",
        percent(prefix, shapes),
        percent(suffix, shapes),
        percent(both, shapes)
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
    let window = TIER_WINDOW - 1;
    let _ = writeln!(
        out,
        "\nAn affix has at most {MAX_TIERS} tiers. Its tier is drawn evenly among the tiers whose \
         item level is at most the item's and at least {window} below it (an item of level 12 \
         rolls the tiers of levels 9 to 12); an item whose level is past the affix's last tier, \
         or with no tier that close, gets the highest tier it may have. What a creature drops \
         has the creature's level as its item level, or the floor's where that is higher (up to \
         {MAX_ITEM_LEVEL}); what lies on the floor has the floor's."
    );
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
    for &kind in ItemKind::ALL {
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
        "| {what} | Effect | Category | Rolls on | Tiers (item level: value) | Weight |\n\
         |---|---|---|---|---|:---:|\n"
    );
    for a in order::affixes(content, kind) {
        let bases: Vec<&str> = a.bases.iter().map(|k| k.plural()).collect();
        let tiers: Vec<String> = (a.tiers.iter())
            .map(|t| format!("{}: {}", t.level, tier_value(a.effect, t, content)))
            .collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} |",
            a.name,
            effect_text(content, a),
            a.category.replace('_', " "),
            bases.join(", "),
            tiers.join(" · "),
            a.weight
        );
    }
    out
}

/// What affix `a` does, in words, without its value: its effect's words, the skill it adds to,
/// or the resource it changes.
fn effect_text(content: &Content, a: &AffixDef) -> String {
    if let Some(id) = a.resource {
        let pool = content.resource(id).name.to_lowercase();
        return match a.effect {
            ItemEffect::ResourceMax => format!("maximum {pool}"),
            ItemEffect::ResourceRate => format!("{pool} comes back faster"),
            ItemEffect::ActiveStamina => format!("weapon feats cost less {pool}"),
            ItemEffect::ResourceOnKill => format!("{pool} on a kill within 3 cells"),
            other => effect_words(other),
        };
    }
    match (a.skill, a.damage_type) {
        (Some(id), _) => format!("{} (a skill)", content.skill(id).name),
        (_, Some(kind)) if a.effect == ItemEffect::Elemental => {
            format!("adds a die of {} damage", content.damage_type(kind).name)
        }
        (_, Some(kind)) => format!("{} resistance", content.damage_type(kind).name),
        (None, None) => effect_words(a.effect),
    }
}

/// What an effect does, in words, without its value.
fn effect_words(effect: ItemEffect) -> String {
    match effect {
        ItemEffect::ToHit => "to hit".into(),
        ItemEffect::Damage => "damage, added to every roll".into(),
        ItemEffect::ExtraDamage => "adds a die of damage".into(),
        ItemEffect::Elemental => "adds a die of damage of a kind (fire, cold, ...)".into(),
        ItemEffect::Threat => "criticals threaten on more rolls".into(),
        ItemEffect::Crit => "critical damage multiplier".into(),
        ItemEffect::AttackSpeed => "attacks faster".into(),
        ItemEffect::Range => "range, in cells".into(),
        ItemEffect::Draw => "less Strength needed to draw".into(),
        ItemEffect::Durability => "breaks less often".into(),
        ItemEffect::Parry => "Parry".into(),
        ItemEffect::Armour => "Armour".into(),
        ItemEffect::Protection => "Protection".into(),
        ItemEffect::Shield => "Shield".into(),
        ItemEffect::MaxDex => "Dexterity may add more to Defence".into(),
        ItemEffect::Weight => "lighter".into(),
        ItemEffect::Dodge => "Dodge".into(),
        ItemEffect::Deflection => "Deflection".into(),
        ItemEffect::Natural => "natural armour".into(),
        ItemEffect::MaxHp => "maximum hit points".into(),
        ItemEffect::Regeneration => "hit points come back faster".into(),
        ItemEffect::MaxMana => "maximum mana".into(),
        ItemEffect::ManaRegeneration => "mana comes back faster".into(),
        ItemEffect::CasterLevel => "caster level".into(),
        ItemEffect::ManaCost => "every mana cost is lower".into(),
        ItemEffect::SpellEase => "this armour's spell cost surcharge is lower, in points".into(),
        ItemEffect::SpellToHit => "to hit with your own spells".into(),
        ItemEffect::SpellThreat => "your spells threaten a critical on more rolls".into(),
        ItemEffect::SpellLift => "your spells' damage lift, in points of percent".into(),
        ItemEffect::SpellReach => "your spells' reach, in cells".into(),
        ItemEffect::SpellDuration => "your buffs and hexes last more degrees".into(),
        ItemEffect::ManaOnKill => "mana on a kill within 3 cells".into(),
        ItemEffect::WandLevel => "the caster level wands are zapped at".into(),
        ItemEffect::ResourceMax => "a resource's maximum".into(),
        ItemEffect::ResourceRate => "a resource comes back faster".into(),
        ItemEffect::ActiveStamina => "weapon feats cost less of a resource".into(),
        ItemEffect::ResourceOnKill => "a resource on a kill within 3 cells".into(),
        ItemEffect::Resistance => "resistance to a kind of damage, in percent".into(),
        ItemEffect::Stat(stat) => stat.name().into(),
        ItemEffect::MoveSpeed => "you move faster".into(),
        ItemEffect::Carry => "carrying capacity".into(),
        ItemEffect::Stealth => "quieter footsteps".into(),
        ItemEffect::DoorKick => "Strength for kicking doors and chests in".into(),
        ItemEffect::DigPercent => "digs faster".into(),
        ItemEffect::Hunger => "hunger comes slower".into(),
        ItemEffect::TrapDamage => "traps hurt less".into(),
        ItemEffect::DiseaseCheck => "disease checks".into(),
        ItemEffect::PotionKept => "a potion drunk is not used up".into(),
        ItemEffect::Light => "the wearer glows".into(),
        ItemEffect::Skill => "a skill".into(),
    }
}

/// What a tier rolls, exactly: `+1–2`, `1d3–1d4`, `20–35%`, `5–8% less time` (speed takes its
/// percent off the cost, `magic::mods::faster`).
fn tier_value(effect: ItemEffect, tier: &warlock_core::content::Tier, content: &Content) -> String {
    let range = |prefix: &str, suffix: &str| {
        if tier.min == tier.max {
            format!("{prefix}{}{suffix}", tier.min)
        } else {
            format!("{prefix}{}–{}{suffix}", tier.min, tier.max)
        }
    };
    match effect {
        ItemEffect::ExtraDamage | ItemEffect::Elemental if tier.min == tier.max => {
            format!("1d{}", tier.min)
        }
        ItemEffect::ExtraDamage | ItemEffect::Elemental => {
            format!("1d{}–1d{}", tier.min, tier.max)
        }
        ItemEffect::Resistance => range("+", "%"),
        ItemEffect::Draw => range("", " less"),
        ItemEffect::AttackSpeed | ItemEffect::MoveSpeed | ItemEffect::DigPercent => {
            range("", "% less time")
        }
        ItemEffect::ManaCost | ItemEffect::TrapDamage => range("", "% less"),
        ItemEffect::SpellEase => range("", " points less"),
        ItemEffect::ActiveStamina => range("", " less"),
        ItemEffect::Hunger => range("", "% slower"),
        ItemEffect::PotionKept => range("", "% kept"),
        ItemEffect::Durability
        | ItemEffect::Weight
        | ItemEffect::Stealth
        | ItemEffect::Regeneration
        | ItemEffect::ManaRegeneration
        | ItemEffect::ResourceRate => range("", "%"),
        ItemEffect::SpellLift => range("+", "%"),
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
