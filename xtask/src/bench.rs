//! `cargo xtask bench-turns`: turn-processing time in the hub, the vast level, and the first
//! generated level of the depths (spec §10 budgets; a generated 80×21 level gets the hub's).

use std::sync::Arc;
use std::time::Instant;
use warlock_core::content::Content;
use warlock_core::{Action, Dir, World};

const TURNS: usize = 400;

/// (scenario, level label, budget in ms)
const CASES: [(&str, &str, f64); 3] = [
    ("showcase", "hub", 5.0),
    ("vast", "vast", 16.0),
    ("depths", "depths", 5.0),
];

/// Tiny deterministic generator for the scripted walk (kept local; the game RNG is not involved).
fn next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state >> 33
}

struct Stats {
    avg_ms: f64,
    max_ms: f64,
    turns: usize,
}

fn measure(content: &Arc<Content>, scenario: &str) -> Result<Stats, String> {
    let mut world = World::new(content.clone(), scenario, None).map_err(|e| e.to_string())?;
    let mut rng = 1u64;
    let mut heading = Dir::E;
    let mut times = Vec::with_capacity(TURNS);
    // Walk with a persistent heading, picking a new one when blocked; every 7th action waits.
    let mut attempts = 0;
    while times.len() < TURNS && attempts < TURNS * 20 {
        attempts += 1;
        let action = if attempts % 7 == 0 {
            Action::Wait
        } else {
            Action::Move(heading)
        };
        let start = Instant::now();
        let outcome = world.apply(&action);
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        if outcome.took_turn {
            times.push(elapsed);
        } else {
            heading = Dir::ALL[next(&mut rng) as usize % Dir::ALL.len()];
        }
    }
    if times.is_empty() {
        return Err(format!("no turns taken in scenario `{scenario}`"));
    }
    Ok(Stats {
        avg_ms: times.iter().sum::<f64>() / times.len() as f64,
        max_ms: times.iter().copied().fold(0.0, f64::max),
        turns: times.len(),
    })
}

pub fn run() -> Result<(), String> {
    if cfg!(debug_assertions) {
        eprintln!(
            "warning: debug build; run `cargo xtask bench-turns` in release for real numbers"
        );
    }
    let content = Arc::new(Content::builtin().map_err(|e| e.to_string())?);
    println!(
        "{:<6} {:>6} {:>10} {:>10} {:>10}  verdict",
        "level", "turns", "avg ms", "max ms", "budget ms"
    );
    let mut failed = false;
    for (scenario, label, budget) in CASES {
        let s = measure(&content, scenario)?;
        let ok = s.max_ms <= budget;
        failed |= !ok;
        println!(
            "{label:<6} {:>6} {:>10.3} {:>10.3} {budget:>10.1}  {}",
            s.turns,
            s.avg_ms,
            s.max_ms,
            if ok { "ok" } else { "OVER BUDGET" }
        );
    }
    if failed {
        Err("turn time over budget".into())
    } else {
        Ok(())
    }
}
