//! `warlock --move-data native | portable [DIR]` (spec §7): moves the game's files between a
//! portable folder and the platform's standard folders, then exits.
//!
//! Safe by construction: every file is copied beside its destination, compared byte for byte,
//! renamed into place, and only then removed from the source, so it is never lost; a file
//! already at the destination is renamed to `NAME.bak-<time>` first, never overwritten; on an
//! error the run stops, undoes the file it was on, and leaves every other file complete in one
//! place or the other. `--dry-run` prints the plan and touches nothing. The move refuses while
//! another Warlock is using either set of files (best effort: a lock file in each `logs`
//! folder, held by every running game).

use crate::paths::{Locations, PORTABLE_DIR, Platform};
use std::fs::{self, File, TryLockError};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The lock file each running game holds (shared) in its logs folder.
const IN_USE: &str = ".in-use";

/// Which way to move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// Into the platform's standard folders.
    Native,
    /// Into a portable folder; `None` means `warlock-data` next to the game.
    Portable(Option<PathBuf>),
}

impl Target {
    /// Reads the words after `--move-data`.
    pub fn parse(words: &[String]) -> Result<Self, String> {
        match words {
            [w] if w == "native" => Ok(Self::Native),
            [w] if w == "portable" => Ok(Self::Portable(None)),
            [w, dir] if w == "portable" => Ok(Self::Portable(Some(PathBuf::from(dir)))),
            _ => Err("usage: --move-data native | --move-data portable [DIR]".into()),
        }
    }
}

/// One folder's contents to move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pair {
    pub from: PathBuf,
    pub to: PathBuf,
}

/// What a move does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    pub pairs: Vec<Pair>,
    /// Folders to rename out of the way afterwards (the emptied portable folder).
    pub retire: Option<PathBuf>,
    /// Said at the end.
    pub hint: Option<String>,
}

/// Works out what to move for `target`. Explicit `--config-dir` / `--save-dir` do not take part:
/// the move is between the portable folder and the standard folders.
pub fn route(
    target: &Target,
    locations: &Locations,
    platform: &Platform,
    game_folder: Option<&Path>,
) -> Result<Route, String> {
    let pairs = |portable: &Path, to_portable: bool| {
        let sides = [
            ("config", &platform.config),
            ("data", &platform.data),
            ("logs", &platform.logs),
        ];
        sides
            .into_iter()
            .map(|(sub, native)| {
                let (a, b) = (native.clone(), portable.join(sub));
                if to_portable {
                    Pair { from: a, to: b }
                } else {
                    Pair { from: b, to: a }
                }
            })
            .collect::<Vec<_>>()
    };
    match target {
        Target::Native => {
            let root = locations.portable_root.as_deref().ok_or(
                "Warlock is not running portable here, so there is nothing to move. \
                 (It is portable when a warlock-data folder is next to the game, or with --portable DIR.)",
            )?;
            Ok(Route {
                pairs: pairs(root, false),
                retire: Some(root.to_path_buf()),
                hint: None,
            })
        }
        Target::Portable(dir) => {
            if let Some(root) = &locations.portable_root {
                return Err(format!(
                    "Warlock is already portable ({}). Move to the standard folders first with \
                     --move-data native.",
                    root.display()
                ));
            }
            let root = match (dir, game_folder) {
                (Some(dir), _) => dir.clone(),
                (None, Some(game)) => game.join(PORTABLE_DIR),
                (None, None) => {
                    return Err("Cannot tell where the game is: name a folder, \
                         --move-data portable DIR."
                        .into());
                }
            };
            let hint = dir.as_ref().map(|d| {
                format!(
                    "Start the game with --portable {} to use these files.",
                    d.display()
                )
            });
            Ok(Route {
                pairs: pairs(&root, true),
                retire: None,
                hint,
            })
        }
    }
}

/// One file to move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Op {
    pub from: PathBuf,
    pub to: PathBuf,
}

/// Every file under the pairs' source folders, in a fixed order.
pub fn plan(pairs: &[Pair]) -> io::Result<Vec<Op>> {
    fn walk(dir: &Path, found: &mut Vec<PathBuf>) -> io::Result<()> {
        let mut entries = fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let path = entry.path();
            if entry.file_type()?.is_dir() {
                walk(&path, found)?;
            } else if entry.file_name() != IN_USE {
                found.push(path);
            }
        }
        Ok(())
    }
    let mut ops = Vec::new();
    for pair in pairs {
        if !pair.from.is_dir() {
            continue;
        }
        let mut files = Vec::new();
        walk(&pair.from, &mut files)?;
        for from in files {
            let rel = from.strip_prefix(&pair.from).expect("under the folder");
            ops.push(Op {
                to: pair.to.join(rel),
                from,
            });
        }
    }
    Ok(ops)
}

/// What a move did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub moved: Vec<Op>,
    /// Existing destination files renamed out of the way: (original place, backup).
    pub backups: Vec<(PathBuf, PathBuf)>,
    /// Why it stopped early.
    pub error: Option<String>,
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

/// A name for `path` plus `suffix` that is not taken.
fn free_name(path: &Path, suffix: &str) -> PathBuf {
    let mut found = with_suffix(path, suffix);
    let mut n = 1;
    while found.exists() {
        found = with_suffix(path, &format!("{suffix}-{n}"));
        n += 1;
    }
    found
}

/// Moves one file: copy beside the destination, compare, rename into place, remove the source.
/// Undoes itself on failure.
fn move_file(op: &Op, stamp: &str, report: &mut Report) -> io::Result<()> {
    if let Some(parent) = op.to.parent() {
        fs::create_dir_all(parent)?;
    }
    let backup = if op.to.exists() {
        let backup = free_name(&op.to, &format!(".bak-{stamp}"));
        fs::rename(&op.to, &backup)?;
        Some(backup)
    } else {
        None
    };
    let part = with_suffix(&op.to, ".part");
    let copied = (|| {
        fs::copy(&op.from, &part)?;
        if fs::read(&op.from)? != fs::read(&part)? {
            return Err(io::Error::other("the copy differs from the original"));
        }
        fs::rename(&part, &op.to)
    })();
    match copied {
        Ok(()) => {
            // The copy is in place and checked; failing to remove the original leaves a
            // duplicate, which is harmless, so say so but carry on as moved.
            if let Err(e) = fs::remove_file(&op.from) {
                report.backups.push((
                    op.from.clone(),
                    PathBuf::from(format!("(not removed: {e})")),
                ));
            }
            if let Some(backup) = backup {
                report.backups.push((op.to.clone(), backup));
            }
            Ok(())
        }
        Err(e) => {
            let _ = fs::remove_file(&part);
            if let Some(backup) = backup {
                let _ = fs::rename(backup, &op.to);
            }
            Err(e)
        }
    }
}

/// Runs the plan. Stops at the first error, recorded in the report.
pub fn execute(ops: &[Op], stamp: &str) -> Report {
    let mut report = Report::default();
    for op in ops {
        match move_file(op, stamp, &mut report) {
            Ok(()) => report.moved.push(op.clone()),
            Err(e) => {
                report.error = Some(format!(
                    "Stopped at {} -> {}: {e}. Nothing was lost: files already moved are \
                     complete at their new place, the rest are where they were.",
                    op.from.display(),
                    op.to.display()
                ));
                break;
            }
        }
    }
    report
}

/// Removes the empty folders left under each source (best effort).
fn tidy(pairs: &[Pair]) {
    fn prune(dir: &Path) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.file_type().is_ok_and(|t| t.is_dir()) {
                    prune(&entry.path());
                }
            }
        }
        let _ = fs::remove_dir(dir);
    }
    for pair in pairs {
        prune(&pair.from);
    }
}

/// Whether a running game holds the files that `logs` belongs to.
fn busy(logs: &Path) -> bool {
    let Ok(file) = File::options().write(true).open(logs.join(IN_USE)) else {
        return false;
    };
    matches!(file.try_lock(), Err(TryLockError::WouldBlock))
}

static HELD: OnceLock<File> = OnceLock::new();

/// Marks the files around `logs` as in use for as long as this process lives, so a move
/// refuses to run under a game. Failure is ignored: the check is best effort.
pub fn mark_in_use(logs: &Path) {
    if let Ok(file) = File::create(logs.join(IN_USE))
        && file.try_lock_shared().is_ok()
    {
        let _ = HELD.set(file);
    }
}

/// Does what `--move-data` asks and returns the text to print, or the reason it did not.
pub fn run(
    target: &Target,
    dry_run: bool,
    locations: &Locations,
    platform: &Platform,
    game_folder: Option<&Path>,
    stamp: &str,
) -> Result<String, String> {
    let route = route(target, locations, platform, game_folder)?;
    for pair in &route.pairs {
        for dir in [&pair.from, &pair.to] {
            if (dir.ends_with("logs")) && busy(dir) {
                return Err(format!(
                    "Warlock is running and using {}. Close it and try again.",
                    dir.parent().unwrap_or(dir).display()
                ));
            }
        }
    }
    let ops = plan(&route.pairs).map_err(|e| format!("Cannot list the files to move: {e}."))?;
    let mut out = String::new();
    let say = |out: &mut String, line: String| {
        out.push_str(&line);
        out.push('\n');
    };
    if ops.is_empty() {
        say(&mut out, "There are no files to move.".into());
    }
    if dry_run {
        say(&mut out, "Dry run: nothing is touched. The plan:".into());
        for op in &ops {
            let note = if op.to.exists() {
                format!(
                    "  (the file there is kept as {})",
                    free_name(&op.to, &format!(".bak-{stamp}")).display()
                )
            } else {
                String::new()
            };
            say(
                &mut out,
                format!("  {} -> {}{note}", op.from.display(), op.to.display()),
            );
        }
        if let Some(root) = &route.retire {
            say(
                &mut out,
                format!(
                    "  then {} is renamed to {}",
                    root.display(),
                    retired(root, stamp).display()
                ),
            );
        }
        return Ok(out);
    }
    let report = execute(&ops, stamp);
    for op in &report.moved {
        say(
            &mut out,
            format!("moved {} -> {}", op.from.display(), op.to.display()),
        );
    }
    for (kept, backup) in &report.backups {
        say(
            &mut out,
            format!("kept {} as {}", kept.display(), backup.display()),
        );
    }
    if let Some(error) = report.error {
        return Err(format!("{out}{error}"));
    }
    tidy(&route.pairs);
    if let Some(root) = &route.retire {
        let new = retired(root, stamp);
        match fs::rename(root, &new) {
            Ok(()) => say(
                &mut out,
                format!(
                    "renamed {} to {}; it no longer makes the game portable",
                    root.display(),
                    new.display()
                ),
            ),
            Err(e) => {
                return Err(format!(
                    "{out}Moved the files, but could not rename {}: {e}. Rename or delete it \
                     yourself, or the game stays portable.",
                    root.display()
                ));
            }
        }
    }
    say(&mut out, format!("Moved {} file(s).", report.moved.len()));
    if let Some(hint) = route.hint {
        say(&mut out, hint);
    }
    Ok(out)
}

/// Where an emptied portable folder goes: `warlock-data.moved-<date>` beside it.
fn retired(root: &Path, stamp: &str) -> PathBuf {
    let date = stamp.split('-').next().unwrap_or(stamp);
    free_name(root, &format!(".moved-{date}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::{Inputs, resolve};
    use std::collections::BTreeMap;

    struct Sandbox(PathBuf);

    impl Sandbox {
        fn new(name: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("warlock-move-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn platform(&self) -> Platform {
            Platform {
                config: self.0.join("native/config"),
                data: self.0.join("native/data"),
                logs: self.0.join("native/data-logs/logs"),
            }
        }

        fn game(&self) -> PathBuf {
            self.0.join("game")
        }

        fn locations(&self) -> Locations {
            let inputs = Inputs {
                game_folder: Some(self.game()),
                platform: Some(self.platform()),
                ..Inputs::default()
            };
            resolve(&inputs, &|p| p.is_dir())
        }

        fn write(&self, rel: &str, text: &str) {
            let path = self.0.join(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }

        /// Every file under `rel`, by path relative to it, with its text.
        fn files(&self, rel: &str) -> BTreeMap<String, String> {
            fn walk(base: &Path, dir: &Path, out: &mut BTreeMap<String, String>) {
                let Ok(entries) = fs::read_dir(dir) else {
                    return;
                };
                for e in entries.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        walk(base, &p, out);
                    } else {
                        let key = p.strip_prefix(base).unwrap().display().to_string();
                        out.insert(key, fs::read_to_string(&p).unwrap());
                    }
                }
            }
            let base = self.0.join(rel);
            let mut out = BTreeMap::new();
            walk(&base, &base, &mut out);
            out
        }
    }

    impl Drop for Sandbox {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn seed(sb: &Sandbox) {
        sb.write("native/config/settings.toml", "font_px = 20\n");
        sb.write("native/data/run.sav", "SAVE");
        sb.write("native/data/snapshots/quick.sav", "SNAP");
        sb.write("native/data-logs/logs/warlock-1.log", "LOG");
    }

    fn go(sb: &Sandbox, target: &Target, dry: bool) -> Result<String, String> {
        run(
            target,
            dry,
            &sb.locations(),
            &sb.platform(),
            Some(&sb.game()),
            "20261019-101500",
        )
    }

    #[test]
    fn parses_the_words() {
        let words = |s: &[&str]| s.iter().map(|w| w.to_string()).collect::<Vec<_>>();
        assert_eq!(Target::parse(&words(&["native"])), Ok(Target::Native));
        assert_eq!(
            Target::parse(&words(&["portable"])),
            Ok(Target::Portable(None))
        );
        assert_eq!(
            Target::parse(&words(&["portable", "/x"])),
            Ok(Target::Portable(Some("/x".into())))
        );
        assert!(Target::parse(&words(&["native", "/x"])).is_err());
        assert!(Target::parse(&words(&["sideways"])).is_err());
    }

    #[test]
    fn native_to_portable_to_native_round_trips() {
        let sb = Sandbox::new("round");
        seed(&sb);
        let before = sb.files("native");
        let out = go(&sb, &Target::Portable(None), false).unwrap();
        assert!(out.contains("Moved 4 file(s)."), "{out}");
        assert_eq!(sb.files("game/warlock-data").len(), 4);
        assert_eq!(
            sb.files("game/warlock-data")["data/snapshots/quick.sav"],
            "SNAP"
        );
        assert!(
            sb.files("native").is_empty(),
            "the standard folders are left empty"
        );

        // Now the game is portable: moving back renames the emptied folder.
        let out = go(&sb, &Target::Native, false).unwrap();
        assert!(out.contains("warlock-data.moved-20261019"), "{out}");
        assert_eq!(sb.files("native"), before);
        assert!(!sb.game().join("warlock-data").exists());
        assert!(sb.game().join("warlock-data.moved-20261019").is_dir());
        // The renamed folder no longer makes the game portable.
        assert_eq!(sb.locations().portable_root, None);
    }

    #[test]
    fn a_file_at_the_destination_is_backed_up_never_overwritten() {
        let sb = Sandbox::new("conflict");
        seed(&sb);
        sb.write("game/warlock-data/data/run.sav", "OLD PORTABLE SAVE");
        // Not "already portable" for the route: name the folder explicitly.
        let target = Target::Portable(Some(sb.game().join("warlock-data")));
        let inputs = Inputs {
            platform: Some(sb.platform()),
            ..Inputs::default()
        };
        let locations = resolve(&inputs, &|_| false);
        let out = run(
            &target,
            false,
            &locations,
            &sb.platform(),
            None,
            "20261019-101500",
        )
        .unwrap();
        assert!(out.contains("run.sav.bak-20261019-101500"), "{out}");
        let files = sb.files("game/warlock-data");
        assert_eq!(files["data/run.sav"], "SAVE");
        assert_eq!(
            files["data/run.sav.bak-20261019-101500"],
            "OLD PORTABLE SAVE"
        );
    }

    #[test]
    fn a_dry_run_touches_nothing() {
        let sb = Sandbox::new("dry");
        seed(&sb);
        let before = sb.files("");
        let out = go(&sb, &Target::Portable(None), true).unwrap();
        assert!(out.starts_with("Dry run"), "{out}");
        assert!(out.contains("run.sav"));
        assert_eq!(sb.files(""), before);
        assert!(!sb.game().exists());
    }

    #[test]
    fn an_error_midway_loses_nothing() {
        let sb = Sandbox::new("error");
        seed(&sb);
        // `snapshots` is a file at the destination, so the snapshot cannot go there.
        sb.write("game/warlock-data/data/snapshots", "in the way");
        let target = Target::Portable(Some(sb.game().join("warlock-data")));
        let inputs = Inputs {
            platform: Some(sb.platform()),
            ..Inputs::default()
        };
        let locations = resolve(&inputs, &|_| false);
        let err = run(
            &target,
            false,
            &locations,
            &sb.platform(),
            None,
            "20261019-101500",
        )
        .unwrap_err();
        assert!(err.contains("Nothing was lost"), "{err}");
        // Every file is complete in one place or the other, and no half copies remain.
        let moved = sb.files("game/warlock-data");
        let kept = sb.files("native");
        assert_eq!(kept["data/snapshots/quick.sav"], "SNAP");
        assert_eq!(moved["config/settings.toml"], "font_px = 20\n");
        assert_eq!(moved["data/snapshots"], "in the way");
        assert!(moved.keys().all(|k| !k.ends_with(".part")));
        let total = moved.len() + kept.len();
        assert_eq!(total, 5, "{moved:?} {kept:?}");
    }

    #[test]
    fn refuses_when_not_portable_or_already_portable() {
        let sb = Sandbox::new("refuse");
        seed(&sb);
        assert!(
            go(&sb, &Target::Native, false)
                .unwrap_err()
                .contains("not running portable")
        );
        fs::create_dir_all(sb.game().join("warlock-data")).unwrap();
        assert!(
            go(&sb, &Target::Portable(None), false)
                .unwrap_err()
                .contains("already portable")
        );
    }

    #[test]
    fn refuses_while_a_game_holds_the_files() {
        let sb = Sandbox::new("busy");
        seed(&sb);
        let logs = sb.platform().logs;
        let running = File::create(logs.join(IN_USE)).unwrap();
        running.try_lock_shared().unwrap();
        let err = go(&sb, &Target::Portable(None), false).unwrap_err();
        assert!(err.contains("is running"), "{err}");
        assert_eq!(sb.files("native").len(), 5, "nothing moved");
        drop(running);
        assert!(go(&sb, &Target::Portable(None), false).is_ok());
        // The lock file itself is not data and stays where it was.
        assert!(logs.join(IN_USE).exists());
    }
}
