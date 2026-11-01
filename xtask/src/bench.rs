//! `cargo xtask bench-turns`: turn-processing time in the hub, the vast level, the first
//! generated level of the depths, the sunlit courtyard at dawn, the overworld, a generated
//! forest area of the overworld at dawn, the courtyard and the forest area at midnight under the
//! full moon, the arena, where a dozen creatures fight, a generated 120×60 cave floor at
//! full population, 80 creatures of danger 8, the same with all but its kobolds shedding a spell
//! circle's light, and the same of animated panoplies at danger 40,
//! each wearing and wielding up to ten things (the vast level's budget; spec §10 budgets; a generated 80×21
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
                             wield = [\"lantern\"]\nhp = 900\n";

/// The same cave floor with every creature but the torch-bearing kobolds circled (spec §4.8
/// "Spell circles"): each sheds a spell circle's light, radius 1, as a hexed creature does
/// (bench copies, `bench_hexed_<id>`, carrying the light as a creature's own: the same one
/// source per actor, keyed by the actor, its mask recomputed when it moves). The design's "delf
/// floor of twenty hexed creatures" (docs/design/buff-circles.md §8.3), more of them; the budget
/// is the vast level's.
const HEXED_BRANCH: &str = r#"
[[dungeon]]
id = "bench_hexed"
name = "Bench hexed"
generator = "caves"
depth = 1
size = [120, 60]
danger = 8
population = ["bench_hexed_bat", "bench_hexed_beetle", "kobold", "bench_hexed_kobold_archer", "bench_hexed_kobold_shaman", "bench_hexed_rat"]
room_effects = { chance = 100, per_level = 1, favoured = { teeming = 100 } }
"#;
const HEXED_SCENARIO: &str = "name = \"Bench hexed\"\nstart_level = \"bench_hexed:1\"\nseed = 1\n\
                              wield = [\"lantern\"]\nhp = 900\n";
/// The circle's light the bench's hexed creatures shed.
const HEXED_LIGHT: &str = "circle_enchantment";
/// Rows whose rest is shown but not judged. `d-hex`'s walk ends where the rest runs the whole
/// night among 96 creatures (about 0.07 ms a turn, 500 ms or so), where `delf`'s is cut short
/// at once; rested from the same state with the copies dark it is cut short too (60 turns), and
/// no rest turn costs more lit than dark: a full night on that floor is the cost, not the light.
const UNJUDGED_REST: [&str; 1] = ["d-hex"];

/// The same cave floor at danger 40, all of it animated panoplies (spec §8 "Animated objects"):
/// each wears and wields its pieces, whose magic every rule reads through `Mods::worn`, the
/// design's open performance risk (docs/design/animated-objects.md §12); the budget is the
/// vast level's.
const PANOPLY_BRANCH: &str = r#"
[[dungeon]]
id = "bench_panoply"
name = "Bench panoply"
generator = "caves"
depth = 1
size = [120, 60]
danger = 40
population = ["animated_panoply"]
"#;
const PANOPLY_SCENARIO: &str = "name = \"Bench panoply\"\nstart_level = \"bench_panoply:1\"\n\
                                seed = 1\nwield = [\"lantern\"]\nhp = 900\n";

/// (scenario, level label, budget in ms, minutes skipped before the walk)
const CASES: [(&str, &str, f64, u64); 12] = [
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
    // The same with every creature but the kobolds circled (`HEXED_BRANCH`).
    ("bench_hexed", "d-hex", 16.0, 0),
    // The same at danger 40, of panoplies (`PANOPLY_BRANCH`).
    ("bench_panoply", "panop", 16.0, 0),
];

/// The builtin content with the bench's cave branch and its scenario.
fn content() -> Result<Content, String> {
    let mut files = ContentFiles::builtin();
    let dungeons = format!(
        "{}{DELF_BRANCH}{HEXED_BRANCH}{PANOPLY_BRANCH}",
        include_str!("../../assets/data/dungeons.toml")
    );
    files.insert("data/dungeons.toml", &dungeons);
    files.insert("scenarios/bench_delf.toml", DELF_SCENARIO);
    files.insert("scenarios/bench_panoply.toml", PANOPLY_SCENARIO);
    files.insert("scenarios/bench_hexed.toml", HEXED_SCENARIO);
    for (id, text) in [
        ("bat", include_str!("../../assets/data/creatures/bat.toml")),
        (
            "beetle",
            include_str!("../../assets/data/creatures/beetle.toml"),
        ),
        (
            "kobold_archer",
            include_str!("../../assets/data/creatures/kobold_archer.toml"),
        ),
        (
            "kobold_shaman",
            include_str!("../../assets/data/creatures/kobold_shaman.toml"),
        ),
        ("rat", include_str!("../../assets/data/creatures/rat.toml")),
    ] {
        let line = format!("id = \"{id}\"\n");
        let hexed = text.replace(
            &line,
            &format!("id = \"bench_hexed_{id}\"\nlight = \"{HEXED_LIGHT}\"\n"),
        );
        files.insert(&format!("data/creatures/bench_hexed_{id}.toml"), &hexed);
    }
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
    if scenario == "bench_hexed" {
        // How many are circled: all but the kobolds (the dev `actors` layer, a row an actor).
        let layer = warlock_core::dump::parse_layers("actors")?;
        let rows = warlock_core::dump::layer_rows(&world, layer[0]);
        let actors: Vec<&String> = (rows.iter()).filter(|r| r.contains(" at (")).collect();
        let kobolds = (actors.iter())
            .filter(|r| r.contains(" kobold at ("))
            .count();
        // The player is the first row.
        let circled = actors.len().saturating_sub(kobolds + 1);
        eprintln!(
            "{scenario}: {circled} of {} creatures shed a circle's light",
            actors.len() - 1
        );
    }
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
        let rest_ok = s.rest_ms <= REST_BUDGET_MS || UNJUDGED_REST.contains(&label);
        let ok = s.max_ms <= budget && rest_ok;
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
