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

/// How many talents a character of `spec` picks at creation: everyone's, the class's more (the
/// bard's), and the more its race's granted talents give (the humans' Versatile).
pub fn creation_count(content: &Content, spec: &CharacterSpec) -> u8 {
    let class = content.class(&spec.class).map_or(0, |c| c.creation_talents);
    let mut granted = talent::Talents::default();
    for id in content.race(&spec.race).map_or(&[][..], |r| &r.talents[..]) {
        granted.take(id, None);
    }
    let race = granted.mods(content).extra_picks_at(1);
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
    /// `id` or `id:choice`) are taken: how many are left, and the talents that may fill the next,
    /// tree by tree, with the choices each may be taken for. An error (in words) when a pick of
    /// `picks` cannot be taken.
    pub fn creation_offers(
        &self,
        picks: &[String],
    ) -> Result<(u8, Vec<(TalentId, Vec<Option<String>>)>), String> {
        let mut world = self.clone();
        for pick in picks {
            let (id, choice) = split(pick);
            world.learn(id, choice)?;
        }
        let left = world.creation_due().map_or(0, |d| d.talents);
        let offers = match left {
            0 => Vec::new(),
            _ => talent::offers(&world.content, &world.candidate()),
        };
        Ok((left, offers))
    }

    /// The Random picks of a drafted run ([`World::draft`]): the talents its character takes
    /// when none are named, drawn one by one from the character's own stream among those it may
    /// take then (a talent first, then a choice of it).
    pub fn random_creation(&self) -> Vec<String> {
        let mut world = self.clone();
        world.take_random().unwrap_or_default()
    }

    /// The creation picks due, if any (level 1's talent picks).
    fn creation_due(&self) -> Option<Due> {
        (self.state.progress.due.first())
            .filter(|d| d.level == 1 && d.talents > 0)
            .copied()
    }

    /// Opens the creation picks as a choice due at level 1 (a character the player chose).
    pub(super) fn open_creation(&mut self) {
        let Some(spec) = self.state.character.as_ref().map(|c| &c.spec) else {
            return;
        };
        let talents = creation_count(&self.content, spec);
        if talents > 0 {
            let due = Due {
                level: 1,
                talents,
                rolls: 0,
                bonus: 0,
            };
            self.state.progress.due.insert(0, due);
            self.settle_due();
        }
    }

    /// Spends the creation picks: the ones the character names, else the Random picks, and
    /// records them in the character. Fails when a named pick cannot be taken, or when they are
    /// too few or too many.
    pub(super) fn take_creation(&mut self) -> Result<(), WorldError> {
        let Some(spec) = self.state.character.as_ref().map(|c| c.spec.clone()) else {
            return Ok(());
        };
        let count = creation_count(&self.content, &spec);
        let taken = if spec.talents.is_empty() {
            self.take_random()?
        } else {
            for pick in &spec.talents {
                let (id, choice) = split(pick);
                self.learn(id, choice).map_err(|why| {
                    let why = why.trim_end_matches('.');
                    WorldError::Character(format!(
                        "talent `{pick}`: {why} (a {} takes {count} at creation)",
                        spec.class
                    ))
                })?;
            }
            spec.talents.clone()
        };
        if let Some(due) = self.creation_due() {
            return Err(WorldError::Character(format!(
                "{} more talent(s) to pick: this character takes {count} at creation",
                due.talents
            )));
        }
        if let Some(character) = &mut self.state.character {
            character.spec.talents = taken;
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
            let offers = talent::offers(&self.content, &self.candidate());
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
