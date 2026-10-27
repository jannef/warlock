//! Guilds: the ranks and what reaches each, and what a guild master's drawn quests can draw.

use super::order;
use super::text::capital;
use std::fmt::Write as _;
use warlock_core::content::{Content, Pick, RankBy, Rolled};

/// Every guild's ranks, lowest first: the title, the metal it is named by and its value (in
/// percent of steel, the order of the ranks), and what reaches it.
pub(super) fn guild_ranks(content: &Content) -> String {
    let mut guilds: Vec<_> = (content.guilds().iter())
        .filter(|g| order::described(content, g.campaign.as_deref()))
        .collect();
    guilds.sort_by_key(|g| g.name.to_lowercase());
    if guilds.is_empty() {
        return "No guild yet.\n".to_string();
    }
    let mut out = String::from(
        "| Guild | Rank | Named by | Metal's value (% of steel) | Reached by |\n\
         |---|---|---|---:|---|\n",
    );
    for g in guilds {
        for rank in &g.ranks {
            let metal = content.material(rank.material);
            let by = match rank.by {
                Some(RankBy::Given(q)) => format!("being given \"{}\"", content.quest(q).name),
                Some(RankBy::Done(q)) => format!("\"{}\" done", content.quest(q).name),
                None => "not yet (later quests)".to_string(),
            };
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {by} |",
                capital(g.bare_name()),
                rank.title,
                metal.name,
                metal.value,
            );
        }
    }
    out
}

/// What each drawn quest a guild's master gives can draw, every dungeon open: a creature type
/// and the dungeons it lives in, a sort of creature with its kinds, or an object; by quest in
/// content order, then by name.
pub(super) fn guild_draws(content: &Content) -> String {
    let mut out = String::from("| Quest | Can draw | Counts | Lives in |\n|---|---|---|---|\n");
    let mut any = false;
    for guild in content.guilds() {
        if !order::described(content, guild.campaign.as_deref()) {
            continue;
        }
        let quests = (content.quests().iter())
            .filter(|q| q.giver == guild.master && q.campaign == guild.campaign);
        for q in quests {
            let Some(rolled) = &q.roll else { continue };
            let branches: &[String] = match rolled {
                Rolled::Pick(Pick::Creatures(b)) => b,
                _ => &[],
            };
            let mut rows: Vec<(String, String, String)> = Vec::new();
            for c in q.candidates(content, &|_| true) {
                let row = match rolled {
                    Rolled::Pick(Pick::Creatures(_)) => {
                        let Some(id) = content.creature_id(&c.target) else {
                            continue;
                        };
                        let lives = lives_in(content, branches, |t| t == id);
                        (capital(&content.creature(id).name), "—".into(), lives)
                    }
                    Rolled::Pick(Pick::Tags(_)) => {
                        let tag = c.target.strip_prefix("tag:").unwrap_or(&c.target);
                        let campaign = q.campaign.as_deref();
                        let mut members: Vec<&str> = (content.tagged_in(tag, campaign))
                            .map(|m| content.creature(m).name.as_str())
                            .collect();
                        members.sort_by_cached_key(|n| n.to_lowercase());
                        let all: Vec<String> = all_branches(content, campaign);
                        let lives = lives_in(content, &all, |t| content.has_tag(t, tag));
                        let said = content.plural(&tag.replace('-', " "));
                        (capital(&said), members.join(", "), lives)
                    }
                    Rolled::Pick(Pick::Objects { .. }) => {
                        let Some(id) = content.object_id(&c.target) else {
                            continue;
                        };
                        (capital(&content.object(id).name), "—".into(), "—".into())
                    }
                    Rolled::Variants(variants) => {
                        let v = &variants[c.variant.unwrap_or(0)];
                        (capital(&v.target), "—".into(), "—".into())
                    }
                };
                rows.push(row);
            }
            rows.sort_by_cached_key(|r| r.0.to_lowercase());
            for (target, counts, lives) in rows {
                any = true;
                let _ = writeln!(out, "| {} | {target} | {counts} | {lives} |", q.name);
            }
        }
    }
    if !any {
        return "No guild gives a drawn quest yet.\n".to_string();
    }
    out
}

/// The dungeon branches of campaign `campaign` (and the shared ones), by id.
fn all_branches(content: &Content, campaign: Option<&str>) -> Vec<String> {
    content
        .dungeons()
        .filter(|d| d.campaign.is_none() || d.campaign.as_deref() == campaign)
        .map(|d| d.id.clone())
        .collect()
}

/// The names of the branches among `branches` whose population holds a type `holds` accepts,
/// joined; "—" when none.
fn lives_in(
    content: &Content,
    branches: &[String],
    holds: impl Fn(warlock_core::content::CreatureId) -> bool,
) -> String {
    let names: Vec<&str> = (branches.iter())
        .filter_map(|b| content.dungeon(b))
        .filter(|d| (d.spawning.as_ref()).is_some_and(|s| s.population.iter().any(|&c| holds(c))))
        .map(|d| d.name.as_str())
        .collect();
    if names.is_empty() {
        "—".into()
    } else {
        names.join(", ")
    }
}
