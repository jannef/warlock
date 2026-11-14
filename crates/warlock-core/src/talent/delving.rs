//! The delving talents (#1495): what a talent does to the holder in the dungeon: searching, traps,
//! spawns, chests and mimics, special rooms, doors, and digging. [`DelvingEffect`] is the data (a
//! talent's `[[effect]]` of one of [`DelvingEffect::KINDS`]), [`DelvingMods`] the sum every rule
//! reads (`TalentMods::delving`). A new effect of the area is a variant here, its field and its arm
//! in [`DelvingMods::add`] here, its wiki text in `xtask/src/wiki/talents/delving.rs`, and its one
//! read.

use crate::content::{Content, StatusId};
use serde::Deserialize;

/// What a talent of the area does, one typed effect, read by one rule each.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DelvingEffect {
    /// Added to its searching, not to noticing in passing (Keen searcher).
    Searching { value: i32 },
    /// The damage traps do it, this percent more (negative: less; Trap sense).
    TrapDamagePercent { value: i32 },
    /// The actor notices hidden things in passing as if trained in Spot and Detect traps, with
    /// this much added.
    Notice { value: i32 },
    /// The actor notices hidden things on stone in passing, as a dwarf does.
    StoneSense,
    /// Added to its checks to find what is hidden on stone, searching and in passing (Stone
    /// sense).
    StoneFind { value: i32 },
    /// Added to every check to find hidden things, searching and in passing.
    Search { value: i32 },
    /// The creatures that come to a level the actor is on come this percent less often (the
    /// time between attempts this percent longer; Quiet delver).
    SpawnEveryPercent { value: i32 },
    /// It tells a locked chest from an unlocked one at a glance (Chest-wise;
    /// [`crate::perception::PerceptionView::chest_at`]), and a shut chest's trap in its sight
    /// becomes known to it (`World::chest_wise_look`).
    ChestWise,
    /// A disguised mimic within this many cells is seen for what it is (Chest-wise;
    /// [`crate::perception::Perception`]'s disguises).
    SeesMimics { value: i32 },
    /// A mimic's grip never holds it (Chest-wise; `World::afflict`).
    Slick,
    /// Where a special room quickens healing or mana, the quickening is this percent more
    /// (Old delver; the room's pace in `World::healing_rate` and `World::mana_rate`).
    RoomPacePercent { value: i32 },
    /// A special room's (or a whole level's) `status` never takes it (Old delver: Ebbing;
    /// `World::dwell`).
    RoomStatusImmune { status: String },
    /// A collapsed floor never gives way under it (Old delver; `World::collapse_step`).
    CollapseProof,
    /// A tempting hoard seals only after this many stacks more are lifted inside it (Old delver;
    /// `World::lift_in_hoard`, `world/fixtures/hoard.rs`).
    HoardGrace { value: i32 },
    /// What it kills never rises in a cursed room (Old delver; `World::curse_corpse`).
    GraveWarden,
    /// Searching reaches this many cells farther.
    SearchReach { value: i32 },
    /// Its door kicks as if this much stronger.
    DoorKick { value: i32 },
    /// Its door kicks are this percent louder (negative: quieter; Lockbreaker).
    KickNoisePercent { value: i32 },
    /// What a chest it kicks open holds breaks this percent more often (negative: less;
    /// Lockbreaker; spec §8 "Chests").
    SmashDestroyPercent { value: i32 },
    /// Its digging does this percent more work a turn.
    DigPercent { value: i32 },
    /// A trap it steps onto first gets a Dexterity check against its difficulty: won, it does
    /// not go off under it, and it is found (Light step).
    SureStep,
    /// Pits never catch it (Light step, Bedrock).
    PitProof,
    /// It can disarm traps (the Disarm ability) without Disable device.
    Disarm,
    /// Hidden traps are this much easier for it to find (none carries it since #684).
    TrapEase { value: i32 },
}

impl DelvingEffect {
    /// The `kind`s of the data that are effects of the area, one a variant (`TalentEffect`'s
    /// deserializer hands these to [`DelvingEffect`]).
    pub const KINDS: [&'static str; 24] = [
        "chest_wise",
        "collapse_proof",
        "dig_percent",
        "disarm",
        "door_kick",
        "grave_warden",
        "hoard_grace",
        "kick_noise_percent",
        "notice",
        "pit_proof",
        "room_pace_percent",
        "room_status_immune",
        "search",
        "search_reach",
        "searching",
        "sees_mimics",
        "slick",
        "smash_destroy_percent",
        "spawn_every_percent",
        "stone_find",
        "stone_sense",
        "sure_step",
        "trap_damage_percent",
        "trap_ease",
    ];
}

/// What an actor's talents of the area do, summed: one field per effect kind, read by one
/// rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DelvingMods {
    /// Added to its searching, not to noticing in passing.
    pub searching: i32,
    /// Percent more damage traps do it (negative: less).
    pub trap_damage_percent: i32,
    /// It notices hidden things in passing, with this much added; `None`: only as its skills
    /// and race allow.
    pub notice: Option<i32>,
    /// It notices hidden things on stone in passing.
    pub stone_sense: bool,
    /// Added to its checks to find what is hidden on stone.
    pub stone_find: i32,
    /// Added to every check to find hidden things.
    pub search: i32,
    /// Percent longer between the attempts to bring creatures to its level (Quiet delver).
    pub spawn_every_percent: i32,
    /// It tells a locked chest from an unlocked one at a glance, and a shut chest's trap in its
    /// sight is known to it (Chest-wise).
    pub chest_wise: bool,
    /// Within this many cells it sees a disguised mimic for what it is (Chest-wise; 0: none).
    pub sees_mimics: i32,
    /// A mimic's grip never holds it (Chest-wise).
    pub slick: bool,
    /// Percent more a special room's quickening of healing and mana (Old delver).
    pub room_pace_percent: i32,
    /// A collapsed floor never gives way under it (Old delver).
    pub collapse_proof: bool,
    /// The statuses of special rooms that never take it (Old delver).
    pub room_status_immune: Vec<StatusId>,
    /// How many stacks more it may lift in a tempting hoard before it seals (Old delver).
    pub hoard_grace: i32,
    /// What it kills never rises in a cursed room (Old delver).
    pub grave_warden: bool,
    /// Cells more its searching reaches.
    pub search_reach: i32,
    /// How much stronger its door kicks are.
    pub door_kick: i32,
    /// Percent louder its door kicks are (negative: quieter).
    pub kick_noise_percent: i32,
    /// Percent more often what a chest it kicks open holds breaks (negative: less; Deft
    /// breaker).
    pub smash_destroy_percent: i32,
    /// Percent more work a turn of digging does.
    pub dig_percent: i32,
    /// A trap it steps onto first gets a Dexterity check.
    pub sure_step: bool,
    /// Pits never catch it.
    pub pit_proof: bool,
    /// It can disarm traps without Disable device.
    pub disarm: bool,
    /// How much easier hidden traps are for it to find (no talent carries it since #684).
    pub trap_ease: i32,
}

impl DelvingMods {
    /// Adds what `effect` does.
    pub(crate) fn add(&mut self, content: &Content, effect: DelvingEffect) {
        match effect {
            DelvingEffect::Searching { value } => self.searching += value,
            DelvingEffect::TrapDamagePercent { value } => self.trap_damage_percent += value,
            DelvingEffect::Notice { value } => {
                self.notice = Some(self.notice.unwrap_or(0) + value);
            }
            DelvingEffect::StoneSense => self.stone_sense = true,
            DelvingEffect::StoneFind { value } => self.stone_find += value,
            DelvingEffect::Search { value } => self.search += value,
            DelvingEffect::SpawnEveryPercent { value } => self.spawn_every_percent += value,
            DelvingEffect::ChestWise => self.chest_wise = true,
            DelvingEffect::SeesMimics { value } => self.sees_mimics += value,
            DelvingEffect::Slick => self.slick = true,
            DelvingEffect::RoomPacePercent { value } => self.room_pace_percent += value,
            DelvingEffect::RoomStatusImmune { status } => {
                if let Some(status) = content.status_id(&status) {
                    self.room_status_immune.push(status);
                }
            }
            DelvingEffect::HoardGrace { value } => self.hoard_grace += value,
            DelvingEffect::CollapseProof => self.collapse_proof = true,
            DelvingEffect::GraveWarden => self.grave_warden = true,
            DelvingEffect::SearchReach { value } => self.search_reach += value,
            DelvingEffect::DoorKick { value } => self.door_kick += value,
            DelvingEffect::KickNoisePercent { value } => self.kick_noise_percent += value,
            DelvingEffect::SmashDestroyPercent { value } => self.smash_destroy_percent += value,
            DelvingEffect::DigPercent { value } => self.dig_percent += value,
            DelvingEffect::SureStep => self.sure_step = true,
            DelvingEffect::PitProof => self.pit_proof = true,
            DelvingEffect::Disarm => self.disarm = true,
            DelvingEffect::TrapEase { value } => self.trap_ease += value,
        }
    }
}
