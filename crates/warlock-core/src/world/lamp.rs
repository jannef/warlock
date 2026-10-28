//! Carried lights (spec §5 "Light sources"; #643): a lantern is an item, and a light-giving item
//! ready in the Tool slot gives the player light by its data (its object's `light`).
//!
//! Its mode is item state ([`LightMode`], saved with the stack wherever it goes): unlit, lit, or
//! hooded. Using the tool (`u`, [`Action::UseTool`](crate::action::Action::UseTool), the
//! `use_tool` action text) turns a light ready in the Tool slot from unlit to lit (the time of
//! `light`), from lit to hooded (`hood`), and from hooded back to unlit (`light`); a light whose
//! preset has no `hooded` one goes from lit straight back to unlit. Hooded, it burns as that
//! `hooded` preset: a dim light of half the reach that is hollow, so it lights the cells around
//! the player but not their own cell. A light in the pack, or on the ground, keeps its mode but
//! gives no light. The light it gives is decided in one place, [`WorldState::carried_light`],
//! which the light cache reads for the player's perception and creature sight alike; a magic
//! light's affixes would modify it there.

use super::{Spent, World, WorldState};
use crate::content::{ActionKind, Content, LightId};
use crate::object::LightMode;

impl WorldState {
    /// The light the player's ready light gives now: its object's preset while lit (a
    /// Lamplighter's brighter one instead, spec §8 "Talents"), the preset's `hooded` one while
    /// hooded; none while unlit, or without a light ready in the Tool slot. The one place the
    /// player's carried light is decided.
    pub(crate) fn carried_light(&self, content: &Content) -> Option<LightId> {
        let player = self.player();
        let item = player.ready_light(content)?;
        let stack = player.inventory.get(item)?;
        let preset = content.object(stack.object).carried_light?;
        match stack.light_mode() {
            LightMode::Unlit => None,
            LightMode::Hooded => {
                let hooded = content.light(preset).hooded.as_deref();
                Some(hooded.and_then(|id| content.light_id(id)).unwrap_or(preset))
            }
            LightMode::Lit => Some((player.talent_mods(content).lantern_light).unwrap_or(preset)),
        }
    }
}

impl World {
    /// Uses the tool ready in the Tool slot without a direction: turns a light there to its next
    /// mode (the module docs). Refused, taking no time, without one (a digging tool is used
    /// toward a direction, `dig:DIR`).
    pub(super) fn use_tool(&mut self) -> Spent {
        let player = self.state.player;
        let Some(item) = self.state.player().ready_light(&self.content) else {
            self.say("You have no light ready to use.");
            return Spent::Nothing;
        };
        let stack = &self.state.player().inventory[item];
        let name = stack.title(&self.content);
        let preset = self.content.object(stack.object).carried_light;
        let hood = preset.is_some_and(|p| self.content.light(p).hooded.is_some());
        let (mode, kind, said) = match stack.light_mode() {
            LightMode::Unlit => (LightMode::Lit, ActionKind::Light, "You light your"),
            LightMode::Lit if hood => (
                LightMode::Hooded,
                ActionKind::Hood,
                "You lower the hood of your",
            ),
            LightMode::Lit | LightMode::Hooded => {
                (LightMode::Unlit, ActionKind::Light, "You douse your")
            }
        };
        self.player_mut().inventory[item].set_light_mode(mode);
        self.say(format!("{said} {name}."));
        Spent::Turn(self.price_of(player, kind))
    }

    /// Puts out the player's ready light, if they have one (an outcome's `snuff`).
    pub(super) fn snuff_light(&mut self) {
        if let Some(item) = self.state.player().ready_light(&self.content) {
            self.player_mut().inventory[item].set_light_mode(LightMode::Unlit);
        }
    }
}
