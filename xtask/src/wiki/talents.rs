//! Talents (spec §8 "Talents"): every tree with its talents, their prerequisites and their rules
//! in full (the game says them in words), and the coverage matrix by system.

use super::text::{Term, anchored, link, signed};
use std::fmt::Write as _;
use warlock_core::content::{
    Content, Faced, Need, OverTime, PrayerUse, Role, SignActive, SmiteRiderKind, SongOn, SpellData,
    SpellModSpec, StatusId, Table, TalentDef, TalentEffect, TreeId,
};
use warlock_core::noise::SONG_NOISE;

/// Every tree in display order: a heading, its description, and its talents (name, ranks, what
/// each rank needs, what it does as the rule, and the systems it touches).
pub(super) fn talents(content: &Content) -> String {
    let book = content.talents();
    let mut out = String::new();
    for (i, tree) in (0u8..).zip(&book.trees) {
        let _ = writeln!(out, "### {}\n\n{}\n", tree.name, tree.description);
        let rows: Vec<&TalentDef> = book.of_tree(TreeId(i)).map(|(_, t)| t).collect();
        if rows.is_empty() {
            out.push_str("No talents yet.\n\n");
            continue;
        }
        out.push_str("| Talent | Ranks | Needs | Effect | Systems |\n|---|---:|---|---|---|\n");
        for def in rows {
            let systems: Vec<&str> = (def.systems.iter())
                .map(|s| book.system(*s).name.as_str())
                .collect();
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} |",
                anchored(Term::Talent, &def.id, &name(def)),
                def.rank_count(),
                needs_text(content, def),
                effects_text(content, def),
                systems.join(", ")
            );
        }
        out.push('\n');
    }
    out
}

/// The talents each class picks at creation (spec §8 "Talents"): everyone's and the class's
/// more, by class name.
pub(super) fn creation(content: &Content) -> String {
    let base = content.experience().creation_talents();
    let mut out = String::from("| Class | Talents at creation |\n|---|---:|\n");
    let mut classes: Vec<_> = content.classes().iter().collect();
    classes.sort_by(|a, b| a.name.cmp(&b.name));
    for class in classes {
        let n = base.saturating_add(class.creation_talents);
        let _ = writeln!(out, "| {} | {n} |", class.name);
    }
    out
}

/// The heirlooms (#697: Father's sword): each race's weapon, races by name, then the one every
/// other race gets.
pub(super) fn heirlooms(content: &Content) -> String {
    let mut out = String::from("| Race | Heirloom |\n|---|---|\n");
    let object = |id: &str| {
        (content.object_id(id)).map_or_else(|| id.to_string(), |o| content.object(o).name.clone())
    };
    for (_, def) in content.talents().iter() {
        for effect in &def.effects {
            let TalentEffect::Heirloom {
                weapons, fallback, ..
            } = effect
            else {
                continue;
            };
            let mut races: Vec<_> = content.races().list.iter().collect();
            races.sort_by(|a, b| a.name.cmp(&b.name));
            let mut others = false;
            for race in races {
                match weapons.get(&race.id) {
                    Some(weapon) => {
                        let _ = writeln!(
                            out,
                            "| {} | {} |",
                            link(Term::Race, &race.id, &race.name),
                            object(weapon)
                        );
                    }
                    None => others = true,
                }
            }
            if others || weapons.is_empty() {
                let _ = writeln!(out, "| any other | {} |", object(fallback));
            }
        }
    }
    out
}

/// The racial talents (spec §8 "Talents", the Racial tree), race by race: those a race grants,
/// then those that ask for it, each with what else it needs and its rule. Races and talents
/// sort by name.
pub(super) fn racial(content: &Content) -> String {
    let book = content.talents();
    let Some(tree) = book.tree_id("racial") else {
        return "No racial talents yet.\n".to_string();
    };
    let mut races: Vec<_> = content.races().list.iter().collect();
    races.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out = String::from("| Race | Talent | Needs | Effect |\n|---|---|---|---|\n");
    for race in races {
        let mut rows: Vec<&TalentDef> = (book.of_tree(tree))
            .map(|(_, t)| t)
            .filter(|t| {
                race.talents.contains(&t.id)
                    || t.needs
                        .iter()
                        .any(|n| matches!(&n, Need::Race(id) if *id == race.id))
            })
            .collect();
        rows.sort_by_key(|t| (!t.granted, t.name.clone()));
        for def in rows {
            let others: Vec<String> = (def.needs.iter())
                .filter(|n| !matches!(n, Need::Race(_)))
                .map(|n| n.words(content))
                .collect();
            let needs = match (def.granted, others.is_empty()) {
                (true, _) => "given, never picked".to_string(),
                (false, true) => "—".to_string(),
                (false, false) => others.join(", "),
            };
            let _ = writeln!(
                out,
                "| {} | {} | {needs} | {} |",
                race.name,
                def.name,
                effects_text(content, def)
            );
        }
    }
    out
}

/// The classes' level tables (spec §8 "Class abilities"), in the order New run offers them:
/// each class with a table, its bonus talent picks, and every level's ability with its rule.
pub(super) fn class_levels(content: &Content) -> String {
    let mut out = String::new();
    for class in content.classes() {
        level_table(content, Table::Class(class), &mut out);
    }
    if out.is_empty() {
        out.push_str("No class has a level table yet.\n");
    }
    out
}

/// Every prestige class's level table (spec §8 "Prestige classes"), at its own levels, as
/// [`class_levels`] shows a class's.
pub(super) fn prestige_levels(content: &Content) -> String {
    let mut out = String::new();
    for prestige in content.prestiges() {
        level_table(content, Table::Prestige(prestige), &mut out);
    }
    if out.is_empty() {
        out.push_str("No prestige class has a level table yet.\n");
    }
    out
}

/// One class's (or prestige class's) table: a heading, its bonus picks, then each row's
/// ability with what it does (a talent handed out rank by rank, that rank's).
fn level_table(content: &Content, table: Table, out: &mut String) {
    let book = content.talents();
    let bonus = table.bonus_talents();
    if table.abilities().is_empty() && bonus.levels.is_empty() {
        return;
    }
    let _ = writeln!(out, "### {}\n", table.name());
    if !bonus.levels.is_empty() {
        let levels: Vec<String> = bonus.levels.iter().map(u8::to_string).collect();
        let trees = match table.bonus_trees(book) {
            None => "any tree".to_string(),
            Some(trees) => (trees.iter())
                .map(|&t| book.tree(t).name.clone())
                .collect::<Vec<_>>()
                .join(" or "),
        };
        let _ = writeln!(
            out,
            "A bonus talent at levels {}, from {trees}.\n",
            levels.join(", ")
        );
    }
    out.push_str("| Level | Ability | Effect |\n|---:|---|---|\n");
    for (level, id, rank) in table.abilities() {
        let Some(talent) = book.id(id) else { continue };
        let def = book.get(talent);
        // A talent of ranks the table hands out rank by rank: each row that rank's.
        let (name, effects) = match def.rank_count() > 1 {
            true => (
                format!("{} {rank}/{}", def.name, def.rank_count()),
                rank_text(content, def, *rank),
            ),
            false => (def.name.clone(), effects_text(content, def)),
        };
        let _ = writeln!(out, "| {level} | {name} | {effects} |");
    }
    out.push('\n');
}

/// The coverage matrix: every system of the closed list with the talents that touch it, and
/// whether the bar (`[coverage]` in `data/talents.toml`) already holds it. Systems and talents
/// sort by name, never by file order.
pub(super) fn by_system(content: &Content) -> String {
    let book = content.talents();
    let bar = book.coverage;
    let mut systems: Vec<_> = (0u8..).zip(&book.systems).collect();
    systems.sort_by(|a, b| a.1.name.cmp(&b.1.name));
    let mut out = String::from("| System | Talents | Held to the bar |\n|---|---|---|\n");
    for (i, system) in systems {
        let mut names: Vec<String> = (book.of_system(warlock_core::content::SystemId(i)))
            .map(|(_, t)| {
                // A class's, a race's, or a birth sign's gift is marked: given, never picked.
                let racial = book.tree_id("racial");
                if t.granted && book.tree(t.tree).class {
                    format!("{} (class)", t.name)
                } else if t.granted && Some(t.tree) == racial {
                    format!("{} (race)", t.name)
                } else if t.granted {
                    format!("{} (birth sign)", t.name)
                } else {
                    t.name.clone()
                }
            })
            .collect();
        names.sort();
        let held = if system.pending {
            "not yet (pending)".to_string()
        } else {
            match system.min_talents.unwrap_or(bar.min_talents) {
                1 => "yes (1 talent at least)".to_string(),
                least => format!("yes ({least} talents at least)"),
            }
        };
        let talents = if names.is_empty() {
            "—".to_string()
        } else {
            names.join(", ")
        };
        let _ = writeln!(out, "| {} | {talents} | {held} |", system.name);
    }
    out
}

/// "Weapon focus (a weapon skill)" for a talent with a choice, "Slight (flaw)" for a birth
/// sign's flaw, else the name.
fn name(def: &TalentDef) -> String {
    match def.choice {
        Some(kind) => format!("{} ({})", def.name, kind.words()),
        None if def.flaw => format!("{} (flaw)", def.name),
        None => def.name.clone(),
    }
}

/// The prerequisites: the talent's own, then each rank's where they differ ("Toughness 11 / 14
/// / 17"); "—" for none; a granted talent is never picked.
pub(super) fn needs_text(content: &Content, def: &TalentDef) -> String {
    if def.granted && content.talents().tree(def.tree).class {
        return "given by a class's table, never picked".into();
    }
    if def.granted {
        let mut races: Vec<String> = (content.races().list.iter())
            .filter(|r| r.talents.contains(&def.id))
            .map(|r| r.name.to_lowercase())
            .collect();
        races.sort();
        if !races.is_empty() {
            return format!("given to every {}, never picked", races.join(", "));
        }
        return "given by a birth sign, never picked".into();
    }
    let words = |needs: &[Need]| -> String {
        (needs.iter())
            .map(|n| match n {
                Need::Creation => "only at character creation".to_string(),
                n => n.words(content),
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let common: Vec<Need> = def.needs.iter().collect();
    let per_rank: Vec<Vec<Need>> = (def.ranks.iter())
        .map(|r| r.needs.iter().collect())
        .collect();
    let mut parts = Vec::new();
    if !common.is_empty() {
        parts.push(words(&common));
    }
    if per_rank.iter().any(|r| !r.is_empty()) {
        let ranks: Vec<String> = (per_rank.iter().enumerate())
            .map(|(i, r)| {
                if r.is_empty() {
                    format!("{}: —", i + 1)
                } else {
                    format!("{}: {}", i + 1, words(r))
                }
            })
            .collect();
        parts.push(ranks.join("; "));
    }
    if parts.is_empty() {
        "—".to_string()
    } else {
        parts.join("; ")
    }
}

/// What the talent does, as the rule: its per-rank effects ("per rank" when it has several
/// ranks), then any rank's own.
pub(super) fn effects_text(content: &Content, def: &TalentDef) -> String {
    let ranked = def.rank_count() > 1;
    let chosen = def.choice.is_some();
    let text_of = |e: &TalentEffect| match e {
        TalentEffect::SpellMod(spec) => spell_mod_text(content, spec, chosen),
        other => effect_text(content, other),
    };
    // An effect under a status (`while`): its text, then "while raging" (the status's name).
    let under = |(status, e): &(StatusId, TalentEffect)| {
        let name = content.status(*status).name.to_lowercase();
        format!("{} while {name}", text_of(e))
    };
    let plain = (def.effects.iter()).map(text_of);
    let mut parts: Vec<String> = (plain.chain(def.effects_while.iter().map(under)))
        .map(|text| {
            if ranked {
                format!("{text} per rank")
            } else {
                text
            }
        })
        .collect();
    for (i, rank) in def.ranks.iter().enumerate() {
        let own = (rank.effects.iter().map(text_of)).chain(rank.effects_while.iter().map(under));
        for text in own {
            parts.push(format!("rank {}: {text}", i + 1));
        }
    }
    if let Some(cost) = def.resource {
        let name = content.resource(cost.resource).name.to_lowercase();
        parts.push(format!("costs {} {name}", cost.amount));
    }
    parts.join("; ")
}

/// What rank `rank` of `def` adds, as a class table gives it: the per-rank effects and that
/// rank's own.
fn rank_text(content: &Content, def: &TalentDef, rank: u8) -> String {
    let parts: Vec<String> = (def.effects_at(rank))
        .map(|e| effect_text(content, &e))
        .collect();
    parts.join("; ")
}

/// A `spell_mod` effect as the rule (the spell scaling design §7.1): each hook it uses, in the
/// cast's order, and the spells it acts on (`chosen`: a talent chosen for a school or an
/// element, whose choice is its filter).
fn spell_mod_text(content: &Content, spec: &SpellModSpec, chosen: bool) -> String {
    let n = |v: i32, one: &str, many: &str| {
        let word = if v.abs() == 1 { one } else { many };
        format!("{} {word}", signed(v))
    };
    let mut parts = Vec::new();
    let rungs = [
        (spec.rungs, "on their damage and healing ladders"),
        (spec.scroll_rungs, "for the spell scrolls you read"),
        (spec.wand_rungs, "for the wands you use"),
        (spec.reach_rungs, "on their reach ladders"),
        (spec.degree_rungs, "on their degrees ladders"),
    ];
    for (v, what) in rungs {
        if v != 0 {
            parts.push(format!("{} {what}", n(v, "rung", "rungs")));
        }
    }
    if spec.sides > 0 {
        parts.push(format!(
            "every die {} bigger",
            n(spec.sides as i32, "side", "sides")
        ));
    }
    if spec.dice > 0 {
        parts.push(format!(
            "{} on the roll",
            n(spec.dice as i32, "die", "dice")
        ));
    }
    if spec.bonus != 0 {
        parts.push(format!("{} on the roll", signed(spec.bonus)));
    }
    if spec.reroll > 0 {
        parts.push(format!(
            "a die showing {} or less is rolled again, once",
            spec.reroll
        ));
    }
    if let Some(rider) = &spec.rider {
        let name = (content.status_id(&rider.status))
            .map_or_else(|| rider.status.clone(), |s| content.status(s).name.clone());
        parts.push(format!(
            "what they harm is {name} ({}) besides",
            rider.severity
        ));
    }
    if spec.cells != 0 {
        parts.push(format!(
            "range or length {}",
            n(spec.cells, "cell", "cells")
        ));
    }
    if spec.radius_cells != 0 {
        parts.push(format!(
            "a burst's radius {}",
            n(spec.radius_cells, "cell", "cells")
        ));
    }
    if spec.degrees != 0 {
        parts.push(format!(
            "their status {}",
            n(spec.degrees, "degree", "degrees")
        ));
    }
    if spec.cost != 0 {
        parts.push(format!("mana {} %", signed(spec.cost)));
    }
    let whose = if chosen {
        "spells of the chosen school or element".to_string()
    } else if spec.when.is_empty() {
        "your spells".to_string()
    } else {
        format!("your {} spells", spec.when.join(" or "))
    };
    format!("{whose}: {}", parts.join(", "))
}

/// One effect as the rule (talents' and class abilities').
/// A cooldown status by name and how long `degrees` of it last: "Graceless for 24 h".
fn cooldown_for(content: &Content, id: &str, degrees: u16) -> String {
    let Some(status) = content.status_id(id) else {
        return id.to_string();
    };
    let def = content.status(status);
    match def.eases.map(|s| s * u32::from(degrees)) {
        Some(s) if s % 3600 == 0 => format!("{} for {} h", def.name, s / 3600),
        Some(s) if s % 60 == 0 => format!("{} for {} min", def.name, s / 60),
        Some(s) => format!("{} for {s} s", def.name),
        None => def.name.clone(),
    }
}

/// The cooldown of the birth sign's active `active` at `degrees`, as [`cooldown_for`] says it.
fn sign_cooldown(content: &Content, active: SignActive, degrees: u16) -> String {
    let id = (content.sign_active(active).into_iter())
        .flat_map(|def| def.effects.iter())
        .find_map(|e| match e {
            TalentEffect::HornsGrace { cooldown, .. } | TalentEffect::Stare { cooldown, .. } => {
                Some(cooldown.clone())
            }
            _ => None,
        });
    id.map_or_else(
        || format!("{degrees} degree of its cooldown"),
        |id| cooldown_for(content, &id, degrees),
    )
}

/// An alternative resource's name ("Focus"), or its id when content has none by it.
fn resource_name(content: &Content, id: &str) -> String {
    (content.resource_id(id)).map_or_else(|| id.to_string(), |r| content.resource(r).name.clone())
}

/// A buff status in words: its name, what it adds to fighting, and how long it lasts
/// ("Inspired (to hit +1, Dodge +1, 5 min)").
fn buff(content: &Content, id: &str) -> String {
    let Some(s) = content.status_id(id).map(|s| content.status(s)) else {
        return id.to_string();
    };
    let c = s.combat;
    let parts = [
        ("to hit", c.to_hit),
        ("damage", c.damage),
        ("Dodge", c.dodge),
        ("Parry", c.parry),
        ("Shield", c.shield),
    ];
    let mut said: Vec<String> = (parts.iter())
        .filter(|(_, v)| *v != 0)
        .map(|(what, v)| format!("{what} {}", signed(*v)))
        .collect();
    let each = if s.max > 1 { " a degree" } else { "" };
    said.extend(s.eases.map(|e| match e {
        e if e % 60 == 0 => format!("{} min{each}", e / 60),
        e => format!("{e} s{each}"),
    }));
    format!("{} ({})", s.name, said.join(", "))
}

/// What a prayer use answers, in words (`data/prayer.toml`'s release list named).
fn prayer_use_text(content: &Content, id: PrayerUse) -> String {
    match id {
        PrayerUse::Disease => "every disease, showing or still brewing (a brewing one alone is \
                               nothing to pray for)"
            .into(),
        PrayerUse::Poison => "poison".into(),
        PrayerUse::Wounds => "bleeding, and hit points to full".into(),
        PrayerUse::Release => {
            let names: Vec<String> = (content.prayer().into_iter())
                .flat_map(|r| r.release.iter())
                .map(|&s| content.status(s).name.clone())
                .collect();
            format!("hexes and holds ({})", names.join(", "))
        }
        PrayerUse::Mana => "mana to full".into(),
        PrayerUse::Hunger => "hunger (Hungry or worse), fed to Satiated".into(),
        PrayerUse::Restoration => "all stat damage, and every point drained".into(),
    }
}

/// The priest's prayer effect in full: the tier's title, what it adds, and, for the first tier,
/// the strain's numbers and the priority order.
fn prayer_text(
    content: &Content,
    tier: Option<u8>,
    title: Option<&Faced>,
    uses: &[PrayerUse],
    answers: Option<u8>,
    all: bool,
) -> String {
    let mut parts = Vec::new();
    if let (Some(tier), Some(title)) = (tier, title) {
        parts.push(format!(
            "the prayer's tier {tier}: it is called {} (answered by the holy powers) or {} (by \
             the dark)",
            title.holy, title.dark
        ));
    }
    if !uses.is_empty() {
        let said: Vec<String> = uses.iter().map(|&u| prayer_use_text(content, u)).collect();
        parts.push(format!("a prayer may answer {}", said.join("; ")));
    }
    if let Some(n) = answers {
        parts.push(format!("a prayer answers {n} problems at once"));
    }
    if all {
        parts.push(
            "a prayer answers every problem at once, every use included, and fills every \
             resource pool held (stamina too)"
                .into(),
        );
    }
    if tier == Some(1)
        && let Some(rules) = content.prayer()
    {
        let order: Vec<&str> = rules.uses.iter().map(|u| u.id.id()).collect();
        parts.push(format!(
            "active (`use:orison`, the time of a spell, no mana): it answers the first problem \
             the priest has, in the order {}. The holy powers answer {} and better, the dark \
             ones {} and worse, by the band at the moment. Every attempt adds {} strain, \
             answered or not, and a point passes every {}; a prayer fizzles with a chance of {} % \
             per 100 strain (at most 100 %), and a fizzle locks the prayer: every prayer fizzles \
             until the strain has passed to 0. With nothing to answer it is refused, with no \
             time or strain",
            order.join(", "),
            rules.holy.min.map_or("every band", |b| b.name()),
            rules.dark.max.map_or("every band", |b| b.name()),
            rules.strain,
            rules.decay,
            rules.fizzle_per_100,
        ));
    }
    parts.join("; ")
}

pub(super) fn effect_text(content: &Content, effect: &TalentEffect) -> String {
    let percent = |v: i32| format!("{} %", signed(v));
    let status = |id: &str| {
        (content.status_id(id)).map_or_else(|| id.to_string(), |s| content.status(s).name.clone())
    };
    match effect.clone() {
        TalentEffect::Dodge { value } => format!("Dodge {}", signed(value)),
        TalentEffect::Natural { value } => format!("natural armour {}", signed(value)),
        TalentEffect::MeleeDamage { value } => format!("melee damage {}", signed(value)),
        TalentEffect::Ferocity { value } => format!(
            "damage {} with every attack while below a third of your hit points",
            signed(value)
        ),
        TalentEffect::Threat { value } => {
            format!("threat range {} with every attack", signed(value))
        }
        TalentEffect::RerollFumble => "a natural 2 on the attack roll is rolled again, once".into(),
        TalentEffect::AllyMod {
            to_hit,
            protection,
            count,
            duration_percent,
            lasting,
            most,
        } => {
            let mut gets = Vec::new();
            if to_hit != 0 {
                gets.push(format!("to-hit {} on their attacks", signed(to_hit)));
            }
            if protection != 0 {
                gets.push(format!("Protection {}", signed(protection)));
            }
            let mut parts = Vec::new();
            if !gets.is_empty() {
                parts.push(format!("your allies get {}", gets.join(" and ")));
            }
            if count != 0 {
                parts.push(format!("every summon spell's call brings {count} more"));
            }
            if duration_percent != 0 {
                parts.push(format!("calls last {duration_percent}% longer"));
            }
            if lasting {
                parts.push("calls never lapse: they stay until slain".into());
            }
            if most != 0 {
                parts.push(format!("{most} more of each summon spell's calls at once"));
            }
            parts.join("; ")
        }
        TalentEffect::SummonBias { tag, weight } => match weight {
            0 => format!(
                "a summon spell never calls a creature tagged `{tag}` while another may come"
            ),
            n => format!("a summon spell calls a creature tagged `{tag}` {n} times as often"),
        },
        TalentEffect::MaxHpPerLevels {
            per_levels,
            at_least,
        } => format!("+1 maximum hit point per {per_levels} levels, at least +{at_least}"),
        TalentEffect::MaxHpPercent { value } => {
            format!(
                "maximum hit points {} (rounded down, at least 1)",
                percent(value)
            )
        }
        TalentEffect::Stat { stat, value } => {
            format!("{} {} (score and potential)", stat.name(), signed(value))
        }
        TalentEffect::HealingPercent { value } => format!("healing pace {}", percent(value)),
        TalentEffect::ManaRegenPercent { value } => {
            format!("mana regeneration pace {}", percent(value))
        }
        TalentEffect::MaxManaPerLevels {
            per_levels,
            at_least,
        } => format!("+1 maximum mana per {per_levels} levels, at least +{at_least}"),
        TalentEffect::ManaRestingPercent { value } => {
            format!("mana regeneration pace while resting {}", percent(value))
        }
        TalentEffect::ManaCostPercent { value } => format!(
            "abilities' mana costs {} (rounded to the nearest, at least 1)",
            percent(value)
        ),
        TalentEffect::RodCostPercent { value } => format!(
            "rods' mana costs {} (rounded to the nearest, at least 1), before other cuts",
            percent(value)
        ),
        TalentEffect::CasterLevel {
            value,
            per_levels: 0,
        } => format!("caster level {}", signed(value)),
        TalentEffect::CasterLevel { value, per_levels } => {
            format!(
                "caster level {} per {per_levels} character levels",
                signed(value)
            )
        }
        TalentEffect::AttunedPercent { value } => format!(
            "the caster level your magic items give counts {value} % more (rounded up: +1 is +2, \
             +3 is +5)"
        ),
        TalentEffect::BloodMagic { hp, mana } => format!(
            "active (the time of a wait): give {hp} hit points (never your last) for {mana} \
             mana, up to your maximum; not while your mana is full"
        ),
        TalentEffect::FootstepsPercent { value } => format!("footsteps {}", percent(value)),
        TalentEffect::SilentDoors => "the doors you open and close are silent".to_string(),
        TalentEffect::Hearing { value } => format!("hears sounds {value} fainter"),
        TalentEffect::ExactFootsteps => {
            "footsteps you hear are placed at the cell they came from, not roughly".to_string()
        }
        TalentEffect::ThresholdPercent { value } => {
            format!("the light your eyes need {}", percent(value))
        }
        TalentEffect::Darkvision { value } => format!("darkvision {} cells", signed(value)),
        TalentEffect::Scent { status: id, radius } => format!(
            "sees creatures that are {} in line of sight within {radius} cells, however dark",
            status(&id).to_lowercase()
        ),
        TalentEffect::NightSight { value } => {
            format!("overworld sight at night {} cells", signed(value))
        }
        TalentEffect::DaySight { value } => {
            format!("overworld sight by day {} cells", signed(value))
        }
        TalentEffect::InnerLight { light } => {
            let reach = |id: &str| (content.light_id(id)).map_or(0, |l| content.light(l).radius);
            format!(
                "you always shed a faint light reaching {} cell (everyone sees it)",
                reach(&light)
            )
        }
        TalentEffect::LanternLight { light } => {
            let reach = |id: &str| (content.light_id(id)).map_or(0, |l| content.light(l).radius);
            format!(
                "your lantern burns brighter: its light reaches {} cells",
                reach(&light)
            )
        }
        TalentEffect::Ambush { dice } => format!(
            "+{dice} damage, after Protection, on a hit at a creature that has not seen you \
             (it hunts no one it has seen, and does not see you now)"
        ),
        TalentEffect::InitiativeBonus { value } => format!(
            "{} to the initiative check when something catches you unawares (see Status effects: \
             Surprised)",
            signed(value)
        ),
        TalentEffect::SurpriseEase { value } => format!(
            "{value}% of the extra time the action you take Surprised costs is spared (at 100%, \
             it costs no more than any other)"
        ),
        TalentEffect::SurpriseDifficulty { value } => format!(
            "those you catch unawares make their initiative check at {} difficulty",
            signed(value)
        ),
        TalentEffect::SneakReach { value } => {
            format!("your shots, throws, and attack spells sneak-attack {value} cells farther")
        }
        TalentEffect::SneakKillStatus { status: id } => format!(
            "a sneak attack of yours that kills leaves you {} (1 degree)",
            status(&id)
        ),
        TalentEffect::Companion => "an [animal companion](allies.md#companions) from the first \
                                    turn: a beast of your race's kind, or rarely a rarer one, \
                                    that fights for you, always comes along, and grows with \
                                    your level"
            .into(),
        TalentEffect::Tame { bonus } => format!("{} on your taming checks", signed(bonus)),
        TalentEffect::SharedSight => "you see what your [companion](allies.md#hunters-bond) \
                                      sees, by its own eyes, and the foes its nose finds, \
                                      through the dark and closed doors"
            .into(),
        TalentEffect::SharedBuffs => "a [buff](allies.md#hunters-bond) that lands on you or \
                                      your companion lands on the other too, at the same \
                                      strength; a hex never does"
            .into(),
        TalentEffect::PackTactics { to_hit } => format!(
            "a creature beside both you and your [companion](allies.md#companions) is flanked: \
             your melee attacks and your companion's at it, to hit {} (not shots, throws, or \
             spells)",
            signed(to_hit)
        ),
        TalentEffect::SharesFavouredPrey => "your companion's attacks at your favoured prey \
                                             carry your Favoured prey bonus, to hit and damage \
                                             (Master hunter's too)"
            .into(),
        TalentEffect::SeesUnaware => "you see which creatures in view have not noticed you: \
                                      look mode says \"unaware of you\", and their glyphs are \
                                      drawn a shade cooler, breathing slowly (standing still \
                                      with reduced effects)"
            .to_string(),
        TalentEffect::StartledBlow => "when something beside you attacks you unawares and you \
                                       make the initiative check, you strike it at once with \
                                       what you wield, before its attack lands (a free blow, \
                                       once per surprise; it brings no free blows of its own)"
            .to_string(),
        TalentEffect::CarryPercent { value } => format!("carrying limit {}", percent(value)),
        TalentEffect::IgnoreBurdened => "being Burdened costs you nothing (as a dwarf)".to_string(),
        TalentEffect::StepSlowdownPercent { value } => format!(
            "the slowdown your burden and being Bloated put on your steps {}",
            percent(value)
        ),
        TalentEffect::CarryKg { value } => format!("carrying limit {} kg", signed(value)),
        TalentEffect::WeightlessCoins => "coins you carry weigh nothing".to_string(),
        TalentEffect::TravelPercent { value } => {
            format!("overworld journey time {}", percent(value))
        }
        TalentEffect::Swimmer => {
            "wading (water, marsh) costs a plain step and never gives a disease".into()
        }
        TalentEffect::TerrainAs { terrains, like } => {
            let name = |id: &str| {
                (content.terrain_id(id))
                    .map_or_else(|| id.to_string(), |t| content.terrain(t).name.clone())
            };
            let names: Vec<String> = terrains.iter().map(|t| name(t)).collect();
            format!(
                "a step onto {} costs what one onto {} does",
                names.join(", "),
                name(&like)
            )
        }
        TalentEffect::PlungeProof => "a plunge through the ice does you no harm".to_string(),
        TalentEffect::LightOnIce => {
            "ice holds you as if you carried nothing (your own weight only)".to_string()
        }
        TalentEffect::ClimbPercent { value } => {
            format!("climbing out onto ice costs {}", percent(value))
        }
        TalentEffect::WadingProof => "wading never gives you a disease".to_string(),
        TalentEffect::BeastFriend => "creatures of animal factions are neutral to you until you \
                                      attack one of that type"
            .to_string(),
        TalentEffect::HungerPercent { value } => format!("hunger pace {}", percent(value)),
        TalentEffect::SmellRot => {
            "the eat list says whether food smells fresh, turning (old),              or rotten"
                .to_string()
        }
        TalentEffect::RotHarmPercent { value } => {
            format!(
                "sickness from rotten food {} (rounded down)",
                percent(value)
            )
        }
        TalentEffect::NoExertion => "blows, shots, and throws burn no nutrition".to_string(),
        TalentEffect::NoBloating => "a meal never leaves you Bloated: what does not fit below \
             Bloated is lost, and you never overeat"
            .to_string(),
        TalentEffect::NoHungerWeakness => {
            "being [Weak, Fainting, or Starving](hunger.md#how-hungry-you-are) \
             takes nothing off your Strength and Dexterity, and you never faint from hunger; \
             Starving still stops your healing and costs hit points"
                .to_string()
        }
        TalentEffect::CorpseStatsPercent { value } => format!(
            "a fresh corpse's chance to raise each stat {} (at most 100 %)",
            percent(value)
        ),
        TalentEffect::XpPercent { value } => format!("experience gained {}", percent(value)),
        TalentEffect::SkillRolls { value } => format!("skill rolls every level {}", signed(value)),
        TalentEffect::ExtraPick { level: 1 } => "one more talent at creation".to_string(),
        TalentEffect::ExtraPick { level } => format!("one more talent at level {level}"),
        TalentEffect::TrainSkill { dice } => format!(
            "the chosen skill trained at {dice}, or raised by {dice} if trained (rolled as the \
             rank is taken)"
        ),
        TalentEffect::WeaponRanks { to, skills, .. } if skills.is_empty() => format!(
            "every [weapon skill](weapon-skills.md) and [fighting style](fighting-styles.md) \
             (not the schools of magic or a hidden style) below rank {to} rises to rank {to}, \
             once, as it is given"
        ),
        TalentEffect::WeaponRanks { to, skills, .. } => {
            let table = content.weapon_skills();
            let names: Vec<String> = (skills.iter())
                .filter_map(|s| table.id(s).map(|id| table.skill(id).name.clone()))
                .collect();
            format!(
                "{} below rank {to} rise to rank {to}, once, as it is given",
                names.join(", ")
            )
        }
        TalentEffect::TakeUpSkills { dice, except, .. } => {
            let mut text = format!(
                "every [skill](skills.md) you are not trained in, whatever skills there are, \
                 trained at {dice} (rolled once, as it is given)"
            );
            let names: Vec<String> = (except.iter())
                .filter_map(|s| content.skill_id(s).map(|id| content.skill(id).name.clone()))
                .collect();
            if !names.is_empty() {
                text.push_str(&format!(", but {}", names.join(" and ")));
            }
            text
        }
        TalentEffect::StartingGold {
            percent,
            at_least,
            at_most,
        } => format!(
            "at creation, [gold coins](shops.md#money): {percent} % of those you start with more, at least \
             {at_least} and at most {at_most}"
        ),
        TalentEffect::StartingGems { min, max, level } => {
            format!(
                "at creation, {min} to {max} loose [gems](gems.md), as found at \
                 [item level](magic-items.md#finding-them) {level}"
            )
        }
        TalentEffect::Heirloom {
            min, max, level, ..
        } => format!(
            "at creation, your race's heirloom weapon (Heirlooms, below) with {min} to {max} \
             [affixes](magic-items.md#prefixes) rolled at [item level](magic-items.md#finding-them) \
             {level} (two: [magic](magic-items.md); more: [rare](magic-items.md#rarity)), \
             [identified](magic-items.md#unidentified-items), readied in its own \
             [slot](equipment.md#what-fits-where) if that is free, else carried"
        ),
        TalentEffect::LootMagicPercent { value } => format!(
            "the weights of the rarities better than a find's least {} (anything found: magic \
             and rare; what creatures drop, always magic: rare)",
            percent(value)
        ),
        TalentEffect::ReadPercent { value } => format!("reading time {}", percent(value)),
        TalentEffect::Study {
            bonus,
            percent: time,
            overreach_percent,
        } => {
            let mut parts = Vec::new();
            if bonus != 0 {
                parts.push(format!("study check {}", signed(bonus)));
            }
            if time != 0 {
                parts.push(format!("study time {}", percent(time)));
            }
            if overreach_percent != 0 {
                parts.push(format!(
                    "the levels a spell is above your caster level count {} in the study \
                     check, rounded down (a spell beyond your learning is that much less beyond \
                     you)",
                    percent(overreach_percent)
                ));
            }
            parts.join(", ")
        }
        TalentEffect::ScrollKept { chance } => {
            format!("a {chance} % chance a scroll you read is not used up")
        }
        TalentEffect::SparesSelf => {
            "your own bursts, a scroll's or a spell's, never catch you".to_string()
        }
        TalentEffect::PotionKept { chance } => {
            format!("a {chance} % chance a potion you drink is not used up")
        }
        TalentEffect::QuestGoldPercent { value } => {
            format!("gold from quests {}", percent(value))
        }
        TalentEffect::FavourPercent { value } => {
            format!("shop favour {} (before the cap)", percent(value))
        }
        TalentEffect::Pilgrim => {
            "temple services cost half (rounded up), and a cure also mends one point of each \
             damaged stat"
                .to_string()
        }
        TalentEffect::GemWorth => "item descriptions show what each gem is worth".to_string(),
        TalentEffect::IdentifyCarried { hours } => format!(
            "a magic item you carry for {hours} hours (counted from when you gain this) is \
             identified"
        ),
        TalentEffect::GemFindPercent { value } => format!(
            "the chance a find is a loose gem {} (anything found, and what creatures drop)",
            percent(value)
        ),
        TalentEffect::Notice { value } => {
            format!(
                "notices hidden things in passing, {} to the check",
                signed(value)
            )
        }
        TalentEffect::SpawnEveryPercent { value } => format!(
            "the time between attempts to bring a creature to your level {}",
            percent(value)
        ),
        TalentEffect::Devout => "at a forgotten altar of their own side the side's lean counts \
                                  double (30 points from bane to boon), and an opposed one leans \
                                  as a neutral one"
            .into(),
        TalentEffect::Penitent => "at a forgotten altar a great bane (Wrath of the forgotten, \
                                    Unmade, Forsaken, Damned) becomes one of the nothing \
                                    outcomes instead; the altar's explosion is not great and \
                                    still happens"
            .into(),
        TalentEffect::ChestWise => "sees whether a shut chest in sight is locked (look mode: \
            \"a locked coffer\", \"an unlocked coffer\"), and knows the trap of every shut chest \
            it sees, no check"
            .into(),
        TalentEffect::RoomPacePercent { value } => format!(
            "where a special room quickens healing or mana, the quickening {} (a room's 300% \
             pace becomes {}%)",
            percent(value),
            100 + 200 * (100 + value) / 100
        ),
        TalentEffect::RoomStatusImmune { status: id } => format!(
            "a special room's (or a whole level's) {} never takes you",
            status(&id)
        ),
        TalentEffect::GraveWarden => {
            "nothing you kill ever rises in a cursed room (see Special rooms)".into()
        }
        TalentEffect::StoneSense => "notices hidden things on stone in passing".into(),
        TalentEffect::StoneFind { value } => format!(
            "{} to every check to find what is hidden on stone, searching and in passing",
            signed(value)
        ),
        TalentEffect::Search { value } => {
            format!("{} to every check to find hidden things", signed(value))
        }
        TalentEffect::Searching { value } => format!("searching {}", signed(value)),
        TalentEffect::CollapseProof => "a collapsed floor never gives way under you".to_string(),
        TalentEffect::SeesMimics { value } => format!(
            "sees a disguised mimic within {value} cell{} for what it is: it is shown as itself, \
             not as a chest",
            if value == 1 { "" } else { "s" }
        ),
        TalentEffect::Slick => "a mimic's grip (its slam's hold) never holds it".into(),
        TalentEffect::HoardGrace { value } => format!(
            "a tempting hoard seals only once you have lifted {} stack{} inside it",
            1 + value,
            if value == 0 { "" } else { "s" }
        ),
        TalentEffect::SearchReach { value } => format!(
            "searching reaches {value} cell{} farther",
            if value == 1 { "" } else { "s" }
        ),
        TalentEffect::TrapDamagePercent { value } => format!(
            "damage traps do you {} (rounded down, at least 1)",
            percent(value)
        ),
        TalentEffect::Skill { skill, value } => {
            let name =
                (content.skill_id(&skill)).map_or(skill.clone(), |s| content.skill(s).name.clone());
            format!("{name} counts {}", signed(value))
        }
        TalentEffect::FreeBlow { trigger } => {
            use warlock_core::content::FreeBlowTrigger as T;
            match trigger {
                T::Cleave => {
                    "free blow: a melee blow that kills gives one at another hostile beside you \
                     (Cleave and Cleaving strike together: on and on, as Cleave's second rank)"
                }
                T::GreatCleave => {
                    "free blow: cleaving goes on while the blows kill (once at each hostile beside \
                     you a turn)"
                }
                T::Riposte => "free blow: a blow that misses you by your Parry gives one back",
                T::Opportunist => "free blow: a hostile that steps away from beside you takes one",
                T::Gatekeeper => "free blow: a hostile that steps up beside you takes one",
                T::Flurry => {
                    "free blow: a melee attack with nothing in hand strikes twice, the second blow \
                     at \u{2212}2 to hit"
                }
            }
            .to_string()
        }
        TalentEffect::Weapon {
            role,
            skill,
            to_hit,
            damage,
            threat,
            ranks,
            speed,
            reload,
            ease,
            bash,
            shield,
            crit,
            at_rank,
        } => {
            let with = match (role, skill) {
                (_, Some(id)) => (content.weapon_skills().id(&id)).map_or(id.clone(), |w| {
                    content.weapon_skills().skill(w).name.clone()
                }),
                (None, None) => "the chosen skill".to_string(),
                (Some(Role::Melee), _) => "every melee weapon skill".to_string(),
                (Some(Role::Launcher), _) => "every missile weapon skill".to_string(),
                (Some(Role::Thrown), _) => "throws".to_string(),
                (Some(Role::Unarmed), _) => "unarmed fighting".to_string(),
                (Some(Role::ShieldBlock), _) => "shield blocking".to_string(),
                (Some(Role::ShieldBash), _) => "shield bashing".to_string(),
                (Some(Role::School), _) => "every school of magic".to_string(),
                (Some(Role::Style), _) => "every fighting style".to_string(),
            };
            let mut gains = Vec::new();
            for (value, what) in [(to_hit, "to hit"), (damage, "damage"), (shield, "Shield")] {
                if value != 0 {
                    gains.push(format!("{what} {}", signed(value)));
                }
            }
            if threat > 0 {
                gains.push(format!("threat range +{threat}"));
            }
            if crit > 0 {
                gains.push(format!("critical multiplier +{crit} (at most \u{d7}5)"));
            }
            if ranks > 0 {
                gains.push(format!("benefits as {ranks} rank higher (at most 10)"));
            }
            if speed > 0 {
                gains.push(format!("attacks {speed} % cheaper"));
            }
            if reload > 0 {
                gains.push(format!("reloading {reload} % cheaper"));
            }
            if ease > 0 {
                gains.push(format!("both two-weapon penalties eased by {ease}"));
            }
            if bash > 0 {
                gains.push(format!(
                    "a free shield bash's chance +{bash} % where there is one"
                ));
            }
            let rank = if at_rank > 0 {
                format!(" at rank {at_rank} or more")
            } else {
                String::new()
            };
            format!("{} with {with}{rank}", gains.join(", "))
        }
        TalentEffect::WeaponMarks { role, value } => {
            let role = match role {
                None => {
                    return format!(
                        "marks in every weapon skill and fighting style {}",
                        percent(value)
                    );
                }
                Some(Role::Launcher) => "missile".to_string(),
                Some(r) => r.id().replace('_', " "),
            };
            format!("marks in every {role} weapon skill {}", percent(value))
        }
        TalentEffect::SchoolMarksPercent { value } => {
            format!("marks in every school of magic {}", percent(value))
        }
        TalentEffect::TwoHandedCrit { value } => {
            format!("critical multiplier +{value} with a two-handed weapon (at most \u{d7}5)")
        }
        TalentEffect::UnarmedDie { sides } => {
            format!("blows with nothing in hand and kicks roll 1d{sides}")
        }
        TalentEffect::StanceDodge { stance, percent } => {
            format!("{} stance's Dodge penalty {percent} % smaller", stance.id())
        }
        TalentEffect::StanceCosts => {
            "changing your combat stance takes the time of a wait (it is free otherwise)".into()
        }
        TalentEffect::StepPercent {
            value,
            unarmoured,
            unburdened,
            no_shield,
        } => {
            let mut when = Vec::new();
            if unarmoured {
                when.push("in no body armour");
            }
            if unburdened {
                when.push("Unburdened");
            }
            if no_shield {
                when.push("with no shield");
            }
            let when = if when.is_empty() {
                String::new()
            } else {
                format!(" while {}", when.join(" and "))
            };
            format!("steps cost {}{when}", percent(value))
        }
        TalentEffect::Stout => {
            "Strained costs you what Burdened costs others (slowdown, Dexterity cap, noise)"
                .to_string()
        }
        TalentEffect::DiseaseImmune => "you never catch a disease".to_string(),
        TalentEffect::NoPenaltyTo { stat } => {
            format!("no status lowers your {}", stat.name())
        }
        TalentEffect::RaisePercent { status: id, value } => {
            format!(
                "what {} raises your scores by {}",
                status(&id),
                percent(value)
            )
        }
        TalentEffect::VeteransWind {
            per_level,
            at_least,
            cooldown,
        } => format!(
            "active (the time of a wait): {per_level} hit point per level back at once, at \
             least {at_least}; then {}",
            status(&cooldown)
        ),
        TalentEffect::LayOnHands { per_level } => {
            format!("active (the time of a cast): {per_level} hit points per level back at once")
        }
        TalentEffect::Blaze {
            radius,
            damage,
            damage_type,
            fire,
            burns,
            cooldown,
            ..
        } => {
            let kind = (content.damage_type_id(&damage_type)).map_or(damage_type.clone(), |d| {
                content.damage_type(d).name.to_lowercase()
            });
            let light = (content.light_id(&fire)).map_or(0, |l| content.light(l).radius);
            let until = (content.status_id(&cooldown))
                .and_then(|s| content.status(s).eases)
                .map_or_else(String::new, |s| format!(" for {} h", s / 3600));
            format!(
                "active (the time of a cast; not on the overworld): a burst of radius {radius} \
                 around you, {damage} {kind} damage (one roll) to every creature in it but you, \
                 whole to each; it melts the ice it reaches and leaves a fire on every walkable \
                 cell of it but yours, its light reaching {light} cells, for {burns} s; then \
                 {}{until}",
                status(&cooldown)
            )
        }
        TalentEffect::Avatar { hands_cost_percent } => format!(
            "Lay on hands heals every hit point and cures every status that ticks (bleeding, \
             poison) and every disease, for {hands_cost_percent} % of its cost (at least 1)"
        ),
        TalentEffect::Rage { resource: id } => {
            let rage = content.resource_id(&id).map(|r| content.resource(r));
            let gain = rage.map_or(id.clone(), |r| {
                format!("{} {}", r.start, r.name.to_lowercase())
            });
            let brings = rage.and_then(|r| r.status).map_or_else(String::new, |s| {
                format!(
                    ", and you are {} while you hold any",
                    content.status(s).name
                )
            });
            format!(
                "active (the time of a wait): you gain {gain} (see Resources){brings}; no rage \
                 while it or what follows it lasts"
            )
        }
        TalentEffect::WarCry {
            status: id,
            severity,
            difficulty,
        } => {
            let eases = (content.status_id(&id))
                .and_then(|s| content.status(s).eases)
                .map_or_else(String::new, |e| format!(", a degree easing every {e} s"));
            format!(
                "active (the time of a wait): a roar of loudness {}; every hostile that hears \
                 it, sees you, and has vitals rolls its Willpower against {difficulty} + your \
                 Strength modifier + your Intimidate (see Skills); failed, it suffers {} at \
                 {severity} degrees (it runs from you while a step takes it farther{eases}). \
                 No level cut; with nobody to hear it the roar is still made, its time and cost \
                 spent",
                warlock_core::noise::HOWL_NOISE,
                status(&id),
            )
        }
        TalentEffect::Cackle { degrees, noise } => {
            let hexes: Vec<String> = (content.hex_statuses().iter())
                .map(|&s| content.status(s).name.clone())
                .collect();
            format!(
                "active (the time of a spell; no mana, cooldown, or check, as often as you like): \
                 a cackle, a voice of loudness {noise}; every hex you laid ({}) on a creature you \
                 see grows {degrees} degree{}, up to its maximum, and its degree's time starts \
                 again (one at its maximum only starts again). Another's hex and a creature out \
                 of sight are untouched. With no creature in sight under a hex of yours it is \
                 refused, with no time spent",
                hexes.join(", "),
                if degrees == 1 { "" } else { "s" },
            )
        }
        TalentEffect::ResourceBuild {
            resource: id,
            on,
            amount,
        } => format!(
            "{} builds {amount} more {} while you hold it",
            on.words(),
            resource_name(content, &id).to_lowercase()
        ),
        TalentEffect::ResourceMax {
            resource: id,
            value,
        } => {
            format!(
                "{} holds {value} more at most",
                resource_name(content, &id).to_lowercase()
            )
        }
        TalentEffect::ResourceStart {
            resource: id,
            value,
        } => format!(
            "{} starts {value} higher when you gain it",
            resource_name(content, &id).to_lowercase()
        ),
        TalentEffect::ResourceRate {
            resource: id,
            value,
        } => {
            let r = content.resource_id(&id).map(|r| content.resource(r));
            let way = match r.and_then(|r| r.over_time) {
                Some(OverTime::Decays { .. }) => "decay",
                _ => "regeneration",
            };
            format!(
                "{} {way} pace {}",
                resource_name(content, &id).to_lowercase(),
                percent(value)
            )
        }
        TalentEffect::Unstoppable { statuses, cost } => {
            let names: Vec<String> = statuses.iter().map(|s| status(s)).collect();
            format!(
                "while raging: {} tick nothing, and a blow that would take your last hit point \
                 leaves it for {cost} rage, when you have that much (spending the last of it \
                 ends the rage)",
                names.join(" and ")
            )
        }
        TalentEffect::Whirlwind => {
            "active (the time of a melee attack, one exertion): one blow at \
             every hostile beside you, clockwise from the north"
                .to_string()
        }
        TalentEffect::Smite {
            per_levels,
            sides,
            aim,
            unholy,
        } => {
            let aim = aim.map_or_else(String::new, |stat| {
                format!(", your {} modifier added to hit", stat.name())
            });
            let unholy = match (unholy, content.smite()) {
                (0, _) | (_, None) => String::new(),
                (n, Some(rules)) => format!(
                    ", and {n}d{sides} more against the unholy (a creature tagged {})",
                    rules.unholy.join(", ")
                ),
            };
            format!(
                "active (the time of a melee attack), in a direction: a blow with 1d{sides} \
                 more damage per {per_levels} levels (at least 1d{sides}){unholy}{aim}"
            )
        }
        TalentEffect::SmiteMode {
            mode,
            status: id,
            severity,
            turns,
            check,
            difficulty,
        } => {
            let rider = match (mode, turns, check) {
                (SmiteRiderKind::Fear, Some(turns), Some(stat)) => format!(
                    "the foe checks {} (2d12 + its modifier) against {difficulty} plus your \
                     Smite's aim modifier; failed, it is {} by you for {turns} turns, fleeing \
                     you",
                    stat.name(),
                    status(&id)
                ),
                _ => {
                    let degrees = if severity == 1 { "degree" } else { "degrees" };
                    format!("the foe suffers {severity} {degrees} of {}", status(&id))
                }
            };
            format!(
                "active (the time of a melee attack), in a direction: your Smite's blow (its \
                 dice, aim, and unholy dice), and on a hit that does damage {rider}"
            )
        }
        TalentEffect::Radiance { status: id, light } => {
            let reach = (content.light_id(&light)).map_or(0, |l| content.light(l).radius);
            format!(
                "active (the time of a cast), on and off: while you are {} you shed a light \
                 reaching {reach} cells",
                status(&id)
            )
        }
        TalentEffect::QuiveringPalm {
            status: id,
            severity,
        }
        | TalentEffect::StunningStrike {
            status: id,
            severity,
        } => format!(
            "active (the time of a melee attack), in a direction, nothing in hand: a bare-handed \
             blow; a hit that does damage leaves {severity} degrees of {}",
            status(&id)
        ),
        TalentEffect::WindStep { status: id } => format!(
            "active, taking no time: a degree of {}, so the next action, whatever it is, costs \
             half its time; refused while you are {} already",
            status(&id),
            status(&id)
        ),
        TalentEffect::Stillness { name, statuses } => {
            let names: Vec<String> = statuses.iter().map(|s| status(s)).collect();
            format!(
                "active ({name}), taking no time: whichever of {} you suffer ends at once; \
                 refused with none of them to end",
                names.join(", ")
            )
        }
        TalentEffect::Hardened { bonus } => format!(
            "every level past the last that adds hit points adds them again, and {bonus} more"
        ),
        TalentEffect::LauncherRange { value, skill: None } => {
            format!("missile weapon range {} cells", signed(value))
        }
        TalentEffect::LauncherRange {
            value,
            skill: Some(skill),
        } => {
            let table = content.weapon_skills();
            let name = (table.id(&skill)).map_or(skill.clone(), |s| table.skill(s).name.clone());
            format!("range {} cells with launchers of {name}", signed(value))
        }
        TalentEffect::ThrowRange { value } => format!("throwing range {} cells", signed(value)),
        TalentEffect::MaxDex { value } => {
            format!("worn armour's Dexterity cap {}", signed(value))
        }
        TalentEffect::Deflection { value } => format!("Deflection {}", signed(value)),
        TalentEffect::Protection {
            value,
            body_armour,
            shield,
            per_piece,
        } => {
            let mut text = format!("Protection {}", signed(value));
            if per_piece {
                text +=
                    " for every piece of armour worn (anything worn with Protection of its own)";
            }
            if let Some(at) = body_armour {
                text += &format!(
                    " while the armour piece worn with the most Protection has {at} or more"
                );
            }
            if shield {
                text += " while holding a shield";
            }
            text
        }
        TalentEffect::StanceToHit { stance, percent } => {
            format!(
                "{} stance's to-hit penalty {percent} % smaller",
                stance.id()
            )
        }
        TalentEffect::Pain { value } => {
            format!(
                "every hit on you {} damage after the soak, never below 1",
                signed(-value)
            )
        }
        TalentEffect::Resist {
            value,
            damage: None,
        } => format!("resistance to the chosen element +{value} %"),
        TalentEffect::Resist {
            value,
            damage: Some(kind),
        } => {
            let name = (content.damage_type_id(&kind))
                .map_or(kind.clone(), |d| content.damage_type(d).name.to_lowercase());
            format!("resistance to {name} +{value} %")
        }
        TalentEffect::SlipPercent { value } => format!(
            "a step out of reach of every hostile beside you costs {}, whatever your stance",
            percent(value)
        ),
        TalentEffect::Streak { max, damage_every } => {
            let damage = if damage_every > 0 {
                format!(", and +1 damage for every {damage_every} blows before it")
            } else {
                String::new()
            };
            format!(
                "each melee blow at the creature your last blow was at (this turn or the one \
                 before) +1 to hit per blow before it, to +{max}{damage}; anything else starts \
                 it over"
            )
        }
        TalentEffect::OffHandDamage { value } => {
            format!("off-hand blows {} damage", signed(value))
        }
        TalentEffect::BestSkillThreat { value } => format!(
            "attacks with your best weapon skill (the highest rank you hold of the melee, \
             unarmed, launcher, and thrown skills; ties all count) threaten a critical {value} \
             lower"
        ),
        TalentEffect::Barbs { damage } => format!(
            "a creature that hits you in melee takes {damage} damage (no roll; a death of it \
             earns no experience)"
        ),
        TalentEffect::WoundMore { value } => {
            format!("a big hit that wounds you wounds {value} degree more")
        }
        TalentEffect::Distrusted => {
            "no one makes way for you, whatever else would have them (Noblesse too)".to_string()
        }
        TalentEffect::HornsGrace {
            cures,
            diseases,
            cooldown,
            degrees,
        } => {
            let names: Vec<String> = cures.iter().map(|c| status(c)).collect();
            format!(
                "active (the time of a wait): {} end and the first {diseases} of the diseases \
                 that show are shaken off (refused with nothing to mend); then {}",
                names.join(" and "),
                cooldown_for(content, &cooldown, degrees)
            )
        }
        TalentEffect::HornOfPlenty { per_level, degrees } => format!(
            "Horn's grace also gives back {per_level} hit point per level (never past your \
             maximum; it may then be used with only wounds to mend), and leaves {} instead",
            sign_cooldown(content, SignActive::HornsGrace, degrees)
        ),
        TalentEffect::Stare {
            status: id,
            severity,
            cooldown,
            degrees,
        } => {
            let (cost, eases) = (content.status_id(&id)).map_or((100, None), |s| {
                let def = content.status(s);
                (def.cost, def.eases)
            });
            let eases =
                eases.map_or_else(String::new, |e| format!(", a degree easing every {e} s"));
            format!(
                "active (the time of a wait, silent): a hostile you see within {} cells that \
                 sees you suffers {} at {severity} degrees (its actions cost {cost} %{eases}); \
                 one that does not see you is not held, and the cooldown waits; then {}",
                warlock_core::world::STARE_RANGE,
                status(&id),
                cooldown_for(content, &cooldown, degrees)
            )
        }
        TalentEffect::PetrifyingGaze { severity, degrees } => format!(
            "the Transfixing stare leaves {severity} degrees of its status, and leaves {} instead",
            sign_cooldown(content, SignActive::TransfixingStare, degrees)
        ),
        TalentEffect::Wildfire { radius, damage } => {
            format!("Blaze bursts out to radius {radius} and does {damage} instead")
        }
        TalentEffect::FireBreath {
            length,
            damage,
            damage_type,
            cooldown,
            degrees,
            ..
        } => {
            let kind = (content.damage_type_id(&damage_type)).map_or(damage_type.clone(), |d| {
                content.damage_type(d).name.to_lowercase()
            });
            format!(
                "active (the time of a cast; not on the overworld), aimed at a cell within {} \
                 cells: a bolt along the line toward it and on, {length} cells long or to the \
                 first wall, {damage} {kind} damage (one roll) to everyone on it; then {}",
                warlock_core::world::BREATH_RANGE,
                cooldown_for(content, &cooldown, degrees)
            )
        }
        TalentEffect::HoundsBay {
            status: id,
            severity,
            cooldown,
        } => {
            let eases = (content.status_id(&id))
                .and_then(|s| content.status(s).eases)
                .map_or_else(String::new, |e| format!(", a degree easing every {e} s"));
            format!(
                "active (the time of a wait; not on the overworld): a howl of loudness {}; \
                 every hostile that hears it and whose level is below yours suffers {} at \
                 {severity} degrees (it runs from you while a step takes it farther{eases}); \
                 no check; then {}",
                warlock_core::noise::HOWL_NOISE,
                status(&id),
                cooldown_for(content, &cooldown, 1)
            )
        }
        TalentEffect::RestingPercent { value } => {
            format!("healing pace while resting {}", percent(value))
        }
        TalentEffect::RiseFromAshes {
            hp_percent,
            status: id,
            radius,
            damage,
            damage_type,
            ..
        } => format!(
            "once in your life, a hit, a trap, or a status that would kill you does not (starving \
             and choking still do): you rise with {hp_percent} % of your maximum hit points \
             (Ashen ones), {damage} {damage_type} damage strikes every creature within {radius} \
             cell (one roll; no roll to hit) and the ice there melts, and you are {}",
            status(&id)
        ),
        TalentEffect::RebornWhole => "rising from the ashes brings you back whole: every hit \
             point, every status effect and disease gone (nothing follows them), and no Ashen"
            .to_string(),
        TalentEffect::SecondRising => {
            "you may rise from the ashes a second time in your life".to_string()
        }
        TalentEffect::StarvingHeals => {
            "hit points come back while you are Starving too (at your usual pace)".to_string()
        }
        TalentEffect::EatsRotten => "rotten food never harms you: no vomiting, no sickness, no \
             disease (as a race that eats rotten food)"
            .to_string(),
        TalentEffect::WadesFree => "wading (water, and ground a disease is caught wading in) \
             costs you a plain step; a disease may still be caught"
            .to_string(),
        TalentEffect::SecondWind {
            per_level,
            at_least,
            cooldown,
        } => format!(
            "active (the time of a wait): {per_level} hit point per level back at once, at \
             least {at_least}; then {} (no second wind while it lasts)",
            status(&cooldown)
        ),
        TalentEffect::IronStomach => "rotten food never makes you vomit and sickens you half \
             as much (degrees halved, rounded down); overeating never makes you vomit"
            .to_string(),
        TalentEffect::StatusEases { status: id, value } => {
            format!("{} eases {value} % faster", status(&id))
        }
        TalentEffect::StatusLasts { status: id, value } => {
            format!("{} lasts {value} % longer", status(&id))
        }
        TalentEffect::NoPenalty { status: id } => {
            format!("{} never lowers your scores", status(&id))
        }
        TalentEffect::Clotting => {
            "a big hit wounds you no more; a confirmed critical one degree less".to_string()
        }
        TalentEffect::PenaltyPercent { value } => format!(
            "what statuses take off your scores {} (each, rounded toward 0)",
            percent(value)
        ),
        TalentEffect::DiseaseChecks { value } => {
            format!("disease checks (catching, progressions) {}", signed(value))
        }
        TalentEffect::DrainChecks { value } => {
            format!("checks against a drain {}", signed(value))
        }
        TalentEffect::Checks { stat, value } => {
            format!("every {} check {}", stat.name(), signed(value))
        }
        TalentEffect::DiseaseIntervalPercent { value } => format!(
            "a disease's progressions come {} further apart",
            percent(value)
        ),
        TalentEffect::DiseaseRecovery { value } => format!(
            "a disease goes after {value} fewer checks won in a row (at least 1; never \
             one that does not recover)"
        ),
        TalentEffect::KickDie { sides } => format!("kicks at creatures roll 1d{sides}"),
        TalentEffect::DoorKick { value } => {
            format!("door kicks as if Strength {}", signed(value))
        }
        TalentEffect::KickNoisePercent { value } => {
            format!("door kicks' noise {}", percent(value))
        }
        TalentEffect::SmashDestroyPercent { value } => format!(
            "the chance that each stack in a chest it kicks open breaks {} (a chest blown apart \
             is no kick)",
            percent(value)
        ),
        TalentEffect::DigPercent { value } => format!("digging work a turn {}", percent(value)),
        TalentEffect::TwoHandedPrice { value } => {
            format!("blows with a two-handed weapon cost {}", percent(value))
        }
        TalentEffect::AttackPricePercent { value } => {
            format!("melee attacks cost {}", percent(value))
        }
        TalentEffect::CriticalStatus { status: id } => format!(
            "a critical you confirm in melee leaves the target {} (one degree)",
            status(&id)
        ),
        TalentEffect::Grip { status: id } => format!(
            "a creature you hit in melee is {} (it cannot step; beside you, it attacks you \
             instead) until you miss it, strike another, or move; no roll",
            status(&id)
        ),
        TalentEffect::CriticalConfirm { value } => {
            format!("the roll confirming your criticals {}", signed(value))
        }
        TalentEffect::AbilityRanks { value } => format!(
            "weapon abilities come {value} rank{} sooner (their thresholds read your ranks {} \
             higher)",
            if value == 1 { "" } else { "s" },
            signed(value)
        ),
        TalentEffect::RendOffHand => "Rend rolls the off hand's damage dice too".into(),
        TalentEffect::WeaponEnergyPercent { value } => format!(
            "weapon actives take {}% time (never below half the action's price)",
            signed(value)
        ),
        TalentEffect::WeaponStamina { value, abilities } if abilities.is_empty() => {
            format!("every weapon active costs {value} stamina less (at least 1)")
        }
        TalentEffect::WeaponStamina { value, abilities } => {
            let names: Vec<String> = (abilities.iter())
                .map(|a| {
                    (content.weapon_ability_id(a))
                        .map_or_else(|| a.clone(), |id| content.weapon_ability(id).name.clone())
                })
                .collect();
            format!(
                "{} cost {value} stamina less (at least 1)",
                names.join(" and ")
            )
        }
        TalentEffect::Brace { to_hit, status: id } => format!(
            "Brace's blow {} to hit, and a foe it hits is {} (one degree)",
            signed(to_hit),
            status(&id)
        ),
        TalentEffect::CriticalPenetration { value } => {
            format!("your confirmed criticals: penetration {}", signed(value))
        }
        TalentEffect::Penetration { damage, value } => {
            let name = (content.damage_type_id(&damage)).map_or(damage.clone(), |d| {
                content.damage_type(d).name.to_lowercase()
            });
            format!(
                "blows that can do {name} damage: penetration {}",
                signed(value)
            )
        }
        TalentEffect::BurdenDex { value } => {
            format!("the burden's cap on Dexterity in Defence {}", signed(value))
        }
        TalentEffect::BurdenEase { value } => format!(
            "{value} % off the burden's slowdown, on top of Balance (Burdened ×1.10 for 60 %)"
        ),
        TalentEffect::SureStep => "a trap you step onto first gets a Dexterity check against \
             its difficulty: made, it does not go off, and you find it"
            .to_string(),
        TalentEffect::PitProof => "pits never catch you (you find them)".to_string(),
        TalentEffect::Disarm => {
            "the Disarm ability without Disable device (Dexterity alone)".to_string()
        }
        TalentEffect::SpeedPercent { value } => format!("speed {}", percent(value)),
        TalentEffect::MissileDodge { value } => {
            format!(
                "Dodge {} against shots, throws, and trap missiles",
                signed(value)
            )
        }
        TalentEffect::SmallTarget { value } => format!(
            "Dodge {} against the attacks of creatures larger than you",
            signed(value)
        ),
        TalentEffect::SpellDamagePercent { value } => format!(
            "damage from bursts and bolts (traps, scrolls) {} (rounded down)",
            percent(value)
        ),
        TalentEffect::Spell { spell } => (content.spell_id(&spell))
            .map(|id| content.spell(id))
            .map_or_else(|| spell.clone(), |def| spell_text(content, def)),
        TalentEffect::SpellMod(spec) => spell_mod_text(content, &spec, false),
        TalentEffect::SpellManaPercent { value } => {
            format!("your spells' mana {} (at least 1)", percent(value))
        }
        TalentEffect::DeflectionUnarmoured { value } => {
            format!("Deflection {} while you wear no body armour", signed(value))
        }
        TalentEffect::Purify => {
            "rotten food never sickens you and never exposes you to a disease (it may still make \
             you vomit)"
                .to_string()
        }
        TalentEffect::Communion => {
            "while the dark answers your prayers (Callous or worse), standing in an unholy place \
             (a cursed, draining, or haunted room, or a level such a curse covers): your prayers \
             add no strain, and the place's status (Ebbing) never touches you"
                .to_string()
        }
        TalentEffect::CorpseHarmless => {
            "a fresh corpse's own harm (its sickness and its vomiting) never touches you"
                .to_string()
        }
        TalentEffect::MealHeals { object, dice } => {
            let what = object.map_or_else(
                || "a fresh corpse".to_string(),
                |o| (content.object_id(&o)).map_or(o, |id| content.object(id).name.clone()),
            );
            format!("eating {what} heals {dice} hit points besides")
        }
        TalentEffect::StatusDamage { status: id, value } => {
            format!(
                "{} damage on hits and spells against a creature {}",
                signed(value),
                status(&id)
            )
        }
        TalentEffect::HexTithe { per_levels } => format!(
            "when a creature dies under a hex you laid, whoever killed it, your mana rises by its \
             level divided by {per_levels} (rounded down, at least 1), never past your maximum"
        ),
        TalentEffect::Reprieve {
            status: id,
            stat,
            by,
            ..
        } => {
            let stat = stat.map_or_else(String::new, |s| {
                format!(", and {} {} for good", s.name(), signed(by))
            });
            format!(
                "once in a life, what would kill you (not starving or choking) leaves you at 1 hit \
                 point, {}{stat}",
                status(&id)
            )
        }
        TalentEffect::FavouredPrey { to_hit, damage } => format!(
            "against the creature type you have killed most (ties: the first to get there), \
             to hit {} and damage {}",
            signed(to_hit),
            signed(damage)
        ),
        TalentEffect::SwornFoe { to_hit, damage } => format!(
            "against creatures of the kind chosen (the creature tag; the tags and who carries \
             them: [Sorts of creatures](quests.md#sorts-of-creatures)), to hit {} and damage {}",
            signed(to_hit),
            signed(damage)
        ),
        TalentEffect::ExactSounds => {
            "every sound you hear is placed at the cell it came from, not roughly".to_string()
        }
        TalentEffect::SilentUnburdened => "no footsteps at all while Unburdened".to_string(),
        TalentEffect::AttackNoisePercent { value } => {
            format!("the noise of your attacks {}", percent(value))
        }
        TalentEffect::TrapEase { value } => {
            format!("hidden traps {value} easier to find (not to disarm)")
        }
        TalentEffect::PickLock => "the Pick lock ability without Open lock".to_string(),
        TalentEffect::Steal => "the Steal ability without the Steal skill".to_string(),
        TalentEffect::ThrowVoice => "the Throw your voice ability".to_string(),
        TalentEffect::AmbushThreat { value } => format!(
            "attacks on a creature that has not seen you threaten a critical {value} lower \
             (never below 3)"
        ),
        TalentEffect::Finisher => "active (the time of a melee attack), in a direction: a blow \
                                   that is a critical whenever it hits, with no confirmation roll"
            .to_string(),
        TalentEffect::PatientDefence { status: id } => format!(
            "active (the time of a wait): you are {} until your next action; a melee blow you \
             dodge (it misses by your Dexterity or Dodge) or parry meanwhile is answered with \
             your blow at once (a riposte, a free blow)",
            status(&id)
        ),
        TalentEffect::Lurk { status: id } => format!(
            "active (the time of a wait): you are {} until your next action; a hostile that steps \
             up beside you meanwhile takes your blow at once (a free blow), caught unawares \
             unless it was already hunting you; refused with a hostile you see beside you",
            status(&id)
        ),
        TalentEffect::LowBlow => "active (the time of a melee attack), in a direction: a blow \
                                  that is a sneak attack whatever the foe knows (the sneak \
                                  attack dice on a hit; the foe is not surprised and keeps its \
                                  Defence; no dice on a thing without vitals)"
            .to_string(),
        TalentEffect::TrainedSkills { value } => {
            format!("every trained skill counts {}", signed(value))
        }
        TalentEffect::GrantsResource { resource } => {
            format!(
                "you hold {} from now on (see Resources)",
                resource_name(content, &resource)
            )
        }
        TalentEffect::RapidShot {
            resource,
            cost,
            saving,
        } => format!(
            "a shot you loose while you hold {cost} {} spends them and takes {saving} % less time",
            resource_name(content, &resource).to_lowercase()
        ),
        TalentEffect::FlurryCost { resource, cost } => format!(
            "each second blow of the flurry spends {cost} {}; with less, there is none",
            resource_name(content, &resource).to_lowercase()
        ),
        TalentEffect::Retribution { resource, cost } => format!(
            "a hostile beside you whose melee attack lands a blow on you takes your plain smite \
             back at once (its dice, its aim, and its unholy die), free but for an exertion, spending {cost} {}; with less, or outside the talent's alignment, \
             nothing; an attack it answers is not also riposted",
            resource_name(content, &resource).to_lowercase()
        ),
        TalentEffect::CatchMissiles { resource, cost } => format!(
            "a shot or throw (a trap's too) that would hit you while you hold no weapon and no \
             shield spends {cost} {} by itself and misses, the missile falling at your feet; \
             with less, it hits",
            resource_name(content, &resource).to_lowercase()
        ),
        TalentEffect::Volley { shots } => format!(
            "active (the time of one shot, one exertion), at a cell: {shots} shots of the readied \
             missile weapon"
        ),
        TalentEffect::Song {
            on,
            status: id,
            severity,
            resource,
            cost,
            mana,
            check,
            difficulty,
            ..
        } => {
            let price = match resource {
                Some(r) => format!("{cost} {}", resource_name(content, &r).to_lowercase()),
                None => format!("{mana} mana"),
            };
            let degrees = match severity {
                1 => "1 degree".to_string(),
                n => format!("{n} degrees"),
            };
            let whom = match (on, check) {
                (SongOn::Hearers, Some(stat)) => format!(
                    "each hostile creature that hears it checks {} against {difficulty} plus \
                     your Music's song difficulty; failed, it suffers",
                    stat.name()
                ),
                _ => "you suffer".to_string(),
            };
            format!(
                "active, a song (the time of a song, heard as a voice of loudness {SONG_NOISE}; \
                 not on the overworld or while silenced), {price}: {whom} {} at {degrees}, plus \
                 your Music's song degrees",
                buff(content, &id)
            )
        }
        TalentEffect::Virtuoso { status: id } => format!(
            "every song costs nothing and gives a degree more; a song on yourself gives {} instead",
            buff(content, &id)
        ),
        TalentEffect::Consecrate { mana, time } => format!(
            "active ({time} of prayer, for {mana} mana; interrupted as a rest is, it lifts \
             nothing): standing in a cursed or draining room, it becomes a plain room for good; \
             on a level a curse or a drain covers whole, the whole level's is lifted instead. \
             A good deed (see Alignment). Anywhere else it is refused, with no time spent"
        ),
        TalentEffect::Sanctuary {
            mana,
            time,
            radius,
            lasts,
            regen,
        } => format!(
            "active ({time} of kneeling, for {mana} mana; interrupted as a rest is, it lays \
             nothing): the ground within {radius} cells of where the player knelt is hallowed \
             for {lasts}. No creature hostile to them steps onto it from outside (one already \
             on it may move about and leave; nothing spawns there; missiles and spells cross \
             it), and whoever stands on it heals and regains mana at {regen} % of the pace, or \
             a special room's, the larger. A blow, shot, throw, or harmful spell of the \
             player's from on it breaks it; being struck does not. The same for either face of \
             the prayer (dreaded ground for the dark: the words alone). Refused, with no time \
             spent, on the overworld, in water, in a shop, or on hallowed ground"
        ),
        TalentEffect::Prayer {
            tier,
            title,
            uses,
            answers,
            all,
        } => prayer_text(content, tier, title.as_ref(), &uses, answers, all),
        TalentEffect::Knock { mana } => format!(
            "active (aimed in a direction, for {mana} mana, the time of a spell): the lock of \
             the locked door or shut chest there springs open; nothing opens and no trap \
             springs. Where nothing locked stands it is refused, with no time or mana spent"
        ),
        TalentEffect::Scribe { mana_times } => format!(
            "active (a spell you know, from the scribe list): on the first [blank \
             scroll](scrolls.md#writing-scrolls) you carry, a scroll of the spell is written, for \
             {mana_times} times the spell's mana (what casting it would cost before Frugal \
             caster's cut; Frugal caster then cuts the whole) and as many turns of writing as a \
             study of the spell takes (Quick study does not shorten it). The mana is spent and \
             the blank used up only at the end; interrupted, nothing is spent. Refused, with no \
             time spent, without a blank scroll or the mana"
        ),
        TalentEffect::MerchantsEye => format!(
            "active (on a thing you carry, for the time of reading a scroll, {} at normal pace; \
             no mana, cooldown, or check, as often as you like): the thing is identified as a \
             scroll of identify would identify it. On a thing you already know it is refused, \
             with no time spent",
            super::actions::at_normal_pace(warlock_core::content::price(
                content
                    .action_costs()
                    .cost(warlock_core::content::ActionKind::Read),
                warlock_core::content::BASE_TURN
            ))
        ),
        TalentEffect::SpiritWeapon {
            to_hit,
            threat,
            ignores_protection,
            ..
        } => {
            let armour = match ignores_protection {
                true => ", and ignores every point of the target's Protection",
                false => "",
            };
            format!(
                "active (in the shape of any melee weapon, for the time of wielding one, {} at \
                 normal pace, with what the hands held put away as any swap; no mana, cooldown, \
                 or check, as often as you like): a weapon of spirit-stuff of that shape, made \
                 and weighing as its plain base, is put in your main hand (both, for a two-handed \
                 one), and one you hold fades first. It strikes {} to hit, threatens criticals \
                 {threat} lower{armour}; it fades the moment it leaves your hands (put away, \
                 swapped, dropped, sold) and cannot be thrown",
                super::actions::at_normal_pace(warlock_core::content::price(
                    content
                        .action_costs()
                        .cost(warlock_core::content::ActionKind::Wield),
                    warlock_core::content::BASE_TURN
                )),
                signed(to_hit),
            )
        }
    }
}

/// A caster's spell, as the class tables say it: what it does, its spell level and mana, and its
/// time.
fn spell_text(content: &Content, def: &SpellData) -> String {
    let casts = match def.casts {
        1 => "a cast".to_string(),
        n => format!("{n} casts"),
    };
    let mana = content.spell_rules().map_or(0, |r| r.mana(def, 0));
    format!(
        "spell `use:{}` (level {}, {casts}, {mana} mana): {}: {}",
        def.id,
        def.level,
        def.name,
        super::spells::does(content, def)
    )
}
