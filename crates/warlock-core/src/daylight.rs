//! Daylight on the overworld (spec §5): how far the traveller sees by time of day, and the faint
//! night light the overworld is drawn by. The sun and the sky themselves are
//! [`crate::light::Sun`], which lights the overworld like any level under the open sky.
//! Integer math only.

use crate::light::Light;
use crate::time::{DAWN, DUSK, GameTime, MINUTE};

/// Where the day is, for the overworld's sight radius.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DayPhase {
    /// From half past dawn (06:30) to half an hour before dusk (17:30).
    Day,
    /// Half an hour either side of dawn (06:00) and dusk (18:00).
    Twilight,
    /// The rest. Moonlight is darkness for now.
    Night,
}

/// Half the length of a twilight, in minutes.
const TWILIGHT_HALF: u32 = 30;

/// Overworld sight radius in cells by day, in twilight, and at night.
pub const SIGHT_DAY: i32 = 9;
pub const SIGHT_TWILIGHT: i32 = 5;
pub const SIGHT_NIGHT: i32 = 2;

/// The least light on the overworld: a faint, cold light. Sight there does not depend on light,
/// so this only draws the land in sight at night apart from the darker remembered map. Local
/// levels get no such light: at night they are dark.
pub const OVERWORLD_NIGHT: Light = Light::new(48, 56, 90, 0);

/// The phase of the day at `time`.
pub fn phase(time: GameTime) -> DayPhase {
    let m = time.minute_of_day();
    let (dawn, dusk) = ((DAWN / MINUTE) as u32, (DUSK / MINUTE) as u32);
    let near = |at: u32| m.abs_diff(at) < TWILIGHT_HALF;
    if near(dawn) || near(dusk) {
        DayPhase::Twilight
    } else if (dawn..dusk).contains(&m) {
        DayPhase::Day
    } else {
        DayPhase::Night
    }
}

/// How far the player sees on the overworld at `time` (line of sight within this radius).
pub fn overworld_sight(time: GameTime) -> i32 {
    match phase(time) {
        DayPhase::Day => SIGHT_DAY,
        DayPhase::Twilight => SIGHT_TWILIGHT,
        DayPhase::Night => SIGHT_NIGHT,
    }
}

/// The light the overworld gets on top of the `sky`'s so that together they are never darker
/// than [`OVERWORLD_NIGHT`] (channel by channel): all of it at night, none by day.
pub fn overworld_night(sky: Light) -> Light {
    let [r, g, b] = std::array::from_fn(|i| OVERWORLD_NIGHT.rgb[i].saturating_sub(sky.rgb[i]));
    Light::new(r, g, b, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::light::Sun;

    fn at(h: u32, m: u32) -> GameTime {
        GameTime::from_calendar(1, 3, 7, h, m).expect("valid time")
    }

    #[test]
    fn sight_follows_the_phases_of_the_day() {
        assert_eq!(overworld_sight(at(12, 0)), SIGHT_DAY);
        assert_eq!(overworld_sight(at(6, 30)), SIGHT_DAY);
        assert_eq!(overworld_sight(at(17, 29)), SIGHT_DAY);
        assert_eq!(overworld_sight(at(6, 0)), SIGHT_TWILIGHT);
        assert_eq!(overworld_sight(at(5, 31)), SIGHT_TWILIGHT);
        assert_eq!(overworld_sight(at(18, 29)), SIGHT_TWILIGHT);
        assert_eq!(overworld_sight(at(5, 30)), SIGHT_NIGHT);
        assert_eq!(overworld_sight(at(18, 30)), SIGHT_NIGHT);
        assert_eq!(overworld_sight(at(0, 0)), SIGHT_NIGHT);
        assert_eq!(overworld_sight(at(22, 0)), SIGHT_NIGHT);
    }

    #[test]
    fn the_night_light_tops_the_sky_up_and_vanishes_by_day() {
        assert_eq!(overworld_night(Light::DARK), OVERWORLD_NIGHT);
        for (h, m) in [
            (0, 0),
            (5, 50),
            (6, 10),
            (12, 0),
            (17, 50),
            (18, 10),
            (22, 0),
        ] {
            let sky = Sun::at(at(h, m)).sky;
            let total = sky.saturating_add(overworld_night(sky));
            assert_eq!(total, sky.max(OVERWORLD_NIGHT), "{h}:{m:02}");
        }
        assert!(overworld_night(Sun::at(at(12, 0)).sky).is_dark());
    }
}
