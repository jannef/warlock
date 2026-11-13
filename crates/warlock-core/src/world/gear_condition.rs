//! The condition creature gear spawns in (spec §8 "Creature gear"; #1193): a rolled piece of
//! arms or armour is often damaged and rarely improved, by the creature's tribe
//! (`data/gear_condition.toml`, [`crate::content::GearConditions`]). The states are the smith's
//! ([`Worked`]): damaged, a point worse with its harm's word ("rusted"), or Fine, Marvellous,
//! Masterwork. The creature fights with it as it is, and it drops as it is.
//!
//! - **Which pieces**: every single piece [`crate::gear::roll`] made that an anvil or a knife
//!   could work (a weapon, a missile weapon, armour, or a shield with something to better; not a
//!   stack of several, ammunition, a ring, or a spirit weapon), and that is not protected
//!   ([`super::item_damage::protected`]: a unique, of Proof, or what any quest may ask for). A
//!   fixed kit (a level file's placed gear, a vault's) is never rolled, so it stays as made.
//! - **The draw**: one draw below [`PER_MILLE`] per piece, in pack order, against the odds of the
//!   creature's tribe (the first `[[tribe]]` whose tag it has, else `[default]`) with the
//!   `[[depth]]` row of the level's item level added ([`crate::content::Odds::deeper`]); for an improved
//!   piece then one draw per improvement for its [`Aspect`] among those its base item has, as the
//!   anvil draws them. The improvements are capped at `temper.toml`'s `most`, and each counts as
//!   one attempt made on it.
//! - **The kind**: by its main part's material, among the kinds its material group lists
//!   (`[kinds]`): the one the material withstands least (a tie: the first listed), never one it
//!   always withstands (100). None left, or a group not listed: plainly "damaged". So iron and
//!   steel rust, bronze and silver corrode, and wood and leather are just damaged.
//! - **The stream**: its own, `condition/<key>` with the gear stream's key (a generated level's
//!   id, drawn in spawn order, or a placed creature's actor id): the gear's draws and everything
//!   else on the level are the same with or without it.
//!
//! **Loot's make** ([`make`], #1436): loot's arms and armour (floor loot, chests, hoards, dens,
//! room loot: every find [`crate::magic::loot`] makes with `Source::Loot`) are as made but for the
//! `[[depth]]` row of their item level: Fine, Marvellous, or Masterwork by its chances, never
//! damaged. One draw below [`PER_MILLE`] from the loot stream, after the wand's rung and before
//! the trim, then the aspects as above; none where the row gives nothing (the shallow levels,
//! whose draws are as before) or for what takes no make.

use super::item_damage::protected;
use super::temper::arms;
use crate::content::{Content, CreatureId, ItemHarm, ObjectId, PER_MILLE};
use crate::kit::Pack;
use crate::magic::Worked;
use crate::magic::worked::aspects;
use crate::object::Stack;
use crate::rng::Rng;
use std::collections::BTreeSet;

/// The stream the condition of the gear rolled from `gear/<key>` is drawn from.
pub(crate) fn stream(seed: u64, key: &str) -> Rng {
    Rng::stream(seed, &format!("condition/{key}"))
}

/// Draws the condition of every piece in `pack`, just rolled for a creature of type `creature`
/// on a level of item level `level` (module docs).
pub(crate) fn apply(
    content: &Content,
    rng: &mut Rng,
    (creature, level): (CreatureId, u8),
    pack: &mut Pack,
) {
    let rules = content.gear_condition();
    let tribe = rules.odds(&content.creature(creature).tags);
    let odds = tribe.deeper(&rules.depth_at(level));
    if odds == Default::default() {
        return;
    }
    let quest = quest_objects(content);
    for stack in &mut pack.items {
        if !conditioned(content, &quest, stack) {
            continue;
        }
        let Some(improved) = odds.of(rng.below(PER_MILLE)) else {
            let mut worked = Worked::default();
            worked.damage(kind(content, stack));
            stack.set_worked(worked);
            continue;
        };
        improve(content, rng, improved, stack);
    }
}

/// Loot's make (module docs): `stack`, a find at item level `level`, made Fine, Marvellous, or
/// Masterwork by the `[[depth]]` row there, from `rng`; no draw where the row gives nothing or
/// `stack` takes no make.
pub(crate) fn make(content: &Content, rng: &mut Rng, level: u8, stack: &mut Stack) {
    let odds = content.gear_condition().depth_at(level).odds();
    if odds.improved() == 0 || !improvable(content, stack) {
        return;
    }
    if let Some(improved) = odds.of(rng.below(PER_MILLE)) {
        improve(content, rng, improved, stack);
    }
}

/// `stack` given `improved` improvements (at most `temper.toml`'s `most`), each one attempt, in
/// aspects drawn as the anvil draws them; nothing for none.
fn improve(content: &Content, rng: &mut Rng, improved: u8, stack: &mut Stack) {
    let among = aspects(content.object(stack.object));
    let most = improved.min(content.temper().most);
    if most == 0 {
        return;
    }
    let mut worked = Worked::default();
    for _ in 0..most {
        worked.attempted();
        worked.improve(among[rng.index(among.len())]);
    }
    stack.set_worked(worked);
}

/// Whether `stack` takes a condition: one that may be improved ([`improvable`]) and is not
/// protected.
pub(super) fn conditioned(content: &Content, quest: &BTreeSet<ObjectId>, stack: &Stack) -> bool {
    improvable(content, stack) && !protected(content, quest, stack)
}

/// Whether `stack` may be made finer: a single piece of arms or armour with something to better,
/// of this world, and worked by nobody yet.
fn improvable(content: &Content, stack: &Stack) -> bool {
    stack.count == 1
        && !stack.is_spirit()
        && stack.worked().is_none()
        && arms(content, stack)
        && !aspects(content.object(stack.object)).is_empty()
}

/// What damaged `stack` is called by its main part's material (module docs).
pub(crate) fn kind(content: &Content, stack: &Stack) -> ItemHarm {
    let Some(material) = stack
        .parts(content)
        .first()
        .map(|p| content.material(p.material))
    else {
        return ItemHarm::Work;
    };
    let resists = material.resists();
    let kinds = content.gear_condition().kinds(material.group.as_deref());
    (kinds.iter().copied())
        .filter(|&k| resists.of(k) < 100)
        .min_by_key(|&k| resists.of(k))
        .unwrap_or(ItemHarm::Work)
}

/// Every object a quest may ask to be brought or delivered: a fixed quest's, and any object a
/// rolled one may draw. Gear is made before a quest is given, so all of them, given or not.
pub(super) fn quest_objects(content: &Content) -> BTreeSet<ObjectId> {
    let open = |_: &str| true;
    (content.quests().iter())
        .flat_map(|def| {
            let fixed = (!def.is_rolled()).then(|| def.hand_in().map(|(o, _)| o));
            let rolled = (def.candidates(content, &open).into_iter())
                .filter_map(|c| content.object_id(&c.target));
            fixed.flatten().into_iter().chain(rolled)
        })
        .collect()
}

// Tests: `world/gear_condition_tests.rs`.
