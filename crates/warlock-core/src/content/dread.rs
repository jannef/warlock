//! An endless branch's dread curve (docs/design/monster-dread.md §8; the owner, 2026-11-06:
//! "Below 70, keep drastically increasing dread monster chance deeper you go"): its
//! `dread_one_in` by depth, since a branch without a last floor has no `[[dungeon.levels]]` to
//! give each depth its own.
//!
//! ```toml
//! [dungeon.endless.dread]
//! steps = [[30, 80], [50, 60], [70, 40]]   # from this depth on: one draw in this many
//! halves = 10                              # past the last step: halved every this many floors
//! least = 5                                # and never below this
//! ```
//!
//! Above the first step a floor draws by the branch's own `dread_one_in` (or `[defaults]`'s).
//! Past the last step the one-in halves every `halves` floors, falling in a straight line within
//! each halving, in whole numbers (no floating point: the same on every platform), down to
//! `least`. Without `halves` it stays at the last step's.

use serde::Deserialize;

/// The curve as written.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DreadCurveSpec {
    steps: Vec<[u32; 2]>,
    halves: Option<u32>,
    least: Option<u16>,
}

/// An endless branch's `dread_one_in` by depth ([`DreadCurve::at`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DreadCurve {
    /// From each depth on, one draw in this many is dread; by depth, rising.
    pub steps: Vec<(u32, u16)>,
    /// Past the last step, the one-in halves every this many floors; `None`: it stays.
    pub halves: Option<u32>,
    /// The least the one-in falls to.
    pub least: u16,
}

impl DreadCurveSpec {
    /// The curve, checked: steps by rising depth ([`DreadCurve::at`] reads them in order), each
    /// one-in up to 65 535 (a `u16`), and `halves` 1 or more (a divisor).
    pub(super) fn resolve(self) -> Result<DreadCurve, String> {
        let mut steps = Vec::new();
        for [depth, one_in] in self.steps {
            let one_in =
                u16::try_from(one_in).map_err(|_| "`dread.steps`: a one-in is 0 to 65535")?;
            if steps.last().is_some_and(|&(d, _)| d >= depth) {
                return Err("`dread.steps`: each depth deeper than the last".into());
            }
            steps.push((depth, one_in));
        }
        if self.halves == Some(0) {
            return Err("`dread.halves` is 1 or more".into());
        }
        Ok(DreadCurve {
            steps,
            halves: self.halves,
            least: self.least.unwrap_or(1),
        })
    }
}

impl DreadCurve {
    /// The `dread_one_in` of floor `depth`; `None` above the first step (the branch's own).
    pub fn at(&self, depth: u32) -> Option<u16> {
        let &(from, one_in) = self.steps.iter().rev().find(|&&(d, _)| d <= depth)?;
        let last = self.steps.last().map_or(from, |&(d, _)| d);
        let (Some(halves), true) = (self.halves, from == last) else {
            return Some(one_in);
        };
        let past = depth - from;
        let (k, into) = (past / halves, past % halves);
        let high = u32::from(one_in).checked_shr(k).unwrap_or(0);
        let low = high / 2;
        let at = high - (high - low) * into / halves;
        Some(u16::try_from(at).unwrap_or(u16::MAX).max(self.least))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn curve(text: &str) -> Result<DreadCurve, String> {
        let spec: DreadCurveSpec = toml::from_str(text).map_err(|e| e.to_string())?;
        spec.resolve()
    }

    #[test]
    fn the_curve_steps_then_halves_down_to_its_least() {
        let c = curve("steps = [[30, 80], [50, 60], [70, 40]]\nhalves = 10\nleast = 5\n").unwrap();
        let at = |d| c.at(d);
        assert_eq!(at(1), None, "above the first step: the branch's own");
        assert_eq!(at(29), None);
        assert_eq!((at(30), at(49)), (Some(80), Some(80)));
        assert_eq!((at(50), at(69)), (Some(60), Some(60)));
        assert_eq!(
            [70, 75, 80, 85, 90, 95, 100, 200, 10_000].map(at),
            [40, 30, 20, 15, 10, 8, 5, 5, 5].map(Some)
        );
        // Never rising with depth.
        let mut last = u16::MAX;
        for d in 30..400 {
            let now = at(d).unwrap();
            assert!(now <= last, "depth {d}: {now} after {last}");
            last = now;
        }
    }

    #[test]
    fn without_halves_the_last_step_holds() {
        let c = curve("steps = [[10, 50]]\n").unwrap();
        assert_eq!((c.at(9), c.at(10), c.at(5000)), (None, Some(50), Some(50)));
    }

    #[test]
    fn a_bad_curve_is_refused() {
        for bad in [
            "steps = [[10, 70000]]\n",
            "steps = [[20, 50], [10, 40]]\n",
            "steps = [[20, 50], [20, 40]]\n",
            "steps = [[20, 50]]\nhalves = 0\n",
            "steps = [[20, 50]]\nodds = 3\n",
        ] {
            assert!(curve(bad).is_err(), "{bad}");
        }
    }
}
