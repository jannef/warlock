//! Map content on the free-glyph layer (spec §4.2): creatures at their sliding positions, and
//! visible terrain whose animation outgrows its cell (`pulse`). Rebuilt every frame.
//!
//! Each glyph is lit by sampling the visual light at its center (`VisualLight::sample`), the
//! same interpolated light the grid shader uses there, so a sliding creature's light changes
//! smoothly and flickers with the torches around it.
//!
//! The unaware mark (spec §4.2; Stalker's eye): a creature the snapshot says has not noticed
//! the player (`SeenActor::unaware`) is drawn a shade cooler, its colour mixed toward
//! [`UNAWARE_TINT`] before lighting, the mix breathing slowly with glyph animation on and
//! standing still without it.

use super::motion::Motion;
use crate::game::Game;
use crate::render::anim::{self, AnimFrame};
use crate::render::free_glyph::{Bands, FreeGlyph, FreeGlyphs};
use crate::render::grid::MapGrid;
use crate::render::lighting::{self, ACTOR_FLOOR, TERRAIN_FLOOR, VisualLight};
use crate::render::post::{emissive, in_range};
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
            through_palette(anim::evaluate(a, time, anim::cell_phase(p)), &settings)
        });
        let at = Vec2::new(p.x as f32, p.y as f32);
        let mut g = glyph(&grid, at, def.glyph, palette(def.fg), frame);
        g.color = lit(g.color, &light, at, at, TERRAIN_FLOOR);
        free.push(glow(g, def.glow, &effects));
    }
    for actor in view.actors() {
        let def = content.creature(actor.creature);
        let frame = match &def.anim {
            Some(a) if effects.glyph_animation => {
                through_palette(anim::evaluate(a, time, creature_phase(actor.id)), &settings)
            }
            _ => AnimFrame::STILL,
        };
        let at = motion.actor_pos(actor.id, actor.pos, time);
        let home = Vec2::new(actor.pos.x as f32, actor.pos.y as f32);
        let colours = actor.shape.as_ref().map_or(&[][..], |s| s.bands());
        let fg = colours.first().copied().unwrap_or(actor.fg(content));
        // The unaware mark, before the palette and the light, on the glyph and every band.
        let mix = unaware_mix(actor.id, time, effects.glyph_animation);
        let colour = |c: Rgb| {
            palette(if actor.unaware {
                unaware_tint(c, mix)
            } else {
                c
            })
        };
        let mut g = glyph(&grid, at, actor.glyph(content), colour(fg), frame);
        // Each band is lit and glows as the glyph does: they share its light.
        let finish = |color| {
            let color = lit(color, &light, at, home, ACTOR_FLOOR);
            glow(FreeGlyph { color, ..g }, def.glow, &effects).color
        };
        if colours.len() < 2 {
            g.color = finish(g.color);
            free.push(g);
            continue;
        }
        let mut bands = Bands {
            count: colours.len() as u8,
            drift: band_drift(actor.id, time, effects.glyph_animation),
            ..Bands::NONE
        };
        for (band, &c) in bands.more.iter_mut().zip(&colours[1..]) {
            *band = finish(glyph(&grid, at, ' ', colour(c), frame).color);
        }
        g.color = finish(g.color);
        free.push_banded(g, bands);
    }
}

/// The cool slate an unaware creature's colour is mixed toward (the unaware mark): a near-grey,
/// so the creature keeps its hue, just quieter. It goes through the palette with the glyph.
pub(super) const UNAWARE_TINT: Rgb = Rgb::new(0x8a, 0x96, 0xa8);

/// How much of [`UNAWARE_TINT`] an unaware creature's colour takes, still (reduced effects).
const UNAWARE_MIX: f32 = 0.25;

/// How far the mix breathes either side of [`UNAWARE_MIX`] with glyph animation on.
const UNAWARE_BREATH: f32 = 0.1;

/// Seconds for one breath of the unaware mark.
const UNAWARE_PERIOD: f32 = 6.0;

/// How much of [`UNAWARE_TINT`] creature `id`'s colour takes at `time`: breathing between
/// 0.15 and 0.35 on its own phase, or a still 0.25 without glyph animation.
fn unaware_mix(id: ActorId, time: f32, animate: bool) -> f32 {
    if !animate {
        return UNAWARE_MIX;
    }
    let breath = anim::breathe(time, UNAWARE_PERIOD, creature_phase(id));
    UNAWARE_MIX - UNAWARE_BREATH + 2.0 * UNAWARE_BREATH * breath
}

/// `fg` mixed toward [`UNAWARE_TINT`] by `mix` (0 none, 1 all): the unaware mark's colour.
pub(super) fn unaware_tint(fg: Rgb, mix: f32) -> Rgb {
    let channel = |from: u8, to: u8| {
        let (from, to) = (f32::from(from), f32::from(to));
        (from + (to - from) * mix.clamp(0.0, 1.0)).round() as u8
    };
    let t = UNAWARE_TINT;
    Rgb::new(channel(fg.r, t.r), channel(fg.g, t.g), channel(fg.b, t.b))
}

/// How far, in band heights, the borders between a composite's colour bands have drifted at
/// `time`: a slow sway up and down, a creature's own phase, still with glyph animation off.
fn band_drift(id: ActorId, time: f32, animate: bool) -> f32 {
    /// The most the borders move either way.
    const SWAY: f32 = 0.3;
    /// Seconds for one sway down and back.
    const PERIOD: f32 = 7.0;
    if !animate {
        return 0.0;
    }
    let turn = time / PERIOD + creature_phase(id);
    SWAY * (turn * std::f32::consts::TAU).sin()
}

/// `frame` with its shimmer tint color run through the active palette, like every other color.
fn through_palette(mut frame: AnimFrame, settings: &Settings) -> AnimFrame {
    frame.tint = frame
        .tint
        .map(|(color, weight)| (settings.palette.apply(color, Role::Fg), weight));
    frame
}

/// `color` lit by the visual light at the center of the cell-sized glyph at `at` (level cells).
/// Where that point is not lit (a creature sliding in from the dark), the light at the center of
/// `home`, the cell the glyph belongs to, is used; with neither, the color is left as it is.
/// While a spell effect darkens the view, the readability floor (or the unlit color) darkens
/// with it, as on the map.
pub(super) fn lit(
    color: LinearRgba,
    light: &VisualLight,
    at: Vec2,
    home: Vec2,
    floor: f32,
) -> LinearRgba {
    let center = Vec2::splat(0.5);
    let Some(cell) = light
        .sample(at + center)
        .or_else(|| light.sample(home + center))
    else {
        return darkened(color, light.keep());
    };
    let floor = floor * light.keep();
    let rgb = lighting::shade(Vec3::new(color.red, color.green, color.blue), cell, floor);
    LinearRgba::new(rgb.x, rgb.y, rgb.z, color.alpha)
}

/// `color` with its red, green, and blue scaled by `keep` (a spell effect's darkening), alpha
/// kept: for glyphs that are light themselves (sparks, a burning missile) or unlit.
pub(super) fn darkened(color: LinearRgba, keep: f32) -> LinearRgba {
    LinearRgba::new(
        color.red * keep,
        color.green * keep,
        color.blue * keep,
        color.alpha,
    )
}

/// With effects on, keeps a lit glyph within display range (bright light alone never blooms) and
/// makes it glow (bloom) by `amount`, as the grid shader does.
fn glow(mut g: FreeGlyph, amount: f32, effects: &Effects) -> FreeGlyph {
    if effects.glow {
        g.color = emissive(in_range(g.color), amount);
    }
    g
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
        slant: 0.0,
        turn: 0.0,
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
    fn shimmer_tint_goes_through_the_palette() {
        let shimmer = |palette| {
            let settings = Settings {
                palette,
                ..Settings::default()
            };
            let frame = AnimFrame {
                tint: Some((Rgb::new(0x80, 0x40, 0x20), 0.5)),
                ..AnimFrame::STILL
            };
            through_palette(frame, &settings).tint.expect("tint kept")
        };
        assert_eq!(
            shimmer(crate::settings::Palette::Default).0,
            Rgb::new(0x80, 0x40, 0x20)
        );
        let high = shimmer(crate::settings::Palette::HighContrast);
        assert_ne!(high.0, Rgb::new(0x80, 0x40, 0x20));
        assert_eq!(high.1, 0.5);
    }

    #[test]
    fn creature_phases_differ() {
        assert_ne!(creature_phase(ActorId(1)), creature_phase(ActorId(2)));
        assert!((0.0..1.0).contains(&creature_phase(ActorId(7))));
    }

    #[test]
    fn band_borders_sway_a_little_and_stand_still_without_animation() {
        let drifts: Vec<f32> = (0..40)
            .map(|i| band_drift(ActorId(3), i as f32 * 0.5, true))
            .collect();
        assert!(drifts.iter().all(|d| d.abs() <= 0.3 + 1e-6), "{drifts:?}");
        assert!(drifts.iter().any(|&d| d > 0.2) && drifts.iter().any(|&d| d < -0.2));
        assert_eq!(band_drift(ActorId(3), 12.3, false), 0.0);
    }

    #[test]
    fn the_unaware_mark_mixes_a_quarter_toward_slate() {
        let red = Rgb::new(0xc0, 0x30, 0x30);
        assert_eq!(unaware_tint(red, 0.0), red);
        assert_eq!(unaware_tint(red, 1.0), UNAWARE_TINT);
        // A quarter of the way: still red, a shade cooler.
        let marked = unaware_tint(red, 0.25);
        assert_eq!(marked, Rgb::new(0xb3, 0x4a, 0x4e));
        assert!(marked.r > marked.g && marked.r > marked.b);
    }

    #[test]
    fn the_unaware_mark_breathes_and_stands_still_without_animation() {
        let id = ActorId(5);
        let mixes: Vec<f32> = (0..60)
            .map(|i| unaware_mix(id, i as f32 * 0.1, true))
            .collect();
        assert!(
            mixes
                .iter()
                .all(|m| (0.15 - 1e-5..=0.35 + 1e-5).contains(m)),
            "{mixes:?}"
        );
        // Both ends of the breath within one period.
        assert!(mixes.iter().any(|&m| m < 0.16) && mixes.iter().any(|&m| m > 0.34));
        // A breath takes six seconds.
        let at = |t| unaware_mix(id, t, true);
        assert!((at(1.3) - at(7.3)).abs() < 1e-4);
        // Reduced effects: still at a quarter, whatever the time.
        for t in [0.0, 1.5, 3.0, 4.5] {
            assert_eq!(unaware_mix(id, t, false), 0.25);
        }
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
        light.composite(&view, &Default::default(), IVec2::ZERO, size, None);

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
