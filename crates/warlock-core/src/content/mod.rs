//! Game content: definitions, levels, scenarios, dungeon branches, and vaults loaded from TOML
//! (spec §7).
//!
//! Content is read from a [`ContentFiles`] set: the built-in set compiled into the binary (so every
//! build, including the web build, is self-contained) or a directory on disk for development.

mod action;
mod anim;
mod calendar;
mod class;
mod creature;
mod dungeon;
mod effect;
mod experience;
mod faction;
mod flow;
mod horoscope;
mod level;
mod light;
mod material;
mod object;
mod overworld;
mod particles;
mod race;
mod scenario;
mod size;
mod skill;
mod sky;
mod slot;
mod terrain;
mod vault;
mod vision;
mod weather;

pub use action::{ActionCosts, ActionKind, BASE_TURN, COSTS, NORMAL_COST, price};
pub use anim::GlyphAnim;
pub use calendar::Calendar;
pub use class::ClassDef;
pub use creature::{
    Combat, CreatureDef, CreatureId, DEFAULT_SIGHT, Hide, NORMAL_SPEED, NaturalAttack, SPEEDS,
};
pub use dungeon::{DungeonDef, Generator, LevelPlan, parse_branch_level};
pub use effect::{
    Darken, EffectDef, EffectLight, EffectShape, FILL_GLYPH, MAX_BURST_RADIUS, MAX_DARKEN,
    MAX_SHAKE_PIXELS, Shake,
};
pub use experience::{Experience, MAX_LEVEL, MAX_XP};
pub use faction::{FactionDef, FactionId, Factions, PLAYER_FACTION, Stance};
pub use flow::{FLOW_SPEEDS, Flow};
pub use horoscope::{Horoscope, Sign, SignPart};
pub(crate) use level::validate_route;
pub use level::{
    CreatureSpawn, EdgeExit, LevelKind, LevelTemplate, Region, StairsLink, parse_route,
};
pub use light::{Dips, Drift, HueDrift, LightId, LightPreset, MAX_LIGHT_RADIUS, Stability};
pub use material::{MaterialDef, MaterialId};
pub use object::{MAX_BONUS, MAX_PLACED, ObjectDef, ObjectId, article};
pub use overworld::{Landscape, LocalArea, local_area_id, parse_local_area};
pub use particles::{ParticleEmitter, ParticleTrigger};
pub use race::{RaceDef, Races};
pub use scenario::ScenarioDef;
pub use size::{CARRY, MEDIUM, SizeDef};
pub use skill::{SkillBook, SkillDef};
pub use sky::{MAX_MOONS, MAX_SIZE, MoonDef, PHASE_ONE, STANDARD_SIZE};
pub use slot::{MAX_SLOTS, SlotDef, SlotId};
pub use terrain::{MAX_HEIGHT, StairsDir, TerrainDef, TerrainId};
pub use vault::Vault;
pub use vision::VisionProfile;
pub use weather::{Climate, MAX_SIGHT_LOSS, WeatherDef};

use crate::noise::MAX_LOUDNESS;
use crate::perception::VIEW_RADIUS;
use crate::skills::{Increase, SkillId, Skills};
use crate::stats::Stats;
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

    /// Leaves a file out, e.g. `data/weather.toml` for an always clear sky.
    pub fn remove(&mut self, path: &str) {
        self.files.remove(path);
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
struct MaterialFile {
    material: Vec<MaterialDef>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectFile {
    object: Vec<object::ObjectSpec>,
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
/// `data/lights.toml`, `data/vision.toml`, `data/factions.toml`, `data/dungeons.toml`,
/// `data/calendar.toml`, `data/sky.toml`, `data/materials.toml`, `data/objects.toml`,
/// `data/weather.toml`, `data/actions.toml`, `data/slots.toml`, `data/sizes.toml`,
/// `data/races.toml`, `data/classes.toml`, `data/skills.toml`, `data/horoscope.toml`, and
/// `data/effects.toml` are optional (without effects, spell effects are not drawn). Without vision
/// profiles, a single `normal` profile that needs no light is used, so content without lighting
/// stays fully visible; without sizes, everything is medium; without factions, everyone is neutral; without a calendar, months are numbered;
/// without a sky file there are no moons; without an objects file there are no objects; without a
/// weather file the sky is always clear; without an actions file every action costs 1000;
/// without a slots file nothing can be worn or wielded; without races or classes there is no
/// character to choose, and scenarios give the player's eyes and stats themselves; without
/// skills nobody is trained in anything; without a horoscope, characters are born under no sign.
#[derive(Debug)]
pub struct Content {
    terrain: Vec<TerrainDef>,
    terrain_ids: BTreeMap<String, TerrainId>,
    creatures: Vec<CreatureDef>,
    creature_ids: BTreeMap<String, CreatureId>,
    materials: Vec<MaterialDef>,
    material_ids: BTreeMap<String, MaterialId>,
    objects: Vec<ObjectDef>,
    object_ids: BTreeMap<String, ObjectId>,
    /// Equipment slots, in display order.
    slots: Vec<SlotDef>,
    factions: Factions,
    lights: Vec<LightPreset>,
    light_ids: BTreeMap<String, LightId>,
    visions: Vec<VisionProfile>,
    vision_ids: BTreeMap<String, usize>,
    /// In `data/sizes.toml` order.
    sizes: Vec<SizeDef>,
    levels: BTreeMap<String, LevelTemplate>,
    scenarios: BTreeMap<String, ScenarioDef>,
    dungeons: BTreeMap<String, DungeonDef>,
    /// Sorted by id.
    vaults: Vec<Vault>,
    calendar: Calendar,
    /// In `data/sky.toml` order.
    moons: Vec<MoonDef>,
    climate: Climate,
    actions: ActionCosts,
    skills: SkillBook,
    races: Races,
    classes: Vec<ClassDef>,
    experience: Experience,
    horoscope: Option<Horoscope>,
    /// Spell effects, in `data/effects.toml` order: presentation only.
    effects: Vec<EffectDef>,
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
            materials: Vec::new(),
            material_ids: BTreeMap::new(),
            objects: Vec::new(),
            object_ids: BTreeMap::new(),
            slots: Vec::new(),
            factions: Factions::default(),
            lights: Vec::new(),
            light_ids: BTreeMap::new(),
            visions: Vec::new(),
            vision_ids: BTreeMap::new(),
            sizes: vec![SizeDef::medium()],
            levels: BTreeMap::new(),
            scenarios: BTreeMap::new(),
            dungeons: BTreeMap::new(),
            vaults: Vec::new(),
            calendar: Calendar::default(),
            moons: Vec::new(),
            climate: Climate::default(),
            actions: ActionCosts::default(),
            skills: SkillBook::default(),
            races: Races::default(),
            classes: Vec::new(),
            experience: Experience::default(),
            horoscope: None,
            effects: Vec::new(),
        };
        let path = "data/calendar.toml";
        if let Some(text) = files.get_optional(path) {
            content.calendar = Calendar::parse(path, text)?;
        }
        let path = "data/sky.toml";
        if let Some(text) = files.get_optional(path) {
            content.moons = sky::parse(path, text)?;
        }
        let path = "data/weather.toml";
        if let Some(text) = files.get_optional(path) {
            content.climate = Climate::parse(path, text)?;
        }
        let path = "data/actions.toml";
        if let Some(text) = files.get_optional(path) {
            content.actions = ActionCosts::parse(path, text)?;
        }
        content.load_lights(files)?;
        content.load_vision(files)?;
        let path = "data/sizes.toml";
        if let Some(text) = files.get_optional(path) {
            content.sizes = size::parse(path, text)?;
        }
        let path = "data/skills.toml";
        if let Some(text) = files.get_optional(path) {
            content.skills = SkillBook::parse(path, text)?;
        }
        let path = "data/races.toml";
        if let Some(text) = files.get_optional(path) {
            let has_vision = |id: &str| content.vision(id).is_some();
            let size_index = |id: &str| content.size_index(id);
            let skill_id = |id: &str| content.skill_id(id);
            content.races = Races::parse(path, text, has_vision, size_index, skill_id)?;
        }
        let path = "data/classes.toml";
        if let Some(text) = files.get_optional(path) {
            content.classes = ClassDef::parse(path, text, |id| content.skill_id(id))?;
        }
        let path = "data/experience.toml";
        if let Some(text) = files.get_optional(path) {
            content.experience = Experience::parse(path, text)?;
        }
        let path = "data/horoscope.toml";
        if let Some(text) = files.get_optional(path) {
            content.horoscope = Some(Horoscope::parse(path, text)?);
        }
        content.load_terrain(files)?;
        content.load_factions(files)?;
        content.load_creatures(files)?;
        content.load_objects(files)?;
        let path = "data/effects.toml";
        if let Some(text) = files.get_optional(path) {
            content.effects = effect::parse(path, text)?;
        }

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
            if spec
                .travel_minutes
                .is_some_and(|m| m == 0 || u32::from(m) > crate::time::MINUTES_PER_DAY)
            {
                return Err(err(format!(
                    "terrain `{}`: travel_minutes must be 1 to 1440 (a day)",
                    spec.id
                )));
            }
            let height = spec.height.unwrap_or(u8::from(!spec.transparent));
            if height > MAX_HEIGHT {
                return Err(err(format!(
                    "terrain `{}` has height {height}; expected 0 to {MAX_HEIGHT}",
                    spec.id
                )));
            }
            check_terrain_noise(spec).map_err(|m| err(format!("terrain `{}`: {m}", spec.id)))?;
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
                travel_minutes: spec.travel_minutes,
                local: resolve(&spec.local)?,
                landscape: spec.landscape,
                opens_to: resolve(&spec.opens_to)?,
                closes_to: resolve(&spec.closes_to)?,
                stairs: spec.stairs,
                light,
                bounce: spec.bounce,
                height,
                roofed: spec.roofed,
                step_noise: spec.step_noise,
                toggle_noise: spec.toggle_noise,
                // Solid terrain blocks sound unless it muffles it; everything else passes it.
                muffle: spec
                    .muffle
                    .or((spec.walkable || spec.transparent).then_some(0)),
                anim: spec.anim.clone(),
                glow: spec.glow,
                haze: spec.haze,
                particles: spec.particles.clone(),
            });
        }
        if let Some(def) = defs
            .iter()
            .find(|d| d.local.is_some_and(|l| !defs[usize::from(l.0)].walkable))
        {
            return Err(err(format!(
                "terrain `{}`: its local area ground must be walkable",
                def.id
            )));
        }
        if let Some(def) = defs.iter().find(|d| {
            d.local.is_some()
                && !d
                    .landscape
                    .is_some_and(|l| l.is_land() || l == Landscape::Road)
        }) {
            return Err(err(format!(
                "terrain `{}` has a local area but no land or road `landscape`",
                def.id
            )));
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
            if !SPEEDS.contains(&spec.speed) {
                return Err(err(format!(
                    "creature `{}`: speed {} is out of range ({} to {})",
                    spec.id,
                    spec.speed,
                    SPEEDS.start(),
                    SPEEDS.end()
                )));
            }
            let light = self.resolve_light(path, spec.light.as_deref())?;
            if spec.footsteps > MAX_LOUDNESS
                || spec.hearing.is_some_and(|h| h == 0 || h > MAX_LOUDNESS)
            {
                return Err(err(format!(
                    "creature `{}`: `footsteps` must be at most {MAX_LOUDNESS} and `hearing` 1 to \
                     {MAX_LOUDNESS}",
                    spec.id
                )));
            }
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
            if spec.weight == 0 {
                return Err(err(format!(
                    "creature `{}`: `weight` is in grams and must be at least 1",
                    spec.id
                )));
            }
            if !(1..=MAX_LEVEL).contains(&spec.level) || spec.xp > MAX_XP {
                return Err(err(format!(
                    "creature `{}`: `level` must be 1 to {MAX_LEVEL} and `xp` at most {MAX_XP}",
                    spec.id
                )));
            }
            let has_stats = !spec.stats.is_empty() || !spec.potential.is_empty();
            if spec.id == "player" && (has_stats || spec.base_hp != 0) {
                return Err(err(
                    "creature `player`: the player's stats and hit points come from the scenario \
                     (`stats`, `potential`, and `hp` in scenarios/*.toml)"
                        .into(),
                ));
            }
            if spec.id == "player" && !spec.skills.is_empty() {
                return Err(err(
                    "creature `player`: the player's skills come from their race and class".into(),
                ));
            }
            let skills = self
                .creature_skills(&spec.skills)
                .map_err(|m| err(format!("creature `{}`: skills: {m}", spec.id)))?;
            if spec
                .natural
                .as_ref()
                .is_some_and(|n| !(1..=object::MAX_BONUS).contains(&n.defence) || n.name.is_empty())
            {
                return Err(err(format!(
                    "creature `{}`: natural: `defence` must be 1 to {} and `name` given",
                    spec.id,
                    object::MAX_BONUS
                )));
            }
            let attack = (spec.attack.clone())
                .map(creature::NaturalAttackSpec::resolve)
                .transpose()
                .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
            let stats = Stats::from_data(&spec.stats, &spec.potential)
                .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
            if let Some(cost) = spec.attack_cost {
                action::check_cost(cost)
                    .map_err(|m| err(format!("creature `{}`: attack_cost: {m}", spec.id)))?;
            }
            let size_id = spec.size.as_deref().unwrap_or(MEDIUM);
            let size = (self.size_index(size_id))
                .ok_or_else(|| err(format!("creature `{}`: unknown size `{size_id}`", spec.id)))?;
            let resolved = creature::Resolved {
                light,
                size,
                stats,
                skills,
                attack,
                faction: self.creature_faction(&spec).map_err(err)?,
                vision: self.vision_index(&spec.vision).ok_or_else(|| {
                    err(format!(
                        "creature `{}`: unknown vision profile `{}`",
                        spec.id, spec.vision
                    ))
                })?,
            };
            check_combat(&spec).map_err(err)?;
            self.creatures.push(spec.resolve(resolved));
        }
        if !self.creature_ids.contains_key("player") {
            return Err(err("a creature with id `player` is required".into()));
        }
        Ok(())
    }

    /// A creature type's skills as data lists them (`skills = { listen = 30 }`): fixed values,
    /// 0 to [`crate::skills::MAX_VALUE`], of skills `data/skills.toml` has.
    fn creature_skills(&self, data: &BTreeMap<String, u16>) -> Result<Skills, String> {
        let mut skills = Skills::default();
        for (name, &value) in data {
            let id = (self.skill_id(name)).ok_or_else(|| format!("unknown skill `{name}`"))?;
            if value > crate::skills::MAX_VALUE {
                return Err(format!(
                    "{name} {value} is more than {}",
                    crate::skills::MAX_VALUE
                ));
            }
            skills.add(id, i32::from(value));
        }
        Ok(skills)
    }

    /// The faction a creature type names. With factions defined, every creature but the player
    /// must name one, and the player belongs to [`PLAYER_FACTION`]; without, none may.
    fn creature_faction(&self, spec: &creature::CreatureSpec) -> Result<Option<FactionId>, String> {
        if self.factions.is_empty() {
            return match &spec.faction {
                Some(name) => Err(format!(
                    "creature `{}`: unknown faction `{name}` (there is no data/factions.toml)",
                    spec.id
                )),
                None => Ok(None),
            };
        }
        let is_player = spec.id == "player";
        let name = match &spec.faction {
            Some(name) => name.as_str(),
            None if is_player => PLAYER_FACTION,
            None => {
                return Err(format!(
                    "creature `{}` needs a `faction` (data/factions.toml)",
                    spec.id
                ));
            }
        };
        if is_player && name != PLAYER_FACTION {
            return Err(format!(
                "the player belongs to the faction `{PLAYER_FACTION}`, not `{name}`"
            ));
        }
        self.factions
            .id(name)
            .map(Some)
            .ok_or_else(|| format!("creature `{}`: unknown faction `{name}`", spec.id))
    }

    fn load_factions(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let path = "data/factions.toml";
        let Some(text) = files.get_optional(path) else {
            return Ok(());
        };
        let err = |m: String| ContentError::new(path, m);
        let parsed: faction::FactionFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        self.factions = Factions::resolve(parsed.faction).map_err(err)?;
        Ok(())
    }

    /// Materials, then the objects made of them.
    fn load_objects(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let path = "data/materials.toml";
        if let Some(text) = files.get_optional(path) {
            let err = |m: String| ContentError::new(path, m);
            let parsed: MaterialFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
            for (i, def) in parsed.material.into_iter().enumerate() {
                let id =
                    MaterialId(u16::try_from(i).map_err(|_| err("too many materials".into()))?);
                if self.material_ids.insert(def.id.clone(), id).is_some() {
                    return Err(err(format!("duplicate material id `{}`", def.id)));
                }
                self.materials.push(def);
            }
        }
        let path = "data/slots.toml";
        if let Some(text) = files.get_optional(path) {
            self.slots = slot::parse(path, text)?;
        }
        let path = "data/objects.toml";
        let Some(text) = files.get_optional(path) else {
            return Ok(());
        };
        let err = |m: String| ContentError::new(path, m);
        let parsed: ObjectFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let mut lit = Vec::new();
        for (i, spec) in parsed.object.into_iter().enumerate() {
            let id = ObjectId(u16::try_from(i).map_err(|_| err("too many objects".into()))?);
            if self.object_ids.insert(spec.id.clone(), id).is_some() {
                return Err(err(format!("duplicate object id `{}`", spec.id)));
            }
            let (def, lit_as) = spec.resolve(
                path,
                |m| self.material_ids.get(m).copied(),
                |l| self.light_ids.get(l).copied(),
                &self.slots,
            )?;
            check_effects(path, "object", &def.id, None, def.glow, 0.0, &def.particles)?;
            lit.push(lit_as);
            self.objects.push(def);
        }
        // What is lit as it is loosed flies as an object that burns.
        for (i, lit_as) in lit.into_iter().enumerate() {
            let Some(name) = lit_as else { continue };
            let target = self.object_ids.get(&name).copied();
            let def = &self.objects[i];
            let Some(target) = target.filter(|t| self.objects[usize::from(t.0)].burning.is_some())
            else {
                return Err(err(format!(
                    "object `{}`: `lit = \"{name}\"` must name an object that burns (`light` and \
                     `burns`)",
                    def.id
                )));
            };
            self.objects[i].lit = Some(target);
        }
        // Launchers and ammunition pair by kind: a launcher shoots something there is.
        for o in &self.objects {
            if let Some(kind) = &o.launches
                && !self.objects.iter().any(|a| a.ammo.as_ref() == Some(kind))
            {
                return Err(err(format!(
                    "object `{}` launches `{kind}`, but no object has `ammo = \"{kind}\"`",
                    o.id
                )));
            }
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
            preset
                .stability
                .check()
                .map_err(|m| err(format!("light `{}`: stability: {m}", preset.id)))?;
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

    /// Cross-file checks: at most one overworld, stair targets, and each scenario's start level,
    /// vision profile, lantern, and character's race and class exist. Levels of dungeon branches (`depths:3`) and local areas
    /// of the overworld (`local:12,30`) count as existing.
    fn validate(&self) -> Result<(), ContentError> {
        let mut overworlds = self
            .levels
            .values()
            .filter(|l| l.kind == LevelKind::Overworld);
        if let (Some(_), Some(second)) = (overworlds.next(), overworlds.next()) {
            return Err(ContentError::new(
                &format!("levels/{}.toml", second.id),
                "there can be only one level of kind `overworld`",
            ));
        }
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
            if self.vision(scenario.vision()).is_none() {
                return Err(ContentError::new(
                    &file,
                    format!("unknown vision profile `{}`", scenario.vision()),
                ));
            }
            if let Some(character) = &scenario.character {
                character
                    .check(self)
                    .map_err(|m| ContentError::new(&file, format!("character: {m}")))?;
            }
            self.resolve_light(&file, scenario.lantern.as_deref())?;
        }
        Ok(())
    }

    /// A stable hash (64-bit FNV-1a) of the ordered terrain, creature, level, light, dungeon,
    /// object, slot, and skill ids.
    ///
    /// `TerrainId`, `CreatureId`, `LightId`, `ObjectId`, `SlotId`, and `SkillId` are indices, so a
    /// save is only meaningful for content with the same ids in the same order; saves record this
    /// fingerprint to detect a mismatch.
    pub fn fingerprint(&self) -> u64 {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        let mut feed = |bytes: &[u8]| {
            for &b in bytes {
                hash = (hash ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3);
            }
        };
        let groups: [Vec<&str>; 8] = [
            self.terrain.iter().map(|t| t.id.as_str()).collect(),
            self.creatures.iter().map(|c| c.id.as_str()).collect(),
            self.levels.keys().map(String::as_str).collect(),
            self.lights.iter().map(|l| l.id.as_str()).collect(),
            self.dungeons.keys().map(String::as_str).collect(),
            self.objects.iter().map(|o| o.id.as_str()).collect(),
            self.slots.iter().map(|s| s.id.as_str()).collect(),
            self.skills.list.iter().map(|s| s.id.as_str()).collect(),
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

    /// The height of the tallest terrain (sun shadows reach no farther than its shadow).
    pub fn tallest_terrain(&self) -> u8 {
        self.terrain.iter().map(|t| t.height).max().unwrap_or(0)
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

    pub fn object(&self, id: ObjectId) -> &ObjectDef {
        &self.objects[usize::from(id.0)]
    }

    pub fn object_id(&self, name: &str) -> Option<ObjectId> {
        self.object_ids.get(name).copied()
    }

    /// Every object type, in file order (by [`ObjectId`]).
    pub fn object_defs(&self) -> &[ObjectDef] {
        &self.objects
    }

    /// The equipment slots, in display order (by [`SlotId`]; empty without `data/slots.toml`).
    pub fn slots(&self) -> &[SlotDef] {
        &self.slots
    }

    pub fn slot(&self, id: SlotId) -> &SlotDef {
        &self.slots[usize::from(id.0)]
    }

    pub fn slot_id(&self, name: &str) -> Option<SlotId> {
        let i = self.slots.iter().position(|s| s.id == name)?;
        u8::try_from(i).ok().map(SlotId)
    }

    /// Every slot, with its id, in display order.
    pub fn slot_ids(&self) -> impl Iterator<Item = (SlotId, &SlotDef)> {
        (0u8..).zip(&self.slots).map(|(i, s)| (SlotId(i), s))
    }

    /// Whether `object` goes in `slot`: its `slot` is what the slot fits.
    pub fn fits(&self, object: ObjectId, slot: SlotId) -> bool {
        let (def, slot) = (self.object(object), self.slot(slot));
        def.slot.as_ref() == Some(&slot.fits)
            || (slot.takes_weapons
                && (def.can_throw() || (def.attack.is_some() && def.ammo.is_none())))
    }

    /// Whether `object` goes in `slot` by its own `slot`, not as a weapon some slot takes too.
    pub fn fits_properly(&self, object: ObjectId, slot: SlotId) -> bool {
        self.object(object).slot.as_ref() == Some(&self.slot(slot).fits)
    }

    /// The hand slots (a two-handed object takes them all), in display order.
    pub fn hand_slots(&self) -> impl Iterator<Item = SlotId> + '_ {
        self.slot_ids().filter(|(_, s)| s.hand).map(|(id, _)| id)
    }

    /// Whether missile weapon `launcher` shoots `ammo`: they pair by kind (`launches` and
    /// `ammo` in `data/objects.toml`).
    pub fn fires(&self, launcher: ObjectId, ammo: ObjectId) -> bool {
        let kind = &self.object(launcher).launches;
        kind.is_some() && *kind == self.object(ammo).ammo
    }

    pub fn material(&self, id: MaterialId) -> &MaterialDef {
        &self.materials[usize::from(id.0)]
    }

    pub fn material_id(&self, name: &str) -> Option<MaterialId> {
        self.material_ids.get(name).copied()
    }

    /// Every material, in file order (by [`MaterialId`]).
    pub fn materials(&self) -> &[MaterialDef] {
        &self.materials
    }

    /// The factions of `data/factions.toml` (empty without the file).
    pub fn factions(&self) -> &Factions {
        &self.factions
    }

    /// How creatures of types `a` and `b` stand toward each other: their factions' stance
    /// (spec §7 "Factions"); neutral without factions. The player is the creature `player`.
    pub fn stance(&self, a: CreatureId, b: CreatureId) -> Stance {
        match (self.creature(a).faction, self.creature(b).faction) {
            (Some(a), Some(b)) => self.factions.stance(a, b),
            _ => Stance::Neutral,
        }
    }

    /// How two actors stand toward each other, given the factions they belong to (their type's,
    /// or the one their level placed them in); neutral if either has none.
    pub fn stance_of(&self, a: Option<FactionId>, b: Option<FactionId>) -> Stance {
        match (a, b) {
            (Some(a), Some(b)) => self.factions.stance(a, b),
            _ => Stance::Neutral,
        }
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

    /// The sizes of `data/sizes.toml`, in file order (only medium without the file).
    pub fn sizes(&self) -> &[SizeDef] {
        &self.sizes
    }

    /// The size at `index` in [`Content::sizes`] (a race's or creature type's `size`).
    pub fn size(&self, index: usize) -> &SizeDef {
        &self.sizes[index]
    }

    /// The index of size `id` in [`Content::sizes`].
    pub fn size_index(&self, id: &str) -> Option<usize> {
        self.sizes.iter().position(|s| s.id == id)
    }

    /// A hand-made level. Generated levels (`depths:3`, `local:12,30`) are not stored here; see
    /// [`Content::has_level`] and [`crate::mapgen`].
    pub fn level(&self, id: &str) -> Option<&LevelTemplate> {
        self.levels.get(id)
    }

    /// Whether `id` names a hand-made level, a level of a dungeon branch, or a local area of the
    /// overworld.
    pub fn has_level(&self, id: &str) -> bool {
        self.levels.contains_key(id)
            || self.branch_level(id).is_some()
            || self.local_area(id).is_some()
    }

    /// The name a player would know a level by: a hand-made level's name or a branch level's
    /// generated name (e.g. "The Depths 3"). `None` for unknown ids and local areas.
    pub fn level_display_name(&self, id: &str) -> Option<String> {
        if let Some(level) = self.levels.get(id) {
            return Some(level.name.clone());
        }
        self.branch_level(id)
            .map(|(def, depth)| def.level_name(depth))
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

    /// The races of `data/races.toml`, in the order offered (none without the file).
    pub fn races(&self) -> &Races {
        &self.races
    }

    /// The race with id `id`.
    pub fn race(&self, id: &str) -> Option<&RaceDef> {
        self.races.list.iter().find(|r| r.id == id)
    }

    /// The classes of `data/classes.toml`, in the order offered (none without the file).
    pub fn classes(&self) -> &[ClassDef] {
        &self.classes
    }

    /// The class with id `id`.
    pub fn class(&self, id: &str) -> Option<&ClassDef> {
        self.classes.iter().find(|c| c.id == id)
    }

    /// The experience rules of `data/experience.toml` (spec §8 "Experience").
    pub fn experience(&self) -> &Experience {
        &self.experience
    }

    /// The skills of `data/skills.toml`, in file order (by [`SkillId`]; none without the file).
    pub fn skills(&self) -> &[SkillDef] {
        &self.skills.list
    }

    pub fn skill(&self, id: SkillId) -> &SkillDef {
        &self.skills.list[usize::from(id.0)]
    }

    /// The skill with data id `name`.
    pub fn skill_id(&self, name: &str) -> Option<SkillId> {
        self.skills.id(name)
    }

    /// Every skill, with its id, in file order.
    pub fn skill_ids(&self) -> impl Iterator<Item = (SkillId, &SkillDef)> {
        (0u8..).zip(&self.skills.list).map(|(i, s)| (SkillId(i), s))
    }

    /// **[provisional]** How training marks become points (`[increase]` in `data/skills.toml`).
    pub fn increase(&self) -> &Increase {
        &self.skills.increase
    }

    /// The horoscope of `data/horoscope.toml` (none without the file).
    pub fn horoscope(&self) -> Option<&Horoscope> {
        self.horoscope.as_ref()
    }

    /// The spell effects of `data/effects.toml`, in file order (none without the file).
    /// Presentation only: no rule reads them.
    pub fn effects(&self) -> &[EffectDef] {
        &self.effects
    }

    /// The spell effect with id `id`.
    pub fn effect(&self, id: &str) -> Option<&EffectDef> {
        self.effects.iter().find(|e| e.id == id)
    }

    /// Month names for showing dates.
    pub fn calendar(&self) -> &Calendar {
        &self.calendar
    }

    /// The moons, in `data/sky.toml` order (none without the file).
    pub fn moons(&self) -> &[MoonDef] {
        &self.moons
    }

    /// The weather's chances and effects (always clear without `data/weather.toml`).
    pub fn climate(&self) -> &Climate {
        &self.climate
    }

    /// What every action costs (`data/actions.toml`; 1000 each without it).
    pub fn action_costs(&self) -> &ActionCosts {
        &self.actions
    }

    /// What action `kind` costs a creature of type `creature`: its type's own attack cost for an
    /// attack, if it has one, else the action's cost (spec §8 "Time").
    pub fn action_cost(&self, creature: CreatureId, kind: ActionKind) -> u32 {
        match self.creature(creature).attack_cost {
            Some(cost) if kind.is_attack() => cost,
            _ => self.actions.cost(kind),
        }
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
        let objects = self.objects.iter().flat_map(|o| {
            std::iter::once(o.glyph)
                .chain(o.missile)
                .chain(particles(&o.particles))
        });
        let weather = crate::weather::Weather::ALL
            .into_iter()
            .flat_map(|w| self.climate.def(w).emitters())
            .flat_map(|e| e.glyphs.chars());
        let effects = self.effects.iter().flat_map(EffectDef::glyphs);
        terrain
            .chain(creatures)
            .chain(objects)
            .chain(weather)
            .chain(effects)
            .collect()
    }
}

/// The most a terrain may scale footsteps by, in percent.
const MAX_STEP_NOISE: u16 = 1000;

/// Checks a terrain type's noise fields (spec §5 "Hearing").
fn check_terrain_noise(spec: &terrain::TerrainSpec) -> Result<(), String> {
    if spec.step_noise > MAX_STEP_NOISE {
        return Err(format!(
            "`step_noise` is {}%; expected 0 to {MAX_STEP_NOISE}",
            spec.step_noise
        ));
    }
    if spec.toggle_noise > MAX_LOUDNESS || spec.muffle.is_some_and(|m| m > MAX_LOUDNESS) {
        return Err(format!(
            "`toggle_noise` and `muffle` must be at most {MAX_LOUDNESS}"
        ));
    }
    if spec.toggle_noise > 0 && spec.opens_to.is_none() && spec.closes_to.is_none() {
        return Err("only doors have a `toggle_noise`".into());
    }
    Ok(())
}

/// Checks a creature type's sight and combat: it sees 1 to [`VIEW_RADIUS`] cells; `ranged` and
/// `caster` creatures need a `range` of 2 up to their sight, melee creatures have none.
fn check_combat(spec: &creature::CreatureSpec) -> Result<(), String> {
    let id = &spec.id;
    if !(1..=VIEW_RADIUS).contains(&spec.sight) {
        return Err(format!(
            "creature `{id}`: sight {} is out of range (1 to {VIEW_RADIUS})",
            spec.sight
        ));
    }
    match (spec.combat, spec.range) {
        (Combat::Melee, None) => Ok(()),
        (Combat::Melee, Some(_)) => Err(format!(
            "creature `{id}`: only `ranged` and `caster` creatures have a `range`"
        )),
        (_, None) => Err(format!("creature `{id}` needs a `range` to shoot or cast")),
        (_, Some(range)) if !(2..=spec.sight).contains(&range) => Err(format!(
            "creature `{id}`: range {range} is out of range (2 to its sight, {})",
            spec.sight
        )),
        _ => Ok(()),
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
        let fits = match emitter.on {
            ParticleTrigger::Ambient => true,
            ParticleTrigger::Attack => kind == "creature",
            ParticleTrigger::Flight => kind == "object",
            ParticleTrigger::Step | ParticleTrigger::Toggle | ParticleTrigger::Obstacle => {
                kind == "terrain"
            }
            ParticleTrigger::Impact | ParticleTrigger::Linger => false,
        };
        if !fits {
            return Err(err(format!(
                "particles: a {kind} cannot have `{}` emitters",
                emitter.on.name()
            )));
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
            weight = 70000
            [[creature]]
            id = "rat"
            name = "rat"
            glyph = "r"
            fg = "#806040"
            weight = 4000
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
                          stability.intensity = { amount = 0.2, speed = 1.5 }\n\
                          stability.hue = { degrees = 5, speed = 0.4 }\n";

    #[test]
    fn builtin_skills_and_who_trains_them() {
        let content = Content::builtin().expect("content");
        let ids: Vec<&str> = content.skills().iter().map(|s| s.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "appraise",
                "athletics",
                "balance",
                "bluff",
                "concentration",
                "decipher_script",
                "disable_device",
                "escape_artist",
                "handle_animal",
                "heal",
                "intimidate",
                "knowledge",
                "listen",
                "metalcraft",
                "music",
                "open_lock",
                "search",
                "spellcraft",
                "spot",
                "steal",
                "stealth",
                "survival",
                "swim",
                "tumble",
                "use_magic_device",
                "woodcraft"
            ]
        );
        for (id, skill) in content.skill_ids() {
            assert_eq!(content.skill_id(&skill.id), Some(id));
            assert!(!skill.name.is_empty() && !skill.description.is_empty());
        }
        let names = |grants: &[(SkillId, crate::dice::Dice)]| -> Vec<&str> {
            grants
                .iter()
                .map(|&(id, _)| content.skill(id).id.as_str())
                .collect()
        };
        let class = |id: &str| names(&content.class(id).expect("class").skills);
        let race = |id: &str| names(&content.race(id).expect("race").skills);
        assert_eq!(
            class("thief"),
            [
                "appraise",
                "disable_device",
                "escape_artist",
                "open_lock",
                "search",
                "spot",
                "steal",
                "stealth",
                "tumble"
            ]
        );
        assert_eq!(
            class("ranger"),
            [
                "athletics",
                "handle_animal",
                "listen",
                "spot",
                "survival",
                "swim",
                "woodcraft"
            ]
        );
        assert_eq!(
            class("wizard"),
            [
                "concentration",
                "decipher_script",
                "knowledge",
                "spellcraft",
                "use_magic_device"
            ]
        );
        assert_eq!(class("commoner"), ["athletics", "handle_animal"]);
        assert!(content.classes().iter().all(|c| !c.skills.is_empty()));
        assert_eq!(race("elf"), ["listen", "spot"]);
        assert_eq!(race("dwarf"), ["metalcraft", "search"]);
        assert_eq!(race("halfling"), ["listen", "stealth"]);
        assert_eq!(race("gnome"), ["appraise", "listen"]);
        assert_eq!(race("half_orc"), ["intimidate"]);
        assert!(race("human").is_empty());
        assert!(!content.race("human").expect("human").class_skill.is_zero());
        // Creatures may list skills; the player's come from the character.
        let deer = content.creature_id("deer").expect("deer");
        let listen = content.skill_id("listen").expect("listen");
        assert_eq!(content.creature(deer).skills.value(listen), Some(30));
        let player = content.creature_id("player").expect("player");
        assert!(content.creature(player).skills.is_empty());
    }

    #[test]
    fn creature_skills_must_exist_and_the_player_lists_none() {
        let creatures = include_str!("../../../../assets/data/creatures.toml");
        for (from, to, why) in [
            (
                "skills = { listen = 30 }",
                "skills = { tracking = 30 }",
                "creature `deer`: skills: unknown skill `tracking`",
            ),
            (
                "skills = { listen = 30 }",
                "skills = { listen = 1000 }",
                "listen 1000 is more than 999",
            ),
            (
                "hearing = 10\nweight = 70000",
                "hearing = 10\nweight = 70000\nskills = { stealth = 5 }",
                "the player's skills come from their race and class",
            ),
        ] {
            assert!(creatures.contains(from), "{from}");
            let mut files = ContentFiles::builtin();
            files.insert("data/creatures.toml", &creatures.replacen(from, to, 1));
            let err = Content::load(&files).unwrap_err();
            assert!(err.message.contains(why), "{why}: {err}");
        }
        // Without a skills file, nothing may name a skill.
        let mut files = ContentFiles::builtin();
        files.remove("data/skills.toml");
        let err = Content::load(&files).unwrap_err();
        assert!(err.message.contains("unknown skill"), "{err}");
    }

    #[test]
    fn builtin_races_and_classes_load_in_the_order_offered() {
        let content = Content::builtin().expect("content");
        let races: Vec<&str> = (content.races().list.iter())
            .map(|r| r.id.as_str())
            .collect();
        assert_eq!(
            races,
            [
                "elf", "half_elf", "human", "dwarf", "halfling", "gnome", "half_orc"
            ]
        );
        let classes: Vec<&str> = content.classes().iter().map(|c| c.id.as_str()).collect();
        assert_eq!(
            classes,
            [
                "barbarian",
                "bard",
                "priest",
                "druid",
                "monk",
                "paladin",
                "ranger",
                "thief",
                "wizard",
                "witch",
                "warrior",
                "aristocrat",
                "commoner",
                "craftsperson",
                "merchant"
            ]
        );
        for race in &content.races().list {
            assert!(content.vision(&race.vision).is_some(), "{}", race.id);
            assert!(!race.description.is_empty() && !race.name.is_empty());
        }
        for class in content.classes() {
            assert!(!class.description.is_empty() && !class.name.is_empty());
        }
        assert_eq!(
            content.race("dwarf").map(|r| r.vision.as_str()),
            Some("darkvision")
        );
        // Sizes (spec §8 "Burden"): halflings and gnomes are small; dwarves shrug off Burdened.
        let size = |race: &str| {
            let race = content.race(race).expect(race);
            let size = content.size(race.size);
            (size.id.as_str(), size.carry, race.ignores_burdened)
        };
        assert_eq!(size("halfling"), ("small", 75, false));
        assert_eq!(size("gnome"), ("small", 75, false));
        assert_eq!(size("dwarf"), ("medium", 100, true));
        assert_eq!(size("human"), ("medium", 100, false));
        let creature = |id: &str| {
            let def = content.creature(content.creature_id(id).expect(id));
            content.size(def.size).id.as_str()
        };
        assert_eq!((creature("kobold"), creature("deer")), ("small", "medium"));
        assert_eq!(
            content
                .class("wizard")
                .map(|c| c.stats[crate::stats::Stat::Intelligence].to_string()),
            Some("2d2+1".to_string())
        );
        let h = content.horoscope().expect("a horoscope");
        assert_eq!(
            (h.animals.len(), h.elements.len(), h.great_cycle()),
            (12, 5, 60)
        );
        let elements: Vec<&str> = h.elements.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(elements, ["Wood", "Fire", "Earth", "Metal", "Water"]);
        for part in h.animals.iter().chain(&h.elements) {
            assert!(
                !part.description.is_empty() && !part.name.is_empty(),
                "{}",
                part.id
            );
            // The horoscope stays flat and small.
            for (_, by) in part.stats.iter() {
                assert!(by.is_fixed() && by.min().abs() <= 2, "{} {by}", part.id);
            }
        }
        assert!(content.race("orc").is_none() && content.class("necromancer").is_none());
    }

    #[test]
    fn scenario_characters_and_race_eyes_must_exist() {
        let bad = "name = \"B\"\nstart_level = \"hub\"\n\
                   character = { race = \"orc\", gender = \"male\", class = \"warrior\" }\n";
        let mut files = ContentFiles::builtin();
        files.insert("scenarios/bad.toml", bad);
        let err = Content::load(&files).unwrap_err();
        assert_eq!(err.file, "scenarios/bad.toml");
        assert!(
            err.message.contains("character: unknown race `orc`"),
            "{}",
            err.message
        );
        // Content without races has no characters to give.
        let mut files = minimal_files("#@#\n", "");
        files.insert(
            "scenarios/bad.toml",
            &bad.replace("start_level = \"hub\"", "start_level = \"test\""),
        );
        let err = Content::load(&files).unwrap_err();
        assert!(
            err.message.contains("unknown race `orc`"),
            "{}",
            err.message
        );
        // A race's eyes are a known vision profile.
        let mut files = ContentFiles::builtin();
        let races = files.get("data/races.toml").unwrap().replacen(
            "vision = \"darkvision\"",
            "vision = \"tremorsense\"",
            1,
        );
        files.insert("data/races.toml", &races);
        let err = Content::load(&files).unwrap_err();
        assert_eq!(err.file, "data/races.toml");
        assert!(
            err.message
                .contains("race `dwarf`: unknown vision profile `tremorsense`"),
            "{}",
            err.message
        );
    }

    #[test]
    fn builtin_lights_and_vision_profiles_load() {
        let content = Content::builtin().unwrap();
        let ids: Vec<_> = content.vision_profiles().iter().map(|v| &v.id).collect();
        assert_eq!(ids, ["normal", "low_light", "darkvision", "uv_sight"]);
        assert_eq!(content.vision("darkvision").map(|v| v.darkvision), Some(5));
        let torch = content.light(content.light_id("torch").unwrap());
        assert!(!torch.stability.is_stable() && torch.radius > 0);
        let crystal = content.light(content.light_id("crystal_red").unwrap());
        assert!(crystal.stability.is_stable(), "magical light is constant");
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

        files.insert(
            "data/lights.toml",
            "[[light]]\nid = \"fire\"\ncolor = [1, 1, 1, 0]\nradius = 3\n\
             stability.hue = { degrees = 90, speed = 1 }\n",
        );
        let err = Content::load(&files).unwrap_err();
        assert!(
            err.message.contains("light `fire`: stability: hue"),
            "{}",
            err.message
        );

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
        assert_eq!(
            ids,
            [
                "flooded_cell",
                "guard_post",
                "ruined_chapel",
                "ruined_tower",
                "shrine",
                "stone_circle"
            ]
        );
        assert_eq!(content.vaults()[4].entrances.len(), 3);
        let outdoor: Vec<_> = content.vaults().iter().filter(|v| v.is_outdoor()).collect();
        assert_eq!(outdoor.len(), 3);
    }

    #[test]
    fn outdoor_vaults_are_open_but_connected() {
        let load = |map: &str, kind: &str| {
            let mut files = ContentFiles::builtin();
            files.insert(
                "vaults/ruin.toml",
                &format!("name = \"Ruin\"\nkind = \"{kind}\"\nmap = '''\n{map}'''\n"),
            );
            Content::load(&files)
        };
        // Open ground (spaces) and walkable cells on the edge are ways in; no doors needed.
        let vault = load(" ### \n #.  \n ### \n", "outdoor").expect("an open ruin");
        let ruin = vault
            .vaults()
            .iter()
            .find(|v| v.id() == "ruin")
            .expect("ruin");
        assert!(ruin.is_outdoor());
        assert!(ruin.entrances.len() > 5);
        // A walled-in cell is still an error, and so is a ruin with no open edge.
        for (map, expected) in [
            (".####\n.#..#\n.####\n", "cannot be reached"),
            ("###\n###\n###\n", "needs an open cell on its edge"),
        ] {
            let err = load(map, "outdoor").unwrap_err();
            assert!(err.message.contains(expected), "{map}: {}", err.message);
        }
        assert!(load("###\n#.#\n###\n", "overworld").is_err());
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
    fn every_object_has_a_glyph_color_material_and_weight() {
        let content = Content::builtin().unwrap();
        let objects = content.object_defs();
        assert!((15..=40).contains(&objects.len()), "{}", objects.len());
        let glyphs = content.glyphs();
        for def in objects {
            assert!(def.weight > 0 && glyphs.contains(&def.glyph), "{}", def.id);
            assert!(!content.material(def.material).name.is_empty());
            assert_eq!(
                content
                    .object_id(&def.id)
                    .map(|id| content.object(id).id.as_str()),
                Some(def.id.as_str())
            );
        }
        let classes: BTreeSet<char> = objects.iter().map(|o| o.glyph).collect();
        for class in [')', '[', '(', '!', '?', '%', '*', '$', '=', '"'] {
            assert!(classes.contains(&class), "an object drawn `{class}`");
        }
        let materials: BTreeSet<MaterialId> = objects.iter().map(|o| o.material).collect();
        assert!(materials.len() >= 10, "objects of many materials");
        let ruby = content.object(content.object_id("ruby").unwrap());
        assert_eq!(
            (ruby.describe(1), ruby.describe(2)),
            ("a ruby".into(), "2 rubies".into())
        );
        // Every creature has a weight; the hub places objects and its kobold carries some.
        assert!(content.creature_defs().iter().all(|c| c.weight > 0));
        let hub = content.level("hub").unwrap();
        assert!(hub.objects.len() >= 8);
        assert!(hub.creatures.iter().any(|c| !c.carry.is_empty()));
    }

    #[test]
    fn bad_objects_are_reported() {
        const MATERIALS: &str = "[[material]]\nid = \"iron\"\nname = \"iron\"\n";
        let object = |fields: &str| {
            format!(
                "[[object]]\nid = \"key\"\nname = \"key\"\nglyph = \"(\"\nfg = \"#ffffff\"\n{fields}"
            )
        };
        let cases = [
            (object("material = \"iron\"\nweight = 0\n"), "at least 1"),
            (
                object("material = \"bronze\"\nweight = 5\n"),
                "unknown material `bronze`",
            ),
            (object("material = \"iron\"\n"), "missing field `weight`"),
            (object("weight = 5\n"), "missing field `material`"),
            (
                object("material = \"iron\"\nweight = 5\n").replace("\"(\"", "\" \""),
                "visible character",
            ),
            (
                format!("{0}\n{0}", object("material = \"iron\"\nweight = 5\n")),
                "duplicate object id `key`",
            ),
        ];
        for (objects, expected) in cases {
            let mut files = minimal_files("#@#\n", "");
            files.insert("data/materials.toml", MATERIALS);
            files.insert("data/objects.toml", &objects);
            let err = Content::load(&files).unwrap_err();
            assert_eq!(err.file, "data/objects.toml", "{objects}");
            assert!(err.message.contains(expected), "{objects}: {}", err.message);
        }
        let mut files = minimal_files("#@#\n", "");
        let creatures = files
            .get("data/creatures.toml")
            .unwrap()
            .replace("weight = 4000", "weight = 0");
        files.insert("data/creatures.toml", &creatures);
        let err = Content::load(&files).unwrap_err();
        assert!(err.message.contains("creature `rat`"), "{}", err.message);
    }

    #[test]
    fn bad_equipment_is_reported() {
        const MATERIALS: &str = "[[material]]\nid = \"iron\"\nname = \"iron\"\n";
        let slot = |id: &str, action: &str, hand: bool| {
            format!(
                "[[slot]]\nid = \"{id}\"\nname = \"{id}\"\naction = \"{action}\"\n\
                 mark = \"worn\"\nhand = {hand}\n"
            )
        };
        let slots = [
            slot("main_hand", "wield", true),
            slot("off_hand", "wield", true),
            slot("head", "wear", false),
        ]
        .concat();
        let object = |fields: &str| {
            format!(
                "[[object]]\nid = \"key\"\nname = \"key\"\nglyph = \"(\"\nfg = \"#ffffff\"\n\
                 material = \"iron\"\nweight = 5\n{fields}\n"
            )
        };
        let load = |slots: &str, objects: &str| {
            let mut files = minimal_files("#@#\n", "");
            files.insert("data/materials.toml", MATERIALS);
            files.insert("data/objects.toml", objects);
            if !slots.is_empty() {
                files.insert("data/slots.toml", slots);
            }
            Content::load(&files)
        };
        for (fields, expected) in [
            ("slot = \"tail\"", "`slot = \"tail\"` is no slot"),
            ("slot = \"head\"\ntwo_handed = true", "`two_handed` needs"),
            ("two_handed = true", "`two_handed` needs"),
            ("ammo = \"\"", "`ammo` must not be empty"),
            (
                "slot = \"head\"\nlaunches = \"bolt\"\ndamage = \"1d6\"\nrange = 9",
                "launches `bolt`, but no object has",
            ),
            // Ranged combat: missile weapons shoot so far, throwing weapons have damage, and
            // only what flies breaks or is lit.
            (
                "slot = \"head\"\nlaunches = \"bolt\"\ndamage = \"1d6\"",
                "a missile weapon (`launches`) needs a `range`",
            ),
            (
                "range = 4",
                "a throwing weapon (`range`) needs its `damage`",
            ),
            ("damage = \"1d3\"\nrange = 0", "`range` must be 1 to 40"),
            ("damage = \"1d3\"\nrange = 41", "`range` must be 1 to 40"),
            ("ammo = \"bolt\"\nrange = 5", "no `range`"),
            ("reloads = true", "only missile weapons"),
            ("breaks = 20", "`breaks` and `lit` are for things that fly"),
            (
                "ammo = \"bolt\"\nbreaks = 101",
                "`breaks` is a chance in percent",
            ),
            (
                "ammo = \"bolt\"\nlit = \"key\"",
                "must name an object that burns",
            ),
            ("light = \"torch\"", "`light` and `burns` go together"),
            ("burns = 30", "`light` and `burns` go together"),
            ("light = \"sun\"\nburns = 30", "unknown light preset `sun`"),
            ("glow = 2.0", "`glow` must be between 0 and 1"),
            (
                "launches = \"bolt\"",
                "a missile weapon (`launches`) needs its `damage`",
            ),
            // Combat: weapons have damage, and only things worn or wielded fight or protect.
            ("damage = \"1d4\"", "a weapon (`damage`) must go in a slot"),
            (
                "slot = \"main_hand\"\nthreat = 20",
                "`threat` is for weapons",
            ),
            ("slot = \"main_hand\"\ncrit = 3", "`crit` is for weapons"),
            (
                "slot = \"main_hand\"\nstrength = 12",
                "`strength` is for weapons",
            ),
            (
                "slot = \"main_hand\"\ndamage = \"1d4\"\nthreat = 1",
                "`threat` must be 2 to 24",
            ),
            (
                "slot = \"main_hand\"\ndamage = \"1d4\"\ncrit = 6",
                "`crit` 2 to 5",
            ),
            (
                "slot = \"main_hand\"\ndamage = \"1d4\"\nto_hit = 21",
                "`to_hit` must be within",
            ),
            (
                "slot = \"main_hand\"\ndamage = \"1d4\"\nstrength = 41",
                "`strength` is a Strength score",
            ),
            (
                "slot = \"main_hand\"\ndamage = \"2d\"",
                "number of sides is missing",
            ),
            ("armour = 2", "are for things worn or wielded"),
            ("max_dex = 3", "are for things worn or wielded"),
            ("slot = \"head\"\nshield = -1", "`shield` must be 0 to 20"),
            ("slot = \"head\"\narmour = 21", "`armour` must be 0 to 20"),
        ] {
            let err = load(&slots, &object(fields)).unwrap_err();
            assert_eq!(err.file, "data/objects.toml", "{fields}");
            assert!(err.message.contains(expected), "{fields}: {}", err.message);
        }
        let staff = object("slot = \"main_hand\"\ntwo_handed = true");
        assert!(load(&slots, &staff).is_ok());
        // Two hands are needed for a two-handed object; no slots file, no slots.
        let one_hand = slot("main_hand", "wield", true);
        assert!(load(&one_hand, &staff).is_err());
        let err = load("", &object("slot = \"head\"")).unwrap_err();
        assert!(err.message.contains("is no slot"), "{}", err.message);
        // A launcher pairs with the ammunition of its kind.
        let pair = format!(
            "{}{}",
            object("slot = \"head\"\nlaunches = \"bolt\"\ndamage = \"1d10+1\"\nrange = 12"),
            object("ammo = \"bolt\"").replace("\"key\"", "\"bolt\"")
        );
        let content = load(&slots, &pair).expect("a crossbow and a bolt");
        let (key, bolt) = (content.object_id("key"), content.object_id("bolt"));
        assert!(content.fires(key.unwrap(), bolt.unwrap()));
        assert!(!content.fires(bolt.unwrap(), key.unwrap()));
        // Slots enter the fingerprint: equipment is saved by slot index.
        let without = load("", &object("")).expect("loads");
        let with = load(&slots, &object("")).expect("loads");
        assert_ne!(without.fingerprint(), with.fingerprint());
    }

    #[test]
    fn creature_types_have_stats_that_default_to_average() {
        use crate::stats::{Stat, Stats};
        let content = Content::builtin().unwrap();
        let def = |id: &str| content.creature(content.creature_id(id).unwrap());
        assert_eq!(
            def("player").stats,
            Stats::default(),
            "the scenario gives the player's"
        );
        let rat = def("rat").stats;
        assert_eq!(rat.score(Stat::Dexterity), 17);
        assert_eq!(
            rat.potential(Stat::Dexterity),
            17,
            "the potential defaults to the score"
        );
        assert_eq!(rat.score(Stat::Intelligence), 1);
        let shaman = def("kobold_shaman").stats;
        assert!(shaman.score(Stat::Willpower) > def("kobold").stats.score(Stat::Willpower));
        // A creature type without stats is average in each.
        let content = Content::load(&minimal_files("#@#\n", "")).unwrap();
        let rat = content.creature(content.creature_id("rat").unwrap());
        assert_eq!(rat.stats, Stats::default());
    }

    #[test]
    fn bad_creature_stats_are_reported() {
        let cases = [
            (
                "rat",
                "stats = { strength = 41 }",
                "creature `rat`: stats: strength 41",
            ),
            (
                "rat",
                "stats = { presence = 0 }",
                "presence 0 is out of range (1 to 40)",
            ),
            (
                "rat",
                "potential = { toughness = 50 }",
                "potential: toughness 50",
            ),
            (
                "rat",
                "stats = { dexterity = 16 }\npotential = { dexterity = 12 }",
                "dexterity 12 is below its score 16",
            ),
            ("rat", "stats = { luck = 3 }", "unknown stat `luck`"),
            (
                "player",
                "stats = { strength = 12 }",
                "the player's stats and hit points come from the scenario",
            ),
            (
                "player",
                "potential = { strength = 12 }",
                "come from the scenario",
            ),
            ("player", "base_hp = 3", "come from the scenario"),
            ("rat", "level = 0", "`level` must be 1 to 99"),
            ("rat", "level = 100", "`level` must be 1 to 99"),
            ("rat", "xp = 100001", "`xp` at most 100000"),
            (
                "rat",
                "attack = { damage = \"1d4\", name = \"bite\", verb = \"bites\", threat = 25 }",
                "creature `rat`: attack: `threat` must be 2 to 24",
            ),
            (
                "rat",
                "attack = { damage = \"1d4\", name = \"\", verb = \"bites\" }",
                "`name` and `verb` must not be empty",
            ),
            (
                "rat",
                "attack = { damage = \"1d1\", name = \"bite\", verb = \"bites\" }",
                "2 to 1000 sides",
            ),
            (
                "rat",
                "natural = { defence = 0, name = \"hide\" }",
                "natural: `defence` must be 1 to 20",
            ),
            // Without data/sizes.toml there is only medium.
            (
                "rat",
                "size = \"small\"",
                "creature `rat`: unknown size `small`",
            ),
        ];
        for (creature, fields, expected) in cases {
            let mut files = minimal_files("#@#\n", "");
            let marker = format!("id = \"{creature}\"\n");
            let creatures = files
                .get("data/creatures.toml")
                .unwrap()
                .replace(&marker, &format!("{marker}{fields}\n"));
            files.insert("data/creatures.toml", &creatures);
            let err = Content::load(&files).unwrap_err();
            assert_eq!(err.file, "data/creatures.toml", "{fields}");
            assert!(err.message.contains(expected), "{fields}: {}", err.message);
        }
        let mut files = minimal_files("#@#\n", "");
        files.insert(
            "scenarios/test.toml",
            "name = \"T\"\nstart_level = \"test\"\nstats = { willpower = 99 }\n",
        );
        let err = Content::load(&files).unwrap_err();
        assert_eq!(err.file, "scenarios/test.toml");
        assert!(err.message.contains("willpower 99"), "{}", err.message);
    }

    #[test]
    fn levels_place_objects_by_legend_and_by_pile() {
        let map = "#####\n#.d.#\n#####\n";
        let legend = "\"d\" = { terrain = \"floor\", objects = [\"dagger\", \"arrow*12\", \"dagger*2\"] }\n\
                      [[pile]]\nat = [3, 1]\nobjects = [\"gold_coin*5\", \"gold_coin*2\"]\n\
                      [[pile]]\nat = [2, 1]\nobjects = [\"ruby\"]\n";
        let content = stream(map, legend).unwrap();
        let level = content.level("stream").unwrap();
        let id = |name: &str| content.object_id(name).unwrap();
        let stack = |name: &str, count| crate::object::Stack {
            object: id(name),
            count,
        };
        assert_eq!(
            level.objects[&Pos::new(2, 1)],
            [
                stack("ruby", 1),
                stack("dagger", 1),
                stack("arrow", 12),
                stack("dagger", 1),
                stack("dagger", 1)
            ],
            "the pile goes on top; daggers do not stack"
        );
        assert_eq!(level.objects[&Pos::new(3, 1)], [stack("gold_coin", 7)]);
    }

    #[test]
    fn misplaced_objects_are_reported() {
        let cases = [
            (
                "\"x\" = { terrain = \"floor\", objects = [\"anvil\"] }",
                "unknown object `anvil`",
            ),
            (
                "\"x\" = { terrain = \"floor\", objects = [\"arrow*0\"] }",
                "count must be",
            ),
            (
                "\"x\" = { terrain = \"floor\", objects = [\"arrow*x\"] }",
                "count must be",
            ),
            (
                "\"x\" = { terrain = \"wall\", objects = [\"dagger\"] }",
                "not walkable",
            ),
            (
                "\"x\" = { terrain = \"floor\", carry = [\"dagger\"] }",
                "carry, wield, or wear but no creature",
            ),
            (
                "\"x\" = { terrain = \"floor\", creature = \"kobold\", carry = [\"dagger*27\"] }",
                "at most 26 stacks",
            ),
            (
                "\"x\" = \"floor\"\n[[pile]]\nat = [9, 9]\nobjects = [\"dagger\"]",
                "outside",
            ),
            (
                "\"x\" = \"floor\"\n[[pile]]\nat = [0, 0]\nobjects = [\"dagger\"]",
                "not walkable",
            ),
            (
                "\"x\" = \"floor\"\n[[pile]]\nat = [1, 1]\nobjects = []",
                "has no objects",
            ),
        ];
        for (legend, expected) in cases {
            let err = stream("###\n#x#\n###\n", &format!("{legend}\n")).unwrap_err();
            assert_eq!(err.file, "levels/stream.toml");
            assert!(err.message.contains(expected), "{legend}: {}", err.message);
        }
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
