//! Guilds (spec §8 "Quests": "Guilds"; `data/guilds/` and each campaign's `data/guilds/`, one
//! file per guild): a guild's name, its master (who names the player to a rank), and its ranks,
//! titles named by a material, each reached as a quest is given or done. A rank is a title only:
//! the player's is derived from their quest records ([`GuildDef::rank`]), never saved, and no
//! rule reads it.

use super::{Content, ContentError, DialogueId, MaterialId, QuestId};
use crate::color::Rgb;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The longest rank title, in characters.
pub const MAX_TITLE: usize = 16;
/// The most ranks a guild has.
pub const MAX_RANKS: usize = 16;

/// What reaches a rank.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RankBy {
    /// The quest being given (joining the guild).
    Given(QuestId),
    /// The quest being done.
    Done(QuestId),
}

/// One rank of a guild.
#[derive(Clone, Debug)]
pub struct RankDef {
    /// Its title ("Pewter").
    pub title: String,
    /// The material it is named for; the title shows in its colour.
    pub material: MaterialId,
    /// What reaches it; `None`: nothing yet (a rank kept for later quests).
    pub by: Option<RankBy>,
}

/// A guild as data.
#[derive(Clone, Debug)]
pub struct GuildDef {
    pub id: String,
    /// Its name in a sentence, with its article if it takes one ("the Weyfarers' Gilde").
    pub name: String,
    /// Who names the player to a rank.
    pub master: DialogueId,
    /// Its ranks, lowest first.
    pub ranks: Vec<RankDef>,
    /// The campaign whose table it is in (`None`: shared, `data/guilds/`).
    pub campaign: Option<String>,
}

impl GuildDef {
    /// Its name without a leading article ("Weyfarers' Gilde"), for labels.
    pub fn bare_name(&self) -> &str {
        self.name.strip_prefix("the ").unwrap_or(&self.name)
    }

    /// The highest rank the player holds, by index, given whether each quest was `given` (on
    /// the log, active or done) and is `done` (completed at least once); `None` before joining.
    pub fn rank(
        &self,
        given: impl Fn(QuestId) -> bool,
        done: impl Fn(QuestId) -> bool,
    ) -> Option<usize> {
        (self.ranks.iter().enumerate())
            .filter(|(_, r)| match r.by {
                Some(RankBy::Given(q)) => given(q),
                Some(RankBy::Done(q)) => done(q),
                None => false,
            })
            .map(|(i, _)| i)
            .last()
    }

    /// The colour rank `i`'s title shows in: its material's, else `fallback`.
    pub fn color(&self, content: &Content, i: usize, fallback: Rgb) -> Rgb {
        (content.material(self.ranks[i].material).color).unwrap_or(fallback)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GuildFile {
    #[serde(default)]
    guild: Vec<GuildSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GuildSpec {
    id: String,
    name: String,
    master: String,
    ranks: Vec<RankSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RankSpec {
    title: String,
    material: String,
    given: Option<String>,
    done: Option<String>,
}

/// Parses the guilds table (`table`: its files in order), against `content`'s materials,
/// dialogues, and quests.
pub(super) fn parse<'a>(
    table: impl Iterator<Item = (&'a str, &'a str)>,
    content: &Content,
) -> Result<Vec<GuildDef>, ContentError> {
    let mut guilds = Vec::new();
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    for (path, text) in table {
        let parsed: GuildFile =
            toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))?;
        for spec in parsed.guild {
            if let Some(first) = seen.insert(spec.id.clone(), path) {
                return Err(super::duplicate_id("guild", &spec.id, path, first));
            }
            let id = spec.id.clone();
            let campaign = super::campaign::of_path(path);
            let def = resolve(spec, campaign, content)
                .map_err(|m| ContentError::new(path, format!("guild `{id}`: {m}")))?;
            guilds.push(def);
        }
    }
    Ok(guilds)
}

fn resolve(spec: GuildSpec, campaign: Option<&str>, content: &Content) -> Result<GuildDef, String> {
    if !super::dialogue::is_id(&spec.id) {
        return Err("ids are lower-case letters, digits, and underscores".into());
    }
    let name = spec.name.trim();
    if name.is_empty() || name.chars().count() > super::MAX_NAME {
        return Err(format!("`name` is 1 to {} characters", super::MAX_NAME));
    }
    let master = (content.speaker(&spec.master)).ok_or_else(|| {
        format!(
            "`master`: `{}` is no one who talks (a creature or npc with a dialogue)",
            spec.master
        )
    })?;
    if spec.ranks.is_empty() || spec.ranks.len() > MAX_RANKS {
        return Err(format!("a guild has 1 to {MAX_RANKS} `ranks`"));
    }
    let quest = |id: &str| {
        let q = (content.quest_id(id)).ok_or_else(|| format!("unknown quest `{id}`"))?;
        if content.quest(q).campaign.as_deref() != campaign {
            return Err(format!("quest `{id}` is another campaign's"));
        }
        Ok(q)
    };
    let mut ranks = Vec::with_capacity(spec.ranks.len());
    for r in spec.ranks {
        let title = r.title.trim();
        if title.is_empty() || title.chars().count() > MAX_TITLE {
            return Err(format!("a rank's `title` is 1 to {MAX_TITLE} characters"));
        }
        if ranks.iter().any(|k: &RankDef| k.title == title) {
            return Err(format!("rank `{title}` is listed twice"));
        }
        let material = (content.material_id(&r.material))
            .ok_or_else(|| format!("rank `{title}`: unknown material `{}`", r.material))?;
        let by = match (r.given, r.done) {
            (Some(_), Some(_)) => {
                return Err(format!(
                    "rank `{title}`: reached by `given` or `done`, not both"
                ));
            }
            (Some(q), None) => Some(RankBy::Given(quest(&q)?)),
            (None, Some(q)) => Some(RankBy::Done(quest(&q)?)),
            (None, None) => None,
        };
        if by.is_some() && ranks.last().is_some_and(|k: &RankDef| k.by.is_none()) {
            return Err(format!(
                "rank `{title}`: a rank nothing reaches yet comes after those something does"
            ));
        }
        ranks.push(RankDef {
            title: title.to_string(),
            material,
            by,
        });
    }
    Ok(GuildDef {
        id: spec.id,
        name: name.to_string(),
        master,
        ranks,
        campaign: campaign.map(str::to_string),
    })
}

impl Content {
    /// The guilds (spec §8 "Quests"), in table order.
    pub fn guilds(&self) -> &[GuildDef] {
        &self.guilds
    }
}
