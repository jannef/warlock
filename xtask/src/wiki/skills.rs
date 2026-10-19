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
        Effect::Burden => "less slowdown from your load",
        Effect::Travel => "quicker journeys on the overworld",
        Effect::Bargain => "better prices in shops",
        Effect::Kick => "more Strength for kicking doors in",
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
            Effect::Burden => "Burdened / Strained",
            Effect::Travel => "journeys",
            Effect::Bargain => "prices",
            Effect::Kick => "kicking doors in",
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
                Effect::Dodge => format!("+{n}"),
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

/// **[provisional]** The increase curve: the marks a point needs at a range of values, and all
/// the marks from 0.
pub(super) fn skill_increase(content: &Content) -> String {
    let inc = content.increase();
    let mut out =
        String::from("| Value | Marks for the next point | Marks from 0 |\n|---:|---:|---:|\n");
    for value in [0, 10, 25, 50, 75, 99, 100, 101, 110, 120, 150, 200] {
        let _ = writeln!(
            out,
            "| {value} | {} | {} |",
            inc.need(value),
            inc.total(0, value)
        );
    }
    out
}
