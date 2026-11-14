//! Traps (spec §8 "Traps").

use super::util::{Term, anchored, article, capital, percent_of, signed};
use std::fmt::Write as _;
use warlock_core::check;
use warlock_core::content::{Bolt, Burst, BurstIce, Content, RigTakes, TrapEffect, TrapNoise};
use warlock_core::skills::Effect;
use warlock_core::stats::modifier;
use warlock_core::time::Duration;
use warlock_core::world::SPRINGS_SHORT_BY;
use warlock_core::world::takes_words;

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
        let rigged = (content.rigs().into_iter())
            .flat_map(|r| r.rigs.iter())
            .any(|r| content.trap(r.trap).id == t.id);
        let goes_off = match t.charges {
            None if rigged => "as [rigged](#rigging-traps)".to_string(),
            None => "every time".to_string(),
            Some(1) => "once".to_string(),
            Some(2) => "twice".to_string(),
            Some(n) => format!("{n} times"),
        };
        let sound = sound_text(t.noise);
        let _ = writeln!(
            out,
            "| {} `{}` | {what} | {damage} | {goes_off} | {sound} |",
            anchored(Term::Trap, &t.id, &capital(&t.name)),
            t.glyph
        );
    }
    out
}

/// The rigs (spec §8 "Traps": "Rigging traps", `data/rigs.toml`): the craftsperson's
/// Trapwright, each rig with the trap it places, what it takes, how long it takes, how often the
/// trap goes off, and its damage; then the check.
pub(super) fn rigs(content: &Content) -> String {
    let Some(rules) = content.rigs() else {
        return "No traps can be rigged.\n".to_string();
    };
    let mut out = String::from(
        "| Rig | Trap | Takes | Time | Goes off | Damage | Needs |\n\
         |---|---|---|---|---|:---:|---|\n",
    );
    for (_, rig) in rules.iter() {
        let trap = content.trap(rig.trap);
        let goes_off = match &rig.takes {
            RigTakes::Ammo { .. } => "once for each arrow".to_string(),
            RigTakes::Stock { charges, .. } | RigTakes::DigTool { charges } => match charges {
                1 => "once".to_string(),
                2 => "twice".to_string(),
                n => format!("{n} times"),
            },
        };
        let damage = (trap.effect.damage()).map_or("—".to_string(), |d| {
            super::damage::typed_dice(content, d, trap.effect.damage_kind())
        });
        let needs = match rig.wall {
            Some(1) => "a wall next to the cell".to_string(),
            Some(n) => format!("a wall within {n} cells of the cell, in a straight line"),
            None => "—".to_string(),
        };
        let time = Duration::seconds(u64::from(rig.seconds)).spoken();
        let _ = writeln!(
            out,
            "| {} | [{}](#trap-{}) | {} | {time} | {goes_off} | {damage} | {needs} |",
            rig.name,
            capital(&trap.name),
            trap.id.replace('_', "-"),
            capital(&takes_words(content, rig)),
        );
    }
    let _ = writeln!(
        out,
        "\nThe check: 2d12 + your [Dexterity](stats.md) modifier + the crafting bonus of \
         [{}](skills.md#skill-{}) + your talents' Dexterity checks, against **{}**. A \
         natural 2 always fails, and springs the trap on you; a natural 24 always succeeds.",
        content.skill(rules.skill).name,
        content.skill(rules.skill).id.replace('_', "-"),
        rules.difficulty
    );
    out
}

/// Each damaging trap's damage by the danger of the level it lies on (spec §8 "Traps",
/// `[trap.grows]`): a column for each band between the rungs any trap has, from 0 (a place with
/// no danger) on, the trap's roll at that band's start.
pub(super) fn by_danger(content: &Content) -> String {
    let damaging: Vec<_> = (content.trap_defs().iter())
        .filter(|t| t.effect.damage().is_some())
        .collect();
    let mut starts: Vec<u8> = vec![0];
    for t in &damaging {
        starts.extend(
            t.grows
                .iter()
                .flat_map(|g| g.damage.iter().map(|(at, _)| *at)),
        );
    }
    starts.sort_unstable();
    starts.dedup();
    let bands: Vec<String> = (starts.iter().enumerate())
        .map(|(i, from)| match starts.get(i + 1) {
            Some(next) if *next == from + 1 => from.to_string(),
            Some(next) => format!("{from}–{}", next - 1),
            None => format!("{from}+"),
        })
        .collect();
    let mut out = format!(
        "| Trap | {} |\n|---|{}\n",
        bands.join(" | "),
        ":---:|".repeat(bands.len())
    );
    let mut rows = damaging;
    rows.sort_by(|a, b| a.name.cmp(&b.name));
    for t in rows {
        let cells: Vec<String> = (starts.iter())
            .map(|d| {
                let roll = t.damage_at(*d).expect("a damaging trap");
                super::damage::typed_dice(content, roll, t.effect.damage_kind())
            })
            .collect();
        let _ = writeln!(
            out,
            "| [{}](#trap-{}) | {} |",
            capital(&t.name),
            t.id.replace('_', "-"),
            cells.join(" | ")
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
                percent_of(made, 144),
                percent_of(set_off, 144)
            );
        }
        out.push('\n');
    }
    out
}

/// Stepping past a known trap (spec §8 "Traps", `[avoid]` in `data/traps.toml`): the chance by
/// dodge (the Dexterity modifier as armour and burden leave it, plus Dodge), from the floor to
/// the ceiling, with the Dexterity that gives each dodge alone and the Tumble that gives it at
/// Dexterity 10.
pub(super) fn avoiding(content: &Content) -> String {
    let avoid = content.trap_avoid();
    let lowest = (-30..=0)
        .rev()
        .find(|&d| avoid.chance(d) == u32::from(avoid.min));
    let highest = (0..=30).find(|&d| avoid.chance(d) == u32::from(avoid.max));
    let (lowest, highest) = (lowest.unwrap_or(-30), highest.unwrap_or(30));
    let tumble = (content.skills().iter())
        .flat_map(|s| s.benefits.iter())
        .find(|b| b.effect == Effect::Dodge);
    let mut out = String::from("| Dodge | Chance to avoid | For example |\n|:---:|:---:|---|\n");
    for dodge in lowest..=highest {
        let label = match dodge {
            d if d == lowest && d != highest => format!("{} or less", signed(d)),
            d if d == highest && d != lowest => format!("{} or more", signed(d)),
            d => signed(d),
        };
        // Dexterity 2d + 10 and 2d + 11 give modifier d.
        let dexterity = 10 + 2 * dodge;
        let mut example = match dodge {
            d if d == lowest => format!("Dexterity {} or less", dexterity + 1),
            d if d == highest => format!("Dexterity {dexterity} or more"),
            _ => format!("Dexterity {dexterity} or {}", dexterity + 1),
        };
        let skill =
            (tumble.filter(|_| dodge > 0)).and_then(|b| (0..=500u16).find(|&v| b.at(v) >= dodge));
        if let Some(value) = skill {
            let _ = write!(example, "; Dexterity 10 with Tumble {value}");
        }
        let _ = writeln!(out, "| {label} | {}% | {example} |", avoid.chance(dodge));
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

/// What a noise sounds like, in words and its loudness: "Explosion, very loud (150)"; "Silent"
/// for none.
pub(super) fn sound_text(noise: Option<TrapNoise>) -> String {
    noise.map_or("Silent".to_string(), |n| {
        let loud = match n.loudness {
            0..50 => "quiet",
            50..150 => "loud",
            150..250 => "very loud",
            _ => "deafening",
        };
        format!(
            "{}, {loud} ({})",
            capital(&n.sound.name().replace('_', " ")),
            n.loudness
        )
    })
}

/// A noun with its article: "an arrow".
fn article_noun(noun: &str) -> String {
    format!("{} {noun}", article(noun))
}
