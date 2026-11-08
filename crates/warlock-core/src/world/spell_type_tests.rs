//! The spell types (`world/spell_type.rs`, `[types]` in `data/spells.toml`): fire's burn and the
//! fire it leaves, cold's numbing and what it freezes and puts out, and who reads them.

use super::caster_tests::{kobold_at, sp};
use super::spell::Caster;
use super::*;
use crate::action::{Action, Event};
use crate::character::CharacterSpec;
use crate::combat::Strike;
use crate::content::{Burst, ContentFiles, Reading, StatusId};
use crate::stats::{PerStat, Stats};

/// A lit hall with a pond (x 5 to 7, y 2 to 4), the player at (2, 3), and a kobold walled up in a
/// closet (`kobold_at` copies one).
const HALL: &str = "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n\
    ######################\n#..................#K#\n#....~~~...........###\n#.@..~~~...........#\n\
    #....~~~...........#\n#..................#\n####################\n'''\n\
    [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
    \"~\" = { terrain = \"water\" }\n\
    \"K\" = { terrain = \"floor\", creature = \"kobold\" }\n";

pub(super) fn files() -> ContentFiles {
    let mut files = super::caster_tests::files();
    files.insert("levels/hall.toml", HALL);
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\n",
    );
    files
}

pub(super) fn start(content: Arc<Content>, seed: u64) -> World {
    let spec: CharacterSpec = "human,female,wizard".parse().expect("a character");
    let spec = spec.with_talents(vec!["keen_ears".into(), "lean".into(), "pious".into()]);
    let w = World::start_run(content.clone(), "hall", Some(seed), Some(&spec), None, true)
        .expect("world");
    let mut state = w.state().clone();
    state.character.as_mut().expect("a character").sign = None;
    let mut w = World::from_state(content, state).expect("restores");
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    for spell in ["fireball", "frost", "scorching_ray", "ray_of_frost"]
        .into_iter()
        .chain(["magic_missile", "blight", "grave_breath"])
    {
        w.player_mut().spells.insert(sp(spell), 1);
    }
    w
}

fn hall(seed: u64) -> World {
    start(Arc::new(Content::load(&files()).expect("content")), seed)
}

pub(super) fn status(w: &World, id: &str) -> StatusId {
    w.content().status_id(id).expect("a status")
}

pub(super) fn cell(x: i32, y: i32) -> Pos {
    Pos::new(x, y)
}

/// `target` stands at `at` again, whole and untouched by any status.
pub(super) fn put_back(w: &mut World, target: ActorId, at: Pos) {
    let actor = w.state.actors.get_mut(&target).expect("there");
    actor.pos = at;
    actor.hp = 10_000;
    actor.effects = Default::default();
}

/// The player, whole and with mana, untrained so no rank grows the dice, casts `spell` at `at`:
/// what the cast did to `target`.
pub(super) fn cast(w: &mut World, spell: &str, at: Pos, target: Option<ActorId>) -> Vec<Strike> {
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = max;
    w.player_mut().mana_base = 400;
    w.player_mut().mana = 400;
    w.player_mut().weapon_skills = Default::default();
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    let out = w.apply(&Action::Cast(spell.into(), Some(at)));
    (out.events.iter())
        .filter_map(|e| match e {
            Event::Attacked {
                target: t,
                strike: Some(strike),
                ..
            } if Some(*t) == target => Some(*strike),
            _ => None,
        })
        .collect()
}

/// How many of 400 Fireballs at a kobold (or `creature`) leave it burning, by `seed`.
fn burns(seed: u64, creature: Option<&str>) -> u32 {
    let mut w = hall(seed);
    let at = cell(12, 3);
    let target = kobold_at(&mut w, (10, 0), 10_000);
    if let Some(id) = creature {
        let id = w.content().creature_id(id).expect("a creature");
        w.state.actors.get_mut(&target).expect("there").creature = id;
    }
    let burning = status(&w, "burning");
    let mut count = 0;
    for _ in 0..400 {
        put_back(&mut w, target, at);
        cast(&mut w, "fireball", at, Some(target));
        count += u32::from(w.state.actors[&target].effects.has(burning));
    }
    count
}

#[test]
fn fire_burns_one_time_in_four_by_the_draw() {
    for seed in [1, 2, 3] {
        let count = burns(seed, None);
        assert!((70..=130).contains(&count), "seed {seed}: {count} of 400");
    }
    assert_eq!(burns(1, None), burns(1, None), "the same draws by seed");
}

#[test]
fn what_is_immune_to_fire_never_burns() {
    assert_eq!(burns(1, Some("hell_hound")), 0);
}

#[test]
fn a_burning_creature_sheds_its_small_light_while_it_burns() {
    let mut w = hall(1);
    let at = cell(12, 3);
    let target = kobold_at(&mut w, (10, 0), 10_000);
    let ember = w.content().light_id("ember").expect("ember");
    let shed = |w: &World| {
        let view = w.perception();
        let at = w.state.actors[&target].pos;
        (view.light_sources().iter()).any(|s| s.pos() == Some(at) && s.preset() == Some(ember))
    };
    assert!(!shed(&w), "no light before");
    let burning = status(&w, "burning");
    let mut events = Vec::new();
    w.afflict(target, burning, 1, None, &mut events);
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    assert!(shed(&w), "the ember's light");
    // Look mode names it: the seen creature's statuses that show.
    assert!(w.perception().effects_of(target).any(|s| s == burning));
    assert!(w.content().light(ember).radius <= 1, "a small one");
    // It burns out by itself in a degree's time, a hit point every ten seconds.
    let hp = w.state.actors[&target].hp;
    let mut waited = 0;
    while w.state.actors[&target].effects.has(burning) && waited < 60 {
        w.apply(&Action::Wait);
        w.state.actors.get_mut(&target).expect("there").pos = at;
        waited += 1;
    }
    assert!(!w.state.actors[&target].effects.has(burning), "burnt out");
    assert!(w.state.actors[&target].hp < hp, "and it cost hit points");
    assert!(!shed(&w), "no light after");
}

#[test]
fn a_step_into_water_puts_it_out_and_dry_ground_does_not() {
    let mut w = hall(1);
    let burning = status(&w, "burning");
    let player = w.state.player;
    let mut events = Vec::new();
    w.afflict(player, burning, 3, None, &mut events);
    w.apply(&Action::Move(crate::geom::Dir::E));
    assert_eq!(w.player_pos(), cell(3, 3));
    assert!(w.state.player().effects.has(burning), "dry ground");
    w.apply(&Action::Move(crate::geom::Dir::E));
    assert!(w.state.player().effects.has(burning), "dry ground");
    w.apply(&Action::Move(crate::geom::Dir::E));
    assert_eq!(w.player_pos(), cell(5, 3), "in the pond");
    assert!(!w.state.player().effects.has(burning), "put out");
    let told: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    assert!(
        told.iter().any(|m| m == "The flames go out."),
        "and told: {told:?}"
    );
}

/// The rider a cold spell lays: every target harmed, none missed (Numbing cold's, now the type's).
#[test]
fn cold_numbs_every_target_harmed_and_none_missed() {
    let mut w = hall(1);
    let sluggish = status(&w, "sluggish");
    let at = cell(4, 3);
    let target = kobold_at(&mut w, (2, 0), 10_000);
    let (mut hit, mut missed) = (0, 0);
    for _ in 0..200 {
        put_back(&mut w, target, at);
        let strikes = cast(&mut w, "ray_of_frost", at, Some(target));
        let slowed = w.state.actors[&target].effects.has(sluggish);
        if strikes[0].is_hit() {
            hit += 1;
            assert!(slowed, "harmed: Sluggish");
        } else {
            missed += 1;
            assert!(!slowed, "missed: nothing");
        }
    }
    assert!(hit > 0 && missed > 0, "{hit} hit, {missed} missed");
}

#[test]
fn another_types_spell_does_not_numb_and_a_cold_one_ends_burning() {
    let mut w = hall(1);
    let (sluggish, burning) = (status(&w, "sluggish"), status(&w, "burning"));
    let at = cell(12, 3);
    let target = kobold_at(&mut w, (10, 0), 10_000);
    put_back(&mut w, target, at);
    cast(&mut w, "fireball", at, Some(target));
    assert!(!w.state.actors[&target].effects.has(sluggish), "not cold");
    put_back(&mut w, target, at);
    let mut events = Vec::new();
    w.afflict(target, burning, 2, None, &mut events);
    cast(&mut w, "frost", at, Some(target));
    let effects = &w.state.actors[&target].effects;
    assert!(!effects.has(burning), "the flames are out");
    assert!(effects.has(sluggish), "and it is numb");
}

#[test]
fn a_ray_of_frost_freezes_the_cell_it_strikes_and_a_scorching_ray_melts_it() {
    let mut w = hall(1);
    let aim = cell(5, 3);
    cast(&mut w, "ray_of_frost", aim, None);
    let level = w.state.current();
    assert!(level.ice_at(aim).is_some(), "the cell aimed at");
    for p in [cell(6, 3), cell(7, 3), cell(5, 2), cell(5, 4)] {
        assert!(level.ice_at(p).is_none(), "that cell alone: {p:?}");
    }
    cast(&mut w, "scorching_ray", aim, None);
    assert!(w.state.current().ice_at(aim).is_none(), "melted");
    // At a creature it is the creature's cell.
    let target = kobold_at(&mut w, (4, 0), 10_000);
    put_back(&mut w, target, cell(6, 3));
    let strikes = cast(&mut w, "ray_of_frost", cell(8, 3), Some(target));
    assert_eq!(strikes.len(), 1);
    assert!(w.state.current().ice_at(cell(6, 3)).is_some());
    assert!(w.state.current().ice_at(cell(7, 3)).is_none());
}

#[test]
fn a_burst_freezes_or_melts_by_its_type_with_no_field_on_the_spell() {
    let mut w = hall(1);
    assert_eq!(w.content().spell(sp("frost")).look.ice, None);
    assert_eq!(w.content().spell(sp("fireball")).look.ice, None);
    cast(&mut w, "frost", cell(6, 3), None);
    let frozen = |w: &World| {
        (2..=4)
            .flat_map(|y| (5..=7).map(move |x| cell(x, y)))
            .filter(|&p| w.state.current().ice_at(p).is_some())
            .count()
    };
    assert_eq!(frozen(&w), 9, "the whole pond");
    let um = w.state.current().ice_at(cell(6, 3)).expect("ice").um;
    assert_eq!(um, 100_000, "100 mm, as Frost's field said");
    cast(&mut w, "fireball", cell(6, 3), None);
    assert_eq!(frozen(&w), 0, "melted");
}

#[test]
fn a_spells_own_field_stands_in_for_its_types() {
    let mut files = files();
    let text = "[[spell]]\nid = \"frost\"\nname = \"Frost\"\ndescription = \"x\"\nlevel = 6\n\
                school = \"evocation\"\nkind = \"damage\"\nshape = \"burst\"\n\
                resolution = \"area\"\ndamage_type = \"cold\"\nrange = 10\nradius = 2\n\
                damage = \"2d6\"\n[spell.cast]\nfrost = 10\n";
    files.insert("data/spells/frost.toml", text);
    let mut w = start(Arc::new(Content::load(&files).expect("loads")), 1);
    cast(&mut w, "frost", cell(6, 3), None);
    let um = w.state.current().ice_at(cell(6, 3)).expect("ice").um;
    assert_eq!(um, 10_000, "its own 10 mm");
}

#[test]
fn a_fire_burst_leaves_a_fire_on_its_centre_and_frost_puts_it_out() {
    let mut w = hall(1);
    let at = cell(12, 3);
    let lasts = crate::time::Duration::seconds(60);
    let before = w.state.time;
    cast(&mut w, "fireball", at, None);
    let fires = &w.state.current().fires;
    assert_eq!(
        fires.keys().copied().collect::<Vec<_>>(),
        [at],
        "its centre"
    );
    let fire = fires[&at];
    assert!(fire.until <= w.state.time + lasts && fire.until >= before + lasts);
    assert_eq!(
        fire.light,
        w.content().light_id("arrow_fire").expect("light")
    );
    // A cast later the fire stays where it was: it never spreads.
    w.apply(&Action::Wait);
    assert_eq!(w.state.current().fires.len(), 1);
    cast(&mut w, "frost", cell(13, 3), None);
    assert!(w.state.current().fires.is_empty(), "doused by the burst");
    // A ray of frost puts out the one it strikes.
    w.lay_fire(cell(4, 3), fire.light, 60);
    cast(&mut w, "ray_of_frost", cell(4, 3), None);
    assert!(w.state.current().fires.is_empty(), "doused by the ray");
}

#[test]
fn a_fire_is_laid_on_walkable_ground_only() {
    let mut w = hall(1);
    let light = w.content().light_id("ember").expect("light");
    w.lay_fire(cell(0, 0), light, 60);
    assert!(w.state.current().fires.is_empty(), "not in a wall");
}

#[test]
fn a_burst_that_is_no_spell_reads_no_type() {
    let mut w = hall(1);
    let burst = Burst {
        spell: String::new(),
        radius: 2,
        damage: crate::dice::Roll::fixed(1),
        reroll: 0,
        kind: w.content().damage_type_id("fire"),
        blast: 0,
        ice: None,
    };
    let at = cell(12, 3);
    let mut events = Vec::new();
    w.cast_burst(Caster::Place, (at, at), &burst, &mut events);
    assert!(w.state.current().fires.is_empty(), "no fire left");
}

#[test]
fn the_rider_hook_stays_for_a_talent_beside_the_types_own() {
    let mut files = files();
    let text = "[[talent]]\nid = \"scorcher\"\nname = \"Scorcher\"\ntree = \"spellcraft\"\n\
                description = \"x\"\nsystems = [\"spells\"]\neffects = [{ kind = \"spell_mod\", \
                rider = { status = \"sluggish\", severity = 1 }, when = [\"fire\"] }]\n";
    files.insert("data/talents/scorcher.toml", text);
    let mut w = start(Arc::new(Content::load(&files).expect("loads")), 1);
    w.player_mut().talents.take("scorcher", None);
    let sluggish = status(&w, "sluggish");
    let at = cell(12, 3);
    let target = kobold_at(&mut w, (10, 0), 10_000);
    put_back(&mut w, target, at);
    cast(&mut w, "fireball", at, Some(target));
    assert!(
        w.state.actors[&target].effects.has(sluggish),
        "the talent's"
    );
}

#[test]
fn the_menu_and_the_dump_say_the_rule_after_the_roll() {
    let w = hall(1);
    let view = w.perception();
    let text = |id: &str| view.own_spell_roll(sp(id)).map(|r| r.text);
    assert_eq!(text("ray_of_frost").as_deref(), Some("2d6 cold, numbs"));
    assert_eq!(
        text("scorching_ray")
            .as_deref()
            .map(|t| t.split_once(' ').map(|x| x.1.to_string())),
        Some(Some("fire, burns one in four".to_string()))
    );
    // The dice left out: they grow with the score.
    let rule = |id: &str| {
        let text = view.own_spell_roll(sp(id)).expect("a roll").text;
        text.split_once(", ").map(|x| x.1.to_string())
    };
    assert_eq!(
        rule("magic_missile").as_deref(),
        Some("shoves on a big hit")
    );
    assert_eq!(rule("blight").as_deref(), Some("poisons"));
    assert_eq!(rule("grave_breath").as_deref(), Some("bites the wounded"));
}

#[test]
fn a_backfired_scroll_of_a_type_carries_its_rule() {
    // A cold spell's scroll going wrong in the reader's face numbs them too (the design's §4).
    let mut w = hall(1);
    let sluggish = status(&w, "sluggish");
    let object = w
        .content()
        .object_id("scroll_of_frost")
        .expect("the scroll");
    let content = w.content().clone();
    let Some(Reading::Cast(scroll)) = content.object(object).read else {
        panic!("a spell's scroll");
    };
    w.player_mut().hp = 1000;
    let mut events = Vec::new();
    w.backfire_spell(scroll.spell, object, &mut events);
    assert!(w.state.player().effects.has(sluggish));
}
