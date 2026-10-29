//! The running game: owns the simulation and applies player actions.

use crate::cli::Args;
use crate::save::{Resume, SaveStore};
use bevy::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use warlock_core::character::CharacterSpec;
use warlock_core::content::{Content, ContentFiles};
use warlock_core::messages::{Message, MessageLog};
use warlock_core::replay::{Divergence, Replay};
use warlock_core::world::WorldError;
use warlock_core::{Action, Outcome, World};

/// Loaded content, shared by the menu (scenario list) and every run.
#[derive(Resource, Debug, Clone)]
pub struct GameContent(pub Arc<Content>);

/// Loads the content named by the command line (the built-in copy unless `--content-dir`).
pub fn load_content(args: &Args) -> Result<Arc<Content>, String> {
    let files = match &args.content_dir {
        Some(dir) => ContentFiles::from_dir(dir)
            .map_err(|e| format!("cannot read content from {}: {e}", dir.display()))?,
        None => ContentFiles::builtin(),
    };
    Content::load(&files)
        .map(Arc::new)
        .map_err(|e| format!("invalid content: {e}"))
}

/// The simulation as an ECS resource. It exists only while a run is in progress: the main menu
/// runs without it, and everything that needs a run is gated on `resource_exists::<Game>`.
#[derive(Resource)]
pub struct Game {
    pub world: World,
    /// Events from the most recent turn, for presentation.
    pub last_outcome: Outcome,
    /// How many actions this session applied: presentation that must read each outcome once,
    /// even one that took no turn (a dev spell effect), watches it.
    pub applied: u64,
    /// The character's save id (`save::SaveStore`): every way of saving writes its files.
    pub save_id: String,
    /// The run was made here and has not been shown yet: New run's Start, or a windowed start
    /// that makes a run (not resumed, replayed, or captured). The app opens the story screen
    /// over its first turn once (`ui::screens::on_run_start`); nothing of it is saved.
    pub story_due: bool,
    /// The message log as it was when the player last acted: what was logged since is this
    /// turn's news, which the message line shows first ([`Game::unread`], `ui::more`).
    read: MessageLog,
    /// Present with `--record`: the run so far, saved after every action.
    recorder: Option<(Replay, PathBuf)>,
}

impl Game {
    /// A fresh run of `scenario` as `character` (the scenario's if `None`) named `name` (the
    /// run's default if `None`), e.g. from the main menu, with a new save id from `store`: it
    /// never touches another character's save. The player chose this character, so it starts
    /// with its starting kit (spec §8 "Character creation").
    pub fn new_run(
        content: Arc<Content>,
        store: &SaveStore,
        scenario: &str,
        seed: Option<u64>,
        character: Option<&CharacterSpec>,
        name: Option<&str>,
    ) -> Result<Self, String> {
        let world = World::start_run(content, scenario, seed, character, name, true)
            .map_err(|e| format!("cannot start scenario `{scenario}`: {e}"))?;
        // The run's first words (the welcome) are news, and its story is told.
        Ok(Self {
            read: MessageLog::default(),
            story_due: true,
            ..Self::new_character(world, store)
        })
    }

    /// A game around a world loaded elsewhere (a saved character's run), saved as `save_id`,
    /// its messages all read.
    pub fn from_world(world: World, save_id: String) -> Self {
        Self {
            read: world.messages().clone(),
            world,
            last_outcome: Outcome::default(),
            applied: 0,
            save_id,
            story_due: false,
            recorder: None,
        }
    }

    /// A game around a fresh run, with a new save id from `store`.
    fn new_character(world: World, store: &SaveStore) -> Self {
        let name = world.character().map_or("", |c| c.name.as_str());
        let id = store.new_id(name);
        Self::from_world(world, id)
    }

    /// Character `id`'s saved run, if it is usable.
    pub fn resumed(store: &SaveStore, id: &str, content: &Arc<Content>) -> Resume {
        match store.resume(id, content) {
            Resume::Loaded(mut world) => {
                info!("resumed {id} at turn {}", world.turn());
                world.notice("Welcome back.");
                Resume::Loaded(world)
            }
            other => other,
        }
    }

    /// Starts the game the command line asks for: the character played last, a replay, or the
    /// scenario (as `--character` named `--name`, else the scenario's). Only used when the main
    /// menu is skipped.
    pub fn start(args: &Args, content: Arc<Content>) -> Result<Self, String> {
        let mut failure = None;
        let store = SaveStore::from_args(args);
        if SaveStore::resumes(args)
            && let Some(last) = store.last_played()
        {
            match Self::resumed(&store, &last.id, &content) {
                Resume::Loaded(world) => return Ok(Self::from_world(*world, last.id)),
                Resume::Failed(message) => failure = Some(message),
                Resume::NoSave => {}
            }
        }
        let mut last_outcome = Outcome::default();
        let mut read = MessageLog::default();
        // A run made here, in a window, tells its story as it begins.
        let mut story_due = false;
        let (world, replay) = match &args.replay {
            Some(path) => {
                let text = std::fs::read_to_string(path)
                    .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
                let replay =
                    Replay::from_json(&text).map_err(|e| format!("{}: {e}", path.display()))?;
                let (world, divergence, outcome, before) = play_replay(&replay, content)
                    .map_err(|e| format!("{}: {e}", path.display()))?;
                if let Some(divergence) = divergence {
                    eprintln!("warlock: warning: {}: {divergence}", path.display());
                }
                (last_outcome, read) = (outcome, before);
                (world, Some(replay))
            }
            None => {
                let scenario = &args.scenario_name(&content).to_owned();
                let name = args.name.as_deref();
                // `--talents` picks for `--character`, else for the scenario's own character.
                let talents = args
                    .talents
                    .as_deref()
                    .map(warlock_core::world::parse_talents);
                let character = match &talents {
                    Some(picks) => warlock_core::world::chosen_with_talents(
                        &content,
                        scenario,
                        args.character.as_ref(),
                        picks,
                    ),
                    None => args.character.clone(),
                };
                // A character chosen with `--character`, `--name`, or `--talents` starts with
                // its kit; the scenario's own (`--scenario` alone) with the scenario's pack.
                let kit = args.character.is_some() || name.is_some() || talents.is_some();
                let character = character.as_ref();
                let mut world =
                    World::start_run(content, scenario, args.seed, character, name, kit)
                        .map_err(|e| format!("cannot start scenario `{scenario}`: {e}"))?;
                if let Some(message) = failure {
                    world.notice(message);
                }
                let replay = args.record.is_some().then(|| Replay::new(&world));
                story_due = args.capture.is_none();
                (world, replay)
            }
        };
        let recorder = args.record.clone().zip(replay).map(|(path, r)| (r, path));
        let game = Self {
            last_outcome,
            read,
            recorder,
            story_due,
            ..Self::new_character(world, &store)
        };
        game.save_recording();
        Ok(game)
    }

    /// Applies `action`, recording it if `--record` is on.
    pub fn apply(&mut self, action: &Action) {
        self.read = self.world.messages().clone();
        self.last_outcome = self.world.apply(action);
        self.applied += 1;
        if let Some((replay, _)) = &mut self.recorder {
            replay.push(action, &self.world);
        }
        self.save_recording();
    }

    /// Counts every message logged so far as read: the world was replaced (a snapshot loaded).
    pub fn mark_read(&mut self) {
        self.read = self.world.messages().clone();
    }

    /// This turn's news: the messages logged since the player last acted (since the run began,
    /// for a new run), oldest first. A message read before and logged again since (folded into
    /// the same entry, `(×2)`) is news again, with its count.
    pub fn unread(&self) -> Vec<&Message> {
        let log: Vec<&Message> = self.world.messages().iter().collect();
        let from = unread_from(&self.read, &log);
        log[from..].to_vec()
    }

    /// Rewrites the `--record` file; a failure is reported but never stops the game.
    fn save_recording(&self) {
        if let Some((replay, path)) = &self.recorder
            && let Err(err) = std::fs::write(path, replay.to_json())
        {
            eprintln!("warlock: cannot write recording {}: {err}", path.display());
        }
    }
}

/// Where the messages logged since `read` begin in `log`. The log only grows at its end (a
/// message logged again in a row folds into the last entry) and drops its oldest entries beyond
/// its capacity, so what is left of `read` lines up with the start of `log`: all of it but its
/// last entry as it was, that one perhaps repeated since (then it is news again). With nothing
/// read, or nothing lining up (more news than the log keeps), everything is news.
fn unread_from(read: &MessageLog, log: &[&Message]) -> usize {
    let read: Vec<&Message> = read.iter().collect();
    let Some((last, before)) = read.split_last() else {
        return 0;
    };
    for dropped in 0..=before.len() {
        let kept = &before[dropped..];
        let n = kept.len();
        if log.len() > n && log[..n] == *kept && log[n].text == last.text {
            return if log[n] == *last { n + 1 } else { n };
        }
    }
    0
}

/// Plays `replay` like [`Replay::play`], also returning the outcome of its last step, so
/// presentation shows that turn's events (e.g. a splash in a `--capture`), and the message log
/// before it, so the message line shows that turn's messages as news.
fn play_replay(
    replay: &Replay,
    content: Arc<Content>,
) -> Result<(World, Option<Divergence>, Outcome, MessageLog), WorldError> {
    let mut head = replay.clone();
    let Some(last) = head.steps.pop() else {
        let (world, divergence) = replay.play(content)?;
        return Ok((world, divergence, Outcome::default(), MessageLog::default()));
    };
    let (mut world, mut divergence) = head.play(content)?;
    let before = world.messages().clone();
    let outcome = world.apply(&last.action);
    let actual = world.state_hash();
    if divergence.is_none() && actual != last.hash {
        divergence = Some(Divergence {
            step: head.steps.len(),
            action: last.action,
            expected: last.hash,
            actual,
        });
    }
    Ok((world, divergence, outcome, before))
}

/// A gameplay action requested by input (or a replay).
#[derive(Message, Debug, Clone)]
pub struct PlayerAction(pub Action);

/// Systems that advance the simulation. Input runs before it, presentation after.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimSet;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlayerAction>().add_systems(
            Update,
            apply_actions.in_set(SimSet).run_if(resource_exists::<Game>),
        );
    }
}

fn apply_actions(mut actions: MessageReader<PlayerAction>, mut game: ResMut<Game>) {
    for PlayerAction(action) in actions.read() {
        game.apply(action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use warlock_core::messages::CAPACITY;
    use warlock_core::{Dir, Event};

    fn texts(messages: &[&Message]) -> Vec<String> {
        messages.iter().map(|m| m.display()).collect()
    }

    #[test]
    fn unread_is_what_was_logged_since_the_mark() {
        let mut read = MessageLog::default();
        let mut log = MessageLog::default();
        log.push(1, "A");
        let news = |read: &MessageLog, log: &MessageLog| {
            let all: Vec<&Message> = log.iter().collect();
            texts(&all[unread_from(read, &all)..])
        };
        // Nothing read: everything is news.
        assert_eq!(news(&read, &log), ["A"]);
        read = log.clone();
        assert!(news(&read, &log).is_empty());
        log.push(2, "B");
        log.push(2, "C");
        assert_eq!(news(&read, &log), ["B", "C"]);
        // The last one read, logged again, is news again with its count.
        read = log.clone();
        log.push(3, "C");
        log.push(3, "D");
        assert_eq!(news(&read, &log), ["C (×2)", "D"]);
    }

    #[test]
    fn unread_lines_up_after_the_full_log_drops_its_oldest() {
        let mut log = MessageLog::default();
        for i in 0..CAPACITY {
            log.push(i as u64, format!("m{i}"));
        }
        let read = log.clone();
        for i in 0..5 {
            log.push(1000, format!("n{i}"));
        }
        let all: Vec<&Message> = log.iter().collect();
        assert_eq!(all.len(), CAPACITY);
        assert_eq!(
            texts(&all[unread_from(&read, &all)..]),
            ["n0", "n1", "n2", "n3", "n4"]
        );
        // More news than the log keeps: all of it is news.
        for i in 0..CAPACITY {
            log.push(2000, format!("o{i}"));
        }
        let all: Vec<&Message> = log.iter().collect();
        assert_eq!(unread_from(&read, &all), 0);
    }

    #[test]
    fn a_new_run_starts_with_news_and_each_action_marks_what_was_read() {
        let content = Arc::new(Content::builtin().expect("built-in content loads"));
        let store = SaveStore::from_args(&Args::parse_from(["warlock", "--capture", "x.png"]));
        let mut game = Game::new_run(content, &store, "showcase", None, None, None).expect("run");
        assert!(texts(&game.unread())[0].starts_with("Welcome to Showcase."));
        for action in warlock_core::action::parse_actions("move:N*3 open:N").expect("actions") {
            game.apply(&action);
        }
        assert_eq!(texts(&game.unread()), ["You open the door."]);
        game.apply(&Action::Wait);
        assert!(game.unread().is_empty());
        // A resumed run has read everything so far.
        let resumed = Game::from_world(game.world, "tamsin-1".into());
        assert!(resumed.unread().is_empty());
    }

    #[test]
    fn replays_keep_the_last_steps_events() {
        let content = Arc::new(Content::builtin().expect("built-in content loads"));
        let mut world = World::new(content.clone(), "showcase", None).expect("scenario starts");
        let mut replay = Replay::new(&world);
        for action in [Action::Move(Dir::N), Action::Move(Dir::N)] {
            world.apply(&action);
            replay.push(&action, &world);
        }
        let (played, divergence, outcome, _) =
            play_replay(&replay, content.clone()).expect("replay plays");
        assert_eq!(divergence, None);
        assert_eq!(played.state_hash(), world.state_hash());
        assert!(matches!(
            outcome.events.as_slice(),
            [Event::ActorMoved { .. }, ..]
        ));

        replay.steps.last_mut().expect("two steps").hash ^= 1;
        let (_, divergence, ..) = play_replay(&replay, content.clone()).expect("replay plays");
        assert_eq!(divergence.map(|d| d.step), Some(1));
        replay.steps.clear();
        let (_, _, outcome, _) = play_replay(&replay, content).expect("empty replay plays");
        assert_eq!(outcome, Outcome::default());
    }

    /// Only a run made here, in a window, has its story due: New run's Start, or a windowed
    /// `--character` start; never a capture, a replay, or a resumed character.
    #[test]
    fn only_a_run_made_in_a_window_has_its_story_due() {
        let content = Arc::new(Content::builtin().expect("content"));
        let dir = std::env::temp_dir().join(format!("warlock-story-{}", std::process::id()));
        let saves = dir.to_string_lossy().into_owned();
        let start = |extra: &[&str]| {
            let mut argv = vec!["warlock", "--save-dir", saves.as_str(), "--new"];
            argv.extend(extra);
            Game::start(&Args::parse_from(argv), content.clone()).expect("starts")
        };
        let windowed = start(&["--character", "dwarf,female,warrior"]);
        assert!(windowed.story_due);
        assert!(!windowed.world.perception().story().is_empty());
        let captured = start(&["--character", "dwarf,female,warrior", "--capture", "x.png"]);
        assert!(!captured.story_due);
        let replay = dir.join("run.json");
        std::fs::create_dir_all(&dir).expect("dir");
        std::fs::write(&replay, Replay::new(&windowed.world).to_json()).expect("replay");
        let replayed = start(&["--replay", &replay.to_string_lossy()]);
        assert!(!replayed.story_due);
        let resumed = Game::from_world(windowed.world, "dagny-1".into());
        assert!(!resumed.story_due);
        let store = SaveStore::from_args(&Args::parse_from(["warlock", "--save-dir", &saves]));
        let spec: CharacterSpec = "dwarf,female,warrior".parse().expect("a character");
        let new = Game::new_run(content, &store, "showcase", None, Some(&spec), None);
        assert!(new.expect("starts").story_due);
        std::fs::remove_dir_all(&dir).ok();
    }
}
