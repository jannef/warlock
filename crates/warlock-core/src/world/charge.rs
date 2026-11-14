//! The warrior's Charge (spec §8 "Class abilities"; #1466): a class active aimed at a hostile
//! creature the player sees ([`crate::action::Action::UseAt`], `use:charge:12,5`). The player
//! rushes along the straight line to it ([`Pos::line_between`], the line a missile flies) and
//! ends in the last cell before it, then strikes it once, as a step into it would
//! ([`World::player_attacks`]: the off hand's blow, a cleave, a riposte follow as ever).
//!
//! - **The way**: every cell between must be open ground, decided before anything moves:
//!   walkable terrain that is no deep water, no ice, no trap the player knows of, and nobody the
//!   player sees. Anything else refuses the charge, taking no time and spending nothing, and so
//!   does a target beside them already (no cell to gain), one they do not see, one not hostile,
//!   the overworld, swimming, or a load too heavy to move under, and so does a way out of a
//!   shop's door while the player owes it (the keeper steps into the way, as on a step).
//! - **Unseen in the way**: someone the player does not see on the line stops the charge short
//!   in the cell before them ("You run into something!"); the cells moved are paid, and no blow
//!   is struck. A trap unknown to them goes off as on any step, and the charge stops there too.
//! - **No free blows**: the steps are no ordinary steps. They never call
//!   [`World::step_away`] or [`World::step_up`], so nobody passed or left gets an opportunist's
//!   blow, a gatekeeper's, a brace, or a point-blank shot.
//! - **Cost**: its talent's `resource` (1 stamina) and `per_cell` more for every cell moved,
//!   all spent as the charge begins; refused while the player holds less than the whole
//!   distance costs. **Time**: the steps' (each priced as the step onto its terrain is); the
//!   blow comes free, with its exertion.
//!
//! - **Whirling charge** (the warrior's level-50 talent, `whirling_charge`): after each step,
//!   every hostile beside the cell just entered takes a free blow ([`Free::Cleave`]'s kind: no
//!   time, no stamina, no exertion, the mark untouched), clockwise from the north, each at most
//!   once a charge; the one charged at is never struck on the way, only by the charge's own
//!   blow at its end. One that falls does not stop the charge.
//!
//! Nothing is drawn deciding; the trap and the blows draw as theirs always do.

use super::free_blow::{Free, FreeBlow};
use super::{ActorId, Spent, World};
use crate::action::Event;
use crate::content::{ActionKind, ClassActive, LevelKind, Stance};
use crate::geom::{Dir, Pos};

impl World {
    /// Charge at `at` (the module docs); `use_class_at` has checked the ability.
    pub(super) fn charge(&mut self, at: Pos, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some(per_cell) = self.state.player().talent_mods(&content).charge else {
            return Spent::Nothing;
        };
        let Some((target, path)) = self.charge_path(at) else {
            return Spent::Nothing;
        };
        let base = (content.class_active(ClassActive::Charge)).and_then(|a| a.resource);
        let cells = u16::try_from(path.len()).unwrap_or(u16::MAX);
        let price = base.map(|b| b.amount.saturating_add(per_cell.saturating_mul(cells)));
        if let (Some(cost), Some(amount)) = (base, price) {
            let held = (self.state.player().resources.get(cost.resource)).map_or(0, |p| p.value);
            if held < amount {
                self.say("You are short of breath for so long a charge.");
                return Spent::Nothing;
            }
            self.spend_resource(player, cost.resource, amount, events);
        }
        self.say("You charge!");
        let whirling = self.state.player().talent_mods(&content).whirling_charge;
        let mut struck = Vec::new();
        let mut time = 0;
        let mut from = self.player_pos();
        for to in path {
            let level = &self.state.current().id;
            if self.state.actor_at(level, to).is_some() {
                self.say("You run into something!");
                return Spent::Turn(time.max(self.price_of(player, ActionKind::Step)));
            }
            let terrain = self.state.current().terrain[to];
            time += self.step_price(player, content.terrain(terrain), false);
            if let Some(actor) = self.state.actors.get_mut(&player) {
                actor.pos = to;
            }
            self.let_go(player, None);
            events.push(Event::ActorMoved {
                actor: player,
                from,
                to,
            });
            self.make_step_noise(player, to, false);
            self.shop_step(from, to);
            let climb = self.step_onto_trap(to, player, events);
            self.collapse_step(to, events);
            time += climb;
            let sprung = (self.state.current().traps.get(&to)).is_some_and(|t| t.known);
            if self.state.death.is_some() || self.player_pos() != to || climb > 0 || sprung {
                return Spent::Turn(time);
            }
            if whirling {
                self.whirl(target, &mut struck, events);
                if self.state.death.is_some() {
                    return Spent::Turn(time);
                }
            }
            from = to;
        }
        if self.state.actors.contains_key(&target) && self.state.death.is_none() {
            self.player_attacks(target, true, events);
            self.exert(ActionKind::Melee);
        }
        Spent::Turn(time)
    }

    /// Whirling charge (the module docs): a free blow at every hostile beside the player but
    /// `target` and those in `struck`, clockwise from the north; each struck joins `struck`.
    fn whirl(&mut self, target: ActorId, struck: &mut Vec<ActorId>, events: &mut Vec<Event>) {
        let player = self.state.player;
        let here = self.player_pos();
        let level = self.state.current().id.clone();
        for dir in Dir::ALL {
            if self.state.death.is_some() {
                return;
            }
            let Some(foe) = (self.state.actor_at(&level, here.step(dir))).map(|a| a.id) else {
                continue;
            };
            if foe == target || foe == player || struck.contains(&foe) {
                continue;
            }
            if self.stance(player, foe) != Stance::Hostile {
                continue;
            }
            struck.push(foe);
            let profile = self.state.player().melee(&self.content);
            let blow = FreeBlow {
                by: player,
                at: foe,
                kind: Free::Cleave,
                with: None,
                profile,
            };
            self.strike_free(blow, events);
        }
    }

    /// The hostile the player charges at `at` and the cells they cross to reach it, if the
    /// charge may be made (the module docs); `None` (said) when not.
    fn charge_path(&mut self, at: Pos) -> Option<(ActorId, Vec<Pos>)> {
        let content = self.content.clone();
        let player = self.state.player;
        if self.state.current().kind == LevelKind::Overworld {
            self.say("You cannot charge on the overworld.");
            return None;
        }
        if self.swimming() {
            self.say("You cannot charge while swimming.");
            return None;
        }
        if !self.can_move() {
            return None;
        }
        let here = self.player_pos();
        let level = self.state.current();
        let target = (self.state.actor_at(&level.id, at))
            .filter(|a| a.id != player)
            .filter(|a| self.perception.sees_actors_at(at) && !a.effects.hides(&content))
            .map(|a| a.id);
        let Some(target) = target else {
            self.say("There is nobody there you see to charge.");
            return None;
        };
        if self.stance(player, target) != Stance::Hostile {
            self.say("You have no quarrel there.");
            return None;
        }
        let path: Vec<Pos> = here.line_between(at).collect();
        if path.is_empty() {
            self.say("You are beside it already.");
            return None;
        }
        let blocked = path.iter().any(|&cell| {
            let open = level.terrain.get(cell).is_some_and(|&t| {
                let def = content.terrain(t);
                def.walkable && !def.swim
            });
            let trap = level.traps.get(&cell).is_some_and(|t| t.known);
            let seen = (self.state.actor_at(&level.id, cell))
                .is_some_and(|_| self.perception.sees_actors_at(cell));
            !open || level.ice_at(cell).is_some() || trap || seen
        });
        if blocked {
            self.say("Something stands in the way of a charge.");
            return None;
        }
        // Out of a shop's door owing: the keeper would turn a step back there (`shop.rs`).
        let mut steps = std::iter::once(here)
            .chain(path.iter().copied())
            .zip(path.iter().copied());
        if let Some(shop) = steps.find_map(|(from, to)| self.owing_exit(from, to)) {
            let text = format!("{} steps into your way: you haven't paid.", shop.name);
            self.say(text);
            return None;
        }
        Some((target, path))
    }
}
