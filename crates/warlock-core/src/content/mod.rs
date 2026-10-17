//! Game content: definitions, levels, and scenarios loaded from TOML (spec §7).
//!
//! Content is read from a [`ContentFiles`] set: the built-in set compiled into the binary (so every
//! build, including the web build, is self-contained) or a directory on disk for development.

mod anim;
mod creature;
mod level;
mod light;
mod scenario;
mod terrain;
mod vision;

pub use anim::GlyphAnim;
pub use creature::{CreatureDef, CreatureId};
pub use level::{CreatureSpawn, LevelTemplate, StairsLink, parse_route};
pub use light::{Flicker, LightId, LightPreset, MAX_LIGHT_RADIUS};
pub use scenario::ScenarioDef;
pub use terrain::{StairsDir, TerrainDef, TerrainId};
pub use vision::VisionProfile;

use level::{LegendEntry, legend_char};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;

/// Content files that failed to load, with the offending file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContentError {
    pub file: String,
    pub message: String,
}

impl ContentError {
    fn new(file: &str, message: impl Into<String>) -> Self {
        Self {
            file: file.to_string(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.file, self.message)
    }
}

impl std::error::Error for ContentError {}

mod builtin {
    include!(concat!(env!("OUT_DIR"), "/builtin_assets.rs"));
}

/// Raw content files keyed by path relative to the assets root, e.g. `levels/hub.toml`.
#[derive(Clone, Debug, Default)]
pub struct ContentFiles {
    files: BTreeMap<String, String>,
}

impl ContentFiles {
    /// The content compiled into the binary from `assets/`.
    pub fn builtin() -> Self {
        let mut files = Self::default();
        for (path, text) in builtin::FILES {
            files.insert(path, text);
        }
        files
    }

    /// Reads `data/`, `levels/`, and `scenarios/` TOML files under `root`.
    pub fn from_dir(root: &Path) -> std::io::Result<Self> {
        let mut files = Self::default();
        for dir in ["data", "levels", "scenarios"] {
            let Ok(entries) = std::fs::read_dir(root.join(dir)) else {
                continue;
            };
            for entry in entries {
                let path = entry?.path();
                if path.extension().is_some_and(|e| e == "toml") {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    files.insert(&format!("{dir}/{name}"), &std::fs::read_to_string(&path)?);
                }
            }
        }
        Ok(files)
    }

    pub fn insert(&mut self, path: &str, text: &str) {
        self.files.insert(path.to_string(), text.to_string());
    }

    fn get(&self, path: &str) -> Result<&str, ContentError> {
        self.get_optional(path)
            .ok_or_else(|| ContentError::new(path, "file is missing"))
    }

    fn get_optional(&self, path: &str) -> Option<&str> {
        self.files.get(path).map(String::as_str)
    }

    /// Files directly under `dir/` with their stem (`levels/hub.toml` → `hub`).
    fn in_dir<'a>(&'a self, dir: &'a str) -> impl Iterator<Item = (&'a str, &'a str, &'a str)> {
        self.files.iter().filter_map(move |(path, text)| {
            let rest = path.strip_prefix(dir)?.strip_prefix('/')?;
            let stem = rest.strip_suffix(".toml")?;
            (!stem.contains('/')).then_some((path.as_str(), stem, text.as_str()))
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TerrainFile {
    terrain: Vec<terrain::TerrainSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreatureFile {
    creature: Vec<creature::CreatureSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LightFile {
    light: Vec<LightPreset>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VisionFile {
    vision: Vec<VisionProfile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegendFile {
    legend: BTreeMap<String, LegendEntry>,
}

/// All loaded and cross-validated content.
///
/// `data/lights.toml` and `data/vision.toml` are optional. Without vision profiles, a single
/// `normal` profile that needs no light is used, so content without lighting stays fully visible.
#[derive(Debug)]
pub struct Content {
    terrain: Vec<TerrainDef>,
    terrain_ids: BTreeMap<String, TerrainId>,
    creatures: Vec<CreatureDef>,
    creature_ids: BTreeMap<String, CreatureId>,
    lights: Vec<LightPreset>,
    light_ids: BTreeMap<String, LightId>,
    visions: Vec<VisionProfile>,
    vision_ids: BTreeMap<String, usize>,
    levels: BTreeMap<String, LevelTemplate>,
    scenarios: BTreeMap<String, ScenarioDef>,
}

impl Content {
    /// Loads the built-in content.
    pub fn builtin() -> Result<Self, ContentError> {
        Self::load(&ContentFiles::builtin())
    }

    pub fn load(files: &ContentFiles) -> Result<Self, ContentError> {
        let mut content = Content {
            terrain: Vec::new(),
            terrain_ids: BTreeMap::new(),
            creatures: Vec::new(),
            creature_ids: BTreeMap::new(),
            lights: Vec::new(),
            light_ids: BTreeMap::new(),
            visions: Vec::new(),
            vision_ids: BTreeMap::new(),
            levels: BTreeMap::new(),
            scenarios: BTreeMap::new(),
        };
        content.load_lights(files)?;
        content.load_vision(files)?;
        content.load_terrain(files)?;
        content.load_creatures(files)?;

        let path = "data/legend.toml";
        let legend_file: LegendFile =
            toml::from_str(files.get(path)?).map_err(|e| ContentError::new(path, e.to_string()))?;
        let mut default_legend = BTreeMap::new();
        for (key, entry) in legend_file.legend {
            default_legend.insert(legend_char(path, &key)?, entry);
        }

        for (path, id, text) in files.in_dir("levels") {
            let level = level::parse_level(path, id, text, &content, &default_legend)?;
            content.levels.insert(id.to_string(), level);
        }
        for (path, id, text) in files.in_dir("scenarios") {
            let scenario = ScenarioDef::parse(path, id, text)?;
            content.scenarios.insert(id.to_string(), scenario);
        }
        content.validate()?;
        Ok(content)
    }

    fn load_terrain(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let path = "data/terrain.toml";
        let err = |m: String| ContentError::new(path, m);
        let parsed: TerrainFile =
            toml::from_str(files.get(path)?).map_err(|e| err(e.to_string()))?;
        for (i, spec) in parsed.terrain.iter().enumerate() {
            let id = TerrainId(u16::try_from(i).map_err(|_| err("too many terrain types".into()))?);
            if self.terrain_ids.insert(spec.id.clone(), id).is_some() {
                return Err(err(format!("duplicate terrain id `{}`", spec.id)));
            }
        }
        let resolve = |name: &Option<String>| -> Result<Option<TerrainId>, ContentError> {
            name.as_ref()
                .map(|n| {
                    self.terrain_ids
                        .get(n)
                        .copied()
                        .ok_or_else(|| err(format!("unknown terrain `{n}`")))
                })
                .transpose()
        };
        let mut defs = Vec::with_capacity(parsed.terrain.len());
        for spec in &parsed.terrain {
            let light = self.resolve_light(path, spec.light.as_deref())?;
            if light.is_some() && !spec.transparent {
                return Err(err(format!(
                    "terrain `{}` emits light but is opaque; its light would leak through walls",
                    spec.id
                )));
            }
            defs.push(TerrainDef {
                id: spec.id.clone(),
                name: spec.name.clone(),
                glyph: spec.glyph,
                fg: spec.fg,
                bg: spec.bg,
                walkable: spec.walkable,
                transparent: spec.transparent,
                opens_to: resolve(&spec.opens_to)?,
                closes_to: resolve(&spec.closes_to)?,
                stairs: spec.stairs,
                light,
                anim: spec.anim.clone(),
            });
        }
        self.terrain = defs;
        Ok(())
    }

    fn load_creatures(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let path = "data/creatures.toml";
        let err = |m: String| ContentError::new(path, m);
        let parsed: CreatureFile =
            toml::from_str(files.get(path)?).map_err(|e| err(e.to_string()))?;
        for (i, spec) in parsed.creature.into_iter().enumerate() {
            let id = CreatureId(u16::try_from(i).map_err(|_| err("too many creatures".into()))?);
            if self.creature_ids.insert(spec.id.clone(), id).is_some() {
                return Err(err(format!("duplicate creature id `{}`", spec.id)));
            }
            let light = self.resolve_light(path, spec.light.as_deref())?;
            self.creatures.push(spec.resolve(light));
        }
        if !self.creature_ids.contains_key("player") {
            return Err(err("a creature with id `player` is required".into()));
        }
        Ok(())
    }

    fn load_lights(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let path = "data/lights.toml";
        let Some(text) = files.get_optional(path) else {
            return Ok(());
        };
        let err = |m: String| ContentError::new(path, m);
        let parsed: LightFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        for (i, preset) in parsed.light.into_iter().enumerate() {
            let id = LightId(u16::try_from(i).map_err(|_| err("too many lights".into()))?);
            if !(0..=MAX_LIGHT_RADIUS).contains(&preset.radius) {
                return Err(err(format!(
                    "light `{}` has radius {}; expected 0 to {MAX_LIGHT_RADIUS}",
                    preset.id, preset.radius
                )));
            }
            if self.light_ids.insert(preset.id.clone(), id).is_some() {
                return Err(err(format!("duplicate light id `{}`", preset.id)));
            }
            self.lights.push(preset);
        }
        Ok(())
    }

    fn load_vision(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let path = "data/vision.toml";
        let err = |m: String| ContentError::new(path, m);
        let profiles = match files.get_optional(path) {
            Some(text) => {
                let parsed: VisionFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
                parsed.vision
            }
            None => vec![VisionProfile::needs_no_light()],
        };
        if profiles.is_empty() {
            return Err(err("at least one vision profile is required".into()));
        }
        for (i, profile) in profiles.into_iter().enumerate() {
            if profile.darkvision < 0 {
                return Err(err(format!(
                    "vision `{}` has a negative darkvision radius",
                    profile.id
                )));
            }
            if self.vision_ids.insert(profile.id.clone(), i).is_some() {
                return Err(err(format!("duplicate vision id `{}`", profile.id)));
            }
            self.visions.push(profile);
        }
        Ok(())
    }

    /// Resolves an optional light preset name used in `file`.
    fn resolve_light(
        &self,
        file: &str,
        name: Option<&str>,
    ) -> Result<Option<LightId>, ContentError> {
        name.map(|n| {
            self.light_id(n)
                .ok_or_else(|| ContentError::new(file, format!("unknown light `{n}`")))
        })
        .transpose()
    }

    /// Cross-file checks: stair targets, and each scenario's start level, vision profile, and
    /// lantern exist.
    fn validate(&self) -> Result<(), ContentError> {
        for level in self.levels.values() {
            for link in &level.stairs {
                if let Some(to) = &link.to
                    && !self.levels.contains_key(to)
                {
                    return Err(ContentError::new(
                        &format!("levels/{}.toml", level.id),
                        format!("stairs at {} lead to unknown level `{to}`", link.pos),
                    ));
                }
            }
        }
        for scenario in self.scenarios.values() {
            let file = format!("scenarios/{}.toml", scenario.id);
            let level = self.levels.get(&scenario.start_level).ok_or_else(|| {
                ContentError::new(
                    &file,
                    format!("unknown start level `{}`", scenario.start_level),
                )
            })?;
            if level.player_start.is_none() {
                return Err(ContentError::new(
                    &file,
                    format!("start level `{}` has no player start", level.id),
                ));
            }
            if self.vision(&scenario.vision).is_none() {
                return Err(ContentError::new(
                    &file,
                    format!("unknown vision profile `{}`", scenario.vision),
                ));
            }
            self.resolve_light(&file, scenario.lantern.as_deref())?;
        }
        Ok(())
    }

    /// A stable hash (64-bit FNV-1a) of the ordered terrain, creature, and level ids.
    ///
    /// `TerrainId` and `CreatureId` are indices, so a save is only meaningful for content with the
    /// same ids in the same order; saves record this fingerprint to detect a mismatch.
    pub fn fingerprint(&self) -> u64 {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        let mut feed = |bytes: &[u8]| {
            for &b in bytes {
                hash = (hash ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3);
            }
        };
        let groups: [Vec<&str>; 3] = [
            self.terrain.iter().map(|t| t.id.as_str()).collect(),
            self.creatures.iter().map(|c| c.id.as_str()).collect(),
            self.levels.keys().map(String::as_str).collect(),
        ];
        for ids in groups {
            for id in ids {
                feed(id.as_bytes());
                feed(&[0]);
            }
            // Group separator, so an id cannot migrate between groups unnoticed.
            feed(&[1]);
        }
        hash
    }

    pub fn terrain(&self, id: TerrainId) -> &TerrainDef {
        &self.terrain[usize::from(id.0)]
    }

    pub fn terrain_id(&self, name: &str) -> Option<TerrainId> {
        self.terrain_ids.get(name).copied()
    }

    pub fn terrain_defs(&self) -> &[TerrainDef] {
        &self.terrain
    }

    pub fn creature(&self, id: CreatureId) -> &CreatureDef {
        &self.creatures[usize::from(id.0)]
    }

    pub fn creature_id(&self, name: &str) -> Option<CreatureId> {
        self.creature_ids.get(name).copied()
    }

    pub fn creature_defs(&self) -> &[CreatureDef] {
        &self.creatures
    }

    pub fn light(&self, id: LightId) -> &LightPreset {
        &self.lights[usize::from(id.0)]
    }

    pub fn light_id(&self, name: &str) -> Option<LightId> {
        self.light_ids.get(name).copied()
    }

    pub fn light_presets(&self) -> &[LightPreset] {
        &self.lights
    }

    /// The vision profile with id `name`.
    pub fn vision(&self, name: &str) -> Option<&VisionProfile> {
        self.vision_index(name).map(|i| &self.visions[i])
    }

    /// Index of profile `name` in [`Content::vision_profiles`].
    pub fn vision_index(&self, name: &str) -> Option<usize> {
        self.vision_ids.get(name).copied()
    }

    /// Every vision profile in file order; never empty.
    pub fn vision_profiles(&self) -> &[VisionProfile] {
        &self.visions
    }

    pub fn level(&self, id: &str) -> Option<&LevelTemplate> {
        self.levels.get(id)
    }

    pub fn levels(&self) -> impl Iterator<Item = &LevelTemplate> {
        self.levels.values()
    }

    pub fn scenario(&self, id: &str) -> Option<&ScenarioDef> {
        self.scenarios.get(id)
    }

    pub fn scenarios(&self) -> impl Iterator<Item = &ScenarioDef> {
        self.scenarios.values()
    }

    /// Every glyph content may display (for building the glyph atlas).
    pub fn glyphs(&self) -> BTreeSet<char> {
        let terrain = self.terrain.iter().flat_map(|t| {
            std::iter::once(t.glyph).chain(t.anim.iter().flat_map(|a| a.extra_glyphs()))
        });
        let creatures = self.creatures.iter().flat_map(|c| {
            std::iter::once(c.glyph).chain(c.anim.iter().flat_map(|a| a.extra_glyphs()))
        });
        terrain.chain(creatures).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::Pos;

    #[test]
    fn builtin_content_loads() {
        let content = Content::builtin().expect("built-in content must be valid");
        assert!(content.scenario("showcase").is_some());
        assert!(content.level("hub").is_some());
        assert!(content.terrain_id("wall").is_some());
        assert!(content.glyphs().contains(&'#'));
    }

    #[test]
    fn fingerprint_is_stable_and_tracks_ids() {
        let a = Content::builtin().expect("content").fingerprint();
        assert_eq!(a, Content::builtin().expect("content").fingerprint());
        let mut files = ContentFiles::builtin();
        let hub = files.get("levels/hub.toml").expect("hub").to_string();
        files.insert("levels/hub_copy.toml", &hub);
        if let Ok(other) = Content::load(&files) {
            assert_ne!(a, other.fingerprint());
        }
    }

    fn minimal_files(level_map: &str, extra_legend: &str) -> ContentFiles {
        let mut files = ContentFiles::default();
        files.insert(
            "data/terrain.toml",
            r##"
            [[terrain]]
            id = "void"
            name = "rock"
            glyph = " "
            fg = "#000000"
            walkable = false
            transparent = false
            [[terrain]]
            id = "floor"
            name = "floor"
            glyph = "."
            fg = "#808080"
            [[terrain]]
            id = "wall"
            name = "wall"
            glyph = "#"
            fg = "#a0a0a0"
            walkable = false
            transparent = false
            "##,
        );
        files.insert(
            "data/creatures.toml",
            r##"
            [[creature]]
            id = "player"
            name = "you"
            glyph = "@"
            fg = "#ffffff"
            [[creature]]
            id = "rat"
            name = "rat"
            glyph = "r"
            fg = "#806040"
            "##,
        );
        files.insert(
            "data/legend.toml",
            "[legend]\n\" \" = \"void\"\n\".\" = \"floor\"\n\"#\" = \"wall\"\n",
        );
        files.insert(
            "levels/test.toml",
            &format!(
                "name = \"Test\"\nmap = '''\n{level_map}'''\n[legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n{extra_legend}"
            ),
        );
        files.insert(
            "scenarios/test.toml",
            "name = \"T\"\nstart_level = \"test\"\n",
        );
        files
    }

    #[test]
    fn short_rows_are_padded_with_void() {
        let content = Content::load(&minimal_files("####\n#@.\n####\n", "")).unwrap();
        let level = content.level("test").unwrap();
        assert_eq!((level.terrain.width(), level.terrain.height()), (4, 3));
        assert_eq!(
            level.terrain[Pos::new(3, 1)],
            content.terrain_id("void").unwrap()
        );
        assert_eq!(level.player_start, Some(Pos::new(1, 1)));
    }

    #[test]
    fn unknown_legend_character_is_reported_with_position() {
        let err = Content::load(&minimal_files("#@X#\n", "")).unwrap_err();
        assert_eq!(err.file, "levels/test.toml");
        assert!(err.message.contains("`X` at (2, 0)"), "{}", err.message);
    }

    #[test]
    fn patrol_routes_must_loop_and_stay_walkable() {
        let map = "#####\n#@.r#\n#...#\n#####\n";
        let good = "\"r\" = { terrain = \"floor\", creature = \"rat\", route = \"S W S\" }";
        let err = Content::load(&minimal_files(map, good)).unwrap_err();
        assert!(
            err.message.contains("leaves walkable terrain"),
            "{}",
            err.message
        );

        let not_loop = "\"r\" = { terrain = \"floor\", creature = \"rat\", route = \"S W\" }";
        let err = Content::load(&minimal_files(map, not_loop)).unwrap_err();
        assert!(err.message.contains("loop back"), "{}", err.message);

        let ok = "\"r\" = { terrain = \"floor\", creature = \"rat\", route = \"S W N E\" }";
        let content = Content::load(&minimal_files(map, ok)).unwrap();
        assert_eq!(content.level("test").unwrap().creatures[0].route.len(), 4);
    }

    const LIGHTS: &str = "[[light]]\nid = \"torch\"\ncolor = [300, 200, 100, 0]\nradius = 6\n\
                          flicker = { amount = 0.2, speed = 1.5 }\n";

    #[test]
    fn builtin_lights_and_vision_profiles_load() {
        let content = Content::builtin().unwrap();
        let ids: Vec<_> = content.vision_profiles().iter().map(|v| &v.id).collect();
        assert_eq!(ids, ["normal", "low_light", "darkvision", "uv_sight"]);
        assert_eq!(content.vision("darkvision").map(|v| v.darkvision), Some(5));
        let torch = content.light(content.light_id("torch").unwrap());
        assert!(torch.flicker.is_some() && torch.radius > 0);
        let kobold = content.creature(content.creature_id("kobold").unwrap());
        assert_eq!(kobold.light, content.light_id("torch"));
        let hub = content.level("hub").unwrap();
        assert!(!hub.lights.is_empty());
        let showcase = content.scenario("showcase").unwrap();
        assert_eq!(showcase.lantern.as_deref(), Some("lantern"));
    }

    #[test]
    fn missing_vision_file_means_light_is_not_needed() {
        let content = Content::load(&minimal_files("#@#\n", "")).unwrap();
        let profiles = content.vision_profiles();
        assert_eq!(profiles.len(), 1);
        assert_eq!(
            (profiles[0].id.as_str(), profiles[0].threshold),
            ("normal", 0)
        );
        assert!(content.light_presets().is_empty());
    }

    #[test]
    fn levels_place_lights_and_set_ambient_light() {
        let mut files = minimal_files(
            "#@T#\n",
            "\"T\" = { terrain = \"floor\", light = \"torch\" }",
        );
        files.insert("data/lights.toml", LIGHTS);
        let text = files
            .get("levels/test.toml")
            .unwrap()
            .replace("name = \"Test\"", "name = \"Test\"\nambient = [1, 2, 3, 4]");
        files.insert("levels/test.toml", &text);
        let content = Content::load(&files).unwrap();
        let level = content.level("test").unwrap();
        assert_eq!(level.ambient, crate::light::Light::new(1, 2, 3, 4));
        assert_eq!(
            level.lights.get(&Pos::new(2, 0)).copied(),
            content.light_id("torch")
        );
    }

    #[test]
    fn light_references_are_validated() {
        let unknown = "\"T\" = { terrain = \"floor\", light = \"sun\" }";
        let err = Content::load(&minimal_files("#@T#\n", unknown)).unwrap_err();
        assert!(
            err.message.contains("unknown light `sun`"),
            "{}",
            err.message
        );

        let mut files = minimal_files(
            "#@T#\n",
            "\"T\" = { terrain = \"wall\", light = \"torch\" }",
        );
        files.insert("data/lights.toml", LIGHTS);
        let err = Content::load(&files).unwrap_err();
        assert!(err.message.contains("opaque terrain"), "{}", err.message);

        let mut files = minimal_files("#@#\n", "");
        files.insert(
            "data/lights.toml",
            "[[light]]\nid = \"sun\"\ncolor = [1, 1, 1, 0]\nradius = 99\n",
        );
        let err = Content::load(&files).unwrap_err();
        assert!(err.message.contains("radius 99"), "{}", err.message);

        let mut files = minimal_files("#@#\n", "");
        files.insert(
            "scenarios/test.toml",
            "name = \"T\"\nstart_level = \"test\"\nvision = \"x-ray\"\n",
        );
        let err = Content::load(&files).unwrap_err();
        assert!(
            err.message.contains("vision profile `x-ray`"),
            "{}",
            err.message
        );

        let mut files = minimal_files("#@#\n", "");
        files.insert(
            "scenarios/test.toml",
            "name = \"T\"\nstart_level = \"test\"\nlantern = \"torch\"\n",
        );
        let err = Content::load(&files).unwrap_err();
        assert!(
            err.message.contains("unknown light `torch`"),
            "{}",
            err.message
        );
    }

    #[test]
    fn opaque_terrain_cannot_emit_light() {
        let mut files = minimal_files("#@#\n", "");
        files.insert("data/lights.toml", LIGHTS);
        let terrain = files.get("data/terrain.toml").unwrap().replace(
            "glyph = \"#\"",
            "glyph = \"#\"\n            light = \"torch\"",
        );
        files.insert("data/terrain.toml", &terrain);
        let err = Content::load(&files).unwrap_err();
        assert!(
            err.message.contains("emits light but is opaque"),
            "{}",
            err.message
        );
    }
}
