//! Throwing one's voice (spec §5 "Hearing"; #235): where it may go, what it costs, who hears it,
//! and who goes to look.

use super::*;
use crate::content::ContentFiles;
use crate::noise::SoundKind;

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
"##;

const FACTIONS: &str = r##"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "vermin"
name = "vermin"
hostile = ["player"]

[[faction]]
id = "wildlife"
name = "wildlife"
"##;

const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 1000
hearing = 10

[[creature]]
id = "rat"
name = "giant rat"
glyph = "r"
fg = "#806040"
weight = 1000
base_hp = 500
faction = "vermin"
hearing = 5

[[creature]]
id = "deer"
name = "deer"
glyph = "d"
fg = "#c09060"
weight = 1000
base_hp = 500
faction = "wildlife"
hearing = 5
"##;

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "rat" }
"d" = { terrain = "floor", creature = "deer" }
"##;

/// The player in a room, a rat and a deer in the passage below it, round a corner from the
/// room's way out at (7, 2): they hear what is said near it, but do not see it. No vision file:
/// every eye needs no light, so sight is line of sight.
const MAP: &str = "############\n#.@........#\n#######.####\n#...r....d.#\n############\n";

const RAT: ActorId = ActorId(1);
const DEER: ActorId = ActorId(2);

fn world() -> World {
    let mut files = ContentFiles::default();
    files.insert("data/terrain.toml", TERRAIN);
    files.insert("data/factions.toml", FACTIONS);
    files.insert("data/creatures.toml", CREATURES);
    files.insert("data/legend.toml", LEGEND);
    files.insert(
        "levels/room.toml",
        &format!("name = \"Room\"\nmap = '''\n{MAP}'''\n"),
    );
    files.insert(
        "scenarios/room.toml",
        "name = \"R\"\nstart_level = \"room\"\nhp = 500\n",
    );
    // The bard's Throw your voice, which gives the ability (#244).
    files.insert("data/talents.toml", TALENTS);
    files.insert("data/talents/throw_voice.toml", THROW_VOICE);
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    let mut w = World::new(content, "room", None).expect("world");
    w.player_mut().talents.take("throw_voice", None);
    w
}

const TALENTS: &str = r#"
[coverage]
min_talents = 1
min_class = 0
free_per_tree = 0
[[tree]]
id = "class"
name = "Class"
description = "Class abilities."
class = true
[[system]]
id = "hearing"
name = "Hearing"
pending = true
"#;

const THROW_VOICE: &str = r#"
[[talent]]
id = "throw_voice"
name = "Throw your voice"
tree = "class"
granted = true
description = "A voice where you will."
effects = [{ kind = "throw_voice" }]
systems = ["hearing"]
"#;

fn last_message(w: &World) -> String {
    w.messages()
        .iter()
        .last()
        .map(|m| m.text.clone())
        .unwrap_or_default()
}

#[test]
fn a_thrown_voice_draws_creatures_with_enemies_to_look() {
    let mut w = world();
    assert_eq!(w.state().actors[&RAT].pos, Pos::new(4, 3));
    let at = Pos::new(7, 1);
    let outcome = w.apply(&Action::ThrowVoice(at));
    assert!(outcome.took_turn);
    assert_eq!(outcome.elapsed, Duration::seconds(6), "a normal action");
    assert_eq!(outcome.events, [Event::VoiceThrown { at }]);
    assert_eq!(last_message(&w), "You throw your voice.");
    // The rat heard it round the corner, did not see where, and set off to look.
    let mind = w.state().actors[&RAT].mind;
    assert_eq!(mind.noise, Some(at));
    assert_eq!(mind.post, Some(Pos::new(4, 3)));
    assert_eq!(w.state().actors[&RAT].pos, Pos::new(5, 3), "it set off");
    // The deer heard it too, and does not care.
    assert!(w.state().actors[&DEER].mind.is_calm());
    assert_eq!(w.state().actors[&DEER].pos, Pos::new(9, 3));
    // The player made it: they do not hear their own voice.
    assert!(w.perception().sounds().is_empty());
    assert!(!w.messages().iter().any(|m| m.text.contains("You hear")));
}

#[test]
fn the_voice_is_heard_where_it_sounds_not_where_the_player_stands() {
    // Thrown to the far end of the room, it is too far from the way out to carry round the
    // corner; the player's own cell is farther still.
    let mut w = world();
    w.apply(&Action::ThrowVoice(Pos::new(3, 1)));
    assert!(w.state().actors[&RAT].mind.is_calm());
    assert_eq!(w.state().actors[&RAT].pos, Pos::new(4, 3));
    let plain = crate::noise::plain_noise(
        w.content(),
        w.state().current(),
        w.state().player(),
        crate::burden::Burden::Unburdened,
        crate::noise::Act::ThrowVoice(Pos::new(3, 1)),
    )
    .expect("a voice");
    assert_eq!(plain.kind, SoundKind::Voice);
    assert_eq!(plain.at, Pos::new(3, 1));
    assert_eq!(plain.loudness, crate::noise::VOICE_NOISE);
}

#[test]
fn the_voice_goes_only_to_a_place_the_player_sees_within_reach() {
    let mut w = world();
    let cases = [
        (Pos::new(2, 1), "You cannot throw your voice at yourself."),
        (Pos::new(9, 1), "That is out of range."),
        (
            Pos::new(5, 3),
            "You can throw your voice only to a place you see.",
        ),
        (
            Pos::new(2, 0),
            "You can throw your voice only to a place you see.",
        ),
    ];
    for (at, said) in cases {
        let outcome = w.apply(&Action::ThrowVoice(at));
        assert!(!outcome.took_turn, "{at:?}");
        assert!(outcome.events.is_empty(), "{at:?}");
        assert_eq!(last_message(&w), said, "{at:?}");
        assert_eq!(w.turn(), 0);
    }
    assert!(w.state().actors[&RAT].mind.is_calm(), "nothing was heard");
    assert_eq!(Pos::new(2, 1).dist2(Pos::new(8, 1)), 36);
    assert!(
        36 <= crate::fov::radius2(VOICE_RANGE),
        "six cells away is within reach"
    );
}
