//! Mimics (spec §8 "Mimics"): the family's bands and their numbers, stated exactly from
//! `data/mimics.toml` and the chest bands, read through the generated creature types themselves.

use std::fmt::Write as _;
use warlock_core::content::Content;
use warlock_core::content::mimic::Mimics;
use warlock_core::content::starter;
use warlock_core::stats::Stat;
use warlock_core::text::and_list;

/// The bands: each band's levels, the chest it passes for, and its first member's numbers (every
/// level between has its own type on the same formulas); then what every mimic is.
pub(super) fn mimic_bands(content: &Content) -> String {
    let mimics = content.mimics();
    if mimics.is_empty() {
        return "No mimics.\n".into();
    }
    // Three tables of a few columns each (#1586): what a band is, its scores, and its fighting.
    let mut out = String::from(
        "| Name | Levels | Passes for | Hit points | Weight |\n|---|---|---|---:|---:|\n",
    );
    let mut scores = String::from(
        "| Name | Strength | Toughness | Dexterity | Willpower |\n|---|---:|---:|---:|---:|\n",
    );
    let mut fights =
        String::from("| Name | Slam | Holds | Grip | Protection |\n|---|---|---:|---:|---:|\n");
    let chests = &content.chests().bands;
    for (i, band) in mimics.bands.iter().enumerate() {
        let from = band.from.max(mimics.levels[0]);
        if from > mimics.levels[1] {
            continue;
        }
        let to = (mimics.bands.get(i + 1))
            .map_or(mimics.levels[1], |b| b.from.saturating_sub(1))
            .min(mimics.levels[1]);
        let Some(def) = content.mimic_at(from).map(|c| content.creature(c)) else {
            continue;
        };
        let score = |s: Stat| def.stats.score(s);
        let hp = i32::from(score(Stat::Toughness)) + def.base_hp;
        let slam = def
            .attack
            .as_ref()
            .map_or("-".into(), |a| a.attack.damage.to_string());
        let chest = chests.get(i);
        let _ = writeln!(
            out,
            "| {} | {from} to {to} | {} | {hp} | {} kg |",
            band.name,
            chest.map_or("-", |c| c.name.as_str()),
            band.weight / 1000,
        );
        let _ = writeln!(
            scores,
            "| {} | {} | {} | {} | {} |",
            band.name,
            score(Stat::Strength),
            score(Stat::Toughness),
            score(Stat::Dexterity),
            score(Stat::Willpower),
        );
        let _ = writeln!(
            fights,
            "| {} | {slam} | {} % | {} | {} |",
            band.name,
            chest.map_or(0, |c| c.hold),
            Mimics::grip(from),
            Mimics::protection(from),
        );
    }
    let _ = write!(out, "\n{scores}\n{fights}");
    let first = content
        .mimic_at(mimics.levels[0])
        .map(|c| content.creature(c));
    if let Some(def) = first {
        let faction = (def.faction.map(|f| content.factions().def(f).name.clone()))
            .unwrap_or_else(|| "none".into());
        // The base of each score is `animated::Numbers::of`'s; the family adds its `stats`.
        let base = |stat: Stat, of: i32| of + mimics.stats.get(&stat).copied().unwrap_or(0);
        let cuts: Vec<String> = (1..=5)
            .map(|l| match l {
                1 => format!("{} % of the formulas' at level 1", starter::hp_percent(l)),
                l => format!("{} % at level {l}", starter::hp_percent(l)),
            })
            .collect();
        let _ = write!(
            out,
            "\nEach row is the band's first level; every level from {} to {} has its own type on \
             the same formulas (Strength {} + 0.4 × level, Toughness {} + 0.4 × level, \
             Dexterity {} + level ÷ 4, Willpower {} + 0.35 × level, rounded down; hit points {} × \
             level beyond Toughness; the slam one step up the dice ladder from the level's). The \
             first five levels are softer: Toughness and the hit points beyond it are {}, \
             rounded. Every mimic: faction **{faction}**, speed {} (slow), sight {} cells in the dark, hearing \
             {}, intelligence and presence 2, never sleeps or is poisoned; its slam holds you by \
             the chance shown (Strength against the grip to break free), and its woody hide's \
             Protection is (level − 10) ÷ 8, rounded down (none below level 18). Experience by \
             the guideline.\n",
            mimics.levels[0],
            mimics.levels[1],
            base(Stat::Strength, 11),
            base(Stat::Toughness, 12),
            base(Stat::Dexterity, 10),
            base(Stat::Willpower, 16),
            2.5 * mimics.hp_factor,
            and_list(&cuts),
            def.speed,
            def.sight,
            def.hearing.unwrap_or(0),
        );
    }
    out
}
