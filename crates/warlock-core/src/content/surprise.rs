//! Surprise (`assets/data/surprise.toml`, spec §8 "Surprise and sneak attacks"): the numbers of
//! being caught unawares.
//!
//! The rule itself lives in `world/surprise.rs`: who is unaware of whom, the initiative check,
//! and the status a failed check brings. The file is optional: without it nobody is surprised
//! (fixtures).

use super::{Content, ContentError, CreatureId, StatusId};
use serde::Deserialize;

/// The rules of surprise that live in data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurpriseRules {
    /// The status an actor caught unawares suffers when it fails its initiative check (and a
    /// sleeper or a disguised mimic with no check): `per_action`, one degree (Surprised).
    pub status: StatusId,
    /// The initiative check's difficulty before the attacker's craft.
    pub difficulty: i32,
    /// How far a shot, a throw, or an attack spell is still a sneak attack, in cells
    /// (`World::sneaks`).
    pub missile_reach: i32,
    /// How many turns the player stays aware of a creature they have seen.
    pub memory_turns: u64,
    /// The tags of what has no vitals to find: immune to criticals and sneak attacks
    /// ([`Content::has_vitals`]).
    pub no_vitals: Vec<String>,
    /// Slipping past (`[slip]`, `world/slip.rs`): how many turns a creature the player slipped
    /// past stays blind to them; `None` when nobody slips past anyone (no table, or `enabled =
    /// false`).
    pub slip_turns: Option<u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SurpriseFile {
    surprise: SurpriseTable,
    #[serde(default)]
    slip: Option<SlipTable>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SlipTable {
    enabled: bool,
    lasts_turns: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SurpriseTable {
    status: String,
    difficulty: i32,
    missile_reach: i32,
    memory_turns: u64,
    #[serde(default)]
    no_vitals: Vec<String>,
}

impl SurpriseRules {
    /// Parses `data/surprise.toml`: the status must be `per_action` with one degree, the numbers
    /// in range, and each tag carried by some creature type.
    pub(super) fn parse(file: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: SurpriseFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let table = &parsed.surprise;
        let status = (content.status_id(&table.status))
            .ok_or_else(|| err(format!("unknown status `{}`", table.status)))?;
        let def = content.status(status);
        if !def.per_action || def.max != 1 {
            return Err(err(format!(
                "`status`: `{}` must be `per_action` with `max = 1`",
                table.status
            )));
        }
        if !(1..=40).contains(&table.difficulty) {
            return Err(err("`difficulty` is 1 to 40".into()));
        }
        if !(1..=20).contains(&table.missile_reach) {
            return Err(err("`missile_reach` is 1 to 20".into()));
        }
        if !(1..=1000).contains(&table.memory_turns) {
            return Err(err("`memory_turns` is 1 to 1000".into()));
        }
        for tag in &table.no_vitals {
            super::tag::check(tag).map_err(|m| err(format!("`no_vitals`: {m}")))?;
            if content.tagged(tag).next().is_none() {
                return Err(err(format!(
                    "`no_vitals`: tag `{tag}`: no creature carries it"
                )));
            }
        }
        if let Some(slip) = &parsed.slip
            && !(1..=1000).contains(&slip.lasts_turns)
        {
            return Err(err("`slip.lasts_turns` is 1 to 1000".into()));
        }
        Ok(Self {
            status,
            difficulty: table.difficulty,
            missile_reach: table.missile_reach,
            memory_turns: table.memory_turns,
            no_vitals: table.no_vitals.clone(),
            slip_turns: (parsed.slip).filter(|s| s.enabled).map(|s| s.lasts_turns),
        })
    }
}

impl Content {
    /// Whether creature type `id` has vitals to find for an attacker whose talents name
    /// `vitals_in` (Anatomist's tags): the one reader of what has none (spec §8 "Surprise and
    /// sneak attacks"). Without vitals it is immune to criticals and to sneak attacks: an
    /// incorporeal type always; one that says `vitals = false`; an animated object or a type
    /// carrying a `no_vitals` tag, unless it carries one of `vitals_in` too. Everything has
    /// vitals without the surprise rules (fixtures).
    pub fn has_vitals(&self, id: CreatureId, vitals_in: &[String]) -> bool {
        let Some(rules) = self.surprise() else {
            return true;
        };
        let def = self.creature(id);
        if def.incorporeal.is_some() || !def.vitals {
            return false;
        }
        if vitals_in.iter().any(|t| self.has_tag(id, t)) {
            return true;
        }
        def.animated.is_none() && !rules.no_vitals.iter().any(|t| self.has_tag(id, t))
    }
}

#[cfg(test)]
mod tests {
    use crate::content::{Content, ContentFiles};

    const PATH: &str = "data/surprise.toml";

    #[test]
    fn the_shipped_rules_parse() {
        let content = Content::builtin().expect("content");
        let rules = content.surprise().expect("surprise is in the game");
        assert_eq!(content.status(rules.status).id, "surprised");
        assert_eq!(rules.difficulty, 13);
        assert_eq!(rules.missile_reach, 6);
        assert_eq!(rules.memory_turns, 20);
        assert!(rules.no_vitals.iter().any(|t| t == "skeleton"));
        assert_eq!(rules.slip_turns, Some(60));
    }

    #[test]
    fn bad_rules_are_refused_and_the_file_is_optional() {
        let good = ContentFiles::builtin()
            .get(PATH)
            .expect("shipped")
            .to_string();
        for (from, to, why) in [
            ("\"surprised\"", "\"nothing\"", "unknown status `nothing`"),
            (
                "\"surprised\"",
                "\"bleeding\"",
                "must be `per_action` with `max = 1`",
            ),
            (
                "difficulty = 13",
                "difficulty = 0",
                "`difficulty` is 1 to 40",
            ),
            (
                "missile_reach = 6",
                "missile_reach = 99",
                "`missile_reach` is 1 to 20",
            ),
            (
                "memory_turns = 20",
                "memory_turns = 0",
                "`memory_turns` is 1 to 1000",
            ),
            (
                "\"skeleton\"",
                "\"no-such-thing\"",
                "no creature carries it",
            ),
            ("\"skeleton\"", "\"Skeleton\"", "lower-case words"),
            (
                "lasts_turns = 60",
                "lasts_turns = 0",
                "`slip.lasts_turns` is 1 to 1000",
            ),
        ] {
            assert!(good.contains(from), "{from}");
            let mut files = ContentFiles::builtin();
            files.insert(PATH, &good.replacen(from, to, 1));
            let err = Content::load(&files).expect_err(to);
            assert!(err.message.contains(why), "{to}: {}", err.message);
        }
        let mut files = ContentFiles::builtin();
        files.insert(PATH, &good.replacen("enabled = true", "enabled = false", 1));
        let content = Content::load(&files).expect("slipping past switched off");
        assert_eq!(content.surprise().expect("rules").slip_turns, None);
        let mut files = ContentFiles::builtin();
        files.remove(PATH);
        let content = Content::load(&files).expect("surprise is optional");
        assert!(content.surprise().is_none());
    }
}
