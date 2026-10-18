//! Text and JSON dumps of the simulation for developers and agents (spec §9).
//!
//! Every layer except [`Layer::Truth`] and [`Layer::Actors`] is built from [`World::perception`]
//! only, so dumps cannot leak information the player does not have. `Truth` and `Actors` read
//! ground truth and are dev-only.

use crate::action::letter;
use crate::geom::Pos;
use crate::noise::{PerceivedSound, SoundKind, Volume};
use crate::object::format_weight;
use crate::perception::{CellView, PerceivedLight, PerceptionView};
use crate::world::World;
use serde_json::{Value, json};
use std::fmt::Write;
use std::str::FromStr;

/// Number of recent messages included in a dump.
const RECENT_MESSAGES: usize = 10;

/// A map layer to render.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    /// What the player sees: actors on visible cells, then the top object of each pile in sight
    /// or remembered, then terrain seen or remembered; unknown as space.
    View,
    /// `*` on every currently visible cell.
    Fov,
    /// Terrain glyphs for every cell the player has seen (visible or remembered), with the top
    /// object of each known pile over them.
    Memory,
    /// Perceived light on every visible cell as a digit, `0` (dark) to `9` (fully lit or
    /// brighter); see [`light_digit`].
    Light,
    /// Sounds the player heard from unseen sources, at their rough places ([`sound_char`]: `f`
    /// footsteps, `o` a door opening, `c` a door closing; upper case when loud), over the view.
    Sound,
    /// Not a map: the player's inventory, one line per stack, then what they carry and weigh.
    Inventory,
    /// Ground truth: all terrain, objects, and actors on the level. **Dev-only.**
    Truth,
    /// Not a map: every actor on the level with its weight and what it carries (ground truth).
    /// **Dev-only.**
    Actors,
}

impl Layer {
    const ALL: [Layer; 8] = [
        Layer::View,
        Layer::Fov,
        Layer::Memory,
        Layer::Light,
        Layer::Sound,
        Layer::Inventory,
        Layer::Truth,
        Layer::Actors,
    ];

    /// Layer name as used on the command line and in JSON.
    pub const fn name(self) -> &'static str {
        match self {
            Layer::View => "view",
            Layer::Fov => "fov",
            Layer::Memory => "memory",
            Layer::Light => "light",
            Layer::Sound => "sound",
            Layer::Inventory => "inventory",
            Layer::Truth => "truth",
            Layer::Actors => "actors",
        }
    }

    /// Whether the layer reads ground truth.
    pub const fn is_dev_only(self) -> bool {
        matches!(self, Layer::Truth | Layer::Actors)
    }
}

impl FromStr for Layer {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Layer::ALL
            .into_iter()
            .find(|l| l.name() == s)
            .ok_or_else(|| {
                format!(
                    "unknown layer `{s}` (expected view, fov, memory, light, sound, inventory, \
                     truth, actors)"
                )
            })
    }
}

/// Parses a comma-separated layer list such as `view,fov`.
pub fn parse_layers(text: &str) -> Result<Vec<Layer>, String> {
    text.split(',').map(|l| l.trim().parse()).collect()
}

/// Renders one layer as rows of text with trailing spaces trimmed.
pub fn layer_rows(world: &World, layer: Layer) -> Vec<String> {
    match layer {
        Layer::Truth => truth_rows(world),
        Layer::Actors => actor_rows(world),
        Layer::Inventory => inventory_rows(&world.perception()),
        _ => {
            let view = world.perception();
            rows(view.width(), view.height(), |p| {
                perceived_glyph(&view, layer, p)
            })
        }
    }
}

fn perceived_glyph(view: &PerceptionView<'_>, layer: Layer, p: Pos) -> char {
    let content = view.content();
    let cell = view.cell(p);
    let terrain_glyph = |cell: CellView| match cell {
        CellView::Unknown => ' ',
        CellView::Remembered(t) | CellView::Visible(t) => content.terrain(t).glyph,
    };
    let known_glyph = |cell: CellView| match view.objects(p).first() {
        Some(top) => content.object(top.object).glyph,
        None => terrain_glyph(cell),
    };
    match layer {
        Layer::View => match view.actor_at(p) {
            Some(actor) => content.creature(actor.creature).glyph,
            None => known_glyph(cell),
        },
        Layer::Fov => {
            if view.is_visible(p) {
                '*'
            } else {
                ' '
            }
        }
        Layer::Memory => known_glyph(cell),
        Layer::Light => view.light(p).map_or(' ', light_digit),
        Layer::Sound => match view.sound_at(p) {
            Some(sound) => sound_char(sound),
            None => perceived_glyph(view, Layer::View, p),
        },
        Layer::Inventory | Layer::Truth | Layer::Actors => {
            unreachable!("not a map built from perception")
        }
    }
}

/// A heard sound in the `sound` layer: `f` footsteps, `o` a door opening, `c` a door closing;
/// upper case when loud.
pub fn sound_char(sound: &PerceivedSound) -> char {
    let c = match sound.kind {
        SoundKind::Footsteps => 'f',
        SoundKind::DoorOpened => 'o',
        SoundKind::DoorClosed => 'c',
    };
    if sound.volume == Volume::Loud {
        c.to_ascii_uppercase()
    } else {
        c
    }
}

/// Perceived brightness ([`PerceivedLight::brightness`]: luma, plus UV for eyes that see it) in
/// tenths of full light: `0` below 26, `9` from 230 up (overbright included).
pub fn light_digit(light: PerceivedLight) -> char {
    let tenths = (u32::from(light.brightness()) * 10 / 256).min(9);
    char::from_digit(tenths, 10).unwrap_or('9')
}

fn truth_rows(world: &World) -> Vec<String> {
    let content = world.content();
    let (level, actors) = world.ground_truth();
    let actors: Vec<_> = actors.collect();
    rows(level.width(), level.height(), |p| {
        match (
            actors.iter().find(|a| a.pos == p),
            level.objects_at(p).first(),
        ) {
            (Some(actor), _) => content.creature(actor.creature).glyph,
            (None, Some(top)) => content.object(top.object).glyph,
            (None, None) => level
                .terrain_at(p)
                .map_or(' ', |t| content.terrain(t).glyph),
        }
    })
}

/// The player's inventory (`a - a dagger (450 g)`), then what they carry and weigh.
fn inventory_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let content = view.content();
    let mut rows: Vec<String> = view
        .inventory()
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let name = s.describe(content);
            let weight = format_weight(s.weight(content));
            format!("{} - {name} ({weight})", letter(i).unwrap_or('?'))
        })
        .collect();
    let weight = view.own_weight();
    rows.push(format!(
        "carried {}, total weight {}",
        format_weight(weight.carried),
        format_weight(weight.total())
    ));
    rows
}

/// Every actor on the level (ground truth): id, creature, position, weight, and what it carries.
fn actor_rows(world: &World) -> Vec<String> {
    let content = world.content();
    let (_, actors) = world.ground_truth();
    actors
        .map(|a| {
            let w = a.weight(content);
            let carried: Vec<String> = a.inventory.iter().map(|s| s.describe(content)).collect();
            let carried = if carried.is_empty() {
                "nothing".to_string()
            } else {
                carried.join(", ")
            };
            format!(
                "{} {} at {}: weight {} (base {} + carried {}: {carried})",
                a.id,
                content.creature(a.creature).id,
                a.pos,
                format_weight(w.total()),
                format_weight(w.base),
                format_weight(w.carried),
            )
        })
        .collect()
}

fn rows(width: i32, height: i32, glyph: impl Fn(Pos) -> char) -> Vec<String> {
    (0..height)
        .map(|y| {
            let row: String = (0..width).map(|x| glyph(Pos::new(x, y))).collect();
            row.trim_end().to_string()
        })
        .collect()
}

/// One-line summary used as the dump header.
fn summary(world: &World) -> String {
    let view = world.perception();
    format!(
        "turn {} time {} level {} viewer {} hash {:016x}",
        world.turn(),
        world.time(),
        view.level_id(),
        view.viewer(),
        world.state_hash()
    )
}

/// Plain-text dump: a header line, then each requested layer under a `== name ==` heading.
/// Dev-only layers are headed like `== truth (DEV ONLY: ground truth) ==`.
pub fn ascii(world: &World, layers: &[Layer]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{}", summary(world));
    for &layer in layers {
        let note = if layer.is_dev_only() {
            " (DEV ONLY: ground truth)"
        } else {
            ""
        };
        let _ = writeln!(out, "== {}{note} ==", layer.name());
        for row in layer_rows(world, layer) {
            let _ = writeln!(out, "{row}");
        }
    }
    out
}

/// Structured dump: turn, game time, level, viewer, seen actors, heard sounds, the player's
/// inventory and weight, recent messages, the requested layers as arrays of rows, and optionally
/// the full [`crate::WorldState`] under `state`.
pub fn json(world: &World, layers: &[Layer], include_state: bool) -> Value {
    let view = world.perception();
    let content = world.content();
    let seen: Vec<Value> = view
        .actors()
        .iter()
        .map(|a| {
            let def = content.creature(a.creature);
            json!({ "id": a.id, "kind": a.kind, "creature": def.id, "glyph": def.glyph.to_string(), "pos": a.pos })
        })
        .collect();
    let messages: Vec<&crate::messages::Message> = {
        let all: Vec<_> = world.messages().iter().collect();
        all[all.len().saturating_sub(RECENT_MESSAGES)..].to_vec()
    };
    let inventory: Vec<Value> = view
        .inventory()
        .iter()
        .enumerate()
        .map(|(i, s)| {
            json!({
                "letter": letter(i).map(String::from),
                "object": content.object(s.object).id,
                "count": s.count,
                "weight": s.weight(content),
            })
        })
        .collect();
    let weight = view.own_weight();
    let layer_map: serde_json::Map<String, Value> = layers
        .iter()
        .map(|&l| (l.name().to_string(), json!(layer_rows(world, l))))
        .collect();
    let mut out = json!({
        "game_version": crate::GAME_VERSION,
        "scenario": world.scenario(),
        "seed": world.seed(),
        "turn": world.turn(),
        "time": time_json(world),
        "level": view.level_id(),
        "viewer": view.viewer(),
        "hash": format!("{:016x}", world.state_hash()),
        "seen_actors": seen,
        "heard_sounds": view.sounds(),
        "inventory": inventory,
        "weight": { "base": weight.base, "carried": weight.carried, "total": weight.total() },
        "messages": messages,
        "layers": layer_map,
    });
    if include_state {
        out["state"] = serde_json::to_value(world.state()).expect("state serializes");
    }
    out
}

/// The clock in seconds since the epoch, and as a calendar date and time of day.
fn time_json(world: &World) -> Value {
    let t = world.time();
    json!({
        "seconds": t.seconds(),
        "text": t.to_string(),
        "year": t.year(),
        "month": t.month(),
        "month_name": world.content().calendar().month_name(t),
        "day": t.day(),
        "hour": t.hour(),
        "minute": t.minute(),
        "second": t.second(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Action, parse_actions};
    use crate::content::Content;
    use std::sync::Arc;

    /// Walks north out of the start room and through the first door, then along the corridor.
    const SHOWCASE_RUN: &str = "move:N move:N move:N move:N move:N move:N move:E move:E";

    fn showcase() -> World {
        let content = Arc::new(Content::builtin().expect("builtin content"));
        World::new(content, "showcase", None).expect("world")
    }

    fn run(world: &mut World, actions: &str) {
        for action in parse_actions(actions).expect("actions") {
            world.apply(&action);
        }
    }

    #[test]
    fn parses_layer_lists() {
        assert_eq!(
            parse_layers("view, fov,truth"),
            Ok(vec![Layer::View, Layer::Fov, Layer::Truth])
        );
        assert_eq!(parse_layers("light"), Ok(vec![Layer::Light]));
        assert!(parse_layers("view,lights").is_err());
    }

    #[test]
    fn view_hides_unseen_actors_that_truth_shows() {
        let world = showcase();
        let view = layer_rows(&world, Layer::View).join("\n");
        let truth = layer_rows(&world, Layer::Truth).join("\n");
        assert!(truth.contains('r') || truth.contains('k'));
        assert_eq!(view.matches('@').count(), 1);
        assert!(view.chars().filter(|c| *c != '\n').all(|c| c != 'k'));
    }

    #[test]
    fn json_has_documented_fields() {
        let mut world = showcase();
        world.apply(&Action::Wait);
        let value = json(&world, &[Layer::View, Layer::Fov], true);
        assert_eq!(value["turn"], 1);
        assert_eq!(value["time"]["text"], "0001-01-01 08:00:06");
        assert_eq!(value["time"]["month_name"], "Frostmere");
        assert_eq!(value["level"], "hub");
        assert_eq!(value["layers"]["fov"].as_array().map(Vec::len), Some(21));
        assert!(value["layers"].get("memory").is_none());
        assert_eq!(value["state"]["turn"], 1);
        assert!(value["seen_actors"][0]["kind"] == "player");
        assert!(json(&world, &[], false).get("state").is_none());
    }

    #[test]
    fn showcase_snapshots() {
        let mut world = showcase();
        let layers = [Layer::View, Layer::Fov];
        insta::assert_snapshot!("showcase_start", ascii(&world, &layers));
        run(&mut world, &SHOWCASE_RUN[..SHOWCASE_RUN.len() / 2]);
        insta::assert_snapshot!("showcase_mid", ascii(&world, &layers));
        let mut world = showcase();
        run(&mut world, SHOWCASE_RUN);
        insta::assert_snapshot!("showcase_end", ascii(&world, &[Layer::View, Layer::Memory]));
    }

    #[test]
    fn light_digits_are_tenths_of_full_light() {
        let light = |rgb: [u16; 3], uv: u16| PerceivedLight {
            rgb,
            uv,
            darkvision: false,
            dist2: 0,
        };
        assert_eq!(light_digit(light([0, 0, 0], 0)), '0');
        assert_eq!(light_digit(light([25, 25, 25], 0)), '0');
        assert_eq!(light_digit(light([26, 26, 26], 0)), '1');
        assert_eq!(light_digit(light([128, 128, 128], 0)), '5');
        assert_eq!(light_digit(light([255, 255, 255], 0)), '9');
        assert_eq!(light_digit(light([900, 900, 900], 0)), '9');
        assert_eq!(
            light_digit(light([0, 0, 0], 128)),
            '5',
            "perceived UV counts"
        );
    }

    #[test]
    fn light_layer_shows_visible_cells_only() {
        let world = showcase();
        insta::assert_snapshot!("showcase_light", ascii(&world, &[Layer::Light]));
        let rows = layer_rows(&world, Layer::Light);
        let fov = layer_rows(&world, Layer::Fov);
        for (light, fov) in rows.iter().zip(&fov) {
            assert_eq!(light.len(), fov.len());
            for (l, f) in light.chars().zip(fov.chars()) {
                assert_eq!(l.is_ascii_digit(), f == '*');
            }
        }
    }
}
