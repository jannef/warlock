//! Ysella the Binder's quest, "The crystal guardian" (#1446; `campaigns/ascmearc/data/quests/
//! ysella.toml`, `world/people/lure.rs`): asked, she opens the crystal portal beside her into Meyn Bew;
//! the guardian waits in the crystal hollow on its fourth floor, follows the player up every stair
//! and out through the portal, and in Gimmwīc she binds it: at peace for good, its circled status
//! on it, it walks to Kerensa's Stones and stays, and Jewelcraft's checks in the town go better.
//! Killed unbound, it forms again in the hollow. The load checks of a lure quest; a replay.

use crate::content::{ContentFiles, StairsDir};
use crate::test_support::run as act;
use crate::world::*;
use std::sync::LazyLock;

const QUEST: &str = "crystal_guardian";
/// Ysella the Binder, in the garden.
const YSELLA: Pos = Pos::new(67, 19);
/// The crystal portal beside her, hidden until the quest is given.
const PORTAL: Pos = Pos::new(68, 18);
/// The corner of Kerensa's Stones the guardian keeps once bound.
const POST: Pos = Pos::new(67, 4);
const BOTTOM: &str = "meynbew:4";

/// Built-in content with nobody hungering.
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    Arc::new(Content::load(&files).expect("builtin content"))
});

fn town() -> World {
    World::new(CONTENT.clone(), "gimmwic", None).expect("a run in Gimmwīc")
}

/// What the player has been told, and what is still to be told this turn.
fn said(world: &World) -> Vec<String> {
    (world.messages().iter().map(|m| m.text.clone()))
        .chain(world.news.iter().cloned())
        .collect()
}

fn heard(world: &World, start: &str) -> bool {
    said(world).iter().any(|m| m.starts_with(start))
}

fn put_player(world: &mut World, at: Pos) {
    let id = world.state.player;
    world.state.actors.get_mut(&id).expect("player").pos = at;
    world.perceive(&mut Vec::new());
}

fn stage(world: &World) -> Option<Stage> {
    world.state.quests.record(QUEST).map(|r| r.stage)
}

/// Every crystal guardian in the run.
fn guardians(world: &World) -> Vec<&Actor> {
    let content = world.content();
    (world.state.actors.values())
        .filter(|a| content.creature(a.creature).id == "crystal_guardian")
        .collect()
}

fn guardian(world: &World) -> ActorId {
    let all = guardians(world);
    assert_eq!(all.len(), 1, "one guardian");
    all[0].id
}

/// The quest given beside Ysella: the portal is open.
fn given() -> World {
    let mut w = town();
    put_player(&mut w, Pos::new(66, 19));
    act(&mut w, &format!("ask:E:quest-{QUEST}"));
    w
}

/// The cell of level `level`'s staircase going `dir`.
fn stairs(world: &World, level: &str, dir: StairsDir) -> Pos {
    (world.state.levels[level].stairs.iter())
        .find(|s| s.dir == dir)
        .map(|s| s.pos)
        .expect("stairs")
}

/// Creature `id` stands beside the player, who stands on a staircase, and has seen them.
fn beside_player(world: &mut World, id: ActorId) {
    let here = world.state.current_level.clone();
    let at = world.player_pos();
    let cell = world
        .free_cell_within(&here, at, 1, |_| true)
        .expect("room beside the player");
    world.cross_to(id, &here, cell);
    let a = world.state.actors.get_mut(&id).expect("the guardian");
    a.mind.last_seen = Some(at);
    world.perceive(&mut Vec::new());
}

#[test]
fn asking_opens_the_portal_beside_her() {
    let mut w = town();
    let floor = w.state.levels["gimmwic"].terrain[PORTAL];
    assert_eq!(
        w.content().terrain(floor).id,
        "floor",
        "hidden: plain floor"
    );
    assert!(w.state.levels["gimmwic"].stairs_at(PORTAL).is_none());
    put_player(&mut w, Pos::new(66, 19));
    assert_eq!(w.state.actors[&w.state.player].pos.chebyshev(YSELLA), 1);
    act(&mut w, &format!("ask:E:quest-{QUEST}"));
    assert_eq!(stage(&w), Some(Stage::Active));
    let level = &w.state.levels["gimmwic"];
    assert_eq!(
        w.content().terrain(level.terrain[PORTAL]).id,
        "crystal_portal"
    );
    let link = level.stairs_at(PORTAL).expect("a way in");
    assert_eq!(link.to.as_deref(), Some("meynbew:1"));
    assert!(heard(&w, "Ysella sings one long note."), "{:?}", said(&w));
    assert!(guardians(&w).is_empty(), "made only when its floor is");
    // Through the portal, and back out on it.
    put_player(&mut w, PORTAL);
    act(&mut w, "descend");
    assert_eq!(w.state.current_level, "meynbew:1");
    let up = stairs(&w, "meynbew:1", StairsDir::Up);
    assert_eq!(w.player_pos(), up);
    act(&mut w, "ascend");
    assert_eq!(w.state.current_level, "gimmwic");
    assert_eq!(w.player_pos(), PORTAL);
}

#[test]
fn the_guardian_waits_in_the_hollow_and_follows_up_every_stair_and_through_the_portal() {
    let mut w = given();
    put_player(&mut w, PORTAL);
    act(&mut w, "descend");
    w.travel(BOTTOM, None).expect("the bottom");
    let g = guardian(&w);
    let hollow = (w.state.levels[BOTTOM].places.iter())
        .find(|(_, p)| p.name == "the crystal hollow")
        .map(|(&at, _)| at)
        .expect("the hollow");
    assert_eq!(w.state.actors[&g].level, BOTTOM);
    assert!(w.state.actors[&g].pos.chebyshev(hollow) <= 1, "on its bed");
    // Up each floor, the guardian beside the player on the stairs, then through the portal.
    for floor in (1..=4).rev() {
        let level = format!("meynbew:{floor}");
        assert_eq!(w.state.current_level, level);
        let up = stairs(&w, &level, StairsDir::Up);
        put_player(&mut w, up);
        beside_player(&mut w, g);
        act(&mut w, "ascend");
        let to = w.state.current_level.clone();
        let want = if floor == 1 {
            "gimmwic".to_string()
        } else {
            format!("meynbew:{}", floor - 1)
        };
        assert_eq!(to, want);
        let a = &w.state.actors[&g];
        assert_eq!(a.level, to, "it came up from floor {floor}");
        if floor > 1 {
            assert_eq!(a.pos.chebyshev(w.player_pos()), 1, "beside the player");
        }
    }
    assert!(
        heard(&w, "The crystal guardian follows you"),
        "{:?}",
        said(&w)
    );
    // Home: bound, and the quest done.
    assert_eq!(stage(&w), Some(Stage::Done));
    let content = w.content().clone();
    let bound = content.status_id("bound").expect("the binding's status");
    let a = &w.state.actors[&g];
    assert!(a.effects.has(bound));
    assert_eq!(a.mind.post, Some(POST));
    assert_ne!(
        stance_between(&w.state, &content, w.state.player(), a),
        Stance::Hostile
    );
    for line in [
        "Ysella lifts her hands",
        "Ysella the Binder says: \"There. Bound",
        "Quest complete: The crystal guardian.",
    ] {
        assert!(heard(&w, line), "{line}: {:?}", said(&w));
    }
    // It walks to Kerensa's Stones and stays there (the reward's level's picks made first: until
    // they are, every other action is refused).
    act(&mut w, "levelup:auto wait*150");
    assert_eq!(w.state.actors[&g].pos, POST);
    act(&mut w, "wait*50");
    assert_eq!(w.state.actors[&g].pos, POST);
    assert_eq!(w.state.actors[&g].level, "gimmwic");
}

#[test]
fn left_behind_it_stays_and_nothing_is_bound() {
    let mut w = given();
    put_player(&mut w, PORTAL);
    act(&mut w, "descend");
    w.travel(BOTTOM, None).expect("the bottom");
    let g = guardian(&w);
    let up = stairs(&w, BOTTOM, StairsDir::Up);
    put_player(&mut w, up);
    act(&mut w, "ascend");
    assert_eq!(w.state.actors[&g].level, BOTTOM, "not beside the player");
    // Brought up by the dev action and out through the portal.
    w.travel("meynbew:1", None).expect("the first floor");
    let up = stairs(&w, "meynbew:1", StairsDir::Up);
    put_player(&mut w, up);
    act(&mut w, &format!("lure:{QUEST} ascend"));
    assert_eq!(w.state.current_level, "gimmwic");
    assert_eq!(stage(&w), Some(Stage::Done));
    assert_eq!(guardians(&w).len(), 1, "the same one, never a second");
}

#[test]
fn killed_unbound_it_forms_again_in_the_hollow() {
    let mut w = given();
    put_player(&mut w, PORTAL);
    act(&mut w, "descend");
    w.travel(BOTTOM, None).expect("the bottom");
    let g = guardian(&w);
    let now = w.state.time;
    let player = w.state.player;
    w.dies(g, Some(player), None, now, &mut Vec::new());
    assert!(guardians(&w).is_empty());
    assert!(
        heard(&w, "The crystal guardian bursts into shards."),
        "{:?}",
        said(&w)
    );
    assert_eq!(stage(&w), Some(Stage::Active), "nothing fails");
    w.travel("meynbew:3", None).expect("up");
    assert!(guardians(&w).is_empty(), "not until the player comes back");
    w.travel(BOTTOM, None).expect("down again");
    let again = guardian(&w);
    assert_ne!(again, g);
    assert_eq!(w.state.actors[&again].level, BOTTOM);
    assert!(
        heard(&w, "Somewhere on this floor, crystal rings"),
        "{:?}",
        said(&w)
    );
}

#[test]
fn the_bound_guardian_steadies_jewelcraft_in_the_town_only() {
    let mut w = given();
    let jewelcraft = w.content().skill_id("jewelcraft").expect("the skill");
    let metalcraft = w.content().skill_id("metalcraft").expect("the skill");
    assert_eq!(w.binding_boon(jewelcraft), 0, "none before the binding");
    // A ring's check, a recipe made rather than cut or set, takes the boon too (#1490).
    let content = w.content().clone();
    let ring = content.recipe(content.recipes().id("gold_ring").expect("the ring"));
    let unbound = w.craft_modifier(ring, None);
    put_player(&mut w, PORTAL);
    act(&mut w, "descend");
    let up = stairs(&w, "meynbew:1", StairsDir::Up);
    put_player(&mut w, up);
    act(&mut w, &format!("lure:{QUEST}"));
    assert_eq!(w.binding_boon(jewelcraft), 0, "unbound, nothing");
    act(&mut w, "ascend");
    assert_eq!(stage(&w), Some(Stage::Done));
    assert_eq!(w.binding_boon(jewelcraft), 2);
    assert_eq!(w.binding_boon(metalcraft), 0, "Jewelcraft's alone");
    assert_eq!(
        w.craft_modifier(ring, None),
        unbound + 2,
        "a ring's check too"
    );
    // Away from the town, nothing; killed, nothing, and the quest stays done. (The reward's
    // experience brings a level's picks, which refuse every other action until made.)
    act(&mut w, "levelup:auto");
    put_player(&mut w, PORTAL);
    act(&mut w, "descend");
    assert_eq!(w.state.current_level, "meynbew:1");
    assert_eq!(w.binding_boon(jewelcraft), 0);
    act(&mut w, "ascend");
    let g = guardian(&w);
    let now = w.state.time;
    w.dies(g, None, None, now, &mut Vec::new());
    assert_eq!(w.binding_boon(jewelcraft), 0);
    assert_eq!(stage(&w), Some(Stage::Done));
    assert!(
        !heard(&w, "The crystal guardian bursts"),
        "bound, no shatters line"
    );
}

#[test]
fn kerensa_and_ysella_speak_of_it_once_it_is_bound() {
    let content = CONTENT.clone();
    for npc in ["kerensa", "ysella"] {
        let d = (content.dialogues().iter())
            .find(|d| matches!(&d.speaker, crate::content::Speaker::Npc(n) if n == npc))
            .expect("a dialogue");
        assert!(d.after.iter().any(|a| a.quest == QUEST), "{npc}");
        for line in d.after.iter().flat_map(|a| &a.lines) {
            assert!(line.chars().count() <= 72, "{npc}: {line}");
        }
    }
}

#[test]
fn a_lure_quest_is_checked_as_it_loads() {
    let quest = |extra: &str| {
        format!(
            "[[quest]]\nid = \"lure_test\"\nname = \"L\"\ngiver = \"ysella\"\ntype = \"lure\"\n\
             offer = \"o\"\nreminder = \"r\"\ndone = \"d\"\n{extra}"
        )
    };
    let binding = "[quest.binding]\nstatus = \"bound\"\nfaction = \"peaceful\"\npost = [67, 4]\n\
                   words = [\"w\"]\n";
    let good = format!(
        "target = \"crystal_guardian\"\nlevel = \"gimmwic\"\n\
         waits = {{ level = \"meynbew:4\", at = \"the crystal hollow\" }}\n{binding}"
    );
    let load = |text: &str| {
        let mut files = ContentFiles::builtin();
        files.insert("campaigns/ascmearc/data/quests/zz_test.toml", text);
        Content::load(&files).map(|_| ()).map_err(|e| e.to_string())
    };
    load(&quest(&good)).expect("a good one loads");
    for (bad, why) in [
        (
            good.replace("crystal_guardian", "earth_elemental"),
            "no unique",
        ),
        (good.replace("[67, 4]", "[0, 0]"), "no walkable cell"),
        (good.replace("\"bound\"", "\"bleeding\""), "eases away"),
        (
            good.replace("\"gimmwic\"", "\"meynbew:2\""),
            "no hand-made level",
        ),
        (
            good.replace(
                "level = \"meynbew:4\"",
                "creature = \"x\", level = \"meynbew:4\"",
            ),
            "has no `creature`",
        ),
        (good.replace(&binding.to_string(), ""), "needs `binding`"),
    ] {
        let err = load(&quest(&bad)).expect_err(why);
        assert!(err.contains(why), "{why}: {err}");
    }
}
