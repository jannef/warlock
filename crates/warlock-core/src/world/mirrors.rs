//! Halls of mirrors (spec §7 "Special rooms"; `docs/dev/special-rooms.md`): every actor inside
//! a room whose effect holds `mirrors` has that many **images**, reflections on free floor of the
//! room. Where they stand is no draw: a hash of the level, the turn, the actor, and the image's
//! index ([`WorldState::images`]), so the rules and perception compute the same images and the
//! random streams never move. Perception shows the images of the actors the player sees as those
//! actors, under ids of their own ([`image_id`]), with nothing that tells them apart; Mirror-wise
//! leaves out those near the player.
//!
//! What the images do in the rules: a step or a blow into an image the player sees meets only
//! glass ([`World::meets_glass`], the blow's time); a missile aimed at an image's cell flies on
//! through it ([`World::reflected`]); a creature shooting or throwing at the player from outside
//! the hall aims at the player or one of their images, a gameplay draw ([`World::aim_at`]).
//! Creatures inside know the hall and are not fooled; creatures never see images at all.

use super::{ActorId, ActorKind, LevelState, World, WorldState, fnv1a};
use crate::action::AttackKind;
use crate::content::ActionKind;
use crate::content::Content;
use crate::geom::Pos;
use std::collections::BTreeSet;

/// Where images' ids begin: above every real actor's.
const IMAGE_IDS: u32 = 1 << 31;

/// One reflection in a hall of mirrors: the `index`th image of actor `of`, at `pos`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Image {
    pub of: ActorId,
    pub index: u8,
    pub pos: Pos,
}

/// The id perception shows the `index`th image of actor `of` under on `turn`: never a real
/// actor's, and another on the next turn, so an image never seems to slide across the hall (it
/// is seen anew where it stands).
pub(crate) fn image_id(of: ActorId, index: u8, turn: u64) -> ActorId {
    let parity = u32::from(turn % 2 == 1);
    ActorId(IMAGE_IDS + ((of.0 & 0x00ff_ffff) * 2 + parity) * 8 + u32::from(index % 8))
}

impl WorldState {
    /// Every image on `level` now (ground truth), hall by hall in the level's room order, and in
    /// each the actors inside in id order, each its `images` in turn: an image takes a free floor
    /// cell of the hall (walkable, no door, no one standing on it, no image yet) picked by a hash
    /// of (level, turn, actor, index). A hall with fewer free cells shows fewer.
    pub(crate) fn images(&self, content: &Content, level: &LevelState) -> Vec<Image> {
        let halls = (level.rooms.iter())
            .filter_map(|r| content.room_effect(r.effect).mirrors.map(|m| (r, m.images)));
        let mut images = Vec::new();
        let mut actors = None;
        for (room, count) in halls {
            let actors: &Vec<(ActorId, Pos)> = actors.get_or_insert_with(|| {
                (self.actors.values())
                    .filter(|a| a.level == level.id)
                    .map(|a| (a.id, a.pos))
                    .collect()
            });
            let taken: BTreeSet<Pos> = actors.iter().map(|(_, p)| *p).collect();
            let mut free: Vec<Pos> = (room.cells().into_iter())
                .filter(|&p| {
                    let def = level.terrain.get(p).map(|&t| content.terrain(t));
                    level.is_walkable(content, p)
                        && def.is_some_and(|d| !d.is_closed_door() && !d.is_open_door())
                        && !taken.contains(&p)
                })
                .collect();
            for &(of, _) in actors.iter().filter(|(_, p)| room.contains(*p)) {
                for index in 0..count {
                    if free.is_empty() {
                        break;
                    }
                    let mut key = Vec::with_capacity(level.id.len() + 13);
                    key.extend_from_slice(level.id.as_bytes());
                    key.extend_from_slice(&self.turn.to_le_bytes());
                    key.extend_from_slice(&of.0.to_le_bytes());
                    key.push(index);
                    let at = (fnv1a(&key) % free.len() as u64) as usize;
                    images.push(Image {
                        of,
                        index,
                        pos: free.remove(at),
                    });
                }
            }
        }
        images
    }
}

impl World {
    /// Every image on the current level now (ground truth; [`WorldState::images`]).
    pub fn images(&self) -> Vec<Image> {
        self.state.images(&self.content, self.state.current())
    }

    /// The player steps or strikes into `to`, where no one stands: if they see an image there,
    /// their blow meets only glass, and it takes a blow's time.
    pub(super) fn meets_glass(&mut self, to: Pos) -> Option<i64> {
        if !self.perception.shows_image_at(to) {
            return None;
        }
        let player = self.state.player;
        let me = self.state.player();
        let melee = me.attack_skills(&self.content, AttackKind::Melee, None);
        let with = me.weapon_item(&self.content);
        let price = self.attack_price(player, ActionKind::Melee, with, melee);
        self.say("Your blow meets only glass.");
        Some(price)
    }

    /// Whether `at`, where `shooter` aims, holds an image to them: one the player sees, for the
    /// player; any, for a creature (which aimed at it, [`World::aim_at`]). A missile flies on
    /// through it.
    pub(super) fn reflected(&self, shooter: ActorId, at: Pos) -> bool {
        if shooter == self.state.player {
            return self.perception.shows_image_at(at);
        }
        self.images().iter().any(|i| i.pos == at)
    }

    /// Where creature `shooter` aims a missile at `target`, which stands at `at`: at the player
    /// in a hall of mirrors, from outside it, at the player or one of their images (a gameplay
    /// draw, each alike); anywhere else at `at`.
    pub(super) fn aim_at(&mut self, shooter: ActorId, target: ActorId, at: Pos) -> Pos {
        let level = self.state.current();
        let is_player =
            (self.state.actors.get(&target)).is_some_and(|a| a.kind == ActorKind::Player);
        let Some(hall) = level.room_at(at).filter(|_| is_player) else {
            return at;
        };
        if self.content.room_effect(hall.effect).mirrors.is_none()
            || self
                .state
                .actors
                .get(&shooter)
                .is_none_or(|a| hall.contains(a.pos))
        {
            return at;
        }
        let images: Vec<Pos> = (self.images().into_iter())
            .filter(|i| i.of == target)
            .map(|i| i.pos)
            .collect();
        if images.is_empty() {
            return at;
        }
        match self.state.rng.below(images.len() as u32 + 1) as usize {
            0 => at,
            k => images[k - 1],
        }
    }
}
