//! Save format: a versioned envelope around [`WorldState`], as gzip-compressed JSON (spec §7).
//!
//! This module only turns a world into bytes and back; where the bytes live is the app's concern.
//! Before 1.0, a save from another format version, a build with different content, or a damaged
//! file is rejected with a clear error and never misread.

use crate::content::Content;
use crate::world::{World, WorldState};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::sync::Arc;

/// Marker identifying a Warlock save inside the envelope.
const MAGIC: &str = "WARLOCK-SAVE";
/// The only save format version this build reads. Bump on any incompatible state change.
///
/// 2: the gameplay random stream in the world state; ambient and placed lights in level states.
/// 3: the game clock (`time`) in the world state; level kind and turn length in level states.
/// 4: level states keep their start (arrival point) and edge exit (local areas of the overworld).
/// 5: actors carry scheduler energy (the speed system).
/// 6: objects: piles and remembered piles in level states, inventories on actors.
/// 7: creatures keep what they have in mind (`Actor::mind`: last seen, noise, post; the AI).
/// 8: actors have stats (`Actor::stats`: a score and a potential for every stat).
/// 9: the player's character (`WorldState::character`: race, gender, and class, the stat the
/// gender raised, birth date, and birth sign).
/// 10: actors wear and wield things (`Actor::equipment`: an inventory stack per slot).
/// 11: combat: actors have hit points (`Actor::hp`, `hp_base`, `regen`); the world records the
///     player's death.
/// 12: ranged combat: level states keep their fires (burning missiles and when they burn out).
/// 13: experience: the player's XP total in the world state (`WorldState::experience`); creature
///     types have levels and XP.
/// 14: skills: actors' trained skills (`Actor::skills`: value and training marks by skill) and the
///     training random stream (`WorldState::training`).
/// 15: special rooms: level states keep their special rooms and whether the player has been in
///     each (`LevelState::rooms`); content fingerprints cover room effect ids.
/// 16: weapon skills: actors have a rank and marks in each (`Actor::weapon_skills`).
/// 17: digging: level states keep the work done on walls dug into (`LevelState::dug`).
/// 18: combat stances (`Actor::stance`).
/// 19: blood: level states keep their stains and the stains as last seen.
/// 20: status effects: actors have them (`Actor::effects`), their healing progress counts
///     hundredths of a second (`Actor::regen`), and the player's death records the status that
///     killed them (`Death::of`).
/// 26: magic items: a stack may carry what it rolled (`Stack::magic`: rarity, item level, each
///     affix by id with its tier and value, a rare item's name, whether it is identified).
pub const FORMAT_VERSION: u32 = 26;
/// Upper bound on the decompressed size, so a hostile file cannot exhaust memory.
const MAX_JSON_BYTES: u64 = 256 * 1024 * 1024;

/// Why a save could not be read. Messages are meant for the player.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SaveError {
    #[error("this is not a Warlock save file")]
    NotASave,
    #[error("this save uses format version {found}, but this game reads version {supported}")]
    UnsupportedVersion { found: u32, supported: u32 },
    #[error("this save was made with different game content and cannot be loaded")]
    ContentMismatch,
    #[error("the save file is damaged: {0}")]
    Corrupt(String),
}

#[derive(Serialize)]
struct EnvelopeOut<'a> {
    magic: &'static str,
    format_version: u32,
    game_version: &'a str,
    content_fingerprint: u64,
    state: &'a WorldState,
}

/// The envelope with the payload still undecoded, so the header is checked before the state.
#[derive(Deserialize)]
struct EnvelopeIn {
    magic: String,
    format_version: u32,
    #[serde(default)]
    content_fingerprint: u64,
    state: serde_json::Value,
}

/// Serializes the world's state into save bytes.
pub fn encode(world: &World) -> Vec<u8> {
    encode_state(world.state(), world.content().fingerprint())
}

/// Serializes a state (for example a clone taken earlier) with the content fingerprint of the
/// content it was played with.
pub fn encode_state(state: &WorldState, content_fingerprint: u64) -> Vec<u8> {
    let envelope = EnvelopeOut {
        magic: MAGIC,
        format_version: FORMAT_VERSION,
        game_version: crate::GAME_VERSION,
        content_fingerprint,
        state,
    };
    let mut gz = GzEncoder::new(Vec::new(), Compression::default());
    // The state is plain data with string-keyed maps, and writing to a Vec cannot fail, so these
    // fallbacks are unreachable; an empty result would simply fail to decode as a save.
    let json = serde_json::to_vec(&envelope).unwrap_or_default();
    let _ = gz.write_all(&json);
    gz.finish().unwrap_or_default()
}

/// Restores a world from save bytes, validating it against `content`.
pub fn decode(bytes: &[u8], content: Arc<Content>) -> Result<World, SaveError> {
    if !bytes.starts_with(&[0x1f, 0x8b]) {
        return Err(SaveError::NotASave);
    }
    let mut json = Vec::new();
    GzDecoder::new(bytes)
        .take(MAX_JSON_BYTES)
        .read_to_end(&mut json)
        .map_err(|e| SaveError::Corrupt(e.to_string()))?;
    let envelope: EnvelopeIn = serde_json::from_slice(&json).map_err(|e| {
        if e.is_syntax() || e.is_eof() {
            SaveError::Corrupt(e.to_string())
        } else {
            SaveError::NotASave
        }
    })?;
    if envelope.magic != MAGIC {
        return Err(SaveError::NotASave);
    }
    if envelope.format_version != FORMAT_VERSION {
        return Err(SaveError::UnsupportedVersion {
            found: envelope.format_version,
            supported: FORMAT_VERSION,
        });
    }
    if envelope.content_fingerprint != content.fingerprint() {
        return Err(SaveError::ContentMismatch);
    }
    let state: WorldState =
        serde_json::from_value(envelope.state).map_err(|e| SaveError::Corrupt(e.to_string()))?;
    World::from_state(content, state).map_err(|e| SaveError::Corrupt(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Action, Dir};

    fn content() -> Arc<Content> {
        Arc::new(Content::builtin().expect("builtin content loads"))
    }

    fn world() -> World {
        let mut w = World::new(content(), "showcase", None).expect("world");
        for _ in 0..5 {
            w.apply(&Action::Move(Dir::E));
        }
        w
    }

    fn gz(json: &str) -> Vec<u8> {
        let mut e = GzEncoder::new(Vec::new(), Compression::default());
        e.write_all(json.as_bytes()).expect("write");
        e.finish().expect("finish")
    }

    #[test]
    fn round_trip_preserves_state() {
        let w = world();
        let restored = decode(&encode(&w), content()).expect("decode");
        assert_eq!(restored.state(), w.state());
    }

    #[test]
    fn encode_state_matches_encode() {
        let w = world();
        let parked = (w.state().clone(), w.content().fingerprint());
        assert_eq!(encode_state(&parked.0, parked.1), encode(&w));
    }

    #[test]
    fn restored_world_continues_identically() {
        let mut a = world();
        let mut b = decode(&encode(&a), content()).expect("decode");
        for _ in 0..20 {
            a.apply(&Action::Wait);
            b.apply(&Action::Wait);
        }
        assert_eq!(a.state(), b.state());
    }

    #[test]
    fn saves_with_a_level_turn_length_still_load() {
        // Before action costs (#105) every level state had its turn length; it is ignored now.
        let w = world();
        let mut state = serde_json::to_value(w.state()).expect("state");
        let levels = state["levels"].as_object_mut().expect("levels");
        for level in levels.values_mut() {
            level["turn"] = serde_json::json!(600);
        }
        let mut envelope = serde_json::json!({
            "magic": MAGIC,
            "format_version": FORMAT_VERSION,
            "content_fingerprint": w.content().fingerprint(),
        });
        envelope["state"] = state;
        let restored = decode(&gz(&envelope.to_string()), content()).expect("loads");
        assert_eq!(restored.state(), w.state());
    }

    #[test]
    fn rejects_other_format_version() {
        let json = r#"{"magic":"WARLOCK-SAVE","format_version":99,"state":null}"#;
        assert_eq!(
            decode(&gz(json), content()).err(),
            Some(SaveError::UnsupportedVersion {
                found: 99,
                supported: FORMAT_VERSION
            })
        );
    }

    #[test]
    fn rejects_content_mismatch() {
        let json = format!(
            r#"{{"magic":"WARLOCK-SAVE","format_version":{FORMAT_VERSION},"content_fingerprint":1,"state":null}}"#
        );
        assert_eq!(
            decode(&gz(&json), content()).err(),
            Some(SaveError::ContentMismatch)
        );
    }

    #[test]
    fn rejects_foreign_data() {
        assert_eq!(decode(b"hello", content()).err(), Some(SaveError::NotASave));
        let other = gz(r#"{"magic":"OTHER","format_version":1,"state":1}"#);
        assert_eq!(decode(&other, content()).err(), Some(SaveError::NotASave));
    }

    #[test]
    fn rejects_corrupt_and_truncated_data() {
        let bytes = encode(&world());
        let truncated = &bytes[..bytes.len() / 2];
        assert!(matches!(
            decode(truncated, content()),
            Err(SaveError::Corrupt(_))
        ));
        assert!(matches!(
            decode(&gz("{ not json"), content()),
            Err(SaveError::Corrupt(_))
        ));
        let bad_state = format!(
            r#"{{"magic":"WARLOCK-SAVE","format_version":{FORMAT_VERSION},"content_fingerprint":{},"state":{{"seed":"x"}}}}"#,
            content().fingerprint()
        );
        assert!(matches!(
            decode(&gz(&bad_state), content()),
            Err(SaveError::Corrupt(_))
        ));
    }
}
