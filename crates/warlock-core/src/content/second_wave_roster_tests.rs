//! The second wave (#1138, the dread design §15 and §12 D14): the black pudding, cockatrice,
//! hydra, rust monster, and gorgon, content only on the mechanisms the dread slices built, plus
//! the hydra's heads. Each keeps to the undead roster's formulas (#383 "Conventions") for its
//! level and role (its natural attacks' dice summed: the hydra's five heads share a brute's
//! budget), sits on the XP guideline at a level of its own, and carries the gimmick it was
//! written for.

use super::creature::CorrodesWhat;
use super::status::Affliction;
use super::{Combat, Content, CreatureDef};
use crate::stats::Stat;

/// How far a type's max HP and dice average may lie from the formulas, in percent.
const TOLERANCE: f64 = 35.0;

/// A role's adjustments (#383 "Conventions"): the ones the wave takes.
#[derive(Clone, Copy, Debug)]
enum Role {
    Shambler,
    Brute,
    Standard,
    Small,
}

impl Role {
    fn toughness(self) -> f64 {
        match self {
            Role::Shambler | Role::Brute => 5.0,
            Role::Standard => 0.0,
            Role::Small => -4.0,
        }
    }

    fn hp(self) -> f64 {
        match self {
            Role::Shambler => 1.5,
            Role::Brute => 1.6,
            Role::Standard => 1.0,
            Role::Small => 0.5,
        }
    }

    fn step(self) -> i32 {
        match self {
            Role::Brute => 1,
            Role::Shambler | Role::Standard => 0,
            Role::Small => -1,
        }
    }
}

/// Every type of the wave, its role, and whether it is dread (the heads are not: they are what a
/// hydra leaves).
const ROLES: &[(&str, Role, bool)] = &[
    ("hydra_head", Role::Small, false),
    ("cockatrice", Role::Small, true),
    ("rust_monster", Role::Standard, true),
    ("black_pudding", Role::Shambler, true),
    ("hydra", Role::Brute, true),
    ("gorgon", Role::Brute, true),
];

/// The dice the roster picks from, by average (#383 "Damage dice": 1d3 to 5d12).
const LADDER: &[f64] = &[
    2.0, 2.5, 3.5, 4.5, 5.5, 7.0, 9.0, 10.5, 13.5, 14.0, 16.5, 18.0, 19.5, 22.0, 26.0, 27.5, 32.5,
];

fn max_hp(level: f64, role: Role) -> f64 {
    (12.0 + 0.4 * level + role.toughness()).min(40.0) + 2.5 * level * role.hp()
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

fn def<'c>(content: &'c Content, id: &str) -> &'c CreatureDef {
    content.creature(content.creature_id(id).unwrap_or_else(|| panic!("{id}")))
}

/// Whether `riders` give Stiffening behind a Toughness check.
fn stiffens(content: &Content, riders: &[super::status::OnHit]) -> bool {
    let stiffening = content.status_id("stiffening").expect("stiffening");
    riders.iter().any(|r| {
        matches!(r.what, Affliction::Status { status, check: Some((Stat::Toughness, _)), .. }
            if status == stiffening)
    })
}

#[test]
fn the_second_wave_keeps_to_the_roster() {
    let content = Content::builtin().expect("content");
    let guideline = content.experience().guideline();
    let mut levels = Vec::new();
    for &(id, role, dread) in ROLES {
        let def = def(&content, id);
        assert_eq!(
            def.xp,
            guideline.xp(def.level),
            "{id}: off the XP guideline"
        );
        assert_eq!(def.dread, dread, "{id}: dread");
        assert!(!def.unique, "{id}: no unique");
        let level = f64::from(def.level);
        let hp = f64::from(def.stats.score(Stat::Toughness)) + f64::from(def.base_hp);
        let want = max_hp(level, role) * super::variance_roster_tests::starter(def.level);
        assert!(within(hp, want), "{id}: max HP {hp}, the formula {want:.1}");
        let average: f64 = (def.natural_attacks())
            .map(|a| f64::from(a.attack.damage.min() + a.attack.damage.max()) / 2.0)
            .sum();
        let want = dice_average(level, role);
        assert!(
            within(average, want),
            "{id}: dice average {average}, the formula {want}"
        );
        levels.push(def.level);
    }
    let mut distinct = levels.clone();
    distinct.sort_unstable();
    distinct.dedup();
    assert_eq!(distinct.len(), levels.len(), "each at a level of its own");
}

#[test]
fn each_carries_its_gimmick() {
    let content = Content::builtin().expect("content");
    // The black pudding: a blade splits it, and its acid eats both ways.
    let pudding = def(&content, "black_pudding");
    let slashing = content.damage_type_id("slashing").expect("slashing");
    let bludgeoning = content.damage_type_id("bludgeoning").expect("bludgeoning");
    let splits = pudding.splits.as_ref().expect("it splits");
    assert!(splits.on.contains(&slashing) && !splits.on.contains(&bludgeoning));
    assert_eq!(pudding.corrodes.map(|c| c.what), Some(CorrodesWhat::Both));
    // The cockatrice: its peck stiffens.
    let cockatrice = def(&content, "cockatrice");
    let peck = cockatrice.attack.as_ref().expect("a peck");
    assert!(stiffens(&content, &peck.on_hit), "the peck stiffens");
    // The hydra: five heads, regeneration, and heads out of the carcass.
    let hydra = def(&content, "hydra");
    assert_eq!(hydra.natural_attacks().count(), 5, "five heads");
    assert!(hydra.regen > 100, "its wounds close");
    let head = content.creature_id("hydra_head").expect("the heads");
    assert_eq!(
        hydra.dies_into.as_ref().map(|d| d.pool.clone()),
        Some(vec![head])
    );
    assert!(
        !content.creature(head).stray,
        "the heads come only of a hydra"
    );
    // The rust monster: its acid both ways.
    let rust = def(&content, "rust_monster");
    assert_eq!(rust.corrodes.map(|c| c.what), Some(CorrodesWhat::Both));
    // The gorgon: a dragon's breath that stiffens.
    let gorgon = def(&content, "gorgon");
    assert_eq!(gorgon.combat, Combat::Dragon);
    let breath = gorgon.breath.as_ref().expect("a breath");
    assert!(stiffens(&content, &breath.riders), "the breath stiffens");
    let poison = content.damage_type_id("poison").expect("poison");
    assert_eq!(breath.kind, poison);
}
