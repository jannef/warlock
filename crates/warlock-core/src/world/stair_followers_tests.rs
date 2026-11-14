//! Stair followers (spec §8 "Creature behavior"; `stair_followers.rs`): who comes along a
//! staircase, where it stands, what it keeps, when it may act, and what the other ways of
//! leaving a level do.

use super::fight::Harm;
use super::*;
use crate::action::Event;
use crate::content::ContentFiles;
use crate::test_support::run;
use crate::test_support::texts;

/// The player stands on stairs down and up (below): `k` beside them, `f` two cells off.
const TOP: &str = "\
#########
#.......#
#..k@.f.#
#.......#
#########
";

/// Where the stairs lead: open floor all around the stairs back up.
const OPEN: &str = "\
#########
#.......#
#...<...#
#.......#
#########
";

/// A dead end: one free cell beside the stairs back up.
const NARROW: &str = "\
#######
###<.##
#######
";

struct Fixture<'a> {
    top: &'a str,
    below: &'a str,
    /// What `k` is, in its legend: `creature = "kobold"`.
    k: &'a str,
    /// What `@` is, in its legend's terrain: `stairs_down` and where to.
    stairs: &'a str,
    /// More of the level below's file, after its map.
    below_rooms: &'a str,
}

impl Default for Fixture<'_> {
    fn default() -> Self {
        Self {
            top: TOP,
            below: OPEN,
            k: "creature = \"kobold\"",
            stairs: "terrain = \"stairs_down\", to = \"test_below\"",
            below_rooms: "",
        }
    }
}

fn level(name: &str, map: &str, rooms: &str, legend: &str) -> String {
    format!(
        "name = \"{name}\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n{rooms}\
         [legend]\n{legend}\n"
    )
}

fn world_of(f: &Fixture) -> World {
    let mut files = ContentFiles::builtin();
    let top = format!(
        "\"@\" = {{ {}, spawn = \"player\" }}\n\
         \"k\" = {{ terrain = \"floor\", {} }}\n\
         \"f\" = {{ terrain = \"floor\", creature = \"kobold\" }}\n",
        f.stairs, f.k
    );
    files.insert("levels/test_top.toml", &level("Top", f.top, "", &top));
    let below = "\"<\" = { terrain = \"stairs_up\", to = \"test_top\" }\n";
    files.insert(
        "levels/test_below.toml",
        &level("Below", f.below, f.below_rooms, below),
    );
    files.insert(
        "scenarios/test_top.toml",
        "name = \"T\"\nstart_level = \"test_top\"\nhp = 100\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_top", None).expect("world")
}

fn world() -> World {
    world_of(&Fixture::default())
}

/// The creature that started at `at`.
fn at(w: &World, x: i32, y: i32) -> ActorId {
    (w.state.actors.values())
        .find(|a| a.kind == ActorKind::Creature && a.pos == Pos::new(x, y))
        .map(|a| a.id)
        .expect("a creature there")
}

fn beside(w: &World) -> ActorId {
    at(w, 3, 2)
}

fn far(w: &World) -> ActorId {
    at(w, 6, 2)
}

/// The player was told `line`.
fn told(w: &World, line: &str) {
    let lines = texts(w);
    assert!(lines.iter().any(|m| m == line), "{lines:?}");
}

fn afflict(w: &mut World, id: ActorId, status: &str) {
    let content = w.content.clone();
    let status = content.status_id(status).expect("status");
    let actor = w.state.actors.get_mut(&id).expect("actor");
    actor.effects.add(status, content.status(status), 1, None);
}

#[test]
fn the_one_beside_follows_down_and_up_and_the_one_two_cells_off_stays() {
    let mut w = world();
    let (k, f) = (beside(&w), far(&w));
    let post = w.state.actors[&f].pos;
    run(&mut w, "descend");
    assert_eq!(w.state.current_level, "test_below");
    let me = w.state.player().pos;
    assert_eq!(w.state.actors[&k].level, "test_below");
    assert_eq!(w.state.actors[&k].pos.chebyshev(me), 1, "beside the player");
    assert_eq!(w.state.actors[&f].level, "test_top");
    assert_eq!(w.state.actors[&f].pos, post, "frozen where it was");
    told(&w, "The kobold follows you down the stairs.");
    // Back up: still beside the player, it comes again.
    run(&mut w, "ascend");
    assert_eq!(w.state.current_level, "test_top");
    assert_eq!(w.state.actors[&k].level, "test_top");
    told(&w, "The kobold follows you up the stairs.");
}

#[test]
fn a_followed_vault_monster_counts_toward_the_new_levels_cap() {
    use crate::world::spawn::Spawner;
    let mut w = world();
    let k = beside(&w);
    let spawner = Spawner::new(1, "test_top", w.state.time, Duration::seconds(60));
    let top = w.state.levels.get_mut("test_top").expect("top");
    top.spawner = Some(spawner);
    top.spawner.as_mut().expect("spawner").outside.insert(k);
    run(&mut w, "descend");
    assert_eq!(w.state.actors[&k].level, "test_below");
    assert_eq!(w.hostiles_here(), 1, "it counts on the new level");
    let top = &w.state.levels["test_top"];
    assert!(top.spawner.as_ref().expect("spawner").outside.is_empty());
}

#[test]
fn a_follower_keeps_its_hit_points_and_statuses_and_hunts_on() {
    let mut w = world();
    let k = beside(&w);
    afflict(&mut w, k, "bleeding");
    let hp = {
        let actor = w.state.actors.get_mut(&k).expect("kobold");
        actor.hp -= 1;
        actor.mind.post = Some(Pos::new(3, 2));
        actor.mind.noise = Some(Pos::new(1, 1));
        actor.patrol = Some(Patrol {
            route: vec![crate::geom::Dir::E],
            step: 0,
        });
        actor.hp
    };
    let bleeding = w.content.status_id("bleeding").expect("bleeding");
    // Taken together with the level's own turn: the bleeding may tick, never heal.
    run(&mut w, "descend");
    let actor = &w.state.actors[&k];
    assert_eq!(actor.level, "test_below");
    assert!(actor.hp <= hp, "no hit points given back");
    assert!(actor.effects.get(bleeding).is_some(), "it still bleeds");
    assert_eq!(actor.mind.last_seen, Some(w.state.player().pos));
    assert_eq!((actor.mind.noise, actor.mind.post), (None, None));
    assert_eq!(actor.patrol, None, "the old level's beat is no use here");
    assert_eq!(actor.home.as_deref(), Some("test_top"));
    assert!(actor.ally.is_none(), "no ally, no order, no retinue");
    assert!(
        !w.unaware(k, w.state.player),
        "it is still aware of the player"
    );
}

#[test]
fn none_kept_still_or_out_of_action_follows() {
    for status in [
        "asleep",
        "disguised",
        "held",
        "webbed",
        "pinned",
        "stunned",
        "surprised",
        "frightened",
    ] {
        let mut w = world();
        let k = beside(&w);
        afflict(&mut w, k, status);
        run(&mut w, "descend");
        assert_eq!(w.state.actors[&k].level, "test_top", "{status}");
    }
}

#[test]
fn a_slowed_one_follows_and_pays_for_it() {
    let mut w = world();
    let k = beside(&w);
    afflict(&mut w, k, "transfixed");
    let energy = w.state.actors[&k].energy;
    let price = w.price_of(k, ActionKind::Stairs);
    assert!(price > w.price_of(w.state.player, ActionKind::Stairs));
    run(&mut w, "descend");
    assert_eq!(w.state.actors[&k].level, "test_below");
    assert!(w.state.actors[&k].energy < energy, "behind the player");
}

#[test]
fn one_that_was_slipped_past_stays() {
    let mut w = world();
    let k = beside(&w);
    let turn = w.state.turn;
    let actor = w.state.actors.get_mut(&k).expect("kobold");
    actor.mind.missed = Some(turn);
    actor.mind.last_seen = None;
    run(&mut w, "descend");
    assert_eq!(w.state.actors[&k].level, "test_top");
}

#[test]
fn peaceful_ones_keepers_npcs_and_those_who_keep_their_place_stay() {
    for k in [
        "creature = \"kobold\", faction = \"peaceful\"",
        "creature = \"shopkeeper\", faction = \"outraged\"",
        "creature = \"meenlock\"",
    ] {
        let mut w = world_of(&Fixture {
            k,
            ..Fixture::default()
        });
        let id = beside(&w);
        run(&mut w, "descend");
        assert_eq!(w.state.current_level, "test_below", "{k}");
        assert_eq!(w.state.actors[&id].level, "test_top", "{k}");
    }
}

#[test]
fn an_npc_stays_though_it_turned_hostile() {
    let mut w = world();
    let k = beside(&w);
    w.state.actors.get_mut(&k).expect("kobold").npc = Some("someone".to_string());
    run(&mut w, "descend");
    assert_eq!(w.state.actors[&k].level, "test_top");
}

#[test]
fn a_lurker_follows_once_hurt() {
    let mut w = world_of(&Fixture {
        k: "creature = \"meenlock\"",
        ..Fixture::default()
    });
    let id = beside(&w);
    w.state.actors.get_mut(&id).expect("meenlock").hp -= 1;
    run(&mut w, "descend");
    assert_eq!(w.state.actors[&id].level, "test_below");
}

#[test]
fn a_found_out_mimic_stays() {
    let mut w = world_of(&Fixture {
        k: "creature = \"mimic\"",
        ..Fixture::default()
    });
    let id = beside(&w);
    let disguise = w.content.disguise_status().expect("disguised");
    w.state
        .actors
        .get_mut(&id)
        .expect("mimic")
        .effects
        .remove(disguise);
    run(&mut w, "descend");
    assert_eq!(w.state.actors[&id].level, "test_top");
}

/// Three around the player, for the room below.
const CROWD: &str = "\
#########
#...k...#
#..k@...#
#...k...#
#########
";

#[test]
fn with_no_room_one_stays_behind() {
    let mut w = world_of(&Fixture {
        top: CROWD,
        below: NARROW,
        ..Fixture::default()
    });
    let ids = [at(&w, 4, 1), at(&w, 3, 2), at(&w, 4, 3)];
    run(&mut w, "descend");
    let below: Vec<ActorId> = (ids.into_iter())
        .filter(|id| w.state.actors[id].level == "test_below")
        .collect();
    assert_eq!(below, [ids[0]], "the first by id takes the one cell");
    assert_eq!(w.state.actors[&ids[1]].pos, Pos::new(3, 2), "unmoved");
    assert_eq!(w.state.actors[&ids[2]].pos, Pos::new(4, 3), "unmoved");
    told(&w, "The kobold follows you down the stairs.");
}

#[test]
fn several_are_told_in_one_line() {
    let mut w = world_of(&Fixture {
        top: CROWD,
        ..Fixture::default()
    });
    run(&mut w, "descend");
    told(&w, "Three kobolds follow you down the stairs.");
}

#[test]
fn an_ally_keeps_its_place_and_the_follower_stays_behind() {
    for order in ["", "order:stay", "order:hunt:6,1"] {
        let mut w = world_of(&Fixture {
            below: NARROW,
            ..Fixture::default()
        });
        let k = beside(&w);
        run(&mut w, "ally:wolf");
        if !order.is_empty() {
            run(&mut w, order);
        }
        let wolf = (w.state.actors.values())
            .find(|a| a.ally.is_some())
            .map(|a| a.id)
            .expect("the wolf");
        // Right beside the player, wherever the order sent it.
        let cell = Pos::new(4, 1);
        w.state.actors.get_mut(&wolf).expect("wolf").pos = cell;
        run(&mut w, "descend");
        assert_eq!(w.state.actors[&wolf].level, "test_below", "{order}");
        assert_eq!(w.state.actors[&wolf].pos, Pos::new(4, 1), "{order}");
        assert_eq!(w.state.actors[&k].level, "test_top", "{order}");
    }
}

#[test]
fn it_gets_no_blow_out_of_the_climb_and_acts_on_its_own_energy_after() {
    let mut w = world();
    let k = beside(&w);
    let attacked = |events: &[Event]| {
        (events.iter()).any(|e| matches!(e, Event::Attacked { attacker, .. } if *attacker == k))
    };
    let outcomes = run(&mut w, "descend");
    assert!(!attacked(&outcomes[0].events), "the player acts first");
    assert_eq!(w.state.actors[&k].level, "test_below");
    let outcomes = run(&mut w, "wait");
    assert!(attacked(&outcomes[0].events), "then it fights on");
}

#[test]
fn it_never_arrives_on_ground_a_hostile_shuns() {
    let mut w = world_of(&Fixture {
        below_rooms: "[[room]]\neffect = \"sanctuary\"\narea = [0, 0, 8, 4]\n",
        ..Fixture::default()
    });
    let k = beside(&w);
    run(&mut w, "descend");
    assert_eq!(w.state.current_level, "test_below");
    assert_eq!(w.state.actors[&k].level, "test_top");
}

#[test]
fn it_does_not_come_out_to_the_overworld() {
    let mut w = world_of(&Fixture {
        stairs: "terrain = \"stairs_up\", to = \"overworld\"",
        ..Fixture::default()
    });
    let k = beside(&w);
    run(&mut w, "ascend");
    assert_eq!(w.state.current_level, "overworld");
    assert_eq!(w.state.actors[&k].level, "test_top");
    assert!(w.state.actors[&k].home.is_none());
}

#[test]
fn it_counts_toward_the_new_levels_cap_and_its_kill_for_its_home() {
    let mut w = world();
    let k = beside(&w);
    run(&mut w, "descend");
    assert_eq!(w.hostiles_here(), 1);
    let player = w.state.player;
    let mut events = Vec::new();
    w.hurt(k, 1000, Harm::Actor(player), &mut events);
    assert!(!w.state.actors.contains_key(&k), "dead");
    let kills: Vec<_> = w.state.quests.kills().collect();
    assert_eq!(kills, [("test_top", "kobold", 1)]);
}

#[test]
fn summons_left_behind_do_not_count_against_a_summoner_who_followed() {
    let mut w = world();
    let (k, f) = (beside(&w), far(&w));
    w.state.actors.get_mut(&f).expect("summon").summoner = Some(k);
    assert_eq!(w.live_summons(k), 1);
    run(&mut w, "descend");
    assert_eq!(w.state.actors[&k].level, "test_below");
    assert_eq!(w.live_summons(k), 0);
}

#[test]
fn the_same_walk_comes_out_the_same() {
    let hash = || {
        let mut w = world_of(&Fixture {
            top: CROWD,
            ..Fixture::default()
        });
        run(&mut w, "descend wait ascend wait");
        w.state_hash()
    };
    assert_eq!(hash(), hash());
}
