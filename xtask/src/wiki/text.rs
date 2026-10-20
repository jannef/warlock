//! Text helpers shared by the tables: glyphs, yes/no, capitals, table cells, articles, signs,
//! damage.

use warlock_core::dice::Dice;

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
pub(super) fn damage_text(dice: Dice, bonus: i32) -> String {
    let text = dice.to_string();
    // The text is `NdS`, `NdS+B`, or `NdS-B` (or a fixed number, which has no `d`).
    let (base, own) = match text.find('d') {
        Some(d) => match text[d..].find(['+', '-']) {
            Some(at) => (&text[..d + at], text[d + at..].parse().unwrap_or(0)),
            None => (text.as_str(), 0),
        },
        None => ("", text.parse().unwrap_or(0)),
    };
    let total: i32 = own + bonus;
    match (base, total) {
        ("", n) => n.max(0).to_string(),
        (base, 0) => base.replace('-', "\u{2212}"),
        (base, n) => format!("{}{}", base.replace('-', "\u{2212}"), signed(n)),
    }
}
