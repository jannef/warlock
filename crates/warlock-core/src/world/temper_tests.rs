//! The forge (spec §7 "Special rooms"; `world/temper.rs`, `magic/temper.rs`): Temper listed only
//! beside an anvil and only for weapons and armour, ten minutes' work that an interruption
//! spoils, the Metalcraft lean, each outcome's work on the item, identification, the mark, and an
//! anvil that is never spent.

use super::*;
use crate::ability::Ability;
use crate::content::{ContentFiles, OutcomeDef, OutcomeKind, OutcomeTable};
use crate::magic::{Affix, Magic, Rarity, may_roll};
use crate::object::Stack;

/// A forge with its anvil at (5, 2), the player beside it at (4, 3), and at `k` a kobold (also
/// beside it), when asked for.
const ROOM: &str = "\
###########
#.........#
#....A....#
#...@k....#
#.........#
#.........#
###########
";

/// The anvil.
const ANVIL: Pos = Pos::new(5, 2);

/// The built-in content with [`ROOM`], a forge; the kobold only with `kobold`. The player (a
/// craftsperson, trained in Metalcraft) carries only what a test gives them.
fn world(kobold: bool) -> World {
    let mut files = ContentFiles::builtin();
    let k = if kobold {
        "{ terrain = \"floor\", creature = \"kobold\" }"
    } else {
        "\"floor\""
    };
    files.insert(
        "levels/test_forge.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{ROOM}'''\n\
             [[room]]\neffect = \"forge\"\narea = [0, 0, 10, 6]\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"A\" = \"anvil\"\n\"k\" = {k}\n"
        ),
    );
    files.insert(
        "scenarios/test_forge.toml",
        "name = \"T\"\nstart_level = \"test_forge\"\n\
         character = { race = \"dwarf\", gender = \"female\", class = \"craftsperson\" }\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    let mut w = World::new(content, "test_forge", None).expect("world");
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    me.inventory.clear();
    me.equipment = Default::default();
    w
}

fn object(w: &World, id: &str) -> crate::content::ObjectId {
    w.content.object_id(id).expect(id)
}

/// A long sword with `n` affixes that roll on it, of tier 1, in the pack's first place; plain
/// with none.
fn sword(w: &mut World, n: usize) {
    let id = object(w, "long_sword");
    let def = w.content.object(id).clone();
    let kind = def.kind.expect("a weapon");
    let mut affixes: Vec<Affix> = Vec::new();
    let mut groups = Vec::new();
    let most = if n > 2 { 3 } else { 1 };
    for a in w.content.affixes().all() {
        if affixes.len() == n {
            break;
        }
        let per_kind = affixes
            .iter()
            .filter(|x| {
                w.content
                    .affixes()
                    .get(&x.id)
                    .is_some_and(|d| d.kind == a.kind)
            })
            .count();
        if may_roll(a, kind, &def) && !groups.contains(&a.group()) && per_kind < most {
            groups.push(a.group());
            affixes.push(Affix {
                id: a.id.clone(),
                tier: 1,
                value: a.tiers[0].min,
            });
        }
    }
    let stack = if n == 0 {
        Stack::one(id)
    } else {
        let rarity = if n > 2 { Rarity::Rare } else { Rarity::Magic };
        let magic = Magic {
            rarity,
            level: 10,
            affixes,
            name: (rarity == Rarity::Rare).then(|| "Grim Bite".to_string()),
            identified: false,
            carried: 0,
            unique: None,
        };
        Stack::with_magic(id, 1, magic)
    };
    w.player_mut().inventory.insert(0, stack);
}

fn table(w: &World) -> OutcomeTable {
    w.content.outcomes().get("forge").expect("table").clone()
}

fn outcome(w: &World, id: &str) -> OutcomeDef {
    (table(w).outcomes.into_iter())
        .find(|o| o.id == id)
        .expect(id)
}

/// `id` befalls the stack at `item`.
fn work(w: &mut World, id: &str, item: usize) {
    let outcome = outcome(w, id);
    w.befall_on(&outcome, ANVIL, Some(item), &mut Vec::new());
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

fn here(w: &World) -> Vec<Ability> {
    (w.perception().abilities().into_iter())
        .map(|a| a.ability)
        .filter(|a| a.source() == crate::ability::Source::Place)
        .collect()
}

fn magic(w: &World, item: usize) -> Option<Magic> {
    w.state.player().inventory[item].magic().cloned()
}

#[test]
fn temper_is_listed_only_beside_an_anvil() {
    let mut w = world(false);
    assert_eq!(here(&w), vec![Ability::Temper]);
    assert!(Ability::every(&w.content).contains(&Ability::Temper));
    assert_eq!(
        Ability::Temper.aim(&w.content),
        crate::ability::Aim::Item,
        "it asks for an item"
    );
    sword(&mut w, 0);
    w.apply(&Action::Move(Dir::S));
    assert!(here(&w).is_empty(), "two cells away");
    let before = w.state.time;
    w.apply(&Action::Temper { item: 0 });
    assert!(said(&w, "There is no anvil here."));
    assert_eq!(w.state.time, before, "refused, no time");
}

#[test]
fn only_weapons_and_armour_are_tempered() {
    let mut w = world(false);
    let turnip = object(&w, "turnip");
    w.player_mut().inventory.push(Stack::one(turnip));
    let before = w.state.time;
    w.apply(&Action::Temper { item: 0 });
    assert!(said(&w, "Only weapons and armour can be tempered."));
    w.apply(&Action::Temper { item: 5 });
    assert!(said(&w, "You carry no such thing."));
    assert_eq!(w.state.time, before, "refused, no time");
    let content = w.content.clone();
    let kinds = |id: &str| temperable(&content, &Stack::one(content.object_id(id).expect(id)));
    assert!(kinds("long_sword") && kinds("chain_mail") && kinds("small_shield"));
    assert!(!kinds("turnip"));
}

#[test]
fn tempering_takes_ten_minutes_identifies_and_trains_metalcraft() {
    let mut w = world(false);
    sword(&mut w, 1);
    let metalcraft = w.content.skill_id("metalcraft").expect("metalcraft");
    let marks = |w: &World| {
        w.state
            .player()
            .skills
            .get(metalcraft)
            .map_or(0, |t| t.marks)
    };
    let before = marks(&w);
    let start = w.state.time;
    w.apply(&Action::Temper { item: 0 });
    let first: Vec<_> = w.messages().iter().map(|m| m.text.clone()).collect();
    assert!(
        (w.messages().iter()).any(|m| m.text.starts_with("You set to work on the long sword")),
        "{first:?}"
    );
    assert_eq!(w.state.time.since(start).as_seconds(), 600);
    let lines = [
        "The metal rings true.",
        "The metal rings true, but it has nothing more to give.",
        "Nothing changes.",
        "Something in the metal is lost.",
        "It shatters under the hammer!",
    ];
    assert!(lines.iter().any(|l| said(&w, l)), "an outcome is said");
    if let Some(stack) = w.state.player().inventory.first() {
        assert!(stack.is_identified(), "what survives is identified");
    }
    assert!(marks(&w) > before, "every tempering marks Metalcraft");
    let anvil = w.state.current().terrain_at(ANVIL).expect("cell");
    assert_eq!(w.content.terrain(anvil).id, "anvil", "never spent");
    assert_eq!(here(&w), vec![Ability::Temper]);
}

#[test]
fn an_interrupted_tempering_does_nothing_to_the_item() {
    let mut w = world(true);
    sword(&mut w, 1);
    let item = magic(&w, 0).expect("magic");
    let start = w.state.time;
    w.apply(&Action::Temper { item: 0 });
    assert!(said(&w, "You put down the hammer."));
    assert!(w.state.time.since(start).as_seconds() < 600);
    let now = magic(&w, 0).expect("magic");
    assert_eq!(now.affixes, item.affixes, "untouched");
    assert!(!now.identified, "not even identified");
}

#[test]
fn metalcraft_leans_the_forge_by_the_checks_margin() {
    let w = world(false);
    let table = table(&w);
    assert_eq!(table.exhaust, 0, "never spent");
    assert_eq!(table.seconds, 600);
    assert_eq!(lean_for(0, table.lean), 0);
    assert_eq!(lean_for(4, table.lean), 8);
    assert_eq!(lean_for(15, table.lean), 20, "held to the table's lean");
    assert_eq!(lean_for(-15, table.lean), -20);
    assert_eq!(table.leaned(0), [35, 30, 35]);
    assert_eq!(table.leaned(20), [55, 30, 15]);
    assert_eq!(table.leaned(-20), [15, 30, 55]);
}

#[test]
fn the_draw_comes_out_near_the_tables_weights() {
    let mut w = world(false);
    let table = table(&w);
    let mut counts = std::collections::BTreeMap::<String, u32>::new();
    let n = 4000;
    for _ in 0..n {
        let o = w.draw_outcome(&table, table.leaned(0)).expect("an outcome");
        *counts.entry(o.id.clone()).or_default() += 1;
    }
    for (id, want) in [
        ("improved", 35),
        ("nothing", 30),
        ("flawed", 25),
        ("ruined", 10),
    ] {
        let got = counts.get(id).copied().unwrap_or(0) * 100 / n;
        assert!(got.abs_diff(want) <= 3, "{id}: {got}% against {want}%");
    }
    assert_eq!(table.total(OutcomeKind::Bane), 35);
}

#[test]
fn improving_adds_an_affix_while_the_rarity_has_room_else_raises_one() {
    let mut w = world(false);
    // A plain sword becomes magic.
    sword(&mut w, 0);
    work(&mut w, "improved", 0);
    let m = magic(&w, 0).expect("magic now");
    assert_eq!((m.rarity, m.affixes.len()), (Rarity::Magic, 1));
    assert!(said(&w, "The metal rings true."));
    // A magic sword with a prefix and a suffix is full: its strongest goes a tier up.
    let mut w = world(false);
    sword(&mut w, 2);
    let before = magic(&w, 0).expect("magic");
    work(&mut w, "improved", 0);
    let after = magic(&w, 0).expect("magic");
    assert_eq!(after.affixes.len(), 2);
    let raised: Vec<_> = (before.affixes.iter().zip(&after.affixes))
        .filter(|(a, b)| b.tier == a.tier + 1)
        .collect();
    assert_eq!(raised.len(), 1, "one affix a tier up");
    let id = w.state.player().inventory[0].object;
    let def = w.content.object(id).clone();
    assert!(after.check(&def, &w.content).is_ok(), "still a valid item");
    // A rare sword with room gains one.
    let mut w = world(false);
    sword(&mut w, 3);
    work(&mut w, "improved", 0);
    let m = magic(&w, 0).expect("rare");
    assert_eq!((m.rarity, m.affixes.len()), (Rarity::Rare, 4));
    assert!(m.check(&def, &w.content).is_ok(), "still a valid item");
}

#[test]
fn flawing_loses_an_affix_and_ruin_breaks_the_item() {
    let mut w = world(false);
    sword(&mut w, 1);
    work(&mut w, "flawed", 0);
    assert!(magic(&w, 0).is_none(), "its last affix lost: plain");
    assert!(said(&w, "Something in the metal is lost."));
    // Nothing to lose on a plain sword.
    let mut w = world(false);
    sword(&mut w, 0);
    work(&mut w, "flawed", 0);
    assert!(said(
        &w,
        "The metal dulls, but there was nothing in it to lose."
    ));
    assert!(!said(&w, "Something in the metal is lost."));
    // Ruined, and wielded: gone from the pack and the hand.
    let mut w = world(false);
    sword(&mut w, 2);
    let turnip = object(&w, "turnip");
    w.player_mut().inventory.push(Stack::one(turnip));
    w.apply(&Action::Equip {
        slot: "main_hand".into(),
        item: 0,
    });
    assert!(!w.state.player().equipment.items().is_empty(), "wielded");
    work(&mut w, "ruined", 0);
    assert!(said(&w, "It shatters under the hammer!"));
    let me = w.state.player();
    assert_eq!(me.inventory.len(), 1);
    assert_eq!(me.inventory[0].object, turnip);
    assert!(me.equipment.items().is_empty(), "nothing wielded");
}

#[test]
fn one_of_a_stack_is_worked_apart_from_the_rest() {
    let mut w = world(false);
    let dagger = object(&w, "dagger");
    w.player_mut().inventory.push(Stack::new(dagger, 3));
    let (at, apart) = w.set_apart(0);
    assert!(apart);
    assert_eq!(at, 1);
    work(&mut w, "improved", at);
    let pack = &w.state.player().inventory;
    assert_eq!((pack[0].count, pack[1].count), (2, 1));
    assert!(pack[1].magic().is_some() && pack[0].magic().is_none());
    // Nothing done, it joins the rest again.
    w.rejoin(1);
    assert_eq!(w.state.player().inventory.len(), 2, "unlike: stays apart");
}
