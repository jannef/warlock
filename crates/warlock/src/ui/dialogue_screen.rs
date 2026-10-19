//! The dialogue screen (spec §6 "Dialogue"): chatting (`C`) with someone who has topics opens it
//! over the run. It shows who the player talks to, what they said last, and the topics to ask
//! about, numbered: a digit (or the arrows and Enter) asks, which takes no time; Esc (or the chat
//! key) leaves.
//!
//! It is built from the last conversation the rules report ([`Event::Talked`]), so it never
//! shows more than the player was told. Asking closes it and hands the question to the
//! simulation ([`ScreenAction::Act`]); the answer reopens it (`screens::converse`) with the
//! reply, on the topic just asked.

use super::menu::{Nav, Panel, Press, wrap};
use super::screens::ScreenAction;
use super::{ACCENT, TEXT};
use crate::input::{Bindings, KeyCommand};
use crate::render::console::Console;
use warlock_core::perception::PerceptionView;
use warlock_core::world::Conversation;
use warlock_core::{Action, Dir, Event, Outcome};

/// The panel's width.
const WIDTH: i32 = 64;

#[derive(Debug)]
pub struct DialogueScreen {
    /// Whom the player talks to, as a title: "The old hermit".
    name: String,
    /// The way to them from the player; `None` when there is no one to talk to.
    toward: Option<Dir>,
    /// What they said last.
    line: String,
    /// Topic ids and names, in order.
    topics: Vec<(String, String)>,
    /// The selected topic.
    selected: usize,
}

impl DialogueScreen {
    /// The screen for `talk` (the run's last conversation, if any) as the player perceives the
    /// run: the speaker must be in view, next to them.
    pub fn new(talk: Option<&Conversation>, view: Option<&PerceptionView<'_>>) -> Self {
        let nobody = || Self {
            name: String::new(),
            toward: None,
            line: "There is no one to talk to.".into(),
            topics: Vec::new(),
            selected: 0,
        };
        let (Some(talk), Some(view)) = (talk, view) else {
            return nobody();
        };
        let Some(actor) = view.actors().iter().find(|a| a.id == talk.with) else {
            return nobody();
        };
        let me = view.viewer();
        let toward = Dir::from_delta(actor.pos.x - me.x, actor.pos.y - me.y);
        let name = &view.content().creature(actor.creature).name;
        let topics: Vec<(String, String)> = (talk.topics.iter())
            .map(|t| (t.id.clone(), t.name.clone()))
            .collect();
        let selected = (talk.asked.as_ref())
            .and_then(|asked| topics.iter().position(|(id, _)| id == asked))
            .unwrap_or(0);
        Self {
            name: capitalized(&format!("the {name}")),
            toward,
            line: talk.line.clone(),
            topics,
            selected,
        }
    }

    /// Applies a key press: a topic's number (or Enter on the selected one) asks about it, the
    /// arrows move the selection, Esc or the chat key leaves.
    pub fn handle(&mut self, press: &Press, bindings: &Bindings) -> ScreenAction {
        let own_key = (press.binding.as_ref())
            .is_some_and(|key| bindings.owner(key) == Some(KeyCommand::Chat));
        if own_key || press.nav == Nav::Back {
            return ScreenAction::Close;
        }
        let n = self.topics.len();
        let number = press
            .text
            .and_then(|c| c.to_digit(10))
            .and_then(|d| (d as usize).checked_sub(1))
            .filter(|&i| i < n);
        match (number, press.nav) {
            (Some(i), _) => self.ask(i),
            (None, Nav::Up) if n > 0 => {
                self.selected = (self.selected + n - 1) % n;
                ScreenAction::Stay
            }
            (None, Nav::Down) if n > 0 => {
                self.selected = (self.selected + 1) % n;
                ScreenAction::Stay
            }
            (None, Nav::Enter) if n > 0 => self.ask(self.selected),
            (None, Nav::Enter) => ScreenAction::Close,
            _ => ScreenAction::Stay,
        }
    }

    /// Asks about topic `i`: the screen closes, the rules answer, and the answer reopens it.
    fn ask(&mut self, i: usize) -> ScreenAction {
        self.selected = i;
        match self.toward {
            Some(dir) => ScreenAction::Act(Action::Ask {
                dir,
                topic: self.topics[i].0.clone(),
            }),
            None => ScreenAction::Close,
        }
    }

    pub fn draw(&self, console: &mut Console) {
        let width = WIDTH.min(console.cols() as i32) - 4;
        let said = wrap(&format!("\"{}\"", self.line), width);
        // Rows: margin, what was said, a gap, the topics (and a gap), the footer, the frame.
        let topic_rows = match self.topics.len() {
            0 => 0,
            n => n as i32 + 1,
        };
        let h = said.len() as i32 + topic_rows + 5;
        let title = if self.name.is_empty() {
            String::new()
        } else {
            format!(" {} ", self.name)
        };
        let panel = Panel::new(console, &title, WIDTH, h);
        let x = panel.inner_x();
        // Rows from the footer's down would draw over it, or the frame, of a panel cut short.
        let fits = |y: i32| y < panel.y + panel.h - 2;
        let mut y = panel.y + 2;
        for line in &said {
            if fits(y) {
                console.print(x, y, line, TEXT);
            }
            y += 1;
        }
        y += 1;
        for (i, (_, name)) in self.topics.iter().enumerate() {
            if fits(y) {
                let selected = i == self.selected;
                let color = if selected { ACCENT } else { TEXT };
                if selected {
                    console.print(x, y, ">", ACCENT);
                }
                console.print(x + 2, y, &format!("{}  {name}", i + 1), color);
            }
            y += 1;
        }
        let footer = match self.topics.len() {
            0 => "Esc  leave".to_string(),
            1 => "1 or Enter  ask    Esc  leave".to_string(),
            n => format!("1-{n} or Enter  ask    Esc  leave"),
        };
        panel.footer(console, &footer);
    }
}

/// The run's last conversation, if its last action was talk.
pub fn last_talk(outcome: &Outcome) -> Option<&Conversation> {
    outcome.events.iter().rev().find_map(|e| match e {
        Event::Talked(talk) => Some(talk),
        _ => None,
    })
}

/// `text` with its first letter in upper case.
fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use warlock_core::World;
    use warlock_core::action::parse_actions;
    use warlock_core::content::Content;

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

    /// The showcase, next to the old hermit in the stairs nook, having chatted with him.
    fn at_the_hermit() -> (World, Outcome) {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut world = World::new(content, "showcase", None).expect("run");
        let walk = "move:N*3 open:N move:N*2 move:E*67 open:S move:S*4 move:SE*3";
        for action in parse_actions(walk).expect("actions") {
            world.apply(&action);
        }
        let outcome = world.apply(&Action::Chat(Dir::SE));
        (world, outcome)
    }

    #[test]
    fn it_shows_who_talks_what_they_said_and_the_topics() {
        let (world, outcome) = at_the_hermit();
        let view = world.perception();
        let screen = DialogueScreen::new(last_talk(&outcome), Some(&view));
        let mut console = Console::default();
        console.resize(80, 30);
        screen.draw(&mut console);
        let shown = text(&console);
        assert!(shown.contains(" The old hermit "), "{shown}");
        assert!(shown.contains("\"Hm? A visitor."), "{shown}");
        assert!(shown.contains("> 1  This hall"), "{shown}");
        assert!(shown.contains("  5  These stairs"), "{shown}");
        assert!(shown.contains("1-5 or Enter  ask"), "{shown}");
    }

    #[test]
    fn numbers_and_enter_ask_arrows_choose_and_esc_leaves() {
        let (world, outcome) = at_the_hermit();
        let view = world.perception();
        let mut screen = DialogueScreen::new(last_talk(&outcome), Some(&view));
        let bindings = Bindings::default();
        let ask = |topic: &str| {
            ScreenAction::Act(Action::Ask {
                dir: Dir::SE,
                topic: topic.into(),
            })
        };
        let mut press = |p: Press| screen.handle(&p, &bindings);
        assert_eq!(press(Press::typed('2')), ask("dark"));
        assert_eq!(press(Press::typed('9')), ScreenAction::Stay);
        assert_eq!(press(Press::nav(Nav::Up)), ScreenAction::Stay);
        assert_eq!(press(Press::nav(Nav::Enter)), ask("rooms"));
        assert_eq!(press(Press::nav(Nav::Up)), ScreenAction::Stay);
        assert_eq!(press(Press::nav(Nav::Enter)), ask("stairs"));
        assert_eq!(press(Press::nav(Nav::Back)), ScreenAction::Close);
    }

    #[test]
    fn the_answer_reopens_it_on_the_topic_asked() {
        let (mut world, _) = at_the_hermit();
        let outcome = world.apply(&Action::Ask {
            dir: Dir::SE,
            topic: "water".into(),
        });
        let view = world.perception();
        let screen = DialogueScreen::new(last_talk(&outcome), Some(&view));
        let mut console = Console::default();
        console.resize(80, 30);
        screen.draw(&mut console);
        let shown = text(&console);
        assert!(
            shown.contains("\"A spring runs out of the east wall"),
            "{shown}"
        );
        assert!(shown.contains("> 4  The water garden"), "{shown}");
    }

    #[test]
    fn without_a_conversation_there_is_no_one_to_talk_to() {
        let mut screen = DialogueScreen::new(None, None);
        let mut console = Console::default();
        console.resize(80, 30);
        screen.draw(&mut console);
        assert!(text(&console).contains("There is no one to talk to."));
        let bindings = Bindings::default();
        let action = screen.handle(&Press::nav(Nav::Enter), &bindings);
        assert_eq!(action, ScreenAction::Close);
    }
}
