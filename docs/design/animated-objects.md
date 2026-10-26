# Warlock — Animated objects: household junk, dancing blades, hollow armour, and panoplies that wear their loot

Design document for the owner's review (2026-10-26). Written against `origin/main` at 2daff477
(after #443, creature tags; #438, spell tiers; #442, undead batch 2). Nothing here is implemented;
§15 cuts it into issues and names the first slice. §16 lists the open questions, each with a
recommendation.

## Contents

1. Summary
2. Research, tied to the decisions
3. What exists, and what this builds on
4. Representation: a type per tier, a composition per creature
5. The data
6. The four tiers
7. Composition: how a creature is made
8. Inherited powers
9. Drops: intact or broken
10. Tags, faction, and where they appear
11. What the player sees
12. Saves, determinism, performance
13. Talents and class abilities
14. Spec, wiki, and docs
15. Implementation plan
16. Open questions

## 1. Summary

**What the owner asked for**: an animated-objects group of creatures, household objects and
weapons at first, composites at higher levels (armour + shield + weapon). They are a little
special: the higher ones are made of **real magic items**, rolled at their level, whose powers
they **inherit**. Each component drops with 50 %; the rest break into **materials** of their
parts. Very procedural; the low levels are harmless junk; the high ones may be very dangerous.
They are loot goblins: worth killing.

| Piece | Design |
|---|---|
| Representation | **four creature types**, one per tier, each with a **level range** (`animated = { tier, levels = [1, 8] }`); each creature gets an **instance**: its level (drawn from the floor's danger, clamped to the range), its **body** (a household object, as a stack), and a **profile** derived once, at birth, from body, level, and components (name, glyph, colour, speed, natural attack and armour, resistances, weight). Saved as `Actor::animated: Option<Box<Animated>>`. The worn and wielded components are the actor's ordinary inventory and equipment, already saved. §4 |
| Data | `data/animated.toml` (tiers, formulas' constants, composition odds, drops, scrap); `data/animated/<body>.toml`, **one file per household body** (broom, stool, candlestick, teapot, ...), each naming a **real object** in `data/objects/` it is made of; a `[default_body]` for any object without a file. §5 |
| Tiers | **Household** (levels 1–8: a scuttling broom, a hopping stool; trivial damage), **Blades** (5–20: one real weapon, sometimes magic), **Hollow armour** (12–35: a suit, helm, sometimes a shield; fights with its gauntlets), **Panoply** (20–70: armour, weapon, shield or helm, then gloves, boots, a cloak, rings, an amulet as the level rises; magic and rare odds rising with level). §6 |
| Formulas | the undead roster's (#383): stats, `base_hp = 2.5 × L × factor`, dice on the `2 + 0.55L` curve flattening after 20, `xp = 5L + 10`; one **role** per tier. §6.2 |
| Components | rolled with the loot system (`loot::make_as`, item level = the creature's level, material by `Source::Gear(level)`), from a dedicated stream, **`animated/<level id>`** on a generated level (spawn order, after gear) and **`animated/<actor id>`** for any other spawn. §7 |
| Powers | **nothing new**: a creature already gets `Mods::worn` from what it wears and wields (strength, Protection, resistances, elemental riders, regeneration). A panoply of rares is exactly as dangerous as its rares. §8 |
| Drops | each component (the body too) falls **intact** with `intact = 50` %, else **breaks**: its main material falls as **scrap** (six new objects, one per material group: "steel scrap", "hardwood kindling", "silk rags", ...), any gem set in it falls **loose**, its magic is lost. A new rule, `object::break_up`, reusable by later sundering and fire. §9 |
| Identity | tags `animated` and `construct`; a new faction `animated`, hostile to the player only; no corpse, no blood, poison-immune; never undead, never incorporeal. §10 |
| Where | populations: household in Cealdhlāw and Fennmynster, blades and hollow armour in Fyrdelf; panoplies wherever danger reaches 20 (strays and out-of-depth draws bring them sooner, rarely). Armoury and workshop rooms later. §10 |
| What the player sees | the name by composition, "an animated brass candlestick", "an animated bronze short sword", "an animated suit of chain mail", look mode adding ", wielding a long sword, wearing an iron helm" as it does for any creature; glyphs `j`, `J`, `A`; colour by material. Never the level, never the magic until identified. §11 |
| Talents | a new system `animated`, with **Salvager** (more components survive) and **Unbinder** (surer, harder blows against animated things); a third, **Appraiser's eye**, optional (§16 Q14). No class ability. §13 |
| First slice | **A1, the household tier alone** (instance, profile, hooks, bodies, the type, the faction, drops intact-or-nothing, the system and its two talents). Everything after it adds tiers and content on top. §15 |

## 2. Research, tied to the decisions

| Finding | Source | Used in |
|---|---|---|
| Dancing weapons are made of a real weapon and fight with it: its brand, enchantment, and ego apply, and killing one drops the weapon | DCSS, *Tukima's Dance* and dancing weapons | §6.3, §8: blades wield a real item and inherit it |
| Animated armour, the flying sword, and the rug of smothering: the iconic low-to-mid animated objects, each one item with one trick (the rug grapples) | D&D 5e Monster Manual, "Animated Objects" | §5.2 the rug's `held`; §6 tier list |
| Animated objects are built from a base and "construction points" that buy features by size and material | Pathfinder 1e Bestiary, "Animated Object" | §4 composition from parts; §5.1 material modifiers |
| Treasure goblins: a rare, mostly harmless creature that is pure loot; players drop everything to chase one | Diablo III | §9 the drop rule; §6.1 household tier as a low-level reward |
| A mimic is an item that turns out to be a creature; the surprise is its point | NetHack, ADOM, D&D | §16 Q11: a separate design, not this one |
| Brogue's spectral blades: short-lived summoned weapons in swarms; quick, fragile | Brogue CE | §6.3 blades are quick and fragile |
| Constructs: immune to poison and disease, no corpse, unaffected by fear | D&D, Pathfinder construct type | §10 the construct's traits |

## 3. What exists, and what this builds on

| Piece | Today (`docs/dev/creatures.md`, `objects.md`, `magic-items.md`, `mapgen.md`) | What animated objects need |
|---|---|---|
| Creature types | `CreatureDef` per file in `data/creatures/`; one fixed `level`, stats, `base_hp`, `attack`, `natural`, `resist`, `speed`, `glyph`, `fg`, `name`, `plural`, `weight` | a type per tier with a **level range** and `animated = { tier }`; per-creature values from the instance (§4) |
| Per-actor state | `Actor` already holds its own `stats`, `hp_base`, `inventory`, `equipment`, `weapon_skills`, `skills` (copied from the type at `travel.rs::make_creature`) | `Actor::animated` for what the type cannot hold: instance level, body, profile |
| Creature gear | `gear::roll` from `gear/<level id>` (generated levels, spawn order, after loot) or `gear/<actor id>`; `loot::make_as` at the floor's item level, material by `Source::Gear(creature level)`; overloading pieces left out | the same machinery with the **creature's** level as item level, slot by slot (§7), no burden check |
| Magic on creatures | `Mods::worn` aggregates what any actor wears and wields: `Actor::effective_stats`, `protection`, `resistances`, the attack profile's riders, `healing_rate` | nothing: inheritance is free (§8) |
| Drops | what a dying creature carried falls where it dies; its type's `loot` chance adds one item from `drop/<actor id>` | a per-component **intact or broken** draw on the same stream (§9) |
| Breaking | **nothing**: no object breaks today; no raw-material objects exist; parts and materials (`item_parts.rs`, `variant.rs`) name and price what an item is made of | `object::break_up` and six scrap objects (§9.2) |
| Danger draws | `Content::candidates` → `danger::Candidate (type, level, factor)`; `danger::weight(level, factor, danger)`; strays from the derived generic list | a candidate with a level **range** weighs as if its level were `clamp(danger, from, to)` (§10.3) |
| Undead roster formulas | #383 "Conventions": stats by level and role, hit points, the dice curve, XP; checked by `content/undead_roster_tests.rs` | the same constants, applied at birth instead of written per file (§6.2) |
| Tags | flat singular tags (#441), `tag:X` kill quests | `animated`, `construct` (§10.1) |

## 4. Representation: a type per tier, a composition per creature

### 4.1 The three ways considered

| | **A. Type per tier + instance** (chosen) | B. Generated `CreatureDef`s at load (a type per tier and level) | C. Whole creature generated and saved per actor |
|---|---|---|---|
| Content files | four creature files, the body directory, one settings file | the same, plus ~110 generated types (tiers overlap) | the same as A |
| Level | instance level (a field), clamped to the tier's range | each generated type's own `level`: no new reads | instance |
| Population lists | name four ids | need a pattern or a tag in population lists (a new feature) | four ids |
| Per-creature body and items | the instance and the inventory | still an instance: a generated type cannot know which teapot or which sword | the instance |
| Wiki, bestiary, first-kill XP | four entries; the wiki gives the formulas | 110 near-identical entries; first-kill XP 110 times | four entries |
| New reads | ~25 call sites behind six accessors (§4.3) | ~15 (no level reads) | as A, but every number saved |
| Saves | inputs plus a small derived profile | inputs only | the whole def per actor (large) |

B looks cheaper (no level reads) but still needs the instance for the body and the items, and
pays with a population-list feature and a cluttered bestiary. A keeps the content small and
puts every per-creature difference in one struct. (§16 Q1.)

### 4.2 `Animated`, the instance

```rust
/// What an animated creature is made of and what that makes it (spec §8 "Animated objects").
/// Derived once, at birth (`animated::make`); never changed after.
pub struct Animated {
    /// Its level: the floor's drawn danger clamped to its tier's range.
    pub level: u8,
    /// The household object it is, if its tier has one (a teapot of brass), with its parts.
    pub body: Option<Stack>,
    /// What its body, level, and components make it (§6): the numbers the rules read.
    pub profile: Profile,
}

pub struct Profile {
    pub name: String,        // "animated brass candlestick" (no article)
    pub plural: String,      // "animated brass candlesticks"
    pub glyph: char,
    pub fg: Rgb,
    pub speed: u32,
    pub weight: u32,         // grams: body or components' weight
    pub attack: Option<NaturalAttack>,   // None: it fights with what it wields
    pub natural: Option<NaturalArmour>,
    pub resist: Vec<(DamageTypeId, i8)>, // added to the type's (poison 100)
}
```

The worn and wielded components are **not** in `Animated`: they are the actor's inventory and
equipment, as for any creature, so combat, look mode, `Mods::worn`, the `actors` dump, and the
drop all see them with no change. The body is kept out of the inventory so it cannot be worn,
stolen, or counted as a pack (§16 Q13).

The profile is **frozen at birth**: content tuning changes new creatures, not those already
saved. Deriving on every read was rejected: speed is read every scheduler tick and the name in
every message, and the inputs (component weights, materials) would be re-summed each time.

### 4.3 The reads that change

Each becomes one accessor on `Actor`, which returns the instance's value when there is one and
the type's otherwise. No rule reads `content.creature(c).X` for these fields after the change
(a grep test, as `undead_roster_tests.rs` checks its formulas).

| Accessor | Today's reads | Notes |
|---|---|---|
| `Actor::name` / `plural` (`Cow<str>`) | 11: `fight.rs` (2), `perceive.rs` (4), `talk.rs`, `dig.rs`, `class_ability.rs`, `quest.rs` (2, by type: unchanged, a quest names the tier's type) | messages, look, quests' progress lines ("3 / 10 animated objects") use the type's plural |
| `Actor::speed` | `actor.rs:308` (already an accessor: only its base changes) | |
| `Actor::natural_attack` | `fight.rs:236`, `hold.rs:95`, `status.rs:229`, `actor.rs:1067` | |
| `Actor::natural_armour` | `actor.rs:477` | |
| `Actor::resistances` | `actor.rs:654` (innate part) | profile adds to the type's |
| `Actor::creature_level` | `experience.rs:104` (kill XP), `fight.rs:810`, `caster_level.rs:49`, `sign_picks.rs:116`, `loot.rs:81` | kill XP is `5 × level + 10` for an animated creature (the guideline), its type's `xp` unused |
| `Actor::weight` | wherever a creature's own weight is read | profile weight, plus what it carries as today |
| Glyph and colour | the renderer, through `SeenActor::creature` | `SeenActor::shape: Option<Shape>` (`Copy`: glyph, colour), filled by perception from the profile |

## 5. The data

### 5.1 `data/animated.toml`: the tiers and the rules

```toml
[drops]
intact = 50            # percent each component falls whole (spec §8 "Animated objects")
scrap = 50             # percent of a broken component's weight that falls as scrap

[[tier]]
id = "household"
creature = "animated_household"   # its type in data/creatures/
levels = [1, 8]
role = "small"                    # the roster's role (§6.2)
hp_factor = 0.5
dice_steps = -2                   # two steps under the roster's curve: almost harmless
glyph = "j"
body = true                       # draws a body from data/animated/

[[tier]]
id = "blade"
creature = "animated_blade"
levels = [5, 20]
role = "quick"
hp_factor = 0.6
glyph = "J"
slots = [{ slot = "main_hand", classes = ["melee_weapons"], chance = 100 }]
magic = { base = 0, per_level = 2, most = 40 }   # percent a component is at least magic
rare = { from = 15, per_level = 1, most = 5 }    # percent a component is rare

[[tier]]
id = "armour"
creature = "animated_armour"
levels = [12, 35]
role = "hollow"
hp_factor = 1.2
glyph = "A"
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
creature = "animated_panoply"
levels = [20, 70]
role = "standard"
hp_factor = 1.2
glyph = "A"
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
uses the real ids). `from` is the lowest instance level a slot line is tried at. The wiki prints
this table whole (§14).

### 5.2 `data/animated/<id>.toml`: the household bodies

One file per body, named by its id, read in path order like creatures (`README.toml` holds the
notes). A body is **a real object** in `data/objects/` (new ones, class `household`, §16 Q2)
plus how it moves and hits:

```toml
[[body]]
id = "candlestick"
object = "candlestick"      # in data/objects/: name, glyph, weight, materials, value
levels = [2, 6]             # within the household tier's range
weight = 100                # draw weight among the bodies at a level
speed = 900                 # its own pace (default 1000)
attack = { damage_type = "bludgeoning", name = "base", verb = "clubs" }
on_hit = []                 # as a creature's attack (rare: the rug's held)
moves = "rocks"             # the look mode's word for how it moves (optional)
dies = "topples and lies still"   # its death line's verb (default by material group, §11.3)
```

| Body | Object (materials) | Levels | Speed | Attack (type, verb) | Note |
|---|---|---|---|---|---|
| broom | wood, plant (bristles) | 1–4 | 1200 | bludgeoning, "swats" | quick, the weakest |
| stool | wood | 1–4 | 900 | bludgeoning, "barges" | hops |
| ladle | iron, brass, wood | 1–5 | 1100 | bludgeoning, "raps" | |
| cutlery | iron, pewter, silver | 1–6 | 1300 | piercing, "jabs" | "a clatter of cutlery": a stack of 6 as its body, drops as cutlery; silver ones are worth it |
| book | paper, leather | 1–6 | 1000 | bludgeoning, "slams shut on" | a scriptorium's (later rooms) |
| candlestick | brass, iron, pewter, silver | 2–6 | 900 | bludgeoning, "clubs" | silver: a low-level prize |
| teapot | brass, pewter, silver (no clay material exists yet) | 2–6 | 900 | fire, "scalds" | trivial fire damage |
| kettle | iron, copper | 3–7 | 900 | fire, "scalds" | |
| bucket | wood, iron | 2–6 | 900 | bludgeoning, "batters" | |
| chair | wood, hardwood | 3–8 | 900 | bludgeoning, "batters" | |
| warming pan | brass, copper | 3–8 | 1000 | fire, "sears" | |
| bellows | wood, leather | 3–7 | 1000 | bludgeoning, "buffets" | |
| cooking pot | iron | 4–8 | 800 | bludgeoning, "bangs into" | the heaviest, slowest |
| rug | wool, silk | 5–8 | 1000 | bludgeoning, "wraps" | `on_hit = [{ status = "held", chance = 10, severity = 1 }]`: the rug of smothering, the household tier's one trick |
| lantern | iron, brass | 4–8 | 1000 | fire, "sears" | carries a dim light (`light = "candle"`): seen coming |
| coat stand | wood | 5–8 | 800 | bludgeoning, "lurches into" | |

`[default_body]` in `data/animated.toml` gives the fields for an object that has no body file (a
legend may animate any object, §10.4): speed 1000, bludgeoning, "strikes", "falls still". A body's
`object` must exist and be of a household-ish class (a load check: not a weapon, armour, food,
coin, or key); its `levels` must lie in the household tier's range.

## 6. The four tiers

### 6.1 Overview

| Tier | Type id | Levels | Composition | How it fights | Rarity | Glyph |
|---|---|---|---|---|---|---|
| Household | `animated_household` | 1–8 | one body (a household object of a drawn material) | its body's natural attack, two dice steps under the curve: almost nothing | common | `j` |
| Blades | `animated_blade` | 5–20 | one melee weapon (sometimes magic, rarely rare) | **wields** it: the weapon's damage, threat, and riders, with Strength and a weapon-skill rank | uncommon | `J` |
| Hollow armour | `animated_armour` | 12–35 | body armour, a helm, sometimes a shield and gauntlets | its gauntlets' slam (the curve; a shield's Defence and the armour's Protection are real) | uncommon | `A` |
| Panoply | `animated_panoply` | 20–70 | armour, a weapon, a shield or helm, and as it rises gloves, boots, cloak, amulet, rings | **wields and wears** everything; rank in its weapon's skill and a style | rare (one tier per type: the type has no per-level rarity) | `A` |

A creature's tier is its type; the levels overlap so that a floor of danger 15 can bring any of
the three lower tiers.

### 6.2 Numbers by level (the roster's formulas, #383)

L is the instance level. The base values are the roster's mindless ones:

| Stat | Formula |
|---|---|
| St | 11 + 0.4L |
| To | 12 + 0.4L |
| Dx | 10 + 0.25L |
| Wp | 16 + 0.35L (a will that is not its own: hard to frighten) |
| In, Pr | 2 (mindless) |

Then the tier's role:

| Role (tier) | St | Dx | To | `base_hp` (2.5 × L × …) | Dice step | Speed | Natural armour | Resist (besides poison 100) |
|---|---|---|---|---|---|---|---|---|
| small (household) | −4 | +4 | −4 | ×0.5 | −2 | the body's | Protection by material group: metal 2, wood 1, else 0 | by material group (§6.5) |
| quick (blades) | ±0 | +5 | −2 | ×0.6 | — (its weapon) | 1200 + 5L, at most 1600 | Defence 2 + L/10 ("its thin edge") | piercing 50, bludgeoning −25 |
| hollow (armour) | +5 | −2 | +5 | ×1.2 | ±0 | 900 + 3L, at most 1100 | none (it wears its Protection) | piercing 25 (blows pass through the hollow) |
| standard (panoply) | ±0 | ±0 | +2 | ×1.2 | — (its weapon) | 1000, +4 a level past 20, at most 1200 | none | none beyond its items' |

- **Dice** (household and hollow): the roster's target `2 + 0.55L` to L20, then
  `13 + 0.25 × (L − 20)`, nearest of 1d3 … 5d12, moved by the step. Household L1 is 1d2, L8 about
  1d6; hollow armour at L20 about 3d8 (+ Strength).
- **Hit points**: `To + 2.5 × L × factor`. Household L1 has about 9, L8 about 19; a panoply at
  L40 about 150, at L70 about 250 (before what its items add).
- **Weapon skills**: blades and panoplies hold `1 + L/10` ranks in the skill of what they wield
  (max 8); a panoply with a shield holds `L/15` in Sword and board.
- **XP**: `5L + 10`, so the guideline test (`creatures_follow_the_guideline`) covers them through
  their instance level.
- **Speed and attack cost** are not changed by weight: the profile is simple on purpose. Every
  number is rounded as the roster rounds.

### 6.3 Blades

The owner's "tools and weapons". A sword, dagger, axe, spear, mace, or staff dances alone. The
component is a **melee** weapon only (no launchers, no ammunition: a bow with no archer has
nothing to draw it; §16 Q10). It is drawn among the base items of class `melee_weapons` with
`level ≤ L` by their `frequency`, as loot draws a base item, then made by `loot::make_as` at grade
plain, magic, or rare by the tier's odds. A blade is quick and fragile: easy to kill, hard to
hit, and its weapon is the prize.

### 6.4 Hollow armour and the panoply

Hollow armour is a suit with nobody inside. It fights with its gauntlets (bludgeoning, the
curve) and is slow, hard to hurt, and loud (footsteps 60). The panoply is the owner's composite:
the full kit of a knight with no knight, and it wields. Two-handed weapons leave the off hand
empty (the shield line is skipped). Rings and the amulet show in look mode as any worn item does.

### 6.5 Materials

The body's or main component's material gives the profile a little more:

| Material group | Resist | Natural Protection (household) | Death line (default) |
|---|---|---|---|
| metal | electricity −25 | 2 | "clatters to the floor" |
| wood | fire −50 | 1 | "splinters" |
| cloth (wool, silk) | fire −50, slashing −25 | 0 | "falls in a heap" |
| hide (leather) | fire −25 | 0 | "falls in a heap" |
| glass, stone, bone | bludgeoning −25 | 1 | "shatters" |

For hollow armour and panoplies, the body armour's material decides (an animated suit of iron
plate takes the metal line).

## 7. Composition: how a creature is made

`animated::make(content, rng, tier, level) -> (Animated, Pack)`, in this draw order:

1. **Level**: given by the caller: the drawn danger clamped to the tier's `levels` (§10.3). No draw.
2. **Body** (household tier): one draw among the bodies whose `levels` contain L, by `weight`;
   then its material by `variant::draw` with `Source::Gear(L)` (a silver teapot is rare, as silver
   is rare in creatures' gear); a cutlery body draws its count (3–6).
3. **Slots** (the other tiers), each line in order: one draw for its `chance` (skipped below its
   `from`, or for a shield beside a two-handed weapon); one for its base item among the line's
   classes with `level ≤ L`, by `frequency`; one for the grade (rare by `rare`, else magic by
   `magic`, else plain); then `loot::make_as(content, rng, object, 1, (L, grade), Source::Gear(L))`.
   Each piece is put in the pack and equipped. **No burden check**: it carries itself.
4. **Profile**: no draws (§6).

The stream: on a generated level, **`animated/<level id>`**, drawn in spawn order after the
level's gear (`gear/<level id>`), so the rest of the level is the same whatever animated things
it holds; for every other spawn (a level's legend, a runtime spawn, the dev `spawn`),
**`animated/<actor id>`**, as gear's `gear/<actor id>`. Neither touches the gameplay stream.
`make_creature` calls it for a type with `animated` and stores the result in `Actor::animated`,
`inventory`, and `equipment`; stats, `hp_base`, and weapon skills come from §6.2 instead of the
type's file. The type's own `gear` and `loot` must be empty (a load check).

## 8. Inherited powers

What an animated creature wears and wields works for it exactly as for a goblin with a magic
sword or the player: `Mods::worn` is the one aggregation, and every rule reads it. Nothing new is
built; this table is what the wiki says it inherits.

| Effect (`data/affixes/`) | Inherited? | Through |
|---|---|---|
| Stat raises (of the ox, of the lynx, ...) | yes | `Actor::effective_stats`: hit points, to hit, damage, Defence |
| Protection (Hardened, Reinforced) | yes | `Stack::protection` → `Actor::protection` |
| Resistances (of warding, ...) | yes | `Mods::resistances` → `Actor::resistances` |
| Elemental riders (Flaming, Shocking, ...) | yes | the attack profile's riders: a flaming blade burns |
| To hit, damage, threat, attack speed (Sharp, of celerity, ...) | yes | the attack profile |
| Regeneration | yes | `World::healing_rate` |
| Max hit points | yes | `Actor::max_hp` (set at birth after equipping, as for gear) |
| Mana, mana regeneration, caster level | held, unused | creatures do not cast yet (#397 and the creature-magic design) |
| Stealth, search, luck, light, vision | held, unused or as for any creature | what the rules read for creatures today (a vision affix gives it the profile, if any reads it) |
| Unique items' powers (`Power::Inflict`, `Vision`, `Ward`; adventurers' guild §9) | yes, once built | they are never rolled by loot, so a panoply never holds one |

**Danger**: a panoply at level 60 with two rare pieces of item level 60 is among the most
dangerous things in the game, and the most rewarding (§9.3). That is the owner's intent; the
wiki says so plainly.

## 9. Drops: intact or broken

### 9.1 The rule

When an animated creature dies, for each component in order (the body first, then its equipment
in slot order): one draw on its existing **`drop/<actor id>`** stream against `intact` (50, data):

| Draw | What falls |
|---|---|
| intact | the component as it was: material, gems, magic (unidentified), all of it |
| broken | `object::break_up(stack)`: its **main material as scrap** (§9.2), any **gem** set in it loose (same size and cut), its magic gone; a message only if the player sees it ("The long sword shatters.") |

The type's `loot` chance is 0 (the components are the loot); Scavenger (identify what falls) and
Treasure hunter (luck) apply as they do to any kill. Salvager (§13) raises `intact`.

### 9.2 Scrap: the smallest broken-parts rule

No object breaks today and no raw material exists, so this adds the smallest thing that gives a
broken item value and a future (Metalcraft and Woodcraft crafting, #196's smithing line):

| Object (new, class `materials`, new item class "Materials") | `made_of` | Named | Weight a piece | Value a piece (× material's value %) |
|---|---|---|---|---|
| `scrap` | metal | "steel scrap", "gold scrap" | 100 g | 1 |
| `kindling` | wood | "hardwood kindling" | 100 g | 1 |
| `rags` | cloth | "silk rags" | 100 g | 1 |
| `offcuts` | hide | "leather offcuts" | 100 g | 1 |
| `shards` | glass, bone, stone | "glass shards", "bone shards" | 100 g | 1 |
| — | paper, wax, plant | nothing: burnt, torn, or crushed beyond use | | |

`break_up`: count = `max(1, item weight × scrap % / 100 g)`, the stack made of the item's main
part's material (instance parts, `Stack::with_parts`), stackable with same-material scrap.
A broken steel long sword (1.5 kg) gives 7 steel scrap; a broken silver candlestick gives a few
silver scrap worth real coin. A gilt or gem-set piece keeps much of its worth broken. Shops buy
materials where they buy miscellaneous goods (§16 Q4).

### 9.3 Loot-goblin arithmetic (the wiki's table)

| Creature | Components (expected) | Intact (50 %) | Magic among them |
|---|---|---|---|
| household L4 | 1 body | 0.5 | none |
| blade L15 | 1 weapon | 0.5 | ~30 % magic, ~1 % rare |
| hollow armour L25 | 2.4 | 1.2 | ~35 % magic, ~5 % rare |
| panoply L40 | 6.1 | 3.0 | ~60 % magic, ~10 % rare |
| panoply L70 | 8.0 | 4.0 | 80 % magic, ~25 % rare |

Plus the broken half's scrap and loose gems. Compare: a goblin chief's `loot` is one roll at some
percent.

## 10. Tags, faction, and where they appear

### 10.1 Identity

| Field | Value (all four types) |
|---|---|
| `tags` | `["animated", "construct"]` (the README's group list gains `animated`; `construct` as a role for things made, not born: the mud golem gains it too, §16 Q14) |
| `faction` | new **`animated`**: "Things a will set moving and left: they set on the living who come near, and leave the rest be." Hostile to `player` only |
| `alignment` | neutral (killing one is no deed) |
| `vision`, `sight`, `hearing` | darkvision; sight 8; hearing 10 (they wake to noise like anything) |
| `footsteps` | household 20, blades 0 (it floats), armour and panoply 60 |
| `resist` | `{ poison = 100 }` plus the profile's (§6.2, §6.5) |
| `corpse`, `blood` | false, "none" |
| `opens_doors` | armour and panoply true (gauntlets); others false |
| `incorporeal`, undead | never |
| `rarity` | household common, blades uncommon, armour uncommon, panoply rare (§6.1) |
| `size` | household and blades small, the rest medium |

Statuses: Frightened rarely takes (Willpower), Bleeding has no blood to take (as the skeleton's
rules). Disease immunity is not modelled for creatures today (only as a talent mod); nothing
infects them in practice (no faction fights them), so none is added now (§16 Q17).

### 10.2 Populations

| Branch | Adds | Why |
|---|---|---|
| Cealdhlāw (barrows, danger 1+) | `animated_household`, `animated_blade` | grave goods that will not lie still |
| Fennmynster (drowned minster, danger 4+) | `animated_household` | candlesticks, books, censers of the drowned church |
| Fyrdelf (the delf, danger 8+) | `animated_blade`, `animated_armour` | the duergar smiths' work, waking |
| Deeper branches (later) | `animated_panoply` | whatever reaches danger 20 |

The generic stray list is derived from populations, so all three tiers so placed also stray
(5 %) into other branches; out-of-depth draws can bring hollow armour into Fyrdelf's top floors.
A panoply appears nowhere until a branch lists it (or the dev `spawn` brings one).

### 10.3 Level ranges in the danger draw

`danger::Candidate` gains `levels: (from, to)` (a fixed type has `(level, level)`). The weight is
`danger::weight(clamp(danger, from, to), factor, danger)`: at danger 3 a household type weighs as a
level-3 creature; at danger 30 it weighs as level 8 (far too weak: it fades out by the same curve
that fades kobolds out). The instance level is the **drawn** danger (the out-of-depth one too)
clamped to the range, passed to `animated::make`. One rule in `content/danger.rs`.

### 10.4 Hand-placed and later rooms

A level legend may place one: `creature = "animated_household", animated = { level = 3, body =
"teapot" }` (both optional; `body` may be any object, which takes `[default_body]` without a body
file), so a hand-made kitchen can hold its own crockery. Later, the special-rooms design's
themed dens and two new rooms, an **armoury** (hollow armour and blades among racks) and a
**workshop** (household things), are natural homes; they are follow-ups, not this design.

**Magic later** (#397, the magic design): an *Animate objects* spell would call
`animated::make` on objects on the floor (the floor's stacks as the components, the caster's
faction). Noted only.

## 11. What the player sees

### 11.1 Names, glyphs, colours

| Tier | Name (the profile's) | Look mode (what is added is the existing gear line) | Glyph | Colour |
|---|---|---|---|---|
| household | "animated brass candlestick", "animated clatter of silver cutlery" | "an animated brass candlestick (hostile), rocking, on the floor" | `j` | the body's material colour |
| blades | "animated bronze short sword" (the weapon's plain name, never its magic name) | "an animated bronze short sword (hostile)" (no "wielding" line: it *is* the sword) | `J` | the weapon's material colour |
| armour | "animated suit of chain mail" | "..., wearing an iron helm, a small shield" | `A` | the armour's material colour |
| panoply | "animated suit of plate armour" | "..., wielding a long sword, wearing a great helm, a tower shield, a silver ring" | `A` | the armour's material colour, a shade brighter |

Names come from the components' **unidentified** names (`Stack::names`, never `describe_truth`),
so a rare blade is "an animated steel long sword" until it falls and is identified. The type's
`name` ("animated object", "animated blade", "animated armour", "animated panoply") is what
quests, the first-kill line ("You learn something of animated blades."), and the wiki use.

### 11.2 Vagueness

The game never shows an animated creature's level, its components' magic, or its numbers;
"Unbinder" and "Appraiser's eye" say their effects in words. The wiki gives the formulas, the
tier table, the bodies table, the composition odds, the inheritance table, and the drop rule
exactly (§14).

### 11.3 Messages

| When | Line |
|---|---|
| attack | "The animated teapot scalds you." (the body's verb), "The animated short sword hits you." (the weapon's verb, as for a wielded weapon) |
| death | "The animated chair splinters." (the body's `dies`, else the material group's, §6.5) |
| a component breaks, seen | "The long sword shatters." (one line per broken component the player sees fall; quiet otherwise) |
| a component falls intact | nothing: the pile shows it |

### 11.4 Dumps

`actors` (dev): `animated level 14 tier blade body -` and the profile's speed, attack, natural, and
resist, beside the inventory it already lists (magic as it truly is). JSON per seen actor: `name`
(the profile's) besides the creature id. `progress` (dev): nothing new.

## 12. Saves, determinism, performance

| Concern | Design |
|---|---|
| Save | `Actor::animated: Option<Box<Animated>>`, `#[serde(default, skip_serializing_if = "Option::is_none")]`: other actors serialize and hash exactly as before, old saves load. Ids in the profile (damage types, materials in the body stack) go by id string, as stacks do |
| Hash | `animated` is part of the actor and so of `World::state_hash` |
| Streams | `animated/<level id>` (generated levels, spawn order, after `gear/<level id>`), `animated/<actor id>` (other spawns), `drop/<actor id>` (drops, existing). No gameplay draws. No `HashMap` iteration: slot lines and bodies in file order |
| Generated levels change | adding a type to a branch's population changes that branch's floors: their insta snapshots and any replay through them are regenerated in the PR that adds the population line (A1 for Cealdhlāw and Fennmynster). `docs/dev/merging.md`'s generated-files rule applies |
| Performance | the profile is computed once at birth; per turn an animated creature costs what a creature with gear costs. A panoply's `Mods::worn` over up to ten stacks is the open risk: A4 adds a `bench-turns` row, a fixture cave floor at danger 40 with panoplies at full population (16 ms per turn, as `delf`); a cache of `Mods::worn` per actor is added only if that row fails |

**Tests** (each slice adds its own; collected here):

| Test | Where |
|---|---|
| `animated::make` is deterministic per stream and level; a body is drawn only in its level range; the profile follows §6.2 for every tier at levels 1, mid, and top (as `undead_roster_tests.rs`) | `world/animated_tests.rs` |
| no rule reads `content.creature(..).{name, attack, natural, speed, resist}` outside the accessors (grep test) | same |
| a candidate with a range weighs by the clamped level; instance level = clamped drawn danger, out-of-depth too | `content/danger.rs` tests |
| a panoply with a Flaming long sword burns; with "of the ox" hits harder; with a resistance affix takes less | `world/animated_tests.rs`, via `fight` |
| drops: with a fixed `drop/` stream, intact components fall whole (magic kept, unidentified); broken ones as scrap of their material and a loose gem; Salvager raises the share | same |
| `break_up` counts, materials, and the paper/wax nothing case | `object` tests |
| save round trip with an animated creature; a replay with one (`replay --create`/`--verify`) | `save` tests, CLI |
| look mode names (household, blade, armour, panoply) and the gear line: text snapshot | `crates/warlock/src/ui/screen_snapshots.rs` |
| no golden screenshot: nothing here is visual beyond a glyph and a colour, which dumps and text snapshots see | |

## 13. Talents and class abilities

**The system**: `animated` ("Animated objects: what they are made of and what they leave"),
placed in `data/talents.toml` beside `creature_gear`. Two talents meet `min_talents = 2`:

| Talent | Tree | Ranks | Effect (in game words) | The one read |
|---|---|---|---|---|
| **Salvager** | delver | 2 | "You know how a bound thing comes apart: more of what an animated thing is made of survives its end." `intact` +15 a rank (50 → 65 → 80); the player's, whoever made the kill (as Treasure hunter) | the drop rule (§9.1) |
| **Unbinder** | weaponcraft | 1 | "You strike at the binding, not the thing: your blows against animated things are surer and harder." +1 to hit, +2 damage against a creature tagged `animated` | the attack profile, beside Favoured prey |
| *Appraiser's eye* (optional, §16 Q14) | scholar | 1 | "You can tell a bound thing's worth at a glance." Look mode adds "it hums with power" when any of its components is magic | perception, the one place look's creature line is built |

**Class abilities**: none (`min_class = 0`). No class is about constructs today; a priest's or
paladin's "unbind" belongs to the magic design (#397), as a spell. Salvager and Unbinder also
carry `creature_gear` and `combat` respectively in their `systems` lists, so the trees' coverage
stays even.

## 14. Spec, wiki, and docs

| File | Change |
|---|---|
| `spec/08b-creatures.md` | a section "Animated objects": tiers, instance level, composition, inherited powers, drops; the faction |
| `spec/07b-data.md` | `data/animated.toml`, `data/animated/`, the `animated` creature field, the legend's `animated` |
| `spec/07c-generation.md` "Danger levels" | level ranges in the draw |
| `spec/03b-campaign.md` | the populations (Cealdhlāw, Fennmynster, Fyrdelf) |
| `spec/07*` objects | household objects, the Materials class, scrap and `break_up` |
| `docs/dev/creatures.md` | the instance, the accessors, the streams; or a new topic `docs/dev/animated.md` (recommended: it earns `world/animated.rs` and `content/animated.rs`) |
| `docs/dev/objects.md` | scrap, `break_up` |
| `assets/data/creatures/README.toml` | `animated` field, the `animated` and `construct` tags |
| Wiki | a page **"Animated objects"** under Creatures: what they are, how they are made, the drop rule, and generated tables (`xtask/src/wiki/animated.rs`): the tiers (levels, role, glyph, slots and odds), the formulas by level (St … XP, dice at 1, 10, 20, 40, 70), the bodies (object, materials, levels, speed, attack), the inheritance table, the scrap table, the loot arithmetic; registered in `generate()` beside `creatures`. The objects page lists household goods and materials |

## 15. Implementation plan

Each issue lands rules, then UI, then content, with its spec section, `docs/dev/` topic, and wiki
in the same PR. "State" marks save state; "action" an action (the overlap rule: about three at
once).

| # | Issue | Contents | Needs | State / action |
|---|---|---|---|---|
| **A1** | **Household tier** (the first slice) | `Actor::animated` (`Animated`, `Profile`), the six accessors and their reads (§4.3), `SeenActor::shape`; `content/animated.rs` (`data/animated.toml` with the household tier and `[drops]`, `data/animated/` with the bodies of §5.2, `[default_body]`, checks); household objects in `data/objects/` (class `household`); `animated::make` for the household tier, its streams; level ranges in the danger draw; the `animated_household` type, the `animated` faction, the tags; drops **intact or nothing** (the broken half falls as nothing until A2); the legend's `animated`; Cealdhlāw and Fennmynster populations (their snapshots regenerated); the system `animated` with Salvager and Unbinder; dumps; look text snapshot; wiki page with the tier, formula, and body tables | — | state |
| A2 | **Breaking into scrap** | the `materials` item class, the five scrap objects, `object::break_up`, loose gems, the broken half of the drop rule, the break line; shops' classes; wiki scrap table | A1 | — |
| A3 | **Blades** | the `blade` tier, slot lines and grade odds (`loot` base draw by class, a small `loot::base_in`), weapon-skill ranks, the blade's name and look line; `animated_blade`; Cealdhlāw and Fyrdelf lines | A1 | — |
| A4 | **Hollow armour and panoplies** | the `armour` and `panoply` tiers, the slot lines with `from` and `count`, the two-handed rule, Sword and board ranks, the armour names; `animated_armour` in Fyrdelf; `animated_panoply` in no population yet; the `bench-turns` row; the inheritance tests | A3 | — |
| A5 | **Homes** (later, optional) | armoury and workshop rooms on the special-rooms system; a deeper branch listing panoplies | A4, special rooms R1 | — |

Order: A1, then A2 and A3 in parallel, then A4. Only A1 adds save state; none adds an action (the
dev `spawn` and the legend cover tests and captures). A1 is small enough to start now and gives
the household tier its full loop (spawn, fight, drop) before any composite exists.

## 16. Open questions

| # | Question | Recommendation |
|---|---|---|
| 1 | Representation: a type per tier with an instance (§4.1 A), or generated types per level at load (B)? | **A**: four types, one instance struct, a clean bestiary; B still needs the instance |
| 2 | Are household bodies real objects (picked up, sold: "a silver candlestick")? In a new item class `household` ("Household goods") or `miscellaneous`? | **Real objects, new class `household`**: a low-level loot-goblin reward and shop stock later |
| 3 | `intact = 50` for every component, one data field, Salvager raising it? | **Yes** |
| 4 | Broken components: scrap objects per material group (§9.2, a new class "Materials"), or simply nothing? | **Scrap**: small, valuable for precious metals, a hook for crafting; shops buy it as miscellaneous goods |
| 5 | Glyphs `j` household, `J` blades, `A` armour and panoply (letters, free today), or the item's own glyph on the creature layer (a mimic's surprise, but a pile and a creature look alike)? | **Letters**: a roguelike's map must be readable |
| 6 | Colour by material only, or a tell for magic components (a faint glow on a panoply holding a rare)? | **Material only**: the game is vague; Appraiser's eye (Q14) is the earned tell |
| 7 | Tier level ranges: household 1–8, blades 5–20, armour 12–35, panoply 20–70? | **As listed** |
| 8 | Composition strength: item level = the creature's level, panoply magic odds 20 % + 1 a level (max 80), rare odds from L20 rising to 30 %? | **As listed**; tune after A4's playtest, in data |
| 9 | Faction: their own (`animated`), hostile only to the player? | **Yes**: they do not fight kobolds or the dead; a will set them on the living |
| 10 | Flying missiles (an animated bow, a dancing javelin that is thrown and returns)? | **No for now**; a line on #196 |
| 11 | Mimics (a chest that is a creature until touched) in this design? | **No**: a separate small design (it needs a hidden-creature rule); a line on #196 |
| 12 | Legend placement with `animated = { level, body }` (and later fixed `items`)? | **Yes, level and body in A1**; fixed items when a hand-made level needs them |
| 13 | Stealing from one (if a level makes one peaceful): refused ("It is all of a piece.")? | **Refused**, no time: the body is not in the inventory and worn things are never stolen |
| 14 | Tags `animated` and `construct`; does the mud golem gain `construct`? Is Appraiser's eye wanted as a third talent? | **Both tags; the golem gains `construct`; Appraiser's eye yes**, in A4 when panoplies make it matter |
| 15 | The panoply's name: by its armour ("animated suit of plate armour"), or a word for the tier ("a hollow knight", "a panoply")? | **By its armour**, the tier word only on the wiki; the gear line tells the rest |
| 16 | Should panoplies from L50 wear rings and an amulet (seen in look mode, as any worn item)? | **Yes**: more components, more inheritance, more loot |
| 17 | Disease immunity for constructs (a creature field)? | **Not now**: nothing infects them; add with the first thing that would |
| 18 | No burden check on components (a suit carries itself)? | **Yes** |
| 19 | XP: the guideline by instance level only, or more for magic components? | **The guideline only**: the items are the reward |
| 20 | Household riders: only the rug's Held (10 %) and the scalding teapots' trivial fire? | **Yes**: the tier stays almost harmless, with one memorable trick |
