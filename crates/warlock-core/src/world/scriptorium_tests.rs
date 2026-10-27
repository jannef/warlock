//! The scriptorium (spec §7 "Special rooms", design §9.4): Decipher script's training marks come
//! twice as often inside, studying a scroll there earns one for certain, other skills train as
//! anywhere, and look mode and the messages name the room and its shelves.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::skills::{SkillId, Trained, Use};

/// A lit hall, 25 wide: the west part plain, the east part (x 12 on) a scriptorium with its
/// shelves at `S`.
const HALL: &str = "\
#########################
#.......................#
#.......................#
#...@.............S.....#
#.......................#
#.......................#
#########################
";

const INSIDE: Pos = Pos::new(17, 3);
const OUTSIDE: Pos = Pos::new(4, 3);

/// The built-in content with [`HALL`]; the player carries two scrolls of frost (c) after a scroll
/// of fireball (a) and one of lightning (b), and has `mana` base mana.
fn world(mana: u16) -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/hall.toml",
        &format!(
            "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{HALL}'''\n\
             [[room]]\neffect = \"scriptorium\"\narea = [12, 0, 24, 6]\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"S\" = {{ terrain = \"shelves\" }}\n"
        ),
    );
    files.insert(
        "scenarios/hall.toml",
        &format!(
            "name = \"Hall\"\nstart_level = \"hall\"\nhp = 500\nmana = {mana}\n\
             carry = [\"scroll_of_fireball\", \"scroll_of_lightning\", \"scroll_of_frost*2\"]\n"
        ),
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "hall", None).expect("world")
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn skill(w: &World, id: &str) -> SkillId {
    w.content().skill_id(id).expect("a skill")
}

/// The player trained in Decipher script and Stealth, with no marks yet.
fn trained(w: &mut World) {
    let (decipher, stealth) = (skill(w, "decipher_script"), skill(w, "stealth"));
    let player = w.state.player;
    let skills = &mut w.state.actors.get_mut(&player).expect("player").skills;
    let mut all: Vec<(SkillId, Trained)> = (skills.iter())
        .filter(|(s, _)| *s != decipher && *s != stealth)
        .collect();
    for id in [decipher, stealth] {
        all.push((
            id,
            Trained {
                value: 10,
                marks: 0,
            },
        ));
    }
    *skills = all.into_iter().collect();
}

fn marks(w: &World, id: &str) -> u32 {
    (w.state.player().skills.get(skill(w, id))).map_or(0, |t| t.marks)
}

fn place(w: &mut World, at: Pos) {
    w.player_mut().pos = at;
}

#[test]
fn decipher_script_marks_come_twice_as_often_inside() {
    let mut w = world(0);
    trained(&mut w);
    let decipher = skill(&w, "decipher_script");
    assert_eq!(w.effects_at("hall", INSIDE).train(decipher), 200);
    assert_eq!(w.effects_at("hall", OUTSIDE).train(decipher), 100);
    // Its rule is 50 % a study: inside, doubled, every one marks.
    place(&mut w, INSIDE);
    for _ in 0..40 {
        w.train(Use::Study);
    }
    assert_eq!(marks(&w, "decipher_script"), 40);
    // Outside, about half.
    place(&mut w, OUTSIDE);
    for _ in 0..400 {
        w.train(Use::Study);
    }
    let outside = marks(&w, "decipher_script") - 40;
    assert!((160..=240).contains(&outside), "{outside}");
}

#[test]
fn other_skills_train_inside_as_anywhere() {
    let mut a = world(0);
    let mut b = world(0);
    for (w, at) in [(&mut a, INSIDE), (&mut b, OUTSIDE)] {
        trained(w);
        place(w, at);
        for _ in 0..200 {
            w.train(Use::Sneak);
        }
    }
    let stealth = marks(&a, "stealth");
    assert!(stealth > 0);
    assert_eq!(
        stealth,
        marks(&b, "stealth"),
        "the same draws, the same marks"
    );
}

#[test]
fn a_study_inside_earns_a_mark_for_certain() {
    let mut w = world(10);
    run(&mut w, "identify");
    while w.player_level() < 6 {
        run(&mut w, "xp:2000 levelup:auto");
    }
    trained(&mut w);
    place(&mut w, INSIDE);
    run(&mut w, "study:c");
    assert!(!w.state.player().spells.is_empty(), "frost learned");
    assert_eq!(marks(&w, "decipher_script"), 1);
}

#[test]
fn the_room_and_its_shelves_are_told() {
    let mut w = world(0);
    // Seen from outside first: the hint, not the room's name.
    let said = |w: &World, text: &str| w.messages().iter().any(|m| m.text == text);
    run(&mut w, "wait");
    assert!(said(&w, "You smell dust and old vellum."));
    assert!(!said(&w, "Shelves of rotting scrolls line the walls."));
    run(&mut w, "move:E*10");
    assert!(said(&w, "Shelves of rotting scrolls line the walls."));
    let view = w.perception();
    let shelves = Pos::new(18, 3);
    let crate::perception::CellView::Visible(terrain) = view.cell(shelves) else {
        panic!("the shelves are in view");
    };
    assert_eq!(w.content().terrain(terrain).name, "shelves");
    assert_eq!(
        view.room(shelves.step(crate::geom::Dir::W))
            .map(|r| r.name.as_str()),
        Some("scriptorium")
    );
}
