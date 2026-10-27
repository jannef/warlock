//! Healing springs (spec §7 "Special rooms"; the design's §9.10; `world/drink.rs`): Drink is
//! listed only beside a spring, a drink takes ten seconds, a broken one gives nothing, each
//! outcome's hook, the shares of the draw, and a spring that never runs dry.

use super::*;
use crate::ability::Ability;
use crate::content::{ContentFiles, OutcomeDef, OutcomeKind, OutcomeTable};

/// A room with a spring at (5, 2), the player beside it at (4, 3), and at `k` a kobold (also
/// beside it), when asked for.
const ROOM: &str = "\
###########
#.........#
#....S....#
#...@k....#
#.........#
#.........#
###########
";

/// The spring.
const SPRING: Pos = Pos::new(5, 2);

/// The built-in content with [`ROOM`], a spring room; the kobold only with `kobold`.
fn world(kobold: bool) -> World {
    let mut files = ContentFiles::builtin();
    let k = if kobold {
        "{ terrain = \"floor\", creature = \"kobold\" }"
    } else {
        "\"floor\""
    };
    files.insert(
        "levels/test_spring.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{ROOM}'''\n\
             [[room]]\neffect = \"spring\"\narea = [0, 0, 10, 6]\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"S\" = \"spring\"\n\"k\" = {k}\n"
        ),
    );
    files.insert(
        "scenarios/test_spring.toml",
        "name = \"T\"\nstart_level = \"test_spring\"\n\
         character = { race = \"human\", gender = \"female\", class = \"priest\" }\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_spring", None).expect("world")
}

fn table(w: &World) -> OutcomeTable {
    (w.content.outcomes().get("spring"))
        .expect("the spring's table")
        .clone()
}

fn outcome(w: &World, id: &str) -> OutcomeDef {
    let table = table(w);
    (table.outcomes.iter())
        .find(|o| o.id == id)
        .expect(id)
        .clone()
}

fn befall(w: &mut World, id: &str) {
    let outcome = outcome(w, id);
    w.befall(&outcome, SPRING, &mut Vec::new());
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

#[test]
fn drink_is_listed_only_beside_a_spring() {
    let mut w = world(false);
    assert_eq!(here(&w), vec![Ability::Drink]);
    assert!(Ability::every(&w.content).contains(&Ability::Drink));
    w.apply(&Action::Move(Dir::S));
    assert!(here(&w).is_empty(), "two cells away");
    let before = w.state.time;
    w.apply(&Action::Drink);
    assert!(said(&w, "There is no spring here to drink from."));
    assert_eq!(w.state.time, before, "refused, no time");
    // Pray is not offered at a spring, nor Drink at an altar.
    w.apply(&Action::Move(Dir::N));
    w.apply(&Action::Pray);
    assert!(said(&w, "There is no altar here to pray at."));
}

#[test]
fn a_drink_takes_ten_seconds_and_a_broken_one_gives_nothing() {
    let mut w = world(false);
    let start = w.state.time;
    w.apply(&Action::Drink);
    assert_eq!(w.state.time.since(start).as_seconds(), 10);
    assert!(said(&w, "You cup your hands and drink from the spring."));
    let lines: Vec<String> = (table(&w).outcomes.iter())
        .flat_map(|o| o.messages.clone())
        .collect();
    let last = w.messages().iter().last().expect("a line").text.clone();
    assert!(lines.contains(&last), "an outcome's line: {last}");
    // Broken by the kobold beside the player, who attacks.
    let mut w = world(true);
    let start = w.state.time;
    w.apply(&Action::Drink);
    assert!(said(&w, "You stop drinking."));
    assert!(w.state.time.since(start).as_seconds() < 10);
    assert!(!lines.iter().any(|l| said(&w, l)), "no outcome");
}

#[test]
fn each_outcome_does_what_it_says() {
    // Refreshed: hit points back, never past the maximum.
    let mut w = world(false);
    let max = w.state.player().max_hp(&w.content);
    w.player_mut().hp = 1;
    befall(&mut w, "refreshed");
    let hp = w.state.player().hp;
    assert!((3..=17).contains(&hp), "2d8 back: {hp}");
    assert!(said(&w, "The water is cold and good."));
    w.player_mut().hp = max;
    befall(&mut w, "refreshed");
    assert_eq!(w.state.player().hp, max, "never past the maximum");
    // Clear-headed: half the mana back.
    let max = w.state.player().max_mana(&w.content);
    assert!(max >= 2, "a priest has mana");
    w.player_mut().mana = 0;
    befall(&mut w, "clear_headed");
    assert_eq!(w.state.player().mana, max / 2);
    assert!(said(&w, "Your head clears."));
    befall(&mut w, "clear_headed");
    befall(&mut w, "clear_headed");
    assert_eq!(w.state.player().mana, max, "never past the maximum");
    // Fouled: marsh ague caught, which a prayer cures.
    let ague = w.content.disease_id("marsh_ague").expect("marsh ague");
    befall(&mut w, "fouled");
    assert!(w.state.player().diseases.has(ague));
    assert!(said(&w, "The water tastes foul."));
    let means = &w.content.disease(ague).cured_by;
    assert!(means.iter().any(|m| m == "prayer"), "{means:?}");
    // Nothing: the line, and nothing else.
    let mut w = world(false);
    let before = w.state.player().clone();
    befall(&mut w, "stone");
    assert!(said(&w, "It tastes of stone."));
    assert_eq!(w.state.player(), &before);
}

#[test]
fn the_shares_follow_the_table() {
    let mut w = world(false);
    let table = table(&w);
    assert_eq!(table.exhaust, 0, "a spring never runs dry");
    assert_eq!(table.lean, 0, "a spring has no side");
    let mut counts = [0u32; 3];
    let n = 4000;
    for _ in 0..n {
        let o = w.draw_outcome(&table, table.kinds).expect("an outcome");
        counts[o.kind as usize] += 1;
    }
    let total: u32 = table.kinds.iter().sum();
    for kind in OutcomeKind::ALL {
        let want = f64::from(table.weight(kind)) / f64::from(total);
        let got = f64::from(counts[kind as usize]) / f64::from(n);
        assert!((got - want).abs() < 0.03, "{kind:?}: {got} against {want}");
    }
}

#[test]
fn a_spring_never_runs_dry() {
    let mut w = world(false);
    let spring = w.state.current().terrain_at(SPRING);
    for _ in 0..30 {
        w.apply(&Action::Drink);
        w.player_mut().diseases = Default::default();
        assert_eq!(w.state.current().terrain_at(SPRING), spring);
    }
    assert_eq!(here(&w), vec![Ability::Drink]);
}

#[test]
fn the_spring_and_its_table_load() {
    let content = Content::builtin().expect("content");
    let spring = content.terrain_id("spring").expect("the spring's terrain");
    let (offer, table) = content.offer_at(spring).expect("a spring offers");
    assert_eq!(
        (offer, table.id.as_str()),
        (crate::content::Offer::Drink, "spring")
    );
    assert_eq!((table.seconds, table.exhaust, table.spent), (10, 0, None));
    assert_eq!(table.kinds, [60, 25, 15]);
    let fouled = table
        .outcomes
        .iter()
        .find(|o| o.id == "fouled")
        .expect("fouled");
    assert_eq!(fouled.effect.catch, content.disease_id("marsh_ague"));
    let clear = (table.outcomes.iter())
        .find(|o| o.id == "clear_headed")
        .expect("clear-headed");
    assert_eq!(clear.effect.mana, Some(crate::content::Amount::Half));
    let room = content
        .room_effects()
        .id("spring")
        .expect("the spring room");
    let def = content.room_effect(room);
    assert_eq!(
        (def.weight, def.min_cells, def.feature),
        (4, 9, Some(spring))
    );
}

#[test]
fn a_bad_spring_is_refused() {
    let head = "seconds = 10\nkinds = { bane = 1 }\nexhaust = 0\n[[outcome]]\nkind = \"bane\"\n\
                tier = \"minor\"\nid = \"x\"\nweight = 1\nmessage = \"X.\"\n";
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/outcomes/bad.toml",
        &format!("{head}effect = {{ catch = \"the_sniffles\" }}\n"),
    );
    let err = Content::load(&files).unwrap_err();
    assert!(err.message.contains("unknown disease"), "{}", err.message);
    // `offers` names one thing, not two.
    let mut files = ContentFiles::builtin();
    let effects = include_str!("../../../../assets/data/room_effects.toml").to_string();
    let both = effects.replace(
        "offers = { drink = \"spring\" }",
        "offers = { drink = \"spring\", pray = \"forgotten_altar\" }",
    );
    assert_ne!(both, effects);
    files.insert("data/room_effects.toml", &both);
    let err = Content::load(&files).unwrap_err();
    assert!(err.message.contains("names one thing"), "{}", err.message);
}
