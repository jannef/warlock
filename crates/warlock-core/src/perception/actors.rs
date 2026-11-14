//! The creatures the player sees, the fourth step of
//! [`Perception::compute`](super::Perception::compute) ([`seen`]): who is hidden from them (the
//! unseen, disguised mimics), who they see and how each looks, what they wear and wield, the
//! statuses that show on them, and a hall of mirrors' images.

use super::seeing::Seeing;
use super::{Disguise, SeenActor, SeenGear, Shape, Sight, Viewing};
use crate::color::Rgb;
use crate::content::{Content, Stance, StatusId};
use crate::geom::Pos;
use crate::grid::Window;
use crate::light::{Light, LightCache};
use crate::world::mirrors::image_id;
use crate::world::surprise::creature_unaware;
use crate::world::{self, Actor, ActorId, ActorKind, Concealed};

/// The creatures the player sees, and those hidden from them.
pub(super) struct Seen {
    pub(super) actors: Vec<SeenActor>,
    pub(super) gear: Vec<SeenGear>,
    pub(super) effects: Vec<(ActorId, StatusId)>,
    pub(super) images: Vec<Pos>,
    pub(super) averted: bool,
    pub(super) disguises: Vec<Disguise>,
    pub(super) concealed: Vec<Concealed>,
    pub(super) draught: bool,
}

/// Who the player sees on the cells `seeing` shows, by `light` (the shadowy hide in the dim).
pub(super) fn seen(
    v: &Viewing,
    seeing: &Seeing,
    light: &Window<Light>,
    lights: &mut LightCache,
) -> Seen {
    let (state, content, level, viewer) = (v.state, v.content, v.level, v.viewer);
    let sight = &seeing.sight;
    // The unseen (`world/unseen.rs`): the shadowy on cells dim to the player's eyes, and the
    // invisible.
    let dim = |p: Pos| !v.overworld && light.get(p).is_some_and(|l| v.eye.dim(*l));
    let concealed = world::unseen_by_player(state, content, &dim, &seeing.scented);
    let near = content.tuning().unseen.draught;
    let draught = (concealed.iter()).any(|c| c.shadowy && c.pos.chebyshev(viewer) <= near);
    let unseen = |a: &Actor| concealed.iter().any(|c| c.id == a.id);
    let disguises = disguises(v, sight);
    let hidden = |a: &Actor| disguises.iter().any(|d| d.pos == a.pos);
    let seen: Vec<&Actor> = state
        .actors
        .values()
        .filter(|a| a.level == level.id && sight.get(a.pos) == Some(&Sight::Seen))
        // The retinue is with the player, not on the map (`world/ally.rs`).
        .filter(|a| !a.in_retinue())
        .filter(|a| !hidden(a) && !unseen(a))
        .collect();
    // Averted eyes (`world/avert.rs`): a hostile whose gaze is marked `averted`, seen.
    let looker = state.player().faction(content);
    let averted = (seen.iter()).any(|a| {
        a.id != state.player
            && (content.creature(a.creature).gaze.as_ref()).is_some_and(|g| g.averted)
            && content.stance_of(a.faction(content), looker) == Stance::Hostile
    });
    let mut actors: Vec<SeenActor> = (seen.iter())
        .map(|a| SeenActor {
            id: a.id,
            kind: a.kind,
            creature: a.creature,
            dialogue: content.dialogue_of(a.creature, a.npc.as_deref()),
            faction: a.faction(content),
            pos: a.pos,
            health: a.health(content),
            shape: shape(a, content),
            unaware: false,
            marked: a.id != state.player && state.player().mark == Some(a.id),
            ally: a.master() == Some(state.player),
            companion: a.master() == Some(state.player) && a.kept().is_some(),
            grade: a.graded(),
        })
        .collect();
    // Stalker's eye: which of them have not noticed the player.
    if v.talents.stealth.sees_unaware {
        for seen in &mut actors {
            seen.unaware = (seen.kind == ActorKind::Creature)
                && creature_unaware(state, content, lights, seen.id, state.player);
        }
    }
    // A hall of mirrors' images of those seen (`world/mirrors.rs`), where the player sees
    // their cells, each shown as what it reflects under an id of its own.
    let images: Vec<Image> = (state.images(content, level).into_iter())
        .filter(|i| sight.get(i.pos) == Some(&Sight::Seen))
        .filter(|i| seen.iter().any(|a| a.id == i.of))
        .map(|i| (i.of, image_id(i.of, i.index, state.turn), i.pos))
        .collect();
    for &(of, id, pos) in &images {
        if let Some(real) = actors.iter().find(|a| a.id == of).copied() {
            actors.push(SeenActor { id, pos, ..real });
        }
    }
    let gear = gear(&seen, &images);
    let effects = effects(content, &seen, &images);
    actors.sort_by_key(|a| a.id);
    Seen {
        actors,
        gear,
        effects,
        images: images.into_iter().map(|(_, _, p)| p).collect(),
        averted,
        disguises,
        concealed,
        draught,
    }
}

/// An image of a hall of mirrors: whom it reflects, its own id, and where it is.
type Image = (ActorId, ActorId, Pos);

/// The disguised mimics of the level: each a shut chest (`world/mimic.rs`), but to Chest-wise
/// within its reach where the player sees it.
fn disguises(v: &Viewing, sight: &Window<Sight>) -> Vec<Disguise> {
    let (content, viewer) = (v.content, v.viewer);
    let reach = v.talents.delving.sees_mimics;
    let shut = content.chest_terrain(crate::chest::ChestState::Shut);
    (v.state.actors.values())
        .filter(|a| a.level == v.level.id && a.effects.hides(content))
        .filter(|a| !(sight.get(a.pos) == Some(&Sight::Seen) && a.pos.chebyshev(viewer) <= reach))
        .filter_map(|a| {
            Some(Disguise {
                pos: a.pos,
                terrain: shut?,
                level: content.creature(a.creature).level,
            })
        })
        .collect()
}

/// The ids actor `a` is shown under: its own, then its images'.
fn shown(a: &Actor, images: &[Image]) -> Vec<ActorId> {
    std::iter::once(a.id)
        .chain(images.iter().filter(|i| i.0 == a.id).map(|i| i.1))
        .collect::<Vec<_>>()
}

/// What the seen creatures wear and wield, in plain sight; the rest of what they carry is not.
fn gear(seen: &[&Actor], images: &[Image]) -> Vec<SeenGear> {
    (seen.iter())
        .filter(|a| a.kind == ActorKind::Creature)
        .flat_map(|a| {
            shown(a, images).into_iter().flat_map(move |id| {
                (a.equipment.items().into_iter()).filter_map(move |(slot, item)| {
                    let stack = a.inventory.get(item)?.clone();
                    Some(SeenGear {
                        actor: id,
                        slot,
                        stack,
                    })
                })
            })
        })
        .collect()
}

/// The statuses that show on the seen creatures, such as bleeding; others (poison) do not, nor
/// those its kind always has (`world/aura.rs`: a salamander is always cloaked in flame).
fn effects(content: &Content, seen: &[&Actor], images: &[Image]) -> Vec<(ActorId, StatusId)> {
    (seen.iter())
        .filter(|a| a.kind == ActorKind::Creature)
        .flat_map(|a| {
            let innate = &content.creature(a.creature).innate;
            shown(a, images).into_iter().flat_map(move |id| {
                (a.effects.iter())
                    .filter(|e| content.status(e.status).seen.is_some())
                    .filter(|e| !innate.contains(&e.status))
                    .map(move |e| (id, e.status))
            })
        })
        .collect()
}

/// How actor `a` looks by what it is made of (spec §8 "Animated objects"): a household thing by
/// its body, its object's glyph in its material's colour; one of a family by the piece in its
/// tier's `shown_by` slot, in that piece's colour (and by its glyph with `item_glyph`: a blade
/// by its weapon's); `None` for any other, or one whose piece is gone.
pub(super) fn shape(a: &Actor, content: &Content) -> Option<Shape> {
    let def = content.creature(a.creature);
    if let Some(tier) = content.animated().family_tier(def.animated.as_ref()) {
        let slot = content.slot_id(tier.shown_by.as_deref()?)?;
        let stack = a.inventory.get(a.equipment.get(slot)?)?;
        let (bands, count) = bands(a, content);
        return Some(Shape {
            glyph: if tier.item_glyph {
                content.object(stack.object).glyph
            } else {
                def.glyph
            },
            fg: stack.fg(content),
            material: stack.parts(content).first()?.material,
            bands,
            count,
        });
    }
    let body = content.animated_body(a.creature)?;
    let stack = (a.inventory.iter()).find(|s| s.object == body.object)?;
    let material = stack.parts(content).first()?.material;
    Some(Shape {
        glyph: content.object(stack.object).glyph,
        fg: stack.fg(content),
        material,
        bands: [Rgb::BLACK; Shape::BANDS],
        count: 0,
    })
}

/// The distinct colours of what actor `a` wears and wields, in slot order (head to feet, more or
/// less: a helm over a hauberk over boots), the first [`Shape::BANDS`], and how many there are.
pub(super) fn bands(a: &Actor, content: &Content) -> ([Rgb; Shape::BANDS], u8) {
    let mut bands = [Rgb::BLACK; Shape::BANDS];
    let mut count = 0;
    let worn = (a.equipment.iter()).filter_map(|(_, i)| a.inventory.get(i));
    for fg in worn.map(|s| s.fg(content)) {
        if count < Shape::BANDS && !bands[..count].contains(&fg) {
            bands[count] = fg;
            count += 1;
        }
    }
    (bands, count as u8)
}
