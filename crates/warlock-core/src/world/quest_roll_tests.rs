//! Rolled quests and the giving rules' named monsters (spec §8 "Quests", #422; the guild design
//! §5, §8.4, §11): a roll drawn as the quest is given, the same for the same seed and state,
//! never drawn again, kept by saves and replays; only branches the player can enter are drawn;
//! kills before the giving do not count; a variant reveals its own location only; the dev
//! `quest_roll`; a named monster's death counts at any hand and any time; the load checks of
//! picks, variants, and placeholders; Quarry sense and Sworn foe.

use super::fight::Harm;
use super::*;
use crate::action::parse_actions;
use crate::content::{ContentFiles, TrapId};
use crate::dump::{self, Layer};
use crate::replay::Replay;

/// Godgifu, in the Gæsthūs: she gives the test quests.
const GODGIFU: Pos = Pos::new(46, 26);

const BOUNTY: &str = r#"
[[quest]]
id = "bounty"
name = "A bounty"
giver = "godgifu"
type = "kill"
pick = { creatures = ["cealdhlaw", "fennmynster", "fyrdelf"] }
reward = { gold = 40, xp = 60 }
offer = "There's a price on {a_target}. Counts from today, mind. {Where}"
reminder = "{A_target}. {Where}"
done = "That's the one."
where = "{Targets} are known in {branch}."

[[quest]]
id = "hundred"
name = "The hundred"
giver = "godgifu"
type = "kill"
count = 100
pick = { tags = ["kobold", "goblin"] }
offer = "A hundred {targets}: {members}."
reminder = "{Count} {targets}."
done = "Done."

[[quest]]
id = "wanted"
name = "Wanted"
giver = "godgifu"
type = "kill"
offer = "{target}?"
reminder = "Still {target}."
done = "Done."

[[quest.variant]]
id = "fen"
target = "giant_leech"
reveals = { location = "fennmynster" }
where = "In the fen."
offer = "The leeches of the fen. Go."

[[quest.variant]]
id = "fyr"
target = "fire_drake"
weight = 3
reveals = { location = "fyrdelf" }
where = "Under the mountain."
"#;

fn content_with(quests: &str) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    files.insert("campaigns/ascmearc/data/quests/roll_test.toml", quests);
    Arc::new(Content::load(&files).expect("test content is valid"))
}

/// A run in Cēaptūn with the test quests, `seed` its seed, the player beside Godgifu.
fn town(seed: u64) -> World {
    let mut w = World::new(content_with(BOUNTY), "ceaptun", Some(seed)).expect("a run in Cēaptūn");
    let id = w.state.player;
    w.state.actors.get_mut(&id).expect("player").pos = GODGIFU.step(Dir::W);
    w.perceive(&mut Vec::new());
    w
}

fn act(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.text.clone()).collect()
}

fn roll(world: &World, id: &str) -> Option<crate::content::Roll> {
    world.quests().record(id).and_then(|r| r.roll.clone())
}

fn logged(world: &World, id: &str) -> QuestView {
    (world.perception().quests().into_iter())
        .find(|q| q.id == id)
        .expect("in the log")
}

fn def<'a>(world: &'a World, id: &str) -> &'a crate::content::QuestDef {
    let content = world.content();
    content.quest(content.quest_id(id).expect("a quest"))
}

/// The creature types of branch `branch`'s population.
fn population(world: &World, branch: &str) -> Vec<String> {
    let content = world.content();
    let spawning = (content.dungeon(branch).and_then(|d| d.spawning.as_ref())).expect("lives");
    (spawning.population.iter())
        .map(|&c| content.creature(c).id.clone())
        .collect()
}

#[test]
fn a_roll_is_drawn_as_the_quest_is_given_and_never_again() {
    let mut w = town(11);
    assert_eq!(roll(&w, "bounty"), None);
    act(&mut w, "ask:E:quest-bounty");
    let first = roll(&w, "bounty").expect("drawn as it is given");
    assert_eq!((first.variant.as_deref(), first.count), (None, 1));
    // Of Cealdhlāw's population, the one branch open, and no unique.
    assert!(
        population(&w, "cealdhlaw").contains(&first.target),
        "{first:?}"
    );
    let c = w.content().creature_id(&first.target).expect("a type");
    assert!(!w.content().creature(c).unique);
    // Its words are filled; the log has the first breadcrumb and where.
    let name = w.content().creature(c).name.clone();
    let plural = w.content().creature(c).plural.clone();
    let offer = said(&w)
        .into_iter()
        .rev()
        .find(|m| m.contains("says:"))
        .expect("said");
    assert!(offer.contains(&format!(
        "a price on {} {name}.",
        crate::content::article(&name)
    )));
    assert!(!offer.contains('{'), "{offer}");
    let mut capital = plural.chars();
    let capital: String = capital
        .next()
        .into_iter()
        .flat_map(char::to_uppercase)
        .chain(capital)
        .collect();
    assert_eq!(
        logged(&w, "bounty").objective,
        format!(
            "Godgifu wants you to kill {} {name}. {capital} are known in Cealdhlāw.",
            crate::content::article(&name)
        )
    );
    assert_eq!(
        logged(&w, "bounty").target.as_deref(),
        Some(first.target.as_str())
    );
    // Asked again: the reminder, and the roll stands.
    act(&mut w, "ask:E:quest-bounty");
    assert_eq!(roll(&w, "bounty").as_ref(), Some(&first));
    // The same seed and state draw the same.
    let mut again = town(11);
    act(&mut again, "ask:E:quest-bounty");
    assert_eq!(roll(&again, "bounty"), Some(first.clone()));
    // A save keeps it.
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(roll(&loaded, "bounty"), Some(first));
    assert_eq!(loaded.perception().quests(), w.perception().quests());
}

#[test]
fn rolls_survive_a_replay_exactly() {
    // The bounty given by Sister Hild, on the way the chalice's replay walks to her.
    let content = content_with(&BOUNTY.replacen("giver = \"godgifu\"", "giver = \"hild\"", 1));
    let mut w = World::new(content, "ceaptun", Some(5)).expect("a run in Cēaptūn");
    let mut replay = Replay::new(&w);
    let actions = "move:SW*4 move:W*15 open:SW move:SW move:S*2 ask:SE:quest-bounty wait \
                   ask:SE:quest-bounty wait";
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(w.content().clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    assert!(roll(&w, "bounty").is_some());
    assert_eq!(roll(&verified, "bounty"), roll(&w, "bounty"));
}

#[test]
fn only_the_branches_the_player_can_enter_are_drawn() {
    let mut w = town(1);
    let content = w.content().clone();
    let open = population(&w, "cealdhlaw");
    let fen = population(&w, "fennmynster");
    let unique = |c: &String| {
        let c = content.creature_id(c).expect("a type");
        content.creature(c).unique
    };
    let only_fen: Vec<&String> = (fen.iter())
        .filter(|c| !open.contains(c) && !unique(c))
        .collect();
    assert!(!only_fen.is_empty(), "the fen has its own");
    let pool = |w: &World| -> Vec<String> {
        (w.quest_pool(def(w, "bounty")).into_iter())
            .map(|c| c.target)
            .collect()
    };
    assert!(pool(&w).iter().all(|t| open.contains(t)), "{:?}", pool(&w));
    assert!(w.branch_open("cealdhlaw") && !w.branch_open("fennmynster"));
    // Revealed, the minster's creatures are drawn too.
    act(&mut w, "reveal:fennmynster");
    assert!(w.branch_open("fennmynster"));
    let now = pool(&w);
    assert!(only_fen.iter().all(|c| now.contains(c)), "{now:?}");
    // Uniques never are.
    assert!((now.iter()).all(|t| {
        let c = content.creature_id(t).expect("a type");
        !content.creature(c).unique
    }));
}

#[test]
fn kills_before_the_giving_do_not_count() {
    let mut w = town(3);
    // Two of every type the bounty may draw, killed before it is given.
    for target in (w.quest_pool(def(&w, "bounty")).into_iter()).map(|c| c.target) {
        let c = w.content().creature_id(&target).expect("a type");
        w.tally_kill(c, "cealdhlaw:1");
        w.tally_kill(c, "cealdhlaw:1");
    }
    act(&mut w, "ask:E:quest-bounty");
    assert_eq!(logged(&w, "bounty").status, "0 / 1 killed.");
    let drawn = roll(&w, "bounty").expect("drawn").target;
    let c = w.content().creature_id(&drawn).expect("a type");
    w.tally_kill(c, "cealdhlaw:2");
    assert_eq!(logged(&w, "bounty").status, "Return to Godgifu.");
}

#[test]
fn a_tag_pick_names_its_members() {
    let mut w = town(2);
    act(&mut w, "ask:E:quest-hundred");
    let drawn = roll(&w, "hundred").expect("drawn");
    assert!(
        ["tag:goblin", "tag:kobold"].contains(&drawn.target.as_str()),
        "{drawn:?}"
    );
    assert_eq!(drawn.count, 100);
    let objective = logged(&w, "hundred").objective;
    let expected = if drawn.target == "tag:kobold" {
        "Godgifu wants you to kill 100 kobolds: kobolds, kobold archers, and kobold shamans."
    } else {
        "Godgifu wants you to kill 100 goblins:"
    };
    assert!(objective.starts_with(expected), "{objective}");
}

#[test]
fn a_variant_reveals_its_own_location_and_no_other() {
    for seed in 0..6 {
        let mut w = town(seed);
        act(&mut w, "ask:E:quest-wanted");
        let drawn = roll(&w, "wanted").expect("drawn");
        let (mine, other) = match drawn.variant.as_deref() {
            Some("fen") => ("fennmynster", "fyrdelf"),
            Some("fyr") => ("fyrdelf", "fennmynster"),
            v => panic!("no variant: {v:?}"),
        };
        assert!(w.state.revealed.contains(mine), "{seed}: {drawn:?}");
        assert!(!w.state.revealed.contains(other), "{seed}: {drawn:?}");
        let name = if mine == "fyrdelf" {
            "Fyrdelf"
        } else {
            "Fennmynster"
        };
        assert_eq!(
            said(&w).last().unwrap(),
            &format!("Godgifu marks {name} on your map.")
        );
        // The variant's own words, or the quest's, filled.
        let objective = logged(&w, "wanted").objective;
        if mine == "fyrdelf" {
            assert_eq!(
                objective,
                "Godgifu wants you to kill the fire drake. Under the mountain."
            );
        } else {
            assert_eq!(
                objective,
                "Godgifu wants you to kill a giant leech. In the fen."
            );
            assert!(
                said(&w)
                    .iter()
                    .any(|m| m.contains("The leeches of the fen. Go."))
            );
        }
        // Out on the overworld: the one location, by its entrance, and not the other.
        let id = w.state.player;
        w.state.actors.get_mut(&id).expect("player").pos = Pos::new(45, 17);
        act(&mut w, "ascend");
        let known = dump::layer_rows(&w, Layer::Locations).join("\n");
        assert!(known.contains(name), "{known}");
        let other_name = if other == "fyrdelf" {
            "Fyrdelf"
        } else {
            "Fennmynster"
        };
        assert!(!known.contains(other_name), "{known}");
        let truth = dump::layer_rows(&w, Layer::Truth).join("\n");
        let view = dump::layer_rows(&w, Layer::View).join("\n");
        assert_ne!(truth, view);
    }
}

#[test]
fn the_dev_roll_replaces_a_roll_without_progress() {
    let mut w = town(4);
    let before = w.quests().clone();
    act(&mut w, "quest_roll:wanted:fen");
    assert_eq!(w.quests(), &before, "refused: not given yet");
    act(&mut w, "ask:E:quest-wanted");
    act(&mut w, "quest_roll:wanted:nobody");
    assert!(said(&w).last().unwrap().contains("cannot draw"));
    act(&mut w, "quest_roll:wanted:fyr");
    assert_eq!(
        roll(&w, "wanted").and_then(|r| r.variant).as_deref(),
        Some("fyr")
    );
    assert!(w.state.revealed.contains("fyrdelf"));
    act(&mut w, "quest_roll:wanted:fen");
    assert_eq!(
        roll(&w, "wanted").and_then(|r| r.variant).as_deref(),
        Some("fen")
    );
    assert!(w.state.revealed.contains("fennmynster"));
    // A pick's target by its id, kills counted from now.
    act(&mut w, "ask:E:quest-bounty quest_roll:bounty:rat");
    assert_eq!(roll(&w, "bounty").map(|r| r.target).as_deref(), Some("rat"));
    let rat = w.content().creature_id("rat").expect("rat");
    w.tally_kill(rat, "cealdhlaw:1");
    let after = w.quests().clone();
    act(&mut w, "quest_roll:bounty:bat");
    assert_eq!(w.quests(), &after, "refused: it has got somewhere");
    assert_eq!(logged(&w, "bounty").status, "Return to Godgifu.");
}

// A named monster (a unique type): its death counts, at any hand and whenever it was.

const NAMED: &str = r#"
[[quest]]
id = "lich"
name = "The lich"
giver = "shopkeeper"
type = "kill"
target = "demilich"
offer = "The lich."
reminder = "The lich."
done = "Gone."
"#;

/// The arena with the demilich quest, and a kobold made the demilich (a unique type).
fn arena() -> (World, ActorId) {
    let mut files = ContentFiles::builtin();
    files.insert("data/quests/named_test.toml", NAMED);
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let mut w = World::new(content.clone(), "arena", None).expect("run");
    let kobold = content.creature_id("kobold").expect("kobold");
    let lich = content.creature_id("demilich").expect("demilich");
    let id = (w.state.actors.values())
        .find(|a| a.creature == kobold)
        .expect("a kobold")
        .id;
    w.state.actors.get_mut(&id).expect("it").creature = lich;
    (w, id)
}

fn given(w: &mut World) {
    let record = QuestRecord {
        stage: Stage::Active,
        completed: 0,
        counted: 0,
        reached: false,
        roll: None,
    };
    w.state.quests.records.insert("lich".into(), record);
}

fn lich_status(w: &World) -> String {
    logged(w, "lich").status
}

#[test]
fn a_named_monster_killed_by_a_trap_counts() {
    let (mut w, lich) = arena();
    given(&mut w);
    assert_eq!(lich_status(&w), "0 / 1 killed.");
    assert!(w.hurt(lich, 100_000, Harm::Trap(TrapId(0)), &mut Vec::new()));
    assert_eq!(lich_status(&w), "Return to the shopkeeper.");
    assert_eq!(w.quests().slain().collect::<Vec<_>>(), ["demilich"]);
    assert_eq!(w.quests().kills().count(), 0, "not the player's kill");
}

#[test]
fn a_named_monster_killed_by_another_creature_counts() {
    let (mut w, lich) = arena();
    given(&mut w);
    let rat = w.content().creature_id("rat").expect("rat");
    let by = (w.state.actors.values())
        .find(|a| a.creature == rat)
        .expect("a rat")
        .id;
    assert!(w.hurt(lich, 100_000, Harm::Actor(by), &mut Vec::new()));
    assert_eq!(lich_status(&w), "Return to the shopkeeper.");
}

#[test]
fn a_named_monster_dead_before_the_quest_is_given_counts_at_once() {
    let (mut w, lich) = arena();
    assert!(w.hurt(lich, 100_000, Harm::Trap(TrapId(0)), &mut Vec::new()));
    assert!(
        w.quests().slain().any(|s| s == "demilich"),
        "kept without a quest"
    );
    given(&mut w);
    assert_eq!(lich_status(&w), "Return to the shopkeeper.");
    // Saved and restored.
    let restored = World::from_state(w.content().clone(), w.state().clone()).expect("restores");
    assert_eq!(lich_status(&restored), "Return to the shopkeeper.");
}

#[test]
fn wulfruns_drake_dead_first_meets_the_quest_as_it_is_given() {
    let mut w = town(1);
    w.state.quests = serde_json::from_str(
        r#"{"records": {"foremans_ledger": {"stage": "done", "completed": 1},
            "barrow_rats": {"stage": "done", "completed": 1},
            "barrow_heart": {"stage": "done", "completed": 1},
            "lost_shaft": {"stage": "done", "completed": 1}},
            "slain": ["fire_drake"]}"#,
    )
    .expect("records");
    let id = w.state.player;
    w.state.actors.get_mut(&id).expect("player").pos = Pos::new(71, 26);
    w.perceive(&mut Vec::new());
    act(&mut w, "ask:E:quest-fire_deep");
    assert_eq!(
        said(&w).last().unwrap(),
        "Quest updated: 1 / 1 fire drake. Return to Factor Wulfrun."
    );
    assert_eq!(logged(&w, "fire_deep").status, "Return to Factor Wulfrun.");
}

// Load checks.

#[test]
fn picks_variants_and_placeholders_are_checked_at_load() {
    let bad = |quests: &str, expect: &str| {
        let mut files = ContentFiles::builtin();
        files.insert("campaigns/ascmearc/data/quests/roll_test.toml", quests);
        let err = Content::load(&files).expect_err(quests);
        assert!(
            err.to_string().contains(expect),
            "{err} (expected {expect:?})"
        );
    };
    let quest = |body: &str, offer: &str| {
        format!(
            "[[quest]]\nid = \"q\"\nname = \"Q\"\ngiver = \"godgifu\"\n{body}\noffer = \"{offer}\"\n\
             reminder = \"R.\"\ndone = \"D.\"\n"
        )
    };
    let kill = "type = \"kill\"\npick = { creatures = [\"cealdhlaw\"] }";
    // A filled text over 280 characters, for some target of the pool.
    let long = format!("{} {{target}}.", "x".repeat(270));
    bad(&quest(kill, &long), "at most 280 characters");
    bad(&quest(kill, "{nobody}"), "unknown placeholder `{nobody}`");
    bad(&quest(kill, "{target"), "unclosed placeholder");
    bad(
        &quest("type = \"kill\"\ntarget = \"rat\"", "{target}"),
        "placeholders (`{target}`) are for a quest with a `pick` or variants",
    );
    bad(
        &quest(
            "type = \"kill\"\ntarget = \"rat\"\npick = { creatures = [\"cealdhlaw\"] }",
            "O.",
        ),
        "exactly one of `target`, `pick`, and variants",
    );
    bad(
        &quest(
            "type = \"bring\"\npick = { creatures = [\"cealdhlaw\"] }",
            "O.",
        ),
        "a pick of creatures is for a kill quest",
    );
    bad(
        &quest("type = \"kill\"\npick = { creatures = [\"moon\"] }", "O."),
        "unknown dungeon branch `moon`",
    );
    bad(
        &quest("type = \"kill\"\npick = { tags = [\"dragonish\"] }", "O."),
        "no creature carries it",
    );
    bad(
        &quest(
            "type = \"bring\"\npick = { objects = { class = \"trinkets\" } }",
            "O.",
        ),
        "unknown item class `trinkets`",
    );
    bad(
        &quest(
            "type = \"reach\"\npick = { creatures = [\"cealdhlaw\"] }",
            "O.",
        ),
        "is for a kill quest",
    );
    bad(
        &quest("type = \"kill\"\ntarget = \"fire_drake\"\ncount = 2", "O."),
        "a unique is killed once",
    );
    let variants = |extra: &str| {
        format!(
            "{}\n[[quest.variant]]\nid = \"a\"\ntarget = \"rat\"\n{extra}",
            quest("type = \"kill\"", "O.")
        )
    };
    bad(&variants("weight = 0"), "`weight` is 1 to 1000");
    bad(
        &variants("reveals = { location = \"atlantis\" }"),
        "no hidden location `atlantis`",
    );
    bad(
        &variants("\n[[quest.variant]]\nid = \"a\"\ntarget = \"bat\""),
        "a second variant",
    );
    bad(&variants("target2 = \"x\""), "unknown field");
    // Fits: each variant's own words stand in for the quest's.
    let mut files = ContentFiles::builtin();
    files.insert(
        "campaigns/ascmearc/data/quests/roll_test.toml",
        "[[quest]]\nid = \"q\"\nname = \"Q\"\ngiver = \"godgifu\"\ntype = \"kill\"\n\
         [[quest.variant]]\nid = \"a\"\ntarget = \"rat\"\noffer = \"A.\"\nreminder = \"R.\"\n\
         done = \"D.\"\n",
    );
    assert!(Content::load(&files).is_ok());
}

// The bounties system's talents.

#[test]
fn quarry_sense_tells_of_a_quarry_on_arrival() {
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/quests/quarry_test.toml",
        "[[quest]]\nid = \"rat_cull\"\nname = \"Rats\"\ngiver = \"shopkeeper\"\ntype = \"kill\"\n\
         target = \"rat\"\ncount = 50\noffer = \"O.\"\nreminder = \"R.\"\ndone = \"D.\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let arrive = |talent: bool, given: bool| {
        let mut w = World::new(content.clone(), "showcase", None).expect("run");
        if given {
            w.state.quests =
                serde_json::from_str(r#"{"records": {"rat_cull": {"stage": "active"}}}"#)
                    .expect("records");
        }
        if talent {
            w.player_mut().talents.take("quarry_sense", None);
        }
        w.travel("arena", None).expect("the arena");
        said(&w).contains(&"Your quarry is here.".to_string())
    };
    assert!(arrive(true, true));
    assert!(!arrive(false, true), "not without the talent");
    assert!(!arrive(true, false), "not without the quest");
}

#[test]
fn sworn_foe_hits_its_kind_surer_and_harder() {
    let content = Arc::new(Content::builtin().expect("content"));
    let mut w = World::new(content.clone(), "arena", None).expect("run");
    let player = w.state.player;
    let first = |w: &World, name: &str| {
        let c = content.creature_id(name).expect("a type");
        (w.state.actors.values())
            .find(|a| a.creature == c)
            .expect("one")
            .id
    };
    let (kobold, rat) = (first(&w, "kobold_archer"), first(&w, "rat"));
    let profile = w.state.player().melee(&content);
    assert_eq!(w.favoured(player, kobold, profile.clone()), profile);
    w.player_mut().talents.take("sworn_foe", Some("kobold"));
    let sworn = w.favoured(player, kobold, profile.clone());
    assert_eq!(
        (sworn.to_hit, sworn.bonus),
        (profile.to_hit + 1, profile.bonus + 2)
    );
    assert_eq!(w.favoured(player, rat, profile.clone()), profile);
    // Its choices are the creature tags.
    let def = content
        .talents()
        .get(content.talents().id("sworn_foe").expect("a talent"));
    let options = def.choice.expect("a choice").options(&content);
    assert!(options.contains(&"kobold".to_string()) && options.contains(&"undead".to_string()));
}
