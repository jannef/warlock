//! Unidentified scrolls (spec §8 "Scrolls"): labels per run, identified by reading or by the
//! scroll of identify, known from a starting kit, saved and hashed; the scroll of identify in
//! loot and in the general store.

use super::*;
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::ContentFiles;
use crate::magic::LootGrade;
use crate::replay::Replay;

/// A lit hall, the player carrying: a, three scrolls of fireball; b, two of lightning; c, two of
/// identify; d, a scroll of light.
fn world(seed: u64) -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/hall.toml",
        "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n##########\n#@.......#\n\
         ##########\n'''\n[legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
    );
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 1000\ncarry = [\"scroll_of_fireball*3\", \
         \"scroll_of_lightning*2\", \"scroll_of_identify*2\", \"scroll_of_light\"]\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "hall", Some(seed)).expect("world")
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn texts(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

fn id(w: &World, object: &str) -> ObjectId {
    w.content.object_id(object).expect("object")
}

/// What the player sees their pack as, in order.
fn names(w: &World) -> Vec<String> {
    let view = w.perception();
    (view.inventory().iter())
        .map(|s| view.lore().describe(s))
        .collect()
}

fn label(w: &World, object: &str) -> String {
    w.labels.get(id(w, object)).expect("a label").to_string()
}

#[test]
fn every_readable_scroll_has_its_own_label_drawn_from_the_seed() {
    let (w, again, other) = (world(1), world(1), world(2));
    let readable: Vec<ObjectId> = (0..w.content.object_defs().len())
        .map(|o| ObjectId(o as u16))
        .filter(|&o| w.content.object(o).read.is_some())
        .collect();
    assert!(readable.len() >= 4);
    let labels: BTreeSet<&str> = readable.iter().filter_map(|&o| w.labels.get(o)).collect();
    assert_eq!(
        labels.len(),
        readable.len(),
        "one label a type, all different"
    );
    for l in &labels {
        assert!((2..=12).contains(&l.len()), "{l}");
        assert_eq!(l.to_uppercase(), *l);
    }
    // The same seed draws the same labels, after a reload too; another seed others.
    assert_eq!(w.labels, again.labels);
    assert_ne!(w.labels, other.labels);
    let loaded = crate::save::decode(&crate::save::encode(&w), w.content.clone()).expect("loads");
    assert_eq!(loaded.labels, w.labels);
    // A scroll that cannot be read has none: the scroll of light is known on sight.
    assert_eq!(w.labels.get(id(&w, "scroll_of_light")), None);
}

#[test]
fn unknown_scrolls_show_by_their_labels_and_colour() {
    let w = world(1);
    let fire = label(&w, "scroll_of_fireball");
    let ident = label(&w, "scroll_of_identify");
    assert_eq!(
        names(&w),
        [
            format!("3 scrolls labelled {fire}"),
            format!("2 scrolls labelled {}", label(&w, "scroll_of_lightning")),
            format!("2 scrolls labelled {ident}"),
            "a scroll of light".to_string(),
        ]
    );
    let view = w.perception();
    let pack = view.inventory();
    let plain = w.content.scroll_labels().expect("labels").fg;
    assert_eq!(view.lore().fg(&pack[0]), plain);
    assert_eq!(view.lore().fg(&pack[2]), plain);
    assert_eq!(
        view.lore().title(&pack[0]),
        format!("scroll labelled {fire}")
    );
    assert!(!view.lore().is_identified(pack[0].object));
    assert!(view.lore().is_identified(pack[3].object), "no label");
    // Player-facing dumps name them so; the dev dump of actors tells the truth.
    let dump = crate::dump::layer_rows(&w, crate::dump::Layer::Inventory).join("\n");
    assert!(
        dump.contains(&format!("a - 3 scrolls labelled {fire}")),
        "{dump}"
    );
    assert!(!dump.contains("fireball"), "{dump}");
}

#[test]
fn reading_a_scroll_identifies_its_type_for_good() {
    let mut w = world(1);
    let fire = label(&w, "scroll_of_fireball");
    run(&mut w, "read:a:8,1");
    let said = texts(&w);
    let at = (said.iter())
        .position(|t| *t == format!("You read the scroll labelled {fire}."))
        .expect("read by its label");
    assert_eq!(said[at + 1], "It is a scroll of fireball.");
    assert_eq!(names(&w)[0], "2 scrolls of fireball");
    assert!(
        w.state
            .known_scrolls
            .contains(&id(&w, "scroll_of_fireball"))
    );
    // The next is read by its name, with nothing more to learn.
    run(&mut w, "read:a:8,1");
    let said = texts(&w);
    assert!(said.contains(&"You read the scroll of fireball.".to_string()));
    assert_eq!(said.iter().filter(|t| t.starts_with("It is a")).count(), 1);
    // Every scroll of the type is known, wherever it lies: one dropped and picked up again too.
    run(&mut w, "drop:a pickup");
    assert!(names(&w).contains(&"a scroll of fireball".to_string()));
}

#[test]
fn the_scroll_of_identify_identifies_a_scroll_type_and_itself() {
    let mut w = world(1);
    let ident = label(&w, "scroll_of_identify");
    let before = w.state_hash();
    run(&mut w, "read:c:b");
    let said = texts(&w);
    assert!(said.contains(&format!("You read the scroll labelled {ident}.")));
    assert!(said.contains(&"It is a scroll of identify.".to_string()));
    assert_eq!(
        said.last().map(String::as_str),
        Some("This is 2 scrolls of lightning!")
    );
    assert_ne!(w.state_hash(), before, "the known set is hashed");
    assert_eq!(names(&w)[1], "2 scrolls of lightning");
    assert_eq!(names(&w)[2], "a scroll of identify");
    // Known now: identify has nothing more to learn about it, and the refusal takes no time.
    let turn = w.state.turn;
    run(&mut w, "read:c:b");
    assert_eq!(w.state.turn, turn);
    assert!(
        texts(&w)
            .last()
            .is_some_and(|t| t.starts_with("You already know"))
    );
    // The known set survives a save.
    let loaded = crate::save::decode(&crate::save::encode(&w), w.content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    assert_eq!(names(&loaded), names(&w));
}

#[test]
fn dev_identify_names_scroll_types_too() {
    let mut w = world(1);
    run(&mut w, "identify");
    assert_eq!(
        names(&w),
        [
            "3 scrolls of fireball",
            "2 scrolls of lightning",
            "2 scrolls of identify",
            "a scroll of light"
        ]
    );
}

#[test]
fn a_characters_kit_scrolls_are_known() {
    let spec: CharacterSpec = "human,male,wizard".parse().expect("a character");
    let content = Arc::new(Content::builtin().expect("content"));
    let w = World::start_run(content, "showcase", Some(7), Some(&spec), None, true).expect("w");
    let view = w.perception();
    let scrolls: Vec<&Stack> = (view.inventory().iter())
        .filter(|s| w.content.object(s.object).read.is_some())
        .collect();
    assert!(scrolls.len() >= 2, "a wizard starts with scrolls");
    for s in scrolls {
        assert!(view.lore().knows(s), "{}", view.lore().describe(s));
    }
}

#[test]
fn replays_that_read_unknown_scrolls_replay() {
    let mut w = world(3);
    let content = w.content.clone();
    let mut replay = Replay::new(&w);
    for action in parse_actions("read:c:a read:a:8,1 read:b:7,1").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let played = replay.run(content).expect("replays");
    assert_eq!(played.state_hash(), w.state_hash());
}

#[test]
fn the_scroll_of_identify_is_the_commonest_scroll_and_the_general_store_has_it() {
    let content = Content::builtin().expect("content");
    let identify = content.object_id("scroll_of_identify").expect("identify");
    let mut by_type: BTreeMap<ObjectId, u32> = BTreeMap::new();
    for i in 0..2000 {
        let mut rng = Rng::stream(i, "loot-test");
        let stack = crate::magic::loot::roll(&content, &mut rng, 5, LootGrade::Any).expect("loot");
        if content.object(stack.object).read.is_some() {
            *by_type.entry(stack.object).or_default() += 1;
        }
    }
    let most = by_type.iter().max_by_key(|(_, n)| **n).map(|(o, _)| *o);
    assert_eq!(most, Some(identify), "{by_type:?}");
    let shops = content.shops().expect("shops");
    let general = (shops.defs().iter())
        .find(|d| d.id == "general_store")
        .expect("a general store");
    assert_eq!(general.always, vec![identify]);
}
