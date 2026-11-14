//! The aristocrat's stipend and servant (#1405; `world/stipend.rs`): the class ability from
//! level 1, the servant in Cēaptūn only in an aristocrat's run (by gender and race), the
//! stipend's growth with every level, its rate of finds in the owner's band, Landed noble's
//! larger stipend, the handover (to the pack, or at the feet), a save, and a replay.

use super::*;
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::replay::Replay;
use crate::talent::TalentMods;
use crate::test_support::CONTENT;
use crate::test_support::run as act;
use crate::test_support::texts as said;

/// Beside the servant's cell (40, 27) in the Gæsthūs, east of it.
const BESIDE: Pos = Pos::new(41, 27);

/// A run in Cēaptūn as `character` with the creation picks `talents`, the rest filled with
/// the first offered of another tree.
fn run(character: &str, talents: &[&str]) -> World {
    let spec: CharacterSpec = character.parse().expect("a character");
    let book = CONTENT.talents();
    let heritage = book.tree_id("heritage").expect("heritage");
    let draft = World::draft(CONTENT.clone(), "ceaptun", None, &spec).expect("a draft");
    let mut picks: Vec<String> = talents.iter().map(|t| t.to_string()).collect();
    loop {
        let (left, offers) = draft.creation_offers(&picks).expect("the picks hold");
        if left == 0 {
            break;
        }
        let (id, ways) = (offers.iter())
            .find(|(id, _)| book.get(*id).tree != heritage)
            .expect("a talent to pick");
        picks.push(crate::world::pick_text(
            &book.get(*id).id,
            ways[0].as_deref(),
        ));
    }
    let spec = spec.with_talents(picks);
    World::start_run(CONTENT.clone(), "ceaptun", None, Some(&spec), None, true)
        .expect("a run in Cēaptūn")
}

/// Whether a new `character` is offered Landed noble.
fn offered_landed(character: &str) -> bool {
    let spec: CharacterSpec = character.parse().expect("a character");
    let draft = World::draft(CONTENT.clone(), "ceaptun", None, &spec).expect("a draft");
    let (_, offers) = draft.creation_offers(&[]).expect("offers");
    (offers.iter()).any(|(id, _)| CONTENT.talents().get(*id).id == "landed_noble")
}

fn servant(world: &World) -> Option<&Actor> {
    let content = world.content();
    (world.state.actors.values()).find(|a| {
        content
            .dialogue_of(a.creature, a.npc.as_deref())
            .is_some_and(|d| content.dialogue(d).stipend.is_some())
    })
}

fn put_player(world: &mut World, at: Pos) {
    let id = world.state.player;
    world.state.actors.get_mut(&id).expect("player").pos = at;
    world.perceive(&mut Vec::new());
}

fn gold(world: &World) -> u32 {
    let coin = world.content().object_id("gold_coin").expect("gold");
    super::shop::count_of(&world.state().player().inventory, coin)
}

#[test]
fn an_aristocrat_has_the_stipend_from_level_one_and_no_one_else() {
    let w = run("human,male,aristocrat", &[]);
    assert!(w.state.player().talents.has("stipend"));
    let w = run("human,male,warrior", &[]);
    assert!(!w.state.player().talents.has("stipend"));
    assert!(servant(&w).is_none(), "no servant in a warrior's run");
}

#[test]
fn the_servant_is_named_by_gender_and_speaks_by_race() {
    for (character, creature, npc) in [
        ("human,male,aristocrat", "squire", "servant"),
        ("dwarf,female,aristocrat", "chambermaid", "servant_dwarf"),
        ("half_elf,other,aristocrat", "servant", "servant"),
        ("goblin,male,aristocrat", "squire", "servant_goblin"),
        (
            "lizardfolk,female,aristocrat",
            "chambermaid",
            "servant_lizardfolk",
        ),
    ] {
        let w = run(character, &[]);
        let s = servant(&w).unwrap_or_else(|| panic!("{character}: a servant"));
        assert_eq!(w.content().creature(s.creature).id, creature, "{character}");
        assert_eq!(s.npc.as_deref(), Some(npc), "{character}");
        assert_eq!(s.pos, Pos::new(40, 27));
        assert_eq!(w.called(s), format!("your {creature}"));
    }
}

#[test]
fn every_level_grows_the_stipend_and_the_servant_hands_it_over() {
    let mut w = run("human,female,aristocrat", &[]);
    assert!(w.state.stipend.is_empty());
    act(&mut w, "xp:5000 levelup:auto");
    let level = w.player_level();
    assert!(level > 3, "level {level}");
    let put_by = w.state.stipend.gold;
    // About 200 a level: 150 + 10d10 each.
    let levels = u32::from(level - 1);
    assert!(
        (160 * levels..=250 * levels).contains(&put_by),
        "{put_by} for {levels}"
    );
    assert!(
        said(&w)
            .iter()
            .any(|m| m.contains("your stipend has grown"))
    );
    let items = w.state.stipend.items.len();
    let had = gold(&w);
    let pack = w.state.player().inventory.len();
    put_player(&mut w, BESIDE);
    act(&mut w, "chat:W");
    assert!(w.state.stipend.is_empty());
    assert_eq!(gold(&w), had + put_by);
    assert!(w.state.player().inventory.len() >= pack + items.min(1).saturating_sub(1));
    let words = said(&w);
    assert!(
        words.iter().any(
            |m| m.starts_with("Your chambermaid says: \"Yer family's sent") && m.contains("m'lady")
        ),
        "{words:?}"
    );
    assert!(
        words.iter().any(|m| m.starts_with("You receive ")),
        "{words:?}"
    );
    // Nothing put by: a one-liner, addressed.
    act(&mut w, "chat:W");
    let last = said(&w).last().cloned().unwrap_or_default();
    assert!(
        last.starts_with("Your chambermaid says:") && !last.contains('{'),
        "{last}"
    );
}

#[test]
fn what_would_overload_the_player_is_set_down_at_their_feet() {
    let mut w = run("human,male,aristocrat", &[]);
    // Fifty levels' gold weighs far more than anyone carries.
    act(&mut w, "xp:4000000 levelup:auto");
    put_player(&mut w, BESIDE);
    act(&mut w, "chat:W");
    let coin = w.content().object_id("gold_coin").expect("gold");
    let here = w.state.current().objects_at(BESIDE);
    assert!(
        here.iter().any(|s| s.object == coin),
        "the gold lies at their feet"
    );
    assert!(said(&w).iter().any(|m| m.starts_with("Too much to carry")));
    assert!(w.state.stipend.is_empty());
}

/// The share of level-ups that put by a find, over `seeds` runs and the levels 2 to 60.
fn find_rate(content: &Content, mods: &TalentMods, seeds: u64) -> f64 {
    let (mut found, mut all) = (0u32, 0u32);
    for seed in 0..seeds {
        for level in 2..=60 {
            let (_, find) = stipend::of_level(content, seed, level, mods).expect("rules");
            found += u32::from(find.is_some());
            all += 1;
        }
    }
    f64::from(found) / f64::from(all)
}

#[test]
fn a_third_to_a_half_of_level_ups_bring_a_find() {
    let content = &**CONTENT;
    let mods = TalentMods {
        fortune: crate::talent::FortuneMods {
            stipend: true,
            ..Default::default()
        },
        ..TalentMods::default()
    };
    let rate = find_rate(content, &mods, 40);
    println!("stipend finds: {:.1} % of level-ups", rate * 100.0);
    assert!((0.30..=0.50).contains(&rate), "{rate}");
}

#[test]
fn a_landed_noble_has_twice_the_gold_and_more_finds() {
    let content = &**CONTENT;
    let plain = TalentMods {
        fortune: crate::talent::FortuneMods {
            stipend: true,
            ..Default::default()
        },
        ..TalentMods::default()
    };
    let w = run("human,male,aristocrat", &["landed_noble"]);
    let landed = w.state.player().talent_mods(content).into_owned();
    assert!(landed.fortune.stipend && landed.fortune.stipend_gold_percent == 100);
    let (a, _) = stipend::of_level(content, 7, 10, &plain).expect("rules");
    let (b, _) = stipend::of_level(content, 7, 10, &landed).expect("rules");
    assert_eq!(b, a * 2);
    let (rate, more) = (
        find_rate(content, &plain, 20),
        find_rate(content, &landed, 20),
    );
    println!(
        "stipend finds: {:.1} %, landed noble {:.1} %",
        rate * 100.0,
        more * 100.0
    );
    assert!(more > rate, "{more} > {rate}");
    // Only an aristocrat is offered it.
    assert!(offered_landed("human,male,aristocrat"));
    assert!(!offered_landed("human,male,warrior"));
}

#[test]
fn the_stipend_survives_a_save_and_replays() {
    let mut w = run("human,male,aristocrat", &[]);
    let mut replay = Replay::new(&w);
    let steps = "xp:5000 levelup:auto move:SW move:S*3 open:S move:S*2 move:SW*4";
    for action in parse_actions(steps).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let json = serde_json::to_string(&w.state).expect("saves");
    let state: WorldState = serde_json::from_str(&json).expect("loads");
    let loaded = World::from_state(w.content.clone(), state).expect("restores");
    assert!(!loaded.state.stipend.is_empty());
    assert_eq!(loaded.state.stipend, w.state.stipend);
    assert_eq!(loaded.state_hash(), w.state_hash());
    assert_eq!(w.player_pos(), BESIDE);
    for action in parse_actions("chat:W").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(w.state.stipend.is_empty());
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(w.content.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
