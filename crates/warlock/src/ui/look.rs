//! Look mode (spec §6): a cursor the player moves over the map to read what is there. It is a
//! free action — no game time passes — and it only describes what the player perceives.

use crate::render::console::{Cell, Console};
use warlock_core::Pos;
use warlock_core::color::Rgb;
use warlock_core::content::{ActionKind, Content, ServiceKind, Stance, ZoneKind, and_list};
use warlock_core::damage::ResistanceWord;
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

/// What the player's Knowledge tells of the creature at `p`, if it tells anything: "You know the
/// skeleton is immune to poison, resistant to piercing, and vulnerable to bludgeoning." Words of
/// the `stats` layer's resistances, never a number (spec §8 "Skills": Knowledge).
pub fn known(view: &PerceptionView<'_>, p: Pos) -> Option<String> {
    let content = view.content();
    let actor = view.actor_at(p)?;
    let known = view.known_resistances(actor);
    let parts: Vec<String> = [
        ResistanceWord::Immune,
        ResistanceWord::Resistant,
        ResistanceWord::Vulnerable,
    ]
    .into_iter()
    .filter_map(|word| {
        let kinds: Vec<&str> = (known.iter())
            .filter(|(_, w)| *w == word)
            .map(|(k, _)| content.damage_type(*k).name.as_str())
            .collect();
        (!kinds.is_empty()).then(|| format!("{} to {}", word.word(), and_list(&kinds)))
    })
    .collect();
    if parts.is_empty() {
        return None;
    }
    let name = &content.creature(actor.creature).name;
    let who = match proper_name(content, actor) {
        Some(proper) => proper.to_string(),
        None => {
            let the = if actor.ally { "your" } else { "the" };
            format!("{the} {name}{}", made_of(content, actor).0)
        }
    };
    let parts: Vec<&str> = parts.iter().map(String::as_str).collect();
    Some(format!("You know {who} is {}.", and_list(&parts)))
}

/// What look mode says of a zone on a cell (`world/zone.rs`).
fn zone_words(kind: ZoneKind) -> &'static str {
    match kind {
        ZoneKind::Fire => "wreathed in fire",
        ZoneKind::Fouled => "under a foul reek",
        ZoneKind::Slick => "slick underfoot",
        ZoneKind::Dark => "under a pall of dark",
    }
}

/// [`describe`] without a place's note.
fn describe_cell(view: &PerceptionView<'_>, p: Pos, blood: bool) -> String {
    let content = view.content();
    // Stairs and locations name where they lead: "a staircase down, leading to The Depths 1";
    // a named place its name: "a volcano, Fyrbeorg";
    // a wall in sight that was dug into says so: "a stone wall, partly dug";
    // a wall found to sound hollow says so: "a hollow stone wall".
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
            // A wall the player found sounds hollow (a sealed room's breach): "a hollow stone
            // wall".
            (None, _) if view.hollow(p) => format!("hollow {}", def.name),
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
    // Ground hallowed by Sanctuary: "on hallowed ground" (the dark powers': "on dreaded ground").
    let hallow = (view.hallow(p).filter(|_| walkable))
        .map(|dark| format!("on {} ground", if dark { "dreaded" } else { "hallowed" }));
    let room = match (room, hallow) {
        (Some(room), Some(hallow)) => Some(format!("{room}, {hallow}")),
        (room, hallow) => room.or(hallow),
    };
    // A zone lying on it: "stone floor under a foul reek".
    let room = match (room, view.zone_at(p).map(zone_words)) {
        (Some(room), Some(zone)) => Some(format!("{room}, {zone}")),
        (room, zone) => room.or(zone.map(str::to_string)),
    };
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
        // A trained eye (Appraise, Survival; #1031) tells food's freshness of what it sees.
        let fresh = matches!(view.cell(p), CellView::Visible(_))
            .then(|| view.judge(&objects[i]))
            .flatten()
            .map(|w| format!("looks {w}"));
        let tags: Vec<String> = numbers.into_iter().chain(fresh).chain(price).collect();
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
                // An ally of the player's is theirs (spec §8 "Allies"): "Your wolf (unhurt)", or
                // a named one "(your ally, unhurt)".
                let stance = match view.stance(actor) {
                    _ if actor.ally => "",
                    Stance::Hostile => "hostile, ",
                    Stance::Neutral => "",
                };
                let health = actor.health.word();
                // The status effects that show on it: ", bleeding".
                let effects: String = (view.effects_of(actor.id))
                    .filter_map(|s| content.status(s).seen.as_deref())
                    .map(|word| format!(", {word}"))
                    .collect();
                // Stalker's eye: it has not noticed the player (never said without it).
                let effects = if actor.unaware {
                    format!("{effects}, unaware of you")
                } else {
                    effects
                };
                // What lives in it shows (docs/design/monster-dread.md §5.5): "writhing".
                let writhing = match content.creature(actor.creature).dies_into {
                    Some(_) => ", writhing",
                    None => "",
                };
                let gear = format!(
                    "{writhing}{}{}",
                    made_of(content, actor).1,
                    gear(view, actor.id)
                );
                // A unique character the player knows by name: "Sister Hild (unhurt), who
                // cures and mends for gold, on plank floor."; or a named monster by its type's
                // proper name: "Black Wystan (hostile, unhurt), ...".
                let proper = proper_name(content, actor);
                let ally = if actor.ally { "your ally, " } else { "" };
                match proper {
                    Some(proper) => format!(
                        "{proper} ({ally}{stance}{health}{effects}){}{gear}, on {}{here}.",
                        services(content, actor),
                        terrain(t)
                    ),
                    // The player's kept beast by its name: "Snel, your hound (unhurt), ...".
                    None if actor.companion => format!(
                        "{}, your {name}{} ({health}{effects}){gear}, on {}{here}.",
                        (view.kept())
                            .find(|(a, _)| a.id == actor.id)
                            .map_or("Your companion", |(_, c)| c.name.as_str()),
                        made_of(content, actor).0,
                        terrain(t)
                    ),
                    // Its grade, which anyone sees (spec §8 "Grades"): "A big
                    // kobold"; messages name its type alone.
                    None => {
                        let def = content.creature(actor.creature);
                        let name = match warlock_core::grade::word(def, actor.grade) {
                            Some(word) => format!("{word} {name}"),
                            None => name.clone(),
                        };
                        format!(
                            "{} {name}{} ({stance}{health}{effects}){}{gear}, on {}{here}.",
                            if actor.ally { "Your" } else { article(&name) },
                            made_of(content, actor).0,
                            waits(content, actor),
                            terrain(t)
                        )
                    }
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

/// The proper name a seen creature goes by, if any: a unique character's or a named monster's.
fn proper_name<'a>(content: &'a Content, actor: &SeenActor) -> Option<&'a str> {
    let name = &content.creature(actor.creature).name;
    (actor.dialogue)
        .and_then(|d| content.dialogue(d).name.as_deref())
        .or_else(|| warlock_core::content::is_proper(name).then_some(name.as_str()))
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

/// ", who waits" after a seen creature's name when its dialogue knows words and offers nothing
/// else, no topic, service, or wares (the Strange Spirit: "a strange spirit (unhurt), who waits";
/// docs/design/prestige-classes.md §7); nothing for any other.
fn waits(content: &Content, actor: &SeenActor) -> &'static str {
    let Some(dialogue) = actor.dialogue else {
        return "";
    };
    let d = content.dialogue(dialogue);
    match !d.words.is_empty() && d.topics.is_empty() && d.services.is_empty() && d.offer.is_none() {
        true => ", who waits",
        false => "",
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
    fn a_companion_is_seen_wearing_its_collar() {
        let content = Arc::new(Content::builtin().expect("content"));
        let spec = "human,male,ranger".parse().expect("a character");
        let mut w = World::start_run(content, "showcase", None, Some(&spec), None, true)
            .expect("a ranger's run");
        // To the darkness chamber's amulet and out to the hound, in the light of the lantern.
        let walk = "move:N*3 open:N move:N*2 move:E*30 open:N move:N*3 move:W*6 move:N*5 move:E*6 move:S*2 move:W*2 pickup move:E*2 move:N*2 wait*2 move:W use_tool";
        for action in warlock_core::action::parse_actions(walk).expect("actions") {
            w.apply(&action);
        }
        // The amulet picked up, wherever the kit left room for it (a ranger's random scrolls).
        let amulet = (w.perception().inventory().iter())
            .position(|s| w.content().object(s.object).id == "bone_amulet")
            .expect("the amulet");
        let give = format!("give:{amulet}");
        for action in warlock_core::action::parse_actions(&give).expect("actions") {
            w.apply(&action);
        }
        let view = w.perception();
        let (ally, data) = view.companion().expect("a companion");
        assert_eq!(ally.wears.len(), 1, "it took the amulet");
        let seen = (view.actors().iter()).find(|a| a.id == ally.id);
        let at = seen.expect("the hound is in sight").pos;
        let line = describe(&view, at, false);
        assert!(
            line.starts_with(&format!("{}, your hound (unhurt), wearing a ", data.name)),
            "{line}"
        );
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

    /// A disguised mimic is a coffer in look mode, word for word; found out, it is the creature
    /// (spec §8 "Mimics").
    /// What a wizard's Knowledge tells, in the stats layer's words; nothing of a creature above
    /// it, nor of what the player cannot see as a creature (a disguised mimic). The core tests
    /// hold the level's edge (`knowledge_tests.rs`).
    #[test]
    fn knowledge_says_what_a_creature_resists_in_words() {
        let mut files = warlock_core::content::ContentFiles::builtin();
        files.insert(
            "levels/look_lore.toml",
            "name = \"Lore\"\nambient = [255, 255, 255, 0]\nmap = '''\n\
             #######\n#s.M.d#\n#@.C..#\n#######\n'''\n[legend]\n\".\" = \"floor\"\n\
             \"#\" = \"wall\"\n\
             \"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
             \"s\" = { terrain = \"floor\", creature = \"skeleton\" }\n\
             \"d\" = { terrain = \"floor\", creature = \"rotting_drake\" }\n\
             \"M\" = { terrain = \"floor\", chest = 1, mimic = true }\n\
             \"C\" = { terrain = \"chest\", chest = 1 }\n",
        );
        files.insert(
            "scenarios/look_lore.toml",
            "name = \"L\"\nstart_level = \"look_lore\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let spec = "human,male,wizard".parse().expect("a character");
        let w = World::start_run(content, "look_lore", None, Some(&spec), None, true)
            .expect("a wizard's run");
        let view = w.perception();
        assert_eq!(
            known(&view, Pos::new(1, 1)).as_deref(),
            Some(
                "You know the skeleton is immune to poison and necrotic, resistant to piercing, \
                 and vulnerable to bludgeoning."
            )
        );
        assert_eq!(known(&view, Pos::new(5, 1)), None, "above their Knowledge");
        assert_eq!(
            known(&view, Pos::new(3, 1)),
            None,
            "a chest, as they see it"
        );
        assert_eq!(known(&view, w.player_pos()), None);
        assert_eq!(known(&view, Pos::new(2, 1)), None, "nobody there");
    }

    #[test]
    fn a_disguised_mimic_looks_like_a_chest() {
        let mut files = warlock_core::content::ContentFiles::builtin();
        files.insert(
            "levels/look_mimic.toml",
            "name = \"Mimic\"\nambient = [255, 255, 255, 0]\nmap = '''\n\
             #####\n#M.C#\n#@..#\n#####\n'''\n[legend]\n\".\" = \"floor\"\n\"#\" = \"wall\"\n\
             \"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
             \"M\" = { terrain = \"floor\", chest = 1, mimic = true }\n\
             \"C\" = { terrain = \"chest\", chest = 1 }\n",
        );
        files.insert(
            "scenarios/look_mimic.toml",
            "name = \"M\"\nstart_level = \"look_mimic\"\nhp = 1000\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let mut w = World::new(content, "look_mimic", None).expect("world");
        let (mimic, coffer) = (Pos::new(1, 1), Pos::new(3, 1));
        assert_eq!(
            describe(&w.perception(), mimic, true),
            "A coffer. A small wooden box with an iron hasp."
        );
        assert_eq!(
            describe(&w.perception(), mimic, true),
            describe(&w.perception(), coffer, true)
        );
        // Touched, it is found out: a creature.
        w.apply(&Action::Move(Dir::N));
        let seen = describe(&w.perception(), mimic, true);
        assert!(seen.starts_with("A mimicling (hostile"), "{seen}");
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

    /// One who knows words and has nothing else to offer waits (the Strange Spirit,
    /// docs/design/prestige-classes.md §7); the encounter is forced, whoever the player is.
    #[test]
    fn one_with_words_and_nothing_to_ask_waits() {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut w = World::new(content.clone(), "ascmearc", None).expect("the Ascmearc");
        let walk = "move:E*7 skip:4h meet:strange_spirit_roads encounter:hail";
        for action in warlock_core::action::parse_actions(walk).expect("actions") {
            w.apply(&action);
        }
        let spirit = content.creature_id("strange_spirit").expect("the spirit");
        let view = w.perception();
        let at = (view.actors().iter())
            .find(|a| a.creature == spirit)
            .map(|a| a.pos)
            .expect("hailed, and in sight");
        let seen = describe(&w.perception(), at, true);
        assert!(
            seen.starts_with("A strange spirit (unhurt), who waits, on "),
            "{seen}"
        );
    }

    #[test]
    fn a_zone_says_what_lies_on_the_ground() {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut w = World::new(content, "showcase", None).expect("the hub");
        let me = w.player_pos();
        let spot = warlock_core::geom::Pos::new(me.x + 1, me.y);
        let words: Vec<(ZoneKind, String)> = (ZoneKind::ALL.iter())
            .map(|&kind| {
                let lay = format!("zone:{}@{},{},0", kind.id(), spot.x, spot.y);
                w.apply(&lay.parse().expect("parses"));
                (kind, describe(&w.perception(), spot, true))
            })
            .collect();
        insta::assert_debug_snapshot!(words);
    }

    #[test]
    fn hallowed_ground_says_so() {
        let content = Arc::new(Content::builtin().expect("content"));
        let spec: warlock_core::character::CharacterSpec =
            "human,female,priest".parse().expect("a character");
        let mut w = World::start_run(
            content.clone(),
            "showcase",
            Some(7),
            Some(&spec),
            None,
            true,
        )
        .expect("world");
        let factor = warlock_core::content::Factor::of(&content, Some(&spec));
        let need = content.experience().threshold_for(50, factor);
        let actions = format!("xp:{need} levelup:auto use:sanctuary");
        for action in warlock_core::action::parse_actions(&actions).expect("actions") {
            w.apply(&action);
        }
        let me = w.player_pos();
        assert_eq!(w.hallows().len(), 1, "hallowed");
        assert_eq!(
            describe(&w.perception(), me, true),
            "You, on stone floor on hallowed ground."
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
        assert!(bat.starts_with("A big cave bat (unhurt), on "), "{bat}");
    }

    #[test]
    fn a_greater_or_lesser_creature_is_named_with_its_grade() {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut w = World::new(content, "arena", None).expect("arena");
        w.apply(&Action::Wait);
        let view = w.perception();
        let content = view.content();
        let mut graded = 0;
        for a in view.actors().iter().filter(|a| a.pos != view.viewer()) {
            let def = content.creature(a.creature);
            let text = describe(&view, a.pos, true);
            match warlock_core::grade::word(def, a.grade) {
                Some(word) => {
                    graded += 1;
                    let named = format!("{word} {} (", def.name);
                    assert!(text.contains(&named), "{text}");
                }
                None => assert!(text.contains(&format!(" {} (", def.name)), "{text}"),
            }
        }
        assert!(graded > 0, "some creature in view is graded");
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
            archer.starts_with("A big kobold archer (hostile, ")
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
    fn what_lives_in_a_wormhost_shows_writhing() {
        let hall = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
#######
#.w@h.#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"w" = { terrain = "floor", creature = "wormhost" }
"h" = { terrain = "floor", creature = "charnel_heap" }
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
        // Either may be graded ("A withered charnel heap"): the grade word is not the point.
        let host = describe(&view, Pos::new(2, 1), true);
        assert!(
            host.ends_with("wormhost (hostile, unhurt), writhing, on stone floor."),
            "{host}"
        );
        let heap = describe(&view, Pos::new(4, 1), true);
        assert!(
            heap.ends_with("charnel heap (hostile, unhurt), on stone floor."),
            "{heap}"
        );
    }

    #[test]
    fn a_wall_found_to_sound_hollow_says_so() {
        let hall = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
#########
#@.#....#
#..#....#
#########
'''
[[room]]
effect = "great_treasury"
area = [3, 0, 8, 3]
breach = [3, 1]
[legend]
"@" = { terrain = "floor", spawn = "player" }
"##;
        let mut files = warlock_core::content::ContentFiles::builtin();
        files.insert("levels/hall.toml", hall);
        files.insert(
            "scenarios/hall.toml",
            "name = \"Hall\"\nstart_level = \"hall\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let mut w = World::new(content, "hall", None).expect("hall");
        let breach = Pos::new(3, 1);
        assert_eq!(describe(&w.perception(), breach, true), "A stone wall.");
        w.apply(&Action::Move(Dir::E));
        for _ in 0..20 {
            if w.perception().hollow(breach) {
                break;
            }
            for action in warlock_core::action::parse_actions("search:100").unwrap() {
                w.apply(&action);
            }
        }
        assert_eq!(
            describe(&w.perception(), breach, true),
            "A hollow stone wall."
        );
    }

    #[test]
    fn a_composite_is_seen_by_its_gear() {
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
        let talents = ["healthy", "lean"].map(String::from).to_vec();
        let spec = spec.expect("a character").with_talents(talents);
        let w = World::start_run(content.clone(), "hall", Some(1), Some(&spec), None, true)
            .expect("hall");
        let view = w.perception();
        // The gear line names what it wears and wields, unidentified.
        said.extend([Pos::new(2, 1), Pos::new(4, 1)].map(|p| describe(&view, p, true)));
        insta::assert_snapshot!(said.join("\n"));
    }

    #[test]
    fn an_ally_is_yours() {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut w = World::new(content, "showcase", None).expect("showcase");
        for action in warlock_core::action::parse_actions("ally:dire_warg").expect("actions") {
            w.apply(&action);
        }
        let view = w.perception();
        let ally = (view.actors().iter())
            .find(|a| a.ally)
            .expect("the warg beside the player");
        assert_eq!(
            describe(&view, ally.pos, true),
            "Your dire warg (unhurt), on stone floor."
        );
    }

    #[test]
    fn stalkers_eye_tells_a_creature_unaware_of_you() {
        // A dark hall: the dwarf sees the deer by darkvision; the deer's eyes miss them.
        let hall = r##"
name = "Hall"
ambient = [0, 0, 0, 0]
map = '''
#######
#@..d.#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"d" = { terrain = "floor", creature = "deer" }
"##;
        let mut files = warlock_core::content::ContentFiles::builtin();
        files.insert("levels/hall.toml", hall);
        files.insert(
            "scenarios/hall.toml",
            "name = \"Hall\"\nstart_level = \"hall\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let spec = "dwarf,male,ranger"
            .parse::<warlock_core::character::CharacterSpec>()
            .expect("a character")
            .with_talents(["healthy", "lean"].map(String::from).to_vec());
        // A ranger gains the talent at level 2.
        let said: Vec<String> = ["", "xp:691 levelup:auto*2"]
            .into_iter()
            .map(|actions| {
                let mut w =
                    World::start_run(content.clone(), "hall", Some(1), Some(&spec), None, true)
                        .expect("starts");
                for action in warlock_core::action::parse_actions(actions).expect("actions") {
                    w.apply(&action);
                }
                describe(&w.perception(), Pos::new(4, 1), true)
            })
            .collect();
        assert_eq!(said[0], "A big deer (unhurt), on stone floor.");
        assert_eq!(
            said[1],
            "A big deer (unhurt, unaware of you), on stone floor."
        );
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
