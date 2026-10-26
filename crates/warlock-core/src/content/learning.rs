//! Learning spells as data (spec §8 "Spells", the magic design §4): the study check and its
//! time (`data/study.toml`, [`StudyRules`]) and the mishaps a failed study draws
//! (`data/mishaps/`, one `[[mishap]]` file each, [`Mishap`]). The rules are `world/learn.rs`.
//!
//! ```toml
//! # data/study.toml
//! base = 12             # the difficulty at overreach 0, before what is known and the bonuses
//! per_overreach = 4     # harder per tier of overreach
//! known_per_point = 10  # tiers known (a cantrip counts 1) per point harder
//! turns_per_tier = 100  # turns of the `study` action per tier + 1 (10 minutes each at tier 0)
//! bad_from = 2          # the overreach a failure's mishap is bad from (slight below)
//! terrible_from = 4     # and terrible from
//! easy = 96             # ways of 144 at least this many: "easy"
//! struggle = 36         # at least this many: "a struggle"; fewer: "far beyond you"
//! ```
//!
//! ```toml
//! # data/mishaps/fizzle.toml
//! [[mishap]]
//! id = "fizzle"
//! band = "slight"       # slight, bad, terrible, or lucky (drawn beside any band)
//! weight = 40           # its share of its band's draw, 1 to 1000
//! message = "The words crumble to dust as you read them."
//! effect = { kind = "nothing" }
//! ```
//!
//! Effects: `nothing`; `status` (`status`, `severity` 1 to its max); `backfire` (the scroll's
//! own spell goes off on the reader); `mana` (`percent` of the mana left, 1 to 100, is lost);
//! `teleport` (to a free cell of the level); `lose_mark` (one stat training mark, of a stat
//! with any, is lost); `drain` (`stat` or `"random"`, `points` 1 to 3, `difficulty`: a
//! Willpower check or the points are lost for good); `summon` (`creature`, hostile, next to the
//! reader); `learn` (the spell is learned anyway); `mark` (`stat`: a training mark more).

use super::{Content, ContentError, CreatureId, StatusId, duplicate_id};
use crate::stats::Stat;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The heaviest a mishap's weight may be.
pub const MAX_WEIGHT: u16 = 1000;
/// The most points a mishap's drain may take.
pub const MAX_DRAIN: u8 = 3;

/// The study check and its time (`data/study.toml`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StudyRules {
    pub base: i32,
    pub per_overreach: i32,
    /// Tiers known per point harder (at least 1).
    pub known_per_point: i32,
    /// Turns of the `study` action per tier + 1.
    pub turns_per_tier: u32,
    /// The overreach a failure's mishap is bad from, and terrible from.
    pub bad_from: i32,
    pub terrible_from: i32,
    /// The ways of 144 a study is "easy" from, and "a struggle" from.
    pub easy: u32,
    pub struggle: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StudyFile {
    base: i32,
    per_overreach: i32,
    known_per_point: i32,
    turns_per_tier: u32,
    bad_from: i32,
    terrible_from: i32,
    easy: u32,
    struggle: u32,
}

impl StudyRules {
    pub(super) fn parse(path: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: &str| ContentError::new(path, m.to_string());
        let f: StudyFile =
            toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))?;
        if !(2..=40).contains(&f.base) || !(0..=20).contains(&f.per_overreach) {
            return Err(err("`base` is 2 to 40 and `per_overreach` 0 to 20"));
        }
        if !(1..=100).contains(&f.known_per_point) {
            return Err(err("`known_per_point` is 1 to 100"));
        }
        if !(1..=1000).contains(&f.turns_per_tier) {
            return Err(err("`turns_per_tier` is 1 to 1000"));
        }
        if !(1..f.terrible_from).contains(&f.bad_from) || f.terrible_from > 9 {
            return Err(err(
                "`bad_from` is at least 1 and below `terrible_from`, at most 9",
            ));
        }
        if f.struggle == 0 || f.struggle >= f.easy || f.easy > 144 {
            return Err(err("`struggle` is above 0 and below `easy`, at most 144"));
        }
        Ok(Self {
            base: f.base,
            per_overreach: f.per_overreach,
            known_per_point: f.known_per_point,
            turns_per_tier: f.turns_per_tier,
            bad_from: f.bad_from,
            terrible_from: f.terrible_from,
            easy: f.easy,
            struggle: f.struggle,
        })
    }

    /// The study check's difficulty at `overreach` with `known` tiers known (a cantrip
    /// counting 1), before the bonuses: `base + per_overreach × overreach + known ÷
    /// known_per_point`.
    pub fn difficulty(&self, overreach: i32, known: i32) -> i32 {
        self.base + self.per_overreach * overreach + known / self.known_per_point
    }

    /// The band of a failure at `overreach` (1 or more).
    pub fn band(&self, overreach: i32) -> Band {
        if overreach >= self.terrible_from {
            Band::Terrible
        } else if overreach >= self.bad_from {
            Band::Bad
        } else {
            Band::Slight
        }
    }

    /// The chance of `ways` of 144 in words, as the game tells it.
    pub fn words(&self, ways: u32) -> &'static str {
        if ways >= 144 {
            "certain"
        } else if ways >= self.easy {
            "easy"
        } else if ways >= self.struggle {
            "a struggle"
        } else {
            "far beyond you"
        }
    }
}

/// How bad a mishap is: by the overreach of the failed study, or a lucky one beside any.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Band {
    Slight,
    Bad,
    Terrible,
    Lucky,
}

impl Band {
    pub const ALL: [Band; 4] = [Band::Slight, Band::Bad, Band::Terrible, Band::Lucky];

    pub const fn id(self) -> &'static str {
        match self {
            Band::Slight => "slight",
            Band::Bad => "bad",
            Band::Terrible => "terrible",
            Band::Lucky => "lucky",
        }
    }
}

/// What a mishap does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MishapEffect {
    Nothing,
    Status {
        status: StatusId,
        severity: u16,
    },
    Backfire,
    Mana {
        percent: u8,
    },
    Teleport,
    LoseMark,
    Drain {
        stat: Option<Stat>,
        points: u8,
        difficulty: i32,
    },
    Summon {
        creature: CreatureId,
    },
    Learn,
    Mark {
        stat: Stat,
    },
}

impl MishapEffect {
    /// Its kind's id, as data writes it.
    pub const fn kind(&self) -> &'static str {
        match self {
            MishapEffect::Nothing => "nothing",
            MishapEffect::Status { .. } => "status",
            MishapEffect::Backfire => "backfire",
            MishapEffect::Mana { .. } => "mana",
            MishapEffect::Teleport => "teleport",
            MishapEffect::LoseMark => "lose_mark",
            MishapEffect::Drain { .. } => "drain",
            MishapEffect::Summon { .. } => "summon",
            MishapEffect::Learn => "learn",
            MishapEffect::Mark { .. } => "mark",
        }
    }
}

/// A mishap (`data/mishaps/`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mishap {
    pub id: String,
    pub band: Band,
    pub weight: u16,
    /// Said as it happens.
    pub message: String,
    pub effect: MishapEffect,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MishapFile {
    #[serde(default)]
    mishap: Vec<MishapSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MishapSpec {
    id: String,
    band: Band,
    weight: u16,
    message: String,
    effect: EffectSpec,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EffectSpec {
    kind: String,
    status: Option<String>,
    severity: Option<u16>,
    percent: Option<u8>,
    stat: Option<String>,
    points: Option<u8>,
    difficulty: Option<i32>,
    creature: Option<String>,
}

/// Parses the mishap files, in path order; statuses and creatures must be loaded.
pub(super) fn parse<'a>(
    files: impl Iterator<Item = (&'a str, &'a str)>,
    content: &Content,
) -> Result<Vec<Mishap>, ContentError> {
    let mut all: Vec<Mishap> = Vec::new();
    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    for (path, text) in files {
        let file: MishapFile =
            toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))?;
        for spec in file.mishap {
            if let Some(first) = seen.get(&spec.id) {
                return Err(duplicate_id("mishap", &spec.id, path, first));
            }
            seen.insert(spec.id.clone(), path.to_string());
            let id = spec.id.clone();
            let mishap = resolve(spec, content)
                .map_err(|m| ContentError::new(path, format!("mishap `{id}`: {m}")))?;
            all.push(mishap);
        }
    }
    Ok(all)
}

fn resolve(spec: MishapSpec, content: &Content) -> Result<Mishap, String> {
    if !(1..=MAX_WEIGHT).contains(&spec.weight) {
        return Err(format!("`weight` is 1 to {MAX_WEIGHT}"));
    }
    if spec.message.trim().is_empty() {
        return Err("`message` is empty".into());
    }
    let e = spec.effect;
    // Only the fields its kind reads.
    let fields = [
        ("status", e.status.is_some()),
        ("severity", e.severity.is_some()),
        ("percent", e.percent.is_some()),
        ("stat", e.stat.is_some()),
        ("points", e.points.is_some()),
        ("difficulty", e.difficulty.is_some()),
        ("creature", e.creature.is_some()),
    ];
    let reads: &[&str] = match e.kind.as_str() {
        "status" => &["status", "severity"],
        "mana" => &["percent"],
        "drain" => &["stat", "points", "difficulty"],
        "summon" => &["creature"],
        "mark" => &["stat"],
        _ => &[],
    };
    if let Some((name, _)) = fields.iter().find(|(n, set)| *set && !reads.contains(n)) {
        return Err(format!("a `{}` effect has no `{name}`", e.kind));
    }
    let stat = |text: &Option<String>, random: bool| -> Result<Option<Stat>, String> {
        match text.as_deref() {
            Some("random") if random => Ok(None),
            Some(id) => Stat::from_id(id)
                .map(Some)
                .ok_or_else(|| format!("unknown stat `{id}`")),
            None => Err("it needs a `stat`".into()),
        }
    };
    let effect = match e.kind.as_str() {
        "nothing" => MishapEffect::Nothing,
        "backfire" => MishapEffect::Backfire,
        "teleport" => MishapEffect::Teleport,
        "lose_mark" => MishapEffect::LoseMark,
        "learn" => MishapEffect::Learn,
        "status" => {
            let name = e.status.ok_or("a `status` effect needs a `status`")?;
            let status = (content.status_id(&name)).ok_or(format!("unknown status `{name}`"))?;
            let severity = e.severity.unwrap_or(1);
            let max = content.status(status).max;
            if severity < 1 || severity > max {
                return Err(format!("`severity` is 1 to the status's max, {max}"));
            }
            MishapEffect::Status { status, severity }
        }
        "mana" => {
            let percent = e.percent.ok_or("a `mana` effect needs a `percent`")?;
            if !(1..=100).contains(&percent) {
                return Err("`percent` is 1 to 100".into());
            }
            MishapEffect::Mana { percent }
        }
        "drain" => {
            let points = e.points.ok_or("a `drain` effect needs `points`")?;
            if !(1..=MAX_DRAIN).contains(&points) {
                return Err(format!("`points` is 1 to {MAX_DRAIN}"));
            }
            let difficulty = e
                .difficulty
                .ok_or("a `drain` effect needs a `difficulty`")?;
            if !(2..=40).contains(&difficulty) {
                return Err("`difficulty` is 2 to 40".into());
            }
            MishapEffect::Drain {
                stat: stat(&e.stat, true)?,
                points,
                difficulty,
            }
        }
        "summon" => {
            let name = e.creature.ok_or("a `summon` effect needs a `creature`")?;
            let creature =
                (content.creature_id(&name)).ok_or(format!("unknown creature `{name}`"))?;
            MishapEffect::Summon { creature }
        }
        "mark" => MishapEffect::Mark {
            stat: stat(&e.stat, false)?.expect("not random"),
        },
        other => return Err(format!("unknown effect kind `{other}`")),
    };
    let lucky = matches!(effect, MishapEffect::Learn | MishapEffect::Mark { .. });
    if lucky != (spec.band == Band::Lucky) && !matches!(effect, MishapEffect::Status { .. }) {
        return Err(
            "`learn` and `mark` are lucky, and a lucky mishap is `learn`, `mark`, or a status"
                .into(),
        );
    }
    Ok(Mishap {
        id: spec.id,
        band: spec.band,
        weight: spec.weight,
        message: spec.message,
        effect,
    })
}

/// The mishaps a failure of `band` draws among: the band's and the lucky ones, in order.
pub fn drawn_among(mishaps: &[Mishap], band: Band) -> impl Iterator<Item = &Mishap> {
    mishaps
        .iter()
        .filter(move |m| m.band == band || m.band == Band::Lucky)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RULES: &str = "base = 12\nper_overreach = 4\nknown_per_point = 10\n\
        turns_per_tier = 100\nbad_from = 2\nterrible_from = 4\neasy = 96\nstruggle = 36\n";

    #[test]
    fn the_rules_parse_and_say_the_difficulty_band_and_words() {
        let r = StudyRules::parse("study.toml", RULES).expect("valid");
        assert_eq!(r.difficulty(1, 0), 16);
        assert_eq!(r.difficulty(2, 25), 22, "25 tiers known: 2 points harder");
        assert_eq!(r.band(1), Band::Slight);
        assert_eq!(r.band(3), Band::Bad);
        assert_eq!(r.band(4), Band::Terrible);
        assert_eq!(r.words(144), "certain");
        assert_eq!(r.words(96), "easy");
        assert_eq!(r.words(40), "a struggle");
        assert_eq!(r.words(35), "far beyond you");
    }

    #[test]
    fn bad_rules_are_refused() {
        for bad in [
            RULES.replace("bad_from = 2", "bad_from = 4"),
            RULES.replace("struggle = 36", "struggle = 100"),
            RULES.replace("known_per_point = 10", "known_per_point = 0"),
            RULES.replace("turns_per_tier = 100", "turns_per_tier = 0"),
        ] {
            assert!(StudyRules::parse("study.toml", &bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn the_shipped_mishaps_cover_every_band() {
        let content = Content::builtin().expect("content");
        let mishaps = content.mishaps();
        for band in Band::ALL {
            assert!(mishaps.iter().any(|m| m.band == band), "{band:?}");
        }
        assert!(content.study_rules().is_some());
    }

    fn one(effect: &str, band: &str) -> Result<Vec<Mishap>, ContentError> {
        let content = Content::builtin().expect("content");
        let text = format!(
            "[[mishap]]\nid = \"x\"\nband = \"{band}\"\nweight = 1\nmessage = \"Oops.\"\n\
             effect = {effect}\n"
        );
        parse(
            [("data/mishaps/x.toml", text.as_str())].into_iter(),
            &content,
        )
    }

    #[test]
    fn malformed_mishaps_are_refused() {
        assert!(one("{ kind = \"nothing\" }", "slight").is_ok());
        assert!(
            one(
                "{ kind = \"status\", status = \"sluggish\", severity = 2 }",
                "bad"
            )
            .is_ok()
        );
        for (effect, band) in [
            ("{ kind = \"boom\" }", "slight"),
            ("{ kind = \"status\", status = \"nope\" }", "slight"),
            (
                "{ kind = \"status\", status = \"sluggish\", severity = 9 }",
                "slight",
            ),
            ("{ kind = \"nothing\", percent = 5 }", "slight"),
            ("{ kind = \"mana\", percent = 0 }", "bad"),
            (
                "{ kind = \"drain\", stat = \"random\", points = 9, difficulty = 15 }",
                "terrible",
            ),
            ("{ kind = \"summon\", creature = \"nobody\" }", "terrible"),
            ("{ kind = \"learn\" }", "slight"),
            ("{ kind = \"nothing\" }", "lucky"),
        ] {
            assert!(one(effect, band).is_err(), "{effect} {band}");
        }
    }
}
