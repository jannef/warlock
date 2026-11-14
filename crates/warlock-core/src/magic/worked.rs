//! What an anvil's work has done to one item (spec §7 "Special rooms": Temper; the rules are
//! `world/temper.rs`): the state a [`Stack`](crate::object::Stack) keeps, and its pure rules.
//!
//! An item is improved up to three times, each time in an [`Aspect`] its base item has, and the
//! same one may come twice: **Fine**, **Marvellous**, **Masterwork**. A failure leaves it
//! **damaged**, which takes all its improvements and makes it a point worse until it is mended:
//! its Protection if it gives some, else its Defence, else its damage ([`super::Mods::local`]).
//! Acid, rust, and heat damage an item the same way (`world/item_damage.rs`), and the state
//! keeps the kind for its word ("rusted", "corroded", "scorched"; a failed attempt's is
//! "damaged"). A damaged item is repaired or destroyed. Every attempt counts for the item's
//! whole life and is never shown in the game. Affixes are another matter ([`super::Magic`]): the
//! work never touches them.
//!
//! A craftsperson's work may also bear their **mark** (Maker's mark, `world/craft_make.rs`):
//! saved with the rest, never lost to damage or repair, worth more ([`Worked::value`]), and
//! read by the patron (`world/patron.rs`) and the capacity (`super::capacity`). A plain marked
//! item carries a `Worked` with nothing else on it.
//!
//! The numbers are read in two places: to hit, plus damage, Defence, and Protection through
//! [`super::Mods::local`], the dice through [`Stack::attack`](crate::object::Stack::attack).

use crate::content::{Content, ItemHarm, ItemKind, ObjectDef};
use crate::object::Stack;
use serde::{Deserialize, Serialize};

/// The make a hand-made item is given in data (`quality = "masterwork"` on a unique, a level's
/// `[[item]]`, the heirloom; docs/design/enchanting.md §4), as the forge's grades give it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Quality {
    Fine,
    Marvellous,
    Masterwork,
}

impl Quality {
    /// How many improvements it is.
    pub const fn improvements(self) -> usize {
        match self {
            Quality::Fine => 1,
            Quality::Marvellous => 2,
            Quality::Masterwork => 3,
        }
    }

    /// An item of base item `def` made at this quality: its improvements in its aspects in turn
    /// (the first first, and round again), each one attempt, with no draw. `None` for an item
    /// with no aspect to better (a ring).
    pub fn worked(self, def: &ObjectDef) -> Option<Worked> {
        let among = aspects(def);
        if among.is_empty() {
            return None;
        }
        let mut worked = Worked::default();
        for i in 0..self.improvements() {
            worked.attempted();
            worked.improve(among[i % among.len()]);
        }
        Some(worked)
    }
}

/// What its dice grow by, in sides, per [`Aspect::Die`].
pub const SIDES_PER_DIE: u32 = 2;

/// The words for an item's improvements, from one up: what it is called, and what its worth is
/// multiplied by.
pub const GRADES: [(&str, u64); 3] = [("Fine", 2), ("Marvellous", 3), ("Masterwork", 5)];

/// What a damaged item is called; its worth is halved, rounded up.
pub const DAMAGED: &str = "damaged";

/// One thing an improvement makes better.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Aspect {
    /// [`SIDES_PER_DIE`] more on its damage dice (`1d8` → `1d10`).
    Die,
    /// +1 to hit.
    ToHit,
    /// +1 damage.
    Damage,
    /// +1 to the Defence it gives: as armour, as a shield, or as a parry, whichever it is.
    Defence,
    /// +1 Protection.
    Protection,
}

impl Aspect {
    /// Every aspect, in the order they are drawn among.
    pub const ALL: [Aspect; 5] = [
        Aspect::Die,
        Aspect::ToHit,
        Aspect::Damage,
        Aspect::Defence,
        Aspect::Protection,
    ];

    /// Its id, as saves and the dev dump write it.
    pub const fn id(self) -> &'static str {
        match self {
            Aspect::Die => "die",
            Aspect::ToHit => "to_hit",
            Aspect::Damage => "damage",
            Aspect::Defence => "defence",
            Aspect::Protection => "protection",
        }
    }

    /// Whether the base item `def` has it: anything with an attack has the first three, anything
    /// that gives Defence (armour, a shield, a parrying weapon) or Protection the others.
    pub fn of(self, def: &ObjectDef) -> bool {
        match self {
            Aspect::Die | Aspect::ToHit | Aspect::Damage => def.attack.is_some(),
            Aspect::Defence => def.armour != 0 || def.shield != 0 || def.parry != 0,
            Aspect::Protection => def.protection != 0,
        }
    }
}

/// The aspects an improvement of base item `def` is drawn among.
pub fn aspects(def: &ObjectDef) -> Vec<Aspect> {
    (Aspect::ALL.into_iter()).filter(|a| a.of(def)).collect()
}

/// Whether `stack` is of a kind an anvil works at all: a weapon, a missile weapon, armour, or a
/// shield (not ammunition, rings, or amulets).
pub fn arms(content: &Content, stack: &Stack) -> bool {
    content.object(stack.object).kind.is_some_and(|k| {
        matches!(k, ItemKind::Weapon | ItemKind::Launcher | ItemKind::Shield) || k.is_armour()
    })
}

/// What the anvil has done to an item so far.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Worked {
    /// Its improvements, oldest first; none while damaged.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) improved: Vec<Aspect>,
    /// Every attempt ever made on it, made or failed. Never shown in the game.
    pub(crate) attempts: u8,
    /// Whether it is damaged.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(crate) damaged: bool,
    /// What damaged it, for its word; none: a failed attempt (or not damaged).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) by: Option<ItemHarm>,
    /// Whether it bears its maker's mark (Maker's mark).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(crate) marked: bool,
}

impl Worked {
    /// How many of its improvements are `aspect`.
    pub fn count(&self, aspect: Aspect) -> i32 {
        let n = self.improved.iter().filter(|&&a| a == aspect).count();
        i32::try_from(n).unwrap_or(i32::MAX)
    }

    /// How many improvements it has now (none while damaged): Fine is one, Masterwork three.
    pub fn improvements(&self) -> usize {
        self.improved.len()
    }

    /// How many sides its improvements add to its dice.
    pub fn sides(&self) -> u32 {
        self.count(Aspect::Die).unsigned_abs() * SIDES_PER_DIE
    }

    /// What it is called, ahead of the rest of its name: none for an item that is neither
    /// improved nor damaged (it may have been worked and repaired).
    pub fn word(&self) -> Option<&'static str> {
        if self.damaged {
            return Some(self.by.map_or(DAMAGED, ItemHarm::word));
        }
        let grade = self.improved.len().checked_sub(1)?;
        GRADES.get(grade).map(|&(word, _)| word)
    }

    /// `worth` as it stands now: multiplied by its grade, or halved (rounded up) while damaged;
    /// then, marked, `mark_percent` % more (rounded down;
    /// [`TalentBook::mark_percent`](crate::content::TalentBook::mark_percent)).
    pub fn value(&self, worth: u64, mark_percent: u32) -> u64 {
        let worth = if self.damaged {
            worth.div_ceil(2)
        } else {
            let grade = self.improved.len().checked_sub(1);
            let times = grade.and_then(|g| GRADES.get(g)).map_or(1, |&(_, by)| by);
            worth.saturating_mul(times)
        };
        match self.marked {
            true => worth.saturating_mul(100 + u64::from(mark_percent)) / 100,
            false => worth,
        }
    }

    /// Whether it bears its maker's mark.
    pub fn marked(&self) -> bool {
        self.marked
    }

    /// It bears its maker's mark from now on.
    pub(crate) fn mark(&mut self) {
        self.marked = true;
    }

    /// Whether its improvements number `most` or more: it can be made no finer.
    pub fn finished(&self, most: u8) -> bool {
        self.improved.len() >= usize::from(most)
    }

    /// One attempt more.
    pub(crate) fn attempted(&mut self) {
        self.attempts = self.attempts.saturating_add(1);
    }

    /// One improvement more, in `aspect`.
    pub(crate) fn improve(&mut self, aspect: Aspect) {
        self.improved.push(aspect);
    }

    /// It is damaged by `harm`: all its improvements are gone.
    pub(crate) fn damage(&mut self, harm: ItemHarm) {
        self.improved.clear();
        self.damaged = true;
        self.by = (harm != ItemHarm::Work).then_some(harm);
    }

    /// The damage is gone: a plain base item, with its attempts.
    pub(crate) fn repair(&mut self) {
        self.damaged = false;
        self.by = None;
    }

    /// Whether this could be the state of an item of base item `def`.
    pub(crate) fn is_valid(&self, def: &ObjectDef) -> bool {
        self.improved.len() <= GRADES.len()
            && !(self.damaged && !self.improved.is_empty())
            && (self.damaged || self.by.is_none())
            && self.improved.iter().all(|a| a.of(def))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::Content;

    fn content() -> Content {
        Content::builtin().expect("content")
    }

    fn def<'c>(content: &'c Content, id: &str) -> &'c ObjectDef {
        content.object(content.object_id(id).expect(id))
    }

    #[test]
    fn the_aspects_are_those_the_base_item_has() {
        let c = content();
        let attack = [Aspect::Die, Aspect::ToHit, Aspect::Damage];
        assert_eq!(aspects(def(&c, "dagger")), attack);
        assert_eq!(aspects(def(&c, "short_bow")), attack);
        let armour = aspects(def(&c, "chain_mail"));
        assert_eq!(armour, [Aspect::Defence, Aspect::Protection]);
    }

    #[test]
    fn each_step_changes_only_what_it_says() {
        let mut w = Worked::default();
        assert_eq!(w.word(), None);
        w.attempted();
        w.improve(Aspect::Die);
        assert_eq!((w.word(), w.sides()), (Some("Fine"), 2));
        w.attempted();
        w.improve(Aspect::Die);
        assert_eq!((w.word(), w.sides()), (Some("Marvellous"), 4));
        w.attempted();
        w.improve(Aspect::ToHit);
        assert_eq!((w.word(), w.count(Aspect::ToHit)), (Some("Masterwork"), 1));
        assert!(w.finished(3) && !Worked::default().finished(3));
        w.damage(ItemHarm::Work);
        assert_eq!((w.word(), w.sides(), w.attempts), (Some("damaged"), 0, 3));
        w.repair();
        assert_eq!((w.word(), w.attempts), (None, 3));
        w.damage(ItemHarm::Rust);
        assert_eq!(w.word(), Some("rusted"));
        w.repair();
        assert_eq!((w.word(), w.by), (None, None));
    }

    #[test]
    fn the_worth_follows_the_grade() {
        let of = |n: usize| Worked {
            improved: vec![Aspect::ToHit; n],
            ..Worked::default()
        };
        let worth: Vec<u64> = (0..=3).map(|n| of(n).value(81, 50)).collect();
        assert_eq!(worth, [81, 162, 243, 405]);
        let damaged = Worked {
            damaged: true,
            ..Worked::default()
        };
        assert_eq!((damaged.value(81, 50), damaged.value(80, 50)), (41, 40));
        // The mark: half again on top of the grade (or the damage), rounded down.
        let marked = |n: usize| Worked {
            marked: true,
            ..of(n)
        };
        let worth: Vec<u64> = (0..=3).map(|n| marked(n).value(81, 50)).collect();
        assert_eq!(worth, [121, 243, 364, 607]);
        let mut damaged = damaged;
        damaged.mark();
        assert_eq!(damaged.value(80, 50), 60);
    }

    #[test]
    fn a_state_that_cannot_be_is_refused() {
        let c = content();
        let (sword, shirt) = (def(&c, "long_sword"), def(&c, "chain_mail"));
        let of = |improved: Vec<Aspect>, damaged| Worked {
            improved,
            attempts: 1,
            damaged,
            by: None,
            marked: false,
        };
        assert!(of(vec![Aspect::Die], false).is_valid(sword));
        assert!(!of(vec![Aspect::Die], false).is_valid(shirt));
        assert!(!of(vec![Aspect::Die], true).is_valid(sword));
        assert!(!of(vec![Aspect::Die; 4], false).is_valid(sword));
    }
}
