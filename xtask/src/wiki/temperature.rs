//! Temperature (spec §5 "Temperature") and ice (spec §5 "Ice"): the months, the weather, what
//! freezes, what ice holds, and how fast it grows.

use super::actions::at_normal_pace;
use super::text::capital;
use std::fmt::Write as _;
use warlock_core::content::{ActionKind, Content, LevelKind};
use warlock_core::ice;
use warlock_core::object::format_weight;
use warlock_core::temperature::{self, Temperature};
use warlock_core::time::GameTime;
use warlock_core::weather::Weather;

/// A temperature in whole degrees, both ways: `-7 °C (19 °F)`.
fn degrees(t: Temperature) -> String {
    format!("{} °C ({} °F)", t.celsius(), t.fahrenheit())
}

/// Each month's mean temperature under the open sky, and how cold a clear night and how warm a
/// clear afternoon get in the middle of it (spec §5 "Temperature").
pub(super) fn temperatures(content: &Content) -> String {
    let climate = content.climate();
    let half_swing = i32::from(climate.swing()) * 5;
    let mut out = String::from(
        "| Month | Mean | A clear night | A clear afternoon |\n|---|---:|---:|---:|\n",
    );
    for month in 1..=12 {
        let mid = GameTime::from_calendar(1, month, 16, 0, 0).expect("a valid date");
        let mean = temperature::mean(mid, climate) as i32;
        let at = |offset: i32| degrees(Temperature::from_tenths(mean + offset));
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} |",
            content.calendar().month_name(mid),
            at(0),
            at(-half_swing),
            at(half_swing),
        );
    }
    out
}

/// What each weather does to the day's swing of temperature, and how much it cools the air.
pub(super) fn temperature_weather(content: &Content) -> String {
    let climate = content.climate();
    let mut out = String::from(
        "| Weather | Coldest night to warmest afternoon | Cooler by |\n|---|---:|---:|\n",
    );
    for w in Weather::ALL {
        let def = climate.def(w);
        let swing =
            Temperature::from_tenths(i32::from(climate.swing()) * i32::from(def.swing) / 10);
        let chill = match def.chill {
            0 => "-".to_string(),
            c => format!("{c} °C"),
        };
        let _ = writeln!(
            out,
            "| {} | {}.{} °C | {chill} |",
            w.name(),
            swing.tenths() / 10,
            swing.tenths() % 10
        );
    }
    out
}

/// What water freezes and how (spec §5 "Ice").
pub(super) fn freezing(content: &Content) -> String {
    let mut out = String::from(
        "| Water | Ice grows below | How fast | A fall through the ice |\n|---|---:|---:|---|\n",
    );
    for t in content.terrain_defs() {
        if let Some(f) = t.freeze {
            // A fixed number reads as a number, dice as written.
            let hurt = match (f.plunge.is_fixed(), f.plunge.min()) {
                (true, 1) => "1 hit point".to_string(),
                (true, n) => format!("{n} hit points"),
                (false, _) => format!("{} hit points", f.plunge),
            };
            let _ = writeln!(
                out,
                "| {} | {} | {} | {hurt} (never your last) |",
                capital(&t.name),
                degrees(Temperature::degrees(f.below.into())),
                if f.rate == 100 {
                    "full pace".to_string()
                } else {
                    format!("{}% of still water's pace", f.rate)
                },
            );
        }
    }
    out
}

/// What ice looks like by its thickness, and what it holds (spec §5 "Ice").
pub(super) fn ice_thickness(content: &Content) -> String {
    let step = content.action_costs().cost(ActionKind::Step);
    let tiers = content.ice_terrain();
    let mut out =
        String::from("| Looks like | Thickness | Holds | Step takes |\n|---|---|---|---:|\n");
    let cm = |mm: u16| {
        if mm.is_multiple_of(10) {
            format!("{} cm", mm / 10)
        } else {
            format!("{}.{} cm", mm / 10, mm % 10)
        }
    };
    for (i, &(mm, id)) in tiers.iter().enumerate() {
        let def = content.terrain(id);
        let holds = |mm: u16| format_weight(ice::holds(u32::from(mm) * 1000));
        let (thickness, weight) = match tiers.get(i + 1) {
            Some(&(next, _)) => (
                format!("{} to {}", cm(mm), cm(next)),
                format!("{} to {}", holds(mm), holds(next)),
            ),
            None => (
                format!("{} and more", cm(mm)),
                format!("{} and more", holds(mm)),
            ),
        };
        let _ = writeln!(
            out,
            "| {} | {thickness} | {weight} | {} |",
            capital(&def.name),
            at_normal_pace(def.step_price(step, LevelKind::Dungeon)),
        );
    }
    out
}

/// How long still water takes to freeze over to each thickness of ice at a few steady
/// temperatures (spec §5 "Ice").
pub(super) fn ice_growth(content: &Content) -> String {
    let Some(still) = content
        .terrain_defs()
        .iter()
        .find_map(|t| t.freeze.filter(|f| f.below == 0 && f.rate == 100))
    else {
        return String::new();
    };
    let tiers = content.ice_terrain();
    let mut out = String::from("| Steady at |");
    for &(_, id) in tiers {
        let _ = write!(out, " {} |", capital(&content.terrain(id).name));
    }
    out.push_str("\n|---|");
    out.push_str(&"---:|".repeat(tiers.len()));
    out.push('\n');
    let ticks_per_hour = 3600 / ice::TICK;
    for c in [-2, -5, -10, -20] {
        let t = Temperature::degrees(c);
        let _ = write!(out, "| {} |", degrees(t));
        for &(mm, _) in tiers {
            let mut um = 0;
            let mut ticks = 0u64;
            while um < u32::from(mm) * 1000 && ticks < 1000 * 144 {
                um = ice::tick(um, t, &still);
                ticks += 1;
            }
            let hours = ticks.div_ceil(ticks_per_hour);
            let took = if hours == 1 {
                "an hour".to_string()
            } else if hours < 48 {
                format!("{hours} hours")
            } else {
                format!("{} days", (hours + 12) / 24)
            };
            let _ = write!(out, " {took} |");
        }
        out.push('\n');
    }
    out
}
