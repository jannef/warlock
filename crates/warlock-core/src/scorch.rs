//! Char (spec §8 "Blood"): the scorch marks fire leaves on floors and walls, building up the
//! more a cell burns, from light soot to blackened to charred.
//!
//! A sibling of [`crate::blood`]: the same [`Stain`] per cell, kept in its own map of the level
//! (`LevelState::scorch`), saved and hashed, and **no rule reads it**. How much lands where comes
//! from a hash of the fire ([`seed`]: the turn, the moment, the cell it centres on), never from
//! the gameplay random stream, so char never shifts a later roll.
//!
//! - **Who chars**: a damage type with a `char` in `data/damage_types.toml` (fire's).
//! - **Where** ([`area`]): every open cell an area of it reaches (a burst's, [`AREA`], half at its
//!   edge; a cone's; a fire zone's, [`ZONE`], as it is laid), and the walls beside those cells on
//!   the faces toward them ([`Stain::faces`], as blood's stained faces), a little less
//!   ([`WALL`] percent, half again on a diagonal). A creature struck by it chars its own cell
//!   ([`blow`]): a little for any blow, more for a big hit or a kill, as blood would have flowed.
//! - **How much**: each cell's share jittered from 60 to 140 % by the hash; amounts add up to
//!   [`crate::blood::CAP`] per cell ([`Stain::add`]), and the colour follows the amount from the
//!   type's light colour (soot) to its heavy one (charred), [`color`].
//! - **Words** ([`word`]): "scorched", "blackened", "charred" by amount.

use crate::blood::{CAP, Stain, cell_key, face_bit, mix};
use crate::color::Rgb;
use crate::content::CharColors;
use crate::geom::{Dir, Pos};
use crate::time::GameTime;
use std::collections::BTreeMap;

/// Char a burst leaves on each cell it fully reaches (half at its edge) and a cone on each of
/// its cells, before the jitter.
pub const AREA: u32 = 20;
/// Char a fire zone leaves on each cell it is laid on: it burns there for turns.
pub const ZONE: u32 = 24;
/// What a wall beside a burning cell takes on its face, in percent of the cell's share (half
/// again across a corner).
pub const WALL: u32 = 75;
/// Char any blow of a charring type leaves on its target's cell.
pub const BLOW: u32 = 3;

/// Keeps char's hashes apart from blood's, should a fire and a blow share a moment.
const SALT: u64 = 0x5c07_c4a2_f12e_0001;

/// The seed of one fire's char: the turn and the moment it burned, and the cell it centres on.
/// Never the gameplay random stream.
pub fn seed(turn: u64, time: GameTime, at: Pos) -> u64 {
    mix(mix(turn ^ SALT) ^ mix(time.seconds().rotate_left(17)) ^ cell_key(at))
}

/// Char landing on one cell: `amount`, on the faces `faces` of a wall (0 on a floor).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Burn {
    pub at: Pos,
    pub amount: u8,
    pub faces: u8,
}

/// The char of a fire over `cells` (each with its share before the jitter), with `seed`:
/// every open cell its jittered share, then the walls beside them ([`WALL`] percent, half across
/// a corner, the most any neighbour gives) on the faces toward them; `is_opaque` tells walls
/// (and the level's edge) from open cells. In position order. Whether a cell takes a stain
/// (water and lava do not) is the caller's to decide.
pub fn area(cells: &[(Pos, u32)], is_opaque: impl Fn(Pos) -> bool, seed: u64) -> Vec<Burn> {
    let jitter = |p: Pos, share: u32| {
        let roll = mix(seed ^ mix(cell_key(p)));
        share * (60 + (roll % 81) as u32) / 100
    };
    let mut out: BTreeMap<Pos, (u32, u8)> = BTreeMap::new();
    for &(p, share) in cells {
        if is_opaque(p) {
            continue;
        }
        let here = out.entry(p).or_default();
        here.0 = here.0.max(jitter(p, share));
        for d in Dir::ALL {
            let w = p.step(d);
            if !is_opaque(w) {
                continue;
            }
            let diagonal = d.delta().0 != 0 && d.delta().1 != 0;
            let percent = if diagonal { WALL / 2 } else { WALL };
            let wall = out.entry(w).or_default();
            wall.0 = wall.0.max(jitter(w, share) * percent / 100);
            wall.1 |= face_bit(d.opposite());
        }
    }
    out.into_iter()
        .filter(|(_, (amount, _))| *amount > 0)
        .map(|(at, (amount, faces))| Burn {
            at,
            amount: amount.min(u32::from(CAP)) as u8,
            faces,
        })
        .collect()
}

/// The char a blow of a charring type leaves on its target's cell `at`: [`BLOW`], and a third of
/// the blood `volume` it would have spilled ([`crate::blood::Wound::volume`]: a big hit or a
/// kill), jittered by `seed`.
pub fn blow(at: Pos, volume: u32, seed: u64) -> Burn {
    let share = BLOW + volume / 3;
    let roll = mix(seed ^ mix(cell_key(at)));
    let amount = (share * (60 + (roll % 81) as u32) / 100).max(1);
    Burn {
        at,
        amount: amount.min(u32::from(CAP)) as u8,
        faces: 0,
    }
}

/// The colour of `amount` of char of `colors`: from soot (`light`) toward charred (`heavy`).
pub fn color(colors: CharColors, amount: u8) -> Rgb {
    let t = u32::from(amount.min(CAP));
    let cap = u32::from(CAP);
    let mix = |a: u8, b: u8| ((u32::from(a) * (cap - t) + u32::from(b) * t) / cap) as u8;
    let (a, b) = (colors.light, colors.heavy);
    Rgb::new(mix(a.r, b.r), mix(a.g, b.g), mix(a.b, b.b))
}

/// Adds `burn` of `colors` at `now` to the char `stain` there (`None`: a fresh one): the amount
/// adds up to the cap, the faces join, and the colour follows the amount ([`color`]).
pub fn add(stain: Option<Stain>, burn: Burn, colors: CharColors, now: GameTime) -> Stain {
    let mut stain = match stain {
        Some(mut s) => {
            s.add(burn.amount, s.color, now, None);
            s
        }
        None => Stain::new(burn.amount, colors.light, now, None),
    };
    stain.faces |= burn.faces;
    stain.color = color(colors, stain.amount);
    stain
}

/// How char reads in look mode, before the terrain's name: "scorched", "blackened", "charred"
/// by amount.
pub fn word(stain: &Stain) -> &'static str {
    match stain.amount {
        ..30 => "scorched",
        30..65 => "blackened",
        _ => "charred",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRE: CharColors = CharColors {
        light: Rgb::new(0x3a, 0x2a, 0x1e),
        heavy: Rgb::new(0x0c, 0x0a, 0x08),
    };

    fn ring(center: Pos, share: u32) -> Vec<(Pos, u32)> {
        (-1..=1)
            .flat_map(|dy| (-1..=1).map(move |dx| Pos::new(center.x + dx, center.y + dy)))
            .map(|p| (p, share))
            .collect()
    }

    #[test]
    fn an_area_chars_its_cells_and_the_faces_of_the_walls_beside_them() {
        // A wall all along x = 12, east of a burst around (10, 10).
        let walls = |p: Pos| p.x >= 12;
        let burns = area(&ring(Pos::new(10, 10), AREA), walls, 7);
        let at = |p: Pos| burns.iter().find(|b| b.at == p).copied();
        for p in [Pos::new(9, 9), Pos::new(10, 10), Pos::new(11, 11)] {
            let b = at(p).expect("an open cell it reached");
            assert!((AREA * 6 / 10..=AREA * 14 / 10).contains(&u32::from(b.amount)));
            assert_eq!(b.faces, 0);
        }
        let wall = at(Pos::new(12, 10)).expect("the wall beside it");
        assert!(
            wall.faces & face_bit(Dir::W) != 0,
            "the face toward the fire"
        );
        assert!(wall.faces & face_bit(Dir::E) == 0);
        assert!(at(Pos::new(13, 10)).is_none(), "nothing behind the wall");
        assert!(at(Pos::new(8, 8)).is_none(), "nothing outside the area");
    }

    #[test]
    fn the_pattern_is_a_pure_function_of_the_fire() {
        let cells = ring(Pos::new(5, 5), AREA);
        let open = |_: Pos| false;
        assert_eq!(area(&cells, open, 1), area(&cells, open, 1));
        assert_ne!(area(&cells, open, 1), area(&cells, open, 2));
        assert_ne!(
            seed(3, GameTime::from_seconds(9), Pos::new(1, 1)),
            crate::blood::seed(3, GameTime::from_seconds(9), 0, 0)
        );
    }

    #[test]
    fn char_builds_up_darker_to_the_cap() {
        let t = GameTime::from_seconds(100);
        let burn = |amount| Burn {
            at: Pos::new(0, 0),
            amount,
            faces: 0,
        };
        let soot = add(None, burn(10), FIRE, t);
        assert_eq!(word(&soot), "scorched");
        let more = add(Some(soot), burn(30), FIRE, t);
        assert_eq!((more.amount, word(&more)), (40, "blackened"));
        let lum = |c: Rgb| u32::from(c.r) + u32::from(c.g) + u32::from(c.b);
        assert!(
            lum(more.color) < lum(soot.color),
            "darker the more it burned"
        );
        let mut charred = more;
        for _ in 0..10 {
            charred = add(Some(charred), burn(40), FIRE, t);
        }
        assert_eq!((charred.amount, word(&charred)), (CAP, "charred"));
        assert_eq!(charred.color, FIRE.heavy);
        let face = Burn {
            faces: face_bit(Dir::E),
            ..burn(5)
        };
        assert!(add(Some(charred), face, FIRE, t).on_face(Dir::E));
    }

    #[test]
    fn a_blow_chars_its_target_s_cell_more_for_a_big_one() {
        let small = blow(Pos::new(3, 3), 0, 11);
        let kill = blow(Pos::new(3, 3), 30, 11);
        assert!(small.amount >= 1 && small.amount < kill.amount);
    }
}
