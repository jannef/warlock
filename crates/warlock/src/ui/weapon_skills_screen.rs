//! The weapon skills screen (spec §6; `W`): the player's weapon skills (spec §8 "Weapon
//! skills"), ADOM-like: by section (melee, missile and thrown, shields), each with its rank, 0 to
//! 10, and a faint bar of their marks toward the next rank. Never what a rank gives: perception
//! has none of it, and the wiki documents the rules.
//!
//! Read-only: Esc, Enter, or its own key closes it. It is read from the perception snapshot when
//! it opens; no game time passes while it is open, so it cannot go stale.

use super::menu::{Nav, Panel, Press};
use super::screens::ScreenAction;
use super::{TEXT, TEXT_DIM};
use crate::input::{Bindings, KeyCommand};
use crate::render::console::Console;
use warlock_core::color::Rgb;
use warlock_core::content::{Section, WeaponSkillTable};
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

/// Width of the panel.
const WIDTH: i32 = 46;
/// Width of the name column.
const NAME_W: i32 = 24;

/// A row of the screen.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Row {
    /// A section's heading: `Melee`.
    Section(&'static str),
    /// A skill: its name, its rank, and how many of the bar's cells its marks fill (`None` at
    /// the last rank: nothing more to earn).
    Skill {
        name: String,
        rank: u8,
        filled: Option<u32>,
    },
    /// A gap between sections.
    Gap,
}

#[derive(Debug)]
pub struct WeaponSkillsScreen {
    rows: Vec<Row>,
}

impl WeaponSkillsScreen {
    /// The screen for what the player perceives of their weapon skills (`None` outside a run:
    /// no skills).
    pub fn new(view: Option<&PerceptionView<'_>>) -> Self {
        match view {
            Some(view) => Self::of(view.content().weapon_skills(), view.own_weapon_skills()),
            None => Self { rows: Vec::new() },
        }
    }

    /// The screen for ranks and marks `own` in the skills of `table`.
    fn of(table: &WeaponSkillTable, own: &WeaponSkills) -> Self {
        let mut rows = Vec::new();
        for section in Section::ALL {
            let skills: Vec<Row> = (table.ids())
                .filter(|(_, def)| def.role.section() == section)
                .map(|(id, def)| Row::Skill {
                    name: def.name.clone(),
                    rank: own.rank(id),
                    filled: (own.progress(table, id))
                        .map(|(marks, need)| marks * BAR / need.max(1)),
                })
                .collect();
            if skills.is_empty() {
                continue;
            }
            if !rows.is_empty() {
                rows.push(Row::Gap);
            }
            rows.push(Row::Section(section.name()));
            rows.extend(skills);
        }
        Self { rows }
    }

    /// Applies a key press: Esc, Enter, or the screen's own key closes it.
    pub fn handle(&mut self, press: &Press, bindings: &Bindings) -> ScreenAction {
        let own_key = press
            .binding
            .as_ref()
            .is_some_and(|key| bindings.owner(key) == Some(KeyCommand::WeaponSkills));
        if own_key || matches!(press.nav, Nav::Back | Nav::Enter) {
            ScreenAction::Close
        } else {
            ScreenAction::Stay
        }
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        // Rows: margin, the rows (or a note when there are none), a gap, the footer, frame. In
        // a console too short for them all, the gaps between sections go first.
        let compact = self.rows.len() as i32 + 5 > console.rows() as i32;
        let rows: Vec<&Row> = (self.rows.iter())
            .filter(|r| !(compact && **r == Row::Gap))
            .collect();
        let body = rows.len().max(1) as i32;
        let panel = Panel::new(console, " Weapon skills ", WIDTH, body + 5);
        let x = panel.inner_x();
        let fits = |y: i32| y < panel.y + panel.h - 2;
        let top = panel.y + 2;
        if self.rows.is_empty() && fits(top) {
            console.print(x, top, "You know no weapon skills.", TEXT_DIM);
        }
        for (y, row) in (top..).zip(rows) {
            if !fits(y) {
                break;
            }
            match row {
                Row::Section(name) => {
                    console.print(x, y, name, TEXT_DIM);
                }
                Row::Skill { name, rank, filled } => {
                    // Untrained skills read dimmer, so what the player knows stands out.
                    let fg = if *rank == 0 { TEXT_DIM } else { TEXT };
                    console.print(x + 2, y, name, fg);
                    console.print(x + 2 + NAME_W, y, &format!("{rank:>2}"), fg);
                    if let Some(filled) = filled {
                        draw_bar(console, x + NAME_W + 6, y, *filled);
                    }
                }
                Row::Gap => {}
            }
        }
        let footer = match bindings.keys_of(KeyCommand::WeaponSkills).first() {
            Some(key) => format!("{} or Esc close", key.display()),
            None => "Esc close".to_string(),
        };
        panel.footer(console, &footer);
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

    #[test]
    fn lists_every_skill_by_section_with_its_rank_and_a_bar() {
        // The showcase thief: daggers 2 and thrown 1.
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
        for heading in ["Melee", "Missile and thrown", "Shields"] {
            row(heading);
        }
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
        let skills = world.content().weapon_skills().skills().len();
        let listed = (screen.rows.iter())
            .filter(|r| matches!(r, Row::Skill { .. }))
            .count();
        assert_eq!(listed, skills);
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
        let screen = WeaponSkillsScreen::of(table, &skills);
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
        // 24 rows (960×540 at the default font): the section gaps give way, and everything fits.
        let text = drawn(&screen, 96, 24);
        let at = |needle: &str| text.iter().position(|l| l.contains(needle));
        assert!(at("Shield bashing").is_some(), "{text:#?}");
        assert_eq!(at("Missile and thrown"), at("Unarmed").map(|y| y + 1));
        assert!(at("Shield bashing") < at("W or Esc close"));
    }
}
