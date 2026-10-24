//! The player's lantern (spec §5 "Light sources"; #235): out, lit, or lit with its hood lowered.
//!
//! The scenario gives the lantern (its `lantern` light preset), unlit. `lantern` lights or douses
//! it (the `lantern` action); `hood` (the `hood` action) lowers or raises its hood, and on a
//! lantern that is out lights it with the hood lowered (the time of both). Hooded, it burns as
//! its preset's `hooded` preset: a dim light of half the reach that is hollow, so it lights the
//! cells around the player but not their own cell, and eyes that need light see them only by
//! other light (or within darkvision). Dousing it raises the hood: lit again, it burns open. The
//! light it gives is decided in one place, [`WorldState::lantern_light`], which the light cache
//! reads for the player's perception and creature sight alike.

use super::{Spent, World, WorldState};
use crate::content::{ActionKind, Content, LightId};
use serde::Serialize;

/// The state of the player's lantern.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Lantern {
    Out,
    Lit,
    /// Lit, with its hood lowered: a dim light around the player that leaves them in the dark.
    Hooded,
}

impl Lantern {
    /// As the status bar says it: "Lantern hooded".
    pub const fn name(self) -> &'static str {
        match self {
            Lantern::Out => "Lantern out",
            Lantern::Lit => "Lantern lit",
            Lantern::Hooded => "Lantern hooded",
        }
    }
}

impl WorldState {
    /// The preset of the player's lantern (the scenario's `lantern`), if the scenario gives one.
    fn lantern_preset(&self, content: &Content) -> Option<LightId> {
        let scenario = content.scenario(&self.scenario)?;
        content.light_id(scenario.lantern.as_deref()?)
    }

    /// The light the player's lantern gives now: its preset while lit, its `hooded` preset while
    /// hooded; none while out or without a lantern. The one place the lantern's light is decided.
    pub(crate) fn lantern_light(&self, content: &Content) -> Option<LightId> {
        let preset = self.lantern_preset(content).filter(|_| self.lantern_lit)?;
        let hooded = (content.light(preset).hooded.as_deref()).and_then(|id| content.light_id(id));
        match hooded {
            Some(hooded) if self.lantern_hooded => Some(hooded),
            _ => Some(preset),
        }
    }
}

impl World {
    /// The player's lantern: `None` if the scenario gives none.
    pub fn lantern(&self) -> Option<Lantern> {
        self.state.lantern_preset(&self.content)?;
        Some(match (self.state.lantern_lit, self.state.lantern_hooded) {
            (false, _) => Lantern::Out,
            (true, false) => Lantern::Lit,
            (true, true) => Lantern::Hooded,
        })
    }

    /// Lights the lantern, or douses it (raising its hood); `false`, with a message, without one.
    pub(super) fn toggle_lantern(&mut self) -> bool {
        if self.lantern().is_none() {
            self.say("You have no lantern.");
            return false;
        }
        self.state.lantern_lit = !self.state.lantern_lit;
        self.state.lantern_hooded = false;
        if self.state.lantern_lit {
            self.say("You light your lantern.");
        } else {
            self.say("You douse your lantern.");
        }
        true
    }

    /// Lowers the lantern's hood, or raises it; a lantern that is out is lit with its hood
    /// lowered, which takes the time of lighting it and of the hood. Refused, taking no time,
    /// without a lantern or with one that has no hood.
    pub(super) fn hood_lantern(&mut self) -> Spent {
        let Some(preset) = self.state.lantern_preset(&self.content) else {
            self.say("You have no lantern.");
            return Spent::Nothing;
        };
        if self.content.light(preset).hooded.is_none() {
            self.say("Your lantern has no hood.");
            return Spent::Nothing;
        }
        let player = self.state.player;
        let hood = self.price_of(player, ActionKind::Hood);
        let (price, said) = match (self.state.lantern_lit, self.state.lantern_hooded) {
            (false, _) => (
                self.price_of(player, ActionKind::Lantern) + hood,
                "You light your lantern with its hood lowered.",
            ),
            (true, false) => (hood, "You lower your lantern's hood."),
            (true, true) => (hood, "You raise your lantern's hood."),
        };
        self.state.lantern_hooded = !(self.state.lantern_lit && self.state.lantern_hooded);
        self.state.lantern_lit = true;
        self.say(said);
        Spent::Turn(price)
    }
}
