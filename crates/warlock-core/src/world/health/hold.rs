//! Holds (spec §8 "Status effects"; #431): a status with a `hold` ([`crate::content::Hold`],
//! Held) is one actor holding another fast. Its effect's `by` is the holder (saved with the
//! effect, as every cause is).
//!
//! - **Turns**: a dose's degrees are drawn from the hold's `turns` dice ([`World::afflict`]), and
//!   the status eases a degree a turn, so a hold lasts a few turns at most; a new dose keeps the
//!   longer (`stacking = "highest"`). The safety net: nothing holds for ever.
//! - **Letting go** ([`World::loosen_holds`]): a hold ends, as if worn off ([`World::ended`]:
//!   "You are free to move."), when its holder is dead, on another level, not beside the held
//!   one, or itself suffers a status that blocks its steps (held) or slows its actions (a `cost`
//!   above 100: staggered, transfixed, sluggish). Asked after the player's action, after every
//!   creature's turn, and as the turn ends (`run_turn`). A hold no one caused (the dev
//!   `afflict:held`) has no holder to lose: only its turns and a struggle end it.
//! - **Struggling** ([`World::struggle`], asked before [`World::refuse_blocked`]): the held
//!   player's step (not an attack: those go ahead) is an attempt to break free, a check of the
//!   hold's `check` stat (Strength: the effective modifier, what talents add to its checks, and
//!   what Knowledge gives against the holder, `knowledge.rs`) against the holder's grip, its
//!   attack's `difficulty` (its `on_hit` entry), else the hold's own. Made, the hold ends and the
//!   step goes ahead; failed, the turn is spent, a step's price ("You struggle, but the ghast
//!   holds you fast."). Two draws of the gameplay stream. Creatures never struggle: one held
//!   fights its holder (`World::held_back`, `lucky.rs`).
//! - **Escape artist** (no action, no ability row: the check is part of the attempt, the owner's
//!   rule, #1015): trained, the check against a hold reads the better of its stat and Dexterity
//!   and adds [`SkillEffect::Escape`]. Bonds that are no hold carry `escape` as their difficulty in
//!   data (Webbed, Pinned) and only the trained struggle against them, with the same modifier and
//!   no Knowledge: made, the status ends with its own `end` line and the action goes ahead at its
//!   price; failed, the action's price ("You struggle, but you are still caught in a web.").
//!   Untrained, `refuse_blocked` refuses, no time, as it does a second status blocking the same
//!   action, which asks on the next attempt. A marked status that only slows (`cost` above 100;
//!   none in the data today) is checked the same way, and the action goes ahead either way: made,
//!   at its plain price; failed, slowed, no worse. Every check marks [`Use::Escape`] (the
//!   training stream) and `StatUse::Struggled`.
//!
//! - **Harm**: a hold that ticks costs its status's `damage` a tick, or the holder's attack's
//!   own `damage` (its `on_hit` entry) while that holder holds ([`World::hold_harms`], read by
//!   the status pass: the rug of smothering's engulfing costs 1, the charnel heap's 2).
//!
//! Drowning grip (`lucky.rs`) is a hold too: the player's hit holds a creature, and a miss,
//! another target, or a step lets go, as before; the rules here end it too.

use crate::action::{Action, Event};
use crate::check::check;
use crate::content::{ActionKind, Affliction, Content, StatusId};
use crate::rng::draw;
use crate::skills::{Effect as SkillEffect, Use};
use crate::stats::Stat;
use crate::status::Effect;
use crate::world::Actor;
use crate::world::{ActorId, Spent, World};

impl World {
    /// The degrees a dose of hold `status` on actor `id` lasts: its `turns` drawn (within 1 and
    /// its `max`), or `severity` for a status that is no hold.
    pub(super) fn hold_turns(&mut self, id: ActorId, status: StatusId, severity: u16) -> u16 {
        let def = self.content.status(status);
        let Some(hold) = def.hold else {
            return severity;
        };
        let turns = hold
            .turns
            .roll(&mut draw!(self, "hold/turns/{}", self.state.actors.key(id)));
        u16::try_from(turns).unwrap_or(1).clamp(1, def.max)
    }

    /// Ends every hold on the current level whose holder no longer holds (module docs).
    pub(in crate::world) fn loosen_holds(&mut self, events: &mut Vec<Event>) {
        let level = &self.state.current_level;
        let mut loose: Vec<(ActorId, Effect)> = Vec::new();
        for actor in self.state.actors.on_level(level) {
            for e in actor.effects.iter() {
                let Some(holder) = e.by else {
                    continue;
                };
                if self.content.status(e.status).hold.is_some() && !self.holds(holder, actor.id) {
                    loose.push((actor.id, *e));
                }
            }
        }
        for (id, e) in loose {
            if let Some(actor) = self.state.actors.get_mut(&id) {
                actor.effects.remove(e.status);
            }
            self.ended(id, &[e], events);
        }
    }

    /// Whether `holder` can still hold `held`: alive, on its level, beside it, and neither held
    /// nor slowed by a status of its own.
    fn holds(&self, holder: ActorId, held: ActorId) -> bool {
        let (Some(h), Some(me)) = (self.state.actors.get(&holder), self.state.actors.get(&held))
        else {
            return false;
        };
        let content = &self.content;
        let bound = h.effects.blocking(content, ActionKind::Step).is_some()
            || h.effects
                .iter()
                .any(|e| content.status(e.status).cost > 100);
        h.level == me.level && h.pos.chebyshev(me.pos) == 1 && !bound
    }

    /// The player's `action`, if a status of theirs they can fix stops or slows it, is a struggle
    /// (module docs): failed, the turn it cost when it stops it, and `None` when it only slows it;
    /// made, the status ends and `None` lets the action go ahead. `None` too for any other action.
    pub(in crate::world) fn struggle(
        &mut self,
        action: &Action,
        events: &mut Vec<Event>,
    ) -> Option<Spent> {
        let kind = self.kind_of(action)?;
        let content = self.content.clone();
        let me = self.state.player();
        let trained = me.skills.trains_effect(&content, SkillEffect::Escape);
        if let Some(status) = me.effects.blocking(&content, kind) {
            if content.status(status).hold.is_some() {
                return self.struggle_hold(status, kind, trained);
            }
            let difficulty = content.status(status).escape.filter(|_| trained)?;
            let free = self.slip(status, difficulty, events);
            return (!free).then(|| Spent::Turn(self.price_of(self.state.player, kind)));
        }
        if !trained {
            return None;
        }
        let (status, difficulty) = me.effects.iter().find_map(|e| {
            let def = content.status(e.status);
            def.escape.filter(|_| def.cost > 100).map(|d| (e.status, d))
        })?;
        self.slip(status, difficulty, events);
        None
    }

    /// A struggle against hold `status`, which blocks actions of `kind`: `trained`, with what
    /// [`World::slipping`] says, else with the hold's own stat; what comes of it is the same.
    fn struggle_hold(
        &mut self,
        status: StatusId,
        kind: ActionKind,
        trained: bool,
    ) -> Option<Spent> {
        let content = self.content.clone();
        let player = self.state.player;
        let hold = content.status(status).hold?;
        let me = self.state.player();
        let effect = *me.effects.get(status)?;
        let holder = effect.by.and_then(|by| self.state.actors.get(&by));
        let difficulty = holder
            .and_then(|h| grip(&content, h, status))
            .and_then(|(difficulty, _)| difficulty)
            .unwrap_or(hold.difficulty);
        let name = holder
            .filter(|h| self.perception.sees_actors_at(h.pos))
            .map(|h| self.called(h));
        let modifier = match trained {
            true => self.slipping(hold.check),
            false => me.check_modifier(&content, hold.check),
        };
        // Know your enemy (Knowledge, `knowledge.rs`): the holder's.
        let modifier = modifier.saturating_add(self.resisting(player, effect.by));
        let free = check(
            &mut draw!(self, "hold/struggle/player"),
            modifier,
            difficulty,
        );
        self.struggled();
        if free {
            self.player_mut().effects.remove(status);
            self.say(match &name {
                Some(name) => format!("You break free of {name}."),
                None => "You break free.".to_string(),
            });
            // Said as it is: the status's own end ("You are free to move.") would repeat it.
            return None;
        }
        self.say(match &name {
            Some(name) => format!("You struggle, but {name} holds you fast."),
            None => "You struggle, but cannot break free.".to_string(),
        });
        Some(Spent::Turn(self.price_of(player, kind)))
    }

    /// A trained player's check against bonds `status` that are no hold, of [`World::slipping`]
    /// against `difficulty`. Made, it ends with its own `end` line; failed, its `seen` says so.
    /// Whether it was made.
    fn slip(&mut self, status: StatusId, difficulty: i32, events: &mut Vec<Event>) -> bool {
        let modifier = self.slipping(Stat::Strength);
        let free = check(&mut draw!(self, "hold/slip/player"), modifier, difficulty);
        self.struggled();
        if free {
            let player = self.state.player;
            if let Some(effect) = self.state.player().effects.get(status).copied() {
                self.player_mut().effects.remove(status);
                self.ended(player, &[effect], events);
            }
        } else if let Some(seen) = &self.content.clone().status(status).seen {
            self.say(format!("You struggle, but you are still {seen}."));
        }
        free
    }

    /// What a trained player adds to a check against bonds (the owner: "better of the two with
    /// the skill"): the better of `stat` and Dexterity, each with what talents add to that
    /// stat's checks, plus Escape artist.
    pub(in crate::world) fn slipping(&self, stat: Stat) -> i32 {
        let content = &self.content;
        let me = self.state.player();
        let of = |s: Stat| me.check_modifier(content, s);
        (of(stat).max(of(Stat::Dexterity)))
            .saturating_add(me.skill_effect(content, SkillEffect::Escape))
    }

    /// The harm a tick of each hold on actor `id` costs where its holder's attack names its own
    /// (`on_hit`'s `damage`): `(status, damage)`, in the order of its effects. A hold not listed
    /// costs its status's `damage`.
    pub(in crate::world) fn hold_harms(&self, id: ActorId) -> Vec<(StatusId, u16)> {
        let Some(me) = self.state.actors.get(&id) else {
            return Vec::new();
        };
        let content = &self.content;
        (me.effects.iter())
            .filter(|e| content.status(e.status).hold.is_some())
            .filter_map(|e| {
                let holder = self.state.actors.get(&e.by?)?;
                let (_, damage) = grip(content, holder, e.status)?;
                Some((e.status, damage?))
            })
            .collect()
    }

    /// The marks of a check against bonds: the stats', and Escape artist's.
    fn struggled(&mut self) {
        self.train_stat(crate::content::StatUse::Struggled);
        if self.trains(Use::Escape) {
            self.train(Use::Escape);
        }
    }
}

/// The hold of `status` that `holder`'s attack lays (its `on_hit` entry), if any: its grip's
/// difficulty and its harm a tick, each `None` where the status's own stands.
fn grip(content: &Content, holder: &Actor, status: StatusId) -> Option<(Option<i32>, Option<u16>)> {
    let attack = content.creature(holder.creature).attack.as_ref()?;
    attack.on_hit.iter().find_map(|o| match o.what {
        Affliction::Hold {
            status: s,
            difficulty,
            damage,
        } if s == status => Some((difficulty, damage)),
        _ => None,
    })
}
