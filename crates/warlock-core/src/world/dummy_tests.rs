//! Target dummies (#1199; `dummy.rs`): the Gilde's yard in Cēaptūn. A dummy never acts or
//! dies and earns no experience; blows at it train weapon skills and styles slowly up to rank
//! 2, shots up to rank 4, and nothing else; missiles at it drop beside it or are destroyed.

use super::fight::Harm;
use super::*;
use crate::action::{AttackKind, parse_actions};
use crate::content::WeaponSkillId;
use crate::object::Stack;
use std::sync::LazyLock;

/// Built-in content, made once.
static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));

/// The dummy at the west end of the shooting lane, and one of the three below it.
const LANE: Pos = Pos::new(73, 17);
const POST: Pos = Pos::new(77, 19);

fn town() -> World {
    World::new(CONTENT.clone(), "ceaptun", None).expect("a run in Cēaptūn")
}

fn act(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn put_player(world: &mut World, at: Pos) {
    let id = world.state.player;
    world.state.actors.get_mut(&id).expect("player").pos = at;
    world.perceive(&mut Vec::new());
}

fn dummy_at(world: &World, at: Pos) -> ActorId {
    let id = (world.state.actor_at("ceaptun", at)).expect("a dummy").id;
    assert!(world.dummy(id).is_some(), "a training dummy at {at:?}");
    id
}

fn skill(world: &World, id: &str) -> WeaponSkillId {
    world.content().weapon_skills().id(id).expect(id)
}

fn rank(world: &World, id: &str) -> u8 {
    world.state.player().weapon_skills.rank(skill(world, id))
}

/// Gives the player skill `id` at `rank`, its marks one short of the next.
fn nearly(world: &mut World, id: &str, rank: u8) {
    let content = world.content.clone();
    let table = content.weapon_skills();
    let s = skill(world, id);
    let me = world.player_mut();
    me.weapon_skills.raise_to(s, rank);
    me.weapon_skills.add_marks(table, s, u32::MAX);
}

/// A short bow ready, with `arrows` arrows, and nothing else carried.
fn archer(world: &mut World, arrows: u32) {
    let bow = world.content().object_id("short_bow").expect("bow");
    let arrow = world.content().object_id("arrow").expect("arrow");
    let me = world.player_mut();
    me.inventory = vec![Stack::one(bow), Stack::new(arrow, arrows)];
    act(world, "equip:missile_weapon:a equip:missiles:b");
}

fn arrows_around(world: &World, at: Pos) -> (u32, u32) {
    let arrow = world.content().object_id("arrow").expect("arrow");
    let level = world.state.current();
    let mut on = 0;
    let mut beside = 0;
    for y in at.y - 2..=at.y + 2 {
        for x in at.x - 2..=at.x + 2 {
            let p = Pos::new(x, y);
            let n: u32 = (level.objects_at(p).iter())
                .filter(|s| s.object == arrow)
                .map(|s| s.count)
                .sum();
            match p.chebyshev(at) {
                0 => on += n,
                1 => beside += n,
                _ => assert_eq!(n, 0, "no arrow two cells away"),
            }
        }
    }
    (on, beside)
}

#[test]
fn a_dummy_never_acts_never_dies_and_pays_nothing() {
    let mut w = town();
    let post = dummy_at(&w, POST);
    let hp = w.state.actors[&post].hp;
    put_player(&mut w, POST.step(Dir::N));
    let before = w.player_xp();
    act(&mut w, "move:S*200");
    let dummy = &w.state.actors[&post];
    assert_eq!((dummy.pos, dummy.hp), (POST, hp), "unmoved and unhurt");
    assert_eq!(w.player_xp(), before, "no experience");
    assert!(w.state.death.is_none());
    assert_eq!(w.player_pos(), POST.step(Dir::N), "it never makes way");
    // Every blow is told.
    let said: Vec<String> = w.messages().iter().map(|m| m.text.clone()).collect();
    assert!(
        said.iter().any(|m| m.contains("training dummy")),
        "{said:?}"
    );
    // Not even a deadly harm kills it.
    let player = w.state.player;
    assert!(!w.hurt(post, 10_000, Harm::Actor(player), &mut Vec::new()));
    w.dies(post, Some(player), None, w.state.time, &mut Vec::new());
    assert!(w.state.actors.contains_key(&post), "still there");
    assert_eq!(w.player_xp(), before);
}

#[test]
fn blows_train_slowly_up_to_rank_two_and_no_stat() {
    let mut w = town();
    dummy_at(&w, POST);
    put_player(&mut w, POST.step(Dir::N));
    let marks = w.state.player().stat_marks;
    // Bare-handed: unarmed fighting and the brawler's style.
    nearly(&mut w, "unarmed", 1);
    nearly(&mut w, "brawler", 1);
    act(&mut w, "move:S*60");
    assert_eq!(rank(&w, "unarmed"), 2, "rank 2 reached");
    assert_eq!(rank(&w, "brawler"), 2);
    let learned = (w.messages().iter()).filter(|m| m.text.contains(dummy::LEARNED_ALL));
    assert_eq!(learned.count(), 2, "said once for each");
    // From rank 2 nothing more is learned here.
    let unarmed = skill(&w, "unarmed");
    let at_two = w.state.player().weapon_skills.get(unarmed);
    act(&mut w, "move:S*40");
    assert_eq!(w.state.player().weapon_skills.get(unarmed), at_two);
    assert_eq!(w.state.player().stat_marks, marks, "no stat trains");
}

#[test]
fn a_dummy_teaches_fewer_marks_than_a_level_one_foe() {
    let mut w = town();
    let post = dummy_at(&w, POST);
    let unarmed = skill(&w, "unarmed");
    let mut foe = town();
    foe.practise(unarmed, true, 1);
    let full = foe.state.player().weapon_skills.get(unarmed).marks;
    w.practise_on(post, (AttackKind::Melee, unarmed), true, 1);
    let got = w.state.player().weapon_skills.get(unarmed).marks;
    assert!(0 < got && got < full, "{got} of {full}");
    // Without talents, its percent of the plain marks, rounded.
    let rules = *w.content().weapon_skills().marks().expect("marks");
    let percent = w.dummy(post).expect("a dummy").percent;
    assert_eq!((rules.earned(true, 1, 0) * percent + 50) / 100, 2);
}

#[test]
fn shots_train_to_rank_four_and_arrows_drop_beside_it_or_break() {
    let mut w = town();
    dummy_at(&w, LANE);
    archer(&mut w, 400);
    put_player(
        &mut w,
        LANE.step(Dir::E).step(Dir::E).step(Dir::E).step(Dir::E),
    );
    nearly(&mut w, "bows", 3);
    let shots = 300;
    act(&mut w, &format!("fire:{},{}*{shots}", LANE.x, LANE.y));
    assert_eq!(rank(&w, "bows"), 4, "rank 4 reached");
    let bows = skill(&w, "bows");
    assert_eq!(w.state.player().weapon_skills.get(bows).marks, 0, "no more");
    let left = (w.state.player().inventory.iter())
        .find(|s| s.object == w.content().object_id("arrow").expect("arrow"))
        .map_or(0, |s| s.count);
    assert_eq!(left, 400 - shots);
    let (on, beside) = arrows_around(&w, LANE);
    assert_eq!(on, 0, "none on the dummy's own cell");
    let broke = shots - beside;
    // A quarter breaks: 75 expected of 300.
    assert!((50..=100).contains(&broke), "{broke} of {shots} broke");
}

#[test]
fn a_dummy_teaches_blows_no_rank_past_two() {
    let mut w = town();
    let post = dummy_at(&w, POST);
    put_player(&mut w, POST.step(Dir::N));
    nearly(&mut w, "unarmed", 2);
    let unarmed = skill(&w, "unarmed");
    let before = w.state.player().weapon_skills.get(unarmed);
    w.practise_on(post, (AttackKind::Melee, unarmed), true, 1);
    w.practise_on(post, (AttackKind::Thrown, unarmed), true, 1);
    assert_eq!(w.state.player().weapon_skills.get(unarmed), before);
    // Shots go on to rank 4.
    w.practise_on(post, (AttackKind::Ranged, unarmed), true, 1);
    assert_eq!(rank(&w, "unarmed"), 3);
}
