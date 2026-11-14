//! The classes for someone choosing one (#1536, #1527): the wiki's own order of the classes
//! (fighters, casters, skulkers, everyday folk), each class's role, how hard it is for a new
//! player, and a line on how it plays, beside the numbers from data: the at-a-glance table and
//! the card at the top of each class's page.
//!
//! Role, difficulty, and the line are the wiki's judgement, not game data, so they live here. A
//! class without an entry fails the wiki build.

use super::character::class_page;
use super::kits;
use super::util::stat_link;
use std::fmt::Write as _;
use warlock_core::content::{ClassDef, Content, TalentEffect};
use warlock_core::stats::Stat;
use warlock_core::talent::MagicEffect;

/// The four kinds of class, in the wiki's order.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Fighter,
    Caster,
    Skulker,
    Folk,
}

/// What the wiki says of one class.
struct Guide {
    id: &'static str,
    kind: Kind,
    role: &'static str,
    difficulty: &'static str,
    plays: &'static str,
}

const GUIDES: &[Guide] = &[
    Guide {
        id: "ascetic",
        kind: Kind::Fighter,
        role: "Fighter, bare hands",
        difficulty: "Normal",
        plays: "Fights unarmed and unarmoured, quick on their feet, spending ki on extra blows and stuns.",
    },
    Guide {
        id: "berserker",
        kind: Kind::Fighter,
        role: "Fighter",
        difficulty: "Easy",
        plays: "The most hit points of any class; rages for harder blows and wades in.",
    },
    Guide {
        id: "paladin",
        kind: Kind::Fighter,
        role: "Fighter, holy",
        difficulty: "Easy",
        plays: "Sword and shield; smites foes, the undead and fiends hardest, and heals with zeal.",
    },
    Guide {
        id: "ranger",
        kind: Kind::Fighter,
        role: "Fighter, archer",
        difficulty: "Normal",
        plays: "Shoots from a distance with a bow, beside an animal companion that holds the foe.",
    },
    Guide {
        id: "warrior",
        kind: Kind::Fighter,
        role: "Fighter",
        difficulty: "Easy",
        plays: "Armour, a sword and a shield, and steady blows. The simplest class to start with.",
    },
    Guide {
        id: "druid",
        kind: Kind::Caster,
        role: "Caster, nature",
        difficulty: "Normal",
        plays: "Staff and sling, spells of storm and frost, and beasts called to fight beside them.",
    },
    Guide {
        id: "priest",
        kind: Kind::Caster,
        role: "Caster, healer",
        difficulty: "Normal",
        plays: "A mace and a shield, and a prayer that mends their wounds and more, for no mana.",
    },
    Guide {
        id: "witch",
        kind: Kind::Caster,
        role: "Caster, hexes",
        difficulty: "Hard",
        plays: "Frail; lays hexes from afar and lets her familiar harry what she has hexed.",
    },
    Guide {
        id: "wizard",
        kind: Kind::Caster,
        role: "Caster",
        difficulty: "Hard",
        plays: "The most mana and the fewest hit points; kills with spells before anything gets close.",
    },
    Guide {
        id: "bard",
        kind: Kind::Skulker,
        role: "Support, all-rounder",
        difficulty: "Hard",
        plays: "A little of everything: songs, many skills, and any weapon, but frail. Stops at level 30.",
    },
    Guide {
        id: "thief",
        kind: Kind::Skulker,
        role: "Sneak",
        difficulty: "Hard",
        plays: "Strikes from hiding and builds openings on one foe, then spends them on one deep blow.",
    },
    Guide {
        id: "aristocrat",
        kind: Kind::Folk,
        role: "Everyday folk",
        difficulty: "Joke",
        plays: "Soft and unfit for this, but money and finds from home wait in town at every level.",
    },
    Guide {
        id: "commoner",
        kind: Kind::Folk,
        role: "Everyday folk",
        difficulty: "Hard",
        plays: "A club and a few turnips. Levels come quickest, and their bonus talents come from any tree.",
    },
    Guide {
        id: "craftsperson",
        kind: Kind::Folk,
        role: "Everyday folk, maker",
        difficulty: "Hard",
        plays: "Makes, mends, and takes apart gear with two trades of their choice; a poor fighter.",
    },
    Guide {
        id: "merchant",
        kind: Kind::Folk,
        role: "Everyday folk",
        difficulty: "Joke",
        plays: "Weak in a fight; their gold grows with a business partner in town.",
    },
];

/// What the wiki says of class `id`; a class without an entry is a bug in this file.
fn guide(id: &str) -> &'static Guide {
    (GUIDES.iter())
        .find(|g| g.id == id)
        .unwrap_or_else(|| panic!("class `{id}` has no entry in xtask/src/wiki/class_guide.rs"))
}

/// The classes in the wiki's order: fighters, casters, skulkers, everyday folk, each kind by
/// name. Every class table and the classes in the sidebar follow it.
pub(super) fn ordered(content: &Content) -> Vec<&ClassDef> {
    let mut classes: Vec<&ClassDef> = content.classes().iter().collect();
    classes.sort_by(|a, b| (guide(&a.id).kind, &a.name).cmp(&(guide(&b.id).kind, &b.name)));
    classes
}

/// The class's key stats: those it rolls dice for, at least one on average ("Strength,
/// Toughness"); "none" for a class with none.
fn key_stats(class: &ClassDef) -> String {
    let stats: Vec<String> = (Stat::ALL.iter())
        .filter(|&&s| {
            let dice = class.stats[s];
            !dice.is_fixed() && dice.min() + dice.max() >= 2
        })
        .map(|&s| stat_link(s))
        .collect();
    if stats.is_empty() {
        "none".into()
    } else {
        stats.join(", ")
    }
}

/// The spells its level table gives: "4, from level 1"; "none".
fn spells(content: &Content, class: &ClassDef) -> String {
    let book = content.talents();
    let mut levels: Vec<u8> = Vec::new();
    for (level, talent) in &class.abilities {
        let Some(id) = book.id(talent) else { continue };
        for effect in &book.get(id).effects {
            if matches!(effect, TalentEffect::Magic(MagicEffect::Spell { .. })) {
                levels.push(*level);
            }
        }
    }
    match levels.iter().min() {
        None => "none".into(),
        Some(first) => format!("{}, from level {first}", levels.len()),
    }
}

/// The at-a-glance table of every class (#1536), in the wiki's order: role, difficulty, key
/// stats, hit points, mana, the spells it is given, and how it plays.
pub(super) fn glance(content: &Content) -> String {
    let mut out = String::from(
        "| Class | Role | Difficulty | Key stats | Hit points | Mana | Spells given | How it plays |\n\
         |---|---|---|---|:---:|:---:|---|---|\n",
    );
    for class in ordered(content) {
        let g = guide(&class.id);
        let _ = writeln!(
            out,
            "| [{}]({}) | {} | {} | {} | {} | {} | {} | {} |",
            class.name,
            class_page(&class.id),
            g.role,
            g.difficulty,
            key_stats(class),
            class.hp,
            class.mana,
            spells(content, class),
            g.plays,
        );
    }
    out
}

/// Each class's card, the top of its page, between mdBook anchors named by its id
/// (`{{#include generated/class-card.md:warrior}}`): what it is, then one row of the
/// at-a-glance table, then its starting kit.
pub(super) fn cards(content: &Content) -> String {
    let mut out = String::new();
    for class in ordered(content) {
        let g = guide(&class.id);
        let _ = writeln!(out, "<!-- ANCHOR: {} -->\n", class.id);
        let _ = writeln!(out, "*{}*\n", class.description);
        out.push_str(
            "| Role | Difficulty | Key stats | Hit points | Mana | Spells given |\n\
             |---|---|---|:---:|:---:|---|\n",
        );
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} |\n",
            g.role,
            g.difficulty,
            key_stats(class),
            class.hp,
            class.mana,
            spells(content, class),
        );
        let _ = writeln!(out, "**Plays like:** {}\n", g.plays);
        let _ = writeln!(
            out,
            "**Starts with:** {} (see [Starting kit](starting-kit.md#what-each-class-starts-with)).\n",
            kits::class_kit(content, class)
        );
        let _ = writeln!(out, "<!-- ANCHOR_END: {} -->\n", class.id);
    }
    out
}
