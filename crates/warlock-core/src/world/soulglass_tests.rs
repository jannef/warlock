//! Soulglass (docs/design/enchanting.md §6; `world/crystal.rs`): a kill leaves its colour's
//! crystals and no corpse or crafting stock, the yield by level, a dragon's heart-crystal, the
//! rare material's chance from level 15, a creature of no colour dying as ever, another spell's
//! kill leaving its corpse, and the witch's grant at 16.

use super::super::class_active_tests::beside;
use super::super::*;
use crate::action::parse_actions;
use crate::content::CreatureId;

/// A lit hall: the player at the west end, a kobold far to the east (the template `beside`
/// copies).
const HALL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
################
#@............k#
#..............#
################
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
"##;

static CONTENT: std::sync::LazyLock<Arc<Content>> = std::sync::LazyLock::new(|| {
    let mut files = super::super::caster_tests::files();
    files.insert("levels/hall.toml", HALL);
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 1000\n",
    );
    Arc::new(Content::load(&files).expect("valid content"))
});

fn content() -> Arc<Content> {
    CONTENT.clone()
}

/// The witch in the hall, `seed` her fate, knowing Soulglass and Leech.
fn world(seed: u64) -> World {
    let spec = "human,female,witch".parse().expect("a character");
    let mut w = World::with_character(content(), "hall", Some(seed), Some(&spec)).expect("world");
    for id in ["soulglass", "leech"] {
        let spell = w.content().spell_id(id).expect("the spell");
        w.player_mut().spells.insert(spell, 1);
    }
    // No familiar in the way.
    w.state.actors.retain(|_, a| a.ally.is_none());
    w
}

/// Casts `spell` at a fresh `creature` of one hit point beside the player until it dies; the
/// pile left on its cell, as object ids.
fn kill(w: &mut World, creature: &str, spell: &str) -> Vec<String> {
    kill_with(w, creature, &|_, at| {
        format!("use:{spell}:{},{}", at.x, at.y)
    })
}

/// [`kill`] by the action `action` makes for the creature's cell, each try.
fn kill_with(
    w: &mut World,
    creature: &str,
    action: &dyn Fn(&mut World, Pos) -> String,
) -> Vec<String> {
    let id = beside(w, creature, 1);
    let at = w.state.actors[&id].pos;
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    for _ in 0..200 {
        let me = w.player_mut();
        (me.mana_base, me.mana, me.hp_base, me.hp) = (400, 400, 1000, 1000);
        // Nothing it does to the player (a gaze, a hex) stops the next try.
        me.effects = Default::default();
        let text = action(w, at);
        for a in &parse_actions(&text).expect("an action") {
            w.apply(a);
        }
        if !w.state.actors.contains_key(&id) {
            let pile = w.state.current().objects_at(at);
            return (pile.iter())
                .flat_map(|s| {
                    let name = w.content().object(s.object).id.clone();
                    std::iter::repeat_n(name, s.count as usize)
                })
                .collect();
        }
    }
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    panic!(
        "{} never killed the {creature}: {:?}",
        action(w, at),
        &said[said.len().saturating_sub(8)..]
    );
}

fn count(pile: &[String], id: &str) -> usize {
    pile.iter().filter(|o| *o == id).count()
}

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.display().starts_with(text))
}

/// The first creature type `pick` takes: one that leaves a corpse and is no animated thing.
fn a_creature(content: &Content, pick: impl Fn(CreatureId) -> bool) -> String {
    (0..content.creature_defs().len())
        .map(|i| CreatureId(u16::try_from(i).expect("few")))
        .filter(|&c| content.corpse_of(c).is_some())
        .filter(|&c| content.creature(c).animated.is_none() && !content.creature(c).unique)
        .find(|&c| pick(c))
        .map(|c| content.creature(c).id.clone())
        .expect("such a creature")
}

#[test]
fn a_kobold_leaves_a_green_crystal_and_no_corpse() {
    let mut w = world(1);
    let pile = kill(&mut w, "kobold", "soulglass");
    assert_eq!(pile, ["crystal_green"], "one crystal, nothing else");
    assert!(said(
        &w,
        "Its body turns to glass: where it fell lies a green crystal."
    ));
}

#[test]
fn a_scroll_of_it_does_the_same() {
    let mut w = world(1);
    w.player_mut().inventory.clear();
    let scroll = w
        .content()
        .object_id("scroll_of_soulglass")
        .expect("the scroll");
    let pile = kill_with(&mut w, "kobold", &|w, at| {
        let stack = crate::object::Stack::new(scroll, 1);
        w.player_mut().inventory.insert(0, stack);
        format!("read:a:{},{}", at.x, at.y)
    });
    assert_eq!(pile, ["crystal_green"]);
}

#[test]
fn leech_leaves_the_corpse() {
    let mut w = world(1);
    let pile = kill(&mut w, "kobold", "leech");
    assert_eq!(count(&pile, "kobold_corpse"), 1);
    assert_eq!(count(&pile, "crystal_green"), 0);
}

#[test]
fn the_yield_grows_by_level_and_a_dragon_leaves_a_heart_crystal() {
    let content = content();
    let crystals = content.crystals().expect("crystals");
    assert_eq!(
        [1, 9, 10, 19, 20, 40].map(|l| crystals.yield_at(l)),
        [1, 1, 2, 2, 3, 3]
    );
    // The gorgon: a dragon, level 28, red.
    let mut w = world(1);
    let pile = kill(&mut w, "gorgon", "soulglass");
    assert_eq!(count(&pile, "crystal_red"), 3);
    assert_eq!(count(&pile, "heart_crystal"), 1);
    assert_eq!(count(&pile, "gorgon_corpse"), 0);
    assert!(said(&w, "Its body turns to glass: where it fell lie 3 red"));
}

#[test]
fn a_creature_of_no_colour_dies_as_ever() {
    let content = content();
    let crystals = content.crystals().expect("crystals");
    let plain = a_creature(&content, |c| crystals.colour_of(&content, c).is_none());
    let mut w = world(1);
    let pile = kill(&mut w, &plain, "soulglass");
    assert_eq!(count(&pile, &format!("{plain}_corpse")), 1, "{plain}");
    assert!(!pile.iter().any(|o| o.starts_with("crystal_")));
}

#[test]
fn the_powerful_sometimes_leave_a_rare_material() {
    let content = content();
    let crystals = content.crystals().expect("crystals");
    let rare = &crystals.rare;
    let dragon = |c: CreatureId| content.creature(c).tags.iter().any(|t| t == "dragon");
    let coloured = |c| crystals.colour_of(&content, c).is_some() && !dragon(c);
    let strong = a_creature(&content, |c| {
        let def = content.creature(c);
        coloured(c) && def.level >= rare.level && def.natural.is_none()
    });
    let weak = a_creature(&content, |c| coloured(c) && content.creature(c).level < 10);
    let rares = |creature: &str| {
        (0..80)
            .filter(|&seed| {
                let pile = kill(&mut world(seed), creature, "soulglass");
                pile.iter()
                    .any(|o| o == "heart_crystal" || o == "soul_fetter")
            })
            .count()
    };
    let found = rares(&strong);
    assert!((1..30).contains(&found), "{strong}: {found} of 80");
    assert_eq!(rares(&weak), 0, "{weak}");
}

#[test]
fn the_witch_is_given_it_at_sixteen() {
    let content = content();
    let witch = content.class("witch").expect("the witch");
    let at = (witch.abilities.iter())
        .find(|(_, talent)| talent == "soulglass")
        .map(|(level, _)| *level);
    assert_eq!(at, Some(16));
}
