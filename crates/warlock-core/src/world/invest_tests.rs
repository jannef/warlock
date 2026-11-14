//! The merchant's investment (#1407; `world/invest.rs`): the class ability from level 1, the
//! partner in Cēaptūn only in a merchant's run (by race), the fees, the weekly return and its
//! catching up over long time, the quests' bonus, the partner's death, the expected yearly
//! return, a save, and a replay.

use super::invest::{self, Fund, WEEK};
use super::*;
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::replay::Replay;
use crate::test_support::CONTENT;
use crate::test_support::run as act;
use crate::test_support::texts;

/// The partner's desk in Wulfrun's office, and the cell west of it.
const DESK: Pos = Pos::new(75, 23);
const BESIDE: Pos = Pos::new(74, 23);

/// A run in Cēaptūn as `character`, with its kit.
fn run(character: &str) -> World {
    let spec: CharacterSpec = character.parse().expect("a character");
    World::start_run(CONTENT.clone(), "ceaptun", None, Some(&spec), None, true)
        .expect("a run in Cēaptūn")
}

fn last(world: &World) -> String {
    texts(world).last().cloned().unwrap_or_default()
}

/// What the partner said last.
fn spoke(world: &World) -> String {
    (texts(world).into_iter().rev())
        .find(|m| m.starts_with("Your business partner says: "))
        .unwrap_or_default()
}

fn partner(world: &World) -> Option<&Actor> {
    let content = world.content();
    (world.state.actors.values()).find(|a| {
        content
            .dialogue_of(a.creature, a.npc.as_deref())
            .is_some_and(|d| content.dialogue(d).partner.is_some())
    })
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

/// Gives the player `n` more gold coins.
fn give_gold(world: &mut World, n: u32) {
    let coin = world.content().object_id("gold_coin").expect("gold");
    let content = world.content.clone();
    crate::object::add_to_inventory(world.state.inventory_mut(), Stack::new(coin, n), &content);
}

/// A merchant beside their partner with 2000 gold more.
fn at_the_desk() -> World {
    let mut w = run("human,female,merchant");
    give_gold(&mut w, 2000);
    put_player(&mut w, BESIDE);
    w
}

fn topic_ids(world: &World) -> Vec<String> {
    let id = partner(world).expect("a partner").id;
    world.topics(id).into_iter().map(|t| t.id).collect()
}

/// Sets the quest records: each quest done once.
fn done(world: &mut World, quests: &[&str]) {
    let records: Vec<String> = (quests.iter())
        .map(|q| format!(r#""{q}": {{"stage": "done", "completed": 1}}"#))
        .collect();
    let json = format!(r#"{{"records": {{{}}}}}"#, records.join(", "));
    world.state.quests = serde_json::from_str(&json).expect("quest records");
}

#[test]
fn a_merchant_has_invest_and_a_partner_of_their_race_and_no_one_else() {
    for (character, npc) in [
        ("human,male,merchant", "partner"),
        ("half_orc,other,merchant", "partner"),
        ("dwarf,female,merchant", "partner_dwarf"),
        ("gnome,male,merchant", "partner_halfling"),
        ("kobold,female,merchant", "partner_goblin"),
        ("drow,male,merchant", "partner_drow"),
    ] {
        let w = run(character);
        assert!(w.state.player().talents.has("invest"), "{character}");
        let p = partner(&w).unwrap_or_else(|| panic!("{character}: a partner"));
        assert_eq!(p.npc.as_deref(), Some(npc), "{character}");
        assert_eq!(p.pos, DESK);
        assert_eq!(w.called(p), "your business partner");
    }
    // An aristocrat has their servant and no partner; a warrior neither.
    let w = run("human,male,aristocrat");
    assert!(partner(&w).is_none());
    assert!(!w.state.player().talents.has("invest"));
    let w = run("human,male,warrior");
    assert!(partner(&w).is_none());
}

#[test]
fn chatting_tells_the_worth_and_lists_the_deals() {
    let mut w = at_the_desk();
    act(&mut w, "chat:E");
    let line = spoke(&w);
    assert!(
        line.starts_with("Your business partner says: \"") && line.contains("Idle gold"),
        "{line}"
    );
    let carried = gold(&w);
    assert_eq!(
        topic_ids(&w),
        ["invest_100", "invest_1000", "invest_all"],
        "nothing to withdraw yet"
    );
    act(&mut w, "ask:E:invest_1000");
    assert_eq!(gold(&w), carried - 1000);
    // 2 % kept: 980 working.
    assert_eq!(w.state.fund.cents, 98_000);
    assert!(texts(&w).iter().any(|m| m == "You hand over 1000 gold."));
    assert!(
        spoke(&w).contains("1000 gold, less 20 for the paperwork"),
        "{}",
        spoke(&w)
    );
    act(&mut w, "chat:E");
    assert!(
        spoke(&w).contains("Your stake stands at 980 gold"),
        "{}",
        spoke(&w)
    );
    assert_eq!(
        topic_ids(&w),
        [
            "invest_100",
            "invest_1000",
            "invest_all",
            "withdraw_100",
            "withdraw_all"
        ]
    );
    // Out again: 2 % of 980 is 19.6, rounded up to 20.
    let before = gold(&w);
    act(&mut w, "ask:E:withdraw_all");
    assert_eq!(gold(&w), before + 960);
    assert_eq!(w.state.fund.cents, 0);
    assert!(spoke(&w).contains("980 gold out, less 20"), "{}", spoke(&w));
    assert!(texts(&w).iter().any(|m| m == "You receive 960 gold coins."));
}

#[test]
fn more_than_is_carried_or_held_is_refused_with_no_time() {
    let mut w = at_the_desk();
    let carried = gold(&w);
    let turn = w.state.turn;
    act(&mut w, &format!("ask:E:invest_{}", carried + 1));
    assert_eq!(last(&w), "You do not carry that much gold.");
    act(&mut w, "ask:E:withdraw_1");
    assert_eq!(last(&w), "Your stake is not worth that much.");
    assert_eq!((w.state.turn, gold(&w)), (turn, carried));
    // Any amount by its id.
    act(&mut w, "ask:E:invest_250");
    assert_eq!(gold(&w), carried - 250);
    assert_eq!(w.state.fund.cents, 24_500);
}

#[test]
fn every_week_takes_its_return_and_a_long_rest_catches_up() {
    let rules = CONTENT.invest().expect("rules").clone();
    let mut w = at_the_desk();
    act(&mut w, "ask:E:invest_1000");
    let start = w.state.fund.clone();
    let week = start.week;
    assert_eq!(week, w.state.time.seconds() / WEEK);
    // Within the week: nothing yet.
    let into_week = w.state.time.seconds() % WEEK;
    let to_next = WEEK - into_week;
    let before_rest = w.clone();
    act(&mut w, &format!("skip:{}s", to_next - 1));
    assert_eq!(w.state.fund, start);
    // Ten weeks at once: each week's return, from its own stream, in turn.
    let mut far = w.clone();
    act(&mut far, &format!("skip:{}s", 1 + 10 * WEEK - WEEK));
    let mut cents = start.cents;
    for k in 1..=10 {
        cents = invest::grown(
            cents,
            invest::week_return(&rules, w.state.seed, week + k, 0),
        );
    }
    assert_eq!(far.state.fund.week, week + 10);
    assert_eq!(far.state.fund.cents, cents);
    assert_ne!(cents, start.cents);
    // A week at a time comes to the same.
    for _ in 0..10 {
        act(&mut w, &format!("skip:{WEEK}s"));
    }
    assert_eq!(far.state.fund.week, w.state.fund.week);
    assert_eq!(far.state.fund.cents, w.state.fund.cents);
    // Resting moves time too: a rest across the week's turn takes its return the same way.
    let mut rested = before_rest;
    act(&mut rested, &format!("skip:{}s", to_next - 3600));
    assert_eq!(rested.state.fund, start);
    act(&mut rested, "rest:2h");
    assert_eq!(rested.state.fund.week, week + 1);
    let first = invest::grown(
        start.cents,
        invest::week_return(&rules, w.state.seed, week + 1, 0),
    );
    assert_eq!(rested.state.fund.cents, first);
}

#[test]
fn quests_of_the_town_and_fire_in_the_deep_raise_the_return() {
    let rules = CONTENT.invest().expect("rules").clone();
    let mut w = at_the_desk();
    assert_eq!(w.invest_bonus(), 0);
    // Two of Cēaptūn's people's quests, one from Holtwīc's wise-woman: only the town's count.
    done(
        &mut w,
        &["grave_goods", "barrow_rats", "taken_under_the_hill"],
    );
    assert_eq!(w.invest_bonus(), 2 * rules.per_quest);
    // Fire in the deep is the town's (Wulfrun's) and adds its own on top.
    done(&mut w, &["grave_goods", "barrow_rats", "fire_deep"]);
    assert_eq!(w.invest_bonus(), 3 * rules.per_quest + 6);
    // The bonus lifts every week's roll.
    act(&mut w, "ask:E:invest_1000");
    let start = w.state.fund.clone();
    let mut plain = w.clone();
    plain.state.quests = Default::default();
    act(&mut w, &format!("skip:{}s", 20 * WEEK));
    act(&mut plain, &format!("skip:{}s", 20 * WEEK));
    assert!(w.state.fund.cents > plain.state.fund.cents);
    let bonus = w.invest_bonus();
    let mut cents = start.cents;
    for k in 1..=20 {
        cents = invest::grown(
            cents,
            invest::week_return(&rules, w.state.seed, start.week + k, bonus),
        );
    }
    assert_eq!(w.state.fund.cents, cents);
}

#[test]
fn the_stake_dies_with_the_partner() {
    let mut w = at_the_desk();
    act(&mut w, "ask:E:invest_1000");
    let id = partner(&w).expect("a partner").id;
    let when = w.state.time;
    w.dies(id, None, None, when, &mut Vec::new());
    assert!(partner(&w).is_none());
    assert_eq!(w.state.fund.cents, 0);
    assert!(texts(&w).iter().any(|m| m.contains("your stake is lost")));
}

/// The expected yearly return of `data/invest.toml` with `bonus` basis points, over many
/// years of 360 days: the mean growth and the share of years that lose.
fn yearly(bonus: i32, years: u64) -> (f64, f64) {
    let rules = CONTENT.invest().expect("rules");
    let weeks = 360 / 7;
    let (mut sum, mut lost) = (0.0, 0u32);
    for seed in 0..years {
        let mut cents = 100_000_000u64;
        for week in 0..weeks {
            cents = invest::grown(cents, invest::week_return(rules, seed, week, bonus));
        }
        let growth = cents as f64 / 100_000_000.0 - 1.0;
        sum += growth;
        lost += u32::from(growth < 0.0);
    }
    (sum / years as f64, f64::from(lost) / years as f64)
}

#[test]
fn a_year_returns_about_five_percent_and_one_in_four_loses() {
    let (mean, lost) = yearly(0, 4000);
    println!(
        "invest: {:.1} % a year, {:.0} % of years lose",
        mean * 100.0,
        lost * 100.0
    );
    assert!((0.035..=0.07).contains(&mean), "{mean}");
    assert!((0.15..=0.40).contains(&lost), "{lost}");
    // Everything done: every quest of Cēaptūn's people (14) and Fire in the deep (6 more).
    let (mean, lost) = yearly(20, 4000);
    println!(
        "invest, all done: {:.1} % a year, {:.0} % of years lose",
        mean * 100.0,
        lost * 100.0
    );
    assert!((0.13..=0.19).contains(&mean), "{mean}");
    assert!(lost < 0.15, "{lost}");
}

#[test]
fn the_stake_survives_a_save_and_replays() {
    let mut w = run("human,male,merchant");
    let mut replay = Replay::new(&w);
    // From the square to the partner's desk in Wulfrun's office, a stake, and a month.
    let steps = "move:SE*4 move:E*22 open:S move:S*2 move:E*2 chat:E ask:E:invest_100 \
                 skip:24h*30";
    for action in parse_actions(steps).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(w.player_pos(), BESIDE);
    assert!(w.state.fund.cents > 0 && w.state.fund.cents != 9_800);
    let json = serde_json::to_string(&w.state).expect("saves");
    let state: WorldState = serde_json::from_str(&json).expect("loads");
    let loaded = World::from_state(w.content.clone(), state).expect("restores");
    assert_eq!(loaded.state.fund, w.state.fund);
    assert_eq!(loaded.state_hash(), w.state_hash());
    for action in parse_actions("ask:E:withdraw_all").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(w.state.fund.cents < 100, "only the hundredths stay");
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(w.content.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}

#[test]
fn a_run_that_never_invests_saves_no_stake() {
    let w = run("human,male,merchant");
    assert!(w.state.fund.is_empty());
    assert_eq!(w.state.fund, Fund::default());
    let json = serde_json::to_string(&w.state).expect("saves");
    assert!(!json.contains("\"fund\""));
}
