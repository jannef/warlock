//! Learning spells (spec §8 "Spells"): the study check's chances by overreach and bonus, how
//! long a study takes by tier, the classes' affinities with the schools, and the mishaps.

use std::fmt::Write as _;
use warlock_core::check;
use warlock_core::content::{Band, Content, MishapEffect, TIERS};
use warlock_core::time::Duration;

/// The bonuses the chance table shows a column for.
const BONUSES: [i32; 7] = [-2, 0, 2, 4, 6, 8, 10];

/// The study check's difficulty by overreach, and the chance of learning at each bonus, with
/// nothing known; then what the tiers known add.
pub(super) fn chances(content: &Content) -> String {
    let Some(rules) = content.study_rules() else {
        return String::new();
    };
    let mut out = String::from("| Overreach | Difficulty |");
    for b in BONUSES {
        let _ = write!(out, " Bonus {b:+} |");
    }
    out.push_str("\n|:---:|:---:|");
    out.push_str(&":---:|".repeat(BONUSES.len()));
    out.push('\n');
    let _ = write!(out, "| 0 or less | none |");
    for _ in BONUSES {
        out.push_str(" certain |");
    }
    out.push('\n');
    for overreach in 1..TIERS as i32 {
        let difficulty = rules.difficulty(overreach, 0);
        let _ = write!(out, "| {overreach} | {difficulty} |");
        for b in BONUSES {
            let ways = check::ways(b, difficulty);
            let percent = f64::from(ways) * 100.0 / 144.0;
            let _ = write!(out, " {percent:.0} % ({}) |", rules.words(ways));
        }
        out.push('\n');
    }
    let _ = writeln!(
        out,
        "\nEvery {} tiers of the spells you know (a cantrip counting 1) make it a point harder.",
        rules.known_per_point
    );
    out
}

/// How long studying a spell takes by its tier, and with Quick study's time.
pub(super) fn times(content: &Content) -> String {
    let Some(rules) = content.study_rules() else {
        return String::new();
    };
    let quick = (content.talents().iter())
        .find(|(_, t)| t.id == "quick_study")
        .and_then(|(_, t)| {
            t.effects.iter().find_map(|e| match e {
                warlock_core::content::TalentEffect::Study { percent, .. } => Some(*percent),
                _ => None,
            })
        });
    let mut out =
        String::from("| Tier | Turns | Takes | With Quick study |\n|:---:|:---:|:---:|:---:|\n");
    let took = |turns: u32| Duration::seconds(u64::from(turns) * 6).spoken();
    for tier in 0..TIERS as u8 {
        let turns = warlock_core::world::study_turns(rules, tier, 0);
        let quick = quick.map_or_else(
            || "—".to_string(),
            |p| took(warlock_core::world::study_turns(rules, tier, p)),
        );
        let _ = writeln!(out, "| {tier} | {turns} | {} | {quick} |", took(turns));
    }
    out
}

/// Each class's affinity with the schools: points off the study check's difficulty.
pub(super) fn affinities(content: &Content) -> String {
    let mut classes: Vec<_> = (content.classes().iter())
        .filter(|c| !c.affinity.is_empty())
        .collect();
    classes.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out = String::from("| Class | Affinity |\n|---|---|\n");
    for class in classes {
        let schools: Vec<String> = (class.affinity.iter())
            .map(|(school, n)| format!("{} {n:+}", school.id()))
            .collect();
        let _ = writeln!(out, "| {} | {} |", class.name, schools.join(", "));
    }
    out.push_str("\nEvery other class, and every school not named, is 0.\n");
    out
}

/// The mishaps by band, each with its chance in its band's draw (lucky ones counted in every
/// band) and what it does.
pub(super) fn mishaps(content: &Content) -> String {
    let all = content.mishaps();
    let mut out = String::from("| Band | Mishap | Chance | What it does |\n|---|---|:---:|---|\n");
    for band in [Band::Slight, Band::Bad, Band::Terrible, Band::Lucky] {
        let mut rows: Vec<_> = all.iter().filter(|m| m.band == band).collect();
        rows.sort_by(|a, b| b.weight.cmp(&a.weight).then(a.id.cmp(&b.id)));
        for m in rows {
            let chance = if band == Band::Lucky {
                [Band::Slight, Band::Bad, Band::Terrible]
                    .iter()
                    .map(|&b| percent_in(content, b, m.weight))
                    .collect::<Vec<_>>()
                    .join(" / ")
            } else {
                percent_in(content, band, m.weight)
            };
            let _ = writeln!(
                out,
                "| {} | {} | {chance} | {} |",
                band.id(),
                m.message,
                effect_text(content, m.effect)
            );
        }
    }
    out.push_str(
        "\nA lucky mishap's chance is given in a slight, a bad, and a terrible failure's draw.\n",
    );
    out
}

/// `weight`'s share of `band`'s draw (its mishaps and the lucky ones), in percent.
fn percent_in(content: &Content, band: Band, weight: u16) -> String {
    let total: u32 = warlock_core::content::drawn_among(content.mishaps(), band)
        .map(|m| u32::from(m.weight))
        .sum();
    if total == 0 {
        return "—".into();
    }
    format!("{:.0} %", f64::from(weight) * 100.0 / f64::from(total))
}

/// What a mishap's effect does, in words.
fn effect_text(content: &Content, effect: MishapEffect) -> String {
    match effect {
        MishapEffect::Nothing => "nothing more".into(),
        MishapEffect::Status { status, severity } => {
            format!("{} ×{severity}", content.status(status).name)
        }
        MishapEffect::Backfire => {
            "the scroll's own spell goes off on you (its burst around you, its bolt through you), \
             never past the cap of the spell it teaches"
                .into()
        }
        MishapEffect::Mana { percent } => format!("{percent} % of your mana is lost"),
        MishapEffect::Teleport => "you are moved to a random free place on the level".into(),
        MishapEffect::LoseMark => "a stat training mark is lost".into(),
        MishapEffect::Drain {
            stat,
            points,
            difficulty,
        } => format!(
            "{} loses {points} for good unless you make a Willpower check against {difficulty}",
            stat.map_or("a random stat", |s| s.name())
        ),
        MishapEffect::Summon { creature } => {
            format!(
                "a hostile {} comes next to you",
                content.creature(creature).name
            )
        }
        MishapEffect::Learn => "you learn the spell anyway".into(),
        MishapEffect::Mark { stat } => format!("a training mark of {}", stat.name()),
    }
}
