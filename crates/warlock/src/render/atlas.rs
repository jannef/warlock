//! Glyph atlas: a monospace TTF rasterized at the exact physical pixel size (spec §4.1).
//!
//! Every glyph occupies one cell-sized slot, so the shader can address a glyph's texels with
//! integer math and draw it 1:1 without filtering.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::BTreeMap;

const FONT_BYTES: &[u8] = include_bytes!("../../../../assets/fonts/JuliaMono-Medium.ttf");

/// Slots per atlas row.
pub const ATLAS_COLUMNS: u32 = 32;

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
    pub cell: UVec2,
    pub size: UVec2,
    pub pixels: Vec<u8>,
    pub index: BTreeMap<char, u16>,
}

/// The atlas as used by rendering.
#[derive(Resource, Debug)]
pub struct GlyphAtlas {
    pub image: Handle<Image>,
    /// Cell size in physical pixels.
    pub cell: UVec2,
    /// Font size in physical pixels the atlas was built for.
    pub px: f32,
    index: BTreeMap<char, u16>,
    fallback: u16,
}

impl GlyphAtlas {
    /// Atlas slot for `c`; unknown characters render as `?`.
    pub fn glyph(&self, c: char) -> u16 {
        self.index.get(&c).copied().unwrap_or(self.fallback)
    }

    pub fn from_bitmap(bitmap: AtlasBitmap, px: f32, images: &mut Assets<Image>) -> Self {
        let fallback = bitmap.index.get(&'?').copied().unwrap_or(0);
        let image = Image::new(
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
        Self {
            image: images.add(image),
            cell: bitmap.cell,
            px,
            index: bitmap.index,
            fallback,
        }
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

/// Rasterizes `chars` at `px` physical pixels into cell-sized slots. Characters the font lacks
/// are left out, so they fall back to `?`.
pub fn rasterize(font: &fontdue::Font, px: f32, chars: &[char]) -> AtlasBitmap {
    let chars: Vec<char> = chars
        .iter()
        .copied()
        .filter(|&c| c == ' ' || font.lookup_glyph_index(c) != 0)
        .collect();
    let line = font.horizontal_line_metrics(px);
    let (ascent, descent) = line.map_or((px * 0.8, -px * 0.2), |l| (l.ascent, l.descent));
    let cell_w = font.metrics('M', px).advance_width.round().max(1.0) as u32;
    let cell_h = (ascent - descent).ceil().max(1.0) as u32;
    let baseline = ascent.round() as i32;

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

    #[test]
    fn rasterizes_into_cell_slots() {
        let font = Font::load().unwrap();
        let chars = glyph_set(['≈']);
        let atlas = rasterize(&font.0, 20.0, &chars);
        assert!(atlas.cell.x >= 8 && atlas.cell.y >= atlas.cell.x);
        assert_eq!(atlas.index[&' '], 0);
        assert_eq!(atlas.size.x, ATLAS_COLUMNS * atlas.cell.x);
        // '#' has ink; ' ' has none.
        let slot_ink = |c: char| {
            let slot = u32::from(atlas.index[&c]);
            let (ox, oy) = (
                slot % ATLAS_COLUMNS * atlas.cell.x,
                slot / ATLAS_COLUMNS * atlas.cell.y,
            );
            (0..atlas.cell.y)
                .flat_map(|y| (0..atlas.cell.x).map(move |x| (x, y)))
                .map(|(x, y)| u32::from(atlas.pixels[((oy + y) * atlas.size.x + ox + x) as usize]))
                .sum::<u32>()
        };
        assert!(slot_ink('#') > 0);
        assert!(slot_ink('≈') > 0);
        assert_eq!(slot_ink(' '), 0);
    }

    #[test]
    fn glyph_set_has_no_duplicates() {
        let set = glyph_set(['#', 'é']);
        let unique: std::collections::BTreeSet<_> = set.iter().collect();
        assert_eq!(unique.len(), set.len());
        assert_eq!(set[0], ' ');
    }
}
