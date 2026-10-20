//! Status effects and passive healing in the world (spec §8 "Status effects", "Healing"); the
//! rules of one actor's effects are [`crate::status`], the statuses data
//! ([`crate::content::StatusDef`]).
//!
//! - **Causes**: a blow that hits and leaves its target alive ([`World::after_blow`]) causes
//!   every status whose `wound` it meets (bleeding: a big hit or a confirmed critical), then each
//!   `on_hit` effect of the attack (a giant rat's bite may poison, or expose its target to filth
//!   fever: `world/disease.rs`), each with one draw of the gameplay stream for its chance (none
//!   at 100 %). Other rules call [`World::afflict`].
//! - **Time**: as the scheduler moves the clock, every actor on the current level lives through
//!   it ([`World::pass_time`]): its effects tick (costing hit points) and ease, and it gets hit
//!   points back at its healing rate, in time order, so it makes no difference how the time is
//!   cut up (by a long journey or many short actions). Diseases take their turns and stat damage
//!   mends at their own moments too (`world/disease.rs`). At one moment a hit point back comes
//!   first, then the ticks, then the easing, then the player's hunger, then a point of stat
//!   damage mending, then the diseases' turns, then food rotting away in its pack
//!   (`world/rot.rs`). Actors on other levels are frozen, as with energy, and the dev time skip
//!   passes no time for anyone.
//! - **Healing** ([`World::healing_rate`]): normally one hit point per
//!   [`combat::regen_seconds`] of the actor's (effective) Toughness; twice as fast while the
//!   player rests (`[healing] resting`) and three times as fast in a sanctuary (a special
//!   room's `regen`); slowed or stopped by the slowest `regen` of its statuses (bleeding stops
//!   it, poison and disease halve it) and of the diseases that show (marsh ague slows it more),
//!   and stopped while the player is Starving (spec §8 "Hunger"); faster with regeneration
//!   affixes on what it wears and wields (spec §8 "Magic items").
//! - **Hunger**: the player's nutrition burns as they live through the time, and starving costs
//!   them hit points ([`World::digest`], `hunger.rs`); each change of their hunger state is a
//!   moment of its own, so healing stops exactly when they start to starve.
//! - **Deaths**: a tick that takes the last hit point kills, at that moment, by the status
//!   (`Event::Died`'s `of`, [`super::Death`]'s `of`); the creature counts as killed by whoever
//!   caused the effect ([`crate::status::Effect::by`]). Starving to death is the player's death
//!   by hunger ([`super::Death`]'s `cause`), at its moment too.
//! - **Messages**: the player's statuses beginning and ending are events
//!   ([`Event::Status`]), said as the turn is perceived, in order with the attacks that caused
//!   them ("The giant rat bites you. You feel sick."); a status ending ("The bleeding stops.") is
//!   only logged and lets a rest go on. Nothing is said of a creature's statuses but its death
//!   ("The giant rat bleeds to death."); look mode shows those that show.
//! - **Cures**: [`World::cure`] ends what a named means cures (`cured_by`: a bandage stops the
//!   bleeding; diseases list their own), for the Heal skill (#132), potions, and priests to use.

use super::fight::Harm;
use super::{ActorId, Spent, World, disease, rot};
use crate::action::{AttackKind, Event};
use crate::combat::{self, Blow, Strike};
use crate::content::Effect as ItemEffect;
use crate::content::{Affliction, Content, DiseaseId, OnHit, StatusId};
use crate::hunger::Hunger;
use crate::stats::Stat;
use crate::status::Added;
use crate::time::{Duration, GameTime};

/// What happened to an actor at one moment while time passed.
struct Moment {
    /// Hit points its effects cost then.
    damage: i32,
    /// The first status that ticked (the most important) and who caused it: a death is by it.
    by: Option<(StatusId, Option<ActorId>)>,
    /// Whether a status that bleeds ticked (blood drips).
    bled: bool,
}

impl World {
    /// Actor `id` comes to suffer `severity` degrees of status `status`, caused by `by` (if
    /// anyone): added to what it has by the status's stacking rule. The player is told when it
    /// begins.
    pub(crate) fn afflict(
        &mut self,
        id: ActorId,
        status: StatusId,
        severity: u16,
        by: Option<ActorId>,
        events: &mut Vec<Event>,
    ) {
        let def = self.content.status(status);
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        let added = actor.effects.add(status, def, severity, by);
        if added == Added::Began && id == self.state.player {
            events.push(Event::Status {
                actor: id,
                status,
                began: true,
            });
        }
    }

    /// Cures actor `id` with `cure`, a means named in the statuses' and diseases' `cured_by`
    /// (`bandage`, `remove_disease`): every status of its that the means cures ends, and every
    /// disease (showing or not). Returns whether any did. Eating a remedy applies its cures
    /// (`world/eat.rs`); the Heal skill's bandaging (#132), potions, and priests will too.
    pub(crate) fn cure(&mut self, id: ActorId, cure: &str, events: &mut Vec<Event>) -> bool {
        let content = self.content.clone();
        let player = self.state.player;
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return false;
        };
        let cures = |by: &[String]| by.iter().any(|c| c == cure);
        let cured: Vec<StatusId> = (actor.effects.iter())
            .map(|e| e.status)
            .filter(|&s| cures(&content.status(s).cured_by))
            .collect();
        let healed: Vec<DiseaseId> = (actor.diseases.iter())
            .map(|i| i.disease)
            .filter(|&d| cures(&content.disease(d).cured_by))
            .collect();
        for &status in &cured {
            actor.effects.remove(status);
            if id == player {
                events.push(Event::Status {
                    actor: id,
                    status,
                    began: false,
                });
            }
        }
        for &disease in &healed {
            self.end_disease(id, disease, events);
        }
        !cured.is_empty() || !healed.is_empty()
    }

    /// A blow by `attacker` with an attack of `kind` hit `target` and left it alive: the
    /// statuses its wound causes (bleeding from a big hit or a confirmed critical), then the
    /// `on_hit` effects of the attack, each by its chance.
    pub(super) fn after_blow(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        kind: AttackKind,
        blow: &Blow,
        events: &mut Vec<Event>,
    ) {
        if !self.state.actors.contains_key(&target) {
            return;
        }
        self.wounded(target, blow, Some(attacker), events);
        for hit in self.on_hit_of(attacker, kind) {
            let lands = hit.chance >= 100 || self.state.rng.chance(u32::from(hit.chance));
            if !lands {
                continue;
            }
            match hit.what {
                Affliction::Status { status, severity } => {
                    self.afflict(target, status, severity, Some(attacker), events);
                }
                Affliction::Disease(disease) => {
                    self.expose(target, disease, Some(attacker));
                }
            }
        }
    }

    /// `target` took `blow`, dealt by `by` (none for a trap's, spec §8 "Traps"), and lives: the
    /// statuses its wound causes (bleeding from a big hit or a confirmed critical).
    pub(super) fn wounded(
        &mut self,
        target: ActorId,
        blow: &Blow,
        by: Option<ActorId>,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let Some(victim) = self.state.actors.get(&target) else {
            return;
        };
        let critical = matches!(blow.strike, Strike::Hit { critical: true });
        let big = combat::big_hit(blow.damage, victim.max_hp(&content));
        // Every blow wounds alike for now. Weapon categories (#134) will let only cutting and
        // piercing ones bleed, and bloodless creatures (#127's `blood = "none"`) will not.
        for (status, def) in content.status_ids() {
            let degrees = def.wound.map_or(0, |w| w.degrees(critical, big));
            if degrees > 0 {
                self.afflict(target, status, degrees, by, events);
            }
        }
    }

    /// What a hit by `attacker` with an attack of `kind` may cause besides its damage: its
    /// natural attack's `on_hit` when it fights with it (a blow with no weapon in hand).
    /// Weapons and ammunition (poisoned arrows) could carry their own later.
    fn on_hit_of(&self, attacker: ActorId, kind: AttackKind) -> Vec<OnHit> {
        let Some(me) = self.actor_or_fallen(attacker) else {
            return Vec::new();
        };
        let natural = self.content.creature(me.creature).attack.as_ref();
        match (kind, natural) {
            (AttackKind::Melee, Some(natural)) if me.weapon(&self.content).is_none() => {
                natural.on_hit.clone()
            }
            _ => Vec::new(),
        }
    }

    /// How fast actor `id` gets hit points back now, in percent of the normal pace (spec §8
    /// "Healing"): the slowest `regen` of its statuses and of the diseases that show (100
    /// without any), times the resting bonus while the player rests, times the `regen` of a
    /// special room it stands in (a sanctuary: three times; spec §7 "Special rooms"), times what
    /// the regeneration affixes of what it wears and wields add (spec §8 "Magic items": +50 % is
    /// half again as fast). The one place that decides it. A Starving player gets none back
    /// (spec §8 "Hunger"); disease (#139) slows it through Diseased's `regen` and its own.
    pub(crate) fn healing_rate(&self, id: ActorId) -> u32 {
        let Some(actor) = self.state.actors.get(&id) else {
            return 0;
        };
        if actor.hunger(&self.content) == Some(Hunger::Starving) {
            return 0;
        }
        let statuses = actor.effects.regen(&self.content);
        let mut rate = u32::from(statuses.min(actor.diseases.regen(&self.content)));
        if self.resting && id == self.state.player {
            rate = rate * u32::from(self.content.healing().resting) / 100;
        }
        let room = (self.state.levels.get(&actor.level))
            .and_then(|level| level.room_effect_at(&self.content, actor.pos));
        let rate = room.map_or(rate, |effect| rate * u32::from(effect.regen) / 100);
        let faster = actor.mods(&self.content).get(ItemEffect::Regeneration);
        let faster = u32::try_from(faster.max(0)).unwrap_or(0);
        rate * (100 + faster) / 100
    }

    /// `seconds` of game time from `start` pass for actors `ids` (on the current level, in
    /// schedule order): each lives through them ([`World::live`]). Some may die.
    pub(super) fn pass_time(
        &mut self,
        ids: impl Iterator<Item = ActorId>,
        start: GameTime,
        seconds: u64,
        events: &mut Vec<Event>,
    ) {
        for id in ids {
            self.live(id, start, seconds, events);
        }
    }

    /// `seconds` of game time from `start` pass for actor `id`: its effects tick and ease, it
    /// heals, (the player) hungers, its stat damage mends, its diseases take their turns, and
    /// the food in its pack rots away, each at the moment it happens, until the time is up or it
    /// dies.
    fn live(&mut self, id: ActorId, start: GameTime, seconds: u64, events: &mut Vec<Event>) {
        let content = self.content.clone();
        // What rotted away while it was frozen (on another level, or over a dev skip) goes first.
        self.rot_pack(id, start);
        let mut passed = 0;
        while passed < seconds {
            let rate = self.healing_rate(id);
            let hungrier = self.until_hungrier(id);
            let Some(actor) = self.state.actors.get_mut(&id) else {
                return;
            };
            if actor.hp <= 0 {
                return;
            }
            let left = seconds - passed;
            let rots = rot::until_rots(
                &actor.inventory,
                &content,
                start + Duration::seconds(passed),
            );
            if actor.effects.is_empty()
                && hungrier.is_none()
                && actor.diseases.is_empty()
                && actor.damage.is_none()
                && rots.is_none_or(|r| r > left)
            {
                heal(actor, &content, left, rate);
                return;
            }
            // Up to the next moment something happens: a tick, an easing, a hit point back, a
            // change in its hunger (a new state, a hit point lost to starving), a disease's
            // turn, a point of stat damage mended, or food in its pack rotting away.
            let mut step = hungrier.map_or(left, |h| h.min(left));
            if let Some(next) = disease::until_next(actor) {
                step = step.min(next);
            }
            if let Some(r) = rots {
                step = step.min(r.max(1));
            }
            for e in actor.effects.iter() {
                let def = content.status(e.status);
                step = (e.until_tick(def).into_iter())
                    .chain(e.until_eases(def))
                    .fold(step, u64::min);
            }
            if let Some(need) = need(actor, &content)
                && rate > 0
            {
                let bank = u64::from(actor.regen);
                step = step.min(need.saturating_sub(bank).div_ceil(u64::from(rate)).max(1));
            }
            passed += step;
            heal(actor, &content, step, rate);
            let mut moment = Moment {
                damage: 0,
                by: None,
                bled: false,
            };
            for e in actor.effects.iter_mut() {
                let def = content.status(e.status);
                let (ticks, eased) = e.advance(def, step);
                if ticks > 0 {
                    let cost = u64::from(def.damage) * ticks;
                    moment.damage += i32::try_from(cost).unwrap_or(i32::MAX);
                    moment.by.get_or_insert((e.status, e.by));
                    moment.bled |= def.wound.is_some();
                }
                e.severity = e
                    .severity
                    .saturating_sub(u16::try_from(eased).unwrap_or(u16::MAX));
            }
            let spent = actor.effects.drop_spent();
            let due = disease::pass(actor, step);
            if moment.bled {
                self.drip_blood(id);
            }
            // The ticks' harm first, then starving's: whichever takes the last hit point kills.
            let when = start + Duration::seconds(passed);
            if let Some((status, by)) = moment.by {
                let of = Some(status);
                if self.hurt(id, moment.damage, Harm::Status { of, by, when }, events) {
                    return;
                }
            }
            let (starved, hunger) = self.digest(id, step);
            if starved > 0 && self.hurt(id, starved, Harm::Starving(when), events) {
                return;
            }
            if id == self.state.player {
                events.extend(spent.into_iter().map(|status| Event::Status {
                    actor: id,
                    status,
                    began: false,
                }));
            }
            if let Some(before) = hunger {
                self.hunger_news(before);
            }
            for disease in due {
                if !self.disease_turn(id, disease, when, events) {
                    return;
                }
            }
            self.rot_pack(id, when);
        }
    }

    /// Actor `id` loses blood where it stands: the hook for #127's stains, which will drip
    /// there. Nothing is stained yet.
    fn drip_blood(&mut self, id: ActorId) {
        let _ = id;
    }

    /// The dev action `afflict:STATUS` ([`crate::action::Action::Afflict`]): the player suffers
    /// status `id` at severity 1. Refused for an unknown status, and for the one diseases show
    /// as. Takes no time.
    pub(super) fn afflict_dev(&mut self, id: &str, events: &mut Vec<Event>) -> Spent {
        let Some(status) = self.content.status_id(id) else {
            self.say(format!("There is no status `{id}`."));
            return Spent::Nothing;
        };
        if self.content.disease_status() == Some(status) {
            self.say(format!("`{id}` comes of a disease: try `infect:DISEASE`."));
            return Spent::Nothing;
        }
        let player = self.state.player;
        self.afflict(player, status, 1, None, events);
        Spent::NoTime
    }
}

/// Hit points back for `actor` over `seconds` at `rate` percent of the normal pace: its
/// progress ([`super::Actor::regen`]) grows by `rate` a second, and every
/// [`combat::regen_seconds`] × 100 of it is a hit point. Nothing at full health (the progress is
/// dropped) or dead.
fn heal(actor: &mut super::Actor, content: &Content, seconds: u64, rate: u32) {
    let Some(need) = need(actor, content) else {
        actor.regen = 0;
        return;
    };
    let max = actor.max_hp(content);
    let banked = u64::from(actor.regen).saturating_add(seconds.saturating_mul(u64::from(rate)));
    let gained = i32::try_from(banked / need).unwrap_or(i32::MAX);
    actor.hp = actor.hp.saturating_add(gained).min(max);
    actor.regen = if actor.hp >= max {
        0
    } else {
        u32::try_from(banked % need).unwrap_or(0)
    };
}

/// The progress `actor` needs for a hit point back ([`super::Actor::regen`]); `None` when it
/// heals no more: at full health, or dead.
fn need(actor: &super::Actor, content: &Content) -> Option<u64> {
    let toughness = actor.effective_stats(content).score(Stat::Toughness);
    (actor.hp > 0 && actor.hp < actor.max_hp(content))
        .then(|| u64::from(combat::regen_seconds(toughness)) * 100)
}
