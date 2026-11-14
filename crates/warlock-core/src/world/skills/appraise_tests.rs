//! Appraise (spec §8 "Skills"; #666): each game hour a player trained in it may tell one
//! unidentified thing they carry or wear for what it is, by a chance its value sets; Negotiation
//! has the shops' prices it once had.

use super::*;
use crate::character::CharacterSpec;
use crate::rng::Rng;
use crate::skills::Effect;
use crate::test_support::{CONTENT as BUILTIN, run, said};
use crate::time::HOUR;

/// A human of `class` in the showcase.
fn world(class: &str) -> World {
    let spec: CharacterSpec = format!("human,male,{class}").parse().expect("a character");
    World::with_character(BUILTIN.clone(), "showcase", Some(7), Some(&spec)).expect("world")
}

/// An unidentified magic long sword, nothing of its own parts.
fn magic_sword() -> Stack {
    let sword = BUILTIN.object_id("long_sword").expect("sword");
    let mut rng = Rng::stream(3, "appraise");
    (0..200)
        .map(|_| {
            crate::magic::loot::make(
                &BUILTIN,
                &mut rng,
                sword,
                1,
                5,
                crate::magic::LootGrade::Magic,
            )
        })
        .find(|s| !s.is_identified() && s.own_parts().is_none())
        .expect("a magic sword")
}

/// The player carries a magic sword they do not know; its position.
fn carry_sword(w: &mut World) -> usize {
    w.player_mut().inventory.push(magic_sword());
    w.state.player().inventory.len() - 1
}

fn skill(id: &str) -> crate::skills::SkillId {
    BUILTIN.skill_id(id).expect("skill")
}

fn known(w: &World, item: usize) -> bool {
    w.state.player().inventory[item].is_identified()
}

#[test]
fn the_skills_split_appraise_identifies_and_negotiation_bargains() {
    let (appraise, negotiation) = (
        BUILTIN.skill(skill("appraise")),
        BUILTIN.skill(skill("negotiation")),
    );
    let effects = |s: &crate::content::SkillDef| -> Vec<Effect> {
        s.benefits.iter().map(|b| b.effect).collect()
    };
    assert_eq!(effects(appraise), [Effect::Identify, Effect::Freshness]);
    assert_eq!(effects(negotiation), [Effect::Bargain]);
    // Every class trained in Appraise has Negotiation at the same dice, so prices stay as they
    // were.
    let with_appraise: Vec<&str> = (BUILTIN.classes().iter())
        .filter(|c| c.skills.iter().any(|&(id, _)| id == skill("appraise")))
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(with_appraise, ["bard", "thief", "craftsperson", "merchant"]);
    for class in BUILTIN
        .classes()
        .iter()
        .filter(|c| with_appraise.contains(&c.id.as_str()))
    {
        let dice = |id| {
            (class.skills.iter())
                .find(|&&(s, _)| s == id)
                .map(|&(_, d)| d)
        };
        assert_eq!(
            dice(skill("appraise")),
            dice(skill("negotiation")),
            "{}",
            class.id
        );
    }
}

#[test]
fn negotiation_moves_prices_and_appraise_no_longer_does() {
    let mut w = world("warrior");
    let content = BUILTIN.clone();
    let shops = content.shops().expect("shops");
    let base = shop::favour(&content, shops, w.state.player());
    w.player_mut().skills.add(skill("appraise"), 100);
    assert_eq!(shop::favour(&content, shops, w.state.player()), base);
    w.player_mut().skills.add(skill("negotiation"), 100);
    assert!(shop::favour(&content, shops, w.state.player()) > base);
}

#[test]
fn an_appraiser_tells_a_carried_thing_for_what_it_is_in_time() {
    let mut w = world("merchant");
    let item = carry_sword(&mut w);
    let before = w.lore().describe(&w.state.player().inventory[item]);
    w.player_mut().skills.add(skill("appraise"), 200);
    let marks = |w: &World| {
        w.state
            .player()
            .skills
            .get(skill("appraise"))
            .expect("trained")
            .marks
    };
    let had = marks(&w);
    let now = w.state.time;
    w.appraise_hours(now, 200 * HOUR);
    assert!(known(&w, item));
    let after = w.lore().describe(&w.state.player().inventory[item]);
    let line = format!(
        "You realise your {} is {after}.",
        before.trim_start_matches("a ")
    );
    assert!(said(&w).contains(&line), "{line} in {:?}", said(&w));
    assert!(!said(&w).iter().any(|m| m.starts_with("This is ")));
    // Each thing told trains the skill (a mark, certain by the data).
    assert_eq!(marks(&w), had + 1);
}

#[test]
fn the_untrained_never_appraise() {
    let mut w = world("warrior");
    assert!(w.state.player().skills.get(skill("appraise")).is_none());
    let item = carry_sword(&mut w);
    let now = w.state.time;
    w.appraise_hours(now, 2000 * HOUR);
    assert!(!known(&w, item));
    let (hash, time) = (w.state_hash(), w.state.time);
    w.appraise_hours(time, 2000 * HOUR);
    assert_eq!(w.state_hash(), hash);
}

#[test]
fn the_checks_come_on_the_hour_however_the_time_is_cut() {
    let whole = |pieces: &[u64]| {
        let mut w = world("merchant");
        let item = carry_sword(&mut w);
        let mut at = w.state.time;
        for &seconds in pieces {
            w.appraise_hours(at, seconds);
            at += crate::time::Duration::seconds(seconds);
        }
        (known(&w, item), said(&w))
    };
    let day = 300 * HOUR;
    let one = whole(&[day]);
    assert_eq!(one, whole(&[HOUR / 2, day - HOUR / 2]));
    assert_eq!(
        one,
        whole(&[7 * HOUR + 13, 100 * HOUR, day - 107 * HOUR - 13])
    );
    // Less than an hour from a time past the hour crosses no hour: nothing is checked.
    let mut w = world("merchant");
    carry_sword(&mut w);
    w.player_mut().skills.add(skill("appraise"), 999);
    let start = crate::time::GameTime::from_seconds(10 * HOUR + 1);
    let hash = w.state_hash();
    w.appraise_hours(start, HOUR - 2);
    assert_eq!(w.state_hash(), hash);
}

#[test]
fn rest_and_the_dev_skip_roll_the_hours_they_pass() {
    for actions in ["skip:24h*10", "rest:24h*10"] {
        let mut w = world("merchant");
        w.player_mut().skills.add(skill("appraise"), 999);
        let item = carry_sword(&mut w);
        run(&mut w, actions);
        assert!(known(&w, item), "{actions}: {:?}", said(&w));
    }
}

#[test]
fn an_appraisal_replays() {
    let mut w = world("merchant");
    w.player_mut().skills.add(skill("appraise"), 999);
    carry_sword(&mut w);
    let mut again = w.clone();
    run(&mut w, "skip:10h wait rest:2h");
    run(&mut again, "skip:10h wait rest:2h");
    assert_eq!(w.state_hash(), again.state_hash());
    let json = serde_json::to_string(&w.state).expect("saved");
    assert!(!json.contains("appraise/"), "no state of its own");
}

/// The player carries a flask of fire whose kind they do not know; its position.
fn carry_flask(w: &mut World) -> usize {
    let flask = BUILTIN.object_id("flask_of_fire").expect("flask");
    assert!(!w.lore().is_identified(flask), "unknown to begin with");
    w.player_mut().inventory.push(Stack::new(flask, 1));
    w.state.player().inventory.len() - 1
}

/// Skills the warrior has none of: whatever they are trained in now, less `ids`.
fn untrain(w: &mut World, ids: &[&str]) {
    let gone: Vec<_> = ids.iter().map(|id| skill(id)).collect();
    let kept: crate::skills::Skills = (w.state.player().skills.iter())
        .filter(|(s, _)| !gone.contains(s))
        .collect();
    w.player_mut().skills = kept;
}

#[test]
fn an_alchemist_tells_potions_and_only_potions() {
    let mut w = world("warrior");
    untrain(&mut w, &["appraise", "alchemy", "enchanting"]);
    let (sword, flask) = (carry_sword(&mut w), carry_flask(&mut w));
    let object = w.state.player().inventory[flask].object;
    let before = w.lore().describe(&w.state.player().inventory[flask]);
    w.player_mut().skills.add(skill("alchemy"), 200);
    assert_eq!(
        w.state
            .player()
            .skill_effect(&BUILTIN, Effect::IdentifyPotion),
        40
    );
    let now = w.state.time;
    w.appraise_hours(now, 200 * HOUR);
    assert!(w.lore().is_identified(object), "{:?}", said(&w));
    assert!(!known(&w, sword), "a sword is no potion");
    let line = format!(
        "You recognise your {}: a flask of fire.",
        before.trim_start_matches("a ")
    );
    assert!(said(&w).contains(&line), "{line} in {:?}", said(&w));
    // No marks until brewing comes.
    let marks = w
        .state
        .player()
        .skills
        .get(skill("alchemy"))
        .map(|t| t.marks);
    assert_eq!(marks, Some(0));
}

#[test]
fn an_enchanter_tells_magic_and_not_potions() {
    let mut w = world("warrior");
    untrain(&mut w, &["appraise", "alchemy", "enchanting"]);
    let (sword, flask) = (carry_sword(&mut w), carry_flask(&mut w));
    let object = w.state.player().inventory[flask].object;
    w.player_mut().skills.add(skill("enchanting"), 200);
    let now = w.state.time;
    w.appraise_hours(now, 200 * HOUR);
    assert!(known(&w, sword), "{:?}", said(&w));
    assert!(!w.lore().is_identified(object), "a flask is no magic thing");
    assert!(
        said(&w)
            .iter()
            .any(|m| m.starts_with("You read the magic bound in your long sword")),
        "{:?}",
        said(&w)
    );
}

#[test]
fn untrained_nobody_tells_a_potion_or_magic() {
    // Without Appraise, Alchemy, or Enchanting the hours draw nothing and tell nothing, with
    // the professions' unknown things carried.
    let mut w = world("warrior");
    untrain(&mut w, &["appraise", "alchemy", "enchanting"]);
    carry_flask(&mut w);
    carry_sword(&mut w);
    let hash = w.state_hash();
    let now = w.state.time;
    w.appraise_hours(now, 50 * HOUR);
    assert_eq!(w.state_hash(), hash, "nothing trained, nothing told");
}

/// An item's capacity (docs/dev/enchanting.md) is told only to one trained in Appraise, and the
/// count of an unidentified item's affixes never.
#[test]
fn the_capacity_line_is_told_only_to_one_trained_in_appraise() {
    let mut w = world("warrior");
    let item = carry_sword(&mut w);
    let plain = BUILTIN.object_id("dagger").expect("dagger");
    w.player_mut().inventory.push(Stack::one(plain));
    let dagger = w.state.player().inventory.len() - 1;
    let lines = |w: &World, at: usize| -> Vec<String> {
        let stack = &w.state.player().inventory[at];
        (w.perception().describe_item(stack).into_iter())
            .map(|l| l.text)
            .collect()
    };
    let told = |lines: &[String]| lines.iter().any(|l| l.contains("bear"));
    w.player_mut().skills = Default::default();
    assert!(!told(&lines(&w, item)) && !told(&lines(&w, dagger)));
    w.player_mut().skills.add(skill("appraise"), 1);
    let sword = lines(&w, item);
    assert!(
        sword.contains(&"It can bear two enchantments.".to_string()),
        "{sword:?}"
    );
    assert!(lines(&w, dagger).contains(&"It can bear two enchantments.".to_string()));
    w.player_mut().inventory[item].identify();
    let known = lines(&w, item);
    let bears = match w.state.player().inventory[item]
        .known_magic()
        .map(|m| m.affixes.len())
    {
        Some(2) => "It bears all it can.",
        _ => "It bears one enchantment and could bear one more.",
    };
    assert!(known.contains(&bears.to_string()), "{known:?}");
}
