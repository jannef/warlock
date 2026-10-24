//! Items made of another material than their base item (spec §7 "Materials", #193, #194): the
//! materials' properties and construction lists are checked as content loads; a variant weighs,
//! strikes, protects, and is worth what its material makes it, is drawn in its colour, and is
//! named "[ornament] [material] base"; it is placed by level files, refused off its base item's
//! list, and survives saves and replays.

use super::*;
use crate::action::parse_actions;
use crate::color::Rgb;
use crate::content::{ContentFiles, ObjectId, PartRole};
use crate::item_parts::ItemPart;
use crate::magic::describe::lines;
use crate::magic::{Affix, LootGrade, Magic, Rarity, loot};
use crate::object::Stack;
use crate::variant::Source;

// A lit room: the player beside a bronze long sword, and a pile of a tin chain mail shirt on a
// steel dagger.
const LEVEL: &str = r##"
name = "Variants"
ambient = [255, 255, 255, 0]
map = '''
########
#@sm...#
########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"s" = { terrain = "floor", objects = ["long_sword:bronze"] }
"m" = { terrain = "floor", objects = ["chain_mail:tin", "dagger"] }
"##;

/// Materials on steel's scale, as #193 and #194 give them.
const MATERIALS: &[(&str, &str)] = &[
    (
        "steel",
        "group = \"metal\"\ncolor = \"#a8b0b8\"\nto_hit = 0\ndamage = 0\nprotection = 100",
    ),
    (
        "iron",
        "group = \"metal\"\ncolor = \"#7a7d80\"\nto_hit = -1\ndamage = 0\nprotection = 90\n\
         value = 60",
    ),
    (
        "bronze",
        "group = \"metal\"\ncolor = \"#cd7f32\"\nweight = 112\ndamage = -1\nprotection = 80\n\
         value = 80",
    ),
    (
        "tin",
        "group = \"metal\"\ncolor = \"#c9ccce\"\nweight = 93\ndamage = -2\nprotection = 40\n\
         value = 40",
    ),
    (
        "gold",
        "precious = true\ngroup = \"metal\"\ncolor = \"#ffd24a\"\nweight = 246\nto_hit = -2\n\
         damage = 1\nprotection = 50\nvalue = 2000",
    ),
    (
        "silver",
        "precious = true\ngroup = \"metal\"\ncolor = \"#d9dee3\"\nweight = 134\nto_hit = -1\n\
         damage = -1\nprotection = 50\nvalue = 1500",
    ),
    (
        "leather",
        "group = \"hide\"\ncolor = \"#8b5a2b\"\nweight = 35\nprotection = 30\nvalue = 50",
    ),
    (
        "dragonscale",
        "group = \"hide\"\ncolor = \"#3fa36b\"\nweight = 40\nprotection = 130\nvalue = 3000",
    ),
    (
        "bone",
        "group = \"bone\"\ncolor = \"#e3dac9\"\nweight = 60\ndamage = -1\nprotection = 50\n\
         value = 30",
    ),
];

/// How often loot and creatures' gear are made of them.
const FOUND: &[(&str, &str)] = &[
    (
        "steel",
        "rarity = { weight = 300, deeper = 60 }\ngear = { weight = 20, deeper = 15 }",
    ),
    ("iron", "rarity = { weight = 300 }\ngear = { weight = 300 }"),
    (
        "bronze",
        "rarity = { weight = 800, deeper = -40 }\ngear = { weight = 600, deeper = -25 }",
    ),
    (
        "tin",
        "rarity = { weight = 300, deeper = -20 }\ngear = { weight = 200 }",
    ),
    ("gold", "rarity = { weight = 10 }\ngear = { weight = 5 }"),
    ("silver", "rarity = { weight = 10 }\ngear = { weight = 5 }"),
    (
        "leather",
        "rarity = { weight = 800 }\ngear = { weight = 800 }",
    ),
    ("dragonscale", "rarity = { weight = 10, level = 10 }"),
    ("bone", "rarity = { weight = 300 }\ngear = { weight = 600 }"),
];

/// Built-in content with [`MATERIALS`] as often found as [`FOUND`] says, these construction
/// lists, and the level.
fn test_files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    for (id, fields) in MATERIALS {
        let found = (FOUND.iter()).find(|(m, _)| m == id).map_or("", |(_, f)| f);
        files.insert(
            &format!("data/materials/{id}.toml"),
            &format!("[[material]]\nid = \"{id}\"\nname = \"{id}\"\n{fields}\n{found}\n"),
        );
    }
    for (id, list) in [
        ("long_sword", "[\"metal\"]"),
        ("dagger", "[\"metal\", \"bone\"]"),
        ("chain_mail", "[\"metal\"]"),
        ("leather_jerkin", "[\"hide\"]"),
        ("iron_shod_boots", "[\"metal\"]"),
        ("gold_ring", "[\"any\"]"),
        ("iron_helm", "[\"metal\", \"hide\", \"bone\"]"),
    ] {
        made_of(&mut files, id, list);
    }
    files.insert("levels/variants.toml", LEVEL);
    files.insert(
        "scenarios/variants.toml",
        "name = \"Variants\"\nstart_level = \"variants\"\n",
    );
    files
}

/// `id`'s object file with its main part's construction list `list` (whatever it had).
fn made_of(files: &mut ContentFiles, id: &str, list: &str) {
    let path = format!("data/objects/{id}.toml");
    let text = (files.table("objects"))
        .find(|(p, _)| *p == path)
        .map(|(_, t)| t.to_string())
        .unwrap_or_else(|| panic!("{path}"));
    let kept: Vec<&str> = text.lines().filter(|l| !l.starts_with("made_of")).collect();
    files.insert(&path, &format!("{}\nmade_of = {list}\n", kept.join("\n")));
}

fn content() -> Arc<Content> {
    Arc::new(Content::load(&test_files()).expect("test content is valid"))
}

fn object(content: &Content, id: &str) -> ObjectId {
    content
        .object_id(id)
        .unwrap_or_else(|| panic!("object {id}"))
}

/// `id` made of `material` (its main part).
fn variant(content: &Content, id: &str, material: &str) -> Stack {
    let m = content.material_id(material).expect(material);
    Stack::one(object(content, id)).with_parts(vec![ItemPart::material(content, None, m)])
}

#[test]
fn materials_check_their_properties() {
    let cases = [
        ("weight = 0", "`weight`"),
        ("damage = 11", "`to_hit` and `damage`"),
        ("protection = 600", "`protection`"),
        ("group = \"any\"", "`group`"),
        ("rarity = { weight = 10 }", "only a material of a `group`"),
        (
            "group = \"metal\"\nrarity = { weight = 10, level = 0 }",
            "`level` is 1 or more",
        ),
        ("color = \"red\"", "color"),
    ];
    for (fields, expected) in cases {
        let mut files = ContentFiles::builtin();
        files.insert(
            "data/materials/odd.toml",
            &format!("[[material]]\nid = \"odd\"\nname = \"odd\"\n{fields}\n"),
        );
        let err = Content::load(&files).expect_err(expected);
        assert!(
            err.message.contains(expected),
            "{expected}: {}",
            err.message
        );
    }
    // A gem is set, never made into anything: it has no group.
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/materials/ruby.toml",
        "[[material]]\nid = \"ruby\"\nname = \"ruby\"\nprecious = true\ngroup = \"stone\"\n\
         gem = { color = \"#d3213b\", value = 300, rarity = 5 }\n",
    );
    let err = Content::load(&files).expect_err("a gem with a group");
    assert!(err.message.contains("never what one is made of"), "{err}");
    // Found weights: at their level and deeper, never below 0.
    let found = crate::content::Found {
        weight: 80,
        deeper: -10,
        level: 3,
    };
    assert_eq!(
        [1, 2, 3, 5, 9, 20].map(|l| found.at(l)),
        [0, 0, 60, 40, 0, 0]
    );
}

#[test]
fn construction_lists_name_groups_and_materials_and_keep_their_own() {
    let content = content();
    let id = |m: &str| content.material_id(m).expect(m);
    let sword = content.object(object(&content, "long_sword"));
    // Groups, sorted by id; the grip has no list: only its own leather.
    let metals = ["bronze", "gold", "iron", "silver", "steel", "tin"].map(id);
    assert_eq!(sword.made_of(0), metals);
    assert_eq!(sword.made_of(1), [id("leather")]);
    assert_eq!(sword.role(), PartRole::Weapon);
    // Groups keep what the main part can be: no leather blade, no flint mail; any material of
    // a group for jewellery.
    let helm = content.object(object(&content, "iron_helm"));
    assert_eq!(helm.role(), PartRole::Armour);
    assert!(helm.made_of(0).contains(&id("leather")));
    let ring = content.object(object(&content, "gold_ring"));
    assert!(ring.made_of(0).contains(&id("leather")) && ring.made_of(0).contains(&id("bone")));
    assert!(
        !ring.made_of(0).contains(&id("paper")),
        "paper has no group"
    );
    // Objects without a list are made only of their own materials.
    let rock = content.object(object(&content, "rock"));
    assert_eq!(rock.made_of(0), [rock.material()]);
    // What a list may not say.
    let cases = [
        (
            "long_sword",
            "[\"bronze\"]",
            "must allow what the part is made of",
        ),
        (
            "long_sword",
            "[\"metal\", \"leather\"]",
            "cannot be a weapon's working part",
        ),
        (
            "chain_mail",
            "[\"metal\", \"bone\", \"paper\"]",
            "cannot be armour",
        ),
        (
            "long_sword",
            "[\"metal\", \"marble\"]",
            "neither a material's `group` nor",
        ),
        ("long_sword", "[\"metal\", \"ruby\"]", "is a gem"),
        ("long_sword", "[]", "one group or material at least"),
    ];
    for (object, list, expected) in cases {
        let mut files = test_files();
        made_of(&mut files, object, list);
        let err = Content::load(&files).expect_err(expected);
        assert!(
            err.message.contains(expected),
            "{expected}: {}",
            err.message
        );
    }
    // Given twice for the main part: in its table and as the object's.
    let mut files = test_files();
    let path = "data/objects/long_sword.toml";
    let text = (files.table("objects"))
        .find(|(p, _)| *p == path)
        .map(|(_, t)| {
            t.replace(
                "of = \"steel\" }",
                "of = \"steel\", made_of = [\"metal\"] }",
            )
        })
        .expect("long sword");
    files.insert(path, &text);
    let err = Content::load(&files).expect_err("twice");
    assert!(err.message.contains("given twice"), "{err}");
    // In its table alone it is the same list.
    let kept: Vec<&str> = text.lines().filter(|l| !l.starts_with("made_of")).collect();
    files.insert(path, &kept.join("\n"));
    let content = Content::load(&files).expect("the table's list");
    let sword = content.object(content.object_id("long_sword").expect("sword"));
    assert_eq!(sword.made_of(0).len(), 6);
}

#[test]
fn a_variant_weighs_strikes_protects_and_costs_by_its_material() {
    let content = content();
    // A bronze long sword: its blade (90%) 112% as heavy, 1800 g × (90 × 1.12 + 10) / 100.
    let sword = variant(&content, "long_sword", "bronze");
    assert_eq!(sword.weight(&content), 1994);
    // Worth 80 × (90 × 80 + 10 × 50) / (90 × 100 + 10 × 50) = 64.8.
    assert_eq!(sword.value(&content), 65);
    let def = content.object(sword.object);
    assert_eq!(
        crate::variant::weapon(def, &sword.parts(&content), &content),
        (0, -1)
    );
    // Gold: heavier, -2 to hit, +1 damage, worth twenty times steel.
    let gold = variant(&content, "long_sword", "gold");
    assert_eq!(
        crate::variant::weapon(def, &gold.parts(&content), &content),
        (-2, 1)
    );
    assert_eq!(gold.weight(&content), 4165);
    // Armour keeps its Protection by the materials' ratio, rounded: chain mail is iron (90%).
    let mail = |m: &str| variant(&content, "chain_mail", m).base_protection(&content);
    assert_eq!([mail("steel"), mail("tin"), mail("gold")], [3, 1, 2]);
    let jerkin = variant(&content, "leather_jerkin", "dragonscale");
    assert_eq!(jerkin.base_protection(&content), 4);
    // Jewellery: only its worth changes. A leather ring weighs what the gold one does.
    let ring = variant(&content, "gold_ring", "leather");
    assert_eq!(
        ring.weight(&content),
        u64::from(content.object(ring.object).weight)
    );
    assert_eq!(ring.value(&content), 4); // 150 × 50 / 2000 = 3.75
    // Made of its own materials, an item is its base item.
    let plain = Stack::one(object(&content, "long_sword"));
    assert_eq!(plain.weight(&content), 1800);
    assert_eq!(plain.value(&content), 80);
}

#[test]
fn a_variant_is_named_and_drawn_by_its_material() {
    let content = content();
    let name = |s: &Stack| s.describe(&content);
    assert_eq!(
        name(&variant(&content, "long_sword", "bronze")),
        "a bronze long sword"
    );
    assert_eq!(
        name(&variant(&content, "chain_mail", "tin")),
        "a tin chain mail shirt"
    );
    // The material's word in the name gives way: "iron-shod", "gold ring", "leather jerkin".
    assert_eq!(
        name(&variant(&content, "iron_shod_boots", "bronze")),
        "a pair of bronze-shod boots"
    );
    assert_eq!(
        name(&variant(&content, "gold_ring", "silver")),
        "a silver ring"
    );
    assert_eq!(
        name(&variant(&content, "leather_jerkin", "dragonscale")),
        "a dragonscale jerkin"
    );
    assert_eq!(name(&variant(&content, "iron_helm", "bone")), "a bone helm");
    // [affix prefix] [ornament] [material] base [affix suffix].
    let emerald = content.material_id("emerald").expect("emerald");
    let gems = content.gems().expect("gems");
    let pommel = ItemPart::gem(
        &content,
        Some(content.part_id("pommel").expect("pommel")),
        emerald,
        gems.plain(),
    );
    let bronze = ItemPart::material(
        &content,
        None,
        content.material_id("bronze").expect("bronze"),
    );
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 3,
        affixes: vec![
            Affix {
                id: "sharp".into(),
                tier: 1,
                value: 1,
            },
            Affix {
                id: "of_the_bear".into(),
                tier: 1,
                value: 1,
            },
        ],
        name: None,
        identified: true,
    };
    let sword = Stack::with_magic(object(&content, "long_sword"), 1, magic)
        .with_parts(vec![bronze.clone(), pommel.clone()]);
    assert_eq!(
        name(&sword),
        "a Sharp emerald-pommelled bronze long sword of the Bear"
    );
    // A precious material is named once: a gold variant, a gold and ruby amulet's silver one.
    assert_eq!(
        name(&variant(&content, "long_sword", "gold")),
        "a gold long sword"
    );
    // Drawn in its material's colour; a ring set with a gem keeps its gem's.
    let bronze_color = Rgb::new(0xcd, 0x7f, 0x32);
    assert_eq!(
        variant(&content, "long_sword", "bronze").fg(&content),
        bronze_color
    );
    let plain = Stack::one(object(&content, "long_sword"));
    assert_eq!(plain.fg(&content), content.object(plain.object).fg);
    let setting = ItemPart::gem(
        &content,
        Some(content.part_id("setting").expect("setting")),
        emerald,
        gems.plain(),
    );
    let silver = content.material_id("silver").expect("silver");
    let ring = Stack::one(object(&content, "gold_ring"))
        .with_parts(vec![ItemPart::material(&content, None, silver), setting]);
    assert_eq!(ring.describe(&content), "a silver and emerald ring");
    assert_eq!(
        ring.fg(&content),
        content.material(emerald).gem.as_ref().expect("gem").color
    );
    // The description: what it is made of, its weight, and its damage with the material's.
    let text: Vec<String> = lines(&variant(&content, "long_sword", "bronze"), &content)
        .into_iter()
        .map(|l| l.text)
        .collect();
    assert_eq!(
        text[0],
        "Bronze long sword · bronze blade, leather grip · 2 kg"
    );
    assert!(
        text.iter().any(|t| t.starts_with("Damage 1d8-1;")),
        "{text:#?}"
    );
    let text: Vec<String> = lines(&variant(&content, "chain_mail", "tin"), &content)
        .into_iter()
        .map(|l| l.text)
        .collect();
    assert!(
        text.iter().any(|t| t.starts_with("Protection 1 (")),
        "{text:#?}"
    );
}

#[test]
fn only_a_base_items_list_makes_a_variant() {
    let content = content();
    let check = |id: &str, part: ItemPart| {
        crate::item_parts::check(content.object(object(&content, id)), &[part], &content)
    };
    let material = |m: &str| content.material_id(m).expect(m);
    let main = |m: &str| ItemPart::material(&content, None, material(m));
    assert!(check("long_sword", main("bronze")).is_ok());
    let off_list = check("long_sword", main("bone")).expect_err("bone");
    assert!(
        off_list.contains("not what `long_sword` may be made of"),
        "{off_list}"
    );
    let same = check("long_sword", main("steel")).expect_err("steel");
    assert!(same.contains("made of that anyway"), "{same}");
    let rock = check("rock", main("iron")).expect_err("no list");
    assert!(rock.contains("may be made of"), "{rock}");
    let added = ItemPart::material(
        &content,
        Some(content.part_id("pommel").expect("pommel")),
        material("bronze"),
    );
    let err = check("long_sword", added).expect_err("an added pommel of bronze");
    assert!(err.contains("only a gem is added"), "{err}");
    // A grip has only its own leather.
    let grip = ItemPart::material(
        &content,
        Some(content.part_id("grip").expect("grip")),
        material("dragonscale"),
    );
    assert!(check("long_sword", grip).is_err());
    // Placements name a material; one off the list is a content error.
    let mut files = test_files();
    files.insert(
        "levels/variants.toml",
        &LEVEL.replace("long_sword:bronze", "long_sword:bone"),
    );
    let err = Content::load(&files).expect_err("a bone long sword");
    assert!(err.message.contains("long_sword:bone"), "{err}");
    let mut files = test_files();
    files.insert(
        "levels/variants.toml",
        &format!(
            "{LEVEL}\n[[item]]\nat = [2, 1]\nobject = \"dagger\"\nparts = [{{ of = \"bone\" }}]\n"
        ),
    );
    let content = Content::load(&files).expect("an [[item]] of bone");
    let level = content.level("variants").expect("level");
    let pile = &level.objects[&Pos::new(2, 1)];
    let names: Vec<String> = pile.iter().map(|s| s.describe(&content)).collect();
    assert!(names.contains(&"a bone dagger".to_string()), "{names:?}");
}

#[test]
fn variants_fight_with_their_material() {
    let content = content();
    let mut w = World::new(content.clone(), "variants", None).expect("world");
    let plain = w.state.player().melee(&content);
    for action in parse_actions("move:E pickup equip:main_hand:a").expect("actions") {
        w.apply(&action);
    }
    let player = w.state.player();
    assert_eq!(
        player.inventory[0].describe(&content),
        "a bronze long sword"
    );
    let bronze = player.melee(&content);
    // A steel long sword against a bronze one: -1 damage, the same to hit.
    let mut steel = w.clone();
    let pack = &mut steel
        .state
        .actors
        .get_mut(&steel.state.player)
        .expect("player")
        .inventory;
    pack[0] = Stack::one(object(&content, "long_sword"));
    let steel = steel.state.player().melee(&content);
    assert_eq!(bronze.to_hit, steel.to_hit);
    assert_eq!(bronze.bonus, steel.bonus - 1);
    assert_ne!(bronze, plain);
}

#[test]
fn variants_survive_saves_and_replays() {
    let content = content();
    let mut w = World::new(content.clone(), "variants", None).expect("world");
    let state = w.state().clone();
    let actions = "move:E pickup move:E pickup:a equip:body:b";
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
    let names = |w: &World| -> Vec<String> {
        (w.perception().inventory().iter())
            .map(|s| s.describe(&content))
            .collect()
    };
    let before = names(&w);
    assert_eq!(before, ["a bronze long sword", "a tin chain mail shirt"]);
    let bytes = crate::save::encode(&w);
    let back = crate::save::decode(&bytes, content.clone()).expect("loads");
    assert_eq!(back.state(), w.state());
    assert_eq!(back.state_hash(), w.state_hash());
    assert_eq!(names(&back), before);
    // The same actions from the start give the same state.
    let mut again = World::from_state(content.clone(), state).expect("world");
    for action in parse_actions(actions).expect("actions") {
        again.apply(&action);
    }
    assert_eq!(again.state_hash(), w.state_hash());
    // A save whose variant its base item may not be made of does not load.
    let mut odd = w.clone();
    let pack = &mut odd
        .state
        .actors
        .get_mut(&odd.state.player)
        .expect("player")
        .inventory;
    pack[0] = variant(&content, "long_sword", "bronze");
    assert!(pack[0].is_valid_for(&content));
    let bone = ItemPart::material(&content, None, content.material_id("bone").expect("bone"));
    pack[0] = Stack::one(object(&content, "long_sword")).with_parts(vec![bone]);
    assert!(!pack[0].is_valid_for(&content));
}

#[test]
fn loot_and_gear_draw_materials_by_how_often_they_are_found() {
    let content = content();
    let id = |m: &str| content.material_id(m).expect(m);
    let sword = content.object(object(&content, "long_sword"));
    let weights = |source, level| crate::variant::weights(&content, sword, source, level);
    // By the construction list's order (material ids), each by its rarity at the item level.
    assert_eq!(
        weights(Source::Loot, 1),
        [
            (id("bronze"), 800),
            (id("gold"), 10),
            (id("iron"), 300),
            (id("silver"), 10),
            (id("steel"), 300),
            (id("tin"), 300),
        ]
    );
    // Deeper: fewer primitive metals, more steel.
    let deep = weights(Source::Loot, 11);
    assert!(deep.contains(&(id("bronze"), 400)) && deep.contains(&(id("steel"), 900)));
    assert!(deep.contains(&(id("tin"), 100)));
    // A creature's gear by its own level, whatever the item level.
    let gear = weights(Source::Gear(1), 20);
    assert!(gear.contains(&(id("steel"), 20)) && gear.contains(&(id("bronze"), 600)));
    // Below its level a material is not found: dragonscale from item level 10.
    let jerkin = content.object(object(&content, "leather_jerkin"));
    let hides = |level| crate::variant::weights(&content, jerkin, Source::Loot, level);
    assert!(hides(9).contains(&(id("dragonscale"), 0)));
    assert!(hides(10).contains(&(id("dragonscale"), 10)));
    // Nothing to draw for an object made only of its own material.
    let rock = content.object(object(&content, "rock"));
    assert!(crate::variant::weights(&content, rock, Source::Loot, 5).is_empty());
}

#[test]
fn worse_but_dearer_is_very_rare_except_in_jewellery() {
    // Gold as common as bronze: still very rare on a sword (worse to hit than steel, dearer)
    // and mail (thinner), but not on a ring (worth only) or a helm of iron for damage.
    let mut files = test_files();
    files.insert(
        "data/materials/gold.toml",
        "[[material]]\nid = \"gold\"\nname = \"gold\"\nprecious = true\ngroup = \"metal\"\n\
         to_hit = -2\ndamage = 1\nprotection = 50\nvalue = 2000\nrarity = { weight = 800 }\n",
    );
    let content = Content::load(&files).expect("content");
    let gold = content.material_id("gold").expect("gold");
    let weight_of = |id: &str| {
        let def = content.object(content.object_id(id).expect(id));
        let weights = crate::variant::weights(&content, def, Source::Loot, 1);
        weights.iter().find(|(m, _)| *m == gold).expect("gold").1
    };
    assert_eq!(weight_of("long_sword"), crate::content::VERY_RARE);
    assert_eq!(weight_of("chain_mail"), crate::content::VERY_RARE);
    assert_eq!(weight_of("gold_ring"), 800, "a ring is worth only");
    // Better and dearer is not capped: steel on an iron chain mail shirt.
    let mail = content.object(content.object_id("chain_mail").expect("mail"));
    let (iron, steel) = (
        content.material(content.material_id("iron").expect("iron")),
        content.material(content.material_id("steel").expect("steel")),
    );
    assert!(!crate::variant::worse_but_dearer(mail, iron, steel));
    assert!(crate::variant::worse_but_dearer(
        mail,
        steel,
        content.material(gold)
    ));
}

#[test]
fn loot_rolls_variants_from_its_list_deterministically() {
    let content = content();
    let roll_many = |seed: u64, level: u8| -> Vec<Stack> {
        let mut rng = crate::rng::Rng::stream(seed, "test/variants");
        (0..3000)
            .filter_map(|_| loot::roll(&content, &mut rng, level, LootGrade::Any))
            .collect()
    };
    let shallow = roll_many(1, 1);
    assert_eq!(shallow, roll_many(1, 1), "the same stream, the same loot");
    let sword = object(&content, "long_sword");
    let swords = |found: &[Stack]| -> Vec<String> {
        (found.iter())
            .filter(|s| s.object == sword)
            .map(|s| s.parts(&content)[0].material)
            .map(|m| content.material(m).id.clone())
            .collect()
    };
    let count = |list: &[String], m: &str| list.iter().filter(|x| *x == m).count();
    let shallow_swords = swords(&shallow);
    assert!(shallow_swords.len() > 50, "{}", shallow_swords.len());
    // Shallow, bronze outnumbers steel; deep, steel outnumbers bronze.
    assert!(count(&shallow_swords, "bronze") > count(&shallow_swords, "steel"));
    let deep_swords = swords(&roll_many(2, 15));
    assert!(count(&deep_swords, "steel") > count(&deep_swords, "bronze"));
    // Every variant is on its base item's list, and named so (a loose gem is its gem).
    for stack in &shallow {
        assert!(stack.is_valid_for(&content), "{stack:?}");
        if stack.gem(&content).is_some() {
            continue;
        }
        let def = content.object(stack.object);
        let main = stack.parts(&content)[0].material;
        assert!(def.made_of(0).contains(&main), "{}", def.id);
        if main != def.material() {
            let name = stack.describe(&content);
            assert!(name.contains(&content.material(main).name), "{name}");
        }
    }
}
