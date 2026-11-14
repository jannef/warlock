//! A creature type's presence (design `docs/design/monster-dread.md` §5.4 G4, §9; #1128): its
//! aura (`[creature.aura]`), the statuses it is made with (`innate`), and its cry (`cry`). The
//! rules are `world/creatures/aura.rs`.
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
use crate::noise::SoundKind;
use crate::stats::Stat;
use serde::Deserialize;

/// The cooldown status an aura gives its bearer (`data/status/`).
pub const STATUS: &str = "aura";
/// An aura's touches without `every`: two turns apart.
pub const DEFAULT_EVERY: u32 = 12;
/// How loud a heard aura is without `loudness`: a voice.
pub const DEFAULT_LOUDNESS: u16 = crate::noise::VOICE_NOISE;
/// The seconds one idle action stands for in a cry's chance: a turn.
pub const CRY_TURN: u32 = 6;
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
        // A hold names its holder, and an innate status has none.
        if content.status(status).hold.is_some() {
            return Err(format!("creature `{id}`: innate: `{name}` holds"));
        }
        innate.push(status);
    }
    let cry = spec
        .cry
        .map(|c| {
            // A cry's chance is a draw below `every` (`world/creatures/aura.rs`).
            if c.every == 0 {
                return Err("`every` is 0".into());
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
    let check = match (spec.check, spec.difficulty) {
        (Some(stat), Some(d)) => Some((stat, d)),
        (None, None) => None,
        _ => return Err("`check` and `difficulty` go together".into()),
    };
    let every = spec.every.unwrap_or(DEFAULT_EVERY);
    let cooldown = (content.status_id(STATUS))
        .ok_or_else(|| format!("needs the `{STATUS}` status in status/"))?;
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
    let kind = (spec.damage_type.as_deref())
        .map(|k| {
            content
                .damage_type_id(k)
                .ok_or_else(|| format!("unknown damage type `{k}`"))
        })
        .transpose()?;
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
