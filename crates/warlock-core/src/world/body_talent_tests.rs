//! The Vigour, Might, and Finesse talents (spec §8 "Talents"; #237): each changes the one rule it
//! names, Second wind is an ability with a cooldown, and the prerequisites that ask for any of
//! several weapon skills are checked and said in words.

use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::{AttackKind, parse_actions};
use crate::character::CharacterSpec;
use crate::combat::{Blow, Category, Strike};
use crate::content::{ContentFiles, Factor, StatusId, Trap};
use crate::damage::Damage;
use crate::equipment::Equipment;
use crate::stats::{PerStat, Stat, Stats};
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// A human warrior in the showcase, every score 14.
fn world() -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let mut w =
        World::with_character(BUILTIN.clone(), "showcase", Some(7), Some(&spec)).expect("world");
    set_scores(&mut w, 14);
    w
}

fn run(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

fn last(world: &World) -> String {
    said(world).last().cloned().unwrap_or_default()
}

fn set_scores(world: &mut World, score: u8) {
    world.player_mut().stats = Stats::new(PerStat::splat(score), PerStat::splat(40));
}

/// The player takes talent `id` (one rank more), without a level to pick it at.
fn give(world: &mut World, id: &str) {
    world.player_mut().talents.take(id, None);
}

fn status(world: &World, id: &str) -> StatusId {
    world.content().status_id(id).expect("a status")
}

fn suffers(world: &World, who: ActorId, id: &str) -> bool {
    world.state.actors[&who].effects.has(status(world, id))
}

/// A creature on the player's level.
fn creature(world: &World) -> ActorId {
    let level = world.state.current().id.clone();
    (world.state.actors.values())
        .find(|a| a.id != world.state.player && a.level == level)
        .map(|a| a.id)
        .expect("a creature in the showcase")
}

#[test]
fn fast_healer_heals_resting_half_again_as_fast() {
    let mut w = world();
    let id = w.state.player;
    w.resting = true;
    let resting = w.healing_rate(id);
    give(&mut w, "fast_healer");
    assert_eq!(w.healing_rate(id), resting * 3 / 2, "200 % becomes 300 %");
    w.resting = false;
    assert_eq!(w.healing_rate(id), resting / 2, "awake, nothing changes");
}

#[test]
fn second_wind_gives_hit_points_back_then_winds() {
    let mut w = world();
    run(&mut w, "second_wind");
    assert_eq!(last(&w), "You have no second wind to catch.");
    assert!(!(w.perception().abilities().iter()).any(|a| a.ability == Ability::SecondWind));

    give(&mut w, "second_wind");
    let own = |w: &World| {
        (w.perception().abilities().into_iter())
            .find(|a| a.ability == Ability::SecondWind)
            .expect("listed")
    };
    assert_eq!(own(&w).unavailable, None);
    w.player_mut().hp = 1;
    let turn = w.state.turn;
    run(&mut w, "second_wind");
    assert!(said(&w).contains(&"You catch your second wind.".to_string()));
    assert_eq!(w.state.player().hp, 9, "1 per level, at least 8");
    assert_eq!(w.state.turn, turn + 1, "it takes a turn");
    let player = w.state.player;
    assert!(suffers(&w, player, "winded"));
    let winded = status(&w, "winded");
    assert_eq!(own(&w).unavailable, Some(Unavailable::While(winded)));
    run(&mut w, "second_wind");
    assert_eq!(last(&w), "You are still winded.");
    assert_eq!(w.state.player().hp, 9);
    assert_eq!(w.state.turn, turn + 1, "refused: no time");
    // A quarter of an hour later the breath is back.
    run(&mut w, "wait*151");
    assert!(!suffers(&w, player, "winded"));
    assert_eq!(own(&w).unavailable, None);
}

#[test]
fn poison_resistance_shortens_poison_and_spares_the_scores() {
    let strength = |w: &World| {
        w.state
            .player()
            .effective_stats(w.content())
            .score(Stat::Strength)
    };
    let player = |w: &World| w.state.player;
    // Strong poison lowers Strength; poisoned once, it lasts its two minutes.
    let mut w = world();
    run(&mut w, "afflict:poisoned*5");
    assert!(strength(&w) < 14);
    let mut w = world();
    run(&mut w, "afflict:poisoned wait*11");
    assert!(
        suffers(&w, player(&w), "poisoned"),
        "66 s in, still poisoned"
    );
    // With the talent: not weakened, and it wears off twice as fast.
    let mut w = world();
    give(&mut w, "poison_resistance");
    run(&mut w, "afflict:poisoned*5");
    assert_eq!(strength(&w), 14);
    let mut w = world();
    give(&mut w, "poison_resistance");
    run(&mut w, "afflict:poisoned wait*11");
    assert!(!suffers(&w, player(&w), "poisoned"));
}

#[test]
fn unbowed_halves_what_statuses_take_and_keeps_what_they_give() {
    let strength = |w: &World| {
        w.state
            .player()
            .effective_stats(w.content())
            .score(Stat::Strength)
    };
    let mut w = world();
    run(&mut w, "afflict:exhausted");
    assert_eq!(strength(&w), 13);
    let mut w = world();
    give(&mut w, "unbowed");
    run(&mut w, "afflict:exhausted");
    assert_eq!(strength(&w), 14, "-1 halved toward 0");
    run(&mut w, "resource:rage");
    assert_eq!(strength(&w), 16, "a raise stays whole");
}

#[test]
fn clotting_stanches_big_hits_and_criticals_and_bleeding_stops_sooner() {
    let content = BUILTIN.clone();
    let slashing = content.damage_type_id("slashing");
    let bleeding = |w: &World| {
        let id = status(w, "bleeding");
        w.state.player().effects.get(id).map_or(0, |e| e.severity)
    };
    let hit = |w: &mut World, critical: bool| {
        let player = w.state.player;
        let max = w.state.player().max_hp(w.content());
        let blow = Blow {
            strike: Strike::Hit { critical },
            damage: max / 2,
            kind: slashing,
        };
        w.wounded(player, &blow, None, &mut Vec::new());
    };
    let mut w = world();
    hit(&mut w, false);
    assert_eq!(bleeding(&w), 1, "a big hit bleeds");
    let mut w = world();
    hit(&mut w, true);
    assert_eq!(bleeding(&w), 2, "a critical bleeds more");
    let mut w = world();
    give(&mut w, "clotting");
    hit(&mut w, false);
    assert_eq!(bleeding(&w), 0);
    hit(&mut w, true);
    assert_eq!(bleeding(&w), 1, "a degree less");
    let talents = w.state.player().talent_mods(&content);
    assert_eq!(talents.eases_every(status(&w, "bleeding"), 60), 30);
}

#[test]
fn strong_constitution_and_fever_proof_are_read_by_disease_checks() {
    let mut w = world();
    give(&mut w, "strong_constitution");
    give(&mut w, "fever_proof");
    let talents = w.state.player().talent_mods(w.content());
    assert_eq!((talents.disease_checks, talents.disease_recovery), (2, 1));
    // A disease that needs two good checks in a row goes after one with Fever-proof: a
    // character this tough wins every check but a natural 2.
    set_scores(&mut w, 40);
    run(&mut w, "infect:filth_fever");
    let player = w.state.player;
    let fever = w.content().disease_id("filth_fever").expect("filth fever");
    let needs = w.content().disease(fever).recovery;
    assert!(needs >= 2, "filth fever needs {needs}");
    for _ in 0..40 {
        if !w.state.player().diseases.has(fever) {
            break;
        }
        run(&mut w, "rest:24h");
    }
    assert!(!w.state.actors[&player].diseases.has(fever), "shaken off");
}

#[test]
fn mighty_kick_and_crusher_change_kicks_and_breaker_digs_faster() {
    let mut w = world();
    let content = w.content().clone();
    let kick = |w: &World| w.state.player().kick(&content);
    assert_eq!((kick(&w).damage.max(), kick(&w).penetration), (3, 0));
    give(&mut w, "mighty_kick");
    assert_eq!(kick(&w).damage.max(), 6);
    give(&mut w, "crusher");
    assert_eq!(kick(&w).penetration, 2, "a kick is bludgeoning");
    let talents = w.state.player().talent_mods(&content);
    assert_eq!(talents.door_kick, 4);
    give(&mut w, "breaker");
    let talents = w.state.player().talent_mods(&content);
    assert_eq!((talents.door_kick, talents.dig_percent), (8, 25));
}

#[test]
fn iron_grip_cheapens_blows_with_two_handed_weapons_only() {
    let mut w = world();
    let content = w.content().clone();
    let id = w.state.player;
    let sword = content.object_id("great_sword").expect("a great sword");
    let dagger = content.object_id("dagger").expect("a dagger");
    let me = w.player_mut();
    me.inventory.push(Stack::one(sword));
    me.inventory.push(Stack::one(dagger));
    let (great, small) = (me.inventory.len() - 2, me.inventory.len() - 1);
    let price = |w: &World, item| {
        w.attack_price(id, ActionKind::Melee, Some(item), AttackSkills::default())
    };
    let (before, light) = (price(&w, great), price(&w, small));
    give(&mut w, "iron_grip");
    assert_eq!(price(&w, great), before * 90 / 100);
    assert_eq!(price(&w, small), light);
}

#[test]
fn a_staggering_blow_staggers_on_a_confirmed_critical_in_melee() {
    let mut w = world();
    let player = w.state.player;
    let foe = creature(&w);
    give(&mut w, "staggering_blow");
    let blow = |critical| Blow {
        strike: Strike::Hit { critical },
        damage: 1,
        kind: None,
    };
    w.after_blow(
        player,
        foe,
        AttackKind::Ranged,
        (&blow(true), &[]),
        &mut Vec::new(),
    );
    assert!(!suffers(&w, foe, "staggered"), "a shot does not");
    w.after_blow(
        player,
        foe,
        AttackKind::Melee,
        (&blow(false), &[]),
        &mut Vec::new(),
    );
    assert!(!suffers(&w, foe, "staggered"), "a plain hit does not");
    w.after_blow(
        player,
        foe,
        AttackKind::Melee,
        (&blow(true), &[]),
        &mut Vec::new(),
    );
    assert!(suffers(&w, foe, "staggered"));
    // Staggered, its next action costs half again as much.
    let wait = w.price_of(foe, ActionKind::Wait);
    let staggered = status(&w, "staggered");
    w.state
        .actors
        .get_mut(&foe)
        .expect("foe")
        .effects
        .remove(staggered);
    assert_eq!(wait, w.price_of(foe, ActionKind::Wait) * 3 / 2);
}

#[test]
fn nimble_raises_the_burden_cap_on_dexterity() {
    let mut w = world();
    set_scores(&mut w, 20);
    w.player_mut().equipment = Equipment::default();
    let content = w.content().clone();
    let dex = |w: &World| {
        (w.state.player().defence(&content, Burden::Strained, None)).bonus(Category::Dexterity)
    };
    assert_eq!(dex(&w), 1, "Strained caps it at +1");
    give(&mut w, "nimble");
    assert_eq!(dex(&w), 2);
}

/// The world with a pit next to the player; returns the way to it.
fn beside_a_pit(w: &mut World) -> Dir {
    let content = w.content().clone();
    let here = w.player_pos();
    let level = w.state.current();
    let dir = (Dir::ALL.into_iter())
        .find(|d| {
            let to = here.step(*d);
            level.is_walkable(&content, to)
                && w.state.actor_at(&level.id, to).is_none()
                && !level.traps.contains_key(&to)
        })
        .expect("room to step");
    let pit = content.trap_id("pit").expect("a pit");
    let trap = Trap::new(pit, content.trap(pit), false, None);
    w.state.current_mut().traps.insert(here.step(dir), trap);
    dir
}

#[test]
fn light_step_keeps_traps_from_going_off_under_the_player() {
    // Without it, the pit catches them.
    let mut w = world();
    let dir = beside_a_pit(&mut w);
    run(&mut w, &format!("move:{dir}"));
    assert!(!said(&w).iter().any(|m| m.contains("step lightly")));
    // Rank 1: a Dexterity check (this nimble a character makes it).
    let mut w = world();
    set_scores(&mut w, 40);
    give(&mut w, "light_step");
    let dir = beside_a_pit(&mut w);
    let hp = w.state.player().hp;
    run(&mut w, &format!("move:{dir}"));
    assert_eq!(last(&w), "You step lightly off a pit.");
    assert_eq!(w.state.player().hp, hp);
    let at = w.player_pos();
    assert!(w.state.current().traps[&at].known, "found");
    // Rank 2: pits never catch them, however clumsy.
    let mut w = world();
    set_scores(&mut w, 3);
    give(&mut w, "light_step");
    give(&mut w, "light_step");
    let dir = beside_a_pit(&mut w);
    run(&mut w, &format!("move:{dir}"));
    assert_eq!(last(&w), "You step lightly off a pit.");
}

#[test]
fn quick_makes_every_action_a_little_cheaper() {
    let mut w = world();
    let id = w.state.player;
    let wait = w.price_of(id, ActionKind::Wait);
    give(&mut w, "quick");
    assert_eq!(
        w.price_of(id, ActionKind::Wait),
        (wait * 100).div_euclid(105) + 1
    );
    give(&mut w, "quick");
    assert_eq!(
        w.price_of(id, ActionKind::Wait),
        (wait * 100).div_euclid(110) + 1
    );
}

#[test]
fn deflect_missiles_dodges_missiles_and_evasion_halves_bursts() {
    let mut w = world();
    let id = w.state.player;
    let dodge = |w: &World, missile| w.target_against(id, missile).defence.bonus(Category::Dodge);
    let (plain, shot) = (dodge(&w, false), dodge(&w, true));
    assert_eq!(plain, shot);
    give(&mut w, "deflect_missiles");
    assert_eq!((dodge(&w, false), dodge(&w, true)), (plain, plain + 2));
    let fire = w.content().damage_type_id("fire");
    assert_eq!(w.evaded(id, Damage::new(11, fire)).amount, 11);
    give(&mut w, "evasion");
    assert_eq!(w.evaded(id, Damage::new(11, fire)).amount, 5);
}

#[test]
fn free_blow_talents_switch_their_blows_on() {
    let mut w = world();
    for id in ["cleave", "great_cleave", "riposte", "opportunist"] {
        give(&mut w, id);
    }
    let blows = w.free_blows_of(w.state.player());
    assert!(blows.cleave && blows.great_cleave && blows.riposte && blows.opportunist);
}

#[test]
fn a_prerequisite_may_ask_for_any_of_several_weapon_skills() {
    let mut w = world();
    let spec = w.state.character.as_ref().map(|c| &c.spec);
    let factor = Factor::of(w.content(), spec);
    let need = w.content().experience().threshold_for(3, factor);
    run(&mut w, &format!("xp:{need}"));
    // A warrior starts with some weapon skills; none of them at 3 yet.
    let table = w.content().weapon_skills().clone();
    w.player_mut().weapon_skills = Default::default();
    run(&mut w, "talent:cleave");
    assert_eq!(
        last(&w),
        "You cannot take Cleave: you need a melee or unarmed weapon skill 3."
    );
    run(&mut w, "talent:crusher");
    assert_eq!(
        last(&w),
        "You cannot take Crusher: you need Maces and clubs or Unarmed 3."
    );
    let axes = table.id("axes").expect("axes");
    w.player_mut().weapon_skills.raise(&table, axes, 3);
    run(&mut w, "talent:cleave");
    assert_eq!(last(&w), "You learn Cleave.");
}

#[test]
fn every_talent_of_the_three_trees_loads_with_its_effects() {
    let files = ContentFiles::builtin();
    let content = Content::load(&files).expect("content");
    let book = content.talents();
    for tree in ["vigour", "might", "finesse"] {
        let tree = book.tree_id(tree).expect("a tree");
        assert!(!book.tree(tree).pending, "the tree is held to the bar");
    }
    let count = |tree: &str| book.of_tree(book.tree_id(tree).expect("tree")).count();
    // The design's 28, Battle trance and Inner calm (the class resources, R5) among them,
    // stamina's Deep lungs and Second breath (#560), and Second blade's rend (#561).
    assert_eq!(count("vigour") + count("might") + count("finesse"), 31);
}
