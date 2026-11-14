//! Enchanting's recipes (docs/design/enchanting.md §7, §8; `world/enchant.rs`): each recipe of
//! `data/recipes/enchanting/` with what it works on, the crystals it takes, its time, the skill it
//! is listed from, its difficulty, and the odds of a made check at three skill values; and the
//! enchanter's reach by skill value. From the recipe files, `data/skills/`'s Enchanting, and
//! `data/enchanting.toml`. And the seals and the rare materials (docs/design/enchanting.md §9;
//! `world/seal.rs`): a seal's cost and time, and where the rare materials come from, with their
//! chances, from `data/enchanting.toml` and `data/crystals.toml`.

use super::util::capital;
use std::fmt::Write as _;
use warlock_core::combat::HIGHEST;
use warlock_core::content::{Content, EnchantKind, RecipeUse};
use warlock_core::skills::{Effect, MAX_ENCHANT_LEVEL};
use warlock_core::time::Duration;
use warlock_core::world::{EnchantBand, MOST_SEALS, enchant_band};

/// The skill values the odds are given at.
const ODDS_AT: [u16; 3] = [20, 60, 110];

/// What Enchanting gives of `effect` at trained value `value`.
fn effect_at(content: &Content, effect: Effect, value: u16) -> i32 {
    let Some(skill) = content.skill_id("enchanting") else {
        return 0;
    };
    (content.skill(skill).benefits.iter())
        .filter(|b| b.effect == effect)
        .map(|b| b.at(value))
        .sum()
}

/// The reach at trained value `value`: the item level an enchanting roll is made at, at least.
fn reach_at(content: &Content, value: u16) -> i32 {
    effect_at(content, Effect::EnchantLevel, value).clamp(1, MAX_ENCHANT_LEVEL)
}

/// The percent of 2d12 rolls that make a check of `bonus` against `against`.
fn made_percent(bonus: i32, against: i32) -> f64 {
    let sides = HIGHEST / 2;
    let mut made = 0;
    for a in 1..=sides {
        for b in 1..=sides {
            let natural = a + b;
            if enchant_band(natural, natural + bonus - against) == EnchantBand::Made {
                made += 1;
            }
        }
    }
    f64::from(made) * 100.0 / f64::from(sides * sides)
}

/// Every Enchanting recipe, by the skill it is listed from and its name, with the odds of a
/// made check for an enchanter of average Willpower (modifier 0) at [`ODDS_AT`], each rolling
/// at their reach; then the reach and the craft bonus at those values.
pub(super) fn recipes_enchanting(content: &Content) -> String {
    let step = content
        .enchanting()
        .map_or(3, |r| i32::from(r.level_step.max(1)));
    let at: Vec<String> = ODDS_AT.iter().map(u16::to_string).collect();
    let mut out = format!(
        "| Recipe | Works on | Crystals | Time | Listed from | Difficulty | Made at Enchanting {} \
         |\n|---|---|:---:|---|:---:|:---:|:---:|\n",
        at.join(" / ")
    );
    let mut rows: Vec<_> = (content.recipes().list.iter())
        .filter_map(|r| match r.uses {
            Some(RecipeUse::Enchant(kind)) => Some((r, kind)),
            _ => None,
        })
        .collect();
    rows.sort_by(|a, b| (a.0.from, &a.0.name).cmp(&(b.0.from, &b.0.name)));
    for (r, kind) in rows {
        let works = match kind {
            EnchantKind::Kindle => "a plain item: one affix",
            EnchantKind::Quicken => {
                "a magic item of one affix: the other kind; a rare, with a heart-crystal: one more"
            }
            EnchantKind::Recast => {
                "a magic item: its affixes rolled anew (one kept with a soul-fetter)"
            }
            EnchantKind::Imbue => "a plain item of capacity 3+: rare, 3 to 6 affixes",
            EnchantKind::Ennoble => "a magic item of capacity 3+: rare, one affix more",
            EnchantKind::Unbind => "a magic or rare item: every affix off",
        };
        let odds: Vec<String> = (ODDS_AT.iter())
            .map(|&v| {
                let bonus = effect_at(content, Effect::Craft, v);
                let against = r.difficulty + reach_at(content, v) / step;
                format!("{:.0}%", made_percent(bonus, against))
            })
            .collect();
        let _ = writeln!(
            out,
            "| {} | {works} | {} | {} | {} | {} | {} |",
            r.name,
            r.crystals,
            Duration::seconds(u64::from(r.seconds)).spoken(),
            r.from,
            r.difficulty,
            odds.join(" / ")
        );
    }
    out.push_str("\n| Enchanting | Reach (item level) | Craft bonus |\n|:---:|:---:|:---:|\n");
    for v in [0, 20, 40, 60, 80, 110, 200] {
        let _ = writeln!(
            out,
            "| {v} | {} | +{} |",
            reach_at(content, v),
            effect_at(content, Effect::Craft, v)
        );
    }
    let _ = writeln!(
        out,
        "\nThe check adds one to the difficulty for every {step} item levels of the roll."
    );
    out
}

/// The seals and the rare materials: what a seal costs, and the heart-crystal's and the
/// soul-fetter's work and sources with their exact chances.
pub(super) fn enchanting_seals(content: &Content) -> String {
    let (Some(crystals), Some(rules)) = (content.crystals(), content.enchanting()) else {
        return "There are no seals.\n".to_string();
    };
    let rare = &crystals.rare;
    let name = |id| content.object(id).describe(1);
    let mut out = String::new();
    let _ = writeln!(
        out,
        "- **A seal** costs one more crystal of the colour it seals and adds {} to the work. A \
         roll takes at most {MOST_SEALS} seals.",
        Duration::seconds(u64::from(rules.seal_seconds)).spoken(),
    );
    let _ = writeln!(
        out,
        "- **{}** is what Quicken takes beside its crystal to add an affix to a rare item.",
        capital(&name(rare.heart)),
    );
    let _ = writeln!(
        out,
        "- **{}** is what Recast takes to keep the affix you choose.",
        capital(&name(rare.fetter)),
    );
    let materials: Vec<String> = rare.materials.iter().map(|&m| name(m)).collect();
    let _ = writeln!(
        out,
        "- [Soulglass](magic.md#spell-soulglass) on a creature of level {} or more: a {}% chance \
         of {} (even odds). A dragon always leaves {}.",
        rare.level,
        rare.chance,
        materials.join(" or "),
        name(rare.dragon),
    );
    let _ = writeln!(
        out,
        "- [Salvaging](crafting.md#salvage) a rare item of item level {} or more: a {}% chance of \
         {} (even odds).",
        rare.salvage_level,
        rare.salvage_chance,
        materials.join(" or "),
    );
    out
}
