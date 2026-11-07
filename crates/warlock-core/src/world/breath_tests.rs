//! Dragons (spec §8 "Creature behavior": "Dragons", `breath.rs`, `natural.rs`): a dragon
//! breathes its cone whenever its breath is ready and the hostile it sees stands in reach with a
//! clear line, and between breaths closes in and strikes with each of its natural attacks.

use super::*;
use crate::action::{AttackKind, Event, parse_actions};
use crate::content::ContentFiles;

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
"##;

const FACTIONS: &str = r##"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "fire"
name = "fire things"
hostile = ["player"]

[[faction]]
id = "wild"
name = "wildlife"
"##;

const STATUS: &str = r##"
[[status]]
id = "breath"
name = "Breath spent"
max = 100
stacking = "refresh"
eases = 6
"##;

const DAMAGE: &str = r##"
[[type]]
id = "slashing"
name = "slashing"
verb = "slashes"
you = "slash"
physical = true

[[type]]
id = "piercing"
name = "piercing"
verb = "pierces"
you = "pierce"
physical = true

[[type]]
id = "fire"
name = "fire"
verb = "burns"
you = "burn"
"##;

/// `{breath}` is the wyrm's `[creature.breath]` body, `{combat}` its combat.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "wyrm"
name = "wyrm"
glyph = "D"
fg = "#e0402a"
weight = 700000
faction = "fire"
sight = 20
base_hp = 1000
level = 10
combat = "{combat}"
resist = { fire = 100 }
[creature.breath]
{breath}
[[creature.attack]]
damage = "2"
damage_type = "slashing"
name = "claw"
verb = "claws"
to_hit = 20
[[creature.attack]]
damage = "3"
damage_type = "slashing"
name = "claw"
verb = "claws"
to_hit = 20
[[creature.attack]]
damage = "5"
damage_type = "piercing"
name = "bite"
verb = "bites"
to_hit = 20

[[creature]]
id = "salamander"
name = "salamander"
glyph = "s"
fg = "#e08020"
weight = 5000
faction = "wild"
base_hp = 1000
resist = { fire = 100 }

[[creature]]
id = "deer"
name = "deer"
glyph = "d"
fg = "#a08040"
weight = 50000
faction = "wild"
base_hp = 1000
"##;

const ACTIONS: &str = "melee = 1000\ncast = 1000\nwait = 1000\nstep = 1000\n";

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
"@" = { terrain = "floor", spawn = "player" }
"D" = { terrain = "floor", creature = "wyrm" }
"s" = { terrain = "floor", creature = "salamander" }
"d" = { terrain = "floor", creature = "deer" }
"##;

/// A long lit hall: the player at the west end, the wyrm six cells east.
const HALL: &str = "############\n#@.....D...#\n#..........#\n#..........#\n############\n";

const FIRE: &str = "damage = \"20\"\ndamage_type = \"fire\"\nlength = 7\ncooldown = 36\n\
effect = \"dragon_fire\"\nnoun = \"fire\"\nverb = \"breathes fire\"";

fn files(breath: &str, combat: &str, map: &str) -> ContentFiles {
    let mut files = ContentFiles::default();
    files.insert(
        "data/item_classes.toml",
        crate::content::BUILTIN_ITEM_CLASSES,
    );
    let creatures = CREATURES
        .replace("{breath}", breath)
        .replace("{combat}", combat);
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/factions.toml", FACTIONS),
        ("data/status.toml", STATUS),
        ("data/damage_types.toml", DAMAGE),
        ("data/creatures.toml", creatures.as_str()),
        ("data/actions.toml", ACTIONS),
        ("data/legend.toml", LEGEND),
    ] {
        files.insert(path, text);
    }
    files.insert(
        "levels/hall.toml",
        &format!("name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n"),
    );
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 1000\n",
    );
    files
}

fn world_with(map: &str) -> World {
    let files = files(FIRE, "dragon", map);
    let content = Arc::new(Content::load(&files).expect("valid content"));
    World::new(content, "hall", None).expect("world")
}

fn run(world: &mut World, actions: &str) -> Vec<Event> {
    let mut events = Vec::new();
    for action in parse_actions(actions).expect("actions") {
        events.extend(world.apply(&action).events);
    }
    events
}

/// The first actor of creature type `kind`.
fn first_of<'w>(w: &'w World, kind: &str) -> &'w Actor {
    let id = w.content().creature_id(kind).expect("a type");
    (w.state.actors.values())
        .find(|a| a.creature == id)
        .expect("the creature")
}

fn said(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

/// The player's hit points unhurt.
fn full(w: &World) -> i32 {
    w.state.player().max_hp(w.content())
}

fn spent(w: &World) -> u16 {
    let status = w.content().status_id("breath").expect("status");
    (first_of(w, "wyrm").effects.get(status)).map_or(0, |e| e.severity)
}

#[test]
fn a_dragon_breathes_at_a_hostile_in_reach_whenever_its_breath_is_ready() {
    let mut w = world_with(HALL);
    let events = run(&mut w, "wait");
    assert_eq!(w.state.player().hp, full(&w) - 20, "the fire struck");
    let said = said(&w);
    assert!(
        said.iter().any(|l| l == "The wyrm breathes fire!"),
        "{said:?}"
    );
    assert!(said.iter().any(|l| l == "The fire burns you."), "{said:?}");
    // 36 s of cooldown at 6 s a degree: six, a degree of which may have eased since.
    assert!((5..=6).contains(&spent(&w)), "{} degrees", spent(&w));
    // The cone's effect, drawn as a burst around the dragon over its cells.
    let wyrm = first_of(&w, "wyrm").pos;
    let cone = events.iter().find_map(|e| match e {
        Event::Spell {
            effect,
            origin,
            cells,
            ..
        } if effect == "dragon_fire" => Some((*origin, cells.clone())),
        _ => None,
    });
    let (origin, cells) = cone.expect("the cone is seen");
    assert_eq!(origin, wyrm);
    assert!(cells.contains(&w.player_pos()));
}

#[test]
fn while_its_breath_cools_it_closes_in_and_never_stands_idle() {
    let mut w = world_with(HALL);
    run(&mut w, "wait");
    let far = first_of(&w, "wyrm").pos.chebyshev(w.player_pos());
    run(&mut w, "wait");
    let near = first_of(&w, "wyrm").pos.chebyshev(w.player_pos());
    assert_eq!(near, far - 1, "it steps toward the player");
    assert_eq!(w.state.player().hp, full(&w) - 20, "no second breath yet");
}

#[test]
fn out_of_the_cones_reach_it_comes_closer_before_it_breathes() {
    let far = "################\n#@...........D.#\n#..............#\n################\n";
    let mut w = world_with(far);
    run(&mut w, "wait");
    assert_eq!(w.state.player().hp, full(&w), "twelve cells off: no breath");
    assert_eq!(first_of(&w, "wyrm").pos.chebyshev(w.player_pos()), 11);
    assert_eq!(spent(&w), 0);
}

#[test]
fn beside_it_during_the_cooldown_it_strikes_with_every_claw_and_its_bite() {
    let mut w = world_with(HALL);
    run(&mut w, "wait");
    let mut blows = Vec::new();
    for _ in 0..10 {
        let events = run(&mut w, "wait");
        let wyrm = first_of(&w, "wyrm").id;
        let melee: Vec<Option<u8>> = (events.iter())
            .filter_map(|e| match e {
                Event::Attacked {
                    attacker,
                    kind: AttackKind::Melee,
                    natural,
                    ..
                } if *attacker == wyrm => Some(*natural),
                _ => None,
            })
            .collect();
        if !melee.is_empty() {
            blows = melee;
            break;
        }
    }
    assert_eq!(
        blows,
        vec![None, Some(1), Some(2)],
        "two claws and a bite, in turn"
    );
    let said = said(&w);
    // Both claws hit: one line, said twice.
    assert!(
        said.iter().any(|l| l == "The wyrm claws you. (×2)"),
        "{said:?}"
    );
    assert!(said.iter().any(|l| l == "The wyrm bites you."), "{said:?}");
}

#[test]
fn several_natural_attacks_are_each_rolled_on_their_own() {
    let w = world_with(HALL);
    let me = first_of(&w, "wyrm");
    let first = me.melee(w.content());
    let more = me.more_natural_blows(w.content());
    assert_eq!(more.len(), 2);
    assert_eq!(more[0].0, 1);
    assert_eq!(more[1].0, 2);
    // Each its own dice, and Strength's bonus on each.
    let bonus = first.bonus;
    let roll = |p: &crate::combat::Profile| p.damage.roll(&mut Rng::stream(1, "t"), 0);
    assert_eq!(roll(&first), 2 + bonus);
    assert_eq!(roll(&more[0].1), 3 + bonus);
    assert_eq!(roll(&more[1].1), 5 + bonus);
}

#[test]
fn the_cone_burns_everyone_in_it_but_what_its_element_cannot_hurt() {
    // The deer stands in the way: no clear line, no breath.
    let map = "############\n#@...d.D...#\n#....s.....#\n#..........#\n############\n";
    let mut w = world_with(map);
    run(&mut w, "wait");
    assert_eq!(w.state.player().hp, full(&w));
    assert_eq!(spent(&w), 0);
    // A deer and a salamander between the wyrm and the player, off the line between them: in
    // the cone, not in the way.
    let map = "############\n#@.....D...#\n#...d......#\n#...s......#\n############\n";
    let mut w = world_with(map);
    run(&mut w, "wait");
    assert_eq!(w.state.player().hp, full(&w) - 20);
    assert_eq!(
        first_of(&w, "deer").hp,
        first_of(&w, "deer").max_hp(w.content()) - 20
    );
    let salamander = first_of(&w, "salamander");
    assert_eq!(
        salamander.hp,
        salamander.max_hp(w.content()),
        "immune to fire"
    );
}

#[test]
fn walls_and_the_shared_cone_bound_the_breath() {
    // Behind a wall the wyrm sees no one: no breath through stone.
    let walled = "############\n#@...#.D...#\n#....#.....#\n############\n";
    let mut w = world_with(walled);
    run(&mut w, "wait*3");
    assert_eq!(w.state.player().hp, full(&w));
    assert_eq!(spent(&w), 0);
}

#[test]
fn only_a_dragon_has_a_breath() {
    let refused = |breath: &str, combat: &str| {
        let files = files(breath, combat, HALL);
        Content::load(&files)
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default()
    };
    let melee = refused(FIRE, "melee");
    assert!(
        melee.contains("only a `combat = \"dragon\"` creature has a breath"),
        "{melee}"
    );
    let element = FIRE.replace("damage_type = \"fire\"", "damage_type = \"frost\"");
    assert!(refused(&element, "dragon").contains("unknown damage type `frost`"));
    // A dragon without the table loads: it has no breath to use.
    let mut files = files(FIRE, "dragon", HALL);
    let bare =
        (CREATURES.replace("[creature.breath]\n{breath}\n", "")).replace("{combat}", "dragon");
    files.insert("data/creatures.toml", &bare);
    Content::load(&files).expect("a dragon without a breath");
}

#[test]
fn the_cooldown_is_saved_and_the_breath_is_deterministic() {
    let mut a = world_with(HALL);
    let mut b = world_with(HALL);
    run(&mut a, "wait*4");
    run(&mut b, "wait*4");
    assert_eq!(a.state_hash(), b.state_hash());
    let saved = serde_json::to_string(&a.state).expect("save");
    let back: WorldState = serde_json::from_str(&saved).expect("load");
    assert_eq!(back.actors, a.state.actors, "the cooldown status kept");
}

#[test]
fn the_fire_drake_is_a_dragon_with_a_cone_of_fire_and_claws_and_a_bite() {
    let content = Content::builtin().expect("content");
    let drake = content.creature(content.creature_id("fire_drake").expect("the drake"));
    assert_eq!(drake.combat, crate::content::Combat::Dragon);
    let breath = drake.breath.as_ref().expect("a breath");
    assert_eq!(content.damage_type(breath.kind).id, "fire");
    assert!(breath.length >= 8, "a long cone");
    let names: Vec<&str> = drake.natural_attacks().map(|a| a.name.as_str()).collect();
    assert_eq!(names, ["claw", "claw", "bite"]);
    assert!(!drake.tags.iter().any(|t| t == "caster"));
}
