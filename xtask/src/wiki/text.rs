//! Text helpers shared by the tables: glyphs, yes/no, capitals, table cells, articles, signs,
//! damage.

use warlock_core::content::{Content, DamageTypeId};
use warlock_core::dice::Roll;
use warlock_core::stats::Stat;

pub(super) fn glyph(c: char) -> String {
    if c == ' ' {
        "(blank)".into()
    } else {
        format!("`{c}`")
    }
}

pub(super) fn yes(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

/// `12920` as `12,920`.
pub(super) fn thousands(n: u32) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// A name for a player-facing table, first letter capitalized.
pub(super) fn capital(name: &str) -> String {
    let mut chars = name.chars();
    chars
        .next()
        .map_or_else(String::new, |c| c.to_uppercase().chain(chars).collect())
}

/// Free text for a table cell: command-line flags (`--seed`) as code, so they keep both hyphens,
/// and `|` escaped.
pub(super) fn table_text(text: &str) -> String {
    text.split(' ')
        .map(|word| match word.strip_prefix("--") {
            Some(flag) if !flag.is_empty() => format!("`--{flag}`"),
            _ => word.replace('|', "\\|"),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// `a` or `an` for a noun.
pub(super) fn article(noun: &str) -> &'static str {
    match noun.chars().next() {
        Some(c) if "aeiouAEIOU".contains(c) => "an",
        _ => "a",
    }
}

/// A signed number with a real minus sign: `+1`, `−1`.
pub(super) fn signed(n: i32) -> String {
    if n < 0 {
        format!("\u{2212}{}", -n)
    } else {
        format!("+{n}")
    }
}

/// Damage dice with `bonus` added to every roll, its own fixed bonus included, with a real minus
/// sign: `1d4`, `1d3−1`, `1d8+3` (a `1d8+2` bow and +1).
pub(super) fn damage_text(dice: Roll, bonus: i32) -> String {
    let total = dice.bonus() + bonus;
    if dice.is_fixed() {
        return total.max(0).to_string();
    }
    let base = dice
        .plus(-dice.bonus())
        .to_string()
        .replace('-', "\u{2212}");
    match total {
        0 => base,
        n => format!("{base}{}", signed(n)),
    }
}

/// A kind of term a generated table defines, one row each: the page the table is on and the
/// prefix of its rows' anchors (`status-staggered`, `skill-open-lock`). Prose links a row as
/// `[Staggered](status-effects.md#status-staggered)`; the term map (`wiki/TERMS.md`) lists them.
#[derive(Clone, Copy)]
pub(super) enum Term {
    Ability,
    Creature,
    Damage,
    Disease,
    ItemClass,
    Potion,
    Race,
    Resource,
    Room,
    School,
    SchoolAbility,
    Scroll,
    Skill,
    Spell,
    Stance,
    Stat,
    Status,
    Style,
    StyleAbility,
    Talent,
    Terrain,
    Trap,
    WeaponAbility,
    WeaponSkill,
}

impl Term {
    /// The page whose table defines it, and its anchors' prefix.
    fn place(self) -> (&'static str, &'static str) {
        match self {
            Term::Ability => ("abilities.md", "ability"),
            Term::Creature => ("creatures.md", "creature"),
            Term::Damage => ("damage-types.md", "damage"),
            Term::Disease => ("diseases.md", "disease"),
            Term::ItemClass => ("objects.md", "class"),
            Term::Potion => ("potions.md", "potion"),
            Term::Race => ("races.md", "race"),
            Term::Resource => ("resources.md", "resource"),
            Term::Room => ("special-rooms.md", "room"),
            Term::School => ("schools.md", "school"),
            Term::SchoolAbility => ("schools.md", "school-ability"),
            Term::Scroll => ("scrolls.md", "scroll"),
            Term::Skill => ("skills.md", "skill"),
            Term::Spell => ("magic.md", "spell"),
            Term::Stance => ("combat.md", "stance"),
            Term::Stat => ("stats.md", "stat"),
            Term::Status => ("status-effects.md", "status"),
            Term::Style => ("fighting-styles.md", "style"),
            Term::StyleAbility => ("fighting-styles.md", "style-ability"),
            Term::Talent => ("talents.md", "talent"),
            Term::Terrain => ("terrain.md", "terrain"),
            Term::Trap => ("traps.md", "trap"),
            Term::WeaponAbility => ("weapon-skills.md", "weapon-ability"),
            Term::WeaponSkill => ("weapon-skills.md", "weapon-skill"),
        }
    }

    fn anchor_id(self, id: &str) -> String {
        let id = id.to_lowercase().replace(['_', ' '], "-");
        let prefix = self.place().1;
        // `potion_of_healing` is `potion-of-healing`, not `potion-potion-of-healing`.
        if id
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.starts_with('-'))
        {
            id
        } else {
            format!("{prefix}-{id}")
        }
    }
}

/// A row's name with the anchor prose links to: `<a id="status-staggered"></a>Staggered`.
pub(super) fn anchored(term: Term, id: &str, name: &str) -> String {
    format!("<a id=\"{}\"></a>{name}", term.anchor_id(id))
}

/// A link to the row that defines a term: `[Staggered](status-effects.md#status-staggered)`.
pub(super) fn link(term: Term, id: &str, name: &str) -> String {
    format!("[{name}]({}#{})", term.place().0, term.anchor_id(id))
}

/// A damage type's name linked to its row: `[fire](damage-types.md#damage-fire)`.
pub(super) fn damage_link(content: &Content, kind: DamageTypeId) -> String {
    let def = content.damage_type(kind);
    link(Term::Damage, &def.id, &def.name)
}

/// A stat's name linked to its row: `[Toughness](stats.md#stat-toughness)`.
pub(super) fn stat_link(stat: Stat) -> String {
    link(Term::Stat, stat.name(), stat.name())
}
