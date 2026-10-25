//! The casters in the world (spec §8 "Class abilities", design §4.9–4.12): the priest's, the
//! druid's, the wizard's, and the witch's spells ([`Action::Cast`], `use:blessing`,
//! `use:arcane_bolt:12,5`), placeholders for the spell system to come, paid in mana and cast
//! through the shared spell code the scrolls use (`world/spell.rs`); and their passives that
//! need a rule of their own: the once-in-a-life reprieves (Miracle, Hag's bargain) and the blows
//! that hurt a suffering creature more (Poison lore).
//!
//! A cast is refused, taking no time and no mana, without the spell, without the mana it costs
//! ([`Unavailable::NoMana`]), without the cell it asks, and where it cannot go (out of range, no
//! one seen to hex, a lightning call under a roof). Otherwise its mana is spent
//! ([`World::spend_mana`]), it takes its `cast`s' time, and it does what its kind does
//! ([`Does`]). A spell's blows are the caster's, as a scroll's are: its kills earn them their
//! experience, and one that kills its caster is what killed them ([`Cause::Spell`]). Nothing
//! here draws at random but the dice of the spell itself.
//!
//! [`Action::Cast`]: crate::action::Action::Cast

use super::spell::Caster;
use super::talk::capitalized;
use super::{Cause, Spent, World};
use crate::ability::{Ability, Unavailable};
use crate::action::{AttackKind, Event};
use crate::combat::Blow;
use crate::content::{ActionKind, Cast, ClassSpell, Does, ScrollSpell, Stance, StatusId};
use crate::geom::Pos;
use crate::world::ActorId;

impl super::Actor {
    /// What one of its spells that costs `base` mana costs it (spec §8 "Mana"): the one place
    /// a spell's price in mana is reckoned, read by the cast and by the abilities menu's "not
    /// enough mana" alike (Archmage halves it; at least 1).
    pub fn spell_cost(&self, content: &crate::content::Content, base: u16) -> i32 {
        self.talent_mods(content).spell_mana(base)
    }
}

impl World {
    /// `use:ID[:X,Y]` ([`crate::action::Action::Cast`]): the player casts `spell`, at cell `at`
    /// if it asks for one (module docs).
    pub(super) fn cast_spell(
        &mut self,
        spell: ClassSpell,
        at: Option<Pos>,
        events: &mut Vec<Event>,
    ) -> Spent {
        let content = self.content.clone();
        let ability = Ability::Spell(spell);
        let me = self.state.player();
        let Some((_, def)) = content.class_spell(spell) else {
            self.say("You have no such ability.");
            return Spent::Nothing;
        };
        if !ability.had_by(me, &content) {
            self.say("You have no such ability.");
            return Spent::Nothing;
        }
        if let Some(why) = ability.unavailable(me, &content) {
            self.say(match why {
                Unavailable::NoMana => "You have not enough mana.".to_string(),
                Unavailable::While(status) => format!(
                    "You are {}: you cannot cast now.",
                    content.status(status).name.to_lowercase()
                ),
                other => format!("You cannot: {}.", other.reason(&content)),
            });
            return Spent::Nothing;
        }
        let at = match (spell.aimed(), at) {
            (true, None) => {
                let id = spell.id();
                self.say(format!("Cast it where? Name a cell: `use:{id}:12,5`."));
                return Spent::Nothing;
            }
            (_, at) => at,
        };
        let player = self.state.player;
        let mana = self.state.player().spell_cost(&content, def.mana);
        let price = self.price_of(player, ActionKind::Cast) * i64::from(def.casts);
        match (&def.does, at) {
            (Does::Boon { status, severity }, _) => {
                self.spend_mana(player, mana);
                self.afflict(player, *status, *severity, Some(player), events);
            }
            (Does::Mend { heal, per_levels }, _) => {
                let me = self.state.player();
                if me.hp >= me.max_hp(&content) {
                    self.say("You are not hurt.");
                    return Spent::Nothing;
                }
                self.spend_mana(player, mana);
                let more = match per_levels {
                    0 => 0,
                    n => i32::from(self.player_level() / n),
                };
                let back = heal.roll(&mut self.state.rng).max(0) + more;
                self.heal_player(back);
                self.say("Your wounds close.");
            }
            (Does::Cleanse { cures, diseases }, _) => {
                if !self.cleanse(cures, *diseases, mana, events) {
                    return Spent::Nothing;
                }
            }
            (
                Does::Hex {
                    status,
                    severity,
                    range,
                },
                Some(at),
            ) => {
                let Some(target) = self.hex_target(at, *range) else {
                    return Spent::Nothing;
                };
                self.spend_mana(player, mana);
                self.afflict(target, *status, *severity, Some(player), events);
                let who = capitalized(&self.called(&self.state.actors[&target]));
                let word = (content.status(*status).seen.clone()).unwrap_or_else(|| "hexed".into());
                self.say(format!("{who} is {word}."));
            }
            (
                Does::Strike {
                    spell: strike,
                    open_sky,
                    afflict,
                },
                Some(at),
            ) => {
                if !self.can_loose_here() || !self.within_reach(at, strike.range, "cast") {
                    return Spent::Nothing;
                }
                if *open_sky && !self.under_open_sky() {
                    self.say("You need the open sky above you for that.");
                    return Spent::Nothing;
                }
                self.spend_mana(player, mana);
                self.strike(spell, strike, at, *afflict, events);
            }
            (Does::Hex { .. } | Does::Strike { .. }, None) => return Spent::Nothing,
        }
        Spent::Turn(price)
    }

    /// Whether the player stands under the open sky: a level under the sky, and no roof over
    /// their cell (whatever the clouds and the hour).
    fn under_open_sky(&self) -> bool {
        let level = self.state.current();
        let here = self.player_pos();
        level.kind.is_sky_exposed()
            && (level.terrain.get(here)).is_some_and(|t| !self.content.terrain(*t).roofed)
    }

    /// A cleanse: the statuses `cures` names end, and with `diseases` every disease the player
    /// has, each said as it ends; `mana` spent first. Refused (said, `false`) when there is
    /// nothing to cure.
    fn cleanse(
        &mut self,
        cures: &[StatusId],
        diseases: bool,
        mana: i32,
        events: &mut Vec<Event>,
    ) -> bool {
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
        self.spend_mana(player, mana);
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

    /// A strike: `strike`'s burst flies at `at` (or its bolt runs through it), as a scroll's
    /// does, with the player's spell talents on its dice; everyone it reaches who lives suffers
    /// `afflict` too. A death of the player by it is put down to `spell`.
    fn strike(
        &mut self,
        spell: ClassSpell,
        strike: &ScrollSpell,
        at: Pos,
        afflict: Option<(StatusId, u16)>,
        events: &mut Vec<Event>,
    ) {
        let player = self.state.player;
        let here = self.player_pos();
        let mods = self.state.player().talent_mods(&self.content);
        if let Some(text) = &strike.message {
            self.say(text.clone());
        }
        let mut made = Vec::new();
        match &strike.cast {
            Cast::Burst(burst) => {
                let mut burst = burst.clone();
                burst.damage = mods.spell_damage(burst.damage);
                let center = self.burst_center(here, at, strike.range);
                self.sound_at(strike.noise, center);
                self.cast_burst(Caster::Actor(player), (here, center), &burst, &mut made);
            }
            Cast::Bolt(bolt) => {
                let mut bolt = bolt.clone();
                bolt.damage = mods.spell_damage(bolt.damage);
                let path = self.bolt_line(here, at, bolt.length);
                self.sound_at(strike.noise, here);
                self.cast_bolt(Caster::Actor(player), here, &path, &bolt, &mut made);
            }
        }
        let reached: Vec<ActorId> = (made.iter())
            .filter_map(|e| match *e {
                Event::Attacked {
                    attacker,
                    target,
                    kind: AttackKind::Spell,
                    ..
                } if attacker == player => Some(target),
                _ => None,
            })
            .collect();
        self.say_spell((&strike.noun, &strike.verb), made, events);
        if let Some((status, severity)) = afflict {
            for id in reached {
                if self.state.actors.contains_key(&id) && self.state.death.is_none() {
                    self.afflict(id, status, severity, Some(player), events);
                }
            }
        }
        if let Some(death) = &mut self.state.death {
            death.by = None;
            death.cause = Some(Cause::Spell(spell));
        }
    }

    /// Miracle, Hag's bargain (spec §8 "Class abilities"): once in a life, what would kill the
    /// player does not. They are left at 1 hit point, suffer the reprieve's status, and lose its
    /// stat for good; its words are said. Returns whether it saved them (never twice).
    pub(super) fn reprieve(&mut self, events: &mut Vec<Event>) -> bool {
        let player = self.state.player;
        let Some(reprieve) = self.state.player().talent_mods(&self.content).reprieve else {
            return false;
        };
        if self.state.progress.happened(&reprieve.once) {
            return false;
        }
        self.state.progress.once.insert(reprieve.once.clone());
        self.say(reprieve.message.clone());
        let me = self.player_mut();
        me.hp = 1;
        if let Some((stat, by)) = reprieve.stat {
            me.stats.gift(stat, by);
        }
        self.afflict(player, reprieve.status, 1, None, events);
        true
    }

    /// `blow`, struck by `attacker` at `target`, with what Poison lore adds against a creature
    /// suffering its status: its damage more on a hit. No draw.
    pub(super) fn lore_blow(&self, attacker: ActorId, target: ActorId, blow: Blow) -> Blow {
        if !blow.strike.is_hit() {
            return blow;
        }
        let (Some(by), Some(on)) = (
            self.state.actors.get(&attacker),
            self.state.actors.get(&target),
        ) else {
            return blow;
        };
        if by.talents.is_empty() {
            return blow;
        }
        let more: i32 = (by.talent_mods(&self.content).status_damage.iter())
            .filter(|(status, _)| on.effects.has(*status))
            .map(|(_, v)| v)
            .sum();
        Blow {
            damage: (blow.damage + more).max(0),
            ..blow
        }
    }
}
