//! How a zone looks on the map (`warlock_core::world::zone`; docs/dev/rendering.md): a still tint
//! of its cells' background, no particles and no animation. Subtle: the owner's standing rule. A
//! fire's life is in the light it sheds, which flickers as any fire's does.

use warlock_core::color::Rgb;
use warlock_core::content::ZoneKind;

/// The colour a zone leans a cell's background toward, and how far, of 1.
const fn tint(kind: ZoneKind) -> (Rgb, u16) {
    match kind {
        ZoneKind::Fire => (Rgb::new(110, 40, 14), 28),
        ZoneKind::Fouled => (Rgb::new(58, 70, 22), 32),
        ZoneKind::Slick => (Rgb::new(60, 82, 94), 35),
        ZoneKind::Dark => (Rgb::new(10, 6, 16), 60),
        ZoneKind::Smoke => (Rgb::new(92, 92, 88), 40),
    }
}

/// `bg` under a zone of `kind`.
pub(super) fn tinted(bg: Rgb, kind: ZoneKind) -> Rgb {
    let (to, percent) = tint(kind);
    let mix = |from: u8, to: u8| {
        let (from, to) = (u16::from(from), u16::from(to));
        ((from * (100 - percent) + to * percent) / 100) as u8
    };
    Rgb::new(mix(bg.r, to.r), mix(bg.g, to.g), mix(bg.b, to.b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_leans_a_floor_toward_its_colour_and_only_so_far() {
        let floor = Rgb::new(20, 20, 20);
        for &kind in ZoneKind::ALL {
            let (to, percent) = tint(kind);
            assert!(percent <= 60, "subtle: {percent}");
            let got = tinted(floor, kind);
            assert_ne!(got, floor, "{kind:?}");
            assert!(got.r.abs_diff(to.r) < floor.r.abs_diff(to.r), "{kind:?}");
        }
    }
}
