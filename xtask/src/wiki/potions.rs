//! Potions (spec §8 "Potions").

use super::text::{Term, anchored, capital, damage_link, link};
use std::fmt::Write as _;
use warlock_core::combat::IMPROVISED_RANGE;
use warlock_core::content::{Content, Draught, ObjectDef, SpellPurpose, StatusId};

/// The potions: what drinking each does in numbers, its colour, the training marks it gives,
/// what throwing it does (a flask's splash, spec §8 "Potions": "Flasks"), from which item level
/// (depth) loot holds it and how often, and what it is worth. By name.
pub(super) fn potions(content: &Content) -> String {
    let mut out = String::from(
        "| Potion | Kind | What drinking it does | Thrown | Colour | Marks | Found from | Value |\n\
         |---|---|---|---|---|---|---|---:|\n",
    );
    let mut potions: Vec<_> = (content.object_defs().iter())
        .filter_map(|o| o.drink.as_ref().map(|d| (o, d)))
        .collect();
    potions.sort_by_key(|(o, _)| o.name.to_lowercase());
    for (o, drink) in potions {
        // What its cures end, by the statuses' names.
        let cured: Vec<String> = (content.status_ids())
            .filter(|(_, s)| drink.does.cures().iter().any(|c| s.cured_by.contains(c)))
            .map(|(_, s)| s.name.clone())
            .collect();
        let also = match cured.is_empty() {
            true => String::new(),
            false => format!("; ends {}", cured.join(" and ")),
        };
        let what = match &drink.does {
            Draught::Heal { dice, .. } => {
                format!("{dice} hit points back at once, never past your maximum{also}")
            }
            Draught::Cure { diseases, .. } => {
                let diseases = match (diseases, cured.is_empty()) {
                    (true, true) => "ends every disease".to_string(),
                    (true, false) => format!("ends every disease and {}", cured.join(" and ")),
                    (false, _) => format!("ends {}", cured.join(" and ")),
                };
                format!("{} at once", capital(&diseases))
            }
            Draught::Mana { dice } => {
                format!("{dice} mana back at once, never past your maximum")
            }
            Draught::Elixir {
                stat,
                points,
                potential,
            } => format!(
                "{} +{points} for good (a trained rise: the potential lifted to the score if \
                 below), then its potential +{} (a roll)",
                stat.name(),
                potential.to_string().trim_start_matches('+')
            ),
            Draught::Harm { dice, status } => format!(
                "{} degrees of {} (as rotten food)",
                dice.to_string().trim_start_matches('+'),
                content.status(*status).name
            ),
            Draught::Boost { status, severity } => {
                let def = content.status(*status);
                let raised: Vec<String> = (def.penalty.iter())
                    .flat_map(|p| p.by.iter().filter(|&(_, v)| v > 0))
                    .map(|(stat, v)| {
                        let by = i32::from(v) * def.penalty_times(*severity);
                        format!("+{by} {}", stat.name())
                    })
                    .collect();
                format!(
                    "{} ({}, {severity} degree{})",
                    raised.join(", "),
                    def.name,
                    plural(*severity)
                ) + &lasting(content, *status, *severity)
            }
            Draught::Vision { status } => {
                let def = content.status(*status);
                let eyes = def
                    .vision
                    .map(|i| content.vision_profiles()[i].name.as_str());
                let eyes = eyes.map_or_else(
                    || {
                        if def.sees_unseen {
                            "Seeing".into()
                        } else {
                            "other eyes".into()
                        }
                    },
                    |e| format!("{e} eyes"),
                );
                format!("{eyes} ({})", def.name) + &lasting(content, *status, 1)
            }
            Draught::Haste { status, severity } => {
                let def = content.status(*status);
                format!(
                    "{}, {severity} degree{}: every action takes {} % of its time",
                    def.name,
                    plural(*severity),
                    def.cost
                ) + &lasting(content, *status, *severity)
            }
            Draught::Resistance { status } => {
                let def = content.status(*status);
                let kinds: Vec<String> = (def.resist.iter())
                    .map(|&(kind, r)| format!("{r} % {}", content.damage_type(kind).name))
                    .collect();
                format!("resists {} ({}, as gear does)", kinds.join(", "), def.name)
                    + &lasting(content, *status, 1)
            }
            Draught::Status { status, severity } => {
                let def = content.status(*status);
                format!(
                    "{}, {severity} degree{}",
                    link(Term::Status, &def.id, &def.name),
                    plural(*severity)
                ) + &lasting(content, *status, *severity)
            }
        };
        let marks = match drink.marks.as_slice() {
            [] => "none".to_string(),
            marks => (marks.iter())
                .map(|(stat, n)| format!("{n} {}", stat.name()))
                .collect::<Vec<_>>()
                .join(", "),
        };
        let found = match o.frequency {
            0 => "not as loot".to_string(),
            f => format!("item level {}, frequency {f}", o.level),
        };
        let _ = writeln!(
            out,
            "| {} `{}` | {} | {what} | {} | {} (`{}`) | {marks} | {found} | {} gold |",
            anchored(Term::Potion, &o.id, &capital(&o.name)),
            o.glyph,
            drink.does.kind(),
            thrown(content, o),
            drink.colour,
            o.fg,
            o.value
        );
    }
    out
}

fn plural(n: u16) -> &'static str {
    if n == 1 { "" } else { "s" }
}

/// How long a status given at `severity` lasts, by its `eases` a degree: "; lasts 10 minutes".
fn lasting(content: &Content, status: StatusId, severity: u16) -> String {
    let Some(every) = content.status(status).eases else {
        return String::new();
    };
    let seconds = u64::from(every) * u64::from(severity);
    let (n, unit) = match seconds % 60 {
        0 => (seconds / 60, "minute"),
        _ => (seconds, "second"),
    };
    let s = if n == 1 { "" } else { "s" };
    let per = match severity {
        1 => String::new(),
        _ => format!(", a degree lost every {every} seconds"),
    };
    format!("; lasts {n} {unit}{s}{per}")
}

/// What throwing potion `o` does: a flask's splash exactly (its spell at its rung, linked, and
/// the numbers it casts with), else a spill. Both with the range before talents.
fn thrown(content: &Content, o: &ObjectDef) -> String {
    let Some(flask) = &o.flask else {
        return format!("spills uselessly; range {IMPROVISED_RANGE}");
    };
    let spell = content.spell(flask.spell);
    let splash = flask.splash(content);
    let mut does = Vec::new();
    if let Some(radius) = splash.radius {
        does.push(format!("radius {radius}"));
    }
    if let (Some(roll), SpellPurpose::Damage, Some(kind)) =
        (splash.roll, spell.kind, spell.damage_type)
    {
        let kind = damage_link(content, kind);
        does.push(format!(
            "{} {kind}",
            roll.to_string().trim_start_matches('+')
        ));
    }
    if let (Some((status, _)), Some(degrees)) = (spell.effect.status, splash.degrees) {
        let def = content.status(status);
        let name = link(Term::Status, &def.id, &def.name);
        does.push(format!("{name} {degrees}"));
    }
    if let Some(zone) = spell.effect.zone {
        does.push(format!(
            "a {} zone for {} turns",
            zone.kind.id(),
            zone.turns
        ));
    }
    format!(
        "shatters into {} (rung {}): {}; range {}",
        link(Term::Spell, &spell.id, &spell.name),
        flask.rungs,
        does.join(", "),
        flask.range
    )
}
