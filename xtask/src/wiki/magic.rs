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
use warlock_core::world::SIPHON_RANGE;

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
            ItemEffect::ResourceOnKill => format!("{pool} on a kill within {SIPHON_RANGE} cells"),
            other => effect_words(other),
        };
    }
    match (a.skill, a.damage_type) {
        (Some(id), _) if super::skills::gated(content.skill(id)) => format!(
            "{} (a skill), for those trained in it",
            content.skill(id).name
        ),
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
        ItemEffect::ManaOnKill => format!("mana on a kill within {SIPHON_RANGE} cells"),
        ItemEffect::WandLevel => "the caster level wands are zapped at".into(),
        ItemEffect::ResourceMax => "a resource's maximum".into(),
        ItemEffect::ResourceRate => "a resource comes back faster".into(),
        ItemEffect::ActiveStamina => "weapon feats cost less of a resource".into(),
        ItemEffect::ResourceOnKill => format!("a resource on a kill within {SIPHON_RANGE} cells"),
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

/// What systems have affixes (docs/design/affixes-systems.md §11): every affix with the system
/// it works in, what it changes there, and the talents read at the same place, whose terms its
/// value joins. By system, then by name.
pub(super) fn systems(content: &Content) -> String {
    let mut rows: Vec<(&str, &AffixDef)> = (content.affixes().all().iter())
        .map(|a| (system_of(content, a), a))
        .collect();
    rows.sort_by_cached_key(|(system, a)| (*system, a.name.to_lowercase()));
    let mut out = String::from(
        "| Affix | Kind | System | What it changes | Talents beside it |\n|---|---|---|---|---|\n",
    );
    let book = content.talents();
    for (system, a) in rows {
        let talents: Vec<&str> = (talents_beside(content, a).iter())
            .map(|id| {
                let id = (book.id(id)).unwrap_or_else(|| panic!("no talent `{id}`"));
                book.get(id).name.as_str()
            })
            .collect();
        let talents = if talents.is_empty() {
            "-".to_string()
        } else {
            talents.join(", ")
        };
        let _ = writeln!(
            out,
            "| {} | {} | {system} | {} | {talents} |",
            a.name,
            a.kind.name().to_lowercase(),
            effect_text(content, a),
        );
    }
    out
}

/// The system an affix works in, as the wiki's pages name it.
fn system_of(content: &Content, a: &AffixDef) -> &'static str {
    use ItemEffect as E;
    if a.resource
        .is_some_and(|r| content.resource(r).id != "stamina")
    {
        return "Resources";
    }
    match a.effect {
        E::ToHit | E::Damage | E::ExtraDamage | E::Threat | E::Crit | E::AttackSpeed => "Combat",
        E::Range | E::Draw => "Ranged combat",
        E::Elemental | E::Resistance => "Damage types",
        E::Parry | E::Armour | E::Protection | E::Shield | E::MaxDex => "Armour and Defence",
        E::Dodge | E::Deflection | E::Natural => "Armour and Defence",
        E::MaxHp | E::Regeneration => "Hit points",
        E::MaxMana | E::ManaRegeneration | E::ManaCost | E::ManaOnKill => "Mana",
        E::CasterLevel | E::SpellEase | E::SpellToHit | E::SpellThreat | E::SpellLift => "Spells",
        E::SpellReach | E::SpellDuration | E::WandLevel => "Spells",
        E::ResourceMax | E::ResourceRate | E::ActiveStamina | E::ResourceOnKill => "Stamina",
        E::Stat(_) => "Stats",
        E::MoveSpeed => "Speed",
        E::Weight | E::Carry => "Burden",
        E::Durability => "Objects",
        E::Stealth => "Stealth",
        E::Light => "Light",
        E::Skill => "Skills",
        E::DoorKick => "Doors",
        E::DigPercent => "Digging",
        E::Hunger => "Hunger",
        E::TrapDamage => "Traps",
        E::DiseaseCheck => "Diseases",
        E::PotionKept => "Potions",
    }
}

/// The talents (ids) whose terms an affix's value joins, read at the same place.
fn talents_beside(content: &Content, a: &AffixDef) -> &'static [&'static str] {
    use ItemEffect as E;
    let pool = a.resource.map(|r| content.resource(r).id.as_str());
    match (a.effect, pool) {
        (E::ToHit, _) => &["weapon_focus"],
        (E::Damage, _) => &["whetstone"],
        (E::Threat, _) => &["improved_critical", "masterwork"],
        (E::Crit, _) => &["brutal_critical"],
        (E::Range, _) => &["far_shot"],
        (E::Resistance, _) => &["elemental_ward"],
        (E::Protection, _) => &["armour_drill", "heavy_armour_mastery"],
        (E::Shield, _) => &["shield_expert"],
        (E::MaxDex, _) => &["armour_training"],
        (E::Dodge, _) => &["dodge"],
        (E::Deflection, _) => &["deflection"],
        (E::Natural, _) => &["thick_skin"],
        (E::MaxHp, _) => &["hardy"],
        (E::Regeneration, _) => &["healthy"],
        (E::MaxMana, _) => &["deep_well"],
        (E::ManaRegeneration, _) => &["clear_mind"],
        (E::ManaCost, _) => &["frugal_caster"],
        (E::CasterLevel, _) => &["arcane_training", "attuned"],
        (E::SpellLift, _) => &["keen_mind"],
        (E::SpellReach, _) => &["school_focus"],
        (E::WandLevel, _) => &["wand_mastery"],
        (E::ResourceMax, Some("stamina")) => &["deep_lungs", "bellows"],
        (E::ResourceMax, Some("ki")) => &["inner_calm"],
        (E::ResourceRate, Some("stamina")) => &["steady_breathing"],
        (E::ActiveStamina, _) => &["drillmaster"],
        (E::ResourceOnKill, Some("stamina")) => &["second_breath"],
        (E::MoveSpeed, _) => &["fleet", "fast_movement"],
        (E::Carry, _) => &["pack_mule"],
        (E::Stealth, _) => &["silent_feet"],
        (E::DoorKick, _) => &["lockbreaker"],
        (E::DigPercent, _) => &["tunneler"],
        (E::Hunger, _) => &["lean"],
        (E::TrapDamage, _) => &["trap_sense"],
        (E::DiseaseCheck, _) => &["strong_constitution"],
        (E::PotionKept, _) => &["thrifty_drinker"],
        _ => &[],
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
