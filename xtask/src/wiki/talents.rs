//! Talents (spec §8 "Talents"): every tree with its talents, their prerequisites and their rules
//! in full (the game says them in words), and the talents by system.

use super::util::{Term, anchored, link, signed};
use std::fmt::Write as _;
use warlock_core::content::{
    Content, Faced, Keep, KitLineSpec, Need, PrayerUse, SignActive, SpellData, SpellModSpec,
    StatusId, Table, TalentDef, TalentEffect, TreeId,
};
use warlock_core::talent::{CraftEffect, FortuneEffect, MagicEffect, SignEffect, StealthEffect};

mod ally;
mod beast;
mod craft;
mod defence;
mod delving;
mod faith;
mod fortune;
mod growth;
mod hunger;
mod magic;
mod martial;
mod movement;
mod offence;
mod resource;
mod senses;
mod sign;
mod song;
mod stealth;
mod vigour;

/// The tree of the birth signs' gifts: on the horoscope page, not the talents page (#1554).
const STARS: &str = "stars";

/// The page a talent's row is on (#1554): a birth sign's gift on the horoscope page, a class
/// ability in its first class's level table, on that class's page (else its first prestige
/// class's), a profane gift on the profane gifts page, any other on the talents page.
fn talent_page(content: &Content, def: &TalentDef) -> String {
    let book = content.talents();
    if book.tree_id(STARS) == Some(def.tree) {
        "horoscope.md".into()
    } else if book.tree(def.tree).id == "profane" {
        "profane-gifts.md".into()
    } else if book.tree(def.tree).class {
        match home_table(content, &def.id) {
            Some(Table::Prestige(_)) => "prestige-classes.md".into(),
            Some(Table::Class(class)) => super::character::class_page(&class.id),
            None => "classes.md".into(),
        }
    } else {
        "talents.md".into()
    }
}

/// The level table a class ability's row is anchored in: the first class's (in New run's
/// order) that teaches it, else the first prestige class's.
fn home_table<'c>(content: &'c Content, id: &str) -> Option<Table<'c>> {
    let teaches = |t: &Table| t.abilities().iter().any(|(_, a)| a == id);
    (content.classes().iter().map(Table::Class))
        .chain(content.prestiges().iter().map(Table::Prestige))
        .find(teaches)
}

/// A link to the row that defines talent `id`, on whichever page it is (#1554): the talents
/// page, the horoscope page, or a class's level table.
pub(super) fn talent_link(content: &Content, id: &str, name: &str) -> String {
    let anchor = link(Term::Talent, id, name);
    match content.talents().id(id) {
        Some(t) => {
            let page = talent_page(content, content.talents().get(t));
            anchor.replacen("](talents.md#", &format!("]({page}#"), 1)
        }
        None => anchor,
    }
}

/// Every tree a talent may be picked from, in display order: a heading, its description, and its
/// talents (name, what it needs, and what it does as the rule). The stars' gifts are on the
/// horoscope page and the class abilities in the class tables (#1554).
pub(super) fn talents(content: &Content) -> String {
    let book = content.talents();
    let mut out = String::new();
    for (i, tree) in (0u8..).zip(&book.trees) {
        // The profane gifts are no picks: they have their own page (`profane.rs`).
        if tree.class || book.tree_id(STARS) == Some(TreeId(i)) || tree.id == "profane" {
            continue;
        }
        let _ = writeln!(out, "### {}\n\n{}\n", tree.name, tree.description);
        tree_table(content, TreeId(i), &mut out);
    }
    out
}

/// One tree's table: each talent with what it needs and what it does.
fn tree_table(content: &Content, tree: TreeId, out: &mut String) {
    let rows: Vec<&TalentDef> = content.talents().of_tree(tree).map(|(_, t)| t).collect();
    if rows.is_empty() {
        out.push_str("No talents yet.\n\n");
        return;
    }
    out.push_str("| Talent | Needs | Effect |\n|---|---|---|\n");
    for def in rows {
        let _ = writeln!(
            out,
            "| {} | {} | {} |",
            anchored(Term::Talent, &def.id, &name(def)),
            needs_text(content, def),
            effects_text(content, def),
        );
    }
    out.push('\n');
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
            let TalentEffect::Fortune(FortuneEffect::Heirloom {
                weapons, fallback, ..
            }) = effect
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

/// The Crafts trades (the craftsperson design §3): each trade's skill, its dice, the tools of
/// the trade, and the races it is closed to (those whose people all have the skill), trades by
/// name.
pub(super) fn trades(content: &Content) -> String {
    let mut rows: Vec<&TalentDef> = (content.talents().iter())
        .map(|(_, t)| t)
        .filter(|t| {
            (t.effects.iter()).any(|e| matches!(e, TalentEffect::Craft(CraftEffect::Trade { .. })))
        })
        .collect();
    rows.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out = String::from(
        "| Trade | Skill | Dice | Tools of the trade | Closed to |\n|---|---|---|---|---|\n",
    );
    for def in rows {
        for effect in &def.effects {
            let TalentEffect::Craft(CraftEffect::Trade { skill, dice, tools }) = effect else {
                continue;
            };
            let Some(id) = content.skill_id(skill) else {
                continue;
            };
            let closed: Vec<String> = (def.needs.iter())
                .filter_map(|n| match n {
                    Need::RaceLacks(skill) => Some(skill),
                    _ => None,
                })
                .flat_map(|skill| content.races().teaching(skill))
                .map(|r| link(Term::Race, &r.id, &r.name))
                .collect();
            let _ = writeln!(
                out,
                "| {} | {} | {dice} | {} | {} |",
                link(Term::Talent, &def.id, &def.name),
                link(Term::Skill, skill, &content.skill(id).name),
                tools_text(content, tools),
                if closed.is_empty() {
                    "—".to_string()
                } else {
                    closed.join(", ")
                }
            );
        }
    }
    out
}

/// A trade's tools in words: each line's alternatives joined by "or", lines by commas, a
/// readied one marked.
fn tools_text(content: &Content, tools: &[KitLineSpec]) -> String {
    let Ok(kit) = content.kit_lines(tools) else {
        return "—".to_string();
    };
    let lines: Vec<String> = (kit.items().iter())
        .map(|line| {
            let one_of: Vec<String> = (line.one_of.iter())
                .map(|stacks| {
                    (stacks.iter())
                        .map(|s| s.describe(content))
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .collect();
            let text = one_of.join(" or ");
            match line.keep {
                Keep::Carry => text,
                Keep::Wield => format!("{text} (readied if its slot is free)"),
                Keep::Wear => format!("{text} (worn if its slot is free)"),
            }
        })
        .collect();
    if lines.is_empty() {
        "—".to_string()
    } else {
        lines.join(", ")
    }
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

/// One class's (or prestige class's) table: a prestige class's heading (a class's page has its
/// own, and its card says what the class is), its bonus picks, then each row's ability with what it does, between mdBook
/// anchors named by its id, so a page can include one class's section
/// (`{{#include generated/class-levels.md:paladin}}`) and follow it with prose.
fn level_table(content: &Content, table: Table, out: &mut String) {
    let book = content.talents();
    let bonus = table.bonus_talents();
    if table.abilities().is_empty() && bonus.levels.is_empty() {
        return;
    }
    let _ = writeln!(out, "<!-- ANCHOR: {} -->\n", table.id());
    // A class's page has its own heading, and what the class is in its card
    // (`class_guide::cards`).
    if let Table::Prestige(_) = table {
        let _ = writeln!(out, "### {}\n", table.name());
    }
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
    let creation = match table {
        Table::Class(class) => (class.creation_bonus.as_ref()).zip(class.creation_trees(book)),
        Table::Prestige(_) => None,
    };
    if let Some((bonus, trees)) = creation {
        let trees: Vec<String> = (trees.iter()).map(|&t| book.tree(t).name.clone()).collect();
        let _ = writeln!(
            out,
            "{} bonus talent{} at character creation, from {}, taken before the others.\n",
            [
                "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight"
            ][usize::from(bonus.count.clamp(1, 8)) - 1],
            if bonus.count == 1 { "" } else { "s" },
            trees.join(" or ")
        );
    }
    out.push_str("| Level | Ability | Effect |\n|---:|---|---|\n");
    let mut seen: Vec<&str> = Vec::new();
    for (level, id) in table.abilities() {
        let Some(talent) = book.id(id) else { continue };
        let def = book.get(talent);
        let effects = effects_text(content, def);
        // A class ability's row is defined here, in its first table, at its first level
        // (#1554); a later one links to it.
        let home = book.tree(def.tree).class
            && home_table(content, id).is_some_and(|t| t.id() == table.id());
        let ability = if home && !seen.contains(&id.as_str()) {
            anchored(Term::Talent, &def.id, &def.name)
        } else if home {
            def.name.clone()
        } else {
            talent_link(content, &def.id, &def.name)
        };
        seen.push(id);
        let _ = writeln!(out, "| {level} | {ability} | {effects} |");
    }
    out.push('\n');
    let _ = writeln!(out, "<!-- ANCHOR_END: {} -->\n", table.id());
}

/// The talents by system: every system of `data/talents/systems/` with the talents
/// that touch it. Systems and talents sort by name, never by file order.
pub(super) fn by_system(content: &Content) -> String {
    let book = content.talents();
    let mut systems: Vec<_> = (0u8..).zip(&book.systems).collect();
    systems.sort_by(|a, b| a.1.name.cmp(&b.1.name));
    let mut out = String::from("| System | Talents |\n|---|---|\n");
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
        let talents = if names.is_empty() {
            "—".to_string()
        } else {
            names.join(", ")
        };
        let _ = writeln!(out, "| {} | {talents} |", system.name);
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

/// The prerequisites; "—" for none; a granted talent is never picked.
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
    if def.needs.is_empty() {
        return "—".to_string();
    }
    (def.needs.iter())
        .map(|n| match n {
            Need::Creation => "only at character creation".to_string(),
            n => n.words(content),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// What the talent does, as the rule.
pub(super) fn effects_text(content: &Content, def: &TalentDef) -> String {
    let chosen = def.choice.is_some();
    let text_of = |e: &TalentEffect| match e {
        TalentEffect::Magic(MagicEffect::SpellMod(spec)) => spell_mod_text(content, spec, chosen),
        other => effect_text(content, other),
    };
    // An effect under a status (`while`): its text, then "while raging" (the status's name).
    let under = |(status, e): &(StatusId, TalentEffect)| {
        let name = content.status(*status).name.to_lowercase();
        format!("{} while {name}", text_of(e))
    };
    let plain = (def.effects.iter()).map(text_of);
    let mut parts: Vec<String> = plain.chain(def.effects_while.iter().map(under)).collect();
    if let Some(cost) = def.resource {
        let name = content.resource(cost.resource).name.to_lowercase();
        // Exploit spends every one held; its cost is the least it needs.
        if (def.effects.iter()).any(|e| {
            matches!(
                e,
                TalentEffect::Stealth(StealthEffect::Exploit)
                    | TalentEffect::Stealth(StealthEffect::ThrownExploit)
            )
        }) {
            parts.push(format!(
                "spends all the {name} held (at least {})",
                cost.amount
            ));
        } else {
            parts.push(format!("costs {} {name}", cost.amount));
        }
    }
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
            TalentEffect::Sign(SignEffect::HornsGrace { cooldown, .. })
            | TalentEffect::Sign(SignEffect::Stare { cooldown, .. }) => Some(cooldown.clone()),
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
            "active (the time of a spell, no mana): it answers the first problem \
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
    match effect {
        TalentEffect::Beast(effect) => beast::text(effect),
        TalentEffect::Offence(effect) => offence::text(content, effect),
        TalentEffect::Defence(effect) => defence::text(content, effect),
        TalentEffect::Vigour(effect) => vigour::text(content, effect),
        TalentEffect::Hunger(effect) => hunger::text(content, effect),
        TalentEffect::Magic(effect) => magic::text(content, effect),
        TalentEffect::Senses(effect) => senses::text(content, effect),
        TalentEffect::Stealth(effect) => stealth::text(content, effect),
        TalentEffect::Movement(effect) => movement::text(content, effect),
        TalentEffect::Delving(effect) => delving::text(content, effect),
        TalentEffect::Fortune(effect) => fortune::text(content, effect),
        TalentEffect::Growth(effect) => growth::text(content, effect),
        TalentEffect::Craft(effect) => craft::text(content, effect),
        TalentEffect::Sign(effect) => sign::text(content, effect),
        TalentEffect::Faith(effect) => faith::text(content, effect),
        TalentEffect::Martial(effect) => martial::text(content, effect),
        TalentEffect::Resource(effect) => resource::text(content, effect),
        TalentEffect::Song(effect) => song::text(content, effect),
        TalentEffect::Ally(effect) => ally::text(content, effect),
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
        "the spell {} (level {}, {casts}, {mana} mana): {}",
        link(Term::Spell, &def.id, &def.name),
        def.level,
        super::spells::does(content, def)
    )
}
