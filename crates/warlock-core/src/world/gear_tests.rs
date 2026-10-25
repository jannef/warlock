//! Creature gear (spec §8 "Creature gear"): the tables, the rolls, where they are made, and
//! Scavenger.

use super::*;
use crate::content::{ContentFiles, Keep, LevelTemplate, ObjectId, Rarity};
use crate::gear;
use crate::kit::Pack;
use crate::magic::{LootGrade, loot};
use crate::object::weight_of;
use crate::rng::Rng;
use std::sync::{Arc, LazyLock};

/// Fixture creature types: a raider with a racial spear and lines of every kind, one that never
/// draws, plain and charmed daggers, a weakling offered plate, and a type that writes its racial
/// weapon's line itself.
const FIXTURES: &str = r##"
[[creature]]
id = "raider"
name = "raider"
glyph = "R"
fg = "#ffffff"
weight = 60000
faction = "kobolds"
stats = { strength = 14, toughness = 12 }
weapon_skills = { spears = 1 }
racial_weapon = "spear"

[[creature.gear]]
chance = 60
one_of = ["padded_coat", "leather_jerkin", "leather_cap"]

[[creature.gear]]
chance = 60
one_of = ["short_sword", "arrow*12", "dagger*2"]

[[creature.gear]]
chance = 0
one_of = ["plate_armour"]

[[creature]]
id = "plain"
name = "plain"
glyph = "P"
fg = "#ffffff"
weight = 60000
faction = "kobolds"

[[creature.gear]]
chance = 100
one_of = ["dagger"]

[[creature]]
id = "charmed"
name = "charmed"
glyph = "C"
fg = "#ffffff"
weight = 60000
faction = "kobolds"

[[creature.gear]]
chance = 100
one_of = ["dagger", "leather_cap"]
magic = 100

[[creature]]
id = "weakling"
name = "weakling"
glyph = "W"
fg = "#ffffff"
weight = 60000
faction = "kobolds"
stats = { strength = 3, toughness = 3 }

[[creature.gear]]
chance = 100
one_of = ["plate_armour"]

[[creature.gear]]
chance = 100
one_of = ["leather_cap"]

[[creature]]
id = "spearman"
name = "spearman"
glyph = "S"
fg = "#ffffff"
weight = 60000
faction = "kobolds"
weapon_skills = { spears = 1 }
racial_weapon = "spear"

[[creature.gear]]
chance = 100
one_of = ["spear"]
"##;

/// A kobold beside the player, with a fixed kit.
const DEN: &str = r##"
name = "Den"
ambient = [200, 200, 200, 0]
map = '''
######
#@k..#
######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold", carry = ["dagger"] }
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("data/creatures/zz_fixtures.toml", FIXTURES);
    files.insert("levels/den.toml", DEN);
    files.insert(
        "scenarios/den.toml",
        "name = \"den\"\nstart_level = \"den\"\nhp = 1000\n",
    );
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("valid fixtures")));

fn creature(name: &str) -> crate::content::CreatureId {
    CONTENT.creature_id(name).expect(name)
}

fn object(name: &str) -> ObjectId {
    CONTENT.object_id(name).expect(name)
}

fn roll(name: &str, seed: u64, item_level: u8) -> Pack {
    gear::roll(
        &CONTENT,
        &mut Rng::gear(seed, "test"),
        creature(name),
        item_level,
    )
}

/// The id of each stack of `pack`, with whether it is worn or wielded.
fn held(pack: &Pack) -> Vec<(String, bool)> {
    (pack.items.iter().enumerate())
        .map(|(i, s)| (CONTENT.object(s.object).id.clone(), pack.equipment.holds(i)))
        .collect()
}

#[test]
fn the_racial_weapon_is_the_first_line_unless_written() {
    let raider = &CONTENT.creature(creature("raider")).gear;
    assert_eq!(raider.racial_weapon, Some(object("spear")));
    assert_eq!(raider.lines.len(), 4);
    let first = &raider.lines[0];
    assert_eq!(
        (first.chance, first.magic),
        (crate::content::RACIAL_CHANCE, 0)
    );
    assert_eq!(first.one_of[0].0[0].object, object("spear"));
    assert_eq!(first.one_of[0].1, Keep::Wield);
    // Defaults by the object: armour worn, weapons and missiles wielded, two of what does not
    // stack carried.
    let keeps: Vec<Keep> = raider.lines[2].one_of.iter().map(|(_, k)| *k).collect();
    assert_eq!(keeps, [Keep::Wield, Keep::Wield, Keep::Carry]);
    assert!(raider.lines[1].one_of.iter().all(|(_, k)| *k == Keep::Wear));
    // A type that writes the line itself keeps its own chance.
    let spearman = &CONTENT.creature(creature("spearman")).gear;
    assert_eq!(spearman.lines.len(), 1);
    assert_eq!(spearman.lines[0].chance, 100);
}

#[test]
fn gear_follows_the_table_and_the_same_seed_gives_the_same() {
    let mut spears = 0;
    let mut armour = 0;
    for seed in 0..300 {
        let pack = roll("raider", seed, 1);
        assert_eq!(pack, roll("raider", seed, 1), "{seed}");
        for (id, held) in held(&pack) {
            match id.as_str() {
                // The racial weapon is wielded whenever carried.
                "spear" => {
                    spears += 1;
                    assert!(held, "{seed}");
                }
                "padded_coat" | "leather_jerkin" | "leather_cap" => {
                    armour += 1;
                    assert!(held, "{seed}: {id}");
                }
                "arrow" => assert!(held, "{seed}"),
                // A second weapon goes to the off hand beside the spear; two daggers are carried.
                "short_sword" => assert!(held, "{seed}"),
                "dagger" => assert!(!held, "{seed}"),
                // A line with chance 0 never draws.
                other => panic!("{seed}: {other}"),
            }
        }
    }
    // About nine in ten carry the spear, three in five something worn.
    assert!((240..=295).contains(&spears), "{spears}");
    assert!((140..=220).contains(&armour), "{armour}");
}

#[test]
fn magic_comes_by_its_chance_at_the_floors_item_level() {
    for seed in 0..60 {
        let plain = roll("plain", seed, 5);
        assert!(plain.items.iter().all(|s| s.rarity() == Rarity::Normal));
        for level in [1, 8] {
            let charmed = roll("charmed", seed, level);
            let [stack] = charmed.items.as_slice() else {
                panic!("{charmed:?}");
            };
            assert!(stack.rarity() >= Rarity::Magic, "{seed} {level}: {stack:?}");
            // Unidentified, as loot is; worn or wielded all the same.
            assert!(!stack.is_identified());
            assert!(charmed.equipment.holds(0));
        }
    }
}

#[test]
fn nothing_is_carried_over_the_carrying_limit() {
    let def = CONTENT.creature(creature("weakling"));
    let limit = crate::burden::limit(
        def.stats.score(crate::stats::Stat::Strength),
        def.stats.score(crate::stats::Stat::Toughness),
        CONTENT.size(def.size).carry,
    );
    let mut left_out = 0;
    for seed in 0..20 {
        let pack = roll("weakling", seed, 1);
        // Steel plate is left out (one of a light material may come); the cap always comes.
        let held = held(&pack);
        assert!(
            held.contains(&("leather_cap".to_string(), true)),
            "{held:?}"
        );
        assert!(weight_of(&pack.items, &CONTENT) <= limit);
        left_out += usize::from(held.len() == 1);
    }
    assert!(left_out > 0);
    for name in ["raider", "kobold", "kobold_archer", "kobold_shaman"] {
        for seed in 0..50 {
            let pack = roll(name, seed, 3);
            let def = CONTENT.creature(creature(name));
            let limit = crate::burden::limit(
                def.stats.score(crate::stats::Stat::Strength),
                def.stats.score(crate::stats::Stat::Toughness),
                CONTENT.size(def.size).carry,
            );
            assert!(weight_of(&pack.items, &CONTENT) <= limit, "{name} {seed}");
        }
    }
}

#[test]
fn a_racial_weapon_needs_a_rank_in_its_skill() {
    let mut files = files();
    files.insert(
        "data/creatures/zz_hobgoblin.toml",
        "[[creature]]\nid = \"hobgoblin\"\nname = \"hobgoblin\"\nglyph = \"g\"\nfg = \"#ffffff\"\n\
         weight = 30000\nfaction = \"kobolds\"\nracial_weapon = \"club\"\n",
    );
    let err = Content::load(&files).expect_err("no rank").to_string();
    assert!(
        err.contains("`hobgoblin`") && err.contains("no rank in `maces`"),
        "{err}"
    );
    for (line, why) in [
        ("racial_weapon = \"leather_cap\"", "is no weapon"),
        ("racial_weapon = \"spork\"", "unknown object `spork`"),
        (
            "[[creature.gear]]\nchance = 101\none_of = [\"dagger\"]",
            "0 to 100",
        ),
        ("[[creature.gear]]\nchance = 5\none_of = []", "at least one"),
        (
            "[[creature.gear]]\nchance = 5\none_of = [\"ration\"]\nmagic = 5",
            "never magic",
        ),
        (
            "[[creature.gear]]\nchance = 5\none_of = [\"dagger\"]\nkeep = \"wear\"",
            "use `wield`",
        ),
    ] {
        let mut files = self::files();
        files.insert(
            "data/creatures/zz_hobgoblin.toml",
            &format!(
                "[[creature]]\nid = \"hobgoblin\"\nname = \"hobgoblin\"\nglyph = \"g\"\n\
                 fg = \"#ffffff\"\nweight = 30000\nfaction = \"kobolds\"\n{line}\n"
            ),
        );
        let err = Content::load(&files).expect_err(line).to_string();
        assert!(err.contains(why), "{line}: {err}");
    }
}

/// `text` without its gear: the `racial_weapon` line and each `[[creature.gear]]` block (up to
/// the next blank line).
pub(super) fn without_gear(text: &str) -> String {
    let mut out = String::new();
    let mut in_gear = false;
    for line in text.lines() {
        if in_gear {
            in_gear = !line.trim().is_empty();
            continue;
        }
        if line == "[[creature.gear]]" {
            in_gear = true;
            continue;
        }
        if !line.starts_with("racial_weapon") {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// A generated level with its creatures' kits left out.
fn bare(template: &LevelTemplate) -> LevelTemplate {
    let mut bare = template.clone();
    for spawn in &mut bare.creatures {
        spawn.carry.clear();
        spawn.equipment = Default::default();
        spawn.rolls_gear = false;
    }
    bare
}

#[test]
fn generated_levels_roll_gear_from_their_own_stream_and_nothing_else_moves() {
    let mut plain = ContentFiles::builtin();
    plain.edit_table("creatures", without_gear);
    let plain = Content::load(&plain).expect("valid without gear");
    let kobold = plain.creature_id("kobold").expect("kobold");
    assert!(plain.creature(kobold).gear.is_empty());
    let builtin = Content::builtin().expect("built-in");
    let mut armed = 0;
    for seed in 0..6 {
        for depth in 1..=4 {
            let id = format!("depths:{depth}");
            let with =
                crate::mapgen::generate(&builtin, ("showcase", seed), &id, None).expect("level");
            let without =
                crate::mapgen::generate(&plain, ("showcase", seed), &id, None).expect("level");
            // The map, its traps, secrets, loot, and creatures are the same; only their kits.
            assert_eq!(bare(&with), bare(&without), "{id} {seed}");
            // The kits rolled are the level's gear stream, in spawn order; a vault's fixed kit
            // (the guard post's key) stays as it is.
            let mut rng = Rng::gear(seed, &id);
            for (spawn, plain) in with.creatures.iter().zip(&without.creatures) {
                assert!(!spawn.rolls_gear);
                if !plain.carry.is_empty() {
                    assert_eq!(spawn.carry, plain.carry);
                    continue;
                }
                let pack = gear::roll(&builtin, &mut rng, spawn.creature, with.item_level);
                assert_eq!(
                    (&pack.items, &pack.equipment),
                    (&spawn.carry, &spawn.equipment)
                );
                armed += usize::from(!spawn.carry.is_empty());
            }
        }
    }
    assert!(armed > 0);
}

#[test]
fn a_hand_made_levels_creatures_roll_from_their_own_stream_unless_given_a_kit() {
    let builtin = Arc::new(Content::builtin().expect("built-in"));
    let traps = World::new(builtin.clone(), "traps", None).expect("traps");
    let level = traps.state.current_level.clone();
    let item_level = builtin.item_level(&level);
    let kobold = builtin.creature_id("kobold").expect("kobold");
    let mut kobolds = 0;
    for actor in traps.state.actors.values() {
        if actor.creature != kobold {
            continue;
        }
        kobolds += 1;
        let mut rng = Rng::gear(traps.state.seed, &actor.id.0.to_string());
        let pack = gear::roll(&builtin, &mut rng, kobold, item_level);
        assert_eq!(
            (&pack.items, &pack.equipment),
            (&actor.inventory, &actor.equipment)
        );
    }
    assert!(kobolds > 0);
    // The gameplay stream is the same without the tables.
    let mut plain = ContentFiles::builtin();
    plain.edit_table("creatures", without_gear);
    let plain = Arc::new(Content::load(&plain).expect("valid"));
    let bare = World::new(plain, "traps", None).expect("traps");
    assert_eq!(bare.state.rng, traps.state.rng);
    // A legend entry with a fixed kit keeps it: the arena's spearman, the den's dagger.
    let arena = World::new(builtin.clone(), "arena", None).expect("arena");
    for actor in arena.state.actors.values() {
        if actor.creature == kobold {
            let objects: Vec<_> = actor.inventory.iter().map(|s| s.object).collect();
            assert_eq!(objects, [builtin.object_id("spear").unwrap()]);
        }
    }
    let den = World::new(CONTENT.clone(), "den", None).expect("den");
    let k = den.state.actor_at("den", Pos::new(2, 1)).expect("kobold");
    assert_eq!(
        k.inventory,
        [crate::object::Stack::new(object("dagger"), 1)]
    );
}

/// The den after its kobold, given a magic dagger, died by the player's hand; with Scavenger
/// if `scavenger`.
fn kill_in_the_den(scavenger: bool) -> World {
    let mut w = World::new(CONTENT.clone(), "den", None).expect("den");
    if scavenger {
        w.player_mut().talents.take("scavenger", None);
    }
    let id = w.state.actor_at("den", Pos::new(2, 1)).expect("kobold").id;
    let mut rng = Rng::stream(1, "test");
    let dagger = loot::make(&CONTENT, &mut rng, object("dagger"), 1, 3, LootGrade::Magic);
    assert!(dagger.rarity() >= Rarity::Magic && !dagger.is_identified());
    w.state.actors.get_mut(&id).expect("kobold").inventory = vec![dagger];
    let (player, time) = (w.state.player, w.state.time);
    w.dies(id, Some(player), None, time, &mut Vec::new());
    w
}

#[test]
fn scavenger_identifies_what_a_kill_drops() {
    let dagger = object("dagger");
    for scavenger in [false, true] {
        let w = kill_in_the_den(scavenger);
        let pile = w.state.current().objects_at(Pos::new(2, 1));
        let stack = pile
            .iter()
            .find(|s| s.object == dagger)
            .expect("the dagger");
        assert_eq!(stack.is_identified(), scavenger);
    }
}
