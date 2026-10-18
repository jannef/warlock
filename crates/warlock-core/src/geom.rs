//! Grid geometry: positions and the eight movement directions.
//!
//! Coordinates are screen-like: `x` grows to the right, `y` grows downward, and `(0, 0)` is the
//! top-left cell of a level.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A cell position on a level grid.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// The neighboring position in direction `dir`.
    pub fn step(self, dir: Dir) -> Self {
        let (dx, dy) = dir.delta();
        Self::new(self.x + dx, self.y + dy)
    }

    /// Squared Euclidean distance (exact integer, used for radii).
    pub fn dist2(self, other: Self) -> i32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// Chebyshev (king-move) distance.
    pub fn chebyshev(self, other: Self) -> i32 {
        (self.x - other.x).abs().max((self.y - other.y).abs())
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// One of the eight compass directions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Dir {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Dir {
    pub const ALL: [Dir; 8] = [
        Dir::N,
        Dir::NE,
        Dir::E,
        Dir::SE,
        Dir::S,
        Dir::SW,
        Dir::W,
        Dir::NW,
    ];

    /// `(dx, dy)` with `y` growing downward.
    pub const fn delta(self) -> (i32, i32) {
        match self {
            Dir::N => (0, -1),
            Dir::NE => (1, -1),
            Dir::E => (1, 0),
            Dir::SE => (1, 1),
            Dir::S => (0, 1),
            Dir::SW => (-1, 1),
            Dir::W => (-1, 0),
            Dir::NW => (-1, -1),
        }
    }

    /// The direction pointing the other way.
    pub const fn opposite(self) -> Self {
        match self {
            Dir::N => Dir::S,
            Dir::NE => Dir::SW,
            Dir::E => Dir::W,
            Dir::SE => Dir::NW,
            Dir::S => Dir::N,
            Dir::SW => Dir::NE,
            Dir::W => Dir::E,
            Dir::NW => Dir::SE,
        }
    }

    pub fn from_delta(dx: i32, dy: i32) -> Option<Self> {
        Dir::ALL.into_iter().find(|d| d.delta() == (dx, dy))
    }

    /// The direction of the eight nearest to the way from `from` to `to` (within 22.5° of an
    /// axis it is that axis, integer math); `None` if they are the same cell.
    pub fn toward(from: Pos, to: Pos) -> Option<Self> {
        let (dx, dy) = (i64::from(to.x - from.x), i64::from(to.y - from.y));
        // tan 22.5° ≈ 5/12.
        let (sx, sy) = if 12 * dy.abs() < 5 * dx.abs() {
            (dx.signum(), 0)
        } else if 12 * dx.abs() < 5 * dy.abs() {
            (0, dy.signum())
        } else {
            (dx.signum(), dy.signum())
        };
        Dir::from_delta(sx as i32, sy as i32)
    }

    /// The compass name, e.g. `north-east`.
    pub const fn compass_name(self) -> &'static str {
        match self {
            Dir::N => "north",
            Dir::NE => "north-east",
            Dir::E => "east",
            Dir::SE => "south-east",
            Dir::S => "south",
            Dir::SW => "south-west",
            Dir::W => "west",
            Dir::NW => "north-west",
        }
    }

    /// Short uppercase name, e.g. `NE`.
    pub const fn name(self) -> &'static str {
        match self {
            Dir::N => "N",
            Dir::NE => "NE",
            Dir::E => "E",
            Dir::SE => "SE",
            Dir::S => "S",
            Dir::SW => "SW",
            Dir::W => "W",
            Dir::NW => "NW",
        }
    }
}

impl fmt::Display for Dir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for Dir {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Dir::ALL
            .into_iter()
            .find(|d| d.name().eq_ignore_ascii_case(s))
            .ok_or_else(|| format!("unknown direction `{s}` (expected N, NE, E, SE, S, SW, W, NW)"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_and_delta_roundtrip() {
        let origin = Pos::new(5, 5);
        for dir in Dir::ALL {
            let next = origin.step(dir);
            assert_eq!(Dir::from_delta(next.x - 5, next.y - 5), Some(dir));
            assert_eq!(next.step(dir.opposite()), origin);
        }
    }

    #[test]
    fn parses_directions_case_insensitively() {
        assert_eq!("ne".parse::<Dir>(), Ok(Dir::NE));
        assert_eq!("W".parse::<Dir>(), Ok(Dir::W));
        assert!("up".parse::<Dir>().is_err());
    }

    #[test]
    fn toward_picks_the_nearest_of_eight_directions() {
        let o = Pos::new(0, 0);
        let toward = |x, y| Dir::toward(o, Pos::new(x, y));
        assert_eq!(toward(0, 0), None);
        assert_eq!(toward(5, 0), Some(Dir::E));
        assert_eq!(toward(5, 2), Some(Dir::E), "within 22.5° of east");
        assert_eq!(toward(5, 3), Some(Dir::SE));
        assert_eq!(toward(1, -1), Some(Dir::NE));
        assert_eq!(toward(-2, -5), Some(Dir::N));
        assert_eq!(toward(-3, 5), Some(Dir::SW));
        assert_eq!(Dir::NW.compass_name(), "north-west");
    }

    #[test]
    fn distances() {
        let a = Pos::new(0, 0);
        let b = Pos::new(3, -4);
        assert_eq!(a.dist2(b), 25);
        assert_eq!(a.chebyshev(b), 4);
    }
}
