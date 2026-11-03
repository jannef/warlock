//! Smiting (spec §8 "Class abilities": "Smite"; docs/design/paladin-smites.md §5): the paladin's
//! blow of holy wrath, paid in zeal.
//!
//! A smite is a melee blow with what the player wields at the hostile in a direction, priced as
//! the attack, with the smite's dice more (`Smite::dice_at`: a die per ten levels, at least one,
//! and its `unholy` dice more against a creature type `Content::unholy` names), and the smiter's
//! modifier in the talent's `aim` stat (Presence) added to hit. `use_class` has already refused
//! it with nobody hostile there, asked the bands (`World::answered`), and spent its zeal. Every
//! mode of smiting resolves here ([`SmiteMode`]): the blow, then the mode's rider on a hit that
//! did damage. Nothing here draws at random but the blow itself.

use super::{ActorId, Spent, World};
use crate::action::{AttackKind, Event};
use crate::combat::Profile;
use crate::content::ActionKind;
use crate::geom::Dir;

/// The modes of smiting: the plain smite, and (later slices) the smites with a rider.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SmiteMode {
    /// The Smite ability itself: the blow and nothing more.
    Plain,
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
        let Some(profile) = self.smite_profile(target) else {
            return Spent::Nothing;
        };
        let me = self.state.player();
        let melee = me.attack_skills(&content, AttackKind::Melee, None);
        let with = me.weapon_item(&content);
        let price = self.attack_price(player, ActionKind::Melee, with, melee);
        match mode {
            SmiteMode::Plain => self.say("You smite!"),
        }
        let blow = self.player_blow_with(target, seen, profile, events);
        self.after_attack(player, target, super::free_blow::parried(blow), events);
        self.exert(ActionKind::Melee);
        Spent::Turn(price)
    }

    /// The player's smite at `target`: their melee profile with the smite's dice (its unholy
    /// dice too against an unholy foe) as its extra damage and their `aim` modifier added to
    /// hit; `None` without the Smite talent.
    pub(super) fn smite_profile(&self, target: ActorId) -> Option<Profile> {
        let content = &self.content;
        let me = self.state.player();
        let smite = me.talent_mods(content).smite?;
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
    pub(super) fn retribution(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        hit: bool,
        events: &mut Vec<Event>,
    ) -> Option<super::free_blow::FreeBlow> {
        let player = self.state.player;
        if !hit || target != player || self.state.death.is_some() {
            return None;
        }
        let content = self.content.clone();
        let (me, them) = (self.state.player(), self.state.actors.get(&attacker)?);
        let cost = me.talent_mods(&content).retribution?;
        let beside = me.level == them.level && me.pos.chebyshev(them.pos) == 1;
        let hostile = self.stance(player, attacker) == crate::content::Stance::Hostile;
        if !beside || !hostile || self.off_guard(player) {
            return None;
        }
        // Its own bands, as a holy gift's (`World::answered`).
        let bands = (content.talents().iter())
            .map(|(_, def)| def)
            .find(|def| {
                (def.effects.iter())
                    .any(|e| matches!(e, crate::content::TalentEffect::Retribution { .. }))
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
        Some(super::free_blow::FreeBlow {
            by: player,
            at: attacker,
            kind: super::free_blow::Free::Retribution,
            with: None,
            profile,
        })
    }
}
