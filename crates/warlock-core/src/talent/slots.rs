//! The slot talents (docs/design/profane-gifts.md §6.4; #1628): what a talent does to the
//! equipment slots the holder's body has: a slot it bars (Horned's head) and one it opens (Many-
//! fingered's extra ring). [`SlotEffect`] is the data (a talent's `[[effect]]` of one of
//! [`SlotEffect::KINDS`]), [`SlotMods`] the sum every rule reads (`TalentMods::slots`). A new
//! effect of the area is a variant here, its field and its arm in [`SlotMods::add`] here, its
//! wiki text in `xtask/src/wiki/talents/slots.rs`, and its one read (`world/body_slots.rs`).
//!
//! The slots are named by id and resolved as the sum is made: a talent table is read before the
//! slots, so the loader checks them once both are in (`content::talent::check_slots`).

use crate::content::{Content, SlotId};
use serde::Deserialize;

/// What a talent of the area does, one typed effect, read by one rule each.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SlotEffect {
    /// Slot `slot` (an id in `data/slots.toml`, never a grip: a hand or a `ready` slot) cannot
    /// be used by the holder: nothing is put on there (refused with `refused`, no time), the
    /// equipment screen shows the row as `— (mark)`, and what is worn there falls as the talent
    /// takes (Horned's head).
    BarsSlot {
        slot: String,
        mark: String,
        refused: String,
    },
    /// Slot `slot` (an id in `data/slots.toml` of a slot gated to a class) is the holder's too,
    /// whatever their class (Many-fingered's extra ring).
    OpensSlot { slot: String },
}

impl SlotEffect {
    /// The `kind`s of the data that are effects of the area, one a variant (`TalentEffect`'s
    /// deserializer hands these to [`SlotEffect`]).
    pub const KINDS: [&'static str; 2] = ["bars_slot", "opens_slot"];

    /// The slot it names, by id.
    pub fn slot(&self) -> &str {
        match self {
            SlotEffect::BarsSlot { slot, .. } | SlotEffect::OpensSlot { slot } => slot,
        }
    }
}

/// A slot barred to the holder, with the words that say so.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Barred {
    pub slot: SlotId,
    /// The equipment screen's note in place of what is worn: "horns", shown `— (horns)`.
    pub mark: String,
    /// Said when something is put on there: "Your horns are in the way."
    pub refused: String,
}

/// What an actor's slot talents do, summed: one field per effect kind, read by one rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SlotMods {
    /// The slots it cannot use, the first talent's words for each (`world/body_slots.rs`).
    pub barred: Vec<Barred>,
    /// The class-gated slots it has too (`world/body_slots.rs`).
    pub opens: Vec<SlotId>,
}

impl SlotMods {
    /// Adds what `effect` does; a slot the content lacks does nothing.
    pub(crate) fn add(&mut self, content: &Content, effect: SlotEffect) {
        let Some(slot) = content.slot_id(effect.slot()) else {
            return;
        };
        match effect {
            SlotEffect::BarsSlot { mark, refused, .. } => {
                if !self.barred.iter().any(|b| b.slot == slot) {
                    self.barred.push(Barred {
                        slot,
                        mark,
                        refused,
                    });
                }
            }
            SlotEffect::OpensSlot { .. } => {
                if !self.opens.contains(&slot) {
                    self.opens.push(slot);
                }
            }
        }
    }

    /// The bar on slot `slot`, if it is barred.
    pub fn barred(&self, slot: SlotId) -> Option<&Barred> {
        self.barred.iter().find(|b| b.slot == slot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every kind in [`SlotEffect::KINDS`] is a variant of its own.
    #[test]
    fn kinds_are_the_variants() {
        let mut seen = Vec::new();
        for kind in SlotEffect::KINDS {
            let fields = match kind {
                "bars_slot" => "slot = \"head\"\nmark = \"horns\"\nrefused = \"No.\"",
                _ => "slot = \"extra_ring_1\"",
            };
            let effect: SlotEffect = toml::from_str(&format!("kind = \"{kind}\"\n{fields}"))
                .unwrap_or_else(|e| panic!("{kind}: {e}"));
            let variant = std::mem::discriminant(&effect);
            assert!(!seen.contains(&variant), "{kind} twice");
            seen.push(variant);
            match effect {
                SlotEffect::BarsSlot { .. } | SlotEffect::OpensSlot { .. } => {}
            }
        }
        assert_eq!(seen.len(), 2, "one kind a variant");
    }
}
