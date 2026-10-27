//! A shrine of a side (`shrine = { ... }` on an effect of `data/room_effects.toml`, spec §7
//! "Special rooms", design §9.3): the room has a hidden side, good or evil, drawn at generation
//! one to one (its feature shows it by its light, as a forgotten altar's), and it treats
//! everyone inside by their own side: one who agrees heals faster, one opposed holds a status
//! (Unwelcome), a neutral one is ignored. `world/shrine.rs` applies it.

use super::room_effect::{DWELL_EASES, REGEN};
use super::{StatusDef, StatusId};
use crate::alignment::Alignment;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ShrineSpec {
    regen: u16,
    status: String,
    welcome: String,
    unwelcome: String,
}

/// What a shrine does to those inside, by whether their side agrees with its own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shrine {
    /// Percent of the usual pace at which hit points come back inside for one whose side
    /// agrees with the shrine's.
    pub regen: u16,
    /// The status one of the other side holds while inside (Unwelcome).
    pub status: StatusId,
    /// The first entry's line for one who agrees (in place of the effect's `enter`, which a
    /// neutral one is told).
    pub welcome: String,
    /// The first entry's line for one opposed.
    pub unwelcome: String,
}

/// How a shrine of one side treats someone of a side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Regard {
    /// Of its side: healed faster.
    Welcome,
    /// Of the other side: Unwelcome.
    Unwelcome,
    /// Neutral (or a neutral shrine, which a hand-made level may give): nothing.
    Ignored,
}

impl Shrine {
    /// How a shrine of side `shrine` treats one of side `them`.
    pub fn regard(shrine: Alignment, them: Alignment) -> Regard {
        match (shrine, them) {
            (Alignment::Neutral, _) | (_, Alignment::Neutral) => Regard::Ignored,
            (a, b) if a == b => Regard::Welcome,
            _ => Regard::Unwelcome,
        }
    }

    /// The line the first entry says to one it treats `regard`, `enter` the effect's own (the
    /// neutral one's).
    pub fn enter_line<'a>(&'a self, regard: Regard, enter: &'a str) -> &'a str {
        match regard {
            Regard::Welcome => &self.welcome,
            Regard::Unwelcome => &self.unwelcome,
            Regard::Ignored => enter,
        }
    }

    /// Resolves and checks `spec`: its status as a room's (it must ease, slowly enough), its
    /// pace in range, its lines not empty. `has_sides`: whether the effect has `sides` (a
    /// shrine needs them, for its side and its light).
    pub(super) fn parse<'s>(
        spec: &ShrineSpec,
        has_sides: bool,
        status: impl Fn(&str) -> Option<(StatusId, &'s StatusDef)>,
    ) -> Result<Shrine, String> {
        if !has_sides {
            return Err("a `shrine` needs `sides` (and so a `feature`)".into());
        }
        if !REGEN.contains(&spec.regen) {
            return Err(format!(
                "shrine.regen {} is out of range ({} to {})",
                spec.regen,
                REGEN.start(),
                REGEN.end()
            ));
        }
        if spec.welcome.trim().is_empty() || spec.unwelcome.trim().is_empty() {
            return Err("shrine: `welcome` and `unwelcome` must not be empty".into());
        }
        let name = &spec.status;
        let (id, def) = status(name).ok_or_else(|| format!("unknown status `{name}`"))?;
        if def.eases.is_none_or(|every| every < DWELL_EASES) {
            return Err(format!(
                "shrine status `{name}` must ease, and take at least {DWELL_EASES} seconds a \
                 degree, so it never lapses while one stays inside"
            ));
        }
        Ok(Shrine {
            regen: spec.regen,
            status: id,
            welcome: spec.welcome.clone(),
            unwelcome: spec.unwelcome.clone(),
        })
    }
}
