//! Learning spells (spec §8 "Spells", the magic design §4, the spell scaling design §5.4): reading
//! a grimoire ([`Action::Read`](crate::Action::Read) of one, `read.rs`; #1260).
//!
//! A character knows a set of spells for good: their class abilities' (talents) and those
//! learned from grimoires ([`Actor::spells`], saved). A grimoire ([`crate::content::grimoire`])
//! teaches the spell of its scroll, and is known on sight (no label):
//!
//! - **Refusals** (no time): a spell known at its last rank ("You know all there is of Frost."),
//!   no mana at all ("You have no gift for magic."), no training in Decipher script
//!   ([`UNTRAINED`]), and whatever a status blocks (Raging blocks `study`).
//! - **Learning** ([`Actor::learning`]): the spell level up to which a grimoire teaches for
//!   certain, the `learn` benefit of the skill that has one (Decipher script; #1270) at the
//!   skill's value with what items and talents add, at least 1. A character not trained in that
//!   skill has none and cannot learn at all, cantrips included; an item's or a talent's bonus
//!   alone trains nobody.
//!   Reading a scroll to cast it, and the spells a class grants, need none of this.
//! - **Time**: turn after turn of the `study` action ([`World::keep_at`], like a meal),
//!   `turns_base + turns_per_level × level` of them (`data/study.toml`), Quick study's `percent`
//!   less. It stops early as a rest does (a creature that disturbs a rest comes into view, a
//!   message is said: a blow, a status); then the time is lost.
//! - **The check** at the end: overreach is the spell's level less the learning. At 0 or less
//!   the spell is learned, no draw. Above, a 2d12 check ([`check::check`]) with the bonuses
//!   (Intelligence's modifier, the skills' `study` benefit (Decipher script), the
//!   class's affinity with the school, Quick study) against [`StudyRules::difficulty`] (the
//!   overreach, Overreach's third off, and the levels of the spells known). Made: learned.
//! - **Failure**: a mishap (`data/mishaps/`), one draw by weight among the band's and the lucky
//!   ones ([`crate::content::Band`], by the overreach), then its effect, through the code the
//!   game has: a status ([`World::afflict`]), the spell going off on the reader (its burst around
//!   them; a death by it is [`Cause::Scroll`], the grimoire's), mana lost, a teleport (the trap's),
//!   a stat training mark lost or won, a drain ([`World::drain`]: a Willpower check), a hostile
//!   summoned next to the reader, or the spell learned anyway.
//! - **The grimoire is kept** whatever comes of it: it may be read again after a failure (no
//!   limit but the time), and sold or kept after it taught.
//! - **Deepening** (#1264, `spell_rank.rs`): the grimoire of a spell already known, below
//!   [`MAX_SPELL_RANK`], is read for the next rank. The same reading, `rank` times as long (the
//!   rank sought) and always a check, [`StudyRules::rank_difficulty`] harder (`per_rank` for
//!   each rank held); the overreach counts as at least 0. Made: the rank is one higher ("The
//!   words settle deeper: you know Frost at rank 3 of 10."). Failed: "The words add nothing
//!   new.", the time gone, no mishap, and it may be read again at once (no limit but the
//!   time). A spell a class gives is deepened like any other.
//!
//! Determinism: the check's two draws only when there is a check, the mishap's one draw only on
//! a failure, then the effect's own (a backfire's damage roll, a drain's, a teleport's hash).
//! Training: the `study` use (Decipher script, Intelligence) at the end, whatever comes of it;
//! casting, not learning, trains Spellcraft (#1270).

use super::spell::Caster;
use super::{Actor, Cause, Ended, Spent, World};
use crate::action::Event;
use crate::character::Character;
use crate::check;
use crate::content::{
    ActionKind, Band, Content, CreatureSpawn, Drain, MAX_SPELL_RANK, Mishap, MishapEffect,
    ObjectId, SpellId, StudyRules,
};
use crate::geom::Dir;
use crate::skills::{Effect, Use};
use crate::stats::Stat;
use std::cell::Cell;

/// What a character not trained in Decipher script is told when they would read a grimoire.
pub const UNTRAINED: &str = "You have no training in Decipher script; you cannot make sense of how to learn a spell from a grimoire.";

/// A study under way: the grimoire read (its position and object as the study began), the
/// spell it teaches, the rank it is read for (`None`: to learn it), and the turns it takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Lesson {
    pub(super) item: usize,
    pub(super) object: ObjectId,
    pub(super) spell: SpellId,
    pub(super) rank: Option<u8>,
    pub(super) turns: u32,
}

/// How a study of a spell would go: its overreach, the rank it would bring (`None`: it would
/// teach the spell), and the check above the learning (`None`: certain).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Odds {
    /// The spell's level less the learning (before Overreach's cut, which only the check's
    /// difficulty reads).
    pub overreach: i32,
    /// The rank a deepening would reach, for a spell known; `None` for one to learn.
    pub rank: Option<u8>,
    /// The check's bonus and difficulty, at overreach 1 or more, and for any deepening.
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
    /// Whether it knows `spell` (spec §8 "Spells"): learned, or its class abilities give it,
    /// and not forgotten (rank 0 in [`Actor::spells`], `spell_rank.rs`).
    pub fn knows_spell(&self, content: &Content, spell: SpellId) -> bool {
        match self.spells.get(&spell) {
            Some(&rank) => rank > 0,
            None => self.talent_mods(content).has_spell(spell),
        }
    }

    /// The rank a reading of `spell`'s grimoire would bring it to: the next, for a spell known
    /// below [`MAX_SPELL_RANK`]; `None` for one not known (a reading would teach it) or at
    /// its last rank (nothing to read for).
    fn rank_sought(&self, content: &Content, spell: SpellId) -> Option<u8> {
        (self.spell_rank(content, spell))
            .filter(|&rank| rank < MAX_SPELL_RANK)
            .map(|rank| rank + 1)
    }

    /// The spells it knows that the content has, in menu order (by spell level, then path
    /// order).
    pub fn known_spells(&self, content: &Content) -> Vec<SpellId> {
        (content.spells_by_level().into_iter())
            .filter(|&s| self.knows_spell(content, s))
            .collect()
    }

    /// The spell level up to which a scroll teaches it for certain (module docs), at least 1;
    /// `None` when it is not trained in the skill that gives it, and so cannot learn from a
    /// scroll.
    pub fn learning(&self, content: &Content) -> Option<i32> {
        (self.skills.trains_effect(content, Effect::Learn))
            .then(|| self.skill_effect(content, Effect::Learn).max(1))
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

/// How `actor` (of `character`) would study `spell` (module docs): to learn it, or, knowing
/// it, to deepen it to the next rank (always a check); `None` when it cannot learn from a
/// grimoire ([`Actor::learning`]), knows the spell at its last rank, or without the study rules
/// or the spell's file.
pub fn study_odds(
    actor: &Actor,
    content: &Content,
    character: Option<&Character>,
    spell: SpellId,
) -> Option<Odds> {
    let (study, def) = (content.study_rules()?, content.spell_of(spell)?);
    let overreach = i32::from(def.level) - actor.learning(content)?;
    let rank = if actor.knows_spell(content, spell) {
        Some(actor.rank_sought(content, spell)?)
    } else {
        None
    };
    // A spell made hard to learn (`study_difficulty`, the owner's Q7 of #828) is a check at any
    // learning, and so is a deepening.
    if overreach <= 0 && def.study_difficulty <= 0 && rank.is_none() {
        return Some(Odds {
            overreach,
            rank,
            check: None,
        });
    }
    let mods = actor.talent_mods(content);
    let bonus = (actor.effective_stats(content).modifier(Stat::Intelligence))
        + actor.skill_effect(content, Effect::Study)
        + affinity(content, character, spell)
        + mods.study_bonus;
    // Overreach's third off, rounded down (the spell scaling design §7.2).
    let counted = overreach.max(0) * (100 + mods.study_overreach_percent).max(0) / 100;
    let difficulty = study.difficulty(counted, actor.known_levels(content))
        + def.study_difficulty
        + rank.map_or(0, |r| study.rank_difficulty(r));
    Some(Odds {
        overreach,
        rank,
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
        let character = self.state.character.as_ref();
        study_odds(self.state.player(), &self.content, character, spell)
    }

    /// The player begins to read grimoire stack `item`, which teaches `spell` (module docs), or
    /// is refused.
    pub(super) fn start_study(&mut self, item: usize, spell: SpellId) -> Spent {
        let content = self.content.clone();
        let me = self.state.player();
        let Some(stack) = me.inventory.get(item) else {
            self.say("You carry no such thing.");
            return Spent::Nothing;
        };
        let object = stack.object;
        // A spell known is read for its next rank; at the last there is nothing to read for.
        let rank = me.rank_sought(&content, spell);
        if me.knows_spell(&content, spell) && rank.is_none() {
            let name = content
                .spell_of(spell)
                .map_or("this spell", |d| d.name.as_str());
            self.say(format!("You know all there is of {name}."));
            return Spent::Nothing;
        }
        if me.max_mana(&content) <= 0 {
            self.say("You have no gift for magic.");
            return Spent::Nothing;
        }
        if me.learning(&content).is_none() {
            self.say(UNTRAINED);
            return Spent::Nothing;
        }
        let (Some(rules), Some(def)) = (content.study_rules(), content.spell_of(spell)) else {
            self.say("You cannot make sense of it.");
            return Spent::Nothing;
        };
        let percent = me.talent_mods(&content).study_percent;
        // A deepening reads the whole book over, rank times.
        let turns = study_turns(rules, def.level, percent) * u32::from(rank.unwrap_or(1));
        let title = self.lore().title(&stack.single());
        self.say(format!("You begin to read the {title}."));
        Spent::Study(Lesson {
            item,
            object,
            spell,
            rank,
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
            Ended::Came(came) => {
                let text = format!("{}; you stop reading.", self.came_line(came));
                self.say(text);
            }
            Ended::Said | Ended::Done => self.say("You stop reading."),
        }
    }

    /// The last turn of a study is done: the check, and the spell learned or a mishap (a
    /// deepening: the rank gained, or nothing); the grimoire is kept.
    fn finish_study(&mut self, lesson: Lesson, events: &mut Vec<Event>) {
        let pack = &self.state.player().inventory;
        let at = match pack.get(lesson.item) {
            Some(s) if s.object == lesson.object => Some(lesson.item),
            _ => pack.iter().position(|s| s.object == lesson.object),
        };
        if at.is_none() {
            self.say("The grimoire is gone.");
            return;
        }
        if self.trains(Use::Study) {
            self.train(Use::Study);
        }
        let odds = self.player_study_odds(lesson.spell);
        let learned = match odds.and_then(|o| o.check) {
            None => true,
            Some((bonus, difficulty)) => check::check(&mut self.state.rng, bonus, difficulty),
        };
        if let Some(rank) = lesson.rank {
            if learned {
                self.deepen_spell(lesson.spell, rank);
            } else {
                self.say("The words add nothing new.");
            }
            return;
        }
        if learned {
            self.learn_spell(lesson.spell);
            return;
        }
        self.say("The words twist away from you.");
        let overreach = odds.map_or(1, |o| o.overreach);
        let band = self
            .content
            .study_rules()
            .map_or(Band::Slight, |r| r.band(overreach));
        if let Some(mishap) = self.draw_mishap(band) {
            self.mishap(&mishap, lesson, events);
        }
    }

    /// The player knows `spell` at `rank` from now on (module docs: "Deepening").
    fn deepen_spell(&mut self, spell: SpellId, rank: u8) {
        let name = (self.content.spell_of(spell)).map_or_else(String::new, |d| d.name.clone());
        self.player_mut().set_spell_rank(spell, rank);
        self.say(format!(
            "The words settle deeper: you know {name} at rank {rank} of {MAX_SPELL_RANK}."
        ));
    }

    /// The player learns `spell` for good, at rank 1 (a spell known already keeps its rank).
    /// A forgotten spell (`spell_rank.rs`) is learned again as if for the first time, its
    /// experience paid again: there is no guard against it (the owner, #1289).
    pub(super) fn learn_spell(&mut self, spell: SpellId) {
        let name = (self.content.spell_of(spell)).map_or_else(String::new, |d| d.name.clone());
        let new = !self.state.player().knows_spell(&self.content, spell);
        if new {
            self.player_mut().set_spell_rank(spell, 1);
        }
        self.say(format!(
            "The words settle in your mind: you have learned {name}."
        ));
        // A spell learned by one's own effort pays each time it is learned: once, unless a
        // miscast made the caster forget it.
        let level = self.content.spell_of(spell).map_or(0, |d| d.level);
        let xp = self.content.experience().spell_xp(level);
        if new && xp > 0 {
            self.news
                .push(format!("You gain {xp} experience for learning {name}."));
            self.grant_xp(xp);
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
            MishapEffect::Backfire { .. } => self.backfire(lesson, events),
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
                times,
            } => {
                let drain = Drain {
                    stat,
                    points,
                    check: Stat::Willpower,
                    difficulty,
                };
                for _ in 0..times {
                    self.drain(player, drain, None, events);
                }
            }
            MishapEffect::Summon { creature } => self.summon(creature),
            MishapEffect::Learn => self.learn_spell(lesson.spell),
            // The miscast's only (`world/miscast.rs`): the loader keeps them off the study's table.
            MishapEffect::Harm { .. }
            | MishapEffect::StatDamage { .. }
            | MishapEffect::RankLoss { .. }
            | MishapEffect::Forget { .. }
            | MishapEffect::Refund
            | MishapEffect::Twice
            | MishapEffect::Swap
            | MishapEffect::Boon { .. }
            | MishapEffect::Noise { .. }
            | MishapEffect::Zone { .. }
            | MishapEffect::Portal { .. } => {}
        }
    }

    /// The spell of the grimoire read goes off on its reader (`backfire_spell`). A death by it
    /// is the grimoire's.
    fn backfire(&mut self, lesson: Lesson, events: &mut Vec<Event>) {
        self.backfire_spell(lesson.spell, lesson.object, events);
    }

    /// Spell `spell`, of the grimoire `object`, backfires: a damage spell goes off as a burst
    /// around the reader (its own radius, else 1) at the spell's base, no rung and no hooks (the
    /// spell scaling design §7.5); a spell of any other kind only fizzles (the mishap's message
    /// is all).
    pub(super) fn backfire_spell(
        &mut self,
        spell: SpellId,
        object: ObjectId,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let def = content.spell(spell);
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
        self.spell_lands(player, (def, &[]), made, None, events);
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
