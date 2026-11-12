//! The professions' recipes (spec §8 "Skills": professions; docs/dev/crafting.md): every generic
//! recipe from `data/recipes/<skill>/`, and the grades a made item comes out at, from
//! `data/recipes.toml`'s bands and the 2d12 rules (`world/craft_make.rs::grade`).

use super::text::{Term, anchored, link};
use std::fmt::Write as _;
use warlock_core::combat::HIGHEST;
use warlock_core::content::{Content, PartSource, RecipeCount, counted};
use warlock_core::dice::Dice;
use warlock_core::time::Duration;
use warlock_core::world::{Grade, grade};

/// The bonuses (the stat modifier, the skill, and the talents together) the odds table has a row
/// for, and what reaches each.
const BONUSES: [(i32, &str); 4] = [
    (4, "Metalcraft 50, an average arm"),
    (8, "Metalcraft 100"),
    (12, "Metalcraft 200"),
    (16, "Metalcraft 200 and a strong arm"),
];

/// The recipe the odds are for: the long sword, of steel.
const ODDS_FOR: &str = "long_sword";

/// Every generic recipe, by profession (skill order), then by the skill it is listed from and
/// its name: what it makes and how many, what is chosen, the stock it takes in its base
/// material (a batch's at its most for an untrained maker), what else it uses, its station,
/// time, and difficulty.
pub(super) fn recipes(content: &Content) -> String {
    let recipes = content.recipes();
    let mut out = String::from(
        "| Recipe | Listed from | Makes | Chosen | Stock | Uses up | Ornament | Station | Time | \
         Difficulty |\n|---|:---:|:---:|---|---|---|:---:|---|---|:---:|\n",
    );
    let mut rows: Vec<_> = (recipes.list.iter())
        .filter(|r| r.result.is_some())
        .collect();
    rows.sort_by(|a, b| (a.skill, a.from, &a.name).cmp(&(b.skill, b.from, &b.name)));
    let fletch = content.fletch();
    for r in rows {
        let Some(result) = r.result.as_ref() else {
            continue;
        };
        let skill = content.skill(r.skill);
        let def = content.object(result.object);
        let (makes, most) = match result.count {
            RecipeCount::Dice(dice) if dice.is_fixed() => {
                (dice.max().to_string(), dice.max().max(1).unsigned_abs())
            }
            RecipeCount::Dice(dice) => (dice.to_string(), dice.max().max(1).unsigned_abs()),
            RecipeCount::Fletch => {
                let (dice, per) =
                    fletch.map_or((Dice::fixed(1), 1), |f| (f.count, f.value_per_count));
                (
                    format!("{dice} + {} ÷ {per}", skill.name),
                    dice.max().max(1).unsigned_abs(),
                )
            }
        };
        let mut chosen = Vec::new();
        let mut stock = Vec::new();
        for part in &r.parts {
            let base = &def.parts[part.part];
            let name = (base.part).map_or("the item".to_string(), |p| {
                content.part_defs()[usize::from(p.0)].name.clone()
            });
            chosen.push(format!("{name} ({})", source(content, &part.source)));
            if part.source != PartSource::Stock {
                continue;
            }
            let grams = u64::from(def.weight) * u64::from(base.share) / 100 * u64::from(most);
            let piece = content
                .object_id(warlock_core::piece::ID)
                .map(|o| content.object(o));
            let n = warlock_core::piece::count_for(content, grams, base.material);
            if let (Some(n), Some(piece)) = (n, piece) {
                let (one, many) = warlock_core::piece::names(content, piece, base.material);
                stock.push(counted(&one, &many, n));
            }
        }
        let uses: Vec<String> = (r.inputs.iter())
            .filter_map(|i| {
                let o = content.object(i.object()?);
                let words = counted(&o.name, &o.plural, i.count);
                Some(match i.per_made {
                    true => format!("{words} for each made"),
                    false => words,
                })
            })
            .collect();
        let mut station: Vec<String> = (r.station.iter())
            .map(|&t| {
                let t = content.terrain(t);
                link(Term::Terrain, &t.id, &t.name)
            })
            .collect();
        if r.knife {
            station.push("a knife".into());
        }
        if r.launcher {
            station.push("a launcher for it, ready".into());
        }
        let listed = match r.anyone {
            true => "anyone".to_string(),
            false => format!("{} {}", link(Term::Skill, &skill.id, &skill.name), r.from),
        };
        let or_dash = |list: Vec<String>| match list.is_empty() {
            true => "—".to_string(),
            false => list.join(", "),
        };
        let _ = writeln!(
            out,
            "| {} | {listed} | {makes} | {} | {} | {} | {} | {} | {} | {} |",
            anchored(Term::Recipe, &r.id, &r.name),
            or_dash(chosen),
            or_dash(stock),
            or_dash(uses),
            if r.ornament { "yes" } else { "no" },
            or_dash(station),
            Duration::seconds(u64::from(r.seconds)).spoken(),
            r.difficulty,
        );
    }
    out.push_str(
        "\nStock is counted in pieces of the base material (an ingot is 500 g of steel). Another \
         material takes as many pieces of its own: a mithril ingot is lighter, and so is the \
         blade made of it. A batch's stock is what the most it can make weighs, for a maker \
         untrained in the skill; a smaller batch takes less, and one piece always covers at \
         least one.\n",
    );
    out
}

/// A part's source in words.
fn source(content: &Content, source: &PartSource) -> String {
    match source {
        PartSource::Stock => "stock".into(),
        PartSource::Sources {
            input,
            stock,
            optional,
        } => {
            let mut from: Vec<String> = input
                .iter()
                .map(|&o| content.object(o).name.clone())
                .collect();
            from.extend(stock.iter().map(|&m| content.material(m).name.clone()));
            if *optional {
                from.push("none".into());
            }
            from.join(" or ")
        }
        PartSource::Gem => "a cut gem".into(),
    }
}

/// The grades by the check's margin with their words, then the odds of each for the long sword
/// at bonuses a smith can reach, then the materials whose `work` moves the difficulty.
pub(super) fn crafting_grades(content: &Content) -> String {
    let settings = content.recipes().settings.as_ref();
    let at = |g: Grade| (0..=40).find(|&m| grade(settings, 12, m) == g).unwrap_or(0);
    let (fine, marvellous, masterwork) = (
        at(Grade::Fine),
        at(Grade::Marvellous),
        at(Grade::Masterwork),
    );
    let mut out =
        String::from("| Margin | Grade | Improvements | What you hear |\n|:---:|---|:---:|---|\n");
    let rows = [
        (
            format!("{masterwork} or more"),
            "Masterwork",
            3,
            "You have made a Masterwork long sword!",
        ),
        (
            format!("{marvellous} to {}", masterwork - 1),
            "Marvellous",
            2,
            "This is a Marvellous long sword!",
        ),
        (
            format!("{fine} to {}", marvellous - 1),
            "Fine",
            1,
            "This is a Fine long sword.",
        ),
        (
            format!("0 to {}", fine - 1),
            "Plain",
            0,
            "You have made a long sword.",
        ),
        (
            "below 0".to_string(),
            "Failed",
            0,
            "The blade comes out brittle and you break it up for the metal.",
        ),
    ];
    for (margin, name, n, words) in rows {
        let _ = writeln!(out, "| {margin} | {name} | {n} | \"{words}\" |");
    }
    let yield_percent = content
        .recipes()
        .settings
        .as_ref()
        .map_or(50, |s| s.yield_percent);
    let _ = writeln!(
        out,
        "\nA natural 24 makes at least Fine; a natural 2 always fails. A failure gives back \
         {yield_percent}% of the stock (at least one piece of each) and keeps the gem whole.\n"
    );
    let recipe = (content.recipes().id(ODDS_FOR)).map(|id| content.recipe(id));
    let Some(recipe) = recipe else {
        return out;
    };
    let difficulty = recipe.difficulty;
    let _ = writeln!(
        out,
        "The odds for a steel {} (difficulty {difficulty}), by the check's whole bonus:\n",
        recipe.name.to_lowercase()
    );
    out.push_str(
        "| Bonus | Who | Failed | Plain | Fine | Marvellous | Masterwork |\n\
         |:---:|---|:---:|:---:|:---:|:---:|:---:|\n",
    );
    let sides = HIGHEST / 2;
    for (bonus, who) in BONUSES {
        let mut counts = [0u32; 5];
        for a in 1..=sides {
            for b in 1..=sides {
                let natural = a + b;
                counts[grade(settings, natural, natural + bonus - difficulty) as usize] += 1;
            }
        }
        let _ = write!(out, "| {bonus:+} | {who} |");
        for n in counts {
            let percent = f64::from(n) * 100.0 / f64::from(sides * sides);
            let _ = write!(out, " {percent:.0}% |");
        }
        out.push('\n');
    }
    let mut works: Vec<_> = (content.materials().iter())
        .filter(|m| m.group.as_deref() == Some("metal") && m.work != 0)
        .map(|m| (m.work, m.name.clone()))
        .collect();
    works.sort();
    if !works.is_empty() {
        let list: Vec<String> = (works.iter())
            .map(|(w, name)| format!("{name} {w:+}"))
            .collect();
        let _ = writeln!(
            out,
            "\nSome metals are harder or easier to work, and add to the difficulty: {}. Any \
             other metal adds nothing.",
            list.join(", ")
        );
    }
    out
}
