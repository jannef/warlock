//! Character creation (spec §8 "Character creation"): a run starts as the chosen character, or
//! as the scenario's; its race gives the player's eyes; its rolls, birth date, and sign follow
//! from the seed, the choices, and the start date; the character is saved and hashed.

use super::*;
use crate::character::{CharacterSpec, Gender, other_stat};
use crate::content::Sign;
use crate::save;
use crate::stats::Stat;
use std::sync::LazyLock;

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));

fn spec(text: &str) -> CharacterSpec {
    text.parse().expect("a character")
}

fn start(character: &str, seed: u64) -> World {
    World::with_character(
        CONTENT.clone(),
        "showcase",
        Some(seed),
        Some(&spec(character)),
    )
    .expect("world")
}

fn scores(w: &World) -> Vec<u8> {
    Stat::ALL.map(|s| w.state.player().stats.score(s)).to_vec()
}

fn chosen(w: &World) -> Option<&CharacterSpec> {
    w.character().map(|c| &c.spec)
}

#[test]
fn a_run_starts_as_the_chosen_character() {
    let w = start("dwarf,female,warrior", 1);
    assert_eq!(chosen(&w), Some(&spec("dwarf,female,warrior")));
    // The player's stats are the character's, rolled for this seed and start date.
    let showcase = CONTENT.scenario("showcase").expect("showcase");
    let made = spec("dwarf,female,warrior")
        .start(&CONTENT, 1, showcase.start)
        .expect("starts");
    assert_eq!(w.state.player().stats, made.stats);
    assert_eq!(w.character(), Some(&made.character));
    // The race's eyes.
    assert_eq!(w.state.vision, "darkvision");
    let view = w.perception();
    assert_eq!(view.vision().id, "darkvision");
    assert_eq!(view.character(), w.character());
    assert_eq!(view.own_scores(), made.stats.scores());
    for (character, vision) in [
        ("elf,male,bard", "low_light"),
        ("half_elf,male,bard", "low_light"),
        ("human,male,bard", "normal"),
        ("halfling,male,bard", "normal"),
        ("gnome,male,bard", "low_light"),
        ("half_orc,male,bard", "darkvision"),
    ] {
        assert_eq!(start(character, 1).state.vision, vision, "{character}");
    }
}

#[test]
fn the_birth_counts_back_from_the_scenarios_start_date() {
    for seed in 0..20 {
        let w = start("elf,male,wizard", seed);
        let c = w.character().expect("a character");
        let today = w.time();
        let age = c.age_on(today);
        assert!((90..=300).contains(&age), "{age}");
        let not_yet = (c.born.month, c.born.day) > (today.month(), today.day());
        assert_eq!(
            c.born.year,
            today.year() as i64 - i64::from(age) - i64::from(not_yet)
        );
        // The sign is the birth year's, and the flags answer for it.
        let h = CONTENT.horoscope().expect("a horoscope");
        let sign = h.sign_of_year(c.born.year);
        assert_eq!(c.sign.as_ref(), Some(&sign));
        assert!(c.born_of_element(&sign.element) && c.born_in_year_of(&sign.animal));
        let others = h
            .elements
            .iter()
            .filter(|e| c.born_of_element(&e.id))
            .count();
        assert_eq!(others, 1, "born of one element only");
    }
    // A scenario that starts a year later makes the same character a year later.
    let mut files = crate::content::ContentFiles::builtin();
    files.insert(
        "scenarios/later.toml",
        "name = \"L\"\nstart_level = \"hub\"\nstart_date = [1445, 11, 11]\n\
         character = { race = \"human\", gender = \"male\", class = \"warrior\" }\n",
    );
    files.insert(
        "scenarios/sooner.toml",
        "name = \"S\"\nstart_level = \"hub\"\nstart_date = [1444, 11, 11]\n\
         character = { race = \"human\", gender = \"male\", class = \"warrior\" }\n",
    );
    let content = Arc::new(Content::load(&files).expect("valid"));
    let later = World::new(content.clone(), "later", Some(3)).expect("world");
    let sooner = World::new(content, "sooner", Some(3)).expect("world");
    let (a, b) = (
        later.character().expect("c"),
        sooner.character().expect("c"),
    );
    assert_eq!(a.born.year, b.born.year + 1);
    assert_eq!((a.born.month, a.born.day), (b.born.month, b.born.day));
    assert_eq!(a.age_on(later.time()), b.age_on(sooner.time()));
}

#[test]
fn without_a_choice_the_run_is_the_scenarios_character() {
    let w = World::new(CONTENT.clone(), "showcase", None).expect("world");
    assert_eq!(chosen(&w), Some(&spec("human,female,thief")));
    assert_eq!(w.state.vision, "normal");
    // Choosing the same character is the same run.
    let same = start("human,female,thief", 1);
    assert_eq!(same.state_hash(), w.state_hash());
    // Every built-in scenario has a character, with normal eyes.
    for def in CONTENT.scenarios() {
        let w = World::new(CONTENT.clone(), &def.id, None).expect("world");
        assert!(w.character().is_some(), "{}", def.id);
        assert_eq!(w.state.vision, "normal", "{}", def.id);
    }
}

#[test]
fn the_same_seed_and_choices_make_the_same_character() {
    for seed in [1, 2, 3, 99, 12345] {
        let a = start("human,other,commoner", seed);
        let b = start("human,other,commoner", seed);
        assert_eq!(a.state_hash(), b.state_hash());
        assert_eq!(scores(&a), scores(&b));
        let raised = other_stat(seed);
        assert_eq!(Gender::Other.raises(seed), raised);
        assert_eq!(a.character().map(|c| c.raised), Some(raised));
    }
    // Other seeds roll other characters.
    let rolled: std::collections::BTreeSet<Vec<u8>> = (0..20)
        .map(|seed| scores(&start("half_orc,male,barbarian", seed)))
        .collect();
    assert!(rolled.len() > 5, "{rolled:?}");
}

#[test]
fn the_character_is_saved_and_hashed() {
    let mut w = start("gnome,other,witch", 5);
    w.apply(&Action::Wait);
    let restored = save::decode(&save::encode(&w), CONTENT.clone()).expect("decode");
    assert_eq!(restored.character(), w.character());
    assert_eq!(restored.state_hash(), w.state_hash());
    // Another class with the same stats, or another birthday, would still be another state.
    let mut other = w.state.as_ref().clone();
    if let Some(c) = &mut other.character {
        c.spec = spec("gnome,other,wizard");
    }
    assert_ne!(other.hash(), w.state.hash());
    let mut older = w.state.as_ref().clone();
    if let Some(c) = &mut older.character {
        c.born.year -= 1;
    }
    assert_ne!(older.hash(), w.state.hash());
    // A save whose race or sign the content does not have is refused.
    let mut unknown = w.state.as_ref().clone();
    if let Some(c) = &mut unknown.character {
        c.spec = spec("orc,other,witch");
    }
    let err = World::from_state(CONTENT.clone(), unknown).unwrap_err();
    assert!(err.to_string().contains("unknown race `orc`"), "{err}");
    let mut dragon = w.state.as_ref().clone();
    if let Some(c) = &mut dragon.character {
        c.sign = Some(Sign {
            animal: "dragon".into(),
            element: "fire".into(),
        });
    }
    let err = World::from_state(CONTENT.clone(), dragon).unwrap_err();
    assert!(err.to_string().contains("unknown birth sign"), "{err}");
}

#[test]
fn unknown_races_and_classes_cannot_start_a_run() {
    for (character, why) in [
        ("orc,male,warrior", "unknown race `orc`"),
        ("elf,female,necromancer", "unknown class `necromancer`"),
    ] {
        let err = World::with_character(CONTENT.clone(), "showcase", None, Some(&spec(character)))
            .expect_err("refused");
        assert_eq!(err, WorldError::Character(why.into()));
    }
}

#[test]
fn the_dev_vision_switch_still_works() {
    let mut w = start("half_orc,male,barbarian", 1);
    assert_eq!(w.perception().vision().id, "darkvision");
    w.apply(&Action::SetVision("uv_sight".into()));
    assert_eq!(w.perception().vision().id, "uv_sight");
    assert_eq!(chosen(&w), Some(&spec("half_orc,male,barbarian")));
}
