# Saves, storage, and file locations

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/07f-scenarios-and-saves.md. Keep it current with the code, as CLAUDE.md says.

Storage backends (`crates/warlock/src/storage.rs`): `Backend::Dir` (native files, atomic writes) or
`Backend::Browser` (wasm `localStorage`, keys `warlock/<name>`, e.g. `warlock/saves/brakka-1.sav`,
`warlock/saves/brakka-1.toml`, `warlock/saves/brakka-1.sav.bad`, `warlock/snapshots/brakka-1.sav`,
`warlock/settings.toml`; bytes base64). `Backend::list(dir)` names the files in a folder (the
browser's from the keys under `warlock/<dir>/`, `storage::files_in`, tested natively).
`SaveStore` and `SettingsStore` hold an `Option<Backend>`; new persistent files go through it, never
`std::fs` directly. On the web the page-hide save cannot ask the world for a save (a hidden tab runs
no frames), so `save.rs::web_hooks` keeps a clone of the state and the character's summary after
every turn (`storage::web::Pending`: encoding is milliseconds on the vast level) and the
`visibilitychange`/`pagehide` listeners encode and write them as that character's files.
Unavailable storage means no saving plus a log warning; failed writes (quota) are messages, never
panics. Only the `web-sys` features in `crates/warlock/Cargo.toml` are enabled (size).

**One save per character** (#218; `save.rs`, `save/characters.rs`): every character has a save id,
a slug of its name and the lowest free number (`characters::fresh_id`: `brakka-1`, then
`brakka-2`; an id is taken while any file of it is left: its save, a kept `.sav.bad`, its summary,
its snapshot), and the files `saves/<id>.sav` (the world, `warlock_core::save`) and, beside it,
`saves/<id>.toml` (`save::Summary`: name, race and class names, level, place, and when it was last
saved, wall-clock seconds since 1970, shown in UTC by `save::played_text`), so the list of saved
characters (`SaveStore::characters`, the one played last first) reads no save. A save without a
readable summary is listed by its id. The run in progress carries its id (`Game::save_id`, given by
`SaveStore::new_id` when a run starts, `Game::new_run`/`Game::start`, or the id resumed), and every
way of saving writes that character's files only: autosave (also as soon as a run starts or is
resumed, so a new character is listed at once), quitting, closing the window, the web page-hide
save, and the dev snapshots (F5 / F9: `snapshots/<id>.sav`). Continue (`MenuChoice::Continue`)
resumes `SaveStore::last_played`, and so does a start that skips the menu unless `--new`,
`--capture`, `--replay`, or `--record` (replays and recordings always start a fresh run); the
Characters screen resumes one by id (`MenuChoice::Resume`) or deletes it after the dialog
(`MenuChoice::DeleteCharacter` → `SaveStore::delete_character`). An unreadable save is renamed
`<id>.sav.bad` when resuming it fails (never deleted; its summary goes, so the list no longer
shows it) and the main menu opens afresh with the reason. Captures and replays never write: with
`--save-dir` their store is read-only (`SaveStore::read_only`), so a capture can show the list of
the characters in a folder (`tests/golden/saved-characters`, whose saves are placeholders: only the
summaries are read). A pre-#218 `run.sav` is left alone: not read, listed, or deleted (the next
release's format bump would reject it anyway, and its character has no name).

Format and `Content::fingerprint` live in `warlock-core/src/save.rs`.
`FORMAT_VERSION` is bumped by the integrator once per release, never in a feature PR
(`docs/dev/merging.md`): an incompatible `WorldState` change, or adding or reordering terrain, creatures,
lights, levels, objects, equipment slots, skills, weapon skills (fighting styles too), statuses, diseases, traps, or
dungeon branches, invalidates existing saves (affixes are saved by id: adding them does not,
removing one a save uses does); say so in the PR and leave the number alone.

**Permadeath**: when the player dies, `screens::on_death` deletes that character's files, and no
other character's (`SaveStore::delete_character(&game.save_id)`: its save, summary, and snapshot)
and opens the death screen; `SaveStore::save_run` never writes a dead run (it deletes instead), so
autosave, quitting, and closing the window cannot bring it back, and the web build parks no state
for the page being hidden.

**File locations** (`crates/warlock/src/paths.rs`, the one place that decides; never call
`ProjectDirs` elsewhere): `paths::init(&args, check_only)` runs first in `main`, before logging,
and `paths::locations()` is what stores, logging, and the Files screen ask (it carries each
folder's `Reason`, shown by `--paths`). Precedence per folder: `--config-dir` / `--save-dir`, then
`--portable [DIR]`, then a `warlock-data` folder next to the game (`paths::game_folder`: the
executable's folder; on macOS next to the `.app`, never inside it), then the platform dirs
(`directories`, app name "warlock"; macOS `~/Library/Application Support/warlock`). Portable
layout `warlock-data/{config,data,logs}`; `Locations::prepare` creates it and warns (never
fails) when it is unwritable. `resolve` is pure over `Inputs` and tested without a filesystem; new
files go in the folder `Locations` gives (`keymaps_dir()` for key map files; `saves_dir()` and
`snapshots_dir()`, from `paths::SAVES_DIR` and `SNAPSHOTS_DIR`, which the save store names its files
under). `warlock --paths`
prints the report and exits; `warlock --move-data native|portable [DIR] [--dry-run]`
(`move_data.rs`: copy, compare, rename, remove; `*.bak-<time>` for conflicts; refuses while a game
holds the shared lock on `logs/.in-use`) moves files between the two layouts. Settings > Files
(`ui/files_screen.rs`, `ScreenName::Files`, `--screen files`) shows the report with Open folder.
