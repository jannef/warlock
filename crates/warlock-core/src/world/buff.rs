//! Status effects as buffs, hexes, and cooldowns (spec §8 "Status effects", #227): what a status
//! does besides ticking and healing, while it lasts and as it ends. What each does is data
//! (`data/status.toml`, [`crate::content::StatusDef`]: `penalty`, `combat`, `cost`, `blocks`,
//! `follows`); the one way to give a status is still [`World::afflict`] (an ability that buffs
//! the player is `afflict(player, status, 1, player)`).
//!
//! - **Stats**: a `penalty` may raise scores as well as lower them (Raging: Strength +2); every
//!   rule reads them through [`super::Actor::effective_stats`].
//! - **Fighting**: a status's `combat` to-hit and damage join every attack profile with the
//!   combat stance's ([`super::Actor::stanced`]); its Dodge and Parry join the actor's Defence
//!   ([`super::Actor::defence`]). Creatures too: a hex is a status on a creature.
//! - **Slowness**: its `cost` multiplies the price of every action the actor takes, after its
//!   burden and hunger (`World::burdened`, the one place every price passes).
//! - **Blocks**: the player cannot take an action of a kind a status of theirs `blocks` (a raging
//!   barbarian cannot read): the rules refuse it, taking no time ([`World::refuse_blocked`]), and
//!   the abilities menu greys out an ability that costs one
//!   ([`crate::ability::Unavailable::While`]). Creatures' plans consult it for steps only: one a
//!   status of its blocks `step` (Held) neither steps nor opens a door, and fights back
//!   (`lucky.rs`).
//! - **Follows**: as a status ends, worn off or cured, the status it `follows` begins at one
//!   degree, caused by whoever caused the one that ended (Raging, then Exhausted); hit points
//!   above a maximum the ended status raised fall back to it ([`World::ended`]).
//! - **Cooldowns** are statuses with `eases` and no tick: while one lasts, the ability it limits
//!   cannot be used ([`crate::ability::Unavailable::While`]), a visible limit that only time
//!   lifts.

use super::{Spent, World};
use crate::action::{Action, Event};
use crate::content::{ActionKind, Stance};
use crate::geom::Dir;
use crate::status::Effect;
use crate::world::ActorId;

impl World {
    /// Refuses the player's `action` while one of their statuses blocks actions of its kind
    /// ("You cannot read while raging."), taking no time; `None` lets it go ahead.
    pub(super) fn refuse_blocked(&mut self, action: &Action) -> Option<Spent> {
        let kind = self.kind_of(action)?;
        let status = (self.state.player().effects).blocking(&self.content, kind)?;
        let name = self.content.status(status).name.to_lowercase();
        self.say(format!("You cannot {} while {name}.", kind.doing()));
        Some(Spent::Nothing)
    }

    /// The kind of action the player's `action` is, as a status's `blocks` names it; `None` for
    /// what is no action in the world (settings, trading, the dev actions).
    fn kind_of(&self, action: &Action) -> Option<ActionKind> {
        Some(match action {
            Action::Move(dir) | Action::Pay(Some(dir)) => self.move_kind(*dir),
            Action::Wait | Action::Rest(_) | Action::SecondWind | Action::BloodMagic => {
                ActionKind::Wait
            }
            Action::Open(_) => ActionKind::OpenDoor,
            Action::Close(_) => ActionKind::CloseDoor,
            Action::Kick(_) => ActionKind::Kick,
            Action::PickLock(_) => ActionKind::PickLock,
            Action::Ascend | Action::Descend => ActionKind::Stairs,
            Action::ToggleLantern => ActionKind::Lantern,
            Action::HoodLantern => ActionKind::Hood,
            Action::PickUp(_) => ActionKind::PickUp,
            Action::Drop(_) => ActionKind::Drop,
            Action::Equip { slot, .. } | Action::Remove(slot) => {
                self.content.slot(self.content.slot_id(slot)?).action
            }
            Action::Bash(_) => ActionKind::Melee,
            Action::Fire(_) => ActionKind::Shoot,
            Action::Throw { .. } => ActionKind::Throw,
            Action::Chat(_) | Action::Ask { .. } => ActionKind::Chat,
            Action::Steal(_) => ActionKind::Steal,
            Action::ThrowVoice(_) => ActionKind::ThrowVoice,
            Action::Eat(_) => ActionKind::Eat,
            Action::Read { .. } => ActionKind::Read,
            Action::Dig(_) => ActionKind::Dig,
            Action::Search(_) => ActionKind::Search,
            Action::Disarm(_) => ActionKind::Disarm,
            Action::Use(active, _) => active.cost(),
            Action::Gift(active, _) => active.cost(),
            Action::Sell(_)
            | Action::Pay(None)
            | Action::SetStance(_)
            | Action::SetVision(_)
            | Action::SkipTime(_)
            | Action::SetWeather(_)
            | Action::Effect { .. }
            | Action::Identify(_)
            | Action::Afflict(_)
            | Action::Drain(_)
            | Action::Resource(_)
            | Action::Infect(_)
            | Action::Talent { .. }
            | Action::Advance(_)
            | Action::LevelUpAuto
            | Action::GrantXp(_)
            | Action::Freeze { .. }
            | Action::Thaw { .. } => return None,
        })
    }

    /// What a step in `dir` would be for the player: an attack on a hostile creature there,
    /// opening a closed door, leaving a local area by its edge (as stairs), or a step.
    fn move_kind(&self, dir: Dir) -> ActionKind {
        let level = self.state.current();
        let to = self.player_pos().step(dir);
        let Some(&terrain) = level.terrain.get(to) else {
            return if level.exit.is_some() {
                ActionKind::Stairs
            } else {
                ActionKind::Step
            };
        };
        if self.content.terrain(terrain).is_closed_door() {
            return ActionKind::OpenDoor;
        }
        match self.state.actor_at(&level.id, to) {
            Some(other) if self.stance(self.state.player, other.id) == Stance::Hostile => {
                ActionKind::Melee
            }
            _ => ActionKind::Step,
        }
    }

    /// Statuses have just ended on actor `id`, worn off or cured (`ended`, their effects as they
    /// last were): the player is told, each status that one `follows` begins at one degree,
    /// caused by whoever caused the one that ended, and hit points above a maximum an ended
    /// status raised fall back within it.
    pub(super) fn ended(&mut self, id: ActorId, ended: &[Effect], events: &mut Vec<Event>) {
        if ended.is_empty() {
            return;
        }
        if id == self.state.player {
            events.extend(ended.iter().map(|e| Event::Status {
                actor: id,
                status: e.status,
                began: false,
            }));
        }
        for e in ended {
            if let Some(next) = self.content.status(e.status).follows {
                self.afflict(id, next, 1, e.by, events);
            }
        }
        self.keep_hp_within_max(id);
    }
}
