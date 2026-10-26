//! Dungeons and danger levels (spec §7 "Dungeon branches", "Danger levels"): every branch's
//! floors with their danger, who lives there, and how they come back.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use warlock_core::content::danger::{self, OUT_OF_DEPTH_STEPS, TOO_WEAK};
use warlock_core::content::{Content, Count, DangerRule, Generator, Spawning};

/// A danger rule in words: `4`, `2 + 1 a floor down`, with a player-scaled part.
fn rule_text(rule: DangerRule, branch: bool) -> String {
    let mut text = rule.base.to_string();
    if branch && rule.per_depth > 0 {
        let _ = write!(text, ", +{} a floor down", rule.per_depth);
    }
    if let Some(p) = rule.player {
        let _ = write!(
            text,
            ", +{}% of each of your levels past {}, at most {}",
            p.percent, p.from, p.max
        );
    }
    text
}

fn count_text(count: Count) -> String {
    match count {
        Count::Fixed(n) => n.to_string(),
        Count::PerCells {
            per_1000,
            least,
            most,
        } => format!("{per_1000} per 1000 walkable cells, {least} to {most}"),
    }
}

/// The types that live there, by level then name: `giant rat (1)`, with `unique` marked.
fn population_text(content: &Content, s: &Spawning) -> String {
    let mut kinds: Vec<_> = (s.population.iter())
        .map(|&c| content.creature(c))
        .collect();
    kinds.sort_by(|a, b| (a.level, &a.name).cmp(&(b.level, &b.name)));
    let names: Vec<String> = (kinds.iter())
        .map(|d| {
            let once = if d.unique { ", once a run" } else { "" };
            format!("{} ({}{once})", d.name, d.level)
        })
        .collect();
    names.join(", ")
}

fn spawn_text(s: &Spawning) -> String {
    let cap = match s.cap {
        Some(cap) => format!("up to {cap} hostile"),
        None => "up to its population".into(),
    };
    format!(
        "every {} of game time, {}% each time, {cap}; {}% of draws 1 to {OUT_OF_DEPTH_STEPS} deeper",
        s.every, s.chance, s.out_of_depth
    )
}

/// Every dungeon branch and every hand-made level with a danger (spec §7 "Danger levels"): the
/// danger of its floors, who lives there, how many, and how they come back. Branches by id, then
/// levels by id.
pub(super) fn dungeons(content: &Content) -> String {
    let mut out = String::from(
        "| Place | Floors | Danger | Lives there | Population | Coming back |\n\
         |---|---|---|---|---|---|\n",
    );
    let mut branches: Vec<_> = content.dungeons().collect();
    branches.sort_by(|a, b| a.id.cmp(&b.id));
    for d in branches {
        let danger = match (d.plan(1).and_then(|p| p.danger), &d.spawning) {
            (Some(rule), Some(_)) => rule_text(rule, true),
            _ => "0".into(),
        };
        let (lives, count, back) = match &d.spawning {
            Some(s) => (
                population_text(content, s),
                count_text(s.count),
                spawn_text(s),
            ),
            None => (
                "only the patrols the generator places".into(),
                "—".into(),
                "nothing comes".into(),
            ),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {danger} | {lives} | {count} | {back} |",
            d.name, d.depth
        );
    }
    let mut levels: Vec<_> = (content.levels())
        .filter_map(|l| Some((l, l.spawning.as_ref()?)))
        .collect();
    levels.sort_by(|a, b| a.0.id.cmp(&b.0.id));
    for (l, (rule, s)) in levels {
        let _ = writeln!(
            out,
            "| {} | 1 | {} | {} | {} | {} |",
            l.name,
            rule_text(*rule, false),
            population_text(content, s),
            count_text(s.count),
            spawn_text(s)
        );
    }
    out
}

/// Every floor of every branch with a danger: its generator, size, danger, and the weight of
/// each type that lives there at that danger. Branches by id, floors by depth.
pub(super) fn dungeon_floors(content: &Content) -> String {
    let mut out = String::from(
        "| Floor | Generator | Size | Danger | Drawn, by weight |\n|---|---|---|---|---|\n",
    );
    let mut branches: Vec<_> = content.dungeons().collect();
    branches.sort_by(|a, b| a.id.cmp(&b.id));
    let mut any = false;
    for d in branches {
        let Some(s) = &d.spawning else { continue };
        for depth in 1..=d.depth {
            let Some(plan) = d.plan(depth) else { continue };
            let Some(rule) = plan.danger else { continue };
            any = true;
            let danger = rule.at(depth, 1);
            let generator = match plan.generator {
                Generator::Rooms => "rooms",
                Generator::Caves => "caves",
            };
            let mut kinds: Vec<_> = (s.population.iter())
                .map(|&c| content.creature(c))
                .map(|def| {
                    let factor = content.creature_rarity(def.rarity).factor;
                    (danger::weight(def.level, factor, danger), def)
                })
                .filter(|(w, _)| *w > 0)
                .collect();
            kinds.sort_by(|a, b| (b.0, &a.1.name).cmp(&(a.0, &b.1.name)));
            let drawn: Vec<String> = (kinds.iter())
                .map(|(w, def)| format!("{} {w}", def.name))
                .collect();
            let scaled = if rule.scales() { " (at level 1)" } else { "" };
            let _ = writeln!(
                out,
                "| {} | {generator} | {}×{} | {danger}{scaled} | {} |",
                d.level_name(depth),
                plan.width,
                plan.height,
                if drawn.is_empty() {
                    "nothing".to_string()
                } else {
                    drawn.join(", ")
                }
            );
        }
    }
    if !any {
        out.push_str("| (no branch has a danger yet) | | | | |\n");
    }
    out
}

/// The weight of a creature type in a draw, by how far its level is below the danger.
pub(super) fn danger_weights(_: &Content) -> String {
    let mut out = String::from("| Its level | Weight |\n|---|---|\n");
    let d = 20u8;
    let rows = [
        ("above the danger".to_string(), d + 1),
        ("the danger".into(), d),
        ("1 below".into(), d - 1),
        ("2 below".into(), d - 2),
        ("3 below".into(), d - 3),
        (format!("4 to {TOO_WEAK} below"), d - 4),
        (format!("{} or more below", TOO_WEAK + 1), d - TOO_WEAK - 1),
    ];
    for (name, level) in rows {
        let _ = writeln!(out, "| {name} | {} |", danger::level_weight(level, d));
    }
    out
}

/// The creature rarities and their factors, in file order (commonest first), with the types of
/// each that some population draws, by name.
pub(super) fn creature_rarities(content: &Content) -> String {
    let mut out = String::from("| Rarity | Weight | Drawn among |\n|---|---:|---|\n");
    let branches = content.dungeons().filter_map(|d| d.spawning.as_ref());
    let levels = content
        .levels()
        .filter_map(|l| Some(&l.spawning.as_ref()?.1));
    let drawn: BTreeSet<_> = (branches.chain(levels))
        .flat_map(|s| s.population.iter().copied())
        .collect();
    for (i, tier) in content.creature_rarities().iter().enumerate() {
        let mut names: Vec<&str> = (drawn.iter())
            .map(|&c| content.creature(c))
            .filter(|def| def.rarity == i)
            .map(|def| def.name.as_str())
            .collect();
        names.sort_unstable();
        let names = if names.is_empty() {
            "-".to_string()
        } else {
            names.join(", ")
        };
        let _ = writeln!(out, "| {} | {}% | {names} |", tier.name, tier.factor);
    }
    out
}
