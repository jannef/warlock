//! The Schools and caster styles page's tables: the seven schools of magic, each a category of
//! spells (`content::School`, the spells' `school`), with how many spells are of it; and the
//! caster styles' ranks (`data/weapon_skills.toml`'s styles whose `grip` is a caster's).

use super::text::{Term, anchored};
use super::weapon_skills::{grip_text, rank_table};
use std::fmt::Write as _;
use warlock_core::content::{Content, School};

/// Each school of magic, in the game's order, with how many of the shipped spells are of it.
pub fn schools(content: &Content) -> String {
    let mut out = String::from("| School | Spells |\n|---|:---:|\n");
    for school in School::ALL {
        let spells = (content.spells().iter())
            .filter(|spell| spell.school == school)
            .count();
        let name = super::text::capital(school.id());
        let _ = writeln!(
            out,
            "| {} | {spells} |",
            anchored(Term::School, school.id(), &name)
        );
    }
    out
}

/// The caster styles (how a spell is cast; #1251), in file order: each by name with the spells
/// cast in it and the curve it trains along, and what each
/// rank gives, a table per style.
pub fn caster_style_benefits(content: &Content) -> String {
    let table = content.weapon_skills();
    let mut out = format!(
        "A Sniper's target is far away at **{}** cells or more (the larger of the two \
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
