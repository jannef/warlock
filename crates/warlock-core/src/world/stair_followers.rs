//! Stair followers (spec §8 "Creature behavior", #985): stairs are no free escape. A creature
//! beside the player as they take a staircase, hostile to them and aware of them, comes along
//! and arrives beside them, by the way allies cross levels (`travel.rs::cross_to`). Only
//! `World::take_stairs` asks: a fall, an edge, and the overworld's ways bring no one.
//!
//! **Who** ([`World::stair_followers`], asked before the player leaves: awareness reads the
//! level's light and walls): a creature at Chebyshev distance 1, hostile to the player and not
//! befriended ([`World::stance`], `beast_friend.rs`), not unaware of them (`surprise.rs`: asleep,
//! disguised, slipped past, or not seeing them). Never one the rules keep still or out of action,
//! read from its statuses' data: one that `sleeps` (Asleep), `hides` (Disguised), `blocks` steps
//! (Held, Webbed, Pinned), is `flat_footed` (Stunned, Surprised), or `flees` (Frightened). Never
//! one of a type that keeps its place: `budge` other than `always` (`off_post`, a shopkeeper,
//! a guildmaster; `never`, rooted), a mimic, even found out, or a lurker no one has hurt
//! (`lurk.rs`). Never an NPC, a shop's keeper, an ally, or a layer boss, which keeps to its
//! lair (`layer_boss.rs`). Slowed creatures do come (Transfixed,
//! Staggered), paying their slowed price for the stairs.
//!
//! **Where** ([`World::followers_arrive`]): in id order, each on the first free walkable cell of
//! the ring around the player's arrival ([`World::free_cell_within`], reach 1), not on ground a
//! hostile shuns (a sanctuary, hallowed ground). No cell left: it stays where it was, frozen on
//! the old level. Allies have stepped out in `World::travel` by then, so they never lose their
//! place to a follower and are never dropped; the follower is the one left behind. Not onto the
//! overworld, which has no creatures: it stays behind.
//!
//! **What it keeps**: hit points, statuses, gear, and its awareness. It hunts on: its
//! `last_seen` is the player's arrival; what it heard, its noise and post (cells of the old
//! level) are gone, and so is its patrol route (it walks no beat here, nor back home).
//! [`super::Actor::home`] remembers the level it first left: its death counts for a kill quest
//! there (`fight.rs`), so a quest's quarry that follows the player out of its place still counts
//! (a fixed band, as the arena's rats, could otherwise fall short; a unique type's goal is met
//! by `slain` wherever it dies).
//!
//! **Timing**: the follower pays the stairs' price from its own energy, as the player does, so
//! it gets no blow out of their climb: at equal speed the player acts first on arrival, and a
//! fast creature comes at its own speed. No draws at all, so replays stay deterministic.

use super::ally::{capital, few};
use super::{ActorId, ActorKind, Mind, World, beast_friend};
use crate::content::{ActionKind, Budge, CreatureId, LevelKind, Stance, and_list, the};

impl World {
    /// The creatures that follow the player if they take a staircase now (module docs), in id
    /// order. Asked on the level they are leaving.
    pub(super) fn stair_followers(&mut self) -> Vec<ActorId> {
        let (player, here) = (self.state.player, self.player_pos());
        let level = self.state.current_level.clone();
        let beside: Vec<ActorId> = (self.state.actors.values())
            .filter(|a| a.id != player && a.level == level && a.pos.chebyshev(here) == 1)
            .map(|a| a.id)
            .collect();
        (beside.into_iter())
            .filter(|&id| self.may_follow(id) && !self.unaware(id, player))
            .collect()
    }

    /// Whether creature `id` is of those who may follow (module docs), awareness aside.
    fn may_follow(&self, id: ActorId) -> bool {
        let (state, content) = (&self.state, &self.content);
        let me = &state.actors[&id];
        let def = content.creature(me.creature);
        let still = me.effects.iter().any(|e| {
            let status = content.status(e.status);
            status.sleeps || status.hides || status.flees || status.flat_footed
        }) || me.effects.blocking(content, ActionKind::Step).is_some();
        let keeps_place = def.budge != Budge::Always
            || def.mimic.is_some()
            || (def.lurks && me.hp >= me.max_hp(content))
            || me.npc.is_some()
            || content.is_layer_boss(me.creature)
            || (state.current().shops.iter()).any(|shop| shop.keeper == id);
        me.kind == ActorKind::Creature
            && me.ally.is_none()
            && !still
            && !keeps_place
            && self.stance(state.player, id) == Stance::Hostile
            && !beast_friend::befriended(state, content, me, state.player())
    }

    /// `followers` (asked as the player left level `from`) arrive beside them on the current
    /// level, as far as there is room (module docs), and the player is told of those they saw:
    /// "The kobold follows you down the stairs." `way` is "down" or "up".
    pub(super) fn followers_arrive(&mut self, followers: &[ActorId], from: &str, way: &str) {
        if self.state.current().kind == LevelKind::Overworld {
            return;
        }
        let to = self.state.current_level.clone();
        let arrival = self.player_pos();
        let mut seen: Vec<ActorId> = Vec::new();
        for &id in followers {
            let Some(pos) = self.follower_cell(arrival) else {
                break;
            };
            let price = self.price_of(id, ActionKind::Stairs);
            self.cross_to(id, &to, pos);
            if let Some(a) = self.state.actors.get_mut(&id) {
                a.energy = a.energy.saturating_sub(price);
                a.patrol = None;
                a.mind = Mind {
                    last_seen: Some(arrival),
                    noise: None,
                    post: None,
                    ..a.mind
                };
                a.home.get_or_insert_with(|| from.to_string());
            }
            if self.perception.shows(id) {
                seen.push(id);
            }
        }
        if let Some(line) = self.followers_line(&seen, way) {
            self.say(line);
        }
    }

    /// Where the next follower stands: the first free cell beside `arrival` on the current
    /// level that a creature hostile to the player does not shun (`ai.rs::shuns`, for one
    /// coming in from outside).
    fn follower_cell(&self, arrival: crate::geom::Pos) -> Option<crate::geom::Pos> {
        let level = self.state.current();
        let now = self.state.time;
        self.free_cell_within(&level.id, arrival, 1, |p| {
            let shunned = (level.room_at(p))
                .is_some_and(|room| self.content.room_effect(room.effect).shunned);
            !shunned && !level.shuts_out(None, p, now)
        })
    }

    /// The line for followers `ids`, by type in the order they first come: "The kobold follows
    /// you down the stairs.", "Two kobolds and the rat follow you up the stairs."
    fn followers_line(&self, ids: &[ActorId], way: &str) -> Option<String> {
        let verb = match ids {
            [] => return None,
            [_] => "follows",
            _ => "follow",
        };
        let mut kinds: Vec<(CreatureId, u32)> = Vec::new();
        for id in ids {
            let creature = self.state.actors[id].creature;
            match kinds.iter_mut().find(|(c, _)| *c == creature) {
                Some((_, n)) => *n += 1,
                None => kinds.push((creature, 1)),
            }
        }
        let words: Vec<String> = (kinds.iter())
            .map(|&(c, n)| {
                let def = self.content.creature(c);
                match n {
                    1 => the(&def.name),
                    n => few(&def.name, &def.plural, n),
                }
            })
            .collect();
        let who = and_list(&words.iter().map(String::as_str).collect::<Vec<_>>());
        Some(format!("{} {verb} you {way} the stairs.", capital(&who)))
    }
}
