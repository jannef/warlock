//! The craftsperson's patron (#1421; `world/patron.rs`, the craftsperson design §7.4): placed in
//! Cēaptūn as the ability comes at level 12, of the player's race; the pool; the offer and the
//! stock advanced; the log; the hand-in, plain and waiting; the next at once; the death; a save.

use super::patron::{self, Patron};
use super::*;
use crate::character::CharacterSpec;
use crate::magic::worked::Quality;
use crate::test_support::CONTENT;
use crate::test_support::run as act;
use crate::test_support::texts as said;

/// The patron's corner of the Gæsthūs, and the cell west of it.
const CORNER: Pos = Pos::new(50, 23);
const BESIDE: Pos = Pos::new(49, 23);

/// A run in Cēaptūn as `character`, with its kit.
fn run(character: &str) -> World {
    let spec: CharacterSpec = character.parse().expect("a character");
    World::start_run(CONTENT.clone(), "ceaptun", None, Some(&spec), None, true)
        .expect("a run in Cēaptūn")
}

/// What the patron said last.
fn spoke(world: &World) -> String {
    (said(world).into_iter().rev())
        .find(|m| m.starts_with("Your patron says: "))
        .unwrap_or_default()
}

fn the_patron(world: &World) -> Option<&Actor> {
    let content = world.content();
    (world.state.actors.values()).find(|a| {
        content
            .dialogue_of(a.creature, a.npc.as_deref())
            .is_some_and(|d| content.dialogue(d).patron.is_some())
    })
}

fn put_player(world: &mut World, at: Pos) {
    let id = world.state.player;
    world.state.actors.get_mut(&id).expect("player").pos = at;
    world.perceive(&mut Vec::new());
}

fn skill(name: &str) -> crate::skills::SkillId {
    CONTENT.skill_id(name).expect("a skill")
}

/// A craftsperson of level 12 or more beside their patron, Metalcraft at 40 at least.
fn at_the_patron(character: &str) -> World {
    let mut w = run(character);
    act(&mut w, "xp:60000 levelup:auto");
    assert!(w.state.player().talents.has("patron"), "{character}");
    w.player_mut().skills.add(skill("metalcraft"), 40);
    put_player(&mut w, BESIDE);
    w
}

fn carried(w: &World, object: &str) -> u32 {
    let id = CONTENT.object_id(object).expect("an object");
    (w.state.player().inventory.iter())
        .filter(|s| s.object == id)
        .map(|s| s.count)
        .sum()
}

fn gold(w: &World) -> u32 {
    carried(w, "gold_coin")
}

/// The open commission's object id.
fn work(w: &World) -> ObjectId {
    let open = w.state.patron.open.as_ref().expect("a commission open");
    let recipe = CONTENT.recipe(CONTENT.recipes().id(&open.recipe).expect("a recipe"));
    recipe.result.as_ref().expect("a result").object
}

/// Gives the player one of `object`, worked to `quality` (or plain).
fn give(w: &mut World, object: ObjectId, quality: Option<Quality>) {
    let mut stack = Stack::new(object, 1);
    if let Some(worked) = quality.and_then(|q| q.worked(CONTENT.object(object))) {
        stack.set_worked(worked);
    }
    let content = w.content.clone();
    crate::object::add_to_inventory(w.state.inventory_mut(), stack, &content);
}

#[test]
fn the_patron_comes_with_level_12_of_the_players_race_and_to_no_one_else() {
    for (character, npc) in [
        ("human,male,craftsperson", "patron"),
        ("dwarf,female,craftsperson", "patron_dwarf"),
        ("gnome,male,craftsperson", "patron_halfling"),
        ("kobold,female,craftsperson", "patron_goblin"),
        ("lizardfolk,male,craftsperson", "patron_lizardfolk"),
    ] {
        let mut w = run(character);
        assert!(the_patron(&w).is_none(), "{character}: not before level 12");
        act(&mut w, "xp:60000 levelup:auto");
        let p = the_patron(&w).unwrap_or_else(|| panic!("{character}: a patron"));
        assert_eq!(p.npc.as_deref(), Some(npc), "{character}");
        assert_eq!((p.pos, p.level.as_str()), (CORNER, "ceaptun"));
        assert_eq!(w.called(p), "your patron");
        // Never twice: more levels place no second one.
        act(&mut w, "xp:200000 levelup:auto");
        let patrons = (w.state.actors.values())
            .filter(|a| a.npc.as_deref() == Some(npc))
            .count();
        assert_eq!(patrons, 1, "{character}");
    }
    // A merchant of the same level has none.
    let mut w = run("human,male,merchant");
    act(&mut w, "xp:60000 levelup:auto");
    assert!(the_patron(&w).is_none());
}

#[test]
fn the_pool_is_every_graded_one_off_recipe_within_the_players_trades() {
    let w = at_the_patron("human,female,craftsperson");
    let pool: Vec<&str> = (patron::pool(&CONTENT, w.state.player()).into_iter())
        .map(|r| CONTENT.recipe(r).id.as_str())
        .collect();
    assert!(pool.contains(&"long_sword"), "{pool:?}");
    assert!(pool.contains(&"dagger"), "{pool:?}");
    // Counted goods, tools without a grade, built-in recipes, and anything beyond the skill
    // are never asked for.
    for never in [
        "arrowheads",
        "lantern",
        "smelt",
        "temper",
        "cut",
        "set",
        "gold_ring",
    ] {
        assert!(!pool.contains(&never), "{never} in {pool:?}");
    }
    for &id in &pool {
        let recipe = CONTENT.recipe(CONTENT.recipes().id(id).expect("a recipe"));
        let value = w.state.player().skills.value(recipe.skill);
        assert!(value.is_some_and(|v| v >= recipe.from), "{id}");
    }
}

#[test]
fn the_long_sword_advance_is_its_blades_steel() {
    let recipe = CONTENT.recipe(CONTENT.recipes().id("long_sword").expect("a recipe"));
    let stock = patron::advance(&CONTENT, recipe);
    let names: Vec<String> = (stock.iter())
        .map(|s| format!("{} {}", s.count, s.names(&CONTENT).0))
        .collect();
    assert_eq!(names.len(), 1, "{names:?}");
    assert!(names[0].contains("steel"), "{names:?}");
    // No gem: the ornament is the maker's.
    assert!(stock.iter().all(|s| s.gem(&CONTENT).is_none()));
}

#[test]
fn a_commission_is_offered_logged_handed_in_and_paid_then_the_next() {
    let mut w = at_the_patron("human,male,craftsperson");
    let before = w.state.player().inventory.len();
    act(&mut w, "chat:E");
    let line = spoke(&w);
    assert!(line.contains("Fine or better. Do take"), "{line}");
    assert!(line.contains(", sir."), "{line}");
    assert_eq!(w.state.patron.taken, 1);
    let object = work(&w);
    assert!(said(&w).iter().any(|m| m.starts_with("You receive ")));
    assert!(
        w.state.player().inventory.len() > before,
        "the stock advanced"
    );
    // In the log, active, a commission; no quest record of it.
    let view = w.perception();
    let quests = view.quests();
    let logged = quests
        .iter()
        .find(|q| q.id == "commission")
        .expect("logged");
    assert_eq!(logged.kind.id(), "commission");
    assert!(logged.active);
    assert_eq!(logged.giver, "your patron");
    assert_eq!(logged.status, "Not made yet.");
    assert!(
        logged
            .objective
            .starts_with("Your patron wants you to bring them a Fine ")
    );
    assert!(w.state.quests.records.is_empty());
    // Nothing made: waiting. A plain one: plain. Neither takes the work or the commission.
    act(&mut w, "chat:E");
    assert!(
        spoke(&w).contains("The stock I gave you was for"),
        "{}",
        spoke(&w)
    );
    give(&mut w, object, None);
    act(&mut w, "chat:E");
    assert!(
        spoke(&w).contains("and no more. I asked for Fine work"),
        "{}",
        spoke(&w)
    );
    assert!(w.state.patron.open.is_some());
    // A Fine one: taken, paid its worth in full.
    give(&mut w, object, Some(Quality::Fine));
    let mut fine = Stack::new(object, 1);
    fine.set_worked(
        Quality::Fine
            .worked(CONTENT.object(object))
            .expect("graded"),
    );
    let worth = u32::try_from(fine.value(&CONTENT)).expect("worth");
    assert_eq!(
        w.perception().commission().expect("open").status,
        "Return to your patron."
    );
    let coins = gold(&w);
    act(&mut w, "chat:E");
    assert!(
        spoke(&w).contains(&format!("{worth} gold, as agreed")),
        "{}",
        spoke(&w)
    );
    assert_eq!(gold(&w), coins + worth);
    assert!(w.state.patron.open.is_none());
    assert_eq!(
        carried(&w, &CONTENT.object(object).id),
        1,
        "the plain one kept"
    );
    assert!(w.perception().commission().is_none());
    assert!(w.state.quests.records.is_empty(), "counted as no quest");
    // The next at once.
    act(&mut w, "chat:E");
    assert!(spoke(&w).contains("Fine or better"), "{}", spoke(&w));
    assert_eq!(w.state.patron.taken, 2);
}

#[test]
fn nothing_to_commission_is_idle_and_the_patrons_death_drops_the_commission() {
    let mut w = run("human,female,craftsperson");
    act(&mut w, "xp:60000 levelup:auto");
    put_player(&mut w, BESIDE);
    // Every skill forgotten: nothing to ask for.
    w.player_mut().skills = crate::skills::Skills::default();
    act(&mut w, "chat:E");
    assert!(spoke(&w).contains("worth hiring, madam"), "{}", spoke(&w));
    assert!(w.state.patron.is_empty());
    // Trained again: a commission, then the patron dies and it is off.
    w.player_mut().skills.add(skill("metalcraft"), 40);
    act(&mut w, "chat:E");
    assert!(w.state.patron.open.is_some());
    let id = the_patron(&w).expect("a patron").id;
    w.patron_dies(id);
    assert!(w.state.patron.open.is_none());
    assert!(
        said(&w)
            .iter()
            .any(|m| m == "With your patron dead, the commission is off.")
    );
}

#[test]
fn the_commission_is_saved_and_drawn_the_same() {
    let mut w = at_the_patron("dwarf,male,craftsperson");
    act(&mut w, "chat:E");
    let saved = serde_json::to_string(&w.state.patron).expect("saved");
    let back: Patron = serde_json::from_str(&saved).expect("loaded");
    assert_eq!(back, w.state.patron);
    // The same run draws the same work.
    let mut again = at_the_patron("dwarf,male,craftsperson");
    act(&mut again, "chat:E");
    assert_eq!(again.state.patron, w.state.patron);
    assert_eq!(again.state_hash(), w.state_hash());
}
