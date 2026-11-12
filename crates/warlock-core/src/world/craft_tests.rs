//! A profession's recipe rows and `craft:` (docs/dev/crafting.md; #1334): who sees which row,
//! what each takes in words, why one cannot be made now, the abilities menu's profession rows,
//! and `craft:` refused with no time.

use super::*;
use crate::ability::Ability;
use crate::content::ContentFiles;
use crate::object::Stack;
use crate::skills::{SkillId, Trained};
use std::sync::LazyLock;

/// The built-in content with two generic recipes: a long sword of stock, from Metalcraft 20, and
/// tipped arrows from arrows, for anyone.
static FIXTURE: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/recipes/metalcraft/long_sword.toml",
        "[[recipe]]\nid = \"long_sword\"\nresult = { object = \"long_sword\" }\ndifficulty = 13\n\
         seconds = 3600\nstation = \"anvil\"\nfrom = 20\n[recipe.parts]\nblade = \"stock\"\n",
    );
    files.insert(
        "data/recipes/metalcraft/tipped.toml",
        "[[recipe]]\nid = \"tipped\"\nname = \"Tipped arrows\"\nanyone = true\n\
         result = { object = \"arrow\", count = 5 }\ndifficulty = 10\nseconds = 600\n\
         inputs = [{ object = \"arrow\", count = 5 }]\n",
    );
    Arc::new(Content::load(&files).expect("fixture content is valid"))
});

/// A run of `scenario` as a human commoner carrying nothing, trained in no skill.
fn world(content: Arc<Content>, scenario: &str) -> World {
    let mut w = World::new(content, scenario, None).expect("world");
    let me = w.player_mut();
    me.inventory.clear();
    me.equipment = Default::default();
    me.skills = Default::default();
    w
}

fn skill(w: &World, id: &str) -> SkillId {
    w.content.skill_id(id).expect(id)
}

fn train(w: &mut World, id: &str, value: u16) {
    let id = skill(w, id);
    let mut all: Vec<(SkillId, Trained)> = w.player_mut().skills.iter().collect();
    all.push((id, Trained { value, marks: 0 }));
    w.player_mut().skills = all.into_iter().collect();
}

fn give(w: &mut World, id: &str, count: u32) {
    let object = w.content.object_id(id).expect(id);
    w.player_mut().inventory.push(Stack::new(object, count));
}

/// The rows of profession `id`, `Name: takes (why)`.
fn rows(w: &World, id: &str) -> Vec<String> {
    let content = w.content.clone();
    (w.perception().recipes(skill(w, id)).into_iter())
        .map(|r| match r.unavailable {
            Some(why) => format!("{}: {} ({})", r.name, r.takes, why.reason(&content)),
            None => format!("{}: {}", r.name, r.takes),
        })
        .collect()
}

#[test]
fn everyone_sees_the_built_in_rows_with_what_they_take_and_why_not() {
    let content = Arc::new(Content::builtin().expect("content"));
    let mut w = world(content.clone(), "showcase");
    assert_eq!(
        rows(&w, "metalcraft"),
        ["Temper: a metal weapon or armour, an anvil, about 10 minutes (no anvil beside you)"]
    );
    assert_eq!(
        rows(&w, "woodcraft"),
        [
            "Fletch: a knife, a bow or crossbow ready, about 10 minutes (no knife)",
            "Whittle: a wooden weapon, a knife, about 10 minutes (no knife)",
        ]
    );
    // A knife: Whittle can be; Fletch still wants a launcher.
    give(&mut w, "knife", 1);
    assert_eq!(
        rows(&w, "woodcraft"),
        [
            "Fletch: a knife, a bow or crossbow ready, about 10 minutes \
             (no bow or crossbow ready)",
            "Whittle: a wooden weapon, a knife, about 10 minutes",
        ]
    );
    // Beside the forge's anvil, Temper can be.
    let w = world(content, "forge");
    let mut w = w;
    w.apply(&Action::Move(Dir::E));
    assert_eq!(
        rows(&w, "metalcraft"),
        ["Temper: a metal weapon or armour, an anvil, about 10 minutes"]
    );
}

#[test]
fn the_untrained_see_only_what_anyone_may_make_and_the_trained_what_is_beyond_them() {
    let mut w = world(FIXTURE.clone(), "showcase");
    let names = |w: &World| -> Vec<String> {
        (w.perception().recipes(skill(w, "metalcraft")).into_iter())
            .map(|r| r.name)
            .collect()
    };
    assert_eq!(names(&w), ["Temper", "Tipped arrows"]);
    // Trained below the long sword's `from`: listed, beyond them.
    train(&mut w, "metalcraft", 10);
    assert_eq!(
        rows(&w, "metalcraft")[3],
        "Long sword: 3 steel ingots, an anvil, about 1 hour (beyond your skill)"
    );
    // At it: the anvil, then the stock, is what is missing.
    train(&mut w, "metalcraft", 20);
    assert!(rows(&w, "metalcraft")[3].ends_with("(no anvil beside you)"));
    // Five arrows to tip: none carried, then too few, then enough.
    assert_eq!(
        rows(&w, "metalcraft")[8],
        "Tipped arrows: 5 arrows, about 10 minutes (no arrows)"
    );
    give(&mut w, "arrow", 4);
    assert!(rows(&w, "metalcraft")[8].ends_with("(no arrows)"));
    give(&mut w, "arrow", 1);
    assert_eq!(
        rows(&w, "metalcraft")[8],
        "Tipped arrows: 5 arrows, about 10 minutes"
    );
}

#[test]
fn a_stock_part_wants_a_piece_the_profession_works_that_the_part_may_be_made_of() {
    let mut w = world(FIXTURE.clone(), "forge");
    w.apply(&Action::Move(Dir::E));
    train(&mut w, "metalcraft", 20);
    let sword = |w: &World| rows(w, "metalcraft")[3].clone();
    assert!(sword(&w).ends_with("(no metal to hand)"), "{}", sword(&w));
    // Hardwood is no metal; iron is.
    let wood = w.content.material_id("hardwood").expect("hardwood");
    let billet = crate::piece::make(&w.content, wood, 2).expect("a billet");
    w.player_mut().inventory.push(billet);
    assert!(sword(&w).ends_with("(no metal to hand)"));
    let iron = w.content.material_id("iron").expect("iron");
    let ingot = crate::piece::make(&w.content, iron, 1).expect("an ingot");
    w.player_mut().inventory.push(ingot);
    assert_eq!(
        sword(&w),
        "Long sword: 3 steel ingots, an anvil, about 1 hour"
    );
}

#[test]
fn the_abilities_menu_lists_a_row_per_profession_and_no_temper_whittle_or_fletch() {
    let content = Arc::new(Content::builtin().expect("content"));
    let w = world(content.clone(), "forge");
    let mut w = w;
    w.apply(&Action::Move(Dir::E));
    let view = w.perception();
    let own: Vec<(String, bool)> = (view.abilities().iter())
        .map(|a| {
            (
                view.ability_name(a.ability).into_owned(),
                a.unavailable.is_none(),
            )
        })
        .collect();
    // Untrained: both rows all the same (their built-in recipes are anyone's), usable; beside
    // the anvil, no Temper of the Here group.
    let metal = Ability::Profession(skill(&w, "metalcraft"));
    let wood = Ability::Profession(skill(&w, "woodcraft"));
    for row in [metal, wood] {
        assert!(
            view.abilities()
                .iter()
                .any(|a| a.ability == row && a.unavailable.is_none())
        );
    }
    let names: Vec<&str> = own.iter().map(|(n, _)| n.as_str()).collect();
    assert!(
        names.contains(&"Metalcraft…") && names.contains(&"Woodcraft…"),
        "{names:?}"
    );
    for gone in ["Temper", "Whittle", "Fletch", "Metalcraft", "Woodcraft"] {
        assert!(!names.contains(&gone), "{gone} in {names:?}");
    }
    assert_eq!(metal.description(&content), "Your recipes as a smith.");
}

#[test]
fn crafting_a_built_in_unknown_or_out_of_reach_recipe_is_refused_with_no_time() {
    let mut w = world(FIXTURE.clone(), "forge");
    let start = w.state.time;
    for (text, said) in [
        ("craft:temper", "Temper is done from its own list."),
        ("craft:long_sword:a", "That is beyond your skill."),
        ("craft:moonblade", "You know no such recipe."),
    ] {
        for action in crate::action::parse_actions(text).expect("parses") {
            w.apply(&action);
        }
        assert_eq!(w.state.time, start, "{text}");
        let last = (w.messages().iter().last()).map(|m| m.text.clone());
        assert_eq!(last.as_deref(), Some(said), "{text}");
    }
}

#[test]
fn jewelcrafts_cut_is_usable_and_set_listed_but_not_usable_yet() {
    // P11 adds Set's file; a fixture stands in for it now, open to anyone.
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/recipes/jewelcraft/set.toml",
        "[[recipe]]\nid = \"set\"\nuse = \"set\"\ndifficulty = 12\nseconds = 1200\n\
         anyone = true\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    let mut w = world(content, "showcase");
    // Cut is the trained's alone.
    assert_eq!(
        rows(&w, "jewelcraft"),
        ["Set: an item and a cut gem, about 20 minutes (not usable yet)"]
    );
    let jewel = skill(&w, "jewelcraft");
    w.player_mut().skills = [(jewel, Trained { value: 5, marks: 0 })]
        .into_iter()
        .collect();
    assert_eq!(
        rows(&w, "jewelcraft"),
        [
            "Cut: a loose gem, about 30 minutes",
            "Set: an item and a cut gem, about 20 minutes (not usable yet)"
        ]
    );
    let start = w.state.time;
    for action in crate::action::parse_actions("craft:set").expect("parses") {
        w.apply(&action);
    }
    assert_eq!(w.state.time, start);
    let last = (w.messages().iter().last()).map(|m| m.text.clone());
    assert_eq!(last.as_deref(), Some("You cannot do that yet."));
}
