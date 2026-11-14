//! What the ally talents do, in the wiki's words (`warlock_core::talent::AllyEffect`, #1495): one
//! arm a variant, beside the area's enum and sum in `warlock-core/src/talent/ally.rs`.

use crate::wiki::util::signed;
use warlock_core::content::Content;
use warlock_core::talent::AllyEffect;

/// What `effect` does, as the talents' tables say it.
pub(super) fn text(content: &Content, effect: &AllyEffect) -> String {
    let status = |id: &str| {
        (content.status_id(id)).map_or_else(|| id.to_string(), |s| content.status(s).name.clone())
    };
    match effect.clone() {
        AllyEffect::AllyMod {
            to_hit,
            protection,
            count,
            duration_percent,
            lasting,
            most,
        } => {
            let mut gets = Vec::new();
            if to_hit != 0 {
                gets.push(format!("to-hit {} on their attacks", signed(to_hit)));
            }
            if protection != 0 {
                gets.push(format!("Protection {}", signed(protection)));
            }
            let mut parts = Vec::new();
            if !gets.is_empty() {
                parts.push(format!("your allies get {}", gets.join(" and ")));
            }
            if count != 0 {
                parts.push(format!("every summon spell's call brings {count} more"));
            }
            if duration_percent != 0 {
                parts.push(format!("calls last {duration_percent}% longer"));
            }
            if lasting {
                parts.push("calls never lapse: they stay until slain".into());
            }
            if most != 0 {
                parts.push(format!("{most} more of each summon spell's calls at once"));
            }
            parts.join("; ")
        }
        AllyEffect::SummonBias { tag, weight } => match weight {
            0 => format!(
                "a summon spell never calls a creature tagged `{tag}` while another may come"
            ),
            n => format!("a summon spell calls a creature tagged `{tag}` {n} times as often"),
        },
        AllyEffect::Sic {
            status: id,
            blows,
            range,
        } => format!(
            "active (the time of a wait), at a creature you see within {range} cells that is \
             hostile to you: your companion hunts it, and is {} ({blows} degrees): half again as \
             fast, and each hit a critical with no confirmation, a degree each",
            status(&id)
        ),
    }
}
