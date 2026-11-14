//! Waste not (#1420; the craftsperson design §7.3): the craftsperson's at level 6, and its three
//! reads: salvage's yield (`world/crafting/salvage.rs::salvage_percent`), a failed recipe's stock and
//! inputs all back (`world/crafting/craft_make.rs::failed_percent`), and a failed Temper or Whittle that
//! damages nothing (`world/crafting/temper.rs`), its attempt counted. Without the talent each is as ever.

use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, Factor};
use crate::magic::{Aspect, Worked};
use crate::object::Stack;
use crate::skills::{SkillId, Trained};
use crate::test_support::{builtin, said_text as said};
use crate::world::crafting::craft_make::make_out_for_test;
use crate::world::*;
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
        "levels/test_waste.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{ROOM}'''\n\
             [[room]]\neffect = \"forge\"\narea = [0, 0, 10, 5]\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"A\" = \"anvil\"\n"
        ),
    );
    files.insert(
        "scenarios/test_waste.toml",
        "name = \"T\"\nstart_level = \"test_waste\"\n\
         character = { race = \"dwarf\", gender = \"female\", class = \"craftsperson\" }\n",
    );
    Arc::new(Content::load(&files).expect("fixture content is valid"))
});

/// The forge, the player carrying nothing, trained in Metalcraft and Woodcraft at 50, holding
/// Waste not when `waste_not`.
fn world(waste_not: bool) -> World {
    let mut w = World::new(FIXTURE.clone(), "test_waste", None).expect("world");
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
    if waste_not {
        w.player_mut().talents.take("waste_not", None);
    }
    w
}

fn one(w: &World, id: &str) -> Stack {
    Stack::one(w.content.object_id(id).expect(id))
}

fn pack(w: &World) -> Vec<String> {
    (w.state.player().inventory.iter())
        .map(|s| w.lore().describe(s))
        .collect()
}

/// How many pieces salvage would give of `id` made at `percent`.
fn pieces(w: &World, id: &str, percent: u8) -> u32 {
    crate::piece::break_up(&one(w, id), &w.content, percent)
        .iter()
        .map(|s| s.count)
        .sum()
}

#[test]
fn the_craftsperson_takes_waste_not_at_level_six() {
    let spec: CharacterSpec = "human,male,craftsperson".parse().expect("a character");
    let content = builtin();
    let mut w =
        World::with_character(content.clone(), "showcase", Some(7), Some(&spec)).expect("world");
    let factor = Factor::of(&content, w.state.character.as_ref().map(|c| &c.spec));
    let held = |w: &World| w.state.player().talents.holds("waste_not", None);
    for level in 2..=6 {
        assert!(!held(&w), "not before level 6");
        let more = content.experience().threshold_for(level, factor) - w.player_xp();
        for action in parse_actions(&format!("xp:{more}")).expect("parses") {
            w.apply(&action);
        }
        while w.due().is_some() {
            w.apply(&parse_actions("levelup:auto").expect("parses")[0]);
        }
    }
    assert_eq!(w.player_level(), 6);
    assert!(held(&w), "at level 6");
}

#[test]
fn salvage_gives_back_three_quarters() {
    let plain = world(false);
    let thrifty = world(true);
    let percent = |w: &World| salvage_percent(&w.content, w.state.player());
    assert_eq!((percent(&plain), percent(&thrifty)), (50, 75));
    // A quarterstaff 8 billets for 5, chain mail 17 ingots for 11.
    assert_eq!(
        (
            pieces(&plain, "quarterstaff", 50),
            pieces(&plain, "quarterstaff", 75)
        ),
        (5, 8)
    );
    assert_eq!(
        (
            pieces(&plain, "chain_mail", 50),
            pieces(&plain, "chain_mail", 75)
        ),
        (11, 17)
    );
    // And the work reads it: a quarterstaff taken apart, its check made.
    let mut w = world(true);
    let staff = one(&w, "quarterstaff");
    w.player_mut().inventory.insert(0, staff);
    let saved = w.state.clone();
    let made = (0..100).any(|n| {
        w.state = saved.clone();
        w.state.seed = n;
        w.apply(&Action::Salvage { item: 0 });
        pack(&w) == ["8 wood billets"]
    });
    assert!(made, "some fate makes the check: {:?}", pack(&w));
}

#[test]
fn a_failed_recipe_gives_back_all_its_stock_and_the_gem() {
    let mut w = world(true);
    let steel = w.content.material_id("steel").expect("steel");
    let ingots = crate::piece::make(&w.content, steel, 7).expect("ingots");
    w.player_mut().inventory.push(ingots);
    let content = w.content.clone();
    let gems = content.gems().expect("gems");
    let mut cutting = gems.plain();
    cutting.cut = gems.cut_id("fair").expect("fair");
    let ruby = content.material_id("ruby").expect("ruby");
    let part = crate::item_parts::ItemPart::gem(&content, None, ruby, cutting);
    (w.player_mut().inventory).push(Stack::one(gems.object).with_parts(vec![part]));
    make_out_for_test(&mut w, "great_sword", &[Some(0), Some(1)], 12, -1);
    assert!(said(
        &w,
        "The blade comes out brittle and you break it up for the metal, every ounce."
    ));
    // Without it, four of the seven (a_failure_makes_nothing_and_gives_half_the_stock_back...).
    assert_eq!(pack(&w), ["7 steel ingots", "a ruby"]);
    // A natural 2 the same.
    let mut w = world(true);
    let ingots = crate::piece::make(&w.content, steel, 2).expect("ingots");
    w.player_mut().inventory.push(ingots);
    make_out_for_test(&mut w, "short_sword", &[Some(0), None], 2, 30);
    assert_eq!(pack(&w), ["2 steel ingots"]);
}

/// Attempts `action` on stack 0 from the same state on fate's draws until one fails (the item
/// is untouched by the talent, so a failure is told by its line): whether one did.
fn fail(w: &mut World, action: &Action, line: &str) -> bool {
    let saved = w.state.clone();
    (0..300).any(|n| {
        w.state = saved.clone();
        w.state.seed = n;
        w.apply(action);
        said(w, line)
    })
}

#[test]
fn a_failed_temper_spoils_nothing_and_counts() {
    let spared = "The work goes wrong, but you know when to stop before the steel spoils.";
    let mut w = world(true);
    let mut sword = one(&w, "long_sword");
    sword.set_worked(Worked {
        improved: vec![Aspect::Die, Aspect::ToHit],
        attempts: 2,
        damaged: false,
        by: None,
        marked: false,
    });
    w.player_mut().inventory.insert(0, sword);
    assert!(fail(&mut w, &Action::Temper { item: 0 }, spared));
    let after = w.state.player().inventory[0]
        .worked()
        .cloned()
        .expect("worked");
    assert_eq!(after.improved, [Aspect::Die, Aspect::ToHit], "kept");
    assert_eq!((after.damaged, after.attempts), (false, 3), "counted");
    // A damaged item is never destroyed by a failure.
    let mut w = world(true);
    let mut sword = one(&w, "long_sword");
    sword.set_worked(Worked {
        improved: Vec::new(),
        attempts: 2,
        damaged: true,
        by: None,
        marked: false,
    });
    w.player_mut().inventory.insert(0, sword);
    assert!(fail(&mut w, &Action::Temper { item: 0 }, spared));
    let after = w.state.player().inventory[0]
        .worked()
        .cloned()
        .expect("kept");
    assert_eq!((after.damaged, after.attempts), (true, 3));
    assert!(!said(&w, "It shatters under the hammer!"));
    // Without the talent the same failure cracks it.
    let mut w = world(false);
    let sword = one(&w, "long_sword");
    w.player_mut().inventory.insert(0, sword);
    assert!(fail(
        &mut w,
        &Action::Temper { item: 0 },
        "The metal cracks under the hammer."
    ));
}

#[test]
fn adamantium_withstands_before_waste_not_spares() {
    // #1444 with #1420: a proof item's failure is withstood by its material, its own line said.
    let mut w = world(true);
    let m = w.content.material_id("adamantium").expect("adamantium");
    let part = crate::item_parts::ItemPart::material(&w.content, None, m);
    let sword = one(&w, "long_sword").with_parts(vec![part]);
    w.player_mut().inventory.insert(0, sword);
    let withstood = "The work goes wrong, but no harm is done.";
    assert!(fail(&mut w, &Action::Temper { item: 0 }, withstood));
    assert!(!said(
        &w,
        "The work goes wrong, but you know when to stop before the steel spoils."
    ));
    let after = w.state.player().inventory[0]
        .worked()
        .cloned()
        .expect("worked");
    assert_eq!((after.damaged, after.attempts), (false, 1));
}

#[test]
fn a_failed_whittle_spoils_nothing() {
    let mut w = world(true);
    let (knife, bow) = (one(&w, "knife"), one(&w, "short_bow"));
    w.player_mut().inventory.push(knife);
    w.player_mut().inventory.insert(0, bow);
    assert!(fail(
        &mut w,
        &Action::Whittle { item: 0 },
        "The work goes wrong, but you know when to stop before the wood spoils."
    ));
    let after = w.state.player().inventory[0]
        .worked()
        .cloned()
        .expect("worked");
    assert_eq!((after.damaged, after.attempts), (false, 1));
}
