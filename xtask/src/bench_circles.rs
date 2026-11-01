//! The `circles` row of `cargo xtask bench-frames`: fifty spell circles in view (spec §4.8 "Spell
//! circles", docs/design/buff-circles.md §10.3's stress case). A hall of 48 peaceful kobolds, each
//! already buffed, hexed, or both (a spawn's `status`), every combination the design sketches,
//! and the player blessed, hasted, ill-wished, and weakened (`afflict`, a replay): a buffs'
//! circle and a hexes' about them, fifty circles in all.
//!
//! The fixture is the built-in content with the hall added: the game reads it with
//! `--content-dir` from a copy of `assets/`'s TOML under `target/xtask/bench-circles/`, so
//! nothing of the bench ships.

use crate::{target_dir, workspace_root};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use warlock_core::World;
use warlock_core::action::parse_actions;
use warlock_core::content::{Content, ContentFiles};
use warlock_core::replay::Replay;
use warlock_core::world::ActorKind;

/// The kobolds' grid: columns and rows, and the cells between them (a double figure is about
/// nine columns by four rows; these overlap a little, as a crowd does).
const COLUMNS: usize = 8;
const ROWS: usize = 6;
const STEP: (usize, usize) = (6, 4);

/// What the kobolds wear, in turn: one buff, two hexes, both, three buffs, four buffs and a hex,
/// two hexes of the burst kind, a hex alone, and six buffs and hexes.
const STATES: [&str; 8] = [
    r#"["blessed"]"#,
    r#"["weakened", "sluggish"]"#,
    r#"["blessed", "ill_wished"]"#,
    r#"["blessed", "hasted", "barkskin"]"#,
    r#"["lit", "blessed", "hasted", "barkskin", "drowsy"]"#,
    r#"["drowsy", "webbed"]"#,
    r#"["ill_wished"]"#,
    r#"["hasted", "barkskin", "weakened", "sluggish", "drowsy", "webbed"]"#,
];

/// The player's two circles: two buffs inside, two hexes outside. `afflict` takes no time.
const PLAYER: &str = "afflict:blessed afflict:hasted afflict:ill_wished afflict:weakened";

const SCENARIO: &str = "name = \"Bench circles\"\nstart_level = \"bench_circles\"\nseed = 1\n\
                        vision = \"normal\"\nhp = 30\n";

/// The hall: the kobolds on a grid, the player in its middle, sconces between.
fn level() -> String {
    let (w, h) = (COLUMNS * STEP.0 + 3, ROWS * STEP.1 + 3);
    let mut map = vec![vec!['.'; w]; h];
    for (y, row) in map.iter_mut().enumerate() {
        for (x, cell) in row.iter_mut().enumerate() {
            if x == 0 || y == 0 || x == w - 1 || y == h - 1 {
                *cell = '#';
            }
        }
    }
    for row in 0..ROWS {
        for col in 0..COLUMNS {
            let state = (row + col) % STATES.len();
            let (x, y) = (3 + col * STEP.0, 2 + row * STEP.1);
            map[y][x] = char::from(b'a' + state as u8);
            map[y + 2][x] = 't';
        }
    }
    map[h / 2][w / 2] = '@';
    let map: Vec<String> = map.into_iter().map(String::from_iter).collect();
    let mut text = format!(
        "name = \"Bench circles\"\nambient = [6, 7, 9, 0]\nmap = '''\n{}\n'''\n\n[legend]\n\
         \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\"t\" = \"sconce\"\n",
        map.join("\n")
    );
    for (i, state) in STATES.iter().enumerate() {
        text.push_str(&format!(
            "\"{}\" = {{ terrain = \"floor\", creature = \"kobold\", faction = \"peaceful\", \
             status = {state} }}\n",
            char::from(b'a' + i as u8)
        ));
    }
    text
}

/// Copies the TOML files under `from` to `to`, keeping the paths.
fn copy_toml(from: &Path, to: &Path) -> Result<(), String> {
    let entries = std::fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        let target = to.join(path.file_name().expect("an entry has a name"));
        if path.is_dir() {
            copy_toml(&path, &target)?;
        } else if path.extension().is_some_and(|e| e == "toml") {
            std::fs::create_dir_all(to).map_err(|e| e.to_string())?;
            std::fs::copy(&path, &target).map_err(|e| format!("{}: {e}", path.display()))?;
        }
    }
    Ok(())
}

/// Writes the fixture (the content directory and the replay) and returns their paths.
pub fn fixture() -> Result<(PathBuf, PathBuf), String> {
    let dir = target_dir()?.join("xtask/bench-circles");
    let content = dir.join("content");
    if content.exists() {
        std::fs::remove_dir_all(&content).map_err(|e| e.to_string())?;
    }
    copy_toml(&workspace_root().join("assets"), &content)?;
    let campaign = content.join("campaigns/showcase");
    let write = |path: PathBuf, text: &str| {
        std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))
    };
    write(campaign.join("levels/bench_circles.toml"), &level())?;
    write(campaign.join("scenarios/bench_circles.toml"), SCENARIO)?;
    let files = ContentFiles::from_dir(&content).map_err(|e| e.to_string())?;
    let content_data = Arc::new(Content::load(&files).map_err(|e| e.to_string())?);
    let mut world = World::new(content_data, "bench_circles", None).map_err(|e| e.to_string())?;
    let mut replay = Replay::new(&world);
    for action in parse_actions(PLAYER)? {
        world.apply(&action);
        replay.push(&action, &world);
    }
    let circled = circled(&world);
    if circled != COLUMNS * ROWS + 1 {
        return Err(format!(
            "bench circles: {circled} actors circled, not {}",
            COLUMNS * ROWS + 1
        ));
    }
    let path = dir.join("bench-circles.replay.json");
    write(path.clone(), &replay.to_json())?;
    Ok((content, path))
}

/// The seen actors with a status the circle shows: the player, and every kobold in view.
fn circled(world: &World) -> usize {
    let view = world.perception();
    let runes = view.content().runes();
    let player = view.own_effects().any(|s| runes.of(s).is_some());
    let others = (view.actors().iter())
        .filter(|a| a.kind != ActorKind::Player)
        .filter(|a| view.effects_of(a.id).any(|s| runes.of(s).is_some()))
        .count();
    others + usize::from(player)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The hall is a level the game loads, every kobold in view and circled, the player with both
    // kinds: fifty circles.
    #[test]
    fn the_hall_holds_fifty_circles() {
        let mut files = ContentFiles::builtin();
        files.insert("campaigns/showcase/levels/bench_circles.toml", &level());
        files.insert("campaigns/showcase/scenarios/bench_circles.toml", SCENARIO);
        let content = Arc::new(Content::load(&files).expect("the fixture loads"));
        let mut world = World::new(content, "bench_circles", None).expect("it starts");
        for action in parse_actions(PLAYER).expect("parses") {
            world.apply(&action);
        }
        assert_eq!(circled(&world), COLUMNS * ROWS + 1);
        let view = world.perception();
        let runes = view.content().runes();
        let kinds: std::collections::BTreeSet<bool> = (view.own_effects())
            .filter_map(|s| runes.of(s).map(|r| r.hex))
            .collect();
        assert_eq!(
            kinds.len(),
            2,
            "the player has a buffs' and a hexes' circle"
        );
    }
}
