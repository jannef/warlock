//! Command-line options of the game binary (see spec §9).

use crate::ui::ScreenName;
use bevy::prelude::*;
use clap::Parser;
use std::path::PathBuf;
use warlock_core::character::CharacterSpec;

/// Options parsed from the command line. On the web build there is no command line, so the
/// defaults are used.
#[derive(Debug, Clone, Parser, Resource)]
#[command(name = "warlock", version, about = "Warlock — an ASCII roguelike")]
pub struct Args {
    /// Scenario to start (a file stem under `assets/scenarios/`; default `showcase`). Giving it
    /// skips the main menu.
    #[arg(long)]
    pub scenario: Option<String>,

    /// Run seed: a number, or a word that is hashed (`--seed warlock`); defaults to the
    /// scenario's seed.
    #[arg(long, value_parser = warlock_core::rng::parse_seed)]
    pub seed: Option<u64>,

    /// The character to start a new run as, RACE,GENDER,CLASS (e.g. `dwarf,female,warrior`; ids
    /// from `data/races.toml` and `data/classes/`; gender male, female, or other); defaults
    /// to the scenario's. Like `--seed`, it applies to the run that `--scenario`, `--new`,
    /// `--record`, or `--capture` starts; a replay's character wins.
    #[arg(long)]
    pub character: Option<CharacterSpec>,

    /// The name of the character a new run starts as (up to 20 characters); defaults to the
    /// scenario's for its own character, else one of the race's names picked from the seed.
    /// Like `--character`, it applies to the run that `--scenario`, `--new`, `--record`, or
    /// `--capture` starts; a replay's name wins.
    #[arg(long)]
    pub name: Option<String>,

    /// Apply this replay file at startup (its scenario and seed win); hashes are verified and a
    /// divergence is logged as a warning.
    #[arg(long)]
    pub replay: Option<PathBuf>,

    /// Record every applied action to this replay file, rewritten after each action. Includes
    /// the steps of `--replay`, if given.
    #[arg(long)]
    pub record: Option<PathBuf>,

    /// Read content (`data/`, `levels/`, `scenarios/`, `vaults/`) from this directory instead of
    /// the built-in copy, e.g. `--content-dir assets`.
    #[arg(long)]
    pub content_dir: Option<PathBuf>,

    /// Freeze visual time at this many seconds (reproducible screenshots). Glyph animations and
    /// particles show that moment; slides, camera moves, and fades complete instantly.
    #[arg(long)]
    pub freeze_time: Option<f32>,

    /// Force reduced effects on: no glyph animation, light flicker, movement slides, camera
    /// easing, particles, bloom, heat haze, screen fades, screen shake, or falling rain, and spell
    /// effects are still flashes (spec §4.7). The settings file can turn them on too; this flag
    /// always wins.
    #[arg(long)]
    pub reduced_effects: bool,

    /// Window size in physical pixels, e.g. `1280x720`.
    #[arg(long, value_parser = parse_size, default_value = "1280x720")]
    pub window: UVec2,

    /// Override the display scale factor (e.g. `1` for reproducible screenshots).
    #[arg(long)]
    pub scale: Option<f32>,

    /// Save a screenshot to this path once the scene has rendered, then exit.
    #[arg(long)]
    pub capture: Option<PathBuf>,

    /// Skip the main menu and start a fresh run (the old save is overwritten on the next save).
    #[arg(long)]
    pub new: bool,

    /// Enable developer features in release builds (F2 reveal map, F3 next vision profile, F4
    /// light values, F5/F9 snapshots, F6 skip an hour, F7 next weather, New run's seed field).
    /// Always on in debug builds.
    #[arg(long)]
    pub dev: bool,

    /// Keep saves in this directory instead of the platform data directory.
    #[arg(long)]
    pub save_dir: Option<PathBuf>,

    /// Keep `settings.toml` in this directory instead of the platform config directory. Captures
    /// and replays otherwise use default settings and never write the file.
    #[arg(long)]
    pub config_dir: Option<PathBuf>,

    /// Run portable: keep settings, saves, and logs in a `warlock-data` folder next to the game
    /// (made if missing), or in DIR. A `warlock-data` folder next to the game does the same
    /// without this flag. `--config-dir` and `--save-dir` still win for their own folder.
    #[arg(long, num_args = 0..=1, value_name = "DIR")]
    pub portable: Option<Option<PathBuf>>,

    /// Print where settings, saves, snapshots, and logs are for this launch, and why, then exit.
    #[arg(long)]
    pub paths: bool,

    /// Move the game's files, then exit: `native` takes the portable folder's contents into the
    /// platform's standard folders (and renames the emptied folder); `portable [DIR]` does the
    /// reverse (DIR defaults to `warlock-data` next to the game). Nothing is overwritten.
    #[arg(long, num_args = 1..=2, value_names = ["TARGET", "DIR"])]
    pub move_data: Option<Vec<String>>,

    /// With `--move-data`: print the plan and change nothing.
    #[arg(long, requires = "move_data")]
    pub dry_run: bool,

    /// Put every key back to its default and save that, then start as usual: the way back from
    /// keys that cannot be used (spec §6 "Rebinding").
    #[arg(long)]
    pub reset_keys: bool,

    /// Write the keymap (every command's keys, by context) to FILE, then exit. In the game,
    /// Controls > Export keys writes `warlock-keys.toml` in the config folder.
    #[arg(long, value_name = "FILE")]
    pub export_keys: Option<PathBuf>,

    /// Read a keymap from FILE into the settings (the commands it does not list keep their keys),
    /// say what changed, then exit. In the game, Controls > Import keys reads
    /// `warlock-keys.toml` from the config folder.
    #[arg(long, value_name = "FILE")]
    pub import_keys: Option<PathBuf>,

    /// Dev: open this screen at startup, even with `--capture`. `main`, `new-run`, and
    /// `characters` (the saved characters: with `--capture`, those in `--save-dir`, read only)
    /// open on the main menu, and so do character creation's `race`, `gender`, `class`, `name`,
    /// and `summary` (over New run and the steps before them, each on its preselection: the
    /// scenario's character, an empty name);
    /// `pause`, `help`, `inventory`, `equipment`, `character`, `pick-up`, and `drop` over a run,
    /// `equip-item` (choosing what goes in the first slot) over the equipment screen, and
    /// `describe` (what the first stack carried is) over the inventory; `settings`, `files`
    /// (over Settings), and `controls` over the main menu, or over the pause menu when `--scenario`,
    /// `--new`, `--replay`, or `--record` names a run.
    #[arg(long, value_enum)]
    pub screen: Option<ScreenName>,

    /// Dev: start the run aiming the readied missile weapon, as `f` would (the cursor on the
    /// nearest hostile creature in view), so `--capture` shows the targeting line.
    #[arg(long)]
    pub fire: bool,

    /// Dev: start the run in look mode with the cursor on level cell `X,Y` (e.g. `--look 56,13`),
    /// so `--capture` shows what look mode says there.
    #[arg(long, value_name = "X,Y", value_parser = parse_cell)]
    pub look: Option<warlock_core::Pos>,

    /// Dev: press these keys at startup, after any `--replay`, as if typed, so `--capture` shows
    /// what they open (a dialog, e.g. `--keys h` at a shop's door owing, `--keys "d b Enter"` to
    /// sell): single characters, or `Enter`, `Esc`, `Tab`, `Up`, `Down`, `Left`, `Right`,
    /// separated by spaces.
    #[arg(long, value_name = "KEYS", value_parser = parse_keys)]
    pub keys: Option<Keys>,

    /// Number of frames to render before taking the `--capture` screenshot.
    #[arg(long, default_value_t = 10)]
    pub capture_after_frames: u32,
}

impl Args {
    /// The scenario to start directly.
    pub fn scenario_name(&self) -> &str {
        self.scenario.as_deref().unwrap_or("showcase")
    }

    /// Whether the game opens on the main menu. Flags that name what to run (`--scenario`,
    /// `--new`, `--replay`, `--record`, `--capture`) start it directly, so dev flows and
    /// captures need no input. `--screen` picks: the main menu for `main`, `new-run`, and
    /// character creation's screens, a run for `pause` and `help`, and for `settings` and
    /// `controls` the menu unless a run is named (`--capture` alone does not name one).
    pub fn starts_in_menu(&self) -> bool {
        let names_run =
            self.scenario.is_some() || self.new || self.replay.is_some() || self.record.is_some();
        match self.screen {
            Some(
                ScreenName::Main
                | ScreenName::Characters
                | ScreenName::NewRun
                | ScreenName::Race
                | ScreenName::Sign
                | ScreenName::Gender
                | ScreenName::Class
                | ScreenName::Name
                | ScreenName::Summary,
            ) => true,
            Some(s) if s.needs_run() => false,
            Some(_) => !names_run,
            None => !names_run && self.capture.is_none(),
        }
    }

    /// Parses the process arguments; uses defaults where there is no command line.
    pub fn from_env() -> Self {
        if cfg!(target_arch = "wasm32") {
            Self::parse_from(["warlock"])
        } else {
            Self::parse()
        }
    }
}

fn parse_size(s: &str) -> Result<UVec2, String> {
    let (w, h) = s
        .split_once('x')
        .ok_or_else(|| format!("expected WIDTHxHEIGHT, got `{s}`"))?;
    let w: u32 = w.parse().map_err(|e| format!("bad width `{w}`: {e}"))?;
    let h: u32 = h.parse().map_err(|e| format!("bad height `{h}`: {e}"))?;
    if w == 0 || h == 0 {
        return Err("window size must be non-zero".into());
    }
    Ok(UVec2::new(w, h))
}

/// A level cell written `X,Y` (`56,13`).
/// Keys for `--keys`: single characters, or the names [`KEY_NAMES`] has.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keys(pub Vec<String>);

/// The keys `--keys` takes by name.
pub const KEY_NAMES: [&str; 7] = ["Enter", "Esc", "Tab", "Up", "Down", "Left", "Right"];

fn parse_keys(s: &str) -> Result<Keys, String> {
    let keys: Vec<String> = s.split_whitespace().map(String::from).collect();
    match keys
        .iter()
        .find(|k| k.chars().count() != 1 && !KEY_NAMES.contains(&k.as_str()))
    {
        Some(bad) => Err(format!(
            "`{bad}`: a key is one character or one of {}",
            KEY_NAMES.join(", ")
        )),
        None if keys.is_empty() => Err("no keys".into()),
        None => Ok(Keys(keys)),
    }
}

fn parse_cell(s: &str) -> Result<warlock_core::Pos, String> {
    let (x, y) = s
        .split_once(',')
        .ok_or_else(|| format!("expected X,Y, got `{s}`"))?;
    let coord = |v: &str| {
        v.trim()
            .parse::<i32>()
            .map_err(|e| format!("bad `{v}`: {e}"))
    };
    Ok(warlock_core::Pos::new(coord(x)?, coord(y)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_picks_the_menu_or_a_run() {
        let menu = |args: &[&str]| Args::parse_from([&["warlock"], args].concat()).starts_in_menu();
        assert!(menu(&[]));
        assert!(!menu(&["--capture", "x.png"]));
        assert!(!menu(&["--scenario", "showcase"]));
        assert!(menu(&["--screen", "new-run", "--capture", "x.png"]));
        assert!(!menu(&["--screen", "pause"]));
        assert!(!menu(&["--screen", "help", "--capture", "x.png"]));
        assert!(menu(&["--screen", "controls", "--capture", "x.png"]));
        assert!(!menu(&["--screen", "settings", "--replay", "r.json"]));
        assert!(!menu(&["--screen", "inventory", "--capture", "x.png"]));
        assert!(!menu(&["--screen", "pick-up"]));
        for creation in [
            "race",
            "sign",
            "gender",
            "class",
            "name",
            "summary",
            "characters",
        ] {
            assert!(menu(&["--screen", creation, "--capture", "x.png"]));
            assert!(menu(&["--screen", creation, "--replay", "r.json"]));
        }
    }

    #[test]
    fn keys_are_characters_or_names() {
        let args = Args::parse_from(["warlock", "--keys", "d b Enter"]);
        let keys = ["d", "b", "Enter"].map(String::from).to_vec();
        assert_eq!(args.keys, Some(Keys(keys)));
        assert!(Args::try_parse_from(["warlock", "--keys", "Return"]).is_err());
        assert!(Args::try_parse_from(["warlock", "--keys", " "]).is_err());
    }

    #[test]
    fn characters_are_race_gender_class() {
        let args = Args::parse_from(["warlock", "--character", "dwarf,female,warrior"]);
        assert_eq!(
            args.character.map(|c| c.to_string()).as_deref(),
            Some("dwarf,female,warrior")
        );
        assert!(Args::try_parse_from(["warlock", "--character", "dwarf"]).is_err());
        // Like `--seed`, it shapes a run that other flags name; alone it opens the menu.
        assert!(Args::parse_from(["warlock", "--character", "elf,male,bard"]).starts_in_menu());
        let named = [
            "warlock",
            "--scenario",
            "depths",
            "--character",
            "elf,male,bard",
        ];
        assert!(!Args::parse_from(named).starts_in_menu());
    }

    #[test]
    fn parses_window_size() {
        assert_eq!(parse_size("640x480"), Ok(UVec2::new(640, 480)));
        assert!(parse_size("640").is_err());
        assert!(parse_size("0x480").is_err());
    }
}
