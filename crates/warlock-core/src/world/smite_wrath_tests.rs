//! Holy wrath, Banishing smite, and Avatar's wrath (`world/smite.rs`; spec §8 "Class abilities";
//! the paladin design P4, #913).

use super::class_active_tests::{at_normal_speed, beside, last, reach, run, world};
use super::smite::SmiteMode;
use super::*;
use crate::ability::Ability;
use crate::action::{Action, parse_actions};
use crate::combat::{Blow, Strike};
use crate::content::{ClassActive, ResourceId};
use crate::replay::Replay;

const HIT: Blow = Blow {
    strike: Strike::Hit { critical: false },
    damage: 10,
    kind: None,
};

fn zeal(w: &World) -> ResourceId {
    w.content().resource_id("zeal").expect("zeal")
}

fn held(w: &World) -> Option<u16> {
    w.state.player().resources.get(zeal(w)).map(|p| p.value)
}

fn set(w: &mut World, value: u16) {
    let id = zeal(w);
    w.player_mut().resources.set(id, value);
}

/// A foe beside the player, in their sight, with `hp` hit points, that fights back for nothing.
fn foe(w: &mut World, creature: &str, hp: i32) -> ActorId {
    let id = beside(w, creature, hp);
    w.player_mut().hp = 100_000;
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    id
}

/// What Holy wrath adds to a blow that hit `target`, `n` times over.
fn wrath_on(w: &mut World, target: ActorId, n: usize) -> Vec<i32> {
    let player = w.state.player;
    (0..n)
        .map(|_| w.holy_wrathed(player, target, HIT).damage - HIT.damage)
        .collect()
}

#[test]
fn the_paladin_gains_holy_wrath_at_40_and_banishing_at_50() {
    let mut w = world("paladin");
    let holds = |w: &World| {
        let mods = w.state.player().talent_mods(w.content());
        let banish = mods.has_active(ClassActive::BanishingSmite);
        (mods.holy_wrath.is_some(), banish)
    };
    reach(&mut w, 39);
    assert_eq!(holds(&w), (false, false));
    reach(&mut w, 40);
    assert_eq!(holds(&w), (true, false));
    reach(&mut w, 49);
    assert_eq!(holds(&w), (true, false));
    reach(&mut w, 50);
    assert_eq!(holds(&w), (true, true));
    let rows: Vec<_> = (w.perception().abilities().into_iter())
        .map(|a| a.ability)
        .collect();
    assert!(
        rows.contains(&Ability::Class(ClassActive::BanishingSmite)),
        "a menu row"
    );
    let action = &parse_actions("use:banishing_smite:E").expect("parses")[0];
    assert!(matches!(action, Action::Use(a, Some(_)) if *a == ClassActive::BanishingSmite));
    assert_eq!(action.to_string(), "use:banishing_smite:E");
}

#[test]
fn holy_wrath_falls_on_the_unholy_alone_until_avatar() {
    let mut w = world("paladin");
    reach(&mut w, 40);
    let draugr = foe(&mut w, "draugr", 500);
    let wrath = wrath_on(&mut w, draugr, 60);
    assert!(wrath.iter().all(|d| (1..=6).contains(d)), "{wrath:?}");
    assert!(wrath.iter().any(|d| *d != wrath[0]), "a die: {wrath:?}");
    w.state.actors.remove(&draugr);
    let goblin = foe(&mut w, "goblin", 500);
    assert!(!w.content().unholy(w.state.actors[&goblin].creature));
    let rng = w.state.rng.clone();
    assert_eq!(wrath_on(&mut w, goblin, 10), vec![0; 10]);
    assert_eq!(w.state.rng, rng, "no draw when it does not fall");
    // Avatar: on every foe, the goblin too.
    reach(&mut w, 60);
    let wrath = wrath_on(&mut w, goblin, 60);
    assert!(wrath.iter().all(|d| (1..=6).contains(d)), "{wrath:?}");
    // Avatar alone is no wrath: the die is Holy wrath's.
    w.player_mut().talents.forget("holy_wrath");
    assert_eq!(wrath_on(&mut w, goblin, 10), vec![0; 10]);
}

#[test]
fn holy_wrath_asks_a_hit_and_is_resisted() {
    let mut w = world("paladin");
    reach(&mut w, 40);
    let player = w.state.player;
    let draugr = foe(&mut w, "draugr", 500);
    // A miss: nothing, and no draw.
    let rng = w.state.rng.clone();
    let missed = Blow::missed(None);
    assert_eq!(w.holy_wrathed(player, draugr, missed), missed);
    assert_eq!(w.state.rng, rng);
    // A blow soaked to nothing still carries it: it is never soaked.
    let soaked = Blow { damage: 0, ..HIT };
    assert!(w.holy_wrathed(player, draugr, soaked).damage >= 1);
    // Of the blow's kind, resisted: a type immune to its kind takes none of it.
    let poison = w.content().damage_type_id("poison").expect("poison");
    let blow = Blow {
        kind: Some(poison),
        ..HIT
    };
    assert_eq!(w.holy_wrathed(player, draugr, blow).damage, HIT.damage);
    // Nobody else has it: the draugr's own blow on the player adds nothing.
    assert_eq!(w.holy_wrathed(draugr, player, HIT), HIT);
}

#[test]
fn a_blow_at_the_unholy_carries_it_through_resolve() {
    let mut w = world("paladin");
    reach(&mut w, 40);
    at_normal_speed(&mut w);
    let player = w.state.player;
    let draugr = foe(&mut w, "draugr", 100_000);
    let profile = w.state.player().melee(w.content());
    let kind = (
        crate::action::AttackKind::Melee,
        None,
        AttackSkills::default(),
    );
    let mut more = Vec::new();
    for _ in 0..40 {
        // The same draws with the talent and without: the difference is the die.
        let mut without =
            World::from_state(w.content().clone(), (*w.state).clone()).expect("restores");
        without.player_mut().talents.forget("holy_wrath");
        let mut events = Vec::new();
        let plain = without.resolve(
            player,
            draugr,
            kind,
            Some(profile.clone()),
            false,
            &mut events,
        );
        let blow = w.resolve(
            player,
            draugr,
            kind,
            Some(profile.clone()),
            false,
            &mut events,
        );
        if let (Some(plain), Some(blow)) = (plain, blow)
            && blow.strike.is_hit()
        {
            more.push(blow.damage - plain.damage);
        }
    }
    assert!(!more.is_empty(), "a blow hit");
    assert!(more.iter().all(|d| (1..=6).contains(d)), "{more:?}");
}

#[test]
fn a_banishing_smite_unmakes_the_summoned_whatever_its_hit_points() {
    let mut w = world("paladin");
    reach(&mut w, 50);
    at_normal_speed(&mut w);
    let player = w.state.player;
    let imp = foe(&mut w, "imp", 100_000);
    w.state.actors.get_mut(&imp).expect("the imp").summoner = Some(player);
    // It does not fade from sight meanwhile (`unseen.rs`).
    let content = w.content.clone();
    let spent = content.status_id("invisibility").expect("the cooldown");
    let imp_actor = w.state.actors.get_mut(&imp).expect("the imp");
    imp_actor
        .effects
        .add(spent, content.status(spent), 100, None);
    // Summoned, its smite is a plain smite's dice: the unmaking is what it does.
    let plain = w.smite_profile(imp).expect("a smite");
    let banish = w.mode_profile(SmiteMode::Banishing, imp).expect("a smite");
    assert_eq!(plain.extra, banish.extra);
    let xp = w.player_xp();
    let mut tries = 0;
    while w.state.actors.contains_key(&imp) {
        tries += 1;
        assert!(tries < 100, "some blow lands");
        w.player_mut().hp = 100_000;
        set(&mut w, 6);
        let from = w.messages().len();
        run(&mut w, "use:banishing_smite:E");
        let said: Vec<_> = (w.messages().iter().skip(from))
            .map(|m| m.display())
            .collect();
        assert_eq!(said.first().map(String::as_str), Some("You smite!"));
        let unmade = said.iter().any(|l| l == "The imp is unmade!");
        assert_eq!(
            unmade,
            !w.state.actors.contains_key(&imp),
            "said when it dies: {said:?}"
        );
    }
    assert!(w.player_xp() > xp, "the kill pays");
}

#[test]
fn a_banishing_smite_falls_twice_on_the_unholy_and_plain_on_the_rest() {
    let mut w = world("paladin");
    reach(&mut w, 50);
    at_normal_speed(&mut w);
    let draugr = foe(&mut w, "draugr", 100_000);
    assert!(w.content().unholy(w.state.actors[&draugr].creature));
    let plain = w.smite_profile(draugr).expect("a smite");
    let twice = w
        .mode_profile(SmiteMode::Banishing, draugr)
        .expect("a smite");
    let dice = plain.extra.expect("its dice");
    assert_eq!(dice.count(), 6, "a die per ten levels and the unholy's");
    assert_eq!(twice.extra.map(|d| d.count()), Some(12));
    assert_eq!(twice.extra.map(|d| d.sides()), Some(dice.sides()));
    assert_eq!(twice.bonus, plain.bonus, "the bonus once");
    assert_eq!(twice.damage, plain.damage);
    // It is told, and lives.
    let mut tries = 0;
    loop {
        tries += 1;
        assert!(tries < 100, "some blow lands");
        w.player_mut().hp = 100_000;
        set(&mut w, 6);
        run(&mut w, "use:banishing_smite:E");
        let said: Vec<_> = w.messages().iter().map(|m| m.display()).collect();
        if said
            .iter()
            .any(|l| l == "The draugr reels under holy wrath!")
        {
            break;
        }
    }
    assert!(w.state.actors.contains_key(&draugr));
    w.state.actors.remove(&draugr);
    // A goblin: a plain smite's dice, and its zeal spent all the same.
    let goblin = foe(&mut w, "goblin", 100_000);
    let plain = w.smite_profile(goblin).expect("a smite");
    let banish = w
        .mode_profile(SmiteMode::Banishing, goblin)
        .expect("a smite");
    assert_eq!(banish.extra, plain.extra);
    w.player_mut().hp = 100_000;
    set(&mut w, 6);
    let from = w.messages().len();
    run(&mut w, "use:banishing_smite:E");
    assert!(held(&w) <= Some(3), "six spent: {:?}", held(&w));
    assert!(w.state.actors.contains_key(&goblin));
    let said: Vec<_> = (w.messages().iter().skip(from))
        .map(|m| m.display())
        .collect();
    assert!(
        !(said.iter()).any(|l| l.contains("unmade") || l.contains("reels")),
        "{said:?}"
    );
}

#[test]
fn a_banishing_smite_spends_nothing_when_refused_or_unanswered() {
    let mut w = world("paladin");
    reach(&mut w, 50);
    at_normal_speed(&mut w);
    set(&mut w, 6);
    let turn = w.state.turn;
    run(&mut w, "use:banishing_smite:E");
    assert_eq!(last(&w), "There is nobody there to strike.");
    assert_eq!((w.state.turn, held(&w)), (turn, Some(6)));
    foe(&mut w, "kobold", 100_000);
    set(&mut w, 5);
    run(&mut w, "use:banishing_smite:E");
    assert_eq!(last(&w), "Your zeal is not enough.");
    assert_eq!((w.state.turn, held(&w)), (turn, Some(5)));
    w.player_mut().alignment = Some(crate::alignment::Score::new(0));
    set(&mut w, 6);
    let from = w.messages().len();
    run(&mut w, "use:banishing_smite:E");
    let first = w.messages().iter().nth(from).map(|m| m.display());
    assert_eq!(
        first.as_deref(),
        Some("You call for holy wrath, but none answers.")
    );
    assert_eq!(w.state.turn, turn + 1);
    assert!(held(&w) >= Some(6), "nothing spent");
}

#[test]
fn a_replay_of_the_wrath_verifies() {
    let content = std::sync::Arc::new(crate::content::Content::builtin().expect("content"));
    let spec: crate::character::CharacterSpec = "human,male,paladin".parse().expect("spec");
    let mut w = World::start_run(content.clone(), "arena", Some(7), Some(&spec), None, true)
        .expect("world");
    let mut replay = Replay::new(&w);
    let actions =
        "xp:90000000 levelup:auto use:banishing_smite:E move:E use:banishing_smite:E wait*3";
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(w.state.player().talents.has("banishing_smite"));
    Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content)
        .expect("the replay verifies");
}
