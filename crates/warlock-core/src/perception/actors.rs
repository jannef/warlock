//! The creatures the player sees: how they look.

use super::Shape;
use crate::color::Rgb;
use crate::content::Content;
use crate::world::Actor;

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
