//! Blood on the map grid (spec §4.8 "Blood", §8 "Blood"): a stained cell's colours shift toward
//! the blood's by how much lies there, before the grid shader lights them like the terrain they
//! lie on, so stains darken in the dark, show in darkvision's monochrome, and dim with the cell
//! when it is remembered. A floor's glyph and background give way to the blood (a soaked cell is
//! a dark pool, its dots drowned); a wall is tinted more lightly. Old blood dries darker and
//! browner over hours of game time. Everything here reads the perception snapshot
//! ([`PerceptionView::blood`](warlock_core::perception::PerceptionView::blood)); the `blood`
//! switch hides it.

use warlock_core::blood::{CAP, Stain};
use warlock_core::color::Rgb;
use warlock_core::time::GameTime;

/// Game seconds until spilled blood has dried (it darkens fastest at first).
const DRY_SECONDS: f32 = 6.0 * 3600.0;
/// How far a soaked floor's glyph and background give way to the blood (0 to 1).
const FLOOR_FG: f32 = 0.9;
const FLOOR_BG: f32 = 0.85;
/// The same on a wall: blood on its face tints it.
const WALL_FG: f32 = 0.8;
const WALL_BG: f32 = 0.7;
/// A pool's background: the blood's colour this much darker than the drops on the glyph.
const POOL: f32 = 1.0;

/// How strongly `stain` shows, 0 to 1: a few drops already read, a soaked cell is all blood.
fn strength(stain: &Stain) -> f32 {
    (f32::from(stain.amount) / f32::from(CAP))
        .clamp(0.0, 1.0)
        .sqrt()
}

/// How dry `stain` is at `now`, 0 (fresh) to 1 (dry).
fn dryness(stain: &Stain, now: GameTime) -> f32 {
    let age = now.since(stain.spilled).as_seconds() as f32;
    (age / DRY_SECONDS).clamp(0.0, 1.0).sqrt()
}

/// The colour of `stain` at `now`: its blood as spilled, drying toward a darker brown.
pub fn color(stain: &Stain, now: GameTime) -> Rgb {
    let c = stain.color;
    let dried = Rgb::new(
        channel(f32::from(c.r) * 0.55 + 10.0),
        channel(f32::from(c.g) * 0.5 + 6.0),
        channel(f32::from(c.b) * 0.45 + 3.0),
    );
    lerp(c, dried, dryness(stain, now))
}

/// A cell's glyph and background colours (`fg`, `bg`) with `stain` on it at `now`. On a `wall`
/// the tint is lighter; with an `object` on top, only the background takes it (the object lies
/// in the blood).
pub fn stained(
    (fg, bg): (Rgb, Rgb),
    stain: &Stain,
    now: GameTime,
    wall: bool,
    object: bool,
) -> (Rgb, Rgb) {
    let blood = color(stain, now);
    let pool = scale(blood, POOL);
    let s = strength(stain);
    let (to_fg, to_bg) = if wall {
        (WALL_FG, WALL_BG)
    } else {
        (FLOOR_FG, FLOOR_BG)
    };
    let fg = if object {
        fg
    } else {
        lerp(fg, blood, to_fg * s)
    };
    (fg, lerp(bg, pool, to_bg * s))
}

fn channel(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

fn scale(c: Rgb, f: f32) -> Rgb {
    Rgb::new(
        channel(f32::from(c.r) * f),
        channel(f32::from(c.g) * f),
        channel(f32::from(c.b) * f),
    )
}

fn lerp(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let mix = |x: u8, y: u8| channel(f32::from(x) + (f32::from(y) - f32::from(x)) * t);
    Rgb::new(mix(a.r, b.r), mix(a.g, b.g), mix(a.b, b.b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use warlock_core::blood::RED;

    const FLOOR: (Rgb, Rgb) = (Rgb::new(0x6a, 0x6a, 0x72), Rgb::new(0x0d, 0x0d, 0x10));

    fn stain(amount: u8) -> Stain {
        Stain::new(amount, RED, GameTime::from_seconds(1000), None)
    }

    fn redness(c: Rgb) -> i32 {
        i32::from(c.r) - (i32::from(c.g) + i32::from(c.b)) / 2
    }

    #[test]
    fn more_blood_reads_redder_and_a_pool_drowns_the_glyph() {
        let now = GameTime::from_seconds(1000);
        let (few, pool) = (
            stained(FLOOR, &stain(9), now, false, false),
            stained(FLOOR, &stain(CAP), now, false, false),
        );
        assert!(redness(few.1) > redness(FLOOR.1) && redness(pool.1) > redness(few.1));
        assert!(redness(few.0) > redness(FLOOR.0) && redness(pool.0) > redness(few.0));
        // A few drops stay subtle: the dots are still mostly the floor's.
        assert!(few.0.g > 0x50, "{:?}", few.0);
        // Soaked, glyph and background are close: one dark pool.
        assert!(
            i32::from(pool.0.r).abs_diff(i32::from(pool.1.r)) < 0x30,
            "{pool:?}"
        );
    }

    #[test]
    fn walls_tint_less_and_objects_keep_their_colour() {
        let now = GameTime::from_seconds(1000);
        let floor = stained(FLOOR, &stain(50), now, false, false);
        let wall = stained(FLOOR, &stain(50), now, true, false);
        assert!(redness(wall.0) < redness(floor.0));
        let object = stained(FLOOR, &stain(50), now, false, true);
        assert_eq!(object.0, FLOOR.0);
        assert_eq!(object.1, floor.1);
    }

    #[test]
    fn blood_dries_darker_and_browner() {
        let s = stain(60);
        let fresh = color(&s, GameTime::from_seconds(1000));
        let hour = color(&s, GameTime::from_seconds(1000 + 3600));
        let dry = color(&s, GameTime::from_seconds(1000 + 24 * 3600));
        assert_eq!(fresh, RED);
        assert!(hour.r < fresh.r && dry.r < hour.r);
        assert_eq!(dry, color(&s, GameTime::from_seconds(1000 + 48 * 3600)));
        // Browner: green keeps more of itself than red does.
        let share = |c: Rgb| f32::from(c.g) / f32::from(c.r);
        assert!(share(dry) > share(fresh));
    }
}
