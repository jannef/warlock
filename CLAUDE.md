# CLAUDE.md — working on Warlock

Warlock is a GPU-rendered ASCII roguelike in Rust + Bevy. **`specification.md` is the source of
truth** for what to build; this file is about *how* to work on it. Update both together whenever
behavior, commands, or architecture change.

## Toolchain

- Rust is pinned in `rust-toolchain.toml` (rustup installs it automatically).
- Cargo lives in `~/.cargo/bin`. In a fresh shell: `source ~/.cargo/env` (bash/zsh) or
  `source ~/.cargo/env.fish` (fish).

## Commands

| Task | Command |
|---|---|
| Build everything | `cargo build --workspace` |
| Run the game | `cargo run -p warlock` |
| Run the headless CLI | `cargo run -p warlock-cli -- --help` |
| Tests | `cargo test --workspace` |
| Lint (CI-equivalent) | `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings` |
| WASM check | `cargo check -p warlock -p warlock-core --target wasm32-unknown-unknown` |
| Dependency audit | `cargo deny check` |
| Screenshot smoke test | `cargo xtask smoke` (writes `target/xtask/smoke.png`) |
| Web build | `cargo xtask web` (writes `target/web/`; see "Releases, web build, logs") |
| Web smoke test | `cargo xtask web-smoke <chrome>` (serves `target/web`, drives headless Chromium through menu → New run → hub, waits five turns, reloads, Continue, and checks the run resumed at turn 5; writes `target/xtask/web-smoke.png`; fails on logged errors; not run in CI) |
| Regenerate the vast level | `cargo xtask gen-vast` (writes `assets/levels/vast.toml`; deterministic, never hand-edit) |
| Check the vast level is fresh | `cargo xtask gen-vast --check` (fails if the file differs from the generator output; run it after touching `xtask/src/vast.rs`) |
| Third-party notices | `cargo xtask licenses` (regenerates `THIRD_PARTY_LICENSES.html` with `cargo-about`, config `about.toml`/`about.hbs`; install: `cargo install cargo-about --locked --features cli`); `--check` fails when it is stale (CI runs it). **Run it after changing dependencies** (`Cargo.toml`/`Cargo.lock`); keep `about.toml`'s `accepted` list in sync with `deny.toml`'s `allow` |
| Player wiki tables | `cargo xtask wiki` (writes `wiki/src/generated/*.md` from the game data), `--check` (fails when stale), `--build` (also runs `mdbook build wiki` into `wiki/book/`; needs `cargo install mdbook --locked`) |
| Golden screenshots | `cargo xtask golden` (compare; authoritative on Linux only), `--local` (capture to `target/xtask/golden/*.local.png`, no compare), `--update` (rewrite references; see "Golden screenshots") |
| Frame-time benchmark | `cargo xtask bench-frames [WxH]` (release, offscreen; method and results in `docs/perf.md`) |
| Turn-time benchmark | `cargo xtask bench-turns` (re-runs itself in release; budgets: hub 5 ms, vast 16 ms, generated depths level 5 ms, sunlit courtyard at dawn 5 ms, overworld 5 ms, generated forest area at dawn 5 ms, courtyard and forest area at full-moon midnight 5 ms per turn; a dusk-to-dawn rest 500 ms) |
| Light stress benchmark | `cargo test --release -p warlock-core turn_cost -- --ignored --nocapture` (400×300, 825 lights; again outdoors at 06:15) |

Game flags (`cargo run -p warlock -- --help`):
- `--scenario NAME` (default `showcase`; `depths` starts in the generated dungeon, `courtyard` in
  the sunlit courtyard at 06:30, `world` on the overworld beside the Hall, `forest` in the glade
  of a generated forest area of the overworld at 06:30), `--seed S`
  (default: the scenario's seed; a number, or a word hashed with FNV-1a, e.g. `--seed warlock`)
- `--content-dir assets` — read content from disk instead of the copy built into the binary
- `--window WxH` (physical pixels), `--scale F` (display scale; `1` for reproducible captures)
- `--capture out.png` — render **offscreen** (no window), save a screenshot, exit.
  `--capture-after-frames N` (default 10), `--freeze-time SECONDS` pins animation time (glyph
  animations show that moment; slides and camera moves complete instantly).
- `--reduced-effects` — force reduced effects on (no glyph animation, light flicker, slides,
  camera easing, particles, bloom/glow, heat haze, or fades); always wins over the settings
  (`Settings::effective_effects(force)`). With `--freeze-time`, particle bursts from the last turn's events (a `--replay`'s last step)
  show a fixed moment of their life; fades complete instantly.
- Without `--scenario`, `--new`, `--replay`, `--record`, or `--capture` the game opens on the main
  menu. Dev: `--screen NAME` opens a screen at startup, even with `--capture`: `main` and
  `new-run` on the main menu (e.g. `--scale 1 --screen new-run --capture target/menu.png`);
  `pause`, `help`, `inventory`, `pick-up` (the pile on the player's cell), and `drop` over a run
  (e.g. after a `--replay` that walks onto a pile); `settings` and `controls` over the main menu, or over the pause
  menu when `--scenario`, `--new`, `--replay`, or `--record` names a run (`--capture` alone does
  not).
  `--config-dir DIR` keeps `settings.toml` in DIR; captures and replays otherwise use default
  settings and never write the file.

- `--replay FILE` — apply a replay at startup (its scenario/seed win; a hash divergence is logged
  as a warning), so `--capture` can screenshot any state. `--record FILE` — write every applied
  action (with state hashes) to a replay, rewritten after each action.

Headless dev loop (`warlock-cli`, no window; `--content-dir` works here too):

| Task | Command |
|---|---|
| ASCII dump after actions | `cargo run -p warlock-cli -- dump --scenario showcase --actions "move:N move:N" --layers view,fov,memory` |
| Dump a replay's end state | `... dump --replay runs/x.json [--actions "wait"] --format ascii\|json [--state]` |
| Create a replay | `... replay --create --scenario showcase --actions "move:N wait" -o runs/x.json` |
| Verify a replay | `... replay --verify runs/x.json` (exit 1 and first diverging step on mismatch) |
| Preview a generated level | `... gen --level depths:3 --seed warlock` (ground truth; `*` = light placed on floor; also local areas, `--level local:49,23`) |

Action text syntax: whitespace-separated `move:DIR wait open:DIR close:DIR ascend descend lantern
pickup[:LETTERS] drop[:LETTERS] rest:GOAL vision:PROFILE skip:DURATION` (`DIR` = N NE E SE S SW W
NW; `LETTERS` = stacks by letter, `a` first, e.g. `pickup:ac` (the pile on the player's cell, `a`
its top) or `drop:b` (the inventory), none = everything; `PROFILE` = an id in
`assets/data/vision.toml`; `GOAL` = `dawn`, `dusk`, or a duration up to `24h`; durations are
`30s`, `10m`, `2h`; `skip` is the dev hour skip). Dump layers: `view` (what the player sees), `fov` (`*` visible),
`memory` (seen terrain and remembered objects), `light` (perceived brightness of visible cells as a
digit, `0` dark to `9` fully lit), `sound` (heard sounds at their rough places over the view: `f`
footsteps, `o`/`c` a door opening/closing, upper case when loud), `inventory` (what the player
carries and weighs), `truth`
(ground truth, **dev-only**), `actors` (every actor on the level with its weight and what it
carries, **dev-only**). Replay files are JSON
`{game_version, scenario, seed, steps:[{action, hash}]}`; `hash` is the FNV-1a 64 state hash
(`World::state_hash`) after the step, as 16 hex digits.

- `--new` start a fresh run instead of resuming the save; `--dev` enables the dev keys in release
  builds (always on in debug: F2 reveal map (ground truth), F3 next vision profile, F4 light-value
  overlay, F5/F9 snapshots, F6 skip an hour (recorded as `skip:1h`); all listed by `?`);
  `--save-dir DIR` keeps saves in DIR instead of the platform data dir.

Storage backends (`crates/warlock/src/storage.rs`): `Backend::Dir` (native files, atomic writes) or
`Backend::Browser` (wasm `localStorage`, keys `warlock/<name>`, e.g. `warlock/run.sav`,
`warlock/run.sav.bad`, `warlock/snapshots/quick.sav`, `warlock/settings.toml`; bytes base64).
`SaveStore` and `SettingsStore` hold an `Option<Backend>`; new persistent files go through it, never
`std::fs` directly. On the web the page-hide save cannot ask the world for a save (a hidden tab runs
no frames), so `save.rs::web_hooks` keeps a clone of the state after every turn (encoding is
milliseconds on the vast level) and the `visibilitychange`/`pagehide` listeners encode and write it. Unavailable storage means no saving plus a log
warning; failed writes (quota) are messages, never panics. Only the `web-sys` features in
`crates/warlock/Cargo.toml` are enabled (size).

Saves: `Q` saves and quits (closing the window saves too); the game autosaves every 50 turns and on
level change to `run.sav` in the platform data dir (`paths.rs`, app name "warlock"; macOS
`~/Library/Application Support/warlock`). On startup an existing save is resumed unless `--new`, `--capture`,
`--replay`, or `--record` (replays and recordings always start a fresh run). F5 / F9 write / load `snapshots/quick.sav`. An unreadable save is renamed `run.sav.bad`,
never deleted. Format and `Content::fingerprint` live in `warlock-core/src/save.rs`: bump
`FORMAT_VERSION` on any incompatible `WorldState` change; adding or reordering terrain, creatures,
lights, levels, objects, or dungeon branches invalidates existing saves.

Settings (`crates/warlock/src/settings/`): `settings.toml` in the platform config dir (macOS
`~/Library/Application Support/warlock`), schema in the `settings/mod.rs` module docs. Changes
apply live and are written at once; a bad file means defaults plus a warning, per value. Effects:
`reduced_effects` is a preset over the `[effects]` switches (`EffectSwitches`: glyph_animation,
flicker, motion, glow, distortion, particles, screen_fades; all on by default). Each consumer reads
its own switch from the `render::Effects` resource (the effective values, synced every frame from
`Settings` + `--reduced-effects`), never the preset; a new effect needs a switch in
`EffectSwitches`, the file, and a row in `ui/settings_screen.rs`. Palettes
(`settings/palette.rs`) are applied only when cells are uploaded to the GPU (`render::upload`) and
in `ui/glyphs.rs`, never to content colors. Rebindable commands have stable ids in
`settings/keys.rs` (`KeyCommand::id`): **a new `KeyCommand` variant must be added there**
(`REBINDABLE` or the non-rebindable arm). `Game` exists only during a run (the menu runs without
it): gate systems that read it with `run_if(resource_exists::<Game>)`; content is the
`GameContent` resource.

Modal screens (spec §6) — main menu, new run, pause, settings, controls, the `?` help, the
inventory (`i`), and choosing what to pick up (`,`/`g` on several stacks) or drop (`d`) — are one
screen stack (`ui/screens.rs`, the `ScreenStack` resource); add a screen there, never a second way
to open, read keys for, or draw one:
- **Open**: `Screens::open(ScreenName)` (a `SystemParam` over the stack, content, save store, and
  the run, if any) is the one way a screen opens: `?`, `i`, `,`, `d`, and Esc in play,
  sub-screens, the startup main menu and `--screen`, and the return to the main menu after a run.
  Screens that show the run read the perception snapshot when they open (no time passes while
  they are open). Back/Esc pops the top screen, so a sub-screen returns to the screen it was
  opened from as it was left.
- **Input**: `input::route_keys` is the one reader of `KeyboardInput`. Stack not empty: the press
  goes to the top screen (`ScreenStack::handle` → `ScreenAction`: stay, close, open, a
  `MenuChoice`, or `Act(Action)`: close and apply a gameplay action, e.g. `pickup:ac`); otherwise
  it is a gameplay command in the current `InputMode`. Opens and closes apply at once, so the next
  press in the same frame goes to the new top; menu choices (start, resume, save-and-quit to menu,
  quit) are messages applied by `screens::apply_choice`, and the rest of that frame's presses are
  dropped. Held-key repeats never open, close, choose, or act.
- **Draw**: `ui::draw` is the one system that writes the UI console: when anything it shows
  changed, it clears the console, draws the gameplay UI if a run exists, then the top screen.
- Prompts in the message line (open/close direction, rest, look mode) are `InputMode`s, not
  screens.
- Screens are plain state machines tested without a window: `ui/menu.rs` (`Press`, `Panel`, main
  and pause menu), `ui/new_run_screen.rs`, `ui/settings_screen.rs`, `ui/controls_screen.rs` (also
  the `?` help: the same binding list, read-only), `ui/object_screen.rs` (inventory, pick up,
  drop: stacks by letter; in the choices letters always name stacks, even `h j k l`).

Typical render check: `cargo run -p warlock -- --scale 1 --freeze-time 1 --capture target/shot.png`,
then read the PNG.

## Architecture (spec §8)

```
crates/warlock-core   plain Rust, no Bevy, deterministic simulation
crates/warlock-cli    headless dev tool on top of core (no Bevy)
crates/warlock        Bevy app: rendering, input, UI, effects, platform
xtask                 screenshot capture, golden-image comparison, vast level generator, benchmarks
assets                fonts, data definitions, levels, scenarios, vaults
```

Content (`assets/data`, `assets/levels`, `assets/scenarios`, `assets/vaults`) is TOML, embedded into
`warlock-core` at build time by `build.rs`. Level maps are ASCII with a legend; see
`assets/levels/hub.toml` and spec §7. Water: `~` is still water; running water (terrain with a
`flow` anim) needs a flow on every cell from its legend entry (`flow = "S"`, optional
`flow_speed` percent), and the shared legend draws it as arrows (`→` normal, `⇒` double speed).
Flow is presentation data carried by `LevelTemplate::flow` / `LevelState::flow` and read by the
renderer through `PerceptionView::flow` (visible cells only); no rule reads it.

Time (spec §8 "Time") lives in `warlock-core`: `time.rs` (`GameTime` = seconds since the
calendar epoch with calendar fields, minute of day, and 16-bit day fraction; `Duration`; 12×30-day
calendar math; `DAWN`/`DUSK`), `content/calendar.rs` (month names from `data/calendar.toml`), and
the turn pipeline in `world/mod.rs`: `player_acts` returns `Spent::Turn(duration)` (the level's
turn, or `LevelState::step_time` for a step — the one place movement time is decided: `move_cost`
on local levels, `travel_minutes` on the overworld), `run_turn` (the energy scheduler,
`world/schedule.rs`, spec §8 "Speed") charges the player for it and runs the other actors on the
level in time order until the player may act again, advancing the clock to each action, and
`rest` loops turns until the goal or an interruption (perceiving only after turns in which a
creature could have come into view: `out_of_view_for`, which accounts for speed). New actions
that take time must return their duration there. `creature_turn` is one creature's action and
returns how long it took: the one place creature behavior lives (AI decisions, the noise and
events an action makes). Speed (`speed` in `data/creatures.toml`, default 1000 = the player's)
is rules-only: never put it in perception, the UI, or messages; energy is per actor in
`WorldState` (saved, hashed); creatures on other levels and the dev skip gain none.
Levels carry `kind` and `turn` (`turn_seconds`); terrain carries `move_cost` (percent); scenarios
carry `start_date`/`start_time`. Rest (`R`) is a message-line input mode (`InputMode::Rest`,
`input::REST_CHOICES`), not a menu screen.

Hearing (spec §5 "Hearing") lives in `warlock-core/src/noise.rs`. `noise::noise(content, level,
actor, Act)` is the one place an action's loudness is decided (creature `footsteps` × terrain
`step_noise` percent for `Act::Step`, the door's `toggle_noise` for `Act::OpenDoor`/`CloseDoor`):
a new noisy action gets an `Act` variant there, and its call site in `world/mod.rs` is one
`self.make_noise(actor, Act::…)` where the action happens. `noise::make` delivers it at once to
every other actor on the level whose `hearing` threshold it reaches (`propagate`: integer Dijkstra,
10 a step, 14 diagonally, plus terrain `muffle`; solid terrain blocks unless it has a muffle),
pushing a `Heard { kind, at, loudness }` onto `Actor::heard`. A creature's list is what it heard
since it last acted and is cleared as it acts in `World::creature_turn` (the AI reads it there
before clearing); the player's is cleared in `World::run_turn` once the player has acted, and
becomes `PerceptionView::sounds()` (sources not visible only; kind, `Volume`, and `rough_place`,
never the actor; a walker's several steps within one turn are one sound). `World::hear` logs news
(`noise::news` against `WorldState::recent_sounds`) as messages, which stop a rest. The app draws
`PerceptionView::sounds()` in `ui/sounds.rs` (a still `◌` marker, always: it is information; the
ripple only with `glyph_animation`). Sounds carry at most `noise::MAX_REACH` (30) cells, under
`VIEW_RADIUS`, which keeps rest batching (`out_of_view_for`) exact (a const assert guards it).
Objects (spec §5, §6, §7; M11): every object in `data/objects.toml` defines its glyph, color,
material (`data/materials.toml`), and weight in grams (`content/object.rs`: `ObjectDef`,
`ObjectId`, and the `"id*N"` placement strings; `content/material.rs`); every creature has a base
`weight`. `object.rs` holds the runtime side: `Stack` (object + count; stackable objects join,
others are stacks of one), piles (`add_to_pile`: dropped stacks go on top, index 0 is the top the
map shows), inventories (`add_to_inventory`, at most `INVENTORY_SLOTS` = 26, lettered by index),
`Weight` (base + carried), `format_weight`, and `phrase` for messages. Piles live in
`LevelState::objects` (by position, never empty) with `object_memory` beside them (piles as last
seen, updated by `Perception::remember` on cells where creatures would be seen); inventories in
`Actor::inventory` (spawned from a level's `carry`). The rules know every actor's weight
(`Actor::weight`, `World::actor_weight`); presentation reads only `PerceptionView::objects`
(current pile in sight, else remembered), `inventory`, and `own_weight`: never what creatures
carry. `Action::PickUp(Letters)` / `Action::Drop(Letters)` (`Letters` = a set of `a`–`z`, empty
= everything) take the level's turn; a refused one takes none. The map grid draws the top object
in place of the terrain glyph (no terrain animation or glow there); look mode names piles.

Key commands `,`/`g` (pick up: one stack at once, else the `PickUp` screen), `d` (`Drop` screen),
`i` (`Inventory` screen) are in `input.rs` and `settings/keys.rs`.

The overworld (spec §3, §5, §7) is `assets/levels/overworld.toml`, the one level of kind
`overworld` (`LevelKind::Overworld`: 10-minute turns, steps cost the terrain's `travel_minutes`).
Its locations are stairs-down terrain (`hall`, `cave_mouth`, `dungeon_entrance`) whose legend `to`
names the level; the level's `<` with `to = "overworld"` leads back to the location (`World::travel`
arrives on the stairs leading back). `>` on any other cell enters the local area `local:x,y`
(`content/overworld.rs`: `local_area_id`, `parse_local_area`, `Content::local_area`, valid where
the terrain has a `local` ground and a land or road `landscape`), generated on the first visit by
the outdoor generator (`mapgen/local/`, below); the edge exit (`exit` in `LevelState`) makes
walking off any edge return to the cell. `>` arrives on the side the player walked onto the cell
from: `WorldState::overworld_step` (the last overworld step, cleared by any level change) picks
the side and `mapgen::local::arrival` the edge cell (road first, then dry, open ground near the
middle of the side); without a step, at the level's start (the middle). Sight on the overworld is line
of sight within `daylight::overworld_sight` (day 9, twilight 5, night 2 plus one per
`MOONLIGHT_PER_CELL` of `Sky::moonlight`, at most `SIGHT_MOONLIT` 4), ignoring light. The
overworld is sky-exposed (sun, moons, and sky, like outdoor levels); `daylight::overworld_night`
tops its ambient light up to the faint `OVERWORLD_NIGHT` where the sun's sky falls short
(moonlight comes on top), so cells in sight stand apart from memory at night (local areas get
none). Regions (`[[region]]` in level
files, `LevelTemplate::region_at`, `PerceptionView::region`) show in the status bar. Unlike
`vast.toml`, the overworld map is hand-authored: edit the file directly (keep every walkable cell
inside a region; a test checks).

Procedural generation (spec §7, §8) lives in `warlock-core`: `rng.rs` (ChaCha8 streams derived
from the run seed: `map/<level id>` per generated level, `gameplay` kept in `WorldState`;
`parse_seed` for `--seed`), `content/dungeon.rs` (branches in `data/dungeons.toml`; level ids
`depths:N`), `content/vault.rs` (vault rules), and `mapgen/` (`rooms.rs`, `caves.rs`, `vault.rs`
stamping, `connect.rs` flood fill / corridor digging / validation). A branch level is generated by
`World::travel` on its first visit and then persists in `LevelState` like a hand-made level (level
states carry their own ambient and placed lights). Generation is integer-only; a level that fails
validation is regenerated from the next derived stream. The insta snapshots in
`src/mapgen/snapshots/` and `src/mapgen/local/snapshots/` flag any change in generator output:
review the maps in the diff, then accept them deliberately.

Local areas (spec §3, §7) are generated in `mapgen/local/`: `mod.rs` (`Setting`: the cell's
landscape and its neighbors' roads, rivers, lakes, and land, read from the overworld; per-landscape
feature chances; the attempt loop, glade, ruins, deer), `kit.rs` (the terrain and creature ids it
builds with, checked when content with an overworld loads), `noise.rs` (integer value noise and
quantiles: "this share of the area"), `land.rs` (the canvas with per-cell `Claim`s: `Fixed` water,
roads, ruins; `Clear` glade, banks, roadsides, no obstacles; `Free`), `water.rs` (river and lake
bands, ponds, brooks, winding courses), `road.rs`, `cover.rs` (one function per landscape; wood
share blends towards the neighbors), and `reach.rs` (trails, joining pockets, ways out,
validation, `arrival`). Overworld terrain picks its generator with `landscape` in
`data/terrain.toml`; outdoor vaults are vault files with `kind = "outdoor"`. Tune by eye:
`WARLOCK_AREAS="25,20 60,44" cargo test -p warlock-core --lib print_areas -- --ignored
--nocapture` prints areas with one character per terrain (`WARLOCK_SEED` for the seed), and the
`forest` scenario or `--content-dir` scenarios with `start_level = "local:x,y"` capture them.

Rendering (spec §4.2) is three layers, bottom to top, all positioned in framebuffer pixels:
1. **Map grid** (`render/grid.rs`): a `Console` (`render/console.rs`) drawn by one quad with the
   console `Material2d` (`render/console.wesl`). Each fragment finds its cell from its framebuffer
   position and reads glyph coverage from the 1:1 font atlas (`render/atlas.rs`) with integer
   texel addressing, so still glyphs are pixel-exact. The grid is one cell larger than the map
   viewport and drawn behind a clip rect with a whole-pixel sub-cell offset (smooth camera). A
   per-cell `Rgba8Uint` texture (table entry, phase byte) drives in-cell animation (`wave`,
   `flow`, `shimmer`, `cycle`) evaluated in the shader from a uniform table (`render/anim.rs`).
   Water (`render/water.rs`): running water's flow (direction and speed, `water::flow_byte`)
   rides in the glyph texture's spare third byte, and the shader scrolls the glyph pattern
   downstream (cells with the same flow line up; a direction or speed change along a stream shows
   a seam). Glints are hashed from the level position (`ConsoleParams::level_origin`), so they
   stay put while the camera scrolls.
2. **Free-glyph layer** (`render/free_glyph.rs` + `.wesl`): producers push `FreeGlyph`s
   (pixel position, color/alpha, scale) each frame; they become one dynamic mesh, one draw call.
   Glyphs at rest read the 1:1 atlas; moving or scaled ones sample a supersampled (2×, linear)
   atlas whose slots are exactly 2× the 1:1 slots. Creatures, particles, and `pulse` terrain live
   here.
3. **UI console**: the same console material over the whole window; unwritten cells are
   transparent.

**Visual lighting** (spec §4.4, `render/lighting.rs` + `lighting.wesl`, imported by
`console.wesl`): every frame `lighting::update` composites the perceived light for the map grid's
cells plus a one-cell margin, from `PerceptionView::base_light` (ambient plus gameplay bounce light,
steady, never shadowed by creatures) and `light_sources()` contributions, less creature shadows,
and folds in UV (violet tint) and the darkvision fade. Gameplay light lives in
`warlock-core/src/light/` (`mod.rs`: falloff and the wall-face rule; `cache.rs`: per-source shadow
masks; `bounce.rs`: one weak bounce; `sun.rs`: sunlight; `moon.rs`: moonlight and `Sky`) and is
combined in `perception.rs`;
Σ contributions + base light == `light(p)` is a tested invariant. **Creature shadows**
(`render/shadows.rs`, visual only): each visible creature shades every perceived source that
lights its cell, except one on its own cell (its carried light). The source is shadowcast
(`fov::compute_fov`) over the perceived terrain with and without the creature's cell as an extra
occluder; contribution cells only the first reaches keep `shadows::keep` of that source's light
(`KEEP` 0.4 right behind the creature, back to all of it at `REACH` 3.5 cells), so colored shadows
come from per-source attenuation. The sun and the moons are directional: `shadows::sky_shade`
walks `Emitter::shadow_steps(CREATURE_HEIGHT)` from the creature instead. `shadows::update` recomputes the `Shadows` resource only when
`Game` changes (once per turn), not per frame; darkvision still measures the unshaded light (else
its glow would brighten shadows). **Light stability** (`render/modulation.rs`; presets'
`stability.{intensity,hue,dips}` in `lights.toml`, absent = steady): each source's `Modulation`
is a pure function of its key, preset, and `VisualTime` (steady with the `flicker` effect off).
Its hue `tint` multiplies the source's contributions on the CPU (exact; drawn light =
Σ tinted contributions · shadow keep + base light, while darkvision still measures the perceived
light); its intensity `gain` becomes the per-cell brightness gain (drawn-light-weighted mean,
base light = 1) applied after shading. Tune with captures at several `--freeze-time` values;
the crystals, fungus, runes, and altar must stay constant. Two textures on the map material:
`light` (`Rgba32Float`: rgb light, 1.0 = fully lit, and saturation) and `light_flags`
(`Rgba8Uint`: lit/opaque flags, intensity gain byte). The shader
interpolates light per pixel between cell centers, never across opaque, unseen, or remembered
cells (walls are lit flat by their wall-face light); then exposure (readability floor, soft
overbright knee plus a faint glow in the light's hue), desaturation, gain. Unlit cells (remembered
terrain, the UI console's 1×1 textures) are drawn as they are. Free glyphs are lit on the CPU by
`VisualLight::sample` + `lighting::shade`, which mirror the shader — change both together.

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

Effects (spec §4.8), each off with its own switch (`particles`, `glow`, `distortion`,
`screen_fades`) and all off with reduced effects:
- **Particles** (`render/particles.rs`): stateless, a pure function of visual time and a seed via
  a visual-only SplitMix RNG (ambient emitters: one particle per `1/rate` slot; bursts: `count`
  particles per event). `ui/effects.rs` emits them from perception (visible cells, seen creatures,
  `Outcome.events`) onto the free-glyph layer, below creatures; non-glowing ones are lit like
  free glyphs (`VisualLight::sample` + `lighting::shade`). `obstacle` emitters (foam) run like
  ambient ones on visible running water whose flow runs into a known cell that is not running
  water, turned so their `y` points downstream (`particles::Source::down`).
- **Glow and heat haze** run in `console.wesl` from two spare bytes of the per-cell animation
  texture (`CellFx`, written by `ui/effects.rs::draw_cell_fx`): haze moves where glyph coverage is
  sampled (backgrounds and light stay put); after lighting, colors are clamped to 1 (overbright
  light alone never blooms) and glowing glyphs are multiplied past 1. Free glyphs do the same
  through their HDR vertex color (`post::in_range`, `post::emissive`).
- **Post** (`render/post.rs` + `tint.wesl`): the camera gets `Hdr` + `Bloom` (threshold 1,
  additive, `Tonemapping::None`), so only glowing glyphs bloom (with `glow` off the camera stays plain LDR,
  pixel-identical to no effects). `ScreenTints` draws a fading color over the map viewport:
  the level-change fade, and `flash()` for hit-style events.

`ui/` decides what is drawn from the perception snapshot: `ui/mod.rs` (UI console, map grid in
base colors), `ui/glyphs.rs` (free glyphs), `ui/motion.rs` (slides from `Outcome.events`, one per
creature along all the steps it took that turn; camera easing; new input completes running
motion), `ui/effects.rs` (particles, bursts, glow and haze, fades), `ui/sounds.rs` (heard-sound
markers), `ui/look.rs` (look mode), `ui/dev.rs` (dev overlays: reveal map, the one reader of
ground truth, and light values).
Animation curves live in `render/anim.rs` (water: `render/water.rs`, test-only mirrors of the
shader) and are mirrored in `console.wesl` — change both together. Everything animated is a function of `VisualTime`. Effects are deliberately subtle (the
bar: sparse, short-lived particles, gentle bloom, barely perceptible distortion, short fades); judge
changes by capturing several `--freeze-time` values, at `--scale 1` and `--window 2560x1440
--scale 2`.

Hard rules:
- `warlock-core` never depends on Bevy, never reads wall-clock time, and never does platform I/O
  inside the rules. Iteration order in core must be deterministic (no `HashMap` iteration).
- Rendering, UI, and dumps read the per-turn **perception snapshot** only — never ground truth —
  so hidden information cannot leak. The only exception is an explicit dev "reveal" mode.
- Gameplay outcomes never depend on frame rate, animation, or visual randomness.
- Keep modules small with clear responsibilities. No speculative abstractions, no premature
  optimization.

## Wiki

`wiki/` is the player wiki: Markdown pages for players (rules and world facts, not developer
notes, and no locations or level walkthroughs yet), built with [mdBook](https://rust-lang.github.io/mdBook/)
(`wiki/book.toml`, table of contents in `wiki/src/SUMMARY.md`, default theme `coal`). It is
published at https://jannef.github.io/warlock/wiki/ by the release workflow's web job, which
builds it into `pages/wiki/` beside the game. Facts that live in data (months, moons, terrain,
footstep loudness in words, vision profiles, light presets, creatures, objects, materials, sight
radii, the pack's size; never hidden rules such as speed, hearing thresholds, or creature weights)
are tables generated by `cargo xtask wiki`
(`xtask/src/wiki.rs`) into `wiki/src/generated/` (committed) and pulled into pages with
`{{#include generated/NAME.md}}`; CI runs `cargo xtask wiki --check` and `mdbook build wiki` on the
Linux job (mdBook pinned there and in `release.yml`). To preview: `cargo xtask wiki --build`, then
open `wiki/book/index.html`.

**A PR that changes gameplay mechanics or world facts updates the wiki in the same PR**, like the
spec and this file: edit the page, and generate a table in `xtask/src/wiki.rs` if the fact lives
in data. Run `cargo xtask wiki` after changing content.

## Bevy 0.20 — pinned references

Bevy 0.19 (2026-06) and 0.20 (2026-10) are newer than most model training data. **Do not write
Bevy code from memory; check the pinned sources.**

- API docs: https://docs.rs/bevy/0.20.0/bevy/
- Examples at the pinned tag: https://github.com/bevyengine/bevy/tree/v0.20.0/examples
- Release notes: https://bevy.org/news/bevy-0-20/ and https://bevy.org/news/bevy-0-19/
- Migration guides: https://bevy.org/learn/migration-guides/
- Fastest: a local shallow clone to grep —
  `git clone --depth 1 --branch v0.20.0 https://github.com/bevyengine/bevy /tmp/bevy-0.20`
  (relevant: `examples/shader/shader_material_2d.rs`, `examples/2d/`, `examples/window/screenshot.rs`,
  `crates/bevy_sprite_render/src/mesh2d/`).

Known differences from older Bevy:
- Shaders are **WESL**: `import bevy_sprite_render::mesh2d::vertex_output::VertexOutput;`,
  conditional code via `@if(FLAG)`, material bind group via `@group(constants::MATERIAL_BIND_GROUP)`.
  The old `#import`/`#ifdef` dialect is gone. Embed shaders with `embedded_asset!` and reference
  them with `AssetPath::from_path_buf(embedded_path!("x.wesl")).with_source("embedded")`.
- `Material2d`, `Mesh2d`, `MeshMaterial2d`, `AlphaMode2d` live in `bevy::sprite_render`.
- Buffered events are **messages**: `MessageWriter<AppExit>`, `MessageReader<T>`; observers take
  `On<E>`.
- Bevy features are trimmed in the workspace `Cargo.toml` (`default-features = false`). Adding a
  Bevy feature is a deliberate change. `tonemapping_luts` is deliberately off (it pulls `ruzstd`
  0.8, RUSTSEC-2026-0344); cameras use `Tonemapping::None`, which also keeps palette colors exact.

## Verification

A successful compile does not verify anything visual. Before opening a PR:
1. `cargo fmt`, clippy with `-D warnings`, `cargo test --workspace`.
2. Simulation changes: add or update unit, property, or snapshot tests in `warlock-core`
   (`insta` snapshots live in `crates/warlock-core/src/snapshots/`, `src/mapgen/snapshots/`, and
   `src/mapgen/local/snapshots/`; review with `cargo insta review` or `INSTA_UPDATE=always`, and read the diff). Look at the
   result with `warlock-cli dump` (or `gen` for a generated level), and check determinism with
   `replay --create` then `replay --verify`.
3. Rendering changes: capture a screenshot with `--capture` and **look at it** (read the PNG).
   Golden images are updated deliberately, never to make a failing check pass.
4. Note in the PR what you verified and how.

## Workflow

- Work is tracked in GitHub issues under milestones (M0–M12, Housekeeping); use the `gh` CLI.
- One branch and PR per issue (or a tight group of related issues); reference issues with
  `Closes #N`. CI must be green before a squash merge into `main`.
- CI runs on Linux only (`CI` workflow: fmt, clippy, tests, headless screenshots, wasm32 check,
  cargo-deny; the web build on pushes to `main`). macOS is not run in Actions: development happens
  on macOS, so run the full local check there before every PR. `main` is branch-protected: PRs
  only, the Linux, wasm32, and cargo-deny checks must pass, linear history.

## Golden screenshots

`tests/golden/manifest.toml` lists one entry per showcase room plus the vast level (its start and
its river): a scenario and text `actions` (`move:N*3 open:E`; `*N` repeats) or a `replay` file,
and optional `window`, `scale`, `freeze_time`, `screen` (`--screen` over the run: `pause`,
`help`, `settings`, `controls`, `inventory`, `pick-up`, `drop`), `tolerance` (per-channel, default 2) and `max_diff_ratio`
(default 0.05% of pixels).
`cargo xtask golden` builds a replay per entry with `warlock-core`, runs the game with
`--replay ... --capture ...`, and compares with `tests/golden/<name>.png`. On a mismatch it writes
`target/xtask/golden/<name>.{actual,diff}.png` (diff: red pixels are over tolerance) and fails.

References are produced on **Linux CI (Mesa lavapipe) only**; other GPUs and drivers rasterize
differently, so on macOS `cargo xtask golden` is advisory and `--local` is the way to look at the
captures. CI compares and never updates.

Deliberate update procedure (after an intended visual change, e.g. lighting or effects landing):
1. Push the branch, then Actions > CI > Run workflow on that branch with "update_golden" ticked
   (`gh workflow run ci.yml --ref <branch> -f update_golden=true`, then `gh run watch`).
2. `gh run download <run-id> -n golden-references -D tests/golden`, review the images (look at them,
   and check the diff against the old ones makes sense), and commit them with the change.
3. Adding a room or scenario: add a manifest entry and generate its reference the same way. Never
   update references just to make a failing check pass.

## Releases, web build, logs

**Cut a release.** Make sure `main` is green, then tag and push the tag (for example
`v0.1.0`); the `Release` workflow does the rest. It builds Linux x86_64 (`.tar.gz`), Windows
x86_64 (`.zip`), macOS arm64 (`.zip` with an unsigned `Warlock.app` plus the bare binary; see
`release/`) and the web build (`.zip` to serve from any static host) and attaches them, with
generated notes, to a GitHub Release; `v0.x` tags are published as pre-releases. Run the workflow
by hand (Actions > Release > Run workflow) to get the archives as workflow artifacts without
publishing. Releases are the only place macOS and Windows runners are used.

**Web build** (wasm32 + WebGPU; all content is embedded, nothing on the startup path may touch
`std::fs`): one-time setup is `rustup target add wasm32-unknown-unknown` and
`cargo install wasm-bindgen-cli --version <X> --locked`, where `<X>` is the `wasm-bindgen` version
in `Cargo.lock` (the CLI must match exactly; `cargo xtask web` checks it and prints the command).
Then `cargo xtask web` builds `target/web/` with the size-tuned `[profile.web]` (opt-level `s`, fat LTO,
1 codegen unit, `panic = "abort"`) and then runs binaryen's `wasm-opt -Oz` (`brew install binaryen`;
locally optional with a warning, required on CI, which installs a pinned binaryen 133 release — not Ubuntu's older apt package, whose `wasm-opt` exported the wrong table as `__wbindgen_externrefs` so the page could not start; `cargo xtask web` now checks that table, `xtask/src/wasm_check.rs`). `multi_threaded` is a
native-only Bevy feature. Sizes (`warlock_bg.wasm`): 45.9 MB raw / 10.1 MB gzip / 5.9 MB brotli
before, 20.8 MB / 6.8 MB / 4.6 MB after. Do not give wasm-opt `-all`: it emits compact imports
that Chromium rejects. After changing the web build run `cargo xtask web-smoke` and read the
screenshot (the Playwright Chromium under `~/Library/Caches/ms-playwright` works with
`--use-angle=metal`). The output is `target/web/` (`index.html` from `web/`, `warlock.js`,
`warlock_bg.wasm`). Serve it with `python3 -m http.server --directory target/web 8080` and open
http://localhost:8080 in a WebGPU browser. CI builds it on pushes to `main` (artifact
`warlock-web`). There is no command line on the web, so `cli.rs` uses the defaults.

**GitHub Pages** (https://jannef.github.io/warlock/): the `web` job of the `Release` workflow also
uploads `target/web` plus `LICENSE` as the Pages artifact, and the `pages` job (`environment:
github-pages`) deploys it on every `v*` tag; a manual run deploys only with the `publish_pages`
input ticked. The page is `web/index.html` (inline, no external requests, relative asset paths so
it works under `/warlock/`): top bar with links, the canvas in `#stage` (Bevy's
`fit_canvas_to_parent` sizes it to that container), fullscreen on `#stage`, a hint footer.
`cargo xtask web` replaces `@VERSION@` in the page with the workspace version (as
`release/Info.plist.in` does in the macOS bundle). Keep the canvas id `#warlock-canvas` (the window
plugin selects it).

**Logs and crash reports** (desktop only): each run writes `warlock-<UTC>.log` (last 5 kept) in
`<data dir>/warlock/logs` (macOS `~/Library/Application Support/warlock/logs`, Linux
`~/.local/share/warlock/logs`, Windows `%LOCALAPPDATA%\warlock\data\logs`). A panic writes
`crash-<UTC>.txt` (version, OS, backtrace, last log lines; last 10 kept) next to them and prints
its path. `WARLOCK_CRASH_TEST=1 cargo run -p warlock` triggers a deliberate panic to test this.
Code: `crates/warlock/src/logging.rs`.
