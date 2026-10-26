# Temperature and ice

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/05-visibility.md. Keep it current with the code, as CLAUDE.md says.

**Temperature** (spec §5 "Temperature", `temperature.rs`; M26): `Temperature` is tenths of a
degree Celsius (`celsius()`/`fahrenheit()` round to whole degrees for display). Underground it is
fixed per level (`LevelState::temperature`, from the level file's `temperature`, else the
branch plan's in `data/dungeons.toml`, else `UNDERGROUND` 10 °C; refused on sky-exposed
levels); under the open sky `temperature::outdoors(seed, time, climate, override)` is stateless
like the weather: the month means (`temperature` per `[[month]]` in `data/weather.toml`,
interpolated between mid-months) plus a cosine day swing (coldest 04:00; `swing` at the top of the
file, times each weather's `swing` percent) less the weather's `chill`, blended across weather
changes by `weather::blend` (the one place the clouds' and the temperature's hour-long
transition is decided). `WorldState::temperature_on(level, ..)` is the one place a level's
temperature is decided, and `WorldState::temperature_at(level, .., pos)` a cell's (the level's
plus a hot or a cold special room's, `docs/dev/special-rooms.md`); `World::temperature()` (ground
truth, at the player's cell), `World::temperature_at`, and `PerceptionView::temperature()` (the
player feels it where they stand) read it. The status bar always shows it (`ui::status_segments`,
priority 4, after the weather; cool tint below 0, warm from 30 °C), in the settings' unit.
Nothing feels the cold yet.

**Ice** (spec §5 "Ice", `ice.rs` the model, `world/ice.rs` the rules; M26): water terrain with a
`freeze` table grows ice; terrain with `ice = MM` is what ice of at least that thickness looks
like (`Content::ice_terrain`, `ice_look`). Ice is kept per level, sparsely, in µm
(`LevelState::ice`: `Ice { water, um }`), and the terrain grid shows its look (the water when
too thin), so sight, memory, look mode, and noise just follow; ice and freezing water are
transparent and lightless (validated), so light masks never need invalidating. **Never tick
ice per cell per second**: `LevelState::advance_ice` brings a level up to a time in fixed
10-minute ticks of the clock (`ice::ticks`, each at its middle's temperature plus a special
room's warmth at the cell; `ice_until` the last tick done), grouping cells by (water, thickness,
warmth), so it is exact however time passed (rests,
skips, journeys) and a year away is cheap. It runs on the current level as the clock moves
(`run_turn`) and as perception begins, and for a new sky-exposed level over `ice::PRE_ROLL` (30
days) on instantiation (and for a new level with a hot or a cold room). Stepping onto ice goes through `World::tread` (player and creatures
alike, and one who makes way for the player onto their old cell, `talk.rs::swap_places`; weight
from `Actor::weight`, `ice::gives_way`: no draw up to what it holds, a gameplay draw up to twice
that), then `tread_price` (the `climb` action from water onto ice) and `after_tread` (the plunge:
`freeze.plunge` dice through `hurt` with `Harm::Spared`, never the last hit point,
`Event::FellThroughIce`). Bursts: `World::frost` (instant freeze; the frost trap, a burst with
`frost`; dev `freeze:x,y,r`, which also shows the `frost_burst` effect of `data/effects.toml`
through `World::frost_look`) and `World::shock` (`Shock::Heat` melts: the fireball trap, a burst
with `heat`; `Shock::Blast(um)` shatters thinner ice, for blasts without heat, none yet; dev
`thaw:x,y,r`) are the hooks traps (`world/trap.rs`, `docs/dev/traps.md`) and later spells call. Saved and hashed. Tests: `ice.rs`, `temperature.rs`,
`world/ice_tests.rs` (growth/melt over rests, weight, falling in, climbing out, sinking objects,
creatures, bursts, saves, replays, a year's catch-up). Tune the look with captures of the Frost
Grotto (hub stairs nook, the `>` below the first) and the courtyard in winter (`skip:` into
Deepwinter) with `--content-dir assets`. Terrain `variants` (presentation-only:
`TerrainDef::glyph_at`, a glyph per cell by position) draw thin ice's hairline cracks.
