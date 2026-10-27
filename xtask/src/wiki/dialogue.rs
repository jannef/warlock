//! Dialogue and quests: who talks and who gives quests.

use super::text::{capital, yes};
use std::fmt::Write as _;
use warlock_core::content::{Content, Reward, Speaker};
use warlock_core::world::quest_asks;

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
            let who = match (&d.speaker, &d.name) {
                (Speaker::Creature(c), _) => capital(&content.creature(*c).name),
                (Speaker::Npc(_), Some(name)) => name.clone(),
                (Speaker::Npc(npc), None) => format!("`{npc}` (a unique character)"),
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

/// Who gives quests (`data/quests.toml` and the campaigns' `data/quests/`, those the wiki
/// describes, in load order: each chain in its order): each quest's name, its giver, its kind,
/// what it asks, what it pays, what must be done first, the character level it is given from,
/// and the hidden location it reveals.
pub(super) fn quests(content: &Content) -> String {
    let mut out = String::from(
        "| Quest | Given by | Kind | Goal | Reward | Offered after | From level | Reveals | Repeatable |\n\
         |---|---|---|---|---|---|:---:|---|:---:|\n",
    );
    let quests = (content.quests().iter())
        .filter(|q| super::order::described(content, q.campaign.as_deref()));
    for q in quests {
        let after: Vec<&str> = (q.requires.iter())
            .map(|&r| content.quest(r).name.as_str())
            .collect();
        let after = if after.is_empty() {
            "—".to_string()
        } else {
            after.join(", ")
        };
        let level = q
            .min_level
            .as_ref()
            .map_or("—".to_string(), |(l, _)| l.to_string());
        let reveals = q.reveals.as_ref().map_or_else(
            || "—".to_string(),
            |r| {
                let campaign = q.campaign.as_deref().unwrap_or_default();
                let name = (content.hidden_location(campaign, &r.location))
                    .map_or(r.location.as_str(), |(_, h)| h.name.as_str());
                format!("{name} (when {})", r.when.id())
            },
        );
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {after} | {level} | {reveals} | {} |",
            q.name,
            capital(&content.speaker_name(q.giver)),
            q.kind().id(),
            quest_asks(content, q).trim_end_matches('.'),
            reward(content, &q.reward),
            yes(q.repeatable)
        );
    }
    out
}

/// A quest's reward in words: "20 gold, 30 XP, jade amulet".
fn reward(content: &Content, reward: &Reward) -> String {
    let mut parts = Vec::new();
    if reward.gold > 0 {
        parts.push(format!("{} gold", reward.gold));
    }
    if reward.xp > 0 {
        parts.push(format!("{} XP", reward.xp));
    }
    for stack in &reward.items {
        parts.push(content.object(stack.object).describe(stack.count));
    }
    if parts.is_empty() {
        "—".to_string()
    } else {
        parts.join(", ")
    }
}
