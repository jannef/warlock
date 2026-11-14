//! Allies (spec §8 "Allies"): the rules of `data/allies.toml` in words and numbers, and the
//! summon spells: what each calls, how many, for how long, how many at once, its mana, and how
//! hard its scroll is to learn.

use super::spells::grows_with;
use super::text::{Term, capital, link};
use std::fmt::Write as _;
use warlock_core::content::{Content, SpellData, SpellPurpose};

/// The ally rules, one a row.
pub(super) fn rules(content: &Content) -> String {
    let Some(r) = content.allies() else {
        return String::new();
    };
    let rows = [
        (
            "Following",
            format!("a following ally keeps within {} cells of you", r.heel),
        ),
        (
            "Leash",
            format!(
                "farther than {} cells from you, a following ally breaks off its fight and comes back",
                r.leash
            ),
        ),
        (
            "Coming along",
            format!(
                "allies within {} cells of you come along when you take stairs or leave an area",
                r.comes_along
            ),
        ),
        (
            "Hold here",
            format!(
                "a holding ally fights only what comes within {} of its place",
                cells(r.stay_reach)
            ),
        ),
        (
            "Set on",
            format!(
                "a set-on ally fights what is within {} of the place you sent it to",
                cells(r.hunt_reach)
            ),
        ),
        (
            "Orders",
            format!(
                "take no time; your voice is heard at loudness {}",
                r.order_noise
            ),
        ),
        (
            "At most",
            format!(
                "{} allies of yours on one level, whatever brought them",
                r.most
            ),
        ),
        (
            "Experience",
            format!(
                "an ally's kill pays you {} % of what your own blow would, unless your class has \
                 a share of its own (below)",
                r.ally_percent
            ),
        ),
    ];
    let mut out = String::from("| Rule | What it is |\n|---|---|\n");
    for (name, text) in rows {
        let _ = writeln!(out, "| {name} | {text} |");
    }
    // The classes with a share of their own (`ally_share`: the pet classes), by name.
    let mut shares: Vec<(&str, &str, u8)> = (content.classes().iter())
        .filter_map(|c| Some((c.name.as_str(), c.id.as_str(), c.ally_share?)))
        .collect();
    shares.sort();
    for (name, id, share) in shares {
        let _ = writeln!(
            out,
            "| Experience: [{name}](classes.md#{id}) | an ally's kill pays a {} {share} % of what \
             their own blow would |",
            name.to_lowercase()
        );
    }
    out
}

/// "1 cell", "3 cells".
fn cells(n: i32) -> String {
    match n {
        1 => "1 cell".into(),
        n => format!("{n} cells"),
    }
}

/// Every summon spell, by spell level, then name.
pub(super) fn summon_spells(content: &Content) -> String {
    let Some(rules) = content.spell_rules() else {
        return String::new();
    };
    let mut spells: Vec<&SpellData> = (content.spells().iter())
        .filter(|s| s.kind == SpellPurpose::Summon)
        .collect();
    spells.sort_by(|a, b| a.level.cmp(&b.level).then(a.name.cmp(&b.name)));
    let mut out = String::from(
        "| Spell | Level | School | Calls | How many | Minutes | At once | Grows with | Mana | Learning |\n\
         |---|:---:|---|---|:---:|:---:|:---:|---|:---:|:---:|\n",
    );
    for spell in spells {
        let Some(call) = &spell.effect.summon else {
            continue;
        };
        let pool: Vec<String> = (call.pool.iter())
            .map(|&c| {
                let def = content.creature(c);
                format!(
                    "{} ({})",
                    link(Term::Creature, &def.id, &def.name),
                    def.level
                )
            })
            .collect();
        let count = if call.count.is_fixed() {
            call.count.min().to_string()
        } else {
            call.count.to_string()
        };
        let learning = match spell.study_difficulty {
            0 => "—".to_string(),
            n => format!("{n:+}"),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {count} | {} | {} | {} | {} | {learning} |",
            link(Term::Spell, &spell.id, &spell.name),
            spell.level,
            link(Term::School, spell.school.id(), &capital(spell.school.id())),
            pool.join(" or "),
            call.minutes,
            call.most,
            grows_with(spell),
            rules.mana(spell, 0),
        );
    }
    out
}

/// The spells that raise a corpse (`[spell.effect] rise`): what the risen stand at, how long and
/// how many serve, and how often one rises berserk.
pub(super) fn rise_spells(content: &Content) -> String {
    let Some(rules) = content.spell_rules() else {
        return String::new();
    };
    let mut spells: Vec<&SpellData> = (content.spells().iter())
        .filter(|s| s.effect.rise.is_some())
        .collect();
    spells.sort_by(|a, b| a.level.cmp(&b.level).then(a.name.cmp(&b.name)));
    let mut out = String::from(
        "| Spell | Level | School | Hit points | Minutes | At once | Berserk | Grows with | Mana | Learning |\n\
         |---|:---:|---|:---:|:---:|:---:|:---:|---|:---:|:---:|\n",
    );
    for spell in spells {
        let Some(rise) = &spell.effect.rise else {
            continue;
        };
        let learning = match spell.study_difficulty {
            0 => "—".to_string(),
            n => format!("{n:+}"),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} % | {} | {} | {} % | {} | {} | {learning} |",
            link(Term::Spell, &spell.id, &spell.name),
            spell.level,
            link(Term::School, spell.school.id(), &capital(spell.school.id())),
            rise.hit_points,
            rise.minutes,
            rise.most,
            rise.berserk,
            grows_with(spell),
            rules.mana(spell, 0),
        );
    }
    out
}

/// The spells that charm (`[spell.effect] charm`): how long the charm holds, what it never takes,
/// and what grows it.
pub(super) fn charm_spells(content: &Content) -> String {
    let Some(rules) = content.spell_rules() else {
        return String::new();
    };
    let mut spells: Vec<&SpellData> = (content.spells().iter())
        .filter(|s| s.effect.charm.is_some())
        .collect();
    spells.sort_by(|a, b| a.level.cmp(&b.level).then(a.name.cmp(&b.name)));
    let mut out = String::from(
        "| Spell | Level | School | Range | Minutes | Never charms | Grows with | Mana |\n\
         |---|:---:|---|:---:|:---:|---|---|:---:|\n",
    );
    for spell in spells {
        let Some(charm) = &spell.effect.charm else {
            continue;
        };
        let mut never = vec!["uniques".to_string(), "animated things".to_string()];
        never.extend(charm.mindless.iter().map(|t| format!("`{t}`")));
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            link(Term::Spell, &spell.id, &spell.name),
            spell.level,
            link(Term::School, spell.school.id(), &capital(spell.school.id())),
            spell
                .base_range()
                .map_or("—".to_string(), |r| r.to_string()),
            charm.minutes,
            never.join(", "),
            grows_with(spell),
            rules.mana(spell, 0),
        );
    }
    out
}
