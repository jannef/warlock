//! The `weapon_skills` and `attacks` layers: the player's ranks and ready attacks.

use super::*;

/// How far the marks are toward the next rank, in whole percent; `None` at the last rank.
pub(super) fn progress_percent(view: &PerceptionView<'_>, id: WeaponSkillId) -> Option<u32> {
    let table = view.content().weapon_skills();
    let (marks, need) = view.own_weapon_skills().progress(table, id)?;
    Some(marks * 100 / need.max(1))
}

/// The player's weapon skills and fighting styles, in file order, with their ranks and how far
/// toward the next (`Long blades 2 (40%)`, `Daggers and knives 10` at the last rank), and the
/// styles they fight in now marked (`Duelist 1 (0%), in use`).
pub(super) fn weapon_skill_rows(view: &PerceptionView<'_>) -> Vec<String> {
    ranked_rows(view, |_| true)
}

/// The player's skills of the weapon skill table that `pick` takes and that show, in file
/// order, with their ranks and progress, the styles in use marked.
pub(super) fn ranked_rows(
    view: &PerceptionView<'_>,
    pick: impl Fn(WeaponSkillId) -> bool,
) -> Vec<String> {
    let skills = view.own_weapon_skills();
    let in_use = view.own_styles_in_use();
    let table = view.content().weapon_skills();
    (table.ids())
        .filter(|&(id, _)| pick(id) && skills.shows(table, id))
        .map(|(id, def)| {
            let rank = skills.rank(id);
            let row = match progress_percent(view, id) {
                Some(percent) => format!("{} {rank} ({percent}%)", def.name),
                None => format!("{} {rank}", def.name),
            };
            if in_use.contains(&id) {
                format!("{row}, in use")
            } else {
                row
            }
        })
        .collect()
}

/// The player's attacks, one a line (`Main hand long sword: +4 to hit, 1d8+2 slashing`; see
/// [`Layer::Attacks`]).
pub(super) fn attack_rows(view: &PerceptionView<'_>) -> Vec<String> {
    (view.own_attacks().iter())
        .map(|a| {
            let line = a.summary(view.content());
            format!("{} {}: {line}", a.source.label(), a.name)
        })
        .collect()
}

/// The player's attacks for JSON: `source`, `kind`, `name`, `to_hit`, `damage` (the text the
/// screen shows), and `range` (null for a blow).
pub(super) fn attacks_json(view: &PerceptionView<'_>) -> Value {
    (view.own_attacks().iter())
        .map(|a| {
            json!({
                "source": a.source.id(),
                "kind": a.kind,
                "name": a.name,
                "to_hit": a.to_hit,
                "damage": a.damage_text(view.content()),
                "range": a.range,
            })
        })
        .collect()
}
