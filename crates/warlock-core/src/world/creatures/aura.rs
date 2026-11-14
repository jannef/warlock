//! A creature's presence on its turn (design `docs/design/monster-dread.md` §5.4 G4, §9; #1128;
//! the data is `content/aura.rs`):
//!
//! - **Innate statuses** ([`wear_innate`]): an actor is made with its type's `innate` statuses,
//!   and as each turn of its starts they are put back (a degree, the status's own stacking; one
//!   that refreshes starts its wear over), so a cure or the clock never takes them for long.
//!   Perception leaves them out of the look line (`perception/actors.rs`): the kind always has them.
//! - **The aura** ([`World::aura_turn`]): as a creature's turn starts, after its innate statuses
//!   and before it decides, unless its cooldown status (`aura`) is on: every hostile within the
//!   aura's radius (the fov's round radius) that it reaches, by line of sight from the bearer (no
//!   light needed) or, a heard aura, by the hearing rules as a voice of its `loudness` made where
//!   the bearer stands (`noise::hearers`: through walls and doors, Listen's keener threshold at a
//!   door included), in actor id order, makes the aura's check (its stat's effective modifier,
//!   the talents' checks of the stat, and the player's Knowledge of the bearer; two draws) or
//!   takes its `severity` degrees, the aura's `damage_type` resistance taking its share, caused by
//!   the bearer (`World::afflict`: immunity, wards, and the status's own `damage_type` apply). A
//!   check made trains the stat as a drain resisted does. If anyone was in reach, the bearer takes
//!   `ceil(every / 6)` degrees of `aura`. No time is spent, and no noise is made (a wail is its
//!   cry). An ally of the player's has none (the auras are the dread's).
//! - **Words**: when the player takes the status anew (it begins), the aura's `touch` is said
//!   first if they see the bearer ("A chill comes off the wraith."), else its `unseen` line if it
//!   has one ("A wail rises somewhere below."); the status's `start` follows as any.
//! - **The cry** ([`World::cry`]): after a turn that was not a fight (no attack, summoning,
//!   breath, or cast), a creature with a `cry` cries with a chance of 6 in `every` (one draw of
//!   the gameplay stream, on its own turn only): a noise of the cry's kind and loudness where it
//!   stands ([`Act::Cry`]), heard by the hearing rules ("You hear a hiss.", the `sound` layer).

use super::ai::Plan;
use crate::action::Event;
use crate::check::check;
use crate::content::{Content, Stance, the};
use crate::fov::radius2;
use crate::noise::{self, Act, Noise, SoundKind};
use crate::sight::LineOfSight;
use crate::text::capitalized;
use crate::world::{Actor, ActorId, World};

/// Puts `actor`'s innate statuses on it (module docs): those it lacks, and those that refresh.
pub(in crate::world) fn wear_innate(actor: &mut Actor, content: &Content) {
    let def = content.creature(actor.creature);
    for &status in &def.innate {
        let sdef = content.status(status);
        let refreshes = sdef.stacking == crate::content::Stacking::Refresh;
        if refreshes || actor.effects.get(status).is_none() {
            actor.effects.add(status, sdef, 1, None);
        }
    }
}

impl World {
    /// The start of creature `id`'s turn: its innate statuses back, then its aura (module docs).
    pub(super) fn aura_turn(&mut self, id: ActorId, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let Some(me) = self.state.actors.get_mut(&id) else {
            return;
        };
        wear_innate(me, &content);
        let Some(aura) = content.creature(me.creature).aura.as_ref() else {
            return;
        };
        if me.ally.is_some() || me.effects.get(aura.cooldown).is_some() {
            return;
        }
        let (pos, level) = (me.pos, me.level.clone());
        let reach2 = radius2(aura.radius);
        let near: Vec<ActorId> = (self.state.actors.values())
            .filter(|a| a.level == level && a.id != id && a.pos.dist2(pos) <= reach2)
            .filter(|a| !a.effects.hides(&content))
            .map(|a| a.id)
            .filter(|&a| self.stance(id, a) == Stance::Hostile)
            .collect();
        if near.is_empty() {
            return;
        }
        let reached: Vec<ActorId> = match aura.heard {
            Some(loudness) => {
                let voice = Noise {
                    kind: SoundKind::Voice,
                    at: pos,
                    loudness,
                    maker: Some(id),
                };
                let hear = noise::hearers(&self.state, &content, voice);
                (near.into_iter())
                    .filter(|a| hear.iter().any(|(h, _)| h == a))
                    .collect()
            }
            None => {
                let current = self.state.current();
                let los =
                    LineOfSight::compute(pos, aura.radius, |p| current.is_opaque(&content, p));
                (near.into_iter())
                    .filter(|a| los.contains(self.state.actors[a].pos))
                    .collect()
            }
        };
        if reached.is_empty() {
            return;
        }
        self.afflict(id, aura.cooldown, aura.degrees, None, events);
        for target in reached {
            self.aura_touches(id, target, events);
        }
    }

    /// Creature `id`'s aura reaches `target`: its check, then the status (module docs).
    fn aura_touches(&mut self, id: ActorId, target: ActorId, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let (Some(me), Some(them)) = (self.state.actors.get(&id), self.state.actors.get(&target))
        else {
            return;
        };
        let def = content.creature(me.creature);
        let Some(aura) = def.aura.as_ref() else {
            return;
        };
        if let Some((stat, difficulty)) = aura.check {
            let modifier = (them.check_modifier(&content, stat))
                .saturating_add(self.resisting(target, Some(id)));
            if check(&mut self.state.rng, modifier, difficulty) {
                self.train_resisting(target, stat);
                return;
            }
        }
        let severity = match aura.kind {
            None => aura.severity,
            Some(kind) => {
                let percent = self.resistances_of(target).get(Some(kind));
                let dose = crate::damage::resist(i32::from(aura.severity), percent);
                match u16::try_from(dose) {
                    Ok(dose) if dose > 0 => dose,
                    _ => return,
                }
            }
        };
        let player = target == self.state.player;
        let had =
            (self.state.actors.get(&target)).and_then(|a| a.effects.get(aura.status).copied());
        let mut made = Vec::new();
        self.afflict(target, aura.status, severity, Some(id), &mut made);
        let began = (made.iter())
            .any(|e| matches!(e, Event::Status { actor, began: true, .. } if *actor == target));
        if player && had.is_none() && began {
            let at = self.state.actors[&id].pos;
            let line = if self.perception.sees_actors_at(at) {
                Some(aura.touch.replace("{the}", &the(&def.name)))
            } else {
                aura.unseen.clone()
            };
            if let Some(line) = line {
                self.say(capitalized(&line));
            }
        }
        events.extend(made);
    }

    /// Creature `id` did `plan` on its turn; not fighting, it may cry (module docs).
    pub(super) fn cry(&mut self, id: ActorId, plan: Plan) {
        if matches!(
            plan,
            Plan::Attack { .. } | Plan::Summon | Plan::Breathe(_) | Plan::Cast { .. }
        ) {
            return;
        }
        let Some(me) = self.state.actors.get(&id) else {
            return;
        };
        let Some(cry) = self.content.creature(me.creature).cry else {
            return;
        };
        if self.state.rng.below(cry.every) >= crate::content::CRY_TURN {
            return;
        }
        self.make_noise(id, Act::Cry(cry.kind));
    }
}
