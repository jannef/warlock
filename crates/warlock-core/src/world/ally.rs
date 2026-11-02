//! Allies (spec §8 "Allies"; docs/design/druid-and-summons.md §4 and §14; `docs/dev/allies.md`):
//! creatures on the player's side. An ally is an ordinary [`Actor`] with [`Actor::ally`] set:
//! its type's stats, speed, eyes, and attacks, scheduled, hurt, and killed as any creature. Its
//! **side** is its master's: it is made with the player's faction as its `faction_override`, so
//! every rule that reads stances (enemy AI, the spawn cap, the quick leave, coarse turns,
//! displacement) treats it as the player's with no branch of its own.
//!
//! **Its mind** ([`World::ally_plan`], the one arm at the top of `ai.rs::decide`): it attacks
//! the nearest actor it sees that its **master** is hostile to (`World::stance(master, other)`:
//! a Beast-friend's wolf leaves the rats alone as the druid does, and a shopkeeper is never an
//! ally's idea), else obeys its [`Order`]: *Follow* (heel within `heel` cells of the master),
//! *Stay* (hold its post, fighting only what comes within `stay.reach` of it), *Hunt* (go to the
//! cell it was set on and fight what is within `hunt.reach` of it; arrived with nothing to
//! fight, it falls back to *Follow*). A following ally fighting farther than `leash` from its
//! master breaks off and heels. Deciding draws nothing.
//!
//! **Orders** ([`World::order`]; `Action::Order`, the three abilities of the Allies group):
//! Come to heel, Hold here, and Set on (a cell) are given to every ally of the player on the
//! level at once, take **no time**, and make a voice's noise (`[orders] noise`) where the player
//! stands. Each order marks Handle animal ([`Use::Handle`]), and so does an ally's kill.
//!
//! **Following across levels** ([`World::follow_along`], from `World::travel`): allies within
//! `comes_along` cells of the player as they leave come with them, placed beside their arrival;
//! the rest stay where they are, frozen as another level's actors are. Onto the **overworld**,
//! which has no creatures, they join the **retinue** instead ([`Ally::retinue`]: on the
//! overworld's level, never drawn, never acting, never standing in anyone's way), and step out
//! beside the player at the next level.
//!
//! **Time** ([`Ally::until`]): a summon's stay ends by the clock. Its departure time is stored
//! as absolute game time, so it runs on in the retinue and on a level the player left; one past
//! its time departs at the end of the turn where the player is ([`World::allies_lapse`]), and is
//! removed silently where they are not: on return to its level, or as the retinue steps out.
//!
//! **Experience**: an ally's kill pays its master as their own blow would, at `[xp]
//! ally_percent` (`fight.rs::dies`): the first-kill bonus, the quest tally, Favoured prey, the
//! named monsters slain, the resources a kill builds. The ally gets nothing.
//!
//! Nothing here draws from any stream, and a run with no allies calls none of it.

use super::ai::{Plan, Route};
use super::{Actor, ActorId, Mind, Spent, World, WorldState, stance_between};
use crate::action::{Command, Event};
use crate::combat::Profile;
use crate::content::{
    ActionKind, Content, CreatureId, CreatureSpawn, LevelKind, SpellId, Stance, and_list, counted,
    the_start,
};
use crate::geom::Pos;
use crate::noise::Act;
use crate::skills::Use;
use crate::time::{Duration, GameTime};
use serde::{Deserialize, Serialize};

/// The most allies the dev `ally:` action makes at once.
pub const MAX_DEV_ALLIES: u8 = 12;
/// How far from the player the dev `ally:` action places them (Chebyshev).
const DEV_SPREAD: i32 = 2;

/// Whose ally an actor is, and what it was told ([`Actor::ally`]).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ally {
    /// The actor it serves: the player, today; the field makes no assumption.
    pub master: ActorId,
    /// What made it an ally.
    pub source: AllySource,
    /// What it was last told, and so what it does with nothing to fight.
    #[serde(default)]
    pub order: Order,
    /// Travelling with its master off any level (the module docs): on the overworld's level at
    /// the master's cell, but never drawn, acting, or in anyone's way.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub retinue: bool,
    /// When its stay ends, on the game clock (a summon's); `None` for one that stays until it is
    /// slain (a companion's, a lasting call's).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<GameTime>,
}

/// What made an actor an ally.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AllySource {
    /// Called by a spell: the spell that called it (`None` for the dev `ally:` action's).
    Summoned {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        spell: Option<SpellId>,
    },
    /// A companion of the master's (the ranger's, #825): its data is that design's.
    Companion,
}

impl AllySource {
    /// Its id in dumps.
    pub const fn id(&self) -> &'static str {
        match self {
            AllySource::Summoned { .. } => "summoned",
            AllySource::Companion => "companion",
        }
    }
}

/// What an ally was last told ([`Ally::order`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Order {
    /// Heel: stay within `heel` cells of the master, fighting what comes.
    #[default]
    Follow,
    /// Hold the cell it stood on when told, fighting only what comes beside it.
    Stay { at: Pos },
    /// Go to that cell and fight what is near it, then follow.
    Hunt { at: Pos },
}

impl Order {
    /// Its id in dumps: `follow`, `stay`, `hunt`.
    pub const fn id(self) -> &'static str {
        match self {
            Order::Follow => "follow",
            Order::Stay { .. } => "stay",
            Order::Hunt { .. } => "hunt",
        }
    }

    /// The cell it names, if any.
    pub const fn at(self) -> Option<Pos> {
        match self {
            Order::Follow => None,
            Order::Stay { at } | Order::Hunt { at } => Some(at),
        }
    }
}

impl Actor {
    /// Whether it travels in its master's retinue (the module docs): off every level's play.
    pub fn in_retinue(&self) -> bool {
        self.ally.as_ref().is_some_and(|a| a.retinue)
    }

    /// Whose ally it is, if anyone's.
    pub fn master(&self) -> Option<ActorId> {
        self.ally.as_ref().map(|a| a.master)
    }
}

impl World {
    /// The allies of `master`, on any level and in the retinue, in actor id order.
    pub fn allies_of(&self, master: ActorId) -> Vec<ActorId> {
        (self.state.actors.values())
            .filter(|a| a.master() == Some(master))
            .map(|a| a.id)
            .collect()
    }

    /// The allies of `master` on the current level and acting (not the retinue), in id order.
    pub(super) fn allies_here(&self, master: ActorId) -> Vec<ActorId> {
        let level = &self.state.current_level;
        (self.state.actors.values())
            .filter(|a| a.master() == Some(master) && a.level == *level && !a.in_retinue())
            .map(|a| a.id)
            .collect()
    }

    /// Whose ally actor `id` is, if anyone's.
    pub(super) fn master_of(&self, id: ActorId) -> Option<ActorId> {
        self.state.actors.get(&id).and_then(Actor::master)
    }

    /// Makes an ally of type `creature` for the player at `at` on the current level, following,
    /// with no gear (a beast carries nothing), on the player's side; `until` is when its stay
    /// ends, if it does. Returns its id. The caller saw to the cell and the cap.
    pub(super) fn make_ally(
        &mut self,
        creature: CreatureId,
        at: Pos,
        source: AllySource,
        until: Option<GameTime>,
    ) -> ActorId {
        let content = self.content.clone();
        let player = self.state.player;
        let spawn = CreatureSpawn {
            creature,
            faction: self.state.player().faction(&content),
            alignment: None,
            npc: None,
            pos: at,
            route: Vec::new(),
            carry: Vec::new(),
            equipment: Default::default(),
            rolls_gear: false,
            asleep: false,
            status: Vec::new(),
        };
        let level = self.state.current_level.clone();
        let id = super::travel::make_creature(&mut self.state, &content, &level, &spawn);
        if let Some(actor) = self.state.actors.get_mut(&id) {
            actor.ally = Some(Ally {
                master: player,
                source,
                order: Order::Follow,
                retinue: false,
                until,
            });
        }
        // A rest perceives after the turn something appeared.
        self.spawned += 1;
        id
    }

    /// Whether the player may lead one more ally on the current level: fewer than the rules'
    /// `most` there now.
    pub(super) fn may_lead_more(&self) -> bool {
        let most = self.content.allies().map_or(0, |r| usize::from(r.most));
        self.allies_here(self.state.player).len() < most
    }

    /// The dev `ally:CREATURE[:N][:DURATION]`: `count` allies of type `creature` beside the
    /// player, following, for `lasts` if given, no time. Refused without `data/allies.toml`,
    /// for an unknown type or the player's, at the cap (`[allies] most`), and without a free
    /// cell beside the player.
    pub(super) fn ally_dev(&mut self, creature: &str, count: u8, lasts: Option<Duration>) -> Spent {
        if self.content.allies().is_none() {
            self.say("This world knows no allies.");
            return Spent::Nothing;
        }
        let Some(kind) = (self.content.creature_id(creature))
            .filter(|&c| Some(c) != self.content.creature_id("player"))
        else {
            self.say(format!("There is no creature `{creature}`."));
            return Spent::Nothing;
        };
        let player = self.state.player;
        // On the overworld, which has no creatures, they join the retinue at once.
        let overworld = self.state.current().kind == LevelKind::Overworld;
        let cells = if overworld {
            vec![self.player_pos(); usize::from(count)]
        } else {
            self.summon_cells(player, DEV_SPREAD)
        };
        if cells.is_empty() {
            self.say("There is no room for them beside you.");
            return Spent::Nothing;
        }
        let until = lasts.map(|d| self.state.time + d);
        let mut made = 0;
        for &at in cells.iter().take(usize::from(count)) {
            if !overworld && !self.may_lead_more() {
                break;
            }
            let id = self.make_ally(kind, at, AllySource::Summoned { spell: None }, until);
            if overworld
                && let Some(ally) = (self.state.actors.get_mut(&id)).and_then(|a| a.ally.as_mut())
            {
                ally.retinue = true;
            }
            made += 1;
        }
        if made == 0 {
            self.say("You cannot lead more than this.");
            return Spent::Nothing;
        }
        let def = self.content.creature(kind);
        let what = few(&def.name, &def.plural, made);
        let verb = if made == 1 { "comes" } else { "come" };
        self.say(format!("{} {verb} to your side.", capital(&what)));
        Spent::NoTime
    }

    /// The Beast-bond and its kin: what the master's talents do to their allies' blows and hide
    /// (`TalentMods::ally`), for actor `id` if it is an ally; nothing for anyone else.
    fn ally_mods(&self, id: ActorId) -> Option<crate::talent::AllyMods> {
        let master = self.master_of(id)?;
        let master = self.state.actors.get(&master)?;
        if master.talents.is_empty() {
            return None;
        }
        Some(master.talent_mods(&self.content).ally)
    }

    /// `profile` with what an ally's master gives its blows (Beast-bond's to-hit); as it was for
    /// anyone else. One of `fight.rs::resolve`'s hooks.
    pub(super) fn ally_blow(&self, attacker: ActorId, mut profile: Profile) -> Profile {
        if let Some(mods) = self.ally_mods(attacker) {
            profile.to_hit += mods.to_hit;
        }
        profile
    }

    /// The Protection an ally's master gives it (Beast-bond's); 0 for anyone else. Read by
    /// `World::protection_of`.
    pub(super) fn ally_protection(&self, id: ActorId) -> i32 {
        self.ally_mods(id).map_or(0, |m| m.protection)
    }

    /// The one arm of `ai.rs::decide` for an ally (the module docs): what ally `id` does now.
    pub(super) fn ally_plan(&mut self, id: ActorId) -> Plan {
        let Some((master, order, pos)) = (self.state.actors.get(&id))
            .and_then(|a| a.ally.as_ref().map(|ally| (ally.master, ally.order, a.pos)))
        else {
            return Plan::Wait;
        };
        let Some(rules) = self.content.allies().cloned() else {
            return Plan::Wait;
        };
        let Some(home) = (self.state.actors.get(&master))
            .filter(|m| m.level == self.state.current_level)
            .map(|m| m.pos)
        else {
            return Plan::Wait;
        };
        // 1. A target: the nearest hostile of its master's it sees, within its order's reach.
        let hostile = move |state: &WorldState, content: &Content, _me: &Actor, a: &Actor| {
            let master = &state.actors[&master];
            stance_between(state, content, master, a) == Stance::Hostile
        };
        let near = |at: Pos, reach: i32| {
            move |s: &WorldState, c: &Content, me: &Actor, a: &Actor| {
                a.pos.chebyshev(at) <= reach && hostile(s, c, me, a)
            }
        };
        let leashed = order == Order::Follow && pos.chebyshev(home) > rules.leash;
        let target = match order {
            // 4. On the leash: drawn too far from its master, it breaks off and heels.
            Order::Follow if leashed => None,
            Order::Follow => self.nearest_in_sight(id, &hostile),
            Order::Stay { at } => self.nearest_in_sight(id, &near(at, rules.stay_reach)),
            Order::Hunt { at } => (self.nearest_in_sight(id, &near(at, rules.hunt_reach)))
                .or_else(|| self.nearest_in_sight(id, &hostile)),
        };
        // 2. Fight it as a creature would.
        if let Some((target, at)) = target {
            return self.fight(id, target, at);
        }
        // 3. Its order.
        match order {
            Order::Follow => {
                let heel = rules.heel;
                match self.route(id, home, heel, |q| q.chebyshev(home) <= heel) {
                    Route::Step(dir) => Plan::Step(dir),
                    Route::There | Route::Wait | Route::Unreachable => Plan::Wait,
                }
            }
            Order::Stay { at } => match self.route(id, at, 0, |q| q == at) {
                Route::Step(dir) => Plan::Step(dir),
                Route::There | Route::Wait | Route::Unreachable => Plan::Wait,
            },
            Order::Hunt { at } => match self.route(id, at, 0, |q| q == at) {
                Route::Step(dir) => Plan::Step(dir),
                Route::Wait => Plan::Wait,
                // Arrived with nothing to fight (or no way there): it looks back to its master.
                Route::There | Route::Unreachable => {
                    self.set_order(id, Order::Follow);
                    if self.perception.sees_actors_at(pos) {
                        let name = self
                            .content
                            .creature(self.state.actors[&id].creature)
                            .name
                            .clone();
                        self.say(format!("Your {name} looks back at you."));
                    }
                    Plan::Wait
                }
            },
        }
    }

    fn set_order(&mut self, id: ActorId, order: Order) {
        if let Some(ally) = (self.state.actors.get_mut(&id)).and_then(|a| a.ally.as_mut()) {
            ally.order = order;
        }
    }

    /// The player gives `command` to every ally of theirs on the level (the module docs): no
    /// time, a voice's noise, a Handle animal mark. Refused, with no time and no noise, with no
    /// ally on the level, and Set on at a cell the player does not see.
    pub(super) fn order(&mut self, command: Command, events: &mut Vec<Event>) -> Spent {
        let player = self.state.player;
        let allies = self.allies_here(player);
        if allies.is_empty() {
            self.say("You have no one to call.");
            return Spent::Nothing;
        }
        if let Command::SetOn(at) = command
            && !self.perception.is_visible(at)
        {
            self.say("You can set them on only a place you see.");
            return Spent::Nothing;
        }
        let pack = self.pack_words(&allies);
        let one = allies.len() == 1;
        for &id in &allies {
            let order = match command {
                Command::Heel => Order::Follow,
                Command::Hold => Order::Stay {
                    at: self.state.actors[&id].pos,
                },
                Command::SetOn(at) => Order::Hunt { at },
            };
            self.set_order(id, order);
        }
        let line = match command {
            Command::Heel if one => format!("Your {pack} comes to heel."),
            Command::Heel => format!("Your {pack} come to heel."),
            Command::Hold if one => format!("Your {pack} holds."),
            Command::Hold => format!("Your {pack} hold."),
            Command::SetOn(at) => {
                let target = (self.state.actor_at(&self.state.current_level, at))
                    .filter(|a| a.id != player && self.perception.sees_actors_at(at))
                    .map(|a| the_start(&self.content.creature(a.creature).name).to_lowercase());
                let on = target.as_deref().unwrap_or("that spot");
                format!("You set your {pack} on {on}.")
            }
        };
        self.make_noise(player, Act::Order);
        events.push(Event::Ordered {
            at: self.player_pos(),
        });
        self.say(line);
        if self.trains(Use::Handle) {
            self.train(Use::Handle);
        }
        Spent::NoTime
    }

    /// The allies `ids` as the player's words name them, after "your": "wolf", "wolves",
    /// "wolves and bear" (by type, in the order the ids run; a possessive takes no count).
    fn pack_words(&self, ids: &[ActorId]) -> String {
        let mut kinds: Vec<(CreatureId, u32)> = Vec::new();
        for id in ids {
            let Some(a) = self.state.actors.get(id) else {
                continue;
            };
            match kinds.iter_mut().find(|(c, _)| *c == a.creature) {
                Some((_, n)) => *n += 1,
                None => kinds.push((a.creature, 1)),
            }
        }
        let words: Vec<String> = (kinds.iter())
            .map(|&(c, n)| {
                let def = self.content.creature(c);
                if n == 1 {
                    def.name.clone()
                } else {
                    def.plural.clone()
                }
            })
            .collect();
        and_list(&words.iter().map(String::as_str).collect::<Vec<_>>())
    }

    /// The player left `from` at `old` for `to`, arriving at `arrival` (the module docs): their
    /// allies within `comes_along` of `old`, not held fast, come along (into the retinue, when
    /// `to` is the overworld), and the retinue steps out around them on any other level; the
    /// rest stay. A summon whose time ran out on the way is removed silently, and so is one
    /// left on `to` earlier whose time has passed. Called by `World::travel` once the player
    /// stands on `to`.
    pub(super) fn follow_along(&mut self, from: &str, to: &str, old: Pos, arrival: Pos) {
        let Some(rules) = self.content.allies().cloned() else {
            return;
        };
        let player = self.state.player;
        let now = self.state.time;
        let overworld = (self.state.levels.get(to)).is_some_and(|l| l.kind == LevelKind::Overworld);
        let from_overworld =
            (self.state.levels.get(from)).is_some_and(|l| l.kind == LevelKind::Overworld);
        let content = self.content.clone();
        // Those who come: the retinue, and the allies beside the player as they left.
        let mut coming: Vec<ActorId> = Vec::new();
        let mut left: Vec<ActorId> = Vec::new();
        for a in self.state.actors.values() {
            if a.master() != Some(player) {
                continue;
            }
            if a.in_retinue() {
                coming.push(a.id);
            } else if a.level == from {
                let held = a.effects.blocking(&content, ActionKind::Step).is_some();
                if a.pos.chebyshev(old) <= rules.comes_along && !held {
                    coming.push(a.id);
                } else {
                    left.push(a.id);
                }
            }
        }
        // Left behind, and seen: said before the arrival's own lines.
        let seen: Vec<ActorId> = (left.iter().copied())
            .filter(|id| self.perception.sees_actors_at(self.state.actors[id].pos))
            .collect();
        if !seen.is_empty() {
            let words = self.pack_words(&seen);
            let verb = if seen.len() == 1 { "is" } else { "are" };
            self.say(format!("Your {words} {verb} left behind."));
        }
        let mut came: Vec<ActorId> = Vec::new();
        for id in coming {
            let lapsed = (self.state.actors[&id].ally.as_ref())
                .and_then(|a| a.until)
                .is_some_and(|until| until <= now);
            if lapsed {
                self.state.actors.remove(&id);
                continue;
            }
            let pos = if overworld {
                arrival
            } else {
                self.free_cell_near(to, arrival)
            };
            if let Some(a) = self.state.actors.get_mut(&id) {
                a.level = to.to_string();
                a.pos = pos;
                a.heard.clear();
                a.mind = Mind::default();
                if let Some(ally) = a.ally.as_mut() {
                    ally.retinue = overworld;
                    ally.order = Order::Follow;
                }
            }
            came.push(id);
        }
        if !came.is_empty() {
            let words = self.pack_words(&came);
            let one = came.len() == 1;
            let line = match (from_overworld, one) {
                (true, true) => format!("Your {words} follows you in."),
                (true, false) => format!("Your {words} follow you in."),
                (false, true) => format!("Your {words} comes with you."),
                (false, false) => format!("Your {words} come with you."),
            };
            self.say(line);
        }
        // Summons left here earlier whose time has passed are gone.
        let gone: Vec<ActorId> = (self.state.actors.values())
            .filter(|a| a.level == to && a.master() == Some(player) && !a.in_retinue())
            .filter(|a| {
                (a.ally.as_ref())
                    .and_then(|x| x.until)
                    .is_some_and(|u| u <= now)
            })
            .map(|a| a.id)
            .collect();
        for id in gone {
            self.state.actors.remove(&id);
        }
    }

    /// The end of a turn (`run_turn`): every ally on the current level whose time has passed
    /// departs, said if the player sees it ("The wolf fades away."). Nothing is left behind.
    pub(super) fn allies_lapse(&mut self, events: &mut Vec<Event>) {
        let now = self.state.time;
        let level = self.state.current_level.clone();
        let lapsed: Vec<ActorId> = (self.state.actors.values())
            .filter(|a| a.level == level && !a.in_retinue())
            .filter(|a| {
                (a.ally.as_ref())
                    .and_then(|x| x.until)
                    .is_some_and(|u| u <= now)
            })
            .map(|a| a.id)
            .collect();
        for id in lapsed {
            self.ally_departs(id, events);
        }
    }

    /// Ally `id` departs: removed from the world with nothing left behind (no corpse, no pile),
    /// said when the player sees its cell.
    pub(super) fn ally_departs(&mut self, id: ActorId, events: &mut Vec<Event>) {
        let Some(actor) = self.state.actors.remove(&id) else {
            return;
        };
        events.push(Event::Departed {
            actor: id,
            at: actor.pos,
        });
        if self.perception.sees_actors_at(actor.pos) {
            let def = self.content.creature(actor.creature);
            let words = def.depart.as_deref().unwrap_or("fades away");
            self.say(format!("{} {words}.", the_start(&def.name)));
        }
        self.fallen.push(actor);
    }
}

/// `count` of a thing in words, as the player is told it: "a wolf", "two wolves", "thirteen
/// wolves" (the counts of a pack are small; a bigger one is a numeral).
pub(super) fn few(name: &str, plural: &str, count: u32) -> String {
    const WORDS: [&str; 13] = [
        "no", "a", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
        "eleven", "twelve",
    ];
    match count {
        1 => counted(name, plural, 1),
        n => match WORDS.get(n as usize) {
            Some(word) => format!("{word} {plural}"),
            None => format!("{n} {plural}"),
        },
    }
}

/// `text` with its first letter upper-case.
pub(super) fn capital(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
