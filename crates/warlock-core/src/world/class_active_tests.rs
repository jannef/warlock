//! Class abilities in the world (spec §8 "Class abilities"; #243): the warrior's, barbarian's,
//! paladin's, and monk's level tables. A level brings its abilities and says so, the bonus picks
//! come from the class's trees, each passive changes the rule it names, and each active is used
//! from the abilities menu with its cooldown.

use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::combat::Category;
use crate::content::{ClassActive, Factor, StatusId};
use crate::dump::{self, Layer};
use crate::replay::Replay;
use crate::stats::{PerStat, Stat, Stats};
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// A human of `class` in the showcase (seed 7), with their kit, every score 14.
/// A human of `class` in the showcase, born under no sign, so a sign's gifts (Rise from the
/// ashes among them) leave the class's rules as the tests rig them.
fn world(class: &str) -> World {
    let spec: CharacterSpec = format!("human,male,{class}").parse().expect("a character");
    // Creation picks that no test here reads.
    let spec = spec.with_talents(vec!["keen_ears".into(), "lean".into()]);
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

fn status(world: &World, id: &str) -> StatusId {
    world.content().status_id(id).expect("a status")
}

fn suffers(world: &World, who: ActorId, id: &str) -> bool {
    world.state.actors[&who].effects.has(status(world, id))
}

/// The names of the class abilities the player holds, in the order given.
fn class_rows(world: &World) -> Vec<String> {
    (dump::layer_rows(world, Layer::Talents).into_iter())
        .filter_map(|row| row.strip_suffix(" (class)").map(str::to_string))
        .collect()
}

fn own(world: &World, active: ClassActive) -> Option<Option<Unavailable>> {
    (world.perception().abilities().into_iter())
        .find(|a| a.ability == Ability::Class(active))
        .map(|a| a.unavailable)
}

/// A creature of type `creature` placed east of the player, with `hp` hit points; its id.
fn beside(world: &mut World, creature: &str, hp: i32) -> ActorId {
    let content = world.content().clone();
    let at = world.player_pos().step(Dir::E);
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
fn a_level_brings_its_abilities_and_says_so() {
    let mut w = world("warrior");
    assert_eq!(
        class_rows(&w),
        ["Weapon drill"],
        "the signature from the start"
    );
    reach(&mut w, 5);
    assert_eq!(class_rows(&w), ["Weapon drill"]);
    reach(&mut w, 6);
    assert!(said(&w).contains(&"You gain Shield line.".to_string()));
    assert_eq!(class_rows(&w), ["Weapon drill", "Shield line"]);
    reach(&mut w, 12);
    assert_eq!(
        class_rows(&w),
        ["Weapon drill", "Shield line", "Veteran's wind"]
    );
    // A restored world has them too.
    let restored = World::from_state(w.content().clone(), w.state().clone()).expect("restored");
    assert_eq!(class_rows(&restored), class_rows(&w));
    assert_eq!(restored.state_hash(), w.state_hash());
    // A run without a character has none.
    let plain = World::new(BUILTIN.clone(), "showcase", Some(7)).expect("world");
    assert!(class_rows(&plain).is_empty());
}

#[test]
fn the_bonus_picks_come_from_the_classs_trees() {
    let mut w = world("warrior");
    reach(&mut w, 3);
    // Level 4 brings the warrior's first bonus pick: Weaponcraft or Guard only.
    let need = w
        .content()
        .experience()
        .threshold_for(4, Factor::of(w.content(), None));
    let more = need - w.player_xp();
    run(&mut w, &format!("xp:{more}"));
    run(&mut w, "advance:athletics*5");
    let due = w.due().expect("a pick due");
    assert_eq!((due.level, due.talents, due.bonus), (4, 0, 1));
    run(&mut w, "talent:lean");
    assert_eq!(
        last(&w),
        "You must choose a talent of Weaponcraft or Guard."
    );
    assert_eq!(w.due().map(|d| d.bonus), Some(1), "nothing spent");
    run(&mut w, "talent:armour_training");
    assert_eq!(last(&w), "You learn Armour training (1).");
    assert_eq!(w.due(), None);
    let progress = dump::layer_rows(&w, Layer::Progress);
    assert!(
        !progress.iter().any(|r| r.contains("bonus")),
        "{progress:?}"
    );
    // A barbarian has none until 10, then from Might and Vigour; levelup:auto spends it there.
    let mut b = world("barbarian");
    reach(&mut b, 9);
    let need = (b.content().experience()).threshold_for(
        10,
        Factor::of(b.content(), b.state.character.as_ref().map(|c| &c.spec)),
    );
    let more = need - b.player_xp();
    run(&mut b, &format!("xp:{more}"));
    assert_eq!(b.due().map(|d| d.bonus), Some(1));
    run(&mut b, "levelup:auto");
    assert_eq!(b.due(), None);
}

#[test]
fn weapon_drill_shield_wall_armour_drill_and_iron_stance() {
    let mut w = world("warrior");
    let content = w.content().clone();
    let mods = w.state.player().talent_mods(&content);
    let melee = crate::content::Role::Melee;
    assert_eq!(
        mods.marks(melee, 100),
        125,
        "Weapon drill: a quarter more marks"
    );
    assert_eq!(mods.marks(crate::content::Role::Launcher, 100), 100);
    let defence = |w: &World| {
        let me = w.state.player();
        me.defence(&content, crate::burden::Burden::Unburdened, None)
    };
    // Shield line: the class talent's own effect gives the shield +1 (the picks levelup:auto
    // makes on the way may add to Defence too).
    let book = content.talents();
    let line = book.get(book.id("shield_line").expect("shield line"));
    let line = crate::talent::TalentMods::of(&content, line.effects_at(1));
    let (benefit, _) = line.weapon_benefit("", crate::content::Role::ShieldBlock, 0);
    assert_eq!(benefit.shield, 1, "Shield line");
    let protection = w.state.player().protection(&content, None).against(None);
    reach(&mut w, 25);
    assert_eq!(
        w.state.player().protection(&content, None).against(None),
        protection + 1,
        "Armour drill, in the warrior's leathers"
    );
    // Iron stance: aggressive costs no Dodge.
    run(&mut w, "stance:aggressive");
    let aggressive = defence(&w).bonus(Category::Dodge);
    reach(&mut w, 40);
    assert_eq!(defence(&w).bonus(Category::Dodge), aggressive + 3);
}

#[test]
fn veterans_wind_heals_by_the_level_then_winds() {
    let mut w = world("warrior");
    run(&mut w, "use:veterans_wind");
    assert_eq!(last(&w), "You have no such ability.");
    reach(&mut w, 12);
    assert_eq!(own(&w, ClassActive::VeteransWind), Some(None));
    w.player_mut().hp = 1;
    let turn = w.state.turn;
    run(&mut w, "use:veterans_wind");
    assert_eq!(w.state.player().hp, 13, "12 at level 12 (at least 10)");
    assert_eq!(w.state.turn, turn + 1);
    let winded = status(&w, "winded");
    assert_eq!(
        own(&w, ClassActive::VeteransWind),
        Some(Some(Unavailable::While(winded)))
    );
    run(&mut w, "use:veterans_wind");
    assert_eq!(last(&w), "You are still winded.");
    assert_eq!(w.state.turn, turn + 1, "refused: no time");
}

#[test]
fn rage_raises_strength_blocks_reading_and_exhausts() {
    let mut w = world("barbarian");
    let content = w.content().clone();
    let player = w.state.player;
    let strength = |w: &World| (w.state.player().effective_stats(&content)).score(Stat::Strength);
    assert_eq!(
        own(&w, ClassActive::Rage),
        Some(None),
        "the signature at level 1"
    );
    let calm = strength(&w);
    run(&mut w, "use:rage");
    assert!(said(&w).contains(&"You fly into a rage!".to_string()));
    assert!(suffers(&w, player, "raging"));
    assert_eq!(strength(&w), calm + 2);
    let raging = status(&w, "raging");
    assert_eq!(
        own(&w, ClassActive::Rage),
        Some(Some(Unavailable::While(raging)))
    );
    run(&mut w, "use:rage");
    assert_eq!(last(&w), "You are still raging.");
    // Five minutes on, exhausted: no rage until that passes.
    run(&mut w, "wait*51");
    assert!(suffers(&w, player, "exhausted"));
    run(&mut w, "use:rage");
    assert_eq!(last(&w), "You are still exhausted.");
    // Mighty rage: twice the Strength.
    let mut w = world("barbarian");
    reach(&mut w, 50);
    let calm = strength(&w);
    run(&mut w, "use:rage");
    assert_eq!(strength(&w), calm + 4);
}

#[test]
fn thick_hide_fast_movement_hardened_and_mighty_thews() {
    let mut w = world("barbarian");
    let content = w.content().clone();
    let natural = |w: &World| {
        let me = w.state.player();
        (me.defence(&content, crate::burden::Burden::Unburdened, None)).bonus(Category::Natural)
    };
    assert_eq!(natural(&w), 0);
    reach(&mut w, 6);
    assert_eq!(natural(&w), 2, "Thick hide: furs are no body armour");
    let player = w.state.player;
    let floor = content.terrain(content.terrain_id("floor").expect("floor"));
    let step = |w: &World| w.step_price(player, floor, false);
    let before = step(&w);
    reach(&mut w, 12);
    assert_eq!(step(&w), before * 9 / 10, "Fast movement");
    // Hardened: past level 20 levels add hit points again, one more each.
    reach(&mut w, 24);
    let at_24 = w.state.player().max_hp(&content);
    reach(&mut w, 25);
    let per_level = w.level_hp(&content);
    assert_eq!(w.state.player().max_hp(&content), at_24 + per_level + 1);
    // Mighty thews: scores and potentials, once.
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(20));
    let stats = w.state.player().stats;
    reach(&mut w, 32);
    let after = w.state.player().stats;
    for stat in [Stat::Strength, Stat::Toughness] {
        assert!(after.score(stat) >= stats.score(stat) + 2, "{stat:?}");
        assert!(
            after.potential(stat) >= stats.potential(stat) + 2,
            "{stat:?}"
        );
    }
}

#[test]
fn blood_frenzy_and_unstoppable_keep_the_rage() {
    let mut w = world("barbarian");
    reach(&mut w, 60);
    let player = w.state.player;
    run(&mut w, "use:rage");
    // A kill sets the rage's easing back a minute.
    let raging = status(&w, "raging");
    w.player_mut().effects.iter_mut().for_each(|e| e.wear = 200);
    w.frenzy();
    assert_eq!(
        w.state.player().effects.get(raging).map(|e| e.wear),
        Some(140)
    );
    // While raging, bleeding ticks nothing.
    let hp = w.state.player().hp;
    run(&mut w, "afflict:bleeding wait*4");
    assert_eq!(w.state.player().hp, hp, "no blood lost while raging");
    // A blow that would fell the raging barbarian leaves 1 hit point and ends the rage.
    let kobold = beside(&mut w, "kobold", 5);
    let mut events = Vec::new();
    assert!(!w.hurt(player, 999, fight::Harm::Actor(kobold), &mut events));
    assert_eq!(w.state.player().hp, 1);
    assert!(!suffers(&w, player, "raging") && suffers(&w, player, "exhausted"));
    assert!(
        w.hurt(player, 999, fight::Harm::Actor(kobold), &mut events),
        "once a rage"
    );
}

#[test]
fn lay_on_hands_smite_and_radiance_answer_only_the_good() {
    let mut w = world("paladin");
    let content = w.content().clone();
    w.player_mut().hp = 1;
    run(&mut w, "use:lay_on_hands");
    assert!(said(&w).contains(&"Warmth flows from your hands.".to_string()));
    assert_eq!(w.state.player().hp, 3, "2 a level at level 1");
    let drained = status(&w, "drained");
    assert_eq!(
        own(&w, ClassActive::LayOnHands),
        Some(Some(Unavailable::While(drained)))
    );
    // Ten minutes later it answers again.
    run(&mut w, "wait*101");
    assert_eq!(own(&w, ClassActive::LayOnHands), Some(None));
    // Strayed from Good: kept but greyed, and refused.
    w.player_mut().alignment = Some(crate::alignment::Score::new(0));
    assert_eq!(
        own(&w, ClassActive::LayOnHands),
        Some(Some(Unavailable::Strayed))
    );
    run(&mut w, "use:lay_on_hands");
    assert_eq!(last(&w), "You have strayed: your gift does not answer.");
    w.player_mut().alignment = Some(crate::alignment::Score::new(800));
    // Smite: a blow with more dice, then spent.
    reach(&mut w, 12);
    let smite = w.state.player().talent_mods(&content).smite.expect("smite");
    assert_eq!(smite.dice_at(12).to_string(), "1d6");
    assert_eq!(smite.dice_at(35).to_string(), "3d6");
    let kobold = beside(&mut w, "kobold", 500);
    let turn = w.state.turn;
    run(&mut w, "use:smite:E");
    assert!(said(&w).contains(&"You smite!".to_string()));
    assert_eq!(w.state.turn, turn + 1);
    assert!(suffers(&w, w.state.player, "spent"));
    assert!(w.state.actors.contains_key(&kobold));
    run(&mut w, "use:smite:E");
    assert_eq!(last(&w), "You are still spent.");
    // Holy radiance: a light of the paladin's own, on and off.
    reach(&mut w, 40);
    let lit = |w: &World| {
        let me = w.state.player();
        let (status, _) = me.talent_mods(&content).radiance.expect("radiance");
        me.effects.has(status)
    };
    run(&mut w, "use:holy_radiance");
    assert!(lit(&w));
    assert!(said(&w).contains(&"A soft light shines about you.".to_string()));
    run(&mut w, "use:holy_radiance");
    assert!(!lit(&w));
    assert_eq!(last(&w), "Your light fades.");
}

#[test]
fn divine_health_steadfast_and_avatar() {
    let mut w = world("paladin");
    let content = w.content().clone();
    let player = w.state.player;
    let fever = content.disease_id("filth_fever").expect("a disease");
    reach(&mut w, 18);
    for _ in 0..20 {
        assert!(!w.expose(player, fever, None), "never caught");
    }
    assert!(!w.state.load(&content, w.state.player()).ignores_burdened);
    reach(&mut w, 32);
    assert!(
        w.state.load(&content, w.state.player()).ignores_burdened,
        "Steadfast"
    );
    // Avatar: lay on hands heals fully and cures what ticks.
    reach(&mut w, 60);
    w.player_mut().hp = 1;
    run(&mut w, "afflict:poisoned afflict:bleeding use:lay_on_hands");
    let max = w.state.player().max_hp(&content);
    assert_eq!(w.state.player().hp, max);
    assert!(!suffers(&w, player, "poisoned") && !suffers(&w, player, "bleeding"));
    assert_eq!(
        w.state
            .player()
            .effects
            .get(status(&w, "drained"))
            .map(|e| e.severity),
        Some(1)
    );
}

#[test]
fn martial_arts_flurry_still_mind_and_quivering_palm() {
    let mut w = world("monk");
    let content = w.content().clone();
    // The monk's own kit: gloves (Brawler) or a staff (Staff fighter); bare hands either way.
    w.player_mut().equipment = Default::default();
    let fists = |w: &World| w.state.player().melee(&content).damage.to_string();
    assert_eq!(fists(&w), "1d6", "Martial arts");
    assert_eq!(
        w.state.player().kick(&content).damage.to_string(),
        "1d6",
        "kicks too"
    );
    reach(&mut w, 18);
    let kobold = beside(&mut w, "kobold", 500);
    let player = w.state.player;
    let flurry = w.flurry(player, kobold).expect("a second blow");
    assert_eq!(
        flurry.profile.to_hit,
        w.state.player().melee(&content).to_hit - 2
    );
    // Still mind: a status's lowering of Intelligence never applies.
    reach(&mut w, 25);
    // Still mind's own effects (the picks levelup:auto made may touch statuses too).
    let book = content.talents();
    let mind = book.get(book.id("still_mind").expect("still mind"));
    let mods = crate::talent::TalentMods::of(&content, mind.effects.iter().cloned());
    let poisoned = status(&w, "poisoned");
    assert_eq!(mods.penalty(poisoned, Stat::Intelligence, -2), 0);
    assert_eq!(mods.penalty(poisoned, Stat::Strength, -2), -2);
    assert_eq!(mods.check(Stat::Willpower), 2);
    // Quivering palm: a wounding blow leaves the foe bleeding three degrees.
    reach(&mut w, 50);
    run(&mut w, "use:quivering_palm:E");
    assert!(suffers(&w, player, "spent"));
    let bleeding = status(&w, "bleeding");
    let wounded =
        (w.state.actors.get(&kobold)).map(|k| (k.hp, k.effects.get(bleeding).map(|e| e.severity)));
    match wounded {
        Some((hp, severity)) if hp < 500 => assert!(severity >= Some(3), "{severity:?}"),
        _ => {}
    }
    // Perfect self: a bigger die.
    reach(&mut w, 60);
    assert_eq!(fists(&w), "1d8");
}

#[test]
fn whirlwind_strikes_every_hostile_beside_for_one_attack() {
    let mut w = world("warrior");
    run(&mut w, "use:whirlwind");
    assert_eq!(last(&w), "You have no such ability.");
    reach(&mut w, 50);
    run(&mut w, "use:whirlwind");
    assert_eq!(last(&w), "There is nobody beside you to strike.");
    let first = beside(&mut w, "kobold", 500);
    let second = beside(&mut w, "kobold", 500);
    let below = w.player_pos().step(Dir::S);
    w.state.actors.get_mut(&second).expect("second").pos = below;
    let turn = w.state.turn;
    let out = w.apply(&Action::Use(ClassActive::Whirlwind, None));
    assert_eq!(w.state.turn, turn + 1, "one attack's time");
    let struck: Vec<ActorId> = (out.events.iter())
        .filter_map(|e| match e {
            Event::Attacked {
                attacker, target, ..
            } if *attacker == w.state.player => Some(*target),
            _ => None,
        })
        .collect();
    assert!(
        struck.contains(&first) && struck.contains(&second),
        "{struck:?}"
    );
}

#[test]
fn class_actives_replay_and_dump_the_same() {
    let spec: CharacterSpec = "human,male,barbarian".parse().expect("a character");
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
    for action in parse_actions("use:rage wait*3 xp:800 levelup:auto wait").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(BUILTIN.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, BUILTIN.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    assert_eq!(class_rows(&loaded), ["Rage"]);
    let abilities = (Action::Use(ClassActive::Smite, Some(Dir::E))).to_string();
    assert_eq!(abilities, "use:smite:E");
}
