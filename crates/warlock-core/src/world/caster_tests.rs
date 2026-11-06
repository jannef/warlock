//! The casters' class tables in the world (spec §8 "Class abilities"; #245): the priest's,
//! druid's, wizard's, and witch's level tables, their spells (migrated onto the one cast path,
//! #395: paid in mana by the spell level), and the passives with rules of their own.

use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::{Factor, LevelKind, SpellId, StatusId};
use crate::dump::{self, Layer};
use crate::replay::Replay;
use crate::stats::{PerStat, Stat, Stats};
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// A human of `class` in the showcase (seed 7), with their kit, born under no sign, every score
/// 14, and plenty of mana.
fn world(class: &str) -> World {
    world_with(BUILTIN.clone(), class)
}

/// [`world`] with `content`.
pub(super) fn world_with(content: Arc<Content>, class: &str) -> World {
    let spec: CharacterSpec = format!("human,female,{class}")
        .parse()
        .expect("a character");
    // Creation picks that no test here reads (a human's three: Versatile adds one).
    let spec = spec.with_talents(vec!["keen_ears".into(), "lean".into(), "pious".into()]);
    let w = World::start_run(
        content.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let mut state = w.state().clone();
    state.character.as_mut().expect("a character").sign = None;
    let mut w = World::from_state(content, state).expect("restores");
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = max;
    w.player_mut().mana_base = 60;
    w.player_mut().mana = 40;
    w
}

pub(super) fn run(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

pub(super) fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

pub(super) fn last(world: &World) -> String {
    said(world).last().cloned().unwrap_or_default()
}

/// Grants the XP that takes the player to `level` and spends every choice it brings, mana
/// topped up again.
pub(super) fn reach(world: &mut World, level: u8) {
    let spec = world.state.character.as_ref().map(|c| &c.spec);
    let factor = Factor::of(world.content(), spec);
    let need = world.content().experience().threshold_for(level, factor);
    let more = need - world.player_xp();
    run(world, &format!("xp:{more}"));
    assert_eq!(world.player_level(), level, "reached level {level}");
    if world.due().is_some() {
        run(world, "levelup:auto");
    }
    assert_eq!(world.due(), None, "every choice spent");
    world.player_mut().mana = 40;
}

fn status(world: &World, id: &str) -> StatusId {
    world.content().status_id(id).expect("a status")
}

fn suffers(world: &World, who: ActorId, id: &str) -> bool {
    world.state.actors[&who].effects.has(status(world, id))
}

fn class_rows(world: &World) -> Vec<String> {
    (dump::layer_rows(world, Layer::Talents).into_iter())
        .filter_map(|row| row.strip_suffix(" (class)").map(str::to_string))
        .collect()
}

/// The spell of id `name` in the shipped content.
pub(super) fn sp(name: &str) -> SpellId {
    BUILTIN.spell_id(name).expect("a spell")
}

fn own(world: &World, spell: SpellId) -> Option<Option<Unavailable>> {
    (world.perception().abilities().into_iter())
        .find(|a| a.ability == Ability::Spell(spell))
        .map(|a| a.unavailable)
}

/// A kobold `dx` cells east of the player with `hp` hit points; its id.
pub(super) fn kobold(world: &mut World, dx: i32, hp: i32) -> ActorId {
    kobold_at(world, (dx, 0), hp)
}

/// A kobold `(dx, dy)` from the player with `hp` hit points; its id.
pub(super) fn kobold_at(world: &mut World, (dx, dy): (i32, i32), hp: i32) -> ActorId {
    let content = world.content().clone();
    let here = world.player_pos();
    let at = Pos::new(here.x + dx, here.y + dy);
    let template = (world.state.actors.values())
        .find(|a| a.id != world.state.player)
        .cloned()
        .expect("a creature in the showcase");
    let id = ActorId(world.state.next_actor_id);
    world.state.next_actor_id += 1;
    let creature = content.creature_id("kobold").expect("kobolds");
    let def = content.creature(creature);
    let actor = Actor {
        id,
        creature,
        pos: at,
        patrol: None,
        npc: None,
        faction_override: None,
        summoner: None,
        ally: None,
        risen: None,
        alignment_override: None,
        inventory: Vec::new(),
        equipment: Default::default(),
        stats: def.stats,
        skills: def.skills.clone(),
        weapon_skills: def.weapon_skills.clone(),
        effects: Default::default(),
        hp_base: hp,
        hp,
        level: world.state.current_level.clone(),
        ..template
    };
    world.state.actors.insert(id, actor);
    *world.perception =
        crate::perception::Perception::compute(&world.state, &world.content, &mut world.lights);
    id
}

#[test]
fn every_caster_has_its_table_by_level_sixty() {
    // (class, how many rows level 1 gives, every row by level 60.)
    let tables = [
        (
            "priest",
            2,
            vec![
                "Mend",
                "Orison",
                "Blessing",
                "Cure wounds",
                "Daily bread",
                "Consecrate",
                "Communion with the dark",
                "Renewal",
                "Silence",
                "Prayer",
                "Rebuke",
                "Prayer of release",
                "Benison",
                "Litany",
                "Litany of the mind",
                "Benediction",
                "Sanctuary",
                "Miracle",
            ],
        ),
        (
            "druid",
            2,
            vec![
                "Beast-friend",
                "Spark",
                "Barkskin",
                "Call of the wild",
                "Water walking",
                "Bramble",
                "Call lightning",
                "Rime",
                "Nature's ward",
                "Gale",
                "Call of the forest",
                "Cloudburst",
                "Pack leader",
                "Call the elements",
                "Flame and frost",
                "Lord of the wild",
            ],
        ),
        (
            "witch",
            4,
            vec![
                "Herb lore",
                "Evil eye",
                "Grave breath",
                "Familiar",
                "Hex of weakness",
                "Cackle",
                "Flay",
                "Poison lore",
                "Cause wounds",
                "Grave feast",
                "Malison",
                "Charnel burst",
                "Leech",
                "Second sight",
                "Familiar's sight",
                "Rouse the dead",
                "Curse of sloth",
                "Fear",
                "Blight",
                "Beguile",
                "Cauldron",
                "Hag's bargain",
            ],
        ),
    ];
    for (class, first, rows) in tables {
        let mut w = world(class);
        assert_eq!(class_rows(&w), rows[..first], "{class} at level 1");
        reach(&mut w, 60);
        assert_eq!(class_rows(&w), rows, "{class} at 60");
    }
    let mut w = world("wizard");
    let first = [
        "Scroll adept",
        "Ray of frost",
        "Burning hands",
        "Arcane training",
    ];
    assert_eq!(class_rows(&w), first);
    reach(&mut w, 12);
    assert_eq!(
        class_rows(&w),
        [
            "Scroll adept",
            "Ray of frost",
            "Burning hands",
            "Arcane training",
            "Magic missile",
            "Knock",
            "Blink",
            "Shatter",
            "Scribe"
        ]
    );
}

/// The wizard's and the witch's first harm (#979): known at level 1, 3 mana, and everyone in the
/// fan is hurt by the one roll, with no hit roll to miss; whoever stands aside is not.
#[test]
fn a_starting_caster_breathes_a_short_fan_that_needs_no_hit_roll() {
    for (class, spell) in [("wizard", "burning_hands"), ("witch", "grave_breath")] {
        for seed in 0..12 {
            let mut w = world(class);
            w.state.rng = crate::rng::Rng::gameplay(seed);
            let id = w.content().spell_id(spell).expect("the spell");
            assert!(w.state.player().knows_spell(w.content(), id), "{class}");
            let mine = kobold_at(&mut w, (1, 0), 100);
            let yours = kobold_at(&mut w, (2, 1), 100);
            let theirs = kobold_at(&mut w, (3, -1), 100);
            let aside = kobold_at(&mut w, (0, 2), 100);
            let here = w.player_pos();
            let mana = w.state.player().mana;
            let out = w.apply(&Action::Cast(
                spell.into(),
                Some(Pos::new(here.x + 3, here.y)),
            ));
            assert!(out.took_turn, "{class}: {:?}", said(&w));
            assert_eq!(mana - w.state.player().mana, 3, "{class}'s {spell}");
            let lost = |id: ActorId| 100 - w.state.actors[&id].hp;
            assert!(lost(mine) >= 1, "{class}: {:?}", said(&w));
            assert_eq!(lost(mine), lost(yours), "{class}: one roll for the fan");
            assert_eq!(lost(mine), lost(theirs), "{class}: one roll for the fan");
            assert_eq!(lost(aside), 0, "{class}: beside the fan");
        }
    }
}

/// The size of it (#979): a pool buys several casts, and at the same price Magic missile always
/// rolls more, a target at a time, and reaches further.
#[test]
fn the_first_fan_does_not_outclass_magic_missile() {
    let content = BUILTIN.clone();
    let rules = content.spell_rules().expect("the cost table");
    let def = |id: &str| content.spell(content.spell_id(id).expect("a spell"));
    let missile = def("magic_missile");
    for fan in ["burning_hands", "grave_breath"] {
        let fan = def(fan);
        assert_eq!(rules.mana(fan, 0), rules.mana(missile, 0), "{}", fan.id);
        assert!(i32::from(fan.level) <= 1, "{} from level 1", fan.id);
        assert!(fan.length.map(i32::from) < missile.range.map(i32::from));
        let rungs = |d: &crate::content::SpellData| {
            (0..=3)
                .map(|r| d.roll_at(r).expect("a roll").to_string())
                .collect::<Vec<_>>()
        };
        assert_eq!(rungs(fan), ["2d4", "2d6", "4d4", "4d4+2"], "{}", fan.id);
        assert_eq!(rungs(missile), ["2d4", "4d4", "6d4", "6d4+1"]);
    }
}

#[test]
fn the_spells_are_abilities_by_level() {
    let content = BUILTIN.clone();
    let every = Ability::every(&content);
    let spells: Vec<String> = (every.iter())
        .filter(|a| matches!(a, Ability::Spell(_)))
        .map(|a| a.name(&content).into_owned())
        .collect();
    assert_eq!(
        spells,
        [
            // Level 0.
            "Evil eye",
            "Light",
            // Level 1.
            "Burning hands",
            "Grave breath",
            "Mend",
            "Ray of frost",
            "Spark",
            // Level 3.
            "Barkskin",
            "Blessing",
            "Cure wounds",
            "Curse of sloth",
            "Hex of weakness",
            "Magic missile",
            // Level 4.
            "Cause wounds",
            "Nightsight",
            // Level 5.
            "Acid arrow",
            "Call of the wild",
            "Mesmerise",
            "Sense the quick",
            "Water walking",
            // Level 6.
            "Arcane bolt",
            "Blink",
            "Frost",
            "Grave feast",
            "Identify",
            "Sleep",
            "Slick",
            // Level 7.
            "Blight",
            "Brand",
            "Scorching ray",
            "Thunderclap",
            "Web",
            // Level 8.
            "Bramble",
            "Flay",
            "Glamour",
            "Shatter",
            "Unbinding",
            // Level 9.
            "Bind",
            "Charnel burst",
            "Rime",
            // Level 10.
            "Fear",
            "Leech",
            "Renewal",
            // Level 11.
            "Call lightning",
            "Fireball",
            "Gale",
            "Haste",
            "Lightning bolt",
            "Remove disease",
            "Silence",
            // Level 12.
            "Benison",
            "Fire cloak",
            "Flame",
            "Mirk",
            // Level 13.
            "Reek",
            // Level 14.
            "Grave spear",
            "Rebuke",
            // Level 15.
            "Slumber",
            // Level 16.
            "Balefire",
            "Beguile",
            // Level 20.
            "Call of the forest",
            "Chain lightning",
            "Cloudburst",
            // Level 24.
            "Rouse the dead",
            // Level 30.
            "Sunfire",
            // Level 35.
            "Call the elements",
        ]
    );
    let bolt = Ability::Spell(sp("arcane_bolt"));
    assert_eq!(bolt.source(), crate::ability::Source::Spell);
    assert_eq!(bolt.aim(&content), crate::ability::Aim::Cell);
    assert_eq!(bolt.reach(&content), Some(6));
    assert_eq!(Ability::Spell(sp("evil_eye")).reach(&content), Some(6));
    assert_eq!(Ability::Spell(sp("flame")).reach(&content), Some(10));
    let at = Pos::new(3, 4);
    assert_eq!(
        bolt.action_at(&content, at),
        Some(Action::Cast("arcane_bolt".into(), Some(at)))
    );
    let bless = Ability::Spell(sp("blessing"));
    assert_eq!(bless.aim(&content), crate::ability::Aim::Nothing);
    assert_eq!(
        bless.action(&content, None),
        Some(Action::Cast("blessing".into(), None))
    );
    assert_eq!(bless.cost(), Some(crate::content::ActionKind::Cast));
}

#[test]
fn blessing_costs_mana_and_wants_more_than_there_is() {
    let mut w = world("priest");
    assert_eq!(
        own(&w, sp("blessing")),
        None,
        "a priest blesses from level 3"
    );
    reach(&mut w, 3);
    let player = w.state.player;
    assert_eq!(own(&w, sp("blessing")), Some(None));
    let time = w.time();
    run(&mut w, "use:blessing");
    assert_eq!(w.state.player().mana, 37, "level 3's 3");
    assert!(suffers(&w, player, "blessed"));
    assert!(said(&w).contains(&"You feel blessed.".to_string()));
    assert!(w.time() > time, "it took time");
    // Blessed: +1 to hit and Willpower 1 higher.
    let me = w.state.player();
    assert_eq!(me.effective_stats(w.content()).score(Stat::Willpower), 15);
    // Without the mana: greyed out, and refused with no time.
    w.player_mut().mana = 0;
    assert_eq!(own(&w, sp("blessing")), Some(Some(Unavailable::NoMana)));
    assert_eq!(
        Unavailable::NoMana.reason(w.content()),
        "you have too little mana"
    );
    let turn = w.state.turn;
    run(&mut w, "use:blessing");
    assert_eq!(last(&w), "You have too little mana.");
    assert_eq!((w.state.turn, w.state.player().mana), (turn, 0));
    // A spell the class does not give.
    run(&mut w, "use:arcane_bolt:12,5");
    assert_eq!(last(&w), "You have no such ability.");
}

#[test]
fn cure_wounds_heals_by_touch_in_two_casts() {
    let mut w = world("priest");
    reach(&mut w, 3);
    let turn = w.state.turn;
    run(&mut w, "use:cure_wounds");
    assert_eq!(
        last(&w),
        "Cast it where? Name a cell: `use:cure_wounds:12,5`."
    );
    let here = w.player_pos();
    let on_me = format!("use:cure_wounds:{},{}", here.x, here.y);
    run(&mut w, &on_me);
    assert_eq!(last(&w), "You are not hurt.");
    assert_eq!((w.state.turn, w.state.player().mana), (turn, 40));
    w.player_mut().hp = 1;
    let time = w.time();
    run(&mut w, &on_me);
    assert_eq!(last(&w), "Your wounds close.");
    let back = w.state.player().hp - 1;
    // 2d6 (level 3, a touch: 3 × 0.7 mana).
    assert!((2..=12).contains(&back), "{back}");
    assert_eq!(w.state.player().mana, 38);
    let cure = w.time().seconds() - time.seconds();
    let time = w.time();
    run(&mut w, "use:blessing");
    let bless = w.time().seconds() - time.seconds();
    assert!(cure + 1 >= 2 * bless, "two casts: {cure} s, one {bless} s");
}

#[test]
fn remove_disease_ends_diseases_and_poison() {
    let mut w = world("priest");
    reach(&mut w, 12);
    // The priest's table no longer gives it (the prayer does its work); its scroll teaches it.
    w.player_mut().spells.insert(sp("remove_disease"));
    run(&mut w, "use:remove_disease");
    assert_eq!(last(&w), "There is nothing to cure.");
    assert_eq!(w.state.player().mana, 40);
    run(&mut w, "infect:the_shakes afflict:poisoned*2");
    assert!(!w.state.player().diseases.is_empty());
    run(&mut w, "use:remove_disease");
    let me = w.state.player();
    assert!(me.diseases.is_empty());
    assert!(!suffers(&w, w.state.player, "poisoned"));
    assert_eq!(w.state.player().mana, 29, "level 11's 11");
    assert!(said(&w).contains(&"A cleansing warmth runs through you.".to_string()));
}

#[test]
fn hexes_fall_on_a_foe_the_witch_sees_within_reach() {
    let mut w = world("witch");
    reach(&mut w, 32);
    let near = kobold(&mut w, 3, 50);
    let here = w.player_pos();
    let cell = |dx: i32| Pos::new(here.x + dx, here.y);
    // The kobold is put back where it stood before each cast (it moves between turns).
    let cast = |w: &mut World, spell: &str, at: Pos| {
        w.state.actors.get_mut(&near).expect("the kobold").pos = cell(3);
        *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
        w.apply(&Action::Cast(spell.into(), Some(at)));
    };
    // No one there, or too far: refused with no time and no mana.
    let turn = w.state.turn;
    cast(&mut w, "evil_eye", cell(2));
    assert_eq!(last(&w), "You see no one there to hex.");
    cast(&mut w, "evil_eye", cell(9));
    assert_eq!(last(&w), "That is out of range.");
    run(&mut w, "use:evil_eye");
    assert_eq!(last(&w), "Cast it where? Name a cell: `use:evil_eye:12,5`.");
    assert_eq!((w.state.turn, w.state.player().mana), (turn, 40));
    cast(&mut w, "evil_eye", cell(3));
    assert_eq!(last(&w), "The kobold is hexed.");
    assert!(suffers(&w, near, "ill_wished"));
    assert_eq!(w.state.player().mana, 39, "level 0 or 1: 1");
    cast(&mut w, "hex_of_weakness", cell(3));
    assert!(suffers(&w, near, "weakened"), "{:?}", said(&w));
    cast(&mut w, "curse_of_sloth", cell(3));
    assert!(suffers(&w, near, "sluggish"));
    assert!(w.state.player().talents.has("frugal_caster"));
    // Level 3's 3 each, less Frugal caster's fifth (`levelup:auto` took it).
    assert_eq!(w.state.player().mana, 35, "2 each");
    // Presence 14 (+2): one degree more of each hex, so it lasts twice as long.
    let sluggish = status(&w, "sluggish");
    assert_eq!(
        w.state.actors[&near]
            .effects
            .get(sluggish)
            .map(|e| e.severity),
        Some(2)
    );
    let content = w.content().clone();
    let effects = &w.state.actors[&near].effects;
    assert_eq!(
        effects.slow(&content, 100),
        150,
        "sluggish: half again as long"
    );
}

#[test]
fn bolts_and_bursts_strike_as_scrolls_do_and_cost_mana() {
    let mut w = world("wizard");
    reach(&mut w, 18);
    // `levelup:auto` takes Frugal caster at 18 (it took Attuned before it went); these tests
    // read the plain costs.
    w.player_mut().talents.forget("frugal_caster");
    let target = kobold(&mut w, 2, 500);
    let here = w.player_pos();
    let at = Pos::new(here.x + 2, here.y);
    w.apply(&Action::Cast("arcane_bolt".into(), Some(at)));
    let hurt = 500 - w.state.actors[&target].hp;
    assert!((4..=24).contains(&hurt), "4d6: {hurt}");
    assert_eq!(w.state.player().mana, 29, "level 6 × 1.8 for a line");
    assert!(
        said(&w).iter().any(|m| m.contains("arcane bolt")),
        "{:?}",
        said(&w)
    );
    // Fireball, learned, at 40: a burst.
    reach(&mut w, 40);
    w.player_mut().spells.insert(sp("fireball"));
    let far = Pos::new(here.x + 2, here.y);
    let before = w.state.actors[&target].hp;
    w.apply(&Action::Cast("fireball".into(), Some(far)));
    assert!(w.state.actors[&target].hp < before);
    assert!(w.state.player().talents.has("frugal_caster"));
    // Level 11 × 2.4 for a burst of radius 2, less Empower's tenth (23), less Frugal caster's
    // fifth (18; `levelup:auto` took it).
    assert_eq!(w.state.player().mana, 22);
}

#[test]
fn call_lightning_wants_the_open_sky() {
    let mut w = world("druid");
    reach(&mut w, 12);
    let target = kobold(&mut w, 2, 500);
    let here = w.player_pos();
    let at = Pos::new(here.x + 2, here.y);
    let turn = w.state.turn;
    w.apply(&Action::Cast("call_lightning".into(), Some(at)));
    assert_eq!(last(&w), "You need the open sky above you for that.");
    assert_eq!((w.state.turn, w.state.player().mana), (turn, 40));
    w.state.current_mut().kind = LevelKind::Outdoor;
    let floor = w.content().terrain_id("floor").expect("floor");
    w.state.current_mut().terrain.set(here, floor);
    w.apply(&Action::Cast("call_lightning".into(), Some(at)));
    let hurt = 500 - w.state.actors[&target].hp;
    assert!(hurt >= 1, "2d6, less what it resists: {hurt}");
    assert_eq!(w.state.player().mana, 20, "level 11 × 1.8");
}

/// The player's allies, by id.
fn pack(world: &World) -> Vec<&Actor> {
    (world.state.actors.values())
        .filter(|a| a.master() == Some(world.state.player))
        .collect()
}

/// How many of the player's allies are of creature type `id`.
fn of_kind(world: &World, id: &str) -> usize {
    (pack(world).into_iter())
        .filter(|a| world.content().creature(a.creature).id == id)
        .count()
}

#[test]
fn the_druids_calls_bring_one_of_each_until_its_abilities_raise_that() {
    let mut w = world("druid");
    assert_eq!(
        own(&w, sp("call_of_the_wild")),
        None,
        "a druid calls from 6"
    );
    reach(&mut w, 6);
    assert_eq!(own(&w, sp("call_of_the_wild")), Some(None));
    run(&mut w, "use:call_of_the_wild");
    assert!(
        said(&w)
            .iter()
            .any(|m| m == "You call, and the wild answers.")
    );
    assert_eq!(last(&w), "A wolf comes to your call.");
    let first = pack(&w)[0].id;
    assert_eq!(of_kind(&w, "wolf"), 1);
    assert!(pack(&w)[0].ally.as_ref().expect("an ally").until.is_some());
    // One at a time: a second call, and the first gives way.
    w.player_mut().mana = 40;
    run(&mut w, "use:call_of_the_wild");
    assert_eq!(of_kind(&w, "wolf"), 1);
    assert!(!w.state.actors.contains_key(&first));
    // A bear beside the wolf: the cap is each call's.
    reach(&mut w, 25);
    run(&mut w, "use:call_of_the_forest");
    assert_eq!((of_kind(&w, "wolf"), of_kind(&w, "bear")), (1, 1));
    // Pack leader: two at once, both from one call.
    reach(&mut w, 32);
    run(&mut w, "use:call_of_the_wild");
    assert_eq!(of_kind(&w, "wolf"), 2);
    // Greater elementals of the elements the wild chooses, two with Pack leader.
    reach(&mut w, 40);
    w.player_mut().mana = 100;
    run(&mut w, "use:call_the_elements");
    let elementals = (pack(&w).into_iter())
        .filter(|a| w.content().has_tag(a.creature, "elemental"))
        .count();
    assert_eq!(elementals, 2);
    // Lord of the wild: three at once, and they stay until slain.
    reach(&mut w, 60);
    run(&mut w, "use:call_of_the_wild");
    assert_eq!(of_kind(&w, "wolf"), 3);
    assert!(
        (pack(&w).iter()).any(|a| a.ally.as_ref().is_some_and(|x| x.until.is_none())),
        "a lasting call"
    );
}

#[test]
fn a_called_wolf_fights_as_yours_and_slips_back_into_the_dark() {
    let mut w = world("druid");
    reach(&mut w, 6);
    let foe = kobold(&mut w, 3, 500);
    run(&mut w, "use:call_of_the_wild wait*6");
    assert!(
        said(&w).iter().any(|m| m.starts_with("Your wolf ")),
        "{:?}",
        said(&w)
    );
    w.state.actors.remove(&foe);
    // Presence 14: eight minutes; then it goes, said by its type's own words.
    run(&mut w, "wait*90");
    assert_eq!(of_kind(&w, "wolf"), 0);
    assert!(
        said(&w)
            .iter()
            .any(|m| m == "The wolf slips back into the dark."),
        "{:?}",
        said(&w)
    );
}

#[test]
fn blight_poisons_everyone_it_reaches() {
    let mut w = world("witch");
    reach(&mut w, 40);
    let target = kobold(&mut w, 3, 500);
    let here = w.player_pos();
    w.apply(&Action::Cast(
        "blight".into(),
        Some(Pos::new(here.x + 3, here.y)),
    ));
    assert!(suffers(&w, target, "poisoned"));
    assert!(w.state.player().talents.has("frugal_caster"));
    // Level 7 × 2.4 (17), less Frugal caster's fifth (`levelup:auto` took it).
    assert_eq!(w.state.player().mana, 26, "14");
}

#[test]
fn the_rung_talents_and_archmage_mana() {
    let content = BUILTIN.clone();
    let book = content.talents();
    let mods = |ids: &[&str]| {
        let effects = ids.iter().flat_map(|id| {
            let def = book.get(book.id(id).expect("a talent"));
            def.effects.clone()
        });
        crate::talent::TalentMods::of(&content, effects)
    };
    let every = |ids: &[&str]| mods(ids).spell_mods(&["evocation", "damage", "burst", "fire"]);
    assert_eq!(every(&[]), crate::content::SpellMods::default());
    assert_eq!(
        (
            every(&["scroll_adept"]).scroll_rungs,
            every(&["scroll_adept"]).rungs
        ),
        (1, 0)
    );
    assert_eq!(every(&["empower"]).rungs, 1);
    assert_eq!(every(&["scroll_adept", "empower", "archmage"]).rungs, 3);
    let all = mods(&["scroll_adept", "empower", "archmage"]);
    // Archmage's half and Empower's tenth off.
    assert_eq!((all.spell_mana(10), all.spell_mana(1)), (4, 1));
    assert_eq!(mods(&[]).spell_mana(9), 9);
}

#[test]
fn a_spell_is_paid_through_the_talents_mana_cost() {
    let mut w = world("priest");
    reach(&mut w, 12);
    let content = w.content().clone();
    w.player_mut().spells.insert(sp("remove_disease"));
    w.player_mut().talents.take("frugal_caster", None);
    let cleanse = content
        .class_spell(sp("remove_disease"))
        .expect("remove disease");
    assert_eq!(
        w.state.player().spell_cost(&content, cleanse),
        9,
        "11, a fifth less"
    );
    run(&mut w, "afflict:poisoned use:remove_disease");
    assert_eq!(w.state.player().mana, 31);
    // One mana short of the cut cost: greyed out.
    w.player_mut().mana = 8;
    assert_eq!(
        own(&w, sp("remove_disease")),
        Some(Some(Unavailable::NoMana))
    );
}

#[test]
fn mage_armour_deflects_only_without_body_armour() {
    let mut w = world("wizard");
    let content = w.content().clone();
    let deflection = |w: &World| {
        let me = w.state.player();
        let d = me.defence(&content, crate::burden::Burden::Unburdened, None);
        d.bonus(crate::combat::Category::Deflection)
    };
    let before = deflection(&w);
    reach(&mut w, 25);
    assert_eq!(deflection(&w), before + 2, "a wizard wears no body armour");
}

#[test]
fn the_hags_bargain_costs_presence_for_good() {
    let mut w = world("witch");
    reach(&mut w, 60);
    let player = w.state.player;
    let presence = w.state.player().stats.score(Stat::Presence);
    let mut events = Vec::new();
    assert!(!w.hurt(player, 999, fight::Harm::Barbs, &mut events));
    assert_eq!(w.state.player().hp, 1);
    assert_eq!(w.state.player().stats.score(Stat::Presence), presence - 2);
}

#[test]
fn poison_lore_hurts_the_poisoned_more() {
    let mut w = world("witch");
    reach(&mut w, 12);
    let target = kobold(&mut w, 1, 500);
    let player = w.state.player;
    let hit = crate::combat::Blow::plain(5, None);
    assert_eq!(w.lore_blow(player, target, hit).damage, 5);
    let poisoned = status(&w, "poisoned");
    let mut events = Vec::new();
    w.afflict(target, poisoned, 1, None, &mut events);
    assert_eq!(w.lore_blow(player, target, hit).damage, 7);
}

#[test]
fn casts_replay_and_save_the_same() {
    let spec: CharacterSpec = "human,female,priest".parse().expect("a character");
    let mut w = World::start_run(
        BUILTIN.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let mut replay = Replay::new(&w);
    for action in parse_actions("use:blessing wait*3 xp:800 levelup:auto drain:1 use:blessing")
        .expect("actions")
    {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(BUILTIN.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, BUILTIN.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    let text = Action::Cast("frost".into(), Some(Pos::new(4, 7))).to_string();
    assert_eq!(text, "use:frost:4,7");
    assert_eq!(
        text.parse::<Action>(),
        Ok(Action::Cast("frost".into(), Some(Pos::new(4, 7))))
    );
}
