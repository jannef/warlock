//! Spells aimed at a corpse (spec §8 "Spells"; the spell types design §7 entries 2, 29, 30;
//! `docs/dev/spells.md`): `[spell.effect] corpse` asks for a cell where the caster **sees** a
//! corpse (their own cell too), the pile's top-most corpse stack is the one used, and with
//! `corpse = "consumed"` one corpse of it is gone. No stage is read: a rotten corpse does as a
//! fresh one, so nothing about its age is told. Refused, with no time and no mana, with no corpse.
//!
//! - **Charnel burst**, a burst of harm: goes off where the corpse lies, flying nowhere, and
//!   spatters its blood (`cast.rs::cast_damage`, one arm; [`World::corpse_bursts`]).
//! - **Grave feast**, a touch that heals: [`World::cast_feast`], its roll in hit points and its
//!   `mana` in mana back to the caster, each held to its maximum.
//! - **Rouse the dead**, a summon with `rise` in place of `summon`: [`World::cast_rise`]. The
//!   corpse's own creature type stands on its cell or the nearest free one, at `hit_points`
//!   percent of its maximum rounded up, with no gear, no pack, no `npc`, and no mimic's disguise.
//!   It keeps everything of its type. A second death leaves nothing (`fight.rs::creature_dies`).
//!   A unique's corpse is refused, and so is the cap of `[allies] most`, unless one of this
//!   spell's own would give way.
//! - **Serving**: as a summon's (`summon_spell.rs`), through [`World::make_ally`] with
//!   [`AllySource::Summoned`]: `most`, the oldest giving way, and `ally_mod`'s `most`,
//!   `duration_percent`, and `lasting` (its `count` not: one corpse, one creature).
//! - **Berserk**: one `below(100)` against `berserk` on the gameplay stream, after every refusal.
//!   Drawn, it rises in `berserk_faction` (`hostile_to_all`: friendly to nothing, its own kind
//!   among them) as its `faction_override`: no ally, the plain AI, no orders, counting toward
//!   neither `most` nor the cap. [`Actor::risen`] holds its clock, `duration_percent` counted and
//!   `lasting` not: it always ends ([`World::allies_lapse`]). Its kill pays as any of its type,
//!   and it counts in no level's population (`spawn.rs`).
//!
//! [`Actor::risen`]: super::Actor::risen

use super::ally::AllySource;
use super::cast::Cast;
use super::{ActorId, World, travel};
use crate::action::Event;
use crate::content::{
    CorpseUse, CreatureId, CreatureSpawn, FactionId, LevelKind, SpellData, SpellRise,
};
use crate::geom::Pos;
use crate::time::{Duration, GameTime};

impl World {
    /// The position in the pile at `at` of its top-most corpse stack, if the player sees the
    /// cell.
    pub(super) fn corpse_in_sight(&self, at: Pos) -> Option<usize> {
        if !self.perception.sees_actors_at(at) {
            return None;
        }
        (self.state.current().objects_at(at).iter())
            .position(|s| self.content.object(s.object).corpse.is_some())
    }

    /// Spell `def` uses the corpse the player sees at `at`: with `consumed` one of its stack is
    /// gone. The creature type it was the corpse of.
    fn use_corpse(&mut self, def: &SpellData, at: Pos) -> Option<CreatureId> {
        let index = self.corpse_in_sight(at)?;
        let content = self.content.clone();
        let level = self.state.current_mut();
        let mut pile = level.objects_at(at).to_vec();
        let creature = content.object(pile[index].object).corpse?;
        if def.effect.corpse == Some(CorpseUse::Consumed) {
            pile[index].count -= 1;
            if pile[index].count == 0 {
                pile.remove(index);
            }
            level.set_pile(at, pile);
        }
        Some(creature)
    }

    /// Charnel burst goes off at `at`: the corpse used, its blood spattered as a kill's.
    pub(super) fn corpse_bursts(&mut self, def: &SpellData, at: Pos) {
        if let Some(creature) = self.use_corpse(def, at) {
            self.spill_corpse(creature, at, self.player_pos());
        }
    }

    /// Grave feast at `at` (module docs); whether it went off. Refused, said, beyond a touch,
    /// with no corpse there, and with nothing to give back.
    pub(super) fn cast_feast(
        &mut self,
        cast: &Cast<'_>,
        at: Option<Pos>,
        events: &mut Vec<Event>,
    ) -> bool {
        let (Some(at), Some(heal)) = (at, cast.roll) else {
            return false;
        };
        if self.player_pos().chebyshev(at) > 1 {
            self.say("You can only touch a corpse next to you.");
            return false;
        }
        let Some(index) = self.corpse_in_sight(at) else {
            self.say("There is no corpse there.");
            return false;
        };
        let content = self.content.clone();
        let me = self.state.player();
        let (max_hp, max_mana) = (me.max_hp(&content), me.max_mana(&content));
        let gives_mana = cast.def.effect.mana.is_some() && me.mana < max_mana;
        if me.hp >= max_hp && !gives_mana {
            self.say("You are not hurt.");
            return false;
        }
        let corpse = self.state.current().objects_at(at)[index].object;
        if !self.pay_cast(cast) {
            return true;
        }
        self.use_corpse(cast.def, at);
        let back = heal.roll(&mut self.state.rng, cast.reroll).max(0);
        let mana = (cast.def.effect.mana).map_or(0, |m| m.roll(&mut self.state.rng, 0).max(0));
        self.heal_player(back);
        let me = self.player_mut();
        me.mana = (me.mana + mana).min(max_mana).max(me.mana);
        let name = &content.object(corpse).name;
        self.say(format!("You draw what life is left out of the {name}."));
        self.spell_landed(cast, self.state.player, back + mana > 0, events);
        true
    }

    /// Rouse the dead at `at` (module docs); whether it went off.
    pub(super) fn cast_rise(
        &mut self,
        cast: &Cast<'_>,
        at: Option<Pos>,
        events: &mut Vec<Event>,
    ) -> bool {
        let content = self.content.clone();
        let (Some(rise), Some(at)) = (cast.def.effect.rise.as_ref(), at) else {
            return false;
        };
        let Some(rules) = content.allies() else {
            self.say("Nothing answers your call.");
            return false;
        };
        if self.state.current().kind == LevelKind::Overworld {
            self.say("Nothing answers your call out here.");
            return false;
        }
        let Some(index) = self.corpse_in_sight(at) else {
            self.say("There is no corpse there.");
            return false;
        };
        let object = self.state.current().objects_at(at)[index].object;
        let Some(creature) = content.object(object).corpse else {
            return false;
        };
        if content.creature(creature).unique {
            self.say("It will not rise for you.");
            return false;
        }
        if self.rising_cell(at).is_none() {
            self.say("There is no room for it to stand.");
            return false;
        }
        let player = self.state.player;
        let mods = self.state.player().talent_mods(&content);
        let most = usize::try_from(i32::from(rise.most) + mods.ally.most)
            .unwrap_or(0)
            .max(1);
        let mut mine = self.calls_of(cast.spell);
        let give_way = (mine.len() + 1).saturating_sub(most);
        let level = self.state.current_level.clone();
        let freed_here = (mine.iter().take(give_way))
            .filter(|id| {
                let a = &self.state.actors[id];
                a.level == level && !a.in_retinue()
            })
            .count();
        if self.allies_here(player).len() - freed_here >= usize::from(rules.most) {
            self.say("You cannot lead more than this.");
            return false;
        }
        if !self.pay_cast(cast) {
            return true;
        }
        self.use_corpse(cast.def, at);
        self.sound_at(cast.def.look.noise, self.player_pos());
        let minutes = i64::from(cast.degrees.unwrap_or(rise.minutes));
        let longer = minutes * (100 + i64::from(mods.ally.duration_percent).max(0)) / 100;
        let ends = self.state.time + Duration::minutes(u64::try_from(longer.max(1)).unwrap_or(1));
        let risen = match self.draw_berserk(rise) {
            Some(faction) => {
                self.say(rise.berserk_message.clone());
                self.rise_berserk(creature, at, faction, ends)
            }
            None => {
                if let Some(text) = &cast.def.look.message {
                    self.say(text.clone());
                }
                for id in mine.drain(..give_way) {
                    self.gives_way(id, events);
                }
                let cell = self.rising_cell(at).unwrap_or(at);
                let source = AllySource::Summoned {
                    spell: Some(cast.spell),
                };
                let until = (!mods.ally.lasting).then_some(ends);
                self.make_ally(creature, cell, source, until)
            }
        };
        self.stands_risen(risen, rise);
        self.spell_landed(cast, player, true, events);
        true
    }

    /// The one draw that says whether it rises berserk, and the faction it then rises in (none
    /// where the world has no such faction).
    fn draw_berserk(&mut self, rise: &SpellRise) -> Option<FactionId> {
        let draw = self.state.rng.below(100);
        (draw < u32::from(rise.berserk))
            .then(|| self.content.factions().id(&rise.berserk_faction))
            .flatten()
    }

    /// The cell a corpse at `at` rises on: its own if it may be stood on and is free, else the
    /// nearest free one.
    pub(super) fn rising_cell(&self, at: Pos) -> Option<Pos> {
        let id = &self.state.current_level;
        let level = self.state.current();
        if level.is_walkable(&self.content, at) && self.state.actor_at(id, at).is_none() {
            return Some(at);
        }
        let reach = level.width().max(level.height()) - 1;
        self.free_cell_within(id, at, reach, |_| true)
    }

    /// A creature of type `creature` rises friendly to nothing, in `faction`, until `ends`.
    fn rise_berserk(
        &mut self,
        creature: CreatureId,
        at: Pos,
        faction: FactionId,
        ends: GameTime,
    ) -> ActorId {
        let content = self.content.clone();
        let spawn = CreatureSpawn {
            creature,
            faction: Some(faction),
            alignment: None,
            npc: None,
            pos: self.rising_cell(at).unwrap_or(at),
            route: Vec::new(),
            carry: Vec::new(),
            equipment: Default::default(),
            rolls_gear: false,
            asleep: false,
            status: Vec::new(),
        };
        let level = self.state.current_level.clone();
        let id = travel::make_creature(&mut self.state, &content, &level, &spawn);
        if let Some(actor) = self.state.actors.get_mut(&id) {
            actor.risen = Some(ends);
        }
        // A newcomer: a rest looks up at once.
        self.spawned += 1;
        id
    }

    /// What every risen creature comes to as it stands: its share of its hit points, rounded
    /// up, and no mimic's disguise.
    fn stands_risen(&mut self, id: ActorId, rise: &SpellRise) {
        let content = self.content.clone();
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        let max = actor.max_hp(&content);
        actor.hp = (max * i32::from(rise.hit_points) + 99) / 100;
        if let Some(disguise) = content.disguise_status() {
            actor.effects.remove(disguise);
        }
    }
}
