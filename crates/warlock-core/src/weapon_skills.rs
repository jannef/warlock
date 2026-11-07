//! Weapon skills in play (spec §8 "Weapon skills"): an actor's rank and marks in every skill of
//! `data/weapon_skills.toml` ([`crate::content::WeaponSkillTable`]), and how they grow.
//!
//! - **Marks**: each use of a skill earns marks ([`crate::content::MarkRules`]); when they reach
//!   what the curve asks for the next rank, the skill gains it ([`WeaponSkills::train`]), and the
//!   marks left over count toward the one after. A skill's rank comes only from its own
//!   training, starting ranks, talents, and the dev `rank:` action: no skill raises another.
//!
//! Every actor has ranks (the player's from their race and class, a creature's from its type);
//! only the player trains. Ranks and marks are part of the world state (saved and hashed). The
//! player knows their own ranks and sees their marks as a bar; what a rank gives stays hidden.

use crate::content::{MAX_RANK, WeaponSkillId, WeaponSkillTable};
use crate::dice::Dice;
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

/// A rank and the marks toward the next.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Training {
    pub rank: u8,
    /// Marks toward the next rank.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub marks: u32,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// An actor's rank and marks in every skill, by [`WeaponSkillId`]; a skill past the end is untrained
/// (rank 0, no marks). Serialized as the list, in skill order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WeaponSkills(Vec<Training>);

impl WeaponSkills {
    /// Starting ranks: at least `rank` in each skill given (the rest untrained, no marks). The
    /// order does not matter.
    pub fn from_ranks(ranks: impl IntoIterator<Item = (WeaponSkillId, u8)>) -> Self {
        let mut skills = Self::default();
        for (id, rank) in ranks {
            let entry = skills.entry(id);
            entry.rank = entry.rank.max(rank.min(MAX_RANK));
        }
        skills.0.truncate(
            (skills.0.iter())
                .rposition(|t| *t != Training::default())
                .map_or(0, |i| i + 1),
        );
        skills
    }

    /// Whether every skill is untrained.
    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|t| *t == Training::default())
    }

    /// The rank and marks in skill `id`.
    pub fn get(&self, id: WeaponSkillId) -> Training {
        self.0.get(usize::from(id.0)).copied().unwrap_or_default()
    }

    /// The rank in skill `id`: 0 untrained.
    pub fn rank(&self, id: WeaponSkillId) -> u8 {
        self.get(id).rank
    }

    /// Whether skill `id` of `table` is listed for its owner: always, unless it is `hidden`
    /// (Two Shields), which shows from its first marks (or a rank).
    pub fn shows(&self, table: &WeaponSkillTable, id: WeaponSkillId) -> bool {
        !table.skill(id).hidden || self.get(id) != Training::default()
    }

    /// Every skill with a rank or marks, in skill order.
    pub fn trained(&self) -> impl Iterator<Item = (WeaponSkillId, Training)> + '_ {
        (self.0.iter().enumerate())
            .filter(|(_, t)| **t != Training::default())
            .map(|(i, t)| (WeaponSkillId(i as u8), *t))
    }

    fn entry(&mut self, id: WeaponSkillId) -> &mut Training {
        let i = usize::from(id.0);
        if self.0.len() <= i {
            self.0.resize(i + 1, Training::default());
        }
        &mut self.0[i]
    }

    /// Whether these ranks fit `table` (a save made with other content): no skill it does not
    /// have, no rank past [`MAX_RANK`], and no more marks than the next rank asks for.
    pub fn is_valid_for(&self, table: &WeaponSkillTable) -> bool {
        self.trained().all(|(id, t)| {
            usize::from(id.0) < table.skills().len()
                && t.rank <= MAX_RANK
                && table
                    .need(id, t.rank)
                    .map_or(t.marks == 0, |need| t.marks < need)
        })
    }

    /// Skill `id` reaches `rank` (capped at [`MAX_RANK`]), its marks starting again from 0.
    pub fn raise(&mut self, id: WeaponSkillId, rank: u8) {
        *self.entry(id) = Training {
            rank: rank.min(MAX_RANK),
            marks: 0,
        };
    }

    /// Skill `id` raised to `rank` if it holds less ([`WeaponSkills::raise`]: marks start
    /// again); a rank held already is untouched (a floor, never a raise). Returns whether it
    /// rose (a talent's hand-out, Travelled hands).
    pub fn raise_to(&mut self, id: WeaponSkillId, rank: u8) -> bool {
        if self.rank(id) >= rank.min(MAX_RANK) {
            return false;
        }
        self.raise(id, rank);
        true
    }

    /// `marks` more in skill `id`. When they reach what the next rank asks for, it gains the
    /// rank and the marks left over count toward the one after (one rank at a time). Returns
    /// whether it gained a rank; never at [`MAX_RANK`].
    pub fn train(&mut self, table: &WeaponSkillTable, id: WeaponSkillId, marks: u32) -> bool {
        let now = self.get(id);
        let Some(need) = table.need(id, now.rank) else {
            return false;
        };
        let total = now.marks.saturating_add(marks);
        if total < need {
            self.entry(id).marks = total;
            return false;
        }
        self.raise(id, now.rank + 1);
        let next = table.need(id, now.rank + 1).unwrap_or(1);
        self.entry(id).marks = (total - need).min(next - 1);
        true
    }

    /// `marks` more in skill `id`, never a rank: they stop one short of what the next rank asks
    /// for (a background story's, spec §8 "Background story"); none at [`MAX_RANK`].
    pub fn add_marks(&mut self, table: &WeaponSkillTable, id: WeaponSkillId, marks: u32) {
        let now = self.get(id);
        if let Some(need) = table.need(id, now.rank) {
            self.entry(id).marks = now.marks.saturating_add(marks).min(need.saturating_sub(1));
        }
    }

    /// How far skill `id` is toward its next rank: the marks it has and the marks it needs;
    /// `None` at [`MAX_RANK`].
    pub fn progress(&self, table: &WeaponSkillTable, id: WeaponSkillId) -> Option<(u32, u32)> {
        let now = self.get(id);
        table.need(id, now.rank).map(|need| (now.marks, need))
    }
}

/// Starting ranks as data gives them: a fixed number or dice per skill (`{ long_blades = 2,
/// shield_bash = "1d2" }`), every result within 0 to [`MAX_RANK`].
pub type RankDice = Vec<(WeaponSkillId, Dice)>;

/// Rolls starting ranks from `rng`, in the order given (fixed numbers draw nothing); the same
/// skill twice adds up.
pub fn roll_ranks(
    ranks: &[(WeaponSkillId, Dice)],
    rng: &mut Rng,
    into: &mut Vec<(WeaponSkillId, i32)>,
) {
    for &(id, dice) in ranks {
        let rolled = dice.roll(rng);
        match into.iter_mut().find(|(s, _)| *s == id) {
            Some((_, rank)) => *rank += rolled,
            None => into.push((id, rolled)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::Content;

    fn table() -> WeaponSkillTable {
        Content::builtin().expect("content").weapon_skills().clone()
    }

    fn id(table: &WeaponSkillTable, name: &str) -> WeaponSkillId {
        table.id(name).expect(name)
    }

    #[test]
    fn marks_gain_ranks_along_the_curve() {
        let t = table();
        let long = id(&t, "long_blades");
        let mut skills = WeaponSkills::default();
        assert!(!skills.train(&t, long, 14));
        assert_eq!(skills.get(long), Training { rank: 0, marks: 14 });
        assert_eq!(skills.progress(&t, long), Some((14, 15)));
        // The 15th mark gains rank 1; three left over count toward rank 2 (45).
        assert!(skills.train(&t, long, 4));
        assert_eq!(skills.get(long), Training { rank: 1, marks: 3 });
        assert_eq!(skills.progress(&t, long), Some((3, 45)));
        // One rank at a time: a flood of marks stops short of the rank after.
        skills.train(&t, long, 10_000);
        assert_eq!(
            skills.get(long),
            Training {
                rank: 2,
                marks: 189
            }
        );
        // Rank 10 is the last: no more marks.
        let mut master = WeaponSkills::from_ranks([(long, 10)]);
        assert!(!master.train(&t, long, 1_000_000));
        assert_eq!(master.get(long), Training { rank: 10, marks: 0 });
        assert_eq!(master.progress(&t, long), None);
        // Missile weapons need fewer marks.
        let bows = id(&t, "bows");
        assert!(t.need(bows, 0) < t.need(long, 0));
    }

    #[test]
    fn a_rank_raises_only_its_own_skill() {
        let t = table();
        let (long, short, daggers) = (
            id(&t, "long_blades"),
            id(&t, "short_blades"),
            id(&t, "daggers"),
        );
        let mut skills = WeaponSkills::from_ranks([(long, 7)]);
        assert_eq!((skills.rank(short), skills.rank(daggers)), (0, 0));
        skills.train(&t, short, 7);
        let need = t.need(long, 7).expect("a next rank");
        assert!(skills.train(&t, long, need));
        assert_eq!(skills.rank(long), 8);
        assert_eq!(skills.get(short), Training { rank: 0, marks: 7 });
        assert!(skills.raise_to(daggers, 4));
        assert!(!skills.raise_to(daggers, 3), "a floor, never a raise");
        assert_eq!(skills.trained().count(), 3);
    }

    #[test]
    fn starting_ranks_take_the_highest_and_validate() {
        let t = table();
        let (long, short) = (id(&t, "long_blades"), id(&t, "short_blades"));
        let skills = WeaponSkills::from_ranks([(short, 3), (long, 4)]);
        assert_eq!((skills.rank(long), skills.rank(short)), (4, 3));
        let skills = WeaponSkills::from_ranks([(short, 1), (long, 12), (short, 0)]);
        assert_eq!((skills.rank(long), skills.rank(short)), (10, 1));
        assert!(skills.is_valid_for(&t));
        assert!(WeaponSkills::default().is_empty());
        assert!(!skills.is_empty());
        let mut bad = WeaponSkills::default();
        *bad.entry(WeaponSkillId(200)) = Training { rank: 1, marks: 0 };
        assert!(!bad.is_valid_for(&t));
        let mut bad = WeaponSkills::default();
        *bad.entry(long) = Training { rank: 0, marks: 15 };
        assert!(!bad.is_valid_for(&t), "15 marks would have been rank 1");
        let json =
            serde_json::to_string(&WeaponSkills::from_ranks([(short, 1)])).expect("serializes");
        assert!(
            json.starts_with('[') && json.contains("{\"rank\":1}"),
            "{json}"
        );
    }
}
