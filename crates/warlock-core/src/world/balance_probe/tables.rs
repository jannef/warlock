//! The balance probe's tables (the parent module's docs): each writes one section of
//! `docs/balance.md` as Markdown.

use super::{
    Cast, Gear, LEVELS, Probe, STRIKES, Side, Snap, armoured, avg, bony, dice_text, fight,
    fight_cell, hide, hit_mean, hit_pct, median, r, roll_mean, strike_mean,
};
use crate::combat::{self, Category, Profile, Target};
use crate::dice::Dice;
use crate::rng::Rng;
use crate::stats::Stat;
use std::fmt::Write as _;

/// The players: to-hit, damage, Defence, Protection; stats, hit points, mana.
pub(super) fn players(p: &Probe, out: &mut String) {
    writeln!(out, "## Players\n").unwrap();
    writeln!(
        out,
        "Each cell: to-hit / damage a hit / Dv / Pv, the seeds' mean; trained stats.\n"
    )
    .unwrap();
    writeln!(
        out,
        "| L | Warrior plain | Warrior magic | Thief plain | Thief magic | Wizard plain | Priest plain | Priest magic |"
    )
    .unwrap();
    writeln!(out, "|---|---|---|---|---|---|---|---|").unwrap();
    for &level in &LEVELS {
        let mut row = format!("| {level} |");
        for (class, gear) in [
            ("warrior", Gear::Plain),
            ("warrior", Gear::Magic),
            ("thief", Gear::Plain),
            ("thief", Gear::Magic),
            ("wizard", Gear::Plain),
            ("priest", Gear::Plain),
            ("priest", Gear::Magic),
        ] {
            let s = p.of(class, level, gear, true);
            let th = avg(s.iter().map(|s| f64::from(s.side.blow.to_hit)));
            let dmg = avg(s.iter().map(|s| hit_mean(&s.side.blow)));
            let dv = avg(s.iter().map(|s| f64::from(s.side.target.defence.total())));
            let pv = avg(s.iter().map(|s| f64::from(s.side.target.protection.base)));
            write!(row, " {th:+.0}/{dmg:.1}/{dv:.0}/{pv:.0} |").unwrap();
        }
        writeln!(out, "{row}").unwrap();
    }

    writeln!(
        out,
        "\nStats, hit points, and mana (plain gear; trained → untrained). Scores St/Dx/To/In/Wi/Pr, the seeds' mean.\n"
    )
    .unwrap();
    writeln!(
        out,
        "| L | Warrior | Thief | Wizard | Priest | HP W/T/Wz/P | MP Wz/P | Caster level Wz/P | Blows a round W/T |"
    )
    .unwrap();
    writeln!(out, "|---|---|---|---|---|---|---|---|---|").unwrap();
    for &level in &LEVELS {
        let mut row = format!("| {level} |");
        for class in ["warrior", "thief", "wizard", "priest"] {
            let t = p.of(class, level, Gear::Plain, true);
            let scores: Vec<String> = (0..6)
                .map(|i| format!("{:.0}", avg(t.iter().map(|s| f64::from(s.scores[i])))))
                .collect();
            write!(row, " {} |", scores.join("/")).unwrap();
        }
        let hp = |class, trained| {
            avg(p
                .of(class, level, Gear::Plain, trained)
                .iter()
                .map(|s| f64::from(s.side.hp)))
        };
        let hps: Vec<String> = ["warrior", "thief", "wizard", "priest"]
            .iter()
            .map(|c| format!("{:.0}→{:.0}", hp(c, true), hp(c, false)))
            .collect();
        write!(row, " {} |", hps.join(" / ")).unwrap();
        let mp = |class| {
            avg(p
                .of(class, level, Gear::Plain, true)
                .iter()
                .map(|s| f64::from(s.mana)))
        };
        let cl = |class| {
            avg(p
                .of(class, level, Gear::Plain, true)
                .iter()
                .map(|s| f64::from(s.caster_level)))
        };
        let blows = |class| {
            avg(p
                .of(class, level, Gear::Plain, true)
                .iter()
                .map(|s| s.side.blows))
        };
        writeln!(
            out,
            "{row} {:.0}/{:.0} | {:.0}/{:.0} | {:.2}/{:.2} |",
            mp("wizard"),
            mp("priest"),
            cl("wizard"),
            cl("priest"),
            blows("warrior"),
            blows("thief")
        )
        .unwrap();
    }
}

/// The Defence of the warrior and the thief by category.
pub(super) fn defence(p: &Probe, out: &mut String) {
    writeln!(
        out,
        "\nThe Defence of the warrior and the thief in plain gear by category (trained → untrained), the seeds' mean.\n"
    )
    .unwrap();
    let cats: Vec<&str> = Category::ALL.iter().map(|c| c.name()).collect();
    writeln!(out, "| Class | L | {} | total |", cats.join(" | ")).unwrap();
    writeln!(out, "|---|---|{}---|", "---|".repeat(cats.len())).unwrap();
    for class in ["warrior", "thief"] {
        for &level in &[10u8, 30, 60] {
            let by = |trained: bool, c: Option<Category>| {
                avg(p.of(class, level, Gear::Plain, trained).iter().map(|s| {
                    let d = &s.side.target.defence;
                    f64::from(c.map_or_else(|| d.total(), |c| d.bonus(c)))
                }))
            };
            let cells: Vec<String> = (Category::ALL.into_iter().map(Some).chain([None]))
                .map(|c| format!("{:.0}→{:.0}", by(true, c), by(false, c)))
                .collect();
            writeln!(out, "| {class} | {level} | {} |", cells.join(" | ")).unwrap();
        }
    }
}

/// The monsters by level band.
pub(super) fn monsters(p: &Probe, out: &mut String) {
    writeln!(out, "\n## Monsters\n").unwrap();
    writeln!(
        out,
        "Within two levels: the median (range) bare → the mean with gear. Damage is a hit's mean before soak.\n"
    )
    .unwrap();
    writeln!(
        out,
        "| L (±2) | n | HP | to-hit | dmg/blow | Dv | Pv | blows a round |"
    )
    .unwrap();
    writeln!(out, "|---|---|---|---|---|---|---|---|").unwrap();
    for &level in LEVELS.iter().chain(&[70]) {
        let w = p.window(level, -2, 2);
        let stat = |f: &dyn Fn(&Side) -> f64| {
            let bare: Vec<f64> = w.iter().map(|m| f(&m.bare)).collect();
            let lo = bare.iter().copied().fold(f64::INFINITY, f64::min);
            let hi = bare.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let geared = avg(w.iter().flat_map(|m| m.geared.iter().map(f)));
            format!("{:.0} ({lo:.0}–{hi:.0}) → {geared:.1}", median(bare))
        };
        writeln!(
            out,
            "| {level} | {} | {:.0} | {} | {} | {} | {} | {:.2} |",
            w.len(),
            median(w.iter().map(|m| f64::from(m.bare.hp)).collect()),
            stat(&|s| f64::from(s.blow.to_hit)),
            stat(&|s| hit_mean(&s.blow)),
            stat(&|s| f64::from(s.target.defence.total())),
            stat(&|s| f64::from(s.target.protection.base)),
            median(w.iter().map(|m| m.bare.blows).collect()),
        )
        .unwrap();
    }
}

/// The fights at the player's level.
pub(super) fn fights(p: &Probe, rng: &mut Rng, out: &mut String) {
    writeln!(out, "\n## Fights\n").unwrap();
    writeln!(
        out,
        "Each cell: P(player hits) / P(monster hits) % · damage a round player : monster · rounds to kill / rounds to die; the median over the monsters within two levels (trained stats, monsters with gear).\n"
    )
    .unwrap();
    let columns = [
        ("Warrior kit", "warrior", Gear::Kit),
        ("Warrior plain", "warrior", Gear::Plain),
        ("Warrior magic", "warrior", Gear::Magic),
        ("Thief plain", "thief", Gear::Plain),
        ("Thief magic", "thief", Gear::Magic),
        ("Wizard plain (melee)", "wizard", Gear::Plain),
        ("Priest plain", "priest", Gear::Plain),
        ("Priest magic", "priest", Gear::Magic),
    ];
    let head: Vec<&str> = columns.iter().map(|c| c.0).collect();
    writeln!(out, "| L | {} |", head.join(" | ")).unwrap();
    writeln!(out, "|---|{}", "---|".repeat(columns.len())).unwrap();
    for &level in &LEVELS {
        let w = p.window(level, -2, 2);
        let mut row = format!("| {level} |");
        for (_, class, gear) in columns {
            let f = fight(&p.of(class, level, gear, true), &w, None, rng);
            write!(row, " {} |", fight_cell(&f)).unwrap();
        }
        writeln!(out, "{row}").unwrap();
    }
}

/// Out of depth, and untrained stats.
pub(super) fn depth(p: &Probe, rng: &mut Rng, out: &mut String) {
    writeln!(
        out,
        "\nOut of depth (+3: the monsters one to five levels above), and untrained stats, for the warrior in plain gear.\n"
    )
    .unwrap();
    writeln!(
        out,
        "| L | Warrior plain +3 | Warrior magic +3 | Warrior plain, untrained | Warrior magic, untrained |"
    )
    .unwrap();
    writeln!(out, "|---|---|---|---|---|").unwrap();
    for &level in &LEVELS {
        let same = p.window(level, -2, 2);
        let above = p.window(level, 1, 5);
        let cells = [
            fight(
                &p.of("warrior", level, Gear::Plain, true),
                &above,
                None,
                rng,
            ),
            fight(
                &p.of("warrior", level, Gear::Magic, true),
                &above,
                None,
                rng,
            ),
            fight(
                &p.of("warrior", level, Gear::Plain, false),
                &same,
                None,
                rng,
            ),
            fight(
                &p.of("warrior", level, Gear::Magic, false),
                &same,
                None,
                rng,
            ),
        ];
        let cells: Vec<String> = cells.iter().map(fight_cell).collect();
        writeln!(out, "| {level} | {} |", cells.join(" | ")).unwrap();
    }
}

/// Protection by body (§8.4).
pub(super) fn bodies(p: &Probe, rng: &mut Rng, out: &mut String) {
    writeln!(out, "\n## Protection by body\n").unwrap();
    writeln!(
        out,
        "Rounds for the plain warrior (trained) to kill the monsters of their level: as shipped, then with every monster's Protection set to §8.2's body line at its level.\n"
    )
    .unwrap();
    writeln!(
        out,
        "| L | as shipped | none (0) | hide | bony | armoured |"
    )
    .unwrap();
    writeln!(out, "|---|---|---|---|---|---|").unwrap();
    for &level in &LEVELS[2..] {
        let w = p.window(level, -2, 2);
        let s = p.of("warrior", level, Gear::Plain, true);
        let lines: [Option<fn(u8) -> i32>; 5] =
            [None, Some(|_| 0), Some(hide), Some(bony), Some(armoured)];
        let cells: Vec<String> = lines
            .iter()
            .map(|pv| r(fight(&s, &w, *pv, rng).kill))
            .collect();
        writeln!(out, "| {level} | {} |", cells.join(" | ")).unwrap();
    }
}

/// The casters, by class name and title, and the school each casts in.
const CASTERS: [(&str, &str, &str); 3] = [
    ("wizard", "Wizard", "evocation"),
    ("priest", "Priest", "restoration"),
    ("witch", "Witch", "necromancy"),
];

/// The levels the spell table prints (the design's §12.3).
const SPELL_LEVELS: [u8; 4] = [10, 20, 30, 60];

/// The one value of `v`, or `lo–hi` when the seeds differ.
fn span<T: Ord + Copy + std::fmt::Display>(v: impl IntoIterator<Item = T>) -> String {
    let v: Vec<T> = v.into_iter().collect();
    match (v.iter().min(), v.iter().max()) {
        (Some(lo), Some(hi)) if lo == hi => format!("{lo}"),
        (Some(lo), Some(hi)) => format!("{lo}–{hi}"),
        _ => "–".into(),
    }
}

/// A reroll in words, after the dice: ` 1s again`, ` 1s and 2s again`.
fn again(reroll: u32) -> String {
    match reroll {
        0 => String::new(),
        1 => " 1s again".into(),
        2 => " 1s and 2s again".into(),
        r => format!(" 1s to {r}s again"),
    }
}

/// The casters' spells that roll: the roll with the school's rank, and at rank 0.
pub(super) fn spells(p: &Probe, out: &mut String) {
    writeln!(out, "\n## Spells\n").unwrap();
    writeln!(
        out,
        "Each spell that rolls (harm or healing) of the schools the trained caster knows or has an affinity with, as `Actor::spell_roll` makes it in plain gear, per cast on one target: the rung of its ladder the stats reach (the seeds' range), the school's rank the pace below reached (and the rank of the caster style it is cast in from where the caster stands), the roll (seed 1's dice, the seeds' mean) and the same roll at rank 0 (the ladder alone), and its mana (the cost table's price with no rank's cut in brackets) over the pool.\n"
    )
    .unwrap();
    for (class, title, _) in CASTERS {
        writeln!(
            out,
            "**{title}**\n\n| Spell (level, res) | L | rung | school rank | roll | mana / pool |\n|---|---|---|---|---|---|"
        )
        .unwrap();
        let mut names: Vec<String> = Vec::new();
        for snap in p.of(class, 60, Gear::Plain, true) {
            for c in &snap.casts {
                if !names.contains(&c.name) {
                    names.push(c.name.clone());
                }
            }
        }
        if names.is_empty() {
            writeln!(
                out,
                "| none: no spell that rolls it knows or has an affinity with ||||||"
            )
            .unwrap();
        }
        for name in &names {
            for level in SPELL_LEVELS {
                let s: Vec<&Snap> = p.of(class, level, Gear::Plain, true);
                let c: Vec<&Cast> = (s.iter())
                    .filter_map(|s| s.casts.iter().find(|c| &c.name == name))
                    .collect();
                let Some(first) = c.first() else {
                    continue;
                };
                let mut rank = span(c.iter().map(|c| c.school_rank));
                if let Some((style, _)) = &first.style {
                    let ranks = span(c.iter().filter_map(|c| c.style.as_ref().map(|s| s.1)));
                    rank = format!("{rank} ({style} {ranks})");
                }
                let mana = avg(s.iter().map(|s| f64::from(s.mana)));
                writeln!(
                    out,
                    "| {name} ({}, {}) | {level} | {} | {rank} | {}{}, {:.1} (rank 0: {}{}, {:.1}) | {} ({}) / {mana:.0} |",
                    first.level,
                    format!("{:?}", first.resolution).to_lowercase(),
                    span(c.iter().map(|c| c.rung)),
                    first.dice,
                    again(first.reroll),
                    avg(c.iter().map(|c| roll_mean(c.dice, c.reroll))),
                    first.bare,
                    again(first.bare_reroll),
                    avg(c.iter().map(|c| roll_mean(c.bare, c.bare_reroll))),
                    first.cost,
                    first.base_cost,
                )
                .unwrap();
            }
        }
        writeln!(out).unwrap();
    }
}

/// The schools' pace: the ranks a pool a day reaches by level, and the pool check.
pub(super) fn school_pace(p: &Probe, out: &mut String) {
    writeln!(out, "\n## School pace\n").unwrap();
    writeln!(
        out,
        "The trained caster's school and caster style as their casting trained them (`balance_probe/schools.rs`): each day of each level, one pool spent on the pace spell (of the school's spells, the most a cast among those costing at most a fifth of the pool, never one cast only under the open sky; for the priest a heal, at most one a kill), every cast landing on a monster of the caster's level through `World::practise`. Each row: the pace spell over the level before and its cost (seed 1's); casts a day from the pool, and the spell casts a day the same level's fights took (the stat training's careers, which refill the pool before every cast; the priest's one a kill); the school's rank and the caster style's at the level (the seeds' range); and the pool check, the pool over a single-target spell of the caster's own level (its base mana: the level; no rank's cut, no talent).\n"
    )
    .unwrap();
    let table = p.content.weapon_skills();
    for (class, title, school) in CASTERS {
        writeln!(
            out,
            "**{title}** ({school})\n\n| L | pace spell (mana) | casts a day: pool / fights | school rank | style rank | pool / level-L casts |\n|---|---|---|---|---|---|"
        )
        .unwrap();
        let id = table.id(school).expect("school");
        for level in LEVELS {
            let s = p.of(class, level, Gear::Plain, true);
            let Some(first) = s.first() else {
                continue;
            };
            let spell = match &first.paced.spell {
                Some((name, cost)) => format!("{name} ({cost})"),
                None => "–".into(),
            };
            let style = match &first.paced.style {
                Some(name) => {
                    let ranks = (s.iter()).filter_map(|s| s.style_rank);
                    format!("{name} {}", span(ranks))
                }
                None => "–".into(),
            };
            let mana = avg(s.iter().map(|s| f64::from(s.mana)));
            writeln!(
                out,
                "| {level} | {spell} | {:.1} / {:.1} | {} | {style} | {mana:.0} / {level} = {:.1} |",
                avg(s.iter().map(|s| s.paced.casts_a_day)),
                avg(s.iter().map(|s| s.fought_a_day)),
                span(s.iter().map(|s| s.ranks.rank(id))),
                mana / f64::from(level.max(1)),
            )
            .unwrap();
        }
        writeln!(out).unwrap();
    }
}

/// The mean damage of `STRIKES` strikes of `p` against `t`, misses included, with `dice` added
/// to each hit as a sneak attack's land (`World::ambushed`).
fn sneak_mean(p: &Profile, t: &Target, dice: Option<Dice>, rng: &mut Rng) -> f64 {
    let total: i64 = (0..STRIKES)
        .map(|_| {
            let blow = combat::strike(p, t, rng);
            let extra = match dice {
                Some(d) if blow.strike.is_hit() => d.roll(rng).max(0),
                _ => 0,
            };
            i64::from(blow.damage + extra)
        })
        .sum();
    total as f64 / f64::from(STRIKES)
}

/// The thief's first blow on a foe unaware of them against its plain blow (#688).
pub(super) fn sneak_attacks(p: &Probe, rng: &mut Rng, out: &mut String) {
    writeln!(out, "## Sneak attacks\n").unwrap();
    writeln!(
        out,
        "The thief's main-hand blow on the monsters of their level (trained stats), aware and caught unawares. The Sneak attack value assumed, the dice it gives with the talents, and the share of the monsters within two levels with vitals (no dice and no critical on the rest, so their first blow gains only the flat-footed Defence); then each cell: P(hit) aware / unaware % · damage of a blow aware : the first blow unaware · how many times as much, the median over the monsters within two levels that have vitals.\n"
    )
    .unwrap();
    writeln!(
        out,
        "| L | Sneak attack | dice | with vitals | Thief plain | Thief magic |\n|---|---|---|---|---|---|"
    )
    .unwrap();
    for &level in &LEVELS {
        let w = p.window(level, -2, 2);
        let fleshy: Vec<_> = (w.iter())
            .filter(|m| p.content.has_vitals(m.creature))
            .collect();
        let share = 100.0 * fleshy.len() as f64 / w.len().max(1) as f64;
        let mut row = String::new();
        let mut dice = String::from("-");
        for gear in [Gear::Plain, Gear::Magic] {
            let snaps = p.of("thief", level, gear, true);
            if let Some(d) = snaps.first().and_then(|s| s.sneak) {
                dice = dice_text(d);
            }
            let mut rows = Vec::new();
            for m in &fleshy {
                let mut per = Vec::new();
                for (s, snap) in snaps.iter().enumerate() {
                    let them = &m.geared[s % m.geared.len()];
                    let blow = &snap.side.blow;
                    let flat = them.flat.clone();
                    let mut sneak = blow.clone();
                    sneak.threat = sneak.threat.saturating_sub(snap.ambush_threat).max(3);
                    let dice = snap.sneak;
                    per.push((
                        hit_pct(blow.to_hit, &them.target.defence),
                        hit_pct(blow.to_hit, &flat.defence),
                        strike_mean(blow, &them.target, rng),
                        sneak_mean(&sneak, &flat, dice, rng),
                    ));
                }
                rows.push(per);
            }
            let pick = |f: fn(&(f64, f64, f64, f64)) -> f64| {
                median(rows.iter().map(|per| avg(per.iter().map(f))).collect())
            };
            let (aware, unaware) = (pick(|r| r.2), pick(|r| r.3));
            write!(
                row,
                " {:.0}/{:.0} · {:.1}:{:.1} · ×{:.1} |",
                pick(|r| r.0),
                pick(|r| r.1),
                aware,
                unaware,
                unaware / aware.max(0.01)
            )
            .unwrap();
        }
        writeln!(
            out,
            "| {level} | {} | {dice} | {share:.0}% |{row}",
            super::sneak_value(level)
        )
        .unwrap();
    }
}

const CLASSES: [&str; 4] = ["warrior", "thief", "wizard", "priest"];

/// Stat training (#727): the trained careers' scores by level, the marks a day the hooks gave,
/// the first rises, against the start and the potential.
pub(super) fn stats(p: &Probe, out: &mut String) {
    writeln!(out, "\n## Stat training\n").unwrap();
    writeln!(
        out,
        "Scores of the trained careers (plain gear; the seeds' mean), start/potential at level 1, and the gain and the points past the potential at 60.\n"
    )
    .unwrap();
    writeln!(
        out,
        "| Class | Stat | Start/potential | L10 | L20 | L30 | L40 | L60 | Gain | Past potential |"
    )
    .unwrap();
    writeln!(out, "|---|---|---|---|---|---|---|---|---|---|").unwrap();
    for class in CLASSES {
        let at = |level| p.of(class, level, Gear::Plain, true);
        let mean = |level, i: usize| avg(at(level).iter().map(|s| f64::from(s.scores[i])));
        let mut all = 0.0;
        for (i, stat) in Stat::ALL.into_iter().enumerate() {
            let start = mean(1, i);
            let pot = avg(at(1).iter().map(|s| f64::from(s.potentials[i])));
            let end = mean(60, i);
            let past = avg(at(60)
                .iter()
                .map(|s| f64::from(s.scores[i].saturating_sub(s.potentials[i]))));
            all += end - start;
            writeln!(
                out,
                "| {} | {} | {start:.1}/{pot:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {end:.1} | {:+.1} | {past:.1} |",
                if i == 0 { class } else { "" },
                stat.abbrev(),
                mean(10, i),
                mean(20, i),
                mean(30, i),
                mean(40, i),
                end - start,
            )
            .unwrap();
        }
        writeln!(out, "| | all six | | | | | | | **{all:+.1}** | |").unwrap();
    }

    writeln!(
        out,
        "\nMarks a day the hooks gave in each stat over the level before (St/Dx/To/In/Wi/Pr, the seeds' mean), and the game day reached.\n"
    )
    .unwrap();
    writeln!(out, "| L | Day | Warrior | Thief | Wizard | Priest |").unwrap();
    writeln!(out, "|---|---|---|---|---|---|").unwrap();
    for &level in LEVELS.iter().skip(1) {
        let day = avg(p
            .of("warrior", level, Gear::Plain, true)
            .iter()
            .map(|s| f64::from(s.day)));
        let mut row = format!("| {level} | {day:.0} |");
        for class in CLASSES {
            let s = p.of(class, level, Gear::Plain, true);
            let marks: Vec<String> = (0..6)
                .map(|i| format!("{:.0}", avg(s.iter().map(|s| s.marks_a_day[i]))))
                .collect();
            write!(row, " {} |", marks.join("/")).unwrap();
        }
        writeln!(out, "{row}").unwrap();
    }

    writeln!(
        out,
        "\nThe game day of each stat's first rise (the seeds' mean of those that rose by level 60; how many of the six in brackets when not all).\n"
    )
    .unwrap();
    writeln!(out, "| Class | St | Dx | To | In | Wi | Pr |").unwrap();
    writeln!(out, "|---|---|---|---|---|---|---|").unwrap();
    for class in CLASSES {
        let s = p.of(class, 60, Gear::Plain, true);
        let mut row = format!("| {class} |");
        for i in 0..6 {
            let days: Vec<f64> = s
                .iter()
                .filter_map(|s| s.first_rise[i].map(f64::from))
                .collect();
            let cell = match days.len() {
                0 => "never".to_string(),
                n if n == s.len() => format!("{:.0}", avg(days)),
                n => format!("{:.0} ({n})", avg(days)),
            };
            write!(row, " {cell} |").unwrap();
        }
        writeln!(out, "{row}").unwrap();
    }
}

/// The sweep of `price_per_point` and `ramp_divisor` (design §6.2): each class's mean gain by
/// level 60 per stat, and in all.
pub(super) fn stat_sweep(p: &Probe, out: &mut String) {
    writeln!(
        out,
        "\nThe knobs (design §6.2): mean gain by level 60 with other `price_per_point` and `ramp_divisor` values (St/Dx/To/In/Wi/Pr · all six), the data's in bold.\n"
    )
    .unwrap();
    writeln!(
        out,
        "| Price | Ramp ÷ | Warrior | Thief | Wizard | Priest |"
    )
    .unwrap();
    writeln!(out, "|---|---|---|---|---|---|").unwrap();
    let data = p.content.stat_training();
    for swept in &p.sweep {
        let bold = swept.per_point == data.price_per_point && swept.ramp == data.ramp_divisor;
        let mut row = match bold {
            true => format!(
                "| **{} + {}s** | **{}** |",
                data.price_base, swept.per_point, swept.ramp
            ),
            false => format!(
                "| {} + {}s | {} |",
                data.price_base, swept.per_point, swept.ramp
            ),
        };
        for class in CLASSES {
            let start = p.of(class, 1, Gear::Plain, true);
            let finals: Vec<&[u8; 6]> = (swept.finals.iter())
                .filter(|(c, _)| *c == class)
                .map(|(_, s)| s)
                .collect();
            let gains: Vec<f64> = (0..6)
                .map(|i| {
                    avg(finals.iter().map(|s| f64::from(s[i])))
                        - avg(start.iter().map(|s| f64::from(s.scores[i])))
                })
                .collect();
            let all: f64 = gains.iter().sum();
            let each: Vec<String> = gains.iter().map(|g| format!("{g:+.1}")).collect();
            write!(row, " {} · {all:+.0} |", each.join("/")).unwrap();
        }
        writeln!(out, "{row}").unwrap();
    }
}
