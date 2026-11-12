//! Making a generic recipe (spec §8 "Skills": professions; docs/design/professions.md §7, §8,
//! docs/design/recipes.md §4, §6; docs/dev/crafting.md "Making a recipe"): the forge's weapons and
//! the hand-made ones.
//!
//! - **The steps** ([`craft_steps`], what the wizard lists, through
//!   [`PerceptionView::craft_steps`]): one choice for each `"stock"` part of the result, in part
//!   order (the pieces whose material the part may be made of and the profession works, each
//!   with the count it would take), each input (the stacks of an object input; the pieces of a
//!   stock input's material or group: a bow's sinew, a crossbow's ingot of any metal), then the
//!   ornament if the recipe offers one (the cut loose gems carried; None may be chosen). A step
//!   with one fitting stack is still a step: the player sees what is spent.
//! - **The plan** ([`craft_plan`]): the picks, inventory positions in step order (`None` a
//!   skipped optional), checked by the rules (the stack exists, fits, and suffices; an ornament
//!   is a cut gem), and what comes of them: the item to be made, named as it would be ("a
//!   ruby-pommelled long sword"), and what it uses in words. The dialog asks with it, and
//!   [`World::start_making`] refuses with its reason, taking no time.
//! - **The work**: the recipe's `seconds`, in turns of [`ActionKind::Craft`]
//!   ([`World::keep_at`]); an interruption costs only the time. Done, the plan is made again
//!   from the pack ("You have nothing left to work with." if it no longer holds), then one check
//!   (two gameplay draws, the natural roll): the recipe's profession's stat modifier (its
//!   directory's: a flint knife is Woodcraft's, with Dexterity), what the skill gives
//!   of [`Effect::Craft`] (that skill's alone), and the talents' checks of the stat, against the
//!   recipe's `difficulty` plus the `work` of the main chosen part's material. The margin's
//!   [`Grade`] by `data/recipes.toml`'s bands: a Masterwork, Marvellous, Fine (three, two, one
//!   improvements, each an [`crate::magic::Aspect`] of the item drawn as Temper draws it, one
//!   gameplay draw each, and `attempts` equal to them, so the anvil goes on from there), plain,
//!   or failed; a natural 24 makes at least Fine, a natural 2 fails. An item with no aspect is
//!   plain whatever the margin. Made, the stock and inputs are spent, the gem set, and the item
//!   goes into the pack identified; failed, nothing is made, the gem is kept, the stock parts
//!   come back at the settings' `yield` (pieces by [`crate::piece::count_for`], at least one of
//!   each), and the inputs at `count × yield / 100` (a bow's one sinew is lost). Either way the skill may earn a mark ([`Use::Craft`]).
//!
//! Only stock parts (`"stock"`, or a table of `stock` alone: a spear's haft of another profession's
//! wood), inputs, and the ornament have steps yet: a recipe needing any other part source, a
//! count, a failure or fumble object, or a brittle result
//! is refused ("You cannot make that yet.") until the phase that builds its step.
//!
//! [`PerceptionView::craft_steps`]: crate::perception::PerceptionView::craft_steps
//! [`ActionKind::Craft`]: crate::content::ActionKind::Craft
//! [`Effect::Craft`]: crate::skills::Effect::Craft
//! [`Use::Craft`]: crate::skills::Use::Craft

use super::craft::{RecipeLack, input_fits, lack, stack_material, stock_only, worked_materials};
use super::rest::Ended;
use super::{Actor, Spent, World};
use crate::action::Event;
use crate::combat;
use crate::content::{
    ActionKind, Content, InputKind, MaterialId, RecipeCount, RecipeDef, RecipeId, RecipeSettings,
    counted,
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
    /// An object used up.
    Input,
    /// A cut gem for the result's ornament part, or none.
    Ornament,
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
    /// How many of it the recipe would take.
    pub takes: u32,
    /// Whether the stack holds that many.
    pub enough: bool,
}

/// One step of the wizard: what it chooses, for which part, and the stacks it may take.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CraftStep {
    pub kind: CraftStepKind,
    /// What it is for: the part ("blade", "pommel") or the input ("arrowheads").
    pub label: String,
    /// None may be chosen (the ornament).
    pub optional: bool,
    pub options: Vec<CraftOption>,
}

/// What a recipe with the picks chosen would make and use, in words (the dialog's).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CraftPlan {
    /// The profession's verb: "Forge".
    pub verb: String,
    /// The item, as it would be named: "a ruby-pommelled long sword".
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

/// The bands and the yield: `data/recipes.toml`'s, else [`BANDS`].
fn bands(settings: Option<&RecipeSettings>) -> (i32, i32, i32, u32) {
    settings.map_or(BANDS, |s| {
        (s.fine, s.marvellous, s.masterwork, s.yield_percent)
    })
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

/// A step of a recipe: which of its parts or inputs, or the ornament.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    /// The part at this index of the result's parts.
    Part(usize),
    /// The input at this index of the recipe's.
    Input(usize),
    /// The result's ornament part.
    Ornament(crate::content::PartId),
}

/// The steps of `recipe` in the wizard's order; `None` for a recipe this phase cannot make
/// (the module docs).
fn steps(content: &Content, recipe: &RecipeDef) -> Option<Vec<Step>> {
    let result = recipe.result.as_ref()?;
    let one = matches!(result.count, RecipeCount::Dice(d) if d.max() == 1 && d.min() == 1);
    if !one || recipe.failure.is_some() || recipe.fumble.is_some() || recipe.brittle.is_some() {
        return None;
    }
    let mut steps = Vec::new();
    for part in &recipe.parts {
        if !stock_only(&part.source) {
            return None;
        }
        steps.push(Step::Part(part.part));
    }
    for i in 0..recipe.inputs.len() {
        steps.push(Step::Input(i));
    }
    if recipe.ornament {
        let def = content.object(result.object);
        let placed = (content.gems()).and_then(|g| g.ornament(def.class));
        steps.push(Step::Ornament(def.ornament.or(placed.map(|o| o.part))?));
    }
    (steps.len() <= MAX_PICKS).then_some(steps)
}

/// The pieces of `material` part `part` of `recipe`'s result takes: its share of the result's
/// weight, scaled by the material's weight over the base part's as the made item weighs it, in
/// pieces ([`crate::piece::count_for`]); with the grams, for a failure's return.
pub(super) fn stock_need(
    content: &Content,
    recipe: &RecipeDef,
    part: usize,
    material: MaterialId,
) -> Option<(u32, u64)> {
    let def = content.object(recipe.result.as_ref()?.object);
    let base = def.parts.get(part)?;
    let weight = |m: MaterialId| u64::from(content.material(m).weight).max(1);
    let grams = u64::from(def.weight) * u64::from(base.share) * weight(material)
        / weight(base.material)
        / 100;
    Some((crate::piece::count_for(content, grams, material)?, grams))
}

/// The cut loose gem of `stack`, if it is one (an uncut one is never set).
fn cut_gem(content: &Content, stack: &Stack) -> Option<(MaterialId, crate::content::Cutting)> {
    let gems = content.gems()?;
    if stack.object != gems.object {
        return None;
    }
    let (gem, cutting) = stack.gem(content)?;
    (!gems.cut(cutting.cut).rough).then_some((gem, cutting))
}

/// What a step is for, in words: the part's name ("blade", "pommel"; a main part without one by
/// the group of the stock it takes, "metal"), or the input's plural.
fn label(content: &Content, recipe: &RecipeDef, step: Step) -> String {
    let Some(result) = recipe.result.as_ref() else {
        return String::new();
    };
    let def = content.object(result.object);
    let part_name = |part: Option<crate::content::PartId>| {
        part.map(|p| content.part_defs()[usize::from(p.0)].name.clone())
    };
    match step {
        Step::Part(i) => (def.parts.get(i))
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
    }
}

/// The wizard's steps of `recipe` for `player` (the module docs); empty for a recipe this phase
/// cannot make.
pub(crate) fn craft_steps(
    content: &Content,
    lore: Lore<'_>,
    player: &Actor,
    recipe: RecipeId,
) -> Vec<CraftStep> {
    let def = content.recipe(recipe);
    let Some(steps) = steps(content, def) else {
        return Vec::new();
    };
    let Some(result) = def.result.as_ref() else {
        return Vec::new();
    };
    let pack = &player.inventory;
    let option = |at: usize, stack: &Stack, takes: u32| CraftOption {
        at,
        name: lore.listed(stack),
        short: match stack_material(content, stack).or(cut_gem(content, stack).map(|g| g.0)) {
            Some(m) => content.material(m).name.clone(),
            None => lore.title(stack),
        },
        takes,
        enough: stack.count >= takes,
    };
    steps
        .into_iter()
        .map(|step| {
            let (kind, optional, options) = match step {
                Step::Part(part) => {
                    let worked = worked_materials(content, def, result.object, part);
                    let options = (pack.iter().enumerate())
                        .filter_map(|(at, stack)| {
                            let m = stack_material(content, stack)?;
                            let (need, _) = stock_need(content, def, part, m)?;
                            worked.contains(&m).then(|| option(at, stack, need))
                        })
                        .collect();
                    (CraftStepKind::Stock, false, options)
                }
                Step::Input(i) => {
                    let input = &def.inputs[i];
                    let options = (pack.iter().enumerate())
                        .filter(|(_, s)| input_fits(content, input, s))
                        .map(|(at, stack)| option(at, stack, input.count))
                        .collect();
                    (CraftStepKind::Input, false, options)
                }
                Step::Ornament(_) => {
                    let options = (pack.iter().enumerate())
                        .filter(|(_, s)| cut_gem(content, s).is_some())
                        .map(|(at, stack)| option(at, stack, 1))
                        .collect();
                    (CraftStepKind::Ornament, true, options)
                }
            };
            CraftStep {
                kind,
                label: label(content, def, step),
                optional,
                options,
            }
        })
        .collect()
}

/// What a stack chosen gives up: how many, and how many come back if the work fails.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Spend {
    at: usize,
    count: u32,
    /// What a failure gives back of it (the gem: all of it).
    back: u32,
}

/// A plan checked against the pack: the item it makes, what it spends, the main material.
#[derive(Clone, Debug)]
struct Resolved {
    made: Stack,
    spends: Vec<Spend>,
    /// The main chosen part's material: its `work` is on the difficulty.
    main: Option<MaterialId>,
}

/// `recipe` made by `player` with `picks` (the module docs): what it makes and spends, or why
/// not, as the rules say it.
fn resolve(
    content: &Content,
    player: &Actor,
    recipe: &RecipeDef,
    picks: &[Option<usize>],
) -> Result<Resolved, String> {
    let not_yet = || "You cannot make that yet.".to_string();
    let steps = steps(content, recipe).ok_or_else(not_yet)?;
    let result = recipe.result.as_ref().ok_or_else(not_yet)?;
    let def = content.object(result.object);
    if picks.len() > steps.len() {
        return Err("That is more than the work takes.".into());
    }
    let yield_percent = bands(content.recipes().settings.as_ref()).3;
    let pack = &player.inventory;
    let mut own: Vec<ItemPart> = Vec::new();
    let mut spends: Vec<Spend> = Vec::new();
    let mut main = None;
    for (n, &step) in steps.iter().enumerate() {
        let pick = picks.get(n).copied().flatten();
        let what = label(content, recipe, step);
        let Some(at) = pick else {
            if matches!(step, Step::Ornament(_)) {
                continue;
            }
            return Err(format!("You must choose the {what}."));
        };
        let stack = pack.get(at).ok_or("You carry no such thing.")?;
        match step {
            Step::Part(part) => {
                let worked = worked_materials(content, recipe, result.object, part);
                let material = (stack_material(content, stack))
                    .filter(|m| worked.contains(m))
                    .ok_or_else(|| format!("That will not do for the {what}."))?;
                let (count, grams) =
                    stock_need(content, recipe, part, material).ok_or_else(not_yet)?;
                let back = crate::piece::count_for(
                    content,
                    grams * u64::from(yield_percent) / 100,
                    material,
                )
                .unwrap_or(0)
                .min(count);
                spends.push(Spend { at, count, back });
                main = main.or(Some(material));
                if def.parts[part].material != material {
                    let id = (part > 0).then_some(def.parts[part].part).flatten();
                    own.push(ItemPart::material(content, id, material));
                }
            }
            Step::Input(i) => {
                let input = &recipe.inputs[i];
                if !input_fits(content, input, stack) {
                    return Err(format!("That will not do for the {what}."));
                }
                let back = input.count * yield_percent / 100;
                spends.push(Spend {
                    at,
                    count: input.count,
                    back,
                });
            }
            Step::Ornament(part) => {
                let (gem, cutting) = (cut_gem(content, stack))
                    .ok_or_else(|| "Only a cut gem can be set.".to_string())?;
                spends.push(Spend {
                    at,
                    count: 1,
                    back: 1,
                });
                own.push(ItemPart::gem(content, Some(part), gem, cutting));
            }
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
    let mut made = Stack::one(result.object);
    if !own.is_empty() {
        item_parts::check(def, &own, content).map_err(|_| not_yet())?;
        made = made.with_parts(own);
    }
    Ok(Resolved { made, spends, main })
}

/// What `recipe` with `picks` would make and use, in words (the module docs), or why not.
pub(crate) fn craft_plan(
    content: &Content,
    lore: Lore<'_>,
    player: &Actor,
    recipe: RecipeId,
    picks: &[Option<usize>],
) -> Result<CraftPlan, String> {
    let def = content.recipe(recipe);
    let resolved = resolve(content, player, def, picks)?;
    let pack = &player.inventory;
    let mut uses: Vec<String> = Vec::new();
    for spend in &resolved.spends {
        let stack = &pack[spend.at];
        uses.push(match cut_gem(content, stack) {
            Some(_) => format!("the {}", lore.title(stack)),
            None => {
                let (one, many) = lore.names(stack);
                counted(&one, &many, spend.count)
            }
        });
    }
    let verb = profession(content, def, resolved.main)
        .map_or_else(|| "Make".to_string(), |p| p.verb.clone());
    Ok(CraftPlan {
        verb,
        name: lore.describe(&resolved.made),
        uses: joined(&uses),
        station: def.station.map(|t| content.terrain(t).name.clone()),
        time: format!(
            "about {}",
            Duration::seconds(u64::from(def.seconds)).spoken()
        ),
    })
}

/// The profession `recipe` is made by: its own skill's (the directory's: a flint knife is
/// Woodcraft's, a steel one Metalcraft's), else the main material's.
fn profession<'c>(
    content: &'c Content,
    recipe: &RecipeDef,
    main: Option<MaterialId>,
) -> Option<&'c crate::content::Profession> {
    let stock = content.stock();
    (stock.profession(recipe.skill)).or_else(|| stock.profession_of(content, main?))
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
    /// taking no time, for a recipe beyond them or this phase, away from its station, or with
    /// picks that do not hold.
    pub(super) fn start_making(&mut self, id: RecipeId, picks: &[Option<usize>]) -> Spent {
        let content = self.content.clone();
        let def = content.recipe(id);
        let me = self.state.player();
        let level = self.state.current();
        let refused = match lack(&content, level, me, def) {
            Some(RecipeLack::Skill) => Some("That is beyond your skill.".to_string()),
            Some(RecipeLack::Station(t)) => {
                Some(format!("There is no {} here.", content.terrain(t).name))
            }
            Some(RecipeLack::Knife) => Some("You have no knife.".into()),
            Some(RecipeLack::NotYet | RecipeLack::Launcher) => {
                Some("You cannot make that yet.".into())
            }
            // The picks say what is missing.
            Some(RecipeLack::Stock(_) | RecipeLack::Input(_)) | None => None,
        };
        let refused = refused.or_else(|| resolve(&content, me, def, picks).err());
        if let Some(text) = refused {
            self.say(text);
            return Spent::Nothing;
        }
        let mut held = [None; MAX_PICKS];
        held[..picks.len()].copy_from_slice(picks);
        let at = def.station.map(|t| content.terrain(t).name.clone());
        let name = self
            .lore()
            .title(&resolve(&content, me, def, picks).expect("checked").made);
        self.say(match at {
            Some(station) => format!("You set to work on the {name} at the {station}."),
            None => format!("You set to work on the {name}."),
        });
        Spent::Craft(Crafting {
            recipe: id,
            picks: held,
            len: picks.len() as u8,
            seconds: def.seconds,
        })
    }

    /// Works turn after turn until the work is done or something stops it (module docs): its
    /// last turn cut short to end exactly when its time is up.
    pub(super) fn make(&mut self, work: Crafting, events: &mut Vec<Event>) {
        let until = self.state.time + Duration::seconds(u64::from(work.seconds));
        let turn = self.price_of(self.state.player, ActionKind::Craft);
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| turn.min(w.price_until(until)),
        );
        match ended {
            Ended::Died => {}
            _ if self.state.time >= until => self.crafted(work),
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
        let stat = profession(content, def, main).map_or(Stat::Strength, |p| p.stat);
        let me = self.state.player();
        me.effective_stats(content).modifier(stat)
            + me.skill_effect_of(content, def.skill, Effect::Craft)
            + me.talent_mods(content).check(stat)
    }

    /// The work of `work` is done: the plan again, the check, what comes of it, the mark.
    fn crafted(&mut self, work: Crafting) {
        let content = self.content.clone();
        let def = content.recipe(work.recipe);
        let Ok(resolved) = resolve(&content, self.state.player(), def, work.picks()) else {
            self.say("You have nothing left to work with.");
            return;
        };
        let modifier = self.craft_modifier(def, resolved.main);
        let natural = combat::roll_natural(&mut self.state.rng);
        let work_of = resolved.main.map_or(0, |m| content.material(m).work);
        let margin = natural + modifier - (def.difficulty + work_of);
        self.make_out(resolved, natural, margin);
        if self.trains(Use::Craft) {
            self.train_skill(Use::Craft, def.skill);
        }
    }

    /// What came of a check of natural roll `natural` made by `margin` on `resolved`: the item
    /// made at its grade and the stock spent, or the stock given back in part.
    fn make_out(&mut self, resolved: Resolved, natural: i32, margin: i32) {
        let content = self.content.clone();
        let settings = content.recipes().settings.as_ref();
        let grade = grade(settings, natural, margin);
        let Resolved {
            mut made,
            spends,
            main,
        } = resolved;
        // Highest position first, so a stack used up leaves the others where they were.
        let mut spends = spends;
        spends.sort_by_key(|s| std::cmp::Reverse(s.at));
        let failed = grade == Grade::Failed;
        for spend in spends {
            let gone = if failed {
                spend.count - spend.back
            } else {
                spend.count
            };
            let pack = &mut self.player_mut().inventory;
            pack[spend.at].count -= gone;
            if pack[spend.at].count == 0 {
                pack.remove(spend.at);
            }
        }
        if failed {
            let metal = (main.and_then(|m| content.material(m).group.as_deref())) == Some("metal");
            self.say(match metal {
                true => "The blade comes out brittle and you break it up for the metal.",
                false => "The work goes wrong, and you save what you can of it.",
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
        // The maker knows their work.
        made.identify();
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
    let resolved = resolve(&content, w.state.player(), def, picks).expect("the picks hold");
    w.make_out(resolved, natural, margin);
}
