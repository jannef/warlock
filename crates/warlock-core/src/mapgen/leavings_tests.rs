//! Leavings (`leavings.rs`): laid near what a population draw and a vault's pool place, from the
//! level's stream, the same every time.

use super::*;
use crate::content::{ContentFiles, Laid, Leaving};

/// A webbing test type in the barrow spider's body, tagged `webber` for a vault's pool.
fn webber() -> String {
    include_str!("../../../../assets/data/creatures/barrow_spider.toml")
        .replace("id = \"barrow_spider\"", "id = \"webber\"")
        .replace("tags = [\"vermin\"]", "tags = [\"webber\"]")
}

/// The builtin content with a rooms branch of `danger` peopled by `population`, its draws dread
/// one in `one_in`, its floor stamping `vault` if given.
fn content(population: &str, danger: u8, one_in: u16, vault: Option<&str>) -> Content {
    let mut files = ContentFiles::builtin();
    files.insert("data/creatures/webber.toml", &webber());
    let mut levels = String::new();
    if let Some(vault) = vault {
        files.remove("vaults/");
        files.insert("vaults/den.toml", vault);
        levels = "[[dungeon.levels]]\nfrom = 1\nvaults = [\"den\"]\n".into();
    }
    let branch = format!(
        "\n[[dungeon]]\nid = \"lair\"\nname = \"Lair\"\ngenerator = \"rooms\"\ndepth = 1\n\
         size = [80, 40]\ndanger = {danger}\npopulation = [{population}]\n\
         dread_one_in = {one_in}\nspawn = {{ every = \"1m\", chance = 100 }}\n{levels}"
    );
    let dungeons = format!(
        "{}{branch}",
        include_str!("../../../../assets/data/dungeons.toml")
    );
    files.insert("data/dungeons.toml", &dungeons);
    Content::load(&files).expect("content")
}

fn floor(content: &Content, seed: u64) -> LevelTemplate {
    generate(content, ("showcase", seed), "lair:1", None).expect("the floor")
}

fn terrain_is(content: &Content, level: &LevelTemplate, p: Pos, leaving: Leaving) -> bool {
    content.leaving(leaving) == Some(Laid::Terrain(level.terrain[p]))
}

#[test]
fn a_basilisk_drawn_stands_among_statues_of_the_floor_s_nobodies() {
    // Every draw dread: the basilisks, with kobolds for the statues to be of.
    let content = content("\"basilisk\", \"kobold\"", 16, 1, None);
    let basilisk = content.creature_id("basilisk").expect("basilisk");
    let kobold = content.creature_id("kobold").expect("kobold");
    let mut laid = 0;
    for seed in 1..=4 {
        let level = floor(&content, seed);
        let basilisks: Vec<Pos> = (level.creatures.iter())
            .filter(|c| c.creature == basilisk)
            .map(|c| c.pos)
            .collect();
        assert!(!basilisks.is_empty(), "seed {seed}: basilisks drawn");
        for (&p, statue) in &level.statues {
            assert!(terrain_is(&content, &level, p, Leaving::Statue), "{p}");
            assert_eq!(statue.of, kobold, "a nobody of the floor's");
            assert!(statue.holds.is_empty(), "holding nothing");
            let near = basilisks.iter().any(|b| b.chebyshev(p) <= leavings::NEAR);
            assert!(near, "seed {seed}: the statue at {p} is near a basilisk");
            // It cuts no way: open ground all about it.
            for d in Dir::ALL {
                let def = content.terrain(level.terrain[p.step(d)]);
                assert!(def.walkable, "seed {seed}: beside the statue at {p}");
            }
            assert!(!level.creatures.iter().any(|c| c.pos == p));
        }
        laid += level.statues.len();
        assert_eq!(
            level,
            floor(&content, seed),
            "seed {seed}: the same every time"
        );
    }
    assert!(laid > 0, "statues were laid");
}

/// Whether a creature on `level` whose type leaves `leaving` stands within reach of `p`.
fn left_by_one_near(content: &Content, level: &LevelTemplate, p: Pos, leaving: Leaving) -> bool {
    (level.creatures.iter()).any(|c| {
        content.creature(c.creature).leavings.contains(&leaving)
            && c.pos.chebyshev(p) <= leavings::NEAR
    })
}

#[test]
fn every_leaving_lies_near_one_that_leaves_it() {
    let content = content("\"kobold\"", 3, 100, None);
    for seed in 1..=6 {
        let level = floor(&content, seed);
        for (p, t) in level.terrain.iter() {
            if let Some(leaving) = content.leaving_terrain(*t) {
                assert!(
                    left_by_one_near(&content, &level, p, leaving),
                    "seed {seed}: {p}"
                );
            }
        }
    }
}

#[test]
fn a_vault_s_pool_leaves_its_webs_about_it() {
    let vault = "name = \"Den\"\nmap = '''\n#########\n#.......#\n#..mmm..#\n#.......#\n\
                 #.......#\n####+####\n'''\n[legend]\n\"m\" = { terrain = \"floor\", monsters = \
                 { pool = \"webber\", count = 3 } }\n";
    let content = content("\"kobold\"", 3, 100, Some(vault));
    let webber = content.creature_id("webber").expect("webber");
    let mut webs = 0;
    for seed in 1..=8 {
        let level = floor(&content, seed);
        let webbers: Vec<Pos> = (level.creatures.iter())
            .filter(|c| c.creature == webber)
            .map(|c| c.pos)
            .collect();
        for (p, _) in level.terrain.iter() {
            if terrain_is(&content, &level, p, Leaving::Web) {
                assert!(left_by_one_near(&content, &level, p, Leaving::Web), "{p}");
                webs += usize::from(webbers.iter().any(|w| w.chebyshev(p) <= leavings::NEAR));
            }
        }
    }
    assert!(webs > 0, "the vault's webbers left webs");
}
