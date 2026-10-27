//! Animated objects in play (spec §8 "Animated objects", #483): a household thing made of its
//! body (placed by a legend, or composed from its own stream), what it shows, its immunities,
//! stealing refused, the drop rule with Salvager, Unbinder, saves, and replays.

use super::fight::Harm;
use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::geom::Dir;
use crate::replay::Replay;

// A lit hall: the player between an animated candlestick of silver (west) and an animated teapot
// (east), a rug further on, and a peaceful kobold below.
const HALL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
#########
#.......#
#.c@t.r.#
#...k...#
#########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"c" = { terrain = "floor", creature = "animated_candlestick", animated = { material = "silver" } }
"t" = { terrain = "floor", creature = "animated_teapot" }
"r" = { terrain = "floor", creature = "animated_rug" }
"k" = { terrain = "floor", creature = "kobold", faction = "peaceful" }
"##;

/// The built-in content with the hall, its `[drops] intact` set to `intact`.
fn files(intact: u8) -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/hall.toml", HALL);
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\n",
    );
    let rules = include_str!("../../../../assets/data/animated.toml")
        .replace("intact = 50 ", &format!("intact = {intact} "));
    files.insert("data/animated.toml", &rules);
    files
}

fn content(intact: u8) -> Arc<Content> {
    Arc::new(Content::load(&files(intact)).expect("test content is valid"))
}

fn hall(intact: u8) -> World {
    World::new(content(intact), "hall", None).expect("run")
}

fn at(w: &World, p: Pos) -> ActorId {
    let level = &w.state.current_level;
    w.state.actor_at(level, p).expect("someone there").id
}

fn last(w: &World) -> String {
    (w.messages().iter().last()).map_or(String::new(), |m| m.text.clone())
}

#[test]
fn a_legend_places_its_body_of_its_material() {
    let w = hall(50);
    let content = w.content();
    let candlestick = &w.state.actors[&at(&w, Pos::new(2, 2))];
    let object = content.object_id("candlestick").expect("the object");
    let body = candlestick.inventory.first().expect("a body");
    assert_eq!(body.object, object);
    assert_eq!(candlestick.inventory.len(), 1, "nothing else");
    assert!(candlestick.equipment.items().is_empty(), "in no slot");
    let silver = content.material_id("silver").expect("silver");
    assert_eq!(body.parts(content)[0].material, silver);
    // Seen by what it is made of: the candlestick's glyph in silver's colour.
    let view = w.perception();
    let seen = view.actor_at(Pos::new(2, 2)).expect("seen");
    let shape = seen.shape.expect("a shape");
    assert_eq!(shape.material, silver);
    assert_eq!(seen.glyph(content), content.object(object).glyph);
    assert_eq!(seen.fg(content), body.fg(content));
    assert_ne!(
        seen.fg(content),
        content.creature(seen.creature).fg,
        "silver, not brass"
    );
    // Anything else looks as its type does.
    assert!(
        view.actor_at(Pos::new(4, 3))
            .expect("the kobold")
            .shape
            .is_none()
    );
}

#[test]
fn a_body_is_composed_from_the_actors_own_stream() {
    let (a, b) = (hall(50), hall(50));
    let teapot = at(&a, Pos::new(4, 2));
    let content = a.content();
    let object = content.object_id("teapot").expect("the object");
    let body = &a.state.actors[&teapot].inventory;
    assert_eq!(body.len(), 1);
    assert_eq!(body[0].object, object);
    assert_eq!(
        body, &b.state.actors[&teapot].inventory,
        "the same every time"
    );
    // Its stream is `animated/<actor id>`: one draw for the material, as gear draws one.
    let creature = a.state.actors[&teapot].creature;
    let mut rng = crate::animated::stream(a.state.seed, &teapot.0.to_string());
    let made = crate::animated::compose(content, &mut rng, creature).expect("a body");
    assert_eq!(made, body[0]);
    // A type that is not animated has none.
    let kobold = a.state.actors[&at(&a, Pos::new(4, 3))].creature;
    assert!(crate::animated::compose(content, &mut rng, kobold).is_none());
}

#[test]
fn materials_are_drawn_by_the_gear_weights() {
    // Over many streams a candlestick is mostly iron (300) and brass (30), now and then pewter
    // (30), rarely silver (5).
    let w = hall(50);
    let content = w.content();
    let creature = content.creature_id("animated_candlestick").expect("type");
    let mut counts: std::collections::BTreeMap<String, u32> = Default::default();
    for key in 0..2000 {
        let mut rng = crate::animated::stream(7, &key.to_string());
        let body = crate::animated::compose(content, &mut rng, creature).expect("a body");
        let material = content.material(body.parts(content)[0].material);
        *counts.entry(material.id.clone()).or_default() += 1;
    }
    let n = |m: &str| counts.get(m).copied().unwrap_or(0);
    assert!(
        n("iron") > n("brass") && n("brass") > n("silver"),
        "{counts:?}"
    );
    assert!(n("silver") > 0, "{counts:?}");
}

#[test]
fn a_clatter_of_cutlery_is_three_to_six_pieces() {
    let w = hall(50);
    let content = w.content();
    let creature = content.creature_id("animated_cutlery").expect("type");
    let counts: std::collections::BTreeSet<u32> = (0..200)
        .map(|key| {
            let mut rng = crate::animated::stream(3, &format!("{key}"));
            (crate::animated::compose(content, &mut rng, creature).expect("a body")).count
        })
        .collect();
    assert_eq!(counts, (3..=6).collect());
}

#[test]
fn animated_things_are_never_poisoned_bleeding_or_diseased() {
    let mut w = hall(50);
    let content = w.content().clone();
    let candlestick = at(&w, Pos::new(2, 2));
    let kobold = at(&w, Pos::new(4, 3));
    let status = |id: &str| content.status_id(id).expect("status");
    let disease = content.disease_id("filth_fever").expect("disease");
    for id in [candlestick, kobold] {
        for s in ["poisoned", "bleeding", "frightened"] {
            w.afflict(id, status(s), 3, None, &mut Vec::new());
        }
        // Exposed until it catches it, or always refused.
        let caught = (0..50).any(|_| w.expose(id, disease, None));
        let has = |s: &str| w.state.actors[&id].effects.has(status(s));
        if id == candlestick {
            assert!(!has("poisoned") && !has("bleeding"), "immune");
            assert!(has("frightened"), "only those three");
            assert!(!caught, "nothing lives in it to sicken");
        } else {
            assert!(
                has("poisoned") && has("bleeding"),
                "the kobold suffers them"
            );
            assert!(caught, "and catches the disease in the end");
        }
    }
}

#[test]
fn stealing_from_one_is_refused_and_takes_no_time() {
    let spec = "human,female,thief".parse().expect("character");
    let mut w = World::with_character(content(50), "hall", None, Some(&spec)).expect("world");
    let turn = w.state.turn;
    w.apply(&Action::Steal(Dir::W));
    assert_eq!(last(&w), "It is all of a piece.");
    assert_eq!(w.state.turn, turn);
    assert_eq!(w.state.actors[&at(&w, Pos::new(2, 2))].inventory.len(), 1);
}

/// Kills the actor at `p` by the player's hand, and returns what lies there.
fn kill_at(w: &mut World, p: Pos) -> Vec<crate::object::Stack> {
    let id = at(w, p);
    let player = w.player_id();
    assert!(
        w.hurt(id, 10_000, Harm::Actor(player), &mut Vec::new()),
        "it died"
    );
    w.state.current().objects_at(p).to_vec()
}

#[test]
fn the_body_falls_whole_by_the_drop_rule_or_not_at_all() {
    let content = content(100);
    let object = content.object_id("candlestick").expect("the object");
    let mut w = hall(100);
    let pile = kill_at(&mut w, Pos::new(2, 2));
    assert_eq!(pile.len(), 1);
    assert_eq!(pile[0].object, object, "whole, of silver still");
    assert_eq!(
        pile[0].parts(&content)[0].material,
        content.material_id("silver").expect("silver")
    );
    // No corpse, no loot: nothing else.
    let mut w = hall(0);
    for p in [Pos::new(2, 2), Pos::new(4, 2), Pos::new(6, 2)] {
        assert!(kill_at(&mut w, p).is_empty(), "nothing falls");
    }
}

#[test]
fn salvager_raises_the_share_that_falls_whole() {
    // At 70 %, two ranks of Salvager (15 each) make it certain.
    let falls = |ranks: u8| {
        let mut whole = 0;
        for seed in 0..40 {
            let mut w = World::new(content(70), "hall", Some(seed)).expect("run");
            for _ in 0..ranks {
                w.player_mut().talents.take("salvager", None);
            }
            whole += [Pos::new(2, 2), Pos::new(4, 2), Pos::new(6, 2)]
                .into_iter()
                .filter(|&p| !kill_at(&mut w, p).is_empty())
                .count();
        }
        whole
    };
    assert_eq!(falls(2), 120, "every one");
    let without = falls(0);
    assert!((60..120).contains(&without), "about 70 % of 120: {without}");
}

#[test]
fn unbinder_strikes_surer_and_harder_at_animated_things() {
    let mut w = hall(50);
    let player = w.player_id();
    let (candlestick, kobold) = (at(&w, Pos::new(2, 2)), at(&w, Pos::new(4, 3)));
    let profile = w.state.player().melee(w.content());
    assert_eq!(w.bane(player, candlestick, profile.clone()), profile);
    w.player_mut().talents.take("unbinder", None);
    let bane = w.bane(player, candlestick, profile.clone());
    assert_eq!(
        (bane.to_hit, bane.bonus),
        (profile.to_hit + 1, profile.bonus + 2)
    );
    assert_eq!(
        w.bane(player, kobold, profile.clone()),
        profile,
        "a kobold is no thing"
    );
}

#[test]
fn an_animated_thing_survives_a_save_and_a_replay() {
    let w = hall(50);
    let loaded = crate::save::decode(&crate::save::encode(&w), content(50)).expect("decodes");
    assert_eq!(loaded.state_hash(), w.state_hash());
    let candlestick = at(&loaded, Pos::new(2, 2));
    assert_eq!(
        loaded.state.actors[&candlestick].inventory,
        w.state.actors[&candlestick].inventory
    );
    // A fight with them, replayed.
    let mut w = hall(50);
    let mut replay = Replay::new(&w);
    for action in parse_actions("move:W move:W wait*3 move:E move:E").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let (again, divergence) = replay.play(content(50)).expect("plays");
    assert!(divergence.is_none(), "the replay verifies");
    assert_eq!(again.state_hash(), w.state_hash());
}
