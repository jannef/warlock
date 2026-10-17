//! Still and running water in the grid shader (spec §4.3).
//!
//! Both are terrain animations evaluated in `console.wesl`:
//! - **Still water** is a `wave`: a faint, slow swell whose phase travels across the pool.
//! - **Running water** is a `flow`. Each cell carries its [`Flow`] (direction and speed, packed
//!   by [`flow_byte`] into the spare byte of the glyph texture). The shader draws the glyph as a
//!   pattern repeating every cell and moves it downstream ([`flow_shift`]), so ripples leave a
//!   cell on one side and come back in on the other and a stream reads as one moving surface.
//!   Neighbors with the same flow line up seamlessly; where the direction or speed changes along a
//!   stream the two patterns meet with a seam, so streams keep an even speed along their length
//!   (the vast river turns only by steps that keep the same speed downstream).
//! - Either may **glint** ([`glint`]): now and then, at a random moment of each cell's own cycle,
//!   the glyph brightens briefly in the color of the light on it (its flicker and hue drift
//!   included), so dark water never glints. Glints are hashed from the level position (the
//!   shader gets the map grid's level origin), so they stay put while the camera scrolls.
//!
//! With glyph animation off nothing here runs (the animation table is empty). The curves below
//! mirror the shader and exist to test it: change both together.

#[cfg(test)]
use bevy::math::Vec2;
use warlock_core::Dir;
use warlock_core::content::Flow;

/// A cell's flow speed is stored in eighths of the terrain's speed (5 bits: up to 31/8). Mirrored
/// in `console.wesl`.
const FLOW_SPEED_STEPS: f32 = 8.0;

/// Packs a cell's flow for the shader: the direction (index in [`Dir::ALL`], clockwise from N)
/// in the low 3 bits, the speed in eighths of the terrain's (1 to 31) above. 0 is no flow.
pub fn flow_byte(flow: Flow) -> u8 {
    let dir = Dir::ALL.iter().position(|d| *d == flow.dir).unwrap_or(0) as u8;
    let speed = (f32::from(flow.speed) * FLOW_SPEED_STEPS / 100.0)
        .round()
        .clamp(1.0, 31.0) as u8;
    dir | (speed << 3)
}

/// The step (each axis -1, 0, or 1; y down) and speed multiplier of a [`flow_byte`]; `None` for
/// no flow (`flow_step` in the shader).
#[cfg(test)]
fn unpack_flow(byte: u8) -> Option<(Vec2, f32)> {
    let speed = f32::from(byte >> 3) / FLOW_SPEED_STEPS;
    let (dx, dy) = Dir::ALL[usize::from(byte & 7)].delta();
    (speed > 0.0).then(|| (Vec2::new(dx as f32, dy as f32), speed))
}

/// How far a running-water glyph has moved downstream at `time`, in pixels within a cell of
/// `cell` pixels: `speed` cell widths per second along each axis of the flow's `step`, wrapping
/// every cell.
#[cfg(test)]
fn flow_shift(time: f32, speed: f32, step: Vec2, cell: Vec2) -> Vec2 {
    let cycles = Vec2::splat(time * speed) * cell.x / cell;
    step * (cycles - cycles.floor()) * cell
}

/// Average seconds between two chances of a glint on one cell; each cell's own cycle is 75 % to
/// 125 % of it, and a glint happens in [`GLINT_CHANCE`] of the cycles. Mirrored in
/// `console.wesl`.
#[cfg(test)]
const GLINT_PERIOD: f32 = 9.0;
#[cfg(test)]
const GLINT_CHANCE: f32 = 0.4;
/// How long a glint lasts, in seconds.
#[cfg(test)]
const GLINT_LENGTH: f32 = 0.9;

/// How bright a glint is at `time` on the cell with hash `hash` ([`super::anim::cell_hash`] of
/// its level position), 0 to 1: in some of the cell's cycles, at a random moment, a soft rise and
/// fall over [`GLINT_LENGTH`] seconds.
#[cfg(test)]
fn glint(time: f32, hash: u32) -> f32 {
    use super::anim::{hash_phase, scramble};
    use std::f32::consts::PI;
    let period = GLINT_PERIOD * (0.75 + 0.5 * hash_phase(hash));
    let cycles = time / period + hash_phase(scramble(hash ^ 0x68e3_1da4));
    let n = cycles.floor();
    let roll = scramble(hash ^ (n as u32).wrapping_mul(0x9e37_79b9));
    if hash_phase(roll) >= GLINT_CHANCE {
        return 0.0;
    }
    let start = hash_phase(scramble(roll)) * (period - GLINT_LENGTH);
    let x = ((cycles - n) * period - start) / GLINT_LENGTH;
    if (0.0..1.0).contains(&x) {
        (PI * x).sin().powi(2)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::anim::cell_hash;
    use warlock_core::Pos;

    #[test]
    fn flow_bytes_round_trip_direction_and_speed() {
        for dir in Dir::ALL {
            let (step, speed) = unpack_flow(flow_byte(Flow::new(dir, 100))).expect("flows");
            assert_eq!(step, Vec2::new(dir.delta().0 as f32, dir.delta().1 as f32));
            assert_eq!(speed, 1.0);
        }
        let fast = unpack_flow(flow_byte(Flow::new(Dir::S, 200))).expect("flows");
        assert_eq!(fast.1, 2.0);
        // Slow flows still move; the fastest is capped by the 5 bits.
        assert_eq!(
            unpack_flow(flow_byte(Flow::new(Dir::S, 1)))
                .expect("flows")
                .1,
            0.125
        );
        assert_eq!(
            unpack_flow(flow_byte(Flow::new(Dir::S, 900)))
                .expect("flows")
                .1,
            3.875
        );
        assert_eq!(unpack_flow(0), None);
    }

    #[test]
    fn flow_moves_downstream_and_wraps_every_cell() {
        let cell = Vec2::new(11.0, 22.0);
        let south = Vec2::new(0.0, 1.0);
        // Half a cell width per second: 5.5 pixels a second on either axis.
        let a = flow_shift(1.0, 0.5, south, cell);
        assert!((a - Vec2::new(0.0, 5.5)).length() < 1e-4);
        // Down a cell (22 px) in 4 seconds, then the pattern starts over.
        assert!(flow_shift(3.99, 0.5, south, cell).y > 21.0);
        assert!(flow_shift(4.01, 0.5, south, cell).y < 1.0);
        // A diagonal moves along both axes at the same pixel speed; west is negative.
        let sw = flow_shift(1.0, 0.5, Vec2::new(-1.0, 1.0), cell);
        assert!((sw - Vec2::new(-5.5, 5.5)).length() < 1e-4);
        // Twice the speed, twice as far.
        let fast = flow_shift(1.0, 1.0, south, cell);
        assert!((fast.y - 2.0 * a.y).abs() < 1e-4);
    }

    #[test]
    fn glints_are_brief_occasional_and_differ_per_cell() {
        let cells: Vec<u32> = (0..40).map(|x| cell_hash(Pos::new(x, 3))).collect();
        let (mut lit, mut samples) = (0, 0);
        for i in 0..2000 {
            let t = i as f32 * 0.05;
            for &h in &cells {
                let g = glint(t, h);
                assert!((0.0..=1.0).contains(&g));
                samples += 1;
                if g > 0.0 {
                    lit += 1;
                }
            }
        }
        // About GLINT_CHANCE × GLINT_LENGTH / GLINT_PERIOD of the time: rare.
        let share = lit as f32 / samples as f32;
        assert!((0.02..0.07).contains(&share), "glinting {share}");
        // Never all at once.
        assert!((0..200).all(|i| cells.iter().any(|h| glint(i as f32 * 0.5, *h) == 0.0)));
        // A glint rises and falls smoothly.
        let h = cells[0];
        let peak = (0..20000)
            .map(|i| (i as f32 * 0.01, glint(i as f32 * 0.01, h)))
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .expect("samples");
        assert!(peak.1 > 0.9, "a full glint");
        assert!(glint(peak.0 + 0.05, h) > 0.5 && glint(peak.0 + 0.6, h) == 0.0);
    }
}
