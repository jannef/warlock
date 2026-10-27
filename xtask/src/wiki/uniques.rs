//! Unique items (spec §8 "Magic items": "Unique items"): each with its base, item level, affixes
//! exactly, power exactly, and lore.

use super::order;
use super::text::capital;
use std::fmt::Write as _;
use warlock_core::content::{Affliction, Content, ItemKind, Power, UniqueDef};
use warlock_core::magic::describe;

/// Every unique item of the campaigns the wiki describes, by name.
pub(super) fn uniques(content: &Content) -> String {
    let mut all: Vec<&UniqueDef> = (content.uniques())
        .filter(|u| order::described(content, u.campaign.as_deref()))
        .collect();
    all.sort_by_cached_key(|u| u.name.to_lowercase());
    let mut out = String::from(
        "| Item | Base | Item level | Affixes (tier: what it does) | Power | Carried by | Lore |\n\
         |---|---|:---:|---|---|---|---|\n",
    );
    for u in all {
        let def = content.object(u.object);
        let magic = u.magic();
        let kind = def.kind.unwrap_or(ItemKind::Ring);
        let affixes: Vec<String> = (magic.rolled(content).iter())
            .map(|r| {
                let does = describe::phrase(r, kind, content);
                format!("{} ({}: {does}, value {})", r.def.name, r.tier, r.value)
            })
            .collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} |",
            u.name,
            capital(&def.name),
            magic.level,
            affixes.join("; "),
            power_text(content, u),
            carried_by(content, u),
            u.lore.as_deref().unwrap_or("-"),
        );
    }
    out
}

/// The creature types whose gear always or sometimes gives them unique `u` ("Black Wystan"), or
/// "-" when it lies elsewhere (a level's pile or legend).
fn carried_by(content: &Content, u: &UniqueDef) -> String {
    let who: Vec<&str> = (content.creature_defs().iter())
        .filter(|c| order::described(content, c.campaign.as_deref()))
        .filter(|c| {
            (c.gear.lines.iter())
                .flat_map(|line| line.one_of.iter())
                .flat_map(|(stacks, _)| stacks.iter())
                .any(|s| *s == u.stack)
        })
        .map(|c| c.name.as_str())
        .collect();
    if who.is_empty() {
        "-".into()
    } else {
        who.join(", ")
    }
}

/// What a unique's power does, exactly.
fn power_text(content: &Content, u: &UniqueDef) -> String {
    match &u.power {
        None => "-".into(),
        Some(Power::Inflict(hit)) => {
            let Affliction::Status { status, severity } = hit.what else {
                return "-".into();
            };
            let name = &content.status(status).name;
            let degrees = if severity == 1 { "degree" } else { "degrees" };
            let when = if hit.chance >= 100 {
                "every hit that leaves".to_string()
            } else {
                format!("{}% of the hits that leave", hit.chance)
            };
            let resisted = (content.status(status).damage_type).map_or(String::new(), |k| {
                let kind = &content.damage_type(k).name;
                format!(" (less for what resists {kind}, none for what is immune)")
            });
            format!("{when} its target alive: {severity} {degrees} of {name}{resisted}")
        }
        Some(Power::Vision(index)) => {
            let eye = &content.vision_profiles()[*index];
            format!(
                "its wearer sees with {} where it is better than their own eyes (each part \
                 counts: the dimmer light seen, the farther darkvision, ultraviolet)",
                eye.name
            )
        }
        Some(Power::Ward { status, .. }) => {
            format!("its wearer is never {}", content.status(*status).name)
        }
    }
}
