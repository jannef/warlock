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

**Saves are pinned to the game version that wrote them** (#840; owner, 2026-11-02: "we do not
need to preserve saves"). The envelope (`warlock-core/src/save.rs`) is `magic`, `game_version`
(`warlock_core::GAME_VERSION`, the workspace version), `content_fingerprint`, and the state;
`save::decode` refuses a save whose `game_version` is not this build's (`SaveError::OtherVersion`,
before the state is read; a save that names none, from before the pin, is "an older version"),
then one whose fingerprint differs (`ContentMismatch`), then one that does not read (`Corrupt`).
Nothing carries a save across versions: no format number, no id maps, no lenient reads. A state
naming something the content does not have (a talent, a story line, an affix value outside its
tier, ...) is refused by `World::from_state`'s checks, never dropped or mapped.
`#[serde(default)]` on new fields stays (harmless, and it keeps replays readable). Between two
releases every development build shares the version, so there the content fingerprint and the
decoder are what refuse a save a build cannot read (a renamed `.sav.bad`, as below); saves are
not promised across development builds.

A save holds only the current floor of an endless branch (`docs/dev/mapgen.md`,
`world/endless.rs`): `WorldState::endless` is a count of floors made and the deepest depth by
branch, left out while empty, so its size does not grow with depth.

The summary beside a save records the version too (`Summary::version`; empty in a summary from
before the pin, which counts as another version). `SavedCharacter::playable` is true when the
summary names this version, or there is no readable summary (the save is tried, and the
envelope decides). The Characters screen lists another version's save with `other version` in
the level column and `Made by version 0.8.0 of the game: it cannot be continued here.` above
its place; Enter does nothing on it, Erase deletes it through the dialog. Continue
(`SaveStore::last_played`, the main menu's Continue row, and a start that skips the menu)
resumes the character played last that is playable; the main menu offers Characters while any
save is listed, Continue only with a playable one. Resuming another version's save
(`Resume::Failed`) leaves it as it is: never renamed `.sav.bad`, still listed. The screen
snapshots' fixture (`tests/golden/saved-characters`) holds Osric, made by 0.8.0 and played
last; the other summaries carry this version, so **the release that bumps the workspace
version sets their `version` too** (`save.rs` test `the_listed_fixture_is_of_this_version`
says so when it fails).

Replays are not pinned: a replay records `game_version`, but `replay --verify` and `--replay`
play it in any version and compare the state hashes step by step, which is the check that
matters (a replay whose hashes still match is still good, and the tests' replays live across
releases). A version other than this build's is said as a warning beside the result, never a
failure (`Replay::version_note`).

**One save per character** (#218; `save.rs`, `save/characters.rs`): every character has a save id,
a slug of its name and the lowest free number (`characters::fresh_id`: `brakka-1`, then
`brakka-2`; an id is taken while any file of it is left: its save, a kept `.sav.bad`, its summary,
its snapshot), and the files `saves/<id>.sav` (the world, `warlock_core::save`) and, beside it,
`saves/<id>.toml` (`save::Summary`: name, race and class names, level, place, the campaign's name
(`docs/dev/campaign.md`), and when it was last
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
summaries are read; the screen snapshots' `main-menu-saved` and `characters-screen` read it too). A pre-#218 `run.sav` is left alone: not read, listed, or deleted (its
version is not this one's, and its character has no name).

The envelope lives in `warlock-core/src/save.rs`, `Content::fingerprint` in the content. There
is no format number to bump: a release's new workspace version is what retires the saves of the
last (`docs/dev/merging.md`). An incompatible `WorldState` change, or adding or reordering
terrain, creatures, lights, levels, objects, equipment slots, skills, weapon skills (fighting
styles too), statuses, diseases, traps, or dungeon branches, invalidates development saves of
the same version too (affixes are saved by id: adding them does not, removing one a save uses
does); say so in the PR. A new optional field (`Character::prestige`, #876: `#[serde(default,
skip_serializing_if)]`) changes nothing a save from before holds; prestige classes and callings
are not fingerprinted (they are read by id). `Stack::spirit` (#881, a spirit weapon in the hand)
is the same: left out while false, saved and hashed while held.

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
