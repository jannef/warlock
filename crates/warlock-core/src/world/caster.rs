//! The casters' passives that need a rule of their own (spec §8 "Class abilities", design
//! §4.9–4.12): the once-in-a-life reprieves (Miracle, Hag's bargain) and the blows that hurt a
//! suffering creature more (Poison lore). Their spells are cast by the one cast path
//! (`world/cast.rs`).

use super::World;
use crate::action::Event;
use crate::combat::Blow;
use crate::world::ActorId;

impl World {
    /// Miracle, Hag's bargain (spec §8 "Class abilities"): once in a life, what would kill the
    /// player does not. They are left at 1 hit point, suffer the reprieve's status, and lose its
    /// stat for good; its words are said. Returns whether it saved them (never twice).
    pub(super) fn reprieve(&mut self, events: &mut Vec<Event>) -> bool {
        let player = self.state.player;
        let Some(reprieve) = self
            .state
            .player()
            .talent_mods(&self.content)
            .into_owned()
            .reprieve
        else {
            return false;
        };
        if self.state.progress.happened(&reprieve.once) {
            return false;
        }
        self.state.progress.once.insert(reprieve.once.clone());
        self.say(reprieve.message.clone());
        let me = self.player_mut();
        me.hp = 1;
        if let Some((stat, by)) = reprieve.stat {
            me.stats.gift(stat, by);
        }
        self.afflict(player, reprieve.status, 1, None, events);
        true
    }

    /// `blow`, struck by `attacker` at `target`, with what Poison lore adds against a creature
    /// suffering its status: its damage more on a hit. No draw.
    pub(super) fn lore_blow(&self, attacker: ActorId, target: ActorId, blow: Blow) -> Blow {
        if !blow.strike.is_hit() {
            return blow;
        }
        let (Some(by), Some(on)) = (
            self.state.actors.get(&attacker),
            self.state.actors.get(&target),
        ) else {
            return blow;
        };
        if by.talents.is_empty() {
            return blow;
        }
        let more: i32 = (by.talent_mods(&self.content).status_damage.iter())
            .filter(|(status, _)| on.effects.has(*status))
            .map(|(_, v)| v)
            .sum();
        Blow {
            damage: (blow.damage + more).max(0),
            ..blow
        }
    }
}
