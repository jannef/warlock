//! What the martial talents do, in the wiki's words (`warlock_core::talent::MartialEffect`, #1495):
//! one arm a variant, beside the area's enum and sum in `warlock-core/src/talent/martial.rs`.

use super::resource_name;
use crate::wiki::util::signed;
use warlock_core::content::Content;
use warlock_core::talent::MartialEffect;

/// What `effect` does, as the talents' tables say it.
pub(super) fn text(content: &Content, effect: &MartialEffect) -> String {
    let status = |id: &str| {
        (content.status_id(id)).map_or_else(|| id.to_string(), |s| content.status(s).name.clone())
    };
    match effect.clone() {
        MartialEffect::VeteransWind {
            per_level,
            at_least,
            cooldown,
        } => format!(
            "active (the time of a wait): {per_level} hit point per level back at once, at \
             least {at_least}; then {}",
            status(&cooldown)
        ),
        MartialEffect::Rage { resource: id } => {
            let rage = content.resource_id(&id).map(|r| content.resource(r));
            let gain = rage.map_or(id.clone(), |r| {
                format!("{} {}", r.start, r.name.to_lowercase())
            });
            let brings = rage.and_then(|r| r.status).map_or_else(String::new, |s| {
                format!(
                    ", and you are {} while you hold any",
                    content.status(s).name
                )
            });
            format!(
                "active (the time of a wait): you gain {gain} (see [Resources](resources.md)){brings}; no rage \
                 while it or what follows it lasts"
            )
        }
        MartialEffect::WarCry {
            status: id,
            severity,
            difficulty,
        } => {
            let eases = (content.status_id(&id))
                .and_then(|s| content.status(s).eases)
                .map_or_else(String::new, |e| format!(", a degree easing every {e} s"));
            format!(
                "active (the time of a wait): a roar of loudness {}; every hostile that hears \
                 it, sees you, and has vitals rolls its Willpower against {difficulty} + your \
                 Strength modifier + your Intimidate (see Skills); failed, it suffers {} at \
                 {severity} degrees (it runs from you while a step takes it farther{eases}). \
                 No level cut; with nobody to hear it the roar is still made, its time and cost \
                 spent",
                warlock_core::noise::HOWL_NOISE,
                status(&id),
            )
        }
        MartialEffect::Unstoppable { statuses, cost } => {
            let names: Vec<String> = statuses.iter().map(|s| status(s)).collect();
            format!(
                "while raging: {} tick nothing, and a blow that would take your last hit point \
                 leaves it for {cost} rage, when you have that much (spending the last of it \
                 ends the rage)",
                names.join(" and ")
            )
        }
        MartialEffect::Charge { per_cell } => format!(
            "active, aimed at a hostile you see: you rush along the straight line to it and \
             strike it once, ending beside it, for {per_cell} more stamina a cell moved (at \
             least 1 cell, the time of the steps, the blow free); creatures passed or left get \
             no free blows, and a wall, a closed door, deep water, a known trap, or a creature \
             you see on the line refuses it"
        ),
        MartialEffect::WhirlingCharge => {
            "your Charge also strikes every hostile beside each cell you pass, in the order \
             passed, each once and the one you charge at only its own blow: free blows, no \
             time, no stamina; one that falls does not stop the charge"
                .to_string()
        }
        MartialEffect::QuiveringPalm {
            status: id,
            severity,
        }
        | MartialEffect::StunningStrike {
            status: id,
            severity,
        } => format!(
            "active (the time of a melee attack), in a direction, nothing in hand: a bare-handed \
             blow; a hit that does damage leaves {severity} degrees of {}",
            status(&id)
        ),
        MartialEffect::WindStep { status: id } => format!(
            "active, taking no time: a degree of {}, so the next action, whatever it is, costs \
             half its time; refused while you are {} already",
            status(&id),
            status(&id)
        ),
        MartialEffect::Stillness { name, statuses } => {
            let names: Vec<String> = statuses.iter().map(|s| status(s)).collect();
            format!(
                "active ({name}), taking no time: whichever of {} you suffer ends at once; \
                 refused with none of them to end",
                names.join(", ")
            )
        }
        MartialEffect::PatientDefence { status: id } => format!(
            "active (the time of a wait): you are {} until your next action; a melee blow you \
             dodge (it misses by your Dexterity or Dodge) or parry meanwhile is answered with \
             your blow at once (a riposte, a free blow)",
            status(&id)
        ),
        MartialEffect::RapidShot {
            resource,
            cost,
            saving,
        } => format!(
            "a shot you loose while you hold {cost} {} spends them and takes {saving} % less time",
            resource_name(content, &resource).to_lowercase()
        ),
        MartialEffect::FlurryCost { resource, cost } => format!(
            "each second blow of the flurry spends {cost} {}; with less, there is none",
            resource_name(content, &resource).to_lowercase()
        ),
        MartialEffect::CatchMissiles { resource, cost } => format!(
            "a shot or throw (a trap's too) that would hit you while you hold no weapon and no \
             shield spends {cost} {} by itself and misses, the missile falling at your feet; \
             with less, it hits",
            resource_name(content, &resource).to_lowercase()
        ),
        MartialEffect::Volley { shots } => format!(
            "active (the time of one shot, one exertion), at a cell: {shots} shots of the readied \
             missile weapon"
        ),
        MartialEffect::SpiritWeapon {
            to_hit,
            threat,
            ignores_protection,
            ..
        } => {
            let armour = match ignores_protection {
                true => ", and ignores every point of the target's Protection",
                false => "",
            };
            format!(
                "active (in the shape of any melee weapon, for the time of wielding one, {} at \
                 normal pace, with what the hands held put away as any swap; no mana, cooldown, \
                 or check, as often as you like): a weapon of spirit-stuff of that shape, made \
                 and weighing as its plain base, is put in your main hand (both, for a two-handed \
                 one), and one you hold fades first. It strikes {} to hit, threatens criticals \
                 {threat} lower{armour}; it fades the moment it leaves your hands (put away, \
                 swapped, dropped, sold) and cannot be thrown",
                crate::wiki::util::at_normal_pace(warlock_core::content::price(
                    content
                        .action_costs()
                        .cost(warlock_core::content::ActionKind::Wield),
                    warlock_core::content::BASE_TURN
                )),
                signed(to_hit),
            )
        }
    }
}
