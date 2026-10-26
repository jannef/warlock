//! Test support: the shipped factions with every grudge removed, so a fixture's creatures stand
//! by. Built from `assets/data/factions.toml`, so a new faction edits no fixture.

/// The shipped factions at peace: no `hostile` and no `provoked` lines, except that the outraged
/// (shopkeepers who come after the player) stay hostile to the player.
pub(super) fn peaceful_factions() -> String {
    let real = include_str!("../../../../assets/data/factions.toml");
    let mut id = "";
    let mut out = String::new();
    for line in real.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("id = \"") {
            id = rest.trim_end_matches('"');
        }
        let grudge = trimmed.starts_with("hostile") || trimmed.starts_with("provoked");
        if grudge && id != "outraged" {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}
