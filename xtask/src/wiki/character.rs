//! Character creation (spec §8 "Character creation"): stats, genders, races, classes, and the
//! horoscope, with adjustments as written in data.

use super::util::{Term, anchored, damage_link, signed, table_text};
use super::weapon_skills;
use std::fmt::Write as _;
use warlock_core::character::Gender;
use warlock_core::content::Content;
use warlock_core::dice::Dice;
use warlock_core::object::format_weight;
use warlock_core::stats::{self, AVERAGE, PerStat, SCORES, Stat};
use warlock_core::{burden, combat};

/// The stats in the order the character screen lists them, with their abbreviations.
pub(super) fn stats(_content: &Content) -> String {
    let mut out = String::from("| Stat | Short |\n|---|:---:|\n");
    for stat in Stat::ALL {
        let name = anchored(Term::Stat, stat.name(), stat.name());
        let _ = writeln!(out, "| {name} | `{}` |", stat.abbrev());
    }
    let _ = writeln!(out, "\nA score of {AVERAGE} is average.");
    out
}

/// Every score's modifier (spec §8 "Stats": [`stats::modifier`], D&D 3.5's), scores of one
/// modifier on one row.
pub(super) fn stat_modifiers(_content: &Content) -> String {
    let mut out = String::from("| Score | Modifier |\n|:---:|:---:|\n");
    let mut rows: Vec<(u8, u8, i32)> = Vec::new();
    for score in SCORES {
        let modifier = stats::modifier(i32::from(score));
        match rows.last_mut() {
            Some((_, last, m)) if *m == modifier => *last = score,
            _ => rows.push((score, score, modifier)),
        }
    }
    for (first, last, modifier) in rows {
        let scores = if first == last {
            first.to_string()
        } else {
            format!("{first}–{last}")
        };
        let _ = writeln!(out, "| {scores} | {} |", signed(modifier));
    }
    out
}

/// What each stat feeds, by its modifier and by its score (spec §8 "Stats"), with the numbers of
/// the rules that read it: the carrying limit's grams a point, the regeneration pace, the shops'
/// Presence share, and the diseases each stat's checks resist.
pub(super) fn stat_uses(content: &Content) -> String {
    let checks = |stat: Stat| {
        let names: Vec<String> = (content.disease_defs().iter())
            .filter(|d| d.check == stat)
            .map(|d| d.name.clone())
            .collect();
        (!names.is_empty()).then(|| format!("checks against {}", names.join(", ")))
    };
    let carry = format!(
        "the carrying limit: {} × (2 × Strength + Toughness), by size ([Burden](burden.md))",
        format_weight(burden::GRAMS_PER_POINT)
    );
    let regen = format!(
        "a hit point back every max({}, {} − {} × Toughness) seconds",
        combat::REGEN_FASTEST,
        combat::REGEN_SLOWEST,
        combat::REGEN_PER_TOUGHNESS
    );
    let presence = content.shops().map(|s| {
        format!(
            "shop prices: {}% in your favour for each point ([Shops](shops.md))",
            s.presence
        )
    });
    #[allow(clippy::type_complexity)]
    let rows: [(Stat, Vec<Option<String>>, Vec<Option<String>>); 6] = [
        (
            Stat::Strength,
            vec![
                Some("to hit and damage up close; damage of throws".into()),
                checks(Stat::Strength),
            ],
            vec![
                Some(carry.clone()),
                Some("a bow's draw: −1 to hit and damage for each point short".into()),
                Some("kicking a door in: 1d(sturdiness) at or below it ([Doors](doors.md))".into()),
                Some("digging: tool power × (Strength + 10) a turn ([Digging](digging.md))".into()),
            ],
        ),
        (
            Stat::Dexterity,
            vec![
                Some("Defence (capped by armour and burden); to hit with shots and throws".into()),
                checks(Stat::Dexterity),
            ],
            vec![],
        ),
        (
            Stat::Toughness,
            vec![
                Some("the hit points each level adds ([Experience](experience.md))".into()),
                checks(Stat::Toughness),
            ],
            vec![
                Some("hit points: Toughness plus race and class".into()),
                Some(regen),
                Some(carry),
            ],
        ),
        (
            Stat::Intelligence,
            vec![
                Some("searching ([Searching](searching.md))".into()),
                checks(Stat::Intelligence),
            ],
            vec![],
        ),
        (
            Stat::Willpower,
            vec![
                Some("noticing hidden things in passing ([Searching](searching.md))".into()),
                checks(Stat::Willpower),
            ],
            vec![],
        ),
        (
            Stat::Presence,
            vec![presence, checks(Stat::Presence)],
            vec![],
        ),
    ];
    let mut out = String::from("| Stat | Its modifier | Its score |\n|---|---|---|\n");
    for (stat, modifier, score) in rows {
        let join = |parts: Vec<Option<String>>| {
            let parts: Vec<String> = parts.into_iter().flatten().collect();
            if parts.is_empty() {
                "-".to_string()
            } else {
                parts.join("; ")
            }
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} |",
            stat.name(),
            join(modifier),
            join(score)
        );
    }
    out
}

/// Each race's potentials (spec §8 "Character creation"): the races file's base potential plus
/// the race's adjustment, rolled once at the start of a run, kept within the scores' range; a
/// potential is never below its starting score.
pub(super) fn potentials(content: &Content) -> String {
    let (names, align) = stat_columns();
    let races = content.races();
    let mut out = format!("{}| Race |{names}\n|---|{align}\n", super::util::stat_key());
    let (lo, hi) = (i32::from(*SCORES.start()), i32::from(*SCORES.end()));
    let base = i32::from(races.base_potential);
    for r in &races.list {
        let cells: String = (r.potential.iter())
            .map(|(_, v)| {
                let (min, max) = (
                    (base + v.min()).clamp(lo, hi),
                    (base + v.max()).clamp(lo, hi),
                );
                if min == max {
                    format!(" {min} |")
                } else {
                    format!(" {min}–{max} |")
                }
            })
            .collect();
        let _ = writeln!(out, "| {} |{cells}", r.name);
    }
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
/// age, then what they are (their potentials are [`potentials`]).
pub(super) fn races(content: &Content) -> String {
    // Two tables, the stats and the rest, so neither is too wide to read (#1586).
    let (names, align) = stat_columns();
    let mut out = format!("{}| Race |{names}\n|---|{align}\n", super::util::stat_key());
    for r in &content.races().list {
        let _ = writeln!(
            out,
            "| {} |{}",
            anchored(Term::Race, &r.id, &r.name),
            stat_cells(&r.stats),
        );
    }
    out.push_str(
        "\n| Race | Vision | Starting age | Hit points | Weapon skills |\n\
         |---|---|---|:---:|---|\n",
    );
    for r in &content.races().list {
        let vision = content
            .vision(&r.vision)
            .map_or(r.vision.as_str(), |v| &v.name);
        let _ = writeln!(
            out,
            "| {} | {vision} | {}–{} | {} | {} |",
            r.name,
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

/// The races' gifts and flaws on other rules, by name: natural armour, resistances,
/// light-blindness, disease checks, and rotten food; a race with none of them is left out.
pub(super) fn race_traits(content: &Content) -> String {
    let mut out = String::from(
        "| Race | Natural armour | Resists | Strong light | Disease checks | Rotten food |\n\
         |---|---|---|---|:---:|---|\n",
    );
    let mut races: Vec<_> = content.races().list.iter().collect();
    races.sort_by(|a, b| a.name.cmp(&b.name));
    for r in races {
        let natural = r.natural.as_ref().map(|h| match h.defence {
            0 => format!("Protection {} ({})", h.protection, h.name),
            d => format!("Defence +{d}, Protection {} ({})", h.protection, h.name),
        });
        let resists: Vec<String> = (r.resist.iter())
            .map(|&(kind, percent)| format!("{} {percent} %", damage_link(content, kind)))
            .collect();
        let light = r.light_blind.map(|b| {
            format!(
                "dazzled from brightness {}: sees {} cells, to-hit {}",
                b.glare,
                b.sight,
                signed(b.to_hit)
            )
        });
        let disease = (r.disease_check != 0).then(|| signed(r.disease_check));
        let rotten = r.eats_rotten.then(|| "eaten unharmed".to_string());
        if natural.is_none()
            && resists.is_empty()
            && light.is_none()
            && disease.is_none()
            && rotten.is_none()
        {
            continue;
        }
        let dash = || "—".to_string();
        let resists = if resists.is_empty() {
            dash()
        } else {
            resists.join(", ")
        };
        let _ = writeln!(
            out,
            "| {} | {} | {resists} | {} | {} | {} |",
            r.name,
            natural.unwrap_or_else(dash),
            light.unwrap_or_else(dash),
            disease.unwrap_or_else(dash),
            rotten.unwrap_or_else(dash),
        );
    }
    out
}

/// The page of class `id`, its level table and its rules (#1540): `class-berserker.md`.
pub(super) fn class_page(id: &str) -> String {
    format!("class-{}.md", id.replace('_', "-"))
}

/// The classes in the wiki's order (fighters, casters, skulkers, everyday folk: #1536): their
/// stat adjustments, then what they are, each linking to its page.
pub(super) fn classes(content: &Content) -> String {
    // Two tables, the stats and the rest, so neither is too wide to read (#1586).
    let (names, align) = stat_columns();
    let mut out = format!(
        "{}| Class |{names}\n|---|{align}\n",
        super::util::stat_key()
    );
    let classes = super::class_guide::ordered(content);
    for c in &classes {
        let _ = writeln!(out, "| {} |{}", c.name, stat_cells(&c.stats));
    }
    out.push_str(
        "\n| Class | Hit points | Weapon skills | Fighting styles |\n|---|:---:|---|---|\n",
    );
    for c in &classes {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} |",
            c.name,
            c.hp,
            weapon_skills::class_ranks(content, c),
            weapon_skills::class_styles(content, c)
        );
    }
    out.push('\n');
    for c in &classes {
        let _ = writeln!(
            out,
            "- **[{}]({})**: {}",
            c.name,
            class_page(&c.id),
            c.description
        );
    }
    out
}

/// Which classes each race may take (spec §8 "Character creation", #1279): per race, by name, the
/// classes closed to it (`races_not`) and those open only with some genders (`only`); then the
/// classes New run never offers (`hidden`).
pub(super) fn class_limits(content: &Content) -> String {
    let mut races: Vec<_> = content.races().list.iter().collect();
    races.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out = String::from("| Race | Classes closed to them |\n|---|---|\n");
    for r in races {
        let mut closed: Vec<String> = Vec::new();
        for c in content.classes().iter().filter(|c| !c.hidden) {
            if c.races_not.contains(&r.id) {
                closed.push(c.name.clone());
            } else if let Some(only) = c.only.iter().find(|o| o.race == r.id) {
                let genders: Vec<String> = (only.genders.iter())
                    .map(|g| g.name().to_lowercase())
                    .collect();
                closed.push(format!("{} (unless {})", c.name, genders.join(" or ")));
            }
        }
        closed.sort();
        let closed = if closed.is_empty() {
            "—".to_string()
        } else {
            closed.join(", ")
        };
        let _ = writeln!(
            out,
            "| <a id=\"limits-{}\"></a>{} | {closed} |",
            r.id, r.name
        );
    }
    let mut hidden: Vec<&str> = (content.classes().iter())
        .filter(|c| c.hidden)
        .map(|c| c.name.as_str())
        .collect();
    hidden.sort();
    if !hidden.is_empty() {
        let _ = writeln!(
            out,
            "\nNot offered at character creation for now: {}.",
            hidden.join(", ")
        );
    }
    out
}

/// The prestige classes (spec §8 "Prestige classes", `data/prestige/`), in file order as the
/// classes are: how many levels each has, what each level adds to the hit points and mana (half
/// of it a level, as a class's), its skill rolls, the skills it is known for; then each one's
/// description.
pub(super) fn prestige_classes(content: &Content) -> String {
    let all = content.prestiges();
    if all.is_empty() {
        return "There are no prestige classes yet.\n".into();
    }
    let mut out = String::from(
        "| Prestige class | Levels | Hit points | Mana | Skill rolls a level | Known for |\n\
         |---|:---:|:---:|:---:|:---:|---|\n",
    );
    for p in all {
        let known: Vec<String> = (p.known_for.iter())
            .map(|&s| {
                let def = content.skill(s);
                format!(
                    "[{}](skills.md#skill-{})",
                    def.name,
                    def.id.replace('_', "-")
                )
            })
            .collect();
        let known = if known.is_empty() {
            "—".to_string()
        } else {
            known.join(", ")
        };
        let _ = writeln!(
            out,
            "| <a id=\"prestige-{}\"></a>{} | {} | {} | {} | {} | {known} |",
            p.id.replace('_', "-"),
            p.name,
            p.levels,
            p.hp,
            p.mana,
            p.rolls
        );
    }
    out.push('\n');
    for p in all {
        let _ = writeln!(out, "- **{}**: {}", p.name, p.description);
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
    let mut out = format!(
        "## Animals\n\n{}| | Animal |{names} Those born under it |\n|---:|---|{align}---|\n",
        super::util::stat_key()
    );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_prestige_tables_read_the_data() {
        let content = Content::builtin().expect("content");
        let classes = prestige_classes(&content);
        assert!(
            classes.contains("<a id=\"prestige-hunter\"></a>Hunter | 30 | 7 |"),
            "{classes}"
        );
        let levels = super::super::talents::prestige_levels(&content);
        assert!(levels.contains("### Hunter"), "{levels}");
        assert!(
            levels.contains("| 1 | <a id=\"talent-spirit-weapon\"></a>Spirit weapon |"),
            "{levels}"
        );
        let callings = super::super::dialogue::callings(&content);
        assert!(callings.contains("grants it when spoken to"), "{callings}");
        assert!(
            callings.contains("[strange spirit](creatures.md#creature-strange-spirit)"),
            "{callings}"
        );
        assert!(callings.contains("Kind or better"), "{callings}");
        assert!(callings.contains("encounter-table-roads"), "{callings}");
    }
}
