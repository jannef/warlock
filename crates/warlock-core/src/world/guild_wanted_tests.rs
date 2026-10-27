//! The Weyfarers' Gilde's fourth link, "Wanted" (#426; the guild design §7.4, §8): given after the
//! scroll, it draws one of the four criminals and reveals that one's lair and no other; the dev
//! roll moves it; and a run that is given it, hunts Black Wystan down through Wulfeshol, finds
//! High Toby where he fell, and hands the quest in is made an Iron, the same every time.

use super::*;
use crate::action::{Action, parse_actions};
use crate::content::{ContentFiles, StairsDir};
use crate::grid::Grid;
use std::collections::{BTreeSet, VecDeque};
use std::sync::LazyLock;

/// Wat One-Eye's post, by the Gildhalle's far wall.
const WAT: Pos = Pos::new(81, 27);
/// Cēaptūn on the overworld.
const TOWN: Pos = Pos::new(18, 28);
/// Each criminal (the variant), their name, and their lair: its id, name, and overworld cell.
const WANTED: [(&str, &str, &str, &str, Pos); 4] = [
    (
        "sallow",
        "Mother Sallow",
        "wyrtholm",
        "Wyrtholm",
        Pos::new(88, 46),
    ),
    (
        "wystan",
        "Black Wystan",
        "wulfeshol",
        "Wulfeshol",
        Pos::new(18, 49),
    ),
    (
        "osrick",
        "Brother Osrick",
        "banfeld",
        "Bānfeld",
        Pos::new(42, 31),
    ),
    (
        "herrick",
        "Sergeant Herrick",
        "brocenburg",
        "Brocenburg",
        Pos::new(66, 12),
    ),
];
/// The links before it.
const CHAIN: [&str; 3] = ["guild_quarry", "guild_hundred", "guild_scroll"];

/// Built-in content with nobody hungering (the lairs are days away).
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    Arc::new(Content::load(&files).expect("builtin content"))
});

/// A run in Cēaptūn with the chain's first three links done, as if handed in.
fn town(seed: Option<u64>) -> World {
    let mut w = World::new(CONTENT.clone(), "ceaptun", seed).expect("a run in Cēaptūn");
    let records: Vec<String> = (CHAIN.iter())
        .map(|q| format!(r#""{q}": {{"stage": "done", "completed": 1}}"#))
        .collect();
    let json = format!(r#"{{"records": {{{}}}}}"#, records.join(", "));
    w.state.quests = serde_json::from_str(&json).expect("quests");
    // A fighter's kit (the scenario brings no pack), put on by `hunt`.
    let id = w.state.player;
    for object in ["long_sword", "chain_mail", "large_shield", "iron_helm"] {
        let object = CONTENT.object_id(object).expect(object);
        (w.state.actors.get_mut(&id).expect("player").inventory).push(Stack::new(object, 1));
    }
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

/// Puts the player north of Wat, so they talk to the south.
fn beside_wat(world: &mut World) {
    let id = world.state.player;
    world.state.actors.get_mut(&id).expect("player").pos = WAT.step(Dir::N);
    world.perceive(&mut Vec::new());
}

fn variant(world: &World) -> String {
    (world.quests().record("guild_wanted"))
        .and_then(|r| r.roll.as_ref())
        .and_then(|r| r.variant.clone())
        .expect("a criminal drawn")
}

#[test]
fn the_wanted_follows_the_scroll_and_reveals_the_drawn_lair_and_no_other() {
    let mut drawn = BTreeSet::new();
    for seed in 1..=24 {
        let mut w = town(Some(seed));
        let wat = w.state.actor_at("ceaptun", WAT).expect("Wat").id;
        let topics: Vec<String> = (w.topics(wat).into_iter()).map(|t| t.id).collect();
        assert_eq!(topics, ["quest-guild_wanted", "guild", "board"]);
        beside_wat(&mut w);
        assert!(w.state.revealed.is_empty());
        act(&mut w, "ask:S:quest-guild_wanted");
        let v = variant(&w);
        let (_, name, lair, lair_name, _) =
            (WANTED.iter()).find(|c| c.0 == v).expect("one of the four");
        assert_eq!(
            w.state.revealed,
            BTreeSet::from([lair.to_string()]),
            "seed {seed}"
        );
        assert_eq!(
            said(&w).last().map(String::as_str),
            Some(format!("Wat One-Eye marks {lair_name} on your map.").as_str())
        );
        let log = (w.perception().quests().into_iter())
            .find(|q| q.id == "guild_wanted")
            .expect("logged");
        let wants = format!("Wat One-Eye wants you to kill {name}.");
        assert!(log.objective.starts_with(&wants), "{}", log.objective);
        // Asked again: the reminder, and the same criminal.
        act(&mut w, "ask:S:quest-guild_wanted");
        assert_eq!(variant(&w), v);
        drawn.insert(v);
    }
    assert_eq!(drawn.len(), 4, "every criminal comes up: {drawn:?}");
}

#[test]
fn the_dev_roll_names_another_criminal_and_reveals_their_lair() {
    let mut w = town(None);
    beside_wat(&mut w);
    act(&mut w, "ask:S:quest-guild_wanted");
    let first = variant(&w);
    let other = (WANTED.iter()).find(|c| c.0 != first).expect("another");
    act(&mut w, &format!("quest_roll:guild_wanted:{}", other.0));
    assert_eq!(variant(&w), other.0);
    assert!(w.state.revealed.contains(other.2));
}

/// The next action towards `target`: a step along a shortest 8-way path (bumping a closed door
/// opens it; with `fight`, bumping a creature attacks it). When a creature blocks every step and
/// the player does not fight, back away.
fn step_towards(world: &World, target: Pos, fight: bool) -> Action {
    let level = world.state.current();
    let content = &world.content;
    let passable = |p: Pos| {
        level.terrain.get(p).is_some_and(|t| {
            let def = content.terrain(*t);
            def.walkable || def.is_closed_door()
        })
    };
    let mut dist = Grid::new(level.width(), level.height(), u32::MAX);
    dist[target] = 0;
    let mut queue = VecDeque::from([target]);
    while let Some(p) = queue.pop_front() {
        for dir in Dir::ALL {
            let q = p.step(dir);
            if passable(q) && dist[q] == u32::MAX {
                dist[q] = dist[p] + 1;
                queue.push_back(q);
            }
        }
    }
    let here = world.player_pos();
    let free = |d: &Dir| {
        let q = here.step(*d);
        dist.get(q).is_some_and(|n| *n != u32::MAX)
            && (fight || world.state.actor_at(&level.id, q).is_none())
    };
    let closer = Dir::ALL
        .into_iter()
        .filter(|d| free(d) && dist[here.step(*d)] < dist[here])
        .min_by_key(|d| dist[here.step(*d)]);
    let away = || {
        Dir::ALL
            .into_iter()
            .filter(free)
            .max_by_key(|d| dist[here.step(*d)])
    };
    closer.or_else(away).map_or(Action::Wait, Action::Move)
}

/// Applies `action` (a wait when it is refused), logging what was applied; the player must live.
fn apply(world: &mut World, action: Action, log: &mut Vec<Action>) {
    let action = if world.apply(&action).took_turn {
        action
    } else {
        world.apply(&Action::Wait);
        Action::Wait
    };
    log.push(action);
    assert!(
        world.state.death.is_none(),
        "the player died on {} after {} actions: {:?}\n{}",
        world.state.current_level,
        log.len(),
        said(world).iter().rev().take(8).collect::<Vec<_>>(),
        crate::dump::layer_rows(world, crate::dump::Layer::Actors).join("\n")
    );
}

/// Walks to `target` (fighting what is in the way, with `fight`).
fn walk_to(world: &mut World, target: Pos, fight: bool, log: &mut Vec<Action>) {
    for _ in 0..4000 {
        if world.player_pos() == target {
            return;
        }
        let action = step_towards(world, target, fight);
        apply(world, action, log);
    }
    panic!("could not reach {target} on {}", world.state.current_level);
}

/// Walks to the staircase in `dir`, fighting the way, and takes it.
fn take(world: &mut World, dir: StairsDir, log: &mut Vec<Action>) {
    let stairs = (world.state.current().stairs.iter())
        .find(|s| s.dir == dir)
        .expect("stairs")
        .pos;
    walk_to(world, stairs, true, log);
    let action = match dir {
        StairsDir::Down => Action::Descend,
        StairsDir::Up => Action::Ascend,
    };
    assert!(world.apply(&action).took_turn, "{:?}", said(world).last());
    log.push(action);
}

/// Black Wystan, while he lives on the player's level.
fn wystan(world: &World) -> Option<Pos> {
    let wystan = CONTENT.creature_id("wystan").expect("Black Wystan");
    (world.state.actors.values())
        .find(|a| a.creature == wystan && a.level == world.state.current_level)
        .map(|a| a.pos)
}

/// From Wat's side: be given the wanted, drawn as Black Wystan; out to Wulfeshol and down its
/// four floors; kill him; back up and home to Wat; hand it in. Returns every action applied.
fn hunt(w: &mut World) -> Vec<Action> {
    let mut log = parse_actions(
        "equip:main_hand:a equip:body:b equip:off_hand:c equip:head:d xp:3000000 levelup:auto \
         ask:S:quest-guild_wanted quest_roll:guild_wanted:wystan ascend",
    )
    .expect("actions");
    for action in &log {
        w.apply(action);
    }
    assert_eq!(w.state.current_level, "ascmearc");
    let lair = WANTED[1].4;
    walk_to(w, lair, false, &mut log);
    apply(w, Action::Descend, &mut log);
    for depth in 1..=3 {
        assert_eq!(w.state.current_level, format!("wulfeshol:{depth}"));
        take(w, StairsDir::Down, &mut log);
    }
    assert_eq!(w.state.current_level, "wulfeshol:4");
    for _ in 0..4000 {
        let Some(at) = wystan(w) else { break };
        let action = step_towards(w, at, true);
        apply(w, action, &mut log);
    }
    assert_eq!(wystan(w), None, "Black Wystan is dead");
    let fell = (w.state.current().objects.values().flatten())
        .any(|s| s.magic().and_then(|m| m.unique.as_deref()) == Some("high_toby"));
    assert!(fell, "High Toby lies where he fell");
    let log_line = (w.perception().quests().into_iter())
        .find(|q| q.id == "guild_wanted")
        .expect("logged");
    assert!(
        log_line.status.contains("Return to Wat One-Eye"),
        "{log_line:?}"
    );
    for _ in 1..=4 {
        take(w, StairsDir::Up, &mut log);
    }
    assert_eq!(w.player_pos(), lair);
    walk_to(w, TOWN, false, &mut log);
    apply(w, Action::Descend, &mut log);
    assert_eq!(w.state.current_level, "ceaptun");
    walk_to(w, WAT.step(Dir::N), false, &mut log);
    let hand_in = parse_actions("ask:S:quest-guild_wanted").expect("ask");
    w.apply(&hand_in[0]);
    log.extend(hand_in);
    log
}

#[test]
fn black_wystan_hunted_down_makes_the_player_an_iron_and_replays_exactly() {
    // A fixed seed: the hunt is a bot that fights its way down, and a roster change moves the draws.
    let mut w = town(Some(4));
    beside_wat(&mut w);
    let start = w.state_hash();
    let log = hunt(&mut w);
    let lines = said(&w);
    assert!(
        lines.contains(&"Quest complete: Wanted.".to_string()),
        "{lines:?}"
    );
    let guilds = w.perception().own_guilds();
    assert_eq!(guilds.first().map(|g| g.title()), Some("Iron"));
    assert!(
        lines.iter().any(|l| l.contains("names you an Iron")),
        "{lines:?}"
    );
    // The same actions from the same start come to the same end.
    let mut again = town(Some(4));
    beside_wat(&mut again);
    assert_eq!(again.state_hash(), start);
    for action in &log {
        again.apply(action);
    }
    assert_eq!(again.state_hash(), w.state_hash());
}
