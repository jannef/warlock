//! Races (`assets/data/races.toml`, spec §8 "Character creation"): the first choice of a new
//! run. A race adjusts the starting scores and the potentials (fixed numbers or dice), gives the
//! player their eyes, sets the range of their starting age, gives the player their size
//! (spec §8 "Burden": how much they can carry), whether being Burdened costs them nothing, and
//! how much they need to eat (spec §8 "Hunger"), how much experience their levels need (spec §8
//! "Experience"), trains them in skills (spec §8 "Skills"; dice,
//! and humans' extra class skill), gives them starting ranks in weapon skills (spec §8
//! "Weapon skills"; fixed numbers or dice), says what they notice of hidden features in
//! passing (spec §8 "Searching"), and may give them natural armour (spec §8 "Combat"),
//! resistances (spec §8 "Damage types"), eyes that strong light dazzles ([`LightBlind`]), a
//! bonus on disease checks (spec §8 "Diseases"), and a stomach for rotten food (spec §8
//! "Hunger").

use super::weapon_skill::Ranked;
use super::{
    ContentError, DamageTypeId, FACTORS, Hide, PassiveSearch, WeaponSkillId, WeaponSkillTable,
};
use crate::combat::MAX_BASE_HP;
use crate::dice::Dice;
use crate::light::Light;
use crate::perception::VIEW_RADIUS;
use crate::skills::{self, GRANTS, SkillId};
use crate::stats::{PerStat, SCORES, Stat, adjustments};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::ops::RangeInclusive;

/// How much a race may need to eat, in percent of a human: a tenth to ten times as much.
pub const HUNGER: RangeInclusive<u16> = 10..=1000;

fn normal_hunger() -> u16 {
    100
}

fn normal_experience() -> u16 {
    100
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RaceFile {
    base_potential: u8,
    race: Vec<RaceSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RaceSpec {
    id: String,
    name: String,
    description: String,
    vision: String,
    age: [u32; 2],
    #[serde(default)]
    stats: BTreeMap<Stat, Dice>,
    #[serde(default)]
    potential: BTreeMap<Stat, Dice>,
    #[serde(default)]
    hp: u16,
    natural: Option<Hide>,
    #[serde(default)]
    resist: BTreeMap<String, i32>,
    light_blind: Option<LightBlind>,
    #[serde(default)]
    disease_check: i32,
    #[serde(default)]
    eats_rotten: bool,
    #[serde(default)]
    alignment: i32,
    size: Option<String>,
    #[serde(default)]
    ignores_burdened: bool,
    passive_search: Option<PassiveSearch>,
    #[serde(default)]
    search: i32,
    #[serde(default)]
    skills: BTreeMap<String, Dice>,
    #[serde(default)]
    class_skill: Dice,
    #[serde(default = "normal_hunger")]
    hunger: u16,
    #[serde(default = "normal_experience")]
    experience: u16,
    #[serde(default)]
    weapon_skills: BTreeMap<String, Dice>,
}

/// The oldest a character may start, in years.
pub const MAX_AGE: u32 = 1000;

/// What a race may add to searching and noticing (spec §8 "Searching").
pub const SEARCH_BONUSES: RangeInclusive<i32> = -10..=10;

/// What a race may add to its checks against diseases (spec §8 "Diseases").
pub const DISEASE_BONUSES: RangeInclusive<i32> = -10..=10;

/// Eyes that strong light dazzles (spec §8 "Character creation"; drow): `light_blind = { glare
/// = 250, sight = 10, to_hit = -1 }`. While the light on the player's own cell, as their eyes
/// perceive it, is at least `glare` bright (the sun well up, a bright fire close by), they see no
/// farther than `sight` cells and their attacks are `to_hit` worse.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LightBlind {
    /// The perceived brightness ([`Light::brightness`]) from which light dazzles.
    pub glare: u16,
    /// How far a dazzled character sees, in cells.
    pub sight: i32,
    /// What being dazzled adds to every attack's to-hit (0 or less).
    pub to_hit: i32,
}

impl LightBlind {
    /// Whether light `light` on the character's cell, as their eyes perceive it, dazzles them.
    /// The one rule of light-blindness: perception reads it for sight, attacks for to-hit.
    pub fn dazzled_by(&self, light: Light) -> bool {
        light.brightness() >= self.glare
    }

    fn check(&self) -> Result<(), String> {
        if self.glare == 0
            || !(1..=VIEW_RADIUS).contains(&self.sight)
            || !(-20..=0).contains(&self.to_hit)
        {
            return Err(format!(
                "light_blind: `glare` must be at least 1, `sight` 1 to {VIEW_RADIUS}, and \
                 `to_hit` -20 to 0"
            ));
        }
        Ok(())
    }
}

/// A race, in the order offered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RaceDef {
    pub id: String,
    pub name: String,
    /// One line, shown when choosing.
    pub description: String,
    /// The race's eyes: a profile id in `data/vision.toml`.
    pub vision: String,
    /// The starting age in years: a character is this old, uniformly at random, when a run
    /// starts.
    pub age: RangeInclusive<u32>,
    /// Adjustments to the starting scores.
    pub stats: PerStat<Dice>,
    /// Adjustments to the potentials on [`Races::base_potential`]; hidden from the player.
    pub potential: PerStat<Dice>,
    /// What the race adds to the hit points (spec §8 "Combat"): maximum hit points are
    /// Toughness plus the race's and the class's.
    pub hp: i32,
    /// Its natural armour (spec §8 "Combat"), as a creature type's: Defence in the Natural
    /// category and Protection, added to what the player wears. Lizardfolk's scales.
    pub natural: Option<Hide>,
    /// What it resists, in percent per damage type, in damage type order (spec §8 "Damage
    /// types"), as a creature type's `resist`: added to what the player wears. Drow and poison.
    pub resist: Vec<(DamageTypeId, i32)>,
    /// Whether strong light dazzles the player, and what that costs them ([`LightBlind`]).
    pub light_blind: Option<LightBlind>,
    /// Points added to the player's checks against diseases (spec §8 "Diseases"): resisting
    /// one they are exposed to, and each progression. Ratfolk.
    pub disease_check: i32,
    /// Whether the player eats rotten food without harm (spec §8 "Hunger"): it feeds them, no
    /// more, as plain food would, and never sickens them or carries a disease. Ratfolk.
    pub eats_rotten: bool,
    /// Its part of a new character's alignment score (spec §8 "Alignment"), added to the
    /// class's.
    pub alignment: i32,
    /// Its size (spec §8 "Burden"): an index into [`super::Content::sizes`], which scales how
    /// much the player can carry.
    pub size: usize,
    /// Whether the player suffers nothing for being Burdened, the first level of burden (no
    /// slowdown, no cap on Dexterity in Defence, no louder steps); from Strained on they are
    /// treated like everyone else. Dwarves.
    pub ignores_burdened: bool,
    /// Which hidden features the player notices in passing, untrained (spec §8 "Searching"):
    /// elves everything, dwarves what is on stone; `None`: only with the Spot skill trained.
    pub passive_search: Option<PassiveSearch>,
    /// Points added to searching and to noticing in passing alike (spec §8 "Searching").
    pub search: i32,
    /// The skills it trains (spec §8 "Skills"), each by dice rolled when the character is made,
    /// in skill order; added to the class's.
    pub skills: Vec<(SkillId, Dice)>,
    /// Dice added to one of the class's skills, picked when the character is made (0: none).
    /// Humans: one extra class skill.
    pub class_skill: Dice,
    /// How much the player needs to eat, in percent of a human (spec §8 "Hunger"): game time
    /// burns their nutrition this much faster (a halfling, 125) or slower (an elf, 80).
    pub hunger: u16,
    /// How much experience a level needs, in percent (spec §8 "Experience"): multiplied with
    /// the class's onto every threshold. Long-lived races pay more (an elf, 140).
    pub experience: u16,
    /// Starting ranks in weapon skills (spec §8 "Weapon skills"), in skill order: added to the
    /// class's.
    pub weapon_skills: Vec<(WeaponSkillId, Dice)>,
}

/// Every race, and the base every potential starts from.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Races {
    /// What every potential is before the race adjusts it.
    pub base_potential: u8,
    /// In file order: the order offered.
    pub list: Vec<RaceDef>,
}

impl Races {
    /// Parses `data/races.toml`; `has_vision` says whether a vision profile id exists,
    /// `size_index` where a size id is in [`super::Content::sizes`], `skill_id` which skill a
    /// skill id is, `weapon_table` the weapon skills starting ranks may name, and `resist`
    /// resolves a `resist` table (damage types and percents, checked).
    pub(super) fn parse(
        file: &str,
        text: &str,
        has_vision: impl Fn(&str) -> bool,
        size_index: impl Fn(&str) -> Option<usize>,
        skill_id: impl Fn(&str) -> Option<SkillId>,
        weapon_table: &WeaponSkillTable,
        resist: impl Fn(&BTreeMap<String, i32>) -> Result<Vec<(DamageTypeId, i32)>, String>,
    ) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: RaceFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if !SCORES.contains(&parsed.base_potential) {
            return Err(err(format!(
                "base_potential {} is out of range ({} to {})",
                parsed.base_potential,
                SCORES.start(),
                SCORES.end()
            )));
        }
        if parsed.race.is_empty() {
            return Err(err("at least one race is required".into()));
        }
        let mut list: Vec<RaceDef> = Vec::new();
        for spec in parsed.race {
            let id = &spec.id;
            if list.iter().any(|r| &r.id == id) {
                return Err(err(format!("duplicate race id `{id}`")));
            }
            if !has_vision(&spec.vision) {
                return Err(err(format!(
                    "race `{id}`: unknown vision profile `{}`",
                    spec.vision
                )));
            }
            let [young, old] = spec.age;
            if young == 0 || young > old || old > MAX_AGE {
                return Err(err(format!(
                    "race `{id}`: age [{young}, {old}] must be [youngest, oldest] within 1 to \
                     {MAX_AGE}"
                )));
            }
            let stats =
                adjustments(&spec.stats).map_err(|m| err(format!("race `{id}`: stats: {m}")))?;
            let potential = adjustments(&spec.potential)
                .map_err(|m| err(format!("race `{id}`: potential: {m}")))?;
            if spec.hp > MAX_BASE_HP {
                return Err(err(format!(
                    "race `{id}`: hp {} is more than {MAX_BASE_HP}",
                    spec.hp
                )));
            }
            if let Some(hide) = &spec.natural {
                hide.check().map_err(|m| err(format!("race `{id}`: {m}")))?;
            }
            let resist = resist(&spec.resist).map_err(|m| err(format!("race `{id}`: {m}")))?;
            if let Some(blind) = &spec.light_blind {
                blind
                    .check()
                    .map_err(|m| err(format!("race `{id}`: {m}")))?;
            }
            let alignment = crate::alignment::check_start(spec.alignment)
                .map_err(|m| err(format!("race `{id}`: {m}")))?;
            let size_id = spec.size.as_deref().unwrap_or(super::MEDIUM);
            let size = size_index(size_id)
                .ok_or_else(|| err(format!("race `{id}`: unknown size `{size_id}`")))?;
            let skills = skills::grants(&spec.skills, &skill_id)
                .map_err(|m| err(format!("race `{id}`: skills: {m}")))?;
            let extra = spec.class_skill;
            if !GRANTS.contains(&extra.min()) || !GRANTS.contains(&extra.max()) {
                return Err(err(format!(
                    "race `{id}`: class_skill {extra} is out of range ({} to {})",
                    GRANTS.start(),
                    GRANTS.end()
                )));
            }
            if !HUNGER.contains(&spec.hunger) {
                return Err(err(format!(
                    "race `{id}`: hunger {} is out of range ({} to {} percent)",
                    spec.hunger,
                    HUNGER.start(),
                    HUNGER.end()
                )));
            }
            if !FACTORS.contains(&spec.experience) {
                return Err(err(format!(
                    "race `{id}`: experience {} is out of range ({} to {} percent)",
                    spec.experience,
                    FACTORS.start(),
                    FACTORS.end()
                )));
            }
            if !DISEASE_BONUSES.contains(&spec.disease_check) {
                return Err(err(format!(
                    "race `{id}`: disease_check {} is out of range ({} to {})",
                    spec.disease_check,
                    DISEASE_BONUSES.start(),
                    DISEASE_BONUSES.end()
                )));
            }
            if !SEARCH_BONUSES.contains(&spec.search) {
                return Err(err(format!(
                    "race `{id}`: search {} is out of range ({} to {})",
                    spec.search,
                    SEARCH_BONUSES.start(),
                    SEARCH_BONUSES.end()
                )));
            }
            let weapon_skills = (weapon_table.starting_ranks(&spec.weapon_skills, Ranked::Weapons))
                .map_err(|m| err(format!("race `{id}`: {m}")))?;
            list.push(RaceDef {
                id: spec.id,
                name: spec.name,
                description: spec.description,
                vision: spec.vision,
                age: young..=old,
                stats,
                potential,
                hp: i32::from(spec.hp),
                natural: spec.natural,
                resist,
                light_blind: spec.light_blind,
                disease_check: spec.disease_check,
                eats_rotten: spec.eats_rotten,
                alignment,
                size,
                ignores_burdened: spec.ignores_burdened,
                passive_search: spec.passive_search,
                search: spec.search,
                skills,
                class_skill: extra,
                hunger: spec.hunger,
                experience: spec.experience,
                weapon_skills,
            });
        }
        Ok(Self {
            base_potential: parsed.base_potential,
            list,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content() -> super::super::Content {
        super::super::Content::builtin().expect("content")
    }

    fn parse(text: &str) -> Result<Races, ContentError> {
        let sizes = ["small", "medium"];
        let skills = ["listen", "spot"];
        let content = content();
        Races::parse(
            "races.toml",
            text,
            |v| v == "normal",
            |s| sizes.iter().position(|id| *id == s),
            |k| (skills.iter().position(|id| *id == k)).map(|i| SkillId(i as u8)),
            content.weapon_skills(),
            |m| content.resolve_per_kind("resist", m, -100..=100, false),
        )
    }

    const ONE: &str = "base_potential = 15\n[[race]]\nid = \"human\"\nname = \"Human\"\n\
                       description = \"Everywhere.\"\nvision = \"normal\"\n\
                       age = [16, 35]\n";

    #[test]
    fn races_read_adjustments_that_default_to_zero() {
        let races = parse(&format!(
            "{ONE}stats = {{ strength = 2, toughness = \"1d2+1\" }}\n\
             potential = {{ presence = -1, willpower = \"1d2\" }}\n"
        ))
        .expect("valid");
        assert_eq!(races.base_potential, 15);
        let human = &races.list[0];
        assert_eq!(human.age, 16..=35);
        assert_eq!(human.stats[Stat::Strength], Dice::fixed(2));
        assert_eq!(human.stats[Stat::Toughness].to_string(), "1d2+1");
        assert!(human.stats[Stat::Dexterity].is_zero());
        assert_eq!(human.potential[Stat::Presence], Dice::fixed(-1));
        assert_eq!(human.potential[Stat::Willpower].to_string(), "1d2");
        assert_eq!(human.size, 1, "medium unless named");
        assert!(!human.ignores_burdened);
        assert!(human.skills.is_empty() && human.class_skill.is_zero());
        assert_eq!(human.hunger, 100, "a human's hunger unless named");
        assert_eq!(human.experience, 100, "a human's experience unless named");
        assert_eq!(human.natural, None, "no hide unless named");
        let scaled = parse(&format!(
            "{ONE}natural = {{ protection = 1, name = \"scales\" }}\n"
        ))
        .expect("valid");
        let hide = scaled.list[0].natural.as_ref().expect("a hide");
        assert_eq!((hide.defence, hide.protection), (0, 1));
        assert_eq!(hide.name, "scales");
        assert!(human.resist.is_empty(), "nothing resisted unless named");
        assert_eq!(
            human.light_blind, None,
            "light does not dazzle unless named"
        );
        assert_eq!(human.disease_check, 0, "no disease bonus unless named");
        let ratfolk = parse(&format!("{ONE}disease_check = 4\neats_rotten = true\n"));
        let ratfolk = &ratfolk.expect("valid").list[0];
        assert_eq!(ratfolk.disease_check, 4);
        assert!(ratfolk.eats_rotten && !human.eats_rotten);
        let drow = parse(&format!(
            "{ONE}light_blind = {{ glare = 250, sight = 10, to_hit = -1 }}\n"
        ))
        .expect("valid");
        let blind = drow.list[0].light_blind.expect("light-blind");
        assert_eq!((blind.glare, blind.sight, blind.to_hit), (250, 10, -1));
        assert!(blind.dazzled_by(Light::new(250, 250, 250, 0)));
        assert!(!blind.dazzled_by(Light::new(249, 249, 249, 0)));
        assert!(
            blind.dazzled_by(Light::new(0, 0, 0, 250)),
            "UV, if the eyes see it"
        );
        let drow = parse(&format!("{ONE}resist = {{ poison = 25, fire = -10 }}\n")).expect("valid");
        let kinds = |r: &RaceDef| -> Vec<(String, i32)> {
            (r.resist.iter())
                .map(|&(k, p)| (content().damage_type(k).id.clone(), p))
                .collect()
        };
        assert_eq!(
            kinds(&drow.list[0]),
            [("fire".to_string(), -10), ("poison".to_string(), 25)],
            "in damage type order"
        );
        let dwarf = parse(&format!(
            "{ONE}size = \"small\"\nignores_burdened = true\nhunger = 110\nexperience = 115\n"
        ))
        .expect("valid");
        assert_eq!(dwarf.list[0].size, 0);
        assert!(dwarf.list[0].ignores_burdened);
        assert_eq!(dwarf.list[0].hunger, 110);
        assert_eq!(dwarf.list[0].experience, 115);
        let elf = parse(&format!(
            "{ONE}skills = {{ spot = \"1d6+4\", listen = 3 }}\nclass_skill = \"1d4\"\n"
        ))
        .expect("valid");
        let elf = &elf.list[0];
        assert_eq!(
            elf.skills,
            [
                (SkillId(0), Dice::fixed(3)),
                (SkillId(1), "1d6+4".parse().expect("dice"))
            ],
            "in skill order"
        );
        assert_eq!(elf.class_skill.to_string(), "1d4");
        assert!(human.weapon_skills.is_empty());
        let elf = parse(&format!(
            "{ONE}weapon_skills = {{ long_blades = 1, bows = \"1d2\" }}\n"
        ))
        .expect("valid");
        let ranks: Vec<String> = (elf.list[0].weapon_skills.iter())
            .map(|(id, dice)| format!("{} {dice}", id.0))
            .collect();
        // In skill order: long blades (2) before bows (10).
        assert_eq!(ranks, ["2 +1", "10 1d2"]);
    }

    #[test]
    fn bad_races_are_refused() {
        for (text, why) in [
            (
                ONE.replace("normal", "x-ray"),
                "unknown vision profile `x-ray`",
            ),
            (
                format!("{ONE}stats = {{ luck = 1 }}\n"),
                "unknown stat `luck`",
            ),
            (
                format!("{ONE}stats = {{ strength = 11 }}\n"),
                "strength +11 is out of range",
            ),
            (
                format!("{ONE}stats = {{ strength = \"2d6\" }}\n"),
                "strength 2d6 is out of range",
            ),
            (
                format!("{ONE}stats = {{ strength = \"2x\" }}\n"),
                "bad dice `2x`",
            ),
            (ONE.replace("[16, 35]", "[35, 16]"), "age [35, 16]"),
            (ONE.replace("[16, 35]", "[0, 16]"), "age [0, 16]"),
            (ONE.replace("[16, 35]", "[16, 1001]"), "within 1 to 1000"),
            (ONE.replace("age = [16, 35]\n", ""), "missing field `age`"),
            (
                format!("{ONE}potential = {{ toughness = -12 }}\n"),
                "potential: toughness",
            ),
            (ONE.replace("15", "0"), "base_potential 0"),
            (format!("{ONE}size = \"huge\"\n"), "unknown size `huge`"),
            (format!("{ONE}hunger = 5\n"), "hunger 5 is out of range"),
            (
                format!("{ONE}experience = 49\n"),
                "experience 49 is out of range",
            ),
            (
                format!("{ONE}experience = 201\n"),
                "experience 201 is out of range",
            ),
            (
                format!("{ONE}skills = {{ luck = 5 }}\n"),
                "race `human`: skills: unknown skill `luck`",
            ),
            (
                format!("{ONE}skills = {{ spot = \"1d6-3\" }}\n"),
                "spot 1d6-3 is out of range",
            ),
            (
                format!("{ONE}class_skill = \"1d100+5\"\n"),
                "class_skill 1d100+5 is out of range",
            ),
            (
                format!("{ONE}weapon_skills = {{ whips = 1 }}\n"),
                "race `human`: unknown weapon skill `whips`",
            ),
            (
                format!("{ONE}weapon_skills = {{ bows = \"1d12\" }}\n"),
                "must give a rank from 0 to 10",
            ),
            (
                format!("{ONE}weapon_skills = {{ bows = -1 }}\n"),
                "must give a rank from 0 to 10",
            ),
            (
                format!("{ONE}{}", ONE.replace("base_potential = 15\n", "")),
                "duplicate race id `human`",
            ),
            (
                "base_potential = 15\nrace = []\n".into(),
                "at least one race",
            ),
            (
                format!("{ONE}natural = {{ protection = 0, name = \"skin\" }}\n"),
                "race `human`: natural: `defence` and `protection` must be 0 to 20, not both 0",
            ),
            (
                format!("{ONE}natural = {{ protection = 21, name = \"skin\" }}\n"),
                "must be 0 to 20",
            ),
            (
                format!("{ONE}natural = {{ protection = 1 }}\n"),
                "missing field `name`",
            ),
            (
                format!("{ONE}resist = {{ luck = 25 }}\n"),
                "race `human`: `resist`: unknown damage type `luck`",
            ),
            (
                format!("{ONE}resist = {{ poison = 101 }}\n"),
                "`resist`: `poison` must be -100 to 100",
            ),
            (
                format!("{ONE}light_blind = {{ glare = 0, sight = 10, to_hit = -1 }}\n"),
                "race `human`: light_blind: `glare` must be at least 1",
            ),
            (
                format!("{ONE}light_blind = {{ glare = 250, sight = 41, to_hit = -1 }}\n"),
                "`sight` 1 to 40",
            ),
            (
                format!("{ONE}light_blind = {{ glare = 250, sight = 10, to_hit = 1 }}\n"),
                "`to_hit` -20 to 0",
            ),
            (
                format!("{ONE}light_blind = {{ glare = 250, sight = 10 }}\n"),
                "missing field `to_hit`",
            ),
            (
                format!("{ONE}disease_check = 11\n"),
                "race `human`: disease_check 11 is out of range (-10 to 10)",
            ),
        ] {
            let e = parse(&text).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
    }
}
