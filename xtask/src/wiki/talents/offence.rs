//! What the offence talents do, in the wiki's words (`warlock_core::talent::OffenceEffect`, #1495):
//! one arm a variant, beside the area's enum and sum in `warlock-core/src/talent/offence.rs`.

use crate::wiki::util::signed;
use warlock_core::content::{Content, Role};
use warlock_core::talent::OffenceEffect;

/// What `effect` does, as the talents' tables say it.
pub(super) fn text(content: &Content, effect: &OffenceEffect) -> String {
    let percent = |v: i32| format!("{} %", signed(v));
    let status = |id: &str| {
        (content.status_id(id)).map_or_else(|| id.to_string(), |s| content.status(s).name.clone())
    };
    match effect.clone() {
        OffenceEffect::MeleeDamage { value } => format!("melee damage {}", signed(value)),
        OffenceEffect::Ferocity { value } => format!(
            "damage {} with every attack while below a third of your hit points",
            signed(value)
        ),
        OffenceEffect::Threat { value } => {
            format!("threat range {} with every attack", signed(value))
        }
        OffenceEffect::RerollFumble => {
            "a natural 2 on the attack roll is rolled again, once".into()
        }
        OffenceEffect::StartledBlow => "when something beside you attacks you unawares and you \
                                       make the initiative check, you strike it at once with \
                                       what you wield, before its attack lands (a free blow, \
                                       once per surprise; it brings no free blows of its own)"
            .to_string(),
        OffenceEffect::FreeBlow {
            trigger: warlock_core::content::FreeBlowTrigger::Gatekeeper,
            one_in: Some(n),
        } => format!(
            "free blow: a hostile that steps up beside you takes one, one time in {n} in the \
             neutral stance, more often the bolder your stance (the stances table)"
        ),
        OffenceEffect::FreeBlow { trigger, .. } => {
            use warlock_core::content::FreeBlowTrigger as T;
            match trigger {
                T::Cleave => {
                    "free blow: a melee blow that kills gives one at another hostile beside you \
                     (Cleave and Cleaving strike together: on and on, as Great cleave)"
                }
                T::GreatCleave => {
                    "free blow: cleaving goes on while the blows kill (once at each hostile beside \
                     you a turn)"
                }
                T::Riposte => "free blow: a blow that misses you by your Parry gives one back",
                T::Opportunist => "free blow: a hostile that steps away from beside you takes one",
                T::Gatekeeper => "free blow: a hostile that steps up beside you takes one",
                T::Flurry => {
                    "free blow: a melee attack with nothing in hand strikes twice, the second blow \
                     at \u{2212}2 to hit"
                }
            }
            .to_string()
        }
        OffenceEffect::Weapon {
            role,
            skill,
            to_hit,
            damage,
            threat,
            ranks,
            speed,
            reload,
            ease,
            bash,
            shield,
            crit,
            at_rank,
        } => {
            let any_style = skill.is_none() && role == Some(Role::Style);
            let with = match (role, skill) {
                (_, Some(id)) => (content.weapon_skills().id(&id)).map_or(id.clone(), |w| {
                    content.weapon_skills().skill(w).name.clone()
                }),
                (None, None) => "the chosen skill".to_string(),
                (Some(Role::Melee), _) => "every melee weapon skill".to_string(),
                (Some(Role::Launcher), _) => "every missile weapon skill".to_string(),
                (Some(Role::Thrown), _) => "throws".to_string(),
                (Some(Role::Unarmed), _) => "unarmed fighting".to_string(),
                (Some(Role::ShieldBash), _) => "shield bashing".to_string(),
                (Some(Role::Style), _) => "every fighting style".to_string(),
            };
            let mut gains = Vec::new();
            for (value, what) in [(to_hit, "to hit"), (damage, "damage"), (shield, "Shield")] {
                if value != 0 {
                    gains.push(format!("{what} {}", signed(value)));
                }
            }
            // A style's Shield counts only in the shield styles' grips.
            if shield != 0 && any_style {
                let last = gains.last_mut().expect("the Shield gain");
                last.push_str(" (a shield held with a weapon, or two shields)");
            }
            if threat > 0 {
                gains.push(format!("threat range +{threat}"));
            }
            if crit > 0 {
                gains.push(format!("critical multiplier +{crit} (at most \u{d7}5)"));
            }
            if ranks > 0 {
                gains.push(format!("benefits as {ranks} rank higher (at most 10)"));
            }
            if speed > 0 {
                gains.push(format!("attacks {speed} % cheaper"));
            }
            if reload > 0 {
                gains.push(format!("reloading {reload} % cheaper"));
            }
            if ease > 0 {
                gains.push(format!("both two-weapon penalties eased by {ease}"));
            }
            if bash > 0 {
                gains.push(format!(
                    "a free shield bash's chance +{bash} % where there is one"
                ));
            }
            let rank = if at_rank > 0 {
                format!(" at rank {at_rank} or more")
            } else {
                String::new()
            };
            format!("{} with {with}{rank}", gains.join(", "))
        }
        OffenceEffect::WeaponMarks { role, value } => {
            let role = match role {
                None => {
                    return format!(
                        "[training marks](weapon-skills.md#raising-a-weapon-skill) in every weapon skill and fighting style {}",
                        percent(value)
                    );
                }
                Some(Role::Launcher) => "missile".to_string(),
                Some(r) => r.id().replace('_', " "),
            };
            format!(
                "[training marks](weapon-skills.md#raising-a-weapon-skill) in every {role} weapon skill {}",
                percent(value)
            )
        }
        OffenceEffect::TwoHandedCrit { value } => {
            format!("critical multiplier +{value} with a two-handed weapon (at most \u{d7}5)")
        }
        OffenceEffect::UnarmedDie { sides } => {
            format!("blows with nothing in hand and kicks roll 1d{sides}")
        }
        OffenceEffect::UnarmedOnHit {
            status: id,
            chance,
            severity,
        } => format!(
            "a blow with nothing in hand that hits (not a kick) has a {chance} % chance to cause \
             {severity} {} of {}",
            if severity == 1 { "degree" } else { "degrees" },
            status(&id)
        ),
        OffenceEffect::LauncherRange { value, skill: None } => {
            format!("missile weapon range {} cells", signed(value))
        }
        OffenceEffect::LauncherRange {
            value,
            skill: Some(skill),
        } => {
            let table = content.weapon_skills();
            let name = (table.id(&skill)).map_or(skill.clone(), |s| table.skill(s).name.clone());
            format!("range {} cells with launchers of {name}", signed(value))
        }
        OffenceEffect::ThrowRange { value } => format!("throwing range {} cells", signed(value)),
        OffenceEffect::Streak { max, damage_every } => {
            let damage = if damage_every > 0 {
                format!(", and +1 damage for every {damage_every} blows before it")
            } else {
                String::new()
            };
            format!(
                "each melee blow at the creature your last blow was at (this turn or the one \
                 before) +1 to hit per blow before it, to +{max}{damage}; anything else starts \
                 it over"
            )
        }
        OffenceEffect::OffHandDamage { value } => {
            format!("off-hand blows {} damage", signed(value))
        }
        OffenceEffect::BestSkillThreat { value } => format!(
            "attacks with your best weapon skill (the highest rank you hold of the melee, \
             unarmed, launcher, and thrown skills; ties all count) threaten a critical {value} \
             lower"
        ),
        OffenceEffect::KickDie { sides } => format!("kicks at creatures roll 1d{sides}"),
        OffenceEffect::KickBack => "a kick that hits drives the foe one cell straight back, \
                                   when that cell is walkable, untrapped, unfrozen, and empty"
            .into(),
        OffenceEffect::TwoHandedPrice { value } => {
            format!("blows with a two-handed weapon cost {}", percent(value))
        }
        OffenceEffect::AttackPricePercent { value } => {
            format!("melee attacks cost {}", percent(value))
        }
        OffenceEffect::CriticalStatus { status: id } => format!(
            "a critical you confirm in melee leaves the target {} (one degree)",
            status(&id)
        ),
        OffenceEffect::Grip { status: id } => format!(
            "a creature you hit in melee is {} (it cannot step; beside you, it attacks you \
             instead) until you miss it, strike another, or move; no roll",
            status(&id)
        ),
        OffenceEffect::CriticalConfirm { value } => {
            format!("the roll confirming your criticals {}", signed(value))
        }
        OffenceEffect::AbilityRanks { value } => format!(
            "weapon abilities come {value} rank{} sooner (their thresholds read your ranks {} \
             higher)",
            if value == 1 { "" } else { "s" },
            signed(value)
        ),
        OffenceEffect::WeaponEnergyPercent { value } => format!(
            "weapon actives take {}% time (never below half the action's price)",
            signed(value)
        ),
        OffenceEffect::WeaponStamina { value, abilities } if abilities.is_empty() => {
            format!("every weapon active costs {value} stamina less (at least 1)")
        }
        OffenceEffect::WeaponStamina { value, abilities } => {
            let names: Vec<String> = (abilities.iter())
                .map(|a| {
                    (content.weapon_ability_id(a))
                        .map_or_else(|| a.clone(), |id| content.weapon_ability(id).name.clone())
                })
                .collect();
            format!(
                "{} cost {value} stamina less (at least 1)",
                names.join(" and ")
            )
        }
        OffenceEffect::Brace { to_hit, status: id } => format!(
            "Brace's blow {} to hit, and a foe it hits is {} (one degree)",
            signed(to_hit),
            status(&id)
        ),
        OffenceEffect::CriticalPenetration { value } => {
            format!("your confirmed criticals: penetration {}", signed(value))
        }
        OffenceEffect::Penetration { damage, value } => {
            let name = (content.damage_type_id(&damage)).map_or(damage.clone(), |d| {
                content.damage_type(d).name.to_lowercase()
            });
            format!(
                "blows that can do {name} damage: penetration {}",
                signed(value)
            )
        }
        OffenceEffect::StatusDamage { status: id, value } => {
            format!(
                "{} damage on hits and spells against a creature {}",
                signed(value),
                status(&id)
            )
        }
        OffenceEffect::FavouredPrey { to_hit, damage } => format!(
            "against the creature type you have killed most (ties: the first to get there), \
             to hit {} and damage {}",
            signed(to_hit),
            signed(damage)
        ),
        OffenceEffect::SwornFoe { to_hit, damage } => format!(
            "against creatures of the kind chosen (the creature tag; the tags and who carries \
             them: [Sorts of creatures](quests.md#sorts-of-creatures)), to hit {} and damage {}",
            signed(to_hit),
            signed(damage)
        ),
    }
}
