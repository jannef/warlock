//! The drow (#519): their stance toward the others, and how they fight: venomed bolts and
//! blades, priestesses calling demons, gear worn and none wielded.

use super::Content;
use crate::test_support::builtin;

/// Every drow type.
const DROW: &[&str] = &[
    "drow_warrior",
    "drow_crossbowman",
    "drow_stalker",
    "drow_mage",
    "drow_priestess",
    "drow_blademaster",
    "drow_assassin",
    "drow_arbalester",
    "drow_captain",
    "drow_archmage",
    "drow_high_priestess",
    "drow_matron",
];

#[test]
fn the_drow_hunt_the_player_and_leave_their_old_rivals_be() {
    use super::Stance;
    let content = builtin();
    let stance = |a: &str, b: &str| {
        let id = |n: &str| content.creature_id(n).expect(n);
        content.stance(id(a), id(b))
    };
    for (a, b, want) in [
        ("drow_warrior", "player", Stance::Hostile),
        ("drow_warrior", "kobold", Stance::Neutral),
        ("drow_crossbowman", "goblin", Stance::Neutral),
        ("drow_captain", "hobgoblin", Stance::Neutral),
        ("drow_blademaster", "orc_warrior", Stance::Neutral),
        ("drow_stalker", "duergar_warrior", Stance::Neutral),
        ("drow_priestess", "dretch", Stance::Neutral),
        ("drow_matron", "balor", Stance::Neutral),
        ("drow_warrior", "barrow_spider", Stance::Neutral),
        ("drow_mage", "skeleton", Stance::Neutral),
        ("drow_assassin", "fire_salamander", Stance::Neutral),
        ("drow_warrior", "drow_matron", Stance::Neutral),
    ] {
        assert_eq!(stance(a, b), want, "{a} and {b}");
    }
}

/// A lit corridor in the shared content with the player at `@` and the drow of `legend` on
/// its letters; the player is hard to kill.
fn range(map: &str, legend: &str) -> crate::world::World {
    let mut files = super::ContentFiles::builtin();
    files.insert(
        "campaigns/showcase/levels/drow_range.toml",
        &format!(
            "name = \"Range\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n[legend]\n\"@\" \
             = {{ terrain = \"floor\", spawn = \"player\" }}\n{legend}\n"
        ),
    );
    files.insert(
        "campaigns/showcase/scenarios/drow_range.toml",
        "name = \"Range\"\nstart_level = \"drow_range\"\nhp = 1000\n",
    );
    let content = std::sync::Arc::new(Content::load(&files).expect("content"));
    crate::world::World::new(content, "drow_range", Some(7)).expect("world")
}

#[test]
fn a_crossbowman_shoots_venomed_bolts_and_a_warrior_cuts_with_venom() {
    use crate::action::parse_actions;
    // From afar the crossbowman shoots poisoned bolts, which may send the player to sleep.
    let mut w = range(
        "##########\n#@.....c.#\n##########\n",
        "\"c\" = { terrain = \"floor\", creature = \"drow_crossbowman\" }",
    );
    for action in parse_actions("wait*40").expect("actions") {
        w.apply(&action);
    }
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    assert!(
        said.iter()
            .any(|m| m == "The drow crossbowman shoots a poisoned bolt."),
        "{said:?}"
    );
    let effects = |w: &crate::world::World, status: &str| {
        let id = w.content().status_id(status).expect(status);
        w.state().player().effects.has(id)
    };
    let said_any = |said: &[String], what: &str| said.iter().any(|m| m == what);
    assert!(
        said_any(&said, "You feel terribly drowsy.") || said_any(&said, "You feel sick."),
        "{said:?}"
    );
    // Up close, the warrior's envenomed sword poisons.
    let mut w = range(
        "######\n#@w..#\n######\n",
        "\"w\" = { terrain = \"floor\", creature = \"drow_warrior\" }",
    );
    for action in parse_actions("wait*30").expect("actions") {
        w.apply(&action);
    }
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    assert!(
        said_any(&said, "You feel sick.") || effects(&w, "poisoned"),
        "{said:?}"
    );
}

#[test]
fn a_priestess_calls_forth_demons_that_stand_with_the_drow() {
    use crate::action::parse_actions;
    let mut w = range(
        "##########\n#@.....p.#\n##########\n",
        "\"p\" = { terrain = \"floor\", creature = \"drow_priestess\" }",
    );
    for action in parse_actions("wait*10").expect("actions") {
        w.apply(&action);
    }
    let drow = w.content().factions().id("drow");
    let priestess = w.content().creature_id("drow_priestess").expect("type");
    let by = (w.state().actors.values())
        .find(|a| a.creature == priestess)
        .expect("the priestess")
        .id;
    let called: Vec<_> = (w.state().actors.values())
        .filter(|a| a.summoner == Some(by))
        .collect();
    assert!(!called.is_empty(), "she summoned");
    assert!(called.len() <= 4, "{} demons", called.len());
    for a in &called {
        let def = w.content().creature(a.creature);
        assert!(def.tags.iter().any(|t| t == "demon"), "{}", def.id);
        assert_eq!(a.faction(w.content()), drow, "{} joins the drow", def.id);
    }
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    assert!(said.iter().any(|m| m.contains("calls forth")), "{said:?}");
}

#[test]
fn the_drow_wear_their_gear_and_wield_none() {
    use crate::gear;
    use crate::rng::Rng;
    let content = builtin();
    for &id in DROW {
        let creature = content.creature_id(id).expect(id);
        for seed in 0..200 {
            let pack = gear::roll(&content, &mut Rng::gear(seed, "test"), creature, 30);
            for (i, stack) in pack.items.iter().enumerate() {
                let object = content.object(stack.object);
                assert!(
                    object.attack.is_none() && object.launches.is_none(),
                    "{id} {seed}: armed with {}",
                    object.id
                );
                if object.slot.is_some() {
                    assert!(
                        pack.equipment.holds(i),
                        "{id} {seed}: {} not worn",
                        object.id
                    );
                }
            }
        }
    }
}
