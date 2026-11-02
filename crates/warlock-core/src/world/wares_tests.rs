//! The pedlar's offer (#808; `world/wares.rs`, docs/dev/encounters.md "The pedlar's offer"):
//! one favoured item at a fixed price, made the same for the same run, bought or left, refused
//! without the gold, sold once; the three uniques by a forced chance, each once a run; the load
//! checks; a save and a replay.

use super::*;
use crate::action::parse_actions;
use crate::content::{ContentFiles, Rarity};
use crate::replay::Replay;
use std::sync::LazyLock;

const ROAD: Pos = Pos::new(19, 28);
const DIALOGUE: &str = "campaigns/ascmearc/data/dialogue/road.toml";

/// The road's dialogue file as `files` has it.
fn road_text(files: &ContentFiles) -> String {
    (files.table("dialogue").find(|(path, _)| *path == DIALOGUE))
        .map(|(_, text)| text.to_string())
        .expect("the road's dialogue")
}

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    files
}

/// Built-in content with nobody hungering.
static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("builtin content")));

/// The same, the pedlar's uniques certain.
static CERTAIN: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = files();
    let text = road_text(&files);
    let text = text.replace("chance = 200,", "chance = 10000,");
    files.insert(DIALOGUE, &text);
    Arc::new(Content::load(&files).expect("certain uniques"))
});

fn act(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn said(world: &World, text: &str) -> bool {
    world.messages().iter().any(|m| m.text.contains(text))
}

/// A run of `content` on the road east of Cēaptūn at noon, the pedlar met and hailed, and set
/// down beside the player: his id and the way to him.
fn beside_the_pedlar(content: &Arc<Content>) -> (World, ActorId, Dir) {
    let mut w = World::new(content.clone(), "ascmearc", None).expect("a run");
    let id = w.state.player;
    w.state.actors.get_mut(&id).expect("player").pos = ROAD;
    w.perceive(&mut Vec::new());
    act(&mut w, "skip:4h meet:road_pedlar encounter:hail");
    let pedlar = pedlar_of(&w).expect("the pedlar");
    let me = w.player_pos();
    let level = &w.state.levels[&w.state.current_level];
    let dir = (Dir::ALL.into_iter())
        .find(|&d| {
            level.is_walkable(content, me.step(d))
                && w.state
                    .actor_at(&w.state.current_level, me.step(d))
                    .is_none()
        })
        .expect("room beside the player");
    w.state.actors.get_mut(&pedlar).expect("pedlar").pos = me.step(dir);
    w.perceive(&mut Vec::new());
    (w, pedlar, dir)
}

fn pedlar_of(w: &World) -> Option<ActorId> {
    (w.state.actors.values())
        .find(|a| w.content.creature(a.creature).id == "pedlar")
        .map(|a| a.id)
}

fn gold(w: &World, id: ActorId) -> u32 {
    let currency = w.content.shops().expect("shops").currency;
    super::shop::count_of(&w.state.actors[&id].inventory, currency)
}

/// The player's purse set to `amount` gold.
fn set_gold(w: &mut World, amount: u32) {
    let content = w.content.clone();
    let currency = content.shops().expect("shops").currency;
    let player = w.state.player;
    let pack = &mut w.state.actors.get_mut(&player).expect("player").inventory;
    pack.retain(|s| s.object != currency);
    if amount > 0 {
        crate::object::add_to_inventory(pack, Stack::new(currency, amount), &content);
    }
}

fn wares_topic(w: &World, pedlar: ActorId) -> TopicView {
    (w.topics(pedlar).into_iter())
        .find(|t| t.id == "wares")
        .expect("the offer")
}

#[test]
fn chatting_makes_one_favoured_identified_offer_the_same_each_run() {
    let (mut w, pedlar, dir) = beside_the_pedlar(&CONTENT);
    // Nothing is made before he is spoken to.
    assert!(w.state.wares.is_empty());
    assert_eq!(wares_topic(&w, pedlar).price, None);
    act(&mut w, &format!("chat:{dir:?}"));
    let wares = w.state.wares[&pedlar].clone();
    let item = wares.item.clone().expect("for sale");
    let magic = item.magic().expect("magic");
    assert_eq!(magic.rarity, Rarity::Rare);
    assert!((4..=6).contains(&magic.affixes.len()), "{magic:?}");
    assert!(magic.unique.is_none());
    // The player's level plus four, at least four.
    assert_eq!(magic.level, (w.player_level() + 4).max(4));
    assert!(w.lore().knows(&item), "a trader names his wares");
    // Its value plus 15 gold per affix per item level.
    let expected =
        item.value(&w.content) + 15 * magic.affixes.len() as u64 * u64::from(magic.level);
    assert_eq!(u64::from(wares.price), expected);
    let topic = wares_topic(&w, pedlar);
    assert_eq!(topic.price, Some(wares.price));
    let ware = topic.ware.expect("the item in words");
    assert!(ware.contains(&w.lore().listed(&item)[1..]), "{ware}");
    // Chatting again changes nothing; the same run makes the same offer.
    act(&mut w, &format!("chat:{dir:?}"));
    assert_eq!(w.state.wares[&pedlar], wares);
    let (mut again, other, dir) = beside_the_pedlar(&CONTENT);
    act(&mut again, &format!("chat:{dir:?}"));
    assert_eq!(again.state.wares[&other], wares);
}

#[test]
fn buying_pays_takes_the_time_and_he_has_nothing_more() {
    let (mut w, pedlar, dir) = beside_the_pedlar(&CONTENT);
    act(&mut w, &format!("chat:{dir:?}"));
    let wares = w.state.wares[&pedlar].clone();
    let item = wares.item.clone().expect("for sale");
    set_gold(&mut w, wares.price + 7);
    let his = gold(&w, pedlar);
    let before = w.state.time;
    act(&mut w, &format!("ask:{dir:?}:wares"));
    assert!(said(&w, &format!("{} gold for", wares.price)), "paid");
    assert_eq!(gold(&w, w.state.player), 7);
    assert_eq!(gold(&w, pedlar), his + wares.price);
    assert!(w.state.player().inventory.contains(&item));
    assert!(w.state.time > before, "a deal takes a chat's time");
    assert!(said(&w, "Pleasure doin' business"));
    assert_eq!(w.state.wares[&pedlar].item, None);
    assert_eq!(wares_topic(&w, pedlar).price, None);
    // Asked again: the only one he had, and nothing taken.
    let before = w.state.time;
    act(&mut w, &format!("ask:{dir:?}:wares"));
    assert!(said(&w, "That was the only one I 'ad"));
    assert_eq!((gold(&w, w.state.player), w.state.time), (7, before));
}

#[test]
fn without_the_gold_nothing_changes_hands() {
    let (mut w, pedlar, dir) = beside_the_pedlar(&CONTENT);
    act(&mut w, &format!("chat:{dir:?}"));
    let wares = w.state.wares[&pedlar].clone();
    set_gold(&mut w, wares.price - 1);
    let (pack, before) = (w.state.player().inventory.clone(), w.state.time);
    act(&mut w, &format!("ask:{dir:?}:wares"));
    assert!(said(&w, "You cannot pay."));
    assert_eq!(w.state.player().inventory, pack);
    assert_eq!(w.state.time, before, "no time");
    assert_eq!(w.state.wares[&pedlar], wares, "still for sale");
}

#[test]
fn the_uniques_come_by_their_chance_each_once_a_run() {
    let (mut w, pedlar, dir) = beside_the_pedlar(&CERTAIN);
    act(&mut w, &format!("chat:{dir:?}"));
    let first = w.state.wares[&pedlar].clone();
    let item = first.item.expect("for sale");
    let unique = item
        .magic()
        .and_then(|m| m.unique.clone())
        .expect("a unique");
    assert_eq!(first.price, 1500);
    assert!(w.lore().knows(&item));
    assert_eq!(w.state.uniques_offered, BTreeSet::from([unique.clone()]));
    // Three more pedlars of the same run: the other two, then a favoured find.
    let mut seen = BTreeSet::from([unique]);
    let creature = CERTAIN.creature_id("pedlar").expect("the pedlar's type");
    let dialogue = CERTAIN.dialogue_of(creature, None).expect("his dialogue");
    for n in 0..3 {
        // Another pedlar's stock, as his first chat makes it.
        let other = ActorId(9000 + n);
        w.stock_wares(other, dialogue);
        let magic = (w.state.wares[&other].item.as_ref())
            .and_then(|s| s.magic())
            .cloned()
            .expect("magic");
        match (n, magic.unique) {
            (0 | 1, Some(u)) => assert!(seen.insert(u), "each once"),
            (2, None) => assert_eq!(magic.rarity, Rarity::Rare),
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(w.state.uniques_offered, seen);
    assert_eq!(seen.len(), 3);
}

#[test]
fn the_three_uniques_are_a_ring_a_neck_and_hands_and_nowhere_else() {
    let slots: BTreeSet<String> = ["brosinga_mene", "eorcanstan", "grendles_grape"]
        .into_iter()
        .map(|id| {
            let def = CONTENT.unique(id).expect(id);
            assert!(def.power.is_some() && def.lore.is_some(), "{id}");
            let magic = def.magic();
            assert_eq!((magic.level, magic.affixes.len()), (12, 6), "{id}");
            let kind = CONTENT.object(def.object).kind.expect("a kind");
            format!("{kind:?}")
        })
        .collect();
    assert_eq!(
        slots,
        BTreeSet::from(["Amulet", "Gloves", "Ring"].map(String::from))
    );
    // A legend that places one is refused: they come only through the offer.
    let mut files = files();
    let text = road_text(&files);
    files.insert(
        DIALOGUE,
        &text.replace(
            "[\"brosinga_mene\", ",
            "[\"widowmaker\", \"brosinga_mene\", ",
        ),
    );
    let err = Content::load(&files).expect_err("Widowmaker is placed by its owner's gear");
    assert!(
        err.to_string().contains("placed only through the offer"),
        "{err}"
    );
}

#[test]
fn an_offer_is_checked_when_content_loads() {
    for (from, to, says) in [
        ("id = \"wares\"", "id = \"news\"", "a topic's"),
        ("affixes = [[4, 50]", "affixes = [[7, 50]", "counts 1 to 6"),
        ("chance = 200,", "chance = 20000,", "basis points"),
        (
            "\"grendles_grape\"]",
            "\"grendles_grape\", \"nobody\"]",
            "unknown unique",
        ),
        ("affix = 15,", "affix = 0,", "`price.affix`"),
    ] {
        let mut files = files();
        let text = road_text(&files);
        assert!(text.contains(from), "{from}");
        files.insert(DIALOGUE, &text.replacen(from, to, 1));
        let err = Content::load(&files).expect_err(to);
        assert!(err.to_string().contains(says), "{to}: {err}");
    }
}

#[test]
fn the_offer_survives_a_save_and_replays() {
    let mut w = World::new(CONTENT.clone(), "ascmearc", None).expect("a run");
    let mut replay = Replay::new(&w);
    let mut push = |w: &mut World, text: &str| {
        for action in parse_actions(text).expect("actions") {
            w.apply(&action);
            replay.push(&action, w);
        }
    };
    push(&mut w, "move:E*7 skip:4h meet:road_pedlar encounter:hail");
    // Walk up to him.
    let pedlar = pedlar_of(&w).expect("the pedlar");
    for _ in 0..40 {
        let (me, him) = (w.player_pos(), w.state.actors[&pedlar].pos);
        if me.chebyshev(him) <= 1 {
            break;
        }
        let dir = Dir::from_delta((him.x - me.x).signum(), (him.y - me.y).signum()).expect("a way");
        push(&mut w, &format!("move:{dir:?}"));
    }
    let (me, him) = (w.player_pos(), w.state.actors[&pedlar].pos);
    let dir = Dir::from_delta(him.x - me.x, him.y - me.y).expect("beside him");
    push(&mut w, &format!("chat:{dir:?} ask:{dir:?}:wares"));
    assert!(w.state.wares.contains_key(&pedlar));
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(CONTENT.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    let json = serde_json::to_string(&w.state).expect("saves");
    let state: WorldState = serde_json::from_str(&json).expect("loads");
    let loaded = World::from_state(w.content.clone(), state).expect("restores");
    assert_eq!(loaded.state.wares, w.state.wares);
    assert_eq!(loaded.state_hash(), w.state_hash());
}
