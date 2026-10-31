//! Wands (spec §8 "Magic items": "Wands"; #398): a wand in the missile weapon slot casts its
//! spell through the one cast path at its own caster level, a charge a cast; an empty wand does
//! nothing; a wand type is unknown until used or identified, its charges until identified; a
//! wand and a launcher compete for the slot; charges are saved.

use super::*;
use crate::action::{AttackKind, parse_actions};
use crate::content::ContentFiles;

/// The pack: a, a wand of magic missile; b, a wand of scorching ray; c, a crossbow; d, a
/// scroll of identify; e, a wand of fireball.
const PACK: &str = "carry = [\"wand_of_magic_missile\", \"wand_of_scorching_ray\", \
     \"crossbow\", \"scroll_of_identify\", \"wand_of_fireball\"]";

/// A lit hall, the player (`@`) with a giant rat (`r`) two cells east of them.
const HALL: &str = "#############\n#...........#\n#..@.r......#\n#...........#\n#############\n";

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

fn left(w: &World, item: usize) -> u8 {
    (w.state.player().inventory[item])
        .charges_left(&w.content)
        .expect("a wand")
}

fn fire_at_rat(w: &mut World, rat: ActorId) -> Outcome {
    let at = w.state.actors[&rat].pos;
    run(w, &format!("fire:{},{}", at.x, at.y)).remove(0)
}

#[test]
fn a_wand_casts_its_spell_for_a_charge_and_names_itself() {
    let mut w = world();
    let target = rat(&mut w);
    let mana = w.state.player().mana;
    let first = w.lore().describe(&w.state.player().inventory[0]);
    assert!(first.ends_with(" wand"), "{first}");
    assert!(!w.lore().knows(&w.state.player().inventory[0]));
    let wand = w.content.object(w.state.player().inventory[0].object).wand;
    assert_eq!(wand.map(|w| w.charges), Some(12), "tier 1: 12 charges");
    run(&mut w, "equip:missile_weapon:a");
    let hp = w.state.actors[&target].hp;
    let out = fire_at_rat(&mut w, target);
    assert!(out.took_turn);
    assert!(
        w.state.actors[&target].hp < hp,
        "magic missile never misses"
    );
    assert_eq!(left(&w, 0), 11);
    assert_eq!(w.state.player().mana, mana, "a charge, not mana");
    let text = said(&w).join(" ");
    let name = first.split_once(" ").expect("an article").1;
    assert!(text.contains(&format!("You zap the {name}.")), "{text}");
    assert!(text.contains("It is a wand of magic missile."), "{text}");
    // The type is known now, the charges are not.
    let stack = &w.state.player().inventory[0];
    assert_eq!(w.lore().describe(stack), "a wand of magic missile");
    assert!(!w.lore().knows(stack) && stack.known_charges(&w.content).is_none());
}

#[test]
fn an_empty_wand_does_nothing_and_takes_no_time() {
    let mut w = world();
    let target = rat(&mut w);
    run(&mut w, "equip:missile_weapon:a");
    for _ in 0..12 {
        w.player_mut().inventory[0].use_charge();
    }
    let hp = w.state.actors[&target].hp;
    let out = fire_at_rat(&mut w, target);
    assert!(!out.took_turn);
    assert_eq!(
        said(&w).last().map(String::as_str),
        Some("Nothing happens.")
    );
    assert_eq!(w.state.actors[&target].hp, hp);
    let stack = &w.state.player().inventory[0];
    assert_eq!(stack.known_charges(&w.content), Some(0), "now known empty");
    assert!(w.lore().describe(stack).ends_with("(empty)"));
}

#[test]
fn a_refused_cast_uses_no_charge() {
    let mut w = world();
    rat(&mut w);
    run(&mut w, "equip:missile_weapon:a");
    let here = w.player_pos();
    // At oneself, and out of reach.
    for at in [here, Pos::new(here.x + 30, here.y)] {
        let out = run(&mut w, &format!("fire:{},{}", at.x, at.y)).remove(0);
        assert!(!out.took_turn, "{:?}", said(&w).last());
    }
    assert_eq!(left(&w, 0), 12);
}

#[test]
fn identifying_a_wand_tells_its_spell_and_charges() {
    let mut w = world();
    let out = run(&mut w, "read:d:a").remove(0);
    assert!(out.took_turn);
    let stack = &w.state.player().inventory[0];
    assert!(w.lore().knows(stack));
    assert_eq!(stack.known_charges(&w.content), Some(12));
    assert_eq!(
        w.lore().describe(stack),
        "a wand of magic missile (12 charges)"
    );
    // Its dice as the player would use it: the wand's rung (its kind's lowest until S5) and
    // their wand rungs, as the abilities menu shows a spell's.
    let lines: Vec<String> = (w.perception().describe(stack).into_iter())
        .map(|l| l.text)
        .collect();
    let content = w.content.clone();
    let wand = content.object(stack.object).wand.expect("a wand");
    let by = super::CastBy::Wand(wand.min_rung);
    let roll = (w.state.player())
        .spell_roll_words(&content, content.spell(wand.spell), by)
        .expect("it rolls");
    let holds = format!("Holds Magic missile: {}", roll.text);
    assert!(lines.contains(&holds), "{lines:?}");
    assert!(lines.iter().any(|l| l == "12 charges left"), "{lines:?}");
}

#[test]
fn an_unidentified_wand_hides_its_spell_and_charges() {
    let w = world();
    let stack = &w.state.player().inventory[4];
    let lines: Vec<String> =
        (crate::magic::describe::appraised(stack, w.lore(), Default::default()))
            .into_iter()
            .map(|l| l.text)
            .collect();
    let text = lines.join(" ");
    assert!(
        !text.contains("Fireball") && !text.contains("fireball"),
        "{text}"
    );
    assert!(text.contains("charges are unknown"), "{text}");
}

#[test]
fn an_attack_wand_rolls_to_hit_and_an_auto_wand_never_misses() {
    let strikes = |item: &str| {
        let mut w = world();
        let target = rat(&mut w);
        run(&mut w, &format!("equip:missile_weapon:{item}"));
        let index = usize::from(item.as_bytes()[0] - b'a');
        let mut all = Vec::new();
        for _ in 0..30 {
            w.player_mut().inventory[index] = Stack::one(w.state.player().inventory[index].object);
            let out = fire_at_rat(&mut w, target);
            assert!(out.took_turn);
            all.extend(out.events.iter().filter_map(|e| match *e {
                Event::Attacked {
                    kind: AttackKind::Spell,
                    strike: Some(strike),
                    ..
                } => Some(strike),
                _ => None,
            }));
        }
        all
    };
    let missiles = strikes("a");
    assert_eq!(missiles.len(), 30);
    assert!(missiles.iter().all(|s| s.is_hit()));
    let rays = strikes("b");
    assert!(rays.iter().any(|s| !s.is_hit()), "{rays:?}");
    assert!(rays.iter().any(|s| s.is_hit()), "{rays:?}");
}

#[test]
fn a_wand_casts_at_its_lowest_rung_and_wand_mastery_raises_it() {
    let mut w = world();
    let content = w.content.clone();
    let fireball = content.object_id("wand_of_fireball").expect("a wand");
    let wand = content.object(fireball).wand.expect("a wand");
    assert_eq!(
        (wand.min_rung, wand.max_rung),
        (0, 2),
        "its own, given in its file"
    );
    assert_eq!(wand.charges, 7);
    let def = content.spell(wand.spell);
    let roll = |w: &World| {
        let by = super::CastBy::Wand(wand.min_rung);
        let r = w.state.player().spell_roll(&content, def, by);
        r.map(|r| r.to_string()).unwrap_or_default()
    };
    // Until each wand draws its own rung (S5): the range's lowest, the base, whatever the
    // user's Intelligence.
    super::cast_tests::scores(&mut w, &[(crate::stats::Stat::Intelligence, 25)]);
    assert_eq!(roll(&w), "2d6");
    w.player_mut().talents.take("wand_mastery", None);
    assert_eq!(roll(&w), "3d6", "a rung higher");
}

#[test]
fn a_wand_and_a_launcher_share_the_missile_weapon_slot() {
    let mut w = world();
    run(&mut w, "equip:missile_weapon:a");
    assert_eq!(w.state.player().wand_item(&w.content), Some(0));
    assert!(w.perception().ready_wand().is_some());
    run(&mut w, "remove:missile_weapon equip:missile_weapon:c");
    assert_eq!(w.state.player().wand_item(&w.content), None);
    assert!(w.state.player().launcher(&w.content).is_some());
}

#[test]
fn charges_are_saved_and_change_the_state_hash() {
    let mut w = world();
    let target = rat(&mut w);
    run(&mut w, "equip:missile_weapon:a");
    let fresh = serde_json::to_string(&w.state.player().inventory[0]).expect("json");
    assert!(
        !fresh.contains("charges"),
        "a fresh wand saves as before: {fresh}"
    );
    let before = w.state_hash();
    fire_at_rat(&mut w, target);
    assert_ne!(w.state_hash(), before);
    let loaded = crate::save::decode(&crate::save::encode(&w), w.content.clone()).expect("decodes");
    assert_eq!(left(&loaded, 0), 11);
}

#[test]
fn wand_labels_leave_the_scrolls_labels_as_they_were() {
    let with = Content::builtin().expect("content");
    let mut files = ContentFiles::builtin();
    files.remove("data/wand_labels.toml");
    let without = Content::load(&files).expect("content");
    let a = crate::scroll_label::Labels::draw(&with, 7);
    let b = crate::scroll_label::Labels::draw(&without, 7);
    let mut wands = Vec::new();
    for (i, o) in with.object_defs().iter().enumerate() {
        let id = crate::content::ObjectId(i as u16);
        if o.read.is_some() {
            assert_eq!(a.get(id), b.get(id), "{}", o.id);
        }
        if o.wand.is_some() {
            assert!(b.get(id).is_none());
            wands.push(a.get(id).expect("a label").to_string());
        }
    }
    let mut unique = wands.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), wands.len(), "{wands:?}");
}

#[test]
fn wand_data_is_checked() {
    let load = |wand: &str| {
        let mut files = ContentFiles::builtin();
        files.insert(
            "data/objects/test_wand.toml",
            &format!(
                "[[object]]\nid = \"test_wand\"\nname = \"test wand\"\nglyph = \"-\"\n\
                 fg = \"#ffffff\"\nclass = \"wands\"\nmaterials = [\"wood\"]\nweight = 10\n\
                 value = 1\nslot = \"missile_weapon\"\n[object.wand]\n{wand}\n"
            ),
        );
        Content::load(&files).map(|_| ()).map_err(|e| e.to_string())
    };
    assert_eq!(load("spell = \"fireball\"\ncharges = 7"), Ok(()));
    assert_eq!(
        load("spell = \"fireball\"\ncharges = 7\nrungs = { min = 1, max = 4 }"),
        Ok(())
    );
    let e = load("spell = \"fireball\"\ncharges = 7\ncaster_level = 12").unwrap_err();
    assert!(e.contains("`caster_level` is gone"), "{e}");
    let refused = [
        "spell = \"haste\"\ncharges = 7",
        "spell = \"nothing\"\ncharges = 7",
        "spell = \"fireball\"",
        "spell = \"fireball\"\ncharges = 0",
        "spell = \"fireball\"\ncharges = 100",
        // `caster_level` is gone (the spell scaling design §7.4); `rungs` within the ladder.
        "spell = \"fireball\"\ncharges = 7\ncaster_level = 12",
        "spell = \"fireball\"\ncharges = 7\nrungs = { min = 0, max = 5 }",
        "spell = \"fireball\"\ncharges = 7\nrungs = { min = 3, max = 2 }",
        "spell = \"web\"\ncharges = 7\nrungs = { min = 0, max = 1 }",
    ];
    for wand in refused {
        assert!(load(wand).is_err(), "{wand}");
    }
}
