//! The professions' recipes (spec §8 "Skills": professions; docs/dev/crafting.md): every generic
//! recipe from `data/recipes/<skill>/`, the grades a made item comes out at, from
//! `data/recipes.toml`'s bands and the 2d12 rules (`world/craft_make.rs::grade`), and gem cutting
//! (`world/cut.rs`: the cut bands, a large uncut ruby's fates, the odds).

use super::text::{Term, anchored, capital, link};
use std::fmt::Write as _;
use warlock_core::combat::HIGHEST;
use warlock_core::content::{Content, CutId, Cutting, PartSource, RecipeUse, counted};
use warlock_core::time::Duration;
use warlock_core::world::{CutBand, Grade, cut_band, cut_outcome, grade};

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
/// its name: what it makes, what is chosen, the stock it takes in its base material, its
/// station, time, and difficulty.
pub(super) fn recipes(content: &Content) -> String {
    let recipes = content.recipes();
    let mut out = String::from(
        "| Recipe | Listed from | Chosen | Stock | Ornament | Station | Time | Difficulty |\n\
         |---|:---:|---|---|:---:|---|---|:---:|\n",
    );
    let mut rows: Vec<_> = (recipes.list.iter())
        .filter(|r| r.result.is_some())
        .collect();
    rows.sort_by(|a, b| (a.skill, a.from, &a.name).cmp(&(b.skill, b.from, &b.name)));
    for r in rows {
        let Some(result) = r.result.as_ref() else {
            continue;
        };
        let def = content.object(result.object);
        let mut chosen = Vec::new();
        let mut stock = Vec::new();
        for part in &r.parts {
            let base = &def.parts[part.part];
            let name = (base.part).map_or("the weapon".to_string(), |p| {
                content.part_defs()[usize::from(p.0)].name.clone()
            });
            chosen.push(format!("{name} ({})", source(content, &part.source)));
            let grams = u64::from(def.weight) * u64::from(base.share) / 100;
            let piece = content
                .object_id(warlock_core::piece::ID)
                .map(|o| content.object(o));
            let n = warlock_core::piece::count_for(content, grams, base.material);
            if let (Some(n), Some(piece)) = (n, piece) {
                let (one, many) = warlock_core::piece::names(content, piece, base.material);
                stock.push(counted(&one, &many, n));
            }
        }
        let station = (r.station).map_or("—".to_string(), |t| {
            let t = content.terrain(t);
            link(Term::Terrain, &t.id, &t.name)
        });
        let skill = content.skill(r.skill);
        let _ = writeln!(
            out,
            "| {} | {} {} | {} | {} | {} | {station} | {} | {} |",
            anchored(Term::Recipe, &r.id, &r.name),
            link(Term::Skill, &skill.id, &skill.name),
            r.from,
            chosen.join(", "),
            stock.join(", "),
            if r.ornament { "yes" } else { "no" },
            Duration::seconds(u64::from(r.seconds)).spoken(),
            r.difficulty,
        );
    }
    out.push_str(
        "\nStock is counted in pieces of the base material (an ingot is 500 g of steel). Another \
         material takes as many pieces of its own: a mithril ingot is lighter, and so is the \
         blade made of it.\n",
    );
    out
}

/// A part's source in words.
fn source(content: &Content, source: &PartSource) -> String {
    match source {
        PartSource::Stock => "stock".into(),
        PartSource::Sources { input, stock, .. } => {
            let mut from: Vec<String> = input
                .iter()
                .map(|&o| content.object(o).name.clone())
                .collect();
            from.extend(stock.iter().map(|&m| content.material(m).name.clone()));
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

/// The gem the cutting tables follow, and the size it starts at: a large uncut ruby.
const CUT_GEM: (&str, &str) = ("ruby", "large");

/// The whole bonuses the cutting odds have a row for, and who reaches each.
const CUT_BONUSES: [(i32, &str); 4] = [
    (4, "an apprentice"),
    (8, "Jewelcraft 100"),
    (12, "Jewelcraft 200"),
    (16, "a master with a steady hand"),
];

/// Gem cutting (Jewelcraft's Cut, `world/cut.rs`): the cut each margin reaches and what it does
/// to the size, a large uncut ruby at each outcome with its worth, then the odds by bonus on a
/// ruby and on a smokestone.
pub(super) fn gem_cutting(content: &Content) -> String {
    let (Some(gems), Some(recipe)) = (content.gems(), content.recipes().of_use(RecipeUse::Cut))
    else {
        return String::new();
    };
    let step = (content.recipes().settings.as_ref()).map_or(5, |s| s.cut_step);
    let cuts = gems.cuts();
    let first = cuts.iter().position(|c| !c.rough).unwrap_or(0);
    let name = |i: usize| capital(&cuts[i].name);
    let mut out =
        String::from("| Margin | Cut | On an uncut gem | On a cut gem |\n|:---:|---|---|---|\n");
    let _ = writeln!(
        out,
        "| below 0 | — | stays uncut, one size smaller | keeps its cut, one size smaller |"
    );
    for i in first..cuts.len() {
        let low = (i - first) as i32 * step;
        let margin = match i + 1 == cuts.len() {
            true => format!("{low} or more"),
            false => format!("{low} to {}", low + step - 1),
        };
        let uncut = match i == first {
            true => "one size smaller",
            false => "size kept",
        };
        let _ = writeln!(
            out,
            "| {margin} | {} | {uncut} | the better cut, one size smaller |",
            name(i)
        );
    }
    let plain = capital(&gems.cut(gems.plain().cut).name);
    let _ = writeln!(
        out,
        "\nA natural 2 shatters the gem, whatever the margin. A natural 24 cuts it at least \
         {}. Cutting takes {} against a difficulty of {} plus the gem's work.\n",
        plain.to_lowercase(),
        Duration::seconds(u64::from(recipe.seconds)).spoken(),
        recipe.difficulty,
    );

    // A large uncut ruby at each outcome.
    let (gem_id, size_id) = CUT_GEM;
    let (Some(material), Some(size)) = (content.material_id(gem_id), gems.size_id(size_id)) else {
        return out;
    };
    let def = content.material(material);
    let Some(gem) = def.gem.as_ref() else {
        return out;
    };
    let start = Cutting {
        size,
        cut: warlock_core::content::CutId(0),
    };
    let words = |c: Cutting| {
        format!(
            "{} {} {}",
            gems.size(c.size).name,
            gems.cut(c.cut).name,
            def.name
        )
    };
    let _ = writeln!(
        out,
        "A {} (worth {} gold) at each outcome:\n",
        words(start),
        gems.value(gem, start)
    );
    out.push_str("| Outcome | Becomes | Worth |\n|---|---|---:|\n");
    let mut bands = vec![
        ("Shattered", CutBand::Shattered),
        ("Failed", CutBand::Failed),
    ];
    for (i, cut) in cuts.iter().enumerate().skip(first) {
        bands.push((cut.name.as_str(), CutBand::Cut(CutId(i as u8))));
    }
    for (label, band) in &bands {
        let (becomes, worth) = match cut_outcome(gems, start, *band) {
            Some(c) => (format!("a {}", words(c)), gems.value(gem, c).to_string()),
            None => ("nothing".to_string(), "0".to_string()),
        };
        let _ = writeln!(out, "| {} | {becomes} | {worth} |", capital(label));
    }
    let crude = Cutting {
        size,
        cut: CutId(first as u8),
    };
    let best = CutId((cuts.len() - 1) as u8);
    if let Some(recut) = cut_outcome(gems, crude, CutBand::Cut(best)) {
        let _ = writeln!(
            out,
            "\nRe-cut at best, a {} ({} gold) becomes a {} ({} gold).\n",
            words(crude),
            gems.value(gem, crude),
            words(recut),
            gems.value(gem, recut),
        );
    }

    // The odds, by the check's whole bonus.
    out.push_str("| Gem | Bonus | Who | Shattered | Failed |");
    for c in &cuts[first..] {
        let _ = write!(out, " {} |", capital(&c.name));
    }
    out.push_str("\n|---|:---:|---|:---:|:---:|");
    for _ in first..cuts.len() {
        out.push_str(":---:|");
    }
    out.push('\n');
    let soft = (content.materials().iter())
        .filter(|m| m.gem.is_some())
        .min_by_key(|m| (m.work, m.gem.as_ref().map_or(0, |g| g.value)));
    let mut rows: Vec<(&str, i32, i32, &str)> = (CUT_BONUSES.iter())
        .map(|&(bonus, who)| (def.name.as_str(), def.work, bonus, who))
        .collect();
    if let Some(soft) = soft.filter(|m| m.work != def.work) {
        let (bonus, who) = CUT_BONUSES[CUT_BONUSES.len() - 1];
        rows.push((soft.name.as_str(), soft.work, bonus, who));
    }
    let sides = HIGHEST / 2;
    for (gem_name, work, bonus, who) in rows {
        let mut counts = vec![0u32; cuts.len() + 2];
        for a in 1..=sides {
            for b in 1..=sides {
                let natural = a + b;
                let margin = natural + bonus - (recipe.difficulty + work);
                let i = match cut_band(gems, step, natural, margin) {
                    CutBand::Shattered => 0,
                    CutBand::Failed => 1,
                    CutBand::Cut(c) => 2 + usize::from(c.0),
                };
                counts[i] += 1;
            }
        }
        let _ = write!(
            out,
            "| {} ({work:+}) | {bonus:+} | {who} |",
            capital(gem_name)
        );
        for i in (0..2).chain(2 + first..2 + cuts.len()) {
            let percent = f64::from(counts[i]) * 100.0 / f64::from(sides * sides);
            let _ = write!(out, " {percent:.0}% |");
        }
        out.push('\n');
    }
    out
}
