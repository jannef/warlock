//! The weapon skills tables (spec §8 "Weapon skills"), from `data/weapon_skills.toml`: the
//! skills and what trains them, how they default to each other, the marks each rank asks for,
//! and what each rank gives; and the starting ranks of races and classes, as data writes them.

use std::fmt::Write as _;
use warlock_core::content::{Benefit, Content, MAX_RANK, Role, WeaponSkillId};
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
    }
}

/// The objects that draw on skill `id`: the weapons that name it, or for the shield skills the
/// shields.
fn weapons_of(content: &Content, id: WeaponSkillId, role: Role) -> String {
    let names: Vec<String> = (content.object_defs().iter())
        .filter(|o| match role {
            Role::ShieldBlock => o.shield > 0,
            Role::ShieldBash => o.bash.is_some(),
            _ => o.skill == Some(id),
        })
        .map(|o| super::capital(&o.name))
        .collect();
    let mut text = names.join(", ");
    let more = match role {
        Role::Thrown => Some("anything thrown"),
        Role::Unarmed => Some("fists, and creatures' natural attacks"),
        _ => None,
    };
    if let Some(more) = more {
        text = if text.is_empty() {
            super::capital(more)
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
    for (id, s) in table.ids() {
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
    let mut out = String::from("| Skill | Defaults to |\n|---|---|\n");
    for (id, s) in table.ids() {
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
    let n = table.skills().len();
    out.push_str("\n| |");
    for i in 1..=n {
        let _ = write!(out, " {i} |");
    }
    out.push_str("\n|---|");
    out.push_str(&":---:|".repeat(n));
    out.push('\n');
    for (a, s) in table.ids() {
        let _ = write!(out, "| {} {} |", a.0 + 1, s.name);
        for (b, _) in table.ids() {
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
        let name = super::capital(name);
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

const COLUMNS: [Column; 7] = [
    ("To hit", |b| b.to_hit, plus),
    ("Damage", |b| b.damage, plus),
    ("Critical range", |b| i32::from(b.threat), wider),
    ("Time", |b| i32::from(b.speed), faster),
    ("Parry", |b| b.parry, plus),
    ("Shield", |b| b.shield, plus),
    ("Reload time", |b| i32::from(b.reload), faster),
];

/// What each rank of each skill gives: a table per skill, ranks 1 to 10, with the benefits it
/// has.
pub fn benefits(content: &Content) -> String {
    let table = content.weapon_skills();
    let mut out = String::new();
    for (id, s) in table.ids() {
        let ranks: Vec<Benefit> = (1..=MAX_RANK).map(|r| table.benefit(id, r)).collect();
        let columns: Vec<&Column> = (COLUMNS.iter())
            .filter(|(_, get, _)| ranks.iter().any(|b| get(b) != 0))
            .collect();
        let _ = writeln!(out, "**{}**\n", s.name);
        out.push_str("| Rank |");
        for (name, ..) in &columns {
            let _ = write!(out, " {name} |");
        }
        out.push_str("\n|:---:|");
        out.push_str(&":---:|".repeat(columns.len()));
        out.push('\n');
        for (r, b) in ranks.iter().enumerate() {
            let _ = write!(out, "| {} |", r + 1);
            for (_, get, show) in &columns {
                let v = get(b);
                let cell = if v == 0 { "-".to_string() } else { show(v) };
                let _ = write!(out, " {cell} |");
            }
            out.push('\n');
        }
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
                super::adjustment(dice)
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

/// A class's starting ranks: its own, then its choice (`Axes 2 or Great blades 2`).
pub fn class_ranks(content: &Content, class: &warlock_core::content::ClassDef) -> String {
    let own = ranks_text(content, &class.weapon_skills);
    let choice: Vec<String> = (class.weapon_skill_choice.iter())
        .map(|set| ranks_text(content, set))
        .collect();
    match (own.as_str(), choice.is_empty()) {
        (_, true) => own,
        ("-", false) => choice.join(" or "),
        (_, false) => format!("{own}; {}", choice.join(" or ")),
    }
}
