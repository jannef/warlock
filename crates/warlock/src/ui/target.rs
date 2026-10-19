//! Targeting (spec §6 "Targeting"): aiming a shot (`f`) or a throw (`t`) with a cursor on the
//! map, ADOM-like. It is an input mode like look mode, not a screen: no game time passes until
//! the missile is loosed.
//!
//! The cursor starts on the last target if the player still sees it (a creature) or it lies on
//! this level (a cell), else on the nearest creature in view that is hostile, else on the player.
//! Tab and Shift-Tab cycle through the creatures in view, nearest first; the direction keys move
//! the cursor freely. Enter (or the key that started it) looses the missile and remembers the
//! target; Esc cancels.
//!
//! The line of fire ([`Line`]) is the integer line the rules fly a missile along
//! ([`Pos::ray`]), drawn as faint dots from the player toward the cursor, as far as the player
//! knows it: it ends before the first cell they know to be solid, and at the edge of the range.
//! A target beyond something solid or behind a creature in the way is blocked, one beyond the
//! range out of range; the message line says so (never color alone). No chance to hit is shown.
//! Everything here reads the perception snapshot only.

use super::look;
use crate::game::Game;
use crate::input::InputMode;
use crate::render::console::Console;
use crate::render::free_glyph::{FreeGlyph, FreeGlyphs};
use crate::render::grid::MapGrid;
use crate::settings::{Role, Settings};
use bevy::prelude::*;
use warlock_core::color::Rgb;
use warlock_core::content::Stance;
use warlock_core::fov::radius2;
use warlock_core::perception::{CellView, PerceptionView};
use warlock_core::world::ActorId;
use warlock_core::{Action, Dir, Pos};

/// The path's dots: faint, warm, and never lit, so the line reads in the dark too.
pub const PATH_GLYPH: char = '·';
pub const PATH_COLOR: Rgb = Rgb::new(0xe0, 0xb0, 0x50);
pub const PATH_ALPHA: f32 = 0.55;
/// The cursor on a target the missile would not reach (blocked, or out of range): muted red.
const BLOCKED_BG: Rgb = Rgb::new(0xc8, 0x70, 0x60);

/// What the player aims.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AimKind {
    /// A shot with the readied missile weapon (`f`).
    Fire,
    /// A throw of inventory stack `usize` (`t`).
    Throw(usize),
}

/// Aiming: what, and where the cursor is (a level position).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Aim {
    pub kind: AimKind,
    pub at: Pos,
}

/// The last target, remembered for a quick shot again (`f`, then Enter): a creature, or a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Creature(ActorId),
    Cell(Pos),
}

/// The last target the player loosed a missile at, on which level.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct LastTarget(pub Option<(String, Target)>);

impl LastTarget {
    /// Remembers what lies under `aim`'s cursor: the creature the player sees there, else the
    /// cell.
    pub fn remember(&mut self, view: &PerceptionView<'_>, aim: Aim) {
        let target = match view.actor_at(aim.at) {
            Some(actor) if aim.at != view.viewer() => Target::Creature(actor.id),
            _ => Target::Cell(aim.at),
        };
        self.0 = Some((view.level_id().to_string(), target));
    }

    /// Where the last target is now, if the player still sees it (a creature) or it lies on
    /// this level (a cell).
    fn now(&self, view: &PerceptionView<'_>) -> Option<Pos> {
        let (level, target) = self.0.as_ref()?;
        if level != view.level_id() {
            return None;
        }
        match *target {
            Target::Creature(id) => view.actors().iter().find(|a| a.id == id).map(|a| a.pos),
            Target::Cell(p) => Some(p),
        }
    }
}

/// The creatures in view but the player, nearest first (ties: the lower id): what Tab cycles.
fn targets(view: &PerceptionView<'_>) -> Vec<(Pos, Stance)> {
    let me = view.viewer();
    let mut seen: Vec<_> = (view.actors().iter())
        .filter(|a| a.pos != me)
        .map(|a| (a.pos.dist2(me), a.id, a.pos, view.stance(a)))
        .collect();
    seen.sort_by_key(|&(d2, id, ..)| (d2, id));
    seen.into_iter().map(|(_, _, p, s)| (p, s)).collect()
}

impl Aim {
    /// Starts aiming `kind`: on the last target if it is still there, else on the nearest hostile
    /// creature in view, else on the player.
    pub fn start(kind: AimKind, view: &PerceptionView<'_>, last: &LastTarget) -> Self {
        let hostile = targets(view)
            .into_iter()
            .find(|(_, stance)| *stance == Stance::Hostile)
            .map(|(p, _)| p);
        let at = last.now(view).or(hostile).unwrap_or(view.viewer());
        Self { kind, at }
    }

    /// The cursor moved one cell in `dir`, kept on the level.
    pub fn step(self, dir: Dir, view: &PerceptionView<'_>) -> Self {
        let to = self.at.step(dir);
        let at = Pos::new(
            to.x.clamp(0, view.width() - 1),
            to.y.clamp(0, view.height() - 1),
        );
        Self { at, ..self }
    }

    /// The cursor on the next creature in view (`forward`), or the one before: from the one it
    /// is on, else the nearest (or the farthest going back).
    pub fn cycle(self, view: &PerceptionView<'_>, forward: bool) -> Self {
        let targets = targets(view);
        if targets.is_empty() {
            return self;
        }
        let n = targets.len();
        let i = match targets.iter().position(|(p, _)| *p == self.at) {
            Some(i) if forward => (i + 1) % n,
            Some(i) => (i + n - 1) % n,
            None if forward => 0,
            None => n - 1,
        };
        Self {
            at: targets[i].0,
            ..self
        }
    }

    /// How far what is aimed flies, if the player has it ready.
    pub fn range(&self, view: &PerceptionView<'_>) -> Option<i32> {
        match self.kind {
            AimKind::Fire => view.ready_to_fire().map(|(_, _, range)| range),
            AimKind::Throw(item) => view.throw_range(item),
        }
    }

    /// The action that looses it at the cursor.
    pub fn action(&self) -> Action {
        match self.kind {
            AimKind::Fire => Action::Fire(self.at),
            AimKind::Throw(item) => Action::Throw { item, at: self.at },
        }
    }

    /// The prompt in the message line: what is aimed, and how. `ready` names the missile ("an
    /// arrow"); the keys are the bindings' (`f`, `t`, Tab).
    pub fn prompt(&self, ready: &str, (confirm, next): (&str, &str)) -> String {
        let (verb, does) = match self.kind {
            AimKind::Fire => ("Fire", "fires"),
            AimKind::Throw(_) => ("Throw", "throws"),
        };
        format!(
            "{verb} {ready}: {next} or direction keys aim, Enter or {confirm} {does}, Esc cancels."
        )
    }
}

impl Line {
    /// Why a missile loosed along it would not reach the cursor, said after the target's
    /// description: "Out of range.", "The way is blocked."; nothing when it would.
    pub fn status(&self) -> Option<&'static str> {
        if self.out_of_range {
            Some("Out of range.")
        } else if self.blocked {
            Some("The way is blocked.")
        } else {
            None
        }
    }
}

/// The line of fire from the player to the cursor, as the player knows it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Line {
    /// The cells the path is drawn over, from the player's neighbor on: up to the cell before
    /// the cursor, or before the first cell known to be solid, or to the edge of the range.
    pub cells: Vec<Pos>,
    /// Something solid the player knows of, or a creature they see, stands between them and the
    /// cursor.
    pub blocked: bool,
    /// The cursor lies beyond the range.
    pub out_of_range: bool,
}

/// The line of fire from `from` to `to` for a missile that flies `range` cells (`None`: nothing
/// ready, so no range to mark), over what the player perceives: the cells they see or remember
/// are solid or not as they know them; cells never seen let it pass.
pub fn line(view: &PerceptionView<'_>, from: Pos, to: Pos, range: Option<i32>) -> Line {
    let content = view.content();
    let solid = |p: Pos| match view.cell(p) {
        CellView::Visible(t) | CellView::Remembered(t) => !content.terrain(t).transparent,
        CellView::Unknown => false,
    };
    let reach = range.map_or(i32::MAX, radius2);
    let mut out = Line {
        out_of_range: to.dist2(from) > reach,
        ..Line::default()
    };
    for p in from.ray(to).take_while(|&p| p != to) {
        if solid(p) {
            out.blocked = true;
            break;
        }
        if view.actor_at(p).is_some() {
            out.blocked = true;
        }
        if p.dist2(from) > reach {
            break;
        }
        out.cells.push(p);
    }
    if from != to && solid(to) {
        out.blocked = true;
    }
    out
}

/// The cells the line of fire is drawn over while aiming (`mode`), but those where a creature
/// stands: none otherwise.
pub fn path_cells(view: &PerceptionView<'_>, mode: InputMode) -> Vec<Pos> {
    let InputMode::Aim(aim) = mode else {
        return Vec::new();
    };
    let line = line(view, view.viewer(), aim.at, aim.range(view));
    (line.cells.into_iter())
        .filter(|&p| view.actor_at(p).is_none())
        .collect()
}

/// Draws the line of fire's dots on the free-glyph layer while aiming: faint, unlit (so they
/// read in the dark), in the palette's colors.
pub(super) fn emit_path(
    game: Res<Game>,
    mode: Res<InputMode>,
    grid: Res<MapGrid>,
    settings: Res<Settings>,
    mut free: ResMut<FreeGlyphs>,
) {
    let view = game.world.perception();
    let mut color = Color::from(Srgba::from_u8_array_no_alpha({
        let c = settings.palette.apply(PATH_COLOR, Role::Fg);
        [c.r, c.g, c.b]
    }))
    .to_linear();
    color.alpha = PATH_ALPHA;
    for p in path_cells(&view, *mode) {
        free.push(FreeGlyph {
            pos: grid.to_screen(Vec2::new(p.x as f32, p.y as f32)),
            ch: PATH_GLYPH,
            color,
            scale: 1.0,
            slant: 0.0,
            turn: 0.0,
        });
    }
}

/// Draws the cursor at console cell `at` over level cell `p`: the look cursor, or on a target the
/// missile would not reach, the same on a muted red.
pub fn draw_cursor(
    console: &mut Console,
    at: (i32, i32),
    view: &PerceptionView<'_>,
    p: Pos,
    line: &Line,
) {
    let bg = if line.blocked || line.out_of_range {
        BLOCKED_BG
    } else {
        look::CURSOR_BG
    };
    look::draw_cursor_on(console, at, view, p, bg);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use warlock_core::World;
    use warlock_core::content::Content;

    fn arena() -> World {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut w = World::new(content, "arena", None).expect("arena");
        // The war band comes into the torchlight.
        for _ in 0..8 {
            w.apply(&Action::Wait);
        }
        w
    }

    #[test]
    fn the_cursor_starts_on_the_nearest_hostile_then_the_last_target() {
        let w = arena();
        let view = w.perception();
        let nearest = (targets(&view).into_iter())
            .find(|(_, s)| *s == Stance::Hostile)
            .expect("a hostile creature in view")
            .0;
        let aim = Aim::start(AimKind::Fire, &view, &LastTarget::default());
        assert_eq!(aim.at, nearest);
        // A remembered creature still in view wins; one on another level does not.
        let other = targets(&view).last().expect("creatures").0;
        let mut last = LastTarget::default();
        last.remember(&view, Aim { at: other, ..aim });
        assert!(matches!(last.0, Some((_, Target::Creature(_)))));
        assert_eq!(Aim::start(AimKind::Fire, &view, &last).at, other);
        // A cell is remembered as a cell.
        let spot = Pos::new(2, 2);
        last.remember(&view, Aim { at: spot, ..aim });
        assert_eq!(last.0, Some(("arena".to_string(), Target::Cell(spot))));
        assert_eq!(Aim::start(AimKind::Fire, &view, &last).at, spot);
        last.0 = Some(("hub".into(), Target::Cell(spot)));
        assert_eq!(Aim::start(AimKind::Fire, &view, &last).at, nearest);
        // With no one in view, the cursor starts on the player.
        let content = Arc::new(Content::builtin().expect("content"));
        let hub = World::new(content, "showcase", None).expect("hub");
        let view = hub.perception();
        let aim = Aim::start(AimKind::Throw(2), &view, &LastTarget::default());
        assert_eq!(
            aim,
            Aim {
                kind: AimKind::Throw(2),
                at: view.viewer()
            }
        );
    }

    #[test]
    fn tab_cycles_through_the_creatures_in_view_nearest_first() {
        let w = arena();
        let view = w.perception();
        let all: Vec<Pos> = targets(&view).into_iter().map(|(p, _)| p).collect();
        assert!(all.len() >= 3, "{all:?}");
        let me = view.viewer();
        assert!(all.windows(2).all(|w| w[0].dist2(me) <= w[1].dist2(me)));
        let aim = Aim {
            kind: AimKind::Fire,
            at: me,
        };
        let mut seen = Vec::new();
        let mut a = aim;
        for _ in 0..all.len() {
            a = a.cycle(&view, true);
            seen.push(a.at);
        }
        assert_eq!(seen, all, "forward from the nearest");
        assert_eq!(a.cycle(&view, true).at, all[0], "and round again");
        assert_eq!(aim.cycle(&view, false).at, *all.last().expect("some"));
        let second = Aim { at: all[1], ..aim };
        assert_eq!(second.cycle(&view, false).at, all[0]);
        // Moving keeps the cursor on the level.
        let corner = Aim {
            at: Pos::new(0, 0),
            ..aim
        };
        assert_eq!(corner.step(Dir::NW, &view).at, Pos::new(0, 0));
        assert_eq!(corner.step(Dir::SE, &view).at, Pos::new(1, 1));
    }

    #[test]
    fn the_line_of_fire_ends_at_what_is_solid_and_marks_range() {
        let content = Arc::new(Content::builtin().expect("content"));
        let w = World::new(content, "showcase", None).expect("hub");
        let view = w.perception();
        let me = view.viewer();
        // Across the lit start room to the east.
        let east = Pos::new(me.x + 5, me.y);
        let open = line(&view, me, east, Some(10));
        let cells: Vec<Pos> = (1..5).map(|dx| Pos::new(me.x + dx, me.y)).collect();
        assert_eq!(
            open,
            Line {
                cells,
                blocked: false,
                out_of_range: false
            }
        );
        assert_eq!(line(&view, me, east, Some(4)).cells.len(), 4);
        assert!(line(&view, me, east, Some(4)).out_of_range);
        // Into the west wall and beyond: the path stops before the wall.
        let beyond = Pos::new(me.x - 9, me.y);
        let walled = line(&view, me, beyond, Some(12));
        assert!(walled.blocked && !walled.out_of_range);
        assert_eq!(walled.cells.len(), (me.x - 1) as usize);
        assert!(walled.cells.iter().all(|p| p.x >= 1));
        // Aimed at the wall itself: blocked too.
        assert!(line(&view, me, Pos::new(0, me.y), Some(12)).blocked);
        // No range given (nothing ready): never out of range.
        assert!(!line(&view, me, Pos::new(me.x + 6, me.y), None).out_of_range);
    }

    #[test]
    fn prompts_say_what_is_aimed_and_why_it_would_not_reach() {
        let aim = Aim {
            kind: AimKind::Fire,
            at: Pos::new(1, 1),
        };
        let text = aim.prompt("a flaming arrow", ("f", "Tab"));
        assert_eq!(
            text,
            "Fire a flaming arrow: Tab or direction keys aim, Enter or f fires, Esc cancels."
        );
        assert_eq!(Line::default().status(), None);
        let far = Line {
            out_of_range: true,
            blocked: true,
            ..Line::default()
        };
        assert_eq!(far.status(), Some("Out of range."));
        let walled = Line {
            blocked: true,
            ..Line::default()
        };
        assert_eq!(walled.status(), Some("The way is blocked."));
        let throw = Aim {
            kind: AimKind::Throw(0),
            ..aim
        };
        let text = throw.prompt("a dagger", ("t", "Tab"));
        assert!(text.starts_with("Throw a dagger:"), "{text}");
        assert!(text.contains("Enter or t throws"), "{text}");
        assert_eq!(
            throw.action(),
            Action::Throw {
                item: 0,
                at: Pos::new(1, 1)
            }
        );
        assert_eq!(aim.action(), Action::Fire(Pos::new(1, 1)));
    }
}
