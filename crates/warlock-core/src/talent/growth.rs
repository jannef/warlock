//! The growth talents (#1495): what a talent does to how the holder grows: experience, skill rolls,
//! talent picks, weapon ranks, and skills. [`GrowthEffect`] is the data (a talent's `[[effect]]` of
//! one of [`GrowthEffect::KINDS`]), [`GrowthMods`] the sum every rule reads (`TalentMods::growth`).
//! A new effect of the area is a variant here, its field and its arm in [`GrowthMods::add`] here,
//! its wiki text in `xtask/src/wiki/talents/growth.rs`, and its one read.

use crate::content::Content;
use crate::skills::SkillId;
use serde::Deserialize;

/// What a talent of the area does, one typed effect, read by one rule each.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GrowthEffect {
    /// Experience gained, this percent more.
    XpPercent { value: i32 },
    /// Skill rolls every level, this many more (Quick learner).
    SkillRolls { value: i32 },
    /// One more talent pick at character level `level` (1: at creation; Versatile).
    ExtraPick { level: u8 },
    /// Handed out once, as the talent is taken or given: every weapon skill and style of
    /// `skills` (none named: every row of `weapon_skills.toml` but the
    /// hidden styles, whatever the content holds) held below rank `to` rises to it (Travelled
    /// hands; `WeaponSkills::raise_to`). `news` is said when a class table gives it.
    WeaponRanks {
        to: u8,
        #[serde(default)]
        skills: Vec<String>,
        #[serde(default)]
        news: Option<String>,
    },
    /// Handed out once, as the talent is taken or given: every skill of `skills/` the holder
    /// is not trained in, but those `except` names, is trained at `dice`, rolled from the
    /// `levels` stream, whatever skills the content holds (Jack of all trades;
    /// `world/jack.rs`). `news` is said when a class table gives it.
    TakeUpSkills {
        dice: crate::dice::Dice,
        #[serde(default)]
        except: Vec<String>,
        #[serde(default)]
        news: Option<String>,
    },
    /// A skill counts this much higher wherever its value is read (an untrained one from 0):
    /// `skill`, or left out the chosen one (Skill focus; the talent takes a `skill` choice).
    Skill {
        #[serde(default)]
        skill: Option<String>,
        value: i32,
    },
    /// Every skill it is trained in counts this much higher (Jack of all trades).
    TrainedSkills { value: i32 },
}

impl GrowthEffect {
    /// The `kind`s of the data that are effects of the area, one a variant (`TalentEffect`'s
    /// deserializer hands these to [`GrowthEffect`]).
    pub const KINDS: [&'static str; 7] = [
        "extra_pick",
        "skill",
        "skill_rolls",
        "take_up_skills",
        "trained_skills",
        "weapon_ranks",
        "xp_percent",
    ];
}

/// What an actor's talents of the area do, summed: one field per effect kind, read by one
/// rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GrowthMods {
    /// Percent more experience.
    pub xp_percent: i32,
    /// More skill rolls every level.
    pub skill_rolls: i32,
    /// The character levels that bring one more talent pick, once for each (1: at creation).
    pub extra_picks: Vec<u8>,
    /// How much higher some skills count, by skill.
    pub skills: Vec<(SkillId, i32)>,
    /// How much higher every skill it is trained in counts (Jack of all trades).
    pub trained_skills: i32,
}

impl GrowthMods {
    /// Adds what `effect` does.
    pub(crate) fn add(&mut self, content: &Content, effect: GrowthEffect, choice: Option<&str>) {
        match effect {
            GrowthEffect::XpPercent { value } => self.xp_percent += value,
            GrowthEffect::SkillRolls { value } => self.skill_rolls += value,
            GrowthEffect::ExtraPick { level } => self.extra_picks.push(level),
            // Handed out as the talent is taken (`World::take_talent`), never read after.
            GrowthEffect::WeaponRanks { .. } | GrowthEffect::TakeUpSkills { .. } => {}
            GrowthEffect::Skill { skill, value } => {
                if let Some(id) = skill
                    .as_deref()
                    .or(choice)
                    .and_then(|s| content.skill_id(s))
                {
                    match self.skills.iter_mut().find(|(s, _)| *s == id) {
                        Some((_, v)) => *v += value,
                        None => self.skills.push((id, value)),
                    }
                }
            }
            GrowthEffect::TrainedSkills { value } => self.trained_skills += value,
        }
    }

    /// The talent picks more that character `level` brings (Versatile's; 1: at creation).
    pub fn extra_picks_at(&self, level: u8) -> u8 {
        let n = self.extra_picks.iter().filter(|&&l| l == level).count();
        u8::try_from(n).unwrap_or(u8::MAX)
    }
}
