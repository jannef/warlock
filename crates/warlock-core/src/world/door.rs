//! Doors in the world (spec §8 "Doors", [`crate::door`]): opening (and unlocking), closing, and
//! kicking.
//!
//! - **Opening** a closed door swings it open (`open_door`, a noise). A locked one opens only
//!   to an actor carrying a key that fits ([`super::Actor::key_for`]): it is unlocked (the lock
//!   is gone) and opened in one action, which costs `unlock` plus `open_door`. Without one the
//!   player is told "The door is locked." and no time passes; a creature that opens doors treats
//!   it as a wall.
//! - **Kicking** (`kick`): at a closed door (locked or not), a banging noise on the door and one
//!   draw of the gameplay stream: with a chance of the kicker's Strength score in the door's
//!   sturdiness it breaks ("The door crashes open!", the terrain's `breaks_to`, its lock gone),
//!   else "WHAMM!". At a creature, an unarmed blow ([`World::player_kicks`]; it draws on unarmed
//!   fighting, spec §8 "Weapon skills"). At anything else it is refused and takes no time:
//!   "You kick at empty space." where one could walk, else "Kicking the stone wall would only
//!   hurt your foot."
//! - **Trapped doors** (spec §8 "Trapped doors"): a trap on a door goes off
//!   ([`World::spring_trap`]) at whoever opens the door (unlocking it too: [`World::swing_open`],
//!   the player and creatures alike) or kicks it, as it swings open or as the kick lands, before
//!   the door breaks; walking through the doorway never sets it off. A door kicked in takes its
//!   trap with it ([`World::set_terrain`]).

use super::{ActorId, AttackSkills, Spent, World};
use crate::action::Event;
use crate::content::Effect as ItemEffect;
use crate::content::{ActionKind, Role};
use crate::door::kick_breaks;
use crate::geom::{Dir, Pos};
use crate::noise::Act;
use crate::skills::{Effect, Use};
use crate::stats::Stat;

impl World {
    /// The player opens the door at `p`; a locked one only with a key that fits, which unlocks
    /// it first. Returns what it cost.
    pub(super) fn open_door(&mut self, p: Pos, events: &mut Vec<Event>) -> Spent {
        // What they take for a chest is a mimic (`mimic.rs`).
        if let Some(spent) = self.touch(p, events) {
            return spent;
        }
        let terrain = self.state.current().terrain.get(p).copied();
        let def = terrain.map(|t| self.content.terrain(t));
        match def {
            Some(def) if def.is_closed_door() => {}
            // A shut chest opens as a door does (spec §8 "Chests").
            Some(def) if def.is_shut_chest() => return self.open_chest(p, events),
            Some(def) if def.is_open_door() => {
                self.say("That door is already open.");
                return Spent::Nothing;
            }
            _ => {
                self.say("There is nothing to open there.");
                return Spent::Nothing;
            }
        }
        let player = self.state.player;
        let Some(lock) = self.state.current().locks.get(&p).cloned() else {
            let price = self.price_of(player, ActionKind::OpenDoor);
            self.swing_open(player, p, events);
            self.say("You open the door.");
            return Spent::Turn(price);
        };
        let Some(key) = self.state.player().key_for(&self.content, &lock) else {
            self.say("The door is locked.");
            return Spent::Nothing;
        };
        let price =
            self.price_of(player, ActionKind::Unlock) + self.price_of(player, ActionKind::OpenDoor);
        self.state.current_mut().locks.remove(&p);
        self.swing_open(player, p, events);
        let key = self.content.object(key).name.clone();
        self.say(format!("You unlock the door with your {key} and open it."));
        Spent::Turn(price)
    }

    /// Actor `id` swings the closed door at `p` open (unlocked by now): its noise, the terrain
    /// it opens to, and the event; then a trap on the door goes off at them. No pit lies on a
    /// door (content refuses one), so opening it costs no more than it did.
    pub(super) fn swing_open(&mut self, id: ActorId, p: Pos, events: &mut Vec<Event>) {
        let terrain = self.state.current().terrain[p];
        let opened = (self.content.terrain(terrain).opens_to).expect("closed doors have opens_to");
        self.make_noise(id, Act::OpenDoor(p));
        self.set_terrain(p, opened);
        events.push(Event::DoorOpened { pos: p });
        self.spring_trap(p, id, events);
    }

    /// The player closes the open door at `p`, unless someone or something is in the doorway.
    /// Returns whether they did.
    pub(super) fn close_door(&mut self, p: Pos, events: &mut Vec<Event>) -> bool {
        let terrain = self.state.current().terrain.get(p).copied();
        let def = terrain.map(|t| self.content.terrain(t));
        match def {
            Some(def) if def.is_open_door() => {
                let closed = def.closes_to.expect("open doors have closes_to");
                if self.state.actor_at(&self.state.current_level, p).is_some()
                    || !self.state.current().objects_at(p).is_empty()
                {
                    self.say("Something is in the way.");
                    return false;
                }
                self.make_noise(self.state.player, Act::CloseDoor(p));
                self.set_terrain(p, closed);
                events.push(Event::DoorClosed { pos: p });
                self.say("You close the door.");
                true
            }
            Some(def) if def.is_closed_door() => {
                self.say("That door is already closed.");
                false
            }
            _ => {
                self.say("There is nothing to close there.");
                false
            }
        }
    }

    /// The player kicks in `dir`: whoever stands there (an unarmed blow), else a closed door
    /// there (it may break). Kicking at anything else is refused and takes no time.
    pub(super) fn kick(&mut self, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let at = self.player_pos().step(dir);
        // What they take for a chest is a mimic (`mimic.rs`).
        if let Some(spent) = self.touch(at, events) {
            return spent;
        }
        let player = self.state.player;
        let level = self.state.current();
        if let Some(other) = self.state.actor_at(&level.id, at) {
            let target = other.id;
            let seen = self.perception.sees_actors_at(at);
            let unarmed = self.content.weapon_skills().of_role(Role::Unarmed);
            let skills = AttackSkills::only(unarmed);
            let price = self.attack_price(player, ActionKind::Kick, None, skills);
            self.player_kicks(target, seen, events);
            return Spent::Turn(price);
        }
        let def = level.terrain.get(at).map(|t| self.content.terrain(*t));
        match def {
            Some(def) if def.is_closed_door() => self.kick_door(at, events),
            Some(def) if def.is_shut_chest() => self.kick_chest(at, events),
            Some(def) if !def.walkable => {
                let name = def.name.clone();
                self.say(format!("Kicking the {name} would only hurt your foot."));
                Spent::Nothing
            }
            _ => {
                self.say("You kick at empty space.");
                Spent::Nothing
            }
        }
    }

    /// The player kicks the closed door at `p`: a banging noise, and with a chance of their
    /// Strength in its sturdiness it breaks for good (one draw of the gameplay stream; a door
    /// that cannot break draws nothing). Their athletics adds to the Strength (spec §8
    /// "Skills"), and the kick trains it. A trap on the door goes off at them as the kick lands,
    /// before the door gives way. Kicking a door is no attack: neither the combat stance nor
    /// attack speed changes its time.
    /// The Strength the player's kick is rolled against, a door's or a chest's: their
    /// effective Strength, their athletics (spec §8 "Skills"), their talents' (Mighty kick,
    /// Breaker, Lockbreaker; spec §8 "Talents"), and what they wear (Hobnailed; spec §8 "Magic
    /// items"): as if stronger.
    pub(super) fn kick_force(&self) -> u8 {
        let me = self.state.player();
        let strength = me.effective_stats(&self.content).score(Stat::Strength);
        let athletics = me.skill_effect(&self.content, Effect::Kick);
        let mighty = me.talent_mods(&self.content).door_kick
            + me.mods(&self.content).get(ItemEffect::DoorKick);
        u8::try_from((i32::from(strength) + athletics + mighty).max(0)).unwrap_or(u8::MAX)
    }

    fn kick_door(&mut self, p: Pos, events: &mut Vec<Event>) -> Spent {
        let player = self.state.player;
        let price = self.price_of(player, ActionKind::Kick);
        let def = self.content.terrain(self.state.current().terrain[p]);
        let force = self.kick_force();
        let broken = def.kicked_in.and_then(|(to, sturdiness)| {
            kick_breaks(force, sturdiness, &mut self.state.rng).then_some(to)
        });
        if self.trains(Use::Kick) {
            self.train(Use::Kick);
        }
        self.make_noise(player, Act::Kick(p));
        self.spring_trap(p, player, events);
        match broken {
            Some(to) => {
                self.set_terrain(p, to);
                events.push(Event::DoorBroken { pos: p });
                self.say("The door crashes open!");
            }
            None => {
                events.push(Event::DoorKicked { pos: p });
                self.say("WHAMM!");
            }
        }
        Spent::Turn(price)
    }
}
