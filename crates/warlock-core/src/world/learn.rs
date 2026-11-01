//! Learning spells (spec §8 "Spells", the magic design §4, the spell scaling design §5.4):
//! [`Action::Study`](crate::Action::Study).
//!
//! A character knows a set of spells for good: their class abilities' (talents) and those
//! learned by study ([`Actor::spells`], saved). A spell scroll whose `read` table `teaches` a
//! spell can be studied once the player knows the scroll's type (identified by reading any copy
//! or by a scroll of identify), unless they know its spell already:
//!
//! - **Refusals** (no time, the scroll kept): no such stack, a thing that teaches nothing ("There
//!   is no spell in it to learn."), a scroll under its label ("You do not know what this says."),
//!   a spell known ("You know this spell already."), no mana at all ("You have no gift for
//!   magic."), and whatever a status blocks (Raging blocks `study`).
//! - **Time**: turn after turn of the `study` action ([`World::keep_at`], like a meal),
//!   `turns_base + turns_per_level × level` of them (`data/study.toml`), Quick study's `percent`
//!   less. It stops
//!   early as a rest does (a creature that disturbs a rest comes into view, a message is said:
//!   a blow, a status); then the time is lost and the scroll kept (design §14 q1's default).
//! - **The check** at the end, on the scroll used up (Scroll savant never keeps it: §14 q7):
//!   overreach is the spell's level less the caster level. At 0 or less the spell is learned, no
//!   draw. Above, a 2d12 check ([`check::check`]) with the bonuses (Intelligence's modifier, the
//!   skills' `study` benefit (Spellcraft, Decipher script), the class's affinity with the
//!   school, Quick study) against [`StudyRules::difficulty`] (the overreach, Overreach's third
//!   off, and the levels of the spells known). Made: learned.
//! - **Failure**: a mishap (`data/mishaps/`), one draw by weight among the band's and the lucky
//!   ones ([`crate::content::Band`], by the overreach), then its effect, through the code the
//!   game has: a status ([`World::afflict`]), the scroll's own spell going off on the reader
//!   (its burst around them, or its bolt on them; a death by it is [`Cause::Scroll`]), mana lost,
//!   a teleport (the trap's), a stat training mark lost or won, a drain ([`World::drain`]: a
//!   Willpower check), a hostile summoned next to the reader, or the spell learned anyway.
//!
//! Determinism: the check's two draws only when there is a check, the mishap's one draw only on
//! a failure, then the effect's own (a backfire's damage roll, a drain's, a teleport's hash).
//! Training: the `study` use (Spellcraft, Decipher script, Intelligence) at the end, whatever
//! comes of it.

use super::spell::Caster;
use super::{Actor, Cause, Ended, Spent, World};
use crate::action::{Event, Positions};
use crate::character::Character;
use crate::check;
use crate::content::{
    ActionKind, Band, Content, CreatureSpawn, Drain, Mishap, MishapEffect, ObjectId, Reading,
    SpellId, StudyRules,
};
use crate::geom::Dir;
use crate::skills::{Effect, Use};
use crate::stats::Stat;
use std::cell::Cell;

/// A study under way: the stack studied (its position and object as the study began), the
/// spell it teaches, and the turns it takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Lesson {
    pub(super) item: usize,
    pub(super) object: ObjectId,
    pub(super) spell: SpellId,
    pub(super) turns: u32,
}

/// How a study of a spell would go: its overreach, and the check above the caster level
/// (`None`: certain).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Odds {
    /// The spell's level less the caster level (before Overreach's cut, which only the check's
    /// difficulty reads).
    pub overreach: i32,
    /// The check's bonus and difficulty, at overreach 1 or more.
    pub check: Option<(i32, i32)>,
}

impl Odds {
    /// The ways of the 144 the study succeeds.
    pub fn ways(&self) -> u32 {
        match self.check {
            None => 144,
            Some((bonus, difficulty)) => check::ways(bonus, difficulty),
        }
    }
}

impl Actor {
    /// Whether it knows `spell` (spec §8 "Spells"): learned, or its class abilities give it.
    pub fn knows_spell(&self, content: &Content, spell: SpellId) -> bool {
        self.spells.contains(&spell) || self.talent_mods(content).has_spell(spell)
    }

    /// The spells it knows that the content has, in menu order (by spell level, then path
    /// order).
    pub fn known_spells(&self, content: &Content) -> Vec<SpellId> {
        (content.spells_by_level().into_iter())
            .filter(|&s| self.knows_spell(content, s))
            .collect()
    }

    /// The levels of the spells it knows, added up (the study check's small cumulative
    /// penalty).
    fn known_levels(&self, content: &Content) -> i32 {
        (self.known_spells(content).into_iter())
            .filter_map(|s| content.spell_of(s))
            .map(|d| i32::from(d.level))
            .sum()
    }
}

/// The class's affinity with the school of `spell` (spec §8 "Spells"): study check points; 0
/// without a class.
fn affinity(content: &Content, character: Option<&Character>, spell: SpellId) -> i32 {
    let Some(def) = content.spell_of(spell) else {
        return 0;
    };
    (character.and_then(|c| c.spec.class(content).ok()))
        .and_then(|class| class.affinity.get(&def.school).copied())
        .map_or(0, i32::from)
}

/// How `actor` (of `character`, at caster level `caster_level`) would study `spell` (module
/// docs); `None` without the study rules or the spell's file.
pub fn study_odds(
    actor: &Actor,
    content: &Content,
    character: Option<&Character>,
    caster_level: i32,
    spell: SpellId,
) -> Option<Odds> {
    let (study, def) = (content.study_rules()?, content.spell_of(spell)?);
    let overreach = i32::from(def.level) - caster_level;
    if overreach <= 0 {
        return Some(Odds {
            overreach,
            check: None,
        });
    }
    let mods = actor.talent_mods(content);
    let bonus = (actor.effective_stats(content).modifier(Stat::Intelligence))
        + actor.skill_effect(content, Effect::Study)
        + affinity(content, character, spell)
        + mods.study_bonus;
    // Overreach's third off, rounded down (the spell scaling design §7.2).
    let counted = overreach * (100 + mods.study_overreach_percent).max(0) / 100;
    let difficulty = study.difficulty(counted, actor.known_levels(content));
    Some(Odds {
        overreach,
        check: Some((bonus, difficulty)),
    })
}

/// How many turns studying a spell of `level` takes with Quick study's `percent`, at least 1:
/// `turns_base + turns_per_level × level`, scaled.
pub fn study_turns(rules: &StudyRules, level: u8, percent: i32) -> u32 {
    let turns = u64::from(rules.turns_base) + u64::from(rules.turns_per_level) * u64::from(level);
    let scaled = turns * u64::try_from((100 + percent).max(0)).unwrap_or(0) / 100;
    u32::try_from(scaled).unwrap_or(u32::MAX).max(1)
}

impl World {
    /// The odds of the player studying `spell` now.
    pub(crate) fn player_study_odds(&self, spell: SpellId) -> Option<Odds> {
        let level = self.caster_level(self.state.player);
        let character = self.state.character.as_ref();
        study_odds(self.state.player(), &self.content, character, level, spell)
    }

    /// The player begins to study inventory stack `item` (module docs), or is refused.
    pub(super) fn start_study(&mut self, item: usize) -> Spent {
        let content = self.content.clone();
        let me = self.state.player();
        let Some(stack) = me.inventory.get(item) else {
            self.say("You carry no such thing.");
            return Spent::Nothing;
        };
        let object = stack.object;
        let what = self.lore().describe(&stack.single());
        let teaches = (content.object(object).read.as_ref()).and_then(Reading::teaches);
        let Some(spell) = teaches.filter(|&s| content.spell_of(s).is_some()) else {
            if self.lore().label(object).is_some() {
                self.say("You do not know what this says.");
            } else {
                self.say(format!("There is no spell in {what} to learn."));
            }
            return Spent::Nothing;
        };
        if self.lore().label(object).is_some() {
            self.say("You do not know what this says.");
            return Spent::Nothing;
        }
        if me.knows_spell(&content, spell) {
            self.say("You know this spell already.");
            return Spent::Nothing;
        }
        if me.max_mana(&content) <= 0 {
            self.say("You have no gift for magic.");
            return Spent::Nothing;
        }
        let (Some(rules), Some(def)) = (content.study_rules(), content.spell_of(spell)) else {
            self.say("You cannot study it.");
            return Spent::Nothing;
        };
        let percent = me.talent_mods(&content).study_percent;
        let turns = study_turns(rules, def.level, percent);
        let title = self.lore().title(&stack.single());
        self.say(format!("You begin to study the {title}."));
        Spent::Study(Lesson {
            item,
            object,
            spell,
            turns,
        })
    }

    /// Studies turn after turn until the lesson is done or something interrupts (module docs).
    pub(super) fn study(&mut self, lesson: Lesson, events: &mut Vec<Event>) {
        let left = Cell::new(lesson.turns);
        let ended = self.keep_at(
            events,
            |_| left.get() > 0,
            |w, _| {
                left.set(left.get() - 1);
                w.price_of(w.state.player, ActionKind::Study)
            },
        );
        match ended {
            Ended::Died => {}
            _ if left.get() == 0 => self.finish_study(lesson, events),
            Ended::Came(id) => {
                let text = format!(
                    "{} comes into view; you stop studying.",
                    self.creature_called(id)
                );
                self.say(text);
            }
            Ended::Said | Ended::Done => self.say("You stop studying."),
        }
    }

    /// The last turn of a study is done: the scroll is used up, then the check, and the spell
    /// learned or a mishap.
    fn finish_study(&mut self, lesson: Lesson, events: &mut Vec<Event>) {
        let pack = &self.state.player().inventory;
        let at = match pack.get(lesson.item) {
            Some(s) if s.object == lesson.object => Some(lesson.item),
            _ => pack.iter().position(|s| s.object == lesson.object),
        };
        let Some(at) = at else {
            self.say("The scroll is gone.");
            return;
        };
        if self.trains(Use::Study) {
            self.train(Use::Study);
        }
        let odds = self.player_study_odds(lesson.spell);
        self.use_scroll(at);
        let learned = match odds.and_then(|o| o.check) {
            None => true,
            Some((bonus, difficulty)) => check::check(&mut self.state.rng, bonus, difficulty),
        };
        if learned {
            self.learn_spell(lesson.spell);
            return;
        }
        self.say("The words twist away from you, and the scroll crumbles to dust.");
        let overreach = odds.map_or(1, |o| o.overreach);
        let band = self
            .content
            .study_rules()
            .map_or(Band::Slight, |r| r.band(overreach));
        if let Some(mishap) = self.draw_mishap(band) {
            self.mishap(&mishap, lesson, events);
        }
    }

    /// The player learns `spell` for good.
    fn learn_spell(&mut self, spell: SpellId) {
        let name = (self.content.spell_of(spell)).map_or_else(String::new, |d| d.name.clone());
        self.player_mut().spells.insert(spell);
        self.say(format!(
            "The scroll crumbles as its words settle in your mind: you have learned {name}."
        ));
    }

    /// Takes one of inventory stack `item` from the pack (the last empties it).
    fn use_scroll(&mut self, item: usize) {
        let player = self.player_mut();
        let Some(stack) = player.inventory.get_mut(item) else {
            return;
        };
        stack.count -= 1;
        if stack.count == 0 {
            player.inventory.remove(item);
            player.equipment.after_removal(&Positions::of([item]));
        }
    }

    /// One draw by weight among the mishaps of `band` and the lucky ones, if there are any.
    pub(super) fn draw_mishap(&mut self, band: Band) -> Option<Mishap> {
        let content = self.content.clone();
        let among: Vec<&Mishap> = crate::content::drawn_among(content.mishaps(), band).collect();
        let total: u32 = among.iter().map(|m| u32::from(m.weight)).sum();
        if total == 0 {
            return None;
        }
        let mut roll = self.state.rng.below(total);
        for m in among {
            let w = u32::from(m.weight);
            if roll < w {
                return Some(m.clone());
            }
            roll -= w;
        }
        None
    }

    /// What `mishap` does to the player who failed to learn `lesson`'s spell (module docs).
    pub(super) fn mishap(&mut self, mishap: &Mishap, lesson: Lesson, events: &mut Vec<Event>) {
        self.say(mishap.message.clone());
        let player = self.state.player;
        let here = self.player_pos();
        match mishap.effect {
            MishapEffect::Nothing => {}
            MishapEffect::Status { status, severity } => {
                self.afflict(player, status, severity, None, events);
            }
            MishapEffect::Backfire => self.backfire(lesson, events),
            MishapEffect::Mana { percent } => {
                let me = self.player_mut();
                let lost = (me.mana.max(0) * i32::from(percent) + 99) / 100;
                me.mana = (me.mana - lost).max(0);
            }
            MishapEffect::Teleport => self.teleport(player, here, events),
            MishapEffect::LoseMark => {
                let marked: Vec<Stat> = (Stat::ALL.iter().copied())
                    .filter(|&s| self.state.player().stat_marks[s] > 0)
                    .collect();
                if !marked.is_empty() {
                    let i = self.state.rng.below(marked.len() as u32) as usize;
                    let marks = &mut self.player_mut().stat_marks[marked[i]];
                    *marks -= 1;
                }
            }
            MishapEffect::Mark { stat } => {
                let marks = &mut self.player_mut().stat_marks[stat];
                *marks = marks.saturating_add(1);
            }
            MishapEffect::Drain {
                stat,
                points,
                difficulty,
            } => {
                let drain = Drain {
                    stat,
                    points,
                    check: Stat::Willpower,
                    difficulty,
                };
                self.drain(player, drain, None, events);
            }
            MishapEffect::Summon { creature } => self.summon(creature),
            MishapEffect::Learn => self.learn_spell(lesson.spell),
        }
    }

    /// The spell of the scroll studied goes off on its reader (`backfire_spell`). A death by it
    /// is the scroll's.
    fn backfire(&mut self, lesson: Lesson, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let object = lesson.object;
        if let Some(Reading::Cast(scroll)) = &content.object(object).read {
            self.backfire_spell(*scroll, object, events);
        }
    }

    /// A scroll of a spell of `data/spells/` backfires: a damage spell goes off as a burst
    /// around the reader (its own radius, else 1) at the spell's base, no rung and no hooks (the
    /// spell scaling design §7.5); a spell of any other kind only fizzles (the mishap's message
    /// is all).
    fn backfire_spell(
        &mut self,
        scroll: crate::content::SpellScroll,
        object: ObjectId,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let def = content.spell(scroll.spell);
        let Some(damage) = def.damage else {
            return;
        };
        let burst = crate::content::Burst {
            spell: def.look.effect.clone().unwrap_or_default(),
            radius: def.radius.map_or(1, i32::from),
            damage,
            reroll: 0,
            kind: def.damage_type,
            blast: 0,
            ice: None,
        };
        let player = self.state.player;
        let here = self.player_pos();
        let mut made = Vec::new();
        self.sound_at(def.look.noise, here);
        self.cast_burst(Caster::Actor(player), (here, here), &burst, &mut made);
        self.say_spell((&def.look.noun, &def.look.verb), made, events);
        if let Some(death) = &mut self.state.death {
            death.by = None;
            death.cause = Some(Cause::Scroll(object));
        }
    }

    /// A hostile `creature` comes to the first free cell next to the player, if there is one.
    fn summon(&mut self, creature: crate::content::CreatureId) {
        let level = self.state.current();
        let here = self.player_pos();
        let free = (Dir::ALL.iter().map(|&d| here.step(d))).find(|&p| {
            level.is_walkable(&self.content, p) && self.state.actor_at(&level.id, p).is_none()
        });
        let Some(pos) = free else {
            return;
        };
        let id = level.id.clone();
        let spawn = CreatureSpawn {
            creature,
            faction: None,
            alignment: None,
            npc: None,
            pos,
            route: Vec::new(),
            carry: Vec::new(),
            equipment: Default::default(),
            rolls_gear: true,
            asleep: false,
            status: Vec::new(),
        };
        super::travel::make_creature(&mut self.state, &self.content, &id, &spawn);
        self.spawned += 1;
    }
}
