//! Experience (spec §8 "Experience"): the level curve in kills and XP with the race and class
//! factors, the zero difference, kill XP scaling, an example, discovery, and the creatures'
//! levels.

use super::text::{capital, thousands};
use std::fmt::Write as _;
use warlock_core::content::{Content, CreatureId, Factor, pace_words};

/// The slowest and the quickest race and class: the characters whose thresholds are the
/// highest and the lowest, by name, with their factors.
fn extremes(content: &Content) -> Option<[(String, Factor); 2]> {
    let races = &content.races().list;
    let classes = content.classes();
    let slow_race = races.iter().max_by_key(|r| r.experience)?;
    let quick_race = races.iter().min_by_key(|r| r.experience)?;
    let slow_class = classes.iter().max_by_key(|c| c.experience)?;
    let quick_class = classes.iter().min_by_key(|c| c.experience)?;
    let named = |race: &warlock_core::content::RaceDef, class: &warlock_core::content::ClassDef| {
        let name = format!("{} {}", race.name, class.name.to_lowercase());
        (name, Factor::new(Some(race), Some(class)))
    };
    Some([named(quick_race, quick_class), named(slow_race, slow_class)])
}

/// The XP needed for each level (spec §8 "Experience"), every level to 20 and then every fifth:
/// the kills of the player's own level the level costs, its XP, the total for a character of
/// 100 %, the totals for the quickest and the slowest race and class, and whether it grows hit
/// points.
pub(super) fn experience_levels(content: &Content) -> String {
    let rules = content.experience();
    let examples = extremes(content);
    let mut out = String::from("| Level | Kills of your level | XP for the level | XP needed |");
    let mut align = String::from("|:---:|---:|---:|---:|");
    for (name, factor) in examples.iter().flatten() {
        let _ = write!(out, " {name} ({} %) |", factor.percent());
        align.push_str("---:|");
    }
    let _ = writeln!(out, " Hit points gained |\n{align}:---:|");
    for level in (2..=rules.max_level()).filter(|l| *l <= 20 || l % 5 == 0) {
        let _ = write!(
            out,
            "| {level} | {} | {} | {} |",
            rules.kills_for(level),
            thousands(rules.xp_for(level)),
            thousands(rules.threshold(level))
        );
        for (_, factor) in examples.iter().flatten() {
            let _ = write!(out, " {} |", thousands(rules.threshold_for(level, *factor)));
        }
        let hp = if level <= rules.hp_level() {
            "yes"
        } else {
            "no"
        };
        let _ = writeln!(out, " {hp} |");
    }
    out
}

/// The race's and the class's experience factors, with what the game says of them.
pub(super) fn experience_factors(content: &Content) -> String {
    let words = |percent: u16| pace_words(percent).unwrap_or("");
    let mut out =
        String::from("| Race | XP a level needs | When you choose it |\n|---|:---:|---|\n");
    for r in &content.races().list {
        let _ = writeln!(
            out,
            "| {} | {} % | {} |",
            r.name,
            r.experience,
            words(r.experience)
        );
    }
    out.push_str("\n| Class | XP a level needs | When you choose it |\n|---|:---:|---|\n");
    for c in content.classes() {
        let _ = writeln!(
            out,
            "| {} | {} % | {} |",
            c.name,
            c.experience,
            words(c.experience)
        );
    }
    out
}

/// What discovery is worth: the first kill of each creature type and the first arrival on a
/// level.
pub(super) fn experience_discovery(content: &Content) -> String {
    let rules = content.experience();
    let found = rules.discovery();
    let mut out = String::from("| Discovery | Experience |\n|---|---|\n");
    let kill = match found.first_kill {
        0 => "nothing".to_string(),
        100 => "its XP once more, whatever your levels".to_string(),
        p => format!("{p} % of its XP once more, whatever your levels"),
    };
    let _ = writeln!(out, "| Your first kill of each kind of creature | {kill} |");
    let arrival = |depth: u8| rules.arrival_xp(depth);
    let depths = if found.arrival == 0 {
        "nothing".to_string()
    } else {
        format!(
            "{} × its depth: {} on the first level, {} on the fifth, {} on the tenth",
            found.arrival,
            arrival(1),
            arrival(5),
            arrival(10)
        )
    };
    let _ = writeln!(out, "| Your first arrival on a dungeon level | {depths} |");
    let hand_made = if found.arrival == 0 {
        "nothing".to_string()
    } else {
        format!(
            "{} × its [item level](magic-items.md): {} on most, {} in the arena",
            found.arrival,
            arrival(1),
            arrival(content.item_level("arena"))
        )
    };
    let _ = writeln!(
        out,
        "| Your first arrival on another level, made by hand | {hand_made} |"
    );
    let _ = writeln!(
        out,
        "| The level you start on, the overworld, the land around it | nothing |"
    );
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

/// The creatures' levels and XP, beside the guideline's XP for their level.
pub(super) fn experience_creatures(content: &Content) -> String {
    let guideline = content.experience().guideline();
    let mut out = String::from(
        "| Creature | Level | XP | The guideline for its level |\n|---|:---:|---:|---:|\n",
    );
    // A family (animated blades) once, its first member's.
    let mut list: Vec<_> = (content.creature_defs().iter().enumerate())
        .filter(|&(i, c)| c.id != "player" && content.family(CreatureId(i as u16)).0 == i as u16)
        .map(|(_, c)| c)
        .collect();
    list.sort_by_key(|c| (c.level, c.xp, c.name.to_lowercase()));
    for c in list {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} |",
            capital(&c.name),
            c.level,
            c.xp,
            guideline.xp(c.level)
        );
    }
    out
}
