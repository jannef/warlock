//! The one cast path (spec §8 "Spells", the magic design §5): a spell of `data/spells/` cast by
//! the player ([`Action::Cast`], `use:arcane_bolt:12,5`), from its cost to what it does.
//!
//! - **Cost** ([`Actor::spell_cost`], the one hook): the tier table's price
//!   ([`SpellRules::mana`]: base by tier × overreach × shape × resolution × the armour worn's
//!   `spell_cost_percent`, rounded once), then Archmage's `spell_mana_percent`
//!   ([`Actor::spell_base`], what [`World::spend_mana`] is given), then the talents' cut there
//!   (Frugal caster). Spent only once the cast is sure to go off.
//! - **Reach**: Willpower grows a spell's range, a burst's radius, and a cone's or a line's
//!   length up to the spell's own caps ([`Actor::spell_reach`]); Presence lengthens a buff's or
//!   a hex's status ([`SpellRules::lengthened`]).
//! - **Dice**: the spell's at the caster's level (one die more per `per_levels` above the tier's
//!   comfortable level, never past `max_dice`), then the talents' (Empower's die, Archmage's
//!   per die), and the roll is held to the spell's cap ([`SpellRules::cap_of`]): nothing lifts
//!   a spell past it.
//! - **Resolution**: `area` (a burst, a cone, a line) strikes everyone in it with one roll, no
//!   roll to hit, Evasion halving it ([`World::spell_hurts`]) and resistances taking their
//!   share; `auto` (a ray or a bolt) strikes the first creature on its way, never missing;
//!   `attack` makes the 2d12 attack roll against Defence (the caster's Dexterity to hit, as a
//!   missile is aimed; Deflect missiles counts), threat and critical as a blow's (a critical
//!   rolls the damage again, still held to the cap); `touch` the same roll against one creature
//!   next to the caster. `auto` and `area` never critical.
//! - **Shapes**: a burst flies and goes off as a scroll's does ([`World::burst_center`],
//!   [`World::cast_burst`]); a line runs as a scroll's bolt, through everyone on it
//!   ([`World::cast_bolt`]); a cone fans out from the caster ([`super::spell::cone_cells`],
//!   drawn as a burst around the caster over its cells); a ray or a bolt flies to the first
//!   creature on its line; a touch reaches one cell; a hex falls on the creature the caster sees
//!   at the cell aimed at; self is the caster.
//!
//! A cast is refused, taking no time and no mana, without the spell, without the mana, without
//! the cell it asks, and where it cannot go (out of range, no one there to hex or touch, a
//! lightning call under a roof, nothing to cure, not hurt). Its blows are the caster's: its
//! kills earn them their experience, and one that kills its caster is what killed them
//! ([`Cause::Spell`]). Determinism: nothing here draws at random but the spell's own rolls (the
//! attack roll, the dice), an area's one roll for all, in the order the shared code strikes
//! (ascending actor id); a spell with no roll draws nothing.
//!
//! [`Action::Cast`]: crate::action::Action::Cast

use super::spell::{Caster, burst_cells, capped, cone_cells};
use super::talk::capitalized;
use super::{Actor, ActorId, Cause, Spent, World};
use crate::ability::{Ability, Unavailable};
use crate::action::position_text;
use crate::action::{AttackKind, Event, SpellShape};
use crate::combat::{self, Attack, Profile, Strike};
use crate::content::{
    ActionKind, Bolt, Burst, Content, Resolution, Shape, SpellData, SpellPurpose, SpellRules,
    Stance,
};
use crate::content::{ObjectId, SpellId};
use crate::damage::{Damage, Kinds};
use crate::dice::Dice;
use crate::geom::Pos;
use crate::stats::Stat;
use std::cell::Cell;

/// How far a spell reaches as one actor casts it, Willpower counted (each `None` where the
/// spell has no such reach).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Reach {
    /// The farthest cell it may be aimed at: a ray's, bolt's, or burst's range, a cone's or a
    /// line's length, a touch's 1.
    pub range: Option<i32>,
    /// A burst's radius.
    pub radius: Option<i32>,
}

impl Actor {
    /// The percent more mana a spell costs it for what it wears (spec §8 "Spells"): the
    /// `spell_cost_percent` of every piece worn or held, added up.
    pub fn armour_spell_cost(&self, content: &Content) -> u32 {
        (self.equipment.items().into_iter())
            .filter_map(|(_, item)| self.inventory.get(item))
            .map(|s| u32::from(content.object(s.object).spell_cost_percent))
            .sum()
    }

    /// What `spell` asks of it at `caster_level` before its talents cut the price: the tier
    /// table's price ([`SpellRules::mana`], with its armour), then Archmage's
    /// `spell_mana_percent`, at least 1. What a cast gives [`World::spend_mana`], which takes
    /// [`Actor::mana_cost`] (Frugal caster) off it.
    pub fn spell_base(&self, content: &Content, spell: &SpellData, caster_level: i32) -> i32 {
        let Some(rules) = content.spell_rules() else {
            return 1;
        };
        let price = rules.mana(spell, caster_level, self.armour_spell_cost(content));
        let price = u16::try_from(price).unwrap_or(u16::MAX);
        self.talent_mods(content).spell_mana(price)
    }

    /// What `spell` costs it at `caster_level` (the one hook, spec §8 "Spells"):
    /// [`Actor::spell_base`] through [`Actor::mana_cost`], what a cast spends and what the
    /// abilities menu's "too little mana" compares with.
    pub fn spell_cost(&self, content: &Content, spell: &SpellData, caster_level: i32) -> i32 {
        self.mana_cost(content, self.spell_base(content, spell, caster_level))
    }

    /// How far `spell` reaches as it casts it: the spell's own reach grown by its Willpower
    /// (a cell per `reach_step` modifier points) up to the spell's caps.
    pub fn spell_reach(&self, content: &Content, spell: &SpellData) -> Reach {
        let Some(rules) = content.spell_rules() else {
            return Reach::default();
        };
        let will = self.effective_stats(content).modifier(Stat::Willpower);
        let grow = |base: Option<u8>, cap: Option<u8>| {
            let base = i32::from(base?);
            Some(rules.grown(base, cap.map_or(base, i32::from), will))
        };
        let range = match spell.shape {
            Shape::Touch => Some(1),
            Shape::Myself => None,
            Shape::Cone | Shape::Line => grow(spell.length, spell.max_length),
            Shape::Ray | Shape::Bolt | Shape::Burst => grow(spell.range, spell.max_range),
        };
        Reach {
            range,
            radius: grow(spell.radius, spell.max_radius),
        }
    }
}

/// A spell's damage dice as a caster at `caster_level` with talents `mods` rolls them: the
/// spell's at that level, then Empower's and Archmage's. The roll is held to the cap after.
fn damage_dice(
    rules: &SpellRules,
    spell: &SpellData,
    caster_level: i32,
    mods: &crate::talent::TalentMods,
) -> Option<Dice> {
    let comfortable = rules.comfortable[usize::from(spell.tier)];
    let dice = spell.damage?.at(caster_level, comfortable);
    Some(mods.spell_damage(dice))
}

impl World {
    /// `use:ID[:X,Y]` ([`crate::action::Action::Cast`]): the player casts the spell of id
    /// `name`, at cell `at` if it asks for one (module docs).
    pub(super) fn cast_spell(
        &mut self,
        name: &str,
        at: Option<Pos>,
        events: &mut Vec<Event>,
    ) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let me = self.state.player();
        let spell = content.spell_id(name);
        let (Some(spell), Some(rules)) = (spell, content.spell_rules()) else {
            self.say("You have no such ability.");
            return Spent::Nothing;
        };
        let def = content.spell(spell);
        let ability = Ability::Spell(spell);
        if !ability.had_by(me, &content) {
            self.say("You have no such ability.");
            return Spent::Nothing;
        }
        if let Some(why) = ability.unavailable(me, &content) {
            self.say(match why {
                Unavailable::While(status) => format!(
                    "You are {}: you cannot cast now.",
                    content.status(status).name.to_lowercase()
                ),
                other => format!("You cannot: {}.", other.reason(&content)),
            });
            return Spent::Nothing;
        }
        let level = self.caster_level(player);
        let me = self.state.player();
        if me.mana < me.spell_cost(&content, def, level) {
            self.say("You have too little mana.");
            return Spent::Nothing;
        }
        let Some(at) = self.cast_aim(def, at, &format!("use:{name}")) else {
            return Spent::Nothing;
        };
        let me = self.state.player();
        let cast = Cast {
            spell,
            def,
            rules,
            level,
            pay: Pay::Mana(me.spell_base(&content, def, level)),
            reach: me.spell_reach(&content, def),
            read: Cell::new(None),
        };
        let price = self.price_of(player, ActionKind::Cast) * i64::from(def.casts);
        if self.perform(&cast, at, events) {
            Spent::Turn(price)
        } else {
            Spent::Nothing
        }
    }

    /// The cell a cast of `def` goes to: `Some(None)` for a spell on oneself, `Some(at)` for an
    /// aimed one given its cell; `None` (said, with `usage` the action text that would name
    /// one) for an aimed spell without it.
    fn cast_aim(&mut self, def: &SpellData, at: Option<Pos>, usage: &str) -> Option<Option<Pos>> {
        match (def.aimed(), at) {
            (true, None) => {
                self.say(format!("Cast it where? Name a cell: `{usage}:12,5`."));
                None
            }
            (true, at) => Some(at),
            (false, _) => Some(None),
        }
    }

    /// The player reads the scroll in inventory stack `item`, which casts `spell` at caster
    /// level `level` (spec §8 "Scrolls"): the cast path's, but paid by the scroll, not mana,
    /// and never refused for what the player knows or holds. Whether it went off (`false`: the
    /// scroll is kept, no time).
    pub(super) fn read_cast(
        &mut self,
        item: usize,
        (spell, level): (SpellId, u8),
        at: Option<Pos>,
        events: &mut Vec<Event>,
    ) -> bool {
        let content = self.content.clone();
        let (def, Some(rules)) = (content.spell(spell), content.spell_rules()) else {
            self.say("Nothing happens.");
            return false;
        };
        let Some(at) = self.cast_aim(def, at, &format!("read:{}", position_text(item))) else {
            return false;
        };
        let cast = Cast {
            spell,
            def,
            rules,
            level: i32::from(level),
            pay: Pay::Scroll(item),
            reach: self.state.player().spell_reach(&content, def),
            read: Cell::new(None),
        };
        self.perform(&cast, at, events)
    }

    /// Casts `cast` at `at` by its kind (module docs); whether it went off. A scroll read is
    /// said kept by Spell recall afterwards, and a death by it is put down to the scroll.
    fn perform(&mut self, cast: &Cast<'_>, at: Option<Pos>, events: &mut Vec<Event>) -> bool {
        let done = match cast.def.kind {
            SpellPurpose::Buff => self.cast_buff(cast, events),
            SpellPurpose::Heal => self.cast_heal(cast, at),
            SpellPurpose::Cleanse => self.cleanse(cast, events),
            SpellPurpose::Hex => self.cast_hex(cast, at, events),
            SpellPurpose::Damage => self.cast_damage(cast, at, events),
            SpellPurpose::Utility => {
                self.say("Nothing happens.");
                false
            }
        };
        if let Some((object, kept)) = cast.read.get() {
            if kept && self.state.death.is_none() {
                self.say(super::read::KEPT);
            }
            if let Some(death) = &mut self.state.death {
                death.by = None;
                death.cause = Some(Cause::Scroll(object));
            }
        }
        done
    }

    /// Pays for `cast` as it is sure to go off: the caster's mana, or the scroll read (used
    /// up, unless Spell recall keeps it, and said as read).
    fn pay_cast(&mut self, cast: &Cast<'_>) {
        match cast.pay {
            Pay::Mana(mana) => {
                self.spend_mana(self.state.player, mana);
            }
            Pay::Scroll(item) => {
                let Some((scroll, kept)) = self.use_up(item) else {
                    return;
                };
                self.say_read(&scroll);
                cast.read.set(Some((scroll.object, kept)));
            }
        }
    }

    /// Presence's degrees of `status` for the player's buff or hex of `severity`.
    fn lengthened(&self, rules: &SpellRules, severity: u16) -> u16 {
        let presence = (self.state.player())
            .effective_stats(&self.content)
            .modifier(Stat::Presence);
        rules.lengthened(severity, presence)
    }

    /// A buff: its status on the caster, lengthened by Presence.
    fn cast_buff(&mut self, cast: &Cast<'_>, events: &mut Vec<Event>) -> bool {
        let Some((status, severity)) = cast.def.effect.status else {
            return false;
        };
        let player = self.state.player;
        let severity = self.lengthened(cast.rules, severity);
        self.pay_cast(cast);
        self.afflict(player, status, severity, Some(player), events);
        true
    }

    /// A heal: its dice, held to its cap, back to the caster (self), or to the one it touches
    /// (a touch: the cell next to the caster, or their own).
    fn cast_heal(&mut self, cast: &Cast<'_>, at: Option<Pos>) -> bool {
        let player = self.state.player;
        let target = match (cast.def.shape, at) {
            (Shape::Touch, Some(at)) if at != self.player_pos() => {
                if self.player_pos().chebyshev(at) > 1 {
                    self.say("You can only touch someone next to you.");
                    return false;
                }
                let level = self.state.current().id.clone();
                match self.state.actor_at(&level, at).map(|a| a.id) {
                    Some(id) => id,
                    None => {
                        self.say("There is no one there to touch.");
                        return false;
                    }
                }
            }
            _ => player,
        };
        let content = self.content.clone();
        let Some(actor) = self.state.actors.get(&target) else {
            return false;
        };
        if actor.hp >= actor.max_hp(&content) {
            self.say(if target == player {
                "You are not hurt.".to_string()
            } else {
                format!("{} is not hurt.", capitalized(&self.called(actor)))
            });
            return false;
        }
        let Some(heal) = cast.def.effect.heal else {
            return false;
        };
        let comfortable = cast.rules.comfortable[usize::from(cast.def.tier)];
        self.pay_cast(cast);
        let rolled = heal
            .at(cast.level, comfortable)
            .roll(&mut self.state.rng)
            .max(0);
        let back = rolled.min(cast.rules.cap_of(cast.def));
        if target == player {
            self.heal_player(back);
            self.say("Your wounds close.");
        } else {
            let actor = self.state.actors.get_mut(&target).expect("there");
            let max = actor.max_hp(&content);
            actor.hp = (actor.hp + back).min(max).max(actor.hp);
            let who = capitalized(&self.called(&self.state.actors[&target]));
            self.say(format!("{who} looks better."));
        }
        true
    }

    /// A cleanse: the statuses it cures end, and with `diseases` every disease the player
    /// has, each said as it ends. Refused (said, `false`) when there is nothing to cure.
    fn cleanse(&mut self, cast: &Cast<'_>, events: &mut Vec<Event>) -> bool {
        let (cures, diseases) = (&cast.def.effect.cures, cast.def.effect.diseases);
        let player = self.state.player;
        let me = self.state.player();
        let ended: Vec<_> = (me.effects.iter())
            .filter(|e| cures.contains(&e.status))
            .copied()
            .collect();
        let sick: Vec<_> = if diseases {
            me.diseases.iter().map(|i| i.disease).collect()
        } else {
            Vec::new()
        };
        if ended.is_empty() && sick.is_empty() {
            self.say("There is nothing to cure.");
            return false;
        }
        self.pay_cast(cast);
        self.say("A cleansing warmth runs through you.");
        let from = events.len();
        for e in &ended {
            self.player_mut().effects.remove(e.status);
        }
        self.ended(player, &ended, events);
        for disease in sick {
            self.end_disease(player, disease, events);
        }
        let content = self.content.clone();
        let said: Vec<String> = (events[from..].iter())
            .filter_map(|e| match *e {
                Event::Disease {
                    disease,
                    news: crate::action::DiseaseNews::Recovered,
                    ..
                } => Some(content.disease(disease).recovered.clone()),
                Event::Status {
                    status,
                    began: false,
                    ..
                } => content.status(status).end.clone(),
                _ => None,
            })
            .collect();
        for text in said {
            self.say(text);
        }
        true
    }

    /// A hex: its status, lengthened by Presence, on the creature the player sees at `at`
    /// within reach, that means them harm.
    fn cast_hex(&mut self, cast: &Cast<'_>, at: Option<Pos>, events: &mut Vec<Event>) -> bool {
        let (Some(at), Some((status, severity))) = (at, cast.def.effect.status) else {
            return false;
        };
        if cast.def.shape == Shape::Burst {
            return self.cast_area_hex(cast, at, (status, severity), events);
        }
        let Some(target) = self.hex_target(at, cast.reach.range.unwrap_or(1)) else {
            return false;
        };
        let player = self.state.player;
        let severity = self.lengthened(cast.rules, severity);
        self.pay_cast(cast);
        self.afflict(target, status, severity, Some(player), events);
        let who = capitalized(&self.called(&self.state.actors[&target]));
        let content = self.content.clone();
        let word = (content.status(status).seen.clone()).unwrap_or_else(|| "hexed".into());
        self.say(format!("{who} is {word}."));
        true
    }

    /// A hex over an area (Sleep, Web): it flies as a burst does and goes off around the
    /// first creature in its way or the cell aimed at, and its status, lengthened by Presence,
    /// falls on everyone it reaches but the caster, friend or foe; each the player sees is
    /// named. It goes off whether or not it finds anyone.
    fn cast_area_hex(
        &mut self,
        cast: &Cast<'_>,
        at: Pos,
        (status, severity): (crate::content::StatusId, u16),
        events: &mut Vec<Event>,
    ) -> bool {
        let range = cast.reach.range.unwrap_or(1);
        if !self.can_loose_here() || !self.within_reach(at, range, "cast") {
            return false;
        }
        let player = self.state.player;
        let here = self.player_pos();
        let severity = self.lengthened(cast.rules, severity);
        self.pay_cast(cast);
        if let Some(text) = &cast.def.look.message {
            self.say(text.clone());
        }
        let center = self.burst_center(here, at, range);
        self.sound_at(cast.def.look.noise, center);
        let level = self.state.current();
        let opaque = |p: Pos| p != center && level.is_opaque(&self.content, p);
        let cells = burst_cells(center, cast.reach.radius.unwrap_or(1), opaque);
        if let Some(effect) = &cast.def.look.effect {
            events.push(Event::Spell {
                effect: effect.clone(),
                origin: here,
                shape: SpellShape::Burst {
                    center,
                    radius: cast.reach.radius.unwrap_or(1),
                },
                cells: cells.clone(),
            });
        }
        let level = self.state.current_level.clone();
        let caught: Vec<ActorId> = (self.state.actors.values())
            .filter(|a| a.level == level && a.id != player && cells.contains(&a.pos))
            .map(|a| a.id)
            .collect();
        let content = self.content.clone();
        let word = (content.status(status).seen.clone()).unwrap_or_else(|| "hexed".into());
        for id in caught {
            self.afflict(id, status, severity, Some(player), events);
            let pos = self.state.actors[&id].pos;
            if self.perception.sees_actors_at(pos) {
                let who = capitalized(&self.called(&self.state.actors[&id]));
                self.say(format!("{who} is {word}."));
            }
        }
        true
    }

    /// The creature a hex at `at` within `range` falls on: one the player sees there, that is
    /// not friendly to them. `None` (said) otherwise.
    fn hex_target(&mut self, at: Pos, range: i32) -> Option<ActorId> {
        if !self.within_reach(at, range, "hex") {
            return None;
        }
        let level = self.state.current().id.clone();
        let target = (self.state.actor_at(&level, at)).map(|a| a.id);
        let Some(target) = target.filter(|_| self.perception.sees_actors_at(at)) else {
            self.say("You see no one there to hex.");
            return None;
        };
        if self.stance(self.state.player, target) != Stance::Hostile {
            let who = capitalized(&self.called(&self.state.actors[&target]));
            self.say(format!("{who} means you no harm."));
            return None;
        }
        Some(target)
    }

    /// Whether the player stands under the open sky: a level under the sky, and no roof over
    /// their cell (whatever the clouds and the hour).
    fn under_open_sky(&self) -> bool {
        let level = self.state.current();
        let here = self.player_pos();
        level.kind.is_sky_exposed()
            && (level.terrain.get(here)).is_some_and(|t| !self.content.terrain(*t).roofed)
    }

    /// A damage spell at `at`, by its shape (module docs); everyone it harms who lives suffers
    /// its status too. A death of the player by it is put down to the spell.
    fn cast_damage(&mut self, cast: &Cast<'_>, at: Option<Pos>, events: &mut Vec<Event>) -> bool {
        let Some(at) = at else {
            return false;
        };
        let def = cast.def;
        let range = cast.reach.range.unwrap_or(1);
        if !self.can_loose_here() {
            return false;
        }
        if def.shape == Shape::Touch {
            if at == self.player_pos() || self.player_pos().chebyshev(at) > 1 {
                self.say("You can only touch someone next to you.");
                return false;
            }
            let level = self.state.current().id.clone();
            if self.state.actor_at(&level, at).is_none() {
                self.say("There is no one there to touch.");
                return false;
            }
        } else if !self.within_reach(at, range, "cast") {
            return false;
        }
        if def.open_sky && !self.under_open_sky() {
            self.say("You need the open sky above you for that.");
            return false;
        }
        let player = self.state.player;
        self.pay_cast(cast);
        let mods = self.state.player().talent_mods(&self.content).into_owned();
        let dice = damage_dice(cast.rules, def, cast.level, &mods).unwrap_or(Dice::of(0, 1));
        let cap = cast.rules.cap_of(def);
        if let Some(text) = &def.look.message {
            self.say(text.clone());
        }
        let here = self.player_pos();
        let effect = def.look.effect.clone().unwrap_or_default();
        let mut made = Vec::new();
        match def.shape {
            Shape::Burst => {
                let burst = Burst {
                    spell: effect,
                    radius: cast.reach.radius.unwrap_or(1),
                    damage: dice,
                    kind: def.damage_type,
                    blast: def.look.blast,
                    ice: def.look.ice,
                    cap: Some(cap),
                };
                let center = self.burst_center(here, at, range);
                self.sound_at(def.look.noise, center);
                self.cast_burst(Caster::Actor(player), (here, center), &burst, &mut made);
            }
            Shape::Line => {
                let bolt = Bolt {
                    spell: effect,
                    length: range,
                    damage: dice,
                    kind: def.damage_type,
                    cap: Some(cap),
                };
                let path = self.bolt_line(here, at, range);
                self.sound_at(def.look.noise, here);
                self.cast_bolt(Caster::Actor(player), here, &path, &bolt, &mut made);
            }
            Shape::Cone => {
                self.sound_at(def.look.noise, here);
                self.cast_cone((here, at, range), (&effect, dice, cap), def, &mut made);
            }
            Shape::Ray | Shape::Bolt | Shape::Touch => {
                self.sound_at(def.look.noise, here);
                self.cast_single((here, at, range), (&effect, dice, cap), def, &mut made);
            }
            Shape::Myself => {}
        }
        let reached: Vec<ActorId> = (made.iter())
            .filter_map(|e| match *e {
                Event::Attacked {
                    attacker,
                    target,
                    kind: AttackKind::Spell,
                    strike: Some(strike),
                    ..
                } if attacker == player && strike.is_hit() => Some(target),
                _ => None,
            })
            .collect();
        self.say_spell((&def.look.noun, &def.look.verb), made, events);
        if let Some((status, severity)) = def.effect.status {
            for id in reached {
                if self.state.actors.contains_key(&id) && self.state.death.is_none() {
                    self.afflict(id, status, severity, Some(player), events);
                }
            }
        }
        if let Some(death) = &mut self.state.death
            && matches!(cast.pay, Pay::Mana(_))
        {
            death.by = None;
            death.cause = Some(Cause::Spell(cast.spell));
        }
        true
    }

    /// A cone from `here` toward `at`, `length` cells long: its effect drawn as a burst around
    /// the caster over its cells, then one roll of `dice` (held to `cap`) for everyone in it, in
    /// ascending actor id order, as an area's (Evasion halves it).
    fn cast_cone(
        &mut self,
        (here, at, length): (Pos, Pos, i32),
        (effect, dice, cap): (&str, Dice, i32),
        def: &SpellData,
        made: &mut Vec<Event>,
    ) {
        let level = self.state.current();
        let opaque = |p: Pos| level.is_opaque(&self.content, p);
        let cells = cone_cells(here, at, length, opaque);
        if !effect.is_empty() {
            made.push(Event::Spell {
                effect: effect.to_string(),
                origin: here,
                shape: SpellShape::Burst {
                    center: here,
                    radius: length,
                },
                cells: cells.clone(),
            });
        }
        let roll = capped(dice.roll(&mut self.state.rng), Some(cap));
        let level = self.state.current_level.clone();
        let caught: Vec<ActorId> = (self.state.actors.values())
            .filter(|a| a.level == level && cells.contains(&a.pos))
            .map(|a| a.id)
            .collect();
        let player = self.state.player;
        for id in caught {
            let damage = Damage::new(roll, def.damage_type);
            self.spell_hurts(Caster::Actor(player), id, (here, false), damage, made);
        }
    }

    /// A ray, a bolt, or a touch from `here` toward `at` (`reach` cells at most): it flies to
    /// the first creature on its line (a touch: the one at `at`), its effect drawn along the
    /// way, and strikes it as its resolution says: `auto` always, `attack` and `touch` by the
    /// attack roll against its Defence. One that finds no one strikes nothing.
    fn cast_single(
        &mut self,
        (here, at, reach): (Pos, Pos, i32),
        (effect, dice, cap): (&str, Dice, i32),
        def: &SpellData,
        made: &mut Vec<Event>,
    ) {
        let level = self.state.current_level.clone();
        let path: Vec<Pos> = if def.shape == Shape::Touch {
            vec![at]
        } else {
            let line = self.bolt_line(here, at, reach);
            let end = line
                .iter()
                .position(|&p| self.state.actor_at(&level, p).is_some());
            match end {
                Some(i) => line[..=i].to_vec(),
                None => line,
            }
        };
        if !effect.is_empty() && !path.is_empty() && def.shape != Shape::Touch {
            made.push(Event::Spell {
                effect: effect.to_string(),
                origin: here,
                shape: SpellShape::Bolt { path: path.clone() },
                cells: path.clone(),
            });
        }
        let Some(target) =
            (path.last()).and_then(|&p| self.state.actor_at(&level, p).map(|a| a.id))
        else {
            return;
        };
        let player = self.state.player;
        let source = (here, def.shape != Shape::Touch);
        let (amount, strike) = match def.resolution {
            Resolution::Attack | Resolution::Touch => {
                let dexterity = (self.state.player())
                    .effective_stats(&self.content)
                    .modifier(Stat::Dexterity);
                let attack = Attack {
                    damage: dice,
                    to_hit: dexterity,
                    threat: combat::DEFAULT_THREAT,
                    crit: combat::DEFAULT_CRIT,
                    strength: None,
                    kinds: Kinds::NONE,
                };
                let missile = def.shape != Shape::Touch;
                let defence = self.target_against(target, missile).defence;
                let against = combat::Target::bare(defence);
                let blow = combat::strike(&Profile::bare(&attack), &against, &mut self.state.rng);
                (blow.damage, blow.strike)
            }
            Resolution::Auto | Resolution::Area => (
                dice.roll(&mut self.state.rng).max(0),
                Strike::Hit { critical: false },
            ),
        };
        let damage = Damage::new(capped(amount, Some(cap)), def.damage_type);
        self.spell_strikes(player, target, source, (damage, strike), made);
    }
}

/// Who pays for a cast.
#[derive(Clone, Copy, Debug)]
enum Pay {
    /// The caster's mana: what it gives [`World::spend_mana`].
    Mana(i32),
    /// The scroll read from this inventory stack.
    Scroll(usize),
}

/// One cast's settled facts, handed to each kind's code.
struct Cast<'c> {
    spell: SpellId,
    def: &'c SpellData,
    rules: &'c SpellRules,
    /// The caster's caster level (a scroll's own, read).
    level: i32,
    pay: Pay,
    reach: Reach,
    /// A scroll read, once paid: its object, and whether Spell recall kept it.
    read: Cell<Option<(ObjectId, bool)>>,
}
