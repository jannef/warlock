//! The one cast path (spec §8 "Spells", the magic design §5): a spell of `data/spells/` cast by
//! the player ([`Action::Cast`], `use:arcane_bolt:12,5`), from its cost to what it does.
//!
//! - **Cost** ([`Actor::spell_cost`], the one hook): the cost table's price
//!   ([`crate::content::SpellRules::mana`]: the spell's level, at least 1, × shape × resolution × the armour
//!   worn's `spell_cost_percent`, rounded once; no caster level), then Archmage's
//!   `spell_mana_percent`
//!   ([`Actor::spell_base`], what [`World::spend_mana`] is given), then the talents' cut there
//!   (Frugal caster). Spent only once the cast is sure to go off.
//! - **Reach**: the spell's base range, radius, or length, climbed on its reach ladders by the
//!   caster's scores, then the reach hook's cells (School focus, of Farcasting), held only by the
//!   engine's limits ([`Actor::spell_reach`], `world/spell_rung.rs`); a buff's or a hex's degrees
//!   the same way on its degrees ladder ([`Actor::spell_degrees`]).
//! - **Dice**: the roll at the rung the caster reaches on the spell's damage or heal ladder (a
//!   scroll's or a wand's own rungs plus the reader's scroll or wand rungs), then the dice hook
//!   ([`Actor::spell_roll`]); drawn with the roll hook's rerolls, and the rider hook's statuses
//!   laid on every target harmed (`world/spell_mods.rs`). Nothing holds the roll (the spell
//!   scaling design, decision 16).
//! - **Resolution**: `area` (a burst, a cone, a line) strikes everyone in it with one roll, no
//!   roll to hit, Evasion halving it ([`World::spell_hurts`]) and resistances taking their
//!   share; `auto` (a ray or a bolt) strikes the first creature on its way, never missing;
//!   `attack` makes the 2d12 attack roll against Defence (the modifier of the spell's `aim` stat
//!   to hit, Dexterity unless it names another, as a missile is aimed; a scroll or a wand
//!   always Dexterity; Deflect missiles counts), threat and critical as a blow's (a critical
//!   rolls the damage again); `touch` the same roll against one creature
//!   next to the caster. `auto` and `area` never critical.
//! - **Shapes**: a burst flies and goes off as a scroll's does ([`World::burst_center`],
//!   [`World::cast_burst`]); a line runs as a scroll's bolt, through everyone on it
//!   ([`World::cast_bolt`]); a cone fans out from the caster ([`super::spell::cone_cells`],
//!   drawn as a burst around the caster over its cells); a ray or a bolt flies to the first
//!   creature on its line; a touch reaches one cell; a hex falls on the creature the caster sees
//!   at the cell aimed at; self is the caster. A spell that identifies (Identify) asks for a
//!   stack the caster carries instead of a cell (`use:identify:c`, [`Action::CastOn`]).
//! - **Explicit costs**: a spell with its own `mana` (Identify) costs that, armour and talents
//!   counted, whatever its level ([`crate::content::SpellRules::mana`]).
//!
//! - **Reflection**: an `attack` or `touch` roll missed by the target's Shield raises Spell
//!   reflection's trigger, and a spell turned back is cast again at its caster with the same
//!   roll, rerolls, to-hit, and riders (`reflect.rs`).
//!
//! A scroll read and a wand used cast through the same path at their own rungs, paid by the
//! scroll or a charge instead of mana ([`World::read_cast`], [`World::wand_cast`]).
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
//! [`Action::CastOn`]: crate::action::Action::CastOn

use super::reflect::{Shot, Turned};
use super::spell::{Caster, burst_cells, cone_cells};
use super::spell_aim::SpellAim;
use super::spell_rung::CastBy;
use super::talk::capitalized;
use super::weapon_ability::Fired;
use super::{Actor, ActorId, AttackSkills, Cause, Spent, World};
use crate::ability::{Ability, Unavailable};
use crate::action::position_text;
use crate::action::{AttackKind, Event, ReadTarget, SpellShape};
use crate::combat::{self, Attack, Category, Profile, Strike};
use crate::content::{
    Bolt, Burst, Content, Grip, Resolution, Shape, SpellData, SpellPurpose, Stance, StatusId,
    Trigger, merge_rider,
};
use crate::content::{Effect as ItemEffect, ObjectId, SpellId};
use crate::damage::{Damage, Kinds};
use crate::dice::Roll;
use crate::geom::Pos;
use crate::magic::Mods;
use std::cell::Cell;

/// How far a spell reaches as one actor casts it, its reach ladders climbed (each `None` where
/// the spell has no such reach).
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
        let worn = self.worn_spell_cost(content);
        if worn == 0 {
            return 0;
        }
        // Battle caster (spec §8 "Fighting styles"): its rank eases the armour's percent.
        let table = content.weapon_skills();
        let style = table.style_for(Grip::ArmouredSpell, &[]);
        let ease = u32::from(self.benefit(content, style).armour.min(100));
        worn * (100 - ease) / 100
    }

    /// The summed `spell_cost_percent` of what it wears, before Battle caster eases it: each
    /// piece's less its own `spell_ease` (Mage-wrought, a local affix), never below 0. Above 0,
    /// its spells that roll to hit are cast in the armoured grip; a piece eased to 0 no longer
    /// counts for it.
    pub fn worn_spell_cost(&self, content: &Content) -> u32 {
        (self.equipment.items().into_iter())
            .filter_map(|(_, item)| self.inventory.get(item))
            .map(|s| {
                let cost = i32::from(content.object(s.object).spell_cost_percent);
                if cost == 0 {
                    return 0;
                }
                let ease = Mods::local(s, content).get(ItemEffect::SpellEase);
                u32::try_from(cost - ease.max(0)).unwrap_or(0)
            })
            .sum()
    }

    /// What `spell` asks of it before its talents cut the price: the cost table's price
    /// ([`crate::content::SpellRules::mana`], with its armour), then the cost hook's percent
    /// (School focus's, [`Actor::spell_mods`]) and Archmage's, Empower's, and Brimming's
    /// (`spell_mana_percent`), at least 1. What a cast gives [`World::spend_mana`],
    /// which takes [`Actor::mana_cost`] (Frugal caster) off it. No caster level counts.
    pub fn spell_base(&self, content: &Content, spell: &SpellData) -> i32 {
        let Some(rules) = content.spell_rules() else {
            return 1;
        };
        let mods = self.talent_mods(content);
        let armour = self.armour_spell_cost(content);
        let price = rules.mana(spell, armour);
        let price = u16::try_from(price).unwrap_or(u16::MAX);
        mods.spell_mana_with(price, self.spell_mods(content, spell).cost)
    }

    /// What `spell` costs it (the one hook, spec §8 "Spells"): [`Actor::spell_base`] through
    /// [`Actor::mana_cost`], what a cast spends and what the abilities menu's "too little mana"
    /// compares with.
    pub fn spell_cost(&self, content: &Content, spell: &SpellData) -> i32 {
        self.mana_cost(content, self.spell_base(content, spell))
    }
}

impl World {
    /// `use:ID[:X,Y]`impl World {
    /// `use:ID[:X,Y]` ([`crate::action::Action::Cast`]) and `use:ID:c`
    /// ([`crate::action::Action::CastOn`]): the player casts the spell of id `name`, at the cell
    /// or on the stack `target` if it asks for one (module docs).
    pub(super) fn cast_spell(
        &mut self,
        name: &str,
        target: Option<ReadTarget>,
        events: &mut Vec<Event>,
    ) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let me = self.state.player();
        let spell = content.spell_id(name);
        let (Some(spell), Some(_)) = (spell, content.spell_rules()) else {
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
        let me = self.state.player();
        if me.mana < me.spell_cost(&content, def) {
            self.say("You have too little mana.");
            return Spent::Nothing;
        }
        let Some(target) = self.cast_aim(def, target, &format!("use:{name}")) else {
            return Spent::Nothing;
        };
        let me = self.state.player();
        let cast = Cast::settle(
            me,
            &content,
            (spell, def),
            Pay::Mana(me.spell_base(&content, def)),
            self.spell_aim(target),
        );
        let price = self.cast_price(player, def);
        // Hands full of shields may spoil it (`two_shields.rs`): the mana and the time spent.
        if let Pay::Mana(mana) = cast.pay
            && self.shields_fumble(mana)
        {
            return Spent::Turn(price);
        }
        if self.perform(&cast, target, events) {
            self.train_spell(def);
            self.practise_cast(&cast);
            Spent::Turn(price)
        } else {
            Spent::Nothing
        }
    }

    /// Where a cast of `def` goes: `Some(None)` for a spell on oneself, `Some(target)` for an
    /// aimed one given its cell, or one on a stack given its stack; `None` (said, with `usage`
    /// the action text that would name one) for one without what it asks.
    fn cast_aim(
        &mut self,
        def: &SpellData,
        target: Option<ReadTarget>,
        usage: &str,
    ) -> Option<Option<ReadTarget>> {
        match (def.aims_item(), def.aimed(), target) {
            (true, _, Some(ReadTarget::Item(on))) => Some(Some(ReadTarget::Item(on))),
            (true, _, _) => {
                self.say(format!(
                    "Cast it on what? Name a stack you carry: `{usage}:c`."
                ));
                None
            }
            (false, true, Some(ReadTarget::Cell(at))) => Some(Some(ReadTarget::Cell(at))),
            (false, true, _) => {
                self.say(format!("Cast it where? Name a cell: `{usage}:12,5`."));
                None
            }
            (false, false, _) => Some(None),
        }
    }

    /// The player reads the scroll in inventory stack `item`, which casts `spell` at its own
    /// `rungs` (spec §8 "Scrolls"): the cast path's, but paid by the scroll, not mana,
    /// and never refused for what the player knows or holds. Whether it went off (`false`: the
    /// scroll is kept, no time).
    pub(super) fn read_cast(
        &mut self,
        item: usize,
        (spell, rungs): (SpellId, u8),
        target: Option<ReadTarget>,
        events: &mut Vec<Event>,
    ) -> bool {
        let content = self.content.clone();
        let (def, Some(_)) = (content.spell(spell), content.spell_rules()) else {
            self.say("Nothing happens.");
            return false;
        };
        let usage = format!("read:{}", position_text(item));
        let Some(target) = self.cast_aim(def, target, &usage) else {
            return false;
        };
        let pay = (Pay::Scroll(item, rungs), self.spell_aim(target));
        let cast = Cast::settle(self.state.player(), &content, (spell, def), pay.0, pay.1);
        self.perform(&cast, target, events)
    }

    /// The player uses the wand in inventory stack `item`, which casts `spell` at its own
    /// `rungs` at cell `at` (spec §8 "Magic items": "Wands"): the cast path's, paid by a
    /// charge, not mana, and never refused for what the player knows or holds. Whether it went
    /// off (`false`: no charge used, no time).
    pub(super) fn wand_cast(
        &mut self,
        item: usize,
        (spell, rungs): (SpellId, u8),
        at: Pos,
        events: &mut Vec<Event>,
    ) -> bool {
        let content = self.content.clone();
        let (def, Some(_)) = (content.spell(spell), content.spell_rules()) else {
            self.say("Nothing happens.");
            return false;
        };
        let target = Some(ReadTarget::Cell(at));
        let pay = (Pay::Wand(item, rungs), self.spell_aim(target));
        let cast = Cast::settle(self.state.player(), &content, (spell, def), pay.0, pay.1);
        self.perform(&cast, target, events)
    }

    /// Casts `cast` at `at` by its kind (module docs); whether it went off. A scroll read is
    /// said kept by Spell recall afterwards, and a death by it is put down to the scroll.
    fn perform(
        &mut self,
        cast: &Cast<'_>,
        target: Option<ReadTarget>,
        events: &mut Vec<Event>,
    ) -> bool {
        let at = match target {
            Some(ReadTarget::Cell(at)) => Some(at),
            _ => None,
        };
        let done = match cast.def.kind {
            SpellPurpose::Buff => self.cast_buff(cast, events),
            SpellPurpose::Heal => self.cast_heal(cast, at, events),
            SpellPurpose::Cleanse => self.cleanse(cast, events),
            SpellPurpose::Hex => self.cast_hex(cast, at, events),
            SpellPurpose::Damage => self.cast_damage(cast, at, events),
            SpellPurpose::Utility if cast.def.effect.identify => {
                let on = match target {
                    Some(ReadTarget::Item(on)) => Some(on),
                    _ => None,
                };
                self.cast_identify(cast, on, events)
            }
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
            Pay::Scroll(item, _) => {
                let Some((scroll, kept)) = self.use_up(item) else {
                    return;
                };
                self.say_read(&scroll);
                cast.read.set(Some((scroll.object, kept)));
            }
            Pay::Wand(item, _) => self.use_charge(item),
        }
    }

    /// The player's cast `cast` went off (spec §8 "Weapon skills": "Schools of magic",
    /// "Fighting styles"; the spell scaling design §8.4, §10.2): one paid in mana practises its
    /// school once, as a blow practises its weapon skill, by the [`Outcome`] its kind filled (an
    /// attempt, and a success when it landed, weighed by the strongest creature it struck at,
    /// else the player's own level), then the caster style it was cast in (`spell_aim.rs`) by
    /// the same outcome, lands or not, as a blow's style. A spell on oneself out of armour that
    /// hinders casting trains no style. A scroll read or a wand used trains nothing.
    fn practise_cast(&mut self, cast: &Cast<'_>) {
        if !matches!(cast.pay, Pay::Mana(_)) || self.state.death.is_some() {
            return;
        }
        let player = self.state.player;
        let outcome = cast.outcome.get();
        let level = outcome.level.unwrap_or_else(|| self.level_of(player));
        for skill in [cast.skills.skill, cast.skills.style].into_iter().flatten() {
            self.practise(skill, outcome.landed, level);
        }
    }

    /// `cast` reached creature `other` (spec §8 "Weapon skills": "Schools of magic"): its
    /// [`Outcome`] weighs the strongest creature struck at, and `landed` (a harm done, a hex
    /// taken, a heal given, a buff or a cure or a utility's work done) makes it a success. The
    /// one place every kind reports a landing, and so where a cast paid in mana that `landed`
    /// raises the school abilities' trigger (spec §8 "Weapon abilities": `spell_lands`; the
    /// spell scaling design §8.5) with the cast's school and caster style, for `other` if it
    /// lives. A lookup when nothing has the trigger: no draw.
    fn spell_landed(
        &mut self,
        cast: &Cast<'_>,
        other: ActorId,
        landed: bool,
        events: &mut Vec<Event>,
    ) {
        let mut outcome = cast.outcome.get();
        let level = self.level_of(other);
        outcome.level = Some(outcome.level.map_or(level, |l| l.max(level)));
        outcome.landed |= landed;
        cast.outcome.set(outcome);
        if !landed
            || !matches!(cast.pay, Pay::Mana(_))
            || self.state.death.is_some()
            || !self.state.actors.contains_key(&other)
        {
            return;
        }
        let player = self.state.player;
        let skills = cast.skills;
        let fired = Fired {
            other,
            kind: AttackKind::Spell,
            skills,
            blow: None,
        };
        self.weapon_trigger(player, Trigger::SpellLands, fired, events);
    }

    /// A buff: its status on the caster, at the degrees its ladder and hooks give.
    fn cast_buff(&mut self, cast: &Cast<'_>, events: &mut Vec<Event>) -> bool {
        let (Some((status, _)), Some(severity)) = (cast.def.effect.status, cast.degrees) else {
            return false;
        };
        let player = self.state.player;
        self.pay_cast(cast);
        self.afflict(player, status, severity, Some(player), events);
        self.spell_landed(cast, player, true, events);
        true
    }

    /// Identify: the truth of inventory stack `on` (spec §8 "Spells"), as the scroll of identify
    /// told it ([`World::identify`]). Refused (said, `false`) without such a stack, or for one
    /// whose truth is known. A scroll read on its own stack identifies nothing more: the read
    /// named it.
    fn cast_identify(
        &mut self,
        cast: &Cast<'_>,
        on: Option<usize>,
        events: &mut Vec<Event>,
    ) -> bool {
        let Some(on) = on else {
            return false;
        };
        let Some(stack) = self.state.player().inventory.get(on) else {
            self.say("You carry no such thing.");
            return false;
        };
        if self.lore().knows(stack) {
            let what = self.lore().describe(&stack.single());
            self.say(format!("You already know what {what} is."));
            return false;
        }
        let before = self.state.player().inventory.len();
        self.pay_cast(cast);
        // A scroll used up moves the stacks after it a place up.
        let on = match cast.pay {
            Pay::Scroll(item, _) if on == item => return true,
            Pay::Scroll(item, _) if on > item && self.state.player().inventory.len() < before => {
                on - 1
            }
            _ => on,
        };
        if matches!(self.identify(on), super::Identified::Now(_)) {
            self.train_stat(crate::content::StatUse::Identified);
        }
        self.spell_landed(cast, self.state.player, true, events);
        true
    }

    /// A heal: its dice back to the caster (self), or to the one it touches
    /// (a touch: the cell next to the caster, or their own).
    fn cast_heal(&mut self, cast: &Cast<'_>, at: Option<Pos>, events: &mut Vec<Event>) -> bool {
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
        let Some(heal) = cast.roll else {
            return false;
        };
        self.pay_cast(cast);
        let back = heal.roll(&mut self.state.rng, cast.reroll).max(0);
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
        self.spell_landed(cast, target, back > 0, events);
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
        self.spell_landed(cast, player, true, events);
        true
    }

    /// A hex: its status, at the degrees its ladder and hooks give, on the creature the player
    /// sees at `at` within reach, that means them harm.
    fn cast_hex(&mut self, cast: &Cast<'_>, at: Option<Pos>, events: &mut Vec<Event>) -> bool {
        let (Some(at), Some((status, _)), Some(severity)) =
            (at, cast.def.effect.status, cast.degrees)
        else {
            return false;
        };
        if cast.def.shape == Shape::Burst {
            return self.cast_area_hex(cast, at, (status, severity), events);
        }
        let Some(target) = self.hex_target(at, cast.reach.range.unwrap_or(1)) else {
            return false;
        };
        let player = self.state.player;
        self.pay_cast(cast);
        self.afflict(target, status, severity, Some(player), events);
        let who = capitalized(&self.called(&self.state.actors[&target]));
        let content = self.content.clone();
        let word = (content.status(status).seen.clone()).unwrap_or_else(|| "hexed".into());
        self.say(format!("{who} is {word}."));
        self.spell_landed(cast, target, self.suffers(target, status), events);
        true
    }

    /// A hex over an area (Sleep, Web): it flies as a burst does and goes off around the
    /// first creature in its way or the cell aimed at, and its status, at its degrees, falls on
    /// everyone it reaches but the caster, friend or foe; each the player sees is
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
            self.spell_landed(cast, id, self.suffers(id, status), events);
        }
        true
    }

    /// Whether actor `id` suffers status `status` now (a hex that took on it).
    fn suffers(&self, id: ActorId, status: StatusId) -> bool {
        (self.state.actors.get(&id)).is_some_and(|a| a.effects.get(status).is_some())
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
        let roll = cast.roll.unwrap_or_default();
        let reroll = cast.reroll;
        if let Some(text) = &def.look.message {
            self.say(text.clone());
        }
        let here = self.player_pos();
        let effect = def.look.effect.clone().unwrap_or_default();
        let mut made = Vec::new();
        let mut turned = None;
        match def.shape {
            Shape::Burst => {
                let burst = Burst {
                    spell: effect,
                    radius: cast.reach.radius.unwrap_or(1),
                    damage: roll,
                    reroll,
                    kind: def.damage_type,
                    blast: def.look.blast,
                    ice: def.look.ice,
                };
                let center = self.burst_center(here, at, range);
                self.sound_at(def.look.noise, center);
                self.cast_burst(Caster::Actor(player), (here, center), &burst, &mut made);
            }
            Shape::Line => {
                let bolt = Bolt {
                    spell: effect,
                    length: range,
                    damage: roll,
                    reroll,
                    kind: def.damage_type,
                };
                let path = self.bolt_line(here, at, range);
                self.sound_at(def.look.noise, here);
                self.cast_bolt(Caster::Actor(player), here, &path, &bolt, &mut made);
            }
            Shape::Cone => {
                self.sound_at(def.look.noise, here);
                self.cast_cone((here, at, range), (&effect, roll, reroll), def, &mut made);
            }
            Shape::Ray | Shape::Bolt | Shape::Touch => {
                self.sound_at(def.look.noise, here);
                let own = matches!(cast.pay, Pay::Mana(_));
                let aim = (here, at, range);
                let by = (def, own.then_some(cast.skills));
                turned = self.cast_single(aim, (&effect, roll, reroll), by, &mut made);
            }
            Shape::Myself => {}
        }
        self.spell_lands(player, (def, &cast.riders), made, Some(cast), events);
        // Spell reflection: the spell a shield blocked goes back at its caster (`reflect.rs`),
        // its riders the caster's.
        if let Some(turned) = turned {
            let mut back = Vec::new();
            self.turn_back(&turned, &mut back);
            self.spell_lands(turned.reflector, (def, &cast.riders), back, None, events);
        }
        if let Some(death) = &mut self.state.death
            && !matches!(cast.pay, Pay::Scroll(..))
        {
            death.by = None;
            death.cause = Some(Cause::Spell(cast.spell));
        }
        true
    }

    /// Says what `by`'s cast `def` did (`made`, its events) and gives everyone it harmed who
    /// lives its status and the cast's `riders` (the rider hook; a status of both merged). The
    /// player's own `cast` then hears of each creature it struck at, harmed or missed, in the
    /// order struck (an area's in ascending actor id; [`World::spell_landed`]).
    fn spell_lands(
        &mut self,
        by: ActorId,
        (def, riders): (&SpellData, &[(StatusId, u16)]),
        made: Vec<Event>,
        cast: Option<&Cast<'_>>,
        events: &mut Vec<Event>,
    ) {
        let struck: Vec<(ActorId, bool)> = (made.iter())
            .filter_map(|e| match *e {
                Event::Attacked {
                    attacker,
                    target,
                    kind: AttackKind::Spell,
                    strike: Some(strike),
                    ..
                } if attacker == by => Some((target, strike.is_hit())),
                _ => None,
            })
            .collect();
        let reached: Vec<ActorId> = (struck.iter())
            .filter(|&&(_, hit)| hit)
            .map(|&(id, _)| id)
            .collect();
        self.say_spell((&def.look.noun, &def.look.verb), made, events);
        let mut laid: Vec<(StatusId, u16)> = def.effect.status.into_iter().collect();
        for &(status, severity) in riders {
            merge_rider(&mut laid, status, severity);
        }
        for id in reached {
            for &(status, severity) in &laid {
                if self.state.actors.contains_key(&id) && self.state.death.is_none() {
                    self.afflict(id, status, severity, Some(by), events);
                }
            }
        }
        if let Some(cast) = cast {
            for (target, hit) in struck {
                self.spell_landed(cast, target, hit, events);
            }
        }
    }

    /// A cone from `here` toward `at`, `length` cells long: its effect drawn as a burst around
    /// the caster over its cells, then one roll of `roll` (with `reroll`) for everyone in it,
    /// in ascending actor id order, as an area's (Evasion halves it).
    fn cast_cone(
        &mut self,
        (here, at, length): (Pos, Pos, i32),
        (effect, roll, reroll): (&str, Roll, u32),
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
        let roll = roll.roll(&mut self.state.rng, reroll);
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
    /// attack roll against its Defence, aimed with its `aim` stat when it is the caster's own
    /// spell (`own`: the school and the style it is cast in), else (a scroll, a wand) with
    /// Dexterity. One that finds no one strikes
    /// nothing. An attack roll missed by the target's Shield may be turned back (Spell
    /// reflection, `reflect.rs`): what to cast back, if so.
    fn cast_single<'s>(
        &mut self,
        (here, at, reach): (Pos, Pos, i32),
        (effect, roll, reroll): (&'s str, Roll, u32),
        (def, own): (&'s SpellData, Option<AttackSkills>),
        made: &mut Vec<Event>,
    ) -> Option<Turned<'s>> {
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
        let target = (path.last()).and_then(|&p| self.state.actor_at(&level, p).map(|a| a.id))?;
        let player = self.state.player;
        let me = self.state.player();
        let to_hit = if own.is_some() {
            me.spell_to_hit(&self.content, def)
        } else {
            me.item_spell_to_hit(&self.content)
        };
        // The spell's school and the caster style it is cast in (spec §8 "Weapon skills",
        // "Fighting styles"): the caster's own spells that roll to hit only, as its aim is.
        let skills = (own.filter(|_| def.rolls_to_hit())).unwrap_or_default();
        // Baleful's wider threat (`spell_threat`): an own cast only, as its aim.
        let threat = if own.is_some() {
            me.spell_threat(&self.content)
        } else {
            combat::DEFAULT_THREAT
        };
        let shot = Shot {
            def,
            effect,
            dice: roll,
            reroll,
            to_hit,
            threat,
            skills,
        };
        let source = (here, def.shape != Shape::Touch);
        let strike = self.spell_attack(player, target, source, shot, made);
        if strike
            != (Strike::Missed {
                by: Some(Category::Shield),
            })
        {
            return None;
        }
        self.spell_blocked((target, player), (&path, here), shot, made)
    }

    /// `by`'s single-target spell `shot` strikes `target` from `source` as its resolution says:
    /// `auto` always, `attack` and `touch` by the attack roll (`shot`'s to-hit) against its
    /// Defence. How it struck.
    pub(super) fn spell_attack(
        &mut self,
        by: ActorId,
        target: ActorId,
        source: (Pos, bool),
        shot: Shot<'_>,
        made: &mut Vec<Event>,
    ) -> Strike {
        let def = shot.def;
        let (amount, strike, sneak) = match def.resolution {
            Resolution::Attack | Resolution::Touch => {
                let attack = Attack {
                    damage: shot.dice,
                    to_hit: shot.to_hit,
                    threat: shot.threat,
                    crit: combat::DEFAULT_CRIT,
                    strength: None,
                    kinds: Kinds::NONE,
                };
                let missile = def.shape != Shape::Touch;
                let skills = shot.skills;
                let here = source.0;
                let to = self.state.actors.get(&target).map_or(here, |a| a.pos);
                let spells = self.content.weapon_skills().spell_attacks();
                let benefit = (self.state.actors.get(&by))
                    .map(|a| a.benefit_of(&self.content, skills))
                    .unwrap_or_default()
                    .at_distance(here.chebyshev(to), spells);
                // Caught unawares (`surprise.rs`): the initiative check, before the roll; a
                // sneak attack within reach (`ambush.rs`), its dice after it.
                let unaware = self.unaware(target, by);
                let unseen = self.sneaks(by, target, unaware, missile);
                // Hair trigger's blow may cut the caster down before the spell lands.
                if unaware && !self.startle(target, Some(by), made) {
                    return Strike::Missed { by: None };
                }
                self.attacked_near(by, target);
                let struck = self.target_against(target, missile);
                let struck = self.vitals(by, target, struck);
                let struck = self.flat_footed(target, unaware, struck);
                let against = combat::Target {
                    no_criticals: struck.no_criticals,
                    ..combat::Target::bare(struck.defence)
                };
                let profile = Profile {
                    reroll: shot.reroll,
                    ..Profile::bare(&attack).with(&benefit)
                };
                let profile = self.sneak_threat(by, unseen, profile);
                let blow = combat::strike(&profile, &against, &mut self.state.rng);
                // The sneak dice come on top, as after Protection.
                let sneak = self.ambushed(by, unseen, blow).damage - blow.damage;
                self.spell_practised(target, skills, blow, made);
                let spell = (AttackKind::Spell, skills);
                self.sneak_trigger((by, target), spell, (unseen, Some(blow)), made);
                (blow.damage, blow.strike, sneak)
            }
            Resolution::Auto | Resolution::Area => (
                shot.dice.roll(&mut self.state.rng, shot.reroll).max(0),
                Strike::Hit { critical: false },
                0,
            ),
        };
        let damage = Damage::new(amount + sneak, def.damage_type);
        self.spell_strikes(by, target, source, (damage, strike), made);
        strike
    }
}

impl World {
    /// The player's spell that rolls to hit came to `blow` against `target`, made with
    /// `skills`: the weapon abilities of `skills` fire on a hit and a critical (spec §8 "Weapon
    /// abilities": Pinning ray, Staggering touch, Spell guard). Its marks are the whole cast's
    /// ([`World::practise_cast`]).
    fn spell_practised(
        &mut self,
        target: ActorId,
        skills: AttackSkills,
        blow: combat::Blow,
        made: &mut Vec<Event>,
    ) {
        if skills.skill.is_none() && skills.style.is_none() {
            return;
        }
        let player = self.state.player;
        let kind = AttackKind::Spell;
        self.resolve_triggers((player, target), (kind, skills), Some(blow), made);
    }
}

/// Who pays for a cast.
#[derive(Clone, Copy, Debug)]
enum Pay {
    /// The caster's mana: what it gives [`World::spend_mana`].
    Mana(i32),
    /// The scroll read from this inventory stack, which casts at these rungs.
    Scroll(usize, u8),
    /// A charge of the wand in this inventory stack, which casts at these rungs.
    Wand(usize, u8),
}

/// One cast's settled facts, handed to each kind's code.
struct Cast<'c> {
    spell: SpellId,
    def: &'c SpellData,
    pay: Pay,
    reach: Reach,
    /// The school and the caster style it is cast in (`spell_aim.rs`), decided once before its
    /// kind runs: its marks, a rolling spell's attack and triggers.
    skills: AttackSkills,
    /// Its roll, harm or healing, at the rung reached with the dice hook applied
    /// ([`Actor::spell_roll`]); `None` for a spell that rolls nothing.
    roll: Option<Roll>,
    /// The roll hook: a die showing this or less is drawn once more.
    reroll: u32,
    /// The rider hook: statuses laid on everyone its harm reaches.
    riders: Vec<(StatusId, u16)>,
    /// A buff's or a hex's degrees ([`Actor::spell_degrees`]).
    degrees: Option<u16>,
    /// A scroll read, once paid: its object, and whether Spell recall kept it.
    read: Cell<Option<(ObjectId, bool)>>,
    /// What it came to, as its kind fills it ([`World::spell_landed`]), for its marks.
    outcome: Cell<Outcome>,
}

/// What a cast came to, for its school's marks ([`World::practise_cast`]): whether it landed on
/// anyone (a harm, a hex taken, a heal, a buff, a cure, an identify), and the level of the
/// strongest creature it struck at (`None`: no one, so the caster's own).
#[derive(Clone, Copy, Debug, Default)]
struct Outcome {
    landed: bool,
    level: Option<u32>,
}

impl<'c> Cast<'c> {
    /// `caster`'s cast of `def`, paid by `pay`, from `aim`: its hooks summed once with the
    /// caster style's ([`Actor::spell_mods_from`]), the roll at the rung of who casts it (its
    /// own, a scroll's, a wand's), the reach and the degrees the caster's own.
    fn settle(
        caster: &Actor,
        content: &Content,
        (spell, def): (SpellId, &'c SpellData),
        pay: Pay,
        aim: SpellAim,
    ) -> Self {
        let mods = caster.spell_mods_from(content, def, aim);
        let by = match pay {
            Pay::Mana(_) => CastBy::Own,
            Pay::Scroll(_, rungs) => CastBy::Scroll(rungs),
            Pay::Wand(_, rungs) => CastBy::Wand(rungs),
        };
        Cast {
            spell,
            def,
            pay,
            skills: caster.spell_skills(content, def, aim),
            reach: caster.spell_reach_with(content, def, &mods),
            roll: caster.spell_roll_with(content, def, by, &mods),
            reroll: mods.reroll,
            degrees: caster.spell_degrees_with(content, def, &mods),
            riders: mods.riders,
            read: Cell::new(None),
            outcome: Cell::new(Outcome::default()),
        }
    }
}
