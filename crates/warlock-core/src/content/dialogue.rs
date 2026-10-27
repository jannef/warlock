//! Dialogue (`assets/data/dialogue.toml` and the campaigns' `data/dialogue/`, spec §7 "Dialogue", #146): what creatures say when the
//! player chats with them (`C`), and what some of them mutter to themselves.
//!
//! Each `[[dialogue]]` belongs to a creature type (`creature = "gardener"`) or to one unique
//! character (`npc = "hermit"`, which a level gives a creature with `npc = "hermit"` in its legend
//! entry; it wins over its type's), which may have a proper `name` ("Sister Hild": what messages,
//! look mode, and the dialogue screen call it, without an article). It has one-liners (`lines`)
//! that rotate, an optional `greet` said the first time the player chats with it, optional cold greetings (`greet_evil`, said by a
//! good speaker to an Evil or Wicked character in place of its greeting and lines, and
//! `greet_good`, by an evil one to a Good or Saintly character; spec §8 "Alignment"), optional `[[dialogue.topic]]`s (`id`, `name`,
//! `reply`, and an optional `route`: the way it describes, marked on the player's overworld map
//! as they ask) asked about on the dialogue screen, optional `[[dialogue.service]]`s (`id`, `name`,
//! `price`, `does`, `reply`, `nothing`: a cure or a mending for gold, asked for like a topic;
//! `world/service.rs`), and an optional `mutter`: the percent chance per
//! action that it mutters one of the mutter's `lines` (with a `verb`, "mutters" by default) while
//! the player is near enough to hear it.

use super::creature::CreatureId;
use super::{Content, ContentError};
use crate::geom::Pos;
use serde::Deserialize;
use std::ops::RangeInclusive;

/// The longest one-liner, greeting, or muttered line, in characters: with the speaker's name it
/// fits the message line.
pub const MAX_LINE: usize = 72;
/// The longest reply to a topic, in characters: the dialogue screen wraps it.
pub const MAX_REPLY: usize = 280;
/// The longest proper name of an npc, in characters (it heads its lines on the message line).
pub const MAX_NAME: usize = 24;
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
    name: Option<String>,
    greet: Option<String>,
    #[serde(default)]
    greet_evil: Vec<String>,
    #[serde(default)]
    greet_good: Vec<String>,
    #[serde(default)]
    lines: Vec<String>,
    #[serde(default)]
    topic: Vec<Topic>,
    #[serde(default)]
    service: Vec<Service>,
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
    /// The way the speaker describes, as waypoints `[x, y]` on the campaign's overworld (spec §8
    /// "Dialogue", "Rumoured ways"): asking marks on the player's map what they would have seen
    /// walking it by day ([`crate::world`]'s `world/rumour.rs`). Empty for most topics.
    #[serde(default)]
    pub route: Vec<[i32; 2]>,
}

impl Topic {
    /// Every cell of the topic's route, in order: its waypoints joined by straight lines
    /// ([`Pos::line_between`]). Empty without one.
    pub fn route_cells(&self) -> Vec<Pos> {
        let mut cells: Vec<Pos> = Vec::new();
        for &[x, y] in &self.route {
            let to = Pos::new(x, y);
            if let Some(&from) = cells.last() {
                cells.extend(from.line_between(to));
            }
            cells.push(to);
        }
        cells
    }
}

/// What a paid service does (`does`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceKind {
    /// Ends every status and disease that prayer cures (`cured_by` lists `prayer`).
    Cure,
    /// Identifies everything the player carries that is not yet identified, paid per stack.
    Identify,
    /// Mends every point of stat damage at once.
    Mend,
}

impl ServiceKind {
    /// Its id in the data: `cure`, `identify`, `mend`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Cure => "cure",
            Self::Identify => "identify",
            Self::Mend => "mend",
        }
    }
}

/// A service sold for gold, asked for on the dialogue screen like a topic (campaign design
/// §6.1).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Service {
    /// Its id in replays (`ask:E:cure`), unique among the speaker's topics.
    pub id: String,
    /// What the player picks: "A cure for what ails you" (the screen adds the price).
    pub name: String,
    /// What it costs, in the shops' currency, before talents; for `identify`, per unidentified
    /// stack carried.
    pub price: u32,
    /// What it does.
    pub does: ServiceKind,
    /// What the speaker says when it is done.
    pub reply: String,
    /// What the speaker says when there is nothing to do: "Nothing ails you, child."
    pub nothing: String,
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
    /// A unique character's proper name ("Sister Hild"), said without an article; `None`: the
    /// speaker is called by its creature type ("the old hermit").
    pub name: Option<String>,
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
    /// What the speaker sells as a service, listed after the topics.
    pub services: Vec<Service>,
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

    /// The service with id `id`.
    pub fn service(&self, id: &str) -> Option<&Service> {
        self.services.iter().find(|s| s.id == id)
    }
}

impl Content {
    /// What whoever speaks dialogue `id` is called inside a sentence: its proper name ("Sister
    /// Hild"), else its type's with the article ("the old hermit").
    pub fn speaker_called(&self, id: DialogueId) -> String {
        match &self.dialogue(id).name {
            Some(name) => name.clone(),
            None => format!("the {}", self.speaker_name(id)),
        }
    }

    /// Checks every topic's `route` (spec §8 "Dialogue", "Rumoured ways"): only in a campaign's
    /// dialogue, at least two waypoints, and every cell of it on walkable ground of the
    /// campaign's overworld, so the way it marks is one the player could walk.
    pub(super) fn check_topic_routes(&self) -> Result<(), ContentError> {
        for dialogue in &self.dialogues {
            for topic in dialogue.topics.iter().filter(|t| !t.route.is_empty()) {
                let campaign = dialogue.campaign.as_deref().unwrap_or("none");
                let file = format!("campaigns/{campaign}/data/dialogue");
                let who = match &dialogue.speaker {
                    Speaker::Creature(c) => format!("creature `{}`", self.creature(*c).id),
                    Speaker::Npc(npc) => format!("npc `{npc}`"),
                };
                let bad = |m: String| {
                    ContentError::new(
                        &file,
                        format!("dialogue of {who}: topic `{}`: {m}", topic.id),
                    )
                };
                let overworld = (dialogue.campaign.as_deref())
                    .and_then(|c| self.overworld_of(c))
                    .ok_or_else(|| bad("a `route` needs its campaign's overworld".into()))?;
                if topic.route.len() < 2 {
                    return Err(bad("a `route` has at least two waypoints".into()));
                }
                for p in topic.route_cells() {
                    let walkable =
                        (overworld.terrain.get(p)).is_some_and(|t| self.terrain(*t).walkable);
                    if !walkable {
                        return Err(bad(format!("the route leaves walkable ground at {p}")));
                    }
                }
            }
        }
        Ok(())
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
        match (&spec.name, &speaker) {
            (Some(_), Speaker::Creature(_)) => {
                return Err(bad("only an npc has a `name` of its own".into()));
            }
            (Some(name), _) if name.trim().is_empty() || name.chars().count() > MAX_NAME => {
                return Err(bad(format!("`name` is 1 to {MAX_NAME} characters")));
            }
            _ => {}
        }
        let said = (spec.greet.iter())
            .chain(&spec.lines)
            .chain(&spec.greet_evil)
            .chain(&spec.greet_good);
        check_lines(said, MAX_LINE).map_err(bad)?;
        if spec.topic.len() + spec.service.len() > MAX_TOPICS {
            return Err(bad(format!("at most {MAX_TOPICS} topics and services")));
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
        for (i, service) in spec.service.iter().enumerate() {
            let bad = |m: String| bad(format!("service `{}`: {m}", service.id));
            if !is_id(&service.id) {
                return Err(bad(
                    "ids are lower-case letters, digits, and underscores".into()
                ));
            }
            let taken = (spec.topic.iter()).any(|t| t.id == service.id)
                || spec.service[..i].iter().any(|s| s.id == service.id);
            if taken {
                return Err(bad("its id is a topic's or another service's".into()));
            }
            if service.price == 0 {
                return Err(bad("`price` is at least 1 gold".into()));
            }
            if content.shops().is_none() {
                return Err(bad(
                    "it is paid in the shops' currency: data/shops.toml is needed".into(),
                ));
            }
            check_lines([&service.name], MAX_LINE)
                .and_then(|()| check_lines([&service.reply, &service.nothing], MAX_REPLY))
                .map_err(bad)?;
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
            name: spec.name,
            greet: spec.greet,
            greet_evil: spec.greet_evil,
            greet_good: spec.greet_good,
            lines: spec.lines,
            topics: spec.topic,
            services: spec.service,
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
