//! Test support: the shipped factions with every grudge removed, so a fixture's creatures stand
//! by. Built from `assets/data/factions/`, so a new faction edits no fixture.

/// The shipped factions at peace: no `hostile` (`hostile_to_all` and `hostile_to_others` too) and
/// no `provoked` lines, except that the outraged (shopkeepers who come after the player) stay
/// hostile to the player.
pub(super) fn peaceful_factions() -> String {
    let real = crate::content::ContentFiles::builtin().table_text("factions");
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

/// `files` at peace: the [`peaceful_factions`], and no campaign's encounter tables (a hostile
/// entry names types hostile to the player, which no type is now; `docs/dev/encounters.md`), and
/// no shared vaults (a tag pool, the Catacomb's `undead`, draws only types hostile to the player).
pub(in crate::world) fn make_peace(files: &mut crate::content::ContentFiles) {
    files.replace_table("factions", &peaceful_factions());
    files.remove_encounter_tables();
    files.remove_shared_vaults();
}
