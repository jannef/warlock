//! What the vigour talents do, in the wiki's words (`warlock_core::talent::VigourEffect`, #1495):
//! one arm a variant, beside the area's enum and sum in `warlock-core/src/talent/vigour.rs`.

use crate::wiki::util::signed;
use warlock_core::content::Content;
use warlock_core::talent::VigourEffect;

/// What `effect` does, as the talents' tables say it.
pub(super) fn text(content: &Content, effect: &VigourEffect) -> String {
    let percent = |v: i32| format!("{} %", signed(v));
    let status = |id: &str| {
        (content.status_id(id)).map_or_else(|| id.to_string(), |s| content.status(s).name.clone())
    };
    match effect.clone() {
        VigourEffect::MaxHpPerLevels {
            per_levels,
            at_least,
        } => format!("+1 maximum hit point per {per_levels} levels, at least +{at_least}"),
        VigourEffect::MaxHpPercent { value } => {
            format!(
                "maximum hit points {} (rounded down, at least 1)",
                percent(value)
            )
        }
        VigourEffect::Stat { stat, value } => {
            format!("{} {} (score and potential)", stat.name(), signed(value))
        }
        VigourEffect::HealingPercent { value } => format!("healing pace {}", percent(value)),
        VigourEffect::DiseaseImmune => "you never catch a disease".to_string(),
        VigourEffect::NoPenaltyTo { stat } => {
            format!("no status lowers your {}", stat.name())
        }
        VigourEffect::RaisePercent { status: id, value } => {
            format!(
                "what {} raises your scores by {}",
                status(&id),
                percent(value)
            )
        }
        VigourEffect::Hardened { bonus } => format!(
            "every level past the last that adds hit points adds them again, and {bonus} more"
        ),
        VigourEffect::WoundMore { value } => {
            format!("a big hit that wounds you wounds {value} degree more")
        }
        VigourEffect::RestingPercent { value } => {
            format!("healing pace while resting {}", percent(value))
        }
        VigourEffect::RiseFromAshes {
            hp_percent,
            status: id,
            radius,
            damage,
            damage_type,
            ..
        } => format!(
            "once in your life, a hit, a trap, or a status that would kill you does not (starving \
             and choking still do): you rise with {hp_percent} % of your maximum hit points \
             (Ashen ones), {damage} {damage_type} damage strikes every creature within {radius} \
             cell (one roll; no roll to hit) and the ice there melts, and you are {}",
            status(&id)
        ),
        VigourEffect::RebornWhole => "rising from the ashes brings you back whole: every hit \
             point, every status effect and disease gone (nothing follows them), and no Ashen"
            .to_string(),
        VigourEffect::SecondRising => {
            "you may rise from the ashes a second time in your life".to_string()
        }
        VigourEffect::StarvingHeals => {
            "hit points come back while you are Starving too (at your usual pace)".to_string()
        }
        VigourEffect::SecondWind {
            per_level,
            at_least,
            cooldown,
        } => format!(
            "active (the time of a wait): {per_level} hit point per level back at once, at \
             least {at_least}; then {} (no second wind while it lasts)",
            status(&cooldown)
        ),
        VigourEffect::StatusEases { status: id, value } => {
            format!("{} eases {value} % faster", status(&id))
        }
        VigourEffect::StatusLasts { status: id, value } => {
            format!("{} lasts {value} % longer", status(&id))
        }
        VigourEffect::NoPenalty { status: id } => {
            format!("{} never lowers your scores", status(&id))
        }
        VigourEffect::Clotting => {
            "a big hit wounds you no more; a confirmed critical one degree less".to_string()
        }
        VigourEffect::PenaltyPercent { value } => format!(
            "what statuses take off your scores {} (each, rounded toward 0)",
            percent(value)
        ),
        VigourEffect::DiseaseChecks { value } => {
            format!("disease checks (catching, progressions) {}", signed(value))
        }
        VigourEffect::DrainChecks { value } => {
            format!("checks against a drain {}", signed(value))
        }
        VigourEffect::Checks { stat, value } => {
            format!("every {} check {}", stat.name(), signed(value))
        }
        VigourEffect::DiseaseIntervalPercent { value } => format!(
            "a disease's progressions come {} further apart",
            percent(value)
        ),
        VigourEffect::DiseaseRecovery { value } => format!(
            "a disease goes after {value} fewer checks won in a row (at least 1; never \
             one that does not recover)"
        ),
        VigourEffect::Reprieve {
            status: id,
            stat,
            by,
            ..
        } => {
            let stat = stat.map_or_else(String::new, |s| {
                format!(", and {} {} for good", s.name(), signed(by))
            });
            format!(
                "once in a life, what would kill you (not starving or choking) leaves you at 1 hit \
                 point, {}{stat}",
                status(&id)
            )
        }
    }
}
