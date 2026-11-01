//! Talents in the world (spec §8 "Talents"): a level every third brings a pick that must be
//! made before anything else, prerequisites are checked in words, ranks come one at a time,
//! each talent changes the one rule it names, picks nothing can be spent on are dropped, and
//! all of it survives saves and replays while runs without talents hash as before.

use super::search::{Hidden, Look};
use super::*;
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, Factor};
use crate::dump::{self, Layer};
use crate::noise::{self, Act};
use crate::replay::Replay;
use crate::stats::{PerStat, Stat, Stats};
use std::sync::LazyLock;

/// The built-in content without the class tables (`world/class_tests.rs` has those): the
/// warrior's bonus picks and abilities stay out of the talents' way.
static BUILTIN: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    crate::content::strip_class_tables(&mut files);
    Arc::new(Content::load(&files).expect("content"))
});

/// A human warrior (a 100 % experience factor) in the showcase.
fn world() -> World {
    world_with(BUILTIN.clone())
}

fn world_with(content: Arc<Content>) -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let w =
        World::with_character(content.clone(), "showcase", Some(7), Some(&spec)).expect("world");
    // Born under no sign, so the talents held are the ones picked (a sign's gifts are talents
    // too: world/stars_tests.rs).
    let mut state = w.state().clone();
    state.character.as_mut().expect("a character").sign = None;
    let mut w = World::from_state(content.clone(), state).expect("restores");
    // Nor the race's grant (a human's Versatile, world/racial_tests.rs) until a restore.
    w.player_mut().talents.regrant(&content, &[]);
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

/// Every score (and potential) of the player set to `score`.
fn set_scores(world: &mut World, score: u8) {
    let player = world.state.player;
    let actor = world.state.actors.get_mut(&player).expect("player");
    actor.stats = Stats::new(PerStat::splat(score), PerStat::splat(score));
}

fn set_score(world: &mut World, stat: Stat, score: u8) {
    let player = world.state.player;
    let actor = world.state.actors.get_mut(&player).expect("player");
    let mut scores = actor.stats.scores();
    scores[stat] = score;
    actor.stats = Stats::new(scores, PerStat::splat(40));
}

/// Grants the XP that takes the player to `level`, and spends the levels' skill rolls (on the
/// first trained skill), so only talent picks are left due.
fn reach(world: &mut World, level: u8) {
    let spec = world.state.character.as_ref().map(|c| &c.spec);
    let factor = Factor::of(world.content(), spec);
    let need = world.content().experience().threshold_for(level, factor);
    let more = need - world.player_xp();
    run(world, &format!("xp:{more}"));
    assert_eq!(world.player_level(), level, "reached level {level}");
    world.advance_auto();
}

fn player(world: &World) -> &Actor {
    world.state.player()
}

#[test]
fn a_third_level_asks_for_a_talent_and_refuses_everything_else_until_it_is_made() {
    let mut w = world();
    assert_eq!(w.due(), None);
    reach(&mut w, 2);
    assert_eq!(w.due(), None, "level 2 brings no pick");
    reach(&mut w, 3);
    assert_eq!(
        w.due(),
        Some(Due {
            level: 3,
            talents: 1,
            rolls: 0,
            bonus: 0,
        })
    );
    assert!(
        last(&w).starts_with("Your athletics improves."),
        "the rolls spent"
    );
    let turn = w.turn();
    for refused in ["move:E", "wait", "rest:1h", "search", "xp:10", "skip:1h"] {
        run(&mut w, refused);
        // The log folds repeats into "(×N)".
        assert!(
            last(&w).starts_with("You must choose a talent first."),
            "{refused}: {}",
            last(&w)
        );
        assert_eq!(w.turn(), turn, "{refused} took no turn");
    }
    run(&mut w, "advance:stealth");
    assert_eq!(last(&w), "You have no skill rolls to spend.");
    run(&mut w, "talent:healthy");
    assert_eq!(last(&w), "You learn Healthy.");
    assert_eq!(w.due(), None);
    assert_eq!(player(&w).talents.rank_of("healthy", None), 1);
    run(&mut w, "wait");
    assert_eq!(w.turn(), turn + 1, "free to act again");
    run(&mut w, "talent:healthy");
    assert_eq!(last(&w), "You have no talent to choose now.");
    run(&mut w, "levelup:auto");
    assert_eq!(last(&w), "Nothing is due.");
}

#[test]
fn prerequisites_are_checked_and_said_in_words() {
    let mut w = world();
    set_scores(&mut w, 10);
    reach(&mut w, 3);
    run(&mut w, "talent:dodge");
    assert_eq!(last(&w), "You cannot take Dodge: you need Dexterity 13.");
    assert!(w.due().is_some(), "still due");
    run(&mut w, "talent:nope");
    assert_eq!(last(&w), "There is no talent `nope`.");
    run(&mut w, "talent:healthy:bows");
    assert_eq!(last(&w), "Healthy takes no choice.");
    run(&mut w, "advance:nope");
    assert_eq!(last(&w), "There is no skill `nope`.");
    set_score(&mut w, Stat::Dexterity, 13);
    run(&mut w, "talent:dodge");
    assert_eq!(last(&w), "You learn Dodge.");
    // Prerequisites hold at the pick and are never checked again: a lower score later keeps it.
    set_scores(&mut w, 10);
    assert_eq!(player(&w).talent_mods(w.content()).dodge, 1);
}

#[test]
fn ranks_come_one_at_a_time_and_hardy_grows_with_the_levels() {
    let mut w = world();
    set_scores(&mut w, 17);
    let content = w.content().clone();
    let max = |w: &World| player(w).max_hp(&content);
    reach(&mut w, 3);
    let before = max(&w);
    let hp_before = player(&w).hp;
    run(&mut w, "talent:hardy");
    assert_eq!(last(&w), "You learn Hardy (1).");
    assert_eq!(max(&w), before + 3, "at least 3 at level 3");
    assert_eq!(player(&w).hp, hp_before + 3, "the current hit points too");
    let rank = |w: &World| player(w).talents.rank_of("hardy", None);
    assert_eq!(rank(&w), 1);
    reach(&mut w, 6);
    run(&mut w, "talent:hardy");
    assert_eq!(last(&w), "You learn Hardy (2).");
    reach(&mut w, 9);
    run(&mut w, "talent:hardy");
    assert_eq!(rank(&w), 3);
    reach(&mut w, 12);
    let at_12 = max(&w);
    run(&mut w, "talent:hardy");
    assert_eq!(last(&w), "You already have all of Hardy.");
    run(&mut w, "levelup:auto");
    assert_eq!(w.due(), None);
    // Levels 13 to 15 grow nothing from Hardy (3 is its floor until 16): what they add is the
    // levels' own hit points, the same each level. Level 16 adds that, and 1 more per rank.
    reach(&mut w, 15);
    let at_15 = max(&w);
    let per_level = (at_15 - at_12) / 3;
    assert_eq!(
        at_15 - at_12,
        per_level * 3,
        "nothing from Hardy at 13 to 15"
    );
    run(&mut w, "levelup:auto");
    reach(&mut w, 16);
    assert_eq!(
        max(&w) - at_15,
        per_level + 3,
        "level 16: one more per rank"
    );
}

#[test]
fn several_levels_at_once_queue_their_picks_and_auto_spends_them_in_order() {
    let mut w = world();
    reach(&mut w, 6);
    assert_eq!(w.due().map(|d| d.level), Some(3));
    assert_eq!(w.state.progress.due.len(), 2);
    run(&mut w, "talent:lean");
    assert_eq!(w.due().map(|d| d.level), Some(6));
    run(&mut w, "levelup:auto");
    assert_eq!(w.due(), None);
    assert_eq!(player(&w).talents.len(), 2);
    // Auto took the first talent of the first tree it may take: Vigour's, in file order,
    // Clotting.
    assert_eq!(last(&w), "You learn Clotting.");
}

/// A save from before a talent was cut (#774: Gifted, Bellows) loads: the talents the content no
/// longer has are let go, each said in the log, and the rest are kept.
#[test]
fn a_save_naming_a_cut_talent_loads_without_it() {
    let mut w = world();
    w.player_mut().talents.take("healthy", None);
    w.player_mut().talents.take("gifted", Some("strength"));
    w.player_mut().talents.take("bellows", None);
    // #787 and #788's: one cut, one merged into Pious, one from creation; #789's capstone.
    // #790's: three chains, now ranks, and three clones, now the pickables' ids.
    let merged = [
        "great_cleave",
        "weapon_specialization",
        "breaker",
        "song_of_rest",
        "wild_empathy",
        "divine_health",
    ];
    for id in ["silver_tongue", "penitent", "favoured_child", "insured"]
        .into_iter()
        .chain(merged)
    {
        w.player_mut().talents.take(id, None);
    }
    let back = World::from_state(w.content().clone(), w.state().clone()).expect("it loads");
    let held: Vec<&str> = (back.state().player().talents.iter())
        .map(|t| t.id.as_str())
        .collect();
    assert_eq!(
        held,
        ["versatile", "healthy"],
        "the race's grant and the pick"
    );
    for id in [
        "gifted",
        "bellows",
        "silver_tongue",
        "penitent",
        "favoured_child",
        "insured",
    ]
    .into_iter()
    .chain(merged)
    {
        assert!(
            back.retiered()
                .iter()
                .any(|l| l.contains(&format!("`{id}`"))),
            "{:?}",
            back.retiered()
        );
    }
}

#[test]
fn a_pick_nothing_can_be_spent_on_is_dropped() {
    // Without talents in the content, a level asks for nothing and the player acts on.
    let mut files = ContentFiles::builtin();
    files.remove("data/talents.toml");
    files.remove("data/talents/");
    files.remove("data/background.toml");
    let content = Arc::new(Content::load(&files).expect("content without talents"));
    let mut w = world_with(content);
    reach(&mut w, 3);
    assert_eq!(w.due(), None);
    let turn = w.turn();
    run(&mut w, "wait");
    assert_eq!(w.turn(), turn + 1);
    // With every talent the character qualifies for held, the pick is dropped too. Apprentice
    // (any skill, three ranks each) would keep a pick open to the last level, and so would Sworn
    // foe (any creature tag) and School focus (any school or element): here they are an
    // elf's, and Apprentice's tree waits for a free talent; Salvager, Unbinder, Overreach, Wand
    // mastery, Channeler, Appraiser's eye, Deep lungs, Second breath, Hard training,
    // Quick study of the body, Early mastery, Gifted (six stats), Late bloomer, Sure hand,
    // Numbing cold, Brimming, and Devotee are held back the same way so the picks fit in
    // the levels there are.
    let mut files = ContentFiles::builtin();
    files.edit_table("talents", |text| {
        text.replace(
            "ranks = 3\ndescription = \"You take up",
            "ranks = 3\nneeds = { race = \"elf\" }\ndescription = \"You take up",
        )
        .replace(
            "ranks = 2\ndescription = \"You know how a bound",
            "ranks = 2\nneeds = { race = \"elf\" }\ndescription = \"You know how a bound",
        )
        .replace(
            "tree = \"weaponcraft\"\ndescription = \"You strike at the binding",
            "tree = \"weaponcraft\"\nneeds = { race = \"elf\" }\ndescription = \"You strike at the binding",
        )
        .replace(
            "tree = \"weaponcraft\"\ndescription = \"You draw more from a wand",
            "tree = \"weaponcraft\"\nneeds = { race = \"elf\" }\ndescription = \"You draw more from a wand",
        )
        .replace(
            "needs = { level = 5 }\neffects = [{ kind = \"rod_cost_percent\"",
            "needs = { level = 5, race = \"elf\" }\neffects = [{ kind = \"rod_cost_percent\"",
        )
        .replace(
            "tree = \"scholar\"\ndescription = \"You can tell a bound",
            "tree = \"scholar\"\nneeds = { race = \"elf\" }\ndescription = \"You can tell a bound",
        )
        .replace(
            "choice = \"tag\"\n",
            "choice = \"tag\"\nneeds = { race = \"elf\" }\n",
        )
        .replace(
            "choice = \"school\"\n",
            "choice = \"school\"\nneeds = { race = \"elf\" }\n",
        )
        .replace(
            "needs = { level = 6 }\neffects = [{ kind = \"study\", overreach_percent",
            "needs = { race = \"elf\" }\neffects = [{ kind = \"study\", overreach_percent",
        )
        .replace(
            "tree = \"vigour\"\ndescription = \"Your stamina holds",
            "tree = \"vigour\"\nneeds = { race = \"elf\" }\ndescription = \"Your stamina holds",
        )
        .replace(
            "tree = \"might\"\ndescription = \"Every kill of yours gives you back",
            "tree = \"might\"\nneeds = { race = \"elf\" }\ndescription = \"Every kill of yours gives you back",
        )
        .replace(
            "needs = { level = 9 }\neffects = [{ kind = \"school_marks_percent\"",
            "needs = { level = 9, race = \"elf\" }\neffects = [{ kind = \"school_marks_percent\"",
        )
        .replace(
            "needs = { level = 9 }\neffects = [{ kind = \"ability_ranks\"",
            "needs = { level = 9, race = \"elf\" }\neffects = [{ kind = \"ability_ranks\"",
        )
        .replace(
            "tree = \"vigour\"\ndescription = \"You drill every day",
            "tree = \"vigour\"\nneeds = { race = \"elf\" }\ndescription = \"You drill every day",
        )
        .replace(
            "tree = \"scholar\"\ndescription = \"What you drink teaches",
            "tree = \"scholar\"\nneeds = { race = \"elf\" }\ndescription = \"What you drink teaches",
        )
        .replace(
            "needs = { level = 12 }\neffects = [{ kind = \"spell_mod\"",
            "needs = { level = 12, race = \"elf\" }\neffects = [{ kind = \"spell_mod\"",
        )
        .replace(
            "needs = { level = 25 }\neffects = [\n    { kind = \"spell_mod\"",
            "needs = { level = 25, race = \"elf\" }\neffects = [\n    { kind = \"spell_mod\"",
        )
        .replace(
            "needs = { level = 20 }\neffects = [{ kind = \"ramp_ease\"",
            "needs = { level = 20, race = \"elf\" }\neffects = [{ kind = \"ramp_ease\"",
        )
        .replace(
            "description = \"Items, scrolls, gems, and learning.\"",
            "description = \"Items, scrolls, gems, and learning.\"\npending = true",
        )
    });
    let content = Arc::new(Content::load(&files).expect("content"));
    let mut w = world_with(content);
    set_scores(&mut w, 1);
    let mut level = 3;
    loop {
        reach(&mut w, level);
        if w.due().is_none() {
            break;
        }
        run(&mut w, "levelup:auto");
        assert_eq!(w.due(), None, "level {level}");
        level += 3;
    }
    assert!(
        player(&w).talents.len() >= 4,
        "the talents without prerequisites (and what the warrior's ranks allow) are held"
    );
    assert_eq!(w.due(), None, "nothing left to take: the pick is dropped");
}

#[test]
fn each_talent_changes_the_rule_it_names() {
    let mut w = world();
    set_scores(&mut w, 18);
    let content = w.content().clone();
    let id = w.state.player;
    let take = |w: &mut World, level: u8, talent: &str| {
        reach(w, level);
        run(w, &format!("talent:{talent}"));
        assert!(last(w).starts_with("You learn "), "{}", last(w));
    };

    // Healthy: hit points come back a quarter faster.
    let rate = w.healing_rate(id);
    take(&mut w, 3, "healthy");
    assert_eq!(w.healing_rate(id), rate * 125 / 100);

    // Dodge and Thick skin: Defence in their categories.
    let defence = |w: &World| player(w).defence(&content, Burden::Unburdened, None);
    let (dodge, natural) = (
        defence(&w).bonus(crate::combat::Category::Dodge),
        defence(&w).bonus(crate::combat::Category::Natural),
    );
    take(&mut w, 6, "dodge");
    take(&mut w, 9, "thick_skin");
    assert_eq!(defence(&w).bonus(crate::combat::Category::Dodge), dodge + 1);
    assert_eq!(
        defence(&w).bonus(crate::combat::Category::Natural),
        natural + 1
    );

    // Brawn: 1 more damage on every blow up close.
    let bonus = player(&w).melee(&content).bonus;
    take(&mut w, 12, "brawn");
    assert_eq!(player(&w).melee(&content).bonus, bonus + 1);

    // Keen ears: sounds 2 fainter.
    let hearing = player(&w).hearing(&content).expect("the player hears");
    take(&mut w, 15, "keen_ears");
    assert_eq!(player(&w).hearing(&content), Some(hearing - 2));

    // Pack mule: a tenth more carrying, kept a multiple of 3 grams.
    let limit = w.load_of(id).limit;
    take(&mut w, 18, "pack_mule");
    assert_eq!(w.load_of(id).limit, limit * 110 / 100 / 3 * 3);

    // Lean: hunger a fraction slower, and exertion free.
    assert_eq!(w.hunger_percent(), 100);
    take(&mut w, 21, "lean");
    assert_eq!(w.hunger_percent(), 85);
    let fed = player(&w).satiation.expect("the player hungers").nutrition;
    w.exert(ActionKind::Melee);
    assert_eq!(player(&w).satiation.expect("hungers").nutrition, fed);

    // Quick learner: one skill roll more every level.
    let rolls = w.level_rolls();
    take(&mut w, 24, "quick_learner");
    assert_eq!(w.level_rolls(), rolls + 1);

    // Alert: notices in passing, 2 better.
    let passive = w.search_modifier(Look::Passive, Hidden::Door(0));
    assert!(
        !w.notices(w.player_pos(), Hidden::Door(0)),
        "a warrior is not trained in Spot"
    );
    take(&mut w, 27, "alert");
    assert!(w.notices(w.player_pos(), Hidden::Door(0)));
    assert_eq!(
        w.search_modifier(Look::Passive, Hidden::Door(0)),
        passive + 2
    );

    // Silent feet: steps a fifth quieter per rank; at rank 2 (Stealth 25) silent doors.
    let level = w.state.current().clone();
    let step = Act::Step(w.player_pos());
    let loud = |w: &World| {
        noise::noise(&content, &level, player(w), Burden::Unburdened, step).map(|n| n.loudness)
    };
    let plain = loud(&w).expect("a step is heard");
    take(&mut w, 30, "silent_feet");
    assert_eq!(loud(&w), Some(plain * 80 / 100));
    reach(&mut w, 33);
    run(&mut w, "talent:silent_feet");
    assert_eq!(
        last(&w),
        "You cannot take Silent feet: you need Stealth 25."
    );
    let stealth = content.skill_id("stealth").expect("stealth");
    w.player_mut().skills.add(stealth, 25);
    run(&mut w, "talent:silent_feet");
    assert_eq!(last(&w), "You learn Silent feet (2).");
    let door = Act::OpenDoor(w.player_pos());
    let mods = player(&w).talent_mods(&content);
    assert!(mods.silent_doors);
    assert_eq!(
        noise::noise(&content, &level, player(&w), Burden::Unburdened, door),
        None,
        "a silent door"
    );
}

#[test]
fn a_talent_may_switch_on_a_free_blow() {
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/talents/test_cleave.toml",
        r#"
[[talent]]
id = "test_cleave"
name = "Test cleave"
tree = "might"
description = "A blow that kills gives you another."
effects = [{ kind = "free_blow", trigger = "cleave" }, { kind = "healing_percent", value = 1 }]
systems = ["free_blows"]
"#,
    );
    let content = Arc::new(Content::load(&files).expect("content"));
    let mut w = world_with(content);
    assert!(w.free_blows_of(player(&w)).is_none());
    reach(&mut w, 3);
    run(&mut w, "talent:test_cleave");
    assert_eq!(last(&w), "You learn Test cleave.");
    let blows = w.free_blows_of(player(&w));
    assert!(blows.cleave && !blows.riposte);
}

#[test]
fn talents_survive_saves_and_replays_and_runs_without_them_hash_as_before() {
    let play = || {
        // Born under the sign the seed gives (a replay restarts the character from its spec),
        // with whatever gifts it grants before the pick.
        let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
        let mut w = World::with_character(BUILTIN.clone(), "showcase", Some(7), Some(&spec))
            .expect("world");
        let mut replay = Replay::new(&w);
        let spec = w.state.character.as_ref().map(|c| c.spec.clone());
        let need =
            (w.content().experience()).threshold_for(3, Factor::of(w.content(), spec.as_ref()));
        for action in
            parse_actions(&format!("xp:{need} talent:healthy move:E wait")).expect("actions")
        {
            w.apply(&action);
            replay.push(&action, &w);
        }
        (w, replay)
    };
    let (a, replay) = play();
    // The sign's gifts and the race's Versatile, then the pick.
    let gifts = (a.character()).map_or(0, |c| c.grants(a.content()).len()) + 1;
    assert_eq!(player(&a).talents.len(), gifts + 1);
    let content = a.content().clone();
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), a.state_hash());
    let bytes = crate::save::encode(&a);
    let loaded = crate::save::decode(&bytes, content).expect("loads");
    assert_eq!(loaded.state(), a.state());
    assert_eq!(loaded.perception().own_talents()[gifts].name, "Healthy");

    // A run that picked nothing carries none of it: no stream, no dues, no talents.
    let mut w = world();
    run(&mut w, "move:E wait");
    let json = serde_json::to_string(w.state()).expect("json");
    assert!(
        !json.contains("levels_rng"),
        "the levels stream is made on its first draw"
    );
    assert!(!json.contains("\"due\""), "{json}");
    assert!(!json.contains("\"talents\""), "{json}");
}

#[test]
fn perception_and_dumps_show_talents_in_words_only() {
    let mut w = world();
    set_scores(&mut w, 10);
    reach(&mut w, 3);
    // While the pick is due: the progress layer says so, with what may be taken.
    let progress = dump::ascii(&w, &[Layer::Progress]);
    assert!(progress.contains("due level 3: 1 talent"), "{progress}");
    for free in ["healthy", "keen_ears", "pious", "lean"] {
        assert!(progress.contains(free), "{progress}");
    }
    assert!(
        progress.contains(
            "takeable deep_lungs, \
             hard_training, healthy, "
        ),
        "{progress}"
    );
    let view = w.perception();
    assert_eq!(view.due().map(|d| d.level), Some(3));
    let talents = view.talents();
    let dodge = talents.iter().find(|t| t.name == "Dodge").expect("dodge");
    assert!(dodge.openings.is_empty());
    assert_eq!(dodge.needs.len(), 1);
    assert_eq!(dodge.needs[0].words, "Dexterity 13");
    assert!(!dodge.needs[0].met);
    let healthy = talents
        .iter()
        .find(|t| t.name == "Healthy")
        .expect("healthy");
    assert_eq!(healthy.openings, [None]);
    assert!(healthy.needs.is_empty());
    assert_eq!(healthy.held, 0);
    assert_eq!(
        talents[0].tree,
        w.content().talents().tree_id("stars").expect("stars"),
        "trees in display order, the birth signs' first"
    );

    run(&mut w, "talent:healthy");
    reach(&mut w, 6);
    set_score(&mut w, Stat::Toughness, 11);
    run(&mut w, "talent:hardy");
    let view = w.perception();
    let own: Vec<String> = view.own_talents().iter().map(|t| t.name.clone()).collect();
    assert_eq!(own, ["Healthy", "Hardy"]);
    let hardy = (view.talents().into_iter())
        .find(|t| t.name == "Hardy")
        .expect("hardy");
    assert_eq!((hardy.held, hardy.ranks), (1, 3));
    assert_eq!(
        hardy.needs[0].words, "Toughness 14",
        "the next rank's prerequisites"
    );
    assert_eq!(
        dump::ascii(&w, &[Layer::Talents])
            .lines()
            .rfind(|l| !l.is_empty()),
        Some("Hardy 1/3")
    );
    let json = dump::json(&w, &[Layer::View], false);
    assert_eq!(json["talents"][1]["id"], "hardy");
    assert_eq!(json["talents"][1]["rank"], 1);
    assert_eq!(json["talents"][0]["choice"], serde_json::Value::Null);
    assert!(json["due"].is_null());
}
