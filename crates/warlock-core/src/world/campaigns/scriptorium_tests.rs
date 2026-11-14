//! The scriptorium (spec §7 "Special rooms", design §9.4): Decipher script's training marks come
//! at twice the chance inside, at most certain (a check to read a scroll is certain anywhere,
//! #1024), other skills train as anywhere, and look mode and the messages name the room and its shelves.

use super::*;
use crate::content::ContentFiles;
use crate::skills::{SkillId, Trained, Use};
use crate::test_support::run;

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
fn the_room_gives_two_marks_per_check_and_one_outside() {
    let mut w = world(0);
    trained(&mut w);
    let decipher = skill(&w, "decipher_script");
    assert_eq!(w.effects_at("hall", INSIDE).train(decipher), 200);
    assert_eq!(w.effects_at("hall", OUTSIDE).train(decipher), 100);
    // Its rule is 100 % a check (#1024): 200 % is two marks for certain, 100 % one.
    for (at, want) in [(INSIDE, 80), (OUTSIDE, 40)] {
        place(&mut w, at);
        for _ in 0..40 {
            w.train(Use::Decipher);
        }
        assert_eq!(marks(&w, "decipher_script"), want);
        trained(&mut w);
    }
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
fn a_check_inside_marks_decipher_script_twice_and_a_study_too() {
    let mut w = world(10);
    run(&mut w, "identify");
    trained(&mut w);
    place(&mut w, INSIDE);
    run(&mut w, "read:c:20,3");
    assert_eq!(
        marks(&w, "decipher_script"),
        2,
        "a read by the check: two marks"
    );
    // Decipher script 40 more: learning 6, Frost's level.
    let decipher = skill(&w, "decipher_script");
    w.player_mut().skills.add(decipher, 40);
    let grimoire = w.content.object_id("grimoire_of_frost").expect("object");
    w.player_mut()
        .inventory
        .push(crate::object::Stack::new(grimoire, 1));
    let at = w.state.player().inventory.len() - 1;
    run(
        &mut w,
        &format!("read:{}", crate::action::position_text(at)),
    );
    assert!(!w.state.player().spells.is_empty(), "frost learned");
    // A study marks Decipher script too (#1270), twice inside.
    assert_eq!(
        marks(&w, "decipher_script"),
        4,
        "a study inside: two marks more"
    );
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
