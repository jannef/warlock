//! What the beast talents do, in the wiki's words (`warlock_core::talent::BeastEffect`, #1495):
//! one arm a variant, beside the area's enum and sum in `warlock-core/src/talent/beast.rs`.

use crate::wiki::util::signed;
use warlock_core::talent::BeastEffect;

/// What `effect` does, as the talents' tables say it.
pub(super) fn text(effect: &BeastEffect) -> String {
    match effect {
        BeastEffect::Companion => "an [animal companion](companions.md#companions) from the first \
                                   turn: a beast of your race's kind, or rarely a rarer one, \
                                   that fights for you, always comes along, and grows with \
                                   your level"
            .into(),
        BeastEffect::Familiar => "a [familiar](familiars.md#familiars) from the first turn: a \
                                  small beast whose touch is your [evil eye](familiars.md#the-touch), \
                                  that goes for what you have hexed, always comes along, and \
                                  grows with your level"
            .into(),
        BeastEffect::Tame { bonus } => format!("{} on your taming checks", signed(*bonus)),
        BeastEffect::SharedSight => "you see what your [companion](companions.md#hunters-bond) \
                                     sees, by its own eyes, and the foes its nose finds, \
                                     through the dark and closed doors"
            .into(),
        BeastEffect::FamiliarSight => "you see what your [familiar](familiars.md#familiars-sight) \
                                       sees, by its own eyes, and the foes its nose finds, \
                                       through the dark and closed doors"
            .into(),
        BeastEffect::SharedBuffs => "a [buff](companions.md#hunters-bond) that lands on you or \
                                     your companion lands on the other too, at the same \
                                     strength; a hex never does"
            .into(),
        BeastEffect::PackTactics { to_hit } => format!(
            "a creature beside both you and your [companion](companions.md#companions) is flanked: \
             your melee attacks and your companion's at it, to hit {} (not shots, throws, or \
             spells)",
            signed(*to_hit)
        ),
        BeastEffect::SharesFavouredPrey => "your companion's attacks at your favoured prey \
                                            carry your Favoured prey bonus, to hit and damage \
                                            (Master hunter's too)"
            .into(),
        BeastEffect::BeastFriend => "creatures of animal factions are neutral to you until you \
                                     attack one of that type"
            .into(),
        BeastEffect::KinTo { tag } => format!(
            "creatures tagged *{}* are neutral to you until you attack one of that type, as \
             [Beast-friend](talents-exploring.md#talent-beast-friend)'s animals are",
            tag.replace('-', " ")
        ),
    }
}
