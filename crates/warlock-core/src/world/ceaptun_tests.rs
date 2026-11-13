//! Cēaptūn, the Ascmearc's market town (#336; docs/design/campaign.md §6): entered from its
//! overworld cell and left by its edge, its two shops and their goods (and nothing sold before
//! #1248 left unsold), the private houses, the town's animals (never a quest's kill), everyone in it at
//! peace with the player, the Mynster a sanctuary where Sister Hild cures and mends for gold, the
//! people known by their proper names, Eadric the smith teaching Metalcraft (#1394), Godgifu's rumours (never a reveal; the ways they mark are `rumour_tests.rs`), the dev scenario, and
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
/// Eadric the smith, the smithy's keeper, at his post.
const EADRIC: Pos = Pos::new(39, 8);
/// Ceolred, his apprentice, beside the smith's fire.
const CEOLRED: Pos = Pos::new(36, 4);

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
fn two_shops_lay_out_only_what_they_trade() {
    let w = town();
    let shops = CONTENT.shops().expect("shops");
    let level = w.state.current();
    let kinds: Vec<&str> = (level.shops.iter())
        .map(|s| shops.get(s.kind).id.as_str())
        .collect();
    assert_eq!(kinds, ["town_store", "smithy"]);
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
    // Angelica root, the remedy, in the general store.
    let root = CONTENT.object_id("angelica_root").expect("root");
    let selling: Vec<&str> = (level.shops.iter())
        .filter(|s| {
            (level.objects.iter())
                .any(|(p, pile)| s.contains(*p) && pile.iter().any(|st| st.object == root))
        })
        .map(|s| shops.get(s.kind).id.as_str())
        .collect();
    assert_eq!(selling, ["town_store"]);
    // Arms and armour at the smithy, the rest at the general store, and nothing the four shops
    // before #1248 sold is missing.
    let sold_at = |id: &str| -> Vec<&str> {
        let object = CONTENT.object_id(id).expect(id);
        (level.shops.iter())
            .filter(|s| {
                (level.objects.iter())
                    .any(|(p, pile)| s.contains(*p) && pile.iter().any(|st| st.object == object))
            })
            .map(|s| shops.get(s.kind).id.as_str())
            .collect()
    };
    for id in [
        "dagger",
        "quarterstaff",
        "sling",
        "short_bow",
        "arrow",
        "leather_jerkin",
        "small_shield",
        "short_sword",
        "long_sword",
        "spear",
        "war_hammer",
        "hand_axe",
        "mace",
        "battle_axe",
        "javelin",
        "bolt",
        "leather_cap",
        "iron_helm",
        "padded_coat",
        "studded_leather",
        "chain_mail",
        "buckler",
        "large_shield",
        "leather_gloves",
        "leather_boots",
        "iron_shod_boots",
    ] {
        assert_eq!(sold_at(id), ["smithy"], "{id}");
    }
    for id in [
        "wool_cloak",
        "travelling_cloak",
        "fur_cloak",
        "potion_of_healing",
        "angelica_root",
        "rope",
        "candle",
        "scroll_of_light",
        "scroll_of_identify",
        "ration",
        "bread",
        "dried_meat",
        "apple",
        "pick_axe",
    ] {
        assert_eq!(sold_at(id), ["town_store"], "{id}");
    }
    // The two kinds are the town's own: no generated level draws them.
    let shared = crate::content::ShopOdds::shared(shops);
    for id in ["town_store", "smithy"] {
        let kind = (0..shops.defs().len())
            .map(|i| crate::content::ShopId(i as u8))
            .find(|&k| shops.get(k).id == id)
            .expect(id);
        assert!(!shared.kinds.contains(&kind), "{id} is generated");
    }
}

/// The houses freed by the shops (#1248): closed doors, furniture, and a townsman at home, no
/// shop.
#[test]
fn the_private_houses_have_doors_furniture_and_folk() {
    let w = town();
    let level = w.state.current();
    let terrain = |x, y| CONTENT.terrain(level.terrain[Pos::new(x, y)]).id.as_str();
    for door in [(53, 9), (65, 9), (70, 9)] {
        assert_eq!(terrain(door.0, door.1), "door_closed", "{door:?}");
    }
    for (x, y, what) in [(49, 4), (51, 4), (63, 4), (68, 8)].map(|(x, y)| (x, y, "bed")) {
        assert_eq!(terrain(x, y), what, "({x}, {y})");
    }
    assert_eq!(terrain(53, 6), "table");
    assert_eq!(terrain(52, 6), "stool");
    for at in [Pos::new(56, 8), Pos::new(70, 7)] {
        assert_eq!(
            CONTENT.creature(w.state.actors[&who(&w, at)].creature).id,
            "villager"
        );
        assert!(
            !level.shops.iter().any(|s| s.contains(at)),
            "{at:?} is no shop"
        );
    }
}

/// The dogs and cats of Cēaptūn and Holtwīc and Holtwīc's hens (#1248) are at peace with the
/// player, and no quest of any campaign, fixed or rolled (with every branch open), ever has
/// them killed.
#[test]
fn the_town_animals_are_never_a_quests_kill() {
    use crate::content::Goal;
    let animals: Vec<_> = ["hound", "cat", "chicken"]
        .map(|id| CONTENT.creature_id(id).expect(id))
        .to_vec();
    let w = town();
    let player = w.state.player;
    let kinds = |w: &World, level: &str| -> Vec<String> {
        let mut kinds: Vec<String> = (w.state.actors.values())
            .filter(|a| a.level == level && animals.contains(&a.creature))
            .map(|a| {
                assert_ne!(w.stance(a.id, player), Stance::Hostile);
                CONTENT.creature(a.creature).id.clone()
            })
            .collect();
        kinds.sort();
        kinds
    };
    assert_eq!(kinds(&w, "ceaptun"), ["cat", "cat", "hound", "hound"]);
    for def in CONTENT.quests() {
        if let Goal::Kill { target, .. } = &def.goal {
            for &a in &animals {
                assert!(!target.has(&CONTENT, a), "{}", def.id);
            }
        }
        for candidate in def.candidates(&CONTENT, &|_| true) {
            let members: Vec<_> = match candidate.target.strip_prefix("tag:") {
                Some(tag) => CONTENT.tagged_in(tag, def.campaign.as_deref()).collect(),
                None => CONTENT.creature_id(&candidate.target).into_iter().collect(),
            };
            for a in &animals {
                assert!(
                    !members.contains(a),
                    "{} draws {}",
                    def.id,
                    candidate.target
                );
            }
        }
    }
}

#[test]
fn everyone_in_town_is_at_peace_with_the_player() {
    let w = town();
    let player = w.state.player;
    let others: Vec<&Actor> = (w.state.actors.values())
        .filter(|a| a.level == "ceaptun" && a.id != player)
        .collect();
    // Eight townsfolk, six people (the smith's apprentice among them), two keepers, four training
    // dummies, two dogs, and two cats.
    assert_eq!(others.len(), 24);
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
fn ceolred_the_apprentice_stands_in_the_smithy_at_peace_and_keeps_no_shop() {
    let mut w = town();
    let id = who(&w, CEOLRED);
    assert_eq!(w.state.actors[&id].npc.as_deref(), Some("ceolred"));
    assert_ne!(w.stance(id, w.state.player), Stance::Hostile);
    assert!((w.state.current().shops.iter()).all(|s| s.keeper != id));
    let smithy = (w.state.current().shops.iter())
        .find(|s| s.name == "Eadric")
        .expect("the smithy");
    let inside = |p: Pos| {
        (smithy.min.x..=smithy.max.x).contains(&p.x) && (smithy.min.y..=smithy.max.y).contains(&p.y)
    };
    assert!(inside(CEOLRED), "inside the smithy");
    put_player(&mut w, CEOLRED.step(Dir::S));
    act(&mut w, "chat:N");
    let line = said(&w).last().unwrap().clone();
    assert!(line.starts_with("Ceolred says: \""), "{line}");
    // Every line is words, never a number.
    let d = CONTENT.dialogue_of_npc("ceolred").expect("a dialogue");
    assert!(
        CONTENT
            .dialogue(d)
            .lines
            .iter()
            .all(|l| !l.contains(char::is_numeric))
    );
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
    // Nobody else in the campaigns offers it but Gimmwīc's gem dealer (#1434).
    let appraisers: Vec<String> = (0..CONTENT.dialogues().len())
        .map(|i| DialogueId(i as u16))
        .filter(|&d| (CONTENT.dialogue(d).services.iter()).any(|s| s.does == ServiceKind::Identify))
        .map(|d| CONTENT.speaker_name(d))
        .collect();
    assert_eq!(appraisers, ["Factor Wulfrun", "Kerensa"]);
}

/// The player's Metalcraft, if trained.
fn metalcraft(world: &World) -> Option<u16> {
    let id = CONTENT.skill_id("metalcraft").expect("Metalcraft");
    world.state().player().skills.value(id)
}

/// The town with the player (untrained in Metalcraft) holding `n` gold coins, west of Eadric.
fn at_the_smithy(n: u32) -> World {
    let mut w = town();
    assert_eq!(metalcraft(&w), None, "Wulfstan never learnt it");
    let coin = CONTENT.object_id("gold_coin").expect("gold");
    let player = w.player_mut();
    player.inventory.retain(|stack| stack.object != coin);
    if n > 0 {
        let mut gold = crate::object::Stack::one(coin);
        gold.count = n;
        player.inventory.push(gold);
    }
    put_player(&mut w, EADRIC.step(Dir::W));
    w
}

#[test]
fn eadric_the_smith_teaches_metalcraft_for_two_thousand_gold() {
    let mut w = at_the_smithy(2100);
    let eadric = who(&w, EADRIC);
    assert_eq!(w.state.actors[&eadric].npc.as_deref(), Some("eadric"));
    let shop = (w.state.current().shops.iter())
        .find(|s| s.keeper == eadric)
        .expect("he keeps the smithy");
    assert_eq!(CONTENT.shops().expect("shops").get(shop.kind).id, "smithy");
    let listed: Vec<(String, Option<u32>)> = (w.topics(eadric).into_iter())
        .map(|t| (t.id, t.price))
        .collect();
    assert_eq!(
        listed,
        [
            ("forge".to_string(), None),
            ("smithing".to_string(), None),
            ("mines".to_string(), None),
            ("train_metalcraft".to_string(), Some(2000)),
        ]
    );
    act(&mut w, "chat:E");
    assert!(
        said(&w)
            .last()
            .unwrap()
            .starts_with("Eadric says: \"Mind the sparks")
    );
    let time = w.time();
    act(&mut w, "ask:E:train_metalcraft");
    assert_eq!((gold(&w), metalcraft(&w)), (100, Some(5)));
    assert!(said(&w).contains(&"You pay Eadric 2000 gold.".to_string()));
    assert!(said(&w).contains(&"Eadric trains you in metalcraft.".to_string()));
    assert!(w.time() > time, "a chat's time");
    // Gone once trained; asked for anyway, his `nothing`, for no time or gold.
    assert!((w.topics(eadric).iter()).all(|t| t.id != "train_metalcraft"));
    let time = w.time();
    act(&mut w, "ask:E:train_metalcraft");
    assert!(
        said(&w)
            .last()
            .unwrap()
            .starts_with("Eadric says: \"You know yer way")
    );
    assert_eq!((gold(&w), w.time(), metalcraft(&w)), (100, time, Some(5)));
}

#[test]
fn the_smith_s_teaching_is_refused_short_of_gold_and_cheaper_with_goodwill() {
    let mut w = at_the_smithy(1999);
    let eadric = who(&w, EADRIC);
    let time = w.time();
    act(&mut w, "ask:E:train_metalcraft");
    assert_eq!(said(&w).last().unwrap(), "You cannot pay.");
    assert_eq!((gold(&w), w.time(), metalcraft(&w)), (1999, time, None));
    // A quest done for the town takes 5% off, as off the town's other services.
    let json = r#"{"records": {"barrow_rats": {"stage": "done", "completed": 1}}}"#;
    w.state.quests = serde_json::from_str(json).expect("quests");
    let price = (w.topics(eadric).into_iter())
        .find(|t| t.id == "train_metalcraft")
        .and_then(|t| t.price);
    assert_eq!(price, Some(1900));
    act(&mut w, "ask:E:train_metalcraft");
    assert_eq!((gold(&w), metalcraft(&w)), (99, Some(5)));
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
