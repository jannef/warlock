//! What the movement talents do, in the wiki's words (`warlock_core::talent::MovementEffect`,
//! #1495): one arm a variant, beside the area's enum and sum in
//! `warlock-core/src/talent/movement.rs`.

use crate::wiki::util::signed;
use warlock_core::content::Content;
use warlock_core::talent::MovementEffect;

/// What `effect` does, as the talents' tables say it.
pub(super) fn text(content: &Content, effect: &MovementEffect) -> String {
    let percent = |v: i32| format!("{} %", signed(v));
    match effect.clone() {
        MovementEffect::CarryPercent { value } => format!("carrying limit {}", percent(value)),
        MovementEffect::IgnoreBurdened => {
            "being Burdened costs you nothing (as a dwarf)".to_string()
        }
        MovementEffect::WeightlessCoins => "coins you carry weigh nothing".to_string(),
        MovementEffect::TravelPercent { value } => {
            format!("overworld journey time {}", percent(value))
        }
        MovementEffect::Swimmer => {
            "wading (water, marsh) costs a plain step and never gives a disease".into()
        }
        MovementEffect::TerrainAs { terrains, like } => {
            let name = |id: &str| {
                (content.terrain_id(id))
                    .map_or_else(|| id.to_string(), |t| content.terrain(t).name.clone())
            };
            let names: Vec<String> = terrains.iter().map(|t| name(t)).collect();
            format!(
                "a step onto {} costs what one onto {} does",
                names.join(", "),
                name(&like)
            )
        }
        MovementEffect::PlungeProof => "a plunge through the ice does you no harm".to_string(),
        MovementEffect::LightOnIce => {
            "ice holds you as if you carried nothing (your own weight only)".to_string()
        }
        MovementEffect::ClimbPercent { value } => {
            format!("climbing out onto ice costs {}", percent(value))
        }
        MovementEffect::WadingProof => "wading never gives you a disease".to_string(),
        MovementEffect::StepPercent {
            value,
            unarmoured,
            unburdened,
            no_shield,
        } => {
            let mut when = Vec::new();
            if unarmoured {
                when.push("in no body armour");
            }
            if unburdened {
                when.push("Unburdened");
            }
            if no_shield {
                when.push("with no shield");
            }
            let when = if when.is_empty() {
                String::new()
            } else {
                format!(" while {}", when.join(" and "))
            };
            format!("steps cost {}{when}", percent(value))
        }
        MovementEffect::Stout => {
            "Strained costs you what Burdened costs others (slowdown, Dexterity cap, noise)"
                .to_string()
        }
        MovementEffect::WadesFree => "wading (water, and ground a disease is caught wading in) \
             costs you a plain step; a disease may still be caught"
            .to_string(),
        MovementEffect::BurdenDex { value } => {
            format!("the burden's cap on Dexterity in Defence {}", signed(value))
        }
        MovementEffect::BurdenEase { value } => format!(
            "{value} % off the burden's slowdown, on top of Balance (Burdened ×1.10 for 60 %)"
        ),
        MovementEffect::SpeedPercent { value } => format!("speed {}", percent(value)),
    }
}
