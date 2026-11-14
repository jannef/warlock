//! What the resource talents do, in the wiki's words (`warlock_core::talent::ResourceEffect`,
//! #1495): one arm a variant, beside the area's enum and sum in
//! `warlock-core/src/talent/resource.rs`.

use super::resource_name;
use crate::wiki::util::signed;
use warlock_core::content::{Content, OverTime};
use warlock_core::talent::ResourceEffect;

/// What `effect` does, as the talents' tables say it.
pub(super) fn text(content: &Content, effect: &ResourceEffect) -> String {
    let percent = |v: i32| format!("{} %", signed(v));
    match effect.clone() {
        ResourceEffect::ResourceBuild {
            resource: id,
            on,
            amount,
        } => format!(
            "{} builds {amount} more {} while you hold it",
            on.words(),
            resource_name(content, &id).to_lowercase()
        ),
        ResourceEffect::ResourceMax {
            resource: id,
            value,
        } => {
            format!(
                "{} holds {value} more at most",
                resource_name(content, &id).to_lowercase()
            )
        }
        ResourceEffect::ResourceStart {
            resource: id,
            value,
        } => format!(
            "{} starts {value} higher when you gain it",
            resource_name(content, &id).to_lowercase()
        ),
        ResourceEffect::ResourceRate {
            resource: id,
            value,
        } => {
            let r = content.resource_id(&id).map(|r| content.resource(r));
            let way = match r.and_then(|r| r.over_time) {
                Some(OverTime::Decays { .. }) => "decay",
                _ => "regeneration",
            };
            format!(
                "{} {way} pace {}",
                resource_name(content, &id).to_lowercase(),
                percent(value)
            )
        }
        ResourceEffect::GrantsResource { resource } => {
            format!(
                "you hold {} from now on (see Resources)",
                resource_name(content, &resource)
            )
        }
    }
}
