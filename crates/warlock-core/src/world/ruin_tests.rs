//! Ruin (spec §8 "Spells": "Ruin"; #1100): an area spell's damage type ruins the items lying in
//! it by `data/ruin.toml`; quest objects, uniques, and what is carried are spared; the draws are
//! the ruin stream's alone.

use super::caster_tests::{reach, said, sp, world_with};
use super::spell::Caster;
use super::*;
use crate::content::{Burst, ContentFiles, DamageTypeId};
use crate::magic::{Magic, Rarity};
use crate::object::Stack;

fn kind(w: &World, id: &str) -> DamageTypeId {
    w.content().damage_type_id(id).expect("a damage type")
}

/// A burst of `kind`, radius 1, doing no harm worth a roll.
fn burst(kind: Option<DamageTypeId>) -> Burst {
    Burst {
        spell: String::new(),
        radius: 1,
        damage: "1".parse().expect("a roll"),
        reroll: 0,
        kind,
        blast: 0,
        ice: None,
    }
}

/// A pile of `count` of `object` two cells east of the player, the player seeing it; its cell.
fn lay(w: &mut World, object: &str, count: u32) -> Pos {
    let content = w.content().clone();
    let object = content.object_id(object).expect("an object");
    let here = w.player_pos();
    let at = Pos::new(here.x + 2, here.y);
    let mut pile = w
        .state
        .current()
        .objects
        .get(&at)
        .cloned()
        .unwrap_or_default();
    pile.push(Stack::new(object, count));
    w.state.current_mut().set_pile(at, pile);
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    at
}

fn count(w: &World, at: Pos, object: &str) -> u32 {
    let object = w.content().object_id(object).expect("an object");
    (w.state.current().objects.get(&at).into_iter().flatten())
        .filter(|s| s.object == object)
        .map(|s| s.count)
        .sum()
}

/// `caster` bursts `burst` over `at`.
fn blow(w: &mut World, caster: Caster, at: Pos, burst: &Burst) {
    let here = w.player_pos();
    w.cast_burst(caster, (here, at), burst, &mut Vec::new());
}

fn wizard() -> World {
    world_with(Arc::new(Content::builtin().expect("content")), "wizard")
}

#[test]
fn fire_burns_scrolls_by_the_table_and_leaves_steel() {
    let mut w = wizard();
    let at = lay(&mut w, "scroll_of_fireball", 200);
    lay(&mut w, "long_sword", 1);
    let fire = Some(kind(&w, "fire"));
    let player = w.state.player;
    blow(&mut w, Caster::Actor(player), at, &burst(fire));
    let left = count(&w, at, "scroll_of_fireball");
    // Each at 50 %: about half go.
    assert!((60..=140).contains(&left), "{left} of 200 left");
    assert_eq!(count(&w, at, "long_sword"), 1, "nothing burns steel");
    let lost = 200 - left;
    assert!(
        // Unread, by its label: "102 scrolls labelled LORZUN burn up."
        (said(&w).iter()).any(
            |m| m.starts_with(&format!("{lost} scrolls labelled ")) && m.ends_with(" burn up.")
        ),
        "{:?}",
        said(&w)
    );
    // The same seed burns the same scrolls.
    let mut again = wizard();
    lay(&mut again, "scroll_of_fireball", 200);
    lay(&mut again, "long_sword", 1);
    blow(&mut again, Caster::Actor(player), at, &burst(fire));
    assert_eq!(count(&again, at, "scroll_of_fireball"), left);
}

#[test]
fn one_lost_is_told_by_its_name_and_a_whole_stack_goes() {
    let mut w = wizard();
    let at = lay(&mut w, "scroll_of_fireball", 1);
    let fire = Some(kind(&w, "fire"));
    let player = w.state.player;
    for _ in 0..40 {
        blow(&mut w, Caster::Actor(player), at, &burst(fire));
    }
    assert_eq!(count(&w, at, "scroll_of_fireball"), 0, "40 tries at a half");
    assert!(
        !w.state.current().objects.contains_key(&at),
        "the pile is gone"
    );
    let told = (said(&w).iter())
        .filter(|m| m.ends_with(" burns up."))
        .count();
    assert_eq!(told, 1, "{:?}", said(&w));
}

#[test]
fn quest_objects_and_uniques_survive_any_number_of_casts() {
    // Fire takes every apple and every long sword it reaches, here.
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/ruin.toml",
        "[[ruin]]\ntype = \"fire\"\nclass = \"food\"\npercent = 100\nsays = [\"a\", \"b\"]\n\
         [[ruin]]\ntype = \"fire\"\nclass = \"melee_weapons\"\npercent = 100\nsays = [\"a\", \"b\"]\n",
    );
    let mut w = world_with(Arc::new(Content::load(&files).expect("content")), "wizard");
    // The showcase's apples quest asks for apples; its letter is delivered.
    let at = lay(&mut w, "apple", 5);
    lay(&mut w, "bread", 3);
    let sword = w.content().object_id("long_sword").expect("a sword");
    let magic = Magic {
        rarity: Rarity::Rare,
        level: 1,
        affixes: Vec::new(),
        name: Some("Widowmaker".into()),
        identified: false,
        carried: 0,
        unique: Some("widowmaker".into()),
    };
    let proof = Magic {
        rarity: Rarity::Magic,
        level: 6,
        affixes: vec![crate::magic::Affix {
            id: "of_proof".into(),
            tier: 1,
            value: 1,
        }],
        name: None,
        identified: false,
        carried: 0,
        unique: None,
    };
    let mut pile = w.state.current().objects[&at].clone();
    pile.push(Stack::with_magic(sword, 1, magic));
    pile.push(Stack::with_magic(sword, 1, proof));
    pile.push(Stack::new(sword, 1));
    // #1444: nor does adamantium, which nothing harms.
    let adamantium = w.content().material_id("adamantium").expect("adamantium");
    let part = crate::item_parts::ItemPart::material(w.content(), None, adamantium);
    pile.push(Stack::new(sword, 1).with_parts(vec![part]));
    w.state.current_mut().set_pile(at, pile);
    let fire = Some(kind(&w, "fire"));
    let player = w.state.player;
    for _ in 0..20 {
        blow(&mut w, Caster::Actor(player), at, &burst(fire));
    }
    assert_eq!(count(&w, at, "apple"), 5);
    assert_eq!(
        count(&w, at, "long_sword"),
        3,
        "a unique, one of Proof, and one of adamantium never burn, a plain one does"
    );
    assert_eq!(count(&w, at, "bread"), 0, "food no quest asks for burns");
}

#[test]
fn what_is_carried_is_untouched_and_traps_ruin_nothing() {
    let mut w = wizard();
    let fire = Some(kind(&w, "fire"));
    let scroll = w
        .content()
        .object_id("scroll_of_fireball")
        .expect("a scroll");
    w.player_mut().inventory.push(Stack::new(scroll, 50));
    let carried = |w: &World| {
        (w.state.player().inventory.iter())
            .filter(|s| s.object == scroll)
            .map(|s| s.count)
            .sum::<u32>()
    };
    let here = w.player_pos();
    let player = w.state.player;
    blow(&mut w, Caster::Actor(player), here, &burst(fire));
    assert_eq!(carried(&w), 50);
    // A trap's burst, and a spell with no damage type, draw nothing and ruin nothing.
    let at = lay(&mut w, "scroll_of_fireball", 50);
    let trap = w.content().trap_id("fireball").expect("a fireball trap");
    w.state.ruin_rng = None;
    blow(&mut w, Caster::Trap(trap), at, &burst(fire));
    blow(&mut w, Caster::Actor(player), at, &burst(None));
    assert_eq!(count(&w, at, "scroll_of_fireball"), 50);
    assert!(w.state.ruin_rng.is_none(), "no draw from the ruin stream");
}

#[test]
fn a_fireball_cast_burns_and_draws_from_its_own_stream() {
    let mut w = wizard();
    reach(&mut w, 40);
    w.player_mut().spells.insert(sp("fireball"), 1);
    w.player_mut().mana = 100;
    let at = lay(&mut w, "scroll_of_fireball", 100);
    let rng = w.state.rng.clone();
    let mut other = w.clone();
    w.apply(&Action::Cast("fireball".into(), Some(at)));
    assert!(count(&w, at, "scroll_of_fireball") < 100);
    // The same cast over no pile rolls the gameplay stream the same.
    other.state.current_mut().set_pile(at, Vec::new());
    other.apply(&Action::Cast("fireball".into(), Some(at)));
    assert_eq!(w.state.rng, other.state.rng);
    assert_ne!(w.state.rng, rng, "the cast rolled its damage");
    assert!(other.state.ruin_rng.is_none());
}
