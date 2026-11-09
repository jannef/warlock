//! Miscasts (spec §8 "Spells": "Miscasts", #1271): the Spellcraft check's difficulty by spell
//! level and its chance of a miscast by bonus, what Spellcraft gives it, and the whole miscast
//! table with its exact ranges.

use std::fmt::Write as _;
use warlock_core::check;
use warlock_core::content::{Content, MishapEffect, ZoneKind};
use warlock_core::skills::Effect;

/// The bonuses the chance table shows a column for.
const BONUSES: [i32; 8] = [0, 3, 6, 10, 15, 20, 30, 40];

/// The spell levels it shows a row for.
const LEVELS: [u8; 10] = [0, 1, 3, 6, 11, 16, 20, 24, 30, 35];

/// The Spellcraft values the skill table shows.
const SKILL_VALUES: [u16; 9] = [0, 17, 20, 40, 60, 80, 100, 150, 200];

/// The difficulty by spell level, and the chance of a miscast at each bonus; then what
/// Spellcraft adds to the check and takes off the total.
pub(super) fn chances(content: &Content) -> String {
    let Some(rules) = content.miscast_rules() else {
        return String::new();
    };
    let mut out = String::from("| Spell level | Difficulty |");
    for b in BONUSES {
        let _ = write!(out, " Bonus {b:+} |");
    }
    out.push_str("\n|:---:|:---:|");
    out.push_str(&":---:|".repeat(BONUSES.len()));
    out.push('\n');
    for level in LEVELS {
        let difficulty = rules.difficulty(level, 0, 1);
        let _ = write!(out, "| {level} | {difficulty} |");
        for b in BONUSES {
            let fails = 144 - check::ways(b, difficulty);
            let percent = f64::from(fails) * 100.0 / 144.0;
            let _ = write!(out, " {percent:.1} % |");
        }
        out.push('\n');
    }
    let benefit = |effect: Effect| {
        (content.skills().iter()).find_map(|s| {
            (s.benefits.iter())
                .find(|b| b.effect == effect)
                .map(|b| (s.name.as_str(), b))
        })
    };
    if let (Some((skill, cast)), Some((_, ward))) = (benefit(Effect::Cast), benefit(Effect::Ward)) {
        let _ = write!(
            out,
            "\n| {skill} | Added to the check | Taken off the total |\n|:---:|:---:|:---:|\n"
        );
        for value in SKILL_VALUES {
            let _ = writeln!(
                out,
                "| {value} | +{} | {} |",
                cast.at(value),
                ward.at(value)
            );
        }
    }
    out
}

/// The whole miscast table, in order: each entry's range of totals, what it does (the protected
/// ones marked), and what you are told.
pub(super) fn table(content: &Content) -> String {
    let Some(rules) = content.miscast_rules() else {
        return String::new();
    };
    let mut out = format!(
        "The total is {die} + {per} for each point the check missed by − what wards you. It is \
         never below 1. Results below {safe} (marked *protected*) cannot kill you: their harm \
         always leaves you at least 1 hit point.\n\n\
         | Total | What happens | You are told |\n|:---:|---|---|\n",
        die = rules.die,
        per = rules.per_margin,
        safe = rules.safe_below,
    );
    for m in content.miscasts() {
        let protected = if rules.protects(m) {
            " (*protected*)"
        } else {
            ""
        };
        let range = match m.to {
            Some(to) if to == m.from => m.from.to_string(),
            Some(to) => format!("{}–{to}", m.from),
            None => format!("{} or more", m.from),
        };
        let _ = writeln!(
            out,
            "| {range} | {}{protected} | {} |",
            effect_text(content, m.effect, rules.protects(m)),
            m.message
        );
    }
    let stances: Vec<String> = (content.stances().all().iter())
        .filter(|s| s.miscast != 0)
        .map(|s| format!("{} {:+}", s.name, -s.miscast))
        .collect();
    if !stances.is_empty() {
        let _ = writeln!(
            out,
            "\nYour [stance](stances.md#stances) counts too, added to the total: {}.",
            stances.join(", ")
        );
    }
    out
}

/// What an entry's effect does, in words.
fn effect_text(content: &Content, effect: MishapEffect, protected: bool) -> String {
    let kills = if protected {
        "never below 1 hit point"
    } else {
        "it can kill"
    };
    let near = warlock_core::world::MISCAST_NEAR;
    match effect {
        MishapEffect::Nothing | MishapEffect::Learn => "nothing more: a harmless glamour".into(),
        MishapEffect::Status { status, severity } => {
            format!("you are {} ×{severity}", content.status(status).name)
        }
        MishapEffect::Backfire { full: false } => {
            format!(
                "the spell goes off on you at its base strength: {}",
                backfire_words(false)
            )
        }
        MishapEffect::Backfire { full: true } => {
            format!(
                "the spell goes off on you at the full strength you cast it with: {}",
                backfire_words(true)
            )
        }
        MishapEffect::Mana { percent } => format!("{percent} % of your mana left is lost"),
        MishapEffect::Teleport => "you are flung to a random free place on the level".into(),
        MishapEffect::LoseMark => "a stat training mark is lost".into(),
        MishapEffect::Drain {
            stat,
            points,
            difficulty,
            times,
        } => super::learning::drain_words(stat, points, difficulty, times),
        MishapEffect::StatDamage { stat, points } => format!(
            "{} takes {points} [stat damage](diseases.md#stat-damage), which mends a point a day",
            stat.map_or("a random stat", |s| s.name())
        ),
        MishapEffect::Summon { creature } => {
            format!(
                "a hostile {} appears next to you",
                content.creature(creature).name
            )
        }
        MishapEffect::Mark { stat } => format!("a training mark of {}", stat.name()),
        MishapEffect::RankLoss { ranks } => format!(
            "you know the spell {ranks} [rank{}](magic.md#spell-ranks) lower, never below rank 1",
            if ranks == 1 { "" } else { "s" }
        ),
        MishapEffect::Forget { spells: Some(1) } => {
            "you [forget](#forgotten-spells) the spell".into()
        }
        MishapEffect::Forget { spells: Some(n) } => format!(
            "you [forget](#forgotten-spells) the spell and {} more of the spells you know, \
             drawn at random",
            n - 1
        ),
        MishapEffect::Forget { spells: None } => {
            "you [forget](#forgotten-spells) every spell you know".into()
        }
        MishapEffect::Harm { dice, damage } => match damage {
            Some(d) => format!(
                "{dice} {} damage to you; {kills}",
                content.damage_type(d).name.to_lowercase()
            ),
            None => format!("{dice} damage of the spell's own type to you; {kills}"),
        },
        MishapEffect::Refund => "the mana the spell cost comes back (a boon)".into(),
        MishapEffect::Twice => {
            "the spell goes off a second time at the same target, free (a boon)".into()
        }
        MishapEffect::Swap => {
            format!("you change places with someone within {near} cells in a clear line")
        }
        MishapEffect::Boon { status, severity } => format!(
            "someone within {near} cells in a clear line, or you, drawn at random, is {} \
             ×{severity}",
            content.status(status).name
        ),
        MishapEffect::Noise { loudness } => format!(
            "a loud noise (loudness {loudness}) that creatures nearby may [hear](hearing.md)"
        ),
        MishapEffect::Zone {
            kind,
            radius,
            turns,
        } => format!(
            "a [zone](spellcasting.md#zones) of {} lies around you for {turns} turns, radius {radius}",
            zone_name(kind)
        ),
        MishapEffect::Portal { gate } => format!(
            "stairs down into a [Pit](pits.md) of its own open beside you, and a hostile {} at their head; \
             on the overworld or a Pit's own floor, only the gate",
            content.creature(gate).name
        ),
    }
}

/// What a backfire does by the spell's kind (`world/backfire.rs`), at its base or `full`.
fn backfire_words(full: bool) -> String {
    let dice = if full {
        "level ÷ 2 + 1"
    } else {
        "level ÷ 3 + 1"
    };
    format!(
        "a damage spell bursts around you, a hex falls on you, a heal mends the nearest foe you \
         see, a buff takes hold of that foe, and a summon's creatures come hostile to everyone, \
         you, your foes, and one another; \
         any other spell, a heal or buff with no foe in sight, or a summon with no room deals \
         ({dice})d6 force to you, by the spell's level"
    )
}

/// A zone kind in words.
fn zone_name(kind: ZoneKind) -> &'static str {
    match kind {
        ZoneKind::Fire => "fire",
        ZoneKind::Fouled => "foul air",
        ZoneKind::Slick => "slick ground",
        ZoneKind::Dark => "darkness",
        ZoneKind::Smoke => "smoke that blocks sight",
    }
}
