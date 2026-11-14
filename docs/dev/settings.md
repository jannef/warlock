# Settings

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/06-input-and-ui.md. Keep it current with the code, as CLAUDE.md says.

Settings (`crates/warlock/src/settings/`): `settings.toml` in the config dir, schema in the `settings/mod.rs` module docs (the config dir also keeps `graveyard.toml`, the dead, append-only and unpinned: `docs/dev/saves.md` "The graveyard"). Changes
apply live and are written at once; a bad file means defaults plus a warning, per value. The
temperature unit (`temperature = "celsius"`/`"fahrenheit"`, `settings::TemperatureUnit`) is
applied only where the status bar shows the temperature (`TemperatureUnit::show`). Effects:
`reduced_effects` is a preset over the `[effects]` switches (`EffectSwitches`: glyph_animation,
flicker, motion, glow, distortion, particles, screen_fades, screen_shake, weather; all on by default), all but
`blood` (on by default too), a content preference the preset leaves as set, and `circles` (the
spell circles, on by default: information the preset only stills, as `motion` off does)
(`EffectSwitches::effective`; their settings rows are not indented and never say "(preset)";
the gap before the settings screen's actions goes first when the window is short). Each consumer reads
its own switch from the `render::Effects` resource (the effective values, synced every frame from
`Settings` + `--reduced-effects`), never the preset; a new effect needs a switch in
`EffectSwitches`, the file, and a row in `ui/settings_screen.rs`. `clock` (`settings/clock.rs`, `"24h"`/`"12h"`) is the one way a time of day is written
in the UI (`Clock::format`: status bar, death screen); dumps stay 24-hour. Palettes
(`settings/palette.rs`) are applied only when cells are uploaded to the GPU (`render::upload`) and
in `ui/glyphs.rs`, never to content colors.

Key bindings (spec §6 "Rebinding", #159) are the `Bindings` resource (`input.rs`; text forms,
ids, and the rules in `settings/keys.rs`). **Only play-context commands are rebindable by
design; the keys of menus, menu lists, screens, info pages, prompts, dialogs, text fields, and the
targeting mode are fixed** (owner, 2026-10-26; #1533), plain keys in `settings/fixed.rs`, which
each place reads, with no `KeyCommand` or `Context` entry: the menu keys (`fixed::menu`: arrows
and `hjkl` move, Enter and NumEnter choose, Delete and Backspace erase, Space and `,` mark, `>`
and `<` (PageDown, PageUp) turn an item list's pages; `Press::from_input` turns them into `Nav`,
so screens read `press.nav`, never a key, and the numpad's 8/2/4/6 are the arrows,
`menu::numpad_nav`), the text field keys (`fixed::text`: Enter accepts, Backspace and Delete
erase, Tab a random value, as `press.edit`; every other key types its text, literally, so a text
field never reads the menu keys), the dialog's `y` and `n` (`fixed::dialog`), the info pages'
top-row digits and Tab (`fixed::page`, Shift reverses Tab), the rest prompt's `h`, `d`, `n`
(`fixed::REST`), and aiming's Enter (loose) and Tab (next target; `fixed::aim`), the last two read
by `Gameplay::fixed_key` before the bindings. Aiming is also loosed by the play key that started
it (`Aim::confirmed_by`: `t` for a shot, a throw, a zap, `T` for a throw, `r` for a scroll, the
abilities key `a` or its quick repeat `X` for an ability, an ability's own key; the owner,
#1533). The back-style commands (`Back`, `TextBack`, `MoreSkip`, `PrevPage`, `PrevTarget`, all
unbound) stay rebindable exactly as they were (owner, #644), and may not take a key fixed where
they are read (`fixed::refusal`, through `KeyBinding::refused_for`); `fixed::read` tries each key
a press may stand for, most specific first, against the place's own command, then its fixed
keys. **Every rebindable key is a `KeyCommand` in its `input::Context`**: `Play` (the map,
direction prompts, look mode), `Aim` (`PrevTarget`), `More` (at `--more--`: `MoreNext`, Space and
Enter; `MoreSkip`, unbound, as Esc always skips; `ui/more.rs`), `Menu` (`Back`), `Text`
(`TextBack`), `Pages` (`PrevPage`), and `Dev` (the developer keys: in every table, read by
`input::command_in` and listed by Controls only where `input::dev_keys` is true). Keys are unique
within a context; a mode with its own context reads it before play's (`command_in`). Nothing else
matches a `KeyCode` or a character directly but Esc (fixed: Cancel in play, which skips the rest at
`--more--`, `Nav::Back` on every screen) and the letters and digits that name list rows (and the
text a text field types).
Hints name play keys through `Bindings::key_name` and the fixed ones through `settings::fixed`'s
names (`ui::menu::HINTS` for footers), never a literal key of their own.

Modifier keys (#1323, `settings/mods.rs`): a play command's key may be held with Ctrl, Alt, and
Super (Command on macOS, the Windows/Super key elsewhere): `KeyBinding::Chord(Mods, key)`, made by
`KeyBinding::chord` (never empty modifiers, never nested, Shift only with a physical key: a
character carries Shift itself, `K` against `k`). The text form puts the modifiers first, `"Ctrl+r"`,
`"Alt+ArrowUp"`, `"Shift+ArrowUp"`, `"Super+s"`, written Ctrl, Alt, Shift, Super in that order;
reading also takes `Control`, `Option`, `Cmd`, `Command`, `Win`, `Meta` (`Mods::split`), and
`settings.toml` and the keymap file use it alike. `KeyBinding::display` names the modifiers as
the platform does (`Ctrl+Option+Cmd+s` on macOS, `Ctrl+Alt+Super+s` elsewhere and on the web).
The router reads the held modifiers from `ButtonInput<KeyCode>` (`Mods::held`; a modifier pressed
alone is dropped before anything reads it) and every lookup takes them
(`Bindings::lookup_in(context, input, mods)`, `Press::from_input(input, mods, bindings)`).
Matching is exact, through `mods::press_candidates`, tried in order: the physical key (held
with Ctrl, Alt, or Super a chord whose key is a bound physical key, else the typed character, or
the physical key's US letter or digit when Alt is held or the typed character is a control
character, a letter's case from Shift: `mods::press_binding`, which is also `Press::binding` and
what Controls captures), with Shift held first with Shift and then without; then the plain
character typed, when neither Super nor
Ctrl alone is held and, with Alt, it is no ASCII letter or digit (Option and AltGr type `@` and
`{` on many layouts; Windows reports AltGr as Ctrl+Alt). So `Ctrl+r` never fires `r`, nor `r`
while Ctrl is held, nor `Alt+r`; Shift+Tab is still a Tab its command reverses. `Press::binding`
never holds Shift (screens read `press.shift`); Controls adds it to a physical key captured for a
play command. `KeyBinding::refused_for` (`mods::refusal`) refuses a modifier on a command of
another context, and the shortcuts the system or the browser keeps (macOS: Cmd+Q/W/H/M, Cmd+Tab,
Cmd+Space; Windows and Linux: Alt+F4, Alt+Tab, Ctrl+Alt+Delete, Super+D/E/L/R/Tab; the web:
Ctrl or Cmd with N/Q/T/W, Ctrl+Tab, Alt+F4), in Controls' capture (with the reason) and when a
settings file or keymap is read (a warning; the key is skipped). The default keys with
modifiers all hold Alt, on every platform and the web alike (#1385): Search for a while is Alt+Num5
and Alt+s (#1323's comment, #1326); Run is Alt with each movement key, Alt+Num1-9 and Alt+arrows
(`Bindings::default` derives them from the moves; #1325), none refused on any platform
(`mods::tests::default_runs_are_never_refused`). They were Ctrl until #1385: macOS keeps
Ctrl+arrows for Mission Control and Spaces, so they never reached the game there. Alt is Option on
a Mac, named `Alt` there too (`Mods::display_on`; only Super is `Cmd`), and Option+letter is read
from the physical key (`Alt+s`, never `Alt+ß`). In a browser, Alt+Left/Right (Back/Forward on
Windows and Linux) never leave the canvas: winit cancels every keydown on it
(`prevent_default_event_handling`, `main.rs`). A settings file written before #1385 lists the
old Ctrl keys and keeps them (Controls' reset gives the new ones).
Core knows no bindings, so a message names a key by its command's id (`messages::key("help")`
writes `{key:help}`, as the welcome line does) and the message line fills it in as bound
(`Message::display_with` with `ui::message_key`; dumps show the token).
Controls binds a key in place of one of a command's keys or after its last
(`Bindings::bind_at(command, key, Some(n) | None)`, #1324; `bind_primary`, the tests' shorthand, is `Some(0)`; a key the
command has already trades places with the one replaced) and removes one by place
(`Bindings::remove_key`). Binding a key another command of the same context uses moves it
(`bind_at` returns `Bound::Moved`); refused with `Conflict` (taken or removed) are the keys of fixed
commands outside `REBINDABLE` (Esc's Cancel). What reaches Controls and its Reset all keys from
anywhere (the menu keys, Enter) and what gets a new character past its name are fixed keys, so
nothing can lock the player out; `--reset-keys`
(`settings::load_for`) puts the defaults back and saves them. A command left keyless is saved as
an empty list and loads unbound. Rebindable commands have stable ids, unique across contexts, in
`settings/keys.rs` (`KeyCommand::id`): **a new `KeyCommand` variant must be added there** (in
`REBINDABLE` next to its group, its `Context` in `KeyCommand::context`, a default key in
`Bindings::default` or an entry in `UNBOUND`), never a key read of its own. An ability's own
command also goes in `ABILITY_KEYS`, which Controls lists last, below every context's group
(#644); the play keys never take a vi letter by default (#644).

The default keys (`Bindings::default`) are ADOM's (#350; the source is ADOM's manual, its
"Miscellaneous commands" list): a command ADOM has takes ADOM's key in its context, one ADOM lacks
(or has only behind a prefix like `:m` or `w s`) a key ADOM leaves free (`*`, `v`, `b`, `X`),
and Throw is Missile's `t` in capitals, Wait/search numpad `5` and `s` (ADOM's search key; `.`
is no longer bound, #1326), Search for a while Alt with either (Quit has no default key, the
pause menu quits). Movement is the
numpad and the arrows only (ADOM's `k`, `l`, `u` are kick, look, use a tool). A new default
key must not be an ADOM key of another command. Changing defaults never rewrites a player's file:
`settings.toml` lists every command's keys once written, so an old file keeps its old keymap, and
`Bindings::from_table` gives a command the file does not list (new, or unbound when the file was
written, like Search for a while before its Ctrl keys) its defaults minus any key a listed command of its context has. A command merged into another
(`keys.rs::MERGED`: `search` into `wait`, #1326) has its keys moved to the one that took it over,
after that one's own, by `fold_merged`, for a settings file and a keymap alike.

The keymap file (`settings/keymap.rs`, #159): `export` writes every rebindable command by context
(`[play]`, `[aim]`, ... from `Context::id`, `version = 1`); `import` applies a file to the live
bindings the way rebinding does (only the listed commands change, keys move within a context, the
first listed in `REBINDABLE` order wins a key, unknown ids and keys are warnings, except the ids in `keys::RETIRED` (deleted commands such as `fire`, and the menu, dialog, text field, info page, rest prompt, and aiming commands that became fixed keys in #1533, dropped silently, in `from_table` too) and the context tables in `keys::RETIRED_CONTEXTS` (`[dialog]`, `[prompt]`), the lockout rules
hold) and returns a `Report`; version 0 is a flat table of ids or a settings file's `[bindings]`.
`KeymapFile` (from `SettingsStore::keymap`) is where Controls' Export keys and Import keys go:
`warlock-keys.toml` beside `settings.toml` (desktop), the browser (a blob download; a file input
whose file arrives through `keymap::take_picked`, applied by `controls_screen::receive_keymap`),
or nowhere (captures). `--export-keys FILE` / `--import-keys FILE` are file commands
(`keymap::command`, from `main.rs::file_command`). The web build's `web-sys` features for them:
`Blob`, `BlobPropertyBag`, `Element`, `File`, `FileList`, `HtmlAnchorElement`, `HtmlElement`,
`HtmlInputElement`, `Url`. `assets/keymaps/default.toml` is the defaults as a file (shipped in the
desktop archives); after changing a default key, regenerate it with
`WARLOCK_UPDATE_KEYMAPS=1 cargo test -p warlock keymap` (a test fails until then).

`Game` exists only during a run (the menu runs without
it): gate systems that read it with `run_if(resource_exists::<Game>)`; content is the
`GameContent` resource.
