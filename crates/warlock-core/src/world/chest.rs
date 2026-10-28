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
//!   draw, `1d(sturdiness) ≤ Strength` + athletics + Lockbreaker's `door_kick` + Hobnailed's
//!   (`World::kick_force`), against its band's sturdiness. It holds ("You kick the coffer. It holds.") or bursts apart: smashed,
//!   each stack inside destroyed with `[smash] destroyed` percent (Deft breaker's cut), coins and
//!   loose gems spared, the rest spilled on its cell.
//! - **A blast** (`World::blast`) reaching a shut chest whose band's hardness is below it blows
//!   it apart: every stack is rolled at the same percent, nothing spared. Its trap, if any, is
//!   dropped with the lid and springs at no one (a trap needs a victim).
//! - **Trapped chests**: a trap on a shut chest (any trap a door may have) goes off at whoever
//!   opens it (with a key, without, or after Pick lock), smashes it with a kick that breaks it
//!   (a kick that holds springs nothing), or fails badly to disarm it ([`World::spring_trap`],
//!   aimed from the chest: a shot out of the box itself). A burst goes off inside the box and
//!   blows it apart, nothing spared, however it was sprung; the other traps leave it as it was.
//!
//! A chest's entry stays on its cell once it is open or broken (empty), so it is still named by
//! its band; nothing closes it again.

use super::{ActorId, Spent, World};
use crate::action::Event;
use crate::chest::{Breaking, ChestState, destroyed};
use crate::content::ActionKind;
use crate::door::kick_breaks;
use crate::geom::Pos;
use crate::noise::Act;
use crate::object::{Stack, add_to_pile, phrase};
use crate::skills::Use;

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
            self.open_lid(p, events);
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
        self.open_lid(p, events);
        Spent::Turn(price)
    }

    /// The player opens the shut chest at `p` (unlocked by now): its trap, if any, goes off at
    /// them first, and a burst blows the chest apart instead of opening it; else the lid lifts
    /// ([`World::lift_lid`]).
    pub(super) fn open_lid(&mut self, p: Pos, events: &mut Vec<Event>) {
        let (_, blown) = self.spring_chest_trap(p, self.state.player, events);
        if !blown {
            self.lift_lid(p, events);
        }
    }

    /// The trap at `p` goes off at actor `victim` ([`World::spring_trap`]); if it lay on a shut
    /// chest and is a burst ([`crate::content::TrapEffect::blows_fixture`]), the chest is blown
    /// apart, nothing spared (unless the burst's own blast already did it). Returns the energy a
    /// pit's climb costs (never on a chest) and whether a chest blew up. Nothing happens where no
    /// trap lies.
    pub(super) fn spring_chest_trap(
        &mut self,
        p: Pos,
        victim: ActorId,
        events: &mut Vec<Event>,
    ) -> (i64, bool) {
        let level = self.state.current();
        let blows = level.is_shut_chest(&self.content, p)
            && (level.traps.get(&p))
                .is_some_and(|t| self.content.trap(t.kind).effect.blows_fixture());
        if !blows {
            return (self.spring_trap(p, victim, events), false);
        }
        // The blow-up is said after the burst's own report ("A fireball explodes!"), which is
        // said as the turn is perceived: held meanwhile, it joins the news.
        let outer = self.held.replace(Vec::new());
        let climb = self.spring_trap(p, victim, events);
        if self.state.current().is_shut_chest(&self.content, p) {
            self.break_chest(p, Breaking::BlownUp, events);
        }
        let lines = std::mem::replace(&mut self.held, outer).unwrap_or_default();
        match &mut self.held {
            Some(outer) => outer.extend(lines),
            None => self.news.extend(lines),
        }
        (climb, true)
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
    /// apart, smashed (one draw of the gameplay stream): its trap, if any, goes off at them
    /// first, and a burst blows it apart instead. A kick that holds springs nothing. Priced as a
    /// door kick; it trains athletics as one does.
    pub(super) fn kick_chest(&mut self, p: Pos, events: &mut Vec<Event>) -> Spent {
        let player = self.state.player;
        let price = self.price_of(player, ActionKind::Kick);
        let level = self.state.current();
        let sturdiness = (level.chests.get(&p))
            .and_then(|c| self.content.chests().band_of(c.level))
            .map_or(STURDINESS, |b| b.sturdiness);
        // As a door's kick ([`World::kick_force`]).
        let force = self.kick_force();
        let broken = kick_breaks(force, sturdiness, &mut self.state.rng);
        if self.trains(Use::Kick) {
            self.train(Use::Kick);
        }
        self.make_noise(player, Act::Kick(p));
        if broken {
            let (_, blown) = self.spring_chest_trap(p, player, events);
            if !blown {
                self.break_chest(p, Breaking::Smashed, events);
            }
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

    /// Chest-wise (spec §8 "Chests"): a shut chest's hidden trap in the player's sight gives
    /// itself away to them, known and remembered at once and said ("You notice that the coffer
    /// is trapped: an arrow trap."). No draw: the talent sees it.
    pub(super) fn chest_wise_look(&mut self) {
        let level = self.state.current();
        if level.chests.is_empty() || self.state.player().talents.is_empty() {
            return;
        }
        if !self.state.player().talent_mods(&self.content).chest_wise {
            return;
        }
        let spotted: Vec<Pos> = (level.chests.keys())
            .filter(|p| {
                level.is_shut_chest(&self.content, **p)
                    && self.perception.sees_actors_at(**p)
                    && level.traps.get(p).is_some_and(|t| !t.known)
            })
            .copied()
            .collect();
        for p in spotted {
            let Some(kind) = self.know_trap(p) else {
                continue;
            };
            self.state.current_mut().trap_memory.insert(p, kind);
            let name = self.chest_name(p);
            let trap = &self.content.trap(kind).name;
            let text = format!(
                "You notice that the {name} is trapped: {} {trap}.",
                crate::content::article(trap)
            );
            self.say(text);
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
