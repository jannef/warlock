# Warlock — Spell circles: a rotating transmutation circle for every buff and hex

Design document for the owner's review (2026-11-01, revised the same day with the owner's answers: §16). Written against `origin/main` at a9a703a6 (the spell scaling slices S0 to S11). Nothing here is implemented; §14 cuts it into issues. Every number is a first guess kept in data or in one named constant, to be tuned with captures of the test scenario (§13).

**The owner's request** (verbatim): *"I want a spell effect to rotate around characters with active buffs. Kinda like transmutation circles in full metal alchemist. Some basic geometric shapes (such as triangles) to connect runes and glyphs as 'spell runes'. Pick one kanji for each buff spell to act as the spell rune. The system should have clear rules how to [combine the] runes (kanji) of different spells when multiple buffs are active."* Then: *"hexes should also have debuff circles. but they need to be visually distinct. Hexes and buffs also need to be able to combine visually when on same character -- so design both at once. The indicators should have visual identity, that reveals: buffs or hex, which spell school, how many effects are active."* Then: *"i still don't want just 1 rune spell spell. hoveing rune looks too much like an 'ioun stone'. I want them to have the feel of rotating transmutation circles of FMA -- so even simplest of spells need some symmetry."* Then: *"we want this to be done in shader probably, or perhaps particle system. it needs to be performant"*, *"the runes probably need to give out light, including UV"*, and *"no need for talents for this. gameplay effects are just side effect. this does not need to drive talents"*.

**The owner's answers to the first draft's questions** (verbatim): *"if the size gets too large because readability, then let's pick some simpler kana, that is readable with smaller font even if not fully 'accurate' meaning. i think let's do visible light too. you need to setup a test scenario with character that have access to all buffs. follow recommendations on other."* And: *"yes, all buffs should be visible to others. no matter what they are (barkskin, haste, whatever)"*.

## Contents

1. Summary
2. What exists today
3. The circle: anatomy and size
4. Composition: the one rule, and its sketches
5. Buffs and hexes: distinct, and together
6. Schools: colour and light
7. The runes: one character per spell, chosen to be read
8. The light: a rule in `warlock-core`
9. Perception: what shows on whom, and `seen` on every buff
10. Rendering: a shader, not particles; performance
11. Motion, reduced effects, frozen time
12. Data, the font, checks, verification
13. The test scenario: `--scenario circles`
14. Implementation plan: three slices, the visible one first
15. Rejected along the way
16. Decisions (owner, 2026-11-01)

## 1. Summary

| Piece | Decision |
|---|---|
| What is drawn | around every actor the player sees who has a **runed status** (a buff's or a hex's), a **spell circle**: concentric thin rings, a regular polygon inscribed in the innermost, and a **rune** (a kana or a low-stroke kanji) upright at each of the polygon's corners, the whole turning slowly about the actor. §3 |
| Size | **2.6 rows by 5.2 columns** at the default font (extent `1.3 h`, a rune `0.55 h` square: 10 px at 18 px): the actor's row and two thirds of the rows above and below, two columns either side. The first draft's 3.6 by 7 was too big (owner); the runes were shrunk and chosen for legibility at the small size. §3, §7 |
| The one composition rule | **a rune for every effect, repeated until there are at least three, evenly around the ring at the corners of a regular polygon; a ring for every effect, up to three.** One effect is a triangle of the same rune thrice (never a lone hovering rune); two are a square `A B A B`; three a triangle `A B C`; four a square; five a pentagon; six a hexagon. The ring count is the count to three ("three or more" at three); the distinct runes give it exactly. §4 |
| Buff or hex | buffs are **whole**: solid rings, solid sides, turning clockwise. Hexes are **broken**: dashed rings, dashed sides, turning counter-clockwise. On an actor with both, the buffs' circle is the inner one and the hexes' the outer, counter-rotating. §5 |
| School | the rune's colour (seven colours, one per school, through the palette like every colour), the circle's light (§8), and the rings' colour (the school of the first rune in status order). §6 |
| Runes | Light ヒ, Haste ハ, Barkskin 木, Blessing シ; Evil eye 目, Hex of weakness 小, Curse of sloth 止, Sleep ネ, Web 井: two to five strokes each, every silhouette distinct from the others at 10 px. In data: `data/runes.toml`, one row per status, with its school. §7 |
| Light | **a rule, not a visual** (the owner: visible light, yes): a runed status sheds a small, steady light of its school's colour **with a UV share**, radius 1, from `data/lights.toml`, through the one existing path (a status's `light`, the brightest radius wins). A buffed or hexed creature glows faintly in the dark, and plainly to eyes that see UV. §8 |
| `seen` on every buff | **every buff status carries `seen`, whatever its source** (owner): others see a hasted, bark-skinned, fortified, cat-eyed, warmed, or warded creature as they see a blessed one. Every runed status carries `seen` too, and a load-time check refuses one that does not. §9 |
| Perception | the circle is drawn from the snapshot only: the player's `own_effects`, and seen creatures' `effects_of` (statuses with `seen`). No severity, no time left: a circle says what the status bar says. §9 |
| Rendering | **a shader**: one instanced draw of a quad per circled actor, signed-distance rings and polygons, the runes sampled from a small atlas rasterised once at load. No CPU text layout, no particles. Budget 0.2 ms a frame at 1440p with fifty circles. §10 |
| Reduced effects | the circle stands still at its first angle and does not fade; the core light is unchanged. A `circles` switch turns the drawing off (never the light). §11 |
| Test scenario | `--scenario circles` in the test world: a lit hall, the player knowing every buff and hex spell with 300 mana and potions of mana, peaceful kobolds around them already buffed, hexed, both, and many, and one bark-skinned in a dark alcove for the glow. Every combination in one capture. §13 |
| Talents | **none, and no new `[[system]]` tag** (owner, 2026-11-01): the light's effect on vision is a side effect of buffs and hexes, not a system of its own. §8.5 |
| Slices | **1** the visible proof of concept (runes, `seen`, the font, the shader, the scenario, one golden); **2** the core light (presets, hook, tests, benches); **3** the polish (fades, glides, blends). §14 |

## 2. What exists today

| Piece | Today |
|---|---|
| Buff spells (`kind = "buff"`, all `shape = "self"`) | light → `lit` (evocation, level 0), haste → `hasted` (transmutation, 11), barkskin → `barkskin` (transmutation, 3), blessing → `blessed` (**abjuration**, 3; `assets/data/spells/*.toml`). Each lays one status of up to three degrees (`max = 3`, `stacking = "refresh"`), lengthened by the caster's Presence (`[spell.grows] degrees`). |
| Hex spells (`kind = "hex"`) | evil_eye → `ill_wished` (enchantment, 0, a ray), hex_of_weakness → `weakened` (necromancy, 3, a ray), curse_of_sloth → `sluggish` (transmutation, 3, a ray), sleep → `drowsy` (enchantment, 6, a burst of radius 1), web → `webbed` (transmutation, 7, a burst of radius 1; six degrees, a degree a turn). |
| The same statuses from elsewhere | `hasted` from the potion of quickness; `blessed` from a forgotten altar's outcome and a study mishap; `barkskin` from the druid's and ranger's class ability (the `barkskin` talent casts the spell); `sluggish` from the Numbing cold rider. Statuses are the unit, so these get the circle too (§9). |
| Other buffs | the potions' six boosts (`fortified`, `nimble`, `hale`, `keen`, `steadfast`, `imposing`), three visions (`cat_eyed`, `owl_eyed`, `moth_eyed`), four resistances (`warmed`, `chill_proof`, `earthed`, `poison_proof`); `spell_ward` (Battle caster), `light_footed`; `inspired`, `inspired_greatly`, `radiant`, `raging`, `walled`, `lit`, `blessed`. |
| `seen` | `lit` ("glowing"), `blessed`, `inspired`, `radiant`, `raging`, `walled`, and every hex status have it; `hasted`, `barkskin`, the boosts, the visions, the resistances, `spell_ward`, and `light_footed` do not (`docs/dev/status-effects.md`: `effects_of` lists only statuses with `seen`). |
| How spells are known | `Actor::spells` (studied from scrolls, `world/learn.rs`) and a class ability's talent (`Taken::spells`); `Actor::knows_spell` asks both. No scenario field gives a spell; no dev action does. |
| What a level places | `CreatureSpawn` (`content/level.rs`): a creature, its faction, alignment, npc id, route, gear, and `asleep` (a den's sleeper: `travel.rs::make_creature` adds the `asleep` status at one degree). No status beyond that. |
| Status light | `lit` carries `light = "spell_light"` ([235, 235, 250, 0], radius 6): `light/cache.rs` pushes one source per actor for the status light of the largest radius. Statuses are the only way an actor's own state sheds light besides a carried lantern, a creature type's light, worn items' affixes, and the Inner light talent (`ember`, [180, 110, 50, 0], radius 1: "you see what is beside you in the dark, and so does everything else"). |
| Light and sight | a cell is visible to an eye if its perceived brightness (luma `(54r + 183g + 19b) / 256`, plus UV for eyes with `sees_uv`) reaches the eye's threshold (40 normal, 12 low-light; darkvision within its radius regardless). Creature sight reads the same light (`sight::light`, tested equal to perception). A light-blind race (drow: `glare = 250`) is dazzled on a cell lit to its glare. |
| Rendering | three layers: the map grid (a console material), the free-glyph layer (creatures, particles, missiles: one dynamic mesh a frame), the UI console. Spell effects draw glyphs and particles and light the scene through `SpellLights`. The shaders are WESL; UV light is drawn as a violet tint (`lighting.rs::UV_TINT`). The font is JuliaMono alone, rasterised by `fontdue` into a 1:1 atlas and a 2× one; it has **no kana and no CJK ideographs** (checked: none of its 11 191 characters are in U+3040 to U+30FF or U+4E00 to U+9FFF; it has the geometric shapes and box drawing). |
| Effects policy | every effect has its own switch and all are off with reduced effects; "deliberately subtle" is the bar (spec §4.8, `docs/dev/rendering.md`); goldens are reserved for what only the GPU shows, 16 today. |
| Test scenarios | one file each under `assets/campaigns/showcase/scenarios/` and `levels/` (`shrine`, `forge`, `spring`: a hand-made room, the scenario's own character, `wield = ["lantern"]`), listed in CLAUDE.md's `--scenario` line; the showcase is the test world (`kind = "test"`, the `showcase` feature, never in a release). |

## 3. The circle: anatomy and size

All sizes are in **cell heights** (`h`, the font size in physical pixels: 18 at the default and scale 1, 12 to 36 by the setting). A cell is about `h / 2` wide, so a circle of radius `r` spans `2r` rows and about `4r` columns.

**The size.** The first draft's circle (extent `1.8 h`: 3.6 rows by 7 columns) was too big (owner). The new extent is **`1.3 h`: 2.6 rows by 5.2 columns** at the default font: the actor's row and two thirds of the rows above and below it, the actor's column and two columns either side, the edge just touching the third. Why `1.3 h` and not less: the circle must clear the actor's glyph (half a cell is `0.5 h` tall), hold up to three rings a pixel apart, and still have a rune band outside them whose characters are legible; `0.55 h` is 10 px at the default font, the floor at which a two- to five-stroke kana or kanji is still read (§7), and `1.3 h` is the least extent that holds all three without the rings and runes touching. A true 3 by 5 is not a circle (a circle `n` rows tall is `2n` columns wide); 2.6 by 5.2 is the circle inside the 3 by 5 box with the margin a one-pixel ring needs. At `--scale 2` or a 36 px font everything doubles in pixels and nothing in cells.

| Part | What | Size and placement |
|---|---|---|
| Centre | the actor's drawn position: its cell's centre, or its sliding position while it slides (`Motion`) | |
| Rings | one thin ring per effect (§4), the innermost at `R`, each next `0.1 h` farther out, at most three | buffs `R = 0.48 h` (just clear of the glyph's ink: a glyph's cap height is about `0.7 h`, so its ink ends about `0.35 h` from the centre); rings at `0.48`, `0.58`, `0.68 h`; line one physical pixel at scale 1, `h / 18` pixels in general, anti-aliased; the gap between rings is `0.1 h`, 1.8 px at the default font, 3.6 at 36 px; the third ring's outer edge (`0.68 h` plus half a pixel, `0.028 h` at 18 px) stays inside the rune band's `0.725 h` |
| Figure | a regular polygon inscribed in the innermost ring, its corners at the rune slots | sides one pixel like the rings |
| Runes | a character at each corner of the figure, **upright**, orbiting with the figure | centred at `1.0 h`, drawn `0.55 h` square (10 px at 18, 20 px at 36), so they span `0.725` to `1.275 h`: outside the rings, which never cross them |
| Seal | nothing more: the school is the rune's colour (§6) | |
| Extent | the farthest ink from the centre | buffs `1.3 h`: 2.6 rows tall and 5.2 columns wide at the default font |

The circle is drawn **under** the free-glyph layer (so the actor's own glyph, neighbouring creatures, particles, and missiles are over it) and **over** the map grid (terrain and objects show through its low alpha). Alpha: rings and sides 0.28, runes 0.7 (up from the first draft's 0.6: a 10 px character needs the contrast), of the school colour; nothing is lit by the scene's light (it is magic, and it is itself a light, §8), but it is darkened as free glyphs are during a spell effect's darkening (`VisualLight::keep`) so a fireball still dims it.

## 4. Composition: the one rule, and its sketches

**The rule.** For the effects of one kind (buffs, or hexes) on one actor, in **status order** (the order of `data/status.toml`, the order the status bar and `effects_of` already use):

1. **Slots**: `slots = n × ⌈3 / n⌉`: three for one effect, four for two, then `n` (3, 4, 5, 6). The figure is the regular polygon with that many corners, inscribed in the innermost ring.
2. **Runes**: the effects' runes fill the slots in sequence, repeating: `A A A`, `A B A B`, `A B C`, `A B C D`, `A B C D E`, `A B C D E F`. Every rune appears the same number of times, evenly spaced: the labelled figure has the full rotational symmetry of the polygon for every `n`.
3. **Rings**: `min(n, 3)` rings. One effect, one ring; three or more, three. (The first draft had four; at the new size a fourth ring would push the runes out or close the gaps to a smear. The distinct runes give the exact count: four runes `A B C D` on three rings is four effects.)
4. **Cap**: six effects show; a seventh and later (in status order) have no rune. No status today comes near it (four buffs and five hexes exist), and the rune ring has room for six runes of `0.55 h` at radius `1.0 h` (a circumference of `6.3 h`) with gaps between, and no more.
5. **Order is stable**: status order never changes within a run, so a rune's place changes only when an effect begins or ends; then every rune glides to its new slot over 0.4 s (§11, slice 3), and the figure's corner count changes with the slot count (a square becomes a triangle in a blend of the two, drawn as the one fading out and the other in over the same 0.4 s). In slice 1 the change is a cut.

Why status order and not the order cast: the snapshot carries statuses in status order and nothing of when each began; the order cast would be new saved state for a visual, and the owner's "clear rules" are better served by a fixed table (Light is always before Blessing; a player learns the figure as they learn the status bar).

Why rings for the count and not the polygon alone: one effect is a triangle of the same rune, and three effects are a triangle of three runes, so the polygon cannot tell one from three without reading the characters; a ring per effect is read at a glance, as a double or triple circle is in the source material. The polygon's corner count then has no job but symmetry, which is what the owner asked of it.

**Sketches** (schematic, at about the real proportions: a cell is half as wide as it is tall, so a true circle is drawn here squashed; `·` rings, `─ ╱ ╲ │` sides, the runes as their characters; the figure is 2.6 rows tall, so the runes above and below sit in the neighbouring rows). One buff, Light:

```
         ヒ
      ·╱   ╲·
     · ╱ @ ╲ ·
      ヒ ─── ヒ
```

Two buffs, Light and Blessing (status order: lit, blessed), two rings, a square `ヒ シ ヒ シ`:

```
      ヒ ──── シ
      │·· @ ··│
      シ ──── ヒ
```

Three buffs, Blessing, Haste, Barkskin (status order: blessed, hasted, barkskin): three rings, a triangle `シ ハ 木`. Four, Light too: three rings, a square `ヒ シ ハ 木`.

One hex, Evil eye (`目`), broken: dashed ring and sides, turning the other way:

```
         目
      ╌╱   ╲╌
     ╌ ╱ k ╲ ╌
      目 ╌ ╌ 目
```

Two hexes on a sleeping, webbed kobold (drowsy, webbed): two dashed rings, a square `ネ 井 ネ 井`.

The whole figure turns as one: the rings' dashes, the polygon, and the slots orbit; the characters stay upright as they orbit (a turned character is not legible, and a stone that turns with the wheel reads as a wheel, not a circle of writing).

## 5. Buffs and hexes: distinct, and together

Three cues say "buff" or "hex", redundant on purpose so that one survives every setting:

| Cue | Buff | Hex | Survives |
|---|---|---|---|
| Line | **whole**: solid rings and sides | **broken**: rings and sides dashed (dash `0.2 h`, gap `0.12 h`, along the line) | every palette, reduced effects, frozen time |
| Turning | clockwise, one revolution in 60 s | counter-clockwise, one in 45 s (a hex is a touch more restless) | motion on only |
| Place, when both are on one actor | the **inner** circle, as in §3 | the **outer** circle: rings from `R = 1.35 h` (`1.35`, `1.45`, `1.55 h`, clear of the inner circle's `1.275 h`), runes `0.55 h` centred at `1.88 h` (`1.605` to `2.155 h`, clear of the third ring's edge), extent `2.15 h`: 4.3 rows by 8.6 columns | always; alone, a hex circle uses the inner geometry too, so a lone hex on a creature is as compact as a lone buff |

Why buffs inside: a buff is the actor's own, worn close like armour; a hex is laid on from outside and cages. And the common case on the player (a buff or two, no hex) stays the compact figure; the large double figure appears only on an actor that is both, which is rare and worth a glance. The two circles counter-rotate, which also keeps a rune of the outer ring from sitting over one of the inner for long. The double figure is the one place the footprint is big; the alternative, one rune band holding both kinds, was rejected because nothing but colour would then say which rune is the hex (§15).

Rejected cues: an inverted polygon (meaningless once it turns); a mirrored or struck-through rune (hurts legibility); a different rune size (hides the count).

## 6. Schools: colour and light

Each of the seven schools has a colour, in `data/runes.toml` (`[[school]]`), applied through the palette as every content colour is (`Palette::apply`, `Role::Fg`: the high-contrast palette brightens it, the colour-blind palette daltonises it), so it is as legible as a creature's colour on the same palette:

| School | Colour | Why |
|---|---|---|
| evocation | ember orange `#e0803c` | fire and lightning (Light) |
| necromancy | grave violet `#a48cc8` | the bone-and-shadow school (Hex of weakness); keeps away from green, the one hue the colour-blind palette folds toward red |
| abjuration | steel blue `#80b4e0` | wards (Blessing) |
| restoration | warm gold `#e0c060` | the priest's healing light |
| transmutation | leaf green `#78c478` | growth and change (Barkskin, Haste, Curse of sloth, Web) |
| enchantment | rose `#d07ab0` | the mind and the will (Sleep, Evil eye) |
| divination | silver `#d8d8e8` | sight |

A rune is drawn in its school's colour; the rings and sides in the colour of the first rune in status order (one colour per ring set keeps the figure quiet). Two cues beyond colour carry the school so that no palette loses it: the rune itself (its character is one spell's), and the circle's **light** (§8), which is the school's colour and shows on the floor around the actor. The owner's "seal", a small glyph per school beside each rune, was tried on paper and dropped: at `0.3 h` it is a smudge, and it crowds the ring (§15).

## 7. The runes: one character per spell, chosen to be read

The owner: *"if the size gets too large because readability, then let's pick some simpler kana, that is readable with smaller font even if not fully 'accurate' meaning."* So: one character per runed status, chosen **for legibility at `0.55 h` (10 px at the default font) first** and for meaning second. Two to five strokes each, kana and kanji mixed, and no two silhouettes alike at 10 px (a box, a lattice, a cross, a pair of diverging strokes, three slanted strokes, a hook with dots, an angle with a curve, a roof with a tail, two blocky angles). The first draft's kanji (光 速 木 祝 呪 弱 遅 眠 網, six to fourteen strokes) were chosen for meaning and were blots below 16 px.

| Spell | Status | Rune | Strokes | Reading and sense | Why this one (readability first) |
|---|---|---|---|---|---|
| Light | `lit` | ヒ | 2 | katakana *hi*, the first syllable of 光 *hikari*, light | an open angle and a curve, nothing else in the set looks like it; 光 (6) closes up at 10 px and 日 (4, sun) is a box like 目 |
| Haste | `hasted` | ハ | 2 | katakana *ha*, the first syllable of 速い *hayai*, fast | two diverging strokes, read at any size, and they look like speed lines; 速 (10) is a blot |
| Barkskin | `barkskin` | 木 | 4 | *ki*, tree (bark is 樹皮, tree-skin) | the simplest tree, a cross with legs, clean at 10 px; kept from the first draft |
| Blessing | `blessed` | シ | 3 | katakana *shi*, the first syllable of 祝 *shuku*, blessing | three slanted strokes, unlike anything else here; 祝 (9) is a blot, and 十 (2, a cross) reads as a plus sign among the polygon's lines |
| Evil eye | `ill_wished` | 目 | 5 | *me*, eye | the eye itself, the one box in the set (Light is not 日 for that reason); 呪 (8) muddies |
| Hex of weakness | `weakened` | 小 | 3 | *shō*, small, lesser | a hook with two dots, open and unmistakable; 弱 (10) is a blot |
| Curse of sloth | `sluggish` | 止 | 4 | *shi*, stop, halt (sloth halts you) | two blocky angles, distinct from the cross of 木 (牛, ox, would blur into it); 遅 (12) is a blot |
| Sleep | `drowsy` | ネ | 4 | katakana *ne*, the first syllable of 寝る *neru*, to sleep | a roof with a tail, asymmetric where 木 is symmetric; 眠 (10) is a blot, 月 (moon) is a box like 目 |
| Web | `webbed` | 井 | 4 | *i*, a well; the lattice reads as a net | the one lattice in the set; 網 (14) is the densest of all and 糸 (6) closes up |

`asleep` (a den's sleep), `radiant` (Holy radiance), `inspired`, `walled`, `raging`, and the potions' boosts, visions, and resistances are not spells' and get no rune (§9): a circle is a spell's mark. (They do get `seen`, §9.)

A new buff or hex spell's PR adds its status's row to `data/runes.toml` and its character to the font subset (§12); a load-time check refuses a buff or hex spell whose status has no rune. The convention for a new rune: a kana or a kanji of five strokes or fewer whose silhouette at 10 px is unlike every rune already in the table; the PR shows a 10 px capture of the set.

## 8. The light: a rule in `warlock-core`

The owner: *"the runes probably need to give out light, including UV"*, and, asked whether the visible share should stay: *"let's do visible light too."* Light is a rule (it decides what the player and creatures see), so it lives in core, deterministic, in turn steps, and the shader only shows it. This is slice 2 (§14): the proof of concept draws the circle without it.

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

For scale: `spell_light` (the Light spell) is `[235, 235, 250, 0]` at radius 6; `ember` (Inner light) `[180, 110, 50, 0]`, luma 120, radius 1; `item_glow_faint` luma 80 at radius 2; `fungus` luma 18 at radius 3; a `sconce` luma 187 at radius 7. A circle's light is a quarter of an ember's: **just over the normal eye's threshold (40) on the actor's own cell, and under it a cell away** (falloff `(R² − d²) / R²` with `R² = 2`: half at `d = 1`, so 20 to 26 luma: seen by low-light eyes, 12, and no other). To eyes that see UV it is bright (130 and more at the source, about 65 a cell away): the rune-light is above all a UV light, as the owner asked.

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
| A hidden buff | a glow with no circle would show a status the look says nothing of. So **every runed status has `seen`** (a load-time check), and with the owner's wider rule **every buff status has it** (§9). | the small rules change this design makes |
| Mimics, the unseen | a disguised mimic is not in the snapshot and its statuses shed no light (it hides as a chest: `hides` keeps it out of the light cache too, a check); an actor out of line of sight lights its own cell, which is not visible, and nothing of it is drawn. | unchanged |
| Turn time | a radius-1 source costs a 3×3 shadow mask, cached while the actor stands. The light stress test runs 825 lights on 400×300; `bench-turns` keeps its budgets (`delf` at 16 ms a turn; a Sleep burst hexes at most a den). A rest with many circled sleepers in view: the sources are recomputed only when one moves. | measured in the PR: `cargo xtask bench-turns` and the light stress test before and after, with a `delf` floor of twenty hexed creatures |
| Light stability | none: steady, as crystals and runes are. | |

### 8.4 The drawn light

The shader's circle matches the core light's colour: the rings and runes are the school's colour, and with `glow` on they are a little emissive (`post::emissive`, 1.15) so the bloom is the school's hue over the actor's cell, where the core light also lies. UV is shown as the renderer shows UV light (`lighting.rs::UV_TINT`, a violet tint per unit of perceived UV: only an eye that sees UV perceives it, so only then does the floor around a circled actor take the violet). The figure itself is never UV-only: it is a drawn thing, seen by whoever sees the actor.

### 8.5 Talents: none (the coverage policy)

**No talents and no new `[[system]]` tag** (the owner, 2026-11-01: "no need for talents for this. gameplay effects are just side effect. this does not need to drive talents"). The circle's drawing is presentation only. Its light is a rule, but a side effect of buffs and hexes, not a system of its own: it rides on the existing `light` ("Carried light"), `vision` ("Vision, darkvision, UV"), and `status` systems through the one existing hook (a status's `light`, §8.2), and the talents those systems have already touch it where they touch any light (Rune sight sees its UV; Night eyes and Owl eyes see it from farther; Kindled, Lamplighter, and Inner light are about one's own lamp and glow, not these). Each slice's issue answers the template's "Talents and class abilities" with one line: *none, presentation side effect, owner decision 2026-11-01*. The core-side design stays as it is: the light is still a deterministic rule in `warlock-core`, never the shader's.

## 9. Perception: what shows on whom, and `seen` on every buff

The renderer reads the snapshot only (the hard rule):

| Whose | Source | Circled |
|---|---|---|
| The player | `PerceptionView::own_effects` (every status, status order) | every runed status |
| A seen creature | `effects_of(actor)` (statuses with `seen`, status order) | every runed status, since every runed status has `seen` |
| An unseen creature, a remembered cell | nothing in the snapshot | nothing; a circle never outlives the sight of its actor |

The circle says nothing the status bar or look mode does not: no severity (the snapshot carries none; the game is vague), no time left, no caster. The order of the runes is the status order every player can see in the status bar. A hex the player suffers (a creature's Evil eye) is their outer circle, by the same rules.

Statuses from other sources than the spell (the potion of quickness's `hasted`, the altar's `blessed`, the class ability's `barkskin`, the rider's `sluggish`) are the same statuses and show the same circle and shed the same light: the circle belongs to the status, and the magic is the same magic. The potions' boosts, Radiant, Asleep, Raging, and the cooldowns have no rune and no circle.

**`seen` on every buff** (the owner: *"yes, all buffs should be visible to others. no matter what they are (barkskin, haste, whatever)"*). The rule, in `docs/dev/status-effects.md` with this design: **a buff status, whatever its source (a spell, a potion, an altar, a talent, a class ability, a style), carries `seen`**, so look mode names it on a creature and `effects_of` lists it; and **every status with a rune carries `seen`**, which a load-time check in `content/runes.rs` enforces (a runed status without `seen` is a content error). A buff is a status that lends something: a positive `penalty`, a `combat` bonus, a `cost` under 100, a `vision`, a `resist`, or a `light`. Slice 1 adds `seen` to every buff that lacks it:

| Status | `seen` |
|---|---|
| `hasted` | `quickened` |
| `barkskin` | `bark-skinned` |
| `fortified`, `nimble`, `hale`, `keen`, `steadfast`, `imposing` (the potions' boosts) | `fortified`, `nimble`, `hale`, `keen`, `steadfast`, `imposing` |
| `cat_eyed`, `owl_eyed`, `moth_eyed` (the potions' visions) | `cat-eyed`, `owl-eyed`, `moth-eyed` |
| `warmed`, `chill_proof`, `earthed`, `poison_proof` (the potions' resistances) | `warmed`, `chill-proof`, `earthed`, `proof against poison` |
| `spell_ward` (Battle caster's Spell guard) | `warded` |
| `light_footed` | `light-footed` |

Already `seen`: `lit` (glowing), `blessed`, `inspired` and `inspired_greatly` (inspired), `radiant` (shining), `raging`, `walled` (behind its shield). Not buffs, left as they are: `disguised` (a mimic's disguise: `seen` would undo it), `lurking` (a thief's ambush posture, not a lent power, ended silently by the next action; the player's only, so nothing else is ever looked at with it), the cooldowns (`exhausted`, `winded`, `spent`, `burnt_out`, `drained`, `graceless`, `spent_eyed`, `summoning`), and every harm. The hex statuses all have `seen` already (the first draft decided it; the rule applies to them naturally). The wiki's status table (generated) shows the new `seen` texts; the new text is what look mode says of a creature, so `cargo xtask wiki` is the only wiki change.

## 10. Rendering: a shader, not particles; performance

### 10.1 Two ways, one chosen

| | (a) A shader: an instanced quad per circled actor, signed-distance rings and polygon, runes sampled from a rune atlas | (b) The particle system and the free-glyph layer: rings of `·`, sides of turned `─` strokes (as a bolt's), runes as free glyphs |
|---|---|---|
| Lines | exact one-pixel anti-aliased rings and sides at any radius and scale; dashes by angle | dotted rings (a `·` every `0.3 h`), sides of overlapping strokes with seams at the corners, as the lightning bolt shows them on purpose |
| Runes | a character is one texture sample from a square-slot atlas | the glyph atlas has cell-wide slots: a kana at `0.55 h` is about a cell wide and fits one slot, but it is then placed on the cell grid, not on the ring, and cannot orbit between pixels |
| CPU a frame | one instance (about 64 bytes) per circled actor; the per-turn part (runes, schools, counts) is built when `Game` changes, the per-frame part is the angle and the fades: an `O(circles)` loop, no allocation | about 100 glyph quads per circle (two rings of thirty dots, three sides of eight strokes, six runes) rebuilt into the dynamic mesh every frame: 5 000 quads for fifty circles, pushed through `FreeGlyphs::push` one by one |
| GPU | fifty quads of `2.6 h × 2.6 h` at `h = 18`: 2 200 px each, 0.11 M fragments a frame (a double figure 6 000 px), each a dozen distance evaluations and up to six atlas taps; one draw call. At 1440p the frame is 3.7 M px, so the circles are a thirtieth of a full-screen pass | 5 000 small quads, one draw call too; cheap, but the dots and strokes overdraw the floor many times |
| Look | thin, steady, the source material's | dotted and jagged: a lightning bolt's register, wrong for a steady circle |
| Code | a new `render/circles.rs` + `circles.wesl` (a `Material2d` like `free_glyph`, with a vertex-per-instance attribute set) and a rune atlas; about the size of `free_glyph.rs` | no new material; `ui/circles.rs` pushing glyphs |

**(a), the shader.** It is the one that looks like the request, it costs the CPU nothing per frame worth measuring, and its runes are a texture sample. The particle system is kept for what it is for (sparks, foam, rain); the circle is not a particle thing: it does not scatter, die, or flicker.

### 10.2 The pipeline

- **The rune atlas** (`render/rune_atlas.rs`): at startup and on a font-size change, every rune in `data/runes.toml` is rasterised with `fontdue` from the rune font (§12) into **square slots of `0.55 h`** (and a 2× copy, as the glyph atlas has, for the slots' linear sampling while a rune orbits between pixels), one `R8Unorm` texture of at most 64 slots (eight by eight: 9 runes today, room for 64 spells). No text layout happens per frame; the shader takes a slot index.
- **The instance** (`CircleInstance`, built in `ui/circles.rs`): the centre in framebuffer pixels (the sliding position, from `Motion`), `h`, for each of the two circles (inner, outer): `rings` (0 to 3), `slots` (0, 3 to 6), six rune slot indices and six school colour indices (packed in two `u32`), `broken` (hex), `angle`, `fade` (0 to 1), and the blend of a slot-count change (the old slot count and a 0 to 1 blend; slice 3). About 64 bytes. **Per turn** (when `Game` changes): the list of circled actors with their runes, schools, counts, and kinds, from the snapshot (§9). **Per frame**: the angle from `VisualTime`, the fades and blends from the per-turn list's begin and end times, the centre from `Motion`. The per-turn list is a `Vec` reused across frames, no allocation per frame.
- **The draw** (`circles.wesl`): one instanced quad per circle, `2 × extent` square, drawn after the map grid and before the free-glyph layer (`LAYER_Z` 0.5). The fragment: `d = length(p)`; rings: `|d − R_k| < half a pixel` smoothed; the polygon: the signed distance to a regular `slots`-gon of circumradius `R` turned by `angle` (one `atan2` and a fold), its edge `|sdf| < half a pixel`; dashes: `fract((atan2 − angle) × circumference / (dash + gap)) < dash share`, the same along a side by its parameter; the runes: for each slot `k < slots`, the slot's centre `c_k = 1.0 h (cos θ_k, sin θ_k)`, and if `p` is within the `0.55 h` square about `c_k`, one tap of the rune atlas at `(p − c_k)` scaled (the rune is upright: the square is axis-aligned, only `c_k` turns), with the school's colour. The colours come from a 7-entry uniform of palette-mapped school colours, uploaded when the palette changes. Everything is a function of the instance and `VisualTime`.
- **Culling**: the snapshot holds seen actors only, so unseen and off-level actors never reach the list; an actor whose quad lies wholly outside the map viewport's clip rect is skipped on the CPU (the free-glyph layer's `clip`), and the material clips to the viewport as the free glyphs do, so circles never bleed into the UI console. Overdraw: a quad is drawn once; two circles on one actor are one quad of the outer extent.
- **Wasm / WebGPU**: a `Material2d` with instancing works as `free_glyph` does today (the web build runs it); WESL, `R8Unorm` textures, and `atan2` are in the baseline. The rune font subset adds about 6 KB to the wasm (§12).

### 10.3 The budget and how it is measured

| Case | Circles | Budget |
|---|---|---|
| The common one: the player with a buff or two | 1 | 0.02 ms a frame |
| A crowd: the player on the generated 120×60 `delf` floor at full population with a Sleep burst and a Web on a den, twenty creatures hexed in view | 20 | 0.1 ms |
| The stress: fifty circles in view, two on the player (a bench fixture: the `circles` scenario (§13) with `afflict` on the player and a dev-spawned den hexed by replayed `use:sleep` casts) | 50 | **0.2 ms a frame at 2560×1440**, against the 16.7 ms frame and `bench-frames`'s 0.89 ms today |

`cargo xtask bench-frames` gets a `circles` scenario row (the stress case above, as a replay) and `docs/perf.md` records it; the PR notes the numbers before and after. The CPU side is measured with the same bench's frame time, and the per-turn build is in the turn (`bench-turns`), where fifty actors' statuses are a few hundred lookups.

## 11. Motion, reduced effects, frozen time

| Setting | The circle |
|---|---|
| Default | turns (60 s and 45 s a revolution, opposite ways); a circle appears over 0.5 s and goes over 0.5 s (`fade`; slice 3); a rune glides to a new slot and the polygon blends over 0.4 s (slice 3); the characters stay upright. In slice 1 a circle appears and goes at once and a change of count is a cut |
| `motion` off, or reduced effects | **stands still** at angle 0 for the inner and 180° for the outer circle, no fade, no glide: still information, no motion (the unaware mark's precedent). The core light is unchanged (it is a rule) |
| `glow` off | no emissive, no bloom |
| `circles` off (a new effect switch, in Settings beside `particles`; on by default) | not drawn at all; the light and the `seen` text stay: a setting never changes a rule |
| `--freeze-time T` | the angle at `T` seconds; a capture at `T = 1` shows the first fade complete and the figure 6° turned |
| A sliding creature | the circle slides with it (the `Motion` position) |
| A spell effect's darkening | darkens the circle as it darkens free glyphs |

Nothing the circle does is read by the rules; the light (§8) is the rules' and is read by nothing drawn except the lighting pass, which already draws every light.

## 12. Data, the font, checks, verification

**`data/runes.toml`** (presentation in slice 1; from slice 2 also the light's school, and then **in the fingerprint** because the school decides a light, a rule: a changed school changes what creatures see, as a changed preset would; read into `Content::runes()`):

```toml
# Spell circles (spec §4.8 "Spell circles", §8 "Status effects"): the rune and the school of
# every buff's and hex's status. The circle is drawn from them; the school's light is shed
# by the status (lights.toml, circle_<school>; slice 2).
[[school]]
id = "evocation"
color = "#e0803c"

[[rune]]
status = "lit"
rune = "ヒ"
school = "evocation"
```

Load-time checks (`content/runes.rs`): every row's status exists and has `seen`; its `rune` is exactly one character; its school is one of the seven (`weapon_skills.toml`'s `role = "school"` rows); every `kind = "buff"` or `"hex"` spell's status has a row; every school has a colour (and, from slice 2, a `circle_<school>` light); no two rows share a rune.

**The rune font.** A **subset of Noto Sans JP Medium** (SIL OFL 1.1, no Reserved Font Name, so a subset may be shipped under its own file name) holding the runes' characters and nothing else: `assets/fonts/NotoSansJP-Runes.ttf`, about 5 KB for nine characters (a kana is 300 to 500 bytes, a CJK glyph 500 to 900), with `assets/fonts/NotoSansJP-OFL.txt` beside it, a paragraph in `about.hbs` (so `THIRD_PARTY_LICENSES.html` names it as it names JuliaMono), and the README's font line. The subset is made with fontTools' `pyftsubset` (`pyftsubset NotoSansJP-Medium.ttf --text="ヒハ木シ目小止ネ井" --output-file=assets/fonts/NotoSansJP-Runes.ttf`), the command kept in `assets/fonts/README.md`; a new rune's PR runs it again. The check that the subset is current is a test in `crates/warlock` (`render/rune_atlas.rs`): every rune of `data/runes.toml` has a glyph in the rune font (`fontdue::Font::lookup_glyph_index != 0`), and `cargo xtask web`'s size line reports the font. Sans, not serif (decision, §16): at 10 px a sans kana keeps its strokes apart and its stroke weight even; the serif's hairlines go.

**Verification** (CLAUDE.md "Verification"):

| What | How |
|---|---|
| The rule (§4): slots, sequence, rings, cap, order, and the blend | unit tests of the pure layout in `render/circles.rs` (`layout(effects) -> Circle`), one per row of §4 and each sketch |
| The light (§8): the preset per status, the tie, a hexed creature seen in the dark, UV eyes, a drow dazzled, a mimic shedding none | core tests in `world/light_tests.rs` and `world/circle_tests.rs` through the game's own paths (`afflict`, a replayed cast, `perception().light`); slice 2 |
| The data checks | `content/runes.rs` tests with bad files; the `seen` rule on every runed status |
| The scenario's loader additions (§13) | `content/scenario.rs` and `content/level.rs` tests: a scenario's `spells` are known at the start (`Actor::knows_spell`) and an unknown id is refused; a spawn's `status` list is on the creature at its degrees in the `actors` dump and an unknown status is refused |
| The font | the subset test above |
| What is drawn | **one golden** (`tests/golden/manifest/circles.toml`): the `circles` scenario (§13) after its capture replay, `freeze_time = 1`, scale 1: the player's three-ring square of `ヒ シ ハ 木`, around them a one-ring triangle, a two-ring dashed square, a double figure, and (slice 2) the glow of the bark-skinned kobold in the dark alcove. Nothing cheaper sees a shader's output: the circle is not in the UI console (no text snapshot), not in the snapshot (no dump), and the policy's list (lighting, glow, spell effects, one overlay) is exactly what it is. One entry, not two: the reduced-effects figure is the same at angle 0, which the layout tests cover |
| Performance | the `bench-frames` row and the `bench-turns` and light stress runs (§10.3); slices 2 and 3 |
| By eye | captures of the `circles` scenario at `--freeze-time` 0.1, 1, 10, 30 at `--scale 1` and `--window 2560x1440 --scale 2`, at the 12 px and 36 px font settings, on the default and the colour-blind palettes, and a drow at noon |

The wiki (the game is vague, the wiki is accurate) gets a "Spell circles" section on the magic page: the rule, the rune per spell and the school colours (a table generated from `runes.toml`, `xtask/src/wiki/`), and (slice 2) the lights' numbers in the lights table. The spec gets §4.8 "Spell circles" and (slice 2) a line under §8 "Status effects" for the light; `docs/dev/rendering.md` and `docs/dev/status-effects.md` their paragraphs.

## 13. The test scenario: `--scenario circles`

The owner: *"you need to setup a test scenario with character that have access to all buffs."* A room of the test world (`assets/campaigns/showcase/`, the `showcase` feature: never in a release), made as `shrine`, `forge`, and `spring` are: one scenario file, one level file, one line in CLAUDE.md's `--scenario` list. One capture of it shows every combination: buffs only (one, three), hexes only (one, two), both, many on the player, and the glow of a buffed creature in the dark.

### 13.1 Two small loader additions (slice 1)

The scenario format gives a pack and a lantern but no spell, and a level places creatures with gear and `asleep` but no other status. Two fields, in the spirit of `carry` and `asleep`:

- **A scenario's `spells`** (`ScenarioDef::spells`, `Vec<SpellId>`; default none): spell ids the player knows at the start of a run of it, on top of their class's. Applied where the starting pack is (`kit.rs::starting_pack`'s caller, as `carry` is: for the scenario's own character and a chosen one alike), into `Actor::spells`, which is what `Actor::knows_spell` asks and what saves and the state hash already carry (`world/learn_tests.rs`). The loader refuses an id that is not a spell's. A scenario of the test world is the only user; nothing forbids a campaign's.
- **A spawn's `status`** (`CreatureSpawn::status`, `Vec<(StatusId, u8)>`; legend `status = ["blessed", "weakened*2"]`, a status id with `*N` degrees as `carry` writes counts, default one degree): statuses the creature is placed with, added in `travel.rs::make_creature` beside `asleep` (`actor.effects.add(id, def, degrees, None)`), so a placed creature can start blessed, hexed, or both. The loader refuses an unknown status and more degrees than its `max`. The `actors` dump shows them; nothing else is new.

Both are content, deterministic, and in the save as every status and known spell already is; neither is an action, and neither changes the save format.

### 13.2 The scenario file

`assets/campaigns/showcase/scenarios/circles.toml`:

```toml
name = "Circles"
description = "Spell circles: a hall of peaceful kobolds to buff and hex, one already blessed, one hexed, one both, one bark-skinned in the dark. You know every buff and hex spell and have the mana to cast them again and again."
start_level = "circles"
seed = 1
# No character table (docs/dev/character-creation.md): the scenario gives the eyes, stats, hit
# points, and mana itself, so every spell below is cast from the start without a class's level
# table, and 300 mana casts the nine of them five times over (a round costs about 56:
# max(1, level) times the shape's share, docs/dev/spells.md "Cost"). A rest refills it, and
# the potions of mana do at once.
vision = "normal"
stats = { intelligence = 24, willpower = 24, presence = 24 }
hp = 30
mana = 300
# Every buff and hex spell (assets/data/spells/, kind = "buff" or "hex"), known from the start.
spells = ["light", "haste", "barkskin", "blessing", "evil_eye", "hex_of_weakness", "curse_of_sloth", "sleep", "web"]
wield = ["lantern"]
carry = ["potion_of_mana*5"]
```

A character-less scenario is an existing form (the `showcase` scenario's comment describes it); its caster level is 1 (`level × 100 ÷ 100`), which only learning reads, and casting reads the scores: Intelligence and Willpower 24 put every spell's roll on a high rung (`docs/dev/spells.md`). Why not a witch or a druid with a name: a level-1 class has a few dozen mana and knows two of the nine; `xp:N` and nine `study` actions would have to precede every capture. The `spells` field is the one addition that makes the scenario itself the fixture.

### 13.3 The level file

`assets/campaigns/showcase/levels/circles.toml`: a sconce-lit hall with the player at its centre, four kobolds about them already in a state the circle must show, and a pair of plain ones within a ray's range (six; a burst's is eight) to hex in the capture, all `faction = "peaceful"` so they stand still and let themselves be hexed (a hex is an attack: the kobold hexed in the capture may turn on the player, which a capture at turn 7 does not reach), and a dark alcove to the west, open to the hall through a gap in line with the player, holding a bark-skinned kobold more than seven cells (the sconce's radius) from every sconce, so that only its own circle's light (slice 2) shows it.

```toml
# Spell circles (spec §4.8): a lit hall of peaceful kobolds to buff and hex, each already in
# one state the circle must show (docs/design/buff-circles.md §13). West, a dark alcove with
# a bark-skinned kobold in it: seen by the faint light its circle sheds, and by nothing else.
arrival = "Kobolds stand about the hall, placid, each with a mark of magic on it."
name = "The Circles"
ambient = [6, 7, 9, 0]
map = '''
#########################
#.......#...............#
#.......#....k.k........#
#.......#...............#
#.......#..b.t...t.h....#
#.......#...............#
#..d.............@......#
#.......#...............#
#.......#..x.t...t.m....#
#.......#...............#
#.......#...............#
#########################
'''

[legend]
"@" = { terrain = "floor", spawn = "player" }
"t" = "sconce"
# Two plain kobolds side by side for the capture's casts: a Sleep burst (radius 1) takes both, a
# ray one.
"k" = { terrain = "floor", creature = "kobold", faction = "peaceful" }
# One buff: a triangle of シ on one solid ring.
"b" = { terrain = "floor", creature = "kobold", faction = "peaceful", status = ["blessed"] }
# Two hexes: a dashed square 小 止 小 止 on two dashed rings, turning the other way.
"h" = { terrain = "floor", creature = "kobold", faction = "peaceful", status = ["weakened", "sluggish"] }
# Both: a solid triangle of シ inside a dashed triangle of 目, counter-rotating.
"x" = { terrain = "floor", creature = "kobold", faction = "peaceful", status = ["blessed", "ill_wished"] }
# Many: three buffs, a triangle シ ハ 木 on three rings.
"m" = { terrain = "floor", creature = "kobold", faction = "peaceful", status = ["blessed", "hasted", "barkskin"] }
# In the dark alcove, in line with the player through the gap: a bark-skinned kobold, seen by
# its circle's green glow alone (slice 2; before it, unseen).
"d" = { terrain = "floor", creature = "kobold", faction = "peaceful", status = ["barkskin"] }
```

The player stands at (17, 6); the sconces at (13, 4), (17, 4), (13, 8), (17, 8) light the hall (the farthest kobold, `h` at (19, 4), is two cells from one) and reach the gap at (8, 6) (five and a half cells from the nearest) but not the alcove beyond it: `d` at (3, 6) is ten cells from the nearest sconce, in the dark, and in a straight line of sight from the player along row 6. The `k k` pair at (13, 2) and (15, 2) are within the burst range (8) and the ray range (6) of the player.

### 13.4 The capture

The scenario alone (no cast yet: the four pre-set kobolds, and the player with no circle):

```
cargo run -p warlock -- --scenario circles --scale 1 --freeze-time 1 --capture target/circles-start.png
```

The full picture, a replay of six casts (four buffs on the player: a three-ring square `ヒ シ ハ 木`; Sleep on the pair: dashed triangles of ネ on both; Evil eye on one of them: its second hex, a dashed square `目 ネ 目 ネ`), which is also the golden's entry (`tests/golden/manifest/circles.toml`: `scenario = "circles"`, `actions = "use:light use:haste use:barkskin use:blessing use:sleep:14,2 use:evil_eye:13,2"`, `freeze_time = 1`, `scale = 1`):

```
cargo run -p warlock-cli -- replay --create --scenario circles \
  --actions "use:light use:haste use:barkskin use:blessing use:sleep:14,2 use:evil_eye:13,2" \
  -o target/circles.json
cargo run -p warlock -- --replay target/circles.json --scale 1 --freeze-time 1 --capture target/circles.png
```

A hex may miss (the target's Willpower check): the seed is fixed, so the replay is the same every time, and the PR picks the cast cells that land on seed 1. To recast at will in play: the 300 mana, the five potions of mana (`eat:LETTER` on a potion, Consume), and `rest:1h` (mana comes back at 200 % of the normal pace while resting). The dump that checks the fixture without a window: `cargo run -p warlock-cli -- dump --scenario circles --layers view,actors` shows each kobold's statuses in the `actors` layer and the player's spells in `spells`.

## 14. Implementation plan: three slices, the visible one first

The owner needs a visible proof of concept first (2026-11-01), so the slices are reordered from the first draft (which put the core light first): slice 1 draws the circle from the snapshot with no new rule but `seen`; slice 2 adds the light, a rule; slice 3 polishes. Rendering reads perception only in every slice. None adds an action, a `KeyCommand`, or a save-format change; each answers the template's "Talents and class abilities" with *none, presentation side effect, owner decision 2026-11-01*. The text below is issue-ready (`.github/ISSUE_TEMPLATE/feature.md`).

### Slice 1 — Spell circles: the visible proof of concept (runes, `seen`, the font, the shader, the test scenario)

**What.** Around every actor the player sees who has a buff's or a hex's status, a slow-turning transmutation circle: thin rings, a polygon, and a kana or low-stroke kanji rune at each corner, in the spell's school colour; solid and clockwise for buffs, dashed and counter-clockwise for hexes, the hexes' circle outside the buffs' on an actor with both. A test scenario, `--scenario circles`, shows every combination in one capture. Design: `docs/design/buff-circles.md` §3 to §7, §9, §10, §13.

**Rules.** `assets/data/runes.toml` (`content/runes.rs`, `Content::runes()`): `[[school]]` colours and `[[rune]]` rows (status, rune, school) with the load-time checks (every buff and hex spell's status has a row; a runed status has `seen`; one character; a known school; no two rows share a rune; presentation only, not in the fingerprint yet). `seen` on every buff status that lacks it (§9's table: `hasted` "quickened", `barkskin` "bark-skinned", the six boosts, the three visions, the four resistances, `spell_ward` "warded", `light_footed`), and the rule in `docs/dev/status-effects.md`: a buff status carries `seen`, whatever its source. A scenario's `spells` (known at the start, into `Actor::spells`; an unknown id refused) and a spawn's `status = ["blessed", "weakened*2"]` (added in `make_creature` beside `asleep`; unknown or over `max` refused). The `circles` scenario and level files (§13), CLAUDE.md's `--scenario` line. No action, no save change, nothing in the state hash but what known spells and statuses already put there.

**What the player sees.** The circle (§3's sizes: extent `1.3 h`, rings at `0.48`, `0.58`, `0.68 h`, runes `0.55 h` at `1.0 h`; the double figure to `2.15 h`), turning (60 s and 45 s a revolution), appearing and going at once, a count change a cut (fades and glides are slice 3); still at its first angle with `motion` off or reduced effects; off with the new `circles` effect switch (Settings beside `particles`; on by default); frozen by `--freeze-time`. Drawn from the snapshot only (`own_effects`, `effects_of`). Look mode names the newly `seen` buffs on creatures ("a kobold (peaceful, quickened)"). The rune font: a subset of Noto Sans JP Medium (`assets/fonts/NotoSansJP-Runes.ttf`, its OFL beside it, `about.hbs`, the README; `pyftsubset` command kept). `render/rune_atlas.rs`, `render/circles.rs` + `circles.wesl` (a `Material2d`, one instanced quad per circled actor), `ui/circles.rs` (the per-turn list from the snapshot, the per-frame angle).

**Tests.** Layout unit tests (`render/circles.rs::layout`: one per row of §4 and each sketch); `content/runes.rs` tests with bad files, including a runed status without `seen`; the scenario `spells` and spawn `status` loader tests (known at the start; refused when bad); the font subset test (every rune has a glyph); the `circles` scenario dumps (`view,actors,spells`) and its replay created and verified; **one golden**, `tests/golden/manifest/circles.toml` (§13.4's replay; only the GPU shows a shader's output: not in the UI console, not in the snapshot); captures by eye at the 12 and 36 px fonts, scale 1 and 2, the default and colour-blind palettes. `cargo xtask wiki` (the status table's new `seen` texts; the rune table). `bench-frames` unchanged in this slice (the row comes with slice 3).

**Docs.** Spec §4.8 "Spell circles" (new); `docs/dev/rendering.md` (the circle layer, the atlas, the switch); `docs/dev/status-effects.md` (the `seen` rule, the rune table); `docs/dev/content.md` (a spawn's `status`); `docs/dev/campaign.md` or `character-creation.md` (a scenario's `spells`); CLAUDE.md (the scenario); the wiki's magic page "Spell circles" with the generated rune table. Talents: none, presentation side effect, owner decision 2026-11-01.

### Slice 2 — Spell circles: the light (a rule in core)

**What.** A buffed or hexed actor sheds a faint, steady light of its spell's school colour with a strong ultraviolet share: seen as a glow in the dark by normal eyes on its own cell, plainly by eyes that see UV, and the floor about it takes the school's hue. Design: `docs/design/buff-circles.md` §8, §12.

**Rules.** Seven presets in `data/lights.toml` (`circle_<school>`, §8.1's colours, radius 1, steady); `StatusDef::light` filled at load from the rune row for every runed status without a light of its own (`lit` keeps `spell_light`), through the existing status-light path in `light/cache.rs` (one source per actor, the largest radius; the first in status order on a tie); constant whatever the severity; ends with the status. `runes.toml` joins the fingerprint (a school now decides a light). Consequences as §8.3: a buffed creature is visible in the dark on its own cell (luma 40 to 52), a hexed one is marked, a buffed sneak is seen by normal eyes in line of sight (as Inner light already is), a drow in full sun is dazzled by their own Blessing, UV eyes see a circled actor's neighbours. A disguised mimic sheds none. No new system, no talents.

**What the player sees.** The glow on the floor around a circled actor in the lighting pass as any light; the violet tint only to UV eyes (`UV_TINT`); the circle's emissive with `glow` on (`post::emissive` 1.15). The `circles` scenario's dark-alcove kobold (`d`) appears as a faint green glow. Nothing new in text; look mode and the status bar are unchanged.

**Tests.** `world/circle_tests.rs` and `world/light_tests.rs` through the game's paths (`afflict`, a replayed cast, `perception().light`): the preset per status; the tie; a hexed creature seen in the dark at range and not its neighbour; low-light eyes a cell away; UV eyes the neighbours; a drow dazzled at noon and not at dusk; a mimic shedding none; creature sight equal to perception for a circle's light. The `circles` scenario's dump with the `light` layer; its golden updated for the glow (deliberately, the change explained). `cargo xtask bench-turns` and the light stress test before and after, with a `delf` floor of twenty hexed creatures (budgets kept). `cargo xtask wiki` (the lights table).

**Docs.** Spec §8 "Status effects" (the light line); `docs/dev/status-effects.md` and `docs/dev/content.md` (the hook and the fingerprint); the wiki's lights table and the stealth note on the magic page ("a buff lights your cell"). Talents: none, presentation side effect, owner decision 2026-11-01.

### Slice 3 — Spell circles: the polish (fades, glides, blends, the benchmark row)

**What.** The circle appears and goes over half a second, a rune glides to its new slot and the polygon blends between corner counts over 0.4 s when an effect begins or ends, and the frame-time benchmark gets a circles row. A polish pass with the owner's captures of the `circles` scenario tunes the alphas, line weights, speeds, and colours. Design: `docs/design/buff-circles.md` §4 (5), §10.3, §11.

**Rules.** None. Nothing in core changes.

**What the player sees.** The fade in and out (`fade`), the glide and the blend (the old slot count and a 0 to 1 blend in `CircleInstance`), all gone with `motion` off or reduced effects (the figure is then still and cut, as slice 1 drew it); frozen by `--freeze-time` (the fades complete instantly, as other fades do). Any tuned number from the pass, each in data or one named constant.

**Tests.** Layout tests for the blend (the old and new slot counts, the glide's path); the golden unchanged at `freeze_time = 1` (the fade complete) or updated deliberately if the pass changes a colour or weight; `cargo xtask bench-frames` gets the `circles` stress row (fifty circles in view: the `circles` scenario with `afflict` on the player and a dev-spawned den hexed by replayed `use:sleep` casts), recorded in `docs/perf.md` against the 0.2 ms budget at 2560×1440; captures at `--freeze-time` 0.1, 0.3, 1 for the fade.

**Docs.** `docs/dev/rendering.md` (the motion rows); `docs/perf.md`. Talents: none, presentation side effect, owner decision 2026-11-01.

## 15. Rejected along the way

| Idea | Why not |
|---|---|
| A lone rune on the ring for one effect | the owner: an ioun stone. Every figure has at least three runes |
| The polygon's corner count as the count (`n + 2` sides, `n` runes) | asymmetric labelling for every `n` but the ones that divide the corners; the rings carry the count instead |
| Four rings (the first draft) | at extent `1.3 h` a fourth ring either pushes the runes out to the old size or closes the gaps to a smear; three rings and the distinct runes carry the count |
| A nested figure per effect (one triangle, then a hexagram, then three triangles) | past two it is a thicket of lines, and three triangles need nine runes, which do not fit |
| Six fixed slots filled in sequence (`A B C D A B` for four) | asymmetric at four and five |
| The order cast | not in the snapshot; new saved state for a visual |
| Severity as ring weight or a fading figure | severities are hidden by the perception rule; the drawn figure would say what look mode may not |
| A seal glyph per school beside each rune | a smudge at `0.3 h`; the colour, the rune, and the light carry the school |
| An inverted polygon for hexes | meaningless once it turns |
| One rune band holding buffs and hexes together (solid rings inside, dashed outside, extent `1.5 h`) | nothing but colour would say which rune is the hex; an inverted or struck rune to mark it hurts legibility. The double figure (§5) is big but rare |
| Meaning-first kanji (光 速 祝 呪 弱 遅 眠 網) | blots at 10 px; the owner chose readable kana over exact meaning |
| An elliptical circle to fit 3 by 5 cells | not a circle; the source material's figures are round |
| Summing several statuses' lights | a synthesised preset and a new source kind for a difference under the eye's threshold; the brightest-radius rule exists |
| Particles or free glyphs for the figure | dotted and jagged, and 100 quads a circle rebuilt every frame (§10.1) |
| A UV-only light | the owner asked for light "including UV" and then for the visible share; it is kept just over the threshold on the actor's own cell so a buff shows in the dark but lights nothing else |
| A dev action to learn a spell, or `xp` and nine `study` actions before every capture of the scenario | a scenario's `spells` field makes the scenario the fixture (§13.1) |
| A witch or druid as the scenario's character | knows two of the nine spells at level 1 with a few dozen mana; a character-less scenario gives 300 mana and every spell |
| Talents for the circle's light (a "Quiet sigils" that sheds none, a "Blazing sigils" that reaches farther) | the owner: no talents; the light is a side effect, not a system (§8.5) |

## 16. Decisions (owner, 2026-11-01)

The first draft's open questions, answered by the owner where quoted and otherwise by the design's own recommendation at the owner's "follow recommendations on other":

1. **Size: 2.6 rows by 5.2 columns** (extent `1.3 h`), down from 3.6 by 7 (owner: too big). Runes `0.55 h` (10 px at the default font), rings at `0.48`, `0.58`, `0.68 h`. §3.
2. **The runes: kana and low-stroke kanji, chosen for legibility at 10 px over exact meaning** (owner): ヒ Light, ハ Haste, 木 Barkskin, シ Blessing; 目 Evil eye, 小 Hex of weakness, 止 Curse of sloth, ネ Sleep, 井 Web. The convention for new ones: five strokes or fewer, a silhouette unlike the rest at 10 px. §7.
3. **Sans: Noto Sans JP Medium**, subset to the runes (recommendation: even stroke weight and open counters at 10 px; a serif's hairlines go). §12.
4. **Visible light, with the UV share** (owner: "let's do visible light too"): just over the normal eye's threshold on the actor's own cell, under it a cell away, bright to UV eyes; the stealth cost accepted as a side effect. §8.
5. **Buffs inside, hexes outside** on an actor with both (recommendation: a buff is worn close, a hex cages). §5.
6. **Rings carry the count, capped at three**; one effect is a triangle of the same rune (recommendation, the cap new with the size). §4.
7. **Status order** for the runes, never the order cast (recommendation: it is in the snapshot and the status bar already). §4.
8. **`seen` on every buff status, whatever its source** (owner: "yes, all buffs should be visible to others. no matter what they are (barkskin, haste, whatever)"): the table in §9, the rule in `docs/dev/status-effects.md`, and a load-time check that every runed status has `seen`. Hexes keep theirs. Mimics' disguise and the thief's lurking are not buffs and stay unseen.
9. **A test scenario, `--scenario circles`** (owner), with two loader additions (a scenario's `spells`, a spawn's `status`) so the scenario itself is the fixture: every buff and hex spell known, 300 mana and potions of mana, kobolds already buffed, hexed, both, and many, and one in the dark for the glow. §13.
10. **Slices reordered: the visible proof of concept first** (owner), then the core light, then the polish. §14.
11. **No talents, no `[[system]]` tag** (owner, the first draft). §8.5.
