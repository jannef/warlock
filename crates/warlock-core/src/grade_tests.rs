//! Grades (spec §8 "Grades"): the draw, the four rules reads, the types with
//! none, the save and the hash, and the dumps.

use super::*;
use crate::content::CreatureId;
use crate::world::{ActorKind, World};
use std::sync::{Arc, LazyLock};

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

fn spawn(content: &Content, creature: &str) -> CreatureSpawn {
    CreatureSpawn {
        creature: content.creature_id(creature).expect(creature),
        faction: None,
        alignment: None,
        npc: None,
        pos: crate::geom::Pos::new(1, 1),
        route: Vec::new(),
        carry: Vec::new(),
        equipment: Default::default(),
        rolls_gear: false,
        asleep: false,
        status: Vec::new(),
    }
}

/// The arena, its creatures made as the run begins.
fn arena() -> World {
    World::new(CONTENT.clone(), "arena", Some(7)).expect("world")
}

#[test]
fn one_in_six_is_greater_and_one_in_six_lesser() {
    let kobold = spawn(&CONTENT, "kobold");
    let grades: Vec<i8> = (0..6000)
        .map(|i| draw(&CONTENT, 11, ActorId(i), &kobold))
        .collect();
    let count = |g| grades.iter().filter(|&&x| x == g).count();
    for g in [-1, 1] {
        assert!(
            (850..=1150).contains(&count(g)),
            "{g}: {} of 6000",
            count(g)
        );
    }
    assert_eq!(count(-1) + count(0) + count(1), 6000);
    // The same run and actor draw the same; another run may not.
    let again: Vec<i8> = (0..6000)
        .map(|i| draw(&CONTENT, 11, ActorId(i), &kobold))
        .collect();
    assert_eq!(grades, again);
    let other: Vec<i8> = (0..6000)
        .map(|i| draw(&CONTENT, 12, ActorId(i), &kobold))
        .collect();
    assert_ne!(grades, other);
}

#[test]
fn a_unique_character_and_a_type_without_grades_are_never_graded() {
    let mut named = spawn(&CONTENT, "kobold");
    named.npc = Some("someone".into());
    let villager = spawn(&CONTENT, "villager");
    for i in 0..600 {
        assert_eq!(draw(&CONTENT, 3, ActorId(i), &named), 0);
        assert_eq!(draw(&CONTENT, 3, ActorId(i), &villager), 0);
    }
}

#[test]
fn uniques_people_mimics_and_generated_types_have_no_grades() {
    let factions = CONTENT.factions();
    let people = ["peaceful", "merchants"].map(|f| factions.id(f).expect(f));
    let mut ungraded = Vec::new();
    for (i, def) in CONTENT.creature_defs().iter().enumerate() {
        let id = CreatureId(u16::try_from(i).expect("few types"));
        let person = def.faction.is_some_and(|f| people.contains(&f));
        let none = def.unique
            || def.mimic.is_some()
            || def.animated.is_some()
            || CONTENT.animated_body(id).is_some()
            || person;
        if none {
            assert!(!def.grades, "{} has grades", def.id);
            ungraded.push(def.id.as_str());
        }
    }
    // Every kind the rule names is among them.
    for id in ["villager", "shopkeeper", "fire_drake"] {
        assert!(ungraded.contains(&id), "{id}");
    }
    assert!(
        CONTENT
            .creature_defs()
            .iter()
            .any(|d| d.mimic.is_some() && !d.grades)
    );
    assert!(
        CONTENT
            .creature_defs()
            .iter()
            .any(|d| d.animated.is_some() && !d.grades)
    );
    // The rest have them: a kobold, a rat, a zombie.
    for id in ["kobold", "rat", "zombie"] {
        let def = CONTENT.creature(CONTENT.creature_id(id).expect(id));
        assert!(def.grades, "{id}");
    }
}

#[test]
fn the_words_are_the_types_else_its_tags_else_small_and_big() {
    let words = |id: &str| {
        let def = CONTENT.creature(CONTENT.creature_id(id).expect(id));
        (word(def, -1), word(def, 0), word(def, 1))
    };
    assert_eq!(words("kobold"), (Some("scrawny"), None, Some("big")));
    assert_eq!(words("zombie"), (Some("withered"), None, Some("ancient")));
    assert_eq!(words("rat"), (Some("small"), None, Some("big")));
}

#[test]
fn a_grade_changes_hit_points_the_blow_and_speed() {
    let w = arena();
    let kobold = CONTENT.creature_id("kobold").expect("kobold");
    let plain = (w.state().actors.values())
        .find(|a| a.creature == kobold)
        .expect("a kobold")
        .clone();
    let graded = |grade| {
        let mut a = plain.clone();
        a.grade = grade;
        a
    };
    let (base, greater, lesser) = (graded(0), graded(1), graded(-1));
    let hp = base.max_hp(&CONTENT);
    assert_eq!(greater.max_hp(&CONTENT), hp * 125 / 100);
    assert_eq!(lesser.max_hp(&CONTENT), hp * 75 / 100);
    assert_eq!(greater.lasting_max_hp(&CONTENT), hp * 125 / 100);
    let blow = base.melee(&CONTENT);
    assert_eq!(greater.melee(&CONTENT).to_hit, blow.to_hit + 1);
    assert_eq!(greater.melee(&CONTENT).bonus, blow.bonus + 2);
    assert_eq!(lesser.melee(&CONTENT).to_hit, blow.to_hit - 1);
    assert_eq!(lesser.melee(&CONTENT).bonus, blow.bonus - 2);
    let speed = base.speed(&CONTENT);
    assert_eq!(greater.speed(&CONTENT), speed * 90 / 100);
    assert_eq!(lesser.speed(&CONTENT), speed * 110 / 100);
}

#[test]
fn the_grade_is_saved_hashed_and_left_out_at_nought() {
    let w = arena();
    let state = w.state().clone();
    let (&id, plain) = (state.actors.iter())
        .find(|(_, a)| a.kind == ActorKind::Creature && a.grade == 0)
        .expect("an ungraded creature");
    let text = serde_json::to_string(plain).expect("json");
    assert!(!text.contains("\"grade\""), "{text}");
    let mut greater = plain.clone();
    greater.grade = 1;
    let text = serde_json::to_string(&greater).expect("json");
    assert!(text.contains("\"grade\":1"), "{text}");
    let back: crate::world::Actor = serde_json::from_str(&text).expect("back");
    assert_eq!(back.grade, 1);
    // The hash tells the two apart.
    let mut changed = state.clone();
    changed.actors.get_mut(&id).expect("actor").grade = 1;
    let changed = World::from_state(CONTENT.clone(), changed).expect("world");
    let again = World::from_state(CONTENT.clone(), state).expect("world");
    assert_eq!(again.state_hash(), w.state_hash());
    assert_ne!(changed.state_hash(), w.state_hash());
}

#[test]
fn the_actors_dump_and_the_json_say_the_grade() {
    let w = arena();
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Actors);
    for a in w.state().actors.values() {
        let def = CONTENT.creature(a.creature);
        let row = (rows.iter())
            .find(|r| r.starts_with(&format!("#{} ", a.id.0)))
            .expect("a row");
        match word(def, a.grade) {
            Some(word) => assert!(row.contains(&format!("; grade {word};")), "{row}"),
            None => assert!(!row.contains("; grade "), "{row}"),
        }
    }
    assert!(
        rows.iter().any(|r| r.contains("; grade ")),
        "some are graded"
    );
    // The JSON names a seen one's grade in its word.
    let json = crate::dump::json(&w, &[], false);
    let view = w.perception();
    for seen in view.actors() {
        let def = CONTENT.creature(seen.creature);
        let entry = (json["seen_actors"].as_array().expect("seen").iter())
            .find(|v| v["id"] == serde_json::json!(seen.id))
            .expect("seen");
        assert_eq!(entry["grade"].as_str(), word(def, seen.grade), "{entry}");
    }
}

#[test]
fn a_kept_beast_reads_as_ungraded() {
    let w = arena();
    let mut a = (w.state().actors.values())
        .find(|a| a.kind == ActorKind::Creature)
        .expect("a creature")
        .clone();
    a.grade = 1;
    assert_eq!(a.graded(), 1);
    a.ally = Some(crate::world::Ally {
        master: w.state().player,
        source: crate::world::AllySource::Companion(crate::world::CompanionData {
            name: "Snel".into(),
            grade: 1,
            scent: None,
            quarry: None,
        }),
        order: crate::world::Order::Follow,
        retinue: false,
        until: None,
    });
    assert_eq!(a.graded(), 0);
}
