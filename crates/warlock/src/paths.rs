//! Where the game's files live (spec §7): the one place that decides, so settings, saves,
//! snapshots, and logs always agree, and `--paths` and Settings > Files can say why.
//!
//! Precedence, per location: an explicit `--config-dir` / `--save-dir` (for its own folder),
//! then `--portable [DIR]`, then a `warlock-data` folder next to the game, then the platform's
//! standard folders. A portable folder holds `config/`, `data/` (saves, `snapshots/`), and
//! `logs/`. "Next to the game" is the folder of the executable; on macOS, inside a `Warlock.app`
//! bundle, it is the folder holding the bundle. The web build has browser storage and none of
//! this.
//!
//! [`resolve`] is a pure function of its [`Inputs`] (so it is tested without a filesystem);
//! [`init`] gathers the real inputs once at startup and [`locations`] hands the result out.

use crate::cli::Args;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Name of the folder that makes the game portable when it sits next to the game.
pub const PORTABLE_DIR: &str = "warlock-data";

/// Why a location is where it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    /// An explicit flag for this folder (`--config-dir`, `--save-dir`).
    Flag(&'static str),
    /// `--portable DIR`.
    PortableDir,
    /// `--portable` alone: `warlock-data` next to the game, made if missing.
    PortableFlag,
    /// A `warlock-data` folder was found next to the game.
    PortableFolder,
    /// The platform's standard folder.
    Platform,
    /// No home directory, so nothing is stored.
    Unavailable,
    /// The browser's local storage for this site.
    Browser,
}

impl Reason {
    pub fn describe(&self) -> String {
        match self {
            Reason::Flag(flag) => format!("set by {flag}"),
            Reason::PortableDir => "portable: set by --portable DIR".into(),
            Reason::PortableFlag => {
                format!("portable: --portable, {PORTABLE_DIR} next to the game")
            }
            Reason::PortableFolder => {
                format!("portable: a {PORTABLE_DIR} folder is next to the game")
            }
            Reason::Platform => "the platform's standard folder".into(),
            Reason::Unavailable => "no home directory was found: nothing is stored".into(),
            Reason::Browser => "the browser's storage for this site".into(),
        }
    }

    fn is_portable(&self) -> bool {
        matches!(
            self,
            Reason::PortableDir | Reason::PortableFlag | Reason::PortableFolder
        )
    }
}

/// One folder and why it is there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    /// `None` where nothing is stored (the web, no home directory).
    pub dir: Option<PathBuf>,
    pub reason: Reason,
}

/// The platform's standard folders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Platform {
    pub config: PathBuf,
    pub data: PathBuf,
    pub logs: PathBuf,
}

/// Everything [`resolve`] needs to know.
#[derive(Debug, Clone, Default)]
pub struct Inputs {
    pub config_dir: Option<PathBuf>,
    pub save_dir: Option<PathBuf>,
    /// `--portable` (`Some(None)`) or `--portable DIR` (`Some(Some(dir))`).
    pub portable: Option<Option<PathBuf>>,
    /// The folder `warlock-data` is looked for in: [`game_folder`] of the executable.
    pub game_folder: Option<PathBuf>,
    pub platform: Option<Platform>,
    pub web: bool,
}

/// One line of `--paths` and of Settings > Files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub label: &'static str,
    /// What is in the folder.
    pub what: &'static str,
    pub dir: Option<PathBuf>,
    pub reason: Reason,
}

/// The resolved locations for this launch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locations {
    /// The portable folder, when running portable.
    pub portable_root: Option<PathBuf>,
    pub config: Place,
    pub data: Place,
    pub logs: Place,
    /// Problems the player should know of (an unwritable portable folder).
    pub warnings: Vec<String>,
}

/// The folder `warlock-data` belongs in, for the executable at `exe`: its own folder, or, for the
/// executable inside a macOS `Something.app/Contents/MacOS/`, the folder holding the bundle.
pub fn game_folder(exe: &Path) -> Option<PathBuf> {
    let dir = exe.parent()?;
    let bundle = (dir.file_name().is_some_and(|n| n == "MacOS"))
        .then(|| dir.parent())
        .flatten()
        .filter(|contents| contents.file_name().is_some_and(|n| n == "Contents"))
        .and_then(Path::parent)
        .filter(|app| app.extension().is_some_and(|e| e == "app"));
    match bundle {
        Some(app) => app.parent().map(Path::to_path_buf),
        None => Some(dir.to_path_buf()),
    }
}

/// Decides every location. `is_dir` says whether a folder exists (so no filesystem is needed).
pub fn resolve(inputs: &Inputs, is_dir: &dyn Fn(&Path) -> bool) -> Locations {
    let mut warnings = Vec::new();
    let beside = inputs.game_folder.as_ref().map(|d| d.join(PORTABLE_DIR));
    let portable = match &inputs.portable {
        _ if inputs.web => None,
        Some(Some(dir)) => Some((dir.clone(), Reason::PortableDir)),
        Some(None) => match beside {
            Some(dir) => Some((dir, Reason::PortableFlag)),
            None => {
                warnings.push(format!(
                    "--portable: cannot tell where the game is, so no {PORTABLE_DIR} folder was used"
                ));
                None
            }
        },
        None => beside
            .filter(|dir| is_dir(dir))
            .map(|dir| (dir, Reason::PortableFolder)),
    };
    let pick = |flag: Option<(&'static str, &PathBuf)>, sub: &str, std: Option<&PathBuf>| {
        if inputs.web {
            Place {
                dir: None,
                reason: Reason::Browser,
            }
        } else if let Some((name, dir)) = flag {
            Place {
                dir: Some(dir.clone()),
                reason: Reason::Flag(name),
            }
        } else if let Some((root, why)) = &portable {
            Place {
                dir: Some(root.join(sub)),
                reason: why.clone(),
            }
        } else if let Some(dir) = std {
            Place {
                dir: Some(dir.clone()),
                reason: Reason::Platform,
            }
        } else {
            Place {
                dir: None,
                reason: Reason::Unavailable,
            }
        }
    };
    let platform = inputs.platform.as_ref();
    Locations {
        config: pick(
            inputs.config_dir.as_ref().map(|d| ("--config-dir", d)),
            "config",
            platform.map(|p| &p.config),
        ),
        data: pick(
            inputs.save_dir.as_ref().map(|d| ("--save-dir", d)),
            "data",
            platform.map(|p| &p.data),
        ),
        logs: pick(None, "logs", platform.map(|p| &p.logs)),
        portable_root: portable.map(|(root, _)| root),
        warnings,
    }
}

impl Locations {
    /// The directory holding key binding files (`keymaps/` beside the settings).
    #[allow(dead_code)] // For the keymap files (#159); the bindings live in settings.toml today.
    pub fn keymaps_dir(&self) -> Option<PathBuf> {
        self.config.dir.as_ref().map(|d| d.join("keymaps"))
    }

    /// Folder for the dev snapshots.
    pub fn snapshots_dir(&self) -> Option<PathBuf> {
        self.data.dir.as_ref().map(|d| d.join("snapshots"))
    }

    /// The entries to show, in order: settings, saves, snapshots, logs.
    pub fn entries(&self) -> Vec<Entry> {
        let entry = |label, what, dir: Option<PathBuf>, reason: &Reason| Entry {
            label,
            what,
            dir,
            reason: reason.clone(),
        };
        vec![
            entry(
                "Settings",
                "settings.toml, with the key bindings",
                self.config.dir.clone(),
                &self.config.reason,
            ),
            entry("Saves", "run.sav", self.data.dir.clone(), &self.data.reason),
            entry(
                "Snapshots",
                "dev snapshots (F5/F9)",
                self.snapshots_dir(),
                &self.data.reason,
            ),
            entry(
                "Logs",
                "logs and crash reports",
                self.logs.dir.clone(),
                &self.logs.reason,
            ),
        ]
    }

    /// The headline: portable or not, and where.
    pub fn mode_line(&self) -> String {
        match &self.portable_root {
            Some(root) => format!("Portable: everything is in {}", root.display()),
            None if self.config.reason == Reason::Browser => {
                "Everything is in your browser's storage for this site.".into()
            }
            None => format!(
                "Not portable: the platform's folders. Make a {PORTABLE_DIR} folder next to the \
                 game to carry everything with it."
            ),
        }
    }

    /// The text of `warlock --paths`.
    pub fn report(&self) -> String {
        let mut out = format!("{}\n", self.mode_line());
        for e in self.entries() {
            let place = match &e.dir {
                Some(dir) => dir.display().to_string(),
                None => "(not stored)".into(),
            };
            out.push_str(&format!(
                "\n{}: {}\n  {} ({})\n",
                e.label,
                place,
                e.what,
                e.reason.describe()
            ));
        }
        for w in &self.warnings {
            out.push_str(&format!("\nwarning: {w}\n"));
        }
        out
    }

    /// Makes the portable folders (only those a portable location uses) and checks that they can
    /// be written. A problem is a warning, never fatal: the folders are still used, so what is
    /// there can be read, and writes report their own errors.
    pub fn prepare(&mut self) {
        let Some(root) = self.portable_root.clone() else {
            return;
        };
        for place in [&self.config, &self.data, &self.logs] {
            let Some(dir) = place.dir.as_ref().filter(|_| place.reason.is_portable()) else {
                continue;
            };
            if let Err(e) = std::fs::create_dir_all(dir).and_then(|()| probe(dir)) {
                self.warnings.push(format!(
                    "The portable folder {} cannot be written ({e}). Settings and saves there are \
                     read but not saved this session.",
                    root.display()
                ));
                break;
            }
        }
    }
}

/// Fails if `dir` cannot be written to.
fn probe(dir: &Path) -> std::io::Result<()> {
    let file = dir.join(".write-test");
    std::fs::write(&file, b"")?;
    std::fs::remove_file(file)
}

static LOCATIONS: OnceLock<Locations> = OnceLock::new();

/// Resolves the locations for this launch (the command line, the executable's place, the
/// platform), prepares portable folders, and keeps the result for [`locations`]. Call once,
/// first thing in `main`. With `check_only`, nothing is created (for `--paths`).
pub fn init(args: &Args, check_only: bool) -> &'static Locations {
    let mut found = resolve(&Inputs::from_args(args), &|p| p.is_dir());
    if !check_only {
        found.prepare();
    }
    LOCATIONS.get_or_init(|| found)
}

/// The locations of this launch. Without [`init`] (tests), the standard ones.
pub fn locations() -> &'static Locations {
    LOCATIONS.get_or_init(|| resolve(&Inputs::standard(), &|_| false))
}

impl Inputs {
    /// Platform folders only: no flags, no portable folder.
    pub fn standard() -> Self {
        Self {
            platform: platform(),
            web: cfg!(target_arch = "wasm32"),
            ..Self::default()
        }
    }

    pub fn from_args(args: &Args) -> Self {
        let absolute = |p: &PathBuf| std::path::absolute(p).unwrap_or_else(|_| p.clone());
        Self {
            config_dir: args.config_dir.as_ref().map(absolute),
            save_dir: args.save_dir.as_ref().map(absolute),
            portable: args.portable.as_ref().map(|dir| dir.as_ref().map(absolute)),
            game_folder: exe_game_folder(),
            ..Self::standard()
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn exe_game_folder() -> Option<PathBuf> {
    game_folder(&std::env::current_exe().ok()?)
}

#[cfg(target_arch = "wasm32")]
pub fn exe_game_folder() -> Option<PathBuf> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn platform() -> Option<Platform> {
    let dirs = directories::ProjectDirs::from("", "", "warlock")?;
    Some(Platform {
        config: dirs.config_dir().to_path_buf(),
        data: dirs.data_dir().to_path_buf(),
        logs: dirs.data_local_dir().join("logs"),
    })
}

#[cfg(target_arch = "wasm32")]
fn platform() -> Option<Platform> {
    None
}

/// Opens `dir` in the platform's file manager (creating it first if it does not exist yet).
/// The error is a message for the player.
#[cfg(not(target_arch = "wasm32"))]
pub fn open_folder(dir: &Path) -> Result<(), String> {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(windows) {
        "explorer"
    } else {
        "xdg-open"
    };
    std::fs::create_dir_all(dir).map_err(|e| format!("Cannot make {}: {e}.", dir.display()))?;
    let mut child = std::process::Command::new(opener)
        .arg(dir)
        .spawn()
        .map_err(|e| format!("Cannot run {opener}: {e}."))?;
    // Reap it in the background so it never lingers as a zombie.
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn open_folder(_dir: &Path) -> Result<(), String> {
    Err("There are no folders in the browser.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn platform() -> Platform {
        Platform {
            config: "/p/config".into(),
            data: "/p/data".into(),
            logs: "/p/logs".into(),
        }
    }

    fn base() -> Inputs {
        Inputs {
            game_folder: Some("/game".into()),
            platform: Some(platform()),
            ..Inputs::default()
        }
    }

    fn dirs(of: &'static [&'static str]) -> impl Fn(&Path) -> bool {
        move |p| of.iter().any(|d| Path::new(d) == p)
    }

    fn dir(place: &Place) -> &str {
        place.dir.as_deref().and_then(Path::to_str).unwrap()
    }

    #[test]
    fn platform_folders_by_default() {
        let l = resolve(&base(), &dirs(&[]));
        assert_eq!(
            (dir(&l.config), dir(&l.data), dir(&l.logs)),
            ("/p/config", "/p/data", "/p/logs")
        );
        assert_eq!(l.config.reason, Reason::Platform);
        assert_eq!(l.portable_root, None);
        assert!(l.mode_line().starts_with("Not portable"));
    }

    #[test]
    fn a_folder_next_to_the_game_makes_it_portable() {
        let l = resolve(&base(), &dirs(&["/game/warlock-data"]));
        assert_eq!(
            (dir(&l.config), dir(&l.data), dir(&l.logs)),
            (
                "/game/warlock-data/config",
                "/game/warlock-data/data",
                "/game/warlock-data/logs"
            )
        );
        assert_eq!(l.data.reason, Reason::PortableFolder);
        assert_eq!(l.portable_root, Some("/game/warlock-data".into()));
        assert_eq!(
            l.snapshots_dir().unwrap(),
            Path::new("/game/warlock-data/data/snapshots")
        );
        assert_eq!(
            l.keymaps_dir().unwrap(),
            Path::new("/game/warlock-data/config/keymaps")
        );
    }

    #[test]
    fn portable_flag_makes_the_folder_even_when_missing() {
        let inputs = Inputs {
            portable: Some(None),
            ..base()
        };
        let l = resolve(&inputs, &dirs(&[]));
        assert_eq!(dir(&l.data), "/game/warlock-data/data");
        assert_eq!(l.logs.reason, Reason::PortableFlag);
    }

    #[test]
    fn portable_dir_beats_a_folder_next_to_the_game() {
        let inputs = Inputs {
            portable: Some(Some("/usb/w".into())),
            ..base()
        };
        let l = resolve(&inputs, &dirs(&["/game/warlock-data"]));
        assert_eq!(dir(&l.config), "/usb/w/config");
        assert_eq!(l.config.reason, Reason::PortableDir);
    }

    #[test]
    fn explicit_flags_win_for_their_own_folder_only() {
        let inputs = Inputs {
            config_dir: Some("/c".into()),
            save_dir: Some("/s".into()),
            portable: Some(Some("/usb/w".into())),
            ..base()
        };
        let l = resolve(&inputs, &dirs(&[]));
        assert_eq!((dir(&l.config), dir(&l.data)), ("/c", "/s"));
        assert_eq!(l.config.reason, Reason::Flag("--config-dir"));
        assert_eq!(l.data.reason, Reason::Flag("--save-dir"));
        assert_eq!(dir(&l.logs), "/usb/w/logs");
        // And over a platform folder, without portable mode.
        let inputs = Inputs {
            save_dir: Some("/s".into()),
            ..base()
        };
        let l = resolve(&inputs, &dirs(&[]));
        assert_eq!((dir(&l.config), dir(&l.data)), ("/p/config", "/s"));
    }

    #[test]
    fn no_game_folder_means_no_automatic_portable_mode() {
        let inputs = Inputs {
            game_folder: None,
            ..base()
        };
        assert_eq!(resolve(&inputs, &|_| true).portable_root, None);
        let inputs = Inputs {
            portable: Some(None),
            ..inputs
        };
        let l = resolve(&inputs, &|_| true);
        assert_eq!(l.portable_root, None);
        assert_eq!(l.warnings.len(), 1);
    }

    #[test]
    fn no_home_directory_stores_nothing() {
        let inputs = Inputs {
            platform: None,
            ..base()
        };
        let l = resolve(&inputs, &dirs(&[]));
        assert_eq!(l.config.dir, None);
        assert_eq!(l.data.reason, Reason::Unavailable);
        assert!(l.report().contains("(not stored)"));
    }

    #[test]
    fn the_web_has_browser_storage_only() {
        let inputs = Inputs {
            web: true,
            portable: Some(None),
            ..base()
        };
        let l = resolve(&inputs, &|_| true);
        assert_eq!(l.config.dir, None);
        assert_eq!(l.data.reason, Reason::Browser);
        assert!(l.mode_line().contains("browser"));
    }

    #[test]
    fn game_folder_is_beside_the_binary_or_the_mac_bundle() {
        let folder = |p: &str| game_folder(Path::new(p)).unwrap();
        assert_eq!(folder("/g/warlock"), Path::new("/g"));
        assert_eq!(
            folder("/g/Warlock.app/Contents/MacOS/warlock"),
            Path::new("/g")
        );
        // Only a real bundle counts.
        assert_eq!(
            folder("/g/Other/Contents/MacOS/warlock"),
            Path::new("/g/Other/Contents/MacOS")
        );
        assert_eq!(
            folder("/g/Warlock.app/warlock"),
            Path::new("/g/Warlock.app")
        );
    }

    #[test]
    fn report_names_each_location_and_why() {
        let l = resolve(&base(), &dirs(&["/game/warlock-data"]));
        let text = l.report();
        for want in [
            "Portable: everything is in /game/warlock-data",
            "Settings: /game/warlock-data/config",
            "Saves: /game/warlock-data/data",
            "Snapshots: /game/warlock-data/data/snapshots",
            "Logs: /game/warlock-data/logs",
            "a warlock-data folder is next to the game",
        ] {
            assert!(text.contains(want), "{want:?} missing from:\n{text}");
        }
    }

    #[test]
    fn prepare_makes_the_portable_layout_and_warns_when_unwritable() {
        let tmp = std::env::temp_dir().join(format!("warlock-paths-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let inputs = Inputs {
            portable: Some(Some(tmp.join("w"))),
            ..base()
        };
        let mut l = resolve(&inputs, &|p| p.is_dir());
        l.prepare();
        assert!(l.warnings.is_empty(), "{:?}", l.warnings);
        for sub in ["config", "data", "logs"] {
            assert!(tmp.join("w").join(sub).is_dir());
        }
        // A file where the folder should be cannot be a folder: a warning, no panic.
        let blocker = tmp.join("file");
        std::fs::write(&blocker, b"x").unwrap();
        let inputs = Inputs {
            portable: Some(Some(blocker.join("w"))),
            ..base()
        };
        let mut l = resolve(&inputs, &|p| p.is_dir());
        l.prepare();
        assert_eq!(l.warnings.len(), 1);
        assert!(l.report().contains("cannot be written"));
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
