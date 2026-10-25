//! Reading scrolls (spec §8 "Scrolls"): [`Action::Read`](crate::Action::Read).
//!
//! The player reads one of a stack they carry whose object says what reading does (`read`,
//! [`Reading`]); it takes the `read` action's time and uses the scroll up. A reading that cannot
//! be done is refused, takes no time, and keeps the scroll: nothing to read, a spell with no
//! cell to aim at (or the player's own, or one out of range, or on the overworld), identify with
//! no stack to read it on or one there is nothing to learn about.
//!
//! **Spell scrolls** cast with the traps' rules ([`World::cast_burst`], [`World::cast_bolt`]),
//! aimed by the reader:
//!
//! - a **burst** (fireball, frost) flies from the reader along the line through the cell aimed
//!   at (the line a missile flies, [`Pos::ray`]) and goes off on the first cell someone stands
//!   on, before the first opaque cell (on the reader's own cell when that is the next), or on the
//!   cell aimed at: it may catch the reader (the interface asks first, spec §6);
//! - a **bolt** (lightning) runs from the reader along that line and on, through everyone on it,
//!   as far as its length or the first opaque cell; the reader's own cell is not on it.
//!
//! The reader casts it ([`Caster::Actor`]): its blows are theirs, so its kills earn them their
//! experience and count for their quests, unlike a trap's; a scroll that kills its reader is
//! what killed them ([`Cause::Scroll`]). The spell's noise is made where it goes off (a burst's
//! center, the reader for a bolt), heard by everyone in earshot. Messages are said at once, by
//! what the player sees: "You read the scroll of fireball.", the scroll's message ("The fireball
//! explodes!"), then what it did to whom ("The fireball burns the kobold.", "The kobold dies.").
//!
//! **The scroll of identify** identifies one unidentified stack the player carries
//! ([`World::identify`]): "This is a Sharp long sword of the Bear!"

use super::fight::{End, Guard, Words, attack_message, death_message};
use super::spell::Caster;
use super::{Cause, Spent, World};
use crate::action::{AttackKind, Event, Positions, ReadTarget};
use crate::content::StatUse;
use crate::content::{ActionKind, Cast, Reading, ScrollSpell, percent_of};
use crate::geom::Pos;
use crate::object::Stack;

/// What the player is told when Spell recall keeps a scroll they read.
const KEPT: &str = "The writing does not fade from the scroll.";

impl World {
    /// The player reads one of inventory stack `item` at `target` (module docs).
    pub(super) fn read(
        &mut self,
        item: usize,
        target: Option<ReadTarget>,
        events: &mut Vec<Event>,
    ) -> Spent {
        let content = self.content.clone();
        let Some(stack) = self.state.player().inventory.get(item) else {
            self.say("You carry no such thing.");
            return Spent::Nothing;
        };
        let def = content.object(stack.object);
        let Some(reading) = &def.read else {
            self.say(format!(
                "You cannot read {}.",
                stack.single().describe(&content)
            ));
            return Spent::Nothing;
        };
        match (reading, target) {
            (Reading::Identify, Some(ReadTarget::Item(on))) => self.read_identify(item, on),
            (Reading::Identify, _) => {
                self.say("Read it on what? Name a stack you carry: `read:a:b`.");
                Spent::Nothing
            }
            (Reading::Spell(spell), Some(ReadTarget::Cell(at))) => {
                self.read_spell(item, spell, at, events)
            }
            (Reading::Spell(_), _) => {
                self.say("Read it at what? Name a cell to aim at: `read:a:12,5`.");
                Spent::Nothing
            }
        }
    }

    /// Reads the scroll of identify in inventory stack `item` on inventory stack `on`.
    fn read_identify(&mut self, item: usize, on: usize) -> Spent {
        let content = self.content.clone();
        let Some(stack) = self.state.player().inventory.get(on) else {
            self.say("You carry no such thing.");
            return Spent::Nothing;
        };
        if stack.is_identified() {
            let what = stack.single().describe(&content);
            self.say(format!("You already know what {what} is."));
            return Spent::Nothing;
        }
        let price = self.read_price();
        let scroll = self.state.player().inventory[item].single();
        self.say(format!(
            "You read the {}.",
            content.object(scroll.object).name
        ));
        // Identified first: using the scroll up may move the stack it was read on.
        if matches!(self.identify(on), super::Identified::Now(_)) {
            self.train_stat(StatUse::Identified);
        }
        if self.use_up(item).is_some_and(|(_, kept)| kept) {
            self.say(KEPT);
        }
        Spent::Turn(price)
    }

    /// Reads the spell scroll in inventory stack `item`, `spell`, at cell `at`.
    fn read_spell(
        &mut self,
        item: usize,
        spell: &ScrollSpell,
        at: Pos,
        events: &mut Vec<Event>,
    ) -> Spent {
        if !self.can_loose_here() || !self.within_reach(at, spell.range, "aim") {
            return Spent::Nothing;
        }
        let content = self.content.clone();
        let player = self.state.player;
        let here = self.player_pos();
        // Priced as the reading begins, with the scroll still in the pack.
        let price = self.read_price();
        let (scroll, kept) = self.use_up(item).expect("the scroll is there");
        self.say(format!(
            "You read the {}.",
            content.object(scroll.object).name
        ));
        if let Some(text) = &spell.message {
            self.say(text.clone());
        }
        let mut made = Vec::new();
        match &spell.cast {
            Cast::Burst(burst) => {
                let center = self.burst_center(here, at, spell.range);
                self.sound_at(spell.noise, center);
                let caster = Caster::Actor(player);
                self.cast_burst(caster, (here, center), burst, &mut made);
            }
            Cast::Bolt(bolt) => {
                let path = self.bolt_line(here, at, bolt.length);
                self.sound_at(spell.noise, here);
                self.cast_bolt(Caster::Actor(player), here, &path, bolt, &mut made);
            }
        }
        self.say_spell((&spell.noun, &spell.verb), made, events);
        if kept && self.state.death.is_none() {
            self.say(KEPT);
        }
        // Killed by their own spell (no action is taken after death, so it was this one): the
        // scroll is what killed them, not "a you".
        if let Some(death) = &mut self.state.death {
            death.by = None;
            death.cause = Some(Cause::Scroll(scroll.object));
        }
        Spent::Turn(price)
    }

    /// Where a burst read from `from` at `at` goes off (module docs): the first cell on the
    /// line through `at` someone stands on, the last before an opaque cell, or `at`.
    fn burst_center(&self, from: Pos, at: Pos, range: i32) -> Pos {
        let level = self.state.current();
        let mut center = from;
        let steps = from.chebyshev(at).clamp(0, range.max(0)) as usize;
        for cell in from.ray(at).take(steps) {
            if level.is_opaque(&self.content, cell) {
                break;
            }
            center = cell;
            if cell == at || self.state.actor_at(&level.id, cell).is_some() {
                break;
            }
        }
        center
    }

    /// The cells a bolt read from `from` at `at` runs over: the line through `at` and on, after
    /// `from`, at most `length` cells, cut short before the first opaque cell.
    fn bolt_line(&self, from: Pos, at: Pos, length: i32) -> Vec<Pos> {
        let level = self.state.current();
        (from.ray(at).take(length.max(1) as usize))
            .take_while(|&p| !level.is_opaque(&self.content, p))
            .collect()
    }

    /// Says what the reader's spell did, by what the player sees as it happens (their own blows
    /// are said at once, as their shots are): each blow on someone they see or themselves, and
    /// the deaths of those they saw; then any news it brought (a level gained). Its events go to
    /// presentation the same way: the spell's own, and the blows and deaths seen. The player's
    /// own death is said as the turn is perceived. `(noun, verb)` say the spell's blows ("the
    /// fireball burns"); a birth sign's Blaze is said the same way.
    pub(super) fn say_spell(
        &mut self,
        (noun, verb): (&str, &str),
        made: Vec<Event>,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let player = self.state.player;
        let words = Words::new(noun, verb);
        let mut said = Vec::new();
        // Whom the last blow struck, as the player saw them (if they did).
        let mut seen: Option<End<'_>> = None;
        for event in made {
            match event {
                Event::Attacked {
                    target, to, strike, ..
                } => {
                    let fallen = self.actor_or_fallen(target);
                    let def = fallen.map(|a| content.creature(a.creature));
                    let guard = Guard::of(fallen, &content, &self.state);
                    let end = match def {
                        _ if target == player => End::You,
                        Some(d) if self.perception.sees_actors_at(to) => End::Seen(&d.name),
                        _ => End::Unseen,
                    };
                    let caster = End::Trap(noun);
                    let kind = AttackKind::Spell;
                    said.extend(attack_message(caster, end, &words, kind, strike, guard));
                    seen = (end != End::Unseen).then_some(end);
                    if seen.is_some() {
                        events.push(event);
                    }
                }
                Event::Died { actor, .. } if actor == player => events.push(event),
                Event::Died { .. } => {
                    if let Some(end) = seen {
                        said.push(death_message(end));
                        events.push(event);
                    }
                }
                other => events.push(other),
            }
        }
        for text in said {
            self.say(text);
        }
        self.say_news();
    }

    /// What reading a scroll costs the player: the `read` action's price, scaled by their
    /// talents (Scroll savant: half), at least 1.
    fn read_price(&self) -> i64 {
        let price = self.price_of(self.state.player, ActionKind::Read);
        match self.state.player().talent_mods(&self.content).read_percent {
            0 => price,
            more => percent_of(price, u32::try_from((100 + more).max(0)).unwrap_or(0)).max(1),
        }
    }

    /// Takes one of inventory stack `item` from the pack (the last empties it, and what was worn
    /// follows its letters); returns it, and whether it was kept instead: with Spell recall
    /// (spec §8 "Talents") a read keeps its scroll by the talent's chance, one draw of the
    /// gameplay stream a read (none without it).
    fn use_up(&mut self, item: usize) -> Option<(Stack, bool)> {
        self.train_stat(StatUse::Read);
        let one = self.state.player().inventory.get(item)?.single();
        let keep = self.state.player().talent_mods(&self.content).scroll_kept;
        if keep > 0 && self.state.rng.chance(u32::from(keep)) {
            return Some((one, true));
        }
        let player = self.player_mut();
        let stack = player.inventory.get_mut(item)?;
        stack.count -= 1;
        if stack.count == 0 {
            player.inventory.remove(item);
            player.equipment.after_removal(&Positions::of([item]));
        }
        Some((one, false))
    }
}
