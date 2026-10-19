//! The death screen (spec §6 "Death"): the run is over. Who died, where, when, and what killed
//! them, as far as the player knows it; the run's save is already gone (permadeath).
//!
//! It opens by itself when the player dies (`screens::on_death`), over the run as it ended, and
//! takes every key press: Enter or Esc returns to the main menu. It is read from the perception
//! snapshot, which carries the death once it has happened.

use super::menu::{Hints, Nav, Panel, Press, wrap};
use super::screens::{MenuChoice, ScreenAction};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::Bindings;
use crate::render::console::Console;
use crate::settings::Clock;
use warlock_core::content::article;
use warlock_core::perception::PerceptionView;
use warlock_core::time::GameTime;

#[derive(Debug)]
pub struct DeathScreen {
    /// The lines of the epitaph, in order; the empty one is the date and time of death.
    lines: Vec<String>,
    /// The date and time of death (the clock setting decides how the time is written).
    when: Option<(String, GameTime)>,
}

impl DeathScreen {
    /// The screen for the run's death; `None` (a run still going, or none) shows that nothing
    /// has died.
    pub fn new(view: Option<&PerceptionView<'_>>) -> Self {
        let Some((view, death)) = view.and_then(|v| v.death().map(|d| (v, d))) else {
            return Self {
                lines: vec!["Nothing has died.".into()],
                when: None,
            };
        };
        let content = view.content();
        let calendar = content.calendar();
        let by = match (death.of, death.cause, death.by) {
            // A status effect killed them: "Bled to death".
            (Some(status), _, _) => {
                let def = content.status(status);
                (def.death.clone())
                    .unwrap_or_else(|| format!("Died of {}", def.name.to_lowercase()))
            }
            // Hunger, overeating (spec §8 "Hunger"), or a trap: "Killed by a fireball trap".
            (None, Some(cause), _) => cause.epitaph(content),
            (None, None, Some(creature)) => {
                let name = &content.creature(creature).name;
                format!("Killed by {} {name}", article(name))
            }
            (None, None, None) => "Killed by something you did not see".into(),
        };
        let turns = match death.turn {
            1 => "after 1 turn".to_string(),
            n => format!("after {n} turns"),
        };
        Self {
            lines: vec![
                format!("{by},"),
                format!("in {},", death.place),
                String::new(),
                format!("{turns}."),
                format!("You had reached level {}.", view.own_experience().level),
            ],
            when: Some((calendar.full_date(death.time), death.time)),
        }
    }

    /// Applies a key press: Enter or Esc leaves the run for the main menu.
    pub fn handle(&mut self, press: &Press) -> ScreenAction {
        match press.nav {
            Nav::Enter | Nav::Back => ScreenAction::Choose(MenuChoice::LeaveRun),
            _ => ScreenAction::Stay,
        }
    }

    pub fn draw(&self, console: &mut Console, clock: Clock, bindings: &Bindings) {
        let w = 52;
        let lines = self.lines.iter().map(|line| match &self.when {
            Some((date, time)) if line.is_empty() => {
                format!("on {date}, at {},", clock.format(*time))
            }
            _ => line.clone(),
        });
        let text: Vec<String> = lines.flat_map(|l| wrap(&l, 44)).collect();
        // Rows: margin, the title and its rule, a gap, the epitaph, a gap, the save's fate, a
        // gap, the footer, frame.
        let h = 10 + text.len() as i32;
        let panel = Panel::new(console, "", w, h);
        let cx = |text: &str| panel.x + (panel.w - text.chars().count() as i32) / 2;
        let title = "Y O U   D I E D";
        // Rows from the footer's down would draw over it, or the frame, of a panel cut short.
        let fits = |y: i32| y < panel.y + panel.h - 2;
        let mut y = panel.y + 2;
        if fits(y) {
            console.print(cx(title), y, title, ACCENT);
        }
        y += 1;
        if fits(y) {
            let rule = "═".repeat(title.chars().count());
            console.print(cx(&rule), y, &rule, TEXT_DIM);
        }
        y += 2;
        for line in &text {
            if fits(y) {
                console.print(cx(line), y, line, TEXT);
            }
            y += 1;
        }
        y += 1;
        let gone = "Your run is over; its save is gone.";
        if fits(y) {
            console.print(cx(gone), y, gone, TEXT_DIM);
        }
        let footer = format!("{} or Esc  main menu", Hints::of(bindings).confirm);
        panel.footer(console, &footer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use warlock_core::content::Content;
    use warlock_core::{Action, World};

    fn text(console: &Console) -> String {
        (0..console.rows() as i32)
            .map(|y| {
                (0..console.cols() as i32)
                    .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn a_death_by_a_status_effect_says_so() {
        // One hit point (Toughness 1), and a bad wound.
        let mut files = warlock_core::content::ContentFiles::builtin();
        files.insert(
            "scenarios/frail.toml",
            "name = \"Frail\"\nstart_level = \"hub\"\nstats = { toughness = 1 }\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let mut w = World::new(content, "frail", None).expect("world");
        for _ in 0..5 {
            w.apply(&Action::Afflict("bleeding".into()));
        }
        while w.death().is_none() {
            w.apply(&Action::Wait);
        }
        let screen = DeathScreen::new(Some(&w.perception()));
        let mut console = Console::default();
        console.resize(80, 30);
        screen.draw(&mut console, Clock::H24, &Bindings::default());
        let shown = text(&console);
        assert!(shown.contains("Bled to death,"), "{shown}");
        assert!(!shown.contains("Killed by"), "{shown}");
    }

    #[test]
    fn it_says_who_died_where_when_and_by_what() {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut w = World::new(content, "arena", None).expect("world");
        w.apply(&Action::ToggleLantern);
        while w.death().is_none() {
            w.apply(&Action::Wait);
        }
        let screen = DeathScreen::new(Some(&w.perception()));
        let mut console = Console::default();
        console.resize(80, 30);
        screen.draw(&mut console, Clock::H24, &Bindings::default());
        let shown = text(&console);
        assert!(shown.contains("Y O U   D I E D"), "{shown}");
        assert!(shown.contains("in The Arena,"), "{shown}");
        assert!(shown.contains("Darkening, Year 1444, at 08:"), "{shown}");
        assert!(shown.contains("turns."), "{shown}");
        assert!(shown.contains("You had reached level 1."), "{shown}");
        // The archer shot from the dark: the player never saw it.
        assert!(
            shown.contains("Killed by something you did not see,"),
            "{shown}"
        );
        // Enter or Esc leaves the run; nothing else does.
        let mut screen = screen;
        assert_eq!(screen.handle(&Press::nav(Nav::Down)), ScreenAction::Stay);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter)),
            ScreenAction::Choose(MenuChoice::LeaveRun)
        );
        assert_eq!(
            screen.handle(&Press::nav(Nav::Back)),
            ScreenAction::Choose(MenuChoice::LeaveRun)
        );
    }

    #[test]
    fn it_says_when_hunger_killed_the_player() {
        // Starting with nothing to eat, never fainting, starving a hit point a second.
        let mut files = warlock_core::content::ContentFiles::builtin();
        let hunger = include_str!("../../../../assets/data/hunger.toml")
            .replace("start = 3600", "start = 0")
            .replace("chance = 5", "chance = 0")
            .replace("seconds = 600", "seconds = 1");
        files.insert("data/hunger.toml", &hunger);
        let content = Arc::new(Content::load(&files).expect("content"));
        let mut w = World::new(content, "showcase", None).expect("world");
        // Starving stops the first rest; the next lasts until the end.
        for _ in 0..3 {
            w.apply(&Action::Rest(warlock_core::Rest::Dusk));
        }
        assert!(w.death().is_some(), "starved");
        let screen = DeathScreen::new(Some(&w.perception()));
        let mut console = Console::default();
        console.resize(80, 30);
        screen.draw(&mut console, Clock::H24, &Bindings::default());
        let shown = text(&console);
        assert!(shown.contains("Starved to death,"), "{shown}");
        assert!(shown.contains("in The Hub,"), "{shown}");
    }
}
