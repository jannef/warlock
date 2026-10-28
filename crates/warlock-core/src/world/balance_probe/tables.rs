//! The balance probe's tables (the parent module's docs): each writes one section of
//! `docs/balance.md` as Markdown.

use super::{
    Gear, LEVELS, Probe, STRIKES, Side, Snap, armoured, avg, bony, dice_text, fight, fight_cell,
    hide, hit_mean, mean, median, r,
};
use crate::combat::{self, Category, Profile, Target};
use crate::content::Resolution;
use crate::rng::Rng;
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

/// The casters' damage spells.
pub(super) fn spells(p: &Probe, rng: &mut Rng, out: &mut String) {
    writeln!(out, "\n## Spells\n").unwrap();
    writeln!(
        out,
        "Each damage spell as the trained caster casts it (plain gear): the lift and the cap it lifts, the dice, and the expected casts to kill the median monster of their level (an attack spell's miss chance included; per target), lifted (unlifted in brackets); mana per cast of the pool.\n"
    )
    .unwrap();
    for class in ["wizard", "priest"] {
        writeln!(
            out,
            "**{}**\n\n| Spell (tier, resolution) | L | lift % | cap → lifted | dice | casts to kill | mana / pool |\n|---|---|---|---|---|---|---|",
            if class == "wizard" { "Wizard" } else { "Priest" }
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
                "| none: no damage spell it knows or of a school it has an affinity with |||||||"
            )
            .unwrap();
        }
        for name in &names {
            for level in [10u8, 30, 60] {
                let s: Vec<&Snap> = p
                    .of(class, level, Gear::Plain, true)
                    .into_iter()
                    .filter(|s| s.casts.iter().any(|c| &c.name == name))
                    .collect();
                let Some(first) = s.first() else {
                    continue;
                };
                let w = p.window(level, -2, 2);
                let c = first.casts.iter().find(|c| &c.name == name).expect("cast");
                let mut per_lifted = Vec::new();
                let mut per_plain = Vec::new();
                for m in &w {
                    let mut lifted = Vec::new();
                    let mut plain = Vec::new();
                    for (k, snap) in s.iter().enumerate() {
                        let cast = snap.casts.iter().find(|c| &c.name == name).expect("cast");
                        let them = &m.geared[k % m.geared.len()];
                        for (dice, cap, into) in [
                            (cast.dice.0, cast.lifted, &mut lifted),
                            (cast.dice.1, cast.cap, &mut plain),
                        ] {
                            let total: i64 = (0..STRIKES)
                                .map(|_| {
                                    let amount = match cast.resolution {
                                        Resolution::Attack | Resolution::Touch => {
                                            let p = Profile {
                                                damage: dice,
                                                ..cast.aim.clone()
                                            };
                                            let t = Target::bare(them.missile);
                                            combat::strike(&p, &t, rng).damage
                                        }
                                        Resolution::Auto | Resolution::Area => {
                                            dice.roll(rng).max(0)
                                        }
                                    };
                                    i64::from(crate::world::spell::capped(amount, Some(cap)))
                                })
                                .sum();
                            let dmg = total as f64 / f64::from(STRIKES);
                            into.push(f64::from(them.hp) / dmg.max(0.01));
                        }
                    }
                    per_lifted.push(avg(lifted));
                    per_plain.push(avg(plain));
                }
                let mana = avg(s.iter().map(|s| f64::from(s.mana)));
                writeln!(
                    out,
                    "| {name} ({}, {:?}) | {level} | {} | {} → {} | {}, {:.1} | {} ({}) | {} / {:.0} |",
                    c.tier,
                    c.resolution,
                    c.lift,
                    c.cap,
                    c.lifted,
                    dice_text(c.dice.0),
                    mean(c.dice.0),
                    r(median(per_lifted)),
                    r(median(per_plain)),
                    c.cost,
                    mana,
                )
                .unwrap();
            }
        }
        writeln!(out).unwrap();
    }
}
