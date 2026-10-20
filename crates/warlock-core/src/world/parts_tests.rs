//! Objects made of parts (spec §7 "Objects and materials", #199): every built-in object's parts
//! are valid, the main part is what a single material was, names show the gems and precious
//! metals, descriptions every part, before identification too, and value follows the parts'
//! materials by their shares. Items carry their base item's parts (instances of their own come
//! with metal variants and gems, #193 and #191), so loot, creature kits, and saves keep them.

use super::*;
use crate::content::{ContentFiles, ObjectId, Part};
use crate::magic::describe::lines;
use crate::magic::{Magic, Rarity};

fn content() -> Arc<Content> {
    Arc::new(Content::builtin().expect("built-in content"))
}

fn object(content: &Content, id: &str) -> ObjectId {
    content
        .object_id(id)
        .unwrap_or_else(|| panic!("object {id}"))
}

fn part(content: &Content, part: &str, material: &str, share: u8) -> Part {
    Part {
        part: Some(content.part_id(part).expect("part")),
        material: content.material_id(material).expect("material"),
        share,
    }
}

/// Every part of `stack` checks out against its base item: the same parts, valid shares, the
/// main part first, and a name and value made of them.
fn check_stack(content: &Content, stack: &Stack) {
    let def = content.object(stack.object);
    let parts = stack.parts(content);
    assert_eq!(parts, def.parts.as_slice(), "{}", def.id);
    assert!(!parts.is_empty() && parts.len() <= crate::content::MAX_OBJECT_PARTS);
    assert_eq!(parts.iter().map(|p| u32::from(p.share)).sum::<u32>(), 100);
    assert!(parts.iter().all(|p| p.share >= 1), "{}", def.id);
    assert!(parts[1..].iter().all(|p| p.part.is_some()), "{}", def.id);
    assert_eq!(def.material(), parts[0].material);
    assert_eq!(
        stack.value(content),
        u64::from(def.value) * u64::from(stack.count)
    );
    let name = if stack.count == 1 {
        &def.name
    } else {
        &def.plural
    };
    assert!(stack.describe(content).contains(name), "{}", def.id);
}

#[test]
fn every_object_is_made_of_valid_parts_the_main_one_first() {
    let content = content();
    for (i, def) in content.object_defs().iter().enumerate() {
        check_stack(&content, &Stack::one(ObjectId(i as u16)));
        assert_eq!(def.value_with(&def.parts, &content), def.value);
    }
    let gold = content.material_id("gold").expect("gold");
    let amulet = content.object(object(&content, "gold_amulet"));
    assert_eq!(amulet.material(), gold, "the chain is the main part");
    assert_eq!(
        (amulet.parts[0].share, amulet.parts[1].share),
        (90, 10),
        "the main part takes what the others leave"
    );
    // Corpses are of one part, the corpses' material.
    let corpse = content.object(object(&content, "kobold_corpse"));
    assert_eq!(
        corpse.parts,
        [Part::whole(content.material_id("flesh").unwrap())]
    );
}

#[test]
fn names_show_gems_and_precious_metals_descriptions_every_part() {
    let content = content();
    let name = |id: &str| {
        let s = Stack::one(object(&content, id));
        (s.describe(&content), lines(&s, &content)[0].text.clone())
    };
    assert_eq!(
        name("gold_amulet"),
        (
            "a gold and ruby amulet".into(),
            "Gold and ruby amulet · gold chain, ruby setting · 90 g".into()
        )
    );
    // The leather is plain: only the description shows it.
    assert_eq!(
        name("amber_amulet"),
        (
            "an amber amulet".into(),
            "Amber amulet · amber pendant, leather cord · 60 g".into()
        )
    );
    assert_eq!(
        name("long_sword"),
        (
            "a long sword".into(),
            "Long sword · steel blade, leather grip · 1.8 kg".into()
        )
    );
    // A material the name says already is not said again; one part is just its material.
    assert_eq!(name("ruby"), ("a ruby".into(), "Ruby · ruby · 5 g".into()));
    assert_eq!(name("gold_ring").0, "a gold ring");
    let rubies = Stack::new(object(&content, "ruby"), 3);
    assert_eq!(rubies.describe(&content), "3 rubies");
    let amulets = Stack::new(object(&content, "gold_amulet"), 2);
    assert_eq!(amulets.describe(&content), "2 gold and ruby amulets");
}

#[test]
fn materials_show_before_the_item_is_identified() {
    let content = content();
    let mut w = World::new(content.clone(), "showcase", None).expect("world");
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 4,
        affixes: vec![crate::magic::Affix {
            id: "of_the_bear".into(),
            tier: 1,
            value: 1,
        }],
        name: None,
        identified: false,
    };
    let amulet = Stack::with_magic(object(&content, "gold_amulet"), 1, magic);
    let player = w.state.player;
    (w.state.actors.get_mut(&player).expect("player").inventory).push(amulet);
    w.apply(&Action::Wait);
    let seen = (w.perception().inventory().iter())
        .find(|s| s.object == object(&content, "gold_amulet"))
        .expect("carried")
        .clone();
    assert_eq!(seen.describe(&content), "a gold and ruby amulet (magic)");
    let text: Vec<String> = lines(&seen, &content).into_iter().map(|l| l.text).collect();
    assert_eq!(
        text[0],
        "Magic gold and ruby amulet · gold chain, ruby setting · 90 g"
    );
    assert!(
        text.iter().any(|t| t.starts_with("Unidentified")),
        "{text:#?}"
    );
}

#[test]
fn value_follows_the_parts_materials_by_their_shares() {
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/materials/gold.toml",
        "[[material]]\nid = \"gold\"\nname = \"gold\"\nprecious = true\nvalue = 2000\n",
    );
    files.insert(
        "data/materials/silver.toml",
        "[[material]]\nid = \"silver\"\nname = \"silver\"\nprecious = true\nvalue = 1500\n",
    );
    files.insert(
        "data/materials/ruby.toml",
        "[[material]]\nid = \"ruby\"\nname = \"ruby\"\nprecious = true\nvalue = 5000\n",
    );
    let content = Content::load(&files).expect("content");
    let amulet = content.object(object(&content, "gold_amulet"));
    assert_eq!(amulet.value, 200);
    // Made of its own parts it is worth its value, whatever its materials are worth.
    assert_eq!(amulet.value_with(&amulet.parts, &content), 200);
    // A silver chain: 200 × (90 × 1500 + 10 × 5000) / (90 × 2000 + 10 × 5000) = 160.9.
    let silver = [
        part(&content, "chain", "silver", 90),
        part(&content, "setting", "ruby", 10),
    ];
    assert_eq!(amulet.value_with(&silver, &content), 161);
    // A steel chain (100) is worth far less; a larger ruby more.
    let steel = [
        part(&content, "chain", "steel", 90),
        part(&content, "setting", "ruby", 10),
    ];
    assert_eq!(amulet.value_with(&steel, &content), 51);
    let ruby = [
        part(&content, "chain", "gold", 70),
        part(&content, "setting", "ruby", 30),
    ];
    assert_eq!(amulet.value_with(&ruby, &content), 252);
    // Shops price a stack by its value: its base item's parts, its value.
    let stack = Stack::new(object(&content, "gold_amulet"), 3);
    assert_eq!(stack.value(&content), 600);
    let shops = content.shops().expect("shops");
    let store = shops.id("general_store").expect("store");
    let dagger = Stack::one(object(&content, "dagger"));
    assert_eq!(
        shops.price(&content, store, &dagger, 0),
        content.object(dagger.object).value * 150 / 100
    );
}

#[test]
fn loot_and_creature_kits_are_made_of_their_base_items_parts() {
    let content = content();
    let mut seen = 0;
    for seed in [1, 7, 42] {
        for depth in 1..=6 {
            let id = format!("depths:{depth}");
            let level = crate::mapgen::generate(&content, seed, &id, None).expect("level");
            let floor = level.objects.values().flatten();
            let kits = level.creatures.iter().flat_map(|c| c.carry.iter());
            for stack in floor.chain(kits) {
                check_stack(&content, stack);
                seen += 1;
            }
        }
    }
    assert!(seen > 100, "{seen} stacks");
    // Creature drops too.
    let w = World::new(content.clone(), "arena", None).expect("world");
    for actor in w.state.actors.values() {
        actor
            .inventory
            .iter()
            .for_each(|s| check_stack(&content, s));
        for n in 0..50 {
            let mut a = actor.clone();
            a.id = ActorId(1000 + n);
            if let Some(drop) = w.loot_of(&a) {
                check_stack(&content, &drop);
            }
        }
    }
}

#[test]
fn objects_of_several_parts_survive_saves_and_replays() {
    let content = content();
    let mut w = World::new(content.clone(), "showcase", None).expect("world");
    let player = w.state.player;
    let pack = &mut w.state.actors.get_mut(&player).expect("player").inventory;
    pack.push(Stack::one(object(&content, "gold_amulet")));
    pack.push(Stack::one(object(&content, "long_sword")));
    let state = w.state().clone();
    for action in crate::action::parse_actions("wait drop:0 pickup").expect("actions") {
        w.apply(&action);
    }
    let names = |w: &World| -> Vec<String> {
        (w.perception().inventory().iter())
            .map(|s| s.describe(&content))
            .collect()
    };
    let before = names(&w);
    assert!(
        before.contains(&"a gold and ruby amulet".to_string()),
        "{before:?}"
    );
    let bytes = crate::save::encode(&w);
    let back = crate::save::decode(&bytes, content.clone()).expect("loads");
    assert_eq!(back.state(), w.state());
    assert_eq!(back.state_hash(), w.state_hash());
    assert_eq!(names(&back), before);
    let mut again = World::from_state(content.clone(), state).expect("world");
    for action in crate::action::parse_actions("wait drop:0 pickup").expect("actions") {
        again.apply(&action);
    }
    assert_eq!(again.state_hash(), w.state_hash());
}
