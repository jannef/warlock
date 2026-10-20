//! Saving and resuming: one save per character, autosave, dev snapshots, and quit handling
//! (spec §7).
//!
//! The core turns a world into bytes (`warlock_core::save`); this module owns where they live.
//! Every character has its own save, `saves/<id>.sav` in the data folder (`paths::SAVES_DIR`),
//! with its summary beside it for the list of saved characters ([`characters`]); its dev
//! snapshot is `snapshots/<id>.sav`. The run in progress knows its id (`Game::save_id`), and
//! every way of saving writes that character's files only: autosave, quitting, closing the
//! window, the web page-hide save, and the snapshots. Death deletes that character's files and
//! no one else's.
//!
//! Saving never panics or aborts the game: failures become a message in the log. Files live in a
//! directory natively and in `localStorage` on the web (`storage.rs`); the envelope and checks are
//! the same. On the web the page being hidden or closed also saves (see `web_hooks`).

mod characters;

pub use characters::{SavedCharacter, Summary, played_text};

use crate::cli::Args;
use crate::game::{Game, SimSet};
use crate::paths::{SAVES_DIR, SNAPSHOTS_DIR};
use crate::storage::Backend;
use bevy::prelude::*;
use bevy::window::ExitSystems;
use std::sync::Arc;
use warlock_core::World;
use warlock_core::content::Content;
use warlock_core::save;

/// Autosave interval, in turns.
const AUTOSAVE_TURNS: u64 = 50;

/// What reading a character's save found.
#[derive(Debug)]
pub enum Resume {
    /// Saving is disabled or there is no such save.
    NoSave,
    Loaded(World),
    /// The save could not be used; the message is for the player.
    Failed(String),
}

/// Where saves live, and whether saving is on at all.
#[derive(Resource, Debug, Clone)]
pub struct SaveStore {
    /// `None` disables saving (capture mode without `--save-dir`, no resolvable data directory,
    /// or no browser storage).
    backend: Option<Backend>,
    /// Saves are read but never written, renamed, or deleted: captures and replays with
    /// `--save-dir` (the list of saved characters in a capture).
    read_only: bool,
    /// Dev snapshots (F5/F9) are available.
    dev: bool,
}

/// The save of character `id`.
fn save_file(id: &str) -> String {
    format!("{SAVES_DIR}/{id}.sav")
}

/// Where an unusable save of character `id` is kept.
fn bad_file(id: &str) -> String {
    format!("{SAVES_DIR}/{id}.sav.bad")
}

/// The summary of character `id`, beside its save.
fn summary_file(id: &str) -> String {
    format!("{SAVES_DIR}/{id}.toml")
}

/// The dev snapshot of character `id`.
fn snapshot_file(id: &str) -> String {
    format!("{SNAPSHOTS_DIR}/{id}.sav")
}

impl SaveStore {
    /// The store for this run's command line.
    pub fn from_args(args: &Args) -> Self {
        // Captures and replays are dev tools: they must never touch the player's saves. Given a
        // save folder, they may read it.
        let dev_tool = args.capture.is_some() || args.replay.is_some();
        let backend = if dev_tool {
            args.save_dir.clone().map(Backend::Dir)
        } else {
            args.save_dir
                .clone()
                .map(Backend::Dir)
                .or_else(default_backend)
        };
        Self {
            backend,
            read_only: dev_tool,
            dev: cfg!(debug_assertions) || args.dev,
        }
    }

    /// Whether the command line asks to resume the character played last.
    /// Replays and recordings always start a fresh run, so they reproduce from the scenario.
    pub fn resumes(args: &Args) -> bool {
        !args.new && args.capture.is_none() && args.replay.is_none() && args.record.is_none()
    }

    /// The backend files may be written to, renamed in, and deleted from.
    fn writable(&self) -> Option<&Backend> {
        self.backend.as_ref().filter(|_| !self.read_only)
    }

    /// Every saved character (a save, with its summary if that can be read), the one played last
    /// first. An unusable save, kept as `.sav.bad`, is not one.
    pub fn characters(&self) -> Vec<SavedCharacter> {
        let Some(backend) = &self.backend else {
            return Vec::new();
        };
        let names = backend.list(SAVES_DIR).unwrap_or_else(|e| {
            warn!("cannot list the saves: {e}");
            Vec::new()
        });
        let mut characters: Vec<SavedCharacter> = (names.iter())
            .filter_map(|name| name.strip_suffix(".sav"))
            .map(|id| SavedCharacter {
                id: id.to_string(),
                summary: (backend.read(&summary_file(id)).ok())
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                    .and_then(|text| Summary::from_text(&text)),
            })
            .collect();
        characters::sort(&mut characters);
        characters
    }

    /// The character played last, if any is saved: the one Continue resumes.
    pub fn last_played(&self) -> Option<SavedCharacter> {
        self.characters().into_iter().next()
    }

    /// A new save id for a character named `name`, taken by no file of another character (its
    /// save, a kept unusable save, its summary, or its snapshot).
    pub fn new_id(&self, name: &str) -> String {
        let listed = |dir: &str| {
            (self.backend.as_ref())
                .and_then(|b| b.list(dir).ok())
                .unwrap_or_default()
        };
        let (saves, snapshots) = (listed(SAVES_DIR), listed(SNAPSHOTS_DIR));
        characters::fresh_id(name, |id| {
            let mine = |file: &String| file.split_once('.').is_some_and(|(stem, _)| stem == id);
            saves.iter().chain(&snapshots).any(mine)
        })
    }

    /// Reads character `id`'s save. An unusable save is renamed to `<id>.sav.bad` (never
    /// deleted), and its summary goes, so the list no longer shows it.
    pub fn resume(&self, id: &str, content: &Arc<Content>) -> Resume {
        let Some(backend) = &self.backend else {
            return Resume::NoSave;
        };
        let bytes = match backend.read(&save_file(id)) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Resume::NoSave,
            Err(e) => return Resume::Failed(format!("Could not read the save: {e}.")),
        };
        match save::decode(&bytes, content.clone()) {
            Ok(world) => Resume::Loaded(world),
            Err(err) => {
                let kept = match self.writable() {
                    Some(backend) => match backend.rename(&save_file(id), &bad_file(id)) {
                        Ok(()) => {
                            let _ = backend.remove(&summary_file(id));
                            format!("The old save was kept as {}.", bad_file(id))
                        }
                        Err(_) => "The old save was left in place.".to_string(),
                    },
                    None => "The old save was left in place.".to_string(),
                };
                Resume::Failed(format!("Could not resume: {err}. {kept}"))
            }
        }
    }

    /// Writes character `id`'s save and its summary. A run that has ended in death is never
    /// saved: that character's files are deleted instead (permadeath), whatever asks to save it
    /// (autosave, quitting, closing the window).
    pub fn save_run(&self, id: &str, world: &World) -> Result<(), String> {
        if world.death().is_some() {
            return self.delete_character(id);
        }
        self.write(&save_file(id), world)?;
        let Some(backend) = self.writable() else {
            return Ok(());
        };
        let summary = Summary::of(world, characters::now());
        backend
            .write(&summary_file(id), summary.to_text().as_bytes())
            .map_err(|e| format!("Could not save: {e}."))
    }

    /// Deletes character `id`'s files: its save, its summary, and its dev snapshot (the
    /// character died, or the player deleted it). Nothing of another character's, and nothing
    /// kept as unusable. A file that is not there is no error.
    pub fn delete_character(&self, id: &str) -> Result<(), String> {
        let Some(backend) = self.writable() else {
            return Ok(());
        };
        for file in [save_file(id), summary_file(id), snapshot_file(id)] {
            backend
                .remove(&file)
                .map_err(|e| format!("Could not delete the save: {e}."))?;
        }
        Ok(())
    }

    /// Writes character `id`'s dev snapshot. Returns `Ok(false)` when snapshots are unavailable.
    pub fn save_snapshot(&self, id: &str, world: &World) -> Result<bool, String> {
        if !self.dev {
            return Ok(false);
        }
        self.write(&snapshot_file(id), world).map(|()| true)
    }

    /// Reads character `id`'s dev snapshot.
    pub fn load_snapshot(&self, id: &str, content: &Arc<Content>) -> Result<World, String> {
        let backend = self.backend.as_ref().filter(|_| self.dev);
        let backend = backend.ok_or("Snapshots are unavailable.")?;
        let bytes = backend
            .read(&snapshot_file(id))
            .map_err(|e| format!("No snapshot to load ({e})."))?;
        save::decode(&bytes, content.clone()).map_err(|e| format!("Could not load snapshot: {e}."))
    }

    fn write(&self, name: &str, world: &World) -> Result<(), String> {
        let Some(backend) = self.writable() else {
            return Ok(());
        };
        backend
            .write(name, &save::encode(world))
            .map_err(|e| format!("Could not save: {e}."))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn default_backend() -> Option<Backend> {
    crate::paths::locations().data.dir.clone().map(Backend::Dir)
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
/// levels) and the character's summary are parked in `storage::web`, and the
/// `visibilitychange`/`pagehide` listeners encode and write them as the character's files.
#[cfg(target_arch = "wasm32")]
mod web_hooks {
    use super::*;
    use crate::storage::web;

    pub fn build(app: &mut App) {
        if app.world().resource::<SaveStore>().writable().is_some() {
            web::hook_page_hide();
            app.add_systems(Update, park_run.after(SimSet));
        }
    }

    fn park_run(game: Option<Res<Game>>, mut last: Local<Option<(String, u64, String)>>) {
        // No run, or one that ended in death (never saved again): nothing to save on hiding.
        let Some(game) = game.filter(|g| g.world.death().is_none()) else {
            if last.take().is_some() {
                web::set_pending(None);
            }
            return;
        };
        let key = (
            game.save_id.clone(),
            game.world.turn(),
            game.world.perception().level_name().to_string(),
        );
        if last.as_ref() != Some(&key) {
            let summary = Summary::of(&game.world, characters::now());
            web::set_pending(Some(web::Pending {
                state: game.world.state().clone(),
                fingerprint: game.world.content().fingerprint(),
                file: save_file(&game.save_id),
                beside: (summary_file(&game.save_id), summary.to_text()),
            }));
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
        let id = game.save_id.clone();
        match command {
            SaveCommand::SaveAndQuit => match store.save_run(&id, &game.world) {
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
            SaveCommand::Snapshot => match store.save_snapshot(&id, &game.world) {
                Ok(true) => game.world.notice("Snapshot saved."),
                Ok(false) => {}
                Err(e) => game.world.notice(e),
            },
            SaveCommand::LoadSnapshot => match store.load_snapshot(&id, game.world.content()) {
                Ok(world) => {
                    game.world = world;
                    game.last_outcome = default();
                    game.mark_read();
                    game.world.notice("Snapshot loaded.");
                }
                Err(e) => game.world.notice(e),
            },
        }
    }
}

/// Saves the character as soon as a run starts or is resumed (so a new character is on the list
/// at once), then every [`AUTOSAVE_TURNS`] turns and whenever the level changes.
fn autosave(
    store: Res<SaveStore>,
    mut game: ResMut<Game>,
    mut last: Local<Option<(String, u64, String)>>,
) {
    let key = (
        game.save_id.clone(),
        game.world.turn() / AUTOSAVE_TURNS,
        game.world.perception().level_name().to_string(),
    );
    if last.replace(key.clone()).as_ref() == Some(&key) {
        return;
    }
    if let Err(e) = store.save_run(&key.0, &game.world) {
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
        && let Err(e) = store.save_run(&game.save_id, &game.world)
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
    use warlock_core::character::CharacterSpec;
    use warlock_core::{Action, Dir};

    fn content() -> Arc<Content> {
        Arc::new(Content::builtin().expect("content"))
    }

    fn store(name: &str) -> SaveStore {
        let dir = std::env::temp_dir().join(format!("warlock-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        SaveStore {
            backend: Some(Backend::Dir(dir)),
            read_only: false,
            dev: true,
        }
    }

    fn dir(store: &SaveStore) -> std::path::PathBuf {
        let Some(Backend::Dir(dir)) = &store.backend else {
            panic!("dir backend");
        };
        dir.clone()
    }

    fn world() -> World {
        let mut w = World::new(content(), "showcase", None).expect("world");
        w.apply(&Action::Move(Dir::E));
        w
    }

    /// A fresh run as a dwarf warrior named `name`.
    fn dwarf(name: &str) -> World {
        let spec: CharacterSpec = "dwarf,female,warrior".parse().expect("a character");
        World::with_character_named(content(), "showcase", Some(3), Some(&spec), Some(name))
            .expect("world")
    }

    fn ids(store: &SaveStore) -> Vec<String> {
        store.characters().into_iter().map(|c| c.id).collect()
    }

    #[test]
    fn a_characters_save_round_trips_through_disk() {
        let s = store("roundtrip");
        assert!(matches!(s.resume("tamsin-1", &content()), Resume::NoSave));
        let w = world();
        let id = s.new_id("Tamsin");
        assert_eq!(id, "tamsin-1");
        s.save_run(&id, &w).expect("save");
        let Resume::Loaded(restored) = s.resume(&id, &content()) else {
            panic!("expected a loaded world");
        };
        assert_eq!(restored.state(), w.state());
        let dir = dir(&s);
        assert!(dir.join("saves/tamsin-1.sav").is_file());
        assert!(!dir.join("saves/tamsin-1.sav.tmp").exists());
        // Its summary lies beside it, for the list.
        let listed = s.characters();
        assert_eq!(listed.len(), 1);
        let summary = listed[0].summary.clone().expect("a summary");
        assert_eq!((summary.name.as_str(), summary.level), ("Tamsin", 1));
        assert_eq!(summary.race_and_class(), "Human Thief");
        assert_eq!(s.last_played().map(|c| c.id), Some(id));
    }

    #[test]
    fn two_characters_of_one_name_have_saves_of_their_own() {
        let s = store("namesakes");
        let (first, second) = (dwarf("Brakka"), dwarf("Brakka"));
        let a = s.new_id("Brakka");
        s.save_run(&a, &first).expect("save");
        let b = s.new_id("Brakka");
        assert_eq!((a.as_str(), b.as_str()), ("brakka-1", "brakka-2"));
        let mut moved = second;
        moved.apply(&Action::Move(Dir::N));
        s.save_run(&b, &moved).expect("save");
        assert_eq!(s.characters().len(), 2);
        let Resume::Loaded(one) = s.resume(&a, &content()) else {
            panic!("first");
        };
        let Resume::Loaded(two) = s.resume(&b, &content()) else {
            panic!("second");
        };
        assert_eq!(one.state(), first.state());
        assert_eq!(two.state(), moved.state());
        // A snapshot or a kept unusable save also takes its id.
        s.save_snapshot("brakka-3", &first).expect("snapshot");
        assert_eq!(s.new_id("Brakka"), "brakka-4");
    }

    #[test]
    fn unreadable_save_is_kept_as_bad_and_leaves_the_list() {
        let s = store("bad");
        s.save_run("tamsin-1", &world()).expect("save");
        s.save_run("brakka-1", &dwarf("Brakka")).expect("save");
        let dir = dir(&s);
        std::fs::write(dir.join("saves/tamsin-1.sav"), b"garbage").expect("write");
        let Resume::Failed(msg) = s.resume("tamsin-1", &content()) else {
            panic!("expected failure");
        };
        assert!(msg.contains("not a Warlock save"), "{msg}");
        assert!(!dir.join("saves/tamsin-1.sav").exists());
        assert_eq!(
            std::fs::read(dir.join("saves/tamsin-1.sav.bad")).expect("bad"),
            b"garbage"
        );
        assert_eq!(ids(&s), ["brakka-1"]);
        // Its id stays taken, so a new Tamsin never overwrites what was kept.
        assert_eq!(s.new_id("Tamsin"), "tamsin-2");
    }

    #[test]
    fn continue_is_the_character_played_last() {
        let s = store("last");
        s.save_run("tamsin-1", &world()).expect("save");
        s.save_run("brakka-1", &dwarf("Brakka")).expect("save");
        // Saved in the same second, so the times tie: make Tamsin's later.
        let file = dir(&s).join("saves/tamsin-1.toml");
        let mut summary =
            Summary::from_text(&std::fs::read_to_string(&file).expect("summary")).expect("valid");
        summary.played += 60;
        std::fs::write(&file, summary.to_text()).expect("write");
        assert_eq!(ids(&s), ["tamsin-1", "brakka-1"]);
        assert_eq!(s.last_played().map(|c| c.id).as_deref(), Some("tamsin-1"));
    }

    #[test]
    fn a_save_without_its_summary_is_listed_by_its_id() {
        let s = store("bare");
        s.save_run("tamsin-1", &world()).expect("save");
        std::fs::remove_file(dir(&s).join("saves/tamsin-1.toml")).expect("remove");
        let listed = s.characters();
        assert_eq!(listed.len(), 1);
        assert_eq!(
            (listed[0].name(), listed[0].summary.is_none()),
            ("tamsin-1", true)
        );
    }

    #[test]
    fn snapshots_save_and_load_per_character() {
        let s = store("snap");
        let (w, other) = (world(), dwarf("Brakka"));
        assert_eq!(s.save_snapshot("tamsin-1", &w), Ok(true));
        assert_eq!(s.save_snapshot("brakka-1", &other), Ok(true));
        let loaded = s.load_snapshot("tamsin-1", &content()).expect("load");
        assert_eq!(loaded.state(), w.state());
        assert!(s.load_snapshot("nobody-1", &content()).is_err());
        let off = SaveStore {
            dev: false,
            ..s.clone()
        };
        assert_eq!(off.save_snapshot("tamsin-1", &w), Ok(false));
        assert!(off.load_snapshot("tamsin-1", &content()).is_err());
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
    fn death_deletes_only_the_dead_characters_save() {
        let s = store("permadeath");
        s.save_run("pip-1", &world()).expect("save");
        s.save_snapshot("pip-1", &world()).expect("snapshot");
        s.save_run("brakka-1", &dwarf("Brakka")).expect("save");
        s.save_snapshot("brakka-1", &dwarf("Brakka"))
            .expect("snapshot");
        let dead = dead_world();
        // Autosave, quitting, and closing the window all go through `save_run`: it deletes.
        s.save_run("pip-1", &dead).expect("no error");
        assert_eq!(
            ids(&s),
            ["brakka-1"],
            "only the dead character's save is gone"
        );
        assert!(matches!(s.resume("pip-1", &content()), Resume::NoSave));
        let dir = dir(&s);
        assert!(!dir.join("saves/pip-1.toml").exists());
        assert!(!dir.join("snapshots/pip-1.sav").exists());
        assert!(dir.join("snapshots/brakka-1.sav").exists());
        assert!(matches!(
            s.resume("brakka-1", &content()),
            Resume::Loaded(_)
        ));
        // Deleting what is not there is fine, and a dead run is never written.
        s.delete_character("pip-1").expect("nothing to delete");
        assert!(s.save_run("pip-1", &dead).is_ok());
        assert_eq!(ids(&s), ["brakka-1"]);
    }

    #[test]
    fn a_read_only_store_reads_but_never_writes() {
        let s = store("read-only");
        s.save_run("tamsin-1", &world()).expect("save");
        let ro = SaveStore {
            read_only: true,
            ..s.clone()
        };
        assert_eq!(ids(&ro), ["tamsin-1"]);
        assert!(matches!(
            ro.resume("tamsin-1", &content()),
            Resume::Loaded(_)
        ));
        ro.save_run("brakka-1", &dwarf("Brakka")).expect("no error");
        ro.delete_character("tamsin-1").expect("no error");
        assert_eq!(ids(&s), ["tamsin-1"], "nothing written or deleted");
        std::fs::write(dir(&s).join("saves/tamsin-1.sav"), b"garbage").expect("write");
        assert!(matches!(
            ro.resume("tamsin-1", &content()),
            Resume::Failed(_)
        ));
        assert!(dir(&s).join("saves/tamsin-1.sav").exists(), "not renamed");
    }

    #[test]
    fn disabled_store_is_a_no_op() {
        let s = SaveStore {
            backend: None,
            read_only: false,
            dev: false,
        };
        assert!(s.save_run("tamsin-1", &world()).is_ok());
        assert!(matches!(s.resume("tamsin-1", &content()), Resume::NoSave));
        assert!(s.characters().is_empty());
        assert_eq!(s.new_id("Tamsin"), "tamsin-1");
    }
}
