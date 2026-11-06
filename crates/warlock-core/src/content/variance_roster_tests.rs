//! The variance pass (#1134, the dread design §7.2): what a role does to a monster's to-hit, its
//! blow's shape, and its speed, on #1097's `L/4`. Each roster's test maps its roles onto a
//! [`Shape`] and calls [`check`] for every type it holds; the speed rule is checked here, tribe by
//! tribe, for every faction of monsters.

use super::{Content, CreatureDef, Keep, MAX_BONUS};
use crate::combat::{DEFAULT_CRIT, DEFAULT_THREAT};

/// What a role does to the numbers the variance pass sets (§7.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Shape {
    /// Quick and assassin: `+L/8` to hit, threat from 19 (crit ×2).
    Striker,
    /// The brutes: `−L/8` to hit, crit ×3 on the natural 24 only.
    Brute,
    /// The shamblers: `−L/8` to hit, the blow as a standard's.
    Shambler,
    /// The casters: `−L/8` to hit, the blow as a standard's.
    Caster,
    /// Everyone else: `L/4`, the default threat and crit.
    Standard,
}

impl Shape {
    /// The natural attack's to-hit at `level` (a striker's at most the data's ±20, which the
    /// quick dread past level 55 reach).
    pub(super) fn to_hit(self, level: u8) -> i32 {
        let (quarter, eighth) = (i32::from(level / 4), i32::from(level / 8));
        match self {
            Shape::Striker => (quarter + eighth).min(MAX_BONUS),
            Shape::Brute | Shape::Shambler | Shape::Caster => quarter - eighth,
            Shape::Standard => quarter,
        }
    }

    /// The natural attack's threat and crit.
    pub(super) fn threat_crit(self) -> (u8, u8) {
        match self {
            Shape::Striker => (19, DEFAULT_CRIT),
            Shape::Brute => (24, 3),
            Shape::Shambler | Shape::Caster | Shape::Standard => (DEFAULT_THREAT, DEFAULT_CRIT),
        }
    }
}

/// Whether the type fights with a weapon in hand (its racial weapon, or a gear line it wields):
/// its spread is the gear's and the skill's, so its natural attack keeps `L/4` as #1097 left it.
pub(super) fn armed(content: &Content, def: &CreatureDef) -> bool {
    (def.gear.lines.iter())
        .flat_map(|line| &line.one_of)
        .filter(|(_, keep)| *keep == Keep::Wield)
        .flat_map(|(stacks, _)| stacks)
        .map(|s| content.object(s.object))
        .any(|o| o.attack.is_some() && o.launches.is_none() && o.ammo.is_none())
}

/// The role's to-hit offset, threat, and crit on the type's natural attack.
pub(super) fn check(content: &Content, def: &CreatureDef, shape: Shape) {
    let id = def.id.as_str();
    let Some(natural) = &def.attack else {
        return;
    };
    let attack = &natural.attack;
    if armed(content, def) {
        assert_eq!(
            attack.to_hit,
            i32::from(def.level / 4),
            "{id}: armed, its natural attack keeps L/4"
        );
        return;
    }
    assert_eq!(
        attack.to_hit,
        shape.to_hit(def.level),
        "{id}: a {shape:?}'s to-hit"
    );
    assert_eq!(
        (attack.threat, attack.crit),
        shape.threat_crit(),
        "{id}: a {shape:?}'s threat and crit"
    );
}

/// The factions the speed rule leaves out: the beasts, the people, the player, and the layer
/// keepers (uniques, each its own fight).
const NOT_MONSTERS: &[&str] = &[
    "layer_keepers",
    "merchants",
    "peaceful",
    "player",
    "spirits",
    "wildlife",
];

/// The bulk's speeds: between these, by temperament (§7.2).
const BULK: std::ops::RangeInclusive<u32> = 800..=1250;

/// The least speed of the quick dread; the slow dread are at 600 or 700.
const QUICK_DREAD: u32 = 1400;

/// Fast by design, kept past the bulk: the quickling (docs/design/monster-dread.md §6, "the claw bug of this game")
/// and the vortices (§5.3, D3: "speed +200").
const FAST_BY_DESIGN: &[&str] = &[
    "dust_devil",
    "ember_mote",
    "quickling",
    "spark_mote",
    "water_mote",
];

#[test]
fn the_speed_rule_holds_tribe_by_tribe() {
    let content = Content::builtin().expect("content");
    let mut tribes: std::collections::BTreeMap<String, Vec<u32>> = Default::default();
    for def in content.creature_defs() {
        let Some(faction) = def.faction else {
            continue;
        };
        let faction = content.factions().def(faction).id.clone();
        let id = def.id.as_str();
        let generated = def.mimic.is_some() || def.animated.is_some();
        if NOT_MONSTERS.contains(&faction.as_str()) || def.unique || generated {
            continue;
        }
        if def.dread {
            assert!(
                matches!(def.speed, 600 | 700) || def.speed >= QUICK_DREAD,
                "{id}: dread at 600 or 700, or {QUICK_DREAD} and up, not {}",
                def.speed
            );
        } else if !FAST_BY_DESIGN.contains(&id) {
            assert!(
                BULK.contains(&def.speed),
                "{id}: bulk between 800 and 1250, not {}",
                def.speed
            );
        }
        tribes.entry(faction).or_default().push(def.speed);
    }
    for (tribe, speeds) in &tribes {
        assert!(
            speeds.len() < 2 || speeds.iter().any(|&s| s != 1000),
            "{tribe}: every type at 1000"
        );
    }
}

/// The peoples' rank-and-file: 950 to 1050.
pub(super) fn rank_and_file(def: &CreatureDef) {
    assert!(
        (950..=1050).contains(&def.speed),
        "{}: a people's rank-and-file at 950 to 1050, not {}",
        def.id,
        def.speed
    );
}

#[test]
fn the_shapes_give_the_designs_numbers() {
    // §7.2 at level 20: 5 the standard, 7 the quick, and 3 the brute (the table's 2 is 2.5
    // rounded: L/4 − L/8 rounds each part down).
    assert_eq!(Shape::Brute.to_hit(20), 3);
    assert_eq!(Shape::Standard.to_hit(20), 5);
    assert_eq!(Shape::Striker.to_hit(20), 7);
    assert_eq!(Shape::Striker.threat_crit(), (19, 2));
    assert_eq!(Shape::Brute.threat_crit(), (24, 3));
}
