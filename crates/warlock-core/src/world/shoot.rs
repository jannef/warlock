//! Ranged combat (spec §8 "Ranged combat"): shooting with a missile weapon, throwing, the flight
//! of a missile, where it comes down, and the fires of burning missiles.
//!
//! **Flight**: a missile flies along the integer line from the shooter through the cell aimed at
//! and on beyond it ([`Pos::ray`]: up to that cell the line a creature checks for a clear shot),
//! one cell after another, as far as its range (the rounded radius, like every radius), over
//! transparent cells only: it stops before the first opaque one (attacking an incorporeal
//! creature that stands inside it first, `incorporeal.rs`). Whoever stands on its way is
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

use super::fight::{End, Guard, Words, attack_message, death_message};
use super::{ActorId, Fire, Spent, World};
use crate::action::{AttackKind, Event, MissileEnd, Positions};
use crate::combat::Profile;
use crate::content::{ActionKind, Content, Effect, LevelKind, ObjectId};
use crate::fov::radius2;
use crate::geom::Pos;
use crate::magic::Mods;
use crate::magic::mods::less;
use crate::noise::Act;
use crate::object::{self, Stack};

/// A missile about to be loosed: a shot or a throw (`kind`), from inventory stack `item`, what
/// its attack does, and how far it flies.
#[derive(Clone, Debug)]
pub(super) struct Loose {
    pub(super) kind: AttackKind,
    pub(super) item: usize,
    pub(super) profile: Profile,
    pub(super) range: i32,
}

/// A missile in flight ([`World::fly`]): a shot's, a throw's, or a creature's throw's.
#[derive(Clone, Debug)]
pub(super) struct Flight {
    pub(super) kind: AttackKind,
    /// What its attacks are made with (an arrow, a dagger, a giant's boulder).
    pub(super) with: ObjectId,
    /// What flies, as it flies (a flaming arrow flies burning).
    pub(super) flies: ObjectId,
    pub(super) profile: Profile,
    /// How far it flies, in cells.
    pub(super) range: i32,
    /// What comes down where it stops, unless it broke; `None`: nothing (a creature's throw).
    pub(super) lands: Option<Stack>,
    /// The chance in percent that it breaks on a hit.
    pub(super) breaks: u32,
}

/// What ammunition of the kind missile weapon `launcher` shoots is called, in the plural:
/// "arrows", "bolts".
fn ammunition_name(content: &Content, launcher: ObjectId) -> String {
    let kind = content.object(launcher).launches.as_deref();
    kind.and_then(|k| content.ammunition(k))
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
        // A wand in the missile weapon slot casts its spell (spec §8 "Magic items": "Wands").
        if let Some(item) = me.wand_item(&content) {
            return self.zap(item, at, events);
        }
        // A rod there does its effect for the user's mana (spec §8 "Magic items": "Rods").
        if let Some(item) = me.rod_item(&content) {
            return self.use_rod(item, at, events);
        }
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
        if !self.may_loose(item) {
            return Spent::Nothing;
        }
        if !self.within_reach(at, range, "shoot") {
            return Spent::Nothing;
        }
        let loose = Loose {
            kind: AttackKind::Ranged,
            item,
            profile,
            range,
        };
        // Priced as the shot begins (every attack is), before the missile leaves and the
        // exertion, which may leave the player hungrier (no longer Bloated).
        let price = self.shot_price(self.state.player, launcher);
        let price = self.rapid_shot(self.state.player, price, events);
        self.player_looses(at, loose, events);
        self.exert(ActionKind::Shoot);
        Spent::Turn(price)
    }

    /// What a shot from missile weapon `launcher` costs actor `id`: `shoot`, and `reload` too
    /// for one that needs it, less what its ranks in the launcher's skill and its fighting style
    /// take off.
    pub(super) fn shot_price(&self, id: ActorId, launcher: ObjectId) -> i64 {
        let shot = self.skills_of(id, AttackKind::Ranged, None);
        let with = self.state.actors[&id].launcher_item(&self.content);
        let mut price = self.attack_price(id, ActionKind::Shoot, with, shot);
        if self.content.object(launcher).reloads {
            price += self.reload_price(id, shot);
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
            self.say("You carry no such thing.");
            return Spent::Nothing;
        };
        let (Some(range), Some(profile)) =
            (me.throw_range(item, &content), me.thrown(item, &content))
        else {
            let what = self.lore().describe(&stack.single());
            self.say(format!("You cannot throw {what}."));
            return Spent::Nothing;
        };
        let throwing = me.attack_skills(&content, AttackKind::Thrown, Some(stack.object));
        if !self.may_loose(item) || !self.within_reach(at, range, "throw") {
            return Spent::Nothing;
        }
        let loose = Loose {
            kind: AttackKind::Thrown,
            item,
            profile,
            range,
        };
        // Priced as the throw begins, with what is thrown still in the pack.
        let price = self.attack_price(self.state.player, ActionKind::Throw, None, throwing);
        self.player_looses(at, loose, events);
        self.exert(ActionKind::Throw);
        Spent::Turn(price)
    }

    /// Whether missiles fly here: not on the overworld, a map of the land.
    pub(super) fn can_loose_here(&mut self) -> bool {
        let here = self.state.current().kind != LevelKind::Overworld;
        if !here {
            self.say("You cannot do that on the overworld.");
        }
        here
    }

    /// Whether the player may `verb` ("shoot", "throw") at cell `at` with a missile that flies
    /// `range` cells: not at themselves, and not out of range. Says why not.
    pub(super) fn within_reach(&mut self, at: Pos, range: i32, verb: &str) -> bool {
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
    pub(super) fn player_looses(&mut self, at: Pos, loose: Loose, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let kind = loose.kind;
        let stack = self.state.player().inventory[loose.item].single();
        let last =
            (self.state.player().inventory[loose.item].count == 1) && kind == AttackKind::Ranged;
        let mut made = Vec::new();
        self.loose(self.state.player, at, loose, &mut made);
        let words = Words::of(
            self.state.player(),
            kind,
            Some(stack.object),
            None,
            &content,
        );
        let mut said = Vec::new();
        // Whether the player saw the target of the last attack, and how it reads.
        let mut seen: Option<(End<'_>, &str)> = None;
        let mut attacked = false;
        for event in made {
            match event {
                Event::Attacked {
                    target, to, strike, ..
                } => {
                    attacked = true;
                    let fallen = self.actor_or_fallen(target);
                    let def = fallen.map(|a| content.creature(a.creature));
                    let guard = Guard::of(fallen, &content, &self.state);
                    let end = match def {
                        Some(d) if self.perception.sees_actors_at(to) => End::Seen(&d.name),
                        _ => End::Unseen,
                    };
                    said.extend(attack_message(End::You, end, &words, kind, strike, guard));
                    seen =
                        (end != End::Unseen).then(|| (end, def.map_or("dies", |d| d.death_verb())));
                    if seen.is_some() {
                        events.push(event);
                    }
                }
                Event::Died { .. } => {
                    if let Some((end, dies)) = seen {
                        said.push(death_message(end, dies));
                        events.push(event);
                    }
                }
                other => events.push(other),
            }
        }
        if !attacked {
            let verb = match kind {
                AttackKind::Thrown => "throw",
                _ => "shoot",
            };
            let what = self.lore().describe(&stack);
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
        let price = self.shot_price(id, launcher);
        let at = self.aim_at(id, target, at);
        self.loose(id, at, loose, events);
        price
    }

    /// Actor `shooter` looses one missile of `loose` at cell `at` (module docs): it leaves the
    /// stack, makes its noise, flies and attacks whoever stands in its way until it hits, and
    /// comes down ([`World::fly`]).
    fn loose(&mut self, shooter: ActorId, at: Pos, loose: Loose, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let Some(me) = self.state.actors.get_mut(&shooter) else {
            return;
        };
        // Fletching (spec §8 "Class abilities"): the shooter's missiles break less often.
        let more = me.talent_mods(&content).missile_breaks;
        let Some(stack) = me.inventory.get_mut(loose.item) else {
            return;
        };
        let missile = stack.single();
        let ammo = stack.object;
        stack.count -= 1;
        if stack.count == 0 {
            me.inventory.remove(loose.item);
            me.equipment.after_removal(&Positions::of([loose.item]));
        }
        let def = content.object(ammo);
        // What is lit flies as another object (a burning arrow), its magic gone; anything else
        // keeps its magic as it flies and lands (spec §8 "Magic items"), and a durable one breaks
        // less often.
        let breaks = break_chance(&missile, &content);
        let breaks = u32::try_from(crate::talent::TalentMods::scaled(u64::from(breaks), more))
            .unwrap_or(0)
            .min(100);
        let lands = match def.lit {
            Some(lit) => Stack::one(lit),
            None => missile,
        };
        self.make_noise(shooter, Act::Attack(loose.kind));
        let flight = Flight {
            kind: loose.kind,
            with: ammo,
            flies: def.lit.unwrap_or(ammo),
            profile: loose.profile,
            range: loose.range,
            lands: Some(lands),
            breaks,
        };
        self.fly(shooter, at, flight, events);
    }

    /// `flight`, loosed by actor `shooter` at cell `at`, flies (module docs) and attacks
    /// whoever stands in its way until it hits, and what lands comes down, unless it broke.
    /// Pushes the [`Event::Missile`] (the whole flight; perception cuts it to what the player
    /// saw), then the attacks it made and the deaths they caused.
    pub(super) fn fly(
        &mut self,
        shooter: ActorId,
        at: Pos,
        flight: Flight,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let Some(from) = self.state.actors.get(&shooter).map(|a| a.pos) else {
            return;
        };
        let reach = radius2(flight.range);
        let with = (flight.kind, flight.with);
        let mut path = Vec::new();
        let mut attacks = Vec::new();
        let mut struck = false;
        // Aimed at a reflection in a hall of mirrors (`mirrors.rs`), it flies on through it.
        let through = self.reflected(shooter, at);
        for cell in from.ray(at) {
            let level = self.state.current();
            if cell.dist2(from) > reach {
                break;
            }
            if level.is_opaque(&content, cell) {
                // An incorporeal creature inside the wall (`incorporeal.rs`) is attacked there:
                // a hit is lost in the stone with it, a miss falls before the wall.
                let inside = (self.state.actor_at(&level.id, cell))
                    .filter(|a| a.id != shooter)
                    .map(|a| a.id);
                if let Some(target) = inside {
                    let profile = flight.profile.clone();
                    let blow = self.missile_hits(shooter, target, with, profile, &mut attacks);
                    if blow.is_some_and(|b| b.strike.is_hit()) {
                        path.push(cell);
                        struck = true;
                    }
                }
                break;
            }
            path.push(cell);
            let target = (self.state.actor_at(&level.id, cell))
                .filter(|a| a.id != shooter)
                .map(|a| a.id);
            match target {
                Some(target) => {
                    let profile = flight.profile.clone();
                    let blow = self.missile_hits(shooter, target, with, profile, &mut attacks);
                    if blow.is_some_and(|b| b.strike.is_hit()) {
                        struck = true;
                        break;
                    }
                }
                None if cell == at && !through => break,
                None => {}
            }
        }
        let end = path.last().copied().unwrap_or(from);
        let broke = struck && flight.breaks > 0 && self.state.rng.chance(flight.breaks);
        if let Some(lands) = flight.lands
            && !broke
            && self.state.current().is_walkable(&content, end)
        {
            self.come_down(end, lands);
        }
        events.push(Event::Missile {
            object: flight.flies,
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
    pub(super) fn come_down(&mut self, p: Pos, missile: Stack) {
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
