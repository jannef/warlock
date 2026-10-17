//! The settings screen: font size, reduced effects, color palette (spec §4.7). Changes apply
//! live, because the screen edits the [`Settings`] resource that rendering reads.

use super::menu::{Nav, Panel, Press};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::render::console::Console;
use crate::settings::{MAX_FONT_PX, MIN_FONT_PX, Settings};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Row {
    FontSize,
    ReducedEffects,
    Palette,
    Reset,
    Back,
}

const ROWS: [Row; 5] = [
    Row::FontSize,
    Row::ReducedEffects,
    Row::Palette,
    Row::Reset,
    Row::Back,
];

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
                Row::Palette => settings.palette = settings.palette.cycle(step < 0),
                Row::Reset | Row::Back => {}
            },
            Nav::Enter if !press.repeat => match row {
                Row::FontSize => {}
                Row::ReducedEffects => settings.reduced_effects = !settings.reduced_effects,
                Row::Palette => settings.palette = settings.palette.cycle(false),
                Row::Reset => *settings = Settings::default(),
                Row::Back => return true,
            },
            _ => {}
        }
        false
    }

    pub fn draw(&self, console: &mut Console, settings: &Settings) {
        let panel = Panel::new(console, " Settings ", 60, 17);
        let label_w = 18;
        for (i, row) in ROWS.iter().enumerate() {
            let (label, value) = match row {
                Row::FontSize => ("Font size", format!("{} px", settings.font_px)),
                Row::ReducedEffects => (
                    "Reduced effects",
                    if settings.reduced_effects {
                        "On"
                    } else {
                        "Off"
                    }
                    .to_string(),
                ),
                Row::Palette => ("Color palette", settings.palette.label().to_string()),
                Row::Reset => ("Reset to defaults", String::new()),
                Row::Back => ("Back", String::new()),
            };
            // A gap before Reset and Back separates actions from options.
            let y = panel.y + 2 + i as i32 + i32::from(i >= 3);
            panel.row_with(console, y, i == self.selected, label, &value, label_w, true);
        }
        let help = match ROWS[self.selected] {
            Row::FontSize => "Size of the glyphs, in logical pixels.",
            Row::ReducedEffects => "No glyph animation, slides, or camera easing.",
            Row::Palette => "Recolors everything on screen; game data is unchanged.",
            Row::Reset => "Restores these settings (not the key bindings).",
            Row::Back => "Settings are saved as you change them.",
        };
        console.print(panel.inner_x(), panel.y + 10, help, TEXT_DIM);
        console.print(
            panel.inner_x(),
            panel.y + 12,
            "Sample: legible text, then  [ marked ]  and  * flagged *",
            TEXT,
        );
        console.print(panel.inner_x(), panel.y + 13, "Sample accent text", ACCENT);
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
        press(&mut screen, &mut s, Nav::Down);
        press(&mut screen, &mut s, Nav::Right);
        assert_eq!(s.palette, Palette::HighContrast);
        press(&mut screen, &mut s, Nav::Down);
        press(&mut screen, &mut s, Nav::Enter);
        assert_eq!(s, Settings::default());
        press(&mut screen, &mut s, Nav::Down);
        assert!(press(&mut screen, &mut s, Nav::Enter));
        assert!(press(&mut screen, &mut s, Nav::Back));
    }
}
