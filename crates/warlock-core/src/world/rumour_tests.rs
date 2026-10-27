//! Rumoured ways (spec §8 "Dialogue", "Rumoured ways"; `world/rumour.rs`): asking Godgifu of
//! Cēaptūn about a place marks the way there on the player's overworld map, the cells about a
//! route and not the whole region, idempotently, in no time; the hidden locations along the ways
//! stay hidden; the memory layer shows the marks once the player is back on the map; a replay.

use super::*;
use crate::action::parse_actions;
use crate::content::{ContentFiles, TerrainId};
use crate::dump::{self, Layer};
use crate::replay::Replay;
use std::sync::LazyLock;

const OVERWORLD: &str = "ascmearc";
const GODGIFU: Pos = Pos::new(46, 26);
/// Cealdhlāw, the barrow beside the end of the way to the barrows: a visible location.
const CEALDHLAW: Pos = Pos::new(34, 25);
/// The topics and a cell near the end of each way, far from the town.
const ENDS: [(&str, Pos); 4] = [
    ("barrows", Pos::new(34, 22)),
    ("marsh", Pos::new(74, 47)),
    ("mines", Pos::new(95, 20)),
    ("volcano", Pos::new(100, 16)),
];

static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    Arc::new(Content::load(&files).expect("builtin content"))
});

/// The walk from the square into the Gæsthūs, to Godgifu's north-west, and a chat with her.
const TO_THE_INN: &str = "move:S*4 open:SW move:SW move:S*3 chat:SE";

/// A run in Cēaptūn with the player beside Godgifu, talking to her.
fn at_the_inn() -> World {
    let mut w = World::new(CONTENT.clone(), "ceaptun", None).expect("a run in Cēaptūn");
    act(&mut w, TO_THE_INN);
    assert_eq!(
        w.state.player().pos.step(Dir::SE),
        GODGIFU,
        "beside Godgifu"
    );
    w
}

fn act(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.text.clone()).collect()
}

/// What the player remembers of overworld cell `p`, if the overworld has been made.
fn remembered(world: &World, p: Pos) -> Option<TerrainId> {
    let level = world.state.levels.get(OVERWORLD)?;
    level.memory.get(p).copied().flatten()
}

fn remembered_count(world: &World) -> usize {
    (world.state.levels.get(OVERWORLD))
        .map_or(0, |l| l.memory.iter().filter(|(_, m)| m.is_some()).count())
}

#[test]
fn asking_marks_the_way_and_nothing_else() {
    for (topic, end) in ENDS {
        let mut w = at_the_inn();
        assert_eq!(remembered(&w, end), None, "{topic}: unknown before");
        let (time, before) = (w.state.time, remembered_count(&w));
        act(&mut w, &format!("ask:SE:{topic}"));
        assert_eq!(w.state.time, time, "{topic}: asking takes no time");
        let terrain = w.state.levels[OVERWORLD].terrain[end];
        assert_eq!(remembered(&w, end), Some(terrain), "{topic}: marked");
        let lines = said(&w);
        assert!(lines[lines.len() - 2].starts_with("Godgifu says: \""));
        assert_eq!(
            lines.last().map(String::as_str),
            Some(super::rumour::MARKED)
        );
        // A corridor of country, not the whole map.
        let marked = remembered_count(&w) - before;
        let level = &w.state.levels[OVERWORLD];
        let cells = (level.width() * level.height()) as usize;
        assert!(
            marked > 50 && marked < cells / 3,
            "{topic}: {marked} cells marked"
        );

        // Asking again marks nothing new and says only her reply.
        let count = remembered_count(&w);
        act(&mut w, &format!("ask:SE:{topic}"));
        assert_eq!(remembered_count(&w), count, "{topic}: idempotent");
        assert!(said(&w).last().unwrap().starts_with("Godgifu says: \""));
    }
}

#[test]
fn the_ways_are_seen_as_by_day_from_the_route() {
    let mut w = at_the_inn();
    act(&mut w, "ask:SE:barrows");
    let level = &w.state.levels[OVERWORLD];
    for y in 0..level.height() {
        for x in 0..level.width() {
            let p = Pos::new(x, y);
            if remembered(&w, p).is_some() {
                // Within day sight of the road or the way north to the barrow.
                let near = |q: Pos| q.dist2(p) <= crate::fov::radius2(crate::daylight::SIGHT_DAY);
                let route = [Pos::new(18, 28), Pos::new(34, 28), Pos::new(34, 26)];
                let on_route = (18..=34)
                    .map(|x| Pos::new(x, 28))
                    .chain([route[2], Pos::new(34, 27)]);
                assert!(on_route.clone().any(near), "{p} is out of sight of the way");
            }
        }
    }
    assert_eq!(
        remembered(&w, Pos::new(34, 10)),
        None,
        "far north stays unknown"
    );
}

#[test]
fn hidden_locations_stay_hidden_and_visible_ones_show() {
    let mut w = at_the_inn();
    for topic in ["barrows", "marsh", "mines", "volcano", "pass"] {
        act(&mut w, &format!("ask:SE:{topic}"));
    }
    assert!(w.state.revealed.is_empty(), "rumours are never a reveal");
    let template = CONTENT.level(OVERWORLD).expect("the overworld");
    let level = &w.state.levels[OVERWORLD];
    assert_eq!(
        level.hidden.len(),
        template.hidden.len(),
        "all still hidden"
    );
    let mut along_the_ways = 0;
    for (pos, hidden) in &template.hidden {
        assert!(level.hidden.contains_key(pos), "{} hidden", hidden.id);
        assert!(
            level.stairs.iter().all(|s| s.pos != *pos),
            "{} leads nowhere",
            hidden.id
        );
        if let Some(seen) = remembered(&w, *pos) {
            along_the_ways += 1;
            assert_eq!(
                seen, template.terrain[*pos],
                "{}: its plain terrain",
                hidden.id
            );
            assert_ne!(seen, hidden.terrain, "{}: not its entrance", hidden.id);
        }
    }
    // Fennmynster, Fyrdelf, and Bānfeld lie beside the ways, remembered as swamp and hills.
    assert!(
        along_the_ways >= 3,
        "{along_the_ways} hidden cells along the ways"
    );
    assert_eq!(remembered(&w, CEALDHLAW), Some(level.terrain[CEALDHLAW]));

    // Back on the map: the barrow is a known location, the hidden ones are not.
    act(&mut w, "ascend");
    assert_eq!(w.state.current_level, OVERWORLD);
    let locations = dump::layer_rows(&w, Layer::Locations).join("\n");
    assert!(locations.contains("(34, 25) Cealdhlāw"), "{locations}");
    for hidden in template.hidden.values() {
        assert!(
            !locations.contains(&hidden.name),
            "{} listed: {locations}",
            hidden.name
        );
    }
}

#[test]
fn the_memory_layer_shows_the_way_after_the_ask_and_not_before() {
    let row_at = |w: &World, p: Pos| {
        let rows = dump::layer_rows(w, Layer::Memory);
        let row = rows.get(p.y as usize).map_or("", String::as_str);
        row.chars().nth(p.x as usize).unwrap_or(' ')
    };
    let end = Pos::new(72, 45);
    let mut silent = at_the_inn();
    act(&mut silent, "ascend");
    assert_eq!(row_at(&silent, end), ' ', "unknown without the rumour");

    let mut told = at_the_inn();
    act(&mut told, "ask:SE:marsh ascend");
    assert_ne!(row_at(&told, end), ' ', "remembered after the rumour");
}

#[test]
fn a_rumour_replays_exactly() {
    let mut w = World::new(CONTENT.clone(), "ceaptun", None).expect("a run in Cēaptūn");
    let mut replay = Replay::new(&w);
    let actions = format!("{TO_THE_INN} ask:SE:mines ask:SE:mines ascend wait");
    for action in parse_actions(&actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(remembered(&w, Pos::new(95, 20)).is_some());
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(CONTENT.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    // The marks are the overworld's memory, which the save keeps.
    let level = &verified.state.levels[OVERWORLD];
    assert_eq!(level.memory, w.state.levels[OVERWORLD].memory);
}

#[test]
fn routes_are_validated() {
    let load = |from: &str, to: &str| {
        let mut files = ContentFiles::builtin();
        files.edit_table("dialogue", |t| t.replace(from, to));
        Content::load(&files).map(|_| ()).map_err(|e| e.message)
    };
    let pass = "route = [[18, 28], [12, 28]]";
    let cases = [
        (
            "route = [[18, 28], [11, 28]]",
            "leaves walkable ground at (11, 28)",
        ),
        ("route = [[18, 28], [500, 28]]", "leaves walkable ground"),
        ("route = [[18, 28]]", "at least two waypoints"),
    ];
    for (to, message) in cases {
        let err = load(pass, to).expect_err(to);
        assert!(
            err.contains("topic `pass`") && err.contains(message),
            "{to}: {err}"
        );
    }
    // Only a campaign's dialogue has an overworld to mark.
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/dialogue/rmp_wanderer.toml",
        "[[dialogue]]\nnpc = \"wanderer\"\nlines = [\"x\"]\n\
         [[dialogue.topic]]\nid = \"road\"\nname = \"The road\"\nreply = \"r\"\n\
         route = [[0, 0], [1, 0]]\n",
    );
    let err = Content::load(&files).expect_err("no overworld").message;
    assert!(err.contains("needs its campaign's overworld"), "{err}");
}
