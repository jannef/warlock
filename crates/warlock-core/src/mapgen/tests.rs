use super::*;
use crate::World;
use crate::content::{ContentFiles, Vault};
use crate::rng::parse_seed;
use proptest::prelude::*;
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::sync::{Arc, LazyLock};

/// The built-in content plus scenarios starting on a rooms level and a caves level.
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    for (name, level) in [("rooms", "depths:1"), ("caves", "depths:3")] {
        files.insert(
            &format!("scenarios/test_{name}.toml"),
            &format!("name = \"T\"\nstart_level = \"{level}\"\n"),
        );
    }
    Arc::new(Content::load(&files).expect("builtin content"))
});

fn level(seed: u64, depth: u32) -> LevelTemplate {
    generate(
        &CONTENT,
        ("showcase", seed),
        &format!("depths:{depth}"),
        Some("vast"),
    )
    .expect("generates")
}

/// Header (name, stairs, creatures, traps) plus the map, for snapshots.
pub(super) fn describe(level: &LevelTemplate) -> String {
    describe_in(&CONTENT, level)
}

/// [`describe`] with `content` (another branch's tests: `cealdhlaw_tests.rs`).
pub(super) fn describe_in(content: &Content, level: &LevelTemplate) -> String {
    let (w, h) = (level.terrain.width(), level.terrain.height());
    let mut out = format!("{} ({}) {w}x{h}\n", level.name, level.id);
    for s in &level.stairs {
        let to = s.to.as_deref().unwrap_or("nowhere");
        let _ = writeln!(out, "stairs {:?} at {} -> {to}", s.dir, s.pos);
    }
    for c in &level.creatures {
        let route: Vec<&str> = c.route.iter().map(|d| d.name()).collect();
        let name = &content.creature(c.creature).id;
        let _ = write!(out, "{name} at {} route {}", c.pos, route.join(" "));
        // Its gear (spec §8 "Creature gear"), with what of it is worn and wielded.
        let pack = crate::kit::Pack {
            items: c.carry.clone(),
            equipment: c.equipment.clone(),
        };
        let gear = pack.names(content);
        if !gear.is_empty() {
            let _ = write!(out, " carrying {}", gear.join(", "));
        }
        let _ = writeln!(out);
    }
    for (p, trap) in &level.traps {
        let _ = writeln!(out, "{} trap at {p}", content.trap(trap.kind).id);
    }
    let _ = writeln!(out, "{} placed lights", level.lights.len());
    for room in &level.rooms {
        let effect = content.room_effect(room.effect);
        let _ = writeln!(
            out,
            "special room {} at {}-{}",
            effect.id, room.min, room.max
        );
    }
    for shop in &level.shops {
        let def = content.shops().expect("shops").get(shop.kind);
        let _ = writeln!(
            out,
            "shop {} kept by {} at {}-{}, post {}",
            def.id, shop.keeper, shop.min, shop.max, shop.post
        );
    }
    out + &ascii_with_rooms(level, content)
}

#[test]
fn rooms_snapshot() {
    insta::assert_snapshot!("rooms_seed_1_depth_1", describe(&level(1, 1)));
}

#[test]
fn caves_snapshot() {
    insta::assert_snapshot!("caves_seed_1_depth_3", describe(&level(1, 3)));
}

/// A rooms level with a flooded room and a caves level with a shadowed chamber (spec §7 "Special
/// rooms"), their plain ground drawn as the effect's mark.
#[test]
fn special_room_snapshots() {
    let flooded = level(30, 2);
    assert_eq!(flooded.rooms.len(), 1);
    insta::assert_snapshot!("rooms_seed_30_depth_2_flooded", describe(&flooded));
    let shadowed = level(9, 4);
    assert_eq!(shadowed.rooms.len(), 1);
    insta::assert_snapshot!("caves_seed_9_depth_4_shadowed", describe(&shadowed));
    // Without the marks, the flooded room shows its water.
    let map = ascii(&flooded, &CONTENT);
    let water = CONTENT
        .terrain(CONTENT.terrain_id("shallow_water").expect("terrain"))
        .glyph;
    assert!(map.contains(&water.to_string().repeat(5)), "{map}");
}

#[test]
fn deepest_level_snapshot() {
    let seed = parse_seed("warlock").expect("word seed");
    insta::assert_snapshot!("rooms_seed_warlock_depth_8", describe(&level(seed, 8)));
}

/// Cells the player can walk to from `from` (8-way moves; closed doors open on a bump, and
/// secret doors once found).
fn player_reach(level: &LevelTemplate, from: Pos) -> Grid<bool> {
    let terrain = &level.terrain;
    let passable = |p: Pos| {
        level.secrets.contains_key(&p)
            || terrain.get(p).is_some_and(|t| {
                let def = CONTENT.terrain(*t);
                def.walkable || def.is_closed_door()
            })
    };
    let mut seen = Grid::new(terrain.width(), terrain.height(), false);
    seen[from] = true;
    let mut queue = VecDeque::from([from]);
    while let Some(p) = queue.pop_front() {
        for dir in Dir::ALL {
            let q = p.step(dir);
            if passable(q) && !seen[q] {
                seen[q] = true;
                queue.push_back(q);
            }
        }
    }
    seen
}

/// Everything a generated level promises.
fn check(level: &LevelTemplate, depth: u32) -> Result<(), TestCaseError> {
    let content = &**CONTENT;
    let def = content.dungeon("depths").expect("depths branch");
    let plan = def.plan(depth).expect("plan");
    let terrain = &level.terrain;
    prop_assert_eq!(
        (terrain.width(), terrain.height()),
        (plan.width, plan.height)
    );
    prop_assert_eq!(&level.name, &def.level_name(depth));

    // Stairs: up first (the arrival point), down unless deepest, linked to the neighbors.
    let up_to = if depth == 1 {
        "vast".to_string()
    } else {
        def.level_id(depth - 1)
    };
    let mut expected = vec![(StairsDir::Up, Some(up_to))];
    if depth < def.depth {
        expected.push((StairsDir::Down, Some(def.level_id(depth + 1))));
    }
    let links: Vec<_> = level.stairs.iter().map(|s| (s.dir, s.to.clone())).collect();
    prop_assert_eq!(links, expected);
    prop_assert_eq!(level.player_start, Some(level.stairs[0].pos));
    for s in &level.stairs {
        prop_assert_eq!(content.terrain(terrain[s.pos]).stairs, Some(s.dir));
    }

    // Connectivity: the player reaches every stair and every walkable cell from the arrival.
    let reach = player_reach(level, level.stairs[0].pos);
    for (p, t) in terrain.iter() {
        let def = content.terrain(*t);
        if def.walkable || def.is_closed_door() {
            prop_assert!(reach[p], "{} unreachable in {}", p, level.id);
        }
        let edge =
            p.x == 0 || p.y == 0 || p.x == terrain.width() - 1 || p.y == terrain.height() - 1;
        prop_assert!(!edge || !def.walkable, "walkable edge at {}", p);
    }

    // Secret doors (spec §8 "Searching"): closed doors hidden in a wall, never locked, and only
    // where the stairs are reached without finding them.
    let plain = player_reach_unlocked(level, level.stairs[0].pos);
    for s in &level.stairs {
        prop_assert!(plain[s.pos], "stairs at {} behind a secret door", s.pos);
    }
    for (p, secret) in &level.secrets {
        let look = content.terrain(terrain[*p]);
        prop_assert!(content.terrain(secret.door).is_closed_door());
        prop_assert!(
            !look.walkable && !look.transparent,
            "secret door at {} shows",
            p
        );
        prop_assert!(
            !level.locks.contains_key(p),
            "a locked secret door at {}",
            p
        );
    }

    // Light: something shines.
    let emitters = terrain
        .cells()
        .iter()
        .filter(|t| content.terrain(**t).light.is_some())
        .count();
    prop_assert!(emitters + level.lights.len() > 0, "{} is dark", level.id);

    // Patrols loop over walkable terrain.
    for spawn in &level.creatures {
        crate::content::validate_route(level, content, spawn).map_err(TestCaseError::fail)?;
    }

    // Special rooms (spec §7 "Special rooms"): at most `per_level`, apart, never where the
    // player arrives; a room's floor is its effect's floor, and creatures do not patrol through
    // one they shun.
    let effects = content.room_effects();
    prop_assert!(level.rooms.len() <= usize::from(effects.per_level));
    for (i, room) in level.rooms.iter().enumerate() {
        prop_assert!(room.is_valid_within(terrain.width(), terrain.height()));
        prop_assert!(level.rooms[..i].iter().all(|o| !o.overlaps(room)));
        prop_assert!(
            !room.contains(level.stairs[0].pos),
            "arrival in a special room"
        );
        let effect = content.room_effect(room.effect);
        let floor = content.terrain_id("floor");
        if let Some(own) = effect.floor {
            let cells = room.cells();
            prop_assert!(
                cells.iter().any(|p| terrain[*p] == own),
                "no {} floor",
                effect.id
            );
            prop_assert!(
                cells.iter().all(|p| Some(terrain[*p]) != floor),
                "plain floor left"
            );
        }
        if effect.shunned {
            for spawn in &level.creatures {
                let mut p = spawn.pos;
                prop_assert!(!room.contains(p));
                for dir in &spawn.route {
                    p = p.step(*dir);
                    prop_assert!(!room.contains(p), "a patrol through a {}", effect.id);
                }
            }
        }
    }

    // Objects stamped from vaults lie on walkable terrain, in piles that are never empty.
    for (p, pile) in &level.objects {
        prop_assert!(
            content.terrain(terrain[*p]).walkable,
            "objects on a wall at {}",
            p
        );
        prop_assert!(!pile.is_empty());
    }

    // Traps (spec §8 "Traps") are hidden, on walkable ground that is not stairs, never next to
    // stairs (the player arrives there), and never on a patrol (creatures would spring them
    // every round).
    let mut patrolled = Vec::new();
    for spawn in &level.creatures {
        let mut p = spawn.pos;
        patrolled.push(p);
        for dir in &spawn.route {
            p = p.step(*dir);
            patrolled.push(p);
        }
    }
    for (p, trap) in &level.traps {
        let ground = content.terrain(terrain[*p]);
        prop_assert!(!trap.known, "a known trap at {}", p);
        prop_assert!(
            ground.walkable && ground.stairs.is_none() && ground.freeze.is_none(),
            "trap on {}",
            ground.id
        );
        prop_assert!(
            level.stairs.iter().all(|s| s.pos.chebyshev(*p) > 1),
            "trap at {} by the stairs",
            p
        );
        prop_assert!(!patrolled.contains(p), "trap at {} on a patrol", p);
    }
    // Locks (spec §8 "Doors"): only on closed doors, and never in the way of the stairs or of a
    // key to every lock, which the player reaches without unlocking anything (the level is
    // solvable without kicking a door in).
    let unlocked = player_reach_unlocked(level, level.stairs[0].pos);
    for (p, lock) in &level.locks {
        prop_assert!(
            content.terrain(terrain[*p]).is_closed_door(),
            "lock at {}",
            p
        );
        prop_assert!(
            level.has_key_for(content, lock, |q| unlocked[q]),
            "no key to the lock at {} in {}",
            p,
            level.id
        );
    }
    for s in &level.stairs {
        prop_assert!(unlocked[s.pos], "stairs at {} locked away", s.pos);
    }
    Ok(())
}

/// Rooms levels hide a door or two (spec §8 "Searching") from a stream of their own: with its
/// secret doors opened, a level is the one generated without any, and every one of them hides a
/// way the level can do without (`check`).
#[test]
fn rooms_levels_hide_a_few_doors_of_their_own() {
    let text = include_str!("../../../../assets/campaigns/showcase/data/dungeons/showcase.toml");
    let without = text.replace("secret_doors = [1, 2]\n", "");
    assert_ne!(without, text, "the depths hide doors");
    let mut files = ContentFiles::builtin();
    files.insert("campaigns/showcase/data/dungeons/showcase.toml", &without);
    let plain = Content::load(&files).expect("content");
    let [_, most] = CONTENT.dungeon("depths").expect("depths").secret_doors;
    let (mut levels, mut doors) = (0, 0);
    for seed in 0..10 {
        for depth in [1, 2, 6, 7, 8] {
            let level = level(seed, depth);
            let id = format!("depths:{depth}");
            let bare = generate(&plain, ("showcase", seed), &id, Some("vast")).expect("generates");
            assert!(bare.secrets.is_empty());
            let mut opened = level.terrain.clone();
            for (p, secret) in &level.secrets {
                opened.set(*p, secret.door);
            }
            assert_eq!(opened, bare.terrain, "{id}, seed {seed}");
            assert!(level.secrets.len() <= most as usize);
            levels += usize::from(!level.secrets.is_empty());
            doors += level.secrets.len();
        }
    }
    assert!(
        levels > 40 && doors > levels,
        "{levels} levels, {doors} doors"
    );
}

/// Cells the player can walk to from `from` without unlocking a door or finding a secret one
/// (8-way moves).
fn player_reach_unlocked(level: &LevelTemplate, from: Pos) -> Grid<bool> {
    let mut open = level.clone();
    open.secrets.clear();
    for p in level.locks.keys() {
        open.terrain
            .set(*p, CONTENT.terrain_id("wall").expect("wall"));
    }
    player_reach(&open, from)
}

#[test]
fn rooms_levels_lock_a_few_rooms_and_leave_their_keys_in_reach() {
    let key = CONTENT.object_id("rusty_key").expect("rusty key");
    let (mut levels, mut locks, mut keys) = (0, 0, 0);
    for seed in 0..40 {
        for depth in [1, 2, 6, 7, 8] {
            let level = level(seed, depth);
            let generated = (level.locks.iter())
                .filter(|(_, l)| l.id.as_deref() == Some("depths"))
                .count();
            let lying = (level.objects.values().flatten())
                .filter(|s| s.object == key)
                .count();
            assert!(generated <= 2, "{} locks {generated} rooms", level.id);
            assert_eq!(
                lying,
                usize::from(generated > 0),
                "one key per level that locks"
            );
            levels += usize::from(generated > 0);
            locks += generated;
            keys += lying;
        }
    }
    assert!(
        levels > 50 && locks > levels,
        "{levels} levels, {locks} locks"
    );
    assert_eq!(keys, levels);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn every_depth_is_valid_and_deterministic(seed: u64) {
        for depth in 1..=8 {
            let a = level(seed, depth);
            check(&a, depth)?;
            prop_assert_eq!(&a, &level(seed, depth), "same seed, same level");
        }
    }

    /// Arriving on a fresh level, normal eyes without a lantern see more than their own cell:
    /// the arrival room has torches, the cave stairs a lamp.
    #[test]
    fn arrival_is_not_pitch_dark(seed: u64) {
        for scenario in ["test_rooms", "test_caves"] {
            let world = World::new(CONTENT.clone(), scenario, Some(seed)).expect("world");
            let view = world.perception();
            let visible = (0..view.height())
                .flat_map(|y| (0..view.width()).map(move |x| Pos::new(x, y)))
                .filter(|p| view.is_visible(*p))
                .count();
            prop_assert!(visible > 9, "{scenario} seed {seed}: only {visible} cells visible");
        }
    }
}

/// "Not terribly common, but not too uncommon either": about three generated levels in five
/// have a special room, every effect turns up, and rooms levels and caves levels both get them.
#[test]
fn special_rooms_come_about_three_levels_in_five() {
    let seeds = 0..150;
    let mut levels = 0;
    let mut special = 0;
    let mut by_effect = vec![0; CONTENT.room_effects().defs().len()];
    let mut by_generator = [0; 2];
    for seed in seeds {
        for depth in 1..=8 {
            let level = level(seed, depth);
            levels += 1;
            if let Some(room) = level.rooms.first() {
                special += 1;
                by_effect[usize::from(room.effect.0)] += 1;
                by_generator[usize::from((3..=5).contains(&depth))] += 1;
            }
        }
    }
    let percent = special * 100 / levels;
    assert!(
        (50..=70).contains(&percent),
        "{special} of {levels} levels ({percent} %) have a special room"
    );
    assert!(by_effect.iter().all(|n| *n > 10), "{by_effect:?}");
    assert!(by_generator.iter().all(|n| *n > 30), "{by_generator:?}");
}

/// Shops are rare: about one generated level in six (rooms levels only), every kind turns up,
/// and each is a lit room with one door, its keeper beside the way in, and its floor laid with
/// what its kind stocks.
#[test]
fn shops_come_about_one_level_in_seven() {
    let shops = CONTENT.shops().expect("shops");
    let mut levels = 0;
    let mut with_shop = 0;
    let mut by_kind = vec![0; shops.defs().len()];
    for seed in 0..150 {
        for depth in 1..=8 {
            let level = level(seed, depth);
            levels += 1;
            assert!(level.shops.len() <= 1, "seed {seed} depth {depth}");
            let Some(shop) = level.shops.first() else {
                continue;
            };
            with_shop += 1;
            by_kind[usize::from(shop.kind.0)] += 1;
            assert!(!(3..=5).contains(&depth), "caverns have no shops");
            check_shop(&level, shop);
        }
    }
    let percent = with_shop * 100 / levels;
    assert!(
        (11..=20).contains(&percent),
        "{with_shop} of {levels} levels ({percent} %) have a shop"
    );
    assert!(by_kind.iter().all(|n| *n > 5), "{by_kind:?}");
}

/// The built-in content plus two branches with odds of their own (spec §7 "Shops", "Special
/// rooms"): one whose special rooms are only ever flooded and that has no shops, rooms on top
/// and caves below; one that always tries for a general store kept by a kobold, Durin, and has
/// no special rooms.
static FIXTURE: LazyLock<Content> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    let fixture = "\n\
         [[dungeon]]\nid = \"fixture_flooded\"\ngenerator = \"rooms\"\ndepth = 2\n\
         size = [80, 21]\n\
         room_effects = { chance = 60, per_level = 3, favoured = { flooded = 100 } }\n\
         shops = { chance = 0 }\n\
         [[dungeon.levels]]\nfrom = 2\ngenerator = \"caves\"\nsize = [100, 40]\n\
         [[dungeon]]\nid = \"fixture_traders\"\ngenerator = \"rooms\"\ndepth = 1\n\
         size = [80, 21]\nroom_effects = { chance = 0 }\n\
         shops = { chance = 100, kinds = [\"general_store\"], names = [\"Durin\"], \
         keeper = \"kobold\" }\n";
    // Appended once, to the first of the table's files.
    let mut once = Some(fixture);
    files.edit_table("dungeons", |text| {
        format!("{text}{}", once.take().unwrap_or_default())
    });
    Content::load(&files).expect("fixture content")
});

/// A branch's own `room_effects` replace the shared odds: a feature favoured with every share is
/// the only one drawn, as many a level as
/// its `per_level`; and `shops = { chance = 0 }` means never a shop.
#[test]
fn a_branch_with_flooded_rooms_only_makes_flooded_rooms_and_no_shops() {
    let flooded = FIXTURE.room_effects().id("flooded").expect("flooded");
    let (mut levels_with, mut most, mut by_depth) = (0, 0, [0; 2]);
    for seed in 0..60 {
        for depth in 1..=2 {
            let id = format!("fixture_flooded:{depth}");
            let level = generate(&FIXTURE, ("showcase", seed), &id, None).expect("generates");
            assert!(level.shops.is_empty(), "seed {seed}: {id} has a shop");
            assert!(level.rooms.iter().all(|r| r.effect == flooded), "{id}");
            assert!(level.rooms.len() <= 3, "{id}: per_level");
            if !level.rooms.is_empty() {
                levels_with += 1;
                by_depth[depth as usize - 1] += 1;
            }
            most = most.max(level.rooms.len());
        }
    }
    assert!(levels_with > 90, "{levels_with} of 120 levels have one");
    assert!(
        by_depth.iter().all(|n| *n > 30),
        "rooms and caves: {by_depth:?}"
    );
    assert!(most > 1, "more than one a level");
}

/// A branch's own `shops`: its kind, its keepers' names and creature, and its chance; and
/// `room_effects = { chance = 0 }` means never a special room.
#[test]
fn a_branch_with_traders_keeps_its_own_shops() {
    let shops = FIXTURE.shops().expect("shops");
    let store = shops.id("general_store").expect("general store");
    let kobold = FIXTURE.creature_id("kobold").expect("kobold");
    let mut with_shop = 0;
    for seed in 0..60 {
        let level =
            generate(&FIXTURE, ("showcase", seed), "fixture_traders:1", None).expect("generates");
        assert!(level.rooms.is_empty(), "seed {seed}: a special room");
        let Some(shop) = level.shops.first() else {
            continue;
        };
        with_shop += 1;
        assert_eq!((shop.kind, shop.keeper.as_str()), (store, "Durin"));
        let keeper = (level.creatures.iter())
            .find(|c| c.pos == shop.post)
            .expect("a keeper at the post");
        assert_eq!(keeper.creature, kobold);
        assert_eq!(keeper.carry, shops.keeper.carry, "the shared keeper's kit");
    }
    assert!(with_shop > 40, "{with_shop} of 60 levels have a shop");
}

/// The odds a branch without tables reads are the shared ones, so the Depths generate as they
/// did before branches had odds of their own (the snapshots above hold the maps).
#[test]
fn branches_without_tables_read_the_shared_odds() {
    let depths = CONTENT.dungeon("depths").expect("the Depths");
    let shared_rooms = crate::content::RoomOdds::shared(CONTENT.room_effects());
    let shared_shops = crate::content::ShopOdds::shared(CONTENT.shops().expect("shops"));
    for depth in 1..=depths.depth {
        assert_eq!(depths.room_odds(depth), Some(&shared_rooms));
        assert_eq!(depths.shop_odds(depth), Some(&shared_shops));
    }
}

/// What a generated shop promises (see `shop.rs`).
fn check_shop(level: &LevelTemplate, shop: &ShopPlan) {
    let shops = CONTENT.shops().expect("shops");
    let keeper = (level.creatures.iter())
        .find(|c| c.pos == shop.post)
        .expect("a keeper at the post");
    assert_eq!(keeper.creature, shops.keeper.creature);
    assert!(keeper.route.is_empty(), "the keeper stays at its post");
    assert!(shop.contains(shop.post));
    // Its walls have one way in, next to the keeper.
    let ring: Vec<Pos> = (shop.min.x - 1..=shop.max.x + 1)
        .flat_map(|x| (shop.min.y - 1..=shop.max.y + 1).map(move |y| Pos::new(x, y)))
        .filter(|p| !shop.contains(*p))
        .collect();
    let doors: Vec<Pos> = (ring.iter().copied())
        .filter(|p| CONTENT.terrain(level.terrain[*p]).walkable || is_door(level, *p))
        .collect();
    assert_eq!(doors.len(), 1, "one door: {doors:?}");
    assert_eq!(
        shop.post.chebyshev(doors[0]),
        1,
        "the keeper stands by the door"
    );
    // Goods of its kind lie on its floor, one stack a cell, and no creature but the keeper.
    let goods: Vec<&Stack> = (level.objects.iter())
        .filter(|(p, _)| shop.contains(**p))
        .flat_map(|(_, pile)| pile)
        .collect();
    let (least, _) = shops.get(shop.kind).items;
    assert!(goods.len() >= usize::from(least).min(4), "{goods:?}");
    for stack in goods {
        assert!(shops.stocks(&CONTENT, shop.kind, stack.object), "{stack:?}");
    }
    let others = (level.creatures.iter()).filter(|c| shop.contains(c.pos) && c.pos != shop.post);
    assert_eq!(others.count(), 0);
    // It is lit.
    let lit = (shop.min.x..=shop.max.x)
        .flat_map(|x| (shop.min.y..=shop.max.y).map(move |y| Pos::new(x, y)))
        .any(|p| {
            level.lights.contains_key(&p) || CONTENT.terrain(level.terrain[p]).light.is_some()
        });
    assert!(lit, "a shop is lit");
}

fn is_door(level: &LevelTemplate, p: Pos) -> bool {
    let def = CONTENT.terrain(level.terrain[p]);
    def.is_closed_door() || def.is_open_door()
}

#[test]
fn seeds_and_levels_give_different_maps() {
    assert_ne!(ascii(&level(1, 1), &CONTENT), ascii(&level(2, 1), &CONTENT));
    assert_ne!(ascii(&level(1, 1), &CONTENT), ascii(&level(1, 2), &CONTENT));
    let from_nowhere = generate(&CONTENT, ("showcase", 1), "depths:1", None).expect("generates");
    assert_eq!(from_nowhere.stairs[0].to, None);
    assert_eq!(
        from_nowhere.terrain,
        level(1, 1).terrain,
        "the map ignores the entry"
    );
}

/// Each generated level hides its branch's few traps (vaults may bring more), from a stream of
/// its own: the map is the same with or without them.
#[test]
fn generated_levels_hide_a_few_traps_of_their_own() {
    let def = CONTENT.dungeon("depths").expect("depths");
    let [least, most] = def.traps_per_level;
    let mut kinds = std::collections::BTreeSet::new();
    for seed in 0..10 {
        for depth in 1..=8 {
            let level = level(seed, depth);
            let from_vaults: usize = (CONTENT.vaults().iter())
                .map(|v| stamped_at(&level, v).len() * v.template.traps.len())
                .sum();
            let own = level.traps.len() - from_vaults;
            assert!(
                (least as usize..=most as usize).contains(&own),
                "{} traps on depths:{depth}, seed {seed}",
                own
            );
            kinds.extend(level.traps.values().map(|t| t.kind));
        }
    }
    assert!(kinds.len() >= 5, "a mix of kinds: {kinds:?}");
    // Without traps in the branch the map is the same.
    let mut files = ContentFiles::builtin();
    let dungeons =
        include_str!("../../../../assets/campaigns/showcase/data/dungeons/showcase.toml")
            .replace("traps = [", "# traps = [");
    files.insert("campaigns/showcase/data/dungeons/showcase.toml", &dungeons);
    let bare = Content::load(&files).expect("content");
    let plain = generate(&bare, ("showcase", 1), "depths:1", Some("vast")).expect("generates");
    assert_eq!(plain.terrain, level(1, 1).terrain);
    assert!(plain.traps.is_empty());
}

#[test]
fn only_branch_levels_generate() {
    let err = |id: &str| generate(&CONTENT, ("showcase", 1), id, None).unwrap_err();
    assert_eq!(err("hub"), GenError::NotAGeneratedLevel("hub".into()));
    assert_eq!(
        err("depths:9"),
        GenError::NotAGeneratedLevel("depths:9".into())
    );
    assert_eq!(
        err("caves:1"),
        GenError::NotAGeneratedLevel("caves:1".into())
    );
}

/// Positions where `vault`'s terrain appears verbatim in `level`.
fn stamped_at(level: &LevelTemplate, vault: &Vault) -> Vec<Pos> {
    let v = &vault.template.terrain;
    let fits = |at: Pos| {
        v.iter()
            .all(|(p, t)| level.terrain.get(Pos::new(at.x + p.x, at.y + p.y)) == Some(t))
    };
    (0..level.terrain.height())
        .flat_map(|y| (0..level.terrain.width()).map(move |x| Pos::new(x, y)))
        .filter(|at| fits(*at))
        .collect()
}

#[test]
fn vaults_are_stamped_verbatim_with_their_lights_and_creatures() {
    // The shared ones (a campaign's stamp in its own branches: `cealdhlaw_tests.rs`).
    let vaults: Vec<&Vault> = (CONTENT.vaults().iter())
        .filter(|v| v.campaign.is_none())
        .collect();
    let mut seen = vec![0; vaults.len()];
    for seed in 0..10 {
        for depth in 1..=8 {
            let level = level(seed, depth);
            for (i, vault) in vaults.iter().enumerate() {
                for at in stamped_at(&level, vault) {
                    seen[i] += 1;
                    let shift = |p: Pos| Pos::new(at.x + p.x, at.y + p.y);
                    for (p, light) in &vault.template.lights {
                        assert_eq!(level.lights.get(&shift(*p)), Some(light), "{}", vault.id());
                    }
                    for (p, trap) in &vault.template.traps {
                        assert_eq!(level.traps.get(&shift(*p)), Some(trap), "{}", vault.id());
                    }
                    for spawn in &vault.template.creatures {
                        assert!(
                            level
                                .creatures
                                .iter()
                                .any(|c| c.pos == shift(spawn.pos) && c.route == spawn.route),
                            "{} creature",
                            vault.id()
                        );
                    }
                }
            }
        }
    }
    // Dungeons get every dungeon vault and never the ruins meant for the overworld's local areas,
    // nor the vaults a branch level names (stamped only there).
    for (vault, count) in vaults.iter().zip(&seen) {
        if vault.pinned {
            assert_eq!(*count, 0, "vault `{}` stamped where not named", vault.id());
        } else if vault.is_outdoor() {
            assert_eq!(*count, 0, "outdoor vault `{}` in a dungeon", vault.id());
        } else {
            assert!(*count > 0, "vault `{}` never placed", vault.id());
        }
    }
}
