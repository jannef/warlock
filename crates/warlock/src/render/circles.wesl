// Spell circles (spec §4.8 "Spell circles", `circles.rs`): one quad per circled actor, all in one
// mesh and one draw. Every vertex of a quad carries the circle (flat): the cell height, the two
// circles' angles, the darkening, and each circle packed as `[head, runes 0-2, runes 3-5, 0]`.
//
// The fragment works in pixels from the actor's centre (`local`, y down): thin anti-aliased
// rings at signed distances, a regular polygon inscribed in the innermost ring with its corners
// at the rune slots, dashed for a hex along both, and an upright rune at each slot, a tap of the
// rune atlas. Sizes are in cell heights and mirror `circles.rs`: change both together.

import bevy_sprite_render::mesh2d::functions::{get_world_from_local, mesh2d_position_local_to_clip};
import bevy_sprite_render::mesh2d::view_bindings::view;

struct CircleParams {
    // Framebuffer rectangle outside of which nothing is drawn (the map viewport).
    clip_min: vec2<f32>,
    clip_max: vec2<f32>,
    // A rune atlas slot's side in texels (a one-texel empty border included), slots per row,
    // and texels per screen pixel.
    slot: u32,
    columns: u32,
    supersample: u32,
    // A rune's square, in cell heights.
    rune_size: f32,
    // The school colours, palette-mapped, linear (rgb), by `School::ALL` position.
    schools: array<vec4<f32>, 8>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> params: CircleParams;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var runes: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var runes_sampler: sampler;

const PI: f32 = 3.14159265;
const TAU: f32 = 6.2831853;

// Mirrors of `circles.rs` (cell heights).
const RING: f32 = 0.48;
const RING_GAP: f32 = 0.1;
const RUNE_RADIUS: f32 = 1.0;
const OUTER_RING: f32 = 1.35;
const OUTER_RUNE_RADIUS: f32 = 1.88;
// A hex's dashes along its rings and sides, and the gaps between them (cell heights).
const DASH: f32 = 0.2;
const GAP: f32 = 0.12;
// Opacity of the rings and sides, and of the runes.
const LINE_ALPHA: f32 = 0.2;
const RUNE_ALPHA: f32 = 0.7;
// The rings' and sides' width, in pixels at the default font and scale 1.
const LINE_WEIGHT: f32 = 1.0;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    // Framebuffer pixels (z unused).
    @location(0) position: vec3<f32>,
    // From the actor's centre, in pixels.
    @location(1) local: vec2<f32>,
    // Cell height, darkening.
    @location(2) shape: vec4<f32>,
    // The buffs' circle and the hexes': `[head, runes 0-2, runes 3-5, the old figure's head]`.
    @location(3) inner: vec4<u32>,
    @location(4) outer: vec4<u32>,
    // The old figures' runes: the buffs' two words, the hexes' two.
    @location(5) olds: vec4<u32>,
    // Each circle's angle, fade, blend (0 the old figure, 1 the new), and place (0 the inner
    // radii, 1 the outer).
    @location(6) inner_motion: vec4<f32>,
    @location(7) outer_motion: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) @interpolate(flat) shape: vec4<f32>,
    @location(2) @interpolate(flat) inner: vec4<u32>,
    @location(3) @interpolate(flat) outer: vec4<u32>,
    @location(4) @interpolate(flat) olds: vec4<u32>,
    @location(5) @interpolate(flat) inner_motion: vec4<f32>,
    @location(6) @interpolate(flat) outer_motion: vec4<f32>,
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
    out.shape = vertex.shape;
    out.inner = vertex.inner;
    out.outer = vertex.outer;
    out.olds = vertex.olds;
    out.inner_motion = vertex.inner_motion;
    out.outer_motion = vertex.outer_motion;
    return out;
}

// What one circle draws at a pixel: the lines' coverage and colour, and the runes'.
struct Ink {
    line: f32,
    line_rgb: vec3<f32>,
    rune: f32,
    rune_rgb: vec3<f32>,
}

// Coverage of a line of width `w` pixels at distance `d` from it, anti-aliased over a pixel.
fn stroke(d: f32, w: f32) -> f32 {
    return clamp(w * 0.5 + 0.5 - abs(d), 0.0, 1.0);
}

// Whether arc length `s` (pixels) falls on a dash, softened over a pixel at its ends.
fn dashed(s: f32, h: f32) -> f32 {
    let period = (DASH + GAP) * h;
    let t = s - floor(s / period) * period;
    let into = min(t, DASH * h - t);
    return clamp(into + 0.5, 0.0, 1.0);
}

// Rune `k` (0 to 5) of a circle's two rune words: its atlas slot and its school.
fn rune_of(words: vec2<u32>, k: u32) -> vec2<u32> {
    var word = words.x;
    var i = k;
    if k >= 3u {
        word = words.y;
        i = k - 3u;
    }
    let bits = (word >> (9u * i)) & 0x1ffu;
    return vec2(bits & 63u, bits >> 6u);
}

// Coverage of the rune in atlas slot `slot` at `q`, the pixel's place in the rune's square
// (0 to 1 across, y down).
fn rune_coverage(slot: u32, q: vec2<f32>) -> f32 {
    if any(q < vec2(0.0)) || any(q > vec2(1.0)) {
        return 0.0;
    }
    let s = f32(params.slot);
    let origin = vec2<f32>(f32(slot % params.columns), f32(slot / params.columns)) * s;
    // Inside the slot's one-texel border: filtering never reads a neighbour.
    let p = origin + vec2(1.0) + q * (s - 2.0);
    let size = vec2<f32>(textureDimensions(runes));
    return textureSampleLevel(runes, runes_sampler, p / size, 0.0).r;
}

// Where slot `k` stands on a figure changing from `was` slots to `to`, `blend` of the way, in
// radians from the first corner (`circles.rs::slot_angle`): from its old place to its new.
fn slot_angle(k: u32, was: u32, to: u32, blend: f32) -> f32 {
    let at_old = TAU * f32(k) / f32(max(was, 1u));
    let at_new = TAU * f32(k) / f32(max(to, 1u));
    return mix(at_old, at_new, blend);
}

// Coverage of the sides of a regular `m`-gon inscribed in a circle of radius `r0`, its first
// corner at angle `first`, at the pixel `d` from the centre at angle `theta`; dashed for a hex.
fn polygon(m: u32, d: f32, theta: f32, first: f32, r0: f32, w: f32, hex: bool, h: f32) -> f32 {
    if m < 3u {
        return 0.0;
    }
    let mf = f32(m);
    let sector = TAU / mf;
    var phi = theta - first + sector * 0.5;
    phi = phi - floor(phi / sector) * sector - sector * 0.5;
    let apothem = r0 * cos(PI / mf);
    var side = stroke(d * cos(phi) - apothem, w);
    if hex {
        let half_side = r0 * sin(PI / mf);
        side *= dashed(d * sin(phi) + half_side, h);
    }
    return side;
}

// One kind's circle at `p`: `c` its ring words, `old` the runes of the figure it changes from
// (whose head is `c.w`), `m` its angle, fade, blend, and place. Settled (blend 1, the old figure
// its own) it is the figure alone; changing, the rings and the polygon cross-fade and every rune
// glides from its old slot to its new one, changing its character on the way if it has to.
fn circle(c: vec4<u32>, old: vec2<u32>, m: vec4<f32>, p: vec2<f32>, h: f32) -> Ink {
    var ink = Ink(0.0, vec3(0.0), 0.0, vec3(0.0));
    let n = c.x & 15u;
    let fade = m.y;
    if n == 0u || fade <= 0.0 {
        return ink;
    }
    let angle = m.x;
    let t = m.z;
    let hex = (c.x & 16u) != 0u;
    let o = c.w;
    let on = max(o & 15u, 1u);
    let r0 = mix(RING, OUTER_RING, m.w) * h;
    let rr = mix(RUNE_RADIUS, OUTER_RUNE_RADIUS, m.w) * h;
    // One physical pixel at the default font and scale 1, more as the cells grow.
    let w = max(h / 18.0, 1.0) * LINE_WEIGHT;
    let d = length(p);
    // The pixel's angle on screen (y down: clockwise), from the circle's turned origin.
    let theta = atan2(p.y, p.x);
    let along = theta - angle;

    // Rings: one per effect, up to three (counted on the CPU, `Ring::rings`); one the old figure
    // had and the new has not (or the other way) fades as the figure changes.
    let rings = (c.x >> 16u) & 3u;
    let old_rings = (o >> 16u) & 3u;
    var line = 0.0;
    for (var k = 0u; k < max(rings, old_rings); k++) {
        let weight = mix(select(0.0, 1.0, k < old_rings), select(0.0, 1.0, k < rings), t);
        let r = r0 + RING_GAP * h * f32(k);
        var cov = stroke(d - r, w);
        if hex {
            let s = (along - floor(along / TAU) * TAU) * r;
            cov *= dashed(s, h);
        }
        line = max(line, cov * weight);
    }

    // The polygon: `slots` corners on the innermost ring (`Ring::slots`), the first at the top,
    // turned; while the count changes, the old polygon fades out as the new fades in.
    let slots = (c.x >> 12u) & 15u;
    let old_slots = (o >> 12u) & 15u;
    let first = angle - PI * 0.5;
    var side = polygon(slots, d, theta, first, r0, w, hex, h);
    if old_slots != slots {
        side = max(side * t, polygon(old_slots, d, theta, first, r0, w, hex, h) * (1.0 - t));
    }
    line = max(line, side);
    ink.line = line * fade;
    let line_rgb = params.schools[(c.x >> 8u) & 7u].rgb;
    let old_rgb = params.schools[(o >> 8u) & 7u].rgb;
    ink.line_rgb = mix(old_rgb, line_rgb, t);

    // The runes: upright squares about each slot's centre on the rune radius, each gliding from
    // its old slot's place to its new one's.
    let size = params.rune_size * h;
    let words = c.yz;
    for (var k = 0u; k < max(slots, old_slots); k++) {
        let a = first + slot_angle(k, old_slots, slots, t);
        let centre = rr * vec2(cos(a), sin(a));
        let q = (p - centre) / size + vec2(0.5);
        let has_new = k < slots;
        let has_old = k < old_slots;
        let rune = rune_of(words, k % n);
        let was = rune_of(old, k % on);
        var cov = 0.0;
        var rgb = vec3(0.0);
        if has_new && has_old && all(rune == was) {
            cov = rune_coverage(rune.x, q);
            rgb = params.schools[rune.y].rgb;
        } else {
            var cov_now = 0.0;
            if has_new {
                cov_now = rune_coverage(rune.x, q) * t;
            }
            var cov_then = 0.0;
            if has_old {
                cov_then = rune_coverage(was.x, q) * (1.0 - t);
            }
            cov = max(cov_now, cov_then);
            rgb = params.schools[select(was.y, rune.y, cov_now >= cov_then)].rgb;
        }
        if cov > ink.rune {
            ink.rune = cov;
            ink.rune_rgb = rgb;
        }
    }
    ink.rune *= fade;
    return ink;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let frag = in.position.xy;
    if any(frag < params.clip_min) || any(frag >= params.clip_max) {
        return vec4(0.0);
    }
    let h = in.shape.x;
    let a = circle(in.inner, in.olds.xy, in.inner_motion, in.local, h);
    let b = circle(in.outer, in.olds.zw, in.outer_motion, in.local, h);
    // The two circles never overlap once settled (the outer's rings clear the inner's runes):
    // take each part from the one that has the more of it.
    var line = a.line;
    var line_rgb = a.line_rgb;
    if b.line > line {
        line = b.line;
        line_rgb = b.line_rgb;
    }
    var rune = a.rune;
    var rune_rgb = a.rune_rgb;
    if b.rune > rune {
        rune = b.rune;
        rune_rgb = b.rune_rgb;
    }
    // A rune over the lines (it never crosses them, but it may touch a polygon's corner).
    let ra = RUNE_ALPHA * rune;
    let la = LINE_ALPHA * line * (1.0 - ra);
    let alpha = ra + la;
    if alpha <= 0.0 {
        return vec4(0.0);
    }
    let rgb = (rune_rgb * ra + line_rgb * la) / alpha;
    // A spell effect's darkening dims it as it dims free glyphs.
    return vec4(rgb * in.shape.y, alpha);
}
