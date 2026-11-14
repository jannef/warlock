//! Resting (spec §8 "Time"): [`crate::action::Action::Rest`], its goal, and the turn-by-turn
//! loop of a long action ([`World::keep_at`]) that resting, eating, and digging share.
//! `world/mod.rs` has only the dispatch line.

use super::encounter::Watch;
use super::{ActorId, Spent, World};
use crate::action::{DiseaseNews, Event, MAX_REST, Rest};
use crate::content::{ActionKind, LevelKind, StatUse};
use crate::geom::Pos;
use crate::perception::VIEW_RADIUS;
use crate::sight::{self, LineOfSight};
use crate::time::{self, GameTime};

/// The player's line of sight as a long action keeps it: for the terrain changes made so far
/// ([`World::reshaped`]) and where they stand.
pub(super) type Sightline = ((u64, Pos), LineOfSight);

/// How near, in steps (Chebyshev), a hostile creature in view must come to stop a long action
/// ([`World::keep_at`], spec §8 "Rest"); one farther off may be seen and the action goes on.
pub(super) const CLOSE: i32 = 5;

/// A creature that stopped a long action by coming within [`CLOSE`] in view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Came {
    pub id: ActorId,
    /// Whether the player saw it farther off before (it came close), or not (it came into view).
    pub seen: bool,
}

/// How a long action (a rest, digging; [`World::keep_at`]) ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Ended {
    /// There was nothing more to do.
    Done,
    /// This hostile creature came close in view.
    Came(Came),
    /// A message was logged that stops it.
    Said,
    /// The player died.
    Died,
}

impl World {
    /// Rests until `until`: turn after turn of waiting ([`World::keep_at`]), the last cut short to
    /// end exactly at `until`, whatever the player's speed ([`World::price_until`]); then says
    /// how long the player rested. The player heals faster all the while.
    pub(super) fn rest(&mut self, until: GameTime, goal: Rest, events: &mut Vec<Event>) {
        self.resting = true;
        self.rest_until(until, goal, events);
        self.resting = false;
    }

    pub(super) fn rest_until(&mut self, until: GameTime, goal: Rest, events: &mut Vec<Event>) {
        let start = self.state.time;
        let wait = self.price_of(self.state.player, ActionKind::Wait);
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| {
                let price = wait.min(w.price_until(until));
                // Who comes upon a camp on the overworld (`encounter.rs`, #807).
                w.watch_roll(Watch::Rest, price);
                price
            },
        );
        let rested = self.state.time.since(start).spoken();
        let text = match (ended, goal) {
            (Ended::Died, _) => return,
            (Ended::Came(came), _) => {
                format!("{}; you stop resting after {rested}.", self.came_line(came))
            }
            (Ended::Said, _) => format!("You stop resting after {rested}."),
            (Ended::Done, Rest::For(_)) => format!("You rest for {rested}."),
            (Ended::Done, Rest::Dawn) => "You rest until dawn.".into(),
            (Ended::Done, Rest::Dusk) => "You rest until dusk.".into(),
        };
        self.say(text);
    }

    /// What is said of a creature that stopped a long action ([`Came`]): "A kobold comes into
    /// view", or, seen farther off before, "The kobold comes close".
    pub(super) fn came_line(&self, came: Came) -> String {
        let called = self.creature_called(came.id);
        if !came.seen {
            return format!("{called} comes into view");
        }
        let name = &self
            .content
            .creature(self.state.actors[&came.id].creature)
            .name;
        let called = match crate::content::is_proper(name) {
            true => called,
            false => crate::content::the_start(name),
        };
        format!("{called} comes close")
    }

    /// Passes turn after turn of a long action (a rest, digging) through the normal pipeline
    /// while `more` says there is more to do: the player does `act` (which returns what it
    /// costs), the others act, perception. It stops after a turn in which a creature hostile to
    /// the player ([`World::disturbs_rest`]) came within [`CLOSE`] in view (one not so the turn
    /// before; one farther off may be seen and the action goes on), in which something attacked
    /// the player or a shot or a spell came within [`CLOSE`] of them ([`World::stirs`]), or in
    /// which a message was logged that stops a rest (an attack on the player, a hurt, hunger
    /// worsening), and when the player dies. Turns that end before any such creature could come
    /// within [`CLOSE`] ([`World::out_of_view_for`]), in which the player heard nothing, in
    /// which nothing happened that perception would stop the action for ([`World::may_stop`]: a
    /// hostile in view farther off may shoot or cast meanwhile), in which the weather the player
    /// sees did not change, in
    /// which none of the player's diseases showed or went badly (news that stops a rest), and
    /// in which no trap went off (a teleport moves a creature farther than its steps) pass
    /// without perception, which cannot change the outcome; only the last turn's events are
    /// kept, and the player's status effects beginning and ending in the others, their
    /// resources ending, and their diseases' news (so they are said).
    pub(super) fn keep_at(
        &mut self,
        events: &mut Vec<Event>,
        more: impl Fn(&Self) -> bool,
        mut act: impl FnMut(&mut Self, &mut Vec<Event>) -> i64,
    ) -> Ended {
        let said = self.said;
        let mut in_view = self.watched_in_view();
        let mut seen = self.hostiles_in_view();
        let mut came = None;
        let mut stirred = false;
        let alive = |w: &Self| w.state.death.is_none();
        let mut turns = 0u32;
        // The player's line of sight, kept while they stay put and the terrain is unchanged.
        let mut sightline = None;
        while more(self) && came.is_none() && !stirred && self.said == said && alive(self) {
            // What was perceived was said.
            events.clear();
            let unseen_until = self.state.time + self.out_of_view_for();
            let (sprung, spawned) = (self.sprung, self.spawned);
            let mut stood = self.player_pos();
            loop {
                events.retain(|e| {
                    matches!(
                        e,
                        Event::Status { .. } | Event::ResourceEnded { .. } | Event::Disease { .. }
                    )
                });
                let price = act(self, events);
                self.run_turn(price, events);
                turns += 1;
                let done = !more(self) || self.said != said || !alive(self);
                // Something heard may be news (or someone muttering): perception says. So does a
                // disease showing or going badly (news that stops a rest), and a trap going off
                // (a teleport moves someone farther than its steps).
                let heard = !self.state.player().heard.is_empty();
                let ill = events.iter().any(
                    |e| matches!(e, Event::Disease { news, .. } if *news != DiseaseNews::Recovered),
                );
                let trapped = self.sprung != sprung;
                // A step taken (a run's, `run.rs`) changes what is seen and what is near.
                let moved = self.player_pos() != std::mem::replace(&mut stood, self.player_pos());
                // A creature that came may walk into view sooner than the others could.
                let newcomer = self.spawned != spawned;
                // Within the stretch, a hostile in view but not yet close may shoot or cast, or a
                // fight be seen near: what perception would stop the action for is looked at.
                let happened = events.iter().any(|e| self.may_stop(e));
                if done
                    || heard
                    || ill
                    || trapped
                    || moved
                    || newcomer
                    || happened
                    || self.weather_turned()
                {
                    break;
                }
                // Past the time a creature could have come close, the player looks, unless those
                // the player would see close are those they saw.
                if self.state.time > unseen_until
                    && (self.watched_in_sight(&mut sightline)).is_none_or(|seen| seen != in_view)
                {
                    break;
                }
            }
            self.perceive(events);
            stirred = events.iter().any(|e| self.stirs(e));
            let now_in_view = self.watched_in_view();
            came = (now_in_view.iter().find(|a| !in_view.contains(a))).map(|&id| Came {
                id,
                seen: seen.contains(&id),
            });
            in_view = now_in_view;
            seen = self.hostiles_in_view();
        }
        let ended = match came {
            _ if !alive(self) => Ended::Died,
            Some(came) => Ended::Came(came),
            None if self.said != said || stirred => Ended::Said,
            None => Ended::Done,
        };
        // Seen through to its end, longer than a turn: it trains Willpower (spec §8 "Stats").
        // A run is no such effort (`run.rs`).
        if ended == Ended::Done && turns > 1 && !self.running {
            self.train_stat(StatUse::Endured);
        }
        ended
    }

    /// Whether a turn of a long action with `event` in it (as it happened, before perception
    /// cuts it to what was seen) must be perceived, as perception may stop the action for it:
    /// what [`World::stirs`] it, an attack or a death within [`CLOSE`] of the player
    /// (`perceive_events` says others' farther off without stopping it), a wall blasted, a fall
    /// through ice, a drain, a trap's blow. Creatures moving, doors, and the rest (the player's
    /// own status effects, resources, and diseases are kept for the next perception) pass
    /// unseen.
    fn may_stop(&self, event: &Event) -> bool {
        let here = self.player_pos();
        let close = |p: Pos| p.chebyshev(here) <= CLOSE;
        match event {
            Event::Attacked { from, to, .. } => self.stirs(event) || close(*from) || close(*to),
            Event::Died { at, .. } => close(*at),
            Event::Missile { .. } | Event::Spell { .. } => self.stirs(event),
            Event::TrapStruck { .. }
            | Event::WallBroken { .. }
            | Event::FellThroughIce { .. }
            | Event::Drained { .. } => true,
            _ => false,
        }
    }

    /// Whether a perceived `event` stops a long action whatever was said of it: an attack on the
    /// player (a blow, a shot, a throw, a spell, missed or resisted too), or a shot or a spell
    /// within [`CLOSE`] of them.
    fn stirs(&self, event: &Event) -> bool {
        let player = self.state.player;
        let here = self.player_pos();
        let close = |p: &Pos| p.chebyshev(here) <= CLOSE;
        match event {
            Event::Attacked { target, .. } | Event::TrapStruck { target, .. } => *target == player,
            Event::Missile { path, .. } => path.iter().any(close),
            Event::Spell { cells, .. } => cells.iter().any(close),
            _ => false,
        }
    }

    /// The creatures that disturb a rest ([`World::disturbs_rest`]) the player sees within
    /// [`CLOSE`] now, in actor order: [`World::watched_in_view`] as a fresh perception would find
    /// it, worked out by the rules of sight ([`sight::sees`], the perception's own) for their
    /// cells alone. `None`
    /// where the perception sees by more than line of sight and light: on the overworld, with
    /// Blood scent, with light-blind eyes; and where smoke lies (a line of sight kept across
    /// turns would not know it).
    pub(super) fn watched_in_sight(
        &mut self,
        kept: &mut Option<Sightline>,
    ) -> Option<Vec<ActorId>> {
        let content = self.content.clone();
        let state = &self.state;
        let level = state.current();
        let player = state.player();
        if level.kind == LevelKind::Overworld
            || !player.talent_mods(&content).senses.scent.is_empty()
            || (state.race_of(&content, player)).is_some_and(|r| r.light_blind.is_some())
            || (level.zones.iter()).any(|z| z.kind == crate::content::ZoneKind::Smoke)
        {
            return None;
        }
        let viewer = player.pos;
        let near: Vec<(ActorId, Pos)> = (state.actors.on_level(&level.id))
            .filter(|a| a.pos.chebyshev(viewer) <= CLOSE)
            .filter(|a| self.disturbs_rest(a))
            .map(|a| (a.id, a.pos))
            .collect();
        if near.is_empty() {
            return Some(Vec::new());
        }
        let key = (self.reshaped, viewer);
        let los = match kept.take() {
            Some((k, los)) if k == key => los,
            _ => LineOfSight::compute(viewer, VIEW_RADIUS, |p| level.is_opaque(&content, p)),
        };
        let eye = state.eyes(&content);
        let World { state, lights, .. } = self;
        let seen = near
            .into_iter()
            .filter(|&(_, p)| sight::sees(state, &content, lights, &los, &eye, p));
        let seen = seen.map(|(id, _)| id).collect();
        *kept = Some((key, los));
        Some(seen)
    }

    /// When a rest toward `goal` ends: at most [`MAX_REST`] from now.
    pub(super) fn start_rest(&mut self, goal: Rest) -> Spent {
        if self.refused_afloat("You cannot rest in deep water.") {
            return Spent::Nothing;
        }
        let now = self.state.time;
        let until = match goal {
            Rest::For(duration) => now + duration.min(MAX_REST),
            Rest::Dawn => now.next_at(time::DAWN),
            Rest::Dusk => now.next_at(time::DUSK),
        };
        if until == now {
            self.say("You do not rest.");
            return Spent::Nothing;
        }
        Spent::Rest(until, goal)
    }
}
