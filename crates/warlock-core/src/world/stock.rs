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
    use crate::object::{Stack, add_to_inventory};
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
        assert!(!shops.buys(&content, kind("provisioner"), &ingots));
        // Worth its material: an iron ingot a coin.
        assert_eq!(ingots.value(&content), 3);
    }

    /// The herbs and the essences (docs/design/recipes.md §8, §9): weight and worth.
    const HERBS: [(&str, &str, u64); 9] = [
        ("glowcap", "glowcap", 5),
        ("moonwort", "sprig of moonwort", 5),
        ("comfrey", "sprig of comfrey", 8),
        ("feverfew", "sprig of feverfew", 15),
        ("henbane", "sprig of henbane", 15),
        ("eyebright", "sprig of eyebright", 25),
        ("bog_myrtle", "sprig of bog myrtle", 25),
        ("vervain", "sprig of vervain", 25),
        ("mandrake", "mandrake root", 60),
    ];
    const ELEMENTS: [&str; 6] = ["fire", "ice", "lightning", "air", "water", "earth"];

    #[test]
    fn herbs_and_essences_are_materials_objects() {
        let content = Content::builtin().expect("content");
        let one = |id: &str| Stack::one(content.object_id(id).expect(id));
        let essences: Vec<String> = ELEMENTS.iter().map(|e| format!("essence_of_{e}")).collect();
        let all = (HERBS.iter().map(|&(id, _, value)| (id, 20, value)))
            .chain(essences.iter().map(|id| (id.as_str(), 50, 30)));
        for (id, grams, value) in all {
            let stack = one(id);
            let def = content.object(stack.object);
            assert_eq!(content.item_class(def.class).id, "materials", "{id}");
            assert!(def.stackable && def.kind.is_none(), "{id}: never loot");
            assert_eq!(
                (stack.weight(&content), stack.value(&content)),
                (grams, value),
                "{id}"
            );
            let mut pack = Vec::new();
            add_to_inventory(&mut pack, stack.clone(), &content);
            add_to_inventory(&mut pack, stack, &content);
            assert_eq!((pack.len(), pack[0].count), (1, 2), "{id} stacks");
        }
        for (id, name, _) in HERBS {
            assert_eq!(content.object(one(id).object).name, name);
        }
        let mut fire = one("essence_of_fire");
        assert_eq!(fire.describe(&content), "an essence of fire");
        fire.count = 3;
        assert_eq!(fire.describe(&content), "3 essences of fire");
        let mut moonwort = one("moonwort");
        moonwort.count = 2;
        assert_eq!(moonwort.describe(&content), "2 sprigs of moonwort");
    }

    #[test]
    fn the_alchemist_sells_herbs_and_buys_materials() {
        let content = Content::builtin().expect("content");
        let shops = content.shops().expect("shops");
        let id = |id: &str| content.object_id(id).expect(id);
        let alchemist = shops.get(shops.id("alchemist").expect("alchemist"));
        for herb in ["glowcap", "moonwort"] {
            assert!(alchemist.always.contains(&id(herb)), "{herb}");
        }
        let kind = shops.id("alchemist").expect("alchemist");
        for bought in ["mandrake", "essence_of_earth"] {
            assert!(
                shops.buys(&content, kind, &Stack::one(id(bought))),
                "{bought}"
            );
        }
        let iron = content.material_id("iron").expect("iron");
        let ingot = crate::piece::make(&content, iron, 1).expect("ingot");
        assert!(shops.buys(&content, kind, &ingot));
        // The general store stocks pieces, never a herb or an essence.
        let store = shops.id("general_store").expect("store");
        for (herb, _, _) in HERBS {
            assert!(!shops.stocks(&content, store, id(herb)), "{herb}");
        }
        for e in ELEMENTS {
            assert!(!shops.stocks(&content, store, id(&format!("essence_of_{e}"))));
        }
    }

    #[test]
    fn a_save_keeps_sinew_and_an_essence() {
        let mut world = town();
        let content = world.content.clone();
        let sinew = content.material_id("sinew").expect("sinew");
        let lengths = crate::piece::make(&content, sinew, 4).expect("sinew");
        let mut essence = Stack::one(content.object_id("essence_of_water").expect("essence"));
        essence.count = 2;
        let me = world.player_mut();
        me.inventory.push(lengths);
        me.inventory.push(essence);
        let bytes = crate::save::encode(&world);
        let loaded = crate::save::decode(&bytes, content.clone()).expect("loads");
        let carried: Vec<String> = (loaded.state.player().inventory.iter().rev().take(2))
            .map(|s| s.describe(&content))
            .collect();
        assert_eq!(carried, ["2 essences of water", "4 lengths of sinew"]);
        assert_eq!(loaded.state_hash(), world.state_hash());
    }
}
