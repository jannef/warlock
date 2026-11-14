//! The checks of a branch's bosses (spec §7 "The endless branch", "The Pits";
//! `world/levels/layer_boss.rs`, `world/levels/pit.rs`): an endless branch's layer bosses, each a unique type
//! that leaves an everlasting, profane corpse, and a portal branch's warden, of any type (every Pit has its own); each lair a dungeon vault of
//! the branch's campaign (or shared) that places its boss once, stamps nowhere else (it is
//! pinned), and marks the hidden stairs, `>`, exactly where the floor leads down (a Pit's last
//! floor leads nowhere, and its lair has none); no other vault has a `>`.

use super::{Content, CreatureId, DungeonDef};

impl Content {
    /// Checks branch `def`'s layer bosses and pins their lairs.
    pub(super) fn pin_lairs(&mut self, def: &DungeonDef) -> Result<(), String> {
        for boss in &def.bosses {
            let creature = self.creature(boss.creature);
            let id = creature.id.clone();
            if def.portal.is_none() && !creature.unique {
                return Err(format!("layer boss `{id}` is not `unique`"));
            }
            // Every keeper of an endless branch leaves a corpse that never rots
            // (docs/design/profane-gifts.md §3).
            if def.portal.is_none() && !(creature.corpse && creature.everlasting) {
                return Err(format!(
                    "layer boss `{id}` must leave an everlasting corpse (`corpse` and `everlasting`)"
                ));
            }
            // ... and a profane one: its signature gift and its draws (§5, §7.1).
            let profane = (creature.on_eat.as_ref())
                .is_some_and(|e| e.gives.is_some() && e.profane.is_some());
            if def.portal.is_none() && !profane {
                return Err(format!(
                    "layer boss `{id}` must leave a profane corpse (`on_eat` with `gives` and \
                     `profane`)"
                ));
            }
            let name = &boss.vault;
            let vault = (self.vaults.iter_mut())
                .find(|v| v.id() == name)
                .ok_or_else(|| format!("layer boss `{id}`: unknown vault `{name}`"))?;
            if vault.is_outdoor() || (vault.campaign.is_some() && vault.campaign != def.campaign) {
                return Err(format!(
                    "layer boss `{id}`: vault `{name}` is no dungeon vault of the branch's campaign"
                ));
            }
            match (def.leads_down(boss.depth), vault.down.is_some()) {
                (true, false) => {
                    return Err(format!(
                        "layer boss `{id}`: lair `{name}` marks no stairs down (`>`)"
                    ));
                }
                (false, true) => {
                    return Err(format!(
                        "layer boss `{id}`: lair `{name}` marks stairs down (`>`) on a floor \
                         that leads nowhere"
                    ));
                }
                _ => {}
            }
            if vault.danger.is_given() {
                return Err(format!(
                    "layer boss `{id}`: lair `{name}` has a `danger`: a lair is stamped for certain"
                ));
            }
            let placed = (vault.template.creatures.iter())
                .filter(|c| c.creature == boss.creature)
                .count();
            if placed != 1 {
                return Err(format!(
                    "layer boss `{id}`: lair `{name}` places it {placed} times, not once"
                ));
            }
            vault.pinned = true;
        }
        Ok(())
    }

    /// Checks, once every branch is loaded, that only lairs mark stairs.
    pub(super) fn check_lairs(&self) -> Result<(), String> {
        let lairs: Vec<&str> = (self.dungeons.values())
            .flat_map(|d| d.bosses.iter().map(|b| b.vault.as_str()))
            .collect();
        match (self.vaults.iter()).find(|v| v.down.is_some() && !lairs.contains(&v.id())) {
            Some(v) => Err(format!(
                "vault `{}`: a vault cannot have stairs, but a layer boss's lair",
                v.id()
            )),
            None => Ok(()),
        }
    }

    /// Whether creature type `c` is some branch's boss (a layer boss, a Pit's warden): it keeps
    /// to its lair
    /// (`world/levels/stair_followers.rs`).
    pub fn is_layer_boss(&self, c: CreatureId) -> bool {
        (self.dungeons.values()).any(|d| d.bosses.iter().any(|b| b.creature == c))
    }
}

#[cfg(test)]
impl super::ContentFiles {
    /// Every shared vault removed but the Pits' warden's hall, which Hellegeat's boss row names:
    /// for fixtures that want no vault in their way.
    pub(crate) fn remove_shared_vaults(&mut self) {
        self.files
            .retain(|p, _| !p.starts_with("vaults/") || p == "vaults/wardens_hall.toml");
    }
}
