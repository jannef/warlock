//! Gathering herbs from the glowing fungus (spec §8 "Skills": Herbalism; #1340, the recipe
//! design's §8): [`Action::Gather`], the Gather ability of the abilities menu's *Here* group,
//! offered beside the fungus.
//!
//! - **Who**: everyone, while `data/herbs.toml` exists. Herbalism makes it worth doing.
//! - **Where**: the cell the way named, beside the player, whose terrain is the file's `terrain`
//!   (the glowing fungus, [`fungus_ways`]). Refused, taking no time, anywhere else ("There is
//!   nothing here to gather."). The menu asks for the way only when fungus stands on more than
//!   one cell beside the player.
//! - **Time**: the file's `seconds`, in turns of the `gather` action ([`World::keep_at`], the
//!   rest's machinery; the last turn cut short to end on time): a creature coming into view, a
//!   message, or a blow stops it, and a gather cut short finds nothing (the time is spent).
//! - **The find**, once the time is up: how many, `count` (a draw per die) plus one for every
//!   `value_per_count` of Herbalism's trained value, or `untrained` (no draw) for one not trained
//!   in it; then each herb one draw by weight from the table at the level's item level
//!   ([`crate::content::HerbRules::at`]). They go into the pack: "You gather 2 sprigs of
//!   moonwort and a glowcap." Then Herbalism may earn a training mark ([`Use::Gather`], the
//!   training stream). Every find draw is from the gameplay stream, in that order.
//! - **The fungus** is never picked over: no state changes, so the same fungus may be gathered
//!   again at once (no anti-farming limit, the owner's rule: time is the price).
//!
//! [`Action::Gather`]: crate::action::Action::Gather

use super::rest::Ended;
use super::{LevelState, Spent, World};
use crate::action::Event;
use crate::content::{ActionKind, Content};
use crate::geom::{Dir, Pos};
use crate::magic::loot::pick_weighted;
use crate::object::{self, Stack};
use crate::skills::{Effect, Use};
use crate::time::Duration;

/// The ways from `at` on `level` to a cell of the fungus herbs are gathered from, in
/// [`Dir::ALL`] order: none without `data/herbs.toml`.
pub(crate) fn fungus_ways(content: &Content, level: &LevelState, at: Pos) -> Vec<Dir> {
    let Some(rules) = content.herbs() else {
        return Vec::new();
    };
    (Dir::ALL.iter().copied())
        .filter(|&d| level.terrain_at(at.step(d)) == Some(rules.terrain))
        .collect()
}

impl World {
    /// The player begins to gather from the fungus the way `dir`; refused, taking no time, with
    /// none there.
    pub(super) fn start_gather(&mut self, dir: Dir) -> Spent {
        let content = self.content.clone();
        let here = self.player_pos();
        let ways = fungus_ways(&content, self.state.current(), here);
        let Some(rules) = content.herbs().filter(|_| ways.contains(&dir)) else {
            self.say("There is nothing here to gather.");
            return Spent::Nothing;
        };
        self.say("You search the glowing fungus for herbs.");
        Spent::Gather(rules.seconds)
    }

    /// Gathers turn after turn until its time is up or something stops it (module docs): its
    /// last turn cut short to end exactly when its time is up, as a forage's is.
    pub(super) fn gather(&mut self, seconds: u32, events: &mut Vec<Event>) {
        let until = self.state.time + Duration::seconds(u64::from(seconds));
        let turn = self.price_of(self.state.player, ActionKind::Gather);
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| turn.min(w.price_until(until)),
        );
        match ended {
            Ended::Died => {}
            _ if self.state.time >= until => self.gathered(),
            Ended::Came(came) => {
                let text = format!("{}; you stop gathering.", self.came_line(came));
                self.say(text);
            }
            Ended::Said | Ended::Done => self.say("You stop gathering."),
        }
    }

    /// A gather ran its time: how many, which herbs, the mark (module docs).
    fn gathered(&mut self) {
        let content = self.content.clone();
        let Some(rules) = content.herbs() else {
            return;
        };
        let me = self.state.player();
        let count = if me.skills.trains_effect(&content, Effect::Herbs) {
            let value = me.skills.value_of_effect(&content, Effect::Herbs);
            let dice = rules.count.roll(&mut self.state.rng).max(1);
            dice + i32::from(value / rules.value_per_count)
        } else {
            i32::from(rules.untrained)
        };
        let table = rules.at(content.item_level(&self.state.current_level));
        // Each herb found, joined to a stack of its kind, in the order first found.
        let mut stacks: Vec<Stack> = Vec::new();
        for _ in 0..count {
            let Some(&herb) = pick_weighted(&mut self.state.rng, &table) else {
                break;
            };
            match stacks.iter_mut().find(|s| s.object == herb) {
                Some(stack) => stack.count += 1,
                None => stacks.push(Stack::new(herb, 1)),
            }
        }
        if stacks.is_empty() {
            self.say("You find nothing worth gathering.");
        } else {
            crate::rot::made_at(&mut stacks, &content, self.state.time);
            let text = format!("You gather {}.", found(&stacks, &content));
            self.say(text);
            for stack in stacks {
                object::add_to_inventory(self.state.inventory_mut(), stack, &content);
            }
        }
        if self.trains(Use::Gather) {
            self.train(Use::Gather);
        }
    }
}

/// The herbs found, in a sentence, however many kinds: "a glowcap", "2 sprigs of moonwort and
/// a glowcap", "a, b, c, and d".
fn found(stacks: &[Stack], content: &Content) -> String {
    let names: Vec<String> = stacks.iter().map(|s| s.describe(content)).collect();
    match names.as_slice() {
        [] => "nothing".into(),
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}
