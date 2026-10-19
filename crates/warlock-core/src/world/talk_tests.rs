//! Talk (spec §8 "Dialogue", #146) and making way (spec §8 "Displacement", #147) in the turn
//! pipeline: chatting, topics, non-answers, mutters heard and seen, swapping places and the
//! refusals, patrols carrying on, rests, saves, and replays.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::noise::SoundKind;
use crate::replay::Replay;

const TERRAIN: &str = r##"
[[terrain]]
id = "void"
name = "rock"
glyph = " "
fg = "#000000"
walkable = false
transparent = false

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

const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000
footsteps = 50
hearing = 10

[[creature]]
id = "sage"
name = "sage"
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
id = "goat"
name = "goat"
glyph = "g"
fg = "#ffffff"
faction = "folk"
weight = 40000

[[creature]]
id = "brigand"
name = "brigand"
glyph = "@"
fg = "#ff4040"
faction = "outlaws"
weight = 70000

[[creature]]
id = "sentry"
name = "sentry"
glyph = "@"
fg = "#8080ff"
faction = "folk"
budge = "never"
weight = 80000

[[creature]]
id = "warden"
name = "warden"
glyph = "@"
fg = "#80ff80"
faction = "folk"
budge = "off_post"
weight = 80000

[[creature]]
id = "ogre"
name = "ogre"
glyph = "O"
fg = "#a0a040"
faction = "folk"
size = "large"
weight = 200000

[[creature]]
id = "giant"
name = "giant"
glyph = "H"
fg = "#a0a040"
faction = "folk"
size = "huge"
weight = 900000

[[creature]]
id = "monk"
name = "monk"
glyph = "@"
fg = "#ffd080"
faction = "folk"
weight = 60000
"##;

const FACTIONS: &str = r##"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "folk"
name = "folk"

[[faction]]
id = "outlaws"
name = "outlaws"
hostile = ["player"]
"##;

const SIZES: &str = r##"
[[size]]
id = "small"
name = "Small"
carry = 75

[[size]]
id = "medium"
name = "Medium"
carry = 100

[[size]]
id = "large"
name = "Large"
carry = 150

[[size]]
id = "huge"
name = "Huge"
carry = 200
"##;

const DIALOGUE: &str = r##"
[[dialogue]]
creature = "sage"
greet = "Welcome, stranger."
lines = ["One.", "Two.", "Three."]

[[dialogue.topic]]
id = "weather"
name = "The weather"
reply = "Fair, for now."

[[dialogue.topic]]
id = "road"
name = "The road"
reply = "Long, and uphill both ways."

[[dialogue]]
npc = "elder"
lines = ["I am the elder."]

[[dialogue]]
creature = "crone"
lines = ["Hm."]
mutter = { chance = 100, verb = "hums", lines = ["La.", "Lo."] }

[[dialogue]]
creature = "brigand"
lines = ["Your money or your life."]

[[dialogue]]
creature = "monk"
lines = ["Peace."]
"##;

const MATERIALS: &str = "[[material]]\nid = \"stone\"\nname = \"stone\"\n";
const OBJECTS: &str = r##"
[[object]]
id = "boulder"
name = "boulder"
glyph = "*"
fg = "#808080"
material = "stone"
weight = 100000
"##;

const LEGEND: &str = r##"
[legend]
" " = "void"
"." = "floor"
"#" = "wall"
"##;

const VISION: &str = r##"
[[vision]]
id = "normal"
name = "Normal"
threshold = 40
"##;

// A lit parlor: the sage east of the player, a goat, and a sage who is the elder.
const PARLOR: &str = r##"
name = "Parlor"
ambient = [255, 255, 255, 0]
map = '''
#########
#@S.g...#
#.......#
#E......#
#########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"S" = { terrain = "floor", creature = "sage" }
"g" = { terrain = "floor", creature = "goat" }
"E" = { terrain = "floor", creature = "sage", npc = "elder" }
"##;

// A lit den: a brigand next to the player.
const DEN: &str = r##"
name = "Den"
ambient = [255, 255, 255, 0]
map = '''
#####
#@b.#
#####
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"b" = { terrain = "floor", creature = "brigand" }
"##;

// A room with the crone at `at`, lit or dark.
fn crone_room(name: &str, lit: bool, at: (usize, usize)) -> String {
    let mut rows: Vec<Vec<char>> = [
        "###########",
        "#@........#",
        "#.........#",
        "#.........#",
        "#.........#",
        "#.........#",
        "###########",
    ]
    .iter()
    .map(|r| r.chars().collect())
    .collect();
    rows[at.1][at.0] = 'c';
    let map: Vec<String> = rows.into_iter().map(|r| r.into_iter().collect()).collect();
    let ambient = if lit {
        "ambient = [255, 255, 255, 0]\n"
    } else {
        ""
    };
    format!(
        "name = \"{name}\"\n{ambient}map = '''\n{}\n'''\n[legend]\n\
         \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
         \"c\" = {{ terrain = \"floor\", creature = \"crone\" }}\n",
        map.join("\n")
    )
}

// A lit yard: those who will not make way around the player, and a walker on its round.
const YARD: &str = r##"
name = "Yard"
ambient = [255, 255, 255, 0]
map = '''
############
#..........#
#.s@m......#
#.w.o.H....#
#..........#
#.....k....#
############
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"s" = { terrain = "floor", creature = "sentry" }
"m" = { terrain = "floor", creature = "goat", carry = ["boulder"] }
"w" = { terrain = "floor", creature = "warden" }
"o" = { terrain = "floor", creature = "ogre" }
"H" = { terrain = "floor", creature = "giant" }
"k" = { terrain = "floor", creature = "goat", route = "E4 W4" }
"##;

// The same yard in the dark.
fn dark_yard() -> String {
    YARD.replace("ambient = [255, 255, 255, 0]\n", "")
}

// A lit room; below it, out of sight, a monk (who talks) or a goat walks up a corridor into the
// room and back, silently.
fn cloister(walker: &str) -> String {
    format!(
        r##"
name = "Cloister"
ambient = [255, 255, 255, 0]
map = '''
##########
#@.......#
#........#
######.###
     #.#
     #.#
     #.#
     #.#
     #w#
     ###
'''
[legend]
"@" = {{ terrain = "floor", spawn = "player" }}
"w" = {{ terrain = "floor", creature = "{walker}", route = "N6 S6" }}
"##
    )
}

fn files() -> ContentFiles {
    let mut files = ContentFiles::default();
    files.insert("data/terrain.toml", TERRAIN);
    files.insert("data/creatures.toml", CREATURES);
    files.insert("data/factions.toml", FACTIONS);
    files.insert("data/sizes.toml", SIZES);
    files.insert("data/dialogue.toml", DIALOGUE);
    files.insert("data/materials.toml", MATERIALS);
    files.insert("data/objects.toml", OBJECTS);
    files.insert("data/legend.toml", LEGEND);
    files.insert("data/vision.toml", VISION);
    files.insert("data/actions.toml", "chat = 500\n");
    let levels = [
        ("parlor", PARLOR.to_string()),
        ("den", DEN.to_string()),
        ("near", crone_room("Near", false, (3, 1))),
        ("faint", crone_room("Faint", false, (5, 1))),
        ("far", crone_room("Far", false, (7, 1))),
        ("hall", crone_room("Hall", true, (3, 1))),
        ("yard", YARD.to_string()),
        ("dark_yard", dark_yard()),
        ("cloister", cloister("monk")),
        ("pen", cloister("goat")),
    ];
    for (id, text) in levels {
        files.insert(&format!("levels/{id}.toml"), &text);
        files.insert(
            &format!("scenarios/{id}.toml"),
            &format!("name = \"{id}\"\nstart_level = \"{id}\"\n"),
        );
    }
    files
}

fn content() -> Arc<Content> {
    Arc::new(Content::load(&files()).expect("test content is valid"))
}

fn world(scenario: &str) -> World {
    World::new(content(), scenario, None).expect("world")
}

fn run(world: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|a| world.apply(a))
        .collect()
}

fn texts(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.text.clone()).collect()
}

fn last_message(world: &World) -> String {
    world
        .messages()
        .iter()
        .last()
        .map(|m| m.text.clone())
        .unwrap_or_default()
}

fn pos(world: &World, id: u32) -> Pos {
    world.state().actors[&ActorId(id)].pos
}

fn talked(out: &Outcome) -> Option<&Conversation> {
    out.events.iter().find_map(|e| match e {
        Event::Talked(talk) => Some(talk),
        _ => None,
    })
}

#[test]
fn chatting_greets_once_then_rotates_the_lines() {
    let mut w = world("parlor");
    let start = w.time();
    let out = w.apply(&Action::Chat(Dir::E));
    assert!(out.took_turn);
    assert_eq!(out.elapsed, Duration::seconds(3), "a chat costs 500: 3 s");
    assert_eq!(last_message(&w), "The sage says: \"Welcome, stranger.\"");
    let talk = talked(&out).expect("an answer");
    assert_eq!(talk.with, ActorId(1));
    assert_eq!(talk.asked, None);
    let topics: Vec<&str> = talk.topics.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(topics, ["The weather", "The road"]);
    // The lines in turn: each once a round, never the same twice in a row.
    let mut said = Vec::new();
    for _ in 0..9 {
        w.apply(&Action::Chat(Dir::E));
        said.push(last_message(&w));
    }
    for round in said.chunks(3) {
        let mut lines = round.to_vec();
        lines.sort();
        let expected: Vec<String> = ["One.", "Three.", "Two."]
            .iter()
            .map(|l| format!("The sage says: \"{l}\""))
            .collect();
        assert_eq!(lines, expected);
    }
    assert!(said.windows(2).all(|p| p[0] != p[1]), "{said:?}");
    assert!(!said.iter().any(|l| l.contains("Welcome")));
    let talk = w.state().actors[&ActorId(1)].talk;
    assert_eq!((talk.met, talk.said), (true, 9));
    assert_eq!(w.time().since(start), Duration::seconds(30));
    // The same run says the same lines.
    let mut again = world("parlor");
    run(&mut again, &"chat:E ".repeat(10));
    assert_eq!(texts(&again), texts(&w));
}

#[test]
fn asking_about_a_topic_answers_and_takes_no_time() {
    let mut w = world("parlor");
    w.apply(&Action::Chat(Dir::E));
    let (time, turn) = (w.time(), w.turn());
    let ask = |topic: &str| Action::Ask {
        dir: Dir::E,
        topic: topic.into(),
    };
    let out = w.apply(&ask("road"));
    assert!(!out.took_turn);
    assert_eq!((w.time(), w.turn()), (time, turn));
    assert_eq!(
        last_message(&w),
        "The sage says: \"Long, and uphill both ways.\""
    );
    let talk = talked(&out).expect("an answer");
    assert_eq!(talk.asked.as_deref(), Some("road"));
    assert_eq!(talk.topics.len(), 2);
    // An unknown topic, and asking where no one stands.
    let out = w.apply(&ask("dragons"));
    assert!(talked(&out).is_none());
    assert_eq!(last_message(&w), "The sage knows nothing about that.");
    let out = w.apply(&Action::Ask {
        dir: Dir::S,
        topic: "road".into(),
    });
    assert!(talked(&out).is_none());
    assert_eq!(last_message(&w), "You see no one there to talk to.");
    assert_eq!((w.time(), w.turn()), (time, turn));
}

#[test]
fn an_npc_says_its_own_lines_and_one_liners_have_no_topics() {
    let mut w = world("parlor");
    run(&mut w, "move:S");
    let out = w.apply(&Action::Chat(Dir::S));
    assert_eq!(last_message(&w), "The sage says: \"I am the elder.\"");
    assert!(talked(&out).expect("an answer").topics.is_empty());
    assert_eq!(w.state().actors[&ActorId(3)].npc.as_deref(), Some("elder"));
    assert!(w.topics(ActorId(3)).is_empty());
    assert_eq!(w.topics(ActorId(1)).len(), 2);
}

#[test]
fn creatures_without_words_or_goodwill_give_a_non_answer() {
    // No one there: refused, no time.
    let mut w = world("parlor");
    let out = w.apply(&Action::Chat(Dir::N));
    assert!(!out.took_turn);
    assert_eq!(last_message(&w), "You see no one there to talk to.");
    // A goat has nothing to say; trying takes the chat's time.
    run(&mut w, "move:SE move:E");
    assert_eq!(w.player_pos(), Pos::new(3, 2));
    let out = w.apply(&Action::Chat(Dir::NE));
    assert!(out.took_turn && talked(&out).is_none());
    assert!(texts(&w).contains(&"The goat doesn't answer.".to_string()));
    // A brigand has words, but not for the player.
    let mut w = world("den");
    let out = w.apply(&Action::Chat(Dir::E));
    assert!(out.took_turn && talked(&out).is_none());
    assert!(texts(&w).contains(&"The brigand is in no mood to talk.".to_string()));
}

#[test]
fn no_one_unseen_is_talked_to() {
    let mut w = world("dark_yard");
    assert!(!w.perception.sees_actors_at(Pos::new(4, 2)));
    let out = w.apply(&Action::Chat(Dir::E));
    assert!(!out.took_turn);
    assert_eq!(last_message(&w), "You see no one there to talk to.");
}

/// The crone's mutter messages after `turns` waits.
fn mutters(scenario: &str, turns: usize) -> (World, Vec<String>) {
    let mut w = world(scenario);
    run(&mut w, &"wait ".repeat(turns));
    let said = texts(&w)
        .into_iter()
        .filter(|t| t.contains("hums") || t.contains("voice"))
        .collect();
    (w, said)
}

#[test]
fn mutters_are_heard_with_their_words_and_seen_with_the_speaker() {
    // In the dark, two cells away: a voice to the east, words and all; a sound on the map.
    let (w, said) = mutters("near", 1);
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].starts_with("You hear a voice to the "), "{said:?}");
    assert!(said[0].ends_with(": \"La.\"") || said[0].ends_with(": \"Lo.\""));
    assert!(
        w.perception()
            .sounds()
            .iter()
            .any(|s| s.kind == SoundKind::Voice)
    );
    // In the light, the crone is seen humming.
    let (w, said) = mutters("hall", 4);
    assert_eq!(said.len(), 4, "{said:?}");
    assert!(said.iter().all(|s| s.starts_with("The crone hums: \"L")));
    assert!(
        said.windows(2).all(|p| p[0] != p[1]),
        "two lines take turns"
    );
    assert!(
        w.perception().sounds().is_empty(),
        "a seen voice is no sound"
    );
    assert_eq!(w.state().actors[&ActorId(1)].talk.muttered, 4);
    // Four cells off it is faint: no words.
    let (_, said) = mutters("faint", 1);
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(
        said[0].starts_with("You faintly hear a voice to the "),
        "{said:?}"
    );
    assert!(said[0].ends_with('.') && !said[0].contains('"'));
    // Six cells off, beyond earshot, nobody mutters.
    let (w, said) = mutters("far", 5);
    assert!(said.is_empty(), "{said:?}");
    assert_eq!(w.state().actors[&ActorId(1)].talk.muttered, 0);
}

#[test]
fn mutters_never_stop_a_rest_nor_draw_on_the_gameplay_stream() {
    let mut w = world("hall");
    let rng = w.state().rng.clone();
    w.apply(&Action::Rest(Rest::For(Duration::minutes(10))));
    assert_eq!(last_message(&w), "You rest for 10 minutes.");
    assert!(texts(&w).iter().any(|t| t.starts_with("The crone hums")));
    assert_eq!(w.state().rng, rng);
    // Off its routine (on its way back to a post), it keeps quiet.
    let mut w = world("hall");
    if let Some(crone) = w.state.actors.get_mut(&ActorId(1)) {
        crone.mind.post = Some(Pos::new(6, 4));
    }
    run(&mut w, "wait");
    assert!(!texts(&w).iter().any(|t| t.contains("hums")));
}

#[test]
fn someone_who_talks_coming_into_view_does_not_stop_a_rest() {
    let mut w = world("cloister");
    w.apply(&Action::Rest(Rest::For(Duration::hours(1))));
    assert_eq!(last_message(&w), "You rest for 1 hour.");
    // A goat on the same round does.
    let mut w = world("pen");
    w.apply(&Action::Rest(Rest::For(Duration::hours(1))));
    assert!(
        last_message(&w).starts_with("A goat comes into view; you stop resting"),
        "{}",
        last_message(&w)
    );
}

#[test]
fn moving_into_someone_peaceful_swaps_places_for_a_step() {
    let mut w = world("parlor");
    let out = w.apply(&Action::Move(Dir::E));
    assert!(out.took_turn);
    assert_eq!(out.elapsed, Duration::seconds(6), "the player's step");
    assert_eq!(
        (w.player_pos(), pos(&w, 1)),
        (Pos::new(2, 1), Pos::new(1, 1))
    );
    assert!(texts(&w).contains(&"You swap places with the sage.".to_string()));
    let moved: Vec<ActorId> = (out.events.iter())
        .filter_map(|e| match e {
            Event::ActorMoved { actor, .. } => Some(*actor),
            _ => None,
        })
        .collect();
    assert_eq!(moved, [ActorId(0), ActorId(1)]);
    // The sage's own turn is unaffected; it walks back to where it stood once the player is gone.
    assert_eq!(
        w.state().actors[&ActorId(1)].mind.post,
        Some(Pos::new(2, 1))
    );
    run(&mut w, "move:E wait");
    assert_eq!(pos(&w, 1), Pos::new(2, 1));
    run(&mut w, "wait");
    assert!(w.state().actors[&ActorId(1)].mind.is_calm());
}

#[test]
fn some_creatures_do_not_make_way_and_refusing_takes_no_time() {
    let mut w = world("yard");
    let me = w.player_pos();
    let refused = |w: &mut World, dir: Dir, text: &str| {
        let out = w.apply(&Action::Move(dir));
        assert!(!out.took_turn && out.events.is_empty(), "{dir:?}");
        assert_eq!(last_message(w), text);
        assert_eq!(w.player_pos(), me);
    };
    refused(&mut w, Dir::W, "The sentry doesn't budge.");
    // Carrying a boulder, the goat cannot move.
    refused(&mut w, Dir::E, "The goat doesn't budge.");
    // The warden keeps its post while it goes about its routine.
    refused(&mut w, Dir::SW, "The warden doesn't budge.");
    // Off its post, it makes way.
    if let Some(warden) = w.state.actors.get_mut(&ActorId(3)) {
        warden.mind.post = Some(Pos::new(2, 2));
    }
    assert!(w.apply(&Action::Move(Dir::SW)).took_turn);
    assert!(texts(&w).contains(&"You swap places with the warden.".to_string()));
    assert_eq!(w.player_pos(), Pos::new(2, 3));
    // One size larger: squeezed past; two sizes: too big.
    let mut w = world("yard");
    assert!(w.apply(&Action::Move(Dir::SE)).took_turn);
    assert_eq!(last_message(&w), "You swap places with the ogre.");
    let mut w = world("yard");
    run(&mut w, "move:N move:E*2 move:SE");
    assert_eq!(w.player_pos(), Pos::new(6, 2));
    let at = w.player_pos();
    let out = w.apply(&Action::Move(Dir::S));
    assert!(!out.took_turn);
    assert_eq!(last_message(&w), "The giant is too big to squeeze past.");
    assert_eq!(w.player_pos(), at);
}

#[test]
fn unseen_creatures_make_way_or_are_in_the_way() {
    let mut w = world("dark_yard");
    let out = w.apply(&Action::Move(Dir::W));
    assert!(!out.took_turn);
    assert_eq!(last_message(&w), "Something is in the way.");
    assert!(w.apply(&Action::Move(Dir::SE)).took_turn);
    assert!(texts(&w).contains(&"You swap places with something.".to_string()));
}

#[test]
fn a_patrol_carries_on_after_a_swap() {
    // The goat walks E4 W4 along the bottom row from (6, 5).
    let mut undisturbed = world("yard");
    let mut w = world("yard");
    // Beside its round, the player waits for it, swaps places with it, and walks off.
    run(&mut w, "move:S move:SE move:E*2");
    assert_eq!(w.player_pos(), Pos::new(6, 4));
    let goat = 6;
    for _ in 0..16 {
        if pos(&w, goat).chebyshev(w.player_pos()) == 1 {
            break;
        }
        w.apply(&Action::Wait);
    }
    let (me, it) = (w.player_pos(), pos(&w, goat));
    let swap_dir = Dir::from_delta(it.x - me.x, it.y - me.y).expect("next to the player");
    w.apply(&Action::Move(swap_dir));
    assert!(
        texts(&w)
            .iter()
            .any(|t| t == "You swap places with the goat.")
    );
    assert_eq!(pos(&w, goat), me);
    run(&mut w, "move:N move:W*3 wait*10");
    // Back on its round: it walks the same cells, the same way, as one never disturbed.
    let round = |w: &mut World| -> Vec<Pos> {
        (0..16)
            .map(|_| {
                w.apply(&Action::Wait);
                pos(w, goat)
            })
            .collect()
    };
    let walked = round(&mut w);
    run(&mut undisturbed, "wait*20");
    let expected = round(&mut undisturbed);
    let cells = |p: &[Pos]| {
        let mut c = p.to_vec();
        c.sort();
        c.dedup();
        c
    };
    assert_eq!(cells(&walked), cells(&expected));
    assert!(walked.iter().all(|p| p.y == 5), "{walked:?}");
    assert!(w.state().actors[&ActorId(goat)].mind.is_calm());
}

#[test]
fn talk_is_saved_and_replayed() {
    let mut w = world("parlor");
    let mut replay = Replay::new(&w);
    let actions = parse_actions("chat:E chat:E ask:E:weather move:E chat:W wait*3").expect("text");
    for action in &actions {
        w.apply(action);
        replay.push(action, &w);
    }
    let talk = w.state().actors[&ActorId(1)].talk;
    assert!(talk.met && talk.said > 0);
    let played = replay.run(content()).expect("replay verifies");
    assert_eq!(played.state(), w.state());
    let mut loaded = crate::save::decode(&crate::save::encode(&w), content()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    run(&mut w, "chat:W chat:W");
    run(&mut loaded, "chat:W chat:W");
    assert_eq!(loaded.state_hash(), w.state_hash());
    assert_eq!(texts(&loaded), texts(&w));
}

#[test]
fn nothing_said_leaves_the_state_as_it_was() {
    // A creature that has not talked serializes without talk.
    let w = world("parlor");
    let json = serde_json::to_string(&w.state().actors[&ActorId(1)]).expect("json");
    assert!(!json.contains("talk") && !json.contains("npc"), "{json}");
    let json = serde_json::to_string(&w.state().actors[&ActorId(3)]).expect("json");
    assert!(json.contains("\"npc\":\"elder\""), "{json}");
}

#[test]
fn dialogue_data_is_validated() {
    let load = |dialogue: &str| {
        let mut files = files();
        files.insert("data/dialogue.toml", dialogue);
        Content::load(&files).map(|_| ()).map_err(|e| e.message)
    };
    let cases = [
        (
            "[[dialogue]]\nlines = [\"x\"]\n",
            "exactly one of `creature` and `npc`",
        ),
        (
            "[[dialogue]]\ncreature = \"sage\"\nnpc = \"x\"\nlines = [\"x\"]\n",
            "exactly one of",
        ),
        (
            "[[dialogue]]\ncreature = \"dragon\"\nlines = [\"x\"]\n",
            "unknown creature",
        ),
        (
            "[[dialogue]]\ncreature = \"player\"\nlines = [\"x\"]\n",
            "the player",
        ),
        (
            "[[dialogue]]\nnpc = \"Old Tam\"\nlines = [\"x\"]\n",
            "lower-case",
        ),
        (
            "[[dialogue]]\ncreature = \"sage\"\nlines = []\n",
            "at least one line",
        ),
        (
            "[[dialogue]]\ncreature = \"sage\"\nlines = [\"\"]\n",
            "not empty",
        ),
        (
            "[[dialogue]]\ncreature = \"sage\"\nlines = [\"x\"]\n\
             [[dialogue]]\ncreature = \"sage\"\nlines = [\"y\"]\n",
            "already one",
        ),
        (
            "[[dialogue]]\ncreature = \"sage\"\nlines = [\"x\"]\n\
             [[dialogue.topic]]\nid = \"A b\"\nname = \"n\"\nreply = \"r\"\n",
            "lower-case",
        ),
        (
            "[[dialogue]]\ncreature = \"sage\"\nlines = [\"x\"]\n\
             [[dialogue.topic]]\nid = \"a\"\nname = \"n\"\nreply = \"r\"\n\
             [[dialogue.topic]]\nid = \"a\"\nname = \"m\"\nreply = \"s\"\n",
            "duplicate topic",
        ),
        (
            "[[dialogue]]\ncreature = \"sage\"\nlines = [\"x\"]\n\
             mutter = { chance = 0, lines = [\"m\"] }\n",
            "percent",
        ),
        (
            "[[dialogue]]\ncreature = \"sage\"\nlines = [\"x\"]\n\
             mutter = { chance = 5, verb = \"Hums loudly\", lines = [\"m\"] }\n",
            "one lower-case word",
        ),
    ];
    for (text, message) in cases {
        let err = load(text).expect_err(text);
        assert!(err.contains(message), "{text}: {err}");
    }
    let long = "x".repeat(crate::content::MAX_LINE + 1);
    let err = load(&format!(
        "[[dialogue]]\ncreature = \"sage\"\nlines = [\"{long}\"]\n"
    ))
    .expect_err("too long");
    assert!(err.contains("at most 72"), "{err}");
    // A level's npc needs a dialogue of its own.
    let mut files = files();
    files.insert(
        "levels/parlor.toml",
        &PARLOR.replace("npc = \"elder\"", "npc = \"hermit\""),
    );
    let err = Content::load(&files).expect_err("unknown npc").message;
    assert!(err.contains("npc `hermit` has no dialogue"), "{err}");
}

#[test]
fn the_showcase_people_talk() {
    let content = Arc::new(Content::builtin().expect("builtin"));
    for creature in ["gardener", "bard", "hermit"] {
        let id = content.creature_id(creature).expect(creature);
        let dialogue = content.dialogue_of(id, None).expect("a dialogue");
        let d = content.dialogue(dialogue);
        assert!(d.lines.len() >= 6, "{creature}");
        assert!(d.mutter.is_some(), "{creature} mutters");
        let faction = content.creature(id).faction.expect("a faction");
        assert!(
            !content.factions().has_enemies(faction),
            "{creature} fights no one"
        );
    }
    // The hermit by the hub's stairs, with his hints.
    let mut w = World::new(content, "showcase", None).expect("showcase");
    run(
        &mut w,
        "move:N*3 open:N move:N*2 move:E*67 open:S move:S*4 move:SE*3 chat:SE ask:SE:stairs",
    );
    let said = texts(&w);
    assert!(
        said.iter()
            .any(|t| t.starts_with("The old hermit says: \"Hm? A visitor.")),
        "{said:?}"
    );
    assert!(
        said.last()
            .is_some_and(|t| t.contains("Two stairs down, two up."))
    );
}
