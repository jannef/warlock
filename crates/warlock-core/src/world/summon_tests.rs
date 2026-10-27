//! Summoning (spec §8 "Summoning", `summon.rs`): a summoner that sees a hostile calls forth
//! creatures of its side around itself, within its dice, its mana, its cooldown, and its cap;
//! Silence stops it; what comes is a creature like any other.

use super::*;
use crate::action::parse_actions;
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
id = "cult"
name = "cultists"
hostile = ["player"]

[[faction]]
id = "undead"
name = "undead"
hostile = ["player"]
"##;

const STATUS: &str = r##"
[[status]]
id = "summoning"
name = "Summoning spent"
max = 100
stacking = "refresh"
eases = 6

[[status]]
id = "hush"
name = "Silenced"
max = 1
eases = 600
blocks = ["cast"]
"##;

/// `{summon}` is the summoner's `[creature.summon]` body, `{mana}` its mana.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "bones"
name = "skeleton"
glyph = "s"
fg = "#e0e0d0"
weight = 20000
faction = "undead"
tags = ["undead"]
level = 3
xp = 25
speed = 100

[[creature]]
id = "lord"
name = "skeleton lord"
glyph = "S"
fg = "#e0e0d0"
weight = 20000
faction = "undead"
tags = ["undead"]
level = 9
unique = true

[[creature]]
id = "summoner"
name = "necromancer"
glyph = "N"
fg = "#7ab05a"
weight = 70000
faction = "cult"
tags = ["summoner"]
sight = 20
base_hp = 1000
mana = {mana}
[creature.summon]
{summon}
"##;

const ACTIONS: &str = "melee = 1000\ncast = 1000\nwait = 1000\n";

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
"@" = { terrain = "floor", spawn = "player" }
"N" = { terrain = "floor", creature = "summoner" }
"##;

/// A long lit hall: the player at the west end, the necromancer six cells east.
const HALL: &str = "##########\n#@.....N.#\n#........#\n#........#\n##########\n";

fn files(summon: &str, mana: u16, map: &str) -> ContentFiles {
    let mut files = ContentFiles::default();
    files.insert(
        "data/item_classes.toml",
        crate::content::BUILTIN_ITEM_CLASSES,
    );
    let creatures = CREATURES
        .replace("{summon}", summon)
        .replace("{mana}", &mana.to_string());
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/factions.toml", FACTIONS),
        ("data/status.toml", STATUS),
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

const RAISES: &str =
    "creatures = [\"bones\"]\ncount = \"1d3\"\ncooldown = 30\nmana = 8\nverb = \"raises\"";

fn world_with(summon: &str, mana: u16, map: &str) -> World {
    let content = Arc::new(Content::load(&files(summon, mana, map)).expect("valid content"));
    World::new(content, "hall", None).expect("world")
}

fn world() -> World {
    world_with(RAISES, 100, HALL)
}

fn run(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn summoner(w: &World) -> &Actor {
    first_of(w, "summoner")
}

/// The first actor of creature type `kind`.
fn first_of<'w>(w: &'w World, kind: &str) -> &'w Actor {
    let id = w.content().creature_id(kind).expect("a type");
    (w.state.actors.values())
        .find(|a| a.creature == id)
        .expect("the summoner")
}

fn summoned(w: &World) -> Vec<&Actor> {
    summoned_by(w, summoner(w).id)
}

fn summoned_by(w: &World, by: ActorId) -> Vec<&Actor> {
    (w.state.actors.values())
        .filter(|a| a.summoner == Some(by))
        .collect()
}

fn said(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

#[test]
fn a_summoner_calls_forth_its_kind_around_it_on_its_side() {
    let mut w = world();
    let mana = summoner(&w).mana;
    run(&mut w, "wait");
    let came = summoned(&w);
    assert!((1..=3).contains(&came.len()), "{} came", came.len());
    let me = summoner(&w);
    let cult = w.content().factions().id("cult");
    for a in &came {
        assert!(a.pos.chebyshev(me.pos) <= 2, "within its spread");
        assert_eq!(a.faction(w.content()), cult, "of the summoner's side");
        assert_eq!(w.content().creature(a.creature).id, "bones");
    }
    assert_eq!(me.mana, mana - 8, "it paid");
    let status = w.content().status_id("summoning").expect("status");
    // 30 s of cooldown at 6 s a degree: five, a degree of which may have eased since.
    let left = me.effects.get(status).map_or(0, |e| e.severity);
    assert!((4..=5).contains(&left), "{left} degrees");
    let said = said(&w);
    let n = came.len();
    let what = if n == 1 {
        "a skeleton".to_string()
    } else {
        format!("{n} skeletons")
    };
    let line = format!("The necromancer raises {what}!");
    assert!(said.contains(&line), "{said:?}");
}

#[test]
fn the_cooldown_holds_it_back_until_it_eases() {
    let mut w = world();
    run(&mut w, "wait");
    let first = summoned(&w).len();
    let start = w.time();
    // While the cooldown lasts, nothing more comes; once it is over, more do.
    let mut more_at = None;
    for _ in 0..200 {
        run(&mut w, "wait");
        if summoned(&w).len() > first {
            more_at = Some(w.time());
            break;
        }
    }
    let at = more_at.expect("it summoned again");
    assert!(
        at.since(start).as_seconds() >= 30 - 6,
        "not before its cooldown ran out"
    );
}

#[test]
fn without_the_mana_it_summons_nothing() {
    let mut w = world_with(RAISES, 7, HALL);
    run(&mut w, "wait*10");
    assert!(summoned(&w).is_empty());
}

#[test]
fn it_never_has_more_than_its_most_alive() {
    let summon = "creatures = [\"bones\"]\ncount = \"3\"\ncooldown = 6\nmana = 1\nmost = 4";
    let mut w = world_with(summon, 100, HALL);
    run(&mut w, "wait");
    assert_eq!(summoned(&w).len(), 3);
    run(&mut w, "wait*40");
    assert_eq!(summoned(&w).len(), 4, "capped at `most`");
}

#[test]
fn silence_stops_it() {
    let mut w = world();
    let id = summoner(&w).id;
    let hush = w.content().status_id("hush").expect("hush");
    let mut events = Vec::new();
    w.afflict(id, hush, 1, None, &mut events);
    run(&mut w, "wait*3");
    assert!(summoned(&w).is_empty());
}

#[test]
fn walls_and_crowding_leave_fewer_cells() {
    // Boxed in: the necromancer has one free cell beside it (west), the rest wall.
    let map = "##########\n#@.....N##\n#######.##\n##########\n";
    let summon = "creatures = [\"bones\"]\ncount = \"3\"\ncooldown = 30\nmana = 8";
    let mut w = world_with(summon, 100, map);
    run(&mut w, "wait");
    let came = summoned(&w);
    let me = summoner(&w).pos;
    assert!(!came.is_empty());
    for a in &came {
        assert!(a.pos.chebyshev(me) <= 2);
        assert!(w.state.current().is_walkable(w.content(), a.pos));
    }
}

#[test]
fn a_pool_by_tag_leaves_out_uniques_and_the_summoner() {
    let summon = "tag = \"undead\"\nlevels = [1, 20]\ncount = \"1\"\ncooldown = 30\nmana = 8";
    let w = world_with(summon, 100, HALL);
    let c = w.content();
    let me = c.creature_id("summoner").expect("summoner");
    let pool = &c.creature(me).summon.as_ref().expect("a summon").pool;
    assert_eq!(pool, &vec![c.creature_id("bones").expect("bones")]);
}

#[test]
fn a_summoner_without_mana_or_with_an_empty_pool_is_refused() {
    let err = Content::load(&files(RAISES, 0, HALL)).expect_err("no mana");
    assert!(err.to_string().contains("needs `mana`"), "{err}");
    let summon = "tag = \"undead\"\nlevels = [50, 60]\ncount = \"1\"\ncooldown = 30\nmana = 8";
    let err = Content::load(&files(summon, 10, HALL)).expect_err("empty pool");
    assert!(err.to_string().contains("no creature fits"), "{err}");
    let summon = "creatures = [\"nobody\"]\ncount = \"1\"\ncooldown = 30\nmana = 8";
    let err = Content::load(&files(summon, 10, HALL)).expect_err("unknown");
    assert!(
        err.to_string().contains("unknown creature `nobody`"),
        "{err}"
    );
}

#[test]
fn the_summoned_and_who_called_them_survive_a_save() {
    let mut w = world();
    run(&mut w, "wait");
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(summoned(&loaded).len(), summoned(&w).len());
    assert_eq!(loaded.state_hash(), w.state_hash());
}

#[test]
fn summoning_is_deterministic() {
    let mut a = world();
    let mut b = world();
    run(&mut a, "wait*20");
    run(&mut b, "wait*20");
    assert_eq!(a.state_hash(), b.state_hash());
}
