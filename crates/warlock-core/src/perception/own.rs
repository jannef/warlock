//! What the player knows of themselves: their skills, resources, talents, needs, and experience.

use crate::content::{Choice, ResourceId, TalentId, TreeId};
use crate::skills::SkillId;
use serde::Serialize;

/// One of the player's own trained skills, as they know it (spec §8 "Skills"): its value and a
/// coarse sense of how near its next point is. Never its marks or what it gives. (Its active use,
/// if it has one, is an ability: [`PerceptionView::abilities`].)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct OwnSkill {
    pub skill: SkillId,
    pub value: u16,
    /// How practised it is, its marks' step on the skill ladder in quarters: 0 (no marks, or
    /// just spent) to 4 (the top dice): how big its next level-up roll is, coarsely.
    pub progress: u8,
}

/// One of the player's own alternative resources (spec §8 "Resources"), as the status bar and
/// the character screen show it: what it holds and its maximum. Never its pace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnResource {
    pub resource: ResourceId,
    pub value: u16,
    pub max: u16,
    /// Its points are in one creature's guard (`marked` in data: Openings), the mark
    /// [`PerceptionView::own_mark`] names.
    pub marked: bool,
}

/// A grimoire the player could learn from (spec §8 "Spells": "Learning"): one teaching a spell
/// they do not know, or one of a spell they know below its last rank (a deepening, "Ranks"),
/// with the chance in words ("easy", "a struggle", "far beyond you", or "certain"). Never the
/// number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnStudy {
    /// The inventory stack, by position.
    pub item: usize,
    pub spell: crate::content::SpellId,
    /// The rank reading it would bring the spell to; `None` for a spell to learn.
    pub rank: Option<u8>,
    pub chance: &'static str,
}

/// A spell the player could write a scroll of (spec §8 "Scrolls": the wizard's Scribe), as the
/// scribe list shows it: what it costs them in mana (they know their own costs) and how long it
/// takes at a normal pace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnScribe {
    pub spell: crate::content::SpellId,
    pub mana: i32,
    pub time: crate::time::Duration,
}

/// One of the player's own talents (spec §8 "Talents"), as the Talents page and the character
/// screen list it: its name for its choice and what it does in the
/// game's words. Never a rule's number.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnTalent {
    pub id: TalentId,
    /// "Hardy", "Weapon focus (long blades)".
    pub name: String,
    pub tree: TreeId,
    /// The choice it was taken for, by id.
    pub choice: Option<String>,
    pub description: String,
    /// Given, not picked: a birth sign's gift (spec §8 "Birth sign") or a race's.
    pub granted: bool,
    /// Their race gives it (spec §8 "Talents": the humans' Versatile), a granted talent of the
    /// racial tree; `granted` too.
    pub race: bool,
    /// A birth sign's flaw.
    pub flaw: bool,
    /// Their class gives it (spec §8 "Class abilities"): a class ability, or a talent their
    /// class's table hands out at their level or below.
    pub class: bool,
    /// What it gives once in a life has been given (spec §8 "Birth sign": Rise from the ashes,
    /// risen): "(spent)".
    pub spent: bool,
    /// A forgotten altar gave it (spec §7 "Special rooms": Chosen, Forsaken).
    pub boon: bool,
    /// The background story gave it (spec §8 "Background story"): "(story)", "(story, flaw)".
    pub story: bool,
}

/// How fresh food smells to those who can tell (spec §8 "Birth sign": Hound's nose): a word,
/// never its age.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Freshness {
    Fresh,
    /// Old: still fine to eat.
    Turning,
    Rotten,
}

impl Freshness {
    /// The word: `fresh`, `turning`, `rotten`.
    pub const fn word(self) -> &'static str {
        match self {
            Freshness::Fresh => "fresh",
            Freshness::Turning => "turning",
            Freshness::Rotten => "rotten",
        }
    }
}

/// A talent as the Talents page and the level-up screen list it (spec §8 "Talents"): whether
/// the player holds it, what it asks and whether each part holds, and the choices it may be
/// taken for now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TalentView {
    pub id: TalentId,
    pub name: String,
    pub tree: TreeId,
    pub choice: Option<Choice>,
    /// Whether the player holds it, for any choice.
    pub held: bool,
    /// Given, never picked.
    pub granted: bool,
    pub description: String,
    /// What it asks, in words, each with whether it holds now.
    pub needs: Vec<NeedView>,
    /// The ways it may be taken now: `[None]` for a talent without a choice, the choice ids
    /// for one with a choice; empty when it may not be taken now.
    pub openings: Vec<Option<String>>,
}

/// One prerequisite as shown: "Toughness 14", met or not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NeedView {
    pub words: String,
    pub met: bool,
}

/// The player's experience (spec §8 "Experience"), all of which they are shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Progress {
    pub level: u8,
    /// The experience total.
    pub xp: u32,
    /// The total at which the next level comes; none at the highest level.
    pub next: Option<u32>,
}
