//! The settings screen: font size, reduced effects and its per-effect switches, color palette
//! (spec §4.7). Changes apply
//! live, because the screen edits the [`Settings`] resource that rendering reads.

use super::menu::{Nav, Panel, Press};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::render::console::Console;
use crate::settings::{EffectSwitches, MAX_FONT_PX, MIN_FONT_PX, Settings};

/// One per-effect switch, listed under the reduced-effects preset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Effect {
    GlyphAnimation,
    Flicker,
    Motion,
    Glow,
    Distortion,
    Particles,
    ScreenFades,
    ScreenShake,
    Weather,
}

impl Effect {
    fn label(self) -> &'static str {
        match self {
            Effect::GlyphAnimation => "Glyph animation",
            Effect::Flicker => "Light flicker",
            Effect::Motion => "Slides, camera",
            Effect::Glow => "Bloom and glow",
            Effect::Distortion => "Heat haze",
            Effect::Particles => "Particles",
            Effect::ScreenFades => "Screen fades",
            Effect::ScreenShake => "Screen shake",
            Effect::Weather => "Weather",
        }
    }

    fn help(self) -> &'static str {
        match self {
            Effect::GlyphAnimation => "Water waves, pulsing altars, sounds rippling in.",
            Effect::Flicker => "Firelight flickers and drifts in hue; steady when off.",
            Effect::Motion => "Slides, the camera, and spell effects; off, they snap.",
            Effect::Glow => "Bloom around glowing glyphs such as lava and crystals.",
            Effect::Distortion => "Heat haze rising over lava and fires.",
            Effect::Particles => "Sparks, embers, splashes, and dust.",
            Effect::ScreenFades => "Level fades, flashes, bright spells darkening the view.",
            Effect::ScreenShake => "The view shakes when a spell lands near you.",
            Effect::Weather => "Rain falling, splashing, and rings on water.",
        }
    }

    fn switch(self, e: &mut EffectSwitches) -> &mut bool {
        match self {
            Effect::GlyphAnimation => &mut e.glyph_animation,
            Effect::Flicker => &mut e.flicker,
            Effect::Motion => &mut e.motion,
            Effect::Glow => &mut e.glow,
            Effect::Distortion => &mut e.distortion,
            Effect::Particles => &mut e.particles,
            Effect::ScreenFades => &mut e.screen_fades,
            Effect::ScreenShake => &mut e.screen_shake,
            Effect::Weather => &mut e.weather,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Row {
    FontSize,
    ReducedEffects,
    Effect(Effect),
    Palette,
    Clock,
    Reset,
    Back,
}

const ROWS: [Row; 15] = [
    Row::FontSize,
    Row::ReducedEffects,
    Row::Effect(Effect::GlyphAnimation),
    Row::Effect(Effect::Flicker),
    Row::Effect(Effect::Motion),
    Row::Effect(Effect::Glow),
    Row::Effect(Effect::Distortion),
    Row::Effect(Effect::Particles),
    Row::Effect(Effect::ScreenFades),
    Row::Effect(Effect::ScreenShake),
    Row::Effect(Effect::Weather),
    Row::Palette,
    Row::Clock,
    Row::Reset,
    Row::Back,
];

/// Index of the first action row (Reset); a gap is drawn before it.
const FIRST_ACTION: usize = 13;

/// Selection state of the settings screen.
#[derive(Debug, Default)]
pub struct SettingsScreen {
    selected: usize,
}

impl SettingsScreen {
    /// Applies a key press to `settings`. Returns true when the player leaves the screen.
    pub fn handle(&mut self, press: &Press, settings: &mut Settings) -> bool {
        let row = ROWS[self.selected];
        let step: i32 = match press.nav {
            Nav::Left => -1,
            Nav::Right => 1,
            _ => 0,
        };
        match press.nav {
            Nav::Back => return true,
            Nav::Up => self.selected = (self.selected + ROWS.len() - 1) % ROWS.len(),
            Nav::Down => self.selected = (self.selected + 1) % ROWS.len(),
            Nav::Left | Nav::Right => match row {
                Row::FontSize => {
                    let px = settings.font_px as i32 + step;
                    settings.font_px = px.clamp(MIN_FONT_PX as i32, MAX_FONT_PX as i32) as u32;
                }
                Row::ReducedEffects => settings.reduced_effects = !settings.reduced_effects,
                Row::Effect(e) => {
                    let on = e.switch(&mut settings.effects);
                    *on = !*on;
                }
                Row::Palette => settings.palette = settings.palette.cycle(step < 0),
                Row::Clock => settings.clock = settings.clock.toggle(),
                Row::Reset | Row::Back => {}
            },
            Nav::Enter if !press.repeat => match row {
                Row::FontSize => {}
                Row::ReducedEffects => settings.reduced_effects = !settings.reduced_effects,
                Row::Effect(e) => {
                    let on = e.switch(&mut settings.effects);
                    *on = !*on;
                }
                Row::Palette => settings.palette = settings.palette.cycle(false),
                Row::Clock => settings.clock = settings.clock.toggle(),
                Row::Reset => *settings = Settings::default(),
                Row::Back => return true,
            },
            _ => {}
        }
        false
    }

    pub fn draw(&self, console: &mut Console, settings: &Settings) {
        let panel = Panel::new(console, " Settings ", 60, 25);
        let label_w = 18;
        let mut switches = settings.effects;
        for (i, row) in ROWS.iter().enumerate() {
            let on_off = |on: bool| if on { "On" } else { "Off" }.to_string();
            let (label, value) = match row {
                Row::FontSize => ("Font size".to_string(), format!("{} px", settings.font_px)),
                Row::ReducedEffects => (
                    "Reduced effects".to_string(),
                    on_off(settings.reduced_effects),
                ),
                Row::Effect(e) => {
                    let own = *e.switch(&mut switches);
                    // The preset overrides every switch; say so rather than show a lie.
                    let value = if own && settings.reduced_effects {
                        "Off (preset)".to_string()
                    } else {
                        on_off(own)
                    };
                    (format!("  {}", e.label()), value)
                }
                Row::Palette => (
                    "Color palette".to_string(),
                    settings.palette.label().to_string(),
                ),
                Row::Clock => ("Clock".to_string(), settings.clock.label().to_string()),
                Row::Reset => ("Reset to defaults".to_string(), String::new()),
                Row::Back => ("Back".to_string(), String::new()),
            };
            // A gap before Reset and Back separates actions from options.
            let y = panel.y + 2 + i as i32 + i32::from(i >= FIRST_ACTION);
            panel.row_with(
                console,
                y,
                i == self.selected,
                &label,
                &value,
                label_w,
                true,
            );
        }
        let help = match ROWS[self.selected] {
            Row::FontSize => "Size of the glyphs, in logical pixels.",
            Row::ReducedEffects => "Preset: turns every effect below off.",
            Row::Effect(e) => e.help(),
            Row::Palette => "Recolors everything on screen; game data is unchanged.",
            Row::Clock => "How the time of day is written: 14:32 or 2:32 PM.",
            Row::Reset => "Restores these settings (not the key bindings).",
            Row::Back => "Settings are saved as you change them.",
        };
        let help_y = panel.y + 18;
        console.print(panel.inner_x(), help_y, help, TEXT_DIM);
        console.print(
            panel.inner_x(),
            help_y + 1,
            "Sample: legible text, then  [ marked ]  and  * flagged *",
            TEXT,
        );
        console.print(panel.inner_x(), help_y + 2, "Sample accent text", ACCENT);
        panel.footer(console, "↑↓ select   ←→ change   Esc back");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Palette;

    fn press(screen: &mut SettingsScreen, settings: &mut Settings, nav: Nav) -> bool {
        screen.handle(&Press::nav(nav), settings)
    }

    #[test]
    fn font_size_steps_and_clamps() {
        let (mut screen, mut s) = (SettingsScreen::default(), Settings::default());
        press(&mut screen, &mut s, Nav::Right);
        assert_eq!(s.font_px, 19);
        s.font_px = MAX_FONT_PX;
        press(&mut screen, &mut s, Nav::Right);
        assert_eq!(s.font_px, MAX_FONT_PX);
        s.font_px = MIN_FONT_PX;
        press(&mut screen, &mut s, Nav::Left);
        assert_eq!(s.font_px, MIN_FONT_PX);
    }

    #[test]
    fn toggles_cycle_and_reset() {
        let (mut screen, mut s) = (SettingsScreen::default(), Settings::default());
        press(&mut screen, &mut s, Nav::Down);
        press(&mut screen, &mut s, Nav::Enter);
        assert!(s.reduced_effects);
        for _ in 0..10 {
            press(&mut screen, &mut s, Nav::Down);
        }
        press(&mut screen, &mut s, Nav::Right);
        assert_eq!(s.palette, Palette::HighContrast);
        press(&mut screen, &mut s, Nav::Down);
        press(&mut screen, &mut s, Nav::Right);
        assert_eq!(s.clock, crate::settings::Clock::H12);
        press(&mut screen, &mut s, Nav::Down);
        press(&mut screen, &mut s, Nav::Enter);
        assert_eq!(s, Settings::default());
        press(&mut screen, &mut s, Nav::Down);
        assert!(press(&mut screen, &mut s, Nav::Enter));
        assert!(press(&mut screen, &mut s, Nav::Back));
    }

    #[test]
    fn effect_rows_flip_their_own_switch() {
        let (mut screen, mut s) = (SettingsScreen::default(), Settings::default());
        // Down past the font size and the preset: the first effect is glyph animation.
        for _ in 0..2 {
            press(&mut screen, &mut s, Nav::Down);
        }
        press(&mut screen, &mut s, Nav::Enter);
        assert!(!s.effects.glyph_animation && s.effects.particles);
        for _ in 0..5 {
            press(&mut screen, &mut s, Nav::Down);
        }
        press(&mut screen, &mut s, Nav::Left);
        assert!(!s.effects.particles && s.effects.glow);
        // Past the screen fades: the screen shake, then the weather.
        for _ in 0..2 {
            press(&mut screen, &mut s, Nav::Down);
        }
        press(&mut screen, &mut s, Nav::Enter);
        assert!(!s.effects.screen_shake && s.effects.screen_fades && s.effects.weather);
        press(&mut screen, &mut s, Nav::Down);
        press(&mut screen, &mut s, Nav::Enter);
        assert!(!s.effects.weather);
        assert!(!s.reduced_effects, "the preset is untouched");
    }
}
