//! The checks of an endless branch's layer bosses (spec §7 "The endless branch",
//! `world/layer_boss.rs`): each boss a unique type, each lair a dungeon vault of the branch's
//! campaign (or shared) that places its boss once and marks the hidden stairs, `>`, and stamps
//! nowhere else (it is pinned); no other vault has a `>`.

use super::{Content, CreatureId, DungeonDef};

impl Content {
    /// Checks branch `def`'s layer bosses and pins their lairs.
    pub(super) fn pin_lairs(&mut self, def: &DungeonDef) -> Result<(), String> {
        for boss in &def.bosses {
            let creature = self.creature(boss.creature);
            let id = creature.id.clone();
            if !creature.unique {
                return Err(format!("layer boss `{id}` is not `unique`"));
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
            if vault.down.is_none() {
                return Err(format!(
                    "layer boss `{id}`: lair `{name}` marks no stairs down (`>`)"
                ));
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

    /// Whether creature type `c` is some endless branch's layer boss: it keeps to its lair
    /// (`world/stair_followers.rs`).
    pub fn is_layer_boss(&self, c: CreatureId) -> bool {
        (self.dungeons.values()).any(|d| d.bosses.iter().any(|b| b.creature == c))
    }
}
