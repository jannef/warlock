//! The lenient read of saved affix tiers (docs/design/affixes-systems.md §8, "Regrading"): an
//! item made under the old ladder loads, re-tiered to the tier that holds its value, its value
//! kept; nothing is refused.

use super::*;
use crate::magic::{Affix, Magic, Rarity};
use crate::object::Stack;

fn content() -> Arc<Content> {
    Arc::new(Content::builtin().expect("built-in content"))
}

/// A magic `object` with `affixes` (id, tier, value), as a save from before the regrade may
/// hold them.
fn old_item(c: &Content, object: &str, affixes: &[(&str, u8, i32)]) -> Stack {
    let magic = Magic {
        rarity: Rarity::Rare,
        level: 7,
        affixes: (affixes.iter())
            .map(|&(id, tier, value)| Affix {
                id: id.into(),
                tier,
                value,
            })
            .collect(),
        name: Some("Grim Bite".into()),
        identified: true,
        carried: 0,
        unique: None,
    };
    Stack::with_magic(c.object_id(object).expect("object"), 1, magic)
}

fn tiers(stack: &Stack) -> Vec<(String, u8, i32)> {
    (stack.magic().expect("magic").affixes.iter())
        .map(|a| (a.id.clone(), a.tier, a.value))
        .collect()
}

#[test]
fn a_stack_made_under_the_old_ladder_loads_re_tiered() {
    let c = content();
    let mut w = World::new(c.clone(), "showcase", None).expect("world");
    // The old ladder's tops: Sharp's tier 4 rolled 3 to 4, now 2 to 3; Hardened's tier 3 was
    // 3, which its ladder now first holds at tier 4; Accurate's tier 9 never was, and 9 is past
    // every tier it has (the nearest, of two as near, the first).
    let sword = old_item(&c, "long_sword", &[("sharp", 4, 4), ("of_the_bear", 2, 2)]);
    let helm = old_item(&c, "iron_helm", &[("hardened", 3, 3), ("accurate", 9, 9)]);
    assert!(
        !sword.is_valid_for(&c),
        "the old tier no longer holds its value"
    );
    let player = w.state.player;
    w.state
        .actors
        .get_mut(&player)
        .expect("player")
        .inventory
        .push(sword);
    let me = w.state.player().pos;
    let at = Pos::new(me.x + 1, me.y);
    w.state
        .current_mut()
        .objects
        .entry(at)
        .or_default()
        .push(helm);
    let bytes = crate::save::encode(&w);
    let back = crate::save::decode(&bytes, c.clone()).expect("the save loads");
    let pack = &back.state.player().inventory;
    let sword = pack.last().expect("the sword");
    assert_eq!(
        tiers(sword),
        [("sharp".into(), 5, 4), ("of_the_bear".into(), 2, 2)],
        "re-tiered to the first tier holding 4, the value kept; the Bear untouched"
    );
    let helm = back
        .state
        .current()
        .objects_at(at)
        .last()
        .expect("the helm");
    assert_eq!(
        tiers(helm),
        [("hardened".into(), 4, 3), ("accurate".into(), 4, 9)]
    );
    assert!(sword.is_valid_for(&c) && helm.is_valid_for(&c));
    assert_eq!(back.retiered().len(), 3, "{:?}", back.retiered());
    assert!(back.retiered()[0].contains("`sharp` re-tiered from tier 4 to 5 (value 4)"));
    // Saved again, it reads as it is: nothing more to move.
    let again = crate::save::decode(&crate::save::encode(&back), c).expect("loads");
    assert!(again.retiered().is_empty());
}

#[test]
fn a_world_made_new_re_tiers_nothing() {
    let w = World::new(content(), "showcase", None).expect("world");
    assert!(w.retiered().is_empty());
    let back = crate::save::decode(&crate::save::encode(&w), content()).expect("loads");
    assert!(back.retiered().is_empty(), "{:?}", back.retiered());
}
