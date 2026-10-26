//! The giant tribe (#457): every `faction = "giant"` type keeps to the plan's formulas (#383's,
//! for a living brute), carries the tribe's tags, and the giants throw boulders as far and as hard
//! as their level says.

use super::Content;
use crate::stats::Stat;

/// How far a type's max HP and dice average may lie from the formulas, in percent.
const TOLERANCE: f64 = 35.0;

/// A role's adjustments (#457 "Formulas").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    /// The giants: their natural attack is their giant-sized weapon, one dice step up.
    Giant,
    /// The ogres, who carry a maul: their fists at the level's dice.
    Ogre,
    /// The ogre mage and the giant shaman.
    Caster,
}

impl Role {
    /// What the role adds to the base Toughness (12 + 0.4 L).
    fn toughness(self) -> f64 {
        match self {
            Role::Giant | Role::Ogre => 5.0,
            Role::Caster => 0.0,
        }
    }

    /// The factor on `base_hp` (2.5 L).
    fn hp(self) -> f64 {
        match self {
            Role::Giant | Role::Ogre => 1.6,
            Role::Caster => 0.8,
        }
    }

    /// The steps it moves its melee dice along [`LADDER`].
    fn step(self) -> i32 {
        match self {
            Role::Giant => 1,
            Role::Ogre => 0,
            Role::Caster => -1,
        }
    }
}

/// Every type of the tribe, its role, and whether it is an ogre.
const ROLES: &[(&str, Role, bool)] = &[
    ("ogre", Role::Ogre, true),
    ("ogre_mauler", Role::Ogre, true),
    ("ogre_mage", Role::Caster, true),
    ("hill_giant", Role::Giant, false),
    ("stone_giant", Role::Giant, false),
    ("frost_giant", Role::Giant, false),
    ("fire_giant", Role::Giant, false),
    ("giant_shaman", Role::Caster, false),
    ("cloud_giant", Role::Giant, false),
    ("storm_giant", Role::Giant, false),
    ("giant_jarl", Role::Giant, false),
];

/// The thrower whose throws are the tribe's signature: two cells farther.
const SIGNATURE: &str = "stone_giant";

/// The dice the roster picks from, by average (#383 "Damage dice": 1d3 to 5d12).
const LADDER: &[(&str, f64)] = &[
    ("1d3", 2.0),
    ("1d4", 2.5),
    ("1d6", 3.5),
    ("1d8", 4.5),
    ("1d10", 5.5),
    ("2d6", 7.0),
    ("2d8", 9.0),
    ("3d6", 10.5),
    ("3d8", 13.5),
    ("4d6", 14.0),
    ("3d10", 16.5),
    ("4d8", 18.0),
    ("3d12", 19.5),
    ("4d10", 22.0),
    ("4d12", 26.0),
    ("5d10", 27.5),
    ("5d12", 32.5),
];

/// The formula's max HP: Toughness (12 + 0.4 L, by role) plus base_hp (2.5 L, by role).
fn max_hp(level: f64, role: Role) -> f64 {
    12.0 + 0.4 * level + role.toughness() + 2.5 * level * role.hp()
}

/// The ladder's nearest to `2 + 0.55 L` (flattening past L20 to `13 + 0.25 (L − 20)`), moved
/// `step` steps.
fn dice_average(level: f64, step: i32) -> f64 {
    let target = if level <= 20.0 {
        2.0 + 0.55 * level
    } else {
        13.0 + 0.25 * (level - 20.0)
    };
    let nearest = (0..LADDER.len())
        .min_by(|&a, &b| {
            let (da, db) = ((LADDER[a].1 - target).abs(), (LADDER[b].1 - target).abs());
            da.total_cmp(&db)
        })
        .expect("a ladder");
    let index = (nearest as i32 + step).clamp(0, LADDER.len() as i32 - 1);
    LADDER[index as usize].1
}

fn within(actual: f64, expected: f64) -> bool {
    ((actual - expected) / expected * 100.0).abs() <= TOLERANCE
}

fn average(damage: crate::dice::Dice) -> f64 {
    f64::from(damage.min() + damage.max()) / 2.0
}

#[test]
fn the_giants_keep_to_the_roster() {
    let content = Content::builtin().expect("content");
    let giant = content.factions().id("giant").expect("the giant faction");
    let guideline = content.experience().guideline();
    let tribe: Vec<_> = (content.creature_defs().iter())
        .filter(|c| c.faction == Some(giant))
        .collect();
    assert_eq!(
        tribe.len(),
        ROLES.len(),
        "every type of the tribe has a role"
    );
    for def in tribe {
        let id = def.id.as_str();
        let (_, role, ogre) = *ROLES
            .iter()
            .find(|(r, _, _)| *r == id)
            .unwrap_or_else(|| panic!("{id}: give it a role in ROLES"));
        let level = f64::from(def.level);
        assert_eq!(
            def.xp,
            guideline.xp(def.level),
            "{id}: off the XP guideline"
        );
        assert!(
            (12..=50).contains(&def.level),
            "{id}: outside the tribe's span"
        );
        // The tags: every one a giant, the ogres ogres, the casters casters.
        assert!(
            def.tags.iter().any(|t| t == "giant"),
            "{id}: tag it `giant`"
        );
        assert_eq!(def.tags.iter().any(|t| t == "ogre"), ogre, "{id}: `ogre`");
        let caster = def.combat == super::Combat::Caster;
        assert_eq!(caster, role == Role::Caster, "{id}: a caster is a Caster");
        assert_eq!(
            def.tags.iter().any(|t| t == "caster"),
            caster,
            "{id}: `caster`"
        );
        let hp = f64::from(def.stats.score(Stat::Toughness)) + f64::from(def.base_hp);
        let want = max_hp(level, role);
        assert!(within(hp, want), "{id}: max HP {hp}, the formula {want:.1}");
        let attack = (def.attack.as_ref()).unwrap_or_else(|| panic!("{id}: no attack"));
        let got = average(attack.attack.damage);
        let want = dice_average(level, role.step());
        assert!(
            within(got, want),
            "{id}: dice average {got}, the formula {want}"
        );
        // The giants throw boulders (but their shaman); the ogres throw nothing.
        let throws = role == Role::Giant;
        assert_eq!(
            def.throw.is_some(),
            throws,
            "{id}: a giant throws, nothing else"
        );
        if let Some(throw) = &def.throw {
            let object = &content.object(throw.object).id;
            assert_eq!(object, "boulder", "{id}: giants hurl boulders");
            let got = average(throw.attack.attack.damage);
            let want = dice_average(level, 0);
            assert!(
                within(got, want),
                "{id}: throw average {got}, the formula {want}"
            );
            let range = 5 + i32::from(def.level) / 10 + if id == SIGNATURE { 2 } else { 0 };
            assert_eq!(throw.range, range, "{id}: throw range");
        }
    }
}

#[test]
fn the_formulas_give_the_plans_numbers() {
    // #457's table: the hill giant's 105 hit points and 4d6 great club, its 3d8 boulder; the
    // ogre's 70 and 2d8 fists; the jarl's 4d10 axe and 3d12 boulder.
    assert_eq!(max_hp(20.0, Role::Giant).round(), 105.0);
    assert_eq!(dice_average(20.0, 1), 14.0);
    assert_eq!(dice_average(20.0, 0), 13.5);
    assert_eq!(max_hp(12.0, Role::Ogre).round(), 70.0);
    assert_eq!(dice_average(12.0, 0), 9.0);
    assert_eq!(dice_average(50.0, 1), 22.0);
    assert_eq!(dice_average(50.0, 0), 19.5);
}
