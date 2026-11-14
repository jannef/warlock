//! What the stealth talents do, in the wiki's words (`warlock_core::talent::StealthEffect`, #1495):
//! one arm a variant, beside the area's enum and sum in `warlock-core/src/talent/stealth.rs`.

use crate::wiki::util::signed;
use warlock_core::content::Content;
use warlock_core::talent::StealthEffect;

/// What `effect` does, as the talents' tables say it.
pub(super) fn text(content: &Content, effect: &StealthEffect) -> String {
    let percent = |v: i32| format!("{} %", signed(v));
    let status = |id: &str| {
        (content.status_id(id)).map_or_else(|| id.to_string(), |s| content.status(s).name.clone())
    };
    match effect.clone() {
        StealthEffect::FootstepsPercent { value } => format!("footsteps {}", percent(value)),
        StealthEffect::SilentDoors => "the doors you open and close are silent".to_string(),
        StealthEffect::Ambush { dice } => format!(
            "+{dice} damage, after Protection, on a hit at a creature that has not seen you \
             (it hunts no one it has seen, and does not see you now)"
        ),
        StealthEffect::InitiativeBonus { value } => format!(
            "{} to the initiative check when something catches you unawares (see Status effects: \
             Surprised)",
            signed(value)
        ),
        StealthEffect::SurpriseEase { value } => format!(
            "{value}% of the extra time the action you take Surprised costs is spared (at 100%, \
             it costs no more than any other)"
        ),
        StealthEffect::SurpriseDifficulty { value } => format!(
            "those you catch unawares make their initiative check at {} difficulty",
            signed(value)
        ),
        StealthEffect::SneakReach { value } => {
            format!("your shots, throws, and attack spells sneak-attack {value} cells farther")
        }
        StealthEffect::SneakKillStatus { status: id } => format!(
            "a sneak attack of yours that kills leaves you {} (1 degree)",
            status(&id)
        ),
        StealthEffect::SeesUnaware => "you see which creatures in view have not noticed you: \
                                      look mode says \"unaware of you\", and their glyphs are \
                                      drawn a shade cooler, breathing slowly (standing still \
                                      with reduced effects)"
            .to_string(),
        StealthEffect::SilentUnburdened => "no footsteps at all while Unburdened".to_string(),
        StealthEffect::AttackNoisePercent { value } => {
            format!("the noise of your attacks {}", percent(value))
        }
        StealthEffect::PickLock => "the Pick lock ability without Open lock".to_string(),
        StealthEffect::Steal => "the Steal ability without the Steal skill".to_string(),
        StealthEffect::ThrowVoice => "the Throw your voice ability".to_string(),
        StealthEffect::AmbushThreat { value } => format!(
            "attacks on a creature that has not seen you threaten a critical {value} lower \
             (never below 3)"
        ),
        StealthEffect::Lurk { status: id } => format!(
            "active (the time of a wait): you are {} until your next action; a hostile that steps \
             up beside you meanwhile takes your blow at once (a free blow), caught unawares \
             unless it was already hunting you; refused with a hostile you see beside you",
            status(&id)
        ),
        StealthEffect::Exploit => "active (the time of a melee attack), in a direction: every \
                                  opening you hold spent on one blow that is a sneak attack \
                                  whatever the foe knows (the sneak attack dice on a hit; the foe \
                                  is not surprised and keeps its Defence); refused with no \
                                  opening, at another than the creature your openings are in, \
                                  and at a thing you see without vitals"
            .to_string(),
        StealthEffect::ThrownExploit => "active (the time of a throw), something to throw and a \
                                        creature you see: Exploit's blow as a throw, every \
                                        opening you hold spent on a sneak attack whatever the \
                                        foe knows, with every increment of an exploit; within \
                                        the item's throw range and 6 cells (Long reach's \
                                        farther), the line clear as far as you see; refused as \
                                        Exploit is, and for a flask"
            .to_string(),
        StealthEffect::ExploitDice { dice } => format!(
            "an exploit that hits does +{dice} damage per opening spent, after Protection, never \
             multiplied by a critical"
        ),
        StealthEffect::ExploitToHit { value } => {
            format!("an exploit is {} to hit per opening spent", signed(value))
        }
        StealthEffect::ExploitStatus { status: id } => format!(
            "an exploit that hits leaves the foe {} (1 degree)",
            status(&id)
        ),
        StealthEffect::ExploitThreat { value, sure_at } => format!(
            "an exploit threatens a critical {value} lower per opening spent (never below 2); \
             with {sure_at} or more spent, it is a critical whenever it hits, with no \
             confirmation roll"
        ),
    }
}
