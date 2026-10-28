//! Active weapon abilities in the world (the combat progression design §4.5, §4.6, §6; #564):
//! each of the twelve on a hand-made fight, their costs (stamina spent after the refusals, time
//! at the energy percent), the rows greyed short of stamina or not so held, Practised technique
//! and Drillmaster, and the pool granted with the first active a rank teaches.

use super::weapon_active::Aimed;
use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::AttackKind;
use crate::combat::{Category, Defence, Strike};
use crate::content::{ContentFiles, STAMINA};
use crate::equipment;
use crate::weapon_skills::WeaponSkills;

// Sure weapons of every kind the actives ask for, a shield, a tower of a shield, and a bow.
const OBJECTS: &str = r##"
[[object]]
id = "wt_knife"
name = "test knife"
glyph = ")"
fg = "#c0c0c0"
class = "melee_weapons"
materials = ["steel"]
weight = 300
value = 0
slot = "main_hand"
light_weapon = true
damage = "1d2"
to_hit = 20
category = "long_blades"
damage_type = "slashing"

[[object]]
id = "wt_spear"
name = "test spear"
glyph = "/"
fg = "#c0c0c0"
class = "melee_weapons"
materials = ["steel"]
weight = 1000
value = 0
slot = "main_hand"
damage = "1d2"
to_hit = 20
category = "spears"
damage_type = "piercing"

[[object]]
id = "wt_axe"
name = "test axe"
glyph = ")"
fg = "#c0c0c0"
class = "melee_weapons"
materials = ["steel"]
weight = 1000
value = 0
slot = "main_hand"
damage = "1d2+19"
to_hit = 20
category = "axes"
damage_type = "slashing"

[[object]]
id = "wt_greatsword"
name = "test greatsword"
glyph = ")"
fg = "#c0c0c0"
class = "melee_weapons"
materials = ["steel"]
weight = 3000
value = 0
slot = "main_hand"
two_handed = true
damage = "20d2"
to_hit = 20
category = "great_blades"
damage_type = "slashing"

[[object]]
id = "wt_staff"
name = "test staff"
glyph = "/"
fg = "#c0c0c0"
class = "melee_weapons"
materials = ["wood"]
weight = 1500
value = 0
slot = "main_hand"
two_handed = true
damage = "1d2"
to_hit = 20
category = "staves"
damage_type = "bludgeoning"

[[object]]
id = "wt_shield"
name = "test shield"
glyph = "["
fg = "#c0c0c0"
class = "shields"
materials = ["wood"]
weight = 2500
value = 0
slot = "off_hand"
shield = 2
bash = "1d2"

[[object]]
id = "wt_tower"
name = "test tower"
glyph = "["
fg = "#c0c0c0"
class = "shields"
materials = ["wood"]
weight = 2500
value = 0
slot = "off_hand"
shield = 20

[[object]]
id = "wt_bow"
name = "test bow"
glyph = "}"
fg = "#c0c0c0"
class = "missile_weapons"
materials = ["wood"]
weight = 500
value = 0
slot = "missile_weapon"
launches = "arrow"
damage = "1d2+19"
to_hit = 20
category = "bows"
damage_type = "piercing"
range = 10
"##;

// Posts that only stand: plain, armoured, behind a tower of a shield; and a peaceful one.
const CREATURES: &str = r##"
[[creature]]
id = "wt_post"
name = "post"
glyph = "p"
fg = "#806040"
weight = 60000
faction = "vermin"
speed = 100
base_hp = 1000

[[creature]]
id = "wt_plate"
name = "plated post"
glyph = "P"
fg = "#806040"
weight = 60000
faction = "vermin"
speed = 100
base_hp = 1000
natural = { defence = 0, protection = 8, name = "iron plates" }

[[creature]]
id = "wt_guard"
name = "guard"
glyph = "g"
fg = "#806040"
weight = 60000
faction = "vermin"
speed = 100
base_hp = 1000

[[creature]]
id = "wt_friend"
name = "friend"
glyph = "f"
fg = "#806040"
weight = 60000
faction = "peaceful"
speed = 100
base_hp = 1000
"##;

fn level(map: &str) -> String {
    format!(
        r##"name = "Yard"
ambient = [255, 255, 255, 0]
map = '''
{map}'''
[legend]
"@" = {{ terrain = "floor", spawn = "player" }}
"p" = {{ terrain = "floor", creature = "wt_post" }}
"P" = {{ terrain = "floor", creature = "wt_plate" }}
"g" = {{ terrain = "floor", creature = "wt_guard", carry = ["wt_tower"], wield = "wt_tower" }}
"f" = {{ terrain = "floor", creature = "wt_friend" }}
"+" = {{ terrain = "door_closed" }}
"##
    )
}

/// A world on `map` where nobody but the player acts, and the player holds stamina.
fn world(map: &str) -> World {
    let mut files = ContentFiles::builtin();
    files.insert("data/objects/zz_wt_test.toml", OBJECTS);
    files.insert("data/creatures/zz_wt_test.toml", CREATURES);
    files.insert("levels/wt_yard.toml", &level(map));
    files.insert(
        "scenarios/wt_yard.toml",
        // Toughness 14: a pool of 12, so a refill to 12 is within the maximum (the pool is kept
        // within it after every action, `keep_resources_within_max`).
        "name = \"Yard\"\nstart_level = \"wt_yard\"\nhp = 1000\nstats = { toughness = 14 }\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let mut w = World::new(content, "wt_yard", None).expect("world");
    let player = w.player_id();
    for actor in w.state.actors.values_mut() {
        if actor.id != player {
            actor.energy = -1_000_000_000;
        }
    }
    let stamina = w.content().resource_id(STAMINA).expect("stamina");
    w.grant_resource(player, stamina);
    refill(&mut w);
    w
}

/// The player's stamina is 12 again.
fn refill(w: &mut World) {
    let stamina = w.content().resource_id(STAMINA).expect("stamina");
    w.player_mut()
        .resources
        .get_mut(stamina)
        .expect("the pool")
        .value = 12;
}

/// The player holds `object` in a free slot it fits.
fn give(w: &mut World, object: &str) {
    let content = w.content().clone();
    let me = w.player_mut();
    let object = content.object_id(object).expect("an object");
    me.inventory.push(crate::object::Stack::new(object, 1));
    let item = me.inventory.len() - 1;
    let slot = equipment::free_slot(&content, &me.equipment, object).expect("a free slot");
    me.equipment
        .put(&equipment::slots_taken(&content, object, slot), item);
}

/// The player carries `count` arrows, readied.
fn arrows(w: &mut World, count: u32) {
    let content = w.content().clone();
    let arrow = content.object_id("arrow").expect("arrow");
    let me = w.player_mut();
    me.inventory.push(crate::object::Stack::new(arrow, count));
    let item = me.inventory.len() - 1;
    let slot = equipment::free_slot(&content, &me.equipment, arrow).expect("a quiver");
    me.equipment
        .put(&equipment::slots_taken(&content, arrow, slot), item);
}

/// The player's ranks are exactly `ranks`.
fn ranks(w: &mut World, ranks: &[(&str, u8)]) {
    let content = w.content().clone();
    let table = content.weapon_skills();
    let ranks = ranks.iter().map(|&(s, r)| (table.id(s).expect(s), r));
    w.player_mut().weapon_skills = WeaponSkills::from_ranks(table, ranks);
}

fn at(w: &World, x: i32, y: i32) -> ActorId {
    let level = &w.state.current().id;
    w.state.actor_at(level, Pos::new(x, y)).expect("someone").id
}

/// The hit points actor `id` has lost.
fn lost_hp(w: &World, id: ActorId) -> i32 {
    let actor = &w.state.actors[&id];
    actor.max_hp(w.content()) - actor.hp
}

fn stamina(w: &World) -> u16 {
    let id = w.content().resource_id(STAMINA).expect("stamina");
    w.state.player().resources.get(id).map_or(0, |p| p.value)
}

fn said(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

fn use_(w: &mut World, text: &str) -> Outcome {
    w.apply(&text.parse::<Action>().expect("action text"))
}

/// The player uses `name` aimed `aim` straight through the rules: what it cost.
fn spent(w: &mut World, name: &str, aim: Aimed) -> Spent {
    let mut events = Vec::new();
    w.use_weapon(name, aim, &mut events)
}

/// What a plain blow with what the player holds costs them now.
fn blow_price(w: &World) -> i64 {
    let me = w.state.player();
    let skills = me.attack_skills(w.content(), AttackKind::Melee, None);
    let with = me.weapon_item(w.content());
    w.attack_price(w.player_id(), ActionKind::Melee, with, skills)
}

fn strikes(outcome: &Outcome, kind: AttackKind) -> Vec<Option<Strike>> {
    (outcome.events.iter())
        .filter_map(|e| match e {
            Event::Attacked {
                kind: k, strike, ..
            } if *k == kind => Some(*strike),
            _ => None,
        })
        .collect()
}

#[test]
fn lunge_reaches_over_an_empty_pace_and_is_refused_over_a_door_or_a_friend() {
    let mut w = world("######\n#@.p.#\n######\n");
    give(&mut w, "wt_spear");
    ranks(&mut w, &[("spears", 9)]);
    let post = at(&w, 3, 1);
    let price = w.active_price(
        w.content().weapon_ability_id("lunge").unwrap(),
        blow_price(&w),
    );
    assert_eq!(price, blow_price(&w), "a lunge at 100 % is a blow's time");
    assert_eq!(
        spent(&mut w, "lunge", Aimed::Dir(Dir::E)),
        Spent::Turn(price)
    );
    assert_eq!(stamina(&w), 12 - 3, "{}", stamina(&w));
    assert!(lost_hp(&w, post) > 0, "{:#?}", said(&w));
    // Refused, costing nothing, over a closed door, a friend, and with nobody in reach.
    for map in [
        "######\n#@+p.#\n######\n",
        "######\n#@fp.#\n######\n",
        "######\n#@..p#\n######\n",
    ] {
        let mut w = world(map);
        give(&mut w, "wt_spear");
        ranks(&mut w, &[("spears", 9)]);
        let before = (stamina(&w), w.state_hash());
        let outcome = use_(&mut w, "use:lunge:E");
        assert!(!outcome.took_turn, "{map}: {:#?}", said(&w));
        assert_eq!(stamina(&w), before.0, "{map}");
    }
    // A foe right beside is struck as any blow.
    let mut w = world("######\n#@p..#\n######\n");
    give(&mut w, "wt_spear");
    ranks(&mut w, &[("spears", 9)]);
    let outcome = use_(&mut w, "use:lunge:E");
    assert_eq!(strikes(&outcome, AttackKind::Melee).len(), 1);
}

#[test]
fn piercing_shot_flies_on_to_the_next() {
    let mut pierced = false;
    for _ in 0..8 {
        let mut w = world("#########\n#@.p.p..#\n#########\n");
        give(&mut w, "wt_bow");
        arrows(&mut w, 5);
        ranks(&mut w, &[("bows", 9)]);
        let outcome = use_(&mut w, "use:piercing_shot:3,1");
        assert!(outcome.took_turn, "{:#?}", said(&w));
        assert_eq!(stamina(&w), 12 - 4);
        let shots = strikes(&outcome, AttackKind::Ranged);
        if shots.first().is_some_and(|s| s.is_some_and(|s| s.is_hit())) {
            assert_eq!(shots.len(), 2, "on to the second: {:#?}", said(&w));
            pierced = true;
            break;
        }
    }
    assert!(pierced);
}

#[test]
fn sunder_and_gap_shot_are_not_soaked() {
    // The axe does 20 a hit; the plates soak 4 to 8 of it, but not of a sunder.
    let mut w = world("#####\n#@P.#\n#####\n");
    give(&mut w, "wt_axe");
    ranks(&mut w, &[("axes", 9)]);
    let plate = at(&w, 2, 1);
    let outcome = use_(&mut w, "use:sunder:E");
    let hit = strikes(&outcome, AttackKind::Melee)[0].expect("a roll");
    if hit.is_hit() {
        assert!(lost_hp(&w, plate) >= 20, "{}", lost_hp(&w, plate));
    }
    let lost = |sunder: bool| {
        let mut w = world("#####\n#@P.#\n#####\n");
        give(&mut w, "wt_axe");
        ranks(&mut w, &[("axes", 9)]);
        let plate = at(&w, 2, 1);
        if sunder {
            use_(&mut w, "use:sunder:E");
        } else {
            use_(&mut w, "move:E");
        }
        lost_hp(&w, plate)
    };
    assert!(lost(true) > lost(false), "{} {}", lost(true), lost(false));
    // Gap shot: the bow's 20 unsoaked.
    let mut w = world("#######\n#@.P..#\n#######\n");
    give(&mut w, "wt_bow");
    arrows(&mut w, 5);
    ranks(&mut w, &[("bows", 1), ("marksman", 7)]);
    let plate = at(&w, 3, 1);
    let outcome = use_(&mut w, "use:gap_shot:3,1");
    assert_eq!(stamina(&w), 12 - 4);
    if strikes(&outcome, AttackKind::Ranged)[0].is_some_and(|s| s.is_hit()) {
        assert!(lost_hp(&w, plate) >= 20);
    }
}

#[test]
fn shield_wall_costs_a_wait_and_walls() {
    let mut w = world("#####\n#@p.#\n#####\n");
    give(&mut w, "wt_knife");
    give(&mut w, "wt_shield");
    ranks(&mut w, &[("long_blades", 1), ("shield_block", 9)]);
    let player = w.player_id();
    let before = w.defence_of(player).bonus(Category::Shield);
    let attack = blow_price(&w);
    let wait = w.price_of(player, ActionKind::Wait);
    assert_eq!(
        spent(&mut w, "shield_wall", Aimed::Nothing),
        Spent::Turn(wait)
    );
    assert_eq!(stamina(&w), 12 - 5);
    let walled = w.content().status_id("walled").expect("walled");
    assert_eq!(
        w.state.player().effects.get(walled).map(|e| e.severity),
        Some(5)
    );
    assert_eq!(w.defence_of(player).bonus(Category::Shield), before + 3);
    assert_eq!(
        blow_price(&w),
        (attack * 125 + 99) / 100,
        "attacks a quarter slower"
    );
    // Without a shield it is not so held.
    let mut w = world("#####\n#@p.#\n#####\n");
    give(&mut w, "wt_knife");
    ranks(&mut w, &[("long_blades", 1), ("shield_block", 9)]);
    let outcome = use_(&mut w, "use:shield_wall");
    assert!(!outcome.took_turn);
    assert!(
        said(&w).iter().any(|m| m == "Your hands are not so held."),
        "{:#?}",
        said(&w)
    );
}

#[test]
fn whirl_and_staff_sweep_strike_all_around() {
    let map = "#####\n#ppp#\n#p@p#\n#ppp#\n#####\n";
    let mut w = world(map);
    give(&mut w, "wt_knife");
    give(&mut w, "wt_knife");
    ranks(&mut w, &[("long_blades", 1), ("twin_blades", 7)]);
    let base = blow_price(&w);
    let id = w.content().weapon_ability_id("whirl").unwrap();
    let mut events = Vec::new();
    let spent = w.use_weapon("whirl", Aimed::Nothing, &mut events);
    assert_eq!(spent, Spent::Turn(w.active_price(id, base)));
    assert_eq!(w.active_price(id, base), base * 3 / 2);
    let blows = (events.iter())
        .filter(|e| {
            matches!(
                e,
                Event::Attacked {
                    kind: AttackKind::Melee,
                    ..
                }
            )
        })
        .count();
    assert!(
        blows >= 8,
        "a blow at each of the eight, with what they bring: {blows}"
    );
    assert_eq!(stamina(&w), 12 - 6);
    // Staff sweep.
    let mut w = world(map);
    give(&mut w, "wt_staff");
    ranks(&mut w, &[("staves", 1), ("staff_fighter", 7)]);
    let outcome = use_(&mut w, "use:staff_sweep");
    assert!(strikes(&outcome, AttackKind::Melee).len() >= 8);
    assert_eq!(stamina(&w), 12 - 5);
    // Nobody beside: refused.
    let mut w = world("#####\n#.@.#\n#####\n");
    give(&mut w, "wt_staff");
    ranks(&mut w, &[("staves", 1), ("staff_fighter", 7)]);
    assert!(!use_(&mut w, "use:staff_sweep").took_turn);
    assert_eq!(stamina(&w), 12);
}

#[test]
fn shield_rush_and_hip_throw_knock_back_but_not_into_a_wall() {
    let rush = |map: &str, hip: bool| {
        let mut w = world(map);
        if hip {
            ranks(&mut w, &[("unarmed", 1), ("brawler", 7)]);
        } else {
            give(&mut w, "wt_knife");
            give(&mut w, "wt_shield");
            ranks(&mut w, &[("long_blades", 1), ("sword_and_board", 7)]);
        }
        let post = at(&w, 2, 1);
        let text = if hip {
            "use:hip_throw:E"
        } else {
            "use:shield_rush:E"
        };
        for _ in 0..10 {
            let outcome = use_(&mut w, text);
            if strikes(&outcome, AttackKind::Melee)[0].is_some_and(|s| s.is_hit()) {
                break;
            }
            refill(&mut w);
        }
        let staggered = w.content().status_id("staggered").expect("staggered");
        let post = &w.state.actors[&post];
        (post.pos.x, post.effects.has(staggered))
    };
    for hip in [false, true] {
        assert_eq!(
            rush("######\n#@p..#\n######\n", hip),
            (3, true),
            "hip {hip}"
        );
        assert_eq!(
            rush("####\n#@p#\n####\n", hip),
            (2, false),
            "hip {hip}: a wall behind"
        );
    }
}

#[test]
fn feint_rolls_without_parry_and_shield() {
    let mut w = world("#####\n#@g.#\n#####\n");
    let guard = at(&w, 2, 1);
    let mut target = w.target_of(guard);
    target.defence = Defence::from_bonuses([
        (Category::Shield, 20),
        (Category::Parry, 5),
        (Category::Armour, 3),
    ]);
    w.technique.ignores = crate::content::Categories::default()
        .with(Category::Parry)
        .with(Category::Shield);
    let player = w.player_id();
    let feinted = w.feinted(player, target.clone());
    assert_eq!(feinted.defence.bonus(Category::Shield), 0);
    assert_eq!(feinted.defence.bonus(Category::Parry), 0);
    assert_eq!(feinted.defence.bonus(Category::Armour), 3);
    // Taken by that one attack.
    assert_eq!(w.feinted(player, target.clone()), target);
    // Against the guard's tower, feints land where blows do not.
    let count = |feint: bool| {
        let mut w = world("#####\n#@g.#\n#####\n");
        give(&mut w, "wt_knife");
        ranks(&mut w, &[("long_blades", 1), ("duelist", 7)]);
        let mut hits = 0;
        for _ in 0..12 {
            let outcome = use_(&mut w, if feint { "use:feint:E" } else { "move:E" });
            hits += (strikes(&outcome, AttackKind::Melee).first())
                .is_some_and(|s| s.is_some_and(|s| s.is_hit())) as u32;
            refill(&mut w);
        }
        hits
    };
    assert!(
        count(true) > count(false),
        "{} {}",
        count(true),
        count(false)
    );
}

#[test]
fn vital_strike_hits_are_criticals_and_mighty_swing_rolls_twice() {
    let mut w = world("#####\n#@p.#\n#####\n");
    give(&mut w, "wt_knife");
    ranks(&mut w, &[("long_blades", 9)]);
    let outcome = use_(&mut w, "use:vital_strike:E");
    let strike = strikes(&outcome, AttackKind::Melee)[0].expect("rolled");
    assert!(
        !strike.is_hit() || strike == Strike::Hit { critical: true },
        "{strike:?}"
    );
    assert_eq!(stamina(&w), 12 - 4);
    // The greatsword does 20d2 (and Strength's bonus): twice the dice, 40d2, is more.
    let lost = |mighty: bool| {
        let mut w = world("#####\n#@p.#\n#####\n");
        give(&mut w, "wt_greatsword");
        ranks(&mut w, &[("great_blades", 1), ("great_weapon", 7)]);
        let post = at(&w, 2, 1);
        for _ in 0..10 {
            let outcome = use_(
                &mut w,
                if mighty {
                    "use:mighty_swing:E"
                } else {
                    "move:E"
                },
            );
            if let Some(Some(Strike::Hit { critical: false })) =
                strikes(&outcome, AttackKind::Melee).first()
            {
                return lost_hp(&w, post);
            }
            refill(&mut w);
        }
        panic!("no plain hit");
    };
    assert!(lost(true) > lost(false), "{} {}", lost(true), lost(false));
}

#[test]
fn a_short_pool_and_the_wrong_grip_grey_the_row_and_refuse() {
    let mut w = world("######\n#@.p.#\n######\n");
    give(&mut w, "wt_spear");
    ranks(&mut w, &[("spears", 9)]);
    let content = w.content().clone();
    let lunge = content.weapon_ability_id("lunge").unwrap();
    let row = |w: &World| {
        (w.perception().abilities().into_iter())
            .find(|a| matches!(a.ability, Ability::Weapon(_, id) if id == lunge))
            .expect("the lunge's row")
    };
    assert_eq!(row(&w).unavailable, None);
    let stamina_id = content.resource_id(STAMINA).unwrap();
    w.player_mut().resources.get_mut(stamina_id).unwrap().value = 2;
    assert_eq!(row(&w).unavailable, Some(Unavailable::Short(stamina_id)));
    assert_eq!(
        Unavailable::Short(stamina_id).reason(&content),
        "you are short of breath"
    );
    let outcome = use_(&mut w, "use:lunge:E");
    assert!(!outcome.took_turn);
    assert_eq!(stamina(&w), 2, "a refused use costs nothing");
    // A knife is no spear: not so held.
    let mut w = world("######\n#@.p.#\n######\n");
    give(&mut w, "wt_knife");
    ranks(&mut w, &[("spears", 9), ("long_blades", 1)]);
    assert_eq!(row(&w).unavailable, Some(Unavailable::NotSoHeld));
    assert!(!use_(&mut w, "use:lunge:E").took_turn);
    // Its row asks for a direction and acts with it.
    let ability = row(&w).ability;
    assert_eq!(
        ability.action(&content, Some(Dir::E)),
        Some(Action::Act("lunge".into(), Dir::E))
    );
}

#[test]
fn practised_technique_cuts_the_time_and_drillmaster_the_stamina() {
    let mut w = world("######\n#@.p.#\n######\n");
    give(&mut w, "wt_spear");
    ranks(&mut w, &[("spears", 9)]);
    let lunge = w.content().weapon_ability_id("lunge").unwrap();
    let base = blow_price(&w);
    w.player_mut().talents.take("practised_technique", None);
    assert_eq!(w.active_price(lunge, base), base * 3 / 4);
    w.player_mut().talents.take("drillmaster", None);
    assert_eq!(w.state.player().weapon_stamina(w.content(), lunge), 2);
    use_(&mut w, "use:lunge:E");
    assert_eq!(stamina(&w), 12 - 2);
    // Never below 1, nor half the action's time.
    let feint = w.content().weapon_ability_id("feint").unwrap();
    w.player_mut().talents.take("drillmaster", None);
    assert_eq!(w.state.player().weapon_stamina(w.content(), feint), 1);
    // Armour-breaker: Sunder 2 less.
    let sunder = w.content().weapon_ability_id("sunder").unwrap();
    let before = w.state.player().weapon_stamina(w.content(), sunder);
    w.player_mut().talents.take("armour_breaker", None);
    assert_eq!(
        w.state.player().weapon_stamina(w.content(), sunder),
        before - 2
    );
}

#[test]
fn the_first_active_rank_grants_the_pool() {
    let mut w = world("#####\n#@p.#\n#####\n");
    let stamina_id = w.content().resource_id(STAMINA).unwrap();
    w.player_mut().resources.remove(stamina_id);
    w.apply(&Action::SetRank {
        skill: "spears".into(),
        rank: 9,
    });
    assert!(
        said(&w)
            .iter()
            .any(|m| m.ends_with("You have learned the lunge. You learn to pace your breath.")),
        "{:#?}",
        said(&w)
    );
    assert!(w.state.player().resources.get(stamina_id).is_some());
    // Once: another active says nothing of breath.
    w.apply(&Action::SetRank {
        skill: "axes".into(),
        rank: 9,
    });
    assert!(!said(&w).last().expect("news").contains("breath"));
}

#[test]
fn every_active_parses_and_round_trips() {
    for text in [
        "use:lunge:E",
        "use:lunge:ne",
        "use:shield_wall",
        "use:piercing_shot:12,5",
    ] {
        let action: Action = text.parse().expect(text);
        let back: Action = action.to_string().parse().expect("again");
        assert_eq!(action, back, "{text}");
    }
    assert_eq!(
        "use:lunge:ne".parse::<Action>(),
        Ok(Action::Act("lunge".into(), Dir::NE))
    );
    // A lower-case letter is a stack, as for Identify.
    assert_eq!(
        "use:identify:e".parse::<Action>(),
        Ok(Action::CastOn("identify".into(), 4))
    );
}
