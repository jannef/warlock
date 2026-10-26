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
//!
//! **Casting** ([`World::cast_burst`], [`World::cast_bolt`]): the one place a burst or a bolt
//! does what it does, for traps (spec §8 "Traps") and scrolls (spec §8 "Scrolls") alike: the
//! effect's event, one roll of its damage for everyone it reaches (in ascending actor id order),
//! then a burst's blast against walls and its heat or frost. Only who cast it differs
//! ([`Caster`]): a trap's blows are its own and its kills no one's; an actor's (the player
//! reading a scroll) are theirs, and so are its kills, which earn them experience.

use super::fight::Harm;
use super::{Actor, ActorId, Spent, World};
use crate::action::{AttackKind, Event, SpellShape};
use crate::combat::{Blow, Strike};
use crate::content::{Bolt, Burst, BurstIce, EffectShape, TrapId, TrapNoise};
use crate::damage::Damage;
use crate::fov::radius2;
use crate::geom::Pos;
use crate::ice::Shock;
use crate::noise::{self, Noise};
use crate::sight::LineOfSight;
use crate::talent::TalentMods;
use std::collections::BTreeSet;

/// Who cast a spell, and so who deals its harm.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Caster {
    /// A trap that went off: its blows are [`Event::TrapStruck`], and its kills earn no one
    /// anything.
    Trap(TrapId),
    /// An actor (the player reading a scroll): its blows are theirs ([`Event::Attacked`], a
    /// spell, with no roll to hit), and so are its kills.
    Actor(ActorId),
}

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

    /// `noise` (a trap's going off, a scroll's spell) on cell `at` of the current level: no actor
    /// made it, so everyone in earshot hears it. A special room scales it like every noise made
    /// inside it.
    pub(super) fn sound_at(&mut self, noise: Option<TrapNoise>, at: Pos) {
        let Some(sound) = noise else {
            return;
        };
        let room = self.state.current().room_effect_at(&self.content, at);
        let percent = room.map_or(100, |r| u32::from(r.noise));
        let loudness = u32::from(sound.loudness) * percent / 100;
        let noise = Noise {
            kind: sound.sound,
            at,
            loudness: loudness.min(u32::from(noise::MAX_LOUDNESS)) as u16,
            maker: None,
        };
        if noise.loudness > 0 {
            noise::make(&mut self.state, &self.content, noise);
        }
    }

    /// `burst` goes off around `center`, cast by `caster` from `origin` (a trap's own cell; the
    /// reader of a scroll, from whom it flew): its effect ([`Event::Spell`] over the cells it
    /// reaches, [`burst_cells`], its own cell never stopping it (a trap's on a shut door), so
    /// what is drawn is what is hit), one roll of its damage for
    /// everyone on them (but a caster its own burst spares, Careful reader), half of it (at
    /// least 1) beyond radius − 1; then the walls its blast
    /// breaks ([`World::blast`]), then the ice it melts ([`World::shock`]) or the water it
    /// freezes ([`World::frost`]).
    pub(super) fn cast_burst(
        &mut self,
        caster: Caster,
        (origin, center): (Pos, Pos),
        burst: &Burst,
        events: &mut Vec<Event>,
    ) {
        let level = self.state.current();
        let radius = burst.radius;
        // Its own cell never stops it: a trap's on a shut door bursts out on both sides.
        let opaque = |p: Pos| p != center && level.is_opaque(&self.content, p);
        let cells = burst_cells(center, radius, opaque);
        events.push(Event::Spell {
            effect: burst.spell.clone(),
            origin,
            shape: SpellShape::Burst { center, radius },
            cells: cells.clone(),
        });
        let roll = burst.damage.roll(&mut self.state.rng);
        let inner = radius2(radius - 1);
        let level = self.state.current_level.clone();
        let reached: BTreeSet<Pos> = cells.into_iter().collect();
        // Careful reader (spec §8 "Talents"): a caster's own burst may spare it.
        let spared = |a: &Actor| {
            matches!(caster, Caster::Actor(id) if id == a.id)
                && a.talent_mods(&self.content).spares_self
        };
        let caught: Vec<(ActorId, Pos)> = (self.state.actors.values())
            .filter(|a| a.level == level && reached.contains(&a.pos) && !spared(a))
            .map(|a| (a.id, a.pos))
            .collect();
        for (id, p) in caught {
            let edge = p.dist2(center) > inner;
            let damage = if edge { (roll / 2).max(1) } else { roll };
            let damage = Damage::new(damage, burst.kind);
            self.spell_hurts(caster, id, (center, false), damage, events);
        }
        if burst.blast > 0 {
            self.blast(center, radius, burst.blast, events);
        }
        match burst.ice {
            Some(BurstIce::Melts) => {
                self.shock(center, radius, Shock::Heat, events);
            }
            Some(BurstIce::Freezes(um)) => {
                self.frost(center, radius, um);
            }
            None => {}
        }
    }

    /// `bolt` runs from `origin` over `path` (its cells in order, as far as it reaches), cast by
    /// `caster`: its effect ([`Event::Spell`], drawn along the cells after its origin: a trap's
    /// own cell, first in its path, is struck but not drawn), and one roll of its damage for
    /// everyone on the path.
    pub(super) fn cast_bolt(
        &mut self,
        caster: Caster,
        origin: Pos,
        path: &[Pos],
        bolt: &Bolt,
        events: &mut Vec<Event>,
    ) {
        let drawn: Vec<Pos> = path.iter().copied().filter(|&p| p != origin).collect();
        if !drawn.is_empty() {
            events.push(Event::Spell {
                effect: bolt.spell.clone(),
                origin,
                shape: SpellShape::Bolt { path: drawn },
                cells: path.to_vec(),
            });
        }
        let roll = bolt.damage.roll(&mut self.state.rng);
        let level = self.state.current_level.clone();
        let mut caught = Vec::new();
        for p in path {
            caught.extend(self.state.actor_at(&level, *p).map(|a| a.id));
        }
        for id in caught {
            let damage = Damage::new(roll, bolt.kind);
            self.spell_hurts(caster, id, (origin, true), damage, events);
        }
    }

    /// What of a burst's or a bolt's `damage` reaches actor `id`: with Evasion (spec §8
    /// "Talents") its share, rounded down (a spell that does nothing more is resisted).
    pub(super) fn evaded(&self, id: ActorId, damage: Damage) -> Damage {
        let percent = (self.state.actors.get(&id))
            .map_or(0, |a| a.talent_mods(&self.content).spell_damage_percent);
        if percent == 0 {
            return damage;
        }
        let amount = u64::try_from(damage.amount.max(0)).unwrap_or(0);
        let amount = i32::try_from(TalentMods::scaled(amount, percent)).unwrap_or(i32::MAX);
        Damage::new(amount, damage.kind)
    }

    /// `caster`'s spell does `damage` to `target` from `source` (a cell, and whether it flew
    /// along a line from it), with no roll to hit, less what Evasion spares it ([`World::evaded`])
    /// and what the target resists of its kind (spec §8 "Damage types"): a trap's blow
    /// ([`World::trap_hurts`]), or an actor's, dealt
    /// the way every blow is (spec §8 "Blood", "Status effects"): the event, a big hit's blood,
    /// the damage, which may kill (a kill by the player earns its experience), and if it lives
    /// the statuses its wound causes. A shopkeeper the player's spell reaches turns on them, as
    /// one they strike does.
    pub(super) fn spell_hurts(
        &mut self,
        caster: Caster,
        target: ActorId,
        source: (Pos, bool),
        damage: Damage,
        events: &mut Vec<Event>,
    ) {
        let by = match caster {
            Caster::Trap(trap) => {
                let damage = self.evaded(target, damage);
                return self.trap_hurts(trap, target, source, damage, events);
            }
            Caster::Actor(by) => by,
        };
        let Some(to) = self.state.actors.get(&target).map(|a| a.pos) else {
            return;
        };
        let from = self.state.actors.get(&by).map_or(source.0, |a| a.pos);
        let damage = self.evaded(target, damage);
        let taken = self.resisted(target, damage);
        let mut blow = Blow::plain(taken, damage.kind);
        if damage.amount > 0 && taken <= 0 {
            blow.strike = Strike::Resisted;
        }
        events.push(Event::Attacked {
            attacker: by,
            target,
            kind: AttackKind::Spell,
            from,
            to,
            strike: Some(blow.strike),
            with: None,
            damage: damage.kind,
        });
        if by == self.state.player && target != by {
            self.provoke(target);
            self.provoke_court(target);
        }
        if taken <= 0 {
            return;
        }
        self.spill(by.0, target, source, blow);
        if !self.hurt(target, taken, Harm::Actor(by), events) {
            self.wounded(target, &blow, Some(by), events);
        }
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
            other.fingerprint_for("showcase"),
            builtin.fingerprint_for("showcase"),
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
