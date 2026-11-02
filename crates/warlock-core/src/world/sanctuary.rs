//! Sanctuary (spec §8 "Class abilities", §7 "Special rooms": hallowed ground; #885, the priest
//! design §6.4): the priest's level-50 class active, `use:sanctuary`. The player kneels for the
//! rite's time, its mana spent ([`World::keep_at`], as Consecrate prays: anything that would stop
//! a rest stops it, and an interrupted rite lays nothing, its mana and time spent). Done, a
//! [`Hallow`] lies on the level around the cell they knelt on for the talent's `lasts`:
//!
//! - no creature hostile to the player steps onto it unless it already stands on it
//!   ([`LevelState::shuts_out`], one clause in [`World::shuns`]), and nothing spawns there;
//! - whoever stands on it heals and regains mana at its `regen` percent, the larger of that and
//!   a special room's pace, never both ([`World::hallow_pace`], one term in the two rates);
//! - a blow, shot, or throw of the player's, or a spell of theirs that strikes another, made
//!   while they stand on it breaks it ([`World::break_hallow`]); being struck breaks nothing;
//! - its end is a moment of [`World::live`] (the paces change then), and at the end of the turn
//!   that passes it, it is dropped ([`World::hallows_fade`]). Leaving the level keeps it.
//!
//! Refused with no time or mana on the overworld, in open water, in a shop, or on ground already
//! hallowed. The face that answers the priest (`Actor::prayer_face`) when the rite ends gives
//! the words and the look ("hallowed" or "dreaded" ground); the rule is the same. Missiles and
//! spells cross it. Nothing here draws at random, and it sheds no light.

use super::{LevelState, Spent, World, rest::Ended};
use crate::action::Event;
use crate::content::{ActionKind, Face, LevelKind};
use crate::geom::Pos;
use crate::time::GameTime;
use serde::{Deserialize, Serialize};

/// Ground hallowed by Sanctuary (kept in its level's state, saved and hashed).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hallow {
    /// The cell knelt on.
    pub(crate) at: Pos,
    /// How far it reaches from `at` (Chebyshev).
    pub(crate) radius: u8,
    /// When it fades.
    pub(crate) until: GameTime,
    /// The healing and mana pace on it, in percent.
    pub(crate) regen: u16,
    /// Whether the dark powers laid it (dreaded ground, not hallowed: the words alone).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(crate) dark: bool,
}

impl Hallow {
    /// Whether it covers `p` at `now`: within its radius and not yet faded.
    pub(crate) fn covers(&self, p: Pos, now: GameTime) -> bool {
        now < self.until && p.chebyshev(self.at) <= i32::from(self.radius)
    }

    /// The cell knelt on.
    pub fn at(&self) -> Pos {
        self.at
    }

    /// How far it reaches.
    pub fn radius(&self) -> u8 {
        self.radius
    }

    /// When it fades.
    pub fn until(&self) -> GameTime {
        self.until
    }

    /// Whether the dark powers laid it.
    pub fn dark(&self) -> bool {
        self.dark
    }
}

impl LevelState {
    /// The hallows that cover `p` at `now`.
    pub(crate) fn hallows_at(&self, p: Pos, now: GameTime) -> impl Iterator<Item = &Hallow> {
        self.hallows.iter().filter(move |h| h.covers(p, now))
    }

    /// Whether a hostile standing at `from` keeps out of `p` at `now`: `p` lies in a hallow that
    /// `from` does not (one already inside may move within it and leave).
    pub(crate) fn shuts_out(&self, from: Option<Pos>, p: Pos, now: GameTime) -> bool {
        self.hallows_at(p, now)
            .any(|h| from.is_none_or(|from| !h.covers(from, now)))
    }
}

impl World {
    /// `use:sanctuary`: the player kneels, its mana spent, or is refused with no time where the
    /// ground cannot be hallowed.
    pub(super) fn begin_sanctuary(&mut self) -> Spent {
        let Some(sanctuary) = self.state.player().talent_mods(&self.content).sanctuary else {
            return Spent::Nothing;
        };
        let (level, here, now) = (self.state.current(), self.player_pos(), self.state.time);
        let refused = if level.kind == LevelKind::Overworld {
            Some("The open land cannot be hallowed: find a place to kneel.")
        } else if level.is_open_water(&self.content, here) {
            Some("You cannot kneel in water.")
        } else if level.shops.iter().any(|s| s.contains(here)) {
            Some("This is a shop, not a sanctuary.")
        } else if level.hallows_at(here, now).next().is_some() {
            Some("This ground is already hallowed.")
        } else {
            None
        };
        if let Some(text) = refused {
            self.say(text);
            return Spent::Nothing;
        }
        let player = self.state.player;
        if !self.spend_mana(player, sanctuary.mana) {
            self.say("You have too little mana.");
            return Spent::Nothing;
        }
        self.say("You kneel and pray.");
        Spent::Sanctuary(self.state.time + sanctuary.time)
    }

    /// Kneels turn after turn until `until`, unless something interrupts (module docs); then the
    /// ground is hallowed.
    pub(super) fn sanctuary(&mut self, until: GameTime, events: &mut Vec<Event>) {
        let wait = self.price_of(self.state.player, ActionKind::Wait);
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| wait.min(w.price_until(until)),
        );
        match ended {
            Ended::Died => {}
            _ if self.state.time >= until => self.hallow(),
            Ended::Came(id) => {
                let text = format!(
                    "{} comes into view; you rise from your knees.",
                    self.creature_called(id)
                );
                self.say(text);
            }
            Ended::Said | Ended::Done => self.say("You rise from your knees."),
        }
    }

    /// The rite is done: the ground about the player is hallowed.
    fn hallow(&mut self) {
        let player = self.state.player();
        let Some(sanctuary) = player.talent_mods(&self.content).sanctuary else {
            return;
        };
        let dark = player.prayer_face(&self.content) == Face::Dark;
        let hallow = Hallow {
            at: player.pos,
            radius: sanctuary.radius,
            until: self.state.time + sanctuary.lasts,
            regen: sanctuary.regen,
            dark,
        };
        self.state.current_mut().hallows.push(hallow);
        self.say(if dark {
            "A dread settles on the ground about you."
        } else {
            "The ground about you is hallowed."
        });
    }

    /// The player strikes: a hallow they stand in is broken (a blow, shot, or throw of theirs,
    /// or a spell of theirs that strikes another).
    pub(super) fn break_hallow(&mut self) {
        let (here, now) = (self.player_pos(), self.state.time);
        let level = self.state.current_mut();
        let before = level.hallows.len();
        level.hallows.retain(|h| !h.covers(here, now));
        if level.hallows.len() != before {
            self.say("The peace is broken.");
        }
    }

    /// The end of a turn that began at `began`: the current level's hallows past their end are
    /// dropped, and one that faded during the turn says so (one that faded while the player was
    /// away goes quietly). The message does not stop a rest.
    pub(super) fn hallows_fade(&mut self, began: GameTime) {
        let now = self.state.time;
        let level = self.state.current_mut();
        if level.hallows.iter().all(|h| h.until > now) {
            return;
        }
        let faded = (level.hallows.iter()).any(|h| h.until <= now && h.until > began);
        level.hallows.retain(|h| h.until > now);
        if faded {
            self.note("The hallowing fades.");
        }
    }

    /// The healing and mana pace of the hallow `actor` stands in at `now`, in percent (the
    /// largest if several; 100 on no hallowed ground).
    pub(super) fn hallow_pace(&self, actor: &super::Actor, now: GameTime) -> u32 {
        (self.state.levels.get(&actor.level).into_iter())
            .flat_map(|level| level.hallows_at(actor.pos, now))
            .map(|h| u32::from(h.regen))
            .fold(100, u32::max)
    }

    /// Seconds from `now` until a hallow actor `id` stands in fades (its paces change then).
    pub(super) fn until_unhallowed(&self, id: super::ActorId, now: GameTime) -> Option<u64> {
        let actor = self.state.actors.get(&id)?;
        let level = self.state.levels.get(&actor.level)?;
        (level.hallows_at(actor.pos, now))
            .map(|h| h.until.since(now).as_seconds())
            .min()
    }

    /// Dev: the current level's hallows, in the order laid.
    pub fn hallows(&self) -> &[Hallow] {
        &self.state.current().hallows
    }
}
