//! Loot (spec §8 "Magic items"): rolling a random item at an item level.
//!
//! [`roll`] draws, in this order, from the stream it is given (a level's `loot/<level id>` while
//! it is generated, `drop/<actor id>` for what a creature drops; spec §8 "Determinism"):
//!
//! 0. **a loose gem**: a find that may be anything (grade `any`) is a gem by the gems' `chance`
//!    (one draw, then the gem's draws: [`crate::gem::find`]), and that is all;
//! 1. **the base item**: one draw among the objects that are loot (base items, with a `kind`,
//!    and scrolls, which are never magic and so left out when the grade asks for magic or
//!    better) whose `level` is at most the item level, weighted by their `frequency`; a scroll
//!    of a spell is its grimoire instead by the grimoires' `chance` (one draw, a spell scroll's
//!    only: [`Content::grimoire_or`]);
//! 2. **the count**: ammunition comes in stacks of [`AMMO`] (one draw); anything else is one;
//! 3. **the rarity**: one draw over the weights of `[rarity]` in `data/affixes.toml` (those below
//!    the grade asked for left out); none for what is never magic (a scroll), which is normal;
//! 4. **how many affixes**: a magic item has a prefix, a suffix, or one of each by the weights
//!    of `[magic]` (one draw); a rare one draws its count from `rare.affixes` (one draw) and how
//!    many of them are prefixes (one draw, evenly among the splits that keep at most three of
//!    each, so prefixes and suffixes are alike);
//! 5. **each affix**, prefixes first: one draw among those that may roll on the item (its kind
//!    in the affix's `bases`, something for the effect to change, a tier at the item level, and
//!    a `category` the item has no affix of yet), weighted by their `weight`; one draw for the tier
//!    among those of the window at the item level ([`tier_window`]: the tiers found there whose
//!    level is within [`TIER_WINDOW`] of it, or the top one found alone past the last tier or
//!    when none is that close); one draw for the value in the tier's range (none when the range
//!    is a single value). With nothing left to roll, the item has fewer affixes; with
//!    none at all it is normal;
//! 6. **a rare item's name**: one draw for its first word and one for its second, which goes
//!    with the kind of item ("Bite" for weapons, "Loop" for rings);
//! 7. **an ornament**: an item of a class set with gems is set with one by its class's chance
//!    (one draw, then the gem's draws: [`crate::gem::ornament`]);
//! 8. **its material** (#193, #194): an item its construction list lets be made of other
//!    materials is made of one by their `rarity` at the item level, or by their `gear` at the
//!    creature's level for what a creature drops (one draw: [`crate::variant::draw`]); none for
//!    one made only of its own;
//! 9. **a wand's rung** (one draw, a wand's only), then **its make** (#1436), loot's only
//!    (`Source::Loot`): arms and armour are Fine, Marvellous, or Masterwork by the `[[depth]]`
//!    row of `data/gear_condition.toml` at the item level (one draw, then one per improvement;
//!    none where the row gives nothing or for what takes no make:
//!    [`super::make::make`]); the same row adds its `ornament` percent to step
//!    7's chance;
//! 10. **the trim** ([`finished`], the very last step of every path that completes an item,
//!     creature gear's after its condition): an item with more affixes than its capacity
//!     ([`super::capacity`]) loses them one at a time, one `index` draw each; none when it fits.
//!
//! What is rolled is unidentified (its gems and material show at once: materials are not
//! hidden).
//!
//! A vault's `loot` cell rolls from a [`LootPool`] ([`roll_pool`]): with no classes named exactly
//! [`roll_with`]; with classes, one draw among those that hold something at the item level, then
//! the find: coins (the purse of a chest of that level, [`crate::chest::purse`]), a loose gem
//! ([`gem::loose`]), or step 1's draw among the class's objects and steps 2 to 8.

use super::{Affix, AffixKind, Magic, Quality, Rarity, may_roll};
use crate::chest;
use crate::content::{AffixDef, Content, ItemClassId, ItemKind, ObjectDef, ObjectId};
use crate::gem;
use crate::object::Stack;
use crate::rng::Rng;
use crate::variant::{self, Source};
use serde::{Deserialize, Serialize};
use std::ops::{Range, RangeInclusive};

/// How many arrows, bolts, or stones a stack of loot ammunition holds.
pub const AMMO: RangeInclusive<i32> = 8..=20;

/// How close below the item level a tier's level must be for loot to roll it: a level-12 find
/// rolls the tiers of levels 9 to 12 (docs/design/affixes-systems.md §8), so depth tells.
pub const TIER_WINDOW: u8 = 4;

/// The least an item of loot may be: anything, magic or better, or rare (a vault's `loot`, what
/// creatures drop).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LootGrade {
    /// By the weights of every rarity.
    #[default]
    Any,
    /// Magic or rare.
    Magic,
    Rare,
}

impl LootGrade {
    /// The lowest rarity it allows.
    pub const fn least(self) -> Rarity {
        match self {
            LootGrade::Any => Rarity::Normal,
            LootGrade::Magic => Rarity::Magic,
            LootGrade::Rare => Rarity::Rare,
        }
    }
}

/// What a vault's `loot` cell may hold (spec §7 "Level file format"): `loot = "magic"`, or
/// `loot = { pool = ["armour", "rings"], grade = "magic" }`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LootPool {
    /// The least it may be.
    pub grade: LootGrade,
    /// The classes it is drawn from, evenly, in the order named; empty: anything.
    pub classes: Vec<ItemClassId>,
}

impl LootPool {
    /// The pool `names` (class ids and class groups) name at `grade`. Refused when one names
    /// nothing. A class named twice counts once; a pool that can hold nothing of its grade
    /// leaves its cell bare ([`roll_pool`]).
    pub fn named(content: &Content, names: &[String], grade: LootGrade) -> Result<Self, String> {
        let mut classes = Vec::new();
        for name in names {
            let named = content.pool_classes(name);
            if named.is_empty() {
                return Err(format!(
                    "loot pool: `{name}` is no item class or class group"
                ));
            }
            for class in named {
                if !classes.contains(&class) {
                    classes.push(class);
                }
            }
        }
        Ok(Self { grade, classes })
    }
}

impl From<LootGrade> for LootPool {
    fn from(grade: LootGrade) -> Self {
        Self {
            grade,
            classes: Vec::new(),
        }
    }
}

/// A finder's luck (spec §8 "Talents", "Birth sign"): their talents make the magic and rare
/// rarities `magic` percent more likely (Treasure hunter), and a loose gem `gems` percent more
/// likely (Hoard sense). The same draws either way.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Luck {
    pub magic: i32,
    pub gems: i32,
}

impl Luck {
    /// The luck of a finder with `talents`.
    pub fn of(talents: &crate::talent::TalentMods) -> Self {
        Self {
            magic: talents.fortune.loot_magic_percent,
            gems: talents.fortune.gem_find_percent,
        }
    }
}

/// Rolls an item of loot at item level `level`, at least of `grade`, from `rng` (the module docs
/// give the draws). `None` when content has no base item found at that level.
pub fn roll(content: &Content, rng: &mut Rng, level: u8, grade: LootGrade) -> Option<Stack> {
    roll_with(content, rng, level, grade, Source::Loot, Luck::default())
}

/// [`roll`], its material drawn as `source` has it: loot's by the item level, a creature's
/// gear by the creature's level.
pub fn roll_from(
    content: &Content,
    rng: &mut Rng,
    level: u8,
    grade: LootGrade,
    source: Source,
) -> Option<Stack> {
    roll_with(content, rng, level, grade, source, Luck::default())
}

/// [`roll_from`] for a finder with `luck` ([`Luck`]): the rarity weights and the gems' chance
/// scaled, the same draws.
pub fn roll_with(
    content: &Content,
    rng: &mut Rng,
    level: u8,
    grade: LootGrade,
    source: Source,
    luck: Luck,
) -> Option<Stack> {
    let level = level.max(1);
    if grade == LootGrade::Any
        && let Some(gem) = gem::find_with(content, rng, level, luck.gems)
    {
        return Some(gem);
    }
    let bases = loot_bases(content, level, grade, |_| true);
    drawn(content, rng, &bases, (level, grade, luck), source)
}

/// The objects that are loot at item level `level` for `grade` among those `within` allows, by
/// id (the string), never by load order: a draw picks the same base however the object files are
/// laid out (docs/dev/content.md).
fn loot_bases(
    content: &Content,
    level: u8,
    grade: LootGrade,
    within: impl Fn(&ObjectDef) -> bool,
) -> Vec<(ObjectId, u32)> {
    let magic_only = grade != LootGrade::Any;
    let mut bases: Vec<(ObjectId, u32)> = (content.object_defs().iter().enumerate())
        .filter(|(_, o)| o.frequency > 0 && o.level <= level && within(o))
        .filter(|(_, o)| !magic_only || can_be_magic(content, o, level))
        .map(|(i, o)| (ObjectId(i as u16), o.frequency))
        .collect();
    bases.sort_by(|a, b| content.object(a.0).id.cmp(&content.object(b.0).id));
    bases
}

/// Steps 1 to 8 of the module docs: one of `bases`, and what is made of it.
fn drawn(
    content: &Content,
    rng: &mut Rng,
    bases: &[(ObjectId, u32)],
    (level, grade, luck): (u8, LootGrade, Luck),
    source: Source,
) -> Option<Stack> {
    // A spell scroll may be its grimoire (spec §8 "Spells": "Learning"): one draw, a scroll's.
    let object = *pick_weighted(rng, bases)?;
    let object = content.grimoire_or(rng, object);
    let count = if content.object(object).kind == Some(ItemKind::Ammo) {
        u32::try_from(rng.range(*AMMO.start(), *AMMO.end())).unwrap_or(1)
    } else {
        1
    };
    let made = make_from(
        content,
        rng,
        object,
        count,
        (level, grade, luck.magic),
        Some(source),
    );
    Some(finished(content, rng, made))
}

/// What a class is found as: coins and gems have no `frequency`, and are found by the purse and
/// the gem's own draws.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Find {
    Coins,
    Gem,
    Item,
}

fn find_of(content: &Content, class: ItemClassId) -> Find {
    let of = |object: ObjectId| content.object(object).class == class;
    if chest::currency(content).is_some_and(of) {
        Find::Coins
    } else if content.gems().is_some_and(|g| of(g.object)) {
        Find::Gem
    } else {
        Find::Item
    }
}

/// Whether class `class` holds anything of `grade` at item level `level`: what is never magic
/// (coins, gems, a potion) holds nothing that must be.
fn holds(content: &Content, class: ItemClassId, level: u8, grade: LootGrade) -> bool {
    match find_of(content, class) {
        Find::Coins => {
            let band = content.chests().band_of(level);
            grade == LootGrade::Any
                && band.is_some_and(|b| b.loot.gold.dice.max() > 0 && b.loot.gold.each > 0)
        }
        Find::Gem => grade == LootGrade::Any,
        Find::Item => !loot_bases(content, level, grade, |o| o.class == class).is_empty(),
    }
}

/// Rolls an item of `pool` at item level `level` for a finder with `luck` (the module docs give
/// the draws). `None` when none of its classes holds anything there yet: a shallow floor may
/// leave a pool of deep things empty.
pub fn roll_pool(
    content: &Content,
    rng: &mut Rng,
    level: u8,
    pool: &LootPool,
    luck: Luck,
) -> Option<Stack> {
    let grade = pool.grade;
    if pool.classes.is_empty() {
        return roll_with(content, rng, level, grade, Source::Loot, luck);
    }
    let level = level.max(1);
    let open: Vec<ItemClassId> = (pool.classes.iter().copied())
        .filter(|c| holds(content, *c, level, grade))
        .collect();
    let class = *rng.pick(&open)?;
    match find_of(content, class) {
        Find::Coins => chest::purse(content, rng, level),
        Find::Gem => gem::loose(content, rng, level),
        Find::Item => {
            let bases = loot_bases(content, level, grade, |o| o.class == class);
            drawn(content, rng, &bases, (level, grade, luck), Source::Loot)
        }
    }
}

/// A base item of one of `classes` that is loot at item level `level` (its `level` at most it,
/// a `frequency`), by frequency, ordered by object id as [`roll`]'s are: one draw of `rng`.
/// `None` when there is none. An animated blade's weapon is drawn so (spec §8 "Animated
/// objects").
pub fn base_in(
    content: &Content,
    rng: &mut Rng,
    classes: &[ItemClassId],
    level: u8,
) -> Option<ObjectId> {
    let mut bases: Vec<(ObjectId, u32)> = (content.object_defs().iter().enumerate())
        .filter(|(_, o)| o.frequency > 0 && o.level <= level.max(1) && classes.contains(&o.class))
        .map(|(i, o)| (ObjectId(i as u16), o.frequency))
        .collect();
    bases.sort_by(|a, b| content.object(a.0).id.cmp(&content.object(b.0).id));
    pick_weighted(rng, &bases).copied()
}

/// `count` of `object` made as loot makes its base item at item level `level`, at least of
/// `grade`: draws 3 to 7 of [`roll`] (the rarity, the affixes, a rare item's name, an ornament),
/// then 9 and 10 (a wand's rung, the trim), unidentified. What is never magic (no `kind`) draws no rarity and stays normal. A starting
/// kit's magic items are made so (spec §8 "Character creation").
pub fn make(
    content: &Content,
    rng: &mut Rng,
    object: ObjectId,
    count: u32,
    level: u8,
    grade: LootGrade,
) -> Stack {
    let made = make_from(content, rng, object, count, (level, grade, 0), None);
    finished(content, rng, made)
}

/// [`make`] for a creature's gear (spec §8 "Creature gear"): its material drawn as `source` has
/// it (a creature's, by its level) after the rest.
pub fn make_as(
    content: &Content,
    rng: &mut Rng,
    object: ObjectId,
    count: u32,
    (level, grade): (u8, LootGrade),
    source: Source,
) -> Stack {
    let made = make_as_unfinished(content, rng, object, count, (level, grade), source);
    finished(content, rng, made)
}

/// [`make_as`] short of its trim: creature gear's ([`crate::gear`]), which draws its condition
/// first and then finishes each piece ([`finished`]).
pub(crate) fn make_as_unfinished(
    content: &Content,
    rng: &mut Rng,
    object: ObjectId,
    count: u32,
    (level, grade): (u8, LootGrade),
    source: Source,
) -> Stack {
    make_from(content, rng, object, count, (level, grade, 0), Some(source))
}

/// `stack` finished: trimmed to its capacity from `rng` ([`super::capacity::trim`]; no draw
/// when it fits). The very last step of every path that makes an item (the module docs' 10).
pub fn finished(content: &Content, rng: &mut Rng, stack: Stack) -> Stack {
    super::capacity::trimmed(content, rng, stack)
}

/// [`make`] at `(level, grade, luck)` ([`roll_with`]), its material drawn as `source` has it
/// (none for `None`: a starting kit's items keep the material they are listed with).
fn make_from(
    content: &Content,
    rng: &mut Rng,
    object: ObjectId,
    count: u32,
    (level, grade, luck): (u8, LootGrade, i32),
    source: Option<Source>,
) -> Stack {
    let level = level.max(1);
    let def = content.object(object);
    let rarity = match def.kind {
        Some(_) => roll_rarity(content, rng, grade, luck),
        None => Rarity::Normal,
    };
    let magic = match rarity {
        Rarity::Normal => None,
        rarity => roll_magic(content, rng, def, level, rarity),
    };
    let stack = match magic {
        Some(magic) => Stack::with_magic(object, count, magic),
        None => Stack::new(object, count),
    };
    // Deeper loot is set with a gem more often, and better made (#1436).
    let loot = source == Some(Source::Loot);
    let more = if loot {
        content.gear_condition().depth_at(level).ornament
    } else {
        0
    };
    let ornament = gem::ornament(content, rng, def, (level, more));
    // Its own parts in its base parts' order: the main part's material, then the gem it is set
    // with.
    let material = source.and_then(|source| variant::draw(content, rng, def, source, level));
    let own: Vec<_> = material.into_iter().chain(ornament).collect();
    let mut stack = if own.is_empty() {
        stack
    } else {
        stack.with_parts(own)
    };
    // A wand's rung (the spell scaling design §7.4): one draw, a wand's only.
    stack.draw_wand_rung(content, rng);
    if loot {
        super::make::make(content, rng, level, &mut stack);
    }
    stack
}

/// A rarity by the weights in content, leaving out those below `grade`; the weights of those
/// better than the least it allows `luck` percent more (Treasure hunter: magic and rare for
/// any find, rare for a find that is magic at least), rounded down.
fn roll_rarity(content: &Content, rng: &mut Rng, grade: LootGrade, luck: i32) -> Rarity {
    let affixes = content.affixes();
    if affixes.is_empty() {
        return Rarity::Normal;
    }
    let lucky = |r: Rarity, weight: u32| {
        if r > grade.least() && luck != 0 {
            let weight = crate::talent::TalentMods::scaled(u64::from(weight), luck);
            u32::try_from(weight).unwrap_or(u32::MAX)
        } else {
            weight
        }
    };
    let weights: Vec<(Rarity, u32)> = (Rarity::ALL.into_iter())
        .filter(|r| *r >= grade.least())
        .map(|r| (r, lucky(r, affixes.rarity_weight(r))))
        .collect();
    match pick_weighted(rng, &weights) {
        Some(r) => *r,
        // Every allowed rarity weighs nothing: the least allowed.
        None => grade.least(),
    }
}

/// A magic or rare item of `def` at `level`: its affixes and, if rare, its name; `None` if it
/// rolled no affix.
fn roll_magic(
    content: &Content,
    rng: &mut Rng,
    def: &ObjectDef,
    level: u8,
    rarity: Rarity,
) -> Option<Magic> {
    let affixes = content.affixes();
    let (prefixes, suffixes) = match rarity {
        Rarity::Rare => {
            let count = i32::from(*pick_weighted(rng, &affixes.rare_counts).unwrap_or(&3));
            rare_split(rng, count)
        }
        _ => {
            let [prefix, suffix, both] = affixes.magic_affixes;
            // Only the shapes the base can take: one that rolls no suffix (a pick-axe with
            // only prefixes) is never a suffix's. Every other base can take all three, so the
            // draw is as it was.
            let fits = |kind| rolls_any(content, def, level, kind);
            let shapes: Vec<((i32, i32), u32)> =
                [((1, 0), prefix), ((0, 1), suffix), ((1, 1), both)]
                    .into_iter()
                    .filter(|((p, s), _)| {
                        (*p == 0 || fits(AffixKind::Prefix)) && (*s == 0 || fits(AffixKind::Suffix))
                    })
                    .collect();
            *pick_weighted(rng, &shapes).unwrap_or(&(1, 0))
        }
    };
    affixed(content, rng, def, level, rarity, (prefixes, suffixes))
}

/// How many of a rare item's `count` affixes are prefixes and how many suffixes: one draw,
/// evenly among the splits that keep at most [`crate::content::MAX_PER_KIND`] of each.
pub(crate) fn rare_split(rng: &mut Rng, count: i32) -> (i32, i32) {
    let most = crate::content::MAX_PER_KIND as i32;
    let prefixes = rng.range((count - most).max(0), count.min(most));
    (prefixes, count - prefixes)
}

/// The affixes of a `rarity` item of `def` at `level`, `prefixes` then `suffixes` (fewer when
/// nothing more may roll), and a rare one's name; `None` if it rolled no affix.
fn affixed(
    content: &Content,
    rng: &mut Rng,
    def: &ObjectDef,
    level: u8,
    rarity: Rarity,
    (prefixes, suffixes): (i32, i32),
) -> Option<Magic> {
    let mut rolled: Vec<Affix> = Vec::new();
    for (kind, count) in [(AffixKind::Prefix, prefixes), (AffixKind::Suffix, suffixes)] {
        for _ in 0..count {
            match roll_affix(content, rng, def, level, kind, &rolled) {
                Some(affix) => rolled.push(affix),
                None => break,
            }
        }
    }
    if rolled.is_empty() {
        return None;
    }
    let name = (rarity == Rarity::Rare).then(|| rare_name(content, rng, def));
    Some(Magic {
        rarity,
        level,
        affixes: rolled,
        name,
        identified: false,
        carried: 0,
        unique: None,
    })
}

/// An heirloom (spec §8 "Talents": Father's sword): `object` with `count` affixes rolled at item
/// level `level`, from `rng`: two are a magic item's prefix and suffix, one a prefix, and more a
/// rare item's, split as a rare item's are and named (fewer when nothing more may roll on it).
/// Unidentified, like any loot; no ornament and no other material; of `quality` if given
/// ([`Quality::worked`]); then [`finished`].
pub fn heirloom(
    content: &Content,
    rng: &mut Rng,
    object: ObjectId,
    count: u8,
    (level, quality): (u8, Option<Quality>),
) -> Stack {
    let def = content.object(object);
    let count = i32::from(count);
    let (rarity, split) = match count {
        ..=1 => (Rarity::Magic, (1, 0)),
        2 => (Rarity::Magic, (1, 1)),
        _ => (Rarity::Rare, rare_split(rng, count)),
    };
    let mut stack = match affixed(content, rng, def, level.max(1), rarity, split) {
        Some(magic) => Stack::with_magic(object, 1, magic),
        None => Stack::one(object),
    };
    if let Some(worked) = quality.and_then(|q| q.worked(def)) {
        stack.set_worked(worked);
    }
    finished(content, rng, stack)
}

/// A favoured find (spec §8 "Overworld encounters": the pedlar's offer, #808): a base item
/// that can be magic at item level `level` (not ammunition: one item), by frequency, ordered by
/// object id as [`roll`]'s are (one draw); how many affixes it has by `counts`' weights (one
/// draw); then it is made as an [`heirloom`] of that many at `level`. Better than loot: never
/// normal, and never fewer affixes than `counts` names. `None` when no base can be magic there.
pub fn favoured(
    content: &Content,
    rng: &mut Rng,
    level: u8,
    counts: &[(u8, u32)],
) -> Option<Stack> {
    let level = level.max(1);
    let mut bases: Vec<(ObjectId, u32)> = (content.object_defs().iter().enumerate())
        .filter(|(_, o)| o.frequency > 0 && o.level <= level && o.kind != Some(ItemKind::Ammo))
        .filter(|(_, o)| !o.stackable && can_be_magic(content, o, level))
        .map(|(i, o)| (ObjectId(i as u16), o.frequency))
        .collect();
    bases.sort_by(|a, b| content.object(a.0).id.cmp(&content.object(b.0).id));
    let object = *pick_weighted(rng, &bases)?;
    let count = *pick_weighted(rng, counts)?;
    Some(heirloom(content, rng, object, count, (level, None)))
}

/// Whether some affix of `kind` may roll on base item `def` at item level `level` (what
/// [`roll_affix`] would draw among, before what the item has already rolled).
fn rolls_any(content: &Content, def: &ObjectDef, level: u8, kind: AffixKind) -> bool {
    def.kind.is_some_and(|item| {
        (content.affixes().all().iter()).any(|a| {
            a.kind == kind && a.weight > 0 && a.tiers_at(level) > 0 && may_roll(a, item, def)
        })
    })
}

/// Whether `def` can be made magic at item level `level`: a base item some affix may roll on.
/// Loot that must be magic draws only among those (a tool before any affix rolls on tools is
/// never drawn so).
fn can_be_magic(content: &Content, def: &ObjectDef, level: u8) -> bool {
    rolls_any(content, def, level, AffixKind::Prefix)
        || rolls_any(content, def, level, AffixKind::Suffix)
}

/// One affix of `kind` for `def` at `level` of a category none of `rolled` is of.
pub(crate) fn roll_affix(
    content: &Content,
    rng: &mut Rng,
    def: &ObjectDef,
    level: u8,
    kind: AffixKind,
    rolled: &[Affix],
) -> Option<Affix> {
    roll_affix_within(content, rng, def, level, kind, rolled, None)
}

/// The affixes [`roll_affix_within`] draws among, as indices into the affixes with their
/// weights, by affix id (like the bases: not by the order the affix files load in).
fn candidates(
    content: &Content,
    def: &ObjectDef,
    level: u8,
    kind: AffixKind,
    rolled: &[Affix],
    families: Option<&[String]>,
) -> Vec<(usize, u32)> {
    let Some(item) = def.kind else {
        return Vec::new();
    };
    let affixes = content.affixes();
    let taken: Vec<&str> = (rolled.iter())
        .filter_map(|a| affixes.get(&a.id).map(|d| d.category.as_str()))
        .collect();
    let mut candidates: Vec<(usize, u32)> = (affixes.all().iter().enumerate())
        .filter(|(_, a)| {
            a.kind == kind
                && a.weight > 0
                && a.tiers_at(level) > 0
                && !taken.contains(&a.category.as_str())
                && families.is_none_or(|f| f.contains(&a.category))
                && may_roll(a, item, def)
        })
        .map(|(i, a)| (i, a.weight))
        .collect();
    candidates.sort_by(|a, b| affixes.all()[a.0].id.cmp(&affixes.all()[b.0].id));
    candidates
}

/// Whether [`roll_affix_within`] has anything to draw (and so draws nothing to find out).
pub(crate) fn rolls_within(
    content: &Content,
    def: &ObjectDef,
    level: u8,
    kind: AffixKind,
    rolled: &[Affix],
    families: Option<&[String]>,
) -> bool {
    !candidates(content, def, level, kind, rolled, families).is_empty()
}

/// [`roll_affix`] with the candidates narrowed to the affix categories `families` holds (a
/// crystal's colour, docs/design/enchanting.md §8; `None`: every category): the affix by weight,
/// then its tier in the window at `level`, then its value in the tier (three draws).
pub(crate) fn roll_affix_within(
    content: &Content,
    rng: &mut Rng,
    def: &ObjectDef,
    level: u8,
    kind: AffixKind,
    rolled: &[Affix],
    families: Option<&[String]>,
) -> Option<Affix> {
    let affixes = content.affixes();
    let candidates = candidates(content, def, level, kind, rolled, families);
    let a = &affixes.all()[*pick_weighted(rng, &candidates)?];
    let window = tier_window(a, level);
    let tier = window.start + rng.index(window.len());
    let t = &a.tiers[tier];
    Some(Affix {
        id: a.id.clone(),
        tier: u8::try_from(tier + 1).unwrap_or(1),
        value: rng.range(t.min, t.max),
    })
}

/// The tiers of `a` loot may roll at item level `level`, as indices into its tiers (0 the
/// first): those found there whose level is within [`TIER_WINDOW`] below it; the top tier alone
/// once the item level is past the last tier's, or when no tier found there is that close (a gap
/// in the ladder). Empty when none is found there.
pub fn tier_window(a: &AffixDef, level: u8) -> Range<usize> {
    let found = a.tiers_at(level);
    let Some(top) = found.checked_sub(1) else {
        return 0..0;
    };
    let past = a.tiers.last().is_some_and(|t| level > t.level);
    let close = (a.tiers[..found].iter()).position(|t| level - t.level < TIER_WINDOW);
    match close {
        Some(first) if !past => first..found,
        _ => top..found,
    }
}

/// A rare item's two-word name: a first word, and a second for its kind of item.
pub(crate) fn rare_name(content: &Content, rng: &mut Rng, def: &ObjectDef) -> String {
    let affixes = content.affixes();
    let first = rng.pick(&affixes.first_names).cloned().unwrap_or_default();
    let seconds = def.kind.and_then(|k| affixes.second_names.get(&k));
    let second = seconds
        .and_then(|words| rng.pick(words))
        .cloned()
        .unwrap_or_default();
    format!("{first} {second}").trim().to_string()
}

/// One of `items` drawn by weight (one draw, [`Rng::weighted`]); `None` if they weigh nothing.
pub(crate) fn pick_weighted<'a, T>(rng: &mut Rng, items: &'a [(T, u32)]) -> Option<&'a T> {
    rng.weighted(items, |(_, w)| *w).map(|(item, _)| item)
}
