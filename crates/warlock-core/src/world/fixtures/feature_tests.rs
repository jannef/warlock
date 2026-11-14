//! A feature of the ground the player arrives on is named (`world/fixtures/feature.rs`): once per arrival,
//! before what lies there, never again while they stay; ordinary ground says nothing. One that
//! cannot be stood on (a statue) is named, with its place's note, on arriving beside it.

use crate::action::Rest;
use crate::content::ContentFiles;
use crate::test_support::builtin;
use crate::time::Duration;
use crate::world::*;

/// A hall with an altar (`A`, in an echoing room), a faint rune (`R`), an altar with a chalice
/// on it (`C`), and stairs down.
const TOP: &str = "\
##########
#@.A.R..>#
#......C.#
##########
";

/// The level below: stairs up beside an altar.
const BELOW: &str = "\
#####
#<.A#
#####
";

const ALTAR: &str = "There is an altar here.";
const RUNE: &str = "There is a faint rune here.";

fn world_with(start: &str) -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_top.toml",
        &format!(
            "name = \"Top\"\nambient = [120, 120, 120, 0]\nmap = '''\n{TOP}'''\n\
             [[room]]\neffect = \"echoing\"\narea = [3, 0, 3, 3]\n[legend]\n\
             \"A\" = \"altar\"\n\"R\" = \"uv_rune\"\n\
             \"C\" = {{ terrain = \"altar\", objects = [\"silver_chalice\"] }}\n\
             \"@\" = {{ terrain = \"{start}\", spawn = \"player\" }}\n\
             \">\" = {{ terrain = \"stairs_down\", to = \"test_below\" }}\n"
        ),
    );
    files.insert(
        "levels/test_below.toml",
        &format!(
            "name = \"Below\"\nambient = [120, 120, 120, 0]\nmap = '''\n{BELOW}'''\n\
             [legend]\n\"A\" = \"altar\"\n\
             \"<\" = {{ terrain = \"stairs_up\", to = \"test_top\" }}\n"
        ),
    );
    files.insert(
        "scenarios/test_feature.toml",
        "name = \"T\"\nstart_level = \"test_top\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_feature", None).expect("world")
}

/// How many times `line` was said.
fn times(w: &World, line: &str) -> usize {
    w.messages().iter().filter(|m| m.text == line).count()
}

fn step(w: &mut World, n: usize) {
    for _ in 0..n {
        w.apply(&Action::Move(Dir::E));
    }
}

#[test]
fn stepping_onto_an_altar_names_it_once() {
    let mut w = world_with("floor");
    let before = w.messages().len();
    step(&mut w, 1);
    assert_eq!(w.messages().len(), before, "plain floor says nothing");
    step(&mut w, 1);
    let texts: Vec<&str> = w.messages().iter().map(|m| m.text.as_str()).collect();
    let room = texts
        .iter()
        .position(|t| *t == "Your footsteps echo loudly.");
    let altar = texts.iter().position(|t| *t == ALTAR);
    assert!(room.is_some() && altar.is_some(), "{texts:?}");
    w.apply(&Action::Wait);
    w.apply(&Action::Search(5));
    w.apply(&Action::Rest(Rest::For(Duration::minutes(10))));
    assert_eq!(times(&w, ALTAR), 1, "staying there says nothing more");
    // Off and back on: a new arrival.
    step(&mut w, 1);
    w.apply(&Action::Move(Dir::W));
    assert_eq!(times(&w, ALTAR), 2);
}

#[test]
fn an_altar_is_named_before_what_lies_on_it() {
    let mut w = world_with("floor");
    step(&mut w, 5);
    w.apply(&Action::Move(Dir::SE));
    let texts: Vec<&str> = w.messages().iter().map(|m| m.text.as_str()).collect();
    let n = texts.len();
    assert_eq!(
        texts[n - 2..],
        [ALTAR, "You see here a silver chalice."],
        "{texts:?}"
    );
}

#[test]
fn stepping_onto_a_faint_rune_names_it_once() {
    let mut w = world_with("floor");
    step(&mut w, 4);
    assert_eq!(times(&w, RUNE), 1);
    w.apply(&Action::Wait);
    assert_eq!(times(&w, RUNE), 1);
}

#[test]
fn a_run_starting_on_an_altar_names_it_once() {
    let mut w = world_with("altar");
    assert_eq!(times(&w, ALTAR), 1);
    w.apply(&Action::Wait);
    assert_eq!(times(&w, ALTAR), 1);
}

#[test]
fn arriving_on_another_level_on_an_altar_names_it() {
    let mut w = world_with("floor");
    step(&mut w, 7);
    w.apply(&Action::Descend);
    assert_eq!(w.state.current_level, "test_below");
    let altars = times(&w, ALTAR);
    // Carried back up onto the top level's altar (as a fall or a teleport would).
    w.state.current_level = "test_top".into();
    let id = w.state.player;
    w.state.actors.set_level(id, "test_top");
    let player = w.player_mut();
    player.pos = Pos::new(3, 1);
    w.apply(&Action::Wait);
    assert_eq!(times(&w, ALTAR), altars + 1);
}

/// A hall with a statue (`S`) with a note, the player north-west of it.
const HALL: &str = "\
#######
#.@...#
#..S..#
#.....#
#######
";

const STATUE: &str = "There is a worn statue here. Stone, old, and worn smooth where hands have \
                      touched it. Letters say: hello.";

fn statue_world() -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_hall.toml",
        &format!(
            "name = \"Hall\"\nambient = [120, 120, 120, 0]\nmap = '''\n{HALL}'''\n[legend]\n\
             \"S\" = {{ terrain = \"statue\", name = \"a figure\", note = \"Letters say: hello.\" }}\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n"
        ),
    );
    files.insert(
        "scenarios/test_statue.toml",
        "name = \"T\"\nstart_level = \"test_hall\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_statue", None).expect("world")
}

#[test]
fn a_statue_is_named_with_its_note_on_arriving_beside_it_once() {
    let mut w = statue_world();
    // The run starts beside it.
    assert_eq!(times(&w, STATUE), 1);
    // Moving from beside it to beside it says nothing more; nor does waiting, nor bumping it.
    w.apply(&Action::Move(Dir::E));
    w.apply(&Action::Move(Dir::S));
    w.apply(&Action::Move(Dir::E));
    w.apply(&Action::Wait);
    assert_eq!(times(&w, STATUE), 1);
    // Off and back beside it: a new arrival.
    w.apply(&Action::Move(Dir::E));
    w.apply(&Action::Move(Dir::W));
    assert_eq!(times(&w, STATUE), 2);
    // Look mode reads the same note.
    let place = w.perception().place(Pos::new(3, 2)).expect("named");
    assert_eq!(place.note.as_deref(), Some("Letters say: hello."));
}

/// On Fyrdelf's eleventh floor (a generated level), arriving beside the hunter's statue reads
/// its inscription: the vault's note is stamped into the level's state, and look mode reads it.
#[test]
fn the_hunters_statue_on_fyrdelf_reads_its_inscription() {
    let content = builtin();
    let mut w = World::new(Arc::clone(&content), "fyrdelf", Some(1)).expect("world");
    let id = "fyrdelf:11";
    let template =
        crate::mapgen::generate(&content, ("ascmearc", w.state.seed), id, None).expect("made");
    let statue = content.terrain_id("statue").expect("statue");
    let (at, _) = (template.terrain.iter())
        .find(|(_, t)| **t == statue)
        .expect("the statue");
    crate::world::levels::travel::instantiate_level(&mut w.state, &content, &template);
    let beside = (Dir::ALL.into_iter())
        .map(|d| at.step(d))
        .find(|&p| w.state.actor_at(id, p).is_none())
        .expect("a free cell beside it");
    w.state.current_level = id.into();
    let me = w.state.player;
    w.state.actors.set_level(me, id);
    let player = w.player_mut();
    player.pos = beside;
    w.apply(&Action::Wait);
    let said: Vec<&str> = (w.messages().iter())
        .map(|m| m.text.as_str())
        .filter(|t| t.starts_with("There is a worn statue here."))
        .collect();
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].contains("LĪF ÆR DĒAÞE"), "{}", said[0]);
    let place = w.perception().place(at).expect("look mode names it");
    assert_eq!(place.name, "a hooded figure");
}
