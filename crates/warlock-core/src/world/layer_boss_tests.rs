//! Layer bosses (`layer_boss.rs`; spec §7 "The endless branch", #1118): the lair on a tenth
//! floor, its hidden way down, the boss's death opening it, the boss made again until it is
//! slain, an ordinary floor after, and the content's checks.

use super::*;
use crate::action::{Action, parse_actions};
use crate::content::{Content, ContentFiles, StairsDir};
use crate::mapgen;
use std::sync::{Arc, LazyLock};

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));

const BRANCH: &str = "grundleas";

fn floor(depth: u32) -> String {
    format!("{BRANCH}:{depth}")
}

fn start(seed: u64) -> World {
    World::new(CONTENT.clone(), BRANCH, Some(seed)).expect("world")
}

/// The player arrives on floor `depth` directly, as if by a staircase.
fn jump(w: &mut World, depth: u32) {
    w.travel(&floor(depth), None).expect("travel");
    let to = floor(depth);
    w.forget_floors(&[Event::LevelChanged {
        from: String::new(),
        to,
    }]);
}

/// The player, set on the staircase `dir` of the current floor, takes it.
fn take(w: &mut World, dir: StairsDir) {
    let at = (w.state.current().stairs.iter())
        .find(|s| s.dir == dir)
        .expect("stairs")
        .pos;
    let player = w.state.player;
    let here = w.state.current_level.clone();
    (w.state.actors).retain(|id, a| *id == player || a.level != here || a.pos != at);
    for action in parse_actions("levelup:auto").expect("actions") {
        w.apply(&action);
    }
    w.state.death = None;
    let content = w.content.clone();
    let me = w.state.actors.get_mut(&player).expect("player");
    me.pos = at;
    me.energy = me.energy.max(0);
    me.hp = me.max_hp(&content);
    me.effects = Default::default();
    let action = match dir {
        StairsDir::Down => Action::Descend,
        StairsDir::Up => Action::Ascend,
    };
    let events = w.apply(&action).events;
    assert!(
        (events.iter()).any(|e| matches!(e, Event::LevelChanged { .. })),
        "the stairs are taken"
    );
}

/// The boss of type `id` in the world, wherever it is.
fn bosses(w: &World, id: &str) -> Vec<ActorId> {
    (w.state.actors.values())
        .filter(|a| w.content.creature(a.creature).id == id)
        .map(|a| a.id)
        .collect()
}

/// The player kills actor `id`.
fn kill(w: &mut World, id: ActorId) {
    let (player, now) = (w.state.player, w.state.time);
    let mut events = Vec::new();
    w.dies(id, Some(player), None, now, &mut events);
}

fn has_way_down(w: &World) -> bool {
    (w.state.current().stairs.iter()).any(|s| s.dir == StairsDir::Down)
}

#[test]
fn a_tenth_floor_holds_its_boss_in_its_lair_and_hides_the_way_down() {
    let mut w = start(1);
    jump(&mut w, 10);
    assert_eq!(
        bosses(&w, "gnaegan").len(),
        1,
        "the Rat King keeps the floor"
    );
    assert!(!has_way_down(&w), "no stairs down to see");
    let level = w.state.current();
    let (at, hidden) = level
        .hidden
        .iter()
        .next()
        .expect("the lair's hidden way down");
    assert_eq!(hidden.id, "gnaegan");
    assert_eq!(hidden.to.as_deref(), Some("grundleas:11"));
    assert_eq!(w.content.terrain(level.terrain[*at]).id, "floor");
    assert!(w.content.terrain(hidden.terrain).stairs == Some(StairsDir::Down));
    // The up staircase is as on any floor: the player can always leave.
    assert!((level.stairs.iter()).any(|s| s.dir == StairsDir::Up));
}

#[test]
fn the_bosss_death_opens_the_stairs_where_the_lair_hides_them() {
    let mut w = start(1);
    jump(&mut w, 10);
    let at = *w.state.current().hidden.keys().next().expect("hidden");
    // The player stands beside the way down and sees it.
    // Only the boss is left on the floor: nothing else gets in the way of the stairs.
    let player = w.state.player;
    let keep = bosses(&w, "gnaegan");
    let here = w.state.current_level.clone();
    w.state
        .actors
        .retain(|id, a| *id == player || keep.contains(id) || a.level != here);
    w.state.actors.get_mut(&player).expect("player").pos = at.step(Dir::W);
    // Her lantern lit, to see by.
    w.apply(&Action::UseTool);
    assert!(w.perception.is_visible(at), "the way down is in sight");
    let boss = bosses(&w, "gnaegan")[0];
    kill(&mut w, boss);
    let level = w.state.current();
    assert!(level.hidden.is_empty());
    let down = (level.stairs.iter()).find(|s| s.dir == StairsDir::Down);
    assert_eq!(down.map(|s| s.pos), Some(at));
    assert_eq!(down.and_then(|s| s.to.as_deref()), Some("grundleas:11"));
    assert_eq!(w.content.terrain(level.terrain[at]).id, "stairs_down");
    assert!(w.quests().slain().any(|s| s == "gnaegan"));
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    assert!(
        (said.iter()).any(|m| m == "With Gnægan the Rat King dead, stairs open in the floor."),
        "{said:?}"
    );
    take(&mut w, StairsDir::Down);
    assert_eq!(w.state.current_level, floor(11));
}

#[test]
fn an_unseen_death_is_heard_and_the_stairs_open_all_the_same() {
    let mut w = start(1);
    jump(&mut w, 10);
    let at = *w.state.current().hidden.keys().next().expect("hidden");
    w.perceive(&mut Vec::new());
    let seen = w.perception.is_visible(at);
    let boss = bosses(&w, "gnaegan")[0];
    kill(&mut w, boss);
    assert!(has_way_down(&w));
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    if !seen {
        assert!(said.iter().any(|m| m == "Somewhere stone grinds on stone."));
    }
}

#[test]
fn leaving_with_the_boss_alive_makes_it_again_and_never_twice() {
    let mut w = start(2);
    jump(&mut w, 10);
    let first = bosses(&w, "gnaegan");
    assert_eq!(first.len(), 1);
    take(&mut w, StairsDir::Up);
    assert!(bosses(&w, "gnaegan").is_empty(), "gone with the floor");
    take(&mut w, StairsDir::Up);
    assert_eq!(w.state.current_level, floor(8));
    take(&mut w, StairsDir::Down);
    take(&mut w, StairsDir::Down);
    assert_eq!(w.state.current_level, floor(10));
    let again = bosses(&w, "gnaegan");
    assert_eq!(again.len(), 1, "the boss and its lair are made again");
    assert_ne!(again, first, "a new floor, a new boss");
    assert!(!has_way_down(&w));
    assert_eq!(w.state.current().hidden.len(), 1);
}

#[test]
fn with_the_boss_slain_its_depth_is_an_ordinary_floor() {
    let mut w = start(3);
    jump(&mut w, 10);
    let boss = bosses(&w, "gnaegan")[0];
    kill(&mut w, boss);
    take(&mut w, StairsDir::Up);
    take(&mut w, StairsDir::Down);
    assert_eq!(w.state.current_level, floor(10));
    assert!(bosses(&w, "gnaegan").is_empty(), "slain, never again");
    assert!(has_way_down(&w), "an ordinary way down");
    assert!(w.state.current().hidden.is_empty());
}

#[test]
fn a_boss_slain_away_from_its_lair_opens_nothing_and_stays_slain() {
    let mut w = start(4);
    jump(&mut w, 9);
    // However it came there: it dies on floor 9, which hides nothing.
    let player = w.state.player;
    let at = w.free_cell_near(&floor(9), w.player_pos());
    let spawn = crate::content::CreatureSpawn {
        creature: CONTENT.creature_id("gnaegan").expect("the Rat King"),
        faction: None,
        alignment: None,
        npc: None,
        pos: at,
        route: Vec::new(),
        carry: Vec::new(),
        equipment: Default::default(),
        rolls_gear: false,
        asleep: false,
        status: Vec::new(),
    };
    let boss = travel::make_creature(&mut w.state, &CONTENT, &floor(9), &spawn);
    let stairs = w.state.current().stairs.len();
    kill(&mut w, boss);
    assert_eq!(w.state.current().stairs.len(), stairs);
    assert!(w.state.actors.contains_key(&player));
    take(&mut w, StairsDir::Down);
    assert!(bosses(&w, "gnaegan").is_empty());
    assert!(has_way_down(&w));
}

#[test]
fn the_boss_keeps_to_its_lair_on_the_stairs() {
    let mut w = start(1);
    jump(&mut w, 10);
    let boss = bosses(&w, "gnaegan")[0];
    assert!(w.content.is_layer_boss(w.state.actors[&boss].creature));
    let up = (w.state.current().stairs.iter())
        .find(|s| s.dir == StairsDir::Up)
        .expect("up")
        .pos;
    let player = w.state.player;
    let beside = Dir::ALL
        .iter()
        .map(|d| up.step(*d))
        .find(|p| {
            w.state
                .current()
                .terrain
                .get(*p)
                .is_some_and(|t| w.content.terrain(*t).walkable)
                && w.state.actor_at(&floor(10), *p).is_none()
        })
        .expect("room beside the stairs");
    w.state.actors.get_mut(&player).expect("player").pos = up;
    w.state.actors.get_mut(&boss).expect("boss").pos = beside;
    w.perceive(&mut Vec::new());
    assert!(!w.stair_followers().contains(&boss));
}

#[test]
fn other_floors_are_as_they_were() {
    for depth in [9, 11, 19, 59, 101] {
        let level =
            mapgen::generate(&CONTENT, ("ascmearc", 7), &floor(depth), None).expect("floor");
        assert!(level.hidden.is_empty(), "{depth}");
        assert!(
            (level.stairs.iter()).any(|s| s.dir == StairsDir::Down),
            "{depth}"
        );
    }
}

#[test]
fn every_boss_depth_makes_its_lair_on_rooms_and_caves() {
    let (def, _) = CONTENT.branch_level(&floor(1)).expect("grundleas");
    assert!(def.bosses.len() >= 10);
    for boss in &def.bosses {
        let id = &CONTENT.creature(boss.creature).id;
        for seed in 0..12 {
            let level = mapgen::generate(&CONTENT, ("ascmearc", seed), &floor(boss.depth), None)
                .unwrap_or_else(|e| panic!("{id} at {seed}: {e}"));
            assert!(
                !(level.stairs.iter()).any(|s| s.dir == StairsDir::Down),
                "{id}"
            );
            assert_eq!(
                level.hidden.values().map(|h| &h.id).collect::<Vec<_>>(),
                [id]
            );
            let placed = (level.creatures.iter()).filter(|c| c.creature == boss.creature);
            assert_eq!(placed.count(), 1, "{id} at {seed}");
        }
    }
}

#[test]
fn every_boss_is_unique_and_every_lair_hides_stairs_and_places_it() {
    let (def, _) = CONTENT.branch_level(&floor(1)).expect("grundleas");
    let depths: Vec<u32> = def.bosses.iter().map(|b| b.depth).collect();
    assert_eq!(depths, [10, 20, 30, 40, 50, 60, 70, 80, 90, 100]);
    for boss in &def.bosses {
        let creature = CONTENT.creature(boss.creature);
        assert!(creature.unique, "{}", creature.id);
        let vault = (CONTENT.vaults().iter())
            .find(|v| v.id() == boss.vault)
            .expect("the lair");
        assert!(vault.down.is_some() && vault.pinned, "{}", boss.vault);
        let placed = (vault.template.creatures.iter()).filter(|c| c.creature == boss.creature);
        assert_eq!(placed.count(), 1, "{}", boss.vault);
    }
}

#[test]
fn a_vault_with_stairs_must_be_a_lair_and_a_lair_must_have_them() {
    let stray = "name = \"Stray stair\"\nmap = '''\n#####\n#.>.+\n#####\n'''\n";
    let mut files = ContentFiles::builtin();
    files.insert("vaults/stray_stair.toml", stray);
    let err = Content::load(&files).expect_err("refused").to_string();
    assert!(err.contains("a vault cannot have stairs"), "{err}");
    // A lair without its `>`.
    let mut files = ContentFiles::builtin();
    let path = "campaigns/ascmearc/vaults/gnaegans_midden.toml";
    let lair = std::fs::read_to_string(format!(
        "{}/../../assets/{path}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("the midden")
    .replace('>', ".");
    files.insert(path, &lair);
    let err = Content::load(&files).expect_err("refused").to_string();
    assert!(err.contains("marks no stairs down"), "{err}");
    // A boss that is no unique.
    let mut files = ContentFiles::builtin();
    let path = "campaigns/ascmearc/data/creatures/gnaegan.toml";
    let rat_king = std::fs::read_to_string(format!(
        "{}/../../assets/{path}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("the king")
    .replace("unique = true\n", "")
    .replace("everlasting = true\n", "")
    .replace(PROFANE, "");
    files.insert(path, &rat_king);
    files.remove(SIGNATURE);
    let err = Content::load(&files).expect_err("refused").to_string();
    assert!(err.contains("is not `unique`"), "{err}");
}

/// The Rat King's profane corpse (`docs/dev/profane-gifts.md`), left out where a test takes away
/// what it needs (`everlasting`), with its signature gift.
const PROFANE: &str =
    ", gives = \"rat_kings_court\", profane = { rolls = 1, tiers = { lesser = 100 } }";
const SIGNATURE: &str = "campaigns/ascmearc/data/profane/";

/// Content with the Rat King's file edited: each `from` (which must be there) replaced by `to`,
/// its corpse no longer profane.
fn rat_king_with(edits: &[(&str, &str)]) -> Result<Content, String> {
    let path = "campaigns/ascmearc/data/creatures/gnaegan.toml";
    let mut text = std::fs::read_to_string(format!(
        "{}/../../assets/{path}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("the king");
    for (from, to) in edits {
        assert!(text.contains(from), "{from}");
        text = text.replacen(from, to, 1);
    }
    let text = text.replace(PROFANE, "");
    let mut files = ContentFiles::builtin();
    files.insert(path, &text);
    files.remove(SIGNATURE);
    Content::load(&files).map_err(|e| e.to_string())
}

#[test]
fn everlasting_is_a_uniques_with_a_corpse_and_every_keeper_has_it() {
    let refused = "`everlasting` is for a unique creature that leaves a corpse";
    // On a creature that is no unique.
    let err = rat_king_with(&[("unique = true\n", "")]).expect_err("refused");
    assert!(err.contains(refused), "{err}");
    // On one that leaves no corpse.
    let err = rat_king_with(&[("on_eat = {", "corpse = false\n#on_eat = {")]).expect_err("refused");
    assert!(err.contains(refused), "{err}");
    // An endless branch's keeper without it, or without a corpse.
    let keeper = "layer boss `gnaegan` must leave an everlasting corpse";
    let err = rat_king_with(&[("everlasting = true\n", "")]).expect_err("refused");
    assert!(err.contains(keeper), "{err}");
    let err = rat_king_with(&[
        ("everlasting = true\n", "corpse = false\n"),
        ("on_eat = {", "#on_eat = {"),
    ])
    .expect_err("refused");
    assert!(err.contains(keeper), "{err}");
}
