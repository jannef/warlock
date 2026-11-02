//! The spell circles' rune atlas (spec §4.8 "Spell circles"): every rune of `data/runes.toml`
//! rasterised once from the rune font (`assets/fonts/NotoSansJP-Runes.ttf`, a subset of Noto
//! Sans JP Medium holding the runes alone; `assets/fonts/README.md`) into square slots; and the
//! circled actors' own glyphs ([`GlyphGlow`]), from the map font, the same way.
//!
//! A rune is drawn [`RUNE_SIZE`] cell heights square, turned to face its circle's centre, as it
//! orbits between pixels, so there is only a supersampled copy ([`SUPERSAMPLE`]×, linear): each
//! slot holds the ink centred in a square of `SUPERSAMPLE × ⌈size⌉` texels, padded on every side
//! for its halo and its glints, and a one-texel empty border, so filtering never reads a
//! neighbour. Four channels a texel: the coverage (red), the halo (green: the coverage blurred
//! by [`HALO_SIGMA`]), and two glints (blue and alpha: a tiny bright point with four short rays,
//! each at a place deep in a stroke, [`glint_points`]), which the shader brings in and out, each
//! on its own beat. Slot `i` is rune row `i` of [`Runes::rows`], in file order, at most
//! [`MAX_RUNES`]. The atlas is rebuilt when the cell height changes (the font size or the
//! display scale), never per frame.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use warlock_core::content::Runes;

const FONT_BYTES: &[u8] = include_bytes!("../../../../assets/fonts/NotoSansJP-Runes.ttf");

/// A rune's square, in cell heights (8 px at the default 18 px font).
pub const RUNE_SIZE: f32 = 0.44;

/// Texels per screen pixel of a slot.
pub const SUPERSAMPLE: u32 = 2;

/// Slots per atlas row.
pub const COLUMNS: u32 = 8;

/// The most runes the atlas holds (eight rows of [`COLUMNS`]), and the most glyphs
/// [`GlyphGlow`] holds.
pub const MAX_RUNES: usize = 64;
pub const MAX_GLYPHS: usize = 64;

/// The rune font, parsed once.
pub fn font() -> fontdue::Font {
    fontdue::Font::from_bytes(FONT_BYTES, fontdue::FontSettings::default())
        .expect("the bundled rune font is valid")
}

/// Rasterised slots in CPU memory, four bytes a texel: the coverage (red), its halo (green),
/// and two glints (blue, alpha).
#[derive(Debug, PartialEq)]
pub struct RuneBitmap {
    /// A slot's side in texels, its border included.
    pub slot: u32,
    /// The texels about a slot's square on each side: room for the halo and the glints' rays,
    /// and a one-texel empty border so filtering never reads a neighbour.
    pub pad: u32,
    pub size: UVec2,
    pub pixels: Vec<u8>,
}

/// The halo's blur, in rune squares (about a pixel for an 8 px rune).
pub const HALO_SIGMA: f32 = 0.12;
/// How much the halo's blurred coverage is raised (then clamped to 1).
const HALO_GAIN: f32 = 1.5;
/// A glint's point (its deviation) and its rays' length, in cell heights: half a pixel and two
/// at the default font.
pub const GLINT_SIGMA: f32 = 0.03;
pub const GLINT_RAY: f32 = 0.11;
/// A glyph's glints' rays, in cell heights.
pub const GLYPH_RAY: f32 = 0.2;
/// How bright a glint's rays are beside its point.
const RAY_GAIN: f32 = 0.8;

/// The sizes of a slot, in screen pixels: the font size its ink is rasterised at, its square,
/// the halo's deviation, and a glint's point and rays.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sizes {
    pub font_px: f32,
    pub square: f32,
    pub halo: f32,
    pub glint: f32,
    pub ray: f32,
}

impl Sizes {
    /// A rune's, for a cell height of `h` pixels.
    pub fn rune(h: f32) -> Self {
        let px = RUNE_SIZE * h;
        Sizes {
            font_px: px,
            square: px,
            halo: HALO_SIGMA * px,
            glint: GLINT_SIGMA * h,
            ray: GLINT_RAY * h,
        }
    }

    /// An actor's glyph's, at font size `px` in a cell `h` high: its square a cell height, the
    /// halo as a rune's, the glints' rays longer ([`GLYPH_RAY`]), so they reach past a glyph's
    /// stroke (a white glyph's own ink cannot be outshone).
    pub fn glyph(px: f32, h: f32) -> Self {
        Sizes {
            font_px: px,
            square: h,
            ray: GLYPH_RAY * h,
            ..Self::rune(h)
        }
    }
}

/// Rasterises `chars` (in order, at most [`MAX_RUNES`]) into slots of `sizes`.
pub fn rasterize(font: &fontdue::Font, chars: &[char], sizes: Sizes) -> RuneBitmap {
    let chars = &chars[..chars.len().min(MAX_RUNES)];
    let ss = SUPERSAMPLE as f32;
    let inner = sizes.square.ceil().max(1.0) as u32 * SUPERSAMPLE;
    let (halo, glint, ray) = (sizes.halo * ss, sizes.glint * ss, sizes.ray * ss);
    let pad = (3.0 * halo).max(ray + 2.0 * glint).ceil() as u32 + 1;
    let slot = inner + 2 * pad;
    let rows = (chars.len() as u32).div_ceil(COLUMNS).max(1);
    let size = UVec2::new(COLUMNS * slot, rows * slot);
    let mut pixels = vec![0u8; (size.x * size.y * 4) as usize];
    let square = pad as i32..(pad + inner) as i32;
    for (i, &c) in chars.iter().enumerate() {
        let (m, bitmap) = font.rasterize(c, sizes.font_px * ss);
        // The ink centred in the slot's square: a rune's place on the ring is its middle, and an
        // actor's circle is centred on its glyph's ink.
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
        let points = glint_points(&ink, &glow, slot, inner);
        let origin = UVec2::new(i as u32 % COLUMNS, i as u32 / COLUMNS) * slot;
        let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        for y in 1..slot - 1 {
            for x in 1..slot - 1 {
                let k = (y * slot + x) as usize;
                let at = (((origin.y + y) * size.x + origin.x + x) * 4) as usize;
                let p = Vec2::new(x as f32, y as f32);
                let spark = |q: Option<Vec2>| q.map_or(0.0, |q| sparkle(p - q, glint, ray));
                pixels[at] = byte(ink[k]);
                pixels[at + 1] = byte(glow[k] * HALO_GAIN);
                pixels[at + 2] = byte(spark(points[0]));
                pixels[at + 3] = byte(spark(points[1]));
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

/// Where a slot's two glints sit (texels): the deepest place in its strokes (the most ink with
/// the most ink about it), and the deepest a good way from the first (a third of the square),
/// if the ink reaches that far.
pub fn glint_points(ink: &[f32], glow: &[f32], side: u32, inner: u32) -> [Option<Vec2>; 2] {
    let at = |k: usize| Vec2::new((k as u32 % side) as f32, (k as u32 / side) as f32);
    let deepest = |far: &dyn Fn(Vec2) -> bool| {
        (0..ink.len())
            .filter(|&k| ink[k] > 0.6 && far(at(k)))
            .max_by(|&a, &b| (ink[a] * glow[a]).total_cmp(&(ink[b] * glow[b])))
            .map(at)
    };
    let first = deepest(&|_| true);
    let apart = inner as f32 / 3.0;
    let second = first.and_then(|f| deepest(&|p| p.distance(f) > apart));
    [first, second]
}

/// A glint at offset `d` from its point (texels): a point of deviation `sigma` and four short
/// rays, `ray` long, along the axes.
fn sparkle(d: Vec2, sigma: f32, ray: f32) -> f32 {
    let point = (-d.length_squared() / (2.0 * sigma * sigma)).exp();
    let thin = 0.6 * sigma;
    let arm = |along: f32, across: f32| {
        let fall = (1.0 - along.abs() / ray).max(0.0);
        RAY_GAIN * fall * (-(across * across) / (2.0 * thin * thin)).exp()
    };
    point.max(arm(d.x, d.y)).max(arm(d.y, d.x))
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

/// `bitmap` as a linearly sampled texture.
fn texture(bitmap: RuneBitmap, images: &mut Assets<Image>) -> Handle<Image> {
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
    images.add(image)
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
        let bitmap = rasterize(font, &chars, Sizes::rune(cell_height));
        let (slot, pad) = (bitmap.slot, bitmap.pad);
        Self {
            image: texture(bitmap, images),
            slot,
            pad,
            cell_height,
        }
    }
}

/// The circled actors' own glyphs (spec §4.8 "Spell circles": a circled actor's glyph glows
/// and glints as its runes do), rasterised from the map font as the runes are, a cell height
/// square: rebuilt when a glyph not in it is circled, or the font size changes.
#[derive(Resource, Debug)]
pub struct GlyphGlow {
    pub image: Handle<Image>,
    pub slot: u32,
    pub pad: u32,
    /// The glyphs held, by slot.
    pub chars: Vec<char>,
    /// The font size and cell height it was built for.
    pub px: f32,
    pub cell_height: f32,
}

impl GlyphGlow {
    pub fn new(
        font: &fontdue::Font,
        mut chars: Vec<char>,
        px: f32,
        cell_height: f32,
        images: &mut Assets<Image>,
    ) -> Self {
        chars.truncate(MAX_GLYPHS);
        let sizes = Sizes::glyph(px.max(1.0), cell_height.max(1.0));
        let bitmap = rasterize(font, &chars, sizes);
        let (slot, pad) = (bitmap.slot, bitmap.pad);
        Self {
            image: texture(bitmap, images),
            slot,
            pad,
            chars,
            px,
            cell_height,
        }
    }

    /// The slot of glyph `c`, if it is held.
    pub fn slot_of(&self, c: char) -> Option<u32> {
        self.chars.iter().position(|&g| g == c).map(|i| i as u32)
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
        // 0.44 of the default 18 px font: an 8 px rune.
        let sizes = Sizes::rune(18.0);
        let atlas = rasterize(&font(), &chars, sizes);
        let (pad, inner) = (atlas.pad, 8 * SUPERSAMPLE);
        assert_eq!(atlas.slot, inner + 2 * pad);
        assert!(pad as f32 > 3.0 * sizes.halo * SUPERSAMPLE as f32);
        assert!(pad as f32 > sizes.ray * SUPERSAMPLE as f32);
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
            // The halo reaches past the ink, beyond the square.
            assert!(sum(1) > sum(0), "rune {i}");
            let outside = |c: u32| {
                texels().any(|(x, y)| !(inside(x) && inside(y)) && channel(c, i, x, y) > 0)
            };
            assert!(!outside(0) && outside(1), "rune {i}");
            // The first glint is brightest on the ink, and small: a point and short rays.
            let peak = texels().max_by_key(|&(x, y)| channel(2, i, x, y)).unwrap();
            assert_eq!(channel(2, i, peak.0, peak.1), 255, "rune {i}");
            assert!(
                channel(0, i, peak.0, peak.1) > 150,
                "rune {i}'s glint on its ink"
            );
            assert!(sum(2) < sum(0), "rune {i}'s glint is small");
            for e in 0..atlas.slot {
                let last = atlas.slot - 1;
                for (x, y) in [(e, 0), (e, last), (0, e), (last, e)] {
                    for c in 0..4 {
                        assert_eq!(channel(c, i, x, y), 0, "rune {i}'s border is empty");
                    }
                }
            }
        }
    }

    #[test]
    fn glints_sit_deep_in_the_strokes_apart() {
        let font = font();
        let (m, bitmap) = font.rasterize('ネ', 32.0);
        let side = 48u32;
        let mut ink = vec![0.0; (side * side) as usize];
        for y in 0..m.height {
            for x in 0..m.width {
                ink[(y + 8) * side as usize + x + 8] = f32::from(bitmap[y * m.width + x]) / 255.0;
            }
        }
        let glow = blur(&ink, side, 2.0);
        let [a, b] = glint_points(&ink, &glow, side, 32);
        let (a, b) = (a.unwrap(), b.unwrap());
        assert!(a.distance(b) > 32.0 / 3.0);
        for p in [a, b] {
            assert!(ink[(p.y as u32 * side + p.x as u32) as usize] > 0.6);
        }
    }
}
