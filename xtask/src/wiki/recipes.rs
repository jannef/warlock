//! The professions' recipes (spec §8 "Skills": professions; docs/dev/crafting.md): every generic
//! recipe from `data/recipes/<skill>/`, a table a profession, the grades a made item comes out at,
//! from `data/recipes.toml`'s bands and the 2d12 rules (`world/craft_make.rs::grade`), and gem
//! cutting (`world/cut.rs`: the cut bands, a large uncut ruby's fates, the odds), and gem setting
//! (`world/set.rs`: the odds).

use super::text::{Term, anchored, capital, link};
use std::fmt::Write as _;
use warlock_core::combat::HIGHEST;
use warlock_core::content::{
    Content, CutId, Cutting, InputKind, MaterialId, PartSource, RecipeCount, RecipeInput,
    RecipeUse, counted,
};
use warlock_core::dice::Dice;
use warlock_core::skills::Effect;
use warlock_core::time::Duration;
use warlock_core::world::{CutBand, Grade, SetBand, cut_band, cut_outcome, grade, set_band};

/// The bonuses (the stat modifier, the skill, and the talents together) the odds table has a row
/// for, and what reaches each.
const BONUSES: [(i32, &str); 4] = [
    (4, "Metalcraft 50, an average arm"),
    (8, "Metalcraft 100"),
    (12, "Metalcraft 200"),
    (16, "Metalcraft 200 and a strong arm"),
];

/// What Master's hand adds to a made check's margin as it is graded (the craftsperson's level-50
/// passive, `world/mastery.rs::graded_margin`): its `grade_ease`, 0 without the talent.
fn grade_ease(content: &Content) -> i32 {
    (content.talents().id("masters_hand"))
        .map(|id| content.talents().get(id))
        .and_then(|t| {
            t.effects.iter().find_map(|e| match e {
                warlock_core::content::TalentEffect::GradeEase { value } => Some(*value),
                _ => None,
            })
        })
        .unwrap_or(0)
}

/// `margin` as Master's hand grades it: `ease` added to a made check, never to a failure.
fn eased(margin: i32, ease: i32) -> i32 {
    match margin >= 0 {
        true => margin + ease,
        false => margin,
    }
}

/// The percentages of `counts` (each of `total`) from `from` on, joined by " / ".
fn percents(counts: &[u32], from: usize, total: u32) -> String {
    let parts: Vec<String> = (counts[from..].iter())
        .map(|&n| format!("{:.0}%", f64::from(n) * 100.0 / f64::from(total)))
        .collect();
    parts.join(" / ")
}

/// The recipe the odds are for: the long sword, of steel.
const ODDS_FOR: &str = "long_sword";

/// The skill values the recipe tables give the odds of a made item at.
const ODDS_AT: [u16; 3] = [50, 100, 200];

/// Metalcraft's generic recipes ([`recipes_of`]).
pub(super) fn recipes_metalcraft(content: &Content) -> String {
    recipes_of(content, "metalcraft")
}

/// Jewelcraft's generic recipes, the jewellery ([`recipes_of`]).
pub(super) fn recipes_jewelcraft(content: &Content) -> String {
    recipes_of(content, "jewelcraft")
}

/// Woodcraft's generic recipes ([`recipes_of`]).
pub(super) fn recipes_woodcraft(content: &Content) -> String {
    recipes_of(content, "woodcraft")
}

/// Every generic recipe of profession `skill`, by the skill it is listed from and its name: what
/// it makes and how many, what is chosen (each part and what it may be of), the stock it takes in
/// its base material (else steel, the piece's own, else the first it may be of, once a group; a
/// batch's at its most for an untrained maker), what else it uses up, the ornament, what it needs
/// (a station, a knife, a launcher), the time, the difficulty, and the chance it comes out made
/// (not failed; a batch whole) at three skill values with an average stat (modifier 0).
pub(super) fn recipes_of(content: &Content, skill: &str) -> String {
    let Some(skill_id) = content.skill_id(skill) else {
        return String::new();
    };
    let recipes = content.recipes();
    let skill_def = content.skill(skill_id);
    let craft = (skill_def.benefits.iter()).find(|b| b.effect == Effect::Craft);
    let at = ODDS_AT.map(|v| format!("{v}"));
    let mut out = format!(
        "| Recipe | Listed from | Makes | Chosen | Stock | Uses up | Ornament | Needs | Time | \
         Difficulty | Made at {} {} |\n|---|:---:|:---:|---|---|---|:---:|---|---|:---:|:---:|\n",
        skill_def.name,
        at.join(" / ")
    );
    let mut rows: Vec<_> = (recipes.list.iter())
        .filter(|r| r.result.is_some() && r.skill == skill_id)
        .collect();
    rows.sort_by(|a, b| (a.from, &a.name, &a.id).cmp(&(b.from, &b.name, &b.id)));
    let piece = (content.object_id(warlock_core::piece::ID)).map(|o| content.object(o));
    let fletch = content.fletch();
    for r in rows {
        let Some(result) = r.result.as_ref() else {
            continue;
        };
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
                    format!("{dice} + {} ÷ {per}", skill_def.name),
                    dice.max().max(1).unsigned_abs(),
                )
            }
        };
        let mut chosen = Vec::new();
        let mut stock = Vec::new();
        let mut work = 0;
        for (n, part) in r.parts.iter().enumerate() {
            let base = &def.parts[part.part];
            let whole = match def.attack.is_some() {
                true => "the weapon",
                false => "the item",
            };
            let name = (base.part).map_or(whole.to_string(), |p| {
                content.part_defs()[usize::from(p.0)].name.clone()
            });
            if part.source == PartSource::Gem {
                chosen.push(format!("{name}: a cut gem"));
                stock.push("a cut gem".to_string());
                continue;
            }
            let materials = materials(content, r.skill, def, part);
            let mut of = of_words(content, def, part.part, &materials);
            // A head of arrowheads or stock, or none: only the stock is priced, a piece a batch.
            if let PartSource::Sources {
                input, optional, ..
            } = &part.source
                && (input.is_some() || *optional)
            {
                if let Some(o) = input {
                    of = format!("{} or {of}", content.object(*o).plural);
                }
                if *optional {
                    of.push_str(" or none");
                }
                chosen.push(format!("{name}: {of}"));
                continue;
            }
            chosen.push(format!("{name}: {of}"));
            // Priced in the base material where the part may be of it, else the first, once a
            // group: another material of the group takes as many of its own.
            let group = |m: MaterialId| content.material(m).group.clone();
            let mut order = materials.clone();
            order.sort_by_key(|&m| group(m) != group(base.material));
            let mut groups = Vec::new();
            let mut words = Vec::new();
            for &first in &order {
                if groups.contains(&group(first)) {
                    continue;
                }
                groups.push(group(first));
                // The base material, else the piece's own (steel), else the group's cheapest.
                let named = |want: MaterialId| {
                    (order.iter().copied()).find(|&m| m == want && group(m) == group(first))
                };
                let m = (named(base.material))
                    .or_else(|| {
                        piece
                            .and_then(|p| p.parts.first())
                            .and_then(|p| named(p.material))
                    })
                    .or_else(|| {
                        (order.iter().copied())
                            .filter(|&m| group(m) == group(first))
                            .min_by_key(|&m| content.material(m).value)
                    })
                    .unwrap_or(first);
                if n == 0 && m == base.material {
                    work = content.material(m).work;
                }
                // The part's grams in `m`, as the made item weighs it (a wooden shield's weight
                // in metal, a leather cap's in steel), for a batch at its most.
                let weight = |m: MaterialId| u64::from(content.material(m).weight).max(1);
                let grams = u64::from(def.weight) * u64::from(base.share) * weight(m)
                    / weight(base.material)
                    / 100;
                let count = warlock_core::piece::count_for(content, grams * u64::from(most), m);
                if let (Some(count), Some(piece)) = (count, piece) {
                    let (one, many) = warlock_core::piece::names(content, piece, m);
                    words.push(counted(&one, &many, count));
                }
            }
            stock.push(words.join(" or "));
        }
        // What is used up besides: arrowheads, a bow's sinew, a crossbow's ingot of any metal.
        let uses: Vec<String> = (r.inputs.iter())
            .filter_map(|i| {
                let words = input_words(content, piece, i)?;
                Some(match i.per_made {
                    true => format!("{words} for each made"),
                    false => words,
                })
            })
            .collect();
        let mut needs = Vec::new();
        if let Some(t) = r.station {
            let t = content.terrain(t);
            needs.push(link(Term::Terrain, &t.id, &t.name));
        }
        if r.knife {
            needs.push("a knife".to_string());
        }
        if r.launcher {
            needs.push("a launcher for it, ready".to_string());
        }
        let or_dash = |list: &[String], by: &str| match list.is_empty() {
            true => "—".to_string(),
            false => list.join(by),
        };
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
            "| {} | {} | {makes} | {} | {} | {} | {} | {} | {} ({} s) | {difficulty} | {} |",
            anchored(Term::Recipe, &r.id, &r.name),
            r.from,
            or_dash(&chosen, "; "),
            or_dash(&stock, "; "),
            or_dash(&uses, ", "),
            if r.ornament { "yes" } else { "no" },
            or_dash(&needs, ", "),
            Duration::seconds(u64::from(r.seconds)).spoken(),
            r.seconds,
            odds.join(" / "),
        );
    }
    out.push_str(
        "\nStock is counted in pieces of the base material (an ingot is 500 g of steel). Another \
         material takes as many pieces of its own: a mithril ingot is lighter, and so is the \
         blade made of it. The difficulty adds the base material's work where it has one; another \
         material adds its own instead (see the table of work above). A batch's stock is what the \
         most it can make weighs, for a maker untrained in the skill; a smaller batch takes less, \
         and one piece always covers at least one. The last column is the chance the work comes \
         out at all (for a batch, all of it), with an average stat.\n",
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

/// The Alchemy values the brewing odds have a column for.
const BREWERS: [u16; 3] = [20, 50, 100];

/// Alchemy's recipes (the brews: no part, no ornament), by the value they are listed from and
/// their name: the potion, what it takes (herbs, essences, a flask), time, difficulty, and the
/// chance of a potion (not a murky draught or a brewer's blunder) at three Alchemy values with an
/// average Intelligence.
pub(super) fn brews(content: &Content) -> String {
    let Some(alchemy) = content.skill_id("alchemy") else {
        return String::new();
    };
    let craft = (content.skill(alchemy).benefits.iter())
        .find(|b| b.effect == warlock_core::skills::Effect::Craft);
    let bonus = |value: u16| craft.map_or(0, |b| b.at(value));
    let settings = content.recipes().settings.as_ref();
    let mut out = String::from("| Potion | Takes | Listed from | Time | Difficulty |");
    for value in BREWERS {
        let _ = write!(out, " Alchemy {value} |");
    }
    out.push_str("\n|---|---|:---:|---|:---:|");
    out.push_str(&":---:|".repeat(BREWERS.len()));
    out.push('\n');
    let mut rows: Vec<_> = (content.recipes().of(alchemy))
        .map(|(_, r)| r)
        .filter(|r| r.result.is_some())
        .collect();
    rows.sort_by(|a, b| (a.from, &a.name).cmp(&(b.from, &b.name)));
    let sides = HIGHEST / 2;
    for r in rows {
        let Some(result) = r.result.as_ref() else {
            continue;
        };
        let potion = content.object(result.object);
        let herbs: Vec<String> = (r.inputs.iter())
            .filter_map(|i| {
                let object = i.object()?;
                let def = content.object(object);
                let name = counted(&def.name, &def.plural, i.count);
                Some(brew_input(content, object, &def.id, &name))
            })
            .collect();
        let _ = write!(
            out,
            "| {} | {} | {} | {} | {} |",
            anchored(Term::Brew, &r.id, &link(Term::Potion, &potion.id, &r.name)),
            herbs.join(", "),
            r.from,
            Duration::seconds(u64::from(r.seconds)).spoken(),
            r.difficulty,
        );
        for value in BREWERS {
            if value < r.from {
                out.push_str(" — |");
                continue;
            }
            let mut made = 0;
            for a in 1..=sides {
                for b in 1..=sides {
                    let natural = a + b;
                    let margin = natural + bonus(value) - r.difficulty;
                    if grade(settings, natural, margin) != Grade::Failed {
                        made += 1;
                    }
                }
            }
            let percent = f64::from(made) * 100.0 / f64::from(sides * sides);
            let _ = write!(out, " {percent:.0}% |");
        }
        out.push('\n');
    }
    out
}

/// What a brew takes, linked where it is told of: a herb to its row, a flask (oil, tar) to its
/// potion row, an essence to the materials page's essences.
fn brew_input(
    content: &Content,
    object: warlock_core::content::ObjectId,
    id: &str,
    name: &str,
) -> String {
    let herb = (content.herbs()).is_some_and(|h| h.herbs.iter().any(|h| h.object == object));
    if herb {
        link(Term::Herb, id, name)
    } else if content.object(object).drink.is_some() {
        link(Term::Potion, id, name)
    } else {
        format!("[{name}](materials.md#herbs-and-essences)")
    }
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
    let ease = grade_ease(content);
    // The margin from which a grade comes with Master's hand: never below 0 (a made check).
    let mastered = |from: i32| (from - ease).max(0);
    let mut out = String::from(
        "| Margin | With Master's hand | Grade | Improvements | What you hear |\n\
         |:---:|:---:|---|:---:|---|\n",
    );
    let range = |from: i32, to: Option<i32>| match to {
        None => format!("{from} or more"),
        Some(to) if to < from => "—".to_string(),
        Some(to) => format!("{from} to {to}"),
    };
    let rows = [
        (
            range(masterwork, None),
            range(mastered(masterwork), None),
            "Masterwork",
            3,
            "You have made a Masterwork long sword!",
        ),
        (
            range(marvellous, Some(masterwork - 1)),
            range(mastered(marvellous), Some(mastered(masterwork) - 1)),
            "Marvellous",
            2,
            "This is a Marvellous long sword!",
        ),
        (
            range(fine, Some(marvellous - 1)),
            range(mastered(fine), Some(mastered(marvellous) - 1)),
            "Fine",
            1,
            "This is a Fine long sword.",
        ),
        (
            range(0, Some(fine - 1)),
            range(0, Some(mastered(fine) - 1)),
            "Plain",
            0,
            "You have made a long sword.",
        ),
        (
            "below 0".to_string(),
            "below 0".to_string(),
            "Failed",
            0,
            "The blade comes out brittle and you break it up for the metal.",
        ),
    ];
    for (margin, with, name, n, words) in rows {
        let _ = writeln!(out, "| {margin} | {with} | {name} | {n} | \"{words}\" |");
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
        "| Bonus | Who | Failed | Plain | Fine | Marvellous | Masterwork | With Master's hand \
         (Plain / Fine / Marvellous / Masterwork) |\n\
         |:---:|---|:---:|:---:|:---:|:---:|:---:|:---:|\n",
    );
    let sides = HIGHEST / 2;
    for (bonus, who) in BONUSES {
        let mut counts = [0u32; 5];
        let mut mastered = [0u32; 5];
        for a in 1..=sides {
            for b in 1..=sides {
                let natural = a + b;
                let margin = natural + bonus - difficulty;
                counts[grade(settings, natural, margin) as usize] += 1;
                mastered[grade(settings, natural, eased(margin, ease)) as usize] += 1;
            }
        }
        let _ = write!(out, "| {bonus:+} | {who} |");
        for n in counts {
            let percent = f64::from(n) * 100.0 / f64::from(sides * sides);
            let _ = write!(out, " {percent:.0}% |");
        }
        let _ = writeln!(out, " {} |", percents(&mastered, 1, (sides * sides) as u32));
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
    let ease = grade_ease(content);
    let mut out = String::from(
        "| Margin | With Master's hand | Cut | On an uncut gem | On a cut gem |\n\
         |:---:|:---:|---|---|---|\n",
    );
    let _ = writeln!(
        out,
        "| below 0 | below 0 | — | stays uncut, one size smaller | keeps its cut, one size \
         smaller |"
    );
    // The margins from `low` to `high` (none: and more) of a made check, eased by `by`.
    let band = |low: i32, high: Option<i32>, by: i32| {
        let from = if low == 0 { 0 } else { (low - by).max(0) };
        match high {
            None => format!("{from} or more"),
            Some(high) if high - by < from => "—".to_string(),
            Some(high) => format!("{from} to {}", high - by),
        }
    };
    for i in first..cuts.len() {
        let low = (i - first) as i32 * step;
        let high = (i + 1 < cuts.len()).then_some(low + step - 1);
        let margin = format!("{} | {}", band(low, high, 0), band(low, high, ease));
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

    // The odds, by the check's whole bonus, then the made cuts' with Master's hand.
    let ease = grade_ease(content);
    out.push_str("| Gem | Bonus | Who | Shattered | Failed |");
    for c in &cuts[first..] {
        let _ = write!(out, " {} |", capital(&c.name));
    }
    let names: Vec<String> = (cuts[first..].iter()).map(|c| capital(&c.name)).collect();
    let _ = write!(out, " With Master's hand ({}) |", names.join(" / "));
    out.push_str("\n|---|:---:|---|:---:|:---:|");
    for _ in first..cuts.len() {
        out.push_str(":---:|");
    }
    out.push_str(":---:|\n");
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
        let mut mastered = vec![0u32; cuts.len() + 2];
        let index = |band| match band {
            CutBand::Shattered => 0,
            CutBand::Failed => 1,
            CutBand::Cut(c) => 2 + usize::from(c.0),
        };
        for a in 1..=sides {
            for b in 1..=sides {
                let natural = a + b;
                let margin = natural + bonus - (recipe.difficulty + work);
                counts[index(cut_band(gems, step, natural, margin))] += 1;
                mastered[index(cut_band(gems, step, natural, eased(margin, ease)))] += 1;
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
        let total = (sides * sides) as u32;
        let _ = writeln!(out, " {} |", percents(&mastered, 2 + first, total));
    }
    out
}

/// Gem setting (Jewelcraft's Set, `world/set.rs`): the odds of each outcome by the check's whole
/// bonus, on the cutting tables' ruby and on the softest gem.
pub(super) fn gem_setting(content: &Content) -> String {
    let Some(recipe) = content.recipes().of_use(RecipeUse::Set) else {
        return String::new();
    };
    let mut out = String::from(
        "| Gem | Bonus | Who | Set | Chipped | Shattered |\n|---|:---:|---|:---:|:---:|:---:|\n",
    );
    let gems = (content.materials().iter()).filter(|m| m.gem.is_some());
    let ruby = gems.clone().find(|m| m.id == CUT_GEM.0);
    let soft = gems.min_by_key(|m| (m.work, m.gem.as_ref().map_or(0, |g| g.value)));
    let mut rows: Vec<(&str, i32, i32, &str)> = Vec::new();
    if let Some(ruby) = ruby {
        for &(bonus, who) in &CUT_BONUSES {
            rows.push((ruby.name.as_str(), ruby.work, bonus, who));
        }
    }
    if let Some(soft) = soft.filter(|m| Some(m.work) != ruby.map(|r| r.work)) {
        let (bonus, who) = CUT_BONUSES[0];
        rows.push((soft.name.as_str(), soft.work, bonus, who));
    }
    let sides = HIGHEST / 2;
    for (gem_name, work, bonus, who) in rows {
        let mut counts = [0u32; 3];
        for a in 1..=sides {
            for b in 1..=sides {
                let natural = a + b;
                let margin = natural + bonus - (recipe.difficulty + work);
                let i = match set_band(natural, margin) {
                    SetBand::Set => 0,
                    SetBand::Chipped => 1,
                    SetBand::Shattered => 2,
                };
                counts[i] += 1;
            }
        }
        let _ = write!(
            out,
            "| {} ({work:+}) | {bonus:+} | {who} |",
            capital(gem_name)
        );
        for n in counts {
            let percent = f64::from(n) * 100.0 / f64::from(sides * sides);
            let _ = write!(out, " {percent:.0}% |");
        }
        out.push('\n');
    }
    let _ = writeln!(
        out,
        "\nSetting takes {} against a difficulty of {} plus the gem's work, and is listed from \
         Jewelcraft {}.",
        Duration::seconds(u64::from(recipe.seconds)).spoken(),
        recipe.difficulty,
        recipe.from,
    );
    out
}
