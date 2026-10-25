//! Dialogue and quests: who talks and who gives quests.

use super::text::{capital, yes};
use std::fmt::Write as _;
use warlock_core::content::{Content, Speaker};

/// Who talks (`data/dialogue.toml` and the campaigns' `data/dialogue/`, those the wiki
/// describes, by name): each speaker, whether they greet you, how many one-liners
/// they take turns with, what you may ask them about, and how they mutter. The lines themselves
/// are left to the game.
pub(super) fn dialogue(content: &Content) -> String {
    let mut out = String::from(
        "| Who | Greets you | One-liners | Topics | Mutters |\n|---|:---:|---:|---|---|\n",
    );
    let mut rows: Vec<_> = (content.dialogues().iter())
        .filter(|d| super::order::described(content, d.campaign.as_deref()))
        .map(|d| {
            let who = match &d.speaker {
                Speaker::Creature(c) => capital(&content.creature(*c).name),
                Speaker::Npc(npc) => format!("`{npc}` (a unique character)"),
            };
            (who, d)
        })
        .collect();
    rows.sort_by_cached_key(|(who, _)| who.to_lowercase());
    for (who, d) in rows {
        let topics: Vec<&str> = d.topics.iter().map(|t| t.name.as_str()).collect();
        let topics = if topics.is_empty() {
            "—".to_string()
        } else {
            topics.join(", ")
        };
        let mutters = d.mutter.as_ref().map_or_else(
            || "—".to_string(),
            |m| {
                format!(
                    "{} ({} % of its actions, when you are near)",
                    m.verb, m.chance
                )
            },
        );
        let _ = writeln!(
            out,
            "| {who} | {} | {} | {topics} | {mutters} |",
            yes(d.greet.is_some()),
            d.lines.len()
        );
    }
    out
}

/// Who gives quests (`data/quests.toml`): each quest's name, its giver, its kind, and what must
/// be done first. What it asks exactly and what it pays are left to the game.
pub(super) fn quests(content: &Content) -> String {
    let mut out = String::from(
        "| Quest | Given by | Kind | Offered after | Repeatable |\n|---|---|---|---|:---:|\n",
    );
    for q in content.quests() {
        let after = q
            .requires
            .map_or_else(|| "—".to_string(), |r| content.quest(r).name.clone());
        let _ = writeln!(
            out,
            "| {} | {} | {} | {after} | {} |",
            q.name,
            capital(&content.speaker_name(q.giver)),
            q.kind().id(),
            yes(q.repeatable)
        );
    }
    out
}
