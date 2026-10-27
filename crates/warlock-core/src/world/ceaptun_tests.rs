//! Cēaptūn, the Ascmearc's market town (#336; docs/design/campaign.md §6): entered from its
//! overworld cell and left by its edge, its four shops and their goods, everyone in it at
//! peace with the player, the Mynster a sanctuary where Sister Hild cures and mends for gold, the
//! people known by their proper names, Godgifu's rumours (never a reveal; the ways they mark are `rumour_tests.rs`), the dev scenario, and
//! a replay.

use super::*;
use crate::action::parse_actions;
use crate::content::{ContentFiles, DialogueId, ServiceKind, Stance};
use crate::dump::{self, Layer};
use crate::replay::Replay;
use std::sync::LazyLock;

const SCENARIO: &str = include_str!("../../../../assets/campaigns/ascmearc/scenarios/ceaptun.toml");
const TOWN_CELL: Pos = Pos::new(18, 28);
const START: Pos = Pos::new(46, 17);
const HILD: Pos = Pos::new(27, 25);
const GODGIFU: Pos = Pos::new(46, 26);
const LEOFRIC: Pos = Pos::new(60, 26);
const WULFRUN: Pos = Pos::new(72, 26);

/// Built-in content with nobody hungering (a disease takes days to show).
/// The dev scenario carries no pack; the tests give the player 60 gold, a cure's price.
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    let text = format!("{SCENARIO}carry = [\"gold_coin*60\"]\n");
    files.insert("campaigns/ascmearc/scenarios/ceaptun.toml", &text);
    Arc::new(Content::load(&files).expect("builtin content"))
});

fn town() -> World {
    World::new(CONTENT.clone(), "ceaptun", None).expect("a run in Cēaptūn")
}

fn act(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.text.clone()).collect()
}

fn put_player(world: &mut World, at: Pos) {
    let id = world.state.player;
    world.state.actors.get_mut(&id).expect("player").pos = at;
    world.perceive(&mut Vec::new());
}

fn gold(world: &World) -> u32 {
    let coin = world.content().object_id("gold_coin").expect("gold");
    super::shop::count_of(&world.state().player().inventory, coin)
}

/// The creature on `at` in the town.
fn who(world: &World, at: Pos) -> ActorId {
    (world.state.actor_at("ceaptun", at))
        .expect("someone there")
        .id
}

#[test]
fn the_town_is_entered_from_its_overworld_cell_and_left_by_its_edge() {
    let mut w = World::new(CONTENT.clone(), "ascmearc", None).expect("the Ascmearc");
    act(&mut w, "move:E*6");
    assert_eq!(w.player_pos(), TOWN_CELL);
    assert_eq!(said(&w).last().unwrap(), "The way into Cēaptūn is here.");
    act(&mut w, "descend");
    assert_eq!(w.state.current_level, "ceaptun");
    assert_eq!(w.player_pos(), START, "on the square");
    let words = said(&w);
    assert!(words.contains(&"Market day in Cēaptūn: the square is loud.".to_string()));
    assert!(words.contains(&"You enter Cēaptūn.".to_string()));
    // No stairs anywhere: the town is left by its edge, like any outdoor area.
    let level = CONTENT.level("ceaptun").expect("the town's level");
    assert!(level.stairs.is_empty(), "{:?}", level.stairs);
    let exit = level.exit.as_ref().expect("an edge exit");
    assert_eq!((exit.to.as_str(), exit.pos), ("ascmearc", TOWN_CELL));
    // West along the main street, out of the gate, and off the edge.
    put_player(&mut w, Pos::new(0, 16));
    act(&mut w, "move:W");
    assert_eq!(w.state.current_level, "ascmearc");
    assert_eq!(w.player_pos(), TOWN_CELL);
    assert_eq!(said(&w).last().unwrap(), "You return to The Ascmearc.");
}

#[test]
fn the_dev_scenario_starts_on_the_square() {
    let w = town();
    assert_eq!(w.state.current_level, "ceaptun");
    assert_eq!(w.player_pos(), START);
    assert_eq!(w.campaign(), "ascmearc");
    assert_eq!(gold(&w), 60, "the tests' purse");
    let level = CONTENT.level("ceaptun").expect("the town's level");
    assert_eq!(level.kind, crate::content::LevelKind::Outdoor);
    assert_eq!(level.terrain.width(), 90);
    assert_eq!(level.terrain.height(), 32);
}

#[test]
fn four_shops_lay_out_only_what_they_trade() {
    let w = town();
    let shops = CONTENT.shops().expect("shops");
    let level = w.state.current();
    let kinds: Vec<&str> = (level.shops.iter())
        .map(|s| shops.get(s.kind).id.as_str())
        .collect();
    assert_eq!(
        kinds,
        ["general_store", "weaponsmith", "armourer", "provisioner"]
    );
    for shop in &level.shops {
        let def = shops.get(shop.kind);
        let goods: Vec<&Stack> = (level.objects.iter())
            .filter(|(p, _)| shop.contains(**p))
            .flat_map(|(_, pile)| pile)
            .collect();
        assert!(goods.len() >= 8, "{}: {} stacks", def.id, goods.len());
        for stack in goods {
            let name = &CONTENT.object(stack.object).id;
            assert!(
                shops.buys(&CONTENT, shop.kind, stack),
                "{} lays out {name}, which it does not trade",
                def.id
            );
        }
        let keeper = &w.state.actors[&shop.keeper];
        assert_eq!(keeper.pos, shop.post, "{}'s keeper at the post", def.id);
    }
    // Angelica root, the remedy, in the general store and the provisioner's.
    let root = CONTENT.object_id("angelica_root").expect("root");
    let selling: Vec<&str> = (level.shops.iter())
        .filter(|s| {
            (level.objects.iter())
                .any(|(p, pile)| s.contains(*p) && pile.iter().any(|st| st.object == root))
        })
        .map(|s| shops.get(s.kind).id.as_str())
        .collect();
    assert_eq!(selling, ["general_store", "provisioner"]);
}

#[test]
fn everyone_in_town_is_at_peace_with_the_player() {
    let w = town();
    let player = w.state.player;
    let others: Vec<&Actor> = (w.state.actors.values())
        .filter(|a| a.level == "ceaptun" && a.id != player)
        .collect();
    // Six townsfolk, five people, four keepers.
    assert_eq!(others.len(), 15);
    for actor in others {
        assert_ne!(
            w.stance(actor.id, player),
            Stance::Hostile,
            "{} at {:?}",
            CONTENT.creature(actor.creature).id,
            actor.pos
        );
    }
    assert_eq!(w.danger("ceaptun"), 0, "nothing comes to the town");
}

#[test]
fn the_people_are_known_by_their_names() {
    let mut w = town();
    for (at, npc, name) in [
        (HILD, "hild", "Sister Hild"),
        (GODGIFU, "godgifu", "Godgifu"),
        (LEOFRIC, "leofric", "Reeve Leofric"),
        (WULFRUN, "wulfrun", "Factor Wulfrun"),
    ] {
        let id = who(&w, at);
        assert_eq!(w.state.actors[&id].npc.as_deref(), Some(npc));
        put_player(&mut w, at.step(Dir::W));
        act(&mut w, "chat:E");
        let line = said(&w).last().unwrap().clone();
        assert!(line.starts_with(&format!("{name} says: \"")), "{line}");
        let seen = (w.perception().actors().iter())
            .find(|a| a.id == id)
            .expect("seen");
        let d = seen.dialogue.expect("a dialogue");
        assert_eq!(CONTENT.dialogue(d).name.as_deref(), Some(name));
    }
}

#[test]
fn the_mynster_is_a_sanctuary_where_sister_hild_cures_and_mends() {
    let mut w = town();
    let room = (w.state.current().room_at(HILD)).expect("the Mynster is a special room");
    assert_eq!(CONTENT.room_effect(room.effect).id, "sanctuary");
    let hild = who(&w, HILD);
    let listed: Vec<(String, Option<u32>)> = (w.topics(hild).into_iter())
        .map(|t| (t.id, t.price))
        .collect();
    assert_eq!(
        listed,
        [
            ("quest-grave_goods".to_string(), None),
            ("mynster".to_string(), None),
            ("fen".to_string(), None),
            ("cure".to_string(), Some(60)),
            ("mend".to_string(), Some(40)),
        ]
    );
    put_player(&mut w, HILD.step(Dir::W));
    act(&mut w, "ask:E:cure");
    assert_eq!(
        said(&w).last().unwrap(),
        "Sister Hild says: \"Nothing ails you, child.\""
    );
    assert_eq!(gold(&w), 60);
    act(&mut w, "infect:the_shakes rest:24h*3");
    assert!(w.state().player().diseases.shows(), "the shakes show");
    act(&mut w, "ask:E:cure");
    assert_eq!(gold(&w), 0);
    assert!(w.state().player().diseases.iter().next().is_none(), "cured");
    assert!(said(&w).contains(&"You pay Sister Hild 60 gold.".to_string()));
}

#[test]
fn factor_wulfrun_identifies_what_the_player_carries_for_gold() {
    let mut w = town();
    let wulfrun = who(&w, WULFRUN);
    let service = (w.topics(wulfrun).into_iter())
        .find(|t| t.id == "identify")
        .expect("Wulfrun appraises");
    assert_eq!((service.price, service.items), (Some(100), Some(0)));
    put_player(&mut w, WULFRUN.step(Dir::W));
    act(&mut w, "ask:E:identify");
    assert!(
        said(&w)
            .last()
            .unwrap()
            .starts_with("Factor Wulfrun says: \"You know everything")
    );
    assert_eq!(gold(&w), 60);
    // An unidentified scroll: 100 gold, more than the player has.
    let scroll = CONTENT.object_id("scroll_of_fireball").expect("fireball");
    w.player_mut()
        .inventory
        .push(crate::object::Stack::one(scroll));
    act(&mut w, "ask:E:identify");
    assert_eq!(said(&w).last().unwrap(), "You cannot pay.");
    assert_eq!(w.unidentified().len(), 1);
    // Nobody else in the campaigns offers it.
    let appraisers: Vec<String> = (0..CONTENT.dialogues().len())
        .map(|i| DialogueId(i as u16))
        .filter(|&d| (CONTENT.dialogue(d).services.iter()).any(|s| s.does == ServiceKind::Identify))
        .map(|d| CONTENT.speaker_name(d))
        .collect();
    assert_eq!(appraisers, ["Factor Wulfrun"]);
}

#[test]
fn godgifu_tells_of_the_march_but_reveals_nothing() {
    let mut w = town();
    let godgifu = who(&w, GODGIFU);
    let topics: Vec<String> = (w.topics(godgifu).into_iter()).map(|t| t.id).collect();
    assert_eq!(topics, ["barrows", "marsh", "mines", "volcano", "pass"]);
    put_player(&mut w, GODGIFU.step(Dir::W));
    act(&mut w, "chat:E");
    for topic in &topics {
        let before = said(&w).len();
        act(&mut w, &format!("ask:E:{topic}"));
        assert!(said(&w)[before].starts_with("Godgifu says: \""));
    }
    assert!(w.state.revealed.is_empty(), "rumours are never a reveal");
}

#[test]
fn the_town_at_the_start() {
    let w = town();
    let rows = dump::layer_rows(&w, Layer::View).join("\n");
    insta::with_settings!({ snapshot_path => "../snapshots" }, {
        insta::assert_snapshot!("ceaptun_start", rows);
    });
}

#[test]
fn a_run_in_the_town_replays_exactly() {
    let mut w = town();
    let mut replay = Replay::new(&w);
    let actions = "move:SW*4 move:W*15 open:SW move:SW move:S*2 chat:SE ask:SE:mynster \
                   infect:the_shakes rest:24h*3 ask:SE:cure";
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(gold(&w), 0, "the cure was paid for");
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(CONTENT.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
