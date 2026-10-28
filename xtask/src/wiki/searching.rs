//! Searching (spec §8 "Searching"): how hard the hidden things are to find, deeper down too, and
//! who notices them in passing. Chances for an average character (Intelligence and Willpower 10,
//! untrained, no racial bonus), never a difficulty or a modifier.

use super::combat::percent;
use super::text::capital;
use std::fmt::Write as _;
use warlock_core::check;
use warlock_core::content::{Content, Generator, PassiveSearch};
use warlock_core::dice::Dice;
use warlock_core::skills::{Effect, SkillId};

/// How many turns the "ten turns" column searches.
const TURNS: i32 = 10;

/// The chance of `ways` in 144 at least once in `turns` tries, in percent.
fn within(ways: u32, turns: i32) -> String {
    let miss = f64::from(144 - ways) / 144.0;
    let p = 100.0 * (1.0 - miss.powi(turns));
    if p >= 99.5 && ways < 144 {
        "over 99%".into()
    } else {
        format!("{p:.0}%")
    }
}

/// The three chances of a row: a turn of searching, ten of them, a turn in passing.
fn chances(content: &Content, difficulty: i32) -> String {
    let active = check::ways(0, difficulty);
    let passive = check::ways(-content.search().passive_penalty, difficulty);
    format!(
        "{} | {} | {}",
        percent(active, 144),
        within(active, TURNS),
        percent(passive, 144)
    )
}

/// The hidden things and how often an average character finds each: secret doors (where the
/// place does not say otherwise), then the traps by name.
pub(super) fn search_features(content: &Content) -> String {
    let mut out = String::from(
        "| Hidden thing | What gives it away | A turn of searching | Ten turns | A turn in passing \
         |\n|---|---|:---:|:---:|:---:|\n",
    );
    let door = i32::from(content.search().secret_door);
    let _ = writeln!(
        out,
        "| Secret door | A seam in the wall | {} |",
        chances(content, door)
    );
    let mut traps: Vec<_> = content.trap_defs().iter().collect();
    traps.sort_by(|a, b| a.name.cmp(&b.name));
    for t in traps {
        let _ = writeln!(
            out,
            "| {} | {} | {} |",
            capital(&t.name),
            capital(&t.sign),
            chances(content, i32::from(t.difficulty))
        );
    }
    out
}

/// Deeper in a dungeon, hidden things are harder: a secret door at each depth of the branches
/// that make them so, for an average character.
pub(super) fn search_depths(content: &Content) -> String {
    let mut out = String::from(
        "| Level | Secret doors per level | A secret door: a turn of searching | Ten turns | A turn \
         in passing |\n|---|:---:|:---:|:---:|:---:|\n",
    );
    let mut branches: Vec<_> = content.dungeons().collect();
    branches.sort_by(|a, b| a.name.cmp(&b.name));
    let door = i32::from(content.search().secret_door);
    for def in branches {
        if def.hidden_per_depth == 0 && def.secret_doors[1] == 0 {
            continue;
        }
        for depth in 1..=def.depth {
            let rooms = def.plan(depth).map(|p| p.generator) == Some(Generator::Rooms);
            let doors = match def.secret_doors {
                _ if !rooms => "none (caves have no doors)".to_string(),
                [0, 0] => "none".to_string(),
                [least, most] if least == most => least.to_string(),
                [least, most] => format!("{least} to {most}"),
            };
            let _ = writeln!(
                out,
                "| {} | {doors} | {} |",
                def.level_name(depth),
                chances(content, door + def.hidden_at(depth))
            );
        }
    }
    out
}

/// Who notices hidden things in passing, by race (by name): by their race, by its training in
/// Spot (everything but traps) or Detect traps (traps), or only by their class's; and what the
/// race adds to finding them.
pub(super) fn search_races(content: &Content) -> String {
    let mut out =
        String::from("| Race | Notices in passing | Finds hidden things |\n|---|---|:---:|\n");
    let trains = |skills: &[(SkillId, Dice)], effect: Effect| {
        (skills.iter())
            .any(|(id, _)| (content.skill(*id).benefits.iter()).any(|b| b.effect == effect))
    };
    let mut races: Vec<_> = content.races().list.iter().collect();
    races.sort_by(|a, b| a.name.cmp(&b.name));
    for r in races {
        let spot = trains(&r.skills, Effect::Spot);
        let traps = trains(&r.skills, Effect::DetectTraps);
        let notices = match r.passive_search {
            Some(PassiveSearch::All) => "Everything",
            Some(PassiveSearch::Stone) if traps => {
                "What is hidden in stone, and hidden traps (trained in Detect traps)"
            }
            Some(PassiveSearch::Stone) => "What is hidden in stone",
            Some(PassiveSearch::Traps) => "Hidden traps",
            None if spot && traps => "Everything (trained in Spot and Detect traps)",
            None if spot => "Everything but traps (trained in Spot)",
            None if traps => "Hidden traps (trained in Detect traps)",
            None => "Only if their class trains Spot or Detect traps",
        };
        let bonus = match r.search {
            0 => "—".to_string(),
            n => format!("more easily ({n:+})"),
        };
        let _ = writeln!(out, "| {} | {notices} | {bonus} |", r.name);
    }
    for (effect, skill, what) in [
        (Effect::Spot, "Spot", "everything but traps"),
        (Effect::DetectTraps, "Detect traps", "hidden traps"),
    ] {
        let mut classes: Vec<&str> = (content.classes().iter())
            .filter(|c| trains(&c.skills, effect))
            .map(|c| c.name.as_str())
            .collect();
        classes.sort_unstable();
        if !classes.is_empty() {
            let _ = write!(
                out,
                "\nClasses trained in {skill}, who notice {what} in passing: {}.\n",
                classes.join(", ")
            );
        }
    }
    out
}
