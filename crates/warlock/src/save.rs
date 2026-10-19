//! Saving and resuming: the run save, autosave, dev snapshots, and quit handling (spec §7).
//!
//! The core turns a world into bytes (`warlock_core::save`); this module owns where they live.
//! Saving never panics or aborts the game: failures become a message in the log. Files live in a
//! directory natively and in `localStorage` on the web (`storage.rs`); the envelope and checks are
//! the same. On the web the page being hidden or closed also saves (see `web_hooks`).

use crate::cli::Args;
use crate::game::{Game, SimSet};
use crate::storage::Backend;
use bevy::prelude::*;
use bevy::window::ExitSystems;
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
    /// `None` disables saving (capture mode, no resolvable data directory, or no browser storage).
    backend: Option<Backend>,
    /// Dev snapshots (F5/F9) are available.
    dev: bool,
}

impl SaveStore {
    /// The store for this run's command line.
    pub fn from_args(args: &Args) -> Self {
        // Captures and replays are dev tools: they must never touch the player's save.
        let backend = if args.capture.is_some() || args.replay.is_some() {
            None
        } else {
            args.save_dir
                .clone()
                .map(Backend::Dir)
                .or_else(default_backend)
        };
        Self {
            backend,
            dev: cfg!(debug_assertions) || args.dev,
        }
    }

    /// Whether the command line asks to resume the saved run.
    /// Replays and recordings always start a fresh run, so they reproduce from the scenario.
    pub fn resumes(args: &Args) -> bool {
        !args.new && args.capture.is_none() && args.replay.is_none() && args.record.is_none()
    }

    /// Whether a run save file exists (it may still turn out to be unusable).
    pub fn has_save(&self) -> bool {
        self.backend.as_ref().is_some_and(|b| b.exists(RUN_FILE))
    }

    /// Reads the run save. An unusable save is renamed to `run.sav.bad` (never deleted).
    pub fn resume(&self, content: &Arc<Content>) -> Resume {
        let Some(backend) = &self.backend else {
            return Resume::NoSave;
        };
        let bytes = match backend.read(RUN_FILE) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Resume::NoSave,
            Err(e) => return Resume::Failed(format!("Could not read the save: {e}.")),
        };
        match save::decode(&bytes, content.clone()) {
            Ok(world) => Resume::Loaded(world),
            Err(err) => {
                let kept = match backend.rename(RUN_FILE, BAD_FILE) {
                    Ok(()) => format!("The old save was kept as {BAD_FILE}."),
                    Err(_) => "The old save was left in place.".to_string(),
                };
                Resume::Failed(format!(
                    "Could not resume: {err}. Starting a new run. {kept}"
                ))
            }
        }
    }

    /// Writes the run save. A run that has ended in death is never saved: its save is deleted
    /// instead (permadeath), whatever asks to save it (autosave, quitting, closing the window).
    pub fn save_run(&self, world: &World) -> Result<(), String> {
        if world.death().is_some() {
            return self.delete_run();
        }
        self.write(RUN_FILE, world)
    }

    /// Deletes the run save (the player died): nothing is left to continue. A save that is not
    /// there is no error.
    pub fn delete_run(&self) -> Result<(), String> {
        let Some(backend) = &self.backend else {
            return Ok(());
        };
        backend
            .remove(RUN_FILE)
            .map_err(|e| format!("Could not delete the save: {e}."))
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
        let backend = self.backend.as_ref().filter(|_| self.dev);
        let backend = backend.ok_or("Snapshots are unavailable.")?;
        let bytes = backend
            .read(SNAPSHOT_FILE)
            .map_err(|e| format!("No snapshot to load ({e})."))?;
        save::decode(&bytes, content.clone()).map_err(|e| format!("Could not load snapshot: {e}."))
    }

    fn write(&self, name: &str, world: &World) -> Result<(), String> {
        let Some(backend) = &self.backend else {
            return Ok(());
        };
        backend
            .write(name, &save::encode(world))
            .map_err(|e| format!("Could not save: {e}."))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn default_backend() -> Option<Backend> {
    crate::paths::project_dirs().map(|d| Backend::Dir(d.data_dir().to_path_buf()))
}

/// Browser storage, unless the browser refuses it (some privacy modes): then saving is off.
#[cfg(target_arch = "wasm32")]
fn default_backend() -> Option<Backend> {
    if crate::storage::web::available() {
        Some(Backend::Browser)
    } else {
        warn!("browser storage is unavailable; progress will not be saved");
        None
    }
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
            .add_systems(
                Update,
                (handle_commands, autosave)
                    .chain()
                    .after(SimSet)
                    .run_if(resource_exists::<Game>),
            )
            .add_systems(Last, save_on_exit.after(ExitSystems));
        #[cfg(target_arch = "wasm32")]
        {
            web_hooks::build(app);
            // Safety net: an exit would freeze the page, so reload it instead.
            app.add_systems(Last, reload_on_exit.after(save_on_exit));
        }
    }
}

/// The web build also saves when the page is hidden or closed. A hidden tab runs no frames, so a
/// system cannot do it: after each turn a copy of the state (cheap; encoding is not, on big
/// levels) is parked in `storage::web`, and the `visibilitychange`/`pagehide` listeners encode and
/// write it.
#[cfg(target_arch = "wasm32")]
mod web_hooks {
    use super::*;
    use crate::storage::web;

    pub fn build(app: &mut App) {
        if app.world().resource::<SaveStore>().backend.is_some() {
            web::hook_page_hide(RUN_FILE);
            app.add_systems(Update, park_run.after(SimSet));
        }
    }

    fn park_run(game: Option<Res<Game>>, mut last: Local<Option<(u64, String)>>) {
        // No run, or one that ended in death (never saved again): nothing to save on hiding.
        let Some(game) = game.filter(|g| g.world.death().is_none()) else {
            if last.take().is_some() {
                web::set_pending(None);
            }
            return;
        };
        let key = (
            game.world.turn(),
            game.world.perception().level_name().to_string(),
        );
        if last.as_ref() != Some(&key) {
            web::set_pending(Some((
                game.world.state().clone(),
                game.world.content().fingerprint(),
            )));
            *last = Some(key);
        }
    }
}

// On the web, quitting reloads the page, so `exit` goes unused.
#[cfg_attr(target_arch = "wasm32", allow(unused_variables, unused_mut))]
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
                    // A web page cannot close itself: reload to a fresh main menu instead.
                    #[cfg(target_arch = "wasm32")]
                    crate::storage::web::reload();
                    #[cfg(not(target_arch = "wasm32"))]
                    exit.write(AppExit::Success);
                }
                Err(e) => game.world.notice(format!("{e} Try again.")),
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
    game: Option<Res<Game>>,
    quit_saved: Res<QuitSaved>,
) {
    // No game: the player quit from the main menu.
    if let Some(game) = game
        && exits.read().count() > 0
        && !quit_saved.0
        && let Err(e) = store.save_run(&game.world)
    {
        eprintln!("warlock: {e}");
    }
}

/// Web: any [`AppExit`] reloads the page (to the main menu) rather than freezing it.
#[cfg(target_arch = "wasm32")]
fn reload_on_exit(mut exits: MessageReader<AppExit>) {
    if exits.read().count() > 0 {
        crate::storage::web::reload();
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
            backend: Some(Backend::Dir(dir)),
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
        let Some(Backend::Dir(dir)) = &s.backend else {
            panic!("dir backend");
        };
        assert!(!dir.join("run.sav.tmp").exists());
    }

    #[test]
    fn unreadable_save_is_kept_as_bad() {
        let s = store("bad");
        let Some(Backend::Dir(dir)) = s.backend.clone() else {
            panic!("dir backend");
        };
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

    /// A run that ended in death: in the arena the player lights their lantern, and the war
    /// band's archer shoots them down from the dark.
    fn dead_world() -> World {
        let mut w = World::new(content(), "arena", None).expect("world");
        w.apply(&Action::ToggleLantern);
        for _ in 0..500 {
            if w.death().is_some() {
                break;
            }
            w.apply(&Action::Wait);
        }
        assert!(w.death().is_some(), "the player died");
        w
    }

    #[test]
    fn a_run_that_ended_in_death_is_never_saved_and_its_save_is_deleted() {
        let s = store("permadeath");
        s.save_run(&world()).expect("save");
        assert!(s.has_save());
        let dead = dead_world();
        // Autosave, quitting, and closing the window all go through `save_run`: it deletes.
        s.save_run(&dead).expect("no error");
        assert!(!s.has_save(), "the save is gone");
        assert!(matches!(s.resume(&content()), Resume::NoSave));
        // Deleting what is not there is fine.
        s.delete_run().expect("nothing to delete");
        assert!(s.save_run(&dead).is_ok());
        assert!(!s.has_save());
    }

    #[test]
    fn disabled_store_is_a_no_op() {
        let s = SaveStore {
            backend: None,
            dev: false,
        };
        assert!(s.save_run(&world()).is_ok());
        assert!(matches!(s.resume(&content()), Resume::NoSave));
    }
}
