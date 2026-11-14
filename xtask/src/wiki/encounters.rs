//! Overworld encounters (spec §7 "Overworld encounters"; `data/encounters.toml` and each
//! campaign's `data/encounters/`; `docs/dev/encounters.md`): the rates by landscape and phase, the
//! checks and their odds, the skill ladders, where everyone stands, and each campaign's tables.

use super::text::{Term, capital, link, percent_of, signed};
use std::fmt::Write as _;
use warlock_core::check;
use warlock_core::content::{
    Content, EncounterEntry, EncounterTable, EncounterWhen, EntryStance, Landscape, TalentEffect,
};
use warlock_core::daylight::DayPhase;
use warlock_core::dice::Dice;
use warlock_core::skills::Effect;

/// The landscapes a step rolls on, in the order the pages list them: safest first.
const LANDSCAPES: [Landscape; 7] = [
    Landscape::Road,
    Landscape::Plains,
    Landscape::Grassland,
    Landscape::Forest,
    Landscape::Hills,
    Landscape::DenseForest,
    Landscape::Swamp,
];

/// A landscape's name in prose: "dense forest".
fn landscape_name(l: Landscape) -> String {
    l.id().replace('_', " ")
}

/// A chance in basis points as a percent: "0.33%", "1%", "16%".
fn basis_points(bp: u64) -> String {
    let p = bp as f64 / 100.0;
    if p.fract() == 0.0 {
        format!("{p:.0}%")
    } else if p < 1.0 {
        format!("{p:.2}%")
    } else {
        format!("{p:.1}%")
    }
}

/// Each landscape's chance an hour by day, in twilight, and at night; a step onto each of its
/// terrains with a local area by day, at a normal pace; and how many a twelve-hour day's march
/// meets, and a night's rest.
pub(super) fn rates(content: &Content) -> String {
    let Some(m) = content.meetings() else {
        return "Nobody meets anybody on the way.\n".into();
    };
    let mut out = String::from(
        "| Landscape | An hour by day | In twilight | At night | A step by day | \
         A day's march (12 hours) | An hour of rest by day |\n\
         |---|---:|---:|---:|---|---:|---:|\n",
    );
    for l in LANDSCAPES {
        let rate = u64::from(m.rate_of(l));
        let by = |phase: DayPhase| rate * u64::from(m.phase.of(phase)) / 100;
        let steps: Vec<String> = (content.terrain_defs().iter())
            .filter(|t| t.landscape == Some(l) && t.local.is_some())
            .filter_map(|t| {
                let minutes = u64::from(t.travel_minutes?);
                Some(format!(
                    "{} ({minutes} min): {}",
                    capital(&t.name),
                    basis_points(rate * minutes / 60)
                ))
            })
            .collect();
        let day = by(DayPhase::Day);
        let march = day as f64 * 12.0 / 10_000.0;
        let rest = day * u64::from(m.rest_percent) / 100;
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {march:.2} | {} |",
            capital(&landscape_name(l)),
            basis_points(day),
            basis_points(by(DayPhase::Twilight)),
            basis_points(by(DayPhase::Night)),
            steps.join("; "),
            basis_points(rest),
        );
    }
    out
}

/// The terms of the two lookouts and of slipping away, with their numbers.
pub(super) fn checks(content: &Content) -> String {
    let Some(m) = content.meetings() else {
        return "Nobody meets anybody on the way.\n".into();
    };
    let c = &m.checks;
    let alert: Vec<String> = (content.talents().iter())
        .flat_map(|(_, t)| t.effects.iter().map(move |e| (t, e)))
        .filter_map(|(t, e)| match e {
            TalentEffect::Notice { value } => Some(format!(
                "{} ({})",
                super::talents::talent_link(content, &t.id, &t.name),
                signed(*value)
            )),
            _ => None,
        })
        .collect();
    let cover: Vec<String> = (LANDSCAPES.iter())
        .map(|&l| format!("{} {}", landscape_name(l), signed(m.cover_of(l))))
        .collect();
    let mut out = String::from("| Term | Number |\n|---|---|\n");
    let rows = [
        (
            "Every check's difficulty".to_string(),
            c.difficulty.to_string(),
        ),
        (
            "Cover, added to both lookouts' difficulty, by landscape".into(),
            cover.join(", "),
        ),
        (
            "The dark, against eyes without darkvision (yours halved with darkvision)".into(),
            format!("night {}, twilight {}", signed(c.night), signed(c.twilight)),
        ),
        (
            "Rain, against both".into(),
            format!(
                "rain {}, heavy rain {}",
                signed(c.rain),
                signed(c.heavy_rain)
            ),
        ),
        (
            "A lit light you carry at night or in twilight, off the party's difficulty".into(),
            format!("\u{2212}{}", c.light),
        ),
        (
            "Talents that add to your lookout".into(),
            if alert.is_empty() {
                "none".into()
            } else {
                alert.join(", ")
            },
        ),
        (
            "Slipping away: its difficulty".into(),
            m.evade.difficulty.to_string(),
        ),
        (
            "Slipping away once they have seen you".into(),
            signed(m.evade.aware),
        ),
        (
            "Slipping away from a quick leader: per this much speed above normal".into(),
            format!("+1 per {}", m.evade.pursuit_per),
        ),
        (
            "The detour slipping away takes, whatever comes of it".into(),
            format!("{}% of the step's time", m.evade.detour_percent),
        ),
        (
            "Lying in wait takes".into(),
            format!("{} minutes", m.ambush_wait / 60),
        ),
        (
            "An hour of rest rolls".into(),
            format!("{}% of the landscape's rate", m.rest_percent),
        ),
    ];
    for (term, number) in rows {
        let _ = writeln!(out, "| {term} | {number} |");
    }
    out
}

/// The totals the odds table shows a column for: the difficulty and what may be added to it.
const NEEDED: [i32; 8] = [11, 13, 15, 17, 19, 21, 23, 25];

/// How often a check is made, by its edge and the total it needs (`check::ways` of 144).
pub(super) fn odds(_content: &Content) -> String {
    let mut out = String::from("| Your edge |");
    for n in NEEDED {
        let _ = write!(out, " needs {n} |");
    }
    out.push_str("\n|:---:|");
    out.push_str(&":---:|".repeat(NEEDED.len()));
    out.push('\n');
    for edge in (-4..=20).step_by(2) {
        let _ = write!(out, "| {} |", signed(edge));
        for n in NEEDED {
            let _ = write!(out, " {} |", percent_of(check::ways(edge, n), 144));
        }
        out.push('\n');
    }
    out
}

/// The skill values the ladders show.
const LADDER_VALUES: [u16; 10] = [1, 10, 25, 50, 75, 100, 125, 150, 200, 300];

/// What each skill gives the lookout, the stalk, and the slip away, by value.
pub(super) fn ladders(content: &Content) -> String {
    let words = |e: Effect| match e {
        Effect::Lookout => "lookout",
        Effect::Stalk => "stalk",
        _ => "slipping away",
    };
    let columns: Vec<(String, &warlock_core::skills::Benefit)> = (content.skills().iter())
        .flat_map(|s| s.benefits.iter().map(move |b| (s, b)))
        .filter(|(_, b)| matches!(b.effect, Effect::Lookout | Effect::Stalk | Effect::Evade))
        .map(|(s, b)| {
            let name = link(Term::Skill, &s.id, &s.name);
            (format!("{name}: {}", words(b.effect)), b)
        })
        .collect();
    let mut out = String::from("| Value |");
    for (name, _) in &columns {
        let _ = write!(out, " {name} |");
    }
    out.push_str("\n|---:|");
    out.push_str(&":---:|".repeat(columns.len()));
    out.push('\n');
    for value in LADDER_VALUES {
        let _ = write!(out, "| {value} |");
        for (_, b) in &columns {
            let n = b.at(value);
            let cell = if n == 0 { "—".to_string() } else { signed(n) };
            let _ = write!(out, " {cell} |");
        }
        out.push('\n');
    }
    out
}

/// Where you and the party stand when an area opens, by how it came about.
pub(super) fn positions(content: &Content) -> String {
    let Some(m) = content.meetings() else {
        return "Nobody meets anybody on the way.\n".into();
    };
    let p = m.positions;
    let mut out = String::from(
        "| How it came about | You stand (of the way from the edge to the middle) | The party | \
         Who acts first |\n|---|---:|---|---|\n",
    );
    let rows = [
        (
            "You lie in wait",
            p.player.ambush,
            format!(
                "{} cells further in, walking toward you and past you, unaware of you",
                p.ambush
            ),
            "you, and they owe one action besides",
        ),
        (
            "You stand and fight, or hail or hunt them",
            p.player.fight,
            format!("{} cells further in, spread across the way", p.fight),
            "whoever is quicker",
        ),
        (
            "You fail to slip away",
            p.player.close,
            format!("{} cells back toward the edge, between you and it", p.close),
            "whoever is quicker",
        ),
        (
            "You are ambushed",
            p.player.ambushed,
            format!(
                "around you, {} cells off, the leader on the side of the edge",
                p.ambushed
            ),
            "they do: one action before you, and you may be surprised",
        ),
    ];
    for (how, percent, party, first) in rows {
        let _ = writeln!(out, "| {how} | {percent}% | {party} | {first} |");
    }
    out
}

/// The conditions of a table or an entry in words.
fn when_text(content: &Content, w: &EncounterWhen) -> Vec<String> {
    let mut out = Vec::new();
    if !w.regions.is_empty() {
        out.push(w.regions.join(", "));
    }
    if !w.landscapes.is_empty() {
        let names: Vec<String> = w.landscapes.iter().map(|&l| landscape_name(l)).collect();
        out.push(names.join(", "));
    }
    if !w.terrains.is_empty() {
        let names: Vec<String> = (w.terrains.iter())
            .map(|&t| content.terrain(t).name.clone())
            .collect();
        out.push(names.join(", "));
    }
    if !w.phase.is_empty() {
        let names: Vec<&str> = (w.phase.iter())
            .map(|p| match p {
                DayPhase::Day => "by day",
                DayPhase::Twilight => "in twilight",
                DayPhase::Night => "at night",
            })
            .collect();
        out.push(names.join(" or "));
    }
    if !w.weather.is_empty() {
        let names: Vec<String> = w.weather.iter().map(|k| k.id().replace('_', " ")).collect();
        out.push(format!("in {}", names.join(" or ")));
    }
    if let Some(max) = w.level_max {
        out.push(format!("up to level {max}"));
    }
    let quest = |id: &str| {
        content
            .quest_id(id)
            .map_or_else(|| id.to_string(), |q| content.quest(q).name.clone())
    };
    if let Some(q) = &w.quest_active {
        out.push(format!("while [{}](quests.md) is under way", quest(q)));
    }
    if let Some(q) = &w.quest_done {
        out.push(format!("once [{}](quests.md) is done", quest(q)));
    }
    if let Some(q) = &w.quest_not {
        out.push(format!("until [{}](quests.md) is given", quest(q)));
    }
    if let Some((q, v)) = &w.quest_variant {
        let target = (content.quest_id(q))
            .and_then(|id| content.quest(id).variant(v).map(|(_, v)| v.target.clone()))
            .and_then(|t| content.creature_id(&t))
            .map_or_else(|| v.clone(), |c| capital(&content.creature(c).name));
        out.push(format!("while it names {target}"));
    }
    let place = |id: &str| {
        let campaign_place = (content.campaigns().iter()).find_map(|c| {
            content
                .hidden_location(&c.id, id)
                .map(|(_, h)| h.name.clone())
        });
        campaign_place.unwrap_or_else(|| id.to_string())
    };
    if let Some(h) = &w.revealed {
        out.push(format!(
            "once [{}](overworld.md#hidden-locations) is known",
            place(h)
        ));
    }
    if let Some(h) = &w.hidden {
        out.push(format!(
            "until [{}](overworld.md#hidden-locations) is known",
            place(h)
        ));
    }
    if let Some(d) = w.after_days {
        out.push(format!("from day {d} of the run"));
    }
    if let Some(d) = w.before_days {
        out.push(format!("before day {d} of the run"));
    }
    if let Some(class) = &w.calling {
        let who = super::dialogue::calling_text(content, None, class);
        out.push(format!("only for {who}"));
    }
    out
}

/// How many dice give: "1d3+1", or a plain number when they always give the same.
fn count(dice: Dice) -> String {
    if dice.min() == dice.max() {
        dice.min().to_string()
    } else {
        dice.to_string().replace('-', "\u{2212}")
    }
}

/// Who an entry brings: its groups, or what it draws from a dungeon's folk.
fn who(content: &Content, e: &EncounterEntry) -> String {
    if let Some(d) = &e.draw {
        let name = content
            .dungeon(&d.pool)
            .map_or_else(|| d.pool.clone(), |b| b.name.clone());
        return format!(
            "{} drawn from the folk of [{name}](dungeons.md#the-dungeons) at danger {}",
            count(d.count),
            d.danger
        );
    }
    let rows: Vec<String> = (e.group.iter())
        .map(|&(c, dice)| {
            let def = content.creature(c);
            let name = link(Term::Creature, &def.id, &def.name);
            format!("{} {name}", count(dice))
        })
        .collect();
    rows.join(", ")
}

/// A campaign's tables: each table's conditions, rate, and what it replaces, then its entries.
fn tables(content: &Content, campaign: &str) -> String {
    let all: Vec<&EncounterTable> = content.encounter_tables(campaign).collect();
    if all.is_empty() {
        return "Nobody is met on the way here.\n".into();
    }
    let mut out = String::from(
        "| Table | Where and when | Rate | Who | Weight | How many | Kind |\n\
         |---|---|---:|---|---:|---|---|\n",
    );
    for t in all {
        let mut when = when_text(content, &t.when);
        if !t.replaces.is_empty() {
            when.push(format!("instead of {}", t.replaces.join(", ")));
        }
        let when = if when.is_empty() {
            "anywhere".to_string()
        } else {
            when.join("; ")
        };
        let _ = writeln!(
            out,
            "| <a id=\"encounter-table-{}\"></a>**{}** | {when} | {}% | | | | |",
            t.id.replace('_', "-"),
            t.id,
            t.rate_percent
        );
        for e in &t.entries {
            let kind = match e.stance {
                EntryStance::Hostile if e.talk_past => {
                    "hostile, [talked past](encounters.md#talking-past)"
                }
                EntryStance::Hostile => "hostile",
                EntryStance::Peaceful => "peaceful",
                EntryStance::Wildlife => "wildlife",
            };
            let narrow = when_text(content, &e.when);
            let name = if narrow.is_empty() {
                e.name.clone()
            } else {
                format!("{} ({})", e.name, narrow.join("; "))
            };
            let _ = writeln!(
                out,
                "| | | | {name} | {} | {} | {kind} |",
                e.weight,
                who(content, e)
            );
        }
    }
    out
}

/// The Ascmearc's tables.
pub(super) fn ascmearc_tables(content: &Content) -> String {
    tables(content, "ascmearc")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tables_read_the_data() {
        let content = Content::builtin().expect("content");
        let rates = rates(&content);
        assert!(rates.contains("| Road | 1% | 1.5% | 2% |"), "{rates}");
        assert!(rates.contains("(20 min): 0.33%"), "{rates}");
        let checks = checks(&content);
        assert!(
            checks.contains("| Every check's difficulty | 13 |"),
            "{checks}"
        );
        assert!(
            checks.contains("[Alert](talents.md#talent-alert) (+2)"),
            "{checks}"
        );
        let odds = odds(&content);
        assert!(odds.contains("| +0 | 68.8% | 54.2% |"), "{odds}");
        let ladders = ladders(&content);
        assert!(
            ladders.contains("[Spot](skills.md#skill-spot): lookout"),
            "{ladders}"
        );
        assert!(ladders.contains("| 100 | +8 |"), "{ladders}");
        let positions = positions(&content);
        assert!(
            positions.contains("| You are ambushed | 100% |"),
            "{positions}"
        );
        let tables = ascmearc_tables(&content);
        assert!(tables.contains("encounter-table-downs-night"), "{tables}");
        assert!(tables.contains("while it names Black Wystan"), "{tables}");
        assert!(
            tables.contains("drawn from the folk of [Cealdhlāw]"),
            "{tables}"
        );
        assert!(!tables.contains("up to level"), "no level gates: {tables}");
        let talked = tables.matches("[talked past]").count();
        assert_eq!(talked, 6, "the flagged entries: {tables}");
        assert!(
            tables.contains("only for a [bard](class-bard.md) of level"),
            "the Strange Spirit's calling: {tables}"
        );
    }
}
