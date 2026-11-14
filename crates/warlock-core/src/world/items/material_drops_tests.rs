//! Material drops (spec §7 "Material drops"; #1341): every tag's rows apply in path order, then a
//! creature's own; bodiless rows reach the skeleton and the elementals, a spirit leaves nothing;
//! a small body rolls at half; the roll has its own stream, so loot and gems never move; rows of
//! one material join one stack; the loader refuses what would drop nothing or the wrong thing;
//! the newly tagged beasts count as beasts; the arena's dead litter stock.

use crate::action::parse_actions;
use crate::content::{ContentFiles, Drop, DropFrom, DropGoods, PER_MILLE};
use crate::object::Stack;
use crate::test_support::builtin;
use crate::world::*;

fn rows<'c>(content: &'c Content, id: &str) -> &'c [Drop] {
    let c = content.creature_id(id).unwrap_or_else(|| panic!("{id}"));
    &content.creature(c).drops
}

fn material(content: &Content, row: &Drop) -> String {
    match row.goods {
        DropGoods::Material(m) => content.material(m).id.clone(),
        DropGoods::Object(o) => content.object(o).id.clone(),
    }
}

/// Each row of `id` as `(from tag or "own", goods, chance per mille)`.
fn summary(content: &Content, id: &str) -> Vec<(String, String, u16)> {
    (rows(content, id).iter())
        .map(|r| {
            let from = match &r.from {
                DropFrom::Tag(t) => t.clone(),
                DropFrom::Own => "own".into(),
            };
            (from, material(content, r), r.chance)
        })
        .collect()
}

fn s(from: &str, what: &str, chance: u16) -> (String, String, u16) {
    (from.into(), what.into(), chance)
}

#[test]
fn every_tag_applies_in_path_order_then_the_creatures_own() {
    let content = builtin();
    // A hobgoblin carries goblin and hobgoblin: both files, goblin's first (path order).
    assert_eq!(
        summary(&content, "hobgoblin"),
        vec![
            s("goblin", "flint", 300),
            s("goblin", "iron", 200),
            s("goblin", "leather", 300),
            s("hobgoblin", "iron", 400),
            s("hobgoblin", "steel", 150),
            s("hobgoblin", "leather", 300),
        ]
    );
    // A wolf: the beast rows, then its own fur.
    assert_eq!(
        summary(&content, "wolf"),
        vec![
            s("beast", "leather", 600),
            s("beast", "bone", 500),
            s("beast", "sinew", 400),
            s("own", "fur", 500),
        ]
    );
}

#[test]
fn bodiless_rows_reach_the_bodiless_and_a_spirit_leaves_nothing() {
    let content = builtin();
    // A skeleton leaves no corpse: the undead row is not its, the skeleton row is (bodiless).
    assert_eq!(
        summary(&content, "skeleton"),
        vec![s("skeleton", "bone", 800)]
    );
    assert_eq!(
        summary(&content, "fire_elemental"),
        vec![s("fire", "essence_of_fire", 600)]
    );
    assert!(rows(&content, "ghost").is_empty(), "a ghost leaves nothing");
    // The cinder elemental is a fire-thing too, but leaves no corpse: only its essence.
    assert_eq!(
        summary(&content, "cinder_elemental"),
        vec![s("fire", "essence_of_fire", 600)]
    );
}

#[test]
fn a_small_body_rolls_at_half_the_chance() {
    let content = builtin();
    assert_eq!(summary(&content, "rat"), vec![s("vermin", "bone", 150)]);
    assert_eq!(
        summary(&content, "kobold"),
        vec![
            s("kobold", "flint", 200),
            s("kobold", "wood", 125),
            s("kobold", "bone", 100),
        ]
    );
}

#[test]
fn the_untagged_beasts_are_beasts_now_and_the_gorgon_a_dragon() {
    let content = builtin();
    let beasts: Vec<&str> = (content.tagged("beast"))
        .map(|c| content.creature(c).id.as_str())
        .collect();
    for id in ["warg", "dire_warg", "deer", "bat"] {
        assert!(
            beasts.contains(&id),
            "{id} is a beast: a kill quest's tag:beast counts it"
        );
    }
    let gorgon = content.creature_id("gorgon").expect("gorgon");
    assert!(content.has_tag(gorgon, "dragon"));
    let target = crate::content::KillTarget::Tag("beast".into());
    let warg = content.creature_id("warg").expect("warg");
    assert!(target.has(&content, warg), "a warg counts for tag:beast");
    // Its dragon rows and its own.
    let from: Vec<String> = summary(&content, "gorgon")
        .into_iter()
        .map(|r| r.0)
        .collect();
    assert_eq!(from, ["dragon", "dragon", "dragon", "own", "own"]);
}

#[test]
fn the_draw_has_its_own_stream_and_loot_never_moves() {
    let content = builtin();
    let mut bare = ContentFiles::builtin();
    bare.remove("data/drops");
    let bare = Arc::new(Content::load(&bare).expect("content without drops"));
    let w = World::new(content.clone(), "arena", None).expect("world");
    let v = World::new(bare, "arena", None).expect("world");
    let kobold = (w.state().actors.values())
        .find(|a| content.creature(a.creature).id == "kobold")
        .cloned()
        .expect("a kobold");
    let (mut dropped, mut flint) = (0, 0);
    for id in 1..2000 {
        let mut a = kobold.clone();
        a.id = ActorId(id);
        assert_eq!(w.loot_of(&a), v.loot_of(&a), "loot and gems stand");
        let drops = w.drops_of(&a);
        assert_eq!(drops, w.drops_of(&a), "the same actor drops the same");
        assert!(v.drops_of(&a).is_empty(), "no files, no rows for a kobold");
        dropped += usize::from(!drops.is_empty());
        flint += usize::from(drops.iter().any(|s| names(&content, s).contains("flint")));
    }
    // Flint at 20 %: about 400 of 1999.
    assert!((300..=500).contains(&flint), "{flint} flint of {dropped}");
}

fn names(content: &Content, stack: &Stack) -> String {
    stack.names(content).0
}

const BONES: &str = r##"
[[creature]]
id = "zz_bones"
level = 1
name = "bone heap"
glyph = "b"
fg = "#ddddcc"
faction = "vermin"
tags = ["kobold"]
weight = 2000
stray = false
drops_from_tags = false
[[creature.drops]]
material = "bone"
count = "1"
chance = 100
[[creature.drops]]
material = "bone"
count = "2"
chance = 100
[[creature.drops]]
object = "essence_of_earth"
count = "1"
chance = 100
"##;

#[test]
fn rows_of_one_material_join_one_stack_and_tags_can_be_switched_off() {
    let mut files = ContentFiles::builtin();
    files.insert("data/creatures/zz_bones.toml", BONES);
    let content = Arc::new(Content::load(&files).expect("content"));
    assert_eq!(rows(&content, "zz_bones").len(), 3, "no kobold rows");
    let w = World::new(content.clone(), "arena", None).expect("world");
    let mut a = w.state().player().clone();
    a.creature = content.creature_id("zz_bones").expect("bones");
    a.id = ActorId(77);
    let drops = w.drops_of(&a);
    let told: Vec<(String, u32)> = (drops.iter())
        .map(|s| (names(&content, s), s.count))
        .collect();
    assert_eq!(
        told,
        [
            ("bone shard".to_string(), 3),
            ("essence of earth".to_string(), 1)
        ]
    );
}

fn refused(path: &str, text: &str) -> String {
    let mut files = ContentFiles::builtin();
    files.insert(path, text);
    Content::load(&files).expect_err(text).to_string()
}

fn row(what: &str, count: &str, chance: u32) -> String {
    format!("[[drop]]\n{what}\ncount = \"{count}\"\nchance = {chance}\n")
}

#[test]
fn the_loader_refuses_what_would_drop_nothing_or_the_wrong_thing() {
    let at = "data/drops/beast.toml";
    let bone = "material = \"bone\"";
    assert!(refused(at, &row("material = \"ruby\"", "1", 50)).contains("is a gem"));
    assert!(refused(at, &row("object = \"no_such\"", "1", 50)).contains("no object"));
    assert!(refused(at, &row("material = \"no_such\"", "1", 50)).contains("no material"));
    assert!(refused(at, &row("material = \"paper\"", "1", 50)).contains("no group"));
    assert!(refused(at, &row(bone, "1d", 50)).contains("dice"));
    assert!(refused(at, &row(bone, "1", 0)).contains("not 1 to 100"));
    assert!(refused(at, &row(bone, "1", 101)).contains("not 1 to 100"));
    let both = format!("{bone}\nobject = \"henbane\"");
    assert!(refused(at, &row(&both, "1", 50)).contains("one of them"));
    let typo = refused("data/drops/beasts.toml", &row(bone, "1", 50));
    assert!(
        typo.contains("no creature carries the tag `beasts`"),
        "{typo}"
    );
    let own = BONES.replace("\"bone\"\ncount = \"2\"", "\"ruby\"\ncount = \"2\"");
    assert!(refused("data/creatures/zz_bones.toml", &own).contains("zz_bones"));
}

#[test]
fn every_tag_file_loads_with_rows() {
    let content = builtin();
    let tags = content.tag_drops();
    assert!(tags.len() >= 20, "{} files", tags.len());
    for file in tags {
        assert!(!file.rows.is_empty(), "{}", file.tag);
        for row in &file.rows {
            assert!(
                (10..=PER_MILLE as u16).contains(&row.chance),
                "{}",
                file.tag
            );
        }
    }
}

#[test]
fn the_arenas_dead_litter_stock() {
    let content = builtin();
    let mut w = World::new(content.clone(), "arena", None).expect("world");
    for action in parse_actions("wait*400").expect("actions") {
        w.apply(&action);
    }
    let level = &w.state().levels[&w.state().current_level];
    let piece = content.object_id(crate::piece::ID).expect("piece");
    let pieces = (0..80)
        .flat_map(|x| (0..40).map(move |y| Pos::new(x, y)))
        .flat_map(|p| level.objects_at(p).iter())
        .filter(|s| s.object == piece)
        .count();
    assert!(pieces > 0, "the arena's dead leave stock");
}
