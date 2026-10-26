# Warlock — Animated objects: household junk, dancing blades, hollow armour, and panoplies that wear their loot

Design document for the owner's review (2026-10-26). Written against `origin/main` at 2daff477
(after #443, creature tags; #438, spell tiers; #442, undead batch 2). Nothing here is implemented;
§15 cuts it into issues and names the first slice. §16 records the owner's decisions of
2026-10-26 on the first draft's questions, and the sections each one changed.

## Contents

1. Summary
2. Research, tied to the decisions
3. What exists, and what this builds on
4. Representation: generated creature types, a composition per creature
5. The data
6. The four tiers
7. Composition: how a creature is made
8. Inherited powers
9. Drops: intact or broken into their parts
10. Tags, faction, immunities, and where they appear
11. What the player sees
12. Saves, determinism, performance
13. Talents and class abilities
14. Spec, wiki, and docs
15. Implementation plan
16. Decisions (owner, 2026-10-26)

## 1. Summary

**What the owner asked for**: an animated-objects group of creatures, household objects and
weapons at first, composites at higher levels (armour + shield + weapon). They are a little
special: the higher ones are made of **real magic items**, rolled at their level, whose powers
they **inherit**. Each component drops with 50 %; the rest break into **the parts they are made
of**. Very procedural; the low levels are harmless junk; the high ones may be very dangerous.
They are loot goblins: worth killing.

| Piece | Design |
|---|---|
| Representation | **ordinary creature types, generated at content load** (§16 D1): one per household **body** (`animated_candlestick`, at the body's level), and one per **(tier, level)** for the other tiers (`animated_blade_5` … `animated_blade_20`, a **family** `animated_blade` drawn as one candidate). Every rule reads them as it reads a kobold: no accessors, no derived profile. What differs per creature is only **what it is made of**: its body and components, rolled at spawn into the actor's **inventory and equipment** (already saved), and the rank in its weapon's skill (`Actor::weapon_skills`, already saved). **No new save state.** §4 |
| Data | `data/animated.toml` (tiers, formula constants, composition odds, drops); `data/animated/<body>.toml`, **one file per household body** (broom, stool, candlestick, teapot, ...), each naming a **real object** in `data/objects/` it is made of and its level. §5 |
| Tiers | **Household** (levels 1–8: a scuttling broom, a hopping stool; trivial damage), **Blades** (5–20: one real weapon, sometimes magic), **Hollow armour** (12–35: a suit, helm, sometimes a shield; fights with its gauntlets), **Panoply** (20–70: armour, weapon, shield or helm, then gloves, boots, a cloak, rings, an amulet as the level rises; magic and rare odds rising with level). §6 |
| Formulas | the undead roster's (#383): stats, `base_hp = 2.5 × L × factor`, dice on the `2 + 0.55L` curve flattening after 20, `xp = 5L + 10`; one **role** per tier; written into each generated type at load. §6.2 |
| Components | rolled with the loot system (`loot::make_as`, item level = the type's level, material by `Source::Gear(level)`), from a dedicated stream, **`animated/<level id>`** on a generated level (spawn order, after gear) and **`animated/<actor id>`** for any other spawn. §7 |
| Powers | **nothing new**: a creature already gets `Mods::worn` from what it wears and wields; **every** affix effect any actor honours applies to it, rings and amulets too (§16 D16); the gaps are listed. §8 |
| Drops | each component (the body too) falls **intact** with `intact = 50` %, else **breaks into its parts** (§16 D4): every gem set in it falls **loose** (the existing gem object, size and cut kept), every other part falls as **pieces of its material** (one generic object, `piece`, made of whatever material it carries, as the loose gem is made of its gem): a tin dagger with a ruby pommel gives a ruby and a piece of tin. Magic is lost. `object::break_up`, reusable by later sundering and fire. §9 |
| Identity | tags `animated` and `construct`; a new faction `animated`, hostile to the player only; no corpse, no blood; **immune to disease, poison, and bleeding** (a new creature field, §16 D17); never undead, never incorporeal; nothing can be stolen from them (§16 D13). §10 |
| Where | populations: household in Cealdhlāw and Fennmynster, blades and hollow armour in Fyrdelf; panoplies wherever danger reaches 20 (strays and out-of-depth draws bring them sooner, rarely). Armoury and workshop rooms later. §10 |
| What the player sees | the type's name, "an animated candlestick", "an animated blade", "an animated armour", with look mode adding the material ("of brass") and, as for any creature, ", wielding a long sword, wearing an iron helm"; glyph: **the item's own** for one-item creatures (household and blades), **`A`** for composites (§16 D5); **colour by material** (§16 D6), composites one colour until slice A6 brings a chromatic look. Never the level, never the magic until identified. §11 |
| Talents | a new system `animated`, with **Salvager** (more components survive), **Unbinder** (surer, harder blows against animated things), and **Appraiser's eye** (A4). No class ability. §13 |
| First slice | **A1, the household tier alone** (generation at load, bodies, the faction, tags, immunities, drops intact-or-nothing, presentation, the system and its two talents). Everything after it adds tiers and content on top; A6, chromatic composites, is a separate rendering issue. §15 |
| Deferred | flying missiles and bows (§6.3) and mimics: lines on #196 (§16 D10, D11) |

## 2. Research, tied to the decisions

| Finding | Source | Used in |
|---|---|---|
| Dancing weapons are made of a real weapon and fight with it: its brand, enchantment, and ego apply, and killing one drops the weapon | DCSS, *Tukima's Dance* and dancing weapons | §6.3, §8: blades wield a real item and inherit it |
| Animated armour, the flying sword, and the rug of smothering: the iconic low-to-mid animated objects, each one item with one trick (the rug grapples) | D&D 5e Monster Manual, "Animated Objects" | §5.2 the rug's `held`; §6 tier list |
| Animated objects are built from a base and "construction points" that buy features by size and material | Pathfinder 1e Bestiary, "Animated Object" | §5.2 a body's numbers from its object; §6.5 material by group |
| Treasure goblins: a rare, mostly harmless creature that is pure loot; players drop everything to chase one | Diablo III | §9 the drop rule; §6.1 household tier as a low-level reward |
| A mimic is an item that turns out to be a creature; the surprise is its point | NetHack, ADOM, D&D | §16 D11: deferred, a separate design |
| Brogue's spectral blades: short-lived summoned weapons in swarms; quick, fragile | Brogue CE | §6.3 blades are quick and fragile |
| Constructs: immune to poison and disease, no corpse, unaffected by fear | D&D, Pathfinder construct type | §10.1 the construct's immunities |

## 3. What exists, and what this builds on

| Piece | Today (`docs/dev/creatures.md`, `objects.md`, `magic-items.md`, `mapgen.md`, `content.md`) | What animated objects need |
|---|---|---|
| Creature types | `CreatureDef` per file in `data/creatures/`; one fixed `level`, stats, `base_hp`, `attack`, `natural`, `resist`, `speed`, `glyph`, `fg`, `name`, `plural`, `weight`; `CreatureId` an index in table order, in the content fingerprint | **generated** `CreatureDef`s, built at load from `data/animated.toml` and `data/animated/`, numbered after the shared creature files (§4.3) |
| Generated content at load | each creature type's corpse object is generated at load (`Content::corpses`, one `ObjectId` per type) | the same pattern, one step up: creature types themselves |
| Per-actor state | `Actor` already holds its own `inventory`, `equipment`, `weapon_skills`, `skills`, `stats`, `hp_base` (copied from the type at `travel.rs::make_creature`) | nothing new: the components go in `inventory`/`equipment`, the weapon rank in `weapon_skills` |
| Creature gear | `gear::roll` from `gear/<level id>` (generated levels, spawn order, after loot) or `gear/<actor id>`; `loot::make_as` at the floor's item level, material by `Source::Gear(creature level)`; overloading pieces left out | the same machinery with the **type's** level as item level, slot by slot (§7), no burden check |
| Magic on creatures | `Mods::worn` aggregates what any actor wears and wields: `Actor::effective_stats`, `max_hp`, `protection`, `resistances`, the attack profile's riders, `healing_rate`, step and attack cost, the glow | nothing: inheritance is free (§8) |
| Parts, materials, gems | an object is made of parts (`ObjectDef::parts`), a stack may carry its own (`Stack::parts`, `ItemPart`: a part, a material, a gem's size and cut, all by id); a loose gem is the one `gem` object made of its gem (`Stack::gem`); `made_of = ["any"]` is every material with a group | `object::break_up` reads a stack's resolved parts (§9.2) |
| Raw materials | **none**: no object is "some tin" | one generic object, `piece`, made of whatever material it carries, as `gem` is (§9.2) |
| Drops | what a dying creature carried falls where it dies; its type's `loot` chance adds one item from `drop/<actor id>` | a per-component **intact or broken** draw on the same stream (§9) |
| Danger draws | `Content::candidates` → `danger::Candidate (type, level, factor)`; `danger::weight(level, factor, danger)`; strays from the derived generic list | a **family** candidate: one entry for a tier's run of generated types (§10.3) |
| Status immunity | poison by `resist poison = 100` (a status with a `damage_type` meets resistance); disease only as a talent mod (`TalentMods::disease_immune`, `World::expose`); bleeding: nothing (a bloodless type still takes the status) | a creature field `immune` (§10.1) |
| Undead roster formulas | #383 "Conventions": stats by level and role, hit points, the dice curve, XP; checked by `content/undead_roster_tests.rs` and `creatures_follow_the_guideline` | the same constants, applied at load to each generated type (§6.2): the guideline test covers them for free |
| Tags | flat singular tags (#441), `tag:X` kill quests | `animated`, `construct` (§10.1) |

## 4. Representation: generated creature types, a composition per creature

### 4.1 The owner's choice, and what it costs

The first draft chose **one type per tier plus a saved per-creature instance** (`Actor::animated`
with a frozen profile: level, body, name, glyph, speed, attack, armour, resistances), read
through six new `Actor` accessors at ~25 call sites. The owner chose **generated types at load**
(§16 D1): "less to maintain, interacts with game logic well." This section redesigns around it.

| | First draft: type per tier + saved instance | **Now: generated types at load** |
|---|---|---|
| What the rules read | the type for most fields, an accessor for name, speed, attack, armour, resist, level, weight, glyph | **the type, always**: an animated candlestick is an ordinary `CreatureDef` |
| New reads in the rules | ~25 sites behind six accessors, kept honest by a grep test; every future rule reading a creature field must remember them | **none in the rules**; four small "family" reads (§4.4) and one presentation read in perception (§11.1) |
| Save state | `Actor::animated: Option<Box<Animated>>` (body, profile, level) | **none new**: body and components in `inventory`/`equipment`, the weapon rank in `weapon_skills`, all saved today |
| Level | continuous per creature, clamped from the drawn danger | the type's own `level`; a tier is a run of types, one per level (§4.2) |
| Content table | four types | **~107 generated types** (16 bodies + 16 + 24 + 51 levels) in the creature table and the fingerprint |
| Per-creature name | "animated brass candlestick", "animated suit of plate armour" | the type's: "animated candlestick", "animated armour"; the material and the components are told by look mode (§11.1) |
| Per-material numbers | resistances by the material group of the drawn body (a wooden bucket burns) | by the body object's **base** material only (§6.5); a drawn variant changes colour, name in look, value, and weight, not resistances |
| Bestiary, first-kill XP, kill quests | four entries | per **family** (§4.4): 16 bodies + 3 tiers |
| Tuning | changes new creatures only (profiles were frozen) | changes every creature of the type, saved ones too, like any creature; changing a tier's range or the body list changes ids, a save-breaking content change like adding a creature file (§12) |

What is lost is per-creature naming and per-material resistances; what is gained is that
combat, AI, the scheduler, XP, the guideline tests, dumps, legends, and the dev `spawn` need no
change at all. Where a creature's numbers depend on its rolled items (Protection, hit points,
damage, resistances, speed), they already come from the equipment through `Mods::worn` and the
gear paths, so nothing about them has to be per-type or saved.

### 4.2 The scheme: the smallest that fits

| Tier | Types generated | Ids | Level | Count |
|---|---|---|---|---|
| Household | **one per body file** | `animated_<body>`: `animated_candlestick`, `animated_teapot` | the body's `level` (one number per body, §5.2) | 16 |
| Blades | **one per level** of the tier's range | `animated_blade_5` … `animated_blade_20`; family `animated_blade` | its own | 16 |
| Hollow armour | one per level | `animated_armour_12` … `_35`; family `animated_armour` | its own | 24 |
| Panoply | one per level | `animated_panoply_20` … `_70`; family `animated_panoply` | its own | 51 |

Why not fewer:

- **One type per tier** (with a level range) is the first draft's instance by another name: the
  level must live somewhere per creature, and every level read becomes an accessor again.
- **A household type per (tier, level)** with the body rolled per creature puts the body's
  name, glyph, attack verb, damage type, and speed per creature: accessors again. A household
  body is what the creature *is*, so it is the type. Household levels are 1–8 and the tier is
  almost harmless; one level per body loses nothing worth its cost.
- **Per (tier, body, level)** or per (tier, weapon, level) multiplies to hundreds of types for
  differences that the equipment already carries (a sword's damage, an armour's Protection).

Why not coarser bands for the composites (a type per five levels): stats, hit points, and dice
would step every five levels, and the roster's guideline test reads exact levels. Fifty-one
panoply types cost nothing at load (a loop) and are invisible to the player (one family).

### 4.3 Where generated types go, and the fingerprint

`CreatureId` is an index in table order and the fingerprint feeds the ids in that order
(`Content::fingerprint_for`). `fingerprint_for` cuts the creature list after the last type the
run's campaign may meet, so that a release build without the test world reads a developer
build's saves; a generated type has no campaign and counts as everyone's. So generated types are
numbered **after the shared `data/creatures/` files and before every campaign's**: tiers in
`data/animated.toml` order, household bodies in path order, levels ascending. Generation is a
pure function of the content (no draws), so the ids, the ids' strings, and the fingerprint are
the same on every load.

Consequences, said plainly in A1's PR ("saves change"): the first generated types shift every
campaign creature's id once; afterwards, adding a body, changing a tier's `levels`, or adding a
tier shifts ids as adding a creature file does today (`docs/dev/merging.md`: the integrator's
format bump). A generated id that collides with a creature file's is a load error.

Each generated type gets everything a creature file has (§5.3 lists the fields), plus
`animated: Some(AnimatedDef { tier, family, main })`, where `family` is the family's first
`CreatureId` (a household type is its own family) and `main` says which component it is shown
by (§11.1): `Body` for household, `Slot(main_hand)` for blades, `Slot(body)` for armour and
panoplies. Its corpse is `None` (`corpse = false`), so the generated corpse objects are unchanged.

### 4.4 The family: the four reads

A family is the run of generated types of one tier. Four places see it, each one line:

| Where | Read | Why |
|---|---|---|
| Danger draw (`content/danger.rs`) | a population line naming `animated_blade` becomes **one candidate** for the whole run (§10.3) | listing 16 levels as 16 candidates would weigh the tier 3–4 times a single type |
| First kill (`world/experience.rs::earn_kill`) | `discovered.kills` keyed by the family's id, the line by its plural: "You learn something of animated blades." once, not 16 times | |
| Kill quests and tag objectives (`world/quest.rs`) | a quest's `target = "animated_blade"` counts any member; a `tag:animated` objective lists families once ("animated candlesticks, ..., animated blades"), not 107 types | |
| Wiki and bestiary tests (`xtask/src/wiki/`, `world/bestiary_tests.rs`) | one row per family, the per-level numbers in the tier's own table | |

### 4.5 What each creature still carries, and where it is saved

| Per creature | Where | Saved today? |
|---|---|---|
| Its household body (a teapot of **brass**, its parts) | its inventory, first stack, in no slot | yes (`Actor::inventory`) |
| Its components (the sword, the suit, the rings) | inventory and `equipment` | yes |
| Its rank in the skill of what it wields (blades, panoplies), Sword and board (panoplies with a shield) | `Actor::weapon_skills`, set at spawn | yes |
| Its hit points with what its items add | `Actor::max_hp` through `Mods::worn` (§8), as for any creature with gear | derived, as today |

The body sits in the inventory, not in a slot: nothing can wear it, and nothing can be stolen
from an animated creature (§10.1), so it never leaves while the creature lives.

## 5. The data

### 5.1 `data/animated.toml`: the tiers and the rules

```toml
[drops]
intact = 50            # percent each component falls whole (spec §8 "Animated objects")
pieces = 50            # percent of a broken part's weight that falls as pieces of its material

[[tier]]
id = "household"
role = "small"                    # the roster's role (§6.2)
hp_factor = 0.5
dice_steps = -2                   # two steps under the roster's curve: almost harmless
bodies = true                     # one type per file in data/animated/ (§5.2)
rarity = "common"

[[tier]]
id = "blade"
name = "animated blade"           # the family's and every member's name
levels = [5, 20]                  # one generated type per level
role = "quick"
hp_factor = 0.6
glyph = "item"                    # one-item creature: its weapon's glyph (§11.1)
rarity = "uncommon"
slots = [{ slot = "main_hand", classes = ["melee_weapons"], chance = 100 }]
magic = { base = 0, per_level = 2, most = 40 }   # percent a component is at least magic
rare = { from = 15, per_level = 1, most = 5 }    # percent a component is rare

[[tier]]
id = "armour"
name = "animated armour"
levels = [12, 35]
role = "hollow"
hp_factor = 1.2
glyph = "A"                       # a composite
rarity = "uncommon"
slots = [
  { slot = "body", classes = ["body_armour"], chance = 100 },
  { slot = "head", classes = ["helmets"], chance = 70 },
  { slot = "off_hand", classes = ["shields"], chance = 40 },
  { slot = "hands", classes = ["gloves"], chance = 30 },
]
magic = { base = 10, per_level = 1, most = 40 }
rare = { from = 20, per_level = 1, most = 10 }

[[tier]]
id = "panoply"
name = "animated panoply"
levels = [20, 70]
role = "standard"
hp_factor = 1.2
glyph = "A"
rarity = "rare"
slots = [
  { slot = "body", classes = ["body_armour"], chance = 100 },
  { slot = "main_hand", classes = ["melee_weapons"], chance = 100 },
  { slot = "off_hand", classes = ["shields"], chance = 60 },   # only beside a one-handed weapon
  { slot = "head", classes = ["helmets"], chance = 75 },
  { slot = "hands", classes = ["gloves"], chance = 50, from = 30 },
  { slot = "feet", classes = ["boots"], chance = 50, from = 30 },
  { slot = "cloak", classes = ["cloaks"], chance = 50, from = 40 },
  { slot = "neck", classes = ["amulets"], chance = 30, from = 50 },
  { slot = "ring", classes = ["rings"], chance = 30, from = 50, count = 2 },
]
magic = { base = 20, per_level = 1, most = 80 }
rare = { from = 20, per_level = 0.5, most = 30 }
```

Slot ids are those of `data/slots.toml` (the names above are illustrative; the implementation
uses the real ids). `from` is the lowest level a slot line is tried at; a generated type keeps
only the lines its level reaches, so `animated_panoply_29` has no gloves line at all. The tier
levels are the owner's (§16 D7). The wiki prints this table whole (§14).

### 5.2 `data/animated/<id>.toml`: the household bodies

One file per body, named by its id, read in path order like creatures (`README.toml` holds the
notes). A body is **a real object** in `data/objects/` (new ones, class `household`, §16 D2) plus
how it moves and hits; each body becomes one generated type, `animated_<id>`:

```toml
[[body]]
id = "candlestick"
object = "candlestick"      # in data/objects/: name, glyph, weight, materials, value
level = 4                   # its type's level, within the household tier's 1-8
rarity = "common"           # its type's rarity (data/creature_rarities.toml)
speed = 900                 # its own pace (default 1000)
attack = { damage_type = "bludgeoning", name = "base", verb = "clubs" }
on_hit = []                 # as a creature's attack (rare: the rug's held)
moves = "rocks"             # the look mode's word for how it moves (optional)
dies = "topples and lies still"   # its death line's verb (default by material group, §6.5)
```

| Body | Object (materials) | Level | Speed | Attack (type, verb) | Note |
|---|---|---|---|---|---|
| broom | wood, plant (bristles) | 1 | 1200 | bludgeoning, "swats" | quick, the weakest |
| stool | wood | 1 | 900 | bludgeoning, "barges" | hops |
| ladle | iron, brass, wood | 2 | 1100 | bludgeoning, "raps" | |
| cutlery | iron, pewter, silver | 2 | 1300 | piercing, "jabs" | "a clatter of cutlery": a stack of 3–6 as its body, drops as cutlery; silver ones are worth it |
| book | paper, leather | 3 | 1000 | bludgeoning, "slams shut on" | a scriptorium's (later rooms) |
| candlestick | brass, iron, pewter, silver | 4 | 900 | bludgeoning, "clubs" | silver: a low-level prize |
| teapot | brass, pewter, silver (no clay material exists yet) | 4 | 900 | fire, "scalds" | trivial fire damage |
| kettle | iron, copper | 5 | 900 | fire, "scalds" | |
| bucket | wood, iron | 4 | 900 | bludgeoning, "batters" | |
| chair | wood, hardwood | 5 | 900 | bludgeoning, "batters" | |
| warming pan | brass, copper | 5 | 1000 | fire, "sears" | |
| bellows | wood, leather | 5 | 1000 | bludgeoning, "buffets" | |
| cooking pot | iron | 6 | 800 | bludgeoning, "bangs into" | the heaviest, slowest |
| rug | wool, silk | 7 | 1000 | bludgeoning, "wraps" | `on_hit = [{ status = "held", chance = 10, severity = 1 }]`: the rug of smothering, the household tier's one trick |
| lantern | iron, brass | 6 | 1000 | fire, "sears" | carries a dim light (`light = "candle"`): seen coming |
| coat stand | wood | 8 | 800 | bludgeoning, "lurches into" | |

Checks at load: a body's `object` must exist and be of a household-ish class (not a weapon,
armour, food, coin, or key); its `level` must lie in the household tier's range; its id must not
make a generated id that a creature file already uses. The first draft's `[default_body]` (any
object animated without a file) is dropped: with types generated per body, a body without a file
has no type. A later *Animate objects* spell (§10.4) adds files, or brings it back.

### 5.3 What a generated type holds

| Field | Household (`animated_<body>`) | Blades, armour, panoply (`animated_<tier>_<L>`) |
|---|---|---|
| `name`, `plural` | "animated " + the body object's name ("animated candlestick"), plural by `data/plurals.toml` | the tier's `name` ("animated blade") |
| `level`, `xp` | the body's `level`; `5L + 10` | L; `5L + 10` |
| `stats`, `base_hp` | §6.2 at L | §6.2 at L |
| `attack` | the body's (type, verb), dice by §6.2 with `dice_steps` | blades and panoplies none (they wield); hollow armour its gauntlets' slam, dice by §6.2 |
| `natural` | Protection by the body object's base material group (§6.5) | blades Defence `2 + L/10`; others none |
| `speed` | the body's | §6.2 at L |
| `resist` | by the body object's base material group (§6.5) | the role's (§6.2) |
| `glyph`, `fg` | the body object's glyph and colour (its base material's); perception shows the drawn material's colour (§11.1) | blades: the tier's `glyph = "item"`, which perception resolves to its weapon's glyph (`)`, §11.1); composites `A`; `fg` steel's, shown as the main component's colour |
| `weight` | the body object's | blades the default for its size; composites the default for medium (§16 D18: no burden) |
| `tags`, `faction`, `immune`, senses | §10.1 | §10.1 |
| `rarity` | the body's | the tier's |
| `gear`, `loot` | empty (the components are rolled by §7; the loot is the components) | empty |
| `animated` | `{ tier = household, family = itself, main = Body }` | `{ tier, family = the first of its run, main = Slot(..) }` |

## 6. The four tiers

### 6.1 Overview

| Tier | Types | Levels | Composition | How it fights | Rarity | Glyph |
|---|---|---|---|---|---|---|
| Household | `animated_<body>`, 16 | 1–8 (one each) | one body (a household object of a drawn material) | its body's natural attack, two dice steps under the curve: almost nothing | by body (mostly common) | the body object's (`(` for a candlestick) |
| Blades | `animated_blade_<L>`, family `animated_blade` | 5–20 | one melee weapon (sometimes magic, rarely rare) | **wields** it: the weapon's damage, threat, and riders, with Strength and a weapon-skill rank | uncommon | its weapon's (`)`) |
| Hollow armour | `animated_armour_<L>`, family `animated_armour` | 12–35 | body armour, a helm, sometimes a shield and gauntlets | its gauntlets' slam (the curve; a shield's Defence and the armour's Protection are real) | uncommon | `A` |
| Panoply | `animated_panoply_<L>`, family `animated_panoply` | 20–70 | armour, a weapon, a shield or helm, and as it rises gloves, boots, cloak, amulet, rings | **wields and wears** everything; rank in its weapon's skill and a style | rare | `A` |

The ranges overlap so that a floor of danger 15 can bring any of the three lower tiers.

### 6.2 Numbers by level (the roster's formulas, #383)

L is the type's level. The base values are the roster's mindless ones:

| Stat | Formula |
|---|---|
| St | 11 + 0.4L |
| To | 12 + 0.4L |
| Dx | 10 + 0.25L |
| Wp | 16 + 0.35L (a will that is not its own: hard to frighten) |
| In, Pr | 2 (mindless) |

Then the tier's role:

| Role (tier) | St | Dx | To | `base_hp` (2.5 × L × …) | Dice step | Speed | Natural armour | Resist |
|---|---|---|---|---|---|---|---|---|
| small (household) | −4 | +4 | −4 | ×0.5 | −2 | the body's | Protection by the body's base material group: metal 2, wood 1, else 0 | by the body's base material group (§6.5) |
| quick (blades) | ±0 | +5 | −2 | ×0.6 | — (its weapon) | 1200 + 5L, at most 1600 | Defence 2 + L/10 ("its thin edge") | piercing 50, bludgeoning −25 |
| hollow (armour) | +5 | −2 | +5 | ×1.2 | ±0 | 900 + 3L, at most 1100 | none (it wears its Protection) | piercing 25 (blows pass through the hollow) |
| standard (panoply) | ±0 | ±0 | +2 | ×1.2 | — (its weapon) | 1000, +4 a level past 20, at most 1200 | none | none beyond its items' |

- **Dice** (household and hollow): the roster's target `2 + 0.55L` to L20, then
  `13 + 0.25 × (L − 20)`, nearest of 1d3 … 5d12, moved by the step. Household L1 is 1d2, L8 about
  1d6; hollow armour at L20 about 3d8 (+ Strength).
- **Hit points**: `To + 2.5 × L × factor`. Household L1 has about 9, L8 about 19; a panoply at
  L40 about 150, at L70 about 250, before what its items add (through `Mods::worn`, §8).
- **Weapon skills**: blades and panoplies hold `1 + L/10` ranks in the skill of what they wield
  (max 8); a panoply with a shield holds `L/15` in Sword and board. The skill depends on the
  rolled weapon, so it is set at spawn in `Actor::weapon_skills` (§7), not in the type.
- **XP**: `5L + 10`, written into each type, so the guideline test
  (`creatures_follow_the_guideline`) covers every generated type with no change.
- **Speed and attack cost** are not changed by weight: the numbers are simple on purpose. Every
  number is rounded as the roster rounds.

### 6.3 Blades

The owner's "tools and weapons". A sword, dagger, axe, spear, mace, or staff dances alone. The
component is a **melee** weapon only: flying missiles and bows (a bow with no archer has nothing
to draw it; a dancing javelin that is thrown and returns) are **deferred** (§16 D10, a line on
#196). It is drawn among the base items of class `melee_weapons` with `level ≤ L` by their
`frequency`, as loot draws a base item, then made by `loot::make_as` at grade plain, magic, or
rare by the tier's odds. A blade is quick and fragile: easy to kill, hard to hit, and its weapon
is the prize.

### 6.4 Hollow armour and the panoply

Hollow armour is a suit with nobody inside. It fights with its gauntlets (bludgeoning, the
curve) and is slow, hard to hurt, and loud (footsteps 60). The panoply is the owner's composite:
the full kit of a knight with no knight, and it wields. Two-handed weapons leave the off hand
empty (the shield line is skipped). From level 50 it may wear **rings and an amulet** (§16 D16):
they show in look mode as any worn item does, and **all** their magic applies to it (§8).

### 6.5 Materials

The **base** material of a household body's object (the material its object file names first)
gives its type a little more, at load. A drawn variant (a silver candlestick where the file says
brass) changes the creature's colour, the material word in look mode, its drop's value and
weight, never its type's numbers (§4.1, the trade-off).

| Material group | Resist | Natural Protection (household) | Death line (default) |
|---|---|---|---|
| metal | electricity −25 | 2 | "clatters to the floor" |
| wood | fire −50 | 1 | "splinters" |
| cloth (wool, silk) | fire −50, slashing −25 | 0 | "falls in a heap" |
| hide (leather) | fire −25 | 0 | "falls in a heap" |
| glass, stone, bone | bludgeoning −25 | 1 | "shatters" |

For blades, hollow armour, and panoplies the material of what they wear and wield already counts
where it matters (a bronze sword's damage, an iron plate's Protection: `variant.rs`), and their
death line is by their tier ("falls in pieces", "collapses with a clang").

## 7. Composition: how a creature is made

`animated::compose(content, rng, def) -> (Vec<Stack>, Equipment, WeaponSkills)`, called by
`travel.rs::make_creature` for a type with `animated`, in this draw order:

1. **Level**: the type's. No draw.
2. **Body** (household): its object, its material by `variant::draw` with `Source::Gear(L)` (a
   silver teapot is rare, as silver is rare in creatures' gear); a cutlery body draws its count
   (3–6). It becomes the first inventory stack, in no slot.
3. **Slots** (the other tiers), each line the type keeps, in order: one draw for its `chance`
   (skipped for a shield beside a two-handed weapon); one for its base item among the line's
   classes with `level ≤ L`, by `frequency`; one for the grade (rare by `rare`, else magic by
   `magic`, else plain); then `loot::make_as(content, rng, object, 1, (L, grade), Source::Gear(L))`.
   Each piece is put in the pack and equipped. **No burden check**: it carries itself (§16 D18).
4. **Weapon skills**: the rank in the wielded weapon's skill, and Sword and board beside a
   shield (§6.2). No draws.

Everything else (stats, `hp_base`, attack, speed, resist) comes from the generated type as for
any creature; `max_hp` is then set after equipping, as `gear::roll` already does.

The stream: on a generated level, **`animated/<level id>`**, drawn in spawn order after the
level's gear (`gear/<level id>`), so the rest of the level is the same whatever animated things
it holds; for every other spawn (a level's legend, a runtime spawn, the dev `spawn`),
**`animated/<actor id>`**, as gear's `gear/<actor id>`. Neither touches the gameplay stream.

## 8. Inherited powers

What an animated creature wears and wields works for it exactly as for a goblin with a magic
sword or the player: `Mods::worn` is the one aggregation, and every rule reads it. The owner's
word for rings and amulets (§16 D16): "add their full effects to the monster (they are
essentially wearing them)." Nothing new is built for the effects that work; this table, checked
against the read sites of `content/affix.rs::Effect` on 2daff477, is what the wiki says it
inherits, and the gaps are follow-ups.

| Effect (`data/affixes/`) | Honoured for creatures today? | Through |
|---|---|---|
| Stat raises (`Stat`: of the ox, of the lynx, ...) | yes | `Actor::effective_stats`: hit points, to hit, damage, Defence |
| To hit, damage, extra damage, threat, crit, parry | yes | the attack profile (`Mods::local` and `Mods::worn`) |
| Elemental riders (Flaming, Shocking, ...) | yes | the attack profile's riders: a flaming blade burns |
| Attack speed, move speed | yes | `world/schedule.rs::cost_of`, for every actor |
| Armour, Shield, MaxDex, Dodge, Deflection, Natural | yes | `Actor`'s Defence |
| Protection | yes | `Stack::protection` → `Actor::protection` |
| Resistances | yes | `Mods::resistances` → `Actor::resistances` |
| Max hit points | yes | `Actor::max_hp` (set at spawn after equipping, as for gear) |
| Regeneration | yes | `World::healing_rate`, for every actor |
| Stealth (quieter steps) | yes | `noise.rs`: the step's loudness, for every actor |
| Light (it glows) | yes | `light/cache.rs`: any actor with a glowing worn item carries the light: a glowing panoply is seen coming |
| Skill affixes | yes, where a rule reads a creature's skill | `Mods::skills` |
| Weight | yes (its items weigh less), unused (no burden, §16 D18) | `Stack::weight` |
| **Max mana, mana regeneration, caster level** | **gap**: held, nothing reads them for a creature that does not cast | follow-up with creature magic (#397 and the creature-magic design) |
| **Carry** | **gap**, harmless: constructs have no burden | none needed |
| **Range, Draw, Durability** (missile weapons) | **gap**, harmless here: blades and panoplies wield melee weapons only | with the deferred flying missiles (§16 D10) |
| Unique items' powers (`Power::Inflict`, `Vision`, `Ward`) | yes, once built | never rolled by loot, so a panoply never holds one |

The follow-ups to file when this design merges: one line on the creature-magic issue ("animated
panoplies hold mana and caster-level affixes: honour them when creatures cast"), and the
missile line rides the deferred #196 entry.

**Danger**: a panoply at level 60 with two rare pieces of item level 60 is among the most
dangerous things in the game, and the most rewarding (§9.3). That is the owner's intent; the
wiki says so plainly.

## 9. Drops: intact or broken into their parts

### 9.1 The rule

When an animated creature dies, for each component in order (the body first, then its equipment
in slot order): one draw on its existing **`drop/<actor id>`** stream against `intact` (50, data):

| Draw | What falls |
|---|---|
| intact | the component as it was: material, gems, magic (unidentified), all of it |
| broken | `object::break_up(stack)`: **the parts it is made of** (§9.2); its magic gone; a message only if the player sees it ("The long sword shatters.") |

The type's `loot` chance is 0 (the components are the loot); Scavenger (identify what falls) and
Treasure hunter (luck) apply as they do to any kill. Salvager (§13) raises `intact`.

### 9.2 Breaking into parts: the existing parts, one generic piece

The owner (§16 D4): "scrap, but you do not need to invent scrap objects. tin ruby pommeled
dagger -> ruby & tin". So a broken component yields **what the objects system already says it is
made of**: `Stack::parts(content)`, the base object's parts with the stack's own in their place
(variants and gem ornaments, `item_parts::resolve`).

| A part of… | Falls as |
|---|---|
| a gem material (a ruby pommel, a ring's sapphire) | **a loose gem**: the existing `gem` object carrying that gem with the part's size and cut (`Stack::gem`), worth what a loose gem is worth |
| a material with a group (metal, wood, cloth, hide, glass, stone, bone, resin) | **pieces of that material** (below): a tin dagger gives pieces of tin |
| a material without a group (paper, wax, plant, flesh) | nothing: burnt, torn, or crushed beyond use |

Nothing in the game today is raw material: no ingot, nugget, or lump. The **single smallest
addition** is one object, not one per material and not hand-made scrap:

```toml
[[object]]
id = "piece"            # a piece of some material (spec §7 "Objects and materials")
name = "piece"          # named by its material: "a piece of tin", "3 pieces of silk"
glyph = ","
fg = "#a0a0a0"          # shown in its material's colour (Stack::fg), as a variant is
class = "miscellaneous"
materials = ["steel"]   # only what it would be made of without its own, which no piece is
made_of = ["any"]       # every material with a group (content/material.rs::ANY)
weight = 100            # grams of steel; a piece of another material weighs by its `weight` %
value = 1               # times the material's `value` %: a piece of gold is worth real coin
stackable = true
```

It is the loose gem's pattern applied to materials: one object that is made of whatever its own
main part says (`Stack::with_parts`, `ItemPart::material`), named by that material
(`piece::names`, beside `gem::names`: "piece of tin", "pieces of tin"), coloured, weighed, and
priced by it through the existing `Stack::fg`, `variant` weight, and `value_with`. Pieces of one
material stack (`Stack::same_goods`). No new item class: shops that buy miscellaneous goods buy
pieces. Being data in one file, it needs no generation from `data/materials/`.

`break_up(stack)`: for each resolved part, in part order: a gem part gives its loose gem; a
grouped material's part gives `max(1, round(item weight × share × pieces % / 100 g of that
material))` pieces, parts of one material summed into one stack; the rest nothing. A broken tin
dagger with a ruby pommel (450 g, all tin but the share-0 pommel) gives **a ruby** (its size and
cut) and **2 pieces of tin**; a broken steel long sword (1.8 kg) 9 pieces of steel; a broken
silver candlestick a few pieces of silver worth real coin. A gem-set piece keeps much of its
worth broken. `break_up` is reusable: sundering and fire can call it later.

### 9.3 Loot-goblin arithmetic (the wiki's table)

| Creature | Components (expected) | Intact (50 %) | Magic among them |
|---|---|---|---|
| household L4 | 1 body | 0.5 | none |
| blade L15 | 1 weapon | 0.5 | ~30 % magic, ~1 % rare |
| hollow armour L25 | 2.4 | 1.2 | ~35 % magic, ~5 % rare |
| panoply L40 | 6.1 | 3.0 | ~60 % magic, ~10 % rare |
| panoply L70 | 8.0 | 4.0 | 80 % magic, ~25 % rare |

Plus the broken half's pieces and loose gems. Compare: a goblin chief's `loot` is one roll at
some percent.

## 10. Tags, faction, immunities, and where they appear

### 10.1 Identity

| Field | Value (every generated type) |
|---|---|
| `tags` | `["animated", "construct"]` (the README's group list gains `animated`; `construct` as a role for things made, not born: the mud golem gains it too, §16 D14) |
| `faction` | new **`animated`**: "Things a will set moving and left: they set on the living who come near, and leave the rest be." Hostile to `player` only (§16 D9) |
| `alignment` | neutral (killing one is no deed) |
| `vision`, `sight`, `hearing` | darkvision; sight 8; hearing 10 (they wake to noise like anything) |
| `footsteps` | household 20, blades 0 (it floats), armour and panoply 60 |
| `resist` | the role's and the body's (§6.2, §6.5); poison comes under `immune` |
| `immune` | **new field**: `["diseased", "poisoned", "bleeding"]` (§16 D17) |
| `corpse`, `blood` | false, "none" |
| `opens_doors` | armour and panoply true (gauntlets); others false |
| `incorporeal`, undead | never |
| `size` | household and blades small, the rest medium |
| stealing | **refused**, no time: "It is all of a piece." (§16 D13) |

**Immunities** (§16 D17, the owner: "also poison and bleed as appropriate"). The statuses that
make no sense for an object are exactly three of `data/status.toml`'s:

| Status | Why not for a thing | Where it is refused |
|---|---|---|
| Diseased (every disease) | nothing lives in it to sicken | `World::expose`, beside the talent's `disease_immune`: an immune actor is never infected, nothing rolled |
| Poisoned | no blood or breath to carry it | `World::afflict`, the one way to give a status |
| Bleeding | no blood (a bloodless type takes the status today) | `World::afflict` |

`immune` is a creature field, a list of status ids (checked at load; `diseased` means every
disease), read in those two places: one rule, reusable by skeletons, golems, and wraiths, which
the undead roster may adopt in its own PR. The others stay as they are: Frightened rarely takes
(Willpower 16 + 0.35L), Held is the rug's own trick and holds them too, Staggered and
Transfixed are a blow's and a gaze's, fair against a hollow suit. Fire, cold, and electricity
are damage types, taken by resistance (§6.5).

### 10.2 Populations

| Branch | Adds | Why |
|---|---|---|
| Cealdhlāw (barrows, danger 1+) | the household types, `animated_blade` | grave goods that will not lie still |
| Fennmynster (drowned minster, danger 4+) | the household types (candlestick, book, lantern, ...) | candlesticks, books, censers of the drowned church |
| Fyrdelf (the delf, danger 8+) | `animated_blade`, `animated_armour` | the duergar smiths' work, waking |
| Deeper branches (later) | `animated_panoply` | whatever reaches danger 20 |

A population line names household types one by one (they are ordinary types, each with its own
level and rarity) and the other tiers by family. The generic stray list is derived from
populations, so all three tiers so placed also stray (5 %) into other branches; out-of-depth
draws can bring hollow armour into Fyrdelf's top floors. A panoply appears nowhere until a
branch lists it (or the dev `spawn` brings one).

### 10.3 Families in the danger draw

A population line naming a family (`animated_blade`) gives `Content::candidates` **one**
candidate for the whole run: `danger::Candidate` gains its range, `(first id, (from, to),
factor)`, an ordinary type being `(id, (level, level), factor)`. `pick` weighs it as
`weight(clamp(at, from, to), factor, at)` and, when it is drawn, returns the member of level
`clamp(at, from, to)`: `CreatureId(first + clamp − from)`, since a family's members are numbered
in a row (§4.3). At danger 15 the blade family weighs as a level-15 creature and gives
`animated_blade_15`; at danger 3 it weighs nothing (level 5 is above the danger); at danger 30 it
weighs as level 20 (fading out by the same curve that fades kobolds out). Out-of-depth draws
clamp the deeper danger the same way. One rule in `content/danger.rs`, pure, no `Content`
needed; household types need none of it.

### 10.4 Hand-placed and later rooms

A level legend places one by its generated id, as any creature: `creature = "animated_teapot"`,
`creature = "animated_blade_12"` (§16 D12: the id carries the level and the body), with an
optional `animated = { material = "silver" }` for a household body's material (else drawn), so a
hand-made kitchen can hold its own silver. Fixed components wait for a hand-made level that
needs them. Later, the special-rooms design's themed dens and two new rooms, an **armoury**
(hollow armour and blades among racks) and a **workshop** (household things), are natural homes;
they are follow-ups, not this design.

**Magic later** (#397, the magic design): an *Animate objects* spell would pick the generated
type for an object on the floor (its body file) and pass that stack as the body (the caster's
faction). Noted only.

## 11. What the player sees

### 11.1 Names, glyphs, colours

Rules and messages use the **type's** name. Perception adds what the creature is made of from
its **main component** (`AnimatedDef::main`: the body, the wielded weapon, the body armour):
`SeenActor::shape: Option<Shape>` (`Copy`: glyph, colour), filled in one place, and look mode's
material word. That is the only per-creature read, and it reads the actor's own, already-known
items (others see what a creature wears and wields).

| Tier | Name (the type's) | Look mode (the material word, then the existing gear line) | Glyph (§16 D5) | Colour (§16 D6) |
|---|---|---|---|---|
| household | "animated candlestick" | "an animated candlestick of brass (hostile), rocking" | the body object's own (`(`) | the body's material colour (`Stack::fg`) |
| blades | "animated blade" | "an animated blade (hostile), wielding a bronze short sword" | its weapon's own (`)`) | the weapon's material colour |
| armour | "animated armour" | "an animated armour (hostile), wearing a suit of chain mail, an iron helm, a small shield" | `A` | the body armour's material colour |
| panoply | "animated panoply" | "an animated panoply (hostile), wielding a long sword, wearing a suit of plate armour, a great helm, a tower shield, a silver ring" | `A` | the body armour's material colour, **one colour until A6** |

- **Glyphs**: a one-item creature shows its item's glyph (the owner: "for 1 item monsters, the
  item glyph"); a composite shows `A` ("for composites, A"). Perception takes the item glyph
  from the main component, so a later weapon with its own glyph follows with no change. A pile
  and a creature may look alike: look mode and the creature layer's colour tell them apart.
- **Colours**: by material (the owner: "color per material"). A composite has many materials;
  it shows its body armour's until **A6, chromatic composites** (§15), a renderer treatment that
  shows several material colours on one cell. The owner: "for composites, please create chromatic
  color thing (custom shader?) ... too big to include."
- **Names**: the components' **unidentified** names (`Stack::names`, never `describe_truth`), so
  a rare blade wields "a steel long sword" until it falls and is identified. The type's name is
  what messages ("The animated blade hits you."), quests, the first-kill line ("You learn
  something of animated blades."), and the wiki use. The first draft's per-creature names
  ("animated suit of plate armour", §16 D15) needed name reads in a dozen rules; with ordinary
  types the gear line tells the same in look mode.

### 11.2 Vagueness

The game never shows an animated creature's level, its components' magic, or its numbers;
Unbinder and Appraiser's eye say their effects in words. The wiki gives the formulas, the tier
table, the bodies table, the composition odds, the inheritance table, and the drop rule exactly
(§14).

### 11.3 Messages

| When | Line |
|---|---|
| attack | "The animated teapot scalds you." (the type's verb), "The animated blade hits you." (the weapon's verb, as for a wielded weapon) |
| death | "The animated chair splinters." (the body's `dies`, else its material group's, §6.5; the tier's for the others) |
| a component breaks, seen | "The long sword shatters." (one line per broken component the player sees fall; quiet otherwise) |
| a component falls intact | nothing: the pile shows it |
| stealing | "It is all of a piece." (refused, no time) |

### 11.4 Dumps

`actors` (dev): nothing new; the generated id (`animated_blade_14`) and the inventory it already
lists (magic as it truly is) say everything. JSON per seen actor: the shape's glyph and colour
besides the creature id. `progress` (dev): nothing new.

## 12. Saves, determinism, performance

| Concern | Design |
|---|---|
| Save | **no new state**: components in `Actor::inventory`/`equipment`, ranks in `Actor::weapon_skills`, all saved and hashed today. Pieces and loose gems are ordinary stacks with own parts, saved by id |
| Fingerprint | the generated ids feed the creature group, numbered after the shared files and before every campaign's (§4.3); A1 shifts campaign creature ids once; later tiers, ranges, and bodies shift them as a new creature file does. Saves change: say so in each PR, leave `FORMAT_VERSION` to the integrator |
| Generation | a pure function of the loaded content: tiers in file order, bodies in path order, levels ascending; no draws, no `HashMap` iteration |
| Hash | nothing new: what an animated creature carries is part of the actor |
| Streams | `animated/<level id>` (generated levels, spawn order, after `gear/<level id>`), `animated/<actor id>` (other spawns), `drop/<actor id>` (drops, existing). No gameplay draws |
| Generated levels change | adding a type to a branch's population changes that branch's floors: their insta snapshots and any replay through them are regenerated in the PR that adds the population line (A1 for Cealdhlāw and Fennmynster). `docs/dev/merging.md`'s generated-files rule applies |
| Performance | generation is a loop at load over ~107 types; per turn an animated creature costs what a creature with gear costs. A panoply's `Mods::worn` over up to ten stacks is the open risk: A4 adds a `bench-turns` row, a fixture cave floor at danger 40 with panoplies at full population (16 ms per turn, as `delf`); a cache of `Mods::worn` per actor is added only if that row fails |

**Tests** (each slice adds its own; collected here):

| Test | Where |
|---|---|
| generation: the ids, their order (after the shared files, before every campaign's), the numbers by §6.2 for every tier at its first, middle, and last level; the same on every load; a colliding id refused | `content/animated_tests.rs` |
| `creatures_follow_the_guideline` covers the generated types (no change: they are ordinary) | `content/experience.rs` |
| `fingerprint_for` with and without the test world still agree for a real campaign | `content` tests |
| the family candidate weighs by the clamped level and returns that member, out-of-depth too | `content/danger.rs` tests |
| first kill, kill quests, and the tag objective count a family once | `world/experience` and `world/tag_tests.rs` |
| `compose` is deterministic per stream; the body's material is drawn; slot lines follow `from`, `count`, and the two-handed rule; the weapon rank is set | `world/animated_tests.rs` |
| a panoply with a Flaming long sword burns; with "of the ox" hits harder; with a resistance affix takes less; a glowing ring lights it | same, via `fight` |
| immunities: an animated creature is never poisoned, bleeding, or diseased; others are as before | same |
| drops: with a fixed `drop/` stream, intact components fall whole (magic kept, unidentified); broken ones as pieces of their materials and a loose gem; nothing of paper; Salvager raises the share | same |
| `break_up` counts, materials, gems, and the no-group case; `piece` names, weight, value, stacking | `object` tests |
| stealing from one is refused, no time | `world/steal_tests.rs` |
| save round trip with an animated creature; a replay with one (`replay --create`/`--verify`) | `save` tests, CLI |
| look mode names, the material word, and the gear line (household, blade, armour, panoply): text snapshot | `crates/warlock/src/ui/screen_snapshots.rs` |
| no golden screenshot: a glyph and a colour are seen by dumps and text snapshots (A6's chromatic look is its own question, §15) | |

## 13. Talents and class abilities

**The system**: `animated` ("Animated objects: what they are made of and what they leave"),
placed in `data/talents.toml` beside `creature_gear`. Two talents meet `min_talents = 2` in A1;
the third comes with panoplies:

| Talent | Tree | Ranks | Effect (in game words) | The one read | Slice |
|---|---|---|---|---|---|
| **Salvager** | delver | 2 | "You know how a bound thing comes apart: more of what an animated thing is made of survives its end." `intact` +15 a rank (50 → 65 → 80); the player's, whoever made the kill (as Treasure hunter) | the drop rule (§9.1) | A1 |
| **Unbinder** | weaponcraft | 1 | "You strike at the binding, not the thing: your blows against animated things are surer and harder." +1 to hit, +2 damage against a creature tagged `animated` | the attack profile, beside Favoured prey | A1 |
| **Appraiser's eye** | scholar | 1 | "You can tell a bound thing's worth at a glance." Look mode adds "it hums with power" when any of its components is magic | perception, the one place look's creature line is built | A4 (§16 D14) |

**Class abilities**: none (`min_class = 0`). No class is about constructs today; a priest's or
paladin's "unbind" belongs to the magic design (#397), as a spell. Salvager and Unbinder also
carry `creature_gear` and `combat` respectively in their `systems` lists, so the trees' coverage
stays even.

## 14. Spec, wiki, and docs

| File | Change |
|---|---|
| `spec/08b-creatures.md` | a section "Animated objects": tiers, generated types and families, composition, inherited powers, drops, immunities; the faction |
| `spec/07b-data.md` | `data/animated.toml`, `data/animated/`, generated creature types, the creature field `immune`, the legend's `animated` |
| `spec/07c-generation.md` "Danger levels" | family candidates in the draw |
| `spec/03b-campaign.md` | the populations (Cealdhlāw, Fennmynster, Fyrdelf) |
| `spec/07*` objects | household objects, the `piece` object, `break_up` |
| `docs/dev/animated.md` (new topic) | generation (ids, order, the fingerprint), families and their four reads, `compose`, presentation, drops; it earns `content/animated.rs` and `world/animated.rs` |
| `docs/dev/content.md` | generated creature types beside generated corpses: where they are numbered |
| `docs/dev/objects.md` | `piece`, `break_up` |
| `docs/dev/status-effects.md` | the `immune` field |
| `assets/data/creatures/README.toml` | the `immune` field, the `animated` and `construct` tags, a note that `data/animated/` generates types |
| Wiki | a page **"Animated objects"** under Creatures: what they are, how they are made, the drop rule, and generated tables (`xtask/src/wiki/animated.rs`): the tiers (levels, role, glyph, slots and odds), the formulas by level (St … XP, dice at 1, 10, 20, 40, 70), the bodies (object, materials, level, speed, attack), the inheritance table with its gaps, the parts a broken component gives, the loot arithmetic; registered in `generate()` beside `creatures`. The creatures tables show families once, not 107 rows. The objects page lists household goods and pieces |

## 15. Implementation plan

Each issue lands rules, then UI, then content, with its spec section, `docs/dev/` topic, and wiki
in the same PR. "Ids" marks a change to creature or object ids (saves change); "state" save state;
"action" an action (the overlap rule: about three at once).

| # | Issue | Contents | Needs | Ids / state / action |
|---|---|---|---|---|
| **A1** | **Household tier** (the first slice) | `content/animated.rs`: `data/animated.toml` (the household tier, `[drops]`), `data/animated/` with the bodies of §5.2, generation of one type per body at load (ids after the shared files, `AnimatedDef`, checks); household objects in `data/objects/` (class `household`); `animated::compose` for the body, its streams; `SeenActor::shape` (the body's glyph and material colour) and look's material word; the creature field `immune` (§10.1); stealing refused; the `animated` faction, the tags; drops **intact or nothing** (the broken half falls as nothing until A2); the legend's `animated = { material }`; Cealdhlāw and Fennmynster populations (their snapshots regenerated); the system `animated` with Salvager and Unbinder; look text snapshot; wiki page with the tier, formula, and body tables | — | ids |
| A2 | **Breaking into parts** | the `piece` object and `piece::names`, `object::break_up` (loose gems, pieces by weight, the no-group case), the broken half of the drop rule, the break line; wiki parts table | A1 | ids (one object) |
| A3 | **Blades** | per-level generation for a tier with `levels`, **families** and their four reads (§4.4, §10.3), the `blade` tier, slot lines and grade odds (`loot` base draw by class, a small `loot::base_in`), weapon-skill ranks at spawn, the item glyph from the main component; `animated_blade` in Cealdhlāw and Fyrdelf | A1 | ids |
| A4 | **Hollow armour and panoplies** | the `armour` and `panoply` tiers, the slot lines with `from` and `count` (rings and the amulet from 50), the two-handed rule, Sword and board ranks, `A` glyphs; `animated_armour` in Fyrdelf; `animated_panoply` in no population yet; Appraiser's eye; the `bench-turns` row; the inheritance tests (§8) and the creature-magic follow-up line | A3 | ids |
| A5 | **Homes** (later, optional) | armoury and workshop rooms on the special-rooms system; a deeper branch listing panoplies | A4, special rooms R1 | — |
| A6 | **Chromatic composites** (rendering) | a composite shows the colours of its several materials on its one cell instead of its body armour's alone (below) | A1 (the owner); shows only once A4 brings composites | — |

Order: A1, then A2 and A3 in parallel, then A4; A6 any time after A1 (it can be built and
tested on a dev-spawned panoply once A4 lands, or on a fixture before). No slice adds save state
or an action (the dev `spawn` and the legend cover tests and captures).

**A6, chromatic composites** (owner, §16 D6: "for composites, please create chromatic color thing
(custom shader?). add this as issue that depends on the first issue. too big to include."). A
sketch, for its own issue and `docs/dev/rendering.md`:

- **Perception** gives a composite's `Shape` up to four colours: the material colours
  (`Stack::fg`) of its components in slot order, distinct ones only (`Shape { glyph, colours:
  [Rgb; 4], count }`, still `Copy`). One-item creatures keep one colour.
- **The map grid** (`render/console.wesl`): a cell with more than one colour gets a palette entry
  in the per-cell animation texture (the table entry the `cycle` animation already uses, or a
  new `chroma` entry in `render/anim.rs`'s uniform table), and the shader colours the glyph's
  covered pixels by the glyph-local position (horizontal bands, plate over mail over leather) and
  a slow drift between them, as `shimmer` drifts. **Reduced effects**: the bands stand still.
- **A sliding creature** is drawn on the free-glyph layer for its slide; the slide uses the first
  colour (or the band treatment ported to `free_glyph.wesl`; the issue decides).
- **Subtle** (the owner's rule for effects): the bands are the materials' own colours, at the
  glyph's normal brightness, never a rainbow.
- **Verification**: a `--capture` looked at in the PR; a golden only if the issue argues that no
  text snapshot or dump can see it (it is GPU-only, so it may qualify: one entry, the panoply in
  the dark and in light).

## 16. Decisions (owner, 2026-10-26)

The first draft's twenty questions, answered on the PR. Where the owner accepted the
recommendation it is marked so; the sections changed are those this revision rewrote.

| # | Question | Decision | Sections changed |
|---|---|---|---|
| D1 | Representation: a type per tier with an instance, or generated types at load? | **Generated types at load** ("generated types at load i guess. less to maintain, interacts with game logic well."): one type per household body, one per (tier, level) for the other tiers in families; components in inventory and equipment, no new save state; trade-off in §4.1 | §1, §3, §4 (rewritten), §5.1–5.3, §6.1–6.2, §6.5, §7, §10.2–10.4, §11, §12, §14, §15 |
| D2 | Household bodies as real objects, class `household`? | **Yes** (recommendation) | — |
| D3 | `intact = 50` for every component, Salvager raising it? | **Yes** (recommendation) | — |
| D4 | Broken components: scrap objects, or nothing? | **Their own parts** ("scrap, but you do not need to invent scrap objects. tin ruby pommeled dagger -> ruby & tin"): gems fall loose, every grouped material as pieces of it, one generic `piece` object; the five scrap objects and the Materials class are gone | §1, §3, §9.1–9.3, §12, §14, §15 (A2) |
| D5 | Glyphs: letters, or the item's own? | **Composites `A`; one-item creatures their item's glyph** ("for composites, A. for 1 item monsters, the item glyph."): household bodies and blades show their item's | §1, §5.1, §5.3, §6.1, §11.1 |
| D6 | Colour by material, or a magic tell? | **Colour per material; composites get a chromatic look later** ("color per material. for composites, please create chromatic color thing (custom shader?). add this as issue that depends on the first issue. too big to include."): slice A6, its own issue; one colour until then | §1, §11.1, §15 (A6) |
| D7 | Tier level ranges 1–8, 5–20, 12–35, 20–70? | **Yes** | §5.1 (unchanged numbers) |
| D8 | Composition strength (item level = L, the magic and rare odds)? | **As listed** (recommendation); tune after A4 in data | — |
| D9 | Their own faction, hostile only to the player? | **Yes** (recommendation) | — |
| D10 | Flying missiles (an animated bow, a returning javelin)? | **Deferred** ("add this as deferred"): a line on #196 | §1, §6.3, §8 |
| D11 | Mimics in this design? | **Deferred**: a line on #196 | §1, §2 |
| D12 | Legend placement with level and body? | **Yes** (recommendation), simplified by D1: the generated id carries level and body; `animated = { material }` optional | §10.4 |
| D13 | Stealing from one? | **Refused** ("It is all of a piece."), no time | §1, §4.5, §10.1, §11.3, §15 (A1) |
| D14 | Tags `animated` and `construct`; the mud golem `construct`; Appraiser's eye? | **Yes to all** (recommendation); Appraiser's eye in A4 | §13, §15 |
| D15 | The panoply's name: by its armour, or a tier word? | Recommendation was "by its armour"; **revised by D1**: messages use the type's name ("animated panoply"), look mode's gear line names the armour. Per-creature names would bring back the name reads D1 removes | §11.1 |
| D16 | Rings and an amulet on panoplies from L50? | **Yes, with their full effects** ("yes. add their full effects to the monster (they are essentially wearing them)"): every affix effect any actor honours applies; gaps (mana, caster level; carry; missile-only effects) listed with their follow-ups | §6.4, §8 |
| D17 | Disease immunity for constructs? | **Yes, and poison and bleeding** ("yes. also poison and bleed as appropriate"): a creature field `immune = ["diseased", "poisoned", "bleeding"]`, refused in `World::expose` and `World::afflict` | §1, §3, §10.1, §12, §14, §15 (A1) |
| D18 | No burden check on components? | **Yes** (recommendation) | — |
| D19 | XP by the guideline only? | **Yes** (recommendation) | — |
| D20 | Household riders only the rug's Held and the teapots' fire? | **Yes** (recommendation) | — |
