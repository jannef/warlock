//! Char on the map grid (spec §4.8 "Blood", §8 "Blood"): a scorched cell's colours shift toward
//! its char ([`warlock_core::scorch`]: brown soot where a little has burned, near black where it
//! is charred through) by how much has burned there, before the grid shader lights them, as
//! [`super::blood::stained`] does for blood. A floor's dots darken into the char; a wall's face
//! is tinted more lightly. Blood is laid over the char (`draw_map` applies this first). Reads the
//! perception snapshot only
//! ([`PerceptionView::scorch`](warlock_core::perception::PerceptionView::scorch)); the `blood`
//! switch hides it with the blood.

use warlock_core::blood::{CAP, Stain};
use warlock_core::color::Rgb;

/// The ground under the char: its colour this much darker than the soot on the glyph, so even a
/// little soot darkens the ground rather than lighting it brown.
const GROUND: f32 = 0.35;
/// How far a floor charred through gives way to the char (0 to 1): its dots and background.
const FLOOR_FG: f32 = 0.9;
const FLOOR_BG: f32 = 0.7;
/// The same on a wall's face.
const WALL_FG: f32 = 0.7;
const WALL_BG: f32 = 0.55;

/// How strongly `stain` shows, 0 to 1: a little soot already reads.
fn strength(stain: &Stain) -> f32 {
    (f32::from(stain.amount) / f32::from(CAP))
        .clamp(0.0, 1.0)
        .sqrt()
}

/// A cell's glyph and background colours (`fg`, `bg`) with char `stain` on it. On a `wall` the
/// tint is lighter; with an `object` on top, only the background takes it.
pub fn charred((fg, bg): (Rgb, Rgb), stain: &Stain, wall: bool, object: bool) -> (Rgb, Rgb) {
    let s = strength(stain);
    let (to_fg, to_bg) = if wall {
        (WALL_FG, WALL_BG)
    } else {
        (FLOOR_FG, FLOOR_BG)
    };
    let fg = if object {
        fg
    } else {
        lerp(fg, stain.color, to_fg * s)
    };
    (fg, lerp(bg, scale(stain.color, GROUND), to_bg * s))
}

fn scale(c: Rgb, f: f32) -> Rgb {
    let ch = |v: u8| (f32::from(v) * f).round().clamp(0.0, 255.0) as u8;
    Rgb::new(ch(c.r), ch(c.g), ch(c.b))
}

fn lerp(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let mix = |x: u8, y: u8| {
        (f32::from(x) + (f32::from(y) - f32::from(x)) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Rgb::new(mix(a.r, b.r), mix(a.g, b.g), mix(a.b, b.b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use warlock_core::time::GameTime;

    const FLOOR: (Rgb, Rgb) = (Rgb::new(0x6a, 0x6a, 0x72), Rgb::new(0x0d, 0x0d, 0x10));
    const SOOT: Rgb = Rgb::new(0x3a, 0x2a, 0x1e);
    const CHARRED: Rgb = Rgb::new(0x0c, 0x0a, 0x08);

    fn stain(amount: u8, color: Rgb) -> Stain {
        Stain::new(amount, color, GameTime::from_seconds(0), None)
    }

    fn lum(c: Rgb) -> u32 {
        u32::from(c.r) + u32::from(c.g) + u32::from(c.b)
    }

    #[test]
    fn more_char_darkens_the_floor_s_dots() {
        let soot = charred(FLOOR, &stain(10, SOOT), false, false);
        let through = charred(FLOOR, &stain(CAP, CHARRED), false, false);
        assert!(lum(soot.0) < lum(FLOOR.0) && lum(through.0) < lum(soot.0));
        // A little soot stays subtle: the dots are still mostly the floor's.
        assert!(soot.0.g > 0x48, "{:?}", soot.0);
        // Browner where a little has burned (red over blue), and the ground no lighter.
        let warmth = |c: Rgb| i32::from(c.r) - i32::from(c.b);
        assert!(warmth(soot.0) > warmth(FLOOR.0));
        assert!(lum(soot.1) <= lum(FLOOR.1) + 3, "{:?}", soot.1);
    }

    #[test]
    fn walls_tint_less_and_objects_keep_their_colour() {
        let floor = charred(FLOOR, &stain(60, CHARRED), false, false);
        let wall = charred(FLOOR, &stain(60, CHARRED), true, false);
        assert!(lum(wall.0) > lum(floor.0));
        let object = charred(FLOOR, &stain(60, CHARRED), false, true);
        assert_eq!((object.0, object.1), (FLOOR.0, floor.1));
    }
}
