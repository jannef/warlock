//! The steady part of the visual light composite, kept between frames (#785).
//!
//! Between turns the only input of [`VisualLight::composite_with`] that changes from one frame to
//! the next is the visual time, and it reaches the light through the modulation of the unsteady
//! sources alone (presets with a `stability`, while the `flicker` effect is on). So the cache
//! holds, per cell, everything the composite adds up to a cell's first unsteady contribution: its
//! flags, its perceived light, its drawn light and gain weights. From there on it keeps the
//! contributions themselves ([`Step`]), in the composite's own order, so each frame replays them
//! for those cells alone with the sources' modulation of the moment, and the sums come out
//! bit for bit as the full composite's.
//!
//! It is rebuilt when an input changes: the perception snapshot (`Game`), the creature
//! [`Shadows`], the grid's origin or size, or whether light flickers at all. While a transient or
//! a spell light is live, [`super::update`] composites in full instead (and that invalidates the
//! cache). A frame whose cells have nothing unsteady, or whose visual time is the last one's, does
//! no work, and `dirty` stays as it was.

use super::*;

/// One contribution to a cell past its first unsteady one, replayed every frame.
#[derive(Clone, Copy, Debug)]
enum Step {
    /// A steady source's drawn light (shadows applied); `slot` is the cell's place in `tail`.
    Steady { slot: u32, draw: Vec4 },
    /// An unsteady source's (`source`, its place in `sources`) light and the share of it creature
    /// shadows keep.
    Unsteady {
        slot: u32,
        source: u32,
        light: Vec4,
        keep: f32,
    },
}

/// What [`VisualLight::composite_cached`] did this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pass {
    /// Rebuilt the cache and recomposited every cell.
    Rebuilt,
    /// Redid the cells unsteady sources light.
    Flickered,
    /// Nothing: no input changed and nothing flickers.
    Skipped,
}

#[derive(Debug, Default)]
pub(super) struct SteadyCache {
    /// False until built, and after a full composite left the light out of step with it.
    valid: bool,
    origin: IVec2,
    size: UVec2,
    /// Whether it was built with the sources modulated (`time` given).
    flicker: bool,
    /// The visual time of the last frame done.
    time: Option<f32>,
    /// Per cell: perceived light (all of it), and drawn light and gain weights up to its first
    /// unsteady contribution.
    sums: Vec<Vec4>,
    drawn: Vec<Vec4>,
    gains: Vec<Vec2>,
    /// The cells with an unsteady contribution, in the order they met one, and the contributions
    /// from each one's first unsteady one on, in the full composite's order.
    tail: Vec<u32>,
    steps: Vec<Step>,
    /// The unsteady sources: key and preset.
    sources: Vec<(u64, LightId)>,
}

impl SteadyCache {
    pub(super) fn invalidate(&mut self) {
        self.valid = false;
    }
}

impl VisualLight {
    /// [`VisualLight::composite`] (no transient or spell light), from the cache: rebuilt when
    /// `changed` (a new snapshot or new shadows, which must be the ones given), when `origin`,
    /// `size` or flickering at all (`time` given) differ from the last call, or after a full
    /// composite; otherwise only the cells unsteady sources light are redone, and nothing at all
    /// when there are none or `time` is the last frame's. Exactly what the full composite gives.
    pub fn composite_cached(
        &mut self,
        view: &PerceptionView<'_>,
        shadows: &Shadows,
        origin: IVec2,
        size: UVec2,
        time: Option<f32>,
        changed: bool,
    ) -> Pass {
        let cache = &self.cache;
        let rebuild = changed
            || !cache.valid
            || (origin, size, time.is_some()) != (cache.origin, cache.size, cache.flicker);
        if rebuild {
            self.rebuild(view, shadows, origin, size, time);
            return Pass::Rebuilt;
        }
        if self.cache.tail.is_empty() || time == self.cache.time {
            return Pass::Skipped;
        }
        let (drawn, gains) = self.replay(view, time);
        let finish = Finish::new(view, 1.0);
        let mut dirty = false;
        for (slot, &i) in self.cache.tail.iter().enumerate() {
            let i = i as usize;
            let p = cell_pos(origin, size, i);
            let cell = finish.cell(
                view,
                p,
                self.cache.sums[i],
                drawn[slot],
                gains[slot],
                Vec3::ZERO,
            );
            if self.cells[i] != cell {
                self.cells[i] = cell;
                dirty = true;
            }
        }
        self.dirty |= dirty;
        Pass::Flickered
    }

    fn rebuild(
        &mut self,
        view: &PerceptionView<'_>,
        shadows: &Shadows,
        origin: IVec2,
        size: UVec2,
        time: Option<f32>,
    ) {
        let content = view.content();
        let (flags, base) = bases(view, origin, size);
        let n = flags.len();
        let mut cache = SteadyCache {
            valid: true,
            origin,
            size,
            flicker: time.is_some(),
            time,
            sums: base.clone(),
            gains: base.iter().map(|b| Vec2::splat(b.element_sum())).collect(),
            drawn: base,
            tail: Vec::new(),
            steps: Vec::new(),
            sources: Vec::new(),
        };
        // Each cell's place in `tail` once it has an unsteady contribution.
        let mut slots: Vec<Option<u32>> = vec![None; n];
        let mut steps = Vec::new();
        for source in view.light_sources() {
            // Unsteady as the full composite would modulate it: a preset with a stability, with
            // time given. The sun and the moons have no preset.
            let unsteady = match (time, source.preset()) {
                (Some(_), Some(preset)) => !content.light(preset).stability.is_stable(),
                _ => false,
            };
            let index = unsteady.then(|| {
                cache
                    .sources
                    .push((source.key, source.preset().expect("unsteady")));
                cache.sources.len() as u32 - 1
            });
            let shadows = shadows.of(source.key);
            for &(p, light) in &source.contributions {
                let Some(i) = cell_index(origin, size, IVec2::new(p.x, p.y)) else {
                    continue;
                };
                if flags[i] & LIT == 0 {
                    continue;
                }
                let light = light_vec(light);
                cache.sums[i] += light;
                let keep = shadows.keep(p);
                if let Some(source) = index {
                    let slot = *slots[i].get_or_insert_with(|| {
                        cache.tail.push(i as u32);
                        cache.tail.len() as u32 - 1
                    });
                    steps.push(Step::Unsteady {
                        slot,
                        source,
                        light,
                        keep,
                    });
                } else {
                    let draw = Modulation::STEADY.color(light) * keep;
                    match slots[i] {
                        Some(slot) => steps.push(Step::Steady { slot, draw }),
                        None => {
                            cache.drawn[i] += draw;
                            cache.gains[i] +=
                                Vec2::new(Modulation::STEADY.gain, 1.0) * draw.element_sum();
                        }
                    }
                }
            }
        }
        cache.steps = steps;
        self.cache = cache;

        let (drawn, gains) = self.replay(view, time);
        let finish = Finish::new(view, 1.0);
        let cache = &self.cache;
        let cells: Vec<CellLight> = (0..n)
            .map(|i| {
                if flags[i] & LIT == 0 {
                    return CellLight::DARK;
                }
                let (d, g) = match slots[i] {
                    Some(slot) => (drawn[slot as usize], gains[slot as usize]),
                    None => (cache.drawn[i], cache.gains[i]),
                };
                let p = cell_pos(origin, size, i);
                finish.cell(view, p, cache.sums[i], d, g, Vec3::ZERO)
            })
            .collect();
        self.set(origin, size, cells, flags, 1.0);
    }

    /// The drawn light and gain weights of the cache's tail cells at visual time `time`: the
    /// cached sums up to each one's first unsteady contribution, plus the rest replayed in order.
    fn replay(&mut self, view: &PerceptionView<'_>, time: Option<f32>) -> (Vec<Vec4>, Vec<Vec2>) {
        let cache = &mut self.cache;
        cache.time = time;
        let content = view.content();
        let modulations: Vec<Modulation> = match time {
            Some(time) => (cache.sources.iter())
                .map(|&(key, preset)| modulation(key, content.light(preset), time))
                .collect(),
            None => Vec::new(),
        };
        let mut drawn: Vec<Vec4> = cache
            .tail
            .iter()
            .map(|&i| cache.drawn[i as usize])
            .collect();
        let mut gains: Vec<Vec2> = cache
            .tail
            .iter()
            .map(|&i| cache.gains[i as usize])
            .collect();
        for step in &cache.steps {
            let (slot, draw, gain) = match *step {
                Step::Steady { slot, draw } => (slot, draw, Modulation::STEADY.gain),
                Step::Unsteady {
                    slot,
                    source,
                    light,
                    keep,
                } => {
                    let modulation = modulations[source as usize];
                    (slot, modulation.color(light) * keep, modulation.gain)
                }
            };
            drawn[slot as usize] += draw;
            gains[slot as usize] += Vec2::new(gain, 1.0) * draw.element_sum();
        }
        (drawn, gains)
    }
}
