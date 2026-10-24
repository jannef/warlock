//! Hunger in the world (spec §8 "Hunger"): game time burns nutrition (resting and overworld
//! journeys too) by the race, exertion burns more, each state does what it does (Weak, Fainting,
//! Starving, Bloated), meals take their turns and stop like a rest, food feeds and overeating
//! has its price, creatures leave corpses that rot away, corpses do what their creature's
//! `on_eat` says, and all of it saves and replays.

use super::*;
use crate::action::{Meal, parse_actions};
use crate::character::CharacterSpec;
use crate::content::ContentFiles;
use crate::hunger::Hunger;
use crate::replay::Replay;
use crate::rot::Stage as Age;
use crate::stats::Stat;

/// The built-in hunger file, to edit for a test.
const HUNGER: &str = include_str!("../../../../assets/data/hunger.toml");

/// Food underfoot, nothing else about.
const PANTRY: &str = r##"
name = "Pantry"
ambient = [200, 200, 200, 0]
map = '''
#######
#@....#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player", objects = ["ration*2", "apple", "dagger"] }
"##;

/// Food underfoot, and a cave beetle that comes into view up the passage within a few turns.
const LARDER: &str = r##"
name = "Larder"
ambient = [200, 200, 200, 0]
map = '''
#########
#@......#
#####.###
#####.###
#####1###
#########
'''
[legend]
"@" = { terrain = "floor", spawn = "player", objects = ["ration*2"] }
"1" = { terrain = "floor", creature = "beetle", route = "N3 S3" }
"##;

/// A giant rat beside the player.
const PIT: &str = r##"
name = "Pit"
ambient = [200, 200, 200, 0]
map = '''
######
#@r..#
######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "rat" }
"##;

/// The built-in content with the test levels, and `data/hunger.toml` edited by `edits` (each
/// `from` replaced by `to`).
fn content_with(edits: &[(&str, &str)]) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    for (name, level) in [("pantry", PANTRY), ("larder", LARDER), ("pit", PIT)] {
        files.insert(&format!("levels/{name}.toml"), level);
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!("name = \"{name}\"\nstart_level = \"{name}\"\nhp = 1000\n"),
        );
    }
    let mut hunger = HUNGER.to_string();
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

fn world(scenario: &str) -> World {
    World::new(content(), scenario, None).expect("world")
}

fn world_in(content: &Arc<Content>, scenario: &str) -> World {
    World::new(content.clone(), scenario, None).expect("world")
}

fn as_character(scenario: &str, spec: &str) -> World {
    let spec: CharacterSpec = spec.parse().expect("a character");
    World::with_character(content(), scenario, None, Some(&spec)).expect("world")
}

fn run(w: &mut World, actions: &str) -> Vec<Outcome> {
    parse_actions(actions)
        .expect("actions")
        .iter()
        .map(|a| w.apply(a))
        .collect()
}

fn texts(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.text.clone()).collect()
}

fn said(w: &World, text: &str) -> bool {
    texts(w).iter().any(|t| t == text)
}

fn nutrition(w: &World) -> i32 {
    w.state
        .player()
        .satiation
        .expect("the player hungers")
        .nutrition
}

fn set_nutrition(w: &mut World, n: i32) {
    let player = w.state.player;
    let fed = w.state.actors.get_mut(&player).unwrap();
    fed.satiation = Some(crate::hunger::Satiation::new(n));
}

fn hunger(w: &World) -> Hunger {
    w.perception().own_hunger().expect("the player hungers")
}

/// Whether the player is sick from what they ate: poisoned (`sickness`).
fn sick(w: &World) -> bool {
    let poisoned = w.content().status_id("poisoned").expect("poison");
    w.state.player().effects.get(poisoned).is_some()
}

fn object(w: &World, id: &str) -> ObjectId {
    w.content().object_id(id).unwrap_or_else(|| panic!("{id}"))
}

fn count_of(stacks: &[Stack], object: ObjectId) -> u32 {
    stacks
        .iter()
        .filter(|s| s.object == object)
        .map(|s| s.count)
        .sum()
}

fn underfoot(w: &World) -> Vec<Stack> {
    w.state.current().objects_at(w.player_pos()).to_vec()
}

use crate::content::ObjectId;

#[test]
fn a_new_run_starts_not_hungry_and_the_clock_burns_nutrition() {
    let mut w = world("pantry");
    assert_eq!(nutrition(&w), 3600);
    assert_eq!(hunger(&w), Hunger::NotHungry);
    // A point a minute for a human: an hour's rest burns 60, and so do 600 waits.
    run(&mut w, "rest:1h");
    assert_eq!(nutrition(&w), 3540);
    run(&mut w, "wait*600");
    assert_eq!(nutrition(&w), 3480);
    // The dev time skip passes no turn: nothing is burned, like nothing heals.
    run(&mut w, "skip:5h");
    assert_eq!(nutrition(&w), 3480);
    // Nothing of the counter is perceived but the state.
    let json = crate::dump::json(&w, &[], false);
    assert_eq!(json["hunger"], "not_hungry");
    assert!(!json.to_string().contains("3480"));
}

#[test]
fn overworld_journeys_burn_by_the_hour() {
    // The world scenario: on the road beside the Hall; a step there takes a quarter of an hour
    // or more (the ranger's survival skill shortens journeys).
    let mut w = world("world");
    let mut seconds = 0;
    for dir in [Dir::E, Dir::W, Dir::S] {
        let out = w.apply(&Action::Move(dir));
        assert!(out.elapsed >= Duration::minutes(15), "{:?}", out.elapsed);
        seconds += out.elapsed.as_seconds();
        // A point a minute, what is left of a minute banked.
        assert_eq!(nutrition(&w), 3600 - (seconds / 60) as i32);
    }
}

#[test]
fn races_need_more_or_less() {
    for (spec, burned) in [
        ("human,female,thief", 240),
        ("halfling,male,thief", 300),
        ("elf,female,wizard", 192),
        ("half_orc,male,warrior", 288),
        ("dwarf,female,warrior", 264),
    ] {
        let mut w = as_character("pantry", spec);
        run(&mut w, "rest:4h");
        assert_eq!(3600 - nutrition(&w), burned, "{spec}");
    }
}

#[test]
fn fighting_burns_extra_by_the_race() {
    let mut w = world("pit");
    let rat = w.state.actor_at("pit", Pos::new(2, 1)).unwrap().id;
    // The rat stands still: this is about the player's blows.
    w.state.actors.get_mut(&rat).unwrap().energy = -1_000_000_000;
    w.state.actors.get_mut(&rat).unwrap().hp = 1000;
    let out = w.apply(&Action::Move(Dir::E));
    assert!(out.took_turn);
    // 6 s of time (nothing yet) and 2 points of exertion.
    assert_eq!(nutrition(&w), 3598);
}

#[test]
fn hungry_is_told_and_stops_a_rest() {
    let mut w = world("pantry");
    set_nutrition(&mut w, 1450);
    let out = w.apply(&Action::Rest(Rest::Dawn));
    assert_eq!(hunger(&w), Hunger::Hungry);
    assert!(said(&w, "You are getting hungry."));
    assert!(
        texts(&w)
            .last()
            .unwrap()
            .starts_with("You stop resting after")
    );
    assert!(out.elapsed <= Duration::minutes(11), "{:?}", out.elapsed);
    // The status bar's state: shown in the dump header as it is.
    let header = crate::dump::ascii(&w, &[]);
    assert!(header.contains(" hunger hungry "), "{header}");
}

#[test]
fn weak_saps_strength_and_dexterity_where_the_rules_use_them() {
    let mut w = world("pantry");
    let fed = w.perception().own_scores();
    let fed_limit = w.perception().own_load().limit;
    let fed_defence = w.perception().own_defence();
    set_nutrition(&mut w, 500);
    assert_eq!(hunger(&w), Hunger::Weak);
    let weak = w.perception().own_scores();
    assert_eq!(weak[Stat::Strength], fed[Stat::Strength] - 1);
    assert_eq!(weak[Stat::Dexterity], fed[Stat::Dexterity] - 1);
    assert_eq!(weak[Stat::Toughness], fed[Stat::Toughness]);
    // A point of Strength less is 3 kg less to carry (1.5 kg × 2).
    assert_eq!(fed_limit - w.perception().own_load().limit, 3000);
    // Dexterity 10 → 9: its modifier drops by 1, and so does Defence.
    assert_eq!(fed_defence - w.perception().own_defence(), 1);
    // The stats themselves are untouched: being fed again restores them.
    set_nutrition(&mut w, 2000);
    assert_eq!(w.perception().own_scores(), fed);
}

#[test]
fn fainting_costs_turns_and_is_deterministic() {
    let play = || {
        let content = content_with(&[("chance = 5", "chance = 50")]);
        let mut w = world_in(&content, "pantry");
        set_nutrition(&mut w, 100);
        assert_eq!(hunger(&w), Hunger::Fainting);
        let outs = run(&mut w, "wait*20");
        (w, outs)
    };
    let (a, outs) = play();
    assert!(said(&a, "You faint from hunger."));
    // A faint costs 5 more turns on top of the wait: 36 s; about half the turns faint.
    let fainted = (outs.iter())
        .filter(|o| o.elapsed == Duration::seconds(36))
        .count();
    assert!((3..=17).contains(&fainted), "{fainted}");
    let short = outs.iter().filter(|o| o.elapsed == Duration::seconds(6));
    assert_eq!(short.count(), 20 - fainted);
    let (b, _) = play();
    assert_eq!(a.state_hash(), b.state_hash(), "the same draws");
    // Not hungry enough, nobody faints, and nothing is drawn.
    let mut fed = world_in(&content_with(&[("chance = 5", "chance = 100")]), "pantry");
    let rng = fed.state.rng.clone();
    run(&mut fed, "wait*5");
    assert_eq!(fed.state.rng, rng);
}

#[test]
fn starving_takes_hit_points_until_the_player_starves_to_death() {
    let content = content_with(&[("chance = 5", "chance = 0")]);
    let mut w = world_in(&content, "pantry");
    set_nutrition(&mut w, -1);
    assert_eq!(hunger(&w), Hunger::Starving);
    let player = w.state.player;
    w.state.actors.get_mut(&player).unwrap().hp = 30;
    // Hurt as they are, no hit point comes back while starving: one goes every 10 minutes.
    run(&mut w, "rest:1h");
    assert_eq!(w.state.player().hp, 24);
    w.state.actors.get_mut(&player).unwrap().hp = 1;
    run(&mut w, "rest:1h");
    let death = w.death().expect("starved");
    assert_eq!(death.cause, Some(Cause::Starvation));
    assert_eq!(death.by, None);
    assert_eq!(texts(&w).last().unwrap(), "You starve to death...");
    assert_eq!(
        Cause::Starvation.epitaph(w.content()),
        "Starved to death".to_string()
    );
    assert!(!w.apply(&Action::Wait).took_turn, "the run is over");
    // Fed, hit points come back again.
    let mut fed = world_in(&content, "pantry");
    fed.state.actors.get_mut(&player).unwrap().hp = 30;
    run(&mut fed, "rest:1h");
    assert!(fed.state.player().hp > 30);
}

#[test]
fn hunger_does_not_depend_on_how_time_is_cut() {
    // Hurt, a few minutes short of starving: hours later it makes no difference whether the time
    // passed at once or second by second. Healing stops exactly as starving begins, and the
    // player starves to death at the same moment.
    let content = content_with(&[("chance = 5", "chance = 0")]);
    let hurt = || {
        let mut w = world_in(&content, "pantry");
        set_nutrition(&mut w, 3);
        let player = w.state.player;
        w.state.actors.get_mut(&player).unwrap().hp = 6;
        w
    };
    let mut events = Vec::new();
    let (mut at_once, mut cut) = (hurt(), hurt());
    let (player, start) = (at_once.state.player, at_once.state.time);
    let hours = 2 * 3600;
    at_once.pass_time(std::iter::once(player), start, hours, &mut events);
    for s in 0..hours {
        cut.pass_time(
            std::iter::once(player),
            start + Duration::seconds(s),
            1,
            &mut events,
        );
    }
    for w in [&at_once, &cut] {
        assert_eq!(w.state.player().satiation, at_once.state.player().satiation);
        assert_eq!(w.state.player().hp, at_once.state.player().hp);
        assert_eq!(w.state.player().regen, at_once.state.player().regen);
    }
    let death = at_once.death().expect("starved");
    assert_eq!(death.cause, Some(Cause::Starvation));
    assert_eq!(cut.death().map(|d| d.time), Some(death.time));
    assert!(
        death.time > start + Duration::seconds(3600),
        "after an hour of starving"
    );
}

#[test]
fn bloated_slows_every_action_like_a_burden() {
    let mut w = world("pantry");
    set_nutrition(&mut w, 6000);
    assert_eq!(hunger(&w), Hunger::Bloated);
    // ×1.25: four waits take 30 s instead of 24.
    let outs = run(&mut w, "wait*4");
    let total: u64 = outs.iter().map(|o| o.elapsed.as_seconds()).sum();
    assert_eq!(total, 30);
    set_nutrition(&mut w, 5000);
    let outs = run(&mut w, "wait*4");
    let total: u64 = outs.iter().map(|o| o.elapsed.as_seconds()).sum();
    assert_eq!(total, 24, "satiated is no slower");
}

#[test]
fn a_meal_takes_its_turns_then_feeds() {
    let mut w = world("pantry");
    let ration = object(&w, "ration");
    // `eat:floor:a`: the rations on top of the pile, four turns.
    let out = w.apply(&Action::Eat(Meal::Floor(0)));
    assert_eq!(out.elapsed, Duration::seconds(24));
    assert_eq!(w.turn(), 4);
    assert_eq!(count_of(&underfoot(&w), ration), 1);
    assert_eq!(nutrition(&w), 3600 + 1500);
    let shown = texts(&w);
    assert_eq!(
        shown[shown.len() - 2..],
        ["You eat the food ration.", "You feel satiated."]
    );
    assert_eq!(hunger(&w), Hunger::Satiated);
    // From the pack: one turn for an apple, and the stack goes when it is eaten.
    run(&mut w, "pickup:b");
    let apple = object(&w, "apple");
    let out = w.apply(&Action::Eat(Meal::Pack(0)));
    assert_eq!(out.elapsed, Duration::seconds(6));
    assert_eq!(count_of(&w.state.player().inventory, apple), 0);
    assert!(said(&w, "You eat the apple."));
    // What is no food is refused, and takes no time.
    let dagger = underfoot(&w)
        .iter()
        .position(|s| s.object == object(&w, "dagger"));
    let out = w.apply(&Action::Eat(Meal::Floor(dagger.unwrap())));
    assert!(!out.took_turn);
    assert_eq!(texts(&w).last().unwrap(), "You cannot eat a dagger.");
    let out = w.apply(&Action::Eat(Meal::Pack(5)));
    assert!(!out.took_turn);
    assert_eq!(texts(&w).last().unwrap(), "You carry no such thing.");
}

#[test]
fn a_creature_coming_into_view_stops_a_meal_and_leaves_the_food() {
    let mut w = world("larder");
    let ration = object(&w, "ration");
    let before = nutrition(&w);
    let out = w.apply(&Action::Eat(Meal::Floor(0)));
    assert!(out.took_turn);
    assert!(out.elapsed < Duration::seconds(24), "{:?}", out.elapsed);
    assert_eq!(
        texts(&w).last().unwrap(),
        "A cave beetle comes into view; you stop eating."
    );
    assert_eq!(count_of(&underfoot(&w), ration), 2, "nothing was eaten");
    assert!(nutrition(&w) <= before);
}

#[test]
fn overeating_may_choke_vomit_or_be_lost() {
    // Vomiting: it feeds nothing, and leaves the player hungry.
    let content = content_with(&[("vomit = 50", "vomit = 100")]);
    let mut w = world_in(&content, "pantry");
    set_nutrition(&mut w, 6000);
    w.apply(&Action::Eat(Meal::Floor(0)));
    assert!(said(&w, "You vomit!"));
    assert_eq!(nutrition(&w), 1000);
    assert_eq!(hunger(&w), Hunger::Hungry);
    // Neither: what does not fit is lost.
    let content = content_with(&[("vomit = 50", "vomit = 0")]);
    let mut w = world_in(&content, "pantry");
    set_nutrition(&mut w, 6000);
    w.apply(&Action::Eat(Meal::Floor(0)));
    assert!(said(&w, "You have a hard time getting it all down."));
    assert_eq!(nutrition(&w), 6500);
    // Choking to death, when it is on.
    let content = content_with(&[("choke = 0", "choke = 100")]);
    let mut w = world_in(&content, "pantry");
    set_nutrition(&mut w, 6000);
    w.apply(&Action::Eat(Meal::Floor(0)));
    let ration = object(&w, "ration");
    assert_eq!(w.death().and_then(|d| d.cause), Some(Cause::Choked(ration)));
    assert_eq!(
        texts(&w).last().unwrap(),
        "You choke to death on the food ration..."
    );
    // Off by default: a meal that fits is just eaten.
    let mut w = world("pantry");
    set_nutrition(&mut w, 4000);
    w.apply(&Action::Eat(Meal::Floor(0)));
    assert!(w.death().is_none());
    assert_eq!(nutrition(&w), 5500);
}

/// The pit after the player killed its rat: the rat's corpse lies where it fell.
fn killed_the_rat() -> World {
    let mut w = world("pit");
    let rat = w.state.actor_at("pit", Pos::new(2, 1)).unwrap().id;
    let r = w.state.actors.get_mut(&rat).unwrap();
    r.energy = -1_000_000_000;
    r.hp = 1;
    for _ in 0..50 {
        if !w.state.actors.contains_key(&rat) {
            return w;
        }
        w.apply(&Action::Move(Dir::E));
    }
    panic!("the rat would not die");
}

#[test]
fn creatures_leave_corpses_that_weigh_what_they_did() {
    let w = killed_the_rat();
    let corpse = object(&w, "rat_corpse");
    let def = w.content().object(corpse);
    assert_eq!(def.name, "giant rat corpse");
    assert_eq!((def.glyph, def.weight), ('%', 4000));
    assert_eq!(
        def.fg,
        w.content()
            .creature(w.content().creature_id("rat").unwrap())
            .fg
    );
    // Small: 400 nutrition, three turns.
    assert_eq!(def.food.map(|f| (f.nutrition, f.turns)), Some((400, 3)));
    let pile = w.state.current().objects_at(Pos::new(2, 1));
    assert_eq!(pile[0].object, corpse, "on top");
    assert!(pile[0].died.is_some_and(|d| d <= w.time()));
    // The player never eats their own: there is no player corpse.
    assert!(w.content().object_id("player_corpse").is_none());
    // Level files cannot place corpses.
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/morgue.toml",
        "name = \"Morgue\"\nmap = '''\n###\n#@#\n###\n'''\n[legend]\n\
         \"@\" = { terrain = \"floor\", spawn = \"player\", objects = [\"rat_corpse\"] }\n",
    );
    let err = Content::load(&files).unwrap_err();
    assert!(
        err.message
            .contains("corpses lie only where creatures died")
    );
}

#[test]
fn corpses_rot_and_then_are_gone_from_the_floor_and_the_pack() {
    let mut w = killed_the_rat();
    let corpse = object(&w, "rat_corpse");
    let at = Pos::new(2, 1);
    let lying = |w: &World| w.state.current().objects_at(at).to_vec();
    assert_eq!(w.stage(&lying(&w)[0]), Some(Age::Fresh));
    run(&mut w, "skip:3h");
    assert_eq!(w.stage(&lying(&w)[0]), Some(Age::Fresh));
    // Old past half of the 8 hours it keeps (fine to eat), then rotten.
    run(&mut w, "skip:1h");
    assert_eq!(w.stage(&lying(&w)[0]), Some(Age::Old));
    run(&mut w, "skip:4h");
    assert_eq!(w.stage(&lying(&w)[0]), Some(Age::Rotten));
    // A day and a half after the death it is gone.
    run(&mut w, "skip:28h");
    assert!(lying(&w).iter().all(|s| s.object != corpse));
    // Carried, it rots away from the pack, said without stopping a rest.
    let mut w = killed_the_rat();
    run(&mut w, "move:E pickup");
    assert_eq!(count_of(&w.state.player().inventory, corpse), 1);
    assert_eq!(w.perception().own_weight().carried, 4000);
    run(&mut w, "skip:36h wait");
    assert_eq!(count_of(&w.state.player().inventory, corpse), 0);
    assert!(said(&w, "Your giant rat corpse has rotted away."));
}

#[test]
fn a_fresh_corpse_feeds_by_size_and_does_what_its_creature_says() {
    let mut w = killed_the_rat();
    run(&mut w, "move:E");
    let out = w.apply(&Action::Eat(Meal::Floor(0)));
    assert_eq!(out.elapsed, Duration::seconds(18), "three turns");
    assert!(said(&w, "You eat the giant rat corpse."));
    assert!(said(&w, "Gamey and stringy, but it stays down."));
    // 400 for a small corpse, less the minutes of the fight and the meal.
    assert!((3990..=4000).contains(&nutrition(&w)), "{}", nutrition(&w));
}

#[test]
fn a_rotten_corpse_makes_you_vomit_or_sick() {
    for (vomit, vomited) in [
        ("[rotten]\nvomit = 100", true),
        ("[rotten]\nvomit = 0", false),
    ] {
        let content = content_with(&[("[rotten]\nvomit = 50", vomit)]);
        let mut w = world_in(&content, "pantry");
        let corpse = object(&w, "kobold_corpse");
        let died = GameTime::from_seconds(w.time().seconds() - 10 * 3600);
        let here = w.player_pos();
        w.state
            .current_mut()
            .set_pile(here, vec![Stack::corpse(corpse, died)]);
        let hp = w.state.player().hp;
        w.apply(&Action::Eat(Meal::Floor(0)));
        assert!(said(&w, "Ugh! The kobold corpse was rotten."));
        // A rotten corpse has lost what its creature does when eaten.
        assert!(!said(&w, "This kobold meat tastes terrible!"));
        assert_eq!(said(&w, "You vomit!"), vomited);
        if vomited {
            assert_eq!(nutrition(&w), 1000);
            assert_eq!(w.state.player().hp, hp);
            assert!(!sick(&w));
        } else {
            // Food poisoning: a status (spec §8 "Status effects") that costs hit points as time
            // passes.
            assert!(said(&w, "You feel sick."));
            assert!(sick(&w));
            assert!(nutrition(&w) > 3900, "and it feeds");
            run(&mut w, "wait*25");
            assert!(w.state.player().hp < hp);
        }
    }
}

#[test]
fn a_rotten_corpse_kept_down_may_give_grave_rot() {
    // Rotten food carries grave rot (spec §8 "Diseases"): kept down, the eater makes a check to
    // resist it; vomited, it never comes to that.
    for (vomit, vomited) in [
        ("[rotten]\nvomit = 100", true),
        ("[rotten]\nvomit = 0", false),
    ] {
        let content = content_with(&[("[rotten]\nvomit = 50", vomit)]);
        let rot = content.disease_id("grave_rot").expect("grave rot");
        let caught = (0..20u64).filter(|&seed| {
            let mut w = World::new(content.clone(), "pantry", Some(seed)).expect("world");
            let corpse = object(&w, "kobold_corpse");
            let died = GameTime::from_seconds(w.time().seconds() - 10 * 3600);
            let here = w.player_pos();
            w.state
                .current_mut()
                .set_pile(here, vec![Stack::corpse(corpse, died)]);
            w.apply(&Action::Eat(Meal::Floor(0)));
            assert!(said(&w, "Ugh! The kobold corpse was rotten."));
            w.state.player().diseases.has(rot)
        });
        let caught = caught.count();
        if vomited {
            assert_eq!(caught, 0, "vomited: no exposure");
        } else {
            // Toughness 10 against 15: resisted about two times in five.
            assert!((1..20).contains(&caught), "caught {caught} times in 20");
        }
    }
}

#[test]
fn harm_from_food_is_a_known_sickness() {
    let sickness = "sickness = \"poisoned\"\n";
    for (to, refused) in [
        ("sickness = \"gout\"\n", "unknown sickness `gout`"),
        ("", "need a `sickness`"),
    ] {
        let mut files = ContentFiles::builtin();
        assert!(HUNGER.contains(sickness));
        files.insert("data/hunger.toml", &HUNGER.replace(sickness, to));
        let e = Content::load(&files).expect_err("refused").to_string();
        assert!(e.contains(refused), "{e}");
    }
}

/// The pantry with a fresh corpse of `creature` underfoot, as `spec` (or the scenario's player).
fn with_corpse(content: &Arc<Content>, creature: &str, spec: Option<&str>) -> World {
    let spec: Option<CharacterSpec> = spec.map(|s| s.parse().expect("a character"));
    let mut w =
        World::with_character(content.clone(), "pantry", None, spec.as_ref()).expect("world");
    let corpse = object(&w, &format!("{creature}_corpse"));
    let (here, now) = (w.player_pos(), w.time());
    w.state
        .current_mut()
        .set_pile(here, vec![Stack::corpse(corpse, now)]);
    w
}

#[test]
fn eating_a_corpse_may_raise_a_stat_up_to_its_potential() {
    let mut files = ContentFiles::builtin();
    files.insert("levels/pantry.toml", PANTRY);
    files.insert(
        "scenarios/pantry.toml",
        "name = \"pantry\"\nstart_level = \"pantry\"\nhp = 1000\n",
    );
    // A sure thing for the test: the beetle always toughens.
    files.edit_table("creatures", |t| {
        t.replace("stat = { toughness = 10 }", "stat = { toughness = 100 }")
    });
    let content = Arc::new(Content::load(&files).expect("valid"));
    // A human's potentials are above the starting scores.
    let mut w = with_corpse(&content, "beetle", Some("human,female,thief"));
    let (score, potential) = {
        let s = w.state.player().stats;
        (s.score(Stat::Toughness), s.potential(Stat::Toughness))
    };
    assert!(score < potential);
    let max_hp = w.state.player().max_hp(w.content());
    w.apply(&Action::Eat(Meal::Floor(0)));
    assert!(said(&w, "Crunchy."));
    assert!(said(&w, "You feel tougher!"));
    assert_eq!(w.state.player().stats.score(Stat::Toughness), score + 1);
    assert_eq!(w.state.player().max_hp(w.content()), max_hp + 1);
    // At its potential, nothing rises (and nothing says so).
    let mut capped = with_corpse(&content, "beetle", None);
    let stats = capped.state.player().stats;
    assert_eq!(
        stats.score(Stat::Toughness),
        stats.potential(Stat::Toughness)
    );
    capped.apply(&Action::Eat(Meal::Floor(0)));
    assert!(said(&capped, "You eat the cave beetle corpse."));
    assert!(!said(&capped, "You feel tougher!"));
    assert_eq!(capped.state.player().stats, stats);
}

#[test]
fn corpses_heal_harm_feed_more_or_come_back_up() {
    let content = content();
    // The deer: very nourishing, and it heals.
    let mut w = with_corpse(&content, "deer", None);
    let player = w.state.player;
    w.state.actors.get_mut(&player).unwrap().hp = 10;
    w.apply(&Action::Eat(Meal::Floor(0)));
    assert!(said(&w, "Venison! It does you good."));
    assert!(said(&w, "You feel better."));
    assert!(w.state.player().hp > 10);
    // 1000 for a medium corpse and 1000 more: bloated, past the satiated.
    assert!(nutrition(&w) >= 5590, "{}", nutrition(&w));
    assert_eq!(hunger(&w), Hunger::Bloated);
    // The bat disagrees with the eater: it makes them sick, which costs hit points in time.
    let mut w = with_corpse(&content, "bat", None);
    let hp = w.state.player().hp;
    w.apply(&Action::Eat(Meal::Floor(0)));
    assert!(said(&w, "You feel sick."));
    assert!(sick(&w));
    run(&mut w, "wait*25");
    assert!(w.state.player().hp < hp);
    // The kobold may come back up: a sure thing here.
    let sure = {
        let mut files = ContentFiles::builtin();
        files.insert("levels/pantry.toml", PANTRY);
        files.insert(
            "scenarios/pantry.toml",
            "name = \"pantry\"\nstart_level = \"pantry\"\nhp = 1000\n",
        );
        files.edit_table("creatures", |t| {
            t.replace(
                "on_eat = { message = \"This kobold meat tastes terrible!\", vomit = 40 }",
                "on_eat = { message = \"This kobold meat tastes terrible!\", vomit = 100 }",
            )
        });
        Arc::new(Content::load(&files).expect("valid"))
    };
    let mut w = with_corpse(&sure, "kobold", None);
    w.apply(&Action::Eat(Meal::Floor(0)));
    assert!(said(&w, "This kobold meat tastes terrible!"));
    assert!(said(&w, "You vomit!"));
    assert_eq!(nutrition(&w), 1000);
}

#[test]
fn hunger_and_corpses_save_and_replay() {
    let content = content();
    // Fight the rat to the death (it bites back, but the player has hit points to spare), step
    // onto its corpse, eat it, and rest.
    let mut w = world_in(&content, "pit");
    let mut replay = Replay::new(&w);
    let actions = "move:E*20 move:W*2 eat:floor:a rest:1h wait";
    for action in parse_actions(actions).expect("parse") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(
        said(&w, "You eat the giant rat corpse."),
        "{:#?}",
        texts(&w)
    );
    let (played, divergence) = (Replay::from_json(&replay.to_json()))
        .expect("replay parses")
        .play(content.clone())
        .expect("plays");
    assert_eq!(divergence, None);
    assert_eq!(played.state_hash(), w.state_hash());
    // Saved and loaded, nutrition, what time banked, and corpses' deaths are as they were.
    let mut saved = killed_the_rat();
    run(&mut saved, "move:E pickup rest:1h");
    let mut loaded =
        crate::save::decode(&crate::save::encode(&saved), saved.content().clone()).expect("loads");
    assert_eq!(loaded.state_hash(), saved.state_hash());
    assert_eq!(
        loaded.state.player().satiation,
        saved.state.player().satiation
    );
    run(&mut loaded, "wait*3 eat:a");
    run(&mut saved, "wait*3 eat:a");
    assert_eq!(loaded.state_hash(), saved.state_hash());
    assert!(said(&saved, "You eat the giant rat corpse."));
}

#[test]
fn a_remedy_cures_every_disease_as_it_is_eaten_even_grave_rot() {
    let mut w = world("pantry");
    let root = object(&w, "angelica_root");
    let grave_rot = w.content().disease_id("grave_rot").expect("grave rot");
    w.player_mut().inventory.insert(0, Stack::new(root, 2));
    run(&mut w, "infect:grave_rot rest:24h");
    assert!(said(&w, "Your skin turns grey and clammy."), "it shows");
    run(&mut w, "eat:a");
    assert!(said(&w, "You eat the angelica root."));
    assert!(said(&w, "The rot is gone from your flesh."));
    assert!(w.state.player().diseases.get(grave_rot).is_none());
    let diseased = w.content().status_id("diseased").expect("diseased");
    assert!(!w.state.player().effects.has(diseased));
    assert_eq!(count_of(&w.state.player().inventory, root), 1);
    // Eaten while it incubates, unseen, it cures it as silently as it came.
    run(&mut w, "infect:grave_rot eat:a");
    assert!(w.state.player().diseases.get(grave_rot).is_none());
    assert_eq!(count_of(&w.state.player().inventory, root), 0);
}

#[test]
fn iron_stomach_keeps_rotten_food_and_too_much_down() {
    // Both chances to vomit certain: with Iron stomach (spec §8 "Talents") neither comes.
    let content = content_with(&[
        (
            "[overeating]\nchoke = 0\nvomit = 50",
            "[overeating]\nchoke = 0\nvomit = 100",
        ),
        ("[rotten]\nvomit = 50", "[rotten]\nvomit = 100"),
    ]);
    let mut w = world_in(&content, "pantry");
    w.player_mut().talents.take("iron_stomach", None);
    set_nutrition(&mut w, 6000);
    w.apply(&Action::Eat(Meal::Floor(0)));
    assert!(!said(&w, "You vomit!"));
    assert!(said(&w, "You have a hard time getting it all down."));
    let mut w = world_in(&content, "pantry");
    w.player_mut().talents.take("iron_stomach", None);
    let corpse = object(&w, "kobold_corpse");
    let died = GameTime::from_seconds(w.time().seconds() - 10 * 3600);
    let here = w.player_pos();
    w.state
        .current_mut()
        .set_pile(here, vec![Stack::corpse(corpse, died)]);
    w.apply(&Action::Eat(Meal::Floor(0)));
    assert!(said(&w, "Ugh! The kobold corpse was rotten."));
    assert!(!said(&w, "You vomit!"));
    assert!(nutrition(&w) > 3900, "kept down, it feeds");
}
