//! Deterministic random streams (spec §8 Determinism).
//!
//! Every stream is a ChaCha8 generator (`rand_chacha`; its output is specified, so it is identical
//! on every platform) seeded from a 64-bit key: FNV-1a over the run seed and a stream label.
//!
//! | Stream | Label | Kept |
//! |---|---|---|
//! | Map generation, per level | `map/<level id>` (retry *k*: `map/<level id>/<k>`) | never; derived on demand |
//! | Gameplay | `gameplay` | in [`crate::WorldState`], so saves continue it |
//! | The stat gender "other" raises | `character/gender` | never; derived on demand |
//! | A new character's age and birthday | `character/birth/<race>` | never |
//! | A new character's dice | `character/rolls/<race>,<gender>,<class>` | never |
//!
//! A level's map therefore depends only on the run seed and its id, never on the order in which
//! levels are visited or on gameplay; a new character only on the run seed and the choices
//! (`crate::character`). Visual randomness in the app uses its own generator and never
//! touches these streams. All helpers are integer-only: no floating point decides anything.

use crate::world::fnv1a;
use rand_chacha::ChaCha8Rng;
use rand_core::{Rng as _, SeedableRng};
use serde::{Deserialize, Serialize};

/// A deterministic random stream. Serializes as ChaCha's abstract state (seed, stream, position).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Rng(ChaCha8Rng);

impl Rng {
    /// The stream `label` of the run with `run_seed`.
    pub fn stream(run_seed: u64, label: &str) -> Self {
        Self(ChaCha8Rng::seed_from_u64(stream_key(run_seed, label)))
    }

    /// The gameplay stream of a run.
    pub fn gameplay(run_seed: u64) -> Self {
        Self::stream(run_seed, "gameplay")
    }

    /// The map-generation stream for `level`; `attempt` > 0 derives the stream for a retry after
    /// a generated map failed validation.
    pub fn map(run_seed: u64, level: &str, attempt: u32) -> Self {
        if attempt == 0 {
            Self::stream(run_seed, &format!("map/{level}"))
        } else {
            Self::stream(run_seed, &format!("map/{level}/{attempt}"))
        }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    /// Uniform in `0..n` without modulo bias (Lemire's multiply-and-reject); 0 when `n` is 0.
    pub fn below(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        let threshold = n.wrapping_neg() % n;
        loop {
            let m = u64::from(self.next_u32()) * u64::from(n);
            if (m as u32) >= threshold {
                return (m >> 32) as u32;
            }
        }
    }

    /// Uniform in `lo..=hi`; `lo` when the range is empty.
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        let span = (i64::from(hi) - i64::from(lo) + 1).min(i64::from(u32::MAX)) as u32;
        (i64::from(lo) + i64::from(self.below(span))) as i32
    }

    /// True with probability `percent` / 100.
    pub fn chance(&mut self, percent: u32) -> bool {
        self.below(100) < percent
    }

    /// A uniformly chosen index into a slice of length `len` (0 when `len` is 0).
    pub fn index(&mut self, len: usize) -> usize {
        self.below(u32::try_from(len).unwrap_or(u32::MAX)) as usize
    }

    /// A uniformly chosen element, or `None` for an empty slice.
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        (!items.is_empty()).then(|| &items[self.index(items.len())])
    }

    /// Fisher–Yates shuffle.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.index(i + 1);
            items.swap(i, j);
        }
    }
}

/// The 64-bit key of stream `label` in the run with `run_seed`: FNV-1a over the seed's
/// little-endian bytes, a `0xff` separator, and the label's UTF-8 bytes.
pub fn stream_key(run_seed: u64, label: &str) -> u64 {
    let mut bytes = Vec::with_capacity(9 + label.len());
    bytes.extend_from_slice(&run_seed.to_le_bytes());
    bytes.push(0xff);
    bytes.extend_from_slice(label.as_bytes());
    fnv1a(&bytes)
}

/// Parses a `--seed` value: a decimal number is used as is; any other text (a word or phrase) is
/// hashed with FNV-1a 64 over its UTF-8 bytes, so `--seed warlock` always means the same run.
/// Surrounding whitespace is ignored; case matters.
pub fn parse_seed(text: &str) -> Result<u64, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("the seed is empty".into());
    }
    if text.bytes().all(|b| b.is_ascii_digit()) {
        return text
            .parse()
            .map_err(|_| format!("seed `{text}` is larger than {}", u64::MAX));
    }
    Ok(fnv1a(text.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::Rng;
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn streams_are_reproducible_and_distinct() {
        let a: Vec<u32> = {
            let mut r = Rng::map(7, "depths:1", 0);
            (0..8).map(|_| r.next_u32()).collect()
        };
        let again: Vec<u32> = {
            let mut r = Rng::map(7, "depths:1", 0);
            (0..8).map(|_| r.next_u32()).collect()
        };
        assert_eq!(a, again);
        let mut other_level = Rng::map(7, "depths:2", 0);
        let mut retry = Rng::map(7, "depths:1", 1);
        let mut other_seed = Rng::map(8, "depths:1", 0);
        assert_ne!(a[0], other_level.next_u32());
        assert_ne!(a[0], retry.next_u32());
        assert_ne!(a[0], other_seed.next_u32());
        assert_ne!(Rng::gameplay(7), Rng::map(7, "gameplay", 0));
    }

    /// Pins the algorithm: if this changes, every seed produces a different run.
    #[test]
    fn output_is_pinned() {
        assert_eq!(stream_key(1, "gameplay"), 0x6a1b_986d_d3b2_e6a1);
        let mut r = Rng::gameplay(1);
        let first: Vec<u32> = (0..3).map(|_| r.next_u32()).collect();
        assert_eq!(first, [2_610_789_080, 3_370_076_174, 2_717_488_749]);
    }

    #[test]
    fn seeds_parse_from_numbers_and_words() {
        assert_eq!(parse_seed("42"), Ok(42));
        assert_eq!(parse_seed(" 7 "), Ok(7));
        assert_eq!(parse_seed("warlock"), Ok(fnv1a(b"warlock")));
        assert_ne!(parse_seed("Warlock"), parse_seed("warlock"));
        assert_eq!(parse_seed("-1"), Ok(fnv1a(b"-1")));
        assert!(parse_seed("").is_err());
        assert!(parse_seed("99999999999999999999").is_err());
    }

    #[test]
    fn serde_round_trip_continues_the_stream() {
        let mut r = Rng::gameplay(3);
        for _ in 0..5 {
            r.next_u32();
        }
        let json = serde_json::to_string(&r).unwrap();
        let mut back: Rng = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
        assert_eq!(back.next_u32(), r.next_u32());
        let via_value: Rng = serde_json::from_value(serde_json::to_value(&r).unwrap()).unwrap();
        assert_eq!(via_value, r);
    }

    proptest! {
        #[test]
        fn helpers_stay_in_range(seed: u64, n in 1u32..1000, lo in -50i32..50, span in 0i32..50) {
            let mut r = Rng::stream(seed, "test");
            prop_assert!(r.below(n) < n);
            let x = r.range(lo, lo + span);
            prop_assert!((lo..=lo + span).contains(&x));
            let mut items: Vec<u32> = (0..span as u32).collect();
            r.shuffle(&mut items);
            items.sort_unstable();
            prop_assert_eq!(items, (0..span as u32).collect::<Vec<_>>());
        }
    }
}
