//! Free blows (spec §8 "Free blows"): the blows a melee attack may bring beyond its first, and
//! the blows others get in. [`World::second_blow`] (`fight.rs`) is the one place each is
//! decided, on its [`Cue`]:
//!
//! - the **second blow** of an attack, at its target: the off hand's, or Sword and Board's
//!   shield bash after a hit (spec §8 "Fighting styles");
//! - a **cleave**: when an attack's blows kill its target, a blow at another hostile beside the
//!   striker, the next clockwise from where the target fell; one per attack, or with great
//!   cleave another after each cleave that kills;
//! - a **riposte**: when a blow of a melee attack missed by its target's Parry, a blow back at
//!   the attacker once the attack is over;
//! - an **opportunist's blow**: a hostile stepping away from beside the striker is struck as it
//!   turns to go, before it leaves its cell (it may not live to take the step).
//!
//! Each is a blow like any other (its own event, blood, wounds, kills, and marks), free (no time,
//! no exertion), and silent but the opportunist's, an attack of its own (the others follow a blow
//! that made the noise of the fight). A free blow brings none of its own but a cleave after a
//! cleave, so free blows never chain past the hostiles around. The player's are said as they
//! strike; what they strike in another's turn (a riposte, an opportunist's blow) waits until that
//! turn's reports have been said, so it never comes before what it answers.
//!
//! Who gets which is switched per actor ([`FreeBlows`], read through
//! [`World::free_blows_of`]): talents and class abilities switch them on. A trigger that does not
//! fire draws nothing from the gameplay stream, so fights without free blows are unchanged.

use super::{Actor, ActorId, World};
use crate::action::{AttackKind, Event};
use crate::combat::{Blow, Category, Profile, Strike};
use crate::content::{ObjectId, Stance};
use crate::geom::{Dir, Pos};
use serde::{Deserialize, Serialize};

/// The free blows an actor is given (spec §8 "Free blows"): the switches talents and class
/// abilities turn on. None by default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreeBlows {
    /// A blow of its melee attack that kills gives it a blow at another hostile beside it.
    #[serde(default, skip_serializing_if = "is_false")]
    pub cleave: bool,
    /// Cleaving goes on while the blows kill (it cleaves as `cleave` does, and more).
    #[serde(default, skip_serializing_if = "is_false")]
    pub great_cleave: bool,
    /// A melee attack on it that its Parry made miss gives it a blow back.
    #[serde(default, skip_serializing_if = "is_false")]
    pub riposte: bool,
    /// A hostile stepping away from beside it gives it a blow.
    #[serde(default, skip_serializing_if = "is_false")]
    pub opportunist: bool,
    /// Its melee attacks with nothing in hand strike twice, the second blow at −2.
    #[serde(default, skip_serializing_if = "is_false")]
    pub flurry: bool,
    /// A hostile stepping up beside it gives it a blow (Gatekeeper).
    #[serde(default, skip_serializing_if = "is_false")]
    pub gatekeeper: bool,
}

impl FreeBlows {
    /// Whether it gets none.
    pub fn is_none(&self) -> bool {
        *self == Self::default()
    }

    /// Those of either: what the actor's own switches and its talents' give together.
    pub fn either(self, other: Self) -> Self {
        Self {
            cleave: self.cleave || other.cleave,
            great_cleave: self.great_cleave || other.great_cleave,
            riposte: self.riposte || other.riposte,
            opportunist: self.opportunist || other.opportunist,
            flurry: self.flurry || other.flurry,
            gatekeeper: self.gatekeeper || other.gatekeeper,
        }
    }
}

/// The flurry's second blow's to-hit.
const FLURRY_TO_HIT: i32 = -2;

fn is_false(b: &bool) -> bool {
    !*b
}

/// What happened that may give a blow more, as [`World::second_blow`] is asked: by the striker,
/// at the target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Cue {
    /// The first blow of the striker's melee attack on the target came to this (`None`: nothing
    /// was rolled): the second blow.
    First(Option<Blow>),
    /// The striker's melee attack on the target is over, `parried` if one of its blows missed by
    /// the target's Parry: a cleave if it killed the target, else the target's riposte.
    Attacked { parried: bool },
    /// The striker's cleave at the target is over: another, with great cleave, if it killed.
    Cleaved,
    /// The target is about to step away from beside the striker, out of its reach.
    Leaves,
    /// The target has just stepped up beside the striker.
    Approaches,
}

/// What a blow more is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Free {
    /// The off hand's blow, or a shield bash, after an attack's first.
    Second,
    /// At another hostile, after a kill.
    Cleave,
    /// Back at a blow the striker's Parry made miss.
    Riposte,
    /// At a hostile stepping away.
    Opportunist,
    /// At a hostile stepping up beside (Gatekeeper).
    Gatekeeper,
}

impl Free {
    /// Whether it makes the noise of an attack: only the opportunist's, an attack of its own.
    pub(super) fn noisy(self) -> bool {
        matches!(self, Free::Opportunist | Free::Gatekeeper)
    }

    /// Whether it is struck in another's turn (in answer to what they did).
    fn out_of_turn(self) -> bool {
        matches!(self, Free::Riposte | Free::Opportunist | Free::Gatekeeper)
    }
}

/// A blow more ([`World::second_blow`]): who strikes whom, what it is, and with what.
#[derive(Clone, Debug)]
pub(super) struct FreeBlow {
    pub(super) by: ActorId,
    pub(super) at: ActorId,
    pub(super) kind: Free,
    /// What it is made with: the off hand's weapon or the shield; `None` for a blow with what
    /// the striker wields (else its natural attack or bare hands).
    pub(super) with: Option<ObjectId>,
    pub(super) profile: Profile,
}

/// Whether `blow` missed by its target's Parry.
pub(super) fn parried(blow: Option<Blow>) -> bool {
    blow.is_some_and(|b| {
        b.strike
            == Strike::Missed {
                by: Some(Category::Parry),
            }
    })
}

impl World {
    /// The free blows actor `actor` is given: the one place the switches are read.
    pub(super) fn free_blows_of(&self, actor: &Actor) -> FreeBlows {
        // Its own switches (class abilities), those its talents turn on, and those its weapon
        // abilities do with what it holds (`weapon_ability.rs`).
        let blows = (actor.free_blows)
            .either(actor.talent_mods(&self.content).free_blows)
            .either(self.ability_switches(actor).free_blows);
        // Lurking, a hostile stepping up beside it takes its blow (`thief.rs`).
        let blows = FreeBlows {
            gatekeeper: blows.gatekeeper || self.lurking(actor),
            ..blows
        };
        // Surprised, it strikes nothing out of its turn (`surprise.rs`).
        if self.surprised(actor.id) {
            return FreeBlows {
                riposte: false,
                opportunist: false,
                gatekeeper: false,
                ..blows
            };
        }
        blows
    }

    /// Whether actor `id` still stands to strike or be struck: alive, and the run not over.
    fn stands(&self, id: ActorId) -> bool {
        self.state.death.is_none() && self.state.actors.contains_key(&id)
    }

    /// The free blows the end of actor `attacker`'s melee attack on `target` brings, `parried`
    /// if one of its blows missed by the target's Parry: a cleave if it killed the target (and
    /// with great cleave another after each cleave that kills), else the target's riposte.
    pub(super) fn after_attack(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        parried: bool,
        events: &mut Vec<Event>,
    ) {
        let mut next = self.second_blow(attacker, target, Cue::Attacked { parried }, events);
        while let Some(blow) = next {
            let (by, at, kind) = (blow.by, blow.at, blow.kind);
            self.strike_free(blow, events);
            next = match kind {
                Free::Cleave => self.second_blow(by, at, Cue::Cleaved, events),
                _ => None,
            };
        }
    }

    /// Actor `id` is about to step to `to`: every hostile opportunist beside it whose reach the
    /// step leaves strikes it first, in actor order. Returns whether it still stands to take the
    /// step.
    pub(super) fn step_away(&mut self, id: ActorId, to: Pos, events: &mut Vec<Event>) -> bool {
        let Some(me) = self.state.actors.get(&id) else {
            return false;
        };
        let behind: Vec<ActorId> = (self.state.actors.values())
            // Who it leaves first: its talents are dear to read.
            .filter(|other| self.leaves(me, other, to) && self.free_blows_of(other).opportunist)
            .map(|other| other.id)
            .collect();
        for other in behind {
            if let Some(blow) = self.second_blow(other, id, Cue::Leaves, events) {
                self.strike_free(blow, events);
            }
        }
        self.stands(id)
    }

    /// Actor `id` has just stepped from `from` up beside a hostile gatekeeper (spec §8 "Birth
    /// sign": Gatekeeper): each one it was not beside before strikes it as it comes, in actor
    /// order, while it stands.
    pub(super) fn step_up(&mut self, id: ActorId, from: Pos, events: &mut Vec<Event>) {
        let Some(me) = self.state.actors.get(&id) else {
            return;
        };
        let keepers: Vec<ActorId> = (self.state.actors.values())
            .filter(|other| other.id != id && other.level == me.level)
            .filter(|other| other.pos.chebyshev(me.pos) == 1 && other.pos.chebyshev(from) > 1)
            .filter(|other| self.free_blows_of(other).gatekeeper)
            .filter(|other| self.stance(id, other.id) == Stance::Hostile)
            .map(|other| other.id)
            .collect();
        for keeper in keepers {
            if let Some(blow) = self.second_blow(keeper, id, Cue::Approaches, events) {
                self.lie_in_wait(keeper, id);
                let struck = self.strike_free(blow, events);
                self.technique.lurk = None;
                // Brace of steel: a braced foe the blow hits staggers.
                let hit = struck.is_some_and(|b| b.strike.is_hit());
                if let Some((_, status)) = self.brace_of(keeper).filter(|_| hit) {
                    self.afflict(id, status, 1, Some(keeper), events);
                }
            }
        }
        // Weapon abilities that fire on a hostile approaching (Point-blank).
        self.approached(id, from, events);
    }

    /// Actor `striker`'s blow at `mover`, which stepped up beside it ([`World::step_up`]): a
    /// blow with what it wields, if it is a gatekeeper.
    pub(super) fn gatekeeper(&self, striker: ActorId, mover: ActorId) -> Option<FreeBlow> {
        if !self.stands(striker) || !self.stands(mover) {
            return None;
        }
        let me = &self.state.actors[&striker];
        let mut profile = me.melee(&self.content);
        // Brace of steel: a Brace's blow is surer.
        if let Some((to_hit, _)) = self.brace_of(striker) {
            profile.to_hit += to_hit;
        }
        self.free_blows_of(me).gatekeeper.then_some(FreeBlow {
            by: striker,
            at: mover,
            kind: Free::Gatekeeper,
            with: None,
            profile,
        })
    }

    /// Whether `me` stepping to `to` takes it out of the reach of `other`, a hostile beside it
    /// now (spec §8 "Stances": the coward's flight; "Free blows": the opportunist's blow).
    pub(super) fn leaves(&self, me: &Actor, other: &Actor, to: Pos) -> bool {
        other.id != me.id
            && other.level == me.level
            && other.pos.chebyshev(me.pos) == 1
            && other.pos.chebyshev(to) > 1
            && self.stance(me.id, other.id) == Stance::Hostile
    }

    /// A cleave by actor `striker` after its blow killed `fallen`, `again` after a cleave (which
    /// takes great cleave): a blow with what it wields at the next hostile beside it clockwise
    /// from where `fallen` stood. No draw: it is the striker's to give.
    pub(super) fn cleave(
        &self,
        striker: ActorId,
        fallen: ActorId,
        again: bool,
    ) -> Option<FreeBlow> {
        if !self.stands(striker) || self.state.actors.contains_key(&fallen) {
            return None;
        }
        let me = &self.state.actors[&striker];
        let blows = self.free_blows_of(me);
        if !(blows.great_cleave || (blows.cleave && !again)) {
            return None;
        }
        let turn =
            |p: Pos| Dir::toward(me.pos, p).and_then(|d| Dir::ALL.iter().position(|&a| a == d));
        let start = self
            .actor_or_fallen(fallen)
            .and_then(|a| turn(a.pos))
            .unwrap_or(0);
        let (_, at) = (self.state.actors.values())
            .filter(|other| other.id != striker && other.level == me.level)
            .filter(|other| other.pos.chebyshev(me.pos) == 1)
            .filter(|other| self.stance(striker, other.id) == Stance::Hostile)
            .filter_map(|other| Some(((turn(other.pos)? + 8 - start) % 8, other.id)))
            .min()?;
        Some(FreeBlow {
            by: striker,
            at,
            kind: Free::Cleave,
            with: None,
            profile: me.melee(&self.content),
        })
    }

    /// Actor `target`'s riposte at `attacker`, whose melee attack on it is over, `parried` if a
    /// blow of it missed by `target`'s Parry: a blow back with what it wields.
    pub(super) fn riposte(
        &self,
        attacker: ActorId,
        target: ActorId,
        parried: bool,
    ) -> Option<FreeBlow> {
        if !parried || !self.stands(attacker) || !self.stands(target) {
            return None;
        }
        let (me, them) = (&self.state.actors[&target], &self.state.actors[&attacker]);
        let beside = me.level == them.level && me.pos.chebyshev(them.pos) == 1;
        (beside && self.free_blows_of(me).riposte).then(|| FreeBlow {
            by: target,
            at: attacker,
            kind: Free::Riposte,
            with: None,
            profile: me.melee(&self.content),
        })
    }

    /// The flurry's second blow of actor `striker` at `target` (spec §8 "Class abilities":
    /// Flurry): with nothing in hand, its bare hands again at −2 to hit, spending what it costs
    /// (a point of Ki, `flurry_cost`; holding less, no blow). No draw.
    pub(super) fn flurry(
        &mut self,
        striker: ActorId,
        target: ActorId,
        events: &mut Vec<Event>,
    ) -> Option<FreeBlow> {
        if !self.stands(striker) || !self.stands(target) {
            return None;
        }
        let me = &self.state.actors[&striker];
        if !self.free_blows_of(me).flurry || me.weapon_item(&self.content).is_some() {
            return None;
        }
        if let Some(cost) = me.talent_mods(&self.content).flurry_cost
            && !self.spend_resource(striker, cost.resource, cost.amount, events)
        {
            return None;
        }
        let me = &self.state.actors[&striker];
        let mut profile = me.melee(&self.content);
        profile.to_hit += FLURRY_TO_HIT;
        Some(FreeBlow {
            by: striker,
            at: target,
            kind: Free::Second,
            with: None,
            profile,
        })
    }

    /// Actor `striker`'s blow at `mover`, stepping away from beside it ([`World::step_away`]):
    /// a blow with what it wields, if it is an opportunist.
    pub(super) fn opportunist(&self, striker: ActorId, mover: ActorId) -> Option<FreeBlow> {
        if !self.stands(striker) || !self.stands(mover) {
            return None;
        }
        let me = &self.state.actors[&striker];
        self.free_blows_of(me).opportunist.then(|| FreeBlow {
            by: striker,
            at: mover,
            kind: Free::Opportunist,
            with: None,
            profile: me.melee(&self.content),
        })
    }

    /// Strikes `blow`, a blow like any other (`fight.rs`). The player's is said as they strike
    /// it, or, in another's turn, after that turn's reports: what is said meanwhile is held
    /// ([`World::say`]) and joins the news, which `perceive` says last. Returns the blow.
    pub(super) fn strike_free(&mut self, blow: FreeBlow, events: &mut Vec<Event>) -> Option<Blow> {
        let player = self.state.player;
        let (by, at) = (blow.by, blow.at);
        if by != player {
            let skills = self.skills_of(by, AttackKind::Melee, blow.with);
            let kind = (AttackKind::Melee, blow.with, skills);
            let noisy = blow.kind.noisy();
            return self.resolve(by, at, kind, Some(blow.profile), noisy, events);
        }
        let seen =
            (self.state.actors.get(&at)).is_some_and(|a| self.perception.sees_actors_at(a.pos));
        if !blow.kind.out_of_turn() {
            return self.player_blow(at, seen, Some(blow), events);
        }
        let earlier = std::mem::take(&mut self.news);
        self.held = Some(Vec::new());
        let struck = self.player_blow(at, seen, Some(blow), events);
        let held = self.held.take().unwrap_or_default();
        self.news = earlier;
        self.news.extend(held);
        struck
    }
}
