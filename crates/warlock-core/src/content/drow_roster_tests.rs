//! The drow (#519): every `faction = "drow"` type keeps to the roster's formulas, the undead
//! roster's (#383) level scaling on the drow's own base (quick, frail, keen-minded), and fights
//! with a venomed natural attack, never a wielded weapon (riders ride only on natural attacks).

use super::{Affliction, Combat, Content};
use crate::alignment::Alignment;
use crate::stats::Stat;

/// How far a type's max HP and dice average may lie from the formulas, in percent.
const TOLERANCE: f64 = 35.0;

/// The roster's span: a drow at each end, none past it.
const SPAN: (u8, u8) = (14, 50);

/// A drow's role (#519 "Conventions").
#[derive(Clone, Copy, Debug, PartialEq)]
enum Role {
    Standard,
    Archer,
    Quick,
    Caster,
    Officer,
}

impl Role {
    /// What the role adds to the base Strength, Dexterity, and Toughness.
    fn adjust(self) -> [f64; 3] {
        match self {
            Role::Standard => [0.0, 0.0, 0.0],
            Role::Caster => [-3.0, 0.0, 0.0],
            Role::Archer => [0.0, 3.0, 0.0],
            Role::Quick => [0.0, 5.0, -1.0],
            Role::Officer => [2.0, 0.0, 2.0],
        }
    }

    /// The factor on `base_hp`.
    fn hp(self) -> f64 {
        match self {
            Role::Standard => 1.0,
            Role::Archer | Role::Quick => 0.9,
            Role::Caster => 0.8,
            Role::Officer => 1.1,
        }
    }

    /// The steps it moves the dice along [`LADDER`].
    fn step(self) -> i32 {
        match self {
            Role::Caster => -1,
            _ => 0,
        }
    }
}

/// Every drow and its role (#519's table).
const ROLES: &[(&str, Role)] = &[
    ("drow_warrior", Role::Standard),
    ("drow_crossbowman", Role::Archer),
    ("drow_stalker", Role::Quick),
    ("drow_mage", Role::Caster),
    ("drow_priestess", Role::Caster),
    ("drow_blademaster", Role::Quick),
    ("drow_assassin", Role::Quick),
    ("drow_arbalester", Role::Archer),
    ("drow_captain", Role::Officer),
    ("drow_archmage", Role::Caster),
    ("drow_high_priestess", Role::Caster),
    ("drow_matron", Role::Caster),
];

/// The dice the roster picks from, by average (#383 "Damage dice": 1d3 to 5d12).
const LADDER: &[f64] = &[
    2.0, 2.5, 3.5, 4.5, 5.5, 7.0, 9.0, 10.5, 13.5, 14.0, 16.5, 18.0, 19.5, 22.0, 26.0, 27.5, 32.5,
];

/// The formula's max HP: Toughness (11 + 0.4 L, by role) plus base_hp (2.5 L, by role).
fn max_hp(level: f64, role: Role) -> f64 {
    11.0 + 0.4 * level + role.adjust()[2] + 2.5 * level * role.hp()
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

fn average(damage: crate::dice::Dice) -> f64 {
    f64::from(damage.min() + damage.max()) / 2.0
}

#[test]
fn the_drow_keep_to_the_roster() {
    let content = Content::builtin().expect("content");
    let drow = content.factions().id("drow").expect("the drow faction");
    let poisoned = content.status_id("poisoned").expect("poisoned");
    let guideline = content.experience().guideline();
    let mut levels = Vec::new();
    for def in (content.creature_defs().iter()).filter(|c| c.faction == Some(drow)) {
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
        assert_eq!(def.alignment, Alignment::Evil, "{id}: evil");
        assert_eq!(def.race.as_deref(), Some("drow"), "{id}: of the drow race");
        let level = f64::from(def.level);
        let [st, dx, to] = role.adjust();
        let mind = 10.0 + 0.4 * level + if role == Role::Caster { 4.0 } else { 0.0 };
        for (stat, want) in [
            (Stat::Strength, 11.0 + 0.4 * level + st),
            (Stat::Dexterity, 12.0 + 0.25 * level + dx),
            (Stat::Toughness, 11.0 + 0.4 * level + to),
            (Stat::Intelligence, mind),
            (Stat::Willpower, mind),
            (Stat::Presence, mind),
        ] {
            let score = f64::from(def.stats.score(stat));
            assert!(
                (score - want.min(40.0)).abs() <= 1.0,
                "{id}: {stat:?} {score}, the formula {want}"
            );
        }
        let hp = f64::from(def.stats.score(Stat::Toughness)) + f64::from(def.base_hp);
        let want = max_hp(level, role);
        assert!(within(hp, want), "{id}: max HP {hp}, the formula {want:.1}");
        // A venomed natural attack at the level's dice and to-hit, and never a weapon in gear.
        let attack = (def.attack.as_ref()).unwrap_or_else(|| panic!("{id}: no attack"));
        let want = dice_average(level, role);
        let got = average(attack.attack.damage);
        assert!(within(got, want), "{id}: dice {got}, the formula {want}");
        let to_hit = if def.level >= 20 { def.level / 10 } else { 0 };
        assert_eq!(attack.attack.to_hit, i32::from(to_hit), "{id}: to-hit");
        let venom = |on_hit: &[super::OnHit]| {
            (on_hit.iter())
                .any(|h| matches!(h.what, Affliction::Status { status, .. } if status == poisoned))
        };
        assert!(venom(&attack.on_hit), "{id}: its blade is venomed");
        assert!(def.gear.racial_weapon.is_none(), "{id}: wields no weapon");
        // The archers shoot venomed bolts (a throw) at the level's dice; no one else does.
        let tagged = |t: &str| def.tags.iter().any(|x| x == t);
        assert!(tagged("drow"), "{id}: tagged `drow`");
        assert_eq!(tagged("archer"), def.throw.is_some(), "{id}: archer tag");
        if let Some(throw) = &def.throw {
            let got = average(throw.attack.attack.damage);
            assert!(within(got, want), "{id}: bolt {got}, the formula {want}");
            assert!(venom(&throw.attack.on_hit), "{id}: its bolts are venomed");
        }
        assert_eq!(
            tagged("caster"),
            def.combat == Combat::Caster,
            "{id}: caster tag"
        );
        if tagged("priest") {
            assert!(tagged("caster"), "{id}: a priestess casts");
        }
    }
    for (id, _) in ROLES {
        let def = content.creature_id(id).map(|c| content.creature(c));
        assert!(
            def.is_some_and(|d| d.faction == Some(drow)),
            "{id}: listed in ROLES but of another faction"
        );
    }
    let (low, high) = SPAN;
    assert_eq!(levels.iter().min(), Some(&low));
    assert_eq!(levels.iter().max(), Some(&high));
}

#[test]
fn the_formulas_give_the_plans_numbers() {
    // #519's table: the warrior's 52 hit points and 2d8, the matron's 131 and 4d8.
    assert_eq!(max_hp(14.0, Role::Standard).round(), 52.0);
    assert_eq!(dice_average(14.0, Role::Standard), 9.0);
    assert_eq!(max_hp(50.0, Role::Caster).round(), 131.0);
    assert_eq!(dice_average(50.0, Role::Caster), 18.0);
}

#[test]
fn the_drow_hunt_their_rivals_and_leave_the_demons_and_spiders_be() {
    use super::Stance;
    let content = Content::builtin().expect("content");
    let stance = |a: &str, b: &str| {
        let id = |n: &str| content.creature_id(n).expect(n);
        content.stance(id(a), id(b))
    };
    for (a, b, want) in [
        ("drow_warrior", "player", Stance::Hostile),
        ("drow_warrior", "kobold", Stance::Hostile),
        ("drow_crossbowman", "goblin", Stance::Hostile),
        ("drow_captain", "hobgoblin", Stance::Hostile),
        ("drow_blademaster", "orc_warrior", Stance::Hostile),
        ("drow_stalker", "duergar_warrior", Stance::Hostile),
        ("drow_priestess", "dretch", Stance::Neutral),
        ("drow_matron", "balor", Stance::Neutral),
        ("drow_warrior", "barrow_spider", Stance::Neutral),
        ("drow_mage", "skeleton", Stance::Neutral),
        ("drow_assassin", "fire_salamander", Stance::Neutral),
        ("drow_warrior", "drow_matron", Stance::Neutral),
    ] {
        assert_eq!(stance(a, b), want, "{a} and {b}");
    }
}

/// A lit corridor in the shared content with the player at `@` and the drow of `legend` on
/// its letters; the player is hard to kill.
fn range(map: &str, legend: &str) -> crate::world::World {
    let mut files = super::ContentFiles::builtin();
    files.insert(
        "campaigns/showcase/levels/drow_range.toml",
        &format!(
            "name = \"Range\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n[legend]\n\"@\" \
             = {{ terrain = \"floor\", spawn = \"player\" }}\n{legend}\n"
        ),
    );
    files.insert(
        "campaigns/showcase/scenarios/drow_range.toml",
        "name = \"Range\"\nstart_level = \"drow_range\"\nhp = 1000\n",
    );
    let content = std::sync::Arc::new(Content::load(&files).expect("content"));
    crate::world::World::new(content, "drow_range", Some(7)).expect("world")
}

#[test]
fn a_crossbowman_shoots_venomed_bolts_and_a_warrior_cuts_with_venom() {
    use crate::action::parse_actions;
    // From afar the crossbowman shoots poisoned bolts, which may send the player to sleep.
    let mut w = range(
        "##########\n#@.....c.#\n##########\n",
        "\"c\" = { terrain = \"floor\", creature = \"drow_crossbowman\" }",
    );
    for action in parse_actions("wait*40").expect("actions") {
        w.apply(&action);
    }
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    assert!(
        said.iter()
            .any(|m| m == "The drow crossbowman shoots a poisoned bolt."),
        "{said:?}"
    );
    let effects = |w: &crate::world::World, status: &str| {
        let id = w.content().status_id(status).expect(status);
        w.state().player().effects.has(id)
    };
    let said_any = |said: &[String], what: &str| said.iter().any(|m| m == what);
    assert!(
        said_any(&said, "You feel terribly drowsy.") || said_any(&said, "You feel sick."),
        "{said:?}"
    );
    // Up close, the warrior's envenomed sword poisons.
    let mut w = range(
        "######\n#@w..#\n######\n",
        "\"w\" = { terrain = \"floor\", creature = \"drow_warrior\" }",
    );
    for action in parse_actions("wait*30").expect("actions") {
        w.apply(&action);
    }
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    assert!(
        said_any(&said, "You feel sick.") || effects(&w, "poisoned"),
        "{said:?}"
    );
}

#[test]
fn the_drow_wear_their_gear_and_wield_none() {
    use crate::gear;
    use crate::rng::Rng;
    let content = Content::builtin().expect("content");
    for &(id, _) in ROLES {
        let creature = content.creature_id(id).expect(id);
        for seed in 0..200 {
            let pack = gear::roll(&content, &mut Rng::gear(seed, "test"), creature, 30);
            for (i, stack) in pack.items.iter().enumerate() {
                let object = content.object(stack.object);
                assert!(
                    object.attack.is_none() && object.launches.is_none(),
                    "{id} {seed}: armed with {}",
                    object.id
                );
                if object.slot.is_some() {
                    assert!(
                        pack.equipment.holds(i),
                        "{id} {seed}: {} not worn",
                        object.id
                    );
                }
            }
        }
    }
}
