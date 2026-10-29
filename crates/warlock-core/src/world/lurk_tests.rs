//! Lurking (spec §8 "Creature behavior"; `world/lurk.rs`): a lurker that sees a hostile waits
//! where it stands, aware of it, attacks it once it is beside it, hunts it once hurt, and waits
//! out a rest the same way; a creature that does not lurk closes in.

use super::*;
use crate::action::{Rest, parse_actions};
use crate::content::ContentFiles;
use crate::time::Duration;

/// A lit corridor: the player at the west end, an Unseelie meenlock at `m`.
const CORRIDOR: &str = "\
##############
#@.......m...#
#............#
##############
";

/// Two rooms: the player alone in the west one, out of sight; an Unseelie meenlock at `m` and
/// a Seelie one at `n`, the courts at war, in the east one.
const ROOMS: &str = "\
#########################
#@........#.............#
#.........#..m.....n....#
#########################
";

/// The built-in content with `map` in full light; the meenlock lurks if `lurks`.
fn world_with(map: &str, lurks: bool) -> World {
    let mut files = ContentFiles::builtin();
    if !lurks {
        let meenlock = include_str!("../../../../assets/data/creatures/meenlock.toml");
        files.insert(
            "data/creatures/meenlock.toml",
            &meenlock.replacen("lurks = true\n", "", 1),
        );
    }
    files.insert(
        "levels/test_lurk.toml",
        &format!(
            "name = \"Test\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"m\" = {{ terrain = \"floor\", creature = \"meenlock\", faction = \"unseelie\" }}\n\
             \"n\" = {{ terrain = \"floor\", creature = \"meenlock\", faction = \"seelie\" }}\n"
        ),
    );
    files.insert(
        "scenarios/test_lurk.toml",
        "name = \"T\"\nstart_level = \"test_lurk\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_lurk", None).expect("world")
}

/// The creatures on the level, west to east.
fn meenlocks(w: &World) -> Vec<ActorId> {
    let mut found: Vec<&Actor> = (w.state.actors.values())
        .filter(|a| a.kind == ActorKind::Creature)
        .collect();
    found.sort_by_key(|a| a.pos.x);
    found.iter().map(|a| a.id).collect()
}

fn pos(w: &World, id: ActorId) -> Pos {
    w.state.actors[&id].pos
}

fn waits(w: &mut World, n: usize) -> Vec<Event> {
    (0..n).flat_map(|_| w.apply(&Action::Wait).events).collect()
}

fn attacked_by(events: &[Event], id: ActorId) -> bool {
    (events.iter()).any(|e| matches!(e, Event::Attacked { attacker, .. } if *attacker == id))
}

#[test]
fn a_lurker_waits_where_it_stands_aware_of_the_player() {
    let mut w = world_with(CORRIDOR, true);
    let m = meenlocks(&w)[0];
    let at = pos(&w, m);
    let events = waits(&mut w, 6);
    assert_eq!(pos(&w, m), at, "it stays put");
    assert!(!attacked_by(&events, m));
    assert_eq!(w.state.actors[&m].mind.last_seen, Some(w.player_pos()));
    let player = w.state.player;
    assert!(!w.unaware(m, player), "it saw the player");
}

#[test]
fn a_creature_that_does_not_lurk_closes_in() {
    let mut w = world_with(CORRIDOR, false);
    let m = meenlocks(&w)[0];
    let at = pos(&w, m);
    waits(&mut w, 2);
    assert!(pos(&w, m).chebyshev(w.player_pos()) < at.chebyshev(w.player_pos()));
}

#[test]
fn a_lurker_attacks_the_player_beside_it() {
    let mut w = world_with(CORRIDOR, true);
    let m = meenlocks(&w)[0];
    let at = pos(&w, m);
    let mut events = Vec::new();
    for action in parse_actions("move:E*7").expect("actions") {
        events.extend(w.apply(&action).events);
    }
    assert_eq!(w.player_pos().chebyshev(at), 1, "beside it");
    events.extend(waits(&mut w, 2));
    assert!(
        attacked_by(&events, m),
        "it strikes once the player is beside it"
    );
    assert_eq!(pos(&w, m), at);
}

#[test]
fn a_hurt_lurker_hunts() {
    let mut w = world_with(CORRIDOR, true);
    let m = meenlocks(&w)[0];
    let at = pos(&w, m);
    waits(&mut w, 2);
    assert_eq!(pos(&w, m), at);
    w.state.actors.get_mut(&m).expect("meenlock").hp -= 1;
    waits(&mut w, 2);
    assert!(pos(&w, m).chebyshev(w.player_pos()) < at.chebyshev(w.player_pos()));
}

/// Away from the resting player (out of sight behind the wall), two lurkers that see each
/// other take their coarse turns (`stride.rs`) and spend them waiting: an hour's rest later
/// both stand where they stood, unhurt; without lurking they would have met and fought.
#[test]
fn lurkers_wait_out_a_rest_away_from_the_player() {
    let mut w = world_with(ROOMS, true);
    let ids = meenlocks(&w);
    let before: Vec<Pos> = ids.iter().map(|&id| pos(&w, id)).collect();
    let out = w.apply(&Action::Rest(Rest::For(Duration::hours(1))));
    assert!(out.took_turn);
    for (&id, at) in ids.iter().zip(before) {
        let me = &w.state.actors[&id];
        assert_eq!(me.pos, at);
        assert_eq!(me.hp, me.max_hp(&w.content));
        assert!(me.mind.last_seen.is_some(), "it saw the other court");
    }
    let mut w = world_with(ROOMS, false);
    w.apply(&Action::Rest(Rest::For(Duration::hours(1))));
    let hurt = (meenlocks(&w).iter()).any(|id| {
        let me = &w.state.actors[id];
        me.hp < me.max_hp(&w.content)
    });
    assert!(
        hurt || meenlocks(&w).len() < 2,
        "without lurking the courts fight"
    );
}
