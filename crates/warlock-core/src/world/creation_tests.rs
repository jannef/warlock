//! Creation picks (spec §8 "Talents"): a character the player chose takes its talents at
//! creation, named or at random, and records them; nothing is said and nothing waits.

use super::*;
use crate::character::CharacterSpec;
use crate::replay::Replay;
use std::sync::LazyLock;

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));

fn spec(text: &str) -> CharacterSpec {
    text.parse().expect("a character")
}

/// A kitted run of the showcase as `character` with `talents` named (none: Random).
fn start(character: &str, talents: &[&str], seed: u64) -> Result<World, WorldError> {
    let spec = spec(character).with_talents(talents.iter().map(|t| t.to_string()).collect());
    World::start_run(
        CONTENT.clone(),
        "showcase",
        Some(seed),
        Some(&spec),
        None,
        true,
    )
}

/// The talents picked, not given (the class's and the sign's are granted).
fn picked(world: &World) -> Vec<String> {
    let book = world.content().talents();
    (world.state().player().talents.iter())
        .filter(|t| book.id(&t.id).is_some_and(|id| !book.get(id).granted))
        .map(|t| crate::world::pick_text(&t.id, t.choice.as_deref()))
        .collect()
}

#[test]
fn a_chosen_character_takes_its_random_picks_and_records_them() {
    let world = start("dwarf,female,warrior", &[], 3).expect("starts");
    let named = &world.character().expect("a character").spec.talents;
    assert_eq!(named.len(), 2, "{named:?}");
    assert_eq!(&picked(&world), named);
    assert_eq!(world.due(), None, "nothing waits");
    assert!(
        !(world.messages().iter()).any(|m| m.text.starts_with("You learn")),
        "nothing is said"
    );
    // The same seed and character pick the same; the draft's Random row says what they are.
    let again = start("dwarf,female,warrior", &[], 3).expect("starts");
    assert_eq!(picked(&again), *named);
    let draft = World::draft(
        CONTENT.clone(),
        "showcase",
        Some(3),
        &spec("dwarf,female,warrior"),
    )
    .expect("a draft");
    assert_eq!(draft.random_creation(), *named);
    // Another seed may pick otherwise, but always two of the talents they may take.
    for seed in 0..20 {
        let world = start("elf,male,wizard", &[], seed).expect("starts");
        assert_eq!(picked(&world).len(), 2, "seed {seed}");
    }
}

#[test]
fn the_bard_picks_one_more_and_the_scenarios_own_character_none() {
    let bard = start("human,other,bard", &[], 1).expect("starts");
    assert_eq!(picked(&bard).len(), 3);
    assert_eq!(creation_count(bard.content(), &spec("human,other,bard")), 3);
    // A dev launch on the scenario's own character, without the kit, takes none.
    let plain = World::new(CONTENT.clone(), "showcase", None).expect("starts");
    assert!(picked(&plain).is_empty());
    assert!(
        plain
            .character()
            .expect("the showcase's")
            .spec
            .talents
            .is_empty()
    );
}

#[test]
fn named_picks_are_taken_in_order_and_checked() {
    let world = start("dwarf,male,warrior", &["healthy", "keen_ears"], 1).expect("starts");
    assert_eq!(picked(&world), ["healthy", "keen_ears"]);
    // A chain within the picks: Fast healer needs Healthy (and Toughness 13, a dwarf's).
    let chain = start("dwarf,male,warrior", &["healthy", "fast_healer"], 1).expect("a chain");
    assert_eq!(picked(&chain), ["healthy", "fast_healer"]);
    let Err(WorldError::Character(why)) = start("dwarf,male,warrior", &["fast_healer"], 1) else {
        panic!("Fast healer without Healthy is refused");
    };
    assert!(why.contains("fast_healer"), "{why}");
    let Err(WorldError::Character(why)) = start("dwarf,male,warrior", &["healthy"], 1) else {
        panic!("too few");
    };
    assert!(why.contains("1 more"), "{why}");
    assert!(start("dwarf,male,warrior", &["healthy", "lean", "keen_ears"], 1).is_err());
    assert!(start("dwarf,male,warrior", &["healthy", "nonsense"], 1).is_err());
    // A choice is written after a colon.
    let focus = start(
        "human,male,warrior",
        &["weapon_focus:long_blades", "lean"],
        1,
    );
    assert_eq!(
        picked(&focus.expect("a choice"))[0],
        "weapon_focus:long_blades"
    );
}

#[test]
fn a_draft_offers_what_the_next_pick_may_be() {
    let draft = World::draft(
        CONTENT.clone(),
        "showcase",
        Some(1),
        &spec("dwarf,male,warrior"),
    )
    .expect("a draft");
    let book = draft.content().talents();
    let id = |name: &str| book.id(name).expect("a talent");
    let (left, offers) = draft.creation_offers(&[]).expect("offers");
    assert_eq!(left, 2);
    assert!(offers.iter().any(|(t, _)| *t == id("healthy")));
    assert!(!offers.iter().any(|(t, _)| *t == id("fast_healer")));
    let (left, offers) = draft.creation_offers(&["healthy".into()]).expect("offers");
    assert_eq!(left, 1);
    // Healthy's rank is held: it is offered no more; what needed it may be.
    assert!(!offers.iter().any(|(t, _)| *t == id("healthy")));
    let (left, offers) =
        (draft.creation_offers(&["healthy".into(), "lean".into()])).expect("offers");
    assert_eq!((left, offers.len()), (0, 0));
    assert!(draft.creation_offers(&["fast_healer".into()]).is_err());
}

#[test]
fn replays_and_saves_carry_the_picks() {
    let world = start("halfling,female,thief", &[], 5).expect("starts");
    let replay = Replay::new(&world);
    let text = replay.to_json();
    assert!(text.contains("\"talents\""), "{text}");
    let back = Replay::from_json(&text).expect("parses");
    let again = back.start(CONTENT.clone()).expect("starts");
    assert_eq!(again.state_hash(), world.state_hash());
    // An old replay of a run without picks (no `talents`) starts without them.
    let old = spec("halfling,female,thief");
    let json = serde_json::to_string(&old).expect("json");
    assert!(!json.contains("talents"), "{json}");
}
