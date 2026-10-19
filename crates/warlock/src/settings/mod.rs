//! Player settings (spec §4.7, §6): font size, reduced effects, color palette, key bindings.
//!
//! They live in `settings.toml` in the platform config directory, or under the key
//! `warlock/settings.toml` in the browser's local storage on the web (`storage.rs`). A missing or damaged file means defaults
//! plus a logged warning, never a crash. Every change is applied live and written back at once.
//! Captures and replays use defaults and never touch the file unless `--config-dir` is given.
//!
//! ```toml
//! font_px = 18            # logical pixels, 12 to 36
//! reduced_effects = false # preset: forces every effect below off
//! palette = "default"     # "default", "high-contrast", "colorblind"
//!
//! [effects]               # per-effect switches, all on by default
//! glyph_animation = true  # wave, pulse, shimmer, cycle; heard sounds ripple in
//! flicker = true          # light flicker: firelight varies in intensity and hue
//! motion = true           # movement slides and camera easing
//! glow = true             # bloom and glyph glow
//! distortion = true       # heat haze
//! particles = true
//! screen_fades = true     # level-change fade, flashes, and spell effects darkening the view
//! screen_shake = true     # the view shakes when a spell effect lands near the player
//! weather = true          # rain falling, splashing, and rings on water
//! blood = true            # blood stains; not part of reduced effects
//!
//! [bindings]              # command id -> keys; a character, or a key name like "ArrowUp"
//! move_n = ["k", "Numpad8", "ArrowUp"]
//! ```

pub mod keys;
pub mod palette;

use crate::cli::Args;
use crate::input::Bindings;
use crate::storage::Backend;
use bevy::prelude::*;
pub use palette::{Palette, Role};

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
    /// Per-effect switches; an effect runs only when its switch is on and reduced effects is off.
    pub effects: EffectSwitches,
    pub palette: Palette,
}

/// Which visual effects run (spec §4.7). In [`Settings`] these are the player's per-effect
/// switches; the `Effects` resource in rendering holds the effective values.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectSwitches {
    /// Glyph animation: wave, pulse, shimmer, cycle, and heard-sound markers rippling in (the
    /// markers themselves stay: they are information).
    pub glyph_animation: bool,
    /// Light flicker: per-source modulation of intensity and hue (light stability).
    pub flicker: bool,
    /// Movement slides and camera easing.
    pub motion: bool,
    /// Bloom and glyph glow.
    pub glow: bool,
    /// Heat haze.
    pub distortion: bool,
    pub particles: bool,
    /// The level-change fade, screen flashes, and bright spell effects darkening the rest of the
    /// view while they peak.
    pub screen_fades: bool,
    /// The view shaking when a spell effect (a trap's fireball) lands near the player.
    pub screen_shake: bool,
    /// Weather drawn over the map: rain falling, splashing, and rings on still water. A switch of
    /// its own (not `particles`): it moves over the whole view, not around a few sources. The
    /// weather itself (its light, the status bar) does not depend on it.
    pub weather: bool,
    /// Blood stains on floors and walls (spec §8 "Blood"). A content preference rather than an
    /// effect: reduced effects leaves it as it is.
    pub blood: bool,
}

impl EffectSwitches {
    pub const ALL_ON: Self = Self {
        glyph_animation: true,
        flicker: true,
        motion: true,
        glow: true,
        distortion: true,
        particles: true,
        screen_fades: true,
        screen_shake: true,
        weather: true,
        blood: true,
    };
    pub const ALL_OFF: Self = Self {
        glyph_animation: false,
        flicker: false,
        motion: false,
        glow: false,
        distortion: false,
        particles: false,
        screen_fades: false,
        screen_shake: false,
        weather: false,
        blood: false,
    };

    /// The effective switches: each as set, and when `reduced` every effect off; blood stays
    /// as set (it is a content preference, not an effect).
    pub fn effective(self, reduced: bool) -> Self {
        if reduced {
            Self {
                blood: self.blood,
                ..Self::ALL_OFF
            }
        } else {
            self
        }
    }

    /// The setting names, for the file.
    const KEYS: [&'static str; 10] = [
        "glyph_animation",
        "flicker",
        "motion",
        "glow",
        "distortion",
        "particles",
        "screen_fades",
        "screen_shake",
        "weather",
        "blood",
    ];

    fn to_map(self) -> std::collections::BTreeMap<&'static str, bool> {
        let mut copy = self;
        Self::KEYS
            .iter()
            .map(|&k| (k, *copy.slot(k).expect("known key")))
            .collect()
    }

    fn slot(&mut self, key: &str) -> Option<&mut bool> {
        Some(match key {
            "glyph_animation" => &mut self.glyph_animation,
            "flicker" => &mut self.flicker,
            "motion" => &mut self.motion,
            "glow" => &mut self.glow,
            "distortion" => &mut self.distortion,
            "particles" => &mut self.particles,
            "screen_fades" => &mut self.screen_fades,
            "screen_shake" => &mut self.screen_shake,
            "weather" => &mut self.weather,
            "blood" => &mut self.blood,
            _ => return None,
        })
    }
}

impl Default for EffectSwitches {
    fn default() -> Self {
        Self::ALL_ON
    }
}

impl Settings {
    /// The effects that actually run; `force_reduced` is the `--reduced-effects` flag.
    pub fn effective_effects(&self, force_reduced: bool) -> EffectSwitches {
        self.effects
            .effective(force_reduced || self.reduced_effects)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            font_px: DEFAULT_FONT_PX,
            reduced_effects: false,
            effects: EffectSwitches::ALL_ON,
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
        if ![
            "font_px",
            "reduced_effects",
            "effects",
            "palette",
            "bindings",
        ]
        .contains(&key.as_str())
        {
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
    match table.get("effects") {
        Some(toml::Value::Table(t)) => {
            for (key, value) in t {
                match settings.effects.slot(key) {
                    None => warnings.push(format!("unknown effect `{key}` ignored")),
                    Some(slot) => match value.as_bool() {
                        Some(on) => *slot = on,
                        None => warnings.push(format!("effects.{key} must be true or false")),
                    },
                }
            }
        }
        Some(_) => warnings.push("effects must be a table; using the defaults".into()),
        None => {}
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
    effects: std::collections::BTreeMap<&'static str, bool>,
    bindings: std::collections::BTreeMap<String, Vec<String>>,
}

/// The `settings.toml` text for these settings.
pub fn to_toml(settings: &Settings, bindings: &Bindings) -> String {
    let out = FileOut {
        font_px: settings.font_px,
        reduced_effects: settings.reduced_effects,
        palette: settings.palette.id(),
        effects: settings.effects.to_map(),
        bindings: bindings.to_map(),
    };
    toml::to_string_pretty(&out).unwrap_or_default()
}

/// Where the settings file lives; `None` means settings are not stored.
#[derive(Resource, Debug, Clone, Default)]
pub struct SettingsStore {
    backend: Option<Backend>,
}

impl SettingsStore {
    /// The store for this run's command line.
    pub fn from_args(args: &Args) -> Self {
        let backend = match &args.config_dir {
            Some(dir) => Some(Backend::Dir(dir.clone())),
            // Captures and replays are reproducible dev tools: no player settings in or out.
            None if args.capture.is_some() || args.replay.is_some() => None,
            None => default_backend(),
        };
        Self { backend }
    }

    #[cfg(test)]
    pub fn at(dir: std::path::PathBuf) -> Self {
        Self {
            backend: Some(Backend::Dir(dir)),
        }
    }

    /// Reads the file. Missing means defaults, silently; unreadable or damaged means defaults
    /// with a warning.
    pub fn load(&self) -> Loaded {
        let Some(backend) = &self.backend else {
            return Loaded::defaults(Vec::new());
        };
        let place = backend.describe(FILE_NAME);
        match backend.read(FILE_NAME) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(text) => {
                    let mut loaded = parse(&text);
                    for w in &mut loaded.warnings {
                        *w = format!("{place}: {w}");
                    }
                    loaded
                }
                Err(_) => Loaded::defaults(vec![format!(
                    "cannot read {place}: not UTF-8; using defaults"
                )]),
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Loaded::defaults(Vec::new()),
            Err(e) => Loaded::defaults(vec![format!("cannot read {place}: {e}; using defaults")]),
        }
    }

    /// Writes the file atomically. A no-op where settings are not stored.
    pub fn save(&self, settings: &Settings, bindings: &Bindings) -> Result<(), String> {
        let Some(backend) = &self.backend else {
            return Ok(());
        };
        backend
            .write(FILE_NAME, to_toml(settings, bindings).as_bytes())
            .map_err(|e| format!("cannot write {}: {e}", backend.describe(FILE_NAME)))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn default_backend() -> Option<Backend> {
    crate::paths::project_dirs().map(|d| Backend::Dir(d.config_dir().to_path_buf()))
}

/// Browser storage; without it (some privacy modes) settings last for the session only.
#[cfg(target_arch = "wasm32")]
fn default_backend() -> Option<Backend> {
    crate::storage::web::available().then_some(Backend::Browser)
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
    use std::path::PathBuf;

    fn temp_store(name: &str) -> SettingsStore {
        let dir =
            std::env::temp_dir().join(format!("warlock-settings-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        SettingsStore::at(dir)
    }

    #[test]
    fn settings_round_trip_through_disk() {
        let store = temp_store("roundtrip");
        let settings = Settings {
            font_px: 24,
            reduced_effects: true,
            effects: EffectSwitches {
                particles: false,
                flicker: false,
                screen_shake: false,
                weather: false,
                blood: false,
                ..EffectSwitches::ALL_ON
            },
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
    fn old_files_without_effects_load_with_everything_on() {
        let loaded = parse("font_px = 20\nreduced_effects = false\npalette = \"default\"\n");
        assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
        assert_eq!(loaded.settings.effects, EffectSwitches::ALL_ON);
    }

    #[test]
    fn bad_effect_values_fall_back_one_by_one() {
        let loaded =
            parse("[effects]\nglow = false\nparticles = \"no\"\nsparkles = true\nmotion = false\n");
        let e = loaded.settings.effects;
        assert!(!e.glow && !e.motion);
        assert!(e.particles && e.flicker && e.distortion);
        assert_eq!(loaded.warnings.len(), 2, "{:?}", loaded.warnings);
        let loaded = parse("effects = 3\n");
        assert_eq!(loaded.settings.effects, EffectSwitches::ALL_ON);
        assert_eq!(loaded.warnings.len(), 1);
    }

    #[test]
    fn effective_effects_combine_switches_and_the_preset() {
        let mut s = Settings::default();
        s.effects.particles = false;
        let e = s.effective_effects(false);
        assert!(!e.particles && e.glow && e.flicker);
        // Reduced effects turns every effect off, but blood stays as set.
        let reduced = EffectSwitches {
            blood: true,
            ..EffectSwitches::ALL_OFF
        };
        s.reduced_effects = true;
        assert_eq!(s.effective_effects(false), reduced);
        s.reduced_effects = false;
        assert_eq!(s.effective_effects(true), reduced);
        s.effects.blood = false;
        assert!(!s.effective_effects(true).blood && !s.effective_effects(false).blood);
        // Turning the preset off again restores the individual switches.
        assert!(s.effective_effects(false).glow);
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
        let backend = store.backend.clone().expect("backend");
        backend.write(FILE_NAME, b"font_px = [[[").expect("write");
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
        assert!(SettingsStore::from_args(&args).backend.is_none());
        let args = Args::parse_from(["warlock", "--capture", "x.png", "--config-dir", "cfg"]);
        assert_eq!(
            SettingsStore::from_args(&args).backend,
            Some(Backend::Dir(PathBuf::from("cfg")))
        );
    }
}
