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

Logs and crash reports
----------------------
Each run writes a log file; if the game crashes it also writes a crash report next to the logs:
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
