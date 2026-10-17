//! Saving and resuming: the run save, autosave, dev snapshots, and quit handling (spec §7).
//!
//! The core turns a world into bytes (`warlock_core::save`); this module owns where they live.
//! Saving never panics or aborts the game: failures become a message in the log. On wasm there is
//! no save directory yet (web storage comes with the web build), so every operation is a no-op.

use crate::cli::Args;
use crate::game::{Game, SimSet};
use bevy::prelude::*;
use bevy::window::ExitSystems;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use warlock_core::World;
use warlock_core::content::Content;
use warlock_core::save;

/// Autosave interval, in turns.
const AUTOSAVE_TURNS: u64 = 50;
const RUN_FILE: &str = "run.sav";
const BAD_FILE: &str = "run.sav.bad";
const SNAPSHOT_FILE: &str = "snapshots/quick.sav";

/// What reading the run save found.
#[derive(Debug)]
pub enum Resume {
    /// Saving is disabled or there is no save file.
    NoSave,
    Loaded(World),
    /// The save could not be used; the message is for the player.
    Failed(String),
}

/// Where saves live, and whether saving is on at all.
#[derive(Resource, Debug, Clone)]
pub struct SaveStore {
    /// `None` disables saving (capture mode, wasm, or no resolvable data directory).
    dir: Option<PathBuf>,
    /// Dev snapshots (F5/F9) are available.
    dev: bool,
}

impl SaveStore {
    /// The store for this run's command line.
    pub fn from_args(args: &Args) -> Self {
        // Captures and replays are dev tools: they must never touch the player's save.
        let dir = if args.capture.is_some() || args.replay.is_some() {
            None
        } else {
            args.save_dir.clone().or_else(default_dir)
        };
        Self {
            dir,
            dev: cfg!(debug_assertions) || args.dev,
        }
    }

    /// Whether the command line asks to resume the saved run.
    /// Replays and recordings always start a fresh run, so they reproduce from the scenario.
    pub fn resumes(args: &Args) -> bool {
        !args.new && args.capture.is_none() && args.replay.is_none() && args.record.is_none()
    }

    /// Reads the run save. An unusable save is renamed to `run.sav.bad` (never deleted).
    pub fn resume(&self, content: &Arc<Content>) -> Resume {
        let Some(dir) = &self.dir else {
            return Resume::NoSave;
        };
        let path = dir.join(RUN_FILE);
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Resume::NoSave,
            Err(e) => return Resume::Failed(format!("Could not read the save: {e}.")),
        };
        match save::decode(&bytes, content.clone()) {
            Ok(world) => Resume::Loaded(world),
            Err(err) => {
                let kept = match std::fs::rename(&path, dir.join(BAD_FILE)) {
                    Ok(()) => format!("The old save was kept as {BAD_FILE}."),
                    Err(_) => "The old save was left in place.".to_string(),
                };
                Resume::Failed(format!(
                    "Could not resume: {err}. Starting a new run. {kept}"
                ))
            }
        }
    }

    /// Writes the run save.
    pub fn save_run(&self, world: &World) -> Result<(), String> {
        self.write(RUN_FILE, world)
    }

    /// Writes the dev snapshot. Returns `Ok(false)` when snapshots are unavailable.
    pub fn save_snapshot(&self, world: &World) -> Result<bool, String> {
        if !self.dev {
            return Ok(false);
        }
        self.write(SNAPSHOT_FILE, world).map(|()| true)
    }

    /// Reads the dev snapshot.
    pub fn load_snapshot(&self, content: &Arc<Content>) -> Result<World, String> {
        let dir = self.dir.as_ref().filter(|_| self.dev);
        let dir = dir.ok_or("Snapshots are unavailable.")?;
        let bytes = std::fs::read(dir.join(SNAPSHOT_FILE))
            .map_err(|e| format!("No snapshot to load ({e})."))?;
        save::decode(&bytes, content.clone()).map_err(|e| format!("Could not load snapshot: {e}."))
    }

    fn write(&self, name: &str, world: &World) -> Result<(), String> {
        let Some(dir) = &self.dir else {
            return Ok(());
        };
        write_atomic(&dir.join(name), &save::encode(world))
            .map_err(|e| format!("Could not save: {e}."))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn default_dir() -> Option<PathBuf> {
    crate::paths::project_dirs().map(|d| d.data_dir().to_path_buf())
}

#[cfg(target_arch = "wasm32")]
fn default_dir() -> Option<PathBuf> {
    None
}

/// Writes `bytes` to `path` via a temp file in the same directory and a rename, so a crash never
/// leaves a half-written save.
fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    let result = (|| {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// Asks the save systems to act. Written by input.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveCommand {
    /// Save the run, then quit.
    SaveAndQuit,
    /// Dev: write the quick snapshot.
    Snapshot,
    /// Dev: replace the world with the quick snapshot.
    LoadSnapshot,
}

/// Set once the quit path has saved, so the exit hook does not save again.
#[derive(Resource, Debug, Default)]
struct QuitSaved(bool);

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        let store = SaveStore::from_args(app.world().resource::<Args>());
        app.insert_resource(store)
            .init_resource::<QuitSaved>()
            .add_message::<SaveCommand>()
            .add_systems(Update, (handle_commands, autosave).chain().after(SimSet))
            .add_systems(Last, save_on_exit.after(ExitSystems));
    }
}

fn handle_commands(
    mut commands: MessageReader<SaveCommand>,
    store: Res<SaveStore>,
    mut game: ResMut<Game>,
    mut quit_saved: ResMut<QuitSaved>,
    mut exit: MessageWriter<AppExit>,
) {
    for command in commands.read() {
        match command {
            SaveCommand::SaveAndQuit => match store.save_run(&game.world) {
                Ok(()) => {
                    game.world.notice("Game saved.");
                    quit_saved.0 = true;
                    exit.write(AppExit::Success);
                }
                Err(e) => game.world.notice(format!("{e} Press Q to try again.")),
            },
            SaveCommand::Snapshot => match store.save_snapshot(&game.world) {
                Ok(true) => game.world.notice("Snapshot saved."),
                Ok(false) => {}
                Err(e) => game.world.notice(e),
            },
            SaveCommand::LoadSnapshot => match store.load_snapshot(game.world.content()) {
                Ok(world) => {
                    game.world = world;
                    game.last_outcome = default();
                    game.world.notice("Snapshot loaded.");
                }
                Err(e) => game.world.notice(e),
            },
        }
    }
}

/// Saves every [`AUTOSAVE_TURNS`] turns and whenever the level changes.
fn autosave(store: Res<SaveStore>, mut game: ResMut<Game>, mut last: Local<Option<(u64, String)>>) {
    let key = (
        game.world.turn() / AUTOSAVE_TURNS,
        game.world.perception().level_name().to_string(),
    );
    let Some(previous) = last.replace(key.clone()) else {
        return; // First frame: the world was just started or resumed.
    };
    if previous == key {
        return;
    }
    if let Err(e) = store.save_run(&game.world) {
        game.world.notice(e);
    }
}

/// Window close and any other exit that skipped the quit command still saves.
fn save_on_exit(
    mut exits: MessageReader<AppExit>,
    store: Res<SaveStore>,
    game: Res<Game>,
    quit_saved: Res<QuitSaved>,
) {
    if exits.read().count() > 0
        && !quit_saved.0
        && let Err(e) = store.save_run(&game.world)
    {
        eprintln!("warlock: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use warlock_core::{Action, Dir};

    fn content() -> Arc<Content> {
        Arc::new(Content::builtin().expect("content"))
    }

    fn store(name: &str) -> SaveStore {
        let dir = std::env::temp_dir().join(format!("warlock-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        SaveStore {
            dir: Some(dir),
            dev: true,
        }
    }

    fn world() -> World {
        let mut w = World::new(content(), "showcase", None).expect("world");
        w.apply(&Action::Move(Dir::E));
        w
    }

    #[test]
    fn run_save_round_trips_through_disk() {
        let s = store("roundtrip");
        assert!(matches!(s.resume(&content()), Resume::NoSave));
        let w = world();
        s.save_run(&w).expect("save");
        let Resume::Loaded(restored) = s.resume(&content()) else {
            panic!("expected a loaded world");
        };
        assert_eq!(restored.state(), w.state());
        let dir = s.dir.as_ref().expect("dir");
        assert!(!dir.join("run.sav.tmp").exists());
    }

    #[test]
    fn unreadable_save_is_kept_as_bad() {
        let s = store("bad");
        let dir = s.dir.clone().expect("dir");
        std::fs::create_dir_all(&dir).expect("mkdir");
        std::fs::write(dir.join(RUN_FILE), b"garbage").expect("write");
        let Resume::Failed(msg) = s.resume(&content()) else {
            panic!("expected failure");
        };
        assert!(msg.contains("not a Warlock save"), "{msg}");
        assert!(!dir.join(RUN_FILE).exists());
        assert_eq!(std::fs::read(dir.join(BAD_FILE)).expect("bad"), b"garbage");
    }

    #[test]
    fn snapshots_save_and_load() {
        let s = store("snap");
        let w = world();
        assert_eq!(s.save_snapshot(&w), Ok(true));
        let loaded = s.load_snapshot(&content()).expect("load");
        assert_eq!(loaded.state(), w.state());
        let off = SaveStore {
            dev: false,
            ..s.clone()
        };
        assert_eq!(off.save_snapshot(&w), Ok(false));
        assert!(off.load_snapshot(&content()).is_err());
    }

    #[test]
    fn disabled_store_is_a_no_op() {
        let s = SaveStore {
            dir: None,
            dev: false,
        };
        assert!(s.save_run(&world()).is_ok());
        assert!(matches!(s.resume(&content()), Resume::NoSave));
    }
}
