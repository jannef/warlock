//! Seals and the rare materials (docs/design/enchanting.md §9; docs/dev/enchanting.md "Seals and
//! the rare materials"): what Enchanting's recipes take past the item and the crystals.
//!
//! - **The picks** ([`slots`]): the item, a crystal for each the recipe takes, then up to
//!   [`MOST_SEALS`] seals (none for Unbind), then Recast's affix kept (a step only while a
//!   soul-fetter is carried; it comes last, so a wizard without it gives the same picks). A seal's
//!   pick is a family's index in `data/crystals.toml`'s `[[family]]` list, the affix kept's its
//!   index among the item's affixes: `craft:kindle:a,b,4`, `craft:recast:a,b,-,-,-,0`.
//! - **A seal** shuts one family (an affix category) out of the roll: it must be of the colour of
//!   a crystal picked (Imbue: of any of its three), sealed once, and one the item could take and
//!   does not have (Recast: its kept affix is all it keeps), else "There is nothing to seal
//!   there.". Each costs one crystal more, from the first crystal picked whose colour holds the
//!   family with one to spare ([`check`]), and `data/enchanting.toml`'s `seal_seconds` more.
//! - **The heart-crystal** (`[rare] heart`): Quicken on a rare takes one beside its crystal, the
//!   first carried; **the soul-fetter** (`[rare] fetter`): Recast keeping an affix takes one, the
//!   first carried. Both spent made, failed, or warped.

use super::Actor;
use super::craft_make::{CraftOption, CraftStep, CraftStepKind};
use crate::content::{AffixKind, Content, EnchantKind, ObjectDef, ObjectId};
use crate::magic::Affix;
use crate::magic::describe::phrase;
use crate::magic::loot::rolls_within;
use std::ops::Range;

/// The most seals one roll may take.
pub const MOST_SEALS: usize = 3;

/// Where each pick of an enchanting goes, by its index.
pub(super) struct Slots {
    pub crystals: Range<usize>,
    pub seals: Range<usize>,
    /// Recast's affix kept.
    pub affix: Option<usize>,
}

impl Slots {
    /// The most picks the work takes.
    pub fn len(&self) -> usize {
        self.affix.map_or(self.seals.end, |a| a + 1)
    }
}

/// The picks of a recipe of `kind` taking `crystals` (the module docs).
pub(super) fn slots(kind: EnchantKind, crystals: u8) -> Slots {
    let end = 1 + usize::from(crystals);
    let seals = match kind {
        EnchantKind::Unbind => end..end,
        _ => end..end + MOST_SEALS,
    };
    let affix = (kind == EnchantKind::Recast).then_some(seals.end);
    Slots {
        crystals: 1..end,
        seals,
        affix,
    }
}

/// The first stack of `object` `player` carries.
pub(super) fn carried(player: &Actor, object: ObjectId) -> Option<usize> {
    (player.inventory.iter()).position(|s| s.object == object)
}

/// Whether an item of `def` at item level `level` that keeps `had` could take an affix of
/// `category` of either kind (what a seal on it shuts out).
pub(super) fn sealable(
    content: &Content,
    (def, level): (&ObjectDef, u8),
    had: &[Affix],
    category: &str,
) -> bool {
    let family = [category.to_string()];
    [AffixKind::Prefix, AffixKind::Suffix]
        .into_iter()
        .any(|k| rolls_within(content, def, level, k, had, Some(&family)))
}

/// The seals checked, in pick order: the categories they shut, and the crystal each is paid
/// with (its position).
#[derive(Default)]
pub(super) struct Sealed {
    pub categories: Vec<String>,
    pub crystals: Vec<usize>,
}

/// `seals` (family indices) checked for an item of `def` at `level` keeping `had`, paid from
/// `crystals` (the positions picked, in order; the module docs), or why not.
pub(super) fn check(
    content: &Content,
    player: &Actor,
    item: (&ObjectDef, u8),
    had: &[Affix],
    crystals: &[usize],
    seals: &[usize],
) -> Result<Sealed, String> {
    let mut sealed = Sealed::default();
    if seals.is_empty() {
        return Ok(sealed);
    }
    let Some(table) = content.crystals() else {
        return Err("There is nothing to seal there.".into());
    };
    let colour =
        |at: usize| (player.inventory.get(at)).and_then(|s| table.colour_of_object(s.object));
    for &seal in seals {
        let (category, word) = table.families.get(seal).ok_or("There is no such seal.")?;
        if sealed.categories.contains(category) {
            return Err("You have sealed that already.".into());
        }
        let holds = |at: &usize| colour(*at).is_some_and(|c| c.families.contains(category));
        if !crystals.iter().any(holds) {
            return Err(match crystals {
                [one] => format!(
                    "The {} crystal holds nothing of {word}.",
                    colour(*one).map_or("", |c| c.name.as_str())
                ),
                _ => format!("The crystals hold nothing of {word}."),
            });
        }
        if !sealable(content, item, had, category) {
            return Err("There is nothing to seal there.".into());
        }
        let used = |at: usize| {
            (crystals.iter().chain(&sealed.crystals))
                .filter(|&&c| c == at)
                .count()
        };
        let pay = (crystals.iter().copied())
            .find(|at| holds(at) && used(*at) < player.inventory[*at].count as usize)
            .ok_or("You carry too few crystals for the seals.")?;
        sealed.categories.push(category.clone());
        sealed.crystals.push(pay);
    }
    Ok(sealed)
}

/// The words of `categories` sealed, joined: "fire and the critical".
pub(super) fn words(content: &Content, categories: &[String]) -> String {
    let table = content.crystals();
    let words: Vec<&str> = (categories.iter())
        .map(|c| (table.and_then(|t| t.family_word(c))).unwrap_or(c.as_str()))
        .collect();
    crate::text::and_list(&words)
}

/// A seal step's label: "seal", "second seal", "third seal".
fn seal_label(n: usize) -> &'static str {
    match n {
        0 => "seal",
        1 => "second seal",
        _ => "third seal",
    }
}

/// What the seal and affix steps need of the picks made: the item's def, its roll's level, and
/// what it keeps through the work, if one is picked that the work may take.
pub(super) struct Item<'a> {
    pub def: &'a ObjectDef,
    pub level: u8,
    pub had: Vec<Affix>,
    pub affixes: &'a [Affix],
}

/// The seal steps and Recast's affix step of a recipe of `kind` taking `crystals`, as the picks
/// made so far narrow them: the families of the crystals picked (of every crystal carried
/// before one is), less those sealed before and, with the item picked, those it could not take.
pub(super) fn steps(
    content: &Content,
    player: &Actor,
    kind: EnchantKind,
    crystals: u8,
    picks: &[Option<usize>],
    item: Option<Item<'_>>,
) -> Vec<CraftStep> {
    let slots = slots(kind, crystals);
    let Some(table) = content.crystals() else {
        return Vec::new();
    };
    let pick = |i: usize| picks.get(i).copied().flatten();
    let picked: Vec<usize> = slots.crystals.clone().filter_map(pick).collect();
    let colours: Vec<_> = match picked.is_empty() {
        true => (player.inventory.iter())
            .filter_map(|s| table.colour_of_object(s.object))
            .collect(),
        false => (picked.iter())
            .filter_map(|&at| player.inventory.get(at))
            .filter_map(|s| table.colour_of_object(s.object))
            .collect(),
    };
    let option = |at: usize, name: String, short: String| CraftOption {
        at,
        name,
        short,
        takes: 1,
        each: false,
        enough: true,
    };
    let mut steps = Vec::new();
    for (n, slot) in slots.seals.clone().enumerate() {
        let before: Vec<usize> = (slots.seals.start..slot).filter_map(pick).collect();
        let mut options: Vec<CraftOption> = (table.families.iter().enumerate())
            .filter(|(i, (category, _))| {
                !before.contains(i)
                    && colours.iter().any(|c| c.families.contains(category))
                    && (item.as_ref())
                        .is_none_or(|it| sealable(content, (it.def, it.level), &it.had, category))
            })
            .map(|(i, (_, word))| option(i, word.clone(), word.clone()))
            .collect();
        options.sort_by(|a, b| a.name.cmp(&b.name));
        steps.push(CraftStep {
            kind: CraftStepKind::Family,
            label: seal_label(n).into(),
            optional: true,
            options,
        });
    }
    let fetter = table.rare.fetter;
    if slots.affix.is_some() && carried(player, fetter).is_some() {
        let options = item.map_or_else(Vec::new, |it| {
            let base = it.def.kind;
            (it.affixes.iter().enumerate())
                .filter_map(|(i, a)| {
                    let def = content.affixes().get(&a.id)?;
                    let rolled = crate::magic::Rolled {
                        def,
                        tier: a.tier,
                        value: a.value,
                    };
                    let does = base.map_or_else(String::new, |k| phrase(&rolled, k, content));
                    Some(option(i, format!("{}: {does}", def.name), def.name.clone()))
                })
                .collect()
        });
        steps.push(CraftStep {
            kind: CraftStepKind::Affix,
            label: "affix kept".into(),
            optional: true,
            options,
        });
    }
    steps
}

#[cfg(test)]
#[path = "seal_tests.rs"]
mod tests;
