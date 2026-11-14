//! What the slot talents do, in the wiki's words (`warlock_core::talent::SlotEffect`, #1628): one
//! arm a variant, beside the area's enum and sum in `warlock-core/src/talent/slots.rs`.

use warlock_core::content::Content;
use warlock_core::talent::SlotEffect;

/// What `effect` does, as the talents' tables say it.
pub(super) fn text(content: &Content, effect: &SlotEffect) -> String {
    let name = |id: &str| {
        (content.slot_id(id)).map_or_else(|| id.to_string(), |s| content.slot(s).name.clone())
    };
    match effect {
        SlotEffect::BarsSlot { slot, .. } => format!(
            "nothing can be worn in the [{} slot](equipment.md#barred-and-opened-slots); what \
             was there falls to the floor",
            name(slot)
        ),
        SlotEffect::OpensSlot { slot } => format!(
            "the [{} slot](equipment.md#barred-and-opened-slots) is yours, whatever your class",
            name(slot)
        ),
    }
}
