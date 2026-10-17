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
| Web smoke test | `cargo xtask web-smoke <chrome>` (serves `target/web`, drives headless Chromium through menu → New run → hub, writes `target/xtask/web-smoke.png`; fails on logged errors) |
| Regenerate the vast level | `cargo xtask gen-vast` (writes `assets/levels/vast.toml`; deterministic, never hand-edit) |
| Check the vast level is fresh | `cargo xtask gen-vast --check` (fails if the file differs from the generator output; run it after touching `xtask/src/vast.rs`) |
| Golden screenshots | `cargo xtask golden` (compare; authoritative on Linux only), `--local` (capture to `target/xtask/golden/*.local.png`, no compare), `--update` (rewrite references; see "Golden screenshots") |
| Frame-time benchmark | `cargo xtask bench-frames [WxH]` (release, offscreen; method and results in `docs/perf.md`) |
| Turn-time benchmark | `cargo xtask bench-turns` (re-runs itself in release; budgets: hub 5 ms, vast 16 ms, generated depths level 5 ms per turn; a dusk-to-dawn rest 500 ms) |
| Light stress benchmark | `cargo test --release -p warlock-core turn_cost -- --ignored --nocapture` (400×300, 825 lights) |

Game flags (`cargo run -p warlock -- --help`):
- `--scenario NAME` (default `showcase`; `depths` starts in the generated dungeon), `--seed S`
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
  menu. Dev: `--screen main|new-run|settings|controls` opens the menu on a screen even with
  `--capture` (e.g. `--scale 1 --screen controls --capture target/menu.png`).
  `--screen pause` starts a game and opens the pause menu over it.
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
| Preview a generated level | `... gen --level depths:3 --seed warlock` (ground truth; `*` = light placed on floor) |

Action text syntax: whitespace-separated `move:DIR wait open:DIR close:DIR ascend descend lantern
rest:GOAL vision:PROFILE skip:DURATION` (`DIR` = N NE E SE S SW W NW; `PROFILE` = an id in
`assets/data/vision.toml`; `GOAL` = `dawn`, `dusk`, or a duration up to `24h`; durations are
`30s`, `10m`, `2h`; `skip` is the dev hour skip). Dump layers: `view` (what the player sees), `fov` (`*` visible),
`memory` (seen terrain), `light` (perceived brightness of visible cells as a digit, `0` dark to `9`
fully lit), `truth` (ground truth, **dev-only**). Replay files are JSON
`{game_version, scenario, seed, steps:[{action, hash}]}`; `hash` is the FNV-1a 64 state hash
(`World::state_hash`) after the step, as 16 hex digits.

- `--new` start a fresh run instead of resuming the save; `--dev` enables the dev keys in release
  builds (always on in debug: F2 reveal map (ground truth), F3 next vision profile, F4 light-value
  overlay, F5/F9 snapshots, F6 skip an hour (recorded as `skip:1h`); all listed by `?`);
  `--save-dir DIR` keeps saves in DIR instead of the platform data dir.

Saves: `Q` saves and quits (closing the window saves too); the game autosaves every 50 turns and on
level change to `run.sav` in the platform data dir (`paths.rs`, app name "warlock"; macOS
`~/Library/Application Support/warlock`). On startup an existing save is resumed unless `--new`, `--capture`,
`--replay`, or `--record` (replays and recordings always start a fresh run). F5 / F9 write / load `snapshots/quick.sav`. An unreadable save is renamed `run.sav.bad`,
never deleted. Format and `Content::fingerprint` live in `warlock-core/src/save.rs`: bump
`FORMAT_VERSION` on any incompatible `WorldState` change; adding or reordering terrain, creatures,
lights, levels, or dungeon branches invalidates existing saves.

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
(`REBINDABLE` or the non-rebindable arm). UI: `ui/menu.rs` (main menu state machine, `Panel`
helpers), `ui/settings_screen.rs`, `ui/controls_screen.rs`. `Game` exists only during a run (the
menu runs without it): gate systems that read it with `run_if(resource_exists::<Game>)`; content
is the `GameContent` resource.

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
`assets/levels/hub.toml` and spec §7.

Time (spec §8 "Time") lives in `warlock-core`: `time.rs` (`GameTime` = seconds since the
calendar epoch with calendar fields, minute of day, and 16-bit day fraction; `Duration`; 12×30-day
calendar math; `DAWN`/`DUSK`), `content/calendar.rs` (month names from `data/calendar.toml`), and
the turn pipeline in `world/mod.rs`: `player_acts` returns `Spent::Turn(duration)` (the level's
turn, or `LevelState::step_time` for a step — the one place movement time is decided, where the
overworld's travel times will plug in), `pass_time` advances the clock and lets creatures act once
per turn, and `rest` loops turns until the goal or an interruption (batching turns only while no
creature can come into view). New actions that take time must return their duration there.
Levels carry `kind` and `turn` (`turn_seconds`); terrain carries `move_cost` (percent); scenarios
carry `start_date`/`start_time`. Rest (`R`) is a message-line input mode (`InputMode::Rest`,
`input::REST_CHOICES`), not a menu screen.

Procedural generation (spec §7, §8) lives in `warlock-core`: `rng.rs` (ChaCha8 streams derived
from the run seed: `map/<level id>` per generated level, `gameplay` kept in `WorldState`;
`parse_seed` for `--seed`), `content/dungeon.rs` (branches in `data/dungeons.toml`; level ids
`depths:N`), `content/vault.rs` (vault rules), and `mapgen/` (`rooms.rs`, `caves.rs`, `vault.rs`
stamping, `connect.rs` flood fill / corridor digging / validation). A branch level is generated by
`World::travel` on its first visit and then persists in `LevelState` like a hand-made level (level
states carry their own ambient and placed lights). Generation is integer-only; a level that fails
validation is regenerated from the next derived stream. The insta snapshots in
`src/mapgen/snapshots/` flag any change in generator output: review the maps in the diff, then
accept them deliberately.

Rendering (spec §4.2) is three layers, bottom to top, all positioned in framebuffer pixels:
1. **Map grid** (`render/grid.rs`): a `Console` (`render/console.rs`) drawn by one quad with the
   console `Material2d` (`render/console.wesl`). Each fragment finds its cell from its framebuffer
   position and reads glyph coverage from the 1:1 font atlas (`render/atlas.rs`) with integer
   texel addressing, so still glyphs are pixel-exact. The grid is one cell larger than the map
   viewport and drawn behind a clip rect with a whole-pixel sub-cell offset (smooth camera). A
   per-cell `Rgba8Uint` texture (table entry, phase byte) drives in-cell animation (`wave`,
   `shimmer`, `cycle`) evaluated in the shader from a uniform table (`render/anim.rs`).
2. **Free-glyph layer** (`render/free_glyph.rs` + `.wesl`): producers push `FreeGlyph`s
   (pixel position, color/alpha, scale) each frame; they become one dynamic mesh, one draw call.
   Glyphs at rest read the 1:1 atlas; moving or scaled ones sample a supersampled (2×, linear)
   atlas whose slots are exactly 2× the 1:1 slots. Creatures, particles, and `pulse` terrain live
   here.
3. **UI console**: the same console material over the whole window; unwritten cells are
   transparent.

**Visual lighting** (spec §4.4, `render/lighting.rs` + `lighting.wesl`, imported by
`console.wesl`): every frame `lighting::update` composites the perceived light for the map grid's
cells plus a one-cell margin, from `PerceptionView::ambient` and `light_sources()` contributions,
less creature shadows, and folds in UV (violet tint) and the darkvision fade. **Creature shadows**
(`render/shadows.rs`, visual only): each visible creature shades every perceived source that
lights its cell, except one on its own cell (its carried light). The source is shadowcast
(`fov::compute_fov`) over the perceived terrain with and without the creature's cell as an extra
occluder; contribution cells only the first reaches keep `shadows::keep` of that source's light
(`KEEP` 0.4 right behind the creature, back to all of it at `REACH` 3.5 cells), so colored shadows
come from per-source attenuation. `shadows::update` recomputes the `Shadows` resource only when
`Game` changes (once per turn), not per frame; darkvision still measures the unshaded light (else
its glow would brighten shadows). Flicker (deterministic noise from the
source key and `VisualTime`; off with the `flicker` effect) is a per-cell brightness gain applied after
shading. Two textures on the map material: `light` (`Rgba32Float`: rgb light, 1.0 = fully lit,
and saturation) and `light_flags` (`Rgba8Uint`: lit/opaque flags, flicker gain byte). The shader
interpolates light per pixel between cell centers, never across opaque, unseen, or remembered
cells (walls are lit flat by their wall-face light); then exposure (readability floor, soft
overbright knee plus a faint glow in the light's hue), desaturation, gain. Unlit cells (remembered
terrain, the UI console's 1×1 textures) are drawn as they are. Free glyphs are lit on the CPU by
`VisualLight::sample` + `lighting::shade`, which mirror the shader — change both together.

Effects (spec §4.8), each off with its own switch (`particles`, `glow`, `distortion`,
`screen_fades`) and all off with reduced effects:
- **Particles** (`render/particles.rs`): stateless, a pure function of visual time and a seed via
  a visual-only SplitMix RNG (ambient emitters: one particle per `1/rate` slot; bursts: `count`
  particles per event). `ui/effects.rs` emits them from perception (visible cells, seen creatures,
  `Outcome.events`) onto the free-glyph layer, below creatures; non-glowing ones are lit like
  free glyphs (`VisualLight::sample` + `lighting::shade`).
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
base colors), `ui/glyphs.rs` (free glyphs), `ui/motion.rs` (slides from `Outcome.events`, camera
easing; new input completes running motion), `ui/effects.rs` (particles, bursts, glow and haze,
fades), `ui/dev.rs` (dev overlays: reveal map, the one reader of ground truth, and light values).
Animation curves live in `render/anim.rs` and are mirrored in `console.wesl` — change both
together. Everything animated is a function of `VisualTime`. Effects are deliberately subtle (the
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
   (`insta` snapshots live in `crates/warlock-core/src/snapshots/` and `src/mapgen/snapshots/`;
   review with `cargo insta review` or `INSTA_UPDATE=always`, and read the diff). Look at the
   result with `warlock-cli dump` (or `gen` for a generated level), and check determinism with
   `replay --create` then `replay --verify`.
3. Rendering changes: capture a screenshot with `--capture` and **look at it** (read the PNG).
   Golden images are updated deliberately, never to make a failing check pass.
4. Note in the PR what you verified and how.

## Workflow

- Work is tracked in GitHub issues under milestones M0–M7; use the `gh` CLI.
- One branch and PR per issue (or a tight group of related issues); reference issues with
  `Closes #N`. CI must be green before a squash merge into `main`.
- CI runs on Linux only (`CI` workflow: fmt, clippy, tests, headless screenshots, wasm32 check,
  cargo-deny; the web build on pushes to `main`). macOS is not run in Actions: development happens
  on macOS, so run the full local check there before every PR. `main` is branch-protected: PRs
  only, the Linux, wasm32, and cargo-deny checks must pass, linear history.

## Golden screenshots

`tests/golden/manifest.toml` lists one entry per showcase room plus the vast level: a scenario and
text `actions` (`move:N*3 open:E`; `*N` repeats) or a `replay` file, and optional `window`, `scale`,
`freeze_time`, `tolerance` (per-channel, default 2) and `max_diff_ratio` (default 0.05% of pixels).
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
locally optional with a warning, required on CI, which apt-installs it). `multi_threaded` is a
native-only Bevy feature. Sizes (`warlock_bg.wasm`): 45.9 MB raw / 10.1 MB gzip / 5.9 MB brotli
before, 20.8 MB / 6.8 MB / 4.6 MB after. Do not give wasm-opt `-all`: it emits compact imports
that Chromium rejects. After changing the web build run `cargo xtask web-smoke` and read the
screenshot (the Playwright Chromium under `~/Library/Caches/ms-playwright` works with
`--use-angle=metal`). The output is `target/web/` (`index.html` from `web/`, `warlock.js`,
`warlock_bg.wasm`). Serve it with `python3 -m http.server --directory target/web 8080` and open
http://localhost:8080 in a WebGPU browser. CI builds it on pushes to `main` (artifact
`warlock-web`). There is no command line on the web, so `cli.rs` uses the defaults.

**Logs and crash reports** (desktop only): each run writes `warlock-<UTC>.log` (last 5 kept) in
`<data dir>/warlock/logs` (macOS `~/Library/Application Support/warlock/logs`, Linux
`~/.local/share/warlock/logs`, Windows `%LOCALAPPDATA%\warlock\data\logs`). A panic writes
`crash-<UTC>.txt` (version, OS, backtrace, last log lines; last 10 kept) next to them and prints
its path. `WARLOCK_CRASH_TEST=1 cargo run -p warlock` triggers a deliberate panic to test this.
Code: `crates/warlock/src/logging.rs`.
