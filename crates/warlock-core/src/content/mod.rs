//! Game content: definitions, levels, scenarios, dungeon branches, and vaults loaded from TOML
//! (spec §7).
//!
//! Content is read from a [`ContentFiles`] set: the built-in set compiled into the binary (so every
//! build, including the web build, is self-contained) or a directory on disk for development.

mod anim;
mod calendar;
mod creature;
mod dungeon;
mod flow;
mod level;
mod light;
mod particles;
mod scenario;
mod terrain;
mod vault;
mod vision;

pub use anim::GlyphAnim;
pub use calendar::Calendar;
pub use creature::{CreatureDef, CreatureId};
pub use dungeon::{DungeonDef, Generator, LevelPlan, parse_branch_level};
pub use flow::{FLOW_SPEEDS, Flow};
pub(crate) use level::validate_route;
pub use level::{CreatureSpawn, LevelKind, LevelTemplate, StairsLink, parse_route};
pub use light::{Flicker, LightId, LightPreset, MAX_LIGHT_RADIUS};
pub use particles::{ParticleEmitter, ParticleTrigger};
pub use scenario::ScenarioDef;
pub use terrain::{StairsDir, TerrainDef, TerrainId};
pub use vault::Vault;
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

    /// Reads `data/`, `levels/`, `scenarios/`, and `vaults/` TOML files under `root`.
    pub fn from_dir(root: &Path) -> std::io::Result<Self> {
        let mut files = Self::default();
        for dir in ["data", "levels", "scenarios", "vaults"] {
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
/// `data/lights.toml`, `data/vision.toml`, `data/dungeons.toml`, and `data/calendar.toml` are
/// optional. Without vision profiles, a single `normal` profile that needs no light is used, so
/// content without lighting stays fully visible; without a calendar, months are numbered.
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
    dungeons: BTreeMap<String, DungeonDef>,
    /// Sorted by id.
    vaults: Vec<Vault>,
    calendar: Calendar,
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
            dungeons: BTreeMap::new(),
            vaults: Vec::new(),
            calendar: Calendar::default(),
        };
        let path = "data/calendar.toml";
        if let Some(text) = files.get_optional(path) {
            content.calendar = Calendar::parse(path, text)?;
        }
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
        for (path, id, text) in files.in_dir("vaults") {
            let template = level::parse_level(path, id, text, &content, &default_legend)?;
            let vault = Vault::new(template, &content).map_err(|m| ContentError::new(path, m))?;
            content.vaults.push(vault);
        }
        content.load_dungeons(files)?;
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
            if spec.move_cost == 0 {
                return Err(err(format!(
                    "terrain `{}` has move_cost 0; every step takes some time",
                    spec.id
                )));
            }
            check_effects(
                path,
                "terrain",
                &spec.id,
                spec.anim.as_ref(),
                spec.glow,
                spec.haze,
                &spec.particles,
            )?;
            defs.push(TerrainDef {
                id: spec.id.clone(),
                name: spec.name.clone(),
                glyph: spec.glyph,
                fg: spec.fg,
                bg: spec.bg,
                walkable: spec.walkable,
                transparent: spec.transparent,
                move_cost: spec.move_cost,
                opens_to: resolve(&spec.opens_to)?,
                closes_to: resolve(&spec.closes_to)?,
                stairs: spec.stairs,
                light,
                bounce: spec.bounce,
                anim: spec.anim.clone(),
                glow: spec.glow,
                haze: spec.haze,
                particles: spec.particles.clone(),
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
            check_effects(
                path,
                "creature",
                &spec.id,
                spec.anim.as_ref(),
                spec.glow,
                0.0,
                &spec.particles,
            )?;
            if spec.anim.as_ref().is_some_and(GlyphAnim::is_flow) {
                return Err(err(format!(
                    "creature `{}`: only terrain can have a `flow` animation",
                    spec.id
                )));
            }
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

    fn load_dungeons(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let path = "data/dungeons.toml";
        let Some(text) = files.get_optional(path) else {
            return Ok(());
        };
        let parsed: dungeon::DungeonFile =
            toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))?;
        for def in parsed.resolve(path, |name| self.creature_id(name))? {
            if self.levels.contains_key(&def.id) {
                return Err(ContentError::new(
                    path,
                    format!("dungeon `{}` has the same id as a level", def.id),
                ));
            }
            self.dungeons.insert(def.id.clone(), def);
        }
        if !self.dungeons.is_empty() {
            crate::mapgen::Palette::resolve(self).map_err(|m| ContentError::new(path, m))?;
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
    /// lantern exist. Levels of dungeon branches (`depths:3`) count as existing.
    fn validate(&self) -> Result<(), ContentError> {
        for level in self.levels.values() {
            for link in &level.stairs {
                if let Some(to) = &link.to
                    && !self.has_level(to)
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
            if !self.has_level(&scenario.start_level) {
                return Err(ContentError::new(
                    &file,
                    format!("unknown start level `{}`", scenario.start_level),
                ));
            }
            if let Some(level) = self.levels.get(&scenario.start_level)
                && level.player_start.is_none()
            {
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

    /// A stable hash (64-bit FNV-1a) of the ordered terrain, creature, level, light, and dungeon
    /// ids.
    ///
    /// `TerrainId`, `CreatureId`, and `LightId` are indices, so a save is only meaningful for
    /// content with the same ids in the same order; saves record this fingerprint to detect a
    /// mismatch.
    pub fn fingerprint(&self) -> u64 {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        let mut feed = |bytes: &[u8]| {
            for &b in bytes {
                hash = (hash ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3);
            }
        };
        let groups: [Vec<&str>; 5] = [
            self.terrain.iter().map(|t| t.id.as_str()).collect(),
            self.creatures.iter().map(|c| c.id.as_str()).collect(),
            self.levels.keys().map(String::as_str).collect(),
            self.lights.iter().map(|l| l.id.as_str()).collect(),
            self.dungeons.keys().map(String::as_str).collect(),
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

    /// A hand-made level. Generated levels (`depths:3`) are not stored here; see
    /// [`Content::has_level`] and [`crate::mapgen`].
    pub fn level(&self, id: &str) -> Option<&LevelTemplate> {
        self.levels.get(id)
    }

    /// Whether `id` names a hand-made level or a level of a dungeon branch.
    pub fn has_level(&self, id: &str) -> bool {
        self.levels.contains_key(id) || self.branch_level(id).is_some()
    }

    /// The branch and depth of a generated level id such as `depths:3`, if the branch exists and
    /// has that depth.
    pub fn branch_level(&self, id: &str) -> Option<(&DungeonDef, u32)> {
        let (branch, depth) = parse_branch_level(id)?;
        let def = self.dungeons.get(branch)?;
        def.plan(depth).map(|_| (def, depth))
    }

    pub fn dungeon(&self, id: &str) -> Option<&DungeonDef> {
        self.dungeons.get(id)
    }

    pub fn dungeons(&self) -> impl Iterator<Item = &DungeonDef> {
        self.dungeons.values()
    }

    /// Vaults, sorted by id.
    pub fn vaults(&self) -> &[Vault] {
        &self.vaults
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

    /// Month names for showing dates.
    pub fn calendar(&self) -> &Calendar {
        &self.calendar
    }

    /// Every glyph content may display (for building the glyph atlas).
    pub fn glyphs(&self) -> BTreeSet<char> {
        let particles = |emitters: &[ParticleEmitter]| -> Vec<char> {
            emitters.iter().flat_map(|e| e.glyphs.chars()).collect()
        };
        let terrain = self.terrain.iter().flat_map(|t| {
            std::iter::once(t.glyph)
                .chain(t.anim.iter().flat_map(|a| a.extra_glyphs()))
                .chain(particles(&t.particles))
        });
        let creatures = self.creatures.iter().flat_map(|c| {
            std::iter::once(c.glyph)
                .chain(c.anim.iter().flat_map(|a| a.extra_glyphs()))
                .chain(particles(&c.particles))
        });
        terrain.chain(creatures).collect()
    }
}

/// Checks the presentation-only animation and effect fields of a terrain or creature type.
fn check_effects(
    file: &str,
    kind: &str,
    id: &str,
    anim: Option<&GlyphAnim>,
    glow: f32,
    haze: f32,
    particles: &[ParticleEmitter],
) -> Result<(), ContentError> {
    let err = |m: String| ContentError::new(file, format!("{kind} `{id}`: {m}"));
    if let Some(anim) = anim {
        anim.check().map_err(err)?;
    }
    if !(0.0..=1.0).contains(&glow) || !(0.0..=1.0).contains(&haze) {
        return Err(err("`glow` and `haze` must be between 0 and 1".into()));
    }
    for emitter in particles {
        emitter
            .check()
            .map_err(|m| err(format!("particles: {m}")))?;
        if emitter.on == ParticleTrigger::Obstacle && !anim.is_some_and(GlyphAnim::is_flow) {
            return Err(err(
                "particles: `obstacle` emitters need a `flow` animation (running water)".into(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::{Dir, Pos};

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

    #[test]
    fn effects_load_and_their_glyphs_join_the_atlas() {
        let content = Content::builtin().unwrap();
        let brazier = content.terrain(content.terrain_id("brazier").unwrap());
        assert!(brazier.glow > 0.0 && brazier.haze > 0.0);
        assert!(
            brazier
                .particles
                .iter()
                .any(|e| e.on == ParticleTrigger::Ambient)
        );
        let water = content.terrain(content.terrain_id("water").unwrap());
        assert!(
            water
                .particles
                .iter()
                .any(|e| e.on == ParticleTrigger::Step)
        );
        let glyphs = content.glyphs();
        for e in brazier.particles.iter().chain(&water.particles) {
            assert!(e.glyphs.chars().all(|c| glyphs.contains(&c)));
        }
    }

    #[test]
    fn bad_effects_are_reported_with_the_terrain() {
        let mut files = minimal_files("#@#\n", "");
        let terrain = files.get("data/terrain.toml").unwrap().to_string();
        let sparks = "glyphs = \"'\"\ncolors = [\"#ffa040\"]\nlife = 1\n";
        files.insert(
            "data/terrain.toml",
            &format!("{terrain}\n[[terrain.particles]]\n{sparks}"),
        );
        let err = Content::load(&files).unwrap_err();
        assert_eq!(err.file, "data/terrain.toml");
        assert!(
            err.message
                .contains("terrain `wall`: particles: ambient particles need a positive `rate`"),
            "{}",
            err.message
        );
        files.insert("data/terrain.toml", &format!("{terrain}glow = 3\n"));
        let err = Content::load(&files).unwrap_err();
        assert!(err.message.contains("`glow` and `haze`"), "{}", err.message);
        files.insert(
            "data/terrain.toml",
            &format!("{terrain}\n[[terrain.particles]]\n{sparks}rate = 2\n"),
        );
        assert!(Content::load(&files).is_ok());
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
    fn stairs_and_scenarios_may_lead_into_dungeon_branches() {
        let content = Content::builtin().unwrap();
        assert!(content.has_level("depths:8") && !content.has_level("depths:9"));
        let (branch, depth) = content.branch_level("depths:3").unwrap();
        assert_eq!((branch.id.as_str(), depth), ("depths", 3));
        assert_eq!(content.scenario("depths").unwrap().start_level, "depths:1");

        let with_target = |to: &str| {
            let mut files = ContentFiles::builtin();
            let hub = files.get("levels/hub.toml").unwrap().to_string();
            files.insert(
                "levels/hub.toml",
                &hub.replace("to = \"vast\"", &format!("to = \"{to}\"")),
            );
            Content::load(&files)
        };
        assert!(with_target("depths:2").is_ok());
        for bad in ["depths:9", "depths:0", "caves:1"] {
            let err = with_target(bad).unwrap_err();
            assert!(
                err.message.contains(&format!("unknown level `{bad}`")),
                "{}",
                err.message
            );
        }

        let mut files = ContentFiles::builtin();
        files.insert(
            "scenarios/deep.toml",
            "name = \"D\"\nstart_level = \"depths:9\"\n",
        );
        let err = Content::load(&files).unwrap_err();
        assert!(
            err.message.contains("unknown start level"),
            "{}",
            err.message
        );
    }

    #[test]
    fn dungeons_need_the_terrain_generators_build_with() {
        let mut files = minimal_files("#@#\n", "");
        files.insert(
            "data/dungeons.toml",
            "[[dungeon]]\nid = \"d\"\ngenerator = \"rooms\"\ndepth = 1\nsize = [80, 21]\n",
        );
        let err = Content::load(&files).unwrap_err();
        assert_eq!(err.file, "data/dungeons.toml");
        assert!(
            err.message.contains("needs terrain `door_closed`"),
            "{}",
            err.message
        );
    }

    #[test]
    fn vaults_are_validated() {
        let cases = [
            ("####\n#..#\n####\n", "needs at least one entrance"),
            ("##+##\n#.<.#\n#####\n", "cannot have stairs"),
            ("##+##\n#...#\n#.#.#\n##.##\n", "must be solid"),
            ("##+##\n#.#.#\n#####\n", "cannot be reached"),
            ("+###\n#..#\n####\n", "not on a corner"),
        ];
        for (map, expected) in cases {
            let mut files = ContentFiles::builtin();
            files.insert(
                "vaults/bad.toml",
                &format!("name = \"Bad\"\nmap = '''\n{map}'''\n"),
            );
            let err = Content::load(&files).unwrap_err();
            assert_eq!(err.file, "vaults/bad.toml");
            assert!(err.message.contains(expected), "{map}: {}", err.message);
        }
        let content = Content::builtin().unwrap();
        let ids: Vec<_> = content.vaults().iter().map(Vault::id).collect();
        assert_eq!(ids, ["flooded_cell", "guard_post", "shrine"]);
        assert_eq!(content.vaults()[2].entrances.len(), 3);
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

    /// The built-in content plus a level `stream` with `map` and `legend`.
    fn stream(map: &str, legend: &str) -> Result<Content, ContentError> {
        let mut files = ContentFiles::builtin();
        files.insert(
            "levels/stream.toml",
            &format!("name = \"Stream\"\nmap = '''\n{map}'''\n[legend]\n{legend}"),
        );
        Content::load(&files)
    }

    #[test]
    fn legend_arrows_are_running_water_with_a_flow() {
        let content = stream("#####\n#→⇓↙#\n#####\n", "").unwrap();
        let level = content.level("stream").unwrap();
        let running = content.terrain_id("running_water").unwrap();
        assert!(content.terrain(running).flows());
        assert!(
            !content
                .terrain(content.terrain_id("water").unwrap())
                .flows()
        );
        assert_eq!(level.terrain[Pos::new(1, 1)], running);
        let flows: Vec<_> = level.flow.iter().map(|(p, f)| (*p, *f)).collect();
        assert_eq!(
            flows,
            [
                (Pos::new(1, 1), Flow::new(Dir::E, 100)),
                (Pos::new(2, 1), Flow::new(Dir::S, 200)),
                (Pos::new(3, 1), Flow::new(Dir::SW, 100)),
            ]
        );
        // A level's own legend may give any speed in range.
        let slow = "\"s\" = { terrain = \"running_water\", flow = \"N\", flow_speed = 40 }\n";
        let content = stream("###\n#s#\n###\n", slow).unwrap();
        let flow = content.level("stream").unwrap().flow[&Pos::new(1, 1)];
        assert_eq!(flow, Flow::new(Dir::N, 40));
    }

    #[test]
    fn flow_is_required_on_running_water_and_only_there() {
        let cases = [
            ("\"s\" = \"running_water\"\n", "needs a flow direction"),
            (
                "\"s\" = { terrain = \"floor\", flow = \"S\" }\n",
                "is not running water",
            ),
            (
                "\"s\" = { terrain = \"floor\", flow_speed = 50 }\n",
                "`flow_speed` but no `flow`",
            ),
            (
                "\"s\" = { terrain = \"running_water\", flow = \"S\", flow_speed = 0 }\n",
                "expected 10 to 300",
            ),
            (
                "\"s\" = { terrain = \"running_water\", flow = \"Down\" }\n",
                "flow = \"Down\"",
            ),
        ];
        for (legend, expected) in cases {
            let err = stream("###\n#s#\n###\n", legend).unwrap_err();
            assert_eq!(err.file, "levels/stream.toml");
            assert!(err.message.contains(expected), "{legend}: {}", err.message);
        }
    }

    #[test]
    fn water_animations_are_checked() {
        let cases = [
            (
                "anim = { kind = \"wave\", glint = 1.5 }",
                "`glint` must be between 0 and 1",
            ),
            (
                "anim = { kind = \"flow\", speed = -1 }",
                "finite and not negative",
            ),
        ];
        for (anim, expected) in cases {
            let mut files = minimal_files("#@#\n", "");
            let terrain = files.get("data/terrain.toml").unwrap().replace(
                "glyph = \".\"",
                &format!("glyph = \".\"\n            {anim}"),
            );
            files.insert("data/terrain.toml", &terrain);
            let err = Content::load(&files).unwrap_err();
            assert!(err.message.contains(expected), "{anim}: {}", err.message);
        }
        // Foam needs running water to foam in.
        let mut files = minimal_files("#@#\n", "");
        let terrain = files.get("data/terrain.toml").unwrap().to_string();
        let foam =
            "on = \"obstacle\"\nglyphs = \".\"\ncolors = [\"#ffffff\"]\nrate = 1\nlife = 1\n";
        files.insert(
            "data/terrain.toml",
            &format!("{terrain}\n[[terrain.particles]]\n{foam}"),
        );
        let err = Content::load(&files).unwrap_err();
        assert!(
            err.message.contains("need a `flow` animation"),
            "{}",
            err.message
        );
    }

    #[test]
    fn the_hub_has_still_ponds_and_running_rills() {
        let content = Content::builtin().unwrap();
        let hub = content.level("hub").unwrap();
        let still = content.terrain_id("water").unwrap();
        let running = content.terrain_id("running_water").unwrap();
        assert!(hub.terrain.cells().iter().filter(|t| **t == still).count() > 50);
        let rill: Vec<_> = hub
            .terrain
            .iter()
            .filter(|(_, t)| **t == running)
            .map(|(p, _)| p)
            .collect();
        assert!(!rill.is_empty());
        assert!(rill.iter().all(|p| hub.flow.contains_key(p)));
        assert_eq!(hub.flow.len(), rill.len());
        assert!(hub.flow.values().any(|f| f.speed > 100), "a fast spring");
    }
}
