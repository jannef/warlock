//! The one cast path (spec §8 "Spells", the magic design §5): a spell of `data/spells/` cast by
//! the player ([`Action::Cast`], `use:arcane_bolt:12,5`), from its cost to what it does.
//!
//! - **Cost** ([`Actor::spell_cost`], the one hook): the tier table's price
//!   ([`SpellRules::mana`]: base by tier × overreach × shape × resolution × the armour worn's
//!   `spell_cost_percent`, rounded once), then Archmage's `spell_mana_percent`
//!   ([`Actor::spell_base`], what [`World::spend_mana`] is given), then the talents' cut there
//!   (Frugal caster). Spent only once the cast is sure to go off.
//! - **Reach**: the stats its `reach` scales name (Willpower unless it says otherwise) grow a
//!   spell's range, a burst's radius, and a cone's or a line's length up to the spell's own caps
//!   ([`Actor::spell_reach`]); its `duration` scales (Presence unless it says otherwise)
//!   lengthen a buff's or a hex's status ([`SpellRules::lengthened`]; `world/spell_scale.rs`).
//! - **Dice**: the spell's at the caster's level (one die more per `per_levels` above the tier's
//!   comfortable level, never past `max_dice`), as many more as the lift of its cap holds
//!   ([`crate::content::SpellDice::lifted`]), then the talents' (Empower's die, Archmage's per
//!   die), and the roll is held to the caster's cap: the spell's cap ([`SpellRules::cap_of`])
//!   lifted by the stats its `damage` scales name, at most its tier's `max_lift`
//!   ([`Actor::spell_cap`]). Nothing passes the lifted cap. A scroll read or a wand used casts
//!   with the unlifted cap.
//! - **Resolution**: `area` (a burst, a cone, a line) strikes everyone in it with one roll, no
//!   roll to hit, Evasion halving it ([`World::spell_hurts`]) and resistances taking their
//!   share; `auto` (a ray or a bolt) strikes the first creature on its way, never missing;
//!   `attack` makes the 2d12 attack roll against Defence (the caster's `to_hit` scales to hit,
//!   Dexterity unless the spell names other stats, as a missile is aimed; a scroll or a wand
//!   always Dexterity; Deflect missiles counts), threat and critical as a blow's (a critical
//!   rolls the damage again, still held to the cap); `touch` the same roll against one creature
//!   next to the caster. `auto` and `area` never critical.
//! - **Shapes**: a burst flies and goes off as a scroll's does ([`World::burst_center`],
//!   [`World::cast_burst`]); a line runs as a scroll's bolt, through everyone on it
//!   ([`World::cast_bolt`]); a cone fans out from the caster ([`super::spell::cone_cells`],
//!   drawn as a burst around the caster over its cells); a ray or a bolt flies to the first
//!   creature on its line; a touch reaches one cell; a hex falls on the creature the caster sees
//!   at the cell aimed at; self is the caster. A spell that identifies (Identify) asks for a
//!   stack the caster carries instead of a cell (`use:identify:c`, [`Action::CastOn`]).
//! - **Explicit costs**: a spell with its own `mana` (Identify) costs that, armour and talents
//!   counted, whatever its tier ([`SpellRules::mana_with`]).
//!
//! A scroll read and a wand used cast through the same path at their own caster level, paid by
//! the scroll or a charge instead of mana ([`World::read_cast`], [`World::wand_cast`]).
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

use super::spell::{Caster, burst_cells, capped, cone_cells};
use super::talk::capitalized;
use super::{Actor, ActorId, AttackSkills, Cause, Spent, World};
use crate::ability::{Ability, Unavailable};
use crate::action::position_text;
use crate::action::{AttackKind, Event, ReadTarget, SpellShape};
use crate::combat::{self, Attack, Profile, Strike};
use crate::content::{
    ActionKind, Bolt, Burst, Content, Grip, Resolution, Shape, SpellData, SpellPurpose, SpellRules,
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

    /// The summed `spell_cost_percent` of what it wears, before Battle caster eases it: above 0,
    /// its spells that roll to hit are cast in the armoured grip.
    pub fn worn_spell_cost(&self, content: &Content) -> u32 {
        (self.equipment.items().into_iter())
            .filter_map(|(_, item)| self.inventory.get(item))
            .map(|s| u32::from(content.object(s.object).spell_cost_percent))
            .sum()
    }

    /// What `spell` asks of it at `caster_level` before its talents cut the price: the tier
    /// table's price ([`SpellRules::mana_with`], with its armour and Overreach's cut of the
    /// overreach), then School focus's and Archmage's and Empower's percents
    /// (`spell_mana_percent`), at least 1. What a cast gives [`World::spend_mana`], which takes
    /// [`Actor::mana_cost`] (Frugal caster) off it.
    pub fn spell_base(&self, content: &Content, spell: &SpellData, caster_level: i32) -> i32 {
        let Some(rules) = content.spell_rules() else {
            return 1;
        };
        let mods = self.talent_mods(content);
        let armour = self.armour_spell_cost(content);
        let price = rules.mana_with(spell, caster_level, armour, mods.overreach_cost_percent);
        let price = u16::try_from(price).unwrap_or(u16::MAX);
        let (focus, _) = mods.focus(&spell.tags(content));
        mods.spell_mana_with(price, focus)
    }

    /// What `spell` costs it at `caster_level` (the one hook, spec §8 "Spells"):
    /// [`Actor::spell_base`] through [`Actor::mana_cost`], what a cast spends and what the
    /// abilities menu's "too little mana" compares with.
    pub fn spell_cost(&self, content: &Content, spell: &SpellData, caster_level: i32) -> i32 {
        self.mana_cost(content, self.spell_base(content, spell, caster_level))
    }

    /// How far `spell` reaches as it casts it: the spell's own reach grown by its `reach`
    /// scales ([`Actor::spell_reach_more`]; a cell per two points of Willpower's modifier
    /// unless the spell names other stats) up to the spell's caps.
    pub fn spell_reach(&self, content: &Content, spell: &SpellData) -> Reach {
        if content.spell_rules().is_none() {
            return Reach::default();
        }
        let stats = self.spell_reach_more(content, spell);
        let grow = |base: Option<u8>, cap: Option<u8>, more: i32| {
            let base = i32::from(base?);
            let cap = cap.map_or(base, i32::from);
            Some(
                SpellRules::grown(base, cap, stats)
                    .saturating_add(more)
                    .min(cap.max(base)),
            )
        };
        // School focus (magic design §10): a step farther, never past the spell's own cap.
        let (_, focus) = self.talent_mods(content).focus(&spell.tags(content));
        let range = match spell.shape {
            Shape::Touch => Some(1),
            Shape::Myself => None,
            Shape::Cone | Shape::Line => grow(spell.length, spell.max_length, focus),
            Shape::Ray | Shape::Bolt | Shape::Burst => grow(spell.range, spell.max_range, focus),
        };
        Reach {
            range,
            radius: grow(spell.radius, spell.max_radius, 0),
        }
    }
}

/// A spell's damage dice as a caster at `caster_level` with talents `mods` rolls them: the
/// spell's at that level with the dice its lifted cap holds (`cap`: the cap and the caster's
/// lifted cap), then Empower's and Archmage's. The roll is held to the lifted cap after.
fn damage_dice(
    rules: &SpellRules,
    spell: &SpellData,
    caster_level: i32,
    cap: (i32, i32),
    mods: &crate::talent::TalentMods,
) -> Option<Dice> {
    let comfortable = rules.comfortable[usize::from(spell.tier)];
    let dice = spell.damage?.lifted(caster_level, comfortable, cap);
    Some(mods.spell_damage(dice))
}

impl World {
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
        let Some(target) = self.cast_aim(def, target, &format!("use:{name}")) else {
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
            cap: (rules.cap_of(def), me.spell_cap(&content, def)),
            read: Cell::new(None),
        };
        let price = self.price_of(player, ActionKind::Cast) * i64::from(def.casts);
        if self.perform(&cast, target, events) {
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

    /// The player reads the scroll in inventory stack `item`, which casts `spell` at caster
    /// level `level` (spec §8 "Scrolls"): the cast path's, but paid by the scroll, not mana,
    /// and never refused for what the player knows or holds. Whether it went off (`false`: the
    /// scroll is kept, no time).
    pub(super) fn read_cast(
        &mut self,
        item: usize,
        (spell, level): (SpellId, u8),
        target: Option<ReadTarget>,
        events: &mut Vec<Event>,
    ) -> bool {
        let content = self.content.clone();
        let (def, Some(rules)) = (content.spell(spell), content.spell_rules()) else {
            self.say("Nothing happens.");
            return false;
        };
        let usage = format!("read:{}", position_text(item));
        let Some(target) = self.cast_aim(def, target, &usage) else {
            return false;
        };
        let cast = Cast {
            spell,
            def,
            rules,
            level: i32::from(level),
            pay: Pay::Scroll(item),
            reach: self.state.player().spell_reach(&content, def),
            cap: (rules.cap_of(def), rules.cap_of(def)),
            read: Cell::new(None),
        };
        self.perform(&cast, target, events)
    }

    /// The player uses the wand in inventory stack `item`, which casts `spell` at caster level
    /// `level` at cell `at` (spec §8 "Magic items": "Wands"): the cast path's, paid by a
    /// charge, not mana, and never refused for what the player knows or holds. Whether it went
    /// off (`false`: no charge used, no time).
    pub(super) fn wand_cast(
        &mut self,
        item: usize,
        (spell, level): (SpellId, i32),
        at: Pos,
        events: &mut Vec<Event>,
    ) -> bool {
        let content = self.content.clone();
        let (def, Some(rules)) = (content.spell(spell), content.spell_rules()) else {
            self.say("Nothing happens.");
            return false;
        };
        let cast = Cast {
            spell,
            def,
            rules,
            level,
            pay: Pay::Wand(item),
            reach: self.state.player().spell_reach(&content, def),
            cap: (rules.cap_of(def), rules.cap_of(def)),
            read: Cell::new(None),
        };
        self.perform(&cast, Some(ReadTarget::Cell(at)), events)
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
            SpellPurpose::Heal => self.cast_heal(cast, at),
            SpellPurpose::Cleanse => self.cleanse(cast, events),
            SpellPurpose::Hex => self.cast_hex(cast, at, events),
            SpellPurpose::Damage => self.cast_damage(cast, at, events),
            SpellPurpose::Utility if cast.def.effect.identify => {
                let on = match target {
                    Some(ReadTarget::Item(on)) => Some(on),
                    _ => None,
                };
                self.cast_identify(cast, on)
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
                self.practise_in_armour(cast.def);
            }
            Pay::Scroll(item) => {
                let Some((scroll, kept)) = self.use_up(item) else {
                    return;
                };
                self.say_read(&scroll);
                cast.read.set(Some((scroll.object, kept)));
            }
            Pay::Wand(item) => self.use_charge(item),
        }
    }

    /// Battle caster (spec §8 "Fighting styles") trains on a spell `def` paid in mana that does
    /// not roll to hit, cast in armour that hinders casting: an attempt's marks, as against an
    /// opponent of the player's own level. (One that rolls to hit trains it as its style, by
    /// the roll: `cast_single`.)
    fn practise_in_armour(&mut self, def: &SpellData) {
        let me = self.state.player();
        if def.rolls_to_hit() || me.worn_spell_cost(&self.content) == 0 {
            return;
        }
        let Some(style) = (self.content.weapon_skills()).style_for(Grip::ArmouredSpell, &[]) else {
            return;
        };
        let level = self.level_of(self.state.player);
        self.practise(style, false, level);
    }

    /// The degrees of the status of the player's buff or hex `def` of `severity`, lengthened
    /// by its `duration` scales (Presence unless it names other stats).
    fn lengthened(&self, def: &SpellData, severity: u16) -> u16 {
        let more = (self.state.player()).spell_degrees_more(&self.content, def);
        SpellRules::lengthened(severity, more)
    }

    /// A buff: its status on the caster, lengthened by Presence.
    fn cast_buff(&mut self, cast: &Cast<'_>, events: &mut Vec<Event>) -> bool {
        let Some((status, severity)) = cast.def.effect.status else {
            return false;
        };
        let player = self.state.player;
        let severity = self.lengthened(cast.def, severity);
        self.pay_cast(cast);
        self.afflict(player, status, severity, Some(player), events);
        true
    }

    /// Identify: the truth of inventory stack `on` (spec §8 "Spells"), as the scroll of identify
    /// told it ([`World::identify`]). Refused (said, `false`) without such a stack, or for one
    /// whose truth is known. A scroll read on its own stack identifies nothing more: the read
    /// named it.
    fn cast_identify(&mut self, cast: &Cast<'_>, on: Option<usize>) -> bool {
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
            Pay::Scroll(item) if on == item => return true,
            Pay::Scroll(item) if on > item && self.state.player().inventory.len() < before => {
                on - 1
            }
            _ => on,
        };
        if matches!(self.identify(on), super::Identified::Now(_)) {
            self.train_stat(crate::content::StatUse::Identified);
        }
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
            .lifted(cast.level, comfortable, cast.cap)
            .roll(&mut self.state.rng)
            .max(0);
        let back = rolled.min(cast.cap.1);
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
        let severity = self.lengthened(cast.def, severity);
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
        let severity = self.lengthened(cast.def, severity);
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
        let dice =
            damage_dice(cast.rules, def, cast.level, cast.cap, &mods).unwrap_or(Dice::of(0, 1));
        let cap = cast.cap.1;
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
                let own = matches!(cast.pay, Pay::Mana(_));
                let aim = (here, at, range);
                self.cast_single(aim, (&effect, dice, cap), (def, own), &mut made);
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
            && !matches!(cast.pay, Pay::Scroll(_))
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
    /// attack roll against its Defence, aimed with its `to_hit` scales when it is the caster's
    /// `own` spell, else (a scroll, a wand) with Dexterity. One that finds no one strikes
    /// nothing.
    fn cast_single(
        &mut self,
        (here, at, reach): (Pos, Pos, i32),
        (effect, dice, cap): (&str, Dice, i32),
        (def, own): (&SpellData, bool),
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
                let me = self.state.player();
                let to_hit = if own {
                    me.spell_to_hit(&self.content, def)
                } else {
                    me.effective_stats(&self.content).modifier(Stat::Dexterity)
                };
                let attack = Attack {
                    damage: dice,
                    to_hit,
                    threat: combat::DEFAULT_THREAT,
                    crit: combat::DEFAULT_CRIT,
                    strength: None,
                    kinds: Kinds::NONE,
                };
                let missile = def.shape != Shape::Touch;
                // The spell attack skill and the caster style (spec §8 "Weapon skills",
                // "Fighting styles"): the caster's own spells only, as the to-hit scales are.
                let skills = if own && def.rolls_to_hit() {
                    me.spell_skills(&self.content, !missile)
                } else {
                    AttackSkills::default()
                };
                let to = self.state.actors.get(&target).map_or(here, |a| a.pos);
                let spells = self.content.weapon_skills().spell_attacks();
                let benefit =
                    (me.benefit_of(&self.content, skills)).at_distance(here.chebyshev(to), spells);
                let defence = self.target_against(target, missile).defence;
                let against = combat::Target::bare(defence);
                let profile = Profile::bare(&attack).with(&benefit);
                let blow = combat::strike(&profile, &against, &mut self.state.rng);
                self.spell_practised(target, skills, blow, made);
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

impl World {
    /// The player's spell that rolls to hit came to `blow` against `target`, made with
    /// `skills`: they earn their marks as an attack does (the weapon skill, then the style),
    /// and the weapon abilities of `skills` fire on a hit and a critical (spec §8 "Weapon
    /// abilities": Pinning ray, Staggering touch, Spell guard).
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
        let level = self.level_of(target);
        for skill in skills.trained() {
            self.practise(skill, blow.strike.is_hit(), level);
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
    /// The scroll read from this inventory stack.
    Scroll(usize),
    /// A charge of the wand in this inventory stack.
    Wand(usize),
}

/// One cast's settled facts, handed to each kind's code.
struct Cast<'c> {
    spell: SpellId,
    def: &'c SpellData,
    rules: &'c SpellRules,
    /// The caster's caster level (a scroll's own, read; a wand's own, used).
    level: i32,
    pay: Pay,
    reach: Reach,
    /// The spell's cap, and the caster's lifted cap ([`Actor::spell_cap`]; a scroll's or a
    /// wand's is the spell's own): the most it does to one target.
    cap: (i32, i32),
    /// A scroll read, once paid: its object, and whether Spell recall kept it.
    read: Cell<Option<(ObjectId, bool)>>,
}
