//! Save format: an envelope around [`WorldState`], as gzip-compressed JSON (spec §7).
//!
//! This module only turns a world into bytes and back; where the bytes live is the app's concern.
//! A save is pinned to the game version that wrote it ([`crate::GAME_VERSION`], the workspace
//! version): a save from another version, a build with different content, or a damaged file is
//! rejected with a clear error and never misread. Nothing carries a save across versions.

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
/// Upper bound on the decompressed size, so a hostile file cannot exhaust memory.
const MAX_JSON_BYTES: u64 = 256 * 1024 * 1024;

/// Why a save could not be read. Messages are meant for the player.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SaveError {
    #[error("this is not a Warlock save file")]
    NotASave,
    /// Made by another version of the game (`found`; empty when the save does not say).
    #[error("this save was made by {}, and only that version can continue it", version_words(.found))]
    OtherVersion { found: String },
    #[error("this save was made with different game content and cannot be loaded")]
    ContentMismatch,
    #[error("the save file is damaged: {0}")]
    Corrupt(String),
}

/// `version 0.8.0`, or for a save that does not say, `an older version`.
fn version_words(found: &str) -> String {
    if found.is_empty() {
        "an older version".into()
    } else {
        format!("version {found}")
    }
}

#[derive(Serialize)]
struct EnvelopeOut<'a> {
    magic: &'static str,
    game_version: &'a str,
    content_fingerprint: u64,
    state: &'a WorldState,
}

/// The envelope with the payload still undecoded, so the header is checked before the state.
#[derive(Deserialize)]
struct EnvelopeIn {
    magic: String,
    #[serde(default)]
    game_version: String,
    #[serde(default)]
    content_fingerprint: u64,
    state: serde_json::Value,
}

/// Serializes the world's state into save bytes.
pub fn encode(world: &World) -> Vec<u8> {
    encode_state(world.state(), world.fingerprint())
}

/// Serializes a state (for example a clone taken earlier) with the content fingerprint of the
/// content it was played with.
pub fn encode_state(state: &WorldState, content_fingerprint: u64) -> Vec<u8> {
    let envelope = EnvelopeOut {
        magic: MAGIC,
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

/// The campaign of a saved state: its own, or for a state without one its scenario's.
fn campaign_of(state: &serde_json::Value, content: &Content) -> String {
    let field = |name: &str| state.get(name).and_then(serde_json::Value::as_str);
    match field("campaign").filter(|c| !c.is_empty()) {
        Some(campaign) => campaign.to_string(),
        None => (field("scenario").and_then(|s| content.scenario(s)))
            .and_then(|s| s.campaign.clone())
            .unwrap_or_default(),
    }
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
    if envelope.game_version != crate::GAME_VERSION {
        return Err(SaveError::OtherVersion {
            found: envelope.game_version,
        });
    }
    if envelope.content_fingerprint
        != content.fingerprint_for(&campaign_of(&envelope.state, &content))
    {
        return Err(SaveError::ContentMismatch);
    }
    let state: WorldState =
        serde_json::from_value(envelope.state).map_err(|e| SaveError::Corrupt(e.to_string()))?;
    World::from_state(content, state).map_err(|e| SaveError::Corrupt(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Action, Dir, GAME_VERSION};

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
        let parked = (w.state().clone(), w.fingerprint());
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
    fn a_save_records_the_version_that_wrote_it() {
        let mut json = Vec::new();
        let bytes = encode(&world());
        GzDecoder::new(&bytes[..])
            .read_to_end(&mut json)
            .expect("gunzip");
        let envelope: serde_json::Value = serde_json::from_slice(&json).expect("json");
        assert_eq!(envelope["game_version"], GAME_VERSION);
    }

    /// A save of another version is refused before its state is read, however well it would
    /// read; one that names no version (older than the pin) too.
    #[test]
    fn rejects_another_version() {
        let w = world();
        let mut state = serde_json::to_value(w.state()).expect("state");
        let other = |version: Option<&str>, state: &mut serde_json::Value| {
            let mut envelope = serde_json::json!({
                "magic": MAGIC,
                "content_fingerprint": w.fingerprint(),
            });
            if let Some(version) = version {
                envelope["game_version"] = version.into();
            }
            envelope["state"] = state.take();
            decode(&gz(&envelope.to_string()), content()).err()
        };
        assert_eq!(
            other(Some("0.1.0"), &mut state.clone()),
            Some(SaveError::OtherVersion {
                found: "0.1.0".into()
            })
        );
        let unnamed = other(None, &mut state);
        assert_eq!(
            unnamed,
            Some(SaveError::OtherVersion {
                found: String::new()
            })
        );
        let text = unnamed.map(|e| e.to_string()).unwrap_or_default();
        assert!(text.contains("an older version"), "{text}");
    }

    #[test]
    fn rejects_content_mismatch() {
        let json = format!(
            r#"{{"magic":"WARLOCK-SAVE","game_version":"{GAME_VERSION}","content_fingerprint":1,"state":null}}"#
        );
        assert_eq!(
            decode(&gz(&json), content()).err(),
            Some(SaveError::ContentMismatch)
        );
    }

    #[test]
    fn rejects_foreign_data() {
        assert_eq!(decode(b"hello", content()).err(), Some(SaveError::NotASave));
        let other = gz(r#"{"magic":"OTHER","game_version":"0.1.0","state":1}"#);
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
            r#"{{"magic":"WARLOCK-SAVE","game_version":"{GAME_VERSION}","content_fingerprint":{},"state":{{"seed":"x"}}}}"#,
            content().fingerprint_for("")
        );
        assert!(matches!(
            decode(&gz(&bad_state), content()),
            Err(SaveError::Corrupt(_))
        ));
    }
}
