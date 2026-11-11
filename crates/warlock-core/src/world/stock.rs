//! Material stock in the world (docs/design/professions.md §3, §4; docs/dev/objects.md "Stock"):
//! the profession that works a material, as `data/stock.toml` names it. Salvage and recipes read
//! it; nothing in code names a group or a skill.

use super::World;
use crate::content::MaterialId;
use crate::skills::SkillId;
use crate::stats::Stat;

impl World {
    /// The skill that works `material` and the stat its checks are made by: the gems' profession
    /// for a gem, else its group's; none for a material no profession takes.
    pub fn profession_of(&self, material: MaterialId) -> Option<(SkillId, Stat)> {
        let p = self
            .content
            .stock()
            .profession_of(&self.content, material)?;
        Some((p.skill, p.stat))
    }
}

#[cfg(test)]
mod tests {
    use crate::content::Content;
    use crate::geom::Pos;
    use crate::object::Stack;
    use crate::stats::Stat;
    use crate::world::World;
    use std::sync::Arc;

    fn town() -> World {
        let content = Arc::new(Content::builtin().expect("content"));
        World::new(content, "ceaptun", None).expect("world")
    }

    #[test]
    fn a_material_is_worked_by_its_groups_profession() {
        let world = town();
        let of = |id: &str| {
            let m = world.content.material_id(id).expect(id);
            (world.profession_of(m)).map(|(s, stat)| (world.content.skill(s).id.clone(), stat))
        };
        assert_eq!(of("iron"), Some(("metalcraft".into(), Stat::Strength)));
        assert_eq!(of("ironwood"), Some(("woodcraft".into(), Stat::Dexterity)));
        assert_eq!(of("fur"), Some(("woodcraft".into(), Stat::Dexterity)));
        assert_eq!(of("silk"), None);
    }

    /// What lies at `(x, y)` on the current level, described.
    fn lying(world: &World, x: i32, y: i32) -> Vec<String> {
        (world.state.current().objects_at(Pos::new(x, y)).iter())
            .map(|s| s.describe(&world.content))
            .collect()
    }

    #[test]
    fn the_town_sells_stock_and_a_save_keeps_it() {
        let world = town();
        // Placed as "piece*6" and "piece:iron*6".
        assert_eq!(lying(&world, 36, 5), ["6 steel ingots"]);
        assert_eq!(lying(&world, 36, 6), ["6 iron ingots"]);
        let bytes = crate::save::encode(&world);
        let loaded = crate::save::decode(&bytes, world.content.clone()).expect("loads");
        assert_eq!(lying(&loaded, 36, 6), ["6 iron ingots"]);
        assert_eq!(loaded.state_hash(), world.state_hash());
    }

    #[test]
    fn smiths_and_stores_trade_in_stock() {
        let content = Content::builtin().expect("content");
        let shops = content.shops().expect("shops");
        let piece = content.object_id(crate::piece::ID).expect("piece");
        let iron = content.material_id("iron").expect("iron");
        let ingots: Stack = crate::piece::make(&content, iron, 3).expect("ingots");
        let kind = |id: &str| shops.id(id).expect(id);
        for id in ["smithy", "general_store", "town_store"] {
            assert!(shops.stocks(&content, kind(id), piece), "{id} stocks it");
        }
        for id in [
            "smithy",
            "weaponsmith",
            "armourer",
            "fletcher",
            "general_store",
            "town_store",
        ] {
            assert!(shops.buys(&content, kind(id), &ingots), "{id} buys it");
        }
        for id in ["alchemist", "provisioner"] {
            assert!(!shops.buys(&content, kind(id), &ingots), "{id} does not");
        }
        // Worth its material: an iron ingot a coin.
        assert_eq!(ingots.value(&content), 3);
    }
}
