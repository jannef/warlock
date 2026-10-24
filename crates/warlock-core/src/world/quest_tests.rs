//! Quests (spec §8 "Quests", #148) in the turn pipeline: offers, accepting and declining, each
//! kind of goal end to end (kill, bring, reach, deliver), prerequisites, rewards with XP,
//! repeats, no cap on active quests, the quest log, saves, and replays; then the showcase's own
//! quests.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::replay::Replay;

const TERRAIN: &str = r##"
[[terrain]]
id = "floor"
name = "floor"
glyph = "."
fg = "#808080"

[[terrain]]
id = "wall"
name = "wall"
glyph = "#"
fg = "#a0a0a0"
walkable = false
transparent = false

[[terrain]]
id = "stairs_down"
name = "staircase down"
glyph = ">"
fg = "#ffffff"
stairs = "down"

[[terrain]]
id = "stairs_up"
name = "staircase up"
glyph = "<"
fg = "#ffffff"
stairs = "up"
"##;

const FACTIONS: &str = r##"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "folk"
name = "folk"

[[faction]]
id = "pests"
name = "pests"
hostile = ["player"]

[[faction]]
id = "hunters"
name = "hunters"
hostile = ["pests"]
"##;

// The pup dies of a scratch; the hunter kills pups.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "elder"
name = "elder"
glyph = "@"
fg = "#c0c0ff"
faction = "folk"
weight = 60000

[[creature]]
id = "crone"
name = "crone"
glyph = "@"
fg = "#c0c0c0"
faction = "folk"
weight = 50000

[[creature]]
id = "pup"
name = "pup"
glyph = "p"
fg = "#806040"
weight = 5000
faction = "pests"
stats = { toughness = 1, dexterity = 1 }
level = 1
xp = 10

[[creature]]
id = "hunter"
name = "hunter"
glyph = "h"
fg = "#60a040"
weight = 20000
faction = "hunters"
stats = { strength = 30, dexterity = 20 }
attack = { damage = "1d10", name = "claw", verb = "claws" }
"##;

const MATERIALS: &str = r##"
[[material]]
id = "plant"
name = "plant"

[[material]]
id = "paper"
name = "paper"

[[material]]
id = "gold"
name = "gold"
"##;

const OBJECTS: &str = r##"
[[object]]
id = "apple"
name = "apple"
glyph = "%"
fg = "#d84838"
class = "food"
materials = ["plant"]
weight = 150
value = 0
stackable = true

[[object]]
id = "letter"
name = "letter"
glyph = "?"
fg = "#e0d2a8"
class = "scrolls"
materials = ["paper"]
weight = 20
value = 0

[[object]]
id = "gold_coin"
name = "gold coin"
glyph = "$"
fg = "#f0c040"
class = "coins"
materials = ["gold"]
weight = 10
value = 0
stackable = true

[[object]]
id = "charm"
name = "charm"
glyph = '"'
fg = "#c0c0ff"
class = "amulets"
materials = ["gold"]
weight = 10
value = 0

[[object]]
id = "rock"
name = "rock"
glyph = "*"
fg = "#808080"
class = "miscellaneous"
materials = ["plant"]
weight = 100
value = 0
"##;

const DIALOGUE: &str = r##"
[[dialogue]]
creature = "elder"
lines = ["Hm."]

[[dialogue.topic]]
id = "weather"
name = "The weather"
reply = "Fair."

[[dialogue]]
creature = "crone"
lines = ["Eh."]
"##;

const QUESTS: &str = r##"
[[quest]]
id = "pups"
name = "Pups"
giver = "elder"
type = "kill"
target = "pup"
count = 2
level = "yard"
reward = { gold = 5, xp = 50 }
offer = "Kill two pups."
accept = "Good."
reminder = "The pups."
done = "Thank you."

[[quest]]
id = "cellar"
name = "The cellar"
giver = "elder"
requires = "pups"
type = "reach"
target = "cellar"
reward = { xp = 10 }
offer = "Go down to the cellar."
accept = "Off you go."
decline = "Pity."
reminder = "The cellar."
done = "So it's still there."

[[quest]]
id = "parcel"
name = "The parcel"
giver = "elder"
type = "deliver"
target = "letter"
to = "crone"
reward = { gold = 3 }
offer = "Take this to the crone."
accept = "Here it is."
reminder = "The crone is waiting."
done = "From the elder? Thank you, dear."

[[quest]]
id = "apples"
name = "Apples"
giver = "crone"
type = "bring"
target = "apple"
count = 2
reward = { items = ["charm"], xp = 5 }
repeatable = true
offer = "Two apples, dear."
accept = "Lovely."
reminder = "My apples?"
done = "Lovely apples."
"##;

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
"@" = { terrain = "floor", spawn = "player" }
"E" = { terrain = "floor", creature = "elder" }
"C" = { terrain = "floor", creature = "crone" }
"p" = { terrain = "floor", creature = "pup" }
"h" = { terrain = "floor", creature = "hunter" }
"a" = { terrain = "floor", objects = ["apple"] }
"r" = { terrain = "floor", objects = ["rock*26"] }
">" = { terrain = "stairs_down", to = "cellar" }
"<" = { terrain = "stairs_up", to = "yard" }
"##;

/// The yard: the player between the elder (west) and the crone (east), apples to the south, and
/// stairs down to the cellar.
const YARD: &str = "\
#########
#E@C....#
#aaa..>.#
#########
";

/// The cellar below, with a pup of its own.
const CELLAR: &str = "\
#####
#<.p#
#####
";

/// Content with `quests` (the fixture's by default), and the yard on `map`.
fn files_with(map: &str, quests: &str) -> ContentFiles {
    let mut files = ContentFiles::default();
    files.insert(
        "data/item_classes.toml",
        crate::content::BUILTIN_ITEM_CLASSES,
    );
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/factions.toml", FACTIONS),
        ("data/creatures.toml", CREATURES),
        ("data/materials.toml", MATERIALS),
        ("data/objects.toml", OBJECTS),
        ("data/dialogue.toml", DIALOGUE),
        ("data/quests.toml", quests),
        ("data/legend.toml", LEGEND),
        (
            "data/experience.toml",
            include_str!("../../../../assets/data/experience.toml"),
        ),
        (
            "data/vision.toml",
            "[[vision]]\nid = \"normal\"\nname = \"Normal\"\nthreshold = 40\n",
        ),
    ] {
        files.insert(path, text);
    }
    let level = |name: &str, map: &str| {
        format!("name = \"{name}\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n")
    };
    files.insert("levels/yard.toml", &level("The Yard", map));
    files.insert("levels/cellar.toml", &level("The Cellar", CELLAR));
    files.insert(
        "scenarios/yard.toml",
        "name = \"Yard\"\nstart_level = \"yard\"\nhp = 30\nstats = { strength = 30 }\n",
    );
    files
}

fn world_on(map: &str) -> World {
    let content = Content::load(&files_with(map, QUESTS)).expect("test content is valid");
    World::new(Arc::new(content), "yard", None).expect("world")
}

fn run(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn ask(world: &mut World, dir: Dir, topic: &str) -> Outcome {
    world.apply(&Action::Ask {
        dir,
        topic: topic.into(),
    })
}

fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.text.clone()).collect()
}

/// The last `n` messages.
fn last(world: &World, n: usize) -> Vec<String> {
    let all = said(world);
    all[all.len().saturating_sub(n)..].to_vec()
}

/// The messages from the last one that is `first` on, at most `n`.
fn from(world: &World, first: &str, n: usize) -> Vec<String> {
    let all = said(world);
    let i =
        (all.iter().rposition(|m| m == first)).unwrap_or_else(|| panic!("{first:?} in {all:?}"));
    all[i..].iter().take(n).cloned().collect()
}

/// The state but its message log: what a refused or idle answer leaves as it was.
fn quiet_state(world: &World) -> WorldState {
    let mut state = world.state().clone();
    state.messages = Default::default();
    state
}

/// The topic ids of the first creature of type `creature` on the level.
fn topics_of(world: &World, creature: &str) -> Vec<String> {
    let c = world.content().creature_id(creature).expect("creature");
    let id = (world.state.actors.values())
        .find(|a| a.creature == c)
        .expect("there is one")
        .id;
    world.topics(id).into_iter().map(|t| t.id).collect()
}

fn stage(world: &World, quest: &str) -> Option<Stage> {
    world.quests().record(quest).map(|r| r.stage)
}

/// How many of `object` the player carries.
fn carried(world: &World, object: &str) -> u32 {
    let o = world.content().object_id(object).expect("object");
    (world.state.player().inventory.iter())
        .filter(|s| s.object == o)
        .map(|s| s.count)
        .sum()
}

fn creatures_left(world: &World, creature: &str) -> usize {
    let c = world.content().creature_id(creature).expect("creature");
    world
        .state
        .actors
        .values()
        .filter(|a| a.creature == c)
        .count()
}

/// Puts the player on `at` (no time passes) and perceives.
fn put_player(world: &mut World, at: Pos) {
    let id = world.state.player;
    world.state.actors.get_mut(&id).expect("player").pos = at;
    world.perceive(&mut Vec::new());
}

/// Accepts quest `id` from the creature in `dir`: asks about it, then accepts.
fn take(world: &mut World, dir: Dir, id: &str) {
    ask(world, dir, &format!("quest-{id}"));
    ask(world, dir, &format!("accept-{id}"));
    assert_eq!(
        stage(world, id),
        Some(Stage::Active),
        "{:?}",
        last(world, 3)
    );
}

/// Moves east into pups until `n` of them are dead (each blow may miss).
fn kill_pups_east(world: &mut World, n: usize) {
    let start = creatures_left(world, "pup");
    for _ in 0..300 {
        if start - creatures_left(world, "pup") >= n {
            return;
        }
        world.apply(&Action::Move(Dir::E));
    }
    panic!("the pups did not die: {:?}", last(world, 6));
}

#[test]
fn an_offer_waits_for_accept_or_decline_and_takes_no_time() {
    let mut w = world_on(YARD);
    // Quest topics come first; one the player may not take yet (the cellar) is not there.
    assert_eq!(
        topics_of(&w, "elder"),
        ["quest-pups", "quest-parcel", "weather"]
    );
    let (turn, time) = (w.turn(), w.time());
    let outcome = ask(&mut w, Dir::W, "quest-pups");
    assert_eq!(last(&w, 1), ["The elder says: \"Kill two pups.\""]);
    let Some(Event::Talked(talk)) = outcome.events.last() else {
        panic!("no answer: {:?}", outcome.events);
    };
    let ids: Vec<&str> = talk.topics.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(
        ids,
        ["accept-pups", "decline-pups", "quest-parcel", "weather"]
    );
    let names: Vec<&str> = talk.topics.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names[..2], ["Accept: Pups", "Decline: Pups"]);
    assert_eq!(stage(&w, "pups"), Some(Stage::Offered));
    // Declining forgets the offer; asking again offers it again.
    ask(&mut w, Dir::W, "decline-pups");
    assert_eq!(last(&w, 1), ["The elder says: \"Another time, then.\""]);
    assert_eq!(stage(&w, "pups"), None);
    assert_eq!(w.quests(), &Quests::default());
    // One offer at a time: hearing another withdraws the first.
    ask(&mut w, Dir::W, "quest-pups");
    ask(&mut w, Dir::W, "quest-parcel");
    assert_eq!(stage(&w, "pups"), None);
    assert_eq!(stage(&w, "parcel"), Some(Stage::Offered));
    // A topic the elder does not have now is refused, changing nothing.
    let before = quiet_state(&w);
    ask(&mut w, Dir::W, "accept-pups");
    assert_eq!(last(&w, 1), ["The elder knows nothing about that."]);
    ask(&mut w, Dir::E, "quest-pups");
    assert_eq!(last(&w, 1), ["The crone knows nothing about that."]);
    assert!(quiet_state(&w) == before);
    ask(&mut w, Dir::W, "accept-parcel");
    assert_eq!(stage(&w, "parcel"), Some(Stage::Active));
    assert_eq!((w.turn(), w.time()), (turn, time), "talk took no time");
}

#[test]
fn a_kill_quest_counts_the_players_kills_and_rewards_with_xp() {
    let mut w = world_on("#########\n#E@pp...#\n#########\n");
    take(&mut w, Dir::W, "pups");
    assert_eq!(
        topics_of(&w, "elder"),
        ["quest-pups", "quest-parcel", "weather"]
    );
    kill_pups_east(&mut w, 1);
    assert!(said(&w).contains(&"Quest updated: 1 / 2 pups.".to_string()));
    kill_pups_east(&mut w, 1);
    let log = said(&w);
    let i = (log.iter())
        .position(|m| m == "Quest updated: 2 / 2 pups. Return to the elder.")
        .expect("the goal is met");
    assert!(log[..i].contains(&"The pup dies.".to_string()), "{log:?}");
    // Kills beyond the count say nothing more.
    assert_eq!(
        log.iter()
            .filter(|m| m.starts_with("Quest updated"))
            .count(),
        2
    );
    // Asking now hands it in: thanks, the reward, the XP.
    let xp = w.player_xp();
    put_player(&mut w, Pos::new(2, 1));
    ask(&mut w, Dir::W, "quest-pups");
    assert_eq!(
        from(&w, "The elder says: \"Thank you.\"", 4),
        [
            "The elder says: \"Thank you.\"",
            "Quest complete: Pups.",
            "The elder gives you 5 gold coins.",
            "You gain 50 experience.",
        ]
    );
    assert_eq!(w.player_xp(), xp + 50);
    assert_eq!(carried(&w, "gold_coin"), 5);
    assert_eq!(stage(&w, "pups"), Some(Stage::Done));
    // Done, and not repeatable: never offered again; the cellar, which required it, is.
    assert_eq!(
        topics_of(&w, "elder"),
        ["quest-cellar", "quest-parcel", "weather"]
    );
}

#[test]
fn class_abilities_make_a_quest_pay_more_gold_and_experience() {
    // The yard with one talent: a quarter more gold, half again the experience.
    let mut files = files_with("#########\n#E@pp...#\n#########\n", QUESTS);
    files.insert(
        "data/talents.toml",
        "[[tree]]\nid = \"class\"\nname = \"Class\"\ndescription = \"d\"\nclass = true\n\
         [[system]]\nid = \"talk\"\nname = \"Talk\"\n",
    );
    files.insert(
        "data/talents/patron.toml",
        "[[talent]]\nid = \"patron\"\nname = \"Patron\"\ntree = \"class\"\ngranted = true\n\
         description = \"d\"\nsystems = [\"talk\"]\neffects = [\
         { kind = \"quest_gold_percent\", value = 25 }, { kind = \"quest_xp_percent\", value = 50 }]\n",
    );
    let content = Content::load(&files).expect("test content is valid");
    let mut w = World::new(Arc::new(content), "yard", None).expect("world");
    w.player_mut().talents.take("patron", None);
    take(&mut w, Dir::W, "pups");
    kill_pups_east(&mut w, 2);
    let xp = w.player_xp();
    put_player(&mut w, Pos::new(2, 1));
    ask(&mut w, Dir::W, "quest-pups");
    assert_eq!(
        from(&w, "Quest complete: Pups.", 3),
        [
            "Quest complete: Pups.",
            "The elder gives you 6 gold coins.",
            "You gain 75 experience.",
        ]
    );
    assert_eq!(w.player_xp(), xp + 75);
    assert_eq!(carried(&w, "gold_coin"), 6);
}

#[test]
fn kills_before_taking_it_count_but_not_others_kills_nor_other_levels() {
    // Two pups killed first, then the quest: it is done as it is taken.
    let mut w = world_on("#########\n#E@pp...#\n#########\n");
    kill_pups_east(&mut w, 2);
    put_player(&mut w, Pos::new(2, 1));
    take(&mut w, Dir::W, "pups");
    assert_eq!(
        last(&w, 2),
        [
            "The elder says: \"Good.\"",
            "Quest updated: 2 / 2 pups. Return to the elder.",
        ]
    );
    // A hunter kills the pups in a room the player cannot see into: nothing counts.
    let mut w = world_on("#########\n#E@.....#\n#########\n#hpp....#\n#########\n");
    take(&mut w, Dir::W, "pups");
    for _ in 0..200 {
        if creatures_left(&w, "pup") == 0 {
            break;
        }
        w.apply(&Action::Wait);
    }
    assert_eq!(creatures_left(&w, "pup"), 0, "the hunter killed them");
    let view = w.perception();
    assert_eq!(view.quests()[0].status, "0 / 2 killed.");
    // The cellar's pup, killed by the player, is on another level than the quest asks.
    let mut w = world_on("#########\n#E@>....#\n#########\n");
    take(&mut w, Dir::W, "pups");
    run(&mut w, "move:E descend");
    assert_eq!(w.perception().level_id(), "cellar");
    for _ in 0..200 {
        if creatures_left(&w, "pup") == 0 {
            break;
        }
        w.apply(&Action::Move(Dir::E));
    }
    assert_eq!(creatures_left(&w, "pup"), 0);
    assert_eq!(w.perception().quests()[0].status, "0 / 2 killed.");
    // Tallied all the same, by level.
    let kills: Vec<_> = w.quests().kills().collect();
    assert_eq!(kills, [("cellar", "pup", 1)]);
}

#[test]
fn a_reach_quest_needs_its_prerequisite_and_counts_where_the_player_has_been() {
    let mut w = world_on(YARD);
    assert!(!topics_of(&w, "elder").contains(&"quest-cellar".to_string()));
    // The pups are done (as if killed and handed in).
    w.state.quests =
        serde_json::from_str(r#"{"records": {"pups": {"stage": "done", "completed": 1}}}"#)
            .expect("quests");
    take(&mut w, Dir::W, "cellar");
    assert_eq!(w.perception().quests()[0].status, "Not reached yet.");
    run(&mut w, "move:SE move:E*3 descend");
    assert_eq!(w.perception().level_id(), "cellar");
    assert!(
        said(&w).contains(
            &"Quest updated: you have reached the Cellar. Return to the elder.".to_string()
        ),
        "{:?}",
        last(&w, 5)
    );
    run(&mut w, "ascend");
    put_player(&mut w, Pos::new(2, 1));
    ask(&mut w, Dir::W, "quest-cellar");
    assert_eq!(last(&w, 3)[0], "The elder says: \"So it's still there.\"");
    assert_eq!(stage(&w, "cellar"), Some(Stage::Done));

    // Having been there before taking it up counts too.
    let mut w = world_on(YARD);
    run(&mut w, "move:SE move:E*3 descend ascend");
    put_player(&mut w, Pos::new(2, 1));
    w.state.quests =
        serde_json::from_str(r#"{"records": {"pups": {"stage": "done", "completed": 1}}}"#)
            .expect("quests");
    take(&mut w, Dir::W, "cellar");
    assert_eq!(
        last(&w, 1),
        ["Quest updated: you have reached the Cellar. Return to the elder."]
    );
}

#[test]
fn a_bring_quest_takes_from_the_pack_and_repeats() {
    let mut w = world_on(YARD);
    take(&mut w, Dir::E, "apples");
    assert_eq!(w.perception().quests()[0].status, "You carry 0 / 2.");
    // The reminder until the player carries enough; it changes nothing.
    let before = quiet_state(&w);
    ask(&mut w, Dir::E, "quest-apples");
    assert_eq!(last(&w, 1), ["The crone says: \"My apples?\""]);
    assert!(quiet_state(&w) == before);
    // Picking them up is progress.
    run(&mut w, "move:SW pickup");
    assert_eq!(last(&w, 1), ["Quest updated: 1 / 2 apples."]);
    run(&mut w, "move:E pickup move:E pickup");
    assert!(said(&w).contains(&"Quest updated: 2 / 2 apples. Return to the crone.".to_string()));
    assert_eq!(carried(&w, "apple"), 3);
    put_player(&mut w, Pos::new(2, 1));
    ask(&mut w, Dir::E, "quest-apples");
    assert_eq!(
        last(&w, 5),
        [
            "You give the crone 2 apples.",
            "The crone says: \"Lovely apples.\"",
            "Quest complete: Apples.",
            "The crone gives you a charm.",
            "You gain 5 experience.",
        ]
    );
    assert_eq!(carried(&w, "apple"), 1, "only two were taken");
    assert_eq!(carried(&w, "charm"), 1);
    // Repeatable: offered again, and the log keeps it among the done ones meanwhile.
    assert_eq!(topics_of(&w, "crone"), ["quest-apples"]);
    let log = w.perception().quests();
    assert_eq!((log[0].name.as_str(), log[0].active), ("Apples", false));
    take(&mut w, Dir::E, "apples");
    assert_eq!(w.perception().quests()[0].status, "You carry 1 / 2.");
    let record = w.quests().record("apples").copied().expect("record");
    assert_eq!((record.stage, record.completed), (Stage::Active, 1));
}

#[test]
fn a_delivery_is_blocked_while_the_item_is_lost_and_handed_in_to_the_recipient() {
    let mut w = world_on(YARD);
    // The crone takes nothing yet: there is nothing to hand her.
    assert_eq!(topics_of(&w, "crone"), ["quest-apples"]);
    take(&mut w, Dir::W, "parcel");
    assert_eq!(
        last(&w, 2),
        [
            "The elder says: \"Here it is.\"",
            "The elder gives you a letter."
        ]
    );
    assert_eq!(carried(&w, "letter"), 1);
    assert_eq!(topics_of(&w, "crone"), ["quest-parcel", "quest-apples"]);
    // The elder only reminds.
    ask(&mut w, Dir::W, "quest-parcel");
    assert_eq!(last(&w, 1), ["The elder says: \"The crone is waiting.\""]);
    // Dropped, it cannot be handed in; picked up again, it can.
    run(&mut w, "drop");
    assert_eq!(carried(&w, "letter"), 0);
    assert_eq!(topics_of(&w, "crone"), ["quest-apples"]);
    let view = w.perception();
    assert_eq!(view.quests()[0].status, "You no longer carry the letter.");
    run(&mut w, "pickup");
    assert_eq!(
        last(&w, 1),
        ["Quest updated: take the letter to the crone."]
    );
    assert_eq!(
        w.perception().quests()[0].status,
        "Take the letter to the crone."
    );
    ask(&mut w, Dir::E, "quest-parcel");
    assert_eq!(
        last(&w, 4),
        [
            "You give the crone a letter.",
            "The crone says: \"From the elder? Thank you, dear.\"",
            "Quest complete: The parcel.",
            "The crone gives you 3 gold coins.",
        ]
    );
    assert_eq!(carried(&w, "letter"), 0);
    assert!(!topics_of(&w, "elder").contains(&"quest-parcel".to_string()));
}

#[test]
fn a_pack_of_any_size_takes_a_delivery_and_a_reward() {
    // No count limit on the pack (spec §6): 26 stacks of one rock each, and the letter and the
    // reward still go into it.
    let mut w = world_on("#########\n#E@C....#\n#r......#\n#########\n");
    run(&mut w, "move:SW pickup move:NE");
    assert_eq!(w.state.player().inventory.len(), 26);
    ask(&mut w, Dir::W, "quest-parcel");
    ask(&mut w, Dir::W, "accept-parcel");
    assert_eq!(carried(&w, "letter"), 1);
    assert_eq!(w.state.player().inventory.len(), 27);
    // The pups' reward: gold, into the pack.
    w.state.quests = serde_json::from_str(
        r#"{"records": {"pups": {"stage": "active"}}, "kills": {"yard": {"pup": 2}}}"#,
    )
    .expect("quests");
    ask(&mut w, Dir::W, "quest-pups");
    assert_eq!(carried(&w, "gold_coin"), 5);
    let here = w.player_pos();
    assert!(w.state.current().objects_at(here).is_empty());
}

#[test]
fn any_number_of_quests_may_be_active_and_none_is_given_up() {
    // The owner's decision: no cap on active quests, and no abandoning.
    let mut w = world_on(YARD);
    take(&mut w, Dir::W, "pups");
    take(&mut w, Dir::W, "parcel");
    take(&mut w, Dir::E, "apples");
    for quest in ["pups", "parcel", "apples"] {
        assert_eq!(stage(&w, quest), Some(Stage::Active), "{quest}");
    }
    assert_eq!(w.perception().quests().len(), 3);
    assert!(parse_actions("abandon:pups").is_err());
}

#[test]
fn the_quest_log_lists_active_quests_then_done_ones() {
    let mut w = world_on(YARD);
    assert!(w.perception().quests().is_empty());
    take(&mut w, Dir::E, "apples");
    take(&mut w, Dir::W, "parcel");
    // An unanswered offer is not in the log.
    ask(&mut w, Dir::W, "quest-pups");
    run(&mut w, "move:S pickup move:W pickup move:NE");
    ask(&mut w, Dir::E, "quest-apples");
    let log = w.perception().quests();
    let rows: Vec<(&str, bool, &str, &str)> = (log.iter())
        .map(|q| {
            (
                q.name.as_str(),
                q.active,
                q.objective.as_str(),
                q.status.as_str(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            (
                "The parcel",
                true,
                "Deliver a letter to the crone.",
                "Take the letter to the crone."
            ),
            ("Apples", false, "Bring 2 apples to the crone.", "Done."),
        ]
    );
    assert_eq!(log[0].giver, "the elder");
    assert_eq!(log[0].offer, "Take this to the crone.");
    assert_eq!(log[0].kind, crate::content::QuestKind::Deliver);
}

#[test]
fn quests_survive_a_save_and_replay_exactly() {
    let actions = "ask:W:quest-pups ask:W:accept-pups move:E*12 ask:W:quest-parcel \
                   ask:W:accept-parcel drop wait pickup";
    let mut w = world_on("#########\n#E@pp...#\n#########\n");
    let mut replay = Replay::new(&w);
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(!w.quests().is_empty());
    let content = w.content().clone();
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    assert_eq!(loaded.perception().quests(), w.perception().quests());
    // A save naming a quest this content does not have is refused.
    let mut state = w.state().clone();
    state.quests =
        serde_json::from_str(r#"{"records": {"gone": {"stage": "active"}}}"#).expect("quests");
    assert!(matches!(
        World::from_state(content, state),
        Err(WorldError::InconsistentState(_))
    ));
}

#[test]
fn a_run_without_quests_keeps_its_state_as_before() {
    // Nothing quest-related is saved (or hashed) until the player hears of a quest or kills
    // something a kill quest asks for.
    let mut w = world_on(YARD);
    run(&mut w, "move:S pickup wait");
    let json = serde_json::to_string(w.state()).expect("json");
    assert!(!json.contains("quests"), "{json}");
}

#[test]
fn quest_files_are_validated() {
    let bad = |quests: &str, expect: &str| {
        let err = Content::load(&files_with(YARD, quests)).expect_err(quests);
        assert!(
            err.to_string().contains(expect),
            "{err} (expected {expect:?})"
        );
    };
    let quest = |body: &str| {
        format!(
            "[[quest]]\nid = \"q\"\nname = \"Q\"\ngiver = \"elder\"\n{body}\noffer = \"O.\"\n\
             accept = \"A.\"\nreminder = \"R.\"\ndone = \"D.\"\n"
        )
    };
    bad(
        &quest("type = \"kill\"\ntarget = \"dragon\""),
        "unknown creature `dragon`",
    );
    bad(
        &quest("type = \"kill\"\ntarget = \"player\""),
        "unknown creature `player`",
    );
    bad(
        &quest("type = \"kill\"\ntarget = \"pup\"\ncount = 0"),
        "`count` is 1 to 99",
    );
    bad(
        &quest("type = \"kill\"\ntarget = \"pup\"\nlevel = \"moon\""),
        "unknown level",
    );
    bad(
        &quest("type = \"kill\"\ntarget = \"pup\"\nto = \"crone\""),
        "has no `to`",
    );
    bad(
        &quest("type = \"bring\"\ntarget = \"pie\""),
        "unknown object `pie`",
    );
    bad(
        &quest("type = \"reach\"\ntarget = \"moon\""),
        "unknown level `moon`",
    );
    bad(
        &quest("type = \"reach\"\ntarget = \"cellar\"\ncount = 2"),
        "has no `count`",
    );
    bad(
        &quest("type = \"reach\"\ntarget = \"cellar\"\nrepeatable = true"),
        "not repeatable",
    );
    bad(
        &quest("type = \"deliver\"\ntarget = \"letter\""),
        "needs `to`",
    );
    bad(
        &quest("type = \"deliver\"\ntarget = \"letter\"\nto = \"elder\""),
        "someone other than the giver",
    );
    bad(
        &quest("type = \"bring\"\ntarget = \"apple\"\nto = \"pup\""),
        "no one who talks",
    );
    bad(
        &quest("type = \"bring\"\ntarget = \"apple\"\nrequires = \"later\""),
        "no quest `later` before this one",
    );
    bad(
        &quest("type = \"bring\"\ntarget = \"apple\"\nreward = { items = [\"pie\"] }"),
        "reward: unknown object `pie`",
    );
    bad(
        &quest("type = \"bring\"\ntarget = \"apple\"\nreward = { xp = 9999999 }"),
        "reward: `xp` is at most",
    );
    bad(
        &quest("type = \"bring\"\ntarget = \"apple\"\nname = \"x\""),
        "duplicate key",
    );
    bad(
        &quest("type = \"bring\"\ntarget = \"apple\"\nwhen = \"now\""),
        "unknown field",
    );
    bad(
        &quest("type = \"bring\"\ntarget = \"apple\"")
            .replace("giver = \"elder\"", "giver = \"pup\""),
        "no one who talks",
    );
    bad(
        &quest("type = \"bring\"\ntarget = \"apple\"").replace("\"q\"", "\"Q-1\""),
        "ids are lower-case",
    );
    bad(
        &format!(
            "{}{}",
            quest("type = \"bring\"\ntarget = \"apple\""),
            quest("type = \"bring\"\ntarget = \"apple\"")
        ),
        "there is already one",
    );
    bad(
        &quest("type = \"bring\"\ntarget = \"apple\"").replace("\"O.\"", "\" O.\""),
        "not empty nor padded",
    );
    bad(
        &quest("type = \"bring\"\ntarget = \"apple\"").replace(
            "name = \"Q\"",
            "name = \"A name far too long for a quest log\"",
        ),
        "at most 32 characters",
    );
    // Nine topics fit the dialogue screen: the elder's own and eight quests' (with the offer's
    // second) do not.
    let many: String = (0..8)
        .map(|i| {
            quest("type = \"bring\"\ntarget = \"apple\"").replace("\"q\"", &format!("\"q{i}\""))
        })
        .collect();
    bad(&many, "at most 9 fit");
    let fits: String = (0..7)
        .map(|i| {
            quest("type = \"bring\"\ntarget = \"apple\"").replace("\"q\"", &format!("\"q{i}\""))
        })
        .collect();
    assert!(Content::load(&files_with(YARD, &fits)).is_ok());
}

// The showcase's own quests, with the built-in content.

/// A showcase run.
fn showcase() -> World {
    let content = Arc::new(Content::builtin().expect("content"));
    World::new(content, "showcase", None).expect("run")
}

/// The first creature of type `creature` on the current level, and a free walkable cell next to
/// it for the player; the player is put there, and the way to the creature returned.
fn beside(world: &mut World, creature: &str) -> Dir {
    let c = world.content().creature_id(creature).expect("creature");
    let level = world.state.current_level.clone();
    let at = (world.state.actors.values())
        .find(|a| a.creature == c && a.level == level)
        .expect("there is one")
        .pos;
    let player = world.state.player;
    let free = Dir::ALL.into_iter().find(|&d| {
        let p = at.step(d);
        world.state.current().is_walkable(&world.content, p)
            && (world.state.actor_at(&level, p)).is_none_or(|a| a.id == player)
    });
    let d = free.expect("room beside it");
    put_player(world, at.step(d));
    d.opposite()
}

#[test]
fn the_showcase_people_give_four_quests() {
    let w = showcase();
    let content = w.content();
    let kinds: Vec<(&str, &str, String)> = (content.quests().iter())
        .map(|q| (q.id.as_str(), q.kind().id(), content.speaker_name(q.giver)))
        .collect();
    assert_eq!(
        kinds,
        [
            ("rats", "kill", "old hermit".to_string()),
            ("depths", "reach", "old hermit".to_string()),
            ("apples", "bring", "gardener".to_string()),
            ("letter", "deliver", "wandering bard".to_string()),
        ]
    );
    let letter = &content.quests()[3];
    assert_eq!(content.speaker_name(letter.turn_in()), "old hermit");
}

#[test]
fn the_hermits_rats_then_the_depths() {
    let mut w = showcase();
    let toward = beside(&mut w, "hermit");
    assert_eq!(topics_of(&w, "hermit")[0], "quest-rats");
    take(&mut w, toward, "rats");
    // Down in the arena, with nobody but three rats about (each dies of a scratch).
    w.travel("arena", None).expect("arena");
    let rat = w.content().creature_id("rat").expect("rat");
    let player = w.state.player;
    w.state
        .actors
        .retain(|id, a| *id == player || a.level != "arena" || a.creature == rat);
    let rats: Vec<ActorId> = (w.state.actors.values())
        .filter(|a| a.creature == rat)
        .map(|a| a.id)
        .take(3)
        .collect();
    w.state
        .actors
        .retain(|id, a| a.creature != rat || rats.contains(id));
    for id in &rats {
        w.state.actors.get_mut(id).expect("rat").hp = 1;
    }
    let me = w.state.player().pos;
    for (i, id) in rats.iter().enumerate() {
        let at = Pos::new(me.x + 1, me.y - i as i32);
        w.state.actors.get_mut(id).expect("rat").pos = at;
    }
    for _ in 0..300 {
        if creatures_left(&w, "rat") == 0 {
            break;
        }
        let me = w.player_pos();
        let next = (w.state.actors.values())
            .filter(|a| a.creature == rat)
            .map(|a| a.pos)
            .min_by_key(|p| p.chebyshev(me))
            .expect("a rat");
        let dir = Dir::toward(me, next).expect("a way");
        w.apply(&Action::Move(dir));
    }
    assert_eq!(creatures_left(&w, "rat"), 0, "{:?}", last(&w, 8));
    assert!(
        said(&w)
            .contains(&"Quest updated: 3 / 3 giant rats. Return to the old hermit.".to_string())
    );
    w.travel("hub", None).expect("hub");
    let toward = beside(&mut w, "hermit");
    ask(&mut w, toward, "quest-rats");
    assert_eq!(
        from(&w, "Quest complete: Rats in the arena.", 3),
        [
            "Quest complete: Rats in the arena.",
            "The old hermit gives you 20 gold coins.",
            "You gain 30 experience.",
        ]
    );
    // Then: three levels down.
    assert_eq!(topics_of(&w, "hermit")[0], "quest-depths");
    take(&mut w, toward, "depths");
    w.travel("depths:3", None).expect("depths:3");
    let log = w.perception().quests();
    assert_eq!(log[0].objective, "Reach the Depths 3.");
    assert_eq!(log[0].status, "Return to the old hermit.");
    w.travel("hub", None).expect("hub");
    let toward = beside(&mut w, "hermit");
    ask(&mut w, toward, "quest-depths");
    let gift = "The old hermit gives you 50 gold coins and a jade amulet.";
    assert!(said(&w).contains(&gift.to_string()));
}

#[test]
fn the_gardeners_apples_and_the_bards_letter() {
    let mut w = showcase();
    w.travel("courtyard", None).expect("courtyard");
    // The gardener has no topics of his own: his quest opens the dialogue screen.
    assert_eq!(topics_of(&w, "gardener"), ["quest-apples"]);
    let toward = beside(&mut w, "gardener");
    take(&mut w, toward, "apples");
    let apple = w.content().object_id("apple").expect("apple");
    w.state.inventory_mut().push(Stack::new(apple, 3));
    ask(&mut w, toward, "quest-apples");
    assert_eq!(
        from(&w, "You give the gardener 3 apples.", 5),
        [
            "You give the gardener 3 apples.",
            "The gardener says: \"Lovely. Good firm ones. Here, for your trouble. And mind the \
             beds on your way out.\"",
            "Quest complete: Three apples.",
            "The gardener gives you 10 gold coins.",
            "You gain 15 experience.",
        ]
    );
    // The bard's letter, carried down to the hermit.
    let toward = beside(&mut w, "bard");
    take(&mut w, toward, "letter");
    assert_eq!(carried(&w, "sealed_letter"), 1);
    w.travel("hub", None).expect("hub");
    let toward = beside(&mut w, "hermit");
    assert_eq!(topics_of(&w, "hermit")[..2], ["quest-rats", "quest-letter"]);
    ask(&mut w, toward, "quest-letter");
    assert_eq!(
        from(&w, "You give the old hermit a sealed letter.", 5)[1..],
        [
            "The old hermit says: \"A letter? From the bard? Hm. A song, and he's spelled half \
             the words wrong. Thank him for me. Here, for the walk.\"",
            "Quest complete: The bard's letter.",
            "The old hermit gives you 5 gold coins.",
            "You gain 15 experience.",
        ]
    );
    assert_eq!(carried(&w, "sealed_letter"), 0);
    let log = w.perception().quests();
    assert!(log.iter().all(|q| !q.active));
    assert_eq!(log.len(), 2);
}
