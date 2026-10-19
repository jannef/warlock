//! Sunlight and sky light on levels under the open sky (spec §5 "Sunlight").
//!
//! **The sun's path** is one table entry per minute of the day ([`Sun::at`]), computed at compile
//! time with integer math only (a fixed-point sine), so it is the same on every platform. The sun
//! rises at 06:00 in the east (+x), stands high in the south (+y) at noon, and sets at 18:00 in
//! the west. Its hour angle runs from 0 at sunrise to half a turn at sunset; its horizontal
//! direction is `(cos, sin)` of that angle, and its elevation is [`NOON_ELEVATION`] times the
//! sine, so it is below the horizon at night and twilight is short.
//!
//! **Light** follows the elevation: [`Sun::light`] is warm, dim, and orange just after sunrise and
//! before sunset, and bright neutral white once the sun is high; [`Sun::sky`] is a dimmer bluish
//! light from the whole sky, which also glows faintly in the twilight while the sun is less than
//! 6° below the horizon. At night both are dark, and the moons light the land
//! ([`super::Moon`], which walk the same path).
//!
//! **Shadows** are directional: terrain stands [`crate::content::TerrainDef::height`] cells tall,
//! and a cell `h` tall casts a shadow `h ×` [`Sun::shadow`] long, away from the sun: long at dawn
//! and dusk, short at noon. The shadow covers the cells an integer line from the caster passes
//! through ([`Sun::shadow_steps`]); a cell is in shade if, marching from it toward the sun, a
//! cell taller than itself is within that cell's shadow ([`Sunlight`]). Moons cast their shadows
//! the same way. Roofed terrain never receives sun, moon, or sky.

use super::Light;
use super::bounce::Area;
use crate::geom::Pos;
use crate::time::{GameTime, MINUTES_PER_DAY};
use serde::Serialize;

/// The sun's elevation at noon, in degrees (it rises and sets at 0°).
pub const NOON_ELEVATION: i32 = 60;

/// [`Sun::shadow`] is in these fractions of a cell.
pub const SHADOW_ONE: i32 = 256;

/// Longest shadow a cell of height 1 casts, in cells (the sun's first and last minutes would
/// otherwise cast shadows across any level).
pub const MAX_SHADOW: i32 = 12;

/// The sun and the sky at one minute of the day.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Sun {
    /// Height above the horizon in tenths of a degree; negative below it.
    pub elevation: i16,
    /// The shadow a cell one unit tall casts, pointing away from the sun, in 1/[`SHADOW_ONE`]
    /// cells (`x` east, `y` south): `[0, 0]` while the sun is down.
    pub shadow: [i16; 2],
    /// Direct sunlight on a cell it reaches; dark while the sun is down.
    pub light: Light,
    /// Light from the whole sky on every cell under it, shaded or not; dark at night.
    pub sky: Light,
}

impl Sun {
    /// The sun at `time` (only the minute of day counts; seasons may come later).
    pub fn at(time: GameTime) -> Sun {
        SUN_PATH[time.minute_of_day() as usize]
    }

    /// The sun at `minute` of the day, 0 (midnight) to 1439.
    pub fn at_minute(minute: u32) -> Sun {
        SUN_PATH[(minute % MINUTES_PER_DAY) as usize]
    }

    /// Whether the sun is up: it shines and casts shadows.
    pub fn is_up(&self) -> bool {
        !self.light.is_dark()
    }

    /// Night: neither sun nor sky light.
    pub fn is_night(&self) -> bool {
        self.light.is_dark() && self.sky.is_dark()
    }

    /// The cells a caster `height` cells tall shades, as offsets from it, nearest first: an
    /// integer line away from the sun, one step per cell along its major axis, as many steps as
    /// the shadow is long along that axis (rounded to whole cells). Empty while the sun is down.
    pub fn shadow_steps(&self, height: u8) -> ShadowSteps {
        ShadowSteps::new(self.shadow, height)
    }
}

/// Iterator over the cells of a shadow; see [`Sun::shadow_steps`].
#[derive(Clone, Debug)]
pub struct ShadowSteps {
    shadow: [i32; 2],
    major: i32,
    step: i32,
    len: i32,
}

impl ShadowSteps {
    /// The cells a caster `height` cells tall shades from a body in the sky whose shadow per
    /// unit of height is `shadow` (1/[`SHADOW_ONE`] cells; `[0, 0]` while it is down): see
    /// [`Sun::shadow_steps`].
    pub fn new(shadow: [i16; 2], height: u8) -> Self {
        let [x, y] = shadow.map(i32::from);
        let major = x.abs().max(y.abs());
        let len = if major == 0 {
            0
        } else {
            (i32::from(height) * major + SHADOW_ONE / 2) / SHADOW_ONE
        };
        Self {
            shadow: [x, y],
            major,
            step: 0,
            len,
        }
    }
}

impl Iterator for ShadowSteps {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<(i32, i32)> {
        if self.step >= self.len {
            return None;
        }
        self.step += 1;
        // Rounding halves away from zero, a line and its mirror image visit mirrored cells.
        let at = |c: i32| round_div(i64::from(self.step * c), i64::from(self.major)) as i32;
        Some((at(self.shadow[0]), at(self.shadow[1])))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = (self.len - self.step).max(0) as usize;
        (left, Some(left))
    }
}

impl ExactSizeIterator for ShadowSteps {}

/// Where the sun (or a moon) shines within an area this turn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Sunlight {
    area: Area,
    light: Light,
    /// Row-major over `area`: whether the light reaches the cell.
    lit: Vec<bool>,
    /// The transparent cells the light reaches, row-major, each with its light.
    pub cells: Vec<(Pos, Light)>,
}

impl Sunlight {
    /// Where the `light` of a body in the sky whose shadow per unit of height is `shadow` (the
    /// sun's or a moon's; `[0, 0]` while it is down) falls within `area`: on every cell `exposed`
    /// (not roofed) that no taller cell shades. `height` gives a cell's height (0 off the level)
    /// and `tallest` bounds it, which bounds how far shadows reach.
    pub fn compute(
        shadow: [i16; 2],
        light: Light,
        area: Area,
        tallest: u8,
        height: impl Fn(Pos) -> u8,
        exposed: impl Fn(Pos) -> bool,
        is_opaque: impl Fn(Pos) -> bool,
    ) -> Self {
        let up = shadow != [0, 0] && !light.is_dark();
        let steps: Vec<(i32, i32)> = ShadowSteps::new(shadow, tallest).collect();
        // `reach[d]`: how many steps the shadow of a cell `d` taller than the receiver covers.
        let reach: Vec<usize> = (0..=tallest)
            .map(|h| ShadowSteps::new(shadow, h).len())
            .collect();
        // Heights around the area, out to the longest shadow, so the march reads an array.
        let margin = steps.last().map_or(0, |&(dx, dy)| dx.abs().max(dy.abs()));
        let around = Area {
            min: Pos::new(area.min.x - margin, area.min.y - margin),
            max: Pos::new(area.max.x + margin, area.max.y + margin),
        };
        let heights: Vec<u8> = (0..around.len()).map(|i| height(around.pos(i))).collect();
        let height_at = |p: Pos| around.index(p).map_or(0, |i| heights[i]);

        // Whether the sun reaches `p`: marching toward the sun, the cell `k + 1` steps away
        // shades `p` if it is taller and `p` lies within its shadow.
        let sunny = |p: Pos| {
            if !exposed(p) {
                return false;
            }
            let own = height_at(p);
            let taller = usize::from(tallest.saturating_sub(own));
            let shaded = steps[..reach[taller]]
                .iter()
                .enumerate()
                .any(|(k, &(dx, dy))| {
                    let h = height_at(Pos::new(p.x - dx, p.y - dy));
                    h > own && k < reach[usize::from(h - own).min(taller)]
                });
            !shaded
        };
        let lit: Vec<bool> = (0..area.len()).map(|i| up && sunny(area.pos(i))).collect();
        let cells = lit
            .iter()
            .enumerate()
            .filter(|&(i, lit)| *lit && !is_opaque(area.pos(i)))
            .map(|(i, _)| (area.pos(i), light))
            .collect();
        Self {
            area,
            light,
            lit,
            cells,
        }
    }

    /// The light at `p` (opaque cells included); dark in shade, under a roof, or outside the
    /// area.
    pub fn at(&self, p: Pos) -> Light {
        match self.area.index(p) {
            Some(i) if self.lit[i] => self.light,
            _ => Light::DARK,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The table, computed at compile time with integer math.

/// Angles are in 1/65 536 of a full turn.
pub(crate) const TURN: i64 = 1 << 16;
const QUARTER: i64 = TURN / 4;
/// Sines and cosines are in 1/65 536.
pub(crate) const ONE: i64 = 1 << 16;
/// 2π in 1/65 536.
const TAU: i64 = 411_775;

/// `a / b` rounded to the nearest integer, halves away from zero; `b > 0`.
pub(super) const fn round_div(a: i64, b: i64) -> i64 {
    if a >= 0 {
        (2 * a + b) / (2 * b)
    } else {
        -((-2 * a + b) / (2 * b))
    }
}

/// An angle of `d` degrees.
const fn degrees(d: i64) -> i64 {
    round_div(d * TURN, 360)
}

/// `sin(a)` for `0 ≤ a ≤` a quarter turn: its Taylor series up to `x⁹` (error below 10⁻⁵).
const fn sin_quadrant(a: i64) -> i64 {
    let x = a * TAU / TURN;
    let x2 = x * x / ONE;
    let mut t = ONE - x2 / 72;
    t = ONE - x2 * t / ONE / 42;
    t = ONE - x2 * t / ONE / 20;
    t = ONE - x2 * t / ONE / 6;
    let s = x * t / ONE;
    if s > ONE { ONE } else { s }
}

/// `sin(a)` for any angle, exactly antisymmetric and mirrored around a quarter turn.
const fn sin(a: i64) -> i64 {
    let a = a.rem_euclid(TURN);
    if a <= QUARTER {
        sin_quadrant(a)
    } else if a <= 2 * QUARTER {
        sin_quadrant(2 * QUARTER - a)
    } else if a <= 3 * QUARTER {
        -sin_quadrant(a - 2 * QUARTER)
    } else {
        -sin_quadrant(TURN - a)
    }
}

pub(crate) const fn cos(a: i64) -> i64 {
    sin(a + QUARTER)
}

/// Sunlight by elevation (degrees): deep orange as it rises, warm gold, then bright neutral white.
const SUN_COLORS: [(i64, [i64; 3]); 7] = [
    (0, [0, 0, 0]),
    (2, [150, 60, 25]),
    (6, [230, 115, 50]),
    (12, [290, 190, 115]),
    (25, [320, 275, 215]),
    (40, [325, 312, 280]),
    (90, [325, 312, 280]),
];

/// Sky light by the sun's elevation (degrees): none until 6° below the horizon, a dim violet
/// twilight, then a bluish light that grows with the sun.
const SKY_COLORS: [(i64, [i64; 3]); 7] = [
    (-6, [0, 0, 0]),
    (-3, [16, 14, 30]),
    (0, [40, 34, 60]),
    (6, [48, 48, 78]),
    (20, [55, 66, 100]),
    (40, [58, 72, 110]),
    (90, [58, 72, 110]),
];

/// The color at elevation `angle`, interpolated between `keys`; dark below the first key.
const fn color_at(keys: &[(i64, [i64; 3]); 7], angle: i64) -> Light {
    let mut i = 0;
    while i + 1 < keys.len() {
        let (lo, hi) = (degrees(keys[i].0), degrees(keys[i + 1].0));
        if angle >= lo && angle <= hi {
            let (a, b) = (keys[i].1, keys[i + 1].1);
            let mut rgb = [0u16; 3];
            let mut c = 0;
            while c < 3 {
                rgb[c] = (a[c] + (b[c] - a[c]) * (angle - lo) / (hi - lo)) as u16;
                c += 1;
            }
            return Light::new(rgb[0], rgb[1], rgb[2], 0);
        }
        i += 1;
    }
    Light::DARK
}

/// The sun at `minute` of the day.
const fn sun_at(minute: i64) -> Sun {
    // Hour angle from sunrise, symmetric around noon: a quarter turn at noon.
    let from_noon = minute - 12 * 60;
    let offset = from_noon.abs() * TURN / (24 * 60);
    let hour = if from_noon < 0 {
        QUARTER - offset
    } else {
        QUARTER + offset
    };
    let elevation = degrees(NOON_ELEVATION as i64) * sin(hour) / ONE;
    let shadow = if elevation > 0 {
        // Shadow length per unit of height: cot(elevation), capped.
        let mut length = cos(elevation) * SHADOW_ONE as i64 / sin(elevation);
        let max = (MAX_SHADOW * SHADOW_ONE) as i64;
        if length > max {
            length = max;
        }
        // Away from the sun, whose direction is (cos, sin) of the hour angle.
        [
            (-length * cos(hour) / ONE) as i16,
            (-length * sin(hour) / ONE) as i16,
        ]
    } else {
        [0, 0]
    };
    let light = if shadow[0] == 0 && shadow[1] == 0 {
        Light::DARK
    } else {
        color_at(&SUN_COLORS, elevation)
    };
    Sun {
        elevation: round_div(elevation * 3600, TURN) as i16,
        shadow,
        light,
        sky: color_at(&SKY_COLORS, elevation),
    }
}

const fn sun_path() -> [Sun; MINUTES_PER_DAY as usize] {
    let night = Sun {
        elevation: 0,
        shadow: [0, 0],
        light: Light::DARK,
        sky: Light::DARK,
    };
    let mut table = [night; MINUTES_PER_DAY as usize];
    let mut minute = 0;
    while minute < table.len() {
        table[minute] = sun_at(minute as i64);
        minute += 1;
    }
    table
}

/// The sun at every minute of the day.
static SUN_PATH: [Sun; MINUTES_PER_DAY as usize] = sun_path();

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::GameTime;

    fn at(hour: u32, minute: u32) -> Sun {
        Sun::at(GameTime::from_calendar(1, 3, 4, hour, minute).expect("valid"))
    }

    #[test]
    fn the_sun_path_is_pinned() {
        // FNV-1a over every entry: the table is integer math evaluated at compile time, so it is
        // the same on every platform. A deliberate change to the sun updates this value.
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for sun in &SUN_PATH {
            let [r, g, b] = sun.light.rgb;
            let [sr, sg, sb] = sun.sky.rgb;
            let words = [sun.elevation, sun.shadow[0], sun.shadow[1]];
            let colors = [r, g, b, sr, sg, sb];
            let bytes = (words.iter().flat_map(|w| w.to_le_bytes()))
                .chain(colors.iter().flat_map(|c| c.to_le_bytes()));
            for byte in bytes {
                hash = (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        assert_eq!(hash, 0xf36e_8991_853c_fa0a, "sun path hash {hash:#018x}");
    }

    #[test]
    fn integer_sine_is_accurate_and_exact_at_the_quarters() {
        assert_eq!(sin(0), 0);
        assert_eq!(sin(QUARTER), ONE);
        assert_eq!(sin(2 * QUARTER), 0);
        assert_eq!(sin(3 * QUARTER), -ONE);
        assert_eq!(cos(QUARTER), 0);
        // sin 30° = 0.5, sin 60° = 0.866 025 (angles are rounded to 1/65 536 of a turn).
        assert!((sin(degrees(30)) - ONE / 2).abs() <= 4);
        assert!((sin(degrees(60)) - 56_756).abs() <= 4);
        for a in (0..TURN).step_by(97) {
            assert_eq!(sin(a), -sin(-a), "antisymmetric at {a}");
            assert_eq!(sin(a), sin(2 * QUARTER - a), "mirrored at {a}");
        }
    }

    #[test]
    fn the_sun_rises_in_the_east_and_sets_in_the_west() {
        let rise = at(6, 30);
        assert!(rise.is_up());
        assert!(rise.shadow[0] < 0, "morning shadows point west: {rise:?}");
        let noon = at(12, 0);
        assert_eq!(noon.shadow[0], 0, "noon shadows point due north");
        assert!(noon.shadow[1] < 0);
        assert_eq!(noon.elevation, 600);
        let set = at(17, 30);
        assert!(set.shadow[0] > 0, "evening shadows point east: {set:?}");
        // Morning and evening mirror each other.
        assert_eq!(set.shadow, [-rise.shadow[0], rise.shadow[1]]);
        assert_eq!((set.light, set.sky), (rise.light, rise.sky));
    }

    #[test]
    fn shadows_shorten_as_the_sun_climbs() {
        // The length in cells, not the steps along the major axis: those can grow as the shadow
        // turns toward an axis.
        let length = |s: Sun| f64::from(s.shadow[0]).hypot(f64::from(s.shadow[1]));
        let mut last = (i16::MIN, f64::MAX);
        for minute in (6 * 60 + 1)..=(12 * 60) {
            let sun = Sun::at_minute(minute);
            let now = (sun.elevation, length(sun));
            assert!(
                now.0 >= last.0 && now.1 <= last.1 + 1.5,
                "{minute}: {sun:?}"
            );
            last = now;
        }
        // Noon: cot 60° ≈ 0.58 cells per unit of height; 07:00: about 3.6.
        assert_eq!(at(12, 0).shadow_steps(1).len(), 1);
        assert_eq!(at(12, 0).shadow_steps(2).len(), 1);
        assert!((3..=4).contains(&at(7, 0).shadow_steps(1).len()));
        assert!((7..=8).contains(&at(7, 0).shadow_steps(2).len()));
        let dawn = at(6, 1);
        assert_eq!(dawn.shadow_steps(1).len(), MAX_SHADOW as usize, "capped");
    }

    #[test]
    fn light_is_warm_and_dim_low_and_bright_white_high() {
        let low = at(6, 20);
        let high = at(12, 0);
        let [r, g, b] = low.light.rgb;
        assert!(r > g && g > b, "orange at dawn: {low:?}");
        assert!(low.light.luma() < high.light.luma());
        let [r, _, b] = high.light.rgb;
        assert!(
            r >= 255 && b >= 255 && r - b < 60,
            "neutral at noon: {high:?}"
        );
        let [r, g, b] = high.sky.rgb;
        assert!(b > g && g > r, "the sky is bluish: {high:?}");
        assert!(high.sky.luma() < high.light.luma() / 3, "and dimmer");
        assert!(high.sky.luma() >= 40, "shade stays visible to normal eyes");
    }

    #[test]
    fn night_is_dark_with_a_short_twilight() {
        for (h, m) in [(0, 0), (3, 0), (5, 30), (18, 30), (22, 0), (23, 59)] {
            let sun = at(h, m);
            assert!(sun.is_night(), "{h}:{m}: {sun:?}");
            assert_eq!(sun.shadow_steps(4).len(), 0);
        }
        for (h, m) in [(5, 50), (18, 10)] {
            let sun = at(h, m);
            assert!(!sun.is_up() && !sun.is_night(), "twilight {h}:{m}: {sun:?}");
        }
        assert!(!at(6, 0).is_up(), "on the horizon");
        assert!(at(6, 1).is_up() && at(17, 59).is_up());
        assert!(!at(18, 0).is_up());
    }

    #[test]
    fn shadow_lines_are_unbroken_and_point_away_from_the_sun() {
        for minute in (6 * 60 + 1)..(18 * 60) {
            let sun = Sun::at_minute(minute);
            let mut last = (0, 0);
            for (dx, dy) in sun.shadow_steps(4) {
                assert!(
                    (dx - last.0).abs() <= 1 && (dy - last.1).abs() <= 1 && (dx, dy) != last,
                    "{minute}: step from {last:?} to {:?}",
                    (dx, dy)
                );
                assert!(dx * i32::from(sun.shadow[0]) >= 0 && dy * i32::from(sun.shadow[1]) >= 0);
                last = (dx, dy);
            }
        }
    }

    /// The cells shaded (`s`) on a map of grass (`.`), walls (`#`, 1 tall), trees (`T`, 2),
    /// boulders (`O`, 3), and roofed floor (`r`); opaque cells and roofs keep their letter.
    fn shaded(map: &[&str], lit: &Sunlight) -> Vec<String> {
        map.iter()
            .enumerate()
            .map(|(y, row)| {
                row.chars()
                    .enumerate()
                    .map(|(x, c)| match c {
                        '.' if lit.at(Pos::new(x as i32, y as i32)).is_dark() => 's',
                        c => c,
                    })
                    .collect()
            })
            .collect()
    }

    fn compute(sun: &Sun, map: &[&str]) -> Sunlight {
        let (width, height) = (map[0].len() as i32, map.len() as i32);
        let cell = |p: Pos| {
            ((0..width).contains(&p.x) && (0..height).contains(&p.y))
                .then(|| map[p.y as usize].as_bytes()[p.x as usize])
        };
        let area = Area {
            min: Pos::new(0, 0),
            max: Pos::new(width - 1, height - 1),
        };
        Sunlight::compute(
            sun.shadow,
            sun.light,
            area,
            3,
            |p| match cell(p) {
                Some(b'#') => 1,
                Some(b'T') => 2,
                Some(b'O') => 3,
                _ => 0,
            },
            |p| cell(p).is_some_and(|c| c != b'r'),
            |p| cell(p).is_none_or(|c| b"#TO".contains(&c)),
        )
    }

    fn sunlight(sun: &Sun, map: &[&str]) -> Vec<String> {
        shaded(map, &compute(sun, map))
    }

    /// The shaded cells of `rows`.
    fn shade_cells(rows: &[String]) -> Vec<(i32, i32)> {
        let mut cells = Vec::new();
        for (y, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                if c == 's' {
                    cells.push((x as i32, y as i32));
                }
            }
        }
        cells
    }

    /// A sun straight from the east (shadows due west), `length` cells per unit of height.
    fn from_the_east(length_256: i16) -> Sun {
        Sun {
            elevation: 100,
            shadow: [-length_256, 0],
            light: Light::new(300, 300, 300, 0),
            sky: Light::new(50, 60, 90, 0),
        }
    }

    #[test]
    fn shade_falls_away_from_the_sun_and_mirrors_from_morning_to_evening() {
        let row = ".".repeat(21);
        let tree = format!("{}T{}", ".".repeat(10), ".".repeat(10));
        let map = [&row[..], &row, &row, &row, &tree, &row, &row, &row, &row];
        // Noon: a short shadow due north.
        assert_eq!(shade_cells(&sunlight(&at(12, 0), &map)), [(10, 3)]);
        let mut last = usize::MAX;
        for hour in [7, 8, 9, 10, 11, 12] {
            let sun = at(hour, 0);
            let morning = shade_cells(&sunlight(&sun, &map));
            assert_eq!(morning.len(), sun.shadow_steps(2).len(), "{hour}:00");
            assert!(morning.len() <= last, "shorter as the sun climbs");
            last = morning.len();
            // The sun is in the east and south: shade lies west and north of the tree.
            assert!(
                morning.iter().all(|&(x, y)| x <= 10 && y <= 4),
                "{morning:?}"
            );
            if hour < 11 {
                assert!(morning.iter().any(|&(x, _)| x < 10), "{morning:?}");
            }
            // The evening at the same distance from noon is the mirror image.
            let mut evening = shade_cells(&sunlight(&at(24 - hour, 0), &map));
            evening.sort();
            let mut mirrored: Vec<_> = morning.iter().map(|&(x, y)| (20 - x, y)).collect();
            mirrored.sort();
            assert_eq!(evening, mirrored, "{hour}:00 and {}:00", 24 - hour);
        }
        assert!(last < 3 && shade_cells(&sunlight(&at(7, 0), &map)).len() >= 7);
        // Night: no sun anywhere.
        assert!(sunlight(&at(22, 0), &map).iter().all(|r| !r.contains('.')));
    }

    #[test]
    fn taller_casters_shade_lower_ones_and_roofs_keep_the_sun_out() {
        // 1.5 cells per unit of height: walls 2 cells, trees 3, boulders 5 (rounded).
        let sun = from_the_east(384);
        let map = ["........", "..#O....", "...O#...", "........"];
        let lit = compute(&sun, &map);
        let rows = shaded(&map, &lit);
        assert_eq!(rows[1], "ss#O....", "the boulder's shade covers the wall's");
        assert!(
            lit.at(Pos::new(2, 1)).is_dark(),
            "the boulder shades the wall"
        );
        assert_eq!(rows[2], "sssO#...", "a wall never shades a boulder");
        assert_eq!(lit.at(Pos::new(3, 2)), sun.light);
        // Roofed cells: no sun, shaded or not.
        let roofed = compute(&sun, &["....", ".rr.", "...."]);
        assert!(roofed.at(Pos::new(1, 1)).is_dark() && roofed.at(Pos::new(2, 1)).is_dark());
        assert_eq!(roofed.at(Pos::new(3, 1)), sun.light);
        assert!(
            roofed
                .cells
                .iter()
                .all(|(p, _)| p.y != 1 || p.x == 0 || p.x == 3)
        );
        // Only transparent cells are listed, row-major.
        let cells: Vec<Pos> = lit.cells.iter().map(|(p, _)| *p).collect();
        assert!(
            cells
                .windows(2)
                .all(|w| (w[0].y, w[0].x) < (w[1].y, w[1].x))
        );
        assert!(!cells.contains(&Pos::new(3, 1)), "the boulder is opaque");
    }
}
