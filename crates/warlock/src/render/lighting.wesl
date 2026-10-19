// Visual lighting of the map grid (spec §4.4), imported by the console shader. Mirrors
// `render/lighting.rs`: change both together.
//
// `light_values` holds each cell's light (red, green, blue: 1.0 = fully lit, more = overbright)
// and color saturation; `light_flags` marks cells that are lit (visible) and opaque (red) and holds
// their intensity gain (green, `gain_byte` in lighting.rs; light stability). Both cover the
// console with a margin of `MARGIN` cells. Consoles without light (the UI) get 1×1 textures, so
// every lookup misses and their cells are drawn as they are.

@group(constants::MATERIAL_BIND_GROUP) @binding(9) var light_values: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(10) var light_flags: texture_2d<u32>;

const MARGIN: i32 = 1;
const LIT: u32 = 1u;
const OPAQUE: u32 = 2u;
const TERRAIN_FLOOR: f32 = 0.12;
const OVERBRIGHT: f32 = 0.6;
const GLOW: f32 = 0.03;
const LUMA: vec3<f32> = vec3(0.2126, 0.7152, 0.0722);

// `CellLight` in lighting.rs.
struct CellLight {
    rgb: vec3<f32>,
    saturation: f32,
    gain: f32,
    // False where the cell is not lit: draw it as it is.
    lit: bool,
}

fn flags_at(t: vec2<i32>) -> vec2<u32> {
    let size = vec2<i32>(textureDimensions(light_flags));
    if any(t < vec2(0)) || any(t >= size) {
        return vec2(0u);
    }
    return textureLoad(light_flags, t, 0).rg;
}

fn is_open(t: vec2<i32>) -> bool {
    return (flags_at(t).r & (LIT | OPAQUE)) == LIT;
}

// A cell's intensity gain from light stability (`gain_byte` in lighting.rs).
fn gain_at(t: vec2<i32>) -> f32 {
    return 1.0 + (f32(flags_at(t).g) - 128.0) / 256.0;
}

// The light at a point of console cell `cell`, `frac` its position inside the cell (0 to 1 on
// each axis). Walls take their own light flat; open cells interpolate bilinearly between the
// centers of their cell and the three nearest neighbors, leaving out neighbors that are not open
// (and the diagonal unless an orthogonal neighbor next to it is open): `interpolation_weights` and
// `VisualLight::sample` in lighting.rs.
fn light_at(cell: vec2<i32>, frac: vec2<f32>) -> CellLight {
    let t = cell + vec2(MARGIN);
    let flags = flags_at(t).r;
    if (flags & LIT) == 0u {
        return CellLight(vec3(0.0), 1.0, 1.0, false);
    }
    let own = textureLoad(light_values, t, 0);
    if (flags & OPAQUE) != 0u {
        return CellLight(own.rgb, own.a, gain_at(t), true);
    }
    let f = frac - vec2(0.5);
    let step = select(vec2(1), vec2(-1), f < vec2(0.0));
    let a = abs(f);
    let dx = vec2(step.x, 0);
    let dy = vec2(0, step.y);
    let open_x = is_open(t + dx);
    let open_y = is_open(t + dy);
    let open_d = (open_x || open_y) && is_open(t + step);
    var total = (1.0 - a.x) * (1.0 - a.y);
    var sum = own * total;
    var gain = gain_at(t) * total;
    if open_x {
        let w = a.x * (1.0 - a.y);
        sum += textureLoad(light_values, t + dx, 0) * w;
        gain += gain_at(t + dx) * w;
        total += w;
    }
    if open_y {
        let w = (1.0 - a.x) * a.y;
        sum += textureLoad(light_values, t + dy, 0) * w;
        gain += gain_at(t + dy) * w;
        total += w;
    }
    if open_d {
        let w = a.x * a.y;
        sum += textureLoad(light_values, t + step, 0) * w;
        gain += gain_at(t + step) * w;
        total += w;
    }
    sum /= total;
    return CellLight(sum.rgb, sum.a, gain / total, true);
}

// `knee` in lighting.rs: how overbright the light is, 0 at full light approaching 1.
fn knee(light: vec3<f32>) -> vec3<f32> {
    let over = max(light - 1.0, vec3(0.0));
    return over / (1.0 + over);
}

// The overbright glow of `shade` in lighting.rs, in the light's hue.
fn glow(light: vec3<f32>) -> vec3<f32> {
    let peak = max(light.r, max(light.g, light.b));
    if peak <= 1.0 {
        return vec3(0.0);
    }
    return light / peak * (GLOW * knee(vec3(peak)).r);
}

// `exposure` in lighting.rs, per channel.
fn exposure(light: vec3<f32>, floor: f32) -> vec3<f32> {
    let dim = floor + (1.0 - floor) * max(light, vec3(0.0));
    let bright = 1.0 + OVERBRIGHT * knee(light);
    return select(dim, bright, light > vec3(1.0));
}

// `shade` in lighting.rs: a linear color under `light` (from `light_at`), with readability
// floor `floor` (`TERRAIN_FLOOR`, less while a spell effect darkens the view).
fn shade(color: vec3<f32>, light: CellLight, floor: f32) -> vec3<f32> {
    let lit = color * exposure(light.rgb, floor) + glow(light.rgb);
    return mix(vec3(dot(lit, LUMA)), lit, light.saturation) * light.gain;
}
