//! Character creation (spec §8 "Character creation"): stats, genders, races, classes, and the
//! horoscope, with adjustments as written in data.

use super::text::table_text;
use super::weapon_skills;
use std::fmt::Write as _;
use warlock_core::character::Gender;
use warlock_core::content::Content;
use warlock_core::dice::Dice;
use warlock_core::stats::{AVERAGE, PerStat, Stat};

/// The stats in the order the character screen lists them, with their abbreviations. Only the
/// names: never a modifier or a potential, nor anyone's scores.
pub(super) fn stats(_content: &Content) -> String {
    let mut out = String::from("| Stat | Short |\n|---|:---:|\n");
    for stat in Stat::ALL {
        let _ = writeln!(out, "| {} | `{}` |", stat.name(), stat.abbrev());
    }
    let _ = writeln!(out, "\nA score of {AVERAGE} is average.");
    out
}

/// What each gender raises (spec §8 "Character creation").
pub(super) fn genders(_content: &Content) -> String {
    let mut out = String::from("| Gender | Starting stats |\n|---|---|\n");
    for gender in Gender::ALL {
        let raises = match gender {
            Gender::Male => format!("{} +1", Stat::Strength.name()),
            Gender::Female => format!("{} +1", Stat::Dexterity.name()),
            Gender::Other => "+1 to one stat, chosen by fate".to_string(),
        };
        let _ = writeln!(out, "| {} | {raises} |", gender.name());
    }
    out
}

/// A stat adjustment in a table, as written in data: `+2`, `0`, `1d3+1`, and with a minus sign
/// (not a hyphen) `−1`, `−1d2`. Rules, not modifiers.
pub(super) fn adjustment(v: Dice) -> String {
    v.to_string().replace('-', "\u{2212}")
}

/// The stat columns' header: `| St | Dx | …` and its alignment row.
fn stat_columns() -> (String, String) {
    let names: Vec<String> = Stat::ALL
        .iter()
        .map(|s| format!(" {} |", s.abbrev()))
        .collect();
    (names.concat(), ":---:|".repeat(Stat::COUNT))
}

/// One adjustment per stat, as table cells.
fn stat_cells(by: &PerStat<Dice>) -> String {
    by.iter()
        .map(|(_, v)| format!(" {} |", adjustment(v)))
        .collect()
}

/// The races in the order New run offers them: their stat adjustments, vision, and starting
/// age, then what they are. Never their potentials.
pub(super) fn races(content: &Content) -> String {
    let (names, align) = stat_columns();
    let mut out = format!(
        "| Race |{names} Vision | Starting age | Hit points | Weapon skills |\n\
         |---|{align}---|---|:---:|---|\n"
    );
    for r in &content.races().list {
        let vision = content
            .vision(&r.vision)
            .map_or(r.vision.as_str(), |v| &v.name);
        let _ = writeln!(
            out,
            "| {} |{} {vision} | {}–{} | {} | {} |",
            r.name,
            stat_cells(&r.stats),
            r.age.start(),
            r.age.end(),
            r.hp,
            weapon_skills::ranks_text(content, &r.weapon_skills),
        );
    }
    out.push('\n');
    for r in &content.races().list {
        let _ = writeln!(out, "- **{}**: {}", r.name, r.description);
    }
    out
}

/// The classes in the order New run offers them: their stat adjustments, then what they are.
pub(super) fn classes(content: &Content) -> String {
    let (names, align) = stat_columns();
    let mut out = format!(
        "| Class |{names} Hit points | Weapon skills | Fighting styles |\n\
         |---|{align}:---:|---|---|\n"
    );
    for c in content.classes() {
        let _ = writeln!(
            out,
            "| {} |{} {} | {} | {} |",
            c.name,
            stat_cells(&c.stats),
            c.hp,
            weapon_skills::class_ranks(content, c),
            weapon_skills::class_styles(content, c)
        );
    }
    out.push('\n');
    for c in content.classes() {
        let _ = writeln!(out, "- **{}**: {}", c.name, c.description);
    }
    out
}

/// The horoscope: the animals in the order of the Great Race and the elements, each with its
/// stat adjustments and what it means; then the great cycle, sign by sign, with its years in the
/// great cycle of the showcase's start and the two before it.
pub(super) fn horoscope(content: &Content) -> String {
    let Some(h) = content.horoscope() else {
        return "There is no horoscope.\n".into();
    };
    let (names, align) = stat_columns();
    let mut out =
        format!("## Animals\n\n| | Animal |{names} Those born under it |\n|---:|---|{align}---|\n");
    for (i, a) in h.animals.iter().enumerate() {
        let _ = writeln!(
            out,
            "| {} | {} |{} {} |",
            i + 1,
            a.name,
            stat_cells(&a.stats),
            table_text(&a.description)
        );
    }
    let _ = write!(
        out,
        "\n## Elements\n\n| Element |{names} Those born under it |\n|---|{align}---|\n"
    );
    for e in &h.elements {
        let _ = writeln!(
            out,
            "| {} |{} {} |",
            e.name,
            stat_cells(&e.stats),
            table_text(&e.description)
        );
    }
    let cycle = h.great_cycle() as i64;
    let showcase = content
        .scenario("showcase")
        .map_or(1, |s| s.start.year() as i64);
    // The first year of the great cycle the showcase starts in.
    let current = showcase - (showcase - h.anchor_year).rem_euclid(cycle);
    let starts = [current - 2 * cycle, current - cycle, current];
    let _ = write!(
        out,
        "\n## Every sign\n\nEvery sign of the great cycle in order, with its years in the cycles \
         that began in {}, {}, and {}. Add or take away {cycle} years for any other cycle.\n\n\
         | | Sign | Years |\n|---:|---|---|\n",
        starts[0], starts[1], starts[2]
    );
    for i in 0..cycle {
        let sign = h.sign_of_year(current + i);
        let years: Vec<String> = starts.iter().map(|s| (s + i).to_string()).collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} |",
            i + 1,
            h.name(&sign),
            years.join(", ")
        );
    }
    out
}
