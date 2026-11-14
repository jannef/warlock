//! What the defence talents do, in the wiki's words (`warlock_core::talent::DefenceEffect`, #1495):
//! one arm a variant, beside the area's enum and sum in `warlock-core/src/talent/defence.rs`.

use crate::wiki::util::signed;
use warlock_core::content::Content;
use warlock_core::talent::DefenceEffect;

/// What `effect` does, as the talents' tables say it.
pub(super) fn text(content: &Content, effect: &DefenceEffect) -> String {
    let percent = |v: i32| format!("{} %", signed(v));
    match effect.clone() {
        DefenceEffect::Dodge { value } => format!("Dodge {}", signed(value)),
        DefenceEffect::Natural { value } => format!("natural armour {}", signed(value)),
        DefenceEffect::StanceDodge { stance, percent } => {
            format!("{} stance's Dodge penalty {percent} % smaller", stance.id())
        }
        DefenceEffect::StanceCosts => {
            "changing your combat stance takes the time of a wait (it is free otherwise)".into()
        }
        DefenceEffect::MaxDex { value } => {
            format!("worn armour's Dexterity cap {}", signed(value))
        }
        DefenceEffect::Deflection { value } => format!("Deflection {}", signed(value)),
        DefenceEffect::Protection {
            value,
            body_armour,
            shield,
            per_piece,
        } => {
            let mut text = format!("Protection {}", signed(value));
            if per_piece {
                text +=
                    " for every piece of armour worn (anything worn with Protection of its own)";
            }
            if let Some(at) = body_armour {
                text += &format!(
                    " while the armour piece worn with the most Protection has {at} or more"
                );
            }
            if shield {
                text += " while holding a shield";
            }
            text
        }
        DefenceEffect::StanceToHit { stance, percent } => {
            format!(
                "{} stance's to-hit penalty {percent} % smaller",
                stance.id()
            )
        }
        DefenceEffect::Pain { value } => {
            format!(
                "every hit on you {} damage after the soak, never below 1",
                signed(-value)
            )
        }
        DefenceEffect::Resist {
            value,
            damage: None,
        } => format!("resistance to the chosen element +{value} %"),
        DefenceEffect::Resist {
            value,
            damage: Some(kind),
        } => {
            let name = (content.damage_type_id(&kind))
                .map_or(kind.clone(), |d| content.damage_type(d).name.to_lowercase());
            format!("resistance to {name} +{value} %")
        }
        DefenceEffect::SlipPercent { value } => format!(
            "a step out of reach of every hostile beside you costs {}, whatever your stance",
            percent(value)
        ),
        DefenceEffect::Barbs { damage } => format!(
            "a creature that hits you in melee takes {damage} damage (no roll; a death of it \
             earns no experience)"
        ),
        DefenceEffect::MissileDodge { value } => {
            format!(
                "Dodge {} against shots, throws, and trap missiles",
                signed(value)
            )
        }
        DefenceEffect::SmallTarget { value } => format!(
            "Dodge {} against the attacks of creatures larger than you",
            signed(value)
        ),
        DefenceEffect::SpellDamagePercent { value } => format!(
            "damage from bursts and bolts (traps, scrolls) {} (rounded down)",
            percent(value)
        ),
        DefenceEffect::DeflectionUnarmoured { value } => {
            format!("Deflection {} while you wear no body armour", signed(value))
        }
    }
}
