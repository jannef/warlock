//! Spell circles from the perception snapshot (spec §4.8 "Spell circles"): which seen actors
//! are circled and with what, and where each circle stands and how far it has turned this frame.
//! `render/circles.rs` draws them.
//!
//! The snapshot only: the player's own statuses (`own_effects`) and a seen creature's that show
//! (`effects_of`, statuses with `seen`; every runed status has it), in status order. No severity
//! and no time left: a circle says what the status bar and look mode say. An unseen actor has no
//! circle. The list is rebuilt when the game changes (a turn), the angles, the fades and glides
//! (`circle_tracks.rs`), and the sliding positions every frame.

use super::circle_tracks::Tracks;
use super::motion::Motion;
use crate::cli::Args;
use crate::game::Game;
use crate::render::atlas::{self, Font, GlyphAtlas};
use crate::render::circles::{self, CircleInstance, Circles, Rune, SchoolColors};
use crate::render::grid::MapGrid;
use crate::render::lighting::VisualLight;
use crate::render::post::emissive;
use crate::render::{Effects, VisualTime};
use crate::settings::{Role, Settings};
use bevy::prelude::*;
use warlock_core::Pos;
use warlock_core::content::{School, school_index};
use warlock_core::world::{ActorId, ActorKind};

/// How emissive a circle's ink is with glow on (`post::emissive`: × 1.1), the design's §8.4.
const CIRCLE_GLOW: f32 = 0.1;

/// The circled actors of the current turn (who and where they stand), their circles' fades and
/// glides, and whether the list is current (it is not after a frame with circles off).
#[derive(Default)]
pub(super) struct Circled {
    list: Vec<(ActorId, Pos)>,
    tracks: Tracks,
    current: bool,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn emit(
    game: Res<Game>,
    grid: Res<MapGrid>,
    motion: Res<Motion>,
    visual: Res<VisualTime>,
    effects: Res<Effects>,
    settings: Res<Settings>,
    light: Res<VisualLight>,
    (font, atlas, args): (Res<Font>, Res<GlyphAtlas>, Res<Args>),
    mut out: ResMut<Circles>,
    mut colors: ResMut<SchoolColors>,
    mut circled: Local<Circled>,
) {
    out.clip = grid.clip;
    out.cell_height = grid.cell.y;
    out.grid_origin = grid.origin;
    out.cell = grid.cell;
    out.time = visual.seconds;
    // The runes pulse with motion on (at the frozen moment under `--freeze-time`), and glow but
    // with reduced effects (which also stills them, as `motion` off does).
    out.pulse = effects.motion;
    out.halo = !(settings.reduced_effects || args.reduced_effects);
    let view = game.world.perception();
    let content = view.content();
    if settings.is_changed() || effects.is_changed() || colors.0 == [Vec4::ZERO; 8] {
        let runes = content.runes();
        let mut table = [Vec4::ONE; 8];
        // The circle sheds its school's light (core: the status's `light`); with glow on its
        // ink is a little emissive too.
        let glow = if effects.glow { CIRCLE_GLOW } else { 0.0 };
        for school in School::ALL {
            let c = settings.palette.apply(runes.color(school), Role::Fg);
            let linear = emissive(Color::srgb_u8(c.r, c.g, c.b).to_linear(), glow);
            table[school_index(school)] = Vec4::new(linear.red, linear.green, linear.blue, 1.0);
        }
        colors.set_if_neq(SchoolColors(table));
    }
    if !effects.circles {
        circled.current = false;
        circled.tracks = Tracks::default();
        return;
    }
    let time = visual.seconds;
    let animate = effects.motion && !visual.frozen;
    if game.is_changed() || !circled.current {
        circled.current = true;
        let runes = content.runes();
        let Circled { list, tracks, .. } = &mut *circled;
        list.clear();
        let mut seen = Vec::new();
        for actor in view.actors() {
            let statuses: Vec<_> = match actor.kind {
                ActorKind::Player => view.own_effects().collect(),
                _ => view.effects_of(actor.id).collect(),
            };
            let circle = circles::layout(statuses.into_iter().filter_map(|s| {
                let (index, def) = (runes.index(s)?, runes.of(s)?);
                let rune = Rune {
                    slot: index as u8,
                    school: school_index(def.school) as u8,
                };
                Some((rune, def.hex))
            }));
            // An actor whose circle has just gone keeps its place in the list while it fades.
            if !circle.is_empty() || tracks.has(actor.id) {
                list.push((actor.id, actor.pos));
                seen.push((actor.id, circle));
            }
        }
        tracks.update(time, &seen);
    }
    circled.tracks.settle(time, animate);
    for &(id, pos) in &circled.list {
        let kinds = circled.tracks.shown(id, time, animate, effects.motion);
        let at = motion.actor_pos(id, pos, time);
        // Centred on the actor's glyph's ink, not its cell: a glyph sits on the baseline, its
        // ink above the cell's middle or below it (`@` reaches under the baseline).
        let glyph = view.actor_at(pos).map_or('@', |a| a.glyph(content));
        let ink = ink(&font.0, atlas.px, glyph, grid.cell);
        out.push(CircleInstance {
            center: grid.to_screen(at) + ink.center(),
            kinds,
            keep: light.keep(),
            cell: grid.to_console(pos),
            ink: ink.half_size(),
        });
    }
}

/// Where the ink of `glyph` is in its cell, in pixels from the cell's top-left: its outline's
/// box as the atlas rasterises it at `px` (`atlas::ink_box`), the whole cell for a glyph with
/// no ink.
fn ink(font: &fontdue::Font, px: f32, glyph: char, cell: Vec2) -> Rect {
    atlas::ink_box(font, px, glyph).unwrap_or(Rect::from_corners(Vec2::ZERO, cell))
}
