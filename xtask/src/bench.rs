//! `cargo xtask bench-turns`: turn-processing time in the hub, the vast level, the first
//! generated level of the depths, the sunlit courtyard at dawn, the overworld, a generated
//! forest area of the overworld at dawn, the courtyard and the forest area at midnight under the
//! full moon, the arena, where a dozen creatures fight, and a generated 120×60 cave floor at
//! full population, 80 creatures of danger 8 (the vast level's budget; spec §10 budgets; a generated 80×21
//! level, the courtyard, the 100×50 overworld, an 80×40 local area, and the 72×21 arena get the
//! hub's), and the time a night's rest takes on each (dusk to dawn: 7200 turns of six seconds,
//! the overworld's too). Fights are real: a walk that gets the player killed starts over from
//! the scenario's start, untimed.

use std::sync::Arc;
use std::time::Instant;
use warlock_core::content::{Content, ContentFiles};
use warlock_core::time::{DAWN, DUSK, Duration};
use warlock_core::{Action, Dir, Rest, World};

const TURNS: usize = 400;
/// Longest a rest from dusk to dawn may take, in ms.
const REST_BUDGET_MS: f64 = 500.0;

/// From the courtyard's and the forest's start (the 1st, 06:30) to midnight under the full moon
/// (the 16th), in minutes.
const TO_FULL_MOON: u64 = (14 * 24 + 17) * 60 + 30;

/// A cave branch of the campaign's deepest kind (spec §7 "Danger levels"): a 120×60 floor of
/// danger 8 laid down at full population (80 creatures, the ceiling), every one awake to the
/// player, who has hit points enough to walk among them, and a teeming den (its sleepers on top);
/// the turn budget is the vast level's.
const DELF_BRANCH: &str = r#"
[[dungeon]]
id = "bench_delf"
name = "Bench delf"
generator = "caves"
depth = 1
size = [120, 60]
danger = 8
population = ["bat", "beetle", "kobold", "kobold_archer", "kobold_shaman", "rat"]
room_effects = { chance = 100, per_level = 1, favoured = { teeming = 100 } }
"#;
const DELF_SCENARIO: &str = "name = \"Bench delf\"\nstart_level = \"bench_delf:1\"\nseed = 1\n\
                             lantern = \"lantern\"\nhp = 900\n";

/// (scenario, level label, budget in ms, minutes skipped before the walk)
const CASES: [(&str, &str, f64, u64); 10] = [
    ("showcase", "hub", 5.0, 0),
    ("vast", "vast", 16.0, 0),
    ("depths", "depths", 5.0, 0),
    // Starts at 06:30, while shadows are long.
    ("courtyard", "court", 5.0, 0),
    ("world", "world", 5.0, 0),
    // A generated forest area of the overworld at 06:30 (walking off an edge, the walk goes
    // straight back in).
    ("forest", "forest", 5.0, 0),
    // Moon shadows instead of sun shadows.
    ("courtyard", "c-moon", 5.0, TO_FULL_MOON),
    ("forest", "f-moon", 5.0, TO_FULL_MOON),
    // Every creature thinks: they see, path, and attack each other (and the player) all along.
    ("arena", "arena", 5.0, 0),
    // A generated 120×60 cave floor at full population (`DELF_BRANCH`).
    ("bench_delf", "delf", 16.0, 0),
];

/// The builtin content with the bench's cave branch and its scenario.
fn content() -> Result<Content, String> {
    let mut files = ContentFiles::builtin();
    let dungeons = format!(
        "{}{DELF_BRANCH}",
        include_str!("../../assets/data/dungeons.toml")
    );
    files.insert("data/dungeons.toml", &dungeons);
    files.insert("scenarios/bench_delf.toml", DELF_SCENARIO);
    Content::load(&files).map_err(|e| e.to_string())
}

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
    /// A rest from dusk to dawn after the walk.
    rest_ms: f64,
    rest_turns: u64,
}

fn measure(content: &Arc<Content>, scenario: &str, skip: u64) -> Result<Stats, String> {
    // The scenario's start; a walk that gets the player killed starts over from it, untimed.
    let fresh = || -> Result<World, String> {
        let mut world = World::new(content.clone(), scenario, None).map_err(|e| e.to_string())?;
        if skip > 0 {
            world.apply(&Action::SkipTime(Duration::minutes(skip)));
        }
        Ok(world)
    };
    let mut world = fresh()?;
    let level = world.perception().level_id().to_string();
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
        // Walked off a local area: back in, untimed, and on in another direction.
        if world.perception().level_id() != level {
            world.apply(&Action::Descend);
            heading = Dir::ALL[next(&mut rng) as usize % Dir::ALL.len()];
        }
        // Killed (the walk blunders into fights): the run starts over, untimed.
        if world.death().is_some() {
            world = fresh()?;
        }
    }
    if times.is_empty() {
        return Err(format!("no turns taken in scenario `{scenario}`"));
    }
    // Rest from dusk to dawn wherever the walk ended (a creature may cut it short; rain
    // starting wakes the player, who rests on until dawn).
    let to_dusk = world.time().next_at(DUSK).since(world.time());
    world.apply(&Action::SkipTime(to_dusk));
    let dawn = world.time().next_at(DAWN);
    let (start, turn) = (Instant::now(), world.turn());
    for _ in 0..8 {
        world.apply(&Action::Rest(Rest::Dawn));
        if world.time() >= dawn {
            break;
        }
    }
    let rest_ms = start.elapsed().as_secs_f64() * 1000.0;
    let rest_turns = world.turn() - turn;
    Ok(Stats {
        avg_ms: times.iter().sum::<f64>() / times.len() as f64,
        max_ms: times.iter().copied().fold(0.0, f64::max),
        turns: times.len(),
        rest_ms,
        rest_turns,
    })
}

pub fn run() -> Result<(), String> {
    if cfg!(debug_assertions) {
        eprintln!(
            "warning: debug build; run `cargo xtask bench-turns` in release for real numbers"
        );
    }
    let content = Arc::new(content()?);
    println!(
        "{:<6} {:>6} {:>10} {:>10} {:>10}  {:>11} {:>8}  verdict (rest budget {REST_BUDGET_MS} ms)",
        "level", "turns", "avg ms", "max ms", "budget ms", "rest turns", "rest ms"
    );
    let mut failed = false;
    for (scenario, label, budget, skip) in CASES {
        let s = measure(&content, scenario, skip)?;
        let ok = s.max_ms <= budget && s.rest_ms <= REST_BUDGET_MS;
        failed |= !ok;
        println!(
            "{label:<6} {:>6} {:>10.3} {:>10.3} {budget:>10.1}  {:>11} {:>8.1}  {}",
            s.turns,
            s.avg_ms,
            s.max_ms,
            s.rest_turns,
            s.rest_ms,
            if ok { "ok" } else { "OVER BUDGET" },
        );
    }
    // A rest beside a fight nobody perceives, until one side is dead: every fighter
    // seeing, aiming, and attacking each turn, in view range, so no turn passes unperceived.
    let (turns, ms) = fight_rest(&content)?;
    let ok = ms <= REST_BUDGET_MS;
    failed |= !ok;
    println!(
        "{:<6} {:>6} {:>10} {:>10} {:>10}  {turns:>11} {ms:>8.1}  {}  (arena camp, fight out of sight)",
        "fight",
        "-",
        "-",
        "-",
        "-",
        if ok { "ok" } else { "OVER BUDGET" }
    );
    if failed {
        Err("turn time over budget".into())
    } else {
        Ok(())
    }
}

/// A night's rest in the arena's camp, its door shut: the fight in the pit goes on unseen and
/// unheard. Returns the turns rested and the time it took in ms.
fn fight_rest(content: &Arc<Content>) -> Result<(u64, f64), String> {
    let mut world = World::new(content.clone(), "arena", None).map_err(|e| e.to_string())?;
    let walk = "move:W*5 move:NW*4 move:N open:W move:W*2 close:E";
    for action in warlock_core::action::parse_actions(walk)? {
        world.apply(&action);
    }
    let to_dusk = world.time().next_at(DUSK).since(world.time());
    world.apply(&Action::SkipTime(to_dusk));
    let (start, turn) = (Instant::now(), world.turn());
    world.apply(&Action::Rest(Rest::Dawn));
    Ok((world.turn() - turn, start.elapsed().as_secs_f64() * 1000.0))
}

/// Frame-time measurement: runs the release game offscreen with `--capture-after-frames` N and
/// M, and divides the wall-clock difference by M - N, which cancels startup and capture cost.
/// `WARLOCK_UNPACED=1` removes the game's 60 Hz capture-mode pacing, so this measures CPU + GPU
/// throughput per frame (offscreen render, no presentation).
pub fn frames(window: &str) -> Result<(), String> {
    use crate::{target_dir, workspace_root};
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
    let target = target_dir()?;
    let exe = target.join("release/warlock");
    let out = target.join("xtask/bench-frame.png");
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
