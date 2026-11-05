# Warlock — Spell types, and thirty-six new spells

Design document (#980), approved by the owner on #982 on 2026-11-05 with the answers in §10. Revised against `origin/main` at b1c203a9. Nothing here is implemented; §9 cuts it into issues. Every number was read from the files named. The balance probe was not run: the power notes reason from `docs/balance.md`, and the first issue that adds spells reruns it.

**The owner's request** (#980, verbatim): *"propose 20-40 new spells. some buffs, some damage spells, some crowd controls. also cool spells (like chain lightning, corpse explosion etc) that interact with the world little different than typical rogue like."* And: *"come up with some fantasy for spell types. Like fire spells could have small chance to leave targets burning. There was a numbing cold talent -- perhaps remove that and just give that to frost spells. Lightning spells could have a small chance to jump into adjacent target (and perhaps affect all targets in the same pool of water) etc etc"*

**Work this document accounts for**: #977 (`docs/design/no-caster-level.md`, merged; #987 to #990 build it: caster level and the class `casting` share go, nothing here reads them, and a spell's level is its mana and its study difficulty only; a scroll is learned only by those trained in Spellcraft), #979 (open: a level 1 no-roll area spell per caster; the level 1 rows of the class tables are left to it, and nothing here is a level 1 damage spell), #952 (one rank per talent, merged: the one talent here has one rank), #973 (skills' uses: untouched).

## Contents

1. Summary
2. What exists today, in one table
3. Part 1: what each spell type does
4. Part 1: the hooks, one per rule
5. Part 1: talents and affixes whose effect becomes a type's
6. Part 2: the new spells, the summary table
7. Part 2: the spells, entry by entry
8. Who gets what: class tables and scrolls
9. Implementation issues
10. Decisions

## 1. Summary

**Part 1.** Every damage type a spell can have gets a **fantasy** and a **rule that every spell of the type carries**, written once as data (`[types]` in `assets/data/spells.toml`) and read at one hook each on the existing cast path. Fire sometimes leaves its target **burning** (a new status that also makes them glow in the dark), melts ice, and leaves a fire where it lands. Cold **numbs** (the Numbing cold talent's rider, now the type's: the talent goes), freezes water, and puts fires out. Electricity **arcs** to a creature beside its target one time in four, and **conducts** through a body of water to everyone standing in it, the caster included. Force **shoves** (a big hit knocks the target a cell back) and **shatters** thin ice. Poison always **poisons** a degree. Necrotic **preys on the weak** (half again as much against a creature below half its hit points) and the undead are immune. Acid **corrodes** (a short tick after the hit). There is no holy damage type: Rebuke does fire (Q2). **No spell, type rule, or talent here protects the caster or their allies from their own magic** (Q14). The rules are magic's alone: weapon blows never read them (Q1). One talent and three spell fields are retired; one talent comes in the talent's place (Conductor, §5), under the existing `spells` system: no new tag.

**Part 2.** Thirty-six spells: 11 damage, 10 crowd control, 7 buffs, 8 that read the world (corpses, water, weather, the dead, the living) or make allies. 14 ship as **data only** on the engine as it stands, 16 need a **small rule** (one function, one hook, no save state), and 6 need a **new mechanic** (zones that lie on the floor for a time, §7 "Zones"; a corpse raised as its own creature, §7 entry 29; a charmed ally). Grouped so the owner can take a column at a time.

| Build size | Count | Spells |
|---|---:|---|
| Data only | 14 | Acid arrow, Flay, Rime, Sunfire, Grave spear, Bind, Fear, Silence, Mesmerise, Slumber, Nightsight, Glamour, Renewal, Unbinding |
| Small rule | 16 | Chain lightning, Charnel burst, Shatter, Thunderclap, Leech, Bramble, Gale, Fire cloak, Water walking, Brand, Grave feast, Blink, Sense the quick, Cloudburst, Benison, Rebuke |
| New mechanic | 6 | Balefire, Mirk, Slick, Reek (the four zones), Rouse the dead (a corpse raised as its own creature), Beguile (a charmed ally) |

## 2. What exists today, in one table

| Thing | Where | What it gives this design |
|---|---|---|
| 30 spells, one file each | `assets/data/spells/*.toml`, format `README.toml` | shapes `ray bolt touch self cone line burst`; resolutions `attack auto area touch`; kinds `damage heal buff hex cleanse utility summon`; `[spell.effect] status, severity` on a damage spell (Blight's Poisoned 2); `[spell.cast] blast, heat, frost` on a burst; ladders `[spell.grows]` |
| The cost table | `assets/data/spells.toml` | mana = level × shape (cone 160, line 180, burst 180/240/300 by radius) × resolution (touch 70) % |
| The cast path | `crates/warlock-core/src/world/cast.rs` | `cast_burst`, `cast_bolt`, `cast_cone`, `cast_single`; `spell_strikes` the one place a blow lands; `spell_lands` the one place statuses and riders fall on "everyone it hit who lives" |
| The six hooks | `content/spell_mod.rs`, `world/spell_mods.rs` | `rider = { status, severity }` on `spell_mod` with `when = ["cold"]`: Numbing cold (`assets/data/talents/numbing_cold.toml`, Spellcraft, level 12) |
| Damage types | `assets/data/damage_types.toml` | `slashing piercing bludgeoning` (physical), `fire cold electricity acid poison force necrotic`; creatures `resist` in percent (33 types resist poison 100; the undead have no necrotic line) |
| Statuses | `assets/data/status.toml` | `bleeding` (tick 40, 1 hp, eases 60), `poisoned` (tick 120), `sluggish` (cost 150, max 3), `drowsy`, `webbed`, `held`, `pinned`, `frightened` (`flees`), `stunned` (`flat_footed`, per action), `staggered`, `transfixed`, `asleep` (`sleeps`), `silenced` (`blocks cast`), `lit` (`light`), `warmed`/`chill_proof`/`earthed`/`poison_proof` (`resist`), `cat_eyed`, `spell_ward`; no burning, no corroded |
| Water and ice | `docs/dev/temperature.md`; `world/ice.rs` | terrain with a `freeze` table is water (`water`, `running_water`, `shallow_water`; plunge dice); `World::frost(center, radius, um)` freezes; `World::shock(Shock::Heat)` melts (Fireball's `heat`); `Shock::Blast(um)` shatters thinner ice, **a hook with no user yet** ("for blasts without heat, none yet") |
| Fires | `world/level.rs` `LevelState::fires: BTreeMap<Pos, Fire { light, until }>`, saved | a burning missile's or Blaze's fire: **a light source until it burns out, no rule reads it** (`docs/dev/ranged-combat.md`); the Fire-born's Blaze lays one on every cell of its burst (`blaze { fire = "arrow_fire", burns = 60 }`) |
| Corpses | `docs/dev/hunger.md`, `content/object.rs` | objects `<creature>_corpse` on a cell's pile, each naming its type (`ObjectDef::corpse: Option<CreatureId>`), one per type with `corpse = true` but the player (`content/mod.rs::load_hunger`), `Stack::died` (when it died, crate-private), stages fresh/old/rotten/gone; the player sees piles in sight and remembers them (`object_memory`). A cursed room's rising (`world/curse.rs`) already takes a corpse and stands a creature on its cell or the nearest free one |
| Blood | `docs/dev/blood.md` | stains the player sees; **no rule reads blood** (a fight is identical with and without it, by test): this design keeps it so |
| Light | `assets/data/lights.toml`, `LightCache::sources` | presets (`spell_light`, `ember`, `arrow_fire`); a status may shed a `light`; drow are `light_blind` (`glare = 250`: strong light dazzles them, `races.toml` 291) |
| Hearing | `docs/dev/hearing.md` | `noise::make` delivers at once to every actor whose `hearing` the loudness reaches; `World::heard_by(Act)` lists the hearers (War cry, Lullaby) |
| Hallowed ground | `world/sanctuary.rs` | the one **area that lies on the floor for a time**: `Hallow` in its level's state, saved and hashed, `lasts`, a check in `World::shuns`, a term in the healing pace, dropped at its end. The precedent for zones (§7) |
| Summons | `docs/dev/allies.md`, `world/summon.rs` | `kind = "summon"`: `creatures`, `count`, `most`, `minutes`, `spread`; `summon_cells` and `summon_draw`; allies with `AllySource::Summoned`, a stay in minutes, orders |
| Knock back, teleport, break a wall | `world/weapon_active.rs` 380 `knock_back` (`pub(super)`), `world/trap.rs` 420 `teleport(victim, at)`, `world/dig.rs` 216 `break_wall(p, Breach, events)` | force's shove, Blink, Shatter |
| Weather | the dev `weather:KIND` action (three hours of the kind) | Cloudburst |
| School abilities | `docs/dev/weapon-abilities.md` 196–198 | Thunderstruck (evocation 10, Staggered on a spell landing), Wasting (necromancy 10, Weakened), Aegis, Lulling: **school** rules, not type rules; untouched, and the type rules are chosen not to double them |
| The class tables | `assets/data/classes/*.toml` | wizard: Ray of frost 1, Magic missile 3, Burning hands 12, Arcane bolt 18; priest: Mend 1, Blessing 3, Cure wounds 3; druid: Spark 1, Barkskin 3, Call of the wild 6, Call lightning 12, Call of the forest 25, Call the elements 40, Flame and frost 50; witch: Evil eye 1, Hex of weakness 3, Cause wounds 12, Curse of sloth 32, Blight 40; bard: Mend 1; paladin: Blessing 6; ranger: Barkskin 6. The `casting` share goes with #990: a table's level is the author's call (`docs/design/no-caster-level.md` §4) |
| The class test | `world/spell_list_tests.rs` | "a full caster to a cantrip at 1 and **three to six spells**": §8 asks to raise it |
| Scrolls | `assets/data/objects/scroll_of_*.toml`, the convention in `objects/README.toml` 232–238 | item level about half the spell's; frequency 60 (level 1) → 20 (6) → 10 (10) → 5 (18); value 40 → 200 → 320 → 500 |
| Pools (`docs/balance.md`, the stats table's `MP Wz/P`) | | wizard / priest: 16/13 at level 1, 52/39 at 5, 103/78 at 10, 156/119 at 15, 214/162 at 20, 334/253 at 30, 464/346 at 40, 602/446 at 50, 747/547 at 60. Between the rows this document takes the straight line; the druid and the witch are read at the priest's row, and the bard, paladin, and ranger have no row, so the cast counts below are for the wizard's and priest's rows and P1's probe rerun records the rest |

## 3. Part 1: what each spell type does

The rule of a type is **always on** for every spell of that damage type, cast from mana, a scroll, or a wand (it is the spell's nature, not the caster's), and for a creature's spell once creatures cast through the path (today they are intents). It never reads the caster's level. **It is magic's alone** (Q1): a Flaming sword's blow, a Freezing arrow, and Brand's fire read no type rule. **It spares no one** (Q14): the caster and their allies are struck like anyone the rule reaches. "Harmed" below means a creature the spell took hit points from: `spell_lands` already walks exactly those.

| Type | The fantasy, in a line | Beyond damage: on the creature | Beyond damage: in the world | Draws |
|---|---|---|---|---|
| **Fire** | Fire catches, and what burns is seen | **Burning** (new status, §4): 1 degree, **one time in four** per creature harmed; a burning creature sheds `ember` light, so it cannot hide in the dark; a step into water, or a cold spell, puts it out | every fire burst **melts ice** on its cells (`heat`, today Fireball's and Flame's only, becomes the type's); a fire ray or bolt melts the cell it strikes; a fire burst **leaves a fire** on its centre cell (`arrow_fire`, 60 s, light only: Blaze's) | one `below(4)` per creature harmed, in the order struck |
| **Cold** | Cold slows the blood | **Sluggish 1** on every creature harmed (Numbing cold's rider, certain); ends **Burning** on them | every cold burst **freezes water** on its cells (`frost = 100`, today Frost's only); a cold ray or bolt freezes the one cell it strikes (Ray of frost across a lake, one cell a cast: a bridge built by hand); **puts out fires** on its cells (`LevelState::fires` removed) | none |
| **Electricity** | Lightning seeks the nearest path | **Arcs**: one time in four a bolt, ray, or line that harms a creature strikes once more at a creature beside it not yet struck (the same roll; the nearest, ties by actor id), **the caster or an ally too** if they stand beside it (Q14): Spark at a rat beside you may jump back into you | **Conducts**: a creature struck while standing in water is one of a **body of water** (the connected cells of terrain with a `freeze` table, ice left out), and everyone standing in that body is struck at the same roll, **the caster too** if they wade. A level 1 druid's Spark from the shore empties a pond of rats; from the ford it empties him | one `below(4)` per creature harmed for the arc; the body is a flood fill in cell order, no draw |
| **Force** | Raw push | **Shove**: a hit that takes at least a third of the target's maximum hit points (the `wound` table's `big_hit` threshold) knocks it one cell straight away from the caster (`knock_back`), if that cell is free; a critical of an `attack` spell always shoves | a force burst's `blast` breaks walls and blows shut chests apart as any blast does today, also **breaks shut doors** (Q5), and **shatters thin ice** through `Shock::Blast(um)` (the hook with no user), so what stood on it plunges | none |
| **Poison** | Venom lingers | **Poisoned 1** on every creature harmed (certain; a resistant creature takes less, an immune one nothing: `damage_type` on the status already does this). Blight's own line drops to severity 1 so it still poisons 2 | none (the undead's immunity is their resist line, 33 types) | none |
| **Necrotic** | Death takes the dying first | against a creature **below half** its maximum hit points the harm is **half again** (×150 %, rounded down), read before resistance; the **undead are immune** (a tag immunity, §4) | none | none |
| **Acid** | It keeps eating | **Corroded 1** (new status, §4: 1 hp every 6 s for 12 s) on every creature harmed | none: acid does not eat armour (Q6) | none |
| **Holy, unholy** | — | no such damage type (Q2): Rebuke does fire; the paladin's smites read alignment, not a type | | |

What each type **reads of the world** that already exists, and what it leaves alone:

| World | Fire | Cold | Electricity | Force | Blood | Corpses | Light | Doors |
|---|---|---|---|---|---|---|---|---|
| Reads | ice (melts), fires (lays one) | water (freezes), fires (puts out), Burning (ends) | water bodies (conducts) | thin ice (shatters), walls, shut doors, and shut chests (`blast`) | — (no rule reads blood: kept) | — (spells read corpses in Part 2, not types) | Burning sheds light | a force `blast` breaks shut doors (Q5) |

A type rule is not a talent and not a rider: Numbing cold's rider on `spell_mod` stays a hook anyone may use (`when = ["fire"]` on a future talent still works), and a type's rule sits beside it.

**Chance numbers.** One in four (25 %) is the owner's "small chance" read as: a Fireball on five kobolds leaves one or two burning; a single Scorching ray leaves its target burning once in four casts. Certain riders (cold, poison, acid) are the ones that would be dull at a chance: a cold spell that only sometimes numbs is a worse Numbing cold.

## 4. Part 1: the hooks, one per rule

The data, in `assets/data/spells.toml` (loaded into `SpellRules::types`, every status and light checked at load, a type without an entry doing nothing beyond damage):

```toml
# [types]: what every spell of a damage type does beyond its damage (the spell types design
# docs/design/spell-types-and-new-spells.md §3, §4). Each key optional.
#   rider = { status, severity, chance }   laid on every creature the spell harms, `chance` in
#            percent (default 100: certain), one draw a creature from the gameplay stream
#   ends = ["burning"]      statuses it ends on every creature it harms
#   heat = true             a burst melts ice on its cells; a ray, bolt, or line the cell it strikes
#   frost = 100             freezes water on those cells, that much (Frost's today)
#   douses = true           puts out the fires on its cells
#   fire = { light = "arrow_fire", burns = 60 }   a burst leaves a fire on its centre cell
#   arc = { chance = 25 }   a single-target or line strike jumps to a creature beside the struck
#   conducts = true         strikes everyone standing in the struck creature's body of water
#   shove = true            a big hit knocks the target a cell away from the caster
#   doors = true            a burst's blast breaks the shut doors in it (a chest is every blast's already)
#   shatter = true          a burst's blast shatters thin ice (Shock::Blast)
#   bite = { below_percent = 50, percent = 150 }   harm × percent against a target below that share of its hit points
#   immune_tags = ["undead"]   creatures with the tag take nothing of it
[types]
fire = { rider = { status = "burning", severity = 1, chance = 25 }, heat = true, fire = { light = "arrow_fire", burns = 60 } }
cold = { rider = { status = "sluggish", severity = 1 }, ends = ["burning"], frost = 100, douses = true }
electricity = { arc = { chance = 25 }, conducts = true }
force = { shove = true, shatter = true, doors = true }
poison = { rider = { status = "poisoned", severity = 1 } }
necrotic = { bite = { below_percent = 50, percent = 150 }, immune_tags = ["undead"] }
acid = { rider = { status = "corroded", severity = 1 } }
```

| Rule | The one hook | Exists | Notes |
|---|---|---|---|
| `rider` with `chance`, `ends` | `cast.rs::spell_lands`, after the cast's own `effect.status` and its riders: `afflict` the type's rider (one `rng.below(100)` per creature when `chance` < 100, in the order `spell_lands` already walks), then `Effects::cure` each of `ends` | the walk and `afflict` exist; the draw is new | a scroll's and a wand's casts go through the same `spell_lands`; the mishap backfire too (it is the spell's nature) |
| `heat`, `frost`, `douses` | `cast_burst` after the harm: `World::shock(Shock::Heat)` / `World::frost(centre, radius, um)` over the burst's cells, as the fields on the spell do today, then `LevelState::fires.remove` on each; `cast_single` and `cast_bolt`: the same at radius 0 on the cell struck (the last cell of the line) | `shock` and `frost` exist for bursts; radius 0 is a call | the spells' own `heat = true` and `frost = 100` fields go (§5); `blast` stays a spell's own (Fireball breaks walls; Frost does not) |
| `fire` | `cast_burst` after the harm: insert `Fire { light, until: now + burns }` at the centre if the cell is walkable (Blaze's code in the stars' module, moved to one `World::lay_fire(cell, light, burns)` both call) | Blaze does it | light only; no rule reads a fire, and it never spreads (Q3; wildfires are deferred to #196) |
| `arc` | `cast_single` (after the strike) and `cast_bolt` (after its targets): `World::arc_from(struck, cast)`: the creatures adjacent to the struck one, not yet struck by this cast, the caster and allies among them (Q14), sorted by distance then actor id; one `below(100)` against `chance`; the first is struck through `spell_strikes` at the cast's settled roll, then `spell_lands` runs over it (so it may arc once more: a chain of 25 % steps, which ends by itself); Chain lightning's `[spell.effect] arcs` makes the first N jumps certain (§7) | `spell_strikes` and the target walk exist; `arc_from` is new (one function) | a reflected spell arcs nothing (`spell_lands`' `cast` is `None` for it) |
| `conducts` | the same place, before `arc_from`: for each creature struck standing on a `freeze` terrain (`TerrainDef::freeze.is_some()`, ice excluded by its own id), `World::water_body(cell)` (a flood fill over 8-neighbours of `freeze` terrain in cell order, memoised per cast) and every actor standing in it not yet struck, the caster included, struck at the settled roll in actor id order | `burst_cells`' walk pattern; the flood fill is new | an area spell (Lightning bolt, Call lightning) conducts from each creature it harmed; ice is not water (frozen ponds do not conduct); the overworld has no water cells (its water is a map glyph): nothing there |
| `shove` | `spell_strikes` after the hurt, when the target lives: if the blow took ≥ ⌈max_hp / 3⌉ (the `wound` table's `big_hit` test, `blood.rs::Wound` already decides this for bleeding) or was a confirmed critical, `World::knock_back(target, away from the cast's origin)` (`weapon_active.rs` 380) | `knock_back` exists, `pub(super)` since Breaker's kick-back (#969) | a shove into water is a wade, onto thin ice the ice's own rule; a creature `held` or `webbed` is not moved (`blocks step` holds); a caster at the burst's centre has no "away" and stays |
| `shatter` | `cast_burst`: a force burst with `blast` calls `World::shock(Shock::Blast(blast))` in place of nothing | the hook exists with no user (`temperature.md` 43) | so Shatter (§7) and a wand of it break ice under a creature |
| `doors` | `cast_burst`: a burst of a type with `doors` and a spell with `blast` calls a new `World::blast_doors(center, radius, events)` after `World::blast`: every closed door in the burst's line of sight with a `breaks_to` (`TerrainDef::kicked_in`) becomes what it breaks to, with no sturdiness check; its lock and any trap on it go with it, sprung at no one, as a blown chest's do | `World::blast` already blows apart every shut chest whose band's `hardness` is below the blast (`world/dig.rs` 194–210, `docs/dev/chests.md`), for every blast; nothing new is needed for chests | Fireball's behaviour is unchanged (Q5): its fire type has no `doors`, so doors stand, and its `blast = 1000` blows chests apart as today. `doors.md`'s "never a door" gains this one exception |
| `bite` | `spell_strikes`, on the damage after the strike and before `resist`: when `target.hp × 100 < max_hp × below_percent`, damage × percent / 100 | one multiplication | the only type rule that reads the target's hit points; the wiki states it |
| `immune_tags` | `Actor::resistance(type)`: a creature whose type has any listed tag resists 100 % (the same place its `resist` line is read), after its own line | one clause | 93 undead files need no edit; a type's tag immunity shows in the `actors` dump as `resists necrotic 100` |

**Determinism**: every draw named above is from the gameplay stream, one per creature in the order the cast already walks them (actor id for an area; the flight's order for a line; the arc's sorted list), and only when the rule arises (no draw for a type without a `chance`). **Perception**: nothing here reads perception; the player is told of a conducted or arced strike only as `perceive_events` already cuts `Event::Attacked` (a creature they see is named, an unseen one is "something", one out of earshot is nothing). The aiming mode shows no arc or water preview: it only shows the aim (Q4). **Saves**: the two new statuses are data; a fire laid is `LevelState::fires` as today; nothing else is stored. **Dumps**: the `spells` layer and the menu show a type's rule in words after the roll (`4d6 fire, burns one in four`): `spell_words.rs`, the one place.

**No protection** (Q14): nothing in this design leaves out the caster or their allies. Arcs, conduction, a burst, a line, a zone, a shove, and Burning take them like anyone. Three protections already in the game are not this design's and are left as they are: Scroll savant's `spares_self` (its own bursts never catch it, `world/spell.rs` 256), `cast_area_hex` leaving out the caster (`cast.rs` 622: Fear, Silence, and Bramble go through it, allies caught, the caster not), and Blaze leaving out its user's cell.

**New statuses** (`assets/data/status.toml`, data):

| Status | Fields | Why these numbers |
|---|---|---|
| `burning` | `name = "Burning"`, `seen = "burning"`, `max = 3`, `stacking = "add"`, `tick = 10`, `damage = 1`, `damage_type = "fire"`, `eases = 30`, `light = "ember"`, `start = "You are burning!"`, `end = "The flames go out."`, `dies = "burns to death"`, `death = "burnt to death"` | a degree is 3 hit points over 30 s: between Bleeding (1 per 40 s, 60 s a degree) and nothing; three degrees from three fire spells is 9 a minute, enough to finish a fleeing kobold, never a dragon. The light (`ember`, `lights.toml`) is the tell: a burning creature in a dark room is seen |
| `corroded` | `name = "Corroded"`, `seen = "corroded"`, `max = 3`, `stacking = "add"`, `tick = 6`, `damage = 1`, `damage_type = "acid"`, `eases = 12`, `start = "The acid keeps eating."`, `end = "The acid is spent."` | two more hit points on the two turns after the hit, every cast: acid's "small number more", certain and short |

## 5. Part 1: talents and affixes whose effect becomes a type's

Every talent, affix, and spell field that touches an element, with a verdict:

| Talent / affix / field | Today | Verdict |
|---|---|---|
| **Numbing cold** (`talents/numbing_cold.toml`, Spellcraft, level 12, `spell_mod { rider = sluggish 1, when = ["cold"] }`) | the owner's example | **Removed.** Cold's own (§3). Its tests in `world/spell_talent_tests.rs` and `cast_tests.rs` ("Numbing cold's rider on every target harmed and none missed and on cold spells only") become the type's tests. Its Spellcraft slot at level 12 goes to **Conductor** (below) |
| **Conductor** (new, `talents/conductor.toml`, Spellcraft, `needs = { level = 12 }`, systems `spells`) | — | *"Lightning knows your hand: your electricity spells arc every time, and what comes back at you bites less."* `effects = [{ kind = "spell_type", arc_chance = 100, when = ["electricity"] }, { kind = "resist", damage = "electricity", value = 25 }]`, as the owner set it (§10 Q14): a new `TalentEffect::SpellType` read in `arc_from` (the chance), and the existing `resist` kind with a named `damage`, as Flame-hardened has it. It is no protection: every arc may jump back and the water still conducts to the caster, each at 25 % less. A chain ends only when no creature not yet struck stands beside the last. The alike check (`content/talent_alike.rs`) compares a talent's whole effect list and its `choice`, so neither Elemental ward (`choice = "element"`, one `resist` with no `damage`) nor Flame-hardened is a clone of it, and no `alike` line is needed; if the check ever refused it, the pair goes in `alike` in `assets/data/talents.toml`. One rank |
| Flame and frost (class, the druid's two spells) | a grant | stays |
| Blaze, Wildfire, Kindled, Flame-hardened, Fire breath, Transfixing stare (`tree = "stars"`) | birth sign actives and resists | stay (they are not spells); Blaze's fire-laying becomes a call of the shared `lay_fire` |
| Elemental ward (Guard, choice: element, `resist 25`) | the other side of the elements | stays (one rank, per the talent-ranks design) |
| School focus, Keen mind (choice: a school or an element) | generic hooks (`cost`, `cells`, `sides`) | stay |
| Sure hand, Brimming, Empower, Archmage, Scroll adept, Wand mastery | generic hooks | stay |
| Thunderstruck, Wasting, Aegis, Lulling (school abilities at rank 7–10) | Staggered, Weakened, Spell ward, Drowsy on a spell landing | stay: **school** rules, and the type rules were picked to not double them (force shoves rather than staggers; necrotic bites rather than weakens) |
| Flaming, Freezing, Shocking, Corrosive, Venomous, Withering (weapon affixes, `effect = "elemental"`) | elemental damage on blows | stay as they are: type rules never apply to weapon blows (Q1) |
| Baleful, of Lingering, of Farcasting, Potent, of the Magus, of Thrift, of Wands, Mage-wrought | generic spell hooks | stay |
| of the Adept (`caster_level`) | learning only | goes with #977, not here |
| Fireball's and Flame's `heat = true`, Frost's `frost = 100` (`[spell.cast]`) | the world reads on three spells | **Removed from the spells**: the type's. `spell_def` keeps accepting the keys for a campaign's spell that wants more or less than its type (an override), but the shipped three drop them |
| Blight's `effect = { status = "poisoned", severity = 2 }` | its own poisoning | `severity = 1`: with the type's degree it still poisons 2 |
| Ray of frost, Spark, Scorching ray, Magic missile, Arcane bolt, Burning hands, Lightning bolt, Call lightning, Cause wounds | no world reads | gain their type's rule with no edit: Ray of frost freezes a cell, Spark arcs and conducts, Magic missile shoves on a big hit, Cause wounds bites the wounded |

**Coverage policy** (`docs/dev/progression.md`): spell types extend the `spells` system; no new `[[system]]` tag. Conductor is the one talent, in the removed one's place, so the Spellcraft tree's count and the `spells` system's `min_talents` are unchanged.

## 6. Part 2: the new spells, the summary table

Mana is the cost table's at the base reach (`level × shape × resolution`), before armour and talents. "Comes" is the class table row (§8) or "scroll". Size: **D** data only, **S** a small rule, **N** a new mechanic. "Borrowed" names a concept from another game for the owner's decision.

| # | Spell | School | Level | Shape, resolution | Type | Mana | Comes | Size | Borrowed |
|---:|---|---|---:|---|---|---:|---|:-:|---|
| | **Damage** | | | | | | | | |
| 1 | Chain lightning | evocation | 20 | bolt, auto | electricity | 20 | wizard 24, scroll | S | D&D |
| 2 | Charnel burst (corpse explosion) | necromancy | 9 | a corpse, burst r1 | necrotic | 16 | witch 18, scroll | S | Diablo II |
| 3 | Balefire (wall of fire) | evocation | 16 | line 5, a zone | fire | 29 | scroll | N | D&D |
| 4 | Shatter | evocation | 8 | burst r1, range 8 | force | 14 | wizard 9, scroll | S | D&D |
| 5 | Acid arrow | evocation | 5 | bolt, attack | acid | 5 | scroll | D | D&D (Melf's) |
| 6 | Thunderclap | evocation | 7 | burst r1 on the caster | force | 13 | bard 12, scroll | S | D&D (Thunderwave) |
| 7 | Flay | necromancy | 8 | ray, auto | necrotic | 8 | witch 8, scroll | D | — |
| 8 | Rime | evocation | 9 | cone 4 | cold | 14 | druid 15, scroll | D | — |
| 9 | Sunfire | evocation | 30 | burst r3, range 10 | fire | 90 | wizard 36, scroll | D | D&D (Sunburst) |
| 10 | Grave spear | necromancy | 14 | line 6 | necrotic | 25 | scroll | D | Diablo II (Bone spear) |
| 11 | Leech | necromancy | 10 | ray, attack | necrotic | 10 | witch 24, scroll | S | DCSS (Vampiric draining) |
| | **Crowd control** | | | | | | | | |
| 12 | Bind | enchantment | 9 | ray, auto | — | 9 | bard 16, scroll | D | D&D (Hold person) |
| 13 | Fear | enchantment | 10 | burst r1, range 8 | — | 18 | witch 36, scroll | D | D&D |
| 14 | Mirk (darkness) | evocation | 12 | burst r2, a zone | — | 29 | scroll | N | D&D (Darkness) |
| 15 | Slick | transmutation | 6 | burst r1, a zone | — | 11 | scroll | N | D&D (Grease) |
| 16 | Reek (stinking cloud) | necromancy | 13 | burst r2, a zone | poison | 31 | scroll | N | D&D |
| 17 | Silence | abjuration | 11 | burst r1, range 8 | — | 20 | priest 12, scroll | D | D&D |
| 18 | Bramble (entangle) | transmutation | 8 | burst r2, range 8 | — | 19 | druid 9, scroll | S | D&D (Entangle) |
| 19 | Mesmerise | enchantment | 5 | ray, auto | — | 5 | bard 8, scroll | D | — |
| 20 | Slumber | enchantment | 15 | ray, auto | — | 15 | scroll | D | — |
| 21 | Gale | evocation | 11 | line 6, a hex | — | 20 | druid 20, scroll | S | D&D (Gust of wind) |
| | **Buffs** | | | | | | | | |
| 22 | Nightsight | transmutation | 4 | self | — | 4 | ranger 9, scroll | D | — |
| 23 | Glamour | abjuration | 8 | self | — | 8 | scroll | D | D&D (Blur) |
| 24 | Fire cloak | abjuration | 12 | self | fire | 12 | scroll | S | D&D (Fire shield) |
| 25 | Water walking | transmutation | 5 | self | — | 5 | ranger 12, druid 6, scroll | S | D&D |
| 26 | Brand | transmutation | 7 | self | fire | 7 | paladin 14, scroll | S | D&D (Flame blade) |
| 27 | Renewal | restoration | 10 | self | — | 10 | priest 9, scroll | D | — |
| 28 | Unbinding | abjuration | 8 | self | — | 8 | paladin 20, scroll | D | D&D (Dispel, narrowed) |
| | **The world, the dead, the living** | | | | | | | | |
| 29 | Rouse the dead | necromancy | 24 | a corpse, rises as its own creature | — | 24 | witch 28, scroll | N | D&D (Animate dead) |
| 30 | Grave feast | necromancy | 6 | a corpse beside, touch | — | 4 | witch 14, scroll | S | — |
| 31 | Blink | transmutation | 6 | self | — | 6 | wizard 6, scroll | S | D&D |
| 32 | Sense the quick | divination | 5 | self | — | 5 | bard 20, scroll | S | — |
| 33 | Cloudburst | transmutation | 20 | self, open sky | — | 20 | druid 30, scroll | S | — |
| 34 | Beguile | enchantment | 18 | ray, auto | — | 18 | witch 44, scroll | N | D&D (Charm) |
| 35 | Benison | restoration | 12 | burst r1 on the caster, heal | — | 22 | priest 24, scroll | S | D&D (Prayer of healing) |
| 36 | Rebuke | abjuration | 14 | burst r2, range 8 | fire | 34 | priest 18, scroll | S | D&D (Turn undead) |

Levels by class and shape, checked against the pools of §2: a spell's mana at its class level is 5 to 10 casts from the pool for the bread-and-butter spells (Flay 8 at witch 8: pool about 62, 7 casts; Shatter 14 at wizard 9: pool about 93, 6 casts), about 4 for the nukes (Sunfire 90 at wizard 36: pool about 412, 4 casts; Rebuke 34 at priest 18: pool about 145, 4), as Fireball's 26 at wizard 12 from about 124 is 4 today.

## 7. Part 2: the spells, entry by entry

Each entry: the file as it would be written (the real format, every field it needs), what it does in the world, who gets it, and its size with the hook named. Ladders follow the README's conventions ([A] reach by Willpower at 14, 18; [B] a die a rung by the school's stat at 13, 16, 19, 25; degrees by Presence at 14, 18).

### Zones: the one new mechanic four spells share

A **zone** is an area that lies on the floor for a time, as hallowed ground does (`world/sanctuary.rs`: `Hallow` in the level's state, `lasts`, dropped at its end). `LevelState::zones: Vec<Zone { kind, cells, until, by }>` (saved and hashed, left out while empty, so saves without one hash as before), laid by a spell with `[spell.effect] zone = { kind, turns }`, its cells the spell's shape's cells at the cast. One reader a kind, each at an existing moment:

| Kind | What it does | The one hook |
|---|---|---|
| `fire` | a creature that starts its turn on it, or steps onto it, takes the zone's roll of fire (the spell's `damage`, settled at the cast, kept on the zone) through `spell_strikes` (so the fire type's Burning applies); a fire of light on each cell while it lasts | the start of a creature's turn (`World::creature_turn`) and the player's (`run_turn`), and `after_tread` for a step; `LightCache::sources` reads the zone's cells as it reads `fires` |
| `fouled` | a creature that starts its turn on it is Poisoned 1 (the type's rider; a poison-immune creature nothing) | the turn-start hook above |
| `slick` | a creature that steps onto it, or starts its turn on it and moves, makes a Dexterity check against 12 (`check(rng, modifier, difficulty)`, `crate::check`): failed, it is Staggered 1 (the next action half again as slow) and its step is spent | `after_tread` |
| `dark` | no light reaches its cells: the light cache sets their light to 0 and a source inside sheds nothing (one clause in the light pass); creatures without darkvision see nothing inside it, the player too, by the existing sight rules | `LightCache` (core: the light cache is the perception's, in `perception/light`; the renderer reads the perceived light as ever) |

Rules: a zone's cells are fixed (it never spreads: Q3; wildfires are deferred to #196); it takes the caster and their allies like anyone (Q14: `by` is whose kill it is, never who it spares); a new zone over an old one replaces it on the shared cells; leaving the level keeps it; it ends at `until` and is dropped at the end of that turn, as a hallow is. Zones are ground truth: the player sees a zone's cells as the perception shows them (a fire by its light and its look, a cloud by a look-mode word, dark by the dark) and remembers them as terrain is remembered, through `Perception` (`PerceptionView::zone_at`, a new perception field; the `truth` dump lists them). The dev action `zone:KIND@X,Y,R` lays one for captures. Draws: Slick's check only, at the stepper's step. Effects: a zone shows as a still tint of its cells, no particles (subtle); a fire zone's light flickers as a fire's does.

### 1. Chain lightning

```toml
[[spell]]
id = "chain_lightning"
name = "Chain lightning"
description = "A bolt of lightning toward a place you aim at: it strikes the first creature in its way, then leaps to the next, and the next."
level = 20
school = "evocation"
kind = "damage"
shape = "bolt"
resolution = "auto"
damage_type = "electricity"
range = 8
damage = "3d6"
effect = { arcs = 3 }
[spell.cast]
effect = "lightning_bolt"
noun = "chain lightning"
verb = "strikes"
message = "Lightning leaps from your hands, and leaps again!"
noise = { sound = "thunder", loudness = 120 }
[spell.grows]
damage = "intelligence 13 4d6, 16 5d6, 19 6d6, 25 6d6+3"
range = "willpower 14 9, 18 10"
count = "willpower 14 4, 18 5, 22 6"
```

| | |
|---|---|
| In the world | the type's arc, made certain and long: `arcs` jumps are taken without a draw, each to the nearest creature within **2 cells** of the last struck not yet struck (ties by actor id), the type's 25 % chance applying to a further jump after them; every struck creature standing in water conducts to its body. Six kobolds in a corridor are six strikes; two by the pond are the pond |
| Who | wizard 24 (the first evocation past Arcane bolt), scroll (item level 10, frequency 10, value 320) |
| Size | **S**: `[spell.effect] arcs` (1 to 12) and the `count` ladder reused for jumps (`Grows::count` is a summon's today; `spell_def` allows it on a damage spell with `arcs`); `arc_from` takes a `reach` of 2 and a `certain` count |
| Against | Lightning bolt (level 11, a line of 8, 2d6 → 5d6+2, 20 mana): the line is cheaper and needs a row; the chain costs as a line, follows the creatures where they stand, and conducts twice |

### 2. Charnel burst (corpse explosion)

```toml
[[spell]]
id = "charnel_burst"
name = "Charnel burst"
description = "Burst a corpse you see: its rot and bone tear everyone beside it."
level = 9
school = "necromancy"
kind = "damage"
shape = "burst"
resolution = "area"
damage_type = "necrotic"
range = 8
radius = 1
damage = "2d6"
effect = { corpse = "consumed" }
[spell.cast]
effect = "blight"
noun = "burst"
verb = "tears"
noise = { sound = "explosion", loudness = 60 }
[spell.grows]
damage = "willpower 13 3d6, 16 4d6, 19 5d6, 25 5d6+2"
range = "willpower 14 9, 18 10"
```

| | |
|---|---|
| In the world | aimed at a cell the player **sees** that holds a corpse (any stage but `gone`): the burst goes off there, the corpse is used up (its stack removed from the pile: no meal, no second burst), and its blood is spattered as a kill's (`blood::Wound` with the corpse's creature's `blood`, presentation only). Refused, no time, no mana, at a cell with no corpse the player sees ("There is nothing there to burst."). The kills it makes leave corpses: a line of kobolds is a chain of bursts, each paid. The necrotic bite applies (wounded creatures beside the corpse take half again); the undead beside it take nothing |
| Who | witch 18, scroll (item level 5, frequency 25, value 150) |
| Size | **S**: `Aim::Corpse` (the aiming mode's cursor accepts a cell in sight with a corpse the player sees: `PerceptionView` already shows piles in sight; `Ability::aim` names it "a corpse you see"); `[spell.effect] corpse = "consumed" \| "kept"` on a damage spell or a summon (§29) or a touch (§30): `cast_aim` refuses without one; the burst is `cast_burst` at the cell with `range` as its reach check. The corpse's stage is not read (a rotten corpse bursts as well as a fresh one: nothing is revealed) |
| Determinism | one roll, the area's; the pile's top-most corpse stack is the one used (pile order is deterministic) |

### 3. Balefire (wall of fire)

```toml
[[spell]]
id = "balefire"
name = "Balefire"
description = "A line of fire from before you toward a place you aim at: it burns a while, and all who stand in it or cross it."
level = 16
school = "evocation"
kind = "damage"
shape = "line"
resolution = "area"
damage_type = "fire"
length = 5
damage = "2d6"
effect = { zone = { kind = "fire", turns = 10 } }
[spell.cast]
effect = "fire_breath"
noun = "balefire"
verb = "burns"
message = "Fire springs up along the ground!"
noise = { sound = "explosion", loudness = 60 }
[spell.grows]
damage = "intelligence 13 3d6, 16 4d6, 25 4d6+2"
length = "willpower 14 6, 18 7"
```

| | |
|---|---|
| In the world | a line of `fire` zone cells from the caster toward the aim, burning 10 turns (the player's turns: 60 s): everyone on the line is burnt at the cast (as a line spell), and whoever starts a turn on it or steps onto it after; each cell a fire of light. A corridor shut behind the caster; a creature that flees through it burns. The fire type: Burning one in four per hit, ice under it melted, and a cold spell's `douses` puts it out cell by cell (Frost against Balefire: the counter) |
| Who | scroll only (item level 8, frequency 15, value 250): a find |
| Size | **N**: the zone mechanic, `fire` kind |

### 4. Shatter

```toml
[[spell]]
id = "shatter"
name = "Shatter"
description = "A clap of force that bursts where you aim it: it cracks walls, ice, shut doors, and shut chests, and batters everyone it reaches."
level = 8
school = "evocation"
kind = "damage"
shape = "burst"
resolution = "area"
damage_type = "force"
range = 8
radius = 1
damage = "2d6"
[spell.cast]
effect = "arcane_bolt"
noun = "shatter"
verb = "batters"
message = "The air cracks!"
blast = 1000
noise = { sound = "explosion", loudness = 100 }
[spell.grows]
damage = "intelligence 13 3d6, 19 3d6+2, 25 3d6+4"
range = "willpower 14 9, 18 10"
```

| | |
|---|---|
| In the world | `blast = 1000` breaks what a Fireball breaks (earth and brick, not granite: `hardness` 600 breaks, 3000 does not), blows apart every shut chest in the burst whose band's `hardness` is below 1000 (`World::blast`, as Fireball's does: coffer 200 to strongbox 900; a kist, 1200, and up hold), and through the force type **breaks every shut door in it**, locked or not (Q5), and **shatters thin ice** on its cells (`Shock::Blast`): a band crossing a frozen pond is in the water. The shove: a big hit knocks each creature a cell outward. A caster in reach of their own burst takes all of it |
| Who | wizard 9 (a level 8 area for 14 mana from a pool of about 80), scroll (item level 4, frequency 30, value 130) |
| Size | **S**: a burst's `effect` presentation needs a force burst look (`effects.toml`: a `shatter` burst, white and brief, no particles); the rest is the type's |

### 5. Acid arrow

```toml
[[spell]]
id = "acid_arrow"
name = "Acid arrow"
description = "A dart of acid toward a place you aim at: it strikes the first creature in its way, if it hits, and keeps eating."
level = 5
school = "evocation"
kind = "damage"
shape = "bolt"
resolution = "attack"
damage_type = "acid"
range = 8
damage = "1d6"
[spell.cast]
effect = "blight"
noun = "acid arrow"
noise = { sound = "chanting", loudness = 30 }
[spell.grows]
damage = "intelligence 13 2d6, 16 3d6, 25 3d6+2"
range = "willpower 14 9, 18 10"
```

| | |
|---|---|
| In the world | the first acid spell: Corroded 1 on a hit (two more points over the next two turns), and only six shipped types resist acid (the beetle and five yugoloths), so it is the ray for creatures that shrug off fire and cold |
| Who | scroll only (item level 3, frequency 35, value 100): a scroll worth finding for a wizard whose Scorching ray meets a fire-thing |
| Size | **D** (a bolt effect: `blight`'s green serves until an `acid` bolt look is drawn) |

### 6. Thunderclap

```toml
[[spell]]
id = "thunderclap"
name = "Thunderclap"
description = "A clap of thunder about you: everyone beside you is battered and reels, you too, and the whole level hears it."
level = 7
school = "evocation"
kind = "damage"
shape = "burst"
resolution = "area"
damage_type = "force"
range = 0
radius = 1
damage = "1d6"
effect = { status = "stunned", severity = 1 }
[spell.cast]
effect = "shatter"
noun = "thunderclap"
verb = "batters"
message = "Thunder cracks about you!"
noise = { sound = "thunder", loudness = 150 }
[spell.grows]
damage = "presence 13 2d6, 19 2d6+2"
radius = "willpower 18 2"
```

| | |
|---|---|
| In the world | a burst **on the caster** (`range = 0`: no aim): everyone in it is hit and Stunned 1 (flat-footed for its next action), **the caster too** (Q14: their cell is in the burst, as any burst's is), and everyone else is shoved on a big hit (the caster, at the centre, has no "away"). It trades your next action for theirs: a price, with a ring of foes round you, that may be worth paying. Loudness 150 is the level's alarm: every sleeper wakes (`sleeps` wakes at the den's threshold), every creature that hears it comes. Time is the penalty |
| Who | bard 12, scroll (item level 4, frequency 30, value 130) |
| Size | **S**: `range = 0` on a burst (`spell_def` allows 0 for a burst only: cast at the caster's cell, no aiming mode, `Aim::None`; no cell left out). The `damage` ladder on Presence is the bard's stat (the README: "unless the spell says otherwise") |

### 7. Flay

```toml
[[spell]]
id = "flay"
name = "Flay"
description = "Open a foe you see nearby: its skin splits and it bleeds."
level = 8
school = "necromancy"
kind = "damage"
shape = "ray"
resolution = "auto"
damage_type = "necrotic"
range = 6
damage = "1d4"
effect = { status = "bleeding", severity = 2 }
[spell.cast]
effect = "frost_ray"
noun = "flaying"
verb = "flays"
noise = { sound = "chanting", loudness = 20 }
[spell.grows]
damage = "willpower 16 1d6, 22 1d8"
range = "willpower 14 7, 18 8"
degrees = "presence 14 3, 18 4"
```

| | |
|---|---|
| In the world | little harm, but Bleeding 2 to 4 (a hit point every 40 s a degree, easing 60 s a degree: 2 degrees are about 5 points over two minutes, 4 degrees about 13): the witch's damage over time, and a blood trail on the floor (`bleed.rs` stains as the ticks hurt, presentation). A bloodless creature (`blood = "none"`: 89 types, the bone and spirit undead and the elementals among them) refuses Bleeding (Q7: one clause in `afflict_one`, `world/status.rs`, beside the `immune` check: a status with a `wound` table takes only on a creature with blood), so Flay does nothing to a skeleton beyond its 1d4. The necrotic bite: the ticks are the status's, not the spell's, so only the 1d4 bites |
| Who | witch 8 (her first damage at range), scroll (item level 4, frequency 30, value 130) |
| Size | **D** (`degrees` on a damage spell's status: `spell_def` today allows `degrees` for a buff or a hex only; allowing it for a damage spell with an `effect.status` is a one-line widening: *small*) |

### 8. Rime

```toml
[[spell]]
id = "rime"
name = "Rime"
description = "A breath of killing frost from your hands toward a place you aim at: everyone in it is chilled and slowed, and the water in it freezes."
level = 9
school = "evocation"
kind = "damage"
shape = "cone"
resolution = "area"
damage_type = "cold"
length = 4
damage = "2d6"
[spell.cast]
effect = "frost_burst"
noun = "rime"
verb = "bites"
message = "Frost pours from your hands!"
noise = { sound = "explosion", loudness = 40 }
[spell.grows]
damage = "willpower 13 3d6, 16 4d6, 22 4d6+2"
length = "willpower 18 5"
```

| | |
|---|---|
| In the world | the cold type over a cone: Sluggish on everyone, water in the cone frozen (`frost = 100` on each cell: a creature wading in it stands on new thin ice, and the ice rules decide whether it holds), fires put out, Burning ended. The druid's Willpower (as Call lightning's) |
| Who | druid 15 (a cold spell before Flame and frost at 50), scroll (item level 5, frequency 25, value 150) |
| Size | **D** |

### 9. Sunfire

```toml
[[spell]]
id = "sunfire"
name = "Sunfire"
description = "A piece of the sun bursts where you aim it: everything in it burns, the dark is driven out, and eyes that shun the light are dazzled."
level = 30
school = "evocation"
kind = "damage"
shape = "burst"
resolution = "area"
damage_type = "fire"
range = 10
radius = 3
damage = "5d6"
[spell.cast]
effect = "fireball"
noun = "sunfire"
verb = "burns"
message = "The sun itself bursts from your hands!"
noise = { sound = "explosion", loudness = 200 }
[spell.grows]
damage = "intelligence 16 6d6, 19 7d6, 22 8d6, 25 8d6+4"
range = "willpower 14 11, 18 12"
```

| | |
|---|---|
| In the world | the biggest shipped burst (radius 3, the cost table's last column: ×300 %), no `blast` (it does not dig), the fire type over 37 cells: ice melted across the pond, a fire laid at the centre, and one in four burning. The 60 s fire is light of radius 5: a dark hall is lit for a minute after, and the drow in it are dazzled by it (`light_blind`, `glare = 250`: `arrow_fire` is 260 at the centre: the existing rule). Fire-immune creatures (7 types with `fire = 100`) shrug it off whole: a nuke with an answer |
| Who | wizard 36 (90 mana, three casts of a pool of about 300), scroll (item level 15, frequency 5, value 500) |
| Size | **D** |

### 10. Grave spear

```toml
[[spell]]
id = "grave_spear"
name = "Grave spear"
description = "A spear of grave-cold bone from your hand toward a place you aim at, through everyone on its line."
level = 14
school = "necromancy"
kind = "damage"
shape = "line"
resolution = "area"
damage_type = "necrotic"
length = 6
damage = "2d6"
[spell.cast]
effect = "arcane_bolt"
noun = "grave spear"
verb = "pierces"
noise = { sound = "chanting", loudness = 40 }
[spell.grows]
damage = "willpower 13 3d6, 16 4d6, 19 5d6, 25 5d6+2"
length = "willpower 14 7, 18 8"
```

| | |
|---|---|
| In the world | the witch's line: through a corridor of the living, nothing to the undead; the bite finishes what Flay bled |
| Who | scroll only (item level 7, frequency 15, value 250): the witch's table already holds six beside her cantrip after this design |
| Size | **D** |

### 11. Leech

```toml
[[spell]]
id = "leech"
name = "Leech"
description = "A ray of hunger toward a place you aim at: it strikes the first creature in its way, if it hits, and what it takes mends you."
level = 10
school = "necromancy"
kind = "damage"
shape = "ray"
resolution = "attack"
damage_type = "necrotic"
range = 6
damage = "2d6"
effect = { leech = 50 }
[spell.cast]
effect = "frost_ray"
noun = "leeching ray"
verb = "drains"
noise = { sound = "chanting", loudness = 20 }
[spell.grows]
damage = "willpower 13 3d6, 16 4d6, 25 4d6+2"
range = "willpower 14 7, 18 8"
```

| | |
|---|---|
| In the world | half the harm done (after resistance and the bite, rounded down) comes back to the caster as hit points, never past their maximum; the undead give nothing (immune). The witch's sustain, aimed: her Toucher style does not apply (a ray), her Sniper does |
| Who | witch 24, scroll (item level 5, frequency 20, value 200) |
| Size | **S**: `[spell.effect] leech` (percent, 1 to 100) read in `spell_strikes` after the hurt (`World::heal_points(caster, harm × leech / 100)`), one line; shown in the menu's roll words, "2d6 necrotic, half of it yours" |

### 12. Bind

```toml
[[spell]]
id = "bind"
name = "Bind"
description = "Bind a foe you see nearby where it stands: it cannot step for a few actions, though it may still strike."
level = 9
school = "enchantment"
kind = "hex"
shape = "ray"
resolution = "auto"
range = 6
effect = { status = "pinned", severity = 2 }
[spell.grows]
range = "willpower 14 7, 18 8"
degrees = "presence 14 3"
```

| | |
|---|---|
| In the world | Pinned (`blocks step`, per action, max 3): the target cannot step for its next 2 or 3 actions; it fights if beside, shoots if it can. Against an archer behind a door it is a held door; against a charging ogre it is three free shots. No level limit (the house rule: Sleep has none) |
| Who | bard 16, scroll (item level 5, frequency 25, value 150) |
| Size | **D** (`pinned` is a hex's status as `webbed` is: `afflict` with the caster as giver) |

### 13. Fear

```toml
[[spell]]
id = "fear"
name = "Fear"
description = "Dread bursts where you aim it: everyone it reaches runs from you a while, and fights only when cornered."
level = 10
school = "enchantment"
kind = "hex"
shape = "burst"
resolution = "area"
range = 8
radius = 1
effect = { status = "frightened", severity = 2 }
[spell.cast]
noise = { sound = "chanting", loudness = 30 }
[spell.grows]
range = "willpower 14 9, 18 10"
degrees = "presence 14 3"
```

| | |
|---|---|
| In the world | Frightened (`flees`: runs from the giver while a step takes it farther, fights only cornered; 6 s a degree): a ring broken for two or three turns; the fleeing drag their friends' line of fire with them. Intimidate's status at an area and no check (the skill's check is its own thing: this costs mana and the spell's time) |
| Who | witch 36, scroll (item level 5, frequency 20, value 200) |
| Size | **D** (an area hex as Sleep: `cast_area_hex`) |

### 14. Mirk (darkness)

```toml
[[spell]]
id = "mirk"
name = "Mirk"
description = "A pall of darkness bursts where you aim it and lies there a while: no light reaches into it, and eyes that need light see nothing in it."
level = 12
school = "evocation"
kind = "hex"
shape = "burst"
resolution = "area"
range = 8
radius = 2
effect = { zone = { kind = "dark", turns = 30 } }
[spell.cast]
noise = { sound = "chanting", loudness = 20 }
[spell.grows]
range = "willpower 14 9, 18 10"
radius = "willpower 18 3"
```

| | |
|---|---|
| In the world | a `dark` zone (§ Zones) of 13 cells for 30 turns: no light reaches them (a torch inside sheds nothing); creatures without darkvision (the vision profiles in `vision.toml`: `normal`, `low_light` see by light) lose sight of what is inside, and so does the player without darkvision: cast it on the archers, not on oneself, unless one is a dwarf. Burning creatures still glow? No: no light reaches the cells, their own included (a burning creature inside is dark too: simple and strict) |
| Who | scroll only (item level 6, frequency 20, value 200) |
| Size | **N**: the zone mechanic's `dark` kind, the one clause in the light pass |

### 15. Slick

```toml
[[spell]]
id = "slick"
name = "Slick"
description = "The ground where you aim it turns slick as ice a while: whoever steps on it may slip and lose their footing."
level = 6
school = "transmutation"
kind = "hex"
shape = "burst"
resolution = "area"
range = 8
radius = 1
effect = { zone = { kind = "slick", turns = 20 } }
[spell.cast]
noise = { sound = "chanting", loudness = 20 }
[spell.grows]
range = "willpower 14 9, 18 10"
```

| | |
|---|---|
| In the world | a `slick` zone of 9 cells for 20 turns: a creature that steps onto a cell of it makes a Dexterity check against 12; failed, Staggered 1 and the step is spent on the cell it stepped onto (it still moved; its next action is half again as slow). Laid at a doorway, a charge becomes a stumble. Ice is not slick (thin ice has its own rule); the player slips too |
| Who | scroll only (item level 3, frequency 35, value 100) |
| Size | **N**: the zone mechanic's `slick` kind (the check draws from the gameplay stream at the stepper's step). Ship it last of the zones: it is the least needed |

### 16. Reek (stinking cloud)

```toml
[[spell]]
id = "reek"
name = "Reek"
description = "A cloud of foul vapour bursts where you aim it and hangs there a while: everyone in it is poisoned, and poisoned again while they stay."
level = 13
school = "necromancy"
kind = "damage"
shape = "burst"
resolution = "area"
damage_type = "poison"
range = 8
radius = 2
damage = "1d4"
effect = { zone = { kind = "fouled", turns = 10 } }
[spell.cast]
effect = "blight"
noun = "reek"
verb = "chokes"
message = "A foul cloud bursts from your hands!"
noise = { sound = "chanting", loudness = 30 }
[spell.grows]
damage = "willpower 16 2d4, 22 2d4+2"
range = "willpower 14 9, 18 10"
```

| | |
|---|---|
| In the world | Blight's little brother that stays: 1d4 and Poisoned 1 at the cast (the type's), then Poisoned 1 more to whoever starts a turn inside for 10 turns: six turns in it is Poisoned 7 (a point every 17 s, Strength and Dexterity −1 from degree 4). The undead and the 33 poison-immune types walk through it. It does not block sight (Q8); that is Mirk's |
| Who | scroll only (item level 7, frequency 15, value 250) |
| Size | **N**: the zone mechanic's `fouled` kind |

### 17. Silence

```toml
[[spell]]
id = "silence"
name = "Silence"
description = "Silence falls where you aim it: everyone it reaches can neither cast nor read nor sing for a moment."
level = 11
school = "abjuration"
kind = "hex"
shape = "burst"
resolution = "area"
range = 8
radius = 1
effect = { status = "silenced", severity = 1 }
[spell.cast]
noise = { sound = "chanting", loudness = 10 }
[spell.grows]
range = "willpower 14 9, 18 10"
radius = "willpower 18 2"
```

| | |
|---|---|
| In the world | Silenced (`blocks = ["cast", "read", "song", "throw_voice"]`, 10 s, max 1): the 37 `caster` creatures and the 6 `summoner`s lose a turn or two of spells; the shaman cannot call. Short by design (the status's own `eases`): it buys a charge, not a fight |
| Who | priest 12 (abjuration, the priest's affinity 2), scroll (item level 6, frequency 20, value 200) |
| Size | **D** (an area hex; creatures already honour `blocks cast`: `ai.rs` 322, a blocked caster fights in melee, and `summon.rs::summons_at`) |

### 18. Bramble (entangle)

```toml
[[spell]]
id = "bramble"
name = "Bramble"
description = "Where things grow, you call them up: everyone on living ground where you aim it is caught in thorn and root a while."
level = 8
school = "transmutation"
kind = "hex"
shape = "burst"
resolution = "area"
range = 8
radius = 2
effect = { status = "webbed", severity = 2, only_terrain = "plants" }
[spell.cast]
noise = { sound = "chanting", loudness = 20 }
[spell.grows]
range = "willpower 14 9, 18 10"
degrees = "presence 14 3, 18 4"
```

| | |
|---|---|
| In the world | Webbed on everyone standing on **plant ground** in the burst, nothing on stone: `grass`, `tall_grass`, `dry_grass`, `marsh`, `reeds`, `flowers`, `buttercups`, `bluebells`, `bush`, `forest_floor`, `fungus` *(the list is the data's: a terrain flag `plants = true` in `terrain.toml`, one line each)*. Outdoors and in the caves' fungus beds it is Web at radius 2; in a flagstone hall it is nothing. The druid reads the floor |
| Who | druid 9, scroll (item level 4, frequency 30, value 130) |
| Size | **S**: `only_terrain` on an area hex's status (`cast_area_hex` skips a creature whose cell's terrain lacks the flag); the `plants` flag is data |

### 19. Mesmerise

```toml
[[spell]]
id = "mesmerise"
name = "Mesmerise"
description = "Catch the eye of a foe you see nearby: transfixed a while, all it does takes twice as long."
level = 5
school = "enchantment"
kind = "hex"
shape = "ray"
resolution = "auto"
range = 6
effect = { status = "transfixed", severity = 2 }
[spell.grows]
range = "willpower 14 7, 18 8"
degrees = "presence 14 3, 18 4"
```

| | |
|---|---|
| In the world | Transfixed (cost 200, 12 s a degree): the Transfixing stare's status for anyone (a birth sign gives the stare; the bard learns the trick). Stronger than Curse of sloth's Sluggish (150) and shorter |
| Who | bard 8, scroll (item level 3, frequency 35, value 100) |
| Size | **D** |

### 20. Slumber

```toml
[[spell]]
id = "slumber"
name = "Slumber"
description = "A foe you see nearby falls asleep where it stands, until a noise, a blow, or a cry wakes it."
level = 15
school = "enchantment"
kind = "hex"
shape = "ray"
resolution = "auto"
range = 6
effect = { status = "asleep", severity = 1 }
[spell.grows]
range = "willpower 14 7, 18 8"
```

| | |
|---|---|
| In the world | Asleep (`sleeps`: takes no turn, has seen no one, a sneak attack lands; woken by noise at its hearing threshold, a blow, or a cry): one creature out of the fight until something wakes it, and a thief's dream. No level limit, no draw: the house rule, and the cost is 15 a creature |
| Who | scroll only (item level 8, frequency 10, value 320) |
| Size | **D** (`asleep` on a hex: `spell_def` allows any known status; the AI's wake rules exist) |

### 21. Gale

```toml
[[spell]]
id = "gale"
name = "Gale"
description = "A blast of wind from you toward a place you aim at: everyone on its line is driven a cell back, fires on it are blown out, and clouds are torn away."
level = 11
school = "evocation"
kind = "hex"
shape = "line"
resolution = "area"
length = 6
effect = { push = 1, douses = true, clears = ["fouled", "dark"] }
[spell.cast]
noise = { sound = "explosion", loudness = 50 }
[spell.grows]
length = "willpower 14 7, 18 8"
```

| | |
|---|---|
| In the world | every creature on the line is knocked a cell farther along it (`knock_back` away from the caster, nearest first so no one is pushed into another; a creature that cannot be moved stays; a shut door stands, for Gale has no `blast`); every fire on the line is put out (`LevelState::fires` and a `fire` zone's cells on it); a `fouled` or `dark` zone loses the cells the line crosses. The druid clears the Reek off his bear, or blows the goblins off the bridge into the river |
| Who | druid 20, scroll (item level 6, frequency 20, value 200) |
| Size | **S**: `[spell.effect] push` (cells, 1 to 3), `douses`, `clears` on a line hex: `cast_bolt`'s target walk with `knock_back` in place of a strike; zone cells removed by `LevelState::clear_zone_cells`. No harm, no roll, no draw |

### 22. Nightsight

```toml
[[spell]]
id = "nightsight"
name = "Nightsight"
description = "Your eyes open to the dark a while: you see as a cat does."
level = 4
school = "transmutation"
kind = "buff"
shape = "self"
resolution = "auto"
effect = { status = "cat_eyed" }
```

| | |
|---|---|
| In the world | `cat_eyed` (a potion's status today: `low_light` vision, `max = 1`, `eases = 1800`), thirty minutes, so no degrees ladder: a lantern left unlit is a stealthy ranger |
| Who | ranger 9, scroll (item level 2, frequency 45, value 80) |
| Size | **D** |

### 23. Glamour

```toml
[[spell]]
id = "glamour"
name = "Glamour"
description = "A glamour lies over you a while: you seem a little to one side of where you stand, and blows meant for you miss more often."
level = 8
school = "abjuration"
kind = "buff"
shape = "self"
resolution = "auto"
effect = { status = "glamoured" }
[spell.grows]
degrees = "presence 14 2, 18 3"
```

| | |
|---|---|
| In the world | a new status `glamoured` (`max = 3`, `stacking = "refresh"`, `eases = 60`, `combat = { dodge = 3 }`): Dodge +3 for one to three minutes; nothing against an area (Evasion is the talent for that). Barkskin's cousin for the arcane |
| Who | scroll only (item level 4, frequency 30, value 130) |
| Size | **D** (a status in data) |

### 24. Fire cloak

```toml
[[spell]]
id = "fire_cloak"
name = "Fire cloak"
description = "Flames wrap you a while without burning you: whoever strikes you in melee is burnt, and you shed a little light."
level = 12
school = "abjuration"
kind = "buff"
shape = "self"
resolution = "auto"
effect = { status = "fire_cloaked" }
[spell.grows]
degrees = "presence 14 2, 18 3"
```

| | |
|---|---|
| In the world | a new status `fire_cloaked` (`max = 3`, `refresh`, `eases = 60`, `light = "ember"`, `retaliates = { damage = "1d6", damage_type = "fire" }`): a melee blow that lands on the wearer burns the striker 1d6 fire, through `spell_strikes` (so the fire type's Burning applies to the striker, one in four). The light is the price: a cloaked caster is seen |
| Who | scroll only (item level 6, frequency 20, value 200) |
| Size | **S**: `retaliates` on a status, read where Barbed hide's barbs are (`world/gifts.rs` 217–228): one clause more there, no new place |

### 25. Water walking

```toml
[[spell]]
id = "water_walking"
name = "Water walking"
description = "Water bears you a while: wading costs you no more than a step on dry land, and ice never gives way under you."
level = 5
school = "transmutation"
kind = "buff"
shape = "self"
resolution = "auto"
effect = { status = "water_borne" }
[spell.grows]
degrees = "presence 14 2, 18 3"
```

| | |
|---|---|
| In the world | a new status `water_borne` (`max = 3`, `refresh`, `eases = 120`, `wades_free = true`, `plunge_proof = true`): Undertow's two effects (the Water sign's gifts, `talents/undertow.toml`) for two to six minutes. A lake is a road; the electricity type still conducts through the water one wades (the spell carries, it does not insulate: the choice stays) |
| Who | druid 6, ranger 12, scroll (item level 3, frequency 35, value 100) |
| Size | **S**: `wades_free` and `plunge_proof` on a status, read where the talent's are (`TalentMods` → one `Actor::wades_free()` that asks both: the two call sites in `tread_price` and `after_tread` already read the talent) |

### 26. Brand

```toml
[[spell]]
id = "brand"
name = "Brand"
description = "Your held weapon burns a while: your blows do fire besides, and the blade lights your way a little."
level = 7
school = "transmutation"
kind = "buff"
shape = "self"
resolution = "auto"
effect = { status = "branded_blade" }
[spell.grows]
degrees = "presence 14 2, 18 3"
```

| | |
|---|---|
| In the world | a new status `branded_blade` (`max = 3`, `refresh`, `eases = 60`, `light = "ember"`, `blows = { damage = "1d4", damage_type = "fire" }`): the wearer's melee blows do 1d4 fire more (as a Flaming affix's blow does, through the same `elemental` path on the attack profile); they do not burn, for type rules never reach a blow (Q1). The paladin's smite-less fire, and a torch in the hand that is not a torch (the ember light) |
| Who | paladin 14, scroll (item level 4, frequency 30, value 130) |
| Size | **S**: `blows` on a status, added where the worn `elemental` affixes become riders (`magic/mods.rs`, `MagicMods::sharpen`, 233–250: one more rider of its kind). Brand's word: a brand is a burning stick and a sword (Old English *brand*, both) |

### 27. Renewal

```toml
[[spell]]
id = "renewal"
name = "Renewal"
description = "Your flesh knits fast a while: wounds close at four times the pace, awake or asleep."
level = 10
school = "restoration"
kind = "buff"
shape = "self"
resolution = "auto"
effect = { status = "renewing" }
[spell.grows]
degrees = "presence 14 2, 18 3"
```

| | |
|---|---|
| In the world | a new status `renewing` (`max = 3`, `refresh`, `eases = 120`, `regen = 400`): the normal pace is a point per max(60 s, 360 − 12 × Toughness) s (`[healing]`): at Toughness 14 a point every 192 s, under Renewal every 48 s; six minutes at three degrees is about 7 points at no further cost. Not a Cure wounds (2d6 at once for 2 mana): a buff for the long walk, and the priest's answer to Bleeding's regen 0 (the slowest status counts: Bleeding still stops it; Renewal does not cure, it waits) |
| Who | priest 9, scroll (item level 5, frequency 25, value 150) |
| Size | **D** |

### 28. Unbinding

```toml
[[spell]]
id = "unbinding"
name = "Unbinding"
description = "A word that breaks the hexes on you: the evil eye, weakness, sloth, drowsiness, webs, and fear are gone."
level = 8
school = "abjuration"
kind = "cleanse"
shape = "self"
resolution = "auto"
effect = { cures = ["ill_wished", "weakened", "sluggish", "drowsy", "webbed", "frightened", "transfixed", "pinned"] }
```

| | |
|---|---|
| In the world | the hexes' statuses (the list Malison reads, plus this design's), ended at once on oneself. Nothing on allies (a touch form is a later spell) |
| Who | paladin 20, scroll (item level 4, frequency 30, value 130) |
| Size | **D** (a cleanse as Remove disease) |

### 29. Rouse the dead

```toml
[[spell]]
id = "rouse_the_dead"
name = "Rouse the dead"
description = "A corpse you see rises as what it was and fights beside you until it falls again, or ten minutes pass. Now and then it rises against everything, you too."
level = 24
school = "necromancy"
kind = "summon"
shape = "self"
resolution = "auto"
study_difficulty = 8
effect = { rise = { most = 2, minutes = 10, hit_points = 50, berserk = 10, berserk_faction = "berserk", berserk_message = "The dead thing stands, and it knows no one." }, corpse = "consumed" }
[spell.cast]
message = "The dead thing stirs, and stands."
noise = { sound = "chanting", loudness = 30 }
[spell.grows]
degrees = "presence 14 15, 18 20, 22 30"
```

| | |
|---|---|
| In the world | aimed at a **corpse the player sees** (the corpse aim, entry 2, any stage: Q10): the corpse is used up and **its own creature type** stands on its cell, or the nearest free one (Q9): an ogre's corpse makes an ogre. One time in ten it rises **berserk**, friendly to nothing, and attacks whatever it sees, the caster included |
| How it is made | the type is the corpse object's `ObjectDef::corpse`. `travel::make_creature` on a `CreatureSpawn` with `rolls_gear = false`, through `World::make_ally` when it serves (`AllySource::Summoned`, the player's faction, orders, kill credit, the cap of `[allies] most`), directly with the `berserk` faction as its `faction_override` when it does not. One `below(100)` against `berserk`, from the gameplay stream, after every refusal has passed |
| What it keeps | everything of its type: stats, speed, natural attacks and armour, senses, tags, resistances, and `summon` table. A risen kobold is a kobold to every rule: Rebuke leaves it alone and necrotic harms it; a risen zombie is undead as any zombie. `hit_points = 50`: it stands at half its maximum (rounded up) |
| What it loses | gear and pack (`rolls_gear = false`; what it carried fell when it died), any `npc` id (no shop, no quests, no talk: those were the actor's, never the corpse's), a mimic's disguise (taken off as it is made, `travel.rs` 642). A second death leaves nothing, no corpse, pile, or loot, so nothing rises twice |
| What cannot rise | a corpse of a `unique` type: refused, no time, no mana ("It will not rise for you."). Whatever leaves no corpse never comes to this: animated objects (the generator writes `corpse = false`), bone and spirit undead, demons, devils, and any other type with `corpse = false`; the player leaves no corpse object. There is no level limit: what rises is what the caster could kill |
| Serving | `most = 2`: the third rouse sends the oldest back to the ground. Ten minutes by default, `degrees` lengthening it, `ally_mod`'s `most`, `duration_percent`, and `lasting` read as in `cast_summon` (`count` not: one corpse, one creature). The allies' cap refuses before anything is spent |
| Berserk | a new faction in `assets/data/factions.toml` (none today is hostile to all; every monster faction lists little but the player): `id = "berserk"`, `name = "the berserk"`, `hostile_to_all = true`, a new loader field in `content/faction.rs` that fills its row and column of the hostility matrix, **its own diagonal too** (friendly to nothing: two berserk risen fight each other). Inserted beside the `outraged` faction; `fixture_factions.rs` strips lines that start with `hostile`, so fixtures stand at peace with no edit. It counts toward neither `most` nor the allies' cap, takes no orders, and acts as any hostile by the plain AI (`nearest_hostile_in_sight`, as the arena's factions fight). It has the same clock: a new `Actor::risen: Option<GameTime>` (left out of saves and hashes while empty), read in `World::allies_lapse` (departing through `ally_departs`) and at the early return of `fight.rs::creature_dies`. Killing it pays as any kill of its type, for the player or their allies |
| Who | witch 28, scroll (item level 12, frequency 8, value 400) |
| Size | **N**: `effect.rise` in place of `effect.summon` on a summon spell (checked at load: `most`, `minutes`, `hit_points` 1 to 100, `berserk` 0 to 100, a known faction), the corpse aim, `hostile_to_all`, `Actor::risen`, the two arms above. No alignment deed: no spell is judged today (`Deed`, `world/alignment.rs`, has none), and this one adds none |

### 30. Grave feast

```toml
[[spell]]
id = "grave_feast"
name = "Grave feast"
description = "Draw what life is left out of a corpse beside you: hit points and mana back."
level = 6
school = "necromancy"
kind = "heal"
shape = "touch"
resolution = "touch"
effect = { heal = "2d6", corpse = "consumed", mana = "1d6" }
[spell.grows]
heal = "willpower 16 3d6, 22 4d6"
```

| | |
|---|---|
| In the world | a touch at a corpse beside the caster (the corpse aim on a touch: the cell beside with a corpse the player sees): the corpse is used up, the caster heals the roll and gains the mana roll (never past the maxima). No stage read (Q10): a rotten corpse feeds as a fresh one. It competes with eating the corpse (hunger, the corpse gifts) and with Charnel burst for the same corpse: a real choice on every kill |
| Who | witch 14, scroll (item level 3, frequency 30, value 100) |
| Size | **S**: `mana` on a heal (a roll, 1 to 500: `World::gain_mana` exists for Hex tithe) and the corpse aim on a touch |

### 31. Blink

```toml
[[spell]]
id = "blink"
name = "Blink"
description = "You are elsewhere: a few cells away in a direction you do not choose."
level = 6
school = "transmutation"
kind = "utility"
shape = "self"
resolution = "auto"
effect = { blink = { cells = 5, ends = ["held", "webbed", "pinned"] } }
[spell.cast]
message = "You blink."
noise = { sound = "chanting", loudness = 10 }
```

| | |
|---|---|
| In the world | the caster is moved to a walkable, empty cell within 5 (not their own; in line of sight or not: through a wall is the gamble), drawn evenly from those cells in cell order (one `rng.below(n)`), through `World::teleport` (the trap's: provoking and the empathy break as a teleport does). A held, webbed, or pinned caster still blinks (those statuses block `step`, not `cast`), and **the blink ends all three** (Q11): each is cured with its own end words. Out of a ring, or into the next one |
| Who | wizard 6, scroll (item level 3, frequency 35, value 100) |
| Size | **S**: `[spell.effect] blink` (`cells`, 1 to 10; `ends`, status ids checked at load) on a `utility` spell on `self` (the second utility after Identify: `SpellEffect::blink`); `World::cast_blink` collects the cells, calls `teleport`, then `Effects::cure` each of `ends` |

### 32. Sense the quick

```toml
[[spell]]
id = "sense_the_quick"
name = "Sense the quick"
description = "For a while you feel the living about you: their heartbeats, roughly placed, as you hear a footstep."
level = 5
school = "divination"
kind = "buff"
shape = "self"
resolution = "auto"
effect = { status = "quick_sensed" }
[spell.grows]
degrees = "presence 14 2, 18 3"
```

| | |
|---|---|
| In the world | a new status `quick_sensed` (`max = 3`, `refresh`, `eases = 60`, `senses = { reach = 8, kind = "heartbeat" }`): while it lasts, at the end of every turn each living creature (not undead, not animated, not a construct: by tag) within 8 of the player that they do not see is pushed to the player's heard list as a sound of kind `heartbeat` at its rough place (`noise::rough_place`, as footsteps are placed: never the exact cell, never the creature). It is a sound, so it goes through `PerceptionView::sounds()` and the sound layer (`q` in the dump), the one channel that already hides the source. The bard hears the ambush behind the door; the undead give it nothing |
| Who | bard 20 (late, because it is strong), scroll (item level 3, frequency 30, value 100) |
| Size | **S**: `senses` on a status, one loop at the end of `run_turn` pushing `Heard { kind: Heartbeat, at, loudness }`; a `SoundKind::Heartbeat` (`q` in the sound layer, "a heartbeat" in words). Perception reads only what the rule pushed: no leak beyond the design's (a rough place within 8) |

### 33. Cloudburst

```toml
[[spell]]
id = "cloudburst"
name = "Cloudburst"
description = "Under the open sky you call the rain: it falls for an hour, puts out fires, and washes the burning clean."
level = 20
school = "transmutation"
kind = "utility"
shape = "self"
resolution = "auto"
open_sky = true
effect = { weather = { kind = "rain", hours = 1 } }
[spell.cast]
message = "The sky darkens, and the rain comes."
noise = { sound = "thunder", loudness = 60 }
```

| | |
|---|---|
| In the world | the weather becomes rain for an hour of game time (the dev `weather:rain` override's code, three hours there, one here): what rain already does happens (the sky, the sounds, "rain wakes the player" in a rest, the temperature by weather, `docs/dev/sky.md`), and besides, as the rain begins, every fire on the level's outdoor cells is put out and every Burning creature under the open sky is quenched (one pass at the cast, in the same place `douses` is done). The druid's answer to the wizard's fire |
| Who | druid 30, scroll (item level 10, frequency 8, value 320) |
| Size | **S**: `[spell.effect] weather` on a utility spell (kind one of `weather.toml`'s, hours 1 to 24); `World::set_weather` is the dev action's function |

### 34. Beguile

```toml
[[spell]]
id = "beguile"
name = "Beguile"
description = "A creature you see nearby takes you for a friend a while: it fights beside you, and goes its own way when the charm ends."
level = 18
school = "enchantment"
kind = "hex"
shape = "ray"
resolution = "auto"
range = 6
effect = { charm = { minutes = 5 } }
[spell.grows]
range = "willpower 14 7, 18 8"
degrees = "presence 14 8, 18 12, 22 16"
```

| | |
|---|---|
| In the world | the creature becomes an ally of the player (`Actor::ally` with a new `AllySource::Charmed`, the stay in minutes from `degrees`, on the player's side, following, under orders, the allies' cap): its friends turn on it, it fights for the witch, and at the end it is itself again (hostile, where it stands: `World::ally_departs` puts a summon away; a charmed creature is set back to its faction instead). Refused on a unique (an npc), an animated thing, a creature with no mind (undead, constructs, vermin: by tag), or one already an ally; no level limit, no check: 18 mana and a turn, and the charm ends. Beguile an ogre at the door, send it at its band, and be elsewhere when it wakes |
| Who | witch 44, scroll (item level 9, frequency 8, value 400) |
| Size | **N**: `AllySource::Charmed { faction }` (the stay is `Ally::until`, as a summon's; `faction` is the `faction_override` to put back; saves change: an ally source variant), the end-of-stay arm that restores it, the refusals; the kill credit and orders are the allies' as they stand (`docs/dev/allies.md`). It never joins the retinue: when the player leaves the level the charm ends, and it is itself again where it stands (Q12): one arm where `follow_along` picks who comes along |

### 35. Benison

```toml
[[spell]]
id = "benison"
name = "Benison"
description = "A blessing spoken over all beside you: hit points back to you and every ally within reach; it takes a while to say."
level = 12
school = "restoration"
kind = "heal"
shape = "burst"
resolution = "area"
range = 0
radius = 1
casts = 2
effect = { heal = "2d6" }
[spell.grows]
heal = "willpower 16 3d6, 22 3d6+3"
radius = "willpower 18 2"
```

| | |
|---|---|
| In the world | a heal over a burst on the caster: the caster and every ally of theirs in the burst heal one roll each; hostiles nothing. The companion, the familiar, a serving risen creature, the charmed ogre, and the priest together. Two casts' time, as Cure wounds |
| Who | priest 24, scroll (item level 6, frequency 15, value 250) |
| Size | **S**: a heal of shape burst (`spell_def` allows `area` with `heal`; `cast_heal_area` walks `burst_cells` for allies and self, one roll shared, `spell_landed` on each healed); `range = 0` as Thunderclap's |

### 36. Rebuke

```toml
[[spell]]
id = "rebuke"
name = "Rebuke"
description = "Hold up your faith against the dead, and it burns them: every undead thing where you aim it is seared, and flees from you a while."
level = 14
school = "abjuration"
kind = "damage"
shape = "burst"
resolution = "area"
damage_type = "fire"
range = 8
radius = 2
damage = "2d6"
effect = { status = "frightened", severity = 3, only_tags = ["undead"] }
[spell.cast]
effect = "blaze"
noun = "rebuke"
verb = "sears"
message = "You hold up your faith against the dead!"
noise = { sound = "chanting", loudness = 40 }
[spell.grows]
damage = "willpower 13 3d6, 16 4d6, 19 5d6, 25 5d6+2"
range = "willpower 14 9, 18 10"
```

| | |
|---|---|
| In the world | harm and Frightened 3 to the **undead only** in the burst (`only_tags`: a creature without the tag is untouched, not even an event; an undead one is struck, friend or foe). Fire, for there is no holy type (Q2), so the fire type's rule comes with it: Burning one time in four on what it harms, ice on its cells melted, a fire left on its centre cell; an undead type's own fire resistance counts. The 93 undead types: a zombie flees, a lich flees too (no level limit) and comes back in 18 s |
| Who | priest 18, scroll (item level 7, frequency 15, value 250) |
| Size | **S**: `only_tags` on a damage spell's or area hex's targets (`spell_hurts` and `cast_area_hex` skip a creature whose type lacks every listed tag), one filter in the target walk; the same field serves Bramble's cousin and any later "the living only" spell |

## 8. Who gets what: class tables and scrolls

Every row keeps the class's existing entries and adds this design's at the levels shown; a table's level is the author's call (#977 removes caster level and `casting`). The level 1 rows are left to #979. **These spells are not class-specific** (Q13): the owner's words, "they are not really class specifics even if they kinda fit fantasy of one class". A class row is a convenient way to meet a spell, not part of what the spell is, and where an entry calls a spell a class's, that is colour, not a rule.

| Class | Today | Added | Count after (beside the cantrip) |
|---|---|---|---|
| Wizard | 1 Ray of frost, 3 Magic missile, 12 Burning hands (→ 1 by #979), 18 Arcane bolt | 6 Blink, 9 Shatter, 24 Chain lightning, 36 Sunfire | 7 |
| Priest | 1 Mend, 3 Blessing, 3 Cure wounds | 9 Renewal, 12 Silence, 18 Rebuke, 24 Benison | 6 |
| Druid | 1 Spark, 3 Barkskin, 6 Call of the wild, 12 Call lightning, 25 Call of the forest, 40 Call the elements, 50 Flame and frost | 6 Water walking, 9 Bramble, 15 Rime, 20 Gale, 30 Cloudburst | 11 |
| Witch | 1 Evil eye, 3 Hex of weakness, 12 Cause wounds, 32 Curse of sloth, 40 Blight | 8 Flay, 14 Grave feast, 18 Charnel burst, 24 Leech, 28 Rouse the dead, 36 Fear, 44 Beguile | 11 |
| Bard | 1 Mend | 8 Mesmerise, 12 Thunderclap, 16 Bind, 20 Sense the quick | 4 |
| Paladin | 6 Blessing | 14 Brand, 20 Unbinding | 3 |
| Ranger | 6 Barkskin | 9 Nightsight, 12 Water walking | 3 |

**The class test** (`world/spell_list_tests.rs`: "a full caster to a cantrip at 1 and three to six spells") is raised to **three to twelve** (Q13). The druid's and witch's eleven read as a spell every four levels, and #979's one new level 1 spell apiece still fits. #990 keeps this clause and removes the one beside it, so the raise is made after #990.

**Scrolls**: every spell has a scroll (`scroll_of_<id>`, the README's convention for level, frequency, and value: the numbers in each entry), so a wizard who finds a scroll of Rebuke may study it (abjuration, affinity 1) and a priest a scroll of Chain lightning (evocation, no affinity: the study check's overreach); anyone not trained in Spellcraft is refused (#987). The nine scroll-only spells (Balefire, Acid arrow, Grave spear, Mirk, Slick, Reek, Slumber, Glamour, Fire cloak) are what makes finding scrolls worth it: a class's table never holds them. Wands: Shatter, Acid arrow, Flay, Bind, Mesmerise, and Fear are aimed and may be wands (`[object.wand]`, `rungs = { min = 0, max = 2 }`); recommended two now, **wand of shatter** (item level 7, 9 charges) and **wand of bind** (item level 8, 7 charges), the rest later.

**The wiki**: the spell list's generated table gains a column, "Beyond its damage" (the type's rule in words from `[types]`: `xtask/src/wiki/spells.rs`), a "Spell types" section on `wiki/src/magic.md` with the table of §3 and the exact chances, the two statuses in `status-effects.md`, the zones in a "Zones" section, and Conductor in the talents' tables. The game says it in words: the menu's row (`4d6 fire, burns one in four`), the description, look mode's "burning" on a creature.

## 9. Implementation issues

Each issue names its slice of this document, its tests, and its docs; the order is the dependency order. Every spell's scroll and class row come with the issue that builds the spell.

| # | Issue | Builds | Tests | Docs |
|---|---|---|---|---|
| T1 | **Spell types: the table, fire and cold** | `[types]` in `spells.toml`, `SpellRules::types`, the `rider`/`ends`/`heat`/`frost`/`douses`/`fire` hooks, `lay_fire` shared with Blaze, the `burning` status, Numbing cold removed (Conductor comes in T2, with what it reads), the three spells' `heat`/`frost` fields dropped, the menu's and dump's words | `world/spell_type_tests.rs`: a fire spell's burn draw by seed (one in four over the rolled sample, never on a fire-immune), a burning creature's light in the perception, a step into water ending it, a cold spell numbing every target harmed and none missed (Numbing cold's test moved), Ray of frost freezing the cell it strikes, Frost dousing a fire; a replay created and verified | `docs/dev/spells.md` "Spell types", `spec/08e` "Spells", the wiki's section, `docs/balance.md` unchanged |
| T2 | **Electricity: arcs, water, Chain lightning, Conductor** | `arc_from`, `water_body`, `conducts`, `[spell.effect] arcs`, the `count` ladder on a damage spell, Chain lightning (its file, scroll, wizard 24), Conductor (its file, `TalentEffect::SpellType`, the `resist` effect) | the arc draw by seed, an arc jumping back to the caster and to an ally, the chain's certain jumps and its end, the pond (every wader struck, the caster too, with Conductor too, at 25 % less), Conductor's arcs every time, ice not water, actor-id order | the same, `docs/dev/temperature.md` (water bodies) |
| T3 | **Force, poison, necrotic, acid** | `shove`, `shatter` through `Shock::Blast`, `doors` (`World::blast_doors`), `bite`, `immune_tags`, the `corroded` status, Blight's severity 1 | a big hit shoves and a small one does not, a held creature stays, a force burst with `blast` breaks thin ice and shut doors, locked and trapped ones too, and a chest below its hardness, a Fireball's doors standing and its chests as today, Cause wounds against a creature below half, a zombie immune to necrotic in the `actors` dump, Corroded's two ticks | `docs/dev/combat.md` (resistance by tag) |
| P1 | **Spells: the data-only fourteen** | the spell files and their scrolls (§7 entries marked D), `degrees` allowed on a damage spell's status, Bleeding refusing the bloodless (Q7, Flay's), the class table rows of these spells (Flay, Rime, Sunfire, Renewal, Silence, Mesmerise, Bind, Fear, Nightsight, Unbinding) and the test's cap raised to twelve | the content tests (every spell loads, the ladders as written), Bleeding refused on a skeleton and taken on a kobold, one replay through each class's new spell, the balance probe rerun and `docs/balance.md` updated | `spec/`, `docs/dev/spells.md` "The list", the wiki |
| P2 | **The corpse aim: Charnel burst, Grave feast, Rouse the dead** | `Aim::Corpse`, `[spell.effect] corpse`, the burst at a corpse, the touch at one, `effect.rise` (entry 29), the `berserk` faction and `hostile_to_all`, `Actor::risen`, `mana` on a heal | refused with no corpse in sight, the corpse used up, the burst's kills leaving corpses for the next, an ogre's corpse rising an ogre on its cell at half its hit points with no gear, serving and departing, refused on a unique's corpse, a mimic undisguised, the berserk draw by seed attacking the caster, an ally, and another berserk, its clock, a second death leaving nothing, the feast's heal and mana capped, the dumps | `docs/dev/spells.md`, `docs/dev/allies.md`, `docs/dev/creatures.md` (the faction), `docs/dev/hunger.md` (a corpse's other uses) |
| P3 | **Zones: Balefire, Reek, Mirk, Slick** | `LevelState::zones`, the four kinds' hooks, the perception field, the look-mode words, the still tint, the dev `zone:` action, Gale's `clears` | a zone laid, saved, hashed, dropped at its end; a creature burnt at its turn's start and on a step; Poisoned per turn in the Reek; no light in the Mirk and a creature blind in it; Slick's check by seed; a snapshot of the look-mode words; one golden only if the fire zone's light cannot be checked by the `light` dump layer (it can: no golden) | `docs/dev/spells.md` "Zones", `docs/dev/rendering.md` (the tint), `docs/dev/saves.md` |
| P4 | **Small rules** | Shatter (its file, the `shatter` effect), Leech, Blink, Thunderclap's and Benison's `range = 0`, Benison's area heal, Rebuke's and Bramble's `only_tags`/`only_terrain` (the `plants` flag), Gale's push, Cloudburst's weather, Sense the quick's `senses` and `SoundKind::Heartbeat` | each rule's test in `world/spell_rule_tests.rs` (the leech capped at the maximum, the blink's draw by seed and never onto a creature, the blink out of held, webbed, and pinned ending each, the clap stunning all in it and the caster too, the heal on allies only, the rebuke's fire on an undead ally and none on the living, the bramble sparing stone, the gale's order of pushes, the rain for an hour, the heartbeat as a rough place and never for the undead) | `docs/dev/spells.md`, `docs/dev/hearing.md` (the heartbeat), `docs/dev/sky.md` (a spell's weather) |
| P5 | **Buff statuses with fields: Fire cloak, Water walking, Brand** | `retaliates`, `wades_free`/`plunge_proof`, `blows` on a status; the three spells and the three statuses | the striker burnt and the burn draw, the wade free and the plunge spared, the blow's fire on the profile and no Burning from it, each status's light in the perception | `docs/dev/status-effects.md` (the three new fields), `docs/dev/equipment.md` (a status's blows beside the affixes') |
| P6 | **Beguile** | `AllySource::Charmed`, the stay, the faction restored, the refusals, the charm ending when the player leaves (Q12) | charmed, fighting its band, ending hostile where it stands, ending when the player takes the stairs and left behind, refused on an npc and an undead, the kill credit, the `allies` dump | `docs/dev/allies.md` "Charmed", `docs/dev/saves.md` |
| W | **Wands and the wiki** | wand of shatter, wand of bind, the "Beyond its damage" column, the "Spell types" and "Zones" sections | the wands' loot tests, `cargo xtask wiki --check-links` | the wiki |

No golden screenshot is asked for: the fire zone's light and the Mirk's dark are read from the `light` dump layer, a burning creature's glow from the perception's light, and every status, menu row, and look-mode word from a text snapshot or a dump.

## 10. Decisions

The owner's answers on #982, verbatim, and what each settles. No question is left open.

| # | The question | The owner | What the design does |
|---|---|---|---|
| Q1 | Do type rules reach weapon blows? | "no, let's keep these magic specific" | Never. A Flaming sword, a Freezing arrow, and Brand's fire read no type rule |
| Q2 | A holy damage type? | "no, let's make them fire damage" | None. Rebuke does fire, with the fire type's rule |
| Q3 | Does fire spread? | "defer wildfires" | Fires and fire zones stay on fixed cells. Wildfires are on #196 |
| Q4 | Does aiming preview an arc or a body of water? | "it's just for aiming" | Read as no: the aiming display only shows the aim, and there is no preview of arcs or water bodies |
| Q5 | Does a force `blast` break a shut door? | "yes, also chests" | A force `blast` breaks shut doors, locked ones too, by the `doors` rule; chests blow apart by `World::blast` as they do for any blast below their hardness. Fireball's behaviour is unchanged |
| Q6 | Does acid eat armour? | "no" | No. Corroded's ticks are acid's number |
| Q7 | Does Bleeding refuse the bloodless? | "exactly like that" | Yes, by one clause in `afflict` for a status with a `wound` table |
| Q8 | Does the Reek block sight? | "no" | No. That is Mirk's |
| Q9 | What rises from a corpse? | "by corpses creature; give it small chance to go berserk (spawn it into faction friendly to nothing)" | The corpse's own creature type; one time in ten, data, it rises in the new `berserk` faction (§7 entry 29) |
| Q10 | Does Grave feast read the corpse's stage? | "no" | No |
| Q11 | Does Blink work while held, webbed, or pinned? | "yes and it breaks those conditions too" | Yes, and it ends all three |
| Q12 | Does a charmed creature follow across levels? | "no" | No. The charm ends when the player leaves, and it is itself again where it stands |
| Q13 | The class tables' cap? | "i really don't care. they are not really class specifics even if they kinda fit fantasy of one class" | Twelve. A spell's class row is not essential to it (§8) |
| Q14 | Conductor's self-shock clause, and protection in general | "no friendly fire protection should be implemented, sucks to suck" | Nothing here protects the caster or allies (§4 "No protection"). The owner then set Conductor as "arcs every time" plus 25 % electricity resistance, whatever the talent rules say (§5). Arcs may jump back to the caster. Thunderclap strikes its caster too |
