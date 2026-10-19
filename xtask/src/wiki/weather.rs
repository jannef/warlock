//! Weather (spec §5 "Weather"): the chances by month and what each weather does.

use std::fmt::Write as _;
use warlock_core::content::Content;
use warlock_core::light::{Sky, Sun};
use warlock_core::time::GameTime;
use warlock_core::weather::{Clouds, Conditions, Weather};

/// The chance of each weather in each month.
pub(super) fn weather_chances(content: &Content) -> String {
    let mut out = String::from("| Month |");
    for w in Weather::ALL {
        let _ = write!(out, " {} |", w.name());
    }
    out.push_str("\n|---|---:|---:|---:|---:|\n");
    for month in 1..=12 {
        let time = GameTime::from_calendar(1, month, 1, 0, 0).expect("a valid date");
        let _ = write!(out, "| {} |", content.calendar().month_name(time));
        for chance in content.climate().chances(month as usize - 1) {
            let _ = write!(out, " {chance}% |");
        }
        out.push('\n');
    }
    out
}

/// A share of `of` in whole percent, rounded.
fn share(part: u16, of: u16) -> u32 {
    (u32::from(part) * 100 + u32::from(of) / 2) / u32::from(of.max(1))
}

/// What each weather does to the light and to overworld sight.
pub(super) fn weather_effects(content: &Content) -> String {
    let mut out = String::from(
        "| Weather | Light at noon, in the open | In the shade | A full moon's light | \
         Overworld sight lost |\n|---|---:|---:|---:|---:|\n",
    );
    let noon = GameTime::from_calendar(1, 1, 1, 12, 0).expect("a valid date");
    let full_moon = GameTime::from_calendar(1, 1, 16, 0, 0).expect("a valid date");
    let clear = Sun::at(noon);
    let open = |sun: &Sun| sun.light.saturating_add(sun.sky).luma();
    let moonlight = |w: Weather| {
        let conditions = Conditions {
            weather: w,
            clouds: Clouds::of(content.climate().def(w)),
        };
        Sky::at(full_moon, content.moons(), conditions)
            .moonlight()
            .luma()
    };
    for w in Weather::ALL {
        let def = content.climate().def(w);
        let clouds = Clouds::of(def);
        let sun = Sun {
            light: clouds.direct(clear.light),
            sky: clouds.sky(clear.sky, clear.light),
            ..clear
        };
        let moon = if content.moons().is_empty() {
            "no moon".to_string()
        } else {
            format!("{}%", share(moonlight(w), moonlight(Weather::Clear)))
        };
        let sight = match def.sight {
            0 => "none".to_string(),
            1 => "1 cell".to_string(),
            n => format!("{n} cells"),
        };
        let _ = writeln!(
            out,
            "| {} | {}% | {}% | {moon} | {sight} |",
            w.name(),
            share(open(&sun), open(&clear)),
            share(sun.sky.luma(), clear.sky.luma()),
        );
    }
    let _ = writeln!(
        out,
        "\nPercentages are of the light of a clear sky at the same moment. The shade is where the \
         sun does not reach but the sky does; the brighter it is next to the open ground, the \
         fainter the shadows. The last column is what the weather itself takes from your sight \
         on the overworld; at night its clouds also dim the moon, and with it how far you see \
         by moonlight (see [the overworld](overworld.md))."
    );
    out
}
