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

/// Frame-time measurement: runs the release game offscreen with `--capture-after-frames` N and
/// M, and divides the wall-clock difference by M - N, which cancels startup and capture cost.
/// `WARLOCK_UNPACED=1` removes the game's 60 Hz capture-mode pacing, so this measures CPU + GPU
/// throughput per frame (offscreen render, no presentation).
pub fn frames(window: &str) -> Result<(), String> {
    use crate::workspace_root;
    use std::process::Command;

    let root = workspace_root();
    let status = Command::new(env!("CARGO"))
        .current_dir(&root)
        .args(["build", "--quiet", "--release", "--package", "warlock"])
        .status()
        .map_err(|e| format!("failed to run cargo: {e}"))?;
    if !status.success() {
        return Err(format!("release build failed: {status}"));
    }
    let exe = root.join("target/release/warlock");
    let out = root.join("target/xtask/bench-frame.png");
    std::fs::create_dir_all(out.parent().expect("has parent")).map_err(|e| e.to_string())?;
    const LOW: u32 = 10;
    const HIGH: u32 = 2010;
    const RUNS: usize = 5;
    println!(
        "{:<6} {:>10} {:>12} {:>10}  verdict ({window}, scale 1, best of {RUNS} runs, {} frames)",
        "level",
        "fps",
        "ms/frame",
        "budget ms",
        HIGH - LOW
    );
    let mut failed = false;
    for (scenario, label) in [("showcase", "hub"), ("vast", "vast")] {
        let time = |frames: u32| -> Result<f64, String> {
            let mut best = f64::MAX;
            for _ in 0..RUNS {
                let start = Instant::now();
                let status = Command::new(&exe)
                    .current_dir(&root)
                    .env("WARLOCK_UNPACED", "1")
                    .args(["--scenario", scenario, "--window", window, "--scale", "1"])
                    .args(["--capture-after-frames", &frames.to_string(), "--capture"])
                    .arg(&out)
                    .status()
                    .map_err(|e| format!("failed to run the game: {e}"))?;
                if !status.success() {
                    return Err(format!("game exited with {status}"));
                }
                best = best.min(start.elapsed().as_secs_f64());
            }
            Ok(best)
        };
        let ms = (time(HIGH)? - time(LOW)?) * 1000.0 / f64::from(HIGH - LOW);
        let budget = 1000.0 / 60.0;
        let ok = ms <= budget;
        failed |= !ok;
        println!(
            "{label:<6} {:>10.1} {ms:>12.3} {budget:>10.2}  {}",
            1000.0 / ms,
            if ok { "ok" } else { "OVER BUDGET" }
        );
    }
    if failed {
        Err("frame time over budget".into())
    } else {
        Ok(())
    }
}
