//! A creature type's gaze (spec §8 "Creature behavior": "Gazes", #1125, design
//! `docs/design/monster-dread.md` §5.1): `[creature.gaze]`, a rider at sight and not at touch.
//! The rules are `world/gaze.rs`; the forced Averted condition a gaze marked `averted` gives is
//! `world/avert.rs`.
//!
//! Its riders are `on_hit` entries (`riders`, each a status with a severity or a drain with its
//! check, by its own chance), never a disease or a hold: a look carries neither. The cooldown is
//! a status (the house rule for ability limits, `ability.rs`): [`STATUS`], given
//! `ceil(cooldown / eases)` degrees, as a summoning's (`summon.rs`).

use super::Content;
use super::disease::DiseaseId;
use super::status::{Affliction, OnHit, OnHitSpec, StatusId};
use serde::Deserialize;
use std::ops::RangeInclusive;

/// The cooldown status a gaze gives (`data/status.toml`).
pub const STATUS: &str = "gaze";
/// How far a gaze reaches, in cells.
pub const RANGES: RangeInclusive<i32> = 1..=8;
/// A gaze's cooldown, in seconds.
pub const COOLDOWNS: RangeInclusive<u32> = 6..=600;
/// A gaze's cooldown unless its type says otherwise, in seconds.
pub const DEFAULT_COOLDOWN: u32 = 12;
/// A glance's share of the gaze's chance unless its type says otherwise, in percent.
pub const DEFAULT_GLANCE: u8 = 25;

/// A gaze as written: `[creature.gaze]` (`data/creatures/README.toml` has the fields).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct GazeSpec {
    pub range: i32,
    pub chance: Option<u8>,
    pub cooldown: Option<u32>,
    #[serde(default)]
    pub averted: bool,
    pub glance: Option<u8>,
    pub riders: Vec<OnHitSpec>,
}

/// A creature type's gaze, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gaze {
    /// It gazes at a hostile it sees within this many cells (Chebyshev).
    pub range: i32,
    /// The chance in percent that eyes that meet it are caught: one draw, made only below 100.
    pub chance: u8,
    /// Seconds before it gazes again.
    pub cooldown: u32,
    /// The cooldown status, and the degrees of it a gaze gives.
    pub status: StatusId,
    pub degrees: u16,
    /// Whether the player averts their eyes by reflex while they see it (`world/avert.rs`).
    pub averted: bool,
    /// The share of `chance` it takes at against averted eyes, in percent (with `averted`).
    pub glance: u8,
    /// What a gaze that takes causes, each by its own chance, in data order.
    pub riders: Vec<OnHit>,
}

impl Gaze {
    /// Its chance against the player's eyes: a glance's share of it when they avert them.
    pub fn chance_against(&self, averted: bool) -> u8 {
        if averted && self.averted {
            let share = u32::from(self.chance) * u32::from(self.glance) / 100;
            u8::try_from(share.max(1)).unwrap_or(self.chance)
        } else {
            self.chance
        }
    }
}

impl GazeSpec {
    /// Resolves the gaze against `content` (the cooldown status, already loaded); `status`
    /// resolves a status id as a natural attack's riders do (to the status, its `max`, and
    /// whether it is a hold), `disease` a disease id.
    pub(super) fn resolve(
        &self,
        content: &Content,
        status: impl Fn(&str) -> Option<(StatusId, u16, bool)>,
        disease: impl Fn(&str) -> Option<DiseaseId>,
    ) -> Result<Gaze, String> {
        if !RANGES.contains(&self.range) {
            return Err(format!(
                "gaze: `range` must be {} to {}",
                RANGES.start(),
                RANGES.end()
            ));
        }
        let chance = self.chance.unwrap_or(100);
        if !(1..=100).contains(&chance) {
            return Err("gaze: `chance` must be 1 to 100 (percent)".into());
        }
        let cooldown = self.cooldown.unwrap_or(DEFAULT_COOLDOWN);
        if !COOLDOWNS.contains(&cooldown) {
            return Err(format!(
                "gaze: `cooldown` must be {} to {} seconds",
                COOLDOWNS.start(),
                COOLDOWNS.end()
            ));
        }
        if self.glance.is_some() && !self.averted {
            return Err("gaze: `glance` goes with `averted = true`".into());
        }
        let glance = self.glance.unwrap_or(DEFAULT_GLANCE);
        if !(1..=100).contains(&glance) {
            return Err("gaze: `glance` must be 1 to 100 (percent)".into());
        }
        if self.riders.is_empty() {
            return Err("gaze: `riders` names nothing".into());
        }
        let riders = (self.riders.iter())
            .map(|r| {
                let rider = r
                    .resolve(&status, &disease)
                    .map_err(|m| format!("gaze: {m}"))?;
                match rider.what {
                    Affliction::Status { .. } | Affliction::Drain(_) => Ok(rider),
                    Affliction::Hold { .. } | Affliction::Disease(_) => Err(
                        "gaze: a rider is a status or a drain, never a hold or a disease".into(),
                    ),
                }
            })
            .collect::<Result<Vec<_>, String>>()?;
        let found = (content.status_id(STATUS))
            .ok_or_else(|| format!("gaze: needs the `{STATUS}` status in data/status.toml"))?;
        let def = content.status(found);
        let eases = (def.eases)
            .filter(|_| def.tick.is_none())
            .ok_or_else(|| format!("gaze: the `{STATUS}` status must ease and never tick"))?;
        let degrees = (u16::try_from(cooldown.div_ceil(eases)).ok())
            .filter(|&d| d <= def.max)
            .ok_or_else(|| {
                format!("gaze: a cooldown of {cooldown} s is more than the `{STATUS}` status lasts")
            })?;
        Ok(Gaze {
            range: self.range,
            chance,
            cooldown,
            status: found,
            degrees,
            averted: self.averted,
            glance,
            riders,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gaze(chance: u8, averted: bool, glance: u8) -> Gaze {
        Gaze {
            range: 6,
            chance,
            cooldown: 12,
            status: StatusId(0),
            degrees: 2,
            averted,
            glance,
            riders: Vec::new(),
        }
    }

    #[test]
    fn a_glance_takes_its_share_only_of_an_averted_gaze_against_averted_eyes() {
        assert_eq!(gaze(100, true, 25).chance_against(true), 25);
        assert_eq!(gaze(100, true, 25).chance_against(false), 100);
        assert_eq!(gaze(100, false, 25).chance_against(true), 100);
        // Never nothing: a glance may always meet the eyes.
        assert_eq!(gaze(2, true, 25).chance_against(true), 1);
    }

    /// The riders moved onto a gaze (design §6, #1125): by type, its range and the statuses and
    /// drained stats its gaze carries; none of them still rides a blow.
    const MOVED: &[(&str, i32, &[&str])] = &[
        ("vampire_spawn", 4, &["transfixed"]),
        ("vampire", 5, &["transfixed"]),
        ("nosferatu", 5, &["transfixed"]),
        ("vampire_lord", 5, &["transfixed"]),
        ("elder_vampire", 6, &["transfixed"]),
        ("eldest_vampire", 6, &["transfixed"]),
        ("sea_hag", 4, &["frightened", "toughness"]),
        ("spectre", 4, &["willpower"]),
        ("white_lady", 5, &["transfixed"]),
        ("night_mare", 5, &["transfixed"]),
        ("ghost_lord", 5, &["transfixed"]),
        ("hand_of_glory", 3, &["transfixed"]),
        ("ultroloth", 5, &["transfixed", "frightened"]),
        ("succubus", 4, &["transfixed"]),
        ("incubus", 4, &["transfixed"]),
        ("rakshasa", 5, &["transfixed", "frightened"]),
        ("duergar_mindbreaker", 4, &["frightened", "transfixed"]),
    ];

    #[test]
    fn the_moved_riders_ride_the_gaze_and_no_blow() {
        let content = Content::builtin().expect("content");
        let named = |hit: &OnHit| match hit.what {
            Affliction::Status { status, .. } => content.status(status).id.clone(),
            Affliction::Drain(d) => d.stat.map_or("random", |s| s.id()).to_string(),
            Affliction::Hold { status, .. } => content.status(status).id.clone(),
            Affliction::Disease(d) => content.disease(d).id.clone(),
        };
        for &(id, range, riders) in MOVED {
            let def = content.creature(content.creature_id(id).expect(id));
            let gaze = def.gaze.as_ref().unwrap_or_else(|| panic!("{id}: no gaze"));
            assert_eq!(gaze.range, range, "{id}");
            assert!(!gaze.averted, "{id}: only the basilisk's gaze is averted");
            let on_gaze: Vec<String> = gaze.riders.iter().map(named).collect();
            assert_eq!(on_gaze, riders, "{id}");
            for attack in def.natural_attacks() {
                for hit in &attack.on_hit {
                    let rider = named(hit);
                    assert!(
                        !riders.contains(&rider.as_str()),
                        "{id}: `{rider}` still rides its {}",
                        attack.name
                    );
                }
            }
        }
        // Nothing else gazes yet.
        let gazers = (content.creature_defs().iter())
            .filter(|d| d.gaze.is_some())
            .count();
        assert_eq!(gazers, MOVED.len());
    }
}
