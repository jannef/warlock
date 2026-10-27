//! An object list left open while the run changes reads it again (#591): the study list stays
//! beneath the dialog that studies, the scroll studied is used up whether the study fails or
//! not, and the stacks after it move up a position. The list then shows what is carried now, so
//! its letters never name another stack (the owner's report: the next pick studied a wax candle).

use super::screen_snapshots::{BASE, Case, press_keys, start, text};
use crate::game::Game;
use crate::render::console::Console;
use bevy::prelude::App;
use std::sync::Arc;
use warlock_core::content::{Content, ContentFiles};

/// A quiet lit hall, the player with mana and `carry` in the pack (as a scenario's `carry`).
fn hall(carry: &str) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/hall.toml",
        "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n#######\n#.....#\n#..@..#\n\
         #.....#\n#######\n'''\n[legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
    );
    files.insert(
        "scenarios/hall.toml",
        &format!("name = \"Hall\"\nstart_level = \"hall\"\nhp = 500\nmana = 10\ncarry = {carry}\n"),
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
}

/// The hall, every scroll type known, nothing open yet.
fn studying(carry: &str) -> App {
    let case = Case {
        scenario: "hall",
        actions: "identify",
        ..BASE
    };
    start(&case, hall(carry))
}

fn said(app: &App) -> Vec<String> {
    let game = app.world().resource::<Game>();
    (game.world.messages().iter())
        .map(|m| m.display())
        .collect()
}

fn screen(app: &App) -> String {
    text(app.world().resource::<Console>())
}

/// The owner's case: the one scroll studied, the wax candles after it. The list afterwards is
/// empty and says so, and picking its old letter again studies nothing.
#[test]
fn the_last_scroll_studied_leaves_the_list_empty() {
    let mut app = studying("[\"scroll_of_fireball\", \"candle*3\"]");
    press_keys(&mut app, "L");
    assert!(
        screen(&app).contains("a - ? a scroll of fireball"),
        "{}",
        screen(&app)
    );
    press_keys(&mut app, "a y");
    let shown = screen(&app);
    assert!(!shown.contains("scroll of fireball ("), "{shown}");
    insta::assert_snapshot!("scroll-study-used-up", shown);
    press_keys(&mut app, "a y");
    let messages = said(&app);
    assert_eq!(
        (messages.iter())
            .filter(|m| m.contains("You begin to study"))
            .count(),
        1,
        "{messages:?}"
    );
    assert!(
        !messages.iter().any(|m| m.contains("candle")),
        "a stale letter studied the candles: {messages:?}"
    );
}

/// Two scrolls: after the first is used up, the list's `a` is the second, and choosing it
/// studies that one, at its new position.
#[test]
fn the_list_after_a_study_names_what_is_carried_now() {
    let mut app = studying("[\"scroll_of_fireball\", \"scroll_of_frost\", \"candle*3\"]");
    press_keys(&mut app, "L a y");
    let shown = screen(&app);
    assert!(shown.contains("a - ? a scroll of frost"), "{shown}");
    assert!(!shown.contains("scroll of fireball ("), "{shown}");
    press_keys(&mut app, "a y");
    let messages = said(&app);
    assert!(
        (messages.iter()).any(|m| m == "You begin to study the scroll of frost."),
        "{messages:?}"
    );
    assert!(
        !messages.iter().any(|m| m.contains("candle")),
        "{messages:?}"
    );
}

/// Presses typed after a study in the same frame never reach the list before it is read again:
/// they are dropped, so one study is made.
#[test]
fn presses_after_a_study_in_the_same_frame_are_dropped() {
    let mut app = studying("[\"scroll_of_fireball\", \"candle*3\"]");
    press_keys(&mut app, "L a y a y");
    let messages = said(&app);
    assert_eq!(
        (messages.iter())
            .filter(|m| m.contains("You begin to study"))
            .count(),
        1,
        "{messages:?}"
    );
    assert!(
        !messages.iter().any(|m| m.contains("candle")),
        "{messages:?}"
    );
}
