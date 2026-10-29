//! Elixirs (spec §8 "Potions"; the combat progression design §10.5): a potion of kind
//! `elixir`, drunk through [`World::drink_potion`] as any potion, raises its stat for good.
//!
//! - **The score**: `points` trained rises ([`Stats::train`]): the score +1 each, never past
//!   the scores' range; the potential then lifted to the score where it was below (a trained
//!   rise alone leaves it, the stat training design §4.4; an elixir is nature's change).
//! - **The potential**: then raised by a roll of `potential` ("1d2"), one gameplay draw, never
//!   past the range ([`Stats::widen`]).
//! - **What follows**: hit points follow a Toughness rise at once (as a trained rise brings them,
//!   `World::settle`); the maximum mana, the carrying limits, and everything else derived
//!   from the score follow by themselves.
//! - **What is said**: the trained rise's words ("You feel stronger."), with ", and that you
//!   could be stronger yet." when the potential rose by 2 or more; never a number.
//!
//! The marks it gives (its `marks`) are the `elixir` use's, given by `drink_potion`.
//!
//! [`Stats::train`]: crate::stats::Stats::train
//! [`Stats::widen`]: crate::stats::Stats::widen

use super::World;
use crate::dice::Dice;
use crate::stats::Stat;

/// What an elixir of `stat` says: `rose` whether the score rose, `widened` whether the potential
/// rose by 2 or more besides.
fn news(stat: Stat, rose: bool, widened: bool) -> String {
    let feel = stat.comparative();
    match (rose, widened) {
        (true, true) => format!("You feel {feel}, and that you could be {feel} yet."),
        (true, false) => format!("You feel {feel}."),
        (false, true) => format!("You feel that you could be {feel} yet."),
        (false, false) => "You feel no different.".to_string(),
    }
}

impl World {
    /// The player drank an elixir of `stat` (module docs): `points` trained rises, then the
    /// potential raised by a roll of `potential` (a gameplay draw), and the hit points a
    /// Toughness rise brings.
    pub(super) fn drink_elixir(&mut self, stat: Stat, points: u8, potential: Dice) {
        let content = self.content.clone();
        let roll = u8::try_from(potential.roll(&mut self.state.rng).max(0)).unwrap_or(u8::MAX);
        let me = self.player_mut();
        let before = me.max_hp(&content);
        let rose = (0..points).filter(|_| me.stats.train(stat)).count() > 0;
        // A trained rise leaves the potential; an elixir's lifts it to the score first.
        let below = me
            .stats
            .score(stat)
            .saturating_sub(me.stats.potential(stat));
        me.stats.widen(stat, below);
        let widened = me.stats.widen(stat, roll) >= 2;
        let grown = me.max_hp(&content) - before;
        if grown > 0 {
            me.hp += grown;
        }
        self.say(news(stat, rose, widened));
    }
}
