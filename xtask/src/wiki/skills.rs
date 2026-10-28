//! Skills (spec §8 "Skills"): what each does and trains on, what it gives by value, who trains
//! it, the provisional increase curve, the odds of stealing (spec §8 "Stealing"), and what
//! foraging finds (spec §8 "Foraging").

use super::burden::factor;
use super::combat::percent;
use std::fmt::Write as _;
use warlock_core::burden::Burden;
use warlock_core::check::{CONTEST_WAYS, contest_ways};
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
        let gives: Vec<String> = (skill.benefits.iter())
            .map(|b| match gated_effect(b.effect) {
                true => format!("{} (**trained only**)", effect_name(b.effect)),
                false => effect_name(b.effect).to_string(),
            })
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

/// Whether `effect` is given only to those trained in its skill at all
/// (`Skills::trains_effect`: noticing in passing, identifying, disarming, picking, stealing, treating,
/// foraging): to the untrained, a magic item's bonus to the skill gives none of it.
pub(super) fn gated_effect(effect: Effect) -> bool {
    matches!(
        effect,
        Effect::Spot
            | Effect::Identify
            | Effect::Disarm
            | Effect::Pick
            | Effect::Steal
            | Effect::Heal
            | Effect::Forage
    )
}

/// Whether everything `skill` gives is for those trained in it only (a skill suffix of it helps
/// them only).
pub(super) fn gated(skill: &warlock_core::content::SkillDef) -> bool {
    !skill.benefits.is_empty() && skill.benefits.iter().all(|b| gated_effect(b.effect))
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
        Effect::Study => "studying spells from scrolls",
        Effect::Disarm => "disarming traps",
        Effect::Burden => "less slowdown from your load",
        Effect::Travel => "quicker journeys on the overworld",
        Effect::Forage => "foraging at all, and finding food more often",
        Effect::Bargain => "better prices in shops",
        Effect::Identify => "now and then telling something you carry for what it is",
        Effect::Kick => "more Strength for kicking doors in",
        Effect::Steal => "stealing at all, and better",
        Effect::Heal => "treating your bleeding, poison, and diseases",
        Effect::Pick => "picking locks",
        Effect::Temper => "tempering at a forge's anvil",
        Effect::Speed => "quicker in everything",
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
            Effect::Study => "study check",
            Effect::Disarm => "disarming traps",
            Effect::Burden => "Burdened / Strained",
            Effect::Travel => "journeys",
            Effect::Forage => "foraging",
            Effect::Bargain => "prices",
            Effect::Identify => "identifying, each hour",
            Effect::Kick => "kicking doors in",
            Effect::Steal => "stealing",
            Effect::Heal => "treating",
            Effect::Pick => "picking locks",
            Effect::Temper => "tempering",
            Effect::Speed => "speed",
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
                Effect::Dodge
                | Effect::Search
                | Effect::Spot
                | Effect::Study
                | Effect::Steal
                | Effect::Heal
                | Effect::Disarm
                | Effect::Pick
                | Effect::Temper => format!("+{n}"),
                Effect::SearchReach => format!("+{n} cell{}", if n == 1 { "" } else { "s" }),
                Effect::Kick => format!("+{n} Strength"),
                Effect::Quiet | Effect::Travel => format!("\u{2212}{n}%"),
                Effect::Bargain => format!("{n}% better"),
                Effect::Forage => format!("+{n}%"),
                Effect::Identify => format!("{n}%"),
                Effect::Speed => format!("+{n}%"),
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

/// How often a theft goes unnoticed (spec §8 "Stealing"), by the thief's edge: their Dexterity
/// modifier plus what their Steal skill gives, less the victim's Willpower modifier. The
/// contest's dice are the same on both sides, so only the difference counts.
pub(super) fn stealing(_content: &Content) -> String {
    let mut out = String::from("| Your edge | Unnoticed | Caught |\n|:---:|:---:|:---:|\n");
    for edge in -6..=10 {
        let won = contest_ways(edge, 0);
        let sign = if edge < 0 { "\u{2212}" } else { "+" };
        let _ = writeln!(
            out,
            "| {sign}{} | {} | {} |",
            edge.abs(),
            percent(won, CONTEST_WAYS),
            percent(CONTEST_WAYS - won, CONTEST_WAYS)
        );
    }
    out
}

/// What a forage finds on each land (spec §8 "Foraging"; `data/forage.toml`), by landscape:
/// the chance before Survival, how many of a find, and each find's share, by name.
pub(super) fn foraging(content: &Content) -> String {
    let Some(rules) = content.forage() else {
        return "Nobody forages.\n".into();
    };
    let mut out = format!(
        "A forage takes {} minutes.\n\n| Land | Chance | How many | Finds |\n|---|:---:|:---:|---|\n",
        rules.seconds / 60
    );
    let mut grounds: Vec<_> = rules.grounds.iter().collect();
    grounds.sort_by_key(|g| g.landscape);
    for g in grounds {
        let total = g.total_weight();
        let mut finds: Vec<(String, u32)> = (g.finds.iter())
            .map(|&(o, w)| (content.object(o).name.clone(), w))
            .collect();
        finds.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let finds: Vec<String> = (finds.iter())
            .map(|(name, w)| format!("{name} {}", percent(*w, total)))
            .collect();
        let _ = writeln!(
            out,
            "| {} | {}% | `{}` | {} |",
            g.landscape.id().replace('_', " "),
            g.chance,
            g.count,
            finds.join(", ")
        );
    }
    out
}

/// What the Heal skill treats (spec §8 "Treating oneself"): each status with a `treat` table at
/// its least and most degrees, then each disease, with the check's difficulty, the chance of
/// making it at a modifier of +0 and of +8 (Intelligence modifier plus Heal), and what comes of
/// a made and a failed try. Statuses in file order, diseases by name.
pub(super) fn heal_targets(content: &Content) -> String {
    let mut out = String::from(
        "| Ailment | Difficulty | Chance at +0 | Chance at +8 | Made | Failed |\n\
         |---|---:|---:|---:|---|---|\n",
    );
    let odds = |difficulty: i32| {
        let at = |m| percent(warlock_core::check::ways(m, difficulty), 144);
        (at(0), at(8))
    };
    for (_, def) in content.status_ids() {
        let Some(treat) = def.treat.as_ref() else {
            continue;
        };
        let (made, failed) = if treat.stops {
            ("it stops", "try again as often as you like")
        } else {
            ("it eases a degree", "no more tries at this dose")
        };
        let degrees = if def.max > 1 {
            vec![1, def.max]
        } else {
            vec![1]
        };
        for severity in degrees {
            let difficulty = treat.difficulty_at(severity);
            let (zero, eight) = odds(difficulty);
            let s = if severity == 1 { "" } else { "s" };
            let _ = writeln!(
                out,
                "| {} ({severity} degree{s}) | {difficulty} | {zero} | {eight} | {made} | {failed} |",
                def.name
            );
        }
    }
    let mut diseases: Vec<_> = content.disease_defs().iter().collect();
    diseases.sort_by(|a, b| a.name.cmp(&b.name));
    for def in diseases {
        let (zero, eight) = odds(def.difficulty);
        let made = if def.recovers() {
            "counts as a check won toward recovery (never the last one needed)"
        } else {
            "its next turn for the worse comes a full interval later"
        };
        let _ = writeln!(
            out,
            "| {} | {} | {zero} | {eight} | {made} | no more tries at this bout |",
            capitalized(&def.name),
            def.difficulty
        );
    }
    out
}

fn capitalized(s: &str) -> String {
    let mut chars = s.chars();
    (chars.next()).map_or_else(String::new, |c| c.to_uppercase().chain(chars).collect())
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
/// extra rolls, each by name; and what Intelligence adds.
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
    let mut classes: Vec<_> = content.classes().iter().collect();
    classes.sort_by(|a, b| a.name.cmp(&b.name));
    for c in classes {
        let _ = writeln!(
            out,
            "| {} | {} | {} |",
            c.name,
            c.rolls,
            names(&c.known_for)
        );
    }
    out.push_str("\n| Race | Extra rolls a level |\n|---|---:|\n");
    let mut races: Vec<_> = content.races().list.iter().collect();
    races.sort_by(|a, b| a.name.cmp(&b.name));
    for r in races {
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
