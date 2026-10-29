//! Slipping past (spec §8 "Surprise and sneak attacks"; `world/slip.rs`): the one moment (dark
//! or lit, hunting or not, by darkvision or by light or not at all, another hostile never), the
//! contest's two outcomes and its four draws, no second roll in a stalk, each ending, the
//! creature that came to look and was caught unawares, saves and replays, and no draw where
//! nobody can slip past.

use super::*;
use crate::action::parse_actions;
use crate::check;
use crate::content::ContentFiles;
use crate::fov::radius2;
use crate::replay::Replay;
use crate::rng::Rng;
use crate::sight::LineOfSight;
use crate::skills::Effect;
use crate::stats::Stat;

/// A dark corridor: the player at the west end, a kobold at `k`.
const CORRIDOR: &str = "\
##############
#@..........k#
#............#
##############
";

/// What `data/surprise.toml` says in a fixture.
#[derive(Clone, Copy, PartialEq)]
enum Slip {
    /// The shipped file.
    Shipped,
    /// The shipped file with `[slip] enabled = false`.
    Off,
}

/// The built-in content with [`CORRIDOR`] at `ambient` light; the kobold sees with `eyes`.
fn world_with(ambient: u8, slip: Slip, eyes: &str) -> World {
    let mut files = ContentFiles::builtin();
    let text = include_str!("../../../../assets/data/surprise.toml");
    if slip == Slip::Off {
        files.insert(
            "data/surprise.toml",
            &text.replacen("enabled = true", "enabled = false", 1),
        );
    }
    let kobold = include_str!("../../../../assets/data/creatures/kobold.toml");
    files.insert(
        "data/creatures/kobold.toml",
        &kobold
            .replacen(
                "vision = \"darkvision\"",
                &format!("vision = \"{eyes}\""),
                1,
            )
            // Its torch would light the player up: this one goes without.
            .replacen("light = \"torch\"\n", "", 1),
    );
    files.insert(
        "levels/test_slip.toml",
        &format!(
            "name = \"Test\"\nambient = [{ambient}, {ambient}, {ambient}, 0]\nmap = '''\n{CORRIDOR}'''\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"k\" = {{ terrain = \"floor\", creature = \"kobold\" }}\n"
        ),
    );
    files.insert(
        "scenarios/test_slip.toml",
        "name = \"T\"\nstart_level = \"test_slip\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_slip", None).expect("world")
}

fn world(ambient: u8, slip: Slip) -> World {
    world_with(ambient, slip, "darkvision")
}

fn kobold(w: &World) -> ActorId {
    (w.state.actors.values())
        .find(|a| a.kind == ActorKind::Creature)
        .expect("the kobold")
        .id
}

fn mind(w: &World, id: ActorId) -> Mind {
    w.state.actors[&id].mind
}

/// Puts the player at column `x` of the top row (the kobold at 12: within its darkvision of 5
/// from 7 on).
fn put(w: &mut World, x: i32) {
    let player = w.state.player;
    w.state.actors.get_mut(&player).expect("player").pos = Pos::new(x, 1);
}

/// The contest's two sides as `slip.rs` reckons them for creature `id`.
fn sides(w: &World, id: ActorId) -> (i32, i32) {
    let (player, me) = (w.state.player(), &w.state.actors[&id]);
    (
        player.effective_stats(&w.content).modifier(Stat::Dexterity)
            + player.skill_effect(&w.content, Effect::Slip),
        me.effective_stats(&w.content).modifier(Stat::Willpower)
            + me.skill_effect(&w.content, Effect::Spot),
    )
}

/// Gives `w` a generator whose next contest for creature `id` comes out `made`; returns it
/// after the contest's four draws.
fn rig(w: &mut World, id: ActorId, made: bool) -> Rng {
    let (modifier, against) = sides(w, id);
    for seed in 0.. {
        let mut rng = Rng::gameplay(seed);
        let start = rng.clone();
        if check::contest(&mut rng, modifier, against) == made {
            w.state.rng = start;
            return rng;
        }
    }
    unreachable!()
}

/// What creature `id`'s look ([`super::slip::looks_at`]) says of `other` now.
fn look(w: &mut World, id: ActorId, other: ActorId) -> bool {
    let World {
        state,
        content,
        lights,
        ..
    } = w;
    let (me, them) = (&state.actors[&id], &state.actors[&other]);
    let (from, at) = (me.pos, them.pos);
    let def = content.creature(me.creature);
    assert!(from.dist2(at) <= radius2(def.sight));
    let eye = me
        .eyes(content, &content.vision_profiles()[def.vision])
        .into_owned();
    let level = state.current();
    let los = LineOfSight::compute(from, def.sight, |p| level.is_opaque(content, p));
    super::slip::looks_at(state, content, lights, &los, &eye, id, other, at)
}

#[test]
fn a_dark_approach_rolls_once_and_made_the_creature_does_not_see() {
    let mut w = world(0, Slip::Shipped);
    let (player, k) = (w.state.player, kobold(&w));
    put(&mut w, 7);
    let after = rig(&mut w, k, true);
    let turn = w.state.turn;
    assert!(!look(&mut w, k, player), "slipped past");
    assert_eq!(w.state.rng, after, "the contest's four draws");
    assert_eq!(mind(&w, k).missed, Some(turn));
    assert!(w.slipped_past(k, player));
    assert!(w.unaware(k, player));
    // Looks again: still blind, no draw.
    assert!(!look(&mut w, k, player));
    assert_eq!(w.state.rng, after);
    // Its turns pass, it does not hunt.
    for _ in 0..3 {
        w.apply(&Action::Wait);
    }
    assert_eq!(mind(&w, k).last_seen, None);
    assert_eq!(mind(&w, k).missed, Some(turn));
    let dump = crate::dump::layer_rows(&w, crate::dump::Layer::Actors).join("\n");
    assert!(dump.contains("slipped past"), "{dump}");
    assert!(dump.contains(&format!("missed turn {turn}")), "{dump}");
}

#[test]
fn a_lost_contest_is_seen_and_rolls_no_more_while_it_hunts() {
    let mut w = world(0, Slip::Shipped);
    let (player, k) = (w.state.player, kobold(&w));
    put(&mut w, 7);
    rig(&mut w, k, false);
    w.apply(&Action::Wait);
    assert_eq!(mind(&w, k).missed, None);
    assert!(mind(&w, k).last_seen.is_some(), "it hunts");
    assert!(!w.unaware(k, player));
    // Hunting, it sees by darkvision as ever: no second roll.
    let before = w.state.rng.clone();
    assert!(look(&mut w, k, player));
    assert_eq!(w.state.rng, before);
}

#[test]
fn nothing_rolls_out_of_the_moment() {
    // Lit: seen by light, no roll.
    let mut w = world(120, Slip::Shipped);
    let (player, k) = (w.state.player, kobold(&w));
    put(&mut w, 7);
    let before = w.state.rng.clone();
    assert!(look(&mut w, k, player));
    assert_eq!(w.state.rng, before, "lit");
    // Beyond its darkvision in the dark: not seen, no roll.
    let mut w = world(0, Slip::Shipped);
    put(&mut w, 4);
    let before = w.state.rng.clone();
    assert!(!look(&mut w, k, player));
    assert_eq!(w.state.rng, before, "dark, out of reach");
    // Eyes without darkvision: not seen, no roll.
    let mut w = world_with(0, Slip::Shipped, "normal");
    put(&mut w, 7);
    let before = w.state.rng.clone();
    assert!(!look(&mut w, k, player));
    assert_eq!(w.state.rng, before, "no darkvision");
    // Slipping past switched off: seen by darkvision, no roll.
    let mut w = world(0, Slip::Off);
    put(&mut w, 7);
    let before = w.state.rng.clone();
    assert!(look(&mut w, k, player));
    assert_eq!(w.state.rng, before, "off");
    assert_eq!(mind(&w, k).missed, None);
}

#[test]
fn another_hostile_never_rolls() {
    let mut w = world(0, Slip::Shipped);
    let k = kobold(&w);
    put(&mut w, 1);
    // A goblin in the kobold's darkvision: seen as ever, no draw.
    let id = ActorId(w.state.next_actor_id);
    w.state.next_actor_id += 1;
    let mut goblin = w.state.actors[&k].clone();
    goblin.id = id;
    goblin.creature = w.content.creature_id("goblin").expect("goblin");
    goblin.pos = Pos::new(9, 2);
    w.state.actors.insert(id, goblin);
    let before = w.state.rng.clone();
    assert!(look(&mut w, k, id));
    assert_eq!(w.state.rng, before);
    assert_eq!(mind(&w, k).missed, None);
}

/// A world where the player has just slipped past the kobold from column 7.
fn slipped() -> (World, ActorId) {
    let mut w = world(0, Slip::Shipped);
    let (player, k) = (w.state.player, kobold(&w));
    put(&mut w, 7);
    rig(&mut w, k, true);
    assert!(!look(&mut w, k, player));
    (w, k)
}

#[test]
fn light_its_eyes_see_by_ends_it() {
    let mut w = world(120, Slip::Shipped);
    let (player, k) = (w.state.player, kobold(&w));
    put(&mut w, 7);
    let turn = w.state.turn;
    w.state.actors.get_mut(&k).expect("kobold").mind.missed = Some(turn);
    let before = w.state.rng.clone();
    assert!(look(&mut w, k, player), "seen in the light");
    assert_eq!(w.state.rng, before);
    assert_eq!(mind(&w, k).missed, None);
}

#[test]
fn being_hurt_ends_it_and_it_sees_the_player_at_once() {
    let (mut w, k) = slipped();
    let player = w.state.player;
    let mut events = Vec::new();
    w.hurt(k, 1, super::fight::Harm::Place, &mut events);
    assert_eq!(mind(&w, k).missed, None);
    assert_eq!(mind(&w, k).last_seen, Some(w.state.actors[&player].pos));
    assert!(!w.unaware(k, player));
}

#[test]
fn it_lapses_after_its_turns() {
    let (mut w, k) = slipped();
    let player = w.state.player;
    let lasts = w.content.surprise().and_then(|r| r.slip_turns).expect("on");
    w.state.turn += lasts;
    assert!(w.slipped_past(k, player), "the last turn of it");
    w.state.turn += 1;
    assert!(!w.slipped_past(k, player), "lapsed");
    let before = w.state.rng.clone();
    w.slip_lapses(k);
    assert_eq!(w.state.rng, before, "no draw");
    assert_eq!(mind(&w, k).missed, None);
    assert!(
        mind(&w, k).last_seen.is_some(),
        "it sees the player at last"
    );
}

#[test]
fn the_creature_that_came_to_look_is_caught_unawares() {
    // The design's dark corridor: slipped past, the player's steps send the kobold to look; it
    // arrives beside them none the wiser, and the player's blow finds it unaware.
    let (mut w, k) = slipped();
    let player = w.state.player;
    for action in parse_actions("move:E move:W move:E").expect("actions") {
        w.apply(&action);
    }
    for _ in 0..10 {
        if w.state.actors[&k]
            .pos
            .chebyshev(w.state.actors[&player].pos)
            <= 1
        {
            break;
        }
        w.apply(&Action::Wait);
    }
    let (at, me) = (w.state.actors[&k].pos, w.state.actors[&player].pos);
    assert!(at.chebyshev(me) <= 1, "it came to look: {at} beside {me}");
    assert!(mind(&w, k).last_seen.is_none(), "none the wiser");
    assert!(w.unaware(k, player));
    let dir = Dir::ALL
        .into_iter()
        .find(|d| me.step(*d) == at)
        .expect("a direction");
    w.apply(&Action::Move(dir));
    let said = w
        .messages()
        .iter()
        .map(|m| m.text.clone())
        .collect::<Vec<_>>();
    // The player, unlit and without darkvision, does not see it either: the blow is all they
    // are told of (the surprise's words are only for what they see).
    assert!(
        said.iter()
            .any(|t| t == "You hit something." || t == "You miss something."),
        "{said:?}"
    );
    // The attack ended it: it hunts the player now.
    if w.state.actors.contains_key(&k) {
        assert_eq!(mind(&w, k).missed, None);
        assert!(!w.unaware(k, player));
    }
}

#[test]
fn no_draws_where_nobody_slips_past() {
    // In the light, the same walk draws the same with slipping past on and off.
    let actions = parse_actions("move:E*8 wait*4").expect("actions");
    let mut on = world(120, Slip::Shipped);
    let mut off = world(120, Slip::Off);
    for action in &actions {
        on.apply(action);
        off.apply(action);
        assert_eq!(on.state.rng, off.state.rng, "{action}");
    }
    assert_eq!(on.state.hash(), off.state.hash());
}

#[test]
fn a_slip_past_survives_saves_and_replays() {
    let mut w = world(0, Slip::Shipped);
    let mut replay = Replay::new(&w);
    for action in &parse_actions("move:E*7 wait*3").expect("actions") {
        w.apply(action);
        replay.push(action, &w);
    }
    let loaded = crate::save::decode(&crate::save::encode(&w), w.content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    let replayed = replay.run(w.content.clone()).expect("the replay verifies");
    assert_eq!(replayed.state_hash(), w.state_hash());
    // The mind's new field round-trips when set.
    let (w, k) = slipped();
    let loaded = crate::save::decode(&crate::save::encode(&w), w.content.clone()).expect("loads");
    assert_eq!(
        loaded.state().actors[&k].mind.missed,
        w.state.actors[&k].mind.missed
    );
}
