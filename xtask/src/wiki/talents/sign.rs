//! What the sign talents do, in the wiki's words (`warlock_core::talent::SignEffect`, #1495): one
//! arm a variant, beside the area's enum and sum in `warlock-core/src/talent/sign.rs`.

use super::{cooldown_for, sign_cooldown};
use warlock_core::content::{Content, SignActive};
use warlock_core::talent::SignEffect;

/// What `effect` does, as the talents' tables say it.
pub(super) fn text(content: &Content, effect: &SignEffect) -> String {
    let status = |id: &str| {
        (content.status_id(id)).map_or_else(|| id.to_string(), |s| content.status(s).name.clone())
    };
    match effect.clone() {
        SignEffect::Blaze {
            radius,
            damage,
            damage_type,
            fire,
            burns,
            cooldown,
            ..
        } => {
            let kind = (content.damage_type_id(&damage_type)).map_or(damage_type.clone(), |d| {
                content.damage_type(d).name.to_lowercase()
            });
            let light = (content.light_id(&fire)).map_or(0, |l| content.light(l).radius);
            let until = (content.status_id(&cooldown))
                .and_then(|s| content.status(s).eases)
                .map_or_else(String::new, |s| format!(" for {} h", s / 3600));
            format!(
                "active (the time of a cast; not on the overworld): a burst of radius {radius} \
                 around you, {damage} {kind} damage (one roll) to every creature in it but you, \
                 whole to each; it melts the ice it reaches and leaves a fire on every walkable \
                 cell of it but yours, its light reaching {light} cells, for {burns} s; then \
                 {}{until}",
                status(&cooldown)
            )
        }
        SignEffect::HornsGrace {
            cures,
            diseases,
            cooldown,
            degrees,
        } => {
            let names: Vec<String> = cures.iter().map(|c| status(c)).collect();
            format!(
                "active (the time of a wait): {} end and the first {diseases} of the diseases \
                 that show are shaken off (refused with nothing to mend); then {}",
                names.join(" and "),
                cooldown_for(content, &cooldown, degrees)
            )
        }
        SignEffect::HornOfPlenty { per_level, degrees } => format!(
            "Horn's grace also gives back {per_level} hit point per level (never past your \
             maximum; it may then be used with only wounds to mend), and leaves {} instead",
            sign_cooldown(content, SignActive::HornsGrace, degrees)
        ),
        SignEffect::Stare {
            status: id,
            severity,
            cooldown,
            degrees,
        } => {
            let (cost, eases) = (content.status_id(&id)).map_or((100, None), |s| {
                let def = content.status(s);
                (def.cost, def.eases)
            });
            let eases =
                eases.map_or_else(String::new, |e| format!(", a degree easing every {e} s"));
            format!(
                "active (the time of a wait, silent): a hostile you see within {} cells that \
                 sees you suffers {} at {severity} degrees (its actions cost {cost} %{eases}); \
                 one that does not see you is not held, and the cooldown waits; then {}",
                warlock_core::world::STARE_RANGE,
                status(&id),
                cooldown_for(content, &cooldown, degrees)
            )
        }
        SignEffect::PetrifyingGaze { severity, degrees } => format!(
            "the Transfixing stare leaves {severity} degrees of its status, and leaves {} instead",
            sign_cooldown(content, SignActive::TransfixingStare, degrees)
        ),
        SignEffect::Wildfire { radius, damage } => {
            format!("Blaze bursts out to radius {radius} and does {damage} instead")
        }
        SignEffect::FireBreath {
            length,
            damage,
            damage_type,
            cooldown,
            degrees,
            ..
        } => {
            let kind = (content.damage_type_id(&damage_type)).map_or(damage_type.clone(), |d| {
                content.damage_type(d).name.to_lowercase()
            });
            format!(
                "active (the time of a cast; not on the overworld), aimed at a cell within {} \
                 cells: a bolt along the line toward it and on, {length} cells long or to the \
                 first wall, {damage} {kind} damage (one roll) to everyone on it; then {}",
                warlock_core::world::BREATH_RANGE,
                cooldown_for(content, &cooldown, degrees)
            )
        }
        SignEffect::HoundsBay {
            status: id,
            severity,
            cooldown,
        } => {
            let eases = (content.status_id(&id))
                .and_then(|s| content.status(s).eases)
                .map_or_else(String::new, |e| format!(", a degree easing every {e} s"));
            format!(
                "active (the time of a wait; not on the overworld): a howl of loudness {}; \
                 every hostile that hears it and whose level is below yours suffers {} at \
                 {severity} degrees (it runs from you while a step takes it farther{eases}); \
                 no check; then {}",
                warlock_core::noise::HOWL_NOISE,
                status(&id),
                cooldown_for(content, &cooldown, 1)
            )
        }
    }
}
