//! The terrain and creatures the outdoor generators build with, looked up by id once.

use crate::content::{Content, CreatureId, TerrainId};

/// Terrain and creature ids for local areas. Content with an overworld must provide them (checked
/// when content loads).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Kit {
    /// The open ground of outdoor vaults (it takes the area's own ground).
    pub void: TerrainId,
    pub grass: TerrainId,
    pub tall_grass: TerrainId,
    pub flowers: TerrainId,
    pub buttercups: TerrainId,
    pub bluebells: TerrainId,
    pub bush: TerrainId,
    pub reeds: TerrainId,
    pub tree: TerrainId,
    pub pine: TerrainId,
    pub dead_tree: TerrainId,
    pub boulder: TerrainId,
    pub crag: TerrainId,
    pub standing_stone: TerrainId,
    /// Still water: ponds, pools, lake shores.
    pub water: TerrainId,
    /// Running water: rivers and brooks.
    pub running_water: TerrainId,
    pub road: TerrainId,
    /// Where a road crosses water.
    pub bridge: TerrainId,
    pub deer: CreatureId,
}

impl Kit {
    /// Looks up the ids and checks what the generators rely on.
    pub(crate) fn resolve(content: &Content) -> Result<Self, String> {
        let terrain = |id: &str| {
            content
                .terrain_id(id)
                .ok_or_else(|| format!("local areas need terrain `{id}`"))
        };
        let kit = Kit {
            void: terrain("void")?,
            grass: terrain("grass")?,
            tall_grass: terrain("tall_grass")?,
            flowers: terrain("flowers")?,
            buttercups: terrain("buttercups")?,
            bluebells: terrain("bluebells")?,
            bush: terrain("bush")?,
            reeds: terrain("reeds")?,
            tree: terrain("tree")?,
            pine: terrain("pine")?,
            dead_tree: terrain("dead_tree")?,
            boulder: terrain("boulder")?,
            crag: terrain("crag")?,
            standing_stone: terrain("standing_stone")?,
            water: terrain("water")?,
            running_water: terrain("running_water")?,
            road: terrain("dirt_road")?,
            bridge: terrain("bridge")?,
            deer: content
                .creature_id("deer")
                .ok_or("local areas need creature `deer`")?,
        };
        let def = |t: TerrainId| content.terrain(t);
        let walkable = [
            kit.grass,
            kit.tall_grass,
            kit.flowers,
            kit.buttercups,
            kit.bluebells,
            kit.bush,
            kit.reeds,
            kit.water,
            kit.running_water,
            kit.road,
            kit.bridge,
        ];
        let solid = [
            kit.tree,
            kit.pine,
            kit.dead_tree,
            kit.boulder,
            kit.crag,
            kit.standing_stone,
        ];
        let checks = [
            (
                walkable.iter().all(|t| def(*t).walkable),
                "ground, plants, water, roads, and bridges must be walkable",
            ),
            (
                solid.iter().all(|t| !def(*t).walkable),
                "trees, boulders, crags, and standing stones must not be walkable",
            ),
            (
                def(kit.running_water).flows() && !def(kit.water).flows(),
                "`running_water` must flow and `water` must be still",
            ),
            (
                !def(kit.bridge).flows() && def(kit.bridge).stairs.is_none(),
                "`bridge` must be plain ground",
            ),
        ];
        match checks.iter().find(|(ok, _)| !ok) {
            Some((_, problem)) => Err(format!("local areas: {problem}")),
            None => Ok(kit),
        }
    }
}
