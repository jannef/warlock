//! Towns (spec §8 "Dialogue": "Goodwill", #1247; each campaign's `data/towns/`, one file per
//! town): a town's people, and the goodwill its quests earn. Every quest one of its people gives
//! that the player completes makes their services `goodwill` percent cheaper (`world/goodwill.rs`).
//! The town's goodwill is derived from the quest records, never saved.

use super::dialogue::Speaker;
use super::{Content, ContentError, DialogueId};
use serde::Deserialize;

/// A town as data.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TownDef {
    pub id: String,
    /// Its name ("Cēaptūn").
    pub name: String,
    /// Its people: who speaks for it, by npc or creature id (as a quest's `giver` names them).
    /// Their quests earn its goodwill, and their services are cheaper for it.
    pub people: Vec<String>,
    /// How much cheaper, in percent, its people's services are per quest of theirs completed.
    pub goodwill: u32,
    /// Said as a quest of its people's is completed: "Cēaptūn's folk think the better of you."
    pub told: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TownFile {
    #[serde(default)]
    town: Vec<TownDef>,
}

/// Parses the towns table (`table`: its files in order).
pub(super) fn parse<'a>(
    table: impl Iterator<Item = (&'a str, &'a str)>,
) -> Result<Vec<TownDef>, ContentError> {
    let mut towns = Vec::new();
    for (path, text) in table {
        let parsed: TownFile =
            toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))?;
        towns.extend(parsed.town);
    }
    Ok(towns)
}

impl Content {
    /// The towns, in table order.
    pub fn towns(&self) -> &[TownDef] {
        &self.towns
    }

    /// The town whose people include whoever speaks dialogue `id`, if any (the first).
    pub fn town_of(&self, id: DialogueId) -> Option<&TownDef> {
        let speaks = |name: &String| match &self.dialogue(id).speaker {
            Speaker::Npc(npc) => npc == name,
            Speaker::Creature(c) => self.creature(*c).id == *name,
        };
        self.towns.iter().find(|t| t.people.iter().any(speaks))
    }
}
