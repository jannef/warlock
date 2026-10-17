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
}

// Glyph flag: whole-pixel position at scale 1, so read the 1:1 atlas.
const CRISP: u32 = 1u;

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
    // Atlas slot and flags.
    @location(3) glyph: vec2<u32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) @interpolate(flat) glyph: vec2<u32>,
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
    return out;
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
    return vec4(in.color.rgb, in.color.a * coverage);
}
