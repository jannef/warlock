//! The content fingerprint a save is pinned to.

use super::{Content, ObjectId};
use std::collections::BTreeSet;

impl Content {
    /// A stable hash (64-bit FNV-1a) of the ordered terrain, creature, level, light, dungeon,
    /// object, slot, skill, room effect, weapon skill, status, disease, trap, damage type,
    /// resource, and shop ids, as a run of campaign `campaign` sees them (spec §3): the levels
    /// and dungeon branches of other campaigns left out, and so are the creature types of other
    /// campaigns numbered after every one the run may meet, with their corpses (the objects
    /// after every other). A save of a campaign then loads in a build that holds more campaigns
    /// or fewer, as long as its indices are the same: a release build without the test world
    /// (whose types come last) and a developer build with it.
    ///
    /// `TerrainId`, `CreatureId`, `LightId`, `ObjectId`, `SlotId`, `SkillId`, `RoomEffectId`,
    /// `WeaponSkillId`, `WeaponAbilityId` (never saved, but the menu's and the tests'),
    /// `StatusId`, `DiseaseId`, `TrapId`, `DamageTypeId`, `ResourceId`,
    /// `SpellId`, and `ShopId` are
    /// indices, so a save
    /// is only meaningful for content with the same ids in the same order; saves record this
    /// fingerprint to detect a mismatch.
    pub fn fingerprint_for(&self, campaign: &str) -> u64 {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        let mut feed = |bytes: &[u8]| {
            for &b in bytes {
                hash = (hash ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3);
            }
        };
        let ours = |owner: Option<&str>| owner.is_none_or(|c| c == campaign);
        // Indices count: only the other campaigns' types after the last the run may meet go.
        let creatures = (self.creatures.iter())
            .rposition(|c| ours(c.campaign.as_deref()))
            .map_or(0, |i| i + 1);
        let gone: BTreeSet<ObjectId> = self.corpses[creatures..]
            .iter()
            .flatten()
            .copied()
            .collect();
        let objects = (0..self.objects.len())
            .rposition(|i| !gone.contains(&ObjectId(i as u16)))
            .map_or(0, |i| i + 1);
        let groups: &[Vec<&str>] = &[
            self.terrain.iter().map(|t| t.id.as_str()).collect(),
            (self.creatures[..creatures].iter())
                .map(|c| c.id.as_str())
                .collect(),
            (self.levels.keys())
                .filter(|id| ours(self.level_campaign(id)))
                .map(String::as_str)
                .collect(),
            self.lights.iter().map(|l| l.id.as_str()).collect(),
            (self.dungeons.values())
                .filter(|d| ours(d.campaign.as_deref()))
                .map(|d| d.id.as_str())
                .collect(),
            (self.objects[..objects].iter())
                .map(|o| o.id.as_str())
                .collect(),
            self.slots.iter().map(|s| s.id.as_str()).collect(),
            self.skills.list.iter().map(|s| s.id.as_str()).collect(),
            (self.room_effects.defs().iter())
                .map(|e| e.id.as_str())
                .collect(),
            (self.weapon_skills.skills().iter())
                .map(|s| s.id.as_str())
                .collect(),
            (self.weapon_abilities.iter())
                .map(|a| a.id.as_str())
                .collect(),
            self.statuses.iter().map(|s| s.id.as_str()).collect(),
            // A circled status sheds its school's light (`data/circles.toml`).
            (self.circles.rows().iter())
                .flat_map(|r| [self.status(r.status).id.as_str(), r.school.id()])
                .collect(),
            self.diseases.iter().map(|d| d.id.as_str()).collect(),
            self.traps.iter().map(|t| t.id.as_str()).collect(),
            self.damage_types.iter().map(|t| t.id.as_str()).collect(),
            self.resources.iter().map(|r| r.id.as_str()).collect(),
            self.spells.iter().map(|s| s.id.as_str()).collect(),
            (self.shops.iter().flat_map(|s| s.defs()))
                .map(|d| d.id.as_str())
                .collect(),
        ];
        for ids in groups {
            for id in ids {
                feed(id.as_bytes());
                feed(&[0]);
            }
            // Group separator, so an id cannot migrate between groups unnoticed.
            feed(&[1]);
        }
        hash
    }
}
