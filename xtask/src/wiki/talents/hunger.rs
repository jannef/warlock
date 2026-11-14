//! What the hunger talents do, in the wiki's words (`warlock_core::talent::HungerEffect`, #1495):
//! one arm a variant, beside the area's enum and sum in `warlock-core/src/talent/hunger.rs`.

use crate::wiki::util::signed;
use warlock_core::content::Content;
use warlock_core::talent::HungerEffect;

/// What `effect` does, as the talents' tables say it.
pub(super) fn text(content: &Content, effect: &HungerEffect) -> String {
    let percent = |v: i32| format!("{} %", signed(v));
    match effect.clone() {
        HungerEffect::HungerPercent { value } => format!("hunger pace {}", percent(value)),
        HungerEffect::SmellRot => {
            "the eat list says whether food smells fresh, turning (old), or rotten".to_string()
        }
        HungerEffect::RotHarmPercent { value } => {
            format!(
                "sickness from rotten food {} (rounded down)",
                percent(value)
            )
        }
        HungerEffect::NoExertion => "blows, shots, and throws burn no nutrition".to_string(),
        HungerEffect::NoBloating => "a meal never leaves you Bloated: what does not fit below \
             Bloated is lost, and you never overeat"
            .to_string(),
        HungerEffect::NoHungerWeakness => {
            "being [Weak, Fainting, or Starving](hunger.md#how-hungry-you-are) \
             takes nothing off your Strength and Dexterity, and you never faint from hunger; \
             Starving still stops your healing and costs hit points"
                .to_string()
        }
        HungerEffect::CorpseStatsPercent { value } => format!(
            "a fresh corpse's chance to raise each stat {} (at most 100 %)",
            percent(value)
        ),
        HungerEffect::EatsRotten => "rotten food never harms you: no vomiting, no sickness, no \
             disease (as a race that eats rotten food)"
            .to_string(),
        HungerEffect::IronStomach => "rotten food never makes you vomit and sickens you half \
             as much (degrees halved, rounded down); overeating never makes you vomit"
            .to_string(),
        HungerEffect::Purify => {
            "rotten food never sickens you and never exposes you to a disease (it may still make \
             you vomit)"
                .to_string()
        }
        HungerEffect::CorpseHarmless => {
            "a fresh corpse's own harm (its sickness and its vomiting) never touches you"
                .to_string()
        }
        HungerEffect::MealHeals { object, dice } => {
            let what = object.map_or_else(
                || "a fresh corpse".to_string(),
                |o| (content.object_id(&o)).map_or(o, |id| content.object(id).name.clone()),
            );
            format!("eating {what} heals {dice} hit points besides")
        }
    }
}
