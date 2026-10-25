//! Stamping [vaults](crate::content::Vault) into generated levels.

use super::Canvas;
use crate::content::{CreatureSpawn, Vault};
use crate::geom::{Dir, Pos};
use crate::rng::Rng;

/// The content's dungeon vaults in a random order, but those a branch level names
/// ([`Vault::pinned`]): they are stamped only where named.
pub(super) fn shuffled<'c>(canvas: &Canvas<'c>, rng: &mut Rng) -> Vec<&'c Vault> {
    let mut vaults: Vec<&Vault> = canvas
        .content
        .vaults()
        .iter()
        .filter(|v| !v.is_outdoor() && !v.pinned)
        .collect();
    rng.shuffle(&mut vaults);
    vaults
}

/// The vaults `names` (a branch level's `vaults`, checked at load), in that order.
pub(super) fn named<'c>(canvas: &Canvas<'c>, names: &[String]) -> Vec<&'c Vault> {
    let vaults = canvas.content.vaults();
    (names.iter())
        .filter_map(|n| vaults.iter().find(|v| v.id() == n))
        .collect()
}

/// Every top-left corner where `vault` [`fits`], in row order.
pub(super) fn spots(canvas: &Canvas, vault: &Vault) -> Vec<Pos> {
    let (max_x, max_y) = (
        canvas.width() - 2 - vault.width(),
        canvas.height() - 2 - vault.height(),
    );
    (2..=max_y)
        .flat_map(|y| (2..=max_x).map(move |x| Pos::new(x, y)))
        .filter(|at| fits(canvas, vault, *at))
        .collect()
}

/// Whether `vault` with its top-left corner at `at` lies on untouched rock, with a ring of rock
/// around it, and away from the map's outer edge (so every entrance opens onto diggable rock).
pub(super) fn fits(canvas: &Canvas, vault: &Vault, at: Pos) -> bool {
    (at.y - 1..=at.y + vault.height()).all(|y| {
        (at.x - 1..=at.x + vault.width()).all(|x| {
            let p = Pos::new(x, y);
            canvas.is_inner(p) && canvas.get(p) == canvas.pal.void && !canvas.is_stamped(p)
        })
    })
}

/// Copies `vault` with its top-left corner at `at` (terrain, placed lights, the flow of running
/// water, objects, traps, locks on doors, secret doors, and creatures with their patrols) and
/// marks its cells stamped. Returns its entrances in level coordinates, each with the direction
/// leading out.
pub(super) fn stamp(canvas: &mut Canvas, vault: &Vault, at: Pos) -> Vec<(Pos, Dir)> {
    let offset = |p: Pos| Pos::new(at.x + p.x, at.y + p.y);
    let template = &vault.template;
    for (p, t) in template.terrain.iter() {
        canvas.set(offset(p), *t);
        canvas.stamped.set(offset(p), true);
    }
    for (p, light) in &template.lights {
        canvas.lights.insert(offset(*p), *light);
    }
    for (p, flow) in &template.flow {
        canvas.flow.insert(offset(*p), *flow);
    }
    for (p, pile) in &template.objects {
        canvas.objects.insert(offset(*p), pile.clone());
    }
    for (p, trap) in &template.traps {
        canvas.traps.insert(offset(*p), *trap);
    }
    for &(p, grade) in &template.loot {
        canvas.loot.push((offset(p), grade));
    }
    for (p, lock) in &template.locks {
        canvas.locks.insert(offset(*p), lock.clone());
    }
    for (p, secret) in &template.secrets {
        canvas.secrets.insert(offset(*p), *secret);
    }
    for spawn in &template.creatures {
        canvas.creatures.push(CreatureSpawn {
            pos: offset(spawn.pos),
            ..spawn.clone()
        });
    }
    vault
        .entrances
        .iter()
        .map(|(p, out)| (offset(*p), *out))
        .collect()
}
