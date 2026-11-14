//! The ally talents (#1495): what a talent does for the holder's allies (spec §8 "Allies"):
//! Beast-bond's edge, the summon draw's bias, and Sic 'em. [`AllyEffect`] is the data (a talent's
//! `[[effect]]` of one of [`AllyEffect::KINDS`]), [`AllyMods`] the sum every rule reads
//! (`TalentMods::ally`). A new effect of the area is a variant here, its field and its arm in
//! [`AllyMods::add`] here, its wiki text in `xtask/src/wiki/talents/ally.rs`, and its one read.

use crate::content::{Content, StatusId};
use serde::Deserialize;

/// What a talent of the area does, one typed effect, read by one rule each.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AllyEffect {
    /// What the holder's allies get (spec §8 "Allies": Beast-bond): to-hit on their attacks
    /// and Protection, read through their master (`world/allies/ally.rs`); and what the holder's summon
    /// spells do (`world/magic/summon_spell.rs`): `count` more come every call, a call lasts
    /// `duration_percent` longer, `lasting` calls stay until slain, `most` more of each call's
    /// are kept at once.
    AllyMod {
        #[serde(default)]
        to_hit: i32,
        #[serde(default)]
        protection: i32,
        #[serde(default)]
        count: i32,
        #[serde(default)]
        duration_percent: i32,
        #[serde(default)]
        lasting: bool,
        #[serde(default)]
        most: i32,
    },
    /// The summon draw's bias (the owner's Q9 of #828): a creature of a summon spell's pool that
    /// carries `tag` is `weight` times as likely to come (0: it never comes while another may).
    SummonBias { tag: String, weight: u32 },
    /// The class active Sic 'em (`world/allies/sic.rs`): the companion is set on a creature the player
    /// sees within `range`, and suffers `blows` degrees of `status`, one that makes its hits
    /// criticals (`auto_critical`; what it costs is the talent's `resource`).
    Sic {
        status: String,
        blows: u16,
        range: i32,
    },
}

impl AllyEffect {
    /// The `kind`s of the data that are effects of the area, one a variant (`TalentEffect`'s
    /// deserializer hands these to [`AllyEffect`]).
    pub const KINDS: [&'static str; 3] = ["ally_mod", "sic", "summon_bias"];
}

/// What a talent gives the holder's allies (spec §8 "Allies"): read where an ally strikes and
/// is struck (`world/allies/ally.rs`), through its master's mods.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AllyMods {
    /// Added to every ally's attacks.
    pub to_hit: i32,
    /// Added to every ally's Protection.
    pub protection: i32,
    /// More creatures every summon spell's call brings (added to its count roll).
    pub count: i32,
    /// Percent longer every call lasts (on its minutes, rounded down, at least 1).
    pub duration_percent: i32,
    /// Calls never lapse: they stay until slain.
    pub lasting: bool,
    /// More of each summon spell's calls kept at once (on its `most`).
    pub most: i32,
    /// The summon draw's bias: a pool creature carrying the tag weighs this many times as much
    /// (`world/magic/summon_spell.rs`), in the order taken.
    pub summon_bias: Vec<(String, u32)>,
    /// The active Sic 'em, if it has it (`world/allies/sic.rs`).
    pub sic: Option<Sic>,
}

impl AllyMods {
    /// Adds what `effect` does.
    pub(crate) fn add(&mut self, content: &Content, effect: AllyEffect) {
        match effect {
            AllyEffect::AllyMod {
                to_hit,
                protection,
                count,
                duration_percent,
                lasting,
                most,
            } => {
                self.to_hit += to_hit;
                self.protection += protection;
                self.count += count;
                self.duration_percent += duration_percent;
                self.lasting |= lasting;
                self.most += most;
            }
            AllyEffect::SummonBias { tag, weight } => self.summon_bias.push((tag, weight)),
            AllyEffect::Sic {
                status,
                blows,
                range,
            } => {
                if let Some(status) = content.status_id(&status) {
                    self.sic = Some(Sic {
                        status,
                        blows,
                        range,
                    });
                }
            }
        }
    }
}

/// The active Sic 'em (spec §8 "Class abilities"; `world/allies/sic.rs`): the companion is set on a
/// creature the player sees within `range`, suffering `blows` degrees of `status`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sic {
    pub status: StatusId,
    pub blows: u16,
    pub range: i32,
}
