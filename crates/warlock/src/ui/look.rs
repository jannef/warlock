//! Look mode (spec §6): a cursor the player moves over the map to read what is there. It is a
//! free action — no game time passes — and it only describes what the player perceives.

use crate::render::console::{Cell, Console};
use warlock_core::Pos;
use warlock_core::color::Rgb;
use warlock_core::content::{ActionKind, Stance};
use warlock_core::object::{Stack, phrase};
use warlock_core::perception::{CellView, PerceptionView};
use warlock_core::world::ActorId;

/// Cursor colors: dark glyph on a warm highlight, so it reads on any terrain and under any light.
pub const CURSOR_FG: Rgb = Rgb::new(0x14, 0x12, 0x10);
pub const CURSOR_BG: Rgb = Rgb::new(0xe0, 0xc0, 0x70);

/// One line describing what the player knows about `p`: who is there, what lies there (as seen
/// or remembered), on what, in which special room (once they have been in it), and what was
/// heard there.
pub fn describe(view: &PerceptionView<'_>, p: Pos) -> String {
    let content = view.content();
    // Stairs and locations name where they lead: "a staircase down, leading to The Depths 1";
    // a wall in sight that was dug into says so: "a stone wall, partly dug".
    let named = |t| match view.leads_to(p) {
        Some(to) => format!("{}, leading to {to}", content.terrain(t).name),
        None if view.is_dug(p) => format!("{}, partly dug", content.terrain(t).name),
        None => content.terrain(t).name.clone(),
    };
    // A special room the player has been in names the ground in it: "in an echoing hall".
    let walkable = match view.cell(p) {
        CellView::Visible(t) | CellView::Remembered(t) => content.terrain(t).walkable,
        CellView::Unknown => false,
    };
    let room = (view.room(p))
        .filter(|_| walkable)
        .map(|r| format!("in {}", r.with_article()));
    let terrain = |t| match (&room, view.leads_to(p)) {
        (Some(room), Some(_)) => format!("{}, {room}", named(t)),
        (Some(room), None) => format!("{} {room}", named(t)),
        (None, _) => named(t),
    };
    let room_here = room.as_ref().map_or(String::new(), |r| format!(", {r}"));
    // A sound heard there (never on a cell the player sees): "You hear footsteps here."
    let heard = view.sound_at(p).map(|s| s.you_hear());
    // "a dagger", "a dagger and 12 arrows", "17 objects".
    let objects = view.objects(p);
    let things = (!objects.is_empty()).then(|| phrase(objects, content));
    let here = things
        .as_ref()
        .map_or(String::new(), |o| format!("; here: {o}"));
    // What the player remembers at a cell out of sight.
    let memory = |t| match &things {
        Some(things) => format!("{things} on {}", terrain(t)),
        None => with_article(&terrain(t)),
    };
    match (view.cell(p), heard) {
        (CellView::Visible(t), _) => match (view.actor_at(p), &things) {
            (Some(_), _) if p == view.viewer() => format!("You, on {}{here}.", terrain(t)),
            (Some(actor), _) => {
                let name = &content.creature(actor.creature).name;
                // Its type tells the player how it stands toward them (its faction), and they
                // see how hurt it is: in words, never in hit points.
                let stance = match view.stance(actor) {
                    Stance::Hostile => "hostile, ",
                    Stance::Neutral => "",
                };
                let health = actor.health.word();
                let gear = gear(view, actor.id);
                format!(
                    "{} {name} ({stance}{health}){gear}, on {}{here}.",
                    article(name),
                    terrain(t)
                )
            }
            (None, Some(things)) => format!("{}, on {}.", capitalize(things), terrain(t)),
            (None, None) => format!("{}.", capitalize(&with_article(&terrain(t)))),
        },
        (CellView::Remembered(t), Some(heard)) => {
            format!("{heard} here, where you remember {}.", memory(t))
        }
        (CellView::Remembered(t), None) => match &things {
            Some(things) => format!("You remember {things} here, on {}.", terrain(t)),
            None => format!("You remember {} here{room_here}.", with_article(&named(t))),
        },
        (CellView::Unknown, Some(heard)) => format!("{heard} here."),
        (CellView::Unknown, None) => "You don't know what is there.".to_string(),
    }
}

/// What a seen creature wears and wields, as it reads after its name: ", wielding a spear,
/// wearing a leather jerkin", ", with a short bow and 20 arrows ready"; nothing if it has none.
/// That is all the player sees of what it carries.
fn gear(view: &PerceptionView<'_>, actor: ActorId) -> String {
    let content = view.content();
    let mut groups: [(&str, &str, Vec<Stack>); 3] = [
        ("wielding ", "", Vec::new()),
        ("wearing ", "", Vec::new()),
        ("with ", " ready", Vec::new()),
    ];
    for g in view.gear(actor) {
        let group = match content.slot(g.slot).action {
            ActionKind::Wield => 0,
            ActionKind::Ready => 2,
            _ => 1,
        };
        groups[group].2.push(g.stack.clone());
    }
    groups
        .iter()
        .filter(|(_, _, stacks)| !stacks.is_empty())
        .map(|(before, after, stacks)| format!(", {before}{}{after}", phrase(stacks, content)))
        .collect()
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
    draw_cursor_on(console, (x, y), view, p, CURSOR_BG);
}

/// [`draw_cursor`] on background `bg` (targeting marks a target out of reach with its own).
pub fn draw_cursor_on(
    console: &mut Console,
    (x, y): (i32, i32),
    view: &PerceptionView<'_>,
    p: Pos,
    bg: Rgb,
) {
    let content = view.content();
    let top = view.objects(p).first();
    let ch = match (view.cell(p), view.actor_at(p), top) {
        (CellView::Visible(_), Some(actor), _) => content.creature(actor.creature).glyph,
        _ if view.sound_at(p).is_some() => super::sounds::MARKER,
        (CellView::Visible(_) | CellView::Remembered(_), None, Some(top)) => {
            content.object(top.object).glyph
        }
        (CellView::Visible(t) | CellView::Remembered(t), _, _) => content.terrain(t).glyph,
        (CellView::Unknown, _, _) => ' ',
    };
    console.put(
        x,
        y,
        Cell {
            ch,
            fg: CURSOR_FG,
            bg,
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
        // The stairs nook is a sanctuary, which the player knows once they have been in it.
        assert_eq!(
            text,
            "A staircase down, leading to The Vast Caverns, in a sanctuary."
        );
        let here = describe(&view, w.player_pos());
        assert_eq!(here, "You, on stone floor in a sanctuary.");
        let wall = Pos::new(79, 15);
        assert_eq!(describe(&view, wall), "A stone wall.", "walls name no room");
        // Walking back out, the nook is remembered as a sanctuary.
        for action in [Action::Move(Dir::N), Action::Move(Dir::N)] {
            w.apply(&action);
        }
        w.apply(&Action::Move(Dir::N));
        w.apply(&Action::Move(Dir::N));
        w.apply(&Action::Move(Dir::W));
        w.apply(&Action::Move(Dir::W));
        w.apply(&Action::Move(Dir::W));
        let view = w.perception();
        let floor = Pos::new(76, 18);
        let text = describe(&view, floor);
        assert_eq!(text, "You remember a stone floor here, in a sanctuary.");
    }

    #[test]
    fn special_rooms_are_named_only_once_entered() {
        let mut w = world();
        // Up the corridor to the stairs nook's door: the nook is seen, but not yet known.
        let steps = |dir, n| std::iter::repeat_n(Action::Move(dir), n);
        let walk: Vec<Action> = steps(Dir::N, 3)
            .chain([Action::Open(Dir::N)])
            .chain(steps(Dir::N, 2))
            .chain(steps(Dir::E, 67))
            .chain([Action::Open(Dir::S)])
            .collect();
        for action in &walk {
            w.apply(action);
        }
        let inside = Pos::new(74, 13);
        let view = w.perception();
        assert!(view.is_visible(inside));
        assert_eq!(describe(&view, inside), "A stone floor.");
        w.apply(&Action::Move(Dir::S));
        w.apply(&Action::Move(Dir::S));
        assert_eq!(
            describe(&w.perception(), inside),
            "A stone floor in a sanctuary."
        );
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
    fn objects_are_named_where_they_lie_and_where_they_are_remembered() {
        let mut w = world();
        let gold = Pos::new(10, 14);
        let pile = Pos::new(5, 17);
        assert_eq!(
            describe(&w.perception(), gold),
            "12 gold coins, on stone floor."
        );
        assert_eq!(
            describe(&w.perception(), pile),
            "A leather jerkin, a dagger, and 2 wax candles, on stone floor."
        );
        w.apply(&Action::Move(Dir::SW));
        w.apply(&Action::Move(Dir::SW));
        assert_eq!(
            describe(&w.perception(), pile),
            "You, on stone floor; here: a leather jerkin, a dagger, and 2 wax candles."
        );
        // Out of the room, the gold is remembered.
        for dir in [Dir::NE, Dir::NE, Dir::N, Dir::N, Dir::N] {
            w.apply(&Action::Move(dir));
        }
        w.apply(&Action::Open(Dir::N));
        for _ in 0..3 {
            w.apply(&Action::Move(Dir::N));
        }
        w.apply(&Action::Move(Dir::E));
        for _ in 0..8 {
            w.apply(&Action::Move(Dir::E));
        }
        let text = describe(&w.perception(), gold);
        assert_eq!(text, "You remember 12 gold coins here, on stone floor.");
        let mut console = Console::default();
        console.resize(3, 1);
        draw_cursor(&mut console, 1, 0, &w.perception(), gold);
        assert_eq!(console.get(1, 0).map(|c| c.ch), Some('$'));
    }

    #[test]
    fn hostile_creatures_are_marked_as_such() {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut w = World::new(content, "arena", None).expect("arena");
        w.apply(&Action::Wait);
        let view = w.perception();
        let content = view.content();
        let described: Vec<(String, String)> = view
            .actors()
            .iter()
            .filter(|a| a.pos != view.viewer())
            .map(|a| {
                let id = content.creature(a.creature).id.clone();
                (id, describe(&view, a.pos))
            })
            .collect();
        let of = |id: &str| described.iter().find(|(c, _)| c == id).map(|(_, d)| d);
        let rat = of("rat").expect("a giant rat in view");
        assert!(
            rat.starts_with("A giant rat (hostile, unhurt), on "),
            "{rat}"
        );
        let bat = of("bat").expect("a cave bat in view");
        assert!(bat.starts_with("A cave bat (unhurt), on "), "{bat}");
    }

    #[test]
    fn creatures_look_as_hurt_as_they_are_in_words() {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut w = World::new(content, "arena", None).expect("arena");
        // The brawl at the fire pit goes on until someone in view is hurt.
        let hurt = (0..60).find_map(|_| {
            w.apply(&Action::Wait);
            let view = w.perception();
            let actor = view.actors().iter().find(|a| {
                a.pos != view.viewer() && a.health != warlock_core::combat::Health::Unhurt
            })?;
            Some((actor.health, describe(&view, actor.pos)))
        });
        let (health, text) = hurt.expect("someone in view was hurt");
        assert!(text.contains(&format!(", {})", health.word())), "{text}");
        // Never a number of hit points.
        assert!(!text.chars().any(|c| c.is_ascii_digit()), "{text}");
    }

    #[test]
    fn creatures_are_seen_with_what_they_wield_and_wear() {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut w = World::new(content, "arena", None).expect("arena");
        // The war band comes into the torchlight of the fire pit, all of it still standing.
        for _ in 0..8 {
            w.apply(&Action::Wait);
        }
        let view = w.perception();
        let content = view.content();
        let described: Vec<(String, String)> = (view.actors().iter())
            .map(|a| {
                let id = content.creature(a.creature).id.clone();
                (id, describe(&view, a.pos))
            })
            .collect();
        let of = |id: &str| {
            (described
                .iter()
                .find(|(c, _)| c == id)
                .map(|(_, d)| d.as_str()))
            .unwrap_or_else(|| panic!("a {id} in view: {described:#?}"))
        };
        // How hurt each looks comes before what it wields: "(hostile, wounded), wielding...".
        let kobold = of("kobold");
        assert!(
            kobold.starts_with("A kobold (hostile, ")
                && kobold.contains("), wielding a spear, on "),
            "{kobold}"
        );
        let archer = of("kobold_archer");
        assert!(
            archer.starts_with("A kobold archer (hostile, ")
                && archer.contains("), with a short bow and ")
                && archer.contains(" arrows ready, on "),
            "{archer}"
        );
        let shaman = of("kobold_shaman");
        assert!(
            shaman.starts_with("A kobold shaman (hostile, ")
                && shaman.contains("), wearing a bone amulet, on "),
            "{shaman}"
        );
        // Nothing else of what they carry; and nothing of what the rats do not have.
        let rat = of("rat");
        assert!(rat.starts_with("A giant rat (hostile, ") && !rat.contains("wield"));
    }

    #[test]
    fn articles() {
        assert_eq!(with_article("open door"), "an open door");
        assert_eq!(with_article("stone wall"), "a stone wall");
        assert_eq!(capitalize("an altar"), "An altar");
    }
}
