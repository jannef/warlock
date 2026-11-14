//! Talents (spec §8 "Talents"; `assets/data/talents.toml` for the settings, one file per talent
//! under `assets/data/talents/`): what a character may pick as they level up. A talent has a
//! tree, prerequisites ([`Needs`]) that must all hold when it is chosen, and typed effects ([`TalentEffect`]) summed by [`crate::talent::TalentMods`] and
//! read where the rules already read what magic items do. A talent may name the systems it
//! touches from the list in the settings file (the wiki's matrix reads them). The loader checks
//! only that the data is well formed: what it names resolves (#1205). Without the files there
//! are no talents, and no level asks for one.

use super::level::OneOrMany;
use super::{
    Content, ContentError, ItemKind, ResourceId, Role, StatusId, WeaponSkillId, duplicate_id,
};
use crate::alignment::Requirement;
use crate::skills::SkillId;
use crate::stats::{PerStat, Stat};
use crate::talent::{
    AllyEffect, BeastEffect, CraftEffect, DefenceEffect, DelvingEffect, FaithEffect, FortuneEffect,
    GrowthEffect, HungerEffect, MagicEffect, MartialEffect, MovementEffect, OffenceEffect,
    ResourceEffect, SensesEffect, SignEffect, SongEffect, StealthEffect, VigourEffect,
};
use crate::text::article;
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;

/// A talent, an index into the book (file order). Saves name talents by id, never by this.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TalentId(pub u16);

/// A tree, an index into [`TalentBook::trees`] (display order).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TreeId(pub u8);

/// A system of the settings file's list, an index into [`TalentBook::systems`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemId(pub u8);

/// A tree of talents (`[[tree]]` in the settings file), in display order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeDef {
    pub id: String,
    pub name: String,
    /// One line: what the tree is about.
    pub description: String,
    /// Its talents are class abilities (spec §8 "Class abilities"): given by the class tables
    /// (`data/classes/`), never picked.
    pub class: bool,
}

/// A system of the game that talents may touch (`[[system]]` in the settings file), for the
/// wiki's matrix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemDef {
    pub id: String,
    pub name: String,
}

/// What a talent with a `choice` is chosen for (`talent:weapon_focus:long_blades`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Choice {
    /// A weapon skill (an id in `data/weapon_skills.toml` that is no style).
    WeaponSkill,
    /// A fighting style.
    Style,
    /// A skill (an id in `data/skills.toml`), trained or not.
    Skill,
    /// An element: a damage type (an id in `data/damage_types.toml`).
    Element,
    /// A creature tag (one some creature type carries, `goblin`).
    Tag,
    /// A spell tag a talent may focus on (magic design §6): a school (`evocation`), or an
    /// element as [`Choice::Element`] offers them (`fire`).
    School,
}

impl Choice {
    /// What is chosen, in words: "a weapon skill".
    pub fn words(self) -> &'static str {
        match self {
            Choice::WeaponSkill => "a weapon skill",
            Choice::Style => "a fighting style",
            Choice::Skill => "a skill you are trained in",
            Choice::Element => "an element",
            Choice::Tag => "a kind of creature",
            Choice::School => "a school of magic or an element",
        }
    }

    /// Whether a weapon skill of `role` may be chosen as a weapon skill: one that attacks (a
    /// style is chosen as a style).
    fn attacks(role: Role) -> bool {
        role != Role::Style
    }

    /// Whether it chooses a row of the weapon skill table: a weapon skill or a style.
    pub fn is_weapon_skill(self) -> bool {
        matches!(self, Choice::WeaponSkill | Choice::Style)
    }

    /// Every valid choice of this kind, by id, in file order.
    pub fn options(self, content: &Content) -> Vec<String> {
        match self {
            Choice::WeaponSkill => (content.weapon_skills().ids())
                .filter(|(_, def)| Choice::attacks(def.role))
                .map(|(_, def)| def.id.clone())
                .collect(),
            Choice::Style => (content.weapon_skills().styles())
                .map(|(_, def, _)| def.id.clone())
                .collect(),
            Choice::Skill => content.skills().iter().map(|s| s.id.clone()).collect(),
            Choice::Element => (content.damage_types().iter())
                .filter(|t| !t.physical)
                .map(|t| t.id.clone())
                .collect(),
            Choice::Tag => content.tags().into_iter().map(str::to_string).collect(),
            Choice::School => (super::School::ALL.iter().map(|s| s.id().to_string()))
                .chain(Choice::Element.options(content))
                .collect(),
        }
    }

    /// The name of the choice `id`, if it is one of this kind.
    pub fn name<'c>(self, content: &'c Content, id: &str) -> Option<&'c str> {
        match self {
            Choice::WeaponSkill | Choice::Style => {
                let skill = content.weapon_skills().id(id)?;
                let def = content.weapon_skills().skill(skill);
                let fits = match self {
                    Choice::Style => def.role == Role::Style,
                    _ => Choice::attacks(def.role),
                };
                fits.then_some(def.name.as_str())
            }
            Choice::Skill => {
                let skill = content.skill_id(id)?;
                Some(content.skill(skill).name.as_str())
            }
            Choice::Element => {
                let kind = content.damage_type_id(id)?;
                let def = content.damage_type(kind);
                (!def.physical).then_some(def.name.as_str())
            }
            Choice::Tag => content.tags().into_iter().find(|t| *t == id),
            Choice::School => (super::School::ALL.iter())
                .map(|s| s.id())
                .find(|s| *s == id)
                .or_else(|| Choice::Element.name(content, id)),
        }
    }
}

/// What a talent does, one typed effect, of one area (#1495): each area's enum is in its own
/// module under `talent/` with its sum ([`crate::talent::TalentMods`]'s field of the area) and
/// its one read per kind. A new kind is a variant of its area's enum, its field and its arm in
/// the area's sum, its wiki words in `xtask/src/wiki/talents/<area>.rs`, and its one read.
///
/// The data is flat (`kind = "companion"` as `kind = "dodge"`): the deserializer hands each kind
/// to the enum of its area (the areas' `KINDS`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TalentEffect {
    /// What the holder's companion, familiar, and beasts get ([`BeastEffect`],
    /// `talent/beast.rs`).
    Beast(BeastEffect),
    /// What the holder's attacks get ([`OffenceEffect`], `talent/offence.rs`).
    Offence(OffenceEffect),
    /// What the holder's defences get ([`DefenceEffect`], `talent/defence.rs`).
    Defence(DefenceEffect),
    /// What the holder's body gets: hit points, healing, statuses, diseases ([`VigourEffect`],
    /// `talent/vigour.rs`).
    Vigour(VigourEffect),
    /// What the holder's hunger and meals get ([`HungerEffect`], `talent/hunger.rs`).
    Hunger(HungerEffect),
    /// What the holder's mana and spells get ([`MagicEffect`], `talent/magic.rs`).
    Magic(MagicEffect),
    /// What the holder's senses get ([`SensesEffect`], `talent/senses.rs`).
    Senses(SensesEffect),
    /// What the holder's stealth, surprise, and exploits get ([`StealthEffect`],
    /// `talent/stealth.rs`).
    Stealth(StealthEffect),
    /// What the holder's movement and burden get ([`MovementEffect`], `talent/movement.rs`).
    Movement(MovementEffect),
    /// What the holder gets in the dungeon: searching, traps, chests, rooms, doors, digging
    /// ([`DelvingEffect`], `talent/delving.rs`).
    Delving(DelvingEffect),
    /// What the holder's wealth, loot, and standing get ([`FortuneEffect`], `talent/fortune.rs`).
    Fortune(FortuneEffect),
    /// What the holder's growth gets: experience, picks, ranks, skills ([`GrowthEffect`],
    /// `talent/growth.rs`).
    Growth(GrowthEffect),
    /// What the holder's crafts get ([`CraftEffect`], `talent/craft.rs`).
    Craft(CraftEffect),
    /// A birth sign's actives and what makes them stronger ([`SignEffect`], `talent/sign.rs`).
    Sign(SignEffect),
    /// What the holy and the unholy get: smites, rites, prayer ([`FaithEffect`],
    /// `talent/faith.rs`).
    Faith(FaithEffect),
    /// The warrior's, berserker's, ascetic's, and hunter's class actives ([`MartialEffect`],
    /// `talent/martial.rs`).
    Martial(MartialEffect),
    /// What the holder's alternative resources get ([`ResourceEffect`], `talent/resource.rs`).
    Resource(ResourceEffect),
    /// The bard's songs ([`SongEffect`], `talent/song.rs`).
    Song(SongEffect),
    /// What the holder's allies get ([`AllyEffect`], `talent/ally.rs`).
    Ally(AllyEffect),
}

/// A talent's own active (spec §8 "Talents"; the abilities menu's Talents group), used as
/// [`crate::action::Action::UseTalent`] with the one verb every ability use has
/// (`use:second_wind`, `use:blood_magic`; #644).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum TalentActive {
    /// Second wind: hit points back at once, then Winded for a while.
    SecondWind,
    /// Blood magic: some hit points (never the last) for some mana.
    BloodMagic,
}

impl TalentActive {
    /// Every one, in menu order.
    pub const ALL: &[TalentActive] = &[TalentActive::SecondWind, TalentActive::BloodMagic];

    /// Its id: its effect's kind, and its replay text (`use:second_wind`).
    pub const fn id(self) -> &'static str {
        match self {
            TalentActive::SecondWind => "second_wind",
            TalentActive::BloodMagic => "blood_magic",
        }
    }

    /// The one with id `id`.
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|a| a.id() == id)
    }
}

/// The free blows a talent may switch on ([`crate::world::FreeBlows`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FreeBlowTrigger {
    Cleave,
    GreatCleave,
    Riposte,
    Opportunist,
    /// A melee attack with nothing in hand strikes twice, the second blow at −2 (Flurry).
    Flurry,
    /// A hostile that steps up beside it is struck as it comes (Gatekeeper).
    Gatekeeper,
}

/// What a mode of smiting does besides the blow ([`FaithEffect::SmiteMode`]): each is its own
/// class active.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SmiteRiderKind {
    /// The foe hit suffers a status (Branded: it sheds light; Branding smite).
    Brand,
    /// The foe hit checks its nerve or flees (Frightened; Wrathful smite).
    Fear,
    /// The summoned foe hit is unmade, the unholy takes the dice twice (Banishing smite).
    Banish,
}

impl SmiteRiderKind {
    /// The class active of this mode.
    pub const fn active(self) -> super::ClassActive {
        match self {
            SmiteRiderKind::Brand => super::ClassActive::BrandingSmite,
            SmiteRiderKind::Fear => super::ClassActive::WrathfulSmite,
            SmiteRiderKind::Banish => super::ClassActive::BanishingSmite,
        }
    }
}

/// Whom a song is sung on ([`SongEffect::Song`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SongOn {
    /// The singer suffers its status (Inspire).
    #[serde(rename = "self")]
    Singer,
    /// Every hostile creature that hears it checks against it (the Lullaby, #873).
    Hearers,
}

impl<'de> Deserialize<'de> for TalentEffect {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let value = toml::Value::deserialize(d)?;
        let Some(kind) = value.get("kind").and_then(toml::Value::as_str) else {
            // No kind, or one that is no string: the derived enum's own words for it.
            return BeastEffect::deserialize(value)
                .map(TalentEffect::Beast)
                .map_err(D::Error::custom);
        };
        let effect = if BeastEffect::KINDS.contains(&kind) {
            BeastEffect::deserialize(value).map(TalentEffect::Beast)
        } else if OffenceEffect::KINDS.contains(&kind) {
            OffenceEffect::deserialize(value).map(TalentEffect::Offence)
        } else if DefenceEffect::KINDS.contains(&kind) {
            DefenceEffect::deserialize(value).map(TalentEffect::Defence)
        } else if VigourEffect::KINDS.contains(&kind) {
            VigourEffect::deserialize(value).map(TalentEffect::Vigour)
        } else if HungerEffect::KINDS.contains(&kind) {
            HungerEffect::deserialize(value).map(TalentEffect::Hunger)
        } else if MagicEffect::KINDS.contains(&kind) {
            MagicEffect::deserialize(value).map(TalentEffect::Magic)
        } else if SensesEffect::KINDS.contains(&kind) {
            SensesEffect::deserialize(value).map(TalentEffect::Senses)
        } else if StealthEffect::KINDS.contains(&kind) {
            StealthEffect::deserialize(value).map(TalentEffect::Stealth)
        } else if MovementEffect::KINDS.contains(&kind) {
            MovementEffect::deserialize(value).map(TalentEffect::Movement)
        } else if DelvingEffect::KINDS.contains(&kind) {
            DelvingEffect::deserialize(value).map(TalentEffect::Delving)
        } else if FortuneEffect::KINDS.contains(&kind) {
            FortuneEffect::deserialize(value).map(TalentEffect::Fortune)
        } else if GrowthEffect::KINDS.contains(&kind) {
            GrowthEffect::deserialize(value).map(TalentEffect::Growth)
        } else if CraftEffect::KINDS.contains(&kind) {
            CraftEffect::deserialize(value).map(TalentEffect::Craft)
        } else if SignEffect::KINDS.contains(&kind) {
            SignEffect::deserialize(value).map(TalentEffect::Sign)
        } else if FaithEffect::KINDS.contains(&kind) {
            FaithEffect::deserialize(value).map(TalentEffect::Faith)
        } else if MartialEffect::KINDS.contains(&kind) {
            MartialEffect::deserialize(value).map(TalentEffect::Martial)
        } else if ResourceEffect::KINDS.contains(&kind) {
            ResourceEffect::deserialize(value).map(TalentEffect::Resource)
        } else if SongEffect::KINDS.contains(&kind) {
            SongEffect::deserialize(value).map(TalentEffect::Song)
        } else if AllyEffect::KINDS.contains(&kind) {
            AllyEffect::deserialize(value).map(TalentEffect::Ally)
        } else {
            // The derived enum's words, every area's kinds listed, under the key as toml says it.
            let unknown = serde::de::value::Error::unknown_variant(kind, TalentEffect::kinds());
            return Err(D::Error::custom(format!("{unknown}\nin `kind`")));
        };
        effect.map_err(D::Error::custom)
    }
}

impl TalentEffect {
    /// Every `kind` of the data, of every area, in alphabetical order.
    pub fn kinds() -> &'static [&'static str] {
        static KINDS: std::sync::LazyLock<Vec<&'static str>> = std::sync::LazyLock::new(|| {
            let mut kinds: Vec<&'static str> = (BeastEffect::KINDS.iter())
                .chain(OffenceEffect::KINDS.iter())
                .chain(DefenceEffect::KINDS.iter())
                .chain(VigourEffect::KINDS.iter())
                .chain(HungerEffect::KINDS.iter())
                .chain(MagicEffect::KINDS.iter())
                .chain(SensesEffect::KINDS.iter())
                .chain(StealthEffect::KINDS.iter())
                .chain(MovementEffect::KINDS.iter())
                .chain(DelvingEffect::KINDS.iter())
                .chain(FortuneEffect::KINDS.iter())
                .chain(GrowthEffect::KINDS.iter())
                .chain(CraftEffect::KINDS.iter())
                .chain(SignEffect::KINDS.iter())
                .chain(FaithEffect::KINDS.iter())
                .chain(MartialEffect::KINDS.iter())
                .chain(ResourceEffect::KINDS.iter())
                .chain(SongEffect::KINDS.iter())
                .chain(AllyEffect::KINDS.iter())
                .copied()
                .collect();
            kinds.sort_unstable();
            kinds
        });
        &KINDS
    }
}

impl TalentEffect {
    /// The birth sign's active it is, if it is one (spec §8 "Birth sign").
    pub fn sign_active(&self) -> Option<super::SignActive> {
        use super::SignActive as A;
        Some(match self {
            TalentEffect::Sign(SignEffect::HornsGrace { .. }) => A::HornsGrace,
            TalentEffect::Sign(SignEffect::Stare { .. }) => A::TransfixingStare,
            TalentEffect::Sign(SignEffect::Blaze { .. }) => A::Blaze,
            TalentEffect::Sign(SignEffect::FireBreath { .. }) => A::FireBreath,
            TalentEffect::Sign(SignEffect::HoundsBay { .. }) => A::HoundsBay,
            _ => return None,
        })
    }

    /// The class active it is, if it is one (spec §8 "Class abilities").
    pub fn active(&self) -> Option<super::ClassActive> {
        use super::ClassActive as A;
        Some(match self {
            TalentEffect::Martial(MartialEffect::Rage { .. }) => A::Rage,
            TalentEffect::Martial(MartialEffect::WarCry { .. }) => A::WarCry,
            TalentEffect::Magic(MagicEffect::Cackle { .. }) => A::Cackle,
            TalentEffect::Beast(BeastEffect::Familiar) => A::CallFamiliar,
            TalentEffect::Martial(MartialEffect::VeteransWind { .. }) => A::VeteransWind,
            TalentEffect::Martial(MartialEffect::Charge { .. }) => A::Charge,
            TalentEffect::Faith(FaithEffect::LayOnHands { .. }) => A::LayOnHands,
            TalentEffect::Faith(FaithEffect::Smite { .. }) => A::Smite,
            TalentEffect::Faith(FaithEffect::SmiteMode { mode, .. }) => mode.active(),
            TalentEffect::Faith(FaithEffect::Radiance { .. }) => A::HolyRadiance,
            TalentEffect::Martial(MartialEffect::PatientDefence { .. }) => A::PatientDefence,
            TalentEffect::Martial(MartialEffect::StunningStrike { .. }) => A::StunningStrike,
            TalentEffect::Martial(MartialEffect::QuiveringPalm { .. }) => A::QuiveringPalm,
            TalentEffect::Martial(MartialEffect::WindStep { .. }) => A::StepOfTheWind,
            TalentEffect::Martial(MartialEffect::Stillness { .. }) => A::Stillness,
            TalentEffect::Martial(MartialEffect::Volley { .. }) => A::Volley,
            TalentEffect::Ally(AllyEffect::Sic { .. }) => A::Sic,
            TalentEffect::Stealth(StealthEffect::Lurk { .. }) => A::Lurk,
            TalentEffect::Stealth(StealthEffect::Exploit) => A::Exploit,
            TalentEffect::Stealth(StealthEffect::ThrownExploit) => A::ThrownExploit,
            TalentEffect::Song(SongEffect::Song { id, .. }) => {
                return A::from_id(id).filter(|a| a.sings());
            }
            TalentEffect::Faith(FaithEffect::Consecrate { .. }) => A::Consecrate,
            TalentEffect::Faith(FaithEffect::Sanctuary { .. }) => A::Sanctuary,
            TalentEffect::Faith(FaithEffect::Prayer { tier: Some(1), .. }) => A::Orison,
            TalentEffect::Magic(MagicEffect::Knock { .. }) => A::Knock,
            TalentEffect::Magic(MagicEffect::Scribe { .. }) => A::Scribe,
            TalentEffect::Fortune(FortuneEffect::MerchantsEye) => A::MerchantsEye,
            TalentEffect::Craft(CraftEffect::FieldForge { .. }) => A::FieldForge,
            TalentEffect::Craft(CraftEffect::MakeDo { .. }) => A::MakeDo,
            TalentEffect::Craft(CraftEffect::Rigs) => A::Rig,
            TalentEffect::Martial(MartialEffect::SpiritWeapon { .. }) => A::SpiritWeapon,
            _ => return None,
        })
    }

    /// The name of the class active it is, where that is not its talent's (Stillness, on Still
    /// mind; Call familiar, on Familiar).
    pub fn active_name(&self) -> Option<&str> {
        match self {
            TalentEffect::Martial(MartialEffect::Stillness { name, .. }) => Some(name),
            TalentEffect::Beast(BeastEffect::Familiar) => Some("Call familiar"),
            _ => None,
        }
    }

    /// Whether it is a passive that works only within its talent's `alignment` bands (Communion
    /// with the dark, Retribution; `docs/dev/alignment.md`), as a holy active's gift does.
    #[cfg(test)]
    pub fn band_gated(&self) -> bool {
        matches!(
            self,
            TalentEffect::Faith(FaithEffect::Communion)
                | TalentEffect::Faith(FaithEffect::Retribution { .. })
        )
    }

    /// Whether it is handed down once as it is taken (`world/heirloom.rs`): Family wealth's gold
    /// and gems, Father's sword's heirloom, a trade's skill and tools.
    pub fn is_handed_down(&self) -> bool {
        matches!(
            self,
            TalentEffect::Fortune(FortuneEffect::StartingGold { .. })
                | TalentEffect::Fortune(FortuneEffect::StartingGems { .. })
                | TalentEffect::Fortune(FortuneEffect::Heirloom { .. })
                | TalentEffect::Craft(CraftEffect::Trade { .. })
        )
    }

    /// Whether it may carry a `while` condition: only a kind the one sum reads
    /// ([`crate::talent::Talents::mods`]), never one read off the talent's effects directly (an
    /// active, a spell given, what is rolled or handed down once when it is taken).
    fn may_be_conditional(&self) -> bool {
        self.active().is_none()
            && !matches!(
                self,
                TalentEffect::Vigour(VigourEffect::SecondWind { .. })
                    | TalentEffect::Magic(MagicEffect::BloodMagic { .. })
                    | TalentEffect::Magic(MagicEffect::Spell { .. })
                    | TalentEffect::Faith(FaithEffect::Prayer { .. })
                    | TalentEffect::Faith(FaithEffect::Communion)
                    | TalentEffect::Magic(MagicEffect::SpellMod(_))
                    | TalentEffect::Magic(MagicEffect::SpellType { .. })
                    | TalentEffect::Offence(OffenceEffect::WeaponStamina { .. })
                    | TalentEffect::Growth(GrowthEffect::WeaponRanks { .. })
                    | TalentEffect::Growth(GrowthEffect::TakeUpSkills { .. })
                    | TalentEffect::Growth(GrowthEffect::ExtraPick { .. })
                    | TalentEffect::Fortune(FortuneEffect::StartingGold { .. })
                    | TalentEffect::Fortune(FortuneEffect::StartingGems { .. })
                    | TalentEffect::Fortune(FortuneEffect::Heirloom { .. })
                    | TalentEffect::Craft(CraftEffect::Trade { .. })
                    | TalentEffect::Craft(CraftEffect::TradeBonus { .. })
                    | TalentEffect::Movement(MovementEffect::TerrainAs { .. })
                    | TalentEffect::Senses(SensesEffect::Scent { .. })
                    | TalentEffect::Senses(SensesEffect::NightSight { .. })
            )
    }

    /// Whether what it names resolves in `content` (statuses, resources, damage types, skills,
    /// weapon skills, races, lights, spells, effects), its times parse, and its choice-keyed
    /// forms have the choice they read; content without a table has nothing to check those ids
    /// by: they stay, and do nothing. Numbers are the data author's (#1205): only a die's sides
    /// and a divisor are kept from 0.
    pub(super) fn check(&self, content: &Content, choice: Option<Choice>) -> Result<(), String> {
        let status = |id: &str| {
            (content.status_ids().next().is_none() || content.status_id(id).is_some())
                .then_some(())
                .ok_or_else(|| format!("names an unknown status `{id}`"))
        };
        let resource = |id: &str| {
            (content.resources().is_empty() || content.resource_id(id).is_some())
                .then_some(())
                .ok_or_else(|| format!("names an unknown resource `{id}`"))
        };
        let damage_type = |id: &str| {
            (content.damage_types().is_empty() || content.damage_type_id(id).is_some())
                .then_some(())
                .ok_or_else(|| format!("names an unknown damage type `{id}`"))
        };
        let light = |id: &str| {
            (content.light_presets().is_empty() || content.light_id(id).is_some())
                .then_some(())
                .ok_or_else(|| format!("names an unknown light preset `{id}`"))
        };
        let weapon_skill = |id: &str| {
            let table = content.weapon_skills();
            (table.skills().is_empty() || table.id(id).is_some())
                .then_some(())
                .ok_or_else(|| format!("names an unknown weapon skill `{id}`"))
        };
        let skill = |id: &str| {
            (content.skills().is_empty() || content.skill_id(id).is_some())
                .then_some(())
                .ok_or_else(|| format!("names an unknown skill `{id}`"))
        };
        // A die's sides and a divisor: 0 would roll an empty range or divide by zero.
        let nonzero = |kind: &str, field: &str, v: u32| {
            (v != 0)
                .then_some(())
                .ok_or_else(|| format!("`{kind}`: {field} is at least 1"))
        };
        let duration = |text: &str| text.parse::<crate::time::Duration>().map(|_| ());
        match self {
            TalentEffect::Resource(ResourceEffect::GrantsResource { resource: id })
            | TalentEffect::Martial(MartialEffect::RapidShot { resource: id, .. })
            | TalentEffect::Martial(MartialEffect::FlurryCost { resource: id, .. })
            | TalentEffect::Faith(FaithEffect::Retribution { resource: id, .. })
            | TalentEffect::Martial(MartialEffect::CatchMissiles { resource: id, .. })
            | TalentEffect::Martial(MartialEffect::Rage { resource: id })
            | TalentEffect::Resource(ResourceEffect::ResourceBuild { resource: id, .. })
            | TalentEffect::Resource(ResourceEffect::ResourceMax { resource: id, .. })
            | TalentEffect::Resource(ResourceEffect::ResourceStart { resource: id, .. })
            | TalentEffect::Resource(ResourceEffect::ResourceRate { resource: id, .. }) => {
                resource(id)
            }
            TalentEffect::Ally(AllyEffect::Sic { status: id, .. })
            | TalentEffect::Song(SongEffect::Virtuoso { status: id })
            | TalentEffect::Delving(DelvingEffect::RoomStatusImmune { status: id })
            | TalentEffect::Vigour(VigourEffect::RaisePercent { status: id, .. })
            | TalentEffect::Martial(MartialEffect::VeteransWind { cooldown: id, .. })
            | TalentEffect::Vigour(VigourEffect::SecondWind { cooldown: id, .. })
            | TalentEffect::Stealth(StealthEffect::Lurk { status: id })
            | TalentEffect::Stealth(StealthEffect::SneakKillStatus { status: id })
            | TalentEffect::Stealth(StealthEffect::ExploitStatus { status: id })
            | TalentEffect::Martial(MartialEffect::WindStep { status: id })
            | TalentEffect::Martial(MartialEffect::QuiveringPalm { status: id, .. })
            | TalentEffect::Martial(MartialEffect::StunningStrike { status: id, .. })
            | TalentEffect::Martial(MartialEffect::PatientDefence { status: id })
            | TalentEffect::Martial(MartialEffect::WarCry { status: id, .. })
            | TalentEffect::Offence(OffenceEffect::StatusDamage { status: id, .. })
            | TalentEffect::Senses(SensesEffect::Scent { status: id, .. })
            | TalentEffect::Vigour(VigourEffect::StatusEases { status: id, .. })
            | TalentEffect::Vigour(VigourEffect::StatusLasts { status: id, .. })
            | TalentEffect::Offence(OffenceEffect::Brace { status: id, .. })
            | TalentEffect::Vigour(VigourEffect::NoPenalty { status: id })
            | TalentEffect::Offence(OffenceEffect::CriticalStatus { status: id })
            | TalentEffect::Offence(OffenceEffect::Grip { status: id })
            | TalentEffect::Vigour(VigourEffect::Reprieve { status: id, .. }) => status(id),
            TalentEffect::Song(SongEffect::Song {
                id,
                status: status_id,
                resource: pool,
                ..
            }) => {
                if (super::ClassActive::from_id(id)).is_none_or(|a| !a.sings()) {
                    return Err(format!("`song`: `{id}` is no song's active"));
                }
                if let Some(pool) = pool {
                    resource(pool)?;
                }
                status(status_id)
            }
            TalentEffect::Faith(FaithEffect::Consecrate { time, .. }) => duration(time),
            TalentEffect::Craft(CraftEffect::FieldForge { seconds: 0 }) => {
                Err("`field_forge`: `seconds` is 0".into())
            }
            TalentEffect::Craft(CraftEffect::MakersMark { worth_percent: 0 }) => {
                Err("`makers_mark`: `worth_percent` is 0".into())
            }
            TalentEffect::Craft(CraftEffect::CraftTimePercent { value: 0 }) => {
                Err("`craft_time_percent`: `value` is 0".into())
            }
            TalentEffect::Faith(FaithEffect::Sanctuary { time, lasts, .. }) => {
                duration(time)?;
                duration(lasts)
            }
            TalentEffect::Growth(GrowthEffect::Skill { skill: name, .. }) => match (name, choice) {
                (Some(name), _) => skill(name),
                (None, Some(Choice::Skill)) => Ok(()),
                (None, _) => Err(
                    "`skill` names a skill, or is for the chosen one: `choice = \"skill\"`".into(),
                ),
            },
            TalentEffect::Fortune(FortuneEffect::Heirloom { weapons, .. }) => {
                for race in weapons.keys() {
                    if !content.races().list.is_empty() && content.race(race).is_none() {
                        return Err(format!("`heirloom`: names an unknown race `{race}`"));
                    }
                }
                // Its weapons are checked once the objects load (`check_heirlooms`).
                Ok(())
            }
            // Its tools are checked once the objects load (`check_trades`).
            TalentEffect::Craft(CraftEffect::Trade { skill: name, .. }) => skill(name),
            TalentEffect::Growth(GrowthEffect::WeaponRanks { skills, .. }) => {
                skills.iter().try_for_each(|id| weapon_skill(id))
            }
            TalentEffect::Growth(GrowthEffect::TakeUpSkills { except, .. }) => {
                except.iter().try_for_each(|id| skill(id))
            }
            TalentEffect::Offence(OffenceEffect::UnarmedDie { sides }) => {
                nonzero("unarmed_die", "sides", u32::from(*sides))
            }
            TalentEffect::Offence(OffenceEffect::KickDie { sides }) => {
                nonzero("kick_die", "sides", u32::from(*sides))
            }
            TalentEffect::Stealth(StealthEffect::ExploitThreat { value, sure_at }) => {
                nonzero("exploit_threat", "value", u32::from(*value))?;
                nonzero("exploit_threat", "sure_at", u32::from(*sure_at))
            }
            TalentEffect::Faith(FaithEffect::Smite {
                per_levels, sides, ..
            }) => {
                nonzero("smite", "per_levels", u32::from(*per_levels))?;
                nonzero("smite", "sides", u32::from(*sides))
            }
            TalentEffect::Magic(MagicEffect::HexTithe { per_levels }) => {
                nonzero("hex_tithe", "per_levels", u32::from(*per_levels))
            }
            TalentEffect::Vigour(VigourEffect::MaxHpPerLevels { per_levels, .. }) => {
                nonzero("max_hp_per_levels", "per_levels", u32::from(*per_levels))
            }
            TalentEffect::Magic(MagicEffect::MaxManaPerLevels { per_levels, .. }) => {
                nonzero("max_mana_per_levels", "per_levels", u32::from(*per_levels))
            }
            TalentEffect::Offence(OffenceEffect::FreeBlow {
                one_in: Some(n), ..
            }) => nonzero("free_blow", "one_in", *n),
            TalentEffect::Faith(FaithEffect::SmiteMode {
                mode, status: id, ..
            }) => match mode {
                SmiteRiderKind::Banish => Ok(()),
                _ => status(id),
            },
            TalentEffect::Martial(MartialEffect::Stillness { statuses, .. })
            | TalentEffect::Martial(MartialEffect::Unstoppable { statuses, .. }) => {
                statuses.iter().try_for_each(|id| status(id))
            }
            TalentEffect::Sign(SignEffect::Blaze {
                damage_type: kind,
                fire,
                cooldown,
                ..
            }) => {
                damage_type(kind)?;
                light(fire)?;
                status(cooldown)
            }
            TalentEffect::Faith(FaithEffect::Radiance {
                status: id,
                light: l,
            }) => {
                status(id)?;
                light(l)
            }
            TalentEffect::Senses(SensesEffect::InnerLight { light: l })
            | TalentEffect::Senses(SensesEffect::LanternLight { light: l }) => light(l),
            TalentEffect::Defence(DefenceEffect::Resist {
                damage: Some(kind), ..
            })
            | TalentEffect::Offence(OffenceEffect::Penetration { damage: kind, .. }) => {
                damage_type(kind)
            }
            TalentEffect::Defence(DefenceEffect::Resist { damage: None, .. }) => match choice {
                Some(Choice::Element) => Ok(()),
                _ => Err(
                    "`resist` without a `damage` is for the chosen element: it needs \
                     `choice = \"element\"`"
                        .into(),
                ),
            },
            TalentEffect::Sign(SignEffect::HornsGrace {
                cures, cooldown, ..
            }) => {
                cures.iter().try_for_each(|id| status(id))?;
                status(cooldown)
            }
            TalentEffect::Sign(SignEffect::Stare {
                status: id,
                cooldown,
                ..
            })
            | TalentEffect::Sign(SignEffect::HoundsBay {
                status: id,
                cooldown,
                ..
            }) => {
                status(id)?;
                status(cooldown)
            }
            TalentEffect::Sign(SignEffect::FireBreath {
                damage_type: kind,
                spell,
                cooldown,
                ..
            }) => {
                damage_type(kind)?;
                super::spell::check_effect(content, spell, super::effect::EffectShape::Bolt)?;
                status(cooldown)
            }
            TalentEffect::Vigour(VigourEffect::RiseFromAshes {
                status: id,
                spell,
                damage_type: kind,
                ..
            }) => {
                damage_type(kind)?;
                super::spell::check_effect(content, spell, super::effect::EffectShape::Burst)?;
                status(id)
            }
            TalentEffect::Offence(OffenceEffect::Weapon { role, skill, .. }) => match skill {
                Some(id) => weapon_skill(id),
                None if role.is_some() || choice.is_some_and(Choice::is_weapon_skill) => Ok(()),
                None => Err(
                    "`weapon` without a `role` or a `skill` is for the chosen weapon skill \
                     or style: it needs that `choice`"
                        .into(),
                ),
            },
            TalentEffect::Magic(MagicEffect::SpellMod(spec)) => spec.check(content),
            TalentEffect::Magic(MagicEffect::SpellType { when, .. }) => {
                when.iter().try_for_each(|id| damage_type(id))
            }
            TalentEffect::Offence(OffenceEffect::LauncherRange {
                skill: Some(id), ..
            }) => weapon_skill(id),
            // A spell ability names a spell of data/spells/ of its id (content without spells
            // has nothing to check it by).
            TalentEffect::Magic(MagicEffect::Spell { spell }) => {
                if content.spells().is_empty() || content.spell_id(spell).is_some() {
                    Ok(())
                } else {
                    Err(format!("`spell`: no spell `{spell}` in data/spells/"))
                }
            }
            // Terrain is checked once it has loaded ([`TalentBook::check_terrain`]); the weapon
            // abilities `weapon_stamina` names once they have (`content/weapon_ability.rs`).
            _ => Ok(()),
        }
    }
}

/// One prerequisite of a talent, as it is checked and as it is shown ("Toughness 14").
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Need {
    /// Another talent.
    Talent(TalentId),
    /// A stat score (the stored score, never the effective one) at least this.
    Stat(Stat, u8),
    /// A skill trained, at this value at least.
    Skill(SkillId, u16),
    /// A weapon skill or style at this rank at least.
    WeaponSkill(WeaponSkillId, u8),
    /// Any one of several weapon skills at a rank at least ("a melee weapon skill 3").
    AnyWeaponSkill(AnyWeaponSkill),
    /// The chosen weapon skill or style at this rank at least (talents with a `choice`).
    Chosen(u8),
    /// A character level.
    Level(u8),
    /// A race, by id.
    Race(String),
    /// Not of a race whose people all have this skill (`skills` in `races.toml`: a dwarf's
    /// Metalcraft): a Crafts trade is never one the race teaches everyone.
    RaceLacks(SkillId),
    /// Eyes: the race's vision profile, any one of these ids.
    Eyes(Vec<String>),
    /// A class, by id.
    Class(String),
    /// A birth sign's animal, by id (spec §8 "Birth sign").
    Animal(String),
    /// A birth sign's element, by id.
    Element(String),
    /// Born on a lucky day.
    Lucky,
    /// A new character: taken only at character creation (level 1's creation picks).
    Creation,
    /// Any one of these ("Intelligence 12 or Appraise 20").
    Either(Vec<Need>),
}

impl Need {
    /// The prerequisite in words, as a refusal and the Talents page say it: "Toughness 14",
    /// "Hardy", "Stealth 25", "to be a dwarf".
    pub fn words(&self, content: &Content) -> String {
        match self {
            Need::Talent(id) => content.talents().get(*id).name.clone(),
            Need::Stat(stat, score) => format!("{} {score}", stat.name()),
            Need::Skill(id, value) => format!("{} {value}", content.skill(*id).name),
            Need::WeaponSkill(id, rank) => {
                format!("{} {rank}", content.weapon_skills().skill(*id).name)
            }
            Need::AnyWeaponSkill(any) => format!("{} {}", any.words, any.rank),
            Need::Chosen(rank) => format!("the chosen one at {rank}"),
            Need::Level(level) => format!("level {level}"),
            Need::Race(id) => {
                let name = content.race(id).map_or(id.as_str(), |r| r.name.as_str());
                format!("to be {} {}", article(name), name.to_lowercase())
            }
            Need::Class(id) => {
                let name = content.class(id).map_or(id.as_str(), |c| c.name.as_str());
                format!("to be {} {}", article(name), name.to_lowercase())
            }
            Need::RaceLacks(skill) => {
                let races: Vec<String> = (content.races().teaching(*skill))
                    .map(|r| format!("{} {}", article(&r.name), r.name.to_lowercase()))
                    .collect();
                format!("not to be {}", crate::text::or_list(&races))
            }
            Need::Eyes(ids) => {
                let names: Vec<String> = (ids.iter())
                    .map(|id| content.vision(id).map_or(id.as_str(), |v| v.name.as_str()))
                    .map(str::to_lowercase)
                    .collect();
                format!("{} eyes", crate::text::or_list(&names))
            }
            Need::Animal(id) => {
                let name = (content.horoscope())
                    .and_then(|h| h.animals.iter().find(|a| &a.id == id))
                    .map_or(id.as_str(), |a| a.name.as_str());
                format!("to be born in the year of the {name}")
            }
            Need::Element(id) => {
                let name = (content.horoscope())
                    .and_then(|h| h.elements.iter().find(|e| &e.id == id))
                    .map_or(id.as_str(), |e| e.name.as_str());
                format!("to be {name}-born")
            }
            Need::Lucky => "to be born on a lucky day".into(),
            Need::Creation => "to be a new character".into(),
            Need::Either(needs) => (needs.iter())
                .map(|n| n.words(content))
                .collect::<Vec<_>>()
                .join(" or "),
        }
    }
}

/// A prerequisite met by any one of several weapon skills at a rank (`any_weapon_skill`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnyWeaponSkill {
    /// The skills, in file order.
    pub skills: Vec<WeaponSkillId>,
    pub rank: u8,
    /// How it is said: "a melee or unarmed weapon skill", "Maces and clubs or Unarmed".
    pub words: String,
}

/// The prerequisites of a talent (spec §8 "Talents"): all must hold
/// when it is chosen; none is checked again later.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Needs {
    pub talents: Vec<TalentId>,
    /// Scores by stat; 0 asks nothing.
    pub stats: PerStat<u8>,
    pub skills: Vec<(SkillId, u16)>,
    pub weapon_skills: Vec<(WeaponSkillId, u8)>,
    pub any_weapon_skill: Option<AnyWeaponSkill>,
    pub chosen: Option<u8>,
    /// 0 asks nothing.
    pub level: u8,
    pub race: Option<String>,
    /// A skill the race must not teach everyone (`race_lacks`); left out when no race does.
    pub race_lacks: Option<SkillId>,
    /// The race's vision profiles, any one of which will do; empty asks nothing.
    pub eyes: Vec<String>,
    pub class: Option<String>,
    /// The birth sign's animal and element asked, by id, and a lucky day.
    pub animal: Option<String>,
    pub element: Option<String>,
    pub lucky: bool,
    /// Only at character creation (`creation = true`).
    pub creation: bool,
    /// Any one of these, if set (`either`): at least two.
    pub either: Vec<Need>,
}

impl Needs {
    /// Whether nothing is asked.
    pub fn is_empty(&self) -> bool {
        *self == Needs::default()
    }

    /// Every prerequisite, in the order they are shown: talents, stats, skills, weapon skills,
    /// the choice, the level, the race, the eyes, the class, the sign, then any one of the
    /// alternatives.
    pub fn iter(&self) -> impl Iterator<Item = Need> + '_ {
        let talents = self.talents.iter().map(|&t| Need::Talent(t));
        let stats = (self.stats.iter())
            .filter(|&(_, score)| score > 0)
            .map(|(stat, score)| Need::Stat(stat, score));
        let skills = self.skills.iter().map(|&(s, v)| Need::Skill(s, v));
        let weapons = (self.weapon_skills.iter()).map(|&(w, r)| Need::WeaponSkill(w, r));
        let any = self.any_weapon_skill.clone().map(Need::AnyWeaponSkill);
        let chosen = self.chosen.map(Need::Chosen);
        let level = (self.level > 0).then_some(Need::Level(self.level));
        let race = self.race.clone().map(Need::Race);
        let race_lacks = self.race_lacks.map(Need::RaceLacks);
        let eyes = (!self.eyes.is_empty()).then(|| Need::Eyes(self.eyes.clone()));
        let class = self.class.clone().map(Need::Class);
        let animal = self.animal.clone().map(Need::Animal);
        let element = self.element.clone().map(Need::Element);
        let lucky = self.lucky.then_some(Need::Lucky);
        let creation = self.creation.then_some(Need::Creation);
        let either = (!self.either.is_empty()).then(|| Need::Either(self.either.clone()));
        talents
            .chain(stats)
            .chain(skills)
            .chain(weapons)
            .chain(any)
            .chain(chosen)
            .chain(level)
            .chain(race)
            .chain(race_lacks)
            .chain(eyes)
            .chain(class)
            .chain(animal)
            .chain(element)
            .chain(lucky)
            .chain(creation)
            .chain(either)
    }
}

/// A talent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TalentDef {
    pub id: String,
    pub name: String,
    pub tree: TreeId,
    /// What it is chosen for, if it takes a choice.
    pub choice: Option<Choice>,
    /// What it does, in the game's words (the wiki has the rule).
    pub description: String,
    /// What it asks.
    pub needs: Needs,
    /// What it adds.
    pub effects: Vec<TalentEffect>,
    /// What it adds only while the holder suffers a status (`while = "raging"`:
    /// Battle-scarred's rage half), apart from the plain effects: only the one sum reads them.
    pub effects_while: Vec<(StatusId, TalentEffect)>,
    /// The systems it touches (the coverage list).
    pub systems: Vec<SystemId>,
    /// Given, never picked (a race's, a birth sign's): off every pick list.
    pub granted: bool,
    /// A birth sign's flaw: granted, shown with "but" and dimmed, never a prerequisite.
    pub flaw: bool,
    /// What using its class active costs of an alternative resource (spec §8 "Resources"):
    /// `resource = { id = "focus", cost = 6 }` (0 asks only that it is held), if anything.
    pub resource: Option<ResourceCost>,
    /// The alignment bands its class active works for (spec §8 "Alignment"; `alignment = { min
    /// = "kind" }` in data: the paladin's holy gifts), or its band-gated passive (Communion), if
    /// any. It bars nothing: the ability is had, listed, and used whatever the band, and outside
    /// these bands it does nothing (`World::answered`, `World::communes`).
    pub alignment: Option<Requirement>,
}

/// An ability's cost in an alternative resource (spec §8 "Resources"): unavailable while the
/// user holds less, and spent as it is used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceCost {
    pub resource: ResourceId,
    /// 0 asks only that it is held.
    pub amount: u16,
}

impl TalentDef {
    /// Whether anyone may take it: no prerequisites at all.
    pub fn is_free(&self) -> bool {
        !self.granted && self.needs.is_empty()
    }

    /// Its name with the choice it was taken for: "Weapon focus (long blades)".
    pub fn name_for(&self, content: &Content, choice: Option<&str>) -> String {
        match (self.choice, choice) {
            (Some(kind), Some(id)) => {
                let name = kind.name(content, id).unwrap_or(id);
                format!("{} ({})", self.name, name.to_lowercase())
            }
            _ => self.name.clone(),
        }
    }
}

/// Every talent, the trees, the systems, and the settings.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TalentBook {
    pub trees: Vec<TreeDef>,
    pub systems: Vec<SystemDef>,
    list: Vec<TalentDef>,
    ids: BTreeMap<String, TalentId>,
    /// What a maker's mark adds to an item's worth, in percent (the most a `makers_mark` effect
    /// says; 0 with none).
    mark_percent: u32,
}

impl TalentBook {
    /// What a marked item is worth more, in percent (Maker's mark's `worth_percent`, the most
    /// any talent says; `Worked::value`).
    pub fn mark_percent(&self) -> u32 {
        self.mark_percent
    }

    pub fn get(&self, id: TalentId) -> &TalentDef {
        &self.list[usize::from(id.0)]
    }

    /// The talent with data id `name`.
    pub fn id(&self, name: &str) -> Option<TalentId> {
        self.ids.get(name).copied()
    }

    /// Every talent with its id, in file order.
    pub fn iter(&self) -> impl Iterator<Item = (TalentId, &TalentDef)> {
        (0u16..).zip(&self.list).map(|(i, t)| (TalentId(i), t))
    }

    /// Every talent of `tree`, in file order.
    pub fn of_tree(&self, tree: TreeId) -> impl Iterator<Item = (TalentId, &TalentDef)> {
        self.iter().filter(move |(_, t)| t.tree == tree)
    }

    /// Whether the terrain the effects name (`terrain_as`) is in `content`, once its terrain has
    /// loaded (after the talents; content without terrain has nothing to check them by).
    pub(super) fn check_terrain(&self, content: &Content) -> Result<(), ContentError> {
        if content.terrain_defs().is_empty() {
            return Ok(());
        }
        for (_, def) in self.iter() {
            for effect in &def.effects {
                let TalentEffect::Movement(MovementEffect::TerrainAs { terrains, like }) = effect
                else {
                    continue;
                };
                if let Some(id) =
                    (terrains.iter().chain([like])).find(|id| content.terrain_id(id).is_none())
                {
                    return Err(ContentError::new(
                        &format!("data/talents/{}.toml", def.id),
                        format!("talent `{}` names an unknown terrain `{id}`", def.id),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Every talent with its id, tree by tree in display order, each tree in file order: the
    /// order pick lists and the Talents page use.
    pub fn by_tree(&self) -> impl Iterator<Item = (TalentId, &TalentDef)> {
        (0u8..)
            .zip(&self.trees)
            .flat_map(|(i, _)| self.of_tree(TreeId(i)))
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn tree(&self, id: TreeId) -> &TreeDef {
        &self.trees[usize::from(id.0)]
    }

    /// Whether it has a class tree: whether the classes' level tables give anything.
    pub fn has_class_tree(&self) -> bool {
        self.trees.iter().any(|t| t.class)
    }

    pub fn tree_id(&self, name: &str) -> Option<TreeId> {
        (self.trees.iter().position(|t| t.id == name)).map(|i| TreeId(i as u8))
    }

    pub fn system(&self, id: SystemId) -> &SystemDef {
        &self.systems[usize::from(id.0)]
    }

    pub fn system_id(&self, name: &str) -> Option<SystemId> {
        (self.systems.iter().position(|s| s.id == name)).map(|i| SystemId(i as u8))
    }

    /// The talents that name `system`, in file order.
    pub fn of_system(&self, system: SystemId) -> impl Iterator<Item = (TalentId, &TalentDef)> {
        self.iter()
            .filter(move |(_, t)| t.systems.contains(&system))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TalentFile {
    #[serde(default)]
    tree: Vec<TreeSpec>,
    #[serde(default)]
    system: Vec<SystemSpec>,
    #[serde(default)]
    talent: Vec<TalentSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TreeSpec {
    id: String,
    name: String,
    description: String,
    #[serde(default)]
    class: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SystemSpec {
    id: String,
    name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TalentSpec {
    id: String,
    name: String,
    tree: String,
    choice: Option<Choice>,
    description: String,
    #[serde(default)]
    needs: NeedsSpec,
    #[serde(default)]
    effects: Vec<EffectSpec>,
    #[serde(default)]
    systems: Vec<String>,
    #[serde(default)]
    granted: bool,
    #[serde(default)]
    flaw: bool,
    resource: Option<CostSpec>,
    alignment: Option<Requirement>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CostSpec {
    id: String,
    #[serde(default)]
    cost: u16,
}

/// One effect as written: its kind and fields, and the status it counts only under, if any
/// (`while = "raging"`; the effect's own fields are checked by [`TalentEffect`]).
#[derive(Deserialize)]
struct EffectSpec {
    #[serde(rename = "while", default)]
    under: Option<String>,
    #[serde(flatten)]
    effect: TalentEffect,
}

/// `effects` split into the plain ones and those under a status, each checked; a `while` on
/// content without statuses has nothing to check it by, and its effect does nothing.
#[allow(clippy::type_complexity)]
fn split_effects(
    specs: Vec<EffectSpec>,
    content: &Content,
    choice: Option<Choice>,
) -> Result<(Vec<TalentEffect>, Vec<(StatusId, TalentEffect)>), String> {
    let (mut plain, mut under) = (Vec::new(), Vec::new());
    for spec in specs {
        spec.effect.check(content, choice)?;
        let Some(name) = spec.under else {
            plain.push(spec.effect);
            continue;
        };
        if !spec.effect.may_be_conditional() {
            return Err(format!(
                "`while = \"{name}\"` is only for an effect the talents' sum reads, not {:?}",
                spec.effect
            ));
        }
        match content.status_id(&name) {
            Some(status) => under.push((status, spec.effect)),
            None if content.status_ids().next().is_none() => {}
            None => return Err(format!("`while` names an unknown status `{name}`")),
        }
    }
    Ok((plain, under))
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
struct NeedsSpec {
    talents: Vec<String>,
    stats: BTreeMap<String, u8>,
    skills: BTreeMap<String, u16>,
    weapon_skills: BTreeMap<String, u8>,
    any_weapon_skill: Option<AnySpec>,
    chosen: Option<u8>,
    level: u8,
    race: Option<String>,
    /// A skill the race must not teach everyone (a Crafts trade's own).
    race_lacks: Option<String>,
    /// One vision profile or several, any one of which will do.
    eyes: Option<OneOrMany>,
    class: Option<String>,
    animal: Option<String>,
    element: Option<String>,
    lucky: bool,
    /// Only at character creation: never a level's pick.
    creation: bool,
    /// Alternatives: any one of what it asks will do (`either = { stats = { intelligence = 12
    /// }, skills = { appraise = 20 } }`).
    either: Option<Box<NeedsSpec>>,
}

/// `any_weapon_skill = { roles = ["melee"], of = ["unarmed"], rank = 3 }`: any skill of those
/// roles, or any named, at that rank.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AnySpec {
    #[serde(default)]
    roles: Vec<super::Role>,
    #[serde(default)]
    of: Vec<String>,
    rank: u8,
}

/// Resolves `any_weapon_skill` against `content`'s weapon skills; `None` without them.
fn resolve_any(spec: AnySpec, content: &Content) -> Result<Option<AnyWeaponSkill>, String> {
    let table = content.weapon_skills();
    if table.skills().is_empty() {
        return Ok(None);
    }
    let mut named = Vec::new();
    for name in &spec.of {
        let id =
            (table.id(name)).ok_or_else(|| format!("needs an unknown weapon skill `{name}`"))?;
        named.push(id);
    }
    let skills: Vec<WeaponSkillId> = (table.ids())
        .filter(|(id, def)| spec.roles.contains(&def.role) || named.contains(id))
        .map(|(id, _)| id)
        .collect();
    let mut parts: Vec<String> = (named.iter())
        .map(|&id| table.skill(id).name.clone())
        .collect();
    if !spec.roles.is_empty() {
        let roles: Vec<String> = (spec.roles.iter())
            .map(|r| match r {
                super::Role::Launcher => "missile".to_string(),
                r => r.id().replace('_', " "),
            })
            .collect();
        parts.insert(0, format!("a {} weapon skill", roles.join(" or ")));
    }
    Ok(Some(AnyWeaponSkill {
        skills,
        rank: spec.rank,
        words: parts.join(" or "),
    }))
}

/// A talent as read, before the talents it needs are known.
struct Pending<'a> {
    file: &'a str,
    spec: TalentSpec,
}

/// Resolves `spec`'s prerequisites against `content` and the talent `ids`.
fn resolve_needs(
    spec: NeedsSpec,
    ids: &BTreeMap<String, TalentId>,
    content: &Content,
) -> Result<Needs, String> {
    let mut talents = Vec::new();
    for name in spec.talents {
        let id = *ids
            .get(&name)
            .ok_or_else(|| format!("needs an unknown talent `{name}`"))?;
        talents.push(id);
    }
    let mut stats = PerStat::splat(0u8);
    for (name, score) in spec.stats {
        let stat = Stat::from_id(&name).ok_or_else(|| format!("needs an unknown stat `{name}`"))?;
        stats[stat] = score;
    }
    // Content without skills (no `data/skills.toml`) has nothing to measure a skill need by:
    // it is left out, as skill affixes are; weapon skills likewise.
    let mut skills = Vec::new();
    for (name, value) in spec.skills {
        if content.skills().is_empty() {
            break;
        }
        let id =
            (content.skill_id(&name)).ok_or_else(|| format!("needs an unknown skill `{name}`"))?;
        skills.push((id, value));
    }
    let mut weapon_skills = Vec::new();
    for (name, rank) in spec.weapon_skills {
        if content.weapon_skills().skills().is_empty() {
            break;
        }
        let id = (content.weapon_skills().id(&name))
            .ok_or_else(|| format!("needs an unknown weapon skill `{name}`"))?;
        weapon_skills.push((id, rank));
    }
    let any_weapon_skill = match spec.any_weapon_skill {
        Some(any) => resolve_any(any, content)?,
        None => None,
    };
    // A race, class, or eyes is checked against its table when the content has one; without
    // (a fixture), the need stays and is simply never met.
    if let Some(race) = &spec.race
        && !content.races().list.is_empty()
        && content.race(race).is_none()
    {
        return Err(format!("needs an unknown race `{race}`"));
    }
    if let Some(class) = &spec.class
        && !content.classes().is_empty()
        && content.class(class).is_none()
    {
        return Err(format!("needs an unknown class `{class}`"));
    }
    // A skill no race teaches everyone asks nothing: the need is left out, and so are its words.
    let race_lacks = match &spec.race_lacks {
        Some(_) if content.skills().is_empty() => None,
        Some(name) => {
            let id = (content.skill_id(name))
                .ok_or_else(|| format!("`race_lacks`: unknown skill `{name}`"))?;
            content.races().teaching(id).next().map(|_| id)
        }
        None => None,
    };
    let eyes = spec.eyes.as_ref().map_or(&[][..], OneOrMany::ids).to_vec();
    if let Some(unknown) = eyes.iter().find(|e| content.vision(e).is_none()) {
        return Err(format!("needs unknown eyes `{unknown}` (a vision profile)"));
    }
    let either = match spec.either {
        Some(alternatives) => {
            // Content without skills (a fixture) leaves a skill alternative out, as above.
            let alternatives = resolve_needs(*alternatives, ids, content)?;
            alternatives.iter().collect()
        }
        None => Vec::new(),
    };
    Ok(Needs {
        talents,
        stats,
        skills,
        weapon_skills,
        any_weapon_skill,
        chosen: spec.chosen,
        level: spec.level,
        race: spec.race,
        race_lacks,
        eyes,
        class: spec.class,
        animal: spec.animal,
        element: spec.element,
        lucky: spec.lucky,
        creation: spec.creation,
        either,
    })
}

/// Every `heirloom` effect names weapons (objects load after the talents): the first that
/// names something else, in words.
pub(super) fn check_heirlooms(content: &Content) -> Result<(), String> {
    for (_, def) in content.talents().iter() {
        for effect in &def.effects {
            let TalentEffect::Fortune(FortuneEffect::Heirloom {
                weapons, fallback, ..
            }) = effect
            else {
                continue;
            };
            for object in weapons.values().chain([fallback]) {
                let weapon = (content.object_id(object)).and_then(|o| content.object(o).kind);
                if !matches!(weapon, Some(ItemKind::Weapon | ItemKind::Launcher)) {
                    return Err(format!(
                        "talent `{}`: `heirloom`: `{object}` is not a weapon",
                        def.id
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Every `trade` effect's tools are kit lines that resolve (objects load after the talents):
/// the first that does not, in words.
pub(super) fn check_trades(content: &Content) -> Result<(), String> {
    for (_, def) in content.talents().iter() {
        for effect in &def.effects {
            if let TalentEffect::Craft(CraftEffect::Trade { tools, .. }) = effect {
                (content.kit_lines(tools))
                    .map_err(|m| format!("talent `{}`: `trade`: {m}", def.id))?;
            }
        }
    }
    Ok(())
}

/// Parses the talent table: the settings (`[[tree]]`, `[[system]]`) and the talents, resolved
/// against `content`, which must have its skills, weapon skills, races, classes, vision profiles,
/// and damage types already.
pub(super) fn parse<'a>(
    files: impl IntoIterator<Item = (&'a str, &'a str)>,
    content: &Content,
) -> Result<TalentBook, ContentError> {
    let mut first_file = None;
    let mut trees: Vec<(&str, TreeSpec)> = Vec::new();
    let mut systems: Vec<(&str, SystemSpec)> = Vec::new();
    let mut pending: Vec<Pending<'a>> = Vec::new();
    for (file, text) in files {
        first_file.get_or_insert(file);
        let parsed: TalentFile =
            toml::from_str(text).map_err(|e| ContentError::new(file, e.to_string()))?;
        trees.extend(parsed.tree.into_iter().map(|t| (file, t)));
        systems.extend(parsed.system.into_iter().map(|s| (file, s)));
        pending.extend(parsed.talent.into_iter().map(|spec| Pending { file, spec }));
    }
    let first_file = first_file.unwrap_or("data/talents.toml");
    let mut book = TalentBook {
        trees: Vec::new(),
        systems: Vec::new(),
        list: Vec::new(),
        ids: BTreeMap::new(),
        mark_percent: 0,
    };
    let mut tree_files: BTreeMap<String, &str> = BTreeMap::new();
    for (file, t) in trees {
        if let Some(first) = tree_files.insert(t.id.clone(), file) {
            return Err(duplicate_id("tree", &t.id, file, first));
        }
        book.trees.push(TreeDef {
            id: t.id,
            name: t.name,
            description: t.description,
            class: t.class,
        });
    }
    let mut system_files: BTreeMap<String, &str> = BTreeMap::new();
    for (file, s) in systems {
        if let Some(first) = system_files.insert(s.id.clone(), file) {
            return Err(duplicate_id("system", &s.id, file, first));
        }
        book.systems.push(SystemDef {
            id: s.id,
            name: s.name,
        });
    }
    if book.trees.len() > usize::from(u8::MAX) || book.systems.len() > usize::from(u8::MAX) {
        return Err(ContentError::new(
            first_file,
            "too many trees or systems (at most 255)",
        ));
    }
    if pending.len() > usize::from(u16::MAX) {
        return Err(ContentError::new(first_file, "too many talents"));
    }
    // Every id first, so a talent may need one defined after it.
    let mut files: BTreeMap<String, &str> = BTreeMap::new();
    for (i, p) in pending.iter().enumerate() {
        let spec = &p.spec;
        if spec.id.is_empty() || spec.id.contains(':') || spec.id.contains(char::is_whitespace) {
            return Err(ContentError::new(
                p.file,
                format!("`{}` is no talent id", spec.id),
            ));
        }
        if let Some(first) = files.insert(spec.id.clone(), p.file) {
            return Err(duplicate_id("talent", &spec.id, p.file, first));
        }
        book.ids.insert(spec.id.clone(), TalentId(i as u16));
    }
    for p in pending {
        let spec = p.spec;
        let err = |m: String| ContentError::new(p.file, format!("talent `{}`: {m}", spec.id));
        let tree = (book.tree_id(&spec.tree))
            .ok_or_else(|| err(format!("unknown tree `{}`", spec.tree)))?;
        let mut systems = Vec::new();
        for name in &spec.systems {
            let id = (book.system_id(name)).ok_or_else(|| {
                err(format!(
                    "names a system `{name}` that is not in the list (data/talents.toml)"
                ))
            })?;
            systems.push(id);
        }
        let needs = resolve_needs(spec.needs, &book.ids, content).map_err(err)?;
        let (effects, effects_while) =
            split_effects(spec.effects, content, spec.choice).map_err(err)?;
        let resource = match &spec.resource {
            None => None,
            Some(cost) => {
                let resource = content.resource_id(&cost.id).ok_or_else(|| {
                    err(format!(
                        "costs a resource `{}` that is not in data/resources.toml",
                        cost.id
                    ))
                })?;
                Some(ResourceCost {
                    resource,
                    amount: cost.cost,
                })
            }
        };
        book.list.push(TalentDef {
            id: spec.id,
            name: spec.name,
            tree,
            choice: spec.choice,
            description: spec.description,
            needs,
            effects,
            effects_while,
            systems,
            granted: spec.granted,
            flaw: spec.flaw,
            resource,
            alignment: spec.alignment,
        });
    }
    book.mark_percent = (book.list.iter())
        .flat_map(|t| &t.effects)
        .filter_map(|e| match e {
            TalentEffect::Craft(CraftEffect::MakersMark { worth_percent }) => Some(*worth_percent),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    Ok(book)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentFiles;

    /// The built-in content with the talent table replaced by `settings` and `talents`.
    fn load(settings: &str, talents: &[(&str, &str)]) -> Result<Content, ContentError> {
        let mut files = ContentFiles::builtin();
        files.remove("data/talents.toml");
        files.remove("data/talents/");
        // Background stories name talents of the built-in set.
        files.remove("data/background.toml");
        files.insert("data/talents.toml", settings);
        for (name, text) in talents {
            files.insert(&format!("data/talents/{name}.toml"), text);
        }
        Content::load(&files)
    }

    const SETTINGS: &str = r#"
[[tree]]
id = "vigour"
name = "Vigour"
description = "Hit points."
[[tree]]
id = "shadow"
name = "Shadow"
description = "Stealth."
[[system]]
id = "hit_points"
name = "Hit points"
[[system]]
id = "stealth"
name = "Stealth"
"#;

    const HEALTHY: &str = r#"
[[talent]]
id = "healthy"
name = "Healthy"
tree = "vigour"
description = "Faster healing."
effects = [{ kind = "healing_percent", value = 25 }]
systems = ["hit_points"]
"#;

    const HARDY: &str = r#"
[[talent]]
id = "hardy"
name = "Hardy"
tree = "vigour"
description = "More hit points."
needs = { stats = { toughness = 17 }, talents = ["healthy"] }
effects = [{ kind = "max_hp_per_levels", per_levels = 4, at_least = 3 }]
systems = ["hit_points"]
"#;

    fn err(result: Result<Content, ContentError>) -> String {
        match result {
            Ok(_) => panic!("loaded"),
            Err(e) => e.to_string(),
        }
    }

    #[test]
    fn talents_load_with_needs_and_effects() {
        let content = load(SETTINGS, &[("healthy", HEALTHY), ("hardy", HARDY)]).expect("loads");
        let book = content.talents();
        assert_eq!(book.len(), 2);
        let hardy = book.get(book.id("hardy").expect("hardy"));
        assert_eq!(hardy.tree, book.tree_id("vigour").expect("tree"));
        let needs: Vec<String> = hardy.needs.iter().map(|n| n.words(&content)).collect();
        assert_eq!(
            needs,
            ["Healthy", "Toughness 17"],
            "talents first, then scores"
        );
        assert_eq!(hardy.effects.len(), 1);
        assert!(!hardy.is_free());
        assert!(book.get(book.id("healthy").expect("healthy")).is_free());
        assert_eq!(book.by_tree().count(), 2);
    }

    #[test]
    fn a_duplicate_id_names_both_files() {
        let e = err(load(
            SETTINGS,
            &[("healthy", HEALTHY), ("hardy", HARDY), ("twin", HEALTHY)],
        ));
        assert!(e.contains("duplicate talent id `healthy`"), "{e}");
        assert!(e.contains("data/talents/healthy.toml"), "{e}");
    }

    #[test]
    fn an_effect_under_a_status_is_kept_apart_and_checked() {
        let with = |effects: &str| {
            let text = format!(
                "[[talent]]\nid = \"x\"\nname = \"X\"\ntree = \"vigour\"\ndescription = \"d\"\n\
                 systems = [\"hit_points\"]\neffects = [{effects}]\n"
            );
            load(
                SETTINGS,
                &[("healthy", HEALTHY), ("hardy", HARDY), ("x", &text)],
            )
        };
        let content = with(
            "{ kind = \"dodge\", value = 1 }, { kind = \"dodge\", value = 2, while = \"raging\" }",
        )
        .expect("loads");
        let book = content.talents();
        let x = book.get(book.id("x").expect("x"));
        let raging = content.status_id("raging").expect("raging");
        assert_eq!(
            x.effects,
            vec![TalentEffect::Defence(DefenceEffect::Dodge { value: 1 })]
        );
        assert_eq!(
            x.effects_while,
            vec![(
                raging,
                TalentEffect::Defence(DefenceEffect::Dodge { value: 2 })
            )]
        );
        // Alone, it is something to do.
        with("{ kind = \"dodge\", value = 2, while = \"raging\" }").expect("loads");
        assert!(
            err(with("{ kind = \"dodge\", value = 2, while = \"sulking\" }"))
                .contains("unknown status `sulking`")
        );
        assert!(
            err(with("{ kind = \"dodge\", value = 2, whilst = \"raging\" }"))
                .contains("unknown field")
        );
        // An active is used, not summed: no condition on it.
        assert!(
            err(with(
                "{ kind = \"rage\", resource = \"rage\", while = \"raging\" }"
            ))
            .contains("only for an effect the talents' sum reads")
        );
    }

    #[test]
    fn a_song_names_a_song_and_what_it_costs() {
        let song = |fields: &str| {
            let text = format!(
                "[[talent]]\nid = \"x\"\nname = \"X\"\ntree = \"vigour\"\n\
                 description = \"d\"\nsystems = [\"hit_points\"]\n\
                 effects = [{{ kind = \"song\", status = \"inspired\", {fields} }}]\n"
            );
            load(
                SETTINGS,
                &[("healthy", HEALTHY), ("hardy", HARDY), ("x", &text)],
            )
        };
        let inspire = "id = \"inspire\", on = \"self\"";
        song(&format!("{inspire}, resource = \"inspiration\", cost = 3")).expect("a pool");
        song(&format!("{inspire}, mana = 6")).expect("mana");
        let refused = |fields: &str| err(song(fields));
        assert!(refused("id = \"rage\", on = \"self\", mana = 6").contains("no song"));
        assert!(
            refused(&format!("{inspire}, resource = \"nerve\", cost = 3"))
                .contains("unknown resource `nerve`")
        );
        song(
            "id = \"inspire\", on = \"hearers\", mana = 6, check = \"willpower\", difficulty = 13",
        )
        .expect("data-ready for the Lullaby");
    }

    #[test]
    fn bad_prerequisites_and_effects_are_refused() {
        let bad = |body: &str| {
            let text = format!(
                "[[talent]]\nid = \"x\"\nname = \"X\"\ntree = \"vigour\"\ndescription = \"d\"\n\
                 systems = [\"hit_points\"]\n{body}\n"
            );
            err(load(
                SETTINGS,
                &[("healthy", HEALTHY), ("hardy", HARDY), ("x", &text)],
            ))
        };
        // Alignment bars nothing: it is no prerequisite.
        assert!(
            bad("effects = [{ kind = \"dodge\", value = 1 }]\n\
                 needs = { requires_alignment = { min = \"good\" } }")
            .contains("unknown field")
        );
        // The spell hooks (the spell scaling design §7.1) filter by known tags.
        assert!(
            bad("effects = [{ kind = \"spell_mod\", rungs = 1, when = [\"sneezing\"] }]")
                .contains("no school, kind, shape, or damage type")
        );
        assert!(
            bad("effects = [{ kind = \"spell_mod\", rungs = 1, lift = 10 }]")
                .contains("unknown field")
        );
        assert!(
            bad("effects = [{ kind = \"dodge\", value = 1 }]\nneeds = { race = \"orc\" }")
                .contains("unknown race")
        );
        assert!(
            bad("effects = [{ kind = \"dodge\", value = 1 }]\nneeds = { eyes = \"xray\" }")
                .contains("unknown eyes")
        );
        assert!(
            bad("effects = [{ kind = \"weapon_ranks\", to = 1, skills = [\"x\"] }]")
                .contains("unknown weapon skill `x`")
        );
        assert!(
            bad("effects = [{ kind = \"skill\", value = 25 }]").contains("`choice = \"skill\"`")
        );
        // The Shadow and Wayfarer effects name what exists (#239).
        assert!(
            bad("effects = [{ kind = \"terrain_as\", terrains = [\"lava\", \"bog\"], like = \"floor\" }]")
                .contains("unknown terrain `bog`")
        );
        assert!(
            bad("effects = [{ kind = \"lantern_light\", light = \"sun\" }]")
                .contains("unknown light preset")
        );
        // The element gifts' (#268): Blaze names what exists.
        let blaze = |fields: &str| bad(&format!("effects = [{{ kind = \"blaze\", {fields} }}]"));
        let ok = "damage = \"2d6\", damage_type = \"fire\", spell = \"blaze\", \
                  fire = \"arrow_fire\", burns = 60, cooldown = \"burnt_out\"";
        let fields = ok.replace("\"arrow_fire\"", "\"sun\"");
        assert!(blaze(&format!("radius = 1, {fields}")).contains("unknown light preset"));
        let fields = ok.replace("\"burnt_out\"", "\"smouldering\"");
        assert!(blaze(&format!("radius = 1, {fields}")).contains("unknown status"));
        let fields = ok.replace("\"fire\",", "\"plasma\",");
        assert!(blaze(&format!("radius = 1, {fields}")).contains("unknown damage type"));
        assert!(
            bad("effects = [{ kind = \"scent\", status = \"sniffles\", radius = 4 }]")
                .contains("unknown status")
        );
        assert!(bad("effects = [{ kind = \"kick_die\", sides = 0 }]").contains("at least 1"));
        // Exploit's ladder (#1463): a die that is no die, a status that does not exist.
        assert!(!bad("effects = [{ kind = \"exploit_dice\", dice = \"lots\" }]").is_empty());
        assert!(
            bad("effects = [{ kind = \"exploit_status\", status = \"dazed\" }]")
                .contains("unknown status")
        );
        // The ladder's top (#1464): a threat or a sure critical of nothing.
        let threat = |fields: &str| {
            bad(&format!(
                "effects = [{{ kind = \"exploit_threat\", {fields} }}]"
            ))
        };
        assert!(threat("value = 0, sure_at = 5").contains("at least 1"));
        assert!(threat("value = 4, sure_at = 0").contains("at least 1"));
        let no_system = "[[talent]]\nid = \"x\"\nname = \"X\"\ntree = \"vigour\"\ndescription = \"d\"\n\
                         effects = [{ kind = \"dodge\", value = 1 }]\nsystems = [\"nope\"]\n";
        assert!(
            err(load(
                SETTINGS,
                &[("healthy", HEALTHY), ("hardy", HARDY), ("x", no_system)]
            ))
            .contains("not in the list")
        );
    }

    #[test]
    fn a_trade_names_its_skill_and_tools_and_the_races_it_is_closed_to() {
        let trade = |needs: &str, effect: &str| {
            let text = format!(
                "[[talent]]\nid = \"x\"\nname = \"X\"\ntree = \"vigour\"\ndescription = \"d\"\n\
                 systems = [\"hit_points\"]\nneeds = {{ {needs} }}\neffects = [{effect}]\n"
            );
            load(
                SETTINGS,
                &[("healthy", HEALTHY), ("hardy", HARDY), ("x", &text)],
            )
        };
        let smith = r#"{ kind = "trade", skill = "metalcraft", dice = "3d6", tools = ["piece*3", { wield = "pick_axe" }, "gem:garnet:uncut|gem:zircon:uncut"] }"#;
        let content = trade(r#"creation = true, race_lacks = "metalcraft""#, smith).expect("loads");
        let book = content.talents();
        let def = book.get(book.id("x").expect("x"));
        let metalcraft = content.skill_id("metalcraft").expect("metalcraft");
        assert!(def.needs.iter().any(|n| n == Need::RaceLacks(metalcraft)));
        assert!(def.effects[0].is_handed_down());
        // Its words name every race whose people all have the skill, in file order.
        assert_eq!(
            Need::RaceLacks(metalcraft).words(&content),
            "not to be a dwarf or a duergar"
        );
        // A skill no race teaches everyone asks nothing: the need is left out.
        let content = trade(r#"race_lacks = "woodcraft""#, smith).expect("loads");
        let book = content.talents();
        assert!(book.get(book.id("x").expect("x")).needs.is_empty());
        for (needs, effect, why) in [
            (
                r#"race_lacks = "smithing""#,
                smith,
                "unknown skill `smithing`",
            ),
            (
                "",
                r#"{ kind = "trade", skill = "smithing", dice = "3d6" }"#,
                "unknown skill",
            ),
            (
                "",
                r#"{ kind = "trade", skill = "mining", dice = "3d6", tools = ["spork"] }"#,
                "unknown object `spork`",
            ),
            (
                "",
                r#"{ kind = "trade", skill = "mining", dice = "3d6", tools = ["gem:garnet:shiny"] }"#,
                "no cut",
            ),
            (
                "",
                r#"{ kind = "trade", skill = "mining", dice = "3d6", while = "raging" }"#,
                "while",
            ),
        ] {
            let e = err(trade(needs, effect));
            assert!(e.contains(why), "{effect}: {e}");
        }
    }

    #[test]
    fn choices_list_their_options() {
        let focus = r#"
[[talent]]
id = "weapon_focus"
name = "Weapon focus"
tree = "vigour"
choice = "weapon_skill"
description = "d"
needs = { chosen = 2 }
effects = [{ kind = "melee_damage", value = 1 }]
systems = ["hit_points"]
"#;
        let content = load(SETTINGS, &[("healthy", HEALTHY), ("focus", focus)]).expect("loads");
        let options = Choice::WeaponSkill.options(&content);
        assert!(options.iter().any(|o| o == "long_blades"), "{options:?}");
        assert!(
            !options.iter().any(|o| o == "duelist"),
            "styles are no weapon skills"
        );
        assert!(
            Choice::Style
                .options(&content)
                .iter()
                .any(|o| o == "duelist")
        );
        assert_eq!(Choice::Element.name(&content, "fire"), Some("fire"));
        assert_eq!(Choice::Skill.name(&content, "stealth"), Some("Stealth"));
        assert!(
            Choice::Skill
                .options(&content)
                .iter()
                .any(|o| o == "appraise")
        );
        assert_eq!(Choice::Element.name(&content, "slashing"), None);
    }

    #[test]
    fn the_built_in_talents_load() {
        let content = Content::builtin().expect("content");
        assert!(
            content.talents().len() >= 12,
            "{} talents",
            content.talents().len()
        );
    }

    /// Every effect kind of [`TalentEffect`] is documented in `data/talents/README.toml`, the
    /// content's schema notes (#1535): a comment line `#   kind ...` names it. The kinds are
    /// read from the sources, each area's enum's variants in snake case as serde names them
    /// (#1495), and each area's `KINDS` are checked against them; together they are
    /// [`TalentEffect::kinds`], every kind once.
    #[test]
    fn readme_documents_every_effect_kind() {
        let readme = include_str!("../../../../assets/data/talents/README.toml");
        let variants = |source: &str, head: &str| -> Vec<String> {
            let start = source.find(head).expect("the enum");
            let body = &source[start..];
            let body = &body[..body.find("\n}\n").expect("the enum's end")];
            (body.lines())
                .filter_map(|l| l.strip_prefix("    "))
                .filter(|l| l.starts_with(|c: char| c.is_ascii_uppercase()))
                .map(|l| {
                    let name = l.split(|c: char| !c.is_ascii_alphanumeric()).next();
                    let mut snake = String::new();
                    for (i, c) in name.unwrap_or_default().chars().enumerate() {
                        if c.is_ascii_uppercase() && i > 0 {
                            snake.push('_');
                        }
                        snake.push(c.to_ascii_lowercase());
                    }
                    snake
                })
                .collect()
        };
        let areas: [(&str, &str, &[&str]); 19] = [
            (
                "ally",
                include_str!("../talent/ally.rs"),
                &AllyEffect::KINDS,
            ),
            (
                "beast",
                include_str!("../talent/beast.rs"),
                &BeastEffect::KINDS,
            ),
            (
                "craft",
                include_str!("../talent/craft.rs"),
                &CraftEffect::KINDS,
            ),
            (
                "defence",
                include_str!("../talent/defence.rs"),
                &DefenceEffect::KINDS,
            ),
            (
                "delving",
                include_str!("../talent/delving.rs"),
                &DelvingEffect::KINDS,
            ),
            (
                "faith",
                include_str!("../talent/faith.rs"),
                &FaithEffect::KINDS,
            ),
            (
                "fortune",
                include_str!("../talent/fortune.rs"),
                &FortuneEffect::KINDS,
            ),
            (
                "growth",
                include_str!("../talent/growth.rs"),
                &GrowthEffect::KINDS,
            ),
            (
                "hunger",
                include_str!("../talent/hunger.rs"),
                &HungerEffect::KINDS,
            ),
            (
                "magic",
                include_str!("../talent/magic.rs"),
                &MagicEffect::KINDS,
            ),
            (
                "martial",
                include_str!("../talent/martial.rs"),
                &MartialEffect::KINDS,
            ),
            (
                "movement",
                include_str!("../talent/movement.rs"),
                &MovementEffect::KINDS,
            ),
            (
                "offence",
                include_str!("../talent/offence.rs"),
                &OffenceEffect::KINDS,
            ),
            (
                "resource",
                include_str!("../talent/resource.rs"),
                &ResourceEffect::KINDS,
            ),
            (
                "senses",
                include_str!("../talent/senses.rs"),
                &SensesEffect::KINDS,
            ),
            (
                "sign",
                include_str!("../talent/sign.rs"),
                &SignEffect::KINDS,
            ),
            (
                "song",
                include_str!("../talent/song.rs"),
                &SongEffect::KINDS,
            ),
            (
                "stealth",
                include_str!("../talent/stealth.rs"),
                &StealthEffect::KINDS,
            ),
            (
                "vigour",
                include_str!("../talent/vigour.rs"),
                &VigourEffect::KINDS,
            ),
        ];
        let mut kinds: Vec<String> = Vec::new();
        for (area, source, listed) in areas {
            let head = format!(
                "pub enum {}{}Effect {{",
                area[..1].to_uppercase(),
                &area[1..]
            );
            let mut read = variants(source, &head);
            read.sort();
            assert_eq!(read, listed, "{area}: KINDS are its variants, sorted");
            kinds.extend(read);
        }
        let mut every: Vec<&str> = kinds.iter().map(String::as_str).collect();
        every.sort_unstable();
        assert_eq!(every, TalentEffect::kinds(), "every kind of one area");
        assert!(every.windows(2).all(|w| w[0] != w[1]), "every kind once");
        assert!(kinds.len() > 200, "{} kinds read", kinds.len());
        // An entry's head runs to the first double space: `kind { fields }`, or several kinds
        // that share their words, `pick_lock, steal, throw_voice`.
        let named = |kind: &str| {
            (readme.lines())
                .filter_map(|l| l.strip_prefix("#   "))
                .filter_map(|rest| rest.split("  ").next())
                .flat_map(|head| head.split(", "))
                .any(|head| head.split(' ').next() == Some(kind))
        };
        let missing: Vec<&String> = kinds.iter().filter(|k| !named(k)).collect();
        assert!(missing.is_empty(), "undocumented effect kinds: {missing:?}");
    }
}
