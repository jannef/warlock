//! Traps (spec §8 "Traps").

use super::combat::percent;
use super::text::{article, capital};
use std::fmt::Write as _;
use warlock_core::check;
use warlock_core::content::{Bolt, Burst, BurstIce, Content, TrapEffect, TrapNoise};
use warlock_core::skills::Effect;
use warlock_core::stats::modifier;
use warlock_core::world::SPRINGS_SHORT_BY;

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
            TrapEffect::Burst(Burst { radius, ice, .. }) => format!(
                "A blast {radius} cells around it (walls stop it): everyone in it takes the \
                 damage, half at the edge; no roll to hit{}",
                ice_text(*ice)
            ),
            TrapEffect::Bolt(Bolt { length, .. }) => format!(
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
        let damage = (t.effect.damage()).map_or("—".to_string(), |d| {
            super::damage::typed_dice(content, d, t.effect.damage_kind())
        });
        let goes_off = match t.charges {
            None => "every time".to_string(),
            Some(1) => "once".to_string(),
            Some(2) => "twice".to_string(),
            Some(n) => format!("{n} times"),
        };
        let sound = sound_text(t.noise);
        let _ = writeln!(
            out,
            "| {} `{}` | {what} | {damage} | {goes_off} | {sound} |",
            capital(&t.name),
            t.glyph
        );
    }
    out
}

/// Disarming each trap (spec §8 "Disarming traps"), by name: its difficulty (the one finding it
/// takes, before a dungeon's depth adds to it) and how often a try disarms it and how often it
/// sets it off, at Dexterity 10 untrained, at Dexterity 14, and at Dexterity 14 with Disable
/// device at 100.
pub(super) fn disarming(content: &Content) -> String {
    let skilled = (content.skills().iter())
        .flat_map(|s| s.benefits.iter())
        .find(|b| b.effect == Effect::Disarm)
        .map_or(0, |b| b.at(100));
    let modifiers = [
        modifier(10),
        modifier(14),
        modifier(14).saturating_add(skilled),
    ];
    let mut out = String::from(
        "| Trap | Difficulty | Dexterity 10 | Dexterity 14 | Dexterity 14, Disable device 100 \
         |\n|---|:---:|:---:|:---:|:---:|\n",
    );
    let mut traps: Vec<_> = content.trap_defs().iter().collect();
    traps.sort_by(|a, b| a.name.cmp(&b.name));
    for t in traps {
        let difficulty = i32::from(t.difficulty);
        let _ = write!(out, "| {} | {difficulty} |", capital(&t.name));
        for m in modifiers {
            let made = check::ways(m, difficulty);
            let set_off = check::ways_short_by(m, difficulty, SPRINGS_SHORT_BY);
            let _ = write!(
                out,
                " {} disarmed, {} set off |",
                percent(made, 144),
                percent(set_off, 144)
            );
        }
        out.push('\n');
    }
    out
}

/// What a burst does to ice and water, after what it does to whom (a trap's, a scroll's).
pub(super) fn ice_text(ice: Option<BurstIce>) -> String {
    match ice {
        Some(BurstIce::Melts) => {
            "; it melts the ice it reaches, and whoever stands on it falls in".to_string()
        }
        Some(BurstIce::Freezes(um)) => format!(
            "; it freezes the water it reaches into ice {} cm thick",
            um / 10_000
        ),
        None => String::new(),
    }
}

/// What a noise sounds like, in words: "Explosion, very loud"; "Silent" for none.
pub(super) fn sound_text(noise: Option<TrapNoise>) -> String {
    noise.map_or("Silent".to_string(), |n| {
        let loud = match n.loudness {
            0..50 => "quiet",
            50..150 => "loud",
            150..250 => "very loud",
            _ => "deafening",
        };
        format!("{}, {loud}", capital(&n.sound.name().replace('_', " ")))
    })
}

/// A noun with its article: "an arrow".
fn article_noun(noun: &str) -> String {
    format!("{} {noun}", article(noun))
}
