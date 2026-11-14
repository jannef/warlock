//! Decoration the state hash leaves out (spec §9; `world/turn/hash.rs`): a field marked with one of
//! these in `skip_serializing_if` is skipped while [`crate::world::WorldState::hash`] serializes
//! the state, and when empty; saves keep it. Its own module, so plain data outside the world (a
//! [`crate::statue::Statue`]) can mark a field too.

use std::cell::Cell;

thread_local! {
    /// Whether the state is being serialized for [`crate::world::WorldState::hash`] on this
    /// thread.
    pub(crate) static HASHING: Cell<bool> = const { Cell::new(false) };
}

/// For `skip_serializing_if` on decoration the state hash leaves out (the graveyard's names,
/// `world/fixtures/graveyard.rs`): skipped while hashing, and when empty; saves keep it.
pub(crate) fn unhashed_vec<T>(items: &[T]) -> bool {
    items.is_empty() || HASHING.with(Cell::get)
}

/// [`unhashed_vec`] for an optional value.
pub(crate) fn unhashed_option<T>(value: &Option<T>) -> bool {
    value.is_none() || HASHING.with(Cell::get)
}

/// [`unhashed_vec`] for a map (a level's gravestones, `world/fixtures/graves.rs`).
pub(crate) fn unhashed_map<K, V>(map: &std::collections::BTreeMap<K, V>) -> bool {
    map.is_empty() || HASHING.with(Cell::get)
}
