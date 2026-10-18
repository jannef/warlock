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
    // Stairs and locations name where they lead: "a staircase down, leading to The Depths 1".
    let terrain = |t| match view.leads_to(p) {
        Some(to) => format!("{}, leading to {to}", content.terrain(t).name),
        None => content.terrain(t).name.clone(),
    };
    // A sound heard there (never on a cell the player sees): "You hear footsteps here."
    let heard = view.sound_at(p).map(|s| s.you_hear());
    match (view.cell(p), heard) {
        (CellView::Visible(t), _) => match view.actor_at(p) {
            Some(_) if p == view.viewer() => format!("You, on {}.", terrain(t)),
            Some(actor) => {
                let name = &content.creature(actor.creature).name;
                format!("{} {name}, on {}.", article(name), terrain(t))
            }
            None => format!("{}.", capitalize(&with_article(&terrain(t)))),
        },
        (CellView::Remembered(t), Some(heard)) => {
            format!(
                "{heard} here, where you remember {}.",
                with_article(&terrain(t))
            )
        }
        (CellView::Remembered(t), None) => {
            format!("You remember {} here.", with_article(&terrain(t)))
        }
        (CellView::Unknown, Some(heard)) => format!("{heard} here."),
        (CellView::Unknown, None) => "You don't know what is there.".to_string(),
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
        _ if view.sound_at(p).is_some() => super::sounds::MARKER,
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
    fn stairs_name_where_they_lead() {
        let mut w = world();
        // Walk to the stairs nook (see the golden manifest's `stairs-nook` walk).
        let steps = |dir, n| std::iter::repeat_n(Action::Move(dir), n);
        let walk: Vec<Action> = steps(Dir::N, 3)
            .chain([Action::Open(Dir::N)])
            .chain(steps(Dir::N, 2))
            .chain(steps(Dir::E, 67))
            .chain([Action::Open(Dir::S)])
            .chain(steps(Dir::S, 4))
            .collect();
        for action in &walk {
            w.apply(action);
        }
        let view = w.perception();
        let (level, _) = w.ground_truth();
        let down = (0..level.height())
            .flat_map(|y| (0..level.width()).map(move |x| Pos::new(x, y)))
            .find(|&p| view.leads_to(p).as_deref() == Some("The Vast Caverns"))
            .expect("the staircase to the vast level is in view");
        let text = describe(&view, down);
        assert!(text.ends_with("leading to The Vast Caverns."), "{text}");
    }

    #[test]
    fn heard_sounds_are_described_where_they_were_heard() {
        let mut w = world();
        // Along the corridor to the patrol gallery's closed door: the kobold is heard behind it.
        let steps = |dir, n| std::iter::repeat_n(Action::Move(dir), n);
        let walk: Vec<Action> = steps(Dir::N, 3)
            .chain([Action::Open(Dir::N)])
            .chain(steps(Dir::N, 2))
            .chain(steps(Dir::E, 36))
            .collect();
        for action in &walk {
            w.apply(action);
        }
        while w.perception().sounds().is_empty() && w.turn() < 200 {
            w.apply(&Action::Wait);
        }
        let view = w.perception();
        let sound = view.sounds().first().expect("footsteps behind the door");
        let text = describe(&view, sound.pos);
        assert!(
            text == "You hear footsteps here." || text == "You faintly hear footsteps here.",
            "{text}"
        );
    }

    #[test]
    fn articles() {
        assert_eq!(with_article("open door"), "an open door");
        assert_eq!(with_article("stone wall"), "a stone wall");
        assert_eq!(capitalize("an altar"), "An altar");
    }
}
