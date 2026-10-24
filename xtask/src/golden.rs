//! `cargo xtask golden [--update | --local]`: golden screenshots per showcase room (spec §10).
//!
//! The manifest is the directory `tests/golden/manifest/`: every `*.toml` directly in it, read in
//! file-name order (one file per showcase room or feature, so feature branches add entries without
//! conflicting). Exactly one file may set `[defaults]` (`defaults.toml`); each `[[golden]]` entry
//! names a scenario plus either a text action sequence (turned into a replay with `warlock-core`)
//! or an existing replay file, and its name is unique across the files. The game renders each
//! entry headlessly (`--replay … --capture …`) and the result is compared with
//! `tests/golden/<name>.png`.
//!
//! Every entry is tried: one whose walk is blocked (an action that takes no turn), that the game
//! fails to capture, that has no reference, or that differs is reported and the run goes on to
//! the next; the run fails at the end, listing them all. A comparison also fails on a reference
//! image that no entry names.
//!
//! References are authoritative only when produced on Linux CI (Mesa lavapipe); other GPUs and
//! drivers rasterize slightly differently. On other platforms use `--local` to just capture.

use crate::{run_game, target_dir, workspace_root};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use warlock_core::action::parse_actions;
use warlock_core::character::CharacterSpec;
use warlock_core::content::Content;
use warlock_core::replay::Replay;
use warlock_core::{Action, Event, World};

/// One manifest file: optional defaults and any number of entries.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestFile {
    defaults: Option<Defaults>,
    #[serde(default, rename = "golden")]
    entries: Vec<Entry>,
}

/// The manifest directory merged: the `[defaults]`, if a file set them, and the entries in file
/// order.
struct Manifest {
    defaults: Option<Defaults>,
    entries: Vec<Entry>,
}

/// Reads every `*.toml` directly under `dir` in file-name order and merges them.
///
/// Errors name the file: a file that does not parse, a second file setting `[defaults]`, or an
/// entry name used twice. Without any `[defaults]` the built-in defaults apply.
fn load_manifest(dir: &Path) -> Result<Manifest, String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("cannot read manifest directory {}: {e}", dir.display()))?;
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "toml"))
        .collect();
    paths.sort();
    if paths.is_empty() {
        return Err(format!("no manifest files (*.toml) in {}", dir.display()));
    }

    let mut defaults: Option<(Defaults, PathBuf)> = None;
    let mut manifest = Vec::new();
    let mut names: HashMap<String, PathBuf> = HashMap::new();
    for path in paths {
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let file: ManifestFile =
            toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        if let Some(found) = file.defaults {
            if let Some((_, first)) = &defaults {
                return Err(format!(
                    "[defaults] is set in both {} and {}; keep it in one file",
                    first.display(),
                    path.display()
                ));
            }
            defaults = Some((found, path.clone()));
        }
        for entry in file.entries {
            if let Some(first) = names.get(&entry.name) {
                return Err(format!(
                    "golden entry `{}` is defined in both {} and {}",
                    entry.name,
                    first.display(),
                    path.display()
                ));
            }
            names.insert(entry.name.clone(), path.clone());
            manifest.push(entry);
        }
    }
    Ok(Manifest {
        defaults: defaults.map(|(d, _)| d),
        entries: manifest,
    })
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
    /// The character the run starts as (`dwarf,female,warrior`), instead of the scenario's.
    character: Option<String>,
    /// Start with the character's starting kit, as a run whose character the player chose does
    /// (spec §8 "Character creation"); default false: the scenario's pack alone.
    #[serde(default)]
    kit: bool,
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
    let character = (entry.character.as_deref())
        .map(str::parse::<CharacterSpec>)
        .transpose()
        .map_err(|e| format!("{}: {e}", entry.name))?;
    let (seed, chosen) = (entry.seed, character.as_ref());
    let mut world = World::start_run(content.clone(), scenario, seed, chosen, None, entry.kit)
        .map_err(|e| format!("{}: {e}", entry.name))?;
    let mut replay = Replay::new(&world);
    let text = entry.actions.as_deref().unwrap_or("");
    let actions = parse_actions(text).map_err(|e| format!("{}: {e}", entry.name))?;
    let count = actions.len();
    for (step, action) in actions.into_iter().enumerate() {
        let outcome = world.apply(&action);
        // Switching vision, setting the weather, and the dev status effect, infection, frost,
        // and thaw are free by design, and so are a spell effect the player sees and a question
        // answered (asking takes no time); anything else that takes no time and no turn was
        // refused. (A turn can pass in no time: a creature quick off the mark may kill the player
        // at the very second they acted.)
        let free = matches!(
            action,
            Action::SetVision(_)
                | Action::SetWeather(_)
                | Action::Afflict(_)
                | Action::Infect(_)
                | Action::Freeze { .. }
                | Action::Thaw { .. }
        ) || matches!(action, Action::Effect { .. }) && !outcome.events.is_empty()
            || matches!(action, Action::Ask { .. })
                && outcome.events.iter().any(|e| matches!(e, Event::Talked(_)));
        if outcome.elapsed.is_zero() && !outcome.took_turn && !free {
            return Err(format!(
                "{}: action `{action}` (step {} of {count}) took no turn (blocked?); fix the \
                 action sequence",
                entry.name,
                step + 1
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
    let out_dir = target_dir()?.join("xtask/golden");
    std::fs::create_dir_all(&out_dir).map_err(|e| format!("cannot create out dir: {e}"))?;
    let manifest = load_manifest(&golden_dir.join("manifest"))?;
    let d = &manifest.defaults.unwrap_or_default();
    let content = Arc::new(Content::builtin().map_err(|e| e.to_string())?);

    if !cfg!(target_os = "linux") && mode == Mode::Compare {
        eprintln!(
            "note: references are produced on Linux CI (lavapipe); comparisons on this platform \
             are advisory only (use --local to capture without comparing)"
        );
    }

    // Every entry is tried (see the module docs), so one broken entry never hides the others.
    let mut failures = Vec::new();
    for entry in &manifest.entries {
        let reference = golden_dir.join(format!("{}.png", entry.name));
        let actual_path = match mode {
            Mode::Update => reference.clone(),
            Mode::Local => out_dir.join(format!("{}.local.png", entry.name)),
            Mode::Compare => out_dir.join(format!("{}.actual.png", entry.name)),
        };
        let mut result = capture(entry, d, &content, &golden_dir, &out_dir, &actual_path);
        if result.is_ok() && mode == Mode::Compare {
            let tolerance = entry.tolerance.unwrap_or(d.tolerance);
            let max_ratio = entry.max_diff_ratio.unwrap_or(d.max_diff_ratio);
            result = check(
                entry,
                &reference,
                &actual_path,
                &out_dir,
                tolerance,
                max_ratio,
            );
        }
        match (result, mode) {
            (Ok(()), Mode::Update) => println!("updated  {}", entry.name),
            (Ok(()), Mode::Local) => {
                println!("captured {} -> {}", entry.name, actual_path.display());
            }
            (Ok(()), Mode::Compare) => {
                println!("ok       {}", entry.name);
                let _ = std::fs::remove_file(&actual_path);
            }
            (Err(msg), _) => {
                println!("FAIL     {msg}");
                failures.push(msg);
            }
        }
    }
    if mode == Mode::Compare {
        for name in orphans(&golden_dir, &manifest.entries)? {
            let msg = format!("{name}: tests/golden/{name}.png has no manifest entry; remove it");
            println!("FAIL     {msg}");
            failures.push(msg);
        }
    }
    if failures.is_empty() {
        return Ok(());
    }
    let what = match mode {
        Mode::Compare => format!(
            "differ or could not be captured (images in {}). If a change is intended, \
             regenerate references on Linux CI; see CLAUDE.md \"Golden screenshots\"",
            out_dir.display()
        ),
        Mode::Update | Mode::Local => "could not be captured".to_string(),
    };
    Err(format!(
        "{} of {} golden screenshots {what}.\n  {}",
        failures.len(),
        manifest.entries.len(),
        failures.join("\n  ")
    ))
}

/// The reference images in `golden_dir` that no entry names (left behind by a renamed or removed
/// entry), sorted.
fn orphans(golden_dir: &Path, entries: &[Entry]) -> Result<Vec<String>, String> {
    let dir = std::fs::read_dir(golden_dir)
        .map_err(|e| format!("cannot read {}: {e}", golden_dir.display()))?;
    let mut orphans: Vec<String> = dir
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "png"))
        .filter_map(|path| Some(path.file_stem()?.to_str()?.to_string()))
        .filter(|name| !entries.iter().any(|entry| &entry.name == name))
        .collect();
    orphans.sort();
    Ok(orphans)
}

/// Builds an entry's replay and renders it to `actual_path`.
fn capture(
    entry: &Entry,
    d: &Defaults,
    content: &Arc<Content>,
    golden_dir: &Path,
    out_dir: &Path,
    actual_path: &Path,
) -> Result<(), String> {
    let replay = make_replay(entry, content, golden_dir, out_dir)?;
    let window = entry.window.as_deref().unwrap_or(&d.window);
    let scale = entry.scale.unwrap_or(d.scale).to_string();
    let freeze = entry.freeze_time.unwrap_or(d.freeze_time).to_string();
    let frames = entry
        .capture_after_frames
        .unwrap_or(d.capture_after_frames)
        .to_string();
    let _ = std::fs::remove_file(actual_path);
    let mut args: Vec<&str> = match &entry.screen {
        Some(screen) => vec!["--screen", screen.as_str()],
        None => Vec::new(),
    };
    args.extend(entry.args.iter().map(String::as_str));
    args.extend([
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
    ]);
    run_game(&args).map_err(|e| format!("{}: {e}", entry.name))?;
    if actual_path.exists() {
        Ok(())
    } else {
        Err(format!("{}: game wrote no capture", entry.name))
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

    /// The checked-in manifest directory loads: every file parses, the names are unique, and
    /// `defaults.toml` sets the defaults.
    #[test]
    fn manifest_directory_loads() {
        let manifest = load_manifest(&workspace_root().join("tests/golden/manifest")).unwrap();
        assert!(!manifest.entries.is_empty());
        let mut names: Vec<&str> = manifest.entries.iter().map(|e| e.name.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), manifest.entries.len());
        assert!(manifest.defaults.is_some(), "defaults.toml sets [defaults]");
    }

    /// Every checked-in reference image belongs to an entry (none left behind by a rename).
    #[test]
    fn references_have_entries() {
        let golden = workspace_root().join("tests/golden");
        let manifest = load_manifest(&golden.join("manifest")).unwrap();
        assert_eq!(
            orphans(&golden, &manifest.entries).unwrap(),
            Vec::<String>::new()
        );
    }

    #[test]
    fn manifest_rejects_duplicates() {
        let dir = std::env::temp_dir().join(format!("wp185-golden-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let entry = "[[golden]]\nname = \"same\"\nactions = \"wait\"\n";
        std::fs::write(dir.join("a.toml"), format!("[defaults]\n{entry}")).unwrap();
        std::fs::write(dir.join("b.toml"), entry).unwrap();
        let err = load_manifest(&dir)
            .err()
            .expect("duplicate names are refused");
        assert!(err.contains("`same`") && err.contains("a.toml") && err.contains("b.toml"));
        std::fs::write(dir.join("b.toml"), "[defaults]\n").unwrap();
        let err = load_manifest(&dir)
            .err()
            .expect("two [defaults] are refused");
        assert!(err.contains("[defaults]") && err.contains("a.toml") && err.contains("b.toml"));
        std::fs::remove_file(dir.join("a.toml")).unwrap();
        assert!(load_manifest(&dir).unwrap().entries.is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
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
