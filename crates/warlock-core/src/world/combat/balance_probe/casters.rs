//! Casters (#1102, `docs/balance.md` "Casters"): each `combat = "caster"` type's spells as it
//! casts them (`creature_cast.rs`: `Actor::spell_roll` at its own stats, `Actor::spell_cost`
//! from its pool) against the trained players of the first probe level from its own, in plain
//! gear. The cells are expected values: its first damage spell's roll (an `attack` spell's
//! times its hit chance against the player's Defence against missiles, its level's quarter on
//! the roll; `auto` and `area` always land, no Evasion in plain gear), its casts of that spell
//! a full pool pays for, and its damage a round while the mana lasts, against its melee a round
//! once it is spent (the hexes, buffs, and heals of its list left out).

use super::{Gear, Probe, Side, hit_pct, melee_mean, r, roll_mean};
use crate::content::{ActionKind, NORMAL_SPEED, Resolution, SpellPurpose};
use crate::rng::Rng;
use crate::world::{ActorId, CastBy, World};
use std::fmt::Write as _;

/// The classes measured.
const CLASSES: [&str; 2] = ["warrior", "wizard"];

/// One spell of a caster's list as it casts it.
#[derive(Clone)]
pub(super) struct Spell {
    pub(super) name: String,
    kind: SpellPurpose,
    resolution: Resolution,
    /// Its roll's mean, and the roll as text.
    mean: f64,
    roll: String,
    /// Its to-hit (an `attack` or `touch` spell's), its level's quarter on top.
    to_hit: i32,
    cost: i32,
    /// The difficulty of its Spellcraft check on a cast (`miscasts.rs`).
    pub(super) difficulty: i32,
}

/// A caster's spells and pool, made bare in `w` as `id`.
pub(super) fn spells_of(w: &World, id: ActorId) -> (Vec<Spell>, i32) {
    let content = &w.content;
    let me = &w.state.actors[&id];
    let def = content.creature(me.creature);
    let spells = (def.spells.iter())
        .map(|&s| {
            let spell = content.spell(s);
            let roll = me.spell_roll(content, spell, CastBy::Own);
            Spell {
                name: spell.name.to_lowercase(),
                kind: spell.kind,
                resolution: spell.resolution,
                mean: roll.map_or(0.0, |r| roll_mean(r, 0)),
                roll: roll.map_or_else(String::new, |r| r.to_string()),
                to_hit: me.spell_to_hit(content, spell) + i32::from(def.level) / 4,
                cost: me.spell_cost(content, spell),
                difficulty: me.cast_difficulty(content, (s, spell)).unwrap_or(0),
            }
        })
        .collect();
    (spells, me.max_mana(content))
}

/// The table.
pub(super) fn casters(p: &Probe, rng: &mut Rng, out: &mut String) {
    writeln!(out, "\n## Casters\n").unwrap();
    writeln!(
        out,
        "Each caster (#1102) against the trained players of the first probe level from its own, in plain gear (the seeds' mean). **Spell** its first damage spell and its roll at its stats; **casts** how many of it its full pool pays for; **a cast** its mean damage (an `attack` spell's times its hit chance); **a round** that times its casts a round, against **melee** its blows a round once the mana is spent; **die** the player's rounds until they fall, the spells first, then its melee.\n"
    )
    .unwrap();
    writeln!(
        out,
        "| Caster | L | Spell (roll) | casts | Player | HP | a cast | a round | melee | die |\n|---|---|---|---|---|---|---|---|---|---|"
    )
    .unwrap();
    for m in &p.monsters {
        let def = p.content.creature(m.creature);
        if def.spells.is_empty() {
            continue;
        }
        let Some(spell) = (m.spells.iter()).find(|s| s.kind == SpellPurpose::Damage) else {
            continue;
        };
        let Some(&level) = super::LEVELS.iter().find(|&&l| l >= m.level) else {
            continue;
        };
        let casts = m.mana / spell.cost.max(1);
        // Its actions a round (a normal turn): its speed over a cast's cost.
        let cost = p.content.action_cost(m.creature, ActionKind::Cast);
        let per_round = f64::from(def.speed) / f64::from(NORMAL_SPEED) * 1000.0 / f64::from(cost);
        let them: &Side = &m.bare;
        for class in CLASSES {
            let snaps = p.of(class, level, Gear::Plain, true);
            let mut rows = Vec::new();
            for s in &snaps {
                let me = &s.side;
                let lands = match spell.resolution {
                    Resolution::Attack | Resolution::Touch => {
                        hit_pct(spell.to_hit, &me.missile) / 100.0
                    }
                    Resolution::Auto | Resolution::Area => 1.0,
                };
                let a_cast = spell.mean * lands;
                let a_round = a_cast * per_round;
                let melee = melee_mean(them, &me.target, rng) * them.blows;
                let hp = f64::from(me.hp);
                let by_spells = a_cast * f64::from(casts);
                let die = if by_spells >= hp {
                    hp / a_round.max(0.01)
                } else {
                    f64::from(casts) / per_round + (hp - by_spells) / melee.max(0.01)
                };
                rows.push((hp, a_cast, a_round, melee, die));
            }
            let n = rows.len().max(1) as f64;
            let avg =
                |f: fn(&(f64, f64, f64, f64, f64)) -> f64| rows.iter().map(f).sum::<f64>() / n;
            writeln!(
                out,
                "| {} | {} | {} ({}) | {casts} | {class} {level} | {:.0} | {:.1} | {:.1} | {:.1} | {} |",
                def.name,
                m.level,
                spell.name,
                spell.roll,
                avg(|r| r.0),
                avg(|r| r.1),
                avg(|r| r.2),
                avg(|r| r.3),
                r(avg(|r| r.4)),
            )
            .unwrap();
        }
    }
}
