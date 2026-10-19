//! Hunger (spec §8 "Hunger"): the states, the facts, each race's need, food, and corpses.

use super::burden::factor;
use super::order;
use super::text::{capital, signed};
use std::fmt::Write as _;
use warlock_core::content::{Content, HungerDef};
use warlock_core::hunger::Hunger;
use warlock_core::object::format_weight;
use warlock_core::stats::Stat;
use warlock_core::time::{DAY, Duration};

/// How long `points` of nutrition last a race of hunger `percent`, in whole minutes: "36
/// hours", "1 hour and 15 minutes".
fn lasts(def: &HungerDef, points: i32, percent: u16) -> String {
    let points = u64::try_from(points).unwrap_or(0);
    let seconds = points * u64::from(def.burn_seconds) * 100 / u64::from(percent.max(1));
    Duration::seconds(seconds / 60 * 60).spoken()
}

/// The states of hunger from the most fed down: the nutrition of each, when a human who eats
/// nothing gets there from the start of a run, and what it does.
pub(super) fn hunger_states(content: &Content) -> String {
    let Some(def) = content.hunger() else {
        return "Nobody hungers.\n".into();
    };
    let mut out = String::from(
        "| State | Nutrition | A human eating nothing gets there after | What it does |\n\
         |---|---:|---:|---|\n",
    );
    let weak: Vec<String> = (Stat::ALL.iter())
        .filter(|s| def.weak[**s] != 0)
        .map(|s| format!("{} {}", s.name(), signed(def.weak[*s])))
        .collect();
    let weak = weak.join(", ");
    let (faint, starve) = (def.faint, def.starve);
    for (i, state) in Hunger::ALL.into_iter().enumerate() {
        let range = match (def.least(state), i) {
            (None, _) => format!("below {}", def.states.fainting),
            (Some(n), 0) => format!("{n} and more"),
            (Some(n), _) => {
                let above = def.least(Hunger::ALL[i - 1]).unwrap_or(n);
                format!("{n} to {}", above - 1)
            }
        };
        // A state is reached by falling below the least nutrition of the one above it.
        let after = match i
            .checked_sub(1)
            .and_then(|above| def.least(Hunger::ALL[above]))
        {
            _ if def.state(def.start) == state => "(you start here)".to_string(),
            Some(above) if above <= def.start => lasts(def, def.start - above, 100),
            _ => "-".to_string(),
        };
        let does = match state {
            Hunger::Bloated => format!(
                "everything you do takes ×{} as long",
                factor(i64::from(def.bloated_slow))
            ),
            Hunger::Satiated => "nothing".into(),
            Hunger::NotHungry => "nothing (the status bar shows nothing)".into(),
            Hunger::Hungry => "a warning; becoming hungry stops a rest".into(),
            Hunger::Weak => weak.clone(),
            Hunger::Fainting => format!(
                "as Weak, and a {}% chance each turn to faint, losing {} turns",
                faint.chance, faint.turns
            ),
            Hunger::Starving => format!(
                "as Fainting; no hit points come back, and you lose {} every {}, until you \
                 starve to death",
                starve.damage,
                Duration::seconds(u64::from(starve.seconds)).spoken()
            ),
        };
        let _ = writeln!(out, "| {} | {range} | {after} | {does} |", state.name());
    }
    out
}

/// How fast nutrition burns, where a run starts, the most there can be, exertion, overeating,
/// vomiting, and how corpses rot.
pub(super) fn hunger_facts(content: &Content) -> String {
    let Some(def) = content.hunger() else {
        return "Nobody hungers.\n".into();
    };
    let day = DAY / u64::from(def.burn_seconds);
    let mut out = String::from("| Fact | Value |\n|---|---|\n");
    let mut row = |fact: &str, value: String| {
        let _ = writeln!(out, "| {fact} | {value} |");
    };
    let every = match def.burn_seconds {
        60 => "a point a minute".to_string(),
        3600 => "a point an hour".to_string(),
        s => format!("a point every {s} seconds"),
    };
    row(
        "A human burns",
        format!("{every}: {day} a day, resting, walking, or travelling the overworld alike"),
    );
    row(
        "A run starts with",
        format!(
            "{} ({})",
            def.start,
            def.state(def.start).name().to_lowercase()
        ),
    );
    row("The most there can be", def.max.to_string());
    let exertion: Vec<String> = (warlock_core::content::EXERTIONS.iter())
        .map(|k| format!("{} {}", def.exertion(*k), k.id()))
        .collect();
    row(
        "Exertion burns more (points, at your race's need)",
        exertion.join(", "),
    );
    let o = def.overeating;
    let choke = if o.choke == 0 {
        "never chokes".to_string()
    } else {
        format!("a {}% chance to choke to death", o.choke)
    };
    row(
        "Overeating (a meal past the most)",
        format!(
            "{choke}; a {}% chance to vomit; else what does not fit is lost",
            o.vomit
        ),
    );
    row(
        "Vomiting leaves you with at most",
        format!(
            "{} ({}); what made you vomit feeds nothing",
            def.vomit_leaves,
            def.state(def.vomit_leaves).name().to_lowercase()
        ),
    );
    if let Some(c) = &def.corpses {
        row(
            "Corpses",
            format!(
                "rotten {} after the death, gone after {}",
                c.rotten.spoken(),
                c.gone.spoken()
            ),
        );
        row(
            "Eating a rotten corpse",
            format!(
                "a {}% chance to vomit; else it makes you sick{}, and it feeds you",
                c.rotten_vomit,
                (c.sickness.as_deref())
                    .and_then(|s| content.status_id(s))
                    .map_or(String::new(), |s| format!(
                        " ({})",
                        content.status(s).name.to_lowercase()
                    ))
            ),
        );
    }
    out
}

/// What each race needs to eat next to a human, burns in a day, and how long it goes from the
/// start of a run before it is hungry.
pub(super) fn hunger_races(content: &Content) -> String {
    let Some(def) = content.hunger() else {
        return "Nobody hungers.\n".into();
    };
    let mut out =
        String::from("| Race | Needs | Burns a day | Hungry after |\n|---|:---:|---:|---:|\n");
    let day = DAY / u64::from(def.burn_seconds);
    for r in &content.races().list {
        let _ = writeln!(
            out,
            "| {} | ×{} | {} | {} |",
            r.name,
            factor(i64::from(r.hunger)),
            day * u64::from(r.hunger) / 100,
            lasts(def, def.start - def.states.not_hungry, r.hunger)
        );
    }
    out
}

/// The food objects: what each gives, how long that feeds a human, how many turns it takes to
/// eat, and what it weighs.
pub(super) fn food(content: &Content) -> String {
    let Some(def) = content.hunger() else {
        return "Nobody hungers.\n".into();
    };
    let mut out = String::from(
        "| Food | Nutrition | Feeds a human for | Turns to eat | Weight |\n\
         |---|---:|---:|:---:|---:|\n",
    );
    for (_, o) in order::objects(content) {
        let (Some(food), None) = (o.food, o.corpse) else {
            continue;
        };
        let nutrition = i32::try_from(food.nutrition).unwrap_or(i32::MAX);
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            capital(&o.name),
            food.nutrition,
            lasts(def, nutrition, 100),
            food.turns,
            format_weight(u64::from(o.weight))
        );
    }
    out
}

/// Every creature's corpse: what it weighs (the creature's weight) and what it gives (by size).
/// What eating it does beyond that stays for the player to find out.
pub(super) fn corpses(content: &Content) -> String {
    let Some(def) = content.hunger() else {
        return "Nobody hungers.\n".into();
    };
    let mut out = String::from(
        "| Corpse | Size | Weight | Nutrition | Feeds a human for | Turns to eat |\n\
         |---|---|---:|---:|---:|:---:|\n",
    );
    for (_, o) in order::objects(content) {
        let (Some(food), Some(creature)) = (o.food, o.corpse) else {
            continue;
        };
        let size = content.size(content.creature(creature).size);
        let nutrition = i32::try_from(food.nutrition).unwrap_or(i32::MAX);
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} |",
            capital(&o.name),
            size.name,
            format_weight(u64::from(o.weight)),
            food.nutrition,
            lasts(def, nutrition, 100),
            food.turns
        );
    }
    out
}
