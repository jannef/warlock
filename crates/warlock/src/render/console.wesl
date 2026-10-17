// Console grid renderer (spec §4.2, grid layer).
//
// The whole console is one quad. Each fragment finds its cell from its framebuffer position, looks
// up the cell's glyph and colors in small per-cell textures, and reads glyph coverage from the atlas
// with integer texel addressing — so glyphs are drawn 1:1, crisp, with no filtering.

import bevy_sprite_render::mesh2d::vertex_output::VertexOutput;

struct ConsoleParams {
    // Framebuffer position (physical pixels) of the console's top-left corner.
    origin: vec2<f32>,
    // Cell size in physical pixels.
    cell_size: vec2<f32>,
    // Console size in cells.
    grid_size: vec2<u32>,
    // Atlas slots per row.
    atlas_columns: u32,
    // Visual time in seconds (frozen by --freeze-time).
    time: f32,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> params: ConsoleParams;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var atlas: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var glyphs: texture_2d<u32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(3) var fg_colors: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(4) var bg_colors: texture_2d<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let px = in.position.xy - params.origin;
    let grid_px = vec2<f32>(params.grid_size) * params.cell_size;
    if px.x < 0.0 || px.y < 0.0 || px.x >= grid_px.x || px.y >= grid_px.y {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }
    let cell = vec2<i32>(floor(px / params.cell_size));
    let local = vec2<i32>(floor(px - vec2<f32>(cell) * params.cell_size));

    let g = textureLoad(glyphs, cell, 0);
    let index = g.r | (g.g << 8u);
    let slot = vec2<i32>(i32(index % params.atlas_columns), i32(index / params.atlas_columns));
    let texel = slot * vec2<i32>(params.cell_size) + local;
    let coverage = textureLoad(atlas, texel, 0).r;

    let fg = textureLoad(fg_colors, cell, 0).rgb;
    let bg = textureLoad(bg_colors, cell, 0).rgb;
    return vec4(mix(bg, fg, coverage), 1.0);
}
