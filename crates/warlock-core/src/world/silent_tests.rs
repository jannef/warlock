//! The silent vault (spec §7 "Special rooms", design §9.1): every noise inside at 1 %, and
//! Silenced on everyone inside, which refuses casting, reading, and throwing one's voice, a
//! creature's spells too; it eases soon after leaving.

use super::*;
use crate::action::{Rest, parse_actions};
use crate::content::{ActionKind, ContentFiles, StatusId};
use crate::time::Duration;

/// A hall, 25 wide: the west part plain, the east part (x 14 on, walls included) a silent vault
/// when `room`. A creature stands at `k` inside it.
const HALL: &str = "\
#########################
#.......................#
#.......................#
#...@...............k...#
#.......................#
#.......................#
#########################
";

/// The built-in content with [`HALL`], its east part a silent vault when `room`, `creature` at
/// `k` (of faction `faction`, its own when empty).
fn world(room: bool, creature: &str, faction: &str) -> World {
    let mut files = ContentFiles::builtin();
    let faction = if faction.is_empty() {
        String::new()
    } else {
        format!(", faction = \"{faction}\"")
    };
    let room = if room {
        "[[room]]\neffect = \"silent\"\narea = [14, 0, 24, 6]\n"
    } else {
        ""
    };
    files.insert(
        "levels/test_room.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{HALL}'''\n{room}\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"k\" = {{ terrain = \"floor\", creature = \"{creature}\"{faction} }}\n"
        ),
    );
    files.insert(
        "scenarios/test_room.toml",
        "name = \"T\"\nstart_level = \"test_room\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_room", None).expect("world")
}

fn silenced(w: &World) -> StatusId {
    w.content.status_id("silenced").expect("silenced")
}

fn creature(w: &World) -> &Actor {
    (w.state.actors.values())
        .find(|a| a.kind == ActorKind::Creature)
        .expect("a creature")
}

fn run(w: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|a| w.apply(a))
        .collect()
}

fn last(w: &World) -> String {
    w.messages()
        .iter()
        .last()
        .map(|m| m.text.clone())
        .unwrap_or_default()
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

#[test]
fn the_silent_vault_is_data_a_hush_and_a_status() {
    let w = world(true, "kobold", "peaceful");
    let silent = w.content.room_effects().id("silent").expect("silent");
    let def = w.content.room_effect(silent);
    assert_eq!(def.name, "silent vault");
    assert_eq!(def.noise, 1);
    assert_eq!(def.status, Some(silenced(&w)));
    assert_eq!(def.level_chance, 0, "never a whole level (design §3.1)");
    let status = w.content.status(silenced(&w));
    assert_eq!(
        status.blocks,
        [ActionKind::Cast, ActionKind::Read, ActionKind::ThrowVoice]
    );
    assert_eq!(status.eases, Some(10));
    assert!(!status.touches_only_mana(), "creatures without mana too");
}

#[test]
fn inside_the_player_cannot_cast_read_or_throw_their_voice_and_sound_returns_after() {
    let mut w = world(true, "kobold", "peaceful");
    // Outside nothing is refused by a status (the spell is simply unknown).
    run(&mut w, "use:blessing");
    assert_ne!(last(&w), "You cannot cast spells while silenced.");
    w.player_mut().pos = Pos::new(16, 3);
    run(&mut w, "wait");
    assert!(
        w.state.player().effects.has(silenced(&w)),
        "inside: Silenced"
    );
    assert!(said(&w, "The silence closes around you."));
    for (action, line) in [
        ("use:blessing", "You cannot cast spells while silenced."),
        ("read:a", "You cannot read while silenced."),
        (
            "throw_voice:18,2",
            "You cannot throw your voice while silenced.",
        ),
    ] {
        let (turn, time) = (w.state.turn, w.state.time);
        let out = run(&mut w, action).remove(0);
        assert!(!out.took_turn, "{action} is refused");
        assert_eq!(
            (w.state.turn, w.state.time),
            (turn, time),
            "{action}: no time"
        );
        assert_eq!(last(&w), line);
    }
    // Spending mana is casting too: the dev drain is refused.
    w.player_mut().mana_base = 10;
    w.player_mut().mana = 10;
    run(&mut w, "drain:1");
    assert_eq!(w.state.player().mana, 10, "no mana spent while silenced");
    // A rest inside keeps it: it never lapses mid-rest.
    w.apply(&Action::Rest(Rest::For(Duration::minutes(5))));
    assert!(w.state.player().effects.has(silenced(&w)));
    // Out of the room it eases within ten seconds, then everything is back.
    w.player_mut().pos = Pos::new(4, 3);
    run(&mut w, "wait*3");
    assert!(!w.state.player().effects.has(silenced(&w)), "eased away");
    assert!(said(&w, "Sound returns."));
    run(&mut w, "use:blessing");
    assert_ne!(last(&w), "You cannot cast spells while silenced.");
}

#[test]
fn noises_inside_are_a_hundredth_as_loud() {
    let w = world(true, "kobold", "peaceful");
    let plain = world(false, "kobold", "peaceful");
    let inside = Pos::new(18, 3);
    let here = w.effects_at("test_room", inside);
    assert_eq!(here.noise(), 1);
    assert_eq!(plain.effects_at("test_room", inside).noise(), 100);
}

#[test]
fn creatures_inside_are_silenced_and_a_caster_fights_in_melee() {
    // A peaceful kobold inside holds Silenced too, though it has no mana.
    let mut w = world(true, "kobold", "peaceful");
    run(&mut w, "wait*2");
    assert!(creature(&w).effects.has(silenced(&w)));
    // A hostile kobold shaman four cells from the player, both outside the vault: it casts
    // from where it stands.
    let shaman = |room: bool| {
        let mut w = world(room, "kobold_shaman", "");
        w.player_mut().pos = Pos::new(16, 3);
        w.player_mut().hp = 1000;
        // Standing in the vault as the run starts, it is silenced before it acts.
        assert_eq!(creature(&w).effects.has(silenced(&w)), room);
        run(&mut w, "wait*3");
        let cast = (w.messages().iter()).any(|m| m.text.contains("casts a spell"));
        (cast, creature(&w).pos.x)
    };
    let (cast, x) = shaman(false);
    assert!(cast, "outside a vault the shaman casts");
    assert_eq!(x, 20, "and keeps its distance");
    // Inside it cannot: it closes in to fight with its claws.
    let (cast, x) = shaman(true);
    assert!(!cast, "silenced, the shaman casts nothing");
    assert!(x < 20, "it closes in: x {x}");
}

#[test]
fn silenced_saves_and_replays() {
    let mut w = world(true, "kobold", "peaceful");
    w.player_mut().pos = Pos::new(16, 3);
    run(&mut w, "wait");
    let restored = World::from_state(w.content().clone(), w.state().clone()).expect("restores");
    assert!(restored.state.player().effects.has(silenced(&w)));
    assert_eq!(restored.state_hash(), w.state_hash());
}
