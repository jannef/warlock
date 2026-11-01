//! Rods (spec §8 "Magic items": "Rods", the magic design §9): an object with an `[object.rod]`
//! table holds an effect no spell has, of the rod effects (`data/rod_effects/`, one
//! `[[rod_effect]]` a file), and costs its user mana for each use instead of charges; its reach
//! grows on a ladder of the user's stats as a spell's does (`grows.range`, the spell scaling
//! design §7.5). No caster level is read: a rod is the same in anyone's hands. It goes in the
//! missile weapon slot and is used as a missile weapon is (`t`, `fire:X,Y`), as a wand is.
//!
//! ```toml
//! [object.rod]
//! effect = "digging"   # an id of data/rod_effects/
//! mana = 6             # what a use costs its user, before their talents
//! ```
//!
//! A rod effect (`data/rod_effects/README.toml` has the format) is its shape, its reach, and what
//! it does, by `kind`, on existing code: `dig` (walls give way, as to digging), `teleport` (the
//! creature hit is moved, as by a teleport trap), `status` (a status on everyone in a burst, as a
//! hex's).

use super::spell_ladder::{Ladder, Unit};
use super::{Content, ContentError, StatusId, TrapNoise, duplicate_id};
use crate::noise::MAX_LOUDNESS;
use crate::stats::Stats;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The most mana a rod may cost a use.
pub const MAX_ROD_MANA: u16 = 100;
/// The farthest a rod effect may reach, in cells.
pub const MAX_ROD_REACH: u8 = 20;
/// The largest burst a rod effect may have.
pub const MAX_ROD_RADIUS: u8 = 3;
/// The hardest wall a digging effect may be able to break.
pub const MAX_ROD_POWER: u16 = 10_000;

/// How a rod effect goes from its user.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RodShape {
    /// Every cell of a straight line from the user toward the cell aimed at, as far as its reach,
    /// through what it breaks.
    Line,
    /// A bolt toward the cell aimed at, as far as its reach, that stops at the first creature (or
    /// the first wall) on its way.
    Bolt,
    /// A burst of its `radius` at the cell aimed at, within its reach (stopped short by a wall
    /// as a thrown burst is).
    Burst,
}

impl RodShape {
    pub const fn id(self) -> &'static str {
        match self {
            RodShape::Line => "line",
            RodShape::Bolt => "bolt",
            RodShape::Burst => "burst",
        }
    }
}

/// What a rod effect does where it goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RodDoes {
    /// Every wall on its line whose hardness is below `power` gives way at once, as dug through;
    /// the first that does not (or the level's edge) stops it.
    Dig { power: u16 },
    /// The creature its bolt hits is moved to a free cell of the level, as by a teleport trap.
    Teleport,
    /// Everyone in its burst but the user suffers `status` at `severity`; with `up_to_level`,
    /// only creatures whose level is at most it do (the others shrug it off).
    Status {
        status: StatusId,
        severity: u16,
        up_to_level: Option<u8>,
    },
}

impl RodDoes {
    /// Its kind's id, as data writes it.
    pub const fn kind(&self) -> &'static str {
        match self {
            RodDoes::Dig { .. } => "dig",
            RodDoes::Teleport => "teleport",
            RodDoes::Status { .. } => "status",
        }
    }
}

/// A rod effect (`data/rod_effects/`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RodEffect {
    pub id: String,
    /// What the player calls it once they know the rod: "Digging".
    pub name: String,
    /// What it does, in words, for the item description.
    pub description: String,
    pub shape: RodShape,
    /// How far it reaches for anyone, in cells: the base.
    pub range: u8,
    /// Its reach's ladder (`grows.range`, cells; spec §8 "Spells": "Ladders"), if it grows.
    pub grows: Option<Ladder>,
    /// A burst's radius (0 for the other shapes).
    pub radius: u8,
    /// The noise it makes where it goes off (a burst's centre, a line's or a bolt's user).
    pub noise: Option<TrapNoise>,
    pub does: RodDoes,
}

impl RodEffect {
    /// How far it reaches for a user of effective `stats`: the rung of its ladder their score
    /// in the ladder's stat reaches, its `range` below the first.
    pub fn reach(&self, stats: &Stats) -> i32 {
        let base = i32::from(self.range);
        let Some(ladder) = &self.grows else {
            return base;
        };
        let rung = ladder.rung_at(i32::from(stats.score(ladder.stat)));
        (ladder.value(rung))
            .and_then(|v| v.number())
            .unwrap_or(base)
    }
}

/// `[object.rod]` as data writes it.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RodSpec {
    effect: String,
    mana: u16,
}

/// A rod, resolved: the effect it holds (an index of [`Content::rod_effects`]) and its cost.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rod {
    pub effect: usize,
    /// The mana a use costs its user, 1 to [`MAX_ROD_MANA`], before their talents (Channeler,
    /// Frugal caster).
    pub mana: u16,
}

impl RodSpec {
    /// Resolves it against the rod effects (loaded before objects).
    pub(super) fn resolve(self, content: &Content) -> Result<Rod, String> {
        let effect = (content.rod_effect_index(&self.effect))
            .ok_or_else(|| format!("`rod`: no effect `{}` in data/rod_effects/", self.effect))?;
        if !(1..=MAX_ROD_MANA).contains(&self.mana) {
            return Err(format!("`rod`: `mana` is 1 to {MAX_ROD_MANA}"));
        }
        Ok(Rod {
            effect,
            mana: self.mana,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RodEffectFile {
    #[serde(default)]
    rod_effect: Vec<RodEffectSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RodEffectSpec {
    id: String,
    name: String,
    description: String,
    shape: String,
    range: u8,
    grows: Option<GrowsSpec>,
    radius: Option<u8>,
    noise: Option<TrapNoise>,
    kind: String,
    power: Option<u16>,
    status: Option<String>,
    severity: Option<u16>,
    up_to_level: Option<u8>,
    // Retired (the spell scaling design §7.5): refused, naming what replaced them.
    max_range: Option<toml::Value>,
    per_levels: Option<toml::Value>,
    up_to_caster_level: Option<toml::Value>,
}

/// `[rod_effect.grows]` as written: the reach's ladder.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GrowsSpec {
    range: Option<String>,
}

/// Parses the rod effect files, in path order; statuses must be loaded.
pub(super) fn parse<'a>(
    files: impl Iterator<Item = (&'a str, &'a str)>,
    content: &Content,
) -> Result<Vec<RodEffect>, ContentError> {
    let mut all: Vec<RodEffect> = Vec::new();
    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    for (path, text) in files {
        let file: RodEffectFile =
            toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))?;
        for spec in file.rod_effect {
            if let Some(first) = seen.get(&spec.id) {
                return Err(duplicate_id("rod effect", &spec.id, path, first));
            }
            seen.insert(spec.id.clone(), path.to_string());
            let id = spec.id.clone();
            let effect = resolve(spec, content)
                .map_err(|m| ContentError::new(path, format!("rod effect `{id}`: {m}")))?;
            all.push(effect);
        }
    }
    Ok(all)
}

fn resolve(spec: RodEffectSpec, content: &Content) -> Result<RodEffect, String> {
    if spec.name.trim().is_empty() || spec.description.trim().is_empty() {
        return Err("`name` and `description` are not empty".into());
    }
    let shape = match spec.shape.as_str() {
        "line" => RodShape::Line,
        "bolt" => RodShape::Bolt,
        "burst" => RodShape::Burst,
        other => return Err(format!("unknown shape `{other}` (line, bolt, burst)")),
    };
    retired(&spec)?;
    if !(1..=MAX_ROD_REACH).contains(&spec.range) {
        return Err(format!("`range` is 1 to {MAX_ROD_REACH}"));
    }
    let grows = match spec.grows.and_then(|g| g.range) {
        Some(text) => Some(reach_ladder(&text, spec.range)?),
        None => None,
    };
    let radius = match (shape, spec.radius) {
        (RodShape::Burst, Some(r)) if (1..=MAX_ROD_RADIUS).contains(&r) => r,
        (RodShape::Burst, _) => return Err(format!("a burst's `radius` is 1 to {MAX_ROD_RADIUS}")),
        (_, Some(_)) => return Err("only a burst has a `radius`".into()),
        (_, None) => 0,
    };
    if let Some(noise) = spec.noise
        && !(1..=MAX_LOUDNESS).contains(&noise.loudness)
    {
        return Err(format!("`noise` loudness is 1 to {MAX_LOUDNESS}"));
    }
    // Only the fields its kind reads.
    let fields = [
        ("power", spec.power.is_some()),
        ("status", spec.status.is_some()),
        ("severity", spec.severity.is_some()),
        ("up_to_level", spec.up_to_level.is_some()),
    ];
    let (reads, wants): (&[&str], RodShape) = match spec.kind.as_str() {
        "dig" => (&["power"], RodShape::Line),
        "teleport" => (&[], RodShape::Bolt),
        "status" => (&["status", "severity", "up_to_level"], RodShape::Burst),
        other => return Err(format!("unknown kind `{other}` (dig, teleport, status)")),
    };
    if let Some((name, _)) = fields.iter().find(|(n, set)| *set && !reads.contains(n)) {
        return Err(format!("a `{}` effect has no `{name}`", spec.kind));
    }
    if shape != wants {
        return Err(format!("a `{}` effect is a `{}`", spec.kind, wants.id()));
    }
    let does = match spec.kind.as_str() {
        "dig" => {
            let power = spec.power.ok_or("a `dig` effect needs a `power`")?;
            if !(1..=MAX_ROD_POWER).contains(&power) {
                return Err(format!("`power` is 1 to {MAX_ROD_POWER}"));
            }
            RodDoes::Dig { power }
        }
        "teleport" => RodDoes::Teleport,
        _ => {
            let name = spec.status.ok_or("a `status` effect needs a `status`")?;
            let status = (content.status_id(&name)).ok_or(format!("unknown status `{name}`"))?;
            let severity = spec.severity.unwrap_or(1);
            let max = content.status(status).max;
            if severity < 1 || severity > max {
                return Err(format!("`severity` is 1 to the status's max, {max}"));
            }
            if spec.up_to_level == Some(0) {
                return Err("`up_to_level` is at least 1".into());
            }
            RodDoes::Status {
                status,
                severity,
                up_to_level: spec.up_to_level,
            }
        }
    };
    Ok(RodEffect {
        id: spec.id,
        name: spec.name,
        description: spec.description,
        shape,
        range: spec.range,
        grows,
        radius,
        noise: spec.noise,
        does,
    })
}

/// The retired fields, each refused naming its replacement (the spell scaling design §7.5).
fn retired(spec: &RodEffectSpec) -> Result<(), String> {
    if spec.max_range.is_some() || spec.per_levels.is_some() {
        return Err(
            "`max_range` and `per_levels` are gone: `range` is what anyone reaches, and a \
             `grows.range` ladder grows it (`willpower 14 4, 18 5`; docs/design/spell-scaling.md \
             §7.5)"
                .into(),
        );
    }
    if spec.up_to_caster_level.is_some() {
        return Err(
            "`up_to_caster_level` is gone: `up_to_level` is the highest creature level it takes \
             (docs/design/spell-scaling.md §7.5)"
                .into(),
        );
    }
    Ok(())
}

/// `grows.range` read and checked: a ladder of whole cells rising strictly from `base` to at
/// most [`MAX_ROD_REACH`].
fn reach_ladder(text: &str, base: u8) -> Result<Ladder, String> {
    let ladder = Ladder::parse(text, Unit::Cells).map_err(|e| format!("grows: {e}"))?;
    let mut before = i32::from(base);
    for value in ladder.values() {
        let n = value.number().unwrap_or(0);
        if n <= before || n > i32::from(MAX_ROD_REACH) {
            return Err(format!(
                "grows: `range`'s rungs rise strictly from its base, {base}, to at most \
                 {MAX_ROD_REACH}; not {n}"
            ));
        }
        before = n;
    }
    Ok(ladder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentFiles;

    fn load_with(file: &str) -> Result<Content, ContentError> {
        let mut files = ContentFiles::builtin();
        files.insert("data/rod_effects/zz_test.toml", file);
        Content::load(&files)
    }

    #[test]
    fn the_shipped_effects_load_and_reach_grows_with_willpower() {
        let content = Content::load(&ContentFiles::builtin()).expect("content");
        let ids: Vec<&str> = content
            .rod_effects()
            .iter()
            .map(|e| e.id.as_str())
            .collect();
        assert_eq!(ids, ["digging", "sleep", "teleport_other", "terror"]);
        let i = content.rod_effect_index("digging").expect("digging");
        let dig = &content.rod_effects()[i];
        assert_eq!(dig.shape, RodShape::Line);
        let with = |willpower: u8| {
            let will = crate::stats::Stat::Willpower;
            let average = i32::from(Stats::default().score(will));
            let stats = Stats::default().raised(|s| {
                if s == will {
                    i32::from(willpower) - average
                } else {
                    0
                }
            });
            dig.reach(&stats)
        };
        let reaches: Vec<i32> = [3, 13, 14, 17, 18, 22, 26, 40].map(with).to_vec();
        assert_eq!(reaches, [3, 3, 4, 4, 5, 6, 7, 7]);
        assert_eq!(
            dig.grows.as_ref().map(|l| l.to_string()).as_deref(),
            Some("willpower 14 4, 18 5, 22 6, 26 7")
        );
    }

    #[test]
    fn malformed_effects_are_refused() {
        let base = "[[rod_effect]]\nid = \"x\"\nname = \"X\"\ndescription = \"x\"\n";
        let cases = [
            (
                "shape = \"cone\"\nrange = 3\nkind = \"teleport\"",
                "unknown shape",
            ),
            (
                "shape = \"bolt\"\nrange = 3\nkind = \"fly\"",
                "unknown kind",
            ),
            (
                "shape = \"line\"\nrange = 3\nkind = \"teleport\"",
                "is a `bolt`",
            ),
            (
                "shape = \"bolt\"\nrange = 3\nkind = \"teleport\"\npower = 4",
                "has no `power`",
            ),
            (
                "shape = \"line\"\nrange = 3\nkind = \"dig\"",
                "needs a `power`",
            ),
            (
                "shape = \"burst\"\nrange = 3\nkind = \"status\"\nstatus = \"asleep\"",
                "radius",
            ),
            (
                "shape = \"burst\"\nrange = 3\nradius = 1\nkind = \"status\"\nstatus = \"nope\"",
                "unknown status",
            ),
            (
                "shape = \"burst\"\nrange = 3\nradius = 1\nkind = \"status\"\nstatus = \"asleep\"\nseverity = 5",
                "`severity` is 1",
            ),
            (
                "shape = \"bolt\"\nrange = 0\nkind = \"teleport\"",
                "`range` is 1 to 20",
            ),
            (
                "shape = \"bolt\"\nrange = 4\ngrows.range = \"willpower 14 4\"\nkind = \"teleport\"",
                "rise strictly from its base, 4",
            ),
            (
                "shape = \"bolt\"\nrange = 4\ngrows.range = \"willpower 14 21\"\nkind = \"teleport\"",
                "to at most 20",
            ),
            (
                "shape = \"bolt\"\nrange = 4\ngrows.range = \"cunning 14 5\"\nkind = \"teleport\"",
                "unknown stat",
            ),
            (
                "shape = \"bolt\"\nrange = 4\ngrows.radius = \"willpower 14 5\"\nkind = \"teleport\"",
                "unknown field",
            ),
            (
                "shape = \"bolt\"\nrange = 4\nkind = \"teleport\"\nup_to_level = 5",
                "has no `up_to_level`",
            ),
            (
                "shape = \"burst\"\nrange = 3\nradius = 1\nkind = \"status\"\nstatus = \"asleep\"\nup_to_level = 0",
                "`up_to_level` is at least 1",
            ),
            // The retired fields, each naming what replaced it (the spell scaling design §7.5).
            (
                "shape = \"bolt\"\nrange = 3\nmax_range = 9\nkind = \"teleport\"",
                "`max_range` and `per_levels` are gone",
            ),
            (
                "shape = \"bolt\"\nrange = 3\nper_levels = 3\nkind = \"teleport\"",
                "a `grows.range` ladder",
            ),
            (
                "shape = \"burst\"\nrange = 3\nradius = 1\nkind = \"status\"\nstatus = \"asleep\"\nup_to_caster_level = true",
                "`up_to_caster_level` is gone: `up_to_level`",
            ),
        ];
        for (body, want) in cases {
            let err = load_with(&format!("{base}{body}\n"))
                .expect_err(body)
                .to_string();
            assert!(err.contains(want), "{body}: {err}");
        }
    }
}
