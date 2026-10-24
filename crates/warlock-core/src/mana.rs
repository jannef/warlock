//! Mana (spec §8 "Mana"): the second pool beside hit points, built the same way. These are the
//! pure rules; `world/mana.rs` applies them to actors, and `world/status.rs` regenerates it as
//! time passes.
//!
//! - **The maximum**: what race and class give (a creature: its type's `mana`; a scenario
//!   without a character: its own), plus the modifiers of the three mental stats, Intelligence,
//!   Willpower, and Presence ([`max_mana`]), at least 0. Levels add more (half the class's mana
//!   and the better of the Intelligence and Willpower modifiers, [`level_modifier`]), and so do
//!   magic items.
//! - **Regeneration**: one point comes back every `max(60 s, 360 s − 12 s × Willpower)` of game
//!   time ([`regen_seconds`]) at the normal pace; resting, status effects, diseases, special
//!   rooms, starving, and magic items change the pace (`World::mana_rate`).
//!
//! Nothing here is shown to the player but their own mana and its maximum: never the pace.

use crate::stats::{Stat, Stats};

/// The most mana a race, a class, a creature type, or a scenario may give.
pub const MAX_BASE_MANA: u16 = 1000;

/// The stats whose modifiers add to the maximum, in stat order.
pub const MENTAL: [Stat; 3] = [Stat::Intelligence, Stat::Willpower, Stat::Presence];

/// The modifiers of the mental stats ([`MENTAL`]) added up: what `stats` add to the maximum
/// mana.
pub fn mental_modifier(stats: &Stats) -> i32 {
    MENTAL.iter().map(|&stat| stats.modifier(stat)).sum()
}

/// What `stats` add to the mana a level gives: the better of the Intelligence and Willpower
/// modifiers.
pub fn level_modifier(stats: &Stats) -> i32 {
    stats
        .modifier(Stat::Intelligence)
        .max(stats.modifier(Stat::Willpower))
}

/// Maximum mana: `base` (race and class for the player, the creature type's `mana` for a
/// creature, with what levels and magic items add) plus the mental stats' modifiers, at least 0.
pub fn max_mana(stats: &Stats, base: i32) -> i32 {
    (base + mental_modifier(stats)).max(0)
}

/// The longest a point of mana takes to come back, and the shortest...
pub const REGEN_SLOWEST: u32 = 360;
pub const REGEN_FASTEST: u32 = 60;
/// ...and how much each point of Willpower takes off.
pub const REGEN_PER_WILLPOWER: u32 = 12;

/// Seconds of game time per point of mana back at the normal pace: `max(60, 360 − 12 ×
/// Willpower)`.
pub fn regen_seconds(willpower: u8) -> u32 {
    REGEN_SLOWEST
        .saturating_sub(REGEN_PER_WILLPOWER * u32::from(willpower))
        .max(REGEN_FASTEST)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::PerStat;

    fn stats(intelligence: u8, willpower: u8, presence: u8) -> Stats {
        let mut score = PerStat::splat(10);
        score[Stat::Intelligence] = intelligence;
        score[Stat::Willpower] = willpower;
        score[Stat::Presence] = presence;
        // Strength and the rest never count.
        score[Stat::Strength] = 30;
        score[Stat::Toughness] = 3;
        Stats::new(score, PerStat::splat(40))
    }

    #[test]
    fn the_maximum_is_the_base_and_the_mental_modifiers() {
        assert_eq!(max_mana(&stats(10, 10, 10), 5), 5);
        assert_eq!(max_mana(&stats(16, 12, 11), 5), 5 + 3 + 1);
        assert_eq!(max_mana(&stats(6, 8, 10), 5), 5 - 2 - 1);
        assert_eq!(max_mana(&stats(1, 1, 1), 2), 0, "never below 0");
        assert_eq!(mental_modifier(&stats(18, 14, 9)), 4 + 2 - 1);
        assert_eq!(
            level_modifier(&stats(18, 14, 30)),
            4,
            "the better, never Presence"
        );
        assert_eq!(level_modifier(&stats(6, 12, 10)), 1);
    }

    #[test]
    fn mana_comes_back_faster_with_willpower() {
        assert_eq!(regen_seconds(10), 240);
        assert_eq!(regen_seconds(1), 348);
        assert_eq!(regen_seconds(0), 360);
        assert_eq!(regen_seconds(25), 60);
        assert_eq!(regen_seconds(40), 60, "a minute at the quickest");
    }
}
