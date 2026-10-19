//! Settings > Files: where the settings, saves, snapshots, and logs are for this launch, why
//! (portable or the platform's folders), and an Open folder action for each (spec §7). The
//! same facts as `warlock --paths`, from `paths.rs`.

use super::menu::{Nav, Panel, Press, wrap};
use super::screens::ScreenAction;
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::paths::{Entry, Locations, Reason};
use crate::render::console::Console;
use std::path::Path;

type Opener = fn(&Path) -> Result<(), String>;

/// Selection state of the Files screen.
#[derive(Debug)]
pub struct FilesScreen {
    mode: String,
    entries: Vec<Entry>,
    warnings: Vec<String>,
    browser: bool,
    selected: usize,
    /// What the last Open folder did.
    status: Option<String>,
    opener: Opener,
}

impl FilesScreen {
    pub fn new(locations: &Locations) -> Self {
        Self::with_opener(locations, crate::paths::open_folder)
    }

    fn with_opener(locations: &Locations, opener: Opener) -> Self {
        let browser = locations.config.reason == Reason::Browser;
        Self {
            mode: locations.mode_line(),
            entries: if browser {
                Vec::new()
            } else {
                locations.entries()
            },
            warnings: locations.warnings.clone(),
            browser,
            selected: 0,
            status: None,
            opener,
        }
    }

    /// Entries, then Back.
    fn rows(&self) -> usize {
        self.entries.len() + 1
    }

    pub fn handle(&mut self, press: &Press) -> ScreenAction {
        match press.nav {
            Nav::Back => return ScreenAction::Close,
            Nav::Up => self.selected = (self.selected + self.rows() - 1) % self.rows(),
            Nav::Down => self.selected = (self.selected + 1) % self.rows(),
            Nav::Enter if !press.repeat => match self.entries.get(self.selected) {
                None => return ScreenAction::Close,
                Some(entry) => {
                    self.status = Some(match &entry.dir {
                        None => format!("{} is not stored anywhere.", entry.label),
                        Some(dir) => match (self.opener)(dir) {
                            Ok(()) => format!("Opened {}.", dir.display()),
                            Err(e) => e,
                        },
                    });
                }
            },
            _ => {}
        }
        ScreenAction::Stay
    }

    pub fn draw(&self, console: &mut Console) {
        let panel = Panel::new(console, " Where are my files? ", 80, 31);
        let (x, w) = (panel.inner_x(), panel.inner_w());
        let mut y = panel.y + 2;
        let text = |console: &mut Console, y: &mut i32, s: &str, color| {
            for line in wrap(s, w) {
                console.print(x, *y, &line, color);
                *y += 1;
            }
        };
        text(console, &mut y, &self.mode, TEXT);
        if self.browser {
            y += 1;
            text(
                console,
                &mut y,
                "Settings and the saved run are kept under warlock/ keys in local storage. \
                 They survive reloads; clearing this site's data erases them. There are no \
                 folders to open.",
                TEXT_DIM,
            );
        }
        y += 1;
        for (i, e) in self.entries.iter().enumerate() {
            let selected = i == self.selected;
            let color = if selected { ACCENT } else { TEXT };
            console.print(x, y, if selected { ">" } else { " " }, ACCENT);
            console.print(x + 2, y, e.label, color);
            console.print(x + 14, y, e.what, TEXT_DIM);
            let place = match &e.dir {
                Some(dir) => shorten(&dir.display().to_string(), w - 4),
                None => "(not stored)".into(),
            };
            console.print(x + 2, y + 1, &place, color);
            console.print(x + 2, y + 2, &e.reason.describe(), TEXT_DIM);
            y += 4;
        }
        let back = self.entries.len();
        let color = if self.selected == back { ACCENT } else { TEXT };
        console.print(x, y, if self.selected == back { ">" } else { " " }, ACCENT);
        console.print(x + 2, y, "Back", color);
        y += 2;
        if let Some(status) = &self.status {
            text(console, &mut y, status, ACCENT);
        }
        for warning in &self.warnings {
            text(console, &mut y, warning, ACCENT);
        }
        let footer = if self.entries.is_empty() {
            "Esc back"
        } else {
            "↑↓ select   Enter open folder   Esc back"
        };
        panel.footer(console, footer);
    }
}

/// `text` cut to `width` cells, keeping its end (the interesting part of a path).
fn shorten(text: &str, width: i32) -> String {
    let (len, width) = (text.chars().count(), width.max(2) as usize);
    if len <= width {
        return text.to_string();
    }
    let tail: String = text.chars().skip(len - (width - 1)).collect();
    format!("…{tail}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::{Inputs, Platform, resolve};
    use std::path::PathBuf;
    use std::sync::Mutex;

    static OPENED: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

    fn record(dir: &Path) -> Result<(), String> {
        OPENED.lock().unwrap().push(dir.to_path_buf());
        Ok(())
    }

    fn refuse(_: &Path) -> Result<(), String> {
        Err("Cannot run xdg-open: not found.".into())
    }

    fn locations() -> Locations {
        let inputs = Inputs {
            platform: Some(Platform {
                config: "/p/config".into(),
                data: "/p/data".into(),
                logs: "/p/logs".into(),
            }),
            ..Inputs::default()
        };
        resolve(&inputs, &|_| false)
    }

    fn press(screen: &mut FilesScreen, nav: Nav) -> ScreenAction {
        screen.handle(&Press::nav(nav))
    }

    #[test]
    fn enter_opens_the_selected_folder_and_back_closes() {
        let mut screen = FilesScreen::with_opener(&locations(), record);
        press(&mut screen, Nav::Down);
        assert_eq!(press(&mut screen, Nav::Enter), ScreenAction::Stay);
        assert_eq!(
            OPENED.lock().unwrap().as_slice(),
            [PathBuf::from("/p/data")]
        );
        assert!(
            screen
                .status
                .as_deref()
                .unwrap()
                .starts_with("Opened /p/data")
        );
        for _ in 0..3 {
            press(&mut screen, Nav::Down);
        }
        assert_eq!(press(&mut screen, Nav::Enter), ScreenAction::Close);
        assert_eq!(press(&mut screen, Nav::Back), ScreenAction::Close);
    }

    #[test]
    fn a_failed_open_is_shown_not_fatal() {
        let mut screen = FilesScreen::with_opener(&locations(), refuse);
        assert_eq!(press(&mut screen, Nav::Enter), ScreenAction::Stay);
        assert_eq!(
            screen.status.as_deref(),
            Some("Cannot run xdg-open: not found.")
        );
    }

    #[test]
    fn the_web_has_nothing_to_open() {
        let inputs = Inputs {
            web: true,
            ..Inputs::default()
        };
        let mut screen = FilesScreen::with_opener(&resolve(&inputs, &|_| false), refuse);
        assert!(screen.browser && screen.entries.is_empty());
        assert_eq!(press(&mut screen, Nav::Enter), ScreenAction::Close);
    }

    #[test]
    fn long_paths_keep_their_end() {
        assert_eq!(shorten("/a/b", 10), "/a/b");
        assert_eq!(shorten("/very/long/path/here", 8), "…th/here");
    }
}
