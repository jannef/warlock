//! Call familiar (`familiar.rs`; spec §8 "Allies": "The familiar", docs/design/witch-familiar.md
//! §7.2 and §16): the refusals and the menu's reasons, the hour, what interrupts it, the draw with
//! the imp from the witch's level 30, a level with no room, the overworld, and determinism.

use super::*;
use crate::ability::{Ability, Unavailable};
use crate::character::{CharacterSpec, RememberedFamiliar};
use crate::content::{ClassActive, Content, ContentFiles};
use crate::perception::Perception;
use crate::test_support::{run, said};
use crate::time::Duration;
use std::sync::{Arc, LazyLock};

const CALL: Ability = Ability::Class(ClassActive::CallFamiliar);

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    for (id, level) in [
        ("fcall_room", ROOM),
        ("fcall_foes", FOES),
        ("fcall_passage", PASSAGE),
        ("fcall_closet", CLOSET),
        ("fcall_cell", CELL),
    ] {
        files.insert(&format!("levels/{id}.toml"), level);
        files.insert(
            &format!("scenarios/{id}.toml"),
            &format!("name = \"{id}\"\nstart_level = \"{id}\"\nhp = 100\n"),
        );
    }
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

const ROOM: &str = r##"
name = "Room"
ambient = [255, 255, 255, 0]
map = '''
########
#@.....#
#......#
########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"##;

const FOES: &str = r##"
name = "Foes"
ambient = [255, 255, 255, 0]
map = '''
############
#@........k#
############
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
"##;

/// A giant rat that comes into view up the passage, close, within a few turns.
const PASSAGE: &str = r##"
name = "Passage"
ambient = [200, 200, 200, 0]
map = '''
#########
#@......#
#####.###
#####.###
#####1###
#########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"1" = { terrain = "floor", creature = "rat", route = "N3 S3" }
"##;

/// No cell but the player's.
const CLOSET: &str = r##"
name = "Closet"
ambient = [255, 255, 255, 0]
map = '''
###
#@#
###
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"##;

/// A kobold walled up where the player cannot see it.
const CELL: &str = r##"
name = "Cell"
ambient = [255, 255, 255, 0]
map = '''
###########
#@...#...k#
###########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
"##;

fn spec(text: &str) -> CharacterSpec {
    text.parse().expect("a character")
}

fn start(content: &Arc<Content>, scenario: &str, character: &str, seed: u64) -> World {
    let spec = spec(character);
    World::with_character(content.clone(), scenario, Some(seed), Some(&spec)).expect("world")
}

fn witch(scenario: &str, seed: u64) -> World {
    start(&CONTENT, scenario, "human,female,witch", seed)
}

fn last(w: &World) -> String {
    said(w).pop().unwrap_or_default()
}

/// The familiar is gone, as if it had died, and the player knows it.
fn lose_familiar(w: &mut World) {
    let id = w.familiar().expect("a familiar");
    w.state.actors.remove(&id);
    *w.perception = Perception::compute(&w.state, &w.content, &mut w.lights);
}

/// She never had one: the familiar is gone and forgotten, as for a witch who gained the talent
/// with none yet.
fn forget_familiar(w: &mut World) {
    lose_familiar(w);
    w.state.character.as_mut().expect("a character").familiar = None;
}

fn reason(w: &World) -> Option<Unavailable> {
    let view = w.perception();
    let row = (view.abilities().into_iter()).find(|a| a.ability == CALL);
    row.expect("Call familiar in the menu").unavailable
}

// ---- the refusals --------------------------------------------------------------------------

#[test]
fn it_is_refused_with_no_time_while_her_familiar_lives() {
    let mut w = witch("fcall_room", 3);
    let (time, kept) = (w.state.time, w.familiar());
    assert!(kept.is_some());
    assert_eq!(reason(&w), Some(Unavailable::FamiliarLives));
    assert_eq!(
        Unavailable::FamiliarLives.reason(w.content()),
        "your familiar lives"
    );
    run(&mut w, "use:call_familiar");
    assert_eq!(last(&w), "Your familiar lives yet.");
    assert_eq!(w.state.time, time);
    assert_eq!(w.familiar(), kept);
}

#[test]
fn it_is_refused_with_a_hostile_in_sight() {
    let mut w = witch("fcall_foes", 3);
    lose_familiar(&mut w);
    let time = w.state.time;
    assert_eq!(reason(&w), Some(Unavailable::Watched));
    assert_eq!(
        Unavailable::Watched.reason(w.content()),
        "something is watching"
    );
    run(&mut w, "use:call_familiar");
    assert_eq!(last(&w), "Not with that watching.");
    assert_eq!(w.state.time, time);
    assert!(w.familiar().is_none());
}

#[test]
fn it_is_refused_without_the_ability_and_while_casting_is_blocked() {
    let mut w = start(&CONTENT, "fcall_room", "human,male,warrior", 3);
    let time = w.state.time;
    run(&mut w, "use:call_familiar");
    assert_eq!(last(&w), "You have no such ability.");
    assert_eq!(w.state.time, time);
    assert!(!(w.perception().abilities().iter()).any(|a| a.ability == CALL));

    let mut w = witch("fcall_room", 3);
    lose_familiar(&mut w);
    assert_eq!(reason(&w), None);
    let silenced = w.content().status_id("silenced").expect("silenced");
    let (time, player) = (w.state.time, w.state.player);
    w.afflict(player, silenced, 1, None, &mut Vec::new());
    assert_eq!(reason(&w), Some(Unavailable::While(silenced)));
    run(&mut w, "use:call_familiar");
    assert_eq!(last(&w), "You are still silenced.");
    assert_eq!(w.state.time, time);
}

#[test]
fn without_the_data_nothing_comes() {
    let mut files = files();
    files.remove("data/familiars.toml");
    let content = Arc::new(Content::load(&files).expect("loads without"));
    let mut w = start(&content, "fcall_room", "human,female,witch", 3);
    assert!(w.familiar().is_none());
    let time = w.state.time;
    run(&mut w, "use:call_familiar");
    assert_eq!(last(&w), "Nothing comes.");
    assert_eq!(w.state.time, time);
}

// ---- the ritual -----------------------------------------------------------------------------

#[test]
fn an_hour_brings_a_new_familiar_named_at_her_grade_at_her_feet() {
    let mut w = witch("fcall_room", 3);
    run(&mut w, "xp:1000 levelup:auto");
    forget_familiar(&mut w);
    let (time, mana) = (w.state.time, w.state.player().mana);
    run(&mut w, "use:call_familiar");
    assert_eq!(w.state.time.since(time), Duration::seconds(3600));
    let id = w.familiar().expect("a new familiar");
    let actor = &w.state.actors[&id];
    let data = actor.familiar().expect("familiar data");
    assert!(!data.name.is_empty());
    assert_eq!(i32::from(data.grade), i32::from(w.player_level()));
    assert_eq!(actor.master(), Some(w.state.player));
    assert_eq!(actor.hp, actor.max_hp(w.content()));
    assert_eq!(actor.pos.chebyshev(w.player_pos()), 1, "at her feet");
    assert!(w.state.player().mana >= mana, "no mana");
    let messages = said(&w);
    assert!(
        messages
            .iter()
            .any(|m| m.contains("You sit down in the dark"))
    );
    let kind = &w.content().creature(actor.creature).name;
    let n = messages.len();
    assert_eq!(
        messages[n - 2],
        "Something comes out of the dark and sits at your feet."
    );
    assert!(
        messages[n - 1].starts_with(&format!("{}, your {kind}, ", data.name)),
        "{messages:?}"
    );
    // She remembers it from now on.
    let remembered = w.state.character.as_ref().and_then(|c| c.familiar.clone());
    assert_eq!(
        remembered,
        Some(RememberedFamiliar {
            creature: actor.creature,
            name: data.name.clone(),
        })
    );
    // The perception knows it at once.
    assert!(w.perception().actors().iter().any(|a| a.id == id));
    assert_eq!(reason(&w), Some(Unavailable::FamiliarLives));
}

#[test]
fn a_familiar_that_dies_may_be_called_again() {
    let mut w = witch("fcall_cell", 3);
    let id = w.familiar().expect("a familiar");
    let kobold = w.content().creature_id("kobold").expect("kobold");
    let foe = (w.state.actors.values())
        .find(|a| a.creature == kobold)
        .map(|a| a.pos)
        .expect("the kobold");
    let actor = w.state.actors.get_mut(&id).expect("the familiar");
    actor.pos = foe.step(crate::geom::Dir::W);
    actor.hp = 1;
    for _ in 0..200 {
        run(&mut w, "wait");
        if w.familiar().is_none() {
            break;
        }
    }
    assert!(w.familiar().is_none(), "the kobold killed it");
    assert_eq!(reason(&w), None);
    run(&mut w, "use:call_familiar");
    let new = w.familiar().unwrap_or_else(|| panic!("{:?}", said(&w)));
    assert_ne!(new, id);
    assert_eq!(w.state.actors[&new].pos.chebyshev(w.player_pos()), 1);
}

#[test]
fn the_familiar_she_lost_comes_back_the_same_at_her_grade_now() {
    let mut w = witch("fcall_room", 3);
    let id = w.familiar().expect("a familiar");
    let (kind, name, grade) = {
        let actor = &w.state.actors[&id];
        let data = actor.familiar().expect("familiar data");
        (actor.creature, data.name.clone(), data.grade)
    };
    run(&mut w, "xp:20000 levelup:auto");
    lose_familiar(&mut w);
    let rng = w.state.rng.clone();
    run(&mut w, "use:call_familiar");
    let new = w.familiar().unwrap_or_else(|| panic!("{:?}", said(&w)));
    assert_ne!(new, id);
    let actor = &w.state.actors[&new];
    let data = actor.familiar().expect("familiar data");
    assert_eq!(actor.creature, kind, "the same kind");
    assert_eq!(data.name, name, "the same name");
    assert!(data.grade > grade, "at the grade her level gives now");
    assert_eq!(i32::from(data.grade), i32::from(w.player_level()));
    assert_eq!(actor.hp, actor.max_hp(w.content()));
    assert_eq!(actor.pos.chebyshev(w.player_pos()), 1, "at her feet");
    assert_eq!(w.state.rng, rng, "no draw on the gameplay stream");
    let kind = &w.content().creature(kind).name;
    let messages = said(&w);
    assert_eq!(
        last(&w),
        format!("{name}, your {kind}, comes back to you out of the dark.")
    );
    assert!(
        !(messages.iter()).any(|m| m.starts_with("Something comes out of the dark")),
        "{messages:?}"
    );
    assert!(w.perception().actors().iter().any(|a| a.id == new));
}

#[test]
fn a_creature_coming_into_view_breaks_it_off_with_the_time_spent() {
    let mut w = witch("fcall_passage", 3);
    lose_familiar(&mut w);
    let time = w.state.time;
    run(&mut w, "use:call_familiar");
    assert!(w.familiar().is_none());
    assert_eq!(
        last(&w),
        "A giant rat comes into view; you break off your calling."
    );
    let spent = w.state.time.since(time);
    assert!(spent > Duration::seconds(0) && spent < Duration::seconds(3600));
}

#[test]
fn with_no_free_cell_nothing_comes_and_the_hour_is_spent() {
    let mut w = witch("fcall_closet", 3);
    assert!(
        w.familiar().is_none(),
        "no room for one at the start either"
    );
    let time = w.state.time;
    run(&mut w, "use:call_familiar");
    assert_eq!(last(&w), "Nothing comes.");
    assert_eq!(w.state.time.since(time), Duration::seconds(3600));
    assert!(w.familiar().is_none());
}

#[test]
fn on_the_overworld_it_joins_the_retinue() {
    let mut w = witch("world", 3);
    let (_, ally) = {
        let id = w.familiar().expect("a familiar");
        (id, w.state.actors[&id].ally.clone().expect("an ally"))
    };
    assert!(ally.retinue);
    lose_familiar(&mut w);
    run(&mut w, "use:call_familiar");
    let id = w.familiar().unwrap_or_else(|| panic!("{:?}", said(&w)));
    assert!(w.state.actors[&id].ally.as_ref().expect("an ally").retinue);
}

// ---- the draw -------------------------------------------------------------------------------

#[test]
fn the_imp_joins_the_draw_from_her_level_30_and_never_a_start() {
    let content = &*CONTENT;
    let rules = content.familiars().expect("familiars.toml");
    let imp = content.creature_id("imp").expect("imp");
    let list = rules.start.list_of("human").expect("a list");
    const DRAWS: u32 = 20_000;
    let count = |level: u8| {
        let mut rng = crate::rng::Rng::stream(1, "fcall/test");
        let (mut imps, mut rares, mut listed) = (0u32, 0u32, 0u32);
        for _ in 0..DRAWS {
            let got = calling_draw(content, &mut rng, "human", level).expect("a beast");
            if got == imp {
                imps += 1;
            } else if rules.start.rare.contains(&got) {
                rares += 1;
            } else {
                assert!(list.contains(&got));
                listed += 1;
            }
        }
        (imps, rares, listed)
    };
    let (imps, rares, listed) = count(29);
    assert_eq!(imps, 0);
    assert_eq!(rares + listed, DRAWS);
    let near = |n: u32, per_mille: u32| n.abs_diff(DRAWS * per_mille / 1000) < DRAWS / 50;
    assert!(near(rares, 50), "{rares} rares");
    // At 30 the imp is one more of the three of a human's list.
    let (imps, rares, listed) = count(30);
    assert!(near(rares, 50), "{rares} rares");
    assert!(near(imps, 950 / 4), "{imps} imps");
    assert!(near(listed, 950 * 3 / 4), "{listed} of the list");
    // A race without a list draws nothing.
    let mut rng = crate::rng::Rng::stream(1, "fcall/test");
    assert_eq!(calling_draw(content, &mut rng, "no race", 30), None);
}

#[test]
fn an_imp_is_her_familiar_as_any_beast_is() {
    let mut w = witch("fcall_room", 3);
    lose_familiar(&mut w);
    let imp = w.content().creature_id("imp").expect("imp");
    let at = w.familiar_cell().expect("a free cell");
    w.make_familiar(imp, at, None);
    let id = w.familiar().expect("the imp");
    let data = w.state.actors[&id].familiar().expect("familiar data");
    assert_eq!(
        last(&w),
        format!(
            "{}, your imp, grins up at you with too many teeth.",
            data.name
        )
    );
    assert_ne!(
        w.stance(w.state.player, id),
        crate::content::Stance::Hostile
    );
    *w.perception = Perception::compute(&w.state, &w.content, &mut w.lights);
    assert_eq!(reason(&w), Some(Unavailable::FamiliarLives));
}

// ---- determinism ----------------------------------------------------------------------------

#[test]
fn a_calling_replays_the_same_and_survives_a_save() {
    let play = || {
        let mut w = witch("fcall_room", 9);
        lose_familiar(&mut w);
        run(&mut w, "use:call_familiar wait");
        w
    };
    let (a, b) = (play(), play());
    assert_eq!(a.state_hash(), b.state_hash());
    let loaded = crate::save::decode(&crate::save::encode(&a), CONTENT.clone()).expect("loads");
    assert_eq!(loaded.state_hash(), a.state_hash());
    assert_eq!(loaded.familiar(), a.familiar());
}

#[test]
fn the_save_keeps_the_familiar_she_remembers_after_it_died() {
    let mut w = witch("fcall_room", 9);
    let remembered = w.state.character.as_ref().and_then(|c| c.familiar.clone());
    assert!(remembered.is_some(), "the start's familiar is remembered");
    lose_familiar(&mut w);
    let mut loaded = crate::save::decode(&crate::save::encode(&w), CONTENT.clone()).expect("loads");
    assert_eq!(loaded.state_hash(), w.state_hash());
    let kept = loaded
        .state
        .character
        .as_ref()
        .and_then(|c| c.familiar.clone());
    assert_eq!(kept, remembered);
    let lost = remembered.as_ref().expect("remembered");
    let kind = &loaded.content().creature(lost.creature).name;
    let dump = crate::dump::ascii(&loaded, &[crate::dump::Layer::Familiar]);
    assert!(
        dump.contains(&format!(
            "no familiar; {}, a {kind}, answers a calling",
            lost.name
        )),
        "{dump}"
    );
    run(&mut loaded, "use:call_familiar");
    let id = loaded.familiar().expect("it came back");
    let data = loaded.state.actors[&id].familiar().expect("familiar data");
    let remembered = remembered.expect("remembered");
    assert_eq!(data.name, remembered.name);
    assert_eq!(loaded.state.actors[&id].creature, remembered.creature);
}
