//! Content's accessors for characters: races, names, classes and prestige classes, skills, talents,
//! experience, the horoscope, and weapon skills.

use super::{
    AlignmentRules, ClassActive, ClassDef, CombatStance, Content, Experience, Flavour, Horoscope,
    Lucky, Names, PrestigeDef, RaceDef, Races, Sign, SignActive, SkillDef, SpellData, SpellId,
    StanceDef, Stances, TalentBook, TalentDef, TalentEffect, WeaponAbility, WeaponAbilityId,
    WeaponSkillId, WeaponSkillTable,
};
use crate::skills::Ladder;
use crate::skills::RollRules;
use crate::skills::SkillId;

impl Content {
    /// The combat stances of `data/stances.toml` (all plain without the file).
    pub fn stances(&self) -> &Stances {
        &self.stances
    }

    /// What combat stance `stance` does.
    pub fn stance_def(&self, stance: CombatStance) -> &StanceDef {
        self.stances.get(stance)
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

    /// The classes of `data/classes/`, in the order offered (none without the files).
    pub fn classes(&self) -> &[ClassDef] {
        &self.classes
    }

    /// The class with id `id`.
    pub fn class(&self, id: &str) -> Option<&ClassDef> {
        self.classes.iter().find(|c| c.id == id)
    }

    /// The classes character creation offers a character of race `race` and `gender`, in the
    /// order offered: none hidden or refused to them (spec §8 "Character creation"); none for
    /// an unknown race.
    pub fn classes_offered(&self, race: &str, gender: crate::character::Gender) -> Vec<&ClassDef> {
        let Some(race) = self.race(race) else {
            return Vec::new();
        };
        (self.classes.iter())
            .filter(|c| c.offered_to(race, gender))
            .collect()
    }

    /// The prestige classes of `data/prestige/`, in table order (spec §8 "Prestige classes").
    pub fn prestiges(&self) -> &[PrestigeDef] {
        &self.prestige
    }

    /// The prestige class with id `id`.
    pub fn prestige(&self, id: &str) -> Option<&PrestigeDef> {
        self.prestige.iter().find(|p| p.id == id)
    }

    /// The birth sign's gift that is active `active`: the talent of the stars tree with its
    /// effect (its name and description are the abilities menu's; spec §8 "Birth sign").
    pub fn sign_active(&self, active: SignActive) -> Option<&TalentDef> {
        (self.talents.iter())
            .map(|(_, def)| def)
            .find(|def| def.effects.iter().any(|e| e.sign_active() == Some(active)))
    }

    /// The class ability that is active `active`: the talent of the class tree with its effect
    /// (shared by the classes that have it; its name and description are the menu's).
    pub fn class_active(&self, active: ClassActive) -> Option<&TalentDef> {
        (self.talents.iter())
            .map(|(_, def)| def)
            .find(|def| def.effects.iter().any(|e| e.active() == Some(active)))
    }

    /// What the warrior's Charge costs (`world/charge.rs`), as its talent says: the stamina
    /// every charge costs, and what each cell moved costs on top.
    pub fn charge_cost(&self) -> Option<(u16, u16)> {
        let def = self.class_active(ClassActive::Charge)?;
        let per_cell = def.effects.iter().find_map(|e| match e {
            TalentEffect::Charge { per_cell } => Some(*per_cell),
            _ => None,
        })?;
        Some((def.resource.map_or(0, |r| r.amount), per_cell))
    }

    /// How far Sic 'em reaches (`world/sic.rs`), as its talent says.
    pub fn sic_range(&self) -> Option<i32> {
        (self.class_active(ClassActive::Sic)?.effects.iter()).find_map(|e| match e {
            TalentEffect::Sic { range, .. } => Some(*range),
            _ => None,
        })
    }

    /// The caster's spell `spell` (spec §8 "Class abilities", "Spells"): the spell of its id in
    /// `data/spells/`, when a class ability gives it (its `spell` effect names it); `None`
    /// otherwise.
    pub fn class_spell(&self, spell: SpellId) -> Option<&SpellData> {
        self.spell_of(spell).filter(|_| self.class_gives(spell))
    }

    /// The spell `spell` names in `data/spells/`, whoever has it (a class ability, or study:
    /// spec §8 "Spells"); `None` without its file.
    pub fn spell_of(&self, spell: SpellId) -> Option<&SpellData> {
        self.spells.get(usize::from(spell.0))
    }

    /// Whether a class ability gives `spell` (its `spell` effect names it).
    pub(super) fn class_gives(&self, spell: SpellId) -> bool {
        let Some(name) = self.spell_of(spell).map(|s| s.id.as_str()) else {
            return false;
        };
        (self.talents.iter()).any(|(_, def)| {
            (def.effects.iter()).any(|e| matches!(e, TalentEffect::Spell { spell: s } if s == name))
        })
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

    /// The skills of `data/skills/`, in file order (by [`SkillId`]; none without the file).
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

    /// What weapon skill and style ranks teach (`data/weapon_abilities/`), in path order: a
    /// [`WeaponAbilityId`] is an index.
    pub fn weapon_abilities(&self) -> &[WeaponAbility] {
        &self.weapon_abilities
    }

    pub fn weapon_ability(&self, id: WeaponAbilityId) -> &WeaponAbility {
        &self.weapon_abilities[usize::from(id.0)]
    }

    pub fn weapon_ability_id(&self, name: &str) -> Option<WeaponAbilityId> {
        (self.weapon_abilities.iter().position(|a| a.id == name)).map(|i| WeaponAbilityId(i as u16))
    }

    /// The weapon abilities a rank of `rank` in weapon skill or style `skill` teaches, in path
    /// order: a lookup, no draw.
    pub fn weapon_abilities_for(
        &self,
        skill: WeaponSkillId,
        rank: u8,
    ) -> impl Iterator<Item = (WeaponAbilityId, &WeaponAbility)> {
        (self.weapon_abilities.iter().enumerate())
            .filter(move |(_, a)| rank >= a.rank && a.skills.contains(&skill))
            .map(|(i, a)| (WeaponAbilityId(i as u16), a))
    }

    /// The talents (`data/talents/`, its trees and systems too, spec §8 "Talents"): none without
    /// the files.
    pub fn talents(&self) -> &TalentBook {
        &self.talents
    }

    /// The skill ladder: the die a level-up roll throws by the skill's marks (`[ladder]` in
    /// `data/skills.toml`, spec §8 "Skills").
    pub fn ladder(&self) -> &Ladder {
        &self.skills.ladder
    }

    /// The skill rolls' rules: Intelligence's extra rolls, the marks a roll spends, the soft cap
    /// (`[rolls]` in `data/skills.toml`).
    pub fn skill_rolls(&self) -> &RollRules {
        &self.skills.rolls
    }

    /// The horoscope of `data/horoscope.toml` (none without the file).
    pub fn horoscope(&self) -> Option<&Horoscope> {
        self.horoscope.as_ref()
    }

    /// The horoscope's calendar and moon flavour (`[flavour]`), if it has the table.
    pub fn flavour(&self) -> Option<&Flavour> {
        self.horoscope.as_ref()?.flavour.as_ref()
    }

    /// The cells a full moon adds at `time` to the reaches of a birth sign's gifts (spec §8
    /// "Birth sign"): the flavour's `full_moon_reach` while one is up, else 0. The one read.
    pub fn full_moon_reach(&self, time: crate::time::GameTime) -> i32 {
        self.flavour().map_or(0, |f| f.reach_at(time, &self.moons))
    }

    /// The lucky day of those born under `sign` on `born` (spec §8 "Birth sign"), if it is
    /// one.
    pub fn lucky(&self, sign: &Sign, born: &crate::character::BirthDate) -> Option<&Lucky> {
        self.horoscope()?.lucky(sign, born.day)
    }

    /// The weapon skills (`data/weapon_skills.toml`; empty without the file).
    pub fn weapon_skills(&self) -> &WeaponSkillTable {
        &self.weapon_skills
    }
}
