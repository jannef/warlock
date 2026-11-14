//! Smiting (spec §8 "Class abilities": "Smite"; docs/design/paladin-smites.md §5): the paladin's
//! blow of holy wrath, paid in zeal.
//!
//! A smite is a melee blow with what the player wields at the hostile in a direction, priced as
//! the attack, with the smite's dice more (`Smite::dice_at`: a die per ten levels, at least one,
//! and its `unholy` dice more against a creature type `Content::unholy` names), and the smiter's
//! modifier in the talent's `aim` stat (Presence) added to hit. `use_class` has already refused
//! it with nobody hostile there, asked the bands (`World::answered`), and spent its zeal. Every
//! mode of smiting resolves here ([`SmiteMode`]): the blow, then the mode's rider on a hit that
//! did damage (§6): Branding smite's foe is Branded (it sheds light a while); Wrathful smite's
//! makes a check of its Willpower against the mode's difficulty plus the smiter's `aim`
//! modifier (two gameplay draws) and, failing, is Frightened for the mode's `turns` degrees (one
//! draw), caused by the smiter; Banishing smite's, if it was summoned (`Actor::summoner`), is
//! unmade whatever its hit points, a kill of the player's as any other, and one that is unholy
//! and not summoned took the smite's dice twice (no draw beyond the blow's). A rider's zeal is
//! spent whatever it finds to do. Nothing else here draws at random but the blow itself.
//!
//! Holy wrath ([`World::holy_wrathed`]) is no mode but a passive on every blow in `resolve`: one
//! draw on a hit by its holder on an unholy target (with Avatar, any target).

use crate::action::{AttackKind, Event};
use crate::combat::{Blow, Profile};
use crate::content::{ActionKind, ClassActive};
use crate::damage::Damage;
use crate::geom::Dir;
use crate::talent::FaithEffect;
use crate::talent::SmiteRider;
use crate::world::{ActorId, Spent, World};

/// The modes of smiting: the plain smite, and the smites with a rider.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SmiteMode {
    /// The Smite ability itself: the blow and nothing more.
    Plain,
    /// Branding smite: the foe hit is Branded.
    Branding,
    /// Wrathful smite: the foe hit is Frightened unless its will holds.
    Wrathful,
    /// Banishing smite: the summoned foe hit is unmade, the unholy takes the dice twice.
    Banishing,
}

impl SmiteMode {
    /// The class active of a mode with a rider; `None` for the plain smite.
    const fn active(self) -> Option<ClassActive> {
        match self {
            SmiteMode::Plain => None,
            SmiteMode::Branding => Some(ClassActive::BrandingSmite),
            SmiteMode::Wrathful => Some(ClassActive::WrathfulSmite),
            SmiteMode::Banishing => Some(ClassActive::BanishingSmite),
        }
    }
}

impl World {
    /// A smite of `mode` at the hostile in `dir`: the blow with the smite's dice and aim, then
    /// the mode's rider. Refused (no time) without the Smite talent or with nobody hostile
    /// there.
    pub(super) fn smite(&mut self, mode: SmiteMode, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some((target, seen)) = self.hostile_toward(dir) else {
            return Spent::Nothing;
        };
        let Some(profile) = self.mode_profile(mode, target) else {
            return Spent::Nothing;
        };
        let me = self.state.player();
        let rider = match mode.active() {
            Some(active) => match me.talent_mods(&content).faith.smite_mode(active) {
                Some(rider) => Some(rider),
                None => return Spent::Nothing,
            },
            None => None,
        };
        let melee = me.attack_skills(&content, AttackKind::Melee, None);
        let with = me.weapon_item(&content);
        let price = self.attack_price(player, ActionKind::Melee, with, melee);
        self.say("You smite!");
        let blow = self.player_blow_with(target, seen, profile, events);
        if let Some(rider) = rider
            && let Some(blow) = blow.filter(|b| b.damage > 0)
            && self.state.actors.contains_key(&target)
        {
            self.smite_rider(target, rider, blow, events);
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

    /// Mode of smiting `rider` on `target`, wounded by the smite's `blow` and alive (module
    /// docs). The player is told how it took it if they see it.
    pub(super) fn smite_rider(
        &mut self,
        target: ActorId,
        rider: SmiteRider,
        blow: Blow,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let player = self.state.player;
        let Some(them) = self.state.actors.get(&target) else {
            return;
        };
        let seen = self.perception.sees_actors_at(them.pos);
        let name = self.called(them);
        match rider {
            SmiteRider::Brand { status, severity } => {
                self.afflict(target, status, severity, Some(player), events);
                let took = (self.state.actors.get(&target)).is_some_and(|a| a.effects.has(status));
                if seen && took {
                    self.say(format!("Holy light clings to {name}."));
                }
            }
            SmiteRider::Fear {
                status,
                turns,
                check,
                difficulty,
            } => {
                let me = self.state.player();
                let aim = (me.talent_mods(&content).faith.smite)
                    .and_then(|s| s.aim)
                    .map_or(0, |stat| me.effective_stats(&content).modifier(stat));
                let nerve = them.check_modifier(&content, check);
                let who = crate::text::capitalized(&name);
                if crate::check::check(&mut self.state.rng, nerve, difficulty + aim) {
                    if seen {
                        self.say(format!("{who}'s will holds."));
                    }
                    return;
                }
                let degrees = u16::try_from(turns.roll(&mut self.state.rng).max(1)).unwrap_or(1);
                self.afflict(target, status, degrees, Some(player), events);
                if seen {
                    self.say(format!("{who} quails!"));
                }
            }
            SmiteRider::Banish => {
                let who = crate::text::capitalized(&name);
                if them.summoner.is_some() {
                    if seen {
                        self.say(format!("{who} is unmade!"));
                    }
                    self.unmake(target, blow, seen, events);
                } else if seen && content.unholy(them.creature) {
                    self.say(format!("{who} reels under holy wrath!"));
                }
            }
        }
    }

    /// Summoned `target` dies of Banishing smite's `blow`, whatever its hit points: the player's
    /// kill as if the blow had done it (`World::resolve`'s arm of a kill), the events shown only
    /// if `seen`.
    fn unmake(&mut self, target: ActorId, blow: Blow, seen: bool, events: &mut Vec<Event>) {
        let player = self.state.player;
        let Some(at) = self.state.actors.get(&target).map(|a| a.pos) else {
            return;
        };
        let kind = AttackKind::Melee;
        let skills = (self.state.player()).attack_skills(&self.content, kind, None);
        let mut made = Vec::new();
        self.dies(target, Some(player), None, self.state.time, &mut made);
        self.train_kill((player, target), kind);
        self.siphon(player, (target, at));
        let fired = crate::world::combat::weapon_ability::Fired {
            other: target,
            kind,
            skills,
            blow: Some(blow),
        };
        self.weapon_trigger(player, crate::content::Trigger::Kill, fired, &mut made);
        self.say_news();
        if seen {
            events.extend(made);
        }
    }

    /// The smite of `mode` at `target`: [`World::smite_profile`], and Banishing smite's dice
    /// twice over on an unholy foe not summoned (the bonus once).
    pub(super) fn mode_profile(&self, mode: SmiteMode, target: ActorId) -> Option<Profile> {
        let mut profile = self.smite_profile(target)?;
        let them = self.state.actors.get(&target);
        if mode == SmiteMode::Banishing
            && them.is_some_and(|a| a.summoner.is_none() && self.content.unholy(a.creature))
        {
            profile.extra = profile.extra.map(|dice| dice.and(dice));
        }
        Some(profile)
    }

    /// `blow` by `attacker` on `target` with Holy wrath's die added when it hit and its holder's
    /// wrath falls there (the unholy; with Avatar, any target): of the blow's kind, resisted but
    /// never soaked, as an `extra_damage`'s, and no zeal.
    pub(in crate::world) fn holy_wrathed(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        blow: Blow,
    ) -> Blow {
        if !blow.strike.is_hit() {
            return blow;
        }
        let (Some(by), Some(them)) = (
            self.state.actors.get(&attacker),
            self.state.actors.get(&target),
        ) else {
            return blow;
        };
        if by.talents.is_empty() {
            return blow;
        }
        let mods = by.talent_mods(&self.content);
        let Some(dice) = mods.faith.holy_wrath else {
            return blow;
        };
        if mods.faith.avatar.is_none() && !self.content.unholy(them.creature) {
            return blow;
        }
        let amount = dice.roll(&mut self.state.rng).max(0);
        let amount = self.resisted(
            target,
            Damage {
                amount,
                kind: blow.kind,
            },
        );
        Blow {
            damage: blow.damage + amount,
            ..blow
        }
    }

    /// The player's smite at `target`: their melee profile with the smite's dice (its unholy
    /// dice too against an unholy foe) as its extra damage and their `aim` modifier added to
    /// hit; `None` without the Smite talent.
    pub(in crate::world) fn smite_profile(&self, target: ActorId) -> Option<Profile> {
        let content = &self.content;
        let me = self.state.player();
        let smite = me.talent_mods(content).faith.smite?;
        let mut profile = me.melee(content);
        if let Some(stat) = smite.aim {
            profile.to_hit += me.effective_stats(content).modifier(stat);
        }
        let unholy = (self.state.actors.get(&target)).is_some_and(|a| content.unholy(a.creature));
        // A magic weapon's extra die is folded into the bonus at its average.
        if let Some(more) = profile.extra {
            profile.bonus += (more.min() + more.max()) / 2;
        }
        profile.extra = Some(smite.dice_at(self.player_level(), unholy));
        Some(profile)
    }
}

impl World {
    /// Retribution (the paladin design §6.4; `Cue::Attacked`, `free_blow.rs`): hostile
    /// `attacker`'s melee attack on `target` is over and landed a blow (`hit`). When `target` is
    /// the player, holding Retribution and its cost, within its bands, not off guard, and the
    /// attacker still stands beside them, the cost is spent and their smite (the dice, the aim,
    /// the unholy die; no mode's rider) answers it, free but for one exertion. Unanswered, it is
    /// silent: no line, nothing spent. No draw while deciding.
    pub(in crate::world) fn retribution(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        hit: bool,
        events: &mut Vec<Event>,
    ) -> Option<crate::world::combat::free_blow::FreeBlow> {
        let player = self.state.player;
        if !hit || target != player || self.state.death.is_some() {
            return None;
        }
        let content = self.content.clone();
        let (me, them) = (self.state.player(), self.state.actors.get(&attacker)?);
        let cost = me.talent_mods(&content).faith.retribution?;
        let beside = me.level == them.level && me.pos.chebyshev(them.pos) == 1;
        let hostile = self.stance(player, attacker) == crate::content::Stance::Hostile;
        if !beside || !hostile || self.off_guard(player) {
            return None;
        }
        // Its own bands, as a holy gift's (`World::answered`).
        let bands = (content.talents().iter())
            .map(|(_, def)| def)
            .find(|def| {
                (def.effects.iter()).any(|e| {
                    matches!(
                        e,
                        crate::content::TalentEffect::Faith(FaithEffect::Retribution { .. })
                    )
                })
            })
            .and_then(|def| def.alignment);
        if let (Some(bands), Some(_)) = (bands, content.alignment())
            && !bands.allows(me.band(&content))
        {
            return None;
        }
        let profile = self.smite_profile(attacker)?;
        if !self.spend_resource(player, cost.resource, cost.amount, events) {
            return None;
        }
        self.exert(ActionKind::Melee);
        Some(crate::world::combat::free_blow::FreeBlow {
            by: player,
            at: attacker,
            kind: crate::world::combat::free_blow::Free::Retribution,
            with: None,
            profile,
        })
    }
}
