//! The scenarios a new run can start.

use super::text::table_text;
use std::fmt::Write as _;
use warlock_core::content::Content;

/// The scenarios a new run can start, as the New run screen lists them (the showcase first), with
/// the moment each starts and whether the character's starting kit comes along.
pub(super) fn scenarios(content: &Content) -> String {
    let mut list: Vec<_> = content.scenarios().collect();
    list.sort_by_key(|s| s.id != "showcase");
    let mut out =
        String::from("| Scenario | Starts at | Your kit | What it is |\n|---|---|---|---|\n");
    for s in list {
        let _ = writeln!(
            out,
            "| {} | {:02}:{:02}, {} | {} | {} |",
            s.name,
            s.start.hour(),
            s.start.minute(),
            content.calendar().full_date(s.start),
            if s.kit { "yes" } else { "left behind" },
            table_text(&s.description)
        );
    }
    out
}
