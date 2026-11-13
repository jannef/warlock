//! Quick hands and Master's hand (#1425; the craftsperson design §7.8, §7.9): the craftsperson's
//! at levels 40 and 50, and their reads: every craft's time through `mastery::craft_seconds`
//! (a recipe's through `fletch::fletcher_seconds`, Fletcher's share multiplied), and a made
//! check's margin eased as it is graded (`mastery::graded_margin`), never a failure's.

use super::craft_make::make_out_for_test;
use super::*;
use crate::content::ContentFiles;
use crate::object::Stack;
use crate::skills::{SkillId, Trained};
use std::sync::LazyLock;

/// A forge with its anvil at (5, 2), the player beside it at (4, 3).
const ROOM: &str = "\
###########
#.........#
#....A....#
#...@.....#
#.........#
###########
";

/// The built-in content with [`ROOM`], a forge, and a dwarf craftsperson.
static FIXTURE: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_mastery.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{ROOM}'''\n\
             [[room]]\neffect = \"forge\"\narea = [0, 0, 10, 5]\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"A\" = \"anvil\"\n"
        ),
    );
    files.insert(
        "scenarios/test_mastery.toml",
        "name = \"T\"\nstart_level = \"test_mastery\"\n\
         character = { race = \"dwarf\", gender = \"female\", class = \"craftsperson\" }\n",
    );
    Arc::new(Content::load(&files).expect("fixture content is valid"))
});

/// The forge, the player carrying nothing, trained in Metalcraft and Woodcraft at 50, holding
/// each of `talents`.
fn world(talents: &[&str]) -> World {
    let mut w = World::new(FIXTURE.clone(), "test_mastery", None).expect("world");
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    me.inventory.clear();
    me.equipment = Default::default();
    for id in ["metalcraft", "woodcraft"] {
        let s: SkillId = w.content.skill_id(id).expect(id);
        let skills = &mut w.player_mut().skills;
        let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(k, _)| *k != s).collect();
        all.push((
            s,
            Trained {
                value: 50,
                marks: 0,
            },
        ));
        *skills = all.into_iter().collect();
    }
    for talent in talents {
        w.player_mut().talents.take(talent, None);
    }
    w
}

fn one(w: &World, id: &str) -> Stack {
    Stack::one(w.content.object_id(id).expect(id))
}

fn said(w: &World) -> String {
    (w.messages().iter().last())
        .map(|m| m.text.clone())
        .unwrap_or_default()
}

/// The seconds `action` takes from the same state (whatever comes of it).
fn took(w: &mut World, action: &Action) -> u64 {
    let start = w.state.time;
    w.apply(action);
    w.state.time.since(start).as_seconds()
}

#[test]
fn the_craftsperson_takes_them_at_forty_and_fifty() {
    let content = &*FIXTURE;
    let class = content.class("craftsperson").expect("the class");
    let book = content.talents();
    let id = |t: &str| book.id(t).expect(t);
    let to = |level| class.abilities_to(book, level);
    assert!(!to(39).contains(&id("quick_hands")));
    assert!(to(40).contains(&id("quick_hands")));
    assert!(!to(49).contains(&id("masters_hand")));
    assert!(to(50).contains(&id("masters_hand")));
}

#[test]
fn quick_hands_halves_every_craft() {
    let plain = world(&[]);
    let quick = world(&["quick_hands"]);
    let seconds = |w: &World, base| craft_seconds_for_test(w, base);
    assert_eq!((seconds(&plain, 600), seconds(&quick, 600)), (600, 300));
    assert_eq!(seconds(&quick, 1), 1, "at least a second");
    // A recipe's time, read in one place, and Fletcher's share multiplied with it.
    let content = plain.content.clone();
    let recipe = |id: &str| content.recipe(content.recipes().id(id).expect(id));
    let sword = recipe("long_sword");
    let fletch = |w: &World, r| super::fletch::fletcher_seconds(&content, w.state.player(), r);
    assert_eq!(fletch(&quick, sword), sword.seconds / 2);
    let arrows = recipe("arrows");
    let mut ranger = world(&["quick_hands", "fletcher"]);
    let alone = fletch(&world(&["fletcher"]), arrows);
    assert_eq!(fletch(&ranger, arrows), alone / 2, "the two multiply");
    ranger.player_mut().talents = Default::default();
    assert_eq!(fletch(&ranger, arrows), arrows.seconds);
}

#[test]
fn a_temper_and_a_salvage_take_half_the_time() {
    let temper = w_seconds(&[], "long_sword", |item| Action::Temper { item });
    let quick = w_seconds(&["quick_hands"], "long_sword", |item| Action::Temper {
        item,
    });
    assert_eq!((temper, quick), (600, 300));
    let salvage = w_seconds(&[], "quarterstaff", |item| Action::Salvage { item });
    let quick = w_seconds(&["quick_hands"], "quarterstaff", |item| Action::Salvage {
        item,
    });
    assert_eq!(quick * 2, salvage);
}

/// The seconds `action` on a fresh `object` (stack 0) takes a player holding `talents`.
fn w_seconds(talents: &[&str], object: &str, action: impl Fn(usize) -> Action) -> u64 {
    let mut w = world(talents);
    let stack = one(&w, object);
    w.player_mut().inventory.insert(0, stack);
    took(&mut w, &action(0))
}

fn craft_seconds_for_test(w: &World, base: u32) -> u32 {
    super::mastery::craft_seconds(&w.content, w.state.player(), base)
}

#[test]
fn masters_hand_eases_a_made_check_only() {
    let plain = world(&[]);
    let master = world(&["masters_hand"]);
    let eased = |w: &World, m| super::mastery::graded_margin(&w.content, w.state.player(), m);
    assert_eq!(eased(&plain, 2), 2);
    assert_eq!((eased(&master, 0), eased(&master, 2)), (4, 6));
    assert_eq!(eased(&master, -1), -1, "a failure stays one");
}

#[test]
fn every_grade_comes_four_sooner() {
    for (margin, without, with) in [
        (
            -1,
            "The blade comes out brittle",
            "The blade comes out brittle",
        ),
        (
            1,
            "You have made a short sword.",
            "You have made a short sword.",
        ),
        (
            2,
            "You have made a short sword.",
            "This is a Fine short sword.",
        ),
        (
            8,
            "This is a Fine short sword.",
            "This is a Marvellous short sword!",
        ),
        (
            16,
            "This is a Marvellous short sword!",
            "You have made a Masterwork short sword!",
        ),
    ] {
        for (talents, line) in [(&[][..], without), (&["masters_hand"][..], with)] {
            let mut w = world(talents);
            let steel = w.content.material_id("steel").expect("steel");
            let ingots = crate::piece::make(&w.content, steel, 2).expect("ingots");
            w.player_mut().inventory.push(ingots);
            make_out_for_test(&mut w, "short_sword", &[Some(0), None], 12, margin);
            assert!(
                said(&w).starts_with(line),
                "{margin} {talents:?}: {}",
                said(&w)
            );
        }
    }
}

#[test]
fn every_cut_comes_four_sooner() {
    let master = world(&["masters_hand"]);
    let gems = master.content.gems().expect("gems");
    let step = (master.content.recipes().settings.as_ref()).map_or(5, |s| s.cut_step);
    let eased = |m| super::mastery::graded_margin(&master.content, master.state.player(), m);
    let band = |m| cut_band(gems, step, 12, m);
    // A margin of 1 is a crude cut for anyone, a fair one in a master's hand.
    assert_ne!(band(1), band(eased(1)));
    assert_eq!(band(eased(1)), band(step));
    assert_eq!(band(eased(-1)), CutBand::Failed);
}
