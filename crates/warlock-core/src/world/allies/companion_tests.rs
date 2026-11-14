//! The animal companion (`companion.rs`; spec §8 "Allies"): the starting beast by race and
//! the rares, the name, the story's chapter, growth with the master's level, always coming
//! along (and the retinue), the nose through a closed door, the words, saves, and replays.

use super::*;
use crate::action::{Action, parse_actions};
use crate::character::CharacterSpec;
use crate::combat::Category;
use crate::content::{Content, ContentFiles, Stance};
use crate::geom::Dir;
use crate::replay::Replay;
use crate::test_support::{CONTENT as BUILTIN, run, said};
use crate::world::{Order, stance_between};
use std::sync::Arc;

fn spec(text: &str) -> CharacterSpec {
    text.parse().expect("a character")
}

/// A kitted run of `spec` on the Ascmearc's start, as New run makes one.
fn kitted(spec: &str, seed: u64) -> World {
    // A human's three creation picks (Versatile adds one), which no test here reads.
    let picks = vec!["keen_ears".into(), "lean".into(), "healthy".into()];
    let spec = self::spec(spec).with_talents(picks);
    World::start_run(
        BUILTIN.clone(),
        "ascmearc",
        Some(seed),
        Some(&spec),
        None,
        true,
    )
    .expect("world")
}

fn companion_of(w: &World) -> (&Actor, &CompanionData) {
    let id = w.companion().expect("a companion");
    let actor = &w.state.actors[&id];
    (actor, actor.companion().expect("companion data"))
}

// ---- the fixture: a lit hall with a door, a dark hall, and the floor below --------------------

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
id = "door_closed"
name = "closed door"
glyph = "+"
fg = "#c08040"
walkable = false
transparent = false
opens_to = "door_open"

[[terrain]]
id = "door_open"
name = "open door"
glyph = "'"
fg = "#c08040"
closes_to = "door_closed"

[[terrain]]
id = "stairs_down"
name = "staircase down"
glyph = ">"
fg = "#ffffff"
stairs = "down"

[[terrain]]
id = "stairs_up"
name = "staircase up"
glyph = "<"
fg = "#ffffff"
stairs = "up"
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
animal = true
"##;

/// A hound with a nose and a lynx without, both strong and sure; a kobold of one hit point that
/// never opens a door.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "kobold"
name = "kobold"
glyph = "k"
fg = "#c08040"
weight = 30000
faction = "kobolds"
stats = { strength = 3, dexterity = 3, toughness = 3 }
base_hp = 1
level = 2
xp = 20
attack = { damage = "1d2", name = "claw", verb = "claws" }

[[creature]]
id = "hound"
name = "hound"
glyph = "d"
fg = "#a0a0a0"
weight = 25000
faction = "wildlife"
stats = { strength = 30, dexterity = 30 }
base_hp = 12
level = 1
xp = 15
speed = 1000
tameable = true
nose = 6
attack = { damage = "1d4", name = "bite", verb = "bites" }

[[creature]]
id = "lynx"
name = "lynx"
plural = "lynxes"
glyph = "f"
fg = "#c0a060"
weight = 20000
faction = "wildlife"
stats = { strength = 30, dexterity = 30 }
base_hp = 12
level = 2
xp = 20
speed = 1000
tameable = true
attack = { damage = "1d6", name = "claws", verb = "claws" }
"##;

/// A lit hall parted by a wall with a closed door in it, six cells east of where an ally is
/// made, a kobold just behind the door, stairs down at the far end.
const DOOR: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
####################
#@.....+k..........#
#......#...........#
#......#..........>#
####################
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
">" = { terrain = "stairs_down", to = "below" }
"##;

/// A lit hall with nothing in it but the stairs.
const EMPTY: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
####################
#@.................#
#..................#
#.................>#
####################
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
">" = { terrain = "stairs_down", to = "below" }
"##;

const BELOW: &str = r##"
name = "Below"
ambient = [255, 255, 255, 0]
map = '''
##########
#<.......#
#........#
#........#
##########
'''
[legend]
"<" = { terrain = "stairs_up", to = "hall" }
"##;

const ALLIES: &str = include_str!("../../../../assets/data/allies.toml");
const EXPERIENCE: &str = include_str!("../../../../assets/data/experience.toml");

fn fixture(hall: &str) -> World {
    let mut files = ContentFiles::default();
    files.insert(
        "data/item_classes.toml",
        crate::content::BUILTIN_ITEM_CLASSES,
    );
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/factions.toml", FACTIONS),
        ("data/creatures.toml", CREATURES),
        ("data/experience.toml", EXPERIENCE),
        ("data/allies.toml", ALLIES),
        (
            "data/legend.toml",
            "[legend]\n\".\" = \"floor\"\n\"#\" = \"wall\"\n\"+\" = \"door_closed\"\n",
        ),
        ("levels/hall.toml", hall),
        ("levels/below.toml", BELOW),
        (
            "scenarios/hall.toml",
            "name = \"Hall\"\nstart_level = \"hall\"\nhp = 100\nstats = { strength = 30, dexterity = 30 }\n",
        ),
    ] {
        files.insert(path, text);
    }
    let content = Arc::new(Content::load(&files).expect("valid content"));
    World::new(content, "hall", None).expect("world")
}

/// The dev `ally:` makes a summon; the fixture has no classes to bring a companion, so the one
/// ally of type `kind` is made the player's companion, named `name`, as Tame will (C2).
fn adopt(w: &mut World, kind: &str, name: &str) -> ActorId {
    let kind = w.content().creature_id(kind).expect("the type");
    let id = (w.state.actors.values())
        .find(|a| a.creature == kind && a.master() == Some(w.state.player))
        .map(|a| a.id)
        .expect("an ally of that type");
    let ally = w.state.actors.get_mut(&id).unwrap().ally.as_mut().unwrap();
    ally.source = AllySource::Companion(CompanionData {
        quarry: None,
        name: name.into(),
        grade: 1,
        scent: None,
    });
    id
}

fn allies(w: &World) -> Vec<&Actor> {
    (w.state.actors.values())
        .filter(|a| a.master() == Some(w.state.player))
        .collect()
}

// ---- the draw ------------------------------------------------------------------------------

#[test]
fn every_race_draws_from_its_list_or_the_rares_at_the_rare_odds() {
    let content = &*BUILTIN;
    let rules = content.companions().expect("companions.toml");
    assert_eq!(rules.start.rare_per_mille, 50);
    assert_eq!(rules.start.rare.len(), 4);
    let mut draws = 0u32;
    let mut rares = 0u32;
    for race in &content.races().list {
        let list = rules
            .start
            .list_of(&race.id)
            .expect("every shipped race has a list");
        assert!((1..=3).contains(&list.len()), "{}", race.id);
        let spec = spec(&format!("{},male,ranger", race.id));
        for seed in 0..400 {
            let beast = super::draw(content, seed, &spec).expect("a beast");
            assert!(content.creature(beast).tameable, "{}", race.id);
            if rules.start.rare.contains(&beast) && !list.contains(&beast) {
                rares += 1;
            } else {
                assert!(
                    list.contains(&beast),
                    "{}: {}",
                    race.id,
                    content.creature(beast).id
                );
            }
            draws += 1;
        }
    }
    // 5 % of the draws, give or take: the wolf, black hound, great boar, or panther.
    let expected = f64::from(draws) * 0.05;
    assert!(
        (f64::from(rares) - expected).abs() < expected / 4.0,
        "{rares} rares of {draws}, about {expected} expected"
    );
    // The same seed, the same beast (and a different class, a different stream).
    let human = spec("human,male,ranger");
    assert_eq!(
        super::draw(content, 7, &human),
        super::draw(content, 7, &human)
    );
    assert_eq!(
        super::name_for(content, 7, ActorId(1)),
        super::name_for(content, 7, ActorId(1))
    );
    assert!(
        content
            .companions()
            .unwrap()
            .names
            .contains(&super::name_for(content, 7, ActorId(1)))
    );
}

// ---- the start -----------------------------------------------------------------------------

#[test]
fn a_ranger_begins_with_a_named_beast_and_its_story_and_a_warrior_with_neither() {
    let content = &*BUILTIN;
    let rules = content.companions().unwrap();
    let w = kitted("human,male,ranger", 7);
    let (beast, data) = companion_of(&w);
    let kind = content.creature(beast.creature);
    assert!(kind.tameable);
    assert!(rules.names.contains(&data.name), "{}", data.name);
    assert_eq!(
        data.grade,
        kind.level.max(1),
        "its master's level, never below its own"
    );
    assert_eq!(beast.hp, beast.max_hp(content), "whole");
    assert!(beast.ally.as_ref().unwrap().until.is_none(), "for good");
    assert_eq!(
        beast.faction(content),
        w.state.player().faction(content),
        "its side"
    );
    let line = format!("{}, your {}, pads at your heel.", data.name, kind.name);
    assert!(said(&w).contains(&line), "{:?}", said(&w));
    // The story's chapter, with the beast's own line (made before the story is told).
    let story = &w.state.character.as_ref().unwrap().story;
    let told = (story.iter())
        .find(|t| t.chapter == "companion")
        .expect("the companion chapter");
    let chapter = (content
        .background()
        .unwrap()
        .chapter("ascmearc", "companion"))
    .unwrap();
    let line = chapter.line(&told.line).expect("a line of the chapter");
    assert!(
        line.only && line.when.companion == [beast.creature],
        "{}",
        told.line
    );
    let paragraphs = w.perception().story();
    assert!(
        paragraphs.iter().any(|p| p.contains(&line.text)),
        "{paragraphs:?}"
    );
    // The dumps.
    let layers = crate::dump::parse_layers("companion").unwrap();
    let text = crate::dump::ascii(&w, &layers);
    assert!(
        text.contains(&format!(
            "{}, {} {} (grade {})",
            data.name,
            crate::text::article(&kind.name),
            kind.name,
            data.grade
        )),
        "{text}"
    );
    let json = crate::dump::json(&w, &[], false);
    assert_eq!(json["companion"]["name"], data.name.as_str());
    assert_eq!(json["companion"]["creature"], kind.id.as_str());
    assert_eq!(json["allies"][0]["source"], "companion");
    assert!(w.perception().companion().is_some());
    // A warrior of the same seed: no beast, no chapter, nothing said of one.
    let w = kitted("human,male,warrior", 7);
    assert!(w.companion().is_none());
    assert!(w.perception().companion().is_none());
    let story = &w.state.character.as_ref().unwrap().story;
    assert!(story.iter().all(|t| t.chapter != "companion"));
    assert!(said(&w).iter().all(|m| !m.contains("pads at your heel")));
    let text = crate::dump::ascii(&w, &layers);
    assert!(text.contains("no companion"), "{text}");
    assert_eq!(
        crate::dump::json(&w, &[], false)["companion"],
        serde_json::Value::Null
    );
}

#[test]
fn the_scenarios_rangers_have_their_beasts_and_a_restored_run_makes_none() {
    // Maren in the forest glade: her beast stands beside her.
    let w = World::new(BUILTIN.clone(), "forest", None).expect("forest");
    let (beast, _) = companion_of(&w);
    assert!(!beast.in_retinue());
    assert!(beast.pos.chebyshev(w.state.player().pos) <= 2);
    // Osric on the overworld: his rides in the retinue, never drawn.
    let w = World::new(BUILTIN.clone(), "world", None).expect("world");
    let (beast, _) = companion_of(&w);
    assert!(beast.in_retinue());
    assert_eq!(w.perception().allies_with(), 1);
    assert!(
        w.perception()
            .actors()
            .iter()
            .all(|a| a.id == w.state.player)
    );
    let layers = crate::dump::parse_layers("companion").unwrap();
    assert!(crate::dump::ascii(&w, &layers).contains(": with you"));
    // Restored, the run keeps the one it has and makes no second.
    let again = World::from_state(BUILTIN.clone(), w.state().clone()).expect("restored");
    assert_eq!(again.allies_of(again.state.player).len(), 1);
    assert_eq!(again.state_hash(), w.state_hash());
    // Tamsin the thief has none.
    let w = World::new(BUILTIN.clone(), "showcase", None).expect("showcase");
    assert!(w.companion().is_none());
}

// ---- growth --------------------------------------------------------------------------------

#[test]
fn the_companion_grows_with_its_masters_level() {
    let content = &*BUILTIN;
    let growth = content.companions().unwrap().growth;
    let mut w = kitted("human,male,ranger", 7);
    let (beast, data) = companion_of(&w);
    let own = content.creature(beast.creature).level;
    let (grade0, hp0) = (data.grade, beast.max_hp(content));
    // Level 4: the grade follows, the hit points with it, and the news says so.
    run(&mut w, "xp:2000 levelup:auto");
    assert_eq!(w.player_level(), 4);
    let (beast, data) = companion_of(&w);
    assert_eq!(data.grade, 4.max(own));
    let above = i32::from(data.grade - own);
    assert_eq!(
        beast.max_hp(content),
        hp0 + growth.hp_per_level * (above - i32::from(grade0 - own))
    );
    assert_eq!(
        beast.hp,
        beast.max_hp(content),
        "the current hit points rise with the maximum"
    );
    let line = format!("{} grows stronger.", data.name);
    assert!(said(&w).contains(&line), "{:?}", said(&w));
    // Far up the curve: every hook reads the grade above the type's level, and the grade never
    // falls.
    run(&mut w, "xp:1000000 levelup:auto");
    let level = w.player_level();
    assert!(level >= 30, "{level}");
    let (beast, data) = companion_of(&w);
    assert_eq!(data.grade, level.max(own));
    let above = i32::from(data.grade - own);
    let mut plain = beast.clone();
    plain.ally.as_mut().unwrap().source = AllySource::Summoned { spell: None };
    assert_eq!(beast.grade_above(content), above);
    assert_eq!(plain.grade_above(content), 0);
    let stat_up = above / growth.stat_every;
    for stat in [Stat::Strength, Stat::Dexterity, Stat::Toughness] {
        assert_eq!(
            beast.effective_stats(content).score(stat),
            plain.effective_stats(content).score(stat) + u8::try_from(stat_up).unwrap(),
            "{stat:?}"
        );
    }
    assert_eq!(
        beast.max_hp(content) - plain.max_hp(content),
        growth.hp_per_level * above + stat_up,
        "hit points a level, and Toughness"
    );
    let (grown, base) = (beast.melee(content), plain.melee(content));
    let str_mod = beast.effective_stats(content).modifier(Stat::Strength)
        - plain.effective_stats(content).modifier(Stat::Strength);
    assert_eq!(
        grown.to_hit - base.to_hit,
        above / growth.to_hit_every + str_mod
    );
    assert_eq!(
        grown.bonus - base.bonus,
        above / growth.damage_bonus_every + str_mod
    );
    assert_eq!(
        grown.damage.groups()[0].count() - base.damage.groups()[0].count(),
        u32::try_from(above / growth.damage_dice_every).unwrap()
    );
    let (gd, bd) = (w.defence_of(beast.id), {
        let burden = w.state.load(content, &plain).penalty();
        plain.defence(content, burden, None)
    });
    assert_eq!(
        gd.bonus(Category::Natural) - bd.bonus(Category::Natural),
        above / growth.defence_every
    );
    let hide = content.creature(beast.creature).natural.as_ref();
    let body = super::body_protection(hide.map(|h| h.name.as_str()), data.grade);
    let own_protection = hide.map_or(0, |h| h.protection);
    assert_eq!(
        w.protection_of(beast.id).against(None)
            - plain.protection_plus(content, None, 0).against(None),
        body.max(own_protection) - own_protection
    );
}

#[test]
fn the_body_rule_at_a_grade() {
    assert_eq!(super::body_protection(None, 1), 0);
    assert_eq!(super::body_protection(None, 30), 2);
    assert_eq!(super::body_protection(Some("thick hide"), 60), 6);
    assert_eq!(super::body_protection(Some("scales"), 30), 5);
    assert_eq!(super::body_protection(Some("shell"), 12), 2);
    assert_eq!(super::body_protection(Some("stony hide"), 20), 6);
    assert_eq!(super::body_protection(Some("flickering flame"), 60), 0);
}

// ---- following -----------------------------------------------------------------------------

#[test]
fn a_companion_comes_along_from_anywhere_and_a_summon_is_left_behind() {
    let mut w = fixture(EMPTY);
    run(&mut w, "ally:hound ally:lynx order:stay");
    let snel = adopt(&mut w, "hound", "Snel");
    // Both hold at the west end while the player walks to the stairs at the east end.
    run(&mut w, "move:E*17 move:S*2");
    let me = w.state.player().pos;
    assert!(allies(&w).iter().all(|a| a.pos.chebyshev(me) > 3));
    run(&mut w, "descend");
    assert_eq!(w.state.current_level, "below");
    let beast = &w.state.actors[&snel];
    assert_eq!(beast.level, "below");
    assert!(beast.pos.chebyshev(w.state.player().pos) <= 2);
    assert_eq!(beast.ally.as_ref().unwrap().order, Order::Follow);
    let lynx = (allies(&w).into_iter())
        .find(|a| a.id != snel)
        .expect("the lynx");
    assert_eq!(lynx.level, "hall", "a summon far off stays");
    let said = said(&w);
    assert!(
        said.contains(&"Your lynx is left behind.".to_string()),
        "{said:?}"
    );
    assert!(
        said.contains(&"Snel comes with you.".to_string()),
        "{said:?}"
    );
    assert_eq!(w.companion(), Some(snel), "always at hand");
}

#[test]
fn the_retinue_carries_the_companion_across_the_overworld() {
    let mut w = World::new(BUILTIN.clone(), "world", None).expect("world");
    let (beast, data) = companion_of(&w);
    let (id, name) = (beast.id, data.name.clone());
    assert!(beast.in_retinue());
    let hall = (w.state.current().stairs.iter())
        .find(|s| s.to.as_deref() == Some("hub"))
        .expect("the Hall")
        .pos;
    let mut actions = Vec::new();
    for _ in 0..3 {
        let me = w.state.player().pos;
        if me == hall {
            break;
        }
        let dir = Dir::from_delta((hall.x - me.x).signum(), (hall.y - me.y).signum());
        let action = Action::Move(dir.expect("a way"));
        w.apply(&action);
        actions.push(action);
    }
    assert_eq!(w.state.player().pos, hall);
    run(&mut w, "descend");
    assert_eq!(w.state.current_level, "hub");
    let beast = &w.state.actors[&id];
    assert!(!beast.in_retinue() && beast.level == "hub");
    assert!(beast.pos.chebyshev(w.state.player().pos) <= 2);
    assert!(
        said(&w).contains(&format!("{name} follows you in.")),
        "{:?}",
        said(&w)
    );
    let layers = crate::dump::parse_layers("companion").unwrap();
    let text = crate::dump::ascii(&w, &layers);
    assert!(
        text.contains(&format!("{name}, ")) && text.contains(": unhurt, order follow"),
        "{text}"
    );
    run(&mut w, "ascend");
    assert!(w.state.actors[&id].in_retinue());
    // The same road, replayed, holds.
    let mut again = World::new(BUILTIN.clone(), "world", None).expect("world");
    let mut replay = Replay::new(&again);
    for action in actions
        .into_iter()
        .chain(parse_actions("descend ascend").unwrap())
    {
        again.apply(&action);
        replay.push(&action, &again);
    }
    assert_eq!(again.state_hash(), w.state_hash());
    let replay = Replay::from_json(&replay.to_json()).expect("parses");
    let (played, divergence) = replay.play(BUILTIN.clone()).expect("plays");
    assert!(divergence.is_none(), "{divergence:?}");
    assert_eq!(played.state_hash(), w.state_hash());
}

// ---- the nose ------------------------------------------------------------------------------

/// A hostile of the player's, as `ally_plan`'s own predicate asks.
fn hostile(s: &WorldState, c: &Content, _: &Actor, a: &Actor) -> bool {
    stance_between(s, c, s.player(), a) == Stance::Hostile
}

#[test]
fn the_nose_finds_a_hostile_through_a_closed_door_that_the_eyes_do_not() {
    let mut w = fixture(DOOR);
    run(&mut w, "ally:hound ally:lynx");
    let kobold = (w.state.actors.values())
        .find(|a| w.content().creature(a.creature).id == "kobold")
        .map(|a| (a.id, a.pos))
        .expect("the kobold");
    let hound = adopt(&mut w, "hound", "Snel");
    let lynx = (allies(&w).into_iter()).find(|a| a.id != hound).unwrap().id;
    // The door stops every eye; the hound's nose reaches six cells, the lynx has none.
    assert_eq!(w.nearest_in_sight(hound, &hostile), None);
    assert_eq!(w.smelled(hound, &hostile), Some(kobold));
    assert_eq!(w.smelled(lynx, &hostile), None);
    // It takes what it smells for its target and growls, once (the door it cannot open keeps it
    // from the kobold: it waits for the ranger to open it); the lynx heels and waits.
    run(&mut w, "wait*3");
    assert_eq!(
        w.state.actors[&hound].companion().unwrap().scent,
        Some(kobold.0)
    );
    let growls = (said(&w).iter())
        .filter(|m| *m == "Snel growls at the dark.")
        .count();
    assert_eq!(growls, 1, "{:?}", said(&w));
    assert!(w.state.actors[&lynx].pos.chebyshev(w.state.player().pos) <= 2);
}

#[test]
fn a_hostile_beyond_the_nose_is_not_smelled() {
    // The kobold ten cells from where the hound stands: farther than its nose reaches.
    const FAR: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
####################
#@.....+....k......#
#......#...........#
#......#..........>#
####################
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
">" = { terrain = "stairs_down", to = "below" }
"##;
    let mut w = fixture(FAR);
    run(&mut w, "ally:hound");
    let hound = allies(&w)[0].id;
    assert_eq!(w.smelled(hound, &hostile), None);
    run(&mut w, "wait*3");
    assert!(allies(&w)[0].pos.chebyshev(w.state.player().pos) <= 2);
    assert!(said(&w).iter().all(|m| !m.contains("growls")));
}

// ---- the words -----------------------------------------------------------------------------

#[test]
fn the_words_say_the_companions_name() {
    let mut w = fixture(DOOR);
    run(&mut w, "ally:hound ally:lynx");
    adopt(&mut w, "hound", "Snel");
    run(&mut w, "order:follow");
    assert_eq!(said(&w).last().unwrap(), "Snel and your lynx come to heel.");
    run(&mut w, "order:stay");
    assert_eq!(said(&w).last().unwrap(), "Snel and your lynx hold.");
    run(&mut w, "order:hunt:3,2");
    assert_eq!(
        said(&w).last().unwrap(),
        "You set Snel and your lynx on that spot."
    );
    // Alone with the player (no lynx to get there first), Snel kills the kobold in its own name
    // once the player opens the door.
    let mut w = fixture(DOOR);
    run(&mut w, "ally:hound");
    adopt(&mut w, "hound", "Snel");
    run(&mut w, "move:E*5 open:E wait*8");
    let said = said(&w);
    assert!(
        said.iter().any(|m| m.starts_with("Snel bites the kobold")),
        "{said:?}"
    );
    assert!(said.contains(&"The kobold dies.".to_string()), "{said:?}");
    // What the player knows of it: the seen actor is marked the companion, the dumps name it.
    let view = w.perception();
    let seen = (view.actors().iter())
        .find(|a| a.companion)
        .expect("Snel in view");
    assert!(seen.ally);
    assert_eq!(view.companion().map(|(_, d)| d.name.as_str()), Some("Snel"));
    let json = crate::dump::json(&w, &[], false);
    assert_eq!(json["companion"]["name"], "Snel");
    assert_eq!(
        json["companion"]["pos"],
        serde_json::to_value(seen.pos).unwrap()
    );
    let layers = crate::dump::parse_layers("actors").unwrap();
    let text = crate::dump::ascii(&w, &layers);
    assert!(
        text.contains("(companion; follow; named Snel, grade 1)"),
        "{text}"
    );
}

// ---- saves ---------------------------------------------------------------------------------

#[test]
fn a_save_round_trip_keeps_the_companion() {
    let mut w = kitted("human,female,ranger", 3);
    run(&mut w, "wait*3 xp:1000 levelup:auto wait");
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, BUILTIN.clone()).expect("loads");
    assert_eq!(loaded.state_hash(), w.state_hash());
    let (before, data) = companion_of(&w);
    let (after, loaded_data) = companion_of(&loaded);
    assert_eq!(before.id, after.id);
    assert_eq!(data, loaded_data);
    assert!(data.grade >= 3);
}
