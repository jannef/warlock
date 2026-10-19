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
in `ui/glyphs.rs`, never to content colors. Binding a key another command of the same context uses moves it
(`Bindings::bind_primary` returns `Bound::Moved`; fixed commands outside `REBINDABLE`, Esc's Cancel,
are refused with `Conflict`); a command left keyless is saved as an empty list and loads unbound.
Rebindable commands have stable ids in
`settings/keys.rs` (`KeyCommand::id`): **a new `KeyCommand` variant must be added there**
(`REBINDABLE` or the non-rebindable arm). `Game` exists only during a run (the menu runs without
it): gate systems that read it with `run_if(resource_exists::<Game>)`; content is the
`GameContent` resource.
