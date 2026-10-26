//! Objects (spec §5, §6): piles, picking up and dropping, inventories, weights, and memory.

use super::*;
use crate::action::{Positions, parse_actions};
use crate::content::ContentFiles;
use crate::dump::{Layer, layer_rows};
use crate::perception::CellView;
use crate::replay::Replay;

// Room A (x 1..5) with a pile, some arrows, a single dagger, and a heap of daggers; a closed door
// at (6, 2) to room B, where a ruby lies and a kobold carrying a spear and coins stands. Fully
// lit, so sight is line of sight.
const LEVEL: &str = r##"
name = "Store"
ambient = [255, 255, 255, 0]
map = '''
###########
#@.ae.#...#
#..d..+.b.#
#.....#..k#
###########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"a" = { terrain = "floor", objects = ["dagger", "arrow*5"] }
"e" = { terrain = "floor", objects = ["arrow*3"] }
"d" = { terrain = "floor", objects = ["dagger"] }
"b" = { terrain = "floor", objects = ["gem:ruby"] }
"k" = { terrain = "floor", creature = "kobold", carry = ["spear", "gold_coin*3"] }

[[pile]]
at = [5, 3]
objects = ["dagger*30"]
"##;

const PILE: Pos = Pos::new(3, 1);
const ARROWS: Pos = Pos::new(4, 1);
const DAGGER: Pos = Pos::new(3, 2);
const RUBY: Pos = Pos::new(8, 2);
const HEAP: Pos = Pos::new(5, 3);
const DOOR: Pos = Pos::new(6, 2);

fn content() -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    // At peace: the kobold only carries objects here; it neither hunts the player nor blocks them.
    files.insert(
        "data/factions.toml",
        "[[faction]]\nid = \"player\"\nname = \"you\"\n\
         [[faction]]\nid = \"vermin\"\nname = \"vermin\"\n\
         [[faction]]\nid = \"kobolds\"\nname = \"kobolds\"\n\
         [[faction]]\nid = \"goblins\"\nname = \"goblins\"\n\
         [[faction]]\nid = \"undead\"\nname = \"undead\"\n\
         [[faction]]\nid = \"marsh\"\nname = \"marsh things\"\n\
         [[faction]]\nid = \"cult\"\nname = \"cultists\"\n\
         [[faction]]\nid = \"hags\"\nname = \"hags\"\n\
         [[faction]]\nid = \"duergar\"\nname = \"duergar\"\n\
         [[faction]]\nid = \"fire\"\nname = \"fire things\"\n\
         [[faction]]\nid = \"seelie\"\nname = \"Seelie court\"\n\
         [[faction]]\nid = \"unseelie\"\nname = \"Unseelie court\"\n\
         [[faction]]\nid = \"yugoloth\"\nname = \"yugoloths\"\n\
         [[faction]]\nid = \"wildlife\"\nname = \"wildlife\"\n\
         [[faction]]\nid = \"peaceful\"\nname = \"peaceful folk\"\n\
         [[faction]]\nid = \"merchants\"\nname = \"merchants\"\n\
         [[faction]]\nid = \"outraged\"\nname = \"outraged\"\nhostile = [\"player\"]\n",
    );
    files.insert("levels/store.toml", LEVEL);
    files.insert(
        "scenarios/store.toml",
        "name = \"Store\"\nstart_level = \"store\"\n",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
}

fn world() -> World {
    World::new(content(), "store", None).expect("world")
}

fn run(w: &mut World, actions: &str) -> Outcome {
    let mut last = Outcome::default();
    for action in parse_actions(actions).expect("actions parse") {
        last = w.apply(&action);
    }
    last
}

fn ids(w: &World, stacks: &[Stack]) -> Vec<(String, u32)> {
    stacks
        .iter()
        .map(|s| (w.content().object(s.object).id.clone(), s.count))
        .collect()
}

fn known(w: &World, p: Pos) -> Vec<(String, u32)> {
    ids(w, w.perception().objects(p))
}

fn truth(w: &World, p: Pos) -> Vec<(String, u32)> {
    ids(w, w.ground_truth().0.objects_at(p))
}

fn inventory(w: &World) -> Vec<(String, u32)> {
    ids(w, w.perception().inventory())
}

fn last_message(w: &World) -> String {
    w.messages()
        .iter()
        .last()
        .map(|m| m.text.clone())
        .unwrap_or_default()
}

fn pair(id: &str, count: u32) -> (String, u32) {
    (id.to_string(), count)
}

#[test]
fn level_files_place_piles_and_creatures_carry_objects() {
    let w = world();
    assert_eq!(truth(&w, PILE), [pair("dagger", 1), pair("arrow", 5)]);
    assert_eq!(truth(&w, HEAP).len(), 30, "daggers do not stack");
    let (_, actors) = w.ground_truth();
    let kobold = actors
        .into_iter()
        .find(|a| a.kind == ActorKind::Creature)
        .expect("the kobold");
    assert_eq!(
        ids(&w, &kobold.inventory),
        [pair("spear", 1), pair("gold_coin", 3)]
    );
}

#[test]
fn the_player_sees_piles_in_sight_and_nothing_behind_the_door() {
    let w = world();
    assert_eq!(known(&w, PILE), [pair("dagger", 1), pair("arrow", 5)]);
    assert!(w.perception().sees_objects_at(PILE));
    // The ruby lies in the closed room: ground truth has it, perception and the dump do not.
    assert_eq!(truth(&w, RUBY), [pair("gem", 1)]);
    assert!(known(&w, RUBY).is_empty());
    let ruby = w
        .content()
        .object(w.content().object_id("gem").unwrap())
        .glyph;
    let view = layer_rows(&w, Layer::View).join("\n");
    assert!(!view.contains(ruby), "{view}");
    assert!(view.contains(')'), "the dagger on top of the pile shows");
    assert!(layer_rows(&w, Layer::Truth).join("\n").contains(ruby));
}

#[test]
fn stepping_onto_objects_says_what_lies_there() {
    let mut w = world();
    run(&mut w, "move:E move:E");
    assert_eq!(last_message(&w), "There are several objects here.");
    run(&mut w, "move:S");
    assert_eq!(last_message(&w), "You see here a dagger.");
}

#[test]
fn picking_up_and_dropping_take_the_level_turn() {
    let mut w = world();
    run(&mut w, "move:E move:E");
    let turn = w.turn();
    let outcome = run(&mut w, "pickup:b");
    assert!(outcome.took_turn);
    assert_eq!(outcome.elapsed, Duration::seconds(6));
    assert_eq!(w.turn(), turn + 1);
    assert_eq!(last_message(&w), "You pick up 5 arrows.");
    assert_eq!(inventory(&w), [pair("arrow", 5)]);
    assert_eq!(truth(&w, PILE), [pair("dagger", 1)]);
    assert_eq!(known(&w, PILE), [pair("dagger", 1)], "seen at once");

    let outcome = run(&mut w, "drop:a");
    assert!(outcome.took_turn);
    assert_eq!(last_message(&w), "You drop 5 arrows.");
    assert!(inventory(&w).is_empty());
    assert_eq!(
        truth(&w, PILE),
        [pair("arrow", 5), pair("dagger", 1)],
        "dropped objects go on top"
    );
}

#[test]
fn picking_up_everything_and_dropping_it_restores_the_pile() {
    let mut w = world();
    run(&mut w, "move:E move:E");
    let before = truth(&w, PILE);
    run(&mut w, "pickup");
    assert_eq!(last_message(&w), "You pick up a dagger and 5 arrows.");
    assert!(truth(&w, PILE).is_empty());
    assert!(!w.state().current().objects.contains_key(&PILE));
    run(&mut w, "drop");
    assert_eq!(truth(&w, PILE), before);
    assert!(inventory(&w).is_empty());
}

#[test]
fn stackable_objects_join_their_stack() {
    let mut w = world();
    // Arrows dropped where arrows lie join them.
    run(&mut w, "move:E move:E pickup:b move:E drop");
    assert_eq!(truth(&w, ARROWS), [pair("arrow", 8)]);
    run(&mut w, "pickup");
    assert_eq!(last_message(&w), "You pick up 8 arrows.");
    // A dagger takes the next letter; more arrows join theirs.
    run(&mut w, "move:W pickup move:E drop:a");
    assert_eq!(inventory(&w), [pair("dagger", 1)]);
    run(&mut w, "pickup");
    assert_eq!(inventory(&w), [pair("dagger", 1), pair("arrow", 8)]);
    run(&mut w, "move:SW pickup");
    assert_eq!(
        inventory(&w),
        [pair("dagger", 1), pair("arrow", 8), pair("dagger", 1)],
        "daggers do not stack"
    );
}

#[test]
fn refused_pick_ups_and_drops_take_no_time() {
    let mut w = world();
    let refused = |w: &mut World, action: &str, message: &str| {
        let turn = w.turn();
        let outcome = run(w, action);
        assert!(!outcome.took_turn, "{action}");
        assert_eq!(w.turn(), turn, "{action}");
        assert_eq!(last_message(w), message, "{action}");
    };
    refused(&mut w, "pickup", "There is nothing here to pick up.");
    refused(&mut w, "drop", "You are not carrying anything.");
    run(&mut w, "move:E move:E");
    refused(&mut w, "pickup:c", "There is no such thing here.");
    run(&mut w, "pickup:a");
    refused(&mut w, "drop:ab", "You carry no such thing.");
    refused(&mut w, "drop:1", "You carry no such thing.");
    assert_eq!(inventory(&w), [pair("dagger", 1)]);
}

#[test]
fn the_pack_holds_any_number_of_stacks_named_by_position() {
    // No count limit (spec §6): weight is the only one. Past `z`, action text names stacks by
    // number, and equipment follows positions past the 26th.
    let mut w = world();
    run(&mut w, "move:SE move:SE move:E move:E");
    assert_eq!(w.player_pos(), HEAP);
    run(&mut w, "pickup");
    assert_eq!(inventory(&w).len(), 30);
    assert_eq!(last_message(&w), "You pick up 30 objects.");
    assert!(truth(&w, HEAP).is_empty());
    let main_hand = w.content().slot_id("main_hand").expect("main hand");
    run(&mut w, "equip:main_hand:29");
    assert_eq!(w.state.player().equipment.get(main_hand), Some(29));
    let turn = w.turn();
    run(&mut w, "drop:a,27");
    assert!(w.turn() > turn);
    assert_eq!((inventory(&w).len(), truth(&w, HEAP).len()), (28, 2));
    assert_eq!(w.state.player().equipment.get(main_hand), Some(27));
    assert_eq!(last_message(&w), "You drop a dagger and a dagger.");
    let turn = w.turn();
    run(&mut w, "drop:28");
    assert_eq!(
        (w.turn(), last_message(&w)),
        (turn, "You carry no such thing.".into())
    );
    // The dump names them as action text does.
    let rows = layer_rows(&w, Layer::Inventory);
    assert!(
        rows.iter()
            .any(|r| r.starts_with("27 - a dagger (wielded)")),
        "{rows:?}"
    );
}

#[test]
fn the_inventory_dump_groups_by_class_and_keeps_positions() {
    // A ruby picked up first, then a dagger: the dagger's class (melee weapons) comes first in
    // the list, but each keeps its position (spec §6 "Item lists").
    let mut w = world();
    run(
        &mut w,
        "move:SE move:E*3 open:E move:E*3 pickup move:W*5 pickup",
    );
    assert_eq!(inventory(&w), [pair("gem", 1), pair("dagger", 1)]);
    let rows = layer_rows(&w, Layer::Inventory);
    assert_eq!(rows[0], "Melee weapons:");
    assert!(rows[1].starts_with("b - a dagger ("), "{rows:?}");
    assert_eq!(rows[2], "Gems:");
    assert!(rows[3].starts_with("a - a ruby ("), "{rows:?}");
    assert!(rows[4].starts_with("carried "), "{rows:?}");
}

#[test]
fn weights_are_base_plus_carried() {
    let mut w = world();
    let content = w.content().clone();
    let grams = |id: &str| u64::from(content.object(content.object_id(id).unwrap()).weight);
    let player = w.perception().own_weight();
    assert_eq!((player.base, player.carried), (70_000, 0));
    assert_eq!(w.actor_weight(w.player_id()), Some(player));
    let kobold = w
        .ground_truth()
        .1
        .find(|a| a.kind == ActorKind::Creature)
        .map(|a| a.id)
        .expect("the kobold");
    let weight = w.actor_weight(kobold).expect("weighed");
    assert_eq!(weight.base, 25_000);
    assert_eq!(weight.carried, grams("spear") + 3 * grams("gold_coin"));
    assert_eq!(weight.total(), weight.base + weight.carried);

    run(&mut w, "move:E move:E pickup");
    let player = w.perception().own_weight();
    assert_eq!(player.carried, grams("dagger") + 5 * grams("arrow"));
    assert_eq!(player.total(), 70_000 + player.carried);
    let actors = layer_rows(&w, Layer::Actors).join("\n");
    assert!(
        actors.contains("kobold") && actors.contains("a spear"),
        "{actors}"
    );
}

#[test]
fn memory_keeps_piles_as_last_seen() {
    let mut w = world();
    // Into room B (the fourth step east opens the door), closing it behind: room A is
    // remembered.
    run(
        &mut w,
        "move:SE move:E move:E move:E move:E move:E move:E close:W",
    );
    assert_eq!(w.player_pos(), Pos::new(7, 2));
    assert!(matches!(w.perception().cell(PILE), CellView::Remembered(_)));
    assert_eq!(known(&w, PILE), [pair("dagger", 1), pair("arrow", 5)]);
    assert_eq!(
        known(&w, RUBY),
        [pair("gem", 1)],
        "seen once the door opened"
    );
    // The pile changes out of sight: the player still remembers it as it was.
    let level = w.state.current_level.clone();
    w.state
        .levels
        .get_mut(&level)
        .unwrap()
        .objects
        .remove(&PILE);
    w.apply(&Action::Wait);
    assert_eq!(known(&w, PILE), [pair("dagger", 1), pair("arrow", 5)]);
    let memory = layer_rows(&w, Layer::Memory);
    assert_eq!(memory[1].chars().nth(3), Some(')'), "{memory:?}");
    // Seen again, it is gone from memory too.
    run(&mut w, "open:W move:W");
    assert!(w.perception().sees_objects_at(PILE));
    assert!(known(&w, PILE).is_empty());
    assert!(!w.state().current().object_memory.contains_key(&PILE));
}

#[test]
fn a_door_with_objects_in_the_doorway_does_not_close() {
    let mut w = world();
    run(
        &mut w,
        "move:E move:E pickup:a move:S move:E move:E move:E move:E drop move:E",
    );
    assert_eq!(truth(&w, DOOR), [pair("dagger", 1)]);
    let turn = w.turn();
    run(&mut w, "close:W");
    assert_eq!(last_message(&w), "Something is in the way.");
    assert_eq!(w.turn(), turn);
    assert_eq!(terrain_id(&w, DOOR), "door_open");
}

fn terrain_id(w: &World, p: Pos) -> String {
    let t = w.ground_truth().0.terrain_at(p).expect("in bounds");
    w.content().terrain(t).id.clone()
}

#[test]
fn objects_and_inventories_survive_a_save() {
    let mut w = world();
    run(&mut w, "move:E move:E pickup:b move:S drop move:N");
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    assert_eq!(loaded.state_hash(), w.state_hash());
    assert_eq!(
        known(&loaded, DAGGER),
        [pair("arrow", 5), pair("dagger", 1)]
    );

    // A save whose objects this content does not have is refused.
    let mut state = w.state().clone();
    let level = state.current_level.clone();
    let bogus = Stack::one(crate::content::ObjectId(9999));
    state
        .levels
        .get_mut(&level)
        .unwrap()
        .objects
        .insert(PILE, vec![bogus.clone()]);
    assert!(World::from_state(w.content().clone(), state).is_err());
    let mut state = w.state().clone();
    let player = state.player;
    state.actors.get_mut(&player).unwrap().inventory.push(bogus);
    assert!(World::from_state(w.content().clone(), state).is_err());
}

#[test]
fn pick_up_and_drop_replay_deterministically() {
    let actions = parse_actions("move:E move:E pickup:b move:S pickup drop:a move:E drop").unwrap();
    let mut w = world();
    let mut replay = Replay::new(&w);
    for action in &actions {
        w.apply(action);
        replay.push(action, &w);
    }
    let text = replay.to_json();
    assert!(text.contains("\"pickup:b\"") && text.contains("\"drop:a\""));
    let (played, divergence) = Replay::from_json(&text)
        .unwrap()
        .play(w.content().clone())
        .unwrap();
    assert_eq!(divergence, None);
    assert_eq!(played.state_hash(), w.state_hash());
    assert_eq!(
        Action::PickUp(Positions::of([1])).to_string(),
        actions[2].to_string()
    );
}
