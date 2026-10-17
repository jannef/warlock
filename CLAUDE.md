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
| Regenerate the vast level | `cargo xtask gen-vast` (writes `assets/levels/vast.toml`; deterministic, never hand-edit) |
| Check the vast level is fresh | `cargo xtask gen-vast --check` (fails if the file differs from the generator output; run it after touching `xtask/src/vast.rs`) |
| Turn-time benchmark | `cargo xtask bench-turns` (re-runs itself in release; budgets: hub 5 ms, vast 16 ms) |
| Light stress benchmark | `cargo test --release -p warlock-core turn_cost -- --ignored --nocapture` (400×300, 825 lights) |

Game flags (`cargo run -p warlock -- --help`):
- `--scenario NAME` (default `showcase`), `--seed N` (default: the scenario's seed)
- `--content-dir assets` — read content from disk instead of the copy built into the binary
- `--window WxH` (physical pixels), `--scale F` (display scale; `1` for reproducible captures)
- `--capture out.png` — render **offscreen** (no window), save a screenshot, exit.
  `--capture-after-frames N` (default 10), `--freeze-time SECONDS` pins animation time (glyph
  animations show that moment; slides and camera moves complete instantly).
- `--reduced-effects` — force reduced effects on (no glyph animation, light flicker, slides, or
  camera easing); the settings file can turn them on too.
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

Action text syntax: whitespace-separated `move:DIR wait open:DIR close:DIR ascend descend lantern
vision:PROFILE` (`DIR` = N NE E SE S SW W NW; `PROFILE` = an id in `assets/data/vision.toml`). Dump layers: `view` (what the player sees), `fov` (`*` visible),
`memory` (seen terrain), `light` (perceived brightness of visible cells as a digit, `0` dark to `9`
fully lit), `truth` (ground truth, **dev-only**). Replay files are JSON
`{game_version, scenario, seed, steps:[{action, hash}]}`; `hash` is the FNV-1a 64 state hash
(`World::state_hash`) after the step, as 16 hex digits.

- `--new` start a fresh run instead of resuming the save; `--dev` enables the dev keys in release
  builds (always on in debug: F2 reveal map (ground truth), F3 next vision profile, F4 light-value
  overlay, F5/F9 snapshots; all listed by `?`); `--save-dir DIR` keeps saves in DIR instead of the platform data dir.

Saves: `Q` saves and quits (closing the window saves too); the game autosaves every 50 turns and on
level change to `run.sav` in the platform data dir (`paths.rs`, app name "warlock"; macOS
`~/Library/Application Support/warlock`). On startup an existing save is resumed unless `--new`, `--capture`,
`--replay`, or `--record` (replays and recordings always start a fresh run). F5 / F9 write / load `snapshots/quick.sav`. An unreadable save is renamed `run.sav.bad`,
never deleted. Format and `Content::fingerprint` live in `warlock-core/src/save.rs`: bump
`FORMAT_VERSION` on any incompatible `WorldState` change; adding or reordering terrain, creatures, or
levels invalidates existing saves.

Settings (`crates/warlock/src/settings/`): `settings.toml` in the platform config dir (macOS
`~/Library/Application Support/warlock`), schema in the `settings/mod.rs` module docs. Changes
apply live and are written at once; a bad file means defaults plus a warning, per value. Palettes
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
assets                fonts, data definitions, levels, scenarios
```

Content (`assets/data`, `assets/levels`, `assets/scenarios`) is TOML, embedded into
`warlock-core` at build time by `build.rs`. Level maps are ASCII with a legend; see
`assets/levels/hub.toml` and spec §7.

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
   atlas whose slots are exactly 2× the 1:1 slots. Creatures and `pulse` terrain live here.
3. **UI console**: the same console material over the whole window; unwritten cells are
   transparent.

**Visual lighting** (spec §4.4, `render/lighting.rs` + `lighting.wesl`, imported by
`console.wesl`): every frame `lighting::update` composites the perceived light for the map grid's
cells plus a one-cell margin, from `PerceptionView::ambient` and `light_sources()` contributions,
and folds in UV (violet tint) and the darkvision fade. Flicker (deterministic noise from the
source key and `VisualTime`; off with reduced effects) is a per-cell brightness gain applied after
shading. Two textures on the map material: `light` (`Rgba32Float`: rgb light, 1.0 = fully lit,
and saturation) and `light_flags` (`Rgba8Uint`: lit/opaque flags, flicker gain byte). The shader
interpolates light per pixel between cell centers, never across opaque, unseen, or remembered
cells (walls are lit flat by their wall-face light); then exposure (readability floor, soft
overbright knee plus a faint glow in the light's hue), desaturation, gain. Unlit cells (remembered
terrain, the UI console's 1×1 textures) are drawn as they are. Free glyphs are lit on the CPU by
`VisualLight::sample` + `lighting::shade`, which mirror the shader — change both together.

`ui/` decides what is drawn from the perception snapshot: `ui/mod.rs` (UI console, map grid in
base colors), `ui/glyphs.rs` (free glyphs), `ui/motion.rs` (slides from `Outcome.events`, camera
easing; new input completes running motion), `ui/dev.rs` (dev overlays: reveal map, the one reader
of ground truth, and light values). Animation curves live in `render/anim.rs` and are mirrored in
`console.wesl` — change both together. Everything animated is a function of `VisualTime`.

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
   (`insta` snapshots live in `crates/warlock-core/src/snapshots/`; review with `cargo insta review`
   or `INSTA_UPDATE=always`, and read the diff). Look at the result with `warlock-cli dump`, and
   check determinism with `replay --create` then `replay --verify`.
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

## Releases, web build, logs

**Cut a release.** Make sure `main` is green, then tag and push the tag (for example
`v0.1.0`); the `Release` workflow does the rest. It builds Linux x86_64 (`.tar.gz`), Windows
x86_64 (`.zip`) and macOS arm64 (`.zip` with an unsigned `Warlock.app` plus the bare binary; see
`release/`) and attaches them, with generated notes, to a GitHub Release. Run the workflow by hand
(Actions > Release > Run workflow) to get the archives as workflow artifacts without publishing.
Releases are the only place macOS and Windows runners are used.

**Web build** (wasm32 + WebGPU; all content is embedded, nothing on the startup path may touch
`std::fs`): one-time setup is `rustup target add wasm32-unknown-unknown` and
`cargo install wasm-bindgen-cli --version <X> --locked`, where `<X>` is the `wasm-bindgen` version
in `Cargo.lock` (the CLI must match exactly; `cargo xtask web` checks it and prints the command).
Then `cargo xtask web` builds `target/web/` (`index.html` from `web/`, `warlock.js`,
`warlock_bg.wasm`). Serve it with `python3 -m http.server --directory target/web 8080` and open
http://localhost:8080 in a WebGPU browser. CI builds it on pushes to `main` (artifact
`warlock-web`). There is no command line on the web, so `cli.rs` uses the defaults.

**Logs and crash reports** (desktop only): each run writes `warlock-<UTC>.log` (last 5 kept) in
`<data dir>/warlock/logs` (macOS `~/Library/Application Support/warlock/logs`, Linux
`~/.local/share/warlock/logs`, Windows `%LOCALAPPDATA%\warlock\data\logs`). A panic writes
`crash-<UTC>.txt` (version, OS, backtrace, last log lines; last 10 kept) next to them and prints
its path. `WARLOCK_CRASH_TEST=1 cargo run -p warlock` triggers a deliberate panic to test this.
Code: `crates/warlock/src/logging.rs`.
