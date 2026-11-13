//! Enchanting's recipes (docs/design/enchanting.md §7, §8; docs/dev/enchanting.md "The
//! recipes"): the built-in recipes of `data/recipes/enchanting/` (`use = "enchant"`, an
//! [`EnchantKind`] each), made by [`Action::Craft`] with the item's position, then a crystal's
//! for each the recipe takes (`craft:kindle:ITEM,CRYSTAL`, `craft:quicken:ITEM,CRYSTAL`,
//! `craft:unbind:ITEM`), from the profession's screen through the wizard's steps ([`steps`],
//! [`plan`]).
//!
//! - **What**: an item carried, not worn or wielded: a base item (one with a kind; ammunition is
//!   one, the whole stack worked at once), not a spirit weapon, its magic identified, not a
//!   unique ([`refusal`]); a quest's object is refused at the start ("It will not take your
//!   will."), unpaid goods as everywhere ([`World::may_loose`]). Then each kind's own: **Kindle**
//!   a plain item; **Quicken** a magic (not rare: a rare's takes a heart-crystal, a later phase)
//!   item with one affix and room for another (`magic::capacity`); **Unbind** a magic or rare
//!   item. No station, no tool: anywhere.
//! - **The colour**: the crystal's colour (`data/crystals.toml`) narrows the roll to its
//!   families ([`crate::magic::loot::roll_affix_within`]); with nothing left to roll there the
//!   work is refused before it starts ("No more of that nature will take.").
//! - **The item level of the roll**: the greater of the item's own and the enchanter's reach
//!   (Enchanting's [`Effect::EnchantLevel`], 1 to 12); Kindle's is the reach, Unbind's the
//!   item's own. The item takes the roll's level.
//! - **Time**: the recipe's `seconds` (Quick hands' share of it, `mastery::craft_seconds`), in
//!   turns of the `craft` action ([`World::keep_at`]): interrupted, nothing is spent, but the
//!   time is. No grade, so Master's hand has nothing to ease.
//! - **The check**: two gameplay draws (the natural roll), plus Willpower's modifier, its talents'
//!   checks, and Enchanting's [`Effect::Craft`], against the recipe's `difficulty` + the roll's
//!   item level ÷ `level_step` (`data/enchanting.toml`; [`enchant_band`]). Made (or a natural
//!   24): Kindle draws the kind (`[magic]`'s prefix and suffix weights among the kinds that can
//!   roll), then the affix, tier, and value; Quicken the affix of the kind the item lacks; Unbind
//!   draws nothing and takes the magic off. Failed: the crystals are spent, the item as it was. A
//!   natural 2: the crystals spent and the item warped ([`ItemHarm::Enchant`], as a failed
//!   Temper damages; of Proof, spared).
//! - **Known**: the magic made is identified: the enchanter knows their work.
//! - **Marks**: [`Use::Craft`] on the recipe's skill alone, whatever came of it. No noise, no
//!   alignment.
//!
//! [`Action::Craft`]: crate::action::Action::Craft

use super::ally::capital;
use super::craft::RecipeLack;
use super::craft_make::{CraftOption, CraftPlan, CraftStep, CraftStepKind};
use super::item_damage::Harmed;
use super::rest::Ended;
use super::{Actor, Spent, World};
use crate::action::Event;
use crate::combat;
use crate::content::{
    ActionKind, AffixKind, Content, EnchantKind, ItemHarm, Rarity, RecipeDef, RecipeId, RecipeUse,
};
use crate::magic::loot::{pick_weighted, roll_affix_within, rolls_within};
use crate::magic::{Magic, capacity};
use crate::object::Stack;
use crate::scroll_label::Lore;
use crate::skills::{Effect, MAX_ENCHANT_LEVEL, Use};
use crate::stats::Stat;
use crate::time::Duration;

/// What the check comes to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnchantBand {
    /// The work takes.
    Made,
    /// The crystals go dark; the item is as it was.
    Failed,
    /// A natural 2: the crystals go dark and the item warps.
    Warped,
}

/// The band of a check of natural roll `natural` made by `margin` (the module docs).
pub fn enchant_band(natural: i32, margin: i32) -> EnchantBand {
    match natural {
        combat::FUMBLE => EnchantBand::Warped,
        combat::HIGHEST => EnchantBand::Made,
        _ if margin >= 0 => EnchantBand::Made,
        _ => EnchantBand::Failed,
    }
}

/// The refusal of an item with no room for one more affix (the row's [`RecipeLack::Room`]).
const FULL: &str = "It holds all it can.";

/// The refusal of a roll with nothing of the crystal's families left to draw.
const NOTHING: &str = "No more of that nature will take.";

/// The words of what `kind` works on, the row's: "a plain item".
fn item_words(kind: EnchantKind) -> &'static str {
    match kind {
        EnchantKind::Kindle => "a plain item",
        EnchantKind::Quicken | EnchantKind::Unbind => "a magic item",
    }
}

/// What a recipe of `kind` taking `crystals` takes, before its time, in the row's words: "a
/// plain item, a crystal".
pub(super) fn takes(kind: EnchantKind, crystals: u8) -> Vec<String> {
    let mut words = vec![item_words(kind).to_string()];
    match crystals {
        0 => {}
        1 => words.push("a crystal".into()),
        n => words.push(format!("{n} crystals")),
    }
    words
}

/// Why `player`'s stack at `at` cannot take work of `kind`, if it cannot (the module docs; a
/// quest's object and unpaid goods are the start's).
pub(super) fn refusal(
    content: &Content,
    player: &Actor,
    at: usize,
    kind: EnchantKind,
) -> Option<&'static str> {
    let Some(stack) = player.inventory.get(at) else {
        return Some("You carry no such thing.");
    };
    let def = content.object(stack.object);
    if def.kind.is_none() || stack.is_spirit() {
        return Some("No magic can be bound in that.");
    }
    if !stack.is_identified() {
        return Some("You must know what is bound in it first.");
    }
    if player.equipment.holds(at) {
        return Some("Take it off first.");
    }
    let magic = stack.magic();
    if magic.is_some_and(|m| m.unique.is_some()) {
        return Some("It will not take your will.");
    }
    match (kind, magic) {
        (EnchantKind::Kindle, Some(_)) => Some("There is magic in it already."),
        (EnchantKind::Kindle, None) => None,
        (EnchantKind::Quicken | EnchantKind::Unbind, None) => Some("There is no magic in it."),
        (EnchantKind::Quicken, Some(m)) if m.rarity == Rarity::Rare => {
            Some("It is rare already: this work cannot add to it.")
        }
        (EnchantKind::Quicken, Some(m)) => {
            let full =
                m.affixes.len() >= 2 || m.affixes.len() >= usize::from(capacity(stack, content));
            full.then_some(FULL)
        }
        (EnchantKind::Unbind, Some(_)) => None,
    }
}

/// Whether `stack` is a crystal (the colours' objects; the rare materials are none).
fn is_crystal(content: &Content, stack: &Stack) -> bool {
    content
        .crystals()
        .is_some_and(|c| c.colour_of_object(stack.object).is_some())
}

/// The first reason `player` cannot work `recipe` now, past the skill: no item it may work, none
/// with room (Quicken's), no crystal.
pub(super) fn lack(content: &Content, player: &Actor, recipe: &RecipeDef) -> Option<RecipeLack> {
    let Some(RecipeUse::Enchant(kind)) = recipe.uses else {
        return None;
    };
    let why: Vec<Option<&str>> = (0..player.inventory.len())
        .map(|at| refusal(content, player, at, kind))
        .collect();
    if !why.contains(&None) {
        return Some(match why.contains(&Some(FULL)) {
            true => RecipeLack::Room,
            false => RecipeLack::Nothing,
        });
    }
    let crystal = (player.inventory.iter()).any(|s| is_crystal(content, s));
    (recipe.crystals > 0 && !crystal).then_some(RecipeLack::Crystal)
}

/// The wizard's steps of `recipe`: the item, then a crystal for each it takes.
pub(super) fn steps(
    content: &Content,
    lore: Lore<'_>,
    player: &Actor,
    recipe: &RecipeDef,
) -> Vec<CraftStep> {
    let Some(RecipeUse::Enchant(kind)) = recipe.uses else {
        return Vec::new();
    };
    let option = |at: usize, stack: &Stack, short: String| CraftOption {
        at,
        name: lore.listed(stack),
        short,
        takes: 1,
        each: false,
        enough: true,
    };
    let pack = player.inventory.iter().enumerate();
    let items = (pack.clone())
        .filter(|&(at, _)| refusal(content, player, at, kind).is_none())
        .map(|(at, s)| option(at, s, lore.title(s)))
        .collect::<Vec<_>>();
    let crystals = pack
        .filter(|(_, s)| is_crystal(content, s))
        .map(|(at, s)| option(at, s, lore.title(s)))
        .collect::<Vec<_>>();
    let mut steps = vec![CraftStep {
        kind: CraftStepKind::Item,
        label: "item".into(),
        optional: false,
        options: items,
    }];
    for _ in 0..recipe.crystals {
        steps.push(CraftStep {
            kind: CraftStepKind::Input,
            label: "crystal".into(),
            optional: false,
            options: crystals.clone(),
        });
    }
    steps
}

/// The picks checked: the item's position, the crystal's (none for Unbind), the item level of
/// the roll, and the kinds an affix may be drawn of (empty for Unbind).
struct Resolved {
    item: usize,
    crystal: Option<usize>,
    level: u8,
    families: Option<Vec<String>>,
    kinds: Vec<AffixKind>,
}

/// `picks` checked against `player`'s pack for `recipe` (the module docs), as the plan and the
/// start say why not.
fn resolve(
    content: &Content,
    player: &Actor,
    recipe: &RecipeDef,
    picks: &[Option<usize>],
) -> Result<Resolved, String> {
    let Some(RecipeUse::Enchant(kind)) = recipe.uses else {
        return Err("You know no such recipe.".into());
    };
    let want = 1 + usize::from(recipe.crystals);
    let item = match picks.first() {
        Some(Some(item)) => *item,
        _ => return Err("You must choose the item.".into()),
    };
    if picks.len() > want {
        return Err("That is more than the work takes.".into());
    }
    if picks.len() < want || picks[1..].contains(&None) {
        return Err("You must choose the crystal.".into());
    }
    if let Some(text) = refusal(content, player, item, kind) {
        return Err(text.into());
    }
    let stack = &player.inventory[item];
    let crystal = picks.get(1).copied().flatten();
    let colour = match crystal {
        Some(at) => {
            let stone = player.inventory.get(at).ok_or("You carry no such thing.")?;
            let crystals = content.crystals().ok_or("That is no crystal.")?;
            Some(
                crystals
                    .colour_of_object(stone.object)
                    .ok_or("That is no crystal.")?,
            )
        }
        None => None,
    };
    let own = stack.magic().map_or(0, |m| m.level);
    let reach = reach(content, player);
    let level = match kind {
        EnchantKind::Kindle => reach,
        EnchantKind::Quicken => own.max(reach),
        EnchantKind::Unbind => own,
    };
    let families = colour.map(|c| c.families.clone());
    let def = content.object(stack.object);
    let rolled: &[crate::magic::Affix] = stack.magic().map_or(&[], |m| &m.affixes);
    let kinds: Vec<AffixKind> = match kind {
        EnchantKind::Kindle => vec![AffixKind::Prefix, AffixKind::Suffix],
        EnchantKind::Quicken => lacking(content, rolled).into_iter().collect(),
        EnchantKind::Unbind => Vec::new(),
    };
    let kinds: Vec<AffixKind> = (kinds.into_iter())
        .filter(|&k| rolls_within(content, def, level, k, rolled, families.as_deref()))
        .collect();
    if kind.rolls() && kinds.is_empty() {
        return Err(NOTHING.into());
    }
    Ok(Resolved {
        item,
        crystal,
        level,
        families,
        kinds,
    })
}

/// The kind of affix a magic item with `rolled` lacks: a suffix beside its prefix, a prefix
/// beside its suffix.
fn lacking(content: &Content, rolled: &[crate::magic::Affix]) -> Option<AffixKind> {
    let kinds: Vec<AffixKind> = (rolled.iter())
        .filter_map(|a| content.affixes().get(&a.id).map(|d| d.kind))
        .collect();
    match (
        kinds.contains(&AffixKind::Prefix),
        kinds.contains(&AffixKind::Suffix),
    ) {
        (true, false) => Some(AffixKind::Suffix),
        (false, true) => Some(AffixKind::Prefix),
        _ => None,
    }
}

/// `player`'s reach: the item level their Enchanting rolls at, 1 to 12.
fn reach(content: &Content, player: &Actor) -> u8 {
    let skill = content.skill_id("enchanting");
    let value = skill.map_or(1, |s| {
        player.skill_effect_of(content, s, Effect::EnchantLevel)
    });
    u8::try_from(value.clamp(1, MAX_ENCHANT_LEVEL)).unwrap_or(1)
}

/// "the long sword", "the arrows": `stack` named as the work speaks of it.
/// A known rare goes by its name alone: "Grim Bite".
fn the(lore: Lore<'_>, stack: &Stack) -> String {
    let named = stack.known_magic().is_some_and(|m| m.name.is_some());
    match stack.count {
        1 if named => lore.title(stack),
        1 => format!("the {}", lore.title(stack)),
        _ => format!("the {}", lore.names(stack).1),
    }
}

/// What working `recipe` with `picks` would do and use, in words (the dialog's), or why not.
pub(super) fn plan(
    content: &Content,
    lore: Lore<'_>,
    player: &Actor,
    recipe: &RecipeDef,
    picks: &[Option<usize>],
) -> Result<CraftPlan, String> {
    let resolved = resolve(content, player, recipe, picks)?;
    let uses = (resolved.crystal)
        .map(|at| lore.describe(&player.inventory[at].single()))
        .unwrap_or_default();
    Ok(CraftPlan {
        verb: recipe.verb.clone().unwrap_or_else(|| recipe.name.clone()),
        name: the(lore, &player.inventory[resolved.item]),
        uses,
        station: None,
        time: format!(
            "about {}",
            Duration::seconds(u64::from(super::mastery::craft_seconds(
                content,
                player,
                recipe.seconds
            )))
            .spoken()
        ),
    })
}

/// An enchanting under way: the recipe, the item's and the crystal's positions, how long.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Enchanting {
    recipe: RecipeId,
    item: usize,
    crystal: Option<usize>,
    seconds: u32,
}

impl Enchanting {
    fn picks(self) -> Vec<Option<usize>> {
        let mut picks = vec![Some(self.item)];
        picks.extend(self.crystal.map(Some));
        picks
    }
}

impl World {
    /// The player begins to work Enchanting recipe `id` with `picks`; refused, taking no time
    /// (module docs).
    pub(super) fn start_enchant(&mut self, id: RecipeId, picks: &[Option<usize>]) -> Spent {
        let content = self.content.clone();
        let def = content.recipe(id);
        let me = self.state.player();
        if me.skills.value(def.skill).unwrap_or(0) < def.from {
            self.say("That is beyond your skill.");
            return Spent::Nothing;
        }
        let resolved = match resolve(&content, me, def, picks) {
            Ok(resolved) => resolved,
            Err(text) => {
                self.say(text);
                return Spent::Nothing;
            }
        };
        let item = resolved.item;
        if self
            .quest_objects()
            .contains(&self.state.player().inventory[item].object)
        {
            self.say("It will not take your will.");
            return Spent::Nothing;
        }
        if !self.may_loose(item) || !resolved.crystal.is_none_or(|c| self.may_loose(c)) {
            return Spent::Nothing;
        }
        let thing = the(self.lore(), &self.state.player().inventory[item]);
        self.say(format!("You take {thing} in your lap and begin."));
        Spent::Enchant(Enchanting {
            recipe: id,
            item,
            crystal: resolved.crystal,
            seconds: self.craft_seconds(def.seconds),
        })
    }

    /// Works turn after turn until the enchanting is done or something stops it (module docs):
    /// its last turn cut short to end exactly when its time is up.
    pub(super) fn enchant(&mut self, work: Enchanting, events: &mut Vec<Event>) {
        let until = self.state.time + Duration::seconds(u64::from(work.seconds));
        let turn = self.price_of(self.state.player, ActionKind::Craft);
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| turn.min(w.price_until(until)),
        );
        match ended {
            Ended::Died => {}
            _ if self.state.time >= until => self.enchanted(work),
            Ended::Came(came) => {
                let text = format!("{}; you put down your work.", self.came_line(came));
                self.say(text);
            }
            Ended::Said | Ended::Done => self.say("You put down your work."),
        }
    }

    /// The enchanting of `work` is done: the check, what comes of it, the mark.
    fn enchanted(&mut self, work: Enchanting) {
        let content = self.content.clone();
        let def = content.recipe(work.recipe);
        let picks = work.picks();
        let Ok(resolved) = resolve(&content, self.state.player(), def, &picks) else {
            self.say("You have nothing left to work on.");
            return;
        };
        let me = self.state.player();
        let modifier = me.effective_stats(&content).modifier(Stat::Willpower)
            + me.talent_mods(&content).check(Stat::Willpower)
            + me.skill_effect_of(&content, def.skill, Effect::Craft);
        let step = content.enchanting().map_or(3, |r| r.level_step.max(1));
        let against = def.difficulty + i32::from(resolved.level / step);
        let natural = combat::roll_natural(&mut self.state.rng);
        let band = enchant_band(natural, natural + modifier - against);
        self.enchant_out(def, &resolved, band);
        if self.trains(Use::Craft) {
            self.train_skill(Use::Craft, def.skill);
        }
    }

    /// What band `band` makes of the work `resolved` of recipe `def`: the item enchanted, or
    /// left, or warped; the crystal spent either way.
    fn enchant_out(&mut self, def: &RecipeDef, resolved: &Resolved, band: EnchantBand) {
        let Some(RecipeUse::Enchant(kind)) = def.uses else {
            return;
        };
        let content = self.content.clone();
        let at = resolved.item;
        let thing = the(self.lore(), &self.state.player().inventory[at]);
        let stone =
            (resolved.crystal).map(|c| self.lore().title(&self.state.player().inventory[c]));
        let mut crystal = resolved.crystal;
        match band {
            EnchantBand::Made => {
                let stack = self.state.player().inventory[at].clone();
                let made = self.worked(&content, kind, stack, resolved);
                let name = self.lore().describe(&made);
                self.player_mut().inventory[at] = made;
                let text = match (kind, &stone) {
                    (EnchantKind::Unbind, _) | (_, None) => {
                        format!("The magic goes out of {thing}. It is now {name}.")
                    }
                    (_, Some(stone)) => {
                        format!(
                            "{} takes the {stone}'s glow. It is now {name}.",
                            capital(&thing)
                        )
                    }
                };
                self.say(text);
            }
            EnchantBand::Failed => match kind {
                EnchantKind::Unbind => {
                    self.say(format!("{} clings to its magic.", capital(&thing)))
                }
                _ => self.say("The crystal goes dark in your hands."),
            },
            EnchantBand::Warped => {
                if stone.is_some() {
                    self.say("The crystal goes dark in your hands.");
                }
                match self.harm_item(at, ItemHarm::Enchant) {
                    Harmed::Withstood => {}
                    Harmed::Damaged => {
                        self.say(format!("{} warps under your hands.", capital(&thing)))
                    }
                    Harmed::Destroyed => {
                        self.say(format!("{} warps and falls apart.", capital(&thing)));
                        crystal = crystal.map(|c| if c > at { c - 1 } else { c });
                    }
                }
            }
        }
        if let Some(c) = crystal {
            let one = self.set_apart(c);
            super::outcome::remove_stacks(self.player_mut(), &[one]);
        }
    }

    /// `stack` after kind `kind`'s work, made (the affix draws: Kindle's kind, then each roll's
    /// affix, tier, and value).
    fn worked(
        &mut self,
        content: &Content,
        kind: EnchantKind,
        mut stack: Stack,
        resolved: &Resolved,
    ) -> Stack {
        let def = content.object(stack.object);
        let families = resolved.families.as_deref();
        let level = resolved.level;
        let rng = &mut self.state.rng;
        match kind {
            EnchantKind::Unbind => stack.set_magic(None),
            EnchantKind::Kindle => {
                let [prefix, suffix, _] = content.affixes().magic_affixes;
                let shapes: Vec<(AffixKind, u32)> =
                    [(AffixKind::Prefix, prefix), (AffixKind::Suffix, suffix)]
                        .into_iter()
                        .filter(|(k, _)| resolved.kinds.contains(k))
                        .collect();
                let affix = pick_weighted(rng, &shapes)
                    .copied()
                    .and_then(|k| roll_affix_within(content, rng, def, level, k, &[], families));
                if let Some(affix) = affix {
                    stack.set_magic(Some(Magic {
                        rarity: Rarity::Magic,
                        level,
                        affixes: vec![affix],
                        name: None,
                        identified: true,
                        carried: 0,
                        unique: None,
                    }));
                }
            }
            EnchantKind::Quicken => {
                let Some(mut magic) = stack.magic().cloned() else {
                    return stack;
                };
                let Some(&kind) = resolved.kinds.first() else {
                    return stack;
                };
                if let Some(affix) =
                    roll_affix_within(content, rng, def, level, kind, &magic.affixes, families)
                {
                    match kind {
                        AffixKind::Prefix => magic.affixes.insert(0, affix),
                        AffixKind::Suffix => magic.affixes.push(affix),
                    }
                    magic.level = level;
                    magic.identified = true;
                    stack.set_magic(Some(magic));
                }
            }
        }
        stack
    }
}

/// What band `band` makes of working recipe `recipe` with `picks` (the check skipped).
#[cfg(test)]
pub(super) fn enchant_out_for_test(
    w: &mut World,
    recipe: &str,
    picks: &[Option<usize>],
    band: EnchantBand,
) {
    let content = w.content.clone();
    let def = content.recipe(content.recipes().id(recipe).expect("recipe"));
    let resolved = resolve(&content, w.state.player(), def, picks).expect("the picks hold");
    w.enchant_out(def, &resolved, band);
}

/// The object a crystal of colour `id` is (tests).
#[cfg(test)]
pub(super) fn crystal_object(content: &Content, id: &str) -> crate::content::ObjectId {
    content
        .crystals()
        .and_then(|c| c.colour(id))
        .expect("colour")
        .object
}
