//! The fey roster (#452): every `fey` type keeps to the plan's formulas (the undead roster's,
//! #383, by level and role) and draws its court, Seelie or Unseelie, by the plan's weights.

use super::Content;
use crate::stats::Stat;

/// How far a type's max HP and dice average may lie from the formulas, in percent.
const TOLERANCE: f64 = 35.0;

/// A role's adjustments (#383 "Conventions").
#[derive(Clone, Copy, Debug)]
enum Role {
    Brute,
    Standard,
    Quick,
    Caster,
    Small,
}

impl Role {
    /// What the role adds to the base Toughness.
    fn toughness(self) -> f64 {
        match self {
            Role::Brute => 5.0,
            Role::Standard | Role::Caster => 0.0,
            Role::Quick => -1.0,
            Role::Small => -4.0,
        }
    }

    /// The factor on `base_hp`.
    fn hp(self) -> f64 {
        match self {
            Role::Brute => 1.6,
            Role::Standard => 1.0,
            Role::Quick => 0.9,
            Role::Caster => 0.8,
            Role::Small => 0.5,
        }
    }

    /// The steps it moves the dice along [`LADDER`].
    fn step(self) -> i32 {
        match self {
            Role::Brute => 1,
            Role::Standard | Role::Quick => 0,
            Role::Caster | Role::Small => -1,
        }
    }
}

/// Every fey type: its level, role, and court weights (Seelie, Unseelie), as #452 plans them.
const ROSTER: &[(&str, u8, Role, (u32, u32))] = &[
    ("sprite", 2, Role::Small, (50, 50)),
    ("pixie", 4, Role::Small, (50, 50)),
    ("quickling", 6, Role::Quick, (25, 75)),
    ("boggle", 7, Role::Small, (50, 50)),
    ("darkling", 9, Role::Small, (50, 50)),
    ("satyr", 10, Role::Standard, (50, 50)),
    ("redcap", 12, Role::Brute, (10, 90)),
    ("blink_dog", 14, Role::Quick, (80, 20)),
    ("meenlock", 16, Role::Standard, (10, 90)),
    ("dryad", 19, Role::Caster, (90, 10)),
    ("nymph", 22, Role::Caster, (90, 10)),
    ("korred", 25, Role::Brute, (50, 50)),
    ("yeth_hound", 31, Role::Quick, (25, 75)),
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
/// moved by the role's step.
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
fn the_fey_keep_to_the_roster() {
    let content = Content::builtin().expect("content");
    let factions = content.factions();
    let seelie = factions.id("seelie").expect("the Seelie court");
    let unseelie = factions.id("unseelie").expect("the Unseelie court");
    let guideline = content.experience().guideline();
    let fey: Vec<_> = (content.creature_defs().iter())
        .filter(|c| c.tags.iter().any(|t| t == "fey"))
        .collect();
    assert_eq!(fey.len(), ROSTER.len(), "every fey type is on the roster");
    for def in fey {
        let id = def.id.as_str();
        let &(_, level, role, (s, u)) = (ROSTER.iter())
            .find(|(r, ..)| *r == id)
            .unwrap_or_else(|| panic!("{id}: put it on ROSTER"));
        assert_eq!(def.level, level, "{id}: level");
        assert_eq!(
            def.xp,
            guideline.xp(def.level),
            "{id}: off the XP guideline"
        );
        assert_eq!(
            def.factions,
            vec![(seelie, s), (unseelie, u)],
            "{id}: court weights"
        );
        let level = f64::from(def.level);
        let hp = f64::from(def.stats.score(Stat::Toughness)) + f64::from(def.base_hp);
        let want = max_hp(level, role);
        assert!(within(hp, want), "{id}: max HP {hp}, the formula {want:.1}");
        let damage = (def.attack.as_ref())
            .unwrap_or_else(|| panic!("{id}: no attack"))
            .attack
            .damage;
        let average = f64::from(damage.min() + damage.max()) / 2.0;
        let want = dice_average(level, role);
        assert!(
            within(average, want),
            "{id}: dice average {average}, the formula {want}"
        );
    }
}

#[test]
fn the_courts_stand_as_planned() {
    use super::Stance::{Hostile, Neutral};
    let content = Content::builtin().expect("content");
    let f = content.factions();
    let id = |n: &str| f.id(n).unwrap_or_else(|| panic!("faction {n}"));
    let (player, seelie, unseelie, wrathful) = (
        id("player"),
        id("seelie"),
        id("unseelie"),
        id("wrathful_fey"),
    );
    assert_eq!(f.stance(seelie, player), Neutral);
    assert_eq!(f.stance(unseelie, player), Hostile);
    assert_eq!(f.stance(seelie, unseelie), Hostile);
    assert_eq!(f.stance(wrathful, player), Hostile);
    assert_eq!(f.stance(wrathful, unseelie), Hostile);
    assert_eq!(f.stance(wrathful, seelie), Neutral);
    assert_eq!(f.def(seelie).provoked, Some(wrathful));
    assert_eq!(f.def(unseelie).provoked, None);
    // The courts keep to their own quarrel: every other faction is neutral to them.
    for (i, other) in f.defs().iter().enumerate() {
        let other_id = super::FactionId(i as u16);
        if ["player", "seelie", "unseelie", "wrathful_fey"].contains(&other.id.as_str()) {
            continue;
        }
        for court in [seelie, unseelie, wrathful] {
            assert_eq!(
                f.stance(court, other_id),
                Neutral,
                "{} and a court",
                other.id
            );
        }
    }
    // A type counts as its heaviest court: the even ones as the first by id, the Seelie.
    let court_of = |t: &str| content.creature(content.creature_id(t).expect(t)).faction;
    assert_eq!(court_of("redcap"), Some(unseelie));
    assert_eq!(court_of("dryad"), Some(seelie));
    assert_eq!(court_of("pixie"), Some(seelie));
}

#[test]
fn factions_are_checked_when_content_loads() {
    let cases = [
        (
            "factions = { seelie = 50, elves = 50 }",
            "unknown faction `elves`",
        ),
        ("factions = { seelie = 0, unseelie = 50 }", "not 1 to 1000"),
        (
            "faction = \"seelie\"\nfactions = { seelie = 50, unseelie = 50 }",
            "replaces `faction`",
        ),
    ];
    for (line, expected) in cases {
        let mut files = super::ContentFiles::builtin();
        files.insert(
            "data/creatures/zz_test_fey.toml",
            &format!(
                "[[creature]]\nid = \"zz_test_fey\"\nname = \"test fey\"\nglyph = \"f\"\n\
                 fg = \"#ffffff\"\nweight = 1000\n{line}\n"
            ),
        );
        let err = Content::load(&files).expect_err(line).to_string();
        assert!(err.contains(expected), "{line}: {err}");
    }
}
