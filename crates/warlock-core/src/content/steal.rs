//! Stealing (`assets/data/steal.toml`, spec §8 "Stealing"): what becomes of a creature that
//! catches the player stealing from it.
//!
//! The rule itself lives in `world/steal.rs`: who can steal (anyone trained in a skill with the
//! `steal` benefit, `data/skills.toml`), from whom (a creature next to them that is not hostile),
//! the contest (their Dexterity and Steal against its Willpower, spec §8 "Checks"), and what is
//! taken. A creature that catches them joins the faction named here and turns on them; a
//! shopkeeper is outraged as a robbed one is (`data/shops.toml`'s `outraged`, spec §7 "Shops").
//!
//! The file is optional: without it nobody steals, and the Steal skill stays a skill whose use
//! is not in the game yet (the abilities menu's "not usable yet").

use super::{ContentError, FactionId, PLAYER_FACTION, Stance};
use serde::Deserialize;

/// The rules of stealing that live in data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StealRules {
    /// The faction a creature that catches the player stealing joins (its `faction_override`):
    /// one hostile to the player, so it turns on them.
    pub caught: FactionId,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StealFile {
    caught: String,
}

impl StealRules {
    /// Parses `data/steal.toml`: `caught` must be a faction hostile to the player.
    pub(super) fn parse(
        file: &str,
        text: &str,
        factions: &super::Factions,
    ) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: StealFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let caught = (factions.id(&parsed.caught))
            .ok_or_else(|| err(format!("unknown faction `{}`", parsed.caught)))?;
        let player = factions.id(PLAYER_FACTION);
        if player.is_none_or(|p| factions.stance(caught, p) != Stance::Hostile) {
            return Err(err(format!(
                "`caught`: the faction `{}` must be hostile to the player",
                parsed.caught
            )));
        }
        Ok(Self { caught })
    }
}

#[cfg(test)]
mod tests {
    use crate::content::{Content, ContentFiles};

    #[test]
    fn a_caught_thief_faces_a_faction_hostile_to_them() {
        let content = Content::builtin().expect("content");
        let rules = content.steal().expect("stealing is in the game");
        let factions = content.factions();
        assert_eq!(factions.def(rules.caught).id, "outraged");
        // A faction that would not mind, or none at all, is refused.
        for (caught, why) in [
            ("peaceful", "must be hostile to the player"),
            ("nobody", "unknown faction `nobody`"),
        ] {
            let mut files = ContentFiles::builtin();
            files.insert("data/steal.toml", &format!("caught = \"{caught}\"\n"));
            let err = Content::load(&files).expect_err(caught);
            assert!(err.message.contains(why), "{caught}: {}", err.message);
        }
        // Without the file nobody steals.
        let mut files = ContentFiles::builtin();
        files.remove("data/steal.toml");
        let content = Content::load(&files).expect("stealing is optional");
        assert!(content.steal().is_none());
    }
}
