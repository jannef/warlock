//! Monster Protection by level and body (#563, `docs/design/combat-progression.md` §8): every
//! creature type's `natural.protection` is its body's at its level, the hide's `name` deciding
//! the body, unless the type fights in rolled armour or is one of the peoples (their race's
//! hide: none), or it is on the exceptions list below. The animated objects keep their own table
//! (`animated_tests`).

use super::{Content, CreatureDef};

/// A body, by what its hide is called.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Body {
    /// Nothing to cut through: spirits, flame, wind, water.
    None,
    /// Hide, skin, fur, feathers, dead flesh.
    Hide,
    /// Bones, scales, chitin, shell.
    Bony,
    /// Stone, ice, iron-hard skin.
    Armoured,
}

impl Body {
    /// The body a hide called `name` covers (`None`: a name the rule does not know).
    fn of(name: &str) -> Option<Self> {
        const NONE: &[&str] = &[
            "insubstantial form",
            "rushing wind",
            "crackling form",
            "flickering flame",
            "flickering form",
            "flowing body",
            "fey grace",
            "mud",
        ];
        const HIDE: &[&str] = &[
            "hide",
            "oily hide",
            "withered hide",
            "thick hide",
            "grey hide",
            "frosted hide",
            "hairy hide",
            "barnacled hide",
            "dark hide",
            "bony hide",
            "warty hide",
            "barbed hide",
            "ancient hide",
            "woody hide",
            "skin",
            "fur",
            "feathers",
            "dead flesh",
            "wrappings",
            "rotting scales",
        ];
        const BONY: &[&str] = &[
            "bones",
            "dry bones",
            "bone",
            "skull",
            "scales",
            "chitin",
            "shell",
            "carapace",
            "plates",
        ];
        const ARMOURED: &[&str] = &[
            "stone body",
            "stony hide",
            "iron-hard skin",
            "frost-hard skin",
            "ice",
            "slag hide",
            "cinder hide",
        ];
        [
            (NONE, Body::None),
            (HIDE, Body::Hide),
            (BONY, Body::Bony),
            (ARMOURED, Body::Armoured),
        ]
        .into_iter()
        .find(|(names, _)| names.contains(&name))
        .map(|(_, body)| body)
    }

    /// Its Protection at level `level` (§8.2's table, rounded down, at least 0).
    fn protection(self, level: u8) -> i32 {
        let level = i32::from(level);
        match self {
            Body::None => 0,
            Body::Hide => ((level - 10) / 8).max(0),
            Body::Bony => level / 6,
            Body::Armoured => level / 4 + 1,
        }
    }
}

/// The hand-tuned types (§8.3) and their Protection.
const EXCEPTIONS: &[(&str, i32)] = &[
    // The spec's early shell (Defence 1, Protection 3), and the delf's beetle with it.
    ("beetle", 3),
    ("ember_beetle", 3),
    // The roster's 5 and its resistances.
    ("stone_troll", 5),
    // Its file: "a blow sinks into it and does little" (the mud body is none).
    ("mud_golem", 5),
    // Bosses held to the bony line: hit points carry them.
    ("balor", 10),
    ("pit_fiend", 10),
];

/// The peoples: they fight in what they wear, over their race's hide (none of these has one).
const PEOPLES: &[&str] = &[
    "kobold",
    "goblin",
    "hobgoblin",
    "orc",
    "drow",
    "duergar",
    "cultist",
    "outlaw",
];

/// Types with no hide whose body is none past the hide's first point (level 18).
const BODILESS: &[&str] = &["dryad"];

/// Whether `def`'s gear may roll a piece of armour (one with Protection or an armour bonus).
fn rolls_armour(content: &Content, def: &CreatureDef) -> bool {
    (def.gear.lines.iter())
        .flat_map(|line| line.one_of.iter())
        .flat_map(|(stacks, _)| stacks.iter())
        .any(|stack| {
            let object = content.object(stack.object);
            object.protection > 0 || object.armour > 0
        })
}

/// The Protection the rule asks of `def`, and why.
fn wanted(content: &Content, def: &CreatureDef) -> (i32, String) {
    let id = def.id.as_str();
    let faction = |name: &str| content.factions().id(name);
    let tagged = |tag: &str| def.tags.iter().any(|t| t == tag);
    if let Some(&(_, pv)) = EXCEPTIONS.iter().find(|(e, _)| *e == id) {
        return (pv, "an exception".into());
    }
    let people = PEOPLES.iter().any(|t| tagged(t))
        || id == "player"
        || [faction("peaceful"), faction("merchants")].contains(&def.faction);
    if people {
        return (0, "one of the peoples".into());
    }
    if rolls_armour(content, def) {
        return (0, "armed: its armour supplies it".into());
    }
    let Some(hide) = &def.natural else {
        let hide_free = Body::Hide.protection(def.level) == 0;
        assert!(
            hide_free || BODILESS.contains(&id) || tagged("hag"),
            "{id} (level {}): no `natural`; give it a hide by its body, or list it in BODILESS",
            def.level
        );
        return (0, "no hide".into());
    };
    let body = Body::of(&hide.name)
        .unwrap_or_else(|| panic!("{id}: hide {:?} is no body of the rule's", hide.name));
    // The hags who cast: casters should not also be hard to cut (half a hide's).
    if tagged("hag") && tagged("caster") {
        return (
            Body::Hide.protection(def.level) / 2,
            "a hag who casts".into(),
        );
    }
    let mut pv = body.protection(def.level);
    // The living trolls keep the tribe's hide of 2 or more.
    if tagged("troll") && def.faction != faction("undead") {
        pv = pv.max(2);
    }
    (pv, format!("{body:?} at level {}", def.level))
}

#[test]
fn every_creature_has_its_bodys_protection() {
    let content = Content::builtin().expect("content");
    for def in content.creature_defs() {
        if def.tags.iter().any(|t| t == "animated") {
            continue;
        }
        let have = def.natural.as_ref().map_or(0, |h| h.protection);
        let (want, why) = wanted(&content, def);
        assert_eq!(
            have, want,
            "{}: Protection {have}, the rule's {want} ({why})",
            def.id
        );
    }
}

#[test]
fn the_exceptions_and_lists_name_real_types() {
    let content = Content::builtin().expect("content");
    for id in (EXCEPTIONS.iter().map(|(id, _)| id)).chain(BODILESS) {
        assert!(content.creature_id(id).is_some(), "{id}: no such creature");
    }
}

#[test]
fn the_rule_matches_the_designs_table() {
    // §8.2: L 10 / 20 / 30 / 40 / 50 / 60 / 70.
    let levels = [10, 20, 30, 40, 50, 60, 70];
    let row = |body: Body| levels.map(|l| body.protection(l));
    assert_eq!(row(Body::None), [0; 7]);
    assert_eq!(row(Body::Hide), [0, 1, 2, 3, 5, 6, 7]);
    assert_eq!(row(Body::Bony), [1, 3, 5, 6, 8, 10, 11]);
    assert_eq!(row(Body::Armoured), [3, 6, 8, 11, 13, 16, 18]);
}
