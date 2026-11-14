//! Helpers shared by the core's tests (#1509): the built-in content, loaded once per test binary,
//! action text applied, and the message log read back.
//!
//! The message readers come in two forms that differ when a line repeats: [`said`] and [`last`]
//! read a message with its counted repeats ([`Message::display`], `... (×2)`), [`texts`],
//! [`last_text`], and [`said_text`] its raw text.
//!
//! [`Message::display`]: crate::messages::Message::display

use crate::action::{Outcome, parse_actions};
use crate::content::Content;
use crate::world::World;
use std::sync::{Arc, LazyLock};

/// The built-in content (`assets/`), loaded once for every test that shares it.
pub(crate) static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("built-in content")));

/// The shared built-in content ([`CONTENT`]).
pub(crate) fn builtin() -> Arc<Content> {
    CONTENT.clone()
}

/// Scenario `scenario` of the built-in content, at its own seed.
pub(crate) fn world(scenario: &str) -> World {
    World::new(builtin(), scenario, None).expect("world")
}

/// Applies action text (`move:N wait*3`) to `world`, returning each action's outcome.
pub(crate) fn run(world: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|action| world.apply(action))
        .collect()
}

/// Every message in the log, with its counted repeats.
pub(crate) fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

/// Every message's raw text.
pub(crate) fn texts(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.text.clone()).collect()
}

/// The last message with its counted repeats, or nothing.
pub(crate) fn last(world: &World) -> String {
    (world.messages().iter().last()).map_or_else(String::new, |m| m.display())
}

/// The last message's raw text, or nothing.
pub(crate) fn last_text(world: &World) -> String {
    (world.messages().iter().last()).map_or_else(String::new, |m| m.text.clone())
}

/// Whether any message's raw text is exactly `text`.
pub(crate) fn said_text(world: &World, text: &str) -> bool {
    world.messages().iter().any(|m| m.text == text)
}
