//! How the player's attack found its target, told in the attack's own words (spec §8 "Surprise
//! and sneak attacks"; `surprise.rs::caught_by`, `caught_name`): asleep, unaware and surprised
//! by it, already Surprised or Stunned, the blow after the surprise, a shot and a spell; nothing
//! for an aware target or one the player does not see, and one line an attack.

use super::cast_tests::cast_at;
use super::caster_tests::{kobold, reach, world_with};
use super::surprise_tests::{Rules, goblin, lull, surprised_status, texts, world};
use super::*;
use crate::action::{AttackKind, Caught};
use crate::content::ContentFiles;

/// What the player is told when they step east into the goblin beside them: their attack's
/// lines and whatever follows in that turn.
fn strike(w: &mut World) -> Vec<String> {
    let before = w.messages().len();
    w.apply(&Action::Move(Dir::E));
    texts(w)[before..].to_vec()
}

/// The lines of `said` that name the goblin as `word`: "the sleeping goblin".
fn naming<'a>(said: &'a [String], word: &str) -> Vec<&'a String> {
    let phrase = format!("{word} goblin");
    said.iter().filter(|t| t.contains(&phrase)).collect()
}

/// Whether any line of `said` tells how the attack found the goblin.
fn told(said: &[String]) -> bool {
    ["sleeping", "unsuspecting", "surprised", "stunned"]
        .iter()
        .any(|word| !naming(said, word).is_empty())
        || said.iter().any(|t| t.contains("unawares"))
}

/// How the player's attack in `events` found its target.
fn caught(events: &[Event]) -> Option<Caught> {
    events.iter().find_map(|e| match e {
        Event::Attacked { caught, .. } => *caught,
        _ => None,
    })
}

#[test]
fn a_blow_on_a_sleeper_says_so_once() {
    let mut w = world(120, Rules::Difficulty(1));
    let id = goblin(&w, 2);
    lull(&mut w, id);
    let said = strike(&mut w);
    assert_eq!(naming(&said, "sleeping").len(), 1, "{said:?}");
    assert!(said.iter().all(|t| !t.contains("unawares")), "{said:?}");
    // Its waking is said after the blow's line.
    let woke = said.iter().position(|t| t == "The goblin wakes up.");
    assert!(woke.is_some_and(|at| at > 0), "{said:?}");
    assert!(said[0].contains("sleeping goblin"), "{said:?}");
}

#[test]
fn a_blow_that_catches_the_awake_unawares_says_so_in_its_own_line() {
    // Lying in wait (the thief's Lurk), the goblin not hunting them: it fails its check.
    let mut w = world(120, Rules::Difficulty(40));
    let id = goblin(&w, 2);
    w.technique.lurk = Some(id);
    let said = strike(&mut w);
    assert_eq!(naming(&said, "unsuspecting").len(), 1, "{said:?}");
    assert!(
        said.iter().all(|t| !t.contains("unawares")),
        "one telling: {said:?}"
    );
    // It makes its check: it whirls to face them, and the blow says nothing more.
    let mut w = world(120, Rules::Difficulty(1));
    let id = goblin(&w, 2);
    w.technique.lurk = Some(id);
    let said = strike(&mut w);
    assert_eq!(
        said.first().map(String::as_str),
        Some("The goblin whirls to face you.")
    );
    assert!(naming(&said, "unsuspecting").is_empty(), "{said:?}");
}

#[test]
fn a_blow_on_the_surprised_or_stunned_says_so() {
    for (status, word) in [("surprised", "surprised"), ("stunned", "stunned")] {
        let mut w = world(120, Rules::Difficulty(1));
        let id = goblin(&w, 2);
        let status = w.content.status_id(status).expect("the status");
        w.afflict(id, status, 1, None, &mut Vec::new());
        let said = strike(&mut w);
        assert_eq!(naming(&said, word).len(), 1, "{said:?}");
    }
}

#[test]
fn the_blow_after_the_surprise_finds_it_surprised() {
    let mut w = world(120, Rules::Difficulty(40));
    let (player, id) = (w.state.player, goblin(&w, 2));
    w.state.actors.get_mut(&id).expect("goblin").hp = 10_000;
    w.technique.lurk = Some(id);
    let blow = |w: &mut World| {
        let profile = w.state.player().melee(&w.content);
        let mut events = Vec::new();
        w.attack_with(player, id, AttackKind::Melee, Some(profile), &mut events);
        caught(&events)
    };
    assert_eq!(blow(&mut w), Some(Caught::Unaware));
    let surprised = surprised_status(&w);
    assert_eq!(blow(&mut w), Some(Caught::OffGuard(surprised)));
}

#[test]
fn a_shot_from_out_of_sight_finds_it_unaware_and_a_creatures_attack_is_not_marked() {
    // Behind the wall the goblin has not seen the player: the shot catches it unawares.
    let mut w = world(120, Rules::Difficulty(40));
    let (player, far) = (w.state.player, goblin(&w, 8));
    w.state.actors.get_mut(&far).expect("goblin").hp = 10_000;
    let profile = w.state.player().melee(&w.content);
    let mut events = Vec::new();
    w.attack_with(player, far, AttackKind::Ranged, Some(profile), &mut events);
    assert_eq!(caught(&events), Some(Caught::Unaware));
    // The goblin's own attack on the player is never marked.
    let near = goblin(&w, 2);
    lull(&mut w, near);
    w.state.actors.get_mut(&player).expect("player").hp = 10_000;
    let profile = w.state.actors[&near].melee(&w.content);
    let mut events = Vec::new();
    w.attack_with(near, player, AttackKind::Melee, Some(profile), &mut events);
    assert_eq!(caught(&events), None);
}

#[test]
fn nothing_is_told_of_an_aware_target_or_one_unseen() {
    let mut w = world(120, Rules::Difficulty(1));
    let said = strike(&mut w);
    assert!(!told(&said), "aware: {said:?}");
    // In the dark the player sees nothing of the sleeper they strike.
    let mut w = world(0, Rules::Difficulty(1));
    let id = goblin(&w, 2);
    lull(&mut w, id);
    let said = strike(&mut w);
    assert!(!told(&said), "unseen: {said:?}");
}

#[test]
fn a_spell_that_rolls_to_hit_says_so_too() {
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/spells/arcane_bolt.toml",
        "[[spell]]\nid = \"arcane_bolt\"\nname = \"Arcane bolt\"\ndescription = \"x\"\n\
         level = 6\nschool = \"evocation\"\nkind = \"damage\"\ndamage_type = \"force\"\n\
         shape = \"ray\"\nresolution = \"attack\"\nrange = 10\ndamage = \"3d6\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("the spell loads"));
    let mut w = world_with(content, "wizard");
    reach(&mut w, 18);
    let target = kobold(&mut w, 3, 10_000);
    lull(&mut w, target);
    let before = w.messages().len();
    cast_at(&mut w, (3, 0), (target, (3, 0)));
    let said = texts(&w)[before..].to_vec();
    let sleeping: Vec<_> = said
        .iter()
        .filter(|t| t.contains("sleeping kobold"))
        .collect();
    assert_eq!(sleeping.len(), 1, "{said:?}");
}
