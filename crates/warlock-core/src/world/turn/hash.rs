//! Stable state hashing for replays and regression checks (spec §9).

use crate::world::{World, WorldState};

pub(crate) use crate::unhashed::{HASHING, unhashed_map, unhashed_vec};

/// Clears [`HASHING`] when dropped, so a panic while hashing leaves it as it was.
struct Hashing;

impl Drop for Hashing {
    fn drop(&mut self) {
        HASHING.with(|h| h.set(false));
    }
}

/// Serializes the levels (`WorldState::levels`): while hashing, each as it would be with no
/// graveyard (`LevelState::unburied`, `fixtures/graves.rs`), so the run's dead stay out of the hash.
pub(crate) fn levels<S: serde::Serializer>(
    levels: &std::collections::BTreeMap<String, crate::world::LevelState>,
    s: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeMap;
    if !HASHING.with(std::cell::Cell::get) {
        return s.collect_map(levels);
    }
    let mut map = s.serialize_map(Some(levels.len()))?;
    for (id, level) in levels {
        map.serialize_entry(id, &*level.unburied())?;
    }
    map.end()
}

pub(crate) const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// An FNV-1a hasher as an [`std::io::Write`], so a serializer streams into it with no buffer.
struct Fnv(u64);

impl std::io::Write for Fnv {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = fnv1a_fold(self.0, bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// FNV-1a 64-bit continued from `hash` over `bytes` (`rng.rs` writes stream keys with it).
pub(crate) fn fnv1a_fold(hash: u64, bytes: &[u8]) -> u64 {
    bytes.iter().fold(hash, |hash, &b| {
        (hash ^ u64::from(b)).wrapping_mul(FNV_PRIME)
    })
}

/// FNV-1a 64-bit. Unlike `std::hash::DefaultHasher` it is specified, so hashes are identical
/// across platforms and Rust versions.
pub(crate) fn fnv1a(bytes: &[u8]) -> u64 {
    fnv1a_fold(FNV_OFFSET, bytes)
}

impl WorldState {
    /// FNV-1a over the canonical JSON encoding of the state, streamed into the hasher (no buffer
    /// of the whole state). All maps are `BTreeMap`s, so the encoding is deterministic.
    /// Decoration marked [`unhashed_vec`], [`unhashed_option`], or [`unhashed_map`] is left out.
    /// The hash therefore also rests on `serde_json`'s output and `rand_chacha`'s serde format
    /// (`docs/dev/merging.md`, "Random streams").
    pub fn hash(&self) -> u64 {
        HASHING.with(|h| h.set(true));
        let _reset = Hashing;
        let mut hasher = Fnv(FNV_OFFSET);
        serde_json::to_writer(&mut hasher, self).expect("world state serializes to JSON");
        hasher.0
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

    /// Streaming the state into the hasher hashes exactly the bytes of its JSON encoding, so
    /// every recorded replay hash stays as it was.
    #[test]
    fn streamed_hash_is_the_hash_of_the_encoding() {
        let content = crate::test_support::builtin();
        let world = World::new(content, "showcase", None).expect("world");
        HASHING.with(|h| h.set(true));
        let json = serde_json::to_vec(&world.state).expect("serializes");
        HASHING.with(|h| h.set(false));
        assert_eq!(world.state_hash(), fnv1a(&json));
    }
}
