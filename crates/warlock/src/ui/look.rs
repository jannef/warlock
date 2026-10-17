//! Look mode (spec §6): a cursor the player moves over the map to read what is there. It is a
//! free action — no game time passes — and it only describes what the player perceives.

use crate::render::console::{Cell, Console};
use warlock_core::Pos;
use warlock_core::color::Rgb;
use warlock_core::perception::{CellView, PerceptionView};

/// Cursor colors: dark glyph on a warm highlight, so it reads on any terrain and under any light.
const CURSOR_FG: Rgb = Rgb::new(0x14, 0x12, 0x10);
const CURSOR_BG: Rgb = Rgb::new(0xe0, 0xc0, 0x70);

/// One line describing what the player knows about `p`.
pub fn describe(view: &PerceptionView<'_>, p: Pos) -> String {
    let content = view.content();
    let terrain = |t| content.terrain(t).name.clone();
    match view.cell(p) {
        CellView::Visible(t) => match view.actor_at(p) {
            Some(_) if p == view.viewer() => format!("You, on {}.", terrain(t)),
            Some(actor) => {
                let name = &content.creature(actor.creature).name;
                format!("{} {name}, on {}.", article(name), terrain(t))
            }
            None => format!("{}.", capitalize(&with_article(&terrain(t)))),
        },
        CellView::Remembered(t) => format!("You remember {} here.", with_article(&terrain(t))),
        CellView::Unknown => "You don't know what is there.".to_string(),
    }
}

fn article(noun: &str) -> &'static str {
    match noun.chars().next() {
        Some(c) if "aeiouAEIOU".contains(c) => "An",
        _ => "A",
    }
}

fn with_article(noun: &str) -> String {
    format!("{} {noun}", article(noun).to_lowercase())
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// Draws the cursor at console cell `(x, y)`, showing what the player perceives there.
pub fn draw_cursor(console: &mut Console, x: i32, y: i32, view: &PerceptionView<'_>, p: Pos) {
    let content = view.content();
    let ch = match (view.cell(p), view.actor_at(p)) {
        (CellView::Visible(_), Some(actor)) => content.creature(actor.creature).glyph,
        (CellView::Visible(t) | CellView::Remembered(t), _) => content.terrain(t).glyph,
        (CellView::Unknown, _) => ' ',
    };
    console.put(
        x,
        y,
        Cell {
            ch,
            fg: CURSOR_FG,
            bg: CURSOR_BG,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use warlock_core::content::Content;
    use warlock_core::{Action, Dir, World};

    fn world() -> World {
        let content = Arc::new(Content::builtin().expect("content"));
        World::new(content, "showcase", None).expect("world")
    }

    #[test]
    fn describes_the_player_terrain_memory_and_the_unknown() {
        let mut w = world();
        let me = w.player_pos();
        assert_eq!(describe(&w.perception(), me), "You, on stone floor.");
        let wall = Pos::new(0, me.y);
        assert_eq!(describe(&w.perception(), wall), "A stone wall.");
        let door = Pos::new(me.x, me.y - 4);
        assert_eq!(describe(&w.perception(), door), "A closed door.");
        assert_eq!(
            describe(&w.perception(), Pos::new(70, 3)),
            "You don't know what is there."
        );
        // Walk out of the start room: the room's floor is remembered, not seen.
        for _ in 0..3 {
            w.apply(&Action::Move(Dir::N));
        }
        for _ in 0..3 {
            w.apply(&Action::Move(Dir::N));
        }
        for _ in 0..8 {
            w.apply(&Action::Move(Dir::E));
        }
        let text = describe(&w.perception(), me);
        assert!(text.starts_with("You remember"), "{text}");
    }

    #[test]
    fn articles() {
        assert_eq!(with_article("open door"), "an open door");
        assert_eq!(with_article("stone wall"), "a stone wall");
        assert_eq!(capitalize("an altar"), "An altar");
    }
}
