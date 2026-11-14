# Rendering, visual lighting, and effects

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/04-rendering.md. Keep it current with the code, as CLAUDE.md says.

Stairs and locations (terrain with `stairs`) under a pile: `ui/mod.rs::draw_map` draws the
terrain's glyph in the top object's colour (remembered cells dimmed as any), and the dump's
`view`/`memory` layers (`dump.rs::perceived_glyph`) show the stairs' glyph; a known trap still wins
over both (#983).

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
   **The ally mark** (#838; `docs/dev/allies.md`): for a creature whose `SeenActor::ally` is
   set, `ui/glyphs.rs::ally_mark` pushes a free glyph before the creature's (so it draws under
   it): `ALLY_MARK_GLYPH` (`_`) scaled by `ALLY_MARK_SCALE` (0.8, so neighbours' lines never
   join), dropped `ALLY_MARK_DROP` (0.1 cell heights) below the cell to clear the baseline, in
   `ALLY_MARK` (`#8fc096`, a muted sage, through the palette) at `ALLY_MARK_ALPHA` (0.85), lit
   with `ACTOR_FLOOR` at the creature's sliding position. The creature's colour is left alone
   (it tells the type; a tint would read as another creature, and the unaware mark already
   tints). Still: no switch needs to stop it, so reduced effects show the same mark.
   **Zones** (`docs/dev/spells.md` "Zones"): `ui/zone_tint.rs` leans a cell's background toward the
   kind's colour in `draw_map`, after blood, for what `PerceptionView::zone_at` shows, on seen and
   remembered cells alike. Still: no particles, no animation, so reduced effects show the same. A
   fire zone's light is an ordinary point source and flickers by its preset (`arrow_fire`).
   **Spell circles** (spec §4.8 "Spell circles", `docs/design/buff-circles.md`; between the map
   grid and the free glyphs, `render/circles.rs::LAYER_Z` 0.5): `ui/circles.rs::emit` (in the
   map chain after `lighting::update`) builds, when `Game` changes, the list of circled actors
   from the snapshot (the player's `own_effects`, a seen creature's `effects_of`, through
   `Content::circles()`), feeds it to `ui/circle_tracks.rs::Tracks` (the fades and glides, below),
   and each frame pushes a `CircleInstance` per actor (its `Motion` position, a `Shown` per kind,
   buffs and hexes, `VisualLight::keep`, its glyph and its id as a seed) into
   `render::circles::Circles` (emptied in `PreUpdate`, like `FreeGlyphs`). `render/circles.rs`
   holds the pure rule (`layout(effects) -> Circle`: a polygon of `n × ⌈3 / n⌉` corners,
   `min(n, 3)` rings, six effects at most, buffs inner, hexes outer, a `Mark` per effect (its
   status's row and school) only to tell figures apart and colour the lines; `angle`: 60 s
   clockwise, 45 s counter-clockwise, still at 0 and π without `motion`) and draws one dynamic
   mesh of one quad per circle (`2 × (extent + quad_margin)` square), the circle packed into
   flat vertex attributes (`pack`: a head of effects, hex, outer, line school, slots, and rings;
   per kind the old figure's head and `[angle, fade, blend, place]`), with `CircleMaterial`
   (`circles.wesl`: rings and the polygon as signed distances one pixel wide at 18 px (`h / 18`,
   × `LINE_WEIGHT`), dashes along arc and side for a hex; rings at `RING` 0.48 and `RING_GAP`
   0.1 apart, the hexes' outer circle from `OUTER_RING` 0.84; the sizes are mirrored constants,
   change both together; `LINE_ALPHA` 0.2; the school colours a uniform, palette-mapped in
   `ui/circles.rs`).

   **No runes** (the owner, 2026-11-02, #797: "let's just get rid of the hovering kana. i think
   the transmutation circles are enough"): the kana at the polygon's corners, their atlas, and
   their font are gone; so are what served them alone (their tidal lock, pulse, backdrop,
   crowding, and per-rune masks on the CPU). `data/circles.toml` keeps each circled status's
   school and the schools' colours, which colour the lines and decide the status's light.

   **Circle motion** (`ui/circle_tracks.rs`, design §11): one `Track` per actor and kind, fed
   each turn's circles (`update`) and read each frame (`settle`, then `shown`). A kind that
   appears fades in over `render/circles.rs::FADE` (0.5 s), one that ends stays drawn as its
   last figure while it fades out (its actor stays in the list), and a fade that turns goes on
   from where it stood. A changed figure (`Ring::same_figure`: marks, count, kind) keeps the old
   one as `from` and blends over `GLIDE` (0.4 s, `ease`: smoothstep): rings beyond either count
   and the old and new polygon fade by the blend; the line colour blends too. The hexes' kind
   stands at `place` 1 (the outer radii, which the shader mixes by it) while its actor has a
   buffs' track (a going one too), else 0, and glides there over `GLIDE`. An actor no longer
   seen, a level change, or the `circles` switch going off drops the tracks at once. Without
   animation (`motion` off, reduced effects, or frozen visual time: `animate = motion &&
   !frozen`, as `ui/motion.rs`) every kind is `Shown::settled` and a going one is dropped:
   still and cut, and a `--freeze-time` capture shows the settled figure at the frozen angle.

   **Where a circle shows** (#786, the owner's review: "cringe when glyphs overlap other
   glyphs"): the circle material binds the map's `light_flags` (`lighting.rs::cell_flags`: `LIT`,
   `OPAQUE`, and `OCCUPIED`, a visible cell with a glyph of its own: an actor, an object or
   chest, a known trap, or terrain not walkable such as a wall torch) and finds a pixel's cell
   from the framebuffer position and the grid's origin and cell size (`Circles::grid_origin`,
   `cell`, as `console.wesl` does). `show_at` is 1 over clear seen floor, 0 over unseen,
   remembered, opaque, or occupied cells (the actor's own cell, an instance attribute, never
   counts), eased across a cell edge over `SOFT` (0.22 cells) either side; a line pixel is
   masked where it is. `under` gives each circle the nearest earlier one whose rings reach it
   (`ATTRIBUTE_UNDER`; earlier is the order `emit` pushes them: the snapshot's actors, the player
   first): its lines are left out inside that circle's outermost ring. The lines also part about
   the actor's glyph's own outline (its halo in `GlyphGlow`, `1 - smoothstep(0.02, 0.25,
   halo)`), or, for a glyph the atlas lacks, stop `GLYPH_GAP` (a pixel at the default font)
   short of its ink box (`ATTRIBUTE_INK`).

   **Centring**: `emit` centres a circle on its actor's glyph's ink (`atlas::ink_box`: the
   glyph's outline box placed as `rasterize` places it, on the baseline at `ascent.round()`), not
   on the cell's middle: the font's cell holds the descent below the baseline, so a `k` sits a
   little above the middle and an `@` (which hangs below the baseline) a little below it.

   **Glow, glints, shimmer** (#786): per pixel, bottom up, the actor's glyph's halo
   (`GLYPH_HALO_ALPHA` 0.6 × its green, in its buffs' circle's line colour, else its hexes';
   under the glyph, which the free-glyph layer draws over it), the lines' glow
   (`LINE_HALO_ALPHA` 0.1 × a Gaussian of the distance to the ring or side, `LINE_HALO_SIGMA`
   0.045 cell heights), and the lines. Every glow is `Circles::halo`, off with reduced effects,
   separate from the `glow` switch's bloom. The glyph's halo and glints come from
   `render/glyph_glow.rs::GlyphGlow`: each glyph a circled actor shows, rasterised from the map
   font at `Circles::glyph_px` into 2× square slots a cell height wide (ink centred), padded for
   the halo and the rays with an empty one-texel border, one `Rgba8Unorm` texture sampled
   linearly: red the coverage, green the halo (blurred by `GLOW_SIGMA` 0.053 cell heights, ×
   1.5), blue and alpha two glints (`glint_points`: the deepest place in the strokes, the most ink
   with the most ink about it, and the deepest a third of the square from it; each a point of
   `GLINT_SIGMA` 0.03 cell heights with four rays `GLINT_RAY` 0.2 long, `sparkle`); built with
   `@`, grown when a circled actor shows a glyph it lacks (at most `MAX_GLYPHS`), and rebuilt
   when the font size or the cell height changes. A **glint** is the atlas's blue or alpha ×
   `glint(phase, period)`, the top of a sine sharpened (`GLINT_SHARP` 24: about a tenth of its
   beat), periods `GLINT_PERIODS` 5.3 and 7.1 s, on `glint_phase(seed, g)` (the actor's id,
   `CircleInstance::seed`, so two actors are never in step), in the glow's colour most of the
   way to white (`GLINT_WHITE` 0.75) and, with `glow` on, overbright × 2.5 so it blooms
   (`Circles::bloom`); only with `motion` on and without reduced effects. The glints are drawn
   by a **second pass** over the free-glyph layer: the same mesh with a second `CircleMaterial`
   (`params.top` 1, `circles.rs::TOP_Z` 1.2), which draws nothing but them within a cell height
   of the centre. The glyph's slot and the seed ride in the fourth of `ATTRIBUTE_UNDER`
   (`pack_glyph`). The **shimmer**: a soft arc a quarter of a ring wide runs round each ring in
   `SHIMMER_PERIOD` 7 s (rings in turn the other way), up to `SHIMMER` 1.5× brighter, lines and
   glow; motion on and no reduced effects. The time and the motion switch ride in
   `ATTRIBUTE_INK`, not the uniform, so the material is never re-prepared a frame. The quad
   reaches `quad_margin()` past the extent for the lines' glow; the glyph's square fits inside
   (a test checks it). The Rust functions are the shader's mirrors, tested in `circles.rs`. The
   `circles` switch turns the drawing off; reduced effects leaves it on and only stills it (as
   `motion` off does). With `glow` on the school colours are `post::emissive` by
   `ui/circles.rs::CIRCLE_GLOW` (0.1). The light about a circled actor is core's (the status's
   `light`, docs/dev/status-effects.md), drawn by the lighting pass like any carried light, its
   UV tint only for UV eyes; the circle switch never hides it. Check with `--scenario circles`
   captures (a capture without `--freeze-time` at a low `--capture-after-frames` shows the fade
   in); there is no golden of the circles (the owner: "do not create golden screen shot out of
   this particle effect. it's not worth"). The tuning (#779): rings and sides from 0.28 to 0.2
   opaque and the glow from 0.15 to 0.1, after captures at the default font where three rings
   1.8 px apart read as one bright band. Frame cost: `cargo xtask bench-frames`' `circles` row
   (`xtask/src/bench_circles.rs`: fifty circles, `docs/perf.md`).
3. **UI console**: the same console material over the whole window; unwritten cells are
   transparent.

**Visual lighting** (spec §4.4, `render/lighting.rs` + `lighting.wesl`, imported by
`console.wesl`): every frame `lighting::update` composites the perceived light for the map grid's
cells plus a one-cell margin, from `PerceptionView::base_light` (ambient plus gameplay bounce light,
steady, never shadowed by creatures) and `light_sources()` contributions, less creature shadows,
and folds in UV (violet tint) and the darkvision fade. It skips the frame (no composite, no
upload) when the composite would repeat the last one (#785): `Game` and `Shadows` unchanged
(change detection), no transient or spell light live now or in the last composite, the same
origin, size, and flicker switch, and either no flickering source lit the grid or the visual time
is the same (`VisualLight::would_repeat`; `--freeze-time`, flicker off, idle frames). A new input
of the composite must become one of these conditions, or the map keeps stale light. Gameplay light lives in
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
(`Rgba8Uint`: lit/opaque/occupied flags, intensity gain byte; the spell circles read the flags
too). The shader
interpolates light per pixel between cell centers, never across opaque, unseen, or remembered
cells (walls are lit flat by their wall-face light); then exposure (readability floor, soft
overbright knee plus a faint glow in the light's hue), desaturation, gain. Unlit cells (remembered
terrain, the UI console's 1×1 textures) are drawn as they are. Free glyphs are lit on the CPU by
`VisualLight::sample` + `lighting::shade`, which mirror the shader — change both together.
Constants copied between Rust and a `.wesl` file (the grid animation kinds and table size, the
water's glint and flow, the light flags and floors, the circles' sizes and glints, `CRISP`) are
checked by a `shader_constants_match` test in each Rust module, which reads the shader with
`include_str!` and `render::wesl_const` (#1561): a one-sided edit fails `cargo test`.

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
  a slow wander plus jitter, forks only into `open` cells), `ray_strokes`/`ray_line`/`ray_share`/
  `ray_tail` (a ray, a bolt with a `head`: straight from the origin's center to the end's, not
  through the path's cell centers; each stroke fades over `tail` seconds after the head passed
  it; `ui/spells.rs::ray` draws the shaft, then the head turned along the line; its `impact`
  particles start as the head passes each cell), `darken_curve`, `shake_offset`);
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
  `effect:fireball@30,14` and `effect:lightning_bolt@21,17>58,17` in the arena, 0.03 to 0.6 s; a ray, `effect:frost_ray@21,17>35,11`).

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
