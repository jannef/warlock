//! The priest's prayer (spec §8 "Class abilities"; `docs/dev/prayer.md`): the strain and the
//! fizzle, the uses in their priority order with both faces' answers, the prayer's growth by the
//! priest's levels with its titles, and the priest's rites beside it (Consecrate, Communion with
//! the dark, Sanctuary). Every number here is hidden in the game.

use super::util::{Term, anchored, capital, link, ordinal};
use std::fmt::Write as _;
use warlock_core::content::{Content, PrayerRules, PrayerUse, TalentDef, TalentEffect};
use warlock_core::time::Duration;

/// The gaps between prayers the fizzle table measures, in seconds (with the gap that lets one
/// prayer's strain pass whole, added from the data).
const GAPS: [u64; 4] = [60, 10 * 60, 30 * 60, 60 * 60];
/// The prayers of a run the fizzle table measures (the first never fizzles: no strain yet).
const NTH: [usize; 4] = [2, 3, 5, 10];

/// The strain's numbers: what a prayer adds, how fast it passes, and the fizzle.
pub(super) fn prayer_strain(content: &Content) -> String {
    let Some(rules) = content.prayer() else {
        return "There is no prayer.\n".into();
    };
    let whole = rules.decay * u64::from(rules.strain);
    let mut out = String::from("| | |\n|---|---|\n");
    let _ = writeln!(
        out,
        "| Strain a prayer adds | {}, answered or fizzled alike (none in an unholy place for a priest who [communes with the dark](classes.md#talent-communion)) |",
        rules.strain
    );
    let _ = writeln!(
        out,
        "| Strain that passes | 1 point every {}: one prayer's is gone after {} |",
        every(rules.decay),
        whole.spoken()
    );
    let _ = writeln!(
        out,
        "| Chance a prayer fizzles | {} % for every 100 strain it finds (at most 100 %) |",
        rules.fizzle_per_100
    );
    out.push_str(
        "| After a fizzle | every prayer fizzles, with no chance drawn, until the strain has passed to 0 |\n\
         | Nothing to answer | refused: no time, no strain |\n",
    );
    out
}

/// The chance a prayer fizzles when the priest prays again and again with the same gap between,
/// none of the earlier ones having fizzled (a fizzle locks the prayer).
pub(super) fn prayer_fizzle(content: &Content) -> String {
    let Some(rules) = content.prayer() else {
        return "There is no prayer.\n".into();
    };
    let whole = (rules.decay * u64::from(rules.strain)).as_seconds();
    let mut gaps: Vec<u64> = GAPS.iter().copied().filter(|&g| g < whole).collect();
    gaps.push(whole);
    let mut out = String::from("| Praying every |");
    for n in NTH {
        let _ = write!(out, " {} prayer |", ordinal(n));
    }
    out.push_str("\n|---|");
    out.push_str(&":---:|".repeat(NTH.len()));
    out.push('\n');
    for gap in gaps {
        let _ = write!(out, "| {} |", Duration::seconds(gap).spoken());
        for n in NTH {
            let _ = write!(out, " {} % |", rules.fizzle_chance(found(rules, gap, n)));
        }
        out.push('\n');
    }
    out
}

/// The strain the `n`th prayer finds, each `gap` seconds after the one before.
fn found(rules: &PrayerRules, gap: u64, n: usize) -> u16 {
    let passed = gap / rules.decay.as_seconds().max(1);
    let mut strain: u64 = 0;
    for _ in 1..n {
        strain = (strain + u64::from(rules.strain)).saturating_sub(passed);
    }
    u16::try_from(strain).unwrap_or(u16::MAX)
}

/// The uses in the priority order: what ails the priest, how each face answers, in its words,
/// and the level that gives it.
pub(super) fn prayer_uses(content: &Content) -> String {
    let Some(rules) = content.prayer() else {
        return "There is no prayer.\n".into();
    };
    let mut out = String::from(
        "| Order | Use | What ails you | The holy powers | The dark powers | From |\n\
         |:---:|---|---|---|---|---|\n",
    );
    for (i, u) in (1..).zip(&rules.uses) {
        let (ails, holy, dark) = use_text(content, rules, u.id);
        let _ = writeln!(
            out,
            "| {i} | {} | {ails} | {holy}: \"{}\" | {dark}: \"{}\" | {} |",
            anchored(Term::PrayerUse, u.id.id(), &capital(u.id.id())),
            u.words.holy,
            u.words.dark,
            gained_at(content, u.id),
        );
    }
    out
}

/// What a use answers, and how each face answers it, in words. The rules' own text (the fixes
/// are code, `world/orison.rs` and `world/dark_face.rs`), so every use has its arm.
fn use_text(content: &Content, rules: &PrayerRules, id: PrayerUse) -> (String, String, String) {
    let status = |id: &str, name: &str| match content.status_id(id) {
        Some(s) => link(Term::Status, &content.status(s).id, name),
        None => name.to_string(),
    };
    let nearest = "and the nearest [creature](creatures.md) you see catches";
    match id {
        PrayerUse::Disease => (
            "a [disease](diseases.md) that shows (one still brewing is no problem yet)".into(),
            "every disease ends, the brewing ones too".into(),
            format!(
                "every disease ends, the brewing ones too, {nearest} each (it brews in them, with no check to resist)"
            ),
        ),
        PrayerUse::Poison => (
            status("poisoned", "Poisoned"),
            "it ends".into(),
            format!(
                "it ends, {nearest} it at the same degrees (one immune or resistant to poison takes less or none)"
            ),
        ),
        PrayerUse::Wounds => (
            format!(
                "{} or lost [hit points](hit-points.md#hit-points)",
                status("bleeding", "Bleeding")
            ),
            "the bleeding stops, hit points to full".into(),
            "the same".into(),
        ),
        PrayerUse::Release => {
            let list: Vec<String> = (rules.release.iter())
                .map(|&s| {
                    let def = content.status(s);
                    link(Term::Status, &def.id, &def.name)
                })
                .collect();
            (
                format!("any of {}", list.join(", ")),
                "every one ends".into(),
                format!(
                    "every one ends, and each is laid at its degrees on whoever laid it, if you see them on the level; whoever held you is {} instead",
                    status("webbed", "Webbed")
                ),
            )
        }
        PrayerUse::Mana => (
            "less than your full [mana](mana.md)".into(),
            "mana to full".into(),
            "the same".into(),
        ),
        PrayerUse::Hunger => (
            "[Hungry](hunger.md#how-hungry-you-are) or worse".into(),
            "fed to [Satiated](hunger.md#how-hungry-you-are)".into(),
            "fed to the brim, one short of [Bloated](hunger.md#how-hungry-you-are)".into(),
        ),
        PrayerUse::Restoration => (
            "[stat damage](diseases.md#stat-damage) or [drained points](stats.md#drained-stats)"
                .into(),
            "all mended, the drained points given back".into(),
            "the same".into(),
        ),
    }
}

/// Where a use is first given: `level 12 ([Prayer](...))` per class whose table gives it.
fn gained_at(content: &Content, id: PrayerUse) -> String {
    let mut at: Vec<String> = Vec::new();
    for class in content.classes() {
        for (level, def) in class_talents(content, &class.abilities) {
            let gives = def.effects.iter().any(|e| match e {
                TalentEffect::Prayer { uses, all, .. } => *all || uses.contains(&id),
                _ => false,
            });
            if gives {
                at.push(format!(
                    "level {level} ({}, {})",
                    super::talents::talent_link(content, &def.id, &def.name),
                    class.name.to_lowercase()
                ));
                break;
            }
        }
    }
    if at.is_empty() {
        "-".into()
    } else {
        at.join("; ")
    }
}

/// A class table's talents, with their levels.
fn class_talents<'c>(
    content: &'c Content,
    abilities: &'c [(u8, String)],
) -> impl Iterator<Item = (u8, &'c TalentDef)> + 'c {
    let book = content.talents();
    (abilities.iter()).filter_map(move |(level, id)| Some((*level, book.get(book.id(id)?))))
}

/// The prayer by the levels of each class that has one: what each talent adds, the prayer's name
/// for each face from there on, and how many things one prayer answers.
pub(super) fn prayer_levels(content: &Content) -> String {
    let mut out = String::new();
    for class in content.classes() {
        let rows: Vec<(u8, &TalentDef)> = class_talents(content, &class.abilities)
            .filter(|(_, d)| (d.effects.iter()).any(|e| matches!(e, TalentEffect::Prayer { .. })))
            .collect();
        if rows.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        let _ = writeln!(
            out,
            "**The {}**\n\n| Level | Talent | Adds | Name (holy) | Name (dark) | Answers a prayer |\n|---:|---|---|---|---|---|",
            class.name.to_lowercase()
        );
        let (mut holy, mut dark, mut answers, mut all) =
            ("-".to_string(), "-".to_string(), 1, false);
        for (level, def) in rows {
            let mut adds: Vec<String> = Vec::new();
            for e in &def.effects {
                let TalentEffect::Prayer {
                    title,
                    uses,
                    answers: more,
                    all: every,
                    ..
                } = e
                else {
                    continue;
                };
                if let Some(t) = title {
                    holy.clone_from(&t.holy);
                    dark.clone_from(&t.dark);
                }
                adds.extend(uses.iter().map(|u| link(Term::PrayerUse, u.id(), u.id())));
                if let Some(n) = more {
                    answers = answers.max(*n);
                    adds.push(format!("{n} answers a prayer"));
                }
                if *every {
                    all = true;
                    adds.push(
                        "everything at once, and every [resource](resources.md) pool filled".into(),
                    );
                }
            }
            let answers = if all {
                "all".to_string()
            } else {
                answers.to_string()
            };
            let _ = writeln!(
                out,
                "| {level} | {} | {} | {holy} | {dark} | {answers} |",
                super::talents::talent_link(content, &def.id, &def.name),
                adds.join(", ")
            );
        }
    }
    if out.is_empty() {
        out.push_str("No class prays.\n");
    }
    out
}

/// The rites of the classes that pray, beside the prayer: Consecrate, Communion with the dark,
/// and Sanctuary, by level, with the bands they work for and their exact numbers.
pub(super) fn prayer_rites(content: &Content) -> String {
    let mut out = String::new();
    for class in content.classes() {
        let prays = class_talents(content, &class.abilities)
            .any(|(_, d)| (d.effects.iter()).any(|e| matches!(e, TalentEffect::Prayer { .. })));
        if !prays {
            continue;
        }
        for (level, def) in class_talents(content, &class.abilities) {
            for e in &def.effects {
                let (name, what) = match e {
                    TalentEffect::Consecrate { mana, time } => (
                        link(Term::Ability, &def.id, &def.name),
                        format!(
                            "{mana} [mana](mana.md#spending-it) and {} of prayer in an unholy place: its curse lifted for good (see [Special rooms](cursed-and-hallowed.md#cursed-rooms))",
                            spoken(time)
                        ),
                    ),
                    TalentEffect::Communion => (
                        super::talents::talent_link(content, &def.id, &def.name),
                        "a passive: standing in a [cursed](special-rooms.md#room-cursed), [draining](special-rooms.md#room-draining), or [haunted](special-rooms.md#room-haunted) room, or on a level such a curse lies over, your prayers add no strain and the place's [Ebbing](status-effects.md#status-ebbing) never touches you; only while the dark powers answer you".into(),
                    ),
                    TalentEffect::Sanctuary {
                        mana,
                        time,
                        radius,
                        lasts,
                        regen,
                    } => {
                        let side = u32::from(*radius) * 2 + 1;
                        let pace = if regen % 100 == 0 {
                            format!("{} times as fast", regen / 100)
                        } else {
                            format!("at {regen} % of the pace")
                        };
                        (
                            link(Term::Ability, &def.id, &def.name),
                            format!(
                                "{mana} [mana](mana.md#spending-it) and {} of kneeling: every cell within {radius} of where you knelt (a {side} by {side} square) is [hallowed ground](cursed-and-hallowed.md#hallowed-ground) for {}; hostile creatures do not step onto it, and whoever stands on it [heals](healing.md#how-fast) and gets [mana back](mana.md#how-fast-it-comes-back) {pace}",
                                spoken(time),
                                spoken(lasts)
                            ),
                        )
                    }
                    _ => continue,
                };
                let works = def
                    .alignment
                    .map_or_else(|| "any".to_string(), |a| a.words());
                if out.is_empty() {
                    out.push_str(
                        "| Level | Ability | Works for | What it does |\n|---:|---|---|---|\n",
                    );
                }
                let _ = writeln!(out, "| {level} | {name} | {works} | {what} |");
            }
        }
    }
    if out.is_empty() {
        out.push_str("No class prays.\n");
    }
    out
}

/// `minute` for a minute, else its words: `10 minutes`.
fn every(d: Duration) -> String {
    let words = d.spoken();
    match words.strip_prefix("1 ") {
        Some(unit) if !unit.contains(' ') => unit.to_string(),
        _ => words,
    }
}

/// A duration of the data in words: `1m` → `1 minute`.
fn spoken(text: &str) -> String {
    text.parse::<Duration>()
        .map_or_else(|_| text.to_string(), Duration::spoken)
}
