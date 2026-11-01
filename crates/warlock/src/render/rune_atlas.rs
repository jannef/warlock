//! The spell circles' rune atlas (spec §4.8 "Spell circles"): every rune of `data/runes.toml`
//! rasterised once from the rune font (`assets/fonts/NotoSansJP-Runes.ttf`, a subset of Noto
//! Sans JP Medium holding the runes alone; `assets/fonts/README.md`) into square slots.
//!
//! A rune is drawn [`RUNE_SIZE`] cell heights square, upright, as it orbits between pixels, so
//! there is only a supersampled copy ([`SUPERSAMPLE`]×, linear): each slot holds the rune's ink
//! centred in `SUPERSAMPLE × ⌈size⌉` texels plus a one-texel empty border, so filtering never
//! reads a neighbour. Slot `i` is rune row `i` of [`Runes::rows`], in file order, at most
//! [`MAX_RUNES`]. The atlas is rebuilt when the cell height changes (the font size or the
//! display scale), never per frame.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use warlock_core::content::Runes;

const FONT_BYTES: &[u8] = include_bytes!("../../../../assets/fonts/NotoSansJP-Runes.ttf");

/// A rune's square, in cell heights (10 px at the default 18 px font).
pub const RUNE_SIZE: f32 = 0.55;

/// Texels per screen pixel of a rune's slot.
pub const SUPERSAMPLE: u32 = 2;

/// Slots per atlas row.
pub const COLUMNS: u32 = 8;

/// The most runes the atlas holds (eight rows of [`COLUMNS`]).
pub const MAX_RUNES: usize = 64;

/// The rune font, parsed once.
pub fn font() -> fontdue::Font {
    fontdue::Font::from_bytes(FONT_BYTES, fontdue::FontSettings::default())
        .expect("the bundled rune font is valid")
}

/// The rasterised runes in CPU memory (single-channel coverage).
#[derive(Debug, PartialEq)]
pub struct RuneBitmap {
    /// A slot's side in texels, its border included.
    pub slot: u32,
    pub size: UVec2,
    pub pixels: Vec<u8>,
}

/// Rasterises `runes` (in order, at most [`MAX_RUNES`]) for a rune square of `px` screen pixels.
pub fn rasterize(font: &fontdue::Font, runes: &[char], px: f32) -> RuneBitmap {
    let runes = &runes[..runes.len().min(MAX_RUNES)];
    let inner = px.ceil().max(1.0) as u32 * SUPERSAMPLE;
    let slot = inner + 2;
    let rows = (runes.len() as u32).div_ceil(COLUMNS).max(1);
    let size = UVec2::new(COLUMNS * slot, rows * slot);
    let mut pixels = vec![0u8; (size.x * size.y) as usize];
    let scale = px * SUPERSAMPLE as f32;
    for (i, &c) in runes.iter().enumerate() {
        let (m, bitmap) = font.rasterize(c, scale);
        let origin = UVec2::new(i as u32 % COLUMNS, i as u32 / COLUMNS) * slot + UVec2::ONE;
        // The ink centred in the slot: a rune's place on the ring is its middle.
        let left = (inner as i32 - m.width as i32) / 2;
        let top = (inner as i32 - m.height as i32) / 2;
        for gy in 0..m.height as i32 {
            for gx in 0..m.width as i32 {
                let (x, y) = (left + gx, top + gy);
                if x < 0 || y < 0 || x >= inner as i32 || y >= inner as i32 {
                    continue;
                }
                let at = (origin.y + y as u32) * size.x + origin.x + x as u32;
                pixels[at as usize] = bitmap[(gy * m.width as i32 + gx) as usize];
            }
        }
    }
    RuneBitmap { slot, size, pixels }
}

/// The atlas as rendering uses it.
#[derive(Resource, Debug)]
pub struct RuneAtlas {
    pub image: Handle<Image>,
    /// A slot's side in texels, its border included.
    pub slot: u32,
    /// The cell height it was built for.
    pub cell_height: f32,
}

impl RuneAtlas {
    /// Rasterises every rune of `runes` for a cell height of `cell_height` pixels.
    pub fn new(
        font: &fontdue::Font,
        runes: &Runes,
        cell_height: f32,
        images: &mut Assets<Image>,
    ) -> Self {
        let chars: Vec<char> = runes.rows().iter().map(|r| r.rune).collect();
        let bitmap = rasterize(font, &chars, RUNE_SIZE * cell_height);
        let mut image = Image::new(
            Extent3d {
                width: bitmap.size.x,
                height: bitmap.size.y,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            bitmap.pixels,
            TextureFormat::R8Unorm,
            RenderAssetUsages::RENDER_WORLD,
        );
        image.sampler = ImageSampler::linear();
        Self {
            image: images.add(image),
            slot: bitmap.slot,
            cell_height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use warlock_core::content::Content;

    /// The subset is current: every rune of `data/runes.toml` has a glyph in the rune font
    /// (a new rune's PR runs `pyftsubset` again, `assets/fonts/README.md`).
    #[test]
    fn every_rune_is_in_the_rune_font() {
        let content = Content::builtin().unwrap();
        let font = font();
        let rows = content.runes().rows();
        assert!(!rows.is_empty());
        assert!(rows.len() <= MAX_RUNES, "the atlas holds {MAX_RUNES} runes");
        for row in rows {
            assert_ne!(
                font.lookup_glyph_index(row.rune),
                0,
                "{} is not in the font",
                row.rune
            );
        }
    }

    #[test]
    fn runes_are_centred_in_bordered_slots() {
        let content = Content::builtin().unwrap();
        let chars: Vec<char> = content.runes().rows().iter().map(|r| r.rune).collect();
        // 0.55 of the default 18 px font: a 10 px rune.
        let atlas = rasterize(&font(), &chars, 9.9);
        assert_eq!(atlas.slot, 10 * SUPERSAMPLE + 2);
        assert_eq!(atlas.size, UVec2::new(COLUMNS * atlas.slot, 2 * atlas.slot));
        let texel = |i: u32, x: u32, y: u32| {
            let o = UVec2::new(i % COLUMNS, i / COLUMNS) * atlas.slot;
            atlas.pixels[((o.y + y) * atlas.size.x + o.x + x) as usize]
        };
        for i in 0..chars.len() as u32 {
            let ink: u32 = (0..atlas.slot)
                .flat_map(|y| (0..atlas.slot).map(move |x| (x, y)))
                .map(|(x, y)| u32::from(texel(i, x, y)))
                .sum();
            assert!(ink > 0, "rune {i} ({}) has ink", chars[i as usize]);
            for e in 0..atlas.slot {
                let last = atlas.slot - 1;
                for (x, y) in [(e, 0), (e, last), (0, e), (last, e)] {
                    assert_eq!(texel(i, x, y), 0, "rune {i}'s border is empty");
                }
            }
        }
    }
}
