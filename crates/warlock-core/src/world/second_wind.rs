//! Second wind (spec §8 "Talents"), the Vigour tree's active talent: from the abilities menu
//! ([`crate::ability::Ability::SecondWind`]), [`Action::UseTalent`] (`use:second_wind`) gives
//! the player hit points back at once (one per character level, at least the talent's floor)
//! for the time of a `wait`, then its cooldown status (Winded), while which it cannot be used
//! again. Nothing is drawn.
//!
//! [`Action::UseTalent`]: crate::action::Action::UseTalent

use super::{Spent, World};
use crate::action::Event;
use crate::content::ActionKind;

impl World {
    /// The player catches their second wind (module docs). Refused, taking no time, without the
    /// talent and while its cooldown lasts.
    pub(super) fn second_wind(&mut self, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let me = self.state.player();
        let Some(wind) = me.talent_mods(&content).second_wind else {
            self.say("You have no second wind to catch.");
            return Spent::Nothing;
        };
        if me.effects.has(wind.cooldown) {
            let name = content.status(wind.cooldown).name.to_lowercase();
            self.say(format!("You are still {name}."));
            return Spent::Nothing;
        }
        let max = me.max_hp(&content);
        let back = wind.hp_at(self.player_level());
        let me = self.player_mut();
        me.hp = (me.hp + back).min(max).max(me.hp);
        self.say("You catch your second wind.");
        self.afflict(player, wind.cooldown, 1, None, events);
        Spent::Turn(self.price_of(player, ActionKind::Wait))
    }
}
