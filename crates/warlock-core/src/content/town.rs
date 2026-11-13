//! Towns (spec §8 "Dialogue": "Goodwill", #1247; each campaign's `data/towns/`, one file per
//! town): a town's people, and the goodwill its quests earn. Every quest one of its people gives
//! that the player completes makes their services `goodwill` percent cheaper (`world/goodwill.rs`).
//! The town's goodwill is derived from the quest records, never saved.
//!
//! A town may keep an aristocrat's own servant (`servant`, #1405; `world/stipend.rs`): a creature
//! of the type named for the player's gender, speaking the npc dialogue named for their race
//! (`speech`, `default` for the races not named), placed on cell `at` of level `level` as it is
//! made, only in a run whose character has the Stipend class ability.

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
    /// An aristocrat's own servant, kept here (`world/stipend.rs`).
    pub servant: Option<TownServant>,
}

/// Where and who a town's servant is (see the module docs).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TownServant {
    /// The level it waits on (the town's own).
    pub level: String,
    /// Its cell there.
    pub at: [i32; 2],
    /// Its creature type by the player's gender (`squire`, `chambermaid`, `servant`).
    pub creature: super::GenderWords,
    /// Its npc dialogue by the player's race id; `default` for every race not named.
    pub speech: std::collections::BTreeMap<String, String>,
}

impl TownServant {
    /// The npc dialogue it speaks to a character of race `race`.
    pub fn npc_for(&self, race: &str) -> &str {
        (self.speech.get(race))
            .or_else(|| self.speech.get("default"))
            .map_or("", String::as_str)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TownFile {
    #[serde(default)]
    town: Vec<TownDef>,
}

/// Parses the towns table (`table`: its files in order). A servant's creature types must be
/// known, its `speech` must have a `default`, and each npc it names must have a dialogue with
/// `[dialogue.stipend]` words.
pub(super) fn parse<'a>(
    table: impl Iterator<Item = (&'a str, &'a str)>,
    content: &Content,
) -> Result<Vec<TownDef>, ContentError> {
    let mut towns = Vec::new();
    for (path, text) in table {
        let parsed: TownFile =
            toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))?;
        for town in &parsed.town {
            if let Some(servant) = &town.servant {
                check_servant(servant, content).map_err(|m| {
                    ContentError::new(path, format!("town `{}`: servant: {m}", town.id))
                })?;
            }
        }
        towns.extend(parsed.town);
    }
    Ok(towns)
}

fn check_servant(servant: &TownServant, content: &Content) -> Result<(), String> {
    let kinds = &servant.creature;
    for id in [&kinds.male, &kinds.female, &kinds.other] {
        if content.creature_id(id).is_none() {
            return Err(format!("unknown creature `{id}`"));
        }
    }
    if !servant.speech.contains_key("default") {
        return Err("`speech` needs a `default`".into());
    }
    for npc in servant.speech.values() {
        let d = content.dialogue_of_npc(npc);
        if d.is_none_or(|d| content.dialogue(d).stipend.is_none()) {
            return Err(format!(
                "npc `{npc}` has no dialogue with `[dialogue.stipend]`"
            ));
        }
    }
    Ok(())
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
