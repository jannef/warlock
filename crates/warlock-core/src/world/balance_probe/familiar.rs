//! The witch's familiar (#933, `docs/balance.md` "Familiar"): a human witch at each level with
//! each kind of familiar a run may draw, at its grade, and what its touch does against the
//! monsters within two levels of hers. The touch needs a hit (`familiar_touch`), so the table is
//! the hit chance, the blows a round, the hexes laid a round, and the rounds to the first one,
//! beside what the same hex costs her to cast.

use super::{Monster, Probe, hit_mean, hit_pct, r, side_of};
use crate::character::CharacterSpec;
use crate::content::Factor;
use crate::world::{AllySource, CompanionData, World};
use std::fmt::Write as _;

/// The witch's levels the table measures: those of the probe's other tables.
const LEVELS: [u8; 9] = [1, 5, 10, 15, 20, 30, 40, 50, 60];
/// The five kinds a run may draw (`familiars.toml`), each measured as the witch's own.
const KINDS: [&str; 5] = ["cat", "toad", "raven", "hare", "bat"];

/// A human witch at `level` with her familiar, every choice spent.
fn witch(p: &Probe, level: u8) -> World {
    let spec: CharacterSpec = "human,female,witch".parse().expect("a character");
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
    if need > 0 {
        w.apply(&format!("xp:{need}").parse().expect("xp"));
    }
    while w.due().is_some() {
        w.apply(&"levelup:auto".parse().expect("levelup"));
    }
    w
}

/// The lowest, median, and highest of `v`.
fn spread(mut v: Vec<f64>) -> (f64, f64, f64) {
    v.sort_by(f64::total_cmp);
    (v[0], v[v.len() / 2], v[v.len() - 1])
}

pub(super) fn familiar(p: &Probe, out: &mut String) {
    writeln!(out, "\n## Familiar\n").unwrap();
    writeln!(
        out,
        "The witch's familiar (#929, #933), a human witch (untrained stats, every score as rolled, seed 1) with each of the five kinds a run may draw in turn, at its grade (her level). Its touch is her Evil eye and needs a blow that lands, so the table is the familiar's side bare against the monsters within two levels of the witch's (as in \"Fights\", the monsters' gear left out): **hit** the chance a blow lands, lowest / median / highest over the window; **blows** a round (a normal turn); **hexes** laid a round (blows × median hit); **rounds to the first** hex (one over the hexes a round); **degrees** her Evil eye lays; **eye** the mana one cast of it costs over her pool, which every landed blow saves her; the familiar's hit points, mean damage of a hit, and to-hit.\n"
    )
    .unwrap();
    writeln!(
        out,
        "| L | familiar | hit % (low / median / high) | blows | hexes a round | rounds to the first | degrees | eye (mana / pool) | hp · mean hit · to hit |\n|---|---|---|---|---|---|---|---|---|"
    )
    .unwrap();
    let eye = p
        .content
        .spell(p.content.spell_id("evil_eye").expect("evil eye"));
    for &level in &LEVELS {
        for kind in KINDS {
            let mut w = witch(p, level);
            let Some(old) = w.familiar() else { continue };
            // The run's own draw is set aside for the kind measured, at its grade.
            let at = w.state.actors[&old].pos;
            w.state.actors.remove(&old);
            let creature = p.content.creature_id(kind).expect("a familiar kind");
            let own = p.content.creature(creature).level;
            let grade =
                (p.content.companions()).map_or(level.max(own), |r| r.growth.grade(level, own));
            let data = CompanionData {
                name: "Tibb".into(),
                grade,
                scent: None,
            };
            let id = w.make_ally(creature, at, AllySource::Familiar(data), None);
            let side = side_of(&w, id);
            let degrees = w.state.player().spell_degrees(&p.content, eye).unwrap_or(0);
            let cost = w.state.player().spell_cost(&p.content, eye);
            let pool = w.state.player().max_mana(&p.content);
            let window: Vec<&Monster> = p.window(level, -2, 2);
            let hits: Vec<f64> = (window.iter())
                .map(|m| hit_pct(side.blow.to_hit, &m.bare.target.defence))
                .collect();
            let (lo, mid, hi) = spread(hits);
            let per_round = side.blows * mid / 100.0;
            writeln!(
                out,
                "| {level} | {kind} | {lo:.0} / {mid:.0} / {hi:.0} | {:.2} | {per_round:.2} | {} | {degrees} | {cost} / {pool} | {} · {:.1} · {:+} |",
                side.blows,
                r(1.0 / per_round.max(1e-9)),
                side.hp,
                hit_mean(&side.blow),
                side.blow.to_hit,
            )
            .unwrap();
        }
    }
}
