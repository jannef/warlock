//! Grimoires in the read list (spec §8 "Spells": "Learning"; #1260): listed with the scrolls, a
//! grimoire whose spell could be learned tells the chance in words, and choosing it asks in the
//! one dialog; one that cannot teach (the spell known, no Spellcraft) is read at once, which says
//! why.

use super::screen_snapshots::{BASE, Case, press_keys, start, text};
use crate::game::Game;
use crate::render::console::Console;
use bevy::prelude::App;
use std::sync::Arc;
use warlock_core::content::{Content, ContentFiles};

/// A quiet lit hall, the player a human woman of `class` with `carry` in the pack (as a
/// scenario's `carry`).
pub(super) fn hall(carry: &str, class: &str) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/hall.toml",
        "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n#######\n#.....#\n#..@..#\n\
         #.....#\n#######\n'''\n[legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
    );
    files.insert(
        "scenarios/hall.toml",
        &format!(
            "name = \"Hall\"\nstart_level = \"hall\"\ncarry = {carry}\n\
             character = {{ race = \"human\", gender = \"female\", class = \"{class}\", name = \"Ada\" }}\n"
        ),
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
}

/// The pack: a grimoire of light (a cantrip), one of fireball (level 11), a scroll of frost.
const PACK: &str = "[\"grimoire_of_light\", \"grimoire_of_fireball\", \"scroll_of_frost\"]";

/// The hall with [`PACK`] for `class`, the keys pressed.
fn reading(class: &str, keys: &str) -> App {
    let case = Case {
        scenario: "hall",
        run: true,
        ..BASE
    };
    let mut app = start(&case, hall(PACK, class));
    press_keys(&mut app, keys);
    app
}

fn screen(app: &App) -> String {
    text(app.world().resource::<Console>())
}

fn last(app: &App) -> String {
    let game = app.world().resource::<Game>();
    (game.world.messages().iter().last())
        .map(|m| m.display())
        .unwrap_or_default()
}

#[test]
fn the_read_list_tells_each_grimoires_chance() {
    let app = reading("wizard", "r");
    let shown = screen(&app);
    assert!(shown.contains("(learning: certain)"), "{shown}");
    insta::assert_snapshot!("grimoire-read-list", shown);
}

#[test]
fn choosing_a_grimoire_asks_first() {
    let app = reading("wizard", "r a");
    let shown = screen(&app);
    assert!(shown.contains("Read the grimoire of light?"), "{shown}");
    insta::assert_snapshot!("grimoire-read-dialog", shown);
}

#[test]
fn without_spellcraft_a_grimoire_is_read_at_once_and_refused() {
    let app = reading("warrior", "r");
    let shown = screen(&app);
    assert!(!shown.contains("(learning:"), "{shown}");
    insta::assert_snapshot!("grimoire-read-list-untrained", shown);
    let app = reading("warrior", "r a");
    assert_eq!(last(&app), warlock_core::world::UNTRAINED);
}
