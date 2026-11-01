//! The schools of magic tables (spec §8 "Weapon skills": "Schools of magic"; the spell scaling
//! design §8), from `data/weapon_skills.toml`'s rows of role `school`: each school with its
//! spells, its defaults, and the classes that start ranked in it; and what each rank gives. The
//! caster styles' ranks too (`grip` a caster's: where a spell is cast from), on the same page.

use super::text::{Term, anchored, link};
use super::weapon_skills::{Kind, default_list, grip_text, rank_table};
use std::fmt::Write as _;
use warlock_core::content::{Content, Role, WeaponSkillId};

/// The schools in file order (the design's order of the seven).
fn schools(content: &Content) -> Vec<(WeaponSkillId, &warlock_core::content::WeaponSkillDef)> {
    (content.weapon_skills().ids())
        .filter(|(_, s)| s.role == Role::School)
        .collect()
}

/// Each school: how many of the shipped spells are of it, the curve it trains along, and the
/// classes that start with a rank in it (by class name), then the defaults between schools.
pub fn skills(content: &Content) -> String {
    let mut out = String::from(
        "| School | Spells | Marks | Starting ranks by class |\n|---|:---:|:---:|---|\n",
    );
    for (id, s) in schools(content) {
        let spells = (content.spells().iter())
            .filter(|spell| Some(spell.school) == s.school)
            .count();
        let mut classes: Vec<(String, String)> = (content.classes().iter())
            .filter_map(|c| {
                let (_, dice) = c.weapon_skills.iter().find(|(skill, _)| *skill == id)?;
                let rank = if dice.is_fixed() {
                    dice.max().to_string()
                } else {
                    super::character::adjustment(*dice)
                };
                Some((c.name.clone(), rank))
            })
            .collect();
        classes.sort();
        let classes: Vec<String> = (classes.into_iter())
            .map(|(name, rank)| format!("{name} {rank}"))
            .collect();
        let classes = if classes.is_empty() {
            "-".to_string()
        } else {
            classes.join(", ")
        };
        let _ = writeln!(
            out,
            "| {} | {spells} | {} | {classes} |",
            anchored(Term::School, &s.id, &s.name),
            s.curve
        );
    }
    out.push('\n');
    out.push_str(&default_list(content.weapon_skills(), Kind::Schools));
    out
}

/// What each rank of each school gives: a table per school, ranks 1 to 10, with the benefits
/// it has (its to-hit and critical range count for its spells that roll to hit).
pub fn benefits(content: &Content) -> String {
    let table = content.weapon_skills();
    let mut out = String::new();
    for (id, s) in schools(content) {
        let _ = writeln!(out, "**{}**\n", link(Term::School, &s.id, &s.name));
        out.push_str(&rank_table(table, id));
        out.push('\n');
    }
    out
}

/// The caster styles (where a spell is cast from; the spell scaling design §10.2), in file
/// order: each by name with where it is cast from and the curve it trains along, and what each
/// rank gives, a table per style.
pub fn caster_style_benefits(content: &Content) -> String {
    let table = content.weapon_skills();
    let mut out = format!(
        "A spell cast from afar is far away at **{}** cells or more (the larger of the two \
         distances).\n\n",
        table.spell_attacks().far
    );
    for (id, s, style) in table.styles().filter(|(_, _, s)| s.grip.casts()) {
        let _ = writeln!(
            out,
            "**{}**: {}; marks on the {} curve.\n",
            anchored(Term::CasterStyle, &s.id, &s.name),
            grip_text(style.grip),
            s.curve
        );
        out.push_str(&rank_table(table, id));
        out.push('\n');
    }
    out
}
