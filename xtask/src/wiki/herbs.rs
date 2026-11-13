//! Herbs (spec §8 "Skills": Herbalism; `data/herbs.toml`, `data/forage.toml`): what gathering
//! from the glowing fungus takes and finds, and each herb's share by depth and where it grows on
//! the surface.

use super::combat::percent;
use super::text::{Term, anchored, capital};
use std::fmt::Write as _;
use warlock_core::content::Content;

/// The depths the herb table shows shares at.
const DEPTHS: [u8; 4] = [1, 3, 6, 10];

/// Gathering's numbers: the time, and how many herbs a gather finds.
pub(super) fn gathering(content: &Content) -> String {
    let Some(rules) = content.herbs() else {
        return "Nobody gathers herbs.\n".into();
    };
    format!(
        "A gather takes {} minutes. You find `{}` herbs, plus one for every {} of {}.\n",
        rules.seconds / 60,
        rules.count,
        rules.value_per_count,
        content.skill(rules.skill).name,
    )
}

/// Every herb, as the gather table lists them: its worth, its share of a gather's draws at
/// depths 1, 3, 6, and 10, and the lands a forage may find it on.
pub(super) fn herbs(content: &Content) -> String {
    let Some(rules) = content.herbs() else {
        return "Nobody gathers herbs.\n".into();
    };
    let mut out = String::from("| Herb | Worth | From depth |");
    for depth in DEPTHS {
        let _ = write!(out, " Depth {depth} |");
    }
    out.push_str(" On a forage (Herbalism) |\n|---|---:|---:|");
    out.push_str(&"---:|".repeat(DEPTHS.len()));
    out.push_str("---|\n");
    let tables: Vec<_> = DEPTHS.iter().map(|&d| rules.at(d)).collect();
    for herb in &rules.herbs {
        let def = content.object(herb.object);
        let _ = write!(
            out,
            "| {} | {} gold | {} |",
            anchored(Term::Herb, &def.id, &capital(&def.name)),
            def.value,
            herb.level
        );
        for table in &tables {
            let total: u32 = table.iter().map(|(_, w)| w).sum();
            let mine = (table.iter())
                .find(|(o, _)| *o == herb.object)
                .map_or(0, |e| e.1);
            let share = match mine {
                0 => "—".to_string(),
                w => percent(w, total.max(1)),
            };
            let _ = write!(out, " {share} |");
        }
        let mut lands: Vec<&str> = (content.forage().into_iter())
            .flat_map(|f| &f.grounds)
            .filter(|g| g.herbs.iter().any(|(o, _)| *o == herb.object))
            .map(|g| g.landscape.id())
            .collect();
        lands.sort_unstable();
        let lands = if lands.is_empty() {
            "never".to_string()
        } else {
            lands.join(", ").replace('_', " ")
        };
        let _ = writeln!(out, " {lands} |");
    }
    out
}
