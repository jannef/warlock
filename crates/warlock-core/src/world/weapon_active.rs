//! Active weapon abilities in the world (spec §8 "Weapon skills", "Fighting styles"; the combat
//! progression design §4.5, §4.6, §6; #564): what a rank teaches that the player uses from the
//! abilities menu (`Ability::Weapon`), as `use:lunge:E` ([`crate::action::Action::Act`]) or
//! `use:shield_wall`, `use:piercing_shot:12,5` ([`crate::action::Action::Cast`] of an id that is
//! no spell's: resolved after the spells).
//!
//! **Refusals first, then the costs.** An active is refused, taking no time and spending
//! nothing, without the ability, for a triggered one, while what the player holds is not for it
//! ([`Actor::holds_for`]: a weapon of its skill, its style's grip, a shield for shield blocking's),
//! short of stamina, while a status blocks the action it is, without the aim it asks, and by its
//! own refusals (nobody hostile there, no room to lunge, nothing to shoot). Then it pays both its
//! costs ([`World::pay_active`]): its stamina through `World::spend_resource` (less Drillmaster's
//! and Armour-breaker's discounts and what of Ease takes off, [`Actor::weapon_stamina`], at least
//! 1), and its time: the price
//! of the action it is (`attack_price` for a blow, so weapon speed, style, stance, and statuses
//! count; `shot_price` for a shot; `wait` for a buff), priced before it is made, times its
//! `percent` ([`World::active_price`]: Practised technique's cut, never below half the action).
//!
//! **What its blow or shot does beyond a plain one** is set on its profile (a sure critical,
//! Protection ignored, the dice twice, a sweep's to-hit), or, where the profile cannot carry it,
//! on the transient [`Technique`] read where the attack is resolved (Feint's Defence categories,
//! `fight.rs::resolve`; Piercing shot's flight, `shoot.rs::fly`) and cleared after it. Creatures
//! use no actives. Deciding never draws: the blows draw as any blow does.

use super::{Actor, ActorId, AttackSkills, Spent, World};
use crate::ability::Ability;
use crate::action::{AttackKind, Event};
use crate::combat::{Blow, Profile, Target};
use crate::content::Effect as ItemEffect;
use crate::content::{
    ActionKind, ActiveAim, Categories, Content, STAMINA, StatusId, WeaponAbilityId, WeaponDoes,
    percent_of,
};
use crate::dice::Dice;
use crate::geom::{Dir, Pos};

/// The penetration an `ignore_protection` blow or shot has: more than any Protection, so none
/// of it soaks (and no soak is drawn).
const ALL_PROTECTION: i32 = 1_000;

/// The least an active's time may be cut to by talents, in percent of the action's price.
const LEAST_PERCENT: i64 = 50;

/// The player's weapon active under way (not saved): what its attack does beyond a plain one
/// that its profile cannot carry, read where the attack is resolved, cleared after it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Technique {
    /// The Defence categories the next attack the player makes is rolled without (Feint): taken
    /// by that attack.
    pub(super) ignores: Categories,
    /// The next shot the player looses flies on past the first it hits (Piercing shot): taken
    /// by that shot.
    pub(super) pierce: bool,
}

/// What a weapon active was aimed at, as the action text gives it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Aimed {
    Nothing,
    Dir(Dir),
    Cell(Pos),
}

impl Actor {
    /// Whether what it holds now is for weapon active `id` (spec §8 "Weapon skills"): the
    /// abilities its attack of that kind brings (a blow's weapon skill and style, a shot's) teach
    /// it, or, for one shield blocking teaches, it holds a shield and its rank teaches it.
    pub(crate) fn holds_for(&self, content: &Content, id: WeaponAbilityId) -> bool {
        let ability = content.weapon_ability(id);
        let Some(active) = ability.active else {
            return false;
        };
        let kind = match active.action {
            ActionKind::Shoot => AttackKind::Ranged,
            _ => AttackKind::Melee,
        };
        let skills = self.attack_skills(content, kind, None);
        if self.abilities_through(content, skills).contains(&id) {
            return true;
        }
        let block = content
            .weapon_skills()
            .of_role(crate::content::Role::ShieldBlock);
        block.is_some_and(|block| {
            ability.skills.contains(&block)
                && self.holds_shield(content)
                && self.ability_rank(content, block) >= ability.rank
        })
    }

    /// Whether it holds a shield in a hand.
    fn holds_shield(&self, content: &Content) -> bool {
        (self.equipment.items().into_iter())
            .filter(|&(slot, _)| content.slot(slot).hand)
            .filter_map(|(_, item)| self.inventory.get(item))
            .any(|s| content.object(s.object).shield > 0)
    }

    /// The stamina weapon active `id` costs it: its own, less its talents' discounts
    /// (Drillmaster's for every active, Armour-breaker's for Sunder and Gap shot) and what it
    /// wears takes off every active (of Ease's `active_stamina`, global: whatever piece holds
    /// it), at least 1.
    pub(crate) fn weapon_stamina(&self, content: &Content, id: WeaponAbilityId) -> u16 {
        let own = content
            .weapon_ability(id)
            .active
            .map_or(0, |a| i32::from(a.stamina));
        let worn = content.resource_id(STAMINA).map_or(0, |stamina| {
            (self.mods(content)).resource(ItemEffect::ActiveStamina, stamina)
        });
        if self.talents.is_empty() && worn == 0 {
            return u16::try_from(own).unwrap_or(0);
        }
        let talents = self.talent_mods(content);
        let named: i32 = (talents.ability_stamina.iter())
            .filter(|(a, _)| *a == id)
            .map(|(_, v)| v)
            .sum();
        u16::try_from((own - talents.weapon_stamina - named - worn).max(1)).unwrap_or(1)
    }
}

impl World {
    /// `use:ID[:DIR|:X,Y]` of weapon active `name` (module docs), aimed `aim`. Refused, taking
    /// no time, as the module docs say.
    pub(super) fn use_weapon(&mut self, name: &str, aim: Aimed, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let me = self.state.player();
        let known = (content.weapon_ability_id(name))
            .and_then(|id| (me.weapon_abilities(&content).into_iter()).find(|&(_, a)| a == id));
        let Some((skill, id)) = known else {
            self.say("You have no such ability.");
            return Spent::Nothing;
        };
        let def = content.weapon_ability(id);
        let Some(active) = def.active else {
            self.say("That fires by itself.");
            return Spent::Nothing;
        };
        if let Some(why) = Ability::Weapon(skill, id).unavailable(me, &content) {
            self.say(format!("{}.", capitalized(&why.reason(&content))));
            return Spent::Nothing;
        }
        let effect = def.effect;
        match (active.aim, aim) {
            (ActiveAim::Direction, Aimed::Dir(dir)) => self.active_blow(id, effect, dir, events),
            (ActiveAim::Cell, Aimed::Cell(at)) => self.active_shot(id, effect, at, events),
            // A sweep may start where it is aimed; a buff asks nothing.
            (ActiveAim::Nothing, Aimed::Nothing | Aimed::Dir(_)) => {
                let from = match aim {
                    Aimed::Dir(dir) => dir,
                    _ => Dir::N,
                };
                match effect {
                    WeaponDoes::Sweep { to_hit } => self.sweep(id, from, to_hit, events),
                    WeaponDoes::Buff { status, severity } => {
                        self.active_buff(id, status, severity, events)
                    }
                    _ => Spent::Nothing,
                }
            }
            (ActiveAim::Direction, _) => {
                self.say("Which way?");
                Spent::Nothing
            }
            (ActiveAim::Cell, _) => {
                self.say("Where?");
                Spent::Nothing
            }
            (ActiveAim::Nothing, Aimed::Cell(_)) => {
                self.say("That is aimed at nothing.");
                Spent::Nothing
            }
        }
    }

    /// The player pays weapon active `id`'s stamina (and its cooldown, if it has one), its
    /// refusals behind it.
    fn pay_active(&mut self, id: WeaponAbilityId, events: &mut Vec<Event>) {
        let player = self.state.player;
        let cost = self.state.player().weapon_stamina(&self.content, id);
        if let Some(stamina) = self.content.resource_id(STAMINA) {
            self.spend_resource(player, stamina, cost, events);
        }
        if let Some(cooldown) = self
            .content
            .weapon_ability(id)
            .active
            .and_then(|a| a.cooldown)
        {
            self.afflict(player, cooldown, 1, None, events);
        }
    }

    /// The time weapon active `id` takes the player when the action it is costs `base`: its
    /// `percent` of it, cut by Practised technique (never below half), at least 1.
    pub(super) fn active_price(&self, id: WeaponAbilityId, base: i64) -> i64 {
        let percent = self
            .content
            .weapon_ability(id)
            .active
            .map_or(100, |a| a.percent);
        let me = self.state.player();
        let cut = if me.talents.is_empty() {
            0
        } else {
            me.talent_mods(&self.content).weapon_energy_percent
        };
        let percent = (i64::from(percent) * i64::from((100 + cut).max(0)) / 100).max(LEAST_PERCENT);
        percent_of(base, u32::try_from(percent).unwrap_or(u32::MAX))
    }

    /// A weapon active's blow at the hostile in `dir`: Vital strike, Sunder, Feint, Mighty swing,
    /// Hip throw (with what is held), Shield rush (a bash), and Lunge (reaching on). Refused (no
    /// time, nothing spent) with nobody hostile there.
    fn active_blow(
        &mut self,
        id: WeaponAbilityId,
        effect: WeaponDoes,
        dir: Dir,
        events: &mut Vec<Event>,
    ) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        if let WeaponDoes::Knockback { bash: true, .. } = effect {
            return self.rush(id, effect, dir, events);
        }
        let (target, seen, beside) = match effect {
            WeaponDoes::ReachBlow { reach } => match self.reach_toward(dir, reach) {
                Some(found) => found,
                None => return Spent::Nothing,
            },
            _ => match self.hostile_toward(dir) {
                Some((target, seen)) => (target, seen, true),
                None => return Spent::Nothing,
            },
        };
        let me = self.state.player();
        let melee = me.attack_skills(&content, AttackKind::Melee, None);
        let with = me.weapon_item(&content);
        let base = self.attack_price(player, ActionKind::Melee, with, melee);
        let mut profile = me.melee(&content);
        match effect {
            WeaponDoes::SureCritical => profile.sure_critical = true,
            WeaponDoes::IgnoreProtection => profile.penetration = ALL_PROTECTION,
            WeaponDoes::DoubleDice { to_hit } => {
                profile.damage = twice(profile.damage);
                profile.to_hit += to_hit;
            }
            WeaponDoes::IgnoreDefence { categories } => self.technique.ignores = categories,
            _ => {}
        }
        self.pay_active(id, events);
        let blow = self.player_blow_with(target, seen, profile, events);
        self.technique = Technique::default();
        // A blow over a pace between brings no free blows of the end of an attack: the foe
        // stands too far to riposte, and nobody beside to cleave on to.
        if beside {
            self.after_attack(player, target, super::free_blow::parried(blow), events);
        }
        if let WeaponDoes::Knockback { status, .. } = effect {
            self.knock_back(target, dir, blow, status, events);
        }
        self.exert(ActionKind::Melee);
        Spent::Turn(self.active_price(id, base))
    }

    /// Shield rush: a bash with the shield held at the hostile in `dir`, driving it back.
    /// Refused (no time) without a shield to bash with or nobody hostile there.
    fn rush(
        &mut self,
        id: WeaponAbilityId,
        effect: WeaponDoes,
        dir: Dir,
        events: &mut Vec<Event>,
    ) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some((shield, profile)) = self.state.player().bash(&content) else {
            self.say("You have no shield to bash with.");
            return Spent::Nothing;
        };
        let Some((target, seen)) = self.hostile_toward(dir) else {
            return Spent::Nothing;
        };
        let me = self.state.player();
        let skills = me.attack_skills(&content, AttackKind::Melee, Some(shield));
        let with = me.shield_item(&content);
        let base = self.attack_price(player, ActionKind::Melee, with, skills);
        self.pay_active(id, events);
        let blow = self.strike_with(target, seen, (Some(shield), profile, skills), events);
        self.after_attack(player, target, super::free_blow::parried(blow), events);
        if let WeaponDoes::Knockback { status, .. } = effect {
            self.knock_back(target, dir, blow, status, events);
        }
        self.exert(ActionKind::Melee);
        Spent::Turn(self.active_price(id, base))
    }

    /// The player strikes `target` in melee with `profile` made `with` the shield, the off
    /// hand's weapon, or not, drawing on `skills`, said as such a blow is.
    fn strike_with(
        &mut self,
        target: ActorId,
        seen: bool,
        (with, profile, skills): (Option<crate::content::ObjectId>, Profile, AttackSkills),
        events: &mut Vec<Event>,
    ) -> Option<Blow> {
        let words = move |me: &Actor, content: &Content, damage| {
            super::fight::Words::of(me, AttackKind::Melee, with, damage, content)
        };
        self.player_strikes(
            target,
            seen,
            (with, profile, skills),
            (&words, true),
            events,
        )
    }

    /// Lunge's target: the first one along `dir` from the player within `reach` cells, the cells
    /// before it empty of actors and passable; with whether it is seen and stands beside them.
    /// `None` (said) when there is nobody there, the way is blocked, or the first one met is no
    /// foe.
    fn reach_toward(&mut self, dir: Dir, reach: u8) -> Option<(ActorId, bool, bool)> {
        let mut cell = self.player_pos();
        for pace in 1..=reach {
            cell = cell.step(dir);
            let level = self.state.current();
            if (self.state.actor_at(&level.id, cell)).is_some() {
                return if pace == 1 {
                    self.hostile_toward(dir).map(|(t, seen)| (t, seen, true))
                } else {
                    self.hostile_at(cell).map(|(t, seen)| (t, seen, false))
                };
            }
            // A lunge is no step: nothing is sprung on the way, but it needs room.
            if !level.is_walkable(&self.content, cell) {
                self.say("There is no room to lunge.");
                return None;
            }
        }
        self.say("There is nobody there to strike.");
        None
    }

    /// The hostile at `cell` and whether the player sees it; `None` (said) when the one there
    /// is no foe.
    fn hostile_at(&mut self, cell: Pos) -> Option<(ActorId, bool)> {
        let level = &self.state.current().id;
        let (target, creature) = (self.state.actor_at(level, cell)).map(|a| (a.id, a.creature))?;
        let seen = self.perception.sees_actors_at(cell);
        if self.stance(self.state.player, target) != crate::content::Stance::Hostile {
            self.say(if seen {
                let name = &self.content.creature(creature).name;
                format!("You have no quarrel with {}.", crate::content::the(name))
            } else {
                "Something is in the way.".into()
            });
            return None;
        }
        Some((target, seen))
    }

    /// A foe the player's knockback `blow` hit is driven a cell on along `dir`, if that cell is
    /// free (walkable, no trap, no ice, nobody there: a teleport's check, no draw), and suffers
    /// `status`. Into a wall or another it stays where it is; the blow landed all the same.
    fn knock_back(
        &mut self,
        target: ActorId,
        dir: Dir,
        blow: Option<Blow>,
        status: Option<StatusId>,
        events: &mut Vec<Event>,
    ) {
        if !blow.is_some_and(|b| b.strike.is_hit()) || self.state.death.is_some() {
            return;
        }
        let Some((from, creature)) = (self.state.actors.get(&target)).map(|a| (a.pos, a.creature))
        else {
            return;
        };
        let to = from.step(dir);
        let level = self.state.current();
        let free = level.is_walkable(&self.content, to)
            && !level.traps.contains_key(&to)
            && level.ice_at(to).is_none()
            && self.state.actor_at(&level.id, to).is_none();
        if !free {
            return;
        }
        if let Some(actor) = self.state.actors.get_mut(&target) {
            actor.pos = to;
        }
        events.push(Event::ActorMoved {
            actor: target,
            from,
            to,
        });
        if self.perception.sees_actors_at(from) {
            let name = crate::content::the_start(&self.content.creature(creature).name);
            self.say(format!("{name} is driven back."));
        }
        if let Some(status) = status {
            self.afflict(target, status, 1, Some(self.state.player), events);
        }
    }

    /// Whirl and Staff sweep: a blow at every hostile beside the player, clockwise from `from`,
    /// each at `to_hit`, with each the free blows the end of an attack brings; with a weapon in
    /// each hand the blades take turns. Refused (no time) with no hostile beside them.
    fn sweep(
        &mut self,
        id: WeaponAbilityId,
        from: Dir,
        to_hit: i32,
        events: &mut Vec<Event>,
    ) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let mut around = self.hostiles_beside();
        if around.is_empty() {
            self.say("There is nobody beside you to strike.");
            return Spent::Nothing;
        }
        // Clockwise from `from`: the list is clockwise from the north.
        let here = self.player_pos();
        let turn = |pos: Pos| {
            let at = Dir::from_delta(pos.x - here.x, pos.y - here.y);
            let i = Dir::ALL.iter().position(|&d| Some(d) == at).unwrap_or(0);
            let start = Dir::ALL.iter().position(|&d| d == from).unwrap_or(0);
            (i + Dir::ALL.len() - start) % Dir::ALL.len()
        };
        around.sort_by_key(|&(t, _)| self.state.actors.get(&t).map_or(0, |a| turn(a.pos)));
        let me = self.state.player();
        let melee = me.attack_skills(&content, AttackKind::Melee, None);
        let with = me.weapon_item(&content);
        let base = self.attack_price(player, ActionKind::Melee, with, melee);
        self.pay_active(id, events);
        for (i, (target, seen)) in around.into_iter().enumerate() {
            if self.state.death.is_some() {
                break;
            }
            if !self.state.actors.contains_key(&target) {
                continue;
            }
            let me = self.state.player();
            let off = (i % 2 == 1).then(|| me.off_hand_blow(&content)).flatten();
            let blow = match off {
                Some((blade, mut profile)) => {
                    profile.to_hit += to_hit;
                    let skills = me.attack_skills(&content, AttackKind::Melee, Some(blade));
                    self.strike_with(target, seen, (Some(blade), profile, skills), events)
                }
                None => {
                    let mut profile = me.melee(&content);
                    profile.to_hit += to_hit;
                    self.player_blow_with(target, seen, profile, events)
                }
            };
            self.after_attack(player, target, super::free_blow::parried(blow), events);
        }
        self.exert(ActionKind::Melee);
        Spent::Turn(self.active_price(id, base))
    }

    /// A weapon active's shot at `at` with the missile weapon and ammunition the player has
    /// ready: Piercing shot (flying on past the first it hits), Gap shot (unsoaked). Refused (no
    /// time) as a shot is: on the overworld, with nothing to shoot, out of range.
    fn active_shot(
        &mut self,
        id: WeaponAbilityId,
        effect: WeaponDoes,
        at: Pos,
        events: &mut Vec<Event>,
    ) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        if !self.can_loose_here() {
            return Spent::Nothing;
        }
        let me = self.state.player();
        let (Some((launcher, _)), Some(mut profile)) = (me.launcher(&content), me.ranged(&content))
        else {
            self.say("You have nothing ready to shoot.");
            return Spent::Nothing;
        };
        let Some(item) = me.ammunition_item(&content) else {
            self.say("You have nothing ready to shoot.");
            return Spent::Nothing;
        };
        let range = me.launcher_range(&content).unwrap_or(1);
        if !self.may_loose(item) || !self.within_reach(at, range, "shoot") {
            return Spent::Nothing;
        }
        if effect == WeaponDoes::IgnoreProtection {
            profile.penetration = ALL_PROTECTION;
        }
        let base = self.shot_price(player, launcher);
        self.pay_active(id, events);
        self.technique.pierce = effect == WeaponDoes::PierceShot;
        let loose = super::shoot::Loose {
            kind: AttackKind::Ranged,
            item,
            profile,
            range,
        };
        self.player_looses(at, loose, events);
        self.technique = Technique::default();
        self.exert(ActionKind::Shoot);
        Spent::Turn(self.active_price(id, base))
    }

    /// Shield wall: `status` (`severity` degrees) on the player, for its percent of a wait.
    fn active_buff(
        &mut self,
        id: WeaponAbilityId,
        status: StatusId,
        severity: u16,
        events: &mut Vec<Event>,
    ) -> Spent {
        let player = self.state.player;
        let base = self.price_of(player, ActionKind::Wait);
        self.pay_active(id, events);
        self.afflict(player, status, severity, Some(player), events);
        Spent::Turn(self.active_price(id, base))
    }

    /// The side of a target the player's attack strikes, without the Defence categories Feint
    /// ignores (each positive bonus there taken away), which this attack takes: the one place
    /// [`Technique::ignores`] is read (`fight.rs::resolve`).
    pub(super) fn feinted(&mut self, attacker: ActorId, target: Target) -> Target {
        if attacker != self.state.player || self.technique.ignores == Categories::default() {
            return target;
        }
        let ignores = std::mem::take(&mut self.technique.ignores);
        target.without(ignores.iter())
    }

    /// Whether the shot `shooter` looses flies on past the first it hits (Piercing shot): the one
    /// place [`Technique::pierce`] is read (`shoot.rs::fly`), which this shot takes.
    pub(super) fn pierces(&mut self, shooter: ActorId) -> bool {
        shooter == self.state.player && std::mem::take(&mut self.technique.pierce)
    }
}

/// `dice` rolled twice: twice the dice, the bonus once (Mighty swing).
fn twice(dice: Dice) -> Dice {
    Dice::of(dice.count() * 2, dice.sides()).plus(dice.bonus())
}

/// `text` with its first letter in upper case.
fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map_or_else(String::new, |c| c.to_uppercase().chain(chars).collect())
}
