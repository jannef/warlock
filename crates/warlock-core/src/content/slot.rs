//! Equipment slots (`assets/data/slots.toml`, spec §7): where actors wear and wield things, in
//! the order the equipment screen lists them (lettered `a` on), as in ADOM.
//!
//! Every slot has an `id` (objects say where they go with `slot = "<id>"`, replays name slots
//! by it: `remove:body`), a `name` for the screen, the action that puts a thing on there or takes
//! it off (its cost: `wield`, `wear`, `body_armor`, or `ready` in `data/actions.toml`), and the
//! `mark` the inventory gives a thing there ("being worn"). Slots may share what fits them
//! (`fits`, default the id): a ring fits either ring slot. The two `hand` slots are the hands: a
//! two-handed object takes both. A slot that `takes_weapons` also holds any weapon or
//! throwable (the missile weapon slot).

use super::{ActionKind, ContentError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Index of a slot in [`super::Content::slots`], in display order.
///
/// Ids are assigned in file order when content loads; saves record the content fingerprint to
/// catch a reordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SlotId(pub u8);

/// Most slots there may be: one per letter, `a` to `z`.
pub const MAX_SLOTS: usize = 26;

/// A slot as written in `data/slots.toml`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SlotSpec {
    id: String,
    name: String,
    /// What objects give as their `slot` to go here; the id if absent.
    fits: Option<String>,
    /// The action that puts a thing on here or takes it off: its cost.
    action: String,
    /// What the inventory says of a thing here, without parentheses: "being worn".
    mark: String,
    #[serde(default)]
    hand: bool,
    /// Any weapon and anything thrown fits, besides what `fits` names.
    #[serde(default)]
    takes_weapons: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SlotFile {
    slot: Vec<SlotSpec>,
}

/// A resolved slot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotDef {
    pub id: String,
    /// As the equipment screen shows it: "Main hand".
    pub name: String,
    /// What objects give as their `slot` to go here (`ring` for both ring slots).
    pub fits: String,
    /// The action that puts a thing on here or takes it off ([`ActionKind::is_equipment`]).
    pub action: ActionKind,
    /// What the inventory says of a thing here: "being worn" (shown as "(being worn)").
    pub mark: String,
    /// One of the hands: a two-handed object takes every hand slot.
    pub hand: bool,
    /// Any weapon (no missile weapon's ammunition) and any throwing weapon fits here too, besides
    /// what `fits` names: the missile weapon slot, whose contents `t` throws or shoots.
    pub takes_weapons: bool,
}

/// Parses `data/slots.toml`.
pub(super) fn parse(file: &str, text: &str) -> Result<Vec<SlotDef>, ContentError> {
    let err = |m: String| ContentError::new(file, m);
    let parsed: SlotFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
    if parsed.slot.len() > MAX_SLOTS {
        return Err(err(format!(
            "at most {MAX_SLOTS} slots (one per letter), not {}",
            parsed.slot.len()
        )));
    }
    let mut ids = BTreeSet::new();
    let mut slots = Vec::with_capacity(parsed.slot.len());
    for spec in parsed.slot {
        let bad = |m: &str| err(format!("slot `{}`: {m}", spec.id));
        if spec.id.trim().is_empty() || !ids.insert(spec.id.clone()) {
            return Err(err(format!(
                "slot ids must be unique and not empty: `{}`",
                spec.id
            )));
        }
        if spec.name.trim().is_empty() || spec.mark.trim().is_empty() {
            return Err(bad("`name` and `mark` must not be empty"));
        }
        let action = ActionKind::ALL
            .iter()
            .copied()
            .find(|k| k.is_equipment() && k.id() == spec.action)
            .ok_or_else(|| {
                bad("`action` must be `wield`, `wear`, `body_armor`, or `ready` (actions.toml)")
            })?;
        let fits = spec.fits.unwrap_or_else(|| spec.id.clone());
        if fits.trim().is_empty() {
            return Err(bad("`fits` must not be empty"));
        }
        slots.push(SlotDef {
            id: spec.id,
            name: spec.name,
            fits,
            action,
            mark: spec.mark,
            hand: spec.hand,
            takes_weapons: spec.takes_weapons,
        });
    }
    Ok(slots)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_parse_and_are_validated() {
        let text = r#"
            [[slot]]
            id = "left_ring"
            name = "Left ring"
            fits = "ring"
            action = "wear"
            mark = "on left hand"

            [[slot]]
            id = "main_hand"
            name = "Main hand"
            action = "wield"
            mark = "wielded"
            hand = true
        "#;
        let slots = parse("s", text).expect("valid");
        assert_eq!(slots[0].fits, "ring");
        assert_eq!(slots[1].fits, "main_hand", "fits defaults to the id");
        assert_eq!(slots[1].action, ActionKind::Wield);
        assert!(slots[1].hand && !slots[0].hand);
        let one = |fields: &str| format!("[[slot]]\nid = \"x\"\nname = \"X\"\n{fields}\n");
        for (text, message) in [
            (one("action = \"wait\"\nmark = \"m\""), "`action` must be"),
            (one("action = \"wear\"\nmark = \"\""), "must not be empty"),
            (
                one("action = \"wear\"\nmark = \"m\"\nfits = \" \""),
                "`fits`",
            ),
            (
                one("action = \"wear\"\nmark = \"m\"").repeat(2),
                "slot ids must be unique",
            ),
            (
                one("action = \"wear\"\nmark = \"m\"\nsize = 2"),
                "unknown field",
            ),
        ] {
            let err = parse("s", &text).unwrap_err();
            assert!(err.message.contains(message), "{text}: {}", err.message);
        }
        let many: String = (0..=MAX_SLOTS)
            .map(|i| {
                format!("[[slot]]\nid = \"s{i}\"\nname = \"S\"\naction = \"wear\"\nmark = \"m\"\n")
            })
            .collect();
        assert!(
            parse("s", &many)
                .unwrap_err()
                .message
                .contains("at most 26")
        );
    }
}
