//! Creatures' bursts (the dread design `docs/design/monster-dread.md` §5.3, #1127): a creature
//! with `[creature.burst]` in its type (`content/burst.rs`) goes off in a burst of its element.
//! Enemy and ally alike; no player ability.
//!
//! **When**: a **vortex** (`when = "touches"`) bursts **instead of attacking**: wherever its AI
//! would strike in melee (`ai.rs::creature_turn`, every melee plan), it bursts, and dies of it.
//! It has no attack of its own (`content` refuses one). Killed, it goes quietly: the burst is
//! never a death's. **Death throes** (`when = "dies"`) go off where the creature dies, however
//! it dies ([`World::death_throes`], from `fight.rs::dies`), on the player's level only.
//!
//! **What** ([`World::go_off`]): an explosion is heard (the traps' `explosion`), and the burst
//! strikes through the spells' shared code ([`World::cast_burst_reaching`]): the effect drawn,
//! one roll of its damage for everyone on its cells, in ascending actor id order, **in full**
//! (no halving at the edge: it is the thing itself coming apart), with no roll to hit, Evasion
//! halving it and resistances taking their share; its own kind (the same type) is spared. What
//! lies on its cells may be ruined (`ruin.rs`), and its element's spell type does to the cells
//! what a spell's burst does (fire left burning on the centre, water frozen by cold). The spell
//! types' riders do not land: the burst's own `rider`s do, each by its chance, on everyone it
//! harmed. A **wave** (`douses`) ends what water ends (Burning) on everyone it reached and puts
//! out the fires on its cells; a **gust** (`gust`) blows what lies loose on its cells a cell
//! outward, away from the centre, where that cell can be walked on. Its blows are the
//! creature's (in `fallen` once it died): one that kills the player is what killed them; one
//! that kills a creature earns no one anything (a dead creature has no master to credit).
//!
//! **Words**: the player who sees its cell is told it bursts ("The ember mote bursts!", "The
//! fire elemental comes apart in flame!"); each blow on someone they see is said as a spell's
//! ("The flame burns you."); death throes are said after the death line that caused them (the
//! lines join the news). Unseen, the explosion is heard.
//!
//! **Determinism**: the damage's one roll, then one `below(100)` per rider below 100 per
//! creature harmed, in actor id order, all from the gameplay stream; the ruin's draws are its
//! own stream's. Nothing is saved: the burst is a type's data.

use crate::action::{AttackKind, Event};
use crate::combat::Strike;
use crate::content::{Burst, BurstWhen, CreatureId, TrapNoise, the_start};
use crate::geom::Pos;
use crate::noise::SoundKind;
use crate::object;
use crate::world::magic::spell::{Caster, Reach, burst_cells};
use crate::world::{ActorId, World};

/// How loud a burst is on its cell: a fireball trap's explosion.
const LOUDNESS: u16 = 150;

impl World {
    /// Whether creature `id` bursts instead of striking (a vortex).
    pub(super) fn bursts_on_touch(&self, id: ActorId) -> bool {
        let Some(me) = self.state.actors.get(&id) else {
            return false;
        };
        (self.content.creature(me.creature).burst.as_ref())
            .is_some_and(|b| b.when == BurstWhen::Touches)
    }

    /// Creature `id`, a vortex beside its target, bursts and dies of it. Returns the price: a
    /// melee attack's.
    pub(super) fn creature_bursts(&mut self, id: ActorId, events: &mut Vec<Event>) -> i64 {
        let kind = AttackKind::Melee;
        let skills = self.skills_of(id, kind, None);
        let price = self.attack_price(id, kind.action(), None, skills);
        let Some((here, creature)) = self.state.actors.get(&id).map(|a| (a.pos, a.creature)) else {
            return price;
        };
        self.go_off(id, here, creature, events);
        if self.state.actors.contains_key(&id) {
            self.dies(id, None, None, self.state.time, events);
        }
        price
    }

    /// Creature `id` of type `creature` just died on cell `at` of level `level`: its death
    /// throes, if its type has them and it died on the player's level. Its words join the news,
    /// said after the death that caused them.
    pub(in crate::world) fn death_throes(
        &mut self,
        id: ActorId,
        (at, level): (Pos, &str),
        creature: CreatureId,
        events: &mut Vec<Event>,
    ) {
        let throes = (self.content.creature(creature).burst.as_ref())
            .is_some_and(|b| b.when == BurstWhen::Dies);
        if !throes || level != self.state.current_level || self.state.death.is_some() {
            return;
        }
        // Said meanwhile, held to join the news (unless a free blow holds it already).
        let own = self.held.is_none();
        if own {
            self.held = Some(Vec::new());
        }
        self.go_off(id, at, creature, events);
        if own {
            let lines = self.held.take().unwrap_or_default();
            self.news.extend(lines);
        }
    }

    /// The burst of creature `id`'s type `creature` goes off around `here` (module docs).
    fn go_off(&mut self, id: ActorId, here: Pos, creature: CreatureId, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let def = content.creature(creature);
        let Some(burst) = def.burst.as_ref() else {
            return;
        };
        if self.perception.sees_actors_at(here) {
            self.say(format!("{} {}!", the_start(&def.name), burst.verb));
        }
        let sound = TrapNoise {
            sound: SoundKind::Explosion,
            loudness: LOUDNESS,
        };
        self.sound_at(Some(sound), here);
        let spell = Burst {
            spell: burst.effect.clone(),
            radius: burst.radius,
            damage: burst.damage.into(),
            reroll: 0,
            kind: Some(burst.kind),
            blast: 0,
            ice: None,
        };
        let reach = Reach {
            only: &[],
            spare: Some(creature),
            full: true,
        };
        let cells = {
            let level = self.state.current();
            burst_cells(here, burst.radius, |p| {
                p != here && level.is_opaque(&content, p)
            })
        };
        let mut made = Vec::new();
        self.cast_burst_reaching(Caster::Actor(id), (here, here), (&spell, reach), &mut made);
        let harmed: Vec<ActorId> = (made.iter())
            .filter_map(|e| match *e {
                Event::Attacked {
                    attacker,
                    target,
                    kind: AttackKind::Spell,
                    strike: Some(Strike::Hit { .. }),
                    ..
                } if attacker == id => Some(target),
                _ => None,
            })
            .collect();
        if burst.douses {
            self.wave(here, burst.radius, &cells, events);
        }
        if burst.gust {
            self.gust(here, &cells);
        }
        let verb = &content.damage_type(burst.kind).verb;
        self.say_spell((&burst.noun, verb), made, events);
        for &target in &harmed {
            for rider in &burst.riders {
                if !self.state.actors.contains_key(&target) || self.state.death.is_some() {
                    break;
                }
                if rider.chance >= 100 || self.state.rng.below(100) < u32::from(rider.chance) {
                    self.afflict(target, rider.status, rider.severity, Some(id), events);
                }
            }
        }
    }

    /// A wave around `center`: what water ends (Burning) ends on everyone on `cells`, and the
    /// fires there go out.
    fn wave(&mut self, center: Pos, radius: i32, cells: &[Pos], events: &mut Vec<Event>) {
        let level = self.state.current_level.clone();
        let wet: Vec<ActorId> = (self.state.actors.values())
            .filter(|a| a.level == level && cells.contains(&a.pos))
            .map(|a| a.id)
            .collect();
        for id in wet {
            self.quench(id, events);
        }
        self.douse(center, radius);
    }

    /// A gust around `center`: what lies on each of `cells` but the centre is blown a cell
    /// outward, onto the cell beyond it away from the centre, if that cell can be walked on.
    /// Every pile moves from where it lay before the gust, so none is blown twice.
    fn gust(&mut self, center: Pos, cells: &[Pos]) {
        let content = self.content.clone();
        let level = self.state.current();
        let moves: Vec<(Pos, Pos)> = (cells.iter().copied())
            .filter(|&p| p != center && !level.objects_at(p).is_empty())
            .filter_map(|p| {
                let to = Pos::new(
                    p.x + (p.x - center.x).signum(),
                    p.y + (p.y - center.y).signum(),
                );
                level.is_walkable(&content, to).then_some((p, to))
            })
            .collect();
        let lifted: Vec<(Pos, Vec<object::Stack>)> = (moves.iter())
            .map(|&(from, to)| {
                let level = self.state.current_mut();
                let pile = level.objects_at(from).to_vec();
                level.set_pile(from, Vec::new());
                (to, pile)
            })
            .collect();
        for (to, pile) in lifted {
            let level = self.state.current_mut();
            let mut there = level.objects_at(to).to_vec();
            object::add_to_pile(&mut there, &pile, &content);
            level.set_pile(to, there);
        }
    }
}
