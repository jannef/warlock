//! Surprise and sneak attacks (spec §8 "Surprise and sneak attacks"; `data/surprise.toml`): the
//! rules' numbers and the two statuses, the skill ladders, the initiative check's odds, and what
//! has no vitals. The slip contest's odds are `skills::slipping_past`, the creatures' craft
//! `creatures::creature_surprise`.

use super::combat::percent;
use super::text::{Term, link};
use std::fmt::Write as _;
use warlock_core::check;
use warlock_core::content::{Content, CreatureDef, CreatureId, TalentEffect};
use warlock_core::skills::Effect;

/// A skill's benefit of `effect`, with the skill's id and name, if any skill gives it.
fn benefit(
    content: &Content,
    effect: Effect,
) -> Option<(&str, &str, &warlock_core::skills::Benefit)> {
    (content.skills().iter()).find_map(|s| {
        (s.benefits.iter())
            .find(|b| b.effect == effect)
            .map(|b| (s.id.as_str(), s.name.as_str(), b))
    })
}

/// The talents with an effect `pick` reads a number from, by name: `Long reach (+3)`.
fn talents_with(content: &Content, pick: impl Fn(&TalentEffect) -> Option<i32>) -> Vec<String> {
    let mut out: Vec<(&str, String)> = Vec::new();
    for (_, def) in content.talents().iter() {
        for n in def.effects.iter().filter_map(&pick) {
            let text = format!("{} ({n:+})", link(Term::Talent, &def.id, &def.name));
            out.push((def.name.as_str(), text));
        }
    }
    out.sort_by(|a, b| a.0.cmp(b.0));
    out.into_iter().map(|(_, text)| text).collect()
}

/// The rules' numbers: the check's difficulty, how far a missile still sneak-attacks, how long
/// the player stays aware of a creature, how long a slip lasts, and the two statuses.
pub(super) fn surprise_rules(content: &Content) -> String {
    let Some(rules) = content.surprise() else {
        return "Nobody is ever surprised.\n".into();
    };
    let status = content.status(rules.status);
    let reach = talents_with(content, |e| match e {
        TalentEffect::SneakReach { value } => Some(*value),
        _ => None,
    });
    let mut out = String::from("| Rule | Number |\n|---|---|\n");
    let _ = writeln!(
        out,
        "| The initiative check's difficulty | {} plus the attacker's craft |",
        rules.difficulty
    );
    let mut far = format!("{} cells", rules.missile_reach);
    if !reach.is_empty() {
        let _ = write!(far, "; more with {}", reach.join(", "));
    }
    let _ = writeln!(
        out,
        "| How far a shot, a throw, or an attack spell is still a sneak attack | {far} |"
    );
    let _ = writeln!(
        out,
        "| How long you stay aware of a creature you have seen | {} turns |",
        rules.memory_turns
    );
    let stalk = match rules.slip_turns {
        Some(turns) => format!("{turns} turns at most"),
        None => "nobody slips past anyone".into(),
    };
    let _ = writeln!(
        out,
        "| How long a creature you slipped past stays blind to you | {stalk} |"
    );
    let _ = writeln!(
        out,
        "| {}: the next action costs | {}% of its time, then it ends |",
        link(Term::Status, &status.id, &status.name),
        status.cost
    );
    if let Some(lurking) = content.status_id("lurking").map(|id| content.status(id)) {
        let _ = writeln!(
            out,
            "| {}: lasts | until your next action |",
            link(Term::Status, &lurking.id, &lurking.name)
        );
    }
    out
}

/// The skill values the ladders show.
const LADDER_VALUES: [u16; 13] = [1, 10, 25, 50, 75, 100, 125, 150, 175, 200, 250, 300, 400];

/// What Sneak attack, Initiative, and Stealth's slip give by value: the dice, the harder check
/// those caught unawares make, the check's bonus, the extra time spared and what the Surprised
/// action then costs, and the slip contest's bonus.
pub(super) fn surprise_ladders(content: &Content) -> String {
    let columns = [
        (Effect::SneakDice, "sneak attack dice"),
        (Effect::Surprise, "their check's difficulty"),
        (Effect::Initiative, "your check"),
        (Effect::SurpriseEase, "extra time spared"),
        (Effect::Slip, "slipping past"),
    ];
    let found: Vec<_> = (columns.iter())
        .filter_map(|&(e, what)| benefit(content, e).map(|b| (e, what, b)))
        .collect();
    let cost = (content.surprise()).map_or(100, |r| content.status(r.status).cost);
    let mut out = String::from("| Value |");
    for (e, what, (id, name, _)) in &found {
        let _ = write!(out, " {}: {what} |", link(Term::Skill, id, name));
        if *e == Effect::SurpriseEase {
            out.push_str(" Surprised action costs |");
        }
    }
    let width = found.len() + usize::from(found.iter().any(|f| f.0 == Effect::SurpriseEase));
    out.push_str("\n|---:|");
    out.push_str(&":---:|".repeat(width));
    out.push('\n');
    for value in LADDER_VALUES {
        let _ = write!(out, "| {value} |");
        for (e, _, (_, _, b)) in &found {
            let n = b.at(value);
            let cell = match e {
                _ if n == 0 => "—".to_string(),
                Effect::SneakDice => format!("{n}d6"),
                Effect::SurpriseEase => format!("{n}%"),
                _ => format!("+{n}"),
            };
            let _ = write!(out, " {cell} |");
            if *e == Effect::SurpriseEase {
                let ease = u32::try_from(n.clamp(0, 100)).unwrap_or(0);
                let paid = 100 + cost.saturating_sub(100) * (100 - ease) / 100;
                let _ = write!(out, " {paid}% |");
            }
        }
        out.push('\n');
    }
    out
}

/// The crafts the check's table shows: what the attacker adds to the difficulty.
const CRAFTS: [i32; 8] = [0, 2, 4, 6, 8, 10, 12, 14];

/// How often the initiative check keeps one's wits, by its edge (the Dexterity modifier and
/// what talents and the Initiative skill add) and the attacker's craft (its Sneak attack's and
/// its talents': the thief's Sneak attack). `check::ways` against the rules' difficulty.
pub(super) fn initiative_odds(content: &Content) -> String {
    let Some(rules) = content.surprise() else {
        return "Nobody is ever surprised.\n".into();
    };
    let mut out = String::from("| Your edge |");
    for craft in CRAFTS {
        let _ = write!(out, " craft +{craft} ({}) |", rules.difficulty + craft);
    }
    out.push_str("\n|:---:|");
    out.push_str(&":---:|".repeat(CRAFTS.len()));
    out.push('\n');
    for edge in (-4..=16).step_by(2) {
        let sign = if edge < 0 { "\u{2212}" } else { "+" };
        let _ = write!(out, "| {sign}{} |", i32::abs(edge));
        for craft in CRAFTS {
            let ways = check::ways(edge, rules.difficulty + craft);
            let _ = write!(out, " {} |", percent(ways, 144));
        }
        out.push('\n');
    }
    out
}

/// Every creature with no vitals to find, by name, and why (incorporeal, `vitals = false`, a
/// `no_vitals` tag), with the talents that find vitals in it after all; then the animated
/// objects, all of them.
pub(super) fn no_vitals(content: &Content) -> String {
    let Some(rules) = content.surprise() else {
        return "Everything has vitals.\n".into();
    };
    let finders: Vec<(&str, &str, &[String])> = (content.talents().iter())
        .flat_map(|(_, def)| {
            def.effects.iter().filter_map(move |e| match e {
                TalentEffect::VitalsIn { tags } => {
                    Some((def.id.as_str(), def.name.as_str(), &tags[..]))
                }
                _ => None,
            })
        })
        .collect();
    let mut out = String::from("| Creature | Why | Found by |\n|---|---|---|\n");
    let listed: Vec<(CreatureId, &CreatureDef)> = (super::order::creatures(content).into_iter())
        .filter(|(_, c)| c.id != "player" && c.animated.is_none())
        .collect();
    for (id, c) in listed {
        if content.has_vitals(id, &[]) {
            continue;
        }
        let why = if c.incorporeal.is_some() {
            "incorporeal".to_string()
        } else if !c.vitals {
            "none of its own".to_string()
        } else {
            let tags: Vec<String> = (rules.no_vitals.iter())
                .filter(|t| content.has_tag(id, t))
                .map(|t| format!("`{t}`"))
                .collect();
            tags.join(", ")
        };
        let found: Vec<String> = (finders.iter())
            .filter(|(_, _, tags)| content.has_vitals(id, tags))
            .map(|(tid, name, _)| link(Term::Talent, tid, name))
            .collect();
        let found = if found.is_empty() {
            "-".to_string()
        } else {
            found.join(", ")
        };
        let _ = writeln!(
            out,
            "| {} | {why} | {found} |",
            link(Term::Creature, &c.id, &super::text::capital(&c.name))
        );
    }
    out.push_str("| every [animated object](animated.md) | made, not born | - |\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ladders_and_odds_read_the_data() {
        let content = Content::builtin().expect("content");
        let ladders = surprise_ladders(&content);
        assert!(
            ladders.contains("| 100 | 5d6 | +8 | +8 | 50% | 125% | +8 |"),
            "{ladders}"
        );
        let odds = initiative_odds(&content);
        // 13 at +0: 2d12 reaches 13 in 78 of 144 ways.
        assert!(odds.contains("| +0 | 54% |"), "{odds}");
        let vitals = no_vitals(&content);
        assert!(vitals.contains("incorporeal"), "{vitals}");
        assert!(vitals.contains("`skeleton`"), "{vitals}");
        assert!(vitals.contains("[Anatomist]"), "{vitals}");
        let rules = surprise_rules(&content);
        assert!(
            rules.contains("| 150% of its time, then it ends |"),
            "{rules}"
        );
    }
}
