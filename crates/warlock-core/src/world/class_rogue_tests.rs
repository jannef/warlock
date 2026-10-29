//! Class abilities in the world (spec §8 "Class abilities"; #244): the ranger's, thief's, and
//! bard's level tables, and the kill tally of every creature type that Favoured prey reads. A
//! level brings its abilities, the bonus picks come from the class's trees, each passive changes
//! the rule it names, and the actives (Volley, Inspire) are used from the abilities menu.

use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::combat::{Blow, Strike};
use crate::content::{ClassActive, Factor};
use crate::dump::{self, Layer};
use crate::noise::{self, Act};
use crate::replay::Replay;
use crate::skills::Effect;
use crate::stats::{PerStat, Stats};
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// A human of `class` in the showcase (seed 7), with their kit, born under no sign, every score
/// 14.
fn world(class: &str) -> World {
    let spec: CharacterSpec = format!("human,male,{class}").parse().expect("a character");
    // Creation picks that no test here reads (a human's Versatile adds one).
    let mut picks = vec!["keen_ears".into(), "lean".into(), "healthy".into()];
    if class == "bard" {
        picks.push("tunneler".into());
    }
    let spec = spec.with_talents(picks);
    let w = World::start_run(
        BUILTIN.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let mut state = w.state().clone();
    state.character.as_mut().expect("a character").sign = None;
    let mut w = World::from_state(BUILTIN.clone(), state).expect("restores");
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = max;
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

/// Grants the XP that takes the player to `level` and spends every choice it brings.
fn reach(world: &mut World, level: u8) {
    let spec = world.state.character.as_ref().map(|c| &c.spec);
    let factor = Factor::of(world.content(), spec);
    let need = world.content().experience().threshold_for(level, factor);
    let more = need - world.player_xp();
    run(world, &format!("xp:{more}"));
    assert_eq!(world.player_level(), level, "reached level {level}");
    if world.due().is_some() {
        run(world, "levelup:auto");
    }
    assert_eq!(world.due(), None, "every choice spent");
}

/// The names of the class abilities the player holds, in the order given.
fn class_rows(world: &World) -> Vec<String> {
    (dump::layer_rows(world, Layer::Talents).into_iter())
        .filter_map(|row| row.strip_suffix(" (class)").map(str::to_string))
        .collect()
}

fn own(world: &World, ability: Ability) -> Option<Option<Unavailable>> {
    (world.perception().abilities().into_iter())
        .find(|a| a.ability == ability)
        .map(|a| a.unavailable)
}

fn suffers(world: &World, id: &str) -> bool {
    let status = world.content().status_id(id).expect("a status");
    world.state.player().effects.has(status)
}

/// A creature of type `creature` placed `dir` of the player, with `hp` hit points; its id.
fn beside(world: &mut World, creature: &str, hp: i32, dir: Dir) -> ActorId {
    let content = world.content().clone();
    let at = world.player_pos().step(dir);
    let template = (world.state.actors.values())
        .find(|a| a.id != world.state.player)
        .cloned()
        .expect("a creature in the showcase");
    let id = ActorId(world.state.next_actor_id);
    world.state.next_actor_id += 1;
    let creature = content.creature_id(creature).expect("a creature type");
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
        level: world.state.current_level.clone(),
        ..template
    };
    world.state.actors.insert(id, actor);
    id
}

#[test]
fn the_three_tables_come_with_the_levels() {
    let mut w = world("ranger");
    assert_eq!(class_rows(&w), ["Favoured prey", "Trained wind"]);
    reach(&mut w, 60);
    assert_eq!(
        class_rows(&w),
        [
            "Favoured prey",
            "Trained wind",
            "Tracker",
            "Barkskin",
            "Rapid shot",
            "Woodland stride",
            "Swift tracker",
            "Evasion",
            "Camouflage",
            "Volley",
            "Master hunter"
        ]
    );
    let mut t = world("thief");
    assert_eq!(class_rows(&t), ["Sneak attack", "Trained wind"]);
    reach(&mut t, 18);
    assert!(said(&t).contains(&"You gain Evasion.".to_string()));
    let mut b = world("bard");
    assert_eq!(
        class_rows(&b),
        ["Lore", "Mend", "Trained wind"],
        "the Scholar's Lore, given"
    );
    reach(&mut b, 6);
    assert_eq!(
        class_rows(&b),
        ["Lore", "Mend", "Trained wind", "Wide learning", "Blessing"]
    );
    let restored = World::from_state(b.content().clone(), b.state().clone()).expect("restored");
    assert_eq!(class_rows(&restored), class_rows(&b));
}

#[test]
fn the_thiefs_bonus_picks_start_at_four_from_shadow_finesse_and_delver() {
    let mut w = world("thief");
    reach(&mut w, 3);
    let need = (w.content().experience()).threshold_for(
        4,
        Factor::of(w.content(), w.state.character.as_ref().map(|c| &c.spec)),
    );
    let more = need - w.player_xp();
    run(&mut w, &format!("xp:{more}"));
    let due = w.due().expect("a pick due");
    assert_eq!((due.level, due.bonus), (4, 1));
    run(&mut w, "talent:hardy");
    assert_eq!(
        last(&w),
        "You must choose a talent of Finesse, Shadow, or Delver."
    );
}

#[test]
fn every_kill_is_tallied_and_the_most_killed_type_is_the_prey() {
    let mut w = world("ranger");
    let content = w.content().clone();
    let id = |name: &str| content.creature_id(name).expect("a creature type");
    assert_eq!(w.state.quests.prey(), None);
    w.tally_kill(id("kobold"), "hub");
    assert_eq!(w.state.quests.prey(), Some("kobold"));
    w.tally_kill(id("rat"), "depths-1");
    assert_eq!(
        w.state.quests.prey(),
        Some("kobold"),
        "a tie: the first there"
    );
    w.tally_kill(id("rat"), "hub");
    assert_eq!(w.state.quests.prey(), Some("rat"));
    assert_eq!(w.state.quests.killed("rat"), 2);
    // Saved and restored, and in the hash.
    let restored = World::from_state(w.content().clone(), w.state().clone()).expect("restored");
    assert_eq!(restored.state.quests.prey(), Some("rat"));
}

#[test]
fn favoured_prey_hits_its_prey_surer_and_harder() {
    let mut w = world("ranger");
    let content = w.content().clone();
    let player = w.state.player;
    let rat = beside(&mut w, "rat", 50, Dir::E);
    let kobold = beside(&mut w, "kobold", 50, Dir::W);
    let profile = w.state.player().melee(&content);
    assert_eq!(
        w.favoured(player, rat, profile.clone()),
        profile,
        "no kills yet"
    );
    w.tally_kill(content.creature_id("rat").expect("rat"), "hub");
    let favoured = w.favoured(player, rat, profile.clone());
    assert_eq!(
        (favoured.to_hit, favoured.bonus),
        (profile.to_hit + 1, profile.bonus + 2)
    );
    assert_eq!(w.favoured(player, kobold, profile.clone()), profile);
    // Master hunter doubles it, and bows threaten sooner.
    reach(&mut w, 60);
    let profile = w.state.player().melee(&content);
    let favoured = w.favoured(player, rat, profile.clone());
    assert_eq!(
        (favoured.to_hit, favoured.bonus),
        (profile.to_hit + 2, profile.bonus + 4)
    );
}

#[test]
fn tracker_hears_fainter_and_places_sounds_exactly() {
    let mut w = world("ranger");
    let content = w.content().clone();
    let before = w.state.player().hearing(&content).expect("hears");
    reach(&mut w, 6);
    let mods = w.state.player().talent_mods(&content);
    assert!(mods.exact_sounds);
    assert_eq!(w.state.player().hearing(&content), Some(before - 3));
}

#[test]
fn rapid_shot_woodland_stride_and_swift_tracker() {
    let mut w = world("ranger");
    let content = w.content().clone();
    let player = w.state.player;
    let terrain = |id: &str| content.terrain(content.terrain_id(id).expect("terrain"));
    let focus = content.resource_id("focus").expect("focus");
    assert!(w.state.player().resources.get(focus).is_none());
    reach(&mut w, 12);
    // Rapid shot grants Focus (its quicker shots are `focus_tests.rs`'s).
    let held = w.state.player().resources.get(focus).map(|p| p.value);
    assert_eq!(held, Some(10), "Rapid shot: Focus, full");
    let grass = w.step_price(player, terrain("grass"), false);
    assert!(w.step_price(player, terrain("bush"), false) > grass);
    reach(&mut w, 18);
    for id in ["forest_floor", "bush", "reeds", "tall_grass", "marsh"] {
        assert_eq!(w.step_price(player, terrain(id), false), grass, "{id}");
    }
    // The bonus picks of Wayfarer may have quickened the journeys too.
    reach(&mut w, 24);
    let before = w.state.player().talent_mods(&content).travel_percent;
    reach(&mut w, 25);
    let mods = w.state.player().talent_mods(&content);
    assert_eq!(mods.travel_percent, before - 20, "Swift tracker");
}

#[test]
fn camouflage_and_hide_in_plain_sight_quiet_the_steps() {
    let step = |w: &World, burden| {
        let to = w.player_pos().step(Dir::E);
        let a = w.state.player();
        noise::noise(w.content(), w.state.current(), a, burden, Act::Step(to))
            .map_or(0, |n| n.loudness)
    };
    let attack = |w: &World| {
        let a = w.state.player();
        let act = Act::Attack(crate::action::AttackKind::Melee);
        noise::noise(w.content(), w.state.current(), a, Burden::Unburdened, act)
            .map_or(0, |n| n.loudness)
    };
    // The showcase is underground: Camouflage changes nothing there.
    let mut r = world("ranger");
    let below = step(&r, Burden::Unburdened);
    reach(&mut r, 40);
    assert_eq!(step(&r, Burden::Unburdened), below, "no sky");
    r.state.current_mut().kind = crate::content::LevelKind::Outdoor;
    let sky = step(&r, Burden::Unburdened);
    assert!(sky < below && sky > 0, "{sky} < {below}");
    // Hide in plain sight: silent while Unburdened, quieter attacks.
    let mut t = world("thief");
    let loud = attack(&t);
    reach(&mut t, 50);
    assert_eq!(step(&t, Burden::Unburdened), 0);
    assert!(step(&t, Burden::Burdened) > 0);
    assert_eq!(attack(&t), loud / 2);
}

#[test]
fn sneak_attacks_add_up_and_master_thief_threatens_sooner() {
    let mut w = world("thief");
    let content = w.content().clone();
    let player = w.state.player;
    // At level 1 the dice are the Sneak attack skill's alone (the class passive has none).
    let mods = w.state.player().talent_mods(&content);
    assert_eq!(mods.ambush, None);
    assert_eq!(mods.surprise_difficulty, 2);
    let skill = |w: &World| (w.state.player()).skill_effect(w.content(), Effect::SneakDice);
    assert!(skill(&w) >= 1, "the thief is trained");
    let dice = w.sneak_dice(player).map(|d| d.to_string());
    assert_eq!(dice, Some(format!("{}d6", skill(&w))));
    // Improved sneak attack's 2d6 add to the skill's.
    reach(&mut w, 32);
    let mods = w.state.player().talent_mods(&content);
    assert_eq!(mods.ambush.map(|d| d.to_string()), Some("2d6".into()));
    let dice = w.sneak_dice(player).map(|d| d.to_string());
    assert_eq!(dice, Some(format!("{}d6", skill(&w) + 2)));
    let profile = w.state.player().melee(&content);
    assert_eq!(w.sneak_threat(player, true, profile.clone()), profile);
    reach(&mut w, 60);
    let profile = w.state.player().melee(&content);
    let sneaking = w.sneak_threat(player, true, profile.clone());
    assert_eq!(sneaking.threat, profile.threat - 2);
    assert_eq!(w.sneak_threat(player, false, profile.clone()), profile);
    // An unseen hit draws the dice; a seen one does not.
    let hit = Blow {
        strike: Strike::Hit { critical: false },
        damage: 2,
        kind: None,
    };
    let rng = w.state.rng.clone();
    assert_eq!(w.ambushed(player, false, hit), hit);
    assert_eq!(w.state.rng, rng);
    let n = skill(&w) + 2;
    let damage = w.ambushed(player, true, hit).damage;
    assert!((2 + n..=2 + 6 * n).contains(&damage), "{damage} for {n}d6");
}

#[test]
fn the_thiefs_skills_give_disarm_pick_lock_and_steal_from_the_first_level() {
    // Trapfinding, Pick lock, and Light fingers are gone (#684): the class's skills give what
    // they gave, Detect traps and Disable device known for.
    let mut w = world("thief");
    let content = w.content().clone();
    for id in ["trapfinding", "pick_lock", "light_fingers"] {
        assert!(content.talents().id(id).is_none(), "{id} is gone");
    }
    let known = &content.class("thief").expect("the thief").known_for;
    for skill in ["detect_traps", "disable_device"] {
        let id = content.skill_id(skill).expect("a skill");
        assert!(known.contains(&id), "{skill} known for");
    }
    assert_eq!(own(&w, Ability::Disarm), Some(None));
    assert_eq!(own(&w, Ability::PickLock), Some(None));
    if content.steal().is_some() {
        assert_eq!(own(&w, Ability::Steal), Some(None));
    }
    reach(&mut w, 60);
    assert_eq!(
        class_rows(&w),
        [
            "Sneak attack",
            "Trained wind",
            "Lurk",
            "Low blow",
            "Evasion",
            "Uncanny dodge",
            "Improved sneak attack",
            "Vanishing act",
            "Hide in plain sight",
            "Master thief"
        ]
    );
}

#[test]
fn a_save_holding_a_talent_the_content_no_longer_has_loads_without_it() {
    let w = world("thief");
    let mut state = w.state().clone();
    let player = state.player;
    let me = state.actors.get_mut(&player).expect("the player");
    let mut talents: Vec<crate::talent::Taken> = me.talents.iter().cloned().collect();
    talents.push(crate::talent::Taken {
        id: "trapfinding".into(),
        rank: 1,
        choice: None,
    });
    me.talents =
        serde_json::from_value(serde_json::to_value(&talents).expect("json")).expect("talents");
    let back = World::from_state(BUILTIN.clone(), state).expect("loads");
    assert!(!back.state.player().talents.has("trapfinding", 1));
    assert_eq!(class_rows(&back), class_rows(&w));
    assert!(
        (back.retiered().iter()).any(|l| l.contains("talent `trapfinding`")),
        "{:?}",
        back.retiered()
    );
}

/// Moves creature `id` to `to` as a step and lets whoever it steps up beside answer
/// ([`World::step_up`]); what the step brought.
fn step_to(w: &mut World, id: ActorId, to: Pos) -> Vec<Event> {
    let from = w.state.actors[&id].pos;
    w.state.actors.get_mut(&id).expect("the creature").pos = to;
    let mut events = Vec::new();
    w.step_up(id, from, &mut events);
    events
}

fn struck(events: &[Event], by: ActorId, at: ActorId) -> bool {
    (events.iter()).any(|e| {
        matches!(e, Event::Attacked { attacker, target, .. } if (*attacker, *target) == (by, at))
    })
}

#[test]
fn lurking_a_hostile_stepping_up_takes_the_thiefs_blow_unawares_unless_it_hunts_them() {
    let mut w = world("thief");
    let player = w.state.player;
    let lurk = Ability::Class(ClassActive::Lurk);
    assert_eq!(own(&w, lurk), None, "from level 6");
    reach(&mut w, 6);
    assert_eq!(own(&w, lurk), Some(None));
    let here = w.player_pos();
    let far = here.step(Dir::E).step(Dir::E);
    let kobold = beside(&mut w, "kobold", 500, Dir::E);
    w.state.actors.get_mut(&kobold).expect("kobold").pos = far;
    // Nothing without the lurk: the thief is no gatekeeper.
    assert!(!struck(
        &step_to(&mut w, kobold, here.step(Dir::E)),
        player,
        kobold
    ));
    // Too late with a hostile beside: refused, no time.
    let turn = w.state.turn;
    run(&mut w, "use:lurk");
    assert_eq!(last(&w), "Too late to lurk.");
    assert_eq!(w.state.turn, turn);
    assert!(!suffers(&w, "lurking"));
    // From two cells off: a wait in ambush (the kobold comes on meanwhile; it is put back).
    let ready = |w: &mut World, hunting: bool| {
        let at = w.player_pos();
        let k = w.state.actors.get_mut(&kobold).expect("kobold");
        k.pos = far;
        k.effects = Default::default();
        k.mind = Default::default();
        k.mind.last_seen = hunting.then_some(at);
        w.news.clear();
    };
    ready(&mut w, false);
    run(&mut w, "use:lurk");
    assert_eq!(w.state.turn, turn + 1, "a wait's time");
    assert!(said(&w).contains(&"You wait in ambush.".to_string()));
    assert!(suffers(&w, "lurking"));
    // A kobold that sees the thief but hunts nobody steps up: struck at once, unawares (the
    // initiative check said one way or the other).
    ready(&mut w, false);
    let events = step_to(&mut w, kobold, here.step(Dir::E));
    assert!(struck(&events, player, kobold));
    let caught = |w: &World| {
        (w.news.iter()).any(|m| m.contains("unawares") || m.contains("whirls to face you"))
    };
    assert!(caught(&w), "{:?}", w.news);
    // One that hunts them is struck, but not caught unawares.
    ready(&mut w, true);
    let events = step_to(&mut w, kobold, here.step(Dir::E));
    assert!(struck(&events, player, kobold));
    assert!(!caught(&w), "{:?}", w.news);
    assert_eq!(w.technique, super::weapon_active::Technique::default());
    // The next action ends it.
    ready(&mut w, false);
    w.state.actors.get_mut(&kobold).expect("kobold").pos = far.step(Dir::E);
    run(&mut w, "wait");
    assert!(!suffers(&w, "lurking"));
}

#[test]
fn a_low_blow_is_a_sneak_attack_for_three_openings_whatever_the_foe_knows() {
    let mut w = world("thief");
    let content = w.content().clone();
    let player = w.state.player;
    let openings = content.resource_id("openings").expect("openings");
    let held = |w: &World| (w.state.player().resources.get(openings)).map_or(0, |p| p.value);
    let low = Ability::Class(ClassActive::LowBlow);
    reach(&mut w, 12);
    w.player_mut().talents.forget("patient_blade");
    w.player_mut().resources.set(openings, 0);
    assert_eq!(own(&w, low), Some(Some(Unavailable::Short(openings))));
    let turn = w.state.turn;
    run(&mut w, "use:low_blow:E");
    assert_eq!(last(&w), "You have too little openings.");
    // Nobody there: refused, nothing spent.
    w.player_mut().resources.set(openings, 5);
    run(&mut w, "use:low_blow:E");
    assert_eq!(last(&w), "There is no one there to blind.");
    assert_eq!((w.state.turn, held(&w)), (turn, 5));
    // A kobold that sees the thief: each low blow spends three, and one that hits is a sneak
    // attack (it builds an opening back) on a foe that is not Surprised.
    let kobold = beside(&mut w, "kobold", 5000, Dir::E);
    let surprised = content.status_id("surprised").expect("surprised");
    let mut hit = false;
    for _ in 0..30 {
        w.player_mut().resources.set(openings, 5);
        let out = w.apply(&Action::Use(ClassActive::LowBlow, Some(Dir::E)));
        assert!(out.took_turn);
        let landed = (out.events.iter()).any(|e| {
            matches!(e, Event::Attacked { attacker, target, strike: Some(s), .. }
                if (*attacker, *target) == (player, kobold) && s.is_hit())
        });
        assert!(!w.state.actors[&kobold].effects.has(surprised));
        if landed {
            assert_eq!(held(&w), 3, "spent three, built one");
            hit = true;
            break;
        }
        assert_eq!(held(&w), 2, "spent three");
    }
    assert!(hit, "one of thirty low blows hit");
    assert_eq!(w.technique, super::weapon_active::Technique::default());
    // A skeleton has no vitals: the blow lands without dice, and the thief is told.
    w.state.actors.remove(&kobold);
    let skeleton = beside(&mut w, "skeleton", 5000, Dir::E);
    for _ in 0..30 {
        w.player_mut().resources.set(openings, 5);
        let out = w.apply(&Action::Use(ClassActive::LowBlow, Some(Dir::E)));
        let landed = (out.events.iter()).any(|e| {
            matches!(e, Event::Attacked { target, strike: Some(s), .. }
                if *target == skeleton && s.is_hit())
        });
        if landed {
            assert_eq!(held(&w), 2, "no sneak attack, nothing built");
            assert!(said(&w).contains(&"It has no vital spot.".to_string()));
            return;
        }
    }
    panic!("no low blow hit the skeleton");
}

#[test]
fn vanishing_act_a_sneak_attack_that_kills_leaves_the_thief_light_footed() {
    let mut w = world("thief");
    let content = w.content().clone();
    let openings = content.resource_id("openings").expect("openings");
    reach(&mut w, 40);
    let mods = w.state.player().talent_mods(&content);
    assert_eq!(mods.sneak_kill_status, content.status_id("light_footed"));
    let kobold = beside(&mut w, "kobold", 1, Dir::E);
    for _ in 0..30 {
        assert!(!suffers(&w, "light_footed"));
        w.player_mut().resources.set(openings, 5);
        run(&mut w, "use:low_blow:E");
        if !w.state.actors.contains_key(&kobold) {
            assert!(
                suffers(&w, "light_footed"),
                "the kill leaves them light-footed"
            );
            return;
        }
    }
    panic!("no low blow killed the kobold");
}

#[test]
fn uncanny_dodge_wide_learning_song_of_rest_and_jack_of_all_trades() {
    let mut t = world("thief");
    let content = t.content().clone();
    // The bonus picks of Finesse may have raised it too.
    reach(&mut t, 24);
    let before = t.state.player().talent_mods(&content).burden_dex;
    reach(&mut t, 25);
    assert_eq!(
        t.state.player().talent_mods(&content).burden_dex,
        before + 2
    );
    let mut b = world("bard");
    let rolls = b.level_rolls();
    reach(&mut b, 6);
    assert_eq!(b.level_rolls(), rolls + 1, "Wide learning");
    // Song of rest: resting's pace half again (`healing_rate` reads it, as Fast healer's).
    reach(&mut b, 12);
    let after = b.state.player().talent_mods(&content).resting_percent;
    // Measured against a twin without it (the level's own pick may move the pace too).
    let mut twin = b.state().clone();
    let player = twin.player;
    (twin.actors.get_mut(&player).expect("player").talents).forget("song_of_rest");
    let before = (twin.actors[&player].talent_mods(&content)).resting_percent;
    assert_eq!(after, before + 50, "Song of rest");
    // Jack of all trades: every trained skill's benefits read 5 higher.
    let id = content.skill_id("tumble").expect("tumble");
    let effect = content
        .skill(id)
        .benefits
        .first()
        .expect("a benefit")
        .effect;
    let raised = |w: &World| {
        let me = w.state.player();
        let bonus: Vec<_> = me.skills.iter().map(|(s, _)| (s, 5)).collect();
        me.skills.effect_with(&content, effect, &bonus)
    };
    let plain = b.state.player().skills.effect(&content, effect);
    assert_eq!(b.state.player().skill_effect(&content, effect), plain);
    reach(&mut b, 25);
    assert_eq!(b.state.player().skill_effect(&content, effect), raised(&b));
    assert_eq!(b.state.player().talent_mods(&content).trained_skills, 5);
}

#[test]
fn inspire_costs_mana_and_virtuoso_sings_longer_for_none() {
    let mut w = world("bard");
    let inspire = Ability::Class(ClassActive::Inspire);
    assert_eq!(own(&w, inspire), None);
    reach(&mut w, 18);
    w.player_mut().mana = 2;
    assert_eq!(own(&w, inspire), Some(Some(Unavailable::NoMana)));
    run(&mut w, "use:inspire");
    assert_eq!(last(&w), "You have too little mana.");
    w.player_mut().mana = 10;
    let turn = w.state.turn;
    run(&mut w, "use:inspire");
    assert!(w.state.turn > turn, "a chat's time");
    assert_eq!(w.state.player().mana, 7);
    assert!(suffers(&w, "inspired"));
    let content = w.content().clone();
    let me = w.state.player();
    let profile = me.melee(&content);
    assert_eq!(
        me.effects.sharpen(&content, profile.clone()).to_hit,
        profile.to_hit + 1
    );
    reach(&mut w, 60);
    w.player_mut().mana = 0;
    assert_eq!(own(&w, inspire), Some(None), "Virtuoso: no mana");
    run(&mut w, "use:inspire");
    assert!(suffers(&w, "inspired_greatly"));
}

#[test]
fn throw_your_voice_is_the_bards_alone() {
    let mut w = world("warrior");
    assert_eq!(own(&w, Ability::ThrowVoice), None);
    let at = w.player_pos().step(Dir::E).step(Dir::E);
    run(&mut w, &format!("throw_voice:{},{}", at.x, at.y));
    assert_eq!(last(&w), "You do not know how to throw your voice.");
    let mut b = world("bard");
    reach(&mut b, 32);
    assert_eq!(own(&b, Ability::ThrowVoice), Some(None));
}

#[test]
fn honeyed_words_and_legend() {
    let mut w = world("bard");
    let content = w.content().clone();
    reach(&mut w, 50);
    let mods = w.state.player().talent_mods(&content);
    assert_eq!(mods.favour_percent, 15);
    assert_eq!((mods.quest_gold_percent, mods.quest_xp_percent), (25, 50));
}

#[test]
fn a_volley_looses_three_shots_for_one_then_spends() {
    let mut w = world("ranger");
    let volley = Ability::Class(ClassActive::Volley);
    reach(&mut w, 50);
    assert_eq!(own(&w, volley), Some(None));
    let content = w.content().clone();
    let count = |w: &World| {
        let me = w.state.player();
        me.ammunition_item(&content)
            .map_or(0, |i| me.inventory[i].count)
    };
    let arrows = count(&w);
    let at = w.player_pos().step(Dir::E).step(Dir::E).step(Dir::E);
    let turn = w.state.turn;
    let out = w.apply(&Action::UseAt(ClassActive::Volley, at));
    assert_eq!(w.state.turn, turn + 1, "one shot's time");
    assert_eq!(count(&w), arrows - 3, "three arrows loosed");
    assert!(said(&w).contains(&"You loose a volley!".to_string()));
    // It costs Focus, not a cooldown (spec §8 "Resources"): 6 of the 10, one back since.
    assert!(!suffers(&w, "spent"));
    assert!(!out.events.is_empty());
    let focus = content.resource_id("focus").expect("focus");
    let held = w.state.player().resources.get(focus).map(|p| p.value);
    assert!(held.is_some_and(|v| (4..6).contains(&v)), "{held:?}");
    assert_eq!(own(&w, volley), Some(Some(Unavailable::Short(focus))));
    let turn = w.state.turn;
    run(&mut w, &format!("use:volley:{},{}", at.x, at.y));
    assert_eq!(last(&w), "You have too little focus.");
    assert_eq!(w.state.turn, turn, "refused, no time");
}

#[test]
fn the_new_actives_replay_and_parse() {
    let spec: CharacterSpec = "human,male,ranger".parse().expect("a character");
    let mut w = World::start_run(
        BUILTIN.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let mut replay = Replay::new(&w);
    let at = w.player_pos().step(Dir::E).step(Dir::E);
    let text = format!("xp:3000000 levelup:auto use:volley:{},{} wait", at.x, at.y);
    for action in parse_actions(&text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(BUILTIN.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    assert_eq!(
        Action::UseAt(ClassActive::Volley, Pos::new(3, 4)).to_string(),
        "use:volley:3,4"
    );
    assert!(
        parse_actions("use:volley").is_err(),
        "a volley needs a cell"
    );
    assert_eq!(
        parse_actions("use:inspire").expect("parses"),
        [Action::Use(ClassActive::Inspire, None)]
    );
}

#[test]
fn the_thiefs_actives_replay_and_parse() {
    let spec: CharacterSpec = "human,male,thief".parse().expect("a character");
    let mut w = World::start_run(
        BUILTIN.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let mut replay = Replay::new(&w);
    let text = "xp:100000 levelup:auto use:lurk wait use:low_blow:E resource:openings wait";
    for action in parse_actions(text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(BUILTIN.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    for (text, action) in [
        ("use:lurk", Action::Use(ClassActive::Lurk, None)),
        (
            "use:low_blow:E",
            Action::Use(ClassActive::LowBlow, Some(Dir::E)),
        ),
    ] {
        assert_eq!(
            parse_actions(text).expect("parses"),
            std::slice::from_ref(&action)
        );
        assert_eq!(action.to_string(), text);
    }
}
