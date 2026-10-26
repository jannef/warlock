//! Halls of mirrors (spec §7 "Special rooms"; `world/mirrors.rs`): the images of everyone inside,
//! where they stand, what the player sees of them, the glass a blow meets, a missile flying on
//! through one, the creatures' aim from outside, Mirror-wise, and the `actors` dump.

use super::*;
use crate::action::{Event, parse_actions};
use crate::content::ContentFiles;
use crate::dump::{Layer, layer_rows};
use crate::object::Stack;
use std::collections::BTreeSet;

/// A hall, 31 wide, fully lit: the east part (x 14 to 24, walls included) a hall of mirrors with
/// a peaceful kobold at `k` inside it, another at `j` outside it.
const HALL: &str = "\
###############################
#.............................#
#.............................#
#...@.....j........k..........#
#.............................#
#.............................#
###############################
";

/// The hall's floor inside.
fn in_hall(p: Pos) -> bool {
    (15..=23).contains(&p.x) && (1..=5).contains(&p.y)
}

fn world() -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_mirrors.toml",
        &format!(
            "name = \"Test\"\nambient = [160, 160, 160, 0]\nmap = '''\n{HALL}'''\n\
             [[room]]\neffect = \"mirrors\"\narea = [14, 0, 24, 6]\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"k\" = {{ terrain = \"floor\", creature = \"kobold\", faction = \"peaceful\" }}\n\
             \"j\" = {{ terrain = \"floor\", creature = \"kobold\", faction = \"peaceful\" }}\n"
        ),
    );
    files.insert(
        "scenarios/test_mirrors.toml",
        "name = \"T\"\nstart_level = \"test_mirrors\"\nhp = 1000\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_mirrors", None).expect("world")
}

/// The kobold that started at column `x` (they are peaceful and keep still enough: found by id).
fn kobold(w: &World, x: i32) -> ActorId {
    let mut kobolds: Vec<&Actor> = (w.state.actors.values())
        .filter(|a| a.kind == ActorKind::Creature)
        .collect();
    kobolds.sort_by_key(|a| a.id);
    // `j` (x 10) comes before `k` (x 19) in the map's reading order.
    let i = usize::from(x != 10);
    kobolds[i].id
}

/// The player stands at `p` (inside the hall, say), and perceives it.
fn put_player(w: &mut World, p: Pos) {
    w.player_mut().pos = p;
    w.apply(&Action::Wait);
}

fn run(w: &mut World, text: &str) -> Vec<Outcome> {
    let actions = parse_actions(text).expect("actions");
    actions.iter().map(|a| w.apply(a)).collect()
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

#[test]
fn the_hall_of_mirrors_is_data() {
    let content = Content::builtin().expect("content");
    let effects = content.room_effects();
    let hall = effects.get(effects.id("mirrors").expect("mirrors"));
    assert_eq!(hall.name, "hall of mirrors");
    assert_eq!(hall.mark, 'm');
    assert_eq!(
        hall.enter,
        "Your reflection stares back at you from every side."
    );
    assert_eq!((hall.weight, hall.max_cells), (3, Some(80)));
    assert_eq!(hall.mirrors.map(|m| m.images), Some(2));
    assert!(!hall.can_pervade() && hall.level_chance == 0);
}

#[test]
fn everyone_inside_has_two_images_on_free_floor_of_the_hall() {
    let w = world();
    let k = kobold(&w, 19);
    let images = w.images();
    // The player and `j` stand outside: only `k` has images.
    assert_eq!(images.len(), 2, "{images:?}");
    assert!(images.iter().all(|i| i.of == k));
    let cells: BTreeSet<Pos> = images.iter().map(|i| i.pos).collect();
    assert_eq!(cells.len(), 2, "distinct");
    let level = w.state.current();
    for i in &images {
        assert!(in_hall(i.pos), "{i:?}");
        assert!(level.is_walkable(&w.content, i.pos));
        assert!(w.state.actor_at(&level.id, i.pos).is_none());
    }
    // The player sees the kobold three times over, alike in all but where they stand.
    let view = w.perception();
    let real = *view.actors().iter().find(|a| a.id == k).expect("k seen");
    let kobolds: Vec<_> = (view.actors().iter())
        .filter(|a| a.creature == real.creature && a.id != kobold(&w, 10))
        .collect();
    assert_eq!(kobolds.len(), 3, "{kobolds:?}");
    for shown in &kobolds {
        assert_eq!(
            (shown.kind, shown.creature, shown.health, shown.faction),
            (real.kind, real.creature, real.health, real.faction)
        );
        assert!(shown.id == k || !w.state.actors.contains_key(&shown.id));
    }
    let shown: BTreeSet<Pos> = (kobolds.iter())
        .filter(|a| a.id != k)
        .map(|a| a.pos)
        .collect();
    assert_eq!(shown, cells);
    // Ids are unique, and still in ascending order.
    let ids: Vec<ActorId> = view.actors().iter().map(|a| a.id).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(ids, sorted);
}

#[test]
fn images_move_with_the_turns_and_are_the_same_however_asked() {
    let mut w = world();
    let mut seen = BTreeSet::new();
    for _ in 0..12 {
        let images = w.images();
        assert_eq!(
            images,
            w.images(),
            "no draw: the same each time it is asked"
        );
        let state = w.state.images(&w.content, w.state.current());
        assert_eq!(images, state);
        seen.insert(images.iter().map(|i| i.pos).collect::<Vec<_>>());
        w.apply(&Action::Wait);
    }
    assert!(seen.len() > 3, "they move from turn to turn: {seen:?}");
    // And nothing was drawn for them: the same walk elsewhere hashes alike.
    let (mut a, mut b) = (world(), world());
    run(&mut a, "wait*5");
    run(&mut b, "wait*5");
    assert_eq!(a.state_hash(), b.state_hash());
}

#[test]
fn one_outside_the_hall_has_none() {
    let mut w = world();
    let k = kobold(&w, 19);
    w.state.actors.get_mut(&k).expect("k").pos = Pos::new(12, 3);
    w.apply(&Action::Wait);
    assert!(w.images().is_empty() || w.images().iter().all(|i| i.of != k));
    let kobolds = (w.perception().actors().iter())
        .filter(|a| a.kind == ActorKind::Creature)
        .count();
    assert_eq!(kobolds, 2, "the two kobolds, no reflections");
}

#[test]
fn the_player_sees_their_own_reflection() {
    let mut w = world();
    put_player(&mut w, Pos::new(16, 3));
    let view = w.perception();
    let me: Vec<_> = (view.actors().iter())
        .filter(|a| a.kind == ActorKind::Player)
        .collect();
    assert_eq!(me.len(), 3, "{me:?}");
    assert!(said(
        &w,
        "Your reflection stares back at you from every side."
    ));
    // Creatures in view are the real ones only (a rest's disturbance reads them).
    let real: Vec<ActorId> = w.creatures_in_view();
    assert!(real.iter().all(|id| w.state.actors.contains_key(id)));
    assert_eq!(real.len(), 2);
}

/// Waits until the player sees an image `near` them that `fits`; panics after 200 turns.
fn wait_for_image(w: &mut World, fits: impl Fn(&World, Pos) -> bool) -> Pos {
    for _ in 0..200 {
        let view = w.perception();
        let found = (view.actors().iter())
            .filter(|a| !w.state.actors.contains_key(&a.id))
            .map(|a| a.pos)
            .find(|&p| fits(w, p));
        if let Some(p) = found {
            return p;
        }
        w.apply(&Action::Wait);
    }
    panic!("no image came where it was wanted");
}

#[test]
fn a_step_into_an_image_meets_only_glass() {
    let mut w = world();
    put_player(&mut w, Pos::new(18, 3));
    let me = w.player_pos();
    let image = wait_for_image(&mut w, |w, p| p.chebyshev(w.player_pos()) == 1);
    assert_eq!(w.player_pos(), me);
    let dir = Dir::ALL
        .into_iter()
        .find(|&d| me.step(d) == image)
        .expect("a direction");
    let before = w.time();
    w.apply(&Action::Move(dir));
    assert_eq!(w.player_pos(), me, "no step: the glass is in the way");
    assert!(said(&w, "Your blow meets only glass."));
    assert!(w.time() > before, "the blow's time");
}

#[test]
fn a_missile_aimed_at_an_image_flies_on_through_it() {
    let mut w = world();
    let dagger = w.content.object_id("dagger").expect("dagger");
    w.player_mut().inventory.push(Stack::one(dagger));
    let letter = w.state.player().inventory.len() - 1;
    put_player(&mut w, Pos::new(16, 3));
    // An image a few cells off, with open floor beyond it on the dagger's line.
    let image = wait_for_image(&mut w, |w, p| {
        let me = w.player_pos();
        let far = p.chebyshev(me);
        let beyond = me.ray(p).nth(usize::try_from(far).unwrap_or(0));
        (2..=3).contains(&far)
            && beyond.is_some_and(|q| {
                in_hall(q) && w.state.actor_at(&w.state.current_level, q).is_none()
            })
    });
    assert!(w.reflected(w.state.player, image));
    let outs = run(&mut w, &format!("throw:{letter}:{},{}", image.x, image.y));
    let path: Vec<Pos> = (outs.iter().flat_map(|o| o.events.iter()))
        .find_map(|e| match e {
            Event::Missile { path, .. } => Some(path.clone()),
            _ => None,
        })
        .expect("the dagger flew");
    assert!(path.contains(&image), "{path:?}");
    assert_ne!(path.last(), Some(&image), "it flew on: {path:?}");
}

#[test]
fn creatures_outside_aim_at_the_player_or_an_image() {
    let mut w = world();
    put_player(&mut w, Pos::new(16, 3));
    let (player, me) = (w.state.player, w.player_pos());
    let (j, k) = (kobold(&w, 10), kobold(&w, 19));
    assert!(!in_hall(w.state.actors[&j].pos) && in_hall(w.state.actors[&k].pos));
    let mine: BTreeSet<Pos> = (w.images().iter())
        .filter(|i| i.of == player)
        .map(|i| i.pos)
        .collect();
    assert_eq!(mine.len(), 2);
    let mut counts = std::collections::BTreeMap::new();
    for _ in 0..300 {
        *counts.entry(w.aim_at(j, player, me)).or_insert(0) += 1;
    }
    let mut cells: BTreeSet<Pos> = mine.clone();
    cells.insert(me);
    assert_eq!(counts.keys().copied().collect::<BTreeSet<_>>(), cells);
    assert!(
        counts.values().all(|&n| n > 60),
        "about a third each: {counts:?}"
    );
    // From inside the hall, a creature is not fooled; nor is anyone aiming at a creature.
    for _ in 0..50 {
        assert_eq!(w.aim_at(k, player, me), me);
        let at = w.state.actors[&k].pos;
        assert_eq!(w.aim_at(j, k, at), at);
    }
}

#[test]
fn mirror_wise_sees_near_reflections_for_what_they_are() {
    let mut w = world();
    w.player_mut().talents.take("mirror_wise", None);
    put_player(&mut w, Pos::new(18, 3));
    let mut near_truth = 0;
    for _ in 0..30 {
        let me = w.player_pos();
        near_truth += (w.images().iter())
            .filter(|i| i.pos.chebyshev(me) <= 2)
            .count();
        let view = w.perception();
        let shown: Vec<Pos> = (view.actors().iter())
            .filter(|a| !w.state.actors.contains_key(&a.id))
            .map(|a| a.pos)
            .collect();
        assert!(shown.iter().all(|p| p.chebyshev(me) > 2), "{shown:?}");
        let far = (w.images().iter())
            .filter(|i| i.pos.chebyshev(me) > 2)
            .count();
        assert_eq!(shown.len(), far, "those farther are shown");
        w.apply(&Action::Wait);
    }
    assert!(near_truth > 0, "images did come near");
}

#[test]
fn the_actors_dump_marks_images_and_the_view_shows_them() {
    let w = world();
    let rows = layer_rows(&w, Layer::Actors);
    let k = kobold(&w, 19);
    let row = (rows.iter())
        .find(|r| r.starts_with(&format!("{k} kobold")))
        .expect("k's row");
    assert!(row.contains("; images ("), "{row}");
    let view = layer_rows(&w, Layer::View);
    let kobolds: usize = view.iter().map(|r| r.matches('k').count()).sum();
    assert_eq!(
        kobolds,
        4,
        "both kobolds and k's two images:\n{}",
        view.join("\n")
    );
}

#[test]
fn resting_among_reflections_is_quiet() {
    let mut w = world();
    put_player(&mut w, Pos::new(16, 3));
    run(&mut w, "rest:10m");
    assert!(w.state.death.is_none());
}
