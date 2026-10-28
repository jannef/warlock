# Settings

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/06-input-and-ui.md. Keep it current with the code, as CLAUDE.md says.

Settings (`crates/warlock/src/settings/`): `settings.toml` in the config dir, schema in the `settings/mod.rs` module docs. Changes
apply live and are written at once; a bad file means defaults plus a warning, per value. The
temperature unit (`temperature = "celsius"`/`"fahrenheit"`, `settings::TemperatureUnit`) is
applied only where the status bar shows the temperature (`TemperatureUnit::show`). Effects:
`reduced_effects` is a preset over the `[effects]` switches (`EffectSwitches`: glyph_animation,
flicker, motion, glow, distortion, particles, screen_fades, screen_shake, weather; all on by default), all but
`blood` (on by default too), a content preference the preset leaves as set
(`EffectSwitches::effective`; its settings row is not indented and never says "(preset)"). Each consumer reads
its own switch from the `render::Effects` resource (the effective values, synced every frame from
`Settings` + `--reduced-effects`), never the preset; a new effect needs a switch in
`EffectSwitches`, the file, and a row in `ui/settings_screen.rs`. `clock` (`settings/clock.rs`, `"24h"`/`"12h"`) is the one way a time of day is written
in the UI (`Clock::format`: status bar, death screen); dumps stay 24-hour. Palettes
(`settings/palette.rs`) are applied only when cells are uploaded to the GPU (`render::upload`) and
in `ui/glyphs.rs`, never to content colors.

Key bindings (spec §6 "Rebinding", #159) are the `Bindings` resource (`input.rs`; text forms,
ids, and the rules in `settings/keys.rs`). **Only play-context commands are rebindable by
design; the keys of menus, menu lists, screens, info pages, prompts, dialogs, text fields, and the
targeting mode are fixed, and new ones are plain fixed keys in their screen or input-mode code,
listed in its hint line, with no `KeyCommand` or `Context` entry (owner, 2026-10-26). The
contexts below are the existing menu-context bindings, which stay as they are (not widened, not
removed; a cleanup is a later owner call); the back-style ones (`Back`, `TextBack`, `MoreSkip`,
`PrevPage`, `PrevTarget`, all unbound) stay exactly as they are (owner, #644). Every key read so far is a `KeyCommand` in its
`input::Context`**: `Play` (the map, direction prompts, look mode), `Aim` (`NextTarget`,
`PrevTarget`, `Loose`), `Prompt` (the rest choices), `More` (at `--more--`: `MoreNext`, Space and
Enter; `MoreSkip`, unbound, as Esc always skips; `ui/more.rs`), `Menu` (`MenuUp`/`Down`/`Left`/`Right`,
`Confirm`, `Back`, `Erase`, `Mark`, `MarkAll`, and the item lists' `NextListPage` (`>`,
PageDown) and `PrevListPage` (`<`, PageUp): `Press::from_input` turns them into `Nav`, so
screens read `press.nav`, never a key), `Text` (a text field, the character's name:
`TextConfirm` (Enter; vital), `TextErase` (Backspace, Delete), `TextRandom` (Tab), `TextBack`
(unbound: Esc always backs out); `Press::from_input` turns them into `press.edit`, and every key
bound to none of them types its text, literally, so a text field never reads the `Menu` keys),
`Pages` (Tab; the previous info page is Shift with it,
`PrevPage` unbound by default), `Dialog` (`Yes`, `No`), and `Dev` (the developer
keys: in every table, read by `input::command_in` and listed by Controls only where
`input::dev_keys` is true). Keys are unique within a context; a mode with its own context reads it
before play's (`command_in`). Nothing matches a `KeyCode` or a character directly but Esc (fixed:
Cancel in play, which skips the rest at `--more--`, `Nav::Back` on every screen) and the letters
and digits that name list rows (and the text a text field types).
Hints name keys through `Bindings::key_name` and `ui::menu::Hints` (footers), never a literal key.
Core knows no bindings, so a message names a key by its command's id (`messages::key("help")`
writes `{key:help}`, as the welcome line does) and the message line fills it in as bound
(`Message::display_with` with `ui::message_key`; dumps show the token).
Binding a key another command of the same context uses moves it
(`Bindings::bind_primary` returns `Bound::Moved`); refused with `Conflict` are the keys of fixed
commands outside `REBINDABLE` (Esc's Cancel) and the last key of a `VITAL` command (`MenuUp`,
`MenuDown`, `Confirm`: what reaches Controls and its Reset all keys from anywhere; `TextConfirm`:
what gets a new character past its name). `reset_command`
keeps a vital command's keys rather than leave it keyless, and `ensure_vital` runs after a
settings file or a keymap is read, so nothing can lock the player out; `--reset-keys`
(`settings::load_for`) puts the defaults back and saves them. A command left keyless is saved as
an empty list and loads unbound. Rebindable commands have stable ids, unique across contexts, in
`settings/keys.rs` (`KeyCommand::id`): **a new `KeyCommand` variant must be added there** (in
`REBINDABLE` next to its group, its `Context` in `KeyCommand::context`, a default key in
`Bindings::default` or an entry in `UNBOUND`), never a key read of its own. An ability's own
command also goes in `ABILITY_KEYS`, which Controls lists last, below every context's group
(#644); the play keys never take a vi letter by default (#644).

The default keys (`Bindings::default`) are ADOM's (#350; the source is ADOM's manual, its
"Miscellaneous commands" list): a command ADOM has takes ADOM's key in its context, one ADOM lacks
(or has only behind a prefix like `:m` or `w s`, or Ctrl, which a binding cannot hold) a key ADOM
leaves free (`*`, `v`, `b`, `X`), and Throw is Missile's `t` in capitals, Search for a while Search's `S` (ADOM's save and quit: Quit has no default key, the pause menu quits). Movement is the
numpad and the arrows only (ADOM's `k`, `l`, `u` are kick, look, use a tool). A new default
key must not be an ADOM key of another command. Changing defaults never rewrites a player's file:
`settings.toml` lists every command's keys once written, so an old file keeps its old keymap, and
`Bindings::from_table` gives a command the file does not list (new, or unbound when the file was
written, like Search for a while before `S`) its defaults minus any key a listed command of its context has.

The keymap file (`settings/keymap.rs`, #159): `export` writes every rebindable command by context
(`[play]`, `[aim]`, ... from `Context::id`, `version = 1`); `import` applies a file to the live
bindings the way rebinding does (only the listed commands change, keys move within a context, the
first listed in `REBINDABLE` order wins a key, unknown ids and keys are warnings, except the ids in `keys::RETIRED` (deleted commands such as `fire`, dropped silently, in `from_table` too), the lockout rules
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
