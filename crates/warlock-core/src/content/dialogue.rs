//! Dialogue (`assets/data/dialogue.toml`, spec §7 "Dialogue", #146): what creatures say when the
//! player chats with them (`C`), and what some of them mutter to themselves.
//!
//! Each `[[dialogue]]` belongs to a creature type (`creature = "gardener"`) or to one unique
//! character (`npc = "hermit"`, which a level gives a creature with `npc = "hermit"` in its legend
//! entry; it wins over its type's). It has one-liners (`lines`) that rotate, an optional `greet`
//! said the first time the player chats with it, optional `[[dialogue.topic]]`s (`id`, `name`,
//! `reply`) asked about on the dialogue screen, and an optional `mutter`: the percent chance per
//! action that it mutters one of the mutter's `lines` (with a `verb`, "mutters" by default) while
//! the player is near enough to hear it.

use super::creature::CreatureId;
use super::{Content, ContentError};
use serde::Deserialize;
use std::ops::RangeInclusive;

/// The longest one-liner, greeting, or muttered line, in characters: with the speaker's name it
/// fits the message line.
pub const MAX_LINE: usize = 72;
/// The longest reply to a topic, in characters: the dialogue screen wraps it.
pub const MAX_REPLY: usize = 280;
/// The most topics a creature talks about: they are numbered 1 to 9 on the dialogue screen.
pub const MAX_TOPICS: usize = 9;
/// The chance, in percent per action, that a creature mutters.
pub const MUTTER_CHANCES: RangeInclusive<u8> = 1..=100;

/// Index of a dialogue in [`Content::dialogues`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DialogueId(pub u16);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DialogueFile {
    dialogue: Vec<DialogueSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DialogueSpec {
    creature: Option<String>,
    npc: Option<String>,
    greet: Option<String>,
    #[serde(default)]
    lines: Vec<String>,
    #[serde(default)]
    topic: Vec<Topic>,
    mutter: Option<MutterSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MutterSpec {
    chance: u8,
    verb: Option<String>,
    lines: Vec<String>,
}

/// Whose a dialogue is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Speaker {
    /// Every creature of this type (but those with a dialogue of their own).
    Creature(CreatureId),
    /// The unique character a level names with this `npc` id.
    Npc(String),
}

/// Something to ask about on the dialogue screen.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Topic {
    /// Its id in replays (`ask:E:rooms`): lower-case letters, digits, and underscores.
    pub id: String,
    /// What the player picks: "The rooms".
    pub name: String,
    /// What the speaker answers.
    pub reply: String,
}

/// What a creature mutters to itself now and then.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mutter {
    /// The chance, in percent per action, that it mutters while the player is near.
    pub chance: u8,
    /// How it says them: "mutters", "hums".
    pub verb: String,
    pub lines: Vec<String>,
}

/// What a creature says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dialogue {
    pub speaker: Speaker,
    /// Said the first time the player chats with it, instead of a line.
    pub greet: Option<String>,
    /// One-liners, said in turn ([`crate::world`]'s rotation: each once a round, in an order of
    /// the speaker's own, never the same twice in a row).
    pub lines: Vec<String>,
    /// What the player may ask about; with any, chatting opens the dialogue screen.
    pub topics: Vec<Topic>,
    pub mutter: Option<Mutter>,
}

impl Dialogue {
    /// The topic with id `id`.
    pub fn topic(&self, id: &str) -> Option<&Topic> {
        self.topics.iter().find(|t| t.id == id)
    }
}

/// Parses `data/dialogue.toml` against the creature types loaded already.
pub(super) fn parse(
    file: &str,
    text: &str,
    content: &Content,
) -> Result<Vec<Dialogue>, ContentError> {
    let err = |m: String| ContentError::new(file, m);
    let parsed: DialogueFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
    let mut dialogues: Vec<Dialogue> = Vec::with_capacity(parsed.dialogue.len());
    for spec in parsed.dialogue {
        let speaker = match (spec.creature, spec.npc) {
            (Some(id), None) => match content.creature_id(&id) {
                Some(_) if id == "player" => {
                    return Err(err("the player has no dialogue of their own".into()));
                }
                Some(c) => Speaker::Creature(c),
                None => return Err(err(format!("dialogue for unknown creature `{id}`"))),
            },
            (None, Some(npc)) if is_id(&npc) => Speaker::Npc(npc),
            (None, Some(npc)) => {
                return Err(err(format!(
                    "npc `{npc}`: ids are lower-case letters, digits, and underscores"
                )));
            }
            _ => {
                return Err(err(
                    "every dialogue names exactly one of `creature` and `npc`".into(),
                ));
            }
        };
        let who = match &speaker {
            Speaker::Creature(c) => format!("creature `{}`", content.creature(*c).id),
            Speaker::Npc(npc) => format!("npc `{npc}`"),
        };
        let bad = |m: String| err(format!("dialogue of {who}: {m}"));
        if dialogues.iter().any(|d| d.speaker == speaker) {
            return Err(bad("there is already one".into()));
        }
        if spec.lines.is_empty() {
            return Err(bad("`lines` needs at least one line".into()));
        }
        let said = spec.greet.iter().chain(&spec.lines);
        check_lines(said, MAX_LINE).map_err(bad)?;
        if spec.topic.len() > MAX_TOPICS {
            return Err(bad(format!("at most {MAX_TOPICS} topics")));
        }
        for (i, topic) in spec.topic.iter().enumerate() {
            if !is_id(&topic.id) {
                return Err(bad(format!(
                    "topic `{}`: ids are lower-case letters, digits, and underscores",
                    topic.id
                )));
            }
            if spec.topic[..i].iter().any(|t| t.id == topic.id) {
                return Err(bad(format!("duplicate topic `{}`", topic.id)));
            }
            check_lines([&topic.name], MAX_LINE)
                .and_then(|()| check_lines([&topic.reply], MAX_REPLY))
                .map_err(|m| bad(format!("topic `{}`: {m}", topic.id)))?;
        }
        let mutter = match spec.mutter {
            Some(m) => {
                if !MUTTER_CHANCES.contains(&m.chance) {
                    return Err(bad(format!(
                        "mutter: `chance` is a percent, {} to {}",
                        MUTTER_CHANCES.start(),
                        MUTTER_CHANCES.end()
                    )));
                }
                if m.lines.is_empty() {
                    return Err(bad("mutter: `lines` needs at least one line".into()));
                }
                check_lines(&m.lines, MAX_LINE).map_err(|e| bad(format!("mutter: {e}")))?;
                let verb = m.verb.unwrap_or_else(|| "mutters".into());
                if verb.is_empty() || !verb.chars().all(|c| c.is_ascii_lowercase()) {
                    return Err(bad("mutter: `verb` is one lower-case word".into()));
                }
                Some(Mutter {
                    chance: m.chance,
                    verb,
                    lines: m.lines,
                })
            }
            None => None,
        };
        dialogues.push(Dialogue {
            speaker,
            greet: spec.greet,
            lines: spec.lines,
            topics: spec.topic,
            mutter,
        });
    }
    if dialogues.len() > usize::from(u16::MAX) {
        return Err(err("too many dialogues".into()));
    }
    Ok(dialogues)
}

/// Whether `id` is a topic or npc id: lower-case letters, digits, and underscores.
pub(super) fn is_id(id: &str) -> bool {
    !id.is_empty() && (id.chars()).all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Every line is said as it is written: not empty, one line, at most `max` characters.
pub(super) fn check_lines<'a>(
    lines: impl IntoIterator<Item = &'a String>,
    max: usize,
) -> Result<(), String> {
    for line in lines {
        if line.trim().is_empty() || line.trim() != line {
            return Err(format!("`{line}`: lines are not empty nor padded"));
        }
        if line.contains('\n') || line.chars().count() > max {
            return Err(format!(
                "`{line}`: lines are one line of at most {max} characters"
            ));
        }
    }
    Ok(())
}
