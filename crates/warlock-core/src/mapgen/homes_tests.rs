//! Homes of animated things (#487, animated objects A5): the armoury and the workshop, special
//! rooms with their own populations and their dressing. Racks and benches stand along the walls,
//! never beside a door or cutting the room apart; the sleepers come from the room's own
//! population whatever the branch's; a room is drawn only where its population weighs something;
//! and Fyrdelf's bottom-floor armouries are where panoplies stand.

use super::*;
use crate::content::{ContentFiles, Swarm};
use std::collections::{BTreeSet, VecDeque};
use std::sync::LazyLock;

/// Built-in content with two fixture branches of kobolds only, every room special and an armoury
/// or a workshop: one at danger 3, one at danger 30 (too deep for anything of a workshop).
static FIXTURE: LazyLock<Content> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    let branch = |id: &str, danger: u8| {
        format!(
            "\n[[dungeon]]\nid = \"{id}\"\ngenerator = \"rooms\"\ndepth = 1\nsize = [80, 21]\n\
             danger = {danger}\npopulation = [\"kobold\"]\n\
             room_effects = {{ chance = 100, per_level = 3, favoured = {{ armoury = 50, \
             workshop = 50 }} }}\nshops = {{ chance = 0 }}\n"
        )
    };
    let fixture = format!(
        "{}{}",
        branch("fixture_homes", 3),
        branch("fixture_deep_homes", 30)
    );
    let mut once = Some(fixture);
    files.edit_table("dungeons", |text| {
        format!("{text}{}", once.take().unwrap_or_default())
    });
    Content::load(&files).expect("fixture content")
});

fn effect(content: &Content, id: &str) -> RoomEffectId {
    content.room_effects().id(id).expect("effect")
}

fn swarm<'c>(content: &'c Content, id: &str) -> &'c Swarm {
    let def = content.room_effect(effect(content, id));
    def.swarm.as_ref().expect("a den")
}

/// The sleepers lying in `room` of `level`.
fn sleepers<'l>(level: &'l LevelTemplate, room: &SpecialRoom) -> Vec<&'l CreatureSpawn> {
    (level.creatures.iter())
        .filter(|c| c.asleep && room.contains(c.pos))
        .collect()
}

/// Checks every home on `level`: its dressing along its walls, never beside a door, its walkable
/// cells joined; its sleepers of its own population. Returns the homes' effects and sleepers.
fn check_homes(content: &Content, level: &LevelTemplate) -> Vec<(RoomEffectId, Vec<CreatureId>)> {
    let mut found = Vec::new();
    for room in &level.rooms {
        let def = content.room_effect(room.effect);
        let Some(dressing) = def.dressing else {
            continue;
        };
        let pieces: Vec<Pos> = (room.cells().into_iter())
            .filter(|p| level.terrain.get(*p) == Some(&dressing.terrain))
            .collect();
        assert!(!pieces.is_empty(), "{}: no dressing in {room:?}", def.id);
        let terrain = |p: Pos| level.terrain.get(p).map(|&t| content.terrain(t));
        for &p in &pieces {
            let walled = [Dir::N, Dir::E, Dir::S, Dir::W]
                .iter()
                .any(|&d| terrain(p.step(d)).is_some_and(|t| !t.walkable));
            assert!(walled, "{}: a piece at {p:?} stands off the walls", def.id);
            let by_door =
                (Dir::ALL.iter()).any(|&d| terrain(p.step(d)).is_some_and(|t| t.is_door()));
            assert!(
                !by_door,
                "{}: a piece at {p:?} stands beside a door",
                def.id
            );
        }
        let open: Vec<Pos> = (room.cells().into_iter())
            .filter(|&p| terrain(p).is_some_and(|t| t.walkable))
            .collect();
        assert!(joined(&open), "{}: its floor is cut apart", def.id);
        let swarm = def.swarm.as_ref().expect("a home is a den");
        let mut kinds = Vec::new();
        for c in sleepers(level, room) {
            let family = content.family(c.creature);
            assert!(
                swarm.kinds.contains(&family),
                "{}: {} is none of its own",
                def.id,
                content.creature(c.creature).id
            );
            kinds.push(c.creature);
        }
        found.push((room.effect, kinds));
    }
    found
}

/// Whether `cells` are all joined, 4-connected.
fn joined(cells: &[Pos]) -> bool {
    let Some(&first) = cells.first() else {
        return true;
    };
    let mut seen = BTreeSet::from([first]);
    let mut queue = VecDeque::from([first]);
    while let Some(p) = queue.pop_front() {
        for d in [Dir::N, Dir::E, Dir::S, Dir::W] {
            let q = p.step(d);
            if cells.contains(&q) && seen.insert(q) {
                queue.push_back(q);
            }
        }
    }
    seen.len() == cells.len()
}

#[test]
fn the_homes_name_their_own_populations_and_dressing() {
    let content = &*FIXTURE;
    let family = |id: &str| content.creature_id(id).map(|c| content.family(c));
    let armoury = swarm(content, "armoury");
    for id in ["animated_blade", "animated_armour", "animated_panoply"] {
        assert!(armoury.kinds.contains(&family(id).expect(id)), "{id}");
    }
    assert_eq!(armoury.danger, 4);
    let workshop = swarm(content, "workshop");
    assert!(
        workshop
            .kinds
            .contains(&family("animated_broom").expect("broom"))
    );
    assert!((workshop.kinds.iter()).all(|&c| content.creature(c).animated.is_some()));
    let rack = content.terrain_id("weapon_rack").expect("rack");
    let bench = content.terrain_id("workbench").expect("bench");
    let dressing = |id: &str| content.room_effect(effect(content, id)).dressing;
    assert_eq!(dressing("armoury").map(|d| d.terrain), Some(rack));
    assert_eq!(dressing("workshop").map(|d| d.terrain), Some(bench));
}

/// A branch of kobolds only still gets armouries and workshops full of animated things: a
/// home's sleepers are its own population's, whatever the branch's.
#[test]
fn homes_in_a_branch_of_kobolds_hold_their_own_sleepers() {
    let content = &*FIXTURE;
    let (armoury, workshop) = (effect(content, "armoury"), effect(content, "workshop"));
    let mut seen = BTreeMap::<RoomEffectId, usize>::new();
    for seed in 0..40 {
        let level = generate(content, ("showcase", seed), "fixture_homes:1", None).expect("gen");
        for (effect, kinds) in check_homes(content, &level) {
            assert!(!kinds.is_empty(), "seed {seed}: an empty home");
            *seen.entry(effect).or_default() += kinds.len();
        }
    }
    assert!(seen.get(&armoury).is_some_and(|&n| n > 10), "{seen:?}");
    assert!(seen.get(&workshop).is_some_and(|&n| n > 10), "{seen:?}");
}

/// Too deep for every household thing (danger 30, all of them more than eight levels weaker), a
/// workshop is never drawn; an armoury still is (its families reach level 70).
#[test]
fn a_home_is_drawn_only_where_its_population_weighs_something() {
    let content = &*FIXTURE;
    let (armoury, workshop) = (effect(content, "armoury"), effect(content, "workshop"));
    let mut armouries = 0;
    for seed in 0..20 {
        let id = "fixture_deep_homes:1";
        let level = generate(content, ("showcase", seed), id, None).expect("gen");
        assert!(
            level.rooms.iter().all(|r| r.effect != workshop),
            "seed {seed}"
        );
        armouries += (level.rooms.iter()).filter(|r| r.effect == armoury).count();
        check_homes(content, &level);
    }
    assert!(armouries > 0);
}

/// Fyrdelf favours armouries; on its bottom floor (danger 16, drawn at 20) panoplies stand among
/// the hollow armour and the blades.
#[test]
fn fyrdelf_bottom_armouries_hold_panoplies() {
    let content = &*FIXTURE;
    let armoury = effect(content, "armoury");
    let panoply = content.creature_id("animated_panoply").expect("panoply");
    let (mut rooms, mut panoplies) = (0, 0);
    for seed in 1..=60 {
        // A seed whose bottom floor finds no valid map (seed 0 already did before #487) is
        // left out.
        let Ok(level) = generate(content, ("ascmearc", seed), "fyrdelf:12", Some("ascmearc"))
        else {
            continue;
        };
        for (effect, kinds) in check_homes(content, &level) {
            if effect != armoury {
                continue;
            }
            rooms += 1;
            panoplies += (kinds.iter())
                .filter(|&&c| content.family(c) == panoply)
                .count();
        }
    }
    assert!(
        rooms > 0 && panoplies > 0,
        "{rooms} armouries, {panoplies} panoplies"
    );
}
