//! Rods (spec §8 "Magic items": "Rods", the magic design §9): an object with an `[object.rod]`
//! table holds an effect no spell has, of the rod effects (`data/rod_effects/`, one
//! `[[rod_effect]]` a file), and costs its user mana for each use instead of charges; the user's
//! caster level sets how far it reaches. It goes in the missile weapon slot and is used as a
//! missile weapon is (`t`, `fire:X,Y`), as a wand is.
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

use super::{Content, ContentError, StatusId, TrapNoise, duplicate_id};
use crate::noise::MAX_LOUDNESS;
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
    /// Everyone in its burst but the user suffers `status` at `severity`; with
    /// `up_to_caster_level`, only creatures whose level is at most the user's caster level do.
    Status {
        status: StatusId,
        severity: u16,
        up_to_caster_level: bool,
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
    /// How far it reaches at caster level 0, in cells.
    pub range: u8,
    /// The farthest it ever reaches.
    pub max_range: u8,
    /// A cell more of reach per this many of the user's caster levels (0: it never grows).
    pub per_levels: u8,
    /// A burst's radius (0 for the other shapes).
    pub radius: u8,
    /// The noise it makes where it goes off (a burst's centre, a line's or a bolt's user).
    pub noise: Option<TrapNoise>,
    pub does: RodDoes,
}

impl RodEffect {
    /// How far it reaches for a user of `caster_level`: its `range`, a cell more per
    /// `per_levels` levels, never past `max_range`.
    pub fn reach(&self, caster_level: i32) -> i32 {
        let grown = match self.per_levels {
            0 => 0,
            per => caster_level.max(0) / i32::from(per),
        };
        (i32::from(self.range) + grown).min(i32::from(self.max_range))
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
    max_range: Option<u8>,
    #[serde(default)]
    per_levels: u8,
    radius: Option<u8>,
    noise: Option<TrapNoise>,
    kind: String,
    power: Option<u16>,
    status: Option<String>,
    severity: Option<u16>,
    up_to_caster_level: Option<bool>,
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
    let max_range = spec.max_range.unwrap_or(spec.range);
    if spec.range < 1 || !(spec.range..=MAX_ROD_REACH).contains(&max_range) {
        return Err(format!(
            "`range` is at least 1, and `max_range` from it to {MAX_ROD_REACH}"
        ));
    }
    if spec.per_levels > 20 {
        return Err("`per_levels` is 0 to 20".into());
    }
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
        ("up_to_caster_level", spec.up_to_caster_level.is_some()),
    ];
    let (reads, wants): (&[&str], RodShape) = match spec.kind.as_str() {
        "dig" => (&["power"], RodShape::Line),
        "teleport" => (&[], RodShape::Bolt),
        "status" => (
            &["status", "severity", "up_to_caster_level"],
            RodShape::Burst,
        ),
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
            RodDoes::Status {
                status,
                severity,
                up_to_caster_level: spec.up_to_caster_level.unwrap_or(false),
            }
        }
    };
    Ok(RodEffect {
        id: spec.id,
        name: spec.name,
        description: spec.description,
        shape,
        range: spec.range,
        max_range,
        per_levels: spec.per_levels,
        radius,
        noise: spec.noise,
        does,
    })
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
    fn the_shipped_effects_load_and_reach_grows_with_caster_level() {
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
        assert_eq!(dig.reach(0), i32::from(dig.range));
        assert!(dig.reach(9) > dig.reach(0));
        assert_eq!(dig.reach(1000), i32::from(dig.max_range));
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
                "shape = \"bolt\"\nrange = 9\nmax_range = 3\nkind = \"teleport\"",
                "max_range",
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
