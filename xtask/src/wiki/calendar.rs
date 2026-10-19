//! The calendar: the months.

use std::fmt::Write as _;
use warlock_core::content::Content;
use warlock_core::time::GameTime;

pub(super) fn months(content: &Content) -> String {
    let mut out = String::from("| Month | Name |\n|---:|---|\n");
    for month in 1..=12 {
        let time = GameTime::from_calendar(1, month, 1, 0, 0).expect("a valid date");
        let _ = writeln!(out, "| {month} | {} |", content.calendar().month_name(time));
    }
    out
}
