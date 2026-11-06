//! The elemental roster (#453): every type tagged `elemental` keeps to the undead roster's
//! formulas (#383, `undead_roster_tests`) for its level and role, carries one element's tag, and
//! each element has its ladder of rungs.

use super::{BurstWhen, Content};
use crate::stats::Stat;

/// The vortices (the dread design §5.3): every mote.
const VORTICES: &[&str] = &[
    "stone_mote",
    "ember_mote",
    "dust_devil",
    "water_mote",
    "frost_mote",
    "spark_mote",
];

/// How far a type's max HP and dice average may lie from the formulas, in percent.
const TOLERANCE: f64 = 35.0;

/// A role's adjustments (#383 "Conventions"; the elementals use four of its seven).
#[derive(Clone, Copy, Debug)]
enum Role {
    Brute,
    Standard,
    Quick,
    Small,
}

impl Role {
    fn toughness(self) -> f64 {
        match self {
            Role::Brute => 5.0,
            Role::Standard => 0.0,
            Role::Quick => -1.0,
            Role::Small => -4.0,
        }
    }

    fn hp(self) -> f64 {
        match self {
            Role::Brute => 1.6,
            Role::Standard => 1.0,
            Role::Quick => 0.9,
            Role::Small => 0.5,
        }
    }

    fn step(self) -> i32 {
        match self {
            Role::Brute => 1,
            Role::Standard | Role::Quick => 0,
            Role::Small => -1,
        }
    }
}

/// The elements: the tag, the role of the rungs above the mote (a mote is [`Role::Small`]),
/// and the damage type each of its rungs is immune to besides poison, if any.
const ELEMENTS: &[(&str, Role, Option<&str>)] = &[
    ("fire", Role::Standard, Some("fire")),
    ("earth", Role::Brute, Some("electricity")),
    ("air", Role::Quick, None),
    ("water", Role::Standard, None),
    ("ice", Role::Brute, Some("cold")),
    ("lightning", Role::Quick, Some("electricity")),
];

/// The fewest rungs an element's ladder has.
const MIN_RUNGS: usize = 6;

/// The dice the roster picks from, by average (#383 "Damage dice": 1d3 to 5d12).
const LADDER: &[f64] = &[
    2.0, 2.5, 3.5, 4.5, 5.5, 7.0, 9.0, 10.5, 13.5, 14.0, 16.5, 18.0, 19.5, 22.0, 26.0, 27.5, 32.5,
];

fn max_hp(level: f64, role: Role) -> f64 {
    12.0 + 0.4 * level + role.toughness() + 2.5 * level * role.hp()
}

fn dice_average(level: f64, role: Role) -> f64 {
    let target = if level <= 20.0 {
        2.0 + 0.55 * level
    } else {
        13.0 + 0.25 * (level - 20.0)
    };
    let nearest = (0..LADDER.len())
        .min_by(|&a, &b| {
            (LADDER[a] - target)
                .abs()
                .total_cmp(&(LADDER[b] - target).abs())
        })
        .expect("a ladder");
    let index = (nearest as i32 + role.step()).clamp(0, LADDER.len() as i32 - 1);
    LADDER[index as usize]
}

fn within(actual: f64, expected: f64) -> bool {
    ((actual - expected) / expected * 100.0).abs() <= TOLERANCE
}

#[test]
fn the_elementals_keep_to_the_roster() {
    let content = Content::builtin().expect("content");
    let guideline = content.experience().guideline();
    let small = content.sizes().iter().position(|s| s.id == "small");
    let factions = ["elemental", "fire"].map(|f| content.factions().id(f).expect(f));
    let poison = content.damage_type_id("poison").expect("poison");
    let mut rungs = vec![Vec::new(); ELEMENTS.len()];
    for c in content.tagged("elemental") {
        let def = content.creature(c);
        let id = def.id.as_str();
        let elements: Vec<usize> = (0..ELEMENTS.len())
            .filter(|&e| content.has_tag(c, ELEMENTS[e].0))
            .collect();
        let [element] = elements[..] else {
            panic!("{id}: give it exactly one element's tag");
        };
        let (_, role, immune) = ELEMENTS[element];
        let role = if Some(def.size) == small {
            Role::Small
        } else {
            role
        };
        rungs[element].push(def.level);
        assert!(
            def.faction.is_some_and(|f| factions.contains(&f)),
            "{id}: of the elemental faction (the cinder elemental: the delf's fire)"
        );
        assert_eq!(
            def.xp,
            guideline.xp(def.level),
            "{id}: off the XP guideline"
        );
        let level = f64::from(def.level);
        let hp = f64::from(def.stats.score(Stat::Toughness)) + f64::from(def.base_hp);
        let want = max_hp(level, role);
        assert!(within(hp, want), "{id}: max HP {hp}, the formula {want:.1}");
        // A vortex's burst is its whole offence (the dread design §5.3): the motes'.
        // A vortex's burst is its whole offence (the dread design §5.3): the motes'.
        match (&def.attack, &def.burst) {
            (Some(attack), _) => {
                let damage = attack.attack.damage;
                let average = f64::from(damage.min() + damage.max()) / 2.0;
                let want = dice_average(level, role);
                assert!(
                    within(average, want),
                    "{id}: dice average {average}, the formula {want}"
                );
            }
            (None, Some(burst)) if burst.when == BurstWhen::Touches => {
                assert!(VORTICES.contains(&id), "{id}: only a mote is a vortex");
                assert!(
                    (4..=6).contains(&(burst.damage.max() / 2)) && burst.damage.min() == 2,
                    "{id}: a vortex bursts for 2d4 to 2d6"
                );
                assert_eq!(burst.radius, 1, "{id}: a vortex's radius");
            }
            _ => panic!("{id}: no attack"),
        }
        let resists = |t| def.resist.iter().any(|&(r, v)| r == t && v == 100);
        assert!(resists(poison), "{id}: immune to poison");
        if let Some(immune) = immune {
            let t = content.damage_type_id(immune).expect("a damage type");
            assert!(resists(t), "{id}: immune to {immune}");
        }
        assert!(!def.corpse, "{id}: nothing is left of an elemental");
    }
    for (e, levels) in rungs.iter_mut().enumerate() {
        levels.sort_unstable();
        let element = ELEMENTS[e].0;
        assert!(
            levels.len() >= MIN_RUNGS,
            "{element}: {} rungs, at least {MIN_RUNGS}",
            levels.len()
        );
        assert!(
            levels.first().is_some_and(|&l| l <= 10) && levels.last().is_some_and(|&l| l >= 50),
            "{element}: its ladder spans the levels ({levels:?})"
        );
    }
}

#[test]
fn the_cinder_elemental_is_the_fire_lines_rung() {
    let content = Content::builtin().expect("content");
    let cinder = content.creature_id("cinder_elemental").expect("cinder");
    assert!(content.has_tag(cinder, "elemental") && content.has_tag(cinder, "fire"));
    assert!(content.has_tag(cinder, "fire-thing"), "still the delf's");
}

/// Each line's rungs (lesser, plain, greater, elder, eldest; the dread design §5.3), with the
/// cinder elemental on the fire line's lesser rung.
const RUNGS: &[(&str, [&[&str]; 5])] = &[
    (
        "fire",
        [
            &["lesser_fire_elemental", "cinder_elemental"],
            &["fire_elemental"],
            &["greater_fire_elemental"],
            &["elder_fire_elemental"],
            &["balefire"],
        ],
    ),
    (
        "lightning",
        [
            &["lesser_lightning_elemental"],
            &["lightning_elemental"],
            &["greater_lightning_elemental"],
            &["elder_lightning_elemental"],
            &["thunderhead"],
        ],
    ),
    (
        "ice",
        [
            &["lesser_ice_elemental"],
            &["ice_elemental"],
            &["greater_ice_elemental"],
            &["elder_ice_elemental"],
            &["rimethurse"],
        ],
    ),
    (
        "air",
        [
            &["lesser_air_elemental"],
            &["air_elemental"],
            &["greater_air_elemental"],
            &["elder_air_elemental"],
            &["welkin_wind"],
        ],
    ),
    (
        "water",
        [
            &["lesser_water_elemental"],
            &["water_elemental"],
            &["greater_water_elemental"],
            &["elder_water_elemental"],
            &["brimwolf"],
        ],
    ),
    (
        "earth",
        [
            &["lesser_earth_elemental"],
            &["earth_elemental"],
            &["greater_earth_elemental"],
            &["elder_earth_elemental"],
            &["hoarstone"],
        ],
    ),
];

/// A line's death throes (the dread design §5.3).
struct Throes {
    kind: &'static str,
    /// By rung, lesser to eldest.
    radii: [i32; 5],
    /// Its rider's status, and the first rung that has it.
    rider: Option<(&'static str, usize)>,
    douses: bool,
    gust: bool,
}

/// Every line's death throes; earth has none.
fn throes(line: &str) -> Option<Throes> {
    let wide = [1, 2, 2, 3, 3];
    let narrow = [1, 1, 2, 2, 3];
    let of = |kind, radii, rider| Throes {
        kind,
        radii,
        rider,
        douses: false,
        gust: false,
    };
    match line {
        "fire" => Some(of("fire", wide, Some(("burning", 1)))),
        "lightning" => Some(of("electricity", wide, Some(("staggered", 0)))),
        "ice" => Some(of("cold", wide, Some(("sluggish", 0)))),
        "air" => Some(Throes {
            gust: true,
            ..of("bludgeoning", narrow, Some(("staggered", 0)))
        }),
        "water" => Some(Throes {
            douses: true,
            ..of("bludgeoning", narrow, None)
        }),
        _ => None,
    }
}

/// The earth line's stones (the dread design §5.3): the giants' rule, range 5 + L/10, the
/// fist's dice a step less (as many dice, a die a size smaller), a throw's 1500, bludgeoning,
/// and the stone staggers.
fn check_stones(content: &Content, id: &str, def: &super::CreatureDef) {
    assert!(def.burst.is_none(), "{id}: stone does not burst");
    let throw = def
        .throw
        .as_ref()
        .unwrap_or_else(|| panic!("{id}: a throw"));
    let level = i32::from(def.level);
    assert_eq!(throw.range, 5 + level / 10, "{id}: range 5 + L/10");
    assert_eq!(throw.cost, Some(1500), "{id}: a throw's cost");
    let dice = |d: crate::dice::Roll| (d.min(), d.max() / d.min().max(1));
    let fist = def.attack.as_ref().expect("a fist").attack.damage;
    let ((n, sides), (m, less)) = (dice(fist), dice(throw.attack.attack.damage));
    assert_eq!(n, m, "{id}: as many dice");
    let step = if sides == 4 { 3 } else { sides - 2 };
    assert_eq!(less, step, "{id}: the dice a step less");
    let bludgeoning = content.damage_type_id("bludgeoning").expect("bludgeoning");
    assert_eq!(throw.attack.attack.kinds.main(), Some(bludgeoning), "{id}");
    let staggered = content.status_id("staggered").expect("staggered");
    let staggers = (throw.attack.on_hit.iter()).any(|h| {
        matches!(h.what, super::status::Affliction::Status { status, .. } if status == staggered)
    });
    assert!(staggers, "{id}: a stone staggers");
}

#[test]
fn every_line_but_earth_has_death_throes_and_earth_throws_stones() {
    let content = Content::builtin().expect("content");
    let mut listed = Vec::new();
    for &(line, rungs) in RUNGS {
        for (rung, ids) in rungs.iter().enumerate() {
            for &id in *ids {
                listed.push(id);
                let c = content.creature_id(id).unwrap_or_else(|| panic!("{id}"));
                assert!(content.has_tag(c, line), "{id}: of the {line} line");
                let def = content.creature(c);
                let Some(want) = throes(line) else {
                    check_stones(&content, id, def);
                    continue;
                };
                let burst = (def.burst.as_ref()).unwrap_or_else(|| panic!("{id}: death throes"));
                assert_eq!(burst.when, BurstWhen::Dies, "{id}");
                let kind = want.kind;
                assert_eq!(
                    Some(burst.kind),
                    content.damage_type_id(kind),
                    "{id}: {kind}"
                );
                assert_eq!(
                    burst.radius, want.radii[rung],
                    "{id}: the radius of rung {rung}"
                );
                // Half the doubled damage spells' dice of its level, (4 + L/4)d6: (2 + L/8)d6.
                let dice = 2 + i32::from(def.level) / 8;
                assert_eq!(
                    (burst.damage.min(), burst.damage.max()),
                    (dice, 6 * dice),
                    "{id}: {dice}d6"
                );
                let rider: Vec<_> = (want.rider.filter(|&(_, from)| rung >= from))
                    .map(|(status, _)| content.status_id(status).expect(status))
                    .into_iter()
                    .collect();
                let riders: Vec<_> = burst.riders.iter().map(|r| r.status).collect();
                assert_eq!(riders, rider, "{id}: its rider");
                assert_eq!((burst.douses, burst.gust), (want.douses, want.gust), "{id}");
            }
        }
    }
    for c in content.tagged("elemental") {
        let id = content.creature(c).id.as_str();
        assert!(
            listed.contains(&id) || VORTICES.contains(&id),
            "{id}: on no rung of its line"
        );
    }
}
