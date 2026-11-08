# Saving

Every character has a save of its own. You can stop a run and continue it later, and keep several
[characters](character.md) going side by side. There are no manual save slots and no going back:
saves are for continuing, not for retrying.

**Death is final.** When you die (see [Combat](hit-points.md#death)), that character's save is
deleted at once. Your other characters are untouched. A run that has ended is never saved again,
since there is nothing to continue.

## When the game saves

- When a run starts, so a new character is saved at once.
- When you close the window (no question asked).
- When you choose **Save and quit to menu** or **Quit** in the [pause menu](the-screen.md#menus)
  (`Esc`) and answer Yes. Quitting always asks first: `y`, or Enter on Yes, confirms; `n`, `Esc`,
  or Enter on No (the default) cancels. You can bind a "Save and quit" key in the
  [Controls](controls.md#rebinding-keys) screen; it asks too.
- Automatically every 50 [turns](time.md#what-takes-time).
- Automatically when you change level.

On the web the page is also saved when it is hidden or closed.

**Continue** on the [main menu](the-screen.md#menus) resumes the character you played last, and
names them. It appears only if a character that this version of the game can continue is saved.

**Characters** lists every saved character: name, [race](races.md) and [class](classes.md), and
[level](experience.md#levels). For the one selected it also shows where they are and when you last
played them (in UTC). Press `Enter` to resume one, or `Delete` to delete one for good (the game
asks first). A character saved by another version of the game is listed as made by that version.
It cannot be continued, and Continue skips it, but you can still delete it.

**Graveyard** appears once any character has died. It lists every fallen character, the one with
the most [experience](experience.md#levels) at the moment of death first: name, race and class,
level, and experience, then what killed them, where, when (in UTC), and how old they were.
Characters who died before the game began keeping their experience count as 0 and come last, the
highest level first. If the list is longer than the screen, the arrow keys or `>` and `<` turn the
page; `Esc` goes back.

**New run** lets you [create a character](character.md) for [the Ascmearc](ascmearc.md) and starts
a fresh run with a save of its own. It never touches your other characters.

## Where saves are kept

**Desktop.** Each character's save is a file in the `saves` folder of the game's data folder,
named after the character with a number: `brakka-1.sav`. Beside it, `brakka-1.toml` holds what the
list of characters shows. A second Brakka is `brakka-2`. Your [settings](settings.md) are
`settings.toml` in the configuration folder:

| System | Data folder (`saves` is in it) | Settings (configuration folder) |
|---|---|---|
| macOS | `~/Library/Application Support/warlock` | `~/Library/Application Support/warlock` |
| Linux | `~/.local/share/warlock` | `~/.config/warlock` |
| Windows | `%APPDATA%\warlock\data` | `%APPDATA%\warlock\config` |

Logs and crash reports are in a `logs` folder: on macOS and Linux inside the data folder, on Windows
in `%LOCALAPPDATA%\warlock\data\logs`.

To see the exact folders for your launch, and why they are there, run `warlock --paths`, or open
**[Settings](settings.md)**, then **Where are my files?**, which also has an **Open folder** action for each.

### Portable mode

To keep everything with the game, for a USB stick or a game folder you can copy, make an empty
folder named **`warlock-data`** next to the game (on macOS, next to `Warlock.app`, not inside it).
The game then keeps its [settings](settings.md), [key bindings](controls.md#rebinding-keys), saves, and logs in it:

| Inside `warlock-data` | What |
|---|---|
| `config/` | `settings.toml` (settings and key bindings) |
| `data/` | `saves/` (one save per character), and `snapshots/` |
| `logs/` | logs and crash reports |

`warlock --portable` does the same, and makes the folder if it is missing. `warlock --portable
DIR` uses the folder DIR instead. When several apply, the first of these wins: `--config-dir` and
`--save-dir` (each for its own folder), then `--portable DIR`, then a `warlock-data` folder next to
the game, then the standard folders above. If a portable folder cannot be written to (a read-only
disk), the game says so and carries on. It reads what is there but does not save.

`warlock --move-data native` moves a portable folder's files into the standard folders and renames
the emptied folder to `warlock-data.moved-<date>`. `warlock --move-data portable [DIR]` does the
reverse. Add `--dry-run` to see the plan first. Nothing is overwritten: a file already there is
kept as `NAME.bak-<time>`. It will not run while the game is running.

**Browser.** Saves and [settings](settings.md) are kept in your browser's local storage for the
page, under `warlock/saves/` (`warlock/saves/brakka-1.sav`). They survive reloads. If the browser
blocks storage (some private modes do), the game runs but cannot save, and writes a warning to its
log. If the browser runs out of room, a message appears on the [message
line](the-screen.md#messages).

## Good to know

- A save only works in the version of the game that made it. After an update, your earlier
  characters stay on the list, marked as made by another version, and cannot be continued. The
  older version of the game can still continue them.
- A save that cannot be read at all is renamed (`brakka-1.sav.bad`), never deleted, and no longer
  shown in the list of characters.
- Saves from before each character had its own (a single `run.sav`) are left where they are and
  not read.
- The [status bar](the-screen.md#the-screen) shows the run's **seed**: give it when you report a problem with a run.
