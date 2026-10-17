// Console grid renderer (spec §4.2: the grid layer and the UI layer).
//
// A console is one quad. Each fragment finds its cell from its framebuffer position, looks up the
// cell's glyph, colors, and animation in small per-cell textures, and reads glyph coverage from the
// atlas with integer texel addressing — so still glyphs are drawn 1:1, crisp, with no filtering.
//
// In-cell animation (spec §4.3) is evaluated here from visual time: `wave` bobs the glyph inside
// its cell (sampling the supersampled atlas with linear filtering), `shimmer` moves the foreground
// toward a color, `cycle` steps through glyph frames. The curves mirror `render/anim.rs`.
//
// Cells nothing was drawn into are transparent (background alpha 0), so layers below show through.

import bevy_sprite_render::mesh2d::vertex_output::VertexOutput;

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
}

// One grid animation (`GridAnim` in anim.rs).
struct GridAnim {
    kind: u32,
    speed: f32,
    amount: f32,
    frame_count: u32,
    color: vec4<f32>,
    frames: vec4<u32>,
}

struct GridAnimTable {
    entries: array<GridAnim, 32>,
}

const KIND_WAVE: u32 = 1u;
const KIND_SHIMMER: u32 = 2u;
const KIND_CYCLE: u32 = 3u;
const TAU: f32 = 6.2831853;
const WAVE_SWAY: f32 = 0.35;

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

    let g = textureLoad(glyphs, cell, 0);
    var index = g.r | (g.g << 8u);
    var fg = textureLoad(fg_colors, cell, 0).rgb;
    let bg = textureLoad(bg_colors, cell, 0);
    let a = textureLoad(anims, cell, 0);

    var coverage = 0.0;
    if a.r == 0u {
        coverage = coverage_crisp(index, vec2<i32>(floor(local)));
    } else {
        let anim = anim_table.entries[a.r - 1u];
        let phase = f32(a.g) / 256.0;
        let t = params.time;
        if anim.kind == KIND_WAVE {
            let offset = wave_offset(t, anim.speed, anim.amount, phase) * params.cell_size.x;
            coverage = coverage_smooth(index, local - offset);
        } else {
            if anim.kind == KIND_SHIMMER {
                fg = mix(fg, anim.color.rgb, shimmer_mix(t, anim.speed, phase));
            } else if anim.kind == KIND_CYCLE {
                index = cycle_glyph(anim, t, phase);
            }
            coverage = coverage_crisp(index, vec2<i32>(floor(local)));
        }
    }
    return compose(fg, coverage, bg);
}
