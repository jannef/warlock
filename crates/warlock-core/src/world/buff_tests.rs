//! Status effects as buffs, hexes, and cooldowns (spec §8 "Status effects", #227): their combat
//! fields join attacks and Defence, `cost` slows every action, `blocks` refuses actions and greys
//! out abilities, `follows` begins a status as another ends, and all of it saves and replays.

use super::*;
use crate::ability::{Ability, OwnAbility, Unavailable};
use crate::action::parse_actions;
use crate::burden::Burden;
use crate::combat::Category;
use crate::content::{ContentFiles, StatusId};
use crate::equipment;
use crate::replay::Replay;
use crate::stats::Stat;
use crate::status::Effect;

/// Statuses for the tests, after the built-in ones: a buff, a hex, a slow, one that blocks, and
/// one that wears off or is cured, followed by a cooldown.
const STATUSES: &str = r#"
[[status]]
id = "inspired"
name = "Inspired"
max = 1
eases = 300
combat = { to_hit = 1, damage = 2, dodge = 1, parry = 3 }

[[status]]
id = "hexed"
name = "Hexed"
seen = "hexed"
max = 1
eases = 60
combat = { to_hit = -2 }

[[status]]
id = "slowed"
name = "Slowed"
max = 1
eases = 60
cost = 150

[[status]]
id = "fumbling"
name = "Fumbling"
max = 1
eases = 60
blocks = ["melee", "read"]

[[status]]
id = "warded"
name = "Warded"
max = 1
eases = 120
cured_by = ["bandage"]
follows = "winded"
"#;

/// A room to oneself.
const ALONE: &str = "#######\n#@....#\n#.....#\n#######\n";
/// A giant rat next to the player.
const NEXT_TO_RAT: &str = "#######\n#@r...#\n#.....#\n#######\n";

/// The built-in content with the test statuses, a level "buffs" on `map` (fully lit), and a
/// scenario "buffs" that starts there.
fn files(map: &str) -> ContentFiles {
    let mut files = ContentFiles::builtin();
    let statuses = format!("{}{STATUSES}", built_in_statuses(&files));
    files.insert("data/status.toml", &statuses);
    files.insert(
        "levels/buffs.toml",
        &format!(
            "name = \"Buffs\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"r\" = {{ terrain = \"floor\", creature = \"rat\" }}\n"
        ),
    );
    files.insert(
        "scenarios/buffs.toml",
        "name = \"Buffs\"\nstart_level = \"buffs\"\n",
    );
    files
}

/// The text of `data/status.toml` in `files`.
fn built_in_statuses(files: &ContentFiles) -> String {
    let (_, text) = files.table("status").next().expect("data/status.toml");
    text.to_string()
}

fn world(map: &str) -> World {
    let content = Content::load(&files(map)).expect("test content is valid");
    World::new(Arc::new(content), "buffs", None).expect("world")
}

fn status(w: &World, id: &str) -> StatusId {
    w.content().status_id(id).expect(id)
}

fn effect(w: &World, actor: ActorId, id: &str) -> Option<Effect> {
    let status = status(w, id);
    (w.state.actors.get(&actor)).and_then(|a| a.effects.get(status).copied())
}

fn afflict(w: &mut World, actor: ActorId, id: &str, by: Option<ActorId>) -> Vec<Event> {
    let status = status(w, id);
    let mut events = Vec::new();
    w.afflict(actor, status, 1, by, &mut events);
    events
}

fn run(w: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|a| w.apply(a))
        .collect()
}

fn messages(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

fn last_message(w: &World) -> String {
    messages(w).last().cloned().unwrap_or_default()
}

/// The giant rat's id.
fn rat(w: &World) -> ActorId {
    let rat = w.content().creature_id("rat").expect("rat");
    (w.state.actors.values())
        .find(|a| a.creature == rat)
        .map(|a| a.id)
        .expect("a rat")
}

#[test]
fn buffs_and_hexes_join_every_attack_and_defence() {
    let mut w = world(NEXT_TO_RAT);
    let content = w.content().clone();
    let (player, rat) = (w.player_id(), rat(&w));
    let me = |w: &World| w.state.actors[&player].clone();
    let (melee, defence) = (
        me(&w).melee(&content),
        me(&w).defence(&content, Burden::Unburdened, None),
    );
    afflict(&mut w, player, "inspired", Some(player));
    let (buffed, guarded) = (
        me(&w).melee(&content),
        me(&w).defence(&content, Burden::Unburdened, None),
    );
    assert_eq!(
        (buffed.to_hit, buffed.bonus),
        (melee.to_hit + 1, melee.bonus + 2),
        "to-hit and damage, as a stance's"
    );
    assert_eq!(
        guarded.bonus(Category::Dodge),
        defence.bonus(Category::Dodge) + 1,
        "Dodge stacks"
    );
    assert_eq!(
        guarded.bonus(Category::Parry),
        defence.bonus(Category::Parry).max(3),
        "of Parry the highest counts"
    );
    // On top of the stance: both count.
    run(&mut w, "stance:aggressive");
    let stanced = me(&w).defence(&content, Burden::Unburdened, None);
    let aggressive = content.stance_def(crate::content::CombatStance::Aggressive);
    assert_eq!(
        stanced.bonus(Category::Dodge),
        guarded.bonus(Category::Dodge) + aggressive.dodge
    );
    // A hex is a status on a creature: its bite misses more.
    let bite = w.state.actors[&rat].melee(&content);
    afflict(&mut w, rat, "hexed", Some(player));
    assert_eq!(w.state.actors[&rat].melee(&content).to_hit, bite.to_hit - 2);
    // It shows by its `seen` word, as any status does.
    run(&mut w, "wait");
    let hexed = status(&w, "hexed");
    assert!(
        w.perception().effects_of(rat).any(|s| s == hexed),
        "look mode names it"
    );
    // Worn off, nothing is left of it.
    let mut calm = world(ALONE);
    let alone = calm.player_id();
    let plain = calm.state.actors[&alone].melee(&content);
    afflict(&mut calm, alone, "inspired", None);
    run(&mut calm, "wait*51");
    assert_eq!(effect(&calm, alone, "inspired"), None);
    assert_eq!(calm.state.actors[&alone].melee(&content), plain);
}

#[test]
fn a_slowing_status_makes_every_action_dearer_while_it_lasts() {
    let mut w = world(ALONE);
    let player = w.player_id();
    let wait = w.price_of(player, ActionKind::Wait);
    let content = w.content().clone();
    let floor = content.terrain(content.terrain_id("floor").expect("floor"));
    let step = w.step_price(player, floor, false);
    afflict(&mut w, player, "slowed", None);
    assert_eq!(w.price_of(player, ActionKind::Wait), wait * 3 / 2);
    assert_eq!(w.step_price(player, floor, false), step * 3 / 2);
    // A turn takes half again as long: 9 seconds instead of 6.
    let before = w.state.time;
    run(&mut w, "wait");
    assert_eq!(w.state.time.since(before), Duration::seconds(9));
    // Worn off after a minute: back to normal.
    run(&mut w, "wait*7");
    assert_eq!(effect(&w, player, "slowed"), None);
    assert_eq!(w.price_of(player, ActionKind::Wait), wait);
    // Creatures too.
    let mut w = world(NEXT_TO_RAT);
    let rat = rat(&w);
    let wait = w.price_of(rat, ActionKind::Wait);
    afflict(&mut w, rat, "slowed", None);
    assert_eq!(w.price_of(rat, ActionKind::Wait), wait * 3 / 2);
}

#[test]
fn blocked_actions_are_refused_and_abilities_that_cost_them_greyed_out() {
    let mut w = world(NEXT_TO_RAT);
    let rat = rat(&w);
    run(&mut w, "afflict:fumbling");
    // Moving into the hostile rat is an attack: refused, no time, nothing done.
    let rat_hp = w.state.actors[&rat].hp;
    let (turn, time) = (w.state.turn, w.state.time);
    let out = w.apply(&Action::Move(Dir::E));
    assert!(!out.took_turn);
    assert_eq!(last_message(&w), "You cannot fight while fumbling.");
    assert_eq!(
        (w.state.turn, w.state.time, w.state.actors[&rat].hp),
        (turn, time, rat_hp)
    );
    let out = run(&mut w, "read:a").remove(0);
    assert!(!out.took_turn);
    assert_eq!(last_message(&w), "You cannot read while fumbling.");
    // Anything else goes on: a step.
    assert!(w.apply(&Action::Move(Dir::S)).took_turn);
    assert_eq!(w.state.player().pos, Pos::new(1, 2));
    // With a shield to bash with, the shield bash is greyed out while it lasts, and refused.
    let mut w = world(ALONE);
    let player = w.player_id();
    let content = w.content().clone();
    let buckler = content.object_id("buckler").expect("a buckler");
    let me = w.state.actors.get_mut(&player).expect("the player");
    me.inventory.push(crate::object::Stack::new(buckler, 1));
    let item = me.inventory.len() - 1;
    let slot = equipment::free_slot(&content, &me.equipment, buckler).expect("a hand");
    me.equipment
        .put(&equipment::slots_taken(&content, buckler, slot), item);
    run(&mut w, "afflict:fumbling");
    let fumbling = status(&w, "fumbling");
    assert_eq!(
        w.perception().abilities(),
        [OwnAbility {
            ability: Ability::ShieldBash,
            unavailable: Some(Unavailable::While(fumbling)),
        }]
    );
    let why = Unavailable::While(fumbling);
    assert_eq!(why.reason(&content), "you are fumbling");
    assert_eq!(why.id(&content), "while:fumbling");
    assert!(!w.apply(&Action::Bash(Dir::E)).took_turn);
    assert_eq!(last_message(&w), "You cannot fight while fumbling.");
    // Worn off, the bash is back.
    run(&mut w, "wait*10");
    assert_eq!(effect(&w, player, "fumbling"), None);
    assert_eq!(w.perception().abilities()[0].unavailable, None);
}

#[test]
fn the_built_in_rage_raises_and_lowers_scores_blocks_reading_and_is_followed_by_exhaustion() {
    let mut w = world(ALONE);
    let content = w.content().clone();
    let player = w.player_id();
    let scores = |w: &World| w.state.actors[&player].effective_stats(&content);
    let calm = scores(&w);
    let lasting = w.state.player().lasting_max_hp(&content);
    run(&mut w, "afflict:raging");
    assert_eq!(last_message(&w), "You fly into a rage!");
    let raging = scores(&w);
    for (stat, by) in [
        (Stat::Strength, 2),
        (Stat::Toughness, 2),
        (Stat::Intelligence, -2),
    ] {
        assert_eq!(
            i32::from(raging.score(stat)),
            i32::from(calm.score(stat)) + by
        );
    }
    // Tougher, it can take more hit points, healed up to them in time.
    let max = w.state.player().max_hp(&content);
    assert!(max > lasting);
    w.state.actors.get_mut(&player).expect("player").hp = max;
    run(&mut w, "read:a");
    assert_eq!(last_message(&w), "You cannot read while raging.");
    // Five minutes later the rage ends, and exhaustion follows at that moment.
    run(&mut w, "wait*50");
    assert_eq!(effect(&w, player, "raging"), None);
    let exhausted = effect(&w, player, "exhausted").expect("exhausted");
    assert_eq!((exhausted.severity, exhausted.wear), (1, 0));
    let said = messages(&w);
    let at = (said.iter().position(|m| m == "Your rage subsides.")).expect("the end is said");
    assert_eq!(said[at + 1], "You are exhausted.");
    assert_eq!(
        i32::from(scores(&w).score(Stat::Strength)),
        i32::from(calm.score(Stat::Strength)) - 1
    );
    // The hit points the rage gave go with it.
    assert_eq!(w.state.player().hp, lasting);
    // Reading again, and ten minutes later the exhaustion passes too.
    run(&mut w, "read:a");
    assert_ne!(last_message(&w), "You cannot read while raging.");
    run(&mut w, "wait*100");
    assert_eq!(effect(&w, player, "exhausted"), None);
    assert_eq!(scores(&w), calm);
}

#[test]
fn whoever_caused_a_status_causes_what_follows_it_and_a_cure_ends_it_too() {
    let mut w = world(ALONE);
    let player = w.player_id();
    afflict(&mut w, player, "warded", Some(player));
    run(&mut w, "wait*19");
    assert!(effect(&w, player, "winded").is_none(), "not yet");
    run(&mut w, "wait");
    assert_eq!(effect(&w, player, "warded"), None);
    let winded = effect(&w, player, "winded").expect("winded");
    assert_eq!((winded.by, winded.wear), (Some(player), 0));
    assert_eq!(last_message(&w), "You are winded.");
    // Cured, a status is followed all the same: its end, then what follows.
    let mut w = world(ALONE);
    afflict(&mut w, player, "warded", None);
    let mut events = Vec::new();
    assert!(w.cure(player, "bandage", &mut events));
    let (warded, winded) = (status(&w, "warded"), status(&w, "winded"));
    assert_eq!(
        events,
        [
            Event::Status {
                actor: player,
                status: warded,
                began: false
            },
            Event::Status {
                actor: player,
                status: winded,
                began: true
            },
        ]
    );
    assert!(effect(&w, player, "winded").is_some());
}

#[test]
fn buffs_save_and_replay_exactly() {
    let content = Arc::new(Content::load(&files(ALONE)).expect("content"));
    let mut w = World::new(content.clone(), "buffs", None).expect("world");
    let mut replay = Replay::new(&w);
    for action in
        parse_actions("afflict:warded afflict:slowed wait*3 read:a afflict:fumbling move:E wait*30")
            .expect("actions")
    {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(effect(&w, w.player_id(), "winded").is_some());
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content)
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
}

#[test]
fn nothing_may_follow_into_or_out_of_the_status_diseases_show_as() {
    let mut files = files(ALONE);
    let statuses = built_in_statuses(&files);
    let into = format!(
        "{statuses}[[status]]\nid = \"feverish\"\nname = \"Feverish\"\nmax = 1\neases = 60\n\
         follows = \"diseased\"\n"
    );
    let out = statuses.replace(
        "id = \"diseased\"\nname = \"Diseased\"\n",
        "id = \"diseased\"\nname = \"Diseased\"\nfollows = \"winded\"\ncured_by = [\"x\"]\n",
    );
    assert_ne!(
        out, statuses,
        "the built-in Diseased is where the test expects it"
    );
    for (text, why) in [
        (into, "status `feverish`: follows `diseased`"),
        (out, "`cured_by`, or `follows`"),
    ] {
        files.insert("data/status.toml", &text);
        let e = Content::load(&files).unwrap_err();
        assert!(e.message.contains(why), "{why}: {}", e.message);
    }
}
