//! Dev actions with no feature module of their own: the vision profile
//! ([`crate::action::Action::SetVision`]), the weather override
//! ([`crate::action::Action::SetWeather`]), and the hour skip
//! ([`crate::action::Action::SkipTime`]). `world/mod.rs` has only the dispatch lines.

use super::{Spent, World};
use crate::time::Duration;
use crate::weather::{self, Weather};

impl World {
    /// The dev action `weather:KIND` ([`crate::action::Action::SetWeather`]): `weather` for
    /// [`weather::OVERRIDE`] from now. Takes no time.
    pub(super) fn set_weather_dev(&mut self, weather: Weather) -> Spent {
        let over = weather::Override::new(weather, self.state.time);
        self.state.weather_override = Some(over);
        self.say(format!(
            "Weather: {} for {} (dev).",
            weather.name().to_lowercase(),
            weather::OVERRIDE.spoken()
        ));
        Spent::NoTime
    }

    /// The dev action `skip:DURATION` ([`crate::action::Action::SkipTime`]): the clock jumps
    /// ahead `duration`; nobody acts, but Appraise looks over what the player carries each hour.
    pub(super) fn skip_time_dev(&mut self, duration: Duration) -> Spent {
        self.say(format!("Time skips ahead {} (dev).", duration.spoken()));
        self.skip_stat_check(self.state.time + duration);
        // Appraise's hourly checks still come as the hours pass (spec §8 "Skills").
        self.appraise_hours(self.state.time, duration.as_seconds());
        Spent::Skip(duration)
    }

    pub(super) fn set_vision(&mut self, id: &str) -> Spent {
        let Some(profile) = self.content.vision(id) else {
            self.say(format!("There is no vision profile `{id}`."));
            return Spent::Nothing;
        };
        let name = profile.name.clone();
        self.state.vision = id.to_string();
        self.state.vision_dev = true;
        self.say(format!("Vision: {name}."));
        Spent::NoTime
    }
}
