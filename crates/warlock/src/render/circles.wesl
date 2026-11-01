// Spell circles (spec §4.8 "Spell circles", `circles.rs`): one quad per circled actor, all in one
// mesh and one draw. Every vertex of a quad carries the circle (flat): the cell height, the two
// circles' angles, the darkening, the actor's cell, and each circle packed as `[head, runes 0-2,
// runes 3-5, 0]`.
//
// The fragment works in pixels from the actor's centre (`local`, y down): thin anti-aliased
// rings at signed distances, a regular polygon inscribed in the innermost ring with its corners
// at the rune slots, dashed for a hex along both, and a rune at each slot, turned so its foot
// faces the centre, a tap of the rune atlas: its coverage, its halo, and its backdrop (red,
// green, blue). Sizes are in cell heights and mirror `circles.rs`: change both together.
//
// The map's per-cell flags (`light_flags`, `lighting.rs`) keep the circle to what the player
// sees: a rune fades by the cell under its centre, a line by the cell under its pixel, softly
// across cell edges; nothing over unseen or remembered cells or walls, little over a cell
// holding another glyph (a creature, an object, a torch), the actor's own cell left out.

import bevy_sprite_render::mesh2d::functions::{get_world_from_local, mesh2d_position_local_to_clip};
import bevy_sprite_render::mesh2d::view_bindings::view;

struct CircleParams {
    // Framebuffer rectangle outside of which nothing is drawn (the map viewport).
    clip_min: vec2<f32>,
    clip_max: vec2<f32>,
    // Framebuffer position of the map grid's cell (0, 0), and the cell size, in pixels.
    grid_origin: vec2<f32>,
    cell_size: vec2<f32>,
    // A rune atlas slot's side in texels (its padding and border included), the texels about
    // its rune square on each side, slots per row, and texels per screen pixel.
    slot: u32,
    pad: u32,
    columns: u32,
    supersample: u32,
    // A rune's square, in cell heights.
    rune_size: f32,
    // The halo's strength (0 or 1).
    halo: f32,
    // The school colours, palette-mapped, linear (rgb), by `School::ALL` position.
    schools: array<vec4<f32>, 8>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> params: CircleParams;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var runes: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var runes_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(3) var light_flags: texture_2d<u32>;

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
// The darkening under a rune, and its halo's opacity in its school's colour.
const BACKDROP_ALPHA: f32 = 0.8;
const HALO_ALPHA: f32 = 0.3;
// `circles.rs`: the pulse's period and swing.
const PULSE_PERIOD: f32 = 3.6;
const PULSE_ALPHA: f32 = 0.12;
const PULSE_SIZE: f32 = 0.04;
// `lighting.rs`'s cell flags, and the margin of its textures (cells).
const LIT: u32 = 1u;
const OPAQUE: u32 = 2u;
const OCCUPIED: u32 = 4u;
const MARGIN: i32 = 1;
// How much of the circle shows over a cell holding another glyph: nothing (a ghost of a rune
// over a creature still reads as a smudge), reached softly at the cell's edge.
const OCCUPIED_SHOW: f32 = 0.0;
// Half the width of the soft edge between cells, in cells, either side of the edge.
const SOFT: f32 = 0.22;
// How far the lines stay off the actor's glyph's ink, in pixels at the default font.
const GLYPH_GAP: f32 = 1.0;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    // Framebuffer pixels (z unused).
    @location(0) position: vec3<f32>,
    // From the actor's centre, in pixels.
    @location(1) local: vec2<f32>,
    // Cell height, darkening, and the actor's map grid cell.
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
    // Half the size of the actor's glyph's ink, in pixels about the centre.
    // Half the size of the actor's glyph's ink (pixels), the visual time, and the pulse (0 or 1).
    @location(8) ink: vec4<f32>,
    // How much of each rune shows where circles crowd, a byte each (`circles.rs::crowding`):
    // the buffs' slots 0-3, 4-5, the hexes' 0-3, 4-5.
    @location(9) crowd: vec4<u32>,
    // The earlier circle this one passes under (`circles.rs::under`): its centre from this
    // one's and its outermost ring's radius, in pixels (radius 0: none).
    @location(10) under: vec4<f32>,
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
    @location(7) @interpolate(flat) ink: vec4<f32>,
    @location(8) @interpolate(flat) crowd: vec4<u32>,
    @location(9) @interpolate(flat) under: vec4<f32>,
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
    out.ink = vertex.ink;
    out.crowd = vertex.crowd;
    out.under = vertex.under;
    return out;
}

// What one circle draws at a pixel: the lines' coverage and colour; the runes', their halo's
// (in the rune's colour), and their backdrop's.
struct Ink {
    line: f32,
    line_rgb: vec3<f32>,
    rune: f32,
    rune_rgb: vec3<f32>,
    halo: f32,
    halo_rgb: vec3<f32>,
    backdrop: f32,
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

// The rune in atlas slot `slot` at `q`, the pixel's place in the rune's square (0 to 1 across,
// y down; its halo and backdrop reach into the padding about it): coverage, halo, backdrop.
fn rune_texel(slot: u32, q: vec2<f32>) -> vec3<f32> {
    let s = f32(params.slot);
    let pad = f32(params.pad);
    let inner = s - 2.0 * pad;
    // Inside the slot's one-texel border: filtering never reads a neighbour.
    let t = pad + q * inner;
    if any(t < vec2(1.0)) || any(t > vec2(s - 1.0)) {
        return vec3(0.0);
    }
    let origin = vec2<f32>(f32(slot % params.columns), f32(slot / params.columns)) * s;
    let size = vec2<f32>(textureDimensions(runes));
    return textureSampleLevel(runes, runes_sampler, (origin + t) / size, 0.0).rgb;
}

// How much of the circle shows over map grid cell `cell`: none where the player does not see
// it now (unseen, remembered, outside the view) or where it is a wall, a little where another
// glyph is (`OCCUPIED`), all of it elsewhere and on the actor's own cell `own`.
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
        return OCCUPIED_SHOW;
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

// Where slot `k` stands on a figure changing from `was` slots to `to`, `blend` of the way, in
// radians from the first corner (`circles.rs::slot_angle`): from its old place to its new.
fn slot_angle(k: u32, was: u32, to: u32, blend: f32) -> f32 {
    let at_old = TAU * f32(k) / f32(max(was, 1u));
    let at_new = TAU * f32(k) / f32(max(to, 1u));
    return mix(at_old, at_new, blend);
}

// `circles.rs::rune_turn`: a rune at angle `a` on its circle turned so its foot faces the
// centre.
fn rune_turn(a: f32) -> f32 {
    return a + PI * 0.5;
}

// `circles.rs::pulse_phase` and `pulse`: slot `k`'s phase, and its swing at the frame's time.
fn pulse_phase(k: u32, hex: bool) -> f32 {
    return fract(f32(k) * 0.618034 + select(0.0, 0.5, hex));
}

// `clock`: the visual time, and 1 if the runes pulse (else 0).
fn pulse(phase: f32, clock: vec2<f32>) -> f32 {
    return sin(TAU * (clock.x / PULSE_PERIOD + phase)) * clock.y;
}

// How much of slot `k`'s rune shows where circles crowd, from its kind's two crowd words.
fn crowd_of(words: vec2<u32>, k: u32) -> f32 {
    let word = select(words.x, words.y, k >= 4u);
    return f32((word >> (8u * (k % 4u))) & 255u) / 255.0;
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
// (whose head is `c.w`), `m` its angle, fade, blend, and place; `at` the actor's centre in
// framebuffer pixels and `own` its cell, for the mask. Settled (blend 1, the old figure its
// own) it is the figure alone; changing, the rings and the polygon cross-fade and every rune
// glides from its old slot to its new one, changing its character on the way if it has to.
fn circle(
    c: vec4<u32>,
    old: vec2<u32>,
    m: vec4<f32>,
    p: vec2<f32>,
    h: f32,
    at: vec2<f32>,
    own: vec2<i32>,
    crowd: vec2<u32>,
    clock: vec2<f32>,
) -> Ink {
    var ink = Ink(0.0, vec3(0.0), 0.0, vec3(0.0), 0.0, vec3(0.0), 0.0);
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
    if line > 0.0 {
        line *= show_at(at + p, own);
    }
    ink.line = line * fade;
    let line_rgb = params.schools[(c.x >> 8u) & 7u].rgb;
    let old_rgb = params.schools[(o >> 8u) & 7u].rgb;
    ink.line_rgb = mix(old_rgb, line_rgb, t);

    // The runes: squares about each slot's centre on the rune radius, turned to face the
    // centre, each gliding from its old slot's place to its new one's, and pulsing.
    let size = params.rune_size * h;
    // Past a rune's centre by this much nothing of it shows: its square's corner, its padding
    // (the halo and backdrop), and its pulse.
    let pad = f32(params.pad) / f32(params.slot - 2u * params.pad);
    let reach = size * (0.5 + pad) * 1.415 * (1.0 + PULSE_SIZE);
    let words = c.yz;
    for (var k = 0u; k < max(slots, old_slots); k++) {
        let a = first + slot_angle(k, old_slots, slots, t);
        let centre = rr * vec2(cos(a), sin(a));
        let v = p - centre;
        if dot(v, v) > reach * reach {
            continue;
        }
        let swing = pulse(pulse_phase(k, hex), clock);
        let turn = rune_turn(a);
        let cs = cos(turn);
        let sn = sin(turn);
        // The pixel in the rune's own frame: turned back by the rune's turn.
        let u = vec2(cs * v.x + sn * v.y, -sn * v.x + cs * v.y);
        let q = u / (size * (1.0 + PULSE_SIZE * swing)) + vec2(0.5);
        let has_new = k < slots;
        let has_old = k < old_slots;
        let rune = rune_of(words, k % n);
        let was = rune_of(old, k % on);
        var tex = vec3(0.0);
        var rgb = vec3(0.0);
        if has_new && has_old && all(rune == was) {
            tex = rune_texel(rune.x, q);
            rgb = params.schools[rune.y].rgb;
        } else {
            var now = vec3(0.0);
            if has_new {
                now = rune_texel(rune.x, q) * t;
            }
            var then = vec3(0.0);
            if has_old {
                then = rune_texel(was.x, q) * (1.0 - t);
            }
            tex = max(now, then);
            rgb = params.schools[select(was.y, rune.y, now.r + now.g >= then.r + then.g)].rgb;
        }
        if all(tex == vec3(0.0)) {
            continue;
        }
        // The whole rune shows by the cells under it (never cut in two) and gives way to an
        // earlier circle's rune near it: both worked out on the CPU (`circles.rs::rune_show`,
        // `crowding`), a byte a rune.
        let show = crowd_of(crowd, k) * fade;
        let cov = tex.r * show * (1.0 + PULSE_ALPHA * swing);
        if cov > ink.rune {
            ink.rune = cov;
            ink.rune_rgb = rgb;
        }
        if tex.g * show > ink.halo {
            ink.halo = tex.g * show;
            ink.halo_rgb = rgb;
        }
        ink.backdrop = max(ink.backdrop, tex.b * show);
    }
    return ink;
}

// `src` (straight colour, alpha `a`) over a premultiplied `dst`.
fn over(dst: vec4<f32>, rgb: vec3<f32>, a: f32) -> vec4<f32> {
    return vec4(rgb * a + dst.rgb * (1.0 - a), a + dst.a * (1.0 - a));
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let frag = in.position.xy;
    if any(frag < params.clip_min) || any(frag >= params.clip_max) {
        return vec4(0.0);
    }
    let h = in.shape.x;
    let at = frag - in.local;
    let own = vec2<i32>(round(in.shape.zw));
    let clock = in.ink.zw;
    let a = circle(in.inner, in.olds.xy, in.inner_motion, in.local, h, at, own, in.crowd.xy, clock);
    let b = circle(in.outer, in.olds.zw, in.outer_motion, in.local, h, at, own, in.crowd.zw, clock);
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
    var halo = a.halo;
    var halo_rgb = a.halo_rgb;
    if b.halo > halo {
        halo = b.halo;
        halo_rgb = b.halo_rgb;
    }
    let backdrop = max(a.backdrop, b.backdrop);
    // No line crosses the actor's own glyph: they stop a pixel short of its ink, softly.
    let px = max(h / 18.0, 1.0);
    let off_box = abs(in.local) - in.ink.xy;
    line *= smoothstep(GLYPH_GAP * px, (GLYPH_GAP + 1.5) * px, max(off_box.x, off_box.y));
    // Nor tangles with an earlier circle's: it passes under it, out of sight inside its rings.
    if in.under.z > 0.0 {
        let past = length(in.local - in.under.xy) - in.under.z;
        line *= smoothstep(GLYPH_GAP * px, (GLYPH_GAP + 1.5) * px, past);
    }
    // Bottom up: the backdrop darkens what is under a rune, the halo glows about it, the lines,
    // and the rune itself on top (it never crosses a line, but may touch a polygon's corner).
    var ink = vec4(0.0);
    ink = over(ink, vec3(0.0), BACKDROP_ALPHA * backdrop);
    ink = over(ink, halo_rgb, HALO_ALPHA * halo * params.halo);
    ink = over(ink, line_rgb, LINE_ALPHA * line);
    ink = over(ink, rune_rgb, min(RUNE_ALPHA * rune, 1.0));
    if ink.a <= 0.0 {
        return vec4(0.0);
    }
    // A spell effect's darkening dims it as it dims free glyphs.
    return vec4(ink.rgb / ink.a * in.shape.y, ink.a);
}
