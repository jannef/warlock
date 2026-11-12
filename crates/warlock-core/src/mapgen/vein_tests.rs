//! Ore veins (#1390, `vein.rs`): only of a branch's own ores and at their dangers, mithril and
//! adamantium only very deep; a floor's veins change nothing but walls and the rock behind them.

use super::*;
use crate::content::{ContentFiles, Mined};
use std::sync::LazyLock;

static CONTENT: LazyLock<Content> =
    LazyLock::new(|| Content::load(&ContentFiles::builtin()).expect("builtin content"));

/// The built-in content with veins on every floor (`[veins] chance` 100), or on none (0), and a
/// one-floor rooms branch `mine` at `danger` with `ores` (a line of its file, or empty).
fn content(chance: u8, danger: u8, ores: &str) -> Content {
    let mut files = ContentFiles::builtin();
    let branch = format!(
        "\n[[dungeon]]\nid = \"mine\"\nname = \"Mine\"\ngenerator = \"rooms\"\ndepth = 1\n\
         size = [60, 30]\n{ores}\ndanger = {danger}\npopulation = [\"rat\"]\n\
         population_count = 0\n"
    );
    let dungeons = format!(
        "{}{branch}",
        include_str!("../../../../assets/data/dungeons.toml")
    );
    files.insert("data/dungeons.toml", &dungeons);
    let mining = include_str!("../../../../assets/data/mining.toml").replace(
        "[veins]\nchance = 20\n",
        &format!("[veins]\nchance = {chance}\n"),
    );
    files.insert("data/mining.toml", &mining);
    Content::load(&files).expect("test content is valid")
}

/// Every vein cell of `level` with its metal's id and the danger its row starts at.
fn veins(content: &Content, level: &LevelTemplate) -> Vec<(Pos, String, u32)> {
    let rules = content.mining().expect("mining.toml");
    let t = &level.terrain;
    (0..t.height())
        .flat_map(|y| (0..t.width()).map(move |x| Pos::new(x, y)))
        .filter_map(|p| {
            let metal = rules.vein_of(t[p])?;
            let row = (rules.finds.iter())
                .find(|f| f.what == Mined::Ore(metal))
                .expect("its row");
            Some((p, content.material(metal).id.clone(), row.danger))
        })
        .collect()
}

#[test]
fn veins_are_of_the_branch_s_ores_at_their_dangers() {
    let (mut floors, mut with) = (0, 0);
    for def in CONTENT.dungeons() {
        let campaign = def.campaign.clone().unwrap_or_else(|| "showcase".into());
        for depth in 1..=def.depth.min(12) {
            let id = match def.portal {
                Some(_) => def.pit_level_id(1, depth),
                None => def.level_id(depth),
            };
            let danger = CONTENT.floor_danger(&id, 0);
            for seed in 0..6 {
                let level = generate(&CONTENT, (&campaign, seed), &id, None).expect("generates");
                let found = veins(&CONTENT, &level);
                floors += 1;
                with += usize::from(!found.is_empty());
                for (p, metal, first) in found {
                    assert!(
                        def.ores.as_ref().is_none_or(|o| o.contains(&metal)),
                        "{id} seed {seed} {p}: {metal} is not of its ores"
                    );
                    assert!(first <= danger, "{id} seed {seed} {p}: {metal} at {danger}");
                    assert!(p.x > 0 && p.y > 0, "{id}: on the edge");
                }
            }
        }
    }
    // A small chance per floor: one in five.
    assert!(
        with * 10 > floors && with * 3 < floors,
        "{with} of {floors}"
    );
}

#[test]
fn a_branch_s_allow_list_keeps_its_veins_and_shallow_floors_hold_no_precious_ore() {
    let content = content(100, 2, "ores = [\"copper\", \"gold\"]");
    for seed in 0..20 {
        let level = generate(&content, ("showcase", seed), "mine:1", None).expect("generates");
        let found = veins(&content, &level);
        assert!(!found.is_empty(), "seed {seed}: every floor has veins here");
        // Gold starts at danger 6: only copper.
        assert!(found.iter().all(|(_, m, _)| m == "copper"), "{found:?}");
        // A vein shows: some cell of the floor's veins touches open ground.
        let open = |p: Pos| {
            (Dir::ALL.iter()).any(|d| {
                (level.terrain.get(p.step(*d))).is_some_and(|t| content.terrain(*t).walkable)
            })
        };
        assert!(
            found.iter().any(|(p, _, _)| open(*p)),
            "seed {seed}: hidden"
        );
    }
}

#[test]
fn mithril_and_adamantium_veins_lie_only_very_deep() {
    let content = content(100, 2, "");
    let mut deep = std::collections::BTreeSet::new();
    for seed in 0..40 {
        let level =
            generate(&content, ("showcase", seed), "grundleas:90", None).expect("generates");
        deep.extend(veins(&content, &level).into_iter().map(|(_, m, _)| m));
    }
    assert!(deep.contains("mithril"), "{deep:?}");
    assert!(CONTENT.floor_danger("grundleas:90", 0) >= 80);
}

#[test]
fn veins_change_only_rock() {
    let (with, without) = (content(100, 5, ""), content(0, 5, ""));
    let wall = with.terrain_id("wall").expect("wall");
    let void = with.terrain_id("void").expect("void");
    for seed in 0..20 {
        let a = generate(&with, ("showcase", seed), "mine:1", None).expect("generates");
        let b = generate(&without, ("showcase", seed), "mine:1", None).expect("generates");
        assert!(veins(&without, &b).is_empty());
        let rules = with.mining().expect("mining");
        let (w, h) = (a.terrain.width(), a.terrain.height());
        for p in (0..h).flat_map(|y| (0..w).map(move |x| Pos::new(x, y))) {
            if a.terrain[p] != b.terrain[p] {
                assert!(rules.vein_of(a.terrain[p]).is_some(), "seed {seed} {p}");
                assert!([wall, void].contains(&b.terrain[p]), "seed {seed} {p}");
            }
        }
        assert_eq!(a.objects, b.objects);
        assert_eq!(a.traps, b.traps);
        assert_eq!(a.creatures.len(), b.creatures.len());
    }
}
