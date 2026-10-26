//! Special rooms in the turn pipeline (spec §7 "Special rooms"): what each effect does, the
//! first-entry message, what the player knows of a room, and saving.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::noise::{self, Act};

/// A hall, 25 wide: the west part plain, the east part (walls included) the special room
/// `effect`. The player stands in the plain part unless the map puts them elsewhere.
const HALL: &str = "\
#########################
#.......................#
#.......................#
#...@...................#
#.......................#
#.......................#
#########################
";
/// The special room's area in [`HALL`]: from x = 14 east, walls included.
const ROOM: [i32; 4] = [14, 0, 24, 6];

/// The built-in content with a level `map` (ambient light `ambient`, legend `legend`) whose
/// [`ROOM`] is the special room `effect` (none if empty), and a scenario starting there.
fn world_in(map: &str, ambient: u16, effect: &str, legend: &str) -> World {
    let mut files = ContentFiles::builtin();
    let room = if effect.is_empty() {
        String::new()
    } else {
        let [x0, y0, x1, y1] = ROOM;
        format!("[[room]]\neffect = \"{effect}\"\narea = [{x0}, {y0}, {x1}, {y1}]\n")
    };
    files.insert(
        "levels/test_room.toml",
        &format!(
            "name = \"Test\"\nambient = [{ambient}, {ambient}, {ambient}, 0]\nmap = '''\n{map}'''\n\
             {room}[legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n{legend}"
        ),
    );
    files.insert(
        "scenarios/test_room.toml",
        "name = \"T\"\nstart_level = \"test_room\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_room", None).expect("world")
}

fn hall(effect: &str) -> World {
    world_in(HALL, 120, effect, "")
}

/// Puts the player at `p` (via saved state, as a save/restore would).
fn teleport(world: &World, p: Pos) -> World {
    let mut state = world.state().clone();
    let player = state.player;
    state.actors.get_mut(&player).expect("player").pos = p;
    World::from_state(world.content().clone(), state).expect("valid state")
}

fn apply(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn last_message(world: &World) -> String {
    (world.messages().iter().last()).map_or(String::new(), |m| m.text.clone())
}

#[test]
fn the_room_effect_is_asked_of_any_visited_level() {
    let w = hall("hushed");
    let inside = Pos::new(18, 3);
    assert_eq!(
        w.effects_at("test_room", inside)
            .room
            .map(|e| e.id.as_str()),
        Some("hushed")
    );
    assert_eq!(
        w.effects_at("test_room", Pos::new(14, 0))
            .room
            .map(|e| e.id.as_str()),
        Some("hushed"),
        "walls too"
    );
    assert!(w.effects_at("test_room", Pos::new(13, 3)).room.is_none());
    assert!(
        w.effects_at("hub", inside).room.is_none(),
        "a level not visited"
    );
    // The showcase's stairs nook is a sanctuary.
    let content = Arc::new(Content::builtin().expect("content"));
    let showcase = World::new(content, "showcase", None).expect("showcase");
    let nook = showcase.effects_at("hub", Pos::new(75, 16)).room;
    assert_eq!(nook.map(|e| e.id.as_str()), Some("sanctuary"));
}

#[test]
fn shadowed_rooms_keep_under_a_third_of_all_light() {
    // Bright ambient light and the player's lantern, from just outside the room.
    let plain = teleport(&world_in(HALL, 255, "", ""), Pos::new(13, 3));
    let dark = teleport(&world_in(HALL, 255, "shadowed", ""), Pos::new(13, 3));
    let (mut plain, mut dark) = (plain, dark);
    plain.apply(&Action::ToggleLantern);
    dark.apply(&Action::ToggleLantern);
    let (pv, dv) = (plain.perception(), dark.perception());
    let mut inside = 0;
    for y in 0..pv.height() {
        for x in 0..pv.width() {
            let p = Pos::new(x, y);
            let (Some(full), Some(kept)) = (pv.light(p), dv.light(p)) else {
                continue;
            };
            let (full, kept) = (full.light(), kept.light());
            if dark.effects_at("test_room", p).room.is_some() {
                // Every part of the light (ambient, bounce, the lantern) keeps 30 %, each
                // rounded down.
                for c in 0..3 {
                    let all = u32::from(full.rgb[c]) * 30 / 100;
                    let k = u32::from(kept.rgb[c]);
                    assert!(k <= all && k + 3 >= all, "{p}: {kept:?} of {full:?}");
                }
                inside += 1;
            } else {
                assert_eq!(kept, full, "{p}: light outside the room is untouched");
            }
        }
    }
    assert!(inside > 10, "the room is in view");
    super::light_tests::check_invariants(&dark).expect("light invariants");
}

#[test]
fn creatures_hide_in_a_shadowed_room_and_are_seen_out_of_it() {
    // Ambient light that normal eyes see by; a third of it they do not.
    let legend = "\"d\" = { terrain = \"floor\", creature = \"deer\" }\n";
    let map = "\
#########################
#.......................#
#.......................#
#...@...................#
#.......................#
#..................d....#
#########################
";
    let seen = |effect: &str| {
        let w = world_in(map, 60, effect, legend);
        let view = w.perception();
        view.actors().iter().any(|a| a.kind == ActorKind::Creature)
    };
    assert!(seen(""), "the deer is seen in the open");
    assert!(!seen("shadowed"), "the deer is hidden in the shadowed room");
}

#[test]
fn echoing_halls_double_noise_and_hushed_chambers_quiet_it() {
    let loudness = |effect: &str, at: Pos, act: Act| {
        let w = hall(effect);
        let state = &w.state;
        let player = state.player();
        let burden = state.load(&w.content, player).penalty();
        let mut actor = player.clone();
        actor.pos = at;
        noise::noise(&w.content, state.current(), &actor, burden, act).map_or(0, |n| n.loudness)
    };
    let (outside, inside) = (Pos::new(5, 3), Pos::new(18, 3));
    // The player's footsteps on stone floor: 60.
    assert_eq!(loudness("", inside, Act::Step(inside)), 60);
    assert_eq!(loudness("echoing", outside, Act::Step(outside)), 60);
    assert_eq!(loudness("echoing", inside, Act::Step(inside)), 120);
    assert_eq!(loudness("hushed", inside, Act::Step(inside)), 24);
    // Fighting inside, too.
    let melee = Act::Attack(crate::action::AttackKind::Melee);
    assert_eq!(loudness("echoing", inside, melee), 120);
    assert_eq!(loudness("hushed", inside, melee), 24);
}

#[test]
fn an_echoing_hall_carries_steps_farther() {
    // A giant rat (hearing 5) 11 cells west of where the player steps: steps 60 loud die out on
    // the way (10 a cell), echoing ones (120) reach it.
    let legend = "\"r\" = { terrain = \"floor\", creature = \"rat\" }\n";
    let map = "\
#########################
#.......................#
#.......................#
#...r...................#
#.......................#
#......................@#
#########################
";
    let heard = |effect: &str| {
        let w = world_in(map, 0, effect, legend);
        let mut w = teleport(&w, Pos::new(14, 3));
        w.apply(&Action::Move(Dir::E));
        let rat = (w.state.actors.values()).find(|a| a.kind == ActorKind::Creature);
        // The rat acted after the step and spent what it heard; it went to look if it heard.
        rat.is_some_and(|r| r.mind.noise.is_some() || r.pos != Pos::new(4, 3))
    };
    assert!(!heard(""), "plain steps are not heard 11 cells away");
    assert!(!heard("hushed"));
    assert!(heard("echoing"), "echoing steps are");
}

#[test]
fn sanctuaries_heal_three_times_as_fast_and_glow() {
    let healed = |effect: &str, at: Pos| {
        let w = hall(effect);
        let mut state = w.state().clone();
        let player = state.player;
        let actor = state.actors.get_mut(&player).expect("player");
        actor.pos = at;
        actor.hp = 1;
        let mut w = World::from_state(w.content().clone(), state).expect("state");
        // Four minutes: one hit point back at Toughness 10.
        for _ in 0..40 {
            w.apply(&Action::Wait);
        }
        w.perception().own_hp().0 - 1
    };
    let (outside, inside) = (Pos::new(5, 3), Pos::new(18, 3));
    assert_eq!(healed("", inside), 1);
    assert_eq!(healed("sanctuary", outside), 1);
    assert_eq!(healed("sanctuary", inside), 3);
    // In the dark, the sanctuary's glow lights it faintly, enough for normal eyes.
    let dark = world_in(HALL, 0, "sanctuary", "");
    let w = teleport(&dark, Pos::new(13, 3));
    let view = w.perception();
    assert!(view.is_visible(Pos::new(17, 3)), "the glowing room is seen");
    assert!(!view.is_visible(Pos::new(10, 3)), "the dark hall is not");
    let glow = view.light(Pos::new(17, 3)).expect("visible").light();
    assert!(
        glow.brightness() >= 40 && glow.brightness() < 80,
        "faint: {glow:?}"
    );
    assert!(glow.rgb[0] > glow.rgb[2], "warm: {glow:?}");
}

#[test]
fn hostile_creatures_do_not_step_into_a_sanctuary() {
    // A giant rat (hostile), within its sight of the player in the sanctuary, and a peaceful
    // kobold walking a round through the room.
    let legend = "\"r\" = { terrain = \"floor\", creature = \"rat\" }\n\
                  \"k\" = { terrain = \"floor\", creature = \"kobold\", faction = \"peaceful\", \
                  route = \"E14 W14\" }\n";
    let map = "\
#########################
#k......................#
#.......................#
#...........r...........#
#.......................#
#......................@#
#########################
";
    let w = world_in(map, 120, "sanctuary", legend);
    let mut w = teleport(&w, Pos::new(19, 3));
    let mut kobold_inside = false;
    for _ in 0..40 {
        w.apply(&Action::Wait);
        for a in w
            .state
            .actors
            .values()
            .filter(|a| a.kind == ActorKind::Creature)
        {
            let inside = w.effects_at("test_room", a.pos).room.is_some();
            match w.content.creature(a.creature).id.as_str() {
                "rat" => assert!(!inside, "the rat stays out, at {}", a.pos),
                _ => kobold_inside |= inside,
            }
        }
    }
    assert!(
        kobold_inside,
        "the peaceful kobold walks its round through the room"
    );
    // It cannot reach the player, so it waits where it is.
    let rat = (w.state.actors.values()).find(|a| w.content.creature(a.creature).id == "rat");
    assert_eq!(rat.map(|r| r.pos), Some(Pos::new(12, 3)), "the rat waits");
    assert_eq!(
        w.perception().own_hp().0,
        w.perception().own_hp().1,
        "unhurt"
    );
    // Out of the room, it comes for the player.
    let mut w = teleport(&w, Pos::new(10, 3));
    for _ in 0..3 {
        w.apply(&Action::Wait);
    }
    let rat = (w.state.actors.values()).find(|a| w.content.creature(a.creature).id == "rat");
    assert!(
        rat.is_some_and(|r| r.pos.chebyshev(Pos::new(10, 3)) <= 1),
        "it closes in"
    );
}

#[test]
fn the_first_entry_is_told_once_and_remembered() {
    let mut w = hall("echoing");
    let at_edge = Pos::new(13, 3);
    w = teleport(&w, at_edge);
    assert!(
        w.perception().room(Pos::new(18, 3)).is_none(),
        "seen, but not known"
    );
    w.apply(&Action::Move(Dir::E));
    assert_eq!(last_message(&w), "Your footsteps echo loudly.");
    assert_eq!(
        w.perception().room(Pos::new(18, 3)).map(|e| e.id.as_str()),
        Some("echoing")
    );
    // Repeats in a row fold into one entry: count them all.
    let told = |w: &World| -> u32 {
        (w.messages().iter())
            .filter(|m| m.text == "Your footsteps echo loudly.")
            .map(|m| m.count)
            .sum()
    };
    apply(&mut w, "move:W move:W move:E move:E move:E");
    assert_eq!(told(&w), 1, "once per room");
    // Saved and loaded, the room is still known, and never told again.
    let bytes = crate::save::encode(&w);
    let mut back = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(back.state(), w.state());
    assert_eq!(back.state_hash(), w.state_hash());
    assert!(back.state.current().rooms[0].entered);
    apply(&mut back, "move:W move:W move:E move:E");
    assert_eq!(told(&back), 1);
}

#[test]
fn arriving_in_a_special_room_tells_it_too() {
    // The scenario starts the player inside the room.
    let map = "\
#########################
#.......................#
#.......................#
#..................@....#
#.......................#
#.......................#
#########################
";
    let w = world_in(map, 120, "hushed", "");
    assert_eq!(last_message(&w), "A deep hush fills this room.");
}

#[test]
fn special_rooms_replay_and_hash_alike() {
    let actions = "move:E*12 wait move:W*3 move:E*3";
    let run = || {
        let mut w = hall("sanctuary");
        apply(&mut w, actions);
        w.state_hash()
    };
    assert_eq!(run(), run());
    let mut a = hall("sanctuary");
    let mut b = hall("hushed");
    apply(&mut a, actions);
    apply(&mut b, actions);
    assert_ne!(
        a.state_hash(),
        b.state_hash(),
        "the effect is part of the state"
    );
}

#[test]
fn level_files_mark_rooms_with_known_effects_inside_the_map() {
    let attempt = |rooms: &str| {
        let mut files = ContentFiles::builtin();
        files.insert(
            "levels/test_room.toml",
            &format!("name = \"T\"\nmap = '''\n#####\n#...#\n#####\n'''\n{rooms}"),
        );
        Content::load(&files).map(|_| ()).map_err(|e| e.to_string())
    };
    let room =
        |effect: &str, area: &str| format!("[[room]]\neffect = \"{effect}\"\narea = {area}\n");
    assert!(attempt(&room("hushed", "[0, 0, 4, 2]")).is_ok());
    let err = attempt(&room("cursed", "[0, 0, 4, 2]")).unwrap_err();
    assert!(err.contains("unknown effect `cursed`"), "{err}");
    let err = attempt(&room("hushed", "[0, 0, 5, 2]")).unwrap_err();
    assert!(err.contains("not a rectangle inside the 5x3 map"), "{err}");
    let err = attempt(&format!(
        "{}{}",
        room("hushed", "[0, 0, 2, 2]"),
        room("echoing", "[2, 0, 4, 2]")
    ))
    .unwrap_err();
    assert!(err.contains("overlaps another room"), "{err}");
    // Vaults mark none.
    let mut files = ContentFiles::builtin();
    let vault = "name = \"V\"\nmap = '''\n##+##\n#...#\n#####\n'''\n";
    files.insert(
        "vaults/test_vault.toml",
        &format!("{vault}{}", room("hushed", "[0, 0, 2, 2]")),
    );
    let err = Content::load(&files).unwrap_err().to_string();
    assert!(err.contains("a vault cannot mark special rooms"), "{err}");
}

#[test]
fn the_light_of_a_room_stays_in_it() {
    // Whatever the room does to the light, the floor outside it is lit as without it.
    let plain = hall("");
    for effect in ["shadowed", "sanctuary", "echoing"] {
        let w = hall(effect);
        let (pv, wv) = (plain.perception(), w.perception());
        let level = w.state.current();
        for (p, t) in level.terrain.iter() {
            if w.content.terrain(*t).transparent && w.effects_at("test_room", p).room.is_none() {
                let light =
                    |v: &crate::perception::PerceptionView<'_>| v.light(p).map(|l| l.light());
                assert_eq!(light(&wv), light(&pv), "{effect} at {p}");
            }
        }
    }
}

/// More effects for the tests below: `gloom`, which keeps half the light, makes noise half again
/// as loud, and may cover a whole level; `glade`, a quickening whose `near` line is said from
/// outside.
const MORE_EFFECTS: &str = "\n[[effect]]\nid = \"gloom\"\nname = \"gloomy room\"\nmark = \"G\"\n\
    enter = \"Gloom.\"\nlevel_chance = 5\npervade = \"A gloom lies over this level.\"\n\
    light = 50\nnoise = 150\n\
    [[effect]]\nid = \"glade\"\nname = \"glade\"\nmark = \"L\"\nenter = \"A glade.\"\n\
    near = \"You glimpse green light ahead.\"\nregen = 200\n";

/// [`world_in`]'s hall with [`MORE_EFFECTS`] in the content, the special room `effect` (none if
/// empty), and `header` (a level file's top-level lines, e.g. `effect = "gloom"`) on the level;
/// a second level `other` (the same hall, plain) to travel to.
fn hall_with(effect: &str, header: &str) -> World {
    let mut files = ContentFiles::builtin();
    files.edit_table("room_effects", |text| format!("{text}{MORE_EFFECTS}"));
    let room = |effect: &str| {
        if effect.is_empty() {
            return String::new();
        }
        let [x0, y0, x1, y1] = ROOM;
        format!("[[room]]\neffect = \"{effect}\"\narea = [{x0}, {y0}, {x1}, {y1}]\n")
    };
    let level = |name: &str, extra: &str, effect: &str| {
        format!(
            "name = \"{name}\"\nambient = [120, 120, 120, 0]\n{extra}map = '''\n{HALL}'''\n{}\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n",
            room(effect)
        )
    };
    files.insert("levels/test_room.toml", &level("Test", header, effect));
    files.insert("levels/other.toml", &level("Other", "", ""));
    files.insert(
        "scenarios/test_room.toml",
        "name = \"T\"\nstart_level = \"test_room\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_room", None).expect("world")
}

#[test]
fn effects_at_folds_the_room_and_the_level() {
    let w = hall_with("shadowed", "effect = \"gloom\"\n");
    let (outside, inside) = (Pos::new(5, 3), Pos::new(18, 3));
    let here = w.effects_at("test_room", inside);
    assert_eq!(here.room.map(|e| e.id.as_str()), Some("shadowed"));
    assert_eq!(here.level.map(|e| e.id.as_str()), Some("gloom"));
    // Percents multiply: 30 % of 50 % of the light, 150 % of the noise.
    assert_eq!((here.light(), here.noise()), (15, 150));
    let there = w.effects_at("test_room", outside);
    assert_eq!((there.room, there.light()), (None, 50), "the level's alone");
    assert!(!here.shunned() && !here.is_none());
    // Noise made anywhere on the level is louder: the player's steps, 60, are 90.
    let state = &w.state;
    let mut actor = state.player().clone();
    actor.pos = outside;
    let burden = state.load(&w.content, &actor).penalty();
    let step = noise::noise(
        &w.content,
        state.current(),
        &actor,
        burden,
        Act::Step(outside),
    );
    assert_eq!(step.map(|n| n.loudness), Some(90));
    super::light_tests::check_invariants(&w).expect("light invariants");
    // A level without one: the room alone.
    let plain = hall_with("shadowed", "");
    assert_eq!(plain.effects_at("test_room", inside).light(), 30);
    assert_eq!(plain.effects_at("test_room", outside), Here::default());
}

#[test]
fn a_pervaded_level_is_told_on_every_arrival_and_saved() {
    let mut w = hall_with("", "effect = \"gloom\"\n");
    let told = |w: &World| -> u32 {
        (w.messages().iter())
            .filter(|m| m.text.starts_with("A gloom lies over this level."))
            .map(|m| m.count)
            .sum()
    };
    assert_eq!(told(&w), 1, "at the start");
    assert_eq!(
        w.perception().pervading().map(|e| e.id.as_str()),
        Some("gloom")
    );
    assert!(
        w.perception().room(Pos::new(5, 3)).is_none(),
        "look mode never names it"
    );
    w.travel("other", None).expect("travels");
    assert!(w.state.current().pervading.is_none());
    w.travel("test_room", None).expect("travels back");
    assert_eq!(told(&w), 2, "each time the player arrives");
    // With Sense of place the line says what it is.
    w.player_mut().talents.take("sense_of_place", None);
    w.travel("other", None).expect("travels");
    w.travel("test_room", None).expect("travels back");
    assert_eq!(
        last_message(&w),
        "A gloom lies over this level. The whole level is like a gloomy room."
    );
    let bytes = crate::save::encode(&w);
    let back = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(back.state(), w.state());
    assert_eq!(back.state_hash(), w.state_hash());
    assert!(back.state.current().pervading.is_some());
}

#[test]
fn level_files_name_only_effects_that_can_cover_a_level() {
    let attempt = |header: &str| {
        let mut files = ContentFiles::builtin();
        files.edit_table("room_effects", |text| format!("{text}{MORE_EFFECTS}"));
        files.insert(
            "levels/test_room.toml",
            &format!("name = \"T\"\n{header}map = '''\n#####\n#...#\n#####\n'''\n"),
        );
        Content::load(&files).map(|_| ()).map_err(|e| e.to_string())
    };
    assert!(attempt("effect = \"gloom\"\n").is_ok());
    for (header, says) in [
        ("effect = \"lava\"\n", "unknown effect `lava`"),
        ("effect = \"sanctuary\"\n", "cannot cover a whole level"),
        ("effect = \"flooded\"\n", "cannot cover a whole level"),
        ("effect = \"glade\"\n", "cannot cover a whole level"),
        (
            "kind = \"overworld\"\neffect = \"gloom\"\n",
            "the overworld has no `effect`",
        ),
    ] {
        let err = attempt(header).unwrap_err();
        assert!(err.contains(says), "{header}: {err}");
    }
}

#[test]
fn a_room_seen_from_outside_says_its_near_line_once() {
    let near = "You glimpse green light ahead.";
    let told = |w: &World| -> u32 {
        (w.messages().iter())
            .filter(|m| m.text == near)
            .map(|m| m.count)
            .sum()
    };
    // The player starts in the plain part of the hall with the room in view.
    let mut w = hall_with("glade", "");
    w.apply(&Action::Wait);
    assert_eq!(told(&w), 1);
    assert!(w.state.current().rooms[0].glimpsed);
    assert!(
        w.perception().room(Pos::new(18, 3)).is_none(),
        "glimpsed, not known"
    );
    apply(&mut w, "wait move:E*10");
    assert_eq!(told(&w), 1, "once per room");
    assert_eq!(
        w.perception().room(Pos::new(18, 3)).map(|e| e.id.as_str()),
        Some("glade")
    );
    // A player first seeing it from inside hears only its `enter` line.
    let mut inside = teleport(&hall_with("glade", ""), Pos::new(18, 3));
    inside.state.current_mut().rooms[0].glimpsed = false;
    inside.apply(&Action::Wait);
    assert_eq!(told(&inside), 0);
    assert!(inside.state.current().rooms[0].glimpsed);
}

#[test]
fn sense_of_place_names_a_glimpsed_room() {
    let mut w = hall_with("echoing", "");
    w.player_mut().talents.take("sense_of_place", None);
    w.apply(&Action::Wait);
    assert_eq!(
        w.perception().room(Pos::new(18, 3)).map(|e| e.id.as_str()),
        Some("echoing"),
        "known from a glimpse"
    );
    assert!(!w.state.current().rooms[0].entered);
}

#[test]
fn hallowed_rest_quickens_a_room_s_quickening_half_again() {
    let mut w = hall("sanctuary");
    let player = w.state.player().clone();
    assert_eq!(w.room_pace(&player, 300), 300);
    w.player_mut().talents.take("hallowed_rest", None);
    let player = w.state.player().clone();
    assert_eq!(
        w.room_pace(&player, 300),
        400,
        "half again of the quickening"
    );
    assert_eq!(w.room_pace(&player, 200), 250);
    assert_eq!(
        w.room_pace(&player, 100),
        100,
        "no quickening, nothing more"
    );
    assert_eq!(w.room_pace(&player, 40), 40, "a slowing is not quickened");
}

#[test]
fn a_black_chamber_keeps_no_light_at_all() {
    let mut w = teleport(&world_in(HALL, 255, "black", ""), Pos::new(13, 3));
    w.apply(&Action::ToggleLantern);
    let view = w.perception();
    for x in 15..24 {
        assert!(
            !view.is_visible(Pos::new(x, 3)),
            "normal eyes see nothing inside at {x}"
        );
    }
    assert!(view.is_visible(Pos::new(10, 3)), "the hall outside is lit");
    assert_eq!(view.light_kept(Pos::new(10, 3)), Some(100));
    super::light_tests::check_invariants(&w).expect("light invariants");
}
