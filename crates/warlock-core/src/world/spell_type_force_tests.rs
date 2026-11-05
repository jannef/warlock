//! The spell types' second table (`world/spell_type.rs`, `[types]` in `data/spells.toml`): force's
//! shove, its blast on doors and thin ice; poison's degree; necrotic's bite and the undead's
//! immunity; acid's Corroded.

use super::caster_tests::{kobold_at, sp};
use super::spell::Caster;
use super::spell_type_tests::{cast, cell, files as hall_files, put_back, start, status};
use super::*;
use crate::action::Event;
use crate::combat::Strike;
use crate::content::{Burst, ContentFiles, DamageTypeId};
use crate::damage::Damage;
use crate::dump::{self, Layer};

/// A hall, the player at (6, 5). In its north wall, y 3, four doors (plain at x 10, locked at
/// x 12, trapped at x 14, far off at x 20) and a stout door with nothing to break to at x 11, and
/// a door behind that wall at (16, 2). In it a coffer (14, 5), a kist (15, 5), and a pond (15 to
/// 17, 6).
const HALL: &str = "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n\
    ######################\n#K...................#\n################D#####\n\
    ##########DXL#T#####D#\n#....................#\n#.....@.......CH.....#\n\
    #..............~~~...#\n#....................#\n######################\n'''\n\
    [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
    \"~\" = { terrain = \"water\" }\n\
    \"K\" = { terrain = \"floor\", creature = \"kobold\" }\n\
    \"C\" = { terrain = \"chest\" }\n\
    \"H\" = { terrain = \"chest\", chest = \"kist\", locked = true }\n\
    \"D\" = { terrain = \"door_closed\" }\n\
    \"X\" = { terrain = \"stout_door\" }\n\
    \"L\" = { terrain = \"door_closed\", locked = true }\n\
    \"T\" = { terrain = \"door_closed\", trap = \"arrow\" }\n";

/// A door no kick breaks, or blast.
const STOUT: &str = "\n[[terrain]]\nid = \"stout_door\"\nname = \"stout door\"\nglyph = \"+\"\n\
    fg = \"#c08040\"\nwalkable = false\ntransparent = false\nopens_to = \"door_open\"\n";

/// Frost made a force burst with a blast, as the design's Shatter is.
const CRACK: &str = "[[spell]]\nid = \"frost\"\nname = \"Crack\"\ndescription = \"x\"\nlevel = 6\n\
    school = \"evocation\"\nkind = \"damage\"\nshape = \"burst\"\n\
    resolution = \"area\"\ndamage_type = \"force\"\nrange = 20\nradius = 3\n\
    damage = \"1d2\"\n[spell.cast]\nblast = 1000\n";

/// Ray of frost made a force ray that hits for 120.
const CLOUT: &str = "[[spell]]\nid = \"ray_of_frost\"\nname = \"Clout\"\ndescription = \"x\"\n\
    level = 1\nschool = \"evocation\"\nkind = \"damage\"\nshape = \"ray\"\n\
    resolution = \"auto\"\ndamage_type = \"force\"\nrange = 10\ndamage = \"+120\"\n\
    [spell.cast]\nnoun = \"clout\"\nverb = \"strikes\"\n";

fn files() -> ContentFiles {
    let mut files = hall_files();
    files.insert("levels/hall.toml", HALL);
    let terrain = include_str!("../../../../assets/data/terrain.toml");
    let terrain = format!("{terrain}{STOUT}");
    files.insert("data/terrain.toml", &terrain);
    files.insert("data/spells/frost.toml", CRACK);
    files.insert("data/spells/ray_of_frost.toml", CLOUT);
    files
}

fn hall() -> World {
    start(Arc::new(Content::load(&files()).expect("content")), 1)
}

fn kind(w: &World, id: &str) -> Option<DamageTypeId> {
    w.content().damage_type_id(id)
}

fn terrain(w: &World, p: Pos) -> String {
    let t = w.state.current().terrain[p];
    w.content().terrain(t).id.clone()
}

fn pos(w: &World, id: ActorId) -> Pos {
    w.state.actors[&id].pos
}

/// The player's spell of `id`'s type lands on `target` as `strike`, for `amount`, from `from`.
fn strike(w: &mut World, target: ActorId, from: Pos, id: &str, amount: i32, strike: Strike) {
    let player = w.state.player;
    let damage = Damage::new(amount, kind(w, id));
    w.spell_strikes(
        player,
        target,
        (from, true),
        (damage, strike),
        &mut Vec::new(),
    );
}

const HIT: Strike = Strike::Hit { critical: false };

/// A kobold in the open, its maximum hit points, and where it stands.
fn target(w: &mut World) -> (ActorId, i32, Pos) {
    let id = kobold_at(w, (-3, 2), 30);
    let most = w.state.actors[&id].max_hp(w.content());
    (id, most, pos(w, id))
}

#[test]
fn a_big_hit_shoves_and_a_small_one_does_not() {
    let mut w = hall();
    let (id, most, at) = target(&mut w);
    let from = cell(1, at.y);
    let big = (most + 2) / 3;
    strike(&mut w, id, from, "force", big - 1, HIT);
    assert_eq!(pos(&w, id), at, "just under a third of its hit points");
    put_back(&mut w, id, at);
    strike(&mut w, id, from, "force", big, HIT);
    assert_eq!(
        pos(&w, id),
        at.step(Dir::E),
        "a third, away from the origin"
    );
    put_back(&mut w, id, at);
    let critical = Strike::Hit { critical: true };
    strike(&mut w, id, from, "force", 1, critical);
    assert_eq!(pos(&w, id), at.step(Dir::E), "a critical, however small");
    put_back(&mut w, id, at);
    strike(&mut w, id, from, "cold", big, HIT);
    assert_eq!(pos(&w, id), at, "another type's blow");
    let miss = Strike::Missed { by: None };
    put_back(&mut w, id, at);
    strike(&mut w, id, from, "force", big, miss);
    assert_eq!(pos(&w, id), at, "a miss");
}

#[test]
fn a_held_or_webbed_creature_stays_and_a_wall_stops_the_shove() {
    let mut w = hall();
    let (id, most, at) = target(&mut w);
    let from = cell(1, at.y);
    for name in ["held", "webbed"] {
        put_back(&mut w, id, at);
        let bound = status(&w, name);
        w.afflict(id, bound, 1, None, &mut Vec::new());
        strike(&mut w, id, from, "force", most, HIT);
        assert_eq!(pos(&w, id), at, "{name}");
    }
    put_back(&mut w, id, cell(1, 4));
    strike(&mut w, id, cell(5, 4), "force", most, HIT);
    assert_eq!(pos(&w, id), cell(1, 4), "the wall west of it");
}

/// A force burst of `amount`, radius 2, by the player.
fn burst(w: &World, amount: i32, blast: u16) -> Burst {
    Burst {
        spell: String::new(),
        radius: 2,
        damage: crate::dice::Roll::fixed(amount),
        reroll: 0,
        kind: kind(w, "force"),
        blast,
        ice: None,
    }
}

#[test]
fn a_burst_shoves_outward_and_its_caster_at_the_centre_stays() {
    let mut w = hall();
    let player = w.state.player;
    let here = w.player_pos();
    w.player_mut().hp_base = 1000;
    w.player_mut().hp = 1000;
    let east = kobold_at(&mut w, (5, 1), 300);
    let centre = kobold_at(&mut w, (4, 1), 300);
    let middle = pos(&w, centre);
    let big = w.state.actors[&east].max_hp(w.content()) / 2;
    let aimed = burst(&w, big, 0);
    let e = pos(&w, east);
    w.cast_burst(
        Caster::Actor(player),
        (here, middle),
        &aimed,
        &mut Vec::new(),
    );
    assert_eq!(pos(&w, east), e.step(Dir::E), "outward from the centre");
    let away = Dir::toward(here, middle).expect("a way");
    assert_eq!(
        pos(&w, centre),
        middle.step(away),
        "at the centre: from the caster"
    );

    let mut w = hall();
    let player = w.state.player;
    let here = w.player_pos();
    w.player_mut().hp_base = 90;
    w.player_mut().hp = 90;
    let aimed = burst(&w, 60, 0);
    w.cast_burst(Caster::Actor(player), (here, here), &aimed, &mut Vec::new());
    assert!(w.state.player().hp < 90, "it spares no one");
    assert_eq!(w.player_pos(), here, "no away from their own cell");
}

#[test]
fn the_player_reads_the_blow_then_the_shove() {
    let mut w = hall();
    let id = kobold_at(&mut w, (5, 0), 300);
    let at = pos(&w, id);
    cast(&mut w, "ray_of_frost", at, Some(id));
    let lines: Vec<&str> = w.messages().iter().map(|m| m.text.as_str()).collect();
    let driven = (lines.iter()).position(|l| *l == "The kobold is driven back.");
    let hit = (lines.iter()).position(|l| l.contains("clout") && l.contains("kobold"));
    assert!(hit.is_some() && hit < driven, "{lines:?}");
}

const DOORS: [(i32, i32); 3] = [(10, 3), (12, 3), (14, 3)];

#[test]
fn a_force_blast_breaks_shut_doors_locks_and_traps_and_all_and_a_chest_below_its_hardness() {
    let mut w = hall();
    let (lock, trap) = (cell(12, 3), cell(14, 3));
    assert!(w.state.current().locks.contains_key(&lock));
    assert!(w.state.current().traps.contains_key(&trap));
    cast(&mut w, "frost", cell(13, 4), None);
    let hp = w.state.player().max_hp(w.content());
    for (x, y) in DOORS {
        assert_eq!(terrain(&w, cell(x, y)), "door_broken", "({x}, {y})");
    }
    let level = w.state.current();
    assert!(!level.locks.contains_key(&lock), "the lock went with it");
    assert!(!level.traps.contains_key(&trap), "the trap too");
    assert_eq!(w.state.player().hp, hp, "sprung at no one");
    assert_eq!(terrain(&w, cell(20, 3)), "door_closed", "out of its reach");
    assert_eq!(
        terrain(&w, cell(11, 3)),
        "stout_door",
        "nothing to break to"
    );
    assert_eq!(
        terrain(&w, cell(14, 5)),
        "chest_smashed",
        "a coffer, below 1000"
    );
    assert_eq!(terrain(&w, cell(15, 5)), "chest", "a kist, above");
    assert_ne!(terrain(&w, cell(16, 3)), "wall", "the wall gave");
    assert_eq!(
        terrain(&w, cell(16, 2)),
        "door_closed",
        "a blast never reaches through the wall it breaks"
    );
    assert!(
        (w.messages().iter()).any(|m| m.text == "The door crashes open!"),
        "the player saw it"
    );
}

#[test]
fn a_fireball_leaves_doors_standing_and_blows_chests_apart_as_ever() {
    let mut w = hall();
    cast(&mut w, "fireball", cell(13, 4), None);
    for (x, y) in DOORS[1..].iter().copied() {
        assert_eq!(terrain(&w, cell(x, y)), "door_closed", "({x}, {y})");
    }
    assert_eq!(terrain(&w, cell(14, 5)), "chest_smashed");
}

#[test]
fn a_force_burst_without_a_blast_and_a_blast_that_is_no_spell_read_no_door() {
    let mut w = hall();
    let player = w.state.player;
    let here = w.player_pos();
    let at = cell(13, 4);
    let bare = Burst {
        radius: 4,
        ..burst(&w, 1, 0)
    };
    w.cast_burst(Caster::Actor(player), (here, at), &bare, &mut Vec::new());
    assert_eq!(terrain(&w, cell(12, 3)), "door_closed", "no blast");
    let placed = Burst {
        radius: 4,
        ..burst(&w, 1, 1000)
    };
    w.cast_burst(Caster::Place, (at, at), &placed, &mut Vec::new());
    assert_eq!(terrain(&w, cell(12, 3)), "door_closed", "no spell");
}

/// Ice `um` thick on the water at `p`.
fn ice(w: &mut World, p: Pos, um: u32) {
    let content = w.content().clone();
    let level = w.state.current_mut();
    let water = level.terrain[p];
    level.set_ice(&content, p, water, um);
}

#[test]
fn a_force_blast_shatters_thin_ice_and_drops_who_stands_on_it() {
    let mut w = hall();
    let (thin, thick) = (cell(15, 6), cell(16, 6));
    ice(&mut w, thin, 60_000);
    ice(&mut w, thick, 200_000);
    assert!(w.state.current().ice_at(thin).is_some());
    let on_it = kobold_at(&mut w, (9, 1), 100);
    let player = w.state.player;
    let here = w.player_pos();
    let aimed = Burst {
        radius: 4,
        ..burst(&w, 1, 1000)
    };
    let mut events = Vec::new();
    w.cast_burst(
        Caster::Actor(player),
        (here, cell(13, 4)),
        &aimed,
        &mut events,
    );
    assert!(
        w.state.current().ice_at(thin).is_none(),
        "thinner than 10 cm"
    );
    assert!(w.state.current().ice_at(thick).is_some(), "20 cm holds");
    assert!(
        (events.iter()).any(|e| matches!(e, Event::FellThroughIce { .. })),
        "kobold {on_it:?} fell in: {events:?}"
    );

    let mut w = hall();
    ice(&mut w, thin, 60_000);
    let player = w.state.player;
    let here = w.player_pos();
    let bare = Burst {
        radius: 4,
        ..burst(&w, 1, 0)
    };
    w.cast_burst(
        Caster::Actor(player),
        (here, cell(13, 4)),
        &bare,
        &mut Vec::new(),
    );
    assert!(
        w.state.current().ice_at(thin).is_some(),
        "no blast, no shatter"
    );
}

#[test]
fn poison_poisons_a_degree_and_blight_still_poisons_two() {
    let mut w = hall();
    let poisoned = status(&w, "poisoned");
    let id = kobold_at(&mut w, (5, 0), 10_000);
    let at = pos(&w, id);
    w.player_mut().spells.insert(sp("blight"));
    let content = w.content().clone();
    let by = w.state.player;
    w.spell_type_lands(by, kind(&w, "poison"), &[id], &mut Vec::new());
    let degree = |w: &World| (w.state.actors[&id].effects.get(poisoned)).map(|e| e.severity);
    assert_eq!(degree(&w), Some(1), "the type's own");
    put_back(&mut w, id, at);
    assert_eq!(
        content.spell(sp("blight")).effect.status.map(|s| s.1),
        Some(1)
    );
    cast(&mut w, "blight", at, Some(id));
    assert_eq!(degree(&w), Some(2), "its line and the type's");
}

#[test]
fn necrotic_bites_a_creature_below_half_before_it_resists() {
    let mut w = hall();
    let (id, most, at) = target(&mut w);
    let from = cell(1, at.y);
    let taken = |w: &mut World, hp: i32, amount: i32| {
        let actor = w.state.actors.get_mut(&id).expect("there");
        actor.hp = hp;
        strike(w, id, from, "necrotic", amount, HIT);
        hp - w.state.actors[&id].hp
    };
    let half = (most + 1) / 2;
    assert_eq!(taken(&mut w, half, 5), 5, "at half and over, no bite");
    assert_eq!(taken(&mut w, half - 1, 5), 7, "half again, rounded down");
    assert_eq!(taken(&mut w, half - 1, 4), 6);
    w.state.actors.get_mut(&id).expect("there").hp = 2;
    strike(&mut w, id, from, "cold", 1, HIT);
    assert_eq!(w.state.actors[&id].hp, 1, "another type bites no one");
}

#[test]
fn the_undead_are_immune_to_necrotic_and_the_dump_says_so() {
    let mut w = hall();
    let (id, _, at) = target(&mut w);
    let zombie = w.content().creature_id("zombie").expect("a zombie");
    let content = w.content().clone();
    let actor = w.state.actors.get_mut(&id).expect("there");
    actor.creature = zombie;
    actor.hp = 2;
    let necrotic = kind(&w, "necrotic");
    assert_eq!(w.resistances_of(id).get(necrotic), 100);
    assert!(
        (content.creature(zombie).resist.iter()).any(|&(k, p)| Some(k) == necrotic && p == 100),
        "on its line, for all that reads one"
    );
    strike(&mut w, id, cell(1, at.y), "necrotic", 50, HIT);
    assert_eq!(w.state.actors[&id].hp, 2, "it took nothing");
    let rows = dump::layer_rows(&w, Layer::Actors).join("\n");
    assert!(rows.contains("resists poison 100 necrotic 100"), "{rows}");
    let kobold = content.creature_id("kobold").expect("kobolds");
    let resist = &content.creature(kobold).resist;
    assert!(!resist.iter().any(|&(k, _)| Some(k) == necrotic));
}

#[test]
fn corroded_eats_two_more_points_and_is_spent() {
    let mut w = hall();
    let corroded = status(&w, "corroded");
    let player = w.state.player;
    w.player_mut().hp_base = 100;
    w.player_mut().hp = 50;
    // Beside the same run without it, whatever a wait heals.
    let mut clean = World::from_state(w.content().clone(), w.state().clone()).expect("restores");
    w.spell_type_lands(player, kind(&w, "acid"), &[player], &mut Vec::new());
    assert!(w.state.player().effects.has(corroded));
    for _ in 0..20 {
        w.apply(&crate::action::Action::Wait);
        clean.apply(&crate::action::Action::Wait);
    }
    assert!(!w.state.player().effects.has(corroded));
    let eaten = clean.state.player().hp - w.state.player().hp;
    assert_eq!(eaten, 2, "its two ticks");
    let said = |line: &str| (w.messages().iter()).any(|m| m.text == line);
    assert!(said("The acid is spent."));
}
