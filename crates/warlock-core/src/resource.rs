//! Alternative resources (spec §8 "Resources", #225): the pools an actor holds beside hit
//! points and mana ([`Pools`], `Actor::resources`), and the pure rules of time passing for them.
//! `content/resource.rs` is the data; `world/resource.rs` grants, builds, spends, and ends them,
//! and `world/status.rs` passes time for them in `World::live`'s loop.
//!
//! - **State**: one [`Pool`] per resource held, in resource order (file order): its value, and
//!   the progress toward its next point in hundredths of a second at the normal pace (like
//!   `Actor::regen` toward a hit point), so a point comes at its exact moment however time is
//!   cut up. A resource not held has no pool; one that ends at zero loses its pool there.
//! - **Time** ([`pass`], [`until_next`]): a resource that regenerates gains a point every
//!   `regenerates` seconds up to its maximum (its progress dropped there); one that decays loses a
//!   point every `decays` seconds down to 0, where one that ends at zero ends. The pace in percent
//!   ([`Pace::rate`]) is the world's to decide (`World::resource_rate`).
//!
//! Only the player's own resources are ever shown: their values and maxima. Never the pace.

use crate::content::{Content, OverTime, ResourceId};
use serde::{Deserialize, Serialize};

/// One resource an actor holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pool {
    pub resource: ResourceId,
    /// What it holds now: 0 to its maximum.
    pub value: u16,
    /// Progress toward its next point gained or lost over time, in hundredths of a second at
    /// the normal pace: each second adds its rate in percent, and every `every` × 100 of it is
    /// a point.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub progress: u32,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// The resources an actor holds, in resource order; none for most (and every creature's).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Pools(Vec<Pool>);

impl Pools {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Every pool, in resource order.
    pub fn iter(&self) -> impl Iterator<Item = &Pool> {
        self.0.iter()
    }

    /// The pool of `resource`, if it is held.
    pub fn get(&self, resource: ResourceId) -> Option<&Pool> {
        self.0.iter().find(|p| p.resource == resource)
    }

    pub fn get_mut(&mut self, resource: ResourceId) -> Option<&mut Pool> {
        self.0.iter_mut().find(|p| p.resource == resource)
    }

    /// Whether `resource` is held.
    pub fn has(&self, resource: ResourceId) -> bool {
        self.get(resource).is_some()
    }

    /// Holds `resource` at `value` (no progress), in its place in resource order; replaces the
    /// value of one already held, keeping its progress.
    pub fn set(&mut self, resource: ResourceId, value: u16) {
        match self.0.binary_search_by_key(&resource, |p| p.resource) {
            Ok(i) => self.0[i].value = value,
            Err(i) => self.0.insert(
                i,
                Pool {
                    resource,
                    value,
                    progress: 0,
                },
            ),
        }
    }

    /// Stops holding `resource`; returns whether it was held.
    pub fn remove(&mut self, resource: ResourceId) -> bool {
        let before = self.0.len();
        self.0.retain(|p| p.resource != resource);
        self.0.len() != before
    }
}

/// How one held resource changes over time now: its maximum and its pace in percent of the
/// normal one (100; 0 stops it), as the world decides them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pace {
    pub resource: ResourceId,
    pub max: u16,
    pub rate: u32,
}

/// The progress a point of `pool` needs, and which way it goes, while it changes at all:
/// `None` for one that does not change over time, at a standstill, or with nowhere to go (a
/// regenerating one full, a decaying one empty).
fn need(pool: &Pool, content: &Content, pace: &Pace) -> Option<(u64, OverTime)> {
    let over = content.resource(pool.resource).over_time?;
    let going = match over {
        OverTime::Regenerates { .. } => pool.value < pace.max,
        OverTime::Decays { .. } => pool.value > 0,
    };
    (going && pace.rate > 0).then(|| (u64::from(over.every()) * 100, over))
}

/// Seconds until the next point any of `pools` gains or loses over time at `paces` (at least
/// 1): a moment of `World::live`'s loop. `None` when none is coming.
pub fn until_next(pools: &Pools, content: &Content, paces: &[Pace]) -> Option<u64> {
    (pools.iter().zip(paces))
        .filter_map(|(pool, pace)| {
            let (need, _) = need(pool, content, pace)?;
            let left = need.saturating_sub(u64::from(pool.progress));
            Some(left.div_ceil(u64::from(pace.rate)).max(1))
        })
        .min()
}

/// `seconds` pass for `pools` at `paces` (one per pool, in order): each that changes over time
/// gains or loses its points, its progress banked. Returns the resources that reached 0 and end
/// there (still held: the caller ends them, `World::end_resource`).
pub fn pass(pools: &mut Pools, content: &Content, paces: &[Pace], seconds: u64) -> Vec<ResourceId> {
    let mut ended = Vec::new();
    for (pool, pace) in pools.0.iter_mut().zip(paces) {
        pool.value = pool.value.min(pace.max);
        let Some((need, over)) = need(pool, content, pace) else {
            // Nowhere to go drops the progress; a standstill (a rest it waits out) keeps it.
            if pace.rate > 0 {
                pool.progress = 0;
            }
            continue;
        };
        let banked =
            u64::from(pool.progress).saturating_add(seconds.saturating_mul(u64::from(pace.rate)));
        let points = u16::try_from(banked / need).unwrap_or(u16::MAX);
        let rest = u32::try_from(banked % need).unwrap_or(0);
        match over {
            OverTime::Regenerates { .. } => {
                pool.value = pool.value.saturating_add(points).min(pace.max);
                pool.progress = if pool.value >= pace.max { 0 } else { rest };
            }
            OverTime::Decays { .. } => {
                pool.value = pool.value.saturating_sub(points);
                pool.progress = if pool.value == 0 { 0 } else { rest };
                if pool.value == 0 && content.resource(pool.resource).ends_at_zero {
                    ended.push(pool.resource);
                }
            }
        }
    }
    ended
}
