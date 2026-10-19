//! Sight on the overworld (spec §5): by daylight, twilight, moonlight, and weather.

use std::fmt::Write as _;
use warlock_core::content::Content;
use warlock_core::daylight::{
    MOONLIGHT_PER_CELL, SIGHT_DAY, SIGHT_MOONLIT, SIGHT_NIGHT, SIGHT_TWILIGHT, overworld_sight,
};
use warlock_core::light::{Light, Moon, Sky};
use warlock_core::perception::VIEW_RADIUS;
use warlock_core::time::GameTime;
use warlock_core::weather::{Clouds, Conditions, Weather};

/// A share of full light (255) in whole percent, rounded.
pub(super) fn percent_of_full(luma: u16) -> u32 {
    (u32::from(luma) * 100 + 127) / 255
}

/// The light of the brightest moon when full and high, if there are moons.
fn brightest_moon(content: &Content) -> Option<Light> {
    content
        .moons()
        .iter()
        .map(Moon::brightest)
        .max_by_key(|l| l.luma())
}

pub(super) fn overworld_sight_table(content: &Content) -> String {
    let mut out = String::from("| When | Sight on the overworld |\n|---|---:|\n");
    let _ = writeln!(out, "| Day (06:30 to 17:30) | {SIGHT_DAY} cells |");
    let _ = writeln!(
        out,
        "| Twilight (half an hour either side of 06:00 and 18:00) | {SIGHT_TWILIGHT} cells |"
    );
    let _ = writeln!(out, "| Night, no moonlight | {SIGHT_NIGHT} cells |");
    let midnight = GameTime::from_calendar(1, 1, 1, 0, 0).expect("a valid date");
    if let Some(full) = brightest_moon(content) {
        let half = Light::new(full.rgb[0] / 2, full.rgb[1] / 2, full.rgb[2] / 2, 0);
        for (name, light) in [("half", half), ("full", full)] {
            let sight = overworld_sight(midnight, light, 0);
            let _ = writeln!(
                out,
                "| Night, a {name} moon high in the sky | {sight} cells |"
            );
        }
    }
    // Clouds dim the moon, so they shorten moonlit sight even when they take no cells away.
    if !content.moons().is_empty() {
        let full_moon = GameTime::from_calendar(1, 1, 16, 0, 0).expect("a valid date");
        for weather in [Weather::Overcast, Weather::Rain, Weather::HeavyRain] {
            let def = content.climate().def(weather);
            let conditions = Conditions {
                weather,
                clouds: Clouds::of(def),
            };
            let moonlight = Sky::at(full_moon, content.moons(), conditions).moonlight();
            let sight = overworld_sight(full_moon, moonlight, def.sight);
            let under = match weather {
                Weather::Overcast => "overcast".to_string(),
                w => format!("in {}", w.name().to_lowercase()),
            };
            let _ = writeln!(
                out,
                "| Night, a full moon high in the sky, {under} | {sight} cells |"
            );
        }
    }
    let noon = GameTime::from_calendar(1, 1, 1, 12, 0).expect("a valid date");
    for weather in [Weather::Rain, Weather::HeavyRain] {
        let lost = content.climate().def(weather).sight;
        if lost > 0 {
            let sight = overworld_sight(noon, Light::DARK, lost);
            let name = weather.name().to_lowercase();
            let _ = writeln!(out, "| Day, in {name} | {sight} cells |");
        }
    }
    let _ = writeln!(
        out,
        "\nAt night moonlight (all moons together) adds a cell of sight for every {}% of full light \
         it gives, but by moonlight you never see farther than {SIGHT_MOONLIT} cells. Clouds dim \
         the moon, and rain takes cells away at any hour (see [the weather](weather.md)), but you \
         always see at least {SIGHT_NIGHT}. The farthest you can ever see is {VIEW_RADIUS} cells.",
        percent_of_full(MOONLIGHT_PER_CELL)
    );
    out
}
