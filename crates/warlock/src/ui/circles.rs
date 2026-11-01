//! Spell circles from the perception snapshot (spec §4.8 "Spell circles"): which seen actors
//! are circled and with what, and where each circle stands and how far it has turned this frame.
//! `render/circles.rs` draws them.
//!
//! The snapshot only: the player's own statuses (`own_effects`) and a seen creature's that show
//! (`effects_of`, statuses with `seen`; every runed status has it), in status order. No severity
//! and no time left: a circle says what the status bar and look mode say. An unseen actor has no
//! circle. The list is rebuilt when the game changes (a turn), the angles and the sliding
//! positions every frame.

use super::motion::Motion;
use crate::game::Game;
use crate::render::circles::{self, CircleInstance, Circles, Rune, SchoolColors};
use crate::render::grid::MapGrid;
use crate::render::lighting::VisualLight;
use crate::render::{Effects, VisualTime};
use crate::settings::{Role, Settings};
use bevy::prelude::*;
use warlock_core::Pos;
use warlock_core::content::{School, school_index};
use warlock_core::world::{ActorId, ActorKind};

/// The circled actors of the current turn: who, where they stand, and their circle; and
/// whether the list is current (it is not after a frame with circles off).
#[derive(Default)]
pub(super) struct Circled {
    list: Vec<(ActorId, Pos, circles::Circle)>,
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
    mut out: ResMut<Circles>,
    mut colors: ResMut<SchoolColors>,
    mut circled: Local<Circled>,
) {
    out.clip = grid.clip;
    out.cell_height = grid.cell.y;
    let view = game.world.perception();
    let content = view.content();
    if settings.is_changed() || colors.0 == [Vec4::ZERO; 8] {
        let runes = content.runes();
        let mut table = [Vec4::ONE; 8];
        for school in School::ALL {
            let c = settings.palette.apply(runes.color(school), Role::Fg);
            let linear = Color::srgb_u8(c.r, c.g, c.b).to_linear();
            table[school_index(school)] = Vec4::new(linear.red, linear.green, linear.blue, 1.0);
        }
        colors.set_if_neq(SchoolColors(table));
    }
    if !effects.circles {
        circled.current = false;
        return;
    }
    if game.is_changed() || !circled.current {
        circled.current = true;
        let runes = content.runes();
        circled.list.clear();
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
            if !circle.is_empty() {
                circled.list.push((actor.id, actor.pos, circle));
            }
        }
    }
    let time = visual.seconds;
    for &(id, pos, circle) in &circled.list {
        let at = motion.actor_pos(id, pos, time);
        let center = grid.to_screen(at) + grid.cell * 0.5;
        let angle = |ring: Option<&circles::Ring>| {
            ring.map_or(0.0, |r| circles::angle(r, time, effects.motion))
        };
        out.push(CircleInstance {
            center,
            circle,
            angles: [angle(circle.inner.as_ref()), angle(circle.outer.as_ref())],
            keep: light.keep(),
        });
    }
}
