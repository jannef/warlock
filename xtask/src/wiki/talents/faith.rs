//! What the faith talents do, in the wiki's words (`warlock_core::talent::FaithEffect`, #1495): one
//! arm a variant, beside the area's enum and sum in `warlock-core/src/talent/faith.rs`.

use super::{prayer_text, resource_name};
use warlock_core::content::{Content, SmiteRiderKind};
use warlock_core::talent::FaithEffect;

/// What `effect` does, as the talents' tables say it.
pub(super) fn text(content: &Content, effect: &FaithEffect) -> String {
    let status = |id: &str| {
        (content.status_id(id)).map_or_else(|| id.to_string(), |s| content.status(s).name.clone())
    };
    match effect.clone() {
        FaithEffect::Pilgrim => {
            "temple services cost half (rounded up), and a cure also mends one point of each \
             damaged stat"
                .to_string()
        }
        FaithEffect::Devout => "at a forgotten altar of their own side the side's lean counts \
                                  double (30 points from bane to boon), and an opposed one leans \
                                  as a neutral one"
            .into(),
        FaithEffect::LayOnHands { per_level } => {
            format!("active (the time of a cast): {per_level} hit points per level back at once")
        }
        FaithEffect::Avatar { hands_cost_percent } => format!(
            "Lay on hands heals every hit point and cures every status that ticks (bleeding, \
             poison) and every disease, for {hands_cost_percent} % of its cost (at least 1); \
             and Holy wrath's die lands on every creature you hit, not only the unholy"
        ),
        FaithEffect::Smite {
            per_levels,
            sides,
            aim,
            unholy,
        } => {
            let aim = aim.map_or_else(String::new, |stat| {
                format!(", your {} modifier added to hit", stat.name())
            });
            let unholy = match (unholy, content.smite()) {
                (0, _) | (_, None) => String::new(),
                (n, Some(rules)) => format!(
                    ", and {n}d{sides} more against the unholy (a creature tagged {})",
                    rules.unholy.join(", ")
                ),
            };
            format!(
                "active (the time of a melee attack), in a direction: a blow with 1d{sides} \
                 more damage per {per_levels} levels (at least 1d{sides}){unholy}{aim}"
            )
        }
        FaithEffect::SmiteMode {
            mode,
            status: id,
            severity,
            turns,
            check,
            difficulty,
        } => {
            let rider = match (mode, turns, check) {
                (SmiteRiderKind::Banish, ..) => "a creature that was summoned is unmade, dying \
                     there whatever its hit points, your kill as any other; an unholy creature \
                     not summoned takes the Smite's dice twice over (the rest of the blow once); \
                     anyone else, a plain smite"
                    .to_string(),
                (SmiteRiderKind::Fear, Some(turns), Some(stat)) => format!(
                    "the foe checks {} (2d12 + its modifier) against {difficulty} plus your \
                     Smite's aim modifier; failed, it is {} by you for {turns} turns, fleeing \
                     you",
                    stat.name(),
                    status(&id)
                ),
                _ => {
                    let degrees = if severity == 1 { "degree" } else { "degrees" };
                    format!("the foe suffers {severity} {degrees} of {}", status(&id))
                }
            };
            format!(
                "active (the time of a melee attack), in a direction: your Smite's blow (its \
                 dice, aim, and unholy dice), and on a hit that does damage {rider}"
            )
        }
        FaithEffect::HolyWrath { dice } => format!(
            "every melee blow, shot, or throw of yours that hits an unholy creature (the undead \
             and the fiends) rolls {dice} more, of the blow's kind of damage, resisted but never \
             soaked, for no zeal"
        ),
        FaithEffect::Radiance { status: id, light } => {
            let reach = (content.light_id(&light)).map_or(0, |l| content.light(l).radius);
            format!(
                "active (the time of a cast), on and off: while you are {} you shed a light \
                 reaching {reach} cells",
                status(&id)
            )
        }
        FaithEffect::Communion => {
            "while the dark answers your prayers (Callous or worse), standing in an unholy place \
             (a cursed, draining, or haunted room, or a level such a curse covers): your prayers \
             add no strain, and the place's status (Ebbing) never touches you"
                .to_string()
        }
        FaithEffect::Retribution { resource, cost } => format!(
            "a hostile beside you whose melee attack lands a blow on you takes your plain smite \
             back at once (its dice, its aim, and its unholy die), free but for an exertion, spending {cost} {}; with less, or outside the talent's alignment, \
             nothing; an attack it answers is not also riposted",
            resource_name(content, &resource).to_lowercase()
        ),
        FaithEffect::Consecrate { mana, time } => format!(
            "active ({time} of prayer, for {mana} mana; interrupted as a rest is, it lifts \
             nothing): standing in a cursed or draining room, it becomes a plain room for good; \
             on a level a curse or a drain covers whole, the whole level's is lifted instead. \
             A good deed (see Alignment). Anywhere else it is refused, with no time spent"
        ),
        FaithEffect::Sanctuary {
            mana,
            time,
            radius,
            lasts,
            regen,
        } => format!(
            "active ({time} of kneeling, for {mana} mana; interrupted as a rest is, it lays \
             nothing): the ground within {radius} cells of where the player knelt is hallowed \
             for {lasts}. No creature hostile to them steps onto it from outside (one already \
             on it may move about and leave; nothing spawns there; missiles and spells cross \
             it), and whoever stands on it heals and regains mana at {regen} % of the pace, or \
             a special room's, the larger. A blow, shot, throw, or harmful spell of the \
             player's from on it breaks it; being struck does not. The same for either face of \
             the prayer (dreaded ground for the dark: the words alone). Refused, with no time \
             spent, on the overworld, in water, in a shop, or on hallowed ground"
        ),
        FaithEffect::Prayer {
            tier,
            title,
            uses,
            answers,
            all,
        } => prayer_text(content, tier, title.as_ref(), &uses, answers, all),
    }
}
