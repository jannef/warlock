//! The fiends of no family (#450): the hell hound, the nightmare, the succubus and incubus, the
//! night hag, the cambion, and the rakshasa keep to the undead roster's formulas (#383) for their
//! level and role, belong to the `fiends` faction, and carry the `fiend` tag. The demon, devil,
//! and yugoloth rosters check their own.

use super::{Affliction, Combat, Content};
use crate::alignment::Alignment;
use crate::stats::Stat;

/// How far a type's max HP and dice average may lie from the formulas, in percent (as the
/// undead roster's).
const TOLERANCE: f64 = 35.0;

/// A role's adjustments (#383 "Conventions"): the ones these fiends take.
#[derive(Clone, Copy, Debug)]
enum Role {
    Standard,
    Quick,
    Caster,
}

impl Role {
    /// What the role adds to the base Toughness.
    fn toughness(self) -> f64 {
        match self {
            Role::Standard | Role::Caster => 0.0,
            Role::Quick => -1.0,
        }
    }

    /// The factor on `base_hp`.
    fn hp(self) -> f64 {
        match self {
            Role::Standard => 1.0,
            Role::Quick => 0.9,
            Role::Caster => 0.8,
        }
    }

    /// The steps it moves the dice along [`LADDER`].
    fn step(self) -> i32 {
        match self {
            Role::Standard | Role::Quick => 0,
            Role::Caster => -1,
        }
    }
}

/// Every fiend of the `fiends` faction: its level, role, and tags (#450's table).
const ROSTER: &[(&str, u8, Role, &[&str])] = &[
    ("hell_hound", 13, Role::Quick, &["fiend"]),
    ("nightmare", 21, Role::Quick, &["fiend"]),
    ("succubus", 27, Role::Standard, &["fiend"]),
    ("incubus", 27, Role::Standard, &["fiend"]),
    ("night_hag", 33, Role::Caster, &["fiend", "hag", "caster"]),
    ("cambion", 39, Role::Standard, &["fiend"]),
    ("rakshasa", 52, Role::Caster, &["fiend", "caster"]),
];

/// The dice the roster picks from, by average (#383 "Damage dice").
const LADDER: &[f64] = &[
    2.0, 2.5, 3.5, 4.5, 5.5, 7.0, 9.0, 10.5, 13.5, 14.0, 16.5, 18.0, 19.5, 22.0, 26.0, 27.5, 32.5,
];

/// The formula's max HP: Toughness (12 + 0.4 L, by role) plus base_hp (2.5 L, by role).
fn max_hp(level: f64, role: Role) -> f64 {
    12.0 + 0.4 * level + role.toughness() + 2.5 * level * role.hp()
}

/// The formula's dice average: the ladder's nearest to `2 + 0.55 L` (flattening past L20),
/// moved by the role's step and one step lower for a drain.
fn dice_average(level: f64, role: Role, drains: bool) -> f64 {
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
    let step = role.step() - i32::from(drains);
    LADDER[(nearest as i32 + step).clamp(0, LADDER.len() as i32 - 1) as usize]
}

fn within(actual: f64, expected: f64) -> bool {
    ((actual - expected) / expected * 100.0).abs() <= TOLERANCE
}

#[test]
fn the_fiends_keep_to_the_roster() {
    let content = Content::builtin().expect("content");
    let fiends = content.factions().id("fiends").expect("the fiends faction");
    let guideline = content.experience().guideline();
    for &(id, level, role, tags) in ROSTER {
        let creature = content
            .creature_id(id)
            .unwrap_or_else(|| panic!("{id}: no type"));
        let def = content.creature(creature);
        assert_eq!(def.faction, Some(fiends), "{id}: of the fiends faction");
        assert_eq!(def.level, level, "{id}: its level");
        assert_eq!(def.xp, guideline.xp(level), "{id}: off the XP guideline");
        assert_eq!(def.alignment, Alignment::Evil, "{id}: evil");
        assert_eq!(def.tags, tags, "{id}: its tags");
        assert_eq!(
            def.combat == Combat::Caster,
            tags.contains(&"caster"),
            "{id}: a caster carries the caster tag"
        );
        let level = f64::from(level);
        let hp = f64::from(def.stats.score(Stat::Toughness)) + f64::from(def.base_hp);
        let want = max_hp(level, role);
        assert!(within(hp, want), "{id}: max HP {hp}, the formula {want:.1}");
        let attack = def.attack.as_ref().expect("an attack");
        let damage = attack.attack.damage;
        let average = f64::from(damage.min() + damage.max()) / 2.0;
        let drains = (attack.on_hit.iter()).any(|h| matches!(h.what, Affliction::Drain(_)));
        let want = dice_average(level, role, drains);
        assert!(
            within(average, want),
            "{id}: dice average {average}, the formula {want}"
        );
    }
    // The roster is the whole faction, and every one of it is a fiend.
    for (i, def) in content.creature_defs().iter().enumerate() {
        if def.faction == Some(fiends) {
            assert!(
                ROSTER.iter().any(|(id, ..)| *id == def.id),
                "{}: of the fiends faction but not in ROSTER",
                def.id
            );
        }
        if ROSTER.iter().any(|(id, ..)| *id == def.id) {
            assert!(content.has_tag(super::CreatureId(i as u16), "fiend"));
        }
    }
}

#[test]
fn no_beast_friend_calms_a_fiend() {
    let content = Content::builtin().expect("content");
    let fiends = content.factions().id("fiends").expect("the fiends faction");
    assert!(!content.factions().def(fiends).animal);
}

#[test]
fn the_succubus_and_the_incubus_share_their_numbers() {
    let content = Content::builtin().expect("content");
    let def = |id: &str| content.creature(content.creature_id(id).expect(id));
    let (succubus, incubus) = (def("succubus"), def("incubus"));
    assert_eq!(succubus.stats, incubus.stats);
    assert_eq!(succubus.base_hp, incubus.base_hp);
    assert_eq!(succubus.speed, incubus.speed);
    assert_eq!(succubus.resist, incubus.resist);
    let (a, b) = (succubus.attack.as_ref(), incubus.attack.as_ref());
    assert_eq!(a.map(|a| a.attack.damage), b.map(|a| a.attack.damage));
    assert_ne!(succubus.fg, incubus.fg, "told apart by colour");
}
