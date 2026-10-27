//! The Ascmearc's overworld (#332; docs/design/campaign.md §5): the campaign and its map, its
//! regions, the start on Beorggeat (a closed way back) with its arrival line, the town five
//! road cells east, the named locations, the two hidden entrances (nothing of them reaches
//! perception before a reveal), the rivers crossed only at the bridge and the ford, the travel
//! budget, and the legend's `name` and `note`.

use super::*;
use crate::action::parse_actions;
use crate::content::{CampaignKind, ContentFiles, LevelTemplate};
use crate::dump::{self, Layer};
use crate::replay::Replay;
use std::collections::BinaryHeap;
use std::sync::LazyLock;

const LEVEL: &str = include_str!("../../../../assets/campaigns/ascmearc/levels/ascmearc.toml");
const PASS: Pos = Pos::new(12, 28);
const TOWN: Pos = Pos::new(18, 28);
const BARROW: Pos = Pos::new(34, 25);
const FENNMYNSTER: Pos = Pos::new(70, 48);
const FYRDELF: Pos = Pos::new(97, 21);
/// The four lairs of the wanted (the guild design §8), hidden on reserved sites.
const LAIRS: [(&str, Pos); 4] = [
    ("wyrtholm", Pos::new(88, 46)),
    ("wulfeshol", Pos::new(18, 49)),
    ("banfeld", Pos::new(42, 31)),
    ("brocenburg", Pos::new(66, 12)),
];

/// Built-in content with nobody hungering (the journeys here take days).
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    Arc::new(Content::load(&files).expect("builtin content"))
});

fn overworld() -> &'static LevelTemplate {
    CONTENT
        .overworld_of("ascmearc")
        .expect("the Ascmearc's overworld")
}

fn start() -> World {
    World::new(CONTENT.clone(), "ascmearc", None).expect("a run of the Ascmearc")
}

fn act(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.text.clone()).collect()
}

fn put_player(world: &mut World, at: Pos) {
    let id = world.state.player;
    world.state.actors.get_mut(&id).expect("player").pos = at;
    world.perceive(&mut Vec::new());
}

fn terrain_at(at: Pos) -> &'static str {
    &CONTENT.terrain(overworld().terrain[at]).id
}

/// The quickest journey from `from` to every cell, in minutes at normal speed, without
/// stepping onto terrain `except` (the overworld's travel times; 8 directions).
fn journeys(from: Pos, except: &[&str]) -> BTreeMap<Pos, u32> {
    let map = &overworld().terrain;
    let minutes = |p: Pos| {
        let t = CONTENT.terrain(map[p]);
        (t.walkable && !except.contains(&t.id.as_str())).then_some(u32::from(t.travel_minutes?))
    };
    let mut best = BTreeMap::from([(from, 0)]);
    let mut queue = BinaryHeap::from([(std::cmp::Reverse(0u32), from)]);
    while let Some((std::cmp::Reverse(d), p)) = queue.pop() {
        if best.get(&p).is_some_and(|b| *b < d) {
            continue;
        }
        for dir in Dir::ALL {
            let n = p.step(dir);
            let Some(m) = map.in_bounds(n).then(|| minutes(n)).flatten() else {
                continue;
            };
            if best.get(&n).is_none_or(|b| *b > d + m) {
                best.insert(n, d + m);
                queue.push((std::cmp::Reverse(d + m), n));
            }
        }
    }
    best
}

#[test]
fn the_ascmearc_is_a_campaign_with_a_120_by_60_overworld() {
    let campaign = (CONTENT.campaigns().iter())
        .find(|c| c.id == "ascmearc")
        .expect("the Ascmearc");
    assert_eq!(campaign.kind, CampaignKind::Campaign);
    assert_eq!(campaign.start, "ascmearc");
    let map = overworld();
    assert_eq!((map.terrain.width(), map.terrain.height()), (120, 60));
    assert_eq!(map.name, "The Ascmearc");
    let walkable = (map.terrain.cells().iter())
        .filter(|t| CONTENT.terrain(**t).walkable)
        .count();
    assert!(
        (4_500..=5_500).contains(&walkable),
        "about a third mountains and sea: {walkable} walkable cells"
    );
}

#[test]
fn every_walkable_cell_has_a_region_and_every_region_land() {
    let map = overworld();
    let mut used = BTreeSet::new();
    for y in 0..map.terrain.height() {
        for x in 0..map.terrain.width() {
            let p = Pos::new(x, y);
            if CONTENT.terrain(map.terrain[p]).walkable {
                used.insert(
                    map.region_at(p)
                        .unwrap_or_else(|| panic!("{p} has no region")),
                );
            }
        }
    }
    let names: BTreeSet<&str> = map.regions.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(used, names, "every region names some land");
    for (at, region) in [
        (PASS, "Grægweall"),
        (TOWN, "Cēapdenu"),
        (BARROW, "Hlāwdūn"),
        (FENNMYNSTER, "Micelfenn"),
        (FYRDELF, "Ascdūn"),
    ] {
        assert_eq!(map.region_at(at), Some(region), "{at}");
    }
}

#[test]
fn a_run_starts_on_beorggeat_with_its_arrival_line() {
    let w = start();
    assert_eq!(w.player_pos(), PASS);
    assert_eq!(w.perception().level_id(), "ascmearc");
    assert_eq!(terrain_at(PASS), "mountain_pass");
    assert_eq!(w.perception().region(), Some("Grægweall"));
    let lines = said(&w);
    assert!(
        lines[0].starts_with("Welcome to The Ascmearc."),
        "{lines:?}"
    );
    assert_eq!(
        lines[1],
        "The pass opens on the Ascmearc. Far to the east, Fyrbeorg smokes."
    );
    let place = w.perception().place(PASS).expect("Beorggeat is named");
    assert_eq!(place.name, "Beorggeat");
    assert_eq!(
        place.note.as_deref(),
        Some("The way you came; snow has closed it behind you.")
    );
}

#[test]
fn beorggeat_is_a_closed_way_back() {
    let map = overworld();
    assert!(map.stairs.iter().all(|s| s.pos != PASS), "no way out");
    let open: Vec<Pos> = (Dir::ALL.iter())
        .map(|d| PASS.step(*d))
        .filter(|p| CONTENT.terrain(map.terrain[*p]).walkable)
        .collect();
    assert_eq!(
        open,
        [PASS.step(Dir::E)],
        "the only way on is the road east"
    );
    let mut w = start();
    let before = w.time();
    act(&mut w, "descend move:W move:NW move:SW");
    assert_eq!(w.player_pos(), PASS);
    assert!(said(&w).contains(&"There is nothing to explore here.".to_string()));
    assert!(
        w.time().seconds() - before.seconds() < 60,
        "nothing but the usual time to try"
    );
}

#[test]
fn the_town_lies_five_road_cells_east_of_the_pass() {
    for x in PASS.x + 1..TOWN.x {
        assert_eq!(terrain_at(Pos::new(x, PASS.y)), "road", "({x}, 28)");
    }
    assert_eq!(terrain_at(TOWN), "town");
    let mut w = start();
    act(&mut w, "move:E*6");
    assert_eq!(w.player_pos(), TOWN);
    assert_eq!(said(&w).last().unwrap(), "The way into Cēaptūn is here.");
    assert_eq!(w.perception().leads_to(TOWN).as_deref(), Some("Cēaptūn"));
    // It leads into the town (#336; `ceaptun_tests`).
    act(&mut w, "descend");
    assert_eq!(w.state.current_level, "ceaptun");
}

#[test]
fn the_locations_are_named_and_six_cells_apart() {
    let map = overworld();
    let named: BTreeMap<Pos, (&str, &str)> = (map.stairs.iter())
        .map(|s| {
            let terrain = CONTENT.terrain(map.terrain[s.pos]).id.as_str();
            (s.pos, (terrain, s.name.as_deref().expect("a name")))
        })
        .collect();
    assert_eq!(
        named,
        BTreeMap::from([
            (BARROW, ("barrow", "Cealdhlāw")),
            (TOWN, ("town", "Cēaptūn"))
        ])
    );
    let (at, fen) = CONTENT
        .hidden_location("ascmearc", "fennmynster")
        .expect("Fennmynster");
    assert_eq!((at, terrain_at(at)), (FENNMYNSTER, "swamp"));
    assert_eq!(
        (CONTENT.terrain(fen.terrain).id.as_str(), fen.name.as_str()),
        ("dungeon_entrance", "Fennmynster")
    );
    let (at, fyr) = CONTENT
        .hidden_location("ascmearc", "fyrdelf")
        .expect("Fyrdelf");
    assert_eq!((at, terrain_at(at)), (FYRDELF, "hills"));
    assert_eq!(
        (CONTENT.terrain(fyr.terrain).id.as_str(), fyr.name.as_str()),
        ("mine", "Fyrdelf")
    );
    let mut all = vec![PASS, TOWN, BARROW, FENNMYNSTER, FYRDELF];
    for (id, at) in LAIRS {
        let (pos, _) = CONTENT.hidden_location("ascmearc", id).expect(id);
        assert_eq!(pos, at, "{id}");
        all.push(at);
    }
    for (i, a) in all.iter().enumerate() {
        for b in &all[i + 1..] {
            assert!(a.chebyshev(*b) >= 6, "{a} and {b} are too close");
        }
    }
    // No road leads to a hidden dungeon.
    for hidden in [FENNMYNSTER, FYRDELF]
        .into_iter()
        .chain(LAIRS.map(|(_, at)| at))
    {
        for d in Dir::ALL {
            assert_ne!(terrain_at(hidden.step(d)), "road", "a road beside {hidden}");
        }
    }
}

#[test]
fn the_rivers_are_crossed_only_at_the_bridge_and_the_ford() {
    let all = journeys(PASS, &[]);
    let walkable = (overworld().terrain.cells().iter())
        .filter(|t| CONTENT.terrain(**t).walkable)
        .count();
    assert_eq!(all.len(), walkable, "all the land is one");
    let dry = journeys(TOWN, &["bridge", "ford"]);
    for beyond in [FENNMYNSTER, FYRDELF, Pos::new(62, 43), Pos::new(94, 28)] {
        assert!(!dry.contains_key(&beyond), "{beyond} lies over a river");
    }
    assert!(dry.contains_key(&BARROW), "the barrow is on this side");
}

#[test]
fn the_travel_budget_matches_the_design() {
    let from_town = journeys(TOWN, &[]);
    let hours = |p: Pos| f64::from(from_town[&p]) / 60.0;
    assert!((1.5..3.0).contains(&hours(PASS)), "{}", hours(PASS));
    assert!(
        (6.0..12.0).contains(&hours(BARROW)),
        "the barrow within a day: {}",
        hours(BARROW)
    );
    for far in [FENNMYNSTER, FYRDELF] {
        assert!(
            (24.0..48.0).contains(&hours(far)),
            "{far}: about a day and a half: {}",
            hours(far)
        );
    }
}

#[test]
fn nothing_of_the_hidden_dungeons_reaches_perception_or_the_dumps() {
    let mut w = start();
    let swamp = CONTENT.terrain(CONTENT.terrain_id("swamp").unwrap()).glyph;
    put_player(&mut w, FENNMYNSTER.step(Dir::N).step(Dir::N));
    assert!(w.perception().is_visible(FENNMYNSTER));
    let rows = dump::layer_rows(&w, Layer::View);
    let seen = rows[FENNMYNSTER.y as usize]
        .chars()
        .nth(FENNMYNSTER.x as usize);
    assert_eq!(seen, Some(swamp));
    let json = dump::json(&w, &[Layer::View, Layer::Memory, Layer::Locations], false).to_string();
    let text = (dump::layer_rows(&w, Layer::Locations)).join("\n");
    for leak in [
        "ennmynster",
        "yrdelf",
        "yrtholm",
        "ulfeshol",
        "nfeld",
        "rocenburg",
    ] {
        assert!(!json.contains(leak), "{leak} in the JSON dump");
        assert!(!text.contains(leak), "{leak} in the locations");
    }
    assert_eq!(w.perception().leads_to(FENNMYNSTER), None);
    // The dev reveal marks it, and it leads down into its branch (#338).
    act(&mut w, "reveal:fennmynster");
    assert_eq!(
        w.perception().leads_to(FENNMYNSTER).as_deref(),
        Some("Fennmynster")
    );
    put_player(&mut w, FENNMYNSTER);
    act(&mut w, "descend");
    assert_eq!(w.perception().level_id(), "fennmynster:1");
}

#[test]
fn a_run_of_the_ascmearc_replays_exactly() {
    let mut w = start();
    let mut replay = Replay::new(&w);
    for action in parse_actions("move:E*6 move:S*3 wait reveal:fyrdelf").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(CONTENT.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}

#[test]
fn the_legend_names_places_and_refuses_what_would_give_a_hidden_one_away() {
    let pass = r#""P" = { terrain = "mountain_pass", spawn = "player", name = "Beorggeat", note = "The way you came; snow has closed it behind you." }"#;
    assert!(LEVEL.contains(pass), "the pass's legend line");
    let with = |line: &str| {
        let mut files = ContentFiles::builtin();
        files.insert(
            "campaigns/ascmearc/levels/ascmearc.toml",
            &LEVEL.replace(pass, line),
        );
        Content::load(&files).expect_err("refused").to_string()
    };
    for (line, says) in [
        (
            r#""P" = { terrain = "mountain_pass", spawn = "player", note = "A note." }"#,
            "a `note` needs a `name`",
        ),
        (
            r#""P" = { terrain = "mountain_pass", spawn = "player", name = " " }"#,
            "the `name` is empty",
        ),
        (
            r#""P" = { terrain = "mountain_pass", spawn = "player", name = "Beorggeat", hidden = { id = "gate", terrain = "town", name = "Gate" } }"#,
            "its name in `hidden` only",
        ),
    ] {
        let error = with(line);
        assert!(error.contains(says), "{line}: {error}");
    }
}
