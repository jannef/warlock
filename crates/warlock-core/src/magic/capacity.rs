//! Affix capacity (docs/design/enchanting.md §3, §4; docs/dev/enchanting.md): how many affixes,
//! prefixes and suffixes together, an item can hold, and the trim that holds every item made to
//! it.
//!
//! - **The capacity** ([`capacity`], the one read): [`BASE`] (2), +1 for jewellery (a ring, an
//!   amulet), + its quality (Fine 0, Marvellous 1, Masterwork 2: [`Worked::improvements`] less
//!   one, never below 0, none while damaged; #1444), + its main part's material's `binding` (−2
//!   to +2; silver −2, adamantium +1, mithril +2), + a gem in its ornament part (+1, a gem of the finest cut +2), + 1 for a maker's mark
//!   (Maker's mark, [`Worked::marked`](super::Worked::marked)), at least 1 and at most
//!   [`MOST`]. The three-per-kind caps stand beside it.
//! - **The trim** ([`trim`]): while an item's affixes outnumber its capacity, one of the kind
//!   (prefix or suffix) it has more of goes, one of that kind at random (one `rng.index` draw), a
//!   suffix on a tie. Its rarity and name stay: a trimmed rare is a rare still. It is the very
//!   last step of making an item ([`super::loot::finished`]), so an item that needs no trim draws
//!   nothing more.
//! - **Over capacity** after the trim is only a damaged piece (its improvements gone): nothing
//!   strips it; [`Magic::check`] does not read the capacity, so saves always load.

use super::{AffixKind, Magic};
use crate::content::Content;
use crate::object::Stack;
use crate::rng::Rng;

/// What every item holds before its kind, quality, material, and gem (the owner, 2026-11-13).
pub const BASE: i32 = 2;

/// The most affixes any item holds: three of each kind.
pub const MOST: u8 = (crate::content::MAX_PER_KIND * 2) as u8;

/// The most affixes `stack` can hold (module docs).
pub fn capacity(stack: &Stack, content: &Content) -> u8 {
    let def = content.object(stack.object);
    let jewellery = i32::from(def.is_jewellery());
    let quality = stack
        .worked()
        .map_or(0, |w| w.improvements().saturating_sub(1) as i32);
    let parts = stack.parts(content);
    let binding = parts
        .first()
        .map_or(0, |p| i32::from(content.material(p.material).binding));
    let gem = gem_term(stack, content);
    // A maker's mark holds one more (Maker's mark, the craftsperson's).
    let mark = i32::from(stack.worked().is_some_and(super::Worked::marked));
    let sum = BASE + jewellery + quality + binding + gem + mark;
    u8::try_from(sum.clamp(1, i32::from(MOST))).unwrap_or(1)
}

/// What the gem set in `stack`'s ornament part adds: 1, or 2 for the finest cut; 0 with none.
fn gem_term(stack: &Stack, content: &Content) -> i32 {
    let def = content.object(stack.object);
    let Some(ornament) = crate::gem::ornament_part(content, def) else {
        return 0;
    };
    let finest = content.gems().and_then(|g| g.cuts().len().checked_sub(1));
    (stack.parts(content).iter())
        .find(|p| p.part == Some(ornament) && content.material(p.material).gem.is_some())
        .map_or(0, |p| match p.gem {
            Some(cutting) if Some(usize::from(cutting.cut.0)) == finest => 2,
            _ => 1,
        })
}

/// Drops affixes from `magic` until it holds no more than `capacity` (module docs), each from
/// `rng`; draws nothing when it fits. Never below one affix.
pub fn trim(magic: &mut Magic, capacity: u8, rng: &mut Rng, content: &Content) {
    let capacity = usize::from(capacity.max(1));
    while magic.affixes.len() > capacity {
        let kind_of = |id: &str| content.affixes().get(id).map(|a| a.kind);
        let of = |kind: AffixKind| -> Vec<usize> {
            (magic.affixes.iter().enumerate())
                .filter(|(_, a)| kind_of(&a.id).unwrap_or(AffixKind::Suffix) == kind)
                .map(|(i, _)| i)
                .collect()
        };
        let (prefixes, suffixes) = (of(AffixKind::Prefix), of(AffixKind::Suffix));
        let among = match prefixes.len() > suffixes.len() {
            true => prefixes,
            false => suffixes,
        };
        let gone = among[rng.index(among.len())];
        magic.affixes.remove(gone);
    }
}

/// `stack` trimmed to its capacity from `rng` ([`trim`]): the last step of making an item.
pub fn trimmed(content: &Content, rng: &mut Rng, mut stack: Stack) -> Stack {
    let room = capacity(&stack, content);
    if let Some(magic) = stack.magic_mut() {
        trim(magic, room, rng, content);
    }
    stack
}

#[cfg(test)]
#[path = "capacity_tests.rs"]
mod tests;
