//! The birth signs' picks (spec §8 "Birth sign"; #270): the seventeen sign-limited talents of
//! the `stars` tree are offered only to those born under their sign (and with their other
//! prerequisites), and each changes the one rule it names, the two new actives (Fire breath and
//! Hound's bay) with their cooldowns in the abilities menu.

use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::{AttackKind, parse_actions};
use crate::character::CharacterSpec;
use crate::combat::Profile;
use crate::content::{Sign, SignActive, StatusId};
use crate::equipment;
use crate::geom::Pos;
use crate::stats::{PerStat, Stats};
use crate::talent::{Candidate, Refusal, may_take};
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// The seventeen picks with the sign each asks for (`(animal, element)`, one of them set).
const PICKS: [(&str, Option<&str>, Option<&str>); 17] = [
    ("hare_footed", Some("salamander"), None),
    ("another_head", Some("hydra"), None),
    ("relentless_fury", Some("manticore"), None),
    ("horn_of_plenty", Some("unicorn"), None),
    ("fire_breath", Some("wyvern"), None),
    ("petrifying_gaze", Some("basilisk"), None),
    ("rocs_grip", Some("roc"), None),
    ("dragon_hoard", Some("griffin"), None),
    ("riddle_of_the_sphinx", Some("sphinx"), None),
    ("second_rising", Some("phoenix"), None),
    ("tentacles", Some("kraken"), None),
    ("hounds_bay", Some("barghest"), None),
    ("deep_roots", None, Some("wood")),
    ("wildfire", None, Some("fire")),
    ("bedrock", None, Some("earth")),
    ("tempered", None, Some("metal")),
    ("undertow", None, Some("water")),
];

/// A human warrior in the showcase (seed 7), born under the `element` `animal` on an unlucky
/// day, every score 14 and every hit point there; the run restored, so the gifts are granted
/// as a load grants them.
fn born(animal: &str, element: &str) -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let w =
        World::with_character(BUILTIN.clone(), "showcase", Some(7), Some(&spec)).expect("world");
    let mut state = w.state().clone();
    let character = state.character.as_mut().expect("a character");
    character.sign = Some(Sign {
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

/// `born`, with the pick `id` taken as a level-up takes it (its prerequisites unchecked).
fn with(animal: &str, element: &str, id: &str) -> World {
    let mut w = born(animal, element);
    w.player_mut().talents.take(id, None);
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

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.display() == text)
}

fn status(id: &str) -> StatusId {
    BUILTIN.status_id(id).expect("a status")
}

fn severity(w: &World, who: ActorId, id: &str) -> u16 {
    (w.state.actors[&who].effects.get(status(id))).map_or(0, |e| e.severity)
}

fn mods(w: &World) -> crate::talent::TalentMods {
    w.state.player().talent_mods(w.content()).into_owned()
}

fn own(w: &World, active: SignActive) -> Option<Option<Unavailable>> {
    (w.perception().abilities().into_iter())
        .find(|a| a.ability == Ability::Sign(active))
        .map(|a| a.unavailable)
}

/// A kobold (hostile to the player, level 2) with `hp` hit points placed at `at`; its id.
fn foe_at(w: &mut World, at: Pos, hp: i32) -> ActorId {
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
        hp_base: hp,
        hp,
        level: w.state.current_level.clone(),
        ..template
    };
    w.state.actors.insert(id, actor);
    id
}

/// The player's weapon skill or style `name` raised to `rank`.
fn raise(w: &mut World, name: &str, rank: u8) {
    let table = w.content.weapon_skills().clone();
    let skill = table.id(name).expect(name);
    let player = w.state.player;
    let me = w.state.actors.get_mut(&player).expect("the player");
    me.weapon_skills.raise(&table, skill, rank);
}

/// Whether the player of `w` at character level `level` may take `id` now.
fn may(w: &World, id: &str, level: u8) -> Result<u8, Refusal> {
    let content = w.content();
    let book = content.talents();
    let def = book.get(book.id(id).expect(id));
    let who = Candidate::player(content, w.state.player(), w.character(), level);
    may_take(content, &who, def, None)
}

#[test]
fn every_pick_is_offered_only_under_its_sign() {
    let book = BUILTIN.talents();
    let stars = book.tree_id("stars").expect("the stars tree");
    for (id, animal, element) in PICKS {
        let def = book.get(book.id(id).expect(id));
        assert_eq!(def.tree, stars, "{id} is a Stars talent");
        assert!(!def.granted && !def.flaw, "{id} is picked");
        let (yes, no) = match (animal, element) {
            (Some(animal), _) => {
                let other = if animal == "hydra" { "roc" } else { "hydra" };
                (born(animal, "earth"), born(other, "earth"))
            }
            (None, Some(element)) => {
                let other = if element == "wood" { "earth" } else { "wood" };
                (born("hydra", element), born("hydra", other))
            }
            _ => unreachable!(),
        };
        let mut yes = yes;
        let mut no = no;
        // Every other prerequisite met: high level, and the weapon skills they ask.
        for w in [&mut yes, &mut no] {
            raise(w, "twin_blades", 5);
            raise(w, "long_blades", 5);
        }
        assert_eq!(may(&yes, id, 30), Ok(1), "{id} under its sign");
        assert!(
            matches!(may(&no, id, 30), Err(Refusal::Unmet(_))),
            "{id} under another"
        );
    }
    // And the pick list offers them by the same rule: a Wyvern at level 15 sees Fire breath,
    // a Hydra does not.
    let offered = |w: &World, level: u8| -> Vec<String> {
        let who = Candidate::player(&BUILTIN, w.state.player(), w.character(), level);
        (crate::talent::offers(&BUILTIN, &who).into_iter())
            .map(|(t, _)| book.get(t).id.clone())
            .collect()
    };
    assert!(offered(&born("wyvern", "fire"), 15).contains(&"fire_breath".to_string()));
    assert!(!offered(&born("wyvern", "fire"), 14).contains(&"fire_breath".to_string()));
    assert!(!offered(&born("hydra", "fire"), 15).contains(&"fire_breath".to_string()));
    // The refusal says the sign in words.
    let words = may(&born("hydra", "fire"), "fire_breath", 15)
        .expect_err("not a wyvern")
        .words(&BUILTIN, "Fire breath");
    assert_eq!(
        words,
        "You cannot take Fire breath: you need to be born in the year of the Wyvern."
    );
}

#[test]
fn hare_footed_is_quicker_still() {
    let fleet = born("salamander", "fire");
    let hare = with("salamander", "fire", "hare_footed");
    let base = born("hydra", "fire").state.player().speed(&BUILTIN);
    assert_eq!(fleet.state.player().speed(&BUILTIN), base * 105 / 100);
    assert_eq!(hare.state.player().speed(&BUILTIN), base * 110 / 100);
}

#[test]
fn another_head_heals_while_starving_and_twice_as_fast_resting() {
    let mut plain = born("hydra", "fire");
    let mut heads = with("hydra", "fire", "another_head");
    let id = plain.player_id();
    let fed = plain.healing_rate(id);
    assert_eq!(heads.healing_rate(id), fed);
    for w in [&mut plain, &mut heads] {
        w.player_mut().satiation = Some(crate::hunger::Satiation::new(-10));
        assert_eq!(
            w.state.player().hunger(&BUILTIN),
            Some(crate::hunger::Hunger::Starving)
        );
    }
    assert_eq!(plain.healing_rate(id), 0, "starving, no healing");
    assert_eq!(heads.healing_rate(id), fed, "the heads grow back");
    assert_eq!(mods(&heads).resting_percent, 100);
}

#[test]
fn relentless_fury_runs_to_five_and_bites_harder() {
    let mut fury = with("manticore", "fire", "relentless_fury");
    let mut plain = born("manticore", "fire");
    let me = fury.player_id();
    let at = fury.player_pos().step(Dir::E);
    let foe = foe_at(&mut fury, at, 500);
    let foe2 = foe_at(&mut plain, at, 500);
    let base: Profile = fury.state.player().melee(&BUILTIN);
    let streak = |w: &mut World, target: ActorId| -> Vec<(i32, i32)> {
        (0..7)
            .map(|_| {
                let p = w.relentless(me, target, AttackKind::Melee, base.clone());
                (p.to_hit - base.to_hit, p.bonus - base.bonus)
            })
            .collect()
    };
    assert_eq!(
        streak(&mut fury, foe),
        [(0, 0), (1, 0), (2, 1), (3, 1), (4, 2), (5, 2), (5, 2)]
    );
    assert_eq!(
        streak(&mut plain, foe2),
        [(0, 0), (1, 0), (2, 0), (3, 0), (3, 0), (3, 0), (3, 0)]
    );
}

#[test]
fn horn_of_plenty_heals_and_rests_eight_hours() {
    let mut w = with("unicorn", "fire", "horn_of_plenty");
    let player = w.state.player;
    // Whole and clean: nothing to mend.
    assert!(
        !w.apply(&Action::Gift(SignActive::HornsGrace, None))
            .took_turn
    );
    // Wounded only: the horn heals a hit point per level, then rests one degree (eight hours).
    w.player_mut().hp = 1;
    let level = i32::from(w.player_level());
    let out = w.apply(&Action::Gift(SignActive::HornsGrace, None));
    assert!(out.took_turn, "{}", last(&w));
    assert!(w.state.player().hp > level, "healed");
    assert_eq!(severity(&w, player, "graceless"), 1);
    // Without it, the grace rests the whole day (three degrees) and a wound alone is no cause.
    let mut plain = born("unicorn", "fire");
    plain.player_mut().hp = 1;
    assert!(
        !plain
            .apply(&Action::Gift(SignActive::HornsGrace, None))
            .took_turn
    );
    run(&mut plain, "afflict:bleeding");
    plain.apply(&Action::Gift(SignActive::HornsGrace, None));
    assert_eq!(severity(&plain, player, "graceless"), 3);
}

#[test]
fn fire_breath_burns_along_its_line_then_drains() {
    let mut w = with("wyvern", "fire", "fire_breath");
    let player = w.state.player;
    let me = w.player_pos();
    let near = foe_at(&mut w, Pos::new(me.x + 2, me.y), 500);
    let far = foe_at(&mut w, Pos::new(me.x + 6, me.y), 500);
    w.perceive(&mut Vec::new());
    assert_eq!(
        own(&w, SignActive::FireBreath),
        Some(None),
        "listed, usable"
    );
    assert_eq!(
        Ability::Sign(SignActive::FireBreath).range(),
        Some(BREATH_RANGE)
    );
    // At yourself or out of range: refused, no time.
    let breathe = |at: Pos| Action::Gift(SignActive::FireBreath, Some(at));
    assert!(!w.apply(&breathe(me)).took_turn);
    assert!(!w.apply(&breathe(Pos::new(me.x + 20, me.y))).took_turn);
    assert_eq!(last(&w), "That is out of range.");
    let out = w.apply(&breathe(Pos::new(me.x + 1, me.y)));
    assert!(out.took_turn, "{}", last(&w));
    assert!(said(&w, "Fire roars from your throat!"));
    assert!(w.state.actors[&near].hp < 500, "on the line, burned");
    assert_eq!(w.state.actors[&far].hp, 500, "past its four cells, spared");
    assert_eq!(severity(&w, player, "drained"), 2);
    assert_eq!(
        own(&w, SignActive::FireBreath),
        Some(Some(Unavailable::While(status("drained"))))
    );
    let out = w.apply(&breathe(Pos::new(me.x + 1, me.y)));
    assert!(!out.took_turn);
    assert_eq!(last(&w), "You are still drained.");
    // No one else has it.
    assert_eq!(own(&born("wyvern", "fire"), SignActive::FireBreath), None);
}

#[test]
fn petrifying_gaze_holds_longer_and_rests_sooner() {
    let mut w = with("basilisk", "fire", "petrifying_gaze");
    let player = w.state.player;
    let me = w.player_pos();
    let at = Pos::new(me.x + 2, me.y);
    let kobold = foe_at(&mut w, at, 50);
    w.perceive(&mut Vec::new());
    let stare = Action::Gift(SignActive::TransfixingStare, Some(at));
    assert!(w.apply(&stare).took_turn, "{}", last(&w));
    assert_eq!(severity(&w, kobold, "transfixed"), 4);
    assert_eq!(severity(&w, player, "spent_eyed"), 1);
    let mut plain = born("basilisk", "fire");
    let kobold = foe_at(&mut plain, at, 50);
    plain.perceive(&mut Vec::new());
    assert!(plain.apply(&stare).took_turn, "{}", last(&plain));
    assert_eq!(severity(&plain, kobold, "transfixed"), 2);
    assert_eq!(severity(&plain, player, "spent_eyed"), 2);
}

#[test]
fn rocs_grip_carries_more_and_shrugs_off_burdened() {
    let limit = |w: &World| w.actor_load(w.player_id()).expect("load").limit;
    let roc = born("roc", "fire");
    let grip = with("roc", "fire", "rocs_grip");
    assert!(limit(&grip) > limit(&roc));
    assert!(mods(&grip).ignore_burdened && !mods(&roc).ignore_burdened);
}

#[test]
fn the_hoard_and_the_riddle_add_to_their_gifts() {
    let griffin = mods(&born("griffin", "fire"));
    let hoard = mods(&with("griffin", "fire", "dragon_hoard"));
    assert_eq!(
        (hoard.loot_magic_percent, hoard.gem_find_percent),
        (
            griffin.loot_magic_percent * 3 / 2,
            griffin.gem_find_percent * 3 / 2
        ),
        "Hoard sense's odds half again"
    );
    assert_eq!(hoard.favour_percent, griffin.favour_percent + 10);
    let sphinx = mods(&born("sphinx", "fire"));
    let riddle = mods(&with("sphinx", "fire", "riddle_of_the_sphinx"));
    assert_eq!(riddle.skill_rolls, sphinx.skill_rolls + 1);
}

#[test]
fn second_rising_lets_the_phoenix_rise_twice_and_no_more() {
    let mut w = with("phoenix", "fire", "second_rising");
    let player = w.state.player;
    let at = w.player_pos().step(Dir::E);
    let kobold = foe_at(&mut w, at, 500);
    let mut events = Vec::new();
    let blow = fight::Harm::Actor(kobold);
    assert!(!w.hurt(player, 999, blow, &mut events), "rises once");
    assert!(w.state.progress.happened(RISEN));
    assert!(!w.state.progress.happened(RISEN_TWICE));
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Talents);
    assert!(
        rows.contains(&"Rise from the ashes (sign)".to_string()),
        "one rising left: {rows:?}"
    );
    assert!(!w.hurt(player, 999, blow, &mut events), "rises twice");
    assert!(w.state.progress.happened(RISEN_TWICE));
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Talents);
    assert!(
        rows.contains(&"Rise from the ashes (sign, spent)".to_string()),
        "{rows:?}"
    );
    assert!(w.hurt(player, 999, blow, &mut events), "and then dies");
    assert!(w.state.death.is_some());
}

#[test]
fn tentacles_strike_harder_with_the_off_hand() {
    let mut w = born("kraken", "fire");
    let content = w.content().clone();
    let player = w.state.player;
    let me = w.state.actors.get_mut(&player).expect("the player");
    me.equipment = Default::default();
    me.inventory.clear();
    for name in ["long_sword", "dagger"] {
        let object = content.object_id(name).expect(name);
        me.inventory.push(crate::object::Stack::new(object, 1));
        let item = me.inventory.len() - 1;
        let slot = equipment::free_slot(&content, &me.equipment, object).expect("a slot");
        let slots = equipment::slots_taken(&content, object, slot);
        me.equipment.put(&slots, item);
    }
    let (_, plain) = w
        .state
        .player()
        .off_hand_blow(&content)
        .expect("an off hand");
    let main = w.state.player().melee(&content);
    w.player_mut().talents.take("tentacles", None);
    let (_, kraken) = w
        .state
        .player()
        .off_hand_blow(&content)
        .expect("an off hand");
    assert_eq!(kraken.bonus, plain.bonus + 2);
    assert_eq!(
        w.state.player().melee(&content),
        main,
        "the main hand as it was"
    );
}

#[test]
fn hounds_bay_sends_lesser_foes_fleeing_then_leaves_you_hoarse() {
    let mut w = with("barghest", "fire", "hounds_bay");
    let player = w.state.player;
    let me = w.player_pos();
    let at = Pos::new(me.x + 2, me.y);
    let kobold = foe_at(&mut w, at, 50);
    w.perceive(&mut Vec::new());
    assert_eq!(own(&w, SignActive::HoundsBay), Some(None), "listed, usable");
    // At level 1 a kobold (level 2) is no lesser foe: it hears and stands.
    let out = w.apply(&Action::Gift(SignActive::HoundsBay, None));
    assert!(out.took_turn);
    assert!(said(&w, "You throw back your head and howl!"));
    assert_eq!(severity(&w, kobold, "frightened"), 0);
    assert_eq!(severity(&w, player, "hoarse"), 1);
    assert_eq!(
        own(&w, SignActive::HoundsBay),
        Some(Some(Unavailable::While(status("hoarse"))))
    );
    // Rested and grown past it: it cowers, and runs from the player.
    let mut w = with("barghest", "fire", "hounds_bay");
    run(&mut w, "xp:20000 levelup:auto");
    assert!(w.player_level() > 2);
    let kobold = foe_at(&mut w, at, 50);
    w.perceive(&mut Vec::new());
    let before = w.state.actors[&kobold].pos.chebyshev(me);
    let out = w.apply(&Action::Gift(SignActive::HoundsBay, None));
    assert!(out.took_turn, "{}", last(&w));
    assert!(said(&w, "The kobold cowers and flees from you."));
    let fear = w.state.actors[&kobold].effects.get(status("frightened"));
    assert_eq!(
        fear.and_then(|e| e.by),
        Some(player),
        "caused by the player"
    );
    run(&mut w, "wait");
    let after = w.state.actors[&kobold].pos.chebyshev(w.player_pos());
    assert!(after > before, "it ran: {before} -> {after}");
    // Its fear passes in a few turns.
    run(&mut w, "wait*5");
    assert_eq!(severity(&w, kobold, "frightened"), 0);
    // The howl is heard: a loud noise.
    assert_eq!(
        crate::noise::SoundKind::Howl.heard_as(),
        "a howl",
        "the player hears others' howls by this name"
    );
}

#[test]
fn deep_roots_eats_rot_unharmed_drawing_nothing() {
    let content = BUILTIN.clone();
    let bread = content.object_id("bread").expect("bread");
    let mut rooted = with("hydra", "wood", "deep_roots");
    let rng = rooted.state.rng.clone();
    let mut events = Vec::new();
    rooted.eat_rotten(100, bread, &mut events);
    assert_eq!(rooted.state.rng, rng, "no draw");
    assert_eq!(
        last(&rooted),
        "The loaf of bread was rotten, but your stomach does not mind."
    );
    let mut plain = born("hydra", "wood");
    plain.eat_rotten(100, bread, &mut events);
    assert!(said(&plain, "Ugh! The loaf of bread was rotten."));
    assert_eq!(mods(&rooted).dodge, mods(&plain).dodge + 1);
}

#[test]
fn wildfire_blazes_farther() {
    let mut fire = born("hydra", "fire");
    let mut wild = with("hydra", "fire", "wildfire");
    let me = fire.player_pos();
    let two = Pos::new(me.x + 2, me.y);
    let plain_foe = foe_at(&mut fire, two, 500);
    let wild_foe = foe_at(&mut wild, two, 500);
    let blaze = Action::Gift(SignActive::Blaze, None);
    assert!(fire.apply(&blaze).took_turn && wild.apply(&blaze).took_turn);
    assert_eq!(
        fire.state.actors[&plain_foe].hp, 500,
        "two cells off, spared"
    );
    assert!(
        wild.state.actors[&wild_foe].hp < 500,
        "the wildfire reaches it"
    );
}

#[test]
fn bedrock_soaks_and_spares_pits_undertow_wades() {
    let earth = mods(&born("hydra", "earth"));
    let bedrock = mods(&with("hydra", "earth", "bedrock"));
    assert_eq!(bedrock.protection.len(), earth.protection.len() + 1);
    assert!(bedrock.pit_proof && !earth.pit_proof);
    let plain = born("manticore", "water");
    let under = with("manticore", "water", "undertow");
    let water = BUILTIN.terrain(BUILTIN.terrain_id("water").expect("water"));
    let floor = BUILTIN.terrain(BUILTIN.terrain_id("floor").expect("floor"));
    let id = plain.player_id();
    let dry = plain.step_price(id, floor, false);
    assert!(plain.step_price(id, water, false) > dry);
    assert_eq!(under.step_price(id, water, false), dry);
    assert!(mods(&under).plunge_proof && !mods(&under).swimmer);
}

#[test]
fn tempered_threatens_lower_with_the_best_skill_only() {
    let mut w = born("hydra", "metal");
    let content = w.content().clone();
    let table = content.weapon_skills();
    let long = table.id("long_blades").expect("long blades");
    let axes = table.id("axes").expect("axes");
    raise(&mut w, "long_blades", 5);
    raise(&mut w, "axes", 3);
    let before = w.state.player().benefit(&content, Some(long)).threat;
    let axes_before = w.state.player().benefit(&content, Some(axes)).threat;
    w.player_mut().talents.take("tempered", None);
    let me = w.state.player();
    assert_eq!(me.benefit(&content, Some(long)).threat, before + 1);
    assert_eq!(me.benefit(&content, Some(axes)).threat, axes_before);
    assert!(me.knows_best(&content, long) && !me.knows_best(&content, axes));
}
