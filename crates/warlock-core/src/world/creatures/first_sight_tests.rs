//! First-sight lines (`first_sight.rs`): said once a run per type, the first time one is seen,
//! and remembered in the save.

use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::world::*;

// A lit hall split by a door: the player, a pixie in sight, and a second pixie and a nymph
// beyond the door.
const HALL: &str = "
###############
#.@..p..#..pn.#
#.......+.....#
###############
";

fn world() -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/sight_hall.toml",
        &format!(
            "name = \"Sight hall\"\nambient = [255, 255, 255, 0]\nmap = '''{HALL}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"p\" = {{ terrain = \"floor\", creature = \"pixie\", faction = \"peaceful\" }}\n\
             \"n\" = {{ terrain = \"floor\", creature = \"nymph\", faction = \"peaceful\" }}\n"
        ),
    );
    files.insert(
        "scenarios/sight_hall.toml",
        "name = \"Sight hall\"\nstart_level = \"sight_hall\"\nhp = 1000\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "sight_hall", None).expect("world")
}

fn said(w: &World, line: &str) -> usize {
    (w.messages().iter()).filter(|m| m.text == line).count()
}

const PIXIE: &str = "A pixie. When it laughs, your pack is lighter.";
const NYMPH: &str = "A nymph. Lovely, and light-fingered; mind your pack.";

#[test]
fn a_dread_type_seen_is_named_once_a_run_and_the_save_remembers() {
    let mut w = world();
    assert_eq!(
        said(&w, PIXIE),
        0,
        "said by the turn's perception, not before"
    );
    w.apply(&Action::Wait);
    assert_eq!(said(&w, PIXIE), 1, "the pixie in sight");
    assert_eq!(said(&w, NYMPH), 0, "the nymph is behind the door");
    assert!(w.state.progress.sighted.contains("pixie"));
    for action in parse_actions("wait*2 move:E*4 move:SE open:E move:E").expect("actions") {
        w.apply(&action);
    }
    assert_eq!(
        said(&w, PIXIE),
        1,
        "never again, for the second pixie either"
    );
    assert_eq!(said(&w, NYMPH), 1, "the nymph once seen");
    let loaded =
        crate::save::decode(&crate::save::encode(&w), w.content.clone()).expect("the save");
    let sighted: Vec<&str> = loaded
        .state
        .progress
        .sighted
        .iter()
        .map(String::as_str)
        .collect();
    assert_eq!(sighted, ["nymph", "pixie"]);
}
