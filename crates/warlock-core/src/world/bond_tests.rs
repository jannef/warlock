//! Hunter's bond (spec §8 "Allies"; docs/design/ranger-companion.md §11.2): the shared sight
//! (the second viewpoint in `Perception::compute`: the companion's eyes plus its nose, memory,
//! the retinue) and the shared buffs (`bond.rs`: a buff both ways, a hex and a harm never, the
//! retinue, no echo). The witch's Familiar's sight (#932): the same viewpoint on the familiar, a
//! nose of its own for her, no buffs.

use super::*;
use crate::action::parse_actions;
use crate::content::{Content, ContentFiles};
use crate::geom::Pos;
use crate::perception::Perception;
use crate::sight::{self, LineOfSight};
use std::sync::{Arc, LazyLock};

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));

/// Two dim rooms, a wall between them and a closed door in it: the player in the west room
/// (human eyes), the east room for the hound (low-light eyes) to see, and a kobold behind the
/// door when `kobold`.
fn world(kobold: bool) -> World {
    hall(kobold, '+')
}

/// The fixture with `door` (`+` a closed door, `#` a wall) in the wall between the rooms.
fn hall(kobold: bool, door: char) -> World {
    let mut w = hall_with(kobold, door, "hound");
    let id = w.allies_of(w.state.player)[0];
    keep(&mut w, id, AllySource::Companion(snel()));
    w
}

fn snel() -> CompanionData {
    CompanionData {
        name: "Snel".into(),
        grade: 1,
        scent: None,
        quarry: None,
    }
}

fn keep(w: &mut World, id: ActorId, source: AllySource) {
    w.state
        .actors
        .get_mut(&id)
        .unwrap()
        .ally
        .as_mut()
        .unwrap()
        .source = source;
}

/// The fixture with an ally of type `beast`, a summon as yet.
fn hall_with(kobold: bool, door: char, beast: &str) -> World {
    let k = if kobold { "k" } else { "." };
    let map = format!(
        "##############\n#@....#......#\n#.....{door}..{k}...#\n#.....#......#\n##############\n"
    );
    let level = format!(
        "name = \"Bond\"\nambient = [20, 20, 20, 0]\nmap = '''\n{map}'''\n[legend]\n\
         \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
         \"k\" = {{ terrain = \"floor\", creature = \"kobold\" }}\n"
    );
    let mut files = ContentFiles::builtin();
    files.insert("levels/bond_hall.toml", &level);
    files.insert(
        "scenarios/bond_hall.toml",
        "name = \"Bond\"\nstart_level = \"bond_hall\"\nseed = 7\nhp = 1000\n",
    );
    let content = Arc::new(Content::load(&files).expect("the fixture content loads"));
    let mut w = World::new(content, "bond_hall", Some(7)).expect("world");
    run(&mut w, &format!("ally:{beast}"));
    w
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn bond(w: &mut World) {
    w.player_mut().talents.take("hunters_bond", None);
}

fn put(w: &mut World, id: ActorId, at: Pos) {
    w.state.actors.get_mut(&id).unwrap().pos = at;
}

fn compute(w: &mut World) -> Perception {
    Perception::compute(&w.state, &w.content, &mut w.lights)
}

fn status(w: &World, id: &str) -> StatusId {
    w.content().status_id(id).expect("the status")
}

fn has(w: &World, actor: ActorId, id: &str) -> Option<u16> {
    let status = status(w, id);
    w.state.actors[&actor]
        .effects
        .get(status)
        .map(|e| e.severity)
}

fn afflict(w: &mut World, actor: ActorId, id: &str, severity: u16) {
    let status = status(w, id);
    let mut events = Vec::new();
    w.afflict(actor, status, severity, None, &mut events);
}

// ---- shared sight ----------------------------------------------------------------------------

#[test]
fn the_shared_sight_is_the_companions_eyes_plus_its_nose() {
    let mut w = world(true);
    let hound = w.companion().expect("the companion");
    put(&mut w, hound, Pos::new(9, 1));
    let kobold = (w.state.actors.values())
        .find(|a| w.content().creature(a.creature).id == "kobold")
        .map(|a| (a.id, a.pos))
        .expect("the kobold");
    let alone = compute(&mut w);
    bond(&mut w);
    let shared = compute(&mut w);
    // What the hound's own sight finds, by creature sight's rule.
    let def = w
        .content()
        .creature(w.state.actors[&hound].creature)
        .clone();
    let level = w.state.current().clone();
    let los = LineOfSight::compute(Pos::new(9, 1), def.sight, |p| {
        level.is_opaque(&w.content, p)
    });
    let eye = w.content().vision_profiles()[def.vision].clone();
    let nose: Vec<Pos> = super::companion::smelt(
        &w.state,
        &w.content,
        &w.state.actors[&hound],
        &|s, c, _, a| stance_between(s, c, s.player(), a) == crate::content::Stance::Hostile,
    )
    .into_iter()
    .map(|(_, p)| p)
    .collect();
    let mut by_hound = 0;
    for y in 0..level.height() {
        for x in 0..level.width() {
            let p = Pos::new(x, y);
            if level.is_opaque(&w.content, p) {
                continue;
            }
            let hound_sees =
                sight::sees_by(&w.state, &w.content, &mut w.lights, &los, &eye, p).is_some();
            let expected = alone.in_sight(p) || hound_sees || nose.contains(&p);
            assert_eq!(shared.in_sight(p), expected, "cell {p:?}");
            by_hound += usize::from(hound_sees && !alone.in_sight(p));
        }
    }
    // The room is too dim for human eyes, not for a hound's.
    assert!(by_hound > 10, "the hound sees cells the player does not");
    assert!(!alone.in_sight(kobold.1) && shared.in_sight(kobold.1));
    assert!(shared.sees_actors_at(kobold.1) && !alone.sees_actors_at(kobold.1));
}

#[test]
fn the_nose_shows_a_hostile_behind_a_closed_door() {
    let mut w = world(true);
    let hound = w.companion().expect("the companion");
    // West of the door, in the player's room: the kobold three cells beyond the door.
    put(&mut w, hound, Pos::new(5, 2));
    let at = (w.state.actors.values())
        .find(|a| w.content().creature(a.creature).id == "kobold")
        .map(|a| a.pos)
        .expect("the kobold");
    let alone = compute(&mut w);
    assert!(!alone.in_sight(at));
    bond(&mut w);
    let shared = compute(&mut w);
    assert!(
        shared.in_sight(at),
        "the nose finds the kobold through the door"
    );
    // The cells beside it stay unseen: the nose shows the foe, not the room.
    assert!(!shared.in_sight(Pos::new(at.x + 1, at.y)));
}

#[test]
fn the_nose_stops_at_a_wall() {
    let mut w = hall(true, '#');
    let hound = w.companion().expect("the companion");
    put(&mut w, hound, Pos::new(5, 2));
    let at = (w.state.actors.values())
        .find(|a| w.content().creature(a.creature).id == "kobold")
        .map(|a| a.pos)
        .expect("the kobold");
    bond(&mut w);
    assert!(
        !compute(&mut w).in_sight(at),
        "no smell comes through a wall"
    );
}

#[test]
fn what_the_companion_sees_is_remembered() {
    let far = Pos::new(12, 3);
    for shared in [false, true] {
        let mut w = world(false);
        let hound = w.companion().expect("the companion");
        if shared {
            bond(&mut w);
        }
        // Shut in the east room: the door is one a hound cannot open.
        put(&mut w, hound, Pos::new(10, 2));
        run(&mut w, "wait");
        assert_eq!(w.state.current().memory[far].is_some(), shared);
    }
}

#[test]
fn a_dazzled_player_keeps_the_companions_view() {
    // A drow ranger beside a brazier in a long lit corridor, their beast far down it.
    let corridor = format!(
        "{}\n#@B{}#\n{}\n",
        "#".repeat(32),
        ".".repeat(28),
        "#".repeat(32)
    );
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/glare_bond.toml",
        &format!(
            "name = \"Corridor\"\nambient = [60, 60, 60, 0]\nmap = '''\n{corridor}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"B\" = {{ terrain = \"floor\", light = \"brazier\" }}\n"
        ),
    );
    files.insert(
        "scenarios/glare_bond.toml",
        "name = \"Corridor\"\nstart_level = \"glare_bond\"\nseed = 7\nhp = 1000\n",
    );
    let content = Arc::new(Content::load(&files).expect("the fixture content loads"));
    let spec = "drow,male,ranger".parse().expect("a character");
    let mut w =
        World::start_run(content, "glare_bond", Some(7), Some(&spec), None, true).expect("world");
    let beast = w.companion().expect("the ranger's beast");
    put(&mut w, beast, Pos::new(25, 1));
    let far = Pos::new(28, 1);
    let alone = compute(&mut w);
    assert!(alone.dazzled() && !alone.in_sight(far));
    bond(&mut w);
    let shared = compute(&mut w);
    assert!(shared.dazzled() && shared.in_sight(far));
}

#[test]
fn a_companion_in_the_retinue_lends_no_eyes() {
    // The showcase's overworld: Osric's beast travels in the retinue.
    let mut w = World::new(BUILTIN.clone(), "world", None).expect("world");
    let beast = w.companion().expect("the companion");
    assert!(w.state.actors[&beast].in_retinue());
    let alone = compute(&mut w);
    bond(&mut w);
    let shared = compute(&mut w);
    let level = w.state.current();
    for y in 0..level.height() {
        for x in 0..level.width() {
            let p = Pos::new(x, y);
            assert_eq!(alone.in_sight(p), shared.in_sight(p), "cell {p:?}");
        }
    }
}

// ---- Familiar's sight ------------------------------------------------------------------------

/// The fixture with a familiar of type `beast`, the player holding Familiar's sight when `sight`.
fn familiar_hall(door: char, beast: &str, sight: bool) -> (World, ActorId, Pos) {
    let mut w = hall_with(true, door, beast);
    let id = w.allies_of(w.state.player)[0];
    keep(&mut w, id, AllySource::Familiar(snel()));
    if sight {
        w.player_mut().talents.take("familiars_sight", None);
    }
    let at = (w.state.actors.values())
        .find(|a| w.content().creature(a.creature).id == "kobold")
        .map(|a| a.pos)
        .expect("the kobold");
    (w, id, at)
}

#[test]
fn a_witch_comes_into_her_familiars_sight_at_25() {
    let content = BUILTIN.clone();
    let spec = "human,female,witch".parse().expect("a character");
    let mut w = World::start_run(content, "showcase", Some(7), Some(&spec), None, true).unwrap();
    assert!(w.familiar().is_some());
    crate::world::caster_tests::reach(&mut w, 24);
    assert!(!w.state.player().talent_mods(&w.content).familiar_sight);
    crate::world::caster_tests::reach(&mut w, 25);
    let mods = w.state.player().talent_mods(&w.content);
    assert!(mods.familiar_sight);
    assert!(!mods.shared_sight && !mods.shared_buffs, "the sight alone");
}

#[test]
fn a_familiars_sight_is_its_own_eyes() {
    let (mut w, cat, _) = familiar_hall('#', "cat", false);
    put(&mut w, cat, Pos::new(9, 1));
    let far = Pos::new(12, 3);
    assert!(!compute(&mut w).in_sight(far), "not without the talent");
    w.player_mut().talents.take("familiars_sight", None);
    assert!(compute(&mut w).in_sight(far), "the room behind the wall");
    run(&mut w, "wait");
    assert!(w.state.current().memory[far].is_some(), "and remembered");
}

#[test]
fn a_familiar_without_a_nose_of_its_own_smells_for_her() {
    let (mut w, cat, at) = familiar_hall('+', "cat", false);
    assert_eq!(
        w.content().creature(w.state.actors[&cat].creature).nose,
        None
    );
    // Before the door: a cat sees nothing through it.
    put(&mut w, cat, Pos::new(5, 2));
    assert!(!compute(&mut w).in_sight(at));
    w.player_mut().talents.take("familiars_sight", None);
    let shared = compute(&mut w);
    assert!(shared.in_sight(at), "through the closed door");
    assert!(shared.sees_actors_at(at));
    assert!(
        !shared.in_sight(Pos::new(at.x + 1, at.y)),
        "the foe's cell only"
    );
    // A wall stops it, and it is hers alone: the cat itself has nothing to hunt by.
    let (mut w, cat, at) = familiar_hall('#', "cat", true);
    put(&mut w, cat, Pos::new(5, 2));
    assert!(!compute(&mut w).in_sight(at), "no smell through a wall");
    let any = |_: &WorldState, _: &Content, _: &Actor, _: &Actor| true;
    assert!(super::companion::smelt(&w.state, &w.content, &w.state.actors[&cat], &any).is_empty());
}

#[test]
fn a_familiars_nose_reaches_the_rules_or_its_kinds_if_farther() {
    let nose = BUILTIN.familiars().expect("the data").nose;
    let smells = |beast: &str, gap: i32| {
        let (mut w, id, at) = familiar_hall('+', beast, true);
        put(&mut w, id, Pos::new(at.x - gap, at.y));
        compute(&mut w).in_sight(at)
    };
    assert!(smells("cat", nose));
    assert!(!smells("cat", nose + 1));
    assert!(smells("rat", nose), "a rat's own nose is shorter");
    assert!(smells("black_hound", nose + 1), "a black hound's is longer");
}

#[test]
fn a_familiar_in_the_retinue_lends_no_eyes() {
    let spec = "human,female,witch".parse().expect("a character");
    let content = BUILTIN.clone();
    let mut w = World::start_run(content, "world", Some(7), Some(&spec), None, true).unwrap();
    let beast = w.familiar().expect("her familiar");
    assert!(w.state.actors[&beast].in_retinue());
    let alone = compute(&mut w);
    w.player_mut().talents.take("familiars_sight", None);
    let shared = compute(&mut w);
    let level = w.state.current();
    for y in 0..level.height() {
        for x in 0..level.width() {
            let p = Pos::new(x, y);
            assert_eq!(alone.in_sight(p), shared.in_sight(p), "cell {p:?}");
        }
    }
}

#[test]
fn each_talent_reads_its_own_beast() {
    let far = Pos::new(12, 3);
    // A companion lends the witch nothing, and a familiar the ranger nothing.
    let (mut w, cat, _) = familiar_hall('#', "cat", true);
    put(&mut w, cat, Pos::new(1, 2));
    run(&mut w, "ally:hound");
    let hound = (w.allies_of(w.state.player).into_iter())
        .find(|&id| id != cat)
        .expect("the hound");
    keep(&mut w, hound, AllySource::Companion(snel()));
    put(&mut w, hound, Pos::new(9, 1));
    assert!(
        !compute(&mut w).in_sight(far),
        "Familiar's sight, a companion"
    );

    let (mut w, cat, _) = familiar_hall('#', "cat", false);
    put(&mut w, cat, Pos::new(9, 1));
    bond(&mut w);
    assert!(!compute(&mut w).in_sight(far), "Hunter's bond, a familiar");
}

#[test]
fn familiars_sight_shares_no_buff() {
    let (mut w, cat, _) = familiar_hall('.', "cat", true);
    let player = w.state.player;
    afflict(&mut w, player, "hasted", 2);
    assert!(has(&w, player, "hasted").is_some());
    assert_eq!(has(&w, cat, "hasted"), None);
    afflict(&mut w, cat, "fortified", 1);
    assert_eq!(has(&w, player, "fortified"), None);
}

// ---- shared buffs ----------------------------------------------------------------------------

#[test]
fn a_buff_is_shared_both_ways_and_a_hex_never() {
    let mut w = world(false);
    let hound = w.companion().expect("the companion");
    let player = w.state.player;
    // Without the bond nothing crosses.
    afflict(&mut w, player, "barkskin", 2);
    assert_eq!(has(&w, hound, "barkskin"), None);
    bond(&mut w);
    // A buff spell's status, with its severity; said for the companion.
    afflict(&mut w, player, "hasted", 2);
    assert_eq!(has(&w, hound, "hasted"), has(&w, player, "hasted"));
    assert!(has(&w, hound, "hasted").is_some());
    run(&mut w, "wait");
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    assert!(
        said.iter().any(|m| m == "Snel is quickened too."),
        "{said:?}"
    );
    // A potion's status (a boost), the other way round: from the companion to the player.
    afflict(&mut w, hound, "fortified", 1);
    assert_eq!(has(&w, player, "fortified"), Some(1));
    // A hex and a harm never cross, either way.
    afflict(&mut w, player, "weakened", 1);
    afflict(&mut w, hound, "sluggish", 1);
    afflict(&mut w, player, "bleeding", 1);
    assert_eq!(has(&w, hound, "weakened"), None);
    assert_eq!(has(&w, player, "sluggish"), None);
    assert_eq!(has(&w, hound, "bleeding"), None);
}

#[test]
fn a_buff_reaches_the_companion_in_the_retinue() {
    let mut w = World::new(BUILTIN.clone(), "world", None).expect("world");
    let beast = w.companion().expect("the companion");
    bond(&mut w);
    let player = w.state.player;
    afflict(&mut w, player, "barkskin", 1);
    assert!(w.state.actors[&beast].in_retinue());
    assert_eq!(has(&w, beast, "barkskin"), Some(1));
}
