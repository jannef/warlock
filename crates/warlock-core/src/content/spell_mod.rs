//! What talents and items do to a spell (spec §8 "Spells": "Hooks"; the spell scaling design
//! `docs/design/spell-scaling.md` §7.1): six named hooks, the fields of one [`SpellMods`], summed
//! once a cast from the caster's talents and worn items, each source filtered by the spell's tags
//! (`world/spell_mods.rs`, `Actor::spell_mods`), and read by the cast path in one order: the
//! rung, then the dice, then the roll, then the harm, then the riders; the reach and the degrees
//! beside. Nothing clamps what they do but the engine's own limits (decision 16).
//!
//! | # | Hook | Fields | Applied |
//! |---|---|---|---|
//! | 1 | rung | `rungs` (an own cast's roll), `scroll_rungs`, `wand_rungs`, `reach_rungs`, `degree_rungs` | before the ladder's lookup |
//! | 2 | dice | `sides`, `dice`, `bonus` | to the roll looked up: `grow(sides).more(dice).plus(bonus)` |
//! | 3 | roll | `reroll` | at the draw: a die showing it or less is drawn once more |
//! | 4 | rider | `riders` | after the harm, on every target harmed |
//! | 5 | reach | `cells` (range or length), `radius_cells` | after the reach ladder |
//! | 6 | degrees | `degrees` | after the degrees ladder; the status's `max` holds it |
//!
//! Besides the six, `cost` is the percent more mana (School focus's), summed into the one cost
//! hook beside `spell_mana_percent` (`Actor::spell_base`).
//!
//! A talent writes one as `{ kind = "spell_mod", rungs = 1 }`, `{ kind = "spell_mod", sides = 1
//! }` on a talent chosen for a school or an element (the choice is its filter), or `{ kind =
//! "spell_mod", rider = { status = "sluggish", severity = 1 }, when = ["cold"] }`.

use super::{Content, MAX_SEVERITY, Shape, SpellPurpose, StatusId};
use serde::Deserialize;

/// The six hooks' fields (module docs), summed. Every field defaults to nothing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpellMods {
    /// Rungs higher on a damage or heal ladder for a cast of one's own.
    pub rungs: i32,
    /// Rungs higher for the spell scrolls one reads.
    pub scroll_rungs: i32,
    /// Rungs higher for the wands one uses.
    pub wand_rungs: i32,
    /// Rungs higher on a reach ladder.
    pub reach_rungs: i32,
    /// Rungs higher on a degrees ladder.
    pub degree_rungs: i32,
    /// Every die this many sides bigger.
    pub sides: u32,
    /// This many dice more, of the roll's first group's kind.
    pub dice: u32,
    /// Added to the roll's bonus.
    pub bonus: i32,
    /// A die showing this or less is drawn once more.
    pub reroll: u32,
    /// Statuses laid on every target the spell harms, with their severities (one a status).
    pub riders: Vec<(StatusId, u16)>,
    /// Cells farther on a range or a length.
    pub cells: i32,
    /// Cells wider on a burst's radius.
    pub radius_cells: i32,
    /// Degrees more of a buff's or a hex's status.
    pub degrees: i32,
    /// Percent more mana (negative: less; School focus).
    pub cost: i32,
}

impl SpellMods {
    /// `other` added to it: every number summed, a rider of a status it has merged into its
    /// severity.
    pub fn add(&mut self, other: &SpellMods) {
        self.rungs += other.rungs;
        self.scroll_rungs += other.scroll_rungs;
        self.wand_rungs += other.wand_rungs;
        self.reach_rungs += other.reach_rungs;
        self.degree_rungs += other.degree_rungs;
        self.sides += other.sides;
        self.dice += other.dice;
        self.bonus += other.bonus;
        self.reroll += other.reroll;
        for &(status, severity) in &other.riders {
            merge_rider(&mut self.riders, status, severity);
        }
        self.cells += other.cells;
        self.radius_cells += other.radius_cells;
        self.degrees += other.degrees;
        self.cost += other.cost;
    }
}

/// Adds a rider of `status` at `severity` to `riders`: a status already there gains the
/// severity.
pub fn merge_rider(riders: &mut Vec<(StatusId, u16)>, status: StatusId, severity: u16) {
    match riders.iter_mut().find(|(s, _)| *s == status) {
        Some((_, sum)) => *sum = sum.saturating_add(severity),
        None => riders.push((status, severity)),
    }
}

/// One source's [`SpellMods`] with the tags it keys on: a spell carrying any of them (its
/// school, kind, shape, or damage type), or every spell when there are none.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpellMod {
    pub mods: SpellMods,
    pub when: Vec<String>,
}

impl SpellMod {
    /// Whether it acts on a spell with `tags`.
    pub fn fits(&self, tags: &[&str]) -> bool {
        self.when.is_empty() || self.when.iter().any(|w| tags.contains(&w.as_str()))
    }
}

/// A rider as data writes it.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RiderSpec {
    pub status: String,
    #[serde(default = "one")]
    pub severity: u16,
}

const fn one() -> u16 {
    1
}

/// `{ kind = "spell_mod", ... }` as a talent writes it (module docs).
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct SpellModSpec {
    pub rungs: i32,
    pub scroll_rungs: i32,
    pub wand_rungs: i32,
    pub reach_rungs: i32,
    pub degree_rungs: i32,
    pub sides: u32,
    pub dice: u32,
    pub bonus: i32,
    pub reroll: u32,
    pub rider: Option<RiderSpec>,
    pub cells: i32,
    pub radius_cells: i32,
    pub degrees: i32,
    pub cost: i32,
    pub when: Vec<String>,
}

impl SpellModSpec {
    /// Checked: something it does; its rider's status known, severity 1 to [`MAX_SEVERITY`];
    /// each tag of `when` a school, a kind, a shape, or a damage type; `when` left out on a
    /// talent `chosen` for a school or an element (the choice is its filter); its cost a
    /// percent. Nothing bounds how much it adds (the spell scaling design, decision 16).
    pub fn check(&self, content: &Content, chosen: bool) -> Result<(), String> {
        let numbers = [
            self.rungs,
            self.scroll_rungs,
            self.wand_rungs,
            self.reach_rungs,
            self.degree_rungs,
            self.bonus,
            self.cells,
            self.radius_cells,
            self.degrees,
            self.cost,
        ];
        let counts = [self.sides, self.dice, self.reroll];
        if numbers.iter().all(|n| *n == 0) && counts.iter().all(|n| *n == 0) && self.rider.is_none()
        {
            return Err("`spell_mod` does nothing: give it a field".into());
        }
        if !(-100..=100).contains(&self.cost) {
            return Err("`spell_mod`: `cost` is a percent, -100 to 100".into());
        }
        if let Some(rider) = &self.rider {
            if content.status_id(&rider.status).is_none() {
                return Err(format!("`spell_mod`: unknown status `{}`", rider.status));
            }
            if !(1..=MAX_SEVERITY).contains(&rider.severity) {
                return Err(format!("`spell_mod`: a rider's severity is 1 to {MAX_SEVERITY}"));
            }
        }
        if chosen && !self.when.is_empty() {
            return Err("`spell_mod`: a talent chosen for a school or an element keys on the \
                        choice; leave `when` out"
                .into());
        }
        if let Some(bad) = self.when.iter().find(|t| !is_tag(content, t)) {
            return Err(format!(
                "`spell_mod`: `when` tag `{bad}` is no school, kind, shape, or damage type"
            ));
        }
        Ok(())
    }

    /// Resolved for a talent taken for `choice` (its filter, when it has one).
    pub fn resolve(&self, content: &Content, choice: Option<&str>) -> SpellMod {
        let riders = (self.rider.iter())
            .filter_map(|r| content.status_id(&r.status).map(|s| (s, r.severity)))
            .collect();
        let when = match choice {
            Some(c) if self.when.is_empty() => vec![c.to_string()],
            _ => self.when.clone(),
        };
        SpellMod {
            mods: SpellMods {
                rungs: self.rungs,
                scroll_rungs: self.scroll_rungs,
                wand_rungs: self.wand_rungs,
                reach_rungs: self.reach_rungs,
                degree_rungs: self.degree_rungs,
                sides: self.sides,
                dice: self.dice,
                bonus: self.bonus,
                reroll: self.reroll,
                riders,
                cells: self.cells,
                radius_cells: self.radius_cells,
                degrees: self.degrees,
                cost: self.cost,
            },
            when,
        }
    }
}

/// Whether `tag` is one a spell may carry: a school, a kind, a shape, or a damage type.
fn is_tag(content: &Content, tag: &str) -> bool {
    super::School::ALL.iter().any(|s| s.id() == tag)
        || SpellPurpose::ALL.iter().any(|k| k.id() == tag)
        || Shape::ALL.iter().any(|s| s.id() == tag)
        || content.damage_type_id(tag).is_some()
}
