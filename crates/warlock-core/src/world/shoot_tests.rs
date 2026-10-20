//! Ranged combat in the world (spec §8 "Ranged combat"): shots and throws fly along their line
//! as far as their range, attack whoever stands in the way, spend ammunition, and come down to be
//! picked up again; burning arrows light the level until they burn out.

use super::*;
use crate::action::{AttackKind, MissileEnd, Positions, parse_actions};
use crate::content::ContentFiles;
use crate::replay::Replay;
use crate::sight::{self, LineOfSight};

const TERRAIN: &str = r##"
[[terrain]]
id = "floor"
name = "floor"
glyph = "."
fg = "#808080"

[[terrain]]
id = "wall"
name = "wall"
glyph = "#"
fg = "#a0a0a0"
walkable = false
transparent = false

[[terrain]]
id = "chasm"
name = "chasm"
glyph = ":"
fg = "#202020"
walkable = false
"##;

const FACTIONS: &str = r##"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "kobolds"
name = "kobolds"
hostile = ["player"]

[[faction]]
id = "wildlife"
name = "wildlife"
"##;

// The post stands still and takes a lot; the watcher, hostile, sees by the light the player
// sees by (no darkvision) and keeps its place until it sees someone.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "post"
name = "training post"
glyph = "P"
fg = "#806040"
weight = 30000
faction = "wildlife"
base_hp = 1000

[[creature]]
id = "watcher"
name = "kobold"
glyph = "k"
fg = "#60a040"
weight = 20000
faction = "kobolds"
sight = 20
base_hp = 1000
"##;

const VISION: &str = r##"
[[vision]]
id = "normal"
name = "Normal"
threshold = 40
"##;

const LIGHTS: &str = r##"
[[light]]
id = "fire"
color = [230, 128, 52, 0]
radius = 5
"##;

const OBJECTS: &str = r##"
[[object]]
id = "bow"
name = "short bow"
glyph = "}"
fg = "#c0c0c0"
class = "missile_weapons"
material = "wood"
weight = 700
value = 0
slot = "missile_weapon"
launches = "arrow"
damage = "1d6"
strength = 10
range = 6

[[object]]
id = "war_bow"
name = "long war bow"
glyph = "}"
fg = "#c0c0c0"
class = "missile_weapons"
material = "wood"
weight = 1300
value = 0
slot = "missile_weapon"
launches = "arrow"
damage = "1d8+2"
crit = 3
strength = 14
range = 6

[[object]]
id = "crossbow"
name = "crossbow"
glyph = "}"
fg = "#c0c0c0"
class = "missile_weapons"
material = "wood"
weight = 3200
value = 0
slot = "missile_weapon"
launches = "bolt"
damage = "1d10+1"
range = 6
reloads = true

[[object]]
id = "arrow"
name = "arrow"
glyph = "/"
fg = "#c0c0c0"
class = "missiles"
material = "wood"
weight = 30
value = 0
stackable = true
slot = "missiles"
ammo = "arrow"

[[object]]
id = "brittle_arrow"
name = "brittle arrow"
glyph = "/"
fg = "#c0c0c0"
class = "missiles"
material = "wood"
weight = 30
value = 0
stackable = true
slot = "missiles"
ammo = "arrow"
breaks = 100

[[object]]
id = "flaming_arrow"
name = "flaming arrow"
glyph = "/"
fg = "#c08040"
class = "missiles"
material = "wood"
weight = 40
value = 0
stackable = true
slot = "missiles"
ammo = "arrow"
lit = "burning_arrow"

[[object]]
id = "burning_arrow"
name = "burning arrow"
glyph = "/"
fg = "#ffc070"
class = "missiles"
material = "wood"
weight = 30
value = 0
stackable = true
light = "fire"
burns = 60

[[object]]
id = "bolt"
name = "bolt"
glyph = "/"
fg = "#c0c0c0"
class = "missiles"
material = "wood"
weight = 60
value = 0
stackable = true
slot = "missiles"
ammo = "bolt"

[[object]]
id = "dagger"
name = "dagger"
glyph = ")"
fg = "#c0c0c0"
class = "melee_weapons"
material = "wood"
weight = 450
value = 0
slot = "main_hand"
damage = "1d4"
range = 4

[[object]]
id = "club"
name = "club"
glyph = ")"
fg = "#c0c0c0"
class = "melee_weapons"
material = "wood"
weight = 1400
value = 0
slot = "main_hand"
damage = "1d6"
parry = 1

[[object]]
id = "rock"
name = "rock"
glyph = "*"
fg = "#c0c0c0"
class = "miscellaneous"
material = "wood"
weight = 800
value = 0
stackable = true
damage = "1d3"
range = 6
"##;

const SLOTS: &str = r##"
[[slot]]
id = "main_hand"
name = "Main hand"
action = "wield"
mark = "wielded"
hand = true

[[slot]]
id = "missile_weapon"
name = "Missile weapon"
action = "ready"
mark = "readied"
takes_weapons = true

[[slot]]
id = "missiles"
name = "Missiles"
action = "ready"
mark = "quivered"
"##;

// Shots cost the normal 1000, a crossbow's reload 1000 more: 12 s a shot.
const ACTIONS: &str = "shoot = 1000\nreload = 1000\nthrow = 1000\n";

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
":" = "chasm"
"P" = { terrain = "floor", creature = "post" }
"k" = { terrain = "floor", creature = "watcher" }
"##;

/// The player stands on `@` with `objects` at their feet; the level is fully lit unless `dark`.
fn files(map: &str, objects: &str, dark: bool, scenario: &str) -> ContentFiles {
    let mut files = ContentFiles::default();
    files.insert(
        "data/item_classes.toml",
        crate::content::BUILTIN_ITEM_CLASSES,
    );
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/factions.toml", FACTIONS),
        ("data/creatures.toml", CREATURES),
        ("data/vision.toml", VISION),
        ("data/lights.toml", LIGHTS),
        ("data/objects.toml", OBJECTS),
        ("data/slots.toml", SLOTS),
        ("data/actions.toml", ACTIONS),
        (
            "data/materials.toml",
            "[[material]]\nid = \"wood\"\nname = \"wood\"\n",
        ),
    ] {
        files.insert(path, text);
    }
    files.insert(
        "data/legend.toml",
        &format!(
            "{LEGEND}\"@\" = {{ terrain = \"floor\", spawn = \"player\", objects = [{objects}] }}\n"
        ),
    );
    let ambient = if dark { 0 } else { 255 };
    files.insert(
        "levels/range.toml",
        &format!(
            "name = \"Range\"\nambient = [{ambient}, {ambient}, {ambient}, 0]\nmap = '''\n{map}'''\n"
        ),
    );
    files.insert(
        "scenarios/range.toml",
        &format!("name = \"Range\"\nstart_level = \"range\"\nhp = 1000\n{scenario}\n"),
    );
    files
}

fn world_with(map: &str, objects: &str, dark: bool, scenario: &str) -> World {
    let files = files(map, objects, dark, scenario);
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "range", None).expect("world")
}

/// A world on `map` where the player has picked up `objects` and readied the first two (a
/// missile weapon and its ammunition).
fn armed(map: &str, objects: &str, dark: bool) -> World {
    let mut w = world_with(
        map,
        objects,
        dark,
        "stats = { strength = 12, dexterity = 12 }",
    );
    run(&mut w, "pickup equip:missile_weapon:a equip:missiles:b");
    w
}

fn run(world: &mut World, actions: &str) -> Vec<Outcome> {
    parse_actions(actions)
        .expect("actions")
        .iter()
        .map(|a| world.apply(a))
        .collect()
}

fn last_message(world: &World) -> String {
    world
        .messages()
        .iter()
        .last()
        .map(|m| m.display())
        .unwrap_or_default()
}

fn pile(world: &World, p: Pos) -> Vec<(String, u32)> {
    let (level, _) = world.ground_truth();
    let content = world.content();
    (level.objects_at(p).iter())
        .map(|s| (content.object(s.object).id.clone(), s.count))
        .collect()
}

fn carried(world: &World, id: &str) -> u32 {
    let content = world.content();
    (world.state().player().inventory.iter())
        .filter(|s| content.object(s.object).id == id)
        .map(|s| s.count)
        .sum()
}

fn missiles(out: &Outcome) -> Vec<(Vec<Pos>, MissileEnd)> {
    (out.events.iter())
        .filter_map(|e| match e {
            Event::Missile { path, end, .. } => Some((path.clone(), *end)),
            _ => None,
        })
        .collect()
}

const OPEN: &str = "###########\n#@........#\n###########\n";

#[test]
fn a_shot_at_an_empty_cell_comes_down_there_to_be_picked_up_again() {
    let mut w = armed(OPEN, "\"bow\", \"arrow*5\"", false);
    assert_eq!(carried(&w, "arrow"), 5);
    let out = w.apply(&Action::Fire(Pos::new(5, 1)));
    assert!(out.took_turn);
    assert_eq!(out.elapsed, Duration::seconds(6));
    assert_eq!(last_message(&w), "You shoot an arrow.");
    assert_eq!(carried(&w, "arrow"), 4, "one arrow less");
    let path = (2..=5).map(|x| Pos::new(x, 1)).collect();
    assert_eq!(missiles(&out), [(path, MissileEnd::Fell)]);
    assert_eq!(pile(&w, Pos::new(5, 1)), [("arrow".to_string(), 1)]);
    // Walk there and pick it up: it joins the quiver again.
    run(&mut w, "move:E*4 pickup");
    assert_eq!(carried(&w, "arrow"), 5);
    let view = w.perception();
    assert_eq!(
        view.ready_to_fire().map(|(_, item, range)| (item, range)),
        Some((1, 6))
    );
}

#[test]
fn shots_stop_at_walls_and_fly_no_farther_than_their_range() {
    // A wall in the way: the arrow comes down before it.
    let walled = "###########\n#@..#.....#\n###########\n";
    let mut w = armed(walled, "\"bow\", \"arrow*5\"", false);
    let out = w.apply(&Action::Fire(Pos::new(6, 1)));
    let path: Vec<Pos> = (2..=3).map(|x| Pos::new(x, 1)).collect();
    assert_eq!(missiles(&out), [(path, MissileEnd::Fell)]);
    assert_eq!(pile(&w, Pos::new(3, 1)), [("arrow".to_string(), 1)]);

    // Beyond the bow's range (6): refused, no time, no arrow spent.
    let mut w = armed(OPEN, "\"bow\", \"arrow*5\"", false);
    let out = w.apply(&Action::Fire(Pos::new(8, 1)));
    assert!(!out.took_turn && out.events.is_empty());
    assert_eq!(last_message(&w), "That is out of range.");
    assert_eq!(carried(&w, "arrow"), 5);
    let out = w.apply(&Action::Fire(Pos::new(7, 1)));
    assert!(out.took_turn, "6 cells is in range");
    let out = w.apply(&Action::Fire(w.player_pos()));
    assert!(!out.took_turn);
    assert_eq!(last_message(&w), "You cannot shoot at yourself.");

    // Something that does not fly, or nothing ready: refused.
    let mut w = world_with(OPEN, "\"bow\", \"arrow*5\"", false, "");
    run(&mut w, "pickup");
    assert!(!w.apply(&Action::Fire(Pos::new(4, 1))).took_turn);
    assert_eq!(
        last_message(&w),
        "You have nothing ready to throw or shoot."
    );
    run(&mut w, "equip:missile_weapon:a");
    assert!(!w.apply(&Action::Fire(Pos::new(4, 1))).took_turn);
    assert_eq!(last_message(&w), "You have no arrows ready.");
    assert!(
        !w.apply(&Action::Throw {
            item: 1,
            at: Pos::new(4, 1)
        })
        .took_turn
    );
    assert_eq!(last_message(&w), "You cannot throw an arrow.");
}

#[test]
fn a_missile_attacks_whoever_stands_in_its_way_and_flies_on_when_it_misses() {
    // A training post three cells away; the shot is aimed past it.
    let map = "###########\n#@..P.....#\n###########\n";
    let mut w = armed(map, "\"bow\", \"arrow*40\"", false);
    let post = ActorId(1);
    let (mut hits, mut misses) = (0, 0);
    for _ in 0..30 {
        let out = w.apply(&Action::Fire(Pos::new(7, 1)));
        let attacked: Vec<_> = (out.events.iter())
            .filter_map(|e| match e {
                Event::Attacked {
                    target,
                    kind,
                    strike,
                    with,
                    ..
                } => Some((*target, *kind, *strike, *with)),
                _ => None,
            })
            .collect();
        let arrow = w.content().object_id("arrow");
        assert_eq!(attacked.len(), 1, "the post, and no one else");
        let (target, kind, strike, with) = attacked[0];
        assert_eq!((target, kind, with), (post, AttackKind::Ranged, arrow));
        let [(path, end)] = &missiles(&out)[..] else {
            panic!("one missile");
        };
        if strike.is_some_and(|s| s.is_hit()) {
            hits += 1;
            assert_eq!(
                (path.last(), *end),
                (Some(&Pos::new(4, 1)), MissileEnd::Struck)
            );
            assert!(last_message(&w).starts_with("You shoot the training post"));
        } else {
            misses += 1;
            // It flew on past the post to the cell aimed at.
            assert_eq!(
                (path.last(), *end),
                (Some(&Pos::new(7, 1)), MissileEnd::Fell)
            );
            assert!(last_message(&w).starts_with("You miss the training post."));
        }
    }
    assert!(hits > 5 && misses > 1, "{hits} hits, {misses} misses");
    let (level, _) = w.ground_truth();
    let arrows = |p| level.objects_at(p).first().map_or(0, |s| s.count);
    assert_eq!(arrows(Pos::new(4, 1)), hits, "under the post");
    assert_eq!(arrows(Pos::new(7, 1)), misses, "where it was aimed");
}

#[test]
fn arrows_may_break_on_a_hit_and_cannot_lie_where_there_is_no_floor() {
    // Brittle arrows always break when they hit; the post is hit sooner or later.
    let map = "#######\n#@.P..#\n#######\n";
    let mut w = armed(map, "\"bow\", \"brittle_arrow*30\"", false);
    let mut hit = false;
    for _ in 0..20 {
        let out = w.apply(&Action::Fire(Pos::new(3, 1)));
        hit |= missiles(&out)
            .iter()
            .any(|(_, end)| *end == MissileEnd::Struck);
    }
    assert!(hit);
    let (level, _) = w.ground_truth();
    assert!(
        level.objects_at(Pos::new(3, 1)).is_empty(),
        "broken, every one"
    );
    // Shot over a chasm, an arrow is lost.
    let chasm = "#######\n#@.::.#\n#######\n";
    let mut w = armed(chasm, "\"bow\", \"arrow*3\"", false);
    let out = w.apply(&Action::Fire(Pos::new(3, 1)));
    assert_eq!(missiles(&out)[0].0.last(), Some(&Pos::new(3, 1)));
    assert!(pile(&w, Pos::new(3, 1)).is_empty());
    assert_eq!(carried(&w, "arrow"), 2);
}

#[test]
fn the_last_arrow_empties_the_quiver() {
    let mut w = armed(OPEN, "\"bow\", \"arrow*2\"", false);
    run(&mut w, "fire:3,1");
    let view = w.perception();
    assert!(view.ready_to_fire().is_some());
    run(&mut w, "fire:3,1");
    assert_eq!(last_message(&w), "You have no arrows left.");
    let view = w.perception();
    assert_eq!(view.ready_to_fire(), None);
    assert_eq!(view.inventory().len(), 1, "only the bow");
    let content = w.content().clone();
    let slot = content
        .slot_ids()
        .find(|(_, s)| s.id == "missiles")
        .map(|(id, _)| id);
    assert_eq!(view.equipment().get(slot.expect("missiles slot")), None);
    assert!(!w.apply(&Action::Fire(Pos::new(3, 1))).took_turn);
    assert_eq!(last_message(&w), "You have no arrows ready.");
}

#[test]
fn a_crossbow_takes_its_reload_and_a_bow_its_draw_strength() {
    let mut w = armed(OPEN, "\"crossbow\", \"bolt*3\"", false);
    let out = w.apply(&Action::Fire(Pos::new(5, 1)));
    assert_eq!(out.elapsed, Duration::seconds(12), "shoot and reload");
    // A long war bow (Strength 14) drawn by Strength 12: two short, −2 to hit and to damage; the
    // short bow (Strength 10) is no trouble.
    let w = armed(OPEN, "\"war_bow\", \"arrow*3\"", false);
    let content = w.content();
    let player = w.state().player();
    let profile = player.ranged(content).expect("a bow");
    // Dexterity 12 (+1) less 2.
    assert_eq!((profile.to_hit, profile.bonus, profile.crit), (-1, -2, 3));
    let w = armed(OPEN, "\"bow\", \"arrow*3\"", false);
    let profile = w.state().player().ranged(w.content()).expect("a bow");
    assert_eq!((profile.to_hit, profile.bonus), (1, 0));
}

#[test]
fn thrown_daggers_fly_their_own_range_and_leave_the_hand() {
    let mut w = world_with(
        OPEN,
        "\"dagger\", \"dagger\"",
        false,
        "stats = { strength = 16, dexterity = 8 }",
    );
    run(&mut w, "pickup equip:main_hand:a");
    // Dexterity to hit, Strength to damage.
    let content = w.content().clone();
    let dagger = content.object(content.object_id("dagger").expect("dagger"));
    let thrown = crate::combat::Profile::thrown(
        dagger.attack.as_ref().expect("a weapon"),
        &w.state().player().stats,
    );
    assert_eq!((thrown.to_hit, thrown.bonus), (-1, 3));
    // Its range is 4.
    assert!(
        !w.apply(&Action::Throw {
            item: 0,
            at: Pos::new(6, 1)
        })
        .took_turn
    );
    assert_eq!(last_message(&w), "That is out of range.");
    let out = w.apply(&Action::Throw {
        item: 0,
        at: Pos::new(5, 1),
    });
    assert_eq!(out.elapsed, Duration::seconds(6));
    assert_eq!(last_message(&w), "You throw a dagger.");
    assert_eq!(pile(&w, Pos::new(5, 1)), [("dagger".to_string(), 1)]);
    // The wielded one is gone from the hand; the other moved up a letter, not wielded.
    let view = w.perception();
    assert_eq!(view.inventory().len(), 1);
    assert!(view.equipment().is_empty());
    assert_eq!(view.throw_range(0), Some(4));
    // Arrows are no throwing weapons.
    let mut w = world_with(OPEN, "\"arrow*3\"", false, "");
    run(&mut w, "pickup");
    assert!(
        !w.apply(&Action::Throw {
            item: 0,
            at: Pos::new(3, 1)
        })
        .took_turn
    );
    assert_eq!(last_message(&w), "You cannot throw an arrow.");
}

/// A dark hall: the player at the west end with a bow and flaming arrows, a kobold (normal eyes,
/// no light) at the east end.
const DARK_HALL: &str = "################\n#@............k#\n################\n";

#[test]
fn a_flaming_arrow_burns_where_it_lands_lighting_the_dark_for_a_minute() {
    let mut w = armed(DARK_HALL, "\"bow\", \"flaming_arrow*3\"", true);
    let spot = Pos::new(6, 1);
    assert!(!w.perception().is_visible(spot), "dark");
    let landed = w.time();
    let out = w.apply(&Action::Fire(spot));
    assert_eq!(last_message(&w), "You shoot a flaming arrow.");
    // It flew burning, and lies there burning: a light the player sees by.
    let burning = w
        .content()
        .object_id("burning_arrow")
        .expect("burning arrow");
    let flew: Vec<_> = (out.events.iter())
        .filter_map(|e| match e {
            Event::Missile { object, end, .. } => Some((*object, *end)),
            _ => None,
        })
        .collect();
    assert_eq!(flew, [(burning, MissileEnd::Fell)]);
    assert_eq!(pile(&w, spot), [("burning_arrow".to_string(), 1)]);
    let view = w.perception();
    assert!(view.is_visible(spot) && view.is_visible(Pos::new(8, 1)));
    let fire = (view.light_sources().iter())
        .find(|s| s.pos() == Some(spot))
        .expect("the fire is a light source");
    assert_eq!(fire.preset(), w.content().light_id("fire"));
    assert!(view.light(spot).expect("lit").brightness() > 100);
    // It cannot be picked up while it burns.
    run(&mut w, "move:E*5");
    assert!(!w.apply(&Action::PickUp(Positions::default())).took_turn);
    assert_eq!(
        last_message(&w),
        "You cannot pick up a burning arrow while it burns."
    );
    // A minute after it landed it has burned out: gone, and dark again (the shot, 5 steps, and
    // 4 waits).
    run(&mut w, "wait*3");
    assert_eq!(w.time().since(landed), Duration::seconds(54));
    assert_eq!(
        pile(&w, spot),
        [("burning_arrow".to_string(), 1)],
        "still burning"
    );
    run(&mut w, "wait");
    assert_eq!(w.time().since(landed), Duration::seconds(60));
    assert!(pile(&w, spot).is_empty());
    let (level, _) = w.ground_truth();
    assert!(level.fires.is_empty());
    let view = w.perception();
    assert!(view.light_sources().iter().all(|s| s.pos() != Some(spot)));
    assert_eq!(
        view.light(spot).map(|l| l.brightness()),
        Some(0),
        "only their own cell"
    );
}

#[test]
fn creatures_see_by_a_burning_arrow_too() {
    // In the dark the kobold cannot see the player; a flaming arrow at the player's feet shows
    // them to it.
    let mut w = armed(DARK_HALL, "\"bow\", \"flaming_arrow*3\"", true);
    let kobold = ActorId(1);
    let sees_player = |w: &mut World| {
        let World {
            state,
            content,
            lights,
            ..
        } = w;
        let me = &state.actors[&kobold];
        let level = state.current();
        let los = LineOfSight::compute(me.pos, 20, |p| level.is_opaque(content, p));
        let eye = &content.vision_profiles()[0];
        sight::sees(state, content, lights, &los, eye, state.player().pos)
    };
    run(&mut w, "wait*2");
    assert!(!sees_player(&mut w));
    assert!(w.state().actors[&kobold].mind.is_calm());
    run(&mut w, "fire:2,1");
    assert!(sees_player(&mut w));
    run(&mut w, "wait");
    assert_eq!(
        w.state().actors[&kobold].mind.last_seen,
        Some(Pos::new(1, 1)),
        "it saw the player and comes for them"
    );
}

#[test]
fn fires_survive_a_save_and_burn_out_on_time_after_it() {
    let mut w = armed(DARK_HALL, "\"bow\", \"flaming_arrow*3\"", true);
    run(&mut w, "fire:5,1 wait*2");
    let bytes = crate::save::encode(&w);
    let mut loaded = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    assert_eq!(loaded.state().current().fires.len(), 1);
    for world in [&mut w, &mut loaded] {
        run(world, "wait*7");
        assert!(world.state().current().fires.is_empty());
    }
    assert_eq!(loaded.state_hash(), w.state_hash());
}

#[test]
fn two_burning_arrows_on_one_cell_burn_until_the_last_burns_out() {
    let mut w = armed(DARK_HALL, "\"bow\", \"flaming_arrow*3\"", true);
    run(&mut w, "fire:5,1 wait*4 fire:5,1");
    assert_eq!(pile(&w, Pos::new(5, 1)), [("burning_arrow".to_string(), 2)]);
    // The first burned out 60 s after it landed; the fire burns on for the second's minute.
    run(&mut w, "wait*5");
    assert_eq!(pile(&w, Pos::new(5, 1)), [("burning_arrow".to_string(), 2)]);
    run(&mut w, "wait*5");
    assert!(pile(&w, Pos::new(5, 1)).is_empty());
}

#[test]
fn shots_in_the_dark_are_seen_only_where_they_fly_over_what_the_player_sees() {
    // An arrow into the dark: the player sees it leave (their own cell is lit by nothing, but
    // always visible), not where it lands.
    let mut w = armed(DARK_HALL, "\"bow\", \"arrow*3\"", true);
    let out = w.apply(&Action::Fire(Pos::new(6, 1)));
    assert_eq!(missiles(&out), []);
    assert_eq!(last_message(&w), "You shoot an arrow.");
    // A flaming arrow is light itself: seen all along its flight, in the dark too.
    let mut w = armed(DARK_HALL, "\"bow\", \"flaming_arrow*3\"", true);
    let out = w.apply(&Action::Fire(Pos::new(6, 1)));
    let path = (2..=6).map(|x| Pos::new(x, 1)).collect();
    assert_eq!(missiles(&out), [(path, MissileEnd::Fell)]);
    // Lit: the whole flight.
    let mut w = armed(DARK_HALL, "\"bow\", \"arrow*3\"", false);
    let out = w.apply(&Action::Fire(Pos::new(6, 1)));
    assert_eq!(missiles(&out).len(), 1);
    let from = out.events.iter().find_map(|e| match e {
        Event::Missile { from, .. } => Some(*from),
        _ => None,
    });
    assert_eq!(from, Some(Some(Pos::new(1, 1))));
}

#[test]
fn ranged_fights_replay_exactly() {
    let map = "###########\n#@..P.....#\n###########\n";
    let files = files(map, "\"bow\", \"flaming_arrow*9\", \"dagger\"", true, "");
    let content = Arc::new(Content::load(&files).expect("content"));
    let mut w = World::new(content.clone(), "range", None).expect("world");
    let mut replay = Replay::new(&w);
    for action in parse_actions(
        "pickup equip:missile_weapon:a equip:missiles:b fire:4,1 fire:7,1 wait throw:c:4,1 \
         fire:4,1 wait*3",
    )
    .expect("actions")
    {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let (verified, divergence) = replay.play(content).expect("replays");
    assert_eq!(divergence, None);
    assert_eq!(verified.state_hash(), w.state_hash());
}

#[test]
fn the_missile_weapon_slot_takes_any_weapon_and_throwable_but_not_ammunition() {
    let w = world_with(
        OPEN,
        "\"club\", \"rock*3\", \"arrow*3\", \"bow\"",
        false,
        "",
    );
    let c = w.content();
    let slot = c.slot_id("missile_weapon").expect("slot");
    let fits = |id: &str| c.fits(c.object_id(id).expect(id), slot);
    assert!(fits("club") && fits("rock") && fits("dagger") && fits("bow"));
    assert!(!fits("arrow"), "ammunition stays in the missiles slot");
    let hand = c.slot_id("main_hand").expect("slot");
    assert!(!c.fits(c.object_id("rock").expect("rock"), hand));
}

#[test]
fn a_melee_weapon_in_the_missile_slot_is_thrown_whole_with_a_penalty() {
    let mut w = world_with(
        OPEN,
        "\"club\"",
        false,
        "stats = { strength = 12, dexterity = 12 }",
    );
    run(&mut w, "pickup equip:missile_weapon:a");
    let view = w.perception();
    assert_eq!(view.ready_to_throw(), Some(0));
    assert_eq!(view.throw_range(0), Some(crate::combat::IMPROVISED_RANGE));
    assert!(
        !view.is_throwing_weapon(0),
        "the throw choice lists made-for-throwing only"
    );
    let content = w.content().clone();
    let club = content.object(content.object_id("club").expect("club"));
    let attack = club.attack.as_ref().expect("a weapon");
    let stats = &w.state().player().stats;
    assert_eq!(
        crate::combat::Profile::thrown(attack, stats)
            .improvised()
            .to_hit,
        crate::combat::Profile::thrown(attack, stats).to_hit + crate::combat::IMPROVISED_TO_HIT
    );
    // Beyond its short range: refused. Within it: thrown, leaves the slot, lands.
    assert!(
        !w.apply(&Action::Throw {
            item: 0,
            at: Pos::new(5, 1)
        })
        .took_turn
    );
    assert_eq!(last_message(&w), "That is out of range.");
    let out = w.apply(&Action::Throw {
        item: 0,
        at: Pos::new(3, 1),
    });
    assert!(out.took_turn);
    assert_eq!(last_message(&w), "You throw a club.");
    assert_eq!(pile(&w, Pos::new(3, 1)), [("club".to_string(), 1)]);
    let view = w.perception();
    assert!(view.inventory().is_empty() && view.equipment().is_empty());
    assert_eq!(view.ready_to_throw(), None);
}

#[test]
fn a_stack_of_rocks_readied_in_the_missile_slot_is_thrown_one_at_a_time() {
    let mut w = world_with(OPEN, "\"rock*3\"", false, "");
    run(&mut w, "pickup equip:missile_weapon:a");
    assert_eq!(w.perception().ready_to_throw(), Some(0));
    assert!(
        w.apply(&Action::Throw {
            item: 0,
            at: Pos::new(4, 1)
        })
        .took_turn
    );
    assert_eq!(carried(&w, "rock"), 2);
    assert_eq!(
        w.perception().ready_to_throw(),
        Some(0),
        "the rest stay readied"
    );
}

#[test]
fn a_weapon_readied_to_throw_does_not_parry_and_nothing_readied_takes_no_time() {
    let mut w = world_with(OPEN, "\"club\"", false, "");
    let before = w
        .state()
        .player()
        .defence(w.content(), crate::burden::Burden::Unburdened);
    run(&mut w, "pickup equip:missile_weapon:a");
    let readied = w
        .state()
        .player()
        .defence(w.content(), crate::burden::Burden::Unburdened);
    assert_eq!(readied.total(), before.total());
    run(&mut w, "equip:main_hand:a");
    let held = w
        .state()
        .player()
        .defence(w.content(), crate::burden::Burden::Unburdened);
    assert_eq!(
        held.total(),
        before.total() + 1,
        "wielded, the club parries"
    );
    // Nothing readied: the rules say so.
    let mut w = world_with(OPEN, "", false, "");
    assert_eq!(w.perception().ready_to_throw(), None);
    let out = w.apply(&Action::Fire(Pos::new(4, 1)));
    assert!(!out.took_turn);
    assert_eq!(
        last_message(&w),
        "You have nothing ready to throw or shoot."
    );
}
