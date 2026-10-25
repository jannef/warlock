//! The racial talents (spec §8 "Talents", the Racial tree; #241): each asks for its race and
//! changes the one rule it names; Versatile, the humans', is granted and adds a pick at
//! creation and at level 30.

use super::*;
use crate::action::parse_actions;
use crate::burden::Burden;
use crate::character::CharacterSpec;
use crate::combat::Category;
use crate::content::{ContentFiles, Role};
use crate::stats::{PerStat, Stats};
use crate::talent;
use std::sync::LazyLock;

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));

fn spec(text: &str) -> CharacterSpec {
    text.parse().expect("a character")
}

/// The showcase as `character`, every score 14, on the scenario's own pack (no picks).
fn world(character: &str) -> World {
    let mut w = World::with_character(CONTENT.clone(), "showcase", Some(7), Some(&spec(character)))
        .expect("world");
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    w
}

/// A kitted run of the showcase as `character`, its creation picks at random.
fn kitted(character: &str) -> World {
    World::start_run(
        CONTENT.clone(),
        "showcase",
        Some(3),
        Some(&spec(character)),
        None,
        true,
    )
    .expect("starts")
}

fn give(w: &mut World, id: &str) {
    w.player_mut().talents.take(id, None);
}

fn mods(w: &World) -> talent::TalentMods {
    w.state.player().talent_mods(w.content())
}

/// The talents the player may pick now, by id.
fn offered(w: &World) -> Vec<String> {
    let book = w.content().talents();
    (talent::offers(w.content(), &w.candidate()).into_iter())
        .map(|(id, _)| book.get(id).id.clone())
        .collect()
}

/// The talents picked at creation (granted ones left out).
fn picked(w: &World) -> usize {
    let book = w.content().talents();
    (w.state.player().talents.iter())
        .filter(|t| book.id(&t.id).is_some_and(|id| !book.get(id).granted))
        .count()
}

#[test]
fn every_racial_talent_asks_for_its_race_and_only_that_race_is_offered_it() {
    let racial = |race: &str| {
        let book = CONTENT.talents();
        let tree = book.tree_id("racial").expect("the racial tree");
        let mut w = world(&format!("{race},male,warrior"));
        // Enough of everything that only the race decides.
        w.player_mut().stats = Stats::new(PerStat::splat(18), PerStat::splat(40));
        let mut ids: Vec<String> = (offered(&w).into_iter())
            .filter(|id| book.id(id).is_some_and(|t| book.get(t).tree == tree))
            .collect();
        ids.sort();
        ids
    };
    assert_eq!(racial("dwarf"), ["dwarven_resilience", "stone_sense"]);
    assert_eq!(racial("gnome"), ["gem_lore", "tinkerer"]);
    assert_eq!(racial("half_orc"), ["ferocity", "orcish_hide"]);
    assert_eq!(racial("half_elf"), ["diplomat", "dual_heritage"]);
    assert_eq!(racial("halfling"), ["lucky", "small_target"]);
    // An elf needs a level and a bow skill first; a human has none to pick (Versatile is given).
    assert!(racial("elf").is_empty());
    assert!(racial("human").is_empty());
}

#[test]
fn humans_are_granted_versatile_and_pick_one_more_at_creation_and_at_level_30() {
    let human = kitted("human,male,warrior");
    let book = human.content().talents();
    assert!(human.state.player().talents.has("versatile", 1));
    let own = human.perception().own_talents();
    let versatile = own.iter().find(|t| t.name == "Versatile").expect("held");
    assert!(versatile.granted && versatile.race && !versatile.class);
    assert_eq!(picked(&human), 3, "two and Versatile's");
    let bard = kitted("human,other,bard");
    assert_eq!(
        picked(&bard),
        5,
        "the bard's, Versatile's, and Lore, which the class table gives"
    );
    assert_eq!(creation_count(&CONTENT, &spec("human,other,bard")), 4);
    assert_eq!(creation_count(&CONTENT, &spec("elf,other,bard")), 3);
    let dwarf = kitted("dwarf,male,warrior");
    assert_eq!(picked(&dwarf), 2);
    assert!(!dwarf.state.player().talents.has("versatile", 1));
    let versatile = book.id("versatile").expect("versatile");
    assert!(book.get(versatile).granted, "never on a pick list");
    // Level 30 brings two picks to a human, one to anyone else.
    let at_30 = |mut w: World| {
        w.state.progress.due.clear();
        w.level_up(29, 30);
        (w.state.progress.due.iter())
            .find(|d| d.level == 30)
            .map_or(0, |d| d.talents)
    };
    assert_eq!(at_30(kitted("human,male,warrior")), 2);
    assert_eq!(at_30(kitted("dwarf,male,warrior")), 1);
    // Level 29 brings nothing more.
    let mut w = kitted("human,male,warrior");
    w.state.progress.due.clear();
    w.level_up(28, 29);
    assert!(w.state.progress.due.iter().all(|d| d.talents == 0));
}

#[test]
fn versatile_is_granted_again_on_a_restore_and_saves_keep_it() {
    let w = kitted("human,female,thief");
    let json = serde_json::to_string(&w.state).expect("serializes");
    let restored = World::from_state(CONTENT.clone(), serde_json::from_str(&json).expect("loads"))
        .expect("restores");
    assert!(restored.state.player().talents.has("versatile", 1));
    assert_eq!(restored.state_hash(), w.state_hash());
}

#[test]
fn elven_accuracy_threatens_more_with_bows_and_ageless_slows_hunger_and_disease() {
    let mut w = world("elf,male,ranger");
    give(&mut w, "elven_accuracy");
    let m = mods(&w);
    assert_eq!(m.weapon_benefit("bows", Role::Launcher, 3).0.threat, 1);
    assert_eq!(m.weapon_benefit("slings", Role::Launcher, 3).0.threat, 0);
    // Ageless: hunger 20 % slower, a disease's progressions twice as far apart.
    let hunger = w.hunger_percent();
    give(&mut w, "ageless");
    assert_eq!(w.hunger_percent(), hunger * 80 / 100);
    let interval = |w: &mut World| {
        let player = w.state.player;
        run(w, "infect:filth_fever");
        let fever = w.content().disease_id("filth_fever").expect("filth fever");
        let when = w.state.time;
        w.disease_turn(player, fever, when, &mut Vec::new());
        let left = w.state.player().diseases.get(fever).map(|i| i.left);
        (left, w.content().disease(fever).interval_seconds())
    };
    let (left, every) = interval(&mut w);
    assert_eq!(left, Some(u32::try_from(every * 2).expect("fits")));
    let mut plain = world("elf,male,ranger");
    let (left, every) = interval(&mut plain);
    assert_eq!(left, Some(u32::try_from(every).expect("fits")));
}

#[test]
fn half_elves_learn_more_and_bargain_better() {
    let mut w = world("half_elf,female,bard");
    let rolls = w.level_rolls();
    give(&mut w, "dual_heritage");
    assert_eq!(w.level_rolls(), rolls + 1);
    give(&mut w, "diplomat");
    let m = mods(&w);
    assert_eq!((m.favour_percent, m.quest_gold_percent), (10, 25));
}

#[test]
fn stone_sense_finds_what_is_hidden_in_stone_and_digs_faster() {
    let mut w = world("dwarf,male,warrior");
    let level = w.state.current();
    let stone = |content: &Content, stone: bool| {
        (level.terrain.iter())
            .find(|(_, t)| content.terrain(**t).stone == stone)
            .map(|(p, _)| p)
            .expect("a cell")
    };
    let (rock, other) = (stone(&CONTENT, true), stone(&CONTENT, false));
    assert_eq!((w.stone_find(rock), w.stone_find(other)), (0, 0));
    give(&mut w, "stone_sense");
    assert_eq!((w.stone_find(rock), w.stone_find(other)), (4, 0));
    assert_eq!(mods(&w).dig_percent, 25);
}

#[test]
fn dwarven_resilience_eases_poison_and_stout_lightens_a_strained_load() {
    let mut w = world("dwarf,male,warrior");
    give(&mut w, "dwarven_resilience");
    let poisoned = w.content().status_id("poisoned").expect("poisoned");
    let m = mods(&w);
    assert_eq!((m.eases_every(poisoned, 60), m.disease_checks), (30, 2));
    // Strained: a dwarf with Stout pays only Burdened's price.
    let content = w.content().clone();
    let limit = w.state.load(&content, w.state.player()).limit;
    let stone = content.object_id("rock").expect("a rock");
    let each = u64::from(content.object(stone).weight);
    let count = u32::try_from(limit * 4 / 5 / each).expect("fits");
    w.player_mut()
        .inventory
        .push(crate::object::Stack::new(stone, count));
    let load = w.state.load(&content, w.state.player());
    assert_eq!(
        (load.burden(), load.penalty()),
        (Burden::Strained, Burden::Strained)
    );
    give(&mut w, "stout");
    let load = w.state.load(&content, w.state.player());
    assert_eq!(
        (load.burden(), load.penalty()),
        (Burden::Strained, Burden::Burdened)
    );
}

#[test]
fn halflings_are_lucky_slingers_and_hard_for_the_big_to_hit() {
    let mut w = world("halfling,male,thief");
    give(&mut w, "lucky");
    give(&mut w, "slinger");
    let m = mods(&w);
    assert!(m.reroll_fumble);
    assert_eq!(m.weapon_benefit("slings", Role::Launcher, 2).0.damage, 1);
    assert_eq!(
        (
            m.launcher_range_for(Some("slings")),
            m.launcher_range_for(Some("bows"))
        ),
        (2, 0)
    );
    // Small target: Dodge +1 against a creature larger than the halfling, none against one as
    // small, and none for the same talent on a medium human.
    let player = w.state.player;
    let content = w.content().clone();
    let creature = |w: &World, larger: bool| {
        let me = w.state.size_of(&content, w.state.player());
        (w.state.actors.iter())
            .filter(|(id, _)| **id != player)
            .find(|(_, a)| (w.state.size_of(&content, a) > me) == larger)
            .map(|(id, _)| *id)
    };
    let big = creature(&w, true).expect("a larger creature");
    let dodge = |w: &World, by: ActorId| {
        let struck = w.target_against(player, false);
        let before = struck.defence.bonus(Category::Dodge);
        w.small_target(by, player, struck)
            .defence
            .bonus(Category::Dodge)
            - before
    };
    assert_eq!(dodge(&w, big), 0, "not yet");
    give(&mut w, "small_target");
    assert_eq!(dodge(&w, big), 1);
    if let Some(small) = creature(&w, false) {
        assert_eq!(dodge(&w, small), 0);
    }
    let mut human = world("human,male,thief");
    give(&mut human, "small_target");
    let player = human.state.player;
    let any = (human.state.actors.keys())
        .find(|id| **id != player)
        .copied();
    let struck = human.target_against(player, false);
    let before = struck.defence.bonus(Category::Dodge);
    let any = any.expect("a creature");
    let after = human
        .small_target(any, player, struck)
        .defence
        .bonus(Category::Dodge);
    assert_eq!(
        after, before,
        "nothing in the showcase is larger than medium"
    );
}

#[test]
fn gnomes_tinker_with_traps_and_know_their_gems() {
    let mut w = world("gnome,female,wizard");
    give(&mut w, "tinkerer");
    give(&mut w, "gem_lore");
    let m = mods(&w);
    assert!(m.disarm && m.gem_worth);
    assert_eq!(m.gem_find_percent, 100);
}

#[test]
fn half_orcs_hit_harder_when_badly_hurt_and_have_a_thicker_hide() {
    let mut w = world("half_orc,male,barbarian");
    let content = w.content().clone();
    let bonus = |w: &World| w.state.player().melee(&content).bonus;
    let max = w.state.player().max_hp(&content);
    let whole = bonus(&w);
    give(&mut w, "ferocity");
    assert_eq!(bonus(&w), whole, "not while hale");
    w.player_mut().hp = (max - 1) / 3;
    assert_eq!(bonus(&w), whole + 2, "below a third");
    let protection = |w: &World| mods(w).protection_with(None, false, 0);
    let before = protection(&w);
    give(&mut w, "orcish_hide");
    assert_eq!(protection(&w), before + 1);
}

#[test]
fn a_race_may_grant_only_a_granted_racial_talent() {
    let mut files = ContentFiles::builtin();
    let races = include_str!("../../../../assets/data/races.toml");
    let wrong = races.replacen("talents = [\"versatile\"]", "talents = [\"healthy\"]", 1);
    assert_ne!(wrong.as_str(), races);
    files.insert("data/races.toml", &wrong);
    let err = Content::load(&files).expect_err("refused").to_string();
    assert!(
        err.contains("not a granted talent of the `racial` tree"),
        "{err}"
    );
}

#[test]
fn a_replay_that_takes_a_racial_talent_verifies() {
    // The scenario's own character, a halfling here: no creation picks, the level's pick Lucky.
    let mut w = World::with_character(
        CONTENT.clone(),
        "showcase",
        Some(7),
        Some(&spec("halfling,male,thief")),
    )
    .expect("world");
    let mut replay = crate::replay::Replay::new(&w);
    let character = w.state.character.as_ref().map(|c| c.spec.clone());
    let factor = crate::content::Factor::of(w.content(), character.as_ref());
    let need = w.content().experience().threshold_for(3, factor);
    let steps = format!("xp:{need} talent:lucky levelup:auto move:E wait");
    for action in parse_actions(&steps).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(w.state.player().talents.has("lucky", 1));
    assert!(mods(&w).reroll_fumble);
    let verified = (crate::replay::Replay::from_json(&replay.to_json()).expect("json"))
        .run(CONTENT.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}

fn run(w: &mut World, actions: &str) {
    for a in parse_actions(actions).expect("actions") {
        w.apply(&a);
    }
}
