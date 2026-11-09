//! Miscasts (#1271, #1299, `docs/balance.md` "Miscasts"): each casting class's Spellcraft, the
//! bonus of its check on a cast (`Actor::cast_bonus`) and its ward (`Actor::miscast_ward`),
//! trained, in plain gear (out of armour), the seeds' mean; then the chance of a miscast (in 144
//! ways, `check::ways`) for a spell of level 1, of 11, and of the highest it knows, at that mean
//! bonus; then where a miscast lands on the table (the miscasts design's bands, §2.4) for both
//! wizard careers; and the creature casters' chance for each spell of their lists.
//!
//! The wizard is measured twice: Spellcraft first (the career's own, every skill roll on
//! Spellcraft, `SPELLCRAFT_ROLLS`) and Concentration first, the same wizard whose rolls went where
//! `levelup:auto` puts them (Concentration, its first trained skill): its Spellcraft stays at its
//! level-1 value, so its bonus and ward are the career's with Spellcraft's `cast` and `ward` read
//! at that value instead.

use super::{Gear, LEVELS, Probe, Snap, avg};
use crate::check;
use crate::combat;
use crate::content::{Content, MiscastRules};
use crate::skills::Effect;
use crate::world::{ActorId, World};
use std::fmt::Write as _;

/// The classes measured: the wizard (Spellcraft first), the priest and the witch, and the
/// paladin, a half-caster.
const CLASSES: [&str; 4] = ["wizard", "priest", "witch", "paladin"];

/// The Concentration-first wizard's row label.
const WIZARD_CONCENTRATION: &str = "wizard (Concentration first)";

/// The bands of the miscast table (the miscasts design §3): the last total of each but the last,
/// catastrophe, which is open. `safe_below` (71) is where *severe* starts.
const BANDS: [(&str, i32); 5] = [
    ("harmless", 16),
    ("mild", 40),
    ("annoying", 70),
    ("severe", 90),
    ("deadly", 110),
];

/// The spells (level, rank) the bands table casts.
const BAND_SPELLS: [(u8, u8); 5] = [(11, 1), (20, 1), (20, 5), (30, 1), (30, 10)];

/// The levels the bands table measures.
const BAND_LEVELS: [u8; 5] = [1, 10, 20, 30, 60];

/// A miscast's chance at `bonus` against `difficulty`, as a percent.
fn chance(bonus: i32, difficulty: i32) -> f64 {
    f64::from(144 - check::ways(bonus, difficulty)) * 100.0 / 144.0
}

/// Spellcraft's `effect` at value `value` (its benefit curve).
fn spellcraft_at(content: &Content, effect: Effect, value: i32) -> i32 {
    let Some(id) = content.skill_id("spellcraft") else {
        return 0;
    };
    let value = u16::try_from(value.max(0)).unwrap_or(u16::MAX);
    (content.skill(id).benefits.iter())
        .filter(|b| b.effect == effect)
        .map(|b| b.at(value))
        .sum()
}

/// One career's measure at a level: Spellcraft, the check's bonus and ward (seeds' means), and
/// the highest spell level known.
struct Measure {
    spellcraft: f64,
    bonus: f64,
    ward: f64,
    top: Option<u8>,
}

/// `class`'s measure at `level`; for [`WIZARD_CONCENTRATION`], the wizard's with Spellcraft held
/// at each seed's level-1 value.
fn measure(p: &Probe, class: &str, level: u8) -> Option<Measure> {
    let concentration = class == WIZARD_CONCENTRATION;
    let of = if concentration { "wizard" } else { class };
    let seeds = p.of(of, level, Gear::Plain, true);
    if seeds.is_empty() {
        return None;
    }
    let firsts = p.of(of, 1, Gear::Plain, true);
    let held = |s: &Snap, i: usize| -> (i32, i32, i32) {
        if !concentration {
            return (s.spellcraft, s.cast_bonus, s.ward);
        }
        let start = firsts.get(i).map_or(s.spellcraft, |f| f.spellcraft);
        let at = |e, v| spellcraft_at(&p.content, e, v);
        (
            start,
            s.cast_bonus - at(Effect::Cast, s.spellcraft) + at(Effect::Cast, start),
            s.ward - at(Effect::Ward, s.spellcraft) + at(Effect::Ward, start),
        )
    };
    let rows: Vec<(i32, i32, i32)> = seeds.iter().enumerate().map(|(i, s)| held(s, i)).collect();
    Some(Measure {
        spellcraft: avg(rows.iter().map(|r| f64::from(r.0))),
        bonus: avg(rows.iter().map(|r| f64::from(r.1))),
        ward: avg(rows.iter().map(|r| f64::from(r.2))),
        top: (seeds.iter())
            .filter_map(|s| s.known_levels.iter().copied().max())
            .max(),
    })
}

/// The ways (out of `die`'s outcomes) each total of `die` comes up: (total, ways).
fn die_ways(rules: &MiscastRules) -> Vec<(i32, u64)> {
    let (count, sides) = (rules.die.count(), rules.die.sides());
    let mut ways = vec![(rules.die.bonus(), 1u64)];
    for _ in 0..count {
        let mut next = std::collections::BTreeMap::new();
        for &(t, w) in &ways {
            for face in 1..=i32::try_from(sides).unwrap_or(1) {
                *next.entry(t + face).or_insert(0) += w;
            }
        }
        ways = next.into_iter().collect();
    }
    ways
}

/// Where a miscast lands at `bonus` and `ward` against `difficulty`: the chance of a miscast
/// (percent), each band's share of the miscasts (percent), and the worst total.
fn bands(rules: &MiscastRules, bonus: i32, ward: i32, difficulty: i32) -> (f64, [f64; 6], i32) {
    let die = die_ways(rules);
    let die_total: u64 = die.iter().map(|d| d.1).sum();
    let mut share = [0u64; 6];
    let mut worst = 0;
    let mut failed = 0u64;
    for natural in combat::FUMBLE..=combat::HIGHEST {
        if check::makes(natural, bonus, difficulty) {
            continue;
        }
        let ways = u64::from(combat::ways(natural));
        failed += ways;
        let miss = (difficulty - natural - bonus).max(0);
        for &(rolled, w) in &die {
            let total = (rolled + rules.per_margin * miss - ward).max(1);
            worst = worst.max(total);
            let band = BANDS.iter().position(|b| total <= b.1).unwrap_or(5);
            share[band] += ways * w;
        }
    }
    let all = (failed * die_total).max(1) as f64;
    (
        failed as f64 * 100.0 / 144.0,
        share.map(|s| s as f64 * 100.0 / all),
        worst,
    )
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
         brackets), at rank 1. The wizard twice: every skill roll on Spellcraft, and the same \
         wizard with its rolls on Concentration (its Spellcraft stays where it started).\n"
    )
    .unwrap();
    writeln!(
        out,
        "| Class | L | Spellcraft | Bonus | Ward | Level 1 | Level 11 | Highest known |\n\
         |---|---:|---:|---:|---:|---:|---:|---:|"
    )
    .unwrap();
    let mut classes: Vec<&str> = CLASSES.to_vec();
    classes.insert(1, WIZARD_CONCENTRATION);
    for class in &classes {
        for level in LEVELS {
            let Some(m) = measure(p, class, level) else {
                continue;
            };
            let b = m.bonus.round() as i32;
            let at = |l: u8| chance(b, rules.difficulty(l, 0, 1));
            let highest =
                (m.top).map_or_else(|| "—".to_string(), |l| format!("{:.1} % ({l})", at(l)));
            writeln!(
                out,
                "| {class} | {level} | {:.0} | {:+.1} | {:.0} | {:.1} % | {:.1} % | {highest} |",
                m.spellcraft,
                m.bonus,
                m.ward,
                at(1),
                at(11)
            )
            .unwrap();
        }
    }
    writeln!(
        out,
        "\nWhere a wizard's miscast lands (the miscasts design's bands: harmless 1–16, mild \
         17–40, annoying 41–70, severe 71–90, deadly 91–110, catastrophe 111+; below 71 is the \
         protected half), out of armour, in the neutral stance: the chance of a miscast, each \
         band's share of the miscasts in percent, and the worst total.\n"
    )
    .unwrap();
    writeln!(
        out,
        "| Wizard | L | Bonus | Ward | Spell (level, rank) | Miscast | Harmless | Mild | \
         Annoying | Severe | Deadly | Catastrophe | Worst |\n\
         |---|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|"
    )
    .unwrap();
    for (label, class) in [
        ("Spellcraft first", "wizard"),
        ("Concentration first", WIZARD_CONCENTRATION),
    ] {
        for level in BAND_LEVELS {
            let Some(m) = measure(p, class, level) else {
                continue;
            };
            let (b, ward) = (m.bonus.round() as i32, m.ward.round() as i32);
            for (spell, rank) in BAND_SPELLS {
                let (miscast, share, worst) =
                    bands(rules, b, ward, rules.difficulty(spell, 0, rank));
                let share: Vec<String> = share.iter().map(|s| format!("{s:.1}")).collect();
                writeln!(
                    out,
                    "| {label} | {level} | {b:+} | {ward} | {spell}, r{rank} | {miscast:.1} % | {} | {worst} |",
                    share.join(" | ")
                )
                .unwrap();
            }
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
