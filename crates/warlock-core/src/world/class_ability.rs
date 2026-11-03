//! The class actives in the world (spec §8 "Class abilities"): what a class's level table gives
//! that is used from the abilities menu ([`crate::ability::Ability::Class`]) as [`Action::Use`]
//! (`use:rage`, `use:smite:E`): Rage, War cry (for rage, `world/war_cry.rs`), Veteran's wind, Whirlwind, Lay on hands, Smite (for zeal, `world/smite.rs`), Holy
//! radiance, Stunning strike, Quivering palm and Step of the wind (for ki, `world/ascetic.rs`; the last takes no time), the songs (Inspire, for inspiration, `world/song.rs`), the thief's
//! Lurk and Low blow (`world/thief.rs`), Master thief's finisher (for openings,
//! `world/openings.rs`; the ranger's Volley, aimed at a cell, is `world/volley.rs`). The
//! abilities themselves are talents of the class tree, given as levels come (`world/class.rs`)
//! and read through `Actor::talent_mods`. Each active refuses, taking no time, without the
//! ability, while its cooldown (or the rage and the exhaustion after it) lasts, and (an aimed
//! blow) with nobody hostile there, before anything is spent. Nothing here draws at random but
//! the blows themselves.
//!
//! Alignment bars none of them (spec §8 "Alignment"). A holy gift (Lay on hands, Smite, Holy
//! radiance, Consecrate: [`ClassActive::holy`]) whose talent names the bands it works for
//! (`alignment = { min = "kind" }` in data) asks [`World::answered`] after its free refusals:
//! outside those bands it is used and does nothing ([`World::unanswered`]: its line, the time of
//! a cast, no cooldown, mana, or resource spent: `use_class` asks before it spends a cost). Nothing remembers it; the next use asks again.
//!
//! [`Action::Use`]: crate::action::Action::Use

use super::smite::SmiteMode;
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
        // An aimed blow with nobody hostile there is refused before anything is spent, and so is
        // a blow of the bare hand (Stunning strike, Quivering palm) with a weapon in hand, a low
        // blow with nobody there, and a lurk too late (`thief.rs`).
        if !self.thief_may(active, dir) {
            return Spent::Nothing;
        }
        if active == ClassActive::StepOfTheWind && !self.may_wind_step() {
            return Spent::Nothing;
        }
        if dir.is_some_and(|d| active.strikes() && self.hostile_toward(d).is_none()) {
            return Spent::Nothing;
        }
        if active.bare_handed() && self.state.player().weapon_item(&self.content).is_some() {
            self.say("Your hands are not free.");
            return Spent::Nothing;
        }
        // Its cost in an alternative resource (spec §8 "Resources"), if it has one: what
        // `unavailable` checked is there to spend. A holy gift paid so is asked the bands first:
        // unanswered, it spends nothing.
        let player = self.state.player;
        let content = self.content.clone();
        if let Some(cost) = crate::ability::class_cost(self.state.player(), &content, active) {
            if active.holy() && !self.answered(active) {
                return self.unanswered(active);
            }
            self.spend_resource(player, cost.resource, cost.amount, events);
        }
        match (active, dir) {
            (ClassActive::Rage, _) => self.rage(events),
            (ClassActive::WarCry, _) => self.war_cry(events),
            (ClassActive::VeteransWind, _) => self.veterans_wind(events),
            (ClassActive::Whirlwind, _) => self.whirlwind(events),
            (ClassActive::LayOnHands, _) => self.lay_on_hands(events),
            (ClassActive::HolyRadiance, _) => self.holy_radiance(events),
            (ClassActive::Inspire | ClassActive::Lullaby, _) => self.sing(active, events),
            (ClassActive::PatientDefence, _) => self.patient_defence(events),
            (ClassActive::Lurk, _) => self.lurk(events),
            (ClassActive::StepOfTheWind, _) => self.wind_step(events),
            (ClassActive::Stillness, _) => self.stillness(events),
            (ClassActive::Consecrate, _) => self.begin_consecrate(),
            (ClassActive::Orison, _) => self.orison(events),
            (ClassActive::Sanctuary, _) => self.begin_sanctuary(),
            (ClassActive::Cackle, _) => self.cackle(events),
            (ClassActive::Smite, Some(dir)) => self.smite(SmiteMode::Plain, dir, events),
            (ClassActive::BrandingSmite, Some(dir)) => self.smite(SmiteMode::Branding, dir, events),
            (ClassActive::WrathfulSmite, Some(dir)) => self.smite(SmiteMode::Wrathful, dir, events),
            (ClassActive::StunningStrike, Some(dir)) => self.stunning_strike(dir, events),
            (ClassActive::QuiveringPalm, Some(dir)) => self.quivering_palm(dir, events),
            (ClassActive::LowBlow, Some(dir)) => self.low_blow(dir, events),
            (ClassActive::MasterThief, Some(dir)) => self.finisher(dir, events),
            (ClassActive::Knock, Some(dir)) => self.knock(dir),
            (
                ClassActive::Smite
                | ClassActive::BrandingSmite
                | ClassActive::WrathfulSmite
                | ClassActive::StunningStrike
                | ClassActive::QuiveringPalm
                | ClassActive::LowBlow
                | ClassActive::MasterThief
                | ClassActive::Knock,
                None,
            ) => Spent::Nothing,
            (ClassActive::Volley | ClassActive::Sic, _) => {
                self.say("Where?");
                Spent::Nothing
            }
            (ClassActive::MerchantsEye, _) => {
                self.say("Use it on what?");
                Spent::Nothing
            }
            (ClassActive::SpiritWeapon, _) => {
                self.say("Call it in what shape?");
                Spent::Nothing
            }
            (ClassActive::Scribe, _) => {
                self.say("Write which spell? Name one you know: `scribe:fireball`.");
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
                Unavailable::While(status) => {
                    format!(
                        "You are still {}.",
                        content.status(status).name.to_lowercase()
                    )
                }
                Unavailable::NoMana => "You have too little mana.".to_string(),
                // A pool with words of its own says them (Rage's "You have not the rage for
                // it.").
                Unavailable::Short(resource) => match &content.resource(resource).lacking {
                    Some(words) => format!("{}.", super::talk::capitalized(words)),
                    None => format!(
                        "You have too little {}.",
                        content.resource(resource).name.to_lowercase()
                    ),
                },
                Unavailable::Still => "Your mind is already still.".to_string(),
                other => format!("You cannot: {}.", other.reason(&content)),
            });
            return false;
        }
        true
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

    /// Lay on hands: hit points back by the level; with Avatar every hit point, every status
    /// that harms as it ticks and every disease cured. Its zeal (Avatar's share of it) was spent
    /// by `use_class`. Takes the time of a `cast`.
    fn lay_on_hands(&mut self, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let mods = self.state.player().talent_mods(&content);
        let Some(hands) = mods.lay_on_hands else {
            return Spent::Nothing;
        };
        if !self.answered(ClassActive::LayOnHands) {
            return self.unanswered(ClassActive::LayOnHands);
        }
        if mods.avatar.is_some() {
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
        } else {
            let back = i32::from(hands.per_level) * i32::from(self.player_level());
            self.heal_player(back);
        }
        self.say("Warmth flows from your hands.");
        Spent::Turn(self.price_of(player, ActionKind::Cast))
    }

    /// Holy radiance: the light on (its status begins) or off again (it ends), for the time of
    /// a `cast`. Putting it out always works.
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
            None if !self.answered(ClassActive::HolyRadiance) => {
                return self.unanswered(ClassActive::HolyRadiance);
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

    /// Quivering palm: a blow of the player's bare hand at the hostile in `dir`; a hit that
    /// does damage leaves it bleeding (the palm's degrees; `ascetic.rs::palm_blow`). Its ki was
    /// spent by `use_class`, which refused (no time) with a weapon in hand or no hostile there.
    fn quivering_palm(&mut self, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let Some(palm) = self
            .state
            .player()
            .talent_mods(&self.content)
            .quivering_palm
        else {
            return Spent::Nothing;
        };
        self.palm_blow(dir, palm, None, events)
    }

    /// The hostiles beside the player, clockwise from the north, each with whether the player
    /// sees it.
    pub(super) fn hostiles_beside(&self) -> Vec<(ActorId, bool)> {
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
    pub(super) fn hostile_toward(&mut self, dir: Dir) -> Option<(ActorId, bool)> {
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
                format!("You have no quarrel with {}.", crate::content::the(name))
            } else {
                "Something is in the way.".into()
            });
            return None;
        }
        Some((target, seen))
    }

    /// Whether holy gift `active` works for the player now: their alignment band is one its
    /// talent's `alignment` allows, or it names none, or the content has no alignment rules.
    pub(super) fn answered(&self, active: ClassActive) -> bool {
        let content = &self.content;
        let bands = content.class_active(active).and_then(|def| def.alignment);
        match (bands, content.alignment()) {
            (Some(bands), Some(_)) => bands.allows(self.state.player().band(content)),
            _ => true,
        }
    }

    /// Holy gift `active` used by a player outside its bands: it does nothing. Says so and takes
    /// the time of a cast, spending nothing else.
    pub(super) fn unanswered(&mut self, active: ClassActive) -> Spent {
        self.say(match active {
            ClassActive::LayOnHands => "You lay on your hands, but no warmth flows from them.",
            ClassActive::Smite | ClassActive::BrandingSmite | ClassActive::WrathfulSmite => {
                "You call for holy wrath, but none answers."
            }
            ClassActive::HolyRadiance => "You will a holy light, but none comes.",
            _ => "You kneel and pray, but no holy power answers you.",
        });
        Spent::Turn(self.price_of(self.state.player, ActionKind::Cast))
    }

    /// The player gets `back` hit points, never past their maximum (nor fewer than they have).
    pub(super) fn heal_player(&mut self, back: i32) {
        let max = self.state.player().max_hp(&self.content);
        let me = self.player_mut();
        me.hp = (me.hp + back).min(max).max(me.hp);
    }
}
