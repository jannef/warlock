//! Stealing in play (spec §8 "Stealing", #234): a stack lifted unnoticed (never what is worn or
//! wielded), empty pockets, a caught thief turned on, a robbed shopkeeper, the refusals, the
//! draws it makes, training, the abilities menu, saves, and replays.

use super::*;
use crate::ability::Ability;
use crate::action::parse_actions;
use crate::combat;
use crate::content::ContentFiles;
use crate::geom::Dir;
use crate::replay::Replay;
use crate::stats::{Stat, Stats};

// A lit den: the player between a gardener (west), who carries nothing, and a peaceful kobold
// (east) with a short sword in hand and coins and apples in its pack.
const DEN: &str = r##"
name = "Den"
ambient = [255, 255, 255, 0]
map = '''
#########
#.......#
#.G@k...#
#.......#
#########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"G" = { terrain = "floor", creature = "gardener" }
"k" = { terrain = "floor", creature = "kobold", faction = "peaceful", carry = ["short_sword", "copper_coin*7", "apple*2"], wield = "short_sword" }
"##;

const THIEF: &str = "human,female,thief";

/// The built-in content with the den; with `eager`, Steal's training marks come every time.
fn files(eager: bool) -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/den.toml", DEN);
    files.insert(
        "scenarios/den.toml",
        "name = \"Den\"\nstart_level = \"den\"\n",
    );
    if eager {
        let text = include_str!("../../../../assets/data/skills.toml");
        let text = text.replace(
            "mark = [{ on = \"steal\", chance = 50 }]",
            "mark = [{ on = \"steal\", chance = 100 }]",
        );
        files.insert("data/skills.toml", &text);
    }
    files
}

fn content() -> Arc<Content> {
    Arc::new(Content::load(&files(false)).expect("test content is valid"))
}

/// A world with the character born under the sign the seed gives.
fn world_signed(scenario: &str, character: &str) -> World {
    let spec = character.parse().expect("character");
    World::with_character(content(), scenario, None, Some(&spec)).expect("world")
}

/// A world with the character born under no sign, so a sign's gifts (talents granted on the
/// run's start) leave the theft's odds and takings as the tests rig them.
fn world_as(scenario: &str, character: &str) -> World {
    let w = world_signed(scenario, character);
    let mut state = w.state().clone();
    state.character.as_mut().expect("a character").sign = None;
    World::from_state(content(), state).expect("restores")
}

fn den() -> World {
    world_as("den", THIEF)
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

/// The creature standing at `p`.
fn at(w: &World, p: Pos) -> ActorId {
    let level = &w.state.current_level;
    w.state.actor_at(level, p).expect("someone there").id
}

/// Sets `actor`'s score in `stat` (and its potential with it).
fn set_score(w: &mut World, actor: ActorId, stat: Stat, score: u8) {
    let me = w.state.actors.get_mut(&actor).expect("the actor");
    let mut scores = me.stats.scores();
    scores[stat] = score;
    me.stats = Stats::new(scores, scores);
}

/// A thief so deft and a victim so dull that only a natural 2 fails (or, `deft` false, only a
/// natural 24 works).
fn rig(w: &mut World, victim: ActorId, deft: bool) {
    let player = w.player_id();
    let (dexterity, willpower) = if deft { (40, 1) } else { (1, 40) };
    set_score(w, player, Stat::Dexterity, dexterity);
    set_score(w, victim, Stat::Willpower, willpower);
}

fn names(w: &World, id: ActorId) -> Vec<String> {
    let content = w.content();
    (w.state.actors[&id].inventory.iter())
        .map(|s| content.object(s.object).id.clone())
        .collect()
}

#[test]
fn an_unnoticed_theft_takes_one_loose_stack_never_what_is_wielded() {
    let mut w = den();
    let kobold = at(&w, Pos::new(4, 2));
    rig(&mut w, kobold, true);
    let before = w.state.time;
    let out = w.apply(&Action::Steal(Dir::E));
    assert!(out.took_turn, "stealing takes a turn");
    assert_eq!(
        w.state.time.since(before),
        Duration::seconds(6),
        "the `steal` action"
    );
    // The sword stays in its hand; the coins or the apples are the player's now.
    let left = names(&w, kobold);
    assert_eq!(left.len(), 2, "{left:?}");
    assert_eq!(left[0], "short_sword");
    let k = &w.state.actors[&kobold];
    assert!(k.equipment.holds(0) && k.equipment.is_valid_for(&k.inventory, w.content()));
    let got = if left.contains(&"apple".to_string()) {
        "7 copper coins"
    } else {
        "2 apples"
    };
    assert_eq!(last(&w), format!("You steal {got} from the kobold."));
    let mine: Vec<String> = (w.state.player().inventory.iter())
        .map(|s| w.content().object(s.object).id.clone())
        .collect();
    assert_eq!(mine.len(), 1, "{mine:?}");
    // It noticed nothing: as peaceful as before.
    assert_eq!(w.stance(w.player_id(), kobold), Stance::Neutral);
    assert_eq!(w.state.actors[&kobold].faction_override, Some(peaceful(&w)));
}

fn peaceful(w: &World) -> crate::content::FactionId {
    w.content().factions().id("peaceful").expect("peaceful")
}

#[test]
fn empty_pockets_give_nothing_and_nobody_notices() {
    let mut w = den();
    let gardener = at(&w, Pos::new(2, 2));
    rig(&mut w, gardener, true);
    let out = w.apply(&Action::Steal(Dir::W));
    assert!(out.took_turn);
    assert_eq!(last(&w), "You find nothing to take from the gardener.");
    assert!(w.state.player().inventory.is_empty());
    assert_eq!(w.stance(w.player_id(), gardener), Stance::Neutral);
}

#[test]
fn a_caught_thief_is_turned_on_and_cannot_steal_from_them_again() {
    let mut w = den();
    let kobold = at(&w, Pos::new(4, 2));
    rig(&mut w, kobold, false);
    let out = w.apply(&Action::Steal(Dir::E));
    assert!(out.took_turn);
    assert!(said(
        &w,
        "The kobold catches you stealing and turns on you!"
    ));
    let caught = w.content().steal().expect("stealing").caught;
    assert_eq!(w.state.actors[&kobold].faction_override, Some(caught));
    assert_eq!(w.stance(w.player_id(), kobold), Stance::Hostile);
    assert!(w.state.player().inventory.is_empty(), "nothing taken");
    // Caught: a deed that costs Presence marks (spec §8 "Stats").
    assert_eq!(w.state.player().stat_marks[Stat::Presence], -20);
    assert_eq!(names(&w, kobold).len(), 3);
    // Now hostile, it lets no one near its pack: refused, no time.
    let turn = w.state.turn;
    w.apply(&Action::Steal(Dir::E));
    assert_eq!(w.state.turn, turn);
    assert_eq!(
        last(&w),
        "The kobold is hostile: it would never let you near enough."
    );
}

/// The player's alignment score.
fn alignment(w: &World) -> i32 {
    w.state.player().alignment.expect("a score").value()
}

#[test]
fn only_a_theft_that_took_something_is_a_deed_for_alignment() {
    let theft = content().alignment().expect("alignment").shift.theft;
    assert!(theft < 0, "stealing is evil");
    // Unnoticed, with something to take: the theft deed's worth, once.
    let mut w = den();
    let kobold = at(&w, Pos::new(4, 2));
    rig(&mut w, kobold, true);
    let before = alignment(&w);
    w.apply(&Action::Steal(Dir::E));
    assert!(said(&w, "You steal "));
    assert!(alignment(&w) < before, "{before} -> {}", alignment(&w));
    // Empty pockets, though unnoticed, took nothing: no deed.
    let mut w = den();
    let gardener = at(&w, Pos::new(2, 2));
    rig(&mut w, gardener, true);
    let before = alignment(&w);
    w.apply(&Action::Steal(Dir::W));
    assert_eq!(last(&w), "You find nothing to take from the gardener.");
    assert_eq!(alignment(&w), before);
    // Caught: the goods stayed, and so does the score.
    let mut w = den();
    let kobold = at(&w, Pos::new(4, 2));
    rig(&mut w, kobold, false);
    let before = alignment(&w);
    w.apply(&Action::Steal(Dir::E));
    assert!(said(&w, "catches you stealing"));
    assert_eq!(alignment(&w), before);
}

#[test]
fn refusals_take_no_time() {
    let mut w = den();
    let turn = w.state.turn;
    w.apply(&Action::Steal(Dir::N));
    assert_eq!(last(&w), "You see no one there to steal from.");
    assert_eq!(w.state.turn, turn);
    // A warrior is not trained in Steal: no stealing, and no Steal in the abilities menu.
    let mut w = world_as("den", "human,male,warrior");
    w.apply(&Action::Steal(Dir::E));
    assert_eq!(last(&w), "You do not know how to steal.");
    assert_eq!(w.state.turn, turn);
    let view = w.perception();
    assert!(!view.abilities().iter().any(|a| a.ability == Ability::Steal));
}

#[test]
fn a_theft_draws_the_contest_then_the_pick_from_the_gameplay_stream() {
    // The contest: the victim's natural roll, then the thief's (four draws); with nothing loose
    // to take, nothing more.
    let mut w = den();
    let gardener = at(&w, Pos::new(2, 2));
    rig(&mut w, gardener, true);
    let mut expected = w.state.rng.clone();
    combat::roll_natural(&mut expected);
    combat::roll_natural(&mut expected);
    w.apply(&Action::Steal(Dir::W));
    assert_eq!(w.state.rng, expected);
    // Two stacks to pick from: one more draw.
    let mut w = den();
    let kobold = at(&w, Pos::new(4, 2));
    rig(&mut w, kobold, true);
    let mut expected = w.state.rng.clone();
    combat::roll_natural(&mut expected);
    combat::roll_natural(&mut expected);
    expected.index(2);
    w.apply(&Action::Steal(Dir::E));
    assert_eq!(w.state.rng, expected);
}

#[test]
fn every_attempt_trains_steal() {
    let spec = THIEF.parse().expect("thief");
    let content = Arc::new(Content::load(&files(true)).expect("eager content"));
    let mut w = World::with_character(content, "den", None, Some(&spec)).expect("den");
    let steal = w.content().skill_id("steal").expect("steal");
    let marks = |w: &World| (w.state.player().skills.get(steal)).map(|t| t.marks);
    let before = marks(&w);
    assert!(before.is_some(), "the thief is trained in Steal");
    w.apply(&Action::Steal(Dir::W));
    assert_ne!(marks(&w), before, "a mark (or a point) for trying");
}

#[test]
fn the_abilities_menu_offers_steal_in_a_direction() {
    let w = den();
    let view = w.perception();
    let abilities = view.abilities();
    let own = (abilities.iter())
        .find(|a| a.ability == Ability::Steal)
        .expect("the thief can steal");
    assert_eq!(own.unavailable, None);
    assert_eq!(
        Ability::Steal.action(w.content(), Some(Dir::E)),
        Some(Action::Steal(Dir::E))
    );
}

/// The courtyard as a thief, in Grimbold's General Store, in front of him (north), carrying 60
/// gold and a scroll of light (shop_tests' walk).
fn in_shop() -> World {
    let mut w = world_as("courtyard", THIEF);
    apply(
        &mut w,
        "move:E*36 move:S pickup move:NE move:NE pickup:b move:SE move:E*5 move:S*5 move:E*5",
    );
    assert_eq!(w.player_pos(), Pos::new(54, 14), "in the shop");
    w
}

fn gold(w: &World, id: ActorId) -> u32 {
    let currency = w.content().shops().expect("shops").currency;
    count_of(&w.state.actors[&id].inventory, currency)
}

#[test]
fn a_shopkeepers_purse_can_be_lifted_and_the_shop_stays_open() {
    let mut w = in_shop();
    let keeper = w.state.current().shops[0].keeper;
    rig(&mut w, keeper, true);
    let purse = gold(&w, keeper);
    assert!(purse > 0);
    let worn = w.state.actors[&keeper].equipment.items().len();
    w.apply(&Action::Steal(Dir::N));
    assert_eq!(
        last(&w),
        format!("You steal {purse} gold coins from Grimbold.")
    );
    assert_eq!(gold(&w, w.player_id()), 60 + purse);
    assert_eq!(gold(&w, keeper), 0);
    let shop = &w.state.current().shops[0];
    assert_eq!(shop.status, ShopStatus::Open);
    assert_eq!(w.stance(w.player_id(), keeper), Stance::Neutral);
    // What it wears and wields stays on it.
    let k = &w.state.actors[&keeper];
    assert!(k.equipment.is_valid_for(&k.inventory, w.content()));
    assert_eq!(
        k.equipment.items().len(),
        worn,
        "the spear, the shield, the mail, the helm"
    );
}

#[test]
fn a_shopkeeper_who_catches_a_thief_is_outraged() {
    let mut w = in_shop();
    let keeper = w.state.current().shops[0].keeper;
    // Owing for a dagger first: the bill becomes theft.
    apply(&mut w, "move:E move:NE pickup move:SW move:W");
    assert!(!w.state.current().shops[0].unpaid.is_empty());
    rig(&mut w, keeper, false);
    w.apply(&Action::Steal(Dir::N));
    assert!(said(&w, "Grimbold catches you stealing. \"Stop, thief!\""));
    assert_eq!(
        w.state.player().stat_marks[Stat::Presence],
        -20,
        "a keeper catches you too"
    );
    let shop = &w.state.current().shops[0];
    assert_eq!(shop.status, ShopStatus::Outraged);
    assert!(shop.unpaid.is_empty());
    assert_eq!(w.stance(w.player_id(), keeper), Stance::Hostile);
    assert_eq!(gold(&w, w.player_id()), 60);
}

#[test]
fn thefts_save_replay_and_hash() {
    // Signed: a replay restarts the character from its spec, under the sign the seed gives.
    let mut w = world_signed("den", THIEF);
    let mut replay = Replay::new(&w);
    for action in parse_actions("steal:E steal:W wait steal:E").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let (again, divergence) = replay.play(content()).expect("plays");
    assert!(divergence.is_none(), "the replay verifies");
    assert_eq!(again.state_hash(), w.state_hash());
    let bytes = crate::save::encode_state(w.state(), w.fingerprint());
    let loaded = crate::save::decode(&bytes, content()).expect("loads");
    assert_eq!(loaded.state_hash(), w.state_hash());
}
