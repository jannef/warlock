//! Salvage: breaking an item into the materials it is made of (docs/design/professions.md §5;
//! docs/dev/crafting.md): [`Action::Salvage`], the Salvage ability of the abilities menu's
//! Skills group, had by everyone; the numbers are `data/salvage.toml`.
//!
//! - **Where, with what**: anywhere, with nothing.
//! - **What**: a stack carried with at least one part of a material a profession works
//!   ([`salvageable`]: `data/stock.toml`, [`World::profession_of`]), not a spirit weapon.
//!   Refused, taking no time: one worn or wielded ("Take it off first."), one protected from
//!   harm ([`World::item_protected`]: a quest's, a unique, of Proof; "It will not come apart."),
//!   unpaid shop goods (`may_loose`), and one with nothing a profession works.
//! - **Time**: `seconds` (ten minutes), in turns of the `salvage` action ([`World::keep_at`], as
//!   Temper): interrupted, nothing happens to the item, but the time is spent.
//! - **The checks**: done, one check for each distinct material among the item's resolved parts
//!   ([`Stack::parts`], gems included) that a profession works, in first-seen order: two gameplay
//!   draws each, the natural roll plus the profession's stat modifier, the talents' checks of
//!   that stat, and the profession's own [`Effect::Craft`] ([`Actor::skill_effect_of`]), against
//!   `difficulty` + the material's `work`. A natural 2 always fails, a natural 24 always succeeds.
//!   Untrained, the check is made all the same.
//! - **The outcome**: the one item (a stack of several gives up one, set apart first, as Temper
//!   sets it apart) is destroyed whatever comes of the checks: its magic, improvements, rung, and
//!   charges are gone. A made material gives its stock, `yield` % of its grams
//!   ([`crate::piece::break_up`]); a gem comes out whole, its size and cut kept. A failed
//!   material is lost, but a failed gem only drops a cut (while above the roughest true cut) or
//!   else a size (while above the smallest) ([`chipped`]); a natural 2, or a gem with nothing to
//!   lose, shatters it. A material no profession works gives nothing and is not mentioned.
//! - **Magic** (docs/design/enchanting.md §10; `world/crystal.rs`, [`World::catch_magic`]):
//!   before the material checks, an identified magic or rare item makes one Enchanting check
//!   per affix, each made a crystal of the affix's colour among what is saved, and a rare of
//!   item level 10 or more a small chance of a rare material; an unidentified one gives nothing.
//! - **Marks**: [`Use::Salvage`] for each profession rolled, on that skill alone
//!   ([`World::train_skill`]), whatever came of it, and on Enchanting when an affix check was
//!   made. No noise, no alignment.
//!
//! [`Action::Salvage`]: crate::action::Action::Salvage
//! [`Actor::skill_effect_of`]: super::Actor::skill_effect_of

use super::rest::Ended;
use super::{Spent, World};
use crate::action::Event;
use crate::combat;
use crate::content::{ActionKind, Content, Cutting, Gems, MaterialId};
use crate::item_parts::ItemPart;
use crate::object::{self, Stack};
use crate::skills::{Effect, SkillId, Use};
use crate::stats::Stat;
use crate::time::Duration;

/// Whether `stack` has anything to salvage: not a spirit weapon, with a part of a material some
/// profession works, and not money (coins are never melted down: gold ingots come from salvaged
/// gold, never from coins, docs/design/recipes.md). Worn, protected, and unpaid stacks are the
/// rules' to refuse.
pub fn salvageable(content: &Content, stack: &Stack) -> bool {
    let money = content.shops().is_some_and(|s| s.currency == stack.object);
    !stack.is_spirit()
        && !money
        && (stack.parts(content).iter())
            .any(|p| content.stock().profession_of(content, p.material).is_some())
}

/// What a failed check leaves of a gem cut as `cutting`: one cut rougher while it is above the
/// roughest true cut (crude), else one size smaller while above the smallest; `None` when it has
/// neither to lose (a tiny crude or uncut stone), and it is lost.
pub fn chipped(gems: &Gems, cutting: Cutting) -> Option<Cutting> {
    let cut = usize::from(cutting.cut.0);
    if cut > 0 && !gems.cuts()[cut - 1].rough {
        let mut less = cutting;
        less.cut.0 -= 1;
        return Some(less);
    }
    (cutting.size.0 > 0).then(|| {
        let mut less = cutting;
        less.size.0 -= 1;
        less
    })
}

/// An attempt under way: the stack taken apart and how long it takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Salvaging {
    item: usize,
    seconds: u32,
}

/// One material's check: the material, the profession, whether it was made, and the natural
/// roll.
struct Check {
    material: MaterialId,
    skill: SkillId,
    made: bool,
    natural: i32,
}

impl World {
    /// The player begins to take the stack at `item` apart; refused, taking no time (module docs).
    pub(super) fn start_salvage(&mut self, item: usize) -> Spent {
        let Some(rules) = self.content.salvage().copied() else {
            return Spent::Nothing;
        };
        let Some(stack) = self.state.player().inventory.get(item) else {
            self.say("You carry no such thing.");
            return Spent::Nothing;
        };
        if !salvageable(&self.content, stack) {
            self.say("There is nothing in it worth saving.");
            return Spent::Nothing;
        }
        if self.state.player().equipment.holds(item) {
            self.say("Take it off first.");
            return Spent::Nothing;
        }
        if self.item_protected(stack) {
            self.say("It will not come apart.");
            return Spent::Nothing;
        }
        if !self.may_loose(item) {
            return Spent::Nothing;
        }
        let stack = &self.state.player().inventory[item];
        let name = self.lore().title(stack);
        self.say(format!("You set to taking the {name} apart."));
        Spent::Salvage(Salvaging {
            item,
            seconds: rules.seconds,
        })
    }

    /// Works turn after turn until the attempt is done or something stops it (module docs): its
    /// last turn cut short to end exactly when its time is up.
    pub(super) fn salvage(&mut self, work: Salvaging, events: &mut Vec<Event>) {
        let until = self.state.time + Duration::seconds(u64::from(work.seconds));
        let turn = self.price_of(self.state.player, ActionKind::Salvage);
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| turn.min(w.price_until(until)),
        );
        match ended {
            Ended::Died => {}
            _ if self.state.time >= until => self.salvaged(work.item),
            Ended::Came(came) => {
                let text = format!("{}; you leave off your work.", self.came_line(came));
                self.say(text);
            }
            Ended::Said | Ended::Done => self.say("You leave off your work."),
        }
    }

    /// The attempt on the stack at `item` is done: the checks, the stock, the item gone, the
    /// marks (module docs).
    fn salvaged(&mut self, item: usize) {
        let content = self.content.clone();
        let Some(rules) = content.salvage().copied() else {
            return;
        };
        let me = self.state.player();
        let still = (me.inventory.get(item)).filter(|s| {
            salvageable(&content, s) && !me.equipment.holds(item) && !self.item_protected(s)
        });
        let Some(stack) = still else {
            self.say("You have nothing left to work on.");
            return;
        };
        let stack = stack.single();
        // Enchanting's checks on its affixes first (`crystal.rs`), drawn before the materials'.
        let caught = self.catch_magic(&stack);
        // Each distinct material a profession works, in first-seen order.
        let mut works: Vec<(MaterialId, SkillId, Stat)> = Vec::new();
        for part in stack.parts(&content).iter() {
            if works.iter().any(|&(m, _, _)| m == part.material) {
                continue;
            }
            if let Some((skill, stat)) = self.profession_of(part.material) {
                works.push((part.material, skill, stat));
            }
        }
        let mut checks = Vec::with_capacity(works.len());
        for (material, skill, stat) in works {
            let me = self.state.player();
            let modifier = me.effective_stats(&content).modifier(stat)
                + me.talent_mods(&content).check(stat)
                + me.skill_effect_of(&content, skill, Effect::Craft);
            let against = rules.difficulty + content.material(material).work;
            let natural = combat::roll_natural(&mut self.state.rng);
            let made = crate::check::makes(natural, modifier, against);
            checks.push(Check {
                material,
                skill,
                made,
                natural,
            });
        }
        let at = self.set_apart(item);
        let one = self.state.player().inventory[at].clone();
        super::outcome::remove_stacks(self.player_mut(), &[at]);
        let name = self.lore().title(&one);
        self.say(format!("You take the {name} apart."));
        for line in caught.lines {
            self.say(line);
        }
        // What it falls to, each fall by its material, kept, chipped, or lost by its check.
        let mut saved: Vec<Stack> = Vec::new();
        for fall in crate::piece::break_up(&one, &content, rules.percent) {
            let Some(material) = fall.parts(&content).first().map(|p| p.material) else {
                continue;
            };
            let Some(check) = checks.iter().find(|c| c.material == material) else {
                continue;
            };
            if check.made {
                saved.push(fall);
                continue;
            }
            let gem_name = &content.material(material).name;
            match fall.gem(&content) {
                Some((gem, cutting)) => {
                    let gems = content.gems().expect("a loose gem has gems");
                    let left = (check.natural != combat::FUMBLE)
                        .then(|| chipped(gems, cutting))
                        .flatten();
                    match left {
                        Some(less) => {
                            let part = ItemPart::gem(&content, None, gem, less);
                            let chipped =
                                Stack::new(gems.object, fall.count).with_parts(vec![part]);
                            self.say(format!("The {gem_name} chips."));
                            saved.push(chipped);
                        }
                        None => self.say(format!("The {gem_name} shatters.")),
                    }
                }
                None => {
                    let what = self.part_word(&one, material);
                    self.say(format!("The {what} cracks and is good for nothing."));
                }
            }
        }
        saved.extend(caught.stacks);
        if saved.is_empty() {
            self.say("Nothing is left worth keeping.");
        } else {
            let text = format!("You save {}.", object::phrase(&saved, self.lore()));
            self.say(text);
            for stack in saved {
                object::add_to_inventory(self.state.inventory_mut(), stack, &content);
            }
        }
        let mut practised: Vec<SkillId> = Vec::new();
        if let Some(enchanting) = (content.skill_id("enchanting")).filter(|_| caught.checked) {
            practised.push(enchanting);
            self.train_skill(Use::Salvage, enchanting);
        }
        for check in &checks {
            if !practised.contains(&check.skill) {
                practised.push(check.skill);
                self.train_skill(Use::Salvage, check.skill);
            }
        }
    }

    /// What the failed lines call the part of `stack` made of `material`: its first part of it
    /// by name ("blade"), else the material ("hardwood").
    fn part_word(&self, stack: &Stack, material: MaterialId) -> String {
        let content = &self.content;
        (stack.parts(content).iter())
            .find(|p| p.material == material)
            .and_then(|p| p.part)
            .map(|id| content.part_defs()[usize::from(id.0)].name.clone())
            .unwrap_or_else(|| content.material(material).name.clone())
    }
}
