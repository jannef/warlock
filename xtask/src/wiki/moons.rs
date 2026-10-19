//! The moons (spec §5 "Moonlight").

use super::sight::percent_of_full;
use std::fmt::Write as _;
use warlock_core::content::Content;
use warlock_core::light::Moon;
use warlock_core::time::{DAY, GameTime};

pub(super) fn moons(content: &Content) -> String {
    if content.moons().is_empty() {
        return "There are no moons: nights are dark.\n".into();
    }
    let mut out = String::from(
        "| Moon | From new moon to new moon | Size | Light when full and high | First new moon |\n\
         |---|---:|---|---:|---|\n",
    );
    for moon in content.moons() {
        let size = if moon.size == 100 {
            "standard".to_string()
        } else {
            format!("{}% of the standard", moon.size)
        };
        let first = (moon.period_days - moon.phase_offset_days) % moon.period_days;
        let first = GameTime::from_seconds(u64::from(first) * DAY);
        let _ = writeln!(
            out,
            "| {} | {} days | {size} | about {}% of full light | {} |",
            moon.name,
            moon.period_days,
            percent_of_full(Moon::brightest(moon).luma()),
            content.calendar().full_date(first)
        );
    }
    out
}
