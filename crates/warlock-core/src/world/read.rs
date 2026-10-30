//! Reading scrolls (spec §8 "Scrolls"): [`Action::Read`](crate::Action::Read).
//!
//! The player reads one of a stack they carry whose object says what reading does (`read`,
//! [`Reading`]); it takes the `read` action's time and uses the scroll up. A reading that cannot
//! be done is refused, takes no time, and keeps the scroll: nothing to read, a spell with no
//! cell to aim at (or the player's own, or one out of range, or on the overworld), identify with
//! no stack to read it on or one there is nothing to learn about.
//!
//! **Scrolls of spells** cast a spell of `data/spells/` through the one cast path
//! ([`World::read_cast`]) at the scroll's own rungs plus the reader's Scroll adept, free of mana:
//! a fireball flies and bursts, a lightning bolt runs through everyone on its line, as the spell's
//! shape says; its blows are the reader's, and a scroll that kills its reader is what killed them
//! ([`Cause::Scroll`]). The scrolls of light and identify too: the scroll of identify casts
//! Identify on one unidentified stack the player carries ([`World::identify`]): "This is a
//! Sharp long sword of the Bear!", or the type of a scroll still under its label.
//!
//! **Unknown scrolls** ([`crate::scroll_label`]) read just as known ones do; the read names the
//! scroll by its label ("You read the scroll labelled ZUNBAR.") and identifies its type ("It is a
//! scroll of fireball."), the scroll of identify's own too.

use super::fight::{End, Guard, Words, attack_message, death_message};
use super::{Spent, World};
use crate::action::{AttackKind, Event, Positions, ReadTarget};
use crate::content::StatUse;
use crate::content::{ActionKind, Reading, percent_of};
use crate::geom::Pos;
use crate::object::Stack;

/// What the player is told when Spell recall keeps a scroll they read.
pub(super) const KEPT: &str = "The writing does not fade from the scroll.";

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
            let what = self.lore().describe(&stack.single());
            self.say(format!("You cannot read {what}."));
            return Spent::Nothing;
        };
        match (reading, target) {
            (Reading::Cast(scroll), Some(ReadTarget::Cell(_)) | None) if scroll.on_item => {
                self.say("Read it on what? Name a stack you carry: `read:a:b`.");
                Spent::Nothing
            }
            (Reading::Cast(_), Some(ReadTarget::Item(_))) if !reading.on_item() => {
                self.say("Read it at what? Name a cell to aim at: `read:a:12,5`.");
                Spent::Nothing
            }
            (Reading::Cast(scroll), target) => {
                // Priced as the reading begins, with the scroll still in the pack.
                let price = self.read_price();
                let cast = (scroll.spell, scroll.rungs);
                if self.read_cast(item, cast, target, events) {
                    Spent::Turn(price)
                } else {
                    Spent::Nothing
                }
            }
        }
    }

    /// Says that the player reads `scroll`, by what they knew it as ("You read the scroll
    /// labelled ZUNBAR."), and, if they did not know its type, what it turned out to be ("It is
    /// a scroll of fireball."): reading a scroll identifies its type (spec §8 "Scrolls").
    pub(super) fn say_read(&mut self, scroll: &Stack) {
        let title = self.lore().title(scroll);
        self.say(format!("You read the {title}."));
        if self.learn_scroll(scroll.object) {
            let what = self.content.object(scroll.object).describe(1);
            self.say(format!("It is {what}."));
        }
    }

    /// Where a burst read from `from` at `at` goes off (module docs): the first cell on the
    /// line through `at` someone stands on, the last before an opaque cell, or `at`.
    pub(super) fn burst_center(&self, from: Pos, at: Pos, range: i32) -> Pos {
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
    pub(super) fn bolt_line(&self, from: Pos, at: Pos, length: i32) -> Vec<Pos> {
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
        let mut seen: Option<(End<'_>, &str)> = None;
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
                    seen =
                        (end != End::Unseen).then(|| (end, def.map_or("dies", |d| d.death_verb())));
                    if seen.is_some() {
                        events.push(event);
                    }
                }
                Event::Died { actor, .. } if actor == player => events.push(event),
                Event::Died { .. } => {
                    if let Some((end, dies)) = seen {
                        said.push(death_message(end, dies));
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
    pub(super) fn use_up(&mut self, item: usize) -> Option<(Stack, bool)> {
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
