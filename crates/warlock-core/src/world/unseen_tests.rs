//! The unseen (`world/unseen.rs`; docs/design/monster-dread.md §5.10, §5.11): the shadowy by
//! the light and beside the player, the invisible by status and by type, what shows them (Seeing,
//! Blood scent, the companion's nose, a blow that found it), striking at the unseen, the sound of
//! its blows, the self-buff step, the wisp's light, the stalker's hunt, and the roster rows.

use super::*;
use crate::action::parse_actions;
use crate::content::{Content, ContentFiles, Invisible};
use crate::geom::Pos;
use crate::noise::SoundKind;
use crate::perception::Perception;
use std::sync::{Arc, LazyLock};

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));

/// A room, the player at its west end, then a second room behind a wall (no way between them
/// but a corridor round the south): `s` a shadow, `d` a duergar warrior, `i` an invisible
/// stalker, `w` a will-o'-wisp, `D` a duergar warrior in the far room.
const ROOM: &str = "\
######################
#@.........#.........#
#..........#......D..#
#..........#.........#
#..........###.#######
#....................#
######################
";

/// The fixture with `ambient` light (0 to 255, grey) and `map`.
fn world_with(map: &str, ambient: u8) -> World {
    let level = format!(
        "name = \"Unseen\"\nambient = [{ambient}, {ambient}, {ambient}, 0]\nmap = '''\n{map}'''\n\
         [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
         \"s\" = {{ terrain = \"floor\", creature = \"shadow\" }}\n\
         \"d\" = {{ terrain = \"floor\", creature = \"duergar_warrior\" }}\n\
         \"D\" = {{ terrain = \"floor\", creature = \"duergar_warrior\" }}\n\
         \"i\" = {{ terrain = \"floor\", creature = \"invisible_stalker\" }}\n\
         \"w\" = {{ terrain = \"floor\", creature = \"will_o_wisp\" }}\n"
    );
    let mut files = ContentFiles::builtin();
    files.insert("levels/unseen_room.toml", &level);
    files.insert(
        "scenarios/unseen_room.toml",
        "name = \"Unseen\"\nstart_level = \"unseen_room\"\nseed = 7\nhp = 1000\n",
    );
    let content = Arc::new(Content::load(&files).expect("the fixture content loads"));
    World::new(content, "unseen_room", Some(7)).expect("world")
}

/// The room with one creature, `glyph`, put on `at`.
fn room_with(glyph: char, at: Pos, ambient: u8) -> (World, ActorId) {
    let mut lines: Vec<Vec<char>> = ROOM.lines().map(|l| l.chars().collect()).collect();
    for line in &mut lines {
        for c in line.iter_mut().filter(|c| **c == 'D') {
            *c = '.';
        }
    }
    lines[usize::try_from(at.y).expect("y")][usize::try_from(at.x).expect("x")] = glyph;
    let map: String = lines
        .iter()
        .map(|l| l.iter().collect::<String>() + "\n")
        .collect();
    let w = world_with(&map, ambient);
    let id = only_creature(&w);
    (w, id)
}

fn only_creature(w: &World) -> ActorId {
    let found: Vec<ActorId> = (w.state.actors.values())
        .filter(|a| a.kind == ActorKind::Creature)
        .map(|a| a.id)
        .collect();
    assert_eq!(found.len(), 1, "one creature");
    found[0]
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn put(w: &mut World, id: ActorId, at: Pos) {
    w.state.actors.get_mut(&id).expect("the actor").pos = at;
}

fn compute(w: &mut World) -> Perception {
    Perception::compute(&w.state, &w.content, &mut w.lights)
}

fn shows(w: &mut World, id: ActorId) -> bool {
    compute(w).shows(id)
}

fn status(w: &World, id: &str) -> StatusId {
    w.content().status_id(id).expect("the status")
}

fn has(w: &World, actor: ActorId, id: &str) -> bool {
    w.state.actors[&actor].effects.has(status(w, id))
}

fn afflict(w: &mut World, actor: ActorId, id: &str) {
    let status = status(w, id);
    let mut events = Vec::new();
    w.afflict(actor, status, 1, None, &mut events);
}

fn said(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

/// It neither moves nor acts: perception alone is under test.
fn hold(w: &mut World, id: ActorId) {
    w.state.actors.get_mut(&id).expect("the actor").energy = i64::MIN / 2;
}

// ---- the perception pass -------------------------------------------------------------------

#[test]
fn a_shadow_is_unseen_in_dim_light_but_beside_the_player_and_seen_in_full_light() {
    // Ambient 50: the room is visible to human eyes (threshold 40), but dim (under 80).
    let (mut w, shadow) = room_with('s', Pos::new(5, 2), 50);
    let view = compute(&mut w);
    assert!(view.is_visible(Pos::new(5, 2)), "its cell is in sight");
    assert!(!view.shows(shadow), "a shadow in the dim");
    assert!(!view.sees_actors_at(Pos::new(5, 2)), "no one shows there");
    put(&mut w, shadow, Pos::new(2, 2));
    assert!(shows(&mut w, shadow), "beside the player, a shadow is seen");
    let (mut w, shadow) = room_with('s', Pos::new(5, 2), 255);
    assert!(shows(&mut w, shadow), "a lit cell shows it");
}

#[test]
fn darkvision_does_not_show_a_shadow() {
    let (mut w, shadow) = room_with('s', Pos::new(4, 2), 0);
    w.state.vision = "darkvision".into();
    w.state.vision_dev = true;
    let view = compute(&mut w);
    assert!(view.is_visible(Pos::new(4, 2)), "darkvision sees the cell");
    assert!(!view.shows(shadow));
}

#[test]
fn a_cold_draught_tells_of_a_shadow_coming_near() {
    let (mut w, _) = room_with('s', Pos::new(9, 3), 50);
    for _ in 0..20 {
        run(&mut w, "wait");
        if said(&w).iter().any(|m| m == "You feel a cold draught.") {
            return;
        }
    }
    panic!("no draught: {:?}", said(&w));
}

#[test]
fn an_invisible_stalker_is_never_seen_however_lit_and_near() {
    let (mut w, stalker) = room_with('i', Pos::new(2, 1), 255);
    assert!(!shows(&mut w, stalker), "beside the player, in full light");
    // Seeing shows it.
    let player = w.state.player;
    afflict(&mut w, player, "seeing");
    assert!(shows(&mut w, stalker));
}

#[test]
fn the_invisible_status_hides_a_duergar_in_full_light() {
    let (mut w, duergar) = room_with('d', Pos::new(5, 2), 255);
    hold(&mut w, duergar);
    assert!(shows(&mut w, duergar));
    afflict(&mut w, duergar, "invisible");
    assert!(!shows(&mut w, duergar));
}

#[test]
fn blood_scent_and_a_blow_that_found_it_show_the_unseen() {
    let (mut w, stalker) = room_with('i', Pos::new(5, 2), 255);
    let dim = |_: Pos| false;
    let unseen = |w: &World, scented: &[ActorId]| {
        unseen_by_player(&w.state, &w.content, &dim, scented)
            .iter()
            .any(|c| c.id == stalker)
    };
    assert!(unseen(&w, &[]));
    assert!(!unseen(&w, &[stalker]), "Blood scent finds it");
    let turn = w.state.turn;
    w.state.actors.get_mut(&stalker).expect("it").mind.glimpsed = Some(turn);
    assert!(!unseen(&w, &[]), "the turn a blow found it");
    w.state.turn += 2;
    assert!(unseen(&w, &[]), "and gone again after");
}

#[test]
fn the_companions_nose_finds_the_unseen() {
    let (mut w, stalker) = room_with('i', Pos::new(6, 2), 255);
    hold(&mut w, stalker);
    run(&mut w, "ally:hound");
    let hound = w.allies_of(w.state.player)[0];
    let snel = CompanionData {
        name: "Snel".into(),
        grade: 1,
        scent: None,
        quarry: None,
    };
    (w.state.actors.get_mut(&hound).and_then(|a| a.ally.as_mut()))
        .expect("an ally")
        .source = AllySource::Companion(snel);
    put(&mut w, hound, Pos::new(1, 4));
    let nose = w.content().creature(w.state.actors[&hound].creature).nose;
    assert!(nose.is_some_and(|n| n >= 5), "a hound's nose");
    assert!(shows(&mut w, stalker), "within its nose");
    put(&mut w, stalker, Pos::new(9, 1));
    put(&mut w, hound, Pos::new(1, 5));
    assert!(!shows(&mut w, stalker), "beyond it");
}

// ---- striking at the unseen ----------------------------------------------------------------

#[test]
fn a_blow_at_the_unseen_is_at_minus_four_and_a_hit_shows_it_for_the_turn() {
    let (mut w, stalker) = room_with('i', Pos::new(2, 1), 255);
    let player = w.state.player;
    let profile = w.state.player().melee(&w.content);
    let at = w.at_the_unseen(player, stalker, profile.clone());
    assert_eq!(at.to_hit, profile.to_hit + unseen::UNSEEN_TO_HIT);
    // Its blows are something's; striking back is at something, until one finds it.
    for _ in 0..60 {
        let from = w.messages().len();
        run(&mut w, "move:E");
        let now: Vec<String> = (w.messages().iter().skip(from))
            .map(|m| m.display())
            .collect();
        assert_eq!(
            now.first().map(String::as_str),
            Some("You strike at something.")
        );
        let found = w.state.actors.get(&stalker).and_then(|a| a.mind.glimpsed);
        if found == Some(w.state.turn - 1) {
            assert!(w.perception().actors().iter().any(|a| a.id == stalker));
            hold(&mut w, stalker);
            run(&mut w, "wait");
            assert!(w.perception().actors().iter().all(|a| a.id != stalker));
            return;
        }
        assert!(w.perception().actors().iter().all(|a| a.id != stalker));
    }
    panic!("no blow found it: {:?}", said(&w));
}

#[test]
fn an_unseen_blow_is_something_and_heard_where_it_came_from() {
    let (mut w, stalker) = room_with('i', Pos::new(2, 1), 255);
    for _ in 0..20 {
        let from = w.messages().len();
        run(&mut w, "wait");
        let now: Vec<String> = (w.messages().iter().skip(from))
            .map(|m| m.display())
            .collect();
        if now.iter().any(|m| m.starts_with("Something ")) {
            let at = w.state.actors[&stalker].pos;
            let sounds = w.perception().sounds().to_vec();
            assert!(
                sounds
                    .iter()
                    .any(|s| s.kind == SoundKind::Fighting && s.pos == at),
                "{sounds:?}"
            );
            return;
        }
    }
    panic!("it never struck: {:?}", said(&w));
}

// ---- the self-buff step --------------------------------------------------------------------

#[test]
fn a_duergar_fades_with_no_one_in_sight_and_not_again_until_its_cooldown_is_off() {
    let mut w = world_with(ROOM, 255);
    let duergar = only_creature(&w);
    assert!(!compute(&mut w).shows(duergar), "out of sight");
    run(&mut w, "wait");
    assert!(has(&w, duergar, "invisible") && has(&w, duergar, "invisibility"));
    let faded = w.state.time;
    let mut again = None;
    let mut gone = None;
    for _ in 0..120 {
        run(&mut w, "wait");
        if gone.is_none() && !has(&w, duergar, "invisible") {
            gone = Some(w.state.time);
        }
        if gone.is_some() && has(&w, duergar, "invisible") {
            again = Some(w.state.time);
            break;
        }
    }
    let lasted = gone.expect("it wears off").since(faded).as_seconds();
    assert!((114..=132).contains(&lasted), "two minutes: {lasted}");
    let between = again.expect("it fades again").since(faded).as_seconds();
    assert!((294..=320).contains(&between), "five minutes: {between}");
}

#[test]
fn a_duergar_fades_and_fights_in_the_same_turn_and_is_said_to() {
    let (mut w, duergar) = room_with('d', Pos::new(2, 1), 255);
    let from = w.messages().len();
    run(&mut w, "wait");
    let now: Vec<String> = (w.messages().iter().skip(from))
        .map(|m| m.display())
        .collect();
    assert!(
        now.iter()
            .any(|m| m == "The duergar warrior fades from sight."),
        "{now:?}"
    );
    assert!(has(&w, duergar, "invisible"));
    // It went on to strike: something swings at the player in the same turn.
    let struck = now.iter().any(|m| m.starts_with("Something "));
    assert!(struck, "{now:?}");
}

#[test]
fn a_silenced_duergar_does_not_fade() {
    let (mut w, duergar) = room_with('d', Pos::new(6, 2), 255);
    afflict(&mut w, duergar, "silenced");
    run(&mut w, "wait");
    assert!(!has(&w, duergar, "invisible"));
}

#[test]
fn you_can_see_it_again_as_it_ends() {
    let (mut w, duergar) = room_with('d', Pos::new(6, 2), 255);
    hold(&mut w, duergar);
    afflict(&mut w, duergar, "invisible");
    let spent = status(&w, "invisibility");
    let content = w.content.clone();
    (w.state.actors.get_mut(&duergar).expect("it")).effects.add(
        spent,
        content.status(spent),
        100,
        None,
    );
    run(&mut w, "wait");
    hold(&mut w, duergar);
    run(&mut w, "rest:3m");
    assert!(
        said(&w)
            .iter()
            .any(|m| m == "You can see the duergar warrior again."),
        "{:?}",
        said(&w)
    );
}

// ---- the rest ------------------------------------------------------------------------------

#[test]
fn a_will_o_wisp_sheds_no_light_while_invisible() {
    let (mut w, wisp) = room_with('w', Pos::new(6, 2), 0);
    hold(&mut w, wisp);
    let near = Pos::new(5, 2);
    assert!(compute(&mut w).is_visible(near), "its light shows the cell");
    afflict(&mut w, wisp, "invisible");
    assert!(!compute(&mut w).is_visible(near), "out with it");
}

#[test]
fn the_stalker_hunts_the_player_out_of_its_sight() {
    let (mut w, stalker) = room_with('i', Pos::new(18, 2), 255);
    let player = w.state.player;
    let far = |w: &World| {
        w.state.actors[&stalker]
            .pos
            .chebyshev(w.state.actors[&player].pos)
    };
    let start = far(&w);
    run(&mut w, "wait*6");
    assert!(far(&w) < start, "it comes: {} from {start}", far(&w));
}

#[test]
fn the_dev_afflict_gives_the_player_invisibility() {
    let (mut w, _) = room_with('d', Pos::new(18, 5), 255);
    run(&mut w, "afflict:invisible");
    let player = w.state.player;
    assert!(has(&w, player, "invisible"));
    assert!(said(&w).iter().any(|m| m == "You fade from sight."));
}

#[test]
fn the_roster_rows() {
    let content = BUILTIN.clone();
    let def = |id: &str| content.creature(content.creature_id(id).expect(id));
    for id in [
        "shadow",
        "greater_shadow",
        "deathshade",
        "darkling",
        "sluagh",
    ] {
        assert!(def(id).shadowy, "{id} is shadowy");
    }
    for id in [
        "duergar_warrior",
        "duergar_veteran",
        "duergar_thane",
        "duergar_miner",
        "duergar_crossbowman",
        "duergar_slinger",
        "duergar_spider_rider",
        "duergar_hulk",
        "duergar_smith",
        "duergar_king",
        "duergar_priest",
        "duergar_mindbreaker",
        "imp",
        "quasit",
        "will_o_wisp",
    ] {
        assert!(
            matches!(
                def(id).invisible,
                Some(Invisible::Fades { cooldown: 300, .. })
            ),
            "{id} fades on a five-minute cooldown"
        );
    }
    assert!(
        def("duergar_skeleton").invisible.is_none(),
        "the dead do not"
    );
    let stalker = def("invisible_stalker");
    assert_eq!(stalker.invisible, Some(Invisible::Innate));
    assert!(stalker.hunts && !stalker.lurks && stalker.dread);
    assert_eq!((stalker.level, stalker.speed), (22, 1500));
    assert!(!stalker.corpse && stalker.blood.is_none() && stalker.stray);
    let attack = stalker.attack.as_ref().expect("its wind");
    assert_eq!(attack.attack.damage.to_string(), "3d8");
    // The wisp's touch sets mana ebbing.
    let ebbing = content.status_id("ebbing").expect("ebbing");
    let wisp = def("will_o_wisp").attack.as_ref().expect("its spark");
    assert!((wisp.on_hit.iter()).any(|h| matches!(
        h.what,
        crate::content::Affliction::Status { status, .. } if status == ebbing
    )));
}
