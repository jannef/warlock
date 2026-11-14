//! How electricity spreads (`world/magic/spell_arc.rs`): the arc's draw, its jump back at the caster
//! and an ally, Chain lightning's certain jumps and their end, the pond, Conductor, and ice.

use super::caster_tests::kobold_at;
use crate::action::{Action, AttackKind, Event, parse_actions};
use crate::character::CharacterSpec;
use crate::content::ContentFiles;
use crate::stats::{PerStat, Stats};
use crate::test_support::draws;
use crate::world::magic::cast::Reach;
use crate::world::*;

/// A lit hall with a pond (x 8 to 10, y 2 to 4), the player at (2, 3), and a kobold walled up in a
/// closet (`kobold_at` copies one).
const HALL: &str = "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n\
    #########################\n#.....................#K#\n#.......~~~...........###\n\
    #.@.....~~~...........#\n#.......~~~...........#\n#.....................#\n\
    #######################\n'''\n\
    [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
    \"~\" = { terrain = \"water\" }\n\
    \"K\" = { terrain = \"floor\", creature = \"kobold\" }\n";

/// A bolt of fixed harm, and a burst, so what each creature lost tells the roll.
fn zap(id: &str, more: &str) -> String {
    let resolution = if more.contains("burst") {
        "area"
    } else {
        "auto"
    };
    format!(
        "[[spell]]\nid = \"{id}\"\nname = \"{id}\"\ndescription = \"A test.\"\nlevel = 1\n\
         school = \"evocation\"\nkind = \"damage\"\nresolution = \"{resolution}\"\n\
         damage_type = \"electricity\"\nrange = 16\ndamage = \"8\"\n{more}\n\
         [spell.cast]\nnoun = \"zap\"\nverb = \"strikes\"\n"
    )
}

fn files() -> ContentFiles {
    let mut files = super::caster_tests::files();
    files.insert("levels/hall.toml", HALL);
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\n",
    );
    files.insert(
        "data/spells/zz_zap.toml",
        &zap("zz_zap", "shape = \"bolt\""),
    );
    files.insert(
        "data/spells/zz_blast.toml",
        &zap("zz_blast", "shape = \"burst\"\nradius = 1"),
    );
    files
}

fn hall(seed: u64) -> World {
    let content = Arc::new(Content::load(&files()).expect("content"));
    let spec: CharacterSpec = "human,female,wizard".parse().expect("a character");
    let spec = spec.with_talents(vec!["keen_ears".into(), "lean".into(), "pious".into()]);
    let w = World::start_run(content.clone(), "hall", Some(seed), Some(&spec), None, true)
        .expect("world");
    let mut state = w.state().clone();
    state.character.as_mut().expect("a character").sign = None;
    let mut w = World::from_state(content, state).expect("restores");
    // Below every rung of Chain lightning's ladders.
    w.player_mut().stats = Stats::new(PerStat::splat(10), PerStat::splat(40));
    for spell in [
        "spark",
        "chain_lightning",
        "lightning_bolt",
        "zz_zap",
        "zz_blast",
    ] {
        learn(&mut w, spell);
    }
    w
}

fn learn(w: &mut World, spell: &str) {
    let id = w.content.spell_id(spell).expect("a spell");
    w.player_mut().spells.insert(id, 1);
}

fn cell(x: i32, y: i32) -> Pos {
    Pos::new(x, y)
}

/// A kobold with hit points to spare on level cell (`x`, `y`).
fn kobold(w: &mut World, x: i32, y: i32) -> ActorId {
    let here = w.player_pos();
    let id = kobold_at(w, (x - here.x, y - here.y), 500);
    whole(w, id, cell(x, y));
    id
}

/// `id` stands whole on `at` again.
fn whole(w: &mut World, id: ActorId, at: Pos) {
    let max = w.state.actors[&id].max_hp(&w.content);
    let actor = w.state.actors.get_mut(&id).expect("there");
    actor.pos = at;
    actor.hp = max;
}

/// The player on `at`, whole.
fn stand(w: &mut World, at: Pos) {
    whole(w, w.state.player, at);
}

/// The player, with mana and untrained, casts `spell` at `at`: everyone the spell struck at, in
/// the order struck.
fn cast(w: &mut World, spell: &str, at: Pos) -> Vec<ActorId> {
    w.player_mut().mana_base = 400;
    w.player_mut().mana = 400;
    w.player_mut().weapon_skills = Default::default();
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    let out = w.apply(&Action::Cast(spell.into(), Some(at)));
    (out.events.iter())
        .filter_map(|e| match e {
            Event::Attacked {
                target,
                kind: AttackKind::Spell,
                ..
            } => Some(*target),
            _ => None,
        })
        .collect()
}

/// The hit points `id` is short of.
fn lost(w: &World, id: ActorId) -> i32 {
    let actor = &w.state.actors[&id];
    actor.max_hp(&w.content) - actor.hp
}

#[test]
fn an_arc_jumps_one_time_in_four_by_the_draw() {
    let sparks = |seed: u64| {
        let mut w = hall(seed);
        let (at, beside) = (cell(6, 3), cell(6, 4));
        let target = kobold(&mut w, at.x, at.y);
        let other = kobold(&mut w, beside.x, beside.y);
        let mut arcs = 0;
        for _ in 0..400 {
            whole(&mut w, target, at);
            whole(&mut w, other, beside);
            stand(&mut w, cell(2, 3));
            let struck = cast(&mut w, "spark", at);
            assert_eq!(struck[0], target);
            arcs += u32::from(struck.contains(&other));
        }
        arcs
    };
    for seed in [1, 2, 3] {
        let arcs = sparks(seed);
        assert!((70..=130).contains(&arcs), "seed {seed}: {arcs} of 400");
    }
    assert_eq!(sparks(1), sparks(1), "the same draws by seed");
}

#[test]
fn with_no_one_beside_it_an_arc_makes_no_draw_and_with_one_it_makes_one() {
    let mut w = hall(1);
    kobold(&mut w, 6, 3);
    let before = draws(&w);
    cast(&mut w, "zz_zap", cell(6, 3));
    assert_eq!(draws(&w), before + 1, "the cast's roll; no one to jump to");
    let mut w = hall(1);
    kobold(&mut w, 6, 3);
    w.player_mut().talents.take("conductor", None);
    kobold(&mut w, 6, 4);
    let before = draws(&w);
    cast(&mut w, "zz_zap", cell(6, 3));
    assert_eq!(draws(&w), before + 1, "a chance of 100 draws nothing");
    let mut w = hall(1);
    kobold(&mut w, 6, 3);
    kobold(&mut w, 6, 4);
    let before = draws(&w);
    cast(&mut w, "zz_zap", cell(6, 3));
    assert_eq!(draws(&w), before + 2, "the type's chance draws");
}

#[test]
fn an_arc_jumps_back_at_the_caster_and_they_take_the_same_roll() {
    let mut w = hall(1);
    w.player_mut().talents.take("conductor", None);
    let target = kobold(&mut w, 3, 3);
    // It dies of it, so its own blow does not muddle what the arc took.
    w.state.actors.get_mut(&target).expect("there").hp = 1;
    stand(&mut w, cell(2, 3));
    let struck = cast(&mut w, "zz_zap", cell(3, 3));
    let me = w.state.player;
    assert_eq!(struck, [target, me], "the only one beside the kobold");
    // 25 % electricity resistance: Conductor bites less, and spares nothing.
    assert_eq!(lost(&w, me), 6);
}

#[test]
fn an_arc_with_no_talent_jumps_back_at_the_caster_too() {
    let mut w = hall(1);
    let target = kobold(&mut w, 3, 3);
    let me = w.state.player;
    let mut back = 0;
    for _ in 0..200 {
        whole(&mut w, target, cell(3, 3));
        stand(&mut w, cell(2, 3));
        back += u32::from(cast(&mut w, "spark", cell(3, 3)).contains(&me));
    }
    assert!((25..=75).contains(&back), "{back} of 200");
}

#[test]
fn an_arc_kills_the_caster_as_their_own_spell() {
    let mut w = hall(1);
    w.player_mut().talents.take("conductor", None);
    w.player_mut().hp = 1;
    kobold(&mut w, 3, 3);
    cast(&mut w, "zz_zap", cell(3, 3));
    let death = w.state.death.as_ref().expect("dead");
    assert!(matches!(death.cause, Some(Cause::Spell(_))));
}

#[test]
fn an_arc_jumps_at_an_ally() {
    let mut w = hall(1);
    w.player_mut().talents.take("conductor", None);
    let target = kobold(&mut w, 6, 3);
    let summoned = parse_actions("ally:wolf").expect("the action");
    w.apply(&summoned[0]);
    let ally = (w.state.actors.values())
        .find(|a| a.ally.is_some())
        .map(|a| a.id)
        .expect("a wolf at the player's side");
    whole(&mut w, ally, cell(6, 4));
    assert_eq!(cast(&mut w, "zz_zap", cell(6, 3)), [target, ally]);
}

#[test]
fn an_arc_never_jumps_at_one_struck_already_and_ties_go_by_actor_id() {
    let mut w = hall(1);
    w.player_mut().talents.take("conductor", None);
    let target = kobold(&mut w, 5, 3);
    // Two beside the target, the higher id first in row order: the lower id goes first.
    let low = kobold(&mut w, 6, 4);
    let high = kobold(&mut w, 4, 4);
    // It does not arc back at the target, and `high` is two cells from it, beyond an arc.
    assert_eq!(cast(&mut w, "zz_zap", cell(5, 3)), [target, low]);
    assert_eq!(lost(&w, high), 0);
}

#[test]
fn a_line_arcs_from_everyone_it_harmed() {
    let mut w = hall(1);
    w.player_mut().talents.take("conductor", None);
    let (a, b) = (kobold(&mut w, 5, 5), kobold(&mut w, 7, 5));
    let (c, d) = (kobold(&mut w, 5, 4), kobold(&mut w, 7, 4));
    stand(&mut w, cell(2, 5));
    let struck = cast(&mut w, "lightning_bolt", cell(9, 5));
    assert_eq!(struck, [a, b, c, d]);
}

#[test]
fn a_chain_takes_its_certain_jumps_and_its_reach_is_two_cells() {
    for seed in [1, 2, 3, 4, 5] {
        let mut w = hall(seed);
        stand(&mut w, cell(2, 1));
        let ids = [6, 8, 10, 12].map(|x| kobold(&mut w, x, 1));
        // Three jumps at these stats, over gaps a plain arc could not cross.
        assert_eq!(cast(&mut w, "chain_lightning", cell(6, 1)), ids, "{seed}");
    }
}

#[test]
fn a_chains_jumps_past_its_certain_ones_are_the_types_draws() {
    let mut w = hall(1);
    stand(&mut w, cell(2, 1));
    let at = [6, 8, 10, 12, 14];
    let ids = at.map(|x| kobold(&mut w, x, 1));
    let mut further = 0;
    for _ in 0..400 {
        for (&x, &id) in at.iter().zip(&ids) {
            whole(&mut w, id, cell(x, 1));
        }
        stand(&mut w, cell(2, 1));
        let struck = cast(&mut w, "chain_lightning", cell(6, 1));
        assert_eq!(struck[..4], ids[..4], "the first four are certain");
        further += u32::from(struck.len() == 5);
    }
    assert!((70..=130).contains(&further), "{further} of 400");
}

#[test]
fn a_chain_ends_with_no_one_in_reach_and_never_jumps_through_a_wall() {
    let mut w = hall(1);
    stand(&mut w, cell(14, 1));
    let target = kobold(&mut w, 21, 1);
    // The closet's kobold is two cells from (21, 1), behind the wall at (22, 1).
    assert_eq!(cast(&mut w, "chain_lightning", cell(21, 1)), [target]);
    let far = kobold(&mut w, 18, 1);
    whole(&mut w, target, cell(21, 1));
    stand(&mut w, cell(14, 1));
    assert_eq!(
        cast(&mut w, "chain_lightning", cell(21, 1)),
        [far],
        "the first in its way, then none within two"
    );
}

#[test]
fn chain_lightnings_jumps_grow_by_its_ladder_and_are_shown_as_reach() {
    let mut w = hall(1);
    let chain = w.content.spell_id("chain_lightning").expect("a spell");
    let def = w.content.spell(chain).clone();
    let jumps = |w: &World| w.state.player().spell_reach(&w.content, &def).jumps;
    assert_eq!(jumps(&w), Some(3));
    for (score, n) in [(14, 4), (18, 5), (22, 6)] {
        w.player_mut().stats = Stats::new(PerStat::splat(score), PerStat::splat(40));
        assert_eq!(jumps(&w), Some(n));
    }
    let view = w.perception();
    let reach: Reach = view.own_spell_reach(chain).expect("known");
    assert_eq!(
        crate::world::reach_words(def.shape, reach),
        ["range 10", "jumps 6"]
    );
    let text = view.own_spell_roll(chain).expect("a roll").text;
    assert!(text.ends_with("electricity, may arc, conducts"), "{text}");
}

#[test]
fn a_pond_strikes_every_wader_in_id_order_the_caster_too() {
    let mut w = hall(1);
    let target = kobold(&mut w, 8, 2);
    let (one, two) = (kobold(&mut w, 10, 4), kobold(&mut w, 8, 4));
    let dry = kobold(&mut w, 13, 5);
    stand(&mut w, cell(10, 2));
    let me = w.state.player;
    let struck = cast(&mut w, "zz_zap", cell(8, 2));
    // The player's id is the lowest; then the rest in id order, not nearest first.
    assert_eq!(struck[..4], [target, me, one, two]);
    assert!(!struck.contains(&dry), "not in the water");
    assert_eq!(lost(&w, one), 8, "the same roll");
    assert_eq!(lost(&w, two), 8);
    assert_eq!(lost(&w, me), 8);
}

#[test]
fn a_pond_strikes_a_caster_with_conductor_too_at_a_quarter_less() {
    let mut w = hall(1);
    w.player_mut().talents.take("conductor", None);
    let target = kobold(&mut w, 8, 2);
    stand(&mut w, cell(8, 4));
    let me = w.state.player;
    assert_eq!(cast(&mut w, "zz_zap", cell(8, 2)), [target, me]);
    assert_eq!(lost(&w, target), 8);
    assert_eq!(lost(&w, me), 6);
}

#[test]
fn a_caster_on_the_shore_is_not_struck() {
    let mut w = hall(1);
    let target = kobold(&mut w, 8, 2);
    stand(&mut w, cell(2, 3));
    let me = w.state.player;
    assert!(!cast(&mut w, "zz_zap", cell(8, 2)).contains(&me));
    assert_eq!(lost(&w, target), 8);
    assert_eq!(lost(&w, me), 0);
}

#[test]
fn ice_is_not_water_and_parts_a_pond() {
    let mut w = hall(1);
    let target = kobold(&mut w, 8, 2);
    let far = kobold(&mut w, 10, 2);
    let on_ice = kobold(&mut w, 9, 3);
    // A frozen column between them.
    for y in 2..=4 {
        w.frost(cell(9, y), 0, 100_000);
    }
    assert_eq!(w.water_body(cell(8, 2)).len(), 3);
    assert!(w.water_body(cell(9, 3)).is_empty(), "ice");
    // The one on the ice stands beside the target, within an arc's plain reach: that arc's
    // chance is rigged to fail, so only the water could carry the cast on, and ice is no water.
    crate::test_support::rig(&mut w, "spell_arc/jump/player", |rng| rng.below(100) == 99);
    let struck = cast(&mut w, "zz_zap", cell(8, 2));
    assert_eq!(struck, [target]);
    assert_eq!(lost(&w, far), 0, "on the other side of the ice");
    assert_eq!(lost(&w, on_ice), 0, "on the ice");
}

#[test]
fn one_that_resisted_it_all_spreads_nothing() {
    let mut w = hall(1);
    w.player_mut().talents.take("conductor", None);
    let target = kobold(&mut w, 6, 3);
    kobold(&mut w, 6, 4);
    let immune = w.content().creature_id("lightning_elemental").expect("one");
    w.state.actors.get_mut(&target).expect("there").creature = immune;
    assert_eq!(cast(&mut w, "zz_zap", cell(6, 3)), [target]);
}

#[test]
fn a_burst_conducts_but_never_arcs() {
    let mut w = hall(1);
    w.player_mut().talents.take("conductor", None);
    let target = kobold(&mut w, 8, 2);
    let wader = kobold(&mut w, 10, 4);
    // On dry land beside the wader, far from the burst: an arc would take it.
    let beside = kobold(&mut w, 11, 5);
    let struck = cast(&mut w, "zz_blast", cell(8, 2));
    assert!(
        struck.contains(&target) && struck.contains(&wader),
        "{struck:?}"
    );
    assert!(!struck.contains(&beside));
}

#[test]
fn other_types_never_spread() {
    let mut w = hall(1);
    learn(&mut w, "arcane_bolt");
    let target = kobold(&mut w, 8, 2);
    let wader = kobold(&mut w, 10, 4);
    let struck = cast(&mut w, "arcane_bolt", cell(8, 2));
    assert!(struck.contains(&target) && !struck.contains(&wader));
}

#[test]
fn conductor_loads_and_is_what_the_owner_set() {
    let content = Content::load(&files()).expect("content");
    let id = content.talents().id("conductor").expect("Conductor");
    assert_eq!(content.talents().get(id).needs.level, 12);
    let mut w = hall(1);
    w.player_mut().talents.take("conductor", None);
    let mods = w.state.player().talent_mods(&w.content);
    let electricity = w.content.damage_type_id("electricity").expect("a type");
    let fire = w.content.damage_type_id("fire").expect("a type");
    assert_eq!(mods.magic.arc_chance(electricity), 100);
    assert_eq!(mods.magic.arc_chance(fire), 0);
}

#[test]
fn conductors_arc_goes_every_time_and_only_for_electricity() {
    let mut w = hall(1);
    w.player_mut().talents.take("conductor", None);
    learn(&mut w, "arcane_bolt");
    let (at, beside) = (cell(6, 3), cell(6, 4));
    let (target, other) = (
        kobold(&mut w, at.x, at.y),
        kobold(&mut w, beside.x, beside.y),
    );
    for _ in 0..100 {
        whole(&mut w, target, at);
        whole(&mut w, other, beside);
        stand(&mut w, cell(2, 3));
        assert_eq!(cast(&mut w, "spark", at), [target, other]);
    }
    whole(&mut w, target, at);
    whole(&mut w, other, beside);
    stand(&mut w, cell(2, 3));
    assert!(!cast(&mut w, "arcane_bolt", at).contains(&other));
}

#[test]
fn a_chain_replays_the_same() {
    let run = || {
        let mut w = hall(7);
        stand(&mut w, cell(2, 1));
        for x in [6, 8, 10, 12, 14] {
            kobold(&mut w, x, 1);
        }
        cast(&mut w, "chain_lightning", cell(6, 1));
        cast(&mut w, "spark", cell(6, 1));
        w.state_hash()
    };
    assert_eq!(run(), run());
}

/// What loading a spell of `body` says.
fn loads(body: &str) -> Result<Content, String> {
    let mut files = files();
    files.insert("data/spells/zz_bad.toml", &zap("zz_bad", body));
    Content::load(&files).map_err(|e| e.message)
}

#[test]
fn arcs_are_checked_as_they_load() {
    assert!(loads("shape = \"bolt\"\neffect = { arcs = 3 }").is_ok());
    for (body, says) in [
        (
            "shape = \"burst\"\nradius = 1\neffect = { arcs = 3 }",
            "`arcs` is a ray's",
        ),
        (
            "shape = \"bolt\"\n[spell.grows]\ncount = \"willpower 14 4\"",
            "`count` is for",
        ),
        (
            "shape = \"bolt\"\neffect = { arcs = 3 }\n[spell.grows]\ncount = \"willpower 14 1d6+3\"",
            "no dice",
        ),
    ] {
        let e = loads(body).err().unwrap_or_else(|| panic!("{body} loads"));
        assert!(e.contains(says), "{body}: {e}");
    }
    let mut files = files();
    let cold =
        zap("zz_cold", "shape = \"bolt\"\neffect = { arcs = 3 }").replace("electricity", "cold");
    files.insert("data/spells/zz_cold.toml", &cold);
    let Err(e) = Content::load(&files) else {
        panic!("a type that does not arc is refused");
    };
    let e = e.message;
    assert!(e.contains("that arcs"), "{e}");
}
