# Warlock specification — 4. Rendering

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

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
   like terrain; a cell with objects does not animate), known traps (their glyph over any objects
   there: a danger shows first; §8 "Traps"), blood stains (§4.8), and in-cell animation.
2. **Free-glyph layer** — instanced glyph quads from the same atlas for actors, particles, and
   anything that moves between cells or grows beyond its cell. Lit by sampling the light texture.
   A glyph may be drawn in up to four horizontal colour bands, top first: a composite animated
   thing (§8 "Animated objects") shows the distinct colours of what it wears and wields, in slot
   order (a gold helm over steel plate over leather boots), each its material's own colour at the
   glyph's brightness, with soft borders that sway slowly up and down (still with glyph
   animation off; never a rainbow, never a cycle through the colours).
   **The unaware mark** (§8 "Surprise and sneak attacks"; Stalker's eye): a creature the
   perception snapshot says has not noticed the player (`SeenActor::unaware`, set only for a
   player with the talent) has its colour, and each band's, mixed toward a cool slate
   (`#8a96a8`, through the palette) by a quarter before lighting: the kobold is still red, just
   quieter. With glyph animation on the mix breathes between 0.15 and 0.35 over 6 s on the
   creature's own phase; off (reduced effects) it stands still at 0.25. Less than the
   darkvision fade; no shader change (free glyphs are coloured on the CPU).
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
  drift), creature shadows, and transient effect lights: a burning missile in flight lights its
  way (its object's light preset, with its flicker, falling off like gameplay light from its
  fractional position to each visible cell's center it has a line of sight to over the terrain the
  player knows), and the gameplay light of a burning missile that has landed is held back until
  it arrives on screen (§4.8); a spell effect's light (§4.8 "Spell effects") lights the scene the
  same way in any color, falling off more softly (the square of the gameplay falloff), through
  the cells the effect was seen on too, and may darken everything else while it peaks.
  Visual-only light never affects gameplay.
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
  particles, screen fades and flashes, screen shake, and falling rain. Each can also be switched
  off on its own (settings: glyph animation, light flicker, motion = slides, camera easing, and
  spell effects' motion, bloom and glow, distortion, particles, screen fades = fades, flashes,
  and spell effects darkening the view, screen shake, weather); an effect runs only when its
  switch is on and reduced effects is off. Blood (§4.8) has a switch too, outside the
  preset: reduced effects leaves it as it is; so do spell circles (§4.8), which reduced effects
  only stills. Rain has a switch of its own rather than riding on particles: it moves
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
- **Glyph particles** on the free-glyph layer (terrain, creatures, objects). An emitter has glyphs to pick from, colors over the
  particle's life, a lifetime, velocity ± spread, gravity, a start offset and spawn area, sideways
  sway, scale, fade-out, glow, and opacity (how opaque it is at most, default 1). Emitters fire `ambient` (a steady rate while the source is in
  view: sparks over braziers and torches, embers over lava, fountain spray, dust motes), on `step`
  (a burst when a creature is seen entering the cell: a splash in water), on `toggle` (a burst
  when a cell is seen turning into the terrain: dust as a door opens or closes, or a wall gives way
  to rubble), or at an `obstacle` (a steady rate from visible
  running water whose flow runs into a known cell that is not running water: foam at banks and
  boulders, ripples where a stream enters a pool; the emitter is turned with the flow, its `y`
  pointing downstream). Creature emitters move with the creature; a creature's `attack` emitters
  burst at the target of each attack the player sees (the kobold shaman's faint violet sparkle).
  Spell effects' emitters burst from each affected cell on `impact` (as the effect reaches it: a
  fireball's embers) and `linger` (as it starts to fade: its smoke), and trail a burst's
  projectile on `flight`.
- **Missiles**: every shot and throw the player perceives (`Event::Missile`, which carries the
  cells it flew over as far as the player saw them, so nothing unseen is drawn; a missile from an
  unseen shooter appears where it enters view) flies on the free-glyph layer from the edge of the
  shooter's cell (or where it came into view) to where it came down (or a little short of the
  center of whom it struck, or out of the last cell seen) at 30 cells a second (at least 60 ms),
  easing out a little; the missiles of one turn fly one after another. Its glyph is its object's
  `missile` glyph (§7), drawn pointing right and turned along the flight on screen (free glyphs
  can turn: an arrow `→`, a bolt `➔`, a spear `⟶`), tumbling if it `spins` (a thrown dagger
  `†`), else its own glyph upright (a stone `•`); it fades in and out at the ends and trails three
  faint dots. It is lit like any free glyph; a burning one is light itself (unlit, glowing a
  little with `glow`), lights its way (§4.4), and sheds embers (its `flight` particles), and it is
  seen wherever it flies in the player's line of sight, in the dark too (§8 "Ranged combat"). A
  tiny puff of dust where it comes down or strikes. What it lands as (the arrow on the floor, a
  burning arrow's glyph, glow, sparks, and light) is held back until it arrives. With the motion
  switch off (or reduced effects) nothing flies: what lands is there at once; embers and puffs run
  with particles; with frozen visual time a missile is shown halfway. Attack bursts run with the
  particles switch. Objects may have particle emitters too: `ambient` while one lies in view on
  top of its pile (a burning arrow's sparks), `flight` while it flies.
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
- **Spell effects** (`data/effects.toml`, §7): every spell-like effect the player perceives
  (`Event::Spell { effect, origin, shape, cells }`: a trap's fireball or lightning bolt now,
  spells later; its cells cut to those the player sees or has in line of sight, since the effect
  is light itself, so nothing unseen is drawn) plays its effect, a pure function of visual time
  and the event like the particles, in three phases: **travel** (a burst's projectile flies from
  its origin to its center, a bolt draws out along its path), **impact** (full strength), and
  **linger** (fading). They are deliberately dramatic, the one place where effects may be loud,
  yet short (well under a second) and never garish:
  - A **fireball** (a burst over the cells around a center) grows from its core to its edge,
    white-hot at the core and cooling outward through yellow and orange to deep red (its glyphs
    by heat, `▓▒░`, flickering, over a glowing body), burns, then cools from its edge in; embers
    fly at the impact and a faint smoke rises as it fades. Its heat is measured as it looks on
    screen (cells are taller than wide), so the hot core looks round.
  - A **lightning bolt** (a line along a path) draws out almost at once and flashes two or three
    times, each flash a little dimmer: a jagged line of turned line glyphs (`━`, about a cell wide
    each, the corners pushed sideways by a slow wander and a sharp jitter) that takes a new shape
    with every flash and flicker and forks now and then into the cells the player sees, very
    bright blue-white, then a violet afterglow.
  - A **frost burst** (`frost_burst`, a burst; §5 "Ice") is the quiet one: a cold flash of rime
    (`✳*·`, white to deep blue, a thin glowing body, a little cold light) over the cells it
    reached, with a few frost crystals drifting down as it fades; no darkening and no shake. The
    dev action `freeze:x,y,r` shows it over the water it froze.
  - All are light themselves (drawn unlit and glowing, on top of everything else on the map)
    and light the scene with their own light (§4.4): from a burst's center, along a bolt every
    other cell.
  - **Darkening**: while a bright effect peaks, everything that is not its light (base light,
    every other source, the readability floor, remembered terrain, other glyphs and particles)
    looks up to `darken` darker (0.65 = 65 % darker on screen), easing in over the first tenth of
    its `seconds`, holding until two fifths, then easing back: the effect dominates the view and
    the scene comes back, deliberately, never like a frame drop. Off with the `screen fades`
    switch.
  - **Screen shake**: when an effect lands within its `shake` radius of the player (the nearest
    of its cells), the map view shakes by whole pixels, at most its `pixels` (logical pixels, on
    the player's cell; less the farther), kicking in a new direction 28 times a second, eased, and
    dying down over its `seconds` (about a quarter of a second). The status bar and messages stay
    put. An experiment: tuned in data, off with its own `screen shake` switch.
  - The effects of one turn play one after another; new input does not cut them short (they hold
    nothing back), a level change does. With the `motion` switch off (or reduced effects) an
    effect is a brief still flash of its cells (0.3 s): no travel, light, darkening, or shake.
    With frozen visual time (`--freeze-time T`) each effect of the last turn shows its moment `T`
    seconds after it began, so captures can show any moment.
  - Presentation only: no rule reads effects. The dev action `effect:ID@X,Y` (a burst there, or a
    bolt toward `>X,Y`; §9) shows one with no other consequence, for captures and goldens.
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
- **Blood** (§8 "Blood"): a stained cell's colours shift toward the blood's by how much lies
  there — a floor's dots and background give way to a muted red, a soaked cell is one dark pool
  with its dots drowned, a wall's face is tinted more lightly, and an object lying in blood keeps
  its own colour over a stained background — before the grid shader lights them like the
  terrain they lie on, so stains fade into the dark, show in darkvision's monochrome, and dim
  with the cell in memory. Old blood dries darker and browner over about six hours of game time
  (fastest at first). A few drops are barely there; only many deaths in one place look gory. A
  **Blood** switch in the settings (on by default) hides stains, in look mode too; it is a
  content preference, not an effect, so reduced effects leaves it as it is.
- **Spell circles** (`data/runes.toml`, §7; design: `docs/design/buff-circles.md`): about every
  actor the player sees with a buff's or a hex's status (a runed status: one of a buff or hex
  spell's, whatever laid it, a potion or an altar too), a thin, slowly turning transmutation
  circle, drawn from the perception snapshot only (the player's own statuses, a seen creature's
  that show) over the map grid and under the creatures:
  - **The rule**: the effects of one kind on one actor, in status order (the status bar's), fill
    `n × ⌈3 / n⌉` slots: one effect is a triangle of its rune thrice (never a lone rune), two a
    square `A B A B`, three a triangle `A B C`, then a square, a pentagon, a hexagon; a regular
    polygon inscribed in the innermost ring has its corners at the slots, and a rune (one kana
    or low-stroke kanji per status, upright) sits outside each corner. One thin ring per effect,
    three at most (the distinct runes give the count beyond); six effects show, a seventh has no
    rune.
  - **Buffs and hexes**: buffs are whole (solid rings and sides) and turn clockwise, a revolution
    a minute; hexes are broken (dashed) and turn counter-clockwise, a revolution in 45 s. On an
    actor with both, the buffs' circle is the inner one and the hexes' the outer; alone, either
    is the compact inner figure.
  - **Schools**: a rune is drawn in its school's colour (seven, through the palette), the rings
    and sides in the first rune's.
  - **Sizes** in cell heights `h`: rings at 0.48, 0.58, and 0.68 h, runes 0.55 h square centred
    1.0 h out (extent 1.3 h: 2.6 rows by 5.2 columns); the outer circle's rings from 1.35 h and
    runes at 1.88 h (extent 2.15 h). Lines one pixel at the default font and scale 1; rings and
    sides 0.28 opaque, runes 0.7; not lit by the scene, but darkened with free glyphs while a
    spell effect darkens the view.
  - A circle appears and goes at once and a change of count is a cut; with `motion` off or
    reduced effects it stands still (the inner at its first angle, the outer half a turn round):
    still information, not off. A **Spell circles** switch (on by default, outside the preset)
    turns the drawing off; the statuses and what look mode says of them stay. With frozen visual
    time it shows the frozen moment's angle. The runes come from a subset of Noto Sans JP
    (§4.1's font has no kana): `assets/fonts/NotoSansJP-Runes.ttf`.
  - **The light** is a rule (§8 "Status effects"): a runed status sheds its school's faint light,
    lit in the lighting pass as any light (its UV share tints only for eyes that see UV), and the
    switch never hides it. With `glow` on the circle's ink is a little emissive (× 1.15).
- With frozen visual time (`--freeze-time`) fades complete instantly and bursts show a fixed
  moment early in their life, so captures are reproducible; spell effects show the frozen moment
  after they began.
