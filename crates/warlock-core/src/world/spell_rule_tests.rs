//! The spells of one small rule each (`world/spell_rule.rs`): Leech's share, Blink's draw and what
//! it ends, Thunderclap on its caster, Benison on allies alone, Rebuke on the undead alone,
//! Bramble on living ground, Gale's order of pushes and the zones it clears, Cloudburst's hour,
//! Sense the quick's heartbeats.

use super::caster_tests::{kobold_at, run, said, sp};
use super::spell_type_tests::{cell, start, status};
use super::*;
use crate::action::{Action, Event};
use crate::content::{ContentFiles, ZoneKind};
use crate::noise::SoundKind;
use crate::stats::{PerStat, Stats};

/// A lit hall: the player at (2, 3), a kobold (1, 1) in a room walled off to the north (cells 1 to
/// 3, y 1), a meadow at x 15 to 18, y 3 to 5, and a shut door at the east end of the player's row.
const HALL: &str = "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n\
    ########################\n#K..#..................#\n#####..................#\n\
    #.@............gggg....D\n#..............gggg....#\n#..............gggg....#\n\
    ########################\n'''\n\
    [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
    \"g\" = { terrain = \"grass\" }\n\
    \"D\" = { terrain = \"door_closed\" }\n\
    \"K\" = { terrain = \"floor\", creature = \"kobold\" }\n";

/// A walled-in cell, every cell within reach of a blink solid: the map is all wall but the player's.
const CLOSET: &str = "name = \"Closet\"\nambient = [255, 255, 255, 0]\nmap = '''\n\
    #############\n#############\n#############\n#############\n#############\n#############\n\
    ######@######\n#############\n#############\n#############\n#############\n#############\n\
    ############K\n#############\n'''\n\
    [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
    \"K\" = { terrain = \"floor\", creature = \"kobold\" }\n";

fn world(seed: u64, level: &str) -> World {
    let mut files = ContentFiles::builtin();
    files.insert("levels/hall.toml", level);
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\n",
    );
    let mut w = start(Arc::new(Content::load(&files).expect("content")), seed);
    for spell in [
        "shatter",
        "thunderclap",
        "leech",
        "bramble",
        "gale",
        "blink",
        "sense_the_quick",
        "benison",
        "rebuke",
    ] {
        w.player_mut().spells.insert(sp(spell));
    }
    w.player_mut().mana_base = 400;
    w.player_mut().mana = 400;
    w.player_mut().weapon_skills = Default::default();
    w
}

fn hall(seed: u64) -> World {
    world(seed, HALL)
}

fn see(w: &mut World) {
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
}

/// A sturdy creature of `creature` at `at` that will take no turn while the test looks.
fn stander(w: &mut World, at: Pos, creature: &str) -> ActorId {
    let here = w.player_pos();
    let id = kobold_at(w, (at.x - here.x, at.y - here.y), 1000);
    let creature = w.content().creature_id(creature).expect("a creature");
    let actor = w.state.actors.get_mut(&id).expect("there");
    actor.creature = creature;
    actor.energy = -1_000_000;
    actor.hp = 1000;
    id
}

fn hp(w: &World, id: ActorId) -> i32 {
    w.state.actors[&id].hp
}

fn pos(w: &World, id: ActorId) -> Pos {
    w.state.actors[&id].pos
}

/// Everyone a cast's events say it struck at.
fn struck(events: &[Event]) -> Vec<ActorId> {
    (events.iter())
        .filter_map(|e| match e {
            Event::Attacked { target, .. } => Some(*target),
            _ => None,
        })
        .collect()
}

/// The hit points a Leech took and gave, on some seed where it hit, from `before` hit points.
fn leeched(before: i32) -> (i32, i32) {
    for seed in 0..40 {
        let mut w = hall(seed);
        let far = stander(&mut w, cell(6, 3), "kobold");
        w.player_mut().hp = before;
        see(&mut w);
        w.apply(&Action::Cast("leech".into(), Some(cell(6, 3))));
        let taken = 1000 - hp(&w, far);
        if taken > 0 {
            return (taken, w.state.player().hp - before);
        }
    }
    panic!("no Leech hit in forty seeds");
}

#[test]
fn a_leech_gives_back_half_of_what_it_took_and_never_past_the_maximum() {
    let (taken, back) = leeched(1);
    assert_eq!(back, taken / 2);
    let w = hall(0);
    let most = w.state.player().max_hp(w.content());
    let (_, back) = leeched(most - 1);
    assert_eq!(back, 1, "held at the maximum");
    let (_, back) = leeched(most);
    assert_eq!(back, 0);
}

#[test]
fn a_leech_on_the_undead_gives_nothing() {
    for seed in 0..12 {
        let mut w = hall(seed);
        let dead = stander(&mut w, cell(6, 3), "zombie");
        w.player_mut().hp = 1;
        see(&mut w);
        w.apply(&Action::Cast("leech".into(), Some(cell(6, 3))));
        assert_eq!(hp(&w, dead), 1000, "immune");
        assert_eq!(w.state.player().hp, 1);
    }
}

#[test]
fn a_leech_says_its_share_in_the_roll_words() {
    let w = hall(0);
    let def = w.content().spell(sp("leech"));
    let roll = w
        .state
        .player()
        .spell_roll_words(w.content(), def, super::spell_rung::CastBy::Own);
    let text = roll.expect("a roll").text;
    assert!(text.contains("necrotic"), "{text}");
    assert!(text.ends_with("half of it yours"), "{text}");
}

#[test]
fn a_blink_lands_on_a_free_cell_within_reach_by_the_draw() {
    let mut landed = std::collections::BTreeSet::new();
    for seed in 0..30 {
        let mut w = hall(seed);
        let here = w.player_pos();
        // A crowd about the player, the cells it stands on never drawn.
        let crowd: Vec<ActorId> = (3..=6)
            .map(|x| stander(&mut w, cell(x, 4), "kobold"))
            .collect();
        see(&mut w);
        let content = w.content().clone();
        let mut same = World::from_state(content, w.state().clone()).expect("restores");
        w.apply(&Action::Cast("blink".into(), None));
        same.apply(&Action::Cast("blink".into(), None));
        let to = w.player_pos();
        assert_eq!(to, same.player_pos(), "the same draw for the same fate");
        assert_ne!(to, here);
        assert!(to.dist2(here) <= crate::fov::radius2(5), "{to:?}");
        assert!(w.state.current().is_walkable(w.content(), to));
        for id in crowd {
            assert_ne!(pos(&w, id), to, "never onto a creature");
        }
        landed.insert(to);
    }
    assert!(landed.len() > 3, "{landed:?}");
}

#[test]
fn a_blink_out_of_being_held_webbed_or_pinned_ends_each() {
    for name in ["held", "webbed", "pinned"] {
        let mut w = hall(3);
        let player = w.state.player;
        let id = status(&w, name);
        let mut events = Vec::new();
        w.afflict(player, id, 2, None, &mut events);
        assert!(w.state.player().effects.get(id).is_some(), "{name}");
        let (here, mana) = (w.player_pos(), w.state.player().mana);
        see(&mut w);
        let out = w.apply(&Action::Cast("blink".into(), None));
        assert_ne!(w.player_pos(), here, "{name} blocks a step, not a cast");
        assert!(w.state.player().mana < mana);
        assert!(w.state.player().effects.get(id).is_none(), "{name}");
        assert!(
            (out.events.iter())
                .any(|e| matches!(e, Event::Status { status, began: false, .. } if *status == id)),
            "{name}'s end is told"
        );
    }
}

#[test]
fn a_blink_with_nowhere_to_go_is_refused_for_nothing() {
    let mut w = world(1, CLOSET);
    let (here, mana) = (w.player_pos(), w.state.player().mana);
    see(&mut w);
    w.apply(&Action::Cast("blink".into(), None));
    assert_eq!(w.player_pos(), here);
    assert_eq!(w.state.player().mana, mana);
    assert!(said(&w).iter().any(|m| m == "There is nowhere to go."));
}

#[test]
fn a_thunderclap_needs_no_aim_and_strikes_and_stuns_all_in_it_the_caster_too() {
    let mut w = hall(2);
    let def = w.content().spell(sp("thunderclap")).clone();
    assert!(!def.aimed());
    assert_eq!(def.base_range(), None);
    assert_eq!(
        crate::ability::Ability::Spell(sp("thunderclap")).aim(w.content()),
        crate::ability::Aim::Nothing
    );
    let player = w.state.player;
    let east = stander(&mut w, cell(3, 3), "kobold");
    let south = stander(&mut w, cell(2, 4), "kobold");
    let far = stander(&mut w, cell(5, 3), "kobold");
    let before = w.state.player().hp;
    see(&mut w);
    let stunned = status(&w, "stunned");
    let out = w.apply(&Action::Cast("thunderclap".into(), None));
    let hit = struck(&out.events);
    for id in [player, east, south] {
        assert!(hit.contains(&id), "{id:?} in it");
    }
    assert!(!hit.contains(&far));
    assert!(w.state.player().hp < before, "the caster takes it");
    assert_eq!(w.player_pos(), cell(2, 3), "no away from the centre");
    assert!(
        (out.events.iter()).any(|e| matches!(e, Event::Status { actor, status, began: true } if *actor == player && *status == stunned)),
        "the caster reels"
    );
    for id in [east, south] {
        assert!(w.state.actors[&id].effects.get(stunned).is_some());
    }
    assert!(
        (w.state.actors[&far].heard.iter()).any(|h| h.kind == SoundKind::Thunder),
        "heard outside it"
    );
}

#[test]
fn a_benison_heals_the_caster_and_allies_in_it_and_no_one_else() {
    let mut w = hall(4);
    run(&mut w, "ally:wolf");
    let player = w.state.player;
    let ally = (w.state.actors.values())
        .find(|a| a.ally.as_ref().is_some_and(|x| x.master == player))
        .map(|a| a.id)
        .expect("the wolf");
    let foe = stander(&mut w, cell(3, 4), "kobold");
    w.state.actors.get_mut(&foe).expect("there").hp = 1;
    let most = w.state.actors[&ally].max_hp(w.content());
    let wolf = w.state.actors.get_mut(&ally).expect("the wolf");
    wolf.hp = 1;
    wolf.pos = cell(3, 3);
    wolf.energy = -1_000_000;
    w.player_mut().hp = 1;
    see(&mut w);
    let ticks = w.state.time;
    w.apply(&Action::Cast("benison".into(), None));
    let back = w.state.player().hp - 1;
    assert!(back >= 2, "2d6 back");
    assert_eq!(hp(&w, ally), (1 + back).min(most), "one roll for them all");
    assert_eq!(hp(&w, foe), 1, "nothing for a foe");
    assert!(w.state.time > ticks);
}

#[test]
fn a_benison_is_refused_when_no_one_is_hurt_and_an_ally_out_of_reach_gets_nothing() {
    let mut w = hall(4);
    run(&mut w, "ally:wolf");
    let player = w.state.player;
    let ally = (w.state.actors.values())
        .find(|a| a.ally.as_ref().is_some_and(|x| x.master == player))
        .map(|a| a.id)
        .expect("the wolf");
    let most = w.state.player().max_hp(w.content());
    w.player_mut().hp = most;
    let wolf = w.state.actors.get_mut(&ally).expect("the wolf");
    wolf.hp = 1;
    wolf.pos = cell(8, 5);
    let mana = w.state.player().mana;
    see(&mut w);
    w.apply(&Action::Cast("benison".into(), None));
    assert_eq!(w.state.player().mana, mana, "no one in reach is hurt");
    assert!(said(&w).iter().any(|m| m == "No one here is hurt."));
    assert_eq!(hp(&w, ally), 1);
}

#[test]
fn a_rebuke_burns_an_undead_ally_and_foe_and_the_living_not_at_all() {
    let mut w = hall(5);
    let foe = stander(&mut w, cell(9, 3), "zombie");
    run(&mut w, "ally:zombie");
    let player = w.state.player;
    let friend = (w.state.actors.values())
        .find(|a| a.ally.as_ref().is_some_and(|x| x.master == player))
        .map(|a| a.id)
        .expect("the zombie");
    let ally = w.state.actors.get_mut(&friend).expect("there");
    ally.pos = cell(10, 3);
    ally.hp = 1000;
    ally.energy = -1_000_000;
    let living = stander(&mut w, cell(9, 4), "kobold");
    see(&mut w);
    let out = w.apply(&Action::Cast("rebuke".into(), Some(cell(9, 3))));
    let hit = struck(&out.events);
    assert!(hit.contains(&foe) && hit.contains(&friend), "{hit:?}");
    assert!(!hit.contains(&living), "not even an event");
    assert_eq!(hp(&w, living), 1000);
    assert!(hp(&w, foe) < 1000 && hp(&w, friend) < 1000);
    let frightened = status(&w, "frightened");
    assert!(w.state.actors[&foe].effects.get(frightened).is_some());
    assert!(w.state.actors[&living].effects.get(frightened).is_none());
    assert!(
        w.state.current().fires.contains_key(&cell(9, 3)),
        "fire's rule: a fire on its centre"
    );
}

#[test]
fn a_bramble_takes_those_on_living_ground_and_spares_stone() {
    let mut w = hall(6);
    w.player_mut().pos = cell(10, 4);
    let on_grass = stander(&mut w, cell(15, 4), "kobold");
    let on_stone = stander(&mut w, cell(14, 4), "kobold");
    see(&mut w);
    let mana = w.state.player().mana;
    w.apply(&Action::Cast("bramble".into(), Some(cell(14, 4))));
    assert!(w.state.player().mana < mana, "it goes off");
    let webbed = status(&w, "webbed");
    assert!(w.state.actors[&on_grass].effects.get(webbed).is_some());
    assert!(w.state.actors[&on_stone].effects.get(webbed).is_none());
}

#[test]
fn a_gale_drives_the_nearest_first_so_no_one_is_pushed_into_another() {
    let mut w = hall(7);
    let near = stander(&mut w, cell(4, 3), "kobold");
    let next = stander(&mut w, cell(5, 3), "kobold");
    let lone = stander(&mut w, cell(7, 3), "kobold");
    let by_door = w.content().light_id("arrow_fire").expect("a light");
    w.lay_fire(cell(6, 3), by_door, 60);
    w.lay_fire(cell(3, 4), by_door, 60);
    see(&mut w);
    w.apply(&Action::Cast("gale".into(), Some(cell(8, 3))));
    assert_eq!(pos(&w, near), cell(4, 3), "the next one in its way");
    assert_eq!(pos(&w, next), cell(6, 3));
    assert_eq!(pos(&w, lone), cell(8, 3));
    for id in [near, next, lone] {
        assert_eq!(hp(&w, id), 1000, "no harm");
    }
    let fires = &w.state.current().fires;
    assert!(!fires.contains_key(&cell(6, 3)), "blown out");
    assert!(fires.contains_key(&cell(3, 4)), "off the line");
}

#[test]
fn a_gale_leaves_the_held_where_they_are_and_a_shut_door_shut() {
    let mut w = hall(7);
    let held = stander(&mut w, cell(4, 3), "kobold");
    let webbed = status(&w, "webbed");
    let mut events = Vec::new();
    w.afflict(held, webbed, 3, None, &mut events);
    let at_door = stander(&mut w, cell(8, 3), "kobold");
    w.state.actors.get_mut(&at_door).expect("there").pos = cell(21, 3);
    w.player_mut().pos = cell(16, 3);
    w.state.actors.get_mut(&held).expect("there").pos = cell(18, 3);
    see(&mut w);
    let door = w.state.current().terrain.get(cell(23, 3)).copied();
    w.apply(&Action::Cast("gale".into(), Some(cell(22, 3))));
    assert_eq!(pos(&w, held), cell(18, 3));
    assert_eq!(pos(&w, at_door), cell(22, 3));
    assert_eq!(w.state.current().terrain.get(cell(23, 3)).copied(), door);
    // At the door it can go no farther.
    see(&mut w);
    w.player_mut().mana = 100;
    w.apply(&Action::Cast("gale".into(), Some(cell(22, 3))));
    assert_eq!(pos(&w, at_door), cell(22, 3));
}

#[test]
fn a_gale_tears_clouds_fire_and_darkness_off_its_line_and_leaves_slick_ground() {
    let mut w = hall(7);
    run(
        &mut w,
        "zone:fouled@5,4,1 zone:dark@3,3,0 zone:fire@8,4,0 zone:fire@8,3,0 zone:slick@7,3,0",
    );
    let kind = |w: &World, x, y| {
        (w.state.current())
            .zone_at(cell(x, y), w.state.time)
            .map(|z| z.kind())
    };
    assert_eq!(kind(&w, 5, 3), Some(ZoneKind::Fouled));
    see(&mut w);
    w.apply(&Action::Cast("gale".into(), Some(cell(8, 3))));
    for x in [3, 4, 5, 6, 8] {
        assert_eq!(kind(&w, x, 3), None, "({x}, 3) is torn clear");
    }
    assert_eq!(kind(&w, 7, 3), Some(ZoneKind::Slick), "slick ground stays");
    assert_eq!(kind(&w, 5, 4), Some(ZoneKind::Fouled), "off the line");
    assert_eq!(kind(&w, 8, 4), Some(ZoneKind::Fire), "off the line");
}

#[test]
fn a_cloudburst_needs_the_open_sky_rains_for_an_hour_and_puts_out_what_burns() {
    let mut w = hall(8);
    w.player_mut().spells.insert(sp("cloudburst"));
    let mana = w.state.player().mana;
    see(&mut w);
    w.apply(&Action::Cast("cloudburst".into(), None));
    assert_eq!(w.state.player().mana, mana, "a hall has no sky");
    assert!(w.state.weather_override.is_none());

    let content = Arc::new(Content::builtin().expect("content"));
    let spec: crate::character::CharacterSpec = "human,female,wizard".parse().expect("a character");
    let spec = spec.with_talents(vec!["keen_ears".into(), "lean".into(), "pious".into()]);
    let mut w = World::start_run(content, "courtyard", Some(9), Some(&spec), None, true)
        .expect("the courtyard");
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    w.player_mut().spells.insert(sp("cloudburst"));
    w.player_mut().mana_base = 400;
    w.player_mut().mana = 400;
    let player = w.state.player;
    let burning = status(&w, "burning");
    let mut events = Vec::new();
    w.afflict(player, burning, 1, None, &mut events);
    let light = w.content().light_id("arrow_fire").expect("a light");
    let here = w.player_pos();
    let beside = crate::geom::Dir::ALL
        .into_iter()
        .map(|d| here.step(d))
        .find(|&p| w.state.current().is_walkable(w.content(), p))
        .expect("somewhere to lay it");
    w.lay_fire(beside, light, 600);
    assert!(w.state.current().fires.contains_key(&beside));
    let began = w.state.time;
    see(&mut w);
    w.apply(&Action::Cast("cloudburst".into(), None));
    let over = w.state.weather_override.as_ref().expect("the rain");
    assert_eq!(over.weather, crate::weather::Weather::Rain);
    assert_eq!(over.since, began);
    assert_eq!(over.until, began + crate::time::Duration::hours(1));
    assert!(!w.state.current().fires.contains_key(&beside));
    assert!(w.state.player().effects.get(burning).is_none());
}

/// The heartbeats the player perceives now.
fn beats(w: &World) -> Vec<Pos> {
    (w.perception().sounds().iter())
        .filter(|s| s.kind == SoundKind::Heartbeat)
        .map(|s| s.pos)
        .collect()
}

#[test]
fn a_heartbeat_comes_at_a_rough_place_and_never_for_the_undead() {
    let mut w = hall(10);
    let walled = (w.state.actors.values())
        .find(|a| a.pos == cell(1, 1))
        .map(|a| a.id)
        .expect("the walled-up kobold");
    w.state.actors.get_mut(&walled).expect("there").energy = -1_000_000;
    assert_eq!(beats(&w), Vec::<Pos>::new());
    see(&mut w);
    w.apply(&Action::Cast("sense_the_quick".into(), None));
    let mut places = std::collections::BTreeSet::new();
    for _ in 0..12 {
        let felt = beats(&w);
        assert_eq!(felt.len(), 1, "{felt:?}");
        assert!(felt[0].chebyshev(cell(1, 1)) <= 2, "{felt:?}");
        assert!(
            felt[0].y == 1,
            "only in its room, where the player does not see"
        );
        places.insert(felt[0]);
        w.player_mut().effects = Default::default();
        let quick = status(&w, "quick_sensed");
        let (mut events, player) = (Vec::new(), w.state.player);
        w.afflict(player, quick, 1, None, &mut events);
        run(&mut w, "wait");
    }
    assert!(
        places.len() > 1,
        "not the exact place every time: {places:?}"
    );
    assert!(w.state.player().heard.iter().all(|h| h.loudness > 0));
    assert!((said(&w).iter()).any(|m| m.contains("a heartbeat")));
    // The same creature dead on its feet gives nothing.
    let undead = w.content().creature_id("zombie").expect("zombies");
    w.state.actors.get_mut(&walled).expect("there").creature = undead;
    run(&mut w, "wait");
    assert_eq!(beats(&w), Vec::<Pos>::new());
}

#[test]
fn a_heartbeat_is_not_felt_past_its_reach_or_where_the_player_sees() {
    let mut w = hall(10);
    let walled = (w.state.actors.values())
        .find(|a| a.pos == cell(1, 1))
        .map(|a| a.id)
        .expect("the walled-up kobold");
    w.state.actors.get_mut(&walled).expect("there").energy = -1_000_000;
    w.player_mut().pos = cell(14, 3);
    stander(&mut w, cell(16, 3), "kobold");
    see(&mut w);
    w.apply(&Action::Cast("sense_the_quick".into(), None));
    assert_eq!(beats(&w), Vec::<Pos>::new());
}

#[test]
fn a_shatter_breaks_a_wall_and_a_door_where_it_lands() {
    let mut w = hall(11);
    w.player_mut().pos = cell(20, 3);
    see(&mut w);
    let out = w.apply(&Action::Cast("shatter".into(), Some(cell(22, 3))));
    assert!(
        (out.events.iter()).any(|e| matches!(e, Event::DoorBroken { pos } if *pos == cell(23, 3))),
        "force's rule breaks the shut door in it"
    );
}
