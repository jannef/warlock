//! Ice on the levels (spec §5 "Ice"; the model is [`crate::ice`]): bringing it up to the clock,
//! stepping onto it, breaking through and climbing out, an instant frost, and heat and blasts.
//!
//! **Stepping onto ice** ([`World::tread`], for the player and creatures alike): ice holds an
//! actor whose weight (its own and everything it carries) is no more than what it holds; above
//! that it gives way with a chance rising to certain at twice that (one draw of the gameplay
//! stream, made only between the two). Ice that held only by luck creaks under the player.
//! Wading into water with a skin of ice too thin to look like ice breaks the skin.
//!
//! **Breaking through**: the cell is open water again (the ice there is gone; it refreezes from
//! nothing), and what lay on the ice sinks to the bottom, where wading in fishes it out. An actor
//! that broke through from firm footing (not already in the water) plunges in: the water's
//! `plunge` hit points (never the last one: the cold does not kill yet), "The ice cracks beneath
//! you and you plunge into the freezing water!" (a creature the player sees: "The kobold falls
//! through the ice!"). From the water it only breaks the ice ahead of it.
//!
//! **Climbing out**: a step from open water onto ice that holds also costs `climb`
//! (`data/actions.toml`), a few turns of hauling oneself up.
//!
//! **Bursts** (the hooks for spell effects and explosions): [`World::frost`] freezes the water
//! in a burst at once ([`crate::ice::FROST`] for the dev action `freeze:x,y,r`); [`World::shock`]
//! melts (heat: a fireball) or shatters (a blast, below a thickness) the ice in a burst, and
//! whoever stood on it falls in. A burst reaches the cells within its radius in line of sight of
//! its centre.

use super::fight::Harm;
use super::{ActorId, LevelState, Spent, World, WorldState};
use crate::action::{Event, SpellShape};
use crate::content::StatUse;
use crate::content::{ActionKind, Content, TerrainId};
use crate::fov::compute_fov;
use crate::geom::Pos;
use crate::ice::{self, Ice, Shock};
use crate::object;
use crate::temperature::{self, Temperature};
use crate::time::GameTime;
use std::collections::{BTreeMap, BTreeSet};

/// What the player is told when they plunge through the ice.
const PLUNGE: &str = "The ice cracks beneath you and you plunge into the freezing water!";
/// The effect a frost burst shows (`data/effects.toml`).
const FROST_BURST: &str = "frost_burst";

/// What stepping onto a cell did to the ice there ([`World::tread`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Footing {
    /// No ice to speak of: an ordinary step (wading through a skin of ice breaks it).
    Plain,
    /// The ice held: a step onto it, climbing out of the water if the actor stood in it.
    Held { climb: bool },
    /// The ice gave way: the actor is in the water there, plunged in from firm footing or
    /// breaking the ice ahead of it while wading.
    Broke { plunge: bool },
}

impl LevelState {
    /// The ice at `p`, if any: ground truth, **for developer tools only** (the player sees what
    /// it looks like: thin ice, ice, thick ice).
    pub fn ice_at(&self, p: Pos) -> Option<Ice> {
        self.ice.get(&p).copied()
    }

    /// When the ice was last brought up to date (dev dumps).
    pub fn ice_until(&self) -> GameTime {
        self.ice_until
    }

    /// The temperature here underground; `None` under the open sky (it follows the weather).
    pub fn fixed_temperature(&self) -> Option<Temperature> {
        self.temperature
    }

    /// Shows the ice at `p` as thick as it is: the ice terrain its thickness reaches, else the
    /// water beneath. (Ice and water are transparent and give no light, so light masks keep.)
    pub(crate) fn show_ice(&mut self, content: &Content, p: Pos) {
        if let Some(ice) = self.ice.get(&p) {
            self.terrain[p] = content.ice_look(ice.um).unwrap_or(ice.water);
        }
    }

    /// Makes the ice at `p`, on `water`, `um` thick (none at all at 0), and shows it.
    pub(crate) fn set_ice(&mut self, content: &Content, p: Pos, water: TerrainId, um: u32) {
        if um == 0 {
            self.ice.remove(&p);
            self.terrain[p] = water;
        } else {
            self.ice.insert(p, Ice { water, um });
            self.show_ice(content, p);
        }
    }

    /// The water at `p` if it is water that freezes, under ice or open.
    pub(crate) fn water_at(&self, content: &Content, p: Pos) -> Option<TerrainId> {
        match self.ice.get(&p) {
            Some(ice) => Some(ice.water),
            None => (self.terrain.get(p).copied()).filter(|t| content.terrain(*t).freeze.is_some()),
        }
    }

    /// Whether `p` is open water: water that freezes with no ice on it that looks like ice
    /// (wading there, a step onto ice is climbing out).
    pub(crate) fn is_open_water(&self, content: &Content, p: Pos) -> bool {
        (self.terrain.get(p)).is_some_and(|t| content.terrain(*t).freeze.is_some())
    }

    /// Brings the ice up to `to`, tick by tick from where it was left ([`crate::ice`]), at the
    /// temperature `air` gives for each tick's moment. Cells that start alike stay alike, so it
    /// works through the thicknesses there are (and each kind of open water), not the cells.
    pub(crate) fn advance_ice(
        &mut self,
        content: &Content,
        to: GameTime,
        air: impl Fn(GameTime) -> Temperature,
    ) {
        let ticks = ice::ticks(self.ice_until.seconds(), to.seconds());
        if ticks.is_empty() {
            return;
        }
        self.ice_until = ice::tick_floor(to);
        let freeze = |water: TerrainId| content.terrain(water).freeze;
        // Each thickness there is on each water, open water of every kind that freezes at 0.
        let mut groups: BTreeMap<(TerrainId, u32), u32> = (content.terrain_defs().iter())
            .enumerate()
            .filter(|(_, t)| t.freeze.is_some())
            .map(|(i, _)| ((TerrainId(i as u16), 0), 0))
            .collect();
        for ice in self.ice.values() {
            groups.insert((ice.water, ice.um), ice.um);
        }
        // Underground the air keeps one temperature: nothing to do if no ice would change in it.
        if let Some(t) = self.temperature {
            let still = |(&(water, um), _): (&(TerrainId, u32), &u32)| {
                freeze(water).is_none_or(|f| ice::tick(um, t, &f) == um)
            };
            if groups.iter().all(still) {
                return;
            }
        }
        for k in ticks {
            let t = air(GameTime::from_seconds(ice::tick_moment(k)));
            for (&(water, _), um) in groups.iter_mut() {
                if let Some(f) = freeze(water) {
                    *um = ice::tick(*um, t, &f);
                }
            }
        }
        // Open water that froze: every cell of it without ice when this began.
        let froze: BTreeSet<TerrainId> = (groups.iter())
            .filter(|&(&(_, from), &um)| from == 0 && um > 0)
            .map(|(&(water, _), _)| water)
            .collect();
        let open: Vec<(Pos, TerrainId)> = if froze.is_empty() {
            Vec::new()
        } else {
            (self.terrain.iter())
                .filter(|(p, t)| froze.contains(t) && !self.ice.contains_key(p))
                .map(|(p, t)| (p, *t))
                .collect()
        };
        let iced: Vec<(Pos, Ice)> = self.ice.iter().map(|(p, i)| (*p, *i)).collect();
        for (p, ice) in iced {
            let um = groups[&(ice.water, ice.um)];
            self.set_ice(content, p, ice.water, um);
        }
        for (p, water) in open {
            self.set_ice(content, p, water, groups[&(water, 0)]);
        }
    }
}

impl WorldState {
    /// The temperature on `level` at `time`: fixed underground, else the weather's under the
    /// open sky (the dev override included while it lasts).
    pub(crate) fn temperature_on(
        &self,
        level: &LevelState,
        content: &Content,
        time: GameTime,
    ) -> Temperature {
        level.temperature.unwrap_or_else(|| {
            let over = self.weather_override.as_ref();
            temperature::outdoors(self.seed, time, content.climate(), over)
        })
    }

    /// Brings the ice on the current level up to the clock.
    pub(crate) fn update_ice(&mut self, content: &Content) {
        let (seed, over, now) = (self.seed, self.weather_override, self.time);
        if let Some(level) = self.levels.get_mut(&self.current_level) {
            advance(level, content, now, seed, over.as_ref());
        }
    }

    /// A level under the open sky seen for the first time has had [`ice::PRE_ROLL`] of weather:
    /// its water froze and thawed with it.
    pub(crate) fn pre_roll_ice(&mut self, content: &Content, id: &str) {
        let (seed, over, now) = (self.seed, self.weather_override, self.time);
        let Some(level) = self.levels.get_mut(id) else {
            return;
        };
        if level.temperature.is_some() {
            return;
        }
        let since = now.seconds().saturating_sub(ice::PRE_ROLL.as_seconds());
        level.ice_until = ice::tick_floor(GameTime::from_seconds(since));
        advance(level, content, now, seed, over.as_ref());
    }
}

/// Brings the ice on `level` up to `to` ([`LevelState::advance_ice`]) at its temperature: fixed
/// underground, else the weather's in the run with `seed` (with the dev override `over`).
fn advance(
    level: &mut LevelState,
    content: &Content,
    to: GameTime,
    seed: u64,
    over: Option<&crate::weather::Override>,
) {
    let fixed = level.temperature;
    level.advance_ice(content, to, |t| {
        fixed.unwrap_or_else(|| temperature::outdoors(seed, t, content.climate(), over))
    });
}

impl World {
    /// The temperature on the player's level now: ground truth, for the rules and dev tools (the
    /// player is told only how it feels, under the open sky: [`crate::PerceptionView`]).
    pub fn temperature(&self) -> Temperature {
        let state = &self.state;
        state.temperature_on(state.current(), &self.content, state.time)
    }

    /// Brings the ice on the player's level up to the clock: as the clock moves, so the rules
    /// always meet the ice of the moment.
    pub(super) fn update_ice(&mut self) {
        self.state.update_ice(&self.content);
    }

    /// Actor `id` steps from `from` onto `to` on the player's level: decides whether ice there
    /// holds it (it may break through), before the step is priced and taken. See the module
    /// docs.
    pub(super) fn tread(&mut self, id: ActorId, from: Pos, to: Pos) -> Footing {
        let Some(ice) = self.state.current().ice_at(to) else {
            return Footing::Plain;
        };
        let content = self.content.clone();
        let level = self.state.current();
        let wading = level.is_open_water(&content, from);
        if content.terrain(level.terrain[to]).ice.is_none() {
            // A skin of ice, too thin to be more than the water beneath: wading breaks it.
            self.state.current_mut().ice.remove(&to);
            return Footing::Plain;
        }
        let load = self.state.actors[&id].weight(&content).total();
        let broke = match ice::gives_way(ice.um, load) {
            0 => false,
            100.. => true,
            chance => {
                let broke = self.state.rng.chance(chance);
                if !broke && id == self.state.player {
                    self.say("The ice creaks under your weight.");
                }
                broke
            }
        };
        if !broke {
            return Footing::Held { climb: wading };
        }
        let sank = self.break_ice(to);
        if id == self.state.player {
            self.say(if wading {
                "You break through the ice."
            } else {
                PLUNGE
            });
            if !sank.is_empty() {
                let sink = if matches!(sank.as_slice(), [one] if one.count == 1) {
                    "sinks"
                } else {
                    "sink"
                };
                let what = capitalized(&object::phrase(&sank, &content));
                self.say(format!("{what} {sink} into the water."));
            }
        }
        Footing::Broke { plunge: !wading }
    }

    /// After a step: actor `id`, now at `at`, plunged through the ice or climbed out onto it.
    pub(super) fn after_tread(
        &mut self,
        id: ActorId,
        at: Pos,
        footing: Footing,
        events: &mut Vec<Event>,
    ) {
        match footing {
            Footing::Broke { plunge: true } => {
                self.fall_in(id, at, events);
                if id == self.state.player {
                    self.train_stat(StatUse::Plunge);
                }
            }
            Footing::Held { climb: true } if id == self.state.player => {
                self.say("You climb out onto the ice.");
                self.train_stat(StatUse::Climb);
            }
            _ => {}
        }
    }

    /// The price of what `footing` adds to a step: climbing out of the water onto ice.
    pub(super) fn tread_price(&self, id: ActorId, footing: Footing) -> i64 {
        match footing {
            Footing::Held { climb: true } => self.price_of(id, ActionKind::Climb),
            _ => 0,
        }
    }

    /// Breaks the ice at `p` on the player's level: open water there again. Returns the pile
    /// that lay on it, which sank to the bottom (it stays in the cell's pile, under the water).
    fn break_ice(&mut self, p: Pos) -> Vec<object::Stack> {
        let content = self.content.clone();
        let level = self.state.current_mut();
        if let Some(water) = level.water_at(&content, p) {
            level.set_ice(&content, p, water, 0);
        }
        level.objects_at(p).to_vec()
    }

    /// Actor `id` plunged through the ice into the freezing water at `at`: it loses the water's
    /// `plunge` hit points, cold damage it resists like any (spec §8 "Damage types"; never the
    /// last hit point: the cold does not kill yet, [`Harm::Spared`]). A creature's fall is said
    /// when the player perceives it ([`World::perceive`]); the player's where it happens.
    fn fall_in(&mut self, id: ActorId, at: Pos, events: &mut Vec<Event>) {
        let water = self.state.current().water_at(&self.content, at);
        let freeze = water.and_then(|w| self.content.terrain(w).freeze);
        let damage = freeze.map_or(0, |f| f.plunge.roll(&mut self.state.rng).max(0));
        let kind = freeze.and_then(|f| f.kind);
        let taken = self.resisted(id, crate::damage::Damage::new(damage, kind));
        self.hurt(id, taken, Harm::Spared, events);
        events.push(Event::FellThroughIce { actor: id, at });
    }

    /// An instant frost (spec §5 "Ice"; spell effects and frost traps): the water within
    /// `radius` of `center` in its line of sight freezes at once to at least `um` thick.
    /// Returns how many cells froze.
    pub(crate) fn frost(&mut self, center: Pos, radius: i32, um: u32) -> usize {
        let content = self.content.clone();
        let cells = self.burst(center, radius);
        let level = self.state.current_mut();
        let mut froze = 0;
        for p in cells {
            let Some(water) = level.water_at(&content, p) else {
                continue;
            };
            let now = level.ice_at(p).map_or(0, |i| i.um);
            if um > now {
                level.set_ice(&content, p, water, um.min(ice::MAX));
                froze += 1;
            }
        }
        froze
    }

    /// An explosion on the ice (the hook for fireballs and blasts): `shock` breaks the ice it
    /// reaches within `radius` of `center` in its line of sight (heat all of it, a blast the
    /// ice thinner than its strength), and whoever stood on that ice falls in. Returns how many
    /// cells broke.
    pub(crate) fn shock(
        &mut self,
        center: Pos,
        radius: i32,
        shock: Shock,
        events: &mut Vec<Event>,
    ) -> usize {
        let content = self.content.clone();
        let mut broke = 0;
        for p in self.burst(center, radius) {
            let level = self.state.current();
            let Some(ice) = level.ice_at(p) else {
                continue;
            };
            if !shock.breaks(ice.um) {
                continue;
            }
            // Only ice that looks like ice bears anyone: on a skin, one stands in the water.
            let bore = content.terrain(level.terrain[p]).ice.is_some();
            let standing = (self.state.actors.values())
                .find(|a| a.level == level.id && a.pos == p)
                .map(|a| a.id);
            self.break_ice(p);
            broke += 1;
            if let (true, Some(id)) = (bore, standing) {
                if id == self.state.player {
                    self.say(PLUNGE);
                }
                self.fall_in(id, p, events);
            }
        }
        broke
    }

    /// What a frost burst of `radius` around `center` looks like (`frost_burst` in
    /// `data/effects.toml`, spec §4.8 "Spell effects"), when content has it: a spell event over
    /// the cells it reaches, for presentation.
    pub(super) fn frost_look(&self, center: Pos, radius: i32, events: &mut Vec<Event>) {
        if self.content.effect(FROST_BURST).is_none() {
            return;
        }
        let level = self.state.current();
        let opaque = |p| level.is_opaque(&self.content, p);
        let cells = super::spell::burst_cells(center, radius, opaque);
        if !cells.is_empty() {
            events.push(Event::Spell {
                effect: FROST_BURST.to_string(),
                origin: center,
                shape: SpellShape::Burst { center, radius },
                cells,
            });
        }
    }

    /// The cells a burst of `radius` around `center` reaches on the player's level: within the
    /// radius and in line of sight of the centre.
    fn burst(&self, center: Pos, radius: i32) -> BTreeSet<Pos> {
        let level = self.state.current();
        let content = &self.content;
        let mut cells = BTreeSet::new();
        let opaque = |p| level.is_opaque(content, p);
        compute_fov(center, radius.max(0), opaque, |p| {
            if level.terrain.in_bounds(p) {
                cells.insert(p);
            }
        });
        cells
    }

    /// The dev action `freeze:x,y,r` ([`crate::action::Action::Freeze`]): a frost burst at `at`
    /// within `radius`, said and shown. Takes no time.
    pub(super) fn freeze_dev(&mut self, at: Pos, radius: i32, events: &mut Vec<Event>) -> Spent {
        let froze = self.frost(at, radius, crate::ice::FROST);
        self.say(format!(
            "Frost seizes the water around {},{} ({froze} cells, dev).",
            at.x, at.y
        ));
        self.frost_look(at, radius, events);
        Spent::NoTime
    }

    /// The dev action `thaw:x,y,r` ([`crate::action::Action::Thaw`]): a burst of heat at `at`
    /// within `radius`, said. Takes no time.
    pub(super) fn thaw_dev(&mut self, at: Pos, radius: i32, events: &mut Vec<Event>) -> Spent {
        self.say(format!(
            "Heat melts the ice around {},{} (dev).",
            at.x, at.y
        ));
        self.shock(at, radius, crate::ice::Shock::Heat, events);
        Spent::NoTime
    }
}

/// `text` with its first letter in upper case.
fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
