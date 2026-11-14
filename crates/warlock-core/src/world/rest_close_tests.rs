//! What stops a rest (spec §8 "Rest", `rest.rs`, #1216): a hostile coming within
//! [`World::stop_within`] in view, an attack on the player (a shot or a spell, missed or
//! resisted too), a message that stops a rest (hunger worsening); never a hostile farther off
//! in view, nor a neutral creature however close. Long actions through `keep_at` (eating,
//! digging) follow the same rule.

use super::*;
use crate::action::{AttackKind, Event, Rest, parse_actions};
use crate::combat::Strike;
use crate::content::ContentFiles;
use crate::hunger::Hunger;
use crate::replay::Replay;

/// The built-in content with `map`: the player at `@` (a ration and a pick-axe under them), a
/// creature of type `them` at `k`, a kobold walking west and back at `r`, a deer at `d`,
/// statues (transparent, not walkable) at `S`, packed earth to dig at `E`.
fn world_seeded(map: &str, them: &str, seed: u64) -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_room.toml",
        &format!(
            "name = \"Test\"\nambient = [200, 200, 200, 0]\nmap = '''\n{map}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\", \
             objects = [\"ration\", \"pick_axe\"] }}\n\
             \"k\" = {{ terrain = \"floor\", creature = \"{them}\" }}\n\
             \"r\" = {{ terrain = \"floor\", creature = \"kobold\", route = \"W8 E8\" }}\n\
             \"d\" = {{ terrain = \"floor\", creature = \"deer\" }}\n\
             \"S\" = \"statue\"\n\"E\" = \"packed_earth\"\n"
        ),
    );
    files.insert(
        "scenarios/test_room.toml",
        "name = \"T\"\nstart_level = \"test_room\"\nhp = 400\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_room", Some(seed)).expect("world")
}

fn world(map: &str, them: &str) -> World {
    world_seeded(map, them, 1)
}

/// A lit hall, a creature penned behind statues seven rows off: it sees the player and never
/// comes closer. Packed earth west of the player.
const PEN: &str = "\
###############
#E@...........#
#.............#
#.............#
#.............#
#.............#
#.............#
#SSSSSSSSSSSSS#
#..k..........#
###############
";

/// A lit hall, a kobold nine cells east of the player walking west toward them.
const OPEN: &str = "\
###############
#E@........r..#
#.............#
#.............#
###############
";

fn of(w: &World, kind: &str) -> ActorId {
    let kind = w.content.creature_id(kind).expect("a type");
    (w.state.actors.values())
        .find(|a| a.creature == kind)
        .expect("there")
        .id
}

fn last(w: &World) -> String {
    (w.messages().iter().last()).map_or(String::new(), |m| m.text.clone())
}

fn dist(w: &World, id: ActorId) -> i32 {
    w.state.actors[&id].pos.chebyshev(w.player_pos())
}

fn sees(w: &World, id: ActorId) -> bool {
    w.creatures_in_view().contains(&id)
}

#[test]
fn a_hostile_in_view_farther_off_than_close_does_not_stop_a_rest() {
    let mut w = world(PEN, "kobold");
    let kobold = of(&w, "kobold");
    assert!(sees(&w, kobold) && dist(&w, kobold) > w.stop_within());
    let out = w.apply(&Action::Rest(Rest::For(Duration::hours(1))));
    assert_eq!(out.elapsed, Duration::hours(1), "{}", last(&w));
    assert_eq!(last(&w), "You rest for 1 hour.");
    assert!(sees(&w, kobold), "it was in view all along");
}

#[test]
fn a_hostile_coming_close_stops_a_rest_on_that_very_turn() {
    // Waiting turn by turn: the first turn it stands that close.
    let mut waited = world(OPEN, "deer");
    let kobold = of(&waited, "kobold");
    assert!(sees(&waited, kobold) && dist(&waited, kobold) > waited.stop_within());
    while dist(&waited, kobold) > waited.stop_within() {
        waited.apply(&Action::Wait);
        assert!(waited.turn() < 100, "it never came");
    }
    let mut rested = world(OPEN, "deer");
    rested.apply(&Action::Rest(Rest::For(Duration::hours(1))));
    assert_eq!(rested.turn(), waited.turn());
    // The player's marks may differ: a wait is a search (#1326), a rest's turns are not.
    let others = |w: &World| {
        let player = w.state().player;
        (w.state().actors.iter())
            .filter(|(id, _)| **id != player)
            .map(|(_, a)| a.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(others(&rested), others(&waited));
    assert!(
        last(&rested).starts_with("The kobold comes close; you stop resting after"),
        "{}",
        last(&rested)
    );
}

#[test]
fn a_neutral_creature_never_stops_a_rest_however_close() {
    // A deer beside the player, wandering about in the light.
    let map = "\
#######
#@d...#
#.....#
#######
";
    let mut w = world(map, "deer");
    let deer = of(&w, "deer");
    assert!(sees(&w, deer));
    let out = w.apply(&Action::Rest(Rest::For(Duration::hours(1))));
    assert_eq!(out.elapsed, Duration::hours(1), "{}", last(&w));
    assert!(!w.disturbs_rest(&w.state.actors[&deer]));
}

/// Rests in `map` with a creature of type `them` behind the statues, over seeds; asserts every
/// rest stopped on the turn of its first attack on the player, made from afar, and that one of
/// them was of `kind` with a strike `wanted` approves.
fn rest_stops_at_the_first_attack(
    map: &str,
    them: &str,
    kind: AttackKind,
    wanted: impl Fn(Option<Strike>) -> bool,
) {
    let (mut found, mut attacks) = (false, 0);
    for seed in 0..40 {
        let fresh = world_seeded(map, them, seed);
        let caster = of(&fresh, them);
        // The turn of its first attack on the player, waiting turn by turn.
        let mut waited = fresh.clone();
        let player = waited.state.player;
        let first = loop {
            let out = waited.apply(&Action::Wait);
            let attacked = (out.events.iter()).find_map(|e| match e {
                Event::Attacked {
                    attacker,
                    target,
                    kind,
                    strike,
                    ..
                } if *attacker == caster && *target == player => Some((*kind, *strike)),
                _ => None,
            });
            if attacked.is_some() || waited.turn() >= 200 {
                break attacked;
            }
        };
        // Without arrows (its gear is rolled) an archer never shoots.
        let Some(first) = first else { continue };
        attacks += 1;
        assert!(
            dist(&waited, caster) > waited.stop_within(),
            "seed {seed}: from afar"
        );
        let mut rested = fresh.clone();
        rested.apply(&Action::Rest(Rest::For(Duration::hours(1))));
        assert_eq!(
            rested.turn(),
            waited.turn(),
            "seed {seed}: {}",
            last(&rested)
        );
        let at = |w: &World| w.state.actors[&caster].pos;
        assert_eq!(at(&rested), at(&waited), "seed {seed}");
        assert!(
            last(&rested).starts_with("You stop resting after"),
            "seed {seed}"
        );
        found |= first.0 == kind && wanted(first.1);
    }
    assert!(
        attacks > 20 && found,
        "{attacks} attacks, none the attack wanted"
    );
}

#[test]
fn a_missed_arrow_from_afar_stops_a_rest() {
    // The archer shoots over the statues from seven rows off.
    let missed = |strike| matches!(strike, Some(Strike::Missed { .. }));
    rest_stops_at_the_first_attack(PEN, "kobold_archer", AttackKind::Ranged, missed);
}

#[test]
fn a_spell_from_afar_stops_a_rest_resisted_or_not() {
    let any = |_| true;
    rest_stops_at_the_first_attack(PEN, "hobgoblin_warcaster", AttackKind::Spell, any);
}

/// The position in the player's pack of their stack of `id`, as action text names it.
fn carried(w: &World, id: &str) -> usize {
    let id = w.content.object_id(id).expect("an object");
    let pack = &w.state.player().inventory;
    (pack.iter().position(|s| s.object == id)).expect("carried")
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

#[test]
fn a_meal_and_digging_go_on_with_a_hostile_farther_off_in_view() {
    let mut w = world(PEN, "kobold");
    w.learn_mining();
    run(&mut w, "pickup");
    let (ration, pick) = (carried(&w, "ration"), carried(&w, "pick_axe"));
    let turn = w.turn();
    run(&mut w, &format!("eat:{ration}"));
    assert!(w.turn() > turn + 1, "a meal of more than a turn");
    assert!(!last(&w).contains("comes"), "{}", last(&w));
    let id = w.content.object_id("ration").expect("a ration");
    assert!(
        !(w.state.player().inventory.iter()).any(|s| s.object == id),
        "eaten"
    );
    // Digging is the same loop: a pick-axe into the earth west of the player.
    let pick = if pick > ration { pick - 1 } else { pick };
    let turn = w.turn();
    run(&mut w, &format!("equip:tool:{pick} dig:W"));
    assert!(w.turn() > turn + 2, "dug for more than a turn");
    assert!(!last(&w).contains("comes"), "{}", last(&w));
}

#[test]
fn a_fight_seen_far_off_is_told_and_the_rest_goes_on() {
    // The arena's pit, seen from its dark corner: blows, shots, spells, and deaths among the
    // kobolds and the rats, none of them near the player.
    let content = Arc::new(Content::builtin().expect("builtin content"));
    let mut w = World::new(content, "arena", None).expect("world");
    w.apply(&Action::Rest(Rest::For(Duration::hours(1))));
    assert_eq!(last(&w), "You rest for 1 hour.");
    let told = |t: &str| w.messages().iter().any(|m| m.text.contains(t));
    assert!(told("The giant rat dies.") && told("casts"));
}

#[test]
fn hunger_worsening_to_weak_stops_a_rest_with_a_hostile_far_off_in_view() {
    let mut w = world(PEN, "kobold");
    let player = w.state.player;
    let fed = w.state.actors.get_mut(&player).unwrap();
    fed.satiation = Some(crate::hunger::Satiation::new(725));
    assert_eq!(w.perception().own_hunger(), Some(Hunger::Hungry));
    let out = w.apply(&Action::Rest(Rest::Dawn));
    assert_eq!(w.perception().own_hunger(), Some(Hunger::Weak));
    assert!(out.elapsed < Duration::hours(1), "{:?}", out.elapsed);
    assert!(
        last(&w).starts_with("You stop resting after"),
        "{}",
        last(&w)
    );
}

#[test]
fn a_rest_beside_a_far_hostile_replays() {
    let mut w = world(PEN, "kobold");
    let mut replay = Replay::new(&w);
    for action in parse_actions("rest:1h wait rest:dawn").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(w.content.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
