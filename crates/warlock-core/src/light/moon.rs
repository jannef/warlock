//! Moonlight on levels under the open sky (spec §5 "Moonlight"), and the sky as a whole
//! ([`Sky`]).
//!
//! **Moons** are content ([`MoonDef`], `data/sky.toml`), each with its own cycle from new moon to
//! new moon ([`MoonDef::phase_at`]).
//!
//! **Path**: a moon walks the sun's path ([`Sun::at_minute`]), trailing the sun by its phase of a
//! day: a new moon rises and sets with the sun, a first quarter rises at noon and stands high at
//! dusk, a full moon rises at sunset and stands high at midnight, and a last quarter rises at
//! midnight. On a 30-day cycle it rises 48 minutes later each day. Its direction, its elevation,
//! and the shadows it casts are the sun's at that minute of the path.
//!
//! **Light**: its `color` (direct light, shaded by tall terrain like sunlight) and its `sky`
//! (on every cell under the sky, shaded or not), both scaled by
//! - the lit fraction of its disc, `(1 − cos phase) / 2`: none at new moon, all at full moon;
//! - its height ([`RISE`]): fading in over its first degrees above the horizon, all from 40°;
//! - the daylight, which drowns it: none while the sun is up, all once the sun is 6° below the
//!   horizon ([`DUSK_DEPTH`]);
//! - its apparent size: the square of its `size` over [`STANDARD_SIZE`].
//!
//! Integer math only (the sun's fixed-point cosine and path), so moonlight is the same on every
//! platform.
//!
//! **The sky** ([`Sky::at`]) puts the sun and every moon of a moment together: it is the one
//! place where light from the sky is assembled, so anything that dims the whole sky (clouds)
//! belongs there.

use super::Light;
use super::sun::{ONE, ShadowSteps, Sun, cos};
use crate::content::{MoonDef, PHASE_ONE, STANDARD_SIZE};
use crate::time::{DAY, GameTime, MINUTE};
use serde::Serialize;

/// Moonlight by the moon's elevation (tenths of a degree), in per mille of its full light: it
/// fades in over its first degrees above the horizon and is full from 40°.
const RISE: [(i32, u32); 6] = [
    (0, 0),
    (20, 150),
    (60, 400),
    (120, 650),
    (250, 900),
    (400, 1000),
];

/// How deep the sun must be below the horizon (tenths of a degree) before the moons shine
/// fully: their light fades in through the evening twilight and out through the morning's.
pub const DUSK_DEPTH: i32 = 60;

/// A moon at one moment: where it stands, how full it is, and the light it gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Moon {
    /// The moon's index in [`crate::content::Content::moons`].
    pub index: u8,
    /// How far through its cycle it is, in 1/[`PHASE_ONE`]: 0 new, a half full.
    pub phase: u32,
    /// Height above the horizon in tenths of a degree; negative below it.
    pub elevation: i16,
    /// The shadow a cell one unit tall casts, pointing away from the moon, in
    /// 1/[`super::SHADOW_ONE`] cells: `[0, 0]` while it is down.
    pub shadow: [i16; 2],
    /// Direct moonlight on a cell it reaches; dark while it is down, new, or drowned by daylight.
    pub light: Light,
    /// Light from the sky around the moon on every cell under it, shaded or not.
    pub sky: Light,
}

impl Moon {
    /// The moon `def` (index `index`) at `time`, with `sun` the sun of that moment.
    pub fn at(time: GameTime, sun: &Sun, def: &MoonDef, index: u8) -> Moon {
        let phase = def.phase_at(time);
        // It trails the sun by its phase of a day.
        let lag = u64::from(phase) * DAY / u64::from(PHASE_ONE);
        let second = (u64::from(time.second_of_day()) + DAY - lag) % DAY;
        let path = Sun::at_minute((second / MINUTE) as u32);
        let up = path.shadow != [0, 0];
        let night = (-i32::from(sun.elevation)).clamp(0, DUSK_DEPTH) as u128;
        let size = u128::from(def.size);
        let num = if up {
            u128::from(lit_fraction(phase)) * u128::from(rise(path.elevation)) * night * size * size
        } else {
            0
        };
        let standard = u128::from(STANDARD_SIZE);
        let den = u128::from(PHASE_ONE) * 1000 * DUSK_DEPTH as u128 * standard * standard;
        let scale = |light: Light| {
            let [r, g, b, uv]: [u16; 4] = light.into();
            let c = |c: u16| u16::try_from(u128::from(c) * num / den).unwrap_or(u16::MAX);
            Light::new(c(r), c(g), c(b), c(uv))
        };
        Moon {
            index,
            phase,
            elevation: path.elevation,
            shadow: if up { path.shadow } else { [0, 0] },
            light: scale(def.color),
            sky: scale(def.sky),
        }
    }

    /// Whether the moon is above the horizon (it may still give no light: new, or by day).
    pub fn is_up(&self) -> bool {
        self.shadow != [0, 0]
    }

    /// Whether its light falls on the land: it is up, not new, and the sun does not drown it.
    pub fn shines(&self) -> bool {
        !self.light.is_dark()
    }

    /// The lit fraction of its disc, in 1/65 536.
    pub fn lit_fraction(&self) -> u32 {
        lit_fraction(self.phase)
    }

    /// Its phase by name.
    pub fn phase_name(&self) -> MoonPhase {
        MoonPhase::of(self.phase)
    }

    /// The cells a caster `height` cells tall shades from the moon, as offsets from it, nearest
    /// first (the line [`Sun::shadow_steps`] draws). Empty while the moon is down.
    pub fn shadow_steps(&self, height: u8) -> ShadowSteps {
        ShadowSteps::new(self.shadow, height)
    }
}

/// The lit fraction of a moon's disc at `phase` (1/[`PHASE_ONE`] of its cycle), in 1/65 536:
/// `(1 − cos phase) / 2`.
fn lit_fraction(phase: u32) -> u32 {
    ((ONE - cos(i64::from(phase))) / 2) as u32
}

/// Per mille of the full moonlight at `elevation` (tenths of a degree); see [`RISE`].
fn rise(elevation: i16) -> u32 {
    let e = i32::from(elevation);
    if e <= 0 {
        return 0;
    }
    for pair in RISE.windows(2) {
        let ((lo, a), (hi, b)) = (pair[0], pair[1]);
        if e <= hi {
            return a + (b - a) * (e - lo) as u32 / (hi - lo) as u32;
        }
    }
    RISE[RISE.len() - 1].1
}

/// The eight phases a moon is named by, each an eighth of its cycle centred on its moment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum MoonPhase {
    New,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    Full,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
}

impl MoonPhase {
    const ALL: [MoonPhase; 8] = [
        MoonPhase::New,
        MoonPhase::WaxingCrescent,
        MoonPhase::FirstQuarter,
        MoonPhase::WaxingGibbous,
        MoonPhase::Full,
        MoonPhase::WaningGibbous,
        MoonPhase::LastQuarter,
        MoonPhase::WaningCrescent,
    ];

    /// The phase at `phase` (1/[`PHASE_ONE`] of the cycle, 0 new).
    pub fn of(phase: u32) -> Self {
        let eighth = (u64::from(phase % PHASE_ONE) * 16 + u64::from(PHASE_ONE)) / 2;
        Self::ALL[(eighth / u64::from(PHASE_ONE)) as usize % 8]
    }

    pub fn name(self) -> &'static str {
        match self {
            MoonPhase::New => "New moon",
            MoonPhase::WaxingCrescent => "Waxing crescent",
            MoonPhase::FirstQuarter => "First quarter",
            MoonPhase::WaxingGibbous => "Waxing gibbous",
            MoonPhase::Full => "Full moon",
            MoonPhase::WaningGibbous => "Waning gibbous",
            MoonPhase::LastQuarter => "Last quarter",
            MoonPhase::WaningCrescent => "Waning crescent",
        }
    }
}

/// The sun and every moon at one moment, over a level under the open sky.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Sky {
    pub sun: Sun,
    /// Every moon, in content order (up or not).
    pub moons: Vec<Moon>,
}

impl Sky {
    /// The sky at `time` with the moons `moons`. The one place light from the sky is assembled.
    pub fn at(time: GameTime, moons: &[MoonDef]) -> Self {
        let sun = Sun::at(time);
        let moons = (moons.iter().enumerate())
            .map(|(i, def)| Moon::at(time, &sun, def, i as u8))
            .collect();
        Self { sun, moons }
    }

    /// Light from the whole sky on every cell under it, shaded or not: the sun's sky light plus
    /// every moon's.
    pub fn glow(&self) -> Light {
        (self.moons.iter()).fold(self.sun.sky, |acc, m| acc.saturating_add(m.sky))
    }

    /// The moons' light on open ground: direct and sky light of every moon together.
    pub fn moonlight(&self) -> Light {
        (self.moons.iter()).fold(Light::DARK, |acc, m| {
            acc.saturating_add(m.light).saturating_add(m.sky)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The built-in moon's numbers (pinned here so the tests pin the math, not the content).
    fn moon() -> MoonDef {
        MoonDef {
            name: "Moon".into(),
            color: Light::new(48, 28, 66, 0),
            sky: Light::new(11, 7, 15, 0),
            size: STANDARD_SIZE,
            period_days: 30,
            phase_offset_days: 0,
        }
    }

    fn at(day: u32, hour: u32, minute: u32) -> GameTime {
        GameTime::from_calendar(1, 1, day, hour, minute).expect("valid")
    }

    fn moon_at(def: &MoonDef, time: GameTime) -> Moon {
        Moon::at(time, &Sun::at(time), def, 0)
    }

    #[test]
    fn the_moon_is_pinned() {
        // FNV-1a over the moon every ten minutes of a whole cycle: integer math, so the same on
        // every platform. A deliberate change to the moon updates this value.
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        let def = moon();
        for step in 0..(30 * 24 * 6) {
            let m = moon_at(&def, GameTime::from_seconds(step * 600));
            let words = [m.elevation, m.shadow[0], m.shadow[1]].map(|w| w as u16);
            let [r, g, b, uv]: [u16; 4] = m.light.into();
            let [sr, sg, sb, suv]: [u16; 4] = m.sky.into();
            let values = [r, g, b, uv, sr, sg, sb, suv];
            let bytes = (m.phase.to_le_bytes().into_iter())
                .chain(words.iter().flat_map(|w| w.to_le_bytes()))
                .chain(values.iter().flat_map(|c| c.to_le_bytes()));
            for byte in bytes {
                hash = (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        assert_eq!(hash, 0x0def_f2cc_43d9_db7e, "moon hash {hash:#018x}");
    }

    #[test]
    fn the_phases_wax_and_wane_over_the_cycle() {
        let def = moon();
        let lit = |day| moon_at(&def, at(day, 0, 0)).lit_fraction();
        let half = |day| {
            moon_at(&def, at(day, 12, 0))
                .lit_fraction()
                .abs_diff(ONE as u32 / 2)
        };
        assert_eq!(lit(1), 0, "new on the first");
        assert_eq!(lit(16), ONE as u32, "full on the sixteenth");
        assert!(half(8) < 10, "first quarter: half lit");
        assert!(half(23) < 10, "last quarter: half lit");
        for day in 1..16 {
            assert!(lit(day + 1) > lit(day), "waxes on day {day}");
        }
        for day in 16..30 {
            assert!(lit(day + 1) < lit(day), "wanes on day {day}");
        }
        let names: Vec<_> = [1, 4, 8, 12, 16, 19, 23, 27, 30]
            .map(|d| moon_at(&def, at(d, 0, 0)).phase_name().name())
            .into();
        assert_eq!(
            names,
            [
                "New moon",
                "Waxing crescent",
                "First quarter",
                "Waxing gibbous",
                "Full moon",
                "Waning gibbous",
                "Last quarter",
                "Waning crescent",
                "New moon"
            ]
        );
    }

    /// When the moon first rises after `hour` on `day` (within a day).
    fn rises(def: &MoonDef, day: u32, hour: u32) -> GameTime {
        let start = at(day, hour, 0);
        (1..24 * 60)
            .map(|m| start + crate::time::Duration::minutes(m))
            .find(|t| {
                let before = GameTime::from_seconds(t.seconds() - MINUTE);
                moon_at(def, *t).is_up() && !moon_at(def, before).is_up()
            })
            .expect("the moon rises once a day")
    }

    #[test]
    fn the_moon_rises_later_each_day_and_a_full_moon_stands_high_at_midnight() {
        let def = moon();
        // New moon: with the sun. First quarter: at noon. Full: at sunset. Last quarter: at
        // midnight.
        assert_eq!(rises(&def, 1, 1).hour(), 6);
        assert_eq!(rises(&def, 8, 1).hour(), 12);
        assert_eq!(rises(&def, 15, 12).hour(), 17, "full the next midnight");
        assert_eq!(rises(&def, 16, 1).hour(), 18);
        assert_eq!(rises(&def, 23, 12).hour(), 0);
        // On a 30-day cycle, about 50 minutes later each day (24 hours / 29).
        let (a, b) = (rises(&def, 3, 1), rises(&def, 4, 1));
        let later = b.since(a).as_seconds() / MINUTE - 24 * 60;
        assert!((49..=50).contains(&later), "{a} then {b}");
        let midnight = moon_at(&def, at(16, 0, 0));
        assert_eq!(midnight.elevation, 600, "as high as the sun at noon");
        assert_eq!(midnight.light, def.color, "all of its light");
        assert_eq!(midnight.sky, def.sky);
        assert_eq!(midnight.shadow, Sun::at(at(1, 12, 0)).shadow);
        // In the evening it stands in the east, so shadows point west; in the morning east.
        let evening = moon_at(&def, at(16, 21, 0));
        let morning = moon_at(&def, at(16, 3, 0));
        assert!(evening.shadow[0] < 0 && morning.shadow[0] > 0);
        assert!(
            !moon_at(&def, at(16, 12, 0)).is_up(),
            "a full moon is down at noon"
        );
    }

    #[test]
    fn moonlight_fades_with_height_daylight_and_phase() {
        let def = moon();
        // A full moon rising at dusk brightens as it climbs and the twilight fades.
        let mut last = 0;
        for minute in (18 * 60..21 * 60).step_by(10) {
            let m = moon_at(&def, at(16, minute / 60, minute % 60));
            assert!(m.light.luma() >= last, "{minute}: {m:?}");
            last = m.light.luma();
        }
        assert!(!moon_at(&def, at(16, 18, 0)).is_up(), "not risen yet");
        // Daylight drowns it: a first quarter moon up all afternoon gives nothing until dusk.
        assert!(moon_at(&def, at(8, 15, 0)).is_up());
        assert!(!moon_at(&def, at(8, 15, 0)).shines());
        assert!(moon_at(&def, at(8, 18, 40)).shines());
        // A new moon never shines; brighter toward full.
        for minute in (0..24 * 60).step_by(7) {
            assert!(!moon_at(&def, at(1, minute / 60, minute % 60)).shines());
        }
        let luma = |day, hour| moon_at(&def, at(day, hour, 0)).light.luma();
        assert!(luma(12, 21) < luma(13, 23) && luma(13, 23) < luma(16, 0));
        // The light stays the moon's hue: faintly purple.
        let [r, g, b] = def.color.rgb;
        assert!(b > r && r > g);
    }

    #[test]
    fn size_scales_the_light_by_area_and_moons_add_up() {
        let big = MoonDef {
            size: 200,
            ..moon()
        };
        let small = MoonDef {
            name: "Small".into(),
            size: 50,
            period_days: 7,
            phase_offset_days: 3,
            ..moon()
        };
        let time = at(16, 0, 0);
        assert_eq!(
            moon_at(&big, time).light,
            Light::new(4 * 48, 4 * 28, 4 * 66, 0)
        );
        let sky = Sky::at(time, &[moon(), small.clone()]);
        assert_eq!(sky.moons.len(), 2);
        let (a, b) = (sky.moons[0], sky.moons[1]);
        assert_eq!((a.index, b.index), (0, 1));
        assert_ne!(a.phase, b.phase, "each has its own cycle");
        assert_ne!(a.shadow, b.shadow, "and its own place in the sky");
        let both = a.sky.saturating_add(b.sky);
        assert_eq!(sky.glow(), sky.sun.sky.saturating_add(both));
        let light = a.light.saturating_add(b.light).saturating_add(both);
        assert_eq!(sky.moonlight(), light);
        assert!(Sky::at(time, &[]).moonlight().is_dark());
        assert_eq!(Sky::at(time, &[]).glow(), Sun::at(time).sky);
    }

    #[test]
    fn the_moon_is_deterministic() {
        let def = moon();
        for seconds in (0..40 * DAY).step_by(3607) {
            let t = GameTime::from_seconds(seconds);
            assert_eq!(moon_at(&def, t), moon_at(&def, t));
            let moons = std::slice::from_ref(&def);
            assert_eq!(Sky::at(t, moons), Sky::at(t, moons));
        }
    }
}
