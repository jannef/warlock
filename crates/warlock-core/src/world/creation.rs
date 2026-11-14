//! Creation picks (spec §8 "Talents"): a character the player chose (a run that starts with its
//! starting kit: New run, `--character`) takes `[levels] creation_talents` talents at creation
//! (`data/experience.toml`), and its class's `creation_talents` more (the bard's one). They are
//! a choice due at level 1 ([`Due`]) for as long as the run is being made: [`World::start_run`]
//! opens it and spends it at once, on the picks the character names
//! ([`CharacterSpec::talents`], from the creation Talents step or `--talents`), else on the
//! **Random** picks, drawn from the stream `character/talents/<race>,<gender>,<class>` (a pure
//! function of the seed and the character), and records what was taken in the character, so a
//! save, a replay, and the summary all name them. Nothing is said and nothing waits: a run
//! begins with its picks made. A run on the scenario's own character (a dev launch, without the
//! kit) takes none, so walks recorded before creation picks keep their hashes.
//!
//! Character creation's Talents step reads a **drafted** run ([`World::draft`]): the run as it
//! would start, its picks still due, so prerequisites see exactly the character the run gets
//! (scores, skills, the class's level 1 abilities, the sign's gifts).

use super::talent::Due;
use super::{World, WorldError};
use crate::character::CharacterSpec;
use crate::content::{Content, TalentId};
use crate::rng::Rng;
use crate::talent;
use crate::text::article;
use std::sync::Arc;

/// A creation pick as written: `id` or `id:choice`.
fn split(pick: &str) -> (&str, Option<&str>) {
    match pick.split_once(':') {
        Some((id, choice)) => (id, Some(choice)),
        None => (pick, None),
    }
}

/// A creation pick written as [`CharacterSpec::talents`] holds it.
pub fn pick_text(id: &str, choice: Option<&str>) -> String {
    match choice {
        Some(choice) => format!("{id}:{choice}"),
        None => id.to_string(),
    }
}

/// The bonus picks a character of `spec` takes at creation, from its class's own trees (the
/// craftsperson's two trades: `creation_bonus`), besides [`creation_count`]'s.
pub fn creation_bonus(content: &Content, spec: &CharacterSpec) -> u8 {
    (content.class(&spec.class))
        .and_then(|c| c.creation_bonus.as_ref())
        .map_or(0, |b| b.count)
}

/// How many talents a character of `spec` picks at creation, its bonus picks
/// ([`creation_bonus`]) aside: everyone's, the class's more (the bard's), and the more its race's
/// granted talents give (the humans' Versatile).
pub fn creation_count(content: &Content, spec: &CharacterSpec) -> u8 {
    let class = content.class(&spec.class).map_or(0, |c| c.creation_talents);
    let mut granted = talent::Talents::default();
    for id in content.race(&spec.race).map_or(&[][..], |r| &r.talents[..]) {
        granted.take(id, None);
    }
    let race = (granted.mods(content, &Default::default())).extra_picks_at(1);
    (content.experience().creation_talents())
        .saturating_add(class)
        .saturating_add(race)
}

/// `--talents`: the character a run of `scenario` starts as when its creation picks are named,
/// `character` (else the scenario's own) with `talents` picked; `None` without a character.
pub fn chosen_with_talents(
    content: &Content,
    scenario: &str,
    character: Option<&CharacterSpec>,
    talents: &[String],
) -> Option<CharacterSpec> {
    let own = content
        .scenario(scenario)
        .and_then(|s| s.character.as_ref());
    (character.or(own)).map(|spec| spec.clone().with_talents(talents.to_vec()))
}

/// Parses `--talents`: picks separated by commas, `hardy,weapon_focus:long_blades`.
pub fn parse_talents(text: &str) -> Vec<String> {
    (text.split(',').map(str::trim))
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect()
}

impl World {
    /// The run of `scenario` with `seed` that character `spec` would start, its creation picks
    /// still due (character creation's Talents step reads it; nothing else should act on it).
    /// `spec`'s own picks are ignored.
    pub fn draft(
        content: Arc<Content>,
        scenario: &str,
        seed: Option<u64>,
        spec: &CharacterSpec,
    ) -> Result<Self, WorldError> {
        Self::begin(content, scenario, seed, Some(spec), None, true)
    }

    /// The creation picks still due on a drafted run ([`World::draft`]), after `picks` (each
    /// `id` or `id:choice`) are taken: how many are left, bonus picks included, and the talents
    /// that may fill the next, tree by tree, with the choices each may be taken for: while a
    /// bonus pick is left, those of the class's creation trees alone (the craftsperson's trades
    /// come first). An error (in words) when a pick of `picks` cannot be taken.
    #[allow(clippy::type_complexity)]
    pub fn creation_offers(
        &self,
        picks: &[String],
    ) -> Result<(u8, Vec<(TalentId, Vec<Option<String>>)>), String> {
        let world = self.after_picks(picks)?;
        let due = world.creation_due();
        let left = due.map_or(0, |d| d.talents + d.bonus);
        Ok((left, world.creation_choices()))
    }

    /// The bonus picks of the class's creation trees still due on a drafted run after `picks`
    /// (the craftsperson's trades left to choose; 0 for every other class).
    pub fn creation_bonus_left(&self, picks: &[String]) -> u8 {
        (self.after_picks(picks).ok())
            .and_then(|w| w.creation_due())
            .map_or(0, |d| d.bonus)
    }

    /// The drafted run with `picks` taken, or why one cannot be.
    fn after_picks(&self, picks: &[String]) -> Result<Self, String> {
        let mut world = self.clone();
        for pick in picks {
            let (id, choice) = split(pick);
            world.learn(id, choice)?;
        }
        Ok(world)
    }

    /// The talents that may fill the next creation pick, with their choices: the creation
    /// trees' while a bonus pick is due (any, should none of them fit), else any tree's; none
    /// when nothing is due.
    fn creation_choices(&self) -> Vec<(TalentId, Vec<Option<String>>)> {
        let Some(due) = self.creation_due() else {
            return Vec::new();
        };
        let bonus = match due.bonus {
            0 => Vec::new(),
            _ => self.bonus_offers_at(1),
        };
        match bonus.is_empty() {
            true => talent::offers(&self.content, &self.candidate()),
            false => bonus,
        }
    }

    /// The Random picks of a drafted run ([`World::draft`]): the talents its character takes
    /// when none are named, drawn one by one from the character's own stream among those it may
    /// take then (a talent first, then a choice of it).
    pub fn random_creation(&self) -> Vec<String> {
        let mut world = self.clone();
        world.take_random().unwrap_or_default()
    }

    /// The creation picks due, if any (level 1's talent and bonus picks).
    fn creation_due(&self) -> Option<Due> {
        (self.state.progress.due.first())
            .filter(|d| d.level == 1 && (d.talents > 0 || d.bonus > 0))
            .copied()
    }

    /// Opens the creation picks as a choice due at level 1 (a character the player chose).
    pub(super) fn open_creation(&mut self) {
        let Some(spec) = self.state.character.as_ref().map(|c| &c.spec) else {
            return;
        };
        let talents = creation_count(&self.content, spec);
        let bonus = creation_bonus(&self.content, spec);
        if talents > 0 || bonus > 0 {
            let due = Due {
                level: 1,
                talents,
                rolls: 0,
                bonus,
            };
            self.state.progress.due.insert(0, due);
            self.settle_due();
        }
    }

    /// Spends the creation picks: the ones the character names, else the scenario's own
    /// character's (`scenario`, its `talents`, none for another character) and then the Random
    /// picks for the rest, and records them in the character. Fails when a named pick cannot be
    /// taken, or when they are too few or too many.
    pub(super) fn take_creation(&mut self, scenario: &[String]) -> Result<(), WorldError> {
        let Some(spec) = self.state.character.as_ref().map(|c| c.spec.clone()) else {
            return Ok(());
        };
        let count = creation_count(&self.content, &spec) + creation_bonus(&self.content, &spec);
        let taken = if spec.talents.is_empty() {
            self.take_named(scenario)?;
            let mut taken = scenario.to_vec();
            taken.extend(self.take_random()?);
            taken
        } else {
            for pick in &spec.talents {
                let (id, choice) = split(pick);
                self.learn(id, choice).map_err(|why| {
                    let why = why.trim_end_matches('.');
                    WorldError::Character(format!(
                        "talent `{pick}`: {why} ({} {} takes {count} at creation)",
                        article(&spec.class),
                        spec.class
                    ))
                })?;
            }
            spec.talents.clone()
        };
        if let Some(due) = self.creation_due() {
            return Err(WorldError::Character(format!(
                "{} more talent(s) to pick: this character takes {count} at creation",
                due.talents + due.bonus
            )));
        }
        if let Some(character) = &mut self.state.character {
            character.spec.talents = taken;
        }
        Ok(())
    }

    /// A run of the scenario's own character without the starting kit (a dev launch, tests,
    /// golden walks): its creation picks are opened, the scenario's named `picks` taken, and
    /// the rest dropped (no Random picks: such a run takes nothing it is not told to).
    pub(super) fn take_scenario_picks(&mut self, picks: &[String]) -> Result<(), WorldError> {
        self.open_creation();
        self.take_named(picks)?;
        self.state.progress.due.retain(|d| d.level != 1);
        Ok(())
    }

    /// Takes the scenario's own character's named picks, in order; a pick that cannot be taken
    /// is an error naming the scenario.
    fn take_named(&mut self, picks: &[String]) -> Result<(), WorldError> {
        for pick in picks {
            let (id, choice) = split(pick);
            self.learn(id, choice).map_err(|why| {
                let why = why.trim_end_matches('.');
                let scenario = &self.state.scenario;
                WorldError::Character(format!("scenario `{scenario}`: talent `{pick}`: {why}"))
            })?;
        }
        Ok(())
    }

    /// Takes the Random picks while creation picks are due, returning them.
    fn take_random(&mut self) -> Result<Vec<String>, WorldError> {
        let Some(spec) = self.state.character.as_ref().map(|c| c.spec.clone()) else {
            return Ok(Vec::new());
        };
        let mut rng = Rng::stream(self.state.seed, &format!("character/talents/{spec}"));
        let mut taken = Vec::new();
        while self.creation_due().is_some() {
            let offers = self.creation_choices();
            if offers.is_empty() {
                break;
            }
            let (id, ways) = &offers[rng.index(offers.len())];
            let choice = ways[rng.index(ways.len())].clone();
            let id = self.content.talents().get(*id).id.clone();
            self.learn(&id, choice.as_deref())
                .map_err(WorldError::Character)?;
            self.settle_due();
            taken.push(pick_text(&id, choice.as_deref()));
        }
        Ok(taken)
    }
}
