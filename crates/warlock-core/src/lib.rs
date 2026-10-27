//! Warlock simulation core.
//!
//! Everything that decides gameplay outcomes lives here. The crate has no Bevy dependency, performs
//! no platform I/O inside the rules, and never reads wall-clock time, so it is deterministic and can
//! be tested and driven headlessly (see `warlock-cli`).
//!
//! Entry points: load [`content::Content`], create a [`World`], feed it [`Action`]s, and read what
//! the player perceives through [`World::perception`].

pub mod ability;
pub mod action;
pub mod alignment;
pub mod animated;
pub mod blood;
pub mod burden;
pub mod character;
pub mod check;
pub mod color;
pub mod combat;
pub mod content;
pub mod damage;
pub mod daylight;
pub mod dice;
pub mod disease;
pub mod door;
pub mod dump;
pub mod equipment;
pub mod fov;
pub mod gear;
pub mod gem;
pub mod geom;
pub mod grid;
pub mod hunger;
pub mod ice;
pub mod item_parts;
pub mod kit;
pub mod light;
pub mod magic;
pub mod mana;
pub mod mapgen;
pub mod messages;
pub mod noise;
pub mod object;
pub mod perception;
pub mod piece;
pub mod replay;
pub mod resource;
pub mod rng;
pub mod rot;
pub mod save;
pub mod scroll_label;
mod sight;
pub mod skills;
pub mod stats;
pub mod status;
pub mod talent;
pub mod temperature;
pub mod time;
pub mod variant;
pub mod weapon_skills;
pub mod weather;
pub mod world;

pub use action::{
    Action, AttackKind, Breach, DiseaseNews, Event, Meal, MissileEnd, Outcome, Positions,
    ReadTarget, Rest, SpellShape,
};
pub use geom::{Dir, Pos};
pub use world::{World, WorldError, WorldState};

/// Game version embedded in saves, replays, and dumps.
pub const GAME_VERSION: &str = env!("CARGO_PKG_VERSION");
