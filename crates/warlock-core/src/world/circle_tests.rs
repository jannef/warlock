//! The spell circles' test scenario (spec §4.8 "Spell circles", docs/design/buff-circles.md §13):
//! a scenario's `spells` are known from the start, a spawn's `status` is on the creature, and
//! what the circles are drawn from shows in perception.

use super::*;
use crate::content::SpellPurpose;
use std::sync::LazyLock;

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("built-in content")));

fn circles() -> World {
    World::new(CONTENT.clone(), "circles", None).expect("the circles hall")
}

fn status(name: &str) -> StatusId {
    CONTENT.status_id(name).unwrap()
}

#[test]
fn the_scenario_knows_every_buff_and_hex_spell() {
    let world = circles();
    let player = world.state.player();
    for spell in CONTENT.spells() {
        let runed = matches!(spell.kind, SpellPurpose::Buff | SpellPurpose::Hex);
        let id = CONTENT.spell_id(&spell.id).unwrap();
        assert_eq!(player.knows_spell(&CONTENT, id), runed, "{}", spell.id);
    }
    assert!(
        player.max_mana(&CONTENT) >= 300,
        "mana to cast them again and again"
    );
}

#[test]
fn a_chosen_character_knows_the_scenario_spells_too() {
    let spec: CharacterSpec = "human,female,warrior".parse().unwrap();
    let world = World::start_run(CONTENT.clone(), "circles", None, Some(&spec), None, false)
        .expect("world");
    let haste = CONTENT.spell_id("haste").unwrap();
    assert!(world.state.player().knows_spell(&CONTENT, haste));
}

#[test]
fn the_kobolds_are_placed_buffed_and_hexed() {
    let world = circles();
    let at = |x, y| {
        let actor = (world.state.actors.values())
            .find(|a| a.pos == Pos::new(x, y))
            .unwrap_or_else(|| panic!("an actor at ({x}, {y})"));
        actor
            .effects
            .iter()
            .map(|e| (e.status, e.severity))
            .collect::<Vec<_>>()
    };
    assert_eq!(at(11, 4), [(status("blessed"), 1)]);
    let mut h = vec![(status("weakened"), 1), (status("sluggish"), 1)];
    h.sort();
    assert_eq!(at(19, 4), h);
    let m = at(19, 8);
    assert_eq!(m.len(), 3);
    assert!(at(13, 2).is_empty(), "a plain kobold");
}

#[test]
fn perception_shows_the_seen_buffs_and_hexes() {
    let world = circles();
    let view = world.perception();
    let x = (world.state.actors.values())
        .find(|a| a.pos == Pos::new(11, 8))
        .unwrap();
    let seen: Vec<StatusId> = view.effects_of(x.id).collect();
    assert!(seen.contains(&status("blessed")), "{seen:?}");
    assert!(seen.contains(&status("ill_wished")), "{seen:?}");
    let m = (world.state.actors.values())
        .find(|a| a.pos == Pos::new(19, 8))
        .unwrap();
    let seen: Vec<StatusId> = view.effects_of(m.id).collect();
    assert!(
        seen.contains(&status("hasted")),
        "hasted is seen now: {seen:?}"
    );
    assert!(seen.contains(&status("barkskin")), "{seen:?}");
}
