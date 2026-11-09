//! Miscasts as data (spec §8 "Spells": "Miscasts", #1271): the Spellcraft check every cast with
//! one's own mana makes (`data/miscast.toml`, [`MiscastRules`]) and the one ordered table a
//! failed check rolls on (`data/miscasts/`, one `[[miscast]]` file each, [`Miscast`]). The rules
//! are `world/miscast.rs`.
//!
//! ```toml
//! # data/miscast.toml
//! stat = "intelligence" # the stat whose modifier the check adds, with Spellcraft's `cast`
//! base = 8              # the difficulty of a spell of level 0
//! per_level = 1         # and more per spell level
//! armour_percent = 10   # +1 for each this many percent armour adds to a spell's cost
//! rank_ease = 1         # and less for each rank of the spell above the first
//! die = "2d12"          # the table's die, a dice expression
//! per_margin = 5        # the total's points for each point the check missed by
//! safe_below = 71       # an entry ending below this is protected (below)
//! backfire_faction = "berserk" # what a backfired summon's creatures come as (optional)
//! ```
//!
//! ```toml
//! # data/miscasts/glamour.toml
//! [[miscast]]
//! id = "glamour"
//! from = 1              # the totals it covers, `to` left out on the last entry only
//! to = 8
//! message = "Your weave slips: the air glitters with harmless motes of colour."
//! seen = "{who}'s weave slips: the air glitters with harmless motes of colour."
//! effect = { kind = "nothing" }
//! ```
//!
//! The entries tile the totals from 1 up with no gap or overlap; their effects are the mishaps'
//! ([`super::learning::resolve_effect`]) with the miscast's own kinds.
//!
//! **Protected entries** (the miscasts design §4): an entry whose `to` is below `safe_below` can
//! never kill its caster nor cost them anything they cannot get back. Its harm is capped at the
//! caster's hit points less one (`world/miscast.rs`), and the loader refuses one whose effect is
//! a `backfire`, a `drain`, a `forget`, a `summon`, a `portal`, a fire `zone`, or a status that
//! ticks harm or mana (`check_protected`), so a campaign's table cannot break the promise either; a
//! `rank_loss` is allowed, as it never takes a spell below rank 1. A stat loss there is
//! `stat_damage`, which mends.

use super::learning::{EffectSpec, resolve_effect};
use super::{Content, ContentError, FactionId, Factions, MishapEffect, ZoneKind, duplicate_id};
use crate::dice::Dice;
use crate::stats::Stat;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The numbers of the Spellcraft check and of the table's roll.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MiscastRules {
    /// The stat whose modifier the check adds.
    pub stat: Stat,
    /// The difficulty of a spell of level 0.
    pub base: i32,
    /// Harder per spell level.
    pub per_level: i32,
    /// One point harder for each this many percent the armour worn adds to a spell's cost.
    pub armour_percent: u32,
    /// Easier for each rank of the spell cast above the first.
    pub rank_ease: i32,
    /// The table's die.
    pub die: Dice,
    /// Points on the table for each point the check missed by.
    pub per_margin: i32,
    /// An entry whose `to` is below this is protected (module docs).
    pub safe_below: i32,
    /// The faction a backfired summon's creatures come in (`world/backfire.rs`), hostile to
    /// everyone; `None`: their type's own.
    #[serde(skip)]
    pub backfire_faction: Option<FactionId>,
}

impl MiscastRules {
    /// Parses `data/miscast.toml`: `armour_percent` positive, a die that rolls at least 1, a
    /// `backfire_faction` that is a known faction.
    pub(super) fn parse(file: &str, text: &str, factions: &Factions) -> Result<Self, ContentError> {
        let err = |m: &str| ContentError::new(file, m);
        let mut table: toml::Table = toml::from_str(text).map_err(|e| err(&e.to_string()))?;
        let faction = match table.remove("backfire_faction") {
            None => None,
            Some(toml::Value::String(name)) => Some(
                (factions.id(&name))
                    .ok_or_else(|| err(&format!("unknown `backfire_faction` `{name}`")))?,
            ),
            Some(_) => return Err(err("`backfire_faction` must be a faction id")),
        };
        let mut parsed: Self = table.try_into().map_err(|e| err(&e.to_string()))?;
        parsed.backfire_faction = faction;
        if parsed.armour_percent < 1 {
            return Err(err("`armour_percent` must be at least 1"));
        }
        if parsed.die.min() < 1 {
            return Err(err("`die` must roll at least 1"));
        }
        Ok(parsed)
    }

    /// The difficulty of casting a spell of `level`, known at `rank` (1 for a creature's), in
    /// armour adding `armour` percent to its cost.
    pub fn difficulty(&self, level: u8, armour: u32, rank: u8) -> i32 {
        let armour = i32::try_from(armour / self.armour_percent).unwrap_or(i32::MAX);
        let ease = self.rank_ease * i32::from(rank.saturating_sub(1));
        self.base + self.per_level * i32::from(level) + armour - ease
    }

    /// Whether `entry` is protected: it ends below `safe_below` (module docs).
    pub fn protects(&self, entry: &Miscast) -> bool {
        entry.to.is_some_and(|to| to < self.safe_below)
    }
}

/// An entry of the miscast table: what befalls a caster whose total falls in `from..=to`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Miscast {
    pub id: String,
    pub from: i32,
    /// `None` on the last entry: every total from `from` up.
    pub to: Option<i32>,
    /// Said to the player who miscast.
    pub message: String,
    /// Said of a creature that miscast where the player sees it, `{who}` its name.
    pub seen: String,
    pub effect: MishapEffect,
}

impl Miscast {
    /// Whether `total` falls in its range.
    pub fn covers(&self, total: i32) -> bool {
        total >= self.from && self.to.is_none_or(|to| total <= to)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MiscastFile {
    #[serde(default)]
    miscast: Vec<MiscastSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MiscastSpec {
    id: String,
    from: i32,
    to: Option<i32>,
    message: String,
    seen: String,
    effect: EffectSpec,
}

/// Parses the miscast files and orders the table by `from`; statuses, creatures, and damage
/// types must be loaded. The entries must tile the totals from 1 up (module docs).
pub(super) fn parse<'a>(
    files: impl Iterator<Item = (&'a str, &'a str)>,
    content: &Content,
) -> Result<Vec<Miscast>, ContentError> {
    let mut all: Vec<Miscast> = Vec::new();
    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    for (path, text) in files {
        let file: MiscastFile =
            toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))?;
        for spec in file.miscast {
            if let Some(first) = seen.get(&spec.id) {
                return Err(duplicate_id("miscast", &spec.id, path, first));
            }
            seen.insert(spec.id.clone(), path.to_string());
            let id = spec.id.clone();
            let err = |m: String| ContentError::new(path, format!("miscast `{id}`: {m}"));
            let effect = resolve_effect(spec.effect, content, true).map_err(err)?;
            if !spec.seen.contains("{who}") {
                return Err(err("its `seen` must name `{who}`".into()));
            }
            let entry = Miscast {
                id: spec.id,
                from: spec.from,
                to: spec.to,
                message: spec.message,
                seen: spec.seen,
                effect,
            };
            if let Some(rules) = content.miscast_rules()
                && rules.protects(&entry)
            {
                check_protected(content, entry.effect).map_err(|m| {
                    err(format!(
                        "{m}, which an entry below {} may not do",
                        rules.safe_below
                    ))
                })?;
            }
            all.push(entry);
        }
    }
    all.sort_by_key(|m| m.from);
    check_tiles(&all).map_err(|m| ContentError::new("data/miscasts", m))?;
    Ok(all)
}

/// Whether a protected entry may do `effect` (module docs): never a backfire, a drain, a
/// forgetting, a summon, a portal, a fire zone, or a status that ticks harm or mana.
fn check_protected(content: &Content, effect: MishapEffect) -> Result<(), String> {
    match effect {
        MishapEffect::Backfire { .. }
        | MishapEffect::Drain { .. }
        | MishapEffect::Forget { .. }
        | MishapEffect::Summon { .. }
        | MishapEffect::Portal { .. } => Err(format!("it does `{}`", effect.kind())),
        MishapEffect::Zone {
            kind: ZoneKind::Fire,
            ..
        } => Err("it lays a fire zone".into()),
        MishapEffect::Status { status, .. } | MishapEffect::Boon { status, .. } => {
            let def = content.status(status);
            if def.damage > 0 || def.mana_damage > 0 {
                Err(format!("its status `{}` ticks", def.id))
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
}

/// Whether `table` (ordered by `from`) covers every total from 1 up exactly once.
fn check_tiles(table: &[Miscast]) -> Result<(), String> {
    let mut next = 1;
    for (i, m) in table.iter().enumerate() {
        if m.from != next {
            return Err(format!(
                "miscast `{}` starts at {}, but the table's next total is {next}",
                m.id, m.from
            ));
        }
        match (m.to, i + 1 == table.len()) {
            (Some(to), _) if to < m.from => {
                return Err(format!("miscast `{}` ends before it starts", m.id));
            }
            (Some(to), _) => next = to + 1,
            (None, true) => return Ok(()),
            (None, false) => {
                return Err(format!(
                    "miscast `{}` has no `to` but is not the last",
                    m.id
                ));
            }
        }
    }
    match table.last() {
        Some(last) => Err(format!(
            "the last miscast, `{}`, must leave out `to`",
            last.id
        )),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, from: i32, to: Option<i32>) -> Miscast {
        Miscast {
            id: id.into(),
            from,
            to,
            message: String::new(),
            seen: String::new(),
            effect: MishapEffect::Nothing,
        }
    }

    #[test]
    fn the_rules_say_the_difficulty() {
        let text = "stat = \"intelligence\"\nbase = 8\nper_level = 1\narmour_percent = 10\n\
            rank_ease = 1\ndie = \"2d12\"\nper_margin = 5\nsafe_below = 71\n";
        let none = Factions::default();
        let r = MiscastRules::parse("miscast.toml", text, &none).expect("valid");
        assert_eq!(r.backfire_faction, None, "left out: the type's own");
        assert_eq!(r.difficulty(0, 0, 1), 8);
        assert_eq!(r.difficulty(11, 0, 1), 19);
        assert_eq!(r.difficulty(1, 25, 1), 11);
        // Each rank above the first eases it by one: rank 10 is a spell nine levels lower.
        assert_eq!(r.difficulty(11, 0, 10), 10);
        assert_eq!(r.difficulty(11, 0, 0), 19, "rank 0 reads as the first");
        assert!(r.protects(&entry("a", 69, Some(70))));
        assert!(!r.protects(&entry("b", 71, Some(72))));
        assert!(
            !r.protects(&entry("c", 1, None)),
            "the open last entry is not"
        );
    }

    #[test]
    fn the_backfire_faction_is_a_known_one() {
        let c = Content::builtin().expect("content");
        let rules = c.miscast_rules().expect("rules");
        let berserk = rules.backfire_faction.expect("the built-in names one");
        assert_eq!(c.factions().def(berserk).id, "berserk");
        let text = "stat = \"intelligence\"\nbase = 8\nper_level = 1\narmour_percent = 10\n\
            rank_ease = 1\ndie = \"2d12\"\nper_margin = 5\nsafe_below = 71\n\
            backfire_faction = \"nobody\"\n";
        assert!(MiscastRules::parse("miscast.toml", text, c.factions()).is_err());
    }

    #[test]
    fn the_table_must_tile_the_totals_from_one_up() {
        let good = [entry("a", 1, Some(5)), entry("b", 6, None)];
        assert!(check_tiles(&good).is_ok());
        assert!(good[1].covers(1000) && good[0].covers(5) && !good[0].covers(6));
        let gap = [entry("a", 1, Some(5)), entry("b", 7, None)];
        assert!(check_tiles(&gap).is_err());
        let late = [entry("a", 2, None)];
        assert!(check_tiles(&late).is_err());
        let closed = [entry("a", 1, Some(5))];
        assert!(check_tiles(&closed).is_err());
        let open_early = [entry("a", 1, None), entry("b", 2, None)];
        assert!(check_tiles(&open_early).is_err());
    }
}
