//! Spell effects in the rules (spec §4.8 "Spell effects"): the shapes they take, what the player
//! perceives of one ([`Event::Spell`]), and the dev action that shows one
//! (`effect:ID@X,Y[>X,Y]`).
//!
//! How an effect looks (`data/effects.toml`) is presentation: no rule reads it. The dev action
//! reads only an effect's `shape` and `radius`, to make an event to show, and has no other
//! consequence.
//!
//! **Perceived**: an effect is light itself, so the player sees its cells wherever they are in
//! line of sight, in the dark too (like a burning missile). The event's cells are cut to those,
//! a bolt's path to the stretch from the first of them to the last, and an origin the player
//! does not see is replaced by where the effect comes into view (a burst's center, the first cell
//! of a bolt's path seen); an event with no cell seen is dropped.

use super::{Spent, World};
use crate::action::{Event, SpellShape};
use crate::content::EffectShape;
use crate::fov::radius2;
use crate::geom::Pos;
use crate::sight::LineOfSight;

/// The cells a burst around `center` of `radius` reaches: within the radius (rounded, like every
/// radius) and in line of sight from the center, opaque cells left out. None if the center is
/// opaque.
pub(crate) fn burst_cells(center: Pos, radius: i32, opaque: impl Fn(Pos) -> bool) -> Vec<Pos> {
    if opaque(center) {
        return Vec::new();
    }
    let radius = radius.max(0);
    let los = LineOfSight::compute(center, radius, &opaque);
    let r2 = radius2(radius);
    let mut cells = Vec::new();
    for y in center.y - radius..=center.y + radius {
        for x in center.x - radius..=center.x + radius {
            let p = Pos::new(x, y);
            if p.dist2(center) <= r2 && los.contains(p) && !opaque(p) {
                cells.push(p);
            }
        }
    }
    cells
}

/// The cells a bolt from `from` toward `to` runs over: the straight line after `from` up to and
/// including `to`, cut short before the first opaque cell.
pub(crate) fn bolt_path(from: Pos, to: Pos, opaque: impl Fn(Pos) -> bool) -> Vec<Pos> {
    if from == to {
        return Vec::new();
    }
    (from.line_between(to).chain(std::iter::once(to)))
        .take_while(|&p| !opaque(p))
        .collect()
}

/// Cuts a spell event to what the player perceives of it (see the module docs): `seen` tells
/// whether the player sees a cell or has it in line of sight. Returns whether anything of it is
/// left to show.
pub(crate) fn perceive_spell(event: &mut Event, seen: impl Fn(Pos) -> bool) -> bool {
    let Event::Spell {
        origin,
        shape,
        cells,
        ..
    } = event
    else {
        return true;
    };
    cells.retain(|&p| seen(p));
    match shape {
        SpellShape::Burst { center, .. } => {
            if !seen(*origin) {
                *origin = *center;
            }
        }
        SpellShape::Bolt { path } => {
            match (
                path.iter().position(|&p| seen(p)),
                path.iter().rposition(|&p| seen(p)),
            ) {
                (Some(first), Some(last)) => {
                    path.truncate(last + 1);
                    path.drain(..first);
                    if first > 0 || !seen(*origin) {
                        *origin = path[0];
                    }
                }
                _ => path.clear(),
            }
        }
    }
    !cells.is_empty()
}

impl World {
    /// The dev action `effect:ID@X,Y[>X,Y]` ([`crate::Action::Effect`]): shows effect `id` at
    /// cell `at` (a burst around it), or from `at` toward `to` (a burst around `to`, flying there
    /// from `at`; a bolt from `at` to `to`). It hurts no one and takes no time; only the message
    /// log changes.
    pub(super) fn show_effect(
        &mut self,
        id: &str,
        at: Pos,
        to: Option<Pos>,
        events: &mut Vec<Event>,
    ) -> Spent {
        let Some(def) = self.content.effect(id) else {
            self.say(format!("There is no effect `{id}`."));
            return Spent::Nothing;
        };
        let level = self.state.current();
        let content = &self.content;
        let opaque = |p: Pos| level.is_opaque(content, p);
        let (shape, cells) = match def.shape {
            EffectShape::Burst => {
                let center = to.unwrap_or(at);
                let radius = def.radius;
                let cells = burst_cells(center, radius, opaque);
                (SpellShape::Burst { center, radius }, cells)
            }
            EffectShape::Bolt => {
                let path = to.map_or_else(Vec::new, |to| bolt_path(at, to, opaque));
                if path.is_empty() {
                    self.say("A bolt needs a way to go: `effect:ID@X,Y>X,Y`, not into a wall.");
                    return Spent::Nothing;
                }
                (SpellShape::Bolt { path: path.clone() }, path)
            }
        };
        if cells.is_empty() {
            self.say(format!("Effect `{id}` reaches no cell there."));
            return Spent::Nothing;
        }
        self.say(format!("Effect: {id} (dev)."));
        events.push(Event::Spell {
            effect: id.to_string(),
            origin: at,
            shape,
            cells,
        });
        Spent::NoTime
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Action, Outcome};
    use crate::content::{Content, ContentFiles};
    use std::sync::Arc;

    /// A lit room (x 1 to 9, y 1 to 5) with a pillar at (6, 3), the player at (2, 3), and a dark
    /// closet behind a wall to the east.
    fn room() -> World {
        let mut files = ContentFiles::builtin();
        files.insert(
            "levels/spells.toml",
            "name = \"Spells\"\nambient = [255, 255, 255, 0]\nmap = '''\n\
             ##############\n#.........#..#\n#.........#..#\n#.@...#...#..#\n\
             #.........#..#\n#.........#..#\n##############\n'''\n\
             [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
        );
        files.insert(
            "scenarios/spells.toml",
            "name = \"Spells\"\nstart_level = \"spells\"\n",
        );
        let content = Content::load(&files).expect("test content loads");
        World::new(Arc::new(content), "spells", None).expect("scenario starts")
    }

    fn spell(outcome: &Outcome) -> (&str, Pos, &SpellShape, &[Pos]) {
        match outcome.events.as_slice() {
            [
                Event::Spell {
                    effect,
                    origin,
                    shape,
                    cells,
                },
            ] => (effect.as_str(), *origin, shape, cells.as_slice()),
            other => panic!("one spell event, not {other:?}"),
        }
    }

    #[test]
    fn bursts_reach_round_and_stop_at_walls() {
        let open = |_: Pos| false;
        let cells = burst_cells(Pos::new(0, 0), 2, open);
        // r² + r = 6: a rounded ball of 21 cells.
        assert_eq!(cells.len(), 21);
        assert!(cells.contains(&Pos::new(2, 1)) && !cells.contains(&Pos::new(2, 2)));
        assert_eq!(burst_cells(Pos::new(0, 0), 0, open), [Pos::new(0, 0)]);
        // A wall at x = 1: nothing beyond it, the wall itself left out.
        let wall = |p: Pos| p.x == 1;
        let cells = burst_cells(Pos::new(0, 0), 2, wall);
        assert!(cells.iter().all(|p| p.x <= 0), "{cells:?}");
        assert!(burst_cells(Pos::new(1, 0), 2, wall).is_empty());
    }

    #[test]
    fn bolts_run_to_their_end_or_a_wall() {
        let open = |_: Pos| false;
        let path = bolt_path(Pos::new(0, 0), Pos::new(4, 2), open);
        assert_eq!(path.len(), 4);
        assert_eq!(path.last(), Some(&Pos::new(4, 2)));
        assert!(!path.contains(&Pos::new(0, 0)), "it leaves its origin");
        let wall = |p: Pos| p.x == 3;
        assert_eq!(
            bolt_path(Pos::new(0, 0), Pos::new(6, 0), wall),
            [Pos::new(1, 0), Pos::new(2, 0)]
        );
        assert!(bolt_path(Pos::new(2, 2), Pos::new(2, 2), open).is_empty());
    }

    #[test]
    fn the_dev_action_shows_an_effect_and_changes_nothing_else() {
        let mut world = room();
        let (turn, time) = (world.turn(), world.time());
        let action: Action = "effect:fireball@4,2".parse().expect("parses");
        let out = world.apply(&action);
        let (effect, origin, shape, cells) = spell(&out);
        assert_eq!((effect, origin), ("fireball", Pos::new(4, 2)));
        let radius = world.content().effect("fireball").expect("fireball").radius;
        assert_eq!(
            shape,
            &SpellShape::Burst {
                center: Pos::new(4, 2),
                radius
            }
        );
        assert!(cells.contains(&Pos::new(4, 2)) && cells.len() > 5);
        assert!(!out.took_turn && out.elapsed.is_zero());
        assert_eq!((world.turn(), world.time()), (turn, time));

        // A bolt from the player east: it stops at the pillar.
        let action: Action = "effect:lightning_bolt@2,3>9,3".parse().expect("parses");
        let out = world.apply(&action);
        let (_, origin, shape, cells) = spell(&out);
        assert_eq!(origin, Pos::new(2, 3));
        let path: Vec<Pos> = (3..=5).map(|x| Pos::new(x, 3)).collect();
        assert_eq!(shape, &SpellShape::Bolt { path: path.clone() });
        assert_eq!(cells, path);

        // Unknown effects and bolts without a way are refused.
        for bad in [
            "effect:sparkle@4,2",
            "effect:lightning_bolt@4,2",
            "effect:lightning_bolt@2,3>2,3",
            "effect:fireball@0,0",
        ] {
            let out = world.apply(&bad.parse().expect("parses"));
            assert!(out.events.is_empty(), "{bad}");
        }
    }

    #[test]
    fn the_player_perceives_only_the_cells_in_line_of_sight() {
        let mut world = room();
        // A fireball on the far side of the east wall, in the closet: nothing to see.
        let out = world.apply(&"effect:fireball@12,3".parse().expect("parses"));
        assert!(out.events.is_empty());
        // Behind the pillar from the player: the cells it hides are cut.
        let out = world.apply(&"effect:fireball@8,3".parse().expect("parses"));
        let (_, _, _, cells) = spell(&out);
        let view = world.perception();
        assert!(cells.iter().all(|&p| view.is_visible(p)), "{cells:?}");
        assert!(!cells.contains(&Pos::new(7, 3)), "hidden by the pillar");
        assert!(cells.contains(&Pos::new(8, 1)));
        // A bolt from the closet's side of the wall is cut to what is seen; it comes into view
        // where it is first seen.
        let mut event = Event::Spell {
            effect: "lightning_bolt".into(),
            origin: Pos::new(0, 0),
            shape: SpellShape::Bolt {
                path: (1..=6).map(|x| Pos::new(x, 0)).collect(),
            },
            cells: (1..=6).map(|x| Pos::new(x, 0)).collect(),
        };
        let seen = |p: Pos| (3..=4).contains(&p.x);
        assert!(perceive_spell(&mut event, seen));
        let Event::Spell {
            origin,
            shape,
            cells,
            ..
        } = &event
        else {
            unreachable!()
        };
        let stretch = vec![Pos::new(3, 0), Pos::new(4, 0)];
        assert_eq!(
            shape,
            &SpellShape::Bolt {
                path: stretch.clone()
            }
        );
        assert_eq!((*origin, cells), (Pos::new(3, 0), &stretch));
    }

    #[test]
    fn the_rules_never_read_how_effects_look() {
        // The same run with every effect looking different plays out the same: same state, same
        // events but for the effect's cells (the dev action's radius).
        let mut files = ContentFiles::builtin();
        let looks = include_str!("../../../../assets/data/effects.toml");
        let changed = looks
            .replace("speed = 24", "speed = 3")
            .replace("glyphs = \"▓▒░\"", "glyphs = \"*\"")
            .replace("darken = { amount = 0.7", "darken = { amount = 0.1")
            .replace("pixels = 4", "pixels = 0")
            .replace("#fffff0", "#000000");
        assert_ne!(looks, changed);
        files.insert("data/effects.toml", &changed);
        let other = Content::load(&files).expect("changed effects load");
        let builtin = Content::builtin().expect("builtin content");
        assert_eq!(
            other.fingerprint(),
            builtin.fingerprint(),
            "saves stay valid"
        );
        let actions = crate::action::parse_actions(
            "effect:fireball@20,18 wait move:N effect:lightning_bolt@19,17>40,17 wait*3",
        )
        .expect("actions parse");
        let mut a = World::new(Arc::new(builtin), "arena", None).expect("arena");
        let mut b = World::new(Arc::new(other), "arena", None).expect("arena");
        for action in &actions {
            assert_eq!(a.apply(action), b.apply(action), "{action}");
            assert_eq!(a.state_hash(), b.state_hash(), "{action}");
        }
    }
}
