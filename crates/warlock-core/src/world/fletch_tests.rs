//! Fletching on the recipes (spec §8 "Fletching"; docs/dev/crafting.md "Arrows and bolts"; #1021,
//! #1336): the arrows and bolts of Woodcraft (`count = "fletch"`), headed by a metal arrowhead, a
//! bone shard, a flint block, or nothing (brittle); the arrowheads of Metalcraft; the sling stones
//! and the flaming arrows; wood free in the woods; the knife and the launcher; the ranger's
//! Fletcher; the marks; and a replay.

use super::*;
use crate::ability::Ability;
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, ObjectId};
use crate::item_parts::ItemPart;
use crate::object::Stack;
use crate::replay::Replay;
use crate::skills::{SkillId, Trained};
use crate::stats::{PerStat, Stats};
use crate::test_support::CONTENT as BUILTIN;
use crate::world::{CraftStepKind, RecipeLack};
use std::sync::LazyLock;

/// A hall with an anvil at (5, 2), the player beside it at (4, 3), and at `k` a kobold, when
/// asked for.
const ROOM: &str = "\
###########
#.........#
#....A....#
#...@.....#
#.........#
#.......k.#
###########
";

/// The built-in content with [`ROOM`] twice: with no one in it, and with a kobold.
static FIXTURE: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
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
                 [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
                 \"A\" = \"anvil\"\n\"k\" = {k}\n"
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
});

/// A hall with no one in it, or with a kobold in sight. The player is a human commoner carrying
/// nothing, every score 10 (a modifier of 0), trained in Woodcraft at 1 (the least, which adds
/// nothing to a batch's count; the untrained make none, #1387), `seed` their fate.
fn world(kobold: bool, seed: u64) -> World {
    let scenario = if kobold { "test_watched" } else { "test_alone" };
    let mut w = World::new(FIXTURE.clone(), scenario, Some(seed)).expect("world");
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    me.inventory.clear();
    me.equipment = Default::default();
    me.stats = Stats::new(PerStat::splat(10), PerStat::splat(10));
    set_skill(&mut w, "woodcraft", Some(1));
    w
}

fn object(w: &World, id: &str) -> ObjectId {
    w.content.object_id(id).expect(id)
}

fn skill(w: &World, id: &str) -> SkillId {
    w.content.skill_id(id).expect(id)
}

/// The player trained in skill `id` at `value` (`None`: untrained).
fn set_skill(w: &mut World, id: &str, value: Option<u16>) {
    let id = skill(w, id);
    let skills = &mut w.player_mut().skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(s, _)| *s != id).collect();
    if let Some(value) = value {
        all.push((id, Trained { value, marks: 0 }));
    }
    *skills = all.into_iter().collect();
}

/// `stack` at the end of the pack; its position.
fn carry(w: &mut World, stack: Stack) -> usize {
    let me = w.player_mut();
    me.inventory.push(stack);
    me.inventory.len() - 1
}

/// One of object `id` into the pack, at its end.
fn give(w: &mut World, id: &str) -> usize {
    let stack = Stack::one(object(w, id));
    carry(w, stack)
}

/// `count` pieces of `material`: ingots, billets, shards, blocks.
fn pieces(w: &mut World, material: &str, count: u32) -> usize {
    let m = w.content.material_id(material).expect(material);
    let stack = crate::piece::make(&w.content, m, count).expect("pieces");
    carry(w, stack)
}

/// `count` arrowheads of `metal`.
fn arrowheads(w: &mut World, metal: &str, count: u32) -> usize {
    let content = w.content.clone();
    let m = content.material_id(metal).expect(metal);
    let mut stack = Stack::new(object(w, "arrowhead"), count);
    if metal != "iron" {
        stack = stack.with_parts(vec![ItemPart::material(&content, None, m)]);
    }
    carry(w, stack)
}

/// Object `id` ready in the missile weapon slot.
fn ready(w: &mut World, id: &str) {
    let item = give(w, id);
    let slot = w.content.slot_id("missile_weapon").expect("slot");
    w.player_mut().equipment.put(&[slot], item);
}

/// A knife in the pack, and `launcher` ready.
fn equipped(kobold: bool, seed: u64, launcher: &str) -> World {
    let mut w = world(kobold, seed);
    give(&mut w, "knife");
    ready(&mut w, launcher);
    w
}

/// Every score `score`: a modifier far above (60) or below (1) any roll's reach.
fn scores(w: &mut World, score: u8) {
    w.player_mut().stats = Stats::new(PerStat::splat(score), PerStat::splat(score));
}

fn count_of(w: &World, id: &str) -> u32 {
    let id = object(w, id);
    (w.state.player().inventory.iter())
        .filter(|s| s.object == id)
        .map(|s| s.count)
        .sum()
}

/// The pack as the inventory lists it.
fn pack(w: &World) -> Vec<String> {
    let lore = w.lore();
    (w.state.player().inventory.iter())
        .map(|s| lore.describe(s))
        .collect()
}

/// Recipe `id`'s row on its profession's screen: whether it is listed, and why it cannot be
/// made now.
fn row(w: &World, id: &str) -> Option<Option<RecipeLack>> {
    let recipe = w.content.recipes().id(id).expect(id);
    let def = w.content.recipe(recipe);
    (w.perception().recipes(def.skill).into_iter())
        .find(|r| r.id == recipe)
        .map(|r| r.unavailable)
}

fn takes(w: &World, id: &str) -> String {
    let recipe = w.content.recipes().id(id).expect(id);
    let def = w.content.recipe(recipe);
    (w.perception().recipes(def.skill).into_iter())
        .find(|r| r.id == recipe)
        .map(|r| r.takes)
        .expect("listed")
}

fn last_message(w: &World) -> String {
    (w.messages().iter().last()).map_or_else(String::new, |m| m.text.clone())
}

/// Applies `craft:ID:PICKS` (action text); the seconds it took.
fn craft(w: &mut World, text: &str) -> u64 {
    let start = w.state.time;
    for action in parse_actions(text).expect("actions") {
        w.apply(&action);
    }
    w.state.time.since(start).as_seconds()
}

/// The arrows recipe's steps as the wizard lists them.
fn steps(w: &World, id: &str) -> Vec<crate::world::CraftStep> {
    let recipe = w.content.recipes().id(id).expect(id);
    w.perception().craft_steps(recipe)
}

#[test]
fn the_head_step_lists_arrowheads_then_shards_and_blocks_then_none() {
    let mut w = equipped(false, 1, "short_bow");
    // The bow and the knife come first: positions 0 and 1.
    pieces(&mut w, "flint", 1); // c
    arrowheads(&mut w, "steel", 20); // d
    pieces(&mut w, "bone", 2); // e
    pieces(&mut w, "hardwood", 1); // f
    pieces(&mut w, "steel", 1); // g: no head
    let steps = steps(&w, "arrows");
    assert_eq!(steps.len(), 2, "the head, then the shaft: {steps:?}");
    let head = &steps[0];
    assert_eq!(
        (head.kind, head.label.as_str()),
        (CraftStepKind::Sources, "head")
    );
    assert!(head.optional, "None may be chosen");
    let options: Vec<(usize, &str, u32, bool)> = (head.options.iter())
        .map(|o| (o.at, o.short.as_str(), o.takes, o.each))
        .collect();
    assert_eq!(
        options,
        [
            (3, "steel", 1, true),
            (2, "flint", 1, false),
            (4, "bone", 1, false)
        ],
        "arrowheads, then the shards and blocks as carried; never an ingot"
    );
    let shaft = &steps[1];
    assert_eq!(
        (shaft.kind, shaft.label.as_str()),
        (CraftStepKind::Stock, "shaft")
    );
    assert_eq!(shaft.options.len(), 1);
    assert_eq!(shaft.options[0].at, 5);
    // The arrowheads are the head's, never a step of their own.
    assert!(steps.iter().all(|s| s.kind != CraftStepKind::Input));
}

/// A batch of `recipe` with `picks` made by a check that cannot fail (every score 60).
fn sure_batch(w: &mut World, text: &str) {
    scores(w, 60);
    let seconds = craft(w, text);
    assert_eq!(seconds, 600, "{text}: {}", last_message(w));
}

#[test]
fn a_metal_arrowhead_heads_one_arrow_and_gives_it_its_metal() {
    let mut w = equipped(false, 2, "short_bow");
    arrowheads(&mut w, "steel", 20); // c
    pieces(&mut w, "hardwood", 1); // d
    sure_batch(&mut w, "craft:arrows:c,d");
    let made = count_of(&w, "arrow");
    assert!((1..=4).contains(&made), "1d4 at Woodcraft 1: {made}");
    assert_eq!(count_of(&w, "arrowhead"), 20 - made, "one an arrow");
    assert_eq!(count_of(&w, "piece"), 0, "the billet is spent");
    let arrows = (w.state.player().inventory.iter())
        .find(|s| s.object == object(&w, "arrow"))
        .expect("made");
    let parts: Vec<&str> = (arrows.parts(&w.content).iter())
        .map(|p| w.content.material(p.material).id.as_str())
        .collect();
    assert_eq!(
        parts,
        ["steel", "hardwood"],
        "the head's metal, the shaft's wood"
    );
    assert!(
        last_message(&w).starts_with("You make "),
        "{}",
        last_message(&w)
    );
}

#[test]
fn a_shard_or_a_block_heads_the_whole_batch_and_is_spent_once() {
    for (material, name) in [("bone", "bone"), ("flint", "flint")] {
        let mut w = equipped(false, 3, "short_bow");
        set_skill(&mut w, "woodcraft", Some(100)); // 5 to 8 a batch
        pieces(&mut w, material, 2); // c
        pieces(&mut w, "wood", 1); // d
        sure_batch(&mut w, "craft:arrows:c,d");
        let made = count_of(&w, "arrow");
        assert!((5..=8).contains(&made), "{made}");
        let left = pack(&w);
        assert!(
            left.iter().any(|s| s.contains(&format!("{name} "))),
            "{left:?}"
        );
        let shard = w.content.material_id(material).expect(material);
        let kept: u32 = (w.state.player().inventory.iter())
            .filter(|s| super::craft::stack_material(&w.content, s) == Some(shard))
            .map(|s| s.count)
            .sum();
        assert_eq!(kept, 1, "one of two spent: {left:?}");
    }
}

#[test]
fn flint_tipped_and_bone_tipped_arrows_are_their_own_goods_and_flint_strikes_as_flint() {
    let mut w = equipped(false, 4, "short_bow");
    pieces(&mut w, "flint", 1); // c
    pieces(&mut w, "bone", 1); // d
    pieces(&mut w, "wood", 2); // e
    sure_batch(&mut w, "craft:arrows:c,e");
    // The flint block is gone (c): the bone is c now, the wood d.
    sure_batch(&mut w, "craft:arrows:c,d");
    let arrow = object(&w, "arrow");
    let stacks: Vec<&Stack> = (w.state.player().inventory.iter())
        .filter(|s| s.object == arrow)
        .collect();
    assert_eq!(stacks.len(), 2, "they do not stack: {:?}", pack(&w));
    let content = &w.content;
    let def = content.object(arrow);
    let flint = (stacks.iter())
        .find(|s| s.parts(content)[0].material == content.material_id("flint").expect("flint"))
        .expect("flint-headed");
    let (hit, damage) = crate::variant::weapon(def, &flint.parts(content), content);
    let iron = content.material(content.material_id("iron").expect("iron"));
    let stone = content.material(content.material_id("flint").expect("flint"));
    let (iron_hit, iron_damage) = iron.weapon().expect("iron strikes");
    let (flint_hit, flint_damage) = stone.weapon().expect("flint strikes");
    assert_eq!(
        (hit, damage),
        (flint_hit - iron_hit, flint_damage - iron_damage)
    );
    assert_ne!((hit, damage), (0, 0), "flint is not iron");
}

#[test]
fn no_head_makes_brittle_arrows_of_the_shafts_wood() {
    let mut w = equipped(false, 5, "short_bow");
    pieces(&mut w, "hardwood", 1); // c
    sure_batch(&mut w, "craft:arrows:-,c");
    let made = count_of(&w, "brittle_arrow");
    assert!(made > 0 && count_of(&w, "arrow") == 0, "{:?}", pack(&w));
    let def = w.content.object(object(&w, "brittle_arrow"));
    assert_eq!(def.breaks, 100);
    let brittle = (w.state.player().inventory.iter())
        .find(|s| s.object == object(&w, "brittle_arrow"))
        .expect("made");
    let hardwood = w.content.material_id("hardwood").expect("hardwood");
    assert_eq!(brittle.parts(&w.content)[0].material, hardwood);
    // Even a master of Woodcraft, or a ranger: no heads, no plain arrows.
    let mut w = equipped(false, 5, "short_bow");
    set_skill(&mut w, "woodcraft", Some(200));
    w.player_mut().talents.take("fletcher", None);
    pieces(&mut w, "wood", 1);
    scores(&mut w, 60);
    assert_eq!(craft(&mut w, "craft:arrows:-,c"), 300, "Fletcher's half");
    assert_eq!(
        last_message(&w),
        "You make 11 brittle arrows.",
        "what a billet covers"
    );
    assert_eq!(count_of(&w, "arrow"), 0);
    assert!(count_of(&w, "brittle_arrow") > 0);
}

#[test]
fn bolts_are_the_twin_for_a_crossbow() {
    let mut w = equipped(false, 6, "crossbow");
    // A crossbow ready: arrows greyed, bolts not.
    pieces(&mut w, "flint", 1); // c
    pieces(&mut w, "wood", 3); // d
    assert_eq!(
        row(&w, "arrows"),
        Some(Some(RecipeLack::Launcher(object(&w, "arrow"))))
    );
    assert_eq!(
        RecipeLack::Launcher(object(&w, "arrow")).reason(&w.content),
        "no bow ready"
    );
    assert_eq!(row(&w, "bolts"), Some(None));
    assert!(
        takes(&w, "bolts").contains("a crossbow ready"),
        "{}",
        takes(&w, "bolts")
    );
    sure_batch(&mut w, "craft:bolts:c,d");
    assert!(count_of(&w, "bolt") > 0, "{:?}", pack(&w));
    assert_eq!(count_of(&w, "piece"), 2, "a billet for up to four bolts");
    sure_batch(&mut w, "craft:bolts:-,c");
    assert!(count_of(&w, "brittle_bolt") > 0);
    // Refused with no time with a bow ready.
    let mut w = equipped(false, 6, "short_bow");
    pieces(&mut w, "wood", 1);
    assert_eq!(craft(&mut w, "craft:bolts:-,c"), 0);
    assert_eq!(last_message(&w), "You have no crossbow ready.");
}

/// #1387: every recipe is its skill's. Untrained in Woodcraft, with all a batch takes, the
/// player has no Woodcraft row and makes no arrows, bolts, or sling stones, taking no time.
#[test]
fn the_untrained_make_no_arrows_bolts_or_sling_stones() {
    let mut w = equipped(false, 2, "short_bow");
    pieces(&mut w, "wood", 1); // c
    pieces(&mut w, "stone", 1); // d
    set_skill(&mut w, "woodcraft", None);
    let wood = skill(&w, "woodcraft");
    assert!(w.perception().recipes(wood).is_empty());
    let own = w.perception().abilities();
    assert!(!own.iter().any(|o| o.ability == Ability::Profession(wood)));
    for text in [
        "craft:arrows:-,c",
        "craft:bolts:-,c",
        "craft:sling_stones:d",
    ] {
        assert_eq!(craft(&mut w, text), 0, "{text}");
        assert_eq!(last_message(&w), "You are not trained in Woodcraft.");
    }
    assert_eq!(count_of(&w, "piece"), 2, "nothing spent");
    // Trained, the same pack makes a batch.
    set_skill(&mut w, "woodcraft", Some(1));
    sure_batch(&mut w, "craft:arrows:-,c");
    assert!(count_of(&w, "brittle_arrow") > 0, "{:?}", pack(&w));
}

#[test]
fn the_knife_and_the_launcher_are_asked_for_with_no_time() {
    let mut w = world(false, 7);
    pieces(&mut w, "wood", 1); // a
    assert_eq!(
        row(&w, "arrows"),
        Some(Some(RecipeLack::Knife)),
        "the trained have it"
    );
    assert_eq!(craft(&mut w, "craft:arrows:-,a"), 0);
    assert_eq!(last_message(&w), "You have no knife.");
    give(&mut w, "dagger"); // a dagger in the pack is a knife
    assert_eq!(
        row(&w, "arrows"),
        Some(Some(RecipeLack::Launcher(object(&w, "arrow"))))
    );
    assert_eq!(craft(&mut w, "craft:arrows:-,a"), 0);
    assert_eq!(last_message(&w), "You have no bow ready.");
    // A sling shoots stones, a bow in the pack is not ready.
    ready(&mut w, "sling");
    assert!(matches!(
        row(&w, "arrows"),
        Some(Some(RecipeLack::Launcher(_)))
    ));
    let mut w = world(false, 7);
    give(&mut w, "knife");
    give(&mut w, "short_bow");
    assert!(matches!(
        row(&w, "arrows"),
        Some(Some(RecipeLack::Launcher(_)))
    ));
    // Ready, with no wood: "no wood to hand".
    let mut w = equipped(false, 7, "short_bow");
    let lack = row(&w, "arrows").flatten().expect("no wood");
    assert_eq!(lack.reason(&w.content), "no wood to hand");
    pieces(&mut w, "wood", 1);
    assert_eq!(row(&w, "arrows"), Some(None));
    let wood = skill(&w, "woodcraft");
    assert!(
        (w.perception().abilities().iter()).any(|a| a.ability == Ability::Profession(wood)),
        "Woodcraft is listed as its profession"
    );
    for knife in ["dagger", "stiletto", "knife"] {
        assert!(w.content.is_knife(object(&w, knife)), "{knife}");
    }
    assert!(!w.content.is_knife(object(&w, "short_sword")));
}

#[test]
fn the_row_says_what_a_batch_takes() {
    let w = equipped(false, 8, "short_bow");
    assert_eq!(
        takes(&w, "arrows"),
        "an arrowhead for each made or a bone shard or a block of flint, a wood billet, a knife, \
         a bow ready, about 10 minutes"
    );
    assert_eq!(
        takes(&w, "sling_stones"),
        "a block of stone, about 10 minutes"
    );
    assert_eq!(
        takes(&w, "flaming_arrows"),
        "an arrow for each made, a flask of oil, about 10 minutes"
    );
}

#[test]
fn the_margin_scales_the_count_by_the_bands() {
    use super::craft_make::batch_made;
    let settings = BUILTIN.recipes().settings.clone();
    let settings = settings.as_ref();
    for (margin, made) in [(5, 5), (0, 5), (-1, 3), (-3, 3), (-4, 1), (-6, 1), (-7, 0)] {
        assert_eq!(batch_made(settings, 12, margin, 5), made, "margin {margin}");
    }
    assert_eq!(batch_made(settings, 2, 20, 5), 0, "a natural 2 makes none");
    assert_eq!(batch_made(settings, 24, -20, 5), 5, "a natural 24 all");
    // So low a score no batch is made: the spoiled batch says so, and the billet is kept.
    let mut spoiled = None;
    for seed in 0..100 {
        let mut w = equipped(false, seed, "short_bow");
        scores(&mut w, 1);
        pieces(&mut w, "wood", 1);
        craft(&mut w, "craft:arrows:-,c");
        if count_of(&w, "brittle_arrow") == 0 {
            spoiled = Some(w);
            break;
        }
    }
    let w = spoiled.expect("a spoiled batch");
    assert_eq!(last_message(&w), "You spoil the whole batch.");
    assert_eq!(count_of(&w, "piece"), 1, "a single billet comes back whole");
}

#[test]
fn arrowheads_are_forged_by_the_score_at_an_anvil() {
    let mut w = world(false, 9);
    set_skill(&mut w, "metalcraft", Some(10));
    pieces(&mut w, "steel", 2); // a
    assert_eq!(row(&w, "arrowheads"), Some(None), "beside the anvil");
    assert_eq!(
        takes(&w, "arrowheads"),
        "an iron ingot, an anvil, about 10 minutes"
    );
    sure_batch(&mut w, "craft:arrowheads:a");
    assert_eq!(count_of(&w, "arrowhead"), 20);
    assert_eq!(count_of(&w, "piece"), 1, "one ingot a batch");
    assert_eq!(last_message(&w), "You make 20 steel arrowheads.");
    let heads = (w.state.player().inventory.iter())
        .find(|s| s.object == object(&w, "arrowhead"))
        .expect("made");
    let steel = w.content.material_id("steel").expect("steel");
    assert_eq!(heads.parts(&w.content)[0].material, steel);
    // Metalcraft's alone: the untrained never see it.
    set_skill(&mut w, "metalcraft", None);
    assert_eq!(row(&w, "arrowheads"), None);
}

#[test]
fn sling_stones_need_no_knife() {
    let mut w = world(false, 10);
    pieces(&mut w, "stone", 1); // a
    assert_eq!(row(&w, "sling_stones"), Some(None), "no knife, no sling");
    sure_batch(&mut w, "craft:sling_stones:a");
    assert_eq!(count_of(&w, "sling_stone"), 10);
    assert_eq!(count_of(&w, "piece"), 0);
}

#[test]
fn flaming_arrows_take_an_arrow_each_and_a_flask_of_oil() {
    let mut w = world(false, 11);
    let arrows = Stack::new(object(&w, "arrow"), 12);
    carry(&mut w, arrows); // a
    give(&mut w, "flask_of_oil"); // b
    // Unknown, the oil is no input (#1383): refused, with no time.
    assert_eq!(craft(&mut w, "craft:flaming_arrows:a,b"), 0);
    assert_eq!(last_message(&w), "That will not do for the flasks of oil.");
    assert_eq!(count_of(&w, "flask_of_oil"), 1);
    let oil = object(&w, "flask_of_oil");
    w.learn_scroll(oil);
    sure_batch(&mut w, "craft:flaming_arrows:a,b");
    assert_eq!(count_of(&w, "flaming_arrow"), 10);
    assert_eq!(count_of(&w, "arrow"), 2, "one an arrow made");
    assert_eq!(count_of(&w, "flask_of_oil"), 0);
    // Fewer arrows, fewer made.
    let mut w = world(false, 11);
    let arrows = Stack::new(object(&w, "arrow"), 4);
    carry(&mut w, arrows);
    give(&mut w, "flask_of_oil");
    w.learn_scroll(oil);
    sure_batch(&mut w, "craft:flaming_arrows:a,b");
    assert_eq!(count_of(&w, "flaming_arrow"), 4);
    assert_eq!(count_of(&w, "arrow"), 0);
}

/// A level-one ranger with their kit, in `scenario`.
fn ranger(scenario: &str, seed: u64) -> World {
    let spec: CharacterSpec = "human,male,ranger".parse().expect("a character");
    let spec = spec.with_talents(vec!["keen_ears".into(), "lean".into(), "healthy".into()]);
    World::start_run(
        BUILTIN.clone(),
        scenario,
        Some(seed),
        Some(&spec),
        None,
        true,
    )
    .expect("world")
}

#[test]
fn fletcher_is_half_the_time_twice_the_count_and_a_better_check() {
    let r = ranger("showcase", 5);
    assert!(r.state.player().talents.has("fletcher"), "a ranger's");
    let numbers = (r.state.player().talent_mods(&r.content).craft.fletcher).expect("the effect");
    assert_eq!(
        (numbers.check, numbers.count_times, numbers.time_percent),
        (4, 2, 50)
    );
    // The same character and fate with and without it: half the time, twice the count.
    let with = |seed, fletches: bool| {
        let mut w = equipped(false, seed, "short_bow");
        if fletches {
            w.player_mut().talents.take("fletcher", None);
        }
        pieces(&mut w, "bone", 1); // c
        pieces(&mut w, "wood", 2); // d
        scores(&mut w, 60);
        let seconds = craft(&mut w, "craft:arrows:c,d");
        (seconds, count_of(&w, "arrow"))
    };
    for seed in 0..30 {
        let (slow, plain) = with(seed, false);
        let (quick, twice) = with(seed, true);
        assert_eq!((slow, quick), (600, 300));
        assert_eq!(twice, 2 * plain, "seed {seed}");
    }
    // +4 on the check: a character for whom a batch is sometimes spoiled spoils fewer.
    let spoiled = |fletches: bool| {
        (0..200)
            .filter(|&seed| {
                let mut w = equipped(false, seed, "short_bow");
                scores(&mut w, 4);
                if fletches {
                    w.player_mut().talents.take("fletcher", None);
                }
                pieces(&mut w, "wood", 1);
                craft(&mut w, "craft:arrows:-,c");
                count_of(&w, "brittle_arrow") == 0
            })
            .count()
    };
    assert!(spoiled(true) < spoiled(false));
}

#[test]
fn wood_is_free_in_the_woods_and_a_billet_elsewhere() {
    // The generated forest area's glade: no shaft step, no billet, no "no wood to hand".
    let mut w = World::new(BUILTIN.clone(), "forest", Some(3)).expect("world");
    let me = w.player_mut();
    me.inventory.clear();
    me.equipment = Default::default();
    give(&mut w, "knife");
    ready(&mut w, "short_bow");
    assert_eq!(row(&w, "arrows"), Some(None));
    let steps = steps(&w, "arrows");
    assert_eq!(steps.len(), 1, "the head alone: {steps:?}");
    assert!(
        !takes(&w, "arrows").contains("billet"),
        "{}",
        takes(&w, "arrows")
    );
    scores(&mut w, 60);
    craft(&mut w, "craft:arrows:-");
    let brittle = (w.state.player().inventory.iter())
        .find(|s| s.object == object(&w, "brittle_arrow"))
        .expect("made from what wood there is");
    let wood = w.content.material_id("wood").expect("wood");
    assert_eq!(brittle.parts(&w.content)[0].material, wood);
    // A sword's recipe is no arrow: wood is free only for those.
    let staff = (w.content.recipes().list.iter()).any(|r| {
        r.result.is_some()
            && !super::fletch::is_fletch(r)
            && r.parts.iter().any(|p| {
                super::fletch::part_free(&w.content, w.state.current(), w.state.player(), r, p.part)
            })
    });
    assert!(!staff);
}

#[test]
fn a_batch_trains_woodcraft_and_an_interruption_spends_nothing() {
    let mut w = equipped(false, 12, "short_bow");
    set_skill(&mut w, "woodcraft", Some(10));
    pieces(&mut w, "wood", 1);
    let marks = |w: &World| {
        w.state
            .player()
            .skills
            .get(skill(w, "woodcraft"))
            .map(|t| t.marks)
    };
    craft(&mut w, "craft:arrows:-,c");
    assert_eq!(marks(&w), Some(1), "every batch");
    // A kobold in sight stops the work: the billet is kept, nothing made.
    let mut w = equipped(true, 12, "short_bow");
    pieces(&mut w, "wood", 1);
    let spent = craft(&mut w, "craft:arrows:-,c");
    assert!(spent > 0 && spent < 600, "{spent}");
    assert_eq!(count_of(&w, "piece"), 1);
    assert_eq!(count_of(&w, "brittle_arrow"), 0);
    assert!(
        last_message(&w).contains("put down your work"),
        "{}",
        last_message(&w)
    );
}

#[test]
fn fletching_replays_verify() {
    // A ranger in the forest area's glade: wood is free, and no heads make brittle arrows.
    let mut w = ranger("forest", 9);
    let mut replay = Replay::new(&w);
    for action in parse_actions("craft:arrows:- wait craft:arrows:-").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let (verified, divergence) = replay.play(BUILTIN.clone()).expect("replays");
    assert_eq!(divergence, None);
    assert_eq!(verified.state_hash(), w.state_hash());
    assert!(count_of(&w, "brittle_arrow") > 0, "{:?}", pack(&w));
}
