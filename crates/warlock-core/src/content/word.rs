//! Words a speaker knows (`[[dialogue.word]]`, spec §8 "Dialogue", "Saying words";
//! docs/design/prestige-classes.md §6.1): a `phrase` the player may say to it, its `reply`, and
//! what saying it `does`. With them come the dialogue's `unheard` (the line for words it does
//! not know), `speaks_to` (the alignment bands it talks to at all), and `silent` (what it does
//! for anyone else). Parsed here for `dialogue.rs`; said in `world/words.rs`.

use super::dialogue::{MAX_LINE, check_lines};
use super::{Content, ContentError, Speaker};
use crate::alignment::Requirement;
use crate::words::{MAX_SAID, normalise};
use serde::Deserialize;

/// The most words one dialogue knows.
pub const MAX_WORDS: usize = 9;
/// The most lines of one reply: each is said as a message.
pub const MAX_REPLY_LINES: usize = 9;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WordSpec {
    phrase: String,
    reply: Vec<String>,
    #[serde(default)]
    does: Does,
}

/// What saying a word does, done in this order: `calls`, `quest`, `departs` (after the reply,
/// so the farewell is heard).
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Does {
    /// A calling of the dialogue's campaign (prestige classes, #876): refused when content loads
    /// until callings exist.
    pub calls: Option<String>,
    /// A quest of the dialogue's campaign, given as asking its topic would give it.
    pub quest: Option<String>,
    /// The speaker leaves the level once it has replied (a creature's dialogue only).
    #[serde(default)]
    pub departs: bool,
}

/// A word a speaker knows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Word {
    /// As written: "Life before death".
    pub phrase: String,
    /// The phrase normalised ([`normalise`]): what what is said must equal.
    pub said: String,
    /// What the speaker answers, a message a line, in order.
    pub reply: Vec<String>,
    pub does: Does,
}

/// What a dialogue knows of words, parsed from its spec.
pub(super) struct Words {
    pub words: Vec<Word>,
    pub unheard: Option<String>,
    pub speaks_to: Option<Requirement>,
    pub silent: Option<String>,
}

/// Checks and resolves a dialogue's words and the lines around them (the errors are said
/// within the dialogue's).
pub(super) fn resolve(
    specs: Vec<WordSpec>,
    unheard: Option<String>,
    speaks_to: Option<Requirement>,
    silent: Option<String>,
    speaker: &Speaker,
) -> Result<Words, String> {
    if specs.len() > MAX_WORDS {
        return Err(format!("at most {MAX_WORDS} words"));
    }
    let mut words: Vec<Word> = Vec::new();
    for spec in specs {
        let bad = |m: String| format!("word `{}`: {m}", spec.phrase);
        let said = normalise(&spec.phrase);
        if said.is_empty() || spec.phrase.chars().count() > MAX_SAID {
            return Err(bad(format!(
                "a `phrase` is 1 to {MAX_SAID} characters with a letter or digit"
            )));
        }
        if let Some(other) = words.iter().find(|w| w.said == said) {
            return Err(bad(format!("it says what `{}` says", other.phrase)));
        }
        if spec.reply.is_empty() || spec.reply.len() > MAX_REPLY_LINES {
            return Err(bad(format!("`reply` is 1 to {MAX_REPLY_LINES} lines")));
        }
        check_lines(&spec.reply, MAX_LINE).map_err(bad)?;
        if spec.does.calls.is_some() {
            return Err(bad(
                "`calls`: there are no callings yet (prestige classes, #876)".into(),
            ));
        }
        if spec.does.departs && matches!(speaker, Speaker::Npc(_)) {
            return Err(bad(
                "`departs` is only for a creature type's dialogue: an npc leaving is a level's"
                    .into(),
            ));
        }
        words.push(Word {
            phrase: spec.phrase,
            said,
            reply: spec.reply,
            does: spec.does,
        });
    }
    check_lines(&unheard, MAX_LINE).map_err(|m| format!("`unheard`: {m}"))?;
    match (speaks_to, &silent) {
        (Some(r), Some(line)) => {
            r.check().map_err(|m| format!("`speaks_to`: {m}"))?;
            check_lines([line], MAX_LINE).map_err(|m| format!("`silent`: {m}"))?;
        }
        (Some(_), None) => return Err("`speaks_to` needs a `silent` line".into()),
        (None, Some(_)) => return Err("`silent` is said only outside a `speaks_to`".into()),
        (None, None) => {}
    }
    Ok(Words {
        words,
        unheard,
        speaks_to,
        silent,
    })
}

impl Content {
    /// Checks every word's `quest` once the quests have loaded: a quest of the dialogue's own
    /// campaign.
    pub(super) fn check_words(&self) -> Result<(), ContentError> {
        for dialogue in self.dialogues() {
            for word in &dialogue.words {
                let Some(quest) = &word.does.quest else {
                    continue;
                };
                let campaign = dialogue.campaign.as_deref();
                let found = self.quest_id(quest).map(|q| self.quest(q));
                if found.is_some_and(|q| q.campaign.as_deref() == campaign) {
                    continue;
                }
                let file = match campaign {
                    Some(c) => format!("campaigns/{c}/data/dialogue"),
                    None => "data/dialogue.toml".into(),
                };
                let who = match &dialogue.speaker {
                    Speaker::Creature(c) => format!("creature `{}`", self.creature(*c).id),
                    Speaker::Npc(npc) => format!("npc `{npc}`"),
                };
                return Err(ContentError::new(
                    &file,
                    format!(
                        "dialogue of {who}: word `{}`: no quest `{quest}` in its campaign",
                        word.phrase
                    ),
                ));
            }
        }
        Ok(())
    }
}
