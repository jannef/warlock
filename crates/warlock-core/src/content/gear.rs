//! Creature gear tables (spec §8 "Creature gear"): what a creature of a type may spawn carrying,
//! wielding, and wearing. A type's `racial_weapon` and `[[creature.gear]]` lines
//! (`data/creatures/README.toml`); [`crate::gear`] rolls them.
//!
//! A line is `chance` (percent), `one_of` (objects as level files place them, one drawn
//! uniformly; `"arrow*20"`), `keep` (`wield`, `wear`, or `carry`; default by the object: what a
//! hand, the missile weapon or missiles slot, or the Tool slot takes is wielded, what another
//! slot takes is worn, the rest carried), and `magic` (the percent that the piece is made magic
//! as loot is, default 0). The racial weapon is the first line, `{ chance = 90, one_of =
//! [racial_weapon], keep = "wield" }`, unless a line of the type's names it alone. What fits no
//! slot as kept is carried (`Pack::equip`), and a piece that is never magic is made plain.

use super::kit::Keep;
use super::{Content, ObjectId, object};
use crate::equipment::{self, Equipment};
use crate::object::Stack;
use serde::Deserialize;

/// The chance in percent that a creature carries its racial weapon, unless its type writes the
/// line itself.
pub const RACIAL_CHANCE: u8 = 90;

/// A gear line as written in a creature's file.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GearLineSpec {
    chance: u8,
    one_of: Vec<String>,
    keep: Option<Keep>,
    #[serde(default)]
    magic: u8,
}

/// One line of a creature type's gear table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GearLine {
    /// The chance in percent that the line gives anything.
    pub chance: u8,
    /// The alternatives, one drawn: the stacks each places (one, but for several of a thing
    /// that does not stack, which are only carried), and how they are kept.
    pub one_of: Vec<(Vec<Stack>, Keep)>,
    /// The chance in percent that what it gives is made magic.
    pub magic: u8,
}

/// The type's gear as written: its racial weapon and its lines.
#[derive(Clone, Debug, Default)]
pub struct GearSpec {
    pub racial_weapon: Option<String>,
    pub lines: Vec<GearLineSpec>,
}

/// A creature type's gear, resolved: its racial weapon, and the lines tried in order (the racial
/// weapon's first, unless the type wrote it).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Gear {
    pub racial_weapon: Option<ObjectId>,
    pub lines: Vec<GearLine>,
}

impl Gear {
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

/// How one of `stacks` is kept by default: what goes in a hand, the missile weapon or missiles
/// slot, or the Tool slot is wielded, what goes in another slot worn, the rest (and several of
/// a thing that does not stack) carried.
fn default_keep(stacks: &[Stack], content: &Content) -> Keep {
    let [stack] = stacks else {
        return Keep::Carry;
    };
    match equipment::free_slot(content, &Equipment::default(), stack.object) {
        None => Keep::Carry,
        Some(slot) => match content.slot(slot).action {
            super::ActionKind::Wield | super::ActionKind::Ready => Keep::Wield,
            _ => Keep::Wear,
        },
    }
}

/// Resolves a creature type's gear.
pub(super) fn resolve(spec: &GearSpec, content: &Content) -> Result<Gear, String> {
    let mut lines = Vec::new();
    let racial_weapon = match &spec.racial_weapon {
        None => None,
        Some(name) => {
            let id = content
                .object_id(name)
                .ok_or_else(|| format!("racial_weapon: unknown object `{name}`"))?;
            let written = (spec.lines.iter()).any(|l| l.one_of.len() == 1 && l.one_of[0] == *name);
            if !written {
                lines.push(GearLine {
                    chance: RACIAL_CHANCE,
                    one_of: vec![(vec![Stack::new(id, 1)], Keep::Wield)],
                    magic: 0,
                });
            }
            Some(id)
        }
    };
    for line in &spec.lines {
        let at = |m: String| format!("gear `{}`: {m}", line.one_of.join("|"));
        // The draw indexes the alternatives.
        if line.one_of.is_empty() {
            return Err(at("`one_of` names at least one object".into()));
        }
        let mut one_of = Vec::new();
        for placed in &line.one_of {
            let stacks = object::parse_stacks(std::slice::from_ref(placed), content).map_err(at)?;
            let keep = line.keep.unwrap_or_else(|| default_keep(&stacks, content));
            one_of.push((stacks, keep));
        }
        lines.push(GearLine {
            chance: line.chance,
            one_of,
            magic: line.magic,
        });
    }
    Ok(Gear {
        racial_weapon,
        lines,
    })
}
