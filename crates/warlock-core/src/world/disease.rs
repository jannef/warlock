//! Diseases in the world (spec §8 "Diseases"); the rules of one actor's infections are
//! [`crate::disease`], the diseases data ([`crate::content::DiseaseDef`]).
//!
//! - **Exposure** ([`World::expose`]): a bite whose `on_hit` names a disease (its chance drawn
//!   first, [`World::after_blow`]), the terrain a disease is caught in as the player steps onto
//!   it ([`World::wade`]), or a named source a rule exposes an actor to ([`World::expose_to`]:
//!   eating a rotten corpse). An actor that does not carry the disease yet makes a check
//!   ([`crate::check`]: 2d12 plus its modifier in the disease's `check` stat, and the player's
//!   race's `disease_check`, against its `difficulty`, [`World::disease_modifier`]); failing
//!   it, it catches it ([`World::infect`]): it incubates for its
//!   `incubation`, rolled in days. Nothing of this is said or shown.
//! - **Its turns** ([`World::disease_turn`], as the clock moves: `World::live`): at the end of
//!   its incubation it shows (its `onset` is said, and the actor is Diseased, the status
//!   diseases show as); then every `interval` days it progresses: a check. A failure costs its
//!   `damage` (stat damage, [`crate::stats::StatDamage`]) and `hp`, and says its next symptom;
//!   a win counts toward `recovery`, and that many in a row shake it off (its `recovered` is
//!   said; with none left showing, Diseased ends). The sufferer knows which disease it is after
//!   its first progression.
//! - **Stat damage** never takes a score below 1, and its hit point damage never takes the last
//!   one, unless the disease `kills`: then a progression that takes the last hit point, or
//!   Toughness to 0, kills (by Diseased, `Event::Died`'s `of`; credited to whoever gave it).
//!   Toughness damage lowers the most hit points (what is left is cut down to them). Once no
//!   disease shows, a point of each damaged stat mends every day ([`mending`]).
//! - **Healing**: while a disease shows, hit points come back at the slowest of Diseased's
//!   `regen` and the disease's own (`World::healing_rate`).
//! - **Cures**: [`World::cure`] ends the diseases that list the means in their `cured_by`,
//!   showing or not; nothing applies one yet (potions and priests will).

use super::fight::Harm;
use super::{Actor, ActorId, Spent, World};
use crate::action::{DiseaseNews, Event};
use crate::check::check;
use crate::content::{DiseaseId, Effect, TerrainId};
use crate::stats::Stat;
use crate::talent::TalentMods;
use crate::time::{DAY, GameTime};

/// The odds of a `per_hour` chance are out of this many percent-seconds: 100 % of an hour.
const PER_HOUR: u64 = 100 * crate::time::HOUR;

impl World {
    /// Actor `id` is exposed to `disease`, by `by` (if anyone): unless it carries the disease
    /// already, it resists with a check (two draws), and catches it if that fails. Returns
    /// whether it caught it. Nothing is said: the sufferer finds out when it shows.
    pub(crate) fn expose(&mut self, id: ActorId, disease: DiseaseId, by: Option<ActorId>) -> bool {
        let def = self.content.disease(disease);
        let Some(actor) = self.state.actors.get(&id) else {
            return false;
        };
        if actor.diseases.has(disease) {
            return false;
        }
        // Purity of body (spec §8 "Class abilities"): never caught, nothing rolled.
        if actor.talent_mods(&self.content).disease_immune {
            return false;
        }
        // Immune by its type (spec §8 "Status effects"): nothing lives in it to sicken.
        let immune = &self.content.creature(actor.creature).immune;
        if (self.content.disease_status()).is_some_and(|s| immune.contains(&s)) {
            return false;
        }
        let (stat, difficulty) = (def.check, def.difficulty);
        let modifier = self.disease_modifier(actor, stat);
        // Know your enemy (Knowledge, `knowledge.rs`): a disease a creature's attack carries.
        let modifier = modifier.saturating_add(self.resisting(id, by));
        if check(&mut self.state.rng, modifier, difficulty) {
            self.train_resisting(id, stat);
            return false;
        }
        self.infect(id, disease, by)
    }

    /// What `actor` adds to a check against a disease whose `check` stat is `stat`: its
    /// modifier in the stat (of the effective score, spec §8 "Stats"), its talents' bonus
    /// (Strong constitution, and what adds to every check of the stat: Stubborn, Iron will; spec §8 "Talents"), and the player's race's `disease_check`
    /// (ratfolk shrug diseases off).
    pub(super) fn disease_modifier(&self, actor: &Actor, stat: Stat) -> i32 {
        let race = self.state.race_of(&self.content, actor);
        let modifier = actor.effective_stats(&self.content).modifier(stat);
        let talents = actor.talent_mods(&self.content);
        modifier
            .saturating_add(talents.disease_checks)
            .saturating_add(actor.mods(&self.content).get(Effect::DiseaseCheck))
            .saturating_add(talents.check(stat))
            .saturating_add(race.map_or(0, |r| r.disease_check))
    }

    /// Actor `id` catches `disease`, given by `by` (if anyone): it incubates for its
    /// `incubation` (rolled, in days). Returns whether it did: not when it carries it already.
    pub(crate) fn infect(&mut self, id: ActorId, disease: DiseaseId, by: Option<ActorId>) -> bool {
        let def = self.content.disease(disease);
        if !(self.state.actors.get(&id)).is_some_and(|a| !a.diseases.has(disease)) {
            return false;
        }
        let days = u64::try_from(def.incubation.roll(&mut self.state.rng).max(1)).unwrap_or(1);
        (self.state.actors.get_mut(&id)).is_some_and(|a| a.diseases.catch(disease, days * DAY, by))
    }

    /// Actor `id` is exposed to every disease caught from `source` (one of
    /// [`crate::content::SOURCES`]: `rotten_food`), in disease order. Eating a rotten corpse
    /// (`world/eat.rs`) exposes the eater here.
    pub(crate) fn expose_to(&mut self, id: ActorId, source: &str) {
        let caught: Vec<DiseaseId> = (self.content.disease_ids())
            .filter(|(_, def)| def.caught.iter().any(|c| c == source))
            .map(|(disease, _)| disease)
            .collect();
        for disease in caught {
            self.expose(id, disease, None);
        }
    }

    /// The player stepped onto `terrain`, a step of `seconds`: for every disease caught there
    /// that they do not carry, one draw for its chance (`per_hour` percent an hour, by the
    /// step's length; none when it is sure), then exposure. Creatures do not catch diseases
    /// from the land, nor does one with Nature's ward (no draw).
    pub(super) fn wade(&mut self, terrain: TerrainId, seconds: u64) {
        let player = self.state.player;
        let content = self.content.clone();
        // A strong swimmer (spec §8 "Birth sign") never catches anything wading, nor does a
        // druid with Nature's ward.
        let mods = self.state.player().talent_mods(&content);
        if mods.swimmer || mods.wading_proof {
            return;
        }
        for (disease, def) in content.disease_ids() {
            let Some(wading) = def.wading.as_ref().filter(|w| w.terrain.contains(&terrain)) else {
                continue;
            };
            if self.state.player().diseases.has(disease) {
                continue;
            }
            let odds = u64::from(wading.per_hour).saturating_mul(seconds.max(1));
            let sure = odds >= PER_HOUR;
            if sure || u64::from(self.state.rng.below(PER_HOUR as u32)) < odds {
                self.expose(player, disease, None);
            }
        }
    }

    /// The turn of actor `id`'s `disease` came, at `when`: its onset, or a progression (a
    /// check; a failure costs its damage). Returns whether the actor lives on.
    pub(super) fn disease_turn(
        &mut self,
        id: ActorId,
        disease: DiseaseId,
        when: GameTime,
        events: &mut Vec<Event>,
    ) -> bool {
        let content = self.content.clone();
        let def = content.disease(disease);
        let player = self.state.player;
        let interval = u32::try_from(def.interval_seconds()).unwrap_or(u32::MAX);
        let Some(actor) = self.state.actors.get(&id) else {
            return false;
        };
        let modifier = self.disease_modifier(actor, def.check);
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return false;
        };
        let talents = actor.talent_mods(&content);
        let recovery = recovery_of(def, &talents);
        // Ageless (an elf's): the progressions come further apart.
        let interval = match talents.disease_interval_percent {
            0 => interval,
            percent => {
                let scaled = TalentMods::scaled(u64::from(interval), percent).max(1);
                u32::try_from(scaled).unwrap_or(u32::MAX)
            }
        };
        let Some(infection) = actor.diseases.get_mut(disease) else {
            return true;
        };
        infection.left = interval;
        let by = infection.by;
        if !infection.shown {
            infection.shown = true;
            if id == player {
                let news = DiseaseNews::Onset;
                events.push(Event::Disease {
                    actor: id,
                    disease,
                    news,
                });
            }
            if let Some(status) = content.disease_status() {
                self.afflict(id, status, 1, by, events);
            }
            return true;
        }
        infection.progressions = infection.progressions.saturating_add(1);
        let n = infection.progressions;
        if check(&mut self.state.rng, modifier, def.difficulty) {
            infection.streak = infection.streak.saturating_add(1);
            if def.recovers() && infection.streak >= recovery {
                self.end_disease(id, disease, events);
            }
            self.train_resisting(id, def.check);
            return true;
        }
        infection.streak = 0;
        // The damage of a failed check, stat by stat in display order, then hit points.
        let mut no_toughness = false;
        for stat in Stat::ALL {
            let dice = def.damage[stat];
            if dice.is_zero() {
                continue;
            }
            let points = dice.roll(&mut self.state.rng);
            let gone = actor.damage.take(stat, points, actor.stats.score(stat));
            no_toughness |= gone && stat == Stat::Toughness;
        }
        let lost = def.hp.map_or(0, |hp| hp.roll(&mut self.state.rng));
        if id == player {
            let news = DiseaseNews::Worse(n);
            events.push(Event::Disease {
                actor: id,
                disease,
                news,
            });
        }
        // Hit points (never the last one unless it kills), then the Toughness it took: that
        // lowers the most hit points, and what is left is cut down to them; at none, it kills.
        let of = content.disease_status();
        let harm = if def.kills {
            Harm::Status { of, by, when }
        } else {
            Harm::Spared
        };
        if self.hurt(id, lost, harm, events) {
            return false;
        }
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return false;
        };
        actor.hp = actor.hp.min(actor.max_hp(&content));
        // Damage to the mental stats lowers the most mana the same way (spec §8 "Mana").
        actor.mana = actor.mana.min(actor.max_mana(&content));
        if def.kills && no_toughness {
            self.dies(id, by, of, when, events);
            return false;
        }
        true
    }

    /// Actor `id` is rid of `disease`: shaken off or cured. The player is told if it showed;
    /// with no disease left showing, Diseased ends.
    pub(super) fn end_disease(&mut self, id: ActorId, disease: DiseaseId, events: &mut Vec<Event>) {
        let player = self.state.player;
        let status = self.content.disease_status();
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        let Some(gone) = actor.diseases.remove(disease) else {
            return;
        };
        if gone.shown && id == player {
            let news = DiseaseNews::Recovered;
            events.push(Event::Disease {
                actor: id,
                disease,
                news,
            });
        }
        if let Some(status) = status
            && !actor.diseases.shows()
            && actor.effects.remove(status)
            && id == player
        {
            events.push(Event::Status {
                actor: id,
                status,
                began: false,
            });
        }
    }

    /// The dev action `infect:DISEASE` ([`crate::action::Action::Infect`]): the player catches
    /// disease `id`. Refused for an unknown disease, or one they carry already. Takes no time.
    pub(super) fn infect_dev(&mut self, id: &str) -> Spent {
        let Some(disease) = self.content.disease_id(id) else {
            self.say(format!("There is no disease `{id}`."));
            return Spent::Nothing;
        };
        let player = self.state.player;
        let name = self.content.disease(disease).name.clone();
        if !self.infect(player, disease, None) {
            self.say(format!("You carry {name} already (dev)."));
            return Spent::Nothing;
        }
        self.say(format!("You catch {name} (dev)."));
        Spent::NoTime
    }
}

/// How many checks won in a row shake `def` off a sufferer with `talents`: its `recovery`, less
/// what Fever-proof takes off (spec §8 "Talents"), never under 1.
pub(super) fn recovery_of(def: &crate::content::DiseaseDef, talents: &TalentMods) -> u8 {
    let recovery = i32::from(def.recovery) - talents.disease_recovery;
    u8::try_from(recovery.max(1)).unwrap_or(def.recovery)
}

/// Whether `actor`'s stat damage mends now: it has some, and no disease shows.
pub(super) fn mending(actor: &Actor) -> bool {
    actor.damage.is_damaged() && !actor.diseases.shows()
}

/// Seconds until the next moment `actor`'s diseases or stat damage do something: a disease's
/// turn, or a point mended. `None`: nothing will.
pub(super) fn until_next(actor: &Actor) -> Option<u64> {
    let mend = mending(actor).then(|| DAY - u64::from(actor.damage.mend));
    actor.diseases.until_next().into_iter().chain(mend).min()
}

/// `seconds` pass for `actor`'s diseases and stat damage (callers keep them within
/// [`until_next`]): a point of every damaged stat mends each day it mends, and the diseases'
/// timers run. Returns the diseases whose turn came, in disease order.
pub(super) fn pass(actor: &mut Actor, seconds: u64) -> Vec<DiseaseId> {
    if mending(actor) {
        let banked = u64::from(actor.damage.mend) + seconds;
        if banked >= DAY {
            actor.damage.mend_point();
        }
        actor.damage.mend = u32::try_from(banked % DAY).unwrap_or(0);
        if !actor.damage.is_damaged() {
            actor.damage.mend = 0;
        }
    } else {
        actor.damage.mend = 0;
    }
    actor.diseases.pass(seconds)
}
