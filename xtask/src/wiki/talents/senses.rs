//! What the senses talents do, in the wiki's words (`warlock_core::talent::SensesEffect`, #1495):
//! one arm a variant, beside the area's enum and sum in `warlock-core/src/talent/senses.rs`.

use crate::wiki::util::signed;
use warlock_core::content::Content;
use warlock_core::talent::SensesEffect;

/// What `effect` does, as the talents' tables say it.
pub(super) fn text(content: &Content, effect: &SensesEffect) -> String {
    let percent = |v: i32| format!("{} %", signed(v));
    let status = |id: &str| {
        (content.status_id(id)).map_or_else(|| id.to_string(), |s| content.status(s).name.clone())
    };
    match effect.clone() {
        SensesEffect::Hearing { value } => format!("hears sounds {value} fainter"),
        SensesEffect::ExactFootsteps => {
            "footsteps you hear are placed at the cell they came from, not roughly".to_string()
        }
        SensesEffect::ThresholdPercent { value } => {
            format!("the light your eyes need {}", percent(value))
        }
        SensesEffect::Darkvision { value } => format!("darkvision {} cells", signed(value)),
        SensesEffect::Scent { status: id, radius } => format!(
            "sees creatures that are {} in line of sight within {radius} cells, however dark",
            status(&id).to_lowercase()
        ),
        SensesEffect::NightSight { value } => {
            format!("overworld sight at night {} cells", signed(value))
        }
        SensesEffect::DaySight { value } => {
            format!("overworld sight by day {} cells", signed(value))
        }
        SensesEffect::InnerLight { light } => {
            let reach = |id: &str| (content.light_id(id)).map_or(0, |l| content.light(l).radius);
            format!(
                "you always shed a faint light reaching {} cell (everyone sees it)",
                reach(&light)
            )
        }
        SensesEffect::LanternLight { light } => {
            let reach = |id: &str| (content.light_id(id)).map_or(0, |l| content.light(l).radius);
            format!(
                "your lantern burns brighter: its light reaches {} cells",
                reach(&light)
            )
        }
        SensesEffect::ExactSounds => {
            "every sound you hear is placed at the cell it came from, not roughly".to_string()
        }
    }
}
