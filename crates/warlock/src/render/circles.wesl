// Spell circles (spec §4.8 "Spell circles", `circles.rs`): one quad per circled actor, all in one
// mesh and one draw. Every vertex of a quad carries the circle (flat): the cell height, the
// darkening, the actor's cell, and each circle's head and motion.
//
// The fragment works in pixels from the actor's centre (`local`, y down): thin anti-aliased
// rings at signed distances with a faint glow about them and a soft arc running round each, and
// a regular polygon inscribed in the innermost ring, dashed for a hex along both. Sizes are in
// cell heights and mirror `circles.rs`: change both together.
//
// The actor's own glyph glows too (the glyph atlas, `glyph_glow.rs::GlyphGlow`): its halo here,
// under the free-glyph layer, and its glints in a second pass over it (`params.top`).
//
// The map's per-cell flags (`light_flags`, `lighting.rs`) keep the circle to what the player
// sees: a line fades by the cell under its pixel, softly across cell edges; nothing over unseen or
// remembered cells or walls, nothing over a cell holding another glyph (a creature, an object, a
// torch), the actor's own cell left out.

import bevy_sprite_render::mesh2d::functions::{get_world_from_local, mesh2d_position_local_to_clip};
import bevy_sprite_render::mesh2d::view_bindings::view;

struct CircleParams {
    // Framebuffer rectangle outside of which nothing is drawn (the map viewport).
    clip_min: vec2<f32>,
    clip_max: vec2<f32>,
    // Framebuffer position of the map grid's cell (0, 0), and the cell size, in pixels.
    grid_origin: vec2<f32>,
    cell_size: vec2<f32>,
    // The glyph atlas's slot side and padding (texels), slots per row, and its square (pixels).
    glyph_slot: u32,
    glyph_pad: u32,
    columns: u32,
    glyph_square: f32,
    // The glows' strength (0 or 1): the lines', the glyph's; and the glints'.
    halo: f32,
    // 1 with the `glow` switch on (glints bloom), else 0.
    bloom: f32,
    // 1: the pass over the free-glyph layer, the actors' glyphs' glints alone.
    top: u32,
    // The school colours, palette-mapped, linear (rgb), by `School::ALL` position.
    schools: array<vec4<f32>, 8>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> params: CircleParams;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var light_flags: texture_2d<u32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var glyphs: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(3) var glyphs_sampler: sampler;

const PI: f32 = 3.14159265;
const TAU: f32 = 6.2831853;

// Mirrors of `circles.rs` (cell heights).
const RING: f32 = 0.48;
const RING_GAP: f32 = 0.1;
const OUTER_RING: f32 = 0.84;
// A hex's dashes along its rings and sides, and the gaps between them (cell heights).
const DASH: f32 = 0.2;
const GAP: f32 = 0.12;
// Opacity of the rings and sides.
const LINE_ALPHA: f32 = 0.2;
// The rings' and sides' width, in pixels at the default font and scale 1.
const LINE_WEIGHT: f32 = 1.0;
// The lines' glow, in their colour: its opacity on the line, and its deviation (cell heights,
// `circles.rs::LINE_GLOW_SIGMA`).
const LINE_HALO_ALPHA: f32 = 0.1;
const LINE_HALO_SIGMA: f32 = 0.045;
// The actor's glyph's halo.
const GLYPH_HALO_ALPHA: f32 = 0.6;
// The shimmer running round the rings: how much brighter at its middle, and its period.
const SHIMMER: f32 = 1.5;
const SHIMMER_PERIOD: f32 = 7.0;
// `circles.rs`: the glints' periods and sharpness, their opacity, how far toward white, and how
// overbright with the `glow` switch on.
const GLINT_PERIOD_A: f32 = 5.3;
const GLINT_PERIOD_B: f32 = 7.1;
const GLINT_SHARP: f32 = 24.0;
const GLINT_ALPHA: f32 = 0.9;
const GLINT_WHITE: f32 = 0.75;
const GLINT_BLOOM: f32 = 1.5;
// The glyph slot that means none (`circles.rs::NO_GLYPH`).
const NO_GLYPH: u32 = 255u;
// `lighting.rs`'s cell flags, and the margin of its textures (cells).
const LIT: u32 = 1u;
const OPAQUE: u32 = 2u;
const OCCUPIED: u32 = 4u;
const MARGIN: i32 = 1;
// Half the width of the soft edge between cells, in cells, either side of the edge.
const SOFT: f32 = 0.22;
// How far the lines stay off the actor's glyph's ink box when its glyph is not in the atlas, in
// pixels at the default font.
const GLYPH_GAP: f32 = 1.0;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    // Framebuffer pixels (z unused).
    @location(0) position: vec3<f32>,
    // From the actor's centre, in pixels.
    @location(1) local: vec2<f32>,
    // Cell height, darkening, and the actor's map grid cell.
    @location(2) shape: vec4<f32>,
    // The buffs' circle and the hexes': `[head, the old figure's head]`.
    @location(3) inner: vec2<u32>,
    @location(4) outer: vec2<u32>,
    // Each circle's angle, fade, blend (0 the old figure, 1 the new), and place (0 the inner
    // radii, 1 the outer).
    @location(5) inner_motion: vec4<f32>,
    @location(6) outer_motion: vec4<f32>,
    // Half the size of the actor's glyph's ink (pixels), the visual time, and motion (0 or 1).
    @location(7) ink: vec4<f32>,
    // The earlier circle this one passes under (`circles.rs::under`): its centre from this
    // one's and its outermost ring's radius, in pixels (radius 0: none); then the actor's glyph's
    // slot and seed (`circles.rs::pack_glyph`).
    @location(8) under: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) @interpolate(flat) shape: vec4<f32>,
    @location(2) @interpolate(flat) inner: vec2<u32>,
    @location(3) @interpolate(flat) outer: vec2<u32>,
    @location(4) @interpolate(flat) inner_motion: vec4<f32>,
    @location(5) @interpolate(flat) outer_motion: vec4<f32>,
    @location(6) @interpolate(flat) ink: vec4<f32>,
    @location(7) @interpolate(flat) under: vec4<f32>,
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
    out.inner_motion = vertex.inner_motion;
    out.outer_motion = vertex.outer_motion;
    out.ink = vertex.ink;
    out.under = vertex.under;
    return out;
}

// What one circle draws at a pixel: the lines' coverage, their glow's, and their colour.
struct Ink {
    line: f32,
    glow: f32,
    rgb: vec3<f32>,
}

// Coverage of a line of width `w` pixels at distance `d` from it, anti-aliased over a pixel.
fn stroke(d: f32, w: f32) -> f32 {
    return clamp(w * 0.5 + 0.5 - abs(d), 0.0, 1.0);
}

// A line's glow at distance `d` from it, deviation `s` pixels: 1 on it.
fn glow_at(d: f32, s: f32) -> f32 {
    return exp(-d * d / (2.0 * s * s));
}

// Whether arc length `s` (pixels) falls on a dash, softened over a pixel at its ends.
fn dashed(s: f32, h: f32) -> f32 {
    let period = (DASH + GAP) * h;
    let t = s - floor(s / period) * period;
    let into = min(t, DASH * h - t);
    return clamp(into + 0.5, 0.0, 1.0);
}

// The glyph in atlas slot `slot` at `q`, the pixel's place in its square (0 to 1 across, y
// down; its halo and glints reach into the padding about it): coverage, halo, two glints.
fn glyph_texel(slot: u32, q: vec2<f32>) -> vec4<f32> {
    let s = f32(params.glyph_slot);
    let pad = f32(params.glyph_pad);
    let inner = s - 2.0 * pad;
    // Inside the slot's one-texel border: filtering never reads a neighbour.
    let t = pad + q * inner;
    if any(t < vec2(1.0)) || any(t > vec2(s - 1.0)) {
        return vec4(0.0);
    }
    let origin = vec2<f32>(f32(slot % params.columns), f32(slot / params.columns)) * s;
    let size = vec2<f32>(textureDimensions(glyphs));
    return textureSampleLevel(glyphs, glyphs_sampler, (origin + t) / size, 0.0);
}

// How much of the circle shows over map grid cell `cell`: none where the player does not see
// it now (unseen, remembered, outside the view), where it is a wall, or where another glyph is
// (`OCCUPIED`); all of it elsewhere and on the actor's own cell `own`.
fn show_cell(cell: vec2<i32>, own: vec2<i32>) -> f32 {
    let t = cell + vec2(MARGIN);
    let size = vec2<i32>(textureDimensions(light_flags));
    if any(t < vec2(0)) || any(t >= size) {
        return 0.0;
    }
    let flags = textureLoad(light_flags, t, 0).r;
    if (flags & (LIT | OPAQUE)) != LIT {
        return 0.0;
    }
    if (flags & OCCUPIED) != 0u && any(cell != own) {
        return 0.0;
    }
    return 1.0;
}

// `show_cell` at framebuffer position `frag`, eased from cell to cell over `SOFT` either side
// of their edge: a soft edge, never a cut.
fn show_at(frag: vec2<f32>, own: vec2<i32>) -> f32 {
    let c = (frag - params.grid_origin) / params.cell_size - vec2(0.5);
    let base = vec2<i32>(floor(c));
    let f = smoothstep(vec2(0.5 - SOFT), vec2(0.5 + SOFT), c - floor(c));
    let top = mix(show_cell(base, own), show_cell(base + vec2(1, 0), own), f.x);
    let bottom = mix(show_cell(base + vec2(0, 1), own), show_cell(base + vec2(1, 1), own), f.x);
    return mix(top, bottom, f.y);
}

// `circles.rs::glint_phase` and `glint`: glint `g` of an actor's (`seed`) glyph, and how bright
// it is at the frame's time (`clock`: the visual time, and 1 with motion on): a moment's
// sparkle, then gone. None without motion or with reduced effects.
fn glint_phase(seed: u32, g: u32) -> f32 {
    return fract(f32(seed) * 0.137036 + f32(g) * 0.5);
}

fn glint(phase: f32, period: f32, clock: vec2<f32>) -> f32 {
    let s = max(sin(TAU * (clock.x / period + phase)), 0.0);
    return pow(s, GLINT_SHARP) * clock.y * params.halo;
}

// A glint's colour: the glyph's glow's school colour most of the way to white, and overbright
// with the `glow` switch on (`params.bloom`), so the bloom gives it a tiny star.
fn glint_colour(rgb: vec3<f32>) -> vec3<f32> {
    return mix(rgb, vec3(1.0), GLINT_WHITE) * (1.0 + GLINT_BLOOM * params.bloom);
}

// The shimmer on ring `k` at angle `along` from the circle's origin: a soft arc a quarter of
// the ring wide, `SHIMMER` brighter at its middle, running round in `SHIMMER_PERIOD` (rings in
// turn the other way). None without motion or with reduced effects.
fn shimmer(along: f32, k: u32, hex: bool, clock: vec2<f32>) -> f32 {
    let way = select(1.0, -1.0, (k & 1u) == 1u);
    let phase = f32(k) * 2.1 + select(0.0, 1.3, hex);
    let c = 0.5 + 0.5 * cos(along - way * TAU * clock.x / SHIMMER_PERIOD - phase);
    return SHIMMER * pow(c, 6.0) * clock.y * params.halo;
}

// The signed distance (pixels) to the sides of a regular `m`-gon inscribed in a circle of
// radius `r0`, its first corner at angle `first`, from the pixel `d` from the centre at angle
// `theta`, and whether it falls on a dash (1 but for a hex). No sides (a large distance) under
// three corners.
fn polygon(m: u32, d: f32, theta: f32, first: f32, r0: f32, hex: bool, h: f32) -> vec2<f32> {
    if m < 3u {
        return vec2(1e4, 0.0);
    }
    let mf = f32(m);
    let sector = TAU / mf;
    var phi = theta - first + sector * 0.5;
    phi = phi - floor(phi / sector) * sector - sector * 0.5;
    let apothem = r0 * cos(PI / mf);
    var dash = 1.0;
    if hex {
        let half_side = r0 * sin(PI / mf);
        dash = dashed(d * sin(phi) + half_side, h);
    }
    return vec2(d * cos(phi) - apothem, dash);
}

// One kind's circle at `p`: `c` its head and the head of the figure it changes from, `m` its
// angle, fade, blend, and place; `at` the actor's centre in framebuffer pixels and `own` its
// cell, for the mask. Settled (blend 1, the old figure its own) it is the figure alone;
// changing, the rings and the polygon cross-fade.
fn circle(
    c: vec2<u32>,
    m: vec4<f32>,
    p: vec2<f32>,
    h: f32,
    at: vec2<f32>,
    own: vec2<i32>,
    clock: vec2<f32>,
) -> Ink {
    var ink = Ink(0.0, 0.0, vec3(0.0));
    let n = c.x & 15u;
    let fade = m.y;
    if n == 0u || fade <= 0.0 {
        return ink;
    }
    let angle = m.x;
    let t = m.z;
    let hex = (c.x & 16u) != 0u;
    let o = c.y;
    let r0 = mix(RING, OUTER_RING, m.w) * h;
    // One physical pixel at the default font and scale 1, more as the cells grow.
    let w = max(h / 18.0, 1.0) * LINE_WEIGHT;
    let sigma = LINE_HALO_SIGMA * h;
    let d = length(p);
    // The pixel's angle on screen (y down: clockwise), from the circle's turned origin.
    let theta = atan2(p.y, p.x);
    let along = theta - angle;

    // Rings: one per effect, up to three (counted on the CPU, `Ring::rings`); one the old figure
    // had and the new has not (or the other way) fades as the figure changes.
    let rings = (c.x >> 16u) & 3u;
    let old_rings = (o >> 16u) & 3u;
    var line = 0.0;
    var glow = 0.0;
    for (var k = 0u; k < max(rings, old_rings); k++) {
        let weight = mix(select(0.0, 1.0, k < old_rings), select(0.0, 1.0, k < rings), t);
        let r = r0 + RING_GAP * h * f32(k);
        var dash = 1.0;
        if hex {
            let s = (along - floor(along / TAU) * TAU) * r;
            dash = dashed(s, h);
        }
        // A soft brighter arc runs round each ring, each its own way along.
        let lift = 1.0 + shimmer(along, k, hex, clock);
        line = max(line, stroke(d - r, w) * dash * weight * lift);
        glow = max(glow, glow_at(d - r, sigma) * dash * weight * lift);
    }

    // The polygon: `slots` corners on the innermost ring (`Ring::slots`), the first at the top,
    // turned; while the count changes, the old polygon fades out as the new fades in.
    let slots = (c.x >> 12u) & 15u;
    let old_slots = (o >> 12u) & 15u;
    let first = angle - PI * 0.5;
    let side = polygon(slots, d, theta, first, r0, hex, h);
    var cov = stroke(side.x, w) * side.y;
    var shine = glow_at(side.x, sigma) * side.y;
    if old_slots != slots {
        let was = polygon(old_slots, d, theta, first, r0, hex, h);
        cov = max(cov * t, stroke(was.x, w) * was.y * (1.0 - t));
        shine = max(shine * t, glow_at(was.x, sigma) * was.y * (1.0 - t));
    }
    line = max(line, cov);
    glow = max(glow, shine);
    if line > 0.0 || glow > 0.001 {
        let show = show_at(at + p, own);
        line *= show;
        glow *= show;
    }
    ink.line = line * fade;
    ink.glow = glow * fade;
    let line_rgb = params.schools[(c.x >> 8u) & 7u].rgb;
    let old_rgb = params.schools[(o >> 8u) & 7u].rgb;
    ink.rgb = mix(old_rgb, line_rgb, t);
    return ink;
}

// `src` (straight colour, alpha `a`) over a premultiplied `dst`.
fn over(dst: vec4<f32>, rgb: vec3<f32>, a: f32) -> vec4<f32> {
    return vec4(rgb * a + dst.rgb * (1.0 - a), a + dst.a * (1.0 - a));
}

// The actor's glyph's colour of glow: its buffs' circle's lines', else its hexes'; and how far
// either has faded in.
fn glyph_rgb(in: VertexOutput) -> vec4<f32> {
    let buffs = (in.inner.x & 15u) != 0u && in.inner_motion.y > 0.0;
    let hexes = (in.outer.x & 15u) != 0u && in.outer_motion.y > 0.0;
    let head = select(in.outer.x, in.inner.x, buffs);
    var fade = 0.0;
    if buffs {
        fade = in.inner_motion.y;
    }
    if hexes {
        fade = max(fade, in.outer_motion.y);
    }
    return vec4(params.schools[(head >> 8u) & 7u].rgb, fade);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let frag = in.position.xy;
    if any(frag < params.clip_min) || any(frag >= params.clip_max) {
        return vec4(0.0);
    }
    let h = in.shape.x;
    let clock = in.ink.zw;
    let packed = u32(in.under.w);
    let glyph = packed >> 16u;
    let seed = packed & 0xffffu;
    // The glyph's own square (and its padding) about the centre, if it has one.
    var glyph_tex = vec4(0.0);
    let near = all(abs(in.local) < vec2(params.glyph_square));
    if params.top == 1u && !near {
        return vec4(0.0);
    }
    if glyph != NO_GLYPH && near {
        glyph_tex = glyph_texel(glyph, in.local / params.glyph_square + vec2(0.5));
    }

    // Over the free-glyph layer: the glyph's glints, nothing else.
    if params.top == 1u {
        let g = glyph_rgb(in);
        let a_glint = glyph_tex.b * glint(glint_phase(seed, 0u), GLINT_PERIOD_A, clock);
        let b_glint = glyph_tex.a * glint(glint_phase(seed, 1u), GLINT_PERIOD_B, clock);
        let a = min(GLINT_ALPHA * max(a_glint, b_glint) * g.a, 1.0);
        if a <= 0.0 {
            return vec4(0.0);
        }
        return vec4(glint_colour(g.rgb) * in.shape.y, a);
    }

    let at = frag - in.local;
    let own = vec2<i32>(round(in.shape.zw));
    let a = circle(in.inner, in.inner_motion, in.local, h, at, own, clock);
    let b = circle(in.outer, in.outer_motion, in.local, h, at, own, clock);
    // The two circles never overlap once settled: take the lines from the one that has the
    // more of them.
    var line = a.line;
    var glow = a.glow;
    var rgb = a.rgb;
    if b.line + b.glow > line + glow {
        line = b.line;
        glow = b.glow;
        rgb = b.rgb;
    }
    // No line crosses the actor's own glyph: they part about its ink's own outline (its halo in
    // the glyph atlas, widened), or, for a glyph the atlas lacks, stop a pixel short of its ink's
    // box, softly.
    let px = max(h / 18.0, 1.0);
    var clear = 1.0;
    if glyph != NO_GLYPH {
        clear = 1.0 - smoothstep(0.02, 0.25, glyph_tex.g);
    } else {
        let off_box = abs(in.local) - in.ink.xy;
        clear = smoothstep(GLYPH_GAP * px, (GLYPH_GAP + 1.5) * px, max(off_box.x, off_box.y));
    }
    line *= clear;
    glow *= clear;
    // Nor tangles with an earlier circle's: it passes under it, out of sight inside its rings.
    if in.under.z > 0.0 {
        let past = length(in.local - in.under.xy) - in.under.z;
        let below = smoothstep(GLYPH_GAP * px, (GLYPH_GAP + 1.5) * px, past);
        line *= below;
        glow *= below;
    }
    // Bottom up: the glyph's halo (under the glyph itself, drawn over this layer), the lines'
    // glow, the lines.
    var ink = vec4(0.0);
    if glyph_tex.g > 0.0 && params.halo > 0.0 {
        let g = glyph_rgb(in);
        let shown = show_at(frag, own);
        ink = over(ink, g.rgb, GLYPH_HALO_ALPHA * glyph_tex.g * g.a * shown);
    }
    ink = over(ink, rgb, LINE_HALO_ALPHA * glow * params.halo);
    ink = over(ink, rgb, LINE_ALPHA * line);
    if ink.a <= 0.0 {
        return vec4(0.0);
    }
    // A spell effect's darkening dims it as it dims free glyphs.
    return vec4(ink.rgb / ink.a * in.shape.y, ink.a);
}
