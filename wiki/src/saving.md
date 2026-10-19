# Saving

A run is a single save: you can suspend it and continue later, but there are no manual save slots
and no going back. Saves are for continuing, not for retrying.

**Death is final.** When you die (see [Combat](combat.md#death)), the save is deleted at once,
and a run that has ended is never saved again: there is nothing to continue.

## When the game saves

- When you press `Q` (save and quit), and when you close the window.
- When you choose **Save and quit to menu** or **Quit** in the pause menu (`Esc`).
- Automatically every 50 turns.
- Automatically when you change level.

On the web the page is also saved when it is hidden or closed.

Choose **Continue** on the main menu to resume the saved run (it appears only if a save exists).
**New run** lets you pick a [scenario](introduction.md#starting-a-run) and optionally a seed (a
number, or a word), then [create your character](character.md), and starts a fresh run, which
replaces the saved one. Your character is saved with the run.

## Where saves are kept

**Desktop.** The save is the file `run.sav` in the game's data folder, and your settings are
`settings.toml` in its configuration folder:

| System | Save (data folder) | Settings (configuration folder) |
|---|---|---|
| macOS | `~/Library/Application Support/warlock` | `~/Library/Application Support/warlock` |
| Linux | `~/.local/share/warlock` | `~/.config/warlock` |
| Windows | `%APPDATA%\warlock\data` | `%APPDATA%\warlock\config` |

Logs and crash reports are in a `logs` folder: on macOS and Linux inside the data folder, on Windows
in `%LOCALAPPDATA%\warlock\data\logs`.

To see the exact folders for your launch, and why they are there, run `warlock --paths`, or open
**Settings**, then **Where are my files?**, which also has an **Open folder** action for each.

### Portable mode

To keep everything with the game, for a USB stick or a game folder you can copy, make an empty
folder named **`warlock-data`** next to the game (on macOS, next to `Warlock.app`, not inside it).
The game then keeps its settings, key bindings, save, and logs in it:

| Inside `warlock-data` | What |
|---|---|
| `config/` | `settings.toml` (settings and key bindings) |
| `data/` | `run.sav`, and `snapshots/` |
| `logs/` | logs and crash reports |

`warlock --portable` does the same and makes the folder if it is missing; `warlock --portable DIR`
uses the folder DIR instead. Which folder wins, first to last: `--config-dir` and `--save-dir`
(each for its own folder), then `--portable DIR`, then a `warlock-data` folder next to the game,
then the standard folders above. If a portable folder cannot be written (a read-only disk), the
game says so and carries on, reading what is there but not saving.

`warlock --move-data native` moves a portable folder's files into the standard folders and renames
the emptied folder to `warlock-data.moved-<date>`; `warlock --move-data portable [DIR]` does the
reverse. Add `--dry-run` to see the plan first. Nothing is overwritten (a file already there is
kept as `NAME.bak-<time>`), and it will not run while the game is running.

**Browser.** The save is kept in your browser's local storage for the page, under `warlock/run.sav`,
as are your settings. It survives reloads. If the browser blocks storage (some private modes), the
game runs but cannot save, and writes a warning to its log. If the browser runs out of room, a
message appears on the message line.

## Good to know

- A save only works with the same game content it was made with. After a game update that changes
  the world's content, an old save may be rejected with a message. The unreadable file is renamed
  `run.sav.bad` and never deleted, and a new run starts.
- Saves from before version 1.0 may be rejected after updates. From 1.0 on, saves will be migrated.
- The run's **seed** (shown in the status bar) decides the generated levels and the weather. The
  same seed gives the same levels and the same weather in the same game version. Seeds are not
  kept the same across versions.
