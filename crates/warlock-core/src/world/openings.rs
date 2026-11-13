//! The thief's Openings in the world (spec §8 "Resources" and "Class abilities"; the openings
//! design `docs/design/thief-openings.md` §5, §6; `docs/dev/openings.md`): the thief's Sneak
//! attack grants the pool (from level 1), sneak attacks, feints, and the mark's kill build it
//! (`builds` in `data/resources.toml`, through `ambush.rs`, `feint.rs`, and `mark.rs`), it
//! decays with time, and two actives spend it:
//!
//! - **Exploit** (`use:exploit:E`, level 1): every opening held (n) spent on a melee blow at
//!   the hostile that way that is a sneak attack whatever the foe knows
//!   ([`super::weapon_active::Technique::sneak`]), with the ladder the thief's class talents
//!   build ([`crate::talent::ExploitMods`]): +n × its dice after the sneak dice
//!   ([`World::land_exploit`]), +n × its to-hit, and its status on the foe struck (Low blow's
//!   Stunned). Refused, no time, nothing spent: no hostile there, no opening (greyed in the
//!   menu), a mark standing that is not this creature (`mark.rs`), a foe the player sees without
//!   vitals. After it the pool is 0 and the mark cleared, unless the blow killed it (then the
//!   kill rule cleared it and gave its opening).
//! - **Master thief's finisher** (`use:master_thief:E`): five openings for a blow that is a
//!   critical if it hits, with no confirmation roll ([`crate::combat::Profile::sure_critical`]).
//!
//! The grant is the general one for talents that grant a resource (`grants_resource`,
//! `world/focus.rs::hold`). The ladder's dice are drawn from the gameplay stream after the sneak
//! dice, on a hit that landed them only.

use super::{ActorId, Spent, World};
use crate::action::{AttackKind, Event};
use crate::combat::Blow;
use crate::content::{ActionKind, ClassActive, Stance};
use crate::geom::Dir;

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
        let ladder = me.talent_mods(&content).exploit;
        let mut profile = me.melee(&content);
        profile.to_hit += ladder.to_hit * i32::from(n);
        self.say(match n {
            0 | 1 => "You strike at the opening.",
            2..=4 => "You press your openings.",
            _ => "You tear into every opening!",
        });
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
        self.after_attack(player, target, super::free_blow::parried(blow), events);
        self.exert(ActionKind::Melee);
        Spent::Turn(price)
    }

    /// Whether `attacker`'s attack is an exploit's ([`World::exploit`]): its sneak attack builds
    /// no openings (`ambush.rs::ambushed`).
    pub(super) fn exploiting(&self, attacker: ActorId) -> bool {
        attacker == self.state.player && self.technique.exploit > 0
    }

    /// `blow`, `attacker`'s attack, `unseen` (its sneak dice landed on a hit), with the ladder's
    /// dice when it is an exploit ([`World::land_exploit`]). The one place
    /// [`super::weapon_active::Technique::exploit`] is taken.
    pub(super) fn exploited(&mut self, attacker: ActorId, unseen: bool, blow: Blow) -> Blow {
        if attacker != self.state.player {
            return blow;
        }
        let n = std::mem::take(&mut self.technique.exploit);
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
        let ladder = self.state.player().talent_mods(&self.content).exploit;
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

    /// Master thief's finisher: a blow with what the player wields at the hostile in `dir` that
    /// is a critical if it hits (no confirmation roll), for the price of a melee attack and its
    /// exertion. Its openings were spent by `use_class`, which refused (no time) with no
    /// hostile there.
    pub(super) fn finisher(&mut self, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some((target, seen)) = self.hostile_toward(dir) else {
            return Spent::Nothing;
        };
        let me = self.state.player();
        let melee = me.attack_skills(&content, AttackKind::Melee, None);
        let with = me.weapon_item(&content);
        let price = self.attack_price(player, ActionKind::Melee, with, melee);
        let mut profile = me.melee(&content);
        profile.sure_critical = true;
        self.say("You strike at the opening!");
        let vitalless = !self.has_vitals(target);
        let blow = self.player_blow_with(target, seen, profile, events);
        self.no_vital_spot(vitalless, blow);
        self.after_attack(player, target, super::free_blow::parried(blow), events);
        self.exert(ActionKind::Melee);
        Spent::Turn(price)
    }
}
