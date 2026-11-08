//! Stable state hashing for replays and regression checks (spec §9).

use super::{World, WorldState};
use std::cell::Cell;

thread_local! {
    /// Whether the state is being serialized for [`WorldState::hash`] on this thread.
    static HASHING: Cell<bool> = const { Cell::new(false) };
}

/// For `skip_serializing_if` on decoration the state hash leaves out (the graveyard's names,
/// `world/graveyard.rs`): skipped while hashing, and when empty; saves keep it.
pub(crate) fn unhashed_vec<T>(items: &[T]) -> bool {
    items.is_empty() || HASHING.with(Cell::get)
}

/// [`unhashed_vec`] for an optional value.
pub(crate) fn unhashed_option<T>(value: &Option<T>) -> bool {
    value.is_none() || HASHING.with(Cell::get)
}

/// [`unhashed_vec`] for a map (a level's gravestones, `world/graves.rs`).
pub(crate) fn unhashed_map<K, V>(map: &std::collections::BTreeMap<K, V>) -> bool {
    map.is_empty() || HASHING.with(Cell::get)
}

/// Clears [`HASHING`] when dropped, so a panic while hashing leaves it as it was.
struct Hashing;

impl Drop for Hashing {
    fn drop(&mut self) {
        HASHING.with(|h| h.set(false));
    }
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// FNV-1a 64-bit. Unlike `std::hash::DefaultHasher` it is specified, so hashes are identical
/// across platforms and Rust versions.
pub(crate) fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(FNV_OFFSET, |hash, &b| {
        (hash ^ u64::from(b)).wrapping_mul(FNV_PRIME)
    })
}

impl WorldState {
    /// FNV-1a over the canonical JSON encoding of the state. All maps are `BTreeMap`s, so the
    /// encoding is deterministic. Decoration marked [`unhashed_vec`], [`unhashed_option`], or
    /// [`unhashed_map`] is left out.
    pub fn hash(&self) -> u64 {
        HASHING.with(|h| h.set(true));
        let _reset = Hashing;
        let json = serde_json::to_vec(self).expect("world state serializes to JSON");
        fnv1a(&json)
    }
}

impl World {
    /// Hash of the persistent simulation state (not of derived perception).
    pub fn state_hash(&self) -> u64 {
        self.state.hash()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_matches_reference_vectors() {
        assert_eq!(fnv1a(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a(b"foobar"), 0x8594_4171_f739_67e8);
    }
}
