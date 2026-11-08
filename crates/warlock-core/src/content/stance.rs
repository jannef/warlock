//! Combat stances (`assets/data/stances.toml`, spec §8 "Stances"): how the player holds
//! themselves in a fight, as in ADOM. A stance trades to-hit and damage against Defence, and the
//! coward's trades speed for flight. Every number is data, tuned by playtesting, and none is
//! shown to the player: only the stance's name is.
//!
//! The file has at most one entry for each [`CombatStance`]; a stance it leaves out (and every
//! stance without the file) is plain (no effect at all). The frenzy is never chosen: only a berserker's rage puts them in it,
//! and its end puts them in the aggressive stance (`world/frenzy.rs`).

use super::ContentError;
use crate::combat::{Category, Profile};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A combat stance. The player chooses one of [`CombatStance::CHOSEN`] (`+` and `-`); a raging
/// berserker is [`CombatStance::Frenzy`], which nothing else gives; creatures stay
/// [`CombatStance::Neutral`].
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum CombatStance {
    Aggressive,
    #[default]
    Neutral,
    Defensive,
    Coward,
    /// A raging berserker's, and only theirs (`world/frenzy.rs`).
    Frenzy,
}

impl CombatStance {
    /// Every stance, from the most defensive to the most aggressive.
    pub const ALL: [CombatStance; 5] = [
        CombatStance::Coward,
        CombatStance::Defensive,
        CombatStance::Neutral,
        CombatStance::Aggressive,
        CombatStance::Frenzy,
    ];

    /// The stances the player may choose (`+` and `-` step through them), from the most
    /// defensive to the most aggressive: every one but the frenzy.
    pub const CHOSEN: [CombatStance; 4] = [
        CombatStance::Coward,
        CombatStance::Defensive,
        CombatStance::Neutral,
        CombatStance::Aggressive,
    ];

    /// Its id: `aggressive`.
    pub const fn id(self) -> &'static str {
        match self {
            CombatStance::Aggressive => "aggressive",
            CombatStance::Neutral => "neutral",
            CombatStance::Defensive => "defensive",
            CombatStance::Coward => "coward",
            CombatStance::Frenzy => "frenzy",
        }
    }

    fn index(self) -> usize {
        Self::ALL.iter().position(|&s| s == self).unwrap_or(2)
    }

    /// One step more aggressive among the chosen stances (Coward, Defensive, Neutral,
    /// Aggressive); Aggressive stays, and so does Frenzy (rage holds it there).
    pub fn more_aggressive(self) -> Self {
        match Self::CHOSEN.iter().position(|&s| s == self) {
            Some(i) => Self::CHOSEN[(i + 1).min(Self::CHOSEN.len() - 1)],
            None => self,
        }
    }

    /// One step more defensive; Coward stays, and so does Frenzy.
    pub fn more_defensive(self) -> Self {
        match Self::CHOSEN.iter().position(|&s| s == self) {
            Some(i) => Self::CHOSEN[i.saturating_sub(1)],
            None => self,
        }
    }

    /// What the player says they take: "an aggressive", "a cowardly" (then "stance").
    pub const fn phrase(self) -> &'static str {
        match self {
            CombatStance::Aggressive => "an aggressive",
            CombatStance::Neutral => "a neutral",
            CombatStance::Defensive => "a defensive",
            CombatStance::Coward => "a cowardly",
            CombatStance::Frenzy => "a frenzied",
        }
    }

    pub fn is_neutral(&self) -> bool {
        *self == CombatStance::Neutral
    }
}

impl fmt::Display for CombatStance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}

impl FromStr for CombatStance {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();
        Self::ALL.into_iter().find(|c| c.id() == s).ok_or_else(|| {
            "`stance` needs aggressive, neutral, defensive, or coward (frenzy comes only of \
             rage), e.g. `stance:coward`"
                .into()
        })
    }
}

fn hundred() -> u32 {
    100
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StanceFile {
    stance: Vec<StanceDef>,
}

/// What a stance does, in the file's own fields.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StanceDef {
    pub id: CombatStance,
    /// Its name, as the status bar and the wiki show it ("Aggressive").
    pub name: String,
    /// Added to the to-hit of the stanced actor's attacks.
    #[serde(default)]
    pub to_hit: i32,
    /// Added to every roll of the damage of its attacks.
    #[serde(default)]
    pub damage: i32,
    /// Added to its Defence in the Dodge category (bonuses stack there; may be negative).
    #[serde(default)]
    pub dodge: i32,
    /// Added to its Defence in the Parry category (only the highest bonus of the category counts,
    /// so it matters when it beats the weapon's own parry).
    #[serde(default)]
    pub parry: i32,
    /// What its attacks (melee, shots, throws) cost, in percent.
    #[serde(default = "hundred")]
    pub attack_cost: u32,
    /// What a step costs while fleeing (stepping away from adjacent hostiles), in percent.
    #[serde(default = "hundred")]
    pub flee_cost: u32,
    /// How loud a step is while fleeing, in percent.
    #[serde(default = "hundred")]
    pub flee_noise: u32,
    /// Weapon skill marks count this much (percent): the hook for weapon skills (#134).
    #[serde(default = "hundred")]
    pub weapon_mark: u32,
    /// The chance of a gatekeeper's blow by chance (the Gatekeeper gift) counts this much
    /// (percent), at most always (`world/free_blow.rs::gatekeeper_odds`).
    #[serde(default = "hundred")]
    pub gatekeeper: u32,
    /// Taken off a miscast's total on the miscast table (spec §8 "Spells": "Miscasts";
    /// `world/miscast.rs`): a calm stance softens a miscast, a wild one worsens it.
    #[serde(default)]
    pub miscast: i32,
}

impl StanceDef {
    /// A stance with no effect.
    pub fn plain(id: CombatStance) -> Self {
        let mut name = id.id().to_string();
        name[..1].make_ascii_uppercase();
        Self {
            id,
            name,
            to_hit: 0,
            damage: 0,
            dodge: 0,
            parry: 0,
            attack_cost: 100,
            flee_cost: 100,
            flee_noise: 100,
            weapon_mark: 100,
            gatekeeper: 100,
            miscast: 0,
        }
    }

    /// `profile` with the stance's to-hit and damage.
    pub fn apply(&self, profile: Profile) -> Profile {
        Profile {
            to_hit: profile.to_hit + self.to_hit,
            bonus: profile.bonus + self.damage,
            ..profile
        }
    }

    /// The Defence bonuses it adds (none that are zero).
    pub fn defence(&self) -> impl Iterator<Item = (Category, i32)> {
        [(Category::Dodge, self.dodge), (Category::Parry, self.parry)]
            .into_iter()
            .filter(|&(_, bonus)| bonus != 0)
    }

    /// `marks` weapon skill marks as this stance counts them (the hook for #134: nothing counts
    /// marks yet).
    pub fn marks(&self, marks: i32) -> i32 {
        marks * i32::try_from(self.weapon_mark).unwrap_or(100) / 100
    }
}

/// `value` scaled by `percent`, at least 1.
pub fn percent_of(value: i64, percent: u32) -> i64 {
    (value * i64::from(percent) / 100).max(1)
}

/// The stances, indexed by [`CombatStance`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stances([StanceDef; CombatStance::ALL.len()]);

impl Default for Stances {
    fn default() -> Self {
        Self(CombatStance::ALL.map(StanceDef::plain))
    }
}

impl Stances {
    /// The definition of `stance`.
    pub fn get(&self, stance: CombatStance) -> &StanceDef {
        &self.0[stance.index()]
    }

    /// Every stance, in [`CombatStance::ALL`] order.
    pub fn all(&self) -> &[StanceDef] {
        &self.0
    }

    /// Parses `data/stances.toml`: at most one entry for each stance; one left out is plain.
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: StanceFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let mut stances = Self::default();
        let mut seen = Vec::new();
        for def in parsed.stance {
            let id = def.id;
            if seen.contains(&id) {
                return Err(err(format!("duplicate stance `{id}`")));
            }
            seen.push(id);
            stances.0[id.index()] = def;
        }
        Ok(stances)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NEUTRAL: &str = "[[stance]]\nid = \"neutral\"\nname = \"Neutral\"\n";

    fn all() -> String {
        let rest = ["aggressive", "defensive", "coward", "frenzy"]
            .map(|id| format!("[[stance]]\nid = \"{id}\"\nname = \"{id}\"\nto_hit = 1\n"));
        format!("{NEUTRAL}{}", rest.join(""))
    }

    #[test]
    fn stances_parse_with_defaults() {
        let stances = Stances::parse("stances.toml", &all()).expect("valid");
        let neutral = stances.get(CombatStance::Neutral);
        assert_eq!(neutral, &StanceDef::plain(CombatStance::Neutral));
        let aggressive = stances.get(CombatStance::Aggressive);
        assert_eq!((aggressive.to_hit, aggressive.attack_cost), (1, 100));
    }

    #[test]
    fn bad_stances_are_refused() {
        for (text, why) in [
            (format!("{}{NEUTRAL}", all()), "duplicate stance `neutral`"),
            (
                all().replace("to_hit = 1", "speed = 3"),
                "unknown field `speed`",
            ),
            (
                all().replace("\"coward\"", "\"sneak\""),
                "unknown variant `sneak`",
            ),
        ] {
            let e = Stances::parse("stances.toml", &text).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
    }

    #[test]
    fn stances_cycle_and_parse() {
        use CombatStance::*;
        let mut stance = Coward;
        let mut up = vec![stance];
        for _ in 0..4 {
            stance = stance.more_aggressive();
            up.push(stance);
        }
        assert_eq!(up, [Coward, Defensive, Neutral, Aggressive, Aggressive]);
        let mut down = vec![stance];
        for _ in 0..4 {
            stance = stance.more_defensive();
            down.push(stance);
        }
        assert_eq!(down, [Aggressive, Neutral, Defensive, Coward, Coward]);
        assert_eq!(Frenzy.more_aggressive(), Frenzy, "rage holds it");
        assert_eq!(Frenzy.more_defensive(), Frenzy);
        assert_eq!(
            "frenzy".parse(),
            Ok(Frenzy),
            "parsed, for the world to refuse"
        );
        assert_eq!("Coward".parse(), Ok(CombatStance::Coward));
        assert!("brave".parse::<CombatStance>().is_err());
    }
}
