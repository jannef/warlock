//! Dialogue (`assets/data/dialogue.toml` and the campaigns' `data/dialogue/`, spec §7 "Dialogue", #146): what creatures say when the
//! player chats with them (`C`), and what some of them mutter to themselves.
//!
//! Each `[[dialogue]]` belongs to a creature type (`creature = "gardener"`) or to one unique
//! character (`npc = "hermit"`, which a level gives a creature with `npc = "hermit"` in its legend
//! entry; it wins over its type's). It has one-liners (`lines`) that rotate, an optional `greet`
//! said the first time the player chats with it, optional cold greetings (`greet_evil`, said by a
//! good speaker to an Evil or Wicked character in place of its greeting and lines, and
//! `greet_good`, by an evil one to a Good or Saintly character; spec §8 "Alignment"), optional `[[dialogue.topic]]`s (`id`, `name`,
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
    #[serde(default)]
    dialogue: Vec<DialogueSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DialogueSpec {
    creature: Option<String>,
    npc: Option<String>,
    greet: Option<String>,
    #[serde(default)]
    greet_evil: Vec<String>,
    #[serde(default)]
    greet_good: Vec<String>,
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
    /// What a good speaker says to an Evil or Wicked character instead of its greeting and
    /// lines, in turn (spec §8 "Alignment"); none: its usual words.
    pub greet_evil: Vec<String>,
    /// What an evil speaker says to a Good or Saintly character, the same way.
    pub greet_good: Vec<String>,
    /// One-liners, said in turn ([`crate::world`]'s rotation: each once a round, in an order of
    /// the speaker's own, never the same twice in a row).
    pub lines: Vec<String>,
    /// What the player may ask about; with any, chatting opens the dialogue screen.
    pub topics: Vec<Topic>,
    pub mutter: Option<Mutter>,
    /// The campaign whose `data/dialogue/` it is in; `None` for shared ones (the shopkeeper's)
    /// and those outside every campaign.
    pub campaign: Option<String>,
}

impl Dialogue {
    /// The topic with id `id`.
    pub fn topic(&self, id: &str) -> Option<&Topic> {
        self.topics.iter().find(|t| t.id == id)
    }
}

/// Parses the dialogue table (`data/dialogue.toml`, `data/dialogue/`, and each campaign's
/// `data/dialogue/`; [`super::ContentFiles::table`]) against the creature types loaded already.
pub(super) fn parse<'a>(
    table: impl Iterator<Item = (&'a str, &'a str)>,
    content: &Content,
) -> Result<Vec<Dialogue>, ContentError> {
    let mut dialogues: Vec<Dialogue> = Vec::new();
    let mut files: Vec<&str> = Vec::new();
    for (file, text) in table {
        parse_file(file, text, content, &mut dialogues, &mut files)?;
    }
    if dialogues.len() > usize::from(u16::MAX) {
        return Err(ContentError::new(
            files.last().copied().unwrap_or("data/dialogue.toml"),
            "too many dialogues",
        ));
    }
    Ok(dialogues)
}

/// One file of the dialogue table, added to `dialogues` (and `files`, the file of each).
fn parse_file<'a>(
    file: &'a str,
    text: &str,
    content: &Content,
    dialogues: &mut Vec<Dialogue>,
    files: &mut Vec<&'a str>,
) -> Result<(), ContentError> {
    let err = |m: String| ContentError::new(file, m);
    let parsed: DialogueFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
    let campaign = super::campaign::of_path(file).map(str::to_string);
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
        if let Some(i) = dialogues.iter().position(|d| d.speaker == speaker) {
            let also = if files[i] == file {
                String::new()
            } else {
                format!(" (also in {})", files[i])
            };
            return Err(bad(format!("there is already one{also}")));
        }
        if spec.lines.is_empty() {
            return Err(bad("`lines` needs at least one line".into()));
        }
        let said = (spec.greet.iter())
            .chain(&spec.lines)
            .chain(&spec.greet_evil)
            .chain(&spec.greet_good);
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
            greet_evil: spec.greet_evil,
            greet_good: spec.greet_good,
            lines: spec.lines,
            topics: spec.topic,
            mutter,
            campaign: campaign.clone(),
        });
        files.push(file);
    }
    Ok(())
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
