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
//!   it ([`World::pass_time`]): its effects tick (costing hit points, and mana for a status with
//!   `mana_damage`) and ease, and it gets hit points back at its healing rate and mana at its
//!   mana rate (`world/mana.rs`), in time order, so it makes no difference how the time is cut up
//!   (by a long journey or many short actions). Diseases take their turns and stat damage mends
//!   at their own moments too (`world/disease.rs`), and so do the points an alternative resource
//!   gains or loses over time (`world/resource.rs`: one that decays to 0 and ends there ends at
//!   that moment). At one moment a hit point back, a point of mana back, and a resource's point
//!   come first, then the ticks, then the easing, then the player's hunger, then a point of stat
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
//! - **Endings**: a status that ends, worn off or cured, may be followed by another (`follows`;
//!   buffs, hexes, and cooldowns are `world/buff.rs`).

use super::fight::Harm;
use super::{ActorId, Spent, World, disease, mana, resource, rot};
use crate::action::{AttackKind, Event};
use crate::combat::{self, Blow, Strike};
use crate::content::Effect as ItemEffect;
use crate::content::{Affliction, Content, DiseaseId, OnHit, StatusId};
use crate::hunger::Hunger;
use crate::stats::Stat;
use crate::status::{Added, Effect};
use crate::time::{Duration, GameTime};

/// What happened to an actor at one moment while time passed.
struct Moment {
    /// Hit points its effects cost then.
    damage: i32,
    /// Mana its effects drain then (`mana_damage`).
    drained: i32,
    /// The first status that ticked (the most important) and who caused it: a death is by it.
    by: Option<(StatusId, Option<ActorId>)>,
    /// Whether a status that bleeds ticked (blood drips).
    bled: bool,
}

impl World {
    /// Actor `id` comes to suffer `severity` degrees of status `status`, caused by `by` (if
    /// anyone): added to what it has by the status's stacking rule. A status that is a kind of
    /// damage (poison; spec §8 "Damage types") meets the actor's resistance to it: an immune
    /// actor never suffers it, and a resistance takes its share off the dose (rounded to the
    /// nearest, half up; a dose shrunk to nothing is no dose). The player is told when it
    /// begins. A unique item the actor wears that wards off the status (spec §8 "Magic items":
    /// Ward) refuses it, and the player is told so.
    pub(crate) fn afflict(
        &mut self,
        id: ActorId,
        status: StatusId,
        severity: u16,
        by: Option<ActorId>,
        events: &mut Vec<Event>,
    ) {
        // Immune by its type (spec §8 "Status effects": an animated thing never bleeds).
        if (self.state.actors.get(&id))
            .is_some_and(|a| self.content.creature(a.creature).immune.contains(&status))
        {
            return;
        }
        // Chest-wise (spec §8 "Mimics"): a mimic's grip never holds it.
        if self.content.status(status).hold.is_some() && self.slips(id, by) {
            return;
        }
        // A unique item's Ward (spec §8 "Magic items"): never taken, and the player is told.
        let ward = (self.state.actors.get(&id))
            .and_then(|a| a.ward(&self.content, status))
            .map(str::to_string);
        if let Some(message) = ward {
            if id == self.state.player {
                self.say(message);
            }
            return;
        }
        let def = self.content.status(status);
        let severity = match def.damage_type {
            None => severity,
            Some(kind) => {
                let percent = self.resistances_of(id).get(Some(kind));
                let dose = crate::damage::resist(i32::from(severity), percent);
                match u16::try_from(dose) {
                    Ok(dose) if dose > 0 => dose,
                    _ => return,
                }
            }
        };
        // A hold lasts its turns, drawn (`hold.rs`).
        let severity = self.hold_turns(status, severity);
        let def = self.content.status(status);
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        let added = actor.effects.add(status, def, severity, by);
        // One given during the sufferer's own action is not eased by it (`weapon_ability.rs`).
        if def.per_action && self.acting == Some(id) {
            self.fresh.push((id, status));
        }
        if added == Added::Began && id == self.state.player {
            events.push(Event::Status {
                actor: id,
                status,
                began: true,
            });
        }
        // Rage makes a berserker (`berserker.rs`).
        self.settle_stance(id);
    }

    /// Cures actor `id` with `cure`, a means named in the statuses' and diseases' `cured_by`
    /// (`bandage`, `remove_disease`): every status of its that the means cures ends (and what
    /// `follows` it begins), and every disease (showing or not). Returns whether any did. Eating
    /// a remedy applies its cures (`world/eat.rs`); the Heal skill's bandaging (#132), potions,
    /// and priests will too.
    pub(crate) fn cure(&mut self, id: ActorId, cure: &str, events: &mut Vec<Event>) -> bool {
        let content = self.content.clone();
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
        let ended: Vec<Effect> = (cured.iter())
            .filter_map(|&status| actor.effects.get(status).copied())
            .collect();
        for &status in &cured {
            actor.effects.remove(status);
        }
        self.ended(id, &ended, events);
        for &disease in &healed {
            self.end_disease(id, disease, events);
        }
        !cured.is_empty() || !healed.is_empty()
    }

    /// A blow by `attacker` with an attack of `kind` hit `target` and left it alive: the
    /// statuses its wound causes (bleeding from a big hit or a confirmed critical), then the
    /// `on_hit` effects of the attack, each by its chance: its natural attack's, then `made`,
    /// what the attack's profile carries (a unique weapon's Inflict power).
    pub(super) fn after_blow(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        kind: AttackKind,
        (blow, made): (&Blow, &[OnHit]),
        events: &mut Vec<Event>,
    ) {
        if !self.state.actors.contains_key(&target) {
            return;
        }
        self.wounded(target, blow, Some(attacker), events);
        // Staggering blow (spec §8 "Talents"): a critical confirmed in melee staggers.
        let critical = matches!(blow.strike, Strike::Hit { critical: true });
        let staggers = (self.actor_or_fallen(attacker))
            .and_then(|a| a.talent_mods(&self.content).critical_status);
        if let Some(status) = staggers.filter(|_| critical && kind == AttackKind::Melee) {
            self.afflict(target, status, 1, Some(attacker), events);
        }
        let mut hits = self.on_hit_of(attacker, kind);
        hits.extend_from_slice(made);
        for hit in hits {
            let lands = hit.chance >= 100 || self.state.rng.chance(u32::from(hit.chance));
            if !lands {
                continue;
            }
            match hit.what {
                Affliction::Status { status, severity } => {
                    self.afflict(target, status, severity, Some(attacker), events);
                }
                Affliction::Hold { status, .. } => {
                    self.afflict(target, status, 1, Some(attacker), events);
                }
                Affliction::Disease(disease) => {
                    self.expose(target, disease, Some(attacker));
                }
                Affliction::Drain(drain) => self.drain(target, drain, Some(attacker), events),
            }
        }
    }

    /// `target` took `blow`, dealt by `by` (none for a trap's, spec §8 "Traps"), and lives: the
    /// statuses its wound causes (bleeding from a big hit or a confirmed critical, of a kind
    /// that wounds that way: slashing and piercing bleed, a club and a fireball do not; spec §8
    /// "Damage types").
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
        // Clotting (spec §8 "Talents"): a big hit wounds no more, a critical a degree less;
        // Thin-skinned (spec §8 "Birth sign"): a big hit that wounds, degrees more.
        let mods = victim.talent_mods(&content);
        let (clots, thin) = (mods.clotting, if big { mods.wound_more } else { 0 });
        for (status, def) in content.status_ids() {
            let degrees = (def.wound.as_ref())
                .filter(|w| w.caused_by(blow.kind))
                .map_or(0, |w| match (clots, critical) {
                    (false, _) => w.degrees(critical, big),
                    (true, false) => 0,
                    (true, true) => w.degrees(true, big).saturating_sub(1),
                });
            if degrees > 0 {
                let degrees = degrees.saturating_add(thin);
                self.afflict(target, status, degrees, by, events);
            }
        }
    }

    /// What a hit by `attacker` with an attack of `kind` may cause besides its damage: its
    /// natural attack's `on_hit` when it fights with it (a blow with no weapon in hand). A unique
    /// weapon's Inflict power comes with the attack's profile instead ([`World::after_blow`]).
    fn on_hit_of(&self, attacker: ActorId, kind: AttackKind) -> Vec<OnHit> {
        let Some(me) = self.actor_or_fallen(attacker) else {
            return Vec::new();
        };
        let def = self.content.creature(me.creature);
        match (kind, def.attack.as_ref()) {
            (AttackKind::Melee, Some(natural)) if me.weapon(&self.content).is_none() => {
                natural.on_hit.clone()
            }
            // A creature's throw (`hurl.rs`); the player's throws are their objects'.
            (AttackKind::Thrown, _) if attacker != self.state.player => (def.throw.as_ref())
                .map(|t| t.attack.on_hit.clone())
                .unwrap_or_default(),
            _ => Vec::new(),
        }
    }

    /// How fast actor `id` gets hit points back now, in percent of the normal pace (spec §8
    /// "Healing"): the slowest `regen` of its statuses and of the diseases that show (100
    /// without any), times the resting bonus while the player rests, times the `regen` of a
    /// special room it stands in (a sanctuary: three times; spec §7 "Special rooms"), times what
    /// the regeneration affixes of what it wears and wields add (spec §8 "Magic items": +50 % is
    /// half again as fast), with its talents' (Healthy; Sun-blessed's under the open sun). The one place that decides it. A Starving player gets none back
    /// (spec §8 "Hunger"); disease (#139) slows it through Diseased's `regen` and its own.
    #[cfg(test)]
    pub(crate) fn healing_rate(&self, id: ActorId) -> u32 {
        self.healing_rate_at(id, self.state.time)
    }

    /// [`World::healing_rate`] at `now`, a moment of [`World::live`]: the player heals the
    /// flavour's `birthday_healing` percent as fast on their birthday (spec §8 "Birth sign").
    pub(crate) fn healing_rate_at(&self, id: ActorId, now: GameTime) -> u32 {
        let Some(actor) = self.state.actors.get(&id) else {
            return 0;
        };
        // Starving, no hit point comes back; Another head (a Hydra's pick) heals on.
        if actor.hunger(&self.content) == Some(Hunger::Starving)
            && !(!actor.talents.is_empty() && actor.talent_mods(&self.content).starving_heals)
        {
            return 0;
        }
        let statuses = actor.effects.regen(&self.content);
        let mut rate = u32::from(statuses.min(actor.diseases.regen(&self.content)));
        if self.resting && id == self.state.player {
            // Resting's pace, and half again with Fast healer (spec §8 "Talents").
            let faster = actor.talent_mods(&self.content).resting_percent;
            let resting = u32::from(self.content.healing().resting);
            let resting = resting * u32::try_from((100 + faster).max(0)).unwrap_or(0) / 100;
            rate = rate * resting / 100;
        }
        let room = (self.state.levels.get(&actor.level)).map_or(100, |level| {
            level.effects_at(&self.content, actor.pos).regen()
        });
        let room = room * self.shrine_pace(actor) / 100;
        let rate = rate * self.room_pace(actor, room) / 100;
        // Then the regeneration of what it wears and wields, and its talents' (Healthy).
        let talents = actor.talent_mods(&self.content);
        let faster =
            actor.mods(&self.content).get(ItemEffect::Regeneration) + talents.healing_percent;
        let percent = u32::try_from((100 + faster).max(0)).unwrap_or(0);
        rate * percent / 100 * self.birthday_healing(id, now) / 100
    }

    /// `seconds` of game time from `start` pass for actors `ids` (on the current level, in
    /// schedule order): each takes what the special room it stands in gives
    /// ([`World::dwell`]), then lives through them ([`World::live`]). Some may die.
    pub(super) fn pass_time(
        &mut self,
        ids: impl Iterator<Item = ActorId>,
        start: GameTime,
        seconds: u64,
        events: &mut Vec<Event>,
    ) {
        let dwelling = self.dwelling_here();
        for id in ids {
            if dwelling {
                self.dwell(id, events);
            }
            self.live(id, start, seconds, events);
        }
    }

    /// Whether `seconds` of game time from `start` change nothing for creature `id` (not the
    /// player): at full hit points and mana, with no effects, diseases, stat damage, resources,
    /// hunger, birthday, or food in its pack to rot. Then [`World::live`] only empties its
    /// healing banks, as it would, without working out its paces (a full floor's creatures
    /// live through every moment of a rest).
    fn untouched_by(&mut self, id: ActorId, start: GameTime, seconds: u64) -> bool {
        let content = &self.content;
        let Some(actor) = self.state.actors.get(&id) else {
            return false;
        };
        let quiet = id != self.state.player
            && actor.hp > 0
            && actor.effects.is_empty()
            && actor.diseases.is_empty()
            && actor.damage.is_none()
            && actor.resources.is_empty()
            && actor.satiation.is_none()
            && rot::until_rots(&actor.inventory, content, start).is_none()
            && self
                .until_birthday_turns(id, start)
                .is_none_or(|b| b >= seconds)
            && actor.is_whole(content);
        if quiet && let Some(actor) = self.state.actors.get_mut(&id) {
            actor.regen = 0;
            actor.mana_regen = 0;
        }
        quiet
    }

    /// `seconds` of game time from `start` pass for actor `id`: its effects tick and ease, it
    /// heals and gets mana back, (the player) hungers, its stat damage mends, its diseases take their turns,
    /// (the player) its stats get their daily check, and the food in its pack rots away, each at the moment it happens, until the time is up or it
    /// dies.
    fn live(&mut self, id: ActorId, start: GameTime, seconds: u64, events: &mut Vec<Event>) {
        // Appraise's hourly look at what the player carries (spec §8 "Skills").
        if id == self.state.player {
            self.appraise_hours(start, seconds);
        }
        // Nothing to live through (and nothing in its pack that rots).
        if self.untouched_by(id, start, seconds) {
            return;
        }
        let content = self.content.clone();
        // What rotted away while it was frozen (on another level, or over a dev skip) goes first.
        self.rot_pack(id, start);
        if id == self.state.player {
            self.appraise_carried(seconds);
        }
        let mut passed = 0;
        while passed < seconds {
            let now = start + Duration::seconds(passed);
            let rate = self.healing_rate_at(id, now);
            let birthday = self.until_birthday_turns(id, now);
            let mana_rate = self.mana_rate(id);
            let hungrier = self.until_hungrier(id);
            let paces = self.resource_paces(id);
            let stat_check = self.until_stat_check(id, now);
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
                && crate::resource::until_next(&actor.resources, &content, &paces)
                    .is_none_or(|r| r > left)
                && birthday.is_none_or(|b| b >= left)
                && stat_check.is_none_or(|c| c > left)
            {
                heal(actor, &content, left, rate);
                mana::regain(actor, &content, left, mana_rate);
                crate::resource::pass(&mut actor.resources, &content, &paces, left);
                return;
            }
            // Up to the next moment something happens: a tick, an easing, a hit point or a point
            // of mana back, a change in its hunger (a new state, a hit point lost to starving), a disease's
            // turn, a point of stat damage mended, or food in its pack rotting away.
            let mut step = hungrier.map_or(left, |h| h.min(left));
            // The birthday's start or end: the healing pace changes then.
            if let Some(b) = birthday {
                step = step.min(b.max(1));
            }
            if let Some(next) = disease::until_next(actor) {
                step = step.min(next);
            }
            // The daily stat check at 06:00 (`stat_training.rs`).
            if let Some(c) = stat_check {
                step = step.min(c.max(1));
            }
            if let Some(r) = rots {
                step = step.min(r.max(1));
            }
            let talents = actor.talent_mods(&content);
            let eases = |e: &crate::status::Effect| {
                let def = content.status(e.status);
                def.eases.map(|every| talents.eases_every(e.status, every))
            };
            for e in actor.effects.iter() {
                let def = content.status(e.status);
                step = (e.until_tick(def).into_iter())
                    .chain(e.until_eases_every(eases(e)))
                    .fold(step, u64::min);
            }
            if let Some(need) = need(actor, &content)
                && rate > 0
            {
                let bank = u64::from(actor.regen);
                step = step.min(need.saturating_sub(bank).div_ceil(u64::from(rate)).max(1));
            }
            if let Some(next) = mana::until_regained(actor, &content, mana_rate) {
                step = step.min(next);
            }
            if let Some(next) = crate::resource::until_next(&actor.resources, &content, &paces) {
                step = step.min(next);
            }
            passed += step;
            heal(actor, &content, step, rate);
            mana::regain(actor, &content, step, mana_rate);
            // A resource's point gained or lost (spec §8 "Resources"); one that decays to 0 and
            // ends there ends now, and the status it brought with it (the rage's Raging).
            let mut gone = Vec::new();
            for ended in crate::resource::pass(&mut actor.resources, &content, &paces, step) {
                gone.extend(resource::ended(actor, &content, ended, events));
            }
            let mut moment = Moment {
                damage: 0,
                drained: 0,
                by: None,
                bled: false,
            };
            // Unstoppable (spec §8 "Class abilities"): while raging, some statuses tick nothing.
            let numb: &[StatusId] = match (&talents.unstoppable, talents.rage_status(&content)) {
                (Some(unstoppable), Some(rage)) if actor.effects.has(rage) => &unstoppable.numb,
                _ => &[],
            };
            for e in actor.effects.iter_mut() {
                let def = content.status(e.status);
                let every = eases(e);
                let (ticks, eased) = e.advance_every(def, every, step);
                if ticks > 0 && !numb.contains(&e.status) {
                    let cost = u64::from(def.damage) * ticks;
                    moment.damage += i32::try_from(cost).unwrap_or(i32::MAX);
                    let drain = u64::from(def.mana_damage) * ticks;
                    let drain = i32::try_from(drain).unwrap_or(i32::MAX);
                    moment.drained = moment.drained.saturating_add(drain);
                    moment.by.get_or_insert((e.status, e.by));
                    moment.bled |= def.wound.is_some();
                }
                e.severity = e
                    .severity
                    .saturating_sub(u16::try_from(eased).unwrap_or(u16::MAX));
            }
            mana::drain(actor, moment.drained);
            let mut spent = actor.effects.drop_spent();
            spent.extend(gone);
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
            self.train_shaking_off(id, &spent);
            self.ended(id, &spent, events);
            if let Some(before) = hunger {
                self.hunger_news(before);
            }
            for disease in due {
                if !self.disease_turn(id, disease, when, events) {
                    return;
                }
            }
            self.rot_pack(id, when);
            self.daily_stat_check(id, when);
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
        // A status a resource brings lasts while that is held (spec §8 "Resources"): Raging
        // comes of Rage.
        if let Some(r) = (self.content.resources().iter()).find(|r| r.status == Some(status)) {
            self.say(format!(
                "`{id}` comes of a resource: try `resource:{}`.",
                r.id
            ));
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
