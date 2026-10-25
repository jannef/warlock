//! The birth signs' gifts (spec §8 "Birth sign"): what each animal, element, and lucky day gives,
//! with the exact rules the game says only in words, and how likely a lucky day is by race.

use super::talents::{effects_text, needs_text};
use super::text::{signed, table_text};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use warlock_core::character::BirthDate;
use warlock_core::content::{Content, Horoscope, Need, TalentDef};
use warlock_core::time::{DAYS_PER_MONTH, MONTHS_PER_YEAR};

/// The gifts of `grants` (talent ids), one a line in a cell: "**Fleet-footed**: speed +5 %", a flaw
/// marked; "—" for none.
fn gifts(content: &Content, grants: &[String]) -> String {
    let book = content.talents();
    let parts: Vec<String> = (grants.iter())
        .filter_map(|id| book.id(id).map(|t| book.get(t)))
        .map(|def| {
            let flaw = if def.flaw { " (flaw)" } else { "" };
            format!("**{}**{flaw}: {}", def.name, effects_text(content, def))
        })
        .collect();
    if parts.is_empty() {
        "—".into()
    } else {
        parts.join("<br>")
    }
}

/// Every animal with its gifts and flaw, the rules exact, and its nudge to the starting
/// alignment score (spec §8 "Alignment", #272).
pub(super) fn animals(content: &Content) -> String {
    let Some(h) = content.horoscope() else {
        return "There is no horoscope.\n".into();
    };
    let mut out = String::from("| | Animal | Gifts and flaw | Alignment |\n|---:|---|---|---:|\n");
    for (i, a) in h.animals.iter().enumerate() {
        let nudge = if a.alignment == 0 {
            "0".into()
        } else {
            signed(a.alignment)
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {nudge} |",
            i + 1,
            a.name,
            gifts(content, &a.grants)
        );
    }
    out
}

/// Every element with its gifts, the rules exact.
pub(super) fn elements(content: &Content) -> String {
    let Some(h) = content.horoscope() else {
        return "There is no horoscope.\n".into();
    };
    let mut out = String::from("| Element | Gifts |\n|---|---|\n");
    for e in &h.elements {
        let _ = writeln!(out, "| {} | {} |", e.name, gifts(content, &e.grants));
    }
    out
}

/// The sign-limited picks of the Stars tree (talents a level-up or creation may pick, only
/// for those born under their sign): by sign, the animals in the order of the Great Race and
/// then the elements in theirs, with what each needs besides and its rule exact.
pub(super) fn talents(content: &Content) -> String {
    let Some(h) = content.horoscope() else {
        return "There is no horoscope.\n".into();
    };
    let book = content.talents();
    let Some(stars) = book.tree_id("stars") else {
        return "There is no Stars tree.\n".into();
    };
    // The sign a pick asks for, as a sort key (the animals first) and a name.
    let sign = |def: &TalentDef| -> Option<(usize, String)> {
        def.needs_at(1).find_map(|need| match need {
            Need::Animal(id) => {
                (h.animals.iter().position(|a| a.id == id)).map(|i| (i, h.animals[i].name.clone()))
            }
            Need::Element(id) => (h.elements.iter().position(|e| e.id == id))
                .map(|i| (h.animals.len() + i, format!("{}-born", h.elements[i].name))),
            _ => None,
        })
    };
    let mut rows: Vec<((usize, String), &TalentDef)> = (book.of_tree(stars))
        .map(|(_, def)| def)
        .filter(|def| !def.granted)
        .filter_map(|def| sign(def).map(|s| (s, def)))
        .collect();
    rows.sort_by(|a, b| (a.0.0, &a.1.name).cmp(&(b.0.0, &b.1.name)));
    if rows.is_empty() {
        return "No sign has a pick yet.\n".into();
    }
    let mut out = String::from("| Sign | Pick | Needs | Effect |\n|---|---|---|---|\n");
    for ((_, name), def) in rows {
        let _ = writeln!(
            out,
            "| {name} | {} | {} | {} |",
            def.name,
            needs_text(content, def),
            effects_text(content, def)
        );
    }
    out
}

/// The calendar and moon flavour (`[flavour]`, #273): the birthday's healing, the birth month's
/// cooldowns, and the full moon's reaches, exactly.
pub(super) fn flavour(content: &Content) -> String {
    let Some(f) = content.flavour() else {
        return "Neither the calendar nor the moons touch the gifts.\n".into();
    };
    let light = |name: &str| {
        content
            .light_id(name)
            .map(|id| content.light(id).radius)
            .unwrap_or(0)
    };
    let mut out = String::from("| When | What changes |\n|---|---|\n");
    let _ = writeln!(
        out,
        "| Your birthday (its month and day, midnight to midnight) | hit points come back at {} % \
         of the pace (all your other healing modifiers apply first); at its dawn, or if a run \
         begins on it, you are told \"It is your birthday.\" |",
        f.birthday_healing
    );
    let _ = writeln!(
        out,
        "| Using a birth sign's active in your birth month | its cooldown (Graceless, Spent-eyed, \
         Burnt out, Drained after Fire breath, Hoarse) lasts {} % as long, rounded up to the \
         second; Ashen is no cooldown and is untouched |",
        f.birth_month_cooldown
    );
    let parts = warlock_core::content::PHASE_ONE / warlock_core::content::FULL_MOON_WITHIN;
    let example = (content.moons().first())
        .map(|m| {
            format!(
                ": {} hours each way for the {}, a cycle of {} days",
                m.period_days * 24 / parts,
                m.name,
                m.period_days
            )
        })
        .unwrap_or_default();
    let moon = format!(
        "| A full moon up (within 1/{parts} of its cycle of full, either side{example}; above the \
         horizon; the sun down; anywhere, underground and under clouds too) | Hound's nose \
         smells, the Transfixing stare reaches, and Eyes of the heights sees at night {} {} \
         farther",
        f.full_moon_reach,
        if f.full_moon_reach == 1 {
            "cell"
        } else {
            "cells"
        }
    );
    let inner = (f.full_moon_light)
        .map(|id| {
            format!(
                "; Inner light shines to radius {} (from {})",
                content.light(id).radius,
                light("ember")
            )
        })
        .unwrap_or_default();
    let _ = writeln!(out, "{moon}{inner} |");
    out
}

/// The years of the sign `animal`–`element` in the great cycle that holds `year` and the one
/// before it.
fn years_of(h: &Horoscope, animal: &str, element: &str, year: i64) -> Vec<i64> {
    let cycle = h.great_cycle() as i64;
    let current = year - (year - h.anchor_year).rem_euclid(cycle);
    ((current - cycle)..(current + cycle))
        .filter(|&y| {
            let sign = h.sign_of_year(y);
            sign.animal == animal && sign.element == element
        })
        .collect()
}

/// The lucky days: the sign, the day, the title, the extra gift, and their years; then, for the
/// showcase's start date, every race's chance of being born on one.
pub(super) fn lucky(content: &Content) -> String {
    let Some(h) = content.horoscope() else {
        return "There is no horoscope.\n".into();
    };
    let Some(start) = content.scenario("showcase").map(|s| s.start) else {
        return String::new();
    };
    let mut out = String::from(
        "| Sign | Day of the month | Title | Extra gift | Years (this great cycle and the last) \
         |\n|---|---:|---|---|---|\n",
    );
    for l in &h.lucky {
        let sign = warlock_core::content::Sign {
            animal: l.animal.clone(),
            element: l.element.clone(),
        };
        let years: Vec<String> = (years_of(h, &l.animal, &l.element, start.year() as i64))
            .iter()
            .map(i64::to_string)
            .collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            h.name(&sign),
            l.day,
            table_text(&l.title),
            gifts(content, &l.grants),
            years.join(", ")
        );
    }
    let _ = write!(
        out,
        "\nHow likely a lucky day is for each race, for a run that starts on {} {} {} (the \
         showcase's start; another start date moves it). Every age in the race's range and every \
         day of the year are equally likely.\n\n| Race | Ages | Lucky signs in reach | About 1 in \
         |\n|---|---|---|---:|\n",
        start.day(),
        content.calendar().month(start.month()),
        start.year()
    );
    let mut races: Vec<_> = content.races().list.iter().collect();
    races.sort_by(|a, b| a.name.cmp(&b.name));
    for race in races {
        let mut lucky = 0u64;
        let mut total = 0u64;
        let mut titles: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
        for age in race.age.clone() {
            for month in 1..=MONTHS_PER_YEAR as u32 {
                for day in 1..=DAYS_PER_MONTH as u32 {
                    total += 1;
                    let born = BirthDate::counted_back(start, age, month, day);
                    let sign = h.sign_of_year(born.year);
                    if let Some(l) = h.lucky(&sign, day) {
                        lucky += 1;
                        let years = titles.entry(l.title.as_str()).or_default();
                        if !years.contains(&born.year) {
                            years.push(born.year);
                        }
                    }
                }
            }
        }
        let odds = (total + lucky / 2)
            .checked_div(lucky)
            .map_or_else(|| "never".to_string(), |n| n.to_string());
        let reach = if titles.is_empty() {
            "none".to_string()
        } else {
            let parts: Vec<String> = (titles.iter_mut())
                .map(|(title, years)| {
                    years.sort();
                    let years: Vec<String> = years.iter().map(i64::to_string).collect();
                    format!("{title} ({})", years.join(", "))
                })
                .collect();
            parts.join("; ")
        };
        let _ = writeln!(
            out,
            "| {} | {} to {} | {reach} | {odds} |",
            race.name,
            race.age.start(),
            race.age.end()
        );
    }
    out
}
