// Console grid renderer (spec §4.2: the grid layer and the UI layer).
//
// A console is one quad. Each fragment finds its cell from its framebuffer position, looks up the
// cell's glyph, colors, and animation in small per-cell textures, and reads glyph coverage from the
// atlas with integer texel addressing — so still glyphs are drawn 1:1, crisp, with no filtering.
//
// In-cell animation (spec §4.3) is evaluated here from visual time: `wave` bobs the glyph inside
// its cell (sampling the supersampled atlas with linear filtering), `flow` moves it downstream
// along the cell's flow (running water), `shimmer` moves the foreground toward a color, `cycle`
// steps through glyph frames. Water may also glint in the color of the light on it. The curves
// mirror `render/anim.rs` and `render/water.rs`.
//
// Per-cell textures: `glyphs` (atlas slot, low and high byte; then the cell's flow, `flow_byte` in
// water.rs), `fg_colors`, `bg_colors`, and `anims` (animation table entry plus one, phase, glow,
// haze).
//
// Effects (spec §4.8): `glow` brightens a glyph past 1 so bloom picks it up, and heat haze
// distorts glyphs over hot cells by shifting where their coverage is read. Both come from per-cell
// bytes next to the animation data, and are off (0) on the UI console and with reduced effects.
//
// Cells nothing was drawn into are transparent (background alpha 0), so layers below show through.
//
// Visual lighting (spec §4.4) lights the map grid's terrain per pixel; see `lighting.wesl`.

import bevy_sprite_render::mesh2d::vertex_output::VertexOutput;
import super::lighting::{light_at, shade};

struct ConsoleParams {
    // Framebuffer position (physical pixels) of cell (0, 0)'s top-left corner.
    origin: vec2<f32>,
    // Cell size in physical pixels.
    cell_size: vec2<f32>,
    // Framebuffer rectangle outside of which the console draws nothing.
    clip_min: vec2<f32>,
    clip_max: vec2<f32>,
    // Console size in cells.
    grid_size: vec2<u32>,
    // Atlas slots per row.
    atlas_columns: u32,
    // Linear size of the smooth atlas relative to the 1:1 atlas.
    supersample: u32,
    // Visual time in seconds (frozen by --freeze-time).
    time: f32,
    // HDR gain of glowing glyphs; 0 turns glow off.
    glow_gain: f32,
    // Largest heat-haze displacement in pixels; 0 turns haze off.
    haze_px: f32,
    // Level position of cell (0, 0) (map grid).
    level_origin: vec2<i32>,
}

// One grid animation (`GridAnim` in anim.rs).
struct GridAnim {
    kind: u32,
    speed: f32,
    amount: f32,
    frame_count: u32,
    color: vec4<f32>,
    frames: vec4<u32>,
    glint: f32,
}

struct GridAnimTable {
    entries: array<GridAnim, 32>,
}

const KIND_WAVE: u32 = 1u;
const KIND_SHIMMER: u32 = 2u;
const KIND_CYCLE: u32 = 3u;
const KIND_FLOW: u32 = 4u;
const PI: f32 = 3.14159265;
const TAU: f32 = 6.2831853;
const WAVE_SWAY: f32 = 0.35;
// Constants of `glint` and `flow_byte` in water.rs.
const GLINT_PERIOD: f32 = 9.0;
const GLINT_CHANCE: f32 = 0.4;
const GLINT_LENGTH: f32 = 0.9;
const FLOW_SPEED_STEPS: f32 = 8.0;
// How much a full glint adds to the glyph, times the (modulated) light on it.
const GLINT_GAIN: f32 = 0.5;

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> params: ConsoleParams;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var atlas: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var glyphs: texture_2d<u32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(3) var fg_colors: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(4) var bg_colors: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(5) var anims: texture_2d<u32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(6) var smooth_atlas: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(7) var smooth_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(8) var<uniform> anim_table: GridAnimTable;

fn slot_of(index: u32) -> vec2<u32> {
    return vec2<u32>(index % params.atlas_columns, index / params.atlas_columns);
}

// Coverage of glyph `index` at whole-pixel position `local` inside the cell (1:1 atlas).
fn coverage_crisp(index: u32, local: vec2<i32>) -> f32 {
    let texel = vec2<i32>(slot_of(index)) * vec2<i32>(params.cell_size) + local;
    return textureLoad(atlas, texel, 0).r;
}

// Coverage of glyph `index` at sub-pixel position `local` inside the cell (smooth atlas). Ink
// moved outside the cell is cut off, so the glyph stays inside its cell.
fn coverage_smooth(index: u32, local: vec2<f32>) -> f32 {
    if any(local < vec2(0.0)) || any(local >= params.cell_size) {
        return 0.0;
    }
    let ss = f32(params.supersample);
    let slot = vec2<f32>(slot_of(index)) * params.cell_size * ss;
    // Stay half a texel inside the slot so filtering never reads a neighbor.
    let p = clamp(local * ss, vec2(0.5), params.cell_size * ss - vec2(0.5));
    let size = vec2<f32>(textureDimensions(smooth_atlas));
    return textureSampleLevel(smooth_atlas, smooth_sampler, (slot + p) / size, 0.0).r;
}

// `wave_offset` in anim.rs: a bob with a smaller sideways sway, in cell widths.
fn wave_offset(t: f32, speed: f32, amount: f32, phase: f32) -> vec2<f32> {
    let a = TAU * (t * speed - phase);
    return vec2(WAVE_SWAY * cos(a), sin(a)) * amount;
}

// `shimmer_mix` in anim.rs.
fn shimmer_mix(t: f32, speed: f32, phase: f32) -> f32 {
    return 0.5 - 0.5 * cos(TAU * (t * speed + phase));
}

// `cycle_frame` in anim.rs, returning the frame's atlas slot.
fn cycle_glyph(anim: GridAnim, t: f32, phase: f32) -> u32 {
    let n = i32(max(anim.frame_count, 1u));
    let step = i32(floor(t * anim.speed + phase * f32(n)));
    let i = u32(((step % n) + n) % n);
    return (anim.frames[i / 2u] >> ((i % 2u) * 16u)) & 0xffffu;
}

// --- Water (spec §4.3) -------------------------------------------------------------------------

// `scramble` in anim.rs (MurmurHash3 finalizer).
fn scramble(x: u32) -> u32 {
    var h = x;
    h ^= h >> 16u;
    h *= 0x85ebca6bu;
    h ^= h >> 13u;
    h *= 0xc2b2ae35u;
    return h ^ (h >> 16u);
}

// `cell_hash` in anim.rs.
fn cell_hash(p: vec2<i32>) -> u32 {
    return scramble((bitcast<u32>(p.x) * 0x8da6b343u) ^ (bitcast<u32>(p.y) * 0xd8163841u));
}

// `hash_phase` in anim.rs.
fn hash_phase(h: u32) -> f32 {
    return f32(h >> 8u) / 16777216.0;
}

// `glint` in water.rs: how bright a glint is on the cell with hash `h`, 0 to 1.
fn glint(t: f32, h: u32) -> f32 {
    let period = GLINT_PERIOD * (0.75 + 0.5 * hash_phase(h));
    let cycles = t / period + hash_phase(scramble(h ^ 0x68e31da4u));
    let n = floor(cycles);
    let roll = scramble(h ^ (u32(n) * 0x9e3779b9u));
    if hash_phase(roll) >= GLINT_CHANCE {
        return 0.0;
    }
    let start = hash_phase(scramble(roll)) * (period - GLINT_LENGTH);
    let x = ((cycles - n) * period - start) / GLINT_LENGTH;
    if x < 0.0 || x >= 1.0 {
        return 0.0;
    }
    let s = sin(PI * x);
    return s * s;
}

// The step (each axis -1, 0, or 1; y down) of a flow byte's direction: `Dir::ALL` runs
// clockwise from N in steps of 45 degrees (`unpack_flow` in water.rs).
fn flow_step(code: u32) -> vec2<f32> {
    let a = f32(code & 7u) * (PI / 4.0);
    return round(vec2(sin(a), -cos(a)));
}

// `flow_shift` in water.rs: how far the glyph pattern has moved, in pixels within one cell.
fn flow_shift(t: f32, speed: f32, step: vec2<f32>, cell: vec2<f32>) -> vec2<f32> {
    let cycles = vec2(t * speed) * cell.x / cell;
    return step * (cycles - floor(cycles)) * cell;
}

// Coverage of a running-water glyph: the glyph repeats every cell and the pattern moves
// downstream, so ripples leave the cell on one side and come back in on the other, and
// neighbors with the same flow line up into one moving surface.
fn flow_coverage(index: u32, local: vec2<f32>, anim: GridAnim, code: u32) -> f32 {
    let speed = anim.speed * f32(code >> 3u) / FLOW_SPEED_STEPS;
    if speed <= 0.0 {
        return coverage_smooth(index, local);
    }
    let p = local - flow_shift(params.time, speed, flow_step(code), params.cell_size);
    return coverage_smooth(index, p - floor(p / params.cell_size) * params.cell_size);
}

// -----------------------------------------------------------------------------------------------

// Coverage of glyph `index` at `local`: 1:1 unless the pixel was moved off the pixel grid.
fn coverage_at(index: u32, local: vec2<f32>, smooth_sample: bool) -> f32 {
    if smooth_sample {
        return coverage_smooth(index, local);
    }
    return coverage_crisp(index, vec2<i32>(floor(local)));
}

// --- Heat haze (spec §4.8) ---------------------------------------------------------------------

// Haze strength of a cell (0 to 1), from the per-cell effect bytes.
fn cell_haze(cell: vec2<i32>) -> f32 {
    let max_cell = vec2<i32>(params.grid_size) - vec2(1);
    return f32(textureLoad(anims, clamp(cell, vec2(0), max_cell), 0).a) / 255.0;
}

// Haze strength at grid pixel `px`, interpolated between cell centers so it has no seams.
fn haze_at(px: vec2<f32>) -> f32 {
    if params.haze_px <= 0.0 {
        return 0.0;
    }
    let c = px / params.cell_size - vec2(0.5);
    let base = vec2<i32>(floor(c));
    let f = c - floor(c);
    let top = mix(cell_haze(base), cell_haze(base + vec2(1, 0)), f.x);
    let bottom = mix(cell_haze(base + vec2(0, 1)), cell_haze(base + vec2(1, 1)), f.x);
    return mix(top, bottom, f.y);
}

// Haze displacement at `p` (in cells) at time `t`, at most about 1: mostly sideways ripples that
// rise, like air over a fire. A glyph pixel at `p` shows the glyph moved by minus this.
fn haze_offset(p: vec2<f32>, t: f32) -> vec2<f32> {
    let rise = TAU * (p.y * 1.4 + t * 0.9) + 1.7 * sin(TAU * (p.x * 0.6 + t * 0.3));
    let x = 0.7 * sin(rise) + 0.3 * sin(TAU * (p.y * 3.1 + p.x * 0.8 + t * 1.9));
    let y = 0.35 * sin(TAU * (p.x * 1.2 - p.y * 0.5 + t * 0.7));
    return vec2(x, y);
}

// -----------------------------------------------------------------------------------------------

// Glyph over background over whatever is below (non-premultiplied output for alpha blending).
fn compose(fg: vec3<f32>, coverage: f32, bg: vec4<f32>) -> vec4<f32> {
    let alpha = coverage + bg.a * (1.0 - coverage);
    if alpha <= 0.0 {
        return vec4(0.0);
    }
    return vec4((fg * coverage + bg.rgb * bg.a * (1.0 - coverage)) / alpha, alpha);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let frag = in.position.xy;
    if any(frag < params.clip_min) || any(frag >= params.clip_max) {
        return vec4(0.0);
    }
    let px = frag - params.origin;
    let grid_px = vec2<f32>(params.grid_size) * params.cell_size;
    if any(px < vec2(0.0)) || any(px >= grid_px) {
        return vec4(0.0);
    }
    let cell = vec2<i32>(floor(px / params.cell_size));
    let local = px - vec2<f32>(cell) * params.cell_size;
    // Heat haze: glyphs read their coverage from a slightly moved position (sub-pixel, so sampled
    // smoothly). Backgrounds and light stay put, so cell edges stay straight.
    let haze = haze_at(px);
    let hazed = haze > 0.0;
    var glyph_local = local;
    if hazed {
        glyph_local += haze_offset(px / params.cell_size, params.time) * haze * params.haze_px;
    }

    let g = textureLoad(glyphs, cell, 0);
    var index = g.r | (g.g << 8u);
    var fg = textureLoad(fg_colors, cell, 0).rgb;
    var bg = textureLoad(bg_colors, cell, 0);
    let a = textureLoad(anims, cell, 0);

    var coverage = 0.0;
    var glint_strength = 0.0;
    if a.r == 0u {
        coverage = coverage_at(index, glyph_local, hazed);
    } else {
        let anim = anim_table.entries[a.r - 1u];
        let phase = f32(a.g) / 256.0;
        let t = params.time;
        if anim.glint > 0.0 {
            glint_strength = anim.glint * glint(t, cell_hash(cell + params.level_origin));
        }
        if anim.kind == KIND_WAVE {
            let offset = wave_offset(t, anim.speed, anim.amount, phase) * params.cell_size.x;
            coverage = coverage_smooth(index, glyph_local - offset);
        } else if anim.kind == KIND_FLOW {
            coverage = flow_coverage(index, glyph_local, anim, g.b);
        } else {
            if anim.kind == KIND_SHIMMER {
                fg = mix(fg, anim.color.rgb, shimmer_mix(t, anim.speed, phase));
            } else if anim.kind == KIND_CYCLE {
                index = cycle_glyph(anim, t, phase);
            }
            coverage = coverage_at(index, glyph_local, hazed);
        }
    }
    // Visual light, sampled where the pixel is (not where haze moved the glyph from).
    let light = light_at(cell, local / params.cell_size);
    if light.lit {
        fg = shade(fg, light);
        bg = vec4(shade(bg.rgb, light), bg.a);
        // A glint reflects the light on the cell: its color (hue drift included) and intensity
        // gain (light stability); none in the dark.
        fg += glint_strength * GLINT_GAIN * min(light.rgb, vec3(1.0)) * light.gain;
    }
    // Effects: lit colors stay within display range, so bright light alone never blooms; then
    // glowing glyphs (not their backgrounds) go past 1, and bloom puts a halo around them.
    if params.glow_gain > 0.0 {
        fg = min(fg, vec3(1.0)) * (1.0 + f32(a.b) / 255.0 * params.glow_gain);
        bg = vec4(min(bg.rgb, vec3(1.0)), bg.a);
    }
    return compose(fg, coverage, bg);
}
