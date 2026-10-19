//! Blood in the world (spec §8 "Blood"): big hits and kills stain the floor and the walls next to
//! the victim, never water; stains add up, are saved and remembered, and are perceived only
//! where the player sees them; and they never change what happens in a fight.

use super::*;
use crate::action::{AttackKind, parse_actions};
use crate::blood::{CAP, RED, Stain};
use crate::combat::{Blow, Strike};
use crate::content::ContentFiles;
use crate::geom::Dir;
use std::sync::LazyLock;

/// A lit test level beside the built-in content: the player (`@`), a giant rat (`r`, 12 hit
/// points, small, peaceful) with a pillar east of it, a cave beetle (`a`, ichor) in a pool of
/// still water (`~`), and a closed door (`+`) to a passage east.
const MAP: &str = "\
##############
#..........#.#
#.@r#......#.#
#..........#.#
#..~~a~~...+.#
#..~~~~~...#.#
#..........#.#
##############
";

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/bloodtest.toml",
        &format!(
            "name = \"Blood\"\nambient = [255, 255, 255, 0]\nmap = '''\n{MAP}'''\n\n\
             [legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"r\" = {{ terrain = \"floor\", creature = \"rat\", faction = \"peaceful\" }}\n\
             \"a\" = {{ terrain = \"floor\", creature = \"beetle\" }}\n"
        ),
    );
    files.insert(
        "scenarios/bloodtest.toml",
        "name = \"Blood\"\nstart_level = \"bloodtest\"\n",
    );
    files
}

/// The built-in creature types, as written.
const CREATURES: &str = include_str!("../../../../assets/data/creatures.toml");

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

fn world() -> World {
    World::new(CONTENT.clone(), "bloodtest", None).expect("world")
}

/// The actor of creature type `id`.
fn actor_of(w: &World, id: &str) -> ActorId {
    let creature = w.content().creature_id(id).expect(id);
    (w.state().actors.values())
        .find(|a| a.creature == creature)
        .map(|a| a.id)
        .expect("on the level")
}

fn hit(damage: i32, critical: bool) -> Blow {
    Blow {
        strike: Strike::Hit { critical },
        damage,
    }
}

fn blood(w: &World) -> &BTreeMap<Pos, Stain> {
    &w.state().current().blood
}

/// Blood for a blow by the player, from where they stand, on `victim`, `times` times, each a
/// second later (another pattern each time).
fn strike(w: &mut World, victim: ActorId, blow: Blow, times: u64) {
    let player = w.state().player;
    let from = w.player_pos();
    for _ in 0..times {
        w.state.time += time::Duration::seconds(1);
        w.bleed((player, victim), from, AttackKind::Melee, blow);
    }
}

#[test]
fn big_hits_and_kills_bleed_lesser_blows_do_not() {
    let mut w = world();
    let rat = actor_of(&w, "rat");
    // The rat has 12 hit points: 3 is a lesser blow, 4 (a third) a big hit, 12 a kill.
    strike(&mut w, rat, hit(3, false), 1);
    assert!(blood(&w).is_empty());
    strike(&mut w, rat, hit(4, false), 1);
    let spatter: u32 = blood(&w).values().map(|s| u32::from(s.amount)).sum();
    assert!(spatter > 0);
    // Any confirmed critical bleeds.
    let mut w = world();
    strike(&mut w, rat, hit(1, true), 1);
    assert!(!blood(&w).is_empty());
    // A kill spills much more, most of it on the victim's cell.
    let mut w = world();
    strike(&mut w, rat, hit(12, false), 1);
    let at = w.state().actors[&rat].pos;
    let pool = blood(&w)[&at].amount;
    let kill: u32 = blood(&w).values().map(|s| u32::from(s.amount)).sum();
    assert!(kill > 2 * spatter, "{kill} vs {spatter}");
    assert!(blood(&w).values().all(|s| s.amount <= pool));
    assert!(blood(&w).values().all(|s| s.color == RED));
}

#[test]
fn a_kill_against_a_wall_paints_its_face_and_nothing_behind_it() {
    let mut w = world();
    let rat = actor_of(&w, "rat");
    let at = w.state().actors[&rat].pos;
    let wall = at.step(Dir::E);
    // Struck from the west, ten times over: the wall east of the rat takes blood on its west
    // face; nothing lands past it, and nothing behind the player but flecks next to the rat.
    strike(&mut w, rat, hit(30, true), 10);
    let stain = blood(&w).get(&wall).expect("the wall behind the victim");
    assert!(stain.on_face(Dir::W) && !stain.on_face(Dir::E));
    for (p, stain) in blood(&w) {
        assert!(p.x >= at.x - 1, "{p} behind the attacker");
        let opaque = w.state().current().is_opaque(&CONTENT, *p);
        assert_eq!(opaque, stain.faces != 0, "{p}: faces on walls only");
        assert!(
            !opaque || p.chebyshev(at) == 1,
            "{p}: only walls next to the victim"
        );
    }
    // Straight behind the pillar, nothing.
    assert!(!blood(&w).contains_key(&wall.step(Dir::E)));
    assert!(!blood(&w).contains_key(&wall.step(Dir::E).step(Dir::E)));
    // Repeated violence adds up, to a cap.
    assert_eq!(blood(&w)[&at].amount, CAP);
}

#[test]
fn water_takes_no_blood_and_ichor_is_its_own_colour() {
    let mut w = world();
    let beetle = actor_of(&w, "beetle");
    let at = w.state().actors[&beetle].pos;
    let ichor = w
        .content()
        .creature(w.state().actors[&beetle].creature)
        .blood;
    assert!(ichor.is_some_and(|c| c != RED));
    strike(&mut w, beetle, hit(40, true), 10);
    let level = w.state().current();
    assert!(!blood(&w).is_empty());
    for (p, stain) in blood(&w) {
        let terrain = CONTENT.terrain(level.terrain[*p]);
        assert!(terrain.stains, "{p}: {} takes no blood", terrain.id);
        assert_eq!(Some(stain.color), ichor);
    }
    // The beetle stands on floor amid the pool: it bleeds there, and on the dry cells around.
    assert!(blood(&w).contains_key(&at));
    assert!(
        blood(&w)
            .keys()
            .all(|p| CONTENT.terrain(level.terrain[*p]).id != "water")
    );
}

#[test]
fn bloodless_creatures_leave_none_and_bad_blood_is_refused() {
    let mut files = files();
    let creatures = CREATURES;
    files.insert(
        "data/creatures.toml",
        &creatures.replace("id = \"rat\"\n", "id = \"rat\"\nblood = \"none\"\n"),
    );
    let content = Arc::new(Content::load(&files).expect("valid"));
    let mut w = World::new(content, "bloodtest", None).expect("world");
    let rat = actor_of(&w, "rat");
    strike(&mut w, rat, hit(30, true), 3);
    assert!(blood(&w).is_empty());
    files.insert(
        "data/creatures.toml",
        &creatures.replace("id = \"rat\"\n", "id = \"rat\"\nblood = \"crimson\"\n"),
    );
    let e = Content::load(&files).expect_err("not a colour");
    assert!(e.to_string().contains("blood"), "{e}");
}

#[test]
fn walls_show_blood_only_on_a_face_the_player_sees() {
    let mut w = world();
    let rat = actor_of(&w, "rat");
    let wall = w.state().actors[&rat].pos.step(Dir::E);
    strike(&mut w, rat, hit(30, false), 1);
    w.apply(&Action::Wait);
    assert!(w.perception().blood(wall).is_some(), "seen from the west");
    // Round the wall to its east side: the wall is in view, its stained face is not.
    for action in parse_actions("move:SE move:E move:NE").expect("walk") {
        w.apply(&action);
    }
    assert_eq!(w.player_pos(), wall.step(Dir::E));
    assert!(w.perception().is_visible(wall));
    assert!(w.perception().blood(wall).is_none(), "the clean side");
    // It is still remembered as seen.
    assert!(w.state().current().blood_memory.contains_key(&wall));
}

#[test]
fn blood_is_remembered_as_last_seen() {
    let mut w = world();
    // Behind the closed door: blood the player has not seen is not perceived.
    let door = Pos::new(11, 4);
    let behind = door.step(Dir::E);
    let stain = Stain::new(20, RED, w.time(), None);
    w.state.current_mut().blood.insert(behind, stain);
    w.apply(&Action::Wait);
    assert!(w.perception().blood(behind).is_none());
    // Walk to the door and open it: now it is seen, and remembered.
    for action in parse_actions("move:SE move:E*7 move:S").expect("walk") {
        w.apply(&action);
    }
    assert_eq!(w.player_pos(), door.step(Dir::W));
    w.apply(&Action::Open(Dir::E));
    assert_eq!(w.perception().blood(behind).map(|s| s.amount), Some(20));
    // Closed again, more blood out of sight changes nothing the player knows...
    w.apply(&Action::Close(Dir::E));
    w.state
        .current_mut()
        .blood
        .get_mut(&behind)
        .expect("stain")
        .amount = 80;
    w.apply(&Action::Wait);
    assert_eq!(w.perception().blood(behind).map(|s| s.amount), Some(20));
    // ...until they look again.
    w.apply(&Action::Open(Dir::E));
    assert_eq!(w.perception().blood(behind).map(|s| s.amount), Some(80));
}

#[test]
fn blood_is_saved() {
    let mut w = world();
    let rat = actor_of(&w, "rat");
    strike(&mut w, rat, hit(30, true), 2);
    w.apply(&Action::Wait);
    assert!(!blood(&w).is_empty());
    let loaded = crate::save::decode(&crate::save::encode(&w), CONTENT.clone()).expect("loads");
    assert_eq!(blood(&loaded), blood(&w));
    assert_eq!(
        loaded.state().current().blood_memory,
        w.state().current().blood_memory
    );
    assert_eq!(loaded.state_hash(), w.state_hash());
}

/// The built-in content with every creature bloodless.
fn bloodless() -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    let text: String = (CREATURES.lines())
        .filter(|l| !l.starts_with("blood = "))
        .map(|l| match l {
            "[[creature]]" => "[[creature]]\nblood = \"none\"\n".to_string(),
            l => format!("{l}\n"),
        })
        .collect();
    files.insert("data/creatures.toml", &text);
    Arc::new(Content::load(&files).expect("valid"))
}

#[test]
fn blood_never_changes_what_happens() {
    // The arena's brawl, with blood and without: every roll, blow, death, message, and step is
    // the same; only the stains differ.
    let builtin = Arc::new(Content::builtin().expect("builtin"));
    let mut bloody = World::new(builtin, "arena", None).expect("arena");
    let mut clean = World::new(bloodless(), "arena", None).expect("arena");
    for action in parse_actions("wait*40").expect("waits") {
        assert_eq!(bloody.apply(&action), clean.apply(&action));
    }
    let (a, b) = (bloody.state(), clean.state());
    assert_eq!(a.rng, b.rng, "the gameplay stream is untouched");
    assert_eq!(a.actors, b.actors);
    assert_eq!(a.messages, b.messages);
    assert_eq!(a.current().objects, b.current().objects);
    assert!(b.current().blood.is_empty());
    let deaths = (a.messages.iter())
        .filter(|m| m.display().contains("dies"))
        .count();
    assert!(
        deaths >= 2 && a.current().blood.len() > 10,
        "the brawl bled"
    );
    let without = |s: &WorldState| {
        let mut s = s.clone();
        for level in s.levels.values_mut() {
            level.blood.clear();
            level.blood_memory.clear();
        }
        s
    };
    assert_eq!(without(a), without(b));
}

#[test]
fn the_blood_layer_shows_what_the_player_knows() {
    use crate::dump::{Layer, layer_rows};
    let mut w = world();
    let rat = actor_of(&w, "rat");
    let at = w.state().actors[&rat].pos;
    strike(&mut w, rat, hit(30, true), 10);
    w.apply(&Action::Wait);
    let rows = layer_rows(&w, Layer::Blood);
    let cell = |p: Pos| rows[p.y as usize].chars().nth(p.x as usize);
    assert_eq!(cell(at), Some('9'), "soaked under the rat");
    let wall = at.step(Dir::E);
    let digit = crate::dump::blood_digit(&blood(&w)[&wall]);
    assert_eq!(cell(wall), Some(digit), "the wall's face");
    assert_eq!(
        cell(Pos::new(12, 4)).unwrap_or(' '),
        ' ',
        "unknown behind the door"
    );
    assert_eq!("blood".parse::<Layer>(), Ok(Layer::Blood));
}
