# Rendering, visual lighting, and effects

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/04-rendering.md. Keep it current with the code, as CLAUDE.md says.

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
   Glyphs at rest read the 1:1 atlas; moving, scaled, slanted, or turned ones sample a
   supersampled (2×, linear) atlas whose slots are exactly 2× the 1:1 slots. Creatures,
   particles, missiles, the aiming line's dots, and `pulse` terrain live here.
   **Colour bands** (#463): `FreeGlyphs::push_banded` draws a glyph in up to four horizontal
   bands (`free_glyph::Bands`: the glyph's colour first, then `more`, a `count`, and a `drift`).
   The vertex's glyph data is `[slot, flags, count, drift bits]` and the other three colours are
   flat vertex attributes, so plain glyphs cost nothing more in the shader than a branch. The
   bands lie over the rows a capital letter covers (`GlyphAtlas::cap`, the params' `ink`), band
   k's middle at k + ½ band heights, with a soft border (`SEAM`, 0.35 of a band) between
   neighbours; `drift` (band heights) moves the borders, clamped so the top and bottom bands
   always stay. `ui/glyphs.rs` uses it for a creature whose `Shape::bands()` has two or more
   colours (a composite animated thing, `docs/dev/animated.md`): each band is palette-mapped,
   lit, and glows as the glyph does, and the drift is a slow sway (`band_drift`: ±0.3 band, 7 s,
   the creature's own phase), 0 without glyph animation (reduced effects). A sliding creature is
   banded too: it is always on this layer.
   **The unaware mark** (#687, Stalker's eye; `docs/dev/surprise.md`): for a creature whose
   `SeenActor::unaware` is set (the snapshot sets it only for a player with the talent),
   `ui/glyphs.rs` mixes its colour, and each band's, toward `UNAWARE_TINT` (`#8a96a8`, a cool
   slate) with `unaware_tint(fg, mix)` before the palette and the light. `unaware_mix` breathes
   between 0.15 and 0.35 over 6 s (`anim::breathe`, CPU only, no shader mirror; the creature's
   own phase) with glyph animation on, and stands at 0.25 without it (reduced effects): still
   information, no motion. Tuned against the darkvision fade (less than it), with captures
   beside an aware creature of the same type.
   **Spell circles** (spec §4.8 "Spell circles", `docs/design/buff-circles.md`; between the map
   grid and the free glyphs, `render/circles.rs::LAYER_Z` 0.5): `ui/circles.rs::emit` (in the
   map chain after `lighting::update`) builds, when `Game` changes, the list of circled actors
   from the snapshot (the player's `own_effects`, a seen creature's `effects_of`, through
   `Content::runes()`), feeds it to `ui/circle_tracks.rs::Tracks` (the fades and glides, below),
   and each frame pushes a `CircleInstance` per actor (its `Motion` position, a `Shown` per kind,
   buffs and hexes, `VisualLight::keep`) into `render::circles::Circles` (emptied in
   `PreUpdate`, like `FreeGlyphs`). `render/circles.rs` holds the pure rule
   (`layout(effects) -> Circle`: slots `n × ⌈3 / n⌉`, `min(n, 3)` rings, six effects at most,
   buffs inner, hexes outer; `angle`: 60 s clockwise, 45 s counter-clockwise, still at 0 and π
   without `motion`; `slot_angle`: the glide's path; its unit tests are the design's §4 rows and
   sketches) and draws one dynamic mesh of one quad per circle (`2 × extent` square), the circle
   packed into flat vertex attributes (`pack`: a head of effects, hex, outer, line school, slots,
   and rings, and nine bits a rune, slot and school; per kind the old figure's head and runes,
   and `[angle, fade, blend, place]`), with `CircleMaterial` (`circles.wesl`: rings and the
   polygon as signed distances one pixel wide at 18 px (`h / 18`, × `LINE_WEIGHT`), dashes along
   arc and side for a hex, an upright rune square per slot; the sizes are mirrored constants,
   change both together; alphas `LINE_ALPHA` 0.2 and `RUNE_ALPHA` 0.7; the school colours a
   uniform, palette-mapped in `ui/circles.rs`).

   **Circle motion** (`ui/circle_tracks.rs`, design §11): one `Track` per actor and kind, fed
   each turn's circles (`update`) and read each frame (`settle`, then `shown`). A kind that
   appears fades in over `render/circles.rs::FADE` (0.5 s), one that ends stays drawn as its
   last figure while it fades out (its actor stays in the list), and a fade that turns goes on
   from where it stood. A changed figure (`Ring::same_figure`: runes, count, kind) keeps the old
   one as `from` and blends over `GLIDE` (0.4 s, `ease`: smoothstep): in the shader every slot up
   to the larger count stands at `slot_angle(k, old, new, blend)`, its rune drawn whole when the
   old and the new rune there agree, else cross-faded; rings beyond either count and the old and
   new polygon fade by the blend; the line colour blends too. The hexes' kind stands at `place`
   1 (the outer radii, which the shader mixes by it) while its actor has a buffs' track (a going
   one too), else 0, and glides there over `GLIDE`. An actor no longer seen, a level change, or
   the `circles` switch going off drops the tracks at once. Without animation (`motion` off,
   reduced effects, or frozen visual time: `animate = motion && !frozen`, as `ui/motion.rs`)
   every kind is `Shown::settled` and a going one is dropped: still and cut, and a
   `--freeze-time` capture shows the settled figure at the frozen angle. The runes come
   from `render/rune_atlas.rs`: every rune of `runes.toml` rasterised from
   `assets/fonts/NotoSansJP-Runes.ttf` (a subset of Noto Sans JP Medium; `assets/fonts/README.md`
   has the `pyftsubset` command, and `every_rune_is_in_the_rune_font` checks it is current) into
   2× square slots with an empty one-texel border, one `R8Unorm` texture sampled linearly,
   rebuilt when the cell height changes. The `circles` switch turns the drawing off; reduced
   effects leaves it on and only stills it (as `motion` off does). With `glow` on the school
   colours are `post::emissive` by `ui/circles.rs::CIRCLE_GLOW` (0.1). The light about a circled
   actor is core's (the status's `light`, docs/dev/status-effects.md), drawn by the lighting pass
   like any carried light, its UV tint only for UV eyes; the circle switch never hides it. Check
   with `--scenario circles` captures (the golden `circles`, at `freeze_time = 1`: settled);
   a capture without `--freeze-time` at a low `--capture-after-frames` shows the fade in. The
   tuning (#779): rings and sides from 0.28 to 0.2 opaque and the glow from 0.15 to 0.1, after
   captures at the default font where three rings 1.8 px apart read as one bright band;
   the runes, the sizes, and the speeds are as designed. Frame cost: `cargo xtask
   bench-frames`' `circles` row (`xtask/src/bench_circles.rs`: fifty circles, `docs/perf.md`).
3. **UI console**: the same console material over the whole window; unwritten cells are
   transparent.

**Visual lighting** (spec §4.4, `render/lighting.rs` + `lighting.wesl`, imported by
`console.wesl`): every frame `lighting::update` composites the perceived light for the map grid's
cells plus a one-cell margin, from `PerceptionView::base_light` (ambient plus gameplay bounce light,
steady, never shadowed by creatures) and `light_sources()` contributions, less creature shadows,
and folds in UV (violet tint) and the darkvision fade. Gameplay light lives in
`warlock-core/src/light/` (`mod.rs`: falloff and the wall-face rule; `cache.rs`: per-source shadow
masks, a `hollow` preset's leaving out its own cell, and the player's carried light from
`WorldState::carried_light`, `world/lamp.rs`: a lantern ready in the Tool slot, by its object's
`light` and the stack's mode, hooded its `hooded` preset; `bounce.rs`: one weak
bounce; `sun.rs`: sunlight; `moon.rs`: moonlight and `Sky`) and is combined in `perception.rs`;
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

Effects (spec §4.8), each off with its own switch (`particles`, `glow`, `distortion`,
`screen_fades`, `screen_shake`, `weather`) and all off with reduced effects (but `blood` and
`circles`, which reduced effects leaves as set):
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
- **Rain** (`ui/rain.rs`, the `weather` switch): the weather's `drops`/`splash`/`ripples` emitters
  (`data/weather.toml`) run as ambient emitters on every visible unroofed cell of the grid (plus
  `BELOW` rows), keyed by level position; a particle is drawn only over a cell it rains on, at most
  `MAX_PARTICLES` a frame. Drops are `│` sheared along their velocity (`FreeGlyph::slant`, pixels
  across per pixel down, applied in `free_glyph::glyph_mesh`) and faint (`ParticleEmitter::opacity`).
- **Spell effects** (spec §4.8 "Spell effects"): `Event::Spell { effect, origin, shape:
  SpellShape::{Burst { center, radius }, Bolt { path }}, cells }` in `warlock-core/src/action.rs`
  (traps emit it; `perceive_events` cuts it with `world/spell.rs::perceive_spell` to the cells the
  player sees or has in line of sight). Data: `data/effects.toml` (`content/effect.rs`,
  `EffectDef`, `Content::effects()`/`effect(id)`; presentation only, not in the fingerprint; a
  test plays the same run with changed looks and checks the hashes). The dev action
  `effect:ID@X,Y[>X,Y]` (`World::show_effect`, `Spent::NoTime`) makes one for captures; because it
  takes no turn, `ui/spells.rs` reads each outcome once by `Game::applied`, not by the turn. The
  model is pure in `render/spells.rs` (`Timeline` travel/impact/linger, `strength`, `burst_heat`
  with `burst_distance` measured on screen, `bolt_strokes` (turned `━` strokes about a cell wide,
  a slow wander plus jitter, forks only into `open` cells), `darken_curve`, `shake_offset`);
  `ui/spells.rs` (`Spells`) starts casts (one after another; frozen time: all at visual time 0, so
  `--freeze-time T` shows T s after), writes `render::lighting::SpellLights` (lights, `dim`, and
  the cells seen, which spell light passes even when unknown), and draws glyphs and particles last
  (on top, unlit, undimmed). **Darkening** happens where the light is made: `VisualLight` scales
  everything but spell light by `SpellLights::keep()` (`(1 − dim)^2.2`, so `darken.amount` is how
  much darker it looks), free glyphs scale their floor (`glyphs::lit`) and emissive particles and
  burning missiles their color (`glyphs::darkened`) by `VisualLight::keep()`, and the map shader
  gets `ConsoleParams::dim` for its floor and unlit (remembered) cells. **Shake**:
  `Spells::shake` (whole physical pixels) is added to the camera in `draw_map_layer` before
  `camera_split`, so the grid, free glyphs, and light move together and the viewport edges never
  gap. `motion` off: a still flash (`Timeline::still`); `screen_fades` off: no darkening;
  `screen_shake` off: no shake. Tune with `--content-dir` and captures across freeze times (e.g.
  `effect:fireball@30,14` and `effect:lightning_bolt@21,17>58,17` in the arena, 0.03 to 0.6 s).

`ui/` decides what is drawn from the perception snapshot: `ui/mod.rs` (UI console, map grid in
base colors, the status bar: `status_segments`), `ui/circles.rs` (spell circles), `ui/glyphs.rs` (free glyphs), `ui/motion.rs`
(slides from `Outcome.events`, one per creature along all the steps it took that turn; camera
easing; new input completes running motion), `ui/effects.rs` (particles, bursts, glow and haze,
fades), `ui/rain.rs` (rain), `ui/spells.rs` (spell effects), `ui/sounds.rs` (heard-sound markers), `ui/look.rs` (look mode), `ui/dev.rs` (dev overlays: reveal map, the one reader of
ground truth, and light values).
Animation curves live in `render/anim.rs` (water: `render/water.rs`, test-only mirrors of the
shader) and are mirrored in `console.wesl` — change both together. Everything animated is a function of `VisualTime`. Effects are deliberately subtle (the
bar: sparse, short-lived particles, gentle bloom, barely perceptible distortion, short fades;
the one exception is spell effects, dramatic but short); judge
changes by capturing several `--freeze-time` values, at `--scale 1` and `--window 2560x1440
--scale 2`.
