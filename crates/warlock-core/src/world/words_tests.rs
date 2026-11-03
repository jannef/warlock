//! Saying words (spec §8 "Dialogue", "Saying words"; `words.rs`): a match in any case and
//! punctuation, unheard words, the time either way, `silent` outside `speaks_to` (and no Say row
//! nor topics), `departs`, `quest`, the refusals, the load checks, no draws, a replay, and a save.

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
"##;

const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "sage"
name = "sage"
glyph = "@"
fg = "#c0c0ff"
faction = "folk"
weight = 60000

[[creature]]
id = "spirit"
name = "spirit"
glyph = "S"
fg = "#c0c0ff"
faction = "folk"
weight = 1000
depart = "fades, and is gone"

[[creature]]
id = "ghost"
name = "ghost"
glyph = "G"
fg = "#c0c0c0"
faction = "folk"
weight = 1000

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

const DIALOGUE: &str = r##"
[[dialogue]]
creature = "sage"
lines = ["Hm."]
unheard = "The sage shakes his head."

[[dialogue.topic]]
id = "road"
name = "The road"
reply = "Long."

[[dialogue.word]]
phrase = "Open, sesame!"
reply = ["Ah, you know it.", "Then you may know this, too."]
does = { quest = "cellar" }

[[dialogue.word]]
phrase = "Hello"
reply = ["Hello to you."]

[[dialogue]]
creature = "spirit"
lines = ["It waits."]
speaks_to = { max = "neutral" }
silent = "The spirit looks through you."

[[dialogue.word]]
phrase = "Life before death"
reply = ["Life before death.", "Journey before destination."]
does = { departs = true }

[[dialogue]]
creature = "ghost"
lines = ["Boo."]
speaks_to = { min = "kind" }
silent = "The ghost looks through you as if you were not there."

[[dialogue.word]]
phrase = "Rest"
reply = ["At last."]

[[dialogue]]
creature = "brigand"
lines = ["Your money or your life."]

[[dialogue.word]]
phrase = "Parley"
reply = ["Fine."]
"##;

const QUESTS: &str = r##"
[[quest]]
id = "cellar"
name = "The cellar"
giver = "sage"
type = "reach"
target = "cellar"
reward = { xp = 10 }
offer = "Go down to the cellar."
reminder = "The cellar."
done = "So it's still there."
"##;

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
"##;

// A lit hall: the sage east of the player, the spirit south-east, the ghost south, a goat
// south-west; a brigand behind a wall.
const HALL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
#######
#.@s..#
#gGS..#
#.....#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"s" = { terrain = "floor", creature = "sage" }
"S" = { terrain = "floor", creature = "spirit" }
"G" = { terrain = "floor", creature = "ghost" }
"g" = { terrain = "floor", creature = "goat" }
"##;

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

const CELLAR: &str = "name = \"Cellar\"\nmap = '''\n###\n#.#\n###\n'''\n";

fn files() -> ContentFiles {
    let mut files = ContentFiles::default();
    files.insert(
        "data/item_classes.toml",
        crate::content::BUILTIN_ITEM_CLASSES,
    );
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/creatures.toml", CREATURES),
        ("data/factions.toml", FACTIONS),
        ("data/dialogue.toml", DIALOGUE),
        ("data/quests.toml", QUESTS),
        ("data/legend.toml", LEGEND),
        (
            "data/experience.toml",
            include_str!("../../../../assets/data/experience.toml"),
        ),
        (
            "data/vision.toml",
            "[[vision]]\nid = \"normal\"\nname = \"Normal\"\nthreshold = 40\n",
        ),
        ("data/actions.toml", "chat = 500\n"),
        ("levels/hall.toml", HALL),
        ("levels/den.toml", DEN),
        ("levels/cellar.toml", CELLAR),
    ] {
        files.insert(path, text);
    }
    for id in ["hall", "den"] {
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

fn say(dir: Dir, words: &str) -> Action {
    Action::Say {
        dir,
        words: words.into(),
    }
}

fn texts(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.text.clone()).collect()
}

fn last_message(world: &World) -> String {
    texts(world).last().cloned().unwrap_or_default()
}

fn talked(out: &Outcome) -> Option<&Conversation> {
    out.events.iter().rev().find_map(|e| match e {
        Event::Talked(talk) => Some(talk),
        _ => None,
    })
}

#[test]
fn words_are_heard_in_any_case_and_punctuation_and_take_a_chats_time() {
    for said in ["hello", "HELLO!", "  hello. ", "Hello?"] {
        let mut w = world("hall");
        let start = w.time();
        let out = w.apply(&say(Dir::E, said));
        assert!(out.took_turn, "{said}");
        assert_eq!(w.time().since(start), Duration::seconds(3), "a chat's time");
        assert_eq!(last_message(&w), "The sage says: \"Hello to you.\"");
        let talk = talked(&out).expect("the screen reopens on the reply");
        assert_eq!(talk.line, "Hello to you.");
        assert_eq!(talk.asked, None);
        assert!(talk.words, "the sage knows words: the Say row");
        assert_eq!(
            talk.topics.len(),
            2,
            "the road, and the quest the word gives"
        );
    }
}

#[test]
fn words_not_known_are_unheard_and_cost_the_same_every_time() {
    let mut w = world("hall");
    let start = w.time();
    for _ in 0..5 {
        let out = w.apply(&say(Dir::E, "abracadabra"));
        assert!(out.took_turn);
        assert_eq!(
            last_message(&w),
            "The sage says: \"The sage shakes his head.\""
        );
        assert!(talked(&out).is_some_and(|t| t.words));
    }
    assert_eq!(w.time().since(start), Duration::seconds(15));
    // The right words after any number of wrong ones: no lock-out.
    w.apply(&say(Dir::E, "hello"));
    assert_eq!(last_message(&w), "The sage says: \"Hello to you.\"");
    // A speaker with no `unheard` says nothing, in words.
    let mut w = world("hall");
    let out = w.apply(&say(Dir::SE, "goodbye"));
    assert_eq!(last_message(&w), "The spirit says nothing.");
    assert_eq!(talked(&out).map(|t| t.line.as_str()), Some("…"));
}

#[test]
fn a_word_with_a_quest_gives_it_once() {
    let mut w = world("hall");
    let out = w.apply(&say(Dir::E, "open sesame"));
    let said = texts(&w);
    assert_eq!(
        said[said.len() - 3..],
        [
            "The sage says: \"Ah, you know it.\"",
            "The sage says: \"Then you may know this, too.\"",
            "The sage says: \"Go down to the cellar.\"",
        ]
    );
    let talk = talked(&out).expect("the offer");
    assert_eq!(talk.asked.as_deref(), Some("quest-cellar"));
    let cellar = w.content().quest_id("cellar").expect("quest");
    assert!(w.state.quests.is_active(w.content().quest(cellar)));
    // Said again, the reply, and nothing given again.
    w.apply(&say(Dir::E, "Open sesame"));
    assert_eq!(
        last_message(&w),
        "The sage says: \"Then you may know this, too.\""
    );
}

#[test]
fn a_word_that_departs_sends_the_speaker_away_after_its_reply() {
    let mut w = world("hall");
    let spirit = w.state().actor_at(&w.state().current_level, Pos::new(3, 2));
    let spirit = spirit.expect("the spirit").id;
    let out = w.apply(&say(Dir::SE, "Life, before death!"));
    assert!(out.took_turn);
    let said = texts(&w);
    assert_eq!(
        said[said.len() - 3..],
        [
            "The spirit says: \"Life before death.\"",
            "The spirit says: \"Journey before destination.\"",
            "The spirit fades, and is gone.",
        ]
    );
    assert!(!w.state().actors.contains_key(&spirit));
    assert!(talked(&out).is_none(), "no one is left to talk to");
    assert!(
        (out.events.iter()).any(|e| matches!(e, Event::Departed { actor, .. } if *actor == spirit))
    );
    assert!(
        w.state().current().objects_at(Pos::new(3, 2)).is_empty(),
        "nothing left behind"
    );
    assert_eq!(w.player_xp(), 0, "no experience");
}

#[test]
fn outside_speaks_to_the_speaker_is_silent_with_no_topics_and_no_row() {
    let mut w = world("hall");
    let ghost = w.state().actor_at(&w.state().current_level, Pos::new(2, 2));
    let ghost = ghost.expect("the ghost").id;
    for action in [Action::Chat(Dir::S), say(Dir::S, "rest")] {
        let out = w.apply(&action);
        assert!(out.took_turn, "{action}");
        assert!(talked(&out).is_none(), "{action}");
        assert_eq!(
            last_message(&w),
            "The ghost looks through you as if you were not there."
        );
    }
    assert!(w.topics(ghost).is_empty());
    assert!(w.state().actors.contains_key(&ghost));
    // Inside its bands (the spirit speaks to the Neutral), it talks: the row is there.
    let out = w.apply(&Action::Chat(Dir::SE));
    assert!(talked(&out).is_some_and(|t| t.words && t.topics.is_empty()));
}

#[test]
fn refusals_take_no_time_or_a_chats_as_chatting_does() {
    let mut w = world("hall");
    let out = w.apply(&say(Dir::N, "hello"));
    assert!(!out.took_turn);
    assert_eq!(last_message(&w), "You see no one there to talk to.");
    let out = w.apply(&say(Dir::E, "?!"));
    assert!(!out.took_turn, "nothing said");
    // A goat has no words; a brigand has, but not for the player.
    w.apply(&Action::Move(Dir::W));
    let out = w.apply(&say(Dir::S, "hello"));
    assert!(out.took_turn && talked(&out).is_none());
    assert_eq!(last_message(&w), "The goat doesn't answer.");
    let mut w = world("den");
    let out = w.apply(&say(Dir::E, "parley"));
    assert!(out.took_turn && talked(&out).is_none());
    assert!(texts(&w).contains(&"The brigand is in no mood to talk.".to_string()));
}

#[test]
fn saying_words_draws_nothing_and_is_replayed_and_saved() {
    let mut w = world("hall");
    let mut replay = Replay::new(&w);
    let actions =
        parse_actions("say:E:abracadabra say:E:Open_Sesame say:SE:life_before_death wait")
            .expect("text");
    let rng = w.state().rng.clone();
    for action in &actions {
        w.apply(action);
        replay.push(action, &w);
    }
    assert_eq!(w.state().rng, rng, "no draws");
    let played = replay.run(content()).expect("replay verifies");
    assert_eq!(played.state(), w.state());
    let loaded = crate::save::decode(&crate::save::encode(&w), content()).expect("loads");
    assert_eq!(loaded.state(), w.state());
}

#[test]
fn words_are_validated() {
    let load = |extra: &str| {
        let mut files = files();
        files.insert("data/dialogue.toml", &format!("{DIALOGUE}\n{extra}"));
        Content::load(&files).map(|_| ()).map_err(|e| e.message)
    };
    let goat = "[[dialogue]]\ncreature = \"goat\"\nlines = [\"Meh.\"]\n";
    let word = |phrase: &str, rest: &str| {
        format!("{goat}[[dialogue.word]]\nphrase = \"{phrase}\"\nreply = [\"Yes.\"]\n{rest}")
    };
    let long = "x".repeat(crate::words::MAX_SAID + 1);
    let many: String = (0..=crate::content::MAX_WORDS)
        .map(|i| format!("[[dialogue.word]]\nphrase = \"w{i}\"\nreply = [\"Yes.\"]\n"))
        .collect();
    let cases = [
        (word("?!", ""), "a letter or digit"),
        (word(&long, ""), "1 to 40 characters"),
        (
            format!(
                "{}{}",
                word("Hello", ""),
                "[[dialogue.word]]\nphrase = \"HELLO!\"\nreply = [\"No.\"]\n"
            ),
            "says what `Hello` says",
        ),
        (format!("{goat}{many}"), "at most 9 words"),
        (
            format!("{goat}[[dialogue.word]]\nphrase = \"x\"\nreply = []\n"),
            "1 to 9 lines",
        ),
        (
            word("x", "does = { calls = \"hunter\" }\n"),
            "no calling `hunter` in its campaign",
        ),
        (
            word("x", "does = { quest = \"well\" }\n"),
            "no quest `well`",
        ),
        (
            "[[dialogue]]\nnpc = \"elder\"\nlines = [\"x\"]\n[[dialogue.word]]\nphrase = \"x\"\n\
             reply = [\"y\"]\ndoes = { departs = true }\n"
                .to_string(),
            "only for a creature type's",
        ),
        (
            format!("{goat}speaks_to = {{ min = \"kind\" }}\n"),
            "needs a `silent` line",
        ),
        (
            format!("{goat}silent = \"Nothing.\"\n"),
            "only outside a `speaks_to`",
        ),
        (
            format!("{goat}speaks_to = {{ min = \"good\", max = \"evil\" }}\nsilent = \"N.\"\n"),
            "`speaks_to`",
        ),
        (format!("{goat}unheard = \"\"\n"), "`unheard`"),
    ];
    for (text, message) in cases {
        let err = load(&text).expect_err(&text);
        assert!(err.contains(message), "{text}: {err}");
    }
}
