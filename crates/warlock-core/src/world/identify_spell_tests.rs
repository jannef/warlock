//! Identify, the spell (spec §8 "Spells", "Scrolls"; #592): the scroll of identify casts it and
//! teaches it; cast for mana it asks for a stack the caster carries (`use:identify:c`), and its
//! cost is its own, a large one, refused without the mana, no time taken.

use super::*;
use crate::ability::{Ability, Aim};
use crate::action::parse_actions;
use crate::content::{ContentFiles, Rarity, SpellId};
use crate::magic::{Affix, Magic};

/// The pack: a, two scrolls of identify.
const PACK: &str = "carry = [\"scroll_of_identify*2\"]";

/// A lit hall, the player with `mana` mana.
fn world(mana: u16) -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/hall.toml",
        "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n#####\n#@..#\n#####\n'''\n\
         [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
    );
    files.insert(
        "scenarios/hall.toml",
        &format!("name = \"Hall\"\nstart_level = \"hall\"\nhp = 500\nmana = {mana}\n{PACK}\n"),
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let mut w = World::new(content, "hall", Some(3)).expect("world");
    let identify = w.content.object_id("scroll_of_identify").expect("identify");
    w.state.known_scrolls.insert(identify);
    w
}

fn run(w: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|a| w.apply(a))
        .collect()
}

fn last(w: &World) -> String {
    (w.messages().iter().last()).map_or_else(String::new, |m| m.display())
}

/// An unidentified magic ring, put at the end of the pack; its position.
fn ring(w: &mut World) -> usize {
    let id = w.content.object_id("silver_ring").expect("ring");
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 1,
        affixes: vec![Affix {
            id: "of_the_mule".into(),
            tier: 1,
            value: 1,
        }],
        name: None,
        identified: false,
        carried: 0,
        unique: None,
    };
    let pack = &mut w.player_mut().inventory;
    pack.push(Stack::with_magic(id, 1, magic));
    pack.len() - 1
}

fn spell(w: &World) -> SpellId {
    w.content.spell_id("identify").expect("Identify")
}

#[test]
fn identify_costs_its_own_large_mana_whatever_its_level() {
    let w = world(100);
    let content = &w.content;
    let rules = content.spell_rules().expect("the cost table");
    let def = content.spell(spell(&w));
    assert_eq!((def.level, def.mana), (6, Some(60)));
    // Neither its level nor the shape changes it; armour adds its share.
    assert_eq!(rules.mana(def, 0), 60);
    assert_eq!(rules.mana(def, 10), 66);
    assert!(def.aims_item() && !def.aimed());
}

#[test]
fn casting_identify_with_too_little_mana_is_refused_with_no_time() {
    let mut w = world(59);
    let id = spell(&w);
    w.player_mut().spells.insert(id);
    let on = ring(&mut w);
    let before = w.state.time;
    let action = format!("use:identify:{}", crate::action::position_text(on));
    let out = run(&mut w, &action);
    assert!(!out[0].took_turn);
    assert_eq!(w.state.time, before);
    assert_eq!(w.state.player().mana, 59);
    assert_eq!(last(&w), "You have too little mana.");
    assert!(!w.state.player().inventory[on].is_identified());
    // The abilities menu greys it out for the same reason.
    let own = w.perception().abilities();
    let row = own.iter().find(|a| a.ability == Ability::Spell(id));
    assert_eq!(
        row.and_then(|a| a.unavailable),
        Some(crate::ability::Unavailable::NoMana)
    );
}

#[test]
fn casting_identify_names_a_stack_and_spends_its_mana() {
    let mut w = world(100);
    let id = spell(&w);
    w.player_mut().spells.insert(id);
    assert_eq!(Ability::Spell(id).aim(&w.content), Aim::Item);
    let layer = crate::dump::parse_layers("abilities").expect("layer")[0];
    let rows = crate::dump::layer_rows(&w, layer);
    assert!(
        rows.contains(&"Identify (spell): an item you carry".to_string()),
        "{rows:?}"
    );
    let on = ring(&mut w);
    assert_eq!(
        Ability::Spell(id).action_on(&w.content, on),
        Some(Action::CastOn("identify".into(), on))
    );
    // Without a stack it asks for one, no time, no mana.
    let out = run(&mut w, "use:identify");
    assert!(!out[0].took_turn);
    assert!(last(&w).starts_with("Cast it on what?"), "{}", last(&w));
    let out = run(&mut w, &Action::CastOn("identify".into(), on).to_string());
    assert!(out[0].took_turn);
    assert_eq!(w.state.player().mana, 40);
    assert!(w.state.player().inventory[on].is_identified());
    assert!(last(&w).starts_with("This is a silver ring of the Mule"));
    assert_eq!(w.state.player().inventory[0].count, 2, "no scroll used");
    // Known now: refused, no mana spent.
    w.player_mut().mana = 100;
    let out = run(&mut w, &Action::CastOn("identify".into(), on).to_string());
    assert!(!out[0].took_turn);
    assert_eq!(w.state.player().mana, 100);
    assert!(
        last(&w).starts_with("You already know what"),
        "{}",
        last(&w)
    );
}

#[test]
fn the_scroll_of_identify_teaches_identify() {
    let mut w = world(100);
    // At the spell's level (6) or above, so the lesson is certain.
    while w.player_level() < 8 {
        run(&mut w, "xp:5000 levelup:auto");
    }
    let id = spell(&w);
    assert!(!w.state.player().knows_spell(&w.content, id));
    let studies = w.perception().studies();
    assert!(
        studies.iter().any(|s| s.item == 0 && s.spell == id),
        "on the study list"
    );
    let out = run(&mut w, "study:a");
    assert!(out[0].took_turn);
    assert!(w.state.player().knows_spell(&w.content, id), "{}", last(&w));
    assert_eq!(w.state.player().inventory[0].count, 1, "one scroll used up");
}
