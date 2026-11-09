//! Deep water in caves (#1020): about one `caves` floor in six, never unreachable stairs.

use super::*;
use crate::content::ContentFiles;
use std::sync::LazyLock;

/// The built-in content without the one vault whose river is deep water of its own (the Broken
/// Bridge's): these tests count the generators' pools.
static CONTENT: LazyLock<Content> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("vaults/broken_bridge.toml");
    Content::load(&files).expect("builtin content")
});

fn caves_levels() -> Vec<(String, String)> {
    (CONTENT.dungeons())
        .flat_map(|def| {
            let campaign = def.campaign.clone().unwrap_or_else(|| "showcase".into());
            (1..=def.depth).filter_map(move |depth| {
                let plan = def.plan(depth)?;
                // A portal branch's floors are a Pit's: the first's stand for them all.
                let id = match def.portal {
                    Some(_) => def.pit_level_id(1, depth),
                    None => def.level_id(depth),
                };
                (plan.generator == Generator::Caves).then(|| (campaign.clone(), id))
            })
        })
        .collect()
}

#[test]
fn about_one_caves_floor_in_six_has_deep_water() {
    let deep = CONTENT.terrain_id("deep_water").expect("deep water");
    let levels = caves_levels();
    assert!(!levels.is_empty());
    let (mut with, mut total) = (0, 0);
    for (campaign, id) in &levels {
        for seed in 0..40 {
            // Generation validates that the stairs, and every walkable cell, are reached on foot.
            let level = generate(&CONTENT, (campaign, seed), id, None).expect("generates");
            total += 1;
            let n = level.terrain.cells().iter().filter(|t| **t == deep).count();
            if n > 0 {
                with += 1;
                assert!(n < 70, "{id} seed {seed}: {n}");
            }
        }
    }
    // One in six, less the floors the pool does not fit.
    assert!(with * 12 > total && with * 3 < total, "{with} of {total}");
}

#[test]
fn rooms_floors_have_none() {
    let deep = CONTENT.terrain_id("deep_water").expect("deep water");
    let caves = caves_levels();
    for def in CONTENT.dungeons() {
        let campaign = def.campaign.clone().unwrap_or_else(|| "showcase".into());
        for depth in 1..=def.depth {
            let id = match def.portal {
                Some(_) => def.pit_level_id(1, depth),
                None => def.level_id(depth),
            };
            if caves.iter().any(|(_, c)| *c == id) {
                continue;
            }
            for seed in 0..10 {
                let level = generate(&CONTENT, (&campaign, seed), &id, None).expect("generates");
                assert!(!level.terrain.cells().contains(&deep), "{id} seed {seed}");
            }
        }
    }
}
