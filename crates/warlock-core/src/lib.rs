//! Warlock simulation core.
//!
//! Everything that decides gameplay outcomes lives here. The crate has no Bevy dependency, performs
//! no platform I/O inside the rules, and never reads wall-clock time, so it is deterministic and can
//! be tested and driven headlessly (see `warlock-cli`).
//!
//! Entry points: load [`content::Content`], create a [`World`], feed it [`Action`]s, and read what
//! the player perceives through [`World::perception`].

pub mod action;
pub mod character;
pub mod color;
pub mod content;
pub mod daylight;
pub mod dice;
pub mod dump;
pub mod fov;
pub mod geom;
pub mod grid;
pub mod light;
pub mod mapgen;
pub mod messages;
pub mod noise;
pub mod object;
pub mod perception;
pub mod replay;
pub mod rng;
pub mod save;
mod sight;
pub mod stats;
pub mod time;
pub mod weather;
pub mod world;

pub use action::{Action, AttackKind, Event, Letters, Outcome, Rest};
pub use geom::{Dir, Pos};
pub use world::{World, WorldError, WorldState};

/// Game version embedded in saves, replays, and dumps.
pub const GAME_VERSION: &str = env!("CARGO_PKG_VERSION");
