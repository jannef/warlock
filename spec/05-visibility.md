# Warlock specification — 5. Visibility, light, and vision

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

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
  the terrain's own), fires (a burning object lying on a cell, such as a flaming arrow that has
  landed, until it has burned out; §8 "Ranged combat"), creatures that carry light, the
  player's lantern (from the scenario, toggled by the player; takes a turn), and the glow of a
  magic item an actor wears or wields (its brightest; a steady preset, §8 "Magic items").
- **The lantern's hood** (#235; `world/lantern.rs`): the lantern is out, lit, or hooded. A preset
  may name the preset it burns as with its hood lowered (`hooded`, itself without a hood); the
  scenario lantern's does (`lantern_hooded`: a third of the light, radius 2). A preset may be
  `hollow`: its own cell gets none of its light, so the hooded lantern lights the cells around
  its bearer but not them. With normal eyes the player sees their 8 neighbours by it; their own
  cell gets only bounce light (at most 12.5 % of the ring's direct light, below every eye's
  threshold), so creatures whose eyes need light see them only by other light, and darkvision
  within its radius. Hooding (`hood`, the `hood` action, 500) lowers or raises the hood; on a
  lantern that is out it lights it hooded (`lantern` + `hood`); dousing raises it. One place
  decides the lantern's light, `WorldState::lantern_light`, which the light cache reads for
  perception and creature sight alike, so they agree. The state is saved (`lantern_hooded`,
  left out while false).
  Emitters must be transparent; a source in an opaque cell
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
- **Special rooms** (§7): a special room changes the light inside it. A shadowed room keeps only
  its effect's `light` percent of every part of a cell's light: the base light (ambient, sky,
  bounce) and each source's contribution (placed lights, carried ones and the lantern, fires, the
  sun and the moons; a wall's face light too), each rounded down on its own, so contributions plus
  base light still add up. A sanctuary's `glow` adds to the base light of its transparent cells,
  like ambient light, as the eye perceives it (its UV only for UV sight); walls take it by the
  wall-face rule (the brightest of their seen transparent neighbors'), so it never shows through
  a wall. Light outside a special room is never changed (bounce is computed from the unchanged
  direct light). Creature sight finds the same light (a test checks it in rooms of both kinds).
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
- **Temperature** (`warlock-core/src/temperature.rs`; M26): the air on every level has a
  temperature, in tenths of a degree Celsius, integer math only.
  - **Underground** (dungeon levels) it is fixed: the level file's `temperature`, else its
    dungeon branch's for that depth (`data/dungeons.toml`; the Depths 10 °C, its caverns 8 °C,
    its deepest levels 6 °C), else 10 °C. It is kept with the level (saved).
  - **Under the open sky** (outdoor levels and the overworld) it is a pure function of the run
    seed and the time, like the weather, so long rests and journeys need no history: the
    month's mean (`temperature` per `[[month]]` in `data/weather.toml`; the middle of each month
    has its mean, and the days between two middles go linearly from one to the next: Frostmere
    −7 °C, Thawmonth −3, Seedtide 3, Rainmoon 8, Greenleaf 13, Highsun 18, Emberheat 22,
    Harvestide 18, Leaffall 12, Mistmoon 6, Darkening 1, Deepwinter −4), plus the day's swing
    around it, a cosine (the sun's fixed-point one) coldest at 04:00 and warmest at 16:00: a clear
    day's `swing` (10 °C from the coldest to the warmest) times the weather's `swing` percent
    (overcast 40, rain 30, heavy rain 25: clouds keep nights milder and days cooler), less the
    weather's `chill` (rain 1 °C, heavy rain 2 °C). Where the weather changes, its swing and chill
    change with the clouds, over the same hour (`weather::Blend`). The dev weather override sets
    them too, from the moment it is given. A clear night on 11 Darkening falls to about −3 °C.
  - **Shown**: the status bar always shows the temperature where the player is, in dungeons and
    outdoors alike, in whole degrees (`4 °C`; °F with the setting, §6), with a subtle cool tint
    below freezing and a warm one from 30 °C. The number is the one exception to numbers being
    hidden: the player feels the air. Nothing else feels it yet: no hypothermia, warmth, or
    clothing; it freezes water ("Ice" below). Special rooms and ice do not change the air.
- **Ice** (`warlock-core/src/ice.rs`, rules in `world/ice.rs`; M26):
  - **Freezing**: every cell of water that freezes (terrain with a `freeze` table, §7: still
    water below 0 °C at full pace; running water only below −6 °C, at 40 %) has an ice
    thickness, kept sparsely with its level (only cells with ice; in micrometres, so slow
    growth is not lost to rounding; saved, hashed). Growth follows Stefan's law: each tick (10
    minutes) the square of the thickness grows by 400 000 µm² per tenth of a degree below the
    water's freezing point, times its rate (2.4 cm per √(°C·day): about 5 cm in a day at −5 °C,
    13 cm in three days at −10 °C), quickly at first and ever more slowly; above 0 °C it melts 5
    µm a tick per tenth of a degree (7.2 mm a day per degree); between the two it keeps. At most
    1 m.
  - **Lazily**: nothing ticks while nobody looks. A level's ice is brought up to the clock when
    the rules may meet it (as the clock moves on the player's level, during rests too, and when
    the player arrives), tick by tick from where it was left (`LevelState::ice_until`). Ticks fall
    on fixed moments of the clock (every 10 minutes since the epoch), each at the temperature of
    its middle, so the ice at a moment is the same however the time passed: by turns, rests,
    overworld journeys, or the dev skip. Cells that start alike stay alike, so catching up works
    through the thicknesses there are (and each kind of open water), not the cells: a year away is
    52 000 ticks of a few groups. A level under the open sky seen for the first time has had 30
    days of its weather already (a pond is frozen when first come upon in deep winter);
    underground levels start with the ice their file lays.
  - **Looks**: a cell shows the thickest ice terrain its ice reaches (`ice` in
    `data/terrain.toml`: thin ice from 1 cm, ice from 5 cm, thick ice from 15 cm), else the water
    beneath; the terrain grid holds that, so memory, sight, and look mode follow ("thin ice",
    "ice", "thick ice"). Ice is still: no waves, glints, or rain ripples. The player never learns
    a thickness, only how it looks.
  - **What it holds**: Gold's formula with `A` = 3.5 kg/cm²: 35 g per mm² of thickness squared,
    so 5 cm holds 87.5 kg (a person with a pack), 15 cm 787.5 kg. An actor stepping onto ice
    whose weight (its creature's base weight plus everything it carries, worn and wielded things
    included) is at most that crosses safely, with no draw. Above that the ice gives way with a
    chance rising evenly to certain at twice that (one draw of the gameplay stream, made only
    between the two); ice that held by luck creaks under the player ("The ice creaks under your
    weight."). A step onto ice costs its terrain's `move_cost` (120: careful steps). Wading into
    water whose ice is too thin to look like ice breaks that skin.
  - **Breaking through**: the cell is open water again (its ice gone; it refreezes from
    nothing), and what lay on the ice sinks to the bottom: it stays in the cell's pile, under the
    shallow water, where wading in picks it up. An actor that broke through from firm footing
    plunges in: "The ice cracks beneath you and you plunge into the freezing water!" (a creature
    the player sees: "The kobold falls through the ice!", `Event::FellThroughIce`), and loses the
    water's `plunge` hit points (1d3; never the last one: the cold does not kill yet). The step
    was a step into the water. From the water, ice that cannot hold the actor breaks ahead of it
    ("You break through the ice."), without another plunge.
  - **Climbing out**: a step from open water onto ice that holds the actor also costs `climb`
    (`data/actions.toml`, 2000: 12 s at a normal pace), a few turns of hauling oneself up ("You
    climb out onto the ice."). Wading out onto the shore is an ordinary step.
  - **Bursts** (hooks for spells, traps, and explosions): `World::frost(center, radius,
    thickness)` freezes the water within the radius in line of sight of the centre to at least
    that thickness at once; `World::shock(center, radius, Shock)` breaks the ice it reaches: heat
    (a fireball) melts all of it, a blast without heat shatters ice thinner than its strength,
    and whoever stood on ice that broke falls in. The dev actions `freeze:x,y,r` (10 cm, shown
    as the `frost_burst` spell effect, §4.8) and `thaw:x,y,r` (heat) use them, and so do traps
    (§8 "Traps"): the fireball trap's heat melts the ice it reaches, and the frost trap freezes
    the water it reaches to 10 cm. Blasts without heat (none yet) will shatter thin ice.
  - Creatures follow the same rules: they step onto ice, fall through, and climb out, and so
    does one who makes way for the player onto the ice they stood on (§8 "Displacement"). The
    overworld's rivers and lakes do not freeze (map scale; crossing a frozen river may come
    later).
- **Brightness** is the integer luma of RGB, `(54·r + 183·g + 19·b) / 256`, plus UV for eyes that
  see UV. Eyes without UV sight do not perceive UV at all (perception reports it as 0).
- **Visibility**: a cell is visible iff it is in line of sight within the maximum view range **and**
  (its perceived brightness ≥ the eye's threshold **or** its distance ≤ the darkvision radius). The
  player's own cell is always visible. Creatures are seen iff their cell is visible: creatures in
  darkness are hidden (unless within darkvision), creatures carrying light are seen. A
  **light-blind** player (their race's `light_blind`: drow) whose own cell is lit at least to its
  `glare` is dazzled and sees nothing farther than its `sight` (§8 "Character creation").
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
  them. What creatures carry is never seen; what a seen creature wears and wields is (look mode,
  §6).
- **Memory**: per level, terrain, piles of objects, and known traps, as last seen (a door is
  remembered open until seen closed, a wall standing until seen broken (§8 "Digging"); a pile is
  remembered as it lay when last in sight, even if it changed out of sight since, and is forgotten
  once seen gone; so is a known trap, seen spent and gone). Never creatures, never hidden traps.
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
    the terrain stepped onto, in percent (water, running water, and shallow water 200, marsh,
    bushes, and rubble
    150, reeds 130, gazebo boards and dry grass 120, tall grass and rocky ground 110, dirt road
    90, forest floor 80, grass, flowers, and dusty floor 70; default 100), and by the walker's
    burden (§8 "Burden": ×1.25 Burdened, ×1.5 Strained; a dwarf's steps are not louder while
    Burdened), rounded down, less what the stealth affixes of what it wears take off (§8 "Magic
    items": "footsteps 20 % quieter"), rounded down. Opening a door makes
    the closed door's `toggle_noise` (60), closing one the open door's (70). A special room
    (§7) scales every noise made inside it (where the step lands, the door, where an attacker
    stands) by its effect's `noise` percent, rounded down: an echoing hall doubles it (so it
    carries farther), a hushed chamber leaves two fifths. One function decides
    an action's loudness (`noise::noise`): the actor's skills take their percent off its steps
    and doors there, rounded down (stealth, §8 "Skills": 60 % at 100; not its attacks). Loudness
    is at most 300; a creature without `footsteps` moves silently, a door without `toggle_noise`
    is silent. Attacks
    (§8 "Combat"; hit or miss) are heard where the attacker stands: fighting (melee, 60), a
    bowstring (a shot, 40), chanting (a spell, 50); a turn of digging where the digger stands
    (digging, 80; §8 "Digging"). A kick at a door (§8 "Doors"), whether it holds or breaks, bangs
    on the door: banging, 100, made on the door's cell, so both sides hear it. A creature
    muttering to itself (§8 "Dialogue") is a voice (50) where it stands; the player's voice,
    thrown (§6 "Abilities"; #235), is a voice (50) on the cell it is thrown to, a cell they see
    (not solid) within 6 (`VOICE_RANGE`, a disc), made there and not where they stand, and as
    quiet as ever (stealth does not touch a voice). A trap going off (§8
    "Traps") makes the noise its data gives on its cell, made by no actor (so everyone in earshot
    hears it): an arrow trap's bowstring (40), a fireball's explosion (150), a lightning bolt's
    thunderclap (120), an alarm's bell (300, as loud as anything gets).
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
    bat 3; none is deaf; the listen skill lowers it, never below 1, §8 "Skills") hears a noise
    another actor
    makes on its level when the loudness at its cell reaches that, at once, as the action
    happens. A creature keeps what it heard (kind, where it was made, how loud it was at the
    creature) until it next acts: the input its AI reads (§8 "Creature behavior": it goes to look
    where it heard a noise whose source it does not see); the list is part of the world state.
  - **The player** perceives the sounds they heard during the turn that just passed whose source
    cell they do not see (a creature in darkness or out of sight, a door around the corner),
    never who or what made them: the kind (footsteps, a door opening, a door closing, fighting, a
    bowstring, chanting, banging, digging, a voice, a bell, an explosion, a thunderclap), how loud
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
    turn is said once per pass, not every turn. Like any message they stop a rest. A voice is
    said with its words instead (§8 "Dialogue"), and never stops a rest.
  - **On the map**, each perceived sound is a faint, still marker (`◌`, pale grey, fainter for
    fainter sounds, never lit) at its rough place until the next turn; with glyph animation on
    it ripples in, a ring growing out of it and fading within a second. Look mode describes it
    ("You hear footsteps here."). The player does not hear their own thrown voice, but where it
    sounded (`Event::VoiceThrown`) is marked the same way, as a loud sound, until the next turn.
- **Creature sight** (`warlock-core/src/sight.rs`; M10): a creature sees by the player's rules,
  with its own eyes (its type's `vision`, a profile above) and radius (its `sight`, default 10,
  at most 40): an actor is seen iff its cell is in the creature's line of sight (the same
  symmetric shadowcasting) within its sight radius and lit at least to its eye's threshold by the
  same gameplay light the player sees (ambient, sky, every source, the sun and the moons through
  the weather's clouds, bounce, as a special room leaves it: a shadowed room's darkness, a
  sanctuary's glow), or within its darkvision; on the overworld, line of sight within
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
  sun, the moons, the sky, and the weather of the moment, the temperature where the player is (§5
  "Temperature"), the perceived sounds (kind, volume, rough place;
  "Hearing" above), and the traps the player knows, seen or remembered (`PerceptionView::trap`,
  `known_traps`; §8 "Traps"; never hidden ones). Visual lighting (§4.4) is derived from this.
