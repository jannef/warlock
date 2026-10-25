//! Hidden locations (spec §7 "Hidden locations", #327): the legend's `hidden`, what the player
//! sees before (nothing: the plain terrain, in every view and dump) and after a reveal, the
//! reveal by a quest at each of its stages and by the dev action, a reveal before the overworld
//! is made, saves, replays, and the content checks. The fixture is the built-in content with a
//! hidden barrow in the Wildlands' Mirefen leading into the Depths, revealed by the hermit's
//! second quest.

use super::*;
use crate::action::parse_actions;
use crate::content::{ContentFiles, RevealWhen};
use crate::dump::{self, Layer};
use crate::replay::Replay;

const OVERWORLD: &str = include_str!("../../../../assets/campaigns/showcase/levels/overworld.toml");
const QUESTS: &str =
    include_str!("../../../../assets/campaigns/showcase/data/quests/showcase.toml");

/// The hidden barrow's cell: a swamp cell of the Mirefen.
const BARROW: Pos = Pos::new(50, 43);
/// A swamp cell north of it, from where the player sees it by day.
const NEARBY: Pos = Pos::new(50, 41);
const LEGEND: &str = r#""2" = { terrain = "swamp", hidden = { id = "old_barrow", terrain = "dungeon_entrance", to = "depths:1", name = "The Old Barrow" } }"#;
const WHERE: &str = "The old barrow lies in the Mirefen, south of the Sallow Plains.";

/// The Wildlands with the hidden barrow at [`BARROW`] (and `legend` in its legend).
fn overworld_with(legend: &str) -> String {
    let (head, rest) = OVERWORLD.split_once("map = '''\n").expect("a map");
    let (map, tail) = rest.split_once("'''").expect("the map ends");
    let mut rows: Vec<Vec<char>> = map.lines().map(|l| l.chars().collect()).collect();
    let (x, y) = (BARROW.x as usize, BARROW.y as usize);
    assert_eq!(rows[y][x], 's', "the barrow lies in the swamp");
    rows[y][x] = '2';
    let map: Vec<String> = rows.into_iter().map(String::from_iter).collect();
    let tail = tail.replacen("[legend]\n", &format!("[legend]\n{legend}\n"), 1);
    format!("{head}map = '''\n{}\n'''{tail}", map.join("\n"))
}

/// The built-in content with the barrow hidden in the Wildlands and the hermit's `depths`
/// quest revealing it `when` (none: it reveals nothing).
fn files(when: Option<RevealWhen>) -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert(
        "campaigns/showcase/levels/overworld.toml",
        &overworld_with(LEGEND),
    );
    let when = match when {
        None => None,
        Some(RevealWhen::Offered) => Some("offered"),
        Some(RevealWhen::Accepted) => Some("accepted"),
        Some(RevealWhen::Done) => Some("done"),
    };
    if let Some(when) = when {
        let reveals = format!(
            "target = \"depths:3\"\nreveals = {{ location = \"old_barrow\", when = \"{when}\" }}\n\
             where = \"{WHERE}\""
        );
        files.insert(
            "campaigns/showcase/data/quests/showcase.toml",
            &QUESTS.replacen("target = \"depths:3\"", &reveals, 1),
        );
    }
    files
}

fn content(when: Option<RevealWhen>) -> Arc<Content> {
    Arc::new(Content::load(&files(when)).expect("content"))
}

/// A run of `scenario` on the fixture content.
fn run_of(scenario: &str, when: Option<RevealWhen>) -> World {
    World::new(content(when), scenario, None).expect("run")
}

/// A run on the overworld, standing in sight of the barrow.
fn beside_the_barrow() -> World {
    let mut w = run_of("world", Some(RevealWhen::Accepted));
    put_player(&mut w, NEARBY);
    assert!(
        w.perception().is_visible(BARROW),
        "the barrow's cell is in sight"
    );
    w
}

fn put_player(world: &mut World, at: Pos) {
    let id = world.state.player;
    world.state.actors.get_mut(&id).expect("player").pos = at;
    world.perceive(&mut Vec::new());
}

fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.text.clone()).collect()
}

fn last(world: &World) -> String {
    said(world).pop().unwrap_or_default()
}

fn glyph(world: &World, layer: Layer, at: Pos) -> char {
    let rows = dump::layer_rows(world, layer);
    rows[at.y as usize]
        .chars()
        .nth(at.x as usize)
        .unwrap_or(' ')
}

fn act(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

#[test]
fn the_legend_hides_a_location_on_the_overworld() {
    let content = content(None);
    let overworld = content.overworld_of("showcase").expect("an overworld");
    let swamp = content.terrain_id("swamp").expect("swamp");
    assert_eq!(overworld.terrain[BARROW], swamp);
    assert!(overworld.stairs.iter().all(|s| s.pos != BARROW));
    let (at, hidden) = content
        .hidden_location("showcase", "old_barrow")
        .expect("hidden");
    assert_eq!(at, BARROW);
    assert_eq!(hidden.to.as_deref(), Some("depths:1"));
    assert_eq!(hidden.name, "The Old Barrow");
    assert_eq!(content.terrain(hidden.terrain).id, "dungeon_entrance");
}

#[test]
fn a_hidden_location_never_reaches_perception_or_the_dumps() {
    let mut w = beside_the_barrow();
    let swamp = w
        .content()
        .terrain(w.content().terrain_id("swamp").expect("swamp"))
        .glyph;
    for layer in [Layer::View, Layer::Memory] {
        assert_eq!(glyph(&w, layer, BARROW), swamp, "{}", layer.name());
    }
    assert_eq!(w.perception().leads_to(BARROW), None);
    let known = dump::layer_rows(&w, Layer::Locations).join("\n");
    let json = dump::json(&w, &[Layer::View, Layer::Memory, Layer::Locations], false).to_string();
    for text in [known.as_str(), json.as_str()] {
        assert!(!text.contains("old_barrow"), "{text}");
        assert!(!text.contains("Old Barrow"), "{text}");
    }
    // Ground truth knows it: the dev `truth` layer draws its entrance.
    assert_eq!(glyph(&w, Layer::Truth, BARROW), '>');
    // Stepping onto it is a step into the swamp, and `>` explores the swamp's local area.
    put_player(&mut w, BARROW.step(Dir::N));
    w.apply(&Action::Move(Dir::S));
    assert_eq!(w.player_pos(), BARROW);
    assert!(
        !said(&w).iter().any(|m| m.contains("way into")),
        "{:?}",
        said(&w)
    );
    w.apply(&Action::Descend);
    assert_eq!(last(&w), "You explore the swamp.");
    assert_eq!(w.state.current_level, "local:50,43");
}

#[test]
fn the_dev_reveal_marks_it_and_it_leads_on() {
    let mut w = run_of("world", Some(RevealWhen::Accepted));
    let before = w.state_hash();
    // Far away: the overworld's memory gets the entrance though it is not in sight.
    assert!(!w.perception().is_visible(BARROW));
    w.apply(&Action::Reveal("old_barrow".into()));
    assert_eq!(last(&w), "The Old Barrow is marked on your map (dev).");
    assert_ne!(w.state_hash(), before);
    assert_eq!(glyph(&w, Layer::Memory, BARROW), '>');
    assert_eq!(glyph(&w, Layer::Truth, BARROW), '>');
    assert_eq!(
        w.perception().leads_to(BARROW).as_deref(),
        Some("The Old Barrow")
    );
    let known = dump::layer_rows(&w, Layer::Locations);
    assert!(
        known.contains(&"(50, 43) The Old Barrow".to_string()),
        "{known:?}"
    );
    let json = dump::json(&w, &[], false);
    let names: Vec<&str> = (json["known_locations"]
        .as_array()
        .expect("locations")
        .iter())
    .filter_map(|l| l["name"].as_str())
    .collect();
    assert!(names.contains(&"The Old Barrow"), "{names:?}");
    // A second reveal changes nothing (but what the player is told); an unknown id is refused.
    let quiet = |w: &World| {
        let mut state = w.state().clone();
        state.messages = Default::default();
        state
    };
    let before = quiet(&w);
    w.apply(&Action::Reveal("old_barrow".into()));
    assert_eq!(last(&w), "The Old Barrow is no secret any more.");
    w.apply(&Action::Reveal("nowhere".into()));
    assert_eq!(last(&w), "There is no hidden location `nowhere`.");
    assert_eq!(quiet(&w), before);
    // Walking there, it is the way in, and `>` enters the Depths.
    put_player(&mut w, BARROW.step(Dir::N));
    w.apply(&Action::Move(Dir::S));
    assert!(said(&w).contains(&"The way into The Old Barrow is here.".to_string()));
    w.apply(&Action::Descend);
    assert_eq!(w.state.current_level, "depths:1");
    assert!(
        said(&w).contains(&"You enter The Old Barrow.".to_string()),
        "{:?}",
        said(&w)
    );
}

#[test]
fn a_reveal_before_the_overworld_is_made_waits_for_it() {
    // The showcase starts in the hall: the overworld is not made yet.
    let mut w = run_of("showcase", Some(RevealWhen::Accepted));
    assert!(!w.state.levels.contains_key("overworld"));
    w.apply(&Action::Reveal("old_barrow".into()));
    assert!(w.state.revealed.contains("old_barrow"));
    w.travel("overworld", None).expect("overworld");
    let level = w.state.current();
    let entrance = w
        .content()
        .terrain_id("dungeon_entrance")
        .expect("entrance");
    assert_eq!(level.terrain[BARROW], entrance);
    assert_eq!(level.memory[BARROW], Some(entrance));
    assert!(level.hidden.is_empty());
    let link = level.stairs_at(BARROW).expect("a way in");
    assert_eq!(link.to.as_deref(), Some("depths:1"));
}

/// The hermit's quests, the rats done, with the player beside the hermit: the way to him.
fn at_the_hermit(when: RevealWhen) -> (World, Dir) {
    let mut w = run_of("showcase", Some(when));
    w.state.quests =
        serde_json::from_str(r#"{"records": {"rats": {"stage": "done", "completed": 1}}}"#)
            .expect("quests");
    let hermit = w.content().creature_id("hermit").expect("hermit");
    let at = (w.state.actors.values())
        .find(|a| a.creature == hermit)
        .expect("the hermit")
        .pos;
    let level = w.state.current_level.clone();
    let player = w.state.player;
    let d = (Dir::ALL.into_iter())
        .find(|&d| {
            let p = at.step(d);
            w.state.current().is_walkable(&w.content, p)
                && (w.state.actor_at(&level, p)).is_none_or(|a| a.id == player)
        })
        .expect("room beside him");
    put_player(&mut w, at.step(d));
    (w, d.opposite())
}

fn ask(world: &mut World, dir: Dir, topic: &str) {
    world.apply(&Action::Ask {
        dir,
        topic: topic.into(),
    });
}

const MARKS: &str = "The old hermit marks the Old Barrow on your map.";

#[test]
fn a_quest_reveals_its_location_at_its_stage() {
    for when in [RevealWhen::Offered, RevealWhen::Accepted, RevealWhen::Done] {
        let (mut w, toward) = at_the_hermit(when);
        let revealed = |w: &World| w.state.revealed.contains("old_barrow");
        ask(&mut w, toward, "quest-depths");
        assert_eq!(
            revealed(&w),
            when == RevealWhen::Offered,
            "{when:?} offered"
        );
        ask(&mut w, toward, "accept-depths");
        assert_eq!(revealed(&w), when != RevealWhen::Done, "{when:?} accepted");
        w.travel("depths:3", None).expect("depths:3");
        w.travel("hub", None).expect("hub");
        let (mut w, toward) = {
            let hermit = w.content().creature_id("hermit").expect("hermit");
            let at = (w.state.actors.values())
                .find(|a| a.creature == hermit)
                .expect("the hermit")
                .pos;
            put_player(&mut w, at.step(toward.opposite()));
            (w, toward)
        };
        ask(&mut w, toward, "quest-depths");
        assert!(revealed(&w), "{when:?} done");
        // Marked once, after the hermit's words.
        let all = said(&w);
        let marked: Vec<usize> = (all.iter().enumerate())
            .filter(|(_, m)| *m == MARKS)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(marked.len(), 1, "{when:?}: {all:?}");
        assert!(
            all[marked[0] - 1].starts_with("The old hermit says:"),
            "{all:?}"
        );
    }
}

#[test]
fn the_quest_log_says_where() {
    let (mut w, toward) = at_the_hermit(RevealWhen::Accepted);
    ask(&mut w, toward, "quest-depths");
    ask(&mut w, toward, "accept-depths");
    let log = w.perception().quests();
    assert_eq!(log[0].objective, format!("Reach the Depths 3. {WHERE}"));
    assert_eq!(
        said(&w)[said(&w).len() - 2..],
        [
            "The old hermit says: \"The old way in opens in the Barrowdowns, out past the \
             courtyard. Take your lantern.\""
                .to_string(),
            MARKS.to_string(),
        ]
    );
}

#[test]
fn reveals_survive_a_save_and_replay_exactly() {
    let mut w = run_of("world", Some(RevealWhen::Accepted));
    let mut replay = Replay::new(&w);
    for action in parse_actions("wait reveal:old_barrow wait").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let content = w.content().clone();
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    // A save naming a location this content does not have is refused.
    let mut state = w.state().clone();
    state.revealed.insert("nowhere".into());
    assert!(matches!(
        World::from_state(content, state),
        Err(WorldError::InconsistentState(_))
    ));
}

#[test]
fn a_run_without_reveals_keeps_its_state_as_before() {
    // The hidden location lies in the overworld's state (it is ground truth), but nothing about
    // reveals is saved until one happens.
    let mut w = run_of("world", None);
    act(&mut w, "wait move:N");
    let json = serde_json::to_string(w.state()).expect("json");
    assert!(!json.contains("\"revealed\""));
    // The built-in Wildlands hide nothing.
    let builtin = World::new(
        Arc::new(Content::builtin().expect("content")),
        "world",
        None,
    )
    .expect("run");
    let json = serde_json::to_string(builtin.state()).expect("json");
    assert!(!json.contains("\"hidden\""));
}

/// The content error loading `files` gives.
fn error(files: &ContentFiles) -> String {
    Content::load(files).expect_err("refused").to_string()
}

#[test]
fn content_checks_hidden_locations_and_reveals() {
    let with = |legend: &str| {
        let mut files = ContentFiles::builtin();
        files.insert(
            "campaigns/showcase/levels/overworld.toml",
            &overworld_with(legend),
        );
        files
    };
    let hidden = |terrain: &str, at: &str, to: &str| {
        format!(
            r#""2" = {{ terrain = "{terrain}", hidden = {{ id = "barrow", terrain = "{at}", to = "{to}", name = "The Barrow" }} }}"#
        )
    };
    for (legend, says) in [
        (
            hidden("mountains", "dungeon_entrance", "depths:1"),
            "only on walkable",
        ),
        (hidden("swamp", "swamp", "depths:1"), "is no way down"),
        (
            hidden("swamp", "nothing", "depths:1"),
            "unknown terrain `nothing`",
        ),
        (
            hidden("swamp", "dungeon_entrance", "nowhere"),
            "unknown level `nowhere`",
        ),
        (
            hidden("swamp", "dungeon_entrance", "depths:1").replace("barrow\"", "Barrow\""),
            "ids are lower-case",
        ),
    ] {
        let message = error(&with(&legend));
        assert!(message.contains(says), "{legend}: {message}");
    }
    // Only on the overworld.
    let mut files = ContentFiles::builtin();
    let arena = include_str!("../../../../assets/campaigns/showcase/levels/arena.toml").replacen(
        "[legend]\n",
        "[legend]\n\"9\" = { terrain = \"floor\", hidden = { id = \"pit\", terrain = \
         \"stairs_down\", to = \"hub\", name = \"The Pit\" } }\n",
        1,
    );
    let arena = arena.replacen("map = '''\n#", "map = '''\n9", 1);
    files.insert("campaigns/showcase/levels/arena.toml", &arena);
    assert!(
        error(&files).contains("only the overworld"),
        "{}",
        error(&files)
    );
    // A quest reveals a location the overworld hides.
    let mut files = files_with_quest_reveal("nowhere");
    assert!(
        error(&files).contains("no hidden location `nowhere`"),
        "{}",
        error(&files)
    );
    files = files_with_quest_reveal("old_barrow");
    assert!(Content::load(&files).is_ok());
}

/// The fixture with the hermit's `depths` quest revealing `location`.
fn files_with_quest_reveal(location: &str) -> ContentFiles {
    let mut files = files(None);
    let reveals = format!("target = \"depths:3\"\nreveals = {{ location = \"{location}\" }}");
    files.insert(
        "campaigns/showcase/data/quests/showcase.toml",
        &QUESTS.replacen("target = \"depths:3\"", &reveals, 1),
    );
    files
}
