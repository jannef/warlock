//! Skills (spec §8 "Skills"): what each does and trains on, what it gives by value, who trains
//! it, and the provisional increase curve.

use super::burden::factor;
use std::fmt::Write as _;
use warlock_core::burden::Burden;
use warlock_core::content::Content;
use warlock_core::dice::Dice;
use warlock_core::skills::{Effect, SkillId};

/// Every skill in the order the Skills screen lists them: what kind it is, what it is about,
/// what it gives (the benefits table has the numbers), and when it earns a training mark (with
/// the chance).
pub(super) fn skills(content: &Content) -> String {
    let mut out = String::from(
        "| Skill | Kind | What it is about | Gives | Trains on |\n|---|---|---|---|---|\n",
    );
    for skill in content.skills() {
        let gives: Vec<&str> = skill
            .benefits
            .iter()
            .map(|b| effect_name(b.effect))
            .collect();
        let gives = if gives.is_empty() {
            "nothing yet".to_string()
        } else {
            gives.join("; ")
        };
        let trains: Vec<String> = (skill.marks.iter())
            .map(|m| format!("{} ({}%)", m.on.describe(), m.chance))
            .collect();
        let trains = if trains.is_empty() {
            "nothing yet".to_string()
        } else {
            trains.join("; ")
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {gives} | {trains} |",
            skill.name,
            skill.kind.name(),
            skill.description
        );
    }
    out
}

/// What an effect does, in words.
fn effect_name(effect: Effect) -> &'static str {
    match effect {
        Effect::Dodge => "a Dodge bonus to Defence",
        Effect::Quiet => "quieter steps and doors",
        Effect::Hearing => "keener hearing",
        Effect::Search => "finding hidden things when you search",
        Effect::SearchReach => "searching farther around you",
        Effect::Spot => "noticing hidden things in passing",
        Effect::Disarm => "disarming traps",
        Effect::Burden => "less slowdown from your load",
        Effect::Travel => "quicker journeys on the overworld",
        Effect::Bargain => "better prices in shops",
        Effect::Kick => "more Strength for kicking doors in",
        Effect::Pick => "picking locks (trained at all: you can try)",
    }
}

/// The values the benefits table shows.
const BENEFIT_VALUES: [u16; 14] = [0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 125, 150, 200];

/// What each skill's benefits give at a range of values, one column per benefit, from the data.
pub(super) fn skill_benefits(content: &Content) -> String {
    let columns: Vec<(&str, &warlock_core::skills::Benefit)> = (content.skills().iter())
        .flat_map(|s| s.benefits.iter().map(move |b| (s.name.as_str(), b)))
        .collect();
    let mut out = String::from("| Value |");
    for (name, b) in &columns {
        let what = match b.effect {
            Effect::Dodge => "Dodge",
            Effect::Quiet => "noise",
            Effect::Hearing => "hears fainter by",
            Effect::Search => "searching",
            Effect::SearchReach => "search reaches",
            Effect::Spot => "noticing",
            Effect::Disarm => "disarming traps",
            Effect::Burden => "Burdened / Strained",
            Effect::Travel => "journeys",
            Effect::Bargain => "prices",
            Effect::Kick => "kicking doors in",
            Effect::Pick => "picking locks",
        };
        let _ = write!(out, " {name}: {what} |");
    }
    out.push_str("\n|---:|");
    out.push_str(&":---:|".repeat(columns.len()));
    out.push('\n');
    for value in BENEFIT_VALUES {
        let _ = write!(out, "| {value} |");
        for (_, b) in &columns {
            let n = b.at(value);
            let cell = match b.effect {
                _ if n == 0 => "—".to_string(),
                Effect::Dodge | Effect::Search | Effect::Spot | Effect::Disarm | Effect::Pick => {
                    format!("+{n}")
                }
                Effect::SearchReach => format!("+{n} cell{}", if n == 1 { "" } else { "s" }),
                Effect::Kick => format!("+{n} Strength"),
                Effect::Quiet | Effect::Travel => format!("\u{2212}{n}%"),
                Effect::Bargain => format!("{n}% better"),
                Effect::Hearing => n.to_string(),
                Effect::Burden => {
                    let slowed = |b: Burden| factor(b.slow_eased(100, n));
                    format!(
                        "×{} / ×{}",
                        slowed(Burden::Burdened),
                        slowed(Burden::Strained)
                    )
                }
            };
            let _ = write!(out, " {cell} |");
        }
        out.push('\n');
    }
    out
}

/// A skill's name, for the grants tables.
fn skill_name(content: &Content, id: SkillId) -> &str {
    &content.skill(id).name
}

/// What each race and each class trains, as dice (`1d6+4`), in the order New run offers them.
pub(super) fn skill_grants(content: &Content) -> String {
    let list = |grants: &[(SkillId, Dice)]| -> String {
        let parts: Vec<String> = (grants.iter())
            .map(|&(id, dice)| format!("{} `{dice}`", skill_name(content, id)))
            .collect();
        parts.join(", ")
    };
    let mut out = String::from("| Race | Trains |\n|---|---|\n");
    for r in &content.races().list {
        let mut trains = list(&r.skills);
        if !r.class_skill.is_zero() {
            if !trains.is_empty() {
                trains.push_str("; ");
            }
            let _ = write!(
                trains,
                "one of the class's skills `+{}`, picked at random",
                r.class_skill
            );
        }
        if trains.is_empty() {
            trains = "-".into();
        }
        let _ = writeln!(out, "| {} | {trains} |", r.name);
    }
    out.push_str("\n| Class | Trains |\n|---|---|\n");
    for c in content.classes() {
        let _ = writeln!(out, "| {} | {} |", c.name, list(&c.skills));
    }
    out
}

/// The skill ladder: the die a level-up roll throws by the skill's marks, and the die a skill the
/// class is known for throws (one step up).
pub(super) fn skill_ladder(content: &Content) -> String {
    let ladder = content.ladder();
    let mut out = String::from(
        "| Marks | Die | Average | Known for: die | Known for: average |\n|---:|---|---:|---|---:|\n",
    );
    let average = |d: Dice| format!("{:.1}", f64::from(d.min() + d.max()) / 2.0);
    for (step, (&marks, &die)) in ladder.marks.iter().zip(&ladder.dice).enumerate() {
        let up = ladder.die(step + 1);
        let _ = writeln!(
            out,
            "| {marks} | `{die}` | {} | `{up}` | {} |",
            average(die),
            average(up)
        );
    }
    out
}

/// The skill rolls each class gives a level and the skills it is known for, then the races'
/// extra rolls, in the order New run offers them; and what Intelligence adds.
pub(super) fn skill_rolls(content: &Content) -> String {
    let names = |ids: &[SkillId]| -> String {
        let parts: Vec<&str> = ids.iter().map(|&id| skill_name(content, id)).collect();
        if parts.is_empty() {
            "-".into()
        } else {
            parts.join(", ")
        }
    };
    let mut out = String::from("| Class | Rolls a level | Known for |\n|---|---:|---|\n");
    for c in content.classes() {
        let _ = writeln!(
            out,
            "| {} | {} | {} |",
            c.name,
            c.rolls,
            names(&c.known_for)
        );
    }
    out.push_str("\n| Race | Extra rolls a level |\n|---|---:|\n");
    for r in &content.races().list {
        let _ = writeln!(out, "| {} | {} |", r.name, r.rolls);
    }
    let rules = content.skill_rolls();
    let _ = writeln!(
        out,
        "\nIntelligence adds one roll at {} and one more for every {} points above it \
         ({}: +1, {}: +2, {}: +3).",
        rules.intelligence_from,
        rules.intelligence_every,
        rules.intelligence_from,
        u16::from(rules.intelligence_from) + u16::from(rules.intelligence_every),
        u16::from(rules.intelligence_from) + 2 * u16::from(rules.intelligence_every),
    );
    out
}
