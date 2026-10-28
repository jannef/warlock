//! Chests (spec §8 "Chests"; `data/chests.toml`): the bands, how hard each is to kick open,
//! which blasts blow it apart, how often a generated one is locked and trapped, what it holds,
//! and how often each dungeon's floors have one.

use std::fmt::Write as _;
use warlock_core::content::Content;
use warlock_core::dice::Dice;
use warlock_core::door;

/// The Strength scores the kick columns show (what the kicker's athletics and talents add counts
/// as Strength).
const KICK_STRENGTHS: [u8; 6] = [6, 10, 14, 18, 24, 30];

/// Every chest band, least first (the file's order: rising levels): its levels, name, how
/// sturdy it is with the chance a kick breaks it by Strength, and the blast it takes to blow
/// it apart.
pub(super) fn chest_bands(content: &Content) -> String {
    let rules = content.chests();
    let mut out = String::from("| Levels | Chest | Sturdiness |");
    let mut rule = String::from("|:---:|---|:---:|");
    for st in KICK_STRENGTHS {
        let _ = write!(out, " Strength {st} |");
        rule.push_str(":---:|");
    }
    let _ = writeln!(out, " Hardness |\n{rule}:---:|");
    for (i, band) in rules.bands.iter().enumerate() {
        let levels = match rules.bands.get(i + 1) {
            Some(next) if next.from - 1 == band.from => band.from.to_string(),
            Some(next) => format!("{}–{}", band.from, next.from - 1),
            None => format!("{} and up", band.from),
        };
        let _ = write!(out, "| {levels} | {} | {} |", band.name, band.sturdiness);
        for st in KICK_STRENGTHS {
            let _ = write!(out, " {}% |", door::break_chance(st, band.sturdiness));
        }
        let _ = writeln!(out, " {} |", band.hardness);
    }
    out
}

/// What each band's description says, as look mode adds it.
pub(super) fn chest_descriptions(content: &Content) -> String {
    let mut out = String::from("| Chest | What you see |\n|---|---|\n");
    for band in &content.chests().bands {
        let _ = writeln!(out, "| {} | {} |", band.name, band.description);
    }
    out
}

/// The chance each stack breaks when a chest is kicked open or blown apart, and what is spared.
pub(super) fn chest_breaking(content: &Content) -> String {
    let destroyed = content.chests().destroyed;
    let deft = (content.talents().id("deft_breaker"))
        .map(|id| content.talents().get(id))
        .and_then(|t| {
            t.effects.iter().find_map(|e| match e {
                warlock_core::content::TalentEffect::SmashDestroyPercent { value } => Some(*value),
                _ => None,
            })
        })
        .unwrap_or(0);
    let with_deft = i32::from(destroyed) * (100 + deft).max(0) / 100;
    format!(
        "| How it opens | Each stack inside breaks | Spared |\n|---|:---:|---|\n\
         | Opened (a key, Pick lock, Knock then open, or unlocked) | never | everything |\n\
         | Kicked open | {destroyed}% | coins and loose gems |\n\
         | Kicked open, with Deft breaker | {with_deft}% | coins and loose gems |\n\
         | Blown apart by a blast | {destroyed}% | nothing |\n"
    )
}

/// How often each band's generated chests come locked and trapped, and are a mimic instead, by
/// levels.
pub(super) fn chest_odds(content: &Content) -> String {
    let rules = content.chests();
    let mut out = String::from(
        "| Levels | Chest | Locked | Trapped | A mimic |\n|:---:|---|:---:|:---:|:---:|\n",
    );
    for (i, band) in rules.bands.iter().enumerate() {
        let levels = levels(band.from, rules.bands.get(i + 1).map(|b| b.from));
        let _ = writeln!(
            out,
            "| {levels} | {} | {}% | {}% | {}% |",
            band.name, band.locked, band.trapped, band.mimic
        );
    }
    out
}

/// What a generated chest of each band holds (spec §8 "Chest loot"): the dice, the levels above
/// the floor's its items and gems are rolled at, how many are magic and rare, the purse, the
/// gems, and the mean of each.
pub(super) fn chest_loot(content: &Content) -> String {
    let rules = content.chests();
    let mut out = String::from(
        "| Chest | Items | Item level | Magic / rare | Gold | Gems | On average |\n\
         |---|:---:|:---:|:---:|:---:|:---:|---|\n",
    );
    for band in &rules.bands {
        let loot = &band.loot;
        let level = match loot.loot_level {
            0 => "the floor's".to_string(),
            n => format!("the floor's + {n}"),
        };
        let gold = match loot.gold.each {
            1 => loot.gold.dice.to_string(),
            each => format!("{} × {each}", loot.gold.dice),
        };
        let items = mean(loot.items);
        let coins = mean(loot.gold.dice) * f64::from(loot.gold.each);
        let gems = mean(loot.gems);
        let _ = writeln!(
            out,
            "| {} | {} | {level} | {}% / {}% | {gold} | {} | {items:.1} items, {coins:.1} gold, \
             {gems:.2} gems |",
            band.name, loot.items, loot.magic, loot.rare, loot.gems
        );
    }
    out
}

/// How often the floors of each dungeon have chests, and how many.
pub(super) fn chest_places(content: &Content) -> String {
    let mut out = String::from(
        "| Place | Floors with chests | Chests on such a floor |\n|---|:---:|:---:|\n",
    );
    let mut branches: Vec<_> = content.dungeons().collect();
    branches.sort_by(|a, b| a.id.cmp(&b.id));
    for d in branches {
        let [least, most] = d.chests.count;
        let count = match least == most {
            true => least.to_string(),
            false => format!("{least} to {most}"),
        };
        let _ = writeln!(out, "| {} | {}% | {count} |", d.name, d.chests.chance);
    }
    out
}

/// A band's levels: `from` alone, `from`–(the next band's less one), or `from` and up.
fn levels(from: u8, next: Option<u8>) -> String {
    match next {
        Some(next) if next - 1 == from => from.to_string(),
        Some(next) => format!("{from}–{}", next - 1),
        None => format!("{from} and up"),
    }
}

/// The mean of `dice`, a result below 0 counting as 0, exactly (over every outcome).
fn mean(dice: Dice) -> f64 {
    let sides = dice.sides().max(1) as usize;
    // The ways each sum of the dice comes up, from 0 up.
    let mut ways = vec![1u64];
    for _ in 0..dice.count() {
        let mut next = vec![0u64; ways.len() + sides];
        for (sum, &w) in ways.iter().enumerate() {
            for face in 1..=sides {
                next[sum + face] += w;
            }
        }
        ways = next;
    }
    let sign = if dice.is_negative() { -1 } else { 1 };
    let total: u64 = ways.iter().sum();
    let weighted: f64 = (ways.iter().enumerate())
        .map(|(sum, &w)| (sign * sum as i64 + i64::from(dice.bonus())).max(0) as f64 * w as f64)
        .sum();
    weighted / total as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn means_count_rolls_below_nothing_as_nothing() {
        let d = |s: &str| s.parse::<Dice>().expect(s);
        assert!((mean(d("2d6")) - 7.0).abs() < 1e-9);
        assert!((mean(d("1d4-3")) - 0.25).abs() < 1e-9);
        assert!((mean(d("1d3-1")) - 1.0).abs() < 1e-9);
        assert!((mean(d("2d3+1")) - 5.0).abs() < 1e-9);
    }
}
