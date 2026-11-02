Warlock
=======

A GPU-rendered ASCII roguelike. https://github.com/jannef/warlock

Run
---
- Linux / Windows: run the `warlock` / `warlock.exe` binary in this folder.
- macOS: open `Warlock.app`, or run the bare `warlock` binary from a terminal. The app is
  unsigned, so macOS will refuse to open it the first time: right-click it and choose Open, or run
  `xattr -dr com.apple.quarantine Warlock.app`.

Run `warlock --help` for command-line options. `warlock-cli` is the headless development tool.

Portable mode
-------------
To keep settings, saves, and logs with the game (a USB stick, say), create an empty folder named
`warlock-data` next to the game (next to Warlock.app on macOS), or run `warlock --portable`.
`warlock --paths` shows where your files are; `warlock --move-data native` moves them back out.

Keys
----
Every key can be rebound in Controls (main menu or pause menu). Esc always backs out. Controls can
export your keys to `warlock-keys.toml` in the settings folder and import them back (or run
`warlock --export-keys FILE` / `warlock --import-keys FILE`); `keymaps/default.toml` holds the
default keys. If your keys ever get in the way, `warlock --reset-keys` puts them all back.

Logs and crash reports
----------------------
Each run writes a log file; if the game crashes it also writes a crash report next to the logs
(in `warlock-data/logs` when portable, else):
- Linux:   ~/.local/share/warlock/logs
- macOS:   ~/Library/Application Support/warlock/logs
- Windows: %LOCALAPPDATA%\warlock\data\logs
Please attach the crash report when reporting a bug.

License
-------
Copyright (C) 2026 Janne Lê Forsell. Warlock is free software under the GNU General Public License,
version 3; see LICENSE. It comes with ABSOLUTELY NO WARRANTY. Source code:
https://github.com/jannef/warlock

The game embeds the JuliaMono font, licensed under the SIL Open Font License 1.1; see
JuliaMono-OFL.txt.

The game is built from many open-source Rust crates (Bevy, wgpu, and others). Their copyright
notices and license texts are in THIRD_PARTY_LICENSES.html; open it in a web browser.
