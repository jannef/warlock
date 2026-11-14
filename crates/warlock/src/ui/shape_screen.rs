//! The shape list (spec §6 "Abilities"; #881): what the Hunter's Spirit weapon asks before it
//! is used, opened from the abilities menu. Every melee weapon there is
//! (`PerceptionView::weapon_shapes`), in the one paged list grouped by weapon skill, each row its
//! glyph, its name with its numbers as a plain one would have them (`Lore::listed`: the gifts are
//! the ability's, said in its description), and its weight. A letter on the page, or Enter on the
//! `>` row, calls a weapon of that shape ([`Action::Shape`], `use:spirit_weapon:long_sword`);
//! the list keys turn the page; Esc closes it. The keys are the list's own, fixed.

use super::menu::{HINTS, Nav, Panel, Press};
use super::paged_list::{PagedList, Row, draw_header};
use super::screens::ScreenAction;
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::Bindings;
use crate::render::console::Console;
use warlock_core::Action;
use warlock_core::color::Rgb;
use warlock_core::content::{ClassActive, WeaponSkillId};
use warlock_core::object::{Stack, format_weight};
use warlock_core::perception::PerceptionView;

/// Panel width: room for the longest name with its numbers (`a morning star (+0 hit, 2d4)`).
const WIDTH: i32 = 60;

/// One shape as the list shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Shape {
    /// The object's id: what the action names.
    id: String,
    glyph: char,
    fg: Rgb,
    name: String,
    /// Grams.
    weight: u64,
}

#[derive(Debug)]
pub struct ShapeScreen {
    shapes: PagedList<Shape, WeaponSkillId>,
}

impl ShapeScreen {
    /// The list, read from the game's objects (`None` outside a run: empty).
    pub fn new(view: Option<&PerceptionView<'_>>) -> Self {
        let Some(view) = view else {
            return Self {
                shapes: PagedList::empty(),
            };
        };
        let content = view.content();
        let rows = (view.weapon_shapes().into_iter())
            .filter_map(|object| {
                let def = content.object(object);
                let stack = Stack::one(object);
                let shape = Shape {
                    id: def.id.clone(),
                    glyph: def.glyph,
                    fg: stack.fg(content),
                    name: view.lore().listed(&stack),
                    weight: stack.weight(content),
                };
                Some((def.skill?, shape))
            })
            .collect();
        let skills = content.weapon_skills();
        Self {
            shapes: PagedList::grouped(rows, |skill| skills.skill(skill).name.clone()),
        }
    }

    /// Lays the list out for a console of `cols` by `rows`: a page takes the rows the frame, the
    /// margin, the gap, and the footer leave.
    pub fn fit(&self, _cols: u32, rows: u32) {
        self.shapes.set_room((rows as i32 - 5).max(1) as usize);
    }

    /// Applies a key press: a letter on the page shown, or Enter on the `>` row, calls that
    /// shape; the list keys move the cursor and turn the page; Esc goes back.
    pub fn handle(&mut self, press: &Press, _bindings: &Bindings) -> ScreenAction {
        if press.nav == Nav::Back || (self.shapes.is_empty() && press.nav == Nav::Enter) {
            return ScreenAction::Close;
        }
        if let Some(c) = press.text.filter(char::is_ascii_lowercase) {
            return match self.shapes.at_letter(c) {
                Some(row) => self.choose(row),
                None => ScreenAction::Stay,
            };
        }
        match press.nav {
            Nav::Enter => return self.choose(self.shapes.cursor()),
            nav => {
                self.shapes.navigate(nav);
            }
        }
        ScreenAction::Stay
    }

    /// Calls the shape of list item `row`.
    fn choose(&self, row: usize) -> ScreenAction {
        match self.shapes.get(row) {
            Some(shape) => {
                ScreenAction::Act(Action::Shape(ClassActive::SpiritWeapon, shape.id.clone()))
            }
            None => ScreenAction::Stay,
        }
    }

    pub fn draw(&self, console: &mut Console) {
        // Rows: margin, the list, a gap, the footer, the frame.
        self.fit(console.cols(), console.rows());
        let count = self.shapes.tallest().max(1) as i32;
        let panel = Panel::new(console, " Spirit weapon: which shape? ", WIDTH, count + 5);
        self.shapes.draw_label(console, &panel);
        let x = panel.inner_x();
        let right = panel.x + panel.w - 3;
        let fits = |y: i32| y < panel.y + panel.h - 2;
        if self.shapes.is_empty() && fits(panel.y + 2) {
            console.print(x + 2, panel.y + 2, "There is no shape to call.", TEXT_DIM);
        }
        for (row, y) in self.shapes.rows().into_iter().zip(panel.y + 2..) {
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
            let Some(shape) = self.shapes.get(index) else {
                continue;
            };
            let selected = index == self.shapes.cursor();
            let color = if selected { ACCENT } else { TEXT };
            if selected {
                console.print(x, y, ">", ACCENT);
            }
            // `b - ) a long sword (+0 hit, 1d9)        1.8 kg`, the page's letter.
            console.print(x + 2, y, &format!("{letter} -"), color);
            console.print(x + 6, y, &shape.glyph.to_string(), shape.fg);
            let weight = format_weight(shape.weight);
            let weight_x = right - weight.chars().count() as i32;
            let room = (weight_x - x - 9).max(0) as usize;
            let name: String = shape.name.chars().take(room).collect();
            console.print(x + 8, y, &name, color);
            console.print(weight_x, y, &weight, TEXT_DIM);
        }
        let keys = HINTS;
        let confirm = &keys.confirm;
        let choose = match self.shapes.last_letter() {
            Some('a') => format!("a or {confirm}"),
            Some(last) => format!("a-{last} or {confirm}"),
            None => confirm.to_string(),
        };
        let page = (self.shapes.hint(&keys)).map_or(String::new(), |hint| format!("{hint}   "));
        panel.footer(console, &format!("{choose} call   {page}Esc back"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use warlock_core::World;
    use warlock_core::content::Content;

    #[test]
    fn a_letter_calls_its_shape() {
        let content = Arc::new(Content::builtin().expect("content"));
        let world = World::new(content, "showcase", None).expect("world");
        let mut screen = ShapeScreen::new(Some(&world.perception()));
        screen.fit(100, 60);
        // Daggers first: the dagger, then the stiletto.
        assert_eq!(
            screen.handle(&Press::typed('a'), &Bindings::default()),
            ScreenAction::Act(Action::Shape(ClassActive::SpiritWeapon, "dagger".into()))
        );
        let nothing = ShapeScreen::new(None);
        assert!(nothing.shapes.is_empty());
    }
}
