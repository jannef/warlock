//! Daylight on the overworld (spec §5): how far the traveller sees by time of day, moonlight, and
//! rain, and the faint night light the overworld is drawn by. The sun, the moons, and the sky
//! themselves are [`crate::light::Sky`], which lights the overworld like any level under the open
//! sky. Integer math only.

use crate::content::Climate;
use crate::light::{Light, Sky};
use crate::time::{DAWN, DUSK, GameTime, MINUTE};

/// Where the day is, for the overworld's sight radius.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DayPhase {
    /// From half past dawn (06:30) to half an hour before dusk (17:30).
    Day,
    /// Half an hour either side of dawn (06:00) and dusk (18:00).
    Twilight,
    /// The rest, when only the moons light the land.
    Night,
}

/// Half the length of a twilight, in minutes.
const TWILIGHT_HALF: u32 = 30;

/// Overworld sight radius in cells by day, in twilight, and at night without a moon.
pub const SIGHT_DAY: i32 = 9;
pub const SIGHT_TWILIGHT: i32 = 5;
pub const SIGHT_NIGHT: i32 = 2;
/// Moonlight (the luma of the moons' direct and sky light) that adds a cell of overworld sight:
/// a full moon high in the sky adds two, a half moon one.
pub const MOONLIGHT_PER_CELL: u16 = 20;
/// The farthest moonlight alone lets one see on the overworld, however many moons shine: less
/// than twilight.
pub const SIGHT_MOONLIT: i32 = SIGHT_TWILIGHT - 1;

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

/// How far the player sees on the overworld at `time` (line of sight within this radius) with
/// `moonlight` (the moons' light on open ground through the clouds,
/// [`crate::light::Sky::moonlight`]) and the weather taking `lost` cells away
/// ([`crate::content::WeatherDef::sight`]): by day and in twilight as far as the daylight allows,
/// at night farther the brighter the moonlight, up to [`SIGHT_MOONLIT`]; then rain takes its
/// cells, but never below [`SIGHT_NIGHT`].
pub fn overworld_sight(time: GameTime, moonlight: Light, lost: u8) -> i32 {
    let by_day = match phase(time) {
        DayPhase::Day => SIGHT_DAY,
        DayPhase::Twilight => SIGHT_TWILIGHT,
        DayPhase::Night => SIGHT_NIGHT,
    };
    let by_moon = SIGHT_NIGHT + i32::from(moonlight.luma() / MOONLIGHT_PER_CELL);
    let clear = by_day.max(by_moon.min(SIGHT_MOONLIT));
    (clear - i32::from(lost)).max(SIGHT_NIGHT)
}

/// How far one sees on the overworld at `time` under `sky` ([`crate::light::Sky`]): by daylight,
/// the moonlight through its clouds, and less in its weather's rain
/// ([`crate::content::WeatherDef::sight`]); see [`overworld_sight`].
pub fn sight_under(time: GameTime, sky: &Sky, climate: &Climate) -> i32 {
    let lost = climate.def(sky.conditions.weather).sight;
    overworld_sight(time, sky.moonlight(), lost)
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
        let dark = |t| overworld_sight(t, Light::DARK, 0);
        assert_eq!(dark(at(12, 0)), SIGHT_DAY);
        assert_eq!(dark(at(6, 30)), SIGHT_DAY);
        assert_eq!(dark(at(17, 29)), SIGHT_DAY);
        assert_eq!(dark(at(6, 0)), SIGHT_TWILIGHT);
        assert_eq!(dark(at(5, 31)), SIGHT_TWILIGHT);
        assert_eq!(dark(at(18, 29)), SIGHT_TWILIGHT);
        assert_eq!(dark(at(5, 30)), SIGHT_NIGHT);
        assert_eq!(dark(at(18, 30)), SIGHT_NIGHT);
        assert_eq!(dark(at(0, 0)), SIGHT_NIGHT);
        assert_eq!(dark(at(22, 0)), SIGHT_NIGHT);
    }

    #[test]
    fn moonlight_lets_one_see_farther_at_night_but_never_as_far_as_twilight() {
        let moon = |luma: u16| Light::new(luma, luma, luma, 0);
        assert_eq!(
            overworld_sight(at(0, 0), moon(MOONLIGHT_PER_CELL - 1), 0),
            SIGHT_NIGHT
        );
        assert_eq!(
            overworld_sight(at(0, 0), moon(MOONLIGHT_PER_CELL), 0),
            SIGHT_NIGHT + 1
        );
        assert_eq!(overworld_sight(at(0, 0), moon(45), 0), SIGHT_NIGHT + 2);
        assert_eq!(overworld_sight(at(0, 0), moon(500), 0), SIGHT_MOONLIT);
        const { assert!(SIGHT_MOONLIT < SIGHT_TWILIGHT) };
        assert_eq!(overworld_sight(at(6, 0), moon(500), 0), SIGHT_TWILIGHT);
        assert_eq!(overworld_sight(at(12, 0), moon(500), 0), SIGHT_DAY);
    }

    #[test]
    fn rain_takes_sight_away_but_never_below_the_darkest_night() {
        assert_eq!(overworld_sight(at(12, 0), Light::DARK, 2), SIGHT_DAY - 2);
        assert_eq!(overworld_sight(at(12, 0), Light::DARK, 4), SIGHT_DAY - 4);
        assert_eq!(
            overworld_sight(at(6, 0), Light::DARK, 2),
            SIGHT_TWILIGHT - 2
        );
        assert_eq!(overworld_sight(at(6, 0), Light::DARK, 4), SIGHT_NIGHT);
        assert_eq!(overworld_sight(at(0, 0), Light::DARK, 4), SIGHT_NIGHT);
        let moon = Light::new(200, 200, 200, 0);
        assert_eq!(overworld_sight(at(0, 0), moon, 1), SIGHT_MOONLIT - 1);
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
