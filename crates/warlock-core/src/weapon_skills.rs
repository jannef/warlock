//! Weapon skills in play (spec §8 "Weapon skills"): an actor's rank and marks in every skill of
//! `data/weapon_skills.toml` ([`crate::content::WeaponSkillTable`]), and how they grow.
//!
//! - **Marks**: each use of a skill earns marks ([`crate::content::MarkRules`]); when they reach
//!   what the curve asks for the next rank, the skill gains it ([`WeaponSkills::train`]), and the
//!   marks left over count toward the one after.
//! - **Defaults** (The Riddle of Steel): when skill A gains a rank, every skill B that defaults
//!   from it with a penalty, `rank(A) − penalty > rank(B)`, rises to `rank(A) − penalty`, and its
//!   marks start again from 0 ([`WeaponSkills::raise`]). Ranks are stored, not derived: a rank
//!   raised by default is a rank like any other. Defaults do not chain within one gain.
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
    /// Starting ranks: at least `rank` in each skill given, and at least `rank − penalty` in
    /// each skill it defaults to (the rest untrained, no marks). The order does not matter.
    pub fn from_ranks(
        table: &WeaponSkillTable,
        ranks: impl IntoIterator<Item = (WeaponSkillId, u8)>,
    ) -> Self {
        let mut skills = Self::default();
        for (id, rank) in ranks {
            let rank = rank.min(MAX_RANK);
            let floors = std::iter::once((id, rank)).chain(
                (table.defaults_of(id).into_iter())
                    .map(|(other, penalty)| (other, rank.saturating_sub(penalty))),
            );
            for (skill, floor) in floors {
                let entry = skills.entry(skill);
                entry.rank = entry.rank.max(floor);
            }
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

    /// Skill `id` reaches `rank` (from a lower one), its marks starting again from 0, and every
    /// skill it defaults to with a penalty rises to `rank − penalty` if that is more than it has
    /// (its marks starting from 0 too). Returns the skills that rose: `id`, then its defaults in
    /// skill order.
    pub fn raise(
        &mut self,
        table: &WeaponSkillTable,
        id: WeaponSkillId,
        rank: u8,
    ) -> Vec<WeaponSkillId> {
        let rank = rank.min(MAX_RANK);
        *self.entry(id) = Training { rank, marks: 0 };
        let mut risen = vec![id];
        for (other, penalty) in table.defaults_of(id) {
            let floor = rank.saturating_sub(penalty);
            if floor > self.rank(other) {
                *self.entry(other) = Training {
                    rank: floor,
                    marks: 0,
                };
                risen.push(other);
            }
        }
        risen
    }

    /// `marks` more in skill `id`. When they reach what the next rank asks for, it gains the
    /// rank ([`WeaponSkills::raise`]: its defaults rise too) and the marks left over count toward
    /// the one after (one rank at a time). Returns the skills that rose; none at [`MAX_RANK`].
    pub fn train(
        &mut self,
        table: &WeaponSkillTable,
        id: WeaponSkillId,
        marks: u32,
    ) -> Vec<WeaponSkillId> {
        let now = self.get(id);
        let Some(need) = table.need(id, now.rank) else {
            return Vec::new();
        };
        let total = now.marks.saturating_add(marks);
        if total < need {
            self.entry(id).marks = total;
            return Vec::new();
        }
        let risen = self.raise(table, id, now.rank + 1);
        let next = table.need(id, now.rank + 1).unwrap_or(1);
        self.entry(id).marks = (total - need).min(next - 1);
        risen
    }

    /// How far skill `id` is toward its next rank: the marks it has and the marks it needs;
    /// `None` at [`MAX_RANK`].
    pub fn progress(&self, table: &WeaponSkillTable, id: WeaponSkillId) -> Option<(u32, u32)> {
        let now = self.get(id);
        table.need(id, now.rank).map(|need| (now.marks, need))
    }
}

/// Starting ranks as data gives them: a fixed number or dice per skill (`{ long_blades = 2,
/// shield_block = "1d2" }`), every result within 0 to [`MAX_RANK`].
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
        assert!(skills.train(&t, long, 14).is_empty());
        assert_eq!(skills.get(long), Training { rank: 0, marks: 14 });
        assert_eq!(skills.progress(&t, long), Some((14, 15)));
        // The 15th mark gains rank 1; three left over count toward rank 2 (30).
        assert_eq!(skills.train(&t, long, 4), [long]);
        assert_eq!(skills.get(long), Training { rank: 1, marks: 3 });
        assert_eq!(skills.progress(&t, long), Some((3, 30)));
        // One rank at a time: a flood of marks stops short of the rank after.
        skills.train(&t, long, 10_000);
        assert_eq!(skills.get(long), Training { rank: 2, marks: 59 });
        // Rank 10 is the last: no more marks.
        let mut master = WeaponSkills::from_ranks(&t, [(long, 10)]);
        assert!(master.train(&t, long, 1_000_000).is_empty());
        assert_eq!(master.get(long), Training { rank: 10, marks: 0 });
        assert_eq!(master.progress(&t, long), None);
        // Missile weapons need fewer marks.
        let bows = id(&t, "bows");
        assert!(t.need(bows, 0) < t.need(long, 0));
    }

    #[test]
    fn a_rank_gained_raises_the_skills_it_defaults_to_and_resets_their_marks() {
        let t = table();
        let (long, short, great, daggers, thrown) = (
            id(&t, "long_blades"),
            id(&t, "short_blades"),
            id(&t, "great_blades"),
            id(&t, "daggers"),
            id(&t, "thrown"),
        );
        let mut skills = WeaponSkills::from_ranks(&t, [(long, 7)]);
        // Long blades 7: short and great blades 5 (−2), daggers 3 (−4).
        assert_eq!(skills.rank(short), 5);
        assert_eq!(skills.rank(great), 5);
        assert_eq!(skills.rank(daggers), 3);
        // Defaults do not chain: thrown (daggers −2, short blades −4) stays untrained.
        assert_eq!(skills.rank(thrown), 0);
        skills.train(&t, short, 100);
        skills.train(&t, daggers, 50);
        assert_eq!(skills.get(short).marks, 100);
        // The owner's example: long blades 8 → short blades 6. Short blades, great blades, and
        // daggers (8 − 4 = 4 > 3) rise, and their marks start again.
        let need = t.need(long, 7).expect("a next rank");
        let risen = skills.train(&t, long, need);
        assert_eq!(risen, [long, daggers, short, great]);
        assert_eq!(skills.get(short), Training { rank: 6, marks: 0 });
        assert_eq!(skills.get(great), Training { rank: 6, marks: 0 });
        assert_eq!(skills.get(daggers), Training { rank: 4, marks: 0 });
        // A skill already as good as the default keeps its rank and its marks: short blades 6
        // gave daggers 4, which long blades 8 does not better.
        let mut skills = WeaponSkills::from_ranks(&t, [(short, 6)]);
        skills.train(&t, short, 7);
        skills.train(&t, daggers, 9);
        let risen = skills.raise(&t, long, 8);
        assert_eq!(risen, [long, great]);
        assert_eq!(skills.get(short), Training { rank: 6, marks: 7 });
        assert_eq!(skills.get(daggers), Training { rank: 4, marks: 9 });
    }

    #[test]
    fn defaults_are_symmetric() {
        let t = table();
        for (a, _) in t.ids() {
            for (b, penalty) in t.defaults_of(a) {
                assert!(t.defaults_of(b).contains(&(a, penalty)), "{a:?} {b:?}");
                // Raising either raises the other by the same penalty.
                let one = WeaponSkills::from_ranks(&t, [(a, 10)]);
                let other = WeaponSkills::from_ranks(&t, [(b, 10)]);
                assert_eq!(one.rank(b), 10 - penalty);
                assert_eq!(other.rank(a), 10 - penalty);
            }
        }
        // Unrelated skills never move.
        let bows = WeaponSkills::from_ranks(&t, [(id(&t, "bows"), 10)]);
        assert_eq!(bows.rank(id(&t, "axes")), 0);
        assert_eq!(bows.rank(id(&t, "crossbows")), 6);
    }

    #[test]
    fn starting_ranks_take_the_highest_and_validate() {
        let t = table();
        let (long, short) = (id(&t, "long_blades"), id(&t, "short_blades"));
        let skills = WeaponSkills::from_ranks(&t, [(short, 3), (long, 4)]);
        assert_eq!((skills.rank(long), skills.rank(short)), (4, 3));
        let skills = WeaponSkills::from_ranks(&t, [(short, 1), (long, 12)]);
        assert_eq!((skills.rank(long), skills.rank(short)), (10, 8));
        // Each given rank raises its defaults, whichever comes first: short blades 5 gives
        // thrown 1 though long blades 8 gave short blades 6.
        let thrown = id(&t, "thrown");
        let one = WeaponSkills::from_ranks(&t, [(long, 8), (short, 5)]);
        let other = WeaponSkills::from_ranks(&t, [(short, 5), (long, 8)]);
        assert_eq!(one, other);
        assert_eq!((one.rank(short), one.rank(thrown)), (6, 1));
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
            serde_json::to_string(&WeaponSkills::from_ranks(&t, [(short, 1)])).expect("serializes");
        assert!(
            json.starts_with('[') && json.contains("{\"rank\":1}"),
            "{json}"
        );
    }
}
