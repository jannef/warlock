//! Hunger (spec §8 "Hunger"), ADOM-like: a nutrition counter that game time burns, the states it
//! puts the player in, and what each state does.
//!
//! **Nutrition** is counted in points (`data/hunger.toml`): a race of hunger 100 % burns one
//! point every `burn_seconds` of game time (60: a point a minute), so long actions, resting,
//! and journeys over the overworld cost food too. A race needs more or less (`hunger` in
//! `data/races.toml`: a halfling 125 %, an elf 80 %), and fighting burns a little extra
//! (`exertion`). The counter starts at `start` and never rises above `max`.
//!
//! **States** ([`Hunger`]), by the least nutrition of each (`data/hunger.toml`): Bloated,
//! Satiated, Not hungry, Hungry, Weak, Fainting, and below the last, Starving. Hungry is a
//! warning only; Weak lowers Strength and Dexterity where the rules use them (combat and burden);
//! Fainting (and Starving) now and then costs the player a few turns; Starving stops hit points
//! coming back and takes them away, until the player starves to death; Bloated makes everything
//! take longer, through the same prices as a burden. The player is told when the state changes,
//! and a change for the hungrier (into Hungry or worse) stops a rest.
//!
//! What the player knows is the state, in words (the status bar shows it unless it is Not
//! hungry): never the counter.

use serde::{Deserialize, Serialize};

/// How hungry the player is, from the hungriest up.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Hunger {
    /// Below the least nutrition of Fainting: hit points ebb away.
    Starving,
    /// Now and then the player faints and loses a few turns.
    Fainting,
    /// Strength and Dexterity count for less.
    Weak,
    /// A warning.
    Hungry,
    /// As it should be: not shown.
    NotHungry,
    /// Well fed.
    Satiated,
    /// Overfed: everything takes longer.
    Bloated,
}

impl Hunger {
    /// Every state, the most fed first.
    pub const ALL: [Hunger; 7] = [
        Hunger::Bloated,
        Hunger::Satiated,
        Hunger::NotHungry,
        Hunger::Hungry,
        Hunger::Weak,
        Hunger::Fainting,
        Hunger::Starving,
    ];

    /// Its name, as the status bar shows it: `Hungry`.
    pub const fn name(self) -> &'static str {
        match self {
            Hunger::Starving => "Starving",
            Hunger::Fainting => "Fainting",
            Hunger::Weak => "Weak",
            Hunger::Hungry => "Hungry",
            Hunger::NotHungry => "Not hungry",
            Hunger::Satiated => "Satiated",
            Hunger::Bloated => "Bloated",
        }
    }

    /// Its id in data files and dumps: `not_hungry`.
    pub const fn id(self) -> &'static str {
        match self {
            Hunger::Starving => "starving",
            Hunger::Fainting => "fainting",
            Hunger::Weak => "weak",
            Hunger::Hungry => "hungry",
            Hunger::NotHungry => "not_hungry",
            Hunger::Satiated => "satiated",
            Hunger::Bloated => "bloated",
        }
    }

    /// Whether Strength and Dexterity count for less (`[weak]` in `data/hunger.toml`): Weak,
    /// Fainting, and Starving.
    pub const fn is_weak(self) -> bool {
        matches!(self, Hunger::Weak | Hunger::Fainting | Hunger::Starving)
    }

    /// Whether the player may faint: Fainting and Starving.
    pub const fn faints(self) -> bool {
        matches!(self, Hunger::Fainting | Hunger::Starving)
    }

    /// What the player is told when the state goes from `from` to `to`; `None` if it did not
    /// change.
    pub const fn news(from: Hunger, to: Hunger) -> Option<&'static str> {
        use Hunger::*;
        if from as u8 == to as u8 {
            return None;
        }
        let hungrier = (to as u8) < (from as u8);
        Some(match (hungrier, to) {
            (true, Satiated) => "You no longer feel bloated.",
            (true, NotHungry) => "You no longer feel satiated.",
            (true, Hungry) => "You are getting hungry.",
            (true, Weak) => "You feel weak from hunger.",
            (true, Fainting) => "You feel faint from hunger.",
            (true, Starving) => "You are starving!",
            (false, Fainting) => "You are no longer starving.",
            (false, Weak) => "You feel less faint.",
            (false, Hungry) => "You feel less weak.",
            (false, NotHungry) => "You are no longer hungry.",
            (false, Satiated) => "You feel satiated.",
            (false, Bloated) => "You feel bloated.",
            // Nothing is hungrier than Starving, nor more fed than Bloated.
            (true, Bloated) | (false, Starving) => return None,
        })
    }

    /// Whether going from `from` to `to` stops a rest: getting hungrier, into Hungry or worse
    /// (spec §6 "Rest"). Satiation wearing off is only logged.
    pub const fn stops_rest(from: Hunger, to: Hunger) -> bool {
        (to as u8) < (from as u8) && (to as u8) <= (Hunger::Hungry as u8)
    }
}

/// The player's hunger state of the moment: the nutrition counter and what game time has banked
/// toward its next changes. Saved; rules only (the player knows their [`Hunger`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Satiation {
    /// Nutrition points: below 0 the player is starving.
    pub nutrition: i32,
    /// Burn banked toward the next point: seconds of game time times the race's hunger percent.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub banked: u64,
    /// Seconds of starving toward the next hit point lost.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub starving: u32,
}

fn is_zero(n: &u64) -> bool {
    *n == 0
}

fn is_zero_u32(n: &u32) -> bool {
    *n == 0
}

impl Satiation {
    pub fn new(nutrition: i32) -> Self {
        Self {
            nutrition,
            banked: 0,
            starving: 0,
        }
    }

    /// `seconds` of game time burn nutrition for a race of hunger `percent`: a point every
    /// `burn_seconds` at 100 %. What does not make a whole point stays banked.
    pub fn burn(&mut self, seconds: u64, percent: u16, burn_seconds: u32) {
        let per_point = u64::from(burn_seconds.max(1)) * 100;
        let banked = self
            .banked
            .saturating_add(seconds.saturating_mul(u64::from(percent)));
        let points = i32::try_from(banked / per_point).unwrap_or(i32::MAX);
        self.banked = banked % per_point;
        self.nutrition = self.nutrition.saturating_sub(points);
    }

    /// `seconds` of starving pass: the hit points lost, `damage` for every `every` seconds, the
    /// seconds left over banked toward the next.
    pub fn starve(&mut self, seconds: u64, every: u32, damage: u16) -> i32 {
        let every = u64::from(every.max(1));
        let banked = u64::from(self.starving).saturating_add(seconds);
        self.starving = u32::try_from(banked % every).unwrap_or(0);
        i32::try_from(banked / every)
            .unwrap_or(i32::MAX)
            .saturating_mul(i32::from(damage))
    }

    /// Food eaten: `points` more nutrition, no more than `max`.
    pub fn feed(&mut self, points: i32, max: i32) {
        self.nutrition = self.nutrition.saturating_add(points).min(max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn states_run_from_the_hungriest_up() {
        assert!(Hunger::Starving < Hunger::Fainting && Hunger::Satiated < Hunger::Bloated);
        let names: Vec<&str> = Hunger::ALL.iter().map(|h| h.name()).collect();
        assert_eq!(
            names,
            [
                "Bloated",
                "Satiated",
                "Not hungry",
                "Hungry",
                "Weak",
                "Fainting",
                "Starving"
            ]
        );
        assert_eq!(Hunger::NotHungry.id(), "not_hungry");
        let weak: Vec<bool> = Hunger::ALL.iter().map(|h| h.is_weak()).collect();
        assert_eq!(weak, [false, false, false, false, true, true, true]);
        let faint: Vec<bool> = Hunger::ALL.iter().map(|h| h.faints()).collect();
        assert_eq!(faint, [false, false, false, false, false, true, true]);
    }

    #[test]
    fn news_says_which_way_it_went() {
        use Hunger::*;
        assert_eq!(Hunger::news(Hungry, Hungry), None);
        assert_eq!(
            Hunger::news(NotHungry, Hungry),
            Some("You are getting hungry.")
        );
        assert_eq!(
            Hunger::news(Weak, Hungry),
            Some("You feel less weak."),
            "fed back up"
        );
        assert_eq!(Hunger::news(Hungry, Satiated), Some("You feel satiated."));
        assert_eq!(Hunger::news(Fainting, Starving), Some("You are starving!"));
        assert_eq!(
            Hunger::news(Satiated, NotHungry),
            Some("You no longer feel satiated.")
        );
        // Only getting hungrier, into Hungry or worse, stops a rest.
        assert!(Hunger::stops_rest(NotHungry, Hungry));
        assert!(Hunger::stops_rest(Weak, Fainting));
        assert!(Hunger::stops_rest(Bloated, Weak));
        assert!(!Hunger::stops_rest(Bloated, Satiated));
        assert!(!Hunger::stops_rest(Satiated, NotHungry));
        assert!(!Hunger::stops_rest(Weak, Hungry));
    }

    #[test]
    fn game_time_burns_by_the_race_and_banks_the_rest() {
        let mut s = Satiation::new(1000);
        // A human: a point a minute.
        s.burn(59, 100, 60);
        assert_eq!((s.nutrition, s.banked), (1000, 5900));
        s.burn(1, 100, 60);
        assert_eq!((s.nutrition, s.banked), (999, 0));
        s.burn(3600, 100, 60);
        assert_eq!(s.nutrition, 939);
        // A halfling (125 %): five points in four minutes.
        let mut h = Satiation::new(1000);
        h.burn(240, 125, 60);
        assert_eq!((h.nutrition, h.banked), (995, 0));
        // An elf (80 %): four points in five minutes, whatever the steps.
        let mut e = Satiation::new(1000);
        for _ in 0..50 {
            e.burn(6, 80, 60);
        }
        assert_eq!((e.nutrition, e.banked), (996, 0));
        // It runs below 0.
        let mut n = Satiation::new(0);
        n.burn(120, 100, 60);
        assert_eq!(n.nutrition, -2);
    }

    #[test]
    fn starving_takes_hit_points_by_the_clock() {
        let mut s = Satiation::new(-10);
        assert_eq!(s.starve(599, 600, 1), 0);
        assert_eq!(s.starve(1, 600, 1), 1);
        assert_eq!(s.starving, 0);
        assert_eq!(s.starve(1500, 600, 2), 4);
        assert_eq!(s.starving, 300);
        let mut fed = Satiation::new(5900);
        fed.feed(500, 6000);
        assert_eq!(fed.nutrition, 6000, "never above the most");
    }
}
