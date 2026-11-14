//! The thief's Openings in the world (spec §8 "Resources" and "Class abilities"; the openings
//! design `docs/design/thief-openings.md` §5, §6; `docs/dev/openings.md`): the thief's Sneak
//! attack grants the pool (from level 1), sneak attacks, feints, and the mark's kill build it
//! (`builds` in `data/resources.toml`, through `ambush.rs`, `feint.rs`, and `mark.rs`), it
//! decays with time, and one active spends it, **Exploit** (`use:exploit:E`, level 1): every
//! opening held (n) spent on a melee blow at the hostile that way that is a sneak attack
//! whatever the foe knows ([`crate::world::combat::weapon_active::Technique::sneak`]), with the ladder the
//! thief's class talents build ([`crate::talent::ExploitMods`]): +n × its dice after the sneak
//! dice ([`World::land_exploit`]; Exploit's 1d6, Improved sneak attack's second from 32), +n ×
//! its to-hit and its status on the foe struck (Low blow's Stunned, 12), and a threat n × its
//! points lower with a sure critical from five spent (Master thief's, 60:
//! [`World::exploit_profile`], [`crate::combat::Profile::sure_critical`]). Refused, no time,
//! nothing spent: no hostile there, no opening (greyed in the menu), a mark standing that is not
//! this creature (`mark.rs`), a foe the player sees without vitals. After it the pool is 0 and
//! the mark cleared, unless the blow killed it (then the kill rule cleared it and gave its
//! opening).
//!
//! From level 18 **Thrown exploit** (`use:thrown_exploit:c:12,5`, #1465) is the same blow
//! delivered by a throw of one of stack `c` at the creature the player sees on that cell:
//! Exploit's refusals, plus what cannot be thrown so (a flask), a cell beyond the throw's range
//! capped at the sneak dice's reach ([`crate::world::Actor::exploit_throw_range`]), and a wall or
//! creature the player sees in the line. The ladder rides `Technique::at`, so a creature the
//! player did not see in the way takes a plain throw.
//!
//! The grant is the general one for talents that grant a resource (`grants_resource`,
//! `world/classes/focus.rs::hold`). The ladder's dice are drawn from the gameplay stream after the sneak
//! dice, on a hit that landed them only.

use crate::action::{AttackKind, Event};
use crate::combat::{Blow, Profile};
use crate::content::{ActionKind, ClassActive, Stance};
use crate::fov::radius2;
use crate::geom::{Dir, Pos};
use crate::perception::CellView;
use crate::world::combat::shoot::Loose;
use crate::world::{ActorId, Spent, World};

impl World {
    /// Exploit's refusals aimed `dir`, before anything is spent (no time; said): no hostile
    /// there, the openings in another creature's guard ([`World::off_mark`]), a foe the player
    /// sees without vitals (Feint refuses the mindless the same way; an unseen one is struck and
    /// told "It has no vital spot." as any blow).
    pub(super) fn exploit_may(&mut self, dir: Dir) -> bool {
        let player = self.state.player;
        let to = self.player_pos().step(dir);
        let level = &self.state.current().id;
        let foe = (self.state.actor_at(level, to)).map(|a| a.id);
        let Some(foe) = foe.filter(|&id| self.stance(player, id) == Stance::Hostile) else {
            self.say("There is no one there.");
            return false;
        };
        if self.off_mark(ClassActive::Exploit, Some(dir)) {
            return false;
        }
        if self.perception.sees_actors_at(to) && !self.has_vitals(foe) {
            self.say("It has no vital spot.");
            return false;
        }
        true
    }

    /// Exploit: a blow with what the player wields at the hostile in `dir` that is a sneak
    /// attack whatever it knows, spending `n` openings (every one held: `use_class` spent them,
    /// [`crate::ability::class_cost`]), for the price of a melee attack and its exertion.
    pub(super) fn exploit(&mut self, dir: Dir, n: u16, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some((target, seen)) = self.hostile_toward(dir) else {
            return Spent::Nothing;
        };
        let me = self.state.player();
        let melee = me.attack_skills(&content, AttackKind::Melee, None);
        let with = me.weapon_item(&content);
        let price = self.attack_price(player, ActionKind::Melee, with, melee);
        let ladder = me.talent_mods(&content).stealth.exploit;
        let profile = self.exploit_profile(n, me.melee(&content));
        self.say_exploit(n);
        let vitalless = !self.has_vitals(target);
        self.technique.sneak = true;
        self.technique.exploit = n;
        let blow = self.player_blow_with(target, seen, profile, events);
        self.technique.sneak = false;
        self.technique.exploit = 0;
        self.no_vital_spot(vitalless, blow);
        // The pool is spent: the mark goes with it, unless the blow killed it (`mark.rs`).
        self.unmark(target);
        let hit = blow.is_some_and(|b| b.strike.is_hit());
        if let Some(status) = ladder.status.filter(|_| hit)
            && self.state.actors.contains_key(&target)
        {
            self.afflict(target, status, 1, Some(player), events);
        }
        self.after_attack(
            player,
            target,
            crate::world::combat::free_blow::parried(blow),
            events,
        );
        self.exert(ActionKind::Melee);
        Spent::Turn(price)
    }

    /// What the player is told as an exploit of `n` openings is made, a blow or a throw.
    fn say_exploit(&mut self, n: u16) {
        self.say(match n {
            0 | 1 => "You strike at the opening.",
            2..=4 => "You press your openings.",
            _ => "You tear into every opening!",
        });
    }

    /// `use:thrown_exploit:c:12,5` ([`crate::action::Action::UseThrow`]): the player uses class
    /// active `active`, one that throws, with inventory stack `item` at cell `at`.
    pub(in crate::world) fn use_class_throw(
        &mut self,
        active: ClassActive,
        item: usize,
        at: Pos,
        events: &mut Vec<Event>,
    ) -> Spent {
        if !active.aims_throw() {
            self.say(format!("`{}` throws nothing.", active.id()));
            return Spent::Nothing;
        }
        if !self.may_use_class(active) || !self.can_loose_here() {
            return Spent::Nothing;
        }
        let Some(target) = self.thrown_exploit_may(active, item, at) else {
            return Spent::Nothing;
        };
        let content = self.content.clone();
        let player = self.state.player;
        let mut n = 0;
        if let Some(cost) = crate::ability::class_cost(self.state.player(), &content, active) {
            self.spend_resource(player, cost.resource, cost.amount, events);
            n = cost.amount;
        }
        self.thrown_exploit(item, (target, at), n, events)
    }

    /// The thrown exploit's refusals with stack `item` at cell `at`, before anything is spent (no
    /// time; said): Exploit's (no hostile the player sees there, the openings in another
    /// creature's guard, a foe without vitals) and a throw's (what cannot be thrown so, a flask
    /// among them; beyond [`crate::world::Actor::exploit_throw_range`]; a wall or a creature the player
    /// sees on the line before it, as the aiming mode draws it). The creature there, when it may
    /// be made.
    fn thrown_exploit_may(&mut self, active: ClassActive, item: usize, at: Pos) -> Option<ActorId> {
        let content = self.content.clone();
        let player = self.state.player;
        let me = self.state.player();
        let Some(stack) = me.inventory.get(item) else {
            self.say("You carry no such thing.");
            return None;
        };
        let Some(range) = me.exploit_throw_range(item, &content) else {
            let what = self.lore().describe(&stack.single());
            self.say(format!("You cannot exploit an opening with {what}."));
            return None;
        };
        // Unpaid goods are not thrown (`shop.rs`).
        if !self.may_loose(item) {
            return None;
        }
        let level = &self.state.current().id;
        let foe = (self.state.actor_at(level, at).map(|a| a.id))
            .filter(|&id| id != player && self.perception.sees_actors_at(at))
            .filter(|&id| self.stance(player, id) == Stance::Hostile);
        let Some(foe) = foe else {
            self.say("There is no one there.");
            return None;
        };
        if self.off_mark_at(active, at) {
            return None;
        }
        if !self.has_vitals(foe) {
            self.say("It has no vital spot.");
            return None;
        }
        let here = self.player_pos();
        if at.dist2(here) > radius2(range) {
            self.say("That is out of reach.");
            return None;
        }
        let view = self.perception();
        let solid = |p: Pos| match view.cell(p) {
            CellView::Visible(t) | CellView::Remembered(t) => !content.terrain(t).transparent,
            CellView::Unknown => false,
        };
        let in_the_way =
            (here.ray(at).take_while(|&p| p != at)).any(|p| solid(p) || view.actor_at(p).is_some());
        if in_the_way {
            self.say("Something is in the way.");
            return None;
        }
        Some(foe)
    }

    /// The thrown exploit: one of inventory stack `item` thrown at `target` on cell `at`, its
    /// thrown attack a sneak attack whatever the foe knows with the ladder of `n` openings (spent
    /// by [`World::use_class_throw`]) on that creature only ([`World::thrown_exploit_profile`],
    /// [`World::exploited`]): one the player did not see in the way takes a plain throw. A
    /// throw's price and exertion.
    fn thrown_exploit(
        &mut self,
        item: usize,
        (target, at): (ActorId, Pos),
        n: u16,
        events: &mut Vec<Event>,
    ) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let me = self.state.player();
        let (Some(range), Some(profile)) =
            (me.throw_range(item, &content), me.thrown(item, &content))
        else {
            return Spent::Nothing;
        };
        let thrown = me.inventory[item].object;
        let throwing = me.attack_skills(&content, AttackKind::Thrown, Some(thrown));
        let ladder = me.talent_mods(&content).stealth.exploit;
        // Priced as the throw begins, with what is thrown still in the pack.
        let price = self.attack_price(player, ActionKind::Throw, None, throwing);
        self.choose_mark(target);
        self.say_exploit(n);
        self.technique.sneak = true;
        self.technique.exploit = n;
        self.technique.at = Some(target);
        self.technique.struck = false;
        let loose = Loose {
            kind: AttackKind::Thrown,
            item,
            profile,
            range,
        };
        self.player_looses(at, loose, events);
        let struck = self.technique.struck;
        self.technique.sneak = false;
        self.technique.exploit = 0;
        self.technique.at = None;
        self.technique.struck = false;
        // The pool is spent: the mark goes with it, unless the throw killed it (`mark.rs`).
        self.unmark(target);
        if let Some(status) = ladder.status.filter(|_| struck)
            && self.state.actors.contains_key(&target)
        {
            self.afflict(target, status, 1, Some(player), events);
        }
        self.exert(ActionKind::Throw);
        Spent::Turn(price)
    }

    /// `profile`, `attacker`'s attack on `target`, with the ladder's before the roll when it is a
    /// thrown exploit's at the creature it is aimed at ([`World::exploit_profile`]); any other
    /// attack as it is. Exploit's blow has its ladder already ([`World::exploit`]).
    pub(in crate::world) fn thrown_exploit_profile(
        &self,
        attacker: ActorId,
        target: ActorId,
        profile: Profile,
    ) -> Profile {
        let technique = &self.technique;
        if attacker == self.state.player && technique.exploit > 0 && technique.at == Some(target) {
            self.exploit_profile(technique.exploit, profile)
        } else {
            profile
        }
    }

    /// Whether `attacker`'s attack is an exploit's ([`World::exploit`]): its sneak attack builds
    /// no openings (`ambush.rs::ambushed`).
    pub(in crate::world) fn exploiting(&self, attacker: ActorId) -> bool {
        attacker == self.state.player && self.technique.exploit > 0
    }

    /// `blow`, `attacker`'s attack, `unseen` (its sneak dice landed on a hit), with the ladder's
    /// dice when it is an exploit ([`World::land_exploit`]). The one place
    /// [`crate::world::combat::weapon_active::Technique::exploit`] is taken.
    pub(in crate::world) fn exploited(
        &mut self,
        (attacker, target): (ActorId, ActorId),
        unseen: bool,
        blow: Blow,
    ) -> Blow {
        // A thrown exploit is taken by the creature it is aimed at only.
        if attacker != self.state.player || self.technique.at.is_some_and(|at| at != target) {
            return blow;
        }
        let n = std::mem::take(&mut self.technique.exploit);
        if n > 0 && self.technique.at.is_some() {
            self.technique.struck = blow.strike.is_hit();
        }
        if unseen {
            self.land_exploit(n, blow)
        } else {
            blow
        }
    }

    /// The ladder's dice on an exploit of `n` openings whose sneak dice landed (`blow` hit):
    /// n × the player's exploit dice, drawn from the gameplay stream after the sneak dice and
    /// added after Protection, never multiplied by a critical, as sneak dice are
    /// (`ambush.rs::ambushed`). The one place the ladder's damage is rolled.
    pub(super) fn land_exploit(&mut self, n: u16, blow: Blow) -> Blow {
        if n == 0 || !blow.strike.is_hit() {
            return blow;
        }
        let ladder = self
            .state
            .player()
            .talent_mods(&self.content)
            .stealth
            .exploit;
        let Some(dice) = ladder.dice else {
            return blow;
        };
        let mut extra = 0;
        for _ in 0..n {
            extra += dice.roll(&mut self.state.rng).max(0);
        }
        Blow {
            damage: blow.damage + extra,
            ..blow
        }
    }

    /// `profile`, the player's attack in an exploit of `n` openings, with what their ladder does
    /// before the roll ([`crate::talent::ExploitMods::profile`]: Low blow's to-hit, Master
    /// thief's threat and sure critical). The one place the ladder touches the attack itself;
    /// the dice after the hit are [`World::land_exploit`]'s.
    pub(super) fn exploit_profile(&self, n: u16, profile: Profile) -> Profile {
        let ladder = self
            .state
            .player()
            .talent_mods(&self.content)
            .stealth
            .exploit;
        ladder.profile(n, profile)
    }
}
