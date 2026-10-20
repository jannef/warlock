//! Food rotting in the world (spec §8 "Hunger"): its stages by age, rotting away from a pack at
//! the exact moment however the time is cut (the player's, told; a creature's, silently, and
//! frozen on another level), piles cleaned as they are perceived, stacks keeping the oldest age,
//! eating rotten food of any kind, shops and rotten food, the dev dump, saves, and replays.

use super::*;
use crate::action::{Meal, Positions, parse_actions};
use crate::content::ContentFiles;
use crate::object::Stack;
use crate::replay::Replay;
use crate::rot::{self, Stage as Age};

/// The built-in hunger file, to edit for a test.
const HUNGER: &str = include_str!("../../../../assets/data/hunger.toml");

/// Food underfoot (two loaves of bread on top, keeping five days and lasting ten; a food
/// ration, keeping a month), and stairs down to the cellar.
const LARDER: &str = r##"
name = "Larder"
ambient = [200, 200, 200, 0]
map = '''
#########
#@.....>#
#########
'''
[legend]
"@" = { terrain = "floor", spawn = "player", objects = ["bread*2", "ration"] }
">" = { terrain = "stairs_down", to = "cellar" }
"##;

/// Apples on the floor (keeping two weeks, lasting four), and a kobold who fights no one,
/// carrying a loaf of bread.
const CELLAR: &str = r##"
name = "Cellar"
ambient = [200, 200, 200, 0]
map = '''
#######
#<a..k#
#######
'''
[legend]
"<" = { terrain = "stairs_up", to = "larder" }
"a" = { terrain = "floor", objects = ["apple*2"] }
"k" = { terrain = "floor", creature = "kobold", faction = "peaceful", carry = ["bread"] }
"##;

/// The bread's shelf life: it keeps five days and lasts ten.
const KEEPS: u64 = 5 * 24;
const LASTS: u64 = 10 * 24;

/// The built-in content with the test levels, and a player who burns a point of nutrition a day
/// (so rests of weeks never starve them), with `data/hunger.toml` edited further by `edits`.
fn content_with(edits: &[(&str, &str)]) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    for (name, level) in [("larder", LARDER), ("cellar", CELLAR)] {
        files.insert(&format!("levels/{name}.toml"), level);
    }
    files.insert(
        "scenarios/larder.toml",
        "name = \"larder\"\nstart_level = \"larder\"\nhp = 1000\n",
    );
    let mut hunger = HUNGER.replace("burn_seconds = 60", "burn_seconds = 86400");
    for (from, to) in edits {
        assert!(hunger.contains(from), "{from}");
        hunger = hunger.replace(from, to);
    }
    files.insert("data/hunger.toml", &hunger);
    Arc::new(Content::load(&files).expect("test content is valid"))
}

fn content() -> Arc<Content> {
    content_with(&[])
}

fn larder(content: &Arc<Content>) -> World {
    World::new(content.clone(), "larder", None).expect("world")
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn texts(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.text.clone()).collect()
}

/// How many times `text` was logged.
fn said(w: &World, text: &str) -> usize {
    (w.messages().iter())
        .filter(|m| m.text == text)
        .map(|m| m.count as usize)
        .sum()
}

fn object(w: &World, id: &str) -> crate::content::ObjectId {
    w.content().object_id(id).unwrap_or_else(|| panic!("{id}"))
}

fn hours(h: u64) -> Duration {
    Duration::hours(h)
}

/// The stack of `id` in `stacks`, if any.
fn find<'a>(w: &World, stacks: &'a [Stack], id: &str) -> Option<&'a Stack> {
    let o = object(w, id);
    stacks.iter().find(|s| s.object == o)
}

fn underfoot(w: &World) -> Vec<Stack> {
    w.state.current().objects_at(w.player_pos()).to_vec()
}

#[test]
fn food_is_made_as_it_comes_into_the_world_and_ages_from_then() {
    let content = content();
    let mut w = larder(&content);
    let start = w.time();
    let bread = find(&w, &underfoot(&w), "bread").cloned().expect("bread");
    assert_eq!(bread.died, Some(start), "made as the level was");
    assert!(bread.is_valid_for(&content));
    // Only food knows its age (and the dagger of a fixture does not).
    let stage = |w: &World, id: &str| w.stage(find(w, &underfoot(w), id).expect(id));
    assert_eq!(stage(&w, "bread"), Some(Age::Fresh));
    run(&mut w, "skip:59h");
    assert_eq!(stage(&w, "bread"), Some(Age::Fresh));
    run(&mut w, "skip:1h");
    assert_eq!(stage(&w, "bread"), Some(Age::Old), "past half of five days");
    run(&mut w, "skip:60h");
    assert_eq!(stage(&w, "bread"), Some(Age::Rotten));
    assert_eq!(
        stage(&w, "ration"),
        Some(Age::Fresh),
        "a ration keeps a month"
    );
    run(&mut w, "skip:119h");
    assert_eq!(stage(&w, "bread"), Some(Age::Rotten));
    // Ten days on it has rotted away, gone from the floor as it is perceived.
    run(&mut w, "skip:1h");
    assert!(find(&w, &underfoot(&w), "bread").is_none(), "gone");
    assert!(find(&w, &underfoot(&w), "ration").is_some());
    // Food does not rot without hunger.
    let mut files = ContentFiles::builtin();
    files.insert("levels/larder.toml", LARDER);
    files.insert("levels/cellar.toml", CELLAR);
    files.insert(
        "scenarios/larder.toml",
        "name = \"l\"\nstart_level = \"larder\"\n",
    );
    files.remove("data/hunger.toml");
    let content = Arc::new(Content::load(&files).expect("valid"));
    let mut w = larder(&content);
    run(&mut w, "skip:300h");
    let bread = find(&w, &underfoot(&w), "bread")
        .cloned()
        .expect("still there");
    assert_eq!(w.stage(&bread), None);
}

/// A larder world in which the player has picked up the bread and the ration, and how long
/// from then until the bread rots away.
fn carrying() -> (World, u64) {
    let mut w = larder(&content());
    let start = w.time();
    run(&mut w, "pickup");
    let bread = find(&w, &w.state.player().inventory, "bread").expect("carried");
    assert_eq!(bread.count, 2);
    let gone = rot::gone_at(w.content(), bread).expect("it rots");
    assert_eq!(gone, start + hours(LASTS));
    let left = gone.since(w.time()).as_seconds();
    (w, left)
}

#[test]
fn food_in_the_pack_rots_away_at_its_moment_however_the_time_is_cut() {
    const LOST: &str = "Your 2 loaves of bread have rotted away.";
    let (at_once, left) = carrying();
    let (player, start) = (at_once.state.player, at_once.time());
    let total = left + 3600;
    let pass = |w: &mut World, cuts: &[u64]| {
        let mut passed = 0;
        for &cut in cuts.iter().chain([&(total - cuts.iter().sum::<u64>())]) {
            w.pass_time(
                std::iter::once(player),
                start + Duration::seconds(passed),
                cut,
                &mut Vec::new(),
            );
            passed += cut;
        }
    };
    let mut worlds = Vec::new();
    for cuts in [
        vec![],
        vec![3599; usize::try_from(left / 3599).unwrap()],
        vec![left],
        vec![left - 1, 1],
        vec![left - 7, 6, 1],
    ] {
        let (mut w, _) = carrying();
        pass(&mut w, &cuts);
        worlds.push(w);
    }
    for w in &worlds {
        let inventory = &w.state.player().inventory;
        assert!(find(w, inventory, "bread").is_none(), "rotted away");
        assert!(find(w, inventory, "ration").is_some(), "a ration keeps");
        assert_eq!(said(w, LOST), 1, "{:#?}", texts(w));
        assert_eq!(inventory, &worlds[0].state.player().inventory);
        assert_eq!(
            w.state.player().satiation,
            worlds[0].state.player().satiation
        );
    }
    // Up to the second before, it is there; at that second it is gone.
    let (mut w, left) = carrying();
    w.pass_time(std::iter::once(player), start, left - 1, &mut Vec::new());
    assert!(find(&w, &w.state.player().inventory, "bread").is_some());
    assert_eq!(said(&w, LOST), 0);
    let at = start + Duration::seconds(left - 1);
    w.pass_time(std::iter::once(player), at, 1, &mut Vec::new());
    assert!(find(&w, &w.state.player().inventory, "bread").is_none());
    assert_eq!(said(&w, LOST), 1);
}

#[test]
fn rests_of_any_length_lose_the_food_at_the_same_moment_without_being_stopped() {
    const LOST: &str = "Your 2 loaves of bread have rotted away.";
    let rested = |rests: &str| {
        let (mut w, _) = carrying();
        run(&mut w, rests);
        w
    };
    // Ten days and an hour, cut three ways.
    let worlds = [
        rested("rest:24h*10 rest:1h"),
        rested("rest:16h*15 rest:1h"),
        rested("rest:7h*34 rest:2h rest:1h"),
    ];
    let turn = |w: &World| {
        let lost = (w.messages().iter()).find(|m| m.text == LOST);
        lost.map(|m| m.turn).expect("told")
    };
    for w in &worlds {
        assert_eq!(w.time(), worlds[0].time());
        assert!(find(w, &w.state.player().inventory, "bread").is_none());
        assert_eq!(said(w, LOST), 1);
        // Each turn of a rest is six seconds, so the turn it was told on is its moment.
        assert_eq!(turn(w), turn(&worlds[0]));
        // The note did not stop the rest: every rest ran to its end.
        assert!(!texts(w).iter().any(|t| t.contains("You stop resting")));
    }
}

#[test]
fn stacks_of_food_keep_the_oldest_age_when_they_join() {
    let (mut w, _) = carrying();
    let made = find(&w, &w.state.player().inventory, "bread").unwrap().died;
    run(&mut w, "skip:100h");
    // A fresh loaf lies underfoot; picked up, it joins the old ones, which keep their age.
    let bread = object(&w, "bread");
    let mut fresh = Stack::one(bread);
    fresh.died = Some(w.time());
    let here = w.player_pos();
    w.state.current_mut().set_pile(here, vec![fresh]);
    run(&mut w, "pickup");
    let inventory = w.state.player().inventory.clone();
    let stacks: Vec<&Stack> = inventory.iter().filter(|s| s.object == bread).collect();
    assert_eq!(stacks.len(), 1, "one stack, not one per age");
    assert_eq!((stacks[0].count, stacks[0].died), (3, made));
    // Dropping some takes the age along.
    let letter = inventory.iter().position(|s| s.object == bread).unwrap();
    let c = crate::action::letter(letter).unwrap();
    run(&mut w, &format!("drop:{c}"));
    assert_eq!(find(&w, &underfoot(&w), "bread").unwrap().died, made);
}

#[test]
fn rotted_food_leaves_the_piles_of_other_levels_when_the_player_comes() {
    let content = content();
    let mut w = larder(&content);
    run(&mut w, "move:E*6");
    let arrived = w.time();
    run(&mut w, "descend");
    assert_eq!(w.state.current().id(), "cellar");
    let apples_at = Pos::new(2, 1);
    let apples = |w: &World| {
        let cellar = &w.state.levels["cellar"];
        find(w, cellar.objects_at(apples_at), "apple").cloned()
    };
    assert_eq!(
        apples(&w).unwrap().died,
        Some(arrived),
        "made as the cellar was"
    );
    run(&mut w, "ascend");
    // Four weeks and more later, the cellar's apples have rotted away, but nobody has looked.
    run(&mut w, "skip:24h*29");
    assert!(apples(&w).is_some(), "lying where nobody perceives them");
    run(&mut w, "descend");
    assert!(apples(&w).is_none(), "gone as the player arrives");
}

#[test]
fn creatures_lose_their_food_at_its_moment_and_only_while_they_live_through_time() {
    let content = content();
    let mut w = larder(&content);
    run(&mut w, "move:E*6 descend");
    let kobold = (w.state.actors.values())
        .find(|a| a.level == "cellar" && a.id != w.state.player)
        .map(|a| a.id)
        .expect("the kobold");
    let carries_bread = |w: &World| find(w, &w.state.actors[&kobold].inventory, "bread").is_some();
    assert!(carries_bread(&w));
    // Away on another level, the kobold is frozen: its bread rots away only once it lives
    // through time again, in the turn the player arrives.
    run(&mut w, "ascend rest:24h*11");
    assert!(carries_bread(&w), "frozen on the cellar");
    run(&mut w, "descend");
    assert!(!carries_bread(&w));
    assert!(
        !texts(&w).iter().any(|t| t.contains("rotted away")),
        "nobody is told of a creature's food"
    );
    // Beside the player, it goes at its moment.
    let mut w = larder(&content);
    run(&mut w, "move:E*6");
    let made = w.time();
    run(&mut w, "descend");
    run(&mut w, &format!("rest:24h*9 rest:{}h", LASTS - 9 * 24 - 1));
    assert!(carries_bread(&w));
    let until = (made + hours(LASTS)).since(w.time());
    run(&mut w, &format!("rest:{}s", until.as_seconds()));
    assert_eq!(w.time(), made + hours(LASTS));
    assert!(!carries_bread(&w));
}

#[test]
fn rotten_food_of_any_kind_does_what_rotten_food_does_and_old_food_is_fine() {
    // Old bread (three days of five) is eaten as fresh.
    let content = content_with(&[("[rotten]\nvomit = 50", "[rotten]\nvomit = 0")]);
    let mut w = larder(&content);
    run(&mut w, &format!("skip:{}h", KEEPS - 48));
    w.apply(&Action::Eat(Meal::Floor(0)));
    assert_eq!(said(&w, "You eat the loaf of bread."), 1);
    assert!(!texts(&w).iter().any(|t| t.contains("rotten")));
    // Rotten bread: "Ugh!", sick, and it feeds.
    run(&mut w, "skip:48h");
    let before = w.state.player().satiation.unwrap().nutrition;
    w.apply(&Action::Eat(Meal::Floor(0)));
    assert_eq!(said(&w, "Ugh! The loaf of bread was rotten."), 1);
    assert_eq!(said(&w, "You feel sick."), 1);
    assert!(w.state.player().satiation.unwrap().nutrition > before + 700);
    // Or it comes back up.
    let content = content_with(&[("[rotten]\nvomit = 50", "[rotten]\nvomit = 100")]);
    let mut w = larder(&content);
    run(&mut w, &format!("skip:{KEEPS}h"));
    w.apply(&Action::Eat(Meal::Floor(0)));
    assert_eq!(said(&w, "Ugh! The loaf of bread was rotten."), 1);
    assert_eq!(said(&w, "You vomit!"), 1);
    // Kept down, rotten bread exposes the eater to grave rot as a rotten corpse does.
    let content = content_with(&[("[rotten]\nvomit = 50", "[rotten]\nvomit = 0")]);
    let rot = content.disease_id("grave_rot").expect("grave rot");
    let caught = (0..20u64).filter(|&seed| {
        let mut w = World::new(content.clone(), "larder", Some(seed)).expect("world");
        run(&mut w, &format!("skip:{KEEPS}h"));
        w.apply(&Action::Eat(Meal::Floor(0)));
        w.state.player().diseases.has(rot)
    });
    let caught = caught.count();
    assert!((1..20).contains(&caught), "caught {caught} times in 20");
}

#[test]
fn the_dev_dump_shows_the_stage_that_food_never_shows_the_player() {
    let (mut w, _) = carrying();
    run(&mut w, &format!("rest:24h*{}", KEEPS / 24));
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Actors);
    assert!(
        rows.iter()
            .any(|r| r.starts_with("food bread*2 carried by 0: made ") && r.ends_with(", rotten")),
        "{rows:#?}"
    );
    assert!(
        rows.iter()
            .any(|r| r.starts_with("food ration carried by 0") && r.ends_with("fresh"))
    );
    // The player sees the same names whatever the age.
    let view = w.perception();
    let names: Vec<String> = (view.inventory().iter())
        .map(|s| s.describe(w.content()))
        .collect();
    assert_eq!(names, ["2 loaves of bread", "a food ration"]);
}

#[test]
fn food_ages_survive_a_save_and_replay() {
    let content = content();
    let mut w = larder(&content);
    let mut replay = Replay::new(&w);
    for action in parse_actions("pickup rest:24h*10 rest:1h wait").expect("parse") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(said(&w, "Your 2 loaves of bread have rotted away."), 1);
    let (played, divergence) = (Replay::from_json(&replay.to_json()))
        .expect("replay parses")
        .play(content.clone())
        .expect("plays");
    assert_eq!(divergence, None);
    assert_eq!(played.state_hash(), w.state_hash());
    // Saved and loaded, the ages are as they were, and the food goes when it would have.
    let (mut saved, left) = carrying();
    run(&mut saved, "rest:24h*5");
    let mut loaded =
        crate::save::decode(&crate::save::encode(&saved), content.clone()).expect("loads");
    assert_eq!(loaded.state_hash(), saved.state_hash());
    let rest = format!("rest:24h*4 rest:{}s wait", left - 9 * 24 * 3600);
    run(&mut saved, &rest);
    run(&mut loaded, &rest);
    assert_eq!(loaded.state_hash(), saved.state_hash());
    assert!(find(&loaded, &loaded.state.player().inventory, "bread").is_none());
    // Food that does not know its age is no valid save.
    let mut broken = larder(&content);
    let bread = object(&broken, "bread");
    broken.player_mut().inventory.push(Stack::one(bread));
    assert!(crate::save::decode(&crate::save::encode(&broken), content.clone()).is_err());
}

/// From the courtyard's start: east along the path, the purse in the gazebo (a), the scroll of
/// light beside it (b), then along the flagstones to Grimbold's General Store, and in (as
/// `shop_tests.rs` walks it).
const INTO_SHOP: &str = "move:E*36 move:S pickup move:NE move:NE pickup:b move:SE move:E*5 \
                         move:S*5 move:E*5";

#[test]
fn rotten_food_is_worth_nothing_in_a_shop() {
    let mut w = World::new(
        Arc::new(Content::builtin().expect("content")),
        "courtyard",
        None,
    )
    .expect("courtyard");
    run(&mut w, INTO_SHOP);
    let meat_at = Pos::new(62, 15);
    let meat = find(&w, w.state.current().objects_at(meat_at), "dried_meat")
        .cloned()
        .unwrap();
    let asked = |w: &World, s: &Stack| w.perception().shop_here().and_then(|v| v.price(s));
    assert!(asked(&w, &meat).is_some(), "fresh, it is for sale");
    // Two months and a day old, it is rotten: not for sale (free to take), and not bought.
    let mut old = meat.clone();
    old.died = Some(GameTime::from_seconds(w.time().seconds() - 61 * 24 * 3600));
    w.state.current_mut().set_pile(meat_at, vec![old.clone()]);
    assert_eq!(asked(&w, &old), None);
    run(&mut w, "move:E*7 move:SE pickup");
    assert!(find(&w, &w.state.player().inventory, "dried_meat").is_some());
    assert!(
        w.state.current().shops[0].unpaid.is_empty(),
        "nothing owed for it"
    );
    assert_eq!(w.perception().sale(&Positions::of([2])), None);
    run(&mut w, "sell:c");
    assert_eq!(
        said(
            &w,
            "Grimbold sniffs at 6 strips of dried meat and says: \"Not at any price.\""
        ),
        1,
        "{:#?}",
        texts(&w)
    );
    assert!(
        find(&w, &w.state.player().inventory, "dried_meat").is_some(),
        "still carried"
    );
    // Fresh, the same is bought.
    let fresh = w
        .state
        .player()
        .inventory
        .iter()
        .position(|s| s.object == meat.object);
    w.player_mut().inventory[fresh.unwrap()].died = Some(w.time());
    assert!(w.perception().sale(&Positions::of([2])).is_some());
}
