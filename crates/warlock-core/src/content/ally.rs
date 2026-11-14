//! Allies (`assets/data/allies.toml`, spec §8 "Allies"; docs/design/druid-and-summons.md §4,
//! §7.1, §14): the constants of the ally core. An ally is a creature on the player's side; the
//! rules live in `world/ally.rs`. The file is optional: without it no ally can be made, and
//! nothing here is read.

use super::ContentError;
use serde::Deserialize;

/// The numbers of the ally core.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllyRules {
    /// A following ally stays within this many cells of its master.
    pub heel: i32,
    /// Farther than this from its master, a following ally breaks off a fight and heels.
    pub leash: i32,
    /// Allies within this many cells (Chebyshev) of the player come along to another level.
    pub comes_along: i32,
    /// A holding ally fights only what comes within this many cells of its post.
    pub stay_reach: i32,
    /// A set-on ally fights what is within this many cells of the cell it was sent to.
    pub hunt_reach: i32,
    /// An order's loudness, where the player stands.
    pub order_noise: u16,
    /// The player's allies on one level, at most, whatever made them.
    pub most: u8,
    /// What an ally's kill pays its master, in percent of the master's own kill.
    pub ally_percent: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AllyFile {
    follow: Follow,
    stay: Reach,
    hunt: Reach,
    orders: Orders,
    allies: Most,
    xp: Xp,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Follow {
    heel: i32,
    leash: i32,
    comes_along: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reach {
    reach: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Orders {
    noise: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Most {
    most: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Xp {
    ally_percent: u8,
}

impl AllyRules {
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: AllyFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        Ok(AllyRules {
            heel: parsed.follow.heel,
            leash: parsed.follow.leash,
            comes_along: parsed.follow.comes_along,
            stay_reach: parsed.stay.reach,
            hunt_reach: parsed.hunt.reach,
            order_noise: parsed.orders.noise,
            most: parsed.allies.most,
            ally_percent: parsed.xp.ally_percent,
        })
    }
}

impl super::RulesFile for AllyRules {
    const PATH: &'static str = "data/allies.toml";

    fn parse(path: &str, text: &str, _content: &super::Content) -> Result<Self, ContentError> {
        Self::parse(path, text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = "[follow]\nheel = 2\nleash = 8\ncomes_along = 3\n[stay]\nreach = 1\n\
                        [hunt]\nreach = 3\n[orders]\nnoise = 60\n[allies]\nmost = 8\n\
                        [xp]\nally_percent = 100\n";

    #[test]
    fn the_file_parses_to_its_numbers() {
        let rules = AllyRules::parse("allies.toml", FILE).expect("valid");
        assert_eq!(
            rules,
            AllyRules {
                heel: 2,
                leash: 8,
                comes_along: 3,
                stay_reach: 1,
                hunt_reach: 3,
                order_noise: 60,
                most: 8,
                ally_percent: 100,
            }
        );
    }

    #[test]
    fn the_shipped_file_is_valid() {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/data/allies.toml"
        ))
        .expect("the shipped file");
        AllyRules::parse("allies.toml", &text).expect("valid");
    }

    #[test]
    fn a_missing_field_is_refused() {
        let text = FILE.replace("[xp]\nally_percent = 100\n", "");
        let err = AllyRules::parse("allies.toml", &text)
            .expect_err("refused")
            .to_string();
        assert!(err.contains("missing field"), "{err}");
    }
}
