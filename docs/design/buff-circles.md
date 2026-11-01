# Warlock — Spell circles: a rotating transmutation circle for every buff and hex

Design document for the owner's review (2026-11-01). Written against `origin/main` at a9a703a6 (the spell scaling slices S0 to S11). Nothing here is implemented; §13 cuts it into issues. Every number is a first guess kept in data or in one named constant, to be tuned with captures.

**The owner's request** (verbatim): *"I want a spell effect to rotate around characters with active buffs. Kinda like transmutation circles in full metal alchemist. Some basic geometric shapes (such as triangles) to connect runes and glyphs as 'spell runes'. Pick one kanji for each buff spell to act as the spell rune. The system should have clear rules how to [combine the] runes (kanji) of different spells when multiple buffs are active."* Then: *"hexes should also have debuff circles. but they need to be visually distinct. Hexes and buffs also need to be able to combine visually when on same character -- so design both at once. The indicators should have visual identity, that reveals: buffs or hex, which spell school, how many effects are active."* Then: *"i still don't want just 1 rune spell spell. hoveing rune looks too much like an 'ioun stone'. I want them to have the feel of rotating transmutation circles of FMA -- so even simplest of spells need some symmetry."* Then: *"we want this to be done in shader probably, or perhaps particle system. it needs to be performant"*, *"the runes probably need to give out light, including UV"*, and *"no need for talents for this. gameplay effects are just side effect. this does not need to drive talents"*.

## Contents

1. Summary
2. What exists today
3. The circle: anatomy
4. Composition: the one rule, and its sketches
5. Buffs and hexes: distinct, and together
6. Schools: colour and light
7. The runes: one kanji per spell
8. The light: a rule in `warlock-core`
9. Perception: what shows on whom
10. Rendering: a shader, not particles; performance
11. Motion, reduced effects, frozen time
12. Data, the font, checks, verification
13. Implementation plan
14. Rejected along the way
15. Open questions for the owner

## 1. Summary

| Piece | Decision |
|---|---|
| What is drawn | around every actor the player sees who has a **runed status** (a buff's or a hex's), a **spell circle**: concentric thin rings, a regular polygon inscribed in the innermost, and a kanji **rune** upright at each of the polygon's corners, the whole turning slowly about the actor. §3 |
| The one composition rule | **a rune for every effect, repeated until there are at least three, evenly around the ring at the corners of a regular polygon; a ring for every effect.** One effect is a triangle of the same rune thrice (never a lone hovering rune); two are a square `A B A B`; three a triangle `A B C`; four a square; five a pentagon; six a hexagon. The count is the ring count (one to four rings; four means "four or more"). §4 |
| Buff or hex | buffs are **whole**: solid rings, solid sides, turning clockwise. Hexes are **broken**: dashed rings, dashed sides, turning counter-clockwise. On an actor with both, the buffs' circle is the inner one and the hexes' the outer, counter-rotating. §5 |
| School | the rune's colour (seven colours, one per school, through the palette like every colour), the circle's light (§8), and the ring's colour (the school of the first rune in status order). §6 |
| Runes | Light 光, Haste 速, Barkskin 木, Blessing 祝; Evil eye 呪, Hex of weakness 弱, Curse of sloth 遅, Sleep 眠, Web 網. In data: `data/runes.toml`, one row per status, with its school. §7 |
| Light | **a rule, not a visual**: a runed status sheds a small, steady light of its school's colour **with a UV share**, radius 1, from `data/lights.toml`, through the one existing path (a status's `light`, the brightest radius wins). A buffed or hexed creature glows faintly in the dark, and plainly to eyes that see UV. §8 |
| Perception | the circle is drawn from the snapshot only: the player's `own_effects`, and seen creatures' `effects_of` (statuses with `seen`). **A runed status must have `seen`** (a load-time check); `hasted` and `barkskin` get one. No severity, no time left: a circle says what the status bar says. §9 |
| Rendering | **a shader**: one instanced draw of a quad per circled actor, signed-distance rings and polygons, the runes sampled from a small kanji atlas rasterised once at load. No CPU text layout, no particles. Budget 0.2 ms a frame at 1440p with fifty circles. §10 |
| Reduced effects | the circle stands still at its first angle and does not fade; the core light is unchanged. A `circles` switch turns the drawing off (never the light). §11 |
| Talents | **none, and no new `[[system]]` tag** (owner, 2026-11-01): the light's effect on vision is a side effect of buffs and hexes, not a system of its own. §8.5 |
| Verification | pure layout and packing tests, core light and data tests, **one golden** (only the GPU shows the circle). §12 |

## 2. What exists today

| Piece | Today |
|---|---|
| Buff spells (`kind = "buff"`, all `shape = "self"`) | light → `lit` (evocation), haste → `hasted` (transmutation), barkskin → `barkskin` (transmutation), blessing → `blessed` (restoration; `assets/data/spells/*.toml`). Each lays one status of up to three degrees (`max = 3`, `stacking = "refresh"`), lengthened by the caster's Presence (`[spell.grows] degrees`). |
| Hex spells (`kind = "hex"`) | evil_eye → `ill_wished` (enchantment, a ray), hex_of_weakness → `weakened` (necromancy, a ray), curse_of_sloth → `sluggish` (transmutation, a ray), sleep → `drowsy` (enchantment, a burst), web → `webbed` (transmutation, a burst; six degrees, a degree a turn). |
| The same statuses from elsewhere | `hasted` from the potion of quickness; `blessed` from a forgotten altar's outcome and a study mishap; `barkskin` from the druid's and ranger's class ability (the `barkskin` talent casts the spell); `sluggish` from the Numbing cold rider. Statuses are the unit, so these get the circle too (§9). |
| `seen` | `lit` ("glowing"), `blessed`, and every hex status have it; `hasted` and `barkskin` do not (`docs/dev/status-effects.md`: `effects_of` lists only statuses with `seen`). |
| Status light | `lit` carries `light = "spell_light"` ([235, 235, 250, 0], radius 6): `light/cache.rs` pushes one source per actor for the status light of the largest radius. Statuses are the only way an actor's own state sheds light besides a carried lantern, a creature type's light, worn items' affixes, and the Inner light talent (`ember`, [180, 110, 50, 0], radius 1: "you see what is beside you in the dark, and so does everything else"). |
| Light and sight | a cell is visible to an eye if its perceived brightness (luma `(54r + 183g + 19b) / 256`, plus UV for eyes with `sees_uv`) reaches the eye's threshold (40 normal, 12 low-light; darkvision within its radius regardless). Creature sight reads the same light (`sight::light`, tested equal to perception). A light-blind race (drow: `glare = 250`) is dazzled on a cell lit to its glare. |
| Rendering | three layers: the map grid (a console material), the free-glyph layer (creatures, particles, missiles: one dynamic mesh a frame), the UI console. Spell effects draw glyphs and particles and light the scene through `SpellLights`. The shaders are WESL; UV light is drawn as a violet tint (`lighting.rs::UV_TINT`). The font is JuliaMono alone, rasterised by `fontdue` into a 1:1 atlas and a 2× one; it has **no CJK ideographs** (checked: 0 of the 11 191 characters are in U+4E00 to U+9FFF; it has the geometric shapes and box drawing). |
| Effects policy | every effect has its own switch and all are off with reduced effects; "deliberately subtle" is the bar (spec §4.8, `docs/dev/rendering.md`); goldens are reserved for what only the GPU shows, 16 today. |

## 3. The circle: anatomy

All sizes are in **cell heights** (`h`, the font size in physical pixels: 18 at the default and scale 1, 12 to 36 by the setting). A cell is about `h / 2` wide, so a circle of radius `r` spans `2r` rows and about `4r` columns; a circle is inherently a few cells across, because a kanji must be about a cell tall to be read.

| Part | What | Size and placement |
|---|---|---|
| Centre | the actor's drawn position: its cell's centre, or its sliding position while it slides (`Motion`) | |
| Rings | one thin ring per effect (§4), the innermost at `R`, each next `0.1 h` farther out, at most four | buffs `R = 0.7 h` (just clear of the actor's cell: half a cell is `0.5 h` tall, `0.25 h` wide); line one physical pixel at scale 1, `h / 18` pixels in general, anti-aliased |
| Figure | a regular polygon inscribed in the innermost ring, its corners at the rune slots | sides one pixel like the rings |
| Runes | a kanji at each corner of the figure, **upright**, orbiting with the figure | centred at `R + 0.65 h` (buffs `1.35 h`), drawn `0.9 h` square; they sit outside the rings, so the rings never cross them |
| Seal | nothing more: the school is the rune's colour (§6) | |
| Extent | the farthest ink from the centre | buffs `1.8 h`: a circle 3.6 rows tall and about 7 columns wide at the default font |

The circle is drawn **under** the free-glyph layer (so the actor's own glyph, neighbouring creatures, particles, and missiles are over it) and **over** the map grid (terrain and objects show through its low alpha). Alpha: rings and sides 0.28, runes 0.6, of the school colour; nothing is lit by the scene's light (it is magic, and it is itself a light, §8), but it is darkened as free glyphs are during a spell effect's darkening (`VisualLight::keep`) so a fireball still dims it.

## 4. Composition: the one rule, and its sketches

**The rule.** For the effects of one kind (buffs, or hexes) on one actor, in **status order** (the order of `data/status.toml`, the order the status bar and `effects_of` already use):

1. **Slots**: `slots = n × ⌈3 / n⌉`: three for one effect, four for two, then `n` (3, 4, 5, 6). The figure is the regular polygon with that many corners, inscribed in the innermost ring.
2. **Runes**: the effects' runes fill the slots in sequence, repeating: `A A A`, `A B A B`, `A B C`, `A B C D`, `A B C D E`, `A B C D E F`. Every rune appears the same number of times, evenly spaced: the labelled figure has the full rotational symmetry of the polygon for every `n`.
3. **Rings**: `min(n, 4)` rings. One effect, one ring; four or more, four.
4. **Cap**: six effects show; a seventh and later (in status order) have no rune and no ring. No status today comes near it (four buffs and five hexes exist), and the rune ring has room for six runes of `0.9 h` at radius `1.35 h` (a circumference of `8.5 h`) and no more.
5. **Order is stable**: status order never changes within a run, so a rune's place changes only when an effect begins or ends; then every rune glides to its new slot over 0.4 s (§11), and the figure's corner count changes with the slot count (a square becomes a triangle in a blend of the two, drawn as the one fading out and the other in over the same 0.4 s).

Why status order and not the order cast: the snapshot carries statuses in status order and nothing of when each began; the order cast would be new saved state for a visual, and the owner's "clear rules" are better served by a fixed table (Light is always before Blessing; a player learns the figure as they learn the status bar).

Why rings for the count and not the polygon alone: one effect is a triangle of the same rune, and three effects are a triangle of three runes, so the polygon cannot tell one from three without reading the kanji; a ring per effect is read at a glance, as a double or triple circle is in the source material. The polygon's corner count then has no job but symmetry, which is what the owner asked of it.

**Sketches** (schematic: a cell is half as wide as it is tall, so a true circle is drawn here squashed; `·` rings, `─ ╱ ╲` sides, the runes as kanji). One buff, Light:

```
            光
          ╱    ╲
      ·  ╱      ╲  ·
     ·  ╱   @    ╲  ·
      · ╱         ╲ ·
       光 ─────── 光
```

Two buffs, Light and Blessing (status order: lit, blessed), two rings, a square `光 祝 光 祝`:

```
         光 ───── 祝
        ·│  · ·   │·
       · │ ·  @ · │ ·
        ·│  · ·   │·
         祝 ───── 光
```

Three buffs, Light, Blessing, Haste: three rings, a triangle `光 祝 速`. Four: four rings, a square `光 祝 速 木`.

One hex, Evil eye (`呪`), broken: dashed ring and sides, turning the other way:

```
            呪
          ╱    ╲
      ╌  ╱      ╲  ╌
     ╌  ╱   k    ╲  ╌
      ╌ ╱         ╲ ╌
       呪 ╌ ╌ ╌ ╌ 呪
```

Two hexes on a sleeping, webbed kobold (drowsy, webbed): two dashed rings, a square `眠 網 眠 網`.

The whole figure turns as one: the rings' dashes, the polygon, and the slots orbit; the kanji stay upright as they orbit (a turned kanji is not legible, and a stone that turns with the wheel reads as a wheel, not a circle of writing).

## 5. Buffs and hexes: distinct, and together

Three cues say "buff" or "hex", redundant on purpose so that one survives every setting:

| Cue | Buff | Hex | Survives |
|---|---|---|---|
| Line | **whole**: solid rings and sides | **broken**: rings and sides dashed (dash `0.25 h`, gap `0.15 h`, along the line) | every palette, reduced effects, frozen time |
| Turning | clockwise, one revolution in 60 s | counter-clockwise, one in 45 s (a hex is a touch more restless) | motion on only |
| Place, when both are on one actor | the **inner** circle, as in §3 | the **outer** circle: rings from `R = 2.0 h`, runes at `2.65 h`, extent `3.1 h` | always; alone, a hex circle uses the inner geometry too, so a lone hex on a creature is as compact as a lone buff |

Why buffs inside: a buff is the actor's own, worn close like armour; a hex is laid on from outside and cages. And the common case on the player (a buff or two, no hex) stays the compact figure; the large double figure appears only on an actor that is both, which is rare and worth a glance. The two circles counter-rotate, which also keeps a rune of the outer ring from sitting over one of the inner for long.

Rejected cues: an inverted polygon (meaningless once it turns); a mirrored or struck-through rune (hurts legibility); a different rune size (hides the count).

## 6. Schools: colour and light

Each of the seven schools has a colour, in `data/runes.toml` (`[[school]]`), applied through the palette as every content colour is (`Palette::apply`, `Role::Fg`: the high-contrast palette brightens it, the colour-blind palette daltonises it), so it is as legible as a creature's colour on the same palette:

| School | Colour | Why |
|---|---|---|
| evocation | ember orange `#e0803c` | fire and lightning |
| necromancy | grave violet `#a48cc8` | the bone-and-shadow school; keeps away from green, the one hue the colour-blind palette folds toward red |
| abjuration | steel blue `#80b4e0` | wards |
| restoration | warm gold `#e0c060` | the priest's light |
| transmutation | leaf green `#78c478` | growth and change (Barkskin, Haste, Web) |
| enchantment | rose `#d07ab0` | the mind and the will (Sleep, Evil eye) |
| divination | silver `#d8d8e8` | sight |

A rune is drawn in its school's colour; the rings and sides in the colour of the first rune in status order (one colour per ring set keeps the figure quiet). Two cues beyond colour carry the school so that no palette loses it: the rune itself (its kanji is one spell's), and the circle's **light** (§8), which is the school's colour and shows on the floor around the actor. The owner's "seal", a small glyph per school beside each rune, was tried on paper and dropped: at `0.3 h` it is a smudge, and it crowds the ring (§14).

## 7. The runes: one kanji per spell

One kanji per runed status, chosen for meaning first and for legibility at `0.9 h` (16 px at the default font) second: few strokes where there is a choice.

| Spell | Status | Kanji | Reading, meaning | Strokes | Why |
|---|---|---|---|---|---|
| Light | `lit` | 光 | hikari, light | 6 | the word itself; open and legible |
| Haste | `hasted` | 速 | soku, speed | 10 | 高速 "high speed"; its opposite 遅 is Curse of sloth's |
| Barkskin | `barkskin` | 木 | ki, tree | 4 | bark is 樹皮, tree-skin; the simplest tree |
| Blessing | `blessed` | 祝 | shuku, blessing | 9 | 祝福 "blessing" |
| Evil eye | `ill_wished` | 呪 | ju, curse | 8 | 呪い "a curse"; the status is Hexed |
| Hex of weakness | `weakened` | 弱 | jaku, weak | 10 | the word itself |
| Curse of sloth | `sluggish` | 遅 | chi, slow | 12 | 遅い "slow"; the pair of 速 |
| Sleep | `drowsy` | 眠 | min, sleep | 10 | 眠る "to sleep" |
| Web | `webbed` | 網 | mō, net | 14 | the densest; 糸 (thread, 6) is the fallback if it muddies at the small fonts |

Alternatives for the owner: 迅 (swift, 6) for Haste; 甲 (shell, armour, 5) for Barkskin; 目 (eye, 5) for Evil eye. `asleep` (a den's sleep), `radiant` (Holy radiance), and the potions' boosts are not spells' and get no rune (§9).

A new buff or hex spell's PR adds its status's row to `data/runes.toml` and its kanji to the font subset (§12); a load-time check refuses a buff or hex spell whose status has no rune.

## 8. The light: a rule in `warlock-core`

The owner: *"the runes probably need to give out light, including UV"*. Light is a rule (it decides what the player and creatures see), so it lives in core, deterministic, in turn steps, and the shader only shows it.

### 8.1 The presets

Seven presets in `data/lights.toml`, one per school, steady (no `stability`: magical light), **radius 1** (the actor's cell and its eight neighbours):

| Preset | Colour `[r, g, b, uv]` | Luma at the source | To UV eyes |
|---|---|---|---|
| `circle_evocation` | `[70, 38, 16, 90]` | 43 | 133 |
| `circle_necromancy` | `[48, 36, 64, 90]` | 41 | 131 |
| `circle_abjuration` | `[32, 48, 68, 90]` | 46 | 136 |
| `circle_restoration` | `[66, 52, 20, 90]` | 52 | 142 |
| `circle_transmutation` | `[30, 56, 30, 90]` | 48 | 138 |
| `circle_enchantment` | `[62, 34, 54, 90]` | 41 | 131 |
| `circle_divination` | `[46, 46, 54, 90]` | 46 | 136 |

For scale: `spell_light` (the Light spell) is `[235, 235, 250, 0]` at radius 6; `ember` (Inner light) `[180, 110, 50, 0]`, luma 120, radius 1; `item_glow_faint` luma 80 at radius 2; `fungus` luma 18 at radius 3. A circle's light is a quarter of an ember's: **just over the normal eye's threshold (40) on the actor's own cell, and under it a cell away** (falloff `(R² − d²) / R²` with `R² = 2`: half at `d = 1`, so 20 to 26 luma: seen by low-light eyes, 12, and no other). To eyes that see UV it is bright (130 and more at the source, about 65 a cell away): the rune-light is above all a UV light, as the owner asked.

### 8.2 The one hook

Each row of `data/runes.toml` names its school; the school names its preset; the status gets it **through the existing status light** (`StatusDef::light`, filled at load from the rune row for every runed status that has no `light` of its own; `lit` keeps `spell_light`, which is brighter in every way). Nothing new is read anywhere: `light/cache.rs` already pushes one source per actor for the status light of the largest radius, keyed by the actor, so an actor with Haste and Blessing sheds **one** light. Several runed statuses of radius 1: the first in status order wins the tie (a stable rule; summing would need a synthesised preset and a new source kind for a difference no eye can tell at these sizes). Severity and the time left do not scale it: a status's severity is hidden by the perception rule (§9), and a constant light keeps the source's shadow mask cached (the cache keys a source by actor and preset; a radius that changed every turn would be recomputed every turn). A light ends with its status, in the turn it ends; there is no fade in the rules (the drawn circle fades, §11).

### 8.3 Consequences, spelled out

| Consequence | What happens | Verdict |
|---|---|---|
| A buffed creature in the dark | its own cell reaches luma 40 to 52: **visible to normal eyes** when in line of sight, at any distance within the view range; its neighbours are not. A creature that casts Barkskin on itself in a dark hall is seen as a faint green glow. | wanted: the circle is a light in the fiction; today only Light does this |
| A hexed foe in the dark | the same: **hexing a foe marks it**. A Sleep burst on a dark den shows every sleeper as a rose glow for as long as the drowsiness lasts (12 s a degree). Web marks for six turns. | wanted (the owner's "visual identity" at range) |
| UV eyes | Rune sight (`sees_uv`), the potion of moth's eyes, UV-sighted races: a circled actor is **plainly lit** (130+) and so are its neighbours (65+): they see a hexed creature's neighbours in the dark too. | a real reward for UV sight, which has little to see today |
| Stealth | creature sight reads the same light: a player who casts Haste, Barkskin, or Blessing before sneaking **lights their own cell** to 40+ luma and is seen by any normal-eyed creature with them in line of sight, as Inner light already does (its description says so). Low-light and UV-eyed creatures see them from a cell away. | the trade: a buff is not free to a sneak; a side effect to play around, not a system (§8.5). The wiki states it |
| Hearing | none: light is not noise |
| Dazzle | a drow's glare is 250: never by a circle alone; a circle adds its 40 to 52 to the cell's light, so a drow in bright daylight (230 on open ground at noon) is dazzled by their own Blessing where they were not. UV counts only for UV eyes. | accepted: a light-blind caster in full sun; the wiki states the numbers |
| A hidden buff | `hasted` and `barkskin` have no `seen` today: a glow with no circle would show a status the look says nothing of. So **a runed status must have `seen`** (a load-time check): `hasted` gets `seen = "quickened"` and `barkskin` `seen = "bark-skinned"`, and look mode names them. | the small rules change this design makes |
| Mimics, the unseen | a disguised mimic is not in the snapshot and its statuses shed no light (it hides as a chest: `hides` keeps it out of the light cache too, a check); an actor out of line of sight lights its own cell, which is not visible, and nothing of it is drawn. | unchanged |
| Turn time | a radius-1 source costs a 3×3 shadow mask, cached while the actor stands. The light stress test runs 825 lights on 400×300; `bench-turns` keeps its budgets (`delf` at 16 ms a turn; a Sleep burst hexes at most a den). A rest with many circled sleepers in view: the sources are recomputed only when one moves. | measured in the PR: `cargo xtask bench-turns` and the light stress test before and after, with a `delf` floor of twenty hexed creatures |
| Light stability | none: steady, as crystals and runes are. | |

### 8.4 The drawn light

The shader's circle matches the core light's colour: the rings and runes are the school's colour, and with `glow` on they are a little emissive (`post::emissive`, 1.15) so the bloom is the school's hue over the actor's cell, where the core light also lies. UV is shown as the renderer shows UV light (`lighting.rs::UV_TINT`, a violet tint per unit of perceived UV: only an eye that sees UV perceives it, so only then does the floor around a circled actor take the violet). The figure itself is never UV-only: it is a drawn thing, seen by whoever sees the actor.

### 8.5 Talents: none (the coverage policy)

**No talents and no new `[[system]]` tag** (the owner, 2026-11-01: "no need for talents for this. gameplay effects are just side effect. this does not need to drive talents"). The circle's drawing is presentation only. Its light is a rule, but a side effect of buffs and hexes, not a system of its own: it rides on the existing `light` ("Carried light"), `vision` ("Vision, darkvision, UV"), and `status` systems through the one existing hook (a status's `light`, §8.2), and the talents those systems have already touch it where they touch any light (Rune sight sees its UV; Night eyes and Owl eyes see it from farther; Kindled, Lamplighter, and Inner light are about one's own lamp and glow, not these). The feature issue answers the template's "Talents and class abilities" with this paragraph. The core-side design stays as it is: the light is still a deterministic rule in `warlock-core`, never the shader's.

## 9. Perception: what shows on whom

The renderer reads the snapshot only (the hard rule):

| Whose | Source | Circled |
|---|---|---|
| The player | `PerceptionView::own_effects` (every status, status order) | every runed status |
| A seen creature | `effects_of(actor)` (statuses with `seen`, status order) | every runed status, since every runed status has `seen` (§8.3) |
| An unseen creature, a remembered cell | nothing in the snapshot | nothing; a circle never outlives the sight of its actor |

The circle says nothing the status bar or look mode does not: no severity (the snapshot carries none; the game is vague), no time left, no caster. The order of the runes is the status order every player can see in the status bar. A hex the player suffers (a creature's Evil eye) is their outer circle, by the same rules.

Statuses from other sources than the spell (the potion of quickness's `hasted`, the altar's `blessed`, the class ability's `barkskin`, the rider's `sluggish`) are the same statuses and show the same circle and shed the same light: the circle belongs to the status, and the magic is the same magic. The potions' boosts, Radiant, Asleep, Raging, and the cooldowns have no rune and no circle.

## 10. Rendering: a shader, not particles; performance

### 10.1 Two ways, one chosen

| | (a) A shader: an instanced quad per circled actor, signed-distance rings and polygon, runes sampled from a kanji atlas | (b) The particle system and the free-glyph layer: rings of `·`, sides of turned `─` strokes (as a bolt's), runes as free glyphs |
|---|---|---|
| Lines | exact one-pixel anti-aliased rings and sides at any radius and scale; dashes by angle | dotted rings (a `·` every `0.3 h`), sides of overlapping strokes with seams at the corners, as the lightning bolt shows them on purpose |
| Runes | a kanji is one texture sample from a square-slot atlas | the glyph atlas has cell-wide slots: a kanji is two cells wide, so it is two half-glyphs in two private-use slots, drawn as two quads with a seam when the position is not on a pixel |
| CPU a frame | one instance (about 64 bytes) per circled actor; the per-turn part (runes, schools, counts) is built when `Game` changes, the per-frame part is the angle and the fades: an `O(circles)` loop, no allocation | about 120 glyph quads per circle (two rings of forty dots, three sides of ten strokes, six runes of two halves) rebuilt into the dynamic mesh every frame: 6 000 quads for fifty circles, pushed through `FreeGlyphs::push` one by one |
| GPU | fifty quads of `3.6 h × 3.6 h` at `h = 18`: 4 200 px each, 0.2 M fragments a frame, each a dozen distance evaluations and up to six atlas taps; one draw call. At 1440p the frame is 3.7 M px, so the circles are a twentieth of a full-screen pass | 6 000 small quads, one draw call too; cheap, but the dots and strokes overdraw the floor many times |
| Look | thin, steady, the source material's | dotted and jagged: a lightning bolt's register, wrong for a steady circle |
| Code | a new `render/circles.rs` + `circles.wesl` (a `Material2d` like `free_glyph`, with a vertex-per-instance attribute set) and a rune atlas; about the size of `free_glyph.rs` | no new material; `ui/circles.rs` pushing glyphs; a two-slot rune trick in `atlas.rs` |

**(a), the shader.** It is the one that looks like the request, it costs the CPU nothing per frame worth measuring, and its runes are a texture sample. The particle system is kept for what it is for (sparks, foam, rain); the circle is not a particle thing: it does not scatter, die, or flicker.

### 10.2 The pipeline

- **The rune atlas** (`render/rune_atlas.rs`): at startup and on a font-size change, every rune in `data/runes.toml` is rasterised with `fontdue` from the rune font (§12) into **square slots of `0.9 h`** (and a 2× copy, as the glyph atlas has, for the slots' linear sampling while a rune orbits between pixels), one `R8Unorm` texture of at most 64 slots (eight by eight: 9 runes today, room for 64 spells). No text layout happens per frame; the shader takes a slot index.
- **The instance** (`CircleInstance`, built in `ui/circles.rs`): the centre in framebuffer pixels (the sliding position, from `Motion`), `h`, for each of the two circles (inner, outer): `rings` (0 to 4), `slots` (0, 3 to 6), six rune slot indices and six school colour indices (packed in two `u32`), `broken` (hex), `angle`, `fade` (0 to 1), and the blend of a slot-count change (the old slot count and a 0 to 1 blend). About 64 bytes. **Per turn** (when `Game` changes): the list of circled actors with their runes, schools, counts, and kinds, from the snapshot (§9). **Per frame**: the angle from `VisualTime`, the fades and blends from the per-turn list's begin and end times, the centre from `Motion`. The per-turn list is a `Vec` reused across frames, no allocation per frame.
- **The draw** (`circles.wesl`): one instanced quad per circle, `2 × extent` square, drawn after the map grid and before the free-glyph layer (`LAYER_Z` 0.5). The fragment: `d = length(p)`; rings: `|d − R_k| < half a pixel` smoothed; the polygon: the signed distance to a regular `slots`-gon of circumradius `R` turned by `angle` (one `atan2` and a fold), its edge `|sdf| < half a pixel`; dashes: `fract((atan2 − angle) × circumference / (dash + gap)) < dash share`, the same along a side by its parameter; the runes: for each slot `k < slots`, the slot's centre `c_k = (R + 0.65 h)(cos θ_k, sin θ_k)`, and if `p` is within the `0.9 h` square about `c_k`, one tap of the rune atlas at `(p − c_k)` scaled (the rune is upright: the square is axis-aligned, only `c_k` turns), with the school's colour. The colours come from a 7-entry uniform of palette-mapped school colours, uploaded when the palette changes. Everything is a function of the instance and `VisualTime`.
- **Culling**: the snapshot holds seen actors only, so unseen and off-level actors never reach the list; an actor whose quad lies wholly outside the map viewport's clip rect is skipped on the CPU (the free-glyph layer's `clip`), and the material clips to the viewport as the free glyphs do, so circles never bleed into the UI console. Overdraw: a quad is drawn once; two circles on one actor are one quad of the outer extent.
- **Wasm / WebGPU**: a `Material2d` with instancing works as `free_glyph` does today (the web build runs it); WESL, `R8Unorm` textures, and `atan2` are in the baseline. The rune font subset adds about 8 KB to the wasm (§12).

### 10.3 The budget and how it is measured

| Case | Circles | Budget |
|---|---|---|
| The common one: the player with a buff or two | 1 | 0.02 ms a frame |
| A crowd: the player on the generated 120×60 `delf` floor at full population with a Sleep burst and a Web on a den, twenty creatures hexed in view | 20 | 0.1 ms |
| The stress: fifty circles in view, two on the player (a bench fixture: `afflict` on the player and a dev-spawned den hexed by replayed `use:sleep` casts) | 50 | **0.2 ms a frame at 2560×1440**, against the 16.7 ms frame and `bench-frames`'s 0.89 ms today |

`cargo xtask bench-frames` gets a `circles` scenario row (the stress case above, as a replay) and `docs/perf.md` records it; the PR notes the numbers before and after. The CPU side is measured with the same bench's frame time, and the per-turn build is in the turn (`bench-turns`), where fifty actors' statuses are a few hundred lookups.

## 11. Motion, reduced effects, frozen time

| Setting | The circle |
|---|---|
| Default | turns (60 s and 45 s a revolution, opposite ways); a circle appears over 0.5 s and goes over 0.5 s (`fade`); a rune glides to a new slot and the polygon blends over 0.4 s; the kanji stay upright |
| `motion` off, or reduced effects | **stands still** at angle 0 for the inner and 180° for the outer circle, no fade, no glide: still information, no motion (the unaware mark's precedent). The core light is unchanged (it is a rule) |
| `glow` off | no emissive, no bloom |
| `circles` off (a new effect switch, in Settings beside `particles`; on by default) | not drawn at all; the light and the `seen` text stay: a setting never changes a rule |
| `--freeze-time T` | the angle at `T` seconds; a capture at `T = 1` shows the first fade complete and the figure 6° turned |
| A sliding creature | the circle slides with it (the `Motion` position) |
| A spell effect's darkening | darkens the circle as it darkens free glyphs |

Nothing the circle does is read by the rules; the light (§8) is the rules' and is read by nothing drawn except the lighting pass, which already draws every light.

## 12. Data, the font, checks, verification

**`data/runes.toml`** (presentation plus the light's school; read into `Content::runes()`; **in the fingerprint** because the school decides a light, a rule: a changed school changes what creatures see, as a changed preset would):

```toml
# Spell circles (spec §4.8 "Spell circles", §8 "Status effects"): the rune and the school of
# every buff's and hex's status. The circle is drawn from them; the school's light is shed
# by the status (lights.toml, circle_<school>).
[[school]]
id = "evocation"
color = "#e0803c"

[[rune]]
status = "lit"
kanji = "光"
school = "evocation"
```

Load-time checks (`content/runes.rs`): every row's status exists and has `seen`; its `kanji` is exactly one character; its school is one of the seven (`weapon_skills.toml`'s `role = "school"` rows); every `kind = "buff"` or `"hex"` spell's status has a row; every school has a colour and a `circle_<school>` light; no two rows share a kanji.

**The rune font.** A **subset of Noto Sans JP Medium** (SIL OFL 1.1, no Reserved Font Name, so a subset may be shipped under its own file name) holding the runes' kanji and nothing else: `assets/fonts/NotoSansJP-Runes.ttf`, about 8 KB for nine kanji (a CJK glyph is 500 to 900 bytes), with `assets/fonts/NotoSansJP-OFL.txt` beside it, a paragraph in `about.hbs` (so `THIRD_PARTY_LICENSES.html` names it as it names JuliaMono), and the README's font line. The subset is made with fontTools' `pyftsubset` (`pyftsubset NotoSansJP-Medium.ttf --text="光速木祝呪弱遅眠網" --output-file=...`), the command kept in `assets/fonts/README.md`; a new rune's PR runs it again. The check that the subset is current is a test in `crates/warlock` (`render/rune_atlas.rs`): every kanji of `data/runes.toml` has a glyph in the rune font (`fontdue::Font::lookup_glyph_index != 0`), and `cargo xtask web`'s size line reports the font. Noto Sans over Noto Serif: at 16 px a sans kanji keeps its strokes apart; the serif's hairlines go (an open question for the owner's taste, §15).

**Verification** (CLAUDE.md "Verification"):

| What | How |
|---|---|
| The rule (§4): slots, sequence, rings, cap, order, and the blend | unit tests of the pure layout in `render/circles.rs` (`layout(effects) -> Circle`), one per row of §4 and each sketch |
| The light (§8): the preset per status, the tie, a hexed creature seen in the dark, UV eyes, a drow dazzled, a mimic shedding none | core tests in `world/light_tests.rs` and `world/circle_tests.rs` through the game's own paths (`afflict`, a replayed cast, `perception().light`) |
| The data checks | `content/runes.rs` tests with bad files |
| The font | the subset test above |
| What is drawn | **one golden** (`tests/golden/manifest/circles.toml`): the showcase start room, `afflict:lit afflict:blessed afflict:weakened`, `freeze_time = 1`, scale 1: the player's two-ring square of 光 and 祝 inside a one-ring dashed triangle of 弱, and the light on the floor. Nothing cheaper sees a shader's output: the circle is not in the UI console (no text snapshot), not in the snapshot (no dump), and the policy's list (lighting, glow, spell effects, one overlay) is exactly what it is. One entry, not two: the reduced-effects figure is the same at angle 0, which the layout tests cover |
| Performance | the `bench-frames` row and the `bench-turns` and light stress runs (§10.3) |
| By eye | captures at `--freeze-time` 0.1, 1, 10, 30 at `--scale 1` and `--window 2560x1440 --scale 2`, on the default and the colour-blind palettes, and a drow at noon |

The wiki (the game is vague, the wiki is accurate) gets a "Spell circles" section on the magic page: the rule, the kanji per spell and the school colours (a table generated from `runes.toml`, `xtask/src/wiki/`), and the lights' numbers in the lights table. The spec gets §4.8 "Spell circles" and a line under §8 "Status effects" for the light; `docs/dev/rendering.md` and `docs/dev/status-effects.md` their paragraphs.

## 13. Implementation plan

| Slice | What | Touches |
|---|---|---|
| C1 | the data and the light: `data/runes.toml`, its checks, `seen` on `hasted` and `barkskin`, the seven presets, the status light hook, the core tests, the wiki table and the spec line, the benchmarks | core, data, wiki; **saves unchanged** (a status light is read from content); no talents (§8.5) |
| C2 | the rune font subset and its licence, the rune atlas, the circle material and shader, `ui/circles.rs`, the `circles` switch, the golden, the docs | `crates/warlock`, assets; depends on C1 |
| C3 | the glide and blend, the polish pass with the owner's captures, the `bench-frames` row | `crates/warlock`; depends on C2 |

C1 is a rules PR and merges first; C2 and C3 are rendering PRs. None adds an action or saved state.

## 14. Rejected along the way

| Idea | Why not |
|---|---|
| A lone rune on the ring for one effect | the owner: an ioun stone. Every figure has at least three runes |
| The polygon's corner count as the count (`n + 2` sides, `n` runes) | asymmetric labelling for every `n` but the ones that divide the corners; the rings carry the count instead |
| A nested figure per effect (one triangle, then a hexagram, then three triangles) | past two it is a thicket of lines, and three triangles need nine runes, which do not fit |
| Six fixed slots filled in sequence (`A B C D A B` for four) | asymmetric at four and five |
| The order cast | not in the snapshot; new saved state for a visual |
| Severity as ring weight or a fading figure | severities are hidden by the perception rule; the drawn figure would say what look mode may not |
| A seal glyph per school beside each rune | a smudge at `0.3 h`; the colour, the kanji, and the light carry the school |
| An inverted polygon for hexes | meaningless once it turns |
| Summing several statuses' lights | a synthesised preset and a new source kind for a difference under the eye's threshold; the brightest-radius rule exists |
| Particles or free glyphs for the figure | dotted and jagged, and 120 quads a circle rebuilt every frame (§10.1) |
| A UV-only light | the owner asked for light "including UV"; the visible share is kept just over the threshold on the actor's own cell so a buff shows in the dark but lights nothing else |
| Talents for the circle's light (a "Quiet sigils" that sheds none, a "Blazing sigils" that reaches farther) | the owner: no talents; the light is a side effect, not a system (§8.5) |

## 15. Open questions for the owner

1. **The kanji** (§7): the nine picks, or any of the alternatives (迅 for Haste, 甲 for Barkskin, 目 for Evil eye, 糸 for Web)?
2. **Sans or serif** (§12): Noto Sans JP (clean at 16 px) or Noto Serif JP (brush-like, older, but its hairlines go at the small fonts)? A capture of both at 18 and 12 px can decide.
3. **The light's visible share** (§8.1): just over the normal threshold on the actor's own cell (a buffed creature is seen in the dark; a buffed sneak is seen), or UV-only (seen only by UV eyes; no stealth cost)? The design takes the first: the stealth cost is the side effect the owner accepts.
4. **Buffs inside, hexes outside** (§5), or the reverse?
5. **The ring count as the count** (§4): rings (one to four) with a triangle of three copies for a single effect, as proposed, or would the owner rather have the count in the polygon and accept asymmetric labels at four and five?
6. **Status order for the runes**, or should the order of the status bar be reversed so the newest shows first (a bigger change, outside this design)?
7. **`seen` for Hasted and Bark-skinned** (§8.3): the design adds them (look mode will name a quickened creature). Fine?
8. **Size**: a circle 3.6 rows by 7 columns for the player's buffs at the default font. Smaller means smaller kanji; at `0.7 h` (13 px) 網 and 遅 are blots. Is 3.6 rows acceptable?
