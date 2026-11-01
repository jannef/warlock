//! The fades and glides of spell circles (spec §4.8 "Spell circles", docs/design/buff-circles.md
//! §11): a circle appears over [`FADE`] seconds and goes over as long; when an effect begins or
//! ends, its figure changes over [`GLIDE`] seconds (the runes glide to their new slots, the
//! polygon and the rings blend from the old count), and a hexes' circle moves out when a buffs'
//! one comes inside it and back in once that one has gone.
//!
//! One track per actor and kind (buffs, hexes), fed the circles of each turn ([`Tracks::update`])
//! and read each frame ([`Tracks::shown`]). Without animation (`motion` off, reduced effects, or
//! `--freeze-time`) every circle is drawn settled, and one that went is gone at once: still and
//! cut, as slice 1 drew them. An actor the player no longer sees loses its circles at once.

use crate::render::circles::{self, BUFFS, Circle, FADE, GLIDE, HEXES, Ring, Shown};
use std::collections::{BTreeMap, BTreeSet};
use warlock_core::world::ActorId;

/// One kind's circle on one actor.
#[derive(Clone, Copy, Debug)]
struct Track {
    /// The figure now (the last one while the circle goes).
    ring: Ring,
    /// The figure it changes from, and when the change began.
    from: Ring,
    changed: f32,
    /// When the fade began (moved back when it turns, so it goes on from where it stood), and
    /// whether the circle is going.
    faded: f32,
    going: bool,
    /// The place (0 inner, 1 outer) it moves from and to, and when it began to.
    place: (f32, f32, f32),
}

impl Track {
    fn new(ring: Ring, now: f32) -> Self {
        let place = if ring.outer { 1.0 } else { 0.0 };
        Track {
            ring,
            from: ring,
            changed: now - GLIDE,
            faded: now,
            going: false,
            place: (place, place, now - GLIDE),
        }
    }

    /// How much of it shows, 0 to 1.
    fn fade(&self, now: f32) -> f32 {
        let t = ((now - self.faded) / FADE).clamp(0.0, 1.0);
        if self.going { 1.0 - t } else { t }
    }

    fn place_at(&self, now: f32) -> f32 {
        let (from, to, when) = self.place;
        from + (to - from) * circles::ease((now - when) / GLIDE)
    }

    fn go(&mut self, now: f32) {
        if !self.going {
            self.faded = now - (1.0 - self.fade(now)) * FADE;
            self.going = true;
        }
    }

    fn come(&mut self, now: f32) {
        if self.going {
            self.faded = now - self.fade(now) * FADE;
            self.going = false;
        }
    }
}

/// Every circle being drawn, by actor and kind.
#[derive(Debug, Default)]
pub(super) struct Tracks(BTreeMap<(ActorId, usize), Track>);

impl Tracks {
    /// Whether `id` has a circle being drawn (appearing, shown, or going).
    pub fn has(&self, id: ActorId) -> bool {
        self.0.range((id, BUFFS)..=(id, HEXES)).next().is_some()
    }

    /// Takes the circles of a new turn at `now`: every seen actor's (empty for none). An actor
    /// left out is no longer seen and loses its circles.
    pub fn update(&mut self, now: f32, seen: &[(ActorId, Circle)]) {
        let ids: BTreeSet<ActorId> = seen.iter().map(|&(id, _)| id).collect();
        self.0.retain(|(id, _), _| ids.contains(id));
        for &(id, circle) in seen {
            for (kind, ring) in circle.kinds().into_iter().enumerate() {
                match (self.0.get_mut(&(id, kind)), ring) {
                    (None, None) => {}
                    (None, Some(ring)) => {
                        self.0.insert((id, kind), Track::new(ring, now));
                    }
                    (Some(track), None) => track.go(now),
                    (Some(track), Some(ring)) => {
                        track.come(now);
                        if !track.ring.same_figure(&ring) {
                            track.from = track.ring;
                            track.changed = now;
                        }
                        track.ring = ring;
                    }
                }
            }
        }
    }

    /// Forgets the circles that have gone, and starts a hexes' circle moving out or in: it stands
    /// outside while its actor has a buffs' circle being drawn (going too), inside when not.
    /// Without animation, a going circle is gone at once.
    pub fn settle(&mut self, now: f32, animate: bool) {
        self.0
            .retain(|_, t| !t.going || animate && t.fade(now) > 0.0);
        let buffed: BTreeSet<ActorId> = (self.0.keys())
            .filter(|&&(_, kind)| kind == BUFFS)
            .map(|&(id, _)| id)
            .collect();
        for (&(id, kind), track) in &mut self.0 {
            let to = if kind == HEXES && buffed.contains(&id) {
                1.0
            } else {
                0.0
            };
            if track.place.1 != to {
                track.place = (track.place_at(now), to, now);
            }
        }
    }

    /// `id`'s circles as drawn at `now`, turned as `motion` says ([`circles::angle`]); settled
    /// without animation.
    pub fn shown(&self, id: ActorId, now: f32, animate: bool, motion: bool) -> [Option<Shown>; 2] {
        [BUFFS, HEXES].map(|kind| {
            let track = self.0.get(&(id, kind))?;
            let angle = circles::angle(&track.ring, now, motion);
            if !animate {
                let place = track.place.1;
                return Some(Shown {
                    place,
                    ..Shown::settled(track.ring, angle)
                });
            }
            let blend = circles::ease((now - track.changed) / GLIDE);
            Some(Shown {
                ring: track.ring,
                from: if blend < 1.0 { track.from } else { track.ring },
                blend,
                fade: track.fade(now),
                place: track.place_at(now),
                angle,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::circles::{Rune, layout};

    const A: ActorId = ActorId(7);

    fn buff(slot: u8) -> (Rune, bool) {
        (Rune { slot, school: 2 }, false)
    }

    fn hex(slot: u8) -> (Rune, bool) {
        (Rune { slot, school: 5 }, true)
    }

    /// Feeds one turn's circle for `A` at `now` and settles.
    fn turn(tracks: &mut Tracks, now: f32, effects: &[(Rune, bool)]) {
        tracks.update(now, &[(A, layout(effects.iter().copied()))]);
        tracks.settle(now, true);
    }

    fn at(tracks: &mut Tracks, now: f32) -> [Option<Shown>; 2] {
        tracks.settle(now, true);
        tracks.shown(A, now, true, true)
    }

    #[test]
    fn a_circle_fades_in_and_out_over_half_a_second() {
        let mut tracks = Tracks::default();
        turn(&mut tracks, 10.0, &[buff(0)]);
        let fade = |s: [Option<Shown>; 2]| s[BUFFS].map(|s| s.fade);
        assert_eq!(fade(at(&mut tracks, 10.0)), Some(0.0));
        assert_eq!(fade(at(&mut tracks, 10.25)), Some(0.5));
        assert_eq!(fade(at(&mut tracks, 10.5)), Some(1.0));
        assert_eq!(fade(at(&mut tracks, 60.0)), Some(1.0));
        turn(&mut tracks, 61.0, &[]);
        assert!(tracks.has(A), "going, still drawn");
        assert_eq!(fade(at(&mut tracks, 61.25)), Some(0.5));
        assert_eq!(fade(at(&mut tracks, 61.5)), None, "gone");
        assert!(!tracks.has(A));
    }

    // An effect back while its circle goes: the fade turns where it stood, no jump.
    #[test]
    fn a_fade_turns_where_it_stands() {
        let mut tracks = Tracks::default();
        turn(&mut tracks, 0.0, &[buff(0)]);
        turn(&mut tracks, 0.2, &[]);
        let fade = |s: [Option<Shown>; 2]| s[BUFFS].map(|s| s.fade).unwrap();
        assert!((fade(at(&mut tracks, 0.2)) - 0.4).abs() < 1e-5);
        assert!((fade(at(&mut tracks, 0.3)) - 0.2).abs() < 1e-5);
        turn(&mut tracks, 0.3, &[buff(0)]);
        assert!((fade(at(&mut tracks, 0.3)) - 0.2).abs() < 1e-5);
        assert!((fade(at(&mut tracks, 0.4)) - 0.4).abs() < 1e-5);
    }

    // §4 rule 5: a second effect blends the figure from the triangle to the square over 0.4 s.
    #[test]
    fn a_new_effect_blends_the_figure() {
        let mut tracks = Tracks::default();
        turn(&mut tracks, 0.0, &[buff(0)]);
        turn(&mut tracks, 5.0, &[buff(0), buff(3)]);
        let s = at(&mut tracks, 5.0)[BUFFS].unwrap();
        assert_eq!((s.from.slots(), s.ring.slots(), s.blend), (3, 4, 0.0));
        assert_eq!(s.fade, 1.0, "the circle itself stays");
        let s = at(&mut tracks, 5.2)[BUFFS].unwrap();
        assert!((s.blend - 0.5).abs() < 1e-5);
        let s = at(&mut tracks, 5.4)[BUFFS].unwrap();
        assert_eq!(s.blend, 1.0);
        assert_eq!(s.from, s.ring, "settled: its own figure");
        // And back, the other way.
        turn(&mut tracks, 9.0, &[buff(0)]);
        let s = at(&mut tracks, 9.0)[BUFFS].unwrap();
        assert_eq!((s.from.slots(), s.ring.slots()), (4, 3));
    }

    // §5: a hex alone is inside; a buff coming moves it out, and it moves back in only once the
    // buff's circle has gone.
    #[test]
    fn a_hexes_circle_moves_out_and_back_in() {
        let mut tracks = Tracks::default();
        turn(&mut tracks, 0.0, &[hex(4)]);
        let place = |s: [Option<Shown>; 2]| s[HEXES].unwrap().place;
        assert_eq!(place(at(&mut tracks, 1.0)), 0.0);
        turn(&mut tracks, 2.0, &[buff(0), hex(4)]);
        let s = at(&mut tracks, 2.0);
        assert_eq!(s[BUFFS].unwrap().fade, 0.0, "the buffs' circle fades in");
        assert_eq!(place(s), 0.0);
        assert!((place(at(&mut tracks, 2.2)) - 0.5).abs() < 1e-5);
        assert_eq!(place(at(&mut tracks, 2.4)), 1.0);
        assert_eq!(s[HEXES].unwrap().blend, 1.0, "the same figure: no blend");
        turn(&mut tracks, 5.0, &[hex(4)]);
        assert_eq!(
            place(at(&mut tracks, 5.3)),
            1.0,
            "outside while the buffs' goes"
        );
        let s = at(&mut tracks, 5.5);
        assert!(s[BUFFS].is_none(), "the buffs' gone");
        assert_eq!(place(s), 1.0, "it starts in");
        assert!((place(at(&mut tracks, 5.7)) - 0.5).abs() < 1e-5);
        assert_eq!(place(at(&mut tracks, 5.9)), 0.0, "and is in");
    }

    // §11: without animation every circle is settled, and one that goes is gone at once.
    #[test]
    fn without_animation_circles_are_still_and_cut() {
        let mut tracks = Tracks::default();
        let effects = [buff(0), hex(4)];
        tracks.update(3.0, &[(A, layout(effects))]);
        tracks.settle(3.0, false);
        let [b, h] = tracks.shown(A, 3.0, false, false);
        let (b, h) = (b.unwrap(), h.unwrap());
        assert_eq!((b.fade, b.blend, b.place, b.angle), (1.0, 1.0, 0.0, 0.0));
        assert_eq!((h.fade, h.place), (1.0, 1.0));
        assert_eq!(h.angle, std::f32::consts::PI, "the outer at a half turn");
        tracks.update(3.1, &[(A, layout([buff(0)]))]);
        tracks.settle(3.1, false);
        let [b, h] = tracks.shown(A, 3.1, false, false);
        assert!(b.is_some() && h.is_none(), "cut");
        // Frozen time turns the figure but completes the fades.
        let [b, _] = tracks.shown(A, 1.0, false, true);
        assert!((b.unwrap().angle - 6f32.to_radians()).abs() < 1e-5);
    }

    // An actor no longer seen loses its circles at once.
    #[test]
    fn an_unseen_actor_loses_its_circles() {
        let mut tracks = Tracks::default();
        turn(&mut tracks, 0.0, &[buff(0)]);
        tracks.update(1.0, &[]);
        assert!(!tracks.has(A));
    }
}
