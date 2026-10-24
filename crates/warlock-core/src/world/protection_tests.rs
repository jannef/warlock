//! Protection in the world (spec §8 "Combat"): what is worn and a hide add up (a shield adds
//! none), a hit soaked to nothing does nothing else and glances off what soaked it, weapons and
//! missiles carry their penetration, and fights against protected targets still replay.

use super::*;
use crate::action::{AttackKind, Event};
use crate::combat::Strike;
use crate::content::{ContentFiles, Role};
use crate::magic::{Affix, Magic, Rarity};
use crate::object::Stack;
use std::sync::LazyLock;

/// A lit test level beside the built-in content: the player (`@`) next to an iron tortoise
/// (`t`), hostile, whose carapace soaks everything bare hands do and whose bite always
/// poisons, and a giant rat (`r`) walled in, out of the way.
const MAP: &str = "\
############
#........#r#
#.@t.....###
#..........#
############
";

/// The iron tortoise: Protection 20 from its carapace, a weak bite (1 damage a hit, Strength
/// 1) that always poisons, and hit points enough to outlast the test.
const TORTOISE: &str = r##"
[[creature]]
id = "tortoise"
level = 1
xp = 1
name = "iron tortoise"
glyph = "t"
fg = "#808080"
faction = "vermin"
vision = "normal"
sight = 6
hearing = 20
weight = 20000
base_hp = 100
stats = { strength = 1 }
natural = { protection = 20, name = "carapace" }
[creature.attack]
damage = "1d2"
name = "bite"
verb = "bites"
on_hit = [{ status = "poisoned", chance = 100, severity = 1 }]
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("data/creatures/tortoise.toml", TORTOISE);
    files.insert(
        "levels/pvtest.toml",
        &format!(
            "name = \"Shell\"\nambient = [255, 255, 255, 0]\nmap = '''\n{MAP}'''\n\n\
             [legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"t\" = {{ terrain = \"floor\", creature = \"tortoise\" }}\n\
             \"r\" = {{ terrain = \"floor\", creature = \"rat\" }}\n"
        ),
    );
    files.insert(
        "scenarios/pvtest.toml",
        "name = \"Shell\"\nstart_level = \"pvtest\"\nhp = 30\n",
    );
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

fn world() -> World {
    World::new(CONTENT.clone(), "pvtest", None).expect("world")
}

/// The actor of creature type `id`.
fn actor_of(w: &World, id: &str) -> ActorId {
    let creature = w.content().creature_id(id).expect(id);
    (w.state().actors.values())
        .find(|a| a.creature == creature)
        .map(|a| a.id)
        .expect("on the level")
}

/// Puts `items` (object id, slot id) on the player, replacing what they carry.
fn wear(w: &mut World, items: &[(Stack, &str)]) {
    let content = w.content().clone();
    let player = w.state.player;
    let me = w.state.actors.get_mut(&player).expect("the player");
    me.inventory = items.iter().map(|(s, _)| s.clone()).collect();
    me.equipment = Default::default();
    for (i, (_, slot)) in items.iter().enumerate() {
        me.equipment.put(&[content.slot_id(slot).expect(slot)], i);
    }
}

fn one(id: &str) -> Stack {
    Stack::one(CONTENT.object_id(id).expect(id))
}

/// A Hardened `id` (+`value` Protection), unidentified.
fn hardened(id: &str, value: i32) -> Stack {
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 4,
        affixes: vec![Affix {
            id: "hardened".into(),
            tier: 2,
            value,
        }],
        name: None,
        identified: false,
        carried: 0,
    };
    let object = CONTENT.object_id(id).expect(id);
    (magic.check(CONTENT.object(object), &CONTENT)).expect("a valid item");
    Stack::with_magic(object, 1, magic)
}

#[test]
fn protection_adds_up_across_what_is_worn_and_the_hide() {
    let mut w = world();
    assert_eq!(w.perception().own_protection(), 0);
    // Chain mail 3, an iron helm, gauntlets, iron-shod boots, and a fur cloak 1 each; the
    // shield none (its is Defence only).
    wear(
        &mut w,
        &[
            (one("chain_mail"), "body"),
            (one("iron_helm"), "head"),
            (one("gauntlets"), "hands"),
            (one("iron_shod_boots"), "feet"),
            (one("fur_cloak"), "cloak"),
            (one("small_shield"), "off_hand"),
        ],
    );
    let player = w.state.player;
    assert_eq!(w.protection_of(player).base, 7);
    w.perceive(&mut Vec::new());
    assert_eq!(w.perception().own_protection(), 7);
    // A Hardened leather cap adds its affix, identified or not (the total is all the player is
    // shown); the iron helm it replaces took its 1 with it.
    wear(
        &mut w,
        &[
            (one("chain_mail"), "body"),
            (hardened("leather_cap", 2), "head"),
        ],
    );
    assert_eq!(w.protection_of(player).base, 5);
    // Identified, its description says so; the base item has none of its own.
    let mut cap = hardened("leather_cap", 2);
    cap.identify();
    let lines: Vec<String> = (crate::magic::describe::lines(&cap, &CONTENT).into_iter())
        .map(|l| l.text)
        .collect();
    assert!(
        lines.contains(&"Hardened: +2 to Protection".into()),
        "{lines:?}"
    );
    assert!(
        !lines.iter().any(|l| l.starts_with("Protection")),
        "{lines:?}"
    );
    let helm = crate::magic::describe::lines(&one("great_helm"), &CONTENT);
    assert!(helm.iter().any(|l| l.text == "Protection 2"));
    // What soaks the most names a blow it soaked: the mail.
    let me = w.state.player();
    assert_eq!(me.soaked_by(w.content(), None), Some("chain mail shirt"));
    // A creature's hide adds to what it wears; the beetle's shell is 3, the rat has none.
    let tortoise = actor_of(&w, "tortoise");
    assert_eq!(w.protection_of(tortoise).base, 20);
    assert_eq!(w.protection_of(actor_of(&w, "rat")).base, 0);
    let shell = w.state.actors[&tortoise].soaked_by(w.content(), None);
    assert_eq!(shell, Some("carapace"));
    let beetle = CONTENT.creature(CONTENT.creature_id("beetle").expect("beetle"));
    let hide = beetle.natural.as_ref().expect("a shell");
    assert_eq!((hide.defence, hide.protection), (1, 3));
}

#[test]
fn a_hit_soaked_to_nothing_does_nothing_else_but_still_trains() {
    let mut w = world();
    let tortoise = actor_of(&w, "tortoise");
    let unarmed = (CONTENT.weapon_skills().of_role(Role::Unarmed)).expect("unarmed fighting");
    let (mut soaked, mut marks) = (0, 0);
    for _ in 0..40 {
        let before = w.state.player().weapon_skills.get(unarmed);
        let mut events = Vec::new();
        w.state.messages = Default::default();
        w.player_attacks(tortoise, true, &mut events);
        let strike = events.iter().find_map(|e| match e {
            Event::Attacked { strike, .. } => *strike,
            _ => None,
        });
        let after = w.state.player().weapon_skills.get(unarmed);
        if strike == Some(Strike::Soaked) {
            soaked += 1;
            // Bare hands do at most 6 against a soak of 10 to 20: every hit glances off.
            let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
            assert_eq!(
                said.first().map(String::as_str),
                Some("Your fist glances off the iron tortoise's carapace."),
                "{said:?}"
            );
            // The blow landed: it trains as a hit does.
            assert!(after.marks > before.marks || after.rank > before.rank);
            marks += 1;
        }
        assert!(
            !matches!(strike, Some(Strike::Hit { .. })),
            "nothing gets through"
        );
    }
    assert!(soaked > 10 && marks == soaked, "{soaked} soaked");
    // No damage, no blood, no wound.
    let shell = &w.state.actors[&tortoise];
    assert_eq!(shell.hp, shell.max_hp(w.content()));
    assert!(shell.effects.is_empty());
    assert!(w.state().current().blood.is_empty());
}

#[test]
fn a_soaked_bite_does_not_poison_and_says_what_it_glanced_off() {
    // Unarmoured, the tortoise's bite hurts and poisons; in a leather jerkin (Protection 1,
    // soaking exactly 1) it glances off, every time.
    let tortoise_bites = |armoured: bool| {
        let mut w = world();
        if armoured {
            wear(&mut w, &[(one("leather_jerkin"), "body")]);
        }
        let player = w.state.player;
        let tortoise = actor_of(&w, "tortoise");
        let mut landed = 0;
        for _ in 0..30 {
            let mut events = Vec::new();
            if let Some(blow) = w.attack(tortoise, player, AttackKind::Melee, &mut events)
                && blow.strike.is_hit()
            {
                landed += 1;
                assert_eq!(blow.strike == Strike::Soaked, armoured);
            }
        }
        let me = w.state.player();
        (
            landed,
            me.hp < me.max_hp(w.content()),
            !me.effects.is_empty(),
        )
    };
    let (landed, hurt, poisoned) = tortoise_bites(false);
    assert!(landed > 0 && hurt && poisoned);
    let (landed, hurt, poisoned) = tortoise_bites(true);
    assert!(landed > 0 && !hurt && !poisoned, "landed {landed}");
    // As the turn is perceived: "The iron tortoise's bite glances off your leather jerkin."
    let mut w = world();
    wear(&mut w, &[(one("leather_jerkin"), "body")]);
    let mut said = Vec::new();
    for _ in 0..30 {
        w.apply(&Action::Wait);
        said.extend(w.messages().iter().map(|m| m.display()));
    }
    let glanced = "The iron tortoise's bite glances off your leather jerkin.";
    assert!(said.iter().any(|m| m == glanced), "{said:#?}");
    assert!(!said.iter().any(|m| m == "The iron tortoise bites you."));
}

#[test]
fn weapons_and_missiles_carry_their_penetration() {
    let mut w = world();
    let content = w.content().clone();
    let profile = |w: &World| w.state.player().melee(&content);
    assert_eq!(profile(&w).penetration, 0, "bare hands");
    wear(&mut w, &[(one("war_hammer"), "main_hand")]);
    assert_eq!(profile(&w).penetration, 2);
    wear(&mut w, &[(one("mace"), "main_hand")]);
    assert_eq!(profile(&w).penetration, 0);
    // A shot's is the launcher's and the ammunition's together; arrows have none.
    let mut bolts = one("bolt");
    bolts.count = 10;
    wear(
        &mut w,
        &[(one("crossbow"), "missile_weapon"), (bolts, "missiles")],
    );
    let shot = w.state.player().ranged(&content).expect("a crossbow ready");
    assert_eq!(shot.penetration, 2);
    let mut arrows = one("arrow");
    arrows.count = 10;
    wear(
        &mut w,
        &[(one("short_bow"), "missile_weapon"), (arrows, "missiles")],
    );
    let shot = w.state.player().ranged(&content).expect("a bow ready");
    assert_eq!(shot.penetration, 0);
    // A stiletto thrown keeps its own.
    wear(&mut w, &[(one("stiletto"), "main_hand")]);
    let throw = w.state.player().thrown(0, &content).expect("thrown");
    assert_eq!(throw.penetration, 2);
}

#[test]
fn fights_against_protection_replay_exactly() {
    let play = || {
        let mut w = world();
        let mut replay = crate::replay::Replay::new(&w);
        for action in crate::action::parse_actions("move:E*12 wait*6").expect("actions") {
            w.apply(&action);
            replay.push(&action, &w);
        }
        (w, replay)
    };
    let (a, replay) = play();
    let (b, _) = play();
    assert_eq!(a.state_hash(), b.state_hash());
    assert_eq!(a.messages(), b.messages());
    let said: Vec<String> = a.messages().iter().map(|m| m.display()).collect();
    assert!(said.iter().any(|m| m.contains("glances off")), "{said:#?}");
    let verified = (crate::replay::Replay::from_json(&replay.to_json()))
        .expect("json")
        .run(CONTENT.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), a.state_hash());
}
