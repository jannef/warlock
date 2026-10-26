//! The undead roster (#383): every `faction = "undead"` type keeps to the plan's formulas. Each
//! batch of the roster adds its types to [`ROLES`] and raises [`ROSTER_TOP`].

use super::{Affliction, Content};
use crate::stats::Stat;

/// The highest level the roster has filled so far: every level from 1 to it has an undead type.
const ROSTER_TOP: u8 = 30;

/// How far a type's max HP and dice average may lie from the formulas, in percent.
const TOLERANCE: f64 = 35.0;

/// Types the XP guideline does not fit exactly: the pale drinker (#382's example of a drain,
/// outside the roster) is worth more for a drain that comes with every touch.
const OFF_GUIDELINE: &[&str] = &["pale_drinker"];

/// A role's adjustments (#383 "Conventions").
#[derive(Clone, Copy, Debug)]
pub(super) enum Role {
    Shambler,
    Brute,
    Standard,
    Quick,
    Spirit,
    Caster,
    Small,
}

impl Role {
    /// What the role adds to the base Toughness.
    fn toughness(self) -> f64 {
        match self {
            Role::Shambler | Role::Brute => 5.0,
            Role::Standard | Role::Caster => 0.0,
            Role::Quick => -1.0,
            Role::Spirit => -3.0,
            Role::Small => -4.0,
        }
    }

    /// The factor on `base_hp`.
    fn hp(self) -> f64 {
        match self {
            Role::Shambler => 1.5,
            Role::Brute => 1.6,
            Role::Standard => 1.0,
            Role::Quick => 0.9,
            Role::Spirit => 0.6,
            Role::Caster => 0.8,
            Role::Small => 0.5,
        }
    }

    /// The steps it moves the dice along [`LADDER`].
    fn step(self) -> i32 {
        match self {
            Role::Brute => 1,
            Role::Shambler | Role::Standard | Role::Quick => 0,
            Role::Spirit | Role::Caster | Role::Small => -1,
        }
    }
}

/// Every undead type and its role: the roster's (#383's table) and those that were there before.
const ROLES: &[(&str, Role)] = &[
    // Before the roster.
    ("skeleton", Role::Standard),
    ("zombie", Role::Shambler),
    ("barrow_ghoul", Role::Quick),
    ("drowned", Role::Shambler),
    ("barrow_wight", Role::Caster),
    ("ghost", Role::Spirit),
    ("ash_wraith", Role::Caster),
    ("stone_haunt", Role::Spirit),
    ("pale_drinker", Role::Spirit),
    // Batch 1 (#384): levels 1 to 10.
    ("crawling_hand", Role::Small),
    ("skeletal_rat", Role::Small),
    ("kobold_zombie", Role::Shambler),
    ("corpse_candle", Role::Small),
    ("skeleton_archer", Role::Standard),
    ("skeletal_wolf", Role::Quick),
    ("poltergeist", Role::Spirit),
    ("ghast", Role::Quick),
    ("skeleton_warrior", Role::Standard),
    ("shadow", Role::Spirit),
    ("bog_body", Role::Shambler),
    ("ogre_zombie", Role::Brute),
    ("mummy", Role::Shambler),
    ("grave_hound", Role::Quick),
    // Batch 2 (#385): levels 11 to 20.
    ("wraith", Role::Spirit),
    ("fen_ghoul", Role::Quick),
    ("draugr", Role::Brute),
    ("burning_skull", Role::Small),
    ("barghest", Role::Quick),
    ("knocker", Role::Spirit),
    ("spectre", Role::Spirit),
    ("charred_dead", Role::Shambler),
    ("duergar_skeleton", Role::Standard),
    ("vampire_spawn", Role::Quick),
    ("hand_of_glory", Role::Small),
    ("gallows_dead", Role::Standard),
    ("rotting_troll", Role::Brute),
    ("gibbering_shade", Role::Spirit),
    ("bone_wyvern", Role::Brute),
    ("revenant", Role::Standard),
    ("fetch", Role::Spirit),
    ("mummy_priest", Role::Caster),
    // Batch 3 (#386): levels 21 to 30.
    ("bodach", Role::Standard),
    ("grave_knight", Role::Standard),
    ("charnel_heap", Role::Brute),
    ("banshee", Role::Spirit),
    ("wight_lord", Role::Caster),
    ("vampire", Role::Quick),
    ("wormhost", Role::Standard),
    ("bone_drake", Role::Brute),
    ("dread_wraith", Role::Spirit),
    ("headless_knight", Role::Standard),
    ("lich", Role::Caster),
    ("rotting_drake", Role::Brute),
];

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
pub(super) fn max_hp(level: f64, role: Role) -> f64 {
    12.0 + 0.4 * level + role.toughness() + 2.5 * level * role.hp()
}

/// The formula's dice average: the ladder's nearest to `2 + 0.55 L` (flattening past L20),
/// moved by the role's step and one step lower for a drain.
pub(super) fn dice_average(level: f64, role: Role, drains: bool) -> f64 {
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
    let step = role.step() - i32::from(drains);
    let index = (nearest as i32 + step).clamp(0, LADDER.len() as i32 - 1);
    LADDER[index as usize].1
}

pub(super) fn within(actual: f64, expected: f64) -> bool {
    ((actual - expected) / expected * 100.0).abs() <= TOLERANCE
}

#[test]
fn the_undead_keep_to_the_roster() {
    let content = Content::builtin().expect("content");
    let undead = content.factions().id("undead").expect("the undead faction");
    let guideline = content.experience().guideline();
    let mut levels = Vec::new();
    for def in (content.creature_defs().iter()).filter(|c| c.faction == Some(undead)) {
        let id = def.id.as_str();
        let role = ROLES.iter().find(|(r, _)| *r == id).map(|&(_, role)| role);
        let role = role.unwrap_or_else(|| panic!("{id}: give it a role in ROLES"));
        levels.push(def.level);
        if !OFF_GUIDELINE.contains(&id) {
            assert_eq!(
                def.xp,
                guideline.xp(def.level),
                "{id}: off the XP guideline"
            );
        }
        let level = f64::from(def.level);
        let hp = f64::from(def.stats.score(Stat::Toughness)) + f64::from(def.base_hp);
        let want = max_hp(level, role);
        assert!(within(hp, want), "{id}: max HP {hp}, the formula {want:.1}");
        let attack = def
            .attack
            .as_ref()
            .unwrap_or_else(|| panic!("{id}: no attack"));
        let damage = attack.attack.damage;
        let average = f64::from(damage.min() + damage.max()) / 2.0;
        let drains = (attack.on_hit.iter()).any(|h| matches!(h.what, Affliction::Drain(_)));
        let want = dice_average(level, role, drains);
        assert!(
            within(average, want),
            "{id}: dice average {average}, the formula {want}"
        );
    }
    for (id, _) in ROLES {
        let def = content.creature_id(id).map(|c| content.creature(c));
        assert!(
            def.is_some_and(|d| d.faction == Some(undead)),
            "{id}: listed in ROLES but no undead type"
        );
    }
    for level in 1..=ROSTER_TOP {
        assert!(levels.contains(&level), "no undead type at level {level}");
    }
}

#[test]
fn the_formulas_give_the_plans_numbers() {
    // #383's table: the crawling hand's 9 hit points (8 + 1, rounded down part by part) and 1d3,
    // the ogre zombie's 57 and 2d8, the shadow's 1d8 (a spirit's step and the drain's below 2d6).
    assert!((max_hp(1.0, Role::Small) - 9.65).abs() < 1e-9);
    assert_eq!(dice_average(1.0, Role::Small, false), 2.0);
    assert_eq!(max_hp(9.0, Role::Brute).round(), 57.0);
    assert_eq!(dice_average(9.0, Role::Brute, false), 9.0);
    assert_eq!(dice_average(8.0, Role::Spirit, true), 4.5);
    // Past level 20 the target flattens: a level 62 brute's 23.5 is 4d10, one step up 4d12 (the
    // dread wyrm's).
    assert_eq!(dice_average(62.0, Role::Brute, false), 26.0);
}
