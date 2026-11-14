//! The druid's calls (#839, `docs/balance.md` "Summons"): at each level a call comes or the
//! druid's abilities change it, a druid made by `World::start_run` and levelled with `xp:` and
//! `levelup:auto` casts it through the game's own cast path; the table says how many come, for
//! how long, its mana over the druid's pool, and how each summoned beast fares against the
//! monsters within two levels of the druid (the "Fights" cells, the beast's bare side).

use super::{Monster, Probe, fight_cell, fight_sides};
use crate::action::Action;
use crate::character::CharacterSpec;
use crate::content::{Factor, SpellPurpose};
use crate::rng::Rng;
use crate::world::World;
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// The druid's levels the table measures: each call's first and the abilities that raise them.
const LEVELS: [u8; 6] = [6, 25, 32, 40, 50, 60];

/// A human druid at `level`, every choice spent, mana full.
fn druid(p: &Probe, level: u8) -> World {
    let spec: CharacterSpec = "human,female,druid".parse().expect("a character");
    let mut w = World::start_run(
        p.content.clone(),
        "showcase",
        Some(1),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let factor = Factor::of(&p.content, w.state.character.as_ref().map(|c| &c.spec));
    let need = p.content.experience().threshold_for(level, factor);
    w.apply(&format!("xp:{need}").parse().expect("xp"));
    while w.due().is_some() {
        w.apply(&"levelup:auto".parse().expect("levelup"));
    }
    let max = w.state.player().max_mana(&p.content);
    w.player_mut().mana = max;
    w
}

pub(super) fn summons(p: &Probe, rng: &mut Rng, out: &mut String) {
    writeln!(out, "\n## Summons\n").unwrap();
    writeln!(
        out,
        "The druid's calls (#839), cast by a human druid (untrained stats, every score as rolled, seed 1) at each level a call comes or Pack leader (32) and Lord of the wild (60) change them: what comes of one cast (each type's count), for how many minutes (`lasting`: until slain), the mana over the pool, and each called type against the monsters within two levels of the druid, as in \"Fights\" (the beast's side bare: P(it hits) / P(it is hit) % · damage a round · rounds to kill / rounds to die).\n"
    )
    .unwrap();
    writeln!(
        out,
        "| Call | L | comes | minutes | mana / pool | the called against the druid's level |\n|---|---|---|---|---|---|"
    )
    .unwrap();
    let calls: Vec<_> = (p.content.spells_by_level().into_iter())
        .filter(|&s| p.content.spell(s).kind == SpellPurpose::Summon)
        .collect();
    for &level in &LEVELS {
        for &spell in &calls {
            let def = p.content.spell(spell);
            let mut w = druid(p, level);
            if !w.state.player().known_spells(&p.content).contains(&spell) {
                continue;
            }
            let player = w.state.player;
            let now = w.time();
            let cost = w.state.player().spell_cost(&p.content, def);
            let pool = w.state.player().max_mana(&p.content);
            w.apply(&Action::Cast(def.id.clone(), None));
            let mut came: BTreeMap<crate::content::CreatureId, (usize, Vec<u64>)> = BTreeMap::new();
            for a in w
                .state
                .actors
                .values()
                .filter(|a| a.master() == Some(player))
            {
                let entry = came.entry(a.creature).or_default();
                entry.0 += 1;
                if let Some(until) = a.ally.as_ref().and_then(|x| x.until) {
                    entry.1.push((until.seconds() - now.seconds()) / 60);
                }
            }
            let comes: Vec<String> = (came.iter())
                .map(|(&id, (c, _))| {
                    let def = p.content.creature(id);
                    let name = if *c == 1 { &def.name } else { &def.plural };
                    format!("{c} {name}")
                })
                .collect();
            let minutes: Vec<u64> = came.values().flat_map(|(_, m)| m.clone()).collect();
            let minutes = match minutes.first() {
                None => "lasting".to_string(),
                Some(m) => m.to_string(),
            };
            let window = p.window(level, -2, 2);
            let mut cells = Vec::new();
            for &creature in def.effect.summon.as_ref().map_or(&[][..], |c| &c.pool[..]) {
                let Some(beast) = p
                    .monsters
                    .iter()
                    .find(|m: &&Monster| m.creature == creature)
                else {
                    continue;
                };
                let f = fight_sides(&[&beast.bare], &window, None, rng);
                cells.push(format!(
                    "{} {}: {}",
                    p.content.creature(creature).name,
                    beast.level,
                    fight_cell(&f)
                ));
            }
            writeln!(
                out,
                "| {} ({}) | {level} | {} | {minutes} | {cost} / {pool} ({:.0} casts) | {} |",
                def.name,
                def.level,
                comes.join(", "),
                f64::from(pool) / f64::from(cost.max(1)),
                cells.join("; ")
            )
            .unwrap();
        }
    }
}
