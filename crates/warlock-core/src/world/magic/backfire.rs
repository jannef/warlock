//! Miscast backfires (spec §8 "Spells": "Miscasts"; docs/design/miscasts-review.md §5.3, #1291):
//! a `backfire` entry turns the spell just cast on its caster, at the spell's base roll, or the
//! roll it was cast with when `full`. What it does goes by the spell's kind:
//!
//! - **damage**: a burst around the caster (its radius, else 1; `miscast.rs::miscast_burst`).
//! - **hex**: its status on the caster at the cast's degrees.
//! - **heal** (the rebound heal): the roll mends the nearest actor hostile to the caster that
//!   the caster sees ([`World::nearest_hostile`]: by Chebyshev distance, then actor id; the
//!   player's sight is the perception snapshot, a creature's its own eyes,
//!   `surprise.rs::sees_now`), never past its maximum, said by name.
//! - **buff**: its status on that nearest hostile at the cast's degrees.
//! - **summon** (a call, not a rise): the call's count (its base, or the cast's roll when `full`;
//!   at least 1, at most the call's `most` and the free cells of its `spread`) of creatures
//!   drawn from its pool as the call draws them (`summon.rs::summon_draw`, the caster's bias),
//!   made beside the caster **hostile to everyone**: of `data/miscast.toml`'s `backfire_faction`
//!   (the berserk, `hostile_to_all`: the caster, the caster's foes, and one another alike; #1311;
//!   their type's own faction where it names none), no ally, no summoner, no gear. Only on the
//!   level in play, never on the overworld.
//! - Anything else (a cleanse, a utility, a rise, a heal or buff with no hostile in sight, a
//!   summon with no room, a damage spell or hex with nothing to roll or lay): the **fallback
//!   harm**, `(level ÷ 3 + 1)d6` force at base, `(level ÷ 2 + 1)d6` when `full`, through
//!   [`World::resisted`] and [`World::hurt`]. Backfire entries are never protected (the loader's
//!   whitelist), so it may kill, and the miscast puts the death down to the spell.
//!
//! Draws, on the gameplay stream: the heal's roll; a summon's count, then one draw per creature;
//! the fallback harm's dice. Picking the nearest hostile draws nothing.

use super::cast::Cast;
use crate::action::Event;
use crate::content::{CreatureId, CreatureSpawn, LevelKind, SpellPurpose, Stance};
use crate::damage::Damage;
use crate::dice::Dice;
use crate::rng::draw;
use crate::text::{and_list, capitalized};
use crate::world::combat::fight::Harm;
use crate::world::{ActorId, World};

impl World {
    /// Spell `cast` backfires on `caster`, at its base roll or, when `full`, its cast's (module
    /// docs).
    pub(super) fn miscast_backfire(
        &mut self,
        caster: ActorId,
        cast: &Cast<'_>,
        full: bool,
        events: &mut Vec<Event>,
    ) {
        let def = cast.def;
        let roll = if full { cast.roll } else { def.roll() };
        let reroll = if full { cast.reroll } else { 0 };
        let done = match def.kind {
            SpellPurpose::Damage => match (roll, self.state.actors.get(&caster)) {
                (Some(roll), Some(me)) => {
                    let here = me.pos;
                    self.miscast_burst(caster, def, (here, roll), events);
                    true
                }
                _ => false,
            },
            SpellPurpose::Hex => match (def.effect.status, cast.degrees) {
                (Some((status, _)), Some(degrees)) => {
                    self.afflict(caster, status, degrees, None, events);
                    true
                }
                _ => false,
            },
            SpellPurpose::Heal => match (roll, self.nearest_hostile(caster)) {
                (Some(roll), Some(on)) => {
                    let mut rng = draw!(self, "backfire/heal/{}", self.state.actors.key(caster));
                    let back = roll.roll(&mut rng, reroll).max(0);
                    self.rebound_heal(caster, on, back);
                    true
                }
                _ => false,
            },
            SpellPurpose::Buff => match (def.effect.status, cast.degrees) {
                (Some((status, _)), Some(degrees)) => match self.nearest_hostile(caster) {
                    Some(on) => {
                        if caster == self.state.player {
                            let who = self.called(&self.state.actors[&on]);
                            self.say(format!("Your spell takes hold of {who} instead."));
                        }
                        self.afflict(on, status, degrees, None, events);
                        true
                    }
                    None => false,
                },
                _ => false,
            },
            SpellPurpose::Summon => self.hostile_call(caster, cast, full),
            SpellPurpose::Cleanse | SpellPurpose::Utility => false,
        };
        if !done {
            self.backfire_harm(caster, def.level, full, events);
        }
    }

    /// The nearest actor on `caster`'s level hostile to it that it sees (module docs), by
    /// Chebyshev distance, then actor id.
    pub(super) fn nearest_hostile(&mut self, caster: ActorId) -> Option<ActorId> {
        let me = self.state.actors.get(&caster)?;
        let (level, here) = (me.level.clone(), me.pos);
        let player = self.state.player;
        let mut near: Vec<(i32, ActorId)> = (self.state.actors.values())
            .filter(|a| a.id != caster && a.level == level && a.hp > 0)
            .map(|a| (a.pos.chebyshev(here), a.id))
            .collect();
        near.sort();
        for (_, id) in near {
            if self.stance(caster, id) != Stance::Hostile {
                continue;
            }
            let seen = if caster == player {
                let at = self.state.actors[&id].pos;
                self.perception.sees_actors_at(at)
            } else {
                self.sees_now(caster, id)
            };
            if seen {
                return Some(id);
            }
        }
        None
    }

    /// `back` hit points of `caster`'s rebounding heal go to `on`, never past its maximum;
    /// said by name to the caster that is the player, else as a heal is seen.
    fn rebound_heal(&mut self, caster: ActorId, on: ActorId, back: i32) {
        let player = self.state.player;
        let content = self.content.clone();
        let Some(them) = self.state.actors.get_mut(&on) else {
            return;
        };
        let max = them.max_hp(&content);
        them.hp = (them.hp + back).min(max).max(them.hp);
        let them = &self.state.actors[&on];
        if caster == player {
            let who = self.called(them);
            self.say(format!("Your healing flows into {who}."));
        } else if on == player {
            self.say("Your wounds close.");
        } else if self.perception.sees_actors_at(them.pos) {
            let who = capitalized(&self.called(them));
            self.say(format!("{who} looks better."));
        }
    }

    /// `caster`'s summon `cast` backfires (module docs): its creatures come beside it hostile.
    /// Whether any came.
    fn hostile_call(&mut self, caster: ActorId, cast: &Cast<'_>, full: bool) -> bool {
        let content = self.content.clone();
        let Some(call) = cast.def.effect.summon.as_ref() else {
            return false;
        };
        let level = self.state.current();
        let Some(me) = self.state.actors.get(&caster) else {
            return false;
        };
        if me.level != level.id || level.kind == LevelKind::Overworld || call.pool.is_empty() {
            return false;
        }
        let level = level.id.clone();
        let bias = me.talent_mods(&content).ally.summon_bias.clone();
        let faction = content.miscast_rules().and_then(|r| r.backfire_faction);
        let cells = self.summon_cells(caster, call.spread);
        let mut rng = draw!(self, "backfire/call/{}", self.state.actors.key(caster));
        let rolled = if full {
            (cast.roll).map_or(1, |r| r.roll(&mut rng, cast.reroll))
        } else {
            call.count.roll(&mut rng, 0)
        };
        let count = usize::try_from(rolled.max(1))
            .unwrap_or(1)
            .min(usize::from(call.most).max(1))
            .min(cells.len());
        if count == 0 {
            return false;
        }
        let mut came: Vec<(CreatureId, u32)> = Vec::new();
        for &pos in cells.iter().take(count) {
            let creature = self.summon_draw(caster, &call.pool, &bias);
            let spawn = CreatureSpawn {
                creature,
                faction,
                alignment: None,
                npc: None,
                pos,
                route: Vec::new(),
                carry: Vec::new(),
                equipment: Default::default(),
                rolls_gear: false,
                asleep: false,
                status: Vec::new(),
            };
            crate::world::levels::travel::make_creature(&mut self.state, &content, &level, &spawn);
            self.spawned += 1;
            match came.iter_mut().find(|(c, _)| *c == creature) {
                Some((_, n)) => *n += 1,
                None => came.push((creature, 1)),
            }
        }
        let words: Vec<String> = (came.iter())
            .map(|&(c, n)| {
                let def = content.creature(c);
                crate::world::allies::ally::few(&def.name, &def.plural, n)
            })
            .collect();
        let list = crate::text::capitalized(&and_list(
            &words.iter().map(String::as_str).collect::<Vec<_>>(),
        ));
        let verb = if count == 1 { "comes" } else { "come" };
        let player = self.state.player;
        let me = &self.state.actors[&caster];
        if caster == player {
            self.say(format!("{list} {verb} to your call, but not to your side!"));
        } else if self.perception.sees_actors_at(me.pos) {
            let who = self.called(me);
            self.say(format!(
                "{list} {verb} to {who}'s call, but not to its side!"
            ));
        }
        true
    }

    /// The fallback harm (module docs): force dice by spell level `level`.
    fn backfire_harm(&mut self, caster: ActorId, level: u8, full: bool, events: &mut Vec<Event>) {
        let dice = Dice::of(fallback_dice(level, full), 6);
        let mut rng = draw!(self, "backfire/harm/{}", self.state.actors.key(caster));
        let amount = dice.roll(&mut rng).max(1);
        let force = self.content.damage_type_id("force");
        let amount = self.resisted(caster, Damage::new(amount, force));
        if amount > 0 {
            self.hurt(caster, amount, Harm::Place, events);
        }
    }
}

/// How many d6 the fallback harm rolls for a spell of `level`: `level ÷ 3 + 1`, or `level ÷ 2 +
/// 1` when `full`.
pub(super) fn fallback_dice(level: u8, full: bool) -> u32 {
    let level = u32::from(level);
    if full { level / 2 + 1 } else { level / 3 + 1 }
}
