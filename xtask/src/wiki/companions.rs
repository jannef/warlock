//! The animal companion (spec §8 "Allies"; `data/companions.toml`): the starting beasts by race
//! and the rares with their odds, the growth table at the cadence levels for every tameable
//! beast, through the game's own code, and the rules in words.

use super::creatures::bare;
use super::order;
use super::text::{Term, damage_text, link, signed};
use std::fmt::Write as _;
use warlock_core::burden::Burden;
use warlock_core::combat::Category;
use warlock_core::content::{Content, CreatureId, RaceBeasts};
use warlock_core::world::{Ally, AllySource, CompanionData, Order};

/// The levels the growth table is printed at.
const GRADES: [u8; 7] = [1, 10, 20, 30, 40, 50, 60];

pub(super) fn beast(content: &Content, id: CreatureId) -> String {
    let def = content.creature(id);
    format!(
        "{} ({})",
        link(Term::Creature, &def.id, &def.name),
        def.level
    )
}

/// Every race's starting beasts, then the rares with the odds.
pub(super) fn starts(content: &Content) -> String {
    let Some(rules) = content.companions() else {
        return String::new();
    };
    starts_table(
        content,
        &rules.start,
        "ranger",
        "; the ranger's starting beast is at least their own level from the first turn (see the \
         growth table below)",
    )
}

/// Every race's starting beasts of `start`, then the rares with the odds, for a `class`'s
/// start; `more` ends the last sentence (the companion's and the familiar's tables, #1567).
pub(super) fn starts_table(
    content: &Content,
    start: &RaceBeasts,
    class: &str,
    more: &str,
) -> String {
    let mut out = String::from("| Race | Starting beasts |\n|---|---|\n");
    for (race, list) in &start.by_race {
        let name = content.race(race).map_or(race.clone(), |r| r.name.clone());
        let beasts: Vec<String> = list.iter().map(|&b| beast(content, b)).collect();
        let _ = writeln!(
            out,
            "| {} | {} |",
            link(Term::Race, race, &name),
            beasts.join(", ")
        );
    }
    let rares: Vec<String> = start.rare.iter().map(|&b| beast(content, b)).collect();
    let _ = write!(
        out,
        "\nAt {} in a thousand ({:.1} %), any race's {class} begins with one of the rares instead, \
         each as likely: {}. The beast's level is in brackets{more}.\n",
        start.rare_per_mille,
        f64::from(start.rare_per_mille) / 10.0,
        rares.join(", ")
    );
    out
}

/// Every tameable beast at the cadence levels: hit points, the blow, Defence, and Protection,
/// read from the rules as a companion of that grade would be.
pub(super) fn growth(content: &Content) -> String {
    let mut beasts: Vec<CreatureId> = (order::creatures(content).into_iter())
        .filter(|(_, c)| c.tameable)
        .map(|(id, _)| id)
        .collect();
    beasts.sort_by_key(|&id| {
        let def = content.creature(id);
        (def.level, def.name.clone())
    });
    growth_table(content, &beasts)
}

/// `beasts` at the cadence levels, as a kept beast of that grade would be (a companion's table,
/// a familiar's: one rule).
pub(super) fn growth_table(content: &Content, beasts: &[CreatureId]) -> String {
    let Some(rules) = content.companions() else {
        return String::new();
    };
    let mut out = String::from("| Beast | Level |");
    for grade in GRADES {
        let _ = write!(out, " At {grade} |");
    }
    out.push_str("\n|---|:---:|");
    for _ in GRADES {
        out.push_str("---|");
    }
    out.push('\n');
    for &id in beasts {
        let def = content.creature(id);
        let _ = write!(
            out,
            "| {} | {} |",
            link(Term::Creature, &def.id, &def.name),
            def.level
        );
        for grade in GRADES {
            let mut actor = bare(content, id);
            actor.ally = Some(Ally {
                master: actor.id,
                source: AllySource::Companion(CompanionData {
                    name: String::new(),
                    grade: rules.growth.grade(grade, def.level),
                    scent: None,
                    quarry: None,
                }),
                order: Order::Follow,
                retinue: false,
                until: None,
            });
            let hp = actor.max_hp(content);
            let blow = actor.melee(content);
            let defence = actor.defence(content, Burden::Unburdened, None);
            let natural = defence.bonus(Category::Natural);
            let protection = actor.protection_plus(content, None, 0).against(None);
            let _ = write!(
                out,
                " {hp} hp, {} to hit, {}, Defence {}, Protection {protection} |",
                signed(blow.to_hit),
                damage_text(blow.damage, blow.bonus),
                if natural != 0 {
                    format!("{} ({} natural)", defence.total(), signed(natural))
                } else {
                    defence.total().to_string()
                }
            );
        }
        out.push('\n');
    }
    out
}

/// The companion's rules in words and numbers.
pub(super) fn rules(content: &Content) -> String {
    let Some(r) = content.companions() else {
        return String::new();
    };
    let g = r.growth;
    let rows = [
        (
            "The start",
            format!(
                "a ranger begins with a beast of their race's list, or at {} in a thousand one of the rares",
                r.start.rare_per_mille
            ),
        ),
        (
            "Its level",
            format!(
                "{} % of yours, never below its kind's own, re-set at every level you gain",
                g.grade_percent
            ),
        ),
        (
            "Hit points",
            format!("+{} per level above its kind's", g.hp_per_level),
        ),
        (
            "To hit",
            format!("+1 per {} levels above its kind's", g.to_hit_every),
        ),
        (
            "Damage",
            format!(
                "+1 per {} levels above its kind's, and one more die of its attack per {}",
                g.damage_bonus_every, g.damage_dice_every
            ),
        ),
        (
            "Defence",
            format!(
                "+1 natural armour per {} levels above its kind's",
                g.defence_every
            ),
        ),
        (
            "Protection",
            "its body's at its level (a hide (level − 10) ÷ 8, scales or a shell level ÷ 6, stone \
             level ÷ 4 + 1), never below its kind's own"
                .to_string(),
        ),
        (
            "Strength, Dexterity, Toughness",
            format!("+1 per {} levels above its kind's", g.stat_every),
        ),
        (
            "Names",
            format!("one of {}: {}", r.names.len(), r.names.join(", ")),
        ),
    ];
    let mut out = String::from("| Rule | What it is |\n|---|---|\n");
    for (name, text) in rows {
        let _ = writeln!(out, "| {name} | {text} |");
    }
    out
}

/// The modifiers the taming table is printed at: a little of everything up to a master's.
const TAME_EDGES: [i32; 7] = [0, 2, 4, 6, 8, 10, 15];

/// Taming (`world/tame.rs`): the check in words, then every tameable beast by level with its
/// difficulty and the chance at a range of modifiers (2d12 + the modifier against it,
/// `check::ways`).
pub(super) fn taming(content: &Content) -> String {
    let Some(rules) = content.companions() else {
        return String::new();
    };
    let t = rules.tame;
    let handle = (content.skills().iter())
        .find(|s| (s.benefits.iter()).any(|b| b.effect == warlock_core::skills::Effect::Tame));
    let at = |value: u16| {
        handle
            .and_then(|s| {
                (s.benefits.iter()).find(|b| b.effect == warlock_core::skills::Effect::Tame)
            })
            .map_or(0, |b| b.at(value))
    };
    let mut out = format!(
        "A try is {} turns beside the beast, then one check: 2d12 + your Willpower modifier + \
         Handle animal's taming bonus (+{} at 50, +{} at 100, +{} at 200), {} if you carry food \
         (one of the first food you carry is eaten) + the ranger's Animal companion bonus, against \
         {} + {} × the beast's level. A natural 2 always fails and a natural 24 always succeeds. \
         The chance of a try, by the modifier:\n\n| Beast (level) | Difficulty |",
        t.turns,
        at(50),
        at(100),
        at(200),
        signed(t.food),
        t.difficulty,
        t.per_level
    );
    for edge in TAME_EDGES {
        let _ = write!(out, " {} |", signed(edge));
    }
    out.push_str("\n|---|---:|");
    out.push_str(&"---:|".repeat(TAME_EDGES.len()));
    out.push('\n');
    let mut beasts: Vec<(CreatureId, &warlock_core::content::CreatureDef)> =
        (order::creatures(content).into_iter())
            .filter(|(_, c)| c.tameable)
            .collect();
    beasts.sort_by(|a, b| a.1.level.cmp(&b.1.level).then(a.1.name.cmp(&b.1.name)));
    for (id, def) in beasts {
        let difficulty = t.difficulty + t.per_level * i32::from(def.level);
        let _ = write!(out, "| {} | {difficulty} |", beast(content, id));
        for edge in TAME_EDGES {
            let ways = warlock_core::check::ways(edge, difficulty);
            let _ = write!(out, " {} |", super::combat::percent(ways, 144));
        }
        out.push('\n');
    }
    out
}
