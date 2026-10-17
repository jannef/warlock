//! Player settings (spec §4.7, §6): font size, reduced effects, color palette, key bindings.
//!
//! They live in `settings.toml` in the platform config directory (never on the web, which has
//! no filesystem; settings then last for the session). A missing or damaged file means defaults
//! plus a logged warning, never a crash. Every change is applied live and written back at once.
//! Captures and replays use defaults and never touch the file unless `--config-dir` is given.
//!
//! ```toml
//! font_px = 18            # logical pixels, 12 to 36
//! reduced_effects = false
//! palette = "default"     # "default", "high-contrast", "colorblind"
//!
//! [bindings]              # command id -> keys; a character, or a key name like "ArrowUp"
//! move_n = ["k", "Numpad8", "ArrowUp"]
//! ```

pub mod keys;
pub mod palette;

use crate::cli::Args;
use crate::input::Bindings;
use bevy::prelude::*;
pub use palette::{Palette, Role};
use std::io::Write;
use std::path::{Path, PathBuf};

pub const MIN_FONT_PX: u32 = 12;
pub const MAX_FONT_PX: u32 = 36;
pub const DEFAULT_FONT_PX: u32 = 18;

const FILE_NAME: &str = "settings.toml";

/// The display and accessibility settings. Key bindings are the [`Bindings`] resource.
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Settings {
    /// Font size in logical pixels.
    pub font_px: u32,
    /// Reduced effects (the `--reduced-effects` flag forces this on regardless).
    pub reduced_effects: bool,
    pub palette: Palette,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            font_px: DEFAULT_FONT_PX,
            reduced_effects: false,
            palette: Palette::Default,
        }
    }
}

/// What reading the settings file produced.
#[derive(Debug)]
pub struct Loaded {
    pub settings: Settings,
    pub bindings: Bindings,
    /// Problems found, each worth a log line; the rest of the file was still used.
    pub warnings: Vec<String>,
}

impl Loaded {
    fn defaults(warnings: Vec<String>) -> Self {
        Self {
            settings: Settings::default(),
            bindings: Bindings::default(),
            warnings,
        }
    }
}

/// Parses `settings.toml` text. Each setting is read on its own, so one bad value does not
/// cost the others.
pub fn parse(text: &str) -> Loaded {
    let table: toml::Table = match toml::from_str(text) {
        Ok(table) => table,
        Err(err) => {
            let message = format!("settings file is not valid TOML ({err}); using defaults");
            return Loaded::defaults(vec![message]);
        }
    };
    let mut warnings = Vec::new();
    let mut settings = Settings::default();
    for key in table.keys() {
        if !["font_px", "reduced_effects", "palette", "bindings"].contains(&key.as_str()) {
            warnings.push(format!("unknown setting `{key}` ignored"));
        }
    }
    if let Some(value) = table.get("font_px") {
        match value
            .as_integer()
            .or_else(|| value.as_float().map(|f| f.round() as i64))
        {
            Some(px) => {
                let clamped = px.clamp(MIN_FONT_PX.into(), MAX_FONT_PX.into()) as u32;
                if i64::from(clamped) != px {
                    warnings.push(format!("font_px {px} out of range; using {clamped}"));
                }
                settings.font_px = clamped;
            }
            None => warnings.push("font_px must be a number; using the default".into()),
        }
    }
    if let Some(value) = table.get("reduced_effects") {
        match value.as_bool() {
            Some(on) => settings.reduced_effects = on,
            None => warnings.push("reduced_effects must be true or false".into()),
        }
    }
    if let Some(value) = table.get("palette") {
        match value.as_str().and_then(Palette::from_id) {
            Some(palette) => settings.palette = palette,
            None => warnings.push(format!("unknown palette {value}; using the default")),
        }
    }
    let bindings = match table.get("bindings") {
        Some(toml::Value::Table(t)) => Bindings::from_table(t, &mut warnings),
        Some(_) => {
            warnings.push("bindings must be a table; using the default keys".into());
            Bindings::default()
        }
        None => Bindings::default(),
    };
    Loaded {
        settings,
        bindings,
        warnings,
    }
}

#[derive(serde::Serialize)]
struct FileOut<'a> {
    font_px: u32,
    reduced_effects: bool,
    palette: &'a str,
    bindings: std::collections::BTreeMap<String, Vec<String>>,
}

/// The `settings.toml` text for these settings.
pub fn to_toml(settings: &Settings, bindings: &Bindings) -> String {
    let out = FileOut {
        font_px: settings.font_px,
        reduced_effects: settings.reduced_effects,
        palette: settings.palette.id(),
        bindings: bindings.to_map(),
    };
    toml::to_string_pretty(&out).unwrap_or_default()
}

/// Where the settings file lives; `None` means settings are not stored.
#[derive(Resource, Debug, Clone, Default)]
pub struct SettingsStore {
    path: Option<PathBuf>,
}

impl SettingsStore {
    /// The store for this run's command line.
    pub fn from_args(args: &Args) -> Self {
        let dir = match &args.config_dir {
            Some(dir) => Some(dir.clone()),
            // Captures and replays are reproducible dev tools: no player settings in or out.
            None if args.capture.is_some() || args.replay.is_some() => None,
            None => default_dir(),
        };
        Self {
            path: dir.map(|d| d.join(FILE_NAME)),
        }
    }

    #[cfg(test)]
    pub fn at(path: PathBuf) -> Self {
        Self { path: Some(path) }
    }

    /// Reads the file. Missing means defaults, silently; unreadable or damaged means defaults
    /// with a warning.
    pub fn load(&self) -> Loaded {
        let Some(path) = &self.path else {
            return Loaded::defaults(Vec::new());
        };
        match std::fs::read_to_string(path) {
            Ok(text) => {
                let mut loaded = parse(&text);
                for w in &mut loaded.warnings {
                    *w = format!("{}: {w}", path.display());
                }
                loaded
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Loaded::defaults(Vec::new()),
            Err(e) => Loaded::defaults(vec![format!(
                "cannot read {}: {e}; using defaults",
                path.display()
            )]),
        }
    }

    /// Writes the file atomically. A no-op where settings are not stored.
    pub fn save(&self, settings: &Settings, bindings: &Bindings) -> Result<(), String> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        write_atomic(path, to_toml(settings, bindings).as_bytes())
            .map_err(|e| format!("cannot write {}: {e}", path.display()))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn default_dir() -> Option<PathBuf> {
    crate::paths::project_dirs().map(|d| d.config_dir().to_path_buf())
}

#[cfg(target_arch = "wasm32")]
fn default_dir() -> Option<PathBuf> {
    None
}

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

/// Loads the settings for this run, logging what was wrong with the file (if anything).
pub fn load_for(args: &Args) -> (SettingsStore, Settings, Bindings) {
    let store = SettingsStore::from_args(args);
    let loaded = store.load();
    for warning in &loaded.warnings {
        eprintln!("warlock: warning: {warning}");
    }
    (store, loaded.settings, loaded.bindings)
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Settings>()
            .init_resource::<SettingsStore>()
            .add_systems(Last, persist);
    }
}

/// Writes the settings file after any change (not for the values loaded at startup).
fn persist(
    settings: Res<Settings>,
    bindings: Res<Bindings>,
    store: Res<SettingsStore>,
    mut started: Local<bool>,
) {
    if !*started {
        *started = true;
        return;
    }
    if (settings.is_changed() || bindings.is_changed())
        && let Err(e) = store.save(&settings, &bindings)
    {
        warn!("{e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{KeyBinding, KeyCommand};

    fn temp_store(name: &str) -> SettingsStore {
        let dir =
            std::env::temp_dir().join(format!("warlock-settings-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        SettingsStore::at(dir.join(FILE_NAME))
    }

    #[test]
    fn settings_round_trip_through_disk() {
        let store = temp_store("roundtrip");
        let settings = Settings {
            font_px: 24,
            reduced_effects: true,
            palette: Palette::Colorblind,
        };
        let mut bindings = Bindings::default();
        bindings
            .bind_primary(KeyCommand::Wait, KeyBinding::Char("w".into()))
            .expect("free key");
        store.save(&settings, &bindings).expect("save");
        let loaded = store.load();
        assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
        assert_eq!(loaded.settings, settings);
        assert_eq!(loaded.bindings.to_map(), bindings.to_map());
    }

    #[test]
    fn missing_file_means_defaults_without_warnings() {
        let loaded = temp_store("missing").load();
        assert_eq!(loaded.settings, Settings::default());
        assert!(loaded.warnings.is_empty());
    }

    #[test]
    fn damaged_file_falls_back_to_defaults_with_a_warning() {
        let store = temp_store("damaged");
        let path = store.path.clone().expect("path");
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&path, "font_px = [[[").expect("write");
        let loaded = store.load();
        assert_eq!(loaded.settings, Settings::default());
        assert_eq!(loaded.bindings.0, Bindings::default().0);
        assert_eq!(loaded.warnings.len(), 1);
        assert!(loaded.warnings[0].contains("not valid TOML"));
    }

    #[test]
    fn bad_values_cost_only_themselves() {
        let loaded = parse(
            "font_px = 400\nreduced_effects = \"yes\"\npalette = \"neon\"\nmystery = 1\n\
             [bindings]\nwait = [\"w\"]\n",
        );
        assert_eq!(loaded.settings.font_px, MAX_FONT_PX);
        assert!(!loaded.settings.reduced_effects);
        assert_eq!(loaded.settings.palette, Palette::Default);
        assert_eq!(loaded.warnings.len(), 4, "{:?}", loaded.warnings);
        assert_eq!(
            loaded.bindings.keys_of(KeyCommand::Wait)[0],
            &KeyBinding::Char("w".into())
        );
    }

    #[test]
    fn captures_do_not_read_player_settings() {
        use clap::Parser;
        let args = Args::parse_from(["warlock", "--capture", "x.png"]);
        assert!(SettingsStore::from_args(&args).path.is_none());
        let args = Args::parse_from(["warlock", "--capture", "x.png", "--config-dir", "cfg"]);
        assert_eq!(
            SettingsStore::from_args(&args).path,
            Some(PathBuf::from("cfg").join(FILE_NAME))
        );
    }
}
