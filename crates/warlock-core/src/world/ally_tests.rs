//! Allies (spec §8 "Allies", `ally.rs`): the side, the mind, the three orders, following across
//! levels and the retinue, the clock, experience, Beast-bond, saves, and replays.

use super::*;
use crate::action::{Command, parse_actions};
use crate::content::{ContentFiles, Stance};
use crate::noise::SoundKind;
use crate::replay::Replay;
use crate::test_support::builtin;

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

/// A wolf that hits hard and surely, a kobold of one hit point, and a deer nobody fights.
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
hearing = 15
base_hp = 1
level = 2
xp = 20
attack = { damage = "1d2", name = "claw", verb = "claws" }

[[creature]]
id = "wolf"
name = "wolf"
plural = "wolves"
glyph = "d"
fg = "#a0a0a0"
weight = 40000
faction = "wildlife"
stats = { strength = 30, dexterity = 30 }
base_hp = 16
level = 4
xp = 30
speed = 1000
attack = { damage = "1d6", name = "bite", verb = "bites" }

[[creature]]
id = "deer"
name = "deer"
plural = "deer"
glyph = "q"
fg = "#c0a060"
weight = 60000
faction = "wildlife"
base_hp = 10
level = 1
xp = 15
"##;

/// A lit hall: the player at the west end, stairs down at its east end.
const HALL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
####################
#@.................#
#..................#
#..................#
#.................>#
####################
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
">" = { terrain = "stairs_down", to = "below" }
"##;

/// The hall with a kobold and a deer a few cells east of the player.
const FOES: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
####################
#@.....k...........#
#..................#
#......q...........#
#.................>#
####################
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
"q" = { terrain = "floor", creature = "deer" }
">" = { terrain = "stairs_down", to = "below" }
"##;

/// The hall with a kobold three cells from the player, within a shout's reach: it hears.
const LISTENER: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
####################
#@..k..............#
#..................#
#..................#
#.................>#
####################
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
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

fn files(hall: &str, allies: Option<&str>) -> ContentFiles {
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
        (
            "data/legend.toml",
            "[legend]\n\".\" = \"floor\"\n\"#\" = \"wall\"\n",
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
    if let Some(allies) = allies {
        files.insert("data/allies.toml", allies);
    }
    files
}

fn world_of(hall: &str, allies: Option<&str>) -> World {
    let content = Arc::new(Content::load(&files(hall, allies)).expect("valid content"));
    World::new(content, "hall", None).expect("world")
}

fn world() -> World {
    world_of(HALL, Some(ALLIES))
}

fn run(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn said(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

fn last_said(w: &World) -> String {
    said(w).last().cloned().unwrap_or_default()
}

fn allies(w: &World) -> Vec<&Actor> {
    (w.state.actors.values())
        .filter(|a| a.master() == Some(w.state.player))
        .collect()
}

fn first_of<'w>(w: &'w World, kind: &str) -> Option<&'w Actor> {
    let id = w.content().creature_id(kind).expect("a type");
    (w.state.actors.values()).find(|a| a.creature == id && a.ally.is_none())
}

#[test]
fn the_dev_action_makes_followers_on_the_players_side() {
    let mut w = world();
    let (turn, time) = (w.turn(), w.time());
    run(&mut w, "ally:wolf:2");
    let pack = allies(&w);
    assert_eq!(pack.len(), 2);
    let player = w.state.player;
    let me = w.state.player().pos;
    for a in &pack {
        let ally = a.ally.as_ref().expect("an ally");
        assert_eq!(ally.master, player);
        assert_eq!(ally.order, Order::Follow);
        assert_eq!(ally.source, AllySource::Summoned { spell: None });
        assert!(!ally.retinue && ally.until.is_none());
        assert_eq!(
            a.faction(w.content()),
            w.state.player().faction(w.content())
        );
        assert!(a.pos.chebyshev(me) <= 2, "{:?} beside {me:?}", a.pos);
        assert!(a.inventory.is_empty(), "a beast carries nothing");
        // Its side is its master's: nobody is hostile to it, and it to nobody (the spawn cap,
        // the quick leave, and coarse turns all read this).
        assert_eq!(w.stance(player, a.id), Stance::Neutral);
    }
    assert_eq!((w.turn(), w.time()), (turn, time), "no time");
    assert_eq!(last_said(&w), "Two wolves come to your side.");
    let header = crate::dump::ascii(&w, &[]);
    assert!(header.contains(" allies 2 "), "{header}");
}

#[test]
fn the_dev_action_is_refused_where_it_cannot_make_one() {
    let mut w = world_of(HALL, None);
    run(&mut w, "ally:wolf");
    assert!(allies(&w).is_empty());
    assert_eq!(last_said(&w), "This world knows no allies.");
    let mut w = world();
    run(&mut w, "ally:dragon");
    assert_eq!(last_said(&w), "There is no creature `dragon`.");
    run(&mut w, "ally:player");
    assert_eq!(last_said(&w), "There is no creature `player`.");
    // The cap: eight at most on one level (from the middle of the room, where there is room).
    run(&mut w, "move:E*8 move:S*2 ally:wolf:8 ally:wolf");
    assert_eq!(allies(&w).len(), 8);
    assert_eq!(last_said(&w), "You cannot lead more than this.");
}

#[test]
fn an_ally_follows_within_heel_of_its_master() {
    let mut w = world();
    run(&mut w, "ally:wolf move:E*12");
    let me = w.state.player().pos;
    let wolf = allies(&w)[0];
    assert!(
        wolf.pos.chebyshev(me) <= 2,
        "{:?} heels at {me:?}",
        wolf.pos
    );
    // Waiting beside its master, it stays put.
    let was = wolf.pos;
    run(&mut w, "wait*3");
    assert_eq!(allies(&w)[0].pos, was);
}

#[test]
fn an_ally_attacks_what_its_master_is_hostile_to_and_spares_the_rest() {
    let mut w = world_of(FOES, Some(ALLIES));
    let deer_hp = first_of(&w, "deer").expect("a deer").hp;
    run(&mut w, "ally:wolf wait*12");
    assert!(first_of(&w, "kobold").is_none(), "the kobold is slain");
    let deer = first_of(&w, "deer").expect("the deer lives");
    assert_eq!(
        deer.hp, deer_hp,
        "the deer, neutral to the master, is untouched"
    );
    assert!(
        said(&w).iter().any(|m| m.contains("wolf bites the kobold")),
        "{:?}",
        said(&w)
    );
}

#[test]
fn an_allys_kill_pays_its_master_as_their_own_blow_would() {
    // The player's own kill: the kobold walks up and is struck down.
    let mut own = world_of(FOES, Some(ALLIES));
    let kobold = first_of(&own, "kobold").expect("a kobold").id;
    for _ in 0..40 {
        let at = own.state.actors.get(&kobold).map(|a| a.pos);
        let Some(at) = at else { break };
        let me = own.state.player().pos;
        if at.chebyshev(me) <= 1 {
            let dir = Dir::from_delta((at.x - me.x).signum(), (at.y - me.y).signum());
            own.apply(&Action::Move(dir.expect("a direction")));
        } else {
            own.apply(&Action::Wait);
        }
    }
    assert!(first_of(&own, "kobold").is_none(), "the player killed it");
    let paid = own.state.experience;
    assert!(paid > 0);
    // At full pay the wolf's kill pays the same, tallies the quest log, and marks nothing of
    // the wolf's.
    let full = ALLIES.replace("ally_percent = 50 ", "ally_percent = 100 ");
    assert_ne!(full, ALLIES, "the shipped share is half");
    let mut w = world_of(FOES, Some(&full));
    run(&mut w, "ally:wolf wait*12");
    assert!(first_of(&w, "kobold").is_none());
    assert_eq!(w.state.experience, paid);
    assert_eq!(w.state.quests.killed("kobold"), 1);
    assert_eq!(own.state.quests.killed("kobold"), 1);
    // At the shipped half pay (a run without a class), half; the quest tally in full.
    let mut w = world_of(FOES, Some(ALLIES));
    run(&mut w, "ally:wolf wait*12");
    assert!(first_of(&w, "kobold").is_none());
    assert_eq!(w.state.experience, paid / 2);
    assert_eq!(w.state.quests.killed("kobold"), 1);
}

/// A run of the built-in content in the hall with the kobold and the deer, as `character`.
fn built_in_as(character: &str) -> World {
    let mut files = ContentFiles::builtin();
    files.insert("levels/hall.toml", FOES);
    files.insert("levels/below.toml", BELOW);
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 1000\n",
    );
    let content = Arc::new(Content::load(&files).expect("valid content"));
    let spec = character.parse().expect("a character");
    World::with_character(content, "hall", None, Some(&spec)).expect("world")
}

#[test]
fn the_pet_classes_take_an_allys_kill_in_full_and_everyone_else_half() {
    // The share by class (design ranger-companion.md §9): the ranger's and the druid's 100,
    // everyone else's `[xp] ally_percent`, 50.
    for (character, share) in [
        ("human,female,druid", 100),
        ("human,female,ranger", 100),
        ("human,female,commoner", 50),
        ("human,female,wizard", 50),
    ] {
        assert_eq!(built_in_as(character).ally_share(), share, "{character}");
    }
    // A wolf's kill: the druid is paid in full, the commoner half; both tally it in full.
    let mut paid = Vec::new();
    for character in ["human,female,druid", "human,female,commoner"] {
        let mut w = built_in_as(character);
        let kobold = first_of(&w, "kobold").expect("a kobold").id;
        run(&mut w, "ally:wolf");
        for _ in 0..60 {
            if !w.state.actors.contains_key(&kobold) {
                break;
            }
            run(&mut w, "wait");
        }
        assert!(
            !w.state.actors.contains_key(&kobold),
            "{character}: the wolf kills it"
        );
        assert_eq!(w.state.quests.killed("kobold"), 1, "{character}");
        paid.push(w.state.experience);
    }
    assert!(paid[0] > 0);
    // Half, give or take the rounding of what talents add to experience after it.
    assert!(
        paid[1] < paid[0] && (paid[0] / 2).abs_diff(paid[1]) <= 1,
        "{paid:?}"
    );
}

#[test]
fn orders_take_no_time_make_a_voice_and_are_refused_without_allies() {
    let mut w = world_of(LISTENER, Some(ALLIES));
    run(&mut w, "order:follow");
    assert_eq!(last_said(&w), "You have no one to call.");
    let kobold = first_of(&w, "kobold").expect("a kobold").id;
    assert!(w.state.actors[&kobold].heard.is_empty(), "no noise");
    run(&mut w, "ally:wolf:2");
    let (turn, time) = (w.turn(), w.time());
    run(&mut w, "order:stay");
    assert_eq!((w.turn(), w.time()), (turn, time), "an order takes no time");
    assert_eq!(last_said(&w), "Your wolves hold.");
    let me = w.state.player().pos;
    for a in allies(&w) {
        assert_eq!(a.ally.as_ref().unwrap().order, Order::Stay { at: a.pos });
    }
    let heard = &w.state.actors[&kobold].heard;
    assert!(
        heard
            .iter()
            .any(|h| h.kind == SoundKind::Voice && h.at == me),
        "the kobold hears the shout where the player stands: {heard:?}"
    );
    run(&mut w, "order:hunt:10,3");
    assert_eq!(last_said(&w), "You set your wolves on that spot.");
    for a in allies(&w) {
        assert_eq!(
            a.ally.as_ref().unwrap().order,
            Order::Hunt {
                at: Pos::new(10, 3)
            }
        );
    }
    run(&mut w, "order:follow");
    assert_eq!(last_said(&w), "Your wolves come to heel.");
    assert!(
        allies(&w)
            .iter()
            .all(|a| a.ally.as_ref().unwrap().order == Order::Follow)
    );
    // Set on wants a place the player sees.
    run(&mut w, "order:hunt:-5,-5");
    assert_eq!(last_said(&w), "You can set them on only a place you see.");
}

#[test]
fn a_holding_ally_holds_and_a_set_on_ally_goes_there_and_comes_back_to_heel() {
    let mut w = world();
    run(&mut w, "ally:wolf order:stay");
    let post = allies(&w)[0].pos;
    run(&mut w, "move:E*8 wait*3");
    assert_eq!(allies(&w)[0].pos, post, "it holds its post");
    run(&mut w, "order:hunt:17,4 wait*20");
    let wolf = allies(&w)[0];
    assert_eq!(
        wolf.ally.as_ref().unwrap().order,
        Order::Follow,
        "arrived, it heels"
    );
    assert!(
        said(&w).iter().any(|m| m == "Your wolf looks back at you."),
        "{:?}",
        said(&w)
    );
    let me = w.state.player().pos;
    run(&mut w, "wait*20");
    assert!(allies(&w)[0].pos.chebyshev(me) <= 2);
}

#[test]
fn a_set_on_ally_fights_what_is_near_the_cell_it_was_sent_to() {
    let mut w = world_of(FOES, Some(ALLIES));
    let kobold = first_of(&w, "kobold").expect("a kobold").pos;
    run(&mut w, "ally:wolf");
    w.apply(&Action::Order(Command::SetOn(kobold)));
    assert_eq!(last_said(&w), "You set your wolf on the kobold.");
    run(&mut w, "wait*12");
    assert!(first_of(&w, "kobold").is_none());
}

#[test]
fn allies_beside_the_player_come_along_and_the_rest_are_left_behind() {
    let mut w = world();
    // One holds at the west end; the other follows to the stairs.
    run(&mut w, "ally:wolf order:stay ally:wolf");
    let held = (allies(&w).iter())
        .find(|a| matches!(a.ally.as_ref().unwrap().order, Order::Stay { .. }))
        .map(|a| a.id)
        .expect("the holder");
    run(&mut w, "move:E*17 move:S*3 descend");
    assert_eq!(w.state.current_level, "below");
    let came: Vec<&Actor> = allies(&w)
        .into_iter()
        .filter(|a| a.level == "below")
        .collect();
    assert_eq!(came.len(), 1);
    let me = w.state.player().pos;
    assert!(came[0].pos.chebyshev(me) <= 1, "placed beside the arrival");
    assert_eq!(came[0].ally.as_ref().unwrap().order, Order::Follow);
    assert_eq!(w.state.actors[&held].level, "hall", "the holder stays");
    let lines = said(&w);
    assert!(
        lines.iter().any(|m| m == "Your wolf comes with you."),
        "{lines:?}"
    );
    assert!(
        lines.iter().any(|m| m == "Your wolf is left behind."),
        "the holder was in view: {lines:?}"
    );
    // Back up: the follower comes again; the holder, frozen meanwhile, is where it was.
    let post = w.state.actors[&held].pos;
    run(&mut w, "ascend");
    assert_eq!(w.state.current_level, "hall");
    assert!(allies(&w).iter().all(|a| a.level == "hall"));
    assert_eq!(w.state.actors[&held].pos, post);
}

#[test]
fn a_summons_time_runs_out_by_the_clock_here_and_elsewhere() {
    let mut w = world();
    run(&mut w, "ally:wolf:1:1m");
    let until = allies(&w)[0].ally.as_ref().unwrap().until.expect("timed");
    assert_eq!(until, w.time() + Duration::minutes(1));
    run(&mut w, "wait*5");
    assert!(w.time() < until);
    assert_eq!(allies(&w).len(), 1, "half a minute in, it is still here");
    run(&mut w, "wait*10");
    assert!(w.time() >= until);
    assert!(allies(&w).is_empty(), "a minute in, it is gone");
    assert_eq!(last_said(&w), "The wolf fades away.");
    assert!(w.state.current().objects_at(Pos::new(2, 1)).is_empty());
    // Left behind, it keeps its time and is gone, silently, when the player returns.
    let mut w = world();
    run(
        &mut w,
        "ally:wolf:1:5m order:stay move:E*17 move:S*3 descend",
    );
    assert_eq!(
        allies(&w).len(),
        1,
        "two minutes' walk: it is still there, frozen"
    );
    run(&mut w, "wait*60 ascend");
    assert!(allies(&w).is_empty());
    assert!(!said(&w).iter().any(|m| m.contains("fades away")));
}

#[test]
fn allies_never_disturb_a_rest() {
    let mut w = world();
    run(&mut w, "ally:wolf:2");
    let time = w.time();
    run(&mut w, "rest:10m");
    assert_eq!(w.time(), time + Duration::minutes(10), "{:?}", said(&w));
}

#[test]
fn an_ally_survives_a_save_as_it_was() {
    let mut w = world();
    run(&mut w, "ally:wolf:1:30m ally:wolf order:stay");
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(loaded.state_hash(), w.state_hash());
    for (a, b) in allies(&w).into_iter().zip(allies(&loaded)) {
        assert_eq!(a.ally, b.ally);
    }
}

#[test]
fn beast_bond_is_read_through_the_master_with_the_shipped_talent() {
    let content = builtin();
    let mut w = World::new(content.clone(), "circles", None).expect("world");
    run(&mut w, "ally:rat");
    let rat = allies(&w)[0].id;
    let plain = w.protection_of(rat).base;
    let profile = w.state.actors[&rat].melee(&content);
    assert_eq!(w.ally_blow(rat, profile.clone()).to_hit, profile.to_hit);
    let player = w.state.player;
    (w.state.actors.get_mut(&player).unwrap().talents).take("beast_bond", None);
    assert_eq!(w.protection_of(rat).base, plain + 1);
    assert_eq!(w.ally_blow(rat, profile.clone()).to_hit, profile.to_hit + 1);
    // Nobody else's blows or hide change.
    let player_profile = w.state.player().melee(&content);
    assert_eq!(w.ally_blow(player, player_profile.clone()), player_profile);
    let book = content.talents();
    let def = book.get(book.id("beast_bond").expect("shipped"));
    let allies = book.system_id("allies").expect("the allies system");
    assert!(def.systems.contains(&allies));
}

#[test]
fn an_order_and_an_allys_kill_mark_handle_animal() {
    let content = builtin();
    let spec: crate::character::CharacterSpec = "human,male,druid".parse().expect("a druid");
    let mut w = World::start_run(
        content.clone(),
        "showcase",
        Some(3),
        Some(&spec),
        None,
        false,
    )
    .expect("world");
    let skill = content.skill_id("handle_animal").expect("the skill");
    let marks = |w: &World| w.state.player().skills.get(skill).expect("trained").marks;
    assert_eq!(marks(&w), 0);
    run(&mut w, "ally:rat");
    for _ in 0..40 {
        run(&mut w, "order:follow");
    }
    assert!(marks(&w) > 0, "forty orders leave marks");
}

#[test]
fn the_retinue_crosses_the_overworld_and_steps_out_at_the_next_level() {
    let content = builtin();
    // A commoner, not the scenario's ranger: no companion in the retinue (`companion_tests.rs`).
    let spec = crate::character::CharacterSpec {
        race: "human".into(),
        gender: crate::character::Gender::Male,
        class: "commoner".into(),
        talents: Vec::new(),
    };
    let mut w = World::with_character(content.clone(), "world", None, Some(&spec)).expect("world");
    assert_eq!(w.state.current_level, "overworld");
    // Two rats join the retinue at once; a third has a minute, which the road spends.
    run(&mut w, "ally:rat:2 ally:rat:1:1m");
    let pack = allies(&w);
    assert_eq!(pack.len(), 3);
    assert!(
        pack.iter()
            .all(|a| a.in_retinue() && a.level == "overworld")
    );
    let me = w.state.player().pos;
    assert!(pack.iter().all(|a| a.pos == me));
    // Never drawn, never in the way: the view shows none of them, and the cell is free.
    assert!(
        w.perception()
            .actors()
            .iter()
            .all(|a| a.id == w.state.player)
    );
    assert_eq!(w.perception().allies_with(), 3);
    assert!(
        w.state
            .actor_at("overworld", me)
            .is_some_and(|a| a.id == w.state.player)
    );
    let header = crate::dump::ascii(&w, &[]);
    assert!(header.contains(" allies 3 "), "{header}");
    // Into the Hall: the two step out beside the player; the third's minute ran out on the way.
    let hall = (w.state.current().stairs.iter())
        .find(|s| s.to.as_deref() == Some("hub"))
        .expect("the Hall")
        .pos;
    let before = w.time();
    for _ in 0..3 {
        let me = w.state.player().pos;
        if me == hall {
            break;
        }
        let dir = Dir::from_delta((hall.x - me.x).signum(), (hall.y - me.y).signum());
        w.apply(&Action::Move(dir.expect("a way")));
    }
    assert_eq!(w.state.player().pos, hall, "from {me:?}: {:?}", said(&w));
    assert!(
        w.time() >= before + Duration::minutes(1),
        "a step on the overworld takes minutes"
    );
    run(&mut w, "descend");
    assert_eq!(w.state.current_level, "hub");
    let pack = allies(&w);
    assert_eq!(pack.len(), 2, "the timed one did not step out");
    let me = w.state.player().pos;
    for a in &pack {
        assert!(!a.in_retinue() && a.level == "hub");
        assert!(a.pos.chebyshev(me) <= 2, "{:?} about {me:?}", a.pos);
    }
    assert!(
        said(&w)
            .iter()
            .any(|m| m == "Your giant rats follow you in."),
        "{:?}",
        said(&w)
    );
    // And back out: they join the retinue again.
    run(&mut w, "ascend");
    assert_eq!(w.state.current_level, "overworld");
    let pack = allies(&w);
    assert_eq!(pack.len(), 2);
    assert!(pack.iter().all(|a| a.in_retinue()));
    assert!(
        said(&w)
            .iter()
            .any(|m| m == "Your giant rats come with you."),
        "{:?}",
        said(&w)
    );
    // The retinue survives a save.
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, content).expect("loads");
    assert_eq!(loaded.state_hash(), w.state_hash());
    assert_eq!(allies(&loaded).len(), 2);
}

#[test]
fn the_dumps_tell_the_allies() {
    let mut w = world();
    run(&mut w, "ally:wolf:1:30m ally:wolf order:stay");
    let layers = crate::dump::parse_layers("actors").expect("layers");
    let text = crate::dump::ascii(&w, &layers);
    assert!(
        text.contains("ally of #0 (summoned, 30 min; stay at ("),
        "{text}"
    );
    assert!(text.contains("ally of #0 (summoned; stay at ("), "{text}");
    let json = crate::dump::json(&w, &[], false);
    let listed = json["allies"].as_array().expect("allies");
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0]["creature"], "wolf");
    assert_eq!(listed[0]["order"], "stay");
    assert_eq!(listed[0]["lasts"], 30);
    assert_eq!(listed[0]["here"], true);
    assert_eq!(listed[1]["lasts"], serde_json::Value::Null);
    let actors = json["seen_actors"].as_array().expect("actors");
    assert!(
        actors.iter().filter(|a| a["ally"] == true).count() == 2,
        "{actors:?}"
    );
    // The orders are abilities of the Allies group, available while an ally is on the level.
    let view = w.perception();
    let orders: Vec<_> = (view.abilities().into_iter())
        .filter(|a| a.ability.source() == crate::ability::Source::Ally)
        .collect();
    assert_eq!(orders.len(), 3);
    assert!(orders.iter().all(|a| a.unavailable.is_none()));
    // With none on the level, listed but not usable; with none at all, absent.
    run(&mut w, "order:stay move:E*17 move:S*3 descend");
    let view = w.perception();
    let orders: Vec<_> = (view.abilities().into_iter())
        .filter(|a| a.ability.source() == crate::ability::Source::Ally)
        .collect();
    assert_eq!(orders.len(), 3);
    assert!(
        orders
            .iter()
            .all(|a| a.unavailable == Some(crate::ability::Unavailable::NoAllies))
    );
    let w = world();
    assert!(
        (w.perception().abilities().into_iter())
            .all(|a| a.ability.source() != crate::ability::Source::Ally)
    );
}

#[test]
fn a_run_with_allies_replays_without_divergence() {
    let mut w = world_of(FOES, Some(ALLIES));
    let mut replay = Replay::new(&w);
    let actions = "ally:wolf:2:5m order:stay wait*3 order:hunt:7,1 wait*8 order:follow \
                   move:E*17 move:S*3 descend wait*3 ascend wait*40";
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let text = replay.to_json();
    let replay = Replay::from_json(&text).expect("parses");
    let (again, divergence) = replay.play(w.content().clone()).expect("plays");
    assert!(divergence.is_none(), "{divergence:?}");
    assert_eq!(again.state_hash(), w.state_hash());
}
