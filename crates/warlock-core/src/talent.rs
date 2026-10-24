//! Talents an actor holds (spec §8 "Talents"): the runtime side of
//! [`crate::content::TalentBook`].
//!
//! [`Talents`] is what an actor has taken, saved by talent id (like affixes: adding talents to
//! the data keeps saves) with the rank reached and the choice made, on `Actor::talents`; the
//! player's are picked at level-ups, creatures' are empty until data gives them some.
//! [`TalentMods`] is the one aggregation every rule reads: the typed effects of every rank held,
//! summed, read where the rules already read what magic items do (`Actor::defence`, `melee`,
//! `hearing`, `World::healing_rate`, `burden::load`, `noise::noise`, `step_price`, hunger,
//! experience, searching, free blows). A new effect is a variant of
//! [`TalentEffect`], a field here, and one read.
//!
//! [`Candidate`] is everything a prerequisite may ask of whoever would take a talent, and
//! [`unmet`] / [`may_take`] decide a pick, for the rules and for what the player is shown alike.

use crate::character::Character;
use crate::content::{Choice, Content, Need, TalentDef, TalentEffect, TalentId};
use crate::skills::Skills;
use crate::stats::Stats;
use crate::weapon_skills::WeaponSkills;
use crate::world::{Actor, FreeBlows};
use serde::{Deserialize, Serialize};

/// A talent an actor has taken: by id, at this rank, for this choice (a talent with a `choice`
/// is taken once per choice, each with its own rank).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Taken {
    pub id: String,
    pub rank: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choice: Option<String>,
}

/// The talents an actor holds, in the order taken.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Talents(Vec<Taken>);

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

    /// The rank held of talent `id` taken for `choice` (0: not held).
    pub fn rank_of(&self, id: &str, choice: Option<&str>) -> u8 {
        (self.0.iter())
            .find(|t| t.id == id && t.choice.as_deref() == choice)
            .map_or(0, |t| t.rank)
    }

    /// Whether talent `id` is held at `rank` at least, for any choice.
    pub fn has(&self, id: &str, rank: u8) -> bool {
        self.0.iter().any(|t| t.id == id && t.rank >= rank)
    }

    /// Takes talent `id` for `choice`: one rank more. Returns the rank reached.
    pub(crate) fn take(&mut self, id: &str, choice: Option<&str>) -> u8 {
        if let Some(taken) =
            (self.0.iter_mut()).find(|t| t.id == id && t.choice.as_deref() == choice)
        {
            taken.rank += 1;
            return taken.rank;
        }
        self.0.push(Taken {
            id: id.to_string(),
            rank: 1,
            choice: choice.map(str::to_string),
        });
        1
    }

    /// What every rank held does, summed ([`TalentMods`]). Talents the content no longer has
    /// do nothing.
    pub fn mods(&self, content: &Content) -> TalentMods {
        let mut mods = TalentMods::default();
        if self.0.is_empty() {
            return mods;
        }
        let book = content.talents();
        for taken in &self.0 {
            let Some(id) = book.id(&taken.id) else {
                continue;
            };
            let def = book.get(id);
            for rank in 1..=taken.rank.min(def.rank_count()) {
                for effect in def.effects_at(rank) {
                    mods.add(effect);
                }
            }
        }
        mods
    }
}

/// What an actor's talents do, summed: one field per effect kind, read by one rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TalentMods {
    /// Defence in the Dodge category.
    pub dodge: i32,
    /// Natural armour, on top of the actor's own.
    pub natural: i32,
    /// Added to every melee blow's damage.
    pub melee_damage: i32,
    /// Hit points by level, one entry per rank held: `(per_levels, at_least)`
    /// ([`TalentMods::hp_at`]).
    pub hp_per_levels: Vec<(u8, i32)>,
    /// Percent faster hit points come back.
    pub healing_percent: i32,
    /// Percent louder footsteps (negative: quieter).
    pub footsteps_percent: i32,
    /// The doors it opens and closes make no noise.
    pub silent_doors: bool,
    /// How much fainter it hears (its hearing threshold, lower).
    pub hearing: i32,
    /// Percent more carrying limit.
    pub carry_percent: i32,
    /// Percent more time an overworld journey takes (negative: quicker).
    pub travel_percent: i32,
    /// Percent faster hunger burns (negative: slower).
    pub hunger_percent: i32,
    /// Exertion burns nothing.
    pub no_exertion: bool,
    /// Percent more experience.
    pub xp_percent: i32,
    /// It notices hidden things in passing, with this much added; `None`: only as its skills
    /// and race allow.
    pub notice: Option<i32>,
    /// The free blows its talents switch on.
    pub free_blows: FreeBlows,
}

impl TalentMods {
    /// Whether nothing is changed.
    pub fn is_empty(&self) -> bool {
        *self == TalentMods::default()
    }

    /// The effects of one rank, summed (what taking it adds).
    pub fn of(effects: impl IntoIterator<Item = TalentEffect>) -> Self {
        let mut mods = TalentMods::default();
        for effect in effects {
            mods.add(effect);
        }
        mods
    }

    fn add(&mut self, effect: TalentEffect) {
        match effect {
            TalentEffect::Dodge { value } => self.dodge += value,
            TalentEffect::Natural { value } => self.natural += value,
            TalentEffect::MeleeDamage { value } => self.melee_damage += value,
            TalentEffect::MaxHpPerLevels {
                per_levels,
                at_least,
            } => self.hp_per_levels.push((per_levels, at_least)),
            TalentEffect::HealingPercent { value } => self.healing_percent += value,
            TalentEffect::FootstepsPercent { value } => self.footsteps_percent += value,
            TalentEffect::SilentDoors => self.silent_doors = true,
            TalentEffect::Hearing { value } => self.hearing += value,
            TalentEffect::CarryPercent { value } => self.carry_percent += value,
            TalentEffect::TravelPercent { value } => self.travel_percent += value,
            TalentEffect::HungerPercent { value } => self.hunger_percent += value,
            TalentEffect::NoExertion => self.no_exertion = true,
            TalentEffect::XpPercent { value } => self.xp_percent += value,
            TalentEffect::Notice { value } => {
                self.notice = Some(self.notice.unwrap_or(0) + value);
            }
            TalentEffect::FreeBlow { trigger } => {
                use crate::content::FreeBlowTrigger as T;
                match trigger {
                    T::Cleave => self.free_blows.cleave = true,
                    T::GreatCleave => self.free_blows.great_cleave = true,
                    T::Riposte => self.free_blows.riposte = true,
                    T::Opportunist => self.free_blows.opportunist = true,
                }
            }
        }
    }

    /// The hit points the per-level effects give at character `level`: for each rank held,
    /// one per `per_levels` levels, at least `at_least`.
    pub fn hp_at(&self, level: u8) -> i32 {
        (self.hp_per_levels.iter())
            .map(|&(per, least)| (i32::from(level) / i32::from(per.max(1))).max(least))
            .sum()
    }

    /// `value` scaled by a signed percent, rounded down, never below 0: the limit at
    /// `carry_percent`, a gain at `xp_percent`.
    pub fn scaled(value: u64, percent: i32) -> u64 {
        let factor = u64::try_from((100 + percent).max(0)).unwrap_or(0);
        value.saturating_mul(factor) / 100
    }
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
            Need::Talent(id, rank) => {
                let other = content.talents().get(*id);
                // A chain of talents with a choice runs through the same choice.
                if other.choice.is_some() && other.choice == choice {
                    self.talents.rank_of(&other.id, chosen) >= *rank
                } else {
                    self.talents.has(&other.id, *rank)
                }
            }
            Need::Stat(stat, score) => self.stats.score(*stat) >= *score,
            Need::Skill(id, value) => self.skills.value(*id).is_some_and(|v| v >= *value),
            Need::WeaponSkill(id, rank) => self.weapon_skills.rank(*id) >= *rank,
            Need::Chosen(rank) => match (choice, chosen) {
                (Some(Choice::WeaponSkill | Choice::Style), Some(id)) => {
                    (content.weapon_skills().id(id))
                        .is_some_and(|w| self.weapon_skills.rank(w) >= *rank)
                }
                _ => false,
            },
            Need::Level(level) => self.level >= *level,
            Need::Race(id) => self.race == Some(id.as_str()),
            Need::Eyes(id) => self.eyes == Some(id.as_str()),
            Need::Class(id) => self.class == Some(id.as_str()),
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
    /// Every rank is held already.
    AtTop,
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
            Refusal::AtTop => format!("You already have all of {name}."),
            Refusal::Unmet(needs) => {
                let words: Vec<String> = needs.iter().map(|n| n.words(content)).collect();
                format!("You cannot take {name}: you need {}.", list(&words))
            }
        }
    }
}

/// `a`, `a and b`, `a, b, and c`.
fn list(words: &[String]) -> String {
    match words {
        [] => String::new(),
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}

/// The prerequisites of taking `rank` (1 first) of `def` for `chosen` that `who` does not
/// meet, in the order they are shown.
pub fn unmet(
    content: &Content,
    who: &Candidate<'_>,
    def: &TalentDef,
    rank: u8,
    chosen: Option<&str>,
) -> Vec<Need> {
    (def.needs_at(rank))
        .filter(|need| !who.meets(content, need, def.choice, chosen))
        .collect()
}

/// Whether `who` may take `def` for `chosen` now: the rank it would reach, or why not.
pub fn may_take(
    content: &Content,
    who: &Candidate<'_>,
    def: &TalentDef,
    chosen: Option<&str>,
) -> Result<u8, Refusal> {
    if def.granted {
        return Err(Refusal::Granted);
    }
    match (def.choice, chosen) {
        (Some(kind), None) => return Err(Refusal::NeedsChoice(kind)),
        (None, Some(_)) => return Err(Refusal::TakesNoChoice),
        (Some(kind), Some(id)) if kind.name(content, id).is_none() => {
            return Err(Refusal::BadChoice(kind, id.to_string()));
        }
        _ => {}
    }
    let rank = who.talents.rank_of(&def.id, chosen) + 1;
    if rank > def.rank_count() {
        return Err(Refusal::AtTop);
    }
    let missing = unmet(content, who, def, rank, chosen);
    if missing.is_empty() {
        Ok(rank)
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

    #[test]
    fn hit_points_by_level_grow_with_the_level_and_never_under_the_floor() {
        let mods = TalentMods::of([TalentEffect::MaxHpPerLevels {
            per_levels: 4,
            at_least: 3,
        }]);
        assert_eq!(mods.hp_at(1), 3);
        assert_eq!(mods.hp_at(12), 3);
        assert_eq!(mods.hp_at(16), 4);
        assert_eq!(mods.hp_at(60), 15);
        let two = TalentMods::of([
            TalentEffect::MaxHpPerLevels {
                per_levels: 4,
                at_least: 3,
            },
            TalentEffect::MaxHpPerLevels {
                per_levels: 4,
                at_least: 3,
            },
        ]);
        assert_eq!(two.hp_at(20), 10);
    }

    #[test]
    fn effects_sum_and_flags_switch() {
        let mods = TalentMods::of([
            TalentEffect::Dodge { value: 1 },
            TalentEffect::Dodge { value: 2 },
            TalentEffect::FootstepsPercent { value: -20 },
            TalentEffect::SilentDoors,
            TalentEffect::Notice { value: 2 },
            TalentEffect::FreeBlow {
                trigger: crate::content::FreeBlowTrigger::Riposte,
            },
        ]);
        assert_eq!(mods.dodge, 3);
        assert_eq!(mods.footsteps_percent, -20);
        assert!(mods.silent_doors);
        assert_eq!(mods.notice, Some(2));
        assert!(mods.free_blows.riposte && !mods.free_blows.cleave);
        assert!(!mods.is_empty());
        assert!(TalentMods::default().is_empty());
        assert_eq!(TalentMods::scaled(300, 10), 330);
        assert_eq!(TalentMods::scaled(300, -15), 255);
        assert_eq!(TalentMods::scaled(300, -150), 0);
    }

    #[test]
    fn taking_counts_ranks_per_choice() {
        let mut talents = Talents::default();
        assert_eq!(talents.take("hardy", None), 1);
        assert_eq!(talents.take("hardy", None), 2);
        assert_eq!(talents.take("weapon_focus", Some("bows")), 1);
        assert_eq!(talents.take("weapon_focus", Some("slings")), 1);
        assert_eq!(talents.rank_of("hardy", None), 2);
        assert_eq!(talents.rank_of("weapon_focus", Some("bows")), 1);
        assert_eq!(talents.rank_of("weapon_focus", None), 0);
        assert!(talents.has("weapon_focus", 1));
        assert!(!talents.has("hardy", 3));
        assert_eq!(talents.len(), 3);
        let json = serde_json::to_string(&talents).expect("json");
        assert_eq!(
            json,
            r#"[{"id":"hardy","rank":2},{"id":"weapon_focus","rank":1,"choice":"bows"},{"id":"weapon_focus","rank":1,"choice":"slings"}]"#
        );
        let back: Talents = serde_json::from_str(&json).expect("back");
        assert_eq!(back, talents);
    }

    #[test]
    fn lists_read_naturally() {
        let w = |s: &[&str]| list(&s.iter().map(|s| s.to_string()).collect::<Vec<_>>());
        assert_eq!(w(&["Strength 13"]), "Strength 13");
        assert_eq!(w(&["Strength 13", "Dodge"]), "Strength 13 and Dodge");
        assert_eq!(w(&["a", "b", "c"]), "a, b, and c");
    }
}
