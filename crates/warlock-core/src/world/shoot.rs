//! Ranged combat (spec §8 "Ranged combat"): shooting with a missile weapon, throwing, the flight
//! of a missile, where it comes down, and the fires of burning missiles.
//!
//! **Flight**: a missile flies along the integer line from the shooter through the cell aimed at
//! and on beyond it ([`Pos::ray`]: up to that cell the line a creature checks for a clear shot),
//! one cell after another, as far as its range (the rounded radius, like every radius), over
//! transparent cells only: it stops before the first opaque one. Whoever stands on its way is
//! attacked in turn, against their Defence, until it hits one. Aimed at a cell where nobody
//! stands it comes down on that cell; having missed whoever stood there, it flies on.
//!
//! **Where it comes down**: on the cell of whom it hit, else the last cell it flew over (at the end
//! of its range, before something solid, or the cell aimed at). It lies there on top of the pile,
//! to be picked up again, unless it broke (on a hit, by its `breaks` chance: one draw of the
//! gameplay stream) or the cell is no floor (not walkable): then it is lost.
//!
//! **Ammunition**: a shot takes one from the readied ammunition, a throw one from the stack
//! thrown; the last one empties its slot.
//!
//! **Burning missiles**: what is `lit` as it is loosed (a flaming arrow) flies and comes down as
//! the burning object (a burning arrow). Where it lies it is a fire ([`Fire`]): a light source,
//! its `light` preset, that shines by the rules of every light, for the player's eyes and every
//! creature's, until it has burned `burns` seconds; then it is gone ([`World::burn_out`]). Several
//! on one cell burn as one fire, until the last of them burns out. A burning object cannot be
//! picked up.
//!
//! A shot makes the noise of a bowstring and a throw that of fighting (quieter), once, as the
//! missile is loosed. A shot takes `shoot` (and `reload` too for a missile weapon that needs it),
//! a throw `throw`, less what the rank in the launcher's skill or in throwing takes off (spec §8
//! "Weapon skills"), which also adds to their to-hit and damage.

use super::fight::{End, Words, attack_message, death_message};
use super::{ActorId, Fire, Spent, World};
use crate::action::{AttackKind, Event, Letters, MissileEnd, letter};
use crate::combat::Profile;
use crate::content::{ActionKind, Content, Effect, LevelKind, ObjectId, Role};
use crate::fov::radius2;
use crate::geom::Pos;
use crate::magic::Mods;
use crate::magic::mods::less;
use crate::noise::Act;
use crate::object::{self, Stack};

/// A missile about to be loosed: a shot or a throw (`kind`), from inventory stack `item`, what
/// its attack does, and how far it flies.
#[derive(Clone, Copy, Debug)]
struct Loose {
    kind: AttackKind,
    item: usize,
    profile: Profile,
    range: i32,
}

/// What ammunition of the kind missile weapon `launcher` shoots is called, in the plural:
/// "arrows", "bolts".
fn ammunition_name(content: &Content, launcher: ObjectId) -> String {
    let kind = content.object(launcher).launches.as_ref();
    (content.object_defs().iter())
        .find(|o| o.ammo.is_some() && o.ammo.as_ref() == kind)
        .map_or_else(|| "ammunition".into(), |o| o.plural.clone())
}

/// The chance in percent that `missile` breaks when it hits: its `breaks`, less what a durability
/// affix takes off (spec §8 "Magic items").
pub(super) fn break_chance(missile: &Stack, content: &Content) -> u32 {
    let def = content.object(missile.object);
    let sturdier = Mods::local(missile, content).get(Effect::Durability);
    u32::try_from(less(u64::from(def.breaks), sturdier)).unwrap_or(0)
}

impl World {
    /// The player shoots at cell `at` with the missile weapon and ammunition they have ready.
    pub(super) fn fire(&mut self, at: Pos, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        if !self.can_loose_here() {
            return Spent::Nothing;
        }
        let me = self.state.player();
        let (Some((launcher, _)), Some(profile)) = (me.launcher(&content), me.ranged(&content))
        else {
            self.say("You have nothing ready to throw or shoot.");
            return Spent::Nothing;
        };
        let Some(item) = me.ammunition_item(&content) else {
            let what = ammunition_name(&content, launcher);
            self.say(format!("You have no {what} ready."));
            return Spent::Nothing;
        };
        let range = me.launcher_range(&content).unwrap_or(1);
        if !self.within_reach(at, range, "shoot") {
            return Spent::Nothing;
        }
        let loose = Loose {
            kind: AttackKind::Ranged,
            item,
            profile,
            range,
        };
        self.player_looses(at, loose, events);
        // Priced before the exertion, which may leave the player hungrier (no longer Bloated).
        let price = self.shot_price(self.state.player, launcher);
        self.exert(ActionKind::Shoot);
        Spent::Turn(price)
    }

    /// What a shot from missile weapon `launcher` costs actor `id`: `shoot`, and `reload` too
    /// for one that needs it, less what its rank in the launcher's skill takes off.
    fn shot_price(&self, id: ActorId, launcher: ObjectId) -> i64 {
        let def = self.content.object(launcher);
        let mut price = self.attack_price(id, ActionKind::Shoot, def.skill);
        if def.reloads {
            price += self.attack_price(id, ActionKind::Reload, def.skill);
        }
        price
    }

    /// The player throws one of inventory stack `item`, a throwing weapon, at cell `at`; any
    /// other weapon is thrown whole, with a penalty to hit ([`Profile::improvised`]).
    pub(super) fn throw(&mut self, item: usize, at: Pos, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        if !self.can_loose_here() {
            return Spent::Nothing;
        }
        let me = self.state.player();
        let Some(stack) = me.inventory.get(item) else {
            let c = letter(item).unwrap_or('?');
            self.say(format!("You carry no {c}."));
            return Spent::Nothing;
        };
        let (Some(range), Some(profile)) =
            (me.throw_range(item, &content), me.thrown(item, &content))
        else {
            let what = stack.single().describe(&content);
            self.say(format!("You cannot throw {what}."));
            return Spent::Nothing;
        };
        if !self.within_reach(at, range, "throw") {
            return Spent::Nothing;
        }
        let loose = Loose {
            kind: AttackKind::Thrown,
            item,
            profile,
            range,
        };
        self.player_looses(at, loose, events);
        let throwing = content.weapon_skills().of_role(Role::Thrown);
        let price = self.attack_price(self.state.player, ActionKind::Throw, throwing);
        self.exert(ActionKind::Throw);
        Spent::Turn(price)
    }

    /// Whether missiles fly here: not on the overworld, a map of the land.
    fn can_loose_here(&mut self) -> bool {
        let here = self.state.current().kind != LevelKind::Overworld;
        if !here {
            self.say("You cannot do that on the overworld.");
        }
        here
    }

    /// Whether the player may `verb` ("shoot", "throw") at cell `at` with a missile that flies
    /// `range` cells: not at themselves, and not out of range. Says why not.
    fn within_reach(&mut self, at: Pos, range: i32, verb: &str) -> bool {
        let here = self.player_pos();
        if at == here {
            self.say(format!("You cannot {verb} at yourself."));
            false
        } else if at.dist2(here) > radius2(range) {
            self.say("That is out of range.");
            false
        } else {
            true
        }
    }

    /// The player looses `loose` at `at`. Its attacks are said at once, by what the player sees
    /// as they shoot (as their melee blows are); one that reaches nobody is said as it is
    /// loosed ("You shoot an arrow."). The missile's flight goes to presentation (cut to what
    /// the player sees as the turn is perceived), its attacks on creatures they see too.
    fn player_looses(&mut self, at: Pos, loose: Loose, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let stack = self.state.player().inventory[loose.item].single();
        let last = (self.state.player().inventory[loose.item].count == 1)
            && loose.kind == AttackKind::Ranged;
        let mut made = Vec::new();
        self.loose(self.state.player, at, loose, &mut made);
        let words = Words::of(
            self.state.player(),
            loose.kind,
            Some(stack.object),
            &content,
        );
        let mut said = Vec::new();
        // Whether the player saw the target of the last attack, and how it reads.
        let mut seen: Option<End<'_>> = None;
        let mut attacked = false;
        for event in made {
            match event {
                Event::Attacked {
                    target, to, strike, ..
                } => {
                    attacked = true;
                    let fallen = self.actor_or_fallen(target);
                    let def = fallen.map(|a| content.creature(a.creature));
                    let hide = def
                        .and_then(|d| d.natural.as_ref())
                        .map(|h| h.name.as_str());
                    let end = match def {
                        Some(d) if self.perception.sees_actors_at(to) => End::Seen(&d.name),
                        _ => End::Unseen,
                    };
                    said.extend(attack_message(
                        End::You,
                        end,
                        &words,
                        loose.kind,
                        strike,
                        hide,
                    ));
                    seen = (end != End::Unseen).then_some(end);
                    if seen.is_some() {
                        events.push(event);
                    }
                }
                Event::Died { .. } => {
                    if let Some(end) = seen {
                        said.push(death_message(end));
                        events.push(event);
                    }
                }
                other => events.push(other),
            }
        }
        if !attacked {
            let verb = match loose.kind {
                AttackKind::Thrown => "throw",
                _ => "shoot",
            };
            let what = stack.describe(&content);
            said.insert(0, format!("You {verb} {what}."));
        }
        if last {
            let ammo = &content.object(stack.object).plural;
            said.push(format!("You have no {ammo} left."));
        }
        for text in said {
            self.say(text);
        }
        self.say_news();
    }

    /// Creature `id` shoots at actor `target` with the missile weapon and ammunition it has
    /// ready (its AI saw to it that it has them). Returns the price: `shoot`, and `reload` too
    /// for a missile weapon that needs it.
    pub(super) fn creature_shoots(
        &mut self,
        id: ActorId,
        target: ActorId,
        events: &mut Vec<Event>,
    ) -> i64 {
        let content = self.content.clone();
        let me = &self.state.actors[&id];
        let at = self.state.actors.get(&target).map(|a| a.pos);
        let (Some((launcher, _)), Some(profile), Some(item), Some(at)) = (
            me.launcher(&content),
            me.ranged(&content),
            me.ammunition_item(&content),
            at,
        ) else {
            return self.price_of(id, ActionKind::Wait);
        };
        let loose = Loose {
            kind: AttackKind::Ranged,
            item,
            profile,
            range: me.launcher_range(&content).unwrap_or(1),
        };
        self.loose(id, at, loose, events);
        self.shot_price(id, launcher)
    }

    /// Actor `shooter` looses one missile of `loose` at cell `at` (module docs): it leaves the
    /// stack, makes its noise, flies and attacks whoever stands in its way until it hits, and
    /// comes down. Pushes the [`Event::Missile`] (the whole flight; perception cuts it to what
    /// the player saw), then the attacks it made and the deaths they caused.
    fn loose(&mut self, shooter: ActorId, at: Pos, loose: Loose, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let Some(me) = self.state.actors.get_mut(&shooter) else {
            return;
        };
        let from = me.pos;
        let Some(stack) = me.inventory.get_mut(loose.item) else {
            return;
        };
        let missile = stack.single();
        let ammo = stack.object;
        stack.count -= 1;
        if stack.count == 0 {
            me.inventory.remove(loose.item);
            me.equipment.after_removal(Letters::of([loose.item]));
        }
        let def = content.object(ammo);
        let flies = def.lit.unwrap_or(ammo);
        // What is lit flies as another object (a burning arrow), its magic gone; anything else
        // keeps its magic as it flies and lands (spec §8 "Magic items"), and a durable one breaks
        // less often.
        let breaks = break_chance(&missile, &content);
        let lands = match def.lit {
            Some(lit) => Stack::one(lit),
            None => missile,
        };
        self.make_noise(shooter, Act::Attack(loose.kind));

        let reach = radius2(loose.range);
        let mut path = Vec::new();
        let mut attacks = Vec::new();
        let mut struck = false;
        for cell in from.ray(at) {
            let level = self.state.current();
            if cell.dist2(from) > reach || level.is_opaque(&content, cell) {
                break;
            }
            path.push(cell);
            let target = (self.state.actor_at(&level.id, cell))
                .filter(|a| a.id != shooter)
                .map(|a| a.id);
            match target {
                Some(target) => {
                    let with = (loose.kind, ammo);
                    let blow =
                        self.missile_hits(shooter, target, with, loose.profile, &mut attacks);
                    if blow.is_some_and(|b| b.strike.is_hit()) {
                        struck = true;
                        break;
                    }
                }
                None if cell == at => break,
                None => {}
            }
        }
        let end = path.last().copied().unwrap_or(from);
        let broke = struck && breaks > 0 && self.state.rng.chance(breaks);
        if !broke && self.state.current().is_walkable(&content, end) {
            self.come_down(end, lands);
        }
        events.push(Event::Missile {
            object: flies,
            from: Some(from),
            path,
            end: if struck {
                MissileEnd::Struck
            } else {
                MissileEnd::Fell
            },
        });
        events.extend(attacks);
    }

    /// One `missile` comes down on cell `p` of the current level, on top of the pile there. If
    /// it burns, it starts a fire there, or the fire there burns on until it has burned out too.
    fn come_down(&mut self, p: Pos, missile: Stack) {
        let object = missile.object;
        let content = self.content.clone();
        let now = self.state.time;
        if self
            .state
            .current()
            .fires
            .get(&p)
            .is_some_and(|f| f.until <= now)
        {
            self.put_out(p);
        }
        let level = self.state.current_mut();
        let mut pile = level.objects_at(p).to_vec();
        object::add_to_pile(&mut pile, &[missile], &content);
        level.set_pile(p, pile);
        if let Some(burning) = content.object(object).burning {
            let until = now + burning.lasts;
            let fire = (level.fires.entry(p)).or_insert(Fire {
                light: burning.light,
                until,
            });
            fire.light = burning.light;
            fire.until = fire.until.max(until);
        }
    }

    /// Fires on the current level that have burned out go out, and what burned there is gone.
    /// (Until then a fire past its time gives no light: [`crate::light::LightCache`] lights only
    /// fires still burning.)
    pub(super) fn burn_out(&mut self) {
        let now = self.state.time;
        let out: Vec<Pos> = (self.state.current().fires.iter())
            .filter(|(_, f)| f.until <= now)
            .map(|(p, _)| *p)
            .collect();
        for p in out {
            self.put_out(p);
        }
    }

    /// The fire on cell `p` goes out: it and the burning objects there are gone.
    fn put_out(&mut self, p: Pos) {
        let content = self.content.clone();
        let level = self.state.current_mut();
        level.fires.remove(&p);
        let pile: Vec<Stack> = (level.objects_at(p).iter())
            .filter(|s| content.object(s.object).burning.is_none())
            .cloned()
            .collect();
        level.set_pile(p, pile);
    }
}
