//! Temperature and ice through the whole turn pipeline (spec §5 "Temperature", "Ice"): dungeon
//! temperatures and the sky's, water freezing and thawing lazily with game time however it
//! passes, ice holding by weight (what is carried included), falling through and climbing out,
//! holes refreezing, objects sinking, creatures falling in, the instant frost and heat bursts,
//! blasts, saves, replays, and catching up after a long absence.

use super::*;
use crate::content::{ContentFiles, TerrainId};
use crate::ice::{self, Shock};
use crate::perception::CellView;
use crate::replay::Replay;
use crate::temperature::{self, Temperature};
use std::sync::LazyLock;

/// A lit pond below freezing: thin ice (t, 20 mm), ice (i, 50 mm), thick ice (T, 200 mm), open
/// water (~), a dagger on thin ice (d), and a puddle behind a wall (bottom right).
const POND: &str = r#"
name = "Pond"
temperature = -5
ambient = [255, 255, 255, 0]
map = '''
##########
#@.......#
#.~~~~~~.#
#.tiTt~d.#
#.~~~~~~.#
#......#~#
##########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"t" = { terrain = "water", ice = 20 }
"i" = { terrain = "water", ice = 50 }
"T" = { terrain = "water", ice = 200 }
"d" = { terrain = "water", ice = 20, objects = ["dagger"] }
"#;

/// The same pond above freezing.
const WARM: &str = r#"
name = "Warm Pond"
temperature = 5
ambient = [255, 255, 255, 0]
map = '''
#######
#@.TT~#
#..tt~#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"t" = { terrain = "water", ice = 20 }
"T" = { terrain = "water", ice = 200 }
"#;

/// A deer's round crosses thin ice.
const CROSSING: &str = r#"
name = "Crossing"
temperature = -5
ambient = [255, 255, 255, 0]
map = '''
#######
#@....#
#.....#
#d.tt.#
#.....#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"d" = { terrain = "floor", creature = "deer", route = "E4 W4" }
"t" = { terrain = "water", ice = 20 }
"#;

/// The player crosses ice that holds them to an ox, who makes way onto it.
const FERRY: &str = r#"
name = "Ferry"
temperature = -5
ambient = [255, 255, 255, 0]
map = '''
#######
#@io..#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"i" = { terrain = "water", ice = 50 }
"o" = { terrain = "floor", creature = "ox" }
"#;

/// A fireball trap beside thick ice, and further on a frost trap above open water.
const BURSTS: &str = r#"
name = "Bursts"
temperature = -5
ambient = [255, 255, 255, 0]
map = '''
############
#@f.....F..#
#.TT...~~~.#
############
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"f" = { terrain = "floor", trap = "fireball" }
"F" = { terrain = "floor", trap = "frost" }
"T" = { terrain = "water", ice = 200 }
"~" = "water"
"#;

/// A heavy beast that makes way for anyone.
const OX: &str = r##"
[[creature]]
id = "ox"
name = "ox"
glyph = "q"
fg = "#a08060"
weight = 400000
faction = "peaceful"
"##;

static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.insert("data/creatures/ox.toml", OX);
    // Strong (St 40, To 40: a limit of 180 kg), so heavy loads still move.
    for (name, level) in [
        ("pond", POND),
        ("warm", WARM),
        ("crossing", CROSSING),
        ("ferry", FERRY),
    ] {
        files.insert(&format!("levels/{name}.toml"), level);
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!(
                "name = \"{name}\"\nstart_level = \"{name}\"\nhp = 20\n\
                 stats = {{ strength = 40, toughness = 40 }}\n"
            ),
        );
    }
    // Tough enough for a fireball and a burst of frost.
    files.insert("levels/bursts.toml", BURSTS);
    files.insert(
        "scenarios/bursts.toml",
        "name = \"bursts\"\nstart_level = \"bursts\"\nhp = 100\n",
    );
    // The courtyard in deep winter and in high summer.
    for (name, month) in [("winter_yard", 12), ("summer_yard", 6)] {
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!(
                "name = \"{name}\"\nstart_level = \"courtyard\"\nhp = 20\n\
                 start_date = [1444, {month}, 20]\nstart_time = \"10:00\"\n"
            ),
        );
    }
    Arc::new(Content::load(&files).expect("test content is valid"))
});

fn world(scenario: &str) -> World {
    World::new(CONTENT.clone(), scenario, None).expect("scenario starts")
}

fn run(world: &mut World, text: &str) -> Vec<Outcome> {
    let actions = crate::action::parse_actions(text).expect("actions");
    actions.iter().map(|a| world.apply(a)).collect()
}

fn texts(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.text.clone()).collect()
}

fn said(world: &World, text: &str) -> bool {
    texts(world).iter().any(|t| t == text)
}

fn terrain_id(world: &World, p: Pos) -> String {
    let t = world.state.current().terrain[p];
    world.content().terrain(t).id.clone()
}

fn mm(world: &World, p: Pos) -> u32 {
    world.state.current().ice_at(p).map_or(0, |i| i.mm())
}

/// Actor `id` carries `n` rocks (800 g each) and nothing else.
fn carry_rocks(world: &mut World, id: ActorId, n: u32) {
    let rock = world.content().object_id("rock").expect("rock");
    let actor = world.state.actors.get_mut(&id).expect("actor");
    actor.inventory = vec![Stack::new(rock, n)];
}

const PLUNGE: &str = "The ice cracks beneath you and you plunge into the freezing water!";
const THIN: Pos = Pos::new(2, 3);
const ICE: Pos = Pos::new(3, 3);
const THICK: Pos = Pos::new(4, 3);

#[test]
fn dungeons_keep_their_temperature_and_the_sky_follows_the_weather() {
    // The hub has none of its own: the dungeon default.
    let hub = world("showcase");
    assert_eq!(hub.temperature(), Temperature::degrees(10));
    assert_eq!(world("pond").temperature(), Temperature::degrees(-5));
    // Branches by depth: colder deeper.
    let depths = CONTENT.dungeon("depths").expect("the depths");
    let at = |d| depths.plan(d).and_then(|p| p.temperature);
    assert_eq!(at(1), Some(Temperature::degrees(10)));
    assert_eq!(at(4), Some(Temperature::degrees(8)));
    assert_eq!(at(8), Some(Temperature::degrees(6)));
    let mut deep = world("depths");
    assert_eq!(deep.temperature(), Temperature::degrees(10));
    // Under the open sky it is the weather's. The player feels it wherever they are.
    let mut yard = world("winter_yard");
    let outdoors = |w: &World| temperature::outdoors(w.seed(), w.time(), CONTENT.climate(), None);
    assert_eq!(yard.temperature(), outdoors(&yard));
    run(&mut yard, "rest:dusk");
    assert_eq!(yard.temperature(), outdoors(&yard));
    assert_eq!(yard.perception().temperature(), yard.temperature());
    assert!(yard.temperature().is_freezing(), "a deep winter dusk");
    assert_eq!(hub.perception().temperature(), Temperature::degrees(10));
    run(&mut deep, "wait");
    assert_eq!(deep.perception().temperature(), Temperature::degrees(10));
}

#[test]
fn ice_looks_like_its_thickness_and_open_water_like_water() {
    let w = world("pond");
    assert_eq!(terrain_id(&w, THIN), "thin_ice");
    assert_eq!(terrain_id(&w, ICE), "ice");
    assert_eq!(terrain_id(&w, THICK), "thick_ice");
    assert_eq!(terrain_id(&w, Pos::new(2, 2)), "water");
    assert_eq!((mm(&w, THIN), mm(&w, ICE), mm(&w, THICK)), (20, 50, 200));
    // The player sees what it looks like, never how thick it is.
    let view = w.perception();
    let name = |p| match view.cell(p) {
        CellView::Visible(t) => CONTENT.terrain(t).name.clone(),
        other => panic!("{other:?}"),
    };
    assert_eq!(
        [name(THIN), name(ICE), name(THICK)],
        ["thin ice", "ice", "thick ice"]
    );
}

/// The ice and the terrain it shows on the player's level.
fn ice_map(w: &World) -> (BTreeMap<Pos, crate::ice::Ice>, Vec<TerrainId>) {
    let level = w.state.current();
    (level.ice.clone(), level.terrain.cells().to_vec())
}

#[test]
fn water_freezes_lazily_and_alike_however_the_time_passes() {
    // A day at -5 °C: by resting, by waiting, and by the dev skip (nobody acts).
    let mut rested = world("pond");
    run(&mut rested, "rest:24h");
    let mut waited = world("pond");
    run(&mut waited, "wait*600 rest:23h");
    let mut skipped = world("pond");
    run(&mut skipped, "skip:5h skip:19h");
    assert_eq!(rested.time(), skipped.time());
    assert_eq!(waited.time(), skipped.time());
    assert_eq!(ice_map(&rested), ice_map(&skipped));
    assert_eq!(ice_map(&waited), ice_map(&skipped));
    // Open water froze to about 5 cm: ice that holds a person.
    let open = Pos::new(2, 2);
    assert!(
        (50..=56).contains(&mm(&rested, open)),
        "{}",
        mm(&rested, open)
    );
    assert_eq!(terrain_id(&rested, open), "ice");
    // Thin ice thickened, by less than open water grew (the thicker, the slower).
    let thin = mm(&rested, THIN);
    assert!(thin > 50 && thin - 20 < 50, "{thin}");
    // Up to the last tick of the clock.
    assert_eq!(
        rested.state.current().ice_until(),
        ice::tick_floor(rested.time())
    );
}

#[test]
fn ice_melts_in_the_warmth() {
    let mut w = world("warm");
    run(&mut w, "rest:24h");
    // 5 °C melts 36 mm a day: thin ice is gone, thick ice thinner.
    assert_eq!(mm(&w, Pos::new(3, 1)), 200 - 36);
    assert_eq!(w.state.current().ice_at(Pos::new(3, 2)), None);
    assert_eq!(terrain_id(&w, Pos::new(3, 2)), "water");
    assert_eq!(terrain_id(&w, Pos::new(5, 1)), "water");
}

#[test]
fn ice_holds_by_weight_carried_things_included() {
    // Thin ice gives way under a person; thick ice holds one, and 5 cm holds one alone (87.5
    // kg).
    let mut w = world("pond");
    run(&mut w, "move:S move:SE");
    assert!(said(&w, PLUNGE), "thin ice gives way under a person");
    let mut w = world("pond");
    let player = w.player_id();
    run(&mut w, "move:S move:E move:E move:SE");
    assert_eq!(w.player_pos(), THICK);
    assert_eq!(terrain_id(&w, THICK), "thick_ice");
    run(&mut w, "move:W");
    assert_eq!((w.player_pos(), terrain_id(&w, ICE)), (ICE, "ice".into()));
    assert!(!said(&w, PLUNGE));
    // Laden with 115 kg of rocks (185 kg in all, more than twice what it holds) it gives way
    // for sure.
    // From the thick ice (which holds anything) onto the 5 cm ice, laden.
    let to_thick = "move:S move:E move:E move:SE";
    let mut w = world("pond");
    carry_rocks(&mut w, player, 144);
    run(&mut w, to_thick);
    assert_eq!(w.player_pos(), THICK);
    run(&mut w, "move:W");
    assert_eq!(w.player_pos(), ICE);
    assert!(said(&w, PLUNGE));
    assert_eq!(terrain_id(&w, ICE), "water");
    // Between what it holds and twice that, the gameplay stream decides; if it holds, it
    // creaks. Under what it holds, nothing is drawn.
    let mut light = world("pond");
    carry_rocks(&mut light, player, 10); // 78 kg
    run(&mut light, to_thick);
    let rng = light.state.rng.clone();
    run(&mut light, "move:W");
    assert_eq!(light.player_pos(), ICE);
    assert_eq!(light.state.rng, rng, "safe ice draws nothing");
    let mut heavy = world("pond");
    carry_rocks(&mut heavy, player, 30); // 94 kg: a 7 % chance
    run(&mut heavy, to_thick);
    let rng = heavy.state.rng.clone();
    run(&mut heavy, "move:W");
    assert_ne!(heavy.state.rng, rng, "risky ice draws");
    assert!(said(&heavy, PLUNGE) || said(&heavy, "The ice creaks under your weight."));
}

#[test]
fn falling_through_hurts_a_little_and_climbing_out_takes_a_few_turns() {
    let mut w = world("pond");
    let (hp, _) = w.perception().own_hp();
    let out = run(&mut w, "move:S move:SE");
    assert_eq!(w.player_pos(), THIN);
    assert!(said(&w, PLUNGE));
    assert!(
        out[1]
            .events
            .iter()
            .any(|e| matches!(e, Event::FellThroughIce { at, .. } if *at == THIN))
    );
    let (now, _) = w.perception().own_hp();
    assert!((1..=3).contains(&(hp - now)), "{hp} -> {now}");
    // A hole: open water, refreezing from nothing; the step was a step into the water.
    assert_eq!(terrain_id(&w, THIN), "water");
    assert_eq!(w.state.current().ice_at(THIN), None);
    assert_eq!(out[1].elapsed, Duration::seconds(12));
    // Climbing out onto ice that holds takes the step (7.2 s) and the climb (12 s).
    let out = run(&mut w, "move:E");
    assert_eq!(w.player_pos(), ICE);
    assert!(said(&w, "You climb out onto the ice."));
    let took = out[0].elapsed.as_seconds();
    assert!((19..=20).contains(&took), "{took}");
    // Back into the hole is wading, and out onto the floor an ordinary step.
    let out = run(&mut w, "move:W move:W");
    assert_eq!(out[1].elapsed, Duration::seconds(6));
}

#[test]
fn the_cold_never_takes_the_last_hit_point() {
    let mut w = world("pond");
    let player = w.player_id();
    w.state.actors.get_mut(&player).expect("player").hp = 1;
    run(&mut w, "move:S move:SE");
    assert!(said(&w, PLUNGE));
    assert_eq!(w.perception().own_hp().0, 1);
    assert!(w.death().is_none());
}

#[test]
fn what_lay_on_the_ice_sinks_to_the_bottom_and_can_be_fished_out() {
    let mut w = world("pond");
    let dagger_at = Pos::new(7, 3);
    run(&mut w, "move:E*7 move:S move:SW");
    assert_eq!(w.player_pos(), dagger_at);
    assert!(said(&w, PLUNGE));
    assert!(said(&w, "A dagger sinks into the water."));
    assert_eq!(terrain_id(&w, dagger_at), "water");
    let dagger = CONTENT.object_id("dagger").expect("dagger");
    assert_eq!(w.perception().objects(dagger_at)[0].object, dagger);
    run(&mut w, "pickup");
    assert_eq!(w.perception().inventory()[0].object, dagger);
}

#[test]
fn a_hole_refreezes_and_wading_breaks_a_skin_of_ice() {
    let mut w = world("pond");
    run(&mut w, "move:S move:SE");
    assert_eq!(mm(&w, THIN), 0);
    // Ten minutes at -5 °C: a skin too thin to look like ice, on all the open water.
    run(&mut w, "wait*100");
    let skin = mm(&w, THIN);
    assert!((1..10).contains(&skin), "{skin}");
    assert_eq!(terrain_id(&w, THIN), "water");
    assert_eq!(mm(&w, Pos::new(2, 2)), skin);
    // Wading into it breaks it.
    run(&mut w, "move:N");
    assert_eq!(w.player_pos(), Pos::new(2, 2));
    assert_eq!(mm(&w, Pos::new(2, 2)), 0);
    // A few hours on, the hole is thin ice again.
    run(&mut w, "move:N rest:4h");
    assert_eq!(terrain_id(&w, THIN), "thin_ice");
}

#[test]
fn creatures_fall_through_too() {
    let mut w = world("crossing");
    let deer = (w.state.actors.values())
        .find(|a| a.kind == ActorKind::Creature)
        .map(|a| a.id)
        .expect("the deer");
    let hp = w.state.actors[&deer].hp;
    let mut fell = Vec::new();
    for _ in 0..4 {
        fell.extend(
            run(&mut w, "wait")
                .remove(0)
                .events
                .into_iter()
                .filter(|e| matches!(e, Event::FellThroughIce { actor, .. } if *actor == deer)),
        );
    }
    assert_eq!(fell.len(), 1, "it fell through once (then waded the hole)");
    assert!(said(&w, "The deer falls through the ice!"));
    let now = w.state.actors[&deer].hp;
    assert!((1..=3).contains(&(hp - now)), "{hp} -> {now}");
    assert_eq!(terrain_id(&w, Pos::new(3, 3)), "water");
}

#[test]
fn one_who_makes_way_onto_the_ice_may_fall_through() {
    let mut w = world("ferry");
    let ox = (w.state.actors.values())
        .find(|a| a.kind == ActorKind::Creature)
        .map(|a| a.id)
        .expect("the ox");
    let out = run(&mut w, "move:E move:E");
    assert!(said(&w, "You swap places with the ox."));
    assert_eq!(w.state.actors[&ox].pos, Pos::new(2, 1));
    let fell = (out[1].events.iter())
        .any(|e| matches!(e, Event::FellThroughIce { actor, .. } if *actor == ox));
    assert!(fell, "four hundred kilos break 5 cm of ice");
    assert_eq!(terrain_id(&w, Pos::new(2, 1)), "water");
}

#[test]
fn an_instant_frost_freezes_the_water_it_reaches() {
    let mut w = world("pond");
    let out = run(&mut w, "freeze:3,2,1");
    assert!(!out[0].took_turn && out[0].elapsed.is_zero());
    // It shows as a cold burst over the cells it reached (`frost_burst`, data/effects.toml).
    assert!(out[0].events.iter().any(|e| matches!(
        e,
        Event::Spell { effect, cells, .. } if effect == "frost_burst" && cells.contains(&ICE)
    )));
    // Open water within the burst is ice that holds a laden person; thick ice stays thick.
    for p in [Pos::new(2, 2), Pos::new(3, 2), Pos::new(4, 2), THIN, ICE] {
        assert_eq!(mm(&w, p), 100, "{p}");
        assert_eq!(terrain_id(&w, p), "ice");
    }
    assert_eq!(mm(&w, THICK), 200);
    assert_eq!(mm(&w, Pos::new(6, 2)), 0, "out of reach");
    // Walls stop it: the puddle behind one stays open.
    let mut w = world("pond");
    run(&mut w, "freeze:6,5,2");
    assert_eq!(mm(&w, Pos::new(6, 4)), 100);
    assert_eq!(mm(&w, Pos::new(8, 5)), 0, "behind the wall");
}

#[test]
fn heat_melts_the_ice_and_whoever_stood_on_it_falls_in() {
    let mut w = world("pond");
    run(&mut w, "move:S move:E move:E move:SE");
    assert_eq!(w.player_pos(), THICK);
    let (hp, _) = w.perception().own_hp();
    let out = run(&mut w, "thaw:4,3,1");
    assert!(said(&w, PLUNGE));
    assert!(w.perception().own_hp().0 < hp);
    assert!(
        out[0]
            .events
            .iter()
            .any(|e| matches!(e, Event::FellThroughIce { .. }))
    );
    for p in [ICE, THICK, Pos::new(5, 3)] {
        assert_eq!(terrain_id(&w, p), "water", "{p}");
    }
    assert_eq!(terrain_id(&w, THIN), "thin_ice", "beyond the burst");
}

#[test]
fn a_fireball_melts_the_ice_and_a_burst_of_frost_freezes_the_water() {
    let mut w = world("bursts");
    let (near, far) = (Pos::new(2, 2), Pos::new(3, 2));
    assert_eq!((mm(&w, near), mm(&w, far)), (200, 200));
    run(&mut w, "move:E");
    assert!(said(&w, "A fireball explodes!"));
    assert_eq!(
        (mm(&w, near), mm(&w, far)),
        (0, 0),
        "the heat melts thick ice"
    );
    assert_eq!(terrain_id(&w, near), "water");
    let water = [Pos::new(7, 2), Pos::new(8, 2), Pos::new(9, 2)];
    assert!(water.iter().all(|&p| mm(&w, p) < 10), "open water");
    run(&mut w, "move:E*6");
    assert!(said(&w, "Frost bursts from the floor!"));
    for p in water {
        assert!(mm(&w, p) >= 100, "{p:?} froze at once");
        assert_eq!(terrain_id(&w, p), "ice");
    }
}

#[test]
fn blasts_shatter_only_thin_enough_ice() {
    let mut w = world("pond");
    let mut events = Vec::new();
    let broke = w.shock(Pos::new(3, 3), 1, Shock::Blast(100_000), &mut events);
    assert_eq!(broke, 2, "the thin ice and the 6 cm ice");
    assert_eq!(terrain_id(&w, THIN), "water");
    assert_eq!(terrain_id(&w, ICE), "water");
    assert_eq!(terrain_id(&w, THICK), "thick_ice");
}

#[test]
fn ice_survives_a_save_and_replays_reproduce_it() {
    let mut w = world("pond");
    run(
        &mut w,
        "move:S move:SE move:E freeze:6,2,1 rest:3h move:W thaw:2,3,0 wait",
    );
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, CONTENT.clone()).expect("the save loads");
    assert_eq!(loaded.state(), w.state());
    assert_eq!(loaded.state_hash(), w.state_hash());
    let actions = crate::action::parse_actions(
        "move:S move:SE move:E freeze:6,2,1 rest:3h move:W thaw:2,3,0 wait move:E*3",
    )
    .expect("actions");
    let mut a = world("pond");
    let mut replay = Replay::new(&a);
    for action in &actions {
        a.apply(action);
        replay.push(action, &a);
    }
    let replayed = replay.run(CONTENT.clone()).expect("the replay verifies");
    assert_eq!(replayed.state_hash(), a.state_hash());
    assert_eq!(replayed.state(), a.state());
}

#[test]
fn a_long_absence_catches_up_at_once_and_exactly() {
    // Leave the winter courtyard for the hub for a year and come back: the pond froze and
    // thawed with every day of it, worked out on the way back in.
    let away = |skips: &str| {
        let mut w = world("winter_yard");
        run(&mut w, "freeze:20,14,2");
        let to_hub = "move:W move:W";
        run(&mut w, to_hub);
        assert_eq!(w.state.current_level, "courtyard");
        // The courtyard's `>` is at (4, 9): walk there, descend, wait out the time, return.
        run(&mut w, "descend");
        assert_eq!(w.state.current_level, "hub");
        run(&mut w, skips);
        let start = std::time::Instant::now();
        run(&mut w, "ascend");
        let took = start.elapsed();
        assert_eq!(w.state.current_level, "courtyard");
        (ice_map(&w), w.time(), took)
    };
    let (once, t1, took) = away("skip:8640h");
    let (in_parts, t2, _) = away("skip:2880h skip:2880h skip:2880h");
    assert_eq!(t1, t2);
    assert_eq!(once, in_parts);
    // A year of ten-minute ticks is caught up in well under a second, even unoptimised.
    assert!(took.as_millis() < 1500, "{took:?}");
}

#[test]
fn the_courtyard_pond_is_frozen_in_deep_winter_and_open_in_high_summer() {
    let pond = Pos::new(20, 14);
    let winter = world("winter_yard");
    assert!(mm(&winter, pond) >= 50, "{}", mm(&winter, pond));
    let summer = world("summer_yard");
    assert_eq!(mm(&summer, pond), 0);
    assert_eq!(terrain_id(&summer, pond), "water");
}
