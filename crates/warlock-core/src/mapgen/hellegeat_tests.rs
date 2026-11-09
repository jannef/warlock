//! Hellegeat, the shared portal branch (#1296; spec §7 "The Pits"): a Pit's floors are
//! `hellegeat@S:N`, a pure function of the run seed and that id; their stairs lead only within
//! their own Pit; two Pits differ; the branch's own ids without a Pit name nothing; its folk are
//! fiends.

use super::*;
use std::sync::LazyLock;

static CONTENT: LazyLock<Content> = LazyLock::new(|| Content::builtin().expect("builtin content"));

fn floor(pit: u32, depth: u32) -> String {
    format!("hellegeat@{pit}:{depth}")
}

/// Floor `depth` of Pit `pit` in a run of `seed`, entered from `from` (the mouth's level).
fn level(seed: u64, pit: u32, depth: u32, from: Option<&str>) -> LevelTemplate {
    generate(&CONTENT, ("ascmearc", seed), &floor(pit, depth), from).expect("generates")
}

fn link(level: &LevelTemplate, dir: StairsDir) -> Option<Option<&str>> {
    (level.stairs.iter())
        .find(|s| s.dir == dir)
        .map(|s| s.to.as_deref())
}

#[test]
fn a_pit_is_shared_and_only_its_instances_are_levels() {
    let def = CONTENT.dungeon("hellegeat").expect("Hellegeat");
    assert!(def.campaign.is_none(), "shared across campaigns");
    assert!(def.portal.is_some() && def.endless.is_none() && def.below.is_none());
    assert_eq!(def.depth, 5);
    for id in ["hellegeat@7:1", "hellegeat@4294967295:5"] {
        assert!(CONTENT.is_portal_floor(id), "{id}");
        assert!(CONTENT.is_endless_floor(id), "{id}: forgotten when left");
        // Every campaign's runs may open one.
        assert!(CONTENT.has_level("ascmearc", id), "{id}");
    }
    let (_, depth) = CONTENT
        .branch_level("hellegeat@7:3")
        .expect("a Pit's floor");
    assert_eq!(depth, 3);
    assert_eq!(
        CONTENT.level_display_name("hellegeat@7:3").as_deref(),
        Some("Hellegeat 3")
    );
    // No Pit, Pit 0, a Pit spelt two ways, a floor past the last, or a Pit of another branch.
    for id in [
        "hellegeat:1",
        "hellegeat@0:1",
        "hellegeat@07:1",
        "hellegeat@:1",
        "hellegeat@x:1",
        "hellegeat@7:6",
        "hellegeat@4294967296:1",
        "grundleas@7:1",
        "cealdhlaw@7:1",
    ] {
        assert!(CONTENT.branch_level(id).is_none(), "{id}");
        assert!(!CONTENT.is_portal_floor(id), "{id}");
    }
    assert!(!CONTENT.is_portal_floor("grundleas:3"));
    assert!(CONTENT.is_endless_floor("grundleas:3"));
}

#[test]
fn a_pits_floors_lead_only_within_it() {
    let first = level(1, 7, 1, Some("ceaptun"));
    assert_eq!(link(&first, StairsDir::Up), Some(Some("ceaptun")));
    assert_eq!(link(&first, StairsDir::Down), Some(Some("hellegeat@7:2")));
    let third = level(1, 7, 3, None);
    assert_eq!(link(&third, StairsDir::Up), Some(Some("hellegeat@7:2")));
    assert_eq!(link(&third, StairsDir::Down), Some(Some("hellegeat@7:4")));
    // The last floor leads nowhere deeper.
    let last = level(1, 7, 5, None);
    assert_eq!(link(&last, StairsDir::Up), Some(Some("hellegeat@7:4")));
    assert_eq!(link(&last, StairsDir::Down), None);
}

#[test]
fn a_pits_floor_is_the_same_each_time_and_differs_from_every_other() {
    let a = level(11, 7, 2, None);
    assert_eq!(
        a.terrain,
        level(11, 7, 2, None).terrain,
        "made again the same"
    );
    assert_eq!(a.stairs, level(11, 7, 2, None).stairs);
    assert_ne!(a.terrain, level(11, 8, 2, None).terrain, "another Pit");
    assert_ne!(a.terrain, level(11, 7, 3, None).terrain, "another floor");
    assert_ne!(a.terrain, level(12, 7, 2, None).terrain, "another run");
}

#[test]
fn a_pit_is_hot_red_lit_and_full_of_fiends() {
    for pit in [1, 2, 3] {
        let floor = level(5, pit, 1, None);
        assert_eq!(floor.name, "Hellegeat 1");
        assert!(!floor.creatures.is_empty(), "Pit {pit} is inhabited");
        for c in &floor.creatures {
            let def = CONTENT.creature(c.creature);
            assert!(
                CONTENT.has_tag(c.creature, "fiend"),
                "Pit {pit}: {} is no fiend",
                def.id
            );
        }
    }
    let def = CONTENT.dungeon("hellegeat").expect("Hellegeat");
    let plan = def.plan(1).expect("floor 1");
    assert_eq!(plan.temperature.map(|t| t.tenths()), Some(450));
}
