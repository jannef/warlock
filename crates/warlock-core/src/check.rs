//! Checks (spec §8 "Checks"): a stat put to the test, rolled like an attack (spec §8 "Combat").
//!
//! A check succeeds when 2d12 (its natural roll, the same dice as an attack's: [`combat::
//! roll_natural`]) plus the modifier of the stat it tests reaches its **difficulty**. A natural 2
//! always fails and a natural 24 always succeeds. Diseases are resisted and fought off with
//! checks, mostly of Toughness, a few of Willpower (spec §8 "Diseases"); disarming a trap is a
//! Dexterity check whose failure counts by how much it fell short ([`roll`], spec §8 "Disarming
//! traps"). Rolls come from the
//! gameplay random stream, so they replay exactly; the player never sees a difficulty, a
//! modifier, or a roll: they see what comes of it.
//!
//! A **contest** puts one side's check against the other's: the other side's natural roll plus
//! its modifier is the difficulty (stealing, spec §8 "Stealing": the thief's Dexterity against
//! the victim's Willpower). A tie goes to the side making the check.

use crate::combat;
use crate::rng::Rng;

/// Rolls a check with `modifier` against `difficulty`: two draws from `rng` (the natural roll).
pub fn check(rng: &mut Rng, modifier: i32, difficulty: i32) -> bool {
    combat::hits(combat::roll_natural(rng), modifier, difficulty)
}

/// How a check came out, for a rule that cares how badly it failed (disarming a trap: a failure
/// by enough sets it off).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Checked {
    Made,
    /// It failed, short of the difficulty by this much: the difficulty less the natural roll and
    /// the modifier, never below 0 (a natural 2 fails however high the modifier).
    Failed {
        by: i32,
    },
}

/// Rolls a check with `modifier` against `difficulty` and says by how much it failed: two draws
/// from `rng`, as [`check`].
pub fn roll(rng: &mut Rng, modifier: i32, difficulty: i32) -> Checked {
    let natural = combat::roll_natural(rng);
    if combat::hits(natural, modifier, difficulty) {
        Checked::Made
    } else {
        let by = difficulty.saturating_sub(natural.saturating_add(modifier));
        Checked::Failed { by: by.max(0) }
    }
}

/// How many of the 144 natural rolls fail a check with `modifier` against `difficulty` by `by`
/// or more ([`roll`]).
pub fn ways_short_by(modifier: i32, difficulty: i32, by: i32) -> u32 {
    (combat::FUMBLE..=combat::SURE_HIT)
        .filter(|&n| !combat::hits(n, modifier, difficulty))
        .filter(|&n| difficulty.saturating_sub(n.saturating_add(modifier)).max(0) >= by)
        .map(combat::ways)
        .sum()
}

/// How many of the 144 natural rolls make a check with `modifier` against `difficulty`.
pub fn ways(modifier: i32, difficulty: i32) -> u32 {
    combat::hit_ways(modifier, difficulty)
}

/// Rolls a contest: a check with `modifier` against the other side's check with `against`,
/// whose total (its natural roll plus `against`) is the difficulty. Four draws from `rng`: the
/// other side's natural roll, then the check's. A tie goes to the check; its natural 2 still
/// fails and its natural 24 still succeeds.
pub fn contest(rng: &mut Rng, modifier: i32, against: i32) -> bool {
    let difficulty = combat::roll_natural(rng) + against;
    check(rng, modifier, difficulty)
}

/// How many of the 144 × 144 ways both sides' dice can fall win a contest with `modifier`
/// against `against` (out of [`CONTEST_WAYS`]).
pub fn contest_ways(modifier: i32, against: i32) -> u32 {
    (2..=24)
        .map(|natural| combat::ways(natural) * ways(modifier, natural + against))
        .sum()
}

/// Every way both sides' dice can fall in a contest: 144 × 144.
pub const CONTEST_WAYS: u32 = 144 * 144;

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

    #[test]
    fn a_contest_is_a_check_against_the_other_sides_roll() {
        // Even sides: a little better than half, ties going to the check.
        let even = contest_ways(0, 0);
        assert!(
            even > CONTEST_WAYS / 2 && even < CONTEST_WAYS * 55 / 100,
            "{even}"
        );
        // A modifier is worth as much on either side, and better odds never fall.
        assert_eq!(contest_ways(3, 1), contest_ways(2, 0));
        assert!((-10..10).all(|m| contest_ways(m, 0) <= contest_ways(m + 1, 0)));
        // The sure ends hold however lopsided it is.
        assert_eq!(contest_ways(-50, 50), CONTEST_WAYS / 144);
        assert_eq!(contest_ways(50, -50), CONTEST_WAYS - CONTEST_WAYS / 144);
        // Four draws: the other side's natural roll, then the check's.
        let mut a = Rng::gameplay(3);
        let mut b = a.clone();
        let theirs = combat::roll_natural(&mut b);
        let mine = combat::roll_natural(&mut b);
        assert_eq!(contest(&mut a, 1, 2), combat::hits(mine, 1, theirs + 2));
        assert_eq!(a, b);
        // Over many rolls, it is won as often as its ways say.
        let mut rng = Rng::gameplay(5);
        let won = (0..20_736).filter(|_| contest(&mut rng, 2, 0)).count();
        let expected = contest_ways(2, 0) as usize;
        assert!(won.abs_diff(expected) < 500, "{won} of 20736, {expected}");
    }

    #[test]
    fn a_roll_says_how_far_short_it_fell() {
        // The same two draws as a check, and the same result.
        for seed in 0..200 {
            let mut a = Rng::gameplay(seed);
            let mut b = a.clone();
            let natural = combat::roll_natural(&mut b.clone());
            let made = check(&mut b, 2, 15);
            let checked = roll(&mut a, 2, 15);
            assert_eq!(a, b);
            assert_eq!(checked == Checked::Made, made);
            if let Checked::Failed { by } = checked {
                assert_eq!(by, (15 - natural - 2).max(0), "natural {natural}");
            }
        }
        // A natural 2 fails by nothing when the modifier alone reaches the difficulty.
        assert_eq!(ways_short_by(20, 15, 0), 1);
        assert_eq!(ways_short_by(20, 15, 1), 0);
        // 2d12 + 0 against 14: short by 5 or more is a natural roll of 9 or less (36 of 144),
        // and every failure is short by at least 1.
        assert_eq!(ways_short_by(0, 14, 5), 36);
        assert_eq!(ways_short_by(0, 14, 1), 144 - ways(0, 14));
        // Made and failed together are every roll.
        assert_eq!(ways(3, 17) + ways_short_by(3, 17, 0), 144);
    }
}
