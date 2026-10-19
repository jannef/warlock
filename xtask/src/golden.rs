//! `cargo xtask golden [--update | --local]`: golden screenshots per showcase room (spec §10).
//!
//! Each entry of `tests/golden/manifest.toml` names a scenario plus either a text action sequence
//! (turned into a replay with `warlock-core`) or an existing replay file. The game renders it
//! headlessly (`--replay … --capture …`) and the result is compared with `tests/golden/<name>.png`.
//!
//! References are authoritative only when produced on Linux CI (Mesa lavapipe); other GPUs and
//! drivers rasterize slightly differently. On other platforms use `--local` to just capture.

use crate::{run_game, workspace_root};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use warlock_core::action::parse_actions;
use warlock_core::content::Content;
use warlock_core::replay::Replay;
use warlock_core::{Action, World};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    #[serde(default)]
    defaults: Defaults,
    #[serde(default, rename = "golden")]
    entries: Vec<Entry>,
}

/// Values used by entries that do not set their own.
#[derive(Deserialize)]
#[serde(deny_unknown_fields, default)]
struct Defaults {
    window: String,
    scale: f32,
    freeze_time: f32,
    /// Frames rendered before the capture (software Vulkan compiles pipelines slowly).
    capture_after_frames: u32,
    /// Largest allowed difference per color channel (0-255) before a pixel counts as different.
    tolerance: u8,
    /// Largest allowed fraction of differing pixels.
    max_diff_ratio: f64,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            window: "960x540".into(),
            scale: 1.0,
            freeze_time: 1.0,
            capture_after_frames: 60,
            tolerance: 2,
            max_diff_ratio: 0.0005,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    name: String,
    scenario: Option<String>,
    seed: Option<u64>,
    /// Text actions (`move:N*3 open:E wait`; `*N` repeats an action); created into a replay.
    actions: Option<String>,
    /// An existing replay file, relative to `tests/golden/`.
    replay: Option<String>,
    window: Option<String>,
    scale: Option<f32>,
    freeze_time: Option<f32>,
    capture_after_frames: Option<u32>,
    /// Open this menu screen over the game (`--screen`, e.g. `pause`).
    screen: Option<String>,
    /// More game flags, e.g. `["--fire"]` (start aiming the readied missile weapon).
    #[serde(default)]
    args: Vec<String>,
    tolerance: Option<u8>,
    max_diff_ratio: Option<f64>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Compare,
    Update,
    Local,
}

pub fn parse_mode(args: &[String]) -> Result<Mode, String> {
    match args {
        [] => Ok(Mode::Compare),
        [a] if a == "--update" => Ok(Mode::Update),
        [a] if a == "--local" => Ok(Mode::Local),
        _ => Err("usage: cargo xtask golden [--update | --local]".into()),
    }
}

/// Writes the replay for an entry and returns its path.
fn make_replay(
    entry: &Entry,
    content: &Arc<Content>,
    golden_dir: &Path,
    out_dir: &Path,
) -> Result<PathBuf, String> {
    if let Some(replay) = &entry.replay {
        return Ok(golden_dir.join(replay));
    }
    let scenario = entry.scenario.as_deref().unwrap_or("showcase");
    let mut world = World::new(content.clone(), scenario, entry.seed).map_err(|e| e.to_string())?;
    let mut replay = Replay::new(&world);
    let text = entry.actions.as_deref().unwrap_or("");
    for action in parse_actions(text)? {
        let outcome = world.apply(&action);
        // Switching vision, setting the weather, and the dev status effect and infection are free
        // by design; anything else that takes no time and no turn was refused. (A turn can pass
        // in no time: a creature quick off the mark may kill the player at the very second they
        // acted.)
        let free = matches!(
            action,
            Action::SetVision(_) | Action::SetWeather(_) | Action::Afflict(_) | Action::Infect(_)
        );
        if outcome.elapsed.is_zero() && !outcome.took_turn && !free {
            return Err(format!(
                "{}: action `{action}` took no turn (blocked?); fix the action sequence",
                entry.name
            ));
        }
        replay.push(&action, &world);
    }
    let path = out_dir.join(format!("{}.replay.json", entry.name));
    std::fs::write(&path, replay.to_json())
        .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(path)
}

struct Comparison {
    differing: usize,
    total: usize,
    max_delta: u8,
}

fn compare(
    reference: &image::RgbaImage,
    actual: &image::RgbaImage,
    tolerance: u8,
) -> (Comparison, image::RgbaImage) {
    let (w, h) = reference.dimensions();
    let mut diff = image::RgbaImage::new(w, h);
    let mut cmp = Comparison {
        differing: 0,
        total: (w * h) as usize,
        max_delta: 0,
    };
    for (x, y, r) in reference.enumerate_pixels() {
        let a = actual.get_pixel(x, y);
        let delta = (0..4).map(|i| r[i].abs_diff(a[i])).max().unwrap_or(0);
        cmp.max_delta = cmp.max_delta.max(delta);
        if delta > tolerance {
            cmp.differing += 1;
            diff.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
        } else {
            // Dimmed reference as context.
            let dim = |c: u8| c / 3;
            diff.put_pixel(x, y, image::Rgba([dim(r[0]), dim(r[1]), dim(r[2]), 255]));
        }
    }
    (cmp, diff)
}

pub fn run(mode: Mode) -> Result<(), String> {
    let root = workspace_root();
    let golden_dir = root.join("tests/golden");
    let out_dir = root.join("target/xtask/golden");
    std::fs::create_dir_all(&out_dir).map_err(|e| format!("cannot create out dir: {e}"))?;
    let manifest_path = golden_dir.join("manifest.toml");
    let text = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("cannot read {}: {e}", manifest_path.display()))?;
    let manifest: Manifest =
        toml::from_str(&text).map_err(|e| format!("{}: {e}", manifest_path.display()))?;
    let d = &manifest.defaults;
    let content = Arc::new(Content::builtin().map_err(|e| e.to_string())?);

    if !cfg!(target_os = "linux") && mode == Mode::Compare {
        eprintln!(
            "note: references are produced on Linux CI (lavapipe); comparisons on this platform \
             are advisory only (use --local to capture without comparing)"
        );
    }

    let mut failures = Vec::new();
    for entry in &manifest.entries {
        let replay = make_replay(entry, &content, &golden_dir, &out_dir)?;
        let window = entry.window.as_deref().unwrap_or(&d.window);
        let scale = entry.scale.unwrap_or(d.scale).to_string();
        let freeze = entry.freeze_time.unwrap_or(d.freeze_time).to_string();
        let frames = entry
            .capture_after_frames
            .unwrap_or(d.capture_after_frames)
            .to_string();
        let tolerance = entry.tolerance.unwrap_or(d.tolerance);
        let max_ratio = entry.max_diff_ratio.unwrap_or(d.max_diff_ratio);

        let reference = golden_dir.join(format!("{}.png", entry.name));
        let actual_path = match mode {
            Mode::Update => reference.clone(),
            Mode::Local => out_dir.join(format!("{}.local.png", entry.name)),
            Mode::Compare => out_dir.join(format!("{}.actual.png", entry.name)),
        };
        let _ = std::fs::remove_file(&actual_path);
        let mut screen_args: Vec<&str> = match &entry.screen {
            Some(screen) => vec!["--screen", screen.as_str()],
            None => Vec::new(),
        };
        screen_args.extend(entry.args.iter().map(String::as_str));
        run_game(
            &[
                &screen_args[..],
                &[
                    "--window",
                    window,
                    "--scale",
                    &scale,
                    "--freeze-time",
                    &freeze,
                    "--capture-after-frames",
                    &frames,
                    "--replay",
                    replay.to_str().ok_or("non-UTF-8 path")?,
                    "--capture",
                    actual_path.to_str().ok_or("non-UTF-8 path")?,
                ],
            ]
            .concat(),
        )?;
        if !actual_path.exists() {
            return Err(format!("{}: game wrote no capture", entry.name));
        }
        match mode {
            Mode::Update => println!("updated  {}", entry.name),
            Mode::Local => println!("captured {} -> {}", entry.name, actual_path.display()),
            Mode::Compare => {
                if let Err(msg) = check(
                    entry,
                    &reference,
                    &actual_path,
                    &out_dir,
                    tolerance,
                    max_ratio,
                ) {
                    println!("FAIL     {msg}");
                    failures.push(msg);
                } else {
                    println!("ok       {}", entry.name);
                    let _ = std::fs::remove_file(&actual_path);
                }
            }
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} of {} golden screenshots differ (images in {}). If the change is intended, \
             regenerate references on Linux CI; see CLAUDE.md \"Golden screenshots\".\n  {}",
            failures.len(),
            manifest.entries.len(),
            out_dir.display(),
            failures.join("\n  ")
        ))
    }
}

fn check(
    entry: &Entry,
    reference: &Path,
    actual_path: &Path,
    out_dir: &Path,
    tolerance: u8,
    max_ratio: f64,
) -> Result<(), String> {
    let name = &entry.name;
    let reference_img = image::open(reference)
        .map_err(|e| {
            format!(
                "{name}: cannot open reference {} ({e}); generate it with `cargo xtask golden --update` on Linux CI",
                reference.display()
            )
        })?
        .to_rgba8();
    let actual = image::open(actual_path)
        .map_err(|e| format!("{name}: cannot open capture: {e}"))?
        .to_rgba8();
    if reference_img.dimensions() != actual.dimensions() {
        return Err(format!(
            "{name}: size {:?} differs from reference {:?}",
            actual.dimensions(),
            reference_img.dimensions()
        ));
    }
    let (cmp, diff) = compare(&reference_img, &actual, tolerance);
    let ratio = cmp.differing as f64 / cmp.total as f64;
    if ratio <= max_ratio {
        return Ok(());
    }
    let diff_path = out_dir.join(format!("{name}.diff.png"));
    diff.save(&diff_path)
        .map_err(|e| format!("{name}: cannot write diff: {e}"))?;
    Err(format!(
        "{name}: {} of {} pixels differ ({:.3}% > {:.3}% allowed, tolerance {tolerance}, max channel delta {}); see {} and {}",
        cmp.differing,
        cmp.total,
        ratio * 100.0,
        max_ratio * 100.0,
        cmp.max_delta,
        actual_path.display(),
        diff_path.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modes() {
        assert!(parse_mode(&[]).unwrap() == Mode::Compare);
        assert!(parse_mode(&["--update".into()]).unwrap() == Mode::Update);
        assert!(parse_mode(&["--bogus".into()]).is_err());
    }

    #[test]
    fn compare_counts_pixels_beyond_tolerance() {
        let a = image::RgbaImage::from_pixel(2, 1, image::Rgba([10, 10, 10, 255]));
        let mut b = a.clone();
        b.put_pixel(0, 0, image::Rgba([12, 10, 10, 255]));
        b.put_pixel(1, 0, image::Rgba([20, 10, 10, 255]));
        let (cmp, _) = compare(&a, &b, 2);
        assert_eq!(cmp.differing, 1);
        assert_eq!(cmp.max_delta, 10);
    }
}
