//! The scenarios a new run can start, by campaign.

use super::order;
use super::text::table_text;
use std::fmt::Write as _;
use warlock_core::content::Content;

/// The scenarios a new run can start, by campaign (those the wiki describes, `order::campaigns`)
/// as the New run screen lists them: each campaign's name and description, then its scenarios,
/// its start first and the rest by id, with the moment each starts.
pub(super) fn scenarios(content: &Content) -> String {
    let mut out = String::new();
    for campaign in order::campaigns(content) {
        let mut list: Vec<_> = (content.scenarios())
            .filter(|s| s.campaign.as_deref() == Some(campaign.id.as_str()))
            .collect();
        list.sort_by_key(|s| s.id != campaign.start);
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
