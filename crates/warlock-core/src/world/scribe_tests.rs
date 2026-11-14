//! Writing scrolls (spec §8 "Scrolls"; the witch and wizard design §6.1): the price by spell and
//! by Archmage, the time, interruption, the scroll made known at its own rung, every shipped
//! spell writable, the refusals, the wizard's table, the dumps, saves and replays.

use super::caster_tests::sp;
use super::*;
use crate::action::parse_actions;
use crate::content::{ContentFiles, Reading};
use crate::replay::Replay;
use crate::test_support::builtin;
use crate::test_support::run;
use crate::time::Duration;

/// The pack: a, two blank scrolls; b, a dagger.
const PACK: &str = "carry = [\"blank_scroll*2\", \"dagger\"]";

const HALL: &str = "#######\n#.....#\n#..@..#\n#.....#\n#######\n";

/// A lit hall of `map` (`@` the player, `r` a giant rat), the player with `mana` base mana and
/// `pack`, knowing Magic missile and Fireball, with Scribe.
fn world_with(map: &str, mana: u16, pack: &str) -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/hall.toml",
        &format!(
            "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"r\" = {{ terrain = \"floor\", creature = \"rat\" }}\n"
        ),
    );
    files.insert(
        "scenarios/hall.toml",
        &format!("name = \"Hall\"\nstart_level = \"hall\"\nhp = 500\nmana = {mana}\n{pack}\n"),
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let mut w = World::new(content, "hall", None).expect("world");
    let me = w.player_mut();
    me.talents.take("scribe", None);
    me.spells.insert(sp("magic_missile"), 1);
    me.spells.insert(sp("fireball"), 1);
    me.mana = me.mana_base;
    w
}

fn world() -> World {
    world_with(HALL, 200, PACK)
}

fn last(w: &World) -> String {
    (w.messages().iter().last())
        .map(|m| m.display())
        .unwrap_or_default()
}

fn count(w: &World, object: &str) -> u32 {
    let id = w.content.object_id(object).expect("object");
    (w.state.player().inventory.iter())
        .filter(|s| s.object == id)
        .map(|s| s.count)
        .sum()
}

fn cost(w: &World, spell: &str) -> i32 {
    let def = w.content.spell(sp(spell));
    (w.state.player().scribe_cost(&w.content, def)).expect("Scribe")
}

#[test]
fn the_price_is_three_casts_of_the_spell() {
    let w = world();
    let me = w.state.player();
    for (spell, cast, written) in [("fireball", 26, 78), ("magic_missile", 4, 12)] {
        let def = w.content.spell(sp(spell));
        assert_eq!(me.spell_cost(&w.content, def), cast, "{spell}");
        assert_eq!(cost(&w, spell), written, "{spell}");
    }
    // Identify's 60 is 180.
    assert_eq!(cost(&w, "identify"), 180);
    // Archmage's cut is the spell's, so the scroll's too: three times the cut price.
    let mut w = world();
    w.player_mut().talents.take("archmage", None);
    let def = w.content.spell(sp("fireball"));
    let base = w.state.player().spell_base(&w.content, def);
    assert!(base < 26, "{base}");
    assert_eq!(cost(&w, "fireball"), 3 * base);
    // Without Scribe there is no price.
    let mut w = world();
    w.player_mut().talents = Default::default();
    assert_eq!(w.state.player().scribe_cost(&w.content, def), None);
}

#[test]
fn a_scroll_is_written_in_a_studys_time_and_known() {
    let mut w = world();
    let scroll = w
        .content
        .object_id("scroll_of_magic_missile")
        .expect("scroll");
    assert!(w.lore().label(scroll).is_some(), "unknown before");
    let mana = w.state.player().mana;
    let out = run(&mut w, "scribe:magic_missile").remove(0);
    // Magic missile is level 3: 100 + 3 × 20 = 160 turns, 16 minutes.
    assert_eq!(out.elapsed, Duration::minutes(16));
    assert_eq!(w.state.player().mana, mana - 12);
    assert_eq!(count(&w, "blank_scroll"), 1, "one used up");
    assert_eq!(count(&w, "scroll_of_magic_missile"), 1);
    assert!(
        w.lore().label(scroll).is_none(),
        "the writer knows what he wrote"
    );
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    assert!(
        said.contains(&"You begin to write.".to_string()),
        "{said:?}"
    );
    assert_eq!(last(&w), "You write a scroll of magic missile.");
    // It is the ordinary scroll, at its own rung (Fireball's is 1, as a found one).
    run(&mut w, "scribe:fireball");
    let fireball = w.content.object_id("scroll_of_fireball").expect("scroll");
    let written = (w.state.player().inventory.iter()).find(|s| s.object == fireball);
    assert!(written.is_some_and(|s| *s == crate::object::Stack::one(fireball)));
    let Some(Reading::Cast(read)) = &w.content.object(fireball).read else {
        panic!("a spell scroll");
    };
    assert_eq!(read.rungs, 1);
    assert_eq!(count(&w, "blank_scroll"), 0);
    // And it reads.
    let at = (w.state.player().inventory.iter()).position(|s| s.object == fireball);
    let text = format!("read:{}:3,1", at.expect("carried"));
    assert!(run(&mut w, &text)[0].took_turn);
}

#[test]
fn quick_study_does_not_shorten_it() {
    let mut w = world();
    w.player_mut().talents.take("quick_study", None);
    let out = run(&mut w, "scribe:magic_missile").remove(0);
    assert_eq!(out.elapsed, Duration::minutes(16));
}

#[test]
fn an_interruption_keeps_the_blank_and_the_mana() {
    let map = "#######\n#.....#\n#..@r.#\n#.....#\n#######\n";
    let mut w = world_with(map, 200, PACK);
    let mana = w.state.player().mana;
    let out = run(&mut w, "scribe:magic_missile").remove(0);
    assert!(out.took_turn);
    assert!(out.elapsed < Duration::minutes(16));
    assert_eq!(last(&w), "You stop writing.");
    assert_eq!(count(&w, "blank_scroll"), 2, "kept");
    assert_eq!(count(&w, "scroll_of_magic_missile"), 0);
    assert!(w.state.player().mana >= mana, "none spent");
}

#[test]
fn refusals_take_no_time_and_spend_nothing() {
    let refused = |w: &mut World, action: &str, said: &str| {
        let before = (w.state.player().mana, count(w, "blank_scroll"));
        let out = run(w, action).remove(0);
        assert!(!out.took_turn, "{action}: {}", last(w));
        assert_eq!(last(w), said, "{action}");
        let after = (w.state.player().mana, count(w, "blank_scroll"));
        assert_eq!(after, before, "{action}");
    };
    let mut w = world();
    refused(&mut w, "scribe:no_such", "There is no spell `no_such`.");
    refused(&mut w, "scribe:haste", "You do not know that spell.");
    // Too little mana for three casts of Fireball (78).
    let mut poor = world_with(HALL, 20, PACK);
    refused(
        &mut poor,
        "scribe:fireball",
        "You lack the mana to write it.",
    );
    // Nothing to write on.
    let mut bare = world_with(HALL, 200, "carry = [\"dagger\"]");
    refused(
        &mut bare,
        "scribe:fireball",
        "You have nothing to write on.",
    );
    // Raging: no mana is spent while raging.
    let mut raging = world();
    run(&mut raging, "resource:rage");
    refused(&mut raging, "scribe:fireball", "You are still raging.");
    // Without the ability.
    let mut w = world();
    w.player_mut().talents = Default::default();
    refused(&mut w, "scribe:fireball", "You have no such ability.");
    // The bare ability from the menu asks which spell.
    let mut w = world();
    refused(
        &mut w,
        "use:scribe",
        "Write which spell? Name one you know: `scribe:fireball`.",
    );
}

#[test]
fn a_blank_scroll_has_nothing_to_read() {
    let mut w = world();
    let out = run(&mut w, "read:a").remove(0);
    assert!(!out.took_turn);
    assert_eq!(last(&w), "There is nothing written on it.");
    assert_eq!(count(&w, "blank_scroll"), 2);
}

#[test]
fn every_shipped_spell_has_a_scroll_to_write() {
    let content = builtin();
    for spell in content.spells_by_level() {
        let scroll = content.scroll_of(spell);
        let id = &content.spell(spell).id;
        assert!(scroll.is_some(), "{id}");
        let read = scroll.and_then(|s| content.object(s).read.as_ref());
        assert_eq!(read.and_then(Reading::spell), Some(spell), "{id}");
    }
    assert!(content.blank_scroll().is_some());
}

#[test]
fn the_wizard_writes_from_12() {
    let content = builtin();
    let spec: crate::character::CharacterSpec = "human,female,wizard".parse().expect("spec");
    let mut w = World::with_character(content, "showcase", None, Some(&spec)).expect("world");
    let has = |w: &World, id: &str| w.state.player().talents.holds(id, None);
    while w.player_level() < 11 {
        run(&mut w, "xp:2000 levelup:auto");
    }
    assert!(!has(&w, "scribe"));
    while w.player_level() < 12 {
        run(&mut w, "xp:2000 levelup:auto");
    }
    assert!(has(&w, "scribe"));
    let abilities = w.perception().abilities();
    let scribe = crate::ability::Ability::Class(crate::content::ClassActive::Scribe);
    assert!(abilities.iter().any(|a| a.ability == scribe));
}

#[test]
fn the_dumps_and_the_scribe_list_say_it() {
    use crate::dump::{layer_rows, parse_layers};
    let abilities = parse_layers("abilities").expect("layer")[0];
    let w = world();
    let rows = layer_rows(&w, abilities);
    assert!(
        rows.contains(&"Scribe (class): a spell you know".to_string()),
        "{rows:?}"
    );
    let view = w.perception();
    let scribes = view.scribes();
    let mm = (scribes.iter()).find(|s| s.spell == sp("magic_missile"));
    assert_eq!(
        mm.map(|s| (s.mana, s.time)),
        Some((12, Duration::minutes(16)))
    );
    assert_eq!(view.blank_scrolls(), 2);
}

#[test]
fn writing_saves_and_replays() {
    // A wizard in the hall with its pack (two blanks): Scribe from level 12, by actions alone.
    let content = world().content.clone();
    let spec: crate::character::CharacterSpec = "human,female,wizard".parse().expect("spec");
    let mut w = World::with_character(content.clone(), "hall", None, Some(&spec)).expect("world");
    let mut replay = Replay::new(&w);
    let actions = "xp:200000 levelup:auto scribe:magic_missile wait scribe:ray_of_frost";
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(w.player_level() >= 12);
    assert_eq!(count(&w, "blank_scroll"), 0, "{:?}", last(&w));
    assert_eq!(count(&w, "scroll_of_magic_missile"), 1);
    let loaded = crate::save::decode(&crate::save::encode(&w), content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    let played = replay.run(content).expect("replays");
    assert_eq!(played.state_hash(), w.state_hash());
}
