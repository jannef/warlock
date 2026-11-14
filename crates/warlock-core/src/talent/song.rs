//! The song talents (#1495): the bard's songs (spec §8 "Class abilities"; `world/song.rs`) and
//! Virtuoso. [`SongEffect`] is the data (a talent's `[[effect]]` of one of [`SongEffect::KINDS`]),
//! [`SongMods`] the sum every rule reads (`TalentMods::song`). A new effect of the area is a
//! variant here, its field and its arm in [`SongMods::add`] here, its wiki text in
//! `xtask/src/wiki/talents/song.rs`, and its one read.

use crate::content::{Content, ResourceCost, SongOn, StatusId};
use crate::stats::Stat;
use serde::Deserialize;

/// What a talent of the area does, one typed effect, read by one rule each.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SongEffect {
    /// A song (spec §8 "Class abilities", the bard's songs; `world/song.rs`): class active `id`
    /// (`use:inspire`), sung aloud. Who it is `on` suffers `severity` degrees of `status`, and the
    /// singer's Music's `song_degrees` more; it costs **one** thing, `cost` of `resource`
    /// (Inspire's 3 Inspiration) or `mana`. A song on its hearers asks each hostile one a
    /// `check` of that stat against `difficulty` plus the singer's `song_craft` (the Lullaby's,
    /// #873).
    Song {
        id: String,
        on: SongOn,
        status: String,
        #[serde(default = "super::one_degree")]
        severity: u16,
        #[serde(default)]
        resource: Option<String>,
        #[serde(default)]
        cost: u16,
        #[serde(default)]
        mana: i32,
        #[serde(default)]
        check: Option<Stat>,
        #[serde(default)]
        difficulty: i32,
    },
    /// Every song costs nothing and gives a degree more; a song on the singer gives `status`
    /// instead (Virtuoso: Inspired greatly).
    Virtuoso { status: String },
}

impl SongEffect {
    /// The `kind`s of the data that are effects of the area, one a variant (`TalentEffect`'s
    /// deserializer hands these to [`SongEffect`]).
    pub const KINDS: [&'static str; 2] = ["song", "virtuoso"];
}

/// What an actor's talents of the area do, summed: one field per effect kind, read by one
/// rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SongMods {
    /// The songs it sings (Inspire), in the order first named: one a class active.
    pub songs: Vec<Song>,
    /// The status a song on the singer gives instead, every song free and a degree greater
    /// (Virtuoso).
    pub virtuoso: Option<StatusId>,
}

impl SongMods {
    /// Adds what `effect` does.
    pub(crate) fn add(&mut self, content: &Content, effect: SongEffect) {
        match effect {
            SongEffect::Song {
                id,
                on,
                status,
                severity,
                resource,
                cost,
                mana,
                check,
                difficulty,
            } => {
                let active = crate::content::ClassActive::from_id(&id);
                // A pool content lacks cannot pay for it: no song (content without resources).
                let cost = match resource {
                    Some(r) => (content.resource_id(&r)).map(|resource| {
                        SongCost::Resource(ResourceCost {
                            resource,
                            amount: cost,
                        })
                    }),
                    None => Some(SongCost::Mana(mana)),
                };
                let status = content.status_id(&status);
                if let (Some(active), Some(status), Some(cost)) = (active, status, cost) {
                    let cost = Some(cost);
                    let song = Song {
                        active,
                        on,
                        status,
                        severity,
                        cost,
                        check: check.map(|stat| (stat, difficulty)),
                    };
                    match self.songs.iter_mut().find(|s| s.active == active) {
                        Some(held) => *held = song,
                        None => self.songs.push(song),
                    }
                }
            }
            SongEffect::Virtuoso { status } => self.virtuoso = content.status_id(&status),
        }
    }

    /// Song `active` as it is sung, if it has it: a Virtuoso's free, a degree greater, and on the
    /// singer giving the Virtuoso's status; else as its data says.
    pub fn song(&self, active: crate::content::ClassActive) -> Option<Song> {
        let song = *self.songs.iter().find(|s| s.active == active)?;
        Some(match self.virtuoso {
            Some(status) => Song {
                status: match song.on {
                    crate::content::SongOn::Singer => status,
                    crate::content::SongOn::Hearers => song.status,
                },
                severity: song.severity + 1,
                cost: None,
                ..song
            },
            None => song,
        })
    }
}

/// A song (spec §8 "Class abilities", the bard's songs; `world/song.rs`): the class active it
/// is, whom it is sung on, the status it gives at its base degrees, what it costs (`None`:
/// nothing, a Virtuoso's), and for a song on its hearers the stat they check and its difficulty.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Song {
    pub active: crate::content::ClassActive,
    pub on: crate::content::SongOn,
    pub status: StatusId,
    pub severity: u16,
    pub cost: Option<SongCost>,
    pub check: Option<(Stat, i32)>,
}

/// What a song costs: one thing (the owner: no dual costs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SongCost {
    /// So much of an alternative resource (Inspire's Inspiration).
    Resource(ResourceCost),
    /// So much mana.
    Mana(i32),
}
