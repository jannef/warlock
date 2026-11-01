//! The stat training talents (spec §8 "Stats"; the design's §9): Late bloomer (the ramp a
//! point shorter), read at its one hook.

use super::*;
use crate::character::CharacterSpec;
use crate::content::PriceEase;
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

fn world(class: &str) -> World {
    let spec: CharacterSpec = format!("human,male,{class}").parse().expect("a character");
    World::with_character(BUILTIN.clone(), "showcase", Some(7), Some(&spec)).expect("world")
}

fn give(world: &mut World, id: &str, choice: Option<&str>) {
    world.player_mut().talents.take(id, choice);
}

#[test]
fn late_bloomer_starts_the_ramp_a_point_later() {
    let mut w = world("warrior");
    give(&mut w, "late_bloomer", None);
    let ease = w.state.player().talent_mods(w.content()).price_ease();
    assert_eq!(ease, PriceEase { ramp: 1 });
    let rules = w.content().stat_training();
    // The point reaching the potential: the base either way.
    assert_eq!(rules.eased_price(17, 18, ease), rules.base_price(17));
    // The first past it: ×2 by nature, the base for a late bloomer.
    assert_eq!(rules.price(18, 18), rules.base_price(18) * 2);
    assert_eq!(rules.eased_price(18, 18, ease), rules.base_price(18));
    // The second past it: ×4, ×2 for a late bloomer.
    assert_eq!(rules.eased_price(19, 18, ease), rules.base_price(19) * 2);
}
