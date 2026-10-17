//! Glyph atlas: a monospace TTF rasterized at the exact physical pixel size (spec §4.1).
//!
//! Every glyph occupies one cell-sized slot, so the shader can address a glyph's texels with
//! integer math and draw it 1:1 without filtering. A second, supersampled copy with the same slot
//! layout (each slot exactly [`SUPERSAMPLE`] times larger) is sampled with linear filtering by
//! glyphs that move within their cell, slide, or scale.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::BTreeMap;

const FONT_BYTES: &[u8] = include_bytes!("../../../../assets/fonts/JuliaMono-Medium.ttf");

/// Slots per atlas row.
pub const ATLAS_COLUMNS: u32 = 32;

/// Linear size of the supersampled atlas relative to the 1:1 atlas.
pub const SUPERSAMPLE: u32 = 2;

/// Glyphs the UI may draw in addition to printable ASCII and content glyphs.
const UI_GLYPHS: &str = "─│┌┐└┘├┤┬┴┼═║╔╗╚╝╠╣╦╩╬░▒▓█▀▄▌▐·•◆◇○●←↑→↓↖↗↘↙…≈";

/// The loaded font, parsed once.
#[derive(Resource)]
pub struct Font(pub fontdue::Font);

impl Font {
    pub fn load() -> Result<Self, String> {
        fontdue::Font::from_bytes(FONT_BYTES, fontdue::FontSettings::default())
            .map(Self)
            .map_err(|e| format!("cannot load the bundled font: {e}"))
    }
}

/// A rasterized atlas in CPU memory (single-channel coverage).
#[derive(Debug)]
pub struct AtlasBitmap {
    /// Slot size in this bitmap's pixels.
    pub cell: UVec2,
    pub size: UVec2,
    pub pixels: Vec<u8>,
    pub index: BTreeMap<char, u16>,
}

impl AtlasBitmap {
    fn into_image(self) -> Image {
        Image::new(
            Extent3d {
                width: self.size.x,
                height: self.size.y,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            self.pixels,
            TextureFormat::R8Unorm,
            RenderAssetUsages::RENDER_WORLD,
        )
    }
}

/// The atlas as used by rendering.
#[derive(Resource, Debug)]
pub struct GlyphAtlas {
    /// 1:1 atlas, read with integer texel addressing.
    pub image: Handle<Image>,
    /// [`SUPERSAMPLE`]× atlas with the same slot layout, read with linear filtering.
    pub smooth: Handle<Image>,
    /// Cell size in physical pixels.
    pub cell: UVec2,
    /// Font size in physical pixels the atlas was built for.
    pub px: f32,
    index: BTreeMap<char, u16>,
    fallback: u16,
}

impl GlyphAtlas {
    /// Rasterizes `chars` at `px` physical pixels into both atlases.
    pub fn new(font: &fontdue::Font, px: f32, chars: &[char], images: &mut Assets<Image>) -> Self {
        let crisp = rasterize(font, px, chars, 1);
        let mut smooth = rasterize(font, px, chars, SUPERSAMPLE).into_image();
        smooth.sampler = ImageSampler::linear();
        let fallback = crisp.index.get(&'?').copied().unwrap_or(0);
        let (cell, index) = (crisp.cell, crisp.index.clone());
        Self {
            image: images.add(crisp.into_image()),
            smooth: images.add(smooth),
            cell,
            px,
            index,
            fallback,
        }
    }

    /// Atlas slot for `c`; unknown characters render as `?`.
    pub fn glyph(&self, c: char) -> u16 {
        self.index.get(&c).copied().unwrap_or(self.fallback)
    }
}

/// The glyph set: blank first, then printable ASCII, UI glyphs, and `extra`.
pub fn glyph_set(extra: impl IntoIterator<Item = char>) -> Vec<char> {
    let mut chars = vec![' '];
    chars.extend((0x21u8..0x7f).map(char::from));
    chars.extend(UI_GLYPHS.chars());
    chars.extend(extra);
    let mut seen = std::collections::BTreeSet::new();
    chars.retain(|c| seen.insert(*c));
    chars
}

/// Rasterizes `chars` at `px` physical pixels into cell-sized slots, `supersample` times larger
/// (1 for the 1:1 atlas). Cell size and baseline are fixed at `px` and then scaled, so a
/// supersampled slot holds exactly the same outline as its 1:1 slot. Characters the font lacks
/// are left out, so they fall back to `?`.
pub fn rasterize(font: &fontdue::Font, px: f32, chars: &[char], supersample: u32) -> AtlasBitmap {
    let chars: Vec<char> = chars
        .iter()
        .copied()
        .filter(|&c| c == ' ' || font.lookup_glyph_index(c) != 0)
        .collect();
    let line = font.horizontal_line_metrics(px);
    let (ascent, descent) = line.map_or((px * 0.8, -px * 0.2), |l| (l.ascent, l.descent));
    let cell_w = font.metrics('M', px).advance_width.round().max(1.0) as u32 * supersample;
    let cell_h = (ascent - descent).ceil().max(1.0) as u32 * supersample;
    let baseline = ascent.round() as i32 * supersample as i32;
    let px = px * supersample as f32;

    let rows = (chars.len() as u32).div_ceil(ATLAS_COLUMNS).max(1);
    let size = UVec2::new(ATLAS_COLUMNS * cell_w, rows * cell_h);
    let mut pixels = vec![0u8; (size.x * size.y) as usize];
    let mut index = BTreeMap::new();

    for (i, &c) in chars.iter().enumerate() {
        let slot = i as u32;
        index.insert(c, slot as u16);
        if c == ' ' {
            continue;
        }
        let (m, bitmap) = font.rasterize(c, px);
        let origin_x = (slot % ATLAS_COLUMNS * cell_w) as i32;
        let origin_y = (slot / ATLAS_COLUMNS * cell_h) as i32;
        let top = baseline - (m.height as i32 + m.ymin);
        for gy in 0..m.height as i32 {
            for gx in 0..m.width as i32 {
                let (x, y) = (m.xmin + gx, top + gy);
                if x < 0 || y < 0 || x >= cell_w as i32 || y >= cell_h as i32 {
                    continue;
                }
                let dst = ((origin_y + y) as u32 * size.x + (origin_x + x) as u32) as usize;
                pixels[dst] = bitmap[(gy * m.width as i32 + gx) as usize];
            }
        }
    }
    AtlasBitmap {
        cell: UVec2::new(cell_w, cell_h),
        size,
        pixels,
        index,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Coverage of texel `(x, y)` of `c`'s slot.
    fn texel(atlas: &AtlasBitmap, c: char, x: u32, y: u32) -> u8 {
        let slot = u32::from(atlas.index[&c]);
        let (ox, oy) = (
            slot % ATLAS_COLUMNS * atlas.cell.x,
            slot / ATLAS_COLUMNS * atlas.cell.y,
        );
        atlas.pixels[((oy + y) * atlas.size.x + ox + x) as usize]
    }

    #[test]
    fn rasterizes_into_cell_slots() {
        let font = Font::load().unwrap();
        let chars = glyph_set(['≈']);
        let atlas = rasterize(&font.0, 20.0, &chars, 1);
        assert!(atlas.cell.x >= 8 && atlas.cell.y >= atlas.cell.x);
        assert_eq!(atlas.index[&' '], 0);
        assert_eq!(atlas.size.x, ATLAS_COLUMNS * atlas.cell.x);
        // '#' has ink; ' ' has none.
        let slot_ink = |c: char| {
            (0..atlas.cell.y)
                .flat_map(|y| (0..atlas.cell.x).map(move |x| (x, y)))
                .map(|(x, y)| u32::from(texel(&atlas, c, x, y)))
                .sum::<u32>()
        };
        assert!(slot_ink('#') > 0);
        assert!(slot_ink('≈') > 0);
        assert_eq!(slot_ink(' '), 0);
    }

    #[test]
    fn supersampled_slots_downsample_to_the_crisp_atlas() {
        let font = Font::load().unwrap();
        let chars = glyph_set([]);
        let crisp = rasterize(&font.0, 18.0, &chars, 1);
        let smooth = rasterize(&font.0, 18.0, &chars, SUPERSAMPLE);
        assert_eq!(smooth.cell, crisp.cell * SUPERSAMPLE);
        assert_eq!(smooth.index, crisp.index);
        // A box filter over each supersampled block reproduces the 1:1 coverage up to rounding:
        // the outline sits in exactly the same place, so switching atlases never shifts a glyph.
        for c in ['@', '#', '_', '~', 'r'] {
            let worst = (0..crisp.cell.y)
                .flat_map(|y| (0..crisp.cell.x).map(move |x| (x, y)))
                .map(|(x, y)| {
                    let block: u32 = (0..SUPERSAMPLE * SUPERSAMPLE)
                        .map(|i| {
                            let (sx, sy) = (x * SUPERSAMPLE + i % SUPERSAMPLE, y * SUPERSAMPLE);
                            u32::from(texel(&smooth, c, sx, sy + i / SUPERSAMPLE))
                        })
                        .sum();
                    (block / (SUPERSAMPLE * SUPERSAMPLE))
                        .abs_diff(u32::from(texel(&crisp, c, x, y)))
                })
                .max();
            assert!(
                worst <= Some(2),
                "{c:?}: worst coverage difference {worst:?}"
            );
        }
    }

    #[test]
    fn glyph_set_has_no_duplicates() {
        let set = glyph_set(['#', 'é']);
        let unique: std::collections::BTreeSet<_> = set.iter().collect();
        assert_eq!(unique.len(), set.len());
        assert_eq!(set[0], ' ');
    }
}
