//! The first spell list (spec §8 "Spells", "Scrolls"; #397): a scroll of a spell casts it
//! through the one cast path at its own caster level, free of mana; the area hexes (Sleep, Web)
//! and Haste; the list's content checks (one scroll a spell, the classes' guaranteed spells
//! within their caster level, mana for everyone).

use super::*;
use crate::action::parse_actions;
use crate::content::{ActionKind, ContentFiles, Reading, SpellPurpose, TalentEffect};

/// The pack: a, two scrolls of magic missile; b, one of haste; c, one of web; d, one of sleep;
/// e, one of fireball.
const PACK: &str = "carry = [\"scroll_of_magic_missile*2\", \"scroll_of_haste\", \
     \"scroll_of_web\", \"scroll_of_sleep\", \"scroll_of_fireball\"]";

/// A lit hall, the player (`@`) with a giant rat (`r`) two cells east of them, no mana.
const HALL: &str = "#########\n#.......#\n#..@.r..#\n#.......#\n#########\n";

fn world() -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/hall.toml",
        &format!(
            "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{HALL}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"r\" = {{ terrain = \"floor\", creature = \"rat\" }}\n"
        ),
    );
    files.insert(
        "scenarios/hall.toml",
        &format!("name = \"Hall\"\nstart_level = \"hall\"\nhp = 500\n{PACK}\n"),
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "hall", Some(3)).expect("world")
}

fn run(w: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|a| w.apply(a))
        .collect()
}

fn said(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

fn count(w: &World, object: &str) -> u32 {
    let id = w.content.object_id(object).expect("object");
    (w.state.player().inventory.iter())
        .filter(|s| s.object == id)
        .map(|s| s.count)
        .sum()
}

/// The rat, put back two cells east of the player with plenty of hit points; its id.
fn rat(w: &mut World) -> ActorId {
    let here = w.player_pos();
    let player = w.state.player;
    let rat = (w.state.actors.values())
        .find(|a| a.id != player)
        .map(|a| a.id)
        .expect("the rat");
    let actor = w.state.actors.get_mut(&rat).expect("the rat");
    actor.pos = Pos::new(here.x + 2, here.y);
    actor.hp = 1000;
    actor.hp_base = 1000;
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    rat
}

fn suffers(w: &World, who: ActorId, status: &str) -> bool {
    let id = w.content.status_id(status).expect("a status");
    w.state.actors[&who].effects.has(id)
}

#[test]
fn a_spell_scroll_casts_its_spell_free_of_mana_and_teaches_it() {
    let mut w = world();
    let target = rat(&mut w);
    let at = w.state.actors[&target].pos;
    let mana = w.state.player().mana;
    // Without a cell an aimed spell is not read: no time, the scroll kept.
    let out = run(&mut w, "read:a").remove(0);
    assert!(!out.took_turn);
    assert_eq!(
        said(&w).last().map(String::as_str),
        Some("Cast it where? Name a cell: `read:a:12,5`.")
    );
    assert_eq!(count(&w, "scroll_of_magic_missile"), 2);
    let out = run(&mut w, &format!("read:a:{},{}", at.x, at.y)).remove(0);
    assert!(out.took_turn);
    assert_eq!(count(&w, "scroll_of_magic_missile"), 1, "used up");
    assert_eq!(w.state.player().mana, mana, "free of mana");
    // Magic missile never misses: 1d4 force at the scroll's level (3, the spell's own).
    let lost = 1000 - w.state.actors[&target].hp;
    assert!((1..=4).contains(&lost), "{lost}");
    let said = said(&w);
    assert!(
        said.iter().any(|m| m == "It is a scroll of magic missile."),
        "{said:?}"
    );
    // Read, its type is known; it teaches its spell to whoever studies it.
    let scroll = w
        .content
        .object_id("scroll_of_magic_missile")
        .expect("scroll");
    let teaches = (w.content.object(scroll).read.as_ref()).and_then(Reading::teaches);
    assert_eq!(teaches, w.content.spell_id("magic_missile"));
    let studies = w.perception().studies();
    assert!(
        studies.iter().any(|s| Some(s.spell) == teaches),
        "{studies:?}"
    );
}

#[test]
fn haste_read_on_oneself_makes_everything_quicker() {
    let mut w = world();
    let player = w.state.player;
    let step = w.price_of(player, ActionKind::Step);
    // A spell on oneself is read without aiming.
    let out = run(&mut w, "read:b").remove(0);
    assert!(out.took_turn);
    assert!(suffers(&w, player, "hasted"));
    assert_eq!(count(&w, "scroll_of_haste"), 0);
    assert_eq!(w.price_of(player, ActionKind::Step), (step * 75 + 99) / 100);
    assert!(said(&w).contains(&"The world slows around you.".to_string()));
}

#[test]
fn an_area_hex_falls_on_everyone_it_reaches_but_the_caster() {
    let mut w = world();
    let player = w.state.player;
    let target = rat(&mut w);
    // Aimed at the cell between them: the burst of radius 1 reaches both, the reader spared.
    let here = w.player_pos();
    run(&mut w, &format!("read:c:{},{}", here.x + 1, here.y));
    assert!(suffers(&w, target, "webbed"));
    assert!(!suffers(&w, player, "webbed"));
    assert!(said(&w).iter().any(|m| m.ends_with("is caught in a web.")));
    let target = rat(&mut w);
    run(&mut w, &format!("read:c:{},{}", here.x + 2, here.y));
    // The web is gone (read), so Sleep: it is now c.
    assert!(suffers(&w, target, "drowsy"));
    assert!(!suffers(&w, player, "drowsy"));
}

#[test]
fn a_spell_scroll_that_kills_its_reader_is_what_killed_them() {
    let mut w = world();
    w.player_mut().hp = 1;
    let here = w.player_pos();
    // The fireball (a burst scroll) next to the reader catches them.
    run(&mut w, &format!("read:e:{},{}", here.x + 1, here.y));
    let death = w.state.death.as_ref().expect("dead");
    let scroll = w.content.object_id("scroll_of_fireball").expect("scroll");
    assert_eq!(death.cause, Some(Cause::Scroll(scroll)));
}

#[test]
fn reading_spell_scrolls_replays_and_saves_the_same() {
    let mut w = world();
    let content = w.content.clone();
    let mut replay = crate::replay::Replay::new(&w);
    let here = w.player_pos();
    let (x, y) = (here.x + 2, here.y);
    let text = format!("read:b read:a:{x},{y} wait read:c:{x},{y}");
    for action in parse_actions(&text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let played = replay.run(content.clone()).expect("replays");
    assert_eq!(played.state_hash(), w.state_hash());
    let loaded = crate::save::decode(&crate::save::encode(&w), content).expect("loads");
    assert_eq!(loaded.state(), w.state());
}

#[test]
fn every_spell_comes_on_one_scroll_and_every_scroll_names_a_spell() {
    let content = Content::builtin().expect("content");
    for (i, spell) in content.spells().iter().enumerate() {
        let id = crate::content::SpellId(i as u16);
        let scrolls: Vec<&str> = (content.object_defs().iter())
            .filter(|o| o.read.as_ref().and_then(Reading::teaches) == Some(id))
            .map(|o| o.id.as_str())
            .collect();
        assert_eq!(scrolls.len(), 1, "{}: {scrolls:?}", spell.id);
        let scroll = content.object(content.object_id(scrolls[0]).expect("scroll"));
        assert_eq!(
            content.item_class(scroll.class).id,
            "scrolls",
            "{}",
            spell.id
        );
        // Loot by spell level: the lower the level, the sooner and the commoner.
        assert!(scroll.frequency > 0, "{} is loot", scroll.id);
    }
    // The scroll of identify keeps its own loot (the commonest scroll, #592), whatever its level.
    let scrolls: Vec<(u8, u8, u32, &str)> = (content.object_defs().iter())
        .filter(|o| o.id != "scroll_of_identify")
        .filter_map(|o| Some((o, content.spell(o.read.as_ref()?.teaches()?))))
        .map(|(o, s)| (s.level, o.level, o.frequency, o.id.as_str()))
        .collect();
    for lower in &scrolls {
        for higher in scrolls.iter().filter(|h| h.0 > lower.0) {
            assert!(
                lower.1 <= higher.1 && lower.2 >= higher.2,
                "{} before {}",
                lower.3,
                higher.3
            );
        }
    }
    let first = |l: u8| scrolls.iter().find(|s| s.0 == l).map(|s| s.1);
    assert!(first(1) < first(3) && first(3) < first(6) && first(6) < first(11));
}

#[test]
fn every_class_has_mana_and_its_guaranteed_spells_within_its_caster_level() {
    let content = Content::builtin().expect("content");
    let book = content.talents();
    for class in content.classes() {
        assert!(class.mana >= 1, "{} has mana", class.id);
        let mut spells = Vec::new();
        for (level, talent) in &class.abilities {
            let def = book.get(book.id(talent).expect("a talent"));
            for effect in &def.effects {
                let TalentEffect::Spell { spell } = effect else {
                    continue;
                };
                let s = content.spell(content.spell_id(spell).expect("a spell"));
                // Given at its level, a spell is within the caster level it comes at (the
                // spell scaling design §5.6).
                let caster = (i32::from(*level) * i32::from(class.casting) / 100).max(1);
                assert!(
                    i32::from(s.level) <= caster,
                    "{}'s {} at {level}",
                    class.id,
                    s.id
                );
                spells.push((*level, s.level, s.kind));
            }
        }
        // A full caster: a cantrip at level 1 and about four spells in all.
        if class.casting == 100 {
            assert!(
                spells.iter().any(|&(l, t, _)| l == 1 && t <= 1),
                "{}'s cantrip",
                class.id
            );
            assert!((4..=6).contains(&spells.len()), "{}: {spells:?}", class.id);
        }
        // Only the casters and the hybrids have any, and a hybrid's are low boons.
        if class.casting < 100 {
            assert!(spells.len() <= 2, "{}: {spells:?}", class.id);
            assert!(
                (spells.iter()).all(|&(_, t, k)| t <= 5 && k != SpellPurpose::Damage),
                "{}: {spells:?}",
                class.id
            );
        }
    }
}
