//! Map content on the free-glyph layer (spec §4.2): creatures at their sliding positions, and
//! visible terrain whose animation outgrows its cell (`pulse`). Rebuilt every frame.
//!
//! Each glyph is lit by sampling the visual light at its center (`VisualLight::sample`), the
//! same interpolated light the grid shader uses there, so a sliding creature's light changes
//! smoothly and flickers with the torches around it.

use super::motion::Motion;
use crate::game::Game;
use crate::render::anim::{self, AnimFrame};
use crate::render::free_glyph::{FreeGlyph, FreeGlyphs};
use crate::render::grid::MapGrid;
use crate::render::lighting::{self, ACTOR_FLOOR, TERRAIN_FLOOR, VisualLight};
use crate::render::{Effects, VisualTime};
use crate::settings::{Role, Settings};
use bevy::prelude::*;
use warlock_core::Pos;
use warlock_core::color::Rgb;
use warlock_core::world::ActorId;

pub(super) fn emit(
    game: Res<Game>,
    grid: Res<MapGrid>,
    motion: Res<Motion>,
    visual: Res<VisualTime>,
    effects: Res<Effects>,
    settings: Res<Settings>,
    light: Res<VisualLight>,
    mut free: ResMut<FreeGlyphs>,
) {
    let view = game.world.perception();
    let content = view.content();
    let time = visual.seconds;
    let palette = |c: Rgb| settings.palette.apply(c, Role::Fg);
    free.clip = grid.clip;
    // Terrain first, so creatures draw on top.
    for &(p, terrain) in &grid.free_terrain {
        let def = content.terrain(terrain);
        let frame = def.anim.as_ref().map_or(AnimFrame::STILL, |a| {
            anim::evaluate(a, time, anim::cell_phase(p))
        });
        let at = Vec2::new(p.x as f32, p.y as f32);
        let mut g = glyph(&grid, at, def.glyph, palette(def.fg), frame);
        g.color = lit(g.color, &light, at, at, TERRAIN_FLOOR);
        free.push(g);
    }
    for actor in view.actors() {
        let def = content.creature(actor.creature);
        let frame = match &def.anim {
            Some(a) if !effects.reduced => anim::evaluate(a, time, creature_phase(actor.id)),
            _ => AnimFrame::STILL,
        };
        let at = motion.actor_pos(actor.id, actor.pos, time);
        let home = Vec2::new(actor.pos.x as f32, actor.pos.y as f32);
        let mut g = glyph(&grid, at, def.glyph, palette(def.fg), frame);
        g.color = lit(g.color, &light, at, home, ACTOR_FLOOR);
        free.push(g);
    }
}

/// `color` lit by the visual light at the center of the cell-sized glyph at `at` (level cells).
/// Where that point is not lit (a creature sliding in from the dark), the light at the center of
/// `home`, the cell the glyph belongs to, is used; with neither, the color is left as it is.
fn lit(color: LinearRgba, light: &VisualLight, at: Vec2, home: Vec2, floor: f32) -> LinearRgba {
    let center = Vec2::splat(0.5);
    let Some(cell) = light
        .sample(at + center)
        .or_else(|| light.sample(home + center))
    else {
        return color;
    };
    let rgb = lighting::shade(Vec3::new(color.red, color.green, color.blue), cell, floor);
    LinearRgba::new(rgb.x, rgb.y, rgb.z, color.alpha)
}

/// A creature's animation phase, keyed by identity rather than position so it does not jump
/// as the creature moves.
fn creature_phase(id: ActorId) -> f32 {
    anim::cell_phase(Pos::new(id.0 as i32, i32::MIN))
}

fn linear(c: Rgb) -> LinearRgba {
    Color::srgb_u8(c.r, c.g, c.b).to_linear()
}

/// The glyph `ch` (colored `fg`) of level position `at`, animated by `frame`.
fn glyph(grid: &MapGrid, at: Vec2, ch: char, fg: Rgb, frame: AnimFrame) -> FreeGlyph {
    let mut color = linear(fg);
    if let Some((tint, weight)) = frame.tint {
        let tint = linear(tint);
        color = LinearRgba::rgb(
            color.red + (tint.red - color.red) * weight,
            color.green + (tint.green - color.green) * weight,
            color.blue + (tint.blue - color.blue) * weight,
        );
    }
    FreeGlyph {
        pos: grid.to_screen(at) + frame.offset * grid.cell.x,
        ch: frame.glyph.unwrap_or(ch),
        color,
        scale: frame.scale,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn still_glyphs_sit_on_the_grid() {
        let grid = MapGrid {
            level_origin: IVec2::new(2, 3),
            origin: Vec2::new(10.0, 20.0),
            cell: Vec2::new(11.0, 22.0),
            ..default()
        };
        let g = glyph(
            &grid,
            Vec2::new(4.0, 3.0),
            '@',
            Rgb::WHITE,
            AnimFrame::STILL,
        );
        assert_eq!(g.pos, Vec2::new(32.0, 20.0));
        assert!(g.is_crisp());
        let swaying = AnimFrame {
            offset: Vec2::new(0.1, 0.0),
            glyph: Some('≈'),
            ..AnimFrame::STILL
        };
        let g = glyph(&grid, Vec2::new(4.0, 3.0), '~', Rgb::WHITE, swaying);
        assert!((g.pos.x - 33.1).abs() < 1e-4 && g.ch == '≈');
        assert!(!g.is_crisp());
    }

    #[test]
    fn tint_blends_toward_the_shimmer_color() {
        let grid = MapGrid::default();
        let frame = AnimFrame {
            tint: Some((Rgb::WHITE, 1.0)),
            ..AnimFrame::STILL
        };
        let g = glyph(&grid, Vec2::ZERO, '*', Rgb::BLACK, frame);
        assert_eq!(g.color, LinearRgba::WHITE);
    }

    #[test]
    fn creature_phases_differ() {
        assert_ne!(creature_phase(ActorId(1)), creature_phase(ActorId(2)));
        assert!((0.0..1.0).contains(&creature_phase(ActorId(7))));
    }

    #[test]
    fn glyphs_are_lit_where_they_are_or_at_home() {
        let content = std::sync::Arc::new(
            warlock_core::content::Content::builtin().expect("builtin content"),
        );
        let world = warlock_core::World::new(content, "showcase", None).expect("world");
        let view = world.perception();
        let mut light = VisualLight::default();
        let size = UVec2::new(view.width() as u32, view.height() as u32);
        light.composite(&view, IVec2::ZERO, size, None);

        let color = LinearRgba::new(0.5, 0.5, 0.5, 0.8);
        let home = Vec2::new(7.0, 15.0);
        let at_home = lit(color, &light, home, home, ACTOR_FLOOR);
        assert_ne!(at_home, color, "lit by the start room's torches");
        assert_eq!(at_home.alpha, 0.8);
        // Halfway in from an unseen cell: lit as at home. Nowhere lit: unchanged.
        let outside = Vec2::new(5.0, 10.0);
        assert_eq!(lit(color, &light, outside, home, ACTOR_FLOOR), at_home);
        assert_eq!(lit(color, &light, outside, outside, ACTOR_FLOOR), color);
    }
}
