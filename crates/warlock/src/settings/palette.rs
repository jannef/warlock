//! Color palettes (spec §4.7): a transform applied to colors when they are written to the
//! console, so content colors (terrain, creatures, UI constants) stay untouched.

use warlock_core::color::Rgb;

/// Whether a color is a glyph color or a cell background; high contrast pushes them apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Fg,
    Bg,
}

/// The selectable color palettes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Palette {
    #[default]
    Default,
    /// Brighter glyphs on darker backgrounds.
    HighContrast,
    /// Daltonized for red-green color blindness: the red-green difference is also carried by
    /// blue and luminance.
    Colorblind,
}

impl Palette {
    pub const ALL: [Palette; 3] = [Palette::Default, Palette::HighContrast, Palette::Colorblind];

    /// Name in the settings file.
    pub fn id(self) -> &'static str {
        match self {
            Palette::Default => "default",
            Palette::HighContrast => "high-contrast",
            Palette::Colorblind => "colorblind",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.id() == id)
    }

    /// Name shown in the settings screen.
    pub fn label(self) -> &'static str {
        match self {
            Palette::Default => "Default",
            Palette::HighContrast => "High contrast",
            Palette::Colorblind => "Colorblind (red-green)",
        }
    }

    /// The next palette in [`Palette::ALL`], wrapping; `back` goes the other way.
    pub fn cycle(self, back: bool) -> Self {
        let i = Self::ALL.iter().position(|p| *p == self).unwrap_or(0);
        let n = Self::ALL.len();
        Self::ALL[(i + if back { n - 1 } else { 1 }) % n]
    }

    /// The color to put on screen for `color`.
    pub fn apply(self, color: Rgb, role: Role) -> Rgb {
        match self {
            Palette::Default => color,
            Palette::HighContrast => {
                let exponent = if role == Role::Fg { 0.6 } else { 1.6 };
                map(color, |v| v.powf(exponent))
            }
            Palette::Colorblind => daltonize(color),
        }
    }
}

fn to_unit(v: u8) -> f32 {
    f32::from(v) / 255.0
}

fn to_byte(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn map(c: Rgb, f: impl Fn(f32) -> f32) -> Rgb {
    Rgb::new(
        to_byte(f(to_unit(c.r))),
        to_byte(f(to_unit(c.g))),
        to_byte(f(to_unit(c.b))),
    )
}

/// How a deuteranope sees `(r, g, b)` (unit sRGB; a coarse linear approximation).
fn simulate_deuteranopia([r, g, b]: [f32; 3]) -> [f32; 3] {
    [0.625 * r + 0.375 * g, 0.7 * r + 0.3 * g, 0.3 * g + 0.7 * b]
}

/// Moves what a red-green color-blind viewer loses into channels they can see.
fn daltonize(c: Rgb) -> Rgb {
    let rgb = [to_unit(c.r), to_unit(c.g), to_unit(c.b)];
    let seen = simulate_deuteranopia(rgb);
    let err = [rgb[0] - seen[0], rgb[1] - seen[1]];
    let shift = 0.7 * err[0] + err[1];
    Rgb::new(
        to_byte(rgb[0]),
        to_byte(rgb[1] + shift),
        to_byte(rgb[2] + shift),
    )
}

/// WCAG relative luminance.
#[cfg(test)]
fn luminance(c: Rgb) -> f32 {
    let lin = |v: u8| {
        let v = to_unit(v);
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
}

/// WCAG contrast ratio between two colors, 1 to 21.
#[cfg(test)]
pub fn contrast_ratio(a: Rgb, b: Rgb) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: Rgb = Rgb::new(220, 40, 40);
    const GREEN: Rgb = Rgb::new(40, 200, 40);

    #[test]
    fn default_palette_changes_nothing() {
        assert_eq!(Palette::Default.apply(RED, Role::Fg), RED);
        assert_eq!(Palette::Default.apply(GREEN, Role::Bg), GREEN);
    }

    #[test]
    fn high_contrast_separates_glyph_from_background() {
        let (fg, bg) = (Rgb::new(0x7c, 0x78, 0x70), Rgb::new(0x30, 0x30, 0x38));
        let before = contrast_ratio(fg, bg);
        let after = contrast_ratio(
            Palette::HighContrast.apply(fg, Role::Fg),
            Palette::HighContrast.apply(bg, Role::Bg),
        );
        assert!(after > before * 1.5, "{before} -> {after}");
        assert_eq!(
            Palette::HighContrast.apply(Rgb::BLACK, Role::Fg),
            Rgb::BLACK
        );
        assert_eq!(
            Palette::HighContrast.apply(Rgb::WHITE, Role::Bg),
            Rgb::WHITE
        );
    }

    #[test]
    fn colorblind_keeps_red_and_green_apart() {
        fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
            a.iter().zip(b).map(|(x, y)| (x - y).abs()).sum()
        }
        let unit = |c: Rgb| [to_unit(c.r), to_unit(c.g), to_unit(c.b)];
        let seen = |c: Rgb| simulate_deuteranopia(unit(Palette::Colorblind.apply(c, Role::Fg)));
        let before = distance(
            simulate_deuteranopia(unit(RED)),
            simulate_deuteranopia(unit(GREEN)),
        );
        let after = distance(seen(RED), seen(GREEN));
        assert!(after > before, "{before} -> {after}");
        // Grays have no red-green difference to carry.
        let gray = Rgb::new(120, 120, 120);
        assert_eq!(Palette::Colorblind.apply(gray, Role::Fg), gray);
    }

    #[test]
    fn ids_round_trip_and_cycle_wraps() {
        for p in Palette::ALL {
            assert_eq!(Palette::from_id(p.id()), Some(p));
        }
        assert_eq!(Palette::from_id("nope"), None);
        assert_eq!(Palette::Colorblind.cycle(false), Palette::Default);
        assert_eq!(Palette::Default.cycle(true), Palette::Colorblind);
    }

    #[test]
    fn contrast_ratio_matches_wcag_extremes() {
        assert!((contrast_ratio(Rgb::BLACK, Rgb::WHITE) - 21.0).abs() < 0.01);
        assert!((contrast_ratio(RED, RED) - 1.0).abs() < 1e-6);
    }
}
