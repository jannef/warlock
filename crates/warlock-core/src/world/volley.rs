//! The ranger's Volley (spec §8 "Class abilities"; #244): a class active aimed at a cell
//! ([`crate::action::Action::UseAt`], `use:volley:12,5`). The readied missile weapon shoots its
//! `shots` at the cell, one after another, for the price of one shot and one exertion; each is
//! a shot as `fire` looses it (its own flight, attack, and missile). It stops early when the
//! ammunition runs out or the player falls. Then its cooldown (Spent). Refused, taking no time,
//! as a shot is (nothing ready, out of range, on the overworld) and as a class active is.

use super::shoot::Loose;
use super::{Spent, World};
use crate::action::{AttackKind, Event};
use crate::content::{ActionKind, ClassActive};
use crate::geom::Pos;

impl World {
    /// `use:ID:X,Y`: the player uses class active `active` at level cell `at`. Refused, taking
    /// no time, without it, while it is unavailable, or for one that aims at no cell.
    pub(super) fn use_class_at(
        &mut self,
        active: ClassActive,
        at: Pos,
        events: &mut Vec<Event>,
    ) -> Spent {
        if !active.aims_cell() {
            self.say(format!("`{}` aims at no place.", active.id()));
            return Spent::Nothing;
        }
        if !self.may_use_class(active) {
            return Spent::Nothing;
        }
        match active {
            ClassActive::Volley => self.volley(at, events),
            _ => Spent::Nothing,
        }
    }

    /// Volley: the readied missile weapon's shots at `at`, then the cooldown.
    fn volley(&mut self, at: Pos, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some(volley) = self.state.player().talent_mods(&content).volley else {
            return Spent::Nothing;
        };
        if !self.can_loose_here() {
            return Spent::Nothing;
        }
        let me = self.state.player();
        let Some((launcher, _)) = me.launcher(&content) else {
            self.say("You have no missile weapon ready.");
            return Spent::Nothing;
        };
        let range = me.launcher_range(&content).unwrap_or(1);
        match me.ammunition_item(&content) {
            Some(item) if self.may_loose(item) => {}
            Some(_) => return Spent::Nothing,
            None => {
                self.say("You have nothing ready to shoot.");
                return Spent::Nothing;
            }
        }
        if !self.within_reach(at, range, "shoot") {
            return Spent::Nothing;
        }
        // Priced as the first shot begins; the rest come free.
        let price = self.shot_price(player, launcher);
        self.say("You loose a volley!");
        for _ in 0..volley.shots {
            if self.state.death.is_some() {
                break;
            }
            let me = self.state.player();
            let (Some(item), Some(profile)) = (me.ammunition_item(&content), me.ranged(&content))
            else {
                break;
            };
            let loose = Loose {
                kind: AttackKind::Ranged,
                item,
                profile,
                range,
            };
            self.player_looses(at, loose, events);
        }
        self.exert(ActionKind::Shoot);
        self.afflict(player, volley.cooldown, volley.degrees, None, events);
        Spent::Turn(price)
    }
}
