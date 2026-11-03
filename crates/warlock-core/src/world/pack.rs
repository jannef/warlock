//! Pack tactics, the ranger's level 18 (spec §8 "Class abilities"; docs/design/ranger-companion.md
//! §11.1; `docs/dev/companion.md`): one hook in `fight.rs::resolve`, [`World::pack_blow`].
//!
//! - **Flanking**: a creature beside both the player and their companion (Chebyshev 1, the
//!   companion on the level and out of the retinue) is flanked: the player's melee blows and
//!   the companion's at it are `TalentMods::pack_tactics` likelier to hit. Shots, throws, and
//!   spells are not (flanking is about footing).
//! - **The shared prey** (`TalentMods::shares_favoured_prey`): the companion's attacks at a
//!   creature of the player's favoured prey carry the player's `TalentMods::favoured_prey`
//!   (Favoured prey's, and Master hunter's added), to hit and damage.
//!
//! Only the player's talents are read: a companion is the player's. Nothing is said: the
//! bonus is a hidden number, as every to-hit is.

use super::World;
use crate::action::AttackKind;
use crate::combat::Profile;
use crate::world::ActorId;

impl World {
    /// `profile` of an attack of `kind` by `attacker` at `target`, with Pack tactics' gifts
    /// (the module docs); as it was for anyone else. One of `fight.rs::resolve`'s hooks.
    pub(super) fn pack_blow(
        &self,
        attacker: ActorId,
        target: ActorId,
        kind: AttackKind,
        mut profile: Profile,
    ) -> Profile {
        let player = self.state.player;
        let me = self.state.player();
        if me.talents.is_empty() {
            return profile;
        }
        let Some(companion) = self.companion() else {
            return profile;
        };
        if attacker != player && attacker != companion {
            return profile;
        }
        let mods = me.talent_mods(&self.content);
        if kind == AttackKind::Melee && mods.pack_tactics != 0 && self.flanked(companion, target) {
            profile.to_hit += mods.pack_tactics;
        }
        if attacker == companion && mods.shares_favoured_prey && self.is_prey(target) {
            let (to_hit, damage) = mods.favoured_prey;
            profile.to_hit += to_hit;
            profile.bonus += damage;
        }
        profile
    }

    /// Whether `target` stands beside both the player and `companion`, on the level and out of
    /// the retinue.
    fn flanked(&self, companion: ActorId, target: ActorId) -> bool {
        let actors = &self.state.actors;
        let (Some(me), Some(pet), Some(foe)) = (
            actors.get(&self.state.player),
            actors.get(&companion),
            actors.get(&target),
        ) else {
            return false;
        };
        let here = |a: &crate::world::Actor| a.level == self.state.current_level && !a.in_retinue();
        here(pet)
            && here(foe)
            && foe.id != companion
            && foe.id != me.id
            && foe.pos.chebyshev(me.pos) == 1
            && foe.pos.chebyshev(pet.pos) == 1
    }

    /// Whether `target` is of the player's favoured prey, the type they have killed most.
    fn is_prey(&self, target: ActorId) -> bool {
        let prey = self.state.quests.prey();
        prey.is_some()
            && (self.state.actors.get(&target))
                .is_some_and(|a| prey == Some(self.content.family_key(a.creature)))
    }
}
