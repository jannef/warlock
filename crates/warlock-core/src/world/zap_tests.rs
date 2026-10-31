//! Wands (spec §8 "Magic items": "Wands"; #398): a wand in the missile weapon slot casts its
//! spell through the one cast path at its own rung, drawn as it was made (the spell scaling
//! design §7.4), a charge a cast; an empty wand does nothing; a wand type is unknown until used
//! or identified, its charges until identified, its rung with its type; a wand and a launcher
//! compete for the slot; charges and rungs are saved.

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
    let lines: Vec<String> =
        (crate::magic::describe::appraised(stack, w.lore(), Default::default()))
            .into_iter()
            .map(|l| l.text)
            .collect();
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("Holds Magic missile (level 3)")),
        "{lines:?}"
    );
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

/// `stack` with its wand rung set to `rung` (`None` leaves it out), as a save writes it.
fn with_rung(stack: &Stack, rung: Option<u8>) -> Stack {
    let mut json = serde_json::to_value(stack).expect("json");
    let map = json.as_object_mut().expect("a stack is an object");
    match rung {
        Some(r) => map.insert("wand_rung".into(), r.into()),
        None => map.remove("wand_rung"),
    };
    serde_json::from_value(json).expect("a stack")
}

#[test]
fn a_wand_casts_at_its_own_rung_and_wand_mastery_raises_it() {
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
        let stack = &w.state.player().inventory[4];
        let by = super::CastBy::Wand(stack.wand_rung(&content).expect("a wand"));
        let r = w.state.player().spell_roll(&content, def, by);
        r.map(|r| r.to_string()).unwrap_or_default()
    };
    // The wand's own rung, whatever the user's Intelligence: rung 1 is 3d6.
    super::cast_tests::scores(&mut w, &[(crate::stats::Stat::Intelligence, 25)]);
    let one = with_rung(&w.state.player().inventory[4], Some(1));
    w.player_mut().inventory[4] = one;
    assert_eq!(roll(&w), "3d6");
    w.player_mut().talents.take("wand_mastery", None);
    assert_eq!(roll(&w), "4d6", "a rung higher");
}

#[test]
fn the_zap_casts_at_the_stacks_rung() {
    // Magic missile: 1d4 at rung 0, 3d4 at rung 2 (it never misses).
    let most = |rung: u8| {
        let mut w = world();
        let target = rat(&mut w);
        let wand = with_rung(&w.state.player().inventory[0], Some(rung));
        w.player_mut().inventory[0] = wand;
        run(&mut w, "equip:missile_weapon:a");
        let mut most = 0;
        for _ in 0..12 {
            let before = w.state.actors[&target].hp;
            assert!(fire_at_rat(&mut w, target).took_turn);
            most = most.max(before - w.state.actors[&target].hp);
        }
        most
    };
    assert!(most(0) <= 4, "1d4 at rung 0");
    assert!(most(2) > 4, "3d4 at rung 2");
}

#[test]
fn a_wand_draws_its_rung_evenly_once_as_it_is_made() {
    let content = Content::builtin().expect("content");
    let fireball = content.object_id("wand_of_fireball").expect("a wand");
    let mut counts = [0u32; 3];
    for seed in 0..3000 {
        let mut stack = Stack::one(fireball);
        let mut rng = crate::rng::Rng::stream(seed, "wand-test");
        stack.draw_wand_rung(&content, &mut rng);
        let rung = stack.wand_rung(&content).expect("a wand");
        counts[usize::from(rung)] += 1;
        // Drawn once: a second draw leaves it.
        let again = stack.clone();
        stack.draw_wand_rung(&content, &mut rng);
        assert_eq!(stack, again);
    }
    for n in counts {
        assert!((900..=1100).contains(&n), "{counts:?}");
    }
    // The same seed, the same rungs (the scenario's pack draws them).
    let rungs = |w: &World| -> Vec<Option<u8>> {
        (w.state.player().inventory.iter())
            .map(|s| s.wand_rung(&w.content))
            .collect()
    };
    assert_eq!(rungs(&world()), rungs(&world()));
    assert_eq!(rungs(&world()).iter().flatten().count(), 3, "three wands");
    let drawn = |w: &World| serde_json::to_string(&w.state.player().inventory[4]).expect("json");
    assert!(drawn(&world()).contains("wand_rung"), "{}", drawn(&world()));
}

#[test]
fn a_non_wand_draws_nothing_and_saves_as_before() {
    let content = Content::builtin().expect("content");
    let dagger = content.object_id("dagger").expect("a dagger");
    let mut stack = Stack::one(dagger);
    let before = serde_json::to_string(&stack).expect("json");
    let mut rng = crate::rng::Rng::stream(1, "wand-test");
    stack.draw_wand_rung(&content, &mut rng);
    assert_eq!(serde_json::to_string(&stack).expect("json"), before);
    assert_eq!(before, format!("{{\"object\":{},\"count\":1}}", dagger.0));
    assert_eq!(
        rng.next_u32(),
        crate::rng::Rng::stream(1, "wand-test").next_u32(),
        "no draw"
    );
    assert_eq!(stack.wand_rung(&content), None);
}

#[test]
fn a_wand_saved_without_a_rung_casts_at_rung_0_and_a_rung_is_saved() {
    let mut w = world();
    let content = w.content.clone();
    // A wand of a save made before rungs were drawn: no `wand_rung`, read as 0.
    let old = with_rung(&w.state.player().inventory[4], None);
    assert!(
        !serde_json::to_string(&old)
            .expect("json")
            .contains("wand_rung")
    );
    assert_eq!(old.wand_rung(&content), Some(0));
    w.player_mut().inventory[4] = with_rung(&old, Some(2));
    let before = w.state_hash();
    w.player_mut().inventory[4] = with_rung(&old, Some(1));
    assert_ne!(w.state_hash(), before, "hashed");
    let loaded = crate::save::decode(&crate::save::encode(&w), content.clone()).expect("decodes");
    let rung = |w: &World| w.state.player().inventory[4].wand_rung(&content);
    assert_eq!(rung(&loaded), Some(1));
    w.player_mut().inventory[4] = old;
    let loaded = crate::save::decode(&crate::save::encode(&w), content.clone()).expect("decodes");
    assert_eq!(rung(&loaded), Some(0));
}

#[test]
fn the_rung_shows_with_the_type_and_not_before() {
    let mut w = world();
    // The wand of fireball: the last stack, before and after the scroll is read.
    let rung_of = |w: &World| (w.state.player().inventory.last()?).known_wand_rung(w.lore());
    let rows = |w: &World| crate::dump::layer_rows(w, crate::dump::Layer::Inventory).join("\n");
    assert_eq!(rung_of(&w), None, "its type unknown");
    assert!(
        w.perception()
            .wand_does(&w.state.player().inventory[4])
            .is_none()
    );
    assert!(!rows(&w).contains("(rung"), "{}", rows(&w));
    let one = with_rung(&w.state.player().inventory[4], Some(1));
    w.player_mut().inventory[4] = one;
    run(&mut w, "read:d:e");
    assert_eq!(rung_of(&w), Some(1));
    assert!(rows(&w).contains("(rung 1)"), "{}", rows(&w));
    let view = w.perception();
    let stack = w.state.player().inventory.last().expect("the wand");
    assert_eq!(view.wand_does(stack).as_deref(), Some("3d6 fire, radius 2"));
    let lines: Vec<String> = (view.describe_item(stack).into_iter())
        .map(|l| l.text)
        .collect();
    assert!(
        lines
            .iter()
            .any(|l| l == "Holds Fireball: 3d6 fire, radius 2"),
        "{lines:?}"
    );
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
