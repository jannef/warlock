//! Dragons (#1099, `docs/balance.md` "Dragons"): each `combat = "dragon"` type against the
//! players of the probe's levels around its own, trained and not, in plain gear. A dragon
//! breathes on its first action and again whenever its cooldown has run out (a degree of its
//! cooldown status a normal action), striking with every natural attack in between; the cells
//! are the expected values of that sequence, the breath's roll before Evasion (none in plain
//! gear) but after the player's resistance.

use super::{Gear, Probe, Side, avg, mean, melee_mean, r, wrathed_mean};
use crate::damage::resist;
use crate::rng::Rng;
use std::fmt::Write as _;

/// The classes measured, as the "Fights" table's plain columns.
const CLASSES: [&str; 4] = ["warrior", "thief", "wizard", "priest"];

/// The probe's levels on either side of a dragon's: the last below it and the first from it.
fn levels_around(level: u8) -> Vec<u8> {
    let below = super::LEVELS.iter().rev().find(|&&l| l < level).copied();
    let from = super::LEVELS.iter().find(|&&l| l >= level).copied();
    below.into_iter().chain(from).collect()
}

/// The drake's actions until the player at `hp` falls, the breath (`breath` a hit) coming every
/// `cycle` actions from the first, the claws and bite (`melee` an action) in between.
fn actions_to_die(hp: f64, breath: f64, melee: f64, cycle: u32) -> f64 {
    let mut taken = 0.0;
    for action in 0..1000u32 {
        let this = if action % cycle == 0 { breath } else { melee };
        if taken + this >= hp {
            return f64::from(action) + (hp - taken) / this.max(0.01);
        }
        taken += this;
    }
    f64::INFINITY
}

pub(super) fn dragons(p: &Probe, rng: &mut Rng, out: &mut String) {
    writeln!(out, "\n## Dragons\n").unwrap();
    writeln!(
        out,
        "Each dragon (#1099) against the players of the probe's levels around its own, in plain gear, trained → untrained (the seeds' mean). **HP** the player's; **breath** its mean after the player's resistance, and that over their hit points; **melee** the claws and bite of one action, misses included; **a round** the dragon's damage an action over a cooldown's cycle (one breath, then a melee action for each degree of its cooldown); **die** the dragon's actions until the player falls, breathing first; **kill** the player's rounds to kill it.\n"
    )
    .unwrap();
    writeln!(
        out,
        "| Dragon | L | Class | HP | breath (of HP) | melee | a round | die | kill |\n|---|---|---|---|---|---|---|---|---|"
    )
    .unwrap();
    for m in &p.monsters {
        let def = p.content.creature(m.creature);
        let Some(breath) = &def.breath else {
            continue;
        };
        let cycle = u32::from(breath.degrees) + 1;
        let them: &Side = &m.bare;
        for level in levels_around(m.level) {
            for class in CLASSES {
                let mut cells = Vec::new();
                for trained in [true, false] {
                    let snaps = p.of(class, level, Gear::Plain, trained);
                    let mut rows = Vec::new();
                    for s in &snaps {
                        let me = &s.side;
                        let rolled = mean(breath.damage).round() as i32;
                        let kind = Some(breath.kind);
                        let burnt = f64::from(resist(rolled, me.target.resistances.get(kind)));
                        let melee = melee_mean(them, &me.target, rng) * them.blows;
                        let a_round = (burnt + melee * f64::from(cycle - 1)) / f64::from(cycle);
                        let die = actions_to_die(f64::from(me.hp), burnt, melee, cycle);
                        let mut dealt = wrathed_mean(&me.blow, None, &them.target, rng);
                        if let Some(off) = &me.off {
                            dealt += wrathed_mean(off, None, &them.target, rng);
                        }
                        let kill = f64::from(them.hp) / (dealt * me.blows).max(0.01);
                        rows.push((f64::from(me.hp), burnt, melee, a_round, die, kill));
                    }
                    cells.push((
                        avg(rows.iter().map(|r| r.0)),
                        avg(rows.iter().map(|r| r.1)),
                        avg(rows.iter().map(|r| r.2)),
                        avg(rows.iter().map(|r| r.3)),
                        avg(rows.iter().map(|r| r.4)),
                        avg(rows.iter().map(|r| r.5)),
                    ));
                }
                let [t, u] = [cells[0], cells[1]];
                writeln!(
                    out,
                    "| {} | {level} | {class} | {:.0}→{:.0} | {:.0} ({:.0}%→{:.0}%) | {:.1}→{:.1} | {:.1}→{:.1} | {}→{} | {}→{} |",
                    def.name,
                    t.0,
                    u.0,
                    t.1,
                    100.0 * t.1 / t.0.max(1.0),
                    100.0 * u.1 / u.0.max(1.0),
                    t.2,
                    u.2,
                    t.3,
                    u.3,
                    r(t.4),
                    r(u.4),
                    r(t.5),
                    r(u.5),
                )
                .unwrap();
            }
        }
    }
}
