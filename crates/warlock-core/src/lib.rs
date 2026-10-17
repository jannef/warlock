//! Warlock simulation core.
//!
//! Everything that decides gameplay outcomes lives here. The crate has no Bevy dependency, performs
//! no platform I/O inside the rules, and never reads wall-clock time, so it is deterministic and can
//! be tested and driven headlessly (see `warlock-cli`).

/// Game version embedded in saves, replays, and dumps.
pub const GAME_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn version_is_set() {
        assert!(!super::GAME_VERSION.is_empty());
    }
}
