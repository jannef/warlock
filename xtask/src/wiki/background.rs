//! Background stories (spec §8 "Background story"): the tiers' odds, what an effect is worth,
//! and every line of every chapter with its conditions and effects exactly (the game tells the
//! story in words only).

use super::kits::line_text;
use super::text::{Term, link, signed, stat_link, table_text};
use std::fmt::Write as _;
use warlock_core::content::{Content, Marks, StoryEffect, StoryLine, StoryTier};

/// The campaign whose own chapter (the road) the wiki lists: the first campaign's.
const CAMPAIGN: &str = "ascmearc";

/// How many chapters a story of the campaign has.
fn chapters(content: &Content) -> usize {
    content
        .background()
        .map_or(0, |b| b.chapters_for(CAMPAIGN).count())
}

/// A percentage to a tenth, the tenth left out when it is 0: `12 %`, `0.5 %`, `3.9 %`.
fn percent(p: f64) -> String {
    let tenths = (p * 1000.0).round() / 10.0;
    if tenths.fract() == 0.0 {
        format!("{tenths:.0} %")
    } else {
        format!("{tenths:.1} %")
    }
}

/// The odds of `tier` per chapter (`Tiers::of`, counted over every roll).
fn odds(content: &Content, tier: StoryTier) -> f64 {
    let b = content.background().expect("stories");
    let hits = (0..1000).filter(|&r| b.tiers.of(r) == tier).count();
    hits as f64 / 1000.0
}

/// The four tiers: the odds of each per chapter as drawn, and for a story of the first
/// campaign's chapters the chance of at least one and the expected count.
pub(super) fn tiers(content: &Content) -> String {
    if content.background().is_none() {
        return "No story is told.\n".into();
    }
    let n = chapters(content);
    let mut out = format!(
        "| Kind | Per chapter | At least one in a story of {n} | \
         Expected in a story of {n} | Worth | May take |\n|---|---:|---:|---:|---|---|\n"
    );
    for tier in StoryTier::ALL {
        let p = odds(content, tier);
        let (low, high, bane) = tier.band();
        let (once, expected) = match tier {
            StoryTier::Common => ("—".to_string(), format!("{:.2}", p * n as f64)),
            _ => (
                percent(1.0 - (1.0 - p).powi(n as i32)),
                format!("{:.2}", p * n as f64),
            ),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {once} | {expected} | {} to {} | {} |",
            capital_tier(tier),
            percent(p),
            signed_or_zero(low),
            signed_or_zero(high),
            if bane { "yes" } else { "no" }
        );
    }
    out
}

fn capital_tier(tier: StoryTier) -> &'static str {
    match tier {
        StoryTier::Common => "Common",
        StoryTier::Uncommon => "Uncommon",
        StoryTier::Rare => "Rare",
        StoryTier::Fateful => "Fateful",
    }
}

fn signed_or_zero(n: i32) -> String {
    if n == 0 { "0".into() } else { signed(n) }
}

/// What each kind of effect is worth, in points (each line's worth must fit its kind's band).
pub(super) fn points(content: &Content) -> String {
    let Some(b) = content.background() else {
        return "No story is told.\n".into();
    };
    let p = &b.points;
    let rows = [
        ("a point of a stat, given or taken".to_string(), p.stat),
        (
            "a skill started or raised, whatever the dice".into(),
            p.skill,
        ),
        (
            "marks in a weapon skill, whatever the count".into(),
            p.weapon_marks,
        ),
        (
            format!(
                "gold: for every {} coins of what it gives (or takes) on average, rounded up",
                p.gold_per
            ),
            1,
        ),
        ("a plain item".into(), p.item),
        ("an item that may be magic".into(), p.magic_any),
        ("a magic item (magic or rare)".into(), p.magic),
        ("a rare magic item".into(), p.magic_rare),
        ("a nudge to the alignment".into(), p.alignment),
    ];
    let mut out = String::from("| Effect | Points |\n|---|---:|\n");
    for (what, n) in rows {
        let _ = writeln!(out, "| {} | {n} |", table_text(&what));
    }
    out
}

/// The chapter `id` of a story of the first campaign: every line with its kind, weight,
/// conditions, text, effects, and the words it leaves for later lines.
fn chapter(content: &Content, id: &str) -> String {
    let Some(chapter) = content.background().and_then(|b| b.chapter(CAMPAIGN, id)) else {
        return "This chapter has no lines.\n".into();
    };
    let mut lines: Vec<&StoryLine> = chapter.lines.iter().collect();
    // The commonest first, then by weight (heaviest first), then by id.
    lines.sort_by(|a, b| (a.tier, b.weight, &a.id).cmp(&(b.tier, a.weight, &b.id)));
    let mut out = String::from(
        "| Line | Kind | Weight | When | Text | Effects | Tags |\n|---|---|---:|---|---|---|---|\n",
    );
    for line in lines {
        let mut kind = capital_tier(line.tier).to_string();
        if line.only {
            kind.push_str(", always told when it fits");
        }
        let mut tags: Vec<String> = line.tags.iter().map(|t| format!("`{t}`")).collect();
        tags.extend(
            line.set
                .iter()
                .map(|(slot, word)| format!("{slot}: “{word}”")),
        );
        let _ = writeln!(
            out,
            "| `{}` | {kind} | {} | {} | {} | {} | {} |",
            line.id,
            line.weight,
            when_text(content, line),
            table_text(&line.text),
            effects_text(content, line),
            dash(tags.join(", "))
        );
    }
    out
}

fn dash(s: String) -> String {
    if s.is_empty() { "—".into() } else { s }
}

/// `a, b, or c`.
fn any_of(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [init @ .., last] => format!("{} or {last}", init.join(", ")),
    }
}

/// A line's conditions in words, each one a clause: "Dwarf or Duergar; aged 14 or more".
fn when_text(content: &Content, line: &StoryLine) -> String {
    let w = &line.when;
    let mut parts = Vec::new();
    let races: Vec<String> = (w.race.iter())
        .map(|id| match content.race(id) {
            Some(r) => link(Term::Race, &r.id, &r.name),
            None => id.clone(),
        })
        .collect();
    if !races.is_empty() {
        parts.push(any_of(&races));
    }
    let classes: Vec<String> = (w.class.iter())
        .map(|id| content.class(id).map_or(id.clone(), |c| c.name.clone()))
        .collect();
    if !classes.is_empty() {
        parts.push(format!("a {}", any_of(&classes)));
    }
    let genders: Vec<String> = w.gender.iter().map(|g| g.name().to_lowercase()).collect();
    if !genders.is_empty() {
        parts.push(any_of(&genders));
    }
    if let Some(h) = content.horoscope() {
        let animals: Vec<String> = (w.animal.iter())
            .filter_map(|id| h.animals.iter().find(|a| &a.id == id))
            .map(|a| a.name.clone())
            .collect();
        if !animals.is_empty() {
            parts.push(format!("born under the {}", any_of(&animals)));
        }
        let elements: Vec<String> = (w.element.iter())
            .filter_map(|id| h.elements.iter().find(|e| &e.id == id))
            .map(|e| e.name.clone())
            .collect();
        if !elements.is_empty() {
            parts.push(format!("{}-born", any_of(&elements)));
        }
    }
    match w.lucky {
        Some(true) => parts.push("born on a lucky day".into()),
        Some(false) => parts.push("not born on a lucky day".into()),
        None => {}
    }
    if let Some(band) = w.band {
        parts.push(match (band.min, band.max) {
            (Some(min), Some(max)) => format!("starting {} to {}", min.name(), max.name()),
            (Some(min), None) => format!("starting {} or better", min.name()),
            (None, Some(max)) => format!("starting {} or worse", max.name()),
            (None, None) => String::new(),
        });
    }
    let talent = |id: &String| match content.talents().id(id) {
        Some(t) => link(Term::Talent, id, &content.talents().get(t).name),
        None => id.clone(),
    };
    for id in &w.talents {
        parts.push(format!("with {}", talent(id)));
    }
    for id in &w.not_talents {
        parts.push(format!("without {}", talent(id)));
    }
    for &id in &w.skills {
        let skill = content.skill(id);
        parts.push(format!(
            "trained in {}",
            link(Term::Skill, &skill.id, &skill.name)
        ));
    }
    if !w.tags.is_empty() {
        let tags: Vec<String> = w.tags.iter().map(|t| format!("`{t}`")).collect();
        parts.push(format!("after {}", tags.join(" and ")));
    }
    if !w.not_tags.is_empty() {
        let tags: Vec<String> = w.not_tags.iter().map(|t| format!("`{t}`")).collect();
        parts.push(format!("never after {}", any_of(&tags)));
    }
    if let Some(age) = w.age {
        parts.push(match (age.min, age.max) {
            (Some(min), Some(max)) => format!("aged {min} to {max}"),
            (Some(min), None) => format!("aged {min} or more"),
            (None, Some(max)) => format!("aged {max} or less"),
            (None, None) => String::new(),
        });
    }
    let campaigns: Vec<String> = (w.campaign.iter())
        .map(|id| content.campaign(id).map_or(id.clone(), |c| c.name.clone()))
        .collect();
    if !campaigns.is_empty() {
        parts.push(format!("in {}", any_of(&campaigns)));
    }
    parts.retain(|p| !p.is_empty());
    dash(parts.join("; "))
}

/// A line's effects exactly, then its worth: "Strength +1; 4d6 gold (worth 3)".
fn effects_text(content: &Content, line: &StoryLine) -> String {
    let Some(b) = content.background() else {
        return String::new();
    };
    let parts: Vec<String> = (line.effects.iter())
        .map(|effect| match effect {
            StoryEffect::Stat(stat, by) => format!("{} {}", stat_link(*stat), signed(*by)),
            StoryEffect::Gold(dice) if dice.max() < 0 => {
                format!("{} gold taken (never below none)", -dice.max())
            }
            StoryEffect::Gold(dice) => format!("{dice} gold"),
            StoryEffect::Item(item) => {
                let mut text = line_text(content, item);
                if item.magic.is_some() {
                    text.push_str(", identified");
                }
                if item.prefer_trained {
                    text.push_str(", preferring a weapon of a weapon skill you have a rank in");
                }
                text
            }
            StoryEffect::Skill(id, dice) => {
                let skill = content.skill(*id);
                format!("{} {dice}", link(Term::Skill, &skill.id, &skill.name))
            }
            StoryEffect::WeaponMarks(Marks::Skill(id), n) => {
                let skill = content.weapon_skills().skill(*id);
                format!(
                    "{n} marks in {}",
                    link(Term::WeaponSkill, &skill.id, &skill.name)
                )
            }
            StoryEffect::WeaponMarks(Marks::Best, n) => {
                format!("{n} marks in the weapon skill of your highest rank (none without one)")
            }
            StoryEffect::Alignment(by) => format!("alignment {}", signed(*by)),
            StoryEffect::Talent(id, choice) => {
                format!("the talent {}", talent_link(content, id, choice.as_deref()))
            }
            StoryEffect::Flaw(id) => format!("the flaw {}", talent_link(content, id, None)),
        })
        .collect();
    if parts.is_empty() {
        return "—".into();
    }
    let (boons, banes) = line.worth(&b.points);
    format!(
        "{} (worth {})",
        table_text(&parts.join("; ")),
        signed_or_zero(boons - banes)
    )
}

/// Talent `id`, by its name for `choice`, linked to its row.
fn talent_link(content: &Content, id: &str, choice: Option<&str>) -> String {
    match content.talents().id(id) {
        Some(t) => {
            let name = content.talents().get(t).name_for(content, choice);
            link(Term::Talent, id, &name)
        }
        None => id.to_string(),
    }
}

pub(super) fn parents(content: &Content) -> String {
    chapter(content, "parents")
}

pub(super) fn birth(content: &Content) -> String {
    chapter(content, "birth")
}

pub(super) fn childhood(content: &Content) -> String {
    chapter(content, "childhood")
}

pub(super) fn youth(content: &Content) -> String {
    chapter(content, "youth")
}

pub(super) fn trial(content: &Content) -> String {
    chapter(content, "trial")
}

pub(super) fn fortune(content: &Content) -> String {
    chapter(content, "fortune")
}

pub(super) fn leaving(content: &Content) -> String {
    chapter(content, "leaving")
}

pub(super) fn road(content: &Content) -> String {
    chapter(content, "road")
}
