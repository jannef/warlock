//! The class actives in the world (spec §8 "Class abilities"): what a class's level table gives
//! that is used from the abilities menu ([`crate::ability::Ability::Class`]) as [`Action::Use`]
//! (`use:rage`, `use:smite:E`): Rage, Veteran's wind, Whirlwind, Lay on hands, Smite, Holy
//! radiance, Quivering palm, Inspire (for mana, #244; the ranger's Volley, aimed at a cell, is
//! `world/volley.rs`). The abilities themselves are talents of the class tree, given as
//! levels come (`world/class.rs`) and read through `Actor::talent_mods`. Each active refuses,
//! taking no time, without the ability, while its cooldown (or the rage and the exhaustion after
//! it) lasts, and while the player has strayed from the alignment it asks. Nothing here draws at
//! random but the blows themselves.
//!
//! [`Action::Use`]: crate::action::Action::Use

use super::{ActorId, Spent, World};
use crate::ability::{Ability, Unavailable};
use crate::action::{AttackKind, Event};
use crate::content::{ActionKind, ClassActive, Stance};
use crate::geom::Dir;
use crate::status::Effect;

impl World {
    /// `use:ID[:DIR]` ([`crate::action::Action::Use`]): the player uses class active `active`,
    /// aimed `dir` if it asks for a direction. Refused, taking no time, without it, while it is
    /// unavailable, and without a direction it asks for.
    pub(super) fn use_class(
        &mut self,
        active: ClassActive,
        dir: Option<Dir>,
        events: &mut Vec<Event>,
    ) -> Spent {
        if !self.may_use_class(active) {
            return Spent::Nothing;
        }
        let dir = match (active.aimed(), dir) {
            (true, None) => {
                self.say("Which way?");
                return Spent::Nothing;
            }
            (_, dir) => dir,
        };
        // Its cost in an alternative resource (spec §8 "Resources"), if it has one: what
        // `unavailable` checked is there to spend.
        let player = self.state.player;
        if let Some(cost) = self.content.class_active(active).and_then(|a| a.resource) {
            self.spend_resource(player, cost.resource, cost.amount, events);
        }
        match (active, dir) {
            (ClassActive::Rage, _) => self.rage(events),
            (ClassActive::VeteransWind, _) => self.veterans_wind(events),
            (ClassActive::Whirlwind, _) => self.whirlwind(events),
            (ClassActive::LayOnHands, _) => self.lay_on_hands(events),
            (ClassActive::HolyRadiance, _) => self.holy_radiance(events),
            (ClassActive::Inspire, _) => self.inspire(events),
            (ClassActive::Smite, Some(dir)) => self.smite(dir, events),
            (ClassActive::QuiveringPalm, Some(dir)) => self.quivering_palm(dir, events),
            (ClassActive::Smite | ClassActive::QuiveringPalm, None) => Spent::Nothing,
            (ClassActive::Volley, _) => {
                self.say("Where?");
                Spent::Nothing
            }
        }
    }

    /// Whether the player may use class active `active` now: they have it, and it is available
    /// (said when not).
    pub(super) fn may_use_class(&mut self, active: ClassActive) -> bool {
        let content = self.content.clone();
        let ability = Ability::Class(active);
        let me = self.state.player();
        if !ability.had_by(me, &content) {
            self.say("You have no such ability.");
            return false;
        }
        if let Some(why) = ability.unavailable(me, &content) {
            self.say(match why {
                Unavailable::Strayed => "You have strayed: your gift does not answer.".to_string(),
                Unavailable::While(status) => {
                    format!(
                        "You are still {}.",
                        content.status(status).name.to_lowercase()
                    )
                }
                Unavailable::NoMana => "You have too little mana.".to_string(),
                Unavailable::Short(resource) => format!(
                    "You have too little {}.",
                    content.resource(resource).name.to_lowercase()
                ),
                other => format!("You cannot: {}.", other.reason(&content)),
            });
            return false;
        }
        true
    }

    /// Inspire: the player sings themselves (Inspired), spending its mana (Virtuoso: a greater
    /// inspiration, for none), for the time of a `chat`.
    fn inspire(&mut self, events: &mut Vec<Event>) -> Spent {
        let player = self.state.player;
        let Some(inspire) = self.state.player().talent_mods(&self.content).inspiring() else {
            return Spent::Nothing;
        };
        if !self.spend_mana(player, inspire.mana) {
            self.say("You have too little mana.");
            return Spent::Nothing;
        }
        self.afflict(player, inspire.status, 1, Some(player), events);
        Spent::Turn(self.price_of(player, ActionKind::Chat))
    }

    /// Rage: the player suffers the rage's status (Raging), for the time of a wait.
    fn rage(&mut self, events: &mut Vec<Event>) -> Spent {
        let player = self.state.player;
        let Some(status) = self.state.player().talent_mods(&self.content).rage else {
            return Spent::Nothing;
        };
        self.afflict(player, status, 1, Some(player), events);
        Spent::Turn(self.price_of(player, ActionKind::Wait))
    }

    /// Veteran's wind: hit points back at once (one a level, at least its floor), then its
    /// cooldown, for the time of a wait.
    fn veterans_wind(&mut self, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some(wind) = self.state.player().talent_mods(&content).veterans_wind else {
            return Spent::Nothing;
        };
        let back = wind.hp_at(self.player_level());
        self.heal_player(back);
        self.say("You draw on an old soldier's wind.");
        self.afflict(player, wind.cooldown, 1, None, events);
        Spent::Turn(self.price_of(player, ActionKind::Wait))
    }

    /// Lay on hands: hit points back by the level, then its cooldown; with Avatar every hit
    /// point, every status that harms as it ticks and every disease cured, and half the
    /// cooldown. Takes the time of a `cast`.
    fn lay_on_hands(&mut self, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let mods = self.state.player().talent_mods(&content);
        let Some(hands) = mods.lay_on_hands else {
            return Spent::Nothing;
        };
        let mut degrees = hands.degrees;
        if mods.avatar {
            let me = self.state.player();
            let max = me.max_hp(&content);
            let harms: Vec<Effect> = (me.effects.iter())
                .filter(|e| content.status(e.status).tick.is_some())
                .copied()
                .collect();
            let diseases: Vec<_> = me.diseases.iter().map(|i| i.disease).collect();
            self.heal_player(max);
            for e in &harms {
                self.player_mut().effects.remove(e.status);
            }
            self.ended(player, &harms, events);
            for disease in diseases {
                self.end_disease(player, disease, events);
            }
            degrees = (degrees / 2).max(1);
        } else {
            let back = i32::from(hands.per_level) * i32::from(self.player_level());
            self.heal_player(back);
        }
        self.say("Warmth flows from your hands.");
        self.afflict(player, hands.cooldown, degrees, None, events);
        Spent::Turn(self.price_of(player, ActionKind::Cast))
    }

    /// Holy radiance: the light on (its status begins) or off again (it ends), for the time of
    /// a `cast`.
    fn holy_radiance(&mut self, events: &mut Vec<Event>) -> Spent {
        let player = self.state.player;
        let Some((status, _)) = self.state.player().talent_mods(&self.content).radiance else {
            return Spent::Nothing;
        };
        let lit = self.state.player().effects.get(status).copied();
        match lit {
            Some(effect) => {
                self.player_mut().effects.remove(status);
                self.ended(player, &[effect], events);
            }
            None => self.afflict(player, status, 1, Some(player), events),
        }
        Spent::Turn(self.price_of(player, ActionKind::Cast))
    }

    /// Whirlwind: one blow at every hostile beside the player, clockwise from the north, each
    /// with the free blows the end of an attack brings, for the price of one attack and its one
    /// exertion. Refused (no time) with no hostile beside them.
    fn whirlwind(&mut self, events: &mut Vec<Event>) -> Spent {
        let player = self.state.player;
        let around = self.hostiles_beside();
        if around.is_empty() {
            self.say("There is nobody beside you to strike.");
            return Spent::Nothing;
        }
        let me = self.state.player();
        let melee = me.attack_skills(&self.content, AttackKind::Melee, None);
        let with = me.weapon_item(&self.content);
        let price = self.attack_price(player, ActionKind::Melee, with, melee);
        self.say("You whirl about you!");
        for (target, seen) in around {
            if self.state.death.is_some() {
                break;
            }
            if !self.state.actors.contains_key(&target) {
                continue;
            }
            let first = self.player_blow(target, seen, None, events);
            self.after_attack(player, target, super::free_blow::parried(first), events);
        }
        self.exert(ActionKind::Melee);
        Spent::Turn(price)
    }

    /// Smite: a blow with what the player wields at the hostile in `dir`, with the smite's dice
    /// more damage, then its cooldown. Refused (no time) with no hostile there.
    fn smite(&mut self, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some(smite) = self.state.player().talent_mods(&content).smite else {
            return Spent::Nothing;
        };
        let Some((target, seen)) = self.hostile_toward(dir) else {
            return Spent::Nothing;
        };
        let me = self.state.player();
        let melee = me.attack_skills(&content, AttackKind::Melee, None);
        let with = me.weapon_item(&content);
        let price = self.attack_price(player, ActionKind::Melee, with, melee);
        let mut profile = me.melee(&content);
        let dice = smite.dice_at(self.player_level());
        // A magic weapon's extra die is folded into the bonus at its average.
        if let Some(more) = profile.extra {
            profile.bonus += (more.min() + more.max()) / 2;
        }
        profile.extra = Some(dice);
        self.say("You smite!");
        let blow = self.player_blow_with(target, seen, profile, events);
        self.after_attack(player, target, super::free_blow::parried(blow), events);
        self.exert(ActionKind::Melee);
        self.afflict(player, smite.cooldown, smite.degrees, None, events);
        Spent::Turn(price)
    }

    /// Quivering palm: a blow of the player's bare hand at the hostile in `dir`; a hit that
    /// does damage leaves it bleeding (the palm's degrees); then its cooldown. Refused (no time)
    /// with a weapon in hand or no hostile there.
    fn quivering_palm(&mut self, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some(palm) = self.state.player().talent_mods(&content).quivering_palm else {
            return Spent::Nothing;
        };
        if self.state.player().weapon_item(&content).is_some() {
            self.say("Your hands are not free.");
            return Spent::Nothing;
        }
        let Some((target, seen)) = self.hostile_toward(dir) else {
            return Spent::Nothing;
        };
        let me = self.state.player();
        let melee = me.attack_skills(&content, AttackKind::Melee, None);
        let price = self.attack_price(player, ActionKind::Melee, None, melee);
        let profile = me.melee(&content);
        let blow = self.player_blow_with(target, seen, profile, events);
        let wounded = blow.is_some_and(|b| b.damage > 0);
        if wounded && self.state.actors.contains_key(&target) {
            self.afflict(target, palm.status, palm.severity, Some(player), events);
        }
        self.after_attack(player, target, super::free_blow::parried(blow), events);
        self.exert(ActionKind::Melee);
        self.afflict(player, palm.cooldown, palm.degrees, None, events);
        Spent::Turn(price)
    }

    /// The hostiles beside the player, clockwise from the north, each with whether the player
    /// sees it.
    fn hostiles_beside(&self) -> Vec<(ActorId, bool)> {
        let player = self.state.player;
        let here = self.player_pos();
        let level = &self.state.current().id;
        (Dir::ALL.iter())
            .filter_map(|&d| self.state.actor_at(level, here.step(d)))
            .filter(|a| self.stance(player, a.id) == Stance::Hostile)
            .map(|a| (a.id, self.perception.sees_actors_at(a.pos)))
            .collect()
    }

    /// The hostile in `dir` of the player and whether they see it; `None` (said) when nobody,
    /// or nobody hostile, stands there.
    fn hostile_toward(&mut self, dir: Dir) -> Option<(ActorId, bool)> {
        let to = self.player_pos().step(dir);
        let level = &self.state.current().id;
        let Some((target, creature)) = (self.state.actor_at(level, to)).map(|a| (a.id, a.creature))
        else {
            self.say("There is nobody there to strike.");
            return None;
        };
        let seen = self.perception.sees_actors_at(to);
        if self.stance(self.state.player, target) != Stance::Hostile {
            self.say(if seen {
                let name = &self.content.creature(creature).name;
                format!("You have no quarrel with the {name}.")
            } else {
                "Something is in the way.".into()
            });
            return None;
        }
        Some((target, seen))
    }

    /// The player gets `back` hit points, never past their maximum (nor fewer than they have).
    pub(super) fn heal_player(&mut self, back: i32) {
        let max = self.state.player().max_hp(&self.content);
        let me = self.player_mut();
        me.hp = (me.hp + back).min(max).max(me.hp);
    }

    /// The player killed something: with Blood frenzy, a rage they are in lasts longer (its
    /// easing set back, never past its full length).
    pub(super) fn frenzy(&mut self) {
        let mods = self.state.player().talent_mods(&self.content);
        let (Some(status), seconds) = (mods.rage, mods.blood_frenzy) else {
            return;
        };
        if seconds == 0 {
            return;
        }
        let me = self.player_mut();
        if let Some(rage) = me.effects.iter_mut().find(|e| e.status == status) {
            rage.wear = rage.wear.saturating_sub(seconds);
        }
    }

    /// Whether actor `id`, hit for `damage`, holds on with Unstoppable: a blow that would take
    /// its last hit point while it rages leaves it that one, and the rage ends (what follows it
    /// begins). The caller then does no more harm.
    pub(super) fn unstoppable(
        &mut self,
        id: ActorId,
        damage: i32,
        events: &mut Vec<Event>,
    ) -> bool {
        let Some(actor) = self.state.actors.get(&id) else {
            return false;
        };
        if actor.hp - damage > 0 || actor.hp <= 0 {
            return false;
        }
        let mods = actor.talent_mods(&self.content);
        let (Some(status), Some(_)) = (mods.rage, mods.unstoppable) else {
            return false;
        };
        let Some(rage) = actor.effects.get(status).copied() else {
            return false;
        };
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return false;
        };
        actor.hp = 1;
        actor.effects.remove(status);
        if id == self.state.player {
            self.say("You refuse to fall!");
        }
        self.ended(id, &[rage], events);
        true
    }
}
