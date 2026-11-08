//! The balance probe's tables (the parent module's docs): each writes one section of
//! `docs/balance.md` as Markdown.

use super::{
    Cast, Gear, LEVELS, Monster, Probe, STRIKES, Side, Snap, armoured, avg, bony, dice_text, fight,
    fight_cell, hide, hit_mean, hit_pct, median, r, roll_mean, strike_mean,
};
use crate::combat::{self, Category, Profile, Strike, Target};
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
        "\nStats, hit points, and mana (plain gear; trained → untrained). Scores St/Dx/To/In/Wi/Pr, the seeds' mean. Decipher script (learning): the trained Decipher script and the learning it gives; the auto-rolls spend every level-up roll on the first trained skill (Concentration for both), so it stays where it starts.\n"
    )
    .unwrap();
    writeln!(
        out,
        "| L | Warrior | Thief | Wizard | Priest | HP W/T/Wz/P | MP Wz/P | Decipher script (learning) Wz/P | Blows a round W/T |"
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
        let decipher = |class| {
            let seeds = p.of(class, level, Gear::Plain, true);
            let skill = avg(seeds.iter().map(|s| f64::from(s.decipher)));
            let learning = avg(seeds.iter().map(|s| f64::from(s.learning)));
            format!("{skill:.0} ({learning:.0})")
        };
        let blows = |class| {
            avg(p
                .of(class, level, Gear::Plain, true)
                .iter()
                .map(|s| s.side.blows))
        };
        writeln!(
            out,
            "{row} {:.0}/{:.0} | {}/{} | {:.2}/{:.2} |",
            mp("wizard"),
            mp("priest"),
            decipher("wizard"),
            decipher("priest"),
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
        "Each spell that rolls (harm or healing) of the schools the trained caster knows or has an affinity with, as `Actor::spell_roll` makes it in plain gear, per cast on one target: the rung of its ladder the stats reach (the seeds' range), the school's rank the pace below reached (and the rank of the caster style it is cast in, if any), the roll (seed 1's dice, the seeds' mean) and the same roll at rank 0 (the ladder alone), its mean on a target under a hex (Malison's bite; `–` without it), and its mana (the cost table's price with no rank's cut in brackets) over the pool.\n"
    )
    .unwrap();
    for (class, title, _) in CASTERS {
        writeln!(
            out,
            "**{title}**\n\n| Spell (level, res) | L | rung | school rank | roll | hexed | mana / pool |\n|---|---|---|---|---|---|---|"
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
                "| none: no spell that rolls it knows or has an affinity with |||||||"
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
                let mean = avg(c.iter().map(|c| roll_mean(c.dice, c.reroll)));
                let hexed = match first.hexed {
                    0 => "–".to_string(),
                    bite => format!("{:.1}", mean + f64::from(bite)),
                };
                writeln!(
                    out,
                    "| {name} ({}, {}) | {level} | {} | {rank} | {}{}, {mean:.1} (rank 0: {}{}, {:.1}) | {hexed} | {} ({}) / {mana:.0} |",
                    first.level,
                    format!("{:?}", first.resolution).to_lowercase(),
                    span(c.iter().map(|c| c.rung)),
                    first.dice,
                    again(first.reroll),
                    first.bare,
                    again(first.bare_reroll),
                    avg(c.iter().map(|c| roll_mean(c.bare, c.bare_reroll))),
                    first.cost,
                    first.base_cost,
                )
                .unwrap();
            }
        }
        prayer_rows(p, class, out);
        writeln!(out).unwrap();
    }
}

/// A caster's prayer (the priest's Orison, `world/orison.rs`) in the spell table: no roll and no
/// mana, so each level's row says its name, how many things one prayer answers, and the uses it
/// has reached (seed 1's; the table is the same for every seed).
fn prayer_rows(p: &Probe, class: &str, out: &mut String) {
    for level in SPELL_LEVELS {
        let snaps = p.of(class, level, Gear::Plain, true);
        let Some((title, state)) = snaps.iter().find_map(|s| s.prayer.as_ref()) else {
            continue;
        };
        let answers = state
            .answers
            .map_or_else(|| "all".to_string(), |n| n.to_string());
        let uses: Vec<&str> = state.uses.iter().map(|u| u.id()).collect();
        let mana = avg(snaps.iter().map(|s| f64::from(s.mana)));
        writeln!(
            out,
            "| {title} (prayer) | {level} | – | – | no roll: answers {answers} of {} ({}) | – | 0 (0) / {mana:.0} |",
            uses.len(),
            uses.join(", "),
        )
        .unwrap();
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

/// The fighters whose weapon pace is printed (`weapon_pace`).
const FIGHTERS: [(&str, &str); 6] = [
    ("warrior", "Warrior"),
    ("thief", "Thief"),
    ("priest", "Priest"),
    ("berserker", "Berserker"),
    ("ascetic", "Ascetic"),
    ("paladin", "Paladin"),
];

/// The weapon pace (#1095): the level at which each trained fighter's weapon skill and style
/// first stood at each rank, as the stat training's fights trained them blow by blow.
pub(super) fn weapon_pace(p: &Probe, out: &mut String) {
    writeln!(out, "\n## Weapon pace\n").unwrap();
    writeln!(
        out,
        "The trained fighters' weapon skill and fighting style as their blows trained them (#1095; `balance_probe/training.rs`): every blow of the stat training's fights, each level's kills against the monsters within two levels of theirs, earns its marks through `World::practise` (the real `[marks]`, curves, and talents), the main hand's skill and style for both hands. Each cell: the level at which the skill first stood at the rank (the range over the seeds that fought with it; `>60` where a seed had not reached it by level 60). The untrained careers take these ranks from their trained twin; the skills the fights do not train (a shield, a second weapon skill) are set by level (`rank`, the warrior's pace).\n"
    )
    .unwrap();
    let table = p.content.weapon_skills();
    for (class, title) in FIGHTERS {
        let snaps = p.of(class, *LEVELS.last().expect("levels"), Gear::Plain, true);
        // Every skill a seed's fights trained; a seed that never fought with it is left out of
        // its cells, and the header says how many seeds did.
        let skills: std::collections::BTreeSet<_> = (snaps.iter())
            .flat_map(|s| s.reached.keys().copied())
            .collect();
        if skills.is_empty() {
            continue;
        }
        let names: Vec<String> = (skills.iter())
            .map(|id| {
                let by = snaps.iter().filter(|s| s.reached.contains_key(id)).count();
                let name = table.skill(*id).name.clone();
                if by == snaps.len() {
                    name
                } else {
                    format!("{name} ({by} of {} seeds)", snaps.len())
                }
            })
            .collect();
        writeln!(
            out,
            "**{title}**\n\n| rank | {} |\n|---|{}",
            names.join(" | "),
            "---|".repeat(names.len())
        )
        .unwrap();
        for rank in 1..=usize::from(crate::content::MAX_RANK) {
            let cells: Vec<String> = (skills.iter())
                .map(|id| {
                    let at: Vec<Option<u8>> = (snaps.iter())
                        .filter_map(|s| s.reached.get(id).map(|r| r[rank]))
                        .collect();
                    let reached: Vec<u8> = at.iter().flatten().copied().collect();
                    match (reached.is_empty(), reached.len() < at.len()) {
                        (true, _) => ">60".into(),
                        (false, true) => format!("{}–>60", reached.iter().min().expect("one")),
                        (false, false) => span(reached),
                    }
                })
                .collect();
            writeln!(out, "| {rank} | {} |", cells.join(" | ")).unwrap();
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

/// The berserker's rage (#842): the fight raging against calm, and what feeds the pool a round
/// against one foe and against three (each foe's blows that hit, soaked or not, build `struck`;
/// the berserker's criticals and kills come at their own pace whatever the crowd).
pub(super) fn rage(p: &Probe, rng: &mut Rng, out: &mut String) {
    writeln!(out, "\n## Rage\n").unwrap();
    writeln!(
        out,
        "The berserker in plain gear (a great axe; from level 5 the warrior's armour: chain mail, banded mail from 15, plate from 30), trained stats, against the monsters within two levels: calm (the stance the run starts in) and raging (Raging and the frenzy stance). The pool as the Rage ability grants it (start / most); the fight cells as in \"Fights\"; then the rage a round against one foe, the medians of hits taken × their build + criticals × theirs + kills × theirs − the decay (kills a round: one over the rounds to kill, so a fight's average); the same without the kills (a foe that outlasts the rage, or the turns before the first kill) and how many rounds the rage lasts from its start at that net (`grows` when it does not fall); and the net against three foes (three foes' hits, the same criticals and kills).\n"
    )
    .unwrap();
    writeln!(
        out,
        "| L | rage | Calm | Raging | one foe: struck + crit + kill − decay = net | one foe that does not die: net · rounds the rage lasts | three foes: net |\n|---|---|---|---|---|---|---|"
    )
    .unwrap();
    for &level in &LEVELS {
        let w = p.window(level, -2, 2);
        let snaps = p.of("berserker", level, Gear::Plain, true);
        let raging: Vec<_> = snaps.iter().filter_map(|s| s.rage.as_ref()).collect();
        let Some(first) = raging.first() else {
            continue;
        };
        let calm = fight(&snaps, &w, None, rng);
        let sides: Vec<&Side> = raging.iter().map(|r| &r.side).collect();
        let hot = super::fight_sides(&sides, &w, None, rng);
        let mut rows = Vec::new();
        for m in &w {
            let mut per = Vec::new();
            for (s, rage) in raging.iter().enumerate() {
                let them = &m.geared[s % m.geared.len()];
                let me = &rage.side;
                let share = |n: u32| f64::from(n) / f64::from(STRIKES);
                let hits = (0..STRIKES)
                    .filter(|_| combat::strike(&them.blow, &me.target, rng).strike.is_hit())
                    .count() as u32;
                let mut crits = 0.0;
                let mut dealt = 0.0;
                for blow in std::iter::once(&me.blow).chain(&me.off) {
                    let mut n = 0;
                    for _ in 0..STRIKES {
                        let b = combat::strike(blow, &them.target, rng);
                        n += u32::from(b.strike == Strike::Hit { critical: true });
                        dealt += f64::from(b.damage) / f64::from(STRIKES);
                    }
                    crits += share(n);
                }
                let kills = dealt * me.blows / f64::from(them.hp.max(1));
                per.push((
                    share(hits) * them.blows * rage.builds[0],
                    crits * me.blows * rage.builds[1],
                    kills * rage.builds[2],
                    rage.decay,
                ));
            }
            rows.push(per);
        }
        let pick = |f: fn(&(f64, f64, f64, f64)) -> f64| {
            median(rows.iter().map(|per| avg(per.iter().map(f))).collect())
        };
        let (struck, crit, kill, decay) =
            (pick(|r| r.0), pick(|r| r.1), pick(|r| r.2), pick(|r| r.3));
        let one = struck + crit + kill - decay;
        let held = struck + crit - decay;
        let three = 3.0 * struck + crit + kill - decay;
        let lasts = if held < 0.0 {
            r(f64::from(first.start) / -held)
        } else {
            "grows".into()
        };
        writeln!(
            out,
            "| {level} | {}/{} | {} | {} | {struck:.2} + {crit:.2} + {kill:.2} − {decay:.2} = {one:+.2} | {held:+.2} · {lasts} | {three:+.2} |",
            first.start,
            first.max,
            fight_cell(&calm),
            fight_cell(&hot),
        )
        .unwrap();
    }
}

/// The ascetic's ki (#846): the pool by level, and the fight with one blow an attack against the
/// fight with the flurry's second, and how long a full pool keeps the flurry up.
pub(super) fn ki(p: &Probe, rng: &mut Rng, out: &mut String) {
    writeln!(out, "\n## Ki\n").unwrap();
    writeln!(
        out,
        "The ascetic bare-handed (a brawler; from level 5 leather gloves, boots, and a wool cloak, no armour), trained stats, against the monsters within two levels. The pool's maximum as Martial arts grants it (the seeds' mean: Willpower and the level; Inner calm where the picks took it) and the ki back a round; the fight cells as in \"Fights\", with one blow an attack (no ki) and with the flurry's second blow (−2 to hit, a point of ki an attack); then the ki the flurry spends a round against what comes back, and how many rounds a full pool keeps it up (`holds` when it comes back as fast).\n"
    )
    .unwrap();
    writeln!(
        out,
        "| L | ki | back a round | One blow | Flurry | spent − back = net · rounds from full |\n|---|---|---|---|---|---|"
    )
    .unwrap();
    for &level in &LEVELS {
        let w = p.window(level, -2, 2);
        let snaps = p.of("ascetic", level, Gear::Plain, true);
        let ki: Vec<_> = snaps.iter().filter_map(|s| s.ki.as_ref()).collect();
        if ki.is_empty() {
            continue;
        }
        let singles: Vec<&Side> = ki.iter().map(|k| &k.single).collect();
        let flurries: Vec<&Side> = ki.iter().map(|k| &k.flurry).collect();
        let one = super::fight_sides(&singles, &w, None, rng);
        let two = super::fight_sides(&flurries, &w, None, rng);
        let max = avg(ki.iter().map(|k| f64::from(k.max)));
        let back = avg(ki.iter().map(|k| k.back));
        let spent = avg(ki.iter().map(|k| k.flurry.blows));
        let net = spent - back;
        let lasts = if net > 0.0 {
            r(max / net)
        } else {
            "holds".into()
        };
        writeln!(
            out,
            "| {level} | {max:.1} | {back:.2} | {} | {} | {spent:.2} − {back:.2} = {net:.2} · {lasts} |",
            fight_cell(&one),
            fight_cell(&two),
        )
        .unwrap();
    }
}

/// The paladin's zeal (#910): the pool by level, the fight with plain blows against the fight
/// with every blow a smite (on the monsters within two levels, and on the unholy among them),
/// and what a round of the plain fight builds against what a smite costs.
pub(super) fn zeal(p: &Probe, rng: &mut Rng, out: &mut String) {
    writeln!(out, "\n## Zeal\n").unwrap();
    writeln!(
        out,
        "The paladin in plain gear (a long sword; from level 5 a shield and the warrior's armour), trained stats, against the monsters within two levels. The pool's maximum as Smite grants it (the seeds' mean: Presence and the level) and the zeal back a round with time; the fight cells as in \"Fights\", with plain blows, with every blow a smite (an upper bound: a smite costs its zeal), with every blow a smite on the unholy among those monsters (the smite's die more), and from level 50 with every blow a Banishing smite on them (its dice twice over; the summoned are unmade instead, whatever their hit points, which no cell shows). From level 40 every cell has Holy wrath's die where it lands, on the unholy, and at 60 (Avatar) on every foe. Then the zeal a round of plain blows builds (the paladin's hits and the foe's, at their chances, plus the time's) against a smite's cost: the rounds of fighting a smite takes to earn, and the same with Zealot.\n"
    )
    .unwrap();
    writeln!(
        out,
        "| L | zeal | back a round | Blow | Smite | Smite on the unholy | Banishing on the unholy | built a round · rounds a smite | with Zealot |\n|---|---|---|---|---|---|---|---|---|"
    )
    .unwrap();
    for &level in &LEVELS {
        let w = p.window(level, -2, 2);
        let unholy: Vec<&Monster> = (w.iter().copied())
            .filter(|m| p.content.unholy(m.creature))
            .collect();
        let snaps = p.of("paladin", level, Gear::Plain, true);
        let zeal: Vec<_> = snaps.iter().filter_map(|s| s.zeal.as_ref()).collect();
        let Some(first) = zeal.first() else {
            continue;
        };
        let plain = fight(&snaps, &w, None, rng);
        let smites: Vec<&Side> = zeal.iter().map(|z| &z.smite).collect();
        let smite = super::fight_sides(&smites, &w, None, rng);
        let holy: Vec<&Side> = zeal.iter().map(|z| &z.unholy).collect();
        let on_unholy = if unholy.is_empty() {
            "none".to_string()
        } else {
            fight_cell(&super::fight_sides(&holy, &unholy, None, rng))
        };
        let banishing: Vec<&Side> = zeal.iter().filter_map(|z| z.banishing.as_ref()).collect();
        let banish = if banishing.is_empty() || unholy.is_empty() {
            "-".to_string()
        } else {
            fight_cell(&super::fight_sides(&banishing, &unholy, None, rng))
        };
        let max = avg(zeal.iter().map(|z| f64::from(z.max)));
        let back = avg(zeal.iter().map(|z| z.back));
        // The paladin's blows a round at their chance, the foe's (one) at theirs.
        let blows = avg(snaps.iter().map(|s| s.side.blows));
        let built = blows * plain.hit / 100.0 * first.builds[0]
            + plain.struck / 100.0 * first.builds[1]
            + back;
        let zealot = built + plain.struck / 100.0 * (first.zealot - first.builds[1]);
        writeln!(
            out,
            "| {level} | {max:.1} | {back:.2} | {} | {} | {on_unholy} | {banish} | {built:.2} · {} | {zealot:.2} · {} |",
            fight_cell(&plain),
            fight_cell(&smite),
            r(f64::from(first.cost) / built.max(0.01)),
            r(f64::from(first.cost) / zealot.max(0.01)),
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
