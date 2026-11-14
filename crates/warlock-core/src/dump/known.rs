//! The `locations` and `traps` layers: what the player knows of the world.

use super::*;

/// The overworld locations the player knows, one a line by position: `(45, 23) The Hall`; `no
/// known locations` when there are none (and on any other level).
pub(super) fn location_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let rows: Vec<String> = (view.known_locations().into_iter())
        .map(|(p, name)| format!("{p} {name}"))
        .collect();
    if rows.is_empty() {
        vec!["no known locations".into()]
    } else {
        rows
    }
}

/// The traps the player knows on the level, one a line by position: `(12, 5) fireball trap`,
/// `(40, 2) alarm trap (remembered)` when out of sight; `no known traps` when there are none.
pub(super) fn trap_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let content = view.content();
    let rows: Vec<String> = (view.known_traps().into_iter())
        .map(|(p, trap)| {
            let seen = if view.sees_objects_at(p) {
                ""
            } else {
                " (remembered)"
            };
            format!("{p} {}{seen}", content.trap(trap).name)
        })
        .collect();
    if rows.is_empty() {
        vec!["no known traps".into()]
    } else {
        rows
    }
}
