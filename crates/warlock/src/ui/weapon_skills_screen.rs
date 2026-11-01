//! The weapon skills screen (spec §6; `W`): the player's weapon skills (spec §8 "Weapon
//! skills") and fighting styles (spec §8 "Fighting styles"), ADOM-like: by section (melee,
//! missile and thrown, shields, fighting styles), each with its rank, 0 to 10, and a faint bar of
//! their marks toward the next rank; the styles the player fights in now are marked. Never what a
//! rank gives: perception has none of it, and the wiki documents the rules.
//!
//! Two columns where the console is wide enough (the weapons on the left, shields and styles on
//! the right), else one. In a console too short for every row the gaps and margins give way,
//! then the list scrolls (↑↓). Read-only: Esc, Enter, or its own key closes it. It is read from
//! the perception snapshot when it opens; no game time passes while it is open, so it cannot go
//! stale.

use super::menu::{Hints, Nav, Panel, Press};
use super::screens::ScreenAction;
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::{Bindings, KeyCommand};
use crate::render::console::Console;
use std::sync::atomic::{AtomicUsize, Ordering};
use warlock_core::color::Rgb;
use warlock_core::content::{Section, WeaponSkillId, WeaponSkillTable};
use warlock_core::perception::PerceptionView;
use warlock_core::weapon_skills::WeaponSkills;

/// Cells of the marks bar.
pub const BAR: u32 = 10;
/// The bar's glyphs: marks earned, and still to earn.
pub const BAR_DONE: char = '━';
pub const BAR_LEFT: char = '─';
/// The bar's colors: a muted gold for marks earned, barely there for the rest (subtle on
/// purpose: it is a hint of progress, not a number).
const BAR_DONE_FG: Rgb = Rgb::new(0x9a, 0x80, 0x48);
const BAR_LEFT_FG: Rgb = Rgb::new(0x3c, 0x3a, 0x40);
/// The mark of a fighting style in use, before its name.
pub const IN_USE: char = '•';

/// Width of the name column.
const NAME_W: i32 = 24;
/// Width of a column of skills: indent, name, rank, bar.
const COLUMN_W: i32 = 2 + NAME_W + 4 + BAR as i32;
/// Cells between the two columns.
const GUTTER: i32 = 4;
/// Width of the panel with one column of skills, and with two.
const WIDTH: i32 = COLUMN_W + 6;
const WIDTH_TWO: i32 = 2 * COLUMN_W + GUTTER + 4;

/// A row of the screen.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Row {
    /// A section's heading (`Melee`), and the legend of the in-use mark for the styles' while
    /// one is in use.
    Section { name: &'static str, legend: bool },
    /// A skill: its name, its rank, how many of the bar's cells its marks fill (`None` at the
    /// last rank: nothing more to earn), and whether it is a fighting style in use.
    Skill {
        name: String,
        rank: u8,
        filled: Option<u32>,
        in_use: bool,
    },
    /// A gap between sections.
    Gap,
}

#[derive(Debug)]
pub struct WeaponSkillsScreen {
    /// The rows of the left column (melee, missile and thrown) and of the right one (shields,
    /// fighting styles); one column stacks them.
    columns: [Vec<Row>; 2],
    /// The first row shown, scrolled with ↑↓ when the rows do not fit.
    top: usize,
    /// How far the list could scroll when it was last drawn (0: it all fits), so ↓ stops at the
    /// end. Drawing knows the console; key presses do not.
    most: AtomicUsize,
}

impl WeaponSkillsScreen {
    /// The screen for what the player perceives of their weapon skills (`None` outside a run:
    /// no skills).
    pub fn new(view: Option<&PerceptionView<'_>>) -> Self {
        match view {
            Some(view) => Self::of(
                view.content().weapon_skills(),
                view.own_weapon_skills(),
                &view.own_styles_in_use(),
            ),
            None => Self::with([Vec::new(), Vec::new()]),
        }
    }

    fn with(columns: [Vec<Row>; 2]) -> Self {
        Self {
            columns,
            top: 0,
            most: AtomicUsize::new(0),
        }
    }

    /// The screen for ranks and marks `own` in the skills of `table`, the styles `in_use`
    /// marked.
    fn of(table: &WeaponSkillTable, own: &WeaponSkills, in_use: &[WeaponSkillId]) -> Self {
        let mut columns: [Vec<Row>; 2] = [Vec::new(), Vec::new()];
        for section in Section::ALL {
            let skills: Vec<Row> = (table.ids())
                .filter(|&(id, def)| def.role.section() == Some(section) && own.shows(table, id))
                .map(|(id, def)| Row::Skill {
                    name: def.name.clone(),
                    rank: own.rank(id),
                    filled: (own.progress(table, id))
                        .map(|(marks, need)| marks * BAR / need.max(1)),
                    in_use: in_use.contains(&id),
                })
                .collect();
            if skills.is_empty() {
                continue;
            }
            let rows = &mut columns[usize::from(section >= Section::Shields)];
            if !rows.is_empty() {
                rows.push(Row::Gap);
            }
            let legend = (skills.iter()).any(|r| matches!(r, Row::Skill { in_use: true, .. }));
            rows.push(Row::Section {
                name: section.name(),
                legend,
            });
            rows.extend(skills);
        }
        Self::with(columns)
    }

    /// Applies a key press: ↑↓ scroll a list too long for the console; Esc, Enter, or the
    /// screen's own key closes it.
    pub fn handle(&mut self, press: &Press, bindings: &Bindings) -> ScreenAction {
        let own_key = press
            .binding
            .as_ref()
            .is_some_and(|key| bindings.owner(key) == Some(KeyCommand::WeaponSkills));
        match press.nav {
            _ if own_key => return ScreenAction::Close,
            Nav::Back | Nav::Enter => return ScreenAction::Close,
            Nav::Up => self.top = self.top.saturating_sub(1),
            Nav::Down => self.top = (self.top + 1).min(self.most.load(Ordering::Relaxed)),
            _ => {}
        }
        ScreenAction::Stay
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        // Two columns where they fit side by side, else one: the right column under the left.
        let [left, right] = &self.columns;
        let two = console.cols() as i32 >= WIDTH_TWO;
        let columns: Vec<Vec<&Row>> = if two {
            vec![left.iter().collect(), right.iter().collect()]
        } else {
            let gap = (!left.is_empty() && !right.is_empty()).then_some(&Row::Gap);
            vec![left.iter().chain(gap).chain(right).collect()]
        };
        // Rows: frame, margin, the rows (or a note when there are none), a gap, the footer,
        // frame. In a console too short for them all, the gaps between sections, the margin, and
        // the gap above the footer give way; then the rows scroll.
        let tallest = |columns: &[Vec<&Row>]| columns.iter().map(Vec::len).max().unwrap_or(0);
        let compact = tallest(&columns) as i32 + 5 > console.rows() as i32;
        let around = if compact { 3 } else { 5 };
        let columns: Vec<Vec<&Row>> = (columns.into_iter())
            .map(|rows| {
                (rows.into_iter())
                    .filter(|r| !(compact && **r == Row::Gap))
                    .collect()
            })
            .collect();
        let body = tallest(&columns).max(1) as i32;
        let width = super::info_tabs::panel_width(if two { WIDTH_TWO } else { WIDTH });
        let panel = Panel::new(console, " Weapon skills ", width, body + around);
        super::info_tabs::draw_tabs(console, &panel, bindings, super::ScreenName::WeaponSkills);
        let room = (panel.h - around).max(0);
        let most = (body - room).max(0) as usize;
        self.most.store(most, Ordering::Relaxed);
        let first = self.top.min(most);
        let top = panel.y + if compact { 1 } else { 2 };
        if left.is_empty() && right.is_empty() && room > 0 {
            console.print(panel.inner_x(), top, "You know no weapon skills.", TEXT_DIM);
        }
        for (i, rows) in columns.iter().enumerate() {
            let x = panel.inner_x() + i as i32 * (COLUMN_W + GUTTER);
            for (y, row) in (top..).zip(rows.iter().skip(first).take(room as usize)) {
                draw_row(console, x, y, row);
            }
        }
        // Arrows on the frame where rows are hidden above or below.
        let edge = panel.x + panel.w - 1;
        if first > 0 && room > 0 {
            console.print(edge, top, "↑", TEXT_DIM);
        }
        if first < most && room > 0 {
            console.print(edge, top + room - 1, "↓", TEXT_DIM);
        }
        let close = match bindings.keys_of(KeyCommand::WeaponSkills).first() {
            Some(key) => format!("{} or Esc close", key.display()),
            None => "Esc close".to_string(),
        };
        let footer = if most > 0 {
            format!("{} scroll   {close}", Hints::of(bindings).up_down())
        } else {
            close
        };
        panel.footer(console, &footer);
    }
}

/// One row of a column at (`x`, `y`).
fn draw_row(console: &mut Console, x: i32, y: i32, row: &Row) {
    match row {
        Row::Section { name, legend } => {
            console.print(x, y, name, TEXT_DIM);
            if *legend {
                let text = format!("{IN_USE} in use");
                let at = x + COLUMN_W - text.chars().count() as i32;
                console.print(at, y, &IN_USE.to_string(), ACCENT);
                console.print(at + 1, y, " in use", TEXT_DIM);
            }
        }
        Row::Skill {
            name,
            rank,
            filled,
            in_use,
        } => {
            // Untrained skills read dimmer, so what the player knows stands out.
            let fg = if *rank == 0 { TEXT_DIM } else { TEXT };
            if *in_use {
                console.print(x, y, &IN_USE.to_string(), ACCENT);
            }
            console.print(x + 2, y, name, fg);
            console.print(x + 2 + NAME_W, y, &format!("{rank:>2}"), fg);
            if let Some(filled) = filled {
                draw_bar(console, x + NAME_W + 6, y, *filled);
            }
        }
        Row::Gap => {}
    }
}

/// The marks bar at (`x`, `y`): `filled` of [`BAR`] cells earned.
fn draw_bar(console: &mut Console, x: i32, y: i32, filled: u32) {
    for i in 0..BAR {
        let (glyph, fg) = if i < filled {
            (BAR_DONE, BAR_DONE_FG)
        } else {
            (BAR_LEFT, BAR_LEFT_FG)
        };
        console.print(x + i as i32, y, &glyph.to_string(), fg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::KeyBinding;
    use std::sync::Arc;
    use warlock_core::World;
    use warlock_core::content::{Content, MAX_RANK};

    fn showcase() -> World {
        let content = Arc::new(Content::builtin().expect("content"));
        World::new(content, "showcase", None).expect("world")
    }

    fn text(console: &Console) -> Vec<String> {
        (0..console.rows() as i32)
            .map(|y| {
                (0..console.cols() as i32)
                    .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
                    .collect()
            })
            .collect()
    }

    fn drawn(screen: &WeaponSkillsScreen, cols: u32, rows: u32) -> Vec<String> {
        let mut console = Console::default();
        console.resize(cols, rows);
        screen.draw(&mut console, &Bindings::default());
        text(&console)
    }

    /// Every skill row of the screen, in both columns.
    fn skill_rows(screen: &WeaponSkillsScreen) -> usize {
        (screen.columns.iter().flatten())
            .filter(|r| matches!(r, Row::Skill { .. }))
            .count()
    }

    #[test]
    fn lists_every_skill_by_section_with_its_rank_and_a_bar() {
        // The showcase thief: daggers 2 and thrown 1, and Duelist 1. In one column (80 wide).
        let world = showcase();
        let screen = WeaponSkillsScreen::new(Some(&world.perception()));
        let text = drawn(&screen, 80, 40);
        let row = |needle: &str| {
            text.iter()
                .find(|l| l.contains(needle))
                .unwrap_or_else(|| panic!("{needle} in {text:#?}"))
                .clone()
        };
        assert!(row(" Weapon skills ").contains('─'));
        for heading in ["Melee", "Missile and thrown", "Shields", "Fighting styles"] {
            row(heading);
        }
        assert!(row("Duelist").contains("Duelist                  1"));
        let daggers = row("Daggers and knives");
        assert!(daggers.contains("Daggers and knives       2"), "{daggers}");
        assert!(row("Thrown weapons").contains(" 1 "));
        assert!(row("Long blades").contains(" 0 "));
        // No marks yet: an empty bar of ten.
        assert!(daggers.contains(&BAR_LEFT.to_string().repeat(BAR as usize)));
        assert!(!daggers.contains(BAR_DONE));
        row("W or Esc close");
        // Sections in order, every skill once.
        let at = |needle: &str| text.iter().position(|l| l.contains(needle));
        assert!(at("Melee") < at("Missile and thrown") && at("Missile and thrown") < at("Shields"));
        assert!(at("Shields") < at("Fighting styles") && at("Fighting styles") < at("Duelist"));
        // Every skill once but Two Shields, hidden until its first marks (#611), and the seven
        // schools of magic, which are magic's, not the weapons'.
        let skills = world.content().weapon_skills().skills().len();
        assert_eq!(skill_rows(&screen), skills - 1 - 7);
        assert!(at("Evocation").is_none());
        assert!(at("Two Shields").is_none());
    }

    #[test]
    fn marks_the_fighting_styles_in_use() {
        // Nothing in the showcase thief's hands: they fight as a brawler.
        let world = showcase();
        let screen = WeaponSkillsScreen::new(Some(&world.perception()));
        let text = drawn(&screen, 80, 40);
        let row = |needle: &str| {
            text.iter()
                .find(|l| l.contains(needle))
                .unwrap_or_else(|| panic!("{needle} in {text:#?}"))
                .clone()
        };
        assert!(row("Brawler").contains(&format!("{IN_USE} Brawler")));
        assert!(!row("Duelist").contains(IN_USE));
        assert!(row("Fighting styles").contains(&format!("{IN_USE} in use")));
        // Only styles are ever marked, and no heading but the styles' has the legend.
        let marked = text.iter().filter(|l| l.contains(IN_USE)).count();
        assert_eq!(marked, 2, "{text:#?}");
        // With none in use there is no legend.
        let content = world.content();
        let none = WeaponSkillsScreen::of(content.weapon_skills(), &WeaponSkills::default(), &[]);
        let text = drawn(&none, 80, 40).join("\n");
        assert!(!text.contains(IN_USE), "{text}");
    }

    #[test]
    fn the_bar_fills_with_marks_and_is_gone_at_the_last_rank() {
        let content = Content::builtin().expect("content");
        let table = content.weapon_skills();
        let (daggers, bows) = (
            table.id("daggers").expect("daggers"),
            table.id("bows").expect("bows"),
        );
        let mut skills = WeaponSkills::from_ranks(table, [(daggers, 2), (bows, MAX_RANK)]);
        // Daggers 2 needs 60 for rank 3: 24 marks fill four cells of ten.
        skills.train(table, daggers, 24);
        let screen = WeaponSkillsScreen::of(table, &skills, &[]);
        let text = drawn(&screen, 80, 40).join("\n");
        let done = BAR_DONE.to_string().repeat(4);
        let left = BAR_LEFT.to_string().repeat(6);
        assert!(text.contains(&format!("{done}{left}")), "{text}");
        let bows_row = text.lines().find(|l| l.contains("Bows")).expect("bows");
        assert!(
            bows_row.contains("10") && !bows_row.contains(BAR_LEFT),
            "{bows_row}"
        );
    }

    #[test]
    fn never_shows_what_a_rank_gives() {
        let text = drawn(
            &WeaponSkillsScreen::new(Some(&showcase().perception())),
            80,
            40,
        )
        .join("\n");
        for hidden in ["+", "%", "to-hit", "damage", "Parry", "marks"] {
            assert!(!text.contains(hidden), "{hidden:?} in\n{text}");
        }
    }

    #[test]
    fn closes_on_its_key_esc_or_enter_and_draws_in_small_consoles() {
        let b = Bindings::default();
        let mut screen = WeaponSkillsScreen::new(None);
        assert_eq!(screen.handle(&Press::typed('a'), &b), ScreenAction::Stay);
        let own = Press {
            binding: KeyBinding::from_text("W"),
            ..Press::typed('W')
        };
        assert_eq!(screen.handle(&own, &b), ScreenAction::Close);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Back), &b),
            ScreenAction::Close
        );
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Close
        );
        let empty = drawn(&screen, 60, 12).join("\n");
        assert!(empty.contains("You know no weapon skills."), "{empty}");
        let world = showcase();
        let screen = WeaponSkillsScreen::new(Some(&world.perception()));
        for (cols, rows) in [(48, 15), (30, 8), (5, 3), (0, 0)] {
            assert_eq!(drawn(&screen, cols, rows).len(), rows as usize);
        }
        // 96 by 24: two columns, the weapons on the left, shields and styles on the right, and
        // everything fits, gaps and all.
        let text = drawn(&screen, 96, 24);
        let at = |needle: &str| text.iter().position(|l| l.contains(needle));
        for needle in ["Shield bashing", "Battle caster", "Slings"] {
            assert!(at(needle) < at("W or Esc close"), "{needle}: {text:#?}");
        }
        assert_eq!(at("Melee"), at("Shields"), "side by side");
        // A margin above the rows, a gap below them, then the footer.
        assert_eq!(at("Melee"), at(" Weapon skills ").map(|y| y + 2));
        assert_eq!(at("W or Esc close"), at("Thrown weapons").map(|y| y + 2));
        assert_eq!(at("Missile and thrown"), at("Unarmed").map(|y| y + 2));
        let (melee, shields) = (&text[at("Melee").expect("melee")], "Shields");
        assert!(melee.find("Melee") < melee.find(shields));
        // One column, tall enough: everything, the styles last.
        let text = drawn(&screen, 60, 40);
        let at = |needle: &str| text.iter().position(|l| l.contains(needle));
        assert!(at("Shield bashing") < at("Fighting styles"));
        assert!(at("Battle caster") < at("W or Esc close"));
        // 64 by 32 (the default 1280×720 window at the default font): one column, the gaps and
        // margins give way, and the last few caster styles scroll (#571).
        let text = drawn(&screen, 64, 32);
        let at = |needle: &str| text.iter().position(|l| l.contains(needle));
        assert_eq!(at("Missile and thrown"), at("Unarmed").map(|y| y + 1));
        assert_eq!(at("Daggers and knives"), at("Melee").map(|y| y + 1));
        assert!(at("Shield bashing") < at("W or Esc close"), "{text:#?}");
        assert!(text.iter().any(|l| l.contains("scroll")), "{text:#?}");
    }

    #[test]
    fn a_list_too_long_for_the_console_scrolls() {
        // 48 by 24 (960×540): the list is longer than the panel.
        let world = showcase();
        let b = Bindings::default();
        let mut screen = WeaponSkillsScreen::new(Some(&world.perception()));
        let text = drawn(&screen, 48, 24);
        let has = |text: &[String], needle: &str| text.iter().any(|l| l.contains(needle));
        assert!(has(&text, "Melee") && !has(&text, "Battle caster"));
        assert!(has(&text, "↑↓ scroll") && has(&text, "↓") && has(&text, "Esc close"));
        // ↑ at the top stays; ↓ scrolls a row at a time, down to the last row and no further.
        assert_eq!(screen.handle(&Press::nav(Nav::Up), &b), ScreenAction::Stay);
        for _ in 0..40 {
            assert_eq!(
                screen.handle(&Press::nav(Nav::Down), &b),
                ScreenAction::Stay
            );
        }
        let text = drawn(&screen, 48, 24);
        assert!(
            has(&text, "Battle caster") && !has(&text, "Melee"),
            "{text:#?}"
        );
        let last = text.iter().position(|l| l.contains("Battle caster"));
        let footer = text.iter().position(|l| l.contains("Esc close"));
        assert_eq!(
            last.map(|y| y + 1),
            footer,
            "the last row right above the footer"
        );
        // One ↑ shows one row more at the top.
        screen.handle(&Press::nav(Nav::Up), &b);
        let up = drawn(&screen, 48, 24);
        assert!(!has(&up, "Battle caster"));
        let at = |text: &[String]| text.iter().position(|l| l.contains("Shield bashing"));
        assert_eq!(at(&up), at(&text).map(|y| y + 1));
    }
}
