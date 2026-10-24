# Sunlight, moonlight, and weather

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/05-visibility.md. Keep it current with the code, as CLAUDE.md says.

**Sunlight** (spec §5 "Sunlight", `light/sun.rs`): a level gets the sun and sky iff
`LevelKind::is_sky_exposed()` (outdoor and the overworld; add new sky kinds there, nowhere
else); terrain `roofed` never does. `Sun::at(time)` reads a per-minute table (`SUN_PATH`) built by
`const fn`s with integer math only (fixed-point sine), so no runtime trig; a test pins a hash of
the whole table — after a deliberate change to the sun's path or colors, update the hash.
`Sun::shadow_steps(height)` is the one shadow line (core terrain shadows and renderer creature
shadows both use it). `Sunlight::compute` marches each cell of the bounce area toward the sun
(terrain `height`, `Content::tallest_terrain` bounds the reach); it is not cached. The sun is a
`SeenLight` with key `SUN_KEY` and `emitter: Emitter::Sun(sun)` (point lights:
`Emitter::Point { pos, preset }`; use `SeenLight::pos()`/`preset()`), last in `light_sources()`;
sky light goes into base light. `PerceptionView::sun()` gives the sun and sky of the moment on a
sky-exposed level (`is_up`, `is_night`, `elevation`), e.g. for day/night rules. The showcase
courtyard (`levels/courtyard.toml`, scenario `courtyard`) is up the hub stairs nook's `<`.

**Moonlight** (spec §5 "Moonlight", `light/moon.rs`): the moons are `data/sky.toml`
(`content/sky.rs`, `MoonDef`, `Content::moons()`; optional, none without the file, so test
fixtures have dark nights unless they add one). `MoonDef::phase_at(time)` is its age over its
period (16-bit fraction, 0 new, half full). `Moon::at` walks the sun's path (`Sun::at_minute`)
trailing the sun by its phase of a day, so a moon's elevation and shadow line are the sun's at that
minute; its `color` and `sky` scale by lit fraction, elevation (`RISE`), daylight (none while the
sun is up, all from `DUSK_DEPTH` 6° below), and size², integer math only; a test pins a hash of a
whole cycle — update it after a deliberate change. **`Sky::at(time, content.moons())` is the one
place sky light is assembled** (sun, moons, `Sky::glow` for base light, `Sky::moonlight` for
overworld sight): anything that dims the whole sky (weather's clouds) applies there. A moon that
`shines()` is a `SeenLight` with key `moon_key(index)` and `Emitter::Moon(moon)` after the sun in
`light_sources()`, lit by `Sunlight::compute(shadow, light, ...)` like the sun;
`Emitter::shadow_steps` gives the shadow line of either. `PerceptionView::sky()` has the moons.
The status bar shows the phase (`MoonPhase`, glyph in `ui/mod.rs`) and the full date
(`Calendar::full_date`); its segments give way by priority, the moon cut to its glyph first
(`ui::fit_segments`). Tune moonlight with captures of the `courtyard` and `forest` scenarios after
`skip:353h skip:30m` (full moon at midnight) and `--content-dir assets`, which reads `sky.toml`
from disk without a rebuild. The wiki's moons and overworld-sight tables come from `sky.toml`
(`Moon::brightest`); run `cargo xtask wiki` after changing it.

**Weather** (spec §5 "Weather", `weather.rs`; data `data/weather.toml`, `content/weather.rs`,
`Climate`, `WeatherDef`, `Content::climate()`; optional, always clear without the file, so test
fixtures without it have clear skies — `ContentFiles::remove("data/weather.toml")` gives built-in
content a clear sky, as the overworld tests do): stateless, `weather::conditions(seed, time,
climate, override)` (weather and `Clouds`) and `weather_at` (just the weather, cheap). Blocks of
~3 h with a hashed start shift; a block's weather is the Irwin–Hall-spread sum of `WINDOW` block
draws against the month's chances; clouds ramp over `CHANGE` around a change. Never keep weather
state: only the dev override (`Action::SetWeather`, `weather:rain`, `WorldState::weather_override`,
3 hours) lives in the world state. The clouds apply in `Sky::at(time, moons, conditions)`, and
nowhere else (rounded up, so `is_up`/`shines` never change); `Sky::clear` is the sky without them;
`WorldState::sky` is the sky of the moment for perception and creature sight alike.
Perception knows the weather only on sky-exposed levels (`Sky::conditions`,
`PerceptionView::weather`); `World::perceive` compares it with the previous snapshot's and says the
change (`weather_news`: rain starting or growing heavier `say`s, which stops a rest; the rest
`note`s, which does not). Overworld sight loses the weather's `sight` (`daylight::overworld_sight`).
A Weather-wise player (`TalentMods::forecast`, hours) also gets `weather::forecast` in the snapshot
(`PerceptionView::forecast`): the first `weather_at` other than the present one, every ten minutes
up to the hours, the dev override included (no state, no draw).
Look at the weather of a seed: `WARLOCK_SEED=1 WARLOCK_FROM=4,1 WARLOCK_DAYS=10 cargo test -p
warlock-core --lib print_weather -- --ignored --nocapture` (a line per day, `.` clear, `o`
overcast, `r` rain, `R` heavy). Tune the clouds and the rain with captures of the `courtyard` and
`forest` scenarios after `skip:330m weather:rain` (noon) and `skip:353h skip:30m weather:heavy`
(full-moon midnight) with `--content-dir assets`; the wiki's weather tables come from the file.
