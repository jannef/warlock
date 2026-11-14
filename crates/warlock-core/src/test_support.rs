//! Helpers shared by the core's tests (#1509): the built-in content, loaded once per test binary,
//! and action text applied.

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

/// Applies action text (`move:N wait*3`) to `world`, returning each action's outcome.
pub(crate) fn run(world: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|action| world.apply(action))
        .collect()
}
