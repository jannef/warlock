//! The level-up screen (spec §6 "Level-up", §8 "Talents"; #236): it opens by itself at the end
//! of the turn in which a level came (`screens::on_level_up`; the death screen wins) and walks
//! that level's choices, one step at a time, skipping steps with nothing to choose: the **skill
//! rolls** (the trained skills, each with its value and its meter), then a **bonus pick** of the
//! class's trees, then the level's **talent** (the talents that may be taken now, grouped by
//! tree). Each step is the one paged list (`paged_list`); a letter, or Enter on the `>` row, is
//! the choice's action (`advance:stealth`, `talent:hardy`), applied at once: the screen closes
//! to apply it and opens again on what is left, so the rules, replays, and dumps see the same
//! steps. A talent with several choices (Weapon focus) lists them in its place first; Esc there
//! returns to the talents.
//!
//! **Nothing is banked**: while a choice is due the screen cannot be left. Esc only says what is
//! left to choose; when the last choice is made it is gone. Several levels gained in one turn
//! are one sequence: the frame names the level whose choices are shown, and the levels after it.
//! Words only: never a die, a mark, or a chance. It reads the perception snapshot.

use super::character_screen::fit_joined;
use super::menu::{Hints, Nav, Panel, Press};
use super::paged_list::{PagedList, Row, draw_header};
use super::screens::ScreenAction;
use super::skills_screen::meter;
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::Bindings;
use crate::render::console::Console;
use warlock_core::Action;
use warlock_core::content::TreeId;
use warlock_core::perception::PerceptionView;

/// Width of the panel.
const WIDTH: i32 = 76;
/// Rows besides the list: the frame, a margin, a gap, the description (three lines), a gap, the
/// footer, the frame.
const EXTRA: i32 = 9;

/// What a step chooses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    /// Skill rolls: a letter rolls one on that skill.
    Rolls,
    /// A bonus pick, of the class's trees.
    Bonus,
    /// The level's talent pick.
    Talent,
}

/// How the list groups its rows: the skills under one header, talents by tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Group {
    Skills,
    Tree(TreeId),
}

/// One row: a skill to roll, a talent to take, or one of a talent's choices.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    name: String,
    /// Beside the name: a skill's value and meter, a ranked talent's next rank ("2/3").
    detail: String,
    description: String,
    /// What choosing it does; `None` for a talent whose choices are listed next.
    action: Option<Action>,
    /// A talent's choices (name, action) when it has more than one way to be taken.
    choices: Vec<(String, Action)>,
}

#[derive(Debug)]
pub struct LevelUpScreen {
    /// The frame's title: the level, and what is left of its choices.
    title: String,
    /// The levels after this one that have choices too, in words ("Level 5 next").
    after: Option<String>,
    step: Step,
    /// What is left to choose, in the words Esc says.
    left: String,
    entries: PagedList<Entry, Group>,
    /// A talent's choices, listed in the talents' place: its name and the list.
    choosing: Option<(String, PagedList<Entry, Group>)>,
    /// Why the screen stays: shown in place of the description until the next press.
    notice: Option<String>,
}

impl LevelUpScreen {
    /// The screen for the choices due now (`None` outside a run, or with nothing due: empty;
    /// the stack does not open it then).
    pub fn new(view: Option<&PerceptionView<'_>>) -> Self {
        let mut screen = Self {
            title: " Level up ".into(),
            after: None,
            step: Step::Talent,
            left: String::new(),
            entries: PagedList::empty(),
            choosing: None,
            notice: None,
        };
        let Some(view) = view else {
            return screen;
        };
        let Some(due) = view.due() else {
            return screen;
        };
        let content = view.content();
        let skills = !view.own_skills().is_empty();
        screen.step = match () {
            () if due.rolls > 0 && skills => Step::Rolls,
            () if due.bonus > 0 => Step::Bonus,
            () => Step::Talent,
        };
        let mut parts = vec![format!("Level {}", due.level)];
        if due.rolls > 0 {
            parts.push(format!("Skill rolls {}", due.rolls));
        }
        if due.bonus > 0 {
            parts.push(format!("Bonus talent {}", due.bonus));
        }
        if due.talents > 0 {
            parts.push(format!("Talent {}", due.talents));
        }
        screen.title = format!(" {} ", parts.join(" · "));
        let later: Vec<String> = (view.dues().iter().skip(1))
            .map(|d| d.level.to_string())
            .collect();
        screen.after = match later.len() {
            0 => None,
            1 => Some(format!("Level {} next", later[0])),
            _ => Some(format!("Levels {} next", later.join(", "))),
        };
        screen.left = match screen.step {
            Step::Rolls if due.rolls == 1 => "You have a skill roll to spend.".into(),
            Step::Rolls => format!("You have {} skill rolls to spend.", due.rolls),
            Step::Bonus | Step::Talent => "You must choose a talent first.".into(),
        };
        let book = content.talents();
        let items: Vec<(Group, Entry)> = match screen.step {
            Step::Rolls => (view.own_skills().iter())
                .map(|s| {
                    let def = content.skill(s.skill);
                    let entry = Entry {
                        name: def.name.clone(),
                        detail: format!("{:>3}  {}", s.value, meter(s.progress)),
                        description: def.description.clone(),
                        action: Some(Action::Advance(def.id.clone())),
                        choices: Vec::new(),
                    };
                    (Group::Skills, entry)
                })
                .collect(),
            Step::Bonus | Step::Talent => {
                let trees = (screen.step == Step::Bonus)
                    .then(|| view.bonus_trees())
                    .flatten();
                (view.talents().into_iter())
                    .filter(|t| !t.openings.is_empty())
                    .filter(|t| trees.as_ref().is_none_or(|trees| trees.contains(&t.tree)))
                    .map(|t| {
                        let def = book.get(t.id);
                        let take = |choice: &Option<String>| Action::Talent {
                            id: def.id.clone(),
                            choice: choice.clone(),
                        };
                        let choices: Vec<(String, Action)> = match t.choice {
                            Some(kind) => (t.openings.iter())
                                .map(|c| {
                                    let id = c.as_deref().unwrap_or_default();
                                    let name = kind.name(content, id).unwrap_or(id);
                                    (name.to_string(), take(c))
                                })
                                .collect(),
                            None => Vec::new(),
                        };
                        let (action, choices) = match (choices.len(), t.choice) {
                            (1, _) => (Some(choices[0].1.clone()), Vec::new()),
                            (_, Some(_)) => (None, choices),
                            (_, None) => (Some(take(&None)), Vec::new()),
                        };
                        let detail = match t.ranks {
                            1 => String::new(),
                            n => format!("{}/{n}", t.held + 1),
                        };
                        let name = match &action {
                            Some(Action::Talent {
                                choice: Some(c), ..
                            }) => def.name_for(content, Some(c)),
                            _ => t.name.clone(),
                        };
                        let entry = Entry {
                            name,
                            detail,
                            description: t.description.clone(),
                            action,
                            choices,
                        };
                        (Group::Tree(t.tree), entry)
                    })
                    .collect()
            }
        };
        screen.entries = PagedList::grouped(items, |group| match group {
            Group::Skills => "Skills".to_string(),
            Group::Tree(tree) => book.tree(tree).name.clone(),
        });
        screen
    }

    /// The list shown: a talent's choices while they are listed, else the step's.
    fn list(&self) -> &PagedList<Entry, Group> {
        self.choosing
            .as_ref()
            .map_or(&self.entries, |(_, list)| list)
    }

    fn list_mut(&mut self) -> &mut PagedList<Entry, Group> {
        match &mut self.choosing {
            Some((_, list)) => list,
            None => &mut self.entries,
        }
    }

    /// Applies a key press: a letter on the page shown, or Enter on the `>` row, makes that
    /// choice; ↑↓ and the list keys move; Esc leaves a talent's choices, and otherwise only says
    /// what must be chosen first (nothing is banked).
    pub fn handle(&mut self, press: &Press) -> ScreenAction {
        self.notice = None;
        if press.nav == Nav::Back {
            match self.choosing.take() {
                Some(_) => {}
                None => self.notice = Some(self.left.clone()),
            }
            return ScreenAction::Stay;
        }
        if let Some(c) = press.text.filter(char::is_ascii_lowercase) {
            return match self.list().at_letter(c) {
                Some(index) => self.choose(index),
                None => ScreenAction::Stay,
            };
        }
        match press.nav {
            Nav::Enter if self.list().is_empty() => ScreenAction::Stay,
            Nav::Enter => self.choose(self.list().cursor()),
            nav => {
                self.list_mut().navigate(nav);
                ScreenAction::Stay
            }
        }
    }

    /// Chooses row `index`: its action, or the talent's choices in the talents' place.
    fn choose(&mut self, index: usize) -> ScreenAction {
        self.list_mut().select(index);
        let Some(entry) = self.list().get(index).cloned() else {
            return ScreenAction::Stay;
        };
        if let Some(action) = entry.action {
            return ScreenAction::Act(action);
        }
        // One group, headed by the talent's name.
        let group = Group::Skills;
        let items = (entry.choices.iter())
            .map(|(name, action)| {
                let choice = Entry {
                    name: name.clone(),
                    detail: String::new(),
                    description: entry.description.clone(),
                    action: Some(action.clone()),
                    choices: Vec::new(),
                };
                (group, choice)
            })
            .collect();
        let title = entry.name.clone();
        let list = PagedList::grouped(items, move |_| title.clone());
        self.choosing = Some((entry.name, list));
        ScreenAction::Stay
    }

    /// Lays the list out for a console of `cols` by `rows` (drawing calls it, and the stack
    /// before a press, so a letter names what is drawn).
    pub fn fit(&self, _cols: u32, rows: u32) {
        let room = (rows as i32 - EXTRA).max(1) as usize;
        self.entries.set_room(room);
        if let Some((_, list)) = &self.choosing {
            list.set_room(room);
        }
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        self.fit(console.cols(), console.rows());
        let list = self.list();
        let count = list.tallest().max(1) as i32;
        let panel = Panel::new(console, &self.title, WIDTH, count + EXTRA);
        list.draw_label(console, &panel);
        let (x, top) = (panel.inner_x(), panel.y + 2);
        let fits = |y: i32| y < panel.y + panel.h - 2;
        if list.is_empty() && fits(top) {
            console.print(x + 2, top, "There is nothing to choose.", TEXT_DIM);
        }
        let name_w = (list.iter().map(|e| e.name.chars().count()).max()).unwrap_or(0) as i32;
        let cursor = list.cursor();
        let visible = (panel.h - EXTRA).max(1) as usize;
        for (row, y) in list.rows().into_iter().take(visible).zip(top..) {
            if !fits(y) {
                break;
            }
            let (index, letter) = match row {
                Row::Header(name) => {
                    draw_header(console, x, y, name);
                    continue;
                }
                Row::Item { index, letter } => (index, letter),
            };
            let Some(entry) = list.get(index) else {
                continue;
            };
            let selected = index == cursor;
            let color = if selected { ACCENT } else { TEXT };
            if selected {
                console.print(x, y, ">", ACCENT);
            }
            console.print(x + 2, y, &format!("{letter}  {}", entry.name), color);
            let detail_x = x + 5 + name_w + 2;
            let room = (panel.x + panel.w - 2 - detail_x).max(0) as usize;
            // A talent's rank, then the start of what it does, as room allows.
            let detail = match self.step {
                Step::Rolls => entry.detail.clone(),
                _ if self.choosing.is_some() => String::new(),
                _ => format!("{:<4} {}", entry.detail, entry.description),
            };
            let detail: String = match detail.chars().count() > room && room > 1 {
                true => detail.chars().take(room - 1).collect::<String>() + "…",
                false => detail,
            };
            console.print(detail_x, y, detail.trim_end(), TEXT_DIM);
        }
        // Below the list: why the screen stays, else what the selected row is.
        let (text, color) = match (&self.notice, list.selected()) {
            (Some(notice), _) => (notice.clone(), ACCENT),
            (None, Some(entry)) => (entry.description.clone(), TEXT_DIM),
            (None, None) => (String::new(), TEXT_DIM),
        };
        let words: Vec<String> = text.split(' ').map(String::from).collect();
        let lines = fit_joined(&words, " ", panel.inner_w());
        for (line, y) in lines.iter().take(3).zip(panel.y + panel.h - 6..) {
            if fits(y) && y >= top + count {
                console.print(x, y, line, color);
            }
        }
        panel.footer(console, &self.footer(bindings));
    }

    /// The footer: the letters of the page shown and Enter, the page keys while there are
    /// pages, the levels still to come, and Esc's meaning on a talent's choices.
    fn footer(&self, bindings: &Bindings) -> String {
        let keys = Hints::of(bindings);
        let list = self.list();
        let verb = match (self.step, &self.choosing) {
            (Step::Rolls, _) => "roll",
            (_, Some(_)) | (_, None) => "choose",
        };
        let confirm = &keys.confirm;
        let letters = match list.last_letter() {
            Some('a') => format!("a or {confirm} {verb}"),
            Some(last) => format!("a-{last} or {confirm} {verb}"),
            None => String::new(),
        };
        let mut parts = vec![letters];
        if let Some(hint) = list.hint(&keys) {
            parts.push(hint);
        }
        if self.choosing.is_some() {
            parts.push("Esc back".into());
        } else if let Some(after) = &self.after {
            parts.push(after.clone());
        }
        parts.retain(|p| !p.is_empty());
        parts.join("   ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use warlock_core::World;
    use warlock_core::action::parse_actions;
    use warlock_core::content::Content;

    /// A human warrior of the showcase brought to level 3 (skill rolls and a talent due).
    fn levelled() -> World {
        let content = Arc::new(Content::builtin().expect("content"));
        let spec = "human,male,warrior".parse().expect("a character");
        let mut world =
            World::start_run(content, "showcase", Some(1), Some(&spec), None, true).expect("world");
        for action in parse_actions("xp:691").expect("actions") {
            world.apply(&action);
        }
        world
    }

    fn text(screen: &LevelUpScreen) -> String {
        let mut console = Console::default();
        console.resize(80, 40);
        screen.draw(&mut console, &Bindings::default());
        (0..console.rows() as i32)
            .map(|y| {
                (0..console.cols() as i32)
                    .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Spends the rolls due now with `action`.
    fn spend_rolls(world: &mut World, action: &Action) {
        while world.due().is_some_and(|d| d.rolls > 0) {
            world.apply(action);
        }
    }

    #[test]
    fn rolls_come_first_then_the_talent_and_esc_never_leaves() {
        let mut world = levelled();
        // Levels 2 and 3 at once: level 2's rolls first, level 3 named as next.
        let due = world.due().expect("choices due");
        assert_eq!((due.level, due.talents), (2, 0), "{due:?}");
        assert!(due.rolls > 0);
        let mut screen = LevelUpScreen::new(Some(&world.perception()));
        assert_eq!(screen.step, Step::Rolls);
        assert_eq!(screen.after.as_deref(), Some("Level 3 next"));
        let drawn = text(&screen);
        assert!(drawn.contains("Level 2 · Skill rolls"), "{drawn}");
        assert!(drawn.contains("Athletics"), "{drawn}");
        assert_eq!(screen.handle(&Press::nav(Nav::Back)), ScreenAction::Stay);
        assert!(text(&screen).contains("skill rolls to spend"));
        // A letter rolls on its skill: the action, which the stack closes the screen to apply.
        let ScreenAction::Act(action) = screen.handle(&Press::typed('a')) else {
            panic!("a roll");
        };
        assert!(matches!(action, Action::Advance(_)), "{action:?}");
        spend_rolls(&mut world, &action);
        assert_eq!(world.due().map(|d| d.level), Some(3));
        spend_rolls(&mut world, &action);
        let screen = LevelUpScreen::new(Some(&world.perception()));
        assert_eq!(screen.step, Step::Talent);
        assert_eq!(screen.after, None);
        let drawn = text(&screen);
        assert!(drawn.contains("Level 3 · Talent 1"), "{drawn}");
        assert!(drawn.contains("Vigour"), "{drawn}");
        assert!(
            !drawn.contains("Fast healer"),
            "not met, so not listed: {drawn}"
        );
    }

    #[test]
    fn a_talent_with_choices_lists_them_in_its_place() {
        let mut world = levelled();
        let roll = Action::Advance("athletics".into());
        spend_rolls(&mut world, &roll);
        spend_rolls(&mut world, &roll);
        let mut screen = LevelUpScreen::new(Some(&world.perception()));
        let apprentice = (screen.entries.iter())
            .position(|e| e.name == "Apprentice")
            .expect("Apprentice is listed");
        assert_eq!(screen.choose(apprentice), ScreenAction::Stay);
        let drawn = text(&screen);
        assert!(drawn.contains("Stealth"), "the skills to choose: {drawn}");
        let ScreenAction::Act(Action::Talent { id, choice }) = screen.handle(&Press::typed('a'))
        else {
            panic!("a choice");
        };
        assert_eq!(id, "apprentice");
        assert!(choice.is_some());
        screen.choose(apprentice);
        assert_eq!(screen.handle(&Press::nav(Nav::Back)), ScreenAction::Stay);
        assert!(screen.choosing.is_none(), "Esc returns to the talents");
    }
}
