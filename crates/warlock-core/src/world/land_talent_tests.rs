//! The Shadow and Wayfarer talents (spec §8 "Talents"; #239): effective eyes (Night eyes, Dark
//! adaptation, Owl eyes, Rune sight), Blood scent, Moon-eyed, Lamplighter, Ambusher, and the
//! land's: Pathfinder, Swimmer, Winter-born, Marsh walker, Beast-friend, Weather-wise,
//! Sun-blessed. Each changes the one rule it names.

use super::ice::Footing;
use super::*;
use crate::action::{AttackKind, parse_actions};
use crate::combat::{Blow, Strike};
use crate::content::{ContentFiles, Stance};
use crate::daylight;
use crate::light::Light;
use std::sync::LazyLock;

/// A pitch-dark corridor: the player at its west end, a giant rat seven cells east.
const DEN: &str = r#"
name = "Den"
ambient = [0, 0, 0, 0]
map = '''
##########
#@......r#
##########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "rat" }
"#;

/// A lit fen below freezing: still water (~), marsh (m), reeds (r), ice 5 cm thick (i), and
/// thin ice (t), the player on the floor west of them.
const FEN: &str = r#"
name = "Fen"
temperature = -5
ambient = [255, 255, 255, 0]
map = '''
#########
#@~mrit.#
#########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"~" = "water"
"m" = "marsh"
"r" = "reeds"
"i" = { terrain = "water", ice = 50 }
"t" = { terrain = "water", ice = 20 }
"#;

static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    for (name, level) in [("den", DEN), ("fen", FEN)] {
        files.insert(&format!("levels/{name}.toml"), level);
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!(
                "name = \"{name}\"\nstart_level = \"{name}\"\nhp = 20\n\
                 stats = {{ strength = 40, toughness = 40 }}\n"
            ),
        );
    }
    Arc::new(Content::load(&files).expect("test content is valid"))
});

fn world(scenario: &str) -> World {
    World::new(CONTENT.clone(), scenario, None).expect("scenario starts")
}

fn run(world: &mut World, text: &str) -> Vec<Outcome> {
    let actions = parse_actions(text).expect("actions");
    actions.iter().map(|a| world.apply(a)).collect()
}

/// The player takes talent `id` (one rank more), without a level to pick it at, and looks
/// again.
fn give(world: &mut World, id: &str) {
    assert!(CONTENT.talents().id(id).is_some(), "{id}");
    world.player_mut().talents.take(id, None);
    world.perceive(&mut Vec::new());
}

fn look(world: &mut World) {
    world.perceive(&mut Vec::new());
}

/// The first creature on the player's level.
fn creature(world: &World) -> ActorId {
    let level = world.state.current().id.clone();
    (world.state.actors.values())
        .find(|a| a.id != world.state.player && a.level == level)
        .map(|a| a.id)
        .expect("a creature")
}

fn sees(world: &World, id: ActorId) -> bool {
    world.perception().actors().iter().any(|a| a.id == id)
}

fn terrain<'a>(world: &'a World, id: &str) -> &'a crate::content::TerrainDef {
    world
        .content()
        .terrain(world.content().terrain_id(id).expect("terrain"))
}

#[test]
fn the_eyes_are_the_profile_as_the_talents_change_it() {
    let mut w = world("den");
    assert_eq!(w.eyes().threshold, 40);
    give(&mut w, "night_eyes");
    assert_eq!(w.eyes().threshold, 20, "half the light");
    assert_eq!(
        w.perception().vision().threshold,
        20,
        "perception reads the eyes"
    );
    assert_eq!(
        w.perception().vision().id,
        "normal",
        "named as the base profile"
    );
    assert!(!w.eyes().sees_uv);
    give(&mut w, "rune_sight");
    assert!(w.eyes().sees_uv);
    // Owl eyes give low-light eyes darkvision of their own.
    let mut w = world("den");
    w.state.vision = "low_light".into();
    give(&mut w, "owl_eyes");
    assert_eq!((w.eyes().threshold, w.eyes().darkvision), (12, 2));
}

#[test]
fn dark_adaptation_sees_farther_in_the_dark() {
    let mut w = world("den");
    let rat = creature(&w);
    w.state.vision = "darkvision".into();
    look(&mut w);
    assert!(!sees(&w, rat), "seven cells off, beyond darkvision");
    give(&mut w, "dark_adaptation");
    assert_eq!(w.eyes().darkvision, 7);
    assert!(sees(&w, rat));
    assert!(
        w.perception()
            .light(w.state.actors[&rat].pos)
            .is_some_and(|l| l.darkvision)
    );
}

#[test]
fn blood_scent_sees_a_bleeding_creature_in_the_dark() {
    let mut w = world("den");
    let rat = creature(&w);
    w.state.vision = "darkvision".into();
    give(&mut w, "blood_scent");
    assert!(!sees(&w, rat), "not bleeding");
    let bleeding = CONTENT.status_id("bleeding").expect("bleeding");
    w.afflict(rat, bleeding, 1, None, &mut Vec::new());
    look(&mut w);
    assert!(sees(&w, rat), "bleeding, within the scent's reach");
    // Without the talent, the same rat stays unseen.
    w.player_mut().talents = Default::default();
    look(&mut w);
    assert!(!sees(&w, rat));
}

#[test]
fn a_full_moon_lets_hounds_nose_smell_blood_two_cells_farther() {
    // Hound's nose reaches six cells: the rat bleeds seven off. The built-in moon is full at
    // the start of the 16th (spec §8 "Birth sign", the flavour; underground all the same).
    let mut w = world("den");
    let rat = creature(&w);
    w.state.vision = "darkvision".into();
    give(&mut w, "hounds_nose");
    let bleeding = CONTENT.status_id("bleeding").expect("bleeding");
    w.afflict(rat, bleeding, 1, None, &mut Vec::new());
    let at = |day: u32| crate::time::GameTime::from_calendar(1444, 11, day, 0, 0).expect("a date");
    w.state.time = at(12);
    look(&mut w);
    assert!(!sees(&w, rat), "a waxing moon: out of reach");
    w.state.time = at(16);
    look(&mut w);
    assert!(sees(&w, rat), "the full moon: two cells more");
    // Blood scent is no gift: the moon does not lengthen it.
    w.player_mut().talents = Default::default();
    give(&mut w, "blood_scent");
    let reach = w.state.player().talent_mods(&CONTENT).scent[0].1;
    if reach < 7 {
        assert!(!sees(&w, rat), "Blood scent's {reach} cells");
    }
}

#[test]
fn moon_eyed_sees_farther_on_the_overworld_at_night() {
    let mut w = world("world");
    run(&mut w, "rest:dusk skip:3h");
    assert_eq!(daylight::phase(w.time()), daylight::DayPhase::Night);
    let count = |w: &World| {
        let level = w.state.current();
        let (width, height) = (level.width(), level.height());
        let view = w.perception();
        (0..height)
            .flat_map(|y| (0..width).map(move |x| Pos::new(x, y)))
            .filter(|&p| view.is_visible(p))
            .count()
    };
    let before = count(&w);
    give(&mut w, "moon_eyed");
    assert!(count(&w) > before, "{before} -> {}", count(&w));
    // By day it changes nothing.
    let mut day = world("world");
    let before = count(&day);
    give(&mut day, "moon_eyed");
    assert_eq!(count(&day), before);
}

#[test]
fn lamplighter_burns_the_lantern_brighter() {
    let mut w = world("showcase");
    run(&mut w, "use_tool");
    let preset = |w: &World, id: &str| {
        let light = CONTENT.light_id(id).expect("preset");
        w.perception()
            .light_sources()
            .iter()
            .any(|s| s.preset() == Some(light))
    };
    assert!(preset(&w, "lantern"));
    give(&mut w, "lamplighter");
    assert!(preset(&w, "lantern_bright"));
    assert!(!preset(&w, "lantern"));
    // Hooded, it burns as the hooded light, talent or not.
    w.apply(&Action::UseTool);
    assert!(preset(&w, "lantern_hooded"));
    assert!(!preset(&w, "lantern_bright"));
    // Doused and lit again, it burns bright.
    w.apply(&Action::UseTool);
    w.apply(&Action::UseTool);
    assert!(preset(&w, "lantern_bright"));
    assert!(
        CONTENT
            .light(CONTENT.light_id("lantern_bright").expect("bright"))
            .radius
            == CONTENT
                .light(CONTENT.light_id("lantern").expect("lantern"))
                .radius
                + 1
    );
}

#[test]
fn ambusher_hurts_a_creature_that_has_not_seen_you() {
    let mut w = world("den");
    let (player, rat) = (w.player_id(), creature(&w));
    let hit = Blow {
        strike: Strike::Hit { critical: false },
        damage: 2,
        kind: None,
    };
    assert_eq!(
        {
            let unseen = w.sneaking(player, rat, false);
            w.ambushed(player, unseen, hit)
        },
        hit,
        "without the talent"
    );
    give(&mut w, "ambusher");
    assert!(
        w.unaware(rat, player),
        "in the dark, it does not see the player"
    );
    let rng = w.state.rng.clone();
    let missed = Blow::missed(None);
    assert_eq!(
        {
            let unseen = w.sneaking(player, rat, false);
            w.ambushed(player, unseen, missed)
        },
        missed,
        "a miss"
    );
    assert_eq!(w.state.rng, rng, "nothing drawn for a miss");
    let ambushed = {
        let unseen = w.sneaking(player, rat, false);
        w.ambushed(player, unseen, hit)
    }
    .damage;
    assert!((3..=8).contains(&ambushed), "{ambushed}");
    // A creature hunting someone it has seen is no longer unaware.
    w.state.actors.get_mut(&rat).expect("rat").mind.last_seen = Some(Pos::new(1, 1));
    assert!(!w.unaware(rat, player));
    assert_eq!(
        {
            let unseen = w.sneaking(player, rat, false);
            w.ambushed(player, unseen, hit)
        },
        hit
    );
    // Nor is one that sees the player, in the light.
    let mut lit = world("den");
    give(&mut lit, "ambusher");
    let rat = creature(&lit);
    lit.state.current_mut().ambient = Light::new(255, 255, 255, 0);
    assert!(!lit.unaware(rat, player));
    assert_eq!(
        {
            let unseen = lit.sneaking(player, rat, false);
            lit.ambushed(player, unseen, hit)
        },
        hit
    );
}

#[test]
fn pathfinder_prices_rough_land_as_plains() {
    let mut w = world("world");
    let player = w.player_id();
    let price = |w: &World, id: &str| w.step_price(player, terrain(w, id), false);
    let plains = price(&w, "plains");
    assert!(price(&w, "forest") > plains);
    give(&mut w, "long_strider");
    give(&mut w, "pathfinder");
    let plains = price(&w, "plains");
    for id in ["forest", "dense_forest", "hills", "swamp"] {
        assert_eq!(price(&w, id), plains, "{id}");
    }
    assert!(price(&w, "road") < plains, "roads stay quicker");
}

#[test]
fn swimmer_and_marsh_walker_wade_as_they_walk() {
    let mut w = world("fen");
    let player = w.player_id();
    let price = |w: &World, id: &str| w.step_price(player, terrain(w, id), false);
    let floor = price(&w, "floor");
    for id in ["water", "running_water", "shallow_water", "marsh", "reeds"] {
        assert!(price(&w, id) > floor, "{id}");
    }
    give(&mut w, "swimmer");
    for id in ["water", "running_water", "shallow_water"] {
        assert_eq!(price(&w, id), floor, "{id}");
    }
    assert!(price(&w, "marsh") > floor);
    give(&mut w, "marsh_walker");
    assert_eq!((price(&w, "marsh"), price(&w, "reeds")), (floor, floor));
    // Through the turn: the wade east takes a plain step.
    let out = run(&mut w, "move:E");
    assert_eq!(out[0].elapsed, Duration::seconds(6));
}

#[test]
fn a_swimmer_takes_no_harm_from_a_plunge() {
    let plunge = |swimmer: bool| {
        let mut w = world("fen");
        if swimmer {
            give(&mut w, "swimmer");
        }
        let hp = w.perception().own_hp().0;
        run(&mut w, "move:E*5");
        assert_eq!(w.player_pos(), Pos::new(6, 1));
        let fell = w.messages().iter().any(|m| m.text.contains("plunge"));
        assert!(fell, "thin ice gives way under a person");
        hp - w.perception().own_hp().0
    };
    assert!(plunge(false) > 0);
    assert_eq!(plunge(true), 0);
}

#[test]
fn winter_born_ice_holds_and_climbing_out_is_quicker() {
    let laden = |winter_born: bool| {
        let mut w = world("fen");
        let rock = CONTENT.object_id("rock").expect("rock");
        w.player_mut().inventory = vec![crate::object::Stack::new(rock, 144)];
        if winter_born {
            give(&mut w, "winter_born");
        }
        run(&mut w, "move:E*4");
        assert_eq!(w.player_pos(), Pos::new(5, 1));
        w.messages().iter().any(|m| m.text.contains("plunge"))
    };
    assert!(laden(false), "115 kg of rocks break 5 cm of ice");
    assert!(
        !laden(true),
        "it holds a Winter-born as if they carried nothing"
    );
    let mut w = world("fen");
    let player = w.player_id();
    let climb = Footing::Held { climb: true };
    let price = w.tread_price(player, climb);
    give(&mut w, "winter_born");
    assert_eq!(w.tread_price(player, climb), price / 2);
}

#[test]
fn a_marsh_walker_never_catches_a_disease_wading() {
    let mut w = world("fen");
    let marsh = CONTENT.terrain_id("marsh").expect("marsh");
    give(&mut w, "marsh_walker");
    let rng = w.state.rng.clone();
    // A hundred hours in the marsh: a sure catch for anyone else.
    w.wade(marsh, 360_000);
    assert_eq!(w.state.rng, rng, "nothing drawn");
    assert!(w.state.player().diseases.is_empty());
    let mut other = world("fen");
    other.wade(marsh, 360_000);
    assert_ne!(other.state.rng, rng, "exposed, they check");
}

#[test]
fn a_beast_friend_is_left_be_by_animals_until_they_attack_one() {
    let mut w = world("den");
    let (player, rat) = (w.player_id(), creature(&w));
    assert_eq!(w.stance(player, rat), Stance::Hostile);
    give(&mut w, "beast_friend");
    assert_eq!(w.stance(player, rat), Stance::Neutral);
    assert_eq!(w.stance(rat, player), Stance::Neutral);
    // An attack (here only its intent) ends it for the type, and it is saved.
    w.attack_with(player, rat, AttackKind::Melee, None, &mut Vec::new());
    assert_eq!(w.stance(player, rat), Stance::Hostile);
    assert!(w.state.progress.empathy_broken.contains("rat"));
    let json = serde_json::to_string(&w.state).expect("serializes");
    let back: WorldState = serde_json::from_str(&json).expect("loads");
    assert_eq!(back.progress, w.state.progress);
}

#[test]
fn a_beast_friend_sees_animals_as_neutral() {
    let mut w = world("den");
    w.state.current_mut().ambient = Light::new(255, 255, 255, 0);
    look(&mut w);
    let hostile = |w: &World| {
        let view = w.perception();
        (view.actors().iter()).any(|a| view.stance(a) == Stance::Hostile)
    };
    assert!(hostile(&w), "the rat, seen in the light");
    give(&mut w, "beast_friend");
    assert!(!hostile(&w));
    let view = w.perception();
    let vermin = CONTENT.factions().id("vermin").expect("vermin");
    let rats: Vec<_> = (view.actors().iter())
        .filter(|a| a.faction == Some(vermin))
        .collect();
    assert!(!rats.is_empty(), "the rat is in sight");
    for seen in rats {
        assert_eq!(view.stance(seen), Stance::Neutral);
    }
}

#[test]
fn weather_wise_knows_the_weather_coming() {
    let mut w = world("courtyard");
    assert_eq!(w.perception().forecast(), None, "without the talent");
    // A moment the weather changes within three hours, and one it does not.
    let climate = CONTENT.climate();
    let seed = w.seed();
    let changes = |t: GameTime| crate::weather::forecast(seed, t, climate, None, 3);
    let soon = (0..200u64)
        .map(|h| w.time() + Duration::hours(h))
        .find(|&t| changes(t).is_some())
        .expect("the weather changes within 200 hours");
    let next = changes(soon).expect("a change");
    assert_ne!(next, crate::weather::weather_at(seed, soon, climate, None));
    give(&mut w, "weather_wise");
    assert_eq!(w.perception().forecast(), changes(w.time()));
    w.state.time = soon;
    look(&mut w);
    assert_eq!(w.perception().forecast(), Some(next));
    // Underground, the sky is not there to read.
    let mut hub = world("showcase");
    give(&mut hub, "weather_wise");
    assert_eq!(hub.perception().forecast(), None);
}

#[test]
fn sun_blessed_heals_faster_under_the_open_sun() {
    let mut w = world("courtyard");
    let player = w.player_id();
    assert!(w.state.sky(&CONTENT).is_some_and(|s| s.sun.is_up()));
    let rate = w.healing_rate(player);
    give(&mut w, "sun_blessed");
    assert_eq!(w.healing_rate(player), rate * 3 / 2);
    run(&mut w, "rest:dusk skip:2h");
    let mut night = world("courtyard");
    run(&mut night, "rest:dusk skip:2h");
    assert_eq!(
        w.healing_rate(player),
        night.healing_rate(player),
        "not at night"
    );
    let mut hub = world("showcase");
    let player = hub.player_id();
    let rate = hub.healing_rate(player);
    give(&mut hub, "sun_blessed");
    assert_eq!(hub.healing_rate(player), rate, "not underground");
}
