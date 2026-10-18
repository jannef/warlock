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
- Test creatures walking fixed patrol routes; from M10 they act at their own hidden speed, belong
  to factions, hear noise, and fight with deterministic AI whose attacks are intents only
- Pickable objects (glyph, color, material, weight), inventory, and creature weight (M11)
- Moons with phases and moonlight, weather starting with rain (M12)
- Save and resume an unfinished run
- Keyboard controls with an in-game controls reference
- Development interface (§9)
- Procedural generation with reproducible seeds (M6): dungeon branches of generated levels
  (rooms and corridors, caves) with hand-made vaults, joined to the hand-authored levels by stairs
- An overworld with travel time, locations, local areas, and day and night (M9)

**Out of scope for now**: combat resolution (attacks are announced, nothing is damaged), statistics,
item effects, character creation, audio, mouse input, infravision, localization.

Vision types are in scope even though there are no other mechanics yet: they are part of perception,
and the light data model must carry them from the start.

## 3. Showcase world

From M1 on, the playable build is a hand-authored **showcase world** where `@` walks through one room
per engine capability. Each milestone adds or upgrades rooms. The showcase world is also the
regression scene for golden screenshots, replays, and snapshot tests.

**Level 1 — Hub (80×21)**

| Room | Demonstrates | Milestone |
|---|---|---|
| Start room | movement, message line, `?` help | M1 |
| Door corridor | opening/closing doors, doors blocking sight | M1–M2 |
| Patrol gallery | creatures moving in and out of view, never shown in memory | M2 |
| Light hall | colored lights, falloff, pillar and creature shadows, steady magical light | M3, M8 |
| Darkness chamber | vision types, a UV-only rune, darkvision fade | M3 |
| Water garden | still ponds and running rills (flow, glints, foam), pulsing altar | M4, M8 |
| Effects room | glyph particles, glow, fades, distortion | M4 |
| Stairs | connection to Level 2 (down), the courtyard and the overworld (up) | M2, M9 |

**Courtyard (66×19, outdoor)**: up the stairs nook's `<`, a walled garden under the open sky —
trees, a ruined wall, a pond, standing stones, flower beds, and a roofed gazebo with a lamp — where
sunlight, sky light, and shadows turn and stretch with the time of day (§5 "Sunlight"; M9). The
`courtyard` scenario starts there at 06:30, while the shadows are long.

**Level 2 — Vast (≥ 400×300)**: scrolling camera, many light sources, performance budget check.
Introduced in M2, lit in M3. Beside its stairs up to the hub, stairs down lead into the depths and
stairs up lead out to the overworld (the cavern mouth). A river of running water winds through it
from north to south, foaming against a few boulders.

**The Depths (generated, M6)**: a dungeon branch of eight procedurally generated levels
(`depths:1` to `depths:8`; rooms and corridors at 80×21, caverns at 120×60 on depths 3–5) built
from the run seed, with vaults stamped in. The `depths` scenario starts on its first level.

**The Overworld (M9)**: *The Wildlands*, a hand-authored 100×50 map of the land at large
(`levels/overworld.toml`, level kind `overworld`), ADOM-like. Overworld terrain: plains,
grassland, forest, dense forest, hills, mountain ranges, swamp, rivers and deep water, roads, a
bridge, and locations. A step is a journey: it takes the destination terrain's `travel_minutes`
(road and bridge 20, plains 45, grassland 50, forest 90, hills 120, swamp and dense forest 150,
locations 30); mountains and open water are impassable, and the river Wendle (drawn flowing, with
legend arrows like running water; §4.3) can only be crossed by the bridge. Named regions (Greenmarch, Thornwood, Barrowdowns, Hollowcrag, Silvermere, Sallow
Plains, Mirefen, Greyspine Mountains) show in the status bar.
- **Locations** are stairs down: the Hall (to the hub), the cavern mouth in the Hollowcrag (to the
  vast level), and the way into the Depths in the Barrowdowns (`depths:1`). `>` on a location
  enters its level, arriving on the stairs that lead back; those stairs (`<`) lead out to the
  overworld at the location. The hub's stairs nook has a `<` to the overworld.
- **Local areas**: `>` on any other cell whose terrain names a local ground enters the outdoor
  local area of that cell (level id `local:x,y`, 80×40), generated on the first visit from the
  run seed and the cell (§7 "Generators", *outdoor*) and then persistent like any level. The
  player arrives on the side (or corner) they walked onto the cell from — stepping east onto it,
  on the area's west edge, on the road if they came along one — or in a glade in the middle if
  they did not walk there (arriving by stairs or back from the area itself). Walking off any edge
  returns to the overworld cell. What an area looks like follows its overworld terrain's
  landscape and its neighbors:
  - *plains*: dry grass with greener hollows, tall grass, a lone tree or a copse, bushes,
    boulders; *grassland*: green meadows, tall grass, wildflowers and buttercups, copses fringed
    with bushes, lone trees, grazing deer;
  - *forest*: woods of leafy trees with stands of pines in clumps, undergrowth and bluebells,
    grassy clearings, a glade in the middle; *dense forest*: thick woods, mostly pines, small
    clearings, game trails from the middle out to every side;
  - *hills*: rocky ground with grassy slopes, crags (rock outcrops, 2 tall) with fallen boulders
    around them, a few pines, now and then a standing stone; *swamp*: marsh with pools of still
    water, reeds, grassy hummocks, willows, and dead trees;
  - a *road* cell runs a dirt road (2 wide, bridged where it crosses water) from near the middle
    out towards every neighboring road, bridge, or location, through the land most of its
    neighbors have; a *river* beside the cell runs along that side of the area (running water,
    one speed, flowing the way the overworld river does) and a *lake* lies along it (still water,
    reeds in the shallows);
  - ponds, a brook (running water across the area, quicker than a river), and ruins (outdoor
    vaults: a stone circle with an old altar that glows faintly at night, a broken watchtower, a
    roofless chapel) turn up now and then, and how wooded the land is blends towards the
    neighboring cells near each side, so areas side by side meet.

  Trees stand 2 tall, pines 3, standing stones 3, crags 2, bushes and reeds 1 (they cast sun
  shadows; trees, pines, crags, and boulders block sight). The `forest` scenario starts in a
  forest glade of the Thornwood at 06:30.
- **Day and night**: sight on the overworld depends on daylight (§5), and the sun and the sky
  light the overworld and its local areas by day (§5 "Sunlight"); at night the overworld is drawn
  by a faint cold light, while local areas are dark.
- The `world` scenario starts on the road beside the Hall; the showcase scenario keeps starting in
  the hub.

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
4. **Post-processing** — glow/bloom, distortion, fades (§4.8).

Rendering is viewport-based: drawing cost does not depend on level size.

### 4.3 Glyph animation
Declared in data per terrain or entity type:
- `wave` — ripple, e.g. still water `~` swaying gently, with waves travelling across a pool
- `flow` — running water: the glyph moves downstream along the cell's flow (§7)
- `pulse` — heartbeat-style "lub-dub" grow/shrink
- `shimmer` — color oscillation
- `cycle` — glyph frame sequence, e.g. `~` ↔ `≈`

Each cell gets its own phase from a hash of its position, so neighbors never move in lockstep.
Animation that stays inside the cell runs in the grid shader; animation that exceeds the cell (a
pulse above 1.0 scale) uses the free-glyph layer.

Water is calm by design; motion stays tasteful, never busy:
- **Still water** (pools, ponds, the water garden, cave pools) is a `wave` with a faint, slow
  swell (a pixel or so), and **glints** (`glint`, 0–1, on `wave` and `flow`): now and then, at a
  random moment of each cell's own cycle (about one glint every 20 s per cell, under a second
  long), the glyph brightens a little in the color of the light on it, following that light's
  stability (its flicker and hue drift, §4.4). Dark water never glints; the glints are
  reflections of nearby lights.
- **Running water** (rivers, streams) is a `flow`. Each cell has a flow direction (8 ways) and
  speed (§7). The shader draws the glyph as a pattern repeating every cell and moves it
  downstream at the terrain's `speed` (cell widths per second) times the cell's flow speed, so
  ripples leave a cell on one side and enter the next, and a stream reads as one moving surface;
  faster water visibly runs faster. Neighbors with the same flow line up seamlessly; where the
  direction or speed changes along a stream the two patterns meet at a seam, so streams keep an
  even speed along their length. Where the flow runs into something that is not running water —
  a bank, a boulder, a pool — sparse foam particles (`obstacle` emitters, §4.8) appear.
- Flow is presentation data like the animations: gameplay never reads it (currents may come
  later). With glyph animation off, water is still and never glints; with particles off, nothing
  foams.

### 4.4 Visual lighting
- Derived from gameplay light (§5), plus visual-only modulation: light stability (flicker and hue
  drift), creature shadows, and transient effect lights (e.g. a spell's glow). Visual-only light
  never affects gameplay.
- Each frame, the CPU composites the perceived light of the viewport's cells (plus a one-cell
  margin) from the perception snapshot — the base light (ambient plus sky plus bounce, steady) plus
  every visible source's per-cell contribution, the sun's included — into a viewport-sized float
  texture aligned with the map grid. The shader
  interpolates it per pixel between cell centers for smooth gradients; a neighbor contributes only
  if it is visible and transparent, so light never bleeds across walls or corners, nor into unseen
  or remembered cells. Walls show their own wall-face light, flat.
- A surface's base color is its look at full light (255). Below that it dims linearly (in linear
  light) to a readability floor; overbright light brightens it a little more along a soft knee and
  adds a faint glow in the light's own color, so strongly lit floors pool with color.
- **Light stability** is per source, from its preset's optional `stability` block; a preset
  without one is perfectly stable (constant hue and intensity), as magical light (crystals, runes,
  fungus) is. Firelight is gently alive; lamps and the lantern burn calmer. Three optional parts,
  each a pure function of the source's stable key and visual time (so frozen-time captures are
  reproducible):
  - `intensity = { amount, speed }`: smooth noise (two octaves of value noise) within ±`amount`,
    changing about `speed` times a second;
  - `hue = { degrees, speed }`: smooth noise from an independent stream turns the light's HSV hue
    within ±`degrees` (positive toward yellow), its luminance kept. The turn scales the two weaker
    channels together against the strongest, the way a flame's color moves when it burns hotter
    or cooler: a warm light goes paler yellow one way and deeper, more saturated orange the other.
    A channel the light lacks stays dark;
  - `dips = { depth, every, length }`: about once every `every` seconds (one dip at a random moment
    in each `every`-second slot) the intensity eases down by up to `depth` and back over `length`
    seconds (a raised cosine, its depth drawn between half and all of `depth`); with hue drift the
    dip also pulls the hue toward its deep end.
  - Compositing: the hue tints the source's contributions on the CPU, exactly (they are
    composited per source); the intensity becomes a per-cell brightness gain (the mean of the
    intensity of the sources lighting the cell, weighted by the light they draw; base light counts
    as steady) applied after shading, so it reads the same in dim and bright light. Creature
    shadows scale the modulated light.
  - Subtle by design: felt more than noticed, no strobing. `amount` and `depth` are at most 0.25
    each, `degrees` at most 30. Off (steady light) with the flicker switch or reduced effects.
- **Creature shadows** (visual only: gameplay light and visibility ignore them, so creatures never
  hide in each other's shadows): every visible creature, the player included, shades each perceived
  source that lights its cell, except a source on its own cell (the light it carries). The cells
  behind it as seen from the source (the source's shadowcast over the terrain the player perceives,
  with the creature's cell as an extra occluder, among the cells the source lights) keep only part of
  that source's light: a creature does not fill its cell, so about 40 % right behind it, easing
  back to all of it 3.5 cells away. Shadows of several creatures multiply. Colored shadows follow
  from the per-source attenuation: blocking a red light leaves the cells behind lit by the others.
  The sun (§5 "Sunlight") has no preset, so it is never modulated (no flicker or hue drift), and
  it is directional: a creature, one cell tall, shades the
  cells along the sun's shadow line from its cell (the line terrain shadows use, up to the first
  opaque cell), 40 % next to it easing back to all of the light one step past the shadow's end — so
  creature shadows are long at dawn and dusk and short at noon.
  Shadows are derived from the perception snapshot once per turn (a sliding creature's shadow is
  already at its destination); darkvision's glow follows the perceived light, not the shadows.
- **Darkvision**: a cell seen only through darkvision gets a faint monochrome glow that dims toward
  the edge of the darkvision radius, and its color fades toward monochrome as it gets darker and
  farther from the viewer: saturation = 1 − darkness · (1 − 0.3 · (1 − distance / radius)) with
  darkness = 1 − brightness / threshold. Full color at the threshold, 30 % in total darkness next
  to the viewer, monochrome in total darkness at the edge of the radius.
- **UV light** is drawn (as a violet tint) only when the viewer has UV sight.
- Free glyphs (creatures, pulsing terrain) are lit by the same light, sampled at their center.

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
- A **reduced effects** setting is a preset that turns off glyph animation, flicker (light
  stability: light is steady), sliding (instant snap), camera easing, bloom and glow, distortion,
  particles, and screen fades and flashes. Each can also be switched off on its own (settings:
  glyph animation, light flicker, motion = slides and camera easing, bloom and glow, distortion,
  particles, screen fades); an effect runs only when its switch is on and reduced effects is off. No gameplay information may
  depend on an effect. These are settings (§6); the `--reduced-effects` flag forces everything off.
- Colors that effects introduce, such as the shimmer target color, go through the active palette
  like any other color.
- **Color palettes** (default, high contrast, a red-green colorblind variant) are transforms
  applied when colors are written to the screen, so game data colors are never altered. UI text
  keeps at least 4.5:1 contrast, and the UI never conveys information by color alone (selection
  has a `>` marker, refusals start with `!`).

### 4.8 Effects
Declared in data per terrain or creature type, like glyph animation. They are presentation-only:
gameplay never reads them. Effects are deliberately subtle — sparse, short-lived particles, gentle
glow, barely perceptible distortion, short fades.
- **Glyph particles** on the free-glyph layer. An emitter has glyphs to pick from, colors over the
  particle's life, a lifetime, velocity ± spread, gravity, a start offset and spawn area, sideways
  sway, scale, fade-out, and glow. Emitters fire `ambient` (a steady rate while the source is in
  view: sparks over braziers and torches, embers over lava, fountain spray, dust motes), on `step`
  (a burst when a creature is seen entering the cell: a splash in water), on `toggle` (a burst
  when a door is seen opening or closing: dust), or at an `obstacle` (a steady rate from visible
  running water whose flow runs into a known cell that is not running water: foam at banks and
  boulders, ripples where a stream enters a pool; the emitter is turned with the flow, its `y`
  pointing downstream). Creature emitters move with the creature.
- Particles are a pure function of visual time and a seed (the source's cell or identity, or the
  event) through a visual-only RNG, so they never depend on frame rate or gameplay randomness. Only
  visible cells, seen creatures, and perceived events emit; particles over cells that are not
  visible are not drawn. Non-glowing particles are lit like free glyphs (§4.4); glowing ones are
  light themselves.
- **Glow**: glowing glyphs (`glow`, 0–1) are drawn brighter than white into an HDR target. Lit
  colors, even under overbright light, are kept at most white, so bloom takes only glowing glyphs
  and adds a tight, faint halo; every other color looks as without bloom and glyph cores stay
  readable.
- **Heat haze** (`haze`, 0–1) rises a few cells above hot terrain (braziers, lava). The grid shader
  moves where glyphs there are sampled by about a pixel, in slow rising ripples; backgrounds stay
  put and text elsewhere stays crisp.
- **Screen tints** over the map viewport: a short fade in from black after a level change, and a
  flash (color, strength, duration) as the hook for hit-style events.
- With frozen visual time (`--freeze-time`) fades complete instantly and bursts show a fixed
  moment early in their life, so captures are reproducible.

## 5. Visibility, light, and vision

These are gameplay rules. They live in `warlock-core`, are deterministic, and never depend on frame
rate or animation.

- **Line of sight**: symmetric shadowcasting (if A sees B, B sees A). Walls and closed doors are
  opaque. Radii are Euclidean with a slightly rounded edge: a cell is within radius `r` iff
  `d² ≤ r² + r`. The maximum view range is 40.
- **Light**: each cell's light is integer **RGB + UV**, one `u16` per channel: **255 = fully lit**
  (a surface shows its base color), more is overbright. A cell's light is the level's ambient light
  (`ambient = [r, g, b, uv]`, default 0) plus sky light (outdoors, below) plus the sum of all
  sources that reach it (the sun included) plus bounce light (saturating).
- **Light sources** come from light presets (`color = [r, g, b, uv]`, which is also the intensity,
  and `radius`): terrain that emits light, lights placed on cells by the level legend (overriding
  the terrain's own), creatures that carry light, and the player's lantern (from the scenario,
  toggled by the player; takes a turn). Emitters must be transparent; a source in an opaque cell
  (e.g. on a closed door) gives no light. Presets also carry visual-only stability (§4.4), never
  read by rules.
- **Falloff and shadows**: a source shadowcasts with the same shadowcasting within its radius and
  adds `color · (R² − d²) / R²` with `R² = r² + r`: full color at the source, monotonically less
  with distance, zero at the edge. Integer math only — no floating point or transcendental
  functions in gameplay.
- **Bounce light**: one weak bounce, gameplay light like the rest. The direct light of all sources
  (red, green, blue; UV does not bounce) on each transparent cell is reflected, tinted by the mean
  `bounce` color (a terrain field, default white = neutral) of the surfaces there: the cell's own
  terrain and the opaque orthogonal neighbors bounding it. The reflected light then spreads 3 steps:
  each step, every transparent cell takes the mean of itself and its linked neighbors (transparent
  orthogonal neighbors, and transparent diagonal ones with a transparent orthogonal cell between).
  A cell's bounce light is 32/256 (12.5 %) of the result: it never crosses a wall, reaches at most
  3 cells from the surface that reflected it, fading with distance, and is never more than 12.5 %
  of the brightest direct light within reach (exactly that in an evenly lit area). Integer math,
  derived from unmodulated gameplay light, so steady. Sunlight bounces like any other light.
  Ambient plus sky plus bounce light is a cell's *base light*.
- **Wall faces**: an opaque cell is lit by each source as a transparent cell at its own distance
  would be (`color · (R² − d²) / R²`), but only if the viewer sees a face the source lights: one of
  its 8 neighbors is transparent, seen, and lit by the same source. This prevents light leaking
  through walls, and a wall is never brighter than the floor at the same distance. Its base light
  is ambient plus the brightest sky-plus-bounce light of its seen transparent neighbors.
- **Sunlight**: levels under the open sky (`kind = "outdoor"` and the overworld; never
  dungeons) are lit by the sun and the sky by day. Terrain with `roofed = true` (indoor floors on
  an outdoor map) never receives either.
  - **Sun path**: one entry per minute of the day, computed at compile time with integer math (a
    fixed-point sine; no runtime trigonometry, so identical on every platform). The hour angle runs
    from 0 at 06:00 to half a turn at 18:00; the sun's direction is its cosine and sine (east,
    screen +x, at sunrise; south, +y, at noon; west at sunset), and its elevation is 60° times its
    sine — below the horizon at night. A test pins a hash of the whole table. (Seasons later.)
  - **Light by elevation**: sunlight fades in over the first 2°, dim deep orange, then orange
    (6°), warm gold (12°), warm white (25°), and bright neutral white from 40° (`[325, 312, 280]`);
    it is one more source with a stable key and no flicker. **Sky light** is a dimmer bluish light
    (`[58, 72, 110]` with the sun high, dimmer and more violet while it is low) on every cell under
    the sky, shaded or not, part of the base light; a short twilight glow while the sun is less
    than 6° below the horizon (about 23 minutes before 06:00 and after 18:00). At night there is
    neither: moonlight is darkness for now.
  - **Shadows**: terrain stands `height` cells tall (default 1 if opaque, 0 if transparent; trees
    2, standing stones 3, at most 4). A cell `h` tall casts a shadow `h · cot(elevation)` long (at
    most 12 cells per unit of height), away from the sun: long at dawn and dusk, short at noon (one
    cell north of a tree). It covers the cells an integer line from the caster passes through, one
    step per cell along the line's major axis, as many steps as the shadow is long along that axis
    (rounded). A cell is in sun shadow iff, marching toward the sun, a cell taller than itself is
    within that cell's shadow. An opaque cell takes sunlight by the wall-face rule (a seen face lit
    by the sun) unless it is shaded itself. Sky light is not shadowed.
  - Computed in the perception phase over the bounce area (the cells in line of sight plus 3),
    marching at most as far as the tallest terrain's shadow reaches, so cost follows the view, not
    the level size.
- **Brightness** is the integer luma of RGB, `(54·r + 183·g + 19·b) / 256`, plus UV for eyes that
  see UV. Eyes without UV sight do not perceive UV at all (perception reports it as 0).
- **Visibility**: a cell is visible iff it is in line of sight within the maximum view range **and**
  (its perceived brightness ≥ the eye's threshold **or** its distance ≤ the darkvision radius). The
  player's own cell is always visible. Creatures are seen iff their cell is visible: creatures in
  darkness are hidden (unless within darkvision), creatures carrying light are seen.
- **Overworld sight** is at map scale: a cell of the overworld is visible iff it is in line of
  sight within the daylight radius: 9 by day (06:30–17:30), 5 in twilight (half an hour either side
  of 06:00 and 18:00), 2 at night (moonlight is darkness for now). Light, the eye's threshold, and
  darkvision play no part. Mountain ranges and dense forest are opaque; forest is not.
- **Overworld night light**: the overworld is lit by the sun and the sky like any level under the
  open sky ("Sunlight" below), and never gets darker than a faint cold light (`[48, 56, 90]`,
  channel by channel, added to its ambient light where sun and sky fall short): it only draws the
  cells in sight at night apart from the remembered map, since sight there ignores light. Local
  outdoor levels get no such light: at night they are dark (bring a lantern).
- **Wall completion**: an opaque cell within the view range that is 8-adjacent to a transparent cell
  seen by light or darkvision is visible too, even outside line of sight, so floor seen through a
  narrow, irregular gap is never shown without the walls bounding it. (The player's own cell, which
  is visible regardless of light, does not complete walls: in total darkness the player sees only
  themselves.) Such a cell reveals only its terrain and wall-face light — never creatures, and
  nothing beyond it.
- **Vision profiles** are data (`data/vision.toml`). First version:

  | Profile | Light threshold | Darkvision radius | Senses |
  |---|---|---|---|
  | Normal | 40 | 0 | visible light |
  | Low-light | 12 | 0 | visible light |
  | Darkvision | 40 | 5 | visible light |
  | UV sight | 40 | 0 | visible + UV light |

  The scenario sets the starting profile. The dev action `SetVision` switches it; it takes no game
  time but is recorded in replays like any action. Infravision is deferred.
- **Memory**: per level, terrain only, as last seen (a door is remembered open until seen closed).
- **Recomputation**: perception updates in the perception phase of every turn (and after
  `SetVision`). Per-light shadow masks are cached and invalidated when the light moves or opacity
  changes within its radius; only sources whose radius can reach the view range (or a surface whose
  bounce light reaches it) are evaluated. Bounce light and sunlight are computed over the bounding
  box of the cells in line of sight plus bounce's 3-cell reach, so they do not depend on where the
  viewer stands. Sunlight is not cached (it moves every minute).
- **Perception API**: the snapshot exposes, for visible cells only, the perceived light (after the
  wall-face rule; plus whether the cell is seen only through darkvision and its squared distance),
  the light sources whose light reaches visible cells (stable key; emitter: a point light's
  position and preset, or the sun with its shadow direction; and per-cell contributions;
  contributions plus the cell's base light sum to the cell's light), the base light of visible
  cells, the active vision profile, the ambient light, and on a level under the sky the sun and sky
  of the moment. Visual lighting (§4.4) is derived from this.

## 6. Input and UI

- Keyboard-first. Default sets: vi-keys (`hjklyubn`), numpad, arrows (4 directions).
- Symbol commands (`<`, `>`, `?`) are bound by **logical key**, so they work on non-US layouts.
- Bindings live in the settings file. The `?` help (the controls list, read-only) is generated from
  the binding table, so it is always accurate.
- **Screens**: the main menu, New run, pause menu, Settings, Controls, and help are modal screens on
  one stack. Opening a screen puts it on top; Esc or Back closes it and returns to the screen
  below as it was left. While any screen is open it receives every key press, the run (if any)
  stays visible beneath it, and no game time passes. Prompts in the message line (a direction
  to open or close, how long to rest, look mode) are not screens.
- **Main menu** (drawn in glyphs; the game opens on it unless `--scenario`, `--new`, `--replay`,
  `--record`, or `--capture` name what to run): Continue (only if a save exists), New run
  (scenario list from content, optional seed), Settings, Controls, Quit.
- **Pause menu**: Esc during a run (when no prompt is open) opens it, drawn in glyphs:
  Resume, Settings, Controls, Save and quit to menu, Quit (which saves). No game time passes while
  it is open; Esc or Back in Settings/Controls returns to it.
- **Settings** (`settings.toml` in the platform config dir; never on the web): font size (logical
  px, 12 to 36), reduced effects and the per-effect switches, color palette, key bindings. A missing or damaged file means
  defaults and a logged warning. Changes apply live (a font size change rebuilds the glyph atlas)
  and are written immediately.
- **Rebinding**: in Controls, select a command and press a key; it becomes the command's primary
  key (alternates such as the numpad stay). A key another command uses is refused and the owner
  is named, so the table never holds a conflict. Esc (cancel) is fixed. Delete resets a command,
  "Reset all keys" resets everything.
- First-phase commands: move, wait, open/close door, ascend/descend, light/douse lantern (`L`),
  rest (`R`), look, help, save and quit (`Q`). Free actions (look, help, menus, vision switch) do
  not advance time.
- **Rest** (`R`) asks in the message line: `Rest: (h)our, until (d)awn, until (n)ightfall? (Esc to
  cancel)`. `h` rests an hour, `d` until 06:00, `n` until 18:00 (replay text `rest:1h`,
  `rest:dawn`, `rest:dusk`); any other command cancels. Resting is one action that passes turn
  after turn and stops early when a creature comes into view or a message is logged (§8 "Time").
  It is a gameplay input mode like the open/close direction prompt, not a menu screen.
- Screen layout: map viewport, message log, status panel (level, region (on the overworld and in
  its local areas), time of day and date as `14:32 · 12 Rainmoon`, turn, position, light at the
  player, vision profile, lantern, seed; segments drop in priority order when the window is narrow,
  the time last after the level name).

## 7. Levels and data formats

Rule: **hand-authored data is TOML; machine-written data is JSON.**

- **Level files** (TOML): a header (name, ambient light, legend, lights, patrols, stair links,
  and how time passes: `kind = "dungeon"` (default), `"outdoor"` (under the open sky, lit by the
  sun and the sky; §5 "Sunlight"), or `"overworld"` (also under the sky), optional `turn_seconds`,
  1 to 3600, default 6 on dungeon and outdoor levels and 600 on the overworld; §8 "Time"; optional
  named regions, `[[region]] name = "Thornwood" area = [x0, y0, x1, y1]`, inclusive, first match
  wins) plus a multiline ASCII map. Level size comes from the map, with no fixed upper limit: most dungeon levels
  are about 80×21, some are vast. The camera scrolls when a level is larger than the viewport. The
  same format defines vaults for procedural generation (below). Illustrative only — exact syntax
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
- **Water**: `~` is still water (`water`). Running water (`running_water`, terrain with a `flow`
  animation) needs a flow on every cell, given by its legend entry: `flow` (the direction it runs,
  `N` … `NW`) and an optional `flow_speed` (percent of the terrain's animation speed, 10–300,
  default 100). The shared legend (`data/legend.toml`) draws running water as arrows pointing the
  way it flows: `↑ ↗ → ↘ ↓ ↙ ← ↖` at normal speed and `⇑ ⇗ ⇒ ⇘ ⇓ ⇙ ⇐ ⇖` at double speed, so a
  stream reads in the map file as it flows, e.g. `~~~←←←~~~`. A `flow` on any other terrain, or
  running water without one, is a content error. Levels carry their flow into the game (saved with
  the level; generated levels get theirs from vaults); like animation it is presentation data that
  no rule reads. The vast level's generator derives the river's flow from its channel: each row
  runs toward where the channel continues in the next row, edge cells turn toward water below them
  where the channel narrows, and a few boulders (from their own random stream, so the rest of the
  level is unchanged) stand in the wide stretches for the water to foam against.
- **Definitions** (TOML): terrain types, light presets, vision profiles, glyph animations, and
  effects (glow, heat haze, particle emitters; §4.8). Terrain has a `move_cost`: the time to step
  onto it in percent of the level's turn (default 100; still and running water 200), a `bounce`
  tint for the light it reflects (§5), a `height` in cells for sun shadows (default 1 if opaque,
  else 0; at most 4), and `roofed` (never lit by the sun or the sky). Overworld terrain has
  `travel_minutes` (1 to 1440; required for every walkable terrain on the overworld),
  optionally `local`, the ground terrain of the local area entered from a cell of it (none:
  nothing to explore there; locations never have one), and `landscape`, what its land is like in
  local areas (required with `local`): `plains`, `grassland`, `forest`, `dense_forest`, `hills`,
  or `swamp` pick the generator of its own areas; `road` runs a road through them amid the
  neighbors' land; `river` and `lake` put running or still water along that side of the
  neighboring areas (§3). The calendar's month names are
  `data/calendar.toml` (exactly 12; without the file months are numbered).
- **Dungeon branches** (`data/dungeons.toml`): stacks of generated levels, e.g.
  `{ id = "depths", generator = "rooms", depth = 8, size = [80, 21] }`, plus optional name,
  patrolling creature kinds, ambient light, and per-depth overrides of generator and size. Stairs
  (and scenarios) may target `depths:N`; content validation accepts such ids when the branch has
  that depth. A branch level is generated on its first visit from its own random stream (§8) and
  then persists like a hand-made level. Its `<` leads to `depths:N-1` (on depth 1, back to the
  level it was first entered from), its `>` to `depths:N+1` (the deepest level has none).
- **The overworld**: at most one level of kind `overworld`. Its stairs down are locations; every
  other cell whose terrain has a `local` ground names a local area, level id `local:x,y` (canonical
  decimal, no signs or leading zeros), which stairs and scenarios may also target. A local area is
  generated on its first visit from the content, the seed, and its id (its own random stream,
  `map/local:x,y`), then persists; it records its edge exit (the overworld and the cell), and
  walking off its edge leads there. `>` arrives on the edge cell of the side the player walked
  onto the cell from (the run remembers the direction of the last overworld step until the next
  level change): on the road if one leaves the area there, else on dry ground if any, near the
  middle of the side (or the corner) but trading a few cells along it for open ground around
  it; with no such step, at the area's start in the middle.
- **Generators** (`warlock-core`, integer-only): *rooms* — walled rooms joined along a minimum
  spanning tree plus a few loops by corridors dug as cheapest paths (bends cost extra), with closed
  doors, open doors, or doorways where corridors enter, and wall torches in most rooms; *caves* —
  cellular automata (45 % rock, five 4-5 smoothing passes, largest cave kept), pools of still water,
  lamps and glowing fungus. Both place the up staircase (the arrival point) and the down staircase
  far from it, and a few creatures on looping patrols. Every level is validated by flood fill —
  every stair, door, and walkable cell reachable from the up staircase, patrols valid, some light
  — and a level that fails is regenerated from the next derived stream. *outdoor* (the local
  areas of the overworld, §3): smooth integer value noise (random values on a coarse lattice,
  eased between) sized as a share of the area lays everything in natural blobs — woods, clearings,
  meadows of flowers, pools, crags; the share of woods blends towards each neighboring cell's
  within the third of the area nearest it. Each attempt lays water along river and lake sides
  (bands whose banks wander with the noise), a brook (a winding course, two cells wide, one flow
  direction per straight stretch), and the road first; keeps a glade clear around the middle;
  adds a pond and ruins (outdoor vaults) where they fit on open land; grows the landscape; then
  joins every walkable pocket to the start (small ones are grown over with what surrounds them,
  larger ones get a way cleared through the cheapest obstacles) and opens every side. A few deer
  graze on looping patrols in meadows and woods. Validation: the start is walkable, every walkable
  cell is reachable from it (4-connected), every side has at least 3 walkable cells, running water
  and only running water has a flow, and patrols are valid.
- **Vaults** (`assets/vaults/*.toml`): hand-made pieces in the level-file format, stamped verbatim
  (terrain, lights, creatures) where they fit — into rock with room for corridors or tunnels to
  every entrance. A vault's edge is solid except for its entrances (doors, not on a corner); it has
  no stairs or player start, and all its walkable cells are reachable from an entrance. Vaults of
  `kind = "outdoor"` are ruins for local areas, stamped on open land with a ring of open ground
  around them: their spaces (`void`) keep the area's own ground, every open cell on their edge is
  a way in, and their walkable cells are reachable from the edge. Dungeon generators use only
  dungeon vaults, local areas only outdoor ones.
- **Scenarios** (`assets/scenarios/*.toml`): level set, start position, vision profile, lantern,
  options, and when the run starts: `start_date = [year, month, day]` (all from 1; default
  `[1, 1, 1]`) and `start_time = "hh:mm"` (default `"08:00"`).
- **Saves**: suspend-only. One save per run, written on quit and by periodic autosave; loading
  continues the run. Dev builds add named snapshots. Format: a versioned envelope (magic, format
  version, game version) around a JSON payload, gzip-compressed with a pure-Rust backend, written
  atomically (temp file + rename). Before 1.0, incompatible saves are rejected with a clear message;
  from 1.0 on, saves are migrated. The web build keeps the same bytes in the browser's
  `localStorage` (below). The envelope also records a
  content fingerprint (hash of the ordered terrain, creature, level, light, and dungeon-branch
  ids), because ids are indices; a save made with different content is rejected. Generated levels
  are saved with the run like visited hand-made ones (terrain, memory, stairs, lights, kind and
  turn length, arrival point, and edge exit), together with the gameplay random stream, the game
  clock, and the direction of the last overworld step (format version 4; saves without it load
  with none). The save is `run.sav` in the platform
  data directory. `Q` saves and quits (closing the window saves too), autosave runs every 50 turns
  and on level change, and a save that cannot be read is renamed `run.sav.bad`, never deleted, while
  a new run starts. Starting resumes the save unless `--new` is given. Dev snapshots: `F5` saves and
  `F9` loads `snapshots/quick.sav` (debug builds or `--dev`).
  On the web the files are `localStorage` keys `warlock/run.sav`, `warlock/run.sav.bad`,
  `warlock/snapshots/quick.sav`, and `warlock/settings.toml`, the save bytes base64-encoded, with the
  same envelope and checks. The same triggers save (autosave, level change, `Q`), and so does the page
  being hidden or closed (`visibilitychange` to hidden, `pagehide`); main menu Continue works after a
  reload. Storage that is unavailable (some privacy modes) turns saving off with a warning in the log;
  a write that fails (quota) shows a message in the message line; neither panics.
- **Settings**: a separate file in the platform config directory.
- **Replays**: JSON action logs with a header (game version, scenario, seed) and per-turn state hashes.
- **Dumps**: JSON simulation state plus ASCII layers.

## 8. Architecture

```
crates/
  warlock-core/   plain Rust, no Bevy: map, actors, actions, turns, time, fov, light,
                  perception, rng, levels, mapgen, save
  warlock-cli/    no Bevy: scenarios, replay, dump, verify
  warlock/        Bevy app: render, input, UI, effects, platform (ECS lives here)
xtask/            screenshot capture, golden-image diff, CI helpers
assets/           fonts, levels, scenarios, vaults, definitions, shaders
```

- **Pure core**: `warlock-core` has no Bevy dependency, no platform I/O inside the rules, and no
  wall-clock time. It builds and tests in seconds and is insulated from Bevy upgrades.
- **Data layout**: dense grids for terrain, light, and memory; a simple entity store with stable ids
  for actors. ECS is used in the presentation layer, where it helps with effects, particles, and
  animation.
- **Perception snapshot**: after each turn, the core produces what the player perceives. The
  renderer, UI, and dumps read only this snapshot, so hidden information cannot leak by construction
  (a dev "reveal" mode excepted).
- **Turn phases**, in order: player action → the clock advances by the action's duration →
  creatures (stable id order) → environment → perception (line of sight, light, memory) → emit
  presentation events.
- **Time** (`warlock-core/src/time.rs`): world time is whole seconds since the calendar epoch
  (year 1, day 1 of the first month, 00:00), kept in the world state (saved, covered by the state
  hash). The calendar has 12 months of 30 days (a 360-day year) and 24-hour days; month names are
  content (Frostmere, Thawmonth, Seedtide, Rainmoon, Greenleaf, Highsun, Emberheat, Harvestide,
  Leaffall, Mistmoon, Darkening, Deepwinter). `World::time()` returns a `GameTime` with year,
  month (number and index; the name from `Content::calendar`), day, hour, minute, second, minute
  of day (the index for per-minute tables such as the sun's path, §5), and the day fraction in
  16-bit fixed point. All
  integer arithmetic; no wall-clock time.
  - **Costs**: every action that takes a turn advances the clock. A turn (a wait, a door, stairs,
    lighting the lantern) takes the turn length of the level it starts on (6 s on `dungeon` and
    `outdoor` levels, 10 min on the overworld, unless the level sets `turn_seconds`). A step takes
    that turn scaled by the destination terrain's `move_cost` percent (at least 1 s); on the
    overworld it takes the destination terrain's `travel_minutes`. `LevelState::step_time` is the
    one place movement time is decided. Walking off a local area's edge takes its turn. Free and
    refused actions take no time. Creatures act once per turn, however long it took (no speed
    system yet).
  - **Rest** passes turn after turn through the normal pipeline: the player waits, creatures act,
    perception runs; the last turn is cut short so the rest ends exactly at its goal (one hour,
    the next 06:00, or the next 18:00; at most a day). It stops after a turn in which a creature
    came into view (one not seen the turn before) or a message was logged, and logs how long the
    player rested. While every creature on the level is farther than the view range plus one
    (Chebyshev) from the player, the turns until one could come within view pass between two
    perceptions: they cannot change when the rest stops, and a night's rest stays fast on large
    levels (`cargo xtask bench-turns` times one; budget 500 ms).
  - **Dev skip** (`F6` in dev builds, replay text `skip:1h`) moves the clock ahead at once;
    nobody acts and no turn passes, but it is recorded like any action.
- **Events to effects**: the core emits events (door opened, actor moved); the presentation layer
  turns them into animations. Gameplay never waits for them.
- **Determinism**: ChaCha8 RNG (`rand_chacha`) with separate streams, each seeded with FNV-1a 64 of
  the run seed (little-endian) and a label: map generation per level (`map/<level id>`, retry *k*
  `map/<level id>/<k>`), so a level depends only on the seed and its id, never on visiting order;
  gameplay (`gameplay`, kept in the world state and saved); and visual-only randomness in the app,
  which never uses either. No hash-order iteration and no floating point in the core's decisions.
  The same seed and actions produce the same result within a game version; seeds are not
  guaranteed stable across versions, and snapshot tests flag when generator output changes. A seed
  is a number or a word (`--seed warlock`, hashed with FNV-1a 64).
- Small modules with clear responsibilities; no speculative abstractions or premature optimization.

## 9. Development interface

```
warlock --scenario showcase --seed 42                     # play a named scenario
warlock --replay runs/doors.json \
        --freeze-time 1.5 --window 1280x720 --scale 1 \
        --capture out.png                                 # reproducible screenshot, then exit
warlock --record runs/new.json                            # record actions for bug reproduction
warlock-cli dump --scenario showcase --actions "move:N move:N" \
        --format ascii --layers view,fov,memory           # text view of the simulation
warlock-cli dump --replay runs/doors.json --format json --state   # structured dump
warlock-cli replay --create --scenario showcase --actions "move:N wait" -o runs/x.json
warlock-cli replay --verify runs/doors.json               # check per-turn state hashes
warlock-cli gen --level depths:3 --seed warlock           # print a generated level (dev-only;
                                                          # also local areas: local:49,23)
cargo xtask golden [--update]                             # compare golden screenshots
```

- Replays are JSON: `{game_version, scenario, seed, steps: [{action, hash}]}`. Actions use a text
  syntax (`move:E wait open:E close:W ascend descend lantern rest:1h rest:dawn rest:dusk
  vision:darkvision skip:1h`; durations are a whole number with `s`, `m`, or `h`, a rest at most
  `24h`); `hash` is the
  FNV-1a 64 hash of the
  canonical JSON encoding of `WorldState` after the step. Verification reports the first
  diverging step. `--replay` takes its scenario and seed from the file; `--record` rewrites its
  file after every action (including replayed steps).
- Dump layers: `view`, `fov`, `memory`, `light` (perceived brightness of each visible cell as a
  digit, `0` dark to `9` fully lit or brighter) are built from the perception snapshot only;
  `truth` shows ground truth and is dev-only. JSON dumps carry turn, game time (seconds and
  calendar fields), level, viewer, seen actors, recent messages, and optionally the full world
  state; the ASCII header line has the turn, the time (`0001-01-01 08:00:00`), level, viewer, and
  hash.
- Freezing visual time and fixing window size and scale make screenshots reproducible. With frozen
  time, glyph animations show that moment and motion (slides, camera) completes instantly.
- Dev builds add in-game commands: switch vision profile (`F3`), reveal map (`F2`, ground truth,
  marked as dev-only on screen), toggle debug overlays (light values `F4`; line of sight and memory
  later), save and load named snapshots (`F5`/`F9`), skip an hour (`F6`, recorded as `skip:1h`).
  They are in the bindings table, so the `?` help lists them.
- CLAUDE.md documents build commands, architecture notes, pinned documentation references, and
  verification procedures, including the agent workflow guidance from the original brief.
  Verification uses tests, dumps, logs, and screenshots — a successful compile alone does not
  verify rendering.

## 10. Quality bar (Definition of Done)

- CI (Linux) is green: `cargo fmt`, `clippy -D warnings`, tests, headless screenshots, a
  `wasm32` compile check, `cargo-deny`. macOS is verified locally before every PR (development
  happens there); macOS does not run in CI.
- Core: unit tests; property tests for line-of-sight symmetry and for sight and light never passing
  through walls; snapshot tests of ASCII dumps; replay determinism tests.
- Renderer: golden screenshots per showcase room, compared on Linux CI with software Vulkan.
  Reference images are updated deliberately, never automatically.
- No panics on player paths; errors are shown to the player gracefully; `tracing` logs go to a file
  in release builds.
- Every milestone ends with a downloadable, playable build for each OS, published by the tagged
  release workflow (Linux, Windows, macOS).
- Performance budgets (the frame budget is not yet measured on an integrated GPU, #74): 60 fps at 1440p on an
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
| CI/CD | GitHub Actions | CI on Linux only (software Vulkan for screenshots); release archives for Linux, Windows, and macOS on tags. |
| Web | wasm32 + WebGPU | Compile check from M0; real web build in M7. |
| License | GPL-3.0-only (`LICENSE`) | © Janne Lê Forsell. Dependencies must be GPL-3.0-compatible (the cargo-deny allow list); the bundled font stays under OFL-1.1. The main menu shows the copyright and license notice, every release archive and the web build carry `LICENSE` and `THIRD_PARTY_LICENSES.html` (every shipped dependency's copyright notice and license text, generated by `cargo xtask licenses` and checked in CI). |

## 12. Milestones

Each milestone ships a playable build.

- **M0 Foundation** — Rust toolchain, cargo workspace, Linux CI plus wasm check, CLAUDE.md with
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
  Released as v0.1.0.
- **M8 Lighting polish** — creature shadows, weak bounce light, walls lit from their own distance,
  fungus glow, still and running water, per-source light stability (hue and intensity drift), and
  along the way the screen stack (one way to open, route keys to, and draw modal screens), look
  mode, and a smaller web build.
- **M9 Time, sunlight and overworld** — game clock and 12×30 calendar, time costs per level and
  terrain, resting; sunlight, sky light and moving shadows on sky-exposed levels over a 24-hour
  day (moonlight is darkness for now); an ADOM-like overworld with per-terrain travel time,
  locations, and sight by daylight; outdoor local areas generated from overworld terrain.
- **M10 Creatures** — an ADOM-like speed system (base speed 1000, never shown to the player),
  factions with hostile or neutral relations, deterministic AI that approaches and announces melee,
  ranged, and spell attacks (no combat resolution), and hearing: actions make noise that the
  player and creatures hear.
- **M11 Objects and inventory** — pickable objects that each define glyph, color, material, and
  weight; piles on the floor, pick up and drop, the inventory screen; creature weight (base plus
  carried).
- **M12 Moons and weather** — configurable moons (one by default: 30-day period, faint purple
  light) with phases, paths, and shadows; weather starting with rain; the full calendar date and
  moon phase in the status bar.

## 13. GitHub workflow

All through the `gh` CLI:
- Milestones M0–M7 as GitHub milestones; one issue per task.
- Labels: `area:core`, `area:render`, `area:input`, `area:tools`, `area:ci`, `area:docs`;
  `type:feature`, `type:bug`, `type:chore`.
- One pull request per issue, referencing it; CI required; squash merge into `main`.
- The player wiki (`wiki/`, mdBook, published at `/wiki/` on GitHub Pages with each release) is
  updated in the same PR as any change to gameplay mechanics or world facts.

## 14. Deferred and open

- Mouse support (hover-inspect first)
- Infravision
- Audio
- Localization
- Performance on an integrated GPU: the frame budget is estimated from offscreen timing on the
  development Mac only; measuring it on an iGPU is deferred until such hardware is at hand (#74)
