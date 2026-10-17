//! Text and JSON dumps of the simulation for developers and agents (spec §9).
//!
//! Every layer except [`Layer::Truth`] is built from [`World::perception`] only, so dumps cannot
//! leak information the player does not have. `Truth` reads ground truth and is dev-only.

use crate::geom::Pos;
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
    /// What the player sees: actors on visible cells, remembered terrain, unknown as space.
    View,
    /// `*` on every currently visible cell.
    Fov,
    /// Terrain glyphs for every cell the player has seen (visible or remembered).
    Memory,
    /// Perceived light on every visible cell as a digit, `0` (dark) to `9` (fully lit or
    /// brighter); see [`light_digit`].
    Light,
    /// Ground truth: all terrain and all actors on the level. **Dev-only.**
    Truth,
}

impl Layer {
    /// Layer name as used on the command line and in JSON.
    pub const fn name(self) -> &'static str {
        match self {
            Layer::View => "view",
            Layer::Fov => "fov",
            Layer::Memory => "memory",
            Layer::Light => "light",
            Layer::Truth => "truth",
        }
    }
}

impl FromStr for Layer {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        [
            Layer::View,
            Layer::Fov,
            Layer::Memory,
            Layer::Light,
            Layer::Truth,
        ]
        .into_iter()
        .find(|l| l.name() == s)
        .ok_or_else(|| format!("unknown layer `{s}` (expected view, fov, memory, light, truth)"))
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
    match layer {
        Layer::View => match view.actor_at(p) {
            Some(actor) => content.creature(actor.creature).glyph,
            None => terrain_glyph(cell),
        },
        Layer::Fov => {
            if view.is_visible(p) {
                '*'
            } else {
                ' '
            }
        }
        Layer::Memory => terrain_glyph(cell),
        Layer::Light => view.light(p).map_or(' ', light_digit),
        Layer::Truth => unreachable!("truth is not built from perception"),
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
        match actors.iter().find(|a| a.pos == p) {
            Some(actor) => content.creature(actor.creature).glyph,
            None => level
                .terrain_at(p)
                .map_or(' ', |t| content.terrain(t).glyph),
        }
    })
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
        "turn {} level {} viewer {} hash {:016x}",
        world.turn(),
        view.level_id(),
        view.viewer(),
        world.state_hash()
    )
}

/// Plain-text dump: a header line, then each requested layer under a `== name ==` heading.
/// The truth layer is headed `== truth (DEV ONLY: ground truth) ==`.
pub fn ascii(world: &World, layers: &[Layer]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{}", summary(world));
    for &layer in layers {
        let note = if layer == Layer::Truth {
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

/// Structured dump: turn, level, viewer, seen actors, recent messages, the requested layers as
/// arrays of rows, and optionally the full [`crate::WorldState`] under `state`.
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
    let layer_map: serde_json::Map<String, Value> = layers
        .iter()
        .map(|&l| (l.name().to_string(), json!(layer_rows(world, l))))
        .collect();
    let mut out = json!({
        "game_version": crate::GAME_VERSION,
        "scenario": world.scenario(),
        "seed": world.seed(),
        "turn": world.turn(),
        "level": view.level_id(),
        "viewer": view.viewer(),
        "hash": format!("{:016x}", world.state_hash()),
        "seen_actors": seen,
        "messages": messages,
        "layers": layer_map,
    });
    if include_state {
        out["state"] = serde_json::to_value(world.state()).expect("state serializes");
    }
    out
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
