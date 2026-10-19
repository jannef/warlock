//! Heard sounds on the map (spec §5 "Hearing", §4.8): a faint marker on the free-glyph layer at
//! the rough place of each sound the player heard during the last turn from a source they do not
//! see (`PerceptionView::sounds`). It lasts until the next turn.
//!
//! The marker is information, so it is always drawn: still, faint, and fainter for fainter
//! sounds, never lit (the cell is not seen). With glyph animation on it ripples in when a turn's
//! sounds arrive: a ring grows out of it and fades within a second. With frozen visual time the
//! ripple shows a fixed early moment, so captures are reproducible.

use crate::game::Game;
use crate::render::free_glyph::{FreeGlyph, FreeGlyphs};
use crate::render::grid::MapGrid;
use crate::render::{Effects, VisualTime};
use crate::settings::{Role, Settings};
use bevy::prelude::*;
use warlock_core::color::Rgb;
use warlock_core::noise::{PerceivedSound, Volume};

/// The marker glyph: a dotted ring, "something here, not seen".
pub const MARKER: char = '◌';

/// A pale, cool grey: apart from firelight and terrain, readable on black and on remembered
/// terrain.
const COLOR: Rgb = Rgb::new(0xc8, 0xd2, 0xe4);

/// How long the ripple runs, in seconds.
const RIPPLE_SECONDS: f32 = 0.9;
/// How far the ring grows, in glyph scale beyond 1.
const RIPPLE_GROWTH: f32 = 0.9;
/// With frozen visual time the ripple is shown this long after it started.
const FROZEN_RIPPLE_AGE: f32 = 0.3;

/// When the current turn's sounds arrived.
#[derive(Resource, Debug, Default)]
pub struct SoundMarkers {
    turn: Option<u64>,
    start: f32,
}

/// The marker's opacity for a sound of `volume`.
fn alpha(volume: Volume) -> f32 {
    match volume {
        Volume::Faint => 0.4,
        Volume::Clear => 0.55,
        Volume::Loud => 0.7,
    }
}

/// The ring at `age` seconds into its ripple: its scale and how much of the marker's opacity it
/// keeps; `None` once it has faded.
fn ripple(age: f32) -> Option<(f32, f32)> {
    let t = age / RIPPLE_SECONDS;
    (0.0..1.0)
        .contains(&t)
        .then_some((1.0 + RIPPLE_GROWTH * t, (1.0 - t) * (1.0 - t)))
}

/// Notes when a new turn's sounds arrive.
pub(super) fn track(game: Res<Game>, visual: Res<VisualTime>, mut markers: ResMut<SoundMarkers>) {
    if !game.is_changed() {
        return;
    }
    let turn = game.world.turn();
    if markers.turn.replace(turn) != Some(turn) {
        markers.start = visual.seconds;
    }
}

/// Puts this frame's sound markers on the free-glyph layer (above particles, below creatures).
pub(super) fn emit(
    game: Res<Game>,
    grid: Res<MapGrid>,
    visual: Res<VisualTime>,
    effects: Res<Effects>,
    settings: Res<Settings>,
    markers: Res<SoundMarkers>,
    mut free: ResMut<FreeGlyphs>,
) {
    let view = game.world.perception();
    let age = if visual.frozen {
        FROZEN_RIPPLE_AGE
    } else {
        visual.seconds - markers.start
    };
    let color = settings.palette.apply(COLOR, Role::Fg);
    for sound in view.sounds() {
        let (still, ring) = marker(&grid, sound, color, effects.glyph_animation.then_some(age));
        free.push(still);
        if let Some(ring) = ring {
            free.push(ring);
        }
    }
}

/// The still marker of `sound`, and its ripple ring `age` seconds in (none without animation).
fn marker(
    grid: &MapGrid,
    sound: &PerceivedSound,
    color: Rgb,
    age: Option<f32>,
) -> (FreeGlyph, Option<FreeGlyph>) {
    let at = Vec2::new(sound.pos.x as f32, sound.pos.y as f32);
    let alpha = alpha(sound.volume);
    let base = Color::srgb_u8(color.r, color.g, color.b).to_linear();
    let still = FreeGlyph {
        pos: grid.to_screen(at),
        ch: MARKER,
        color: base.with_alpha(alpha),
        scale: 1.0,
        slant: 0.0,
        turn: 0.0,
    };
    let ring = age.and_then(ripple).map(|(scale, keep)| FreeGlyph {
        color: base.with_alpha(alpha * keep),
        scale,
        ..still
    });
    (still, ring)
}

#[cfg(test)]
mod tests {
    use super::*;
    use warlock_core::Pos;
    use warlock_core::noise::SoundKind;

    fn sound(volume: Volume) -> PerceivedSound {
        PerceivedSound {
            kind: SoundKind::Footsteps,
            pos: Pos::new(4, 3),
            volume,
        }
    }

    #[test]
    fn markers_are_still_and_crisp_with_a_ring_only_while_animating() {
        let grid = MapGrid {
            origin: Vec2::new(10.0, 20.0),
            cell: Vec2::new(11.0, 22.0),
            ..default()
        };
        let (still, ring) = marker(&grid, &sound(Volume::Clear), COLOR, None);
        assert!(still.is_crisp() && still.ch == MARKER);
        assert_eq!(still.pos, grid.to_screen(Vec2::new(4.0, 3.0)));
        assert_eq!(ring, None, "no ripple without glyph animation");
        let (_, ring) = marker(&grid, &sound(Volume::Clear), COLOR, Some(0.3));
        let ring = ring.expect("rippling");
        assert!(ring.scale > 1.0 && ring.color.alpha < still.color.alpha);
        let (_, gone) = marker(&grid, &sound(Volume::Clear), COLOR, Some(RIPPLE_SECONDS));
        assert_eq!(gone, None, "the ripple fades within a second");
    }

    #[test]
    fn fainter_sounds_have_fainter_markers() {
        assert!(alpha(Volume::Faint) < alpha(Volume::Clear));
        assert!(alpha(Volume::Clear) < alpha(Volume::Loud));
        assert!(alpha(Volume::Loud) <= 0.75, "subtle");
    }

    #[test]
    fn the_ripple_grows_and_fades() {
        let (s0, k0) = ripple(0.0).expect("starts");
        let (s1, k1) = ripple(0.5).expect("runs");
        assert_eq!((s0, k0), (1.0, 1.0));
        assert!(s1 > s0 && k1 < k0);
        assert_eq!(ripple(1.0), None);
    }
}
