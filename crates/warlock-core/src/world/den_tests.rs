//! Teeming dens (spec §7 "Special rooms"; `world/den.rs`, `mapgen/den.rs`): sleepers take no
//! turn and never wake by time, a quiet noise wakes no one, a loud one wakes the near ones and
//! their cries the rest, a blow wakes, sneak attacks land on sleepers, Hush-foot, sleepers count
//! toward the spawn cap, and a generated den's sleepers and hoard.

use super::fight::Harm;
use super::*;
use crate::content::{ContentFiles, StatusId, Swarm};
use crate::noise::{Noise, SoundKind};

/// A hall, 25 wide: its east part (x 4 on, walls included) a teeming den. Hostile kobolds lie at
/// `a`, `b`, and `c`, three cells apart (a cry carries to the next), and at `d`, five cells past
/// `c` (none reaches it).
const HALL: &str = "\
#########################
#.......................#
#.......................#
#@.....a..b..c....d.....#
#.......................#
#.......................#
#########################
";

/// The built-in content with [`HALL`], its east part a teeming den, the level's header `header`;
/// the kobolds asleep.
fn world(header: &str) -> World {
    let mut files = ContentFiles::builtin();
    let legend: String = ["a", "b", "c", "d"]
        .map(|k| format!("\"{k}\" = {{ terrain = \"floor\", creature = \"kobold\" }}\n"))
        .concat();
    files.insert(
        "levels/test_den.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\n{header}map = '''\n{HALL}'''\n\
             [[room]]\neffect = \"teeming\"\narea = [4, 0, 24, 6]\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n{legend}"
        ),
    );
    files.insert(
        "scenarios/test_den.toml",
        "name = \"T\"\nstart_level = \"test_den\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    let mut w = World::new(content, "test_den", None).expect("world");
    let asleep = asleep(&w);
    let ids: Vec<ActorId> = (w.state.actors.values())
        .filter(|a| a.kind == ActorKind::Creature)
        .map(|a| a.id)
        .collect();
    for id in ids {
        let def = w.content.status(asleep).clone();
        let kobold = w.state.actors.get_mut(&id).expect("kobold");
        kobold.effects.add(asleep, &def, 1, None);
    }
    w
}

fn asleep(w: &World) -> StatusId {
    w.content.status_id("asleep").expect("asleep")
}

/// The kobold lying in column `x`.
fn at(w: &World, x: i32) -> ActorId {
    (w.state.actors.values())
        .find(|a| a.kind == ActorKind::Creature && a.pos.x == x)
        .expect("a kobold")
        .id
}

fn sleeps(w: &World, x: i32) -> bool {
    w.state.actors[&at(w, x)].effects.has(asleep(w))
}

/// Which of the four kobolds (a, b, c, d) sleep.
fn sleeping(w: &World) -> [bool; 4] {
    [7, 10, 13, 18].map(|x| sleeps(w, x))
}

/// A voice of `loudness` made by the player at `(x, 3)`.
fn noise(w: &World, x: i32, loudness: u16) -> Noise {
    Noise {
        kind: SoundKind::Voice,
        at: Pos::new(x, 3),
        loudness,
        maker: Some(w.state.player),
    }
}

#[test]
fn asleep_is_a_status_that_sleeps_and_a_den_is_data() {
    let w = world("");
    let def = w.content.status(asleep(&w));
    assert!(def.sleeps);
    assert_eq!(def.seen.as_deref(), Some("asleep"));
    assert_eq!(def.eases, None, "never by time");
    assert_eq!(w.content.sleep_status(), Some(asleep(&w)));
    let teeming = w.content.room_effects().id("teeming").expect("teeming");
    let def = w.content.room_effect(teeming);
    assert!(def.needs_danger);
    assert_eq!(def.min_cells, 16);
    let swarm = def.swarm.clone().expect("a swarm");
    assert_eq!((swarm.wake, swarm.cry), (200, 60));
    assert_eq!(swarm.sleepers(36), 9);
    assert_eq!(swarm.sleepers(10), 4, "at least four");
    assert_eq!(swarm.sleepers(200), 16, "at most sixteen");
    assert_eq!(Swarm::hoard(9), 2);
    assert_eq!(Swarm::hoard(3), 1);
}

#[test]
fn bad_swarms_are_refused() {
    let shipped = include_str!("../../../../assets/data/room_effects.toml");
    let good = "swarm = { per_cells = 4, min = 4, max = 16, wake = 200, cry = 60, loot_level = 1, \
                themes = { kind = 10, faction = 20 } }";
    assert!(shipped.contains(good), "the shipped den");
    for bad in [
        "swarm = { per_cells = 0, min = 4, max = 16, wake = 200, cry = 60, loot_level = 1 }",
        "swarm = { per_cells = 4, min = 8, max = 4, wake = 200, cry = 60, loot_level = 1 }",
        "swarm = { per_cells = 4, min = 4, max = 99, wake = 200, cry = 60, loot_level = 1 }",
        "swarm = { per_cells = 4, min = 4, max = 16, wake = 50, cry = 60, loot_level = 1 }",
        "swarm = { per_cells = 4, min = 4, max = 16, wake = 200, cry = 900, loot_level = 1 }",
        "swarm = { per_cells = 4, min = 4, max = 16, wake = 200, cry = 60, loot_level = 1, \
         themes = { kind = 60, faction = 60 } }",
    ] {
        let mut files = ContentFiles::builtin();
        files.insert("data/room_effects.toml", &shipped.replace(good, bad));
        assert!(Content::load(&files).is_err(), "{bad}");
    }
    // A den needs a danger.
    let mut files = ContentFiles::builtin();
    let text = shipped.replace("needs_danger = true\n", "");
    files.insert("data/room_effects.toml", &text);
    assert!(Content::load(&files).is_err(), "no needs_danger");
}

#[test]
fn sleepers_take_no_turn_and_never_wake_by_time() {
    let mut w = world("");
    let before: Vec<Pos> = [7, 10, 13, 18]
        .map(|x| w.state.actors[&at(&w, x)].pos)
        .into();
    for _ in 0..200 {
        w.apply(&Action::Wait);
    }
    assert_eq!(sleeping(&w), [true; 4]);
    let after: Vec<Pos> = [7, 10, 13, 18]
        .map(|x| w.state.actors[&at(&w, x)].pos)
        .into();
    assert_eq!(before, after, "nobody moved");
    // A rest of hours wakes no one either.
    w.apply(&Action::Rest(crate::action::Rest::For(
        crate::time::Duration::hours(3),
    )));
    assert_eq!(sleeping(&w), [true; 4]);
}

#[test]
fn a_quiet_noise_wakes_no_one_a_loud_one_the_near_and_their_cries_the_rest() {
    let mut w = world("");
    // Beside `a` (hearing 15): 15 there is heard, but a den wakes at twice that.
    w.spread_noise(noise(&w, 6, 25));
    assert_eq!(sleeping(&w), [true; 4]);
    assert!(
        !w.state.actors[&at(&w, 7)].heard.is_empty(),
        "heard all the same"
    );
    // 30 there wakes `a`; its cry wakes `b`, whose cry wakes `c`; `d` is too far.
    w.spread_noise(noise(&w, 6, 40));
    assert_eq!(sleeping(&w), [false, false, false, true]);
}

#[test]
fn a_door_kicked_beside_the_den_wakes_it() {
    let mut w = world("");
    // A kick's bang (100) two cells from `a`.
    let kick = Noise {
        kind: SoundKind::Banging,
        ..noise(&w, 5, crate::noise::KICK_NOISE)
    };
    w.spread_noise(kick);
    assert_eq!(
        sleeping(&w),
        [false, false, false, true],
        "the bang and the cries; `d` lies beyond both"
    );
}

#[test]
fn a_blow_wakes_the_one_struck() {
    let mut w = world("");
    let id = at(&w, 18);
    let player = w.state.player;
    let mut events = Vec::new();
    w.hurt(id, 1, Harm::Actor(player), &mut events);
    assert!(!sleeps(&w, 18), "struck awake");
    // Its cry reaches no one (five cells to `c`).
    assert_eq!(sleeping(&w), [true, true, true, false]);
}

#[test]
fn sleepers_are_unaware_so_sneak_attacks_land() {
    let mut w = world("");
    let id = at(&w, 7);
    let player = w.state.player;
    // Beside it, in a lit hall, it has seen no one.
    w.player_mut().pos = Pos::new(6, 3);
    assert!(w.unaware(id, player));
    w.player_mut().talents.take("ambusher", None);
    assert!(w.sneaking(player, id, false));
    // Awake, it sees them.
    w.wake(id);
    assert!(!w.unaware(id, player));
}

#[test]
fn a_waking_sleeper_is_seen_waking() {
    let mut w = world("");
    w.apply(&Action::Wait);
    let id = at(&w, 18);
    w.wake(id);
    assert!(
        w.messages()
            .iter()
            .any(|m| m.text == "The kobold wakes up.")
    );
    // Look mode names the sleepers.
    let kobold = at(&w, 7);
    let seen: Vec<StatusId> = w.perception().effects_of(kobold).collect();
    assert_eq!(seen, [asleep(&w)], "seen asleep");
}

#[test]
fn sleepers_count_toward_the_spawn_cap() {
    let w = world("danger = 3\npopulation = [\"kobold\"]\n");
    let (_, hostiles, _, _) = w.spawn_status().expect("a danger");
    assert_eq!(hostiles, 4, "the four sleepers");
}

#[test]
fn a_plain_step_beside_a_sleeper_wakes_it() {
    let mut w = world("");
    w.player_mut().pos = Pos::new(5, 2);
    w.apply(&Action::Move(Dir::E));
    assert_eq!(w.state.player().pos, Pos::new(6, 2));
    assert!(!sleeps(&w, 7), "footsteps of 60, 50 beside it: awake");
}

#[test]
fn sleepers_save_and_hash_alike() {
    let w = world("");
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(loaded.state_hash(), w.state_hash());
    assert!(loaded.state.actors[&at(&w, 7)].effects.has(asleep(&w)));
}

/// A rooms branch whose every special room is a teeming den where one fits.
const WARREN: &str = r#"
[[dungeon]]
id = "warren"
name = "Warren"
generator = "rooms"
depth = 1
size = [80, 21]
danger = 3
population = ["kobold", "kobold_archer", "kobold_shaman", "rat"]
room_effects = { chance = 100, per_level = 1, favoured = { teeming = 100 } }
"#;

#[test]
fn a_generated_den_holds_sleepers_of_the_danger_and_their_hoard() {
    let mut files = ContentFiles::builtin();
    let dungeons = format!(
        "{}{WARREN}",
        include_str!("../../../../assets/data/dungeons.toml")
    );
    files.insert("data/dungeons.toml", &dungeons);
    let content = Content::load(&files).expect("content");
    let teeming = content.room_effects().id("teeming").expect("teeming");
    let swarm = content.room_effect(teeming).swarm.clone().expect("swarm");
    let player = content.creature_id("player").expect("player");
    let mut dens = 0;
    for seed in 0..20 {
        let make = |s| crate::mapgen::generate(&content, ("showcase", s), "warren:1", None);
        let level = make(seed).expect("warren:1");
        assert_eq!(make(seed).expect("again"), level, "deterministic");
        let Some(room) = level.rooms.iter().find(|r| r.effect == teeming) else {
            continue;
        };
        dens += 1;
        let (inside, outside): (Vec<_>, Vec<_>) =
            level.creatures.iter().partition(|c| room.contains(c.pos));
        assert!(inside.iter().all(|c| c.asleep), "seed {seed}: all asleep");
        assert!(
            outside.iter().all(|c| !c.asleep),
            "seed {seed}: none outside"
        );
        let n = inside.len();
        assert!((4..=16).contains(&n), "seed {seed}: {n} sleepers");
        let floor = (room.cells().into_iter())
            .filter(|&p| content.terrain(level.terrain[p]).walkable)
            .count();
        assert!(
            n <= swarm.sleepers(floor),
            "seed {seed}: one per four cells"
        );
        for c in &inside {
            let def = content.creature(c.creature);
            assert!(!def.unique);
            assert_eq!(
                content.stance(c.creature, player),
                crate::content::Stance::Hostile
            );
            assert!(c.route.is_empty(), "no post to return to");
            assert!(!c.rolls_gear, "gear rolled with the rest");
        }
        let hoard = (level.objects.keys())
            .filter(|p| room.contains(**p))
            .count();
        assert!(hoard >= 1, "seed {seed}: a hoard");
    }
    assert!(dens >= 10, "{dens} dens in 20 floors");
}
