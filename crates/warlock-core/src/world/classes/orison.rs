//! The priest's prayer (spec §8 "Class abilities"; the priest design, `docs/design/priest.md`
//! §6.1; `docs/dev/prayer.md`): the class active Orison, `use:orison` at every tier. A prayer
//! answers what ails the player most, as far as their prayer talents reach
//! ([`crate::talent::Prayer`]: the uses, how many answers, all at once), in the priority order
//! of `data/prayer.toml`, under a hidden **strain** ([`Strain`]) that every attempt raises and
//! time lowers.
//!
//! - **The face**: the player's band at the moment picks who answers ([`Actor::prayer_face`]):
//!   the holy powers or the dark ones, each with its own words and titles. The dark powers
//!   answer the same problems with a twist (`dark_face.rs`: a plague and a poison passed on, a
//!   hex turned back, a meal to the brim).
//! - **Refused** (no time, no strain): without a prayer talent, without the file, or with nothing
//!   among the unlocked uses to answer (the face's `nothing`). A disease still brewing is no
//!   problem of its own, so a prayer tells the player nothing of one.
//! - **The attempt** takes the time of a cast and adds the strain, answered or not. A locked
//!   prayer fizzles; else it fizzles by chance, `fizzle_per_100` percent per 100 points of the
//!   strain it finds (one gameplay draw, made only while there is strain). A fizzle locks the
//!   prayer until the strain has passed to 0. Answered, the first `answers` problems (every one
//!   with `all`, and every resource pool filled) are fixed, each through its one place, each
//!   said in the face's words; the statuses and diseases it ends say nothing of their own.
//!
//! Communion with the dark (`dark_face.rs`): standing in an unholy place, a dark priest's
//! prayers add no strain.
//!
//! The strain is lazy: its value at a moment, less a point for every `decay` since; reading it
//! writes nothing, so however time is cut it passes the same. Nothing trains, and no score moves.

use crate::action::{DiseaseNews, Event};
use crate::content::{ActionKind, Content, Face, PrayerRules, PrayerUse, TalentEffect};
use crate::hunger::Hunger;
use crate::talent::FaithEffect;
use crate::time::GameTime;
use crate::world::Actor;
use crate::world::{Spent, World};
use serde::{Deserialize, Serialize};

/// The strain of the player's prayers (saved in `Progress::prayer`): its value at `at`, and
/// whether a fizzle has locked the prayer until it passes to 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Strain {
    pub(crate) value: u16,
    pub(crate) at: GameTime,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(crate) locked: bool,
}

impl Strain {
    /// Its value at `time`: a point less for every `decay` since `at`, never below 0.
    pub(crate) fn now(&self, rules: &PrayerRules, time: GameTime) -> u16 {
        let passed = time.since(self.at).as_seconds() / rules.decay.as_seconds().max(1);
        self.value
            .saturating_sub(u16::try_from(passed).unwrap_or(u16::MAX))
    }

    /// Whether the prayer is locked at `time`: a fizzle locked it and the strain has not passed.
    pub(crate) fn locked_at(&self, rules: &PrayerRules, time: GameTime) -> bool {
        self.locked && self.now(rules, time) > 0
    }
}

impl Actor {
    /// Who answers its prayers now: by its band, the holy powers or the dark ones (the holy
    /// without alignment rules or the prayer's file).
    pub fn prayer_face(&self, content: &Content) -> Face {
        let band = content.alignment().map(|_| self.band(content));
        content
            .prayer()
            .map_or(Face::Holy, |rules| rules.face(band))
    }

    /// Its prayer's name and description in the abilities menu: the highest tier's title as its
    /// face gives it, and that tier's talent's description; `None` without a prayer.
    pub fn prayer_title<'c>(&self, content: &'c Content) -> Option<(&'c str, &'c str)> {
        let tier = (self.talent_mods(content).faith.prayer.as_ref()).map(|p| p.tier)?;
        let face = self.prayer_face(content);
        (content.talents().iter()).find_map(|(_, def)| {
            def.effects.iter().find_map(|e| match e {
                TalentEffect::Faith(FaithEffect::Prayer {
                    tier: Some(t),
                    title: Some(title),
                    ..
                }) if *t == tier => Some((title.of(face), def.description.as_str())),
                _ => None,
            })
        })
    }
}

/// The prayer as the dev dumps show it (ground truth): the strain now and whether it is locked,
/// the face that answers, the uses held in priority order, and the answers a prayer gives
/// (`None`: every one).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrayerState {
    pub strain: u16,
    pub locked: bool,
    pub face: Face,
    pub uses: Vec<PrayerUse>,
    pub answers: Option<u8>,
}

impl World {
    /// The player's prayer (dev; `None` without a prayer talent or the rules).
    pub fn prayer_state(&self) -> Option<PrayerState> {
        let rules = self.content.prayer()?;
        let me = self.state.player();
        let prayer = me.talent_mods(&self.content).faith.prayer.clone()?;
        let strain = self.state.progress.prayer;
        let time = self.state.time;
        Some(PrayerState {
            strain: strain.map_or(0, |s| s.now(rules, time)),
            locked: strain.is_some_and(|s| s.locked_at(rules, time)),
            face: me.prayer_face(&self.content),
            uses: (rules.uses.iter())
                .map(|u| u.id)
                .filter(|&u| prayer.answers_to(u))
                .collect(),
            answers: (!prayer.all).then_some(prayer.answers),
        })
    }

    /// `use:orison`: the player prays (module docs).
    pub(super) fn orison(&mut self, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let me = self.state.player();
        let Some(prayer) = me.talent_mods(&content).faith.prayer.clone() else {
            return Spent::Nothing;
        };
        let face = me.prayer_face(&content);
        let Some(rules) = content.prayer() else {
            self.say("You have nothing to pray for.");
            return Spent::Nothing;
        };
        let problems: Vec<PrayerUse> = (rules.uses.iter())
            .map(|u| u.id)
            .filter(|&u| prayer.answers_to(u) && self.ails(rules, u))
            .collect();
        let words = rules.words(face);
        if problems.is_empty() {
            self.say(words.nothing.clone());
            return Spent::Nothing;
        }
        let time = self.state.time;
        let strain = self.state.progress.prayer;
        let now = strain.map_or(0, |s| s.now(rules, time));
        let locked = strain.is_some_and(|s| s.locked_at(rules, time));
        let chance = rules.fizzle_chance(now);
        let fizzled = locked || (chance > 0 && self.state.rng.chance(chance));
        let added = if self.communing_here() {
            0
        } else {
            rules.strain
        };
        self.state.progress.prayer = Some(Strain {
            value: now.saturating_add(added),
            at: time,
            locked: fizzled,
        });
        let spent = Spent::Turn(self.price_of(player, ActionKind::Cast));
        if fizzled {
            self.say(words.fizzle.clone());
            return spent;
        }
        self.say(words.pray.clone());
        let answers = if prayer.all {
            problems.len()
        } else {
            usize::from(prayer.answers.max(1))
        };
        let mut quiet = Vec::new();
        for answer in problems.into_iter().take(answers) {
            self.answer_prayer(rules, answer, face, &mut quiet);
            if let Some(line) = rules.use_words(answer) {
                self.say(line.of(face).to_string());
            }
        }
        if prayer.all {
            self.fill_pools();
        }
        // What the prayer ended says nothing of its own: the prayer's lines cover it.
        events.extend(quiet.into_iter().filter(|e| {
            !matches!(
                e,
                Event::Status { began: false, .. }
                    | Event::Disease {
                        news: DiseaseNews::Recovered,
                        ..
                    }
            )
        }));
        spent
    }

    /// Whether `id` ails the player now: the problem its use answers.
    fn ails(&self, rules: &PrayerRules, id: PrayerUse) -> bool {
        let content = &self.content;
        let me = self.state.player();
        let suffers = |status: Option<_>| status.is_some_and(|s| me.effects.has(s));
        match id {
            PrayerUse::Disease => me.diseases.iter().any(|i| i.shown),
            PrayerUse::Poison => suffers(content.ids().poisoned),
            PrayerUse::Wounds => suffers(content.ids().bleeding) || me.hp < me.max_hp(content),
            PrayerUse::Release => rules.release.iter().any(|&s| me.effects.has(s)),
            PrayerUse::Mana => me.mana < me.max_mana(content),
            PrayerUse::Hunger => me.hunger(content).is_some_and(|h| h <= Hunger::Hungry),
            PrayerUse::Restoration => {
                me.damage.is_damaged() || !crate::world::turn::actor::undrained(&me.drained)
            }
        }
    }

    /// Use `id` answered by `face`: its problem fixed through its one place, and the dark
    /// face's twist (`dark_face.rs`).
    fn answer_prayer(
        &mut self,
        rules: &PrayerRules,
        id: PrayerUse,
        face: Face,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let player = self.state.player;
        let dark = face == Face::Dark;
        match id {
            PrayerUse::Disease => {
                let diseases: Vec<_> = (self.state.player().diseases.iter())
                    .map(|i| i.disease)
                    .collect();
                for &disease in &diseases {
                    self.end_disease(player, disease, events);
                }
                if dark {
                    self.pass_on_diseases(&diseases);
                }
            }
            PrayerUse::Poison => {
                let ended = self.end_statuses(&[content.ids().poisoned], events);
                if dark {
                    self.pass_on_poison(&ended, events);
                }
            }
            PrayerUse::Wounds => {
                self.end_statuses(&[content.ids().bleeding], events);
                let max = self.state.player().max_hp(&content);
                let me = self.player_mut();
                me.hp = me.hp.max(max);
            }
            PrayerUse::Release => {
                let release: Vec<_> = rules.release.iter().copied().map(Some).collect();
                let ended = self.end_statuses(&release, events);
                if dark {
                    self.turn_hexes_back(&ended, events);
                }
            }
            PrayerUse::Mana => {
                let max = self.state.player().max_mana(&content);
                let me = self.player_mut();
                me.mana = me.mana.max(max);
            }
            PrayerUse::Hunger => {
                let Some(hunger) = content.hunger() else {
                    return;
                };
                // The dark feeds to the brim: one short of Bloated.
                let to = if dark {
                    hunger.states.bloated - 1
                } else {
                    hunger.states.satiated
                };
                if let Some(satiation) = self.player_mut().satiation.as_mut() {
                    satiation.nutrition = satiation.nutrition.max(to);
                }
            }
            PrayerUse::Restoration => self.mend(player),
        }
    }

    /// Every status of `statuses` the player suffers ends; returns what ended.
    fn end_statuses(
        &mut self,
        statuses: &[Option<crate::content::StatusId>],
        events: &mut Vec<Event>,
    ) -> Vec<crate::status::Effect> {
        let player = self.state.player;
        let me = self.player_mut();
        let ended: Vec<crate::status::Effect> = (statuses.iter().flatten())
            .filter_map(|&status| me.effects.get(status).copied())
            .collect();
        for e in &ended {
            me.effects.remove(e.status);
        }
        self.ended(player, &ended, events);
        ended
    }

    /// Every resource pool the player holds fills to its maximum (Miracle's).
    fn fill_pools(&mut self) {
        let player = self.state.player;
        let pools: Vec<_> = (self.state.player().resources.iter())
            .map(|p| p.resource)
            .collect();
        for resource in pools {
            let max = self.resource_max(player, resource);
            self.player_mut().resources.set(resource, max);
        }
    }
}
