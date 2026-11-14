//! The resource talents (#1495): what a talent does to the holder's alternative resources (spec §8
//! "Resources"): which it holds and how they build, fill, start, and change over time.
//! [`ResourceEffect`] is the data (a talent's `[[effect]]` of one of [`ResourceEffect::KINDS`]),
//! [`ResourceMods`] the sum every rule reads (`TalentMods::resource`). A new effect of the area is
//! a variant here, its field and its arm in [`ResourceMods::add`] here, its wiki text in
//! `xtask/src/wiki/talents/resource.rs`, and its one read.

use crate::content::{Build, Content, ResourceId};
use serde::Deserialize;

/// What a talent of the area does, one typed effect, read by one rule each.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResourceEffect {
    /// Event `on` builds `resource` this much more while it is held, beside what the resource's
    /// own `builds` give (Blood frenzy: 5 rage a kill).
    ResourceBuild {
        resource: String,
        on: Build,
        amount: u16,
    },
    /// `resource` holds this much more at most (Mighty rage: 10 more rage).
    ResourceMax { resource: String, value: u16 },
    /// `resource` is granted holding this much more (Battle trance: rage starts 6 higher).
    ResourceStart { resource: String, value: u16 },
    /// `resource` changes over time this percent faster (negative: slower): one that
    /// regenerates comes back sooner, one that decays wears off sooner (Steady hand, Patient
    /// blade).
    ResourceRate { resource: String, value: i32 },
    /// It holds alternative resource `resource` (spec §8 "Resources") from the moment it has the
    /// talent, for good (Rapid shot's Focus).
    GrantsResource { resource: String },
}

impl ResourceEffect {
    /// The `kind`s of the data that are effects of the area, one a variant (`TalentEffect`'s
    /// deserializer hands these to [`ResourceEffect`]).
    pub const KINDS: [&'static str; 5] = [
        "grants_resource",
        "resource_build",
        "resource_max",
        "resource_rate",
        "resource_start",
    ];
}

/// What an actor's talents of the area do, summed: one field per effect kind, read by one
/// rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResourceMods {
    /// What events build resources beyond the resources' own `builds`: `(resource, on,
    /// amount)` (Blood frenzy's rage a kill).
    pub resource_builds: Vec<(ResourceId, Build, u16)>,
    /// How much more of a resource it may hold: `(resource, more)` (Mighty rage).
    pub resource_max: Vec<(ResourceId, u16)>,
    /// How much more of a resource it holds when granted: `(resource, more)` (Battle trance).
    pub resource_start: Vec<(ResourceId, u16)>,
    /// How much faster a resource changes over time, in percent: `(resource, percent)` (Steady
    /// hand's +50, Patient blade's -50).
    pub resource_rate: Vec<(ResourceId, i32)>,
    /// The alternative resources its talents grant it, in the order first named (Rapid shot's
    /// Focus, the thief's Sneak attack's Openings).
    pub grants: Vec<ResourceId>,
}

impl ResourceMods {
    /// Adds what `effect` does.
    pub(crate) fn add(&mut self, content: &Content, effect: ResourceEffect) {
        match effect {
            ResourceEffect::ResourceBuild {
                resource,
                on,
                amount,
            } => {
                if let Some(r) = content.resource_id(&resource) {
                    self.resource_builds.push((r, on, amount));
                }
            }
            ResourceEffect::ResourceMax { resource, value } => {
                if let Some(r) = content.resource_id(&resource) {
                    self.resource_max.push((r, value));
                }
            }
            ResourceEffect::ResourceStart { resource, value } => {
                if let Some(r) = content.resource_id(&resource) {
                    self.resource_start.push((r, value));
                }
            }
            ResourceEffect::ResourceRate { resource, value } => {
                if let Some(r) = content.resource_id(&resource) {
                    self.resource_rate.push((r, value));
                }
            }
            ResourceEffect::GrantsResource { resource } => {
                if let Some(resource) = content.resource_id(&resource)
                    && !self.grants.contains(&resource)
                {
                    self.grants.push(resource);
                }
            }
        }
    }
}
