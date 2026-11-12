//! The professions' recipes (spec §8 "Skills": professions; docs/dev/crafting.md): every generic
//! recipe from `data/recipes/<skill>/`, a table a profession, and the grades a made item comes out at, from
//! `data/recipes.toml`'s bands and the 2d12 rules (`world/craft_make.rs::grade`).

use super::text::{Term, anchored, link};
use std::fmt::Write as _;
use warlock_core::combat::HIGHEST;
use warlock_core::content::{Content, InputKind, MaterialId, PartSource, RecipeInput, counted};
use warlock_core::skills::Effect;
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

/// The skill values the recipe tables give the odds of a made item at.
const ODDS_AT: [u16; 3] = [50, 100, 200];

/// Metalcraft's generic recipes ([`recipes_of`]).
pub(super) fn recipes_metalcraft(content: &Content) -> String {
    recipes_of(content, "metalcraft")
}

/// Woodcraft's generic recipes ([`recipes_of`]).
pub(super) fn recipes_woodcraft(content: &Content) -> String {
    recipes_of(content, "woodcraft")
}

/// Every generic recipe of profession `skill`, by the skill it is listed from and its name: what
/// it makes, what is chosen (each part and what it may be of), the stock it takes in its base
/// material (or the first it may be of, once a group), the ornament, what it needs (a station, a
/// knife), the time, the difficulty, and the chance it comes out made (not failed) at three skill
/// values with an average stat (modifier 0).
pub(super) fn recipes_of(content: &Content, skill: &str) -> String {
    let Some(skill_id) = content.skill_id(skill) else {
        return String::new();
    };
    let recipes = content.recipes();
    let skill_def = content.skill(skill_id);
    let craft = (skill_def.benefits.iter()).find(|b| b.effect == Effect::Craft);
    let at = ODDS_AT.map(|v| format!("{v}"));
    let mut out = format!(
        "| Recipe | Listed from | Chosen | Stock | Ornament | Needs | Time | Difficulty | Made at \
         {} {} |\n|---|:---:|---|---|:---:|---|---|:---:|:---:|\n",
        skill_def.name,
        at.join(" / ")
    );
    let mut rows: Vec<_> = (recipes.list.iter())
        .filter(|r| r.result.is_some() && r.skill == skill_id)
        .collect();
    rows.sort_by(|a, b| (a.from, &a.name, &a.id).cmp(&(b.from, &b.name, &b.id)));
    let piece = (content.object_id(warlock_core::piece::ID)).map(|o| content.object(o));
    for r in rows {
        let Some(result) = r.result.as_ref() else {
            continue;
        };
        let def = content.object(result.object);
        let mut chosen = Vec::new();
        let mut stock = Vec::new();
        let mut work = 0;
        for (n, part) in r.parts.iter().enumerate() {
            let base = &def.parts[part.part];
            // A lute or a pipe is no weapon: it is held in no slot.
            let whole = if def.slot.is_some() {
                "the weapon"
            } else {
                "the item"
            };
            let name = (base.part).map_or(whole.to_string(), |p| {
                content.part_defs()[usize::from(p.0)].name.clone()
            });
            let materials = materials(content, r.skill, def, part);
            chosen.push(format!(
                "{name}: {}",
                of_words(content, def, part.part, &materials)
            ));
            // Priced in the base material where the part may be of it, else the first, once a
            // group: another material of the group takes as many of its own.
            let group = |m: MaterialId| content.material(m).group.clone();
            let mut order = materials.clone();
            order.sort_by_key(|&m| (group(m) != group(base.material), content.material(m).value));
            let mut groups = Vec::new();
            let mut words = Vec::new();
            for &first in &order {
                if groups.contains(&group(first)) {
                    continue;
                }
                groups.push(group(first));
                let m = (order.iter().copied())
                    .find(|&m| m == base.material && group(m) == group(first))
                    .unwrap_or(first);
                if n == 0 && m == base.material {
                    work = content.material(m).work;
                }
                let grams = u64::from(def.weight) * u64::from(base.share) / 100;
                let count = warlock_core::piece::count_for(content, grams, base.material);
                if let (Some(count), Some(piece)) = (count, piece) {
                    let (one, many) = warlock_core::piece::names(content, piece, m);
                    words.push(counted(&one, &many, count));
                }
            }
            stock.push(words.join(" or "));
        }
        // What is used up besides: a bow's sinew, a crossbow's ingot of any metal.
        for input in &r.inputs {
            stock.extend(input_words(content, piece, input));
        }
        let mut needs = Vec::new();
        if let Some(t) = r.station {
            let t = content.terrain(t);
            needs.push(link(Term::Terrain, &t.id, &t.name));
        }
        if r.knife {
            needs.push("a knife".to_string());
        }
        let difficulty = match work {
            0 => r.difficulty.to_string(),
            w => format!("{} ({w:+} work)", r.difficulty),
        };
        let odds: Vec<String> = (ODDS_AT.iter())
            .map(|&v| {
                let bonus = craft.map_or(0, |b| b.at(v));
                format!("{:.0}%", made_chance(content, bonus - r.difficulty - work))
            })
            .collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} ({} s) | {difficulty} | {} |",
            anchored(Term::Recipe, &r.id, &r.name),
            r.from,
            chosen.join("; "),
            stock.join("; "),
            if r.ornament { "yes" } else { "no" },
            if needs.is_empty() {
                "—".to_string()
            } else {
                needs.join(", ")
            },
            Duration::seconds(u64::from(r.seconds)).spoken(),
            r.seconds,
            odds.join(" / "),
        );
    }
    out.push_str(
        "\nStock is counted in pieces of the base material (an ingot is 500 g of steel). Another \
         material takes as many pieces of its own: a mithril ingot is lighter, and so is the \
         blade made of it. The difficulty adds the base material's work where it has one; another \
         material adds its own instead (see the table of work above). The last column is the \
         chance the work comes out at all, with an average stat.\n",
    );
    out
}

/// An input in words: "a length of sinew", "an ingot of any metal", "20 arrowheads".
fn input_words(
    content: &Content,
    piece: Option<&warlock_core::content::ObjectDef>,
    input: &RecipeInput,
) -> Option<String> {
    let (one, many) = match &input.kind {
        InputKind::Object(o) => {
            let def = content.object(*o);
            (def.name.clone(), def.plural.clone())
        }
        InputKind::Material(m) => warlock_core::piece::names(content, piece?, *m),
        InputKind::Group(group) => {
            let first = (content.materials().iter())
                .position(|m| m.group.as_deref() == Some(group.as_str()))?;
            let first = MaterialId(u16::try_from(first).ok()?);
            let (name, plural, _) = content.stock().form_of(content, first)?;
            (
                format!("{name} of any {group}"),
                format!("{plural} of any {group}"),
            )
        }
    };
    Some(counted(&one, &many, input.count))
}

/// The materials part `part` of `def` may be made of as the recipe takes it: a `"stock"` part
/// what profession `skill` works, a table's what it lists.
fn materials(
    content: &Content,
    skill: warlock_core::skills::SkillId,
    def: &warlock_core::content::ObjectDef,
    part: &warlock_core::content::RecipePart,
) -> Vec<MaterialId> {
    match &part.source {
        PartSource::Sources { stock, .. } => stock.clone(),
        _ => (def.made_of(part.part).iter().copied())
            .filter(|&m| {
                (content.stock().profession_of(content, m)).is_some_and(|p| p.skill == skill)
            })
            .collect(),
    }
}

/// What a part may be of, in words: a group whose every material on the part's list is there
/// by the group's name, the rest by their own ("metal", "bone or stone", "ironwood").
fn of_words(
    content: &Content,
    def: &warlock_core::content::ObjectDef,
    part: usize,
    materials: &[MaterialId],
) -> String {
    let mut words: Vec<String> = Vec::new();
    for &m in materials {
        let mat = content.material(m);
        let whole = (mat.group.as_ref()).filter(|g| {
            (def.made_of(part).iter())
                .filter(|&&o| content.material(o).group.as_ref() == Some(*g))
                .all(|o| materials.contains(o))
        });
        let word = whole.cloned().unwrap_or_else(|| mat.name.to_lowercase());
        if !words.contains(&word) {
            words.push(word);
        }
    }
    words.join(" or ")
}

/// The chance in percent that a check of 2d12 plus `edge` (the bonus less the difficulty) comes
/// out made: a natural 2 always fails, a natural 24 always makes.
fn made_chance(content: &Content, edge: i32) -> f64 {
    let settings = content.recipes().settings.as_ref();
    let sides = HIGHEST / 2;
    let mut made = 0;
    for a in 1..=sides {
        for b in 1..=sides {
            let natural = a + b;
            made += i32::from(grade(settings, natural, natural + edge) != Grade::Failed);
        }
    }
    f64::from(made) * 100.0 / f64::from(sides * sides)
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
