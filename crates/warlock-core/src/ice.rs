//! Ice (spec §5 "Ice"): how it grows on water in the cold, melts in the warmth, and what it holds.
//!
//! Every cell of water that freezes (terrain with a `freeze` table: still water, and running
//! water in a harder frost) has an ice thickness, kept sparsely with the level
//! ([`crate::world::LevelState`]: only cells with ice), in micrometres so that slow growth is not
//! lost to rounding. The thickness decides what the cell looks like: the thickest ice terrain
//! (`ice` in `data/terrain.toml`: thin ice from 10 mm, ice from 50 mm, thick ice from 150 mm)
//! whose thickness it reaches, else the water beneath; walking on it is walking on that terrain.
//!
//! **Growth** follows Stefan's law: the square of the thickness grows with the frost, so ice
//! thickens quickly at first and ever more slowly (it insulates the water beneath). While the
//! air is colder than the water's `below` (0 °C for still water), each [`TICK`] the square of the
//! thickness grows by [`GROWTH`] per tenth of a degree below it, times the water's `rate`
//! percent: about 5 cm in a day at -5 °C, 13 cm in three days at -10 °C. Above 0 °C it melts
//! [`MELT`] per tenth of a degree a tick (7.2 mm a day per degree). It is at most [`MAX_MM`].
//!
//! **Lazily**: nothing ticks while nobody looks. A level's ice is brought up to the clock when
//! the rules need it (as the clock moves on the player's level, and when the player arrives),
//! tick by tick from where it was left: the ticks fall on fixed moments (every ten minutes of
//! the clock, each at the temperature of its middle), so the ice at a moment is the same however
//! the time passed, by turns, rests, journeys, or a skip. Cells that start alike stay alike, so
//! catching up works through the thicknesses there are, not the cells. A level under the open
//! sky visited for the first time has had [`PRE_ROLL`] of weather already: a pond is frozen when
//! one first comes upon it in deep winter.
//!
//! **What it holds**: like real lake ice, the square of the thickness: [`HOLDS`] grams per mm²,
//! so 5 cm holds 87.5 kg (a person with a pack) and 15 cm three quarters of a tonne. An actor
//! whose weight (its own and everything it carries) is more than that may break through: the
//! chance rises from nothing at what the ice holds to certain at twice that ([`gives_way`]).

use crate::content::{Freeze, TerrainId};
use crate::temperature::Temperature;
use crate::time::{DAY, Duration, GameTime};
use serde::{Deserialize, Serialize};

/// The ice grows or melts in steps of this many seconds of the clock, at fixed moments.
pub const TICK: u64 = 600;
/// Growth: the square of the thickness (µm²) grows this much a tick per tenth of a degree of
/// frost (Stefan's law with 2.4 cm per √(°C · day): 5.76 cm² per degree-day).
pub const GROWTH: u64 = 400_000;
/// Melting: µm a tick per tenth of a degree above 0 °C (7.2 mm per degree a day).
pub const MELT: u32 = 5;
/// The thickest ice, in mm.
pub const MAX_MM: u16 = 1000;
/// The thickest ice, in µm.
pub const MAX: u32 = MAX_MM as u32 * 1000;
/// The hardest frost water may need to freeze (`freeze.below`), in °C.
pub const COLDEST_FREEZE: i8 = -30;
/// What ice holds: grams per mm² of its thickness squared (Gold's formula, `A` = 3.5 kg/cm²).
pub const HOLDS: u64 = 35;
/// How much weather a level under the open sky has had when it is first visited.
pub const PRE_ROLL: Duration = Duration::seconds(30 * DAY);
/// The ice an instant frost leaves (the dev action `freeze:x,y,r`), in µm: 10 cm, which holds a
/// laden person.
pub const FROST: u32 = 100_000;

/// The ice on one cell of water.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ice {
    /// The water beneath, which the cell shows while the ice is too thin to look like ice and
    /// when it is gone.
    pub water: TerrainId,
    /// How thick it is, in µm (more than 0).
    pub um: u32,
}

impl Ice {
    /// Its thickness in whole mm, rounded down.
    pub fn mm(&self) -> u32 {
        self.um / 1000
    }
}

/// What an explosion does to the ice in its burst ([`crate::World`]'s hook for fireballs and
/// blasts; heat for now only through the dev action `thaw:x,y,r`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shock {
    /// Heat (a fireball): melts all of it.
    Heat,
    /// A blast without heat: shatters ice thinner than this many µm.
    Blast(u32),
}

impl Shock {
    /// Whether it breaks ice `um` thick.
    pub fn breaks(self, um: u32) -> bool {
        match self {
            Shock::Heat => true,
            Shock::Blast(up_to) => um < up_to,
        }
    }
}

/// Ice `um` thick after one [`TICK`] at temperature `t` on water that freezes by `freeze`: it
/// grows while the air is colder than the water's `below`, melts above 0 °C, and keeps between.
pub fn tick(um: u32, t: Temperature, freeze: &Freeze) -> u32 {
    let tenths = t.tenths();
    let below = i32::from(freeze.below) * 10;
    if tenths < below {
        let frost = u64::from((below - tenths).unsigned_abs());
        let square = u64::from(um).pow(2) + GROWTH * frost * u64::from(freeze.rate) / 100;
        u32::try_from(square.isqrt()).unwrap_or(MAX).min(MAX)
    } else if tenths > 0 {
        um.saturating_sub(MELT.saturating_mul(tenths.unsigned_abs()))
    } else {
        um
    }
}

/// What ice `um` thick holds without giving way, in grams.
pub fn holds(um: u32) -> u64 {
    HOLDS * u64::from(um).pow(2) / 1_000_000
}

/// The chance in percent that ice `um` thick gives way under `load` grams: none up to what it
/// holds, then rising evenly to certain at twice that.
pub fn gives_way(um: u32, load: u64) -> u32 {
    let holds = holds(um);
    if load <= holds {
        0
    } else if load >= 2 * holds {
        100
    } else {
        ((load - holds) * 100 / holds) as u32
    }
}

/// The ticks from the one after `from` to `to` (seconds since the epoch), as tick numbers: tick
/// `k` falls at `k · TICK` and stands for the ten minutes before it.
pub fn ticks(from: u64, to: u64) -> std::ops::RangeInclusive<u64> {
    from / TICK + 1..=to / TICK
}

/// The moment whose temperature tick `k` takes: the middle of its ten minutes.
pub fn tick_moment(k: u64) -> u64 {
    (k * TICK).saturating_sub(TICK / 2)
}

/// The last tick at or before `time`: what the ice is as of then.
pub fn tick_floor(time: GameTime) -> GameTime {
    GameTime::from_seconds(time.seconds() / TICK * TICK)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::Dice;

    const STILL: Freeze = Freeze {
        below: 0,
        rate: 100,
        plunge: Dice::fixed(1),
    };
    const RUNNING: Freeze = Freeze {
        below: -6,
        rate: 50,
        plunge: Dice::fixed(1),
    };

    /// Ice after `ticks` ticks at `c` °C, from `um`.
    fn after(mut um: u32, c: i32, ticks: u64, freeze: &Freeze) -> u32 {
        for _ in 0..ticks {
            um = tick(um, Temperature::degrees(c), freeze);
        }
        um
    }

    const PER_DAY: u64 = DAY / TICK;

    #[test]
    fn still_water_freezes_like_a_lake_quickly_at_first_then_ever_more_slowly() {
        // A day at -5 °C: about 5 cm (Stefan: 2.4 cm √(5 degree-days) = 5.4 cm).
        let day = after(0, -5, PER_DAY, &STILL);
        assert!((52_000..=54_000).contains(&day), "{day}");
        // Three days at -10 °C: about 13 cm.
        let three = after(0, -10, 3 * PER_DAY, &STILL);
        assert!((128_000..=132_000).contains(&three), "{three}");
        // The second day adds less than the first.
        let two = after(0, -5, 2 * PER_DAY, &STILL);
        assert!(two - day < day, "{day} {two}");
        // Colder is faster.
        assert!(after(0, -10, PER_DAY, &STILL) > day);
        // Never thicker than a metre.
        assert_eq!(after(0, -30, 1000 * PER_DAY, &STILL), MAX);
    }

    #[test]
    fn it_keeps_at_freezing_and_melts_in_the_warmth() {
        assert_eq!(after(40_000, 0, PER_DAY, &STILL), 40_000);
        // 7.2 mm a day per degree.
        assert_eq!(after(40_000, 1, PER_DAY, &STILL), 40_000 - 7_200);
        assert_eq!(after(40_000, 5, PER_DAY, &STILL), 4_000);
        assert_eq!(after(40_000, 10, PER_DAY, &STILL), 0);
        assert_eq!(after(0, 20, PER_DAY, &STILL), 0);
    }

    #[test]
    fn running_water_freezes_only_in_a_harder_frost_and_more_slowly() {
        assert_eq!(after(0, -5, PER_DAY, &RUNNING), 0);
        // Between its freezing point and 0 °C ice on it neither grows nor melts.
        assert_eq!(after(30_000, -3, PER_DAY, &RUNNING), 30_000);
        let running = after(0, -16, PER_DAY, &RUNNING);
        let still = after(0, -10, PER_DAY, &STILL);
        // Ten degrees of frost below its freezing point at half the pace.
        assert!(running < still && running > still / 2, "{running} {still}");
    }

    #[test]
    fn five_centimetres_hold_a_person_with_a_pack() {
        assert_eq!(holds(50_000), 87_500);
        assert_eq!(holds(150_000), 787_500);
        assert_eq!(holds(20_000), 14_000);
        // A person of 70 kg with 10 kg in their pack crosses 5 cm, not 4.
        assert_eq!(gives_way(50_000, 80_000), 0);
        assert!(gives_way(40_000, 80_000) > 0);
        // Rising evenly to certain at twice what it holds.
        assert_eq!(gives_way(50_000, 87_500), 0);
        assert_eq!(gives_way(50_000, 131_250), 50);
        assert_eq!(gives_way(50_000, 175_000), 100);
        assert_eq!(gives_way(20_000, 70_000), 100);
        // A heavily laden dwarf falls in where a light elf crosses.
        let (elf, dwarf) = (70_000 + 8_000, 70_000 + 45_000);
        assert_eq!(gives_way(55_000, elf), 0);
        assert!(gives_way(55_000, dwarf) > 0);
        // Nothing holds on no ice.
        assert_eq!(gives_way(0, 1), 100);
    }

    #[test]
    fn ticks_fall_on_fixed_moments_however_time_is_split() {
        assert_eq!(ticks(0, 599).count(), 0);
        assert_eq!(ticks(0, 600), 1..=1);
        assert_eq!(ticks(599, 1200), 1..=2);
        assert_eq!(ticks(600, 1199).count(), 0);
        // Split anywhere, the ticks are the same.
        let all: Vec<u64> = ticks(1234, 98_765).collect();
        let split: Vec<u64> = ticks(1234, 50_000).chain(ticks(50_000, 98_765)).collect();
        assert_eq!(all, split);
        assert_eq!(tick_moment(2), 900);
    }
}
