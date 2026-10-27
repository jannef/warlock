//! A feature of the ground the player arrives on is named (`world/feature.rs`): once per arrival,
//! before what lies there, never again while they stay; ordinary ground says nothing.

use super::*;
use crate::action::Rest;
use crate::content::ContentFiles;
use crate::time::Duration;

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
    let player = w.player_mut();
    player.level = "test_top".into();
    player.pos = Pos::new(3, 1);
    w.apply(&Action::Wait);
    assert_eq!(times(&w, ALTAR), altars + 1);
}

#[test]
fn only_walkable_terrain_may_announce() {
    let mut files = ContentFiles::builtin();
    let terrain = include_str!("../../../../assets/data/terrain.toml");
    files.insert(
        "data/terrain.toml",
        &format!(
            "{terrain}\n[[terrain]]\nid = \"test_wall_feature\"\nname = \"carved wall\"\nannounce = true\n\
         walkable = false\ntransparent = false\nglyph = \"#\"\nfg = \"#ffffff\"\n"
        ),
    );
    let err = Content::load(&files).expect_err("refused").to_string();
    assert!(
        err.contains("only walkable terrain may `announce`"),
        "{err}"
    );
}
