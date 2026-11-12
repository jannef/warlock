//! Setting a gem and making jewellery (docs/design/recipes.md §5, §6; `world/set.rs`): a cut gem
//! set and the name following, the wizard's steps (no uncut gem, no item set already), a failure
//! chipping the gem and leaving the item, a natural 2 shattering it, the refusals with no time,
//! the twenty minutes and the mark, each ring and amulet made of its own material and never
//! graded, the gold amulet's required gem, the bone amulet by Woodcraft, and a replay.

use super::craft_make::make_out_for_test;
use super::set::set_out_for_test;
use super::*;
use crate::action::parse_actions;
use crate::content::{ContentFiles, RecipeId};
use crate::item_parts::ItemPart;
use crate::object::Stack;
use crate::replay::Replay;
use crate::skills::{SkillId, Trained};
use crate::stats::{PerStat, Stats};
use std::sync::LazyLock;

const ROOM: &str = "\
###########
#.........#
#.@.......#
#.........#
#.......k.#
###########
";

/// The built-in content with [`ROOM`], with no one in it or with a kobold; a long sword and a
/// medium fine ruby lie east of the player.
static FIXTURE: LazyLock<Arc<Content>> = LazyLock::new(|| fixture(false));

/// [`FIXTURE`] with Set open to the untrained character the scenario starts as (`from = 0`), for
/// a replay, which starts as the scenario does.
static OPEN: LazyLock<Arc<Content>> = LazyLock::new(|| fixture(true));

fn fixture(open: bool) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    if open {
        files.insert(
            "data/recipes/jewelcraft/set.toml",
            "[[recipe]]\nid = \"set\"\nuse = \"set\"\ndifficulty = 12\nseconds = 1200\n",
        );
    }
    for (name, k) in [
        ("test_alone", "\"floor\""),
        (
            "test_watched",
            "{ terrain = \"floor\", creature = \"kobold\" }",
        ),
    ] {
        files.insert(
            &format!("levels/{name}.toml"),
            &format!(
                "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{ROOM}'''\n\
                 [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\"k\" = {k}\n\
                 [[item]]\nat = [3, 2]\nobject = \"long_sword\"\n\
                 [[item]]\nat = [3, 2]\nobject = \"gem\"\n\
                 parts = [{{ of = \"ruby\", size = \"medium\", cut = \"fine\" }}]\n"
            ),
        );
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!(
                "name = \"T\"\nstart_level = \"{name}\"\n\
                 character = {{ race = \"human\", gender = \"female\", class = \"commoner\" }}\n"
            ),
        );
    }
    Arc::new(Content::load(&files).expect("fixture content is valid"))
}

/// A hall, alone or watched. The player is a human commoner carrying nothing, Jewelcraft and
/// Woodcraft 100 with no marks, every score 10 (a modifier of 0), `seed` their fate.
fn world(kobold: bool, seed: u64) -> World {
    let scenario = if kobold { "test_watched" } else { "test_alone" };
    let mut w = World::new(FIXTURE.clone(), scenario, Some(seed)).expect("world");
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    me.inventory.clear();
    me.equipment = Default::default();
    me.stats = Stats::new(PerStat::splat(10), PerStat::splat(10));
    for id in ["jewelcraft", "woodcraft"] {
        train(&mut w, id, 100);
    }
    w
}

fn skill(w: &World, id: &str) -> SkillId {
    w.content.skill_id(id).expect(id)
}

/// `id` at `value`, no marks, the other skills as they were.
fn train(w: &mut World, id: &str, value: u16) {
    let id = skill(w, id);
    let mut all: Vec<(SkillId, Trained)> = (w.player_mut().skills.iter())
        .filter(|(s, _)| *s != id)
        .collect();
    all.push((id, Trained { value, marks: 0 }));
    w.player_mut().skills = all.into_iter().collect();
}

fn marks(w: &World, id: &str) -> Option<u32> {
    let id = skill(w, id);
    w.state.player().skills.get(id).map(|t| t.marks)
}

/// A loose `gem` cut as `size` and `cut`, at the end of the pack.
fn gem(w: &mut World, gem: &str, size: &str, cut: &str) {
    let content = w.content.clone();
    let gems = content.gems().expect("gems");
    let cutting = crate::content::Cutting {
        size: gems.size_id(size).expect(size),
        cut: gems.cut_id(cut).expect(cut),
    };
    let m = content.material_id(gem).expect(gem);
    let part = ItemPart::gem(&content, None, m, cutting);
    w.player_mut()
        .inventory
        .push(Stack::one(gems.object).with_parts(vec![part]));
}

/// One of `object`, at the end of the pack.
fn item(w: &mut World, object: &str) {
    let id = w.content.object_id(object).expect(object);
    w.player_mut().inventory.push(Stack::one(id));
}

/// `count` pieces of `material` stock, at the end of the pack.
fn stock(w: &mut World, material: &str, count: u32) {
    let m = w.content.material_id(material).expect(material);
    let stack = crate::piece::make(&w.content, m, count).expect("stock");
    w.player_mut().inventory.push(stack);
}

fn texts(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.text.clone()).collect()
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

fn last(w: &World) -> String {
    (w.messages().iter().last()).map_or_else(String::new, |m| m.text.clone())
}

fn pack(w: &World) -> Vec<String> {
    (w.state.player().inventory.iter())
        .map(|s| w.lore().describe(s))
        .collect()
}

fn recipe(w: &World, id: &str) -> RecipeId {
    w.content.recipes().id(id).expect(id)
}

fn apply(w: &mut World, text: &str) {
    for action in parse_actions(text).expect("parses") {
        w.apply(&action);
    }
}

#[test]
fn the_band_is_set_from_a_margin_of_0_and_a_natural_2_shatters() {
    assert_eq!(set_band(12, -1), SetBand::Chipped);
    assert_eq!(set_band(12, 0), SetBand::Set);
    assert_eq!(set_band(2, 40), SetBand::Shattered);
    assert_eq!(set_band(24, -10), SetBand::Set);
}

#[test]
fn a_set_gem_becomes_the_ornament_and_the_name_follows() {
    let mut w = world(false, 1);
    item(&mut w, "long_sword");
    gem(&mut w, "ruby", "medium", "fine");
    set_out_for_test(&mut w, 0, 1, SetBand::Set);
    assert_eq!(pack(&w), ["a ruby-pommelled long sword"]);
    assert!(
        said(
            &w,
            "You set the ruby. It is now a ruby-pommelled long sword."
        ),
        "{:?}",
        texts(&w)
    );
    // The gem keeps its size and cut in the setting.
    let parts = w.state.player().inventory[0].parts(&w.content);
    let set = parts.iter().find(|p| p.gem.is_some()).expect("a gem part");
    let gems = w.content.gems().expect("gems");
    let cutting = set.gem.expect("cut");
    assert_eq!(gems.cut(cutting.cut).id, "fine");
    assert_eq!(gems.size(cutting.size).id, "medium");
    // Set now, it is no longer offered.
    assert!(!settable(&w.content, &w.state.player().inventory[0]));

    // A ring takes its gem into its name with its metal.
    let mut w = world(false, 1);
    item(&mut w, "silver_ring");
    gem(&mut w, "sapphire", "small", "fair");
    set_out_for_test(&mut w, 0, 1, SetBand::Set);
    assert_eq!(pack(&w), ["a silver and sapphire ring"]);
}

#[test]
fn a_failure_chips_the_gem_and_leaves_the_item() {
    for (size, cut, left) in [
        ("medium", "fine", Some("a medium ruby")),
        // Crude stays crude: a size goes instead.
        ("large", "crude", Some("a medium crude ruby")),
        // A tiny crude stone has nothing to lose: it is gone.
        ("tiny", "crude", None),
    ] {
        let mut w = world(false, 1);
        item(&mut w, "long_sword");
        gem(&mut w, "ruby", size, cut);
        set_out_for_test(&mut w, 0, 1, SetBand::Chipped);
        let mut want = vec!["a long sword".to_string()];
        want.extend(left.map(String::from));
        assert_eq!(pack(&w), want, "{size} {cut}: {:?}", texts(&w));
        match left {
            Some(_) => assert!(said(&w, "The ruby slips and chips.")),
            None => assert!(said(&w, "The ruby shatters.")),
        }
    }
    // A natural 2 shatters whatever the gem.
    let mut w = world(false, 1);
    item(&mut w, "long_sword");
    gem(&mut w, "ruby", "huge", "flawless");
    set_out_for_test(&mut w, 0, 1, SetBand::Shattered);
    assert_eq!(pack(&w), ["a long sword"]);
    assert!(said(&w, "The ruby shatters."));
}

#[test]
fn the_steps_offer_unset_items_and_cut_gems_only() {
    let mut w = world(false, 1);
    item(&mut w, "long_sword");
    gem(&mut w, "ruby", "medium", "uncut");
    gem(&mut w, "garnet", "small", "fair");
    item(&mut w, "gold_amulet"); // its ruby is set already
    let ring = w.content.object_id("iron_ring").expect("ring");
    let set = ItemPart::gem(
        &w.content,
        w.content.part_id("setting"),
        w.content.material_id("topaz").expect("topaz"),
        w.content.gems().expect("gems").plain(),
    );
    w.player_mut()
        .inventory
        .push(Stack::one(ring).with_parts(vec![set]));
    item(&mut w, "silver_ring");
    let steps = w.perception().craft_steps(recipe(&w, "set"));
    let at = |n: usize| -> Vec<usize> { steps[n].options.iter().map(|o| o.at).collect() };
    assert_eq!(steps.len(), 2);
    assert_eq!(
        (steps[0].label.as_str(), steps[1].label.as_str()),
        ("item", "gem")
    );
    assert_eq!(at(0), [0, 5], "the sword and the plain silver ring");
    assert_eq!(at(1), [2], "the cut garnet, never the uncut ruby");
    assert_eq!(steps[1].options[0].short, "garnet");

    let plan = w
        .perception()
        .craft_plan(recipe(&w, "set"), &[Some(5), Some(2)])
        .expect("a plan");
    assert_eq!(plan.verb, "Set");
    assert_eq!(plan.name, "a silver and garnet ring");
    assert_eq!(plan.uses, "the garnet");
    assert_eq!(plan.time, "about 20 minutes");
}

#[test]
fn every_refusal_takes_no_time() {
    let mut w = world(false, 1);
    item(&mut w, "long_sword");
    gem(&mut w, "ruby", "medium", "uncut");
    gem(&mut w, "garnet", "small", "fair");
    item(&mut w, "gold_amulet");
    item(&mut w, "iron_ring");
    apply(&mut w, "equip:left_ring:e");
    let before = w.state.time;
    for (picks, line) in [
        ("craft:set", "You must choose the item."),
        ("craft:set:a", "You must choose the gem."),
        ("craft:set:a,b,c", "That is more than the work takes."),
        ("craft:set:a,b", "Only a cut gem can be set."),
        ("craft:set:d,c", "It is set already."),
        ("craft:set:c,c", "No gem can be set in that."),
        ("craft:set:e,c", "Take it off first."),
        ("craft:set:z,c", "You carry no such thing."),
        ("craft:set:a,z", "You carry no such thing."),
    ] {
        apply(&mut w, picks);
        assert_eq!(last(&w), line, "{picks}");
    }
    train(&mut w, "jewelcraft", 5);
    apply(&mut w, "craft:set:a,c");
    assert_eq!(last(&w), "That is beyond your skill.");
    assert_eq!(w.state.time, before);
}

#[test]
fn a_setting_takes_twenty_minutes_and_marks_jewelcraft() {
    let (mut set, mut chipped) = (false, false);
    for seed in 1..40 {
        let mut w = world(false, seed);
        item(&mut w, "long_sword");
        gem(&mut w, "ruby", "medium", "fine");
        let start = w.state.time;
        apply(&mut w, "craft:set:a,b");
        assert!(said(
            &w,
            "You begin to set the medium fine ruby in the long sword."
        ));
        assert_eq!(w.state.time.since(start).as_seconds(), 1200);
        assert_eq!(marks(&w, "jewelcraft"), Some(1));
        let p = pack(&w);
        if said(&w, "The ruby slips and chips.") {
            assert_eq!(p, ["a long sword", "a medium ruby"]);
            chipped = true;
        } else if said(&w, "The ruby shatters.") {
            assert_eq!(p, ["a long sword"]);
        } else {
            assert_eq!(p, ["a ruby-pommelled long sword"], "{:?}", texts(&w));
            set = true;
        }
    }
    assert!(set && chipped);
}

#[test]
fn an_interrupted_setting_spends_nothing() {
    let mut w = world(true, 4);
    item(&mut w, "long_sword");
    gem(&mut w, "ruby", "medium", "fine");
    apply(&mut w, "craft:set:a,b");
    assert!(
        (w.messages().iter()).any(|m| m.text.ends_with("you put down your work.")),
        "{:?}",
        texts(&w)
    );
    assert_eq!(pack(&w), ["a long sword", "a medium fine ruby"]);
}

#[test]
fn each_ring_and_amulet_is_made_of_its_own_material_and_never_graded() {
    for (id, material, made) in [
        ("iron_ring", "iron", "an iron ring"),
        ("silver_ring", "silver", "a silver ring"),
        ("gold_ring", "gold", "a gold ring"),
        ("bone_amulet", "ivory", "an ivory amulet"),
        ("jade_amulet", "jade", "a jade amulet"),
        ("amber_amulet", "amber", "an amber amulet"),
    ] {
        let mut w = world(false, 1);
        stock(&mut w, material, 1);
        item(&mut w, "knife");
        // A margin of 20 would be a Masterwork weapon; jewellery has no aspect.
        make_out_for_test(&mut w, id, &[Some(0)], 12, 20);
        assert_eq!(last(&w), format!("You have made {made}."), "{id}");
        let p = pack(&w);
        assert_eq!(p, ["a knife", made], "{id}: one piece is spent");
        let at = w.state.player().inventory.len() - 1;
        assert_eq!(w.state.player().inventory[at].worked(), None, "{id}");
    }
    // A ring is its own metal: iron stock never makes a gold ring.
    let mut w = world(false, 1);
    stock(&mut w, "iron", 1);
    let plan = w
        .perception()
        .craft_plan(recipe(&w, "gold_ring"), &[Some(0)]);
    assert_eq!(plan, Err("That will not do for the metal.".into()));
    let plan = w
        .perception()
        .craft_plan(recipe(&w, "iron_ring"), &[Some(0)])
        .expect("a plan");
    assert_eq!(
        (plan.verb.as_str(), plan.uses.as_str()),
        ("Make", "an iron ingot")
    );
}

#[test]
fn the_gold_amulet_must_have_a_cut_gem() {
    let mut w = world(false, 1);
    stock(&mut w, "gold", 1);
    gem(&mut w, "ruby", "medium", "uncut");
    let row = |w: &World| {
        let jewel = skill(w, "jewelcraft");
        let rows = w.perception().recipes(jewel);
        let row = rows.iter().find(|r| r.id == recipe(w, "gold_amulet"));
        row.and_then(|r| r.unavailable)
            .map(|u| u.reason(&w.content))
    };
    assert_eq!(row(&w).as_deref(), Some("no gem to hand"));
    let before = w.state.time;
    apply(&mut w, "craft:gold_amulet:a,b");
    assert_eq!(last(&w), "You have no gem to hand.");
    assert_eq!(w.state.time, before);

    gem(&mut w, "sapphire", "small", "fine");
    assert_eq!(row(&w), None);
    let steps = w.perception().craft_steps(recipe(&w, "gold_amulet"));
    let labels: Vec<(&str, bool)> = (steps.iter())
        .map(|s| (s.label.as_str(), s.optional))
        .collect();
    assert_eq!(labels, [("chain", false), ("setting", false)]);
    assert_eq!(
        (steps[1].options.iter()).map(|o| o.at).collect::<Vec<_>>(),
        [2]
    );
    make_out_for_test(&mut w, "gold_amulet", &[Some(0), Some(2)], 12, 0);
    assert_eq!(last(&w), "You have made a gold and sapphire amulet.");
    assert_eq!(
        pack(&w),
        ["an uncut medium ruby", "a gold and sapphire amulet"]
    );
}

#[test]
fn the_bone_amulet_is_woodcrafts_by_hand() {
    let mut w = world(false, 1);
    let id = recipe(&w, "bone_amulet");
    assert_eq!(w.content.recipe(id).skill, skill(&w, "woodcraft"));
    stock(&mut w, "bone", 1);
    let rows = w.perception().recipes(skill(&w, "woodcraft"));
    let row = rows.iter().find(|r| r.id == id).expect("listed");
    assert_eq!(row.unavailable, Some(RecipeLack::Knife));
    item(&mut w, "knife");
    apply(&mut w, "craft:bone_amulet:a");
    assert_eq!(marks(&w, "woodcraft"), Some(1));
    assert_eq!(marks(&w, "jewelcraft"), Some(0));
}

#[test]
fn a_setting_replays() {
    // As the scenario starts: a long sword and a medium fine ruby lie east of the player.
    // A gnome, trained in Jewelcraft from birth: the untrained make nothing (#1387).
    let gnome: crate::character::CharacterSpec = "gnome,female,commoner".parse().expect("spec");
    let mut w =
        World::with_character(OPEN.clone(), "test_alone", Some(8), Some(&gnome)).expect("world");
    let mut replay = Replay::new(&w);
    for action in parse_actions("move:E pickup").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let sword = (w.state.player().inventory.iter())
        .position(|s| w.lore().title(s) == "long sword")
        .expect("the sword");
    let ruby = (w.state.player().inventory.iter())
        .position(|s| s.gem(&w.content).is_some())
        .expect("the ruby");
    let text = format!("craft:set:{sword},{ruby} wait");
    for action in parse_actions(&text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(
        said(
            &w,
            "You begin to set the medium fine ruby in the long sword."
        ),
        "{:?}",
        texts(&w)
    );
    let (verified, divergence) = replay.play(OPEN.clone()).expect("replays");
    assert_eq!(divergence, None);
    assert_eq!(verified.state_hash(), w.state_hash());
}
