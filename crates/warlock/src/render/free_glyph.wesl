// Free-glyph layer (spec §4.2, layer 2): one quad per glyph, all in one mesh and one draw.
//
// Vertex positions are framebuffer pixels; the vertex shader maps them straight to clip space, so
// glyphs line up with the console grids pixel for pixel. A glyph at rest (whole-pixel position,
// scale 1) reads the 1:1 atlas with integer texel addressing, exactly like the grid; a moving or
// scaled glyph samples the supersampled atlas with linear filtering.

import bevy_sprite_render::mesh2d::functions::{get_world_from_local, mesh2d_position_local_to_clip};
import bevy_sprite_render::mesh2d::view_bindings::view;

struct FreeGlyphParams {
    // Framebuffer rectangle outside of which nothing is drawn.
    clip_min: vec2<f32>,
    clip_max: vec2<f32>,
    // Cell (and 1:1 atlas slot) size in physical pixels.
    cell_size: vec2<f32>,
    atlas_columns: u32,
    supersample: u32,
    // The rows of a cell a capital letter covers, top and bottom: where colour bands lie.
    ink: vec2<f32>,
}

// Glyph flag: whole-pixel position at scale 1, so read the 1:1 atlas.
const CRISP: u32 = 1u;

// The width of the soft border between two colour bands, in band heights.
const SEAM: f32 = 0.35;

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> params: FreeGlyphParams;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var atlas: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var smooth_atlas: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(3) var smooth_sampler: sampler;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    // Framebuffer pixels (z unused).
    @location(0) position: vec3<f32>,
    // Position inside the glyph's slot, in 1:1 atlas pixels.
    @location(1) local: vec2<f32>,
    // Linear RGB and alpha.
    @location(2) color: vec4<f32>,
    // Atlas slot, flags, band count, and band drift (f32 bits).
    @location(3) glyph: vec4<u32>,
    // A banded glyph's second, third, and fourth colours (linear RGB).
    @location(4) band1: vec4<f32>,
    @location(5) band2: vec4<f32>,
    @location(6) band3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) @interpolate(flat) glyph: vec4<u32>,
    @location(3) @interpolate(flat) band1: vec3<f32>,
    @location(4) @interpolate(flat) band2: vec3<f32>,
    @location(5) @interpolate(flat) band3: vec3<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    // Depth (draw order) comes from the entity's transform, like any other 2D mesh.
    let anchor = mesh2d_position_local_to_clip(
        get_world_from_local(vertex.instance_index),
        vec4(0.0, 0.0, 0.0, 1.0),
    );
    let uv = (vertex.position.xy - view.viewport.xy) / view.viewport.zw;
    let ndc = vec2(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
    var out: VertexOutput;
    out.position = vec4(ndc * anchor.w, anchor.z, anchor.w);
    out.local = vertex.local;
    out.color = vertex.color;
    out.glyph = vertex.glyph;
    out.band1 = vertex.band1.rgb;
    out.band2 = vertex.band2.rgb;
    out.band3 = vertex.band3.rgb;
    return out;
}

// Band `i` of a banded glyph (0 is the glyph's own colour).
fn band(in: VertexOutput, i: u32) -> vec3<f32> {
    switch i {
        case 0u: { return in.color.rgb; }
        case 1u: { return in.band1; }
        case 2u: { return in.band2; }
        default: { return in.band3; }
    }
}

// The glyph's colour at this pixel: its own, or for a composite (spec §8 "Animated objects")
// horizontal bands of its colours over the rows a capital letter covers, top first, with soft
// borders that the drift moves up and down (the top and bottom bands always stay).
fn banded(in: VertexOutput) -> vec3<f32> {
    let n = in.glyph.z;
    if n < 2u {
        return in.color.rgb;
    }
    let span = max(params.ink.y - params.ink.x, 1.0);
    let t = clamp((in.local.y - params.ink.x) / span, 0.0, 1.0);
    // Band coordinates, centred: band k's middle is at k.
    let x = t * f32(n) + bitcast<f32>(in.glyph.w) - 0.5;
    let i = floor(x);
    let last = f32(n - 1u);
    let above = u32(clamp(i, 0.0, last));
    let below = u32(clamp(i + 1.0, 0.0, last));
    let w = smoothstep(0.5 - SEAM * 0.5, 0.5 + SEAM * 0.5, x - i);
    return mix(band(in, above), band(in, below), w);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let frag = in.position.xy;
    if any(frag < params.clip_min) || any(frag >= params.clip_max) {
        return vec4(0.0);
    }
    let index = in.glyph.x;
    let slot = vec2<f32>(f32(index % params.atlas_columns), f32(index / params.atlas_columns));
    var coverage: f32;
    if (in.glyph.y & CRISP) != 0u {
        let local = clamp(vec2<i32>(floor(in.local)), vec2(0), vec2<i32>(params.cell_size) - 1);
        coverage = textureLoad(atlas, vec2<i32>(slot * params.cell_size) + local, 0).r;
    } else {
        let ss = f32(params.supersample);
        // Stay half a texel inside the slot so filtering never reads a neighbor.
        let p = clamp(in.local * ss, vec2(0.5), params.cell_size * ss - vec2(0.5));
        let size = vec2<f32>(textureDimensions(smooth_atlas));
        coverage = textureSampleLevel(smooth_atlas, smooth_sampler, (slot * params.cell_size * ss + p) / size, 0.0).r;
    }
    return vec4(banded(in), in.color.a * coverage);
}
