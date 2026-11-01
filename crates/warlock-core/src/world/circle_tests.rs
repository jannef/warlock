//! The spell circles' test scenario (spec §4.8 "Spell circles", docs/design/buff-circles.md §13):
//! a scenario's `spells` are known from the start, a spawn's `status` is on the creature, and
//! what the circles are drawn from shows in perception. And the circle's light (spec §8 "Status
//! effects", the design's §8): a runed status sheds its school's faint light.

use super::*;
use crate::action::parse_actions;
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

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

/// The bark-skinned kobold archer in the dark alcove, in line with the player through the gap.
const ALCOVE: Pos = Pos::new(3, 6);

fn actor_at(world: &World, p: Pos) -> &Actor {
    (world.state.actors.values())
        .find(|a| a.pos == p)
        .unwrap_or_else(|| panic!("an actor at {p}"))
}

/// The light preset of a source at `p` that the player sees, a kobold's torch left out.
fn source_at(world: &World, p: Pos) -> Option<String> {
    let view = world.perception();
    (view.light_sources().iter())
        .filter(|s| s.pos() == Some(p))
        .filter_map(|s| s.preset())
        .map(|l| CONTENT.light(l).id.clone())
        .find(|id| id != "torch")
}

#[test]
fn a_buffed_creature_in_the_dark_is_seen_by_its_circles_light_alone() {
    let world = circles();
    let view = world.perception();
    assert_eq!(view.viewer(), Pos::new(17, 6));
    assert!(view.actor_at(ALCOVE).is_some(), "seen, 14 cells off");
    let here = view.light(ALCOVE).expect("visible").brightness();
    assert!((40..60).contains(&here), "just over the threshold: {here}");
    let beside = ALCOVE.step(Dir::E);
    assert!(!view.is_visible(beside), "a cell away it is too faint");
    assert_eq!(
        source_at(&world, ALCOVE).as_deref(),
        Some("circle_transmutation")
    );
    super::light_tests::check_creature_sight(&world).expect("creature sight agrees");
}

#[test]
fn low_light_eyes_see_a_cell_away_and_uv_eyes_plainly() {
    let mut low = circles();
    run(&mut low, "vision:low_light");
    let view = low.perception();
    let beside = ALCOVE.step(Dir::E);
    assert!(
        view.is_visible(beside),
        "low-light eyes see the cell beside it"
    );
    let dim = view.light(beside).unwrap().brightness();
    assert!((12..40).contains(&dim), "{dim}");
    let mut uv = circles();
    run(&mut uv, "vision:uv_sight");
    let view = uv.perception();
    let here = view.light(ALCOVE).unwrap();
    assert!(
        here.brightness() >= 130,
        "plainly lit: {}",
        here.brightness()
    );
    assert!(here.uv > 0);
    let beside = view
        .light(ALCOVE.step(Dir::E))
        .expect("its neighbour is seen");
    assert!(beside.brightness() >= 60, "{}", beside.brightness());
    super::light_tests::check_creature_sight(&uv).expect("creature sight agrees");
}

#[test]
fn several_runed_statuses_shed_one_light_the_first_in_status_order() {
    let world = circles();
    // Blessed, hasted, bark-skinned: one light, of the first.
    let m = actor_at(&world, Pos::new(19, 8));
    let first = m.effects.iter().next().unwrap().status;
    let want = CONTENT
        .light(CONTENT.status(first).light.unwrap())
        .id
        .clone();
    let sources = world.perception().light_sources();
    let at_m = sources.iter().filter(|s| s.pos() == Some(m.pos)).count();
    // Its kobold's torch and its circle's light.
    assert_eq!(at_m, 2, "one circle light beside the torch");
    let circle = (sources.iter())
        .filter(|s| s.pos() == Some(m.pos))
        .filter_map(|s| s.preset())
        .map(|l| CONTENT.light(l).id.clone())
        .find(|id| id.starts_with("circle_"));
    assert_eq!(circle, Some(want));
}

#[test]
fn a_cast_lights_its_target_and_the_light_ends_with_the_status() {
    // The player's Light keeps the Light spell's own, brighter light; a hex marks its target.
    let mut world = circles();
    let player = world.perception().viewer();
    run(&mut world, "use:blessing");
    assert_eq!(
        source_at(&world, player).as_deref(),
        Some("circle_abjuration")
    );
    run(&mut world, "use:light");
    assert_eq!(source_at(&world, player).as_deref(), Some("spell_light"));
    run(&mut world, "use:web:13,2");
    let kobold = actor_at(&world, Pos::new(13, 2));
    let webbed = status("webbed");
    assert!(kobold.effects.has(webbed), "the web holds it (seed 1)");
    assert_eq!(
        source_at(&world, Pos::new(13, 2)).as_deref(),
        Some("circle_transmutation")
    );
    // Web wears off in a few turns, and its light goes with it.
    let id = kobold.id;
    for _ in 0..40 {
        run(&mut world, "wait");
        if !world.state.actors[&id].effects.has(webbed) {
            break;
        }
    }
    let kobold = &world.state.actors[&id];
    assert!(!kobold.effects.has(webbed), "the web is gone");
    assert_eq!(source_at(&world, kobold.pos), None, "no light without it");
}

#[test]
fn a_disguised_actor_sheds_no_circle_light() {
    let world = circles();
    let mut state = world.state.clone();
    let disguised = CONTENT.disguise_status().expect("a disguise status");
    let id = actor_at(&world, ALCOVE).id;
    let def = CONTENT.status(disguised);
    state
        .actors
        .get_mut(&id)
        .unwrap()
        .effects
        .add(disguised, def, 1, None);
    let world = World::from_state(CONTENT.clone(), *state).expect("restores");
    let view = world.perception();
    assert!(!view.is_visible(ALCOVE), "dark: its circle sheds nothing");
    assert_eq!(source_at(&world, ALCOVE), None);
}

#[test]
fn a_light_blind_caster_in_full_daylight_is_dazzled_by_their_own_blessing() {
    // The drow's glare is 250: an overcast noon in the courtyard is under it, a circle's light on
    // top is over it; at dusk the circle alone is nowhere near.
    let spec: CharacterSpec = "drow,male,warrior".parse().unwrap();
    let start = || {
        World::with_character(CONTENT.clone(), "courtyard", Some(1), Some(&spec)).expect("world")
    };
    let light = |w: &World| {
        let view = w.perception();
        view.light(view.viewer()).unwrap().brightness()
    };
    let mut noon = start();
    run(&mut noon, "weather:overcast skip:330m");
    let before = light(&noon);
    assert!(!noon.perception().dazzled(), "{before}");
    run(&mut noon, "afflict:blessed");
    let after = light(&noon);
    assert!(after > before, "{before} -> {after}");
    assert!(noon.perception().dazzled(), "{before} -> {after}");
    let mut dusk = start();
    run(&mut dusk, "weather:overcast skip:12h afflict:blessed");
    assert!(!dusk.perception().dazzled(), "{}", light(&dusk));
}
