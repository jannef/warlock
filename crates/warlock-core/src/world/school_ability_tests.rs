//! School abilities and the `spell_lands` trigger (spec §8 "Weapon abilities"; the spell scaling
//! design §8.5), each on a hand-made cast: an area lands on each it struck, in actor id order; a
//! buff lands on its caster (Aegis) and lays nothing meant for the other end on them; a hex that
//! took and a harm landed fire; a miss, a scroll, and a wand fire nothing; no draws without a
//! rank; Early mastery brings one a rank sooner; the news and the menu's row.

use super::caster_tests::{kobold, kobold_at, said, world_with};
use super::school_skill_tests::{cast, ranks, wizard};
use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::{Event, parse_actions};
use crate::combat::Strike;
use crate::content::{ContentFiles, StatusId};
use crate::stats::{PerStat, Stats};

/// The school abilities' files (`data/weapon_abilities/`).
const FILES: [&str; 4] = ["aegis", "lulling", "thunderstruck", "wasting"];

fn status(w: &World, id: &str) -> StatusId {
    w.content().status_id(id).expect(id)
}

/// The player's own statuses begun in `events` (creatures' are not events: see `suffers`).
fn began(events: &[Event], status: StatusId) -> Vec<ActorId> {
    (events.iter())
        .filter_map(|e| match *e {
            Event::Status {
                actor,
                status: s,
                began: true,
            } if s == status => Some(actor),
            _ => None,
        })
        .collect()
}

/// Whether actor `id` suffers status `name` now.
fn suffers(w: &World, id: ActorId, name: &str) -> bool {
    w.state.actors[&id].effects.has(status(w, name))
}

/// Keeps every creature from acting, so a status eased by actions stays to be seen.
fn freeze(w: &mut World) {
    let player = w.player_id();
    for actor in w.state.actors.values_mut() {
        if actor.id != player {
            actor.energy = -1_000_000_000;
        }
    }
}

#[test]
fn an_area_lands_on_each_it_struck_in_actor_id_order() {
    let mut w = wizard();
    // Made out of order on the map: the trigger's order is the ids'.
    let ids = [
        kobold_at(&mut w, (5, 1), 10_000),
        kobold_at(&mut w, (6, 0), 10_000),
        kobold_at(&mut w, (5, 0), 10_000),
    ];
    // Three kinds, so the lines say whose they are.
    let content = w.content().clone();
    let mut names = Vec::new();
    for (&id, kind) in ids.iter().zip(["rat", "kobold", "goblin"]) {
        let creature = content.creature_id(kind).expect(kind);
        w.state.actors.get_mut(&id).expect("there").creature = creature;
        let name = crate::content::the_start(&content.creature(creature).name);
        names.push(format!("{name} staggers."));
    }
    freeze(&mut w);
    ranks(&mut w, &[("evocation", 10)]);
    cast(&mut w, "fireball", Some((5, 0)));
    for id in ids {
        assert!(suffers(&w, id, "staggered"), "{id:?}: {:?}", said(&w));
    }
    let staggers: Vec<String> = (said(&w).into_iter())
        .filter(|m| m.ends_with(" staggers."))
        .collect();
    assert_eq!(staggers, names, "in actor id order");
    // A rank short of it: nothing.
    let mut w = wizard();
    let id = kobold(&mut w, 5, 10_000);
    freeze(&mut w);
    ranks(&mut w, &[("evocation", 9)]);
    cast(&mut w, "fireball", Some((5, 0)));
    assert!(!suffers(&w, id, "staggered"));
}

#[test]
fn a_harm_and_a_hex_that_took_lay_their_schools_statuses() {
    // Wasting: necromancy's harm leaves the struck weak.
    let mut w = wizard();
    let target = kobold(&mut w, 4, 10_000);
    ranks(&mut w, &[("necromancy", 10)]);
    cast(&mut w, "blight", Some((4, 0)));
    let weakened = status(&w, "weakened");
    assert!(w.state.actors[&target].effects.has(weakened));
    // Lulling: an enchantment that took leaves the creature sluggish too.
    let mut w = wizard();
    let target = kobold(&mut w, 3, 10_000);
    ranks(&mut w, &[("enchantment", 10)]);
    cast(&mut w, "evil_eye", Some((3, 0)));
    let sluggish = status(&w, "sluggish");
    assert!(w.state.actors[&target].effects.has(sluggish));
    assert!(said(&w).iter().any(|m| m == "The kobold is sluggish."));
    // Without the rank, Evil eye lays only its own.
    let mut w = wizard();
    let target = kobold(&mut w, 3, 10_000);
    ranks(&mut w, &[("enchantment", 9)]);
    cast(&mut w, "evil_eye", Some((3, 0)));
    assert!(!w.state.actors[&target].effects.has(sluggish));
}

#[test]
fn a_buff_lands_on_its_caster() {
    // Aegis: Blessing wards its caster.
    let mut w = wizard();
    ranks(&mut w, &[("abjuration", 7)]);
    let events = cast(&mut w, "blessing", None);
    let ward = status(&w, "spell_ward");
    let player = w.state.player;
    assert_eq!(began(&events, ward), [player]);
    // Thunderstruck is for what an evocation harms: Light staggers no caster.
    let mut w = wizard();
    ranks(&mut w, &[("evocation", 10)]);
    let events = cast(&mut w, "light", None);
    assert!(began(&events, status(&w, "staggered")).is_empty());
}

#[test]
fn a_miss_fires_nothing() {
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/spells/arcane_bolt.toml",
        "[[spell]]\nid = \"arcane_bolt\"\nname = \"Arcane bolt\"\ndescription = \"x\"\n\
         level = 6\nschool = \"evocation\"\nkind = \"damage\"\ndamage_type = \"force\"\n\
         shape = \"ray\"\nresolution = \"attack\"\nrange = 8\ndamage = \"3d6\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("the spell loads"));
    let mut w = world_with(content, "wizard");
    super::caster_tests::reach(&mut w, 18);
    let target = kobold(&mut w, 3, 10_000);
    // A nimble target, so the ray misses now and then.
    w.state.actors.get_mut(&target).expect("the kobold").stats =
        Stats::new(PerStat::splat(30), PerStat::splat(30));
    let staggered = status(&w, "staggered");
    freeze(&mut w);
    let (mut missed, mut hit) = (false, false);
    for _ in 0..200 {
        w.state
            .actors
            .get_mut(&target)
            .expect("there")
            .effects
            .remove(staggered);
        ranks(&mut w, &[("evocation", 10)]);
        let events = cast(&mut w, "arcane_bolt", Some((3, 0)));
        let strike = (events.iter()).find_map(|e| match e {
            Event::Attacked {
                target: t,
                strike: Some(s),
                ..
            } if *t == target => Some(*s),
            _ => None,
        });
        let fired = suffers(&w, target, "staggered");
        match strike {
            Some(Strike::Missed { .. }) => {
                missed = true;
                assert!(!fired, "a miss staggers nothing");
            }
            Some(_) => {
                hit = true;
                assert!(fired, "a hit staggers");
            }
            None => panic!("the ray found the kobold"),
        }
        if missed && hit {
            break;
        }
    }
    assert!(missed && hit, "both happened");
}

#[test]
fn a_scroll_or_a_wand_fires_nothing() {
    const HALL: &str =
        "#############\n#...........#\n#..@.r......#\n#...........#\n#############\n";
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
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 500\n\
         carry = [\"wand_of_magic_missile\", \"scroll_of_frost\"]\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let mut w = World::new(content, "hall", Some(3)).expect("world");
    ranks(&mut w, &[("evocation", 10)]);
    let rat = (w.state.actors.values())
        .find(|a| a.id != w.state.player)
        .map(|a| a.id)
        .expect("the rat");
    w.state.actors.get_mut(&rat).expect("the rat").hp = 1000;
    let at = w.state.actors[&rat].pos;
    freeze(&mut w);
    let actions = format!(
        "equip:missile_weapon:a fire:{},{} read:b:{},{}",
        at.x, at.y, at.x, at.y
    );
    for action in parse_actions(&actions).expect("actions") {
        let out = w.apply(&action);
        assert!(out.took_turn, "{:?}", said(&w));
        assert!(!suffers(&w, rat, "staggered"), "{action:?}");
    }
    assert!(w.state.actors[&rat].hp < 1000, "the rat was struck");
}

#[test]
fn no_draws_without_a_rank() {
    let run = |abilities: bool| {
        let mut files = ContentFiles::builtin();
        if !abilities {
            for id in FILES {
                files.remove(&format!("data/weapon_abilities/{id}.toml"));
            }
        }
        let content = Arc::new(Content::load(&files).expect("content"));
        assert_eq!(
            content.weapon_ability_id("thunderstruck").is_some(),
            abilities
        );
        let mut w = world_with(content, "wizard");
        super::caster_tests::reach(&mut w, 18);
        kobold(&mut w, 5, 10_000);
        kobold_at(&mut w, (5, 1), 10_000);
        freeze(&mut w);
        let mut hashes = Vec::new();
        for spell in ["fireball", "blight", "evil_eye", "blessing"] {
            ranks(
                &mut w,
                &[
                    ("evocation", 9),
                    ("necromancy", 9),
                    ("enchantment", 9),
                    ("abjuration", 6),
                ],
            );
            let at = (spell != "blessing").then_some((5, 0));
            cast(&mut w, spell, at);
            hashes.push(w.state_hash());
        }
        hashes
    };
    assert_eq!(run(true), run(false));
}

#[test]
fn early_mastery_brings_one_a_rank_sooner() {
    let mut w = wizard();
    let content = w.content().clone();
    let id = content
        .weapon_ability_id("thunderstruck")
        .expect("thunderstruck");
    let knows =
        |w: &World| (w.state.player().weapon_abilities(&content).iter()).any(|(_, a)| *a == id);
    ranks(&mut w, &[("evocation", 9)]);
    assert!(!knows(&w));
    w.player_mut().talents.take("early_mastery", None);
    assert!(knows(&w));
}

#[test]
fn the_news_and_the_menu_name_the_school() {
    let mut w = wizard();
    ranks(&mut w, &[("evocation", 9)]);
    let content = w.content().clone();
    w.apply(&Action::SetRank {
        skill: "evocation".into(),
        rank: 10,
    });
    assert!(
        (said(&w).iter())
            .any(|m| m == "You feel more skilled at evocation. You have learned Thunderstruck."),
        "{:#?}",
        said(&w)
    );
    let mine = w.perception().abilities();
    let row = (mine.iter())
        .find(|a| a.ability.id(&content) == "thunderstruck")
        .expect("listed");
    assert!(matches!(row.ability, Ability::Weapon(..)));
    assert_eq!(row.ability.name(&content), "Evocation: Thunderstruck");
    assert_eq!(row.unavailable, Some(Unavailable::FiresByItself));
}
