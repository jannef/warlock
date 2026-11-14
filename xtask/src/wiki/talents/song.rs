//! What the song talents do, in the wiki's words (`warlock_core::talent::SongEffect`, #1495): one
//! arm a variant, beside the area's enum and sum in `warlock-core/src/talent/song.rs`.

use super::{buff, resource_name};
use warlock_core::content::{Content, SongOn};
use warlock_core::noise::SONG_NOISE;
use warlock_core::talent::SongEffect;

/// What `effect` does, as the talents' tables say it.
pub(super) fn text(content: &Content, effect: &SongEffect) -> String {
    match effect.clone() {
        SongEffect::Song {
            on,
            status: id,
            severity,
            resource,
            cost,
            mana,
            check,
            difficulty,
            ..
        } => {
            let price = match resource {
                Some(r) => format!("{cost} {}", resource_name(content, &r).to_lowercase()),
                None => format!("{mana} mana"),
            };
            let degrees = match severity {
                1 => "1 degree".to_string(),
                n => format!("{n} degrees"),
            };
            let whom = match (on, check) {
                (SongOn::Hearers, Some(stat)) => format!(
                    "each hostile creature that hears it checks {} against {difficulty} plus \
                     your Music's song difficulty; failed, it suffers",
                    stat.name()
                ),
                _ => "you suffer".to_string(),
            };
            format!(
                "active, a song (the time of a song, heard as a voice of loudness {SONG_NOISE}; \
                 not on the overworld or while silenced), {price}: {whom} {} at {degrees}, plus \
                 your Music's song degrees",
                buff(content, &id)
            )
        }
        SongEffect::Virtuoso { status: id } => format!(
            "every song costs nothing and gives a degree more; a song on yourself gives {} instead",
            buff(content, &id)
        ),
    }
}
