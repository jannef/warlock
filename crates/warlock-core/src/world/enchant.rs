//! Enchanting's recipes (docs/design/enchanting.md §7, §8; docs/dev/enchanting.md "The
//! recipes"): the built-in recipes of `data/recipes/enchanting/` (`use = "enchant"`, an
//! [`EnchantKind`] each), made by [`Action::Craft`] with the item's position, then a crystal's
//! for each the recipe takes (`craft:kindle:ITEM,CRYSTAL`, `craft:quicken:ITEM,CRYSTAL`,
//! `craft:recast:ITEM,CRYSTAL`, `craft:imbue:ITEM,C1,C2,C3`, `craft:ennoble:ITEM,CRYSTAL`,
//! `craft:unbind:ITEM`), then up to three seals and Recast's affix kept (`world/seal.rs`:
//! `craft:kindle:ITEM,CRYSTAL,SEAL`, `craft:recast:ITEM,CRYSTAL,-,-,-,AFFIX`), from the
//! profession's screen through the wizard's steps ([`steps`], [`plan`]). One stack may give
//! several of Imbue's crystals, one each.
//!
//! - **What**: an item carried, not worn or wielded: a base item (one with a kind; ammunition is
//!   one, the whole stack worked at once), not a spirit weapon, its magic identified, not a
//!   unique ([`refusal`]); a quest's object is refused at the start ("It will not take your
//!   will."), unpaid goods as everywhere ([`World::may_loose`]). Then each kind's own: **Kindle**
//!   a plain item; **Quicken** a magic item with one affix and room for another
//!   (`magic::capacity`), or a rare with room, which takes a heart-crystal too; **Recast** a
//!   magic (not rare: Unbind, then Imbue) item; **Imbue** a plain item of capacity 3 or more ("It would not
//!   hold so much."); **Ennoble** a magic item of capacity 3 or more with room for one affix
//!   more; **Unbind** a magic or rare item. No station, no tool: anywhere.
//! - **The colour**: the crystal's colour (`data/crystals.toml`) narrows the roll to its
//!   families, less those the seals shut ([`crate::magic::loot::roll_affix_within`]); with
//!   nothing left to roll there the work is refused before it starts ("No more of that nature will take."). Imbue's first
//!   affix is of its first crystal's families, the second of the second's, the third of the
//!   third's, any more of all three; each of its crystals must have something to roll.
//! - **The item level of the roll**: the greater of the item's own and the enchanter's reach
//!   (Enchanting's [`Effect::EnchantLevel`], 1 to 12); Kindle's and Imbue's are the reach,
//!   Unbind's the item's own. The item takes the roll's level.
//! - **Time**: the recipe's `seconds` and each seal's `seal_seconds` (Quick hands' share of
//!   them, `mastery::craft_seconds`), in turns of the `craft` action ([`World::keep_at`]):
//!   interrupted, nothing is spent, but the time is. No grade, so Master's hand has nothing to ease.
//! - **The check**: two gameplay draws (the natural roll), plus Willpower's modifier, its talents'
//!   checks, and Enchanting's [`Effect::Craft`], against the recipe's `difficulty` + the roll's
//!   item level ÷ `level_step` (`data/enchanting.toml`; [`enchant_band`]). Made (or a natural
//!   24): Kindle draws the kind (`[magic]`'s prefix and suffix weights among the kinds that can
//!   roll), then the affix, tier, and value; Quicken the affix of the kind the item lacks;
//!   Recast takes every affix off and rolls as many of the same kinds, in order; Imbue draws the
//!   count (`[rare] affixes`, held to the capacity), the split (`loot::rare_split`), each affix
//!   (a prefix while the split has one and the crystal can roll it, else a suffix), and the name
//!   (`loot::rare_name`); Ennoble draws the kind evenly among those with room that can roll,
//!   the affix, and the name; Unbind draws nothing and takes the magic off. Failed: the crystals are spent, the item as it was. A
//!   natural 2: the crystals spent and the item warped ([`ItemHarm::Enchant`], as a failed
//!   Temper damages; of Proof, spared).
//! - **Known**: the magic made is identified: the enchanter knows their work.
//! - **Marks**: [`Use::Craft`] on the recipe's skill alone, whatever came of it. No noise, no
//!   alignment.
//!
//! [`Action::Craft`]: crate::action::Action::Craft

use super::craft::RecipeLack;
use super::craft_make::{CraftOption, CraftPlan, CraftStep, CraftStepKind, MAX_PICKS};
use super::item_damage::Harmed;
use super::rest::Ended;
use super::seal;
use super::{Actor, LevelState, Spent, World};
use crate::action::Event;
use crate::combat;
use crate::content::{
    ActionKind, AffixKind, Content, EnchantKind, ItemHarm, MAX_PER_KIND, ObjectDef, Rarity,
    RecipeDef, RecipeId, RecipeUse,
};
use crate::magic::loot::{pick_weighted, rare_name, rare_split, roll_affix_within, rolls_within};
use crate::magic::{Affix, Magic, capacity};
use crate::object::Stack;
use crate::rng::Rng;
use crate::scroll_label::Lore;
use crate::skills::{Effect, MAX_ENCHANT_LEVEL, Use};
use crate::stats::Stat;
use crate::text::capitalized;
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

/// The refusal of an item whose capacity is below a rare's three affixes (Imbue's and
/// Ennoble's; the row's [`RecipeLack::Room`] too).
const SMALL: &str = "It would not hold so much.";

/// The fewest affixes a rare holds, and so the capacity Imbue and Ennoble want.
const RARE_LEAST: u8 = 3;

/// The words of what `kind` works on, the row's: "a plain item".
fn item_words(kind: EnchantKind) -> &'static str {
    match kind {
        EnchantKind::Kindle | EnchantKind::Imbue => "a plain item",
        EnchantKind::Quicken | EnchantKind::Recast | EnchantKind::Ennoble | EnchantKind::Unbind => {
            "a magic item"
        }
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
    let room = capacity(stack, content);
    match (kind, magic) {
        (EnchantKind::Kindle | EnchantKind::Imbue, Some(_)) => {
            Some("There is magic in it already.")
        }
        (EnchantKind::Kindle, None) => None,
        (EnchantKind::Imbue, None) => (room < RARE_LEAST).then_some(SMALL),
        (
            EnchantKind::Quicken | EnchantKind::Recast | EnchantKind::Ennoble | EnchantKind::Unbind,
            None,
        ) => Some("There is no magic in it."),
        // A rare's Quicken takes a heart-crystal too (`resolve`).
        (EnchantKind::Quicken, Some(m)) if m.rarity == Rarity::Rare => {
            let full =
                m.affixes.len() >= usize::from(room) || with_room(content, &m.affixes).is_empty();
            full.then_some(FULL)
        }
        (EnchantKind::Ennoble, Some(m)) if m.rarity == Rarity::Rare => {
            Some("It is rare already: only a heart-crystal's Quicken adds to it.")
        }
        (EnchantKind::Recast, Some(m)) if m.rarity == Rarity::Rare => {
            Some("It is rare: unbind it, then imbue it anew.")
        }
        (EnchantKind::Quicken, Some(m)) => {
            let full = m.affixes.len() >= 2 || m.affixes.len() >= usize::from(room);
            full.then_some(FULL)
        }
        (EnchantKind::Ennoble, Some(_)) if room < RARE_LEAST => Some(SMALL),
        (EnchantKind::Ennoble, Some(m)) => (m.affixes.len() >= usize::from(room)).then_some(FULL),
        (EnchantKind::Recast | EnchantKind::Unbind, Some(_)) => None,
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
        return Some(
            match why.contains(&Some(FULL)) || why.contains(&Some(SMALL)) {
                true => RecipeLack::Room,
                false => RecipeLack::Nothing,
            },
        );
    }
    let crystals: u32 = (player.inventory.iter())
        .filter(|s| is_crystal(content, s))
        .map(|s| s.count)
        .sum();
    (crystals < u32::from(recipe.crystals)).then_some(RecipeLack::Crystal)
}

/// The wizard's steps of `recipe` as `picks` (in step order, so far) narrow them: the item, a
/// crystal for each it takes, the seals, and Recast's affix kept (`seal::steps`).
pub(super) fn steps(
    content: &Content,
    lore: Lore<'_>,
    player: &Actor,
    recipe: &RecipeDef,
    picks: &[Option<usize>],
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
    for n in 0..recipe.crystals {
        // Imbue's crystals are told apart: the first affix is the first crystal's.
        let label = match (recipe.crystals, n) {
            (1, _) => "crystal",
            (_, 0) => "first crystal",
            (_, 1) => "second crystal",
            _ => "third crystal",
        };
        steps.push(CraftStep {
            kind: CraftStepKind::Input,
            label: label.into(),
            optional: false,
            options: crystals.clone(),
        });
    }
    let item = (picks.first().copied().flatten())
        .filter(|&at| refusal(content, player, at, kind).is_none())
        .map(|at| {
            let stack = &player.inventory[at];
            let affixes: &[Affix] = stack.magic().map_or(&[], |m| &m.affixes);
            seal::Item {
                def: content.object(stack.object),
                level: roll_level(content, player, kind, stack),
                had: match kind {
                    EnchantKind::Recast => Vec::new(),
                    _ => affixes.to_vec(),
                },
                affixes,
            }
        });
    steps.extend(seal::steps(
        content,
        player,
        kind,
        recipe.crystals,
        picks,
        item,
    ));
    steps
}

/// The steps of `recipe` for `player` as `picks` narrow them: an Enchanting recipe's seals and
/// affix kept follow its picks (`seal::steps`); any other's are [`craft_steps`]'s.
///
/// [`craft_steps`]: super::craft_steps
pub(crate) fn craft_steps_given(
    content: &Content,
    lore: Lore<'_>,
    level: &LevelState,
    player: &Actor,
    recipe: RecipeId,
    picks: &[Option<usize>],
) -> Vec<CraftStep> {
    let def = content.recipe(recipe);
    match def.uses {
        Some(RecipeUse::Enchant(_)) => steps(content, lore, player, def, picks),
        _ => super::craft_steps(content, lore, level, player, recipe),
    }
}

/// The item level a work of `kind` on `stack` rolls at (the module docs).
fn roll_level(content: &Content, player: &Actor, kind: EnchantKind, stack: &Stack) -> u8 {
    let own = stack.magic().map_or(0, |m| m.level);
    let reach = reach(content, player);
    match kind {
        EnchantKind::Kindle | EnchantKind::Imbue => reach,
        EnchantKind::Quicken | EnchantKind::Recast | EnchantKind::Ennoble => own.max(reach),
        EnchantKind::Unbind => own,
    }
}

/// The picks checked: the item's position, the crystals' (none for Unbind, the same stack once
/// for each it gives), the item level of the roll, the families each crystal allows (in pick
/// order, the seals' shut out), and the kinds of affix the roll may draw: Kindle's and Ennoble's
/// choice, Quicken's (a rare's: those with room), Recast's in the item's order (less the one
/// kept); empty for Imbue (its split draws them) and Unbind. Then the seals' categories, what
/// else is spent (a crystal for each seal, the heart-crystal, the soul-fetter: positions), and
/// Recast's affix kept (its index among the item's).
struct Resolved {
    item: usize,
    crystals: Vec<usize>,
    level: u8,
    families: Vec<Vec<String>>,
    kinds: Vec<AffixKind>,
    seals: Vec<String>,
    spent: Vec<usize>,
    heart: Option<usize>,
    fetter: Option<usize>,
    kept: Option<usize>,
}

impl Resolved {
    /// The families of the first crystal (the one-crystal works'), or every family (none).
    fn first(&self) -> Option<&[String]> {
        self.families.first().map(Vec::as_slice)
    }

    /// Every position spent, a stack once for each of it: the crystals, then the rest.
    fn all_spent(&self) -> Vec<usize> {
        let mut all = self.crystals.clone();
        all.extend(&self.spent);
        all
    }
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
    let slots = seal::slots(kind, recipe.crystals);
    let item = match picks.first() {
        Some(Some(item)) => *item,
        _ => return Err("You must choose the item.".into()),
    };
    if picks.len() > slots.len() {
        return Err("That is more than the work takes.".into());
    }
    let pick = |i: usize| picks.get(i).copied().flatten();
    if slots.crystals.clone().any(|i| pick(i).is_none()) {
        return Err(match recipe.crystals {
            1 => "You must choose the crystal.",
            _ => "You must choose the crystals.",
        }
        .into());
    }
    if let Some(text) = refusal(content, player, item, kind) {
        return Err(text.into());
    }
    let stack = &player.inventory[item];
    let crystals: Vec<usize> = slots.crystals.clone().filter_map(pick).collect();
    let mut families = Vec::new();
    for (n, &at) in crystals.iter().enumerate() {
        let stone = player.inventory.get(at).ok_or("You carry no such thing.")?;
        let colour = (content.crystals())
            .and_then(|c| c.colour_of_object(stone.object))
            .ok_or("That is no crystal.")?;
        let picked = crystals[..=n].iter().filter(|&&c| c == at).count();
        if picked > stone.count as usize {
            return Err("You do not carry so many crystals.".into());
        }
        families.push(colour.families.clone());
    }
    let level = roll_level(content, player, kind, stack);
    let def = content.object(stack.object);
    let rolled: &[Affix] = stack.magic().map_or(&[], |m| &m.affixes);
    let rare = stack.magic().is_some_and(|m| m.rarity == Rarity::Rare);
    let rares = content.crystals().map(|c| (c.rare.heart, c.rare.fetter));
    let heart = match (kind, rare) {
        (EnchantKind::Quicken, true) => Some(
            (rares.and_then(|(heart, _)| seal::carried(player, heart)))
                .ok_or("A rare takes a heart-crystal beside the crystal, and you have none.")?,
        ),
        _ => None,
    };
    let kept = slots.affix.and_then(pick);
    if kept.is_some_and(|k| k >= rolled.len()) {
        return Err("There is no such magic in it.".into());
    }
    let fetter = match kept {
        Some(_) => Some(
            (rares.and_then(|(_, fetter)| seal::carried(player, fetter)))
                .ok_or("You have no soul-fetter to keep it with.")?,
        ),
        None => None,
    };
    // What the item keeps through the work: Recast's kept affix alone, the rest all it has.
    let had: Vec<Affix> = match kind {
        EnchantKind::Recast => kept.map(|k| rolled[k].clone()).into_iter().collect(),
        _ => rolled.to_vec(),
    };
    let seals: Vec<usize> = slots.seals.clone().filter_map(pick).collect();
    let sealed = seal::check(content, player, (def, level), &had, &crystals, &seals)?;
    for family in &mut families {
        family.retain(|c| !sealed.categories.contains(c));
    }
    let first = families.first().map(Vec::as_slice);
    let can = |k: AffixKind, rolled: &[Affix], families: Option<&[String]>| {
        rolls_within(content, def, level, k, rolled, families)
    };
    let both = [AffixKind::Prefix, AffixKind::Suffix];
    let kinds: Vec<AffixKind> = match kind {
        EnchantKind::Kindle => both.into_iter().filter(|&k| can(k, &[], first)).collect(),
        EnchantKind::Quicken if rare => (with_room(content, rolled).into_iter())
            .filter(|&k| can(k, rolled, first))
            .collect(),
        EnchantKind::Quicken => (lacking(content, rolled).into_iter())
            .filter(|&k| can(k, rolled, first))
            .collect(),
        EnchantKind::Ennoble => (with_room(content, rolled).into_iter())
            .filter(|&k| can(k, rolled, first))
            .collect(),
        // Each kind the item has, again (less the one kept): a magic item holds one affix of
        // each kind, so no two of them can crowd each other's category out.
        EnchantKind::Recast => {
            let others: Vec<Affix> = (rolled.iter().enumerate())
                .filter(|&(i, _)| Some(i) != kept)
                .map(|(_, a)| a.clone())
                .collect();
            let kinds = kinds_of(content, &others);
            if kinds.is_empty() {
                return Err("Nothing would be left to recast.".into());
            }
            if !kinds.iter().all(|&k| can(k, &had, first)) {
                return Err(NOTHING.into());
            }
            kinds
        }
        EnchantKind::Imbue => {
            let rolls = |f: &Vec<String>| both.into_iter().any(|k| can(k, &[], Some(f)));
            if !families.iter().all(rolls) {
                return Err(NOTHING.into());
            }
            Vec::new()
        }
        EnchantKind::Unbind => Vec::new(),
    };
    let drawn = !matches!(kind, EnchantKind::Imbue | EnchantKind::Unbind);
    if drawn && kinds.is_empty() {
        return Err(NOTHING.into());
    }
    let mut spent = sealed.crystals;
    spent.extend(heart);
    spent.extend(fetter);
    Ok(Resolved {
        item,
        crystals,
        level,
        families,
        kinds,
        seals: sealed.categories,
        spent,
        heart,
        fetter,
        kept,
    })
}

/// How long work of recipe `def` with `seals` takes before Quick hands: its own time and each
/// seal's (`data/enchanting.toml` `seal_seconds`).
fn work_seconds(content: &Content, def: &RecipeDef, seals: usize) -> u32 {
    let each = content.enchanting().map_or(0, |r| r.seal_seconds);
    def.seconds + each * seals as u32
}

/// The kinds of `rolled`'s affixes, in their order.
fn kinds_of(content: &Content, rolled: &[Affix]) -> Vec<AffixKind> {
    (rolled.iter())
        .filter_map(|a| content.affixes().get(&a.id).map(|d| d.kind))
        .collect()
}

/// The kind of affix a magic item with `rolled` lacks: a suffix beside its prefix, a prefix
/// beside its suffix.
fn lacking(content: &Content, rolled: &[Affix]) -> Option<AffixKind> {
    let kinds = kinds_of(content, rolled);
    match (
        kinds.contains(&AffixKind::Prefix),
        kinds.contains(&AffixKind::Suffix),
    ) {
        (true, false) => Some(AffixKind::Suffix),
        (false, true) => Some(AffixKind::Prefix),
        _ => None,
    }
}

/// The kinds a rare with `rolled` has room for: those below [`MAX_PER_KIND`].
fn with_room(content: &Content, rolled: &[Affix]) -> Vec<AffixKind> {
    let kinds = kinds_of(content, rolled);
    [AffixKind::Prefix, AffixKind::Suffix]
        .into_iter()
        .filter(|k| kinds.iter().filter(|&x| x == k).count() < MAX_PER_KIND)
        .collect()
}

/// `affix`, of `kind`, put among `affixes` with prefixes before suffixes: a prefix after the
/// last prefix, a suffix at the end.
fn place(content: &Content, affixes: &mut Vec<Affix>, affix: Affix, kind: AffixKind) {
    match kind {
        AffixKind::Prefix => {
            let at = (kinds_of(content, affixes).iter())
                .filter(|&&k| k == AffixKind::Prefix)
                .count();
            affixes.insert(at.min(affixes.len()), affix);
        }
        AffixKind::Suffix => affixes.push(affix),
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

/// What the work `resolved` uses, in words, in order (the order of the crystals matters to
/// Imbue): "a red crystal", "a red crystal, a red crystal, and a blue crystal", then the seals'
/// crystals, the heart-crystal, the soul-fetter with what it keeps, and what the seals shut out:
/// "a red crystal and one more red crystal, sealing out fire".
fn uses_words(content: &Content, lore: Lore<'_>, player: &Actor, resolved: &Resolved) -> String {
    let pack = &player.inventory;
    let mut words: Vec<String> = (resolved.crystals.iter())
        .map(|&at| lore.describe(&pack[at].single()))
        .collect();
    let mut more: Vec<(usize, u32)> = Vec::new();
    for &at in &resolved.spent {
        if Some(at) == resolved.heart || Some(at) == resolved.fetter {
            continue;
        }
        match more.iter_mut().find(|(a, _)| *a == at) {
            Some(seen) => seen.1 += 1,
            None => more.push((at, 1)),
        }
    }
    for (at, n) in more {
        words.push(match n {
            1 => format!("one more {}", lore.title(&pack[at].single())),
            n => format!(
                "{} more {}",
                crate::text::number_word(n as usize),
                lore.names(&pack[at]).1
            ),
        });
    }
    if let Some(at) = resolved.heart {
        words.push(lore.describe(&pack[at].single()));
    }
    if let (Some(at), Some(k)) = (resolved.fetter, resolved.kept) {
        let item = &pack[resolved.item];
        let affix = (item.magic().and_then(|m| m.affixes.get(k)))
            .and_then(|a| content.affixes().get(&a.id))
            .map_or(String::new(), |d| d.name.clone());
        words.push(format!(
            "{} (keeping {affix})",
            lore.describe(&pack[at].single())
        ));
    }
    let mut text = crate::text::and_list(&words);
    if !resolved.seals.is_empty() {
        let shut = seal::words(content, &resolved.seals);
        text = match text.is_empty() {
            true => format!("sealing out {shut}"),
            false => format!("{text}, sealing out {shut}"),
        };
    }
    text
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
    let seconds = work_seconds(content, recipe, resolved.seals.len());
    Ok(CraftPlan {
        verb: recipe.verb.clone().unwrap_or_else(|| recipe.name.clone()),
        name: the(lore, &player.inventory[resolved.item]),
        uses: uses_words(content, lore, player, &resolved),
        station: None,
        time: format!(
            "about {}",
            Duration::seconds(u64::from(super::mastery::craft_seconds(
                content, player, seconds
            )))
            .spoken()
        ),
    })
}

/// An enchanting under way: the recipe, the picks (in step order), how long.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Enchanting {
    recipe: RecipeId,
    picks: [Option<usize>; MAX_PICKS],
    len: u8,
    seconds: u32,
}

impl Enchanting {
    fn picks(self) -> Vec<Option<usize>> {
        self.picks[..usize::from(self.len)].to_vec()
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
        if !self.may_loose(item) || !resolved.all_spent().iter().all(|&c| self.may_loose(c)) {
            return Spent::Nothing;
        }
        let thing = the(self.lore(), &self.state.player().inventory[item]);
        self.say(format!("You take {thing} in your lap and begin."));
        let mut held = [None; MAX_PICKS];
        held[..picks.len()].copy_from_slice(picks);
        let seconds = work_seconds(&content, def, resolved.seals.len());
        Spent::Enchant(Enchanting {
            recipe: id,
            picks: held,
            len: picks.len() as u8,
            seconds: self.craft_seconds(seconds),
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
    /// left, or warped; the crystals spent either way.
    fn enchant_out(&mut self, def: &RecipeDef, resolved: &Resolved, band: EnchantBand) {
        let Some(RecipeUse::Enchant(kind)) = def.uses else {
            return;
        };
        let content = self.content.clone();
        let at = resolved.item;
        let thing = the(self.lore(), &self.state.player().inventory[at]);
        let stone = (resolved.crystals.first())
            .map(|&c| self.lore().title(&self.state.player().inventory[c]));
        let dark = match resolved.crystals.len() + resolved.seals.len() {
            1 => "The crystal goes dark in your hands.",
            _ => "The crystals go dark in your hands.",
        };
        let mut crystals = resolved.all_spent();
        // What the fetter keeps, by name, before the work changes the item.
        let kept = (resolved.kept)
            .and_then(|k| self.state.player().inventory[at].magic()?.affixes.get(k))
            .and_then(|a| content.affixes().get(&a.id))
            .map(|d| d.name.clone());
        match band {
            EnchantBand::Made => {
                if let Some(kept) = &kept {
                    self.say(format!(
                        "The soul-fetter holds one enchantment fast: {kept}."
                    ));
                }
                let stack = self.state.player().inventory[at].clone();
                let made = self.worked(&content, kind, stack, resolved);
                let name = self.lore().describe(&made);
                let title = (made.magic().and_then(|m| m.name.clone())).unwrap_or(name.clone());
                self.player_mut().inventory[at] = made;
                let text = match (kind, &stone) {
                    (EnchantKind::Unbind, _) | (_, None) => {
                        format!("The magic goes out of {thing}. It is now {name}.")
                    }
                    (EnchantKind::Imbue, _) => format!(
                        "{} drinks the {} crystals and wakes as {title}.",
                        capitalized(&thing),
                        crate::text::number_word(crystals.len())
                    ),
                    (EnchantKind::Ennoble, Some(stone)) => format!(
                        "{} takes the {stone}'s glow and wakes as {title}.",
                        capitalized(&thing)
                    ),
                    (EnchantKind::Quicken, Some(stone)) if resolved.heart.is_some() => format!(
                        "{} takes the {stone}'s glow and the heart-crystal's, and holds more.",
                        capitalized(&thing)
                    ),
                    (_, Some(stone)) => format!(
                        "{} takes the {stone}'s glow. It is now {name}.",
                        capitalized(&thing)
                    ),
                };
                self.say(text);
            }
            EnchantBand::Failed => match kind {
                EnchantKind::Unbind => {
                    self.say(format!("{} clings to its magic.", capitalized(&thing)))
                }
                _ => {
                    self.say(dark);
                    self.rare_lost(resolved);
                }
            },
            EnchantBand::Warped => {
                if stone.is_some() {
                    self.say(dark);
                    self.rare_lost(resolved);
                }
                match self.harm_item(at, ItemHarm::Enchant) {
                    Harmed::Withstood => {}
                    Harmed::Damaged => {
                        self.say(format!("{} warps under your hands.", capitalized(&thing)))
                    }
                    Harmed::Destroyed => {
                        self.say(format!("{} warps and falls apart.", capitalized(&thing)));
                        for c in &mut crystals {
                            if *c > at {
                                *c -= 1;
                            }
                        }
                    }
                }
            }
        }
        self.spend_crystals(&crystals);
    }

    /// The heart-crystal or the soul-fetter of a work that did not take, said gone.
    fn rare_lost(&mut self, resolved: &Resolved) {
        if resolved.heart.is_some() {
            self.say("The heart-crystal goes dark with it.");
        }
        if resolved.fetter.is_some() {
            self.say("The soul-fetter crumbles to nothing.");
        }
    }

    /// One crystal from the stack at each of `crystals` (a stack named twice gives two); a stack
    /// given whole is taken out of the pack.
    fn spend_crystals(&mut self, crystals: &[usize]) {
        let mut sorted = crystals.to_vec();
        sorted.sort_unstable();
        let mut gone = Vec::new();
        let pack = &mut self.player_mut().inventory;
        for (n, &at) in sorted.iter().enumerate() {
            if n > 0 && sorted[n - 1] == at {
                continue;
            }
            let times = sorted.iter().filter(|&&c| c == at).count() as u32;
            match pack.get_mut(at) {
                Some(stack) if stack.count > times => stack.count -= times,
                Some(_) => gone.push(at),
                None => {}
            }
        }
        super::outcome::remove_stacks(self.player_mut(), &gone);
    }

    /// `stack` after kind `kind`'s work, made (the draws in the module docs' order).
    fn worked(
        &mut self,
        content: &Content,
        kind: EnchantKind,
        mut stack: Stack,
        resolved: &Resolved,
    ) -> Stack {
        let def = content.object(stack.object);
        let families = resolved.first();
        let level = resolved.level;
        let room = capacity(&stack, content);
        let rng = &mut self.state.rng;
        let fresh = |affixes: Vec<Affix>, rarity: Rarity, name: Option<String>| Magic {
            rarity,
            level,
            affixes,
            name,
            identified: true,
            carried: 0,
            unique: None,
        };
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
                    stack.set_magic(Some(fresh(vec![affix], Rarity::Magic, None)));
                }
            }
            EnchantKind::Quicken | EnchantKind::Ennoble => {
                let Some(mut magic) = stack.magic().cloned() else {
                    return stack;
                };
                // Ennoble and a rare's Quicken draw evenly among the kinds with room.
                let evenly = kind == EnchantKind::Ennoble || magic.rarity == Rarity::Rare;
                let drawn = match (evenly, resolved.kinds.as_slice()) {
                    (_, []) => return stack,
                    (true, kinds) => kinds[rng.index(kinds.len())],
                    (false, kinds) => kinds[0],
                };
                let rolled = &magic.affixes;
                let Some(affix) =
                    roll_affix_within(content, rng, def, level, drawn, rolled, families)
                else {
                    return stack;
                };
                place(content, &mut magic.affixes, affix, drawn);
                if kind == EnchantKind::Ennoble {
                    magic.rarity = Rarity::Rare;
                    magic.name = Some(rare_name(content, rng, def));
                }
                magic.level = level;
                magic.identified = true;
                stack.set_magic(Some(magic));
            }
            EnchantKind::Recast => {
                let Some(mut magic) = stack.magic().cloned() else {
                    return stack;
                };
                // The kept affix stays; the rest roll anew, each placed by its kind.
                let kept: Vec<Affix> = (resolved.kept)
                    .and_then(|k| magic.affixes.get(k).cloned())
                    .into_iter()
                    .collect();
                let mut rolled = kept.clone();
                for &k in &resolved.kinds {
                    if let Some(affix) =
                        roll_affix_within(content, rng, def, level, k, &rolled, families)
                    {
                        place(content, &mut rolled, affix, k);
                    }
                }
                if rolled.len() == kept.len() {
                    return stack;
                }
                magic.affixes = rolled;
                magic.level = level;
                magic.identified = true;
                stack.set_magic(Some(magic));
            }
            EnchantKind::Imbue => {
                let rolled = imbued(content, rng, (def, level, room), &resolved.families);
                if rolled.is_empty() {
                    return stack;
                }
                let name = rare_name(content, rng, def);
                stack.set_magic(Some(fresh(rolled, Rarity::Rare, Some(name))));
            }
        }
        stack
    }
}

/// Imbue's affixes for `def` at `level` with capacity `room`, from crystals of `families` in
/// pick order (the module docs): the count (held to the capacity), the split, then each affix,
/// prefixes before suffixes.
fn imbued(
    content: &Content,
    rng: &mut Rng,
    (def, level, room): (&ObjectDef, u8, u8),
    families: &[Vec<String>],
) -> Vec<Affix> {
    let counts = &content.affixes().rare_counts;
    let count = pick_weighted(rng, counts).copied().unwrap_or(RARE_LEAST);
    let count = count.min(room).max(1);
    let (prefixes, suffixes) = rare_split(rng, i32::from(count));
    let mut left = [prefixes, suffixes];
    let mut every: Vec<String> = Vec::new();
    for family in families.iter().flatten() {
        if !every.contains(family) {
            every.push(family.clone());
        }
    }
    let slot = |k: AffixKind| usize::from(k == AffixKind::Suffix);
    let mut rolled: Vec<Affix> = Vec::new();
    for n in 0..usize::from(count) {
        let these = families.get(n).unwrap_or(&every);
        let order = match left[0] > 0 {
            true => [AffixKind::Prefix, AffixKind::Suffix],
            false => [AffixKind::Suffix, AffixKind::Prefix],
        };
        let fits = |k: &AffixKind| {
            left[slot(*k)] > 0 && rolls_within(content, def, level, *k, &rolled, Some(these))
        };
        let Some(k) = order.into_iter().find(fits) else {
            continue;
        };
        if let Some(affix) = roll_affix_within(content, rng, def, level, k, &rolled, Some(these)) {
            place(content, &mut rolled, affix, k);
            left[slot(k)] -= 1;
        }
    }
    rolled
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
