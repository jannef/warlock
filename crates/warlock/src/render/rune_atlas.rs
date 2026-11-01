//! The spell circles' rune atlas (spec §4.8 "Spell circles"): every rune of `data/runes.toml`
//! rasterised once from the rune font (`assets/fonts/NotoSansJP-Runes.ttf`, a subset of Noto
//! Sans JP Medium holding the runes alone; `assets/fonts/README.md`) into square slots.
//!
//! A rune is drawn [`RUNE_SIZE`] cell heights square, turned to face its circle's centre, as it
//! orbits between pixels, so there is only a supersampled copy ([`SUPERSAMPLE`]×, linear): each
//! slot holds the rune's ink centred in a square of `SUPERSAMPLE × ⌈size⌉` texels, padded on
//! every side for its halo and its backdrop (the coverage blurred, in the green and blue
//! channels) and a one-texel empty border, so filtering never reads a neighbour. Slot `i` is rune row `i` of [`Runes::rows`], in file order, at most
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

/// The rasterised runes in CPU memory, four bytes a texel: the rune's coverage (red), its halo
/// (green: the coverage blurred by [`HALO_SIGMA`]), and its backdrop (blue: blurred wider by
/// [`BACKDROP_SIGMA`] and spread, so it lies under the whole rune, strokes and gaps).
#[derive(Debug, PartialEq)]
pub struct RuneBitmap {
    /// A slot's side in texels, its border included.
    pub slot: u32,
    /// The texels about a slot's rune square on each side: room for the halo and the backdrop,
    /// and a one-texel empty border so filtering never reads a neighbour.
    pub pad: u32,
    pub size: UVec2,
    pub pixels: Vec<u8>,
}

/// The halo's blur, in rune squares (1.2 px for a 10 px rune).
pub const HALO_SIGMA: f32 = 0.12;
/// The backdrop's blur, in rune squares (2.5 px for a 10 px rune).
pub const BACKDROP_SIGMA: f32 = 0.25;
/// How much the halo's and the backdrop's blurred coverage is raised (then clamped to 1): the
/// halo a little, the backdrop so it covers the strokes and the gaps between them.
const HALO_GAIN: f32 = 1.5;
const BACKDROP_GAIN: f32 = 3.0;

/// Rasterises `runes` (in order, at most [`MAX_RUNES`]) for a rune square of `px` screen pixels.
pub fn rasterize(font: &fontdue::Font, runes: &[char], px: f32) -> RuneBitmap {
    let runes = &runes[..runes.len().min(MAX_RUNES)];
    let inner = px.ceil().max(1.0) as u32 * SUPERSAMPLE;
    let texels = px * SUPERSAMPLE as f32;
    let (halo, backdrop) = (HALO_SIGMA * texels, BACKDROP_SIGMA * texels);
    let pad = (3.0 * backdrop).ceil() as u32 + 1;
    let slot = inner + 2 * pad;
    let rows = (runes.len() as u32).div_ceil(COLUMNS).max(1);
    let size = UVec2::new(COLUMNS * slot, rows * slot);
    let mut pixels = vec![0u8; (size.x * size.y * 4) as usize];
    let square = pad as i32..(pad + inner) as i32;
    for (i, &c) in runes.iter().enumerate() {
        let (m, bitmap) = font.rasterize(c, texels);
        // The ink centred in the slot's square: a rune's place on the ring is its middle.
        let mut ink = vec![0.0f32; (slot * slot) as usize];
        let left = pad as i32 + (inner as i32 - m.width as i32) / 2;
        let top = pad as i32 + (inner as i32 - m.height as i32) / 2;
        for gy in 0..m.height as i32 {
            for gx in 0..m.width as i32 {
                let (x, y) = (left + gx, top + gy);
                if !square.contains(&x) || !square.contains(&y) {
                    continue;
                }
                let v = bitmap[(gy * m.width as i32 + gx) as usize];
                ink[(y as u32 * slot + x as u32) as usize] = f32::from(v) / 255.0;
            }
        }
        let glow = blur(&ink, slot, halo);
        let under = blur(&ink, slot, backdrop);
        let origin = UVec2::new(i as u32 % COLUMNS, i as u32 / COLUMNS) * slot;
        let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        for y in 1..slot - 1 {
            for x in 1..slot - 1 {
                let k = (y * slot + x) as usize;
                let at = (((origin.y + y) * size.x + origin.x + x) * 4) as usize;
                pixels[at] = byte(ink[k]);
                pixels[at + 1] = byte(glow[k] * HALO_GAIN);
                pixels[at + 2] = byte(under[k] * BACKDROP_GAIN);
                pixels[at + 3] = 255;
            }
        }
    }
    RuneBitmap {
        slot,
        pad,
        size,
        pixels,
    }
}

/// A Gaussian blur of the `side × side` image `src`, deviation `sigma` texels (separable, zero
/// outside).
fn blur(src: &[f32], side: u32, sigma: f32) -> Vec<f32> {
    let reach = (3.0 * sigma).ceil() as i32;
    let weights: Vec<f32> = (-reach..=reach)
        .map(|d| (-((d * d) as f32) / (2.0 * sigma * sigma)).exp())
        .collect();
    let total: f32 = weights.iter().sum();
    let n = side as i32;
    let pass = |src: &[f32], step: (i32, i32)| -> Vec<f32> {
        let mut out = vec![0.0; src.len()];
        for y in 0..n {
            for x in 0..n {
                let mut sum = 0.0;
                for (j, w) in weights.iter().enumerate() {
                    let d = j as i32 - reach;
                    let (sx, sy) = (x + d * step.0, y + d * step.1);
                    if (0..n).contains(&sx) && (0..n).contains(&sy) {
                        sum += w * src[(sy * n + sx) as usize];
                    }
                }
                out[(y * n + x) as usize] = sum / total;
            }
        }
        out
    };
    pass(&pass(src, (1, 0)), (0, 1))
}

/// The atlas as rendering uses it.
#[derive(Resource, Debug)]
pub struct RuneAtlas {
    pub image: Handle<Image>,
    /// A slot's side in texels, its border included.
    pub slot: u32,
    /// The texels about a slot's rune square on each side ([`RuneBitmap::pad`]).
    pub pad: u32,
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
        let (slot, pad) = (bitmap.slot, bitmap.pad);
        let mut image = Image::new(
            Extent3d {
                width: bitmap.size.x,
                height: bitmap.size.y,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            bitmap.pixels,
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::RENDER_WORLD,
        );
        image.sampler = ImageSampler::linear();
        Self {
            image: images.add(image),
            slot,
            pad,
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
        let (pad, inner) = (atlas.pad, 10 * SUPERSAMPLE);
        assert_eq!(atlas.slot, inner + 2 * pad);
        assert!(pad as f32 > 3.0 * BACKDROP_SIGMA * 9.9 * SUPERSAMPLE as f32);
        assert_eq!(atlas.size, UVec2::new(COLUMNS * atlas.slot, 2 * atlas.slot));
        let channel = |c: u32, i: u32, x: u32, y: u32| {
            let o = UVec2::new(i % COLUMNS, i / COLUMNS) * atlas.slot;
            atlas.pixels[(((o.y + y) * atlas.size.x + o.x + x) * 4 + c) as usize]
        };
        let texels = || (0..atlas.slot).flat_map(|y| (0..atlas.slot).map(move |x| (x, y)));
        let inside = |v: u32| (pad..pad + inner).contains(&v);
        for i in 0..chars.len() as u32 {
            let sum =
                |c: u32| -> u32 { texels().map(|(x, y)| u32::from(channel(c, i, x, y))).sum() };
            assert!(sum(0) > 0, "rune {i} ({}) has ink", chars[i as usize]);
            // The halo reaches past the ink and the backdrop farther, both beyond the square.
            assert!(sum(2) > sum(1) && sum(1) > sum(0), "rune {i}");
            let outside = |c: u32| {
                texels().any(|(x, y)| !(inside(x) && inside(y)) && channel(c, i, x, y) > 0)
            };
            assert!(!outside(0) && outside(1) && outside(2), "rune {i}");
            for e in 0..atlas.slot {
                let last = atlas.slot - 1;
                for (x, y) in [(e, 0), (e, last), (0, e), (last, e)] {
                    for c in 0..3 {
                        assert_eq!(channel(c, i, x, y), 0, "rune {i}'s border is empty");
                    }
                }
            }
        }
    }
}
