//! Miscasts (#1271, `docs/balance.md` "Miscasts"): each casting class's Spellcraft, the bonus of
//! its check on a cast (`Actor::cast_bonus`) and its ward (`Actor::miscast_ward`), trained, in
//! plain gear (out of armour), the seeds' mean; then the chance of a miscast (in 144 ways,
//! `check::ways`) for a spell of level 1, of 11, and of the highest it knows, at that mean bonus;
//! and the creature casters' chance for each spell of their lists.

use super::{Gear, LEVELS, Probe, avg};
use crate::check;
use crate::world::{ActorId, World};
use std::fmt::Write as _;

/// The classes measured.
const CLASSES: [&str; 3] = ["wizard", "priest", "witch"];

/// A miscast's chance at `bonus` against `difficulty`, as a percent.
fn chance(bonus: i32, difficulty: i32) -> f64 {
    f64::from(144 - check::ways(bonus, difficulty)) * 100.0 / 144.0
}

pub(super) fn miscasts(p: &Probe, out: &mut String) {
    let content = &p.content;
    let Some(rules) = content.miscast_rules() else {
        return;
    };
    writeln!(
        out,
        "\n## Miscasts\n\nEach class's Spellcraft, the bonus of its check on a cast and its ward \
         (trained, plain gear, the seeds' mean, the bonus rounded), and the chance of a miscast \
         for a spell of level 1, of level 11, and of the highest level it knows (that level in \
         brackets).\n"
    )
    .unwrap();
    writeln!(
        out,
        "| Class | L | Spellcraft | Bonus | Ward | Level 1 | Level 11 | Highest known |\n\
         |---|---:|---:|---:|---:|---:|---:|---:|"
    )
    .unwrap();
    for class in CLASSES {
        for level in LEVELS {
            let seeds = p.of(class, level, Gear::Plain, true);
            if seeds.is_empty() {
                continue;
            }
            let skill = avg(seeds.iter().map(|s| f64::from(s.spellcraft)));
            let bonus = avg(seeds.iter().map(|s| f64::from(s.cast_bonus)));
            let ward = avg(seeds.iter().map(|s| f64::from(s.ward)));
            let top = (seeds.iter())
                .filter_map(|s| s.known_levels.iter().copied().max())
                .max();
            let b = bonus.round() as i32;
            let at = |l: u8| chance(b, rules.difficulty(l, 0));
            let highest = top.map_or_else(|| "—".to_string(), |l| format!("{:.1} % ({l})", at(l)));
            writeln!(
                out,
                "| {class} | {level} | {skill:.0} | {bonus:+.1} | {ward:.0} | {:.1} % | {:.1} % | {highest} |",
                at(1),
                at(11)
            )
            .unwrap();
        }
    }
    writeln!(
        out,
        "\nCreature casters: the chance of a miscast for each spell of their lists (their \
         Spellcraft and Intelligence as their type makes them).\n"
    )
    .unwrap();
    writeln!(
        out,
        "| Creature | L | Spellcraft | Bonus | Spells |\n|---|---:|---:|---:|---|"
    )
    .unwrap();
    for m in p.monsters.iter().filter(|m| !m.spells.is_empty()) {
        let (spellcraft, bonus) = m.cast;
        let spells: Vec<String> = (m.spells.iter())
            .map(|s| format!("{} {:.1} %", s.name, chance(bonus, s.difficulty)))
            .collect();
        writeln!(
            out,
            "| {} | {} | {spellcraft} | {bonus:+} | {} |",
            content.creature(m.creature).name,
            m.level,
            spells.join(", ")
        )
        .unwrap();
    }
}

/// Creature `id`'s Spellcraft in `w` and the bonus of its check on a cast.
pub(super) fn cast_of(w: &World, id: ActorId) -> (i32, i32) {
    let content = &w.content;
    let me = &w.state.actors[&id];
    let spellcraft = (content.skill_id("spellcraft"))
        .and_then(|s| me.skills.value(s))
        .map_or(0, i32::from);
    (spellcraft, me.cast_bonus(content))
}
