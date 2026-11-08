//! Zones (`world/zone.rs`): laid, saved, hashed, and dropped; what each kind does and when; who
//! it spares (no one); what the player knows of one.

use super::caster_tests::{kobold_at, sp};
use super::spell_type_tests::{cast, cell, files, start, status};
use super::*;
use crate::action::{Action, parse_actions};
use crate::content::ZoneKind;
use crate::light::Light;
use crate::stats::{PerStat, Stat, Stats};

fn hall(seed: u64) -> World {
    let mut w = start(Arc::new(Content::load(&files()).expect("content")), seed);
    for spell in ["balefire", "reek", "mirk", "slick", "frost"] {
        w.player_mut().spells.insert(sp(spell));
    }
    w
}

/// The player clear of the pond, with dry floor on every side.
fn on_dry_floor(w: &mut World) -> Pos {
    w.player_mut().pos = cell(9, 3);
    cell(9, 3)
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn zone_kind(w: &World, p: Pos) -> Option<ZoneKind> {
    let level = w.state.current();
    level.zone_at(p, w.state.time).map(|z| z.kind())
}

/// A kobold `(dx, dy)` from the player, set to take it: it stands still, no hostile near.
fn dummy(w: &mut World, offset: (i32, i32)) -> ActorId {
    let id = kobold_at(w, offset, 10_000);
    let actor = w.state.actors.get_mut(&id).expect("there");
    actor.stats = Stats::new(PerStat::splat(10), PerStat::splat(40));
    id
}

fn severity(w: &World, id: ActorId, name: &str) -> u16 {
    let status = status(w, name);
    let actor = &w.state.actors[&id];
    actor.effects.get(status).map_or(0, |e| e.severity)
}

#[test]
fn a_zone_is_laid_saved_hashed_and_dropped_at_its_end() {
    let mut w = hall(1);
    let before = serde_json::to_string(&w.state).expect("saves");
    assert!(!before.contains("zones"), "left out while empty");
    let hash = w.state_hash();
    let (turn, time) = (w.turn(), w.time());
    run(&mut w, "zone:fouled@12,3,1");
    assert_eq!((w.turn(), w.time()), (turn, time), "no time");
    assert_ne!(w.state_hash(), hash, "hashed");
    assert_eq!(zone_kind(&w, cell(12, 3)), Some(ZoneKind::Fouled));
    assert_eq!(zone_kind(&w, cell(12, 2)), Some(ZoneKind::Fouled));
    assert_eq!(zone_kind(&w, cell(15, 3)), None);

    // Saved, it comes back the same.
    let saved = serde_json::to_string(&w.state).expect("saves");
    assert!(saved.contains("zones"));
    let state: WorldState = serde_json::from_str(&saved).expect("loads");
    let again = World::from_state(w.content.clone(), state).expect("restores");
    assert_eq!(again.state_hash(), w.state_hash());
    assert_eq!(again.zones(), w.zones());

    // In the truth, and gone by its end: the dev zone lies ten turns, a minute.
    let truth = crate::dump::layer_rows(&w, crate::dump::Layer::Truth);
    assert!(truth.iter().any(|r| r.starts_with("zone fouled by #0")));
    let end = w.zones()[0].until();
    while w.time() < end {
        assert_eq!(w.zones().len(), 1);
        run(&mut w, "wait");
    }
    assert!(w.zones().is_empty(), "dropped at the end of its turn");
}

#[test]
fn a_new_zone_replaces_an_old_on_the_shared_cells_and_one_with_none_left_is_gone() {
    let mut w = hall(1);
    run(&mut w, "zone:fouled@12,3,1 zone:slick@13,3,0");
    assert_eq!(zone_kind(&w, cell(13, 3)), Some(ZoneKind::Slick));
    assert_eq!(zone_kind(&w, cell(12, 3)), Some(ZoneKind::Fouled));
    run(&mut w, "zone:dark@12,3,3");
    assert_eq!(w.zones().len(), 1, "both swallowed");
    assert_eq!(zone_kind(&w, cell(13, 3)), Some(ZoneKind::Dark));
}

#[test]
fn a_creature_burns_at_its_turns_start_awake_or_not_and_its_kill_is_the_casters() {
    let mut w = hall(1);
    let id = dummy(&mut w, (6, 0));
    let at = w.state.actors[&id].pos;
    run(&mut w, &format!("zone:fire@{},{},0", at.x, at.y));
    let hp = w.state.actors[&id].hp;
    run(&mut w, "wait*2");
    let lost = hp - w.state.actors[&id].hp;
    assert!(lost >= 4, "burnt at least once: {lost}");

    // Its death is the player's kill, experience and all (it walks off: back it goes; a kobold
    // is slower than the player, so not every wait is a turn of its).
    let xp = w.perception().own_experience().xp;
    w.state.actors.get_mut(&id).expect("there").hp = 1;
    for _ in 0..3 {
        if let Some(kobold) = w.state.actors.get_mut(&id) {
            kobold.pos = at;
        }
        run(&mut w, "wait");
    }
    assert!(!w.state.actors.contains_key(&id), "it burnt to death");
    assert!(w.perception().own_experience().xp > xp);
}

#[test]
fn a_step_onto_fire_burns_the_caster_like_anyone_and_a_turn_on_it_again() {
    let mut w = hall(1);
    let here = w.player_pos();
    let east = cell(here.x + 1, here.y);
    run(&mut w, &format!("zone:fire@{},{},0", east.x, east.y));
    let hp = w.state.player().hp;
    run(&mut w, "move:E");
    assert_eq!(w.player_pos(), east);
    // The step's burn, then the turn's that starts there.
    assert_eq!(hp - w.state.player().hp, 8);
    let hp = w.state.player().hp;
    run(&mut w, "move:W");
    assert_eq!(w.state.player().hp, hp, "off it, nothing");
}

#[test]
fn the_caster_killed_by_their_own_fire_is_told_so() {
    let mut w = hall(1);
    let here = on_dry_floor(&mut w);
    let burn = cell(here.x + 3, here.y);
    cast(&mut w, "balefire", burn, None);
    assert_eq!(zone_kind(&w, burn), Some(ZoneKind::Fire));
    w.player_mut().hp = 1;
    run(&mut w, "move:E");
    let death = w.state.death.as_ref().expect("dead");
    assert!(matches!(death.cause, Some(Cause::Spell(_))), "{death:?}");
}

#[test]
fn balefire_burns_at_the_cast_and_lies_on_its_line_for_ten_turns_with_a_fires_light() {
    let mut w = hall(1);
    let here = on_dry_floor(&mut w);
    let id = dummy(&mut w, (2, 0));
    w.state.current_mut().ambient = Light::DARK;
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    let bright = |w: &World, p: Pos| w.perception().light(p).map_or(0, |l| l.brightness());
    let before = bright(&w, cell(here.x + 3, here.y));
    let began = w.time();
    let struck = cast(&mut w, "balefire", cell(here.x + 2, here.y), Some(id));
    assert!(!struck.is_empty(), "struck at the cast");
    assert_eq!(w.zones().len(), 1);
    let zone = w.zones()[0].clone();
    // Willpower 14: length 6, from the cell before the caster on.
    assert_eq!(zone.kind(), ZoneKind::Fire);
    let line: Vec<Pos> = (1..=6).map(|dx| cell(here.x + dx, here.y)).collect();
    assert_eq!(zone.cells().collect::<Vec<_>>(), line);
    assert!(zone.harm() >= 3, "3d6 at intelligence 14: {}", zone.harm());
    assert_eq!(zone.until().since(began).as_seconds(), 60);
    // Its light, in a hall with none of its own.
    let lit = bright(&w, line[2]);
    assert!(lit > before, "its fire's light");
    // Over: its light gone (with nothing left of the burnt kobold's own burning).
    w.state.actors.remove(&id);
    run(&mut w, "wait*12");
    assert!(w.zones().is_empty());
    assert!(bright(&w, line[2]) < lit, "its light gone");
}

#[test]
fn a_fire_zone_is_put_out_by_frost_cell_by_cell() {
    let mut w = hall(1);
    run(&mut w, "zone:fire@12,3,2");
    let all = w.zones()[0].cells().count();
    assert_eq!(cast(&mut w, "frost", cell(12, 3), None).len(), 0);
    let left = w.zones().iter().map(|z| z.cells().count()).sum::<usize>();
    assert!(left < all, "{left} of {all}");
    assert_eq!(zone_kind(&w, cell(12, 3)), None);
}

#[test]
fn a_fire_lies_on_dry_ground_but_a_cloud_hangs_over_the_pond() {
    let mut w = hall(1);
    run(&mut w, "zone:fire@6,3,1");
    assert!(w.zones().is_empty(), "all water");
    run(&mut w, "zone:fouled@6,3,1");
    assert_eq!(zone_kind(&w, cell(6, 3)), Some(ZoneKind::Fouled));
}

#[test]
fn the_reek_poisons_per_turn_and_the_immune_walk_through() {
    let mut w = hall(1);
    let id = dummy(&mut w, (6, 0));
    let zombie = dummy(&mut w, (7, 0));
    let creature = w.content.creature_id("zombie").expect("a zombie");
    w.state.actors.get_mut(&zombie).expect("there").creature = creature;
    run(&mut w, "zone:fouled@8,3,1");
    for _ in 0..6 {
        // Back they go, as they walk off to fight.
        w.state.actors.get_mut(&id).expect("there").pos = cell(8, 3);
        w.state.actors.get_mut(&zombie).expect("there").pos = cell(9, 3);
        run(&mut w, "wait");
    }
    let poisoned = severity(&w, id, "poisoned");
    assert!(poisoned >= 3, "a degree a turn: {poisoned}");
    assert_eq!(severity(&w, zombie, "poisoned"), 0);
    // It does not block sight: the player sees through it.
    assert!(w.perception().is_visible(cell(8, 3)));
    assert!(w.perception().is_visible(cell(10, 3)));
}

#[test]
fn reek_harms_at_the_cast_and_lies_where_it_went_off() {
    let mut w = hall(1);
    let id = dummy(&mut w, (6, 0));
    let at = w.state.actors[&id].pos;
    assert_eq!(cast(&mut w, "reek", at, Some(id)).len(), 1);
    assert_eq!(zone_kind(&w, at), Some(ZoneKind::Fouled));
    assert_eq!(w.zones()[0].harm(), 0, "the cast's roll is all its harm");
    assert!(severity(&w, id, "poisoned") >= 1);
}

#[test]
fn no_light_reaches_the_mirk_and_a_source_inside_sheds_none() {
    let mut w = hall(1);
    run(&mut w, "zone:fouled@40,40,0");
    let dark_cell = cell(10, 3);
    assert!(w.perception().is_visible(dark_cell), "lit, before");
    run(&mut w, "zone:dark@10,3,1 wait");
    assert!(!w.perception().is_visible(dark_cell), "ambient light gone");
    assert_eq!(w.perception().zone_at(dark_cell), None, "dark by the dark");
    assert!(w.perception().is_visible(cell(13, 3)), "the rest still lit");

    // A fire inside lights nothing outside it; the same fire outside does.
    let mut w = hall(1);
    w.state.current_mut().ambient = Light::DARK;
    let light = w.content.light_id("arrow_fire").expect("a light");
    let until = w.state.time + Duration::hours(1);
    w.state
        .current_mut()
        .fires
        .insert(cell(10, 3), Fire { light, until });
    run(&mut w, "wait");
    assert!(w.perception().is_visible(cell(12, 3)), "the fire lights it");
    run(&mut w, "zone:dark@10,3,0 wait");
    assert!(!w.perception().is_visible(cell(12, 3)), "inside: no light");
}

#[test]
fn eyes_that_need_light_lose_sight_of_who_stands_in_the_mirk_and_darkvision_does_not() {
    let mut w = hall(1);
    let far = dummy(&mut w, (8, 0));
    let near = dummy(&mut w, (4, 0));
    let sees = |w: &mut World, id: ActorId| w.nearest_hostile_in_sight(id).is_some();
    // Kobolds see by light, and within darkvision's 5 cells without it.
    w.state.actors.get_mut(&near).expect("there").pos = cell(6, 1);
    assert!(sees(&mut w, far), "a lit hall");
    let me = w.player_pos();
    run(&mut w, &format!("zone:dark@{},{},0", me.x, me.y));
    assert!(!sees(&mut w, far), "blind to one in the dark");
    assert!(sees(&mut w, near), "within its darkvision");
}

/// What a step onto slick ground came to, Staggered or not, at a Dexterity score.
fn slips(seed: u64, dexterity: u8) -> bool {
    let mut w = hall(seed);
    let mut scores = PerStat::splat(14);
    scores[Stat::Dexterity] = dexterity;
    w.player_mut().stats = Stats::new(scores, PerStat::splat(40));
    let here = w.player_pos();
    run(&mut w, &format!("zone:slick@{},{},0", here.x + 1, here.y));
    run(&mut w, "move:E");
    assert_eq!(w.player_pos(), cell(here.x + 1, here.y), "it still moved");
    let player = w.state.player;
    severity(&w, player, "staggered") > 0
}

#[test]
fn slicks_check_falls_by_the_seed_and_by_dexterity() {
    let count = |dex: u8| (0..200).filter(|&seed| slips(seed, dex)).count();
    let (clumsy, nimble) = (count(3), count(30));
    assert!(clumsy > 100, "{clumsy}");
    assert!(nimble < 20, "{nimble}");
    assert_eq!(slips(11, 12), slips(11, 12), "by the seed");
}

#[test]
fn a_step_off_slick_ground_may_trip_too_but_walking_clear_draws_nothing() {
    let mut w = hall(5);
    let plain = w.state.rng.clone();
    run(&mut w, "zone:slick@15,3,0");
    assert_eq!(w.state.rng, plain, "laying draws nothing");
    let mut seen = false;
    for seed in 0..200 {
        let mut w = hall(seed);
        let here = w.player_pos();
        run(&mut w, &format!("zone:slick@{},{},0", here.x, here.y));
        run(&mut w, "move:E");
        let player = w.state.player;
        seen |= severity(&w, player, "staggered") > 0;
    }
    assert!(seen, "someone slipped stepping off");
}

#[test]
fn hexes_of_mirk_and_slick_lay_their_zones() {
    let mut w = hall(3);
    let at = cell(on_dry_floor(&mut w).x + 4, 3);
    cast(&mut w, "mirk", at, None);
    assert_eq!(w.zones().len(), 1);
    let dark = w.zones()[0].cells().count();
    assert!(dark >= 13, "radius 2: {dark}");
    assert_eq!(w.zones()[0].kind(), ZoneKind::Dark);
    let mut w = hall(3);
    on_dry_floor(&mut w);
    cast(&mut w, "slick", at, None);
    assert_eq!(w.zones()[0].kind(), ZoneKind::Slick);
    assert!(w.zones()[0].cells().count() >= 9, "radius 1");
}

#[test]
fn a_scroll_lays_it_too_and_the_overworld_takes_none() {
    let mut w = hall(1);
    let scroll = w.content.object_id("scroll_of_slick").expect("a scroll");
    w.player_mut()
        .inventory
        .push(crate::object::Stack::new(scroll, 1));
    let item = w.state.player().inventory.len() - 1;
    let at = cell(on_dry_floor(&mut w).x + 3, 3);
    let action: Action = format!("read:{item}:{},{}", at.x, at.y)
        .parse()
        .expect("parses");
    let out = w.apply(&action);
    assert!(out.took_turn);
    assert_eq!(zone_kind(&w, at), Some(ZoneKind::Slick));

    let content = Arc::new(super::caster_tests::quiet());
    let mut world = World::new(content, "world", None).expect("the overworld");
    run(&mut world, "zone:fire@45,23,1");
    assert!(world.zones().is_empty());
}

#[test]
fn hidden_cells_show_nothing_and_seen_ones_are_remembered_until_seen_clear() {
    let mut w = hall(1);
    // The walled closet: in no one's sight.
    run(&mut w, "zone:fouled@20,1,0");
    assert_eq!(w.zones().len(), 1);
    assert_eq!(w.perception().zone_at(cell(20, 1)), None);

    run(&mut w, "zone:fouled@12,3,0");
    assert_eq!(w.perception().zone_at(cell(12, 3)), Some(ZoneKind::Fouled));

    // Out of sight it is remembered, past its end too, as terrain is.
    w.player_mut().pos = cell(20, 1);
    w.state.time += Duration::minutes(10);
    run(&mut w, "wait");
    assert!(
        w.zones()
            .iter()
            .all(|z| z.cells().all(|p| p != cell(12, 3)))
    );
    assert_eq!(w.perception().zone_at(cell(12, 3)), Some(ZoneKind::Fouled));
    // Seen again, clear: forgotten.
    w.player_mut().pos = cell(2, 3);
    run(&mut w, "wait");
    assert_eq!(w.perception().zone_at(cell(12, 3)), None);
}

#[test]
fn a_spells_zone_is_checked_as_it_loads() {
    let load = |effect: &str, rest: &str| {
        let mut files = super::caster_tests::files();
        let text = format!(
            "[[spell]]\nid = \"test_zone\"\nname = \"Test\"\ndescription = \"T.\"\nlevel = 5\n\
             school = \"evocation\"\n{rest}\neffect = {{ zone = {effect} }}\n"
        );
        files.insert("data/spells/test_zone.toml", &text);
        Content::load(&files)
            .map(|_| ())
            .expect_err("refused")
            .message
    };
    let ray = "kind = \"hex\"\nshape = \"ray\"\nresolution = \"auto\"\nrange = 5";
    let buff = "kind = \"buff\"\nshape = \"self\"\nresolution = \"auto\"";
    let dark = "{ kind = \"dark\", turns = 5 }";
    assert!(load(dark, ray).contains("`burst`'s or a `line`'s"));
    assert!(load(dark, buff).contains("has no `zone`"));
}

#[test]
fn the_dev_action_round_trips_and_refuses_what_is_no_zone() {
    let action: Action = "zone:fire@12,5,3".parse().expect("parses");
    assert_eq!(action.to_string(), "zone:fire@12,5,3");
    for bad in [
        "zone",
        "zone:fire",
        "zone:mud@1,2,3",
        "zone:fire@1,2",
        "zone:fire@1,2,99",
    ] {
        assert!(bad.parse::<Action>().is_err(), "{bad}");
    }
}

#[test]
fn a_run_with_zones_replays_the_same() {
    let content = Arc::new(Content::load(&files()).expect("content"));
    let actions = "zone:slick@3,3,0 zone:fire@5,1,1 move:E wait*3 move:NE move:E wait*12";
    let play = || {
        let mut w = start(content.clone(), 9);
        run(&mut w, actions);
        w.state_hash()
    };
    assert_eq!(play(), play());
}
