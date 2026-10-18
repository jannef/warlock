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
- Test creatures walking fixed patrol routes; from M10 they act at their own hidden speed (§8
  "Speed"), belong to factions (§7 "Factions") that are hostile or neutral to the player and to
  each other, hear noise, see by the player's rules (§5 "Creature sight"), and fight with
  deterministic AI whose attacks are intents only (§8 "Creature behavior")
- Pickable objects (glyph, color, material, weight), inventory, and creature weight (M11)
- Moons with phases and moonlight, weather starting with rain (M12)
- Action costs: what an actor does and its speed decide how long it takes (M13)
- Stats (M14): Strength, Dexterity, Toughness, Intelligence, Willpower, and Presence, inspired by
  D&D, with D&D 3.5 modifiers and ADOM-like potentials that the player never sees; every actor
  has them, and the player sees their own scores on a character screen (§8 "Stats"). No rule
  uses them yet, and no other D&D concept comes with them
- Save and resume an unfinished run
- Keyboard controls with an in-game controls reference
- Development interface (§9)
- Procedural generation with reproducible seeds (M6): dungeon branches of generated levels
  (rooms and corridors, caves) with hand-made vaults, joined to the hand-authored levels by stairs
- An overworld with travel time, locations, local areas, and day and night (M9)

**Out of scope for now**: combat resolution (attacks are announced, nothing is damaged), rules
that use stats (ability checks, saving throws, hit points, carrying capacity, training), item
effects, character creation, audio, mouse input, infravision, localization.

Vision types are in scope even though there are no other mechanics yet: they are part of perception,
and the light data model must carry them from the start.

## 3. Showcase world

From M1 on, the playable build is a hand-authored **showcase world** where `@` walks through one room
per engine capability. Each milestone adds or upgrades rooms. The showcase world is also the
regression scene for golden screenshots, replays, and snapshot tests.

**Level 1 — Hub (80×21)**

| Room | Demonstrates | Milestone |
|---|---|---|
| Start room | movement, message line, `?` help; gold and a small pile to pick up | M1, M11 |
| Door corridor | opening/closing doors, doors blocking sight; a storeroom and a pantry of objects | M1–M2, M11 |
| Patrol gallery | creatures moving in and out of view, never shown in memory; a cave beetle and a quicker cave bat (wildlife, so they keep to their lanes) on parallel lanes, a slower torch-bearing kobold circling them (carrying a sword and coins), which comes for the player once it sees them, searches where it lost them, and returns to its round; heard from the corridor behind its closed door, and in its dark middle (the bat flies silently) | M2, M10, M11 |
| Light hall | colored lights, falloff, pillar and creature shadows, steady magical light | M3, M8 |
| Darkness chamber | vision types, a UV-only rune, darkvision fade; an amulet seen only by those who see in the dark | M3, M11 |
| Water garden | still ponds and running rills (flow, glints, foam), pulsing altar with a ring before it | M4, M8, M11 |
| Effects room | glyph particles, glow, fades, distortion | M4 |
| Stairs | connection to Level 2 and the arena (down), the courtyard and the overworld (up) | M2, M9, M10 |

**Courtyard (66×19, outdoor)**: up the stairs nook's `<`, a walled garden under the open sky —
trees, a ruined wall, a pond, standing stones, flower beds, and a roofed gazebo with a lamp — where
sunlight, sky light, and shadows turn and stretch with the time of day (§5 "Sunlight"; M9), and the
moon lights it faintly by night, waxing and waning with the month (§5 "Moonlight"; M12). The
`courtyard` scenario starts there at 06:30, while the shadows are long. Windfall apples lie under
a tree, and a candle and a scroll in the gazebo (M11).

**Arena (72×21, M10)**: down the stairs nook's second `>`: factions at war. A kobold war band
(three torch-bearing kobolds, an archer, a slow shaman) and five giant rats meet at a fire pit in
a pit lit by wall torches, with pillars for cover; two cave bats and a cave beetle (wildlife) go
their ways untouched. West lies the kobolds' camp behind two doors (kobolds open doors, rats do
not), east the rat warren. The player arrives in the dark south-west corner and can watch unseen;
each creature fights the nearest foe it sees, so coming close (lit, or within a kobold's
darkvision) turns them on the player. Nothing dies, so the fight settles into a brawl that
goes on: melee pairs attack each other turn after turn, the archer shoots and the shaman casts
over them. The `arena` scenario starts there.

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
locations 30) at normal speed; every other action there (waiting, entering a location or a local
area) takes its usual time, 6 s for a normal action (§8 "Time"); mountains and open water are
impassable, and the river Wendle (drawn flowing, with legend arrows like running water; §4.3) can
only be crossed by the bridge. Named regions (Greenmarch, Thornwood, Barrowdowns, Hollowcrag, Silvermere, Sallow
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
- **Day and night**: sight on the overworld depends on daylight and moonlight (§5), and the sun
  and the sky light the overworld and its local areas by day (§5 "Sunlight"), the moon by night
  (§5 "Moonlight"); at night the overworld is drawn by a faint cold light with the moonlight on
  top, while local areas are dark but for the moon.
- **Weather** (M12): clear, overcast, rain, or heavy rain over the overworld and its local areas
  (and the courtyard), changing every few hours by the month's chances; clouds dim the sun, the
  sky, and the moon, rain shortens sight on the overworld, and faint rain falls on what the player
  sees (§5 "Weather", §4.8).
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
   perception state) drawn as one quad with a custom 2D material. Terrain, remembered terrain,
   objects (the top object of each pile in place of the terrain's glyph, over its background, lit
   like terrain; a cell with objects does not animate), and in-cell animation.
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
  every visible source's per-cell contribution, the sun's and the moons' included — into a
  viewport-sized float texture aligned with the map grid. The shader interpolates it per pixel
  between cell centers for smooth gradients; a neighbor contributes only if it is visible and
  transparent, so light never bleeds across walls or corners, nor into unseen or remembered cells.
  Walls show their own wall-face light, flat.
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
  The sun and the moons (§5 "Sunlight", "Moonlight") have no preset, so they are never modulated
  (no flicker or hue drift), and they are directional: a creature, one cell tall, shades the
  cells along the body's shadow line from its cell (the line terrain shadows use, up to the first
  opaque cell), 40 % next to it easing back to all of the light one step past the shadow's end — so
  creature shadows are long at dawn and dusk (or under a low moon) and short at noon.
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
- **Remembered** — dim, desaturated terrain and objects as last seen. Never creatures, effects, or
  current light.
- **Unknown** — blank.

### 4.6 Motion and time
- Visual time runs continuously while gameplay waits for input. Animation never blocks or changes
  gameplay.
- Moving glyphs slide between cells over ~80 ms; the camera follows smoothly. A creature that
  steps several times in one turn (one quicker than the player, §8 "Speed") slides along the cells
  it passed in one motion, a little longer (√steps × 80 ms). A creature attacking in melee (the
  player too) lunges a fifth of a cell toward its target and back over 160 ms. Any new input
  completes running animations instantly, so fast play never waits.
- Visual randomness uses its own RNG and never consumes gameplay randomness.

### 4.7 Readability and reduced effects
- Glyphs stay legible under every effect: minimum contrast against their background, glow added
  around glyphs rather than over them, small distortion amplitudes.
- A **reduced effects** setting is a preset that turns off glyph animation, flicker (light
  stability: light is steady), sliding (instant snap), camera easing, bloom and glow, distortion,
  particles, screen fades and flashes, and falling rain. Each can also be switched off on its own
  (settings: glyph animation, light flicker, motion = slides and camera easing, bloom and glow,
  distortion, particles, screen fades, weather); an effect runs only when its switch is on and
  reduced effects is off. Rain has a switch of its own rather than riding on particles: it moves
  over the whole view, not around a few sources. The weather itself (its light, the status bar)
  never depends on it. No gameplay information may
  depend on an effect: heard sounds keep a still marker when their ripple (glyph animation) is
  off (§5 "Hearing"). These are settings (§6); the `--reduced-effects` flag forces everything off.
- Colors that effects introduce, such as the shimmer target color, go through the active palette
  like any other color.
- **Color palettes** (default, high contrast, a red-green colorblind variant) are transforms
  applied when colors are written to the screen, so game data colors are never altered. UI text
  keeps at least 4.5:1 contrast, and the UI never conveys information by color alone (selection
  has a `>` marker, refusals start with `!`).

### 4.8 Effects
Declared in data per terrain or creature type (and per weather), like glyph animation. They are
presentation-only: gameplay never reads them. Effects are deliberately subtle — sparse, short-lived particles, gentle
glow, barely perceptible distortion, short fades.
- **Glyph particles** on the free-glyph layer. An emitter has glyphs to pick from, colors over the
  particle's life, a lifetime, velocity ± spread, gravity, a start offset and spawn area, sideways
  sway, scale, fade-out, glow, and opacity (how opaque it is at most, default 1). Emitters fire `ambient` (a steady rate while the source is in
  view: sparks over braziers and torches, embers over lava, fountain spray, dust motes), on `step`
  (a burst when a creature is seen entering the cell: a splash in water), on `toggle` (a burst
  when a door is seen opening or closing: dust), or at an `obstacle` (a steady rate from visible
  running water whose flow runs into a known cell that is not running water: foam at banks and
  boulders, ripples where a stream enters a pool; the emitter is turned with the flow, its `y`
  pointing downstream). Creature emitters move with the creature; a creature's `attack` emitters
  burst at the target of each attack the player sees (the kobold shaman's faint violet sparkle).
- **Shots**: a ranged attack the player sees sends one glyph (`-`, `|`, `/`, or `\`, pointing
  the way it flies; pale wood, lit like free glyphs) from the edge of the shooter's cell to the
  edge of the target's at 24 cells a second (at least 60 ms), fading in and out at the ends.
  Shots and attack bursts run with the particles switch; with frozen visual time a shot is shown
  halfway.
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
- **Weather** (§5 "Weather"): while it rains where the player is, faint drops fall on every visible
  cell under the open sky (never on a roofed cell), small splashes rise from visible open ground,
  and rings spread on visible still water; heavy rain falls thicker. They are particle emitters of
  the weather (`data/weather.toml`, §7), ambient per cell and keyed by the cell's level position, so
  the rain stays put while the camera scrolls; a drop is drawn only while it is over a cell it
  rains on. Drops are thin strokes (`│`) sheared along their fall (the free-glyph layer's `slant`),
  slightly slanted, short-lived, and at most about half opaque (`opacity`); they are lit like other
  particles, so they vanish in the dark and catch a lantern's light. At most 3000 rain particles
  are drawn a frame. The darker, cooler look of a rainy day is the gameplay light itself, not an
  effect. Off with the `weather` switch.
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
  sources that reach it (the sun and the moons included) plus bounce light (saturating).
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
    neither; the moons light the land (below).
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
- **Moonlight**: the moons (`data/sky.toml`, §7; one by default) light the same levels by night.
  Without moons the night is dark.
  - **Phase**: a moon's age is the time since its last new moon (`period_days` from new to new;
    `phase_offset_days` its age at the calendar epoch). The default moon has a 30-day period, so its
    phase follows the month: new on the 1st, first quarter on the 8th at noon, full at midnight
    starting the 16th, last quarter on the 23rd at noon. Its phase is named in eighths of the cycle
    (new moon, waxing crescent, first quarter, waxing gibbous, full moon, waning gibbous, last
    quarter, waning crescent).
  - **Path**: a moon walks the sun's path (the same per-minute table), trailing the sun by its
    phase of a day: a new moon rises and sets with the sun, a first quarter rises at noon and
    stands high at dusk, a full moon rises at sunset and stands high in the south at midnight, a
    last quarter rises at midnight. On a 30-day cycle it rises about 50 minutes later each day. Its
    direction, elevation, and shadows are the sun's at that minute of the path.
  - **Light**: a moon's direct light (`color`, shaded by tall terrain exactly like sunlight, and
    never under a roof) and its sky light (`sky`, on every cell under the sky, shaded or not, part
    of the base light), both scaled by the lit fraction of its disc (`(1 − cos phase) / 2`: none at
    new moon), its elevation (fading in over its first degrees: 15 % at 2°, 40 % at 6°, 65 % at
    12°, 90 % at 25°, all from 40°), the daylight (none while the sun is up, all once the sun is 6°
    below the horizon, so it fades in through the evening twilight), and its size (the square of
    its apparent size over a standard moon's). Integer math only (the sun's fixed-point cosine);
    a test pins a hash of the default moon over a whole cycle. Several moons add up, each with its
    own phase, place in the sky, and shadows; each is one more source with a stable key and no
    flicker.
  - The default moon is faint and slightly purple: full and high, `[48, 28, 66]` direct and
    `[11, 7, 15]` sky light (brightness 35 and 8, plus bounce), so normal eyes dimly make out open
    ground in its light but not its shadows, low-light eyes see by a half moon too, and a new moon
    gives nothing.
  - Computed like sunlight, per moon that shines, over the same area, and not cached.
- **Weather** (`warlock-core/src/weather.rs`; M12): one weather over every level under the open
  sky — clear, overcast, rain, or heavy rain — a pure function of the run seed and the time, so
  skipping time and resting need no history.
  - **Blocks**: time is cut into blocks of about 3 hours; block `k` starts at `k · 3 h` plus a
    shift of 0 to 90 minutes drawn for it (so blocks last 1½ to 4½ hours and the weather does not
    change on the hour). Each block has one weather.
  - **Which weather**: each block draws a number uniform in `[0, 1)` (FNV-1a over the seed, a
    label, and `k`, mixed by SplitMix64's finalizer; 16 bits). A block's value is the sum of its
    own and the 3 previous blocks' draws mapped through the sum's distribution (the Irwin–Hall
    CDF, integer math), which makes it uniform again while neighboring blocks share most of their
    value: weather lasts (in Rainmoon a wet spell lasts about 12 hours on average) and mostly
    changes to a neighboring kind. The value picks the weather by the chances of the month the
    block starts in (`data/weather.toml`, §7; ordered clear, overcast, rain, heavy rain, the lowest
    values the clearest), so each weather comes up exactly as often as its chance. Rainmoon is
    wet (35 % rain, 15 % heavy), Highsun dry (70 % clear).
  - **Clouds** dim the light of the sky, applied where the sky's light is assembled
    (`Sky::at`): `direct` percent of the direct light of the sun and the moons gets through,
    `sky` percent of the sky light (the sun's and the moons'), and the clouds spread `scatter`
    percent of the direct light over the whole sky as more sky light, so shadows grow faint and the
    light flat: overcast 35/85/15, rain 22/70/12, heavy rain 14/55/8 — at noon in the open 57 %,
    41 %, and 28 % of a clear sky's light, and a full moon's light 58 %, 44 %, and 30 %. Channels
    are rounded up, so a sun or moon that shines at all still shines behind the clouds (the sun
    stays up, a moon still casts its shadows). Where the weather changes, the clouds change
    linearly over an hour centred on the change, so the light dims and brightens smoothly; the
    weather itself (messages, status, rain) changes at the block's start. Integer math only.
  - **Perception**: the player knows the weather only under the open sky (it is part of the sky in
    the snapshot); not underground. A change the player sees there is a message ("Clouds
    gather.", "It starts to rain.", "It starts to pour.", "The rain grows heavy.", "The rain
    eases.", "The rain stops.", "The rain stops and the sky clears.", "The sky clears."), at most
    one per change (changes are at least 1½ hours apart); arriving under the sky says nothing (the
    status bar shows it). Rain starting or growing heavier stops a rest; the other news is logged
    without stopping it.
  - **Dev override**: the action `weather:clear|overcast|rain|heavy` sets the weather (with its
    full clouds, at once) for 3 hours of game time from the moment it is given; then the weather
    of the day returns. It takes no time, is recorded in replays, and is kept in the world state
    (saved, hashed) until it runs out. Captures and tests use it.
  - Weather is the same over the whole land for now; snow, fog, and rain masking sounds (hearing)
    may come later.
- **Brightness** is the integer luma of RGB, `(54·r + 183·g + 19·b) / 256`, plus UV for eyes that
  see UV. Eyes without UV sight do not perceive UV at all (perception reports it as 0).
- **Visibility**: a cell is visible iff it is in line of sight within the maximum view range **and**
  (its perceived brightness ≥ the eye's threshold **or** its distance ≤ the darkvision radius). The
  player's own cell is always visible. Creatures are seen iff their cell is visible: creatures in
  darkness are hidden (unless within darkvision), creatures carrying light are seen.
- **Overworld sight** is at map scale: a cell of the overworld is visible iff it is in line of
  sight within the daylight radius: 9 by day (06:30–17:30), 5 in twilight (half an hour either side
  of 06:00 and 18:00), at night 2 plus one cell per 20 of moonlight (the brightness of all moons'
  direct and sky light together on open ground, through the clouds), at most 4: a full moon high in
  the sky gives 4, a half moon 3. Rain then takes its `sight` away (2 cells in rain, 4 in heavy
  rain; "Weather" above), never below 2. Light, the eye's threshold, and darkvision play no part.
  Mountain ranges and dense forest are opaque; forest is not.
- **Overworld night light**: the overworld is lit by the sun and the sky like any level under the
  open sky ("Sunlight" below), and never gets darker than a faint cold light (`[48, 56, 90]`,
  channel by channel, added to its ambient light where sun and sky fall short; moonlight comes on
  top): it only draws the cells in sight at night apart from the remembered map, since sight there
  ignores light. Local outdoor levels get no such light: at night only the moon lights them (bring
  a lantern).
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
- **Objects are seen** where creatures are: on cells in line of sight that are visible (never on
  completed walls). The player's own cell is always visible, so they always know what lies under
  them. What creatures carry is never seen.
- **Memory**: per level, terrain and piles of objects, as last seen (a door is remembered open
  until seen closed; a pile is remembered as it lay when last in sight, even if it changed out of
  sight since, and is forgotten once seen gone). Never creatures.
- **Recomputation**: perception updates in the perception phase of every turn (and after
  `SetVision`). Per-light shadow masks are cached and invalidated when the light moves or opacity
  changes within its radius; only sources whose radius can reach the view range (or a surface whose
  bounce light reaches it) are evaluated. Bounce light, sunlight, and moonlight are computed over
  the bounding box of the cells in line of sight plus bounce's 3-cell reach, so they do not depend
  on where the viewer stands. Sunlight and moonlight are not cached (they move every minute).
- **Hearing** (`warlock-core/src/noise.rs`; M10). Integer and deterministic, like sight:
  - **Noise**: actions make noise. A step makes the walker's `footsteps` (creature data: the
    player 60, kobold 80, kobold archer 70, kobold shaman 60, giant rat and deer 50, cave beetle
    30; bats fly silently) scaled by the `step_noise` of
    the terrain stepped onto, in percent (water and running water 200, marsh, bushes, and rubble
    150, reeds 130, gazebo boards and dry grass 120, tall grass and rocky ground 110, dirt road
    90, forest floor 80, grass, flowers, and dusty floor 70; default 100). Opening a door makes
    the closed door's `toggle_noise` (60), closing one the open door's (70). One function decides
    an action's loudness, so skills such as moving silently can change it there later. Loudness
    is at most 300; a creature without `footsteps` moves silently, a door without `toggle_noise`
    is silent. (Footsteps may scale with a creature's weight once it has one, M11.) Attacks
    (intents, §8 "Creature behavior") are heard where the attacker stands: fighting (melee, 60), a
    bowstring (a shot, 40), chanting (a spell, 50).
  - **Propagation**: a noise spreads from its cell through the 8 neighbors, losing 10 loudness
    per orthogonal step and 14 per diagonal one, plus the `muffle` of every cell it enters (a
    closed door 30). Solid terrain (neither walkable nor transparent: walls, rock, trees,
    boulders) blocks sound unless it has a `muffle`, and a diagonal step needs one of the two
    cells beside it to let sound through unmuffled. So sound goes around corners but never
    through walls or past a closed door's corner. The loudness at a cell is that of its loudest
    path; the search is bounded by the loudness (at most 30 cells) and by the quietest listener
    in reach, so its cost does not depend on the level size. The overworld is silent (its steps
    are journeys).
  - **Listeners**: every creature type with `hearing` (the faintest loudness it hears: the
    player 10, kobold 15, kobold archer 12, kobold shaman 10, cave beetle 20, giant rat and deer 5,
    bat 3; none is deaf) hears a noise another actor
    makes on its level when the loudness at its cell reaches that, at once, as the action
    happens. A creature keeps what it heard (kind, where it was made, how loud it was at the
    creature) until it next acts: the input its AI reads (§8 "Creature behavior": it goes to look
    where it heard a noise whose source it does not see); the list is part of the world state.
  - **The player** perceives the sounds they heard during the turn that just passed whose source
    cell they do not see (a creature in darkness or out of sight, a door around the corner),
    never who or what made them: the kind (footsteps, a door opening, a door closing, fighting, a
    bowstring, chanting), how loud
    (faint, clear, or loud: under 10, under 30, or at least 30 above their threshold), and
    roughly where: a cell up to 2 (faint), 1 (clear), or 0 (loud) steps from the source through
    cells that pass sound unmuffled, which the player does not see, picked by a hash of the
    source cell and the turn (deterministic, not the source more often than its neighbors).
    Sounds whose source the player sees are not perceived: the player sees what happens. A quick
    creature's steps within one turn are one sound (sounds of one kind made within 2 cells of a
    later one that turn are left out).
  - **Messages** for sounds that are news: "You hear footsteps to the north-east.", "You faintly
    hear a door close to the west." (the direction of the rough place, of eight). A perceived
    sound is no news if one of the same kind was perceived within 4 cells plus 2 per turn since,
    in the last 10 turns (a creature takes up to two steps a turn), so a patrol heard turn after
    turn is said once per pass, not every turn. Like any message they stop a rest.
  - **On the map**, each perceived sound is a faint, still marker (`◌`, pale grey, fainter for
    fainter sounds, never lit) at its rough place until the next turn; with glyph animation on
    it ripples in, a ring growing out of it and fading within a second. Look mode describes it
    ("You hear footsteps here.").
- **Creature sight** (`warlock-core/src/sight.rs`; M10): a creature sees by the player's rules,
  with its own eyes (its type's `vision`, a profile above) and radius (its `sight`, default 10,
  at most 40): an actor is seen iff its cell is in the creature's line of sight (the same
  symmetric shadowcasting) within its sight radius and lit at least to its eye's threshold by the
  same gameplay light the player sees (ambient, sky, every source, the sun and the moons through
  the weather's clouds, bounce), or within its darkvision; on the overworld, line of sight within
  its sight and the daylight radius (moonlight and rain included). (An actor
  on an opaque cell is seen only within darkvision; no such terrain exists off the overworld.)
  Creatures never read the player's perception: each computes its own, and only for the cells
  where a hostile stands, nearest first, stopping at the first it sees: line of sight only out to
  that distance, light only where darkvision does not settle it (and bounce light only when the
  rest is not enough), so the cost follows the hostiles nearby, not the level. A test checks that
  on every cell the player sees, the light a creature with the player's eyes finds is the
  player's. So a creature carrying a torch is seen and sees, a player in the dark is unseen by
  eyes without darkvision, and lighting the lantern gets the player noticed.
- **Perception API**: the snapshot exposes, for visible cells only, the perceived light (after the
  wall-face rule; plus whether the cell is seen only through darkvision and its squared distance),
  the light sources whose light reaches visible cells (stable key; emitter: a point light's
  position and preset, or the sun or a moon with its shadow direction; and per-cell
  contributions; contributions plus the cell's base light sum to the cell's light), the base light
  of visible cells, the active vision profile, the ambient light, on a level under the sky the
  sun, the moons, the sky, and the weather of the moment, and the perceived sounds (kind, volume, rough place;
  "Hearing" above). Visual lighting (§4.4) is derived from this.

## 6. Input and UI

- Keyboard-first. Default sets: vi-keys (`hjklyubn`), numpad, arrows (4 directions).
- Symbol commands (`<`, `>`, `?`) are bound by **logical key**, so they work on non-US layouts.
- Bindings live in the settings file. The `?` help (the controls list, read-only) is generated from
  the binding table, so it is always accurate.
- **Screens**: the main menu, New run, pause menu, Settings, Controls, help, the inventory, the
  character screen, and choosing what to pick up or drop are modal screens on one stack. Opening a screen puts it on top; Esc or Back closes it and returns to the screen
  below as it was left. While any screen is open it receives every key press, the run (if any)
  stays visible beneath it, and no game time passes. Prompts in the message line (a direction
  to open or close, how long to rest, look mode) are not screens.
- **Main menu** (drawn in glyphs; the game opens on it unless `--scenario`, `--new`, `--replay`,
  `--record`, or `--capture` name what to run): Continue (only if a save exists), New run
  (scenario list from content, optional seed), Settings, Controls, Quit.
- **Pause menu**: Esc during a run (when no prompt is open) opens it, drawn in glyphs:
  Resume, Settings, Controls, Save and quit to menu, Quit (which saves). No game time passes while
  it is open; Esc or Back in Settings/Controls returns to it.
- **Settings** (`settings.toml` in the platform config dir; on the web, browser `localStorage`, §7): font size (logical
  px, 12 to 36), reduced effects and the per-effect switches, color palette, key bindings. A missing or damaged file means
  defaults and a logged warning. Changes apply live (a font size change rebuilds the glyph atlas)
  and are written immediately.
- **Rebinding**: in Controls, select a command and press a key; it becomes the command's primary
  key (alternates such as the numpad stay). A key another command uses is refused and the owner
  is named, so the table never holds a conflict. Esc (cancel) is fixed. Delete resets a command,
  "Reset all keys" resets everything.
- First-phase commands: move, wait, open/close door, ascend/descend, light/douse lantern (`L`),
  pick up (`,` or `g`), drop (`d`), inventory (`i`), character (`@`), rest (`R`), look, help,
  save and quit (`Q`). Free actions (look, help, the inventory, the character screen, menus,
  vision switch) do not advance time.
- **Objects** (M11) lie in piles on cells, the top first: the map shows the top object's glyph
  (§4.2), and dropped objects go on top. Stepping onto a pile says what lies there: `You see here
  a dagger.` for one stack, `There are several objects here.` for more. Look mode names what lies
  on a cell, seen or remembered: `a dagger`, `a dagger and 12 arrows`, up to three stacks, then a
  count (`17 objects`). A stack is some number of one object: stackable objects (coins, arrows,
  potions) join a stack of the same object wherever they go; any other object is a stack of one.
  A door does not close on objects in its doorway.
- **Pick up** (`,` or `g`) takes a lone stack at once. Where several lie, a screen lists them by
  letter (`a` the top) with glyph, name, and weight: a letter marks or unmarks its stack (`+`
  marked, `-` not), `,` marks everything, the arrows move the `>` cursor and Space marks its row,
  Enter picks up the marked stacks (the cursor's row if none is marked), Esc cancels. **Drop**
  (`d`) chooses from the inventory the same way. Picking up and dropping are one action each
  (replay text `pickup:ac`, `drop:b`; `pickup` and `drop` alone take everything) and take their
  action's time (§8 "Time"); with nothing there or nothing carried, they are refused and take
  none.
- **Inventory** (`i`): what the player carries, ADOM-like: stacks lettered `a` to `z` in the order
  they were picked up (at most 26; a pile with more stacks keeps what does not fit), each with its
  glyph, name, and weight, then what the player carries and their weight (their own plus what
  they carry). Read-only; `i` or Esc closes it.
- **Character** (`@`): who the player is, as far as they know it: their name (the player
  creature's, "You"), their score in each stat (§8 "Stats") in display order with its
  abbreviation and full name (`St  Strength  12`), and their weight (their own plus what they
  carry). Never a modifier or a potential, which the player does not know. Read-only; `@`, Enter,
  or Esc closes it. The status bar does not show stats: it is full at common window sizes.
- **Weight**: every object and every creature (the player included) has a weight in grams. An
  actor weighs its creature's base weight plus everything it carries; the rules know every
  actor's weight (no rule uses it yet; footstep noise, §5 "Hearing", may), but the player is shown
  only their own.
  Creatures carry objects from their spawn (`carry`, §7); nothing makes them drop anything yet.
  Weights read `450 g` below a kilogram and `3.5 kg` above.
- **Rest** (`R`) asks in the message line: `Rest: (h)our, until (d)awn, until (n)ightfall? (Esc to
  cancel)`. `h` rests an hour, `d` until 06:00, `n` until 18:00 (replay text `rest:1h`,
  `rest:dawn`, `rest:dusk`); any other command cancels. Resting is one action that passes turn
  after turn and stops early when a creature comes into view or a message is logged (§8 "Time"),
  such as a sound that is news (§5 "Hearing") or rain starting or growing heavier (§5 "Weather");
  a sound already known goes on without stopping it, and so does other news of the weather (the
  rain easing or stopping, clouds gathering or parting), which is logged all the same.
  It is a gameplay input mode like the open/close direction prompt, not a menu screen.
- **Messages**: the last two show in the message line; the same text logged again in a row folds
  into one entry with a count (`You open the door. (×2)`). Heard sounds are logged only when they
  are news (§5 "Hearing"), so a patrol walking back and forth out of sight gives one message per
  pass, not one a turn.
- **Attacks** (intents only, §8 "Creature behavior") are reported when the player perceives them:
  they see the attacker or the target, or are the target. "The kobold attacks you.", "The kobold
  archer shoots at the giant rat.", "The kobold shaman casts a spell at you."; an end the player
  does not see is "something" ("Something attacks you.", "The kobold archer shoots at
  something."). Attacks the player perceives nothing of leave no message (and no event). Moving
  into a hostile creature attacks it ("You attack the giant rat.", or "You attack something."
  when it is not seen; it takes a turn); into a neutral one stops ("The deer is in the way.",
  "Something is in the way."; no time passes).
- **Look mode** describes what the player perceives at the cursor: a creature and its terrain
  (a hostile one marked: "A giant rat (hostile), on stone floor."; its type tells the player its
  faction's stance toward them, the only faction information presentation gets), terrain,
  remembered terrain, where stairs lead, and a sound heard there ("You hear footsteps here.").
- Screen layout: map viewport, message log, status panel (level, region (on the overworld and in
  its local areas), time of day and date as `14:32 · 12 Rainmoon, Year 1444`, the moon's phase as a
  glyph and its name (`◑ First quarter`; with several moons each moon's name and glyph, `Moon ●
  Ilse ☽`), the weather under the open sky (`Clear`, `Overcast`, `Rain`, `Heavy rain`; §5
  "Weather"), turn, position, light at the player, vision profile, lantern, seed). The phase glyph
  shows the lit part of the disc, lit on the right while waxing: `○` new, `☽` waxing crescent,
  `◑` first quarter and waxing gibbous, `●` full, `◐` waning gibbous and last quarter, `☾` waning
  crescent. When the window is narrow, segments give way in priority order — seed, position,
  lantern, vision, light, turn, then the moon's phase (first cut down to its glyph), region, date
  (first cut down to day and month, `12 Rainmoon`), weather, and time, the level name last — and
  segments that fit again in the room left take it back, most important first. Outdoors the
  weather outranks the date: it changes the light and how far one sees.

## 7. Levels and data formats

Rule: **hand-authored data is TOML; machine-written data is JSON.**

- **Level files** (TOML): a header (name, ambient light, legend, lights, patrols, stair links,
  and how time passes: `kind = "dungeon"` (default), `"outdoor"` (under the open sky, lit by the
  sun and the sky; §5 "Sunlight"), or `"overworld"` (also under the sky, where a step's base time
  is the terrain's travel time; §8 "Time"; a level has no turn length of its own); optional
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
- **Objects in level files** (M11): a legend entry places a pile with `objects = ["dagger",
  "arrow*12"]` (top first; `*N` places N, as one stack if the object is stackable, else as N
  stacks of one), and a creature's entry gives what it carries with `carry = ["spear",
  "copper_coin*7"]` (at most 26 stacks). `[[pile]] at = [x, y] objects = [...]` places a pile
  without a legend character (on top of the legend's there, if any). Objects lie only on walkable
  terrain. Vaults place objects the same way, and generators stamp them with the vault.
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
- **Definitions** (TOML): terrain types, creature types, light presets, vision profiles, glyph
  animations, and effects (glow, heat haze, particle emitters; §4.8). Creature types have a
  `speed` (100 to 10 000, default 1000, the player's; never shown to the player; §8 "Speed"): the
  giant rat 1200, the cave bat 2000, the kobold 900, the deer 1300; and for hearing (§5)
  `footsteps` (at most 300; default 0, silent) and `hearing` (the faintest loudness heard, 1 to
  300; default none, deaf). For behavior (§8 "Creature behavior") a creature type names its
  `faction` (below; required when factions exist, the player's is `player`), its eyes `vision`
  (a profile; default `normal`) and `sight` (radius, 1 to 40, default 10), how it fights:
  `combat = "melee"` (default), `"ranged"`, or `"caster"`, the latter two with a `range` (2 to its
  sight), `opens_doors` (default false), and optionally `attack_cost`, what its attacks cost
  instead of the attack's cost in `data/actions.toml` (100 to 10 000; §8 "Time"; none of the
  built-in creatures has one yet). The kobold: darkvision, sight 10, melee, opens doors;
  the kobold archer: darkvision, sight 12, ranged 7; the kobold shaman (speed 800): darkvision,
  caster 5; the giant rat: low-light, sight 8; the cave bat: darkvision, sight 8; the cave
  beetle (1200): darkvision, sight 6; the deer: normal, sight 12. Its stats (§8 "Stats"; never
  shown): `stats`, its scores by stat id (`stats = { strength = 9, dexterity = 13 }`; ids
  `strength`, `dexterity`, `toughness`, `intelligence`, `willpower`, `presence`; 1 to 40, a
  missing one 10), and `potential`, the same for its potentials (1 to 40, not below the score;
  a missing one equals the score). The player creature has none: the scenario gives the
  player's. The test creatures have D&D-like scores (the kobold St 9 Dx 13 To 10 In 10 Wi 9
  Pr 8, the giant rat Dx 17 In 1, the cave bat St 1, the shaman Wi 14). Creature particle emitters may
  fire `on = "attack"`: a burst at the target of each attack the player sees (the shaman's
  violet sparkle). Terrain has a `move_cost`: what a step onto it costs in percent of a normal
  step (the `step` action; default 100; still and running water 200; local levels only), a
  `bounce` tint for the light it reflects (§5), a `height` in cells for sun shadows (default 1 if opaque, else 0; at most 4),
  `roofed` (never lit by the sun or the sky), and for hearing a `step_noise` (how loud steps onto
  it are, in percent of the walker's footsteps, 0 to 1000; default 100), a door's `toggle_noise`
  (how loud opening the closed door or closing the open one is, at most 300; default 0, silent;
  only doors), and a `muffle` (the loudness a sound loses entering the cell, at most 300; default
  0, but solid terrain blocks sound unless it has one). Overworld terrain has
  `travel_minutes` (1 to 1440; required for every walkable terrain on the overworld),
  optionally `local`, the ground terrain of the local area entered from a cell of it (none:
  nothing to explore there; locations never have one), and `landscape`, what its land is like in
  local areas (required with `local`): `plains`, `grassland`, `forest`, `dense_forest`, `hills`,
  or `swamp` pick the generator of its own areas; `road` runs a road through them amid the
  neighbors' land; `river` and `lake` put running or still water along that side of the
  neighboring areas (§3). The calendar's month names are
  `data/calendar.toml` (exactly 12; without the file months are numbered).
- **Action costs** (`data/actions.toml`, optional): action ids mapped to their cost in energy,
  e.g. `wait = 1000`, 1000 being a normal action (6 s at normal speed; §8 "Time"), 100 to
  10 000. The ids are `step` (onto terrain of `move_cost` 100), `wait`, `open_door`,
  `close_door`, `lantern`, `stairs` (also entering a location or local area and walking off an
  area's edge), `pick_up`, `drop`, `melee`, `shoot`, and `cast`. An action left out, or every
  action without the file, costs 1000; an unknown id or a cost out of range is a content error.
  The wiki shows them in a generated table.
- **The sky** (`data/sky.toml`, optional; without it there are no moons): up to 8 `[[moon]]`s,
  each with `name` (shown in the status bar when there are several), `color` (`[r, g, b, uv]`,
  the direct light of the moon when full and high in the night sky), `sky` (the same for the
  light of the sky around it; default none), `size` (apparent size in percent of a standard moon,
  1 to 400, default 100; its light scales with the square), `period_days` (new moon to new moon,
  1 to 3600), and `phase_offset_days` (its age at the calendar epoch, less than the period,
  default 0). §5 "Moonlight".
- **Objects and materials** (M11): every object (`[[object]]` in `data/objects.toml`) defines the
  symbol it renders with and its color, material, and weight: `id`, `name` (singular, without an
  article; `plural` defaults to the name plus `s`), `glyph`, `fg`, `material` (an id in
  `data/materials.toml`, whose `[[material]]` entries have an `id` and a `name`; properties come
  later), `weight` (integer grams, at least 1), and `stackable` (default false). Glyphs follow
  ADOM loosely: `)` weapons, `}` missile weapons, `/` missiles, `[` armor, `(` tools, `!`
  potions, `?` scrolls, `%` food, `*` stones and gems, `$` coins, `=` rings, `"` amulets.
  Every creature in `data/creatures.toml` has a base `weight` in grams (required, at least 1).
- **Factions** (`data/factions.toml`, optional; without it everyone is neutral): `[[faction]]`
  with `id`, `name`, and `hostile` (faction ids). Relations are pairwise and symmetric: if A lists
  B, A and B are hostile to each other; every pair not listed (a faction and itself too) is
  neutral. Validation: unique ids, known ids, no faction hostile to itself, a `player` faction
  (the player's), and every creature type but the player naming a known faction. The core API
  gives the stance (`Hostile` or `Neutral`) of two factions, creature types, or actors. Test
  factions: `vermin` (giant rats; hostile to the player), `kobolds` (the war band; hostile to the
  player and the vermin), `wildlife` (deer, cave bats, the cave beetle; neutral to everyone).
- **The weather** (`data/weather.toml`, optional; without it the sky is always clear): exactly 12
  `[[month]]`s in calendar order, each with the chance in percent of `clear`, `overcast`, `rain`,
  and `heavy_rain` (adding up to 100), and a table for each of `overcast`, `rain`, and
  `heavy_rain` (clear is the sky as it is): `direct` and `sky` (percent of the direct light of the
  sun and the moons, and of the sky light, that gets through; 1 to 100), `scatter` (percent of the
  direct light the clouds spread over the sky as sky light, 0 to 100, default 0), `sight` (cells
  of overworld sight lost, 0 to 8, default 0), and presentation-only `ambient` particle emitters
  (§4.8; `rate` per cell and second): `drops` (on every visible cell under the sky), `splash` (on
  open ground: walkable, transparent, not running water), and `ripples` (on still water, terrain
  with a `wave` animation). §5 "Weather".
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
  (terrain, lights, objects, creatures) where they fit — into rock with room for corridors or tunnels to
  every entrance. A vault's edge is solid except for its entrances (doors, not on a corner); it has
  no stairs or player start, and all its walkable cells are reachable from an entrance. Vaults of
  `kind = "outdoor"` are ruins for local areas, stamped on open land with a ring of open ground
  around them: their spaces (`void`) keep the area's own ground, every open cell on their edge is
  a way in, and their walkable cells are reachable from the edge. Dungeon generators use only
  dungeon vaults, local areas only outdoor ones.
- **Scenarios** (`assets/scenarios/*.toml`): level set, start position, vision profile, lantern,
  options, when the run starts: `start_date = [year, month, day]` (all from 1; default
  `[1444, 11, 11]`, 11 Darkening, Year 1444) and `start_time = "hh:mm"` (default `"08:00"`), and the player's stats: `stats`
  and `potential` as for creature types (default 10 each, the potential the score). The
  showcase's player has St 12 Dx 14 To 11 In 13 Wi 9 Pr 10 (potentials 15, 17, 14, 16, 13, 12).
- **Saves**: suspend-only. One save per run, written on quit and by periodic autosave; loading
  continues the run. Dev builds add named snapshots. Format: a versioned envelope (magic, format
  version, game version) around a JSON payload, gzip-compressed with a pure-Rust backend, written
  atomically (temp file + rename). Before 1.0, incompatible saves are rejected with a clear message;
  from 1.0 on, saves are migrated. The web build keeps the same bytes in the browser's
  `localStorage` (below). The envelope also records a content fingerprint (hash of the ordered
  terrain, creature, level, light, dungeon-branch, and object ids), because ids are indices; a
  save made with different content is rejected. Generated levels are saved with the run like
  visited hand-made ones (terrain, memory, stairs, lights, kind, arrival point,
  edge exit, piles of objects and the piles as remembered), together with the gameplay random
  stream, the game clock, every actor's scheduler energy (§8 "Speed"), the direction of the last
  overworld step (saves without the step load with none), what each actor heard and has not yet
  acted on, the sounds the player perceived lately (§5 "Hearing"; saves without them load with
  none), what every actor carries, and what each creature has in mind (where it last saw a
  hostile or heard a noise to look into, and the post it left; §8 "Creature behavior"; saves
  without it load with every creature calm; format version 7), every actor's stats, scores and
  potentials (§8 "Stats"; format version 8), and a dev weather override while
  it lasts (§5 "Weather"; none otherwise, so saves without one load as they are). The weather
  itself is not saved: it follows from the seed and the clock. The save is `run.sav` in the
  platform data directory. `Q` saves and quits (closing the window saves too), autosave runs every 50 turns
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
                  perception, rng, levels, mapgen, stats, save
  warlock-cli/    no Bevy: scenarios, replay, dump, verify
  warlock/        Bevy app: render, input, UI, effects, platform (ECS lives here)
xtask/            screenshot capture, golden-image diff, CI helpers
assets/           fonts, levels, scenarios, vaults, definitions, shaders
```

- **Pure core**: `warlock-core` has no Bevy dependency, no platform I/O inside the rules, and no
  wall-clock time. It builds and tests in seconds and is insulated from Bevy upgrades.
- **Data layout**: dense grids for terrain, light, and memory; a simple entity store with stable ids
  for actors; objects as stacks (object id and count) in sparse piles by position, remembered
  piles beside them, and an inventory and stats on each actor. ECS is used in the presentation
  layer, where it helps with effects, particles, and animation.
- **Perception snapshot**: after each turn, the core produces what the player perceives. The
  renderer, UI, and dumps read only this snapshot, so hidden information cannot leak by construction
  (a dev "reveal" mode excepted).
- **Turn phases**, in order: player action → the other actors on the level act in time order
  until the player may act again, the clock advancing to the moment of each action (the energy
  scheduler, "Speed" below) → environment → perception (line of sight, light, memory, heard
  sounds and their messages) → emit presentation events. An action that makes noise is heard by
  every actor in earshot as it happens (§5 "Hearing"); a creature drops what it heard as it
  acts, the player once they have acted.
- **Time** (`warlock-core/src/time.rs`): world time is whole seconds since the calendar epoch
  (year 1, day 1 of the first month, 00:00), kept in the world state (saved, covered by the state
  hash). The calendar has 12 months of 30 days (a 360-day year) and 24-hour days; month names are
  content (Frostmere, Thawmonth, Seedtide, Rainmoon, Greenleaf, Highsun, Emberheat, Harvestide,
  Leaffall, Mistmoon, Darkening, Deepwinter). `World::time()` returns a `GameTime` with year,
  month (number and index; the name from `Content::calendar`), day, hour, minute, second, minute
  of day (the index for per-minute tables such as the sun's path, §5), and the day fraction in
  16-bit fixed point. All
  integer arithmetic; no wall-clock time.
  - **Costs** (M13): the action decides how long it takes, and the actor's speed scales
    it. Every action that takes time has a **cost** in energy, 1000 for a normal action, given in
    `data/actions.toml` (`step`, `wait`, `open_door`, `close_door`, `lantern`, `stairs`,
    `pick_up`, `drop`, `melee`, `shoot`, `cast`; 100 to 10 000; an action left out, or every
    action without the file, costs 1000; unknown ids are errors), and a **base time**: 6 s
    (`BASE_TURN`) for every action but a step on the overworld. An action of cost *c* takes an
    actor of speed *v* `c / 1000 × 1000 / v × 6 s`: a normal action takes 6 s at normal speed
    (the player's), one of cost 1400 (two-weapon fighting, say) 8.4 s, and a normal action at
    speed 1111 about 5.4 s, at speed 900 about 6.7 s. All built-in actions cost 1000 for now.
    - A step costs `step` scaled by the destination terrain's `move_cost` percent (water 200: 12 s
      at normal speed); on the overworld its base is the destination terrain's `travel_minutes`
      instead of 6 s (scaled by speed like everything else: twice the speed, half the journey).
      `TerrainDef::step_price` (through `LevelState::step_price`) is the one place movement time
      is decided.
    - `stairs` covers taking stairs, entering a location or a local area from the overworld, and
      walking off a local area's edge: 6 s each at normal speed, on the overworld too, where
      waiting and the lantern take 6 s as well. Bumping a closed door costs `open_door`; moving
      into a hostile creature, `melee`.
    - A creature's action costs the same: a step onto the destination terrain, else its action's
      cost (`melee`, `shoot`, `cast` for attacks, `open_door`, `wait` when it waits or is
      blocked). A creature type may give its attacks their own `attack_cost` (§7), so a future
      attack or ability is one more entry. Free and refused actions take no time.
    - The scheduler (below) prices an action at its cost × its base in seconds; an actor gains its
      speed in energy per second, so it takes cost × base / speed seconds. The clock keeps whole
      seconds and leftover energy carries, so a run of actions takes exactly its total time up to
      the whole second the actor is ready, never drifting: ten 5.4 s steps (packed earth at 90 %)
      take exactly 54 s, five 8.4 s attacks exactly 42 s, 1111 steps at speed 1111 exactly
      6000 s; ten steps at speed 1111 (5.4005 s each) take 55 s, the first whole second after
      54.005 s. Single actions show as whole seconds (8.4 s as 9, 8, 9, 8, 8).
  - **Speed** (`warlock-core/src/world/schedule.rs`), ADOM-like: every creature type has a
    `speed` (§7), 1000 being normal and the player's; twice the speed is twice as many actions in
    the same game time. An energy scheduler, integer only: each actor on the current level gains
    its speed in energy per second of game time; an action costs its price, cost × base seconds
    ("Costs" above: 6000 for a normal action); an actor may act while its energy is at least 0,
    and leftover energy carries over, so action counts stay exactly proportional to speed and
    cost over any span while the clock keeps whole seconds. The player's speed may differ from
    1000 too (none of the rules assume it does not). After the player acts, the other actors act
    in time order until the player may act again: next is whoever reaches 0 soonest, the clock moves to that moment, and ties go to
    the player first, then by ascending actor id. With equal speeds every creature acts once per
    player turn, in id order, right after the player; a speed-2000 creature acts exactly twice
    per normal turn and a speed-1500 one three times in two. A creature's action is one function
    (`World::creature_turn`, returning its price): creature behavior lives there.
    Creatures on other levels are frozen (they gain no energy), so coming back to a level never
    sets off a burst of actions. Energy is part of the world state (saved, covered by the state
    hash). Speed is hidden from the player: it is not in the perception snapshot, the status bar,
    look mode, or messages; only ground-truth dev dumps (the world state) show energy.
  - **Rest** passes turn after turn through the normal pipeline: the player waits (a `wait`
    each turn), the others act, perception runs; the last turn is cut short so the rest ends
    exactly at its goal at any speed of the player (it costs all the energy the player has and
    gains until the goal; one hour, the next 06:00, or the next 18:00; at most a day). It stops after a turn in which a
    creature came into view (one not seen the turn before) or a message was logged that stops a
    rest (all but weather news other than rain starting or growing heavier, §5 "Weather"), and
    logs how long the player rested. Turns that end before any creature on the level could come
    within view, and in which the weather the player would see did not change, pass without
    perception: a creature is seen only within the view range, and before its
    first step that could bring it there it needs the energy for every step that cannot (each at
    least the level's cheapest step), so a fast creature shortens the stretch. Those turns cannot
    change when the rest stops (sounds carry at most 30 cells, less than the view range, so none
    is missed), and a night's rest stays fast on large levels
    (`cargo xtask bench-turns` times one; budget 500 ms; also the worst case, a fight in the
    arena going on all night out of the player's sight and hearing).
  - **Dev skip** (`F6` in dev builds, replay text `skip:1h`) moves the clock ahead at once;
    nobody acts or gains energy and no turn passes, but it is recorded like any action.
- **Creature behavior** (`warlock-core/src/world/ai.rs`, inside `World::creature_turn`; M10):
  quite deterministic, outplayable by design; no randomness. Each action, in order:
  1. **Look**: the hostile actors it sees (§5 "Creature sight"; hostile by faction, §7); the
     nearest is its target (squared distance; ties: the lower actor id).
  2. **Fight**: a `melee` creature attacks a target next to it (diagonals too), else steps along
     a shortest path toward a cell next to it. A `ranged` creature or `caster` shoots or casts
     when the target is within its `range` (the rounded radius, like every radius) with a clear
     line of fire: no opaque cell and no actor on the cells between (an integer Bresenham line);
     else it steps toward the nearest cell that gives it one, failing that toward the target.
     Leaving its routine it remembers where it was (its post) and where it saw the target.
  3. **Search**: with no hostile in sight, it goes to where it last saw one (next to the spot if
     someone it cannot see stands there).
  4. **Listen**: failing that, to where it last heard a noise (§5 "Hearing"; the newest it heard
     since it last acted) whose source cell it does not see. Only creatures whose faction is
     hostile to someone go to look; wildlife does not care.
  5. **Return**: then back to its post, and on along its patrol route from where it left it (or
     it waits there).

  Paths are A* over king's moves within a bounded box (12 cells beyond the target, at most 64
  from the creature), each step costing 1: walkable cells, and closed doors for creatures that
  open them (1 more: it opens the door, a turn and a noise, then steps in). Another actor's cell
  costs 4 more, so a creature walks a short detour around someone in its way but waits behind
  them rather than take a long one. Ties go to cells nearer the target, then to the earlier
  direction (N, NE, E, … NW). A spot it cannot reach is given up.

  **Attacks are intents only**: no damage, hit points, or death. An attack is an event
  (`Event::Attacked { attacker, target, kind: Melee | Ranged | Spell, from, to }`), a noise (§5),
  and a message when the player perceives it (§6). The event reaches presentation only when the
  player sees both ends (or made the attack). The player moving into a hostile creature is a
  melee attack; into a neutral one it stops. What a creature has in mind (last seen, noise to
  look into, post) is part of the world state.
- **Stats** (`warlock-core/src/stats.rs`; M14): six stats inspired by D&D, in display order
  Strength (St), Dexterity (Dx), Toughness (To; D&D's Constitution), Intelligence (In),
  Willpower (Wi; D&D's Wisdom), and Presence (Pr; D&D's Charisma). More will come (mostly
  magic): the list is one typed definition in core (id, abbreviation, name per stat), and every
  per-stat value is stored by stat, so a new stat is one entry there (plus a save format bump).
  - **Scores** are integers, 10 the average and the default. Every actor has one in each stat:
    a creature from its type, the player from the scenario (§7). There is no character creation.
  - **Modifier**: as in D&D 3.5, `floor((score − 10) / 2)` (1 → −5, 8–9 → −1, 10–11 → 0,
    12–13 → +1, 18 → +4; rounded down for scores below 10 and below 0 alike). It is a core API
    for the rules (`stats::modifier`, `Stats::modifier`) and is **never shown**: not in the UI,
    messages, look mode, player-facing dumps, or the wiki.
  - **Potential** (ADOM-like): each stat's natural ceiling, the score it could grow towards.
    Stored and saved, **hidden from the player** like the modifiers. Nothing raises a score
    yet (training comes later).
  - **What the player sees**: their own scores, on the character screen (§6) and in the
    `stats` dump layer; the perception snapshot carries those and nothing else of stats
    (`PerceptionView::own_scores`). A creature's stats are never shown; ground-truth dev dumps
    (`actors`, the world state) show every actor's scores and potentials.
  - **No rule uses stats yet**, and no other D&D concept comes with them: no ability checks,
    saving throws, hit points from Toughness, or carrying capacity from Strength. Stats and
    potentials are part of the world state (saved, covered by the state hash).
- **Events to effects**: the core emits events (door opened, actor moved, attacked); the
  presentation layer turns them into animations: slides, a short lunge toward the target for a
  melee attack (motion), a shot glyph flying from shooter to target and a creature's `attack`
  particles where its attack lands, e.g. the shaman's violet sparkle (particles). Gameplay never
  waits for them.
- **Determinism**: ChaCha8 RNG (`rand_chacha`) with separate streams, each seeded with FNV-1a 64 of
  the run seed (little-endian) and a label: map generation per level (`map/<level id>`, retry *k*
  `map/<level id>/<k>`), so a level depends only on the seed and its id, never on visiting order;
  gameplay (`gameplay`, kept in the world state and saved); the weather, which keeps no stream at
  all: each weather block hashes the seed, a label, and its number (FNV-1a, then SplitMix64's
  finalizer; §5 "Weather"); and visual-only randomness in the app, which never uses either. No hash-order iteration and no floating point in the core's decisions.
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
  pickup pickup:ac drop drop:b vision:darkvision skip:1h weather:rain`; durations are a whole
  number with `s`, `m`, or `h`, a rest at most `24h`; letters name stacks of the pile or the
  inventory, `a` first, and without letters `pickup` and `drop` take everything; the dev weather
  override takes `clear`, `overcast`, `rain`, or `heavy`; in scripts, `ACTION*N` repeats an
  action, e.g. `wait*10`); `hash` is the
  FNV-1a 64 hash of the
  canonical JSON encoding of `WorldState` after the step. Verification reports the first
  diverging step. `--replay` takes its scenario and seed from the file; `--record` rewrites its
  file after every action (including replayed steps).
- Dump layers: `view`, `fov`, `memory` (terrain and the top object of each known pile), `light`
  (perceived brightness of each visible cell as a digit, `0` dark to `9` fully lit or brighter),
  `sound` (the view with each heard sound at its rough place: `f` footsteps, `o` a door opening,
  `c` a door closing, `x` fighting, `b` a bowstring, `m` chanting, upper case when loud),
  `inventory` (what the player carries, by letter with weights, then what they carry and weigh),
  and `stats` (the player's own scores, `Strength 12`) are built from the perception snapshot
  only;
  `truth` (terrain, objects, and creatures) and `actors` (every actor on the level with its
  weight, base plus carried, what it carries, and its stats as score/potential, `St 12/15`) show
  ground truth and are dev-only. JSON dumps
  carry turn, game time (seconds and calendar fields), level, viewer, the weather (under the open
  sky; else null), seen actors, heard sounds, the player's inventory, weight, and scores
  (`stats`, by stat id), recent messages,
  and optionally the full world state; the ASCII header line has the turn, the time
  (`0001-01-01 08:00:00`), level, viewer, the weather under the open sky (`weather rain`), and
  hash.
- Freezing visual time and fixing window size and scale make screenshots reproducible. With frozen
  time, glyph animations show that moment and motion (slides, camera) completes instantly.
- Dev builds add in-game commands: switch vision profile (`F3`), reveal map (`F2`, ground truth,
  marked as dev-only on screen), toggle debug overlays (light values `F4`; line of sight and memory
  later), save and load named snapshots (`F5`/`F9`), skip an hour (`F6`, recorded as `skip:1h`),
  set the next weather for three hours (`F7`, recorded as `weather:overcast` and so on).
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
  day (moonlight followed in M12); an ADOM-like overworld with per-terrain travel time,
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
- **M13 Action costs** — every action has a cost in energy (data, 1000 a normal action; creature
  types may give their attacks their own), and the cost and the actor's speed decide how long it
  takes over one 6-second base (overworld steps: their travel time); levels lose their own turn
  length; exact over any run of actions and at any speed of the player.
- **M14 Stats** — D&D-inspired stats (Strength, Dexterity, Toughness, Intelligence, Willpower,
  Presence) on every actor, with hidden D&D 3.5 modifiers and hidden ADOM-like potentials; the
  player's own scores on a character screen (`@`).

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
