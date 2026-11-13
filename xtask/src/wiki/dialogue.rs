//! Dialogue and quests: who talks and who gives quests.

use super::text::{capital, yes};
use std::fmt::Write as _;
use warlock_core::content::{Content, Dialogue, Reward, Speaker};
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
            // Those a done quest adds count too.
            d.lines.len() + d.after.iter().map(|a| a.lines.len()).sum::<usize>()
        );
    }
    out
}

/// The rumoured ways (a topic's `route`, those the wiki describes): who tells each, about
/// what, its waypoints on the overworld, and the region where it ends; sorted by who tells it,
/// then by the topic's name.
pub(super) fn rumoured_ways(content: &Content) -> String {
    let mut rows: Vec<(String, String, String, String)> = Vec::new();
    for d in (content.dialogues().iter())
        .filter(|d| super::order::described(content, d.campaign.as_deref()))
    {
        let who = match (&d.speaker, &d.name) {
            (Speaker::Creature(c), _) => capital(&content.creature(*c).name),
            (Speaker::Npc(_), Some(name)) => name.clone(),
            (Speaker::Npc(npc), None) => format!("`{npc}`"),
        };
        let overworld = d.campaign.as_deref().and_then(|c| content.overworld_of(c));
        for t in d.topics.iter().filter(|t| !t.route.is_empty()) {
            let way: Vec<String> = (t.route.iter())
                .map(|[x, y]| format!("({x}, {y})"))
                .collect();
            let end = t.route_cells().last().copied();
            let region = (overworld.zip(end))
                .and_then(|(o, p)| o.region_at(p))
                .unwrap_or("—");
            rows.push((
                who.clone(),
                t.name.clone(),
                way.join(" → "),
                region.to_string(),
            ));
        }
    }
    if rows.is_empty() {
        return "No one tells the way anywhere yet.\n".to_string();
    }
    rows.sort();
    let mut out =
        String::from("| Who | Topic | The way (waypoints) | Ends in |\n|---|---|---|---|\n");
    for (who, topic, way, region) in rows {
        let _ = writeln!(out, "| {who} | {topic} | {way} | {region} |");
    }
    out
}

/// Whom dialogue `d` speaks to at all, by alignment band: "Kind or better", or "anyone".
fn speaks_to_text(d: &Dialogue) -> String {
    d.speaks_to.map_or_else(
        || "anyone".to_string(),
        |r| match (r.min, r.max) {
            (Some(min), Some(max)) => format!("{} to {}", min.name(), max.name()),
            (Some(min), None) => format!("{} or better", min.name()),
            (None, Some(max)) => format!("{} or worse", max.name()),
            (None, None) => "anyone".to_string(),
        },
    )
}

/// Each campaign's callings (spec §8 "Prestige classes"), stated plain: who the calling is
/// open to, who grants it when spoken to (the speakers with a topic that `calls` it, in the
/// calling's campaign), whom the speaker speaks to, and the encounter tables
/// that bring the speaker (entries whose `when.calling` is the class). By campaign, then class.
pub(super) fn callings(content: &Content) -> String {
    let mut rows: Vec<(String, String, String)> = Vec::new();
    for c in content.callings() {
        let Some(campaign) = c.campaign.as_deref() else {
            continue;
        };
        if !super::order::described(content, Some(campaign)) {
            continue;
        }
        let place = (content.campaigns().iter())
            .find(|k| k.id == campaign)
            .map_or(campaign, |k| k.name.as_str());
        let who = calling_text(content, Some(campaign), &c.class);
        let mut ways: Vec<String> = Vec::new();
        for d in (content.dialogues().iter()).filter(|d| d.campaign.as_deref() == Some(campaign)) {
            let speaker = match (&d.speaker, &d.name) {
                (Speaker::Creature(id), _) => {
                    let def = content.creature(*id);
                    format!(
                        "the [{}](creatures.md#creature-{})",
                        def.name,
                        def.id.replace('_', "-")
                    )
                }
                (Speaker::Npc(_), Some(name)) => name.clone(),
                (Speaker::Npc(npc), None) => format!("`{npc}`"),
            };
            let band = match d.speaks_to {
                Some(_) => format!(" (who speaks only to the {})", speaks_to_text(d)),
                None => String::new(),
            };
            if (d.topics.iter()).any(|t| t.calls.as_deref() == Some(c.class.as_str())) {
                ways.push(format!("{speaker}{band} grants it when spoken to"));
            }
        }
        let tables: Vec<String> = (content.encounter_tables(campaign))
            .filter(|t| {
                (t.entries.iter()).any(|e| e.when.calling.as_deref() == Some(c.class.as_str()))
            })
            .map(|t| {
                format!(
                    "[{}](encounters.md#encounter-table-{})",
                    t.id,
                    t.id.replace('_', "-")
                )
            })
            .collect();
        let met = match tables.is_empty() {
            true => String::new(),
            false => format!(", met on the way ({})", tables.join(", ")),
        };
        let how = match ways.is_empty() {
            true => "nothing there calls you yet".to_string(),
            false => ways.join(", or "),
        };
        let line = format!(
            "- **{place}**, for {who}: {how}{met}. You are told: \"{}\"",
            c.message
        );
        rows.push((place.to_string(), c.class.clone(), line));
    }
    if rows.is_empty() {
        return "No world calls anyone into a prestige class yet.\n".into();
    }
    rows.sort();
    let mut out = String::new();
    for (_, _, line) in rows {
        out.push_str(&line);
        out.push('\n');
    }
    out
}

/// Who a calling (spec §8 "Prestige classes") is open to, and into what, by the prestige class
/// `class` it opens, as the wiki says it: "a [bard](classes.md#bard) of level 30 or more, not
/// yet called, to be a Hunter". The calling of `campaign` (or of none) if there is one, else the
/// first of that class.
pub(super) fn calling_text(content: &Content, campaign: Option<&str>, class: &str) -> String {
    let callings = content.callings();
    let calling = (callings.iter())
        .find(|c| c.class == class && (c.campaign.as_deref() == campaign || c.campaign.is_none()))
        .or_else(|| callings.iter().find(|c| c.class == class));
    let Some(calling) = calling else {
        return format!("into `{class}`");
    };
    let from: Vec<String> = (calling.from.iter())
        .map(|f| {
            let name = content.class(f).map_or(f.as_str(), |c| c.name.as_str());
            format!("[{}](classes.md#{f})", name.to_lowercase())
        })
        .collect();
    let name = content.prestige(class).map_or(class, |p| p.name.as_str());
    format!(
        "a {} of level {} or more, not yet called, to be a {name}",
        from.join(" or "),
        calling.level
    )
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

/// A quest's reward in words: "20 gold, 30 XP, jade amulet", "Heal skill (1d4 untrained, 100
/// marks trained)".
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
    for taught in &reward.skills {
        let name = &content.skill(taught.skill).name;
        let mut how = Vec::new();
        if let Some(start) = taught.start {
            how.push(format!("{start} untrained"));
        }
        if taught.marks > 0 {
            how.push(format!("{} marks trained", taught.marks));
        }
        parts.push(format!("{name} skill ({})", how.join(", ")));
    }
    if parts.is_empty() {
        "—".to_string()
    } else {
        parts.join(", ")
    }
}
