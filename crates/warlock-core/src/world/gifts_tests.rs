//! The birth sign's gifts from the Salamander to the Basilisk (spec §8 "Birth sign"; #266):
//! each changes the one rule it names, the two actives are used from the abilities menu with
//! their cooldowns, and a character born under another sign keeps the rule as it was.

use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::{AttackKind, parse_actions};
use crate::character::CharacterSpec;
use crate::combat::{Blow, Profile, Strike};
use crate::content::{Sign, SignActive, StatusId};
use crate::damage::Damage;
use crate::geom::Pos;
use crate::stats::{PerStat, Stats};
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// A human warrior in `scenario` (seed 7), born under the `element` `animal` (`None`: under no
/// sign), every score 14; the run restored, so the gifts are granted as a load grants them.
fn born(scenario: &str, sign: Option<(&str, &str)>) -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let w = World::with_character(BUILTIN.clone(), scenario, Some(7), Some(&spec)).expect("world");
    let mut state = w.state().clone();
    let character = state.character.as_mut().expect("a character");
    character.sign = sign.map(|(element, animal)| Sign {
        animal: animal.into(),
        element: element.into(),
    });
    character.born.day = 2;
    let mut w = World::from_state(BUILTIN.clone(), state).expect("restores");
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = max;
    w
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn last(w: &World) -> String {
    (w.messages().iter().map(|m| m.display()).next_back()).unwrap_or_default()
}

fn status(w: &World, id: &str) -> StatusId {
    w.content().status_id(id).expect("a status")
}

fn suffers(w: &World, who: ActorId, id: &str) -> bool {
    w.state.actors[&who].effects.has(status(w, id))
}

/// A kobold (hostile to the player) placed at `at` on the player's level; its id.
fn foe_at(w: &mut World, at: Pos) -> ActorId {
    let content = w.content().clone();
    let template = (w.state.actors.values())
        .find(|a| a.id != w.state.player)
        .cloned()
        .expect("a creature in the showcase");
    let id = ActorId(w.state.next_actor_id);
    w.state.next_actor_id += 1;
    let creature = content.creature_id("kobold").expect("kobolds");
    let def = content.creature(creature);
    let actor = Actor {
        id,
        creature,
        pos: at,
        patrol: None,
        npc: None,
        faction_override: None,
        summoner: None,
        alignment_override: None,
        inventory: Vec::new(),
        equipment: Default::default(),
        stats: def.stats,
        skills: def.skills.clone(),
        weapon_skills: def.weapon_skills.clone(),
        effects: Default::default(),
        hp_base: 10,
        hp: 10,
        level: w.state.current_level.clone(),
        ..template
    };
    w.state.actors.insert(id, actor);
    id
}

fn own(w: &World, active: SignActive) -> Option<Option<Unavailable>> {
    (w.perception().abilities().into_iter())
        .find(|a| a.ability == Ability::Sign(active))
        .map(|a| a.unavailable)
}

#[test]
fn each_sign_grants_its_gifts_and_flaw_in_order() {
    let gifts = |animal: &str| {
        let w = born("showcase", Some(("wood", animal)));
        let book = w.content().talents();
        (w.state.player().talents.iter())
            .filter(|t| book.id(&t.id).is_some_and(|id| book.get(id).granted))
            .filter(|t| !["supple", "evergreen"].contains(&t.id.as_str()))
            .filter(|t| !book.tree(book.get(book.id(&t.id).unwrap()).tree).class)
            .filter(|t| t.id != "versatile")
            .map(|t| t.id.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(gifts("salamander"), ["fleet_footed", "slip_away", "slight"]);
    assert_eq!(gifts("hydra"), ["regrowth", "strong_swimmer", "many_minds"]);
    assert_eq!(
        gifts("manticore"),
        ["relentless", "barbed_hide", "little_loved"]
    );
    assert_eq!(
        gifts("unicorn"),
        ["pure_blood", "horns_grace", "thin_skinned"]
    );
    assert_eq!(
        gifts("wyvern"),
        ["trailblazer", "flame_hardened", "heavy_footed"]
    );
    assert_eq!(
        gifts("basilisk"),
        ["unblinking", "transfixing_stare", "trusted_by_none"]
    );
}

#[test]
fn slip_away_makes_a_step_out_of_reach_cheaper() {
    let price = 1000;
    for (sign, cheaper) in [(Some(("wood", "salamander")), 750), (None, 1000)] {
        let mut w = born("showcase", sign);
        let me = w.player_pos();
        let player = w.state.player;
        foe_at(&mut w, me.step(Dir::E));
        // West, out of its reach: a quarter off for the Salamander.
        assert_eq!(w.slipped(player, me.step(Dir::W), price), cheaper);
        // North, still beside it: the full price.
        assert_eq!(w.slipped(player, me.step(Dir::N), price), price);
    }
    // With no one hostile beside, nothing to slip away from.
    let w = born("showcase", Some(("wood", "salamander")));
    let (me, player) = (w.player_pos(), w.state.player);
    assert_eq!(w.slipped(player, me.step(Dir::W), price), price);
}

#[test]
fn many_minds_divides_the_will_and_heavy_feet_and_keen_eyes_are_data() {
    use crate::stats::Stat;
    let hydra = born("showcase", Some(("wood", "hydra")));
    let mods = hydra.state.player().talent_mods(hydra.content());
    assert_eq!(mods.checks[Stat::Willpower], -2);
    let wyvern = born("showcase", Some(("wood", "wyvern")));
    let mods = wyvern.state.player().talent_mods(wyvern.content());
    assert_eq!(mods.footsteps_percent, 25);
    let basilisk = born("showcase", Some(("wood", "basilisk")));
    let mods = basilisk.state.player().talent_mods(basilisk.content());
    assert_eq!(mods.notice, Some(2));
    assert!(mods.distrusted);
}

#[test]
fn relentless_lands_more_surely_blow_after_blow_at_one_foe() {
    let mut w = born("showcase", Some(("wood", "manticore")));
    let me = w.player_pos();
    let player = w.state.player;
    let kobold = foe_at(&mut w, me.step(Dir::E));
    let profile = Profile {
        to_hit: 0,
        ..w.state.player().melee(w.content())
    };
    let mut hits = Vec::new();
    for _ in 0..5 {
        hits.push(
            w.relentless(player, kobold, AttackKind::Melee, profile.clone())
                .to_hit,
        );
        w.state.turn += 1;
    }
    assert_eq!(hits, [0, 1, 2, 3, 3], "+1 a blow, to +3");
    // Another foe starts it over; so does a turn spent on anything else.
    let other = ActorId(999);
    assert_eq!(
        w.relentless(player, other, AttackKind::Melee, profile.clone())
            .to_hit,
        0
    );
    w.state.turn += 2;
    assert_eq!(
        w.relentless(player, other, AttackKind::Melee, profile.clone())
            .to_hit,
        0
    );
    // A shot is no blow; and without the gift no streak is kept.
    assert_eq!(
        w.relentless(player, other, AttackKind::Ranged, profile.clone())
            .to_hit,
        0
    );
    let mut plain = born("showcase", None);
    let kobold = foe_at(&mut plain, me.step(Dir::E));
    plain.relentless(player, kobold, AttackKind::Melee, profile.clone());
    assert_eq!(plain.state.player().streak, None);
}

#[test]
fn barbed_hide_cuts_whoever_hits_in_melee() {
    let mut w = born("showcase", Some(("wood", "manticore")));
    let me = w.player_pos();
    let player = w.state.player;
    let kobold = foe_at(&mut w, me.step(Dir::E));
    w.perceive(&mut Vec::new());
    let hp = w.state.actors[&kobold].hp;
    let mut events = Vec::new();
    w.barbed(kobold, player, AttackKind::Melee, &mut events);
    assert_eq!(w.state.actors[&kobold].hp, hp - 1);
    assert_eq!(last(&w), "The kobold is cut by your barbs.");
    // A shot is not a blow up close.
    w.barbed(kobold, player, AttackKind::Ranged, &mut events);
    assert_eq!(w.state.actors[&kobold].hp, hp - 1);
    // A death of the barbs earns nothing.
    w.state.actors.get_mut(&kobold).expect("kobold").hp = 1;
    let xp = w.player_xp();
    w.barbed(kobold, player, AttackKind::Melee, &mut events);
    assert!(!w.state.actors.contains_key(&kobold), "it died of them");
    assert_eq!(w.player_xp(), xp);
    // Without the gift, nothing.
    let mut plain = born("showcase", None);
    let kobold = foe_at(&mut plain, me.step(Dir::E));
    let hp = plain.state.actors[&kobold].hp;
    plain.barbed(kobold, player, AttackKind::Melee, &mut events);
    assert_eq!(plain.state.actors[&kobold].hp, hp);
}

#[test]
fn thin_skin_bleeds_a_degree_more_from_a_big_hit() {
    let bleeding = |sign| {
        let mut w = born("showcase", sign);
        let player = w.state.player;
        let max = w.state.player().max_hp(w.content());
        let slashing = w.content().damage_type_id("slashing");
        let blow = Blow {
            strike: Strike::Hit { critical: false },
            damage: max / 2,
            kind: slashing,
        };
        w.wounded(player, &blow, None, &mut Vec::new());
        let id = status(&w, "bleeding");
        w.state.player().effects.get(id).map_or(0, |e| e.severity)
    };
    assert_eq!(bleeding(None), 1);
    assert_eq!(bleeding(Some(("wood", "unicorn"))), 2);
}

#[test]
fn trailblazers_cross_rough_country_as_plains_and_journey_quicker() {
    let price = |sign, terrain: &str| {
        let w = born("world", sign);
        let def = w
            .content()
            .terrain(w.content().terrain_id(terrain).expect("terrain"));
        w.step_price(w.state.player, def, false)
    };
    let wyvern = Some(("wood", "wyvern"));
    for rough in ["forest", "hills", "swamp"] {
        assert_eq!(price(wyvern, rough), price(wyvern, "plains"), "{rough}");
        assert!(price(None, rough) > price(None, "plains"), "{rough}");
    }
    assert_eq!(price(wyvern, "plains"), price(None, "plains") * 80 / 100);
}

#[test]
fn the_damage_type_forms_resist_fire_and_cold() {
    // Flame-hardened (the Wyvern), Kindled (Fire), and Of the deep (the Kraken): 25 % each,
    // summed where a sign has two (a Fire Wyvern), and nothing for the unborn.
    let (fire, cold) = (
        BUILTIN.damage_type_id("fire"),
        BUILTIN.damage_type_id("cold"),
    );
    let resists = |sign| {
        let w = born("showcase", sign);
        let all = w.resistances_of(w.state.player);
        (all.get(fire), all.get(cold))
    };
    assert_eq!(resists(Some(("wood", "wyvern"))), (25, 0));
    assert_eq!(resists(Some(("fire", "salamander"))), (25, 0));
    assert_eq!(resists(Some(("fire", "wyvern"))), (50, 0));
    assert_eq!(resists(Some(("wood", "kraken"))), (0, 25));
    assert_eq!(resists(Some(("fire", "kraken"))), (25, 25));
    assert_eq!(resists(None), (0, 0));
    // And what reaches them: a blast of 8 fire or cold, a quarter less.
    let w = born("showcase", Some(("fire", "kraken")));
    let player = w.state.player;
    assert_eq!(w.resisted(player, Damage::new(8, fire)), 6);
    assert_eq!(w.resisted(player, Damage::new(8, cold)), 6);
    let unborn = born("showcase", None);
    assert_eq!(unborn.resisted(player, Damage::new(8, cold)), 8);
}

#[test]
fn no_one_makes_way_for_one_trusted_by_none() {
    let make_way = |sign| {
        let w = born("showcase", sign);
        let hermit = BUILTIN.creature_id("hermit").expect("the hermit");
        let id = (w.state.actors.values())
            .find(|a| a.creature == hermit)
            .map(|a| a.id)
            .expect("the hub's hermit");
        w.refusal(id)
    };
    assert_eq!(make_way(None), None, "the hermit makes way");
    assert_eq!(
        make_way(Some(("wood", "basilisk"))),
        Some(super::talk::Refusal::Budge)
    );
}

#[test]
fn horns_grace_ends_bleeding_and_poison_then_is_spent_for_a_day() {
    let mut w = born("showcase", Some(("wood", "unicorn")));
    let player = w.state.player;
    assert_eq!(
        own(&w, SignActive::HornsGrace),
        Some(None),
        "listed, usable"
    );
    // Nothing to mend: refused, no time.
    let out = w.apply(&Action::Gift(SignActive::HornsGrace, None));
    assert!(!out.took_turn);
    assert_eq!(last(&w), "There is nothing for your horn's grace to mend.");
    run(&mut w, "afflict:bleeding*2 afflict:poisoned*3");
    assert!(suffers(&w, player, "bleeding") && suffers(&w, player, "poisoned"));
    let out = w.apply(&Action::Gift(SignActive::HornsGrace, None));
    assert!(out.took_turn);
    assert!(!suffers(&w, player, "bleeding") && !suffers(&w, player, "poisoned"));
    assert!(suffers(&w, player, "graceless"));
    let graceless = status(&w, "graceless");
    assert_eq!(
        own(&w, SignActive::HornsGrace),
        Some(Some(Unavailable::While(graceless)))
    );
    run(&mut w, "afflict:bleeding");
    let out = w.apply(&Action::Gift(SignActive::HornsGrace, None));
    assert!(!out.took_turn);
    assert_eq!(last(&w), "You are still graceless.");
    // No other sign has it.
    let plain = born("showcase", None);
    assert_eq!(own(&plain, SignActive::HornsGrace), None);
}

#[test]
fn horns_grace_shakes_off_a_disease_that_shows() {
    let mut w = born("showcase", Some(("wood", "unicorn")));
    run(&mut w, "infect:the_shakes rest:24h*3");
    let shows = |w: &World| w.state.player().diseases.iter().any(|i| i.shown);
    if !shows(&w) {
        run(&mut w, "rest:24h*4");
    }
    assert!(shows(&w), "the disease shows");
    w.apply(&Action::Gift(SignActive::HornsGrace, None));
    assert!(w.state.player().diseases.is_empty(), "shaken off");
}

#[test]
fn the_transfixing_stare_slows_a_foe_that_sees_you() {
    let mut w = born("showcase", Some(("wood", "basilisk")));
    let me = w.player_pos();
    let player = w.state.player;
    let at = Pos::new(me.x + 2, me.y);
    let kobold = foe_at(&mut w, at);
    w.perceive(&mut Vec::new());
    assert_eq!(own(&w, SignActive::TransfixingStare), Some(None));
    assert_eq!(
        Ability::Sign(SignActive::TransfixingStare).range(),
        Some(STARE_RANGE)
    );
    // At yourself, or where you see no one: refused, no time.
    assert!(
        !w.apply(&Action::Gift(SignActive::TransfixingStare, Some(me)))
            .took_turn
    );
    let empty = Pos::new(me.x, me.y + 2);
    assert!(
        !w.apply(&Action::Gift(SignActive::TransfixingStare, Some(empty)))
            .took_turn
    );
    assert_eq!(last(&w), "You see no one there to stare at.");
    let before = w.price_of(kobold, crate::content::ActionKind::Step);
    let out = w.apply(&Action::Gift(SignActive::TransfixingStare, Some(at)));
    assert!(out.took_turn, "{}", last(&w));
    assert!(
        w.messages()
            .iter()
            .any(|m| m.display() == "The kobold freezes under your gaze."),
        "said"
    );
    assert!(suffers(&w, player, "spent_eyed"));
    let transfixed = status(&w, "transfixed");
    let held = w.state.actors[&kobold].effects.get(transfixed).copied();
    assert!(held.is_some(), "transfixed");
    let slowed = w.price_of(kobold, crate::content::ActionKind::Step);
    assert_eq!(slowed, before * 2, "twice as slow");
    // Spent-eyed: no second stare until it passes.
    let out = w.apply(&Action::Gift(SignActive::TransfixingStare, Some(at)));
    assert!(!out.took_turn);
    assert_eq!(last(&w), "You are still spent-eyed.");
}

#[test]
fn the_stare_spares_the_harmless() {
    let mut w = born("showcase", Some(("wood", "basilisk")));
    let me = w.player_pos();
    let player = w.state.player;
    let hermit = BUILTIN.creature_id("hermit").expect("hermit");
    let hermit = (w.state.actors.values())
        .find(|a| a.creature == hermit)
        .map(|a| a.id)
        .expect("hermit");
    let there = Pos::new(me.x, me.y - 2);
    w.state.actors.get_mut(&hermit).expect("hermit").pos = there;
    w.perceive(&mut Vec::new());
    let out = w.apply(&Action::Gift(SignActive::TransfixingStare, Some(there)));
    assert!(!out.took_turn);
    assert_eq!(last(&w), "The old hermit means you no harm.");
    assert!(!suffers(&w, player, "spent_eyed"));
}

#[test]
fn gifts_are_action_text_and_the_menus_birth_sign_group() {
    for text in ["use:horns_grace", "use:transfixing_stare:12,5"] {
        let action: Action = text.parse().expect("parses");
        assert_eq!(action.to_string(), text);
    }
    assert_eq!(
        "use:transfixing_stare:12,5".parse::<Action>(),
        Ok(Action::Gift(
            SignActive::TransfixingStare,
            Some(Pos::new(12, 5))
        ))
    );
    let stare = Ability::Sign(SignActive::TransfixingStare);
    assert_eq!(stare.aim(&BUILTIN), crate::ability::Aim::Cell);
    assert_eq!(stare.source(), crate::ability::Source::Sign);
    assert_eq!(stare.action(&BUILTIN, None), None, "not without a cell");
    let grace = Ability::Sign(SignActive::HornsGrace);
    assert_eq!(grace.aim(&BUILTIN), crate::ability::Aim::Nothing);
    assert_eq!(
        grace.action(&BUILTIN, None),
        Some(Action::Gift(SignActive::HornsGrace, None))
    );
    assert_eq!(grace.name(&BUILTIN), "Horn's grace");
}
