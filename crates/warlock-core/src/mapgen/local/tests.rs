use super::*;
use crate::content::{Vault, local_area_id};
use crate::grid::Grid;
use crate::rng::parse_seed;
use proptest::prelude::*;
use std::collections::{BTreeSet, VecDeque};
use std::fmt::Write as _;
use std::sync::{Arc, LazyLock};

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));

fn area(seed: u64, x: i32, y: i32) -> LevelTemplate {
    generate(&CONTENT, seed, &local_area_id(Pos::new(x, y))).expect("generates")
}

/// Overworld cells with every landscape and every neighbor that shapes an area (see the map in
/// `assets/levels/overworld.toml`).
const CELLS: [(&str, i32, i32); 12] = [
    ("plains", 42, 19),
    ("grassland", 40, 10),
    ("forest", 25, 20),
    ("forest by the plains", 30, 19),
    ("dense forest", 10, 20),
    ("hills", 20, 40),
    ("swamp", 60, 44),
    ("road west to east", 40, 23),
    ("road through the Thornwood", 10, 25),
    ("road to the bridge, river at a corner", 61, 21),
    ("plains by the river", 57, 12),
    ("plains by the lake", 65, 28),
];

/// The terrain id of `name`.
fn id(name: &str) -> TerrainId {
    CONTENT.terrain_id(name).expect("terrain")
}

/// A map with one character per terrain (several share a glyph in the game), creatures as `d`,
/// the start as `@`, and running water as the way it flows.
fn debug_map(level: &LevelTemplate) -> String {
    let chars: &[(&str, char)] = &[
        ("forest_floor", '.'),
        ("grass", ','),
        ("dry_grass", ':'),
        ("rocky_ground", '_'),
        ("marsh", '"'),
        ("tall_grass", ';'),
        ("flowers", '*'),
        ("buttercups", '+'),
        ("bluebells", '%'),
        ("bush", 'b'),
        ("tree", 'T'),
        ("pine", 'P'),
        ("dead_tree", 'Y'),
        ("boulder", 'o'),
        ("crag", 'A'),
        ("standing_stone", 'O'),
        ("reeds", '|'),
        ("water", '~'),
        ("dirt_road", '='),
        ("bridge", 'H'),
        ("wall", '#'),
        ("flagstones", '-'),
        ("rubble", '\''),
        ("altar", '_'),
    ];
    let mut out = String::new();
    for y in 0..level.terrain.height() {
        for x in 0..level.terrain.width() {
            let p = Pos::new(x, y);
            let def = CONTENT.terrain(level.terrain[p]);
            let c = if level.creatures.iter().any(|c| c.pos == p) {
                'd'
            } else if level.player_start == Some(p) {
                '@'
            } else if let Some(f) = level.flow.get(&p) {
                match f.dir {
                    Dir::N => '^',
                    Dir::S => 'v',
                    Dir::E => '>',
                    Dir::W => '<',
                    Dir::NE | Dir::SW => '/',
                    Dir::NW | Dir::SE => '\\',
                }
            } else {
                chars
                    .iter()
                    .find(|(id, _)| *id == def.id)
                    .map_or('?', |(_, c)| *c)
            };
            out.push(c);
        }
        out.push('\n');
    }
    out
}

/// Header (name, exit, creatures, lights, flow speeds) plus the map, for snapshots.
fn describe(level: &LevelTemplate) -> String {
    let (w, h) = (level.terrain.width(), level.terrain.height());
    let mut out = format!("{} ({}) {w}x{h}\n", level.name, level.id);
    if let Some(exit) = &level.exit {
        let _ = writeln!(out, "edges -> {} at {}", exit.to, exit.pos);
    }
    for c in &level.creatures {
        let route: Vec<&str> = c.route.iter().map(|d| d.name()).collect();
        let name = &CONTENT.creature(c.creature).id;
        let _ = writeln!(out, "{name} at {} route {}", c.pos, route.join(" "));
    }
    let speeds: BTreeSet<u16> = level.flow.values().map(|f| f.speed).collect();
    let _ = writeln!(
        out,
        "{} placed lights, flow speeds {speeds:?}",
        level.lights.len()
    );
    out + &debug_map(level)
}

#[test]
fn forest_snapshot() {
    insta::assert_snapshot!("forest_seed_1", describe(&area(1, 25, 20)));
}

#[test]
fn dense_forest_snapshot() {
    insta::assert_snapshot!("dense_forest_seed_1", describe(&area(1, 10, 20)));
}

#[test]
fn hills_snapshot() {
    let seed = parse_seed("warlock").expect("word seed");
    insta::assert_snapshot!("hills_seed_warlock", describe(&area(seed, 20, 40)));
}

#[test]
fn swamp_snapshot() {
    insta::assert_snapshot!("swamp_seed_7", describe(&area(7, 60, 44)));
}

#[test]
fn riverbank_snapshot() {
    insta::assert_snapshot!("riverbank_seed_1", describe(&area(1, 57, 12)));
}

#[test]
fn road_snapshot() {
    insta::assert_snapshot!("road_seed_2", describe(&area(2, 40, 23)));
}

/// Cells the player can walk to from `from` (8-way moves).
fn player_reach(level: &LevelTemplate, from: Pos) -> Grid<bool> {
    let terrain = &level.terrain;
    let walkable = |p: Pos| terrain.get(p).is_some_and(|t| CONTENT.terrain(*t).walkable);
    let mut seen = Grid::new(terrain.width(), terrain.height(), false);
    seen[from] = true;
    let mut queue = VecDeque::from([from]);
    while let Some(p) = queue.pop_front() {
        for dir in Dir::ALL {
            let q = p.step(dir);
            if walkable(q) && !seen[q] {
                seen[q] = true;
                queue.push_back(q);
            }
        }
    }
    seen
}

/// Share of the cells (permille) with one of `terrain`.
fn share(level: &LevelTemplate, terrain: &[&str]) -> usize {
    let ids: Vec<TerrainId> = terrain.iter().map(|t| id(t)).collect();
    let n = level.terrain.cells().len();
    level
        .terrain
        .cells()
        .iter()
        .filter(|t| ids.contains(t))
        .count()
        * 1000
        / n
}

/// Everything a local area promises.
fn check(level: &LevelTemplate, cell: Pos) -> Result<(), TestCaseError> {
    let content = &**CONTENT;
    let terrain = &level.terrain;
    let (w, h) = SIZE;
    prop_assert_eq!((terrain.width(), terrain.height()), (w, h));
    prop_assert_eq!(level.kind, LevelKind::Outdoor);
    prop_assert!(level.stairs.is_empty());
    let exit = level.exit.as_ref().expect("an edge exit");
    prop_assert_eq!((exit.to.as_str(), exit.pos), ("overworld", cell));

    // The start is dry ground in the middle third, and the player reaches every walkable cell
    // from it.
    let start = level.player_start.expect("a start");
    let def = content.terrain(terrain[start]);
    prop_assert!(def.walkable && def.move_cost <= 100, "start on {}", def.id);
    prop_assert!((w / 3..2 * w / 3).contains(&start.x) && (h / 3..2 * h / 3).contains(&start.y));
    let reach = player_reach(level, start);
    for (p, t) in terrain.iter() {
        prop_assert!(
            !content.terrain(*t).walkable || reach[p],
            "{} unreachable in {}",
            p,
            level.id
        );
    }

    // Every way in: an arrival on that side (or corner), reachable, on the edge.
    for side in Dir::ALL {
        let p = arrival(terrain, content, side).expect("an arrival");
        prop_assert!(reach[p], "arrival {} from {}", p, side);
        let (dx, dy) = side.delta();
        let on = (dx == -1 && p.x == 0)
            || (dx == 1 && p.x == w - 1)
            || (dy == -1 && p.y == 0)
            || (dy == 1 && p.y == h - 1);
        prop_assert!(on, "arrival {} not on side {}", p, side);
    }

    // Running water, and only running water, flows; one speed per area (a river or a brook).
    for (p, t) in terrain.iter() {
        prop_assert_eq!(
            content.terrain(*t).flows(),
            level.flow.contains_key(&p),
            "{}",
            p
        );
    }
    let speeds: BTreeSet<u16> = level.flow.values().map(|f| f.speed).collect();
    prop_assert!(speeds.len() <= 1, "{:?}", speeds);

    // Patrols loop over walkable terrain; nothing glows but ruins; objects from ruins lie on
    // walkable terrain.
    for spawn in &level.creatures {
        crate::content::validate_route(level, content, spawn).map_err(TestCaseError::fail)?;
    }
    for p in level.objects.keys() {
        prop_assert!(content.terrain(level.terrain[*p]).walkable, "{}", p);
    }
    prop_assert_eq!(level.ambient, crate::light::Light::DARK);
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    #[test]
    fn every_kind_of_area_is_valid_and_deterministic(seed: u64) {
        for (what, x, y) in CELLS {
            let a = area(seed, x, y);
            check(&a, Pos::new(x, y)).map_err(|e| TestCaseError::fail(format!("{what}: {e}")))?;
            prop_assert_eq!(&a, &area(seed, x, y), "{}: same seed, same area", what);
        }
    }

    /// The landscapes look like themselves: woods where the overworld has forest, water by
    /// rivers, lakes, and in swamps, a road on roads.
    #[test]
    fn landscapes_look_like_themselves(seed: u64) {
        let trees = ["tree", "pine"];
        let plains = area(seed, 42, 19);
        let forest = area(seed, 25, 20);
        let dense = area(seed, 10, 20);
        prop_assert!(share(&plains, &trees) < 80);
        prop_assert!((200..600).contains(&share(&forest, &trees)), "{}", share(&forest, &trees));
        prop_assert!(share(&dense, &trees) > share(&forest, &trees));
        prop_assert!(share(&area(seed, 20, 40), &["crag"]) > 20);
        prop_assert!(share(&area(seed, 60, 44), &["water"]) > 150);
        prop_assert!(share(&area(seed, 40, 23), &["dirt_road"]) > 30);
        let road = area(seed, 10, 25);
        prop_assert!(share(&road, &trees) > 150, "a road through the woods");
        // The river runs along the east side, south as on the overworld.
        let bank = area(seed, 57, 12);
        let east = bank.terrain.width() - 1;
        for y in 0..bank.terrain.height() {
            let flow = bank.flow.get(&Pos::new(east, y));
            prop_assert_eq!(flow.map(|f| f.dir), Some(Dir::S), "{}", y);
        }
        let lake = area(seed, 65, 28);
        prop_assert_eq!(lake.terrain[Pos::new(east, 20)], id("water"));
    }
}

#[test]
fn roads_lead_to_the_neighboring_roads() {
    for seed in 0..8 {
        // West to east beside the Hall: the road meets both sides, not the top or bottom.
        let a = area(seed, 40, 23);
        let road = id("dirt_road");
        let on = |p: Pos| a.terrain[p] == road || a.terrain[p] == id("bridge");
        let (w, h) = SIZE;
        assert!((0..h).any(|y| on(Pos::new(0, y))), "seed {seed}: west");
        assert!((0..h).any(|y| on(Pos::new(w - 1, y))), "seed {seed}: east");
        assert!(!(0..w).any(|x| on(Pos::new(x, 0)) || on(Pos::new(x, h - 1))));
    }
}

#[test]
fn neighboring_areas_blend_their_woods() {
    // A forest cell with plains to the east is more open on that side than on the other.
    let trees = |level: &LevelTemplate, xs: std::ops::Range<i32>| {
        let ids = [id("tree"), id("pine")];
        xs.flat_map(|x| (0..SIZE.1).map(move |y| Pos::new(x, y)))
            .filter(|p| ids.contains(&level.terrain[*p]))
            .count()
    };
    let (mut west, mut east) = (0, 0);
    for seed in 0..12 {
        let a = area(seed, 30, 19);
        west += trees(&a, 0..20);
        east += trees(&a, 60..80);
    }
    assert!(west > east * 3 / 2, "west {west}, east {east}");
}

/// Positions where `vault`'s terrain (all but its open ground) appears verbatim in `level`.
fn stamped_at(level: &LevelTemplate, vault: &Vault) -> Vec<Pos> {
    let void = id("void");
    let v = &vault.template.terrain;
    let fits = |at: Pos| {
        v.iter().all(|(p, t)| {
            *t == void || level.terrain.get(Pos::new(at.x + p.x, at.y + p.y)) == Some(t)
        })
    };
    (0..level.terrain.height())
        .flat_map(|y| (0..level.terrain.width()).map(move |x| Pos::new(x, y)))
        .filter(|at| fits(*at))
        .collect()
}

#[test]
fn ruins_are_outdoor_vaults_stamped_verbatim() {
    let outdoor: Vec<&Vault> = CONTENT.vaults().iter().filter(|v| v.is_outdoor()).collect();
    assert!(outdoor.len() >= 2);
    let mut seen = vec![0; outdoor.len()];
    for seed in 0..40 {
        for (_, x, y) in CELLS {
            let level = area(seed, x, y);
            for (i, vault) in outdoor.iter().enumerate() {
                for at in stamped_at(&level, vault) {
                    seen[i] += 1;
                    for (p, light) in &vault.template.lights {
                        let shifted = Pos::new(at.x + p.x, at.y + p.y);
                        assert_eq!(level.lights.get(&shifted), Some(light));
                    }
                    for (p, pile) in &vault.template.objects {
                        let shifted = Pos::new(at.x + p.x, at.y + p.y);
                        assert_eq!(level.objects.get(&shifted), Some(pile));
                    }
                }
            }
        }
    }
    for (vault, count) in outdoor.iter().zip(&seen) {
        assert!(*count > 0, "vault `{}` never placed", vault.id());
    }
}

#[test]
fn arrivals_prefer_dry_ground_near_the_middle_of_the_side() {
    let level = area(1, 57, 12);
    let (w, h) = SIZE;
    // The east side is all river: wading in there is the only way, as near the middle as can be.
    let east = arrival(&level.terrain, &CONTENT, Dir::E).expect("east");
    assert_eq!(east, Pos::new(w - 1, h / 2));
    let west = arrival(&level.terrain, &CONTENT, Dir::W).expect("west");
    assert_eq!(west.x, 0);
    assert!(CONTENT.terrain(level.terrain[west]).move_cost <= 100);
    assert!((west.y - h / 2).abs() < 6);
    let corner = arrival(&level.terrain, &CONTENT, Dir::NW).expect("north-west");
    assert!(corner.x + corner.y < 8, "{corner}");
}

#[test]
fn only_explorable_cells_have_areas() {
    let err = |id: &str| generate(&CONTENT, 1, id).unwrap_err();
    // Mountains, the river, and the Hall.
    for cell in ["local:0,0", "local:59,12", "local:46,23", "local:9,99"] {
        assert_eq!(err(cell), GenError::NotAGeneratedLevel(cell.into()));
    }
}

/// Prints areas while tuning the generators: `WARLOCK_AREAS="25,20 30,9" cargo test -p
/// warlock-core --lib print_areas -- --ignored --nocapture` (seed from `WARLOCK_SEED`, a number
/// or a word; default 1).
#[test]
#[ignore = "a tuning aid, not a check"]
fn print_areas() {
    let seed = std::env::var("WARLOCK_SEED")
        .ok()
        .and_then(|s| parse_seed(&s).ok())
        .unwrap_or(1);
    let list = std::env::var("WARLOCK_AREAS").unwrap_or_else(|_| "25,20".into());
    for cell in list.split_whitespace() {
        let (x, y) = cell.split_once(',').expect("x,y");
        let level = area(seed, x.parse().expect("x"), y.parse().expect("y"));
        println!("{}", describe(&level));
    }
}
