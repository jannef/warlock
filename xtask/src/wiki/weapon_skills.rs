//! The weapon skills tables (spec §8 "Weapon skills"), from `data/weapon_skills.toml`: the
//! skills and what trains them, how they default to each other, the marks each rank asks for,
//! and what each rank gives; the fighting styles (spec §8 "Fighting styles"): their
//! configurations, defaults, and what each rank gives (two-weapon penalties included); and the
//! starting ranks of races and classes, as data writes them.

use super::order;
use super::text::capital;
use std::fmt::Write as _;
use warlock_core::content::{
    Benefit, ClassDef, Content, Grip, MAX_RANK, Role, TwoWeapons, WeaponSkillDef, WeaponSkillId,
    WeaponSkillTable,
};
use warlock_core::dice::Dice;

/// `n` with a real minus sign: `−2`.
fn minus(n: u8) -> String {
    format!("\u{2212}{n}")
}

/// What trains a skill of `role`, in words.
fn trained_by(role: Role) -> &'static str {
    match role {
        Role::Melee => "blows with its weapons",
        Role::Launcher => "shots with its weapons",
        Role::Thrown => "every throw",
        Role::Unarmed => "blows with bare hands",
        Role::ShieldBlock => "blows your shield blocks",
        Role::ShieldBash => "bashing with a shield",
        Role::Style => "every attack in its configuration",
    }
}

/// The weapon skills, in file order, without the fighting styles.
fn weapons(table: &WeaponSkillTable) -> impl Iterator<Item = (WeaponSkillId, &WeaponSkillDef)> {
    table.ids().filter(|(_, s)| !s.is_style())
}

/// The objects that draw on skill `id`: the weapons that name it, or for the shield skills the
/// shields.
fn weapons_of(content: &Content, id: WeaponSkillId, role: Role) -> String {
    let names: Vec<String> = (order::objects(content).into_iter())
        .map(|(_, o)| o)
        .filter(|o| match role {
            Role::ShieldBlock => o.shield > 0,
            Role::ShieldBash => o.bash.is_some(),
            _ => o.skill == Some(id),
        })
        .map(|o| capital(&o.name))
        .collect();
    let mut text = names.join(", ");
    let more = match role {
        Role::Thrown => Some("anything thrown"),
        Role::Unarmed => Some("fists, and creatures' natural attacks"),
        _ => None,
    };
    if let Some(more) = more {
        text = if text.is_empty() {
            capital(more)
        } else {
            format!("{text}; {more}")
        };
    }
    if text.is_empty() { "-".into() } else { text }
}

/// The skills in file order: what trains each, the weapons that draw on it, and the curve it
/// trains along.
pub fn skills(content: &Content) -> String {
    let table = content.weapon_skills();
    let mut out = String::from("| Skill | Trained by | Weapons | Marks |\n|---|---|---|---|\n");
    for (id, s) in weapons(table) {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} |",
            s.name,
            trained_by(s.role),
            weapons_of(content, id, s.role),
            s.curve
        );
    }
    out
}

/// The defaults: for each skill the skills it raises, with the penalty, then the whole table
/// as a matrix (rows and columns numbered alike).
pub fn defaults(content: &Content) -> String {
    let table = content.weapon_skills();
    let mut out = default_list(table, false);
    let n = weapons(table).count();
    out.push_str("\n| |");
    for i in 1..=n {
        let _ = write!(out, " {i} |");
    }
    out.push_str("\n|---|");
    out.push_str(&":---:|".repeat(n));
    out.push('\n');
    for (i, (a, s)) in weapons(table).enumerate() {
        let _ = write!(out, "| {} {} |", i + 1, s.name);
        for (b, _) in weapons(table) {
            let cell = match table.penalty(a, b) {
                _ if a == b => "·".to_string(),
                Some(p) => minus(p),
                None => String::new(),
            };
            let _ = write!(out, " {cell} |");
        }
        out.push('\n');
    }
    out
}

/// For each weapon skill (`styles`: each fighting style) the skills it raises, with the penalty.
fn default_list(table: &WeaponSkillTable, styles: bool) -> String {
    let mut out = format!(
        "| {} | Defaults to |\n|---|---|\n",
        if styles { "Style" } else { "Skill" }
    );
    for (id, s) in table.ids().filter(|(_, s)| s.is_style() == styles) {
        let to: Vec<String> = (table.defaults_of(id).into_iter())
            .map(|(other, p)| format!("{} {}", table.skill(other).name, minus(p)))
            .collect();
        let to = if to.is_empty() {
            "nothing".to_string()
        } else {
            to.join(", ")
        };
        let _ = writeln!(out, "| {} | {to} |", s.name);
    }
    out
}

/// How marks are earned: what an attempt and a hit earn, how the opponent's level scales it,
/// and the marks each rank asks for along each curve.
pub fn marks(content: &Content) -> String {
    let table = content.weapon_skills();
    let Some(rules) = table.marks() else {
        return String::new();
    };
    let mut out = format!(
        "Every attempt earns **{}** {}, and a hit **{}** more. Against an opponent above your \
         rank you earn **{}%** more for each level between you, below it {}% less for each, up \
         to **{}%** in all and down to nothing:\n\n",
        rules.attempt,
        if rules.attempt == 1 { "mark" } else { "marks" },
        rules.hit,
        rules.per_level,
        rules.per_level,
        rules.most
    );
    let differences: Vec<i32> = (-5..=5).collect();
    out.push_str("| The opponent's level against your rank |");
    for d in &differences {
        let cell = match d {
            0 => "the same".to_string(),
            d if *d > 0 => format!("{d} above"),
            d => format!("{} below", -d),
        };
        let _ = write!(out, " {cell} |");
    }
    out.push_str("\n|---|");
    out.push_str(&":---:|".repeat(differences.len()));
    out.push('\n');
    // The rank in the middle of the ladder, so the opponent's level can be anything around it.
    let rank = 5u8;
    let level = |d: i32| (i32::from(rank) + d) as u32;
    for (label, row) in [
        ("Marks", None),
        ("A miss earns", Some(false)),
        ("A hit earns", Some(true)),
    ] {
        let _ = write!(out, "| {label} |");
        for d in &differences {
            let cell = match row {
                None => format!("{}%", rules.percent(level(*d), rank)),
                Some(hit) => rules.earned(hit, level(*d), rank).to_string(),
            };
            let _ = write!(out, " {cell} |");
        }
        out.push('\n');
    }
    let curves = table.curves();
    out.push_str("\n| Rank |");
    for name in curves.keys() {
        let name = capital(name);
        let _ = write!(out, " {name} marks | {name}, in all |");
    }
    out.push_str("\n|:---:|");
    out.push_str(&":---:|:---:|".repeat(curves.len()));
    out.push('\n');
    for rank in 1..=MAX_RANK {
        let _ = write!(out, "| {rank} |");
        for need in curves.values() {
            let one = need[usize::from(rank - 1)];
            let all: u32 = need[..usize::from(rank)].iter().sum();
            let _ = write!(out, " {} | {} |", thousands(one), thousands(all));
        }
        out.push('\n');
    }
    out
}

/// `12920` as `12,920`.
fn thousands(n: u32) -> String {
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

/// The benefits a table shows, with their headings, and how each value reads.
type Column = (&'static str, fn(&Benefit) -> i32, fn(i32) -> String);

fn plus(v: i32) -> String {
    format!("+{v}")
}

fn wider(v: i32) -> String {
    format!("{v} wider")
}

fn faster(v: i32) -> String {
    format!("{v}% less")
}

fn rolls(v: i32) -> String {
    format!("+{v} {}", if v == 1 { "roll" } else { "rolls" })
}

fn percent(v: i32) -> String {
    format!("{v}%")
}

const COLUMNS: [Column; 9] = [
    ("To hit", |b| b.to_hit, plus),
    ("Damage", |b| b.damage, plus),
    ("Critical range", |b| i32::from(b.threat), wider),
    ("Critical damage", |b| i32::from(b.crit), rolls),
    ("Time", |b| i32::from(b.speed), faster),
    ("Parry", |b| b.parry, plus),
    ("Shield", |b| b.shield, plus),
    ("Reload time", |b| i32::from(b.reload), faster),
    ("Shield bash after a hit", |b| i32::from(b.bash), percent),
];

/// A two-weapon penalty as it reads: `−4`, or `none`.
fn penalty(p: i32) -> String {
    if p == 0 {
        "none".into()
    } else {
        format!("\u{2212}{}", -p)
    }
}

/// A table of ranks 1 to 10 of skill `id`: the [`COLUMNS`] it has, and for a style that eases
/// the two-weapon penalties the penalty of each hand at each rank.
fn rank_table(table: &WeaponSkillTable, id: WeaponSkillId) -> String {
    let ranks: Vec<Benefit> = (1..=MAX_RANK).map(|r| table.benefit(id, r)).collect();
    let mut columns: Vec<(&str, Vec<String>)> = (COLUMNS.iter())
        .filter(|(_, get, _)| ranks.iter().any(|b| get(b) != 0))
        .map(|(name, get, show)| {
            let cells = (ranks.iter())
                .map(|b| match get(b) {
                    0 => "-".to_string(),
                    v => show(v),
                })
                .collect();
            (*name, cells)
        })
        .collect();
    let two = table.two_weapons();
    let hands: [(&str, i32, fn(&Benefit) -> u8); 2] = [
        ("Main hand", two.main, |b| b.ease_main),
        ("Off hand", two.off, |b| b.ease_off),
    ];
    for (name, base, ease) in hands {
        if ranks.iter().any(|b| ease(b) != 0) {
            let cells = (ranks.iter())
                .map(|b| penalty(TwoWeapons::eased(base, ease(b))))
                .collect();
            columns.push((name, cells));
        }
    }
    let mut out = String::from("| Rank |");
    for (name, _) in &columns {
        let _ = write!(out, " {name} |");
    }
    out.push_str("\n|:---:|");
    out.push_str(&":---:|".repeat(columns.len()));
    out.push('\n');
    for r in 0..ranks.len() {
        let _ = write!(out, "| {} |", r + 1);
        for (_, cells) in &columns {
            let _ = write!(out, " {} |", cells[r]);
        }
        out.push('\n');
    }
    out
}

/// What each rank of each weapon skill gives: a table per skill, ranks 1 to 10, with the
/// benefits it has.
pub fn benefits(content: &Content) -> String {
    let table = content.weapon_skills();
    let mut out = String::new();
    for (id, s) in weapons(table) {
        let _ = writeln!(out, "**{}**\n", s.name);
        out.push_str(&rank_table(table, id));
        out.push('\n');
    }
    out
}

/// A grip in words: `a one-handed weapon and a shield`.
fn grip_text(grip: Grip) -> &'static str {
    match grip {
        Grip::OneWeapon => "a one-handed weapon, the other hand empty",
        Grip::TwoWeapons => "a weapon in each hand",
        Grip::TwoHanded => "a two-handed weapon",
        Grip::WeaponAndShield => "a one-handed weapon and a shield",
        Grip::Unarmed => "nothing in your hands",
        Grip::Launcher => "shooting a missile weapon",
        Grip::Thrown => "throwing",
    }
}

/// The fighting styles in file order: the configuration each is fought in, the weapon skills it
/// is fought with, the curve it trains along, and the styles it defaults to.
pub fn styles(content: &Content) -> String {
    let table = content.weapon_skills();
    let mut out = String::from("| Style | Fought with | Weapons | Marks |\n|---|---|---|---|\n");
    for (_, s, style) in table.styles() {
        let names = |ids: &[WeaponSkillId]| -> Vec<String> {
            ids.iter().map(|&w| table.skill(w).name.clone()).collect()
        };
        let others: Vec<String> = (table.styles())
            .filter(|(_, _, o)| o.grip == style.grip && !o.weapons.is_empty())
            .flat_map(|(_, _, o)| names(&o.weapons))
            .collect();
        let weapons = match (style.grip, style.weapons.is_empty(), others.is_empty()) {
            (Grip::Unarmed, ..) => "fists".to_string(),
            (_, false, _) => names(&style.weapons).join(", "),
            (_, true, true) => "any".to_string(),
            (_, true, false) => format!("any but {}", others.join(", ").to_lowercase()),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {weapons} | {} |",
            s.name,
            capital(grip_text(style.grip)),
            s.curve
        );
    }
    out
}

/// The defaults between fighting styles.
pub fn style_defaults(content: &Content) -> String {
    default_list(content.weapon_skills(), true)
}

/// What each rank of each fighting style gives, a table per style; Twin Blades' shows the
/// two-weapon penalty of each hand at each rank.
pub fn style_benefits(content: &Content) -> String {
    let table = content.weapon_skills();
    let two = table.two_weapons();
    let mut out = format!(
        "With a weapon in each hand and no rank in Twin Blades, the main hand strikes at **{}** \
         to hit and the off hand at **{}**.\n\n",
        penalty(two.main),
        penalty(two.off)
    );
    for (id, s, _) in table.styles() {
        let _ = writeln!(out, "**{}**\n", s.name);
        out.push_str(&rank_table(table, id));
        out.push('\n');
    }
    out
}

/// Starting ranks as data gives them: `Long blades 1, Bows 1d2`; `-` for none.
pub fn ranks_text(content: &Content, ranks: &[(WeaponSkillId, Dice)]) -> String {
    let table = content.weapon_skills();
    let parts: Vec<String> = (ranks.iter())
        .map(|&(id, dice)| {
            let rank = if dice.is_fixed() {
                dice.max().to_string()
            } else {
                super::character::adjustment(dice)
            };
            format!("{} {rank}", table.skill(id).name)
        })
        .collect();
    if parts.is_empty() {
        "-".into()
    } else {
        parts.join(", ")
    }
}

/// A class's starting ranks in weapon skills: its own, then its choice (`Axes 2 or Great blades
/// 2`).
pub fn class_ranks(content: &Content, class: &ClassDef) -> String {
    own_or_chosen(content, &class.weapon_skills, &class.weapon_skill_choice)
}

/// A class's starting ranks in fighting styles: its own, then its choice (`Twin Blades 1 or
/// Duelist 1`).
pub fn class_styles(content: &Content, class: &ClassDef) -> String {
    own_or_chosen(content, &class.styles, &class.style_choice)
}

/// Ranks `own`, then one of `choice`.
fn own_or_chosen(
    content: &Content,
    own: &[(WeaponSkillId, Dice)],
    choice: &[Vec<(WeaponSkillId, Dice)>],
) -> String {
    let own = ranks_text(content, own);
    let choice: Vec<String> = (choice.iter())
        .map(|set| ranks_text(content, set))
        .collect();
    match (own.as_str(), choice.is_empty()) {
        (_, true) => own,
        ("-", false) => choice.join(" or "),
        (_, false) => format!("{own}; {}", choice.join(" or ")),
    }
}
