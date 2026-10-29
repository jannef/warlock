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
//! - **Struggling** ([`World::struggle`]): the held player's step (not an attack: those go
//!   ahead) is an attempt to break free, a check of the hold's `check` stat (Strength: the
//!   effective modifier, what talents add to its checks, and what Knowledge gives against the
//!   holder, `knowledge.rs`) against the holder's grip, its
//!   attack's `difficulty` (its `on_hit` entry), else the hold's own. Made, the hold ends and the
//!   step goes ahead; failed, the turn is spent, a step's price ("You struggle, but the ghast
//!   holds you fast."). Two draws of the gameplay stream. Creatures never struggle: one held
//!   fights its holder (`World::held_back`, `lucky.rs`).
//!
//! Drowning grip (`lucky.rs`) is a hold too: the player's hit holds a creature, and a miss,
//! another target, or a step lets go, as before; the rules here end it too.

use super::{ActorId, Spent, World};
use crate::action::{Action, Event};
use crate::check::check;
use crate::content::{ActionKind, Affliction, StatusId};
use crate::status::Effect;

impl World {
    /// The degrees a dose of hold `status` lasts: its `turns` drawn (within 1 and its `max`),
    /// or `severity` for a status that is no hold.
    pub(super) fn hold_turns(&mut self, status: StatusId, severity: u16) -> u16 {
        let def = self.content.status(status);
        let Some(hold) = def.hold else {
            return severity;
        };
        let turns = hold.turns.roll(&mut self.state.rng);
        u16::try_from(turns).unwrap_or(1).clamp(1, def.max)
    }

    /// Ends every hold on the current level whose holder no longer holds (module docs).
    pub(super) fn loosen_holds(&mut self, events: &mut Vec<Event>) {
        let level = &self.state.current_level;
        let mut loose: Vec<(ActorId, Effect)> = Vec::new();
        for actor in self.state.actors.values().filter(|a| a.level == *level) {
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

    /// The player's `action`, if it is a step while a hold blocks their steps, is a struggle
    /// (module docs): failed, the turn it cost; made, the hold ends and `None` lets the step go
    /// ahead. `None` too for any other action.
    pub(super) fn struggle(&mut self, action: &Action) -> Option<Spent> {
        let content = self.content.clone();
        let player = self.state.player;
        if self.kind_of(action) != Some(ActionKind::Step) {
            return None;
        }
        let me = self.state.player();
        let status = me.effects.blocking(&content, ActionKind::Step)?;
        let hold = content.status(status).hold?;
        let effect = *me.effects.get(status)?;
        let holder = effect.by.and_then(|by| self.state.actors.get(&by));
        let difficulty = holder
            .and_then(|h| content.creature(h.creature).attack.as_ref())
            .and_then(|a| {
                a.on_hit.iter().find_map(|o| match o.what {
                    Affliction::Hold {
                        status: s,
                        difficulty,
                    } if s == status => difficulty,
                    _ => None,
                })
            })
            .unwrap_or(hold.difficulty);
        let name = holder
            .filter(|h| self.perception.sees_actors_at(h.pos))
            .map(|h| self.called(h));
        let talents = me.talent_mods(&content);
        let modifier = (me.effective_stats(&content).modifier(hold.check))
            .saturating_add(talents.check(hold.check));
        // Know your enemy (Knowledge, `knowledge.rs`): the holder's.
        let modifier = modifier.saturating_add(self.resisting(player, effect.by));
        let free = check(&mut self.state.rng, modifier, difficulty);
        self.train_stat(crate::content::StatUse::Struggled);
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
        Some(Spent::Turn(self.price_of(player, ActionKind::Step)))
    }
}
