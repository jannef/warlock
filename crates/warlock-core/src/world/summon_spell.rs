//! Player summons (spec §8 "Allies": "Summons"; docs/design/druid-and-summons.md §5 and §14;
//! `docs/dev/allies.md`): a spell of `kind = "summon"` calls creatures of its pool to the
//! caster's side, as allies ([`super::Ally`]) that follow and fight for them until their time
//! runs out or they are slain.
//!
//! **The cast** ([`World::cast_summon`], the summon arm of `cast.rs::perform`, after the cast
//! path's own refusals): refused, with no time and no mana, without `data/allies.toml`, on the
//! overworld (which has no creatures), with no free cell within the call's `spread`
//! (`World::summon_cells`, the enemy summoner's), and at the global cap (`[allies] most` of the
//! player's allies on the level) unless one of this spell's own would give way. Then: paid, said
//! (`[spell.cast] message`), heard (`[spell.cast] noise`); if this spell's calls alive (on any
//! level and in the retinue) are already at its `most` (raised by `ally_mod`'s `most`), the
//! **oldest** (lowest id) departs first, so a call always brings at least one. The count is the
//! spell's roll at the caster's rung (its `count` ladder, the dice hooks: Empower and Brimming
//! act on it), plus `ally_mod`'s `count`, at least 1, then held to the spell's room, the global
//! cap's, and the free cells. Each comes from the pool by [`World::summon_draw`] (the enemy
//! summoner's draw, weighed by the caster's `summon_bias`), made by `World::make_ally` beside
//! the caster, nearest first, with no gear.
//!
//! **Its time**: the spell's minutes (its `degrees` ladder, Lingering's degrees) made `ally_mod`'s
//! `duration_percent` longer (rounded down, at least 1) set [`super::Ally::until`] on the clock;
//! `ally_mod`'s `lasting` leaves it `None`: the call stays until slain.
//!
//! **Its death**: a summoned ally leaves no corpse and drops nothing (`fight.rs::creature_dies`).
//!
//! Draws: the count roll, then one draw per creature, on the gameplay stream, only when a call
//! goes off.

use super::ally::{AllySource, capital, few};
use super::cast::Cast;
use super::{ActorId, World};
use crate::action::Event;
use crate::content::{CreatureId, LevelKind, SpellId, and_list};
use crate::time::Duration;

impl World {
    /// The player's summon spell `cast` (module docs); whether it went off.
    pub(super) fn cast_summon(&mut self, cast: &Cast<'_>, events: &mut Vec<Event>) -> bool {
        let content = self.content.clone();
        let Some(call) = cast.def.effect.summon.as_ref() else {
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
        let player = self.state.player;
        if self.summon_cells(player, call.spread).is_empty() {
            self.say("There is no room for them beside you.");
            return false;
        }
        let mods = self.state.player().talent_mods(&content);
        let most = usize::try_from(i32::from(call.most) + mods.ally.most)
            .unwrap_or(0)
            .max(1);
        let mut mine = self.calls_of(cast.spell);
        // So many of this spell's own give way, the oldest first, that one more fits.
        let give_way = (mine.len() + 1).saturating_sub(most);
        let level = self.state.current_level.clone();
        let freed_here = (mine.iter().take(give_way))
            .filter(|id| {
                let a = &self.state.actors[id];
                a.level == level && !a.in_retinue()
            })
            .count();
        let global = usize::from(rules.most);
        if self.allies_here(player).len() - freed_here >= global {
            self.say("You cannot lead more than this.");
            return false;
        }
        if !self.pay_cast(cast) {
            return true;
        }
        if let Some(text) = &cast.def.look.message {
            self.say(text.clone());
        }
        self.sound_at(cast.def.look.noise, self.player_pos());
        for id in mine.drain(..give_way) {
            self.gives_way(id, events);
        }
        let rolled = cast
            .roll
            .map_or(1, |r| r.roll(&mut self.state.rng, cast.reroll))
            + mods.ally.count;
        let cells = self.summon_cells(player, call.spread);
        let room = global.saturating_sub(self.allies_here(player).len());
        let count = usize::try_from(rolled.max(1))
            .unwrap_or(1)
            .min(most - mine.len())
            .min(room)
            .min(cells.len());
        let until = (!mods.ally.lasting).then(|| {
            let minutes = i64::from(cast.degrees.unwrap_or(call.minutes));
            let longer = minutes * (100 + i64::from(mods.ally.duration_percent).max(0)) / 100;
            self.state.time + Duration::minutes(u64::try_from(longer.max(1)).unwrap_or(1))
        });
        let mut came: Vec<(CreatureId, u32)> = Vec::new();
        for &at in cells.iter().take(count) {
            let creature = self.summon_draw(&call.pool, &mods.summon_bias);
            let source = AllySource::Summoned {
                spell: Some(cast.spell),
            };
            self.make_ally(creature, at, source, until);
            match came.iter_mut().find(|(c, _)| *c == creature) {
                Some((_, n)) => *n += 1,
                None => came.push((creature, 1)),
            }
        }
        let words: Vec<String> = (came.iter())
            .map(|&(c, n)| {
                let def = content.creature(c);
                few(&def.name, &def.plural, n)
            })
            .collect();
        let list = and_list(&words.iter().map(String::as_str).collect::<Vec<_>>());
        let verb = if count == 1 { "comes" } else { "come" };
        self.say(format!("{} {verb} to your call.", capital(&list)));
        self.spell_landed(cast, player, true);
        true
    }

    /// The player's allies called by spell `spell`, on any level and in the retinue, oldest
    /// (lowest id) first.
    pub(super) fn calls_of(&self, spell: SpellId) -> Vec<ActorId> {
        let player = self.state.player;
        (self.allies_of(player).into_iter())
            .filter(|id| {
                (self.state.actors[id].ally.as_ref())
                    .is_some_and(|a| a.source == AllySource::Summoned { spell: Some(spell) })
            })
            .collect()
    }

    /// Ally `id` gives way to a new call of its spell: it departs as a lapsed one does where the
    /// player is, and is removed silently on another level or in the retinue.
    pub(super) fn gives_way(&mut self, id: ActorId, events: &mut Vec<Event>) {
        let Some(a) = self.state.actors.get(&id) else {
            return;
        };
        if a.level == self.state.current_level && !a.in_retinue() {
            self.ally_departs(id, events);
        } else {
            self.state.actors.remove(&id);
        }
    }
}
