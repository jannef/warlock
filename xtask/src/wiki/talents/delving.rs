//! What the delving talents do, in the wiki's words (`warlock_core::talent::DelvingEffect`, #1495):
//! one arm a variant, beside the area's enum and sum in `warlock-core/src/talent/delving.rs`.

use crate::wiki::util::signed;
use warlock_core::content::Content;
use warlock_core::talent::DelvingEffect;

/// What `effect` does, as the talents' tables say it.
pub(super) fn text(content: &Content, effect: &DelvingEffect) -> String {
    let percent = |v: i32| format!("{} %", signed(v));
    let status = |id: &str| {
        (content.status_id(id)).map_or_else(|| id.to_string(), |s| content.status(s).name.clone())
    };
    match effect.clone() {
        DelvingEffect::Notice { value } => {
            format!(
                "notices hidden things in passing, {} to the check",
                signed(value)
            )
        }
        DelvingEffect::SpawnEveryPercent { value } => format!(
            "the time between attempts to bring a creature to your level {}",
            percent(value)
        ),
        DelvingEffect::ChestWise => "sees whether a shut chest in sight is locked (look mode: \
            \"a locked coffer\", \"an unlocked coffer\"), and knows the trap of every shut chest \
            it sees, no check"
            .into(),
        DelvingEffect::RoomPacePercent { value } => format!(
            "where a special room quickens healing or mana, the quickening {} (a room's 300% \
             pace becomes {}%)",
            percent(value),
            100 + 200 * (100 + value) / 100
        ),
        DelvingEffect::RoomStatusImmune { status: id } => format!(
            "a special room's (or a whole level's) {} never takes you",
            status(&id)
        ),
        DelvingEffect::GraveWarden => {
            "nothing you kill ever rises in a cursed room (see Special rooms)".into()
        }
        DelvingEffect::StoneSense => "notices hidden things on stone in passing".into(),
        DelvingEffect::StoneFind { value } => format!(
            "{} to every check to find what is hidden on stone, searching and in passing",
            signed(value)
        ),
        DelvingEffect::Search { value } => {
            format!("{} to every check to find hidden things", signed(value))
        }
        DelvingEffect::Searching { value } => format!("searching {}", signed(value)),
        DelvingEffect::CollapseProof => "a collapsed floor never gives way under you".to_string(),
        DelvingEffect::SeesMimics { value } => format!(
            "sees a disguised mimic within {value} cell{} for what it is: it is shown as itself, \
             not as a chest",
            if value == 1 { "" } else { "s" }
        ),
        DelvingEffect::Slick => "a mimic's grip (its slam's hold) never holds it".into(),
        DelvingEffect::HoardGrace { value } => format!(
            "a tempting hoard seals only once you have lifted {} stack{} inside it",
            1 + value,
            if value == 0 { "" } else { "s" }
        ),
        DelvingEffect::SearchReach { value } => format!(
            "searching reaches {value} cell{} farther",
            if value == 1 { "" } else { "s" }
        ),
        DelvingEffect::TrapDamagePercent { value } => format!(
            "damage traps do you {} (rounded down, at least 1)",
            percent(value)
        ),
        DelvingEffect::DoorKick { value } => {
            format!("door kicks as if Strength {}", signed(value))
        }
        DelvingEffect::KickNoisePercent { value } => {
            format!("door kicks' noise {}", percent(value))
        }
        DelvingEffect::SmashDestroyPercent { value } => format!(
            "the chance that each stack in a chest it kicks open breaks {} (a chest blown apart \
             is no kick)",
            percent(value)
        ),
        DelvingEffect::DigPercent { value } => format!("digging work a turn {}", percent(value)),
        DelvingEffect::SureStep => "a trap you step onto first gets a Dexterity check against \
             its difficulty: made, it does not go off, and you find it"
            .to_string(),
        DelvingEffect::PitProof => "pits never catch you (you find them)".to_string(),
        DelvingEffect::Disarm => {
            "the Disarm ability without Disable device (Dexterity alone)".to_string()
        }
        DelvingEffect::TrapEase { value } => {
            format!("hidden traps {value} easier to find (not to disarm)")
        }
    }
}
