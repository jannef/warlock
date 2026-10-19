//! Checks (spec §8 "Checks"): a stat put to the test, rolled like an attack (spec §8 "Combat").
//!
//! A check succeeds when 2d12 (its natural roll, the same dice as an attack's: [`combat::
//! roll_natural`]) plus the modifier of the stat it tests reaches its **difficulty**. A natural 2
//! always fails and a natural 24 always succeeds. Diseases are resisted and fought off with
//! checks, mostly of Toughness, a few of Willpower (spec §8 "Diseases"). Rolls come from the
//! gameplay random stream, so they replay exactly; the player never sees a difficulty, a
//! modifier, or a roll: they see what comes of it.

use crate::combat;
use crate::rng::Rng;

/// Rolls a check with `modifier` against `difficulty`: two draws from `rng` (the natural roll).
pub fn check(rng: &mut Rng, modifier: i32, difficulty: i32) -> bool {
    combat::hits(combat::roll_natural(rng), modifier, difficulty)
}

/// How many of the 144 natural rolls make a check with `modifier` against `difficulty`.
pub fn ways(modifier: i32, difficulty: i32) -> u32 {
    combat::hit_ways(modifier, difficulty)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_follow_the_attack_roll_and_its_sure_ends() {
        // 2d12 + 0 against 13: 78 of 144; every point of modifier is a point of difficulty.
        assert_eq!(ways(0, 13), 78);
        assert_eq!(ways(2, 15), 78);
        // A natural 24 always makes it, a natural 2 never does.
        assert_eq!(ways(-50, 40), 1);
        assert_eq!(ways(50, 1), 143);
        // One check is two draws: the same as an attack roll.
        let mut a = Rng::gameplay(7);
        let mut b = a.clone();
        let natural = combat::roll_natural(&mut b);
        assert_eq!(check(&mut a, 0, 13), natural >= 13);
        assert_eq!(a, b);
        // Over many rolls, it succeeds as often as its ways say.
        let mut rng = Rng::gameplay(1);
        let made = (0..14_400).filter(|_| check(&mut rng, 1, 15)).count();
        let expected = 100 * ways(1, 15) as usize;
        assert!(made.abs_diff(expected) < 400, "{made} of 14400, {expected}");
    }
}
