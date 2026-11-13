//! The anvil (spec §7 "Special rooms"; `world/temper.rs`, `magic/worked.rs`): Temper listed only
//! beside an anvil and only for metal arms and armour, ten minutes' work that an interruption
//! spoils, every row of the one action's table, the rising difficulty, the numbers, the names and
//! the worth of a worked item, identification, the mark, and an anvil that is never spent.

use super::temper::check_made;
use super::*;
use crate::ability::Ability;
use crate::content::ContentFiles;
use crate::magic::{Affix, Aspect, Effect, Magic, Mods, Numbers, Rarity, Worked, may_roll};
use crate::object::{Stack, add_to_inventory};
use crate::rng::Rng;

/// A forge with its anvil at (5, 2), the player beside it at (4, 3), and at `k` a kobold (also
/// beside it), when asked for.
const ROOM: &str = "\
###########
#.........#
#....A....#
#...@k....#
#.........#
#.........#
###########
";

/// The anvil.
const ANVIL: Pos = Pos::new(5, 2);

/// The built-in content with [`ROOM`], a forge; the kobold only with `kobold`. The player (a
/// craftsperson, trained in Metalcraft) carries only what a test gives them.
fn world(kobold: bool) -> World {
    let mut files = ContentFiles::builtin();
    let k = if kobold {
        "{ terrain = \"floor\", creature = \"kobold\" }"
    } else {
        "\"floor\""
    };
    files.insert(
        "levels/test_forge.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{ROOM}'''\n\
             [[room]]\neffect = \"forge\"\narea = [0, 0, 10, 6]\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"A\" = \"anvil\"\n\"k\" = {k}\n"
        ),
    );
    files.insert(
        "scenarios/test_forge.toml",
        "name = \"T\"\nstart_level = \"test_forge\"\n\
         character = { race = \"dwarf\", gender = \"female\", class = \"craftsperson\" }\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    let mut w = World::new(content, "test_forge", None).expect("world");
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    me.inventory.clear();
    me.equipment = Default::default();
    w
}

fn object(w: &World, id: &str) -> crate::content::ObjectId {
    w.content.object_id(id).expect(id)
}

/// One of `id` in the pack's first place.
fn give(w: &mut World, id: &str) {
    let stack = Stack::one(object(w, id));
    w.player_mut().inventory.insert(0, stack);
}

#[test]
fn a_failed_attempt_never_harms_a_sword_of_proof() {
    // #1170: protected, a failure is withstood (`item_damage.rs`), its attempt still counted.
    let mut w = world(false);
    let id = object(&w, "long_sword");
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 10,
        affixes: vec![Affix {
            id: "of_proof".into(),
            tier: 1,
            value: 1,
        }],
        name: None,
        identified: false,
        carried: 0,
        unique: None,
    };
    (w.player_mut().inventory).insert(0, Stack::with_magic(id, 1, magic));
    let saved = w.state.clone();
    let failed = (0..200).find(|&n| {
        w.state = saved.clone();
        w.state.rng = Rng::stream(n, "temper_tests_proof");
        w.apply(&Action::Temper { item: 0 });
        said(&w, "The work goes wrong, but no harm is done.")
    });
    assert!(failed.is_some(), "some attempt fails");
    let after = first(&w).worked().cloned().expect("worked");
    assert_eq!((after.damaged, after.attempts), (false, 1));
}

#[test]
fn a_failed_attempt_never_harms_adamantium_but_it_may_fail_to_improve() {
    // #1444: adamantium is never damaged; a failure is withstood, its attempt still counted.
    let mut w = world(false);
    let id = object(&w, "long_sword");
    let m = w.content.material_id("adamantium").expect("adamantium");
    let part = crate::item_parts::ItemPart::material(&w.content, None, m);
    (w.player_mut().inventory).insert(0, Stack::one(id).with_parts(vec![part]));
    let saved = w.state.clone();
    let failed = (0..500).find(|&n| {
        w.state = saved.clone();
        w.state.rng = Rng::stream(n, "temper_tests_adamantium");
        w.apply(&Action::Temper { item: 0 });
        said(&w, "The work goes wrong, but no harm is done.")
    });
    assert!(failed.is_some(), "some attempt fails");
    let after = first(&w).worked().cloned().expect("worked");
    assert_eq!((after.damaged, after.attempts), (false, 1));
    assert!(after.improved.is_empty(), "nor is it improved");
    assert_eq!(w.state.player().inventory.len(), 1, "never destroyed");
}

/// A long sword with an affix that rolls on it, of tier 1, unidentified, in the pack's first place.
fn magic_sword(w: &mut World) {
    let id = object(w, "long_sword");
    let def = w.content.object(id).clone();
    let kind = def.kind.expect("a weapon");
    let affix = (w.content.affixes().all().iter())
        .find(|a| may_roll(a, kind, &def))
        .expect("an affix");
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 10,
        affixes: vec![Affix {
            id: affix.id.clone(),
            tier: 1,
            value: affix.tiers[0].min,
        }],
        name: None,
        identified: false,
        carried: 0,
        unique: None,
    };
    w.player_mut()
        .inventory
        .insert(0, Stack::with_magic(id, 1, magic));
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

/// Why Metalcraft's Temper row cannot be made now (`None`: it can).
fn here(w: &World) -> Option<RecipeLack> {
    let metal = w.content.skill_id("metalcraft").expect("metalcraft");
    (w.perception().recipes(metal).into_iter())
        .find(|r| r.name == "Temper")
        .expect("the trained see Temper")
        .unavailable
}

/// What came of an attempt, by what it did to the item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Came {
    /// Made.
    Made,
    /// Failed.
    Failed,
}

/// Attempts stack 0 until it comes out as `came`: what fate decides is tried over, from the same
/// state, on other draws. Each row of the table is reached this way by seed.
fn attempt(w: &mut World, came: Came) {
    let saved = w.state.clone();
    let before = w.state.player().inventory[0].worked().cloned();
    for n in 0..500 {
        w.state = saved.clone();
        w.state.rng = Rng::stream(n, "temper_tests");
        w.apply(&Action::Temper { item: 0 });
        let made = match (before.as_ref().is_some_and(|b| b.damaged), worked_after(w)) {
            // Damaged: gone, or the damage gone.
            (true, None) => false,
            (true, Some(after)) => !after.damaged,
            (false, after) => !after.expect("it survives").damaged,
        };
        if made == (came == Came::Made) {
            return;
        }
    }
    panic!("no draw came out {came:?}");
}

/// The state of the stack in the pack's first place, if there is one worked.
fn worked_after(w: &World) -> Option<Worked> {
    (w.state.player().inventory.first()).and_then(|s| s.worked().cloned())
}

fn first(w: &World) -> &Stack {
    &w.state.player().inventory[0]
}

fn set_worked(w: &mut World, improved: Vec<Aspect>, attempts: u8, damaged: bool) {
    let worked = Worked {
        improved,
        attempts,
        damaged,
        by: None,
        marked: false,
    };
    w.player_mut().inventory[0].set_worked(worked);
}

#[test]
fn temper_can_be_made_only_beside_an_anvil() {
    let mut w = world(false);
    assert_eq!(here(&w), None);
    // A recipe of Metalcraft's, no row of the Here group.
    let at = w.player_pos();
    assert!(Ability::here(w.state.current(), &w.content, at).is_empty());
    give(&mut w, "long_sword");
    w.apply(&Action::Move(Dir::S));
    let anvil = w.content.terrain_id("anvil").expect("anvil");
    assert_eq!(here(&w), Some(RecipeLack::Station(anvil)), "two cells away");
    let before = w.state.time;
    w.apply(&Action::Temper { item: 0 });
    assert!(said(&w, "There is no anvil here."));
    assert_eq!(w.state.time, before, "refused, no time");
}

/// #1387: Temper is Metalcraft's. Untrained, there is no row, `temper:` is refused with no
/// time, and walking into the anvil says what it needs; trained, it is a plain bump.
#[test]
fn the_untrained_cannot_temper_and_are_told_so_at_the_anvil() {
    let mut w = world(false);
    let metal = w.content.skill_id("metalcraft").expect("metalcraft");
    // Trained (the craftsperson): walking into the anvil is a plain bump, with no time.
    let before = w.state.time;
    w.apply(&Action::Move(Dir::NE));
    assert!(said(&w, "The anvil blocks your way."));
    assert_eq!(w.state.time, before);
    let skills = &mut w.player_mut().skills;
    *skills = skills.iter().filter(|(s, _)| *s != metal).collect();
    assert!(w.perception().recipes(metal).is_empty());
    assert!(!Ability::Profession(metal).had_by(w.state.player(), &w.content));
    give(&mut w, "long_sword");
    w.apply(&Action::Temper { item: 0 });
    assert!(said(&w, "You are not trained in Metalcraft."));
    assert_eq!(w.state.time, before, "refused, no time");
    assert_eq!(worked_after(&w), None, "untouched");
    w.apply(&Action::Move(Dir::NE));
    assert!(said(&w, "You need Metalcraft to work at the anvil."));
    assert_eq!(w.state.time, before, "a bump, no time");
    assert_eq!(w.player_pos(), Pos::new(4, 3));
}

#[test]
fn only_metal_arms_and_armour_are_worked() {
    let mut w = world(false);
    give(&mut w, "turnip");
    give(&mut w, "leather_jerkin");
    let before = w.state.time;
    w.apply(&Action::Temper { item: 0 });
    assert!(said(&w, "Only metal can be worked at an anvil."));
    w.apply(&Action::Temper { item: 1 });
    assert!(said(&w, "Only weapons and armour can be tempered."));
    w.apply(&Action::Temper { item: 5 });
    assert!(said(&w, "You carry no such thing."));
    assert_eq!(w.state.time, before, "refused, no time");
    let content = w.content.clone();
    let works = |id: &str| workable(&content, &Stack::one(content.object_id(id).expect(id)));
    assert!(works("long_sword") && works("chain_mail") && works("iron_helm"));
    assert!(!works("turnip") && !works("leather_jerkin") && !works("short_bow"));
    assert!(!works("quarterstaff"), "wood");
}

#[test]
fn the_main_part_says_what_it_is_made_of() {
    let w = world(false);
    let c = &w.content;
    let sword = Stack::one(object(&w, "long_sword"));
    let bone = c.material_id("dragonbone").expect("dragonbone");
    let part = crate::item_parts::ItemPart::material(c, None, bone);
    let dragonbone = sword.clone().with_parts(vec![part]);
    assert_ne!(
        dragonbone.names(c),
        sword.names(c),
        "really another material"
    );
    assert!(temperable(c, &sword) && !temperable(c, &dragonbone));
}

#[test]
fn a_spirit_weapon_is_refused() {
    let mut w = world(false);
    let sword = object(&w, "long_sword");
    w.player_mut().inventory.push(Stack::spirit(sword));
    let before = w.state.time;
    w.apply(&Action::Temper { item: 0 });
    assert!(said(&w, "Only metal can be worked at an anvil."));
    assert_eq!(w.state.time, before, "refused, no time");
}

#[test]
fn an_attempt_takes_ten_minutes_identifies_and_trains_metalcraft() {
    let mut w = world(false);
    magic_sword(&mut w);
    let metalcraft = w.content.skill_id("metalcraft").expect("metalcraft");
    let marks = |w: &World| {
        w.state
            .player()
            .skills
            .get(metalcraft)
            .map_or(0, |t| t.marks)
    };
    let before = marks(&w);
    let start = w.state.time;
    w.apply(&Action::Temper { item: 0 });
    let all: Vec<_> = w.messages().iter().map(|m| m.text.clone()).collect();
    assert!(
        (w.messages().iter()).any(|m| m.text.starts_with("You set to work on the long sword")),
        "{all:?}"
    );
    assert_eq!(w.state.time.since(start).as_seconds(), 600);
    assert!(first(&w).is_identified(), "what survives is identified");
    assert!(marks(&w) > before, "every attempt marks Metalcraft");
    let anvil = w.state.current().terrain_at(ANVIL).expect("cell");
    assert_eq!(w.content.terrain(anvil).id, "anvil", "never spent");
    assert_eq!(here(&w), None);
}

#[test]
fn an_interrupted_attempt_does_nothing_to_the_item_and_counts_for_nothing() {
    let mut w = world(true);
    magic_sword(&mut w);
    let start = w.state.time;
    w.apply(&Action::Temper { item: 0 });
    assert!(said(&w, "You put down the hammer."));
    assert!(w.state.time.since(start).as_seconds() < 600);
    assert!(!first(&w).is_identified(), "not even identified");
    assert_eq!(first(&w).worked(), None, "no attempt counted");
}

#[test]
fn the_check_is_harder_for_every_attempt_and_the_extremes_stand() {
    let rules = crate::content::TemperRules::default();
    assert_eq!(rules.against(0), 13);
    assert_eq!(rules.against(5), 23);
    assert!(check_made(13, 0, 13) && !check_made(12, 0, 13));
    assert!(check_made(24, -50, 500), "a natural 24 always succeeds");
    assert!(!check_made(2, 50, 13), "a natural 2 always fails");
}

#[test]
fn every_attempt_counts_through_damage_and_repair() {
    let mut w = world(false);
    give(&mut w, "long_sword");
    attempt(&mut w, Came::Made);
    assert_eq!(worked_after(&w).expect("worked").attempts, 1);
    attempt(&mut w, Came::Failed);
    let after = worked_after(&w).expect("worked");
    assert_eq!((after.attempts, after.damaged), (2, true));
    attempt(&mut w, Came::Made);
    let after = worked_after(&w).expect("worked");
    assert_eq!((after.attempts, after.damaged), (3, false));
    attempt(&mut w, Came::Made);
    assert_eq!(worked_after(&w).expect("worked").attempts, 4);
}

#[test]
fn many_attempts_behind_it_only_a_natural_24_will_do() {
    let mut w = world(false);
    give(&mut w, "long_sword");
    set_worked(&mut w, Vec::new(), 200, false);
    let saved = w.state.clone();
    let mut made = 0;
    for n in 0..40 {
        w.state = saved.clone();
        w.state.rng = Rng::stream(n, "temper_tests");
        w.apply(&Action::Temper { item: 0 });
        let after = worked_after(&w).expect("worked");
        assert_eq!(after.attempts, 201);
        made += i32::from(!after.damaged);
    }
    assert!(made <= 2, "{made} of 40");
}

#[test]
fn made_it_improves_up_to_three_times_then_it_is_refused() {
    let mut w = world(false);
    give(&mut w, "chain_mail");
    let names: Vec<String> = (0..3)
        .map(|_| {
            attempt(&mut w, Came::Made);
            w.lore().describe(first(&w))
        })
        .collect();
    assert_eq!(
        names,
        [
            "a Fine chain mail shirt",
            "a Marvellous chain mail shirt",
            "a Masterwork chain mail shirt",
        ]
    );
    let after = worked_after(&w).expect("worked");
    assert!(
        (after.improved.iter()).all(|a| matches!(a, Aspect::Defence | Aspect::Protection)),
        "only what it has: {:?}",
        after.improved
    );
    let before = w.state.time;
    w.apply(&Action::Temper { item: 0 });
    assert!(said(&w, "It can be made no finer."));
    assert_eq!(w.state.time, before, "refused, no time");
    assert_eq!(worked_after(&w), Some(after), "nothing counted");
    assert!(!workable(&w.content, first(&w)), "and it leaves the list");
    assert!(temperable(&w.content, first(&w)));
}

#[test]
fn failed_it_is_damaged_and_all_its_improvements_go() {
    let mut w = world(false);
    give(&mut w, "long_sword");
    set_worked(&mut w, vec![Aspect::Die, Aspect::ToHit], 2, false);
    attempt(&mut w, Came::Failed);
    let after = worked_after(&w).expect("worked");
    assert!(after.damaged && after.improved.is_empty());
    assert_eq!(w.lore().describe(first(&w)), "a damaged long sword");
    assert!(said(&w, "The metal cracks under the hammer."));
    assert!(said(&w, "This is a damaged long sword!"));
}

#[test]
fn the_actors_dump_reads_the_attempts_and_the_improvements() {
    let mut w = world(false);
    give(&mut w, "long_sword");
    let rows = |w: &World| crate::dump::layer_rows(w, crate::dump::Layer::Actors).join("\n");
    assert!(!rows(&w).contains("(attempts"), "{}", rows(&w));
    set_worked(&mut w, vec![Aspect::Die, Aspect::ToHit], 2, false);
    let text = rows(&w);
    assert!(
        text.contains("Marvellous long sword (attempts 2: die to_hit)"),
        "{text}"
    );
    set_worked(&mut w, Vec::new(), 3, true);
    let text = rows(&w);
    assert!(
        text.contains("damaged long sword (attempts 3)"),
        "no dangling colon: {text}"
    );
    assert!(!text.contains("attempts 3:"), "{text}");
}

#[test]
fn damaged_it_is_repaired_or_destroyed() {
    let mut w = world(false);
    give(&mut w, "long_sword");
    set_worked(&mut w, Vec::new(), 2, true);
    let saved = w.state.clone();
    attempt(&mut w, Came::Made);
    let after = worked_after(&w).expect("worked");
    assert_eq!((after.damaged, after.attempts), (false, 3));
    assert_eq!(w.lore().describe(first(&w)), "a long sword");
    w.state = saved;
    attempt(&mut w, Came::Failed);
    assert!(w.state.player().inventory.is_empty(), "gone");
    assert!(said(&w, "It shatters under the hammer!"));
}

/// Item damage of `harm` on stack 0 until it takes (fate tried over on other draws): what came.
fn harmed(w: &mut World, harm: crate::content::ItemHarm) -> super::item_damage::Harmed {
    let saved = w.state.clone();
    for n in 0..500 {
        w.state = saved.clone();
        w.state.rng = Rng::stream(n, "temper_tests_harm");
        let came = w.harm_item(0, harm);
        if came != super::item_damage::Harmed::Withstood {
            return came;
        }
    }
    panic!("the item never took {harm:?}");
}

#[test]
fn acid_and_a_failed_attempt_leave_the_one_state_the_anvil_mends() {
    use super::item_damage::Harmed;
    use crate::content::ItemHarm;
    // #1168: acid damages a fine sword exactly as a failure does, but for its word.
    let mut w = world(false);
    give(&mut w, "long_sword");
    set_worked(&mut w, vec![Aspect::Die], 1, false);
    let saved = w.state.clone();
    attempt(&mut w, Came::Failed);
    let failed = worked_after(&w).expect("worked");
    w.state = saved;
    assert_eq!(harmed(&mut w, ItemHarm::Acid), Harmed::Damaged);
    let corroded = worked_after(&w).expect("worked");
    assert_eq!(corroded.by, Some(ItemHarm::Acid));
    assert_eq!(
        Worked {
            attempts: failed.attempts,
            ..corroded.clone()
        },
        Worked {
            by: Some(ItemHarm::Acid),
            ..failed
        },
        "the same state"
    );
    assert_eq!(w.lore().describe(first(&w)), "a corroded long sword");
    // The anvil mends it as any damage.
    let saved = w.state.clone();
    attempt(&mut w, Came::Made);
    let mended = worked_after(&w).expect("worked");
    assert_eq!((mended.damaged, mended.by), (false, None));
    assert_eq!(w.lore().describe(first(&w)), "a long sword");
    // A failure on it, or rust on it, destroys it.
    w.state = saved.clone();
    attempt(&mut w, Came::Failed);
    assert!(w.state.player().inventory.is_empty(), "shattered");
    w.state = saved;
    assert_eq!(harmed(&mut w, ItemHarm::Rust), Harmed::Destroyed);
    assert!(w.state.player().inventory.is_empty(), "rusted away");
}

#[test]
fn the_numbers_follow_the_improvements() {
    let mut w = world(false);
    let c = w.content.clone();
    give(&mut w, "long_sword");
    let bare = Numbers::of(first(&w), &c);
    let dice = |s: &Stack| s.attack(&c).expect("an attack").damage;
    let plain = dice(first(&w));
    set_worked(&mut w, vec![Aspect::Die], 1, false);
    assert_eq!(dice(first(&w)), plain.grow(2), "2 sides more");
    set_worked(&mut w, vec![Aspect::Die, Aspect::Die], 2, false);
    assert_eq!(dice(first(&w)), plain.grow(4), "and 2 more");
    set_worked(&mut w, vec![Aspect::ToHit, Aspect::Damage], 2, false);
    let local = Mods::local(first(&w), &c);
    assert_eq!(
        (local.get(Effect::ToHit), local.get(Effect::Damage)),
        (1, 1)
    );
    let worked = Numbers::of(first(&w), &c);
    let (was, is) = (bare.attack.expect("attack"), worked.attack.expect("attack"));
    assert_eq!(is.to_hit, was.to_hit + 1);
    assert_eq!(is.damage, was.damage.plus(1));

    w.player_mut().inventory.clear();
    give(&mut w, "chain_mail");
    let plain = Numbers::of(first(&w), &c).defence.expect("defence");
    set_worked(&mut w, vec![Aspect::Defence, Aspect::Protection], 2, false);
    let now = Numbers::of(first(&w), &c).defence.expect("defence");
    assert_eq!(now.dv, plain.dv + 1);
    assert_eq!(now.pv.expect("pv"), plain.pv.expect("pv") + 1);
    // What an actor wearing it stands behind.
    let slot = c.slot_id("body").expect("body");
    let bare = w.state.player().protection(&c, None).base;
    w.player_mut().equipment.put(&[slot], 0);
    let worn = w.state.player().protection(&c, None).base;
    assert_eq!(worn - bare, plain.pv.expect("pv") + 1);
}

#[test]
fn a_shield_takes_its_defence_as_a_shield() {
    let mut w = world(false);
    let c = w.content.clone();
    give(&mut w, "small_shield");
    set_worked(&mut w, vec![Aspect::Defence], 1, false);
    assert_eq!(Mods::local(first(&w), &c).get(Effect::Shield), 1);
    assert_eq!(Mods::local(first(&w), &c).get(Effect::Armour), 0);
}

#[test]
fn the_attacks_read_the_dice_as_they_were_worked() {
    let mut w = world(false);
    let c = w.content.clone();
    give(&mut w, "long_sword");
    let slot = c.slot_id("main_hand").expect("main hand");
    w.player_mut().equipment.put(&[slot], 0);
    let before = w.state.player().melee(&c).damage;
    set_worked(&mut w, vec![Aspect::Die], 1, false);
    assert_eq!(w.state.player().melee(&c).damage, before.grow(2));
    let weapon = w.state.player().weapon(&c).expect("a weapon");
    assert_eq!(
        weapon.1.damage,
        c.object(weapon.0).attack.expect("a").damage.grow(2)
    );
}

#[test]
fn a_damaged_item_is_a_point_worse_than_its_base_until_mended() {
    let mut w = world(false);
    let c = w.content.clone();
    give(&mut w, "long_sword");
    let plain = Numbers::of(first(&w), &c);
    set_worked(&mut w, Vec::new(), 3, true);
    assert_ne!(Numbers::of(first(&w), &c), plain);
    // A weapon's damage; armour's Protection.
    let mods = Mods::local(first(&w), &c);
    assert_eq!((mods.get(Effect::Damage), mods.get(Effect::ToHit)), (-1, 0));
    set_worked(&mut w, Vec::new(), 3, false);
    assert_eq!(Numbers::of(first(&w), &c), plain);
    let mut w = world(false);
    give(&mut w, "chain_mail");
    set_worked(&mut w, Vec::new(), 1, true);
    let mods = Mods::local(first(&w), &c);
    assert_eq!(
        (mods.get(Effect::Protection), mods.get(Effect::Armour)),
        (-1, 0)
    );
}

#[test]
fn affixes_stay_through_damage_and_go_with_the_item() {
    let mut w = world(false);
    magic_sword(&mut w);
    let affixes = first(&w).magic().expect("magic").affixes.clone();
    attempt(&mut w, Came::Failed);
    assert_eq!(first(&w).magic().expect("magic").affixes, affixes, "kept");
    attempt(&mut w, Came::Made);
    assert_eq!(first(&w).magic().expect("magic").affixes, affixes, "kept");
    attempt(&mut w, Came::Made);
    assert_eq!(first(&w).magic().expect("magic").affixes, affixes, "kept");
    attempt(&mut w, Came::Failed);
    attempt(&mut w, Came::Failed);
    assert!(w.state.player().inventory.is_empty(), "gone with it");
}

#[test]
fn the_name_shows_ahead_of_the_rest_whatever_is_identified() {
    let mut w = world(false);
    magic_sword(&mut w);
    set_worked(&mut w, vec![Aspect::ToHit, Aspect::Damage], 2, false);
    let lore = w.lore();
    assert_eq!(lore.describe(first(&w)), "a Marvellous long sword (magic)");
    assert_eq!(lore.title(first(&w)), "Marvellous long sword");
    w.player_mut().inventory[0].identify();
    let known = w.lore().describe(first(&w));
    assert!(known.starts_with("a Marvellous "), "{known}");
    let affix = first(&w).magic().expect("magic").rolled(&w.content)[0]
        .def
        .name
        .clone();
    assert!(known.contains(&affix), "{known}");
    assert!(
        known.find("Marvellous") < known.find(&affix),
        "ahead of the affix: {known}"
    );
}

#[test]
fn a_rare_item_keeps_its_name_first() {
    let mut w = world(false);
    magic_sword(&mut w);
    let id = first(&w).object;
    let mut magic = first(&w).magic().expect("magic").clone();
    magic.rarity = Rarity::Rare;
    magic.name = Some("Grim Bite".into());
    magic.identified = true;
    w.player_mut().inventory[0] = Stack::with_magic(id, 1, magic);
    set_worked(&mut w, vec![Aspect::ToHit], 1, false);
    assert_eq!(w.lore().describe(first(&w)), "Grim Bite, a Fine long sword");
}

#[test]
fn the_worth_follows_the_state() {
    let mut w = world(false);
    let c = w.content.clone();
    give(&mut w, "long_sword");
    let worth = first(&w).value(&c);
    assert!(worth > 1);
    let of = |w: &mut World, improved: Vec<Aspect>, damaged| {
        set_worked(w, improved, 1, damaged);
        first(w).value(&c)
    };
    let one = vec![Aspect::Die];
    let three = vec![Aspect::Die; 3];
    assert_eq!(of(&mut w, one.clone(), false), worth * 2);
    assert_eq!(of(&mut w, vec![Aspect::Die; 2], false), worth * 3);
    assert_eq!(of(&mut w, three, false), worth * 5);
    assert_eq!(of(&mut w, Vec::new(), true), worth.div_ceil(2));
    assert_eq!(
        of(&mut w, Vec::new(), false),
        worth,
        "attempts are worth nothing"
    );
    // A shop's bill counts it apart from one unworked, at its worth.
    let plain = Stack::one(first(&w).object);
    set_worked(&mut w, one, 1, false);
    assert!(!first(&w).same_goods(&plain));
    assert_eq!(first(&w).goods(1).value(&c), worth * 2);
}

#[test]
fn a_worked_item_is_like_no_other_and_one_of_a_stack_is_worked_apart() {
    let mut w = world(false);
    let c = w.content.clone();
    let dagger = object(&w, "dagger");
    w.player_mut().inventory.push(Stack::new(dagger, 3));
    w.apply(&Action::Temper { item: 0 });
    let pack = &w.state.player().inventory;
    assert_eq!((pack.len(), pack[0].count, pack[1].count), (2, 2, 1));
    assert!(pack[0].worked().is_none() && pack[1].worked().is_some());
    // Of what stacks, a worked one joins nothing, with nothing left to show for it too.
    let arrow = object(&w, "arrow");
    let mut one = Stack::one(arrow);
    one.set_worked(Worked {
        attempts: 1,
        ..Worked::default()
    });
    assert_eq!(one.describe(&c), Stack::one(arrow).describe(&c));
    let mut pack = vec![Stack::one(arrow)];
    add_to_inventory(&mut pack, one.clone(), &c);
    add_to_inventory(&mut pack, one, &c);
    assert_eq!(pack.len(), 3);
    add_to_inventory(&mut pack, Stack::one(arrow), &c);
    assert_eq!(pack[0].count, 2, "plain ones still join each other");
}

#[test]
fn saves_and_the_hash_carry_it() {
    let mut w = world(false);
    give(&mut w, "long_sword");
    let plain = w.state_hash();
    set_worked(&mut w, vec![Aspect::Die, Aspect::ToHit], 3, false);
    let worked = w.state_hash();
    assert_ne!(plain, worked);
    set_worked(&mut w, vec![Aspect::Die, Aspect::ToHit], 4, false);
    assert_ne!(w.state_hash(), worked, "the attempts count");
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(loaded.state_hash(), w.state_hash());
    assert_eq!(
        loaded.state().player().inventory[0].worked(),
        first(&w).worked()
    );
}
