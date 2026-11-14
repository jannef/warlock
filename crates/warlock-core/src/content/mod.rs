//! Game content: definitions, levels, scenarios, dungeon branches, and vaults loaded from TOML
//! (spec §7).
//!
//! Content is read from a [`ContentFiles`] set: the built-in set compiled into the binary (so every
//! build, including the web build, is self-contained) or a directory on disk for development.

mod access_character;
mod access_creatures;
mod access_items;
mod access_magic;
mod access_rules;
mod access_status;
mod access_world;
mod action;
mod affix;
mod alignment;
mod ally;
mod anim;
pub mod animated;
#[cfg(test)]
mod animated_tests;
mod aura;
#[cfg(test)]
mod aura_roster_tests;
mod background;
mod breath;
mod burst;
mod calendar;
mod calling;
mod campaign;
mod chest;
mod circles;
mod class;
mod companion;
mod creature;
mod creature_rarity;
mod creature_spells;
mod crystal;
mod damage_type;
pub mod danger;
mod decipher;
#[cfg(test)]
mod demon_roster_tests;
mod dialogue;
mod disease;
pub mod dread;
#[cfg(test)]
mod dread_roster_tests;
mod drops;
#[cfg(test)]
mod drow_roster_tests;
#[cfg(test)]
mod duergar_roster_tests;
mod dummy;
mod dungeon;
mod effect;
#[cfg(test)]
mod elemental_roster_tests;
mod encounter;
#[cfg(test)]
mod encounter_tests;
mod experience;
mod faction;
mod familiar;
mod feint;
#[cfg(test)]
mod fey_roster_tests;
#[cfg(test)]
mod fiend_roster_tests;
mod files;
mod fingerprint;
mod fletch;
mod flow;
mod forage;
#[cfg(test)]
mod forge_tests;
#[cfg(test)]
mod gate_roster_tests;
mod gaze;
mod gear;
mod gear_condition;
mod gem;
pub mod grimoire;
mod guild;
#[cfg(test)]
mod hag_roster_tests;
#[cfg(test)]
mod hellegeat_tests;
mod herbs;
mod horoscope;
mod hunger;
mod intimidate;
mod invest;
mod item_class;
mod kit;
mod layer_boss;
mod learning;
pub mod leaving;
mod level;
mod light;
mod load_creatures;
mod load_items;
mod load_status;
mod load_world;
#[cfg(test)]
mod loader_check_tests;
mod material;
pub mod mimic;
#[cfg(test)]
mod mimic_roster_tests;
mod mining;
mod miscast;
mod monster_pool;
mod names;
mod object;
mod offer;
#[cfg(test)]
mod orc_roster_tests;
mod outcome;
mod overworld;
pub(crate) mod part;
mod particles;
mod patron;
mod plural;
mod potion;
mod prayer;
mod prestige;
mod quest;
mod quest_roll;
mod race;
mod ranged;
mod recipe;
#[cfg(test)]
mod recipe_tests;
pub mod required;
mod resource;
mod rig;
mod rod;
#[cfg(test)]
mod room_burden_tests;
#[cfg(test)]
mod room_collapse_tests;
mod room_effect;
mod room_haunt;
#[cfg(test)]
mod room_haunt_tests;
#[cfg(test)]
mod room_homes_tests;
mod room_lore;
#[cfg(test)]
mod room_lore_tests;
#[cfg(test)]
mod room_mirrors_tests;
mod room_shrine;
#[cfg(test)]
mod room_shrine_tests;
#[cfg(test)]
mod room_temperature_tests;
mod room_treasury;
#[cfg(test)]
mod room_treasury_tests;
#[cfg(test)]
mod room_warp_tests;
mod ruin;
mod salvage;
mod scenario;
mod scroll;
mod scroll_label;
mod search;
#[cfg(test)]
mod second_wave_roster_tests;
mod shop;
mod size;
mod skill;
mod sky;
mod slot;
mod smite;
mod spell;
mod spell_def;
mod spell_ladder;
mod spell_mod;
mod spell_rank;
mod spell_type;
mod split;
mod stance;
pub mod starter;
mod stat_training;
mod status;
mod steal;
mod stipend;
mod stock;
mod summon;
#[cfg(test)]
mod summoner_roster_tests;
mod surprise;
mod swim;
mod tag;
mod talent;
mod temper;
mod terrain;
#[cfg(test)]
mod tests;
mod theft;
mod throw;
mod town;
mod trap;
mod unique;
mod unseen;
mod validate;
mod vault;
mod vault_turn;
mod vision;
mod walk;
mod wand;
mod weapon_ability;
mod weapon_skill;
mod weather;

pub use action::{ActionCosts, ActionKind, BASE_TURN, COSTS, NORMAL_COST, price};
pub use affix::{
    AffixDef, AffixKind, Affixes, Effect, Group, ItemKind, MAX_ITEM_LEVEL, MAX_PER_KIND, MAX_SPEED,
    Rarity, Tier,
};
pub use alignment::{AlignmentRules, BandDef, ByAlignment, Shifts};
pub use ally::AllyRules;
pub use anim::GlyphAnim;
pub use animated::{Animated, AnimatedDef, Body, ID_PREFIX, Line, Numbers, Ramp, SlotLine};
pub use aura::{Aura, CRY_TURN, Cry};
pub use background::{
    AgeRange, Background, Chapter, ChapterWhen, Effect as StoryEffect, Line as StoryLine, Marks,
    Points, Tier as StoryTier, Tiers, When,
};
pub use breath::Breath;
pub use burst::{BurstWhen, CreatureBurst};
pub use calendar::Calendar;
pub use calling::CallingDef;
pub use campaign::{CAMPAIGNS, CampaignDef, CampaignKind};
pub use chest::{ChestBand, ChestGold, ChestLoot, ChestRules};
pub use circles::{CircleStatuses, CircledStatus, school_index};
#[cfg(test)]
pub(crate) use class::strip_tables as strip_class_tables;
pub use class::{BonusTalents, ClassActive, ClassDef};
pub use companion::{
    CHAPTER as COMPANION_CHAPTER, CompanionRules, Growth as CompanionGrowth, TameRules,
};
pub use creature::{
    Budge, Combat, CorpseGifts, Corrodes, CorrodesWhat, CreatureDef, CreatureId, DEFAULT_SIGHT,
    Disarms, Hide, Incorporeal, NORMAL_SPEED, NaturalAttack, OnEat, SkillGift, is_proper,
    possessive, the, the_start,
};
pub use creature_rarity::{COMMON, CreatureRarity};
pub use crystal::{CrystalColour, Crystals, EnchantingRules, RareMaterials};
pub use damage_type::{CharColors, DamageTypeDef, DamageTypeId, MAX_DAMAGE_TYPES};
pub use danger::{Count, DangerRule, PlayerScale, Spawning};
pub use decipher::DecipherRules;
pub use dialogue::{
    AfterLines, Dialogue, DialogueId, GenderWords, Mutter, PartnerWords, PatronWords, ServantWords,
    Service, ServiceKind, ServiceTraining, Speaker, Topic,
};
pub use disease::{DiseaseDef, DiseaseId, SOURCES, Wading};
pub use drops::{Drop, DropFrom, DropGoods, TagDrops};
pub use dummy::DummyTraining;
pub use dungeon::{
    ChestOdds, DungeonDef, Endless, Generator, LayerBoss, LevelPlan, Portal, RoomOdds, ShopOdds,
    parse_branch_level, parse_instance, sibling,
};
pub use effect::{Darken, EffectDef, EffectLight, EffectShape, FILL_GLYPH, MAX_DARKEN, Shake};
pub use encounter::{
    Checks, Draw as EncounterDraw, EncounterEntry, EncounterRules, EncounterTable, EntryStance,
    Evade, Meetings, PhaseRates, PlayerPlaces, Positions, When as EncounterWhen,
};
pub use experience::{
    Discovery, Experience, FACTORS, Factor, Guideline, Kills, Levels, MAX_LEVEL, MAX_XP, pace_words,
};
pub use faction::{FactionDef, FactionId, Factions, PLAYER_FACTION, Stance};
pub use familiar::FamiliarRules;
pub use feint::FeintRules;
pub use files::ContentFiles;
pub use fletch::{FletchMakes, FletchRules};
pub use flow::{FLOW_SPEEDS, Flow};
pub use forage::{ForageGround, ForageRules};
pub use gaze::Gaze;
pub use gear::{Gear, GearLine, RACIAL_CHANCE};
pub use gear_condition::{Depth, GearConditions, Odds, PER_MILLE, TribeOdds};
pub use gem::{
    Cut, CutId, Cutting, GemDef, GemSize, GemSizeId, Gems, Ornament, ROUGH_SATURATION, dull,
};
pub use guild::{GuildDef, RankBy, RankDef};
pub use herbs::{Herb, HerbRules};
pub use horoscope::{FULL_MOON_WITHIN, Flavour, Horoscope, Lucky, Sign, SignActive, SignPart};
pub use hunger::{
    Corpses, EXERTIONS, Faint, Food, HungerDef, MAX_NUTRITION, Overeating, Rotten, Starve, States,
};
pub use intimidate::IntimidateRules;
pub use invest::InvestRules;
#[cfg(test)]
pub(crate) use item_class::BUILTIN as BUILTIN_ITEM_CLASSES;
pub use item_class::{ItemClassDef, ItemClassId, MAX_ITEM_CLASSES};
pub use kit::{Keep, Kit, KitItem, KitLineSpec, RandomScrolls};
pub use learning::{Band, Mishap, MishapEffect, StudyRules, drawn_among};
pub use leaving::{Laid, Leaving};
pub(crate) use level::validate_route;
pub use level::{
    CreatureSpawn, EdgeExit, Graves, HiddenLocation, LevelKind, LevelTemplate, Place, Region,
    StairsLink, parse_route,
};
pub use light::{Dips, Drift, HueDrift, LightId, LightPreset, Stability};
pub use material::{
    ANY, Fit, Fits, Found, ItemHarm, MaterialDef, MaterialId, ODD, POOR, Resists, VERY_RARE,
};
pub use mining::{Mined, MiningBand, MiningFind, MiningRules, Vein, Veins};
pub use miscast::{Miscast, MiscastRules};
pub use monster_pool::{MonsterPool, VaultMonsters};
pub use names::{NameLists, Names};
pub use object::{MAX_VALUE, ObjectDef, ObjectId, counted, weapon_noun};
pub use offer::TradeOffer;
pub use outcome::{
    Amount, Explosion, Harmful, Near, OutcomeDef, OutcomeEffect, OutcomeKind, OutcomeSummon,
    OutcomeTable, OutcomeTables, OutcomeTier,
};
pub use overworld::{Landscape, LocalArea, local_area_id, parse_local_area};
pub use part::{MAX_OBJECT_PARTS, MAX_PARTS, Part, PartDef, PartId, PartRole};
pub use particles::{ParticleEmitter, ParticleTrigger};
pub use patron::PatronRules;
pub use plural::Plurals;
pub use potion::{Draught, Drink, Flask, Splash};
pub use prayer::{Face, FaceWords, Faced, PrayerRules, PrayerUse, UseDef};
pub use prestige::{PrestigeDef, Table};
pub use quest::{
    Binding, Boon, GOLD, Goal, KillTarget, QuestDef, QuestId, QuestKind, QuestTexts, Reveal,
    RevealWhen, Reward, SkillReward, Waits,
};
pub(crate) use quest_roll::roll_known;
pub use quest_roll::{Candidate, Pick, Roll, Rolled, Variant, one_of};
pub use race::{HUNGER, LightBlind, RaceDef, Races};
pub use ranged::{RangeBand, RangedRules};
pub use recipe::{
    EnchantKind, InputKind, MOST_CRYSTALS, PartSource, ProfessionText, RecipeCount, RecipeDef,
    RecipeId, RecipeInput, RecipePart, RecipeResult, RecipeSettings, RecipeUse, Recipes,
};
pub use required::RequiredIds;
pub use resource::{Build, MAX_RESOURCE, MaxFrom, OverTime, ResourceDef, ResourceId};
pub use rig::{RigDef, RigId, RigRules, RigTakes};
pub use rod::{Rod, RodDoes, RodEffect, RodShape};
pub use room_effect::{
    Collapse, Dressing, Here, Hoard, Mirrors, Offer, RoomEffectDef, RoomEffectId, RoomEffects,
    SpecialRoom, Swarm, Themes, Treasure,
};
pub use room_haunt::Haunt;
pub use room_lore::RoomLoot;
pub use room_shrine::{Regard, Shrine};
pub(crate) use room_treasury::breach_fault;
pub use room_treasury::{Heaps, Sealed, SecretDoors, Treasury};
pub use ruin::{RuinOf, RuinRow, RuinRules};
pub use salvage::SalvageRules;
pub use scenario::ScenarioDef;
pub use scroll::{Cleansing, Reading, SpellScroll};
pub use scroll_label::ScrollLabelDef;
pub use search::{PassiveSearch, SEARCH_TURNS, SearchRules, Secret, secret_look};
pub use shop::{
    KEEPER, Keeper, MAX_FAVOUR, MAX_SHOPS, MOST_FAVOUR, ShopDef, ShopId, ShopPlan, Shops,
};
pub use size::{MEDIUM, SizeDef};
pub use skill::{OptionalSkill, SkillBook, SkillDef};
pub use sky::{MoonDef, PHASE_ONE, STANDARD_SIZE};
pub use slot::{MAX_SLOTS, SlotDef, SlotId};
pub use smite::SmiteRules;
pub use spell::{Bolt, Burst, BurstIce};
pub use spell_def::{
    ARC_REACH, CorpseUse, EYELESS, Grows, MAX_CONE, MAX_REACH, MAX_SPELL_LEVEL, OnlyTerrain, Ratio,
    Resolution, School, Shape, SpellBlink, SpellCharm, SpellData, SpellEffect, SpellId, SpellLook,
    SpellPurpose, SpellRise, SpellRules, SpellSummon, SpellWeather, SpellZone, ZoneKind,
};
pub use spell_ladder::{Ladder as SpellLadder, MAX_SCORE, RungValue, Unit as LadderUnit};
pub use spell_mod::{SpellMod, SpellModSpec, SpellMods, merge_rider};
pub use spell_rank::{MAX_SPELL_RANK, RankSpec, Ranks};
pub use spell_type::{SpellType, TypeBite, TypeRider};
pub use split::{CORPSE_SHARE, DiesInto, HALF_SHARE, Splits};
pub use stance::{CombatStance, StanceDef, Stances, percent_of};
pub use stat_training::{
    DeedRule, HAUL_GRAMS, ODDS_SCALE, StatDeed, StatRule, StatTraining, StatUse,
};
pub use status::{
    Affliction, CombatBonus, Drain, Healing, MAX_SEVERITY, OnHit, Penalty, Stacking, StatusDef,
    StatusId, Wound,
};
pub use steal::StealRules;
pub use stipend::StipendRules;
pub use stock::{Profession, Stock, StockGroup, StockMaterial};
pub use summon::Summon;
pub use surprise::SurpriseRules;
pub use swim::SwimRules;
pub use tag::{TAG_TARGET, spoken};
pub use talent::{
    AnyWeaponSkill, Choice, FreeBlowTrigger, Need, Needs, ResourceCost, SmiteRiderKind, SongOn,
    SystemDef, SystemId, TalentActive, TalentBook, TalentDef, TalentEffect, TalentId, TreeDef,
    TreeId,
};
pub use temper::TemperRules;
pub use terrain::{Breakable, Freeze, FreezeSpec, StairsDir, TerrainDef, TerrainId};
pub use theft::{Blinks, SMALL as SMALL_LOOT, Steals, Takes};
pub use throw::Throw;
pub use town::{TownDef, TownServant};
pub use trap::{Trap, TrapAvoid, TrapDef, TrapEffect, TrapId, TrapNoise};
pub use unique::{Power, UNIQUE, UniqueDef, Uniques};
pub use unseen::Invisible;
pub use vault::{DangerRange, Vault};
pub use vault_turn::Turn;
pub use vision::VisionProfile;
pub use wand::Wand;
pub use weapon_ability::{
    Active, ActiveAim, Categories, ExtraDice, FreeBlowKind, On, STAMINA, Trigger, WeaponAbility,
    WeaponAbilityId, WeaponDoes,
};
pub use weapon_skill::{
    Benefit, Grip, MAX_RANK, MOST_PERCENT, MarkRules, RANKS, Role, Section, SpellAttacks, Style,
    TwoShields, TwoWeapons, WeaponSkillDef, WeaponSkillId, WeaponSkillTable,
};
pub use weather::{Climate, MILD, MILD_SWING, WeatherDef};

use crate::skills::SkillId;
use level::{LegendEntry, legend_char};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

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
struct LegendFile {
    legend: BTreeMap<String, LegendEntry>,
}

/// All loaded and cross-validated content.
///
/// `data/lights.toml`, `data/vision.toml`, `data/factions.toml`, `data/dungeons.toml`,
/// `data/calendar.toml`, `data/sky.toml`, `data/materials/`, `data/parts.toml`, `data/objects/`,
/// `data/gems.toml`, `data/weather.toml`, `data/actions.toml`, `data/slots.toml`, `data/sizes.toml`,
/// `data/creature_rarities.toml`,
/// `data/races.toml`, `data/names.toml`, `data/classes/`, `data/skills.toml`,
/// `data/horoscope.toml`,
/// `data/effects.toml`, `data/room_effects.toml`, `data/weapon_skills.toml`, `data/stances.toml`,
/// `data/status.toml`, `data/resources.toml`, `data/diseases.toml`, `data/dialogue.toml`,
/// `data/plurals.toml`, `data/quests.toml`,
/// `data/traps.toml`, and `data/shops.toml` are optional (without effects, spell effects are not
/// drawn; without traps or shops there are none). Without vision
/// profiles, a single `normal` profile that needs no light is used, so content without lighting
/// stays fully visible; without sizes, everything is medium; without creature rarities, every
/// type is common; without factions, everyone is neutral; without a calendar, months are numbered;
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
/// fast; without diseases nobody falls ill; without dialogue, nobody talks; without plurals, every plural is the word and an "s";
/// without quests,
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
    /// Animated objects (`data/animated.toml`, `data/animated/`; spec §8 "Animated objects"):
    /// their creature types are generated into [`Content::creatures`].
    animated: Animated,
    /// Mimics (`data/mimics.toml`; spec §8 "Mimics"): a family generated into
    /// [`Content::creatures`] after the animated types.
    mimics: mimic::Mimics,
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
    /// Grimoires (`data/grimoires.toml`, spec §8 "Spells": "Learning"): generated into
    /// [`Content::objects`] after every object file, one for each spell scroll.
    grimoires: grimoire::Grimoires,
    /// Gem sizes, cuts, and ornaments (`data/gems.toml`); none without the file.
    gems: Option<Gems>,
    /// Material stock's words and professions (`data/stock.toml`); empty without the file.
    stock: Stock,
    /// The professions' recipes (`data/recipes/<skill>/`, `data/recipes.toml`); none without them.
    recipes: Recipes,
    /// Unique items (`data/uniques/`, each campaign's too; spec §8 "Magic items").
    uniques: Uniques,
    /// Equipment slots, in display order.
    slots: Vec<SlotDef>,
    /// Trap types, in file order.
    /// The chest bands (`data/chests.toml`; none without it).
    chests: ChestRules,
    traps: Vec<TrapDef>,
    trap_ids: BTreeMap<String, TrapId>,
    /// The chance of stepping onto a known trap unharmed (`[avoid]` in `data/traps.toml`).
    trap_avoid: TrapAvoid,
    /// The numbers of searching for hidden features (`data/search.toml`, spec §8 "Searching").
    search: SearchRules,
    /// The numbers of working an item at an anvil (`data/temper.toml`, spec §7 "Special rooms").
    temper: TemperRules,
    /// The condition creature gear spawns in (`data/gear_condition.toml`, spec §8 "Creature
    /// gear"); without the file, as made.
    gear_condition: GearConditions,
    /// Material drops by creature tag (`data/drops/`, `content/drops.rs`); each creature type's
    /// resolved rows are on it ([`CreatureDef::drops`]).
    drops: Vec<TagDrops>,
    /// The numbers of encounters (`data/encounters.toml`, spec §7 "Leaving an outdoor area").
    encounters: EncounterRules,
    /// What becomes of a creature that catches the player stealing (`data/steal.toml`, spec §8
    /// "Stealing"); none without the file.
    steal: Option<StealRules>,
    /// The numbers of being caught unawares (`data/surprise.toml`, spec §8 "Surprise and sneak
    /// attacks"); none without the file, when nobody is surprised.
    surprise: Option<SurpriseRules>,
    /// What holy wrath falls hardest on, and the pool a sin empties (`data/smite.toml`, spec §8
    /// "Class abilities": "Smite"); none without the file, when nothing is unholy.
    smite: Option<SmiteRules>,
    /// The campaigns' encounter tables (`campaigns/<id>/data/encounters/`), by campaign and id.
    encounter_tables: Vec<EncounterTable>,
    /// How arrows and bolts are made (`data/fletch.toml`, spec §8 "Fletching"); without it
    /// nobody fletches.
    fletch: Option<FletchRules>,
    /// The numbers of salvaging an item for its materials (`data/salvage.toml`); without it
    /// nobody salvages.
    salvage: Option<SalvageRules>,
    /// The traps the craftsperson's Trapwright rigs (`data/rigs.toml`, spec §8 "Traps"); none
    /// without the file, when Trapwright is not usable yet.
    rigs: Option<RigRules>,
    /// What foraging in the wild gives (`data/forage.toml`, spec §8 "Foraging"); none without
    /// the file.
    forage: Option<ForageRules>,
    /// What the glowing fungus gives a gatherer (`data/herbs.toml`, spec §8 "Skills": Herbalism);
    /// none without the file, when nobody gathers.
    herbs: Option<HerbRules>,
    /// What a wall a miner digs through gives (`data/mining.toml`, spec §8 "Digging"); none
    /// without the file, when nothing is mined.
    mining: Option<MiningRules>,
    /// The numbers of feinting (`data/feint.toml`, spec §8 "Skills": Bluff); none without the
    /// file, when nobody feints.
    feint: Option<FeintRules>,
    /// What an aristocrat's stipend grows by with each level (`data/stipend.toml`, spec §8
    /// "Class abilities": Stipend); none without the file, when it never grows.
    stipend: Option<StipendRules>,
    /// What a merchant's investment pays and costs (`data/invest.toml`, spec §8 "Class
    /// abilities": Invest); none without the file, when no one invests.
    invest: Option<InvestRules>,
    /// What a craftsperson's patron pays for a commission (`data/patron.toml`, spec §8 "Class
    /// abilities": Patron); none without the file, when no one commissions work.
    patron: Option<PatronRules>,
    /// The numbers of intimidating (`data/intimidate.toml`, spec §8 "Skills": Intimidate); none
    /// without the file, when nobody intimidates.
    intimidate: Option<IntimidateRules>,
    /// What area spells do to the items lying in them (`data/ruin.toml`, spec §8 "Spells":
    /// "Ruin"); empty without the file, when nothing is ruined.
    ruin: RuinRules,
    /// The numbers of swimming in deep water (`data/swim.toml`, spec §8 "Skills": Swim); none
    /// without the file, when nobody swims.
    swim: Option<SwimRules>,
    /// The penalties of shots by distance (`data/ranged.toml`, spec §8 "Ranged combat"); none
    /// without the file.
    ranged: RangedRules,
    /// The priest's prayer (`data/prayer.toml`, spec §8 "Class abilities"): the strain, the
    /// faces, the uses in their order; none without the file, when every prayer is refused.
    prayer: Option<PrayerRules>,
    /// The constants of the ally core (`data/allies.toml`, spec §8 "Allies"); none without the
    /// file, when no ally can be made.
    allies: Option<AllyRules>,
    /// The animal companion's data (`data/companions.toml`, spec §8 "Allies"); none without
    /// the file, when no companion can be made.
    companions: Option<CompanionRules>,
    /// The witch's familiar's data (`data/familiars.toml`, spec §8 "Allies"); none without the
    /// file, when no familiar can be made.
    familiars: Option<FamiliarRules>,
    /// The rules of stat training (`data/stats.toml`, spec §8 "Stats").
    stat_training: StatTraining,
    factions: Factions,
    lights: Vec<LightPreset>,
    light_ids: BTreeMap<String, LightId>,
    visions: Vec<VisionProfile>,
    vision_ids: BTreeMap<String, usize>,
    /// In `data/sizes.toml` order.
    sizes: Vec<SizeDef>,
    /// In `data/creature_rarities.toml` order.
    creature_rarities: Vec<CreatureRarity>,
    /// The campaigns (spec §3), in [`ContentFiles::campaign_order`].
    campaigns: Vec<CampaignDef>,
    levels: BTreeMap<String, LevelTemplate>,
    /// The campaign of each hand-made level: `None` for one outside every campaign.
    level_campaigns: BTreeMap<String, Option<String>>,
    scenarios: BTreeMap<String, ScenarioDef>,
    dungeons: BTreeMap<String, DungeonDef>,
    /// The default odds of a draw (spec §7 "Danger levels"), `data/dungeons.toml`'s
    /// `[defaults]`; `None` without them (then no area may give a danger).
    draw_odds: Option<danger::Odds>,
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
    /// The prestige classes (`data/prestige/`), in table order (spec §8 "Prestige classes").
    prestige: Vec<PrestigeDef>,
    /// The alignment rules (`data/alignment.toml`): none without it.
    alignment: Option<AlignmentRules>,
    experience: Experience,
    horoscope: Option<Horoscope>,
    /// Background stories (`data/background.toml`, `data/background/`): none without the file.
    background: Option<Background>,
    /// Spell effects, in `data/effects.toml` order: presentation only.
    effects: Vec<EffectDef>,
    /// Spell circles' statuses (`data/circles.toml`): a circled status sheds its school's light, so the
    /// rows' statuses and schools are fingerprinted.
    circles: CircleStatuses,
    hunger: Option<HungerDef>,
    /// The corpse object of each creature type, by creature id (`None`: it leaves none).
    corpses: Vec<Option<ObjectId>>,
    weapon_skills: WeaponSkillTable,
    /// What weapon skill and style ranks teach (`data/weapon_abilities/`, the combat
    /// progression design §4), in path order.
    weapon_abilities: Vec<WeaponAbility>,
    /// The spells' cost table (`data/spells.toml`, spec §8 "Spells"); none without the file.
    spell_rules: Option<SpellRules>,
    /// The study check and its time (`data/study.toml`, spec §8 "Spells"); none without it.
    study: Option<StudyRules>,
    /// The check of reading a scroll (`data/decipher.toml`, spec §8 "Scrolls"); none without it.
    decipher: Option<DecipherRules>,
    /// The magic crystals' colours (`data/crystals.toml`, docs/dev/enchanting.md); none without
    /// it.
    crystals: Option<Crystals>,
    /// Enchanting's numbers (`data/enchanting.toml`); none without it.
    enchanting: Option<EnchantingRules>,
    /// What a failed study may bring (`data/mishaps/`), in path order.
    mishaps: Vec<Mishap>,
    /// The Spellcraft check every cast makes (`data/miscast.toml`, spec §8 "Spells": "Miscasts");
    /// none without it, and then no cast miscasts.
    miscast: Option<MiscastRules>,
    /// The miscast table (`data/miscasts/`), ordered by its totals.
    miscasts: Vec<Miscast>,
    /// What rods do (`data/rod_effects/`, spec §8 "Magic items": "Rods"), in path order.
    rod_effects: Vec<RodEffect>,
    /// Spells (`data/spells/`, spec §8 "Spells"), in path order; none without the files.
    spells: Vec<SpellData>,
    /// The statuses `kind = "hex"` spells lay, in status order (computed from `spells`).
    hex_statuses: Vec<StatusId>,
    /// Talents (`data/talents.toml` and `data/talents/`, spec §8 "Talents"); none without the
    /// files.
    talents: TalentBook,
    /// Special rooms' effects (`data/room_effects.toml`).
    room_effects: RoomEffects,
    /// What a place's ability draws (`data/outcomes/`, spec §7 "Special rooms"), by id.
    outcomes: OutcomeTables,
    /// Prefixes, suffixes, and how loot rolls them (`data/affixes.toml`).
    affixes: Affixes,
    stances: Stances,
    /// Unidentified scrolls' labels (`data/scroll_labels.toml`): none, every scroll known on
    /// sight, without it.
    scroll_labels: Option<ScrollLabelDef>,
    /// Unidentified wands' labels (`data/wand_labels.toml`, the scroll labels' format): none,
    /// every wand known on sight.
    wand_labels: Option<ScrollLabelDef>,
    /// Unidentified rods' labels (`data/rod_labels.toml`, the scroll labels' format): none,
    /// every rod known on sight.
    rod_labels: Option<ScrollLabelDef>,
    /// Unidentified potions' labels (`data/potion_labels.toml`, the scroll labels' format, a
    /// word and the type's colour): none, every potion known on sight.
    potion_labels: Option<ScrollLabelDef>,
    /// In `data/status.toml` order: most important first.
    statuses: Vec<StatusDef>,
    /// Alternative resources (`data/resources.toml`, spec §8 "Resources"), in file order; none
    /// without the file.
    resources: Vec<ResourceDef>,
    status_ids: BTreeMap<String, StatusId>,
    healing: Healing,
    /// In `data/diseases.toml` order.
    diseases: Vec<DiseaseDef>,
    disease_ids: BTreeMap<String, DiseaseId>,
    /// The status diseases show as ("Diseased"), with any disease.
    disease_status: Option<StatusId>,
    /// What creatures say (`data/dialogue.toml`), in file order.
    dialogues: Vec<Dialogue>,
    /// The plural dictionary (`data/plurals.toml` and each campaign's).
    plurals: Plurals,
    /// What people ask of the player (`data/quests.toml`), in file order.
    quests: Vec<QuestDef>,
    /// Guilds and their ranks (`data/guilds/`, each campaign's too), in table order.
    guilds: Vec<GuildDef>,
    /// Towns and the goodwill their quests earn (each campaign's `data/towns/`), in table order.
    towns: Vec<TownDef>,
    /// The callings into prestige classes (each campaign's `data/callings/`), in table order.
    callings: Vec<CallingDef>,
    /// Shops (`data/shops.toml`); none without the file.
    shops: Option<Shops>,
    /// The ids the engine names in code, resolved at load (`content/required.rs`).
    ids: RequiredIds,
}

impl Content {
    /// Loads the built-in content.
    pub fn builtin() -> Result<Self, ContentError> {
        Self::load(&ContentFiles::builtin())
    }

    pub fn load(files: &ContentFiles) -> Result<Self, ContentError> {
        files.read.borrow_mut().clear();
        let mut content = Content {
            terrain: Vec::new(),
            terrain_ids: BTreeMap::new(),
            ice: Vec::new(),
            creatures: Vec::new(),
            creature_ids: BTreeMap::new(),
            animated: Animated::default(),
            mimics: mimic::Mimics::default(),
            damage_types: Vec::new(),
            materials: Vec::new(),
            material_ids: BTreeMap::new(),
            parts: Vec::new(),
            item_classes: Vec::new(),
            objects: Vec::new(),
            object_ids: BTreeMap::new(),
            grimoires: grimoire::Grimoires::default(),
            gems: None,
            stock: Stock::default(),
            recipes: Recipes::default(),
            uniques: Uniques::default(),
            slots: Vec::new(),
            chests: ChestRules::default(),
            traps: Vec::new(),
            trap_ids: BTreeMap::new(),
            trap_avoid: TrapAvoid::default(),
            search: SearchRules::default(),
            temper: TemperRules::default(),
            gear_condition: GearConditions::default(),
            drops: Vec::new(),
            encounters: EncounterRules::default(),
            steal: None,
            surprise: None,
            smite: None,
            encounter_tables: Vec::new(),
            fletch: None,
            salvage: None,
            rigs: None,
            forage: None,
            herbs: None,
            mining: None,
            feint: None,
            stipend: None,
            invest: None,
            patron: None,
            intimidate: None,
            ruin: RuinRules::default(),
            decipher: None,
            crystals: None,
            enchanting: None,
            swim: None,
            ranged: RangedRules::default(),
            prayer: None,
            allies: None,
            companions: None,
            familiars: None,
            stat_training: StatTraining::default(),
            factions: Factions::default(),
            lights: Vec::new(),
            light_ids: BTreeMap::new(),
            visions: Vec::new(),
            vision_ids: BTreeMap::new(),
            sizes: vec![SizeDef::medium()],
            creature_rarities: vec![CreatureRarity::common()],
            campaigns: Vec::new(),
            levels: BTreeMap::new(),
            level_campaigns: BTreeMap::new(),
            scenarios: BTreeMap::new(),
            dungeons: BTreeMap::new(),
            draw_odds: None,
            vaults: Vec::new(),
            calendar: Calendar::default(),
            moons: Vec::new(),
            climate: Climate::default(),
            actions: ActionCosts::default(),
            skills: SkillBook::default(),
            races: Races::default(),
            names: Names::default(),
            classes: Vec::new(),
            prestige: Vec::new(),
            alignment: None,
            experience: Experience::default(),
            horoscope: None,
            background: None,
            effects: Vec::new(),
            circles: CircleStatuses::default(),
            hunger: None,
            corpses: Vec::new(),
            weapon_skills: WeaponSkillTable::default(),
            room_effects: RoomEffects::default(),
            outcomes: OutcomeTables::default(),
            spell_rules: None,
            study: None,
            mishaps: Vec::new(),
            miscast: None,
            miscasts: Vec::new(),
            rod_effects: Vec::new(),
            weapon_abilities: Vec::new(),
            spells: Vec::new(),
            hex_statuses: Vec::new(),
            talents: TalentBook::default(),
            affixes: Affixes::default(),
            stances: Stances::default(),
            scroll_labels: None,
            wand_labels: None,
            rod_labels: None,
            potion_labels: None,
            statuses: Vec::new(),
            resources: Vec::new(),
            status_ids: BTreeMap::new(),
            healing: Healing::default(),
            diseases: Vec::new(),
            disease_ids: BTreeMap::new(),
            disease_status: None,
            dialogues: Vec::new(),
            plurals: Plurals::default(),
            quests: Vec::new(),
            guilds: Vec::new(),
            towns: Vec::new(),
            callings: Vec::new(),
            shops: None,
            ids: RequiredIds::default(),
        };
        content.campaigns = campaign::parse(files)?;
        content.load_plurals(files)?;
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
        let path = "data/creature_rarities.toml";
        if let Some(text) = files.get_optional(path) {
            content.creature_rarities = creature_rarity::parse(path, text)?;
        }
        let path = "data/skills.toml";
        if let Some(text) = files.get_optional(path) {
            content.skills = SkillBook::parse(path, text)?;
        }
        let path = "data/damage_types.toml";
        if let Some(text) = files.get_optional(path) {
            content.damage_types = damage_type::parse(path, text)?;
        }
        let path = "data/scroll_labels.toml";
        if let Some(text) = files.get_optional(path) {
            content.scroll_labels = Some(ScrollLabelDef::parse(path, text)?);
        }
        let path = "data/wand_labels.toml";
        if let Some(text) = files.get_optional(path) {
            content.wand_labels = Some(ScrollLabelDef::parse(path, text)?);
        }
        let path = "data/rod_labels.toml";
        if let Some(text) = files.get_optional(path) {
            content.rod_labels = Some(ScrollLabelDef::parse(path, text)?);
        }
        let path = "data/potion_labels.toml";
        if let Some(text) = files.get_optional(path) {
            content.potion_labels = Some(ScrollLabelDef::parse(path, text)?);
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
            let resist = |m: &BTreeMap<String, i32>| content.resolve_per_kind("resist", m, false);
            content.races = Races::parse(
                path, text, has_vision, size_index, skill_id, weapons, resist,
            )?;
        }
        let path = "data/names.toml";
        if let Some(text) = files.get_optional(path) {
            content.names = Names::parse(path, text, |id| content.race(id).is_some())?;
        }
        // The class table (spec §8 "Character creation"): optional, so fixtures may go without.
        if files.table("classes").next().is_some() {
            let skill_id = |id: &str| content.skill_id(id);
            let table = files.table("classes");
            content.classes = ClassDef::parse(table, skill_id, &content.weapon_skills)?;
            for class in &content.classes {
                class.check_races(|id| content.race(id).is_some())?;
            }
        }
        // Prestige classes (spec §8 "Prestige classes"): a class in what a level reads.
        {
            let skill_id = |id: &str| content.skill_id(id);
            let is_class = |id: &str| content.class(id).is_some();
            let table = files.table("prestige");
            content.prestige = PrestigeDef::parse(table, skill_id, is_class)?;
        }
        let path = "data/alignment.toml";
        if let Some(text) = files.get_optional(path) {
            content.alignment = Some(AlignmentRules::parse(path, text)?);
        }
        let path = "data/experience.toml";
        if let Some(text) = files.get_optional(path) {
            content.experience = Experience::parse(path, text)?;
        }
        // Each class's highest level: its own, else the experience file's default.
        let (default, highest) = (
            content.experience.class_max(),
            content.experience.max_level(),
        );
        for class in &mut content.classes {
            class.settle_cap(default, highest)?;
        }
        // Statuses before talents, which may name them (a cooldown, a critical's status).
        content.load_statuses(files)?;
        // Resources before talents, whose actives may cost them.
        let path = "data/resources.toml";
        if let Some(text) = files.get_optional(path) {
            let status = |name: &str| {
                let id = content.status_id(name)?;
                Some((id, content.status(id).eases.is_some()))
            };
            content.resources = resource::parse(path, text, status)?;
        }
        // The affix table: optional, so loot without it is always normal. After lights, skills,
        // damage types, and resources, which its affixes name.
        if files.table("affixes").next().is_some() {
            let skill: &dyn Fn(&str) -> Option<SkillId> = &|id| content.skill_id(id);
            let damage: &dyn Fn(&str) -> Option<DamageTypeId> = &|id| content.damage_type_id(id);
            let resource: &dyn Fn(&str) -> Option<ResourceId> = &|id| content.resource_id(id);
            let lookups = affix::Lookups {
                light: &|id| content.light_id(id),
                skill: (!content.skills().is_empty()).then_some(skill),
                damage_type: (!content.damage_types().is_empty()).then_some(damage),
                resource: (!content.resources().is_empty()).then_some(resource),
            };
            content.affixes = affix::parse(files.table("affixes"), &lookups)?;
        }
        content.check_follows_end()?;
        // Spells (spec §8 "Spells") after damage types and statuses, which they name.
        let path = "data/spells.toml";
        let rules = files.get_optional(path).map(|text| (path, text));
        let table = files.table("spells").filter(|(p, _)| *p != path);
        (content.spell_rules, content.spells) = spell_def::parse(rules, table, &content)?;
        content.hex_statuses = (content.spells.iter())
            .filter(|s| s.kind == SpellPurpose::Hex)
            .filter_map(|s| s.effect.status.map(|(status, _)| status))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        // Rod effects (spec §8 "Magic items": "Rods") name statuses; rods (objects) name them.
        content.rod_effects = rod::parse(files.table("rod_effects"), &content)?;
        // The talent table (spec §8 "Talents"): optional, so no level asks for a pick without it.
        if files.table("talents").next().is_some() {
            content.talents = talent::parse(files.table("talents"), &content)?;
        }
        content.check_class_tables()?;
        // Weapon abilities (the combat progression design §4) name weapon skills, statuses,
        // damage types, and the talent table's systems.
        content.weapon_abilities =
            weapon_ability::parse(files.table("weapon_abilities"), &content)?;
        // The races' granted talents (Versatile) are talents of the racial tree.
        race::check_talents(&content.races, &content.talents)?;
        let path = "data/horoscope.toml";
        if let Some(text) = files.get_optional(path) {
            let mut horoscope = Horoscope::parse(path, text)?;
            horoscope::resolve_flavour(&mut horoscope, |name| content.light_id(name))?;
            content.horoscope = Some(horoscope);
        }
        // The signs' gifts are talents, and talents may ask for a sign: checked both ways.
        horoscope::check_talents(content.horoscope.as_ref(), &content.talents)?;
        content.load_terrain(files)?;
        content.talents.check_terrain(&content)?;
        content.load_diseases(files)?;
        let path = "data/room_effects.toml";
        if let Some(text) = files.get_optional(path) {
            let terrain = |name: &str| {
                let id = content.terrain_id(name)?;
                Some((id, content.terrain(id)))
            };
            let status = |name: &str| {
                let id = content.status_id(name)?;
                Some((id, content.status(id)))
            };
            content.room_effects = RoomEffects::parse_with(path, text, terrain, status)?;
        }
        content.load_factions(files)?;
        content.room_effects.check_factions(&content.factions)?;
        // Objects before creatures: an animated type is generated from its body's object.
        content.load_objects(files)?;
        // Stock after objects and skills: its rows name material groups and skills.
        let path = "data/stock.toml";
        if let Some(text) = files.get_optional(path) {
            content.stock = Stock::parse(path, text, &content)?;
        }
        let ids = &content.material_ids;
        (content.factions)
            .resolve_gear(|name| ids.get(name).copied())
            .map_err(|m| ContentError::new("data/factions.toml", m))?;
        content.animated = Animated::parse(files, &content)?;
        // Chests before creatures: a mimic's band is its chest's, and its hold the band's.
        let path = "data/chests.toml";
        if let Some(text) = files.get_optional(path) {
            content.chests = ChestRules::parse(path, text)?;
        }
        content.mimics = mimic::Mimics::parse(files, &content)?;
        content.load_creatures(files)?;
        spell_type::lay_immunities(&mut content);
        // A den's own population (an armoury's) names creatures.
        let mut room_effects = std::mem::take(&mut content.room_effects);
        let creature =
            |name: &str| (content.creature_id(name)).map(|c| (c, content.creature(c).unique));
        room_effects.resolve_populations(creature)?;
        content.room_effects = room_effects;
        content.load_gems(files)?;
        // Recipes after stock, terrain, and gems: a stock part's materials must be the
        // profession's, and an ornament needs the result's ornament part.
        let settings = (files.get_optional(recipe::SETTINGS)).map(|text| (recipe::SETTINGS, text));
        content.recipes = recipe::parse(settings, files.table("recipes"), &content)?;
        // Unique items before everything that may place one (gear, shops, levels, kits, quests).
        content.uniques = unique::parse(files.table("uniques"), &content)?;
        content.load_gear()?;
        content.load_throws()?;
        content.load_summons()?;
        content.load_splits()?;
        content.load_drops(files)?;
        // Summon spells' pools, once every creature type is known.
        spell_def::resolve_summons(&mut content)?;
        spell_def::check_only_tags(&content).map_err(|m| ContentError::new("data/spells", m))?;
        let path = "data/shops.toml";
        if let Some(text) = files.get_optional(path) {
            content.shops = Some(Shops::parse(path, text, &content)?);
        }
        let path = "data/effects.toml";
        if let Some(text) = files.get_optional(path) {
            content.effects = effect::parse(path, text)?;
        }
        // Spell circles name statuses and the spells that lay them.
        let path = "data/circles.toml";
        if let Some(text) = files.get_optional(path) {
            content.circles = circles::parse(path, text, &content)?;
            circles::shed_lights(&mut content);
        }
        scroll::check_spells(&content).map_err(|m| ContentError::new("data/objects", m))?;
        talent::check_heirlooms(&content).map_err(|m| ContentError::new("data/talents", m))?;
        talent::check_trades(&content).map_err(|m| ContentError::new("data/talents", m))?;
        spell_def::check_effects(&content).map_err(|m| ContentError::new("data/spells", m))?;
        breath::check_effects(&content)?;
        burst::check_effects(&content)?;
        let path = "data/traps.toml";
        if let Some(text) = files.get_optional(path) {
            let (traps, ids, avoid) = trap::parse(path, text, &content)?;
            (content.traps, content.trap_ids, content.trap_avoid) = (traps, ids, avoid);
        }
        // Learning spells (spec §8 "Spells"): mishaps name statuses and creatures.
        let path = "data/study.toml";
        if let Some(text) = files.get_optional(path) {
            content.study = Some(StudyRules::parse(path, text)?);
        }
        content.mishaps = learning::parse(files.table("mishaps"), &content)?;
        // Miscasts (spec §8 "Spells": "Miscasts"): the table names statuses, creatures, and damage
        // types.
        let path = "data/miscast.toml";
        if let Some(text) = files.get_optional(path) {
            content.miscast = Some(MiscastRules::parse(path, text, &content.factions)?);
        }
        content.miscasts = miscast::parse(files.table("miscasts"), &content)?;
        let path = "data/decipher.toml";
        if let Some(text) = files.get_optional(path) {
            content.decipher = Some(DecipherRules::parse(path, text)?);
        }
        let path = "data/encounters.toml";
        if let Some(text) = files.get_optional(path) {
            content.encounters = EncounterRules::parse(path, text)?;
        }
        let path = "data/search.toml";
        if let Some(text) = files.get_optional(path) {
            content.search = SearchRules::parse(path, text)?;
        }
        let path = "data/temper.toml";
        if let Some(text) = files.get_optional(path) {
            content.temper = TemperRules::parse(path, text)?;
        }
        let path = "data/gear_condition.toml";
        if let Some(text) = files.get_optional(path) {
            content.gear_condition = GearConditions::parse(path, text, &content)?;
        }
        let path = "data/steal.toml";
        if let Some(text) = files.get_optional(path) {
            content.steal = Some(StealRules::parse(path, text, &content.factions)?);
        }
        let path = "data/surprise.toml";
        if let Some(text) = files.get_optional(path) {
            content.surprise = Some(SurpriseRules::parse(path, text, &content)?);
        }
        let path = "data/smite.toml";
        if let Some(text) = files.get_optional(path) {
            content.smite = Some(SmiteRules::parse(path, text, &content)?);
        }
        let path = "data/fletch.toml";
        if let Some(text) = files.get_optional(path) {
            content.fletch = Some(FletchRules::parse(path, text, &content)?);
        }
        let path = "data/salvage.toml";
        if let Some(text) = files.get_optional(path) {
            content.salvage = Some(SalvageRules::parse(path, text)?);
        }
        let path = "data/rigs.toml";
        if let Some(text) = files.get_optional(path) {
            content.rigs = Some(RigRules::parse(path, text, &content)?);
        }
        let path = "data/forage.toml";
        if let Some(text) = files.get_optional(path) {
            content.forage = Some(ForageRules::parse(path, text, &content)?);
        }
        let path = "data/herbs.toml";
        if let Some(text) = files.get_optional(path) {
            content.herbs = Some(HerbRules::parse(path, text, &content)?);
        }
        let path = "data/feint.toml";
        if let Some(text) = files.get_optional(path) {
            content.feint = Some(FeintRules::parse(path, text, &content)?);
        }
        let path = "data/stipend.toml";
        if let Some(text) = files.get_optional(path) {
            content.stipend = Some(StipendRules::parse(path, text, &content)?);
        }
        let path = "data/invest.toml";
        if let Some(text) = files.get_optional(path) {
            content.invest = Some(InvestRules::parse(path, text)?);
        }
        let path = "data/patron.toml";
        if let Some(text) = files.get_optional(path) {
            content.patron = Some(PatronRules::parse(path, text)?);
        }
        let path = "data/intimidate.toml";
        if let Some(text) = files.get_optional(path) {
            content.intimidate = Some(IntimidateRules::parse(path, text, &content)?);
        }
        let path = "data/ruin.toml";
        if let Some(text) = files.get_optional(path) {
            content.ruin = RuinRules::parse(path, text, &content)?;
        }
        let path = "data/swim.toml";
        if let Some(text) = files.get_optional(path) {
            content.swim = Some(SwimRules::parse(path, text, &content)?);
        }
        let path = "data/ranged.toml";
        if let Some(text) = files.get_optional(path) {
            content.ranged = RangedRules::parse(path, text)?;
        }
        let path = "data/prayer.toml";
        if let Some(text) = files.get_optional(path) {
            content.prayer = Some(PrayerRules::parse(path, text, &content)?);
        }
        let path = "data/allies.toml";
        if let Some(text) = files.get_optional(path) {
            content.allies = Some(AllyRules::parse(path, text)?);
        }
        let path = "data/stats.toml";
        if let Some(text) = files.get_optional(path) {
            content.stat_training = StatTraining::parse(path, text)?;
        }
        content.dialogues = dialogue::parse(files.table("dialogue"), &content)?;
        content.load_hunger(files)?;
        // Outcome tables (spec §7 "Special rooms") last of the rules' data: they name terrain,
        // statuses, objects, damage types, and spell effects; then the rooms that offer them.
        content.outcomes = OutcomeTables::parse(files.table("outcomes"), &content)?;
        potion::check_gambles(&content)?;
        let lights = content.light_ids.clone();
        let outcomes = content.outcomes.clone();
        (content.room_effects).resolve_places(&outcomes, |name| lights.get(name).copied())?;
        // A scriptorium's loot and training name item classes (loaded with the objects) and skills.
        let mut rooms = std::mem::take(&mut content.room_effects);
        let skills = !content.skills().is_empty();
        rooms.resolve_lore(
            |n| content.item_class_id(n),
            (|n| content.skill_id(n), skills),
        )?;
        content.room_effects = rooms;

        let path = "data/legend.toml";
        let legend_file: LegendFile =
            toml::from_str(files.get(path)?).map_err(|e| ContentError::new(path, e.to_string()))?;
        let mut default_legend = BTreeMap::new();
        for (key, entry) in legend_file.legend {
            default_legend.insert(legend_char(path, &key)?, entry);
        }

        let path = "data/dungeons.toml";
        if let Some(text) = files.get_optional(path) {
            content.draw_odds =
                danger::DefaultsFile::parse(text).map_err(|m| ContentError::new(path, m))?;
        }
        let mut seen: BTreeMap<String, &str> = BTreeMap::new();
        for (path, id, text) in files.in_dir("levels") {
            if let Some(first) = seen.insert(id.to_string(), path) {
                return Err(duplicate_id("level", id, path, first));
            }
            let level = level::parse_level(path, id, text, &content, &default_legend)?;
            if !vault::parse_dungeons(path, text)?.is_empty() {
                return Err(ContentError::new(
                    path,
                    "`dungeons`: only a vault names where it stamps",
                ));
            }
            if vault::parse_turns(path, text)?.is_some() {
                return Err(ContentError::new(path, "`turns`: only a vault turns"));
            }
            if vault::parse_danger(path, text)?.is_some() {
                return Err(ContentError::new(
                    path,
                    "`danger = { from, to }`: only a vault keeps to a range of dangers",
                ));
            }
            if let Some((at, _)) = level.loot.first() {
                return Err(ContentError::new(
                    path,
                    format!("`loot` at {at}: loot is rolled only in vaults"),
                ));
            }
            if let Some(group) = level.monsters.first() {
                return Err(ContentError::new(
                    path,
                    format!(
                        "`monsters` at {}: a monster pool is drawn only in vaults",
                        group.cells[0]
                    ),
                ));
            }
            let campaign = campaign::of_path(path).map(str::to_string);
            content.level_campaigns.insert(id.to_string(), campaign);
            content.levels.insert(id.to_string(), level);
        }
        overworld::link_edge_exits(&mut content.levels);
        content.load_kits()?;
        content.load_background(files)?;
        // The companion's data names creatures, races, and the story's chapter.
        let path = "data/companions.toml";
        if let Some(text) = files.get_optional(path) {
            content.companions = Some(CompanionRules::parse(path, text, &content)?);
        }
        // The familiar's names creatures, races, and a hex spell.
        let path = "data/familiars.toml";
        if let Some(text) = files.get_optional(path) {
            content.familiars = Some(FamiliarRules::parse(path, text, &content)?);
        }
        // The crystals name objects, creature tags, and affix categories.
        let path = "data/crystals.toml";
        if let Some(text) = files.get_optional(path) {
            content.crystals = Some(Crystals::parse(path, text, &content)?);
        }
        let path = "data/enchanting.toml";
        if let Some(text) = files.get_optional(path) {
            content.enchanting = Some(EnchantingRules::parse(path, text)?);
        }
        let mut seen: BTreeMap<String, &str> = BTreeMap::new();
        for (path, id, text) in files.in_dir("scenarios") {
            if let Some(first) = seen.insert(id.to_string(), path) {
                return Err(duplicate_id("scenario", id, path, first));
            }
            let mut scenario = ScenarioDef::parse(path, id, text)?;
            let err = |m: String| ContentError::new(path, format!("carry: {m}"));
            scenario.carry = object::parse_stacks(&scenario.carry_spec, &content).map_err(err)?;
            let err = |m: String| ContentError::new(path, format!("wield: {m}"));
            scenario.wield = object::parse_stacks(&scenario.wield_spec, &content).map_err(err)?;
            for name in &scenario.spells_spec {
                let spell = (content.spell_id(name)).ok_or_else(|| {
                    ContentError::new(path, format!("spells: unknown spell `{name}`"))
                })?;
                scenario.spells.push(spell);
            }
            scenario.campaign = campaign::of_path(path).map(str::to_string);
            content.scenarios.insert(id.to_string(), scenario);
        }
        let mut seen: BTreeMap<String, &str> = BTreeMap::new();
        for (path, id, text) in files.in_dir("vaults") {
            if path == "vaults/README.toml" {
                continue; // the authors' rules, comments only
            }
            if let Some(first) = seen.insert(id.to_string(), path) {
                return Err(duplicate_id("vault", id, path, first));
            }
            // Its range first: the level parser would refuse a bad one without saying why.
            let danger = vault::parse_danger(path, text)?.unwrap_or_default();
            let template = level::parse_level(path, id, text, &content, &default_legend)?;
            let mut vault =
                Vault::new(template, &content).map_err(|m| ContentError::new(path, m))?;
            vault.campaign = campaign::of_path(path).map(str::to_string);
            vault.dungeons = vault::parse_dungeons(path, text)?;
            vault.turns = vault::parse_turns(path, text)?.unwrap_or(true);
            vault.danger = danger;
            content.vaults.push(vault);
        }
        content.load_dungeons(files)?;
        // After the branches: their `ores` are checked against the table.
        let path = "data/mining.toml";
        if let Some(text) = files.get_optional(path) {
            content.mining = Some(MiningRules::parse(path, text, &mut content)?);
        }
        content.fill_generic();
        for v in &content.vaults {
            (v.check_dungeons(&content)).map_err(|m| ContentError::new(&v.file(), m))?;
        }
        // A scenario outside every campaign (a test fixture) plays in its start level's.
        let loose: Vec<(String, Option<String>)> = (content.scenarios.values())
            .filter(|s| s.campaign.is_none())
            .map(|s| {
                let campaign = content.level_campaign(&s.start_level);
                (s.id.clone(), campaign.map(str::to_string))
            })
            .collect();
        for (id, campaign) in loose {
            if let Some(s) = content.scenarios.get_mut(&id) {
                s.campaign = campaign;
            }
        }
        content.quests = quest::parse(files.table("quests"), &content)?;
        content.guilds = guild::parse(files.table("guilds"), &content)?;
        content.towns = town::parse(files.table("towns"), &content)?;

        content.callings = calling::parse(files.table("callings"), &content)?;
        content.encounter_tables = encounter::parse(files.table("encounters"), &content)?;
        content.ids = required::resolve(&content)?;
        content.validate()?;
        content.check_topic_routes()?;
        content.check_topic_callings()?;
        content.check_campaigns()?;
        files.check_all_read()?;
        Ok(content)
    }

    /// The plural dictionary: `data/plurals.toml`, then each campaign's `data/plurals.toml`, in
    /// one (a word given two plurals is an error).
    fn load_plurals(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let campaigns = files.campaign_order();
        let paths = std::iter::once("data/plurals.toml".to_string()).chain(
            (campaigns.iter()).map(|c| format!("{}/{c}/data/plurals.toml", campaign::CAMPAIGNS)),
        );
        for path in paths {
            if let Some(text) = files.get_optional(&path) {
                self.plurals.add(&path, text)?;
            }
        }
        Ok(())
    }
}
