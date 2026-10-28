//! Chests in the world (spec §8 "Chests", [`crate::chest`]): opening (and unlocking), kicking
//! open, and blowing apart.
//!
//! - **Opening** a shut chest (`open:DIR`, or walking into it) lifts its lid (`open_chest`, a
//!   creak): what it held spills into the pile on its cell, which is walkable from then on (an
//!   open chest), and the player is told what they find. A locked one opens only to a key that
//!   fits (only a skeleton key fits a lock without an id), unlocking it in the same action
//!   (`unlock` + `open_chest`); without one, "The coffer is locked." and no time passes. Pick
//!   lock (`world/lockpick.rs`) and Knock (`world/knock.rs`) open the lock without a key.
//! - **Kicking** a shut chest (`kick:DIR`, priced as a door kick, banging as one): one gameplay
//!   draw, `1d(sturdiness) ≤ Strength` + athletics + Lockbreaker's `door_kick`, against its
//!   band's sturdiness. It holds ("You kick the coffer. It holds.") or bursts apart: smashed,
//!   each stack inside destroyed with `[smash] destroyed` percent (Deft breaker's cut), coins and
//!   loose gems spared, the rest spilled on its cell.
//! - **A blast** (`World::blast`) reaching a shut chest whose band's hardness is below it blows
//!   it apart: every stack is rolled at the same percent, nothing spared.
//!
//! A chest's entry stays on its cell once it is open or broken (empty), so it is still named by
//! its band; nothing closes it again.

use super::{Spent, World};
use crate::action::Event;
use crate::chest::{Breaking, ChestState, destroyed};
use crate::content::ActionKind;
use crate::door::kick_breaks;
use crate::geom::Pos;
use crate::noise::Act;
use crate::object::{Stack, add_to_pile, phrase};
use crate::skills::{Effect, Use};
use crate::stats::Stat;

/// The sturdiness of a chest without a band (content without `data/chests.toml`).
const STURDINESS: u8 = 20;

impl World {
    /// The name of the chest at `p` on the current level: its band's ("coffer"), else its
    /// terrain's.
    pub(super) fn chest_name(&self, p: Pos) -> String {
        let level = self.state.current();
        let band = (level.chests.get(&p)).and_then(|c| self.content.chests().band_of(c.level));
        match band {
            Some(band) => band.name.clone(),
            None => (level.terrain.get(p))
                .map_or_else(|| "chest".into(), |t| self.content.terrain(*t).name.clone()),
        }
    }

    /// The player opens the shut chest at `p`; a locked one only with a key that fits, which
    /// unlocks it first. Returns what it cost.
    pub(super) fn open_chest(&mut self, p: Pos, events: &mut Vec<Event>) -> Spent {
        let player = self.state.player;
        let name = self.chest_name(p);
        let open = self.price_of(player, ActionKind::OpenChest);
        let Some(lock) = self.state.current().locks.get(&p).cloned() else {
            self.say(format!("You open the {name}."));
            self.lift_lid(p, events);
            return Spent::Turn(open);
        };
        let Some(key) = self.state.player().key_for(&self.content, &lock) else {
            self.say(format!("The {name} is locked."));
            return Spent::Nothing;
        };
        let price = self.price_of(player, ActionKind::Unlock) + open;
        self.state.current_mut().locks.remove(&p);
        let key = self.content.object(key).name.clone();
        self.say(format!(
            "You unlock the {name} with your {key} and open it."
        ));
        self.lift_lid(p, events);
        Spent::Turn(price)
    }

    /// The player lifts the lid of the shut chest at `p` (unlocked by now): its creak, its
    /// contents spilled on its cell, the terrain the open chest, and what they find said.
    pub(super) fn lift_lid(&mut self, p: Pos, events: &mut Vec<Event>) {
        self.make_noise(self.state.player, Act::OpenDoor(p));
        let holds = self.empty_chest(p);
        if let Some(open) = self.content.chest_terrain(ChestState::Open) {
            self.set_terrain(p, open);
        }
        self.spill_chest(p, &holds);
        events.push(Event::DoorOpened { pos: p });
        self.say_found(&holds);
    }

    /// The player kicks the shut chest at `p`: a banging noise, and with a chance of their
    /// Strength (with athletics and Lockbreaker's might) in its band's sturdiness it bursts
    /// apart, smashed (one draw of the gameplay stream). Priced as a door kick; it trains
    /// athletics as one does.
    pub(super) fn kick_chest(&mut self, p: Pos, events: &mut Vec<Event>) -> Spent {
        let player = self.state.player;
        let price = self.price_of(player, ActionKind::Kick);
        let level = self.state.current();
        let sturdiness = (level.chests.get(&p))
            .and_then(|c| self.content.chests().band_of(c.level))
            .map_or(STURDINESS, |b| b.sturdiness);
        let me = self.state.player();
        let strength = me.effective_stats(&self.content).score(Stat::Strength);
        let athletics = me.skill_effect(&self.content, Effect::Kick);
        // Mighty kick, Breaker, and Lockbreaker (spec §8 "Talents"): as if stronger.
        let mighty = me.talent_mods(&self.content).door_kick;
        let force =
            u8::try_from((i32::from(strength) + athletics + mighty).max(0)).unwrap_or(u8::MAX);
        let broken = kick_breaks(force, sturdiness, &mut self.state.rng);
        if self.trains(Use::Kick) {
            self.train(Use::Kick);
        }
        self.make_noise(player, Act::Kick(p));
        if broken {
            self.break_chest(p, Breaking::Smashed, events);
        } else {
            events.push(Event::DoorKicked { pos: p });
            let name = self.chest_name(p);
            self.say(format!("You kick the {name}. It holds."));
        }
        Spent::Turn(price)
    }

    /// The shut chest at `p` breaks open `how`: each stack it holds is destroyed with `[smash]
    /// destroyed` percent (a smash spares coins and loose gems, and Deft breaker cuts the
    /// chance; a blast spares nothing), the rest spills on its cell, and the terrain becomes the
    /// smashed chest. It is said (a kicked chest always, a blast's if the player sees it), then
    /// what broke and what is left.
    pub(super) fn break_chest(&mut self, p: Pos, how: Breaking, events: &mut Vec<Event>) {
        let name = self.chest_name(p);
        let seen = how == Breaking::Smashed || self.perception.is_visible(p);
        let holds = self.empty_chest(p);
        let mut percent = i32::from(self.content.chests().destroyed);
        if how == Breaking::Smashed {
            let cut = self
                .state
                .player()
                .talent_mods(&self.content)
                .smash_destroy_percent;
            percent = percent * (100 + cut).max(0) / 100;
        }
        let content = self.content.clone();
        let spared = |s: &Stack| {
            let class = content.item_class(content.object(s.object).class);
            class.id == "coins" || class.id == "gems"
        };
        let percent = percent.clamp(0, 100) as u32;
        let (kept, lost) = destroyed(holds, how, percent, spared, &mut self.state.rng);
        if let Some(smashed) = self.content.chest_terrain(ChestState::Smashed) {
            self.set_terrain(p, smashed);
        }
        self.spill_chest(p, &kept);
        events.push(Event::ChestBroken { pos: p });
        if !seen {
            return;
        }
        self.say(match how {
            Breaking::Smashed => format!("The {name} bursts apart!"),
            Breaking::BlownUp => format!("The {name} is blown apart!"),
        });
        if !lost.is_empty() {
            let count: u32 = lost.iter().map(|s| s.count).sum();
            let verb = if count == 1 { "is" } else { "are" };
            let lost = capitalize(&phrase(&lost, self.lore()));
            self.say(format!("{lost} {verb} destroyed."));
        }
        if !kept.is_empty() {
            self.say_found(&kept);
        }
    }

    /// Takes what the chest at `p` holds out of it (it stays, empty).
    fn empty_chest(&mut self, p: Pos) -> Vec<Stack> {
        let level = self.state.current_mut();
        (level.chests.get_mut(&p)).map_or_else(Vec::new, |c| std::mem::take(&mut c.holds))
    }

    /// Lays `stacks` in the pile at `p`, in the order they were held.
    fn spill_chest(&mut self, p: Pos, stacks: &[Stack]) {
        if stacks.is_empty() {
            return;
        }
        let content = self.content.clone();
        let pile = self.state.current_mut().objects.entry(p).or_default();
        add_to_pile(pile, stacks, &content);
    }

    /// What the player finds in a chest just opened: "Inside you find a dagger and 12 gold
    /// coins." or "It is empty." (nothing identified).
    fn say_found(&mut self, found: &[Stack]) {
        if found.is_empty() {
            self.say("It is empty.");
        } else {
            let found = phrase(found, self.lore());
            self.say(format!("Inside you find {found}."));
        }
    }
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
