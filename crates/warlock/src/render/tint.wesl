// Screen tint overlay (spec §4.2, post-processing): one color over the map viewport, for the fade
// on level change and flashes. The quad covers the screen; pixels outside the clip rectangle are
// left untouched. Hidden (not drawn) while no tint is running.

import bevy_sprite_render::mesh2d::vertex_output::VertexOutput;

struct TintParams {
    // Framebuffer rectangle the tint covers (the map viewport).
    clip_min: vec2<f32>,
    clip_max: vec2<f32>,
    // Linear RGB and opacity.
    color: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> params: TintParams;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let frag = in.position.xy;
    if any(frag < params.clip_min) || any(frag >= params.clip_max) {
        return vec4(0.0);
    }
    return params.color;
}
