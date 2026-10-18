//! What grows and lies on the land of a local area, by landscape: ground cover in patches (green
//! hollows, tall grass, meadows of flowers), woods from copses to thick forest, undergrowth, lone
//! trees, crags and boulders, standing stones, and the pools and reeds of a swamp.
//!
//! Everything comes in blobs of smooth noise (`noise.rs`) sized as a share of the area, so a
//! meadow has meadows rather than speckle. How much of the land is wooded blends towards the
//! neighboring overworld cells near each side, so neighboring areas meet.

use super::Setting;
use super::land::{Claim, Land};
use super::noise::{self, Quantiles};
use super::reach;
use crate::content::{Landscape, TerrainId};
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::rng::Rng;

/// Permille of the land under woods, by landscape.
pub(super) fn wood_share(landscape: Landscape) -> i32 {
    match landscape {
        Landscape::Plains => 30,
        Landscape::Grassland => 110,
        Landscape::Forest => 700,
        Landscape::DenseForest => 840,
        Landscape::Hills => 60,
        Landscape::Swamp => 90,
        Landscape::Road | Landscape::River | Landscape::Lake => 0,
    }
}

/// The wooded share around `p`: the area's own, drawn towards each land neighbor's within the
/// third of the area nearest it (halfway at the edge; corners count half).
fn wood_share_at(land: &Land, setting: &Setting, p: Pos) -> i32 {
    let (w, h) = (land.width() - 1, land.height() - 1);
    let u = p.x * 1000 / w.max(1);
    let v = p.y * 1000 / h.max(1);
    let mut total = 1000 * wood_share(setting.landscape);
    let mut weight = 1000;
    for &(dir, landscape) in &setting.neighbors {
        let (dx, dy) = dir.delta();
        let near_x = match dx {
            -1 => 1000 - u,
            1 => u,
            _ => 1000,
        };
        let near_y = match dy {
            -1 => 1000 - v,
            1 => v,
            _ => 1000,
        };
        // 1000 at the side (or corner), 0 a third of the way in.
        let near = (near_x.min(near_y) - 667).max(0) * 3;
        let near = if dx != 0 && dy != 0 { near / 2 } else { near };
        total += near * wood_share(landscape);
        weight += near;
    }
    total / weight
}

/// Open ground an obstacle may take: free, walkable, and not water.
fn open(land: &Land, p: Pos) -> bool {
    land.may_block(p) && land.is_walkable(p) && !land.is_water(p)
}

/// Covers a `share` (permille) of the map, in blobs of noise with `octaves`, with `t` wherever
/// the terrain is one of `on` and ground cover may go.
fn patches(
    land: &mut Land,
    rng: &mut Rng,
    octaves: &[(i32, i32)],
    share: i32,
    t: TerrainId,
    on: &[TerrainId],
) {
    let field = noise::field(rng, land.width(), land.height(), octaves);
    let cut = Quantiles::of(&field).cut(share);
    for p in land.cells() {
        if field[p] > cut && land.may_cover(p) && land.get(p).is_some_and(|g| on.contains(&g)) {
            land.set(p, t);
        }
    }
}

/// Per-cell jitter on the clumping of trees, so clumps have ragged edges.
const JITTER: i32 = 140;

/// Noise octaves of woods in broad stretches.
const BROAD: &[(i32, i32)] = &[(8, 3), (3, 1)];

/// How the woods of a landscape grow.
struct Woods {
    /// Permille of the cells deep in the woods that hold a tree.
    trees: i32,
    /// Permille of the trees that are pines (in stands of their own).
    pines: i32,
    /// Percent of the open cells in the woods that are bushes.
    undergrowth: u32,
}

/// How deep into the woods each cell lies: positive inside, negative in the open (the share of
/// woods blends towards the neighbors near the sides).
fn wood_depth(land: &Land, setting: &Setting, rng: &mut Rng, octaves: &[(i32, i32)]) -> Grid<i32> {
    let wood = noise::field(rng, land.width(), land.height(), octaves);
    let q = Quantiles::of(&wood);
    let mut depth = Grid::new(land.width(), land.height(), 0);
    for p in land.cells() {
        depth[p] = wood[p] - q.cut(wood_share_at(land, setting, p));
    }
    depth
}

/// Grass in the clearings: wherever the woods are at least `margin` away.
fn clearings(land: &mut Land, depth: &Grid<i32>, margin: i32) {
    for p in land.cells() {
        if depth[p] < -margin && land.may_cover(p) && land.get(p) == Some(land.ground) {
            land.set(p, land.kit.grass);
        }
    }
}

/// Trees in the woods: clumped by a fine noise, thinning out towards the edge of the woods, with
/// undergrowth between them and the odd bush just outside.
fn plant_woods(land: &mut Land, rng: &mut Rng, depth: &Grid<i32>, woods: &Woods) {
    let (w, h) = (land.width(), land.height());
    // Small clumps with ragged edges: fine noise plus a little jitter per cell.
    let mut clumps = noise::field(rng, w, h, &[(2, 1), (1, 1)]);
    for p in land.cells() {
        clumps[p] += rng.range(-JITTER, JITTER);
    }
    let tree_cut = Quantiles::of(&clumps).cut(woods.trees);
    let species = noise::field(rng, w, h, &[(5, 2), (2, 1)]);
    let pine_cut = Quantiles::of(&species).cut(woods.pines);
    for p in land.cells() {
        if !open(land, p) {
            continue;
        }
        let d = depth[p];
        if d <= 0 {
            if d > -20 && rng.chance(10) {
                land.set(p, land.kit.bush);
            }
            continue;
        }
        // The edge of the woods (the first 40 points of depth) is thinner.
        let thin = (40 - d).max(0) * 5;
        if clumps[p] - thin > tree_cut {
            let t = if species[p] > pine_cut {
                land.kit.pine
            } else {
                land.kit.tree
            };
            land.set(p, t);
        } else if rng.chance(woods.undergrowth) {
            land.set(p, land.kit.bush);
        }
    }
}

/// Places `count` of `t` on open ground, no two closer than `gap` columns (rows count double).
fn scatter(land: &mut Land, rng: &mut Rng, count: i32, gap: i32, t: TerrainId) {
    let (w, h) = (land.width(), land.height());
    let mut placed: Vec<Pos> = Vec::new();
    let mut tries = 0;
    while (placed.len() as i32) < count && tries < count * 40 {
        tries += 1;
        let p = Pos::new(rng.range(1, w - 2), rng.range(1, h - 2));
        let close = |q: &Pos| {
            let (dx, dy) = (q.x - p.x, 2 * (q.y - p.y));
            dx * dx + dy * dy < gap * gap
        };
        if !open(land, p) || placed.iter().any(close) {
            continue;
        }
        land.set(p, t);
        placed.push(p);
    }
}

/// Grows the land of the area's landscape, and reeds along the shores of still water.
pub(super) fn grow(land: &mut Land, setting: &Setting, rng: &mut Rng) {
    match setting.landscape {
        Landscape::Plains | Landscape::Road | Landscape::River | Landscape::Lake => {
            plains(land, setting, rng);
        }
        Landscape::Grassland => grassland(land, setting, rng),
        Landscape::Forest => forest(land, setting, rng),
        Landscape::DenseForest => dense_forest(land, setting, rng),
        Landscape::Hills => hills(land, setting, rng),
        Landscape::Swamp => swamp(land, setting, rng),
    }
    if setting.landscape != Landscape::Swamp {
        shores(land, rng);
    }
}

/// Beds of reeds here and there along the shores of ponds and lakes, in the shallows and on the
/// bank.
fn shores(land: &mut Land, rng: &mut Rng) {
    let k = land.kit;
    let beds = noise::field(rng, land.width(), land.height(), &[(2, 2), (1, 1)]);
    let cut = Quantiles::of(&beds).cut(300);
    let mut reeds = Vec::new();
    for p in land.cells() {
        if beds[p] <= cut {
            continue;
        }
        let t = land.get(p);
        let bank = t.is_some_and(|t| {
            let def = land.content.terrain(t);
            def.walkable && def.move_cost <= 100 && t != k.road && t != k.bridge
        }) && land.may_cover(p);
        // Within 2 columns and a row of still water.
        let near = (-1..=1)
            .any(|dy| (-2..=2).any(|dx| land.get(Pos::new(p.x + dx, p.y + dy)) == Some(k.water)));
        let shallows = t == Some(k.water)
            && Dir::ALL.iter().any(|d| {
                land.get(p.step(*d)).is_some_and(|q| {
                    q != k.water && q != k.reeds && land.content.terrain(q).walkable
                })
            });
        if (bank && near) || shallows {
            reeds.push(p);
        }
    }
    for p in reeds {
        land.set(p, k.reeds);
    }
}

/// Dry grass with greener hollows, tall grass, a copse or a lone tree, bushes, a few boulders.
fn plains(land: &mut Land, setting: &Setting, rng: &mut Rng) {
    let k = land.kit;
    let ground = land.ground;
    let depth = wood_depth(land, setting, rng, BROAD);
    patches(land, rng, &[(6, 2), (3, 1)], 280, k.grass, &[ground]);
    patches(
        land,
        rng,
        &[(3, 2), (1, 1)],
        110,
        k.tall_grass,
        &[ground, k.grass],
    );
    patches(land, rng, &[(2, 1)], 25, k.buttercups, &[k.grass]);
    let woods = Woods {
        trees: 650,
        pines: 150,
        undergrowth: 15,
    };
    plant_woods(land, rng, &depth, &woods);
    let n = rng.range(1, 4);
    scatter(land, rng, n, 10, k.tree);
    let n = rng.range(6, 14);
    scatter(land, rng, n, 4, k.bush);
    let n = rng.range(1, 5);
    scatter(land, rng, n, 8, k.boulder);
}

/// Green meadows: tall grass, wildflowers and buttercups, copses with bushes at their edges, lone
/// trees.
fn grassland(land: &mut Land, setting: &Setting, rng: &mut Rng) {
    let k = land.kit;
    let ground = land.ground;
    let depth = wood_depth(land, setting, rng, BROAD);
    patches(land, rng, &[(5, 2), (2, 1)], 220, k.tall_grass, &[ground]);
    patches(
        land,
        rng,
        &[(3, 2), (1, 1)],
        45,
        k.flowers,
        &[ground, k.tall_grass],
    );
    patches(
        land,
        rng,
        &[(3, 2), (1, 1)],
        35,
        k.buttercups,
        &[ground, k.tall_grass],
    );
    let woods = Woods {
        trees: 700,
        pines: 100,
        undergrowth: 20,
    };
    plant_woods(land, rng, &depth, &woods);
    let n = rng.range(3, 7);
    scatter(land, rng, n, 8, k.tree);
    let n = rng.range(4, 10);
    scatter(land, rng, n, 5, k.bush);
    let n = rng.range(0, 2);
    scatter(land, rng, n, 10, k.boulder);
}

/// Woods of leafy trees and stands of pines, with undergrowth and bluebells; grassy clearings.
fn forest(land: &mut Land, setting: &Setting, rng: &mut Rng) {
    let k = land.kit;
    let ground = land.ground;
    let depth = wood_depth(land, setting, rng, &[(7, 3), (3, 1)]);
    clearings(land, &depth, 40);
    patches(land, rng, &[(3, 2), (1, 1)], 60, k.flowers, &[k.grass]);
    patches(land, rng, &[(3, 2), (1, 1)], 40, k.bluebells, &[ground]);
    let woods = Woods {
        trees: 600,
        pines: 300,
        undergrowth: 16,
    };
    plant_woods(land, rng, &depth, &woods);
    let n = rng.range(1, 4);
    scatter(land, rng, n, 10, k.boulder);
}

/// Thick woods, mostly pines, crossed by game trails from the middle out to every side.
fn dense_forest(land: &mut Land, setting: &Setting, rng: &mut Rng) {
    let k = land.kit;
    let ground = land.ground;
    let depth = wood_depth(land, setting, rng, &[(4, 2), (2, 1)]);
    clearings(land, &depth, 30);
    patches(land, rng, &[(3, 2), (1, 1)], 25, k.bluebells, &[ground]);
    let woods = Woods {
        trees: 760,
        pines: 620,
        undergrowth: 24,
    };
    plant_woods(land, rng, &depth, &woods);
    let n = rng.range(1, 3);
    scatter(land, rng, n, 10, k.boulder);
    let from = land.center();
    for side in [Dir::N, Dir::E, Dir::S, Dir::W] {
        reach::trail(land, rng, from, side);
    }
}

/// Rocky slopes with grassy patches, crags with boulders fallen around them, a few pines, and
/// now and then a standing stone.
fn hills(land: &mut Land, setting: &Setting, rng: &mut Rng) {
    let k = land.kit;
    let ground = land.ground;
    let depth = wood_depth(land, setting, rng, BROAD);
    patches(land, rng, &[(6, 2), (3, 1)], 380, k.grass, &[ground]);
    patches(land, rng, &[(3, 2), (1, 1)], 60, k.tall_grass, &[k.grass]);
    // Crags in clusters, with fallen boulders around them.
    let crags = noise::field(rng, land.width(), land.height(), &[(4, 2), (2, 1)]);
    let cut = Quantiles::of(&crags).cut(80);
    for p in land.cells() {
        if crags[p] > cut && open(land, p) {
            land.set(p, k.crag);
        }
    }
    for p in land.cells() {
        if open(land, p) && land.count_around(p, k.crag) > 0 && rng.chance(18) {
            land.set(p, k.boulder);
        }
    }
    let woods = Woods {
        trees: 600,
        pines: 700,
        undergrowth: 10,
    };
    plant_woods(land, rng, &depth, &woods);
    let n = rng.range(5, 10);
    scatter(land, rng, n, 6, k.boulder);
    let n = rng.range(3, 7);
    scatter(land, rng, n, 6, k.bush);
    if rng.chance(35) {
        let n = rng.range(1, 2);
        scatter(land, rng, n, 16, k.standing_stone);
    }
}

/// Marsh with pools of still water fringed with reeds, grassy hummocks, willows, dead trees.
fn swamp(land: &mut Land, setting: &Setting, rng: &mut Rng) {
    let k = land.kit;
    let ground = land.ground;
    let depth = wood_depth(land, setting, rng, BROAD);
    let pools = noise::field(rng, land.width(), land.height(), &[(5, 2), (2, 1)]);
    let cut = Quantiles::of(&pools).cut(260);
    for p in land.cells() {
        if pools[p] > cut && land.may_block(p) {
            land.set(p, k.water);
            land.set_claim(p, Claim::Fixed);
        }
    }
    for p in land.cells() {
        let wet = land.count_around(p, k.water) > 0;
        if wet && land.may_cover(p) && land.get(p) == Some(ground) && rng.chance(25) {
            land.set(p, k.reeds);
        }
    }
    patches(land, rng, &[(3, 2), (1, 1)], 50, k.reeds, &[ground]);
    patches(land, rng, &[(4, 2), (2, 1)], 120, k.grass, &[ground]);
    let woods = Woods {
        trees: 520,
        pines: 0,
        undergrowth: 12,
    };
    plant_woods(land, rng, &depth, &woods);
    let n = rng.range(5, 12);
    scatter(land, rng, n, 5, k.dead_tree);
    let n = rng.range(2, 5);
    scatter(land, rng, n, 6, k.bush);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn woods_blend_towards_the_neighbors_at_the_sides() {
        let content = crate::content::Content::builtin().expect("builtin content");
        let kit = super::super::Kit::resolve(&content).expect("kit");
        let land = Land::new(&content, kit, kit.grass, 80, 40);
        let setting = Setting {
            landscape: Landscape::Forest,
            ground: kit.grass,
            roads: Vec::new(),
            rivers: Vec::new(),
            lakes: Vec::new(),
            neighbors: vec![(Dir::E, Landscape::Plains), (Dir::SE, Landscape::Plains)],
        };
        let at = |x, y| wood_share_at(&land, &setting, Pos::new(x, y));
        let (own, plains) = (wood_share(Landscape::Forest), wood_share(Landscape::Plains));
        // The area's own share in the middle and away from the plains.
        assert_eq!(at(40, 20), own);
        assert_eq!(at(0, 39), own);
        // Halfway at the side, a little more open still in the corner (the corner counts half).
        assert!(
            (at(79, 20) - (own + plains) / 2).abs() <= 1,
            "{}",
            at(79, 20)
        );
        assert!(at(79, 39) < at(79, 20));
        // Thinning out towards the plains.
        let row: Vec<i32> = (40..80).map(|x| at(x, 20)).collect();
        assert!(row.windows(2).all(|w| w[1] <= w[0]), "{row:?}");
    }
}
