# Warlock — Specification

Status: refined draft v2 (2026-10-17)
Repository: https://github.com/jannef/warlock — used for code, documentation, issues, and pull requests.

## 1. Vision

Warlock is a turn-based, grid-based roguelike in the spirit of classic ASCII ADOM, rendered natively
on the GPU. Everything on screen — creatures, terrain, items, effects, UI — is a glyph. No tiles, no
real terminal.

It should feel far beyond the ASCII roguelikes of old: colored dynamic light with real shadows,
perception that depends on the viewer's eyes, living animated glyphs, and glyph-based particles and
shader effects — while every glyph stays readable.

Targets: native desktop on Windows, macOS, and Linux. A web build (WASM + WebGPU) is nice to have.

## 2. Scope of the first phase

Build the engine and framework, not the game. Everything built is production quality (§10).

**In scope**
- GPU glyph renderer with lighting, glyph animation, and effects
- Hand-authored levels from text files; multiple levels joined by stairs, each persistent
- Player movement (8 directions), doors (open/close), stairs
- Line of sight, terrain memory, gameplay lighting, vision types
- Inert test creatures walking fixed patrol routes
- Save and resume an unfinished run
- Keyboard controls with an in-game controls reference
- Development interface (§9)
- Procedural generation with reproducible seeds — late (M6), not a first priority

**Out of scope for now**: combat, statistics, item effects, AI, character creation, audio, mouse
input, overworld/wilderness map, infravision, localization.

Vision types are in scope even though there are no other mechanics yet: they are part of perception,
and the light data model must carry them from the start.

## 3. Showcase world

From M1 on, the playable build is a hand-authored **showcase world** where `@` walks through one room
per engine capability. Each milestone adds or upgrades rooms. The showcase world is also the
regression scene for golden screenshots, replays, and snapshot tests.

**Level 1 — Hub (80×21)**

| Room | Demonstrates | Milestone |
|---|---|---|
| Start room | movement, message line, `?` controls overlay | M1 |
| Door corridor | opening/closing doors, doors blocking sight | M1–M2 |
| Patrol gallery | creatures moving in and out of view, never shown in memory | M2 |
| Light hall | colored lights, falloff, pillar shadows, flicker | M3 |
| Darkness chamber | vision types, a UV-only rune, darkvision fade | M3 |
| Water garden | rippling water, pulsing altar, shimmer | M4 |
| Effects room | glyph particles, glow, fades, distortion | M4 |
| Stairs | connection to Level 2 | M2 |

**Level 2 — Vast (≥ 400×300)**: scrolling camera, many light sources, performance budget check.
Introduced in M2, lit in M3.

## 4. Rendering

### 4.1 Glyphs
- A monospace TTF font is rasterized into a glyph atlas at the exact pixel size for the current zoom
  and display scale. Static glyphs are drawn 1:1 with nearest sampling, so they are crisp at any
  window size and DPI.
- Animated or scaled glyphs sample a supersampled copy of the atlas with linear filtering.
- Any Unicode glyph the font provides may be used (box drawing, blocks, arrows).
- Bundled fonts must have an OFL-compatible license. CP437 bitmap sheets may be added later as an
  alternative glyph source.

### 4.2 Layers
1. **Grid layer** — viewport-sized textures (glyph index, foreground, background, visual light,
   perception state) drawn as one quad with a custom 2D material. Terrain, remembered terrain, and
   in-cell animation.
2. **Free-glyph layer** — instanced glyph quads from the same atlas for actors, particles, and
   anything that moves between cells or grows beyond its cell. Lit by sampling the light texture.
3. **UI layer** — panels, message log, and overlays, drawn in glyphs.
4. **Post-processing** — glow/bloom, distortion, fades.

Rendering is viewport-based: drawing cost does not depend on level size.

### 4.3 Glyph animation
Declared in data per terrain or entity type:
- `wave` — ripple, e.g. water `~` swaying gently, with waves travelling across a pool
- `pulse` — heartbeat-style "lub-dub" grow/shrink
- `shimmer` — color oscillation
- `cycle` — glyph frame sequence, e.g. `~` ↔ `≈`

Each cell gets its own phase from a hash of its position, so neighbors never move in lockstep.
Animation that stays inside the cell runs in the grid shader; animation that exceeds the cell (a
pulse above 1.0 scale) uses the free-glyph layer.

### 4.4 Visual lighting
- Derived from gameplay light (§5), plus visual-only modulation: flicker and transient effect lights
  (e.g. a spell's glow). Visual-only light never affects gameplay.
- Each frame, the CPU composites cached per-light shadow masks × flicker into a viewport-sized float
  texture. The shader interpolates it for smooth gradients without bleeding across walls.
- **Darkvision**: color fades toward monochrome as a cell gets darker and farther from the viewer.
- **UV light** is drawn (as a violet tint) only when the viewer has UV sight.

### 4.5 Perception states
- **Visible** — fully lit rendering, including creatures and effects.
- **Remembered** — dim, desaturated terrain as last seen. Never creatures, effects, or current light.
- **Unknown** — blank.

### 4.6 Motion and time
- Visual time runs continuously while gameplay waits for input. Animation never blocks or changes
  gameplay.
- Moving glyphs slide between cells over ~80 ms; the camera follows smoothly. Any new input
  completes running animations instantly, so fast play never waits.
- Visual randomness uses its own RNG and never consumes gameplay randomness.

### 4.7 Readability and reduced effects
- Glyphs stay legible under every effect: minimum contrast against their background, glow added
  around glyphs rather than over them, small distortion amplitudes.
- A **reduced effects** setting turns off glyph animation, flicker, sliding (instant snap), bloom,
  and distortion, and minimizes particles. No gameplay information may depend on an effect.

## 5. Visibility, light, and vision

These are gameplay rules. They live in `warlock-core`, are deterministic, and never depend on frame
rate or animation.

- **Line of sight**: symmetric shadowcasting (if A sees B, B sees A). Walls and closed doors are opaque.
- **Light**: each cell stores integer **RGB + UV**. A light source has a position, color (RGB + UV),
  intensity, and radius. Falloff uses integer math or lookup tables — no transcendental
  floating-point functions in gameplay. Shadows use the same shadowcasting. Each level has an
  ambient light value.
- **Wall faces**: an opaque cell's perceived light is the maximum light of the adjacent transparent
  cells the viewer can see. This prevents light leaking through walls. (Refined in M3.)
- **Visibility**: a cell is visible iff it is in line of sight within the maximum view range **and**
  (its perceived light ≥ the eye's threshold **or** its distance ≤ the darkvision radius).
- **Vision profiles** are data. First version:

  | Profile | Light threshold | Darkvision radius | Senses |
  |---|---|---|---|
  | Normal | 40 | 0 | visible light |
  | Low-light | 12 | 0 | visible light |
  | Darkvision | 40 | 5 | visible light |
  | UV sight | 40 | 0 | visible + UV light |

  Values are illustrative; the light scale is fixed in M3. The player's profile can be switched
  with a dev command. Infravision is deferred.
- **Memory**: per level, terrain only, as last seen (a door is remembered open until seen closed).
- **Recomputation**: perception updates in the perception phase of every turn. Per-light shadow masks
  are cached and invalidated when the light moves or opacity changes within its radius.

## 6. Input and UI

- Keyboard-first. Default sets: vi-keys (`hjklyubn`), numpad, arrows (4 directions).
- Symbol commands (`<`, `>`, `?`) are bound by **logical key**, so they work on non-US layouts.
- Bindings live in the settings file. The `?` controls overlay is generated from the binding table,
  so it is always accurate. A rebinding UI comes in M5.
- First-phase commands: move, wait, open/close door, ascend/descend, look, help, save and quit.
  Free actions (look, help, menus) do not advance time.
- Screen layout: map viewport, message log, status panel (turn, position, level, light at the
  player, vision profile, seed).

## 7. Levels and data formats

Rule: **hand-authored data is TOML; machine-written data is JSON.**

- **Level files** (TOML): a header (name, ambient light, legend, lights, patrols, stair links) plus a
  multiline ASCII map. Level size comes from the map, with no fixed upper limit: most dungeon levels
  are about 80×21, some are vast. The camera scrolls when a level is larger than the viewport. The
  same format will later define vaults for procedural generation. Illustrative only — exact syntax
  is designed in M1:

  ```toml
  name = "Hub"
  ambient = [8, 8, 12, 0]  # r, g, b, uv
  map = '''
  ##########
  #@..T...+.
  #..~~~..#>
  ##########
  '''

  [legend]
  "#" = "wall"
  "." = "floor"
  "+" = "door_closed"
  "~" = "water"
  "T" = { terrain = "floor", light = "torch" }
  ">" = { terrain = "stairs_down", to = "vast" }
  "@" = { terrain = "floor", spawn = "player" }

  [[patrol]]
  creature = "rat"
  path = [[12, 3], [20, 3], [20, 8]]
  ```
- **Definitions** (TOML): terrain types, light presets, vision profiles, glyph animations.
- **Scenarios** (`assets/scenarios/*.toml`): level set, start position, vision profile, options.
- **Saves**: suspend-only. One save per run, written on quit and by periodic autosave; loading
  continues the run. Dev builds add named snapshots. Format: a versioned envelope (magic, format
  version, game version) around a JSON payload, gzip-compressed with a pure-Rust backend, written
  atomically (temp file + rename). Before 1.0, incompatible saves are rejected with a clear message;
  from 1.0 on, saves are migrated. The web build uses browser storage.
- **Settings**: a separate file in the platform config directory.
- **Replays**: JSON action logs with a header (game version, scenario, seed) and per-turn state hashes.
- **Dumps**: JSON simulation state plus ASCII layers.

## 8. Architecture

```
crates/
  warlock-core/   plain Rust, no Bevy: map, actors, actions, turns, fov, light,
                  perception, rng, levels, save
  warlock-cli/    no Bevy: scenarios, replay, dump, verify
  warlock/        Bevy app: render, input, UI, effects, platform (ECS lives here)
xtask/            screenshot capture, golden-image diff, CI helpers
assets/           fonts, levels, scenarios, definitions, shaders
```

- **Pure core**: `warlock-core` has no Bevy dependency, no platform I/O inside the rules, and no
  wall-clock time. It builds and tests in seconds and is insulated from Bevy upgrades.
- **Data layout**: dense grids for terrain, light, and memory; a simple entity store with stable ids
  for actors. ECS is used in the presentation layer, where it helps with effects, particles, and
  animation.
- **Perception snapshot**: after each turn, the core produces what the player perceives. The
  renderer, UI, and dumps read only this snapshot, so hidden information cannot leak by construction
  (a dev "reveal" mode excepted).
- **Turn phases**, in order: player action → creatures (stable id order) → environment → perception
  (line of sight, light, memory) → emit presentation events.
- **Events to effects**: the core emits events (door opened, actor moved); the presentation layer
  turns them into animations. Gameplay never waits for them.
- **Determinism**: ChaCha8 RNG with separate streams — map generation (hash of run seed and level
  id), gameplay, and visual-only. No hash-order iteration in the core. The same seed and actions
  produce the same result within a game version; seeds are not guaranteed stable across versions,
  and snapshot tests flag when generator output changes.
- Small modules with clear responsibilities; no speculative abstractions or premature optimization.

## 9. Development interface

```
warlock --scenario showcase --seed 42                     # play a named scenario
warlock --scenario showcase --replay runs/doors.json \
        --freeze-time 1.5 --window 1280x720 --scale 1 \
        --capture out.png                                 # reproducible screenshot, then exit
warlock --record runs/new.json                            # record actions for bug reproduction
warlock-cli dump --scenario showcase --replay runs/doors.json \
        --format ascii --layers view,light,fov,memory     # text view of the simulation
warlock-cli replay --verify runs/doors.json               # check per-turn state hashes
cargo xtask golden [--update]                             # compare golden screenshots
```

- Freezing visual time and fixing window size and scale make screenshots reproducible.
- Dev builds add in-game commands: switch vision profile, reveal map, toggle debug overlays (light
  values, line of sight, memory), save and load named snapshots.
- CLAUDE.md documents build commands, architecture notes, pinned documentation references, and
  verification procedures, including the agent workflow guidance from the original brief.
  Verification uses tests, dumps, logs, and screenshots — a successful compile alone does not
  verify rendering.

## 10. Quality bar (Definition of Done)

- CI is green: `cargo fmt`, `clippy -D warnings`, and tests on Linux, macOS, and Windows; a
  `wasm32` compile check; `cargo-deny`.
- Core: unit tests; property tests for line-of-sight symmetry and for sight and light never passing
  through walls; snapshot tests of ASCII dumps; replay determinism tests.
- Renderer: golden screenshots per showcase room, compared on Linux CI with software Vulkan.
  Reference images are updated deliberately, never automatically.
- No panics on player paths; errors are shown to the player gracefully; `tracing` logs go to a file
  in release builds.
- Every milestone ends with a downloadable, playable build for each OS.
- Performance budgets (proposed; to be confirmed on reference hardware): 60 fps at 1440p on an
  integrated GPU; turn processing ≤ 5 ms in the hub and ≤ 16 ms in the vast level.
- The spec and CLAUDE.md are updated together with behavior changes.

## 11. Technology

| Area | Choice | Notes |
|---|---|---|
| Language | Rust (stable) | |
| Engine | Bevy 0.20.x, `default-features = false` | Pinned; upgrades are deliberate tasks. 0.19 and 0.20 postdate most model training data, so pinned docs and examples are mandatory. No audio, 3D, or glTF features. |
| Shaders | WGSL (WESL-compatible) | Bevy 0.20 adopted WESL; plain WGSL remains valid. |
| Font rasterization | chosen in M1 | Bevy's text stack or a dedicated rasterizer. |
| RNG | ChaCha8 | Pinned algorithm with stable output. |
| Serialization | serde + JSON, TOML | bincode rejected: permanently unmaintained (RUSTSEC-2025-0141). |
| Testing | cargo test, proptest, insta | |
| CI/CD | GitHub Actions | Linux/macOS/Windows; software Vulkan for screenshots. |
| Web | wasm32 + WebGPU | Compile check from M0; real web build in M7. |

## 12. Milestones

Each milestone ships a playable build.

- **M0 Foundation** — Rust toolchain, cargo workspace, CI matrix plus wasm check, CLAUDE.md with
  pinned docs, lints, cargo-deny, xtask; a window opens and `--capture` works in CI.
- **M1 Walking skeleton (first playable)** — grid renderer v1, TTF atlas, level format and loader,
  hub v0, 8-direction movement, walls, doors, turn pipeline, bindings and `?` overlay, message line,
  scenario/seed/capture/freeze-time flags, downloadable builds.
- **M2 Perception and dev loop** — symmetric shadowcasting, memory, perception snapshot, patrol
  creatures, stairs and level persistence, vast level v0, replay/record/dump/state hash, save and
  resume v1 with dev snapshots, property and snapshot tests.
- **M3 Light and vision** — gameplay light grid (RGB + UV), wall-face rule, vision profiles, visual
  light compositing with flicker, interpolated lighting, darkvision fade, debug overlays; light hall
  and darkness chamber.
- **M4 Renderer quality and effects** — free-glyph layer, glyph animations (wave, pulse, shimmer,
  cycle), glyph particles, bloom, fades, distortion, movement slide, camera, HiDPI and resize,
  golden screenshots per room; water garden and effects room.
- **M5 Settings and accessibility** — settings persistence, reduced effects, font and scale, color
  palettes, rebinding, main menu with new run and continue.
- **M6 Procedural generation** — RNG streams, generators, vault embedding, connectivity validation,
  seed UX, snapshot tests.
- **M7 Ship** — packaging per OS, tagged releases, web build, performance budgets verified, crash logs.

## 13. GitHub workflow

All through the `gh` CLI:
- Milestones M0–M7 as GitHub milestones; one issue per task.
- Labels: `area:core`, `area:render`, `area:input`, `area:tools`, `area:ci`, `area:docs`;
  `type:feature`, `type:bug`, `type:chore`.
- One pull request per issue, referencing it; CI required; squash merge into `main`.

## 14. Deferred and open

- License (the repo is private for now; bundled fonts must be OFL-compatible regardless)
- Mouse support (hover-inspect first)
- Overworld / wilderness map
- Infravision
- Audio
- Localization
- Performance budget numbers, pending reference hardware
