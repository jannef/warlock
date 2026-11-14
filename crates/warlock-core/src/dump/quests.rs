//! The `quests` layer: the quest log, active first.

use super::*;

/// The player's quest log, one quest a line: its name, whether it is active or done, its giver,
/// what it asks, and how far it has got.
pub(super) fn quest_rows(view: &PerceptionView<'_>) -> Vec<String> {
    (view.quests().iter())
        .map(|q| {
            let stage = match (q.active, q.failed) {
                (true, _) => "active",
                (false, true) => "failed",
                (false, false) => "done",
            };
            let status = if q.active {
                format!(" {}", q.status)
            } else {
                String::new()
            };
            format!(
                "{} ({stage}, from {}): {}{status}",
                q.name, q.giver, q.objective
            )
        })
        .collect()
}

/// The player's quest log: `[{"id", "name", "kind", "active", "completed", "giver",
/// "objective", "target", "status"}]`, active quests first (`target`: what a rolled quest drew,
/// else null).
pub(super) fn quests_json(view: &PerceptionView<'_>) -> Value {
    (view.quests().iter())
        .map(|q| {
            json!({
                "id": q.id,
                "name": q.name,
                "kind": q.kind.id(),
                "active": q.active,
                "failed": q.failed,
                "completed": q.completed,
                "giver": q.giver,
                "objective": q.objective,
                "target": q.target,
                "status": q.status,
            })
        })
        .collect()
}
