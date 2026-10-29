//! Praying at a forgotten altar (spec §7 "Special rooms"; the design's §7): [`Action::Pray`],
//! the Pray ability of the abilities menu's *Here* group, offered beside a forgotten altar.
//!
//! - **Where**: beside a cell whose terrain a room effect's `feature` names and that `offers`
//!   pray ([`crate::content::Content::offer_at`]); the first such cell around the player in
//!   [`Dir::ALL`] order. A spent altar is another terrain, so it offers nothing. Refused, taking
//!   no time, anywhere else.
//! - **Time**: the table's `seconds`, in turns of the `pray` action ([`World::keep_at`], the
//!   rest's machinery; the last cut short to end on time): a creature coming into view, a message, or a blow stops it, and a broken
//!   prayer gives nothing and never spends the altar ("Your prayer is broken."), but the time is
//!   spent.
//! - **The answer**: the deed first ([`Deed::Prayed`], by the altar's side), then one outcome
//!   drawn from the table, leaned by the altar's side against the player's band
//!   ([`super::outcome::leaned_kinds`]; Devout), and its hooks (`outcome.rs`; the great ones
//!   `boons.rs`); for one with Penitent a great bane drawn becomes one of the table's nothing
//!   outcomes, drawn in its place (the explosion is moderate: it stays). Then, unless the
//!   altar is gone (it blew apart), one draw of the table's `exhaust`: the altar is spent, its
//!   cell the table's `spent` terrain and its own light gone ("The altar's glow gutters and
//!   dies."). Nothing else is saved: the terrain is the whole record.
//!
//! Every draw is from the gameplay stream.
//!
//! [`Action::Pray`]: crate::action::Action::Pray

use super::alignment::Deed;
use super::outcome::leaned_kinds;
use super::rest::Ended;
use super::{Spent, World};
use crate::action::Event;
use crate::alignment::Alignment;
use crate::content::{ActionKind, Offer, OutcomeKind, OutcomeTier};
use crate::geom::{Dir, Pos};
use crate::time::Duration;

/// A prayer under way: the altar's cell and how long it takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Prayer {
    altar: Pos,
    seconds: u32,
}

impl World {
    /// The forgotten altar beside the player: the first cell around them (in [`Dir::ALL`]
    /// order) whose terrain offers Pray.
    fn altar_beside(&self) -> Option<Pos> {
        let level = self.state.current();
        let here = self.player_pos();
        (Dir::ALL.iter().map(|&d| here.step(d))).find(|&p| {
            (level.terrain_at(p))
                .and_then(|t| self.content.offer_at(t))
                .is_some_and(|(offer, _)| offer == Offer::Pray)
        })
    }

    /// The hidden side of the altar on `at`: its room's, else neutral.
    pub(super) fn altar_side(&self, at: Pos) -> Alignment {
        (self.state.current().room_at(at))
            .and_then(|r| r.side)
            .unwrap_or_default()
    }

    /// The player begins to pray at the altar beside them; refused, taking no time, with none.
    pub(super) fn start_prayer(&mut self) -> Spent {
        let Some(altar) = self.altar_beside() else {
            self.say("There is no altar here to pray at.");
            return Spent::Nothing;
        };
        let terrain = self.state.current().terrain_at(altar);
        let Some((_, table)) = terrain.and_then(|t| self.content.offer_at(t)) else {
            return Spent::Nothing;
        };
        let seconds = table.seconds;
        self.say("You kneel before the altar and pray.");
        Spent::Pray(Prayer { altar, seconds })
    }

    /// Prays turn after turn until the prayer is done or something breaks it (module docs): its
    /// last turn cut short to end exactly when its time is up, whatever the player's speed, as
    /// a rest's is.
    pub(super) fn pray(&mut self, prayer: Prayer, events: &mut Vec<Event>) {
        let until = self.state.time + Duration::seconds(u64::from(prayer.seconds));
        let turn = self.price_of(self.state.player, ActionKind::Pray);
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| turn.min(w.price_until(until)),
        );
        match ended {
            Ended::Died => {}
            _ if self.state.time >= until => self.prayer_answered(prayer.altar, events),
            Ended::Came(id) => {
                let text = format!(
                    "{} comes into view; your prayer is broken.",
                    self.creature_called(id)
                );
                self.say(text);
            }
            Ended::Said | Ended::Done => self.say("Your prayer is broken."),
        }
    }

    /// The prayer at the altar on `altar` is done: the deed, the outcome, and perhaps the altar
    /// spent (module docs).
    fn prayer_answered(&mut self, altar: Pos, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let terrain = self.state.current().terrain_at(altar);
        let Some((_, table)) = terrain.and_then(|t| content.offer_at(t)) else {
            self.say("Your prayer goes unheard.");
            return;
        };
        let side = self.altar_side(altar);
        self.player_did(Deed::Prayed(side));
        self.train_stat(crate::content::StatUse::Prayed);
        let me = self.state.player();
        let devout = me.talent_mods(&content).devout;
        let band = me.band(&content).side();
        let penitent = me.talent_mods(&content).penitent;
        let kinds = leaned_kinds(table, side, band, devout);
        let mut drawn = self.draw_outcome(table, kinds);
        // Penitent: a great bane becomes nothing, one of the table's nothing lines.
        if penitent
            && drawn.is_some_and(|o| o.kind == OutcomeKind::Bane && o.tier == OutcomeTier::Great)
        {
            drawn = self.draw_in(table, OutcomeKind::Nothing);
        }
        if let Some(outcome) = drawn {
            self.befall(outcome, altar, events);
        }
        // Blown apart (or otherwise changed), nothing is left to spend.
        if self.state.current().terrain_at(altar) != terrain || self.state.death.is_some() {
            return;
        }
        if let Some(spent) = table.spent
            && table.exhaust > 0
            && self.state.rng.chance(u32::from(table.exhaust))
        {
            self.state.current_mut().lights.remove(&altar);
            self.set_terrain(altar, spent);
            self.say("The altar's glow gutters and dies.");
        }
    }
}
