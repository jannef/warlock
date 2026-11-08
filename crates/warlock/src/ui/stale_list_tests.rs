//! An object list left open while the run changes reads it again (#591): the read list stays
//! beneath the dialog that reads a grimoire, and shows what is carried and known now, so its
//! letters never name another stack (the owner's report: the next pick studied a wax candle) and
//! its marks never tell a chance that is gone.

use super::grimoire_tests::hall;
use super::screen_snapshots::{BASE, Case, press_keys, start, text};
use crate::game::Game;
use crate::render::console::Console;
use bevy::prelude::App;

/// The hall, the player a wizard (trained in Spellcraft) carrying `carry`, nothing open yet.
fn reading(carry: &str) -> App {
    let case = Case {
        scenario: "hall",
        run: true,
        ..BASE
    };
    start(&case, hall(carry, "wizard"))
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

fn begun(app: &App) -> usize {
    (said(app).iter())
        .filter(|m| m.contains("You begin to read"))
        .count()
}

/// A cantrip learned: the list beneath is read again and tells no chance any more, and picking
/// the grimoire again reads nothing.
#[test]
fn the_list_after_a_grimoire_is_read_tells_what_is_known_now() {
    let mut app = reading("[\"grimoire_of_light\", \"candle*3\"]");
    press_keys(&mut app, "r");
    assert!(
        screen(&app).contains("a - ? a grimoire of light (learning: certain)"),
        "{}",
        screen(&app)
    );
    press_keys(&mut app, "a y");
    let shown = screen(&app);
    assert!(shown.contains("a - ? a grimoire of light"), "kept: {shown}");
    assert!(!shown.contains("(learning:"), "{shown}");
    insta::assert_snapshot!("grimoire-read-list-after", shown);
    press_keys(&mut app, "a");
    let messages = said(&app);
    assert_eq!(begun(&app), 1, "{messages:?}");
    assert_eq!(
        messages.last().map(String::as_str),
        Some("You know this spell already.")
    );
    assert!(
        !messages.iter().any(|m| m.contains("candle")),
        "a stale letter read the candles: {messages:?}"
    );
}

/// Presses typed after a read in the same frame never reach the list before it is read again:
/// they are dropped, so one read is made.
#[test]
fn presses_after_a_read_in_the_same_frame_are_dropped() {
    let mut app = reading("[\"grimoire_of_light\", \"candle*3\"]");
    press_keys(&mut app, "r a y a y");
    let messages = said(&app);
    assert_eq!(begun(&app), 1, "{messages:?}");
    assert!(
        !messages.iter().any(|m| m.contains("candle")),
        "{messages:?}"
    );
}
