//! Weapon abilities (spec §8 "Weapon skills", "Fighting styles"; the combat progression design
//! §4): what a rank in a weapon skill or a fighting style teaches, for the player and, by the same
//! rule and data, for every creature with the rank (`content/weapon_ability.rs` holds the data).
//!
//! **Who has one**: an actor whose rank in one of the ability's skills, as abilities read it
//! ([`Actor::ability_rank`]: a weapon talent's ranks and Early mastery count), is at least the
//! ability's. It *works* only through what the actor fights with: a triggered ability fires for
//! the skill and style of the attack concerned (the attack's for the attacker, what the actor
//! holds for the one attacked), and a switch (a free blow, Measure) is on while the actor wields
//! a weapon of the skill or fights in the style ([`World::ability_switches`]).
//!
//! **Triggers: one hook.** [`World::weapon_trigger`] is the one place triggered abilities fire.
//! It is called from the lines that already raise each event (`fight.rs::resolve` for hits,
//! criticals, big hits, kills, being struck, and the blocked and dodged attributions;
//! `ambush.rs::sneak_trigger` for a sneak attack that hits, from `resolve` and the spell attack;
//! `second_blow`'s end of an attack for a parry; the two-weapon attack for both hands;
//! `step_up` for a hostile approaching; `cast.rs::cast_single` for a spell's attack blocked by
//! a shield, `reflect.rs`), reads the abilities (a lookup: no draw) and applies
//! each effect in path order. Deciding never draws: a fight with no qualifying rank keeps its
//! draws, and an effect draws only as it lands (Pinning shot's turns, Rend's dice, a free shot).
//!
//! **Statuses that ease by actions** (Staggered, Pinned, Light-footed): a status with
//! `per_action` eases a degree with each action its sufferer takes, instead of with time
//! ([`World::spend_actions`], called where each action's price is paid): at one degree it is the
//! next action's, which is priced with it. One given during the sufferer's own action is not
//! eased by that action.

use super::fight::{End, death_message};
use super::free_blow::FreeBlows;
use super::shoot::Loose;
use super::{Actor, ActorId, AttackSkills, Spent, World};
use crate::action::{AttackKind, Event};
use crate::combat::{self, Blow, Category, Strike};
use crate::content::{
    Content, ExtraDice, FreeBlowKind, LevelKind, On, Stance, StatusId, Trigger, WeaponAbilityId,
    WeaponDoes, WeaponSkillId,
};
use crate::damage::Damage;
use crate::dice::Dice;
use crate::geom::Pos;
use crate::talent::TalentMods;

/// `text` with its first letter in upper case.
fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map_or_else(String::new, |c| c.to_uppercase().chain(chars).collect())
}

/// What an event a trigger fires on was ([`World::weapon_trigger`]): the other end of it (the
/// one struck, or the attacker), the kind of attack, the skills concerned, and the blow, if one.
#[derive(Clone, Copy, Debug)]
pub(super) struct Fired {
    pub(super) other: ActorId,
    pub(super) kind: AttackKind,
    pub(super) skills: AttackSkills,
    pub(super) blow: Option<Blow>,
}

/// The switches an actor's weapon abilities turn on, with what it fights with now.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Switches {
    pub(super) free_blows: FreeBlows,
    /// Its Parry counts against shots and throws (Measure).
    pub(super) parry_missiles: bool,
}

impl Actor {
    /// Its rank in weapon skill or style `skill` as weapon abilities read it (spec §8 "Weapon
    /// skills"): its own, raised by a weapon talent's ranks (Stylist, Fencing) and by Early
    /// mastery; an untrained skill stays 0.
    pub fn ability_rank(&self, content: &Content, skill: WeaponSkillId) -> u8 {
        let rank = self.weapon_skills.rank(skill);
        if rank == 0 || self.talents.is_empty() {
            return rank;
        }
        let def = content.weapon_skills().skill(skill);
        let talents = self.talent_mods(content);
        let (_, ranks) = talents.weapon_benefit(&def.id, def.role, rank);
        let raised = i32::from(TalentMods::raised(rank, ranks)) + talents.ability_ranks;
        u8::try_from(raised.max(0)).unwrap_or(u8::MAX)
    }

    /// The weapon abilities it knows, each once, with the skill it is listed under (the first
    /// of the ability's skills, in skill order, whose rank teaches it), in skill order, then
    /// rank, then path order: the abilities menu's order.
    pub fn weapon_abilities(&self, content: &Content) -> Vec<(WeaponSkillId, WeaponAbilityId)> {
        let mut known: Vec<(WeaponSkillId, u8, WeaponAbilityId)> = Vec::new();
        for (skill, _) in self.weapon_skills.trained() {
            let rank = self.ability_rank(content, skill);
            for (id, ability) in content.weapon_abilities_for(skill, rank) {
                if !known.iter().any(|&(_, _, k)| k == id) {
                    known.push((skill, ability.rank, id));
                }
            }
        }
        known.sort();
        known.into_iter().map(|(s, _, id)| (s, id)).collect()
    }

    /// The weapon abilities an attack drawing on `skills` brings (its weapon skill's and its
    /// style's), each once, in path order.
    pub(super) fn abilities_through(
        &self,
        content: &Content,
        skills: AttackSkills,
    ) -> Vec<WeaponAbilityId> {
        let mut found: Vec<WeaponAbilityId> = (skills.trained())
            .flat_map(|skill| {
                let rank = self.ability_rank(content, skill);
                content.weapon_abilities_for(skill, rank).map(|(id, _)| id)
            })
            .collect();
        found.sort();
        found.dedup();
        found
    }
}

impl World {
    /// The switches actor `actor`'s weapon abilities turn on with what it holds now: those of
    /// the wielded weapon's skill and of the style its hands are in (the one place they are
    /// read; [`World::free_blows_of`] folds the free blows in). No draw.
    pub(super) fn ability_switches(&self, actor: &Actor) -> Switches {
        let content = &self.content;
        let mut switches = Switches::default();
        if content.weapon_abilities().is_empty() || actor.weapon_skills.is_empty() {
            return switches;
        }
        let held = actor.attack_skills(content, AttackKind::Melee, None);
        for id in actor.abilities_through(content, held) {
            match content.weapon_ability(id).effect {
                WeaponDoes::FreeBlow(kind) => {
                    let blows = &mut switches.free_blows;
                    match kind {
                        FreeBlowKind::Riposte => blows.riposte = true,
                        FreeBlowKind::Cleave => blows.cleave = true,
                        FreeBlowKind::GreatCleave => blows.great_cleave = true,
                        FreeBlowKind::Opportunist => blows.opportunist = true,
                        FreeBlowKind::Gatekeeper => blows.gatekeeper = true,
                    }
                }
                WeaponDoes::ParryMissiles => switches.parry_missiles = true,
                // Triggered effects fire, and actives are used: no switch.
                _ => {}
            }
        }
        switches
    }

    /// What actor `id`'s Brace adds to its blow (Brace of steel): to-hit and a status on a foe
    /// it hits; `None` unless its gatekeeper's blow is a weapon ability's.
    pub(super) fn brace_of(&self, id: ActorId) -> Option<(i32, StatusId)> {
        let actor = self.state.actors.get(&id)?;
        let brace = actor.talent_mods(&self.content).brace?;
        self.ability_switches(actor)
            .free_blows
            .gatekeeper
            .then_some(brace)
    }

    /// The skills actor `id` defends with: those of a blow with what it holds.
    pub(super) fn held_skills(&self, id: ActorId) -> AttackSkills {
        self.skills_of(id, AttackKind::Melee, None)
    }

    /// Triggered abilities fire (the design §4.3): actor `by`'s that the attack's (or what it
    /// holds') skills bring and `trigger` fires, each effect in path order. The one place they
    /// fire. No draw to decide; an effect draws as it lands.
    pub(super) fn weapon_trigger(
        &mut self,
        by: ActorId,
        trigger: Trigger,
        fired: Fired,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        // Most events fire nothing: no ability is read for them.
        if !(content.weapon_abilities().iter()).any(|a| a.triggers.contains(&trigger)) {
            return;
        }
        let Some(actor) = self.state.actors.get(&by) else {
            return;
        };
        if actor.weapon_skills.is_empty() {
            return;
        }
        let abilities: Vec<WeaponDoes> = (actor.abilities_through(&content, fired.skills))
            .into_iter()
            .map(|id| content.weapon_ability(id))
            .filter(|a| a.triggers.contains(&trigger))
            .map(|a| a.effect)
            .collect();
        for effect in abilities {
            if self.state.death.is_some() {
                return;
            }
            match effect {
                WeaponDoes::Status {
                    status,
                    severity,
                    turns,
                    on,
                } => {
                    let whom = match on {
                        On::Target => fired.other,
                        On::Own => by,
                    };
                    let severity = match turns {
                        Some(dice) => {
                            let max = i32::from(content.status(status).max);
                            let rolled = dice.roll(&mut self.state.rng).clamp(1, max);
                            u16::try_from(rolled).unwrap_or(1)
                        }
                        None => severity,
                    };
                    self.afflict(whom, status, severity, Some(by), events);
                    let has = self
                        .state
                        .actors
                        .get(&whom)
                        .is_some_and(|a| a.effects.has(status));
                    if trigger.defending() && whom != by && has {
                        self.staggers_from(by, whom, trigger, status);
                    }
                }
                WeaponDoes::ExtraDamage { dice, kind } => {
                    self.extra_damage(by, fired, (dice, kind), events);
                }
                WeaponDoes::FreeShot => self.free_shot(by, fired, events),
                WeaponDoes::ReflectSpell { per_turn } => self.reflect_ready(by, per_turn),
                // Switches are read where their events are; actives have no trigger.
                _ => {}
            }
        }
    }

    /// The player's defending ability (Staggering block) left `whom`, who attacked them, with
    /// `status`: "The orc staggers from your shield." Said only for the player's, of one they
    /// see.
    fn staggers_from(&mut self, by: ActorId, whom: ActorId, trigger: Trigger, status: StatusId) {
        if by != self.state.player {
            return;
        }
        let Some(them) = self.state.actors.get(&whom) else {
            return;
        };
        if !self.perception.sees_actors_at(them.pos) {
            return;
        }
        let content = self.content.clone();
        let name = crate::content::the_start(&content.creature(them.creature).name);
        let def = content.status(status);
        let does = if def.id == "staggered" {
            "staggers".to_string()
        } else {
            format!(
                "is {}",
                def.seen.as_deref().unwrap_or(&def.name.to_lowercase())
            )
        };
        let from = match trigger {
            Trigger::Blocked => " from your shield",
            Trigger::Parried => " from your parry",
            _ => "",
        };
        self.news.push(format!("{name} {does}{from}."));
    }

    /// An `extra_damage` effect (Rend): `dice` rolled once more (no bonus, no riders) on the
    /// other end, of the blow's kind of damage (or `kind`), resisted but never soaked, after the
    /// blow's own damage; with Second blade's rend, the off hand's dice too. Said: "Your blades
    /// rend the orc."
    fn extra_damage(
        &mut self,
        by: ActorId,
        fired: Fired,
        (dice, kind): (ExtraDice, Option<crate::content::DamageTypeId>),
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let target = fired.other;
        let (Some(me), Some(them)) = (self.state.actors.get(&by), self.state.actors.get(&target))
        else {
            return;
        };
        let bare = |d: Dice| Dice::of(d.count(), d.sides());
        let main = me.weapon(&content).map(|(_, a)| bare(a.damage));
        let off = me.off_hand(&content).map(|(_, a)| bare(a.damage));
        let mut rolls: Vec<Dice> = match dice {
            ExtraDice::MainHand => main.into_iter().collect(),
            ExtraDice::OffHand => off.into_iter().collect(),
            ExtraDice::Fixed(d) => vec![d],
        };
        if dice == ExtraDice::MainHand && me.talent_mods(&content).rend_off_hand {
            rolls.extend(off);
        }
        if rolls.is_empty() {
            return;
        }
        let kind = kind.or(fired.blow.and_then(|b| b.kind));
        let player = self.state.player;
        let seen_them = self.perception.sees_actors_at(them.pos);
        let seen_me = self.perception.sees_actors_at(me.pos);
        let names = (
            content.creature(me.creature).name.clone(),
            content.creature(them.creature).name.clone(),
            content.creature(them.creature).death_verb().to_string(),
        );
        let amount: i32 = (rolls.iter())
            .map(|d| d.roll(&mut self.state.rng).max(0))
            .sum();
        let amount = self.resisted(target, Damage { amount, kind });
        let end = if seen_them {
            End::Seen(&names.1)
        } else {
            End::Unseen
        };
        if by == player {
            self.news
                .push(format!("Your blades rend {}.", end.object()));
        } else if target == player && seen_me {
            let who = crate::content::the(&names.0);
            self.news
                .push(format!("{}'s blades rend you.", capitalized(&who)));
        }
        if amount > 0 {
            let mut made = Vec::new();
            let died = self.hurt(target, amount, super::fight::Harm::Actor(by), &mut made);
            if died && by == player && seen_them {
                self.news.push(death_message(end, &names.2));
            }
            if by != player || seen_them {
                events.extend(made);
            }
        }
        if by == player {
            self.say_news();
        }
    }

    /// A `free_shot` effect (Point-blank): actor `by` looses what it has ready at the one that
    /// stepped up beside it, free (no time, no exertion): a shot with its missile weapon and the
    /// ammunition it has ready for an attack of [`AttackKind::Ranged`], else a throw of what it
    /// has ready to throw. Nothing without (no draw), on the overworld, or in a shop (the player).
    fn free_shot(&mut self, by: ActorId, fired: Fired, events: &mut Vec<Event>) {
        let content = self.content.clone();
        if self.state.current().kind == LevelKind::Overworld {
            return;
        }
        let (Some(me), Some(at)) = (
            self.state.actors.get(&by),
            self.state.actors.get(&fired.other).map(|a| a.pos),
        ) else {
            return;
        };
        let loose = match fired.kind {
            AttackKind::Ranged => {
                let (Some(profile), Some(item), Some(range)) = (
                    me.ranged(&content),
                    me.ammunition_item(&content),
                    me.launcher_range(&content),
                ) else {
                    return;
                };
                Loose {
                    kind: AttackKind::Ranged,
                    item,
                    profile,
                    range,
                }
            }
            AttackKind::Thrown => {
                let Some(item) = me.ready_to_throw(&content) else {
                    return;
                };
                let (Some(profile), Some(range)) =
                    (me.thrown(item, &content), me.throw_range(item, &content))
                else {
                    return;
                };
                Loose {
                    kind: AttackKind::Thrown,
                    item,
                    profile,
                    range,
                }
            }
            AttackKind::Melee | AttackKind::Spell => return,
        };
        if by != self.state.player {
            self.loose(by, at, loose, events);
            return;
        }
        if self.shop_here().is_some() {
            return;
        }
        // In another's turn: said after that turn's reports, as a free blow is.
        let earlier = std::mem::take(&mut self.news);
        self.held = Some(Vec::new());
        self.player_looses(at, loose, events);
        let held = self.held.take().unwrap_or_default();
        self.news = earlier;
        self.news.extend(held);
    }

    /// The missile skills of actor `id` for a free shot: its missile weapon's (a shot), else
    /// what it has ready to throw (a throw); `None` with neither.
    fn missile_skills(&self, id: ActorId) -> Option<(AttackKind, AttackSkills)> {
        let me = self.state.actors.get(&id)?;
        if me.launcher(&self.content).is_some() {
            return Some((
                AttackKind::Ranged,
                self.skills_of(id, AttackKind::Ranged, None),
            ));
        }
        let item = me.ready_to_throw(&self.content)?;
        let thrown = me.inventory.get(item)?.object;
        Some((
            AttackKind::Thrown,
            self.skills_of(id, AttackKind::Thrown, Some(thrown)),
        ))
    }

    /// Actor `mover` has just stepped from `from` up beside others: each hostile it was not
    /// beside before fires what a hostile approaching fires (Point-blank), in actor order, while
    /// the mover stands.
    pub(super) fn approached(&mut self, mover: ActorId, from: Pos, events: &mut Vec<Event>) {
        if !(self.content.weapon_abilities().iter())
            .any(|a| a.triggers.contains(&Trigger::Approaches))
        {
            return;
        }
        let Some(me) = self.state.actors.get(&mover) else {
            return;
        };
        let near: Vec<ActorId> = (self.state.actors.values())
            .filter(|other| other.id != mover && other.level == me.level)
            .filter(|other| other.pos.chebyshev(me.pos) == 1 && other.pos.chebyshev(from) > 1)
            .filter(|other| !other.weapon_skills.is_empty())
            // Surprised, it fires nothing out of its turn (`surprise.rs`).
            .filter(|other| !self.surprised(other.id))
            .filter(|other| self.stance(mover, other.id) == Stance::Hostile)
            .map(|other| other.id)
            .collect();
        for keeper in near {
            if !self.state.actors.contains_key(&mover) || self.state.death.is_some() {
                return;
            }
            let Some((kind, skills)) = self.missile_skills(keeper) else {
                continue;
            };
            let fired = Fired {
                other: mover,
                kind,
                skills,
                blow: None,
            };
            self.weapon_trigger(keeper, Trigger::Approaches, fired, events);
        }
    }

    /// The end of a two-weapon attack by `attacker` on `target`: both hands' blows, `first` and
    /// `second` (`second_with` what it was made with), hit and the target lives: Rend's trigger,
    /// once per attack. A shield bash is no second hand, nor a flurry's bare fist.
    pub(super) fn both_hands(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        (first, second): (Option<Blow>, Option<Blow>),
        second_with: Option<crate::content::ObjectId>,
        events: &mut Vec<Event>,
    ) {
        let hit = |b: Option<Blow>| b.is_some_and(|b| matches!(b.strike, Strike::Hit { .. }));
        let weapon = second_with.is_some_and(|w| self.content.object(w).bash.is_none());
        if !(hit(first) && hit(second) && weapon && self.state.actors.contains_key(&target)) {
            return;
        }
        let fired = Fired {
            other: target,
            kind: AttackKind::Melee,
            skills: self.skills_of(attacker, AttackKind::Melee, None),
            blow: second,
        };
        self.weapon_trigger(attacker, Trigger::BothHandsHit, fired, events);
    }

    /// What resolving an attack of `kind` drawing on `skills` by `attacker` on `target` fires
    /// (`fight.rs::resolve`, beside the resources' builds): a hit and a critical for the
    /// attacker, being struck for the target; a melee attack's miss by the target's Shield, or
    /// by its Dexterity or Dodge, for the target.
    pub(super) fn resolve_triggers(
        &mut self,
        (attacker, target): (ActorId, ActorId),
        (kind, skills): (AttackKind, AttackSkills),
        blow: Option<Blow>,
        events: &mut Vec<Event>,
    ) {
        let Some(b) = blow else {
            return;
        };
        let at = |other, skills| Fired {
            other,
            kind,
            skills,
            blow: Some(b),
        };
        match b.strike {
            Strike::Hit { critical } => {
                self.weapon_trigger(attacker, Trigger::Hit, at(target, skills), events);
                if critical {
                    self.weapon_trigger(attacker, Trigger::Critical, at(target, skills), events);
                }
                let held = self.held_skills(target);
                self.weapon_trigger(target, Trigger::Struck, at(attacker, held), events);
            }
            Strike::Soaked | Strike::Resisted => {
                self.weapon_trigger(attacker, Trigger::Hit, at(target, skills), events);
                let held = self.held_skills(target);
                self.weapon_trigger(target, Trigger::Struck, at(attacker, held), events);
            }
            Strike::Missed { by } if kind == AttackKind::Melee => {
                let trigger = match by {
                    Some(Category::Shield) => Trigger::Blocked,
                    Some(Category::Dexterity | Category::Dodge) => Trigger::Dodged,
                    _ => return,
                };
                let held = self.held_skills(target);
                self.weapon_trigger(target, trigger, at(attacker, held), events);
            }
            Strike::Missed { .. } => {}
        }
    }

    /// A blow by `attacker` that left `target` alive was a big hit (a third of its hit points or
    /// more; `combat::big_hit`): Deep cut's and Crushing blow's trigger.
    pub(super) fn big_hit_trigger(
        &mut self,
        (attacker, target): (ActorId, ActorId),
        (kind, skills): (AttackKind, AttackSkills),
        blow: Blow,
        events: &mut Vec<Event>,
    ) {
        let Some(victim) = self.state.actors.get(&target) else {
            return;
        };
        if !combat::big_hit(blow.damage, victim.max_hp(&self.content)) {
            return;
        }
        let fired = Fired {
            other: target,
            kind,
            skills,
            blow: Some(blow),
        };
        self.weapon_trigger(attacker, Trigger::BigHit, fired, events);
    }

    /// Actor `id`'s action is taken (its price paid): each of its statuses with `per_action`
    /// eases a degree (gone at none, an [`Event::Status`] end: "You recover your wits."), but
    /// those given during that action.
    pub(super) fn spend_actions(&mut self, id: ActorId, events: &mut Vec<Event>) {
        let fresh: Vec<StatusId> = (self.fresh.iter())
            .filter(|(a, _)| *a == id)
            .map(|&(_, s)| s)
            .collect();
        self.fresh.retain(|(a, _)| *a != id);
        self.reflections_spent(id);
        let content = self.content.clone();
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        let spent: Vec<StatusId> = (actor.effects.iter())
            .map(|e| e.status)
            .filter(|&s| content.status(s).per_action && !fresh.contains(&s))
            .collect();
        let surprise = content.surprise().map(|s| s.status);
        let mut steadied = false;
        for status in spent {
            let left = (actor.effects.get_mut(status)).map_or(0, |e| {
                e.severity = e.severity.saturating_sub(1);
                e.severity
            });
            if left == 0 {
                actor.effects.remove(status);
                steadied |= Some(status) == surprise;
                events.push(Event::Status {
                    actor: id,
                    status,
                    began: false,
                });
            }
        }
        // Recovering one's wits after a surprise trains Willpower.
        if steadied && id == self.state.player {
            self.train_stat(crate::content::StatUse::Steadied);
        }
    }

    /// The weapon abilities the player has learned, as a rank risen with marks says them:
    /// "You have learned the deep cut.", each with the skill it is listed under and whether it
    /// is an active.
    pub(super) fn learned(
        &self,
        before: &[(WeaponSkillId, WeaponAbilityId)],
    ) -> Vec<(WeaponSkillId, String, bool)> {
        let content = &self.content;
        (self.state.player().weapon_abilities(content).into_iter())
            .filter(|(_, id)| !before.iter().any(|(_, b)| b == id))
            .map(|(skill, id)| {
                let ability = content.weapon_ability(id);
                let name = ability.name.to_lowercase();
                let line = format!("You have learned the {name}.");
                (skill, line, ability.active.is_some())
            })
            .collect()
    }

    /// The dev action `rank:SKILL:N` ([`crate::action::Action::SetRank`]): the player's rank in
    /// weapon skill or style `skill` becomes `rank` (the skills it raises by default rise too),
    /// said as a rank risen is, with what the ranks teach. Takes no time.
    pub(super) fn set_rank_dev(&mut self, skill: &str, rank: u8) -> Spent {
        let content = self.content.clone();
        let table = content.weapon_skills();
        let Some(id) = table.id(skill) else {
            self.say(format!("There is no weapon skill `{skill}`."));
            return Spent::Nothing;
        };
        let before = self.state.player().weapon_abilities(&content);
        let risen = self.player_mut().weapon_skills.raise(table, id, rank);
        self.ranks_news(&risen, &before);
        self.say_news();
        Spent::NoTime
    }
}
