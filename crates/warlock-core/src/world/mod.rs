//! The simulation: world state and the turn pipeline (spec §8).
//!
//! Turn phases, in order: player action → the other actors on the level act in time order until
//! the player may act again, the clock advancing to the moment of each action (the energy
//! scheduler, `schedule.rs`; [`World::creature_turn`] is one creature's action) → environment →
//! perception (sight, light, memory) → perceived events are returned to the caller.
//!
//! Time (spec §8): every action that takes time has a cost in energy (`data/actions.toml`, 1000
//! for a normal action; [`ActionKind`]) and a base time: 6 s ([`crate::content::BASE_TURN`])
//! for everything but a step on the overworld, whose base is the terrain's travel time. Its
//! energy price is the cost times the base in seconds ([`crate::content::price`]; for a step,
//! [`LevelState::step_price`]), and the actor's speed decides how long that takes:
//! `cost / 1000 × 1000 / speed × base`, so a normal action takes the player (normal speed) 6 s,
//! and twice the speed takes half of it. Resting ([`Action::Rest`]) passes many turns in one
//! action.
//!
//! Hearing (spec §5 "Hearing", [`crate::noise`]): steps and doors make noise as they happen, and
//! every other actor in earshot on the level hears it at once ([`Actor::heard`], until it next
//! acts). Perception turns what the player heard from unseen sources into perceived sounds, and
//! news among them into messages ("You hear footsteps to the north-east."), which stop a rest.
//!
//! Weather (spec §5 "Weather", [`crate::weather`]) is a pure function of the seed and the time;
//! the world only keeps a dev override. Perception knows it under the open sky, and a change seen
//! there is a message ("It starts to rain."). Rain starting or growing heavier stops a rest; the
//! rest of the news does not.
//!
//! The overworld (spec §3): stairs down on it are locations leading into other levels; `>` on
//! any other cell enters that cell's local area (`local:x,y`, generated on the first visit) on
//! the side the player walked onto the cell from ([`WorldState`]'s last overworld step), and
//! walking off a local area's edge leads back to its overworld cell (the level state's edge exit).
//!
//! Objects (spec §6): piles lie on cells of a level ([`LevelState`]), actors carry inventories.
//! Picking up and dropping are actions with their own costs; stepping onto a pile says what lies
//! there. An actor's weight is its creature's plus what it carries ([`Actor::weight`]), worn and
//! wielded things included. Wearing and wielding (`equip.rs`) take the time of each slot's
//! action; dropping something worn takes it off first.
//!
//! Combat (spec §8 "Combat", `fight.rs`): attacks are rolled against Defence and take hit
//! points; a creature at 0 dies and drops what it carried, and the player's death ends the run
//! ([`World::death`]): every action after it is refused. Hit points come back with game time.
//!
//! Burden (spec §8 "Burden", [`crate::burden`]): what an actor carries against what its
//! Strength, Toughness, and size let it carry slows every action it takes (in the prices,
//! `schedule.rs`), caps its Dexterity in Defence, makes its steps louder, and, overloaded, keeps
//! it from moving. The player is told when their burden changes.
//!
//! Ranged combat (`shoot.rs`): shots and throws fly along a line, attack whoever stands in their
//! way, and come down to be picked up again; burning missiles are fires that light the level
//! until they burn out.
//!
//! Special rooms (spec §7 "Special rooms", [`crate::content::RoomEffectDef`]): a level's special
//! rooms change the rules inside them, through one question, [`World::effects_at`]: the
//! light (and so what the player and creatures see), the noise made there, regeneration, and
//! whether creatures hostile to the player step in. The first time the player is inside one they
//! are told what it is.
//! Weapon skills (spec §8 "Weapon skills", [`crate::weapon_skills`], `fight.rs`): every attack
//! draws on a skill (the weapon's, unarmed, thrown, shield bashing) for its to-hit, damage,
//! threat, and cost, and the player's attacks and shield blocks earn marks toward its next rank
//! ("You feel more skilled with daggers."). A shield bash ([`Action::Bash`]) is a blow with a
//! held shield.
//! Digging (`dig.rs`): walls that break give way to a digging tool, turn after turn like a rest,
//! and to explosions stronger than they are hard; they become rubble.
//! Blood (`bleed.rs`, [`crate::blood`]): big hits and kills stain the floor and the walls around
//! the victim. Presentation state kept with the level; no rule reads it. Char (`sear.rs`,
//! [`crate::scorch`]): fire scorches the floor and walls it burns, the same way.
//! Status effects and healing (spec §8 "Status effects", "Healing", `status.rs`): big hits and
//! criticals make their targets bleed, a giant rat's bite may poison; effects cost hit points
//! and ease as the clock moves, slow healing, and can kill. Hit points come back with game
//! time, twice as fast while the player rests.
//! Hunger (spec §8 "Hunger", `hunger.rs`, `eat.rs`): game time burns the player's nutrition, and
//! the state it leaves them in takes effect (Weak, Fainting, Starving, Bloated); meals pass turns
//! like a rest and feed at their end. Creatures that die leave corpses, which rot away.
//! Temperature and ice (spec §5 "Temperature", "Ice"; `ice.rs`): water freezes in the cold and
//! thaws in the warmth, brought up to the clock as it moves on the player's level (and on
//! arrival), never ticking while nobody is there. Ice holds actors by their weight; one that
//! breaks through plunges into the freezing water and climbs out onto the ice in a few turns.
//!
//! Traps (spec §8 "Traps", `trap.rs`): whoever steps onto a trap sets it off, the player and
//! creatures alike: arrows, spears, fireballs, lightning, pits, alarms, and teleports. Traps are
//! hidden until the player finds one, sets one off, or sees a creature do it; then they are
//! known and remembered.
//! Searching (spec §8 "Searching", `search.rs`): secret doors look like walls and hidden traps
//! like the floor until the player finds them, searching (a check of Intelligence) or in passing
//! (of Willpower, after every turn, for those who notice such things).

// The modules, by subsystem (docs/dev/merging.md, "Hot registries"): each group lists its modules,
// then its test modules, each sorted by name. A new module is one line in its group's list.

/// Gives every item inside `#[cfg(test)]`, so a test module is one line in its group. Shaped and
/// named after the `cfg-if` crate's macro on purpose: rustfmt follows `mod` declarations inside a
/// `cfg_if!` (and inside no other macro), so `cargo fmt` still reaches these files.
macro_rules! cfg_if {
    (if #[cfg(test)] { $($item:item)* }) => {
        $(#[cfg(test)] $item)*
    };
}

// The run, the turn, and time (docs/dev/time.md, saves.md, rendering.md's perception).
mod actor;
mod actors;
mod dev;
mod eyes;
mod glare;
mod hash;
mod hearing;
mod level;
mod perceive;
mod rest;
mod run;
mod schedule;
mod stride;
mod voice;
cfg_if! {
    if #[cfg(test)] {
        mod cost_tests;
        mod hearing_tests;
        mod light_tests;
        mod moon_tests;
        mod numbers_tests;
        mod rest_close_tests;
        mod run_tests;
        mod speed_tests;
        mod stamina_tests;
        mod stride_tests;
        mod sun_tests;
        mod tests;
        mod time_tests;
        mod voice_tests;
        mod weather_tests;
    }
}

// Travel, levels, and the overworld (docs/dev/overworld.md, encounters.md, mapgen.md,
// temperature.md).
mod edge_leave;
mod encounter;
mod encounter_area;
mod endless;
mod enter;
mod footing;
mod ice;
mod layer_boss;
mod pit;
mod quick_leave;
mod reveal;
mod stair_followers;
mod swim;
mod talk_past;
mod travel;
mod wriggle;
cfg_if! {
    if #[cfg(test)] {
        mod beside_tests;
        mod branch_tests;
        mod edge_leave_tests;
        mod encounter_tests;
        mod endless_tests;
        mod escape_tests;
        mod everlasting_tests;
        mod footing_tests;
        mod gate_tests;
        mod ice_tests;
        mod layer_boss_tests;
        mod overworld_tests;
        mod pit_tests;
        mod quick_leave_tests;
        mod reveal_tests;
        mod room_temperature_tests;
        mod stair_followers_tests;
        mod swim_deep_tests;
        mod swim_tests;
        mod way_out_tests;
        mod wriggle_tests;
    }
}

// Combat (docs/dev/combat.md, ranged-combat.md, weapon-skills.md, weapon-abilities.md, blood.md,
// surprise.md).
mod ambush;
mod attacks;
mod bleed;
mod corrode;
mod dummy;
mod fight;
mod free_blow;
mod frenzy;
mod hurl;
mod item_damage;
mod knock_away;
mod lurk;
mod natural;
mod reflect;
mod sear;
mod shoot;
mod slip;
pub(crate) mod surprise;
mod two_shields;
mod weapon_ability;
mod weapon_active;
cfg_if! {
    if #[cfg(test)] {
        mod ambush_tests;
        mod attacks_tests;
        mod balance_probe;
        mod blood_tests;
        mod corrode_tests;
        mod damage_tests;
        mod dummy_tests;
        mod fight_tests;
        mod free_blow_tests;
        mod frenzy_tests;
        mod hurl_tests;
        mod item_damage_tests;
        mod knock_away_tests;
        mod lurk_tests;
        mod protection_tests;
        mod sear_tests;
        mod shoot_tests;
        mod shot_range_tests;
        mod silent_tests;
        mod slip_tests;
        mod stunning_strike_tests;
        mod style_tests;
        mod surprise_tests;
        mod surprise_words_tests;
        mod thrown_exploit_tests;
        mod two_shield_tests;
        mod vitals_tests;
        mod weapon_ability_tests;
        mod weapon_active_tests;
        mod weapon_skill_tests;
    }
}

// Statuses, healing, hunger, and potions (docs/dev/status-effects.md, hunger.md, potions.md,
// burden.md).
mod buff;
mod disease;
mod drain;
mod eat;
mod elixir;
mod flask;
pub(crate) mod freshness;
mod heal;
mod heal_harm;
mod hold;
mod hunger;
mod potion;
mod regen;
mod rot;
mod status;
cfg_if! {
    if #[cfg(test)] {
        mod buff_field_tests;
        mod buff_tests;
        mod burden_tests;
        mod disease_tests;
        mod drain_tests;
        mod eat_tests;
        mod elixir_tests;
        mod flask_content_tests;
        mod flask_fire_tests;
        mod flask_tests;
        mod freshness_tests;
        mod heal_harm_tests;
        mod heal_tests;
        mod hold_tests;
        mod hunger_tests;
        mod potion_gamble_tests;
        mod potion_status_tests;
        mod potion_tests;
        mod potion_utility_tests;
        mod rot_tests;
        mod smoke_tests;
        mod status_tests;
    }
}

// Magic: mana, spells, scrolls, wands (docs/dev/mana.md, spells.md, caster-styles.md, scrolls.md).
mod backfire;
mod cast;
mod cast_style;
mod caster;
mod concentration;
mod corpse_spell;
mod decipher;
mod forget;
mod learn;
mod malison;
mod mana;
mod miscast;
mod read;
mod restore;
mod rod;
mod ruin;
mod scribe;
mod spell;
mod spell_arc;
mod spell_mods;
mod spell_rank;
mod spell_rule;
mod spell_rung;
mod spell_type;
mod spell_words;
mod summon_spell;
mod zap;
mod zone;
cfg_if! {
    if #[cfg(test)] {
        mod backfire_tests;
        mod cast_tests;
        mod caster_style_tests;
        mod caster_tests;
        mod circle_tests;
        mod concentration_tests;
        mod corpse_spell_tests;
        mod decipher_tests;
        mod forget_tests;
        mod identify_spell_tests;
        mod learn_tests;
        mod magic_tests;
        mod malison_tests;
        mod mana_tests;
        mod miscast_stat_tests;
        mod miscast_tests;
        mod necromancy_rank_tests;
        mod read_tests;
        mod rod_tests;
        mod ruin_tests;
        mod scribe_tests;
        mod scroll_label_tests;
        mod spell_arc_tests;
        mod spell_data_tests;
        mod spell_list_tests;
        mod spell_rank_tests;
        mod spell_rule_tests;
        mod spell_type_force_tests;
        mod spell_type_tests;
        mod summon_spell_tests;
        mod transmutation_rank_tests;
        mod zap_tests;
        mod zone_tests;
    }
}

// Creatures (docs/dev/creatures.md, summoning.md, animated.md).
mod ai;
mod animated;
mod aura;
mod avert;
mod blink;
mod breath;
mod court;
mod creature_burst;
mod creature_cast;
mod den;
mod first_sight;
mod gaze;
mod incorporeal;
mod mimic;
mod path;
mod rooted;
mod snatch;
mod spawn;
mod split;
mod statue;
mod summon;
mod unseen;
cfg_if! {
    if #[cfg(test)] {
        mod ai_tests;
        mod animated_blade_tests;
        mod animated_dread_tests;
        mod animated_panoply_tests;
        mod animated_tests;
        mod aura_tests;
        mod beast_den_tests;
        mod bestiary_tests;
        mod breath_tests;
        mod court_tests;
        mod creature_burst_tests;
        mod creature_cast_tests;
        mod creature_flask_tests;
        mod creature_magic_tests;
        mod den_tests;
        mod dwell_tests;
        mod first_sight_tests;
        mod fixture_factions;
        mod gaze_tests;
        mod incorporeal_tests;
        mod mimic_tests;
        mod retribution_tests;
        mod rooted_tests;
        mod spawn_tests;
        mod split_tests;
        mod statue_tests;
        mod summon_tests;
        mod tag_tests;
        mod unique_tests;
        mod unseen_tests;
        mod variant_tests;
    }
}

// Allies (docs/dev/allies.md, companion.md, familiar.md).
mod ally;
mod beast_friend;
mod bond;
mod charm;
mod collar;
mod companion;
mod familiar;
mod pack;
mod sic;
mod tame;
cfg_if! {
    if #[cfg(test)] {
        mod ally_tests;
        mod bond_tests;
        mod charm_tests;
        mod collar_tests;
        mod pack_tests;
        mod sic_tests;
        mod tame_tests;
    }
}

// The character: creation, progression, stats (docs/dev/character-creation.md, background.md,
// progression.md, experience.md, stats.md, skills.md, alignment.md, resources.md).
mod alignment;
mod background;
mod creation;
mod element_gifts;
mod experience;
mod flavour;
mod gifts;
mod heirloom;
mod lucky;
mod profane;
mod racial;
mod resource;
mod second_wind;
mod sign_picks;
mod skill_roll;
mod stars;
mod stat_deeds;
mod stat_training;
mod talent;
mod train;
cfg_if! {
    if #[cfg(test)] {
        mod alignment_tests;
        mod body_talent_tests;
        mod character_tests;
        mod creation_tests;
        mod damage_talent_tests;
        mod element_gift_tests;
        mod experience_tests;
        mod flavour_tests;
        mod gifts_tests;
        mod heirloom_tests;
        mod land_talent_tests;
        mod lucky_tests;
        mod mana_talent_tests;
        mod race_tests;
        mod racial_tests;
        mod resource_talent_tests;
        mod resource_tests;
        mod sign_pick_tests;
        mod skill_roll_tests;
        mod skills_tests;
        mod spell_talent_tests;
        mod stars_roc_tests;
        mod stars_tests;
        mod stat_deeds_tests;
        mod stat_paths_tests;
        mod stat_training_tests;
        mod stats_tests;
        mod talent_arms_tests;
        mod talent_tests;
        mod talent_wits_tests;
        mod waste_not_tests;
    }
}

// Classes and their abilities (docs/dev/progression.md, openings.md, prayer.md).
mod ascetic;
mod calling;
mod charge;
mod class;
mod class_ability;
mod consecrate;
mod dark_face;
mod focus;
mod invest;
mod jack;
mod mark;
mod merchants_eye;
mod openings;
mod orison;
mod rage;
mod sanctuary;
mod smite;
mod song;
mod spirit_weapon;
mod stipend;
mod thief;
mod volley;
mod war_cry;
mod witch;
cfg_if! {
    if #[cfg(test)] {
        mod ascetic_tests;
        mod calling_tests;
        mod charge_tests;
        mod class_active_tests;
        mod class_rogue_tests;
        mod class_tests;
        mod dark_face_tests;
        mod focus_tests;
        mod invest_tests;
        mod ki_tests;
        mod lullaby_tests;
        mod mark_tests;
        mod merchants_eye_tests;
        mod openings_tests;
        mod orison_tests;
        mod sanctuary_tests;
        mod smite_mode_tests;
        mod smite_tests;
        mod smite_wrath_tests;
        mod song_tests;
        mod spirit_weapon_tests;
        mod stipend_tests;
        mod war_cry_tests;
        mod witch_tests;
    }
}

// Skill uses (docs/dev/skills.md, searching.md, stealing.md, hearing.md).
mod appraise;
mod disarm;
mod feint;
mod forage;
mod gather;
mod intimidate;
mod knowledge;
mod listen;
mod lockpick;
mod search;
mod sidestep;
mod steal;
mod wedge;
cfg_if! {
    if #[cfg(test)] {
        mod appraise_tests;
        mod disarm_tests;
        mod feint_tests;
        mod forage_tests;
        mod gather_tests;
        mod intimidate_tests;
        mod knowledge_tests;
        mod lockpick_tests;
        mod search_tests;
        mod sidestep_tests;
        mod steal_tests;
        mod theft_tests;
        mod wedge_tests;
    }
}

// Objects, equipment, and magic items (docs/dev/objects.md, equipment.md, magic-items.md).
mod carry;
mod equip;
pub(crate) mod gear_condition;
mod lamp;
mod loot;
mod material_drops;
mod siphon;
mod stock;
cfg_if! {
    if #[cfg(test)] {
        mod affix_draught_tests;
        mod affix_spell_tests;
        mod affix_stamina_tests;
        mod affix_utility_tests;
        mod equip_tests;
        mod gear_condition_tests;
        mod gear_tests;
        mod gem_tests;
        mod lamp_tests;
        mod material_draw_tests;
        mod material_drops_tests;
        mod object_tests;
        mod parts_tests;
        mod siphon_tests;
    }
}

// Crafting, enchanting, and digging (docs/dev/crafting.md, enchanting.md, digging.md).
mod craft;
mod craft_make;
mod crystal;
mod cut;
mod dig;
mod enchant;
mod field_forge;
mod fletch;
mod make_do;
mod mastery;
mod mine;
pub(crate) mod patron;
mod rig;
mod salvage;
mod seal;
mod set;
mod temper;
mod trade;
mod work;
mod work_on;
cfg_if! {
    if #[cfg(test)] {
        mod brew_tests;
        mod craft_armour_tests;
        mod craft_bow_tests;
        mod craft_hand_tests;
        mod craft_hide_tests;
        mod craft_make_tests;
        mod craft_tests;
        mod craftsperson_tests;
        mod cut_tests;
        mod dig_tests;
        mod enchant_tests;
        mod field_forge_tests;
        mod fletch_tests;
        mod make_do_tests;
        mod makers_mark_tests;
        mod mastery_tests;
        mod mine_tests;
        mod patron_tests;
        mod rig_tests;
        mod salvage_tests;
        mod set_tests;
        mod temper_tests;
        mod trade_tests;
        mod whittle_tests;
    }
}

// Doors, chests, traps, and special rooms (docs/dev/doors.md, chests.md, traps.md,
// special-rooms.md).
mod boons;
mod chest;
mod collapse;
mod curse;
mod door;
mod drink;
mod feature;
mod graves;
mod graveyard;
mod haunt;
mod hoard;
mod knock;
pub(crate) mod mirrors;
mod outcome;
mod pray;
mod rooms;
mod shrine;
mod trap;
mod treasury;
mod warp;
cfg_if! {
    if #[cfg(test)] {
        mod chest_hall_tests;
        mod chest_tests;
        mod chest_trap_tests;
        mod collapse_tests;
        mod curse_tests;
        mod door_tests;
        mod door_trap_tests;
        mod feature_tests;
        mod graveyard_tests;
        mod haunt_tests;
        mod heavy_hall_tests;
        mod hoard_tests;
        mod mirrors_tests;
        mod outcome_tests;
        mod room_tests;
        mod shrine_tests;
        mod spring_tests;
        mod trap_rung_tests;
        mod trap_tests;
        mod treasury_tests;
        mod unstable_room_tests;
    }
}

// People: talk, quests, shops, guilds (docs/dev/npcs.md, quests.md, shops.md, guild.md).
mod escort;
mod escort_waits;
mod goodwill;
mod guild;
mod lure;
mod quest;
mod quest_roll;
mod rumour;
mod service;
mod shop;
mod talk;
mod wares;
cfg_if! {
    if #[cfg(test)] {
        mod escort_tests;
        mod goodwill_tests;
        mod guild_tests;
        mod guild_wanted_tests;
        mod identify_service_tests;
        mod outlaw_tests;
        mod quest_roll_tests;
        mod quest_tests;
        mod rumour_tests;
        mod service_haggle_tests;
        mod service_tests;
        mod shop_tests;
        mod talk_tests;
        mod train_service_tests;
        mod wares_tests;
    }
}

// The campaigns' places and people, end to end.
cfg_if! {
    if #[cfg(test)] {
        mod ascmearc_encounter_tests;
        mod ascmearc_tests;
        mod ceaptun_quest_tests;
        mod ceaptun_tests;
        mod crystal_guardian_tests;
        mod deep_gnome_tests;
        mod druid_quest_tests;
        mod gnome_letter_tests;
        mod hellegeat_tests;
        mod holtwic_tests;
        mod lictun_tests;
        mod pit_warden_tests;
        mod scriptorium_tests;
        mod strange_spirit_tests;
        mod wynflaed_quest_tests;
    }
}

pub use actor::{Actor, ActorId, ActorKind, AttackSkills, Mind, Patrol};
pub use actors::Actors;
pub use alignment::Deed;
pub use ally::{Ally, AllySource, MAX_DEV_ALLIES, Order};
pub use attacks::{AttackSource, OwnAttack};
pub use calling::CallingStatus;
pub use cast::Reach;
pub use collar::fits_neck;
pub(crate) use companion::smelt_within;
pub use companion::{CompanionData, draw as companion_draw, name_for as companion_name};
pub(crate) use craft::recipe_rows;
pub use craft::{RecipeLack, RecipeRow};
pub use craft_make::{CraftOption, CraftPlan, CraftStep, CraftStepKind, Grade, grade};
pub(crate) use craft_make::{craft_plan, craft_steps};
pub use creation::{chosen_with_talents, creation_bonus, creation_count, parse_talents, pick_text};
pub use curse::{Rising, rise_candidates, rises_as};
pub use cut::{CutBand, cut_band, cut_outcome, cuttable};
pub use decipher::undeciphered;
pub use dig::{MAX_DIG, WORK_SCALE, dig_refusal, dig_work, digs_to_break};
pub use disarm::SPRINGS_SHORT_BY;
pub(crate) use enchant::craft_steps_given;
pub use enchant::{EnchantBand, enchant_band};
pub(crate) use encounter::view as encounter_view;
pub use encounter::{
    Choice as EncounterChoice, EncounterView, Forced as EncounterForced, KNOWN_LINE,
    Outcome as EncounterOutcome, PENDING_REFUSED, Party, Pending,
};
pub use escort::{escort_slot, fits_escort};
pub use experience::{Discovered, Progress};
pub use familiar::{draw as familiar_draw, name_for as familiar_name};
pub(crate) use field_forge::open_ground;
pub use fight::{Cause, Death};
pub(crate) use forage::forage_ground;
pub use free_blow::FreeBlows;
pub(crate) use gather::{fungus_ways, unskilled};
pub use gifts::{STARE_RANGE, Streak};
pub use graveyard::Fallen;
pub use guild::GuildRank;
pub(crate) use hash::fnv1a;
pub(crate) use heal::allies_beside;
pub(crate) use item_damage::protected_in;
pub use learn::{Odds, UNTRAINED, study_odds, study_turns};
pub use level::{Fire, LevelState};
pub use loot::Identified;
pub use make_do::mendable;
pub(crate) use make_do::mends_any;
pub use mirrors::Image;
pub use miscast::{NEAR as MISCAST_NEAR, entry_for as miscast_entry};
pub use orison::PrayerState;
pub use pit::Pit;
pub use quest::{QuestRecord, QuestView, Quests, Stage, asks as quest_asks, wants as quest_wants};
use rest::Ended;
pub(crate) use rig::rigged_rows;
pub use rig::{RigRow, takes_words};
pub use salvage::{chipped, salvage_percent, salvageable};
pub use sanctuary::Hallow;
pub use scribe::scribe_turns;
pub use seal::MOST_SEALS;
pub use set::{SetBand, set_band, settable};
pub use shop::{Shop, ShopStatus, count_of, favour, part_of, unpaid_parts};
pub use sign_picks::BREATH_RANGE;
pub use siphon::SIPHON_RANGE;
pub use skill_roll::{RollBonus, RollCount, SkillRoll, roll_count, skill_roll};
pub use spell::cone_cells;
pub use spell_rung::CastBy;
pub use spell_words::{SpellRoll, reach_words, reroll_words, roll_words};
pub use spirit_weapon::{gift as spirit_gift, pierces_all, shapes as weapon_shapes};
pub(crate) use stars::{RISEN, RISEN_TWICE};
pub use statue::Statue;
pub(crate) use swim::shore_refusal;
pub use talent::Due;
pub use talk::{Conversation, Talk, TopicView};
pub(crate) use talk_past::offered as encounter_choices;
pub use temper::{temperable, whittleable, workable, workable_wood};
use travel::{enter_room, instantiate_level, level_template};
pub(crate) use unseen::{Concealed, DRAUGHT, unseen_by_player};
pub use voice::VOICE_RANGE;
pub use zone::Zone;

use crate::action::{Action, Event, Outcome, Rest};
use crate::alignment::Score;
use crate::burden::{self, Burden, Load};
use crate::character::{Character, CharacterSpec};
use crate::content::{
    ActionKind, CombatStance, Content, Here, ObjectId, RaceDef, StairsDir, Stance, StatusId,
    TalentActive, TalentEffect,
};
use crate::geom::{Dir, Pos};
use crate::hunger::Satiation;
use crate::light::{LightCache, Sky};
use crate::messages::MessageLog;
use crate::noise::Recent;
use crate::object::Stack;
use crate::perception::{Perception, PerceptionView};
use crate::rng::Rng;
use crate::scroll_label::{Labels, Lore};
use crate::skills::{Skills, Use};
use crate::time::{Duration, GameTime};
use crate::weapon_skills::WeaponSkills;
use crate::weather;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Why a world could not be created or restored.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WorldError {
    #[error("unknown scenario `{0}`")]
    UnknownScenario(String),
    #[error("unknown level `{0}`")]
    UnknownLevel(String),
    #[error("level `{0}` has no player start")]
    NoPlayerStart(String),
    #[error("cannot create the character: {0}")]
    Character(String),
    #[error("cannot generate level `{0}`: {1}")]
    Generation(String, String),
    #[error("saved state is inconsistent: {0}")]
    InconsistentState(String),
}

/// All persistent simulation state. This is what saves contain.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldState {
    pub(crate) scenario: String,
    /// The campaign the run is of (spec §3): its scenario's. Local areas are its overworld's,
    /// and saves carry its content fingerprint ([`Content::fingerprint_for`]). A save without
    /// one takes its scenario's ([`World::from_state`]).
    #[serde(default)]
    pub(crate) campaign: String,
    pub(crate) seed: u64,
    /// The gameplay random stream (spec §8), continued across saves. Map generation never draws
    /// from it: each generated level has its own stream derived from the seed and its id.
    pub(crate) rng: Rng,
    pub(crate) turn: u64,
    /// The game clock.
    pub(crate) time: GameTime,
    pub(crate) current_level: String,
    pub(crate) levels: BTreeMap<String, LevelState>,
    pub(crate) actors: Actors,
    pub(crate) player: ActorId,
    pub(crate) next_actor_id: u32,
    pub(crate) messages: MessageLog,
    /// The player's vision profile (an id in `data/vision.toml`).
    pub(crate) vision: String,
    /// Whether `vision` is the dev `vision:` override, which a unique item's Vision power never
    /// betters (`world/eyes.rs`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(crate) vision_dev: bool,
    /// The player's character (spec §8 "Character creation"): race, gender, and class, the stat
    /// the gender raised, birth date, and birth sign; none when the scenario gives the player's
    /// eyes and stats itself.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) character: Option<Character>,
    /// Whether the run started with the character's starting kit (a character the player chose;
    /// spec §8 "Character creation"); replays record it. Left out of saves when false.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(crate) kit: bool,
    /// The step that brought the player onto their overworld cell, if they walked there: `>`
    /// then arrives on the side of the local area they came from. Cleared on any level change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) overworld_step: Option<Dir>,
    /// The closed door the player listens at during the turn being taken, and how much fainter
    /// a sound they hear through it (`world/listen.rs`). Set by the action, cleared as it ends:
    /// never in a save.
    #[serde(skip)]
    pub(crate) listening: Option<crate::noise::Listening>,
    /// The overworld encounters' random stream (spec §7 "Overworld encounters": the rate roll,
    /// the entry, the party's dice), made on the first roll (`world/encounter.rs`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) encounters: Option<Rng>,
    /// The encounter the player must answer before anything else (`world/encounter.rs`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) encounter: Option<encounter::Pending>,
    /// The endless branches' floors made and depths reached (`world/endless.rs`).
    #[serde(default, skip_serializing_if = "endless::EndlessFloors::is_empty")]
    pub(crate) endless: endless::EndlessFloors,
    /// The Pits' records by their integers (`world/pit.rs`; left out while there are none).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) pits: BTreeMap<u32, pit::Pit>,
    /// The parties encounters made, by their areas (`world/encounter.rs`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) parties: Vec<encounter::Party>,
    /// What traders offer, by trader, once asked (`world/wares.rs`): the item and its price.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) wares: BTreeMap<ActorId, wares::Wares>,
    /// The unique items traders' offers have made in the run (`world/wares.rs`): each once.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub(crate) uniques_offered: BTreeSet<String>,
    /// What an aristocrat's family has put by for them, until their servant hands it over
    /// (`world/stipend.rs`).
    #[serde(default, skip_serializing_if = "stipend::Stipend::is_empty")]
    pub(crate) stipend: stipend::Stipend,
    /// A merchant's stake with their business partner (`world/invest.rs`).
    #[serde(default, skip_serializing_if = "invest::Fund::is_empty")]
    pub(crate) fund: invest::Fund,
    /// A craftsperson's commissions from their patron (`world/patron.rs`).
    #[serde(default, skip_serializing_if = "patron::Patron::is_empty")]
    pub(crate) patron: patron::Patron,
    /// Sounds the player perceived lately: those like them are no news ([`noise::news`]).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) recent_sounds: Vec<Recent>,
    /// The training random stream (spec §8 "Skills"): the chances of training marks, apart from
    /// the gameplay stream so training never changes a fight's rolls.
    pub(crate) training: Rng,
    /// The levels random stream (spec §8 "Talents"): what level-ups roll (skill rolls, stat
    /// checks), apart from the gameplay stream so a level moves no fight. Made on the first
    /// draw ([`Rng::levels`]), so a run that never drew from it hashes as before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) levels_rng: Option<Rng>,
    /// The ruin random stream (spec §8 "Spells": "Ruin"): which items lying in an area spell
    /// are lost. Made on the first draw ([`Rng::ruin`]), so a run that never drew from it hashes
    /// as before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) ruin_rng: Option<Rng>,
    /// When the daily stat check comes next (spec §8 "Stats": 06:00 game time;
    /// `world/stat_training.rs`). `None` while the content has no training rules (so content
    /// without them hashes as before).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) next_stat_check: Option<GameTime>,
    /// Dev: the weather set by hand (`weather:rain`), until it runs out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) weather_override: Option<weather::Override>,
    /// How the run ended, once the player has died: nothing more happens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) death: Option<Death>,
    /// The player's experience total (spec §8 "Experience"); their level follows from it.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub(crate) experience: u32,
    /// What the player has discovered for experience (spec §8 "Experience").
    #[serde(default, skip_serializing_if = "Progress::is_empty")]
    pub(crate) progress: Progress,
    /// The player's quests and the kills they count (spec §8 "Quests").
    #[serde(default, skip_serializing_if = "Quests::is_empty")]
    pub(crate) quests: Quests,
    /// The hidden locations revealed so far, by id (spec §7 "Hidden locations"): applied to the
    /// overworld as it is made, if it was not yet when they were revealed.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub(crate) revealed: BTreeSet<String>,
    /// The scroll types the player has identified (spec §8 "Scrolls"; [`crate::scroll_label`]):
    /// they show by their names, the rest by their labels.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub(crate) known_scrolls: BTreeSet<ObjectId>,
    /// The petrified dead of past runs, the most recent first, as the app gave them when the run
    /// began (`world/graveyard.rs`): statues' names, decoration out of the state hash.
    #[serde(default, skip_serializing_if = "hash::unhashed_vec")]
    pub(crate) graveyard: Vec<Fallen>,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

impl WorldState {
    pub(crate) fn player(&self) -> &Actor {
        &self.actors[&self.player]
    }

    /// The sky over the current level now, under the weather of the moment (the dev override
    /// while it lasts): the sun, the moons, and their light through the clouds; `None` on a level
    /// not under the open sky. The player's perception and creature sight both see this sky.
    pub(crate) fn sky(&self, content: &Content) -> Option<Sky> {
        self.current().kind.is_sky_exposed().then(|| {
            let over = self.weather_override.as_ref();
            let now = weather::conditions(self.seed, self.time, content.climate(), over);
            Sky::at(self.time, content.moons(), now)
        })
    }

    pub(crate) fn current(&self) -> &LevelState {
        &self.levels[&self.current_level]
    }

    /// The race of `actor`: the player's when the run has a character; `None` for anyone else,
    /// who goes by its creature type.
    pub(crate) fn race_of<'c>(&self, content: &'c Content, actor: &Actor) -> Option<&'c RaceDef> {
        (self.character.as_ref())
            .filter(|_| actor.kind == ActorKind::Player)
            .and_then(|c| c.spec.race(content).ok())
    }

    /// What `actor` carries against what it can (spec §8 "Burden"): the player by their race's
    /// size (and whether it ignores Burdened) when the run has a character, anyone else by its
    /// creature type's size; inside a special room that scales the limit (a heavy hall), by its
    /// percent. The one place the rules ask for a load.
    pub(crate) fn load(&self, content: &Content, actor: &Actor) -> Load {
        self.load_at(content, actor, actor.pos)
    }

    /// [`WorldState::load`] as it would be were `actor` at `pos` on its level (a step into a
    /// heavy hall).
    pub(crate) fn load_at(&self, content: &Content, actor: &Actor, pos: Pos) -> Load {
        let room =
            (self.levels.get(&actor.level)).map_or(100, |l| l.effects_at(content, pos).burden());
        burden::load(actor, self.race_of(content, actor), content, room)
    }

    /// The size of `actor`, an index into [`Content::sizes`]: the player's race's when the run
    /// has a character, else its creature type's.
    pub(crate) fn size_of(&self, content: &Content, actor: &Actor) -> usize {
        let race = self.race_of(content, actor);
        race.map_or(content.creature(actor.creature).size, |r| r.size)
    }

    fn current_mut(&mut self) -> &mut LevelState {
        self.levels
            .get_mut(&self.current_level)
            .expect("current level is instantiated")
    }

    /// Who stands on `p` of level `level`, if anyone: never an ally in the retinue, which stands
    /// in no one's way (`ally.rs`).
    fn actor_at(&self, level: &str, p: Pos) -> Option<&Actor> {
        (self.actors.on_level(level)).find(|a| a.pos == p && !a.in_retinue())
    }

    fn check(&self) -> Result<(), WorldError> {
        let bad = |m: String| Err(WorldError::InconsistentState(m));
        if !self.levels.contains_key(&self.current_level) {
            return bad(format!("current level `{}` missing", self.current_level));
        }
        // A dead player stopped where they fell, whether or not they were ready to act.
        match self.actors.get(&self.player) {
            Some(p)
                if p.kind == ActorKind::Player
                    && p.level == self.current_level
                    && (p.energy >= 0 || self.death.is_some()) => {}
            _ => return bad("player actor missing, misplaced, or not ready to act".into()),
        }
        for actor in self.actors.values() {
            let Some(level) = self.levels.get(&actor.level) else {
                return bad(format!("actor {} is on missing level", actor.id));
            };
            if !level.terrain.in_bounds(actor.pos) || actor.id.0 >= self.next_actor_id {
                return bad(format!(
                    "actor {} is out of bounds or has a future id",
                    actor.id
                ));
            }
        }
        Ok(())
    }

    /// The player's inventory.
    fn inventory_mut(&mut self) -> &mut Vec<Stack> {
        let player = self.player;
        &mut self
            .actors
            .get_mut(&player)
            .expect("the player actor exists")
            .inventory
    }
}

/// A running game: state plus the content it refers to and the current perception.
#[derive(Clone, Debug)]
pub struct World {
    content: Arc<Content>,
    /// Boxed: the state (with its random stream) is large, and worlds are moved around whole.
    state: Box<WorldState>,
    /// Boxed: the snapshot is large and replaced wholesale every turn.
    perception: Box<Perception>,
    /// Cached light masks; derived, never saved.
    lights: LightCache,
    /// The unidentified scrolls' labels of the run; derived from the seed, never saved.
    labels: Labels,
    /// Messages logged since this world was created or loaded (not saved): resting stops when it
    /// changes.
    said: u64,
    /// Creatures that died during the current action, as they were (not saved): what they did
    /// that turn is said when it is perceived.
    fallen: Vec<Actor>,
    /// What the player's kills earned them during the current action (not saved): said after
    /// the attack's own messages.
    news: Vec<String>,
    /// Notes the current action earned (not saved): said after its own messages and the news,
    /// as notes that do not stop a rest (an alignment band crossed, `alignment.rs`).
    notes: Vec<String>,
    /// While the player strikes a free blow in another's turn (not saved): what is said
    /// meanwhile, to join the news (`free_blow.rs`).
    held: Option<Vec<String>>,
    /// The sleeper the player's attack now under way found asleep (not saved; `surprise.rs`):
    /// its waking is said after the attack's own line ("You hit the sleeping goblin.").
    sleeper_struck: Option<ActorId>,
    /// Whether the missile that just reached someone was caught (not saved; Diamond soul,
    /// `ascetic.rs`): its flight ends there, and it falls.
    caught: bool,
    /// The creature a spell with `crystal` is striking now (not saved; Soulglass, `crystal.rs`):
    /// if it dies of the blow it leaves crystals, not a corpse.
    glassing: Option<ActorId>,
    /// Traps sprung since this world was created or loaded (not saved): a rest perceives every
    /// turn in which one went off (a teleport moves someone farther than a step).
    sprung: u64,
    /// Who rigged the trap going off now, if the player did (not saved; `trap.rs`, `rig.rs`):
    /// its blows are theirs.
    rigger: Option<ActorId>,
    /// Creatures that came to the player's level since this world was created or loaded (not
    /// saved): a rest perceives every turn in which one came (`spawn.rs`).
    spawned: u64,
    /// Terrain changes made on any level since this world was created or loaded (not saved;
    /// [`World::set_terrain`]): a rest keeps the player's line of sight while it is unchanged.
    reshaped: u64,
    /// Hidden features the player found since this world was created or loaded (not saved):
    /// searching for a while stops when one is found (`search.rs`).
    found: u64,
    /// Lines muttered during the current turn (not saved): said when perception runs, if the
    /// player heard them.
    mutters: Vec<talk::Mutter>,
    /// Whether the player is resting ([`World::rest`]; not saved: a rest is one action): they
    /// heal faster.
    resting: bool,
    /// Whether the player is waiting, which is searching (#1326; not saved): sounds heard then
    /// train Listen.
    waiting: bool,
    /// Whether the player is running ([`World::run`]; not saved: a run is one action): a run
    /// seen through to its end trains no Willpower (`keep_at`).
    running: bool,
    /// The game time of the last perception (not saved: a loaded world has just perceived its
    /// moment): the birthday's note looks for the dawn between it and now.
    perceived_at: GameTime,
    /// Where the player stood at the last perception (`feature.rs`; not saved): arriving
    /// elsewhere names a feature of the ground there.
    stood_on: feature::StoodOn,
    /// Whose action is under way (not saved): the player's outside the turn's schedule, a
    /// creature's during its turn, nobody's while time passes (`schedule.rs`).
    acting: Option<ActorId>,
    /// Statuses with `per_action` given to an actor during its own action (not saved): that
    /// action does not ease them (`weapon_ability.rs`).
    fresh: Vec<(ActorId, StatusId)>,
    /// Spell reflection's counts a turn and the reflector chosen (not saved; `reflect.rs`).
    reflection: reflect::Reflections,
    /// The player's weapon active under way (not saved): what its attack does beyond a plain
    /// one, read where that is resolved (`weapon_active.rs`).
    technique: weapon_active::Technique,
    /// What an overworld encounter has under way within the action (not saved:
    /// `encounter.rs`).
    encounter: encounter::Underway,
}

/// How much time an action took.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Spent {
    /// Refused or failed: nothing changed but the message log.
    Nothing,
    /// Changed state without game time passing; perception must be recomputed.
    NoTime,
    /// A turn: the player's action costs this energy price; the others act until the player may
    /// act again (the clock advances), then perception is recomputed.
    Turn(i64),
    /// A rest until this moment (see [`World::rest`]).
    Rest(GameTime, Rest),
    /// Running in this direction (see [`World::run`]).
    Run(Dir),
    /// A meal (see [`World::dine`]).
    Meal(eat::Dish),
    /// Digging into the wall at this cell (see [`World::dig`]).
    Dig(Pos),
    /// Searching for up to this many turns (see [`World::search`]).
    Search(u8),
    /// Studying a spell from a scroll (see [`World::study`]).
    Study(learn::Lesson),
    /// Writing a scroll of a spell (see [`World::scribe`]).
    Scribe(scribe::Writing),
    /// Calling a familiar for this many turns (see [`World::calling`]).
    Calling(u32),
    /// Praying to consecrate a place until this moment (see [`World::consecrate`]).
    Consecrate(GameTime),
    /// Kneeling to hallow the ground until this moment (see [`World::sanctuary`]).
    Sanctuary(GameTime),
    /// Praying at a forgotten altar (see [`World::pray`]).
    Pray(pray::Prayer),
    /// Drinking at a healing spring (see [`World::drink`]).
    Drink(drink::Draught),
    /// Foraging in the wild for this many seconds (see [`World::forage`]).
    Forage(u32),
    /// Gathering herbs from the glowing fungus for this many seconds (see [`World::gather`]).
    Gather(u32),
    /// Long work: deciphering, a craft, salvage, a rig (see [`World::work`]).
    Work(work::Work),
    /// Taming a beast beside the player (see [`World::tame`]).
    Tame(tame::Taming),
    /// Dev: the clock jumps ahead; nobody acts, then perception is recomputed.
    Skip(Duration),
}

impl World {
    /// Starts a new run of `scenario` with the scenario's own character. `seed` defaults to the
    /// scenario's seed.
    pub fn new(
        content: Arc<Content>,
        scenario: &str,
        seed: Option<u64>,
    ) -> Result<Self, WorldError> {
        Self::with_character(content, scenario, seed, None)
    }

    /// Starts a new run of `scenario` as `character` (spec §8 "Character creation"); without
    /// one, as the scenario's character, or if it names none, with the eyes and stats it gives.
    /// `seed` defaults to the scenario's seed. The character's name is the run's default
    /// ([`character::default_name`](crate::character::default_name)).
    pub fn with_character(
        content: Arc<Content>,
        scenario: &str,
        seed: Option<u64>,
        character: Option<&CharacterSpec>,
    ) -> Result<Self, WorldError> {
        Self::with_character_named(content, scenario, seed, character, None)
    }

    /// [`World::with_character`], the character named `name` (checked by
    /// [`character::check_name`](crate::character::check_name)); `None` gives the run's default
    /// name. A run without a character has no name either.
    pub fn with_character_named(
        content: Arc<Content>,
        scenario: &str,
        seed: Option<u64>,
        character: Option<&CharacterSpec>,
        name: Option<&str>,
    ) -> Result<Self, WorldError> {
        Self::start_run(content, scenario, seed, character, name, false)
    }

    /// [`World::with_character_named`]; with `kit`, the character starts with their starting
    /// kit and the scenario's `carry` on top (spec §8 "Character creation"), as a run whose
    /// character the player chose does (New run, `--character`, `--name`). Without it, with the
    /// scenario's pack alone, as a dev launch on the scenario's own character does (tests,
    /// golden walks, `--scenario` alone). A run without a character has no kit. A run with the
    /// kit also takes the character's creation picks (`world/creation.rs`): the talents `character`
    /// names, else the Random picks, recorded in the character.
    pub fn start_run(
        content: Arc<Content>,
        scenario: &str,
        seed: Option<u64>,
        character: Option<&CharacterSpec>,
        name: Option<&str>,
        kit: bool,
    ) -> Result<Self, WorldError> {
        // The scenario's own character's named picks (`character = { .., talents = [..] }`): for
        // a run as that character (none given, or the same race, gender, and class naming no
        // picks of its own, as a replay records it).
        let own = (content.scenario(scenario))
            .filter(|s| match (character, &s.character) {
                (None, _) => true,
                (Some(c), Some(theirs)) => c.talents.is_empty() && *c == *theirs,
                (Some(_), None) => false,
            })
            .map(|s| s.character_talents.clone())
            .unwrap_or_default();
        let mut world = Self::begin(content, scenario, seed, character, name, kit)?;
        if !world.started_with_kit() && !own.is_empty() {
            world.take_scenario_picks(&own)?;
            *world.perception =
                Perception::compute(&world.state, &world.content, &mut world.lights);
        }
        if world.started_with_kit() {
            world.take_creation(&own)?;
            world.tell_background();
            world.player_mut().mana = world.state.player().max_mana(&world.content);
            *world.perception =
                Perception::compute(&world.state, &world.content, &mut world.lights);
            let current = world.state.current_level.clone();
            if let Some(level) = world.state.levels.get_mut(&current) {
                world.perception.remember(level);
            }
        }
        Ok(world)
    }

    /// [`World::start_run`] with the creation picks still due (`world/creation.rs`).
    fn begin(
        content: Arc<Content>,
        scenario: &str,
        seed: Option<u64>,
        character: Option<&CharacterSpec>,
        name: Option<&str>,
        kit: bool,
    ) -> Result<Self, WorldError> {
        let def = content
            .scenario(scenario)
            .ok_or_else(|| WorldError::UnknownScenario(scenario.to_string()))?;
        let seed = seed.unwrap_or(def.seed);
        // The player's hit points beyond their Toughness (and mana beyond their mental stats):
        // what their race and class give, or without a character what the scenario gives.
        // Without a character the player is trained in nothing, has no weapon skills, and
        // starts Neutral.
        let chosen = character.or(def.character.as_ref());
        let start = match chosen {
            Some(spec) => {
                let mut s =
                    (spec.start(&content, seed, def.start)).map_err(WorldError::Character)?;
                s.character.name = match name {
                    Some(name) => crate::character::check_name(name)
                        .map_err(WorldError::Character)?
                        .to_string(),
                    None => crate::character::default_name(&content, def, seed, spec),
                };
                Some(s)
            }
            None => None,
        };
        // The character's kit, if they start with it, then the scenario's pack on top.
        let kit = start.as_ref().filter(|_| kit).map(|s| &s.kit);
        let mut pack = crate::kit::starting_pack(&content, def, kit);
        // A wand of the scenario's own pack draws its rung (a kit's has one already).
        let mut rng = crate::rng::Rng::objects(seed, "pack");
        for stack in &mut pack.items {
            stack.draw_wand_rung(&content, &mut rng);
        }
        let with_kit = kit.is_some();
        // Everything a character sets out with is known by type, the kit's and the scenario's
        // own pack alike (spec §8 "Scrolls", "Potions"). A run without a character (a test's
        // hall) knows nothing: its pack is as found.
        let known_scrolls: BTreeSet<ObjectId> = match start {
            Some(_) => crate::kit::known_types(&content, &pack.items).collect(),
            None => BTreeSet::new(),
        };
        let (stats, vision, (hp_base, mana_base), skills, character, weapon_skills, alignment) =
            match start {
                Some(s) => {
                    let character = Some(s.character);
                    (
                        s.stats,
                        s.vision,
                        (s.hp, s.mana),
                        s.skills,
                        character,
                        s.weapon_skills,
                        s.alignment,
                    )
                }
                None => (
                    def.player_stats,
                    def.vision().to_string(),
                    (i32::from(def.hp), i32::from(def.mana)),
                    Skills::default(),
                    None,
                    WeaponSkills::default(),
                    Score::ZERO,
                ),
            };
        let campaign = def.campaign.clone().unwrap_or_default();
        // A run that starts on a Pit's floor (a dev scenario) climbs out to its campaign's start.
        let pit_from = pit::start_from(&content, &campaign, &def.start_level);
        let start = level_template(
            &content,
            (
                &campaign,
                endless::floor_seed(&content, seed, &def.start_level, 0),
            ),
            &def.start_level,
            (pit_from.as_deref(), &Default::default()),
        )?;
        let player_pos = start
            .player_start
            .ok_or_else(|| WorldError::NoPlayerStart(start.id.clone()))?;
        let player_creature = content
            .ids()
            .player
            .expect("content validation guarantees a player creature");

        let mut state = WorldState {
            scenario: def.id.clone(),
            campaign,
            seed,
            rng: Rng::gameplay(seed),
            training: Rng::training(seed),
            levels_rng: None,
            ruin_rng: None,
            next_stat_check: None,
            turn: 0,
            time: def.start,
            current_level: start.id.clone(),
            levels: BTreeMap::new(),
            actors: Actors::default(),
            player: ActorId(0),
            next_actor_id: 1,
            messages: MessageLog::default(),
            vision,
            vision_dev: false,
            character,
            kit: with_kit,
            overworld_step: None,
            listening: None,
            encounters: None,
            encounter: None,
            endless: Default::default(),
            pits: BTreeMap::new(),
            parties: Vec::new(),
            wares: BTreeMap::new(),
            uniques_offered: BTreeSet::new(),
            stipend: stipend::Stipend::default(),
            fund: invest::Fund::default(),
            patron: patron::Patron::default(),
            recent_sounds: Vec::new(),
            weather_override: None,
            death: None,
            experience: 0,
            progress: Progress::default(),
            quests: Quests::default(),
            revealed: BTreeSet::new(),
            known_scrolls,
            graveyard: Vec::new(),
        };
        let mut player = Actor {
            id: ActorId(0),
            kind: ActorKind::Player,
            creature: player_creature,
            level: start.id.clone(),
            pos: player_pos,
            patrol: None,
            energy: 0,
            heard: Vec::new(),
            mind: Mind::default(),
            inventory: pack.items,
            equipment: pack.equipment,
            stats,
            hp: 0,
            hp_base,
            regen: 0,
            mana: 0,
            mana_base,
            mana_regen: 0,
            resources: Default::default(),
            skills,
            effects: Default::default(),
            diseases: Default::default(),
            damage: Default::default(),
            drained: Default::default(),
            weapon_skills,
            faction_override: None,
            grade: 0,
            summoner: None,
            split_from: None,
            ally: None,
            risen: None,
            mark: None,
            alignment: content.alignment().map(|_| alignment),
            alignment_override: None,
            npc: None,
            talk: Talk::default(),
            home: None,
            // The scenario's spells, known from the start (a test world's fixture).
            spells: def.spells.iter().map(|&s| (s, 1)).collect(),
            streak: None,
            satiation: content.hunger().map(|h| Satiation::new(h.start)),
            stance: CombatStance::Neutral,
            free_blows: FreeBlows::default(),
            talents: Default::default(),
            boons: Vec::new(),
            stat_marks: Default::default(),
        };
        stars::grant(&content, state.character.as_ref(), &[], &mut player.talents);
        // The class abilities of level 1 (spec §8 "Class abilities").
        let given = class::grant(&content, state.character.as_ref(), &mut player, 1);
        player.hp = player.max_hp(&content);
        player.mana = player.max_mana(&content);
        crate::rot::made_at(&mut player.inventory, &content, def.start);
        state.actors.insert(ActorId(0), player);
        if let (Some(from), Some((s, _))) = (pit_from, pit::pit_of(&content, &start.id)) {
            state.pits.insert(s, pit::Pit::new(from));
        }
        instantiate_level(&mut state, &content, &start);
        drop(start); // It may borrow `content`, which `assemble` takes.
        let help = crate::messages::key("help");
        let welcome = format!("Welcome to {}. Press {help} for controls.", def.name);
        state.messages.push(0, welcome);
        if let Some(line) = &def.arrival {
            state.messages.push(0, line.clone());
        }
        if let Some(text) = rooms::pervade_line(&state, &content) {
            state.messages.push(0, text);
        }
        for text in treasury::arrive_lines(&state, &content) {
            state.messages.push(0, text);
        }
        if let Some(text) = enter_room(&mut state, &content) {
            state.messages.push(0, text);
        }
        for text in feature::feature_lines(&state, &content, None) {
            state.messages.push(0, text);
        }
        let mut world = Self::assemble(content, state);
        // What the class's level-1 talents hand down (the aristocrat's Family wealth: its gold
        // and gems, on top of the kit), as a creation pick's would be (`heirloom.rs`).
        let content = world.content.clone();
        let handed: Vec<_> = (given.iter())
            .flat_map(|&id| content.talents().get(id).effects.iter().cloned())
            .filter(TalentEffect::is_handed_down)
            .collect();
        let inherits = !handed.is_empty();
        world.hand_down(handed.into_iter());
        // Animal companion's beast and the witch's familiar (spec §8 "Allies"; `companion.rs`,
        // `familiar.rs`), before the story knows them.
        world.bring_companion();
        world.bring_familiar();
        let kept = world.companion().is_some() || world.familiar().is_some();
        if world.settle_in() || kept || inherits {
            *world.perception =
                Perception::compute(&world.state, &world.content, &mut world.lights);
        }
        if let Some(text) = world.birthday_welcome() {
            world.state.messages.push(0, text);
        }
        if with_kit {
            world.open_creation();
        }
        Ok(world)
    }

    /// Restores a world from saved state. The player's sign's gifts are granted afresh
    /// (`world/stars.rs`), as the content now has them.
    pub fn from_state(content: Arc<Content>, mut state: WorldState) -> Result<Self, WorldError> {
        state.check()?;
        let Some(scenario) = content.scenario(&state.scenario) else {
            return Err(WorldError::UnknownScenario(state.scenario.clone()));
        };
        let campaign = scenario.campaign.as_deref().unwrap_or_default();
        if state.campaign.is_empty() {
            state.campaign = campaign.to_string();
        } else if state.campaign != campaign {
            return Err(WorldError::InconsistentState(format!(
                "campaign `{}` is not scenario `{}`'s",
                state.campaign, state.scenario
            )));
        }
        if content.vision(&state.vision).is_none() {
            return Err(WorldError::InconsistentState(format!(
                "unknown vision profile `{}`",
                state.vision
            )));
        }
        if let Some(character) = &state.character {
            character
                .check(&content)
                .map_err(|m| WorldError::InconsistentState(format!("character: {m}")))?;
        }
        if let Some(line) = (state.character.as_ref())
            .and_then(|c| crate::background::unknown_line(&content, &state.campaign, c))
        {
            return Err(WorldError::InconsistentState(format!(
                "story line `{line}` is not in this content"
            )));
        }
        if let Some((actor, id)) = (state.actors.values())
            .find_map(|a| a.talents.unknown_to(&content).map(|id| (a.id, id)))
        {
            return Err(WorldError::InconsistentState(format!(
                "actor {actor} holds talent `{id}`, which this content does not have"
            )));
        }
        if let Some(id) = state.quests.unknown_to(&content) {
            return Err(WorldError::InconsistentState(format!(
                "the quests know `{id}`, which this content does not have"
            )));
        }
        if let Some(id) = (state.revealed.iter())
            .find(|id| content.hidden_location(&state.campaign, id).is_none())
        {
            return Err(WorldError::InconsistentState(format!(
                "hidden location `{id}` was revealed, which this content does not have"
            )));
        }
        if let Some(level) = state.levels.values().find(|l| !l.ids_valid_for(&content)) {
            return Err(WorldError::InconsistentState(format!(
                "level `{}` uses terrain, lights, or objects this content does not have",
                level.id
            )));
        }
        let creatures = content.creature_defs().len();
        let skills = content.skills().len();
        if let Some(actor) = state.actors.values().find(|a| {
            usize::from(a.creature.0) >= creatures
                || !a.inventory.iter().all(|s| s.is_valid_for(&content))
                || !a.equipment.is_valid_for(&a.inventory, &content)
                || a.skills.iter().any(|(id, _)| usize::from(id.0) >= skills)
                || !a.weapon_skills.is_valid_for(content.weapon_skills())
        }) {
            return Err(WorldError::InconsistentState(format!(
                "actor {} is a creature, carries objects, wears them in slots, or knows skills or \
                 weapon skills this content does not have",
                actor.id
            )));
        }
        if let Some(actor) = state
            .actors
            .values()
            .find(|a| !a.effects.is_valid_for(&content, &a.talent_mods(&content)))
        {
            return Err(WorldError::InconsistentState(format!(
                "actor {} suffers status effects this content does not have",
                actor.id
            )));
        }
        // Diseased exactly while a disease shows; stat damage no score could carry is refused.
        let diseased = |a: &Actor| content.disease_status().is_some_and(|s| a.effects.has(s));
        if let Some(actor) = state.actors.values().find(|a| {
            !a.diseases.is_valid_for(&content)
                || diseased(a) != a.diseases.shows()
                || !a.damage.is_valid_for(&a.stats)
        }) {
            return Err(WorldError::InconsistentState(format!(
                "actor {} carries diseases or stat damage this content does not allow",
                actor.id
            )));
        }
        // What the class gives up to the player's level, if its table grew since (spec §8
        // "Class abilities"); a birth sign's gifts first (spec §8 "Birth sign").
        if let Some(player) = state.actors.get_mut(&state.player) {
            let boons = std::mem::take(&mut player.boons);
            stars::grant(
                &content,
                state.character.as_ref(),
                &boons,
                &mut player.talents,
            );
            player.boons = boons;
        }
        let mut world = Self::assemble(content, state);
        world.grant_class(world.player_level(), false);
        Ok(world)
    }

    fn assemble(content: Arc<Content>, mut state: WorldState) -> Self {
        stat_training::schedule(&mut state, &content);
        let mut lights = LightCache::default();
        let perception = Box::new(Perception::compute(&state, &content, &mut lights));
        perception.remember(state.current_mut());
        let time = state.time;
        let player = state.player;
        let stood_on = feature::standing(&state);
        let labels = Labels::draw(&content, state.seed);
        Self {
            content,
            state: Box::new(state),
            perception,
            lights,
            labels,
            said: 0,
            fallen: Vec::new(),
            news: Vec::new(),
            notes: Vec::new(),
            held: None,
            sleeper_struck: None,
            caught: false,
            glassing: None,
            sprung: 0,
            rigger: None,
            spawned: 0,
            reshaped: 0,
            found: 0,
            mutters: Vec::new(),
            resting: false,
            waiting: false,
            running: false,
            perceived_at: time,
            stood_on,
            acting: Some(player),
            fresh: Vec::new(),
            reflection: Default::default(),
            technique: weapon_active::Technique::default(),
            encounter: Default::default(),
        }
    }

    pub fn content(&self) -> &Arc<Content> {
        &self.content
    }

    /// The campaign the run is of (spec §3).
    pub fn campaign(&self) -> &str {
        &self.state.campaign
    }

    /// The content fingerprint a save of the run carries: its campaign's
    /// ([`Content::fingerprint_for`]).
    pub fn fingerprint(&self) -> u64 {
        self.content.fingerprint_for(&self.state.campaign)
    }

    pub fn state(&self) -> &WorldState {
        &self.state
    }

    pub fn scenario(&self) -> &str {
        &self.state.scenario
    }

    pub fn seed(&self) -> u64 {
        self.state.seed
    }

    /// The player's quests and the kills they count (spec §8 "Quests"). For dev tools; the
    /// player's quest log reads [`PerceptionView::quests`].
    pub fn quests(&self) -> &Quests {
        &self.state.quests
    }

    /// The player's character, if the run has one (spec §8 "Character creation").
    pub fn character(&self) -> Option<&Character> {
        self.state.character.as_ref()
    }

    /// Whether the run started with the character's starting kit ([`World::start_run`]).
    pub fn started_with_kit(&self) -> bool {
        self.state.kit
    }

    pub fn turn(&self) -> u64 {
        self.state.turn
    }

    /// The game clock. Month names are in [`Content::calendar`].
    pub fn time(&self) -> GameTime {
        self.state.time
    }

    pub fn player_id(&self) -> ActorId {
        self.state.player
    }

    pub fn player_pos(&self) -> Pos {
        self.state.player().pos
    }

    /// How actors `a` and `b` stand toward each other: the stance of their factions (spec §7
    /// "Factions"): their types' unless a level placed them elsewhere; [`Stance::Neutral`] if either is unknown,
    /// or if they are a Beast-friend and an animal they befriended ([`beast_friend::befriended`]).
    pub fn stance(&self, a: ActorId, b: ActorId) -> Stance {
        match (self.state.actors.get(&a), self.state.actors.get(&b)) {
            (Some(a), Some(b)) => stance_between(&self.state, &self.content, a, b),
            _ => Stance::Neutral,
        }
    }

    pub fn messages(&self) -> &MessageLog {
        &self.state.messages
    }

    /// What the player perceives right now. Rendering and dumps must go through this.
    pub fn perception(&self) -> PerceptionView<'_> {
        PerceptionView {
            content: &self.content,
            level: self.state.current(),
            player: self.state.player(),
            character: self.state.character.as_ref(),
            campaign: &self.state.campaign,
            time: self.state.time,
            turn: self.state.turn,
            death: self.state.death.as_ref(),
            experience: self.state.experience,
            progress: &self.state.progress,
            quests: &self.state.quests,
            patron: &self.state.patron,
            messages: &self.state.messages,
            lore: self.lore(),
            snapshot: &self.perception,
            encounter: self.state.encounter.as_ref(),
            seed: self.state.seed,
        }
    }

    /// What the player knows of scroll types ([`crate::scroll_label`]): how they name what they
    /// see.
    pub fn lore(&self) -> Lore<'_> {
        Lore {
            content: &self.content,
            labels: &self.labels,
            known: &self.state.known_scrolls,
        }
    }

    /// The weight of actor `id`, on any level: its creature's base weight plus everything it
    /// carries (tests; the player is shown only their own,
    /// [`PerceptionView::own_weight`]).
    #[cfg(test)]
    pub fn actor_weight(&self, id: ActorId) -> Option<crate::object::Weight> {
        self.state.actors.get(&id).map(|a| a.weight(&self.content))
    }

    /// What actor `id`, on any level, carries against what it can (spec §8 "Burden"). For the
    /// rules and dev tools; the player is shown only their own ([`PerceptionView::own_load`]).
    pub fn actor_load(&self, id: ActorId) -> Option<Load> {
        (self.state.actors.get(&id)).map(|a| self.state.load(&self.content, a))
    }

    /// What actor `id` carries against what it can.
    fn load_of(&self, id: ActorId) -> Load {
        self.state.load(&self.content, &self.state.actors[&id])
    }

    /// Ground truth for the current level, **for developer tools only** (reveal mode, debug
    /// dumps). Gameplay presentation must use [`World::perception`].
    pub fn ground_truth(&self) -> (&LevelState, impl Iterator<Item = &Actor>) {
        let level = self.state.current();
        let actors = self
            .state
            .actors
            .values()
            .filter(move |a| a.level == level.id);
        (level, actors)
    }

    /// What the special rooms do at `pos` on level `level` (spec §7 "Special rooms"): the effect
    /// of the room there and the one covering the level, none for a level not visited. This is
    /// the one question the rules ask of special rooms (through the level's state inside the
    /// core); ground truth, **for the rules, tests, and developer tools**: the player knows only
    /// the rooms they have been in ([`PerceptionView::room`]).
    pub fn effects_at(&self, level: &str, pos: Pos) -> Here<'_> {
        (self.state.levels.get(level))
            .map_or_else(Here::default, |l| l.effects_at(&self.content, pos))
    }

    /// Applies one player action and runs the rest of the turn if it took time. Once the player
    /// has died, nothing more happens: every action is refused.
    pub fn apply(&mut self, action: &Action) -> Outcome {
        if self.state.death.is_some() {
            return Outcome::default();
        }
        self.fallen.clear();
        let (turn, time) = (self.state.turn, self.state.time);
        let mut events = Vec::new();
        let load = self.load_of(self.state.player);
        let (burden, slowed) = (load.burden(), load.penalty() != Burden::Unburdened);
        let quests = self.quest_progress();
        let spent = self.player_acts(action, &mut events);
        self.forget_floors(&events);
        self.drop_orphan_pits(&events);
        // Taking off what raised them lowers the player's maximum hit points, mana, and pools.
        self.keep_hp_within_max(self.state.player);
        self.keep_mana_within_max(self.state.player);
        self.keep_resources_within_max(self.state.player);
        self.burden_news(burden);
        self.quest_news(&quests);
        // Acting (not waiting or searching, `Spent::Search`) under a load that slows the player
        // trains balance.
        let acted = matches!(spent, Spent::Turn(_));
        if acted && slowed && self.trains(Use::Burdened) {
            self.train(Use::Burdened);
        }
        // Acting (not waiting or dropping) while Overloaded costs Dexterity marks.
        if acted && burden == Burden::Overloaded && !matches!(action, Action::Drop(_)) {
            self.stat_deed(crate::content::StatDeed::Overloaded);
        }
        if acted {
            self.train_hungry_step();
        }
        // A status effect that lowers Strength during the turn may change it too, and a wound
        // the player gave may kill what a quest asks for.
        let burden = self.load_of(self.state.player).burden();
        let quests = self.quest_progress();
        match spent {
            Spent::Nothing => {}
            Spent::NoTime => self.perceive(&mut events),
            Spent::Turn(price) => {
                self.run_turn(price, &mut events);
                self.perceive(&mut events);
            }
            Spent::Rest(until, goal) => self.rest(until, goal, &mut events),
            Spent::Run(dir) => self.run(dir, &mut events),
            Spent::Meal(dish) => self.dine(dish, &mut events),
            Spent::Dig(at) => self.dig(at, &mut events),
            Spent::Search(turns) => {
                // A sound heard again while waiting trains Listen a little (`perceive.rs`).
                self.waiting = true;
                self.search(turns, &mut events);
                self.waiting = false;
            }
            Spent::Study(lesson) => self.study(lesson, &mut events),
            Spent::Scribe(writing) => self.scribe(writing, &mut events),
            Spent::Calling(turns) => self.calling(turns, &mut events),
            Spent::Consecrate(until) => self.consecrate(until, &mut events),
            Spent::Sanctuary(until) => self.sanctuary(until, &mut events),
            Spent::Pray(prayer) => self.pray(prayer, &mut events),
            Spent::Drink(draught) => self.drink(draught, &mut events),
            Spent::Forage(seconds) => self.forage(seconds, &mut events),
            Spent::Gather(seconds) => self.gather(seconds, &mut events),
            Spent::Work(work) => self.work(work, &mut events),
            Spent::Tame(taming) => self.tame(taming, &mut events),
            Spent::Skip(duration) => {
                // Nobody acts or gains energy.
                self.state.time += duration;
                self.player_mut().heard.clear();
                self.perceive(&mut events);
            }
        }
        self.state.listening = None;
        // A merchant's stake takes every week's return begun meanwhile (`invest.rs`).
        self.tick_fund();
        self.encounter_tick(&mut events);
        if self.state.death.is_none() {
            self.burden_news(burden);
            self.quest_news(&quests);
        }
        // A level changed past `Actors::set_level` would leave the index behind (`actors.rs`).
        debug_assert!(
            self.state.actors.indexed(),
            "the actors' level index is stale"
        );
        Outcome {
            took_turn: self.state.turn != turn,
            elapsed: self.state.time.since(time),
            events,
        }
    }

    fn player_mut(&mut self) -> &mut Actor {
        let player = self.state.player;
        self.state
            .actors
            .get_mut(&player)
            .expect("the player actor exists")
    }

    /// Appends a system notice (for example "Game saved.") to the message log.
    pub fn notice(&mut self, text: impl Into<String>) {
        self.say(text);
    }

    /// Logs a message; it stops a rest. While the player strikes in another's turn (a free blow,
    /// `free_blow.rs`), it is held instead, to be said with the news after that turn's reports.
    fn say(&mut self, text: impl Into<String>) {
        if let Some(held) = &mut self.held {
            held.push(text.into());
            return;
        }
        self.note(text);
        self.said += 1;
    }

    /// Logs a message that does not stop a rest.
    fn note(&mut self, text: impl Into<String>) {
        let turn = self.state.turn;
        self.state.messages.push(turn, text);
    }

    /// A turn of the player's action `kind` if they did it, else nothing.
    fn took(&self, done: bool, kind: ActionKind) -> Spent {
        if done {
            Spent::Turn(self.price_of(self.state.player, kind))
        } else {
            Spent::Nothing
        }
    }

    /// Phase 1. Returns what the action cost.
    fn player_acts(&mut self, action: &Action, events: &mut Vec<Event>) -> Spent {
        if let Some(refused) = self.refuse_due(action) {
            return refused;
        }
        if let Some(refused) = self.refuse_pending(action) {
            return refused;
        }
        // What bonds stop or slow is a struggle to break free (`hold.rs`).
        if let Some(spent) = self.struggle(action, events) {
            return spent;
        }
        if let Some(refused) = self.refuse_blocked(action) {
            return refused;
        }
        let here = self.player_pos();
        // Whether the action was done, and what kind of action it was for its cost (the same on
        // any level: taking stairs ends on another).
        let (done, kind) = match action {
            Action::Move(dir) => return self.player_moves(*dir, events),
            Action::Run(dir) => return self.start_running(*dir, events),
            Action::Open(dir) => return self.open_door(here.step(*dir), events),
            Action::Kick(dir) => return self.kick(*dir, events),
            Action::PickLock(dir) => return self.pick_lock(*dir, events),
            Action::Close(dir) => (
                self.close_door(here.step(*dir), events),
                ActionKind::CloseDoor,
            ),
            Action::Descend => (
                self.take_stairs(StairsDir::Down, events),
                ActionKind::Stairs,
            ),
            Action::Ascend => (self.take_stairs(StairsDir::Up, events), ActionKind::Stairs),
            Action::UseTool => return self.use_tool(),
            Action::PickUp(positions) => return self.pick_up_objects(positions),
            Action::Drop(positions) => return self.drop_objects(positions),
            Action::Sell(positions) => {
                return self.trade(positions, true).unwrap_or_else(|| {
                    self.say("There is nobody here to sell to.");
                    Spent::Nothing
                });
            }
            Action::Pay(then) => return self.pay_then(*then, events),
            Action::Equip { slot, item } => return self.equip(slot, *item),
            Action::Remove(slot) => return self.remove(slot),
            Action::Bash(dir) => return self.bash(*dir, events),
            Action::Fire(at) => return self.fire(*at, events),
            Action::Throw { item, at } => return self.throw(*item, *at, events),
            Action::Rest(goal) => return self.start_rest(*goal),
            Action::Encounter(choice) => return self.answer_encounter(*choice),
            Action::Chat(dir) => return self.chat(*dir, events),
            Action::Ask { dir, topic } => return self.ask(*dir, topic, events),
            Action::Steal(dir) => return self.steal(*dir),
            Action::Feint(dir) => return self.feint(*dir, events),
            Action::Tame(dir) => return self.start_tame(*dir),
            Action::Intimidate(at) => return self.intimidate(*at, events),
            Action::ThrowVoice(at) => return self.throw_voice(*at, events),
            Action::Order(command) => return self.order(*command, events),
            Action::Eat(meal) => return self.eat(*meal, events),
            Action::Read { item, target } => return self.read(*item, *target, events),
            Action::Scribe { spell } => return self.start_scribe(spell),
            Action::Dig(dir) => return self.start_dig(*dir),
            // Waiting is searching (#1326): a turn of it looks around, one code path. Afloat, the
            // turn passes without the checks (`search_turn`); searching for a while is refused.
            Action::Search(turns)
                if *turns > 1 && self.refused_afloat("You cannot search in deep water.") =>
            {
                return Spent::Nothing;
            }
            Action::Wait => return Spent::Search(1),
            Action::Search(turns) => return Spent::Search(*turns),
            Action::Disarm(dir) => return self.disarm(*dir, events),
            Action::Listen(dir) => return self.listen(*dir),
            Action::Wedge(dir) => return self.wedge(*dir),
            Action::Heal { what, ally } => return self.heal(what.as_deref(), *ally, events),
            Action::Pray => return self.start_prayer(),
            Action::Drink => return self.start_drink(),
            Action::Forage => return self.start_forage(),
            Action::Gather(dir) => return self.start_gather(*dir),
            Action::Craft { recipe, picks } => return self.start_craft(recipe, picks),
            Action::Decipher { item } => return self.start_decipher(*item),
            Action::Temper { item } => return self.start_temper(*item),
            Action::Whittle { item } => return self.start_whittle(*item),
            Action::Salvage { item } => return self.start_salvage(*item),
            Action::Rig { rig, dir } => return self.start_rig(rig, *dir),
            Action::Give { item, ally } => return self.give_collar(*item, *ally),
            Action::UseTalent(TalentActive::SecondWind) => return self.second_wind(events),
            Action::UseTalent(TalentActive::BloodMagic) => return self.blood_magic(),
            Action::Use(active, dir) => return self.use_class(*active, *dir, events),
            Action::UseAt(active, at) => return self.use_class_at(*active, *at, events),
            Action::UseOn(active, item) => return self.use_class_on(*active, *item),
            Action::UseThrow(active, item, at) => {
                return self.use_class_throw(*active, *item, *at, events);
            }
            Action::Shape(active, shape) => return self.use_class_shape(*active, shape),
            Action::Gift(active, at) => return self.use_gift(*active, *at, events),
            // A `use:ID` that names no spell but a weapon active is that active.
            Action::Cast(id, at)
                if self.content.spell_id(id).is_none()
                    && self.content.weapon_ability_id(id).is_some() =>
            {
                let aim = at.map_or(weapon_active::Aimed::Nothing, weapon_active::Aimed::Cell);
                return self.use_weapon(id, aim, events);
            }
            Action::Cast(spell, at) => {
                let at = at.map(crate::action::ReadTarget::Cell);
                return self.cast_spell(spell, at, events);
            }
            Action::Act(id, dir) => {
                return self.use_weapon(id, weapon_active::Aimed::Dir(*dir), events);
            }
            Action::CastOn(spell, on) => {
                let on = Some(crate::action::ReadTarget::Item(*on));
                return self.cast_spell(spell, on, events);
            }
            Action::Talent { id, choice } => return self.take_talent(id, choice.as_deref()),
            Action::Advance(skill) => return self.advance_skill(skill),
            Action::SetVision(id) => return self.set_vision(id),
            Action::SetStance(stance) => return self.set_stance(*stance),
            Action::SetWeather(weather) => return self.set_weather_dev(*weather),
            Action::SkipTime(duration) => return self.skip_time_dev(*duration),
            Action::Spawn => return self.spawn_dev(),
            Action::Ally {
                creature,
                count,
                lasts,
            } => return self.ally_dev(creature, *count, *lasts),
            Action::Meet { entry, outcome } => return self.meet_dev(entry, *outcome),
            Action::Effect { effect, at, to } => {
                return self.show_effect(effect, *at, *to, events);
            }
            Action::Identify(positions) => return self.identify_dev(positions),
            Action::Freeze { at, radius } => return self.freeze_dev(*at, *radius, events),
            Action::Thaw { at, radius } => return self.thaw_dev(*at, *radius, events),
            Action::Zone { kind, at, radius } => {
                return self.zone_dev(*kind, *at, *radius, events);
            }
            Action::Afflict(id) => return self.afflict_dev(id, events),
            Action::Drain(amount) => return self.drain_dev(*amount),
            Action::Resource(id) => return self.grant_resource_dev(id, events),
            Action::Infect(id) => return self.infect_dev(id),
            Action::Reveal(id) => return self.reveal_dev(id),
            Action::Lure(id) => return self.lure_dev(id),
            Action::Profane(id) => return self.gift_dev(id),
            Action::Rescind(id) => return self.rescind_dev(id),
            Action::QuestRoll { quest, target } => return self.quest_roll_dev(quest, target),
            Action::LevelUpAuto => return self.level_up_auto(),
            Action::GrantXp(xp) => return self.grant_xp_dev(*xp),
            Action::Call(id) => return self.call_dev(id),
            Action::SetRank { skill, rank } => return self.set_rank_dev(skill, *rank),
        };
        self.took(done, kind)
    }

    /// The player takes combat stance `stance` (spec §8 "Stances"). Takes no time, but a wait's
    /// for those set in their ways (spec §8 "Birth sign").
    /// The frenzy stance is refused, and so is any while rage holds the player in it
    /// (`frenzy.rs`).
    fn set_stance(&mut self, stance: CombatStance) -> Spent {
        if let Some(refused) = self.stance_refused(stance) {
            return refused;
        }
        self.player_mut().stance = stance;
        self.say(format!("You take {} stance.", stance.phrase()));
        if self
            .state
            .player()
            .talent_mods(&self.content)
            .defence
            .stance_costs
        {
            return Spent::Turn(self.price_of(self.state.player, ActionKind::Wait));
        }
        Spent::NoTime
    }
}

/// How actors `a` and `b` stand toward each other ([`World::stance`], for a borrowed state): the
/// stance of their factions, [`Stance::Neutral`] for a Beast-friend and an animal they befriended.
pub(crate) fn stance_between(
    state: &WorldState,
    content: &Content,
    a: &Actor,
    b: &Actor,
) -> Stance {
    // A faction friendly to nothing is hostile to its own, never to itself.
    if a.id == b.id || beast_friend::befriended(state, content, a, b) {
        return Stance::Neutral;
    }
    content.stance_of(a.faction(content), b.faction(content))
}
