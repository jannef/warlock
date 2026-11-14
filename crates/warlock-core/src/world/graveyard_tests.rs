//! Statues named from the graveyard (`graveyard.rs`): one name a statue, the most recent first,
//! never twice on a floor, the same every time, and out of the state hash.

use super::*;
use crate::content::ContentFiles;

/// The builtin content with a rooms branch peopled by basilisks among kobolds, every draw dread
/// (so a basilisk is drawn, and its statues laid), and a scenario starting on its floor.
fn content() -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    let branch = "\n[[dungeon]]\nid = \"lair\"\nname = \"Lair\"\ngenerator = \"rooms\"\ndepth = 1\n\
         size = [80, 40]\ndanger = 16\npopulation = [\"basilisk\", \"kobold\"]\n\
         dread_one_in = 1\nspawn = { every = \"1m\", chance = 100 }\n";
    let dungeons = format!(
        "{}{branch}",
        include_str!("../../../../assets/data/dungeons.toml")
    );
    files.insert("data/dungeons.toml", &dungeons);
    files.insert(
        "scenarios/lair.toml",
        "name = \"Lair\"\nstart_level = \"lair:1\"\nhp = 1000\n",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
}

/// One turned to stone: only these name statues.
fn fallen(name: &str, race: &str) -> Fallen {
    Fallen {
        name: name.into(),
        race: race.into(),
        petrified: true,
        death: "Turned to stone".into(),
        ..Fallen::default()
    }
}

fn names(w: &World) -> Vec<Option<String>> {
    let (level, _) = w.ground_truth();
    (level.statues().values())
        .map(|s| s.name.as_ref().map(|f| f.words(w.content())))
        .collect()
}

/// A run on the lair's floor with at least `n` statues, and its seed.
fn with_statues(content: &Arc<Content>, n: usize) -> (u64, World) {
    (1..=40)
        .find_map(|seed| {
            let w = World::new(content.clone(), "lair", Some(seed)).expect("world");
            let enough = w.ground_truth().0.statues().len() >= n;
            enough.then_some((seed, w))
        })
        .expect("a floor with statues")
}

#[test]
fn a_floor_s_statues_take_the_petrified_names_most_recent_first_never_twice() {
    let content = content();
    let (seed, mut w) = with_statues(&content, 2);
    let count = w.ground_truth().0.statues().len();
    let hash = w.state_hash();
    // Brakka twice: one statue of her only. Wynn is the oldest, left over on a small floor.
    // Osk, the most recent, was not turned to stone: no statue of his (a gravestone's, #1258).
    let graveyard = vec![
        Fallen {
            petrified: false,
            death: "Killed by a kobold".into(),
            ..fallen("Osk", "halfling")
        },
        fallen("Brakka", "dwarf"),
        fallen("Brakka", "dwarf"),
        fallen("Tamsin", "elf"),
        fallen("Wynn", "no_such_race"),
    ];
    w.set_graveyard(graveyard.clone());
    let shown = names(&w);
    let named: Vec<&String> = shown.iter().flatten().collect();
    assert_eq!(named.len(), count.min(3), "{shown:?}");
    assert!(
        named.contains(&&"Brakka the dwarf".to_string()),
        "{shown:?}"
    );
    assert!(named.contains(&&"Tamsin the elf".to_string()), "{shown:?}");
    assert!(!named.iter().any(|n| n.starts_with("Osk")), "{shown:?}");
    if count >= 3 {
        assert!(named.contains(&&"Wynn the no_such_race".to_string()));
    }
    // Decoration: out of the state hash.
    assert_eq!(w.state_hash(), hash, "the names are not hashed");
    // The same every time, and the same for the level made with the list as for the list given
    // after it was made.
    let mut again = World::new(content.clone(), "lair", Some(seed)).expect("world");
    again.set_graveyard(graveyard);
    assert_eq!(names(&again), shown);
}

#[test]
fn a_level_made_later_is_named_and_a_save_keeps_the_names() {
    let content = content();
    let (seed, w) = with_statues(&content, 1);
    let shown_unnamed = names(&w);
    assert!(
        shown_unnamed.iter().all(Option::is_none),
        "no graveyard, no names"
    );
    // Given before the floor is made: the floor is made with it (here made again, the player
    // alone kept).
    let mut w = World::new(content.clone(), "lair", Some(seed)).expect("world");
    let current = w.state.current_level.clone();
    w.state.levels.remove(&current);
    let player = w.state.player;
    w.state.actors.retain(|&id, _| id == player);
    w.set_graveyard(vec![fallen("Brakka", "dwarf")]);
    let template = travel::level_template(
        &content,
        ("showcase", seed),
        &current,
        (None, &Default::default()),
    )
    .expect("the floor")
    .into_owned();
    travel::instantiate_level(&mut w.state, &content, &template);
    assert_eq!(names(&w).iter().flatten().count(), 1);
    // A save keeps the list and the names.
    let loaded = crate::save::decode(&crate::save::encode(&w), content.clone()).expect("loads");
    assert_eq!(loaded.graveyard(), w.graveyard());
    assert_eq!(names(&loaded), names(&w));
}

#[test]
fn the_player_sees_whom_a_named_statue_is_of() {
    let content = content();
    let (_, mut w) = with_statues(&content, 1);
    w.set_graveyard(vec![fallen("Brakka", "dwarf")]);
    let level = w.state.current();
    let (&at, _) = (level.statues.iter())
        .find(|(_, s)| s.name.is_some())
        .expect("a named statue");
    // The player beside it, on open ground (a statue has open ground all about it).
    let player = w.state.player;
    w.state.actors.get_mut(&player).expect("player").pos = at.step(crate::geom::Dir::E);
    let w = World::from_state(content.clone(), (*w.state).clone()).expect("restored");
    let view = w.perception();
    assert!(view.statue_at(at).is_some(), "seen");
    assert_eq!(view.statue_name(at).as_deref(), Some("Brakka the dwarf"));
}
