//! What the growth talents do, in the wiki's words (`warlock_core::talent::GrowthEffect`, #1495):
//! one arm a variant, beside the area's enum and sum in `warlock-core/src/talent/growth.rs`.

use crate::wiki::util::signed;
use warlock_core::content::Content;
use warlock_core::talent::GrowthEffect;

/// What `effect` does, as the talents' tables say it.
pub(super) fn text(content: &Content, effect: &GrowthEffect) -> String {
    let percent = |v: i32| format!("{} %", signed(v));
    match effect.clone() {
        GrowthEffect::XpPercent { value } => format!("experience gained {}", percent(value)),
        GrowthEffect::SkillRolls { value } => format!("skill rolls every level {}", signed(value)),
        GrowthEffect::ExtraPick { level: 1 } => "one more talent at creation".to_string(),
        GrowthEffect::ExtraPick { level } => format!("one more talent at level {level}"),
        GrowthEffect::WeaponRanks { to, skills, .. } if skills.is_empty() => format!(
            "every [weapon skill](weapon-skills.md) and [fighting style](fighting-styles.md) \
             (not a hidden style) below rank {to} rises to rank {to}, \
             once, as it is given"
        ),
        GrowthEffect::WeaponRanks { to, skills, .. } => {
            let table = content.weapon_skills();
            let names: Vec<String> = (skills.iter())
                .filter_map(|s| table.id(s).map(|id| table.skill(id).name.clone()))
                .collect();
            format!(
                "{} below rank {to} rise to rank {to}, once, as it is given",
                names.join(", ")
            )
        }
        GrowthEffect::TakeUpSkills { dice, except, .. } => {
            let mut text = format!(
                "every [skill](skills.md) you are not trained in, whatever skills there are, \
                 trained at {dice} (rolled once, as it is given)"
            );
            let names: Vec<String> = (except.iter())
                .filter_map(|s| content.skill_id(s).map(|id| content.skill(id).name.clone()))
                .collect();
            if !names.is_empty() {
                text.push_str(&format!(", but {}", names.join(" and ")));
            }
            text
        }
        GrowthEffect::Skill { skill: None, value } => {
            format!("the chosen [skill](skills.md) counts {}", signed(value))
        }
        GrowthEffect::Skill {
            skill: Some(skill),
            value,
        } => {
            let name =
                (content.skill_id(&skill)).map_or(skill.clone(), |s| content.skill(s).name.clone());
            format!("{name} counts {}", signed(value))
        }
        GrowthEffect::TrainedSkills { value } => {
            format!("every trained skill counts {}", signed(value))
        }
    }
}
