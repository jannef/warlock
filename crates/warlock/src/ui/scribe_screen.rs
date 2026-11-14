//! The scribe list (spec §6 "Abilities", §8 "Scrolls"; #906): what the wizard's Scribe asks
//! before it is used, opened from the abilities menu. The spells the player knows that a scroll
//! holds (`PerceptionView::scribes`), in the one paged list grouped by school, each row the
//! spell's name, what writing it costs them in mana (they know their own costs), and how long it
//! takes at a normal pace in words; the blank scrolls carried above. A letter on the page, or
//! Enter on the `>` row, writes it ([`Action::Scribe`], `scribe:fireball`); the list keys turn
//! the page; Esc closes it. The keys are the list's own, fixed.

use super::menu::{HINTS, Nav, Panel, Press};
use super::paged_list::{PagedList, Row, draw_header};
use super::screens::ScreenAction;
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::Bindings;
use crate::render::console::Console;
use warlock_core::Action;
use warlock_core::content::School;
use warlock_core::perception::PerceptionView;
use warlock_core::text::capitalized;

/// Panel width: room for the longest spell name with its price and time.
const WIDTH: i32 = 64;

/// One spell as the list shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Writable {
    /// The spell's id: what the action names.
    id: String,
    name: String,
    /// "78 mana".
    mana: String,
    /// "about 32 minutes".
    time: String,
}

#[derive(Debug)]
pub struct ScribeScreen {
    spells: PagedList<Writable, School>,
    /// How many blank scrolls the player carries.
    blanks: u32,
}

impl ScribeScreen {
    /// The list, read from the player's perception (`None` outside a run: empty).
    pub fn new(view: Option<&PerceptionView<'_>>) -> Self {
        let Some(view) = view else {
            return Self {
                spells: PagedList::empty(),
                blanks: 0,
            };
        };
        let content = view.content();
        let rows = (view.scribes().into_iter())
            .map(|own| {
                let def = content.spell(own.spell);
                let writable = Writable {
                    id: def.id.clone(),
                    name: def.name.clone(),
                    mana: format!("{} mana", own.mana),
                    time: format!("about {}", own.time.spoken()),
                };
                (def.school, writable)
            })
            .collect();
        Self {
            spells: PagedList::grouped(rows, |school| capitalized(school.id())),
            blanks: view.blank_scrolls(),
        }
    }

    /// Lays the list out for a console of `cols` by `rows`: a page takes the rows the frame, the
    /// margin, the blanks' line, the gaps, and the footer leave.
    pub fn fit(&self, _cols: u32, rows: u32) {
        self.spells.set_room((rows as i32 - 7).max(1) as usize);
    }

    /// Applies a key press: a letter on the page shown, or Enter on the `>` row, writes that
    /// spell; the list keys move the cursor and turn the page; Esc goes back.
    pub fn handle(&mut self, press: &Press, _bindings: &Bindings) -> ScreenAction {
        if press.nav == Nav::Back || (self.spells.is_empty() && press.nav == Nav::Enter) {
            return ScreenAction::Close;
        }
        if let Some(c) = press.text.filter(char::is_ascii_lowercase) {
            return match self.spells.at_letter(c) {
                Some(row) => self.choose(row),
                None => ScreenAction::Stay,
            };
        }
        match press.nav {
            Nav::Enter => return self.choose(self.spells.cursor()),
            nav => {
                self.spells.navigate(nav);
            }
        }
        ScreenAction::Stay
    }

    /// Writes the spell of list item `row`.
    fn choose(&self, row: usize) -> ScreenAction {
        match self.spells.get(row) {
            Some(spell) => ScreenAction::Act(Action::Scribe {
                spell: spell.id.clone(),
            }),
            None => ScreenAction::Stay,
        }
    }

    pub fn draw(&self, console: &mut Console) {
        // Rows: margin, the blanks' line, a gap, the list, a gap, the footer, the frame.
        self.fit(console.cols(), console.rows());
        let count = self.spells.tallest().max(1) as i32;
        let panel = Panel::new(console, " Scribe: which spell? ", WIDTH, count + 7);
        self.spells.draw_label(console, &panel);
        let x = panel.inner_x();
        let right = panel.x + panel.w - 3;
        let fits = |y: i32| y < panel.y + panel.h - 2;
        let blanks = match self.blanks {
            0 => "You carry no blank scroll to write on.".to_string(),
            1 => "You carry 1 blank scroll.".to_string(),
            n => format!("You carry {n} blank scrolls."),
        };
        console.print(x + 2, panel.y + 2, &blanks, TEXT_DIM);
        if self.spells.is_empty() && fits(panel.y + 4) {
            console.print(x + 2, panel.y + 4, "You know no spell to write.", TEXT_DIM);
        }
        for (row, y) in self.spells.rows().into_iter().zip(panel.y + 4..) {
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
            let Some(spell) = self.spells.get(index) else {
                continue;
            };
            let selected = index == self.spells.cursor();
            let color = if selected { ACCENT } else { TEXT };
            if selected {
                console.print(x, y, ">", ACCENT);
            }
            // `b - Fireball          78 mana   about 32 minutes`, the page's letter.
            console.print(x + 2, y, &format!("{letter} - {}", spell.name), color);
            let time_x = right - spell.time.chars().count() as i32;
            console.print(time_x, y, &spell.time, TEXT_DIM);
            let mana_x = right - 20 - spell.mana.chars().count() as i32;
            console.print(mana_x, y, &spell.mana, color);
        }
        let keys = HINTS;
        let confirm = &keys.confirm;
        let choose = match self.spells.last_letter() {
            Some('a') => format!("a or {confirm}"),
            Some(last) => format!("a-{last} or {confirm}"),
            None => confirm.to_string(),
        };
        let page = (self.spells.hint(&keys)).map_or(String::new(), |hint| format!("{hint}   "));
        if self.spells.is_empty() {
            panel.footer(console, "Esc back");
        } else {
            panel.footer(console, &format!("{choose} write   {page}Esc back"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use warlock_core::World;
    use warlock_core::character::CharacterSpec;
    use warlock_core::content::Content;

    #[test]
    fn a_letter_writes_its_spell() {
        let content = Arc::new(Content::builtin().expect("content"));
        let spec: CharacterSpec = "human,female,wizard".parse().expect("a character");
        let mut world =
            World::with_character(content, "showcase", None, Some(&spec)).expect("world");
        for action in warlock_core::action::parse_actions("xp:200000 levelup:auto").unwrap() {
            world.apply(&action);
        }
        let mut screen = ScribeScreen::new(Some(&world.perception()));
        screen.fit(100, 60);
        // Evocation first, lowest level first: Burning hands.
        assert_eq!(
            screen.handle(&Press::typed('a'), &Bindings::default()),
            ScreenAction::Act(Action::Scribe {
                spell: "burning_hands".into()
            })
        );
        let nothing = ScribeScreen::new(None);
        assert!(nothing.spells.is_empty());
    }
}
