//! Leavings (docs/design/monster-dread.md §9 channel 3; #1133): what a creature type leaves about
//! it, `leavings = ["statue", "web"]` on the type, a cell dressing the generator lays within a
//! few cells of where a population draw or a vault places one (`mapgen/leavings.rs`). Each kind
//! is a terrain or an object of a fixed id (the statue is the one `statue` terrain):
//!
//! - `statue`: a statue of the stoned (`data/terrain.toml`'s `statue = true`), a nobody of the
//!   floor's own population, holding nothing;
//! - `web`: the [`WEB`] terrain, old webs, which catch no one;
//! - `acid_pit`: the [`ACID_PIT`] terrain, floor eaten into pits;
//! - `torn_cloth`: the [`TORN_CLOTH`] object, worth nothing.

use super::{Content, ObjectId, TerrainId};
use serde::{Deserialize, Serialize};

/// The terrain id of old webs.
pub const WEB: &str = "web";
/// The terrain id of acid-pitted floor.
pub const ACID_PIT: &str = "acid_pit";
/// The object id of torn cloth.
pub const TORN_CLOTH: &str = "torn_cloth";

/// A kind of leaving.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Leaving {
    Statue,
    Web,
    AcidPit,
    TornCloth,
}

/// What a leaving is laid as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Laid {
    Terrain(TerrainId),
    Object(ObjectId),
}

impl Leaving {
    /// Its id as data and dumps write it.
    pub fn id(self) -> &'static str {
        match self {
            Self::Statue => "statue",
            Self::Web => "web",
            Self::AcidPit => "acid_pit",
            Self::TornCloth => "torn_cloth",
        }
    }

    /// How many of it the generator lays about one creature: at least and at most.
    pub fn count(self) -> (i32, i32) {
        match self {
            Self::Statue => (1, 3),
            Self::Web => (2, 4),
            Self::AcidPit => (2, 5),
            Self::TornCloth => (1, 2),
        }
    }
}

impl Content {
    /// What `leaving` is laid as, if the content has it.
    pub fn leaving(&self, leaving: Leaving) -> Option<Laid> {
        match leaving {
            Leaving::Statue => self.statue_terrain().map(Laid::Terrain),
            Leaving::Web => self.terrain_id(WEB).map(Laid::Terrain),
            Leaving::AcidPit => self.terrain_id(ACID_PIT).map(Laid::Terrain),
            Leaving::TornCloth => self.object_id(TORN_CLOTH).map(Laid::Object),
        }
    }

    /// The leaving terrain `t` is, if it is one (a statue's terrain too: a stoned creature's
    /// statue is a basilisk's leaving all the same).
    pub fn leaving_terrain(&self, t: TerrainId) -> Option<Leaving> {
        [Leaving::Statue, Leaving::Web, Leaving::AcidPit]
            .into_iter()
            .find(|&l| self.leaving(l) == Some(Laid::Terrain(t)))
    }

    /// Whether object `o` is a leaving (torn cloth).
    pub fn is_leaving_object(&self, o: ObjectId) -> bool {
        self.leaving(Leaving::TornCloth) == Some(Laid::Object(o))
    }
}
