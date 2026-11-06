//! Grades (spec §8 "Grades"; `docs/design/monster-dread.md` §7.4): one creature in
//! six comes greater than its kind and one in six lesser, so two kobolds are two fights.
//!
//! The grade is [`Actor::grade`] (−1, 0, +1), drawn once as the creature is made
//! ([`draw`], one line in `travel::make_creature`) from its own stream `grade/<actor id>`
//! ([`Rng::grade`]), so no other draw moves with it. A type with `grades = false` (the people),
//! a unique, a mimic, an animated type but the blades (their tier's `grade_words`), and a
//! unique character (a spawn with an `npc`) are never graded; a kept beast (a companion or a familiar) reads as ungraded
//! ([`Actor::graded`]), its own growth standing in for it.
//!
//! What it does, read in four places of the rules: maximum hit points ([`hp`],
//! `Actor::max_hp`), the melee blow's to-hit and damage ([`sharpen`], `Actor::melee`), and
//! speed ([`speed_percent`], `Actor::speed`). The kill pays its kind's experience either way.
//! The player sees it as a word in look mode ([`word`]: "a big kobold"); messages name the
//! type alone.

use crate::combat::Profile;
use crate::content::{Content, CreatureDef, CreatureSpawn};
use crate::rng::Rng;
use crate::world::{Actor, ActorId};

/// One in this many creatures that may be graded comes lesser, and one in as many greater.
pub const ONE_IN: u32 = 6;

/// Maximum hit points a greater creature has more than its kind, in percent (a lesser one as
/// many less).
pub const HP_PERCENT: i32 = 25;

/// To hit a greater creature's melee blows have more (a lesser one's less).
pub const TO_HIT: i32 = 1;

/// Damage a greater creature's melee blows do more (a lesser one's less).
pub const DAMAGE: i32 = 2;

/// Speed a greater creature has more than its kind, in percent: less, a slower step (a lesser
/// one is as much quicker).
pub const SPEED_PERCENT: i32 = -10;

/// The words of the lesser and greater grade of a type whose file and tags name none.
pub const DEFAULT_WORDS: [&str; 2] = ["small", "big"];

/// The words of the lesser and greater grade of a type tagged so, where its file names none:
/// the first tag listed here that it carries wins.
pub const TAG_WORDS: &[(&str, [&str; 2])] = &[("undead", ["withered", "ancient"])];

/// The grade words of a type: its file's `grade_words`, else its tags', else the defaults.
pub(crate) fn words(own: Option<[String; 2]>, tags: &[String]) -> [String; 2] {
    own.unwrap_or_else(|| {
        let words = (TAG_WORDS.iter())
            .find(|(tag, _)| tags.iter().any(|t| t == tag))
            .map_or(DEFAULT_WORDS, |&(_, words)| words);
        words.map(str::to_string)
    })
}

/// The grade of the creature `spawn` places as actor `id` in the run of `seed`: −1 lesser, +1
/// greater, 0 neither, one in [`ONE_IN`] each; always 0 for a type with no grades or a unique
/// character. One draw on `grade/<actor id>`, and nothing else.
pub fn draw(content: &Content, seed: u64, id: ActorId, spawn: &CreatureSpawn) -> i8 {
    if !content.creature(spawn.creature).grades || spawn.npc.is_some() {
        return 0;
    }
    match Rng::grade(seed, &id.0.to_string()).below(ONE_IN) {
        0 => -1,
        n if n == ONE_IN - 1 => 1,
        _ => 0,
    }
}

/// Maximum hit points `max` at grade `grade`: [`HP_PERCENT`] more or less, never below 1.
pub fn hp(max: i32, grade: i8) -> i32 {
    if grade == 0 {
        return max;
    }
    let percent = 100 + HP_PERCENT * i32::from(grade);
    (max * percent / 100).max(1)
}

/// `profile`, a melee blow, at grade `grade`: [`TO_HIT`] and [`DAMAGE`] more or less.
pub fn sharpen(mut profile: Profile, grade: i8) -> Profile {
    profile.to_hit += TO_HIT * i32::from(grade);
    profile.bonus += DAMAGE * i32::from(grade);
    profile
}

/// What grade `grade` adds to speed, in percent.
pub fn speed_percent(grade: i8) -> i32 {
    SPEED_PERCENT * i32::from(grade)
}

/// The word grade `grade` gives a creature of type `def` in look mode ("big"); `None` at 0.
pub fn word(def: &CreatureDef, grade: i8) -> Option<&str> {
    match grade {
        ..0 => Some(&def.grade_words[0]),
        0 => None,
        _ => Some(&def.grade_words[1]),
    }
}

impl Actor {
    /// Its grade as the rules read it: [`Actor::grade`], but none for a kept beast (a companion
    /// or a familiar grows with its master instead).
    pub fn graded(&self) -> i8 {
        if self.kept().is_some() { 0 } else { self.grade }
    }
}

#[cfg(test)]
#[path = "grade_tests.rs"]
mod tests;
