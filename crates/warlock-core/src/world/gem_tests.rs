//! Gems (spec §7 "Gems", #191): the data is checked; a gem is worth its value × size × cut, and
//! an ornament adds its gem's worth to its item; names, descriptions, colours, and weights follow
//! the gems; identical gems stack; loot finds gems and sets items with them, rarely and
//! deterministically, larger and finer deeper; gems survive saves and replays; shops price them
//! by their gem.

use super::*;
use crate::action::{Positions, parse_actions};
use crate::color::Rgb;
use crate::content::{ContentFiles, Cutting, ObjectId, dull};
use crate::item_parts::{GemCut, ItemPart};
use crate::magic::describe::lines;
use crate::magic::{Affix, LootGrade, Magic, Rarity, loot};
use crate::object::Stack;
use crate::replay::Replay;
use crate::rng::Rng;

// A lit room: the player beside a pile with two rubies and a large fine emerald on top.
const LEVEL: &str = r##"
name = "Gems"
ambient = [255, 255, 255, 0]
map = '''
#######
#@g...#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"g" = { terrain = "floor", objects = ["gem:ruby*2"] }

[[item]]
at = [2, 1]
object = "gem"
parts = [{ of = "emerald", size = "large", cut = "fine" }]
"##;

fn content() -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert("levels/gems.toml", LEVEL);
    files.insert(
        "scenarios/gems.toml",
        "name = \"Gems\"\nstart_level = \"gems\"\n",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
}

fn object(content: &Content, id: &str) -> ObjectId {
    content
        .object_id(id)
        .unwrap_or_else(|| panic!("object {id}"))
}

fn cutting(content: &Content, size: &str, cut: &str) -> Cutting {
    let gems = content.gems().expect("gems");
    Cutting {
        size: gems.size_id(size).expect(size),
        cut: gems.cut_id(cut).expect(cut),
    }
}

/// A loose gem.
fn gem(content: &Content, gem: &str, size: &str, cut: &str) -> Stack {
    let gems = content.gems().expect("gems");
    let material = content.material_id(gem).expect(gem);
    let part = ItemPart::gem(content, None, material, cutting(content, size, cut));
    Stack::one(gems.object).with_parts(vec![part])
}

/// `id` set with `gem` as its `part`.
fn set(content: &Content, id: &str, part: &str, gem: &str, size: &str, cut: &str) -> Stack {
    let material = content.material_id(gem).expect(gem);
    let part = Some(content.part_id(part).expect(part));
    let set = ItemPart::gem(content, part, material, cutting(content, size, cut));
    Stack::one(object(content, id)).with_parts(vec![set])
}

#[test]
fn the_ten_gems_their_sizes_and_cuts_are_data() {
    let content = Content::builtin().expect("content");
    let gems = content.gems().expect("gems");
    let mut names: Vec<&str> = (content.materials().iter())
        .filter(|m| m.gem.is_some())
        .map(|m| m.id.as_str())
        .collect();
    names.sort_unstable();
    assert_eq!(
        names,
        [
            "amethyst",
            "diamond",
            "emerald",
            "garnet",
            "heliodor",
            "ruby",
            "sapphire",
            "smokestone",
            "topaz",
            "zircon"
        ]
    );
    assert!(
        content
            .materials()
            .iter()
            .all(|m| m.gem.is_none() || m.precious)
    );
    assert!(
        content.material_id("amber").is_some(),
        "amber stays, as no gem"
    );
    let size: Vec<(&str, u32, u32)> = (gems.sizes().iter())
        .map(|s| (s.id.as_str(), s.value, s.weight))
        .collect();
    assert_eq!(
        size,
        [
            ("tiny", 30, 1),
            ("small", 100, 1),
            ("medium", 400, 2),
            ("large", 1500, 3),
            ("huge", 5000, 4)
        ]
    );
    let cut: Vec<(&str, u32, bool)> = (gems.cuts().iter())
        .map(|c| (c.id.as_str(), c.value, c.rough))
        .collect();
    assert_eq!(
        cut,
        [
            ("uncut", 25, true),
            ("crude", 60, false),
            ("fair", 100, false),
            ("fine", 160, false),
            ("exquisite", 250, false),
            ("flawless", 400, false)
        ]
    );
    assert_eq!(gems.plain(), cutting(&content, "small", "fair"));
    assert_eq!(content.object(gems.object).id, "gem");
}

#[test]
fn bad_gem_data_is_reported() {
    let gems = include_str!("../../../../assets/data/gems.toml");
    let cases = [
        (
            "data/materials/ruby.toml",
            "[[material]]\nid = \"ruby\"\nname = \"ruby\"\ngem = { color = \"#ff0000\", value = 1, \
             rarity = 1 }\n"
                .to_string(),
            "a gem is precious",
        ),
        (
            "data/materials/ruby.toml",
            "[[material]]\nid = \"ruby\"\nname = \"ruby\"\nprecious = true\ngem = { color = \
             \"#ff0000\", value = 0, rarity = 1 }\n"
                .to_string(),
            "`value` is in gold",
        ),
        (
            "data/gems.toml",
            gems.replacen("plain = true", "", 1),
            "exactly one size",
        ),
        (
            "data/gems.toml",
            gems.replace("object = \"gem\"", "object = \"opal\""),
            "no such object",
        ),
        (
            "data/gems.toml",
            gems.replace("object = \"gem\"", "object = \"long_sword\""),
            "no magic item",
        ),
        (
            "data/gems.toml",
            gems.replace("part = \"studs\"", "part = \"facet\""),
            "no part `facet`",
        ),
        (
            "data/gems.toml",
            gems.replace("class = \"helmets\"", "class = \"hats\""),
            "no such item class",
        ),
        (
            "data/gems.toml",
            gems.replace("class = \"helmets\"", "class = \"rings\""),
            "one ornament per class",
        ),
        (
            "data/gems.toml",
            gems.replace("value = 1500", "value = 0"),
            "multiplies a gem's value",
        ),
        (
            "data/gems.toml",
            gems.replace("part = \"setting\"\nchance = 30", "part = \"pendant\"\nchance = 30"),
            "never as its main part",
        ),
    ];
    for (file, text, expected) in cases {
        let mut files = ContentFiles::builtin();
        files.insert(file, &text);
        let err = Content::load(&files).expect_err(expected);
        assert!(
            err.message.contains(expected),
            "{expected}: {}",
            err.message
        );
    }
}

#[test]
fn a_gem_is_worth_its_value_times_its_size_and_cut() {
    let content = Content::builtin().expect("content");
    let gems = content.gems().expect("gems");
    let worth = |g: &str, size: &str, cut: &str| gem(&content, g, size, cut).value(&content);
    // The plain size and cut are ×1 both: a gem's own value.
    assert_eq!(worth("ruby", "small", "fair"), 300);
    assert_eq!(worth("diamond", "small", "fair"), 400);
    // 300 × 15 × 1.6.
    assert_eq!(worth("emerald", "large", "fine"), 7200);
    // 60 × 0.3 × 0.25 = 4.5, rounded to the nearest coin; never below 1.
    assert_eq!(worth("garnet", "tiny", "uncut"), 5);
    assert_eq!(worth("smokestone", "tiny", "uncut"), 2);
    // The most precious: 400 × 50 × 4.
    assert_eq!(worth("diamond", "huge", "flawless"), 80_000);
    // A stack is worth each one times the count.
    let mut rubies = gem(&content, "ruby", "medium", "crude");
    rubies.count = 3;
    assert_eq!(rubies.value(&content), 3 * 720);
    // Every size and cut makes a gem worth more than the one before.
    let ruby = content
        .material(content.material_id("ruby").unwrap())
        .gem
        .clone()
        .unwrap();
    for (i, _) in gems.sizes().iter().enumerate().skip(1) {
        let at = |s: usize| Cutting {
            size: crate::content::GemSizeId(s as u8),
            cut: gems.plain().cut,
        };
        assert!(gems.value(&ruby, at(i)) > gems.value(&ruby, at(i - 1)));
    }
}

#[test]
fn an_ornament_adds_its_gems_worth_to_its_item() {
    let content = Content::builtin().expect("content");
    let sword = content.object(object(&content, "long_sword"));
    let pommelled = set(&content, "long_sword", "pommel", "emerald", "small", "fine");
    // The long sword's own parts are all still there, at their shares; the pommel weighs nothing.
    assert_eq!(pommelled.value(&content), u64::from(sword.value) + 480);
    // A ring's setting is a part it did not have.
    let ring = content.object(object(&content, "gold_ring"));
    let set_ring = set(&content, "gold_ring", "setting", "ruby", "small", "fair");
    assert_eq!(set_ring.value(&content), u64::from(ring.value) + 300);
    // The gold amulet's own ruby setting gives way to a sapphire: the plain parts left (the
    // gold chain, 90% of its weight but nearly all of its materials' worth, gold being worth
    // twenty times the ruby's material) are worth 200 × 90 × 2000 / (90 × 2000 + 10 × 100), and
    // the sapphire its own.
    let amulet = set(
        &content,
        "gold_amulet",
        "setting",
        "sapphire",
        "small",
        "fair",
    );
    assert_eq!(amulet.value(&content), 199 + 280);
    // Weight: an ornament weighs nothing beside its item; a loose gem what its size says.
    assert_eq!(pommelled.weight(&content), u64::from(sword.weight));
    assert_eq!(gem(&content, "ruby", "huge", "fair").weight(&content), 4);
    assert_eq!(gem(&content, "ruby", "tiny", "fair").weight(&content), 1);
}

#[test]
fn loose_gems_are_named_by_their_gem_size_and_cut() {
    let content = Content::builtin().expect("content");
    let name = |g: &str, size: &str, cut: &str| gem(&content, g, size, cut).describe(&content);
    assert_eq!(name("ruby", "small", "fair"), "a ruby");
    assert_eq!(name("emerald", "large", "fine"), "a large fine emerald");
    assert_eq!(name("garnet", "tiny", "uncut"), "an uncut tiny garnet");
    assert_eq!(name("ruby", "small", "uncut"), "an uncut ruby");
    assert_eq!(name("amethyst", "medium", "fair"), "a medium amethyst");
    assert_eq!(name("diamond", "small", "flawless"), "a flawless diamond");
    let mut topazes = gem(&content, "topaz", "medium", "exquisite");
    topazes.count = 3;
    assert_eq!(topazes.describe(&content), "3 medium exquisite topazes");
    let mut rubies = gem(&content, "ruby", "small", "fair");
    rubies.count = 2;
    assert_eq!(rubies.describe(&content), "2 rubies");
    // The description says the size and cut always, and what it weighs.
    let text =
        |s: &Stack| -> Vec<String> { lines(s, &content).into_iter().map(|l| l.text).collect() };
    assert_eq!(
        text(&gem(&content, "garnet", "tiny", "uncut")),
        [
            "Uncut tiny garnet · garnet · 1 g",
            "Gem: tiny garnet, uncut"
        ]
    );
    assert_eq!(
        text(&gem(&content, "ruby", "small", "fair")),
        ["Ruby · ruby · 1 g", "Gem: small ruby, fair cut"]
    );
}

#[test]
fn ornaments_go_in_the_naming_order_and_the_description() {
    let content = Content::builtin().expect("content");
    let sword = || set(&content, "long_sword", "pommel", "emerald", "small", "fine");
    assert_eq!(
        sword().describe(&content),
        "an emerald-pommelled long sword"
    );
    let lines_of =
        |s: &Stack| -> Vec<String> { lines(s, &content).into_iter().map(|l| l.text).collect() };
    let text = lines_of(&sword());
    assert_eq!(
        text[..2],
        [
            "Emerald-pommelled long sword · steel blade, leather grip, emerald pommel · 1.8 kg",
            "Pommel: small emerald, fine cut"
        ]
    );
    // Magic: [prefix] [ornament] base [suffix]; unidentified, its gem shows and its magic not.
    let affixes = vec![
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
    ];
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 1,
        affixes: affixes.clone(),
        name: None,
        identified: true,
        carried: 0,
        unique: None,
    };
    let own = sword().own_parts().expect("its pommel").to_vec();
    let sharp =
        Stack::with_magic(object(&content, "long_sword"), 1, magic.clone()).with_parts(own.clone());
    assert_eq!(
        sharp.describe(&content),
        "a Sharp emerald-pommelled long sword of the Bear"
    );
    assert_eq!(
        sharp.title(&content),
        "Sharp emerald-pommelled long sword of the Bear"
    );
    let unknown = Magic {
        identified: false,
        carried: 0,
        unique: None,
        ..magic.clone()
    };
    let hidden =
        Stack::with_magic(object(&content, "long_sword"), 1, unknown).with_parts(own.clone());
    assert_eq!(
        hidden.describe(&content),
        "an emerald-pommelled long sword (magic)"
    );
    // A rare item keeps its two-word name; what it is made of follows it, and its description.
    let rare = Magic {
        rarity: Rarity::Rare,
        name: Some("Grim Bite".into()),
        ..magic
    };
    let grim = Stack::with_magic(object(&content, "long_sword"), 1, rare).with_parts(own);
    assert_eq!(grim.title(&content), "Grim Bite");
    assert_eq!(
        grim.describe(&content),
        "Grim Bite, an emerald-pommelled long sword"
    );
    assert!(lines_of(&grim).contains(&"Pommel: small emerald, fine cut".to_string()));
    // Each class's part: a setting joins the material, the others name the item.
    let named = |id: &str, part: &str, g: &str| {
        set(&content, id, part, g, "small", "fine").describe(&content)
    };
    assert_eq!(
        named("gold_ring", "setting", "ruby"),
        "a gold and ruby ring"
    );
    assert_eq!(
        named("iron_ring", "setting", "zircon"),
        "an iron and zircon ring"
    );
    assert_eq!(
        named("gold_amulet", "setting", "sapphire"),
        "a gold and sapphire amulet"
    );
    assert_eq!(
        named("quarterstaff", "tip", "topaz"),
        "a topaz-tipped quarterstaff"
    );
    assert_eq!(
        named("iron_helm", "studs", "garnet"),
        "a garnet-studded iron helm"
    );
    assert_eq!(
        named("small_shield", "inlay", "amethyst"),
        "an amethyst-inlaid small shield"
    );
    let mut daggers = set(&content, "dagger", "pommel", "ruby", "small", "fine");
    daggers.count = 1;
    assert_eq!(daggers.names(&content).1, "ruby-pommelled daggers");
}

#[test]
fn gems_colour_their_glyph() {
    let content = Content::builtin().expect("content");
    let color = |g: &str| {
        content
            .material(content.material_id(g).unwrap())
            .gem
            .as_ref()
            .unwrap()
            .color
    };
    let emerald = gem(&content, "emerald", "large", "fine");
    assert_eq!(emerald.fg(&content), color("emerald"));
    assert_eq!(emerald.fg(&content), Rgb::new(0x2e, 0xa0, 0x5a));
    // Uncut is drawn duller.
    let rough = gem(&content, "emerald", "large", "uncut");
    assert_eq!(rough.fg(&content), dull(color("emerald")));
    assert_ne!(rough.fg(&content), emerald.fg(&content));
    // A ring takes its gem's colour; a sword keeps its own.
    let ring = set(&content, "silver_ring", "setting", "ruby", "small", "fine");
    assert_eq!(ring.fg(&content), color("ruby"));
    let sword = set(&content, "long_sword", "pommel", "ruby", "small", "fine");
    assert_eq!(sword.fg(&content), content.object(sword.object).fg);
    let plain = Stack::one(object(&content, "silver_ring"));
    assert_eq!(plain.fg(&content), content.object(plain.object).fg);
}

#[test]
fn only_identical_gems_stack() {
    let content = Content::builtin().expect("content");
    let mut pile = Vec::new();
    let small = gem(&content, "ruby", "small", "fair");
    crate::object::add_to_pile(&mut pile, &[small.clone(), small.clone()], &content);
    assert_eq!(pile.len(), 1);
    assert_eq!(pile[0].count, 2);
    let others = [
        gem(&content, "ruby", "medium", "fair"),
        gem(&content, "ruby", "small", "fine"),
        gem(&content, "garnet", "small", "fair"),
    ];
    crate::object::add_to_pile(&mut pile, &others, &content);
    assert_eq!(pile.len(), 4, "another size, cut, or gem is another stack");
    // Swords set with the same gem still do not stack.
    let sword = set(&content, "long_sword", "pommel", "ruby", "small", "fine");
    let mut pack = Vec::new();
    crate::object::add_to_inventory(&mut pack, sword.clone(), &content);
    crate::object::add_to_inventory(&mut pack, sword, &content);
    assert_eq!(pack.len(), 2);
}

#[test]
fn placements_and_items_name_their_gems() {
    let content = content();
    let level = content.level("gems").expect("level");
    let pile = &level.objects[&Pos::new(2, 1)];
    let names: Vec<String> = pile.iter().map(|s| s.describe(&content)).collect();
    assert_eq!(names, ["a large fine emerald", "2 rubies"]);
    // A loose gem names its gem; nothing else does, and only gems are named.
    let cases = [
        (
            "objects = [\"gem:ruby*2\"]",
            "objects = [\"gem\"]",
            "name its gem",
        ),
        (
            "objects = [\"gem:ruby*2\"]",
            "objects = [\"dagger:ruby\"]",
            "only a loose gem",
        ),
        (
            "objects = [\"gem:ruby*2\"]",
            "objects = [\"gem:steel\"]",
            "is no gem",
        ),
        ("cut = \"fine\"", "cut = \"sparkly\"", "no cut `sparkly`"),
        (
            "of = \"emerald\"",
            "of = \"steel\"",
            "only a gem has a `size`",
        ),
        ("of = \"emerald\"", "of = \"marble\"", "no such material"),
        (
            "of = \"emerald\"",
            "part = \"pommel\", of = \"emerald\"",
            "made of one gem, its main part",
        ),
        (
            "object = \"gem\"",
            "object = \"long_sword\"",
            "never its main part",
        ),
    ];
    for (from, to, expected) in cases {
        let mut files = ContentFiles::builtin();
        files.insert("levels/gems.toml", &LEVEL.replace(from, to));
        files.insert(
            "scenarios/gems.toml",
            "name = \"Gems\"\nstart_level = \"gems\"\n",
        );
        let err = Content::load(&files).expect_err(expected);
        assert!(
            err.message.contains(expected),
            "{expected}: {}",
            err.message
        );
    }
}

#[test]
fn loot_finds_gems_rarely_and_larger_and_finer_deeper() {
    let content = Content::builtin().expect("content");
    let gems = content.gems().expect("gems");
    // Across seeds: 5000 finds from each of four runs' streams.
    let roll_many = |level: u8| {
        let mut found = Vec::new();
        for seed in 1..=4 {
            let mut rng = Rng::stream(seed, &format!("test/gems/{level}"));
            found.extend(
                (0..5000).filter_map(|_| loot::roll(&content, &mut rng, level, LootGrade::Any)),
            );
        }
        found
    };
    for level in [1, 8] {
        let found = roll_many(level);
        let loose: Vec<Cutting> = found
            .iter()
            .filter_map(|s| s.gem(&content))
            .map(|(_, c)| c)
            .collect();
        // About `chance` percent of finds are gems.
        let share = loose.len() * 1000 / found.len();
        assert!((40..=60).contains(&share), "level {level}: {share}‰ gems");
        // Uncut is the most common find.
        let uncut = gems.cut_id("uncut").unwrap();
        let by_cut = |c| loose.iter().filter(|g| g.cut == c).count();
        for (i, _) in gems.cuts().iter().enumerate() {
            assert!(
                by_cut(uncut) >= by_cut(crate::content::CutId(i as u8)),
                "level {level}"
            );
        }
        // Every stack found is valid: gems, ornaments, and magic alike.
        for stack in &found {
            assert!(stack.is_valid_for(&content), "{stack:?}");
        }
    }
    // Deeper finds run larger and finer.
    let average = |level: u8| {
        let loose: Vec<Cutting> = roll_many(level)
            .iter()
            .filter_map(|s| s.gem(&content))
            .map(|(_, c)| c)
            .collect();
        let sum: usize = loose
            .iter()
            .map(|c| usize::from(c.size.0) + usize::from(c.cut.0))
            .sum();
        sum * 100 / loose.len().max(1)
    };
    assert!(average(8) > average(1), "{} vs {}", average(8), average(1));
}

#[test]
fn loot_sets_rings_often_and_weapons_sometimes() {
    let content = Content::builtin().expect("content");
    let mut rng = Rng::stream(7, "test/ornaments");
    let (mut rings, mut set_rings, mut weapons, mut set_weapons) = (0, 0, 0, 0);
    let ring_class = content.object(object(&content, "gold_ring")).class;
    let weapon_class = content.object(object(&content, "long_sword")).class;
    for _ in 0..30_000 {
        let Some(stack) = loot::roll(&content, &mut rng, 5, LootGrade::Any) else {
            continue;
        };
        let class = content.object(stack.object).class;
        // Set with a gem (its material may be its own or another, #193).
        let ornamented =
            stack.gem(&content).is_none() && stack.parts(&content).iter().any(|p| p.gem.is_some());
        if ornamented {
            // Never an uncut gem in an item, and the item's main part is its own.
            let parts = stack.parts(&content);
            assert!(parts[0].gem.is_none());
            let set = parts.iter().find_map(|p| p.gem).expect("a gem");
            assert!(!content.gems().unwrap().cut(set.cut).rough);
        }
        if class == ring_class {
            rings += 1;
            set_rings += usize::from(ornamented);
        } else if class == weapon_class {
            weapons += 1;
            set_weapons += usize::from(ornamented);
        }
    }
    let ring_share = set_rings * 100 / rings;
    let weapon_share = set_weapons * 100 / weapons;
    assert!(
        (28..=42).contains(&ring_share),
        "{ring_share}% of rings set"
    );
    assert!(
        (3..=9).contains(&weapon_share),
        "{weapon_share}% of weapons set"
    );
}

#[test]
fn creatures_drop_gems_now_and_then_from_their_own_stream() {
    // The arena's kobolds drop loot one time in four.
    let w = World::new(content(), "arena", None).expect("world");
    let kobold = (w.state().actors.values())
        .find(|a| w.content().creature(a.creature).loot > 0)
        .cloned()
        .expect("a creature that drops loot");
    let (mut drops, mut gems) = (0, 0);
    for id in 1..8000 {
        let mut a = kobold.clone();
        a.id = ActorId(id);
        if let Some(stack) = w.loot_of(&a) {
            drops += 1;
            gems += usize::from(stack.gem(w.content()).is_some());
            assert!(stack.is_valid_for(w.content()));
            assert_eq!(w.loot_of(&a), Some(stack), "the same actor drops the same");
        }
    }
    // About the gems' chance (5%) of drops are gems.
    let share = gems * 1000 / drops;
    assert!((25..=80).contains(&share), "{gems} gems of {drops} drops");
}

#[test]
fn gems_survive_saves_and_a_replay_picks_them_up() {
    let content = content();
    let mut w = World::new(content.clone(), "gems", None).expect("world");
    let mut replay = Replay::new(&w);
    for action in parse_actions("move:E pickup").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let names: Vec<String> = (w.perception().inventory().iter())
        .map(|s| s.describe(&content))
        .collect();
    assert_eq!(names, ["a large fine emerald", "2 rubies"]);
    // The inventory dump names them as the player knows them.
    let dump = crate::dump::layer_rows(&w, crate::dump::Layer::Inventory).join("\n");
    assert!(dump.contains("a large fine emerald"), "{dump}");
    let bytes = crate::save::encode(&w);
    let back = crate::save::decode(&bytes, content.clone()).expect("loads");
    assert_eq!(back.state(), w.state());
    assert_eq!(back.state_hash(), w.state_hash());
    // The replay of those steps plays back to the same state, hash for hash.
    let replayed = Replay::from_json(&replay.to_json()).expect("replay");
    let again = replayed.run(content.clone()).expect("no divergence");
    assert_eq!(again.state_hash(), w.state_hash());
    // A save whose gem content does not know is refused.
    let mut bad = w.state().clone();
    let player = bad.player;
    let pack = &mut bad.actors.get_mut(&player).expect("player").inventory;
    pack.push(
        Stack::one(object(&content, "gem")).with_parts(vec![ItemPart {
            part: None,
            of: "opal".into(),
            gem: Some(GemCut {
                size: "small".into(),
                cut: "fair".into(),
            }),
        }]),
    );
    assert!(World::from_state(content.clone(), bad).is_err());
    // So is a loose gem without its gem.
    let mut bad = w.state().clone();
    let pack = &mut bad.actors.get_mut(&player).expect("player").inventory;
    pack.push(Stack::one(object(&content, "gem")));
    assert!(World::from_state(content, bad).is_err());
}

/// From the courtyard's start into Grimbold's General Store (shop_tests.rs).
const INTO_SHOP: &str = "move:E*36 move:S pickup move:NE move:NE pickup:b move:SE move:E*5 \
                         move:S*5 move:E*5";

#[test]
fn the_general_store_buys_gems_and_sells_them_back_by_their_gem() {
    let content = Arc::new(Content::builtin().expect("content"));
    let mut w = World::new(content.clone(), "courtyard", None).expect("courtyard");
    for action in parse_actions(INTO_SHOP).expect("actions") {
        w.apply(&action);
    }
    let shops = content.shops().expect("shops");
    let kind = w.state.current().shops[0].kind;
    // A medium fine garnet, 384 gold (the keeper's purse is 300), and an uncut tiny one, 5.
    let fine = gem(&content, "garnet", "medium", "fine");
    let rough = gem(&content, "garnet", "tiny", "uncut");
    let player = w.state.player;
    let pack = &mut w.state.actors.get_mut(&player).expect("player").inventory;
    pack.push(fine.clone());
    pack.push(rough.clone());
    let f = super::shop::favour(&content, shops, w.state.player(), &[]);
    let offer = shops.offer(&content, kind, &fine, f);
    // The store pays its share of the gem's worth, not of a "gem".
    assert!(offer >= 100, "{offer}");
    assert!(shops.offer(&content, kind, &fine, f) > shops.offer(&content, kind, &rough, f));
    let len = w.perception().inventory().len();
    let sell = Positions::of([len - 2]);
    w.apply(&Action::Sell(sell));
    let last = w.messages().iter().last().expect("said").text.clone();
    assert!(last.contains("medium fine garnet"), "{last}");
    // Picking it back up owes its price, by its gem, size, and cut.
    w.apply(&Action::PickUp(Positions::of([0])));
    let owed = w.state.current().shops[0].owed(shops, &content, f);
    assert_eq!(owed, shops.price(&content, kind, &fine, f));
    assert!(owed > offer);
    // Nobody stocks gems: the store only buys them.
    assert!(!shops.stocks(&content, kind, fine.object));
}
