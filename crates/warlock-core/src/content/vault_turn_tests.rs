//! A turned vault is what a file drawn that way would load as (#959).

use super::*;
use crate::content::level::validate_route;
use crate::content::{Content, ContentFiles};
use crate::test_support::builtin;

/// Everything a vault may hold that has a position or a direction: a patrol, a bolt trap with its
/// way, running water, a light, loot and monster pools, a pile, and a named place.
const AS_DRAWN: &str = "name = \"Turning\"\nmap = '''\n#######\n#k..~~#\n#.^.m.#\n#g.g*S#\n###+###\n'''\n\
[[pile]]\nat = [2, 3]\nobjects = [\"dagger\"]\n";

/// The same drawn a clockwise quarter on, by hand: every position and direction turned.
const QUARTER_ON: &str = "name = \"Turning\"\nmap = '''\n#####\n#g.k#\n#.^.#\n+g..#\n#*m~#\n#S.~#\n#####\n'''\n\
[[pile]]\nat = [1, 2]\nobjects = [\"dagger\"]\n";

const LEGEND: &str = "[legend]\n\
\"g\" = { terrain = \"floor\", monsters = { pool = \"level\", count = 1 } }\n\
\"m\" = { terrain = \"floor\", loot = \"magic\" }\n\
\"*\" = { terrain = \"floor\", light = \"lamp\" }\n\
\"S\" = { terrain = \"statue\", name = \"a figure\", note = \"It looks at the door.\" }\n";

fn fixture(map: &str, route: &str, bolt: &str, flow: &str) -> String {
    format!(
        "{map}{LEGEND}\
         \"k\" = {{ terrain = \"floor\", creature = \"kobold\", route = \"{route}\" }}\n\
         \"^\" = {{ terrain = \"floor\", trap = \"lightning_bolt\", dir = \"{bolt}\" }}\n\
         \"~\" = {{ terrain = \"running_water\", flow = \"{flow}\" }}\n"
    )
}

fn turning(content: &Content, id: &str) -> Vault {
    (content.vaults().iter())
        .find(|v| v.id() == id)
        .expect("the vault")
        .clone()
}

#[test]
fn a_quarter_turn_is_the_vault_drawn_a_quarter_on() {
    let mut files = ContentFiles::builtin();
    let drawn = fixture(AS_DRAWN, "E2 W2", "E", "NE");
    let hand = fixture(QUARTER_ON, "S2 N2", "S", "SE");
    files.insert("vaults/as_drawn.toml", &drawn);
    files.insert("vaults/quarter_on.toml", &hand);
    let content = Content::load(&files).expect("content");
    let as_drawn = turning(&content, "as_drawn");
    let quarter_on = turning(&content, "quarter_on");
    let mut turned = as_drawn.turned(Turn::ALL[1]);
    assert_eq!(turned.template.id, "as_drawn");
    turned.template.id = quarter_on.template.id.clone();
    assert_eq!(turned.template, quarter_on.template);
    assert_eq!(turned.entrances, quarter_on.entrances);
    assert_eq!(turned.entrances, vec![(Pos::new(0, 3), Dir::W)]);
    assert_eq!((turned.width(), turned.height()), (5, 7));
}

#[test]
fn a_position_and_a_direction_turn_alike() {
    for turn in Turn::ALL {
        for (w, h) in [(9, 6), (3, 3)] {
            let (tw, th) = turn.size(w, h);
            let mut seen = std::collections::BTreeSet::new();
            for y in 0..h {
                for x in 0..w {
                    let p = Pos::new(x, y);
                    let q = turn.pos(p, w, h);
                    assert!((0..tw).contains(&q.x) && (0..th).contains(&q.y), "{turn:?}");
                    assert!(seen.insert(q), "{turn:?}: {q} twice");
                    for dir in Dir::ALL {
                        let on = p.step(dir);
                        if (0..w).contains(&on.x) && (0..h).contains(&on.y) {
                            assert_eq!(turn.pos(on, w, h), q.step(turn.dir(dir)), "{turn:?}");
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn the_eight_ways_are_all_different_and_come_back() {
    let content = builtin();
    let guard = turning(&content, "guard_post");
    let shapes: Vec<_> = (Turn::ALL.iter())
        .map(|t| guard.turned(*t).template.terrain)
        .collect();
    for (i, a) in shapes.iter().enumerate() {
        for b in &shapes[i + 1..] {
            assert_ne!(a, b, "the guard post has no symmetry");
        }
    }
    for v in content.vaults() {
        assert_eq!(v.turned(Turn::NONE).template, v.template, "{}", v.id());
        assert_eq!(v.turned(Turn::NONE).entrances, v.entrances, "{}", v.id());
        let quarter = Turn::ALL[1];
        let back = (v.turned(quarter).turned(quarter).turned(quarter)).turned(quarter);
        assert_eq!(back.template, v.template, "{}", v.id());
        let mirror = Turn::ALL[4];
        let back = v.turned(mirror).turned(mirror);
        assert_eq!(back.template, v.template, "{}", v.id());
    }
}

/// Every turn of every vault passes the rules of a vault file, finds the entrances the map shows,
/// and has every thing it holds on the cell it stood on, every patrol walkable.
#[test]
fn every_turn_of_every_vault_is_a_valid_vault() {
    let content = builtin();
    for v in content.vaults() {
        let (w, h) = (v.width(), v.height());
        for turn in Turn::ALL {
            let turned = v.turned(turn);
            let name = format!("{} {turn:?}", v.id());
            let again = Vault::new(turned.template.clone(), &content)
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            // An outdoor vault's corner is an entrance two ways out, either as good.
            if v.is_outdoor() {
                let cells = |v: &Vault| v.entrances.iter().map(|e| e.0).collect::<Vec<_>>();
                assert_eq!(cells(&again), cells(&turned), "{name}");
            } else {
                assert_eq!(again.entrances, turned.entrances, "{name}");
            }
            let template = &turned.template;
            for (p, t) in v.template.terrain.iter() {
                assert_eq!(template.terrain[turn.pos(p, w, h)], *t, "{name} at {p}");
            }
            for (door, out) in &turned.entrances {
                let beyond = door.step(*out);
                assert!(template.terrain.get(beyond).is_none(), "{name}: {door}");
            }
            for spawn in &template.creatures {
                validate_route(template, &content, spawn).unwrap_or_else(|e| panic!("{name}: {e}"));
            }
            for (p, pile) in &v.template.objects {
                assert_eq!(template.objects.get(&turn.pos(*p, w, h)), Some(pile));
            }
            for (p, lock) in &v.template.locks {
                assert_eq!(template.locks.get(&turn.pos(*p, w, h)), Some(lock));
            }
            for (p, chest) in &v.template.chests {
                assert_eq!(template.chests.get(&turn.pos(*p, w, h)), Some(chest));
            }
            for (p, secret) in &v.template.secrets {
                assert_eq!(template.secrets.get(&turn.pos(*p, w, h)), Some(secret));
            }
            for (p, place) in &v.template.places {
                assert_eq!(template.places.get(&turn.pos(*p, w, h)), Some(place));
            }
            for (p, light) in &v.template.lights {
                assert_eq!(template.lights.get(&turn.pos(*p, w, h)), Some(light));
            }
            for (p, trap) in &v.template.traps {
                let moved = template.traps.get(&turn.pos(*p, w, h)).expect("the trap");
                assert_eq!(moved.dir, trap.dir.map(|d| turn.dir(d)), "{name}");
            }
            for (p, flow) in &v.template.flow {
                let moved = template.flow.get(&turn.pos(*p, w, h)).expect("the flow");
                assert_eq!(moved.dir, turn.dir(flow.dir), "{name}");
            }
            assert_eq!(template.loot.len(), v.template.loot.len());
            for (a, b) in v.template.monsters.iter().zip(&template.monsters) {
                let mut want: Vec<Pos> = a.cells.iter().map(|p| turn.pos(*p, w, h)).collect();
                want.sort_by_key(|p| row_order(*p));
                assert_eq!(b.cells, want, "{name}: the cells, in row order");
            }
        }
    }
}

#[test]
fn the_guard_post_turns_with_its_guard_key_and_store_room() {
    let content = builtin();
    let guard = turning(&content, "guard_post");
    let (w, h) = (guard.width(), guard.height());
    for turn in Turn::ALL {
        let turned = guard.turned(turn);
        let template = &turned.template;
        let [kobold] = template.creatures.as_slice() else {
            panic!("one guard");
        };
        assert_eq!(kobold.pos, turn.pos(Pos::new(2, 2), w, h));
        assert_eq!(kobold.route[0], turn.dir(Dir::E));
        assert_eq!(kobold.route[2], turn.dir(Dir::W));
        // The key on him still opens the door, now that way round.
        let (door, lock) = template.locks.iter().next().expect("the lock");
        assert_eq!(*door, turn.pos(Pos::new(5, 2), w, h));
        assert!(template.has_key_for(&content, lock, |p| p == kobold.pos));
    }
}

#[test]
fn turns_is_for_a_vault_and_says_whether_it_may() {
    let mut files = ContentFiles::builtin();
    files.insert(
        "vaults/still.toml",
        "name = \"Still\"\nturns = false\nmap = '''\n##+##\n#...#\n#####\n'''\n",
    );
    files.insert(
        "vaults/free.toml",
        "name = \"Free\"\nmap = '''\n##+##\n#...#\n#####\n'''\n",
    );
    let content = Content::load(&files).expect("content");
    assert!(!turning(&content, "still").may_turn());
    assert!(turning(&content, "free").may_turn());

    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/still.toml",
        "name = \"Still\"\nturns = false\nmap = '''\n###\n#.#\n###\n'''\n",
    );
    let err = Content::load(&files).expect_err("a level").message;
    assert!(err.contains("only a vault turns"), "{err}");
}

#[test]
fn a_vault_a_branch_level_names_never_turns() {
    let content = builtin();
    let pinned: Vec<&Vault> = content.vaults().iter().filter(|v| v.pinned).collect();
    assert!(!pinned.is_empty());
    assert!(pinned.iter().all(|v| !v.may_turn()), "pinned stay as drawn");
    assert!(
        (content.vaults().iter())
            .filter(|v| !v.pinned)
            .all(Vault::may_turn),
        "nothing shipped opts out yet"
    );
}
