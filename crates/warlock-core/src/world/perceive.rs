//! Perception, the turn's last phase (spec §5): recomputing what the player perceives, what of
//! the turn's events they saw (and the messages for them), the news they heard, and the weather
//! they see change. `world/mod.rs` has only the call from the turn pipeline.

use super::{ActorId, ActorKind, World, enter_room, fight, spell};
use crate::action::{AttackKind, Breach, DiseaseNews, Event, MissileEnd};
use crate::content::{CreatureId, TrapEffect};
use crate::geom::Pos;
use crate::noise;
use crate::perception::{Perception, VIEW_RADIUS};
use crate::sight::LineOfSight;
use crate::skills::Use;
use crate::weather::{self, Weather};

impl World {
    /// The weather now, which the player would see: on a level under the open sky.
    pub(super) fn weather_now(&self) -> Option<Weather> {
        let state = &self.state;
        state.current().kind.is_sky_exposed().then(|| {
            let over = state.weather_override.as_ref();
            weather::weather_at(state.seed, state.time, self.content.climate(), over)
        })
    }

    /// Whether the weather the player would see differs from what they saw at the last
    /// perception (so a rest perceives the change and says so).
    pub(super) fn weather_turned(&self) -> bool {
        self.weather_now() != self.perception.weather()
    }

    /// Creatures (not the player) the player sees.
    pub(super) fn creatures_in_view(&self) -> Vec<ActorId> {
        self.perception()
            .actors()
            .iter()
            .filter(|a| a.kind == ActorKind::Creature)
            .map(|a| a.id)
            .collect()
    }

    /// Creatures the player sees whose coming into view stops a rest ([`World::disturbs_rest`]).
    pub(super) fn watched_in_view(&self) -> Vec<ActorId> {
        let mut seen = self.creatures_in_view();
        seen.retain(|id| self.disturbs_rest(&self.state.actors[id]));
        seen
    }

    /// Phase 4: recomputes perception, updates memory, tells the player about a special room they
    /// are in for the first time, says what news they heard and how the weather changed, reports
    /// the attacks they perceived, and drops events they did not see.
    pub(super) fn perceive(&mut self, events: &mut Vec<Event>) {
        let now = self.state.time;
        // The ice of the moment (after a skip or on arrival), under the weather it had.
        self.update_ice();
        if self.state.weather_override.is_some_and(|o| o.until <= now) {
            self.state.weather_override = None;
        }
        self.burn_out();
        self.rot_away();
        let before = self.perception.weather();
        let dazzled = self.perception.dazzled();
        *self.perception = Perception::compute(&self.state, &self.content, &mut self.lights);
        self.glare_news(dazzled);
        self.learn_traps(events);
        let current = self.state.current_level.clone();
        if let Some(level) = self.state.levels.get_mut(&current) {
            self.perception.remember(level);
        }
        if let Some(text) = enter_room(&mut self.state, &self.content) {
            self.say(text);
        }
        self.hear();
        self.say_mutters();
        if let (Some(from), Some(to)) = (before, self.perception.weather()) {
            self.weather_news(from, to);
        }
        let reports = self.perceive_events(events);
        for (text, stops_rest) in reports {
            if stops_rest {
                self.say(text);
            } else {
                self.note(text);
            }
        }
        // What kills earned the player meanwhile (a creature their wound bled to death), and a
        // weapon skill a blocked blow raised (`fight.rs`).
        self.say_news();
    }

    /// Keeps the events the player perceived (dropping the rest) and returns the messages for
    /// the attacks, traps, deaths, status effects, diseases, and walls giving way among them,
    /// each with whether it stops a rest. A creature's attack is said if the player saw the
    /// attacker or the target, or is the target, and kept for presentation if they saw both (the
    /// player's own attacks were said, and only seen ones recorded, when made); a death is said
    /// and kept with the attack that caused it, if the player saw who died. The player's own
    /// death is always said, and settles whether they saw what killed them. A death of a status
    /// effect is said if the player sees where it happened ("The giant rat bleeds to death.").
    /// The player's own status effects are said as they begin (stopping a rest) and end (not
    /// stopping one); their diseases as they show and worsen (stopping a rest) and leave (not
    /// stopping one). A trap going off is said and kept if the player set it off, saw its cell,
    /// or saw any of its blast; what it did to someone if they are the player or the player saw
    /// them (each stopping a rest). A wall that gave way is kept if the player sees its cell, and
    /// one blasted open is said ("The wall collapses!"; digging says its own).
    pub(super) fn perceive_events(&mut self, events: &mut Vec<Event>) -> Vec<(String, bool)> {
        use fight::{End, Guard, Words, attack_message, death_message};
        let player = self.state.player;
        let content = self.content.clone();
        let mut reports: Vec<(String, bool)> = Vec::new();
        // What came last: an attack (its attacker, its target, whether the player saw the
        // target, and the attacker's creature type if the player saw that), or a trap (its
        // target and whether the player saw it).
        enum Last {
            Attack(ActorId, ActorId, bool, Option<CreatureId>),
            Trap(ActorId, bool),
        }
        let mut last: Option<Last> = None;
        let mut killer_seen: Option<Option<CreatureId>> = None;
        let mut collapsed = 0;
        let perception = &self.perception;
        let end = |id: ActorId, seen: bool| match self.actor_or_fallen(id) {
            _ if id == player => End::You,
            Some(a) if seen => End::Seen(content.creature(a.creature).name.as_str()),
            _ => End::Unseen,
        };
        // A missile is seen where it flies over cells the player sees: its path is cut to the
        // stretch from the first of them to the last, and where it was loosed is known only if
        // the player saw it leave. A burning one is light itself: it is seen wherever it flies
        // in the player's line of sight, in the dark too.
        let mut los: Option<LineOfSight> = None;
        let line_of_sight = |los: &mut Option<LineOfSight>| {
            if los.is_none() {
                let level = self.state.current();
                let viewer = self.state.player().pos;
                *los = Some(LineOfSight::compute(viewer, VIEW_RADIUS, |p| {
                    level.is_opaque(&content, p)
                }));
            }
        };
        for event in events.iter_mut() {
            // A spell effect is light itself too (`spell.rs`).
            if let Event::Spell { .. } = event {
                line_of_sight(&mut los);
                let los = los.as_ref().expect("computed");
                spell::perceive_spell(event, |p| perception.is_visible(p) || los.contains(p));
                continue;
            }
            if let Event::Missile {
                object,
                from,
                path,
                end,
            } = event
            {
                let burning = content.object(*object).burning.is_some();
                if burning {
                    line_of_sight(&mut los);
                }
                let in_sight = |p: Pos| burning && los.as_ref().is_some_and(|l| l.contains(p));
                let seen = |p: &Pos| perception.is_visible(*p) || in_sight(*p);
                let (Some(first), Some(last)) =
                    (path.iter().position(seen), path.iter().rposition(seen))
                else {
                    path.clear();
                    continue;
                };
                if last + 1 < path.len() {
                    *end = MissileEnd::Unseen;
                }
                path.truncate(last + 1);
                path.drain(..first);
                if first > 0 || from.is_some_and(|f| !perception.sees_actors_at(f)) {
                    *from = None;
                }
            }
        }
        // Whether the player perceived each trap that went off, in order: they set it off, see
        // its cell, or see any of its blast (a spell before the next trap).
        let mut sprung_seen = Vec::new();
        for (i, event) in events.iter().enumerate() {
            if let Event::TrapSprung { at, victim, .. } = *event {
                let blast = (events[i + 1..].iter())
                    .take_while(|e| !matches!(e, Event::TrapSprung { .. }))
                    .any(|e| matches!(e, Event::Spell { cells, .. } if !cells.is_empty()));
                sprung_seen.push(victim == player || perception.sees_actors_at(at) || blast);
            }
        }
        let mut sprung_seen = sprung_seen.into_iter();
        events.retain(|e| match *e {
            Event::ActorMoved { actor, from, to } => {
                actor == player || perception.sees_actors_at(from) || perception.sees_actors_at(to)
            }
            Event::DoorOpened { pos }
            | Event::DoorClosed { pos }
            | Event::DoorKicked { pos }
            | Event::DoorBroken { pos } => perception.is_visible(pos),
            Event::WallBroken { pos, cause } => {
                let seen = perception.is_visible(pos);
                if seen && cause == Breach::Blast {
                    collapsed += 1;
                }
                seen
            }
            Event::LevelChanged { .. } | Event::Talked(_) | Event::VoiceThrown { .. } => true,
            Event::Missile { ref path, .. } => !path.is_empty(),
            Event::Spell { ref cells, .. } => !cells.is_empty(),
            Event::Attacked {
                attacker,
                target,
                kind,
                from,
                to,
                strike,
                with,
                damage,
            } => {
                let sees_attacker = attacker == player || perception.sees_actors_at(from);
                let sees_target = target == player || perception.sees_actors_at(to);
                let creature = (self.actor_or_fallen(attacker)).map(|a| a.creature);
                let by = creature.filter(|_| sees_attacker);
                last = Some(Last::Attack(attacker, target, sees_target, by));
                if attacker == player {
                    return true;
                }
                if let Some(me) = self.actor_or_fallen(attacker) {
                    let words = Words::of(me, kind, with, damage, &content);
                    let guard = Guard::of(self.actor_or_fallen(target), &content, &self.state);
                    let (a, t) = (end(attacker, sees_attacker), end(target, sees_target));
                    let said = attack_message(a, t, &words, kind, strike, guard);
                    reports.extend(said.map(|text| (text, true)));
                }
                sees_attacker && sees_target
            }
            Event::FellThroughIce { actor, at } => {
                // The player's own fall was said as it happened.
                if actor == player {
                    return true;
                }
                let seen = perception.sees_actors_at(at);
                if let Some(me) = self.actor_or_fallen(actor).filter(|_| seen) {
                    let name = &content.creature(me.creature).name;
                    reports.push((format!("The {name} falls through the ice!"), true));
                }
                seen
            }
            Event::Status {
                actor,
                status,
                began,
            } => {
                let def = content.status(status);
                let text = if began { &def.start } else { &def.end };
                if let Some(text) = text.clone().filter(|_| actor == player) {
                    reports.push((text, began));
                }
                actor == player
            }
            Event::Disease {
                actor,
                disease,
                news,
            } => {
                let def = content.disease(disease);
                let (text, stops) = match news {
                    DiseaseNews::Onset => (def.onset.as_str(), true),
                    DiseaseNews::Worse(n) => (def.symptom(n), true),
                    DiseaseNews::Recovered => (def.recovered.as_str(), false),
                };
                if actor == player {
                    reports.push((text.to_string(), stops));
                }
                actor == player
            }
            Event::Died {
                actor,
                at,
                of: Some(status),
            } => {
                if actor == player {
                    reports.push((death_message(End::You), true));
                    return true;
                }
                let seen = perception.sees_actors_at(at);
                if let (true, Some(dead)) = (seen, self.actor_or_fallen(actor)) {
                    let name = &content.creature(dead.creature).name;
                    let dies = content.status(status).dies.as_deref().unwrap_or("dies");
                    reports.push((format!("The {name} {dies}."), true));
                }
                seen
            }
            Event::Died { actor, .. } => match last {
                None => actor == player,
                Some(Last::Attack(_, _, _, by)) if actor == player => {
                    killer_seen = Some(by);
                    reports.push((death_message(End::You), true));
                    true
                }
                // A trap's victim: the death records the trap, not who was seen.
                Some(Last::Trap(..)) if actor == player => {
                    reports.push((death_message(End::You), true));
                    true
                }
                // A death by the player's hand was said with their blow, and recorded only if
                // they saw it.
                Some(Last::Attack(attacker, ..)) if attacker == player => true,
                Some(Last::Attack(_, target, seen, _) | Last::Trap(target, seen)) => {
                    let seen = seen && target == actor;
                    if seen {
                        reports.push((death_message(end(actor, true)), true));
                    }
                    seen
                }
            },
            Event::TrapSprung { trap, at, victim } => {
                last = None;
                let seen = sprung_seen.next().unwrap_or(false);
                if seen {
                    let def = content.trap(trap);
                    reports.extend(def.message.clone().map(|text| (text, true)));
                    let name = (self.actor_or_fallen(victim))
                        .map(|a| content.creature(a.creature).name.as_str());
                    let creature = name.filter(|_| perception.sees_actors_at(at));
                    let text = match (&def.effect, victim == player, creature) {
                        (TrapEffect::Pit { .. }, true, _) => Some("You fall into a pit.".into()),
                        (TrapEffect::Pit { .. }, false, Some(name)) => {
                            Some(format!("The {name} falls into a pit."))
                        }
                        (TrapEffect::Teleport, true, _) => {
                            Some("You are wrenched elsewhere.".into())
                        }
                        (TrapEffect::Teleport, false, Some(name)) => {
                            Some(format!("The {name} vanishes!"))
                        }
                        _ => None,
                    };
                    reports.extend(text.map(|text| (text, true)));
                }
                seen
            }
            Event::TrapStruck {
                trap,
                target,
                to,
                strike,
            } => {
                let seen = target == player || perception.sees_actors_at(to);
                last = Some(Last::Trap(target, seen));
                let def = content.trap(trap);
                // A fall is said with the trap ("You fall into a pit.").
                if seen && !matches!(def.effect, TrapEffect::Pit { .. }) {
                    let words = Words::new(&def.noun, &def.verb);
                    let guard = Guard::of(self.actor_or_fallen(target), &content, &self.state);
                    let said = attack_message(
                        End::Trap(&def.noun),
                        end(target, true),
                        &words,
                        AttackKind::Melee,
                        Some(strike),
                        guard,
                    );
                    reports.extend(said.map(|text| (text, true)));
                }
                seen
            }
        });
        if let (Some(by), Some(death)) = (killer_seen, self.state.death.as_mut())
            && death.cause.is_none()
        {
            death.by = by;
        }
        match collapsed {
            0 => {}
            1 => reports.push(("The wall collapses!".into(), true)),
            _ => reports.push(("The walls collapse!".into(), true)),
        }
        reports
    }

    /// A message for each sound the player perceives that is news ([`noise::news`]): a patrol
    /// heard turn after turn is said once, not every turn. Messages stop a rest. A voice is said
    /// with its words, if any ([`World::say_mutters`]).
    pub(super) fn hear(&mut self) {
        let turn = self.state.turn;
        let news = noise::news(
            &mut self.state.recent_sounds,
            self.perception.sounds(),
            turn,
        );
        let viewer = self.player_pos();
        let mut said: Vec<String> = Vec::new();
        for sound in news.iter().filter(|s| s.kind != noise::SoundKind::Voice) {
            let text = sound.message(viewer);
            if !said.contains(&text) {
                self.say(text.clone());
                said.push(text);
            }
        }
        // Hearing something new trains listening.
        if !news.is_empty() && self.trains(Use::Hear) {
            self.train(Use::Hear);
        }
    }

    /// A message when the weather the player sees changes under the open sky (never on
    /// arriving there: the status bar shows it). Rain starting or growing heavier stops a rest;
    /// clouds gathering or parting and rain easing or stopping are only logged.
    pub(super) fn weather_news(&mut self, from: Weather, to: Weather) {
        use Weather::{Clear, HeavyRain, Overcast, Rain};
        let text = match (from, to) {
            (Clear, Overcast) => "Clouds gather.",
            (Overcast, Clear) => "The sky clears.",
            (Clear | Overcast, Rain) => "It starts to rain.",
            (Clear | Overcast, HeavyRain) => "It starts to pour.",
            (Rain, HeavyRain) => "The rain grows heavy.",
            (HeavyRain, Rain) => "The rain eases.",
            (Rain | HeavyRain, Overcast) => "The rain stops.",
            (Rain | HeavyRain, Clear) => "The rain stops and the sky clears.",
            _ => return,
        };
        if to.is_rain() && to > from {
            self.say(text);
        } else {
            self.note(text);
        }
    }
}
