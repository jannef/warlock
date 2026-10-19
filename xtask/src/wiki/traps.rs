//! Traps (spec §8 "Traps").

use super::text::{article, capital};
use std::fmt::Write as _;
use warlock_core::content::{BurstIce, Content, TrapEffect};

/// The traps (spec §8 "Traps"): what each does, its damage dice, how often it goes off, and what
/// it sounds like. Never a to-hit, which the player does not know.
pub(super) fn traps(content: &Content) -> String {
    let mut out = String::from(
        "| Trap | What it does | Damage | Goes off | Sound |\n|---|---|:---:|---|---|\n",
    );
    for t in content.trap_defs() {
        let what = match &t.effect {
            TrapEffect::Shot { object, .. } => format!(
                "{} shoots out of a hidden slit in the nearest wall at whoever steps on it: an \
                 attack against their Defence; it lands where it falls",
                capital(&article_noun(&content.object(*object).name))
            ),
            TrapEffect::Strike { .. } => format!(
                "{}s spring from the floor: an attack against the Defence of whoever steps on it",
                capital(&t.noun)
            ),
            TrapEffect::Burst { radius, ice, .. } => {
                let ice = match ice {
                    Some(BurstIce::Melts) => {
                        "; it melts the ice it reaches, and whoever stands on it falls in"
                            .to_string()
                    }
                    Some(BurstIce::Freezes(um)) => format!(
                        "; it freezes the water it reaches into ice {} cm thick",
                        um / 10_000
                    ),
                    None => String::new(),
                };
                format!(
                    "A blast {radius} cells around it (walls stop it): everyone in it takes the \
                     damage, half at the edge; no roll to hit{ice}"
                )
            }
            TrapEffect::Bolt { length, .. } => format!(
                "A bolt from it along a line until a wall, up to {length} cells: everyone on it \
                 takes the damage; no roll to hit"
            ),
            TrapEffect::Pit { .. } => {
                "Whoever steps on it falls in and loses their next action climbing out".into()
            }
            TrapEffect::Alarm => "Only its noise: creatures that hear it come to look".into(),
            TrapEffect::Teleport => {
                "Whoever steps on it is moved to a free place on the level".into()
            }
        };
        let damage = t.effect.damage().map_or("—".to_string(), |d| d.to_string());
        let goes_off = match t.charges {
            None => "every time".to_string(),
            Some(1) => "once".to_string(),
            Some(2) => "twice".to_string(),
            Some(n) => format!("{n} times"),
        };
        let sound = t.noise.map_or("Silent".to_string(), |n| {
            let loud = match n.loudness {
                0..50 => "quiet",
                50..150 => "loud",
                150..250 => "very loud",
                _ => "deafening",
            };
            format!("{}, {loud}", capital(&n.sound.name().replace('_', " ")))
        });
        let _ = writeln!(
            out,
            "| {} `{}` | {what} | {damage} | {goes_off} | {sound} |",
            capital(&t.name),
            t.glyph
        );
    }
    out
}

/// A noun with its article: "an arrow".
fn article_noun(noun: &str) -> String {
    format!("{} {noun}", article(noun))
}
