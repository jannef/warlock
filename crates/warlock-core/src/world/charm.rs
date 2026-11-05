//! Charmed allies (spec §8 "Allies"; the spell types design §7 entry 34; `docs/dev/allies.md`
//! "Charmed"): a hex with `[spell.effect] charm` (Beguile) makes the creature it falls on the
//! player's ally for a while, against its own band, and then it is itself again where it stands.
//!
//! - **The cast** ([`World::cast_charm`], the `charm` arm of `cast.rs::cast_hex`): the target is
//!   a hex's (`hex_target`: one the player sees within the spell's range, hostile to them). No
//!   check and no level limit. Refused, with no time and no mana, on a unique or an npc, an
//!   animated thing, a creature with no mind (one of the charm's `mindless` tags, or a risen
//!   berserk), one already an ally, without `allies.toml`, and at the cap of `[allies] most`.
//! - **The ally**: [`Ally`] with [`AllySource::Charmed`]`{ spell, faction }`: the spell, for its
//!   words, and the `faction_override` it had (saved: the variant is the save change), `until` now + the cast's degrees in minutes,
//!   following. Its `faction_override` becomes the player's faction, so every reader of stances
//!   takes it for the player's (its band turns on it), and it is an ally by every rule of
//!   `ally.rs`: orders, the kill credit, the friendly mark, the dumps. Its `Mind` is left as it
//!   was. It keeps its gear and leaves its corpse, as the creature it is.
//! - **The end** ([`World::charm_ends`]): when `until` passes (`allies_lapse`, through
//!   `ally_departs`), and when the player leaves its level (`ally.rs::follow_along`: it never
//!   comes along and never joins the retinue, the owner's Q12), the `faction_override` is put
//!   back and `ally` cleared: hostile again where it stands, said when the player sees it. What it
//!   summoned while charmed (`summon.rs`: a summon takes its summoner's faction) goes back with it. Leaving,
//!   it is said nothing (the player is gone). Hostile stair followers are chosen before the player
//!   leaves (`travel.rs::take_stairs`), while it is still an ally, so it is never taken along.
//!
//! Nothing here draws from any stream.

use super::ally::{Ally, AllySource, Order, capital};
use super::cast::Cast;
use super::{ActorId, World};
use crate::action::Event;
use crate::content::SpellCharm;
use crate::geom::Pos;
use crate::time::Duration;

impl World {
    /// A charm hex at `at` (the module docs). Returns whether the action took its time.
    pub(super) fn cast_charm(
        &mut self,
        cast: &Cast<'_>,
        charm: &SpellCharm,
        at: Pos,
        events: &mut Vec<Event>,
    ) -> bool {
        let Some(rules) = self.content.allies().cloned() else {
            self.say("Nothing answers your call.");
            return false;
        };
        let level = self.state.current_level.clone();
        let seen = (self.state.actor_at(&level, at))
            .filter(|_| self.perception.sees_actors_at(at))
            .map(|a| a.id);
        if let Some(id) = seen
            && self.master_of(id).is_some()
        {
            let who = capital(&self.called(&self.state.actors[&id]));
            self.say(format!("{who} is already at someone's side."));
            return false;
        }
        let Some(target) = self.hex_target(at, cast.reach.range.unwrap_or(1)) else {
            return false;
        };
        if let Some(line) = self.uncharmable(target, charm) {
            self.say(line);
            return false;
        }
        let player = self.state.player;
        if self.allies_here(player).len() >= usize::from(rules.most) {
            self.say("You cannot lead more than this.");
            return false;
        }
        if !self.pay_cast(cast) {
            return true;
        }
        let minutes = u64::from(cast.degrees.unwrap_or(charm.minutes).max(1));
        let until = self.state.time + Duration::minutes(minutes);
        let faction = self.state.player().faction(&self.content);
        if let Some(actor) = self.state.actors.get_mut(&target) {
            let source = AllySource::Charmed {
                spell: cast.spell,
                faction: actor.faction_override,
            };
            actor.faction_override = faction;
            actor.ally = Some(Ally {
                master: player,
                source,
                order: Order::Follow,
                retinue: false,
                until: Some(until),
            });
        }
        let who = capital(&self.called(&self.state.actors[&target]));
        self.say(format!("{who} {}.", charm.message));
        self.spell_landed(cast, target, true, events);
        true
    }

    /// Why creature `id` cannot be charmed, as the player is told; `None` when it can.
    fn uncharmable(&self, id: ActorId, charm: &SpellCharm) -> Option<String> {
        let actor = &self.state.actors[&id];
        let def = self.content.creature(actor.creature);
        let who = capital(&self.called(actor));
        if def.unique || actor.npc.is_some() {
            return Some(format!("{who} is no one's fool."));
        }
        let mindless = actor.risen.is_some()
            || def.animated.is_some()
            || (charm.mindless.iter()).any(|t| self.content.has_tag(actor.creature, t));
        mindless.then(|| format!("{who} has no mind to charm."))
    }

    /// Charmed creature `id`'s charm ends (the module docs): its faction is put back and it is
    /// no one's ally, where it stands; said when the player sees it and `told`.
    pub(super) fn charm_ends(&mut self, id: ActorId, told: bool) {
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        let Some(AllySource::Charmed { spell, faction }) =
            actor.ally.as_ref().map(|a| a.source.clone())
        else {
            return;
        };
        actor.faction_override = faction;
        actor.ally = None;
        let pos = actor.pos;
        // What it summoned while charmed took the player's side with it, and goes back with it.
        let content = self.content.clone();
        let theirs = self.state.actors[&id].faction(&content);
        let mine = self.state.player().faction(&content);
        for summoned in self.state.actors.values_mut() {
            if summoned.summoner == Some(id) && summoned.faction(&content) == mine {
                let own = content.creature(summoned.creature).faction;
                summoned.faction_override = theirs.filter(|&f| Some(f) != own);
            }
        }
        if told && self.perception.sees_actors_at(pos) {
            let ends = (self.content.spell_of(spell))
                .and_then(|s| s.effect.charm.as_ref())
                .map_or("shakes off the charm", |c| c.ends.as_str())
                .to_string();
            let who = capital(&self.called(&self.state.actors[&id]));
            self.say(format!("{who} {ends}."));
        }
    }
}
