//! Sleepers (spec §7 "Special rooms": teeming dens; `docs/dev/special-rooms.md`): a creature
//! with a status that `sleeps` (Asleep; a den's sleepers are placed with it, `mapgen/den.rs`)
//! takes no turn ([`World::slumbers`], at the top of `creature_turn`) and is unaware of everyone
//! (`ambush.rs`: a sneak attack lands). Nothing wakes it with time; it wakes when
//!
//! - **a noise** reaches it ([`World::spread_noise`], where every noise is heard): at least its hearing
//!   threshold × its den's `wake` percent (100 % outside a den) where it lies; a noise the
//!   player makes needs Hush-foot's percent more (`TalentMods::wake_percent`);
//! - **a blow** or any other harm leaves it alive (`fight.rs::hurt`);
//! - a neighbour's **cry**: a sleeper that wakes cries out at once (a voice of its den's `cry`
//!   loudness where it lies, through [`World::make_noise_at`]; none outside a den), which may
//!   wake the next, and so on through the den: a cascade, deterministic, each sleeper waking
//!   once.
//!
//! Light and sight never wake a sleeper. Waking ends the status ("The goblin wakes up." when the
//! player sees it); the creature is an ordinary one from its next turn on.

use super::{ActorId, World};
use crate::content::ActionKind;
use crate::noise::{self, Act, Heard, Noise, VOICE_NOISE};

impl World {
    /// Creature `id`'s sleep, if it sleeps: its status that `sleeps` (Asleep).
    fn sleep_of(&self, id: ActorId) -> Option<crate::content::StatusId> {
        let actor = self.state.actors.get(&id)?;
        if actor.effects.is_empty() {
            return None;
        }
        (actor.effects.iter())
            .map(|e| e.status)
            .find(|&s| self.content.status(s).sleeps)
    }

    /// Whether actor `id` sleeps.
    pub(super) fn asleep(&self, id: ActorId) -> bool {
        self.sleep_of(id).is_some()
    }

    /// Creature `id`'s turn if it sleeps: it forgets what it heard and waits (a stride's waits
    /// at once while the player rests, as an idle creature away from them would: a sleeper costs
    /// next to nothing in a rest). `None`: it is awake and acts as usual.
    /// A disguised mimic (`mimic.rs`) waits the same way: it never acts until found out.
    pub(super) fn slumbers(&mut self, id: ActorId) -> Option<i64> {
        if !self.asleep(id) && !self.disguised(id) {
            return None;
        }
        if let Some(actor) = self.state.actors.get_mut(&id) {
            actor.heard.clear();
        }
        let wait = self.price_of(id, ActionKind::Wait);
        let times = if self.resting {
            i64::from(super::stride::STRIDE)
        } else {
            1
        };
        Some(wait.saturating_mul(times))
    }

    /// `noise` is heard on the current level: everyone it reaches loudly enough keeps it in
    /// their [`super::Actor::heard`] list, and the sleepers it is loud enough for wake.
    pub(super) fn spread_noise(&mut self, noise: Noise) {
        let hearers = noise::hearers(&self.state, &self.content, noise);
        let mut woken = Vec::new();
        for (id, loudness) in hearers {
            if let Some(actor) = self.state.actors.get_mut(&id) {
                actor.heard.push(Heard {
                    kind: noise.kind,
                    at: noise.at,
                    loudness,
                });
            }
            if self.wakes_to(id, noise, loudness) {
                woken.push(id);
            }
        }
        for id in woken {
            self.wake(id);
        }
    }

    /// Whether sleeper `id` wakes to `noise`, `loudness` loud where it lies: at least its
    /// hearing threshold × its den's `wake` percent, more for a noise of the player's with
    /// Hush-foot.
    fn wakes_to(&self, id: ActorId, noise: Noise, loudness: u16) -> bool {
        if !self.asleep(id) {
            return false;
        }
        let Some(actor) = self.state.actors.get(&id) else {
            return false;
        };
        let Some(threshold) = actor.hearing(&self.content) else {
            return false;
        };
        let level = self.state.current();
        let here = level.effects_at(&self.content, actor.pos);
        let mut percent = u64::from(here.swarm().map_or(100, |s| s.wake));
        let player = self.state.player;
        if noise.maker == Some(player) {
            let more = self.state.player().talent_mods(&self.content).wake_percent;
            percent = percent * u64::try_from((100 + more).max(0)).unwrap_or(0) / 100;
        }
        u64::from(loudness) * 100 >= u64::from(threshold) * percent
    }

    /// Actor `id` wakes, if it sleeps: its sleep ends, the player is told if they see it, and it
    /// cries out (its den's `cry`), which may wake others.
    pub(super) fn wake(&mut self, id: ActorId) {
        let Some(sleep) = self.sleep_of(id) else {
            return;
        };
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        actor.effects.remove(sleep);
        let (pos, creature) = (actor.pos, actor.creature);
        if id != self.state.player && self.perception.sees_actors_at(pos) {
            let name = self.content.creature(creature).name.clone();
            self.say(format!("{} wakes up.", crate::content::the_start(&name)));
        }
        let level = self.state.current();
        let cry = (level.effects_at(&self.content, pos).swarm()).map_or(0, |s| s.cry);
        if cry > 0 {
            let percent = u32::from(cry) * 100 / u32::from(VOICE_NOISE);
            self.make_noise_at(id, Act::Mutter, percent);
        }
    }

    /// Sixth sense on arrival (with the danger's feeling): a note when something sleeps on the
    /// level.
    pub(super) fn sense_sleepers(&mut self) {
        let (player, level) = (self.state.player, self.state.current_level.clone());
        let sleepers = (self.state.actors.values())
            .filter(|a| a.level == level && a.id != player)
            .any(|a| {
                !a.effects.is_empty()
                    && (a.effects.iter()).any(|e| self.content.status(e.status).sleeps)
            });
        if sleepers {
            self.note("Something sleeps on this level.");
        }
    }
}
