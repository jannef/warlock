# Saves, storage, and file locations

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/07f-scenarios-and-saves.md. Keep it current with the code, as CLAUDE.md says.

Storage backends (`crates/warlock/src/storage.rs`): `Backend::Dir` (native files, atomic writes) or
`Backend::Browser` (wasm `localStorage`, keys `warlock/<name>`, e.g. `warlock/run.sav`,
`warlock/run.sav.bad`, `warlock/snapshots/quick.sav`, `warlock/settings.toml`; bytes base64).
`SaveStore` and `SettingsStore` hold an `Option<Backend>`; new persistent files go through it, never
`std::fs` directly. On the web the page-hide save cannot ask the world for a save (a hidden tab runs
no frames), so `save.rs::web_hooks` keeps a clone of the state after every turn (encoding is
milliseconds on the vast level) and the `visibilitychange`/`pagehide` listeners encode and write it. Unavailable storage means no saving plus a log
warning; failed writes (quota) are messages, never panics. Only the `web-sys` features in
`crates/warlock/Cargo.toml` are enabled (size).

Saves: Quit in the pause menu saves and quits (closing the window saves too; the `Q` key is unbound by default, rebindable); the game autosaves every 50 turns and on
level change to `run.sav` in the data dir (see "File locations" below). On startup an existing save is resumed unless `--new`, `--capture`,
`--replay`, or `--record` (replays and recordings always start a fresh run). F5 / F9 write / load `snapshots/quick.sav`. An unreadable save is renamed `run.sav.bad`,
never deleted. Format and `Content::fingerprint` live in `warlock-core/src/save.rs`.
`FORMAT_VERSION` is bumped by the integrator once per release, never in a feature PR
(`docs/dev/merging.md`): an incompatible `WorldState` change, or adding or reordering terrain, creatures,
lights, levels, objects, equipment slots, skills, weapon skills (fighting styles too), statuses, diseases, traps, or
dungeon branches, invalidates existing saves (affixes are saved by id: adding them does not,
removing one a save uses does); say so in the PR and leave the number alone.

**Permadeath**: when the player dies, `screens::on_death` deletes `run.sav`
(`SaveStore::delete_run`) and opens the death screen; `SaveStore::save_run` never writes a dead
run (it deletes instead), so autosave, quitting, and closing the window cannot bring it back, and the
web build parks no state for the page being hidden.

**File locations** (`crates/warlock/src/paths.rs`, the one place that decides; never call
`ProjectDirs` elsewhere): `paths::init(&args, check_only)` runs first in `main`, before logging,
and `paths::locations()` is what stores, logging, and the Files screen ask (it carries each
folder's `Reason`, shown by `--paths`). Precedence per folder: `--config-dir` / `--save-dir`, then
`--portable [DIR]`, then a `warlock-data` folder next to the game (`paths::game_folder`: the
executable's folder; on macOS next to the `.app`, never inside it), then the platform dirs
(`directories`, app name "warlock"; macOS `~/Library/Application Support/warlock`). Portable
layout `warlock-data/{config,data,logs}`; `Locations::prepare` creates it and warns (never
fails) when it is unwritable. `resolve` is pure over `Inputs` and tested without a filesystem; new
files go in the folder `Locations` gives (`keymaps_dir()` for key map files). `warlock --paths`
prints the report and exits; `warlock --move-data native|portable [DIR] [--dry-run]`
(`move_data.rs`: copy, compare, rename, remove; `*.bak-<time>` for conflicts; refuses while a game
holds the shared lock on `logs/.in-use`) moves files between the two layouts. Settings > Files
(`ui/files_screen.rs`, `ScreenName::Files`, `--screen files`) shows the report with Open folder.
