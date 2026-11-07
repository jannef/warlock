//! Zones (the spell types design `docs/design/spell-types-and-new-spells.md` §7 "Zones"; #1000):
//! an area that lies on the floor for a time, laid by a spell's `[spell.effect] zone` over the
//! cells of its shape, or by the dev action `zone:KIND@X,Y,R`. Kept in [`LevelState::zones`],
//! saved and hashed, left out while empty. One reader a kind, each at an existing moment:
//!
//! - **fire** (Balefire): a creature that starts its turn on it ([`World::zone_turn_start`], from
//!   `creature_turn` and `run_turn`) or steps onto it ([`World::zone_tread`], from `after_tread`)
//!   takes its harm, one number rolled as it is laid, through `spell_hurts`, then the fire type's
//!   rule. Each cell sheds a fire's light (`LightCache::sources`); ice under it melts as it is
//!   laid; a cold spell's `douses` puts it out cell by cell ([`LevelState::clear_zone_cells`]).
//! - **fouled** (Reek): a creature that starts its turn in it suffers the poison type's rule, no
//!   harm (the poison-immune nothing, `afflict`). It does not block sight.
//! - **slick** (Slick): a step onto it or off it is a Dexterity check against
//!   [`SLICK_DIFFICULTY`]; failed, Staggered, the step already spent. Nothing incorporeal slips.
//! - **dark** (Mirk): no light reaches its cells and a source on them sheds none
//!   ([`LevelState::shade`]), so eyes that need light see nothing there, the player's and a
//!   creature's alike (`perception.rs`, `sight.rs`, one shade for both).
//!
//! Rules: the cells are fixed, it never spreads; a new zone replaces an old one on the shared
//! cells; it takes the caster and their allies like anyone (`by` is whose kill it is, never who
//! it spares); leaving the level keeps it; past its `until` it does nothing, and it is dropped at
//! the end of the turn ([`World::zones_fade`]), quietly. No zone on the overworld.
//!
//! What the player knows is `Perception::zones`, the zones on the cells they see, and what they
//! saw is remembered as terrain is (`LevelState::zone_memory`), through
//! [`crate::perception::PerceptionView::zone_at`].
//!
//! Determinism: a fire's one roll as it is cast; Slick's check (two draws) at a step onto or off
//! it; the type's rider's chance per creature harmed, as any spell of the type. Nothing else
//! draws, and with no zone on the level nothing here costs more than a look at an empty list.

use super::spell::{Caster, burst_cells};
use super::{ActorId, Cause, LevelState, Spent, World};
use crate::action::{AttackKind, Event, SpellShape};
use crate::check;
use crate::combat::Strike;
use crate::content::{BurstIce, Content, LightId, Shape, SpellId, ZoneKind};
use crate::damage::Damage;
use crate::geom::Pos;
use crate::light::Light;
use crate::stats::Stat;
use crate::time::{Duration, GameTime};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::cast::Cast;

/// The check a step onto slick ground is made against.
const SLICK_DIFFICULTY: i32 = 12;
/// A player's turn, for a zone's `turns`.
const TURN_SECONDS: u64 = 6;
/// The dev action's zone: how many turns it lies, and the harm of a fire.
const DEV_TURNS: u64 = 10;
const DEV_HARM: i32 = 4;

/// An area that lies on the floor for a time (the module docs).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Zone {
    pub(crate) kind: ZoneKind,
    pub(crate) cells: BTreeSet<Pos>,
    /// When it ends.
    pub(crate) until: GameTime,
    /// Whose kill it is, never who it spares.
    pub(crate) by: ActorId,
    /// A fire's harm, rolled as it was laid.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub(crate) harm: i32,
    /// A fire's light on each cell.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) light: Option<LightId>,
    /// The spell that laid it: its words, and a death of its caster by it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) spell: Option<SpellId>,
}

fn is_zero(n: &i32) -> bool {
    *n == 0
}

impl Zone {
    /// What lies here.
    pub fn kind(&self) -> ZoneKind {
        self.kind
    }

    /// Its cells, in position order.
    pub fn cells(&self) -> impl Iterator<Item = Pos> + '_ {
        self.cells.iter().copied()
    }

    /// When it ends.
    pub fn until(&self) -> GameTime {
        self.until
    }

    /// Whose kill it is.
    pub fn by(&self) -> ActorId {
        self.by
    }

    /// A fire's harm (0 for the other kinds).
    pub fn harm(&self) -> i32 {
        self.harm
    }
}

impl LevelState {
    /// The zone on `p` at `now`: at most one, as a new zone replaces an old on shared cells.
    pub(crate) fn zone_at(&self, p: Pos, now: GameTime) -> Option<&Zone> {
        (self.zones.iter()).find(|z| now < z.until && z.cells.contains(&p))
    }

    /// Whether a darkness lies on `p` at `now`.
    pub(crate) fn darkened(&self, p: Pos, now: GameTime) -> bool {
        (self.zone_at(p, now)).is_some_and(|z| z.kind == ZoneKind::Dark)
    }

    /// What is left of `light` at `p` at `now`: a special room's share of it, and none in a
    /// darkness. The one shade of the light pass, the player's and a creature's alike.
    pub(crate) fn shade(&self, content: &Content, p: Pos, light: Light, now: GameTime) -> Light {
        if self.darkened(p, now) {
            return Light::DARK;
        }
        self.room_shade(content, p, light)
    }

    /// Lays `zone`, taking its cells from the zones already there.
    fn lay_zone(&mut self, zone: Zone) {
        for old in &mut self.zones {
            old.cells.retain(|p| !zone.cells.contains(p));
        }
        self.zones.retain(|z| !z.cells.is_empty());
        self.zones.push(zone);
    }

    /// `cells` are cleared of the zones of `kinds` (a cold spell's `douses`, a fire; Gale's
    /// `clears`, a cloud).
    pub(crate) fn clear_zone_cells(&mut self, cells: &[Pos], kinds: &[ZoneKind]) {
        for zone in self.zones.iter_mut().filter(|z| kinds.contains(&z.kind)) {
            zone.cells.retain(|p| !cells.contains(p));
        }
        self.zones.retain(|z| !z.cells.is_empty());
    }
}

impl World {
    /// The cells `cast`'s zone would lie on, aimed from `here` at `at`, before the cast changes
    /// who stands where (a burst goes off at the first creature in its way): none for a spell
    /// that lays none.
    pub(super) fn zone_cells(
        &self,
        cast: &Cast<'_>,
        (here, at, range): (Pos, Pos, i32),
        radius: i32,
    ) -> Vec<Pos> {
        if cast.def.effect.zone.is_none() {
            return Vec::new();
        }
        let level = self.state.current();
        if cast.def.shape == Shape::Line {
            return self.bolt_line(here, at, range);
        }
        let center = self.burst_center(here, at, range);
        let opaque = |p: Pos| p != center && level.is_opaque(&self.content, p);
        burst_cells(center, radius, opaque)
    }

    /// `cast` went off: its zone lies on `cells` for its turns. A fire's harm is rolled here,
    /// one number for all its time, and the ice under a fire melts.
    pub(super) fn lay_spell_zone(
        &mut self,
        cast: &Cast<'_>,
        cells: Vec<Pos>,
        events: &mut Vec<Event>,
    ) {
        let Some(zone) = cast.def.effect.zone.filter(|_| self.state.death.is_none()) else {
            return;
        };
        let harm = match (zone.kind, cast.roll) {
            (ZoneKind::Fire, Some(roll)) => roll.roll(&mut self.state.rng, cast.reroll),
            _ => 0,
        };
        let lasts = u64::from(zone.turns) * TURN_SECONDS;
        let by = cast.by;
        let laid = (zone.kind, by, harm, Some(cast.spell));
        self.lay_cells(laid, cells, lasts, events);
    }

    /// Lays a zone of `(kind, by, harm, spell)` for `seconds` on those of `cells` it may lie on.
    fn lay_cells(
        &mut self,
        (kind, by, harm, spell): (ZoneKind, ActorId, i32, Option<SpellId>),
        cells: Vec<Pos>,
        seconds: u64,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let level = self.state.current();
        let lies = |p: &Pos| {
            !level.is_opaque(&content, *p)
                && (!kind.on_ground()
                    || (level.is_walkable(&content, *p) && !level.is_open_water(&content, *p)))
        };
        let cells: BTreeSet<Pos> = cells.into_iter().filter(lies).collect();
        if cells.is_empty() {
            return;
        }
        let light = (kind == ZoneKind::Fire)
            .then(|| content.spell_type(content.damage_type_id("fire")))
            .flatten()
            .and_then(|rule| rule.fire)
            .map(|(light, _)| light);
        if kind == ZoneKind::Fire {
            for &p in &cells {
                self.burst_ice(p, 0, Some(BurstIce::Melts), events);
            }
        }
        let until = self.state.time + Duration::seconds(seconds);
        self.state.current_mut().lay_zone(Zone {
            kind,
            cells,
            until,
            by,
            harm,
            light,
            spell,
        });
    }

    /// A hex that does nothing but lay its zone (Mirk, Slick): it flies as a burst or a line
    /// does, and lies where it goes off whether or not it finds anyone.
    pub(super) fn cast_zone(
        &mut self,
        cast: &Cast<'_>,
        (at, range, radius): (Pos, i32, i32),
        events: &mut Vec<Event>,
    ) -> bool {
        if !self.can_loose_here() || !self.within_reach(at, range, "cast") {
            return false;
        }
        let here = self.player_pos();
        let cells = self.zone_cells(cast, (here, at, range), radius);
        if !self.pay_cast(cast) {
            return true;
        }
        if let Some(text) = &cast.def.look.message {
            self.say(text.clone());
        }
        let center = match cast.def.shape {
            Shape::Burst => self.burst_center(here, at, range),
            _ => here,
        };
        self.sound_at(cast.def.look.noise, center);
        if let Some(effect) = &cast.def.look.effect {
            events.push(Event::Spell {
                effect: effect.clone(),
                origin: here,
                shape: SpellShape::Burst { center, radius },
                cells: cells.clone(),
            });
        }
        self.lay_spell_zone(cast, cells, events);
        true
    }

    /// The dev action `zone:KIND@X,Y,R` ([`crate::action::Action::Zone`]): a zone of `kind`
    /// around `at`, within `radius` in its line of sight, by the player, for ten turns. A fire's
    /// harm is fixed. Takes no time and draws nothing.
    pub(super) fn zone_dev(
        &mut self,
        kind: ZoneKind,
        at: Pos,
        radius: i32,
        events: &mut Vec<Event>,
    ) -> Spent {
        if !self.can_loose_here() {
            return Spent::Nothing;
        }
        let cells: Vec<Pos> = self.burst(at, radius).into_iter().collect();
        let harm = if kind == ZoneKind::Fire { DEV_HARM } else { 0 };
        let by = self.state.player;
        let seconds = DEV_TURNS * TURN_SECONDS;
        self.lay_cells((kind, by, harm, None), cells, seconds, events);
        self.say(format!(
            "A {} zone lies around {},{} (dev).",
            kind.id(),
            at.x,
            at.y
        ));
        Spent::NoTime
    }

    /// Actor `id`'s turn starts where it stands: a fire burns it, a cloud poisons it.
    pub(super) fn zone_turn_start(&mut self, id: ActorId, events: &mut Vec<Event>) {
        let Some(pos) = self.zoned(id) else {
            return;
        };
        match self.zone_here(pos) {
            Some((ZoneKind::Fire, zone)) => self.zone_burns(id, pos, zone, events),
            Some((ZoneKind::Fouled, zone)) => self.zone_fouls(id, zone, events),
            _ => {}
        }
    }

    /// Actor `id` stepped from `from` to `to`: a fire there burns it, slick ground under either
    /// end of the step may trip it.
    pub(super) fn zone_tread(
        &mut self,
        id: ActorId,
        (from, to): (Pos, Pos),
        events: &mut Vec<Event>,
    ) {
        if self.zoned(id).is_none() {
            return;
        }
        let onto = self.zone_here(to);
        if let Some((ZoneKind::Fire, zone)) = onto {
            self.zone_burns(id, to, zone, events);
        }
        let mut ends = [onto, self.zone_here(from)].into_iter().flatten();
        if let Some((_, zone)) = ends.find(|(k, _)| *k == ZoneKind::Slick) {
            self.zone_trips(id, zone, events);
        }
    }

    /// Where actor `id` stands, if a zone lies anywhere on the current level (`None`: nothing
    /// for it here, at the cost of a look at an empty list).
    fn zoned(&self, id: ActorId) -> Option<Pos> {
        if self.state.current().zones.is_empty() {
            return None;
        }
        self.state.actors.get(&id).map(|a| a.pos)
    }

    /// The zone on `p` of the current level now, as its hooks read it.
    fn zone_here(&self, p: Pos) -> Option<(ZoneKind, ZoneOf)> {
        let zone = self.state.current().zone_at(p, self.state.time)?;
        let of = ZoneOf {
            by: zone.by,
            harm: zone.harm,
            spell: zone.spell,
        };
        Some((zone.kind, of))
    }

    /// Fire at `at` burns actor `id` as a spell of its caster's: Evasion, resistance, the blow
    /// said, its kill the caster's, then the fire type's rule on one it harmed.
    fn zone_burns(&mut self, id: ActorId, at: Pos, zone: ZoneOf, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let kind = content.damage_type_id(ZoneKind::Fire.damage_type().unwrap_or_default());
        let mut made = Vec::new();
        let damage = Damage::new(zone.harm, kind);
        self.spell_hurts(Caster::Actor(zone.by), id, (at, false), damage, &mut made);
        let harmed = (made.iter()).any(|e| {
            matches!(
                e,
                Event::Attacked {
                    kind: AttackKind::Spell,
                    strike: Some(Strike::Hit { .. }),
                    ..
                }
            )
        });
        let spell = zone.spell.and_then(|s| content.spell_of(s));
        let (noun, verb) = spell.map_or(("fire", "burns"), |s| {
            (s.look.noun.as_str(), s.look.verb.as_str())
        });
        self.say_spell((noun, verb), made, events);
        if id == self.state.player
            && id == zone.by
            && let (Some(death), Some(spell)) = (&mut self.state.death, zone.spell)
        {
            death.by = None;
            death.cause = Some(Cause::Spell(spell));
        }
        if harmed {
            self.spell_type_lands(zone.by, kind, &[id], events);
        }
    }

    /// A foul cloud poisons actor `id` by the poison type's rule, which a poison-immune creature
    /// takes nothing of. A creature the player's cloud reaches turns on them, as their spells do.
    fn zone_fouls(&mut self, id: ActorId, zone: ZoneOf, events: &mut Vec<Event>) {
        let kind =
            (self.content).damage_type_id(ZoneKind::Fouled.damage_type().unwrap_or_default());
        if zone.by == self.state.player && id != zone.by {
            self.provoke(id);
        }
        self.spell_type_lands(zone.by, kind, &[id], events);
    }

    /// A step on slick ground: actor `id` makes its check, or slips, Staggered. Nothing
    /// incorporeal presses on the ground.
    fn zone_trips(&mut self, id: ActorId, zone: ZoneOf, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let Some(actor) = self.state.actors.get(&id) else {
            return;
        };
        let Some(staggered) = content.status_id("staggered") else {
            return;
        };
        if content.incorporeal(actor.creature).is_some() {
            return;
        }
        let modifier = actor.effective_stats(&content).modifier(Stat::Dexterity)
            + actor.talent_mods(&content).check(Stat::Dexterity);
        let pos = actor.pos;
        if check::check(&mut self.state.rng, modifier, SLICK_DIFFICULTY) {
            return;
        }
        if id == self.state.player {
            self.say("You slip on the slick ground!");
        } else if self.perception.sees_actors_at(pos) {
            let who = super::talk::capitalized(&self.called(&self.state.actors[&id]));
            self.say(format!("{who} slips."));
        }
        self.afflict(id, staggered, 1, Some(zone.by), events);
    }

    /// The end of a turn: the current level's zones past their end are dropped, quietly.
    pub(super) fn zones_fade(&mut self) {
        let now = self.state.time;
        let level = self.state.current_mut();
        if !level.zones.is_empty() {
            level.zones.retain(|z| z.until > now);
        }
    }

    /// Dev: the current level's zones, in the order laid.
    pub fn zones(&self) -> &[Zone] {
        &self.state.current().zones
    }
}

/// What a zone's hooks read of it, copied out so the world may change under them.
#[derive(Clone, Copy, Debug)]
struct ZoneOf {
    by: ActorId,
    harm: i32,
    spell: Option<SpellId>,
}
