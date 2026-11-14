//! Cutting a gem (docs/design/recipes.md §5; docs/dev/crafting.md "Cutting a gem"): Jewelcraft's
//! built-in recipe Cut (`data/recipes/jewelcraft/cut.toml`, `use = "cut"`), made by
//! [`Action::Craft`] with one pick, the gem (`craft:cut:LETTER`), from its profession's screen
//! through the Cut list.
//!
//! - **What**: a loose gem carried ([`cuttable`]): any uncut one, and any cut one above the
//!   smallest size and below the finest cut. Refused, taking no time: a tiny cut gem ("There is
//!   nothing left to cut."), a gem of the finest cut ("It could not be finer."), anything else
//!   ("That is no loose gem."), a protected one, and unpaid shop goods (`may_loose`).
//! - **Time**: the recipe's `seconds`, in turns of the `craft` action ([`World::work`], as
//!   Temper): interrupted, nothing happens to the gem, but the time is spent.
//! - **The check**: done, two gameplay draws (the natural roll), plus the gem's profession's stat
//!   modifier (`data/stock.toml`: Dexterity), its talents' checks of that stat, and the recipe
//!   skill's own [`Effect::Craft`] ([`Actor::skill_effect_of`]), against the recipe's
//!   `difficulty` + the gem's `work`. The margin sets the cut ([`cut_band`]): below 0 failed,
//!   then a cut higher every `cut_step` (`data/recipes.toml`) from the roughest true cut, up to
//!   the finest. A natural 2 shatters the gem; a natural 24 cuts it at least to the plain cut.
//! - **The outcome** ([`cut_outcome`]): on an uncut gem, a failure leaves it uncut and a size
//!   smaller, the roughest cut a size smaller too, a better cut keeps the size. A re-cut (a gem
//!   already cut) takes the better of the old cut and the check's, and always a size. A gem that
//!   would go below the smallest size is lost. One gem of a stack is cut (set apart first).
//! - **Marks**: [`Use::Craft`] on the recipe's skill alone ([`World::train_skill`]), whatever came
//!   of it. No noise, no alignment.
//!
//! [`Action::Craft`]: crate::action::Action::Craft
//! [`Actor::skill_effect_of`]: super::Actor::skill_effect_of

use super::work::{Job, Task};
use super::{Spent, World};
use crate::action::Event;
use crate::combat;
use crate::content::{Content, CutId, Cutting, Gems, RecipeId};
use crate::item_parts::ItemPart;
use crate::object::{self, Stack};
use crate::skills::{Effect, Use};
use crate::stats::Stat;

/// What the check comes to, before the gem's own state is weighed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CutBand {
    /// A natural 2: the gem is lost.
    Shattered,
    /// The margin fell below 0.
    Failed,
    /// The cut the check reaches.
    Cut(CutId),
}

/// The cut a check of natural roll `natural` made by `margin` reaches, with bands of `step`
/// (the module docs).
pub fn cut_band(gems: &Gems, step: i32, natural: i32, margin: i32) -> CutBand {
    if natural == combat::FUMBLE {
        return CutBand::Shattered;
    }
    let cuts = gems.cuts();
    let first = roughest_true(gems);
    let top = cuts.len().saturating_sub(1);
    let by_margin = (margin >= 0).then(|| (first + (margin / step.max(1)) as usize).min(top));
    let reached = match natural == combat::HIGHEST {
        true => Some(by_margin.unwrap_or(0).max(usize::from(gems.plain().cut.0))),
        false => by_margin,
    };
    match reached {
        Some(i) => CutBand::Cut(CutId(i as u8)),
        None => CutBand::Failed,
    }
}

/// The index of the roughest true cut (crude): the first that is not `rough`.
fn roughest_true(gems: &Gems) -> usize {
    (gems.cuts().iter()).position(|c| !c.rough).unwrap_or(0)
}

/// What a gem cut as `from` comes to by `band`: `None` when it is lost (shattered, or a size
/// lost below the smallest), else its new cut and size (the module docs).
pub fn cut_outcome(gems: &Gems, from: Cutting, band: CutBand) -> Option<Cutting> {
    let rough = gems.cut(from.cut).rough;
    let crude = roughest_true(gems);
    let (cut, smaller) = match band {
        CutBand::Shattered => return None,
        CutBand::Failed => (from.cut, true),
        CutBand::Cut(cut) if rough => (cut, usize::from(cut.0) <= crude),
        // A re-cut never worsens the cut, and always takes stone.
        CutBand::Cut(cut) => (cut.max(from.cut), true),
    };
    let size = match smaller {
        true => crate::content::GemSizeId(from.size.0.checked_sub(1)?),
        false => from.size,
    };
    Some(Cutting { size, cut })
}

/// Whether `stack` is a loose gem the Cut list offers: uncut, or cut above the smallest size and
/// below the finest cut.
pub fn cuttable(content: &Content, stack: &Stack) -> bool {
    let (Some(gems), Some((_, cutting))) = (content.gems(), stack.gem(content)) else {
        return false;
    };
    refusal(gems, cutting).is_none()
}

/// Why a loose gem cut as `cutting` cannot be cut, if it cannot.
fn refusal(gems: &Gems, cutting: Cutting) -> Option<&'static str> {
    if gems.cut(cutting.cut).rough {
        return None;
    }
    if usize::from(cutting.cut.0) + 1 >= gems.cuts().len() {
        return Some("It could not be finer.");
    }
    (cutting.size.0 == 0).then_some("There is nothing left to cut.")
}

/// A cut under way: the recipe and the gem's position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GemWork {
    recipe: RecipeId,
    item: usize,
}

impl Job for GemWork {
    fn on(&self, _: &Content) -> Vec<Option<usize>> {
        vec![Some(self.item)]
    }

    fn found(&mut self, now: &[Option<usize>]) {
        self.item = super::work::one(now);
    }

    fn done(&self, w: &mut World, _: &mut Vec<Event>) {
        w.gem_cut(*self);
    }
}

impl World {
    /// The player begins to cut the gem `picks` names with Cut recipe `id`; refused, taking no
    /// time (module docs).
    pub(super) fn start_cut(&mut self, id: RecipeId, picks: &[Option<usize>]) -> Spent {
        let content = self.content.clone();
        let def = content.recipe(id);
        let item = match picks {
            [Some(item)] => *item,
            [] | [None] => {
                self.say("You must choose the gem.");
                return Spent::Nothing;
            }
            _ => {
                self.say("That is more than the work takes.");
                return Spent::Nothing;
            }
        };
        let me = self.state.player();
        if me.skills.value(def.skill).unwrap_or(0) < def.from {
            self.say("That is beyond your skill.");
            return Spent::Nothing;
        }
        let Some(stack) = me.inventory.get(item) else {
            self.say("You carry no such thing.");
            return Spent::Nothing;
        };
        let (Some(gems), Some((_, cutting))) = (content.gems(), stack.gem(&content)) else {
            self.say("That is no loose gem.");
            return Spent::Nothing;
        };
        if let Some(text) = refusal(gems, cutting) {
            self.say(text);
            return Spent::Nothing;
        }
        if self.item_protected(stack) {
            self.say("You dare not cut it.");
            return Spent::Nothing;
        }
        if !self.may_loose(item) {
            return Spent::Nothing;
        }
        let name = self.lore().title(&self.state.player().inventory[item]);
        self.say(format!("You set to cutting the {name}."));
        self.craft_work(Task::Cut(GemWork { recipe: id, item }), def.seconds)
    }

    /// The cut of `work` is done: the check, the gem's new state, the mark (module docs).
    fn gem_cut(&mut self, work: GemWork) {
        let content = self.content.clone();
        let def = content.recipe(work.recipe);
        let Some(gems) = content.gems() else {
            return;
        };
        let still = (self.state.player().inventory.get(work.item))
            .filter(|s| cuttable(&content, s))
            .and_then(|s| s.gem(&content));
        let Some((gem, cutting)) = still else {
            self.say("You have nothing left to work on.");
            return;
        };
        let stat = (self.profession_of(gem)).map_or(Stat::Dexterity, |(_, stat)| stat);
        let me = self.state.player();
        let modifier = me.check_modifier(&content, stat)
            + me.skill_effect_of(&content, def.skill, Effect::Craft)
            + self.binding_boon(def.skill);
        let against = def.difficulty + content.material(gem).work;
        let natural = combat::roll_natural(&mut self.state.rng);
        let step = (content.recipes().settings.as_ref()).map_or(5, |s| s.cut_step);
        // Master's hand: the cut comes sooner, never the making (`mastery.rs`).
        let margin = natural + modifier - against;
        let margin = super::mastery::graded_margin(&content, self.state.player(), margin);
        let band = cut_band(gems, step, natural, margin);
        let left = cut_outcome(gems, cutting, band);

        let at = self.set_apart(work.item);
        super::outcome::remove_stacks(self.player_mut(), &[at]);
        let gem_name = &content.material(gem).name;
        let recut = !gems.cut(cutting.cut).rough;
        let line = match (band, recut) {
            (CutBand::Shattered, _) => format!("The {gem_name} shatters."),
            // A tiny stone with a size to lose: nothing is left of it.
            (_, false) if left.is_none() => {
                "The stone crumbles under your tools; nothing worth keeping is left.".into()
            }
            (CutBand::Failed, false) => "The stone splits badly; what is left is smaller.".into(),
            (CutBand::Cut(c), false) if usize::from(c.0) <= roughest_true(gems) => {
                "You rough it out, and lose more than you meant to.".into()
            }
            (CutBand::Cut(_), false) => "You cut the stone.".into(),
            (CutBand::Cut(c), true) if c > cutting.cut => "You cut the stone again.".into(),
            (CutBand::Failed | CutBand::Cut(_), true) => {
                "You take off more stone and make it no finer.".into()
            }
        };
        self.say(line);
        if let Some(less) = left {
            let part = ItemPart::gem(&content, None, gem, less);
            let made = Stack::new(gems.object, 1).with_parts(vec![part]);
            let text = format!(
                "It is now {}.",
                object::phrase(std::slice::from_ref(&made), self.lore())
            );
            self.say(text);
            object::add_to_inventory(self.state.inventory_mut(), made, &content);
        }
        if self.trains(Use::Craft) {
            self.train_skill(Use::Craft, def.skill);
        }
    }
}
