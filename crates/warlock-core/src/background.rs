//! Background stories (spec §8 "Background story"; docs/dev/background.md): a chosen character's
//! youth, told in eight chapters of one line each (the last the campaign's own), with what it
//! left them. [`tell`] is pure: given the content, the run seed, and what the conditions may read
//! of the finished character ([`Witness`]), it draws each chapter from its own stream,
//! [`Rng::background`] (`background/<race>,<gender>,<class>/<chapter>`), so no other roll moves:
//!
//! 1. the tier: one draw of `below(1000)` against `[tiers]` (scaled by [`Witness::luck`]);
//! 2. the pool: the chapter's lines of that tier whose `when` holds (against the witness and the
//!    tags earlier lines set), falling to the tier below, down to common, when none does; if an
//!    `only` line of the chapter fits, the fitting `only` lines alone (common, whatever the tier);
//! 3. the line: one draw by weight;
//! 4. its effects' dice, in order, from the same stream (gold, skills, an item's alternatives and
//!    a magic item's loot draws).
//!
//! Nothing is counted across chapters, capped, dropped, or drawn again (the owner: no caps, no
//! protection from a bad story). The ids are saved ([`Told`], `Character::story`); the text is
//! told again from the content ([`retell`]), so an edited sentence reaches a saved character
//! and a line removed from the content is skipped.

use crate::alignment::Band;
use crate::character::{Character, Gender};
use crate::content::{Content, Keep, Marks, StoryEffect, StoryLine, WeaponSkillId};
use crate::magic::loot;
use crate::object::Stack;
use crate::rng::Rng;
use crate::skills::SkillId;
use crate::skills::Skills;
use crate::stats::Stat;
use crate::talent::Talents;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// One chapter's line of a story, by ids (what a save keeps).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Told {
    pub chapter: String,
    pub line: String,
}

impl Told {
    /// `chapter/line`, as dumps name it.
    pub fn id(&self) -> String {
        format!("{}/{}", self.chapter, self.line)
    }
}

/// What a story's conditions may read of the character, all settled before it is told.
#[derive(Clone, Copy, Debug)]
pub struct Witness<'a> {
    /// The spec (race, gender, class), the sign, and the birth date.
    pub character: &'a Character,
    /// The age at the start of the run.
    pub age: u32,
    /// Born on a lucky day.
    pub lucky: bool,
    /// The starting band, before the story's own nudges (`None` without alignment).
    pub band: Option<Band>,
    /// Trained skills (the optional class and race skills included).
    pub skills: &'a Skills,
    /// Talents held: the creation picks (Random ones too), the sign's gifts, the class's level 1
    /// grants.
    pub talents: &'a Talents,
    /// Weapon skills and styles with a rank: what an item line's `prefer_trained` prefers.
    pub trained: &'a [WeaponSkillId],
    /// The run's campaign (`""` outside every campaign).
    pub campaign: &'a str,
    /// The luckier tiers' odds in percent of `[tiers]` (100: as written).
    pub luck: u32,
}

impl Witness<'_> {
    /// Whether `line`'s `when` holds for this character with `tags` set so far.
    pub fn fits(&self, line: &StoryLine, tags: &BTreeSet<String>) -> bool {
        let w = &line.when;
        let any = |ids: &[String], id: &str| ids.is_empty() || ids.iter().any(|i| i == id);
        let spec = &self.character.spec;
        let sign = self.character.sign.as_ref();
        any(&w.race, &spec.race)
            && any(&w.class, &spec.class)
            && (w.gender.is_empty() || w.gender.contains(&spec.gender))
            && (w.animal.is_empty() || sign.is_some_and(|s| w.animal.contains(&s.animal)))
            && (w.element.is_empty() || sign.is_some_and(|s| w.element.contains(&s.element)))
            && w.lucky.is_none_or(|l| l == self.lucky)
            && w.band
                .is_none_or(|r| self.band.is_some_and(|b| r.allows(b)))
            && w.talents.iter().all(|t| self.talents.has(t, 1))
            && !w.not_talents.iter().any(|t| self.talents.has(t, 1))
            && w.skills.iter().all(|&s| self.skills.value(s).is_some())
            && w.tags.iter().all(|t| tags.contains(t))
            && !w.not_tags.iter().any(|t| tags.contains(t))
            && w.age.is_none_or(|a| a.holds(self.age))
            && any(&w.campaign, self.campaign)
    }
}

/// An effect with its dice rolled, to apply to the character (`World::tell_background`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rolled {
    /// The score and its potential by this much.
    Stat(Stat, i32),
    /// Gold coins given (taken when negative; never below none).
    Gold(i32),
    /// An item, kept so; a magic one identified.
    Item(Stack, Keep),
    /// A skill started or raised by this much.
    Skill(SkillId, i32),
    /// Marks toward the next rank.
    WeaponMarks(Marks, u32),
    /// The starting alignment score moved.
    Alignment(i32),
}

/// A story told: the lines (saved), and their effects rolled (applied once, never saved).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Story {
    pub told: Vec<Told>,
    pub effects: Vec<Rolled>,
}

/// The story of the character `witness` describes in a run with `seed` (see the module docs).
/// Empty for content without background stories.
pub fn tell(content: &Content, seed: u64, witness: &Witness) -> Story {
    let mut story = Story::default();
    let Some(background) = content.background() else {
        return story;
    };
    let mut tags: BTreeSet<String> = BTreeSet::new();
    for chapter in background.chapters_for(witness.campaign) {
        let mut rng = Rng::background(seed, &witness.character.spec, &chapter.id);
        let mut tier = Some(background.tiers.of(rng.below(1000), witness.luck));
        let fits = |l: &&StoryLine| witness.fits(l, &tags);
        let mut pool: Vec<&StoryLine> = (chapter.lines.iter())
            .filter(|l| l.only)
            .filter(fits)
            .collect();
        while pool.is_empty() {
            let Some(t) = tier else {
                break;
            };
            pool = (chapter.lines.iter())
                .filter(|l| l.tier == t)
                .filter(fits)
                .collect();
            tier = t.below();
        }
        let total: u32 = pool.iter().map(|l| l.weight).sum();
        if total == 0 {
            continue;
        }
        let mut at = rng.below(total);
        let line = *pool
            .iter()
            .find(|l| {
                let here = at < l.weight;
                at = at.saturating_sub(l.weight);
                here
            })
            .expect("the draw is below the total weight");
        for effect in &line.effects {
            story
                .effects
                .extend(roll(content, &mut rng, effect, witness));
        }
        tags.extend(line.tags.iter().cloned());
        story.told.push(Told {
            chapter: chapter.id.clone(),
            line: line.id.clone(),
        });
    }
    story
}

/// One effect with its dice rolled from `rng`.
fn roll(content: &Content, rng: &mut Rng, effect: &StoryEffect, witness: &Witness) -> Vec<Rolled> {
    match effect {
        StoryEffect::Stat(stat, by) => vec![Rolled::Stat(*stat, *by)],
        StoryEffect::Gold(dice) => vec![Rolled::Gold(dice.roll(rng))],
        StoryEffect::Skill(id, dice) => vec![Rolled::Skill(*id, dice.roll(rng))],
        StoryEffect::WeaponMarks(to, count) => vec![Rolled::WeaponMarks(*to, *count)],
        StoryEffect::Alignment(by) => vec![Rolled::Alignment(*by)],
        StoryEffect::Item(item) => {
            let picked = item.pick(content, rng, witness.trained);
            (item.one_of[picked].iter())
                .map(|stack| {
                    let stack = match item.magic {
                        Some(grade) => {
                            let (object, count) = (stack.object, stack.count);
                            let mut made = loot::make(
                                content,
                                rng,
                                object,
                                count,
                                crate::kit::ITEM_LEVEL,
                                grade,
                            );
                            made.identify();
                            made
                        }
                        None => stack.clone(),
                    };
                    Rolled::Item(stack, item.keep)
                })
                .collect()
        }
    }
}

/// The paragraphs of `character`'s story in a run of `campaign`, told again from the content
/// with the slots filled: empty without a story. A line the content no longer has is skipped
/// (its `set` with it), and the paragraph closes up.
pub fn retell(content: &Content, campaign: &str, character: &Character) -> Vec<String> {
    let mut paragraphs: BTreeMap<u8, Vec<String>> = BTreeMap::new();
    for (_, paragraph, text) in lines(content, campaign, character) {
        paragraphs.entry(paragraph).or_default().push(text);
    }
    paragraphs
        .into_values()
        .map(|lines| lines.join(" "))
        .collect()
}

/// Each line of `character`'s story the content (in a run of `campaign`) has, in order: its ids,
/// its paragraph, and its text with the slots filled.
pub fn lines<'c>(
    content: &Content,
    campaign: &str,
    character: &'c Character,
) -> Vec<(&'c Told, u8, String)> {
    let Some(background) = content.background() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut slots: BTreeMap<String, String> = BTreeMap::new();
    let mut set: BTreeMap<String, String> = BTreeMap::new();
    for chapter in background.chapters_for(campaign) {
        slots.extend(chapter.defaults.clone());
        let Some(told) = character.story.iter().find(|t| t.chapter == chapter.id) else {
            continue;
        };
        let Some(line) = chapter.line(&told.line) else {
            continue;
        };
        let mut words = slots.clone();
        words.extend(set.clone());
        out.push((told, chapter.paragraph, fill(&line.text, character, &words)));
        set.extend(line.set.clone());
    }
    out
}

/// The lines of `character`'s story the content (in a run of `campaign`) no longer has, as log
/// lines: they are skipped when the story is told again.
pub fn missing(content: &Content, campaign: &str, character: &Character) -> Vec<String> {
    let known = |t: &Told| {
        (content.background())
            .and_then(|b| b.chapter(campaign, &t.chapter))
            .is_some_and(|c| c.line(&t.line).is_some())
    };
    (character.story.iter())
        .filter(|t| !known(t))
        .map(|t| format!("story line `{}` is not in the content: skipped", t.id()))
        .collect()
}

/// The pronouns of `gender`: they, them, their.
fn pronouns(gender: Gender) -> [&'static str; 3] {
    match gender {
        Gender::Male => ["he", "him", "his"],
        Gender::Female => ["she", "her", "her"],
        Gender::Other => ["they", "them", "their"],
    }
}

/// `text` with its slots filled: the name, the pronouns, and `words` (the chapter defaults and
/// what earlier lines set); a capitalised slot capitalised.
fn fill(text: &str, character: &Character, words: &BTreeMap<String, String>) -> String {
    let [they, them, their] = pronouns(character.spec.gender);
    let mut out = String::new();
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else {
            break;
        };
        let slot = &after[..close];
        let word = match slot.to_lowercase().as_str() {
            "name" => character.name.clone(),
            "they" => they.to_string(),
            "them" => them.to_string(),
            "their" => their.to_string(),
            other => words.get(other).cloned().unwrap_or_default(),
        };
        let capital = slot.starts_with(|c: char| c.is_uppercase());
        out.push_str(&match capital {
            true => capitalised(&word),
            false => word,
        });
        rest = &after[close + 1..];
    }
    out.push_str(rest);
    out
}

fn capitalised(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// The built-in content without background stories (tests of what a story would add to: the
/// kits, the heirlooms, the attacks of a kitted run).
#[cfg(test)]
pub(crate) fn storyless() -> Content {
    let mut files = crate::content::ContentFiles::builtin();
    files.remove("data/background.toml");
    Content::load(&files).expect("built-in content without stories")
}

#[cfg(test)]
#[path = "background_tests.rs"]
mod tests;
