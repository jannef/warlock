//! Random vaults come turned on real floors (#959); pinned ones do not. The turn itself:
//! `content/vault_turn_tests.rs`.

use super::*;
use crate::content::{ContentFiles, Turn, validate_route};
use std::borrow::Cow;
use std::collections::{BTreeSet, VecDeque};
use std::sync::LazyLock;

static CONTENT: LazyLock<Content> = LazyLock::new(|| Content::builtin().expect("content"));

fn shared(id: &str) -> &'static Vault {
    (CONTENT.vaults().iter())
        .find(|v| v.id() == id)
        .expect("the vault")
}

/// Every way `vault` lies stamped on `level`, by the top-left corner.
pub(super) fn stamps(level: &LevelTemplate, vault: &Vault) -> Vec<(Pos, Turn)> {
    let (lw, lh) = (level.terrain.width(), level.terrain.height());
    let mut found = Vec::new();
    for turn in Turn::ALL {
        let turned = vault.turned(turn);
        let v = &turned.template.terrain;
        for y in 0..=lh - v.height() {
            for x in 0..=lw - v.width() {
                let at = Pos::new(x, y);
                let same = v
                    .iter()
                    .all(|(p, t)| level.terrain.get(Pos::new(at.x + p.x, at.y + p.y)) == Some(t));
                if same {
                    found.push((at, turn));
                }
            }
        }
    }
    found
}

/// The cells reachable from the up stairs, doors and water and all (no locked door stops this:
/// a kick opens one; nor a secret door: a search finds one).
pub(super) fn reached(level: &LevelTemplate) -> BTreeSet<Pos> {
    let up = (level.stairs.iter())
        .find(|s| s.dir == StairsDir::Up)
        .expect("stairs up")
        .pos;
    let passable = |p: Pos| {
        level.secrets.contains_key(&p)
            || level.terrain.get(p).is_some_and(|t| {
                let def = CONTENT.terrain(*t);
                def.walkable || def.is_closed_door() || def.is_open_door()
            })
    };
    let mut seen = BTreeSet::from([up]);
    let mut queue = VecDeque::from([up]);
    while let Some(p) = queue.pop_front() {
        for dir in Dir::ALL {
            let q = p.step(dir);
            if passable(q) && seen.insert(q) {
                queue.push_back(q);
            }
        }
    }
    seen
}

/// On every floor of a rooms and a cave branch that drew it, the shared vault lies some way round,
/// and its doors lead out to the stairs. Over the seeds it lies several ways.
#[test]
fn flooded_cell_and_guard_post_come_turned_and_their_doors_lead_out() {
    let floors = [("showcase", "depths"), ("ascmearc", "wulfeshol")];
    for id in ["flooded_cell", "guard_post"] {
        let vault = shared(id);
        let mut sizes = BTreeSet::new();
        let mut ways = Vec::new();
        for (campaign, branch) in floors {
            for seed in 1..=60 {
                for depth in 1..=4 {
                    let level_id = format!("{branch}:{depth}");
                    let Ok(level) = generate(&CONTENT, (campaign, seed), &level_id, None) else {
                        continue;
                    };
                    for (at, turn) in stamps(&level, vault) {
                        let turned = vault.turned(turn);
                        sizes.insert((turned.width(), turned.height()));
                        if !ways.contains(&turn) {
                            ways.push(turn);
                        }
                        let reach = reached(&level);
                        for (door, _) in &turned.entrances {
                            let door = Pos::new(at.x + door.x, at.y + door.y);
                            assert!(reach.contains(&door), "{id} {level_id} seed {seed}: {door}");
                        }
                    }
                }
            }
        }
        assert_eq!(sizes.len(), 2, "{id} lies along and across: {sizes:?}");
        assert!(ways.len() >= 4, "{id} several ways: {ways:?}");
    }
}

/// The guard post's patrol and key come along, wherever it lands.
#[test]
fn a_stamped_guard_post_keeps_its_patrol_and_its_key() {
    let mut first_steps = BTreeSet::new();
    for seed in 1..=120 {
        for depth in 1..=4 {
            let id = format!("depths:{depth}");
            let Ok(level) = generate(&CONTENT, ("showcase", seed), &id, None) else {
                continue;
            };
            let key = CONTENT.object_id("brass_key").expect("the key");
            let Some(guard) =
                (level.creatures.iter()).find(|c| c.carry.iter().any(|s| s.object == key))
            else {
                continue;
            };
            validate_route(&level, &CONTENT, guard).expect("the patrol");
            first_steps.insert(format!("{:?}", guard.route[0]));
            let (_, lock) = (level.locks.iter())
                .find(|(_, l)| l.id.as_deref() == Some("guard_post"))
                .expect("the lock");
            assert!(level.has_key_for(&CONTENT, lock, |p| p == guard.pos));
        }
    }
    assert_eq!(first_steps.len(), 4, "every way round: {first_steps:?}");
}

#[test]
fn the_same_seed_gives_the_same_turn() {
    for seed in 1..=8 {
        for depth in 1..=4 {
            let id = format!("depths:{depth}");
            let a = generate(&CONTENT, ("showcase", seed), &id, None).expect("a floor");
            let b = generate(&CONTENT, ("showcase", seed), &id, None).expect("a floor");
            assert_eq!(a, b, "{id} seed {seed}");
        }
    }
}

#[test]
fn a_draw_finds_all_eight_and_one_that_may_not_turn_draws_nothing() {
    let vault = shared("guard_post");
    let mut rng = Rng::stream(1, "test");
    let mut shapes = Vec::new();
    for _ in 0..200 {
        let turned = vault::turned(vault, &mut rng).template.terrain.clone();
        if !shapes.contains(&turned) {
            shapes.push(turned);
        }
    }
    assert_eq!(shapes.len(), 8);

    let mut files = ContentFiles::builtin();
    files.insert(
        "vaults/still.toml",
        "name = \"Still\"\nturns = false\nmap = '''\n##+##\n#...#\n#####\n'''\n",
    );
    let content = Content::load(&files).expect("content");
    let still = (content.vaults().iter())
        .find(|v| v.id() == "still")
        .expect("the vault");
    let pinned = (CONTENT.vaults().iter())
        .find(|v| v.pinned)
        .expect("a pinned vault");
    let before = rng.clone();
    for v in [still, pinned] {
        assert!(matches!(vault::turned(v, &mut rng), Cow::Borrowed(_)));
    }
    assert_eq!(rng, before, "no draw");
}

/// A floor that names a vault has it as its file draws it, whatever the seed.
#[test]
fn a_pinned_vault_keeps_its_orientation() {
    let vault = shared("barrow_chamber");
    assert!(vault.pinned);
    for seed in 1..=8 {
        let level = generate(&CONTENT, ("ascmearc", seed), "cealdhlaw:2", None).expect("a floor");
        // Its map is the same mirrored; the patrol is not.
        let found = stamps(&level, vault);
        assert!(found.iter().any(|(_, t)| *t == Turn::NONE), "seed {seed}");
        let key = CONTENT.object_id("bronze_key").expect("the key");
        let guardian = (level.creatures.iter())
            .find(|c| c.carry.iter().any(|s| s.object == key))
            .expect("its guardian");
        assert_eq!(guardian.route[0], Dir::E, "seed {seed}");
    }
}
