//! Talents an actor holds (spec §8 "Talents"): the runtime side of
//! [`crate::content::TalentBook`].
//!
//! [`Talents`] is what an actor has taken, saved by talent id (like affixes: adding talents to
//! the data keeps saves) with the choice made, on `Actor::talents`; the
//! player's are picked at level-ups, creatures' are empty until data gives them some.
//! [`TalentMods`] is the one aggregation every rule reads: the typed effects of every talent held,
//! summed, read where the rules already read what magic items do (`Actor::defence`, `melee`,
//! `hearing`, `World::healing_rate`, `burden::load`, `noise::noise`, `step_price`, hunger,
//! experience, searching, free blows). Effects come in areas (#1495), each a module here
//! (`talent/<area>.rs`: `offence`, `defence`, `vigour`, ... `beast`, `slots`) with its enum of
//! effects (`OffenceEffect`, wrapped by [`TalentEffect`]) and its sum (`OffenceMods`, a field
//! here); a new effect is a variant of its area's enum, a field and an arm there, and one read.
//!
//! [`Candidate`] is everything a prerequisite may ask of whoever would take a talent, and
//! [`unmet`] / [`may_take`] decide a pick, for the rules and for what the player is shown alike.

use crate::character::Character;
use crate::content::{
    Choice, Content, MAX_RANK, Need, Sign, StatusId, TalentDef, TalentEffect, TalentId,
};
use crate::skills::Skills;
use crate::stats::Stats;
use crate::status::Effects;
use crate::text::and_list;
use crate::weapon_skills::WeaponSkills;
use crate::world::Actor;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};

mod ally;
mod beast;
mod craft;
mod defence;
mod delving;
mod faith;
mod fortune;
mod growth;
mod hunger;
mod magic;
mod martial;
mod movement;
mod offence;
mod resource;
mod senses;
mod sign;
mod slots;
mod song;
mod stealth;
mod vigour;
pub use ally::{AllyEffect, AllyMods, Sic};
pub use beast::{BeastEffect, BeastMods};
pub use craft::{CraftEffect, CraftMods, Fletcher, MakeDo};
pub use defence::{DefenceEffect, DefenceMods, Guard};
pub use delving::{DelvingEffect, DelvingMods};
pub use faith::{
    Consecrate, FaithEffect, FaithMods, LayOnHands, Prayer, Sanctuary, Smite, SmiteRider,
};
pub use fortune::{FortuneEffect, FortuneMods};
pub use growth::{GrowthEffect, GrowthMods};
pub use hunger::{HungerEffect, HungerMods};
pub use magic::{BloodMagic, Cackle, Knock, MagicEffect, MagicMods};
pub use martial::{
    MartialEffect, MartialMods, PalmBlow, RapidShot, SpiritWeapon, Unstoppable, Volley, WarCry,
};
pub use movement::{MovementEffect, MovementMods, StepBonus};
pub use offence::{OffenceEffect, OffenceMods, WeaponBonus};
pub use resource::{ResourceEffect, ResourceMods};
pub use senses::{SensesEffect, SensesMods};
pub use sign::{Blaze, FireBreath, HornsGrace, HoundsBay, SignEffect, SignMods, Stare};
pub use slots::{Barred, SlotEffect, SlotMods};
pub use song::{Song, SongCost, SongEffect, SongMods};
pub use stealth::{ExploitMods, StealthEffect, StealthMods};
pub use vigour::{Reprieve, Rise, VigourEffect, VigourMods};

/// One degree: a cooldown's default (the areas' `serde` default).
fn one_degree() -> u16 {
    1
}

/// The most percent off a mana cost talents and magic items give together (spec §8 "Mana"):
/// Frugal caster's fifth and of Thrift's percents summed, held here, as speeds are held at
/// [`crate::content::MAX_SPEED`].
pub const MAX_MANA_CUT: i32 = 50;

/// A talent an actor has taken: by id, for this choice (a talent with a `choice` is taken once
/// per choice, any other once).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Taken {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choice: Option<String>,
}

/// The talents an actor holds, in the order taken, with what they do summed and kept
/// ([`Talents::mods_cached`]): derived, never saved or hashed, and dropped by every change.
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Talents(Vec<Taken>, #[serde(skip)] ModsCache);

impl PartialEq for Talents {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Talents {}

impl std::fmt::Debug for Talents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Talents").field(&self.0).finish()
    }
}

/// What [`Talents::mods`] summed, kept for the content it was summed with (#1495: the player's
/// were summed again on each of some 300 reads a turn). Set on the first read after a change;
/// every `&mut` method of [`Talents`] empties it, so a talent taken or given is never missed.
/// Statuses are not in it: the `while` effects are summed afresh while one of theirs is held.
#[derive(Clone, Default)]
struct ModsCache(OnceLock<Kept>);

/// What [`Talents::mods_cached`] hands out: the kept sum, shared ([`std::ops::Deref`] to read
/// it), or an owned copy for a rule that changes it ([`SharedMods::into_owned`]).
#[derive(Clone, Debug, Default)]
pub struct SharedMods(Arc<TalentMods>);

impl std::ops::Deref for SharedMods {
    type Target = TalentMods;

    fn deref(&self) -> &TalentMods {
        &self.0
    }
}

impl SharedMods {
    /// The sum to own: moved out when nothing else shares it, else copied.
    pub fn into_owned(self) -> TalentMods {
        Arc::unwrap_or_clone(self.0)
    }
}

#[derive(Clone)]
struct Kept {
    /// The content summed with, by address: another content sums afresh.
    content: usize,
    /// Every talent's effects but those under a status.
    mods: Arc<TalentMods>,
    /// The statuses the held talents' `while` effects count under.
    whiles: Vec<StatusId>,
}

impl Talents {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Every talent held, in the order taken.
    pub fn iter(&self) -> impl Iterator<Item = &Taken> {
        self.0.iter()
    }

    /// Lets go of talent `id`, every choice of it (tests: a pick made only to spend a level).
    #[cfg(test)]
    pub(crate) fn forget(&mut self, id: &str) {
        self.1 = ModsCache::default();
        self.0.retain(|t| t.id != id);
    }

    /// The first talent held that the content does not have, if any (a saved state is refused
    /// for it: saves are pinned to the version that wrote them, `docs/dev/saves.md`).
    pub fn unknown_to(&self, content: &Content) -> Option<&str> {
        let book = content.talents();
        (self.0.iter())
            .find(|t| book.id(&t.id).is_none())
            .map(|t| t.id.as_str())
    }

    /// Whether talent `id` is held for `choice`.
    pub fn holds(&self, id: &str, choice: Option<&str>) -> bool {
        (self.0.iter()).any(|t| t.id == id && t.choice.as_deref() == choice)
    }

    /// Whether talent `id` is held, for any choice.
    pub fn has(&self, id: &str) -> bool {
        self.0.iter().any(|t| t.id == id)
    }

    /// Takes talent `id` for `choice`, if it is not held already.
    pub(crate) fn take(&mut self, id: &str, choice: Option<&str>) {
        self.1 = ModsCache::default();
        if !self.holds(id, choice) {
            self.0.push(Taken {
                id: id.to_string(),
                choice: choice.map(str::to_string),
            });
        }
    }

    /// Replaces the granted talents held (those `content` marks `granted` outside a class tree:
    /// a race's, the birth sign's; a class's abilities are left alone) with `ids`, first and in
    /// that order (spec §8 "Birth sign": a sign's gifts are computed, never
    /// chosen; a race's likewise). Picked talents keep their order after them; ids the content
    /// lacks are skipped.
    pub(crate) fn regrant(&mut self, content: &Content, ids: &[&str]) {
        self.1 = ModsCache::default();
        let book = content.talents();
        let is_granted = |id: &str| {
            (book.id(id)).is_some_and(|t| {
                let def = book.get(t);
                def.granted && !book.tree(def.tree).class
            })
        };
        self.0.retain(|t| !is_granted(&t.id));
        let given = (ids.iter()).filter(|id| is_granted(id)).map(|id| Taken {
            id: id.to_string(),
            choice: None,
        });
        self.0.splice(0..0, given);
    }

    /// What every talent held does, summed ([`TalentMods`]), with the effects that count only
    /// under a status (`while`) counted while its holder suffers it (`effects`, its statuses):
    /// the one place the condition is read, so no rule knows it exists. Talents the content no
    /// longer has do nothing.
    pub fn mods(&self, content: &Content, effects: &Effects) -> TalentMods {
        self.sum(content, Some(effects))
    }

    /// [`Talents::mods`], kept until the talents change: the same sum, shared. While a status
    /// that one of their `while` effects counts under is held, summed afresh (rarely: a raging
    /// berserker's), so the effects add up in the very order [`Talents::mods`] adds them.
    pub fn mods_cached(&self, content: &Content, effects: &Effects) -> SharedMods {
        let address = std::ptr::from_ref(content) as usize;
        let kept = self.1.0.get_or_init(|| Kept {
            content: address,
            mods: Arc::new(self.sum(content, None)),
            whiles: self.whiles(content),
        });
        if kept.content != address || kept.whiles.iter().any(|s| effects.has(*s)) {
            return SharedMods(Arc::new(self.mods(content, effects)));
        }
        SharedMods(kept.mods.clone())
    }

    /// The statuses the `while` effects of the talents held count under.
    fn whiles(&self, content: &Content) -> Vec<StatusId> {
        let book = content.talents();
        let mut whiles: Vec<StatusId> = (self.0.iter())
            .filter_map(|taken| book.id(&taken.id))
            .flat_map(|id| book.get(id).effects_while.iter().map(|(s, _)| *s))
            .collect();
        whiles.sort_unstable();
        whiles.dedup();
        whiles
    }

    /// What every talent held does, summed, with the `while` effects whose status `effects`
    /// holds (none without `effects`).
    fn sum(&self, content: &Content, effects: Option<&Effects>) -> TalentMods {
        let mut mods = TalentMods::default();
        if self.0.is_empty() {
            return mods;
        }
        let book = content.talents();
        let stars = book.tree_id("stars");
        for taken in &self.0 {
            let Some(id) = book.id(&taken.id) else {
                continue;
            };
            let def = book.get(id);
            let gift = stars.is_some_and(|t| t == def.tree);
            for effect in &def.effects {
                if gift {
                    mods.gift_reach.note(content, effect);
                }
                mods.add(content, effect.clone(), taken.choice.as_deref());
            }
            for (status, effect) in &def.effects_while {
                if effects.is_some_and(|e| e.has(*status)) {
                    mods.add(content, effect.clone(), taken.choice.as_deref());
                }
            }
        }
        mods
    }
}

/// Which of an actor's reaches come of a birth sign's gifts (talents of the `stars` tree), the
/// ones a full moon lengthens (spec §8 "Birth sign", the flavour; the stare and Inner light are
/// gifts only, so they need no mark).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GiftReach {
    /// The statuses of the `scent` entries a gift gives (Hound's nose: bleeding).
    pub scent: Vec<StatusId>,
    /// Whether a gift lengthens its sight on the overworld at night (Eyes of the heights).
    pub night_sight: bool,
}

impl GiftReach {
    fn note(&mut self, content: &Content, effect: &TalentEffect) {
        match effect {
            TalentEffect::Senses(SensesEffect::Scent { status, .. }) => {
                if let Some(status) = content.status_id(status) {
                    self.scent.push(status);
                }
            }
            TalentEffect::Senses(SensesEffect::NightSight { .. }) => self.night_sight = true,
            _ => {}
        }
    }
}

/// What an actor's talents do, summed: one field per effect kind, read by one rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TalentMods {
    /// Which of its reaches are a birth sign's gifts (a full moon lengthens them).
    pub gift_reach: GiftReach,
    /// What its beast talents do: its companion, its familiar, beasts (`talent/beast.rs`).
    pub beast: BeastMods,
    /// What the holder's attacks get ([`OffenceMods`], `talent/offence.rs`).
    pub offence: OffenceMods,
    /// What the holder's defences get ([`DefenceMods`], `talent/defence.rs`).
    pub defence: DefenceMods,
    /// What the holder's body gets: hit points, healing, statuses, diseases ([`VigourMods`],
    /// `talent/vigour.rs`).
    pub vigour: VigourMods,
    /// What the holder's hunger and meals get ([`HungerMods`], `talent/hunger.rs`).
    pub hunger: HungerMods,
    /// What the holder's mana and spells get ([`MagicMods`], `talent/magic.rs`).
    pub magic: MagicMods,
    /// What the holder's senses get ([`SensesMods`], `talent/senses.rs`).
    pub senses: SensesMods,
    /// What the holder's stealth, surprise, and exploits get ([`StealthMods`],
    /// `talent/stealth.rs`).
    pub stealth: StealthMods,
    /// What the holder's movement and burden get ([`MovementMods`], `talent/movement.rs`).
    pub movement: MovementMods,
    /// The equipment slots the holder's body bars or opens ([`SlotMods`], `talent/slots.rs`).
    pub slots: SlotMods,
    /// What the holder gets in the dungeon: searching, traps, chests, rooms, doors, digging
    /// ([`DelvingMods`], `talent/delving.rs`).
    pub delving: DelvingMods,
    /// What the holder's wealth, loot, and standing get ([`FortuneMods`], `talent/fortune.rs`).
    pub fortune: FortuneMods,
    /// What the holder's growth gets: experience, picks, ranks, skills ([`GrowthMods`],
    /// `talent/growth.rs`).
    pub growth: GrowthMods,
    /// What the holder's crafts get ([`CraftMods`], `talent/craft.rs`).
    pub craft: CraftMods,
    /// A birth sign's actives and what makes them stronger ([`SignMods`], `talent/sign.rs`).
    pub sign: SignMods,
    /// What the holy and the unholy get: smites, rites, prayer ([`FaithMods`], `talent/faith.rs`).
    pub faith: FaithMods,
    /// The warrior's, berserker's, ascetic's, and hunter's class actives ([`MartialMods`],
    /// `talent/martial.rs`).
    pub martial: MartialMods,
    /// What the holder's alternative resources get ([`ResourceMods`], `talent/resource.rs`).
    pub resource: ResourceMods,
    /// The bard's songs ([`SongMods`], `talent/song.rs`).
    pub song: SongMods,
    /// What the holder's allies get ([`AllyMods`], `talent/ally.rs`).
    pub ally: AllyMods,
}

/// The active Second wind (spec §8 "Talents"): hit points back at once, then a cooldown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SecondWind {
    pub per_level: u8,
    pub at_least: i32,
    pub cooldown: StatusId,
}

impl SecondWind {
    /// The hit points it gives back at character `level`.
    pub fn hp_at(self, level: u8) -> i32 {
        (i32::from(self.per_level) * i32::from(level)).max(self.at_least)
    }
}

impl TalentMods {
    /// Whether nothing is changed.
    pub fn is_empty(&self) -> bool {
        *self == TalentMods::default()
    }

    /// Whether it has class active `active` (spec §8 "Class abilities").
    pub fn has_active(&self, active: crate::content::ClassActive) -> bool {
        use crate::content::ClassActive as A;
        match active {
            A::Rage => self.martial.rage.is_some(),
            A::WarCry => self.martial.war_cry.is_some(),
            A::Cackle => self.magic.cackle.is_some(),
            A::VeteransWind => self.martial.veterans_wind.is_some(),
            A::Charge => self.martial.charge.is_some(),
            A::LayOnHands => self.faith.lay_on_hands.is_some(),
            A::Smite => self.faith.smite.is_some(),
            A::BrandingSmite | A::WrathfulSmite | A::BanishingSmite => {
                self.faith.smite_mode(active).is_some()
            }
            A::HolyRadiance => self.faith.radiance.is_some(),
            A::PatientDefence => self.martial.patient_defence.is_some(),
            A::StunningStrike => self.martial.stunning_strike.is_some(),
            A::QuiveringPalm => self.martial.quivering_palm.is_some(),
            A::StepOfTheWind => self.martial.wind_step.is_some(),
            A::Stillness => self.martial.stillness.is_some(),
            A::Volley => self.martial.volley.is_some(),
            A::Sic => self.ally.sic.is_some(),
            A::Lurk => self.stealth.lurk.is_some(),
            A::Exploit => self.stealth.exploit_active,
            A::ThrownExploit => self.stealth.thrown_exploit,
            A::MerchantsEye => self.fortune.merchants_eye,
            A::FieldForge => self.craft.field_forge.is_some(),
            A::MakeDo => self.craft.make_do.is_some(),
            A::Rig => self.craft.rigs,
            A::SpiritWeapon => self.martial.spirit_weapon.is_some(),
            A::Inspire | A::Lullaby => self.song.songs.iter().any(|s| s.active == active),
            A::CallFamiliar => self.beast.familiar,
            A::Consecrate => self.faith.consecrate.is_some(),
            A::Sanctuary => self.faith.sanctuary.is_some(),
            A::Knock => self.magic.knock.is_some(),
            A::Scribe => self.magic.scribe.is_some(),
            A::Orison => self.faith.prayer.as_ref().is_some_and(|p| p.tier > 0),
        }
    }

    /// The effects of one talent, summed (what taking it adds); the statuses and damage types
    /// they name are looked up in `content`.
    pub fn of(content: &Content, effects: impl IntoIterator<Item = TalentEffect>) -> Self {
        let mut mods = TalentMods::default();
        for effect in effects {
            mods.add(content, effect, None);
        }
        mods
    }

    /// Adds `effect` of a talent taken for `choice`.
    fn add(&mut self, content: &Content, effect: TalentEffect, choice: Option<&str>) {
        match effect {
            TalentEffect::Beast(effect) => self.beast.add(&effect),
            TalentEffect::Offence(effect) => self.offence.add(content, effect, choice),
            TalentEffect::Defence(effect) => self.defence.add(effect, choice),
            TalentEffect::Vigour(effect) => self.vigour.add(content, effect),
            TalentEffect::Hunger(effect) => self.hunger.add(content, effect),
            TalentEffect::Magic(effect) => self.magic.add(content, effect, choice),
            TalentEffect::Senses(effect) => self.senses.add(content, effect),
            TalentEffect::Stealth(effect) => self.stealth.add(content, effect),
            TalentEffect::Movement(effect) => self.movement.add(content, effect),
            TalentEffect::Slots(effect) => self.slots.add(content, effect),
            TalentEffect::Delving(effect) => self.delving.add(content, effect),
            TalentEffect::Fortune(effect) => self.fortune.add(effect),
            TalentEffect::Growth(effect) => self.growth.add(content, effect, choice),
            TalentEffect::Craft(effect) => self.craft.add(content, effect),
            TalentEffect::Sign(effect) => self.sign.add(content, effect),
            TalentEffect::Faith(effect) => self.faith.add(content, effect),
            TalentEffect::Martial(effect) => self.martial.add(content, effect),
            TalentEffect::Resource(effect) => self.resource.add(content, effect),
            TalentEffect::Song(effect) => self.song.add(content, effect),
            TalentEffect::Ally(effect) => self.ally.add(content, effect),
        }
    }

    /// `rank` (held) raised by `ranks`, never past [`MAX_RANK`]; an untrained skill stays 0.
    pub fn raised(rank: u8, ranks: u8) -> u8 {
        if rank == 0 {
            0
        } else {
            rank.saturating_add(ranks).min(MAX_RANK)
        }
    }

    /// `value` scaled by a signed percent, rounded down, never below 0: the limit at
    /// `carry_percent`, a gain at `xp_percent`.
    pub fn scaled(value: u64, percent: i32) -> u64 {
        let factor = u64::try_from((100 + percent).max(0)).unwrap_or(0);
        value.saturating_mul(factor) / 100
    }
}

/// What per-level effects `(per_levels, at_least)` give at character `level`: for each, one per
/// `per_levels` levels, at least `at_least`.
fn per_levels_at(effects: &[(u8, i32)], level: u8) -> i32 {
    (effects.iter())
        .map(|&(per, least)| (i32::from(level) / i32::from(per.max(1))).max(least))
        .sum()
}

/// Everything a prerequisite may ask of whoever would take a talent (spec §8 "Talents"): the
/// talents held, the stored scores (never the effective ones), the trained skills, the weapon
/// skills, the character level, and the character's race, class, and eyes (the race's vision
/// profile), by id.
#[derive(Clone, Copy, Debug)]
pub struct Candidate<'a> {
    pub talents: &'a Talents,
    pub stats: &'a Stats,
    pub skills: &'a Skills,
    pub weapon_skills: &'a WeaponSkills,
    pub level: u8,
    pub race: Option<&'a str>,
    pub class: Option<&'a str>,
    pub eyes: Option<&'a str>,
    /// The birth sign (spec §8 "Birth sign"), if any, and whether it is a lucky day's.
    pub sign: Option<&'a Sign>,
    pub lucky: bool,
}

impl<'a> Candidate<'a> {
    /// The player as prerequisites see them: `actor` with their `character` (none for a
    /// scenario without one: no race, class, or eyes) at character `level`.
    pub fn player(
        content: &'a Content,
        actor: &'a Actor,
        character: Option<&'a Character>,
        level: u8,
    ) -> Self {
        let spec = character.map(|c| &c.spec);
        let race = spec.and_then(|s| content.race(&s.race));
        Candidate {
            talents: &actor.talents,
            stats: &actor.stats,
            skills: &actor.skills,
            weapon_skills: &actor.weapon_skills,
            level,
            race: spec.map(|s| s.race.as_str()),
            class: spec.map(|s| s.class.as_str()),
            eyes: race.map(|r| r.vision.as_str()),
            sign: character.and_then(|c| c.sign.as_ref()),
            lucky: character.is_some_and(|c| c.lucky(content).is_some()),
        }
    }

    /// Whether `need` holds, for a talent of `choice` kind taken for `chosen`.
    pub fn meets(
        &self,
        content: &Content,
        need: &Need,
        choice: Option<Choice>,
        chosen: Option<&str>,
    ) -> bool {
        match need {
            Need::Talent(id) => {
                let other = content.talents().get(*id);
                // A chain of talents with a choice runs through the same choice.
                if other.choice.is_some() && other.choice == choice {
                    self.talents.holds(&other.id, chosen)
                } else {
                    self.talents.has(&other.id)
                }
            }
            Need::Stat(stat, score) => self.stats.score(*stat) >= *score,
            Need::Skill(id, value) => self.skills.value(*id).is_some_and(|v| v >= *value),
            Need::WeaponSkill(id, rank) => self.weapon_skills.rank(*id) >= *rank,
            Need::AnyWeaponSkill(any) => {
                (any.skills.iter()).any(|id| self.weapon_skills.rank(*id) >= any.rank)
            }
            Need::Chosen(rank) => match (choice, chosen) {
                (Some(kind), Some(id)) if kind.is_weapon_skill() => {
                    (content.weapon_skills().id(id))
                        .is_some_and(|w| self.weapon_skills.rank(w) >= *rank)
                }
                _ => false,
            },
            Need::Level(level) => self.level >= *level,
            Need::Race(id) => self.race == Some(id.as_str()),
            // No race (a run without a character) has no skill of its people's to lack.
            Need::RaceLacks(skill) => (self.race)
                .and_then(|r| content.race(r))
                .is_none_or(|r| !r.teaches(*skill)),
            Need::Eyes(ids) => ids.iter().any(|id| self.eyes == Some(id.as_str())),
            Need::Class(id) => self.class == Some(id.as_str()),
            Need::Animal(id) => self.sign.is_some_and(|s| s.is_animal(id)),
            Need::Element(id) => self.sign.is_some_and(|s| s.is_element(id)),
            Need::Lucky => self.lucky,
            // Creation picks are level 1's; every later pick is a later level's.
            Need::Creation => self.level <= 1,
            Need::Either(needs) => (needs.iter()).any(|n| self.meets(content, n, choice, chosen)),
        }
    }
}

/// Why a talent cannot be taken.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// It is given, never picked.
    Granted,
    /// It takes a choice and none was made.
    NeedsChoice(Choice),
    /// It takes no choice and one was made.
    TakesNoChoice,
    /// The choice names nothing of its kind.
    BadChoice(Choice, String),
    /// It is held already.
    Held,
    /// These prerequisites do not hold.
    Unmet(Vec<Need>),
}

impl Refusal {
    /// The refusal in words, for a talent `name`: "You cannot take Cleave: you need Strength
    /// 13."
    pub fn words(&self, content: &Content, name: &str) -> String {
        match self {
            Refusal::Granted => format!("{name} is given, not chosen."),
            Refusal::NeedsChoice(kind) => format!("{name} needs a choice: {}.", kind.words()),
            Refusal::TakesNoChoice => format!("{name} takes no choice."),
            Refusal::BadChoice(kind, id) => format!("`{id}` is not {}.", kind.words()),
            Refusal::Held => format!("You have {name} already."),
            Refusal::Unmet(needs) => {
                let words: Vec<String> = needs.iter().map(|n| n.words(content)).collect();
                format!("You cannot take {name}: you need {}.", and_list(&words))
            }
        }
    }
}

/// The prerequisites of taking `def` for `chosen` that `who` does not meet, in the order they
/// are shown.
pub fn unmet(
    content: &Content,
    who: &Candidate<'_>,
    def: &TalentDef,
    chosen: Option<&str>,
) -> Vec<Need> {
    (def.needs.iter())
        .filter(|need| !who.meets(content, need, def.choice, chosen))
        .collect()
}

/// Whether `who` may take `def` for `chosen` now, or why not.
pub fn may_take(
    content: &Content,
    who: &Candidate<'_>,
    def: &TalentDef,
    chosen: Option<&str>,
) -> Result<(), Refusal> {
    if def.granted {
        return Err(Refusal::Granted);
    }
    match (def.choice, chosen) {
        (Some(kind), None) => return Err(Refusal::NeedsChoice(kind)),
        (None, Some(_)) => return Err(Refusal::TakesNoChoice),
        (Some(kind), Some(id)) if kind.name(content, id).is_none() => {
            return Err(Refusal::BadChoice(kind, id.to_string()));
        }
        // A hidden weapon skill or style (Two Shields) is no choice until it has marks.
        (Some(kind), Some(id))
            if kind.is_weapon_skill()
                && (content.weapon_skills().id(id))
                    .is_some_and(|w| !who.weapon_skills.shows(content.weapon_skills(), w)) =>
        {
            return Err(Refusal::BadChoice(kind, id.to_string()));
        }
        // An untrained skill does not exist for its actor (Skill focus): nothing to be better at.
        (Some(Choice::Skill), Some(id))
            if (content.skill_id(id)).is_none_or(|s| who.skills.value(s).is_none()) =>
        {
            return Err(Refusal::BadChoice(Choice::Skill, id.to_string()));
        }
        _ => {}
    }
    if who.talents.holds(&def.id, chosen) {
        return Err(Refusal::Held);
    }
    let missing = unmet(content, who, def, chosen);
    if missing.is_empty() {
        Ok(())
    } else {
        Err(Refusal::Unmet(missing))
    }
}

/// The choices `who` may take `def` for now: `[None]` for a talent without a choice that may
/// be taken, the valid choice ids for one with a choice, nothing when it cannot be taken.
pub fn openings(content: &Content, who: &Candidate<'_>, def: &TalentDef) -> Vec<Option<String>> {
    match def.choice {
        None => may_take(content, who, def, None)
            .is_ok()
            .then_some(None)
            .into_iter()
            .collect(),
        Some(kind) => (kind.options(content).into_iter())
            .filter(|id| may_take(content, who, def, Some(id)).is_ok())
            .map(Some)
            .collect(),
    }
}

/// Every talent `who` may take now, with the choices it may be taken for, tree by tree in
/// display order (the pick list).
pub fn offers(content: &Content, who: &Candidate<'_>) -> Vec<(TalentId, Vec<Option<String>>)> {
    (content.talents().by_tree())
        .filter_map(|(id, def)| {
            let ways = openings(content, who, def);
            (!ways.is_empty()).then_some((id, ways))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The kept sum is the sum made afresh: after a talent is taken, given, or let go, and
    /// while a `while` effect's status is held and after it ends.
    #[test]
    fn the_kept_sum_is_the_sum() {
        let content = crate::test_support::builtin();
        let raging = content.status_id("raging").expect("raging");
        let calm = Effects::default();
        let mut rage = Effects::default();
        rage.add(raging, content.status(raging), 1, None);
        let same = |t: &Talents, effects: &Effects| {
            assert_eq!(*t.mods_cached(&content, effects), t.mods(&content, effects));
        };
        let mut talents = Talents::default();
        same(&talents, &calm);
        talents.take("hardy", None);
        same(&talents, &calm);
        talents.take("battle_scarred", None);
        same(&talents, &calm);
        same(&talents, &rage);
        assert_ne!(
            talents.mods(&content, &calm),
            talents.mods(&content, &rage),
            "the rage counts"
        );
        same(&talents, &calm);
        talents.regrant(&content, &["hardy"]);
        same(&talents, &calm);
        talents.forget("battle_scarred");
        same(&talents, &calm);
        talents.forget("hardy");
        assert_eq!(*talents.mods_cached(&content, &calm), TalentMods::default());
    }

    #[test]
    fn hit_points_by_level_grow_with_the_level_and_never_under_the_floor() {
        let mods = TalentMods::of(
            &crate::test_support::CONTENT,
            [TalentEffect::Vigour(VigourEffect::MaxHpPerLevels {
                per_levels: 4,
                at_least: 3,
            })],
        );
        assert_eq!(mods.vigour.hp_at(1), 3);
        assert_eq!(mods.vigour.hp_at(12), 3);
        assert_eq!(mods.vigour.hp_at(16), 4);
        assert_eq!(mods.vigour.hp_at(60), 15);
        let two = TalentMods::of(
            &crate::test_support::CONTENT,
            [
                TalentEffect::Vigour(VigourEffect::MaxHpPerLevels {
                    per_levels: 4,
                    at_least: 3,
                }),
                TalentEffect::Vigour(VigourEffect::MaxHpPerLevels {
                    per_levels: 4,
                    at_least: 3,
                }),
            ],
        );
        assert_eq!(two.vigour.hp_at(20), 10);
    }

    #[test]
    fn effects_sum_and_flags_switch() {
        let mods = TalentMods::of(
            &crate::test_support::CONTENT,
            [
                TalentEffect::Defence(DefenceEffect::Dodge { value: 1 }),
                TalentEffect::Defence(DefenceEffect::Dodge { value: 2 }),
                TalentEffect::Stealth(StealthEffect::FootstepsPercent { value: -20 }),
                TalentEffect::Stealth(StealthEffect::SilentDoors),
                TalentEffect::Delving(DelvingEffect::Notice { value: 2 }),
                TalentEffect::Offence(OffenceEffect::FreeBlow {
                    trigger: crate::content::FreeBlowTrigger::Riposte,
                    one_in: None,
                }),
            ],
        );
        assert_eq!(mods.defence.dodge, 3);
        assert_eq!(mods.stealth.footsteps_percent, -20);
        assert!(mods.stealth.silent_doors);
        assert_eq!(mods.delving.notice, Some(2));
        assert!(mods.offence.free_blows.riposte && !mods.offence.free_blows.cleave);
        assert!(!mods.is_empty());
        assert!(TalentMods::default().is_empty());
        assert_eq!(TalentMods::scaled(300, 10), 330);
        assert_eq!(TalentMods::scaled(300, -15), 255);
        assert_eq!(TalentMods::scaled(300, -150), 0);
    }

    #[test]
    fn a_talent_is_taken_once_per_choice() {
        let mut talents = Talents::default();
        talents.take("hardy", None);
        talents.take("hardy", None);
        talents.take("weapon_focus", Some("bows"));
        talents.take("weapon_focus", Some("slings"));
        assert!(talents.holds("hardy", None));
        assert!(talents.holds("weapon_focus", Some("bows")));
        assert!(!talents.holds("weapon_focus", None));
        assert!(talents.has("weapon_focus"));
        assert!(!talents.has("lean"));
        assert_eq!(talents.len(), 3, "hardy once");
        let json = serde_json::to_string(&talents).expect("json");
        assert_eq!(
            json,
            r#"[{"id":"hardy"},{"id":"weapon_focus","choice":"bows"},{"id":"weapon_focus","choice":"slings"}]"#
        );
        let back: Talents = serde_json::from_str(&json).expect("back");
        assert_eq!(back, talents);
    }

    #[test]
    fn lists_read_naturally() {
        let w = |s: &[&str]| and_list(s);
        assert_eq!(w(&["Strength 13"]), "Strength 13");
        assert_eq!(w(&["Strength 13", "Dodge"]), "Strength 13 and Dodge");
        assert_eq!(w(&["a", "b", "c"]), "a, b, and c");
    }
}
