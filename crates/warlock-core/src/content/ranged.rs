//! Shots by distance (`assets/data/ranged.toml`, spec §8 "Ranged combat"; #956): the to-hit
//! penalties of a shot at long range and at an adjacent target.
//!
//! The rule itself lives in `world/shoot.rs` (`World::fly`) and is worked out as each creature on
//! the line is attacked, so it saves nothing. The file is optional: without it a shot is as
//! accurate at any distance.

use super::ContentError;
use crate::fov::radius2;
use crate::geom::Pos;
use serde::Deserialize;

/// One band of long range: beyond `beyond` percent of the weapon's range, `to_hit`.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RangeBand {
    pub beyond: i32,
    pub to_hit: i32,
}

/// The penalties of shots by distance; none by default.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RangedRules {
    /// The to-hit of a shot at a target adjacent to the shooter (diagonals too); it wins over
    /// the bands.
    pub adjacent: i32,
    /// The bands of long range, rising outward, each harsher.
    #[serde(rename = "band", default)]
    pub bands: Vec<RangeBand>,
}

impl RangedRules {
    /// Parses `data/ranged.toml`: the bands rise in `beyond` ([`Self::to_hit_at`] reads them in
    /// order).
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let parsed: Self =
            toml::from_str(text).map_err(|e| ContentError::new(file, e.to_string()))?;
        if !parsed.bands.windows(2).all(|w| w[0].beyond < w[1].beyond) {
            return Err(ContentError::new(file, "the bands rise in `beyond`"));
        }
        Ok(parsed)
    }

    /// The to-hit of a shot from `from` at a creature on `at`, with a weapon of `range` cells:
    /// the adjacent penalty for an adjacent target, else the farthest band it lies beyond.
    pub fn to_hit_at(&self, range: i32, from: Pos, at: Pos) -> i32 {
        if from.chebyshev(at) <= 1 {
            return self.adjacent;
        }
        // Beyond `beyond` percent of the range: the squared distance against that share of the
        // range's `radius2`, in whole numbers.
        let far = i64::from(at.dist2(from)) * 100 * 100;
        let reach = i64::from(radius2(range));
        (self.bands.iter())
            .take_while(|b| far > reach * i64::from(b.beyond) * i64::from(b.beyond))
            .last()
            .map_or(0, |b| b.to_hit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RULES: &str = "adjacent = -2\n\
        [[band]]\nbeyond = 50\nto_hit = -1\n\
        [[band]]\nbeyond = 75\nto_hit = -2\n";

    fn rules() -> RangedRules {
        RangedRules::parse("data/ranged.toml", RULES).expect("valid")
    }

    /// The to-hit of a shot straight along a row at `cells` from the shooter.
    fn along(rules: &RangedRules, range: i32, cells: i32) -> i32 {
        rules.to_hit_at(range, Pos::new(0, 0), Pos::new(cells, 0))
    }

    #[test]
    fn the_bands_follow_the_range_outward() {
        let r = rules();
        let at: Vec<i32> = (2..=10).map(|d| along(&r, 10, d)).collect();
        // Up to 5 cells (50 %) nothing, to 7 (of 7.5) -1, then -2.
        assert_eq!(at, [0, 0, 0, 0, -1, -1, -2, -2, -2]);
    }

    #[test]
    fn the_bands_are_measured_as_the_range_is() {
        let r = rules();
        let from = Pos::new(0, 0);
        // A range of 10 reaches `radius2` 110: 50 % of it is 27.5, 75 % is 61.875.
        assert_eq!(r.to_hit_at(10, from, Pos::new(5, 1)), 0, "26");
        assert_eq!(r.to_hit_at(10, from, Pos::new(5, 2)), -1, "29");
        assert_eq!(r.to_hit_at(10, from, Pos::new(6, 5)), -1, "61");
        assert_eq!(r.to_hit_at(10, from, Pos::new(7, 4)), -2, "65");
    }

    #[test]
    fn adjacent_wins_over_the_bands_diagonals_too() {
        let r = rules();
        let from = Pos::new(0, 0);
        // A range of 2 reaches `radius2` 6: 50 % of it is 1.5, so the diagonal neighbour (2) lies
        // beyond it and the straight one (1) does not, and both take the adjacent penalty.
        assert_eq!(r.to_hit_at(2, from, Pos::new(1, 1)), -2);
        assert_eq!(r.to_hit_at(2, from, Pos::new(1, 0)), -2);
        // With a milder adjacent penalty the band still does not stack or win.
        let mild = RangedRules {
            adjacent: 0,
            ..rules()
        };
        assert_eq!(mild.to_hit_at(2, from, Pos::new(1, 1)), 0);
    }

    #[test]
    fn no_rules_no_penalty() {
        let none = RangedRules::default();
        assert_eq!(along(&none, 10, 1), 0);
        assert_eq!(along(&none, 10, 10), 0);
    }

    #[test]
    fn bad_numbers_are_refused() {
        for bad in [
            "adjacent = -2\nspare = 1\n",
            "adjacent = 0\n[[band]]\nbeyond = 75\nto_hit = -1\n[[band]]\nbeyond = 50\nto_hit = -2\n",
        ] {
            assert!(
                RangedRules::parse("data/ranged.toml", bad).is_err(),
                "{bad}"
            );
        }
    }
}
