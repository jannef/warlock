//! Mimics (spec §8 "Mimics", docs/design/chests-and-treasure.md §5, §9; `world/mimic.rs`): a
//! disguised mimic is a chest in every snapshot; a touch reveals it with a free blow and no
//! time; Knock reveals it without one; a blow reveals it; a search and the passive Spot find it;
//! Chest-wise sees it; Chest-wise slips its grip; it takes no turn and no creature sets on it; it
//! counts toward the cap; it carries the chest's loot and drops it; a sneak attack lands on an
//! unaware one; it survives a save and a replay.

use super::fight::Harm;
use super::*;
use crate::action::parse_actions;
use crate::chest::{ChestLook, ChestState};
use crate::content::ContentFiles;
use crate::dump::{self, Layer};
use crate::replay::Replay;
use crate::skills::{SkillId, Trained};

/// A lit hall: the player at the west end of row 2; on row 1 a mimic disguised as a coffer
/// (`M`, x 3) carrying a dagger and coins, and a real coffer (`C`, x 7); a kobold (`k`) in the
/// far corner that hunts the player.
const HALL: &str = r##"
name = "Mimic hall"
ambient = [255, 255, 255, 0]
item_level = 2
map = '''
###########
#..M...C..#
#@........#
#.........#
###########
'''
[legend]
"." = "floor"
"#" = "wall"
"@" = { terrain = "floor", spawn = "player" }
"M" = { terrain = "floor", chest = 2, mimic = true, objects = ["dagger", "gold_coin*12"] }
"C" = { terrain = "chest", objects = ["bread"] }
"##;

/// The same hall with a real coffer where the mimic stands, holding what it carries.
const TWIN: &str = r##"
name = "Mimic hall"
ambient = [255, 255, 255, 0]
item_level = 2
map = '''
###########
#..M...C..#
#@........#
#.........#
###########
'''
[legend]
"." = "floor"
"#" = "wall"
"@" = { terrain = "floor", spawn = "player" }
"M" = { terrain = "chest", chest = 2, objects = ["dagger", "gold_coin*12"] }
"C" = { terrain = "chest", objects = ["bread"] }
"##;

/// The hall with a kobold in the east of row 3, which hunts the player.
const KOBOLDS: &str = r##"
name = "Mimic hall"
ambient = [255, 255, 255, 0]
item_level = 2
map = '''
###########
#..M...C..#
#@........#
#........k#
###########
'''
[legend]
"." = "floor"
"#" = "wall"
"@" = { terrain = "floor", spawn = "player" }
"M" = { terrain = "floor", chest = 2, mimic = true }
"C" = "chest"
"k" = { terrain = "floor", creature = "kobold" }
"##;

const MIMIC: Pos = Pos::new(3, 1);

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/mimic_hall.toml", HALL);
    files.insert("levels/mimic_twin.toml", TWIN);
    files.insert("levels/mimic_kobolds.toml", KOBOLDS);
    for (name, level, extra) in [
        ("mimics", "mimic_hall", "hp = 1000\n"),
        ("twin", "mimic_twin", "hp = 1000\n"),
        ("kobolds", "mimic_kobolds", "hp = 1000\n"),
        (
            "keen",
            "mimic_hall",
            "hp = 1000\nstats = { intelligence = 20, willpower = 20 }\n",
        ),
    ] {
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!("name = \"{name}\"\nstart_level = \"{level}\"\n{extra}"),
        );
    }
    files
}

fn content() -> Arc<Content> {
    Arc::new(Content::load(&files()).expect("fixture content is valid"))
}

fn world(scenario: &str) -> World {
    World::new(content(), scenario, None).expect("world")
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

/// The mimic's actor.
fn mimic(w: &World) -> ActorId {
    (w.state.actors.values())
        .find(|a| w.content().is_mimic(a.creature))
        .map(|a| a.id)
        .expect("a mimic")
}

fn disguised(w: &World) -> bool {
    w.disguised(mimic(w))
}

/// The player steps east along row 2 to stand below x.
fn below(w: &mut World, x: i32) {
    while w.player_pos().x < x {
        w.apply(&Action::Move(Dir::E));
    }
    assert_eq!(w.player_pos(), Pos::new(x, 2));
}

#[test]
fn a_legend_places_a_disguised_mimic_of_the_chests_level_carrying_its_objects() {
    let w = world("mimics");
    let id = mimic(&w);
    let actor = &w.state.actors[&id];
    assert_eq!(actor.pos, MIMIC);
    assert_eq!(w.content().creature(actor.creature).id, "mimic_2");
    assert_eq!(actor.inventory.len(), 2, "the chest's objects");
    assert!(disguised(&w));
    assert!(!w.state.current().chests.contains_key(&MIMIC), "no chest");
    // A mimic is never locked or trapped, and stands on no chest.
    let mut bad = files();
    bad.insert(
        "levels/mimic_hall.toml",
        &HALL.replace("mimic = true,", "mimic = true, locked = true,"),
    );
    assert!(Content::load(&bad).is_err());
    let mut bad = files();
    bad.insert(
        "levels/mimic_hall.toml",
        &HALL.replace(
            "{ terrain = \"floor\", chest = 2, mimic",
            "{ terrain = \"chest\", chest = 2, mimic",
        ),
    );
    assert!(Content::load(&bad).is_err());
}

#[test]
fn a_disguised_mimic_is_a_chest_in_every_snapshot() {
    let (mut w, mut twin) = (world("mimics"), world("twin"));
    for _ in 0..2 {
        let strip = |w: &World| {
            let mut json = dump::json(w, &[Layer::View, Layer::Memory], false);
            let object = json.as_object_mut().expect("an object");
            for key in ["hash", "level", "scenario", "messages"] {
                object.remove(key);
            }
            json
        };
        assert_eq!(strip(&w), strip(&twin));
        assert_eq!(
            w.perception().chest_at(MIMIC),
            Some(ChestLook {
                band: 0,
                state: ChestState::Shut,
                locked: None
            })
        );
        assert!(w.perception().actor_at(MIMIC).is_none());
        run(&mut w, "wait");
        run(&mut twin, "wait");
    }
}

#[test]
fn a_touch_reveals_it_with_a_free_blow_and_no_time() {
    for actions in ["move:N", "open:N", "kick:N"] {
        let mut w = world("mimics");
        below(&mut w, 3);
        let (turn, hp) = (w.state.turn, w.state.player().hp);
        run(&mut w, actions);
        assert!(!disguised(&w), "{actions}");
        assert!(
            said(&w, "The coffer lurches: it is a mimicling!"),
            "{actions}"
        );
        assert_eq!(w.state.turn, turn, "{actions}: no time");
        let struck = (w.messages().iter()).any(|m| m.text.contains("mimicling"));
        assert!(struck && w.state.player().hp <= hp, "{actions}: its blow");
        // Revealed, it is a creature: the next step into it is an attack.
        assert!(w.perception().actor_at(MIMIC).is_some(), "{actions}");
    }
}

#[test]
fn knock_reveals_it_without_a_blow_and_spends_the_mana() {
    let mut w = world("mimics");
    let me = w.player_mut();
    me.talents.take("knock", None);
    me.mana_base = 40;
    me.mana = 40;
    below(&mut w, 3);
    run(&mut w, "use:knock:N");
    assert!(!disguised(&w));
    assert!(said(&w, "The coffer lurches: it is a mimicling!"));
    assert_eq!(w.state.player().mana, 35);
    // The reveal itself strikes no blow (the mimic's own turn may, after).
    let mut w = world("mimics");
    let (id, hp) = (mimic(&w), w.state.player().hp);
    w.reveal_mimic(id, super::mimic::Reveal::Knocked, &mut Vec::new());
    assert!(!disguised(&w));
    assert_eq!(w.state.player().hp, hp, "no blow");
}

#[test]
fn a_blow_or_harm_reveals_it() {
    // A missile stops at it.
    let mut w = world("mimics");
    w.player_mut().inventory.clear();
    let dagger = w.content().object_id("dagger").expect("dagger");
    w.player_mut()
        .inventory
        .push(crate::object::Stack::one(dagger));
    run(&mut w, "throw:a:3,1");
    assert!(!disguised(&w));
    assert!(said(&w, "The coffer shudders: it is a mimicling!"));
    // A burst.
    let mut w = world("mimics");
    let mut events = Vec::new();
    let id = mimic(&w);
    w.hurt(id, 1, Harm::Place, &mut events);
    assert!(!disguised(&w));
}

#[test]
fn searching_and_spot_find_it_out() {
    let mut w = world("keen");
    below(&mut w, 3);
    run(&mut w, "search:100");
    assert!(!disguised(&w));
    assert!(said(&w, "You notice the coffer breathing: a mimicling!"));
    // In passing, only those who notice everything notice it: trained in Spot.
    let mut w = world("keen");
    below(&mut w, 2);
    run(&mut w, "wait*30");
    assert!(disguised(&w), "untrained: never");
    let spot = w.content().skill_id("spot").expect("spot");
    let skills = &mut w.player_mut().skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(s, _)| *s != spot).collect();
    all.push((
        spot,
        Trained {
            value: 60,
            marks: 0,
        },
    ));
    *skills = all.into_iter().collect();
    run(&mut w, "wait*30");
    assert!(!disguised(&w), "trained in Spot");
}

#[test]
fn chest_wise_sees_a_mimic_it_close_by_and_it_stays_disguised() {
    let mut w = world("mimics");
    w.player_mut().talents.take("chest_wise", None);
    below(&mut w, 6);
    assert!(w.perception().actor_at(MIMIC).is_none(), "too far");
    run(&mut w, "move:W");
    assert!(w.perception().actor_at(MIMIC).is_some(), "within two cells");
    assert_eq!(w.perception().chest_at(MIMIC), None, "no chest to them");
    assert!(disguised(&w), "the rules still see a chest");
}

#[test]
fn chest_wise_slips_its_grip() {
    let held = |slick: bool| {
        let mut w = world("mimics");
        if slick {
            w.player_mut().talents.take("chest_wise", None);
        }
        let id = mimic(&w);
        let player = w.state.player;
        let status = w.content().status_id("held").expect("held");
        let mut events = Vec::new();
        w.afflict(player, status, 3, Some(id), &mut events);
        w.state.player().effects.has(status)
    };
    assert!(held(false));
    assert!(!held(true));
}

#[test]
fn it_takes_no_turn_and_no_creature_sets_on_it() {
    let mut w = world("kobolds");
    let id = mimic(&w);
    let kobold = (w.state.actors.values())
        .find(|a| w.content().creature(a.creature).id == "kobold")
        .map(|a| a.id)
        .expect("kobold");
    // The kobold and the mimic: the vermin and the kobolds leave each other be; and the AI
    // never targets a hidden thing.
    run(&mut w, "rest:1h");
    assert_eq!(w.state.actors[&id].pos, MIMIC, "it never moved");
    assert!(disguised(&w));
    assert_eq!(
        w.state.actors[&id].hp,
        w.state.actors[&id].max_hp(w.content())
    );
    let _ = kobold;
    // It counts as a hostile on the level (the spawn cap).
    assert!(w.stance(w.state.player, id) == Stance::Hostile);
}

#[test]
fn killed_it_drops_the_chests_loot() {
    let mut w = world("mimics");
    let id = mimic(&w);
    let mut events = Vec::new();
    let player = w.state.player;
    let hp = w.state.actors[&id].hp;
    w.hurt(id, hp, Harm::Actor(player), &mut events);
    assert!(!w.state.actors.contains_key(&id));
    let pile = w.state.current().objects_at(MIMIC);
    let dagger = w.content().object_id("dagger").expect("dagger");
    assert!(
        pile.iter().any(|s| s.object == dagger),
        "the chest's dagger"
    );
}

#[test]
fn a_disguised_mimic_is_unaware_so_a_sneak_attack_lands() {
    let mut w = world("mimics");
    let id = mimic(&w);
    let player = w.state.player;
    assert!(w.unaware(id, player), "disguised, it has seen no one");
}

#[test]
fn mimics_survive_a_save_and_replay_exactly() {
    let mut w = world("mimics");
    below(&mut w, 3);
    run(&mut w, "move:N");
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    let content = content();
    let mut w = World::new(content.clone(), "mimics", None).expect("world");
    let mut replay = Replay::new(&w);
    for action in parse_actions("move:E*2 move:N move:N wait").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let (verified, divergence) = replay.play(content.clone()).expect("replays");
    assert_eq!(divergence, None);
    assert_eq!(verified.state_hash(), w.state_hash());
}

#[test]
fn the_dumps_hide_it_but_the_dev_layers() {
    let w = world("mimics");
    let view = dump::ascii(&w, &[Layer::View]);
    let truth = dump::ascii(&w, &[Layer::Truth]);
    let row = |dump: &str| dump.lines().nth(3).map(str::to_string).unwrap_or_default();
    assert!(!row(&view).contains('m'), "{view}");
    assert!(row(&truth).contains('m'), "{truth}");
    let actors = dump::ascii(&w, &[Layer::Actors]);
    assert!(actors.contains("disguised"), "{actors}");
    let json = dump::json(&w, &[], false);
    assert_eq!(json["chests"].as_array().map(Vec::len), Some(2));
}
