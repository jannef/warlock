//! The ranger and their hound (#923, `docs/balance.md` "Ranger and companion"): a human
//! ranger made by `World::start_run` (the starting beast drawn as in a game) and levelled with
//! `xp:` and `levelup:auto`, in the starting kit's short sword (the bow is not measured: the
//! probe's blows are melee), against the monsters within two levels. The companion is the
//! actor `World::companion` names, so its grade, Defence, Protection, and stats are the
//! game's own; its side is read as any monster's.

use super::{LEVELS, Probe, Side, fight_cell, fight_sides, r, side_of};
use crate::character::CharacterSpec;
use crate::content::Factor;
use crate::rng::Rng;
use crate::world::World;
use std::fmt::Write as _;

/// The seeds measured (each draws its own starting beast: most are hounds).
const SEEDS: u64 = 6;

/// A human ranger at `level` of run `seed`, every choice spent.
fn ranger(p: &Probe, level: u8, seed: u64) -> World {
    let spec: CharacterSpec = "human,male,ranger".parse().expect("a character");
    let mut w = World::start_run(
        p.content.clone(),
        "ascmearc",
        Some(seed),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let factor = Factor::of(&p.content, w.state.character.as_ref().map(|c| &c.spec));
    let need = p.content.experience().threshold_for(level, factor);
    let more = need.saturating_sub(w.player_xp());
    if more > 0 {
        w.apply(&format!("xp:{more}").parse().expect("xp"));
    }
    while w.due().is_some() {
        w.apply(&"levelup:auto".parse().expect("levelup"));
    }
    // The short blades at the rank the probe assumes for the level, as every career has.
    let table = p.content.weapon_skills();
    let id = table.id("short_blades").expect("short_blades");
    let rank = super::rank(level);
    if rank > w.state.player().weapon_skills.rank(id) {
        w.player_mut().weapon_skills.raise(id, rank);
    }
    w
}

pub(super) fn ranger_and_hound(p: &Probe, rng: &mut Rng, out: &mut String) {
    writeln!(out, "\n## Ranger and companion\n").unwrap();
    writeln!(
        out,
        "The ranger (#923) and their starting beast, in the starting kit (the short sword: the probe's blows are melee, so the bow is not measured), stats as rolled, short blades at the rank the other tables assume for the level, seeds 1 to {SEEDS} (each draws its own beast), against the monsters within two levels. Fight cells as in \"Fights\" (P(it hits) / P(it is hit) % · damage a round · rounds to kill / rounds to die) for the ranger alone and the companion alone (its grade is the ranger's level: its hit points, to-hit, damage, Defence and Protection raised by `[growth]`); together, the rounds to kill with both striking (the monster's hit points over the two damages added); the last column is the ranger alone's rounds over those.\n"
    )
    .unwrap();
    writeln!(
        out,
        "| L | beasts | its hp | Ranger | Companion | Together: rounds to kill | alone over together |\n|---|---|---|---|---|---|---|"
    )
    .unwrap();
    for &level in &LEVELS {
        let window = p.window(level, -2, 2);
        let mut hunters: Vec<Side> = Vec::new();
        let mut beasts: Vec<Side> = Vec::new();
        let mut names = Vec::new();
        for seed in 1..=SEEDS {
            let w = ranger(p, level, seed);
            let Some(id) = w.companion() else {
                continue;
            };
            hunters.push(side_of(&w, w.state.player));
            beasts.push(side_of(&w, id));
            let name = p
                .content
                .creature(w.state.actors[&id].creature)
                .name
                .clone();
            if !names.contains(&name) {
                names.push(name);
            }
        }
        if hunters.is_empty() {
            continue;
        }
        let hunter_refs: Vec<&Side> = hunters.iter().collect();
        let beast_refs: Vec<&Side> = beasts.iter().collect();
        let alone = fight_sides(&hunter_refs, &window, None, rng);
        let beast = fight_sides(&beast_refs, &window, None, rng);
        let together = 1.0 / (1.0 / alone.kill.max(0.01) + 1.0 / beast.kill.max(0.01));
        let hp = beasts.iter().map(|s| f64::from(s.hp)).sum::<f64>() / beasts.len() as f64;
        writeln!(
            out,
            "| {level} | {} | {hp:.0} | {} | {} | {} | x{:.1} |",
            names.join(", "),
            fight_cell(&alone),
            fight_cell(&beast),
            r(together),
            alone.kill / together.max(0.01)
        )
        .unwrap();
    }
}
