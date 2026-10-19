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

Effects (spec §4.8), each off with its own switch (`particles`, `glow`, `distortion`,
`screen_fades`, `screen_shake`, `weather`) and all off with reduced effects:
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
base colors, the status bar: `status_segments`), `ui/glyphs.rs` (free glyphs), `ui/motion.rs`
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
