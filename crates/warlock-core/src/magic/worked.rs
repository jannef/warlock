//! What an anvil's work has done to one item (spec §7 "Special rooms": Temper; the rules are
//! `world/temper.rs`): the state a [`Stack`](crate::object::Stack) keeps, and its pure rules.
//!
//! An item is improved up to three times, each time in an [`Aspect`] its base item has, and the
//! same one may come twice: **Fine**, **Marvellous**, **Masterwork**. A failure leaves it
//! **damaged**, which takes all its improvements and makes it a point worse until it is mended:
//! its Protection if it gives some, else its Defence, else its damage ([`super::Mods::local`]).
//! Acid damages an item the same way (`world/corrode.rs`). A damaged item is repaired or
//! destroyed. Every attempt counts for the item's
//! whole life and is never shown in the game. Affixes are another matter ([`super::Magic`]): the
//! work never touches them.
//!
//! The numbers are read in two places: to hit, plus damage, Defence, and Protection through
//! [`super::Mods::local`], the dice through [`Stack::attack`](crate::object::Stack::attack).

use crate::content::ObjectDef;
use serde::{Deserialize, Serialize};

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
}

impl Worked {
    /// How many of its improvements are `aspect`.
    pub fn count(&self, aspect: Aspect) -> i32 {
        let n = self.improved.iter().filter(|&&a| a == aspect).count();
        i32::try_from(n).unwrap_or(i32::MAX)
    }

    /// How many sides its improvements add to its dice.
    pub fn sides(&self) -> u32 {
        self.count(Aspect::Die).unsigned_abs() * SIDES_PER_DIE
    }

    /// What it is called, ahead of the rest of its name: none for an item that is neither
    /// improved nor damaged (it may have been worked and repaired).
    pub fn word(&self) -> Option<&'static str> {
        if self.damaged {
            return Some(DAMAGED);
        }
        let grade = self.improved.len().checked_sub(1)?;
        GRADES.get(grade).map(|&(word, _)| word)
    }

    /// `worth` as it stands now: multiplied by its grade, or halved (rounded up) while damaged.
    pub fn value(&self, worth: u64) -> u64 {
        if self.damaged {
            return worth.div_ceil(2);
        }
        let grade = self.improved.len().checked_sub(1);
        let times = grade.and_then(|g| GRADES.get(g)).map_or(1, |&(_, by)| by);
        worth.saturating_mul(times)
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

    /// It is damaged: all its improvements are gone.
    pub(crate) fn damage(&mut self) {
        self.improved.clear();
        self.damaged = true;
    }

    /// The damage is gone: a plain base item, with its attempts.
    pub(crate) fn repair(&mut self) {
        self.damaged = false;
    }

    /// Whether this could be the state of an item of base item `def`.
    pub(crate) fn is_valid(&self, def: &ObjectDef) -> bool {
        self.improved.len() <= GRADES.len()
            && !(self.damaged && !self.improved.is_empty())
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
        w.damage();
        assert_eq!((w.word(), w.sides(), w.attempts), (Some("damaged"), 0, 3));
        w.repair();
        assert_eq!((w.word(), w.attempts), (None, 3));
    }

    #[test]
    fn the_worth_follows_the_grade() {
        let of = |n: usize| Worked {
            improved: vec![Aspect::ToHit; n],
            ..Worked::default()
        };
        let worth: Vec<u64> = (0..=3).map(|n| of(n).value(81)).collect();
        assert_eq!(worth, [81, 162, 243, 405]);
        let damaged = Worked {
            damaged: true,
            ..Worked::default()
        };
        assert_eq!((damaged.value(81), damaged.value(80)), (41, 40));
    }

    #[test]
    fn a_state_that_cannot_be_is_refused() {
        let c = content();
        let (sword, shirt) = (def(&c, "long_sword"), def(&c, "chain_mail"));
        let of = |improved: Vec<Aspect>, damaged| Worked {
            improved,
            attempts: 1,
            damaged,
        };
        assert!(of(vec![Aspect::Die], false).is_valid(sword));
        assert!(!of(vec![Aspect::Die], false).is_valid(shirt));
        assert!(!of(vec![Aspect::Die], true).is_valid(sword));
        assert!(!of(vec![Aspect::Die; 4], false).is_valid(sword));
    }
}
