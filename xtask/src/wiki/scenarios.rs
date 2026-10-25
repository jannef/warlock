//! The scenarios a new run can start: each campaign's start.

use super::order;
use super::text::table_text;
use std::fmt::Write as _;
use warlock_core::content::Content;

/// The scenarios a new run can start, as a release build's New run starts them: per campaign the
/// wiki describes (`order::campaigns`), its name and description and its start scenario, with
/// the moment it starts. The campaign's other scenarios (in its town, on a branch's first level)
/// are developer conveniences, listed by New run in developer builds only (spec §3b).
pub(super) fn scenarios(content: &Content) -> String {
    let mut out = String::new();
    for campaign in order::campaigns(content) {
        let list: Vec<_> = content.scenario(&campaign.start).into_iter().collect();
        if !out.is_empty() {
            out.push('\n');
        }
        let _ = writeln!(
            out,
            "**{}**: {}\n\n| Scenario | Starts at | What it is |\n|---|---|---|",
            campaign.name,
            table_text(&campaign.description)
        );
        table(content, &list, &mut out);
    }
    out
}

/// The rows of `list`.
fn table(content: &Content, list: &[&warlock_core::content::ScenarioDef], out: &mut String) {
    for s in list {
        let _ = writeln!(
            out,
            "| {} | {:02}:{:02}, {} | {} |",
            s.name,
            s.start.hour(),
            s.start.minute(),
            content.calendar().full_date(s.start),
            table_text(&s.description)
        );
    }
}
