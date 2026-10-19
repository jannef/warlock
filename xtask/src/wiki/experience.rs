//! Experience (spec §8 "Experience"): the level curve, the zero difference, kill XP scaling, an
//! example, and the creatures' levels.

use super::text::capital;
use std::fmt::Write as _;
use warlock_core::content::Content;

/// The XP needed for each level (spec §8 "Experience"), every level to 20 and then every fifth.
pub(super) fn experience_levels(content: &Content) -> String {
    let rules = content.experience();
    let mut out = String::from("| Level | XP needed | Hit points gained |\n|:---:|---:|:---:|\n");
    for level in (2..=rules.max_level()).filter(|l| *l <= 20 || l % 5 == 0) {
        let hp = if level <= rules.hp_level() {
            "yes"
        } else {
            "no"
        };
        let _ = writeln!(out, "| {level} | {} | {hp} |", rules.threshold(level));
    }
    out
}

/// The "zero difference" by the player's level.
pub(super) fn experience_zero_difference(content: &Content) -> String {
    let rules = content.experience();
    let mut out =
        String::from("| Your level | Grey at this many levels below you |\n|---|:---:|\n");
    let bands = rules.bands();
    for (i, &(from, zd)) in bands.iter().enumerate() {
        let to = bands
            .get(i + 1)
            .map_or(rules.max_level(), |next| next.0 - 1);
        let levels = if from >= to {
            from.to_string()
        } else {
            format!("{from} to {to}")
        };
        let _ = writeln!(out, "| {levels} | {zd} |");
    }
    out
}

/// What a kill is worth by the monster's level against yours, as a share of its XP, for a player
/// of level 7 (a zero difference of 5).
pub(super) fn experience_scaling(content: &Content) -> String {
    let rules = content.experience();
    let player = 7u8;
    let mut out = String::from(
        "| Monster's level | Compared with you | Share of its XP |\n|:---:|---|:---:|\n",
    );
    let (bonus_levels, _) = rules.bonus();
    for diff in (-6i32..=i32::from(bonus_levels) + 1).rev() {
        let monster = (i32::from(player) + diff) as u8;
        let share = rules.kill_xp(100, monster, player);
        let against = match diff {
            0 => "the same level".to_string(),
            d if d > 0 => format!("{d} above"),
            d => format!("{} below", -d),
        };
        let note = if share == 0 {
            "nothing (grey)".to_string()
        } else {
            format!("{share}%")
        };
        let _ = writeln!(out, "| {monster} | {against} | {note} |");
    }
    out
}

/// One creature's XP at several player levels: the kobold shaman, if there is one.
pub(super) fn experience_example(content: &Content) -> String {
    let rules = content.experience();
    let Some(id) = content.creature_id("kobold_shaman") else {
        return String::new();
    };
    let shaman = content.creature(id);
    let mut out = format!(
        "| Your level | XP for a {} (level {}, worth {}) |\n|:---:|---:|\n",
        shaman.name, shaman.level, shaman.xp
    );
    for level in 1..=10 {
        let xp = rules.kill_xp(shaman.xp, shaman.level, level);
        let _ = writeln!(out, "| {level} | {xp} |");
    }
    out
}

/// The creatures' levels and XP.
pub(super) fn experience_creatures(content: &Content) -> String {
    let mut out = String::from("| Creature | Level | XP |\n|---|:---:|---:|\n");
    let mut list: Vec<_> = (content.creature_defs().iter())
        .filter(|c| c.id != "player")
        .collect();
    list.sort_by_key(|c| (c.level, c.xp));
    for c in list {
        let _ = writeln!(out, "| {} | {} | {} |", capital(&c.name), c.level, c.xp);
    }
    out
}
