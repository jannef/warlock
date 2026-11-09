//! Learning spells as data (spec §8 "Spells", the magic design §4, the spell scaling design
//! §5.4): the study check and its
//! time (`data/study.toml`, [`StudyRules`]) and the mishaps a failed study draws
//! (`data/mishaps/`, one `[[mishap]]` file each, [`Mishap`]). The rules are `world/learn.rs`.
//!
//! ```toml
//! # data/study.toml
//! base = 12             # the difficulty at overreach 0, before what is known and the bonuses
//! per_overreach = 1     # harder per spell level of overreach
//! known_per_point = 20  # the sum of the levels of the spells known, per point harder
//! turns_base = 100      # turns of the `study` action for any spell (10 minutes)
//! turns_per_level = 20  # and more per spell level (2 minutes each)
//! bad_from = 6          # the overreach a failure's mishap is bad from (slight below)
//! terrible_from = 15    # and terrible from
//! easy = 96             # ways of 144 at least this many: "easy"
//! struggle = 36         # at least this many: "a struggle"; fewer: "far beyond you"
//! ```
//!
//! ```toml
//! # data/mishaps/fizzle.toml
//! [[mishap]]
//! id = "fizzle"
//! band = "slight"       # slight, bad, terrible, or lucky (drawn beside any band)
//! weight = 40           # its share of its band's draw
//! message = "The words crumble to dust as you read them."
//! effect = { kind = "nothing" }
//! ```
//!
//! Effects: `nothing`; `status` (`status`, `severity`, default 1); `backfire` (the scroll's
//! own spell goes off on the reader); `mana` (`percent` of the mana left is lost);
//! `teleport` (to a free cell of the level); `lose_mark` (one stat training mark, of a stat
//! with any, is lost); `drain` (`stat` or `"random"`, `points`, `difficulty`: a
//! Willpower check or the points are lost for good); `summon` (`creature`, hostile, next to the
//! reader); `learn` (the spell is learned anyway); `mark` (`stat`: a training mark more).
//!
//! The miscast table (`data/miscasts/`, `content/miscast.rs`) writes its entries' effects the
//! same way ([`resolve_effect`]), with kinds of its own: `harm` (`dice`, `damage`), `refund`,
//! `twice`, `swap`, `boon` (`status`, `severity`), `noise` (`loudness`), `zone` (`zone`,
//! `radius`, `turns`), and `backfire`'s `full`; `learn` is the study's only.

use super::{Content, ContentError, CreatureId, DamageTypeId, StatusId, ZoneKind, duplicate_id};
use crate::dice::Dice;
use crate::stats::Stat;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The study check and its time (`data/study.toml`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StudyRules {
    pub base: i32,
    /// Harder per spell level of overreach.
    pub per_overreach: i32,
    /// Levels of the spells known per point harder (at least 1).
    pub known_per_point: i32,
    /// Harder per rank already held, for a deepening (#1264).
    pub per_rank: i32,
    /// Turns of the `study` action for any spell, and more per spell level.
    pub turns_base: u32,
    pub turns_per_level: u32,
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
    per_rank: i32,
    turns_base: u32,
    turns_per_level: u32,
    bad_from: i32,
    terrible_from: i32,
    easy: u32,
    struggle: u32,
}

impl StudyRules {
    pub(super) fn parse(path: &str, text: &str) -> Result<Self, ContentError> {
        let f: StudyFile =
            toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))?;
        // A divisor of the difficulty.
        if f.known_per_point < 1 {
            let why = "`known_per_point` is at least 1";
            return Err(ContentError::new(path, why.to_string()));
        }
        Ok(Self {
            base: f.base,
            per_overreach: f.per_overreach,
            known_per_point: f.known_per_point,
            per_rank: f.per_rank,
            turns_base: f.turns_base,
            turns_per_level: f.turns_per_level,
            bad_from: f.bad_from,
            terrible_from: f.terrible_from,
            easy: f.easy,
            struggle: f.struggle,
        })
    }

    /// The study check's difficulty at `overreach` spell levels (after Overreach's cut) with
    /// spells of `known` levels in all known, before the bonuses: `base + per_overreach ×
    /// overreach + known ÷ known_per_point`.
    pub fn difficulty(&self, overreach: i32, known: i32) -> i32 {
        self.base + self.per_overreach * overreach + known / self.known_per_point
    }

    /// What a deepening to `rank` adds to the difficulty (#1264): `per_rank` for each rank
    /// already held, `per_rank × (rank − 1)`.
    pub fn rank_difficulty(&self, rank: u8) -> i32 {
        self.per_rank * (i32::from(rank) - 1).max(0)
    }

    /// The band of a failure at `overreach` (1 or more).
    /// (Spell levels.)
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

/// What a mishap does, or a miscast (`content/miscast.rs`): those from `Harm` on are the
/// miscast's only, `Learn` the study's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MishapEffect {
    Nothing,
    Status {
        status: StatusId,
        severity: u16,
    },
    /// The spell goes off on its caster: at its base roll, or `full`, at the roll it was cast
    /// with (a miscast's only).
    Backfire {
        full: bool,
    },
    Mana {
        percent: u8,
    },
    Teleport,
    LoseMark,
    /// `points` off `stat` (a random one when `None`) for good unless a Willpower check against
    /// `difficulty` is made, `times` over, each its own draw and check.
    Drain {
        stat: Option<Stat>,
        points: u8,
        difficulty: i32,
        times: u8,
    },
    Summon {
        creature: CreatureId,
    },
    Learn,
    Mark {
        stat: Stat,
    },
    /// `dice` of harm to the caster, of type `damage` (the spell's own when `None`).
    Harm {
        dice: Dice,
        damage: Option<DamageTypeId>,
    },
    /// `points` of stat damage to `stat` (a random one when `None`), which mends (a miscast's
    /// only).
    StatDamage {
        stat: Option<Stat>,
        points: u8,
    },
    /// The mana the cast cost comes back.
    Refund,
    /// The spell goes off a second time, at no cost.
    Twice,
    /// The caster changes places with someone near them (a creature never with the player).
    Swap,
    /// `status` falls on someone near the caster, drawn at random (the caster too).
    Boon {
        status: StatusId,
        severity: u16,
    },
    /// A sound of `loudness` on the caster's cell, a thunderclap's.
    Noise {
        loudness: u16,
    },
    /// A zone of `kind` around the caster, within `radius`, for `turns`.
    Zone {
        kind: ZoneKind,
        radius: u8,
        turns: u16,
    },
}

impl MishapEffect {
    /// Its kind's id, as data writes it.
    pub const fn kind(&self) -> &'static str {
        match self {
            MishapEffect::Nothing => "nothing",
            MishapEffect::Status { .. } => "status",
            MishapEffect::Backfire { .. } => "backfire",
            MishapEffect::Mana { .. } => "mana",
            MishapEffect::Teleport => "teleport",
            MishapEffect::LoseMark => "lose_mark",
            MishapEffect::Drain { .. } => "drain",
            MishapEffect::Summon { .. } => "summon",
            MishapEffect::Learn => "learn",
            MishapEffect::Mark { .. } => "mark",
            MishapEffect::Harm { .. } => "harm",
            MishapEffect::StatDamage { .. } => "stat_damage",
            MishapEffect::Refund => "refund",
            MishapEffect::Twice => "twice",
            MishapEffect::Swap => "swap",
            MishapEffect::Boon { .. } => "boon",
            MishapEffect::Noise { .. } => "noise",
            MishapEffect::Zone { .. } => "zone",
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

/// An effect as data writes it: its `kind` and only that kind's fields.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct EffectSpec {
    kind: String,
    status: Option<String>,
    severity: Option<u16>,
    percent: Option<u8>,
    stat: Option<String>,
    points: Option<u8>,
    difficulty: Option<i32>,
    times: Option<u8>,
    creature: Option<String>,
    dice: Option<String>,
    damage: Option<String>,
    full: Option<bool>,
    loudness: Option<u16>,
    zone: Option<String>,
    radius: Option<u8>,
    turns: Option<u16>,
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
    let effect = resolve_effect(spec.effect, content, false)?;
    Ok(Mishap {
        id: spec.id,
        band: spec.band,
        weight: spec.weight,
        message: spec.message,
        effect,
    })
}

/// An effect of a mishap (`miscast` false) or of a miscast (true): each kind for its table only
/// (module docs), with only its own fields.
pub(super) fn resolve_effect(
    e: EffectSpec,
    content: &Content,
    miscast: bool,
) -> Result<MishapEffect, String> {
    let only_miscast = [
        "harm",
        "stat_damage",
        "refund",
        "twice",
        "swap",
        "boon",
        "noise",
        "zone",
    ];
    if !miscast && only_miscast.contains(&e.kind.as_str()) {
        return Err(format!("a `{}` effect is a miscast's only", e.kind));
    }
    if miscast && e.kind == "learn" {
        return Err("a `learn` effect is a failed study's only".into());
    }
    // Only the fields its kind reads.
    let fields = [
        ("status", e.status.is_some()),
        ("severity", e.severity.is_some()),
        ("percent", e.percent.is_some()),
        ("stat", e.stat.is_some()),
        ("points", e.points.is_some()),
        ("difficulty", e.difficulty.is_some()),
        ("times", e.times.is_some()),
        ("creature", e.creature.is_some()),
        ("dice", e.dice.is_some()),
        ("damage", e.damage.is_some()),
        ("full", e.full.is_some()),
        ("loudness", e.loudness.is_some()),
        ("zone", e.zone.is_some()),
        ("radius", e.radius.is_some()),
        ("turns", e.turns.is_some()),
    ];
    let reads: &[&str] = match e.kind.as_str() {
        "status" | "boon" => &["status", "severity"],
        "harm" => &["dice", "damage"],
        "backfire" if miscast => &["full"],
        "mana" => &["percent"],
        "drain" => &["stat", "points", "difficulty", "times"],
        "stat_damage" => &["stat", "points"],
        "summon" => &["creature"],
        "mark" => &["stat"],
        "noise" => &["loudness"],
        "zone" => &["zone", "radius", "turns"],
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
        "backfire" => MishapEffect::Backfire {
            full: e.full.unwrap_or(false),
        },
        "refund" => MishapEffect::Refund,
        "twice" => MishapEffect::Twice,
        "swap" => MishapEffect::Swap,
        "noise" => {
            let loudness = e.loudness.ok_or("a `noise` effect needs a `loudness`")?;
            if !(1..=crate::noise::MAX_LOUDNESS).contains(&loudness) {
                return Err(format!(
                    "a `noise` effect's `loudness` is 1 to {}",
                    crate::noise::MAX_LOUDNESS
                ));
            }
            MishapEffect::Noise { loudness }
        }
        "zone" => {
            let name = e.zone.ok_or("a `zone` effect needs a `zone`")?;
            let kind = (ZoneKind::ALL.iter())
                .find(|k| k.id() == name)
                .copied()
                .ok_or(format!("unknown zone kind `{name}`"))?;
            let radius = e.radius.filter(|&r| r <= 20);
            let radius = radius.ok_or("a `zone` effect needs a `radius`, 0 to 20")?;
            let turns = e.turns.filter(|&t| t > 0);
            let turns = turns.ok_or("a `zone` effect needs `turns`, at least 1")?;
            MishapEffect::Zone {
                kind,
                radius,
                turns,
            }
        }
        "boon" => {
            let name = e.status.ok_or("a `boon` effect needs a `status`")?;
            let status = (content.status_id(&name)).ok_or(format!("unknown status `{name}`"))?;
            let severity = e.severity.unwrap_or(1);
            MishapEffect::Boon { status, severity }
        }
        "harm" => {
            let text = e.dice.ok_or("a `harm` effect needs `dice`")?;
            let dice: Dice = text.parse().map_err(|m| format!("dice `{text}`: {m}"))?;
            let damage = match e.damage {
                Some(name) => Some(
                    (content.damage_type_id(&name))
                        .ok_or(format!("unknown damage type `{name}`"))?,
                ),
                None => None,
            };
            MishapEffect::Harm { dice, damage }
        }
        "teleport" => MishapEffect::Teleport,
        "lose_mark" => MishapEffect::LoseMark,
        "learn" => MishapEffect::Learn,
        "status" => {
            let name = e.status.ok_or("a `status` effect needs a `status`")?;
            let status = (content.status_id(&name)).ok_or(format!("unknown status `{name}`"))?;
            let severity = e.severity.unwrap_or(1);
            MishapEffect::Status { status, severity }
        }
        "mana" => {
            let percent = e.percent.ok_or("a `mana` effect needs a `percent`")?;
            MishapEffect::Mana { percent }
        }
        "drain" => {
            let points = e.points.ok_or("a `drain` effect needs `points`")?;
            let difficulty = e
                .difficulty
                .ok_or("a `drain` effect needs a `difficulty`")?;
            let times = e.times.unwrap_or(1);
            if times < 1 {
                return Err("a `drain` effect's `times` is at least 1".into());
            }
            MishapEffect::Drain {
                stat: stat(&e.stat, true)?,
                points,
                difficulty,
                times,
            }
        }
        "stat_damage" => {
            let points = e.points.filter(|&p| p > 0);
            let points = points.ok_or("a `stat_damage` effect needs `points`, at least 1")?;
            MishapEffect::StatDamage {
                stat: stat(&e.stat, true)?,
                points,
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
    Ok(effect)
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

    const RULES: &str = "base = 12\nper_overreach = 1\nknown_per_point = 20\nper_rank = 2\n\
        turns_base = 100\nturns_per_level = 20\nbad_from = 6\nterrible_from = 15\neasy = 96\n\
        struggle = 36\n";

    #[test]
    fn the_rules_parse_and_say_the_difficulty_band_and_words() {
        let r = StudyRules::parse("study.toml", RULES).expect("valid");
        assert_eq!(r.difficulty(1, 0), 13);
        assert_eq!(r.difficulty(2, 45), 16, "45 levels known: 2 points harder");
        assert_eq!(r.rank_difficulty(1), 0, "a learning adds nothing");
        assert_eq!(r.rank_difficulty(2), 2);
        assert_eq!(r.rank_difficulty(10), 18, "per_rank for each rank held");
        assert_eq!(r.band(5), Band::Slight);
        assert_eq!(r.band(6), Band::Bad);
        assert_eq!(r.band(14), Band::Bad);
        assert_eq!(r.band(15), Band::Terrible);
        assert_eq!(r.words(144), "certain");
        assert_eq!(r.words(96), "easy");
        assert_eq!(r.words(40), "a struggle");
        assert_eq!(r.words(35), "far beyond you");
    }

    #[test]
    fn bad_rules_are_refused() {
        for bad in [
            RULES.replace("known_per_point = 20", "known_per_point = 0"),
            RULES.replace("easy = 96\n", ""),
        ] {
            assert!(StudyRules::parse("study.toml", &bad).is_err(), "{bad}");
        }
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
            ("{ kind = \"nothing\", percent = 5 }", "slight"),
            ("{ kind = \"mana\" }", "bad"),
            (
                "{ kind = \"drain\", stat = \"random\", points = 2 }",
                "terrible",
            ),
            ("{ kind = \"summon\", creature = \"nobody\" }", "terrible"),
            ("{ kind = \"mark\", stat = \"luck\" }", "lucky"),
        ] {
            assert!(one(effect, band).is_err(), "{effect} {band}");
        }
    }
}
