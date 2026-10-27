//! The duergar (#518): every `faction = "duergar"` type keeps to the tribe's formulas, the undead
//! roster's (#383) hit points and roles on living stats, as the orcs' (#455) do; the armed ones
//! wield their racial weapons at a rank that grows with their level, the two who fight bare by
//! the roster's dice, and every one of them resists poison and a little cold.

use super::{Combat, Content, Stance};
use crate::stats::Stat;

/// How far a type's hit points and dice may lie from the formulas, in percent.
const TOLERANCE: f64 = 35.0;

/// A role's adjustments (#518, #383's roles).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Role {
    /// The three of the delf wave (#335), which came before the roster: held to the XP guideline
    /// and the hit points only.
    Older,
    Archer,
    Standard,
    Officer,
    Quick,
    Brute,
    Caster,
}

impl Role {
    /// What the role adds to the base Strength, Dexterity, and Toughness.
    fn adjust(self) -> [f64; 3] {
        match self {
            Role::Older | Role::Standard | Role::Officer => [0.0, 0.0, 0.0],
            Role::Archer => [0.0, 3.0, 0.0],
            Role::Quick => [0.0, 5.0, -1.0],
            Role::Brute => [5.0, -2.0, 5.0],
            Role::Caster => [-3.0, 0.0, 0.0],
        }
    }

    /// The factor on `base_hp`.
    fn hp(self) -> f64 {
        match self {
            Role::Older | Role::Standard | Role::Archer => 1.0,
            Role::Officer => 1.1,
            Role::Quick => 0.9,
            Role::Brute => 1.6,
            Role::Caster => 0.8,
        }
    }

    /// The dice ladder steps the role moves a natural attack.
    fn step(self) -> isize {
        match self {
            Role::Brute => 1,
            Role::Caster => -1,
            _ => 0,
        }
    }
}

/// Every type of the duergar's faction and its role, in level order.
const ROLES: &[(&str, Role)] = &[
    ("duergar_slinger", Role::Archer),
    ("duergar_miner", Role::Older),
    ("duergar_warrior", Role::Older),
    ("duergar_smith", Role::Older),
    ("duergar_crossbowman", Role::Archer),
    ("duergar_veteran", Role::Standard),
    ("duergar_priest", Role::Caster),
    ("duergar_hulk", Role::Brute),
    ("duergar_spider_rider", Role::Quick),
    ("duergar_mindbreaker", Role::Caster),
    ("duergar_thane", Role::Officer),
    ("duergar_king", Role::Officer),
];

/// The roster's dice by average (#383 "Damage dice").
const LADDER: &[f64] = &[
    2.0, 2.5, 3.5, 4.5, 5.5, 7.0, 9.0, 10.5, 13.5, 14.0, 16.5, 18.0, 19.5, 22.5,
];

/// The formula's max HP: Toughness (12 + 0.4 L, by role) plus base_hp (2.5 L, by role).
fn max_hp(level: f64, role: Role) -> f64 {
    12.0 + 0.4 * level + role.adjust()[2] + 2.5 * level * role.hp()
}

/// The formula's natural attack: the ladder's nearest to `2 + 0.55 L`, flattening past L20,
/// moved by the role's step.
fn dice_average(level: f64, role: Role) -> f64 {
    let target = if level <= 20.0 {
        2.0 + 0.55 * level
    } else {
        13.0 + 0.25 * (level - 20.0)
    };
    let distance = |a: f64| (a - target).abs();
    let nearest = (0..LADDER.len())
        .min_by(|&a, &b| distance(LADDER[a]).total_cmp(&distance(LADDER[b])))
        .expect("a ladder");
    let stepped = (nearest as isize + role.step()).clamp(0, LADDER.len() as isize - 1);
    LADDER[stepped as usize]
}

fn within(actual: f64, expected: f64) -> bool {
    ((actual - expected) / expected * 100.0).abs() <= TOLERANCE
}

#[test]
fn the_duergar_keep_to_the_tribes_formulas() {
    let content = Content::builtin().expect("content");
    let duergar = content
        .factions()
        .id("duergar")
        .expect("the duergar's faction");
    let guideline = content.experience().guideline();
    let mut levels = Vec::new();
    for def in (content.creature_defs().iter()).filter(|c| c.faction == Some(duergar)) {
        let id = def.id.as_str();
        let &(_, role) = (ROLES.iter().find(|(r, _)| *r == id))
            .unwrap_or_else(|| panic!("{id}: give it a role in ROLES"));
        levels.push(def.level);
        assert_eq!(def.campaign, None, "{id}: shared");
        assert_eq!(
            def.xp,
            guideline.xp(def.level),
            "{id}: off the XP guideline"
        );
        let creature = content.creature_id(id).expect(id);
        assert!(content.has_tag(creature, "duergar"), "{id}: tag `duergar`");
        assert_eq!(
            content.has_tag(creature, "archer"),
            def.combat == Combat::Ranged,
            "{id}: `archer` on the shooters only"
        );
        assert_eq!(
            content.has_tag(creature, "caster"),
            def.combat == Combat::Caster,
            "{id}: `caster` on the casters only"
        );
        // Poison 25 and cold 10, the deep's fungus and its chill.
        for (kind, least) in [("poison", 25), ("cold", 10)] {
            let kind = content.damage_type_id(kind).expect(kind);
            let resist = (def.resist.iter()).find(|(k, _)| *k == kind);
            assert!(
                resist.is_some_and(|&(_, r)| r >= least),
                "{id}: resists {kind:?} {least}"
            );
        }
        let level = f64::from(def.level);
        let hp = f64::from(def.stats.score(Stat::Toughness)) + f64::from(def.base_hp);
        let want = max_hp(level, role);
        assert!(within(hp, want), "{id}: max HP {hp}, the formula {want:.1}");
        if role == Role::Older {
            continue;
        }
        let [st, dx, to] = role.adjust();
        for (stat, want) in [
            (Stat::Strength, 12.0 + 0.4 * level + st),
            (Stat::Dexterity, 10.0 + 0.25 * level + dx),
            (Stat::Toughness, 12.0 + 0.4 * level + to),
        ] {
            let score = f64::from(def.stats.score(stat));
            assert!(
                (score - want).abs() <= 1.0,
                "{id}: {stat:?} {score}, the formula {want}"
            );
        }
        match def.gear.racial_weapon {
            // Armed: its racial weapon, at the rank the formula gives (1 + L/7).
            Some(weapon) => {
                let skill = (content.object(weapon).skill)
                    .unwrap_or_else(|| panic!("{id}: its racial weapon has no skill"));
                assert_eq!(
                    u32::from(def.weapon_skills.rank(skill)),
                    1 + u32::from(def.level) / 7,
                    "{id}: weapon rank"
                );
            }
            // Bare: a natural attack on the roster's dice, with a rider.
            None => {
                let attack = (def.attack.as_ref())
                    .unwrap_or_else(|| panic!("{id}: neither a racial weapon nor an attack"));
                let damage = attack.attack.damage;
                let average = f64::from(damage.min() + damage.max()) / 2.0;
                let want = dice_average(level, role);
                assert!(
                    within(average, want),
                    "{id}: attack {average}, the formula {want}"
                );
                assert!(!attack.on_hit.is_empty(), "{id}: a rider");
            }
        }
    }
    for (id, _) in ROLES {
        let def = content.creature_id(id).map(|c| content.creature(c));
        assert!(
            def.is_some_and(|d| d.faction == Some(duergar)),
            "{id}: listed in ROLES but of another faction"
        );
    }
    // A notch below the drow: from level 8 to 40, two types or more in every band.
    assert_eq!(levels.iter().min(), Some(&8));
    assert_eq!(levels.iter().max(), Some(&40));
    for band in [8..=14u8, 15..=21, 22..=30, 31..=40] {
        let count = levels.iter().filter(|&&l| band.contains(&l)).count();
        assert!(count >= 2, "{count} duergar types in levels {band:?}");
    }
}

#[test]
fn the_duergar_fight_the_goblins_kin_and_the_dead_and_leave_the_cult_be() {
    let content = Content::builtin().expect("content");
    let stance = |a: &str, b: &str| {
        let id = |n: &str| content.creature_id(n).expect(n);
        content.stance(id(a), id(b))
    };
    for (a, b, want) in [
        ("duergar_slinger", "player", Stance::Hostile),
        ("duergar_crossbowman", "goblin", Stance::Hostile),
        ("duergar_veteran", "orc_warrior", Stance::Hostile),
        ("duergar_priest", "skeleton", Stance::Hostile),
        ("duergar_hulk", "hill_giant", Stance::Hostile),
        ("duergar_spider_rider", "troll", Stance::Hostile),
        ("duergar_thane", "dretch", Stance::Hostile),
        ("duergar_king", "duergar_miner", Stance::Neutral),
        ("duergar_mindbreaker", "cultist", Stance::Neutral),
        ("duergar_priest", "kobold", Stance::Neutral),
        ("duergar_veteran", "fire_salamander", Stance::Neutral),
        ("duergar_slinger", "duergar_trader", Stance::Neutral),
    ] {
        assert_eq!(stance(a, b), want, "{a} and {b}");
    }
}

#[test]
fn the_duergar_wield_their_weapons_and_the_shooters_their_ammunition() {
    use crate::gear;
    use crate::rng::Rng;
    let content = Content::builtin().expect("content");
    for &(id, role) in ROLES {
        if role == Role::Older {
            // bestiary_tests' `the_delf_humanoids_wield_their_weapons` has them.
            continue;
        }
        let creature = content.creature_id(id).expect(id);
        let def = content.creature(creature);
        let mut armed = 0;
        let mut loaded = 0;
        for seed in 0..200 {
            let pack = gear::roll(&content, &mut Rng::gear(seed, "test"), creature, 10);
            let has =
                |object: &str| (pack.items.iter()).any(|s| content.object(s.object).id == object);
            if (has("sling") && has("sling_stone")) || (has("crossbow") && has("bolt")) {
                loaded += 1;
            }
            for (i, stack) in pack.items.iter().enumerate() {
                let object = content.object(stack.object);
                if object.slot.is_some() {
                    assert!(
                        pack.equipment.holds(i),
                        "{id} {seed}: {} not held",
                        object.id
                    );
                }
                if object.attack.is_some() || object.launches.is_some() {
                    armed += 1;
                }
            }
        }
        if def.gear.racial_weapon.is_some() {
            assert!(armed >= 160, "{id}: {armed} of 200 armed");
        } else {
            assert_eq!(armed, 0, "{id}: fights bare");
        }
        if def.combat == Combat::Ranged {
            assert!(loaded >= 150, "{id}: {loaded} of 200 with ammunition");
        }
    }
}
