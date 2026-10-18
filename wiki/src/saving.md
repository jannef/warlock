# Saving

A run is a single save: you can suspend it and continue later, but there are no manual save slots
and no going back. Saves are for continuing, not for retrying.

## When the game saves

- When you press `Q` (save and quit), and when you close the window.
- When you choose **Save and quit to menu** or **Quit** in the pause menu (`Esc`).
- Automatically every 50 turns.
- Automatically when you change level.

On the web the page is also saved when it is hidden or closed.

Choose **Continue** on the main menu to resume the saved run (it appears only if a save exists).
**New run** lets you pick a [scenario](introduction.md#starting-a-run) and optionally a seed (a
number, or a word), and starts a fresh run, which replaces the saved one.

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
