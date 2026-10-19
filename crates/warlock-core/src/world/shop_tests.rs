//! Shops in play (spec §7 "Shops"): greeting, prices, buying and paying, putting back, selling,
//! theft, an attacked or dead keeper, saves, replays, and generated shops.

use super::*;
use crate::action::{Letters, parse_actions};
use crate::content::{ContentFiles, Shops};
use crate::geom::Dir;
use crate::object::Stack;
use crate::perception::Sale;
use crate::replay::Replay;
use crate::stats::{Stat, Stats};

/// From the courtyard's start: east along the path, the purse in the gazebo (a), the scroll of
/// light beside it (b), then along the flagstones to the hut, opening its door, and in.
const INTO_SHOP: &str = "move:E*36 move:S pickup move:NE move:NE pickup:b move:SE move:E*5 \
                         move:S*5 move:E*5";
/// The shop's door, the cell inside it, and where Grimbold stands.
const DOOR: Pos = Pos::new(53, 14);
const FRONT: Pos = Pos::new(54, 14);
const POST: Pos = Pos::new(54, 13);
/// Goods on the shop floor.
const DAGGER: Pos = Pos::new(56, 13);
const SWORD: Pos = Pos::new(58, 13);

fn content() -> Arc<Content> {
    Arc::new(Content::builtin().expect("built-in content"))
}

fn courtyard() -> World {
    World::new(content(), "courtyard", None).expect("courtyard")
}

/// The courtyard with the player in Grimbold's General Store, carrying 60 gold (a) and a scroll
/// of light (b).
fn in_shop() -> World {
    let mut w = courtyard();
    apply(&mut w, INTO_SHOP);
    assert_eq!(w.player_pos(), FRONT, "in the shop");
    w
}

fn apply(w: &mut World, actions: &str) -> Outcome {
    let mut last = Outcome::default();
    for action in parse_actions(actions).expect("actions") {
        last = w.apply(&action);
    }
    last
}

fn last(w: &World) -> String {
    (w.messages().iter().last()).map_or(String::new(), |m| m.text.clone())
}

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.text.contains(text))
}

fn shops(w: &World) -> &Shops {
    w.content().shops().expect("shops")
}

fn shop(w: &World) -> &Shop {
    &w.state.current().shops[0]
}

fn gold(w: &World, id: ActorId) -> u32 {
    count_of(&w.state.actors[&id].inventory, shops(w).currency)
}

fn keeper(w: &World) -> ActorId {
    shop(w).keeper
}

/// What the store asks the player for `stack` now.
fn price(w: &World, stack: &Stack) -> u32 {
    let f = favour(w.content(), shops(w), w.state.player());
    shops(w).price(w.content(), shop(w).kind, stack, f)
}

fn object(w: &World, id: &str) -> Stack {
    Stack::one(w.content().object_id(id).expect(id))
}

#[test]
fn the_courtyard_has_a_general_store_kept_by_grimbold() {
    let w = courtyard();
    let shop = shop(&w);
    assert_eq!(shop.sign(w.content()), "Grimbold's General Store");
    assert_eq!((shop.post, shop.status), (POST, ShopStatus::Open));
    let keeper = &w.state.actors[&shop.keeper];
    assert_eq!(keeper.pos, POST);
    assert_eq!(w.content().creature(keeper.creature).id, "shopkeeper");
    // A merchant fights no one: the showcase stays safe.
    assert_eq!(w.stance(w.player_id(), shop.keeper), Stance::Neutral);
    assert!(gold(&w, shop.keeper) > 0, "a purse to buy with");
}

#[test]
fn the_keeper_keeps_its_post_and_talks_trade_by_its_name() {
    let mut w = in_shop();
    // Next to the door the keeper does not make way (as an NPC who keeps its post).
    let out = apply(&mut w, "move:N");
    assert!(!out.took_turn);
    assert_eq!(w.player_pos(), FRONT);
    assert_eq!(last(&w), "Grimbold doesn't budge.");
    apply(&mut w, "chat:N");
    assert!(
        last(&w).starts_with("Grimbold says: \"Look around, take your time."),
        "{}",
        last(&w)
    );
}

#[test]
fn entering_greets_and_goods_show_their_prices() {
    let mut w = in_shop();
    assert!(said(&w, "Welcome to Grimbold's General Store!"));
    let dagger = object(&w, "dagger");
    let asked = price(&w, &dagger);
    assert!(asked > 0);
    let view = w.perception();
    let here = view.shop_here().expect("in the shop");
    assert_eq!(
        (here.sign(), here.keeper()),
        ("Grimbold's General Store".into(), "Grimbold")
    );
    assert_eq!(view.price_tag(DAGGER, &dagger), Some(asked));
    assert_eq!(
        view.price_tag(DOOR, &dagger),
        None,
        "outside the shop nothing is for sale"
    );
    apply(&mut w, "move:E move:NE");
    assert_eq!(
        last(&w),
        format!("You see here a dagger (for sale, {asked} gold).")
    );
}

#[test]
fn picking_up_adds_to_the_debt_and_paying_settles_it_at_no_time() {
    let mut w = in_shop();
    let (player, keeper) = (w.player_id(), keeper(&w));
    let (purse, till) = (gold(&w, player), gold(&w, keeper));
    let asked = price(&w, &object(&w, "dagger"));
    apply(&mut w, "move:E move:NE pickup");
    assert_eq!(
        last(&w),
        format!("You pick up a dagger (unpaid, {asked} gold).")
    );
    assert_eq!(shop(&w).unpaid, [object(&w, "dagger")]);
    let item = w.perception().inventory().len() - 1;
    assert_eq!(w.perception().unpaid()[item], Some(asked));
    assert_eq!(w.perception().shop_here().map(|s| s.owed()), Some(asked));
    // Dumps say it too.
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Inventory);
    let unpaid = format!("a dagger (unpaid, {asked} gold)");
    assert!(rows.iter().any(|r| r.contains(&unpaid)), "{rows:?}");
    let json = crate::dump::json(&w, &[], false);
    assert_eq!(json["shop"]["keeper"], "Grimbold");
    assert_eq!(json["shop"]["owed"], asked);
    assert_eq!(json["inventory"][item]["unpaid"], asked);
    let time = w.time();
    let paid = apply(&mut w, "pay");
    assert_eq!(
        (paid.took_turn, w.time()),
        (false, time),
        "paying takes no time"
    );
    assert_eq!(last(&w), format!("You pay Grimbold {asked} gold."));
    assert_eq!(gold(&w, player), purse - asked);
    assert_eq!(gold(&w, keeper), till + asked);
    assert!(shop(&w).unpaid.is_empty());
    assert_eq!(w.perception().unpaid()[item], None);
    apply(&mut w, "pay");
    assert_eq!(last(&w), "You owe Grimbold nothing.");
    // Paid for, the dagger leaves with the player: no theft.
    apply(&mut w, "move:SW move:W move:W");
    assert_eq!(w.player_pos(), DOOR);
    assert_eq!(shop(&w).status, ShopStatus::Open);
    assert_eq!(w.stance(player, keeper), Stance::Neutral);
}

#[test]
fn without_enough_gold_the_player_puts_it_back() {
    let mut w = in_shop();
    let asked = price(&w, &object(&w, "short_sword"));
    assert!(asked > 60, "the sword costs more than the purse holds");
    apply(&mut w, "move:E move:NE move:E move:E pickup pay");
    assert_eq!(
        last(&w),
        format!("You owe Grimbold {asked} gold, but you have only 60.")
    );
    assert_eq!(gold(&w, w.player_id()), 60, "nothing was paid");
    let sword = (w.perception().inventory().iter())
        .position(|s| s.object == object(&w, "short_sword").object)
        .expect("carried");
    let put = apply(
        &mut w,
        &format!("drop:{}", letter(sword).expect("a letter")),
    );
    assert!(put.took_turn, "putting back is dropping");
    assert_eq!(last(&w), "You put back a short sword.");
    assert!(shop(&w).unpaid.is_empty());
    assert_eq!(w.perception().objects(SWORD), [object(&w, "short_sword")]);
    assert_eq!(
        w.perception().price_tag(SWORD, &object(&w, "short_sword")),
        Some(asked)
    );
}

#[test]
fn dropping_ones_own_gets_an_offer_and_selling_takes_it() {
    let mut w = in_shop();
    let (player, keeper) = (w.player_id(), keeper(&w));
    let (purse, till) = (gold(&w, player), gold(&w, keeper));
    let scroll = object(&w, "scroll_of_light");
    let f = favour(w.content(), shops(&w), w.state.player());
    let offer = shops(&w).offer(w.content(), shop(&w).kind, &scroll, f);
    assert!(
        offer > 0 && offer < price(&w, &scroll),
        "it pays less than it asks"
    );
    assert_eq!(
        w.perception().sale(Letters::of([1])),
        Some(Sale {
            offer,
            stacks: vec![scroll.clone()]
        })
    );
    // A drop only hears the offer: the scroll stays, and no time passes.
    let dropped = apply(&mut w, "drop:b");
    assert!(!dropped.took_turn);
    assert_eq!(
        last(&w),
        format!("Grimbold offers {offer} gold for a scroll of light.")
    );
    assert_eq!(w.perception().inventory().len(), 2);
    // Nobody buys the currency.
    apply(&mut w, "drop:a");
    assert_eq!(last(&w), "Grimbold has no use for 60 gold coins.");
    assert_eq!(w.perception().sale(Letters::of([0])), None);
    let sold = apply(&mut w, "sell:b");
    assert!(sold.took_turn, "selling takes a drop's time");
    assert_eq!(
        last(&w),
        format!("You sell a scroll of light to Grimbold for {offer} gold.")
    );
    assert_eq!(gold(&w, player), purse + offer);
    assert_eq!(gold(&w, keeper), till - offer);
    assert_eq!(w.perception().objects(FRONT), std::slice::from_ref(&scroll));
    // Now it is the shop's, for sale like the rest.
    assert_eq!(
        w.perception().price_tag(FRONT, &scroll),
        Some(price(&w, &scroll))
    );
    let coins = gold(&w, player);
    apply(&mut w, "sell");
    assert_eq!(
        last(&w),
        format!("Grimbold has no use for {coins} gold coins.")
    );
    // Outside a shop there is nobody to sell to.
    apply(&mut w, "move:W");
    apply(&mut w, "sell:a");
    assert_eq!(last(&w), "There is nobody here to sell to.");
}

#[test]
fn walking_out_of_the_door_owing_is_turned_back_one_step() {
    let mut w = in_shop();
    let (player, keeper) = (w.player_id(), keeper(&w));
    assert_eq!(shop(&w).exits, [DOOR], "the hut's one way out");
    apply(&mut w, "move:E move:NE pickup move:SW move:W");
    assert_eq!(w.player_pos(), FRONT);
    let owed = price(&w, &object(&w, "dagger"));
    let view = w.perception();
    let (seen, n) = view.owed_on_leaving(DOOR).expect("leaving owes");
    assert_eq!((seen.is_exit(DOOR), n), (true, owed));
    assert!(
        w.perception().owed_on_leaving(Pos::new(55, 14)).is_none(),
        "still inside"
    );
    // Grimbold steps into the way: a step back, the other way, in a step's time.
    let time = w.time();
    let out = apply(&mut w, "move:W");
    assert!(out.took_turn);
    assert!(w.time() > time);
    assert_eq!(w.player_pos(), Pos::new(55, 14));
    assert_eq!(last(&w), "Grimbold steps into your way: you haven't paid.");
    assert_eq!(shop(&w).status, ShopStatus::Open, "no theft");
    assert_eq!(w.stance(player, keeper), Stance::Neutral);
    assert_eq!(shop(&w).unpaid, [object(&w, "dagger")], "still owed");
    // Diagonally out of the door from the south corner: the way back is the wall, so onto the
    // nearest free cell (the first in row order), beside the door.
    apply(&mut w, "move:SW");
    assert_eq!(w.player_pos(), Pos::new(54, 15));
    apply(&mut w, "move:NW");
    assert_eq!(w.player_pos(), FRONT);
    // Paying and leaving is one action: paid, the step goes out.
    let gold_before = gold(&w, player);
    let out = apply(&mut w, "pay:W");
    assert!(out.took_turn, "the step's time");
    assert_eq!(w.player_pos(), DOOR);
    assert_eq!(gold(&w, player), gold_before - owed);
    assert!(said(&w, &format!("You pay Grimbold {owed} gold.")));
    assert_eq!(shop(&w).status, ShopStatus::Open);
    assert_eq!(w.stance(player, keeper), Stance::Neutral);
}

#[test]
fn without_the_gold_paying_and_leaving_goes_nowhere() {
    let mut w = in_shop();
    apply(
        &mut w,
        "move:E move:NE move:E move:E pickup move:W*3 move:SW",
    );
    assert_eq!(w.player_pos(), FRONT);
    let out = apply(&mut w, "pay:W");
    assert!(!out.took_turn);
    assert_eq!(w.player_pos(), FRONT, "no step without paying");
    assert!(last(&w).starts_with("You owe Grimbold"), "{}", last(&w));
}

#[test]
fn turned_back_with_nowhere_to_go_the_player_stays() {
    // A shop of two cells, the keeper on one: the player, on the other, has nowhere to go.
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/booth.toml",
        "name = \"Booth\"\nambient = [255, 255, 255, 0]\nmap = '''\n####\n#@.#\n#+##\n#..#\n####\n'''\n\
         [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\", objects = [\"dagger\"] }\n\
         [[shop]]\nkind = \"weaponsmith\"\nkeeper = \"Orvald\"\narea = [1, 1, 2, 1]\npost = [2, 1]\n",
    );
    files.insert(
        "scenarios/booth.toml",
        "name = \"Booth\"\nstart_level = \"booth\"\nhp = 100\n",
    );
    let content = Arc::new(Content::load(&files).expect("valid"));
    let mut w = World::new(content, "booth", None).expect("booth");
    let here = Pos::new(1, 1);
    assert_eq!(shop(&w).exits, [Pos::new(1, 2)], "the door, shut");
    apply(&mut w, "pickup move:S");
    assert!(!shop(&w).unpaid.is_empty());
    let time = w.time();
    let out = apply(&mut w, "move:S");
    assert!(out.took_turn, "a step's time, spent at the door");
    assert!(w.time() > time);
    assert_eq!(w.player_pos(), here);
    assert_eq!(last(&w), "Orvald steps into your way: you haven't paid.");
    assert_eq!(shop(&w).status, ShopStatus::Open);
}

#[test]
fn leaving_any_other_way_owing_is_theft_and_the_keeper_comes_after_the_thief() {
    let mut w = in_shop();
    let (player, keeper) = (w.player_id(), keeper(&w));
    apply(&mut w, "move:E move:NE pickup move:SW move:E*7");
    assert_eq!(w.player_pos(), Pos::new(62, 14));
    // The hut's east wall gives way (as a wall dug open would): not a way out of the shop.
    let hole = Pos::new(63, 14);
    let floor = w.content().terrain_id("plank_floor").expect("plank floor");
    w.set_terrain(hole, floor);
    let (seen, _) = w.perception().owed_on_leaving(hole).expect("leaving owes");
    assert!(!seen.is_exit(hole));
    apply(&mut w, "move:E");
    assert_eq!(w.player_pos(), hole);
    assert!(said(
        &w,
        "You leave without paying. \"Stop, thief!\" shouts Grimbold."
    ));
    assert_eq!(shop(&w).status, ShopStatus::Outraged);
    assert!(shop(&w).unpaid.is_empty(), "the dagger is stolen now");
    assert_eq!(w.stance(player, keeper), Stance::Hostile);
    // An outraged keeper trades no more: no greeting, no prices.
    assert!(w.perception().shop_at(SWORD).is_none());
    // It comes after the thief.
    apply(&mut w, "wait");
    assert_ne!(w.state.actors[&keeper].pos, POST, "it went for the thief");
}

#[test]
fn unpaid_goods_are_not_thrown_and_an_attacked_keeper_is_outraged() {
    let mut w = in_shop();
    apply(&mut w, "move:E move:NE pickup");
    let dagger = w.perception().inventory().len() - 1;
    let c = letter(dagger).expect("a letter");
    let thrown = apply(&mut w, &format!("throw:{c}:58,14"));
    assert!(!thrown.took_turn);
    assert_eq!(last(&w), format!("You have not paid for a dagger ({c})."));
    // Paid for, it flies; at the keeper, it outrages him, hit or miss.
    apply(&mut w, "pay");
    apply(&mut w, &format!("throw:{c}:54,13"));
    assert!(said(&w, "\"You will pay for that!\" shouts Grimbold."));
    assert_eq!(shop(&w).status, ShopStatus::Outraged);
    assert_eq!(w.stance(w.player_id(), keeper(&w)), Stance::Hostile);
}

#[test]
fn a_dead_keepers_goods_are_free_to_take() {
    let mut w = in_shop();
    let keeper = keeper(&w);
    apply(&mut w, "move:E move:NE pickup pay");
    apply(&mut w, "throw:c:54,13");
    assert_eq!(shop(&w).status, ShopStatus::Outraged);
    // A frail keeper and a sturdy player, so the fight ends quickly the right way.
    w.state.actors.get_mut(&keeper).expect("alive").hp = 1;
    let player = w.player_id();
    let me = w.state.actors.get_mut(&player).expect("the player");
    me.hp_base = 10_000;
    me.hp = 10_000;
    for _ in 0..200 {
        let Some(k) = w.state.actors.get(&keeper).map(|a| a.pos) else {
            break;
        };
        let dir = Dir::toward(w.player_pos(), k).expect("apart");
        w.apply(&Action::Move(dir));
    }
    assert!(!w.state.actors.contains_key(&keeper), "Grimbold is dead");
    assert_eq!(shop(&w).status, ShopStatus::Abandoned);
    assert!(
        w.perception().shop_at(SWORD).is_none(),
        "nothing is for sale"
    );
    // What lies there is free: no debt, no theft.
    let sword = object(&w, "short_sword");
    assert_eq!(w.perception().price_tag(SWORD, &sword), None);
}

#[test]
fn prices_follow_presence_and_appraise() {
    let mut w = in_shop();
    let dagger = object(&w, "dagger");
    let player = w.player_id();
    let set_presence = |w: &mut World, score: u8| {
        let me = w.state.actors.get_mut(&player).expect("the player");
        let mut scores = me.stats.scores();
        scores[Stat::Presence] = score;
        me.stats = Stats::new(scores, scores);
    };
    set_presence(&mut w, 10);
    let plain = price(&w, &dagger);
    set_presence(&mut w, 18);
    let charming = price(&w, &dagger);
    set_presence(&mut w, 4);
    let off_putting = price(&w, &dagger);
    assert!(
        charming < plain && plain < off_putting,
        "{charming} {plain} {off_putting}"
    );
    set_presence(&mut w, 10);
    let presence_free = price(&w, &dagger);
    let appraise = w.content().skill_id("appraise").expect("appraise");
    let me = w.state.actors.get_mut(&player).expect("the player");
    me.skills = crate::skills::Skills::default();
    me.skills.add(appraise, 100);
    let appraising = price(&w, &dagger);
    assert!(appraising < presence_free, "{appraising} {presence_free}");
    // Perception asks the same.
    assert_eq!(w.perception().price_tag(DAGGER, &dagger), Some(appraising));
}

#[test]
fn shops_save_replay_and_hash() {
    // Into the shop for the dagger, a scroll sold; out of the door unpaid, turned back; then
    // paid and out in one action.
    let actions =
        format!("{INTO_SHOP} move:E move:NE pickup move:SW sell:b move:W move:W move:W pay:W");
    let mut w = courtyard();
    let mut replay = Replay::new(&w);
    for action in parse_actions(&actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(w.player_pos(), DOOR, "paid and out");
    assert!(shop(&w).unpaid.is_empty());
    assert!(said(&w, "Grimbold steps into your way: you haven't paid."));
    let (again, divergence) = replay.play(content()).expect("plays");
    assert!(divergence.is_none(), "the replay verifies");
    assert_eq!(again.state_hash(), w.state_hash());
    let bytes = crate::save::encode_state(w.state(), w.content().fingerprint());
    let loaded = crate::save::decode(&bytes, content()).expect("loads");
    assert_eq!(loaded.state_hash(), w.state_hash());
    assert_eq!(loaded.state().current().shops, w.state().current().shops);
    // Being turned back lands the same every time.
    let mut a = courtyard();
    let mut b = courtyard();
    let theft = format!("{INTO_SHOP} move:E move:NE pickup move:SW move:W move:W wait*3");
    apply(&mut a, &theft);
    apply(&mut b, &theft);
    assert_eq!(a.state_hash(), b.state_hash());
}

#[test]
fn level_files_mark_shops_with_known_kinds_and_posts() {
    let level = |shop: &str| {
        let mut files = ContentFiles::builtin();
        files.insert(
            "levels/test_shop.toml",
            &format!(
                "name = \"T\"\nmap = '''\n#######\n#@....#\n#.....#\n#######\n'''\n\
                 [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n{shop}"
            ),
        );
        Content::load(&files)
    };
    let shop = |kind: &str, area: &str, post: &str| {
        format!("[[shop]]\nkind = \"{kind}\"\nkeeper = \"Hilda\"\narea = {area}\npost = {post}\n")
    };
    let ok = level(&shop("armourer", "[2, 1, 5, 2]", "[5, 1]")).expect("valid");
    let template = ok.level("test_shop").expect("the level");
    assert_eq!(template.shops.len(), 1);
    assert!(template.creatures.iter().any(|c| c.pos == Pos::new(5, 1)));
    for (text, why) in [
        (
            shop("tailor", "[2, 1, 5, 2]", "[5, 1]"),
            "unknown kind `tailor`",
        ),
        (shop("armourer", "[2, 1, 9, 2]", "[5, 1]"), "inside the map"),
        (shop("armourer", "[2, 1, 5, 2]", "[1, 2]"), "`post`"),
        (shop("armourer", "[1, 1, 5, 2]", "[1, 1]"), "`post`"),
        (
            format!(
                "{}{}",
                shop("armourer", "[2, 1, 3, 2]", "[3, 1]"),
                shop("fletcher", "[3, 1, 5, 2]", "[5, 1]")
            ),
            "overlaps",
        ),
    ] {
        let err = level(&text).expect_err(&text);
        assert!(err.message.contains(why), "{why}: {}", err.message);
    }
    let mut files = ContentFiles::builtin();
    files.remove("data/shops.toml");
    let err = Content::load(&files).expect_err("the courtyard has a shop");
    assert!(err.message.contains("data/shops.toml"), "{}", err.message);
}

#[test]
fn generated_shops_are_kept_by_their_keepers() {
    // The Depths of seed 2 have a provisioner's on their first level.
    let w = World::new(content(), "depths", Some(2)).expect("depths");
    let shop = shop(&w);
    let def = shops(&w).get(shop.kind);
    assert_eq!(def.id, "provisioner");
    let keeper = &w.state.actors[&shop.keeper];
    assert_eq!((keeper.pos, keeper.level.as_str()), (shop.post, "depths:1"));
    assert_eq!(w.content().creature(keeper.creature).id, "shopkeeper");
    assert!(!keeper.equipment.is_empty(), "armed and armoured");
    assert_eq!(shop.exits.len(), 1, "its one door: {:?}", shop.exits);
    let goods = (w.state.current().objects.iter()).filter(|(p, _)| shop.contains(**p));
    assert!(goods.count() >= usize::from(def.items.0));
}
