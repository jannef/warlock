//! The orc tribe (#455): every `faction = "orcs"` type keeps to the tribe's formulas, the undead
//! roster's (#383) hit points and roles on living stats; the orcs wield their racial weapons at a
//! rank that grows with their level, and the wargs bite by the roster's dice.

use super::Content;
use crate::stats::Stat;

/// How far a type's numbers may lie from the formulas, in percent.
const TOLERANCE: f64 = 35.0;

/// A role's adjustments (#455 "Formulas", #383's roles).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Role {
    Small,
    Standard,
    Archer,
    Quick,
    Brute,
    Caster,
}

impl Role {
    /// What the role adds to the base Strength, Dexterity, and Toughness.
    fn adjust(self) -> [f64; 3] {
        match self {
            Role::Small => [-4.0, 4.0, -4.0],
            Role::Standard => [0.0, 0.0, 0.0],
            Role::Archer => [0.0, 3.0, 0.0],
            Role::Quick => [0.0, 5.0, -1.0],
            Role::Brute => [5.0, -2.0, 5.0],
            Role::Caster => [-3.0, 0.0, 0.0],
        }
    }

    /// The factor on `base_hp`.
    fn hp(self) -> f64 {
        match self {
            Role::Small => 0.5,
            Role::Standard | Role::Archer => 1.0,
            Role::Quick => 0.9,
            Role::Brute => 1.6,
            Role::Caster => 0.8,
        }
    }
}

/// Every type of the orcs' faction and its role; `true` for the beasts (the wargs), who bite and
/// carry no `orc` tag.
const ROLES: &[(&str, Role, bool)] = &[
    ("orc_runt", Role::Small, false),
    ("orc_warrior", Role::Standard, false),
    ("orc_archer", Role::Archer, false),
    ("warg", Role::Quick, true),
    ("orc_raider", Role::Quick, false),
    ("orc_shaman", Role::Caster, false),
    ("orc_berserker", Role::Brute, false),
    ("orc_veteran", Role::Standard, false),
    ("orc_bowman", Role::Archer, false),
    ("orc_war_chief", Role::Standard, false),
    ("dire_warg", Role::Quick, true),
    ("orc_high_shaman", Role::Caster, false),
    ("orc_warlord", Role::Brute, false),
];

/// The roster's dice by average (#383 "Damage dice"), up to what a warg needs.
const LADDER: &[f64] = &[
    2.0, 2.5, 3.5, 4.5, 5.5, 7.0, 9.0, 10.5, 13.5, 14.0, 16.5, 18.0,
];

/// The formula's max HP: Toughness (12 + 0.4 L, by role) plus base_hp (2.5 L, by role).
fn max_hp(level: f64, role: Role) -> f64 {
    12.0 + 0.4 * level + role.adjust()[2] + 2.5 * level * role.hp()
}

/// The formula's bite: the ladder's nearest to `2 + 0.55 L`, flattening past L20 (quick: no step).
fn bite_average(level: f64) -> f64 {
    let target = if level <= 20.0 {
        2.0 + 0.55 * level
    } else {
        13.0 + 0.25 * (level - 20.0)
    };
    let distance = |a: f64| (a - target).abs();
    (LADDER.iter().copied())
        .min_by(|&a, &b| distance(a).total_cmp(&distance(b)))
        .expect("a ladder")
}

fn within(actual: f64, expected: f64) -> bool {
    ((actual - expected) / expected * 100.0).abs() <= TOLERANCE
}

#[test]
fn the_orcs_keep_to_the_tribes_formulas() {
    let content = Content::builtin().expect("content");
    let orcs = content.factions().id("orcs").expect("the orcs' faction");
    let guideline = content.experience().guideline();
    let mut levels = Vec::new();
    for def in (content.creature_defs().iter()).filter(|c| c.faction == Some(orcs)) {
        let id = def.id.as_str();
        let &(_, role, beast) = (ROLES.iter().find(|(r, ..)| *r == id))
            .unwrap_or_else(|| panic!("{id}: give it a role in ROLES"));
        levels.push(def.level);
        assert_eq!(
            def.xp,
            guideline.xp(def.level),
            "{id}: off the XP guideline"
        );
        let level = f64::from(def.level);
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
        let hp = f64::from(def.stats.score(Stat::Toughness)) + f64::from(def.base_hp);
        let want = max_hp(level, role);
        assert!(within(hp, want), "{id}: max HP {hp}, the formula {want:.1}");
        let tagged = content.has_tag(content.creature_id(id).expect(id), "orc");
        if beast {
            assert!(!tagged, "{id}: a beast, not an orc");
            let attack = def
                .attack
                .as_ref()
                .unwrap_or_else(|| panic!("{id}: no bite"));
            let damage = attack.attack.damage;
            let average = f64::from(damage.min() + damage.max()) / 2.0;
            let want = bite_average(level);
            assert!(
                within(average, want),
                "{id}: bite {average}, the formula {want}"
            );
        } else {
            assert!(tagged, "{id}: an orc carries the `orc` tag");
            // Its racial weapon, at the rank the formula gives (1 + L/7).
            let weapon =
                (def.gear.racial_weapon).unwrap_or_else(|| panic!("{id}: no racial weapon"));
            let skill = (content.object(weapon).skill)
                .unwrap_or_else(|| panic!("{id}: its racial weapon has no skill"));
            let rank = def.weapon_skills.rank(skill);
            assert_eq!(
                u32::from(rank),
                1 + u32::from(def.level) / 7,
                "{id}: weapon rank"
            );
        }
    }
    for (id, ..) in ROLES {
        let def = content.creature_id(id).map(|c| content.creature(c));
        assert!(
            def.is_some_and(|d| d.faction == Some(orcs)),
            "{id}: listed in ROLES but of another faction"
        );
    }
    // The low and middle levels: two types or more in each band from 3 to 30.
    for band in [3..=7u8, 8..=14, 15..=21, 22..=30] {
        let count = levels.iter().filter(|&&l| band.contains(&l)).count();
        assert!(count >= 2, "{count} orc types in levels {band:?}");
    }
}

#[test]
fn the_orcs_raid_the_kobolds_and_the_duergar_and_leave_the_goblins_be() {
    use super::Stance;
    let content = Content::builtin().expect("content");
    let stance = |a: &str, b: &str| {
        let id = |n: &str| content.creature_id(n).expect(n);
        content.stance(id(a), id(b))
    };
    for (a, b, want) in [
        ("orc_warrior", "player", Stance::Hostile),
        ("orc_warrior", "kobold", Stance::Hostile),
        ("orc_raider", "rat", Stance::Hostile),
        ("orc_veteran", "duergar_warrior", Stance::Hostile),
        ("orc_shaman", "skeleton", Stance::Hostile),
        ("warg", "orc_runt", Stance::Neutral),
        ("orc_warrior", "goblin", Stance::Neutral),
        ("orc_berserker", "ogre", Stance::Neutral),
        ("orc_archer", "giant_leech", Stance::Neutral),
        ("orc_bowman", "fire_salamander", Stance::Neutral),
    ] {
        assert_eq!(stance(a, b), want, "{a} and {b}");
    }
}

#[test]
fn the_orcs_are_shared_and_wield_their_weapons() {
    use crate::gear;
    use crate::rng::Rng;
    let content = Content::builtin().expect("content");
    for &(id, _, beast) in ROLES {
        let creature = content.creature_id(id).expect(id);
        assert_eq!(content.creature(creature).campaign, None, "{id}: shared");
        let mut armed = 0;
        let mut quivers = 0;
        for seed in 0..200 {
            let pack = gear::roll(&content, &mut Rng::gear(seed, "test"), creature, 10);
            let has =
                |object: &str| (pack.items.iter()).any(|s| content.object(s.object).id == object);
            if (has("short_bow") || has("composite_bow")) && has("arrow") {
                quivers += 1;
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
        // The racial weapon comes nine times in ten; the wargs carry nothing.
        if beast {
            assert_eq!(armed, 0, "{id}");
        } else {
            assert!(armed >= 160, "{id}: {armed} of 200 armed");
        }
        if id == "orc_archer" || id == "orc_bowman" {
            assert!(quivers >= 150, "{id}: {quivers} of 200 with bow and arrows");
        }
    }
}
