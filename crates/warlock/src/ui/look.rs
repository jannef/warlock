//! Look mode (spec §6): a cursor the player moves over the map to read what is there. It is a
//! free action — no game time passes — and it only describes what the player perceives.

use crate::render::console::{Cell, Console};
use warlock_core::Pos;
use warlock_core::color::Rgb;
use warlock_core::content::{ActionKind, Content, ServiceKind, Stance};
use warlock_core::magic::Numbers;
use warlock_core::object::{Stack, phrase, phrase_with};
use warlock_core::perception::{CellView, PerceptionView, SeenActor};
use warlock_core::world::{ActorId, ActorKind};

/// Cursor colors: dark glyph on a warm highlight, so it reads on any terrain and under any light.
pub const CURSOR_FG: Rgb = Rgb::new(0x14, 0x12, 0x10);
pub const CURSOR_BG: Rgb = Rgb::new(0xe0, 0xc0, 0x70);

/// One line describing what the player knows about `p`: who is there, what lies there (as seen
/// or remembered), on what (a known trap on its terrain: "on a pit on stone floor"), in which
/// special room (once they have been in it), and what was heard there. Blood they know of there
/// reads before the terrain ("on blood-stained stone floor") if `blood` is shown. A named place
/// with a note adds it after: "A mountain pass, Beorggeat. The way you came; snow has closed it
/// behind you."; so does terrain with a description (a forgotten altar's).
pub fn describe(view: &PerceptionView<'_>, p: Pos, blood: bool) -> String {
    let text = describe_cell(view, p, blood);
    let text = match view.place(p).and_then(|place| place.note.as_deref()) {
        Some(note) => format!("{text} {note}"),
        None => text,
    };
    // Terrain with a description of its own (a forgotten altar) adds it after; a chest its
    // band's (spec §8 "Chests").
    let terrain = match view.cell(p) {
        CellView::Visible(t) | CellView::Remembered(t) => Some(t),
        CellView::Unknown => None,
    };
    let content = view.content();
    let band = (view.chest_at(p)).and_then(|c| content.chests().bands.get(c.band));
    let described = match band {
        Some(band) => Some(band.description.as_str()),
        None => terrain.and_then(|t| content.terrain(t).description.as_deref()),
    };
    match described {
        Some(description) => format!("{text} {description}"),
        None => text,
    }
}

/// [`describe`] without a place's note.
fn describe_cell(view: &PerceptionView<'_>, p: Pos, blood: bool) -> String {
    let content = view.content();
    // Stairs and locations name where they lead: "a staircase down, leading to The Depths 1";
    // a named place its name: "a volcano, Fyrbeorg";
    // a wall in sight that was dug into says so: "a stone wall, partly dug".
    // A chest goes by its band: "a coffer", "an open coffer", "a locked coffer" (Chest-wise);
    // its known trap after it: "a coffer, trapped: an arrow trap".
    let chest = view.chest_at(p);
    let chest_trap = (view.trap(p).filter(|_| chest.is_some())).map(|t| &content.trap(t).name);
    let named = |t| {
        let def = content.terrain(t);
        let def_name = match (chest, chest_trap) {
            (Some(chest), Some(trap)) => format!(
                "{}, trapped: {}",
                chest.name(content.chests(), &def.name),
                with_article(trap)
            ),
            (Some(chest), None) => chest.name(content.chests(), &def.name),
            (None, _) => def.name.clone(),
        };
        let name = match view.blood(p).filter(|_| blood) {
            Some(stain) => format!("{} {def_name}", stain.word(!def.transparent)),
            None => def_name,
        };
        match view.leads_to(p) {
            Some(to) => format!("{name}, leading to {to}"),
            None if view.is_dug(p) => format!("{name}, partly dug"),
            None => match view.place(p) {
                Some(place) => format!("{name}, {}", place.name),
                None => name,
            },
        }
    };
    // A special room the player has been in names the ground in it: "in an echoing hall".
    let walkable = match view.cell(p) {
        CellView::Visible(t) | CellView::Remembered(t) => content.terrain(t).walkable,
        CellView::Unknown => false,
    };
    let room = (view.room(p))
        .filter(|_| walkable)
        .map(|r| format!("in {}", r.with_article()));
    let ground = |t| match (&room, view.leads_to(p)) {
        (Some(room), Some(_)) => format!("{}, {room}", named(t)),
        (Some(room), None) => format!("{} {room}", named(t)),
        (None, _) => named(t),
    };
    let room_here = room.as_ref().map_or(String::new(), |r| format!(", {r}"));
    // A known trap is part of the ground: "a fireball trap on stone floor".
    let trap = (view.trap(p).filter(|_| chest.is_none())).map(|t| content.trap(t).name.as_str());
    let terrain = |t| match trap {
        Some(trap) => format!("{} on {}", with_article(trap), ground(t)),
        None => ground(t),
    };
    // Nothing but a trap there: the trap first ("A pit, on stone floor.").
    if let Some(trap) = trap
        && view.actor_at(p).is_none()
        && view.objects(p).is_empty()
    {
        let a_trap = with_article(trap);
        return match (view.cell(p), view.sound_at(p)) {
            (CellView::Visible(t), _) => format!("{}, on {}.", capitalize(&a_trap), ground(t)),
            (CellView::Remembered(t), Some(heard)) => format!(
                "{} here, where you remember {a_trap} on {}.",
                heard.you_hear(),
                ground(t)
            ),
            (CellView::Remembered(t), None) => {
                format!("You remember {a_trap} here, on {}.", ground(t))
            }
            (CellView::Unknown, _) => "You don't know what is there.".to_string(),
        };
    }
    // A sound heard there (never on a cell the player sees): "You hear footsteps here."
    let heard = view.sound_at(p).map(|s| s.you_hear());
    // "a dagger", "a dagger and 12 arrows", "17 objects".
    let objects = view.objects(p);
    // Each says what it gives in numbers (spec §6 "Item lists", `magic::numbers`), and goods for
    // sale in a shop their price (spec §7 "Shops"): "a dagger (+0 hit, 1d4, range 5; for sale,
    // 45 gold)".
    let tag = |i: usize| {
        let numbers = Numbers::of(&objects[i], view.content()).text(view.content());
        let price = (view.price_tag(p, &objects[i])).map(|g| format!("for sale, {g} gold"));
        let tags: Vec<String> = numbers.into_iter().chain(price).collect();
        (!tags.is_empty()).then(|| tags.join("; "))
    };
    let things = (!objects.is_empty()).then(|| phrase_with(objects, view.lore(), tag));
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
            // The player's own image in a hall of mirrors.
            (Some(actor), _) if actor.kind == ActorKind::Player => {
                format!("Your reflection, on {}{here}.", terrain(t))
            }
            (Some(actor), _) => {
                let name = &content.creature(actor.creature).name;
                // Its type tells the player how it stands toward them (its faction), and they
                // see how hurt it is: in words, never in hit points.
                let stance = match view.stance(actor) {
                    Stance::Hostile => "hostile, ",
                    Stance::Neutral => "",
                };
                let health = actor.health.word();
                // The status effects that show on it: ", bleeding".
                let effects: String = (view.effects_of(actor.id))
                    .filter_map(|s| content.status(s).seen.as_deref())
                    .map(|word| format!(", {word}"))
                    .collect();
                let gear = format!("{}{}", made_of(content, actor).1, gear(view, actor.id));
                // Appraiser's eye: something an animated thing is made of is magic.
                let hums = if actor.hums {
                    " It hums with power."
                } else {
                    ""
                };
                // A unique character the player knows by name: "Sister Hild (unhurt), who
                // cures and mends for gold, on plank floor."; or a named monster by its type's
                // proper name: "Black Wystan (hostile, unhurt), ...".
                let proper = (actor.dialogue)
                    .and_then(|d| content.dialogue(d).name.as_deref())
                    .or_else(|| warlock_core::content::is_proper(name).then_some(name.as_str()));
                match proper {
                    Some(proper) => format!(
                        "{proper} ({stance}{health}{effects}){}{gear}, on {}{here}.{hums}",
                        services(content, actor),
                        terrain(t)
                    ),
                    None => format!(
                        "{} {name}{} ({stance}{health}{effects}){gear}, on {}{here}.{hums}",
                        article(name),
                        made_of(content, actor).0,
                        terrain(t)
                    ),
                }
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

/// What a seen creature sells as a service, as it reads after its name: ", who cures and mends
/// for gold"; nothing if it sells none (campaign design §6.1).
fn services(content: &Content, actor: &SeenActor) -> String {
    let Some(dialogue) = actor.dialogue else {
        return String::new();
    };
    let mut verbs: Vec<&str> = Vec::new();
    for service in &content.dialogue(dialogue).services {
        let verb = match service.does {
            ServiceKind::Cure => "cures",
            ServiceKind::Identify => "identifies",
            ServiceKind::Mend => "mends",
        };
        if !verbs.contains(&verb) {
            verbs.push(verb);
        }
    }
    match verbs.as_slice() {
        [] => String::new(),
        [one] => format!(", who {one} for gold"),
        [rest @ .., last] => format!(", who {} and {last} for gold", rest.join(", ")),
    }
}

/// What a seen animated thing is made of and how it moves (spec §8 "Animated objects"), as they
/// read after its name and after how hurt it looks: (" of silver", ", rocking"); nothing for any
/// other creature.
fn made_of(content: &Content, actor: &SeenActor) -> (String, String) {
    // A household thing's body; a blade's weapon is told by the gear line instead.
    let Some(shape) = actor
        .shape
        .filter(|_| content.animated_body(actor.creature).is_some())
    else {
        return Default::default();
    };
    let moves = (content.animated_body(actor.creature))
        .and_then(|b| b.spec.moves.as_deref())
        .map_or(String::new(), |m| format!(", {m}"));
    (
        format!(" of {}", content.material(shape.material).name),
        moves,
    )
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
        .map(|(before, after, stacks)| format!(", {before}{}{after}", phrase(stacks, view.lore())))
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
    // A chest keeps its glyph over its known trap (spec §8 "Chests").
    let trap = view.trap(p).filter(|_| view.chest_at(p).is_none());
    let ch = match (view.cell(p), view.actor_at(p), top) {
        (CellView::Visible(_), Some(actor), _) => actor.glyph(content),
        _ if view.sound_at(p).is_some() => super::sounds::MARKER,
        (CellView::Visible(_) | CellView::Remembered(_), None, _) if trap.is_some() => {
            trap.map_or(' ', |t| content.trap(t).glyph)
        }
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
    fn named_places_say_their_name_and_note() {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut w = World::new(content, "ascmearc", None).expect("the Ascmearc");
        let pass = w.player_pos();
        assert_eq!(
            describe(&w.perception(), pass, true),
            "You, on mountain pass, Beorggeat. The way you came; snow has closed it behind you."
        );
        for _ in 0..3 {
            w.apply(&Action::Move(Dir::E));
        }
        let town = Pos::new(18, 28);
        assert_eq!(
            describe(&w.perception(), town, true),
            "A town, leading to Cēaptūn."
        );
        assert_eq!(
            describe(&w.perception(), pass, true),
            "A mountain pass, Beorggeat. The way you came; snow has closed it behind you."
        );
    }

    #[test]
    fn a_forgotten_altar_says_what_it_is_and_a_spent_one_too() {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut w = World::new(content, "shrine", None).expect("the shrine");
        let walk = "move:N open:N move:N*3";
        for action in warlock_core::action::parse_actions(walk).expect("actions") {
            w.apply(&action);
        }
        let altar = Pos::new(13, 3);
        assert_eq!(
            describe(&w.perception(), altar, true),
            "A forgotten altar. An altar of worn stone. Its carvings have been rubbed smooth by \
             hands or by time, and whatever god it served has no name left. Something here still \
             listens."
        );
        for _ in 0..8 {
            w.apply(&Action::Pray);
        }
        assert_eq!(
            describe(&w.perception(), altar, true),
            "A spent altar. An altar of worn stone, cold and dark. Whatever listened here has \
             stopped.",
            "eight prayers at one altar spend it, but for one chance in 256"
        );
    }

    /// A chest goes by its band, with the band's description; open, it is the pile's ground
    /// (spec §8 "Chests").
    #[test]
    fn a_chest_goes_by_its_band() {
        let mut files = warlock_core::content::ContentFiles::builtin();
        files.insert(
            "levels/look_chests.toml",
            "name = \"Chests\"\nambient = [255, 255, 255, 0]\nitem_level = 9\nmap = '''\n\
             #####\n#C.K#\n#@..#\n#####\n'''\n[legend]\n\".\" = \"floor\"\n\"#\" = \"wall\"\n\
             \"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
             \"C\" = { terrain = \"chest\", chest = \"coffer\", objects = [\"dagger\"] }\n\
             \"K\" = { terrain = \"chest\", locked = true, trap = \"arrow\", known = true }\n",
        );
        files.insert(
            "scenarios/look_chests.toml",
            "name = \"C\"\nstart_level = \"look_chests\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let mut w = World::new(content, "look_chests", None).expect("world");
        let (coffer, banded) = (Pos::new(1, 1), Pos::new(3, 1));
        assert_eq!(
            describe(&w.perception(), coffer, true),
            "A coffer. A small wooden box with an iron hasp."
        );
        assert_eq!(
            describe(&w.perception(), banded, true),
            "A banded chest, trapped: an arrow trap. A heavy chest banded with black iron, its \
             lid studded with nails.",
            "the level's own: 9; a known trap after the name; whether it is locked is not shown"
        );
        w.apply(&Action::Open(Dir::N));
        assert_eq!(
            describe(&w.perception(), coffer, true),
            "A dagger (+0 hit, 1d4, range 5), on open coffer. A small wooden box with an iron hasp."
        );
    }

    #[test]
    fn people_are_named_with_the_services_they_sell() {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut w = World::new(content, "ceaptun", None).expect("Cēaptūn");
        let walk = "move:SW*4 move:W*15 open:SW move:SW move:S*2";
        for action in warlock_core::action::parse_actions(walk).expect("actions") {
            w.apply(&action);
        }
        assert_eq!(w.player_pos(), Pos::new(26, 24), "inside the Mynster");
        assert_eq!(
            describe(&w.perception(), Pos::new(27, 25), true),
            "Sister Hild (unhurt), who cures and mends for gold, on plank floor in a sanctuary."
        );
    }

    #[test]
    fn describes_the_player_terrain_memory_and_the_unknown() {
        let mut w = world();
        let me = w.player_pos();
        assert_eq!(describe(&w.perception(), me, true), "You, on stone floor.");
        let wall = Pos::new(0, me.y);
        assert_eq!(describe(&w.perception(), wall, true), "A stone wall.");
        let door = Pos::new(me.x, me.y - 4);
        assert_eq!(describe(&w.perception(), door, true), "A closed door.");
        assert_eq!(
            describe(&w.perception(), Pos::new(70, 3), true),
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
        let text = describe(&w.perception(), me, true);
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
        let text = describe(&view, down, true);
        // The stairs nook is a sanctuary, which the player knows once they have been in it.
        assert_eq!(
            text,
            "A staircase down, leading to The Vast Caverns, in a sanctuary."
        );
        let here = describe(&view, w.player_pos(), true);
        assert_eq!(here, "You, on stone floor in a sanctuary.");
        let wall = Pos::new(79, 15);
        assert_eq!(
            describe(&view, wall, true),
            "A stone wall.",
            "walls name no room"
        );
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
        let text = describe(&view, floor, true);
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
        assert_eq!(describe(&view, inside, true), "A stone floor.");
        w.apply(&Action::Move(Dir::S));
        w.apply(&Action::Move(Dir::S));
        assert_eq!(
            describe(&w.perception(), inside, true),
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
        let text = describe(&view, sound.pos, true);
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
            describe(&w.perception(), gold, true),
            "12 gold coins, on stone floor."
        );
        assert_eq!(
            describe(&w.perception(), pile, true),
            "A leather jerkin (Dv +1, Pv 1), a dagger (+0 hit, 1d4, range 5), and 2 wax candles, \
             on stone floor."
        );
        w.apply(&Action::Move(Dir::SW));
        w.apply(&Action::Move(Dir::SW));
        assert_eq!(
            describe(&w.perception(), pile, true),
            "You, on stone floor; here: a leather jerkin (Dv +1, Pv 1), a dagger (+0 hit, 1d4, \
             range 5), and 2 wax candles."
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
        let text = describe(&w.perception(), gold, true);
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
                (id, describe(&view, a.pos, true))
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
            Some((actor.health, describe(&view, actor.pos, true)))
        });
        let (health, text) = hurt.expect("someone in view was hurt");
        assert!(text.contains(&format!(", {})", health.word())), "{text}");
        // Never a number of hit points.
        assert!(!text.chars().any(|c| c.is_ascii_digit()), "{text}");
    }

    #[test]
    fn a_bleeding_creature_looks_it() {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut w = World::new(content, "arena", None).expect("arena");
        // The brawl at the fire pit wounds someone in view before long.
        let bleeding = (0..60).find_map(|_| {
            w.apply(&Action::Wait);
            let view = w.perception();
            let actor = (view.actors().iter()).find(|a| view.effects_of(a.id).next().is_some())?;
            Some(describe(&view, actor.pos, true))
        });
        let text = bleeding.expect("someone in view bled");
        assert!(text.contains(", bleeding)"), "{text}");
        assert!(!text.chars().any(|c| c.is_ascii_digit()), "{text}");
    }

    #[test]
    fn blood_is_named_before_the_terrain_unless_hidden() {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut w = World::new(content, "arena", None).expect("arena");
        for _ in 0..40 {
            w.apply(&Action::Wait);
        }
        let view = w.perception();
        let stained = (0..view.height())
            .flat_map(|y| (0..view.width()).map(move |x| Pos::new(x, y)))
            .find(|&p| {
                view.blood(p).is_some() && view.actor_at(p).is_none() && view.objects(p).is_empty()
            })
            .expect("the brawl left blood in view");
        let text = describe(&view, stained, true);
        let word = view.blood(stained).expect("blood").word(false);
        assert_eq!(text, format!("A {word} stone floor."));
        assert_eq!(describe(&view, stained, false), "A stone floor.");
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
                (id, describe(&view, a.pos, true))
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
    fn known_traps_are_named_as_part_of_the_ground() {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut w = World::new(content, "traps", None).expect("the trap hall");
        // A hidden trap (the fireball in the crossing) is just floor.
        assert_eq!(
            describe(&w.perception(), Pos::new(27, 11), true),
            "A stone floor."
        );
        let actions = warlock_core::action::parse_actions("move:NW*14 move:W*7 move:N*2").unwrap();
        for action in actions {
            w.apply(&action);
        }
        assert_eq!(
            describe(&w.perception(), Pos::new(16, 2), true),
            "An arrow trap, on stone floor."
        );
        // On the trap with the player: part of the ground they stand on.
        w.apply(&Action::Move(Dir::N));
        let view = w.perception();
        let here = view.viewer();
        assert!(describe(&view, here, true).starts_with("You, on an arrow trap on stone floor"));
    }

    #[test]
    fn animated_things_are_seen_by_what_they_are_made_of() {
        let hall = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
#######
#.c@r.#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"c" = { terrain = "floor", creature = "animated_candlestick", animated = { material = "silver" } }
"r" = { terrain = "floor", creature = "animated_rug" }
"##;
        let mut files = warlock_core::content::ContentFiles::builtin();
        files.insert("levels/hall.toml", hall);
        files.insert(
            "scenarios/hall.toml",
            "name = \"Hall\"\nstart_level = \"hall\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let w = World::new(content, "hall", None).expect("hall");
        let view = w.perception();
        assert_eq!(
            describe(&view, Pos::new(2, 1), true),
            "An animated candlestick of silver (hostile, unhurt), rocking, on stone floor."
        );
        assert_eq!(
            describe(&view, Pos::new(4, 1), true),
            "An animated rug of wool (hostile, unhurt), rippling, on stone floor."
        );
    }

    #[test]
    fn a_composite_is_seen_by_its_gear_and_hums_to_an_appraiser() {
        let hall = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
#######
#.a@p.#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"a" = { terrain = "floor", creature = "animated_armour_12" }
"p" = { terrain = "floor", creature = "animated_panoply_70" }
"##;
        let mut files = warlock_core::content::ContentFiles::builtin();
        files.insert("levels/hall.toml", hall);
        files.insert(
            "scenarios/hall.toml",
            "name = \"Hall\"\nstart_level = \"hall\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let spec = "dwarf,male,warrior".parse::<warlock_core::character::CharacterSpec>();
        let mut said = Vec::new();
        for picks in [["healthy", "lean"], ["appraisers_eye", "healthy"]] {
            let talents = picks.map(String::from).to_vec();
            let spec = spec.clone().expect("a character").with_talents(talents);
            let w = World::start_run(content.clone(), "hall", Some(1), Some(&spec), None, true)
                .expect("hall");
            let view = w.perception();
            // The gear line names what it wears and wields, unidentified; with Appraiser's eye a
            // thing of which anything is magic hums with power.
            said.extend([Pos::new(2, 1), Pos::new(4, 1)].map(|p| describe(&view, p, true)));
        }
        insta::assert_snapshot!(said.join("\n"));
    }

    #[test]
    fn a_blade_is_seen_by_the_weapon_it_wields() {
        let hall = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
#######
#.b@c.#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"b" = { terrain = "floor", creature = "animated_blade_8" }
"c" = { terrain = "floor", creature = "animated_blade_19" }
"##;
        let mut files = warlock_core::content::ContentFiles::builtin();
        files.insert("levels/hall.toml", hall);
        files.insert(
            "scenarios/hall.toml",
            "name = \"Hall\"\nstart_level = \"hall\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let w = World::new(content, "hall", None).expect("hall");
        let view = w.perception();
        // No material word: the gear line names the weapon as anyone's, unidentified.
        let said = [Pos::new(2, 1), Pos::new(4, 1)].map(|p| describe(&view, p, true));
        insta::assert_snapshot!(said.join("\n"));
    }

    #[test]
    fn articles() {
        assert_eq!(with_article("open door"), "an open door");
        assert_eq!(with_article("stone wall"), "a stone wall");
        assert_eq!(capitalize("an altar"), "An altar");
    }
}
