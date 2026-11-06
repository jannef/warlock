//! A creature type's presence (design `docs/design/monster-dread.md` §5.4 G4, §9; #1128): its
//! aura (`[creature.aura]`), the statuses it is made with (`innate`), and its cry (`cry`). The
//! rules are `world/aura.rs`.
//!
//! - **Aura**: a status every hostile within `radius` takes on the bearer's turn, once every
//!   `every` seconds, unless it makes a check of `check` against `difficulty` (none: it always
//!   takes). By sight (line of sight, no light needed: a chill needs none) or, `heard = true`, by
//!   hearing: to anyone within the radius a voice of `loudness` reaches by the hearing rules,
//!   through walls and doors. `damage_type` makes the dose meet the target's resistance (the cold
//!   aura's Sluggish against cold). The cooldown is a status ([`STATUS`], the house rule).
//! - **Innate**: statuses an actor of the type is made with and that are refreshed each turn it
//!   takes (Fire-cloaked's `retaliates` on a salamander, a devil's barbs); never shown in the
//!   look line, since the type always carries them.
//! - **Cry**: an idle sound now and then (`kind`, `loudness`, `every`: one idle action in
//!   `every / 6` on average), heard by the hearing rules, a warning.

use super::damage_type::DamageTypeId;
use super::status::StatusId;
use super::{Content, creature::CreatureSpec};
use crate::noise::{MAX_LOUDNESS, SoundKind};
use crate::stats::Stat;
use serde::Deserialize;
use std::ops::RangeInclusive;

/// The cooldown status an aura gives its bearer (`data/status.toml`).
pub const STATUS: &str = "aura";
/// How far an aura reaches, in cells.
pub const RADII: RangeInclusive<i32> = 1..=6;
/// Seconds between an aura's touches.
pub const EVERIES: RangeInclusive<u32> = 6..=600;
/// An aura's touches without `every`: two turns apart.
pub const DEFAULT_EVERY: u32 = 12;
/// How loud a heard aura is without `loudness`: a voice.
pub const DEFAULT_LOUDNESS: u16 = crate::noise::VOICE_NOISE;
/// Seconds between a creature's cries, on average.
pub const CRY_EVERIES: RangeInclusive<u32> = 6..=3600;
/// The seconds one idle action stands for in a cry's chance: a turn.
pub const CRY_TURN: u32 = 6;
/// The sounds a creature may cry.
pub const CRY_KINDS: [SoundKind; 9] = [
    SoundKind::Banging,
    SoundKind::Voice,
    SoundKind::Howl,
    SoundKind::Buzzing,
    SoundKind::Giggle,
    SoundKind::Hiss,
    SoundKind::Rattle,
    SoundKind::Scream,
    SoundKind::Wail,
];

/// An aura as written: `[creature.aura]` (`data/creatures/README.toml` has the fields).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuraSpec {
    pub status: String,
    pub severity: Option<u16>,
    pub radius: i32,
    pub check: Option<Stat>,
    pub difficulty: Option<i32>,
    pub every: Option<u32>,
    #[serde(default)]
    pub heard: bool,
    pub loudness: Option<u16>,
    pub damage_type: Option<String>,
    pub touch: String,
    pub unseen: Option<String>,
}

/// A creature type's aura, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Aura {
    /// What it gives, and how many degrees.
    pub status: StatusId,
    pub severity: u16,
    /// How far it reaches (the fov's round radius).
    pub radius: i32,
    /// The check that keeps it off, and its difficulty; `None`: it always takes.
    pub check: Option<(Stat, i32)>,
    /// Seconds between its touches, and the cooldown status's degrees for them.
    pub every: u32,
    pub cooldown: StatusId,
    pub degrees: u16,
    /// `Some(loudness)`: it reaches by hearing, as a voice that loud; `None`: by sight.
    pub heard: Option<u16>,
    /// The damage type whose resistance takes its share of the dose.
    pub kind: Option<DamageTypeId>,
    /// The words of its first touch on the player who sees the bearer, `{the}` its name:
    /// "A chill comes off {the}."
    pub touch: String,
    /// The words when they do not see it: "A wail rises somewhere below."; none: nothing more
    /// than the status's own.
    pub unseen: Option<String>,
}

/// A cry as written: `cry = { kind = "hiss", loudness = 40, every = 60 }`.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CrySpec {
    pub kind: SoundKind,
    pub loudness: u16,
    pub every: u32,
}

/// A creature type's cry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cry {
    pub kind: SoundKind,
    /// How loud it is where the crier stands.
    pub loudness: u16,
    /// Seconds between cries, on average.
    pub every: u32,
}

/// What a creature type's presence resolves to.
pub(super) struct Presence {
    pub aura: Option<Aura>,
    pub innate: Vec<StatusId>,
    pub cry: Option<Cry>,
}

/// Checks and resolves `spec`'s aura, innate statuses, and cry.
pub(super) fn resolve(content: &Content, spec: &CreatureSpec) -> Result<Presence, String> {
    let id = &spec.id;
    let aura = (spec.aura.as_ref())
        .map(|a| aura(content, a))
        .transpose()
        .map_err(|m| format!("creature `{id}`: aura: {m}"))?;
    let mut innate = Vec::new();
    for name in &spec.innate {
        let status = (content.status_id(name))
            .ok_or_else(|| format!("creature `{id}`: innate: unknown status `{name}`"))?;
        let def = content.status(status);
        if def.hold.is_some() || def.hides || def.flees || def.sleeps {
            return Err(format!(
                "creature `{id}`: innate: `{name}` holds, hides, frightens, or sleeps"
            ));
        }
        if innate.contains(&status) {
            return Err(format!("creature `{id}`: innate: `{name}` twice"));
        }
        innate.push(status);
    }
    let cry = spec
        .cry
        .map(|c| {
            if !CRY_KINDS.contains(&c.kind) {
                return Err(format!("`{}` is no cry", c.kind.name()));
            }
            if c.loudness == 0 || c.loudness > MAX_LOUDNESS {
                return Err(format!("`loudness` must be 1 to {MAX_LOUDNESS}"));
            }
            if !CRY_EVERIES.contains(&c.every) {
                return Err(format!(
                    "`every` must be {} to {} seconds",
                    CRY_EVERIES.start(),
                    CRY_EVERIES.end()
                ));
            }
            Ok(Cry {
                kind: c.kind,
                loudness: c.loudness,
                every: c.every,
            })
        })
        .transpose()
        .map_err(|m: String| format!("creature `{id}`: cry: {m}"))?;
    Ok(Presence { aura, innate, cry })
}

fn aura(content: &Content, spec: &AuraSpec) -> Result<Aura, String> {
    let status = (content.status_id(&spec.status))
        .filter(|&s| Some(s) != content.disease_status)
        .ok_or_else(|| format!("unknown status `{}`", spec.status))?;
    let def = content.status(status);
    if def.hold.is_some() {
        return Err("a hold is a blow's, not an aura's".into());
    }
    let severity = spec.severity.unwrap_or(1);
    if severity == 0 || severity > def.max {
        return Err(format!("`severity` must be 1 to {}", def.max));
    }
    if !RADII.contains(&spec.radius) {
        return Err(format!(
            "`radius` must be {} to {}",
            RADII.start(),
            RADII.end()
        ));
    }
    let check = match (spec.check, spec.difficulty) {
        (Some(stat), Some(d)) if (1..=40).contains(&d) => Some((stat, d)),
        (Some(_), Some(_)) => return Err("`difficulty` must be 1 to 40".into()),
        (None, None) => None,
        _ => return Err("`check` and `difficulty` go together".into()),
    };
    let every = spec.every.unwrap_or(DEFAULT_EVERY);
    if !EVERIES.contains(&every) {
        return Err(format!(
            "`every` must be {} to {} seconds",
            EVERIES.start(),
            EVERIES.end()
        ));
    }
    let cooldown = (content.status_id(STATUS))
        .ok_or_else(|| format!("needs the `{STATUS}` status in status.toml"))?;
    let eases = content.status(cooldown).eases.unwrap_or(1).max(1);
    let degrees = u16::try_from(every.div_ceil(eases)).unwrap_or(u16::MAX);
    if degrees > content.status(cooldown).max {
        return Err(format!(
            "`every` is longer than the `{STATUS}` status lasts"
        ));
    }
    let heard = match (spec.heard, spec.loudness) {
        (true, l) => Some(l.unwrap_or(DEFAULT_LOUDNESS)),
        (false, None) => None,
        (false, Some(_)) => return Err("`loudness` goes with `heard = true`".into()),
    };
    if heard.is_some_and(|l| l == 0 || l > MAX_LOUDNESS) {
        return Err(format!("`loudness` must be 1 to {MAX_LOUDNESS}"));
    }
    let kind = (spec.damage_type.as_deref())
        .map(|k| {
            content
                .damage_type_id(k)
                .ok_or_else(|| format!("unknown damage type `{k}`"))
        })
        .transpose()?;
    if spec.touch.trim().is_empty() || spec.unseen.as_ref().is_some_and(|u| u.trim().is_empty()) {
        return Err("`touch` and `unseen` must not be empty".into());
    }
    if !spec.touch.contains("{the}") {
        return Err("`touch` names the bearer as `{the}`".into());
    }
    Ok(Aura {
        status,
        severity,
        radius: spec.radius,
        check,
        every,
        cooldown,
        degrees,
        heard,
        kind,
        touch: spec.touch.clone(),
        unseen: spec.unseen.clone(),
    })
}
