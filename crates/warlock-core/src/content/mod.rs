//! Game content: definitions, levels, scenarios, dungeon branches, and vaults loaded from TOML
//! (spec §7).
//!
//! Content is read from a [`ContentFiles`] set: the built-in set compiled into the binary (so every
//! build, including the web build, is self-contained) or a directory on disk for development.

mod action;
mod affix;
mod alignment;
mod anim;
mod calendar;
mod class;
mod creature;
mod damage_type;
mod dialogue;
mod disease;
mod dungeon;
mod effect;
mod experience;
mod faction;
mod flow;
mod gem;
mod horoscope;
mod hunger;
mod item_class;
mod level;
mod light;
mod material;
mod names;
mod object;
mod overworld;
pub(crate) mod part;
mod particles;
mod quest;
mod race;
mod room_effect;
mod scenario;
mod scroll;
mod search;
mod shop;
mod size;
mod skill;
mod sky;
mod slot;
mod spell;
mod stance;
mod status;
mod terrain;
mod trap;
mod vault;
mod vision;
mod weapon_skill;
mod weather;

pub use action::{ActionCosts, ActionKind, BASE_TURN, COSTS, NORMAL_COST, price};
pub use affix::{
    AffixDef, AffixKind, Affixes, Effect, Group, ItemKind, MAX_ITEM_LEVEL, MAX_PER_KIND, MAX_SPEED,
    Rarity, Tier,
};
pub use alignment::{AlignmentRules, BandDef, ByAlignment, Shifts};
pub use anim::GlyphAnim;
pub use calendar::Calendar;
pub use class::ClassDef;
pub use creature::{
    Budge, Combat, CreatureDef, CreatureId, DEFAULT_SIGHT, Hide, NORMAL_SPEED, NaturalAttack,
    OnEat, SPEEDS,
};
pub use damage_type::{DamageTypeDef, DamageTypeId, MAX_DAMAGE_TYPES};
pub use dialogue::{
    Dialogue, DialogueId, MAX_LINE, MAX_REPLY, MAX_TOPICS, MUTTER_CHANCES, Mutter, Speaker, Topic,
};
pub use disease::{DIFFICULTIES, DiseaseDef, DiseaseId, SOURCES, Wading};
pub use dungeon::{DungeonDef, Generator, LevelPlan, parse_branch_level};
pub use effect::{
    Darken, EffectDef, EffectLight, EffectShape, FILL_GLYPH, MAX_BURST_RADIUS, MAX_DARKEN,
    MAX_SHAKE_PIXELS, Shake,
};
pub use experience::{
    Discovery, Experience, FACTORS, Factor, Guideline, Kills, MAX_LEVEL, MAX_XP, pace_words,
};
pub use faction::{FactionDef, FactionId, Factions, PLAYER_FACTION, Stance};
pub use flow::{FLOW_SPEEDS, Flow};
pub use gem::{
    Cut, CutId, Cutting, GemDef, GemSize, GemSizeId, Gems, MAX_GEM_VALUE, MAX_GEM_WEIGHT,
    MAX_MULTIPLIER, Ornament, ROUGH_SATURATION, dull,
};
pub use horoscope::{Horoscope, Sign, SignPart};
pub use hunger::{
    Corpses, EXERTIONS, Faint, Food, HungerDef, MAX_MEAL, MAX_NUTRITION, MAX_SHELF_LIFE,
    Overeating, Rotten, Starve, States,
};
#[cfg(test)]
pub(crate) use item_class::BUILTIN as BUILTIN_ITEM_CLASSES;
pub use item_class::{ItemClassDef, ItemClassId, MAX_ITEM_CLASSES};
pub(crate) use level::validate_route;
pub use level::{
    CreatureSpawn, EdgeExit, LevelKind, LevelTemplate, Region, StairsLink, parse_route,
};
pub use light::{Dips, Drift, HueDrift, LightId, LightPreset, MAX_LIGHT_RADIUS, Stability};
pub use material::{MAX_MATERIAL_VALUE, MaterialDef, MaterialId};
pub use names::{NameLists, Names};
pub use object::{
    MAX_BONUS, MAX_DIG, MAX_PLACED, MAX_VALUE, ObjectDef, ObjectId, article, counted, weapon_noun,
};
pub use overworld::{Landscape, LocalArea, local_area_id, parse_local_area};
pub use part::{MAX_OBJECT_PARTS, MAX_PARTS, Part, PartDef, PartId};
pub use particles::{ParticleEmitter, ParticleTrigger};
pub use quest::{
    DEFAULT_DECLINE, GOLD, Goal, MAX_COUNT, MAX_NAME, MAX_QUEST_XP, QuestDef, QuestId, QuestKind,
    QuestTexts, Reward,
};
pub use race::{HUNGER, RaceDef, Races};
pub use room_effect::{RoomEffectDef, RoomEffectId, RoomEffects, SpecialRoom};
pub use scenario::ScenarioDef;
pub use scroll::{Cast, Reading, ScrollSpell};
pub use search::{
    PASSIVE_PENALTIES, PassiveSearch, SEARCH_TURNS, SearchRules, Secret, secret_look,
};
pub use shop::{KEEPER, Keeper, MAX_FAVOUR, MAX_SHOPS, ShopDef, ShopId, ShopPlan, Shops};
pub use size::{CARRY, MEDIUM, SizeDef};
pub use skill::{SkillBook, SkillDef};
pub use sky::{MAX_MOONS, MAX_SIZE, MoonDef, PHASE_ONE, STANDARD_SIZE};
pub use slot::{MAX_SLOTS, SlotDef, SlotId};
pub use spell::{BOLT_LENGTHS, BURST_RADII, Bolt, Burst, BurstIce};
pub use stance::{CombatStance, StanceDef, Stances, percent_of};
pub use status::{
    Affliction, COST_PERCENT, CombatBonus, Healing, MAX_REGEN, MAX_SEVERITY, OnHit, Penalty,
    Stacking, StatusDef, StatusId, Wound,
};
pub use terrain::{
    Breakable, Freeze, FreezeSpec, MAX_HARDNESS, MAX_HEIGHT, StairsDir, TerrainDef, TerrainId,
};
pub use trap::{MAX_CHARGES, Trap, TrapDef, TrapEffect, TrapId, TrapNoise};
pub use vault::Vault;
pub use vision::VisionProfile;
pub use weapon_skill::{
    Benefit, Grip, MAX_RANK, MarkRules, RANKS, Role, Section, Style, TwoWeapons, WeaponSkillDef,
    WeaponSkillId, WeaponSkillTable,
};
pub use weather::{
    Climate, MAX_CHILL, MAX_SIGHT_LOSS, MAX_SWING, MILD, MILD_SWING, MONTH_TEMPERATURES, WeatherDef,
};

use crate::dice::Dice;
use crate::noise::MAX_LOUDNESS;
use crate::perception::VIEW_RADIUS;
use crate::skills::{Increase, SkillId, Skills};
use crate::stats::Stats;
use crate::weapon_skills::WeaponSkills;
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

    /// Reads the `data/`, `levels/`, `scenarios/`, and `vaults/` TOML files under `root`,
    /// subdirectories included (`data/creatures/kobold.toml`): the same set, under the same
    /// paths, as `build.rs` embeds.
    pub fn from_dir(root: &Path) -> std::io::Result<Self> {
        fn collect(dir: &Path, rel: &str, files: &mut ContentFiles) -> std::io::Result<()> {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return Ok(());
            };
            for entry in entries {
                let path = entry?.path();
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if path.is_dir() {
                    collect(&path, &format!("{rel}/{name}"), files)?;
                } else if path.extension().is_some_and(|e| e == "toml") {
                    files.insert(&format!("{rel}/{name}"), &std::fs::read_to_string(&path)?);
                }
            }
            Ok(())
        }
        let mut files = Self::default();
        for dir in ["data", "levels", "scenarios", "vaults"] {
            collect(&root.join(dir), dir, &mut files)?;
        }
        Ok(files)
    }

    pub fn insert(&mut self, path: &str, text: &str) {
        self.files.insert(path.to_string(), text.to_string());
    }

    /// Leaves a file out, e.g. `data/weather.toml` for an always clear sky, or a whole directory
    /// (`data/creatures/`, with or without the slash) for a test that brings its own table.
    pub fn remove(&mut self, path: &str) {
        self.files.remove(path);
        let dir = format!("{}/", path.trim_end_matches('/'));
        self.files.retain(|p, _| !p.starts_with(&dir));
    }

    /// The files of the content table `name`, in the order its entries are numbered:
    /// `data/<name>.toml` first, if there is one, then every file under `data/<name>/`
    /// (subdirectories included) in path order. Each is `(path, text)`.
    pub fn table<'a>(&'a self, name: &str) -> impl Iterator<Item = (&'a str, &'a str)> {
        let single = format!("data/{name}.toml");
        let dir = format!("data/{name}/");
        let first = self.files.get_key_value(&single);
        let rest = self
            .files
            .range(dir.clone()..)
            .take_while(move |(p, _)| p.starts_with(&dir))
            .filter(|(p, _)| p.ends_with(".toml"));
        (first.into_iter().chain(rest)).map(|(p, t)| (p.as_str(), t.as_str()))
    }

    /// Rewrites every file of table `name` with `edit` (tests: a built-in creature with a line
    /// changed, whichever file it is in).
    pub fn edit_table(&mut self, name: &str, mut edit: impl FnMut(&str) -> String) {
        let edited: Vec<(String, String)> = (self.table(name))
            .map(|(p, t)| (p.to_string(), edit(t)))
            .collect();
        for (path, text) in edited {
            self.insert(&path, &text);
        }
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

/// One file of the creature table (`data/creatures.toml` or `data/creatures/*.toml`): any number
/// of entries, none in a file of comments only (`README.toml`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreatureFile {
    #[serde(default)]
    creature: Vec<creature::CreatureSpec>,
}

/// One file of the material table.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MaterialFile {
    #[serde(default)]
    material: Vec<MaterialDef>,
}

/// One file of the object table.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectFile {
    #[serde(default)]
    object: Vec<object::ObjectSpec>,
}

/// The entries of content table `name` (`data/<name>.toml`, then `data/<name>/**/*.toml` in path
/// order; `ContentFiles::table`), each with the file it came from, so an error names it. A file
/// that does not parse is an error naming it; `entries` takes a parsed file apart.
fn load_table<'a, F: serde::de::DeserializeOwned, T>(
    files: &'a ContentFiles,
    name: &str,
    entries: impl Fn(F) -> Vec<T>,
) -> Result<Vec<(&'a str, T)>, ContentError> {
    let mut all = Vec::new();
    for (path, text) in files.table(name) {
        let parsed: F = toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))?;
        all.extend(entries(parsed).into_iter().map(|e| (path, e)));
    }
    Ok(all)
}

/// The error for an id defined twice in a table: in `path`, and before that in `first` (another
/// file of the table, or the same one).
fn duplicate_id(kind: &str, id: &str, path: &str, first: &str) -> ContentError {
    let also = if first == path {
        String::new()
    } else {
        format!(" (also in {first})")
    };
    ContentError::new(path, format!("duplicate {kind} id `{id}`{also}"))
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
/// `data/calendar.toml`, `data/sky.toml`, `data/materials/`, `data/parts.toml`, `data/objects/`,
/// `data/gems.toml`, `data/weather.toml`, `data/actions.toml`, `data/slots.toml`, `data/sizes.toml`,
/// `data/races.toml`, `data/names.toml`, `data/classes.toml`, `data/skills.toml`,
/// `data/horoscope.toml`,
/// `data/effects.toml`, `data/room_effects.toml`, `data/weapon_skills.toml`, `data/stances.toml`,
/// `data/status.toml`, `data/diseases.toml`, `data/dialogue.toml`, `data/quests.toml`,
/// `data/traps.toml`, and `data/shops.toml` are optional (without effects, spell effects are not
/// drawn; without traps or shops there are none). Without vision
/// profiles, a single `normal` profile that needs no light is used, so content without lighting
/// stays fully visible; without sizes, everything is medium; without factions, everyone is neutral; without a calendar, months are numbered;
/// without a sky file there are no moons; without part names every object is of one unnamed part
/// (`materials = ["steel"]`); without `data/gems.toml` there are no gems to find or set; without
/// an objects file there are no objects, and
/// without `data/item_classes.toml` there may be none (each names its class); without a
/// weather file the sky is always clear; without an actions file every action costs 1000;
/// without a slots file nothing can be worn or wielded; without races or classes there is no
/// character to choose, and scenarios give the player's eyes and stats themselves; without
/// names a character told none is nameless (`character::NAMELESS`); without
/// skills nobody is trained in anything; without a horoscope, characters are born under no sign;
/// without room effects there are no special rooms; without weapon skills nobody has any, and
/// weapons name no category; without statuses nothing ails anyone, and resting heals twice as
/// fast; without diseases nobody falls ill; without dialogue, nobody talks; without quests,
/// nobody asks anything of the player; without `data/hunger.toml` nobody hungers, nothing is
/// eaten, food does not rot, and creatures leave no corpses.
#[derive(Debug)]
pub struct Content {
    terrain: Vec<TerrainDef>,
    terrain_ids: BTreeMap<String, TerrainId>,
    /// The terrain ice looks like, by the thinnest ice that looks like it (mm), thinnest first.
    ice: Vec<(u16, TerrainId)>,
    creatures: Vec<CreatureDef>,
    creature_ids: BTreeMap<String, CreatureId>,
    /// Damage types (`data/damage_types.toml`), in file order; none without the file.
    damage_types: Vec<DamageTypeDef>,
    materials: Vec<MaterialDef>,
    material_ids: BTreeMap<String, MaterialId>,
    /// Part names (`data/parts.toml`), in file order.
    parts: Vec<PartDef>,
    /// Item classes, in the order item lists group by.
    item_classes: Vec<ItemClassDef>,
    objects: Vec<ObjectDef>,
    object_ids: BTreeMap<String, ObjectId>,
    /// Gem sizes, cuts, and ornaments (`data/gems.toml`); none without the file.
    gems: Option<Gems>,
    /// Equipment slots, in display order.
    slots: Vec<SlotDef>,
    /// Trap types, in file order.
    traps: Vec<TrapDef>,
    trap_ids: BTreeMap<String, TrapId>,
    /// The numbers of searching for hidden features (`data/search.toml`, spec §8 "Searching").
    search: SearchRules,
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
    /// Names offered to new characters, by race (`data/names.toml`): flavour, not fingerprinted.
    names: Names,
    classes: Vec<ClassDef>,
    /// The alignment rules (`data/alignment.toml`): none without it.
    alignment: Option<AlignmentRules>,
    experience: Experience,
    horoscope: Option<Horoscope>,
    /// Spell effects, in `data/effects.toml` order: presentation only.
    effects: Vec<EffectDef>,
    hunger: Option<HungerDef>,
    /// The corpse object of each creature type, by creature id (`None`: it leaves none).
    corpses: Vec<Option<ObjectId>>,
    weapon_skills: WeaponSkillTable,
    /// Special rooms' effects (`data/room_effects.toml`).
    room_effects: RoomEffects,
    /// Prefixes, suffixes, and how loot rolls them (`data/affixes.toml`).
    affixes: Affixes,
    stances: Stances,
    /// In `data/status.toml` order: most important first.
    statuses: Vec<StatusDef>,
    status_ids: BTreeMap<String, StatusId>,
    healing: Healing,
    /// In `data/diseases.toml` order.
    diseases: Vec<DiseaseDef>,
    disease_ids: BTreeMap<String, DiseaseId>,
    /// The status diseases show as ("Diseased"), with any disease.
    disease_status: Option<StatusId>,
    /// What creatures say (`data/dialogue.toml`), in file order.
    dialogues: Vec<Dialogue>,
    /// What people ask of the player (`data/quests.toml`), in file order.
    quests: Vec<QuestDef>,
    /// Shops (`data/shops.toml`); none without the file.
    shops: Option<Shops>,
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
            ice: Vec::new(),
            creatures: Vec::new(),
            creature_ids: BTreeMap::new(),
            damage_types: Vec::new(),
            materials: Vec::new(),
            material_ids: BTreeMap::new(),
            parts: Vec::new(),
            item_classes: Vec::new(),
            objects: Vec::new(),
            object_ids: BTreeMap::new(),
            gems: None,
            slots: Vec::new(),
            traps: Vec::new(),
            trap_ids: BTreeMap::new(),
            search: SearchRules::default(),
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
            names: Names::default(),
            classes: Vec::new(),
            alignment: None,
            experience: Experience::default(),
            horoscope: None,
            effects: Vec::new(),
            hunger: None,
            corpses: Vec::new(),
            weapon_skills: WeaponSkillTable::default(),
            room_effects: RoomEffects::default(),
            affixes: Affixes::default(),
            stances: Stances::default(),
            statuses: Vec::new(),
            status_ids: BTreeMap::new(),
            healing: Healing::default(),
            diseases: Vec::new(),
            disease_ids: BTreeMap::new(),
            disease_status: None,
            dialogues: Vec::new(),
            quests: Vec::new(),
            shops: None,
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
        let path = "data/weapon_skills.toml";
        if let Some(text) = files.get_optional(path) {
            content.weapon_skills = WeaponSkillTable::parse(path, text)?;
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
        let path = "data/damage_types.toml";
        if let Some(text) = files.get_optional(path) {
            content.damage_types = damage_type::parse(path, text)?;
        }
        // The affix table: optional, so loot without it is always normal.
        if files.table("affixes").next().is_some() {
            let light = |id: &str| content.light_id(id);
            let skill = |id: &str| content.skill_id(id);
            let has_skills = !content.skills().is_empty();
            let damage = |id: &str| content.damage_type_id(id);
            let has_types = !content.damage_types().is_empty();
            let table = files.table("affixes");
            let typed = (damage, has_types);
            content.affixes = affix::parse(table, light, (skill, has_skills), typed)?;
        }
        let path = "data/stances.toml";
        if let Some(text) = files.get_optional(path) {
            content.stances = Stances::parse(path, text)?;
        }
        let path = "data/races.toml";
        if let Some(text) = files.get_optional(path) {
            let has_vision = |id: &str| content.vision(id).is_some();
            let size_index = |id: &str| content.size_index(id);
            let skill_id = |id: &str| content.skill_id(id);
            let weapons = &content.weapon_skills;
            let percent = crate::damage::MIN_RESISTANCE..=crate::damage::IMMUNE;
            let resist = |m: &BTreeMap<String, i32>| {
                content.resolve_per_kind("resist", m, percent.clone(), false)
            };
            content.races = Races::parse(
                path, text, has_vision, size_index, skill_id, weapons, resist,
            )?;
        }
        let path = "data/names.toml";
        if let Some(text) = files.get_optional(path) {
            content.names = Names::parse(path, text, |id| content.race(id).is_some())?;
        }
        let path = "data/classes.toml";
        if let Some(text) = files.get_optional(path) {
            let skill_id = |id: &str| content.skill_id(id);
            content.classes = ClassDef::parse(path, text, skill_id, &content.weapon_skills)?;
        }
        let path = "data/alignment.toml";
        if let Some(text) = files.get_optional(path) {
            content.alignment = Some(AlignmentRules::parse(path, text)?);
        }
        let path = "data/experience.toml";
        if let Some(text) = files.get_optional(path) {
            content.experience = Experience::parse(path, text)?;
        }
        let path = "data/horoscope.toml";
        if let Some(text) = files.get_optional(path) {
            content.horoscope = Some(Horoscope::parse(path, text)?);
        }
        content.load_statuses(files)?;
        content.load_terrain(files)?;
        content.load_diseases(files)?;
        let path = "data/room_effects.toml";
        if let Some(text) = files.get_optional(path) {
            let terrain = |name: &str| {
                let id = content.terrain_id(name)?;
                Some((id, content.terrain(id)))
            };
            content.room_effects = RoomEffects::parse(path, text, terrain)?;
        }
        content.load_factions(files)?;
        content.load_creatures(files)?;
        content.load_objects(files)?;
        content.load_gems(files)?;
        let path = "data/shops.toml";
        if let Some(text) = files.get_optional(path) {
            content.shops = Some(Shops::parse(path, text, &content)?);
        }
        let path = "data/effects.toml";
        if let Some(text) = files.get_optional(path) {
            content.effects = effect::parse(path, text)?;
        }
        scroll::check_spells(&content).map_err(|m| ContentError::new("data/objects", m))?;
        let path = "data/traps.toml";
        if let Some(text) = files.get_optional(path) {
            let (traps, ids) = trap::parse(path, text, &content)?;
            (content.traps, content.trap_ids) = (traps, ids);
        }
        let path = "data/search.toml";
        if let Some(text) = files.get_optional(path) {
            content.search = SearchRules::parse(path, text)?;
        }
        let path = "data/dialogue.toml";
        if let Some(text) = files.get_optional(path) {
            content.dialogues = dialogue::parse(path, text, &content)?;
        }
        content.load_hunger(files)?;

        let path = "data/legend.toml";
        let legend_file: LegendFile =
            toml::from_str(files.get(path)?).map_err(|e| ContentError::new(path, e.to_string()))?;
        let mut default_legend = BTreeMap::new();
        for (key, entry) in legend_file.legend {
            default_legend.insert(legend_char(path, &key)?, entry);
        }

        for (path, id, text) in files.in_dir("levels") {
            let level = level::parse_level(path, id, text, &content, &default_legend)?;
            if let Some((at, _)) = level.loot.first() {
                return Err(ContentError::new(
                    path,
                    format!("`loot` at {at}: loot is rolled only in vaults"),
                ));
            }
            content.levels.insert(id.to_string(), level);
        }
        for (path, id, text) in files.in_dir("scenarios") {
            let mut scenario = ScenarioDef::parse(path, id, text)?;
            let err = |m: String| ContentError::new(path, format!("carry: {m}"));
            scenario.carry = object::parse_stacks(&scenario.carry_spec, &content).map_err(err)?;
            content.scenarios.insert(id.to_string(), scenario);
        }
        for (path, id, text) in files.in_dir("vaults") {
            let template = level::parse_level(path, id, text, &content, &default_legend)?;
            let vault = Vault::new(template, &content).map_err(|m| ContentError::new(path, m))?;
            content.vaults.push(vault);
        }
        content.load_dungeons(files)?;
        let path = "data/quests.toml";
        if let Some(text) = files.get_optional(path) {
            content.quests = quest::parse(path, text, &content)?;
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
            check_terrain_ice(spec).map_err(|m| err(format!("terrain `{}`: {m}", spec.id)))?;
            check_kicking(spec).map_err(|m| err(format!("terrain `{}`: {m}", spec.id)))?;
            let breaks = match (spec.hardness, &spec.becomes) {
                (None, None) => None,
                (Some(hardness), Some(_)) if (1..=MAX_HARDNESS).contains(&hardness) => {
                    Some(Breakable {
                        hardness,
                        becomes: resolve(&spec.becomes)?.expect("given"),
                    })
                }
                (Some(_), Some(_)) => {
                    return Err(err(format!(
                        "terrain `{}`: hardness must be 1 to {MAX_HARDNESS}",
                        spec.id
                    )));
                }
                _ => {
                    return Err(err(format!(
                        "terrain `{}`: `hardness` and `becomes` go together",
                        spec.id
                    )));
                }
            };
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
                variants: spec.variants.chars().collect(),
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
                kicked_in: match (resolve(&spec.breaks_to)?, spec.sturdiness) {
                    (Some(to), Some(sturdiness)) => Some((to, sturdiness)),
                    _ => None,
                },
                stairs: spec.stairs,
                light,
                bounce: spec.bounce,
                height,
                roofed: spec.roofed,
                stains: spec.stains,
                stone: spec.stone,
                step_noise: spec.step_noise,
                toggle_noise: spec.toggle_noise,
                // Solid terrain blocks sound unless it muffles it; everything else passes it.
                muffle: spec
                    .muffle
                    .or((spec.walkable || spec.transparent).then_some(0)),
                freeze: match &spec.freeze {
                    None => None,
                    Some(f) => {
                        let kind = match &f.damage_type {
                            None => None,
                            Some(name) => Some(self.damage_type_id(name).ok_or_else(|| {
                                ContentError::new(
                                    path,
                                    format!(
                                        "terrain `{}`: `freeze`: unknown damage type `{name}`",
                                        spec.id
                                    ),
                                )
                            })?),
                        };
                        Some(f.resolve(kind))
                    }
                },
                ice: spec.ice,
                breaks,
                face: resolve(&spec.face)?,
                anim: spec.anim.clone(),
                glow: spec.glow,
                haze: spec.haze,
                particles: spec.particles.clone(),
            });
        }
        // A broken door is open for good: walkable, and no door to open or close.
        if let Some(def) = defs.iter().find(|d| {
            d.kicked_in.is_some_and(|(to, _)| {
                let to = &defs[usize::from(to.0)];
                !to.walkable || to.opens_to.is_some() || to.closes_to.is_some()
            })
        }) {
            return Err(err(format!(
                "terrain `{}`: what it `breaks_to` must be walkable and no door",
                def.id
            )));
        }
        for def in &defs {
            check_breaking(def, &defs).map_err(|m| err(format!("terrain `{}`: {m}", def.id)))?;
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
        let mut ice: Vec<(u16, TerrainId)> = (defs.iter().enumerate())
            .filter_map(|(i, d)| d.ice.map(|mm| (mm, TerrainId(i as u16))))
            .collect();
        ice.sort();
        if let Some(pair) = ice.windows(2).find(|w| w[0].0 == w[1].0) {
            return Err(err(format!(
                "terrain `{}` and `{}` both look like ice from {} mm",
                defs[usize::from(pair[0].1.0)].id,
                defs[usize::from(pair[1].1.0)].id,
                pair[0].0
            )));
        }
        if let Some(water) = defs.iter().find(|d| d.freeze.is_some())
            && ice.is_empty()
        {
            return Err(err(format!(
                "terrain `{}` freezes, but no terrain looks like ice (`ice`)",
                water.id
            )));
        }
        self.terrain = defs;
        self.ice = ice;
        Ok(())
    }

    /// The creature table: `data/creatures.toml` and `data/creatures/*.toml`, numbered in path
    /// order (the single file first); required, with a `player`.
    fn load_creatures(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let Some((first_file, _)) = files.table("creatures").next() else {
            return Err(ContentError::new("data/creatures.toml", "file is missing"));
        };
        let specs = load_table(files, "creatures", |f: CreatureFile| f.creature)?;
        let mut seen: BTreeMap<String, &str> = BTreeMap::new();
        for (i, (path, spec)) in specs.into_iter().enumerate() {
            let err = |m: String| ContentError::new(path, m);
            let id = CreatureId(u16::try_from(i).map_err(|_| err("too many creatures".into()))?);
            if let Some(first) = seen.insert(spec.id.clone(), path) {
                return Err(duplicate_id("creature", &spec.id, path, first));
            }
            self.creature_ids.insert(spec.id.clone(), id);
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
            // Without races (no characters) a creature's race means nothing, and is not checked.
            let unknown = |r: &&str| self.race(r).is_none() && !self.races.list.is_empty();
            if let Some(race) = spec.race.as_deref().filter(unknown) {
                return Err(err(format!(
                    "creature `{}`: unknown race `{race}`",
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
            if spec.id == "player" && !spec.weapon_skills.is_empty() {
                return Err(err(
                    "creature `player`: the player's weapon skills come from their race and class"
                        .into(),
                ));
            }
            let ranks: BTreeMap<String, Dice> = (spec.weapon_skills.iter())
                .map(|(id, &rank)| (id.clone(), Dice::fixed(i32::from(rank))))
                .collect();
            let ranks = (self
                .weapon_skills
                .starting_ranks(&ranks, weapon_skill::Ranked::Any))
            .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
            let ranks = ranks.iter().map(|&(id, rank)| (id, rank.max() as u8));
            let weapon_skills = WeaponSkills::from_ranks(&self.weapon_skills, ranks);
            if let Some(hide) = &spec.natural {
                hide.check()
                    .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
            }
            // The status diseases show as comes of a disease, never of a blow.
            let status = |id: &str| {
                let found = self
                    .status_id(id)
                    .filter(|&s| Some(s) != self.disease_status)?;
                Some((found, self.status(found).max))
            };
            let kinds = (spec.attack.as_ref())
                .map(|a| self.resolve_kinds(a.damage_type.as_ref()))
                .transpose()
                .map_err(|m| err(format!("creature `{}`: attack: {m}", spec.id)))?;
            let attack = (spec.attack.clone())
                .zip(kinds)
                .map(|(a, kinds)| a.resolve(status, |id| self.disease_id(id), kinds))
                .transpose()
                .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
            let percent = crate::damage::MIN_RESISTANCE..=crate::damage::IMMUNE;
            let resist = (self.resolve_per_kind("resist", &spec.resist, percent, false))
                .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
            let stats = Stats::from_data(&spec.stats, &spec.potential)
                .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
            if let Some(cost) = spec.attack_cost {
                action::check_cost(cost)
                    .map_err(|m| err(format!("creature `{}`: attack_cost: {m}", spec.id)))?;
            }
            if spec.loot > 100 {
                return Err(err(format!(
                    "creature `{}`: `loot` is a chance in percent, 0 to 100",
                    spec.id
                )));
            }
            let size_id = spec.size.as_deref().unwrap_or(MEDIUM);
            let size = (self.size_index(size_id))
                .ok_or_else(|| err(format!("creature `{}`: unknown size `{size_id}`", spec.id)))?;
            let blood = creature::blood(spec.blood.as_deref())
                .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
            let resolved = creature::Resolved {
                blood,
                light,
                size,
                stats,
                skills,
                attack,
                weapon_skills,
                resist,
                faction: self.creature_faction(&spec).map_err(err)?,
                vision: self.vision_index(&spec.vision).ok_or_else(|| {
                    err(format!(
                        "creature `{}`: unknown vision profile `{}`",
                        spec.id, spec.vision
                    ))
                })?,
            };
            check_combat(&spec).map_err(err)?;
            if let Some(on_eat) = &spec.on_eat {
                on_eat
                    .check()
                    .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
                if !spec.corpse || spec.id == "player" {
                    return Err(err(format!(
                        "creature `{}`: `on_eat` is for creatures that leave a corpse",
                        spec.id
                    )));
                }
            }
            self.creatures.push(spec.resolve(resolved));
        }
        if !self.creature_ids.contains_key("player") {
            return Err(ContentError::new(
                first_file,
                "a creature with id `player` is required",
            ));
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

    /// Status effects and the healing rules (optional: none, and the default healing).
    fn load_statuses(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let path = "data/status.toml";
        let Some(text) = files.get_optional(path) else {
            return Ok(());
        };
        let (healing, list) = status::parse(path, text, |id| self.damage_type_id(id))?;
        for (i, def) in list.iter().enumerate() {
            let id = u16::try_from(i).map_err(|_| ContentError::new(path, "too many statuses"))?;
            self.status_ids.insert(def.id.clone(), StatusId(id));
        }
        self.healing = healing;
        self.statuses = list;
        Ok(())
    }

    /// Diseases (optional: none). Their status must be one that nothing but diseases gives or
    /// takes away: no ticks, no easing, no wounds, no cures of its own.
    fn load_diseases(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let path = "data/diseases.toml";
        let Some(text) = files.get_optional(path) else {
            return Ok(());
        };
        let (status, list) = disease::parse(
            path,
            text,
            |id| self.status_id(id),
            |id| self.terrain_id(id),
        )?;
        if let Some(status) = status {
            let def = self.status(status);
            if def.tick.is_some()
                || def.eases.is_some()
                || def.wound.is_some()
                || !def.cured_by.is_empty()
                || def.follows.is_some()
            {
                return Err(ContentError::new(
                    path,
                    format!(
                        "status `{}`: the status diseases show as lasts while they do: no \
                         `tick`, `eases`, `wound`, `cured_by`, or `follows` (cure the diseases \
                         instead)",
                        def.id
                    ),
                ));
            }
            if let Some(other) = self.statuses.iter().find(|s| s.follows == Some(status)) {
                return Err(ContentError::new(
                    path,
                    format!(
                        "status `{}`: follows `{}`, the status diseases show as (only diseases \
                         give it)",
                        other.id, def.id
                    ),
                ));
            }
        }
        for (i, def) in list.iter().enumerate() {
            let id = u16::try_from(i).map_err(|_| ContentError::new(path, "too many diseases"))?;
            self.disease_ids.insert(def.id.clone(), DiseaseId(id));
        }
        self.disease_status = status;
        self.diseases = list;
        Ok(())
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

    /// `data/gems.toml` (spec §7 "Gems"), once the objects, item classes, and part names it
    /// names are known.
    fn load_gems(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let path = "data/gems.toml";
        let Some(text) = files.get_optional(path) else {
            return Ok(());
        };
        let object_id = |id: &str| self.object_ids.get(id).copied();
        let known = gem::Known {
            object_id: &object_id,
            objects: &self.objects,
            classes: &self.item_classes,
            parts: &self.parts,
        };
        let gems = Gems::parse(path, text, &known)?;
        self.gems = Some(gems);
        Ok(())
    }

    /// Materials and part names, then the objects made of them.
    /// The material and object tables (`data/materials.toml` and `data/materials/*.toml`,
    /// `data/objects.toml` and `data/objects/*.toml`; both optional), numbered in path order
    /// (the single file first), with `data/parts.toml`, `data/slots.toml`, and
    /// `data/item_classes.toml` between them.
    fn load_objects(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let mut seen: BTreeMap<String, &str> = BTreeMap::new();
        for (i, (path, def)) in (load_table(files, "materials", |f: MaterialFile| f.material)?)
            .into_iter()
            .enumerate()
        {
            let id = MaterialId(
                u16::try_from(i).map_err(|_| ContentError::new(path, "too many materials"))?,
            );
            if let Some(first) = seen.insert(def.id.clone(), path) {
                return Err(duplicate_id("material", &def.id, path, first));
            }
            def.check().map_err(|m| ContentError::new(path, m))?;
            self.material_ids.insert(def.id.clone(), id);
            self.materials.push(def);
        }
        let path = "data/parts.toml";
        if let Some(text) = files.get_optional(path) {
            self.parts = part::parse(path, text)?;
        }
        let path = "data/slots.toml";
        if let Some(text) = files.get_optional(path) {
            self.slots = slot::parse(path, text)?;
        }
        let path = "data/item_classes.toml";
        if let Some(text) = files.get_optional(path) {
            self.item_classes = item_class::parse(path, text)?;
        }
        let specs = load_table(files, "objects", |f: ObjectFile| f.object)?;
        let mut seen: BTreeMap<String, &str> = BTreeMap::new();
        // Each object's file, for the errors found once every object is known.
        let mut paths = Vec::with_capacity(specs.len());
        let mut lit = Vec::with_capacity(specs.len());
        for (i, (path, spec)) in specs.into_iter().enumerate() {
            let id = ObjectId(
                u16::try_from(i).map_err(|_| ContentError::new(path, "too many objects"))?,
            );
            if let Some(first) = seen.insert(spec.id.clone(), path) {
                return Err(duplicate_id("object", &spec.id, path, first));
            }
            self.object_ids.insert(spec.id.clone(), id);
            let ornament = match &spec.ornament {
                Some(part) => Some(self.part_id(part).ok_or_else(|| {
                    ContentError::new(
                        path,
                        format!(
                            "object `{}`: `ornament = \"{part}\"` is no part of data/parts.toml",
                            spec.id
                        ),
                    )
                })?),
                None => None,
            };
            let (mut def, lit_as) = spec.resolve(
                path,
                self,
                &self.item_classes,
                |entries| {
                    part::resolve(
                        entries,
                        |m| self.material_ids.get(m).copied(),
                        |p| self.part_id(p),
                    )
                },
                |l| self.light_ids.get(l).copied(),
                &self.slots,
                &self.weapon_skills,
            )?;
            def.ornament = ornament;
            (def.name, def.plural) = part::names(
                &def.name,
                &def.plural,
                &def.parts,
                &self.materials,
                &self.parts,
            );
            check_effects(path, "object", &def.id, None, def.glow, 0.0, &def.particles)?;
            lit.push(lit_as);
            paths.push(path);
            self.objects.push(def);
        }
        // What is lit as it is loosed flies as an object that burns.
        for (i, lit_as) in lit.into_iter().enumerate() {
            let Some(name) = lit_as else { continue };
            let target = self.object_ids.get(&name).copied();
            let def = &self.objects[i];
            let Some(target) = target.filter(|t| self.objects[usize::from(t.0)].burning.is_some())
            else {
                return Err(ContentError::new(
                    paths[i],
                    format!(
                        "object `{}`: `lit = \"{name}\"` must name an object that burns (`light` \
                         and `burns`)",
                        def.id
                    ),
                ));
            };
            self.objects[i].lit = Some(target);
        }
        // A remedy cures what some status or disease names as its cure.
        for (o, path) in self.objects.iter().zip(&paths) {
            let named = |cure: &String| {
                (self.statuses.iter().any(|s| s.cured_by.contains(cure)))
                    || (self.diseases.iter().any(|d| d.cured_by.contains(cure)))
            };
            if let Some(cure) = o.cures.iter().find(|c| !named(c)) {
                return Err(ContentError::new(
                    path,
                    format!(
                        "object `{}` cures `{cure}`, but no status or disease names it in \
                         `cured_by`",
                        o.id
                    ),
                ));
            }
        }
        // Launchers and ammunition pair by kind: a launcher shoots something there is.
        for (o, path) in self.objects.iter().zip(&paths) {
            if let Some(kind) = &o.launches
                && !self.objects.iter().any(|a| a.ammo.as_ref() == Some(kind))
            {
                return Err(ContentError::new(
                    path,
                    format!(
                        "object `{}` launches `{kind}`, but no object has `ammo = \"{kind}\"`",
                        o.id
                    ),
                ));
            }
        }
        Ok(())
    }

    /// `data/hunger.toml` (spec §8 "Hunger"), then a corpse object for each creature type that
    /// leaves one (not the player), after the objects of `data/objects.toml`, in creature order.
    fn load_hunger(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        self.corpses = vec![None; self.creatures.len()];
        let path = "data/hunger.toml";
        let Some(text) = files.get_optional(path) else {
            return Ok(());
        };
        let err = |m: String| ContentError::new(path, m);
        let size_index = |id: &str| self.size_index(id);
        let def = HungerDef::parse(path, text, self.sizes.len(), size_index)?;
        if let Some(corpses) = &def.corpses {
            let material =
                (self.material_ids.get(&corpses.material).copied()).ok_or_else(|| {
                    err(format!(
                        "corpses: unknown material `{}` (data/materials.toml)",
                        corpses.material
                    ))
                })?;
            let class = (self.item_classes.iter().position(|c| c.corpses))
                .map(|i| ItemClassId(i as u8))
                .ok_or_else(|| {
                    err(
                        "creatures leave corpses: a class in data/item_classes.toml must take \
                         them (`corpses = true`)"
                            .into(),
                    )
                })?;
            for (i, creature) in self.creatures.iter().enumerate() {
                if creature.id == "player" || !creature.corpse {
                    continue;
                }
                let id = ObjectId(
                    u16::try_from(self.objects.len())
                        .map_err(|_| err("too many objects".into()))?,
                );
                let creature_id = CreatureId(i as u16);
                let food = corpses.sizes[creature.size];
                let def = ObjectDef::corpse_of(creature_id, creature, class, material, food);
                if self.object_ids.insert(def.id.clone(), id).is_some() {
                    return Err(err(format!(
                        "the corpse of `{}` would be object `{}`, which data/objects.toml has",
                        creature.id, def.id
                    )));
                }
                self.corpses[i] = Some(id);
                self.objects.push(def);
            }
        }
        // Harm from eating is degrees of a status (spec §8 "Status effects").
        let harms = def.rotten.harm.max() > 0
            || (def.corpses.is_some()
                && (self.creatures.iter())
                    .any(|c| c.on_eat.as_ref().is_some_and(|e| e.harm.is_some())));
        match &def.sickness {
            Some(id) if self.status_id(id).is_none() => {
                return Err(err(format!("unknown sickness `{id}` (data/status.toml)")));
            }
            None if harms => {
                return Err(err(
                    "rotten food's `harm` and `on_eat` harm need a `sickness`, a status in \
                     data/status.toml"
                        .into(),
                ));
            }
            _ => {}
        }
        self.hunger = Some(def);
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
        let key_id = |name: &str| {
            self.object_id(name)
                .filter(|o| self.object(*o).key.is_some())
        };
        let resolved = parsed.resolve(
            path,
            |name| self.creature_id(name),
            key_id,
            |name| self.trap_id(name),
        )?;
        for def in resolved {
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
    /// object, slot, skill, room effect, weapon skill, status, disease, trap, damage type, and
    /// shop ids.
    ///
    /// `TerrainId`, `CreatureId`, `LightId`, `ObjectId`, `SlotId`, `SkillId`, `RoomEffectId`,
    /// `WeaponSkillId`, `StatusId`, `DiseaseId`, `TrapId`, `DamageTypeId`, and `ShopId` are
    /// indices, so a save
    /// is only meaningful for content with the same ids in the same order; saves record this
    /// fingerprint to detect a mismatch.
    pub fn fingerprint(&self) -> u64 {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        let mut feed = |bytes: &[u8]| {
            for &b in bytes {
                hash = (hash ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3);
            }
        };
        let groups: &[Vec<&str>] = &[
            self.terrain.iter().map(|t| t.id.as_str()).collect(),
            self.creatures.iter().map(|c| c.id.as_str()).collect(),
            self.levels.keys().map(String::as_str).collect(),
            self.lights.iter().map(|l| l.id.as_str()).collect(),
            self.dungeons.keys().map(String::as_str).collect(),
            self.objects.iter().map(|o| o.id.as_str()).collect(),
            self.slots.iter().map(|s| s.id.as_str()).collect(),
            self.skills.list.iter().map(|s| s.id.as_str()).collect(),
            (self.room_effects.defs().iter())
                .map(|e| e.id.as_str())
                .collect(),
            (self.weapon_skills.skills().iter())
                .map(|s| s.id.as_str())
                .collect(),
            self.statuses.iter().map(|s| s.id.as_str()).collect(),
            self.diseases.iter().map(|d| d.id.as_str()).collect(),
            self.traps.iter().map(|t| t.id.as_str()).collect(),
            self.damage_types.iter().map(|t| t.id.as_str()).collect(),
            (self.shops.iter().flat_map(|s| s.defs()))
                .map(|d| d.id.as_str())
                .collect(),
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

    /// The terrain ice looks like (thin ice, ice, thick ice; spec §5 "Ice"), each with the
    /// thinnest ice that looks like it in mm, thinnest first; empty if nothing freezes.
    pub fn ice_terrain(&self) -> &[(u16, TerrainId)] {
        &self.ice
    }

    /// What ice `um` micrometres thick looks like; `None` when it is too thin to be more than
    /// the water beneath (thinner than the thinnest ice terrain).
    pub fn ice_look(&self, um: u32) -> Option<TerrainId> {
        (self.ice.iter().rev())
            .find(|(mm, _)| um >= u32::from(*mm) * 1000)
            .map(|(_, t)| *t)
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

    /// The damage types (spec §8 "Damage types"), in file order (by [`DamageTypeId`]); empty
    /// without `data/damage_types.toml`, and then nothing is typed.
    pub fn damage_types(&self) -> &[DamageTypeDef] {
        &self.damage_types
    }

    pub fn damage_type(&self, id: DamageTypeId) -> &DamageTypeDef {
        &self.damage_types[usize::from(id.0)]
    }

    pub fn damage_type_id(&self, name: &str) -> Option<DamageTypeId> {
        let i = self.damage_types.iter().position(|t| t.id == name)?;
        u8::try_from(i).ok().map(DamageTypeId)
    }

    /// The kind of damage bare hands, kicks, and shield bashes do (the type marked `unarmed`),
    /// if content names one.
    pub fn unarmed_damage(&self) -> Option<DamageTypeId> {
        let i = self.damage_types.iter().position(|t| t.unarmed)?;
        u8::try_from(i).ok().map(DamageTypeId)
    }

    /// Whether damage of `kind` is physical, which Protection soaks (untyped damage counts as
    /// physical: it is soaked as every blow was before damage had types).
    pub fn is_physical(&self, kind: Option<DamageTypeId>) -> bool {
        kind.is_none_or(|k| self.damage_type(k).physical)
    }

    /// The kinds `names` name (`damage_type` in data: one or two type ids), checked; none
    /// named is an untyped attack (soaked and wounding as every blow did before damage had
    /// types; the built-in data names one everywhere, a test checks).
    pub(crate) fn resolve_kinds(
        &self,
        names: Option<&damage_type::TypeNames>,
    ) -> Result<crate::damage::Kinds, String> {
        use crate::damage::{Kinds, MAX_KINDS};
        let Some(names) = names else {
            return Ok(Kinds::NONE);
        };
        if self.damage_types.is_empty() {
            return Err("`damage_type` needs data/damage_types.toml".into());
        }
        let mut kinds = Vec::new();
        for name in names.names() {
            let id = self
                .damage_type_id(name)
                .ok_or_else(|| format!("unknown damage type `{name}`"))?;
            if kinds.contains(&id) {
                return Err(format!("`damage_type` names `{name}` twice"));
            }
            kinds.push(id);
        }
        if kinds.is_empty() {
            return Err("`damage_type` names no type".into());
        }
        Kinds::of(&kinds).ok_or_else(|| format!("`damage_type` names at most {MAX_KINDS} types"))
    }

    /// The percentages per kind `map` gives (`resist = { fire = 25 }` on a creature or an
    /// object; `protects = { slashing = 1 }` on armour, `physical` only), checked against
    /// `range`.
    pub(crate) fn resolve_per_kind(
        &self,
        field: &str,
        map: &BTreeMap<String, i32>,
        range: std::ops::RangeInclusive<i32>,
        physical: bool,
    ) -> Result<Vec<(DamageTypeId, i32)>, String> {
        let mut out = Vec::new();
        for (name, value) in map {
            let id = self
                .damage_type_id(name)
                .ok_or_else(|| format!("`{field}`: unknown damage type `{name}`"))?;
            if physical && !self.damage_type(id).physical {
                return Err(format!("`{field}`: `{name}` is no physical damage type"));
            }
            if !range.contains(value) {
                return Err(format!(
                    "`{field}`: `{name}` must be {} to {}",
                    range.start(),
                    range.end()
                ));
            }
            out.push((id, *value));
        }
        // In damage type order, whatever order the file wrote them in.
        out.sort_by_key(|(id, _)| *id);
        Ok(out)
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

    /// The item classes, in the order item lists group by (by [`ItemClassId`]).
    pub fn item_classes(&self) -> &[ItemClassDef] {
        &self.item_classes
    }

    pub fn item_class(&self, id: ItemClassId) -> &ItemClassDef {
        &self.item_classes[usize::from(id.0)]
    }

    pub fn item_class_id(&self, name: &str) -> Option<ItemClassId> {
        (self.item_classes.iter().position(|c| c.id == name)).map(|i| ItemClassId(i as u8))
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

    /// Whether `object` goes in `slot`: it fits there properly ([`Content::fits_properly`]), or
    /// the slot takes weapons (the missile weapon slot) and it is one or can be thrown.
    pub fn fits(&self, object: ObjectId, slot: SlotId) -> bool {
        let def = self.object(object);
        self.fits_properly(object, slot)
            || (self.slot(slot).takes_weapons
                && (def.can_throw() || (def.attack.is_some() && def.ammo.is_none())))
    }

    /// Whether `object` goes in `slot` by its own `slot`, not as a weapon some slot takes too:
    /// its `slot` is what the slot fits, or it is a light weapon and the slot is a hand (the off
    /// hand: two-weapon fighting, spec §8 "Fighting styles").
    pub fn fits_properly(&self, object: ObjectId, slot: SlotId) -> bool {
        let (def, slot) = (self.object(object), self.slot(slot));
        def.slot.as_ref() == Some(&slot.fits) || (def.light_weapon && slot.hand)
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

    /// The plain ammunition of `kind` (`ammo = "arrow"`): the first by object id, so "shoots
    /// arrows" names the arrow before the flaming arrow whatever order the files load in.
    pub fn ammunition(&self, kind: &str) -> Option<&ObjectDef> {
        (self.objects.iter())
            .filter(|o| o.ammo.as_deref() == Some(kind))
            .min_by(|a, b| a.id.cmp(&b.id))
    }

    pub fn trap(&self, id: TrapId) -> &TrapDef {
        &self.traps[usize::from(id.0)]
    }

    pub fn trap_id(&self, name: &str) -> Option<TrapId> {
        self.trap_ids.get(name).copied()
    }

    /// Every trap type, in file order (by [`TrapId`]; none without `data/traps.toml`).
    pub fn trap_defs(&self) -> &[TrapDef] {
        &self.traps
    }

    /// The numbers of searching for hidden features (spec §8 "Searching"; `data/search.toml`,
    /// else [`SearchRules::default`]).
    pub fn search(&self) -> &SearchRules {
        &self.search
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

    /// Gem sizes, cuts, and ornaments (`data/gems.toml`, spec §7 "Gems"); `None` without the
    /// file: no gems are found or set.
    pub fn gems(&self) -> Option<&Gems> {
        self.gems.as_ref()
    }

    /// The part names of `data/parts.toml`, in file order (by [`PartId`]).
    pub fn part_defs(&self) -> &[PartDef] {
        &self.parts
    }

    pub fn part_id(&self, name: &str) -> Option<PartId> {
        let i = self.parts.iter().position(|p| p.id == name)?;
        Some(PartId(u8::try_from(i).ok()?))
    }

    /// `parts` in words, every part, the main one first: "steel blade, leather grip", "iron"
    /// (spec §6 "Item description").
    pub fn describe_parts(&self, parts: &[Part]) -> String {
        part::phrase(parts, &self.materials, &self.parts)
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

    /// The combat stances of `data/stances.toml` (all plain without the file).
    pub fn stances(&self) -> &Stances {
        &self.stances
    }

    /// What combat stance `stance` does.
    pub fn stance_def(&self, stance: CombatStance) -> &StanceDef {
        self.stances.get(stance)
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

    /// Status effects (`data/status.toml`), most important first; empty without the file.
    pub fn status_defs(&self) -> &[StatusDef] {
        &self.statuses
    }

    pub fn status(&self, id: StatusId) -> &StatusDef {
        &self.statuses[usize::from(id.0)]
    }

    pub fn status_id(&self, name: &str) -> Option<StatusId> {
        self.status_ids.get(name).copied()
    }

    /// Every status with its id, most important first.
    pub fn status_ids(&self) -> impl Iterator<Item = (StatusId, &StatusDef)> {
        (0u16..).map(StatusId).zip(&self.statuses)
    }

    /// The passive healing rules (`[healing]` in `data/status.toml`).
    pub fn healing(&self) -> &Healing {
        &self.healing
    }

    /// Diseases (`data/diseases.toml`), in file order; empty without the file.
    pub fn disease_defs(&self) -> &[DiseaseDef] {
        &self.diseases
    }

    pub fn disease(&self, id: DiseaseId) -> &DiseaseDef {
        &self.diseases[usize::from(id.0)]
    }

    pub fn disease_id(&self, name: &str) -> Option<DiseaseId> {
        self.disease_ids.get(name).copied()
    }

    /// Every disease with its id, in file order.
    pub fn disease_ids(&self) -> impl Iterator<Item = (DiseaseId, &DiseaseDef)> {
        (0u16..).map(DiseaseId).zip(&self.diseases)
    }

    /// The status diseases show as ("Diseased"); `None` without diseases.
    pub fn disease_status(&self) -> Option<StatusId> {
        self.disease_status
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

    /// The names offered to new characters, by race and gender (`data/names.toml`; none without
    /// the file).
    pub fn names(&self) -> &Names {
        &self.names
    }

    /// The classes of `data/classes.toml`, in the order offered (none without the file).
    pub fn classes(&self) -> &[ClassDef] {
        &self.classes
    }

    /// The class with id `id`.
    pub fn class(&self, id: &str) -> Option<&ClassDef> {
        self.classes.iter().find(|c| c.id == id)
    }

    /// The alignment rules of `data/alignment.toml` (spec §8 "Alignment"); `None` without the
    /// file: nobody has a score and nothing moves it.
    pub fn alignment(&self) -> Option<&AlignmentRules> {
        self.alignment.as_ref()
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

    /// Prefixes and suffixes, and how loot rolls them (spec §8 "Magic items"); empty without
    /// `data/affixes.toml`.
    pub fn affixes(&self) -> &Affixes {
        &self.affixes
    }

    /// The item level of loot on level `id` (spec §8 "Magic items"): a dungeon branch level's
    /// depth, a hand-made level's `item_level` (default 1), else 1.
    pub fn item_level(&self, id: &str) -> u8 {
        if let Some(template) = self.levels.get(id) {
            return template.item_level;
        }
        match self.branch_level(id) {
            Some((_, depth)) => u8::try_from(depth)
                .unwrap_or(MAX_ITEM_LEVEL)
                .clamp(1, MAX_ITEM_LEVEL),
            None => 1,
        }
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

    /// How the player hungers, `data/hunger.toml` (spec §8 "Hunger"); `None` without the file:
    /// nobody hungers, nothing is eaten, and food does not rot.
    pub fn hunger(&self) -> Option<&HungerDef> {
        self.hunger.as_ref()
    }

    /// The corpse a creature of type `creature` leaves when it dies, if any.
    pub fn corpse_of(&self, creature: CreatureId) -> Option<ObjectId> {
        self.corpses.get(usize::from(creature.0)).copied().flatten()
    }

    /// The weapon skills (`data/weapon_skills.toml`; empty without the file).
    pub fn weapon_skills(&self) -> &WeaponSkillTable {
        &self.weapon_skills
    }

    /// What creatures say (`data/dialogue.toml`; empty without it), in file order.
    pub fn dialogues(&self) -> &[Dialogue] {
        &self.dialogues
    }

    pub fn dialogue(&self, id: DialogueId) -> &Dialogue {
        &self.dialogues[usize::from(id.0)]
    }

    /// The dialogue of a creature of type `creature` that a level named `npc`: the npc's own if
    /// it has one, else its type's; `None` if neither talks.
    pub fn dialogue_of(&self, creature: CreatureId, npc: Option<&str>) -> Option<DialogueId> {
        let find = |speaker: &Speaker| self.dialogues.iter().position(|d| d.speaker == *speaker);
        let npc = npc.and_then(|n| find(&Speaker::Npc(n.to_string())));
        npc.or_else(|| find(&Speaker::Creature(creature)))
            .map(|i| DialogueId(i as u16))
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

    /// The effects of special rooms and how often generated levels get them
    /// (`data/room_effects.toml`; none without it).
    pub fn room_effects(&self) -> &RoomEffects {
        &self.room_effects
    }

    /// The effect `id` of a special room.
    /// The shops of `data/shops.toml` (spec §7 "Shops"); `None` without the file.
    pub fn shops(&self) -> Option<&Shops> {
        self.shops.as_ref()
    }

    pub fn room_effect(&self, id: RoomEffectId) -> &RoomEffectDef {
        self.room_effects.get(id)
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
                .chain(t.variants.iter().copied())
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
        let traps = self.traps.iter().map(|t| t.glyph);
        terrain
            .chain(creatures)
            .chain(objects)
            .chain(weather)
            .chain(effects)
            .chain(traps)
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

/// Checks a terrain type's ice fields (spec §5 "Ice"): water that freezes and the ice on it
/// are walkable, transparent, give no light, and are neither doors nor stairs (ice and water
/// trade places as it freezes and thaws, so nothing else may change with them); water freezes
/// from 0 °C down to [`crate::ice::COLDEST_FREEZE`], at 1 to 100 percent of still water's
/// pace, and a fall into it takes no hit points back; ice is 1 mm to [`crate::ice::MAX_MM`]
/// thick and does not move like water.
fn check_terrain_ice(spec: &terrain::TerrainSpec) -> Result<(), String> {
    if spec.freeze.is_none() && spec.ice.is_none() {
        return Ok(());
    }
    if spec.freeze.is_some() && spec.ice.is_some() {
        return Err("terrain is water that freezes (`freeze`) or ice (`ice`), not both".into());
    }
    let plain = spec.walkable
        && spec.transparent
        && spec.light.is_none()
        && spec.stairs.is_none()
        && spec.opens_to.is_none()
        && spec.closes_to.is_none()
        && spec.travel_minutes.is_none();
    if !plain {
        return Err(
            "water that freezes and ice must be walkable, transparent, without a light, \
                    and neither a door nor stairs nor overworld terrain"
                .into(),
        );
    }
    if let Some(freeze) = &spec.freeze {
        if !(crate::ice::COLDEST_FREEZE..=0).contains(&freeze.below) {
            return Err(format!(
                "`freeze.below` must be {} to 0 (°C)",
                crate::ice::COLDEST_FREEZE
            ));
        }
        if !(1..=100).contains(&freeze.rate) {
            return Err("`freeze.rate` must be 1 to 100 (percent)".into());
        }
        if freeze.plunge.min() < 0 {
            return Err("`freeze.plunge` must not heal".into());
        }
    }
    if let Some(mm) = spec.ice {
        if !(1..=crate::ice::MAX_MM).contains(&mm) {
            return Err(format!("`ice` must be 1 to {} (mm)", crate::ice::MAX_MM));
        }
        if spec.anim.as_ref().is_some_and(GlyphAnim::is_flow) {
            return Err("ice does not flow".into());
        }
    }
    Ok(())
}

/// Checks a terrain type's kicking fields (spec §8 "Doors"): only a closed door breaks, and
/// `breaks_to` and `sturdiness` go together.
fn check_kicking(spec: &terrain::TerrainSpec) -> Result<(), String> {
    match (&spec.breaks_to, spec.sturdiness) {
        (None, None) => Ok(()),
        (Some(_), Some(_)) if spec.opens_to.is_none() => {
            Err("only closed doors (`opens_to`) break when kicked".into())
        }
        (Some(_), Some(s)) if !crate::door::STURDINESS.contains(&s) => Err(format!(
            "`sturdiness` must be {} to {}",
            crate::door::STURDINESS.start(),
            crate::door::STURDINESS.end()
        )),
        (Some(_), Some(_)) => Ok(()),
        _ => Err("`breaks_to` and `sturdiness` go together".into()),
    }
}

/// Checks how a terrain type breaks (spec §8 "Digging"): only solid terrain (neither walkable
/// nor a door, stairs, or a light) breaks, into plain open ground (walkable and transparent,
/// that does not break itself); solid rock shows a solid face.
fn check_breaking(def: &TerrainDef, defs: &[TerrainDef]) -> Result<(), String> {
    let plain = |t: &TerrainDef| t.stairs.is_none() && !t.is_closed_door() && !t.is_open_door();
    if let Some(breaks) = def.breaks {
        if def.walkable || !plain(def) || def.light.is_some() {
            return Err(
                "only solid terrain breaks (`hardness`): not walkable, no door, stairs, or light"
                    .into(),
            );
        }
        let becomes = &defs[usize::from(breaks.becomes.0)];
        if !becomes.walkable || !becomes.transparent || becomes.breaks.is_some() || !plain(becomes)
        {
            return Err(format!(
                "it `becomes` `{}`, which must be open ground: walkable, transparent, no door or \
                 stairs, and not breaking itself",
                becomes.id
            ));
        }
    }
    if let Some(face) = def.face {
        let face = &defs[usize::from(face.0)];
        let solid = |t: &TerrainDef| !t.walkable && !t.transparent;
        if !solid(def) || !solid(face) || face.face.is_some() {
            return Err(format!(
                "only solid rock has a `face`, and its face `{}` must be solid with none of its own",
                face.id
            ));
        }
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
        files.insert("data/item_classes.toml", BUILTIN_ITEM_CLASSES);
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

    /// A creature entry of nothing but the required fields.
    fn creature_entry(id: &str, glyph: char) -> String {
        format!(
            "[[creature]]\nid = \"{id}\"\nname = \"{id}\"\nglyph = \"{glyph}\"\nfg = \"#ffffff\"\n\
             weight = 1000\n"
        )
    }

    /// A table split over its single file and a directory: the single file's entries come
    /// first, then the directory's files in path order (subdirectories included), a file of
    /// comments only adds nothing, and `remove` drops a directory whole.
    #[test]
    fn a_table_directory_loads_in_path_order_after_the_single_file() {
        let mut files = minimal_files("#@#\n", "");
        files.insert("data/creatures/README.toml", "# Notes only.\n");
        files.insert(
            "data/creatures/b/kobold.toml",
            &creature_entry("kobold", 'k'),
        );
        files.insert("data/creatures/a.toml", &creature_entry("bat", 'b'));
        files.insert(
            "data/creatures/c.toml",
            &format!(
                "{}{}",
                creature_entry("deer", 'd'),
                creature_entry("beetle", 'a')
            ),
        );
        let table: Vec<&str> = files.table("creatures").map(|(p, _)| p).collect();
        assert_eq!(
            table,
            [
                "data/creatures.toml",
                "data/creatures/README.toml",
                "data/creatures/a.toml",
                "data/creatures/b/kobold.toml",
                "data/creatures/c.toml",
            ]
        );
        let content = Content::load(&files).expect("valid");
        let ids: Vec<&str> = (content.creature_defs().iter())
            .map(|c| c.id.as_str())
            .collect();
        assert_eq!(ids, ["player", "rat", "bat", "kobold", "deer", "beetle"]);
        assert_eq!(content.creature_id("deer"), Some(CreatureId(4)));
        // Without the single file, the directory alone is the table.
        files.remove("data/creatures.toml");
        files.insert("data/creatures/player.toml", &creature_entry("player", '@'));
        let content = Content::load(&files).expect("valid");
        let ids: Vec<&str> = (content.creature_defs().iter())
            .map(|c| c.id.as_str())
            .collect();
        assert_eq!(ids, ["bat", "kobold", "deer", "beetle", "player"]);
        // `edit_table` rewrites every file of the table.
        files.edit_table("creatures", |t| t.replace("weight = 1000", "weight = 2000"));
        let content = Content::load(&files).expect("valid");
        assert!(content.creature_defs().iter().all(|c| c.weight == 2000));
        // Removing the directory leaves no creature file at all.
        files.remove("data/creatures");
        assert_eq!(files.table("creatures").count(), 0);
        let err = Content::load(&files).unwrap_err();
        assert_eq!(err.file, "data/creatures.toml");
        assert!(err.message.contains("missing"), "{err}");
    }

    /// An id in two files of a table is an error naming both files; a file that does not parse
    /// is an error naming it.
    #[test]
    fn a_duplicate_id_across_table_files_names_both_files() {
        let mut files = minimal_files("#@#\n", "");
        files.insert("data/creatures/rat.toml", &creature_entry("rat", 'r'));
        let err = Content::load(&files).unwrap_err();
        assert_eq!(err.file, "data/creatures/rat.toml");
        assert_eq!(
            err.message,
            "duplicate creature id `rat` (also in data/creatures.toml)"
        );
        files.remove("data/creatures/");
        let iron = "[[material]]\nid = \"iron\"\nname = \"iron\"\n";
        files.insert("data/materials.toml", iron);
        files.insert("data/materials/iron.toml", iron);
        let err = Content::load(&files).unwrap_err();
        assert_eq!(err.file, "data/materials/iron.toml");
        assert_eq!(
            err.message,
            "duplicate material id `iron` (also in data/materials.toml)"
        );
        files.remove("data/materials/");
        let nail = "[[object]]\nid = \"nail\"\nname = \"nail\"\nglyph = \"(\"\nfg = \"#ffffff\"\nclass = \"tools\"\n\
                    materials = [\"iron\"]\nweight = 5\nvalue = 1\n";
        files.insert("data/objects/a.toml", nail);
        files.insert("data/objects/b.toml", nail);
        let err = Content::load(&files).unwrap_err();
        assert_eq!(err.file, "data/objects/b.toml");
        assert_eq!(
            err.message,
            "duplicate object id `nail` (also in data/objects/a.toml)"
        );
        files.insert("data/objects/b.toml", "[[object]\n");
        let err = Content::load(&files).unwrap_err();
        assert_eq!(err.file, "data/objects/b.toml");
        files.remove("data/objects/b.toml");
        let content = Content::load(&files).expect("valid");
        assert_eq!(content.object_id("nail"), Some(ObjectId(0)));
    }

    /// `from_dir` reads subdirectories like `build.rs` embeds them: the same paths, the built-in
    /// set being exactly what is on disk under `assets/`.
    #[test]
    fn from_dir_reads_nested_directories_like_the_build() {
        let root = std::env::temp_dir().join(format!("warlock-content-{}", std::process::id()));
        let deep = root.join("data/creatures/deep");
        std::fs::create_dir_all(&deep).expect("temp dir");
        std::fs::create_dir_all(root.join("levels")).expect("temp dir");
        std::fs::write(root.join("data/legend.toml"), "x = 1\n").expect("write");
        std::fs::write(root.join("data/creatures/rat.toml"), "a = 1\n").expect("write");
        std::fs::write(deep.join("bat.toml"), "b = 2\n").expect("write");
        std::fs::write(deep.join("notes.txt"), "not content\n").expect("write");
        std::fs::write(root.join("levels/hub.toml"), "c = 3\n").expect("write");
        let files = ContentFiles::from_dir(&root).expect("readable");
        std::fs::remove_dir_all(&root).expect("cleanup");
        let paths: Vec<&str> = files.files.keys().map(String::as_str).collect();
        assert_eq!(
            paths,
            [
                "data/creatures/deep/bat.toml",
                "data/creatures/rat.toml",
                "data/legend.toml",
                "levels/hub.toml",
            ]
        );
        let table: Vec<(&str, &str)> = files.table("creatures").collect();
        assert_eq!(
            table,
            [
                ("data/creatures/deep/bat.toml", "b = 2\n"),
                ("data/creatures/rat.toml", "a = 1\n"),
            ]
        );
        let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let on_disk = ContentFiles::from_dir(&assets).expect("assets");
        assert_eq!(on_disk.files, ContentFiles::builtin().files);
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
        // The built-in creature table as one text, for the checks that an edit applies.
        let creatures: String = (ContentFiles::builtin().table("creatures"))
            .map(|(_, t)| t.to_string())
            .collect::<Vec<_>>()
            .join("\n");
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
            files.edit_table("creatures", |t| t.replacen(from, to, 1));
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
        // Every race offers names to every gender, and every scenario names its character.
        for race in &content.races().list {
            for gender in crate::character::Gender::ALL {
                let names = content.names().offered(&race.id, gender);
                assert!(names.len() >= 10, "{} {gender:?}: {names:?}", race.id);
            }
        }
        for scenario in content.scenarios() {
            assert!(scenario.character_name.is_some(), "{}", scenario.id);
        }
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
        // A vault may be stamped many times: no unique character in one.
        let mut files = ContentFiles::builtin();
        let dialogue = files.get("data/dialogue.toml").unwrap().to_string();
        let twin = "[[dialogue]]\nnpc = \"twin\"\nlines = [\"Again?\"]\n";
        files.insert("data/dialogue.toml", &format!("{dialogue}\n{twin}"));
        files.insert(
            "vaults/bad.toml",
            "name = \"Bad\"\nmap = '''\n##+##\n#.s.#\n#####\n'''\n[legend]\n\
             \"s\" = { terrain = \"floor\", creature = \"deer\", npc = \"twin\" }\n",
        );
        let err = Content::load(&files).unwrap_err();
        assert!(err.message.contains("unique npc"), "{}", err.message);
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
        // With a corpse for every creature type but the player, and the magic base items.
        assert!((15..=150).contains(&objects.len()), "{}", objects.len());
        let glyphs = content.glyphs();
        for def in objects {
            assert!(def.weight > 0 && glyphs.contains(&def.glyph), "{}", def.id);
            assert!(!content.material(def.material()).name.is_empty());
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
        let materials: BTreeSet<MaterialId> = objects.iter().map(|o| o.material()).collect();
        assert!(materials.len() >= 10, "objects of many materials");
        let rubies = object::parse_stacks(&["gem:ruby*2".into()], &content).expect("rubies");
        assert_eq!(rubies[0].describe(&content), "2 rubies");
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
                "[[object]]\nid = \"key\"\nname = \"key\"\nglyph = \"(\"\nfg = \"#ffffff\"\nclass = \"tools\"\n{fields}"
            )
        };
        let cases = [
            (
                object("materials = [\"iron\"]\nweight = 0\nvalue = 1\n"),
                "at least 1",
            ),
            (
                object("materials = [\"iron\"]\nweight = 5\nvalue = 1000001\n"),
                "`value` is in gold coins",
            ),
            (
                object("materials = [\"iron\"]\nweight = 5\n"),
                "missing field `value`",
            ),
            (
                object("materials = [\"bronze\"]\nweight = 5\nvalue = 1\n"),
                "unknown material `bronze`",
            ),
            (
                object("materials = [\"iron\"]\nweight = 5\nvalue = 1\n").replace("tools", "toys"),
                "unknown item class `toys`",
            ),
            (
                object("materials = [\"iron\"]\nweight = 5\nvalue = 1\n")
                    .replace("class = \"tools\"\n", ""),
                "missing field `class`",
            ),
            (
                object("materials = [\"iron\"]\nweight = 5\nvalue = 1\nkind = \"ring\"\n"),
                "`kind = \"ring\"` is of class `rings`, not `tools`",
            ),
            (
                object("materials = [\"iron\"]\nvalue = 1\n"),
                "missing field `weight`",
            ),
            (
                object("weight = 5\nvalue = 1\n"),
                "`materials` lists its parts, at least one",
            ),
            (
                object("material = \"iron\"\nweight = 5\nvalue = 1\n"),
                "`material` is now `materials`, a list of parts, the first the main one: \
                 `materials = [\"iron\"]` for one",
            ),
            (
                object(
                    "materials = [\"iron\", { part = \"grip\", of = \"iron\", share = 5 }]\n\
                     weight = 5\nvalue = 1\n",
                ),
                "unknown part `grip` (data/parts.toml)",
            ),
            (
                object("materials = [\"iron\"]\nweight = 5\nvalue = 1\n").replace("\"(\"", "\" \""),
                "visible character",
            ),
            (
                format!(
                    "{0}\n{0}",
                    object("materials = [\"iron\"]\nweight = 5\nvalue = 1\n")
                ),
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
                "[[object]]\nid = \"key\"\nname = \"key\"\nglyph = \"(\"\nfg = \"#ffffff\"\nclass = \"tools\"\n\
                 materials = [\"iron\"]\nweight = 5\nvalue = 1\n{fields}\n"
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
            // Protection is for armour worn, never in a hand or on a shield; penetration for
            // weapons and ammunition.
            ("protection = 1", "`protection` is for armour worn"),
            (
                "slot = \"main_hand\"\nprotection = 1",
                "`protection` is for armour worn",
            ),
            (
                "slot = \"head\"\nshield = 1\nprotection = 1",
                "`protection` is for armour worn",
            ),
            (
                "slot = \"head\"\nprotection = 21",
                "`protection` must be 0 to 20",
            ),
            (
                "slot = \"head\"\npenetration = 1",
                "`penetration` is for weapons",
            ),
            (
                "slot = \"main_hand\"\ndamage = \"1d4\"\npenetration = -1",
                "`penetration` must be 0 to 20",
            ),
        ] {
            let err = load(&slots, &object(fields)).unwrap_err();
            assert_eq!(err.file, "data/objects.toml", "{fields}");
            assert!(err.message.contains(expected), "{fields}: {}", err.message);
        }
        for fields in [
            "slot = \"head\"\nprotection = 2",
            "slot = \"main_hand\"\ndamage = \"1d8\"\npenetration = 2",
            "ammo = \"bolt\"\npenetration = 2",
        ] {
            assert!(load(&slots, &object(fields)).is_ok(), "{fields}");
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
                "natural: `defence` and `protection` must be 0 to 20, not both 0",
            ),
            (
                "rat",
                "natural = { protection = 21, name = \"hide\" }",
                "natural: `defence` and `protection` must be 0 to 20",
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
                      [[pile]]\nat = [2, 1]\nobjects = [\"gem:ruby\"]\n";
        let content = stream(map, legend).unwrap();
        let level = content.level("stream").unwrap();
        let id = |name: &str| content.object_id(name).unwrap();
        let stack = |name: &str, count| crate::object::Stack::new(id(name), count);
        let ruby = object::parse_stacks(&["gem:ruby".into()], &content).expect("a ruby");
        assert_eq!(
            level.objects[&Pos::new(2, 1)],
            [
                ruby[0].clone(),
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
