//! Alternative resources (spec §8 "Resources"): each resource's maximum, start, how it changes
//! over time, what builds it, and whether it ends at zero.

use std::fmt::Write as _;
use warlock_core::content::{Build, Content, OverTime, ResourceDef};

/// Every resource, by name.
pub(super) fn resources(content: &Content) -> String {
    let mut list: Vec<&ResourceDef> = content.resources().iter().collect();
    list.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out = String::from(
        "| Resource | Most | Starts at | Over time | Built by | At zero |\n\
         |---|:---:|:---:|---|---|---|\n",
    );
    for def in list {
        let resting = if def.resting {
            ""
        } else {
            ", not while resting"
        };
        let over = match def.over_time {
            Some(OverTime::Regenerates { every }) => {
                format!("a point back every {}{resting}", seconds(every))
            }
            Some(OverTime::Decays { every }) => {
                format!("a point lost every {}{resting}", seconds(every))
            }
            None => "-".to_string(),
        };
        let built: Vec<String> = (Build::ALL.iter())
            .filter(|&&on| def.built_by(on) > 0)
            .map(|&on| format!("+{} for {}", def.built_by(on), on.words()))
            .collect();
        let built = if built.is_empty() {
            "-".to_string()
        } else {
            built.join(", ")
        };
        let brings = def.status.map_or_else(String::new, |s| {
            format!(", and {} with it", content.status(s).name)
        });
        let zero = if def.ends_at_zero {
            format!("ends, and its bar goes{brings}")
        } else {
            "stays, empty".to_string()
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {over} | {built} | {zero} |",
            def.name, def.max, def.start
        );
    }
    out
}

/// `6 seconds` (a base turn), `2 minutes`, `1 minute and 30 seconds`.
fn seconds(every: u32) -> String {
    let (m, s) = (every / 60, every % 60);
    let unit = |n: u32, one: &str| match n {
        1 => format!("1 {one}"),
        n => format!("{n} {one}s"),
    };
    match (m, s) {
        (0, s) => unit(s, "second"),
        (m, 0) => unit(m, "minute"),
        (m, s) => format!("{} and {}", unit(m, "minute"), unit(s, "second")),
    }
}
