//! Mining (spec §8 "Digging"; `data/mining.toml`, the branches' `ores`): which walls may hold
//! something and how hard it is to break out, what comes out by danger, and each dungeon's ores.

use super::combat::percent;
use super::text::capital;
use std::fmt::Write as _;
use warlock_core::content::{Content, MAX_LEVEL, Mined};

/// The dangers the finds table shows shares at.
const DANGERS: [u32; 8] = [1, 5, 10, 15, 30, 60, 80, 100];

/// What a row of the table is called.
fn find_name(content: &Content, what: Mined) -> String {
    match what {
        Mined::Ore(m) => format!("{} ore", capital(&content.material(m).name)),
        Mined::Piece(m) => {
            let (one, _) = warlock_core::piece::names(
                content,
                content.object(content.object_id("piece").expect("the piece object")),
                m,
            );
            capital(&one)
        }
        Mined::Gem => "Uncut gem".into(),
    }
}

/// The anchor id of a row of the table.
fn find_id(content: &Content, what: Mined) -> String {
    match what {
        Mined::Ore(m) | Mined::Piece(m) => format!("mined-{}", content.material(m).id),
        Mined::Gem => "mined-gem".into(),
    }
}

/// Every wall that breaks (no statue), softest first: its hardness, the chance it holds
/// something for a miner, and the check's difficulty.
pub(super) fn mining_walls(content: &Content) -> String {
    let Some(rules) = content.mining() else {
        return "Nothing is mined.\n".into();
    };
    let mut walls: Vec<_> = (content.terrain_defs().iter())
        .filter(|t| t.statue.is_none())
        .filter_map(|t| Some((t, t.breaks?.hardness)))
        .collect();
    walls.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.name.cmp(&b.0.name)));
    let mut out =
        String::from("| Wall | Hardness | Holds something | Difficulty |\n|---|---:|---:|---:|\n");
    for (t, hardness) in walls {
        let Some(band) = rules.band(hardness) else {
            let _ = writeln!(out, "| {} | {hardness} | never | — |", capital(&t.name));
            continue;
        };
        let _ = writeln!(
            out,
            "| {} | {hardness} | {}% | {} |",
            capital(&t.name),
            band.chance,
            band.difficulty
        );
    }
    out
}

/// Every find, in the file's order: its rarity, the first danger it is found at, and its share
/// of the draws at several dangers (where a dungeon allows every ore).
pub(super) fn mining_finds(content: &Content) -> String {
    let Some(rules) = content.mining() else {
        return "Nothing is mined.\n".into();
    };
    let mut out = String::from("| Find | Rarity | Deeper | From danger |");
    for d in DANGERS {
        let _ = write!(out, " Danger {d} |");
    }
    out.push_str("\n|---|---:|---:|---:|");
    out.push_str(&"---:|".repeat(DANGERS.len()));
    out.push('\n');
    let tables: Vec<_> = DANGERS.iter().map(|&d| rules.at(d, None)).collect();
    for (i, find) in rules.finds.iter().enumerate() {
        let _ = write!(
            out,
            "| <a id=\"{}\"></a>{} | {} | {:+} | {} |",
            find_id(content, find.what),
            find_name(content, find.what),
            find.rarity,
            find.deeper,
            find.danger
        );
        for table in &tables {
            let total: u32 = table.iter().map(|(_, w)| w).sum();
            let share = match table.iter().find(|(r, _)| *r == i) {
                Some(&(_, w)) => percent(w, total.max(1)),
                None => "—".into(),
            };
            let _ = write!(out, " {share} |");
        }
        out.push('\n');
    }
    out
}

/// Every dungeon by name: the danger of its floors and the ores its rock holds.
pub(super) fn mining_dungeons(content: &Content) -> String {
    let Some(rules) = content.mining() else {
        return "Nothing is mined.\n".into();
    };
    let mut dungeons: Vec<_> = content.dungeons().collect();
    dungeons.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.id.cmp(&b.id)));
    let mut out = String::from("| Dungeon | Danger | Ores |\n|---|---|---|\n");
    for def in dungeons {
        let id = |depth: u32| match def.portal {
            Some(_) => format!("{}@1:{depth}", def.id),
            None => format!("{}:{depth}", def.id),
        };
        let first = content.floor_danger(&id(1), 1);
        let danger = match def.endless {
            Some(_) => format!("{first} and rising, a floor at a time"),
            None => {
                let last = content.floor_danger(&id(def.depth), MAX_LEVEL);
                if last == first {
                    format!("{first}")
                } else {
                    format!("{first} to {last}")
                }
            }
        };
        let ores = match &def.ores {
            None => {
                let all: Vec<String> = rules
                    .ores()
                    .map(|m| content.material(m).name.clone())
                    .collect();
                format!("every ore ({})", all.join(", "))
            }
            Some(list) if list.is_empty() => "none".into(),
            Some(list) => (list.iter())
                .filter_map(|m| content.material_id(m))
                .map(|m| content.material(m).name.clone())
                .collect::<Vec<_>>()
                .join(", "),
        };
        let _ = writeln!(out, "| {} | {danger} | {ores} |", def.name);
    }
    out
}
