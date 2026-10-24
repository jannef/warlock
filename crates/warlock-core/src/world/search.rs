//! Searching (spec §8 "Searching"): finding hidden features, with one check for all of them.
//!
//! Hidden features are **secret doors**, which look and act like the wall around them (the cell's
//! terrain is that wall; [`crate::content::Secret`] keeps the door), and **hidden traps**, which
//! look like the floor or the door they lie on (spec §8 "Traps", "Trapped doors"). Each has a
//! **difficulty**: the trap type's, or the secret door's own, plus on a dungeon branch its
//! `hidden_per_depth` for each level below the first. Two checks find them (spec §8 "Checks",
//! [`crate::check`]: 2d12 plus a modifier against the difficulty):
//!
//! - **Searching** ([`crate::Action::Search`]): each turn of it (the `search` action) checks
//!   every hidden feature within reach, the cells around the player (farther with a high Search
//!   skill, those of them in line of sight), with the **active** check: the Intelligence
//!   modifier, plus what the Search skill gives, plus the race's `search`. Searching for a while
//!   goes on turn after turn through [`World::keep_at`], so it stops like a rest, and at once when
//!   something is found.
//! - **Noticing in passing**: after every turn the player takes, each hidden feature on the cells
//!   around them gets the **passive** check: the Willpower modifier, plus what the Spot skill
//!   gives, plus the race's `search`, less the passive penalty (`data/search.toml`). Elves notice
//!   everything so, dwarves what lies on stone (`passive_search`), anyone trained in Spot
//!   everything; everyone else notices nothing in passing.
//!
//! Every check is rolled afresh: nothing is remembered of a failed one, so the next turn may
//! find what this one missed (no limit on trying again: the owner's rule). Checks draw from the
//! gameplay stream only when something hidden is in reach, so a turn with nothing hidden around
//! draws nothing. A turn spent searching may earn the Search skill a training mark, and noticing
//! something the Spot skill (training stream).
//!
//! A found secret door becomes its closed door ([`World::set_terrain`]), a found trap is known
//! ([`World::know_trap`], as a trap seen going off is), and both are remembered at once, in the
//! dark too: "You find a secret door!", "You notice a pressure plate: an arrow trap.", and for a
//! door's trap "You notice that the door is trapped: an arrow trap." (a trap's `sign` is the
//! floor's).

use super::World;
use super::rest::Ended;
use crate::action::Event;
use crate::check::check;
use crate::content::StatUse;
use crate::content::{ActionKind, PassiveSearch, RaceDef, TrapId, article};
use crate::geom::Pos;
use crate::sight::LineOfSight;
use crate::skills::{Effect, Use};
use crate::stats::Stat;
use std::cell::Cell;

/// A hidden feature on a cell of the current level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Hidden {
    /// A secret door of this difficulty.
    Door(u8),
    /// A hidden trap of this type.
    Trap(TrapId),
}

/// Which of the two checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Look {
    /// Searching: Intelligence and the Search skill.
    Active,
    /// In passing: Willpower and the Spot skill, less the passive penalty.
    Passive,
}

impl World {
    /// The player's race, if the run has a character.
    fn player_race(&self) -> Option<&RaceDef> {
        (self.state.character.as_ref()).and_then(|c| c.spec.race(&self.content).ok())
    }

    /// The hidden features within `reach` cells of the player (Chebyshev distance), not on their
    /// own cell (a trap there has just gone off), in position order; beyond the cells around
    /// them, only those in their line of sight.
    fn hidden_near(&self, reach: i32) -> Vec<(Pos, Hidden)> {
        let level = self.state.current();
        let here = self.player_pos();
        let near = |p: &Pos| *p != here && p.chebyshev(here) <= reach;
        let doors = (level.secrets.iter())
            .filter(|(p, _)| near(p))
            .map(|(p, s)| (*p, Hidden::Door(s.difficulty)));
        let traps = (level.traps.iter())
            .filter(|(p, t)| !t.known && near(p))
            .map(|(p, t)| (*p, Hidden::Trap(t.kind)));
        let mut hidden: Vec<(Pos, Hidden)> = doors.chain(traps).collect();
        hidden.sort_by_key(|(p, _)| *p);
        if reach > 1 && hidden.iter().any(|(p, _)| p.chebyshev(here) > 1) {
            let radius = LineOfSight::radius_for(2 * reach * reach);
            let los = LineOfSight::compute(here, radius, |p| level.is_opaque(&self.content, p));
            hidden.retain(|(p, _)| p.chebyshev(here) <= 1 || los.contains(*p));
        }
        hidden
    }

    /// What finding `hidden` takes (spec §8 "Searching"): its own difficulty, plus on a branch
    /// level the branch's `hidden_per_depth` for each level below the first.
    pub(super) fn difficulty_of(&self, hidden: Hidden) -> i32 {
        let own = match hidden {
            Hidden::Door(difficulty) => i32::from(difficulty),
            Hidden::Trap(kind) => i32::from(self.content.trap(kind).difficulty),
        };
        let deeper = (self.content.branch_level(&self.state.current_level))
            .map_or(0, |(def, depth)| def.hidden_at(depth));
        own.saturating_add(deeper)
    }

    /// What the player adds to a check of `look`: the stat's modifier (of the effective score,
    /// spec §8 "Stats"), the skill's benefit, and the race's bonus; less the passive penalty in
    /// passing.
    pub(super) fn search_modifier(&self, look: Look) -> i32 {
        let content = &self.content;
        let player = self.state.player();
        let stats = player.effective_stats(content);
        let race = self.player_race().map_or(0, |r| r.search);
        let own = match look {
            Look::Active => {
                stats.modifier(Stat::Intelligence) + player.skill_effect(content, Effect::Search)
            }
            Look::Passive => {
                stats.modifier(Stat::Willpower)
                    + player.skill_effect(content, Effect::Spot)
                    + player.talent_mods(content).notice.unwrap_or(0)
                    - content.search().passive_penalty
            }
        };
        own.saturating_add(race)
    }

    /// Whether the player notices `hidden`, the hidden feature at `at`, in passing at all:
    /// trained in the Spot skill, or with a talent that notices (Alert), anything; else by their
    /// race, elves anything, dwarves what is on stone, and kobolds traps; else nothing.
    pub(super) fn notices(&self, at: Pos, hidden: Hidden) -> bool {
        let content = &self.content;
        let player = self.state.player();
        if player.skills.trains_effect(content, Effect::Spot)
            || player.talent_mods(content).notice.is_some()
        {
            return true;
        }
        match self.player_race().and_then(|r| r.passive_search) {
            Some(PassiveSearch::All) => true,
            Some(PassiveSearch::Stone) => {
                let terrain = self.state.current().terrain.get(at).copied();
                terrain.is_some_and(|t| content.terrain(t).stone)
            }
            Some(PassiveSearch::Traps) => matches!(hidden, Hidden::Trap(_)),
            None => false,
        }
    }

    /// Searching for up to `turns` turns (spec §8 "Searching"): a turn of the `search` action
    /// after another through [`World::keep_at`], stopping when something is found, like a rest
    /// otherwise. A search of more than one turn that finds nothing says how it ended.
    pub(super) fn search(&mut self, turns: u8, events: &mut Vec<Event>) {
        let (start, found) = (self.state.time, self.found);
        let done = Cell::new(0u8);
        let ended = self.keep_at(
            events,
            |w| done.get() < turns && w.found == found,
            |w, _| {
                let price = w.price_of(w.state.player, ActionKind::Search);
                done.set(done.get() + 1);
                w.search_turn();
                price
            },
        );
        if turns == 1 || self.found != found {
            return;
        }
        let took = self.state.time.since(start).spoken();
        let text = match ended {
            Ended::Died => return,
            Ended::Came(id) => format!(
                "{} comes into view; you stop searching after {took}.",
                self.creature_called(id)
            ),
            Ended::Said => format!("You stop searching after {took}."),
            Ended::Done => format!("You search for {took} and find nothing."),
        };
        self.say(text);
    }

    /// One turn of searching: the active check for every hidden feature within reach, then a
    /// training mark's chance for the Search skill.
    fn search_turn(&mut self) {
        let reach = 1 + (self.state.player()).skill_effect(&self.content, Effect::SearchReach);
        let hidden = self.hidden_near(reach);
        if !hidden.is_empty() {
            let modifier = self.search_modifier(Look::Active);
            for (at, hidden) in hidden {
                let difficulty = self.difficulty_of(hidden);
                if check(&mut self.state.rng, modifier, difficulty) {
                    self.reveal(at, hidden);
                    self.train_stat(StatUse::Found);
                }
            }
        }
        if self.trains(Use::Search) {
            self.train(Use::Search);
        }
    }

    /// After a turn the player took (`run_turn`, before anyone else acts): the passive check for
    /// every hidden feature on the cells around them that they notice in passing at all. Draws
    /// nothing when there is none.
    pub(super) fn notice_hidden(&mut self) {
        let hidden = self.hidden_near(1);
        if hidden.is_empty() {
            return;
        }
        let modifier = self.search_modifier(Look::Passive);
        for (at, hidden) in hidden {
            if !self.notices(at, hidden) {
                continue;
            }
            let difficulty = self.difficulty_of(hidden);
            if check(&mut self.state.rng, modifier, difficulty) {
                self.reveal(at, hidden);
                if self.trains(Use::Spot) {
                    self.train(Use::Spot);
                }
            }
        }
    }

    /// The player found `hidden` at `at`: a secret door becomes its door, a trap is known, and
    /// either is remembered at once (in the dark too) and said, which stops a rest.
    fn reveal(&mut self, at: Pos, hidden: Hidden) {
        self.found += 1;
        match hidden {
            Hidden::Door(_) => {
                let Some(secret) = self.state.current().secrets.get(&at).copied() else {
                    return;
                };
                self.set_terrain(at, secret.door);
                self.state.current_mut().memory.set(at, Some(secret.door));
                self.say("You find a secret door!");
            }
            Hidden::Trap(_) => {
                let Some(kind) = self.know_trap(at) else {
                    return;
                };
                self.state.current_mut().trap_memory.insert(at, kind);
                let def = self.content.trap(kind);
                let sign = if self.state.current().is_door(&self.content, at) {
                    "that the door is trapped"
                } else {
                    &def.sign
                };
                let text = format!("You notice {sign}: {} {}.", article(&def.name), def.name);
                self.say(text);
            }
        }
    }
}
