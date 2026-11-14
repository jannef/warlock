//! Making a generic recipe (spec §8 "Skills": professions; docs/design/professions.md §7, §8,
//! docs/design/recipes.md §4, §6, §10; docs/dev/crafting.md "Making a recipe"): the forge's
//! weapons and the hand-made ones, the arrowheads, the arrows and bolts, the sling stones, the
//! flaming arrows, and the brews.
//!
//! - **The steps** ([`craft_steps`], what the wizard lists, through
//!   [`PerceptionView::craft_steps`]): one choice for each chosen part of the result, in part
//!   order: a `"stock"` part (the pieces whose material the part may be made of and the
//!   profession works, each with the count it would take), a part with a table of sources (the
//!   stacks of its `input`, then the pieces of its `stock` materials, then None if `optional`:
//!   the arrows' head), or a part that must be a gem (`{ gem = true }`: a gold amulet's setting;
//!   the cut loose gems carried, with no None); then each input no part's sources name (the
//!   stacks of an object input; the pieces of a stock input's material or group: a bow's sinew,
//!   a crossbow's ingot of any metal), then the ornament if the recipe offers one (the cut loose
//!   gems carried; None may be chosen). Set's two steps are `world/set.rs`'s ([`super::set`]).
//!   A step with one fitting stack is still a step: the player sees what is spent. A
//!   `count = "fletch"` recipe's wood part has no step on wooded ground (`world/fletch.rs`).
//! - **The plan** ([`craft_plan`]): the picks, inventory positions in step order (`None` a
//!   skipped optional), checked by the rules (the stack exists, fits, and suffices; an ornament
//!   is a cut gem), and what comes of them: the item to be made, named as it would be ("a
//!   ruby-pommelled long sword", "flint arrows"), and what it uses in words. The dialog asks with
//!   it, and [`World::start_making`] refuses with its reason, taking no time. None on an
//!   optional part makes the recipe's `brittle` object, of the other parts' chosen materials.
//! - **The work**: the recipe's `seconds` (Fletcher's share of it for the arrows), in turns of
//!   [`ActionKind::Craft`] ([`World::keep_at`]; on the overworld watched as a forage is); an
//!   interruption costs only the time. Done, the picks are found again (`work_on.rs`) and the
//!   plan made again from the pack ("You have nothing left to work with." if either fails), then
//!   one check (two gameplay draws, the natural roll): the profession's stat modifier, what the
//!   skill gives of [`Effect::Craft`] (that skill's alone), the talents' checks of the stat (and
//!   Fletcher's for the arrows), a bound lure's boon of the skill (`World::binding_boon`),
//!   against the recipe's `difficulty` plus the `work` of the main chosen part's material. The
//!   profession is the recipe's directory's (a flint knife and a steel-headed arrow are
//!   Woodcraft's work, by Dexterity), its stat and verb its row's in `data/recipes.toml` where
//!   that names them ([`profession_stat`], [`profession_verb`]; the dialog's verb is the
//!   recipe's own `verb` before the row's: a ring is made, not cut).
//! - **Graded** (a count of 1): the margin's [`Grade`] by `data/recipes.toml`'s bands: a
//!   Masterwork, Marvellous, Fine (three, two, one improvements, each an
//!   [`crate::magic::Aspect`] of the item drawn as Temper draws it, one gameplay draw each, and
//!   `attempts` equal to them, so the anvil goes on from there), plain, or failed; a natural 24
//!   makes at least Fine, a natural 2 fails. An item with no aspect is plain whatever the margin.
//!   Made, the stock and inputs are spent, the gem set, and the item goes into the pack
//!   identified; failed, nothing is made, the gem is kept, and the stock and object inputs come
//!   back at the settings' `yield`, or the holder's Waste not's where more ([`failed_percent`]) (pieces by [`crate::piece::count_for`], at least one of each;
//!   a stock input `count × yield / 100`, so none of a bow's one sinew). Inputs never set the
//!   main material. Made by a holder of Maker's mark, an item with a `kind` (arms, armour,
//!   jewellery: never an ingot or a brew) bears their mark (`Worked::marked`, the talent's one
//!   read), whatever its grade.
//! - **Counted** (a count above 1, or `"fletch"`): no grades. The count is rolled (its dice, or
//!   `world/fletch.rs`'s), then the margin keeps all of it, half rounded up within
//!   `half_within`, one within `one_within`, else none ([`batch_made`]; a natural 2 none, a
//!   natural 24 all), then Fletcher's multiple, at most the batch: what each `per_made` input's
//!   stack covers. A stock part is priced by the batch's weight at its most for the plan, and
//!   spent by the weight of what was made (at least one piece); a `per_made` input is spent for
//!   each made; another input whole. Nothing made is a failure: the stock and the whole inputs
//!   come back at the `yield`, the `per_made` ones untouched.
//! - Either way the skill may earn a mark ([`Use::Craft`]).
//!
//! **Brewing** (docs/dev/crafting.md "Brewing"): a recipe with no part, no ornament, and no
//! count above 1 chooses nothing ([`chooses_nothing`], Alchemy's): it has no steps, its object inputs are taken from
//! the pack (the first stacks of each), and the dialog only confirms them. Its check's stat and
//! the dialog's verb are its profession row's (`data/recipes.toml`: Intelligence, "Brew"). A
//! recipe with a `failure` object spends its inputs whole when it fails and makes that object; a
//! natural 2 makes its `fumble` object (else the failure's). What is made of a labelled type (a
//! potion) is named truly beforehand and its type learned when made: the maker knows their work.
//!
//! [`PerceptionView::craft_steps`]: crate::perception::PerceptionView::craft_steps
//! [`ActionKind::Craft`]: crate::content::ActionKind::Craft
//! [`Effect::Craft`]: crate::skills::Effect::Craft
//! [`Use::Craft`]: crate::skills::Use::Craft

use super::craft::{
    RecipeLack, input_fits, known_object, lack, stack_material, stock_only, worked_materials,
};
use super::encounter::Watch;
use super::fletch;
use super::rest::Ended;
use super::{Actor, LevelState, Spent, World};
use crate::action::Event;
use crate::combat;
use crate::content::{
    ActionKind, Content, InputKind, MaterialId, ObjectId, PartSource, RecipeCount, RecipeDef,
    RecipeId, RecipeInput, RecipeSettings, RecipeUse, counted,
};
use crate::item_parts::{self, ItemPart};
use crate::magic::Worked;
use crate::magic::worked::aspects;
use crate::object::Stack;
use crate::scroll_label::Lore;
use crate::skills::{Effect, Use};
use crate::stats::Stat;
use crate::time::Duration;

/// The most picks a recipe's work holds (its steps): parts, inputs, and the ornament.
pub const MAX_PICKS: usize = 8;

/// What a step of the wizard chooses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CraftStepKind {
    /// Stock for a part of the result.
    Stock,
    /// A part's material from its sources: an input's stacks, then stock (the arrows' head).
    Sources,
    /// An object used up.
    Input,
    /// A cut gem for the result's ornament part, or none.
    Ornament,
    /// A cut gem for a part that must be one (a gold amulet's setting), or Set's gem.
    Gem,
    /// An item carried to work on (Set's: one with its ornament part unset).
    Item,
    /// An affix family a seal shuts out of an enchanting's roll (an option's `at` is the
    /// family's index in `data/crystals.toml`'s `[[family]]` list).
    Family,
    /// The affix a soul-fetter keeps through a Recast (`at`: its index among the item's).
    Affix,
}

/// One stack a step may take.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CraftOption {
    /// Its inventory position.
    pub at: usize,
    /// As the inventory lists it.
    pub name: String,
    /// What it is, in a word for the running choice: its material ("steel", "ruby"), or the
    /// object's name for an input.
    pub short: String,
    /// How many of it the recipe would take (for each made, with `each`).
    pub takes: u32,
    /// `takes` is for each one made (an arrowhead an arrow).
    pub each: bool,
    /// Whether the stack holds that many.
    pub enough: bool,
}

/// One step of the wizard: what it chooses, for which part, and the stacks it may take.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CraftStep {
    pub kind: CraftStepKind,
    /// What it is for: the part ("blade", "head", "pommel") or the input ("arrowheads").
    pub label: String,
    /// None may be chosen (the ornament, the arrows' head).
    pub optional: bool,
    pub options: Vec<CraftOption>,
}

/// What a recipe with the picks chosen would make and use, in words (the dialog's).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CraftPlan {
    /// The profession's verb: "Forge".
    pub verb: String,
    /// The item, as it would be named: "a ruby-pommelled long sword", or a counted recipe's in
    /// the plural, "flint arrows".
    pub name: String,
    /// What it uses: "3 steel ingots and the ruby".
    pub uses: String,
    /// The station's name: "anvil".
    pub station: Option<String>,
    /// How long it takes: "about 1 hour".
    pub time: String,
}

/// How well a made item came out: the margin's band ([`grade`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Grade {
    Failed,
    Plain,
    Fine,
    Marvellous,
    Masterwork,
}

impl Grade {
    /// How many improvements an item of this grade has.
    pub const fn improvements(self) -> u8 {
        match self {
            Grade::Failed | Grade::Plain => 0,
            Grade::Fine => 1,
            Grade::Marvellous => 2,
            Grade::Masterwork => 3,
        }
    }
}

/// The bands when content has no `data/recipes.toml` (fixtures): the shipped ones.
const BANDS: (i32, i32, i32, u32) = (6, 12, 20, 50);

/// The counted bands when content has no `data/recipes.toml`: the shipped `half_within` and
/// `one_within`.
const COUNTED: (i32, i32) = (3, 6);

/// The bands and the yield: `data/recipes.toml`'s, else [`BANDS`].
fn bands(settings: Option<&RecipeSettings>) -> (i32, i32, i32, u32) {
    settings.map_or(BANDS, |s| {
        (s.fine, s.marvellous, s.masterwork, s.yield_percent)
    })
}

/// What a failed recipe gives back to `player`, in percent of its stock and object inputs:
/// `data/recipes.toml`'s `yield`, or the holder's Waste not where that is more
/// (`TalentMods::failed_percent`; the craftsperson design §7.3), at most all of it. The one read,
/// by [`resolve`] and [`taken`] as they set each [`Spend::back`].
fn failed_percent(content: &Content, player: &Actor) -> u32 {
    let percent = bands(content.recipes().settings.as_ref()).3;
    percent
        .max(player.talent_mods(content).failed_percent)
        .min(100)
}

/// The grade of a check of natural roll `natural` made by `margin` (the roll and the modifier
/// less the difficulty; below 0 a failure): a natural 2 fails, a natural 24 makes at least Fine.
pub fn grade(settings: Option<&RecipeSettings>, natural: i32, margin: i32) -> Grade {
    let (fine, marvellous, masterwork, _) = bands(settings);
    if natural == combat::FUMBLE {
        return Grade::Failed;
    }
    let by_margin = match margin {
        m if m >= masterwork => Grade::Masterwork,
        m if m >= marvellous => Grade::Marvellous,
        m if m >= fine => Grade::Fine,
        m if m >= 0 => Grade::Plain,
        _ => Grade::Failed,
    };
    match natural == combat::HIGHEST {
        true => by_margin.max(Grade::Fine),
        false => by_margin,
    }
}

/// How many of `count` a counted recipe's check of natural roll `natural` made by `margin`
/// makes: all of it from 0, half rounded up within `half_within`, one within `one_within`, else
/// none; a natural 2 none, a natural 24 all.
pub fn batch_made(settings: Option<&RecipeSettings>, natural: i32, margin: i32, count: i32) -> i32 {
    let (half, one) = settings.map_or(COUNTED, |s| (s.half_within, s.one_within));
    if natural == combat::FUMBLE {
        return 0;
    }
    let margin = match natural == combat::HIGHEST {
        true => margin.max(0),
        false => margin,
    };
    match margin {
        m if m >= 0 => count,
        m if m >= -half => (count + 1) / 2,
        m if m >= -one => count.min(1),
        _ => 0,
    }
}

/// Whether `recipe` is counted (module docs): a count above 1, or `"fletch"`.
pub(crate) fn is_counted(recipe: &RecipeDef) -> bool {
    match recipe.result.as_ref().map(|r| r.count) {
        Some(RecipeCount::Dice(dice)) => dice.max() > 1,
        Some(RecipeCount::Fletch) => true,
        None => false,
    }
}

/// The most a batch of `recipe` by `player` can make (1 for a recipe that is not counted).
pub(crate) fn batch_most(content: &Content, player: &Actor, recipe: &RecipeDef) -> u32 {
    match recipe.result.as_ref().map(|r| r.count) {
        Some(RecipeCount::Dice(dice)) => dice.max().max(1).unsigned_abs(),
        Some(RecipeCount::Fletch) => fletch::fletch_most(content, player, recipe),
        None => 1,
    }
}

/// The profession `recipe` is made by: its own skill's (the directory's: a flint knife is
/// Woodcraft's, a steel one Metalcraft's, a steel-headed arrow Woodcraft's), else the main
/// material's.
fn profession<'c>(
    content: &'c Content,
    recipe: &RecipeDef,
    main: Option<MaterialId>,
) -> Option<&'c crate::content::Profession> {
    let stock = content.stock();
    (stock.profession(recipe.skill)).or_else(|| stock.profession_of(content, main?))
}

/// The check's stat for `recipe`: its profession row's in `data/recipes.toml` where it names one
/// (Alchemy's Intelligence: herbs are no stock), else [`profession`]'s, else Strength.
fn profession_stat(content: &Content, recipe: &RecipeDef, main: Option<MaterialId>) -> Stat {
    (content.recipes().profession(recipe.skill))
        .and_then(|row| row.stat)
        .or_else(|| profession(content, recipe, main).map(|p| p.stat))
        .unwrap_or(Stat::Strength)
}

/// The dialog's verb for `recipe`: its own (`verb`: a ring is made, not cut), else its
/// profession row's ("Brew"), else [`profession`]'s, else "Make".
fn profession_verb(content: &Content, recipe: &RecipeDef, main: Option<MaterialId>) -> String {
    (recipe.verb.clone())
        .or_else(|| (content.recipes().profession(recipe.skill)).and_then(|row| row.verb.clone()))
        .or_else(|| profession(content, recipe, main).map(|p| p.verb.clone()))
        .unwrap_or_else(|| "Make".to_string())
}

/// A step of a recipe: which of its parts or inputs, or the ornament.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    /// The `"stock"` part at this index of the result's parts.
    Part(usize),
    /// The part with a table of sources at this index of the result's parts.
    Sources(usize),
    /// The input at this index of the recipe's.
    Input(usize),
    /// The result's ornament part.
    Ornament(crate::content::PartId),
    /// The part at this index of the result's parts, a cut gem.
    Gem(usize),
}

/// The source of part `part` of `recipe`'s result, if the maker chooses it.
fn source_of(recipe: &RecipeDef, part: usize) -> Option<&PartSource> {
    (recipe.parts.iter())
        .find(|p| p.part == part)
        .map(|p| &p.source)
}

/// The object inputs a part's sources name: chosen in that part's step, never in one of their
/// own.
fn sourced(recipe: &RecipeDef) -> Vec<ObjectId> {
    (recipe.parts.iter())
        .filter_map(|p| match &p.source {
            PartSource::Sources { input, .. } => *input,
            _ => None,
        })
        .collect()
}

/// The steps of `recipe` in the wizard's order, `free` saying which `"stock"` parts need none
/// here; `None` for a recipe this phase cannot make (the module docs).
fn steps(content: &Content, recipe: &RecipeDef, free: impl Fn(usize) -> bool) -> Option<Vec<Step>> {
    let result = recipe.result.as_ref()?;
    if chooses_nothing(recipe) {
        // Its inputs are taken from the pack, never chosen; a stock input is only chosen.
        return (recipe.inputs.iter())
            .all(|i| i.object().is_some())
            .then(Vec::new);
    }
    let mut steps = Vec::new();
    for part in &recipe.parts {
        match &part.source {
            PartSource::Stock if free(part.part) => {}
            // Stock alone (a spear's haft of another profession's wood) is a stock step.
            source if stock_only(source) => steps.push(Step::Part(part.part)),
            PartSource::Stock => steps.push(Step::Part(part.part)),
            PartSource::Sources { .. } => steps.push(Step::Sources(part.part)),
            PartSource::Gem => steps.push(Step::Gem(part.part)),
        }
    }
    let taken = sourced(recipe);
    for (i, input) in recipe.inputs.iter().enumerate() {
        if !input.object().is_some_and(|o| taken.contains(&o)) {
            steps.push(Step::Input(i));
        }
    }
    if recipe.ornament {
        let def = content.object(result.object);
        let placed = (content.gems()).and_then(|g| g.ornament(def.class));
        steps.push(Step::Ornament(def.ornament.or(placed.map(|o| o.part))?));
    }
    (steps.len() <= MAX_PICKS).then_some(steps)
}

/// Whether `recipe` chooses nothing (no part, no ornament, not counted: a brew): its object
/// inputs are taken from the pack, the first stacks of each. A counted recipe's inputs are
/// chosen, for the stacks chosen cap its batch (the flaming arrows' arrows).
fn chooses_nothing(recipe: &RecipeDef) -> bool {
    recipe.parts.is_empty() && !recipe.ornament && !is_counted(recipe)
}

/// The grams of `material` part `part` of one of `recipe`'s result takes: its share of the
/// result's weight, scaled by the material's weight over the base part's, as the made item
/// weighs it.
fn part_grams(
    content: &Content,
    recipe: &RecipeDef,
    part: usize,
    material: MaterialId,
) -> Option<u64> {
    let def = content.object(recipe.result.as_ref()?.object);
    let base = def.parts.get(part)?;
    let weight = |m: MaterialId| u64::from(content.material(m).weight).max(1);
    Some(
        u64::from(def.weight) * u64::from(base.share) * weight(material)
            / weight(base.material)
            / 100,
    )
}

/// The pieces of `material` part `part` of `batch` of `recipe`'s result take
/// ([`crate::piece::count_for`] of their grams).
pub(super) fn stock_need(
    content: &Content,
    recipe: &RecipeDef,
    part: usize,
    material: MaterialId,
    batch: u32,
) -> Option<u32> {
    let grams = part_grams(content, recipe, part, material)?;
    crate::piece::count_for(content, grams * u64::from(batch), material)
}

/// How many, up to `most`, `have` pieces of `material` make of a part of `grams` each (0 when
/// not even one).
fn covers(content: &Content, grams: u64, material: MaterialId, have: u32, most: u32) -> u32 {
    let need = |n: u32| crate::piece::count_for(content, grams * u64::from(n), material);
    (1..=most)
        .rev()
        .find(|&n| need(n).is_some_and(|c| c <= have))
        .unwrap_or(0)
}

/// The material of `stack`'s main part (an arrowhead's metal).
fn main_material(content: &Content, stack: &Stack) -> Option<MaterialId> {
    stack.parts(content).first().map(|p| p.material)
}

/// The cut loose gem of `stack`, if it is one (an uncut one is never set).
pub(super) fn cut_gem(
    content: &Content,
    stack: &Stack,
) -> Option<(MaterialId, crate::content::Cutting)> {
    let gems = content.gems()?;
    if stack.object != gems.object {
        return None;
    }
    let (gem, cutting) = stack.gem(content)?;
    (!gems.cut(cutting.cut).rough).then_some((gem, cutting))
}

/// What a step is for, in words: the part's name ("blade", "head", "pommel"; a main part without
/// one by the group of the stock it takes, "metal"), or the input's plural.
fn label(content: &Content, recipe: &RecipeDef, step: Step) -> String {
    let Some(result) = recipe.result.as_ref() else {
        return String::new();
    };
    let def = content.object(result.object);
    let part_name = |part: Option<crate::content::PartId>| {
        part.map(|p| content.part_defs()[usize::from(p.0)].name.clone())
    };
    match step {
        Step::Part(i) | Step::Sources(i) => (def.parts.get(i))
            .and_then(|p| part_name(p.part))
            .or_else(|| {
                // The group of the stock it takes, when that is one group: "metal".
                let worked = worked_materials(content, recipe, result.object, i);
                let group = |m: &MaterialId| content.material(*m).group.clone();
                let first = group(worked.first()?);
                worked.iter().all(|m| group(m) == first).then_some(first)?
            })
            .unwrap_or_else(|| "material".into()),
        Step::Input(i) => match &recipe.inputs[i].kind {
            InputKind::Object(o) => content.object(*o).plural.clone(),
            InputKind::Material(m) => content.material(*m).name.clone(),
            InputKind::Group(group) => group.clone(),
        },
        Step::Ornament(part) => part_name(Some(part)).unwrap_or_default(),
        Step::Gem(i) => (def.parts.get(i))
            .and_then(|p| part_name(p.part))
            .unwrap_or_else(|| "gem".into()),
    }
}

/// The wizard's steps of `recipe` for `player` on `level` (the module docs); empty for a recipe
/// this phase cannot make.
pub(crate) fn craft_steps(
    content: &Content,
    lore: Lore<'_>,
    level: &LevelState,
    player: &Actor,
    recipe: RecipeId,
) -> Vec<CraftStep> {
    let def = content.recipe(recipe);
    if def.uses == Some(RecipeUse::Set) {
        return super::set::steps(content, lore, player);
    }
    if let Some(RecipeUse::Enchant(_)) = def.uses {
        return super::enchant::steps(content, lore, player, def, &[]);
    }
    let free = |part| fletch::part_free(content, level, player, def, part);
    let Some(steps) = steps(content, def, free) else {
        return Vec::new();
    };
    let Some(result) = def.result.as_ref() else {
        return Vec::new();
    };
    let most = batch_most(content, player, def);
    let pack = &player.inventory;
    let option = |at: usize, stack: &Stack, takes: u32, each: bool, of: Option<MaterialId>| {
        let of = of.or(cut_gem(content, stack).map(|g| g.0));
        CraftOption {
            at,
            name: lore.listed(stack),
            short: of.map_or_else(|| lore.title(stack), |m| content.material(m).name.clone()),
            takes,
            each,
            enough: stack.count >= takes,
        }
    };
    // The pieces of `materials` carried, each with what the batch at its most takes of it for
    // part `part` (a counted batch at most what the stack covers; one made at the least).
    let pieces = |part: usize, materials: &[MaterialId]| -> Vec<CraftOption> {
        (pack.iter().enumerate())
            .filter_map(|(at, stack)| {
                let m = stack_material(content, stack).filter(|m| materials.contains(m))?;
                let one = stock_need(content, def, part, m, 1)?;
                let need = stock_need(content, def, part, m, most)?;
                let mut option = option(at, stack, need.min(stack.count).max(one), false, Some(m));
                option.enough = stack.count >= one;
                Some(option)
            })
            .collect()
    };
    // The stacks of input `object` carried, with what each made takes.
    let inputs = |object: ObjectId, part: Option<usize>| -> Vec<CraftOption> {
        let Some(input) = def.inputs.iter().find(|i| i.object() == Some(object)) else {
            return Vec::new();
        };
        let made_of = part.map(|p| content.object(result.object).made_of(p));
        (pack.iter().enumerate())
            .filter(|(_, s)| known_object(lore, s, object))
            .filter_map(|(at, stack)| {
                let of = part.and(main_material(content, stack));
                if made_of.is_some_and(|list| !of.is_some_and(|m| list.contains(&m))) {
                    return None;
                }
                Some(option(at, stack, input.count, input.per_made, of))
            })
            .collect()
    };
    steps
        .into_iter()
        .map(|step| {
            let (kind, optional, options) = match step {
                Step::Part(part) => {
                    let worked = worked_materials(content, def, result.object, part);
                    (CraftStepKind::Stock, false, pieces(part, &worked))
                }
                Step::Sources(part) => {
                    let Some(PartSource::Sources {
                        input,
                        stock,
                        optional,
                    }) = source_of(def, part)
                    else {
                        return None;
                    };
                    let mut options = input.map_or_else(Vec::new, |o| inputs(o, Some(part)));
                    options.extend(pieces(part, stock));
                    (CraftStepKind::Sources, *optional, options)
                }
                Step::Input(i) => {
                    let input = &def.inputs[i];
                    // A stock input: the pieces of its material or group (a crossbow's ingot of
                    // any metal: every metal ingot carried).
                    let stock = || {
                        (pack.iter().enumerate())
                            .filter(|(_, s)| input_fits(lore, input, s))
                            .map(|(at, stack)| {
                                let of = stack_material(content, stack);
                                option(at, stack, input.count, input.per_made, of)
                            })
                            .collect()
                    };
                    let options = input.object().map_or_else(stock, |o| inputs(o, None));
                    (CraftStepKind::Input, false, options)
                }
                Step::Ornament(_) | Step::Gem(_) => {
                    let options = (pack.iter().enumerate())
                        .filter(|(_, s)| cut_gem(content, s).is_some())
                        .map(|(at, stack)| option(at, stack, 1, false, None))
                        .collect();
                    match step {
                        Step::Ornament(_) => (CraftStepKind::Ornament, true, options),
                        _ => (CraftStepKind::Gem, false, options),
                    }
                }
            };
            Some(CraftStep {
                kind,
                label: label(content, def, step),
                optional,
                options,
            })
        })
        .collect::<Option<Vec<_>>>()
        .unwrap_or_default()
}

/// How a stack chosen is given up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Per {
    /// All of `count` when anything is made.
    Whole,
    /// This many for each one made (`per_made`).
    Made(u32),
    /// Stock: the pieces of these grams of the material for each one made.
    Stock(u64, MaterialId),
}

/// What a stack chosen gives up: how many at most, and how many come back if the work fails.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Spend {
    at: usize,
    count: u32,
    /// What a failure gives back of it (the gem: all of it).
    back: u32,
    per: Per,
}

impl Spend {
    /// How many of it go when `made` are made (none made: a failure).
    fn gone(&self, content: &Content, made: u32) -> u32 {
        match (self.per, made) {
            (Per::Made(each), _) => each * made,
            (_, 0) => self.count - self.back,
            (Per::Whole, _) => self.count,
            (Per::Stock(grams, material), _) => {
                (crate::piece::count_for(content, grams * u64::from(made), material))
                    .unwrap_or(self.count)
                    .min(self.count)
            }
        }
    }
}

/// A plan checked against the pack: the item it makes, what it spends, the main material.
#[derive(Clone, Debug)]
struct Resolved {
    made: Stack,
    spends: Vec<Spend>,
    /// The main chosen part's material: its `work` is on the difficulty.
    main: Option<MaterialId>,
    /// A counted recipe's batch at its most (capped by its `per_made` stacks); `None` graded.
    batch: Option<u32>,
    /// What a failure makes, and a natural 2 (the recipe's `failure`, `fumble`).
    failure: Option<ObjectId>,
    fumble: Option<ObjectId>,
    /// The profession.
    skill: crate::skills::SkillId,
}

/// What input `input` chosen at `at` gives up.
fn input_spend(at: usize, input: &RecipeInput, recipe: &RecipeDef, yield_percent: u32) -> Spend {
    match input.per_made {
        true => Spend {
            at,
            count: input.count,
            back: 0,
            per: Per::Made(input.count),
        },
        false => Spend {
            at,
            count: input.count,
            back: match recipe.failure {
                Some(_) => 0,
                None => input.count * yield_percent / 100,
            },
            per: Per::Whole,
        },
    }
}

/// `recipe` made by `player` on `level` with `picks` (the module docs): what it makes and
/// spends, or why not, as the rules say it.
fn resolve(
    content: &Content,
    lore: Lore<'_>,
    level: &LevelState,
    player: &Actor,
    recipe: &RecipeDef,
    picks: &[Option<usize>],
) -> Result<Resolved, String> {
    let not_yet = || "You cannot make that yet.".to_string();
    let free = |part| fletch::part_free(content, level, player, recipe, part);
    let steps = steps(content, recipe, free).ok_or_else(not_yet)?;
    let result = recipe.result.as_ref().ok_or_else(not_yet)?;
    let def = content.object(result.object);
    if picks.len() > steps.len() {
        return Err("That is more than the work takes.".into());
    }
    let yield_percent = failed_percent(content, player);
    let pack = &player.inventory;
    // The parts' chosen materials, in step order, and the ornament's gem.
    let mut chosen: Vec<(usize, MaterialId)> = Vec::new();
    let mut gems: Vec<ItemPart> = Vec::new();
    let mut spends: Vec<Spend> = Vec::new();
    let mut brittle = false;
    for (n, &step) in steps.iter().enumerate() {
        let pick = picks.get(n).copied().flatten();
        let what = label(content, recipe, step);
        let will_not_do = || format!("That will not do for the {what}.");
        let Some(at) = pick else {
            match step {
                Step::Ornament(_) => continue,
                Step::Sources(part)
                    if matches!(
                        source_of(recipe, part),
                        Some(PartSource::Sources { optional: true, .. })
                    ) =>
                {
                    brittle = true;
                    continue;
                }
                _ => return Err(format!("You must choose the {what}.")),
            }
        };
        let stack = pack.get(at).ok_or("You carry no such thing.")?;
        let stock_spend = |part: usize, material: MaterialId| -> Result<Spend, String> {
            let grams = part_grams(content, recipe, part, material).ok_or_else(not_yet)?;
            Ok(Spend {
                at,
                count: 0,
                back: 0,
                per: Per::Stock(grams, material),
            })
        };
        match step {
            Step::Part(part) => {
                let worked = worked_materials(content, recipe, result.object, part);
                let material = (stack_material(content, stack))
                    .filter(|m| worked.contains(m))
                    .ok_or_else(will_not_do)?;
                spends.push(stock_spend(part, material)?);
                chosen.push((part, material));
            }
            Step::Sources(part) => {
                let Some(PartSource::Sources { input, stock, .. }) = source_of(recipe, part) else {
                    return Err(not_yet());
                };
                let made_of = def.made_of(part);
                if input.is_some_and(|o| known_object(lore, stack, o)) {
                    let material = (main_material(content, stack))
                        .filter(|m| made_of.contains(m))
                        .ok_or_else(will_not_do)?;
                    let spec = (recipe.inputs.iter())
                        .find(|i| i.object() == Some(stack.object))
                        .ok_or_else(not_yet)?;
                    spends.push(input_spend(at, spec, recipe, yield_percent));
                    chosen.push((part, material));
                } else {
                    let material = (stack_material(content, stack))
                        .filter(|m| stock.contains(m))
                        .ok_or_else(will_not_do)?;
                    spends.push(stock_spend(part, material)?);
                    chosen.push((part, material));
                }
            }
            Step::Input(i) => {
                let input = &recipe.inputs[i];
                if !input_fits(lore, input, stack) {
                    return Err(will_not_do());
                }
                spends.push(input_spend(at, input, recipe, yield_percent));
            }
            Step::Ornament(_) | Step::Gem(_) => {
                let (gem, cutting) = (cut_gem(content, stack))
                    .ok_or_else(|| "Only a cut gem can be set.".to_string())?;
                spends.push(Spend {
                    at,
                    count: 1,
                    back: 1,
                    per: Per::Whole,
                });
                let part = match step {
                    Step::Gem(i) => def.parts.get(i).and_then(|p| p.part),
                    Step::Ornament(part) => Some(part),
                    Step::Part(_) | Step::Sources(_) | Step::Input(_) => None,
                };
                gems.push(ItemPart::gem(content, part, gem, cutting));
            }
        }
    }
    if chooses_nothing(recipe) {
        taken(content, lore, player, recipe, &mut spends)?;
    }
    // The batch: its most, at most what each stack taken covers (a counted recipe's).
    let most = batch_most(content, player, recipe);
    let batch = (spends.iter())
        .map(|s| {
            let have = pack[s.at].count;
            match s.per {
                Per::Made(each) => have / each.max(1),
                Per::Stock(grams, material) => covers(content, grams, material, have, most),
                Per::Whole => most,
            }
        })
        .fold(most, u32::min)
        .max(1);
    for spend in &mut spends {
        if let Per::Stock(grams, material) = spend.per {
            let total = grams * u64::from(batch);
            spend.count = crate::piece::count_for(content, total, material).ok_or_else(not_yet)?;
            spend.back =
                crate::piece::count_for(content, total * u64::from(yield_percent) / 100, material)
                    .unwrap_or(0)
                    .min(spend.count);
        }
    }
    // A stack chosen twice must hold what both take.
    for spend in &spends {
        let total: u32 = (spends.iter())
            .filter(|s| s.at == spend.at)
            .map(|s| s.count)
            .sum();
        if pack[spend.at].count < total {
            return Err("You have too few of those.".into());
        }
    }
    let object = match brittle {
        true => recipe.brittle.ok_or_else(not_yet)?,
        false => result.object,
    };
    let made_def = content.object(object);
    let mut own: Vec<ItemPart> = Vec::new();
    for (i, base) in made_def.parts.iter().enumerate() {
        // Each part of the made item takes its own chosen material; the brittle object's take
        // the first chosen that fits them (the shaft's wood).
        let material = match brittle {
            true => (chosen.iter())
                .map(|&(_, m)| m)
                .find(|m| made_def.made_of(i).contains(m)),
            false => (chosen.iter()).find(|&&(p, _)| p == i).map(|&(_, m)| m),
        };
        if let Some(material) = material.filter(|&m| m != base.material) {
            let id = (i > 0).then_some(base.part).flatten();
            own.push(ItemPart::material(content, id, material));
        }
    }
    own.extend(gems);
    let mut made = Stack::one(object);
    if !own.is_empty() {
        item_parts::check(made_def, &own, content).map_err(|_| not_yet())?;
        made = made.with_parts(own);
    }
    Ok(Resolved {
        made,
        spends,
        main: chosen.first().map(|&(_, m)| m),
        batch: is_counted(recipe).then_some(batch),
        failure: recipe.failure,
        fumble: recipe.fumble,
        skill: recipe.skill,
    })
}

/// The object inputs of `recipe`, which chooses nothing, taken from `player`'s pack: the first
/// stacks of each object until the count is met (all spent on a failure when the recipe has a
/// `failure` object, else the settings' yield back as a chosen input's); too few is refused.
fn taken(
    content: &Content,
    lore: Lore<'_>,
    player: &Actor,
    recipe: &RecipeDef,
    spends: &mut Vec<Spend>,
) -> Result<(), String> {
    let yield_percent = failed_percent(content, player);
    for input in &recipe.inputs {
        // Stock inputs are chosen, never taken (`steps`).
        let Some(object) = input.object() else {
            return Err("You cannot make that yet.".into());
        };
        let mut left = input.count;
        for (at, stack) in player.inventory.iter().enumerate() {
            if left == 0 {
                break;
            }
            if !input_fits(lore, input, stack) {
                continue;
            }
            let count = stack.count.min(left);
            left -= count;
            let back = match recipe.failure {
                Some(_) => 0,
                None => count * yield_percent / 100,
            };
            spends.push(Spend {
                at,
                count,
                back,
                per: Per::Whole,
            });
        }
        if left > 0 {
            let plural = &content.object(object).plural;
            return Err(format!("You have too few {plural}."));
        }
    }
    Ok(())
}

/// What `made` is called before it is made: truly, though its type (a potion's) is unknown, for
/// the recipe names it; else as the player would know it.
fn made_name(content: &Content, lore: Lore<'_>, made: &Stack, article: bool) -> String {
    match (lore.label(made.object).is_some(), article) {
        (true, true) => made.describe(content),
        (true, false) => made.title(content),
        (false, true) => lore.describe(made),
        (false, false) => lore.title(made),
    }
}

/// What `recipe` with `picks` would make and use, in words (the module docs), or why not.
pub(crate) fn craft_plan(
    content: &Content,
    lore: Lore<'_>,
    level: &LevelState,
    player: &Actor,
    recipe: RecipeId,
    picks: &[Option<usize>],
) -> Result<CraftPlan, String> {
    let def = content.recipe(recipe);
    if def.uses == Some(RecipeUse::Set) {
        return super::set::plan(content, lore, player, def, picks);
    }
    if let Some(RecipeUse::Enchant(_)) = def.uses {
        return super::enchant::plan(content, lore, player, def, picks);
    }
    let resolved = resolve(content, lore, level, player, def, picks)?;
    let pack = &player.inventory;
    // Stacks of one thing taken together (a brew's herbs from two stacks) are told as one; what
    // goes for each made, then the gem (the ornament, the last step), after them.
    let mut counts: Vec<(String, String, u32)> = Vec::new();
    let mut each: Vec<String> = Vec::new();
    let mut gems: Vec<String> = Vec::new();
    for spend in &resolved.spends {
        let stack = &pack[spend.at];
        let (one, many) = lore.names(stack);
        match (cut_gem(content, stack), spend.per) {
            (Some(_), _) => gems.push(format!("the {}", lore.title(stack))),
            (None, Per::Made(n)) => {
                each.push(format!("{} for each made", counted(&one, &many, n)));
            }
            (None, _) => match counts.iter_mut().find(|(o, _, _)| *o == one) {
                Some(seen) => seen.2 += spend.count,
                None => counts.push((one, many, spend.count)),
            },
        }
    }
    let mut uses: Vec<String> = (counts.iter())
        .map(|(one, many, n)| counted(one, many, *n))
        .collect();
    uses.extend(each);
    uses.extend(gems);
    let verb = profession_verb(content, def, resolved.main);
    let name = match resolved.batch {
        Some(_) => lore.names(&resolved.made).1,
        None => made_name(content, lore, &resolved.made, true),
    };
    let seconds = fletch::fletcher_seconds(content, player, def);
    Ok(CraftPlan {
        verb,
        name,
        uses: joined(&uses),
        station: def.station.map(|t| content.terrain(t).name.clone()),
        time: format!("about {}", Duration::seconds(u64::from(seconds)).spoken()),
    })
}

/// `items` joined as a list is said: "a, b and c".
fn joined(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// A recipe's work under way: which, with what (positions in step order), and how long.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Crafting {
    recipe: RecipeId,
    picks: [Option<usize>; MAX_PICKS],
    len: u8,
    seconds: u32,
}

impl Crafting {
    fn picks(&self) -> &[Option<usize>] {
        &self.picks[..usize::from(self.len)]
    }
}

impl World {
    /// The player begins to make generic recipe `id` with `picks` (the module docs): refused,
    /// taking no time, for a recipe beyond them or this phase, away from its station, without
    /// its knife or launcher, or with picks that do not hold.
    pub(super) fn start_making(&mut self, id: RecipeId, picks: &[Option<usize>]) -> Spent {
        let content = self.content.clone();
        let def = content.recipe(id);
        let me = self.state.player();
        let level = self.state.current();
        let refused = match lack(&content, self.lore(), level, me, def) {
            Some(RecipeLack::Skill) => Some("That is beyond your skill.".to_string()),
            Some(RecipeLack::Station(t)) => {
                Some(format!("There is no {} here.", content.terrain(t).name))
            }
            Some(RecipeLack::Knife) => Some("You have no knife.".into()),
            Some(RecipeLack::Gem) => Some("You have no gem to hand.".into()),
            Some(RecipeLack::Launcher(o)) => Some(format!(
                "You have no {} ready.",
                fletch::launcher_word(&content, o)
            )),
            Some(
                RecipeLack::Setting | RecipeLack::Nothing | RecipeLack::Room | RecipeLack::Crystal,
            ) => Some("You cannot make that yet.".into()),
            // The picks say what is missing.
            Some(RecipeLack::Stock(_) | RecipeLack::Input(_)) | None => None,
        };
        let resolved = resolve(&content, self.lore(), level, me, def, picks);
        let resolved = match (refused, resolved) {
            (Some(text), _) | (None, Err(text)) => {
                self.say(text);
                return Spent::Nothing;
            }
            (None, Ok(resolved)) => resolved,
        };
        let mut held = [None; MAX_PICKS];
        held[..picks.len()].copy_from_slice(picks);
        // The station beside them by its own name: a makeshift anvil is one.
        let level = self.state.current();
        let at = (def.station)
            .and_then(|t| super::craft::station_beside(&content, level, me.pos, t))
            .and_then(|p| level.terrain_at(p))
            .map(|t| content.terrain(t).name.clone());
        let name = match resolved.batch {
            Some(_) => self.lore().names(&resolved.made).1,
            None => made_name(&content, self.lore(), &resolved.made, false),
        };
        let seconds = fletch::fletcher_seconds(&content, me, def);
        self.say(match at {
            Some(station) => format!("You set to work on the {name} at the {station}."),
            None => format!("You set to work on the {name}."),
        });
        Spent::Craft(Crafting {
            recipe: id,
            picks: held,
            len: picks.len() as u8,
            seconds,
        })
    }

    /// Works turn after turn until the work is done or something stops it (module docs): its
    /// last turn cut short to end exactly when its time is up.
    pub(super) fn make(&mut self, mut work: Crafting, events: &mut Vec<Event>) {
        let on = self.work_on(work.picks());
        let until = self.state.time + Duration::seconds(u64::from(work.seconds));
        let turn = self.price_of(self.state.player, ActionKind::Craft);
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| {
                let price = turn.min(w.price_until(until));
                // Who comes upon one at work on the overworld (`encounter.rs`), as a forager.
                w.watch_roll(Watch::Forage, price);
                price
            },
        );
        match ended {
            Ended::Died => {}
            _ if self.state.time >= until => match self.worked_on(&on) {
                Some(now) => {
                    work.picks[..now.len()].copy_from_slice(&now);
                    self.crafted(work);
                }
                None => self.say("You have nothing left to work with."),
            },
            Ended::Came(came) => {
                let text = format!("{}; you put down your work.", self.came_line(came));
                self.say(text);
            }
            Ended::Said | Ended::Done => self.say("You put down your work."),
        }
    }

    /// The modifier of the player's check for recipe `def` (the module docs).
    pub(crate) fn craft_modifier(&self, def: &RecipeDef, main: Option<MaterialId>) -> i32 {
        let content = &self.content;
        let stat = profession_stat(content, def, main);
        let me = self.state.player();
        me.effective_stats(content).modifier(stat)
            + me.skill_effect_of(content, def.skill, Effect::Craft)
            + me.talent_mods(content).check(stat)
            + fletch::fletcher_check(content, me, def)
            + self.binding_boon(def.skill)
    }

    /// The work of `work` is done: the plan again, the check, what comes of it, the mark.
    fn crafted(&mut self, work: Crafting) {
        let content = self.content.clone();
        let def = content.recipe(work.recipe);
        let level = self.state.current();
        let player = self.state.player();
        let Ok(resolved) = resolve(&content, self.lore(), level, player, def, work.picks()) else {
            self.say("You have nothing left to work with.");
            return;
        };
        let modifier = self.craft_modifier(def, resolved.main);
        let natural = combat::roll_natural(&mut self.state.rng);
        let work_of = resolved.main.map_or(0, |m| content.material(m).work);
        let margin = natural + modifier - (def.difficulty + work_of);
        match resolved.batch {
            Some(_) => self.make_batch(def, resolved, natural, margin),
            None => self.make_out(resolved, natural, margin),
        }
        if self.trains(Use::Craft) {
            self.train_skill(Use::Craft, def.skill);
        }
    }

    /// Gives up what `spends` take when `made` are made (none: a failure), highest position
    /// first, so a stack used up leaves the others where they were.
    fn spend_out(&mut self, mut spends: Vec<Spend>, made: u32) {
        let content = self.content.clone();
        spends.sort_by_key(|s| std::cmp::Reverse(s.at));
        for spend in spends {
            let gone = spend.gone(&content, made);
            let pack = &mut self.player_mut().inventory;
            pack[spend.at].count -= gone.min(pack[spend.at].count);
            if pack[spend.at].count == 0 {
                pack.remove(spend.at);
            }
        }
    }

    /// What came of a counted recipe's check of natural roll `natural` made by `margin` on
    /// `resolved` (the module docs): the count rolled and scaled, at most the batch.
    fn make_batch(&mut self, def: &RecipeDef, resolved: Resolved, natural: i32, margin: i32) {
        let content = self.content.clone();
        let me = self.state.player();
        let share = fletch::share(&content, me, def);
        let times = fletch::count_times(&content, me, def);
        let count = match def.result.as_ref().map(|r| r.count) {
            Some(RecipeCount::Fletch) => fletch::fletch_count(&mut self.state.rng, &content, share),
            Some(RecipeCount::Dice(dice)) => dice.roll(&mut self.state.rng),
            None => 1,
        };
        let settings = content.recipes().settings.as_ref();
        let made = batch_made(settings, natural, margin, count.max(1)) * times;
        let made = made.max(0).unsigned_abs().min(resolved.batch.unwrap_or(1));
        let Resolved {
            made: mut stack,
            spends,
            ..
        } = resolved;
        self.spend_out(spends, made);
        if made == 0 {
            self.say("You spoil the whole batch.");
            return;
        }
        stack.count = made;
        stack.identify();
        let text = format!(
            "You make {}.",
            crate::object::phrase(std::slice::from_ref(&stack), self.lore())
        );
        self.say(text);
        crate::object::add_to_inventory(&mut self.player_mut().inventory, stack, &content);
    }

    /// What came of a check of natural roll `natural` made by `margin` on `resolved`: the item
    /// made at its grade and the stock spent, or the stock given back in part.
    fn make_out(&mut self, resolved: Resolved, natural: i32, margin: i32) {
        let content = self.content.clone();
        let settings = content.recipes().settings.as_ref();
        // Master's hand: the grade comes sooner, never the making (`mastery.rs`).
        let eased = super::mastery::graded_margin(&content, self.state.player(), margin);
        let grade = grade(settings, natural, eased);
        let spoilt = match grade {
            Grade::Failed if natural == combat::FUMBLE => resolved.fumble.or(resolved.failure),
            Grade::Failed => resolved.failure,
            _ => None,
        };
        let Resolved {
            mut made,
            spends,
            skill,
            main,
            ..
        } = resolved;
        let failed = grade == Grade::Failed;
        self.spend_out(spends, u32::from(!failed));
        if let Some(object) = spoilt {
            self.brew_spoilt(skill, object, natural == combat::FUMBLE);
            return;
        }
        if failed {
            let metal = (main.and_then(|m| content.material(m).group.as_deref())) == Some("metal");
            // Arms and armour come out brittle; a ring (no aspect) just goes wrong.
            let arms = !aspects(content.object(made.object)).is_empty();
            let all = self.state.player().talent_mods(&content).failed_percent >= 100;
            self.say(match (metal && arms, all) {
                (true, false) => "The blade comes out brittle and you break it up for the metal.",
                (true, true) => {
                    "The blade comes out brittle and you break it up for the metal, every ounce."
                }
                (false, false) => "The work goes wrong, and you save what you can of it.",
                (false, true) => "The work goes wrong, but you save every scrap of it.",
            });
            return;
        }
        let among = aspects(content.object(made.object));
        let most = grade.improvements().min(content.temper().most);
        let grade = if among.is_empty() || most == 0 {
            Grade::Plain
        } else {
            let mut worked = Worked::default();
            for _ in 0..most {
                let i = self.state.rng.index(among.len());
                worked.improve(among[i]);
                worked.attempted();
            }
            made.set_worked(worked);
            grade
        };
        // A master's work bears their mark (Maker's mark, the one read): arms, armour, and
        // jewellery, whatever the grade.
        let def = content.object(made.object);
        if def.kind.is_some() && self.state.player().talent_mods(&content).makers_mark {
            let mut worked = made.worked().cloned().unwrap_or_default();
            worked.mark();
            made.set_worked(worked);
        }
        // The maker knows their work, and the type of what they made (a potion's).
        made.identify();
        self.learn_scroll(made.object);
        let name = self.lore().describe(&made);
        let pack = &mut self.player_mut().inventory;
        crate::object::add_to_inventory(pack, made, &content);
        self.say(match grade {
            Grade::Masterwork => format!("You have made {name}!"),
            Grade::Marvellous => format!("This is {name}!"),
            Grade::Fine => format!("This is {name}."),
            Grade::Plain | Grade::Failed => format!("You have made {name}."),
        });
    }
}

impl World {
    /// A recipe of profession `skill` failed and made `object` instead (a brew's murky draught,
    /// or on a natural 2 (`fumbled`) its brewer's blunder): the profession's line, then the
    /// object, known for what it is, into the pack.
    fn brew_spoilt(&mut self, skill: crate::skills::SkillId, object: ObjectId, fumbled: bool) {
        let content = self.content.clone();
        let text = content
            .recipes()
            .profession(skill)
            .and_then(|p| match fumbled {
                true => p.fumbled.clone().or(p.failed.clone()),
                false => p.failed.clone(),
            });
        self.say(text.unwrap_or_else(|| "The work goes wrong.".into()));
        let mut made = Stack::one(object);
        made.identify();
        self.learn_scroll(object);
        let name = self.lore().describe(&made);
        let pack = &mut self.player_mut().inventory;
        crate::object::add_to_inventory(pack, made, &content);
        self.say(format!("You are left with {name}."));
    }
}

/// Makes `recipe` with `picks` as a check of natural roll `natural` by `margin` would: the
/// outcome without the time or the draws of the check (a counted recipe still rolls its count).
#[cfg(test)]
pub(super) fn make_out_for_test(
    w: &mut World,
    recipe: &str,
    picks: &[Option<usize>],
    natural: i32,
    margin: i32,
) {
    let content = w.content.clone();
    let id = content.recipes().id(recipe).expect("recipe");
    let def = content.recipe(id);
    let level = w.state.current();
    let resolved =
        resolve(&content, w.lore(), level, w.state.player(), def, picks).expect("the picks hold");
    match resolved.batch {
        Some(_) => w.make_batch(def, resolved, natural, margin),
        None => w.make_out(resolved, natural, margin),
    }
}
