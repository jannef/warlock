//! Material drops (spec §7 "Material drops"; #1341): what each creature's body leaves of
//! crafting stock, its rows resolved as the game rolls them (its tags' and its own, corpse and
//! size applied), and the rows of each tag's file as written.

use super::order;
use super::text::{Term, capital, link};
use std::fmt::Write as _;
use warlock_core::content::{Content, Drop, DropFrom, DropGoods, TagDrops};

/// A per mille chance as a percentage, without trailing zeros.
fn percent(per_mille: u16) -> String {
    let text = format!("{:.1}", f64::from(per_mille) / 10.0);
    let text = text.strip_suffix(".0").unwrap_or(&text);
    format!("{text}%")
}

/// What a row gives, by the name the game shows it by: "leather hide", "essence of fire".
fn goods(content: &Content, row: &Drop) -> String {
    match row.goods {
        DropGoods::Material(m) => (warlock_core::piece::make(content, m, 1))
            .map_or_else(|| content.material(m).name.clone(), |s| s.names(content).0),
        DropGoods::Object(o) => content.object(o).name.clone(),
    }
}

/// One row in words: "bone shard (1d2, 50%)", how many and the chance.
fn row_text(content: &Content, row: &Drop) -> String {
    let count = row.count.to_string();
    // A fixed count reads as a plain number: "1", not "+1".
    let count = count.trim_start_matches('+');
    let (what, chance) = (goods(content, row), percent(row.chance));
    format!("{what} ({count}, {chance})")
}

/// Every creature type that leaves anything, by name: its resolved rows, each with its chance
/// to drop as rolled (a small one's halved) and its count, and where each comes from.
pub(super) fn creature_drops(content: &Content) -> String {
    let mut out = String::from("| Creature | Drops | From |\n|---|---|---|\n");
    for (_, c) in order::creatures(content) {
        if c.drops.is_empty() {
            continue;
        }
        let rows: Vec<String> = c.drops.iter().map(|r| row_text(content, r)).collect();
        let mut from: Vec<String> = Vec::new();
        for r in &c.drops {
            let said = match &r.from {
                DropFrom::Tag(t) => format!("`{t}`"),
                DropFrom::Own => "its own".to_string(),
            };
            if !from.contains(&said) {
                from.push(said);
            }
        }
        let _ = writeln!(
            out,
            "| <a id=\"drops-{}\"></a>{} | {} | {} |",
            c.id.replace('_', "-"),
            link(Term::Creature, &c.id, &capital(&c.name)),
            rows.join("; "),
            from.join(", "),
        );
    }
    out
}

/// Each tag's rows as written (before a small body halves them), by tag, and whether they reach
/// a creature that leaves no corpse.
pub(super) fn tag_drops(content: &Content) -> String {
    let mut out = String::from("| Tag | Without a body too | Drops |\n|---|:---:|---|\n");
    let mut tags: Vec<&TagDrops> = content.tag_drops().iter().collect();
    tags.sort_by(|a, b| a.tag.cmp(&b.tag));
    for t in tags {
        let rows: Vec<String> = t.rows.iter().map(|r| row_text(content, r)).collect();
        let _ = writeln!(
            out,
            "| <a id=\"drops-tag-{}\"></a>`{}` | {} | {} |",
            t.tag,
            t.tag,
            if t.bodiless { "yes" } else { "no" },
            rows.join("; "),
        );
    }
    out
}
