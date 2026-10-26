# Warlock — Special rooms, second wave: teeming dens, cursed rooms, draining rooms, forgotten altars, hot and cold rooms, and more

Design document for the owner's review (2026-10-26). Written against `origin/main` at 1c0ce120. Nothing here is implemented; §12 cuts it into issues and §13 lists the open questions. Everything builds on the existing room-effect system (`assets/data/room_effects.toml`, spec §7 "Special rooms", `docs/dev/mapgen.md`): an effect is data, a room carries it, and the rules ask one question about a cell.

## Contents

1. Summary
2. What exists today
3. Shared mechanisms (each specified once)
4. Teeming dens (the owner's "swarms")
5. Cursed rooms
6. Draining rooms
7. Forgotten altars and prayer
8. Hot and cold rooms
9. More rooms, ranked
10. What the player sees, and the wiki
11. Talents and class abilities
12. Implementation plan
13. Open questions

## 1. Summary

| Piece | Proposal |
|---|---|
| Rare whole-level versions | one field, `level_chance` (percent): when a generated level draws an effect that has one, it applies to the **whole level** instead of the room with that chance (5 for every effect that uses it). Hand-made levels may name one: `[level] effect = "cursed"`. §3.1 |
| Which effect is drawn | a `weight` per effect (default 1). With every weight 1 the draw is today's uniform pick, draw for draw. The new rooms are weighted so altars and dens are not vanishingly rare. §3.2 |
| A status while inside | `status = "chilled"`: everyone inside holds that status at least at one degree; it eases once they leave. Statuses already carry pace, ticks, mana drain, damage types, and resistances, so draining, hot, and cold rooms need no new rule of their own. §3.3 |
| Terrain in the room | `feature = "altar"`: a generator places one cell of that terrain at the room's middle. §3.4 |
| A hint from outside | `near` (a message) said once when a cell of the room first comes into the player's view. §3.5 |
| Teeming den (swarm) | a room filled with **sleeping** hostile creatures of the floor's danger, and their hoard. They wake to noise, a blow, or a sleeper's cry, so one loud step can wake the whole room. Stealth and sneak attacks matter here. §4 |
| Cursed room | a non-undead creature killed inside rises from its corpse a little later, as a random undead of its level. Taking or eating the corpse first stops it. Whole level at 5 %. §5 |
| Draining room | the status *Ebbing* drains mana while one is inside, from creatures too. Whole level at 5 %. §6 |
| Forgotten altar | **Pray** (abilities menu, a new source, *Here*; no key of its own) takes half an hour and draws a boon, nothing, or a bane from a data table. The odds lean on the altar's hidden side against the player's alignment. No limit on prayers: time is the penalty. No wish. §7 |
| Hot and cold rooms | `temperature` (degrees added to the level's) plus a status (*Sweltering*, *Chilled*): slower healing, cold makes the player hungrier, resistances to fire and cold shield one, and ice on water grows or melts by the room's temperature. §8 |
| More rooms | twelve proposals, ranked: recommended are the silent vault, tempting hoard, haunted room, shrine of a side, black chamber, scriptorium, beast den, and unstable room. Skipped for now: heavy hall, healing spring, hall of mirrors, forge, collapsed floor. §9 |
| Talents | a new system `special_rooms` with seven talents, and a new system `prayer` with two. One class ability, *Consecrate* (priest, paladin). §11 |
| Slices | seven issues in three waves, never more than three at once that add actions or save state. §12 |

## 2. What exists today

| Part | Today (`docs/dev/mapgen.md`, spec §7) |
|---|---|
| Fields | `id`, `name`, `mark`, `enter`, `light`, `glow`, `noise`, `regen`, `mana_regen`, `shunned`, `floor`; at most 32 effects; ids are indices (in the save fingerprint) |
| Effects | shadowed room, echoing hall, hushed chamber, sanctuary, flooded room |
| Generation | `mapgen/special.rs`, last, from the level's map stream: each room except the arrival room and vaults (each of eight cave chambers) is special with `chance` 7 % until `per_level` 1; the effect is a **uniform** pick; about 40 % of generated levels have one. Branches override `chance`, `per_level`, `effects` (`RoomOdds`) |
| State | `LevelState::rooms` (effect, box, a cave blob's cells, entered), saved and hashed |
| The one question | `LevelState::room_effect_at(pos)` / `World::room_effect_at`, read by perception light and creature sight (together), noise, `World::healing_rate`, `World::mana_rate`, and `World::shuns` |
| Player | the `enter` line once per room; look mode names a room the player has been in; numbers never shown |
| Nearby systems this design reuses | statuses (`data/status.toml`: `tick`, `damage`, `damage_type`, `mana_damage`, `regen`, `mana_regen`, `cost`, `blocks`, `seen`); temperature per level (`WorldState::temperature_on`) and ice (`LevelState::advance_ice`, 10-minute ticks); spawning (`world/spawn.rs`: `make_creature`, the danger draw `content/danger.rs`); `World::unaware` (sneak attacks); alignment `Deed`s and `shift_alignment`; the `altar` terrain (`_`, steady glow); the cure id `prayer` already on every disease; the teleport trap |
| What nothing does yet | nothing feels the cold (`docs/dev/temperature.md`); creatures never sleep; there is no "rise from a corpse" |

## 3. Shared mechanisms (each specified once)

### 3.1 `level_chance`: the rare whole-level version

| | |
|---|---|
| Field | `level_chance` (percent, 0 to 100, default 0) on an `[[effect]]` |
| Rule | when the generator has drawn this effect for a room, it draws once more from the **map stream**: with `level_chance` the effect **pervades the level** and the room stays plain. The extra draw happens only for effects that have a `level_chance` above 0. |
| State | `LevelState::pervading: Option<RoomEffectId>` (saved and hashed, left out while `None`, so other levels hash as before) |
| The one question | `room_effect_at` becomes `effects_at(pos) -> Here` (the room's effect, then the pervading one). Every reader folds both the same way: percents multiply, flags OR, statuses both apply. Inside the core a level has at most two effects at a cell, so the cost stays the same. The light invariant test (`special_rooms_keep_the_light_invariants`) gains a pervading case. |
| Hand-made levels | `effect = "cursed"` in a level file's header (refused on the overworld and in local areas) |
| Player | the effect's `pervade` message on arrival, said each time the player arrives (as a level feeling, not once per run); look mode does not name the whole level, since the status line and the arrival line already say it |
| Uses it (5 %) | cursed (§5), draining (§6). Possible, not proposed: shadowed, hushed, echoing (open question 2). Never: sanctuary (a level hostiles shun is broken), flooded and every `floor`/`feature` effect, teeming dens, altars, hot and cold (a hot level is just its `temperature`) |
| Odds | with §3.2's weights about 0.15 % of generated floors are cursed throughout and as many drained: a story a player tells, not a regular feature. The wiki states it. |

### 3.2 `weight`: which effect is drawn

`weight` (1 to 100, default 1) on an effect. The pick becomes `range(0, total)` over the branch's `effects`. **With every weight 1 that is exactly today's `index(len)`**, so a table without weights draws as before. Adding effects to the shared table changes which effect existing seeds draw, and the Depths' snapshots change once, in R1 (reviewed, accepted). Proposed weights:

| Effect | Weight | Share | Floors with one (at ~40 % special) |
|---|---|---|---|
| shadowed | 10 | 13 % | 5 % |
| echoing, hushed, flooded, teeming, altar | 8 each | 10 % each | 4 % each |
| sanctuary, cursed, draining | 6 each | 8 % each | 3 % each |
| hot, cold | 5 each | 6 % each | 2.5 % each |
| *(total 78)* | | | |

The Ascmearc's 26 floors would then hold about one altar and one den per game. A branch may still list its own `effects` (Fennmynster: cursed and draining are natural there).

### 3.3 `status`: a status while inside

| | |
|---|---|
| Field | `status` (a status id) on an effect |
| Rule | after every action of an actor whose cell has the effect (the player's in `World::apply`, a creature's at the end of `creature_turn`, and each step of a rest or a long action), `World::dwell(id)` gives it the status at one degree with `refresh` stacking. Once the actor leaves, the status eases by its own `eases`. Its `eases` must be at least the longest single action (10 s) so it never lapses mid-rest; validated. |
| Resistance | a status with a `damage_type` is never given to one immune, and is shrunk for one resistant (existing rule). So *Chilled* is `cold`, *Sweltering* is `fire`, and the existing resistances, talents (Flame-hardened, Winter-born's line), and affixes (of Warmth) count without new code. |
| Who | every actor inside: creatures too (a kobold shaman's mana ebbs, a troll heals slowly) |
| One new status field | `hunger` (percent of the burn pace, 50 to 300, default 100; several multiply), read once in `World::digest`. Used by *Chilled*. |

### 3.4 `feature`: one cell of terrain

`feature` (a terrain id; stone, not walkable) is placed by the generator at the free floor cell nearest the room's centre (a cave: the blob's seed cell), and never in a doorway or on a path between two doors (the generators' connectivity check runs after). A hand-made level draws its own. Used by the altar (§7), later the scriptorium's shelves and the haunted room's bones (§9).

### 3.5 `near`: a hint from outside

`near` (a message) is said once per room, the first time any of its cells comes into the player's view, before they set foot in it. `LevelState::rooms` gains `glimpsed: bool` beside `entered` (saved, hashed). Perception then knows the room as "glimpsed" but does not name it; look mode names it only once entered, as today (Sense of place changes that, §11). Optional on every effect. Proposed for dens, cursed rooms, and altars; the five existing effects get none.

### 3.6 Eligibility

| Field | Meaning |
|---|---|
| `needs_danger` (bool) | drawn only on a level with a danger (`World::danger` > 0); hand-made levels without one never get it. Teeming dens, cursed rooms, haunted rooms |
| `min_cells` (default 0) | eligible only for a room (blob) of at least this many floor cells |

The pick runs over the eligible effects only, so on a level where none are excluded the draw is unchanged.

### 3.7 Determinism

| Draw | Stream |
|---|---|
| room chance, effect pick, `level_chance`, the altar's side, `feature` placement | the level's **map** stream (as today, at the end) |
| a den's sleepers and hoard | new `swarm/<level id>` (`Rng::swarm`), so the population, loot, and gear streams do not move |
| a sleeper's gear | `gear/<level id>`, after the population's, as `make_creature` does |
| what a corpse rises as, a prayer's outcome | the gameplay stream (they happen in play) |
| waking, dwelling, rising times | none: thresholds and game time |

## 4. Teeming dens (the owner's "swarms")

**Name**: id `teeming`, name **teeming den**. The word swarm reads as insects, and the room is mostly sleepers of every kind.

**Messages** (no ADOM wording, and no sense of dread before the room is seen): `near` *"A warm, rank smell drifts out of the dark ahead."* `enter` *"You hear slow breathing all around you."* The room is often dark, so the player may hear it before seeing it. Sixth sense (an existing talent) adds *"Something sleeps on this level."* on arrival.

| Rule | Proposal |
|---|---|
| Sleepers | a new status **`asleep`** (`max = 1`, no `eases`, `seen = "asleep"`, `sleeps = true`): an actor with a `sleeps` status takes no turn (one check at the top of `creature_turn`, beside the coarse-turn check) and is `World::unaware` of everyone. Look mode says "a goblin (hostile, asleep)". A status keeps it saved, hashed, and shown with no new state, and later sleep spells can use the same status. |
| How many | one sleeper per 4 floor cells of the room, 4 to 16 (`swarm.per_cells`, `swarm.min`, `swarm.max`), drawn by the floor's danger from the branch's `population` (`content/danger.rs::draw`, the out-of-depth chance included), hostile types only, never uniques, from `swarm/<level id>`; placed on free floor in the room, made by `make_creature` with gear rolled, asleep |
| Waking | **noise**: in `World::hear` (where a creature's `Heard` is pushed), a sleeper wakes when the loudness at its cell is at least its hearing threshold × `swarm.wake` (200 %): a sneaking step does not wake it, a fight or a door kicked open beside it does. **A blow** of any kind wakes the one struck. **A cry**: a sleeper that wakes makes an `Act::Voice` noise of `swarm.cry` (60) at once, which may wake the next: a cascade, deterministic, through the existing propagation. **Light and sight do not wake** (a lantern carried in is fine). No timer: they sleep until disturbed. |
| The hoard | one item per 4 sleepers (at least 1) at the floor's item level + 1, `loot::make`, on the room's floor, from `swarm/<level id>` after the sleepers |
| Versus the normal population | sleepers come on top of the 35 per 1 000 cells. They **count** toward the spawning cap (`attempt` counts hostiles on the level), so a floor with a den spawns nothing until enough of it is dead. Nothing else changes. |
| Stealth | step noise already falls with Stealth (`noise.rs`), so a stealthy player crosses a den that a clanking one wakes. Sleepers are `unaware`, so **sneak attacks** land on each (Sneak attack, Ambusher). A kill is a noise too (`Act::Attack`): killing quietly in a den is the thief's game. |
| Alignment | none. Killing a sleeping hostile is a hostile kill (`kill_hostile`), as now. A good character murdering sleepers is a story, not a rule (open question 4). |
| Shunned, patrols | a den is never where a creature patrols (as for a shunned room); sleepers have no post to return to until woken, after which the normal AI applies |
| Fields | `swarm = { per_cells = 4, min = 4, max = 16, wake = 200, cry = 60, loot_level = 1 }`, `needs_danger = true`, `min_cells = 16` |
| Cost | M. New: the `sleeps` status field and its one check, the wake hook in `hear`, the den filler in `mapgen/special.rs`, `Rng::swarm`. The risk is a den of 16 at danger 8 on a 120×60 cave: `bench-turns`' `delf` row gets a fixture den, and sleepers cost nothing per turn. |

```toml
[[effect]]
id = "teeming"
name = "teeming den"
mark = "t"
near = "A warm, rank smell drifts out of the dark ahead."
enter = "You hear slow breathing all around you."
weight = 8
needs_danger = true
min_cells = 16
swarm = { per_cells = 4, min = 4, max = 16, wake = 200, cry = 60, loot_level = 1 }
```

## 5. Cursed rooms

**Messages**: `near` *"A cold draught smells of the grave."* `enter` *"The air is cold and the dead are near."* `pervade` *"The dead do not rest on this level."*

| Rule | Proposal |
|---|---|
| Who rises | a creature that dies on a cursed cell, whose faction is not `rise.faction` (`undead`), and that leaves a corpse (`corpse = true`). Anyone's kill counts, a creature's included. Never the player. |
| As what | an undead type (faction `undead`, not unique, no npc) of the **same level** as the dead, drawn uniformly from the gameplay stream at death; if none has that level, the **nearest** level (a tie goes to the lower). Today's undead are skeleton and zombie (3), barrow ghoul (4), drowned (5), ghost (8), and ash wraith (15), so a rat (1) rises as a skeleton or a zombie and a troll as a ghost; more undead make the match closer (open question 17). |
| When | `rise.after` (2 minutes of game time, 20 normal turns) later. `LevelState::risings: Vec<Rising { at, pos, from: CreatureId, into: CreatureId }>` (saved, hashed, left out while empty); `rise_tick` runs once at the end of `run_turn`, beside `spawn_tick`. A frozen level (the player away) rises when the player returns, as spawning catches up. |
| From its corpse | at `at`, if a pile at `pos` still holds a corpse of `from`, one is taken and the undead stands on `pos` (or the nearest free cell, the spawner's helper). **A corpse picked up, eaten, or destroyed does not rise**: that is the counterplay. A corpse carried out of the room and dropped is not cursed again. |
| Warnings | at death, if seen: *"The goblin's corpse twitches."* At rising: *"The goblin's corpse rises as a zombie!"*, or a noise (`Act::Voice`, a groan) if unseen |
| XP, loot | a risen undead gives its own kill XP (no anti-farming limit) and carries nothing; what the dead carried already lay on the floor |
| Whole level (5 %) | every cell is cursed: a whole floor of risings. With a teeming den elsewhere on the level, that is the floor players remember. |
| Fields | `rise = { faction = "undead", after = "2m" }`, `needs_danger = true`, `level_chance = 5` |
| Cost | M. New: `rise` parse, `risings` state, one line in `fight.rs::dies`, `rise_tick`. Risk: a rising during a rest, since it is a creature arriving in view and the rest stops as usual. |

## 6. Draining rooms

**Messages**: `enter` *"You feel your power seeping out of you."* `pervade` *"Something here drinks at your power."* Glow: a faint cold violet (`[10, 6, 24, 8]`), a visual cue.

| Rule | Proposal |
|---|---|
| Status | **`ebbing`** (name *Ebbing*, `seen = "ebbing"`, `max = 1`, `stacking = "refresh"`, `tick = 20`, `mana_damage = 1`, `mana_regen = 0`, `eases = 30`, start *"Your power ebbs away."*, end *"Your power stops ebbing."*), given while inside (§3.3). It drains 1 mana per 20 s (180 an hour) and stops regeneration, never below 0 and never a death (existing `mana::drain`). |
| Who | everyone with mana: a wizard and a kobold shaman alike. A casters' fight in a draining room is a melee. |
| Whole level (5 %) | `level_chance = 5`: a floor where casters must ration and leave quickly |
| Fields | `status = "ebbing"`, `level_chance = 5`, `glow` |
| Cost | S once §3.3 exists (no new rule: the status framework already drains). |

## 7. Forgotten altars and prayer

**Messages**: `near` *"A faint glow rises from an altar ahead."* `enter` *"An old altar stands here, its god long forgotten."* Look mode on the altar cell: "a forgotten altar".

### 7.1 The altar

| | |
|---|---|
| Room | `feature = "altar"` (the existing terrain), `prayer = "forgotten"` (the prayer table, §7.3), `weight = 8`, `min_cells = 9` |
| Its side | drawn at generation from the map stream: good, neutral, or evil (1 : 2 : 1), kept in the room's state (`Room::side: Option<Alignment>`, saved and hashed only when set). A hand-made level gives it: `[[room]] effect = "altar" side = "evil"`. |
| What shows | the altar's **light** by side: `altar_good` (warm gold, today's `altar_glow`), `altar_neutral` (pale grey), `altar_evil` (dull red). The terrain is the same `altar` with its light chosen per cell (a terrain light override in `LevelState`, the same mechanism a placed light uses). A player who learns the colours reads the side; the game never names it. |

### 7.2 Praying

| | |
|---|---|
| How | **Pray** is an ability in the one abilities menu, under a new source **Here** (`Source::Place`: abilities a place offers, listed while they apply). No key of its own (`KeyCommand` unbound by default); bindable like every ability. It is listed while the player stands on or next to an altar cell of a room with `prayer`. Otherwise it is absent, not greyed out. |
| Action | `Action::Pray` (`pray` in action text). A long action of `prayer.minutes` (30) through `World::keep_at`, the rest's machinery: anything that disturbs a rest interrupts it (a creature comes into view, a hit), and an interrupted prayer gives **nothing** (*"Your prayer is broken."*) but the time is spent. Each minute is a `pray` action (`data/actions.toml`, 10000). |
| Outcome | at the end, one draw from the gameplay stream on the table's weights, adjusted by side (§7.3), then the outcome's effect through its existing hook, and its message |
| Alignment | `Deed::Prayed(side)`: praying at a good altar `+5`, at an evil one `−10`, at a neutral one nothing (`[shift] prayed_good`, `prayed_evil` in `data/alignment.toml`). A paladin who prays at red altars strays. |
| Repeats | **no limit**: pray again at once. The penalty is time: half an hour of hunger, and on a floor with a danger about one new creature an hour. A player who prays forty times spends a day and fights what came. The altar never crumbles, never grows angry, and never refuses (the owner's no-anti-farming rule). |
| No wish | no outcome grants a wish; wishes stay on #196 |

### 7.3 The table: `data/prayers.toml`

```toml
[forgotten]
minutes = 30
# Weights of the three kinds, before the side's lean.
boon = 40
nothing = 25
bane = 35
# Moved between boon and bane when the altar's side and the player's band agree or oppose
# (a neutral altar, or a player without alignment rules: no lean).
lean = 15

[[forgotten.outcome]]
kind = "boon"
id = "mended"
weight = 10
effect = { heal = "full", cure = ["bleeding", "poisoned"] }
message = "Warmth runs through you, and your wounds close."
```

Agreement means the band's side (good, neutral, evil) equals the altar's. With agreement boon is 55 and bane 20; with opposition boon is 25 and bane 50. Each outcome's `effect` is one of a closed set of kinds, each calling an existing hook:

| Kind | Outcome | Weight in its kind | Hook (existing) | Message (vague) |
|---|---|---|---|---|
| boon | Mended: full hit points, bleeding and poison cured | 10 | `heal`, status cure | "Warmth runs through you, and your wounds close." |
| boon | Replenished: mana to full | 6 | mana | "Your mind feels clear and full." |
| boon | Cleansed: every disease cured (cure id `prayer`) | 6 | `Diseases::cure` | "A sickness leaves you." (else "You feel clean.") |
| boon | Restored: all stat damage mended | 6 | the temple's mending service code | "You feel whole again." |
| boon | Blessed: the `blessed` status, 3 degrees | 8 | statuses | "You feel watched over." |
| boon | Offering: an item at item level + 3 on the altar | 6 | `loot::make` | "Something lies on the altar that was not there before." |
| boon | Insight: everything carried identified | 4 | identify | "You understand what you carry." |
| boon | Favour: a fifth of the XP to the next level | 3 | `earn` | "You feel wiser." |
| boon | **Gift of the forgotten** (great boon): +1 to a random stat's score, lifting its potential | 1 | `Stats::train` | "You feel changed." |
| nothing | | (one line) | | "Silence answers you." |
| bane | Sickened: a random disease (one cured by `prayer`) | 8 | `infect` | "A chill settles in your bones." |
| bane | Withered: 1d2 stat damage to a random stat | 8 | stat damage | "Something is taken from you." |
| bane | Hexed: `ill_wished`, 3 degrees | 8 | statuses | "You feel watched with malice." |
| bane | Emptied: mana to 0 | 5 | mana | "Your mind goes dark." |
| bane | Famished: nutrition to Hungry (no lower) | 5 | hunger | "A gnawing hunger takes you." |
| bane | Answered: a hostile creature of danger + 2 beside the altar (danger 0: the level's deepest population type, else none) | 6 | `make_creature` | "Something answers your prayer." |
| bane | Tithe: one unequipped stack carried vanishes | 4 | inventory | "Your pack feels lighter." |
| bane | **Wrath of the forgotten** (great bane): −1 to a random stat's potential (never below 3) | 1 | `Stats` | "You feel lessened, forever." |

The table is data: the owner can tune every weight in review. The wiki lists it exactly, with the lean (open question 7 on the jackpots).

### 7.4 Also prayable?

The temple of Cēaptūn sells services and is not an altar room. The showcase's water-garden altar is scenery. **Only rooms with `prayer` take prayers**; a hand-made level makes one by marking the room (`[[room]] effect = "altar"`).

## 8. Hot and cold rooms

| | Hot room | Cold room |
|---|---|---|
| id, name | `hot`, "sweltering chamber" | `cold`, "frigid hall" |
| enter | "A wave of heat meets you." | "Your breath mists in the bitter cold." |
| glow | faint red `[30, 10, 0, 0]` | faint blue `[0, 8, 22, 0]` |
| `temperature` | +25 °C over the level's | −25 °C under the level's |
| `status` | **Sweltering** (`damage_type = "fire"`, `regen = 50`, `mana_regen = 75`, `eases = 120`) | **Chilled** (`damage_type = "cold"`, `regen = 50`, `hunger = 150`, `eases = 120`) |
| Resisted by | fire resistance (Flame-hardened, a fire salamander's nature) | cold resistance (of Warmth, Winter-born's line where it resists) |

| Rule | Proposal |
|---|---|
| The temperature | new field `temperature` (−40 to 40 °C, default 0) added to the level's. `WorldState::temperature_on(level)` gains `temperature_at(level, pos)` (the level's plus `effects_at(pos)`'s), the one place a cell's temperature is decided. Readers: **perception** (the status bar shows the player's cell, so stepping in shows the change, the effect's only number, which the game already shows), and **ice**: `advance_ice` groups cells by (water, thickness, room), so a cold room's water freezes on a 10 °C level and a hot room's ice melts. A generated cold room in a cave with water is a frozen pool; Winter-born's footing applies. |
| What feels it | the status, not the degrees: §3.3. Healing is slowed (rests take twice as long), mana slowed in heat, hunger faster in cold. Nothing deals damage (open question 9: a *furnace* and an *ice tomb* that hurt with a tick). |
| Resting | allowed. Resting inside heals at the slowed pace, so a cold room is a bad camp, which is the point. |
| Food | rot does not read temperature (`crate::rot` is pure in age); not proposed here (open question 10). |
| Creatures | everyone inside gets the status; a cold-blooded type has no rule yet |
| Not whole-level | a hot or cold level is a `temperature` in its file, and needs no `level_chance` |
| Cost | S once §3.3 exists, plus the per-cell temperature and the ice grouping (S–M). Risk: the ice tick's per-group cost; a test checks a year's catch-up still runs in its budget. |

## 9. More rooms, ranked

Ranked by fun per cost. "Recommend" means build it after §4–§8; "skip" means not now, with the reason.

| Rank | Room | Rules | Why it is fun | Cost, risks | Verdict |
|---|---|---|---|---|---|
| 1 | **Silent vault** | `noise = 1`, `status = "silenced"` (`blocks = ["cast", "read", "throw_voice"]`; casters' spells and scrolls refused inside, creatures' too) | turns a fight upside down: the wizard and the kobold shaman fight with sticks; a thief's paradise (nothing is heard) | S (§3.3). Risk: a caster cornered inside; they can walk out. | **recommend** |
| 2 | **Tempting hoard** | `hoard = { items = 3, loot_level = 3 }` on the floor; the first item lifted **seals the room**: every door locks (`lock` of the floor's level, pickable, kickable), and `guardians` (2 to 4 hostiles of danger + 1) appear at the room's edges, out of view if they can be | the classic greed check; doors, locks, kicking, and Pick lock all come into play | M: a trigger in `pick_up` (one line), door locking through `doors.rs`, spawn through `make_creature`. Risk: a cave blob has no doors (caves then get only the guardians). | **recommend** |
| 3 | **Haunted room** | `needs_danger`; while the player **rests** inside, every hour of rest (or once at dusk) an undead of the floor's danger appears inside, out of view if it can be; `near` "You hear a whisper, and then nothing." | a fine camp spot that is not; a reason the cursed system's undead appear without kills | S–M (spawn machinery). Risk: overlaps the cursed room's theme; distinct in rule. | **recommend** |
| 4 | **Shrine of a side** | `requires_alignment = { min = "good" }` (or evil): for one whose band qualifies it is a sanctuary (`regen = 200`); for one opposed, the status *Unwelcome* (`regen = 50`, `combat = { to_hit = -1 }`); a light by side as altars | alignment gets a place on the map; cheap reward for playing a side | S: `requires_alignment` exists (`Requirement`); one fold in `effects_at`. | **recommend** |
| 5 | **Black chamber** | `light = 0` (carried light too), no glow | pure data on today's fields; terror with darkvision as the counter | none (data only). Risk: none. | **recommend** (with R1) |
| 6 | **Scriptorium** | `feature = "shelves"` (a new stone terrain), floor loot of 2 to 4 **scrolls** (`loot = { class = "scroll", count = [2, 4] }`); reading inside trains Decipher script twice as often | a reason to explore for casters and readers; ties into scrolls | S: loot class override in the special-room step (`loot/<level>` stream after the room's other loot). | **recommend** |
| 7 | **Beast den** | a teeming den whose sleepers are `animal` faction only (`swarm.factions = ["animal"]`), hoard of bones and fresh corpses (food) | the ranger's and Beast-friend's room; food in the deep | S on top of §4 (data and a faction filter). | **recommend** |
| 8 | **Unstable room** | every step inside has `warp` (10 %) of the teleport trap's effect (`world/trap.rs`'s teleport, a gameplay draw); `enter` "The floor shimmers under your feet." | chaos; fleeing through it is a gamble | S: reuse the teleport. Risk: frustrating if large; `max_cells`. | **recommend (low)** |
| 9 | Heavy hall | `burden` percent (50: limits halved inside, the one burden hook) | heavy armour suddenly Burdened | S. Fun is thin: a slower walk. | skip |
| 10 | Healing spring | a fountain: Drink (Here source) heals, with a chance of a disease | overlaps the sanctuary and the altar's Mended | S | skip (fold into the altar's table) |
| 11 | Hall of mirrors | reflections: missiles may strike a reflection | needs illusion rules | L | skip |
| 12 | Forge, collapsed floor | an anvil to improve gear; a floor that drops one a level | need crafting, and falling between levels (neither exists) | L each | skip (deferred list #196) |

## 10. What the player sees, and the wiki

| Where | What |
|---|---|
| Messages | `near` once per room when first seen; `enter` once per room when first entered; `pervade` on every arrival on a pervaded level; the dens', curses', and altars' lines in §4–§7; the statuses' start and end |
| Status bar | the statuses (*Ebbing*, *Chilled*, *Sweltering*, *Silenced*, *Unwelcome*) and the temperature of the player's cell |
| Look mode | "in a teeming den" (once entered); "a goblin (hostile, asleep)"; "a forgotten altar" |
| The abilities menu | *Here*: Pray, listed only at an altar |
| Never shown | weights, chances, wake thresholds, the altar's side in words, rise timers, which undead will rise |
| Dumps | `rooms` (dev): each room's effect, its side for an altar, `glimpsed`/`entered`, and `pervading <id>` in the header; `actors` (dev): `asleep`, the pending risings with their times; `abilities` lists Pray; JSON: `room` as today, `pervading` (only once arrived, as the player knows it) |
| Action text | `pray` |
| Wiki | a **Special rooms** page (`wiki/src/`, by topic beside traps) with a generated table (`xtask/src/wiki/rooms.rs`): every effect with what it does in exact numbers, its weight, its share, its whole-level chance, and the per-floor odds; the prayer table with weights, the lean, and the side colours; the den numbers; the risings table (which undead each level becomes) |

## 11. Talents and class abilities

Two new systems join `data/talents.toml`, placed by topic (after `danger`, before `traps`).

| System | Talent (tree) | Effect | Hook (one read) |
|---|---|---|---|
| `special_rooms` | **Sense of place** (wayfarer, free) | a room's nature is known when glimpsed (look names it before entering); a pervading effect's line says what it is | `PerceptionView::room` (glimpsed counts as entered) |
| `special_rooms` | **Hallowed rest** (grit) | where a room quickens healing or mana, it quickens them half again more for you | `World::healing_rate`, `World::mana_rate`: the room factor |
| `special_rooms`, `mana` | **Anchored mind** (scholar) | Ebbing never takes you | `World::dwell` (immune to that status) |
| `special_rooms`, `stealth` | **Hush-foot** (shadow) | sleepers need twice the noise to wake from what you do | `World::hear`'s wake check (`swarm.wake` × 2 for the player's noises) |
| `special_rooms` | **Grave warden** (guard) | what you kill never rises | `fight.rs::dies`' rise line |
| `special_rooms` | **Hardy traveller** (grit) | heat and cold never take hold of you: Sweltering and Chilled are not given to you | `World::dwell` (immune to the statuses of rooms with a `temperature`) |
| `special_rooms` | **Hoard-wise** (delver) | a tempting hoard seals only after the second item lifted | the hoard trigger (with §9 rank 2) |
| `prayer` | **Devout** (wayfarer) | the lean counts double at agreeing altars, and an opposed one leans as neutral | the prayer draw's lean |
| `prayer` | **Penitent** (grit) | a great bane becomes nothing | the prayer draw's outcome |
| class ability (`special_rooms`) | **Consecrate** (priest 9, paladin 12; `requires_alignment = { min = "good" }`; mana 15) | inside a cursed, draining, or haunted room, ends its effect for good (the room becomes plain); on a pervaded level, ends the pervading effect | `LevelState::rooms`/`pervading`; `Deed::Consecrated` +20 |

`special_rooms` ships in R1 with its first two (Sense of place, Hallowed rest: both on today's hooks), so the bar holds from the start. The rest come with their slices. `prayer` ships with R5 and its two. `min_class` is 0, so Consecrate is not required by the bar; it is proposed because the priest needs place-bound power (open question 12).

## 12. Implementation plan

Each item is one issue, one branch, one PR, updating the spec (§7 "Special rooms", the statuses), `docs/dev/mapgen.md` (or a new `docs/dev/special-rooms.md`, open question 13), and the wiki in the same PR. "Save" means it adds saved state; "act" means it adds an action. Sizes: S about a day, M two to three.

| # | Issue title | Sections | Adds | Depends on | Size |
|---|---|---|---|---|---|
| **R1** | Special rooms: effect weights, the whole-level version (`level_chance`, `pervading`, `effect` in level files), `effects_at`, `near` hints, eligibility; the `special_rooms` system with Sense of place and Hallowed rest; the black chamber | §3.1, §3.2, §3.5, §3.6, §9 rank 5, §11 | save | — | M |
| **R2** | Special rooms: statuses while inside (`status`, `World::dwell`, the status `hunger` field); the draining room and Anchored mind | §3.3, §6 | — | R1 | S |
| **R3** | Hot and cold rooms: per-cell temperature, ice by room, Sweltering and Chilled, Hardy traveller | §8 | — | R2 | M |
| **R4** | Teeming dens: the `asleep` status and `sleeps`, waking by noise, blow, and cry, the den filler and hoard (`Rng::swarm`), Hush-foot; the beast den | §4, §9 rank 7 | — | R1 | M |
| **R5** | Forgotten altars and prayer: `feature`, the altar's side and light, `Source::Place`, `Action::Pray`, `data/prayers.toml`, `Deed::Prayed`; the `prayer` system with Devout and Penitent | §3.4, §7 | save, act | R1 | M |
| **R6** | Cursed rooms: risings (`LevelState::risings`, `rise_tick`), Grave warden; Consecrate | §5, §11 | save, act (Consecrate) | R1; class tables for Consecrate | M |
| **R7** | More rooms (each its own small issue once approved): silent vault (R2), shrine of a side (R1), scriptorium (R5's `feature`), haunted room (R6's undead pick), unstable room, tempting hoard with Hoard-wise | §9 | per room | as listed | S–M each |

**Waves** (at most about three PRs at once that add actions or save state):

1. **R1** alone (it changes the special-room pick and the Depths' snapshots; nothing should race it).
2. **R2, R4, R5** together. R5 is the only one with save state and an action; R2 and R4 touch statuses (`data/status.toml` is a hot file: each adds its entries by group).
3. **R3, R6**, then the R7 rooms in pairs.

**Tests each item carries**: `content/room_effect.rs` (new fields parse and validate; a weight-1 table picks exactly as `index` did, checked against a fixed seed list); `mapgen/special_tests.rs` (the shares over 1 200 levels within bounds; `level_chance` near 5 % over many draws; eligibility; a den's count and hoard; an altar's feature placed and the level still connected); `world/rooms_tests.rs` (`effects_at` folding; `dwell` gives and eases; resistances shield; Ebbing drains a creature; ice freezes in a cold room and not beside it; temperature on the status bar); `world/den_tests.rs` (a sneaking step wakes nobody, a kicked door wakes the near ones and the cascade the rest, sneak attacks land on sleepers, sleepers count toward the spawn cap); `world/curse_tests.rs` (rises after two minutes, not if the corpse is eaten or carried, the level pick and its fallback, Grave warden, a pervaded level, frozen floors catch up); `world/prayer_tests.rs` (Pray listed only at an altar, an interrupted prayer gives nothing, the lean by side and band, each outcome's hook, the alignment deed, a hundred prayers in a row are allowed); screen snapshots for the abilities menu's *Here* group and look mode's "asleep"; a replay per action (`replay --create`, `--verify`); `cargo xtask wiki`. No golden: the glows are light, and the light tests and a `--capture` checked by eye cover them.

## 13. Open questions

| # | Question | Recommendation |
|---|---|---|
| 1 | Is 5 % the right `level_chance`, given it is 5 % *of the floors that drew the effect* (about 0.15 % of floors)? | Keep 5 % as the owner said; the rarity is the point. Raise it per branch later if a cursed dungeon is wanted (Fennmynster could list `level_chance` overrides). |
| 2 | Should shadowed, hushed, and echoing also get a whole-level version? | **No for now**: a whole dark or loud floor is a level feeling, not a room; cheap to add later as data. |
| 3 | The den's name and lines ("teeming den", "A warm, rank smell…", "You hear slow breathing all around you.") | Use them; the owner may rename freely (data). |
| 4 | Should killing a sleeping creature move alignment (a good character murdering sleepers)? | **No**: hostiles are hostiles, and it would punish the thief's whole style. |
| 5 | Should sleepers ever wake on their own (a timer, or at the floor's spawn ticks)? | **No**: the tension is that the player chooses when. |
| 6 | Prayer length: 30 minutes? | **30 minutes**: long enough that farming costs food and spawns, short enough to try twice. |
| 7 | Keep the two jackpots (a permanent +1 stat, a permanent −1 potential) at 1 in ~45 prayers each? | **Keep**: the owner asked for degenerate gambling, and the jackpots are why players kneel a twentieth time. |
| 8 | Should the altar's side show through its light colour, or stay hidden until a prayer's outcome? | **Light colour**: readable for players who learn it, never named; the wiki says it. |
| 9 | Should hot and cold rooms ever deal damage (a furnace, an ice tomb, as rarer variants)? | **Later, as separate effects** using a status `tick`/`damage`; not in R3. |
| 10 | Should a cold room slow rot (a larder) and a hot room speed it? | **No**: rot is pure in age today and food moves; per-place rot is costly for little play. |
| 11 | Do creatures suffer room statuses (Ebbing, Chilled), or only the player? | **Everyone**: one rule, and a draining room that also silences the shaman is fair and fun. |
| 12 | Consecrate as the first place-bound class ability (priest 9, paladin 12)? | **Yes**, in R6; it gives the priest something no other class has. |
| 13 | Keep the special-room notes in `docs/dev/mapgen.md`, or move them to a new `docs/dev/special-rooms.md`? | **New topic file** in R1: dens, curses, and prayer are rules, not generation. |
| 14 | Should **Pray** also work at hand-made altars outside special rooms (the showcase garden, a future temple)? | **No**: only rooms with `prayer`; a level marks one if it wants it. |
| 15 | Weights in §3.2 (altars and dens at ~4 % of floors each) | Accept; tune after a play-through of the Ascmearc. |
| 16 | Which §9 rooms to build, and in which order | Ranks 1 to 7 in that order; 8 if the owner likes chaos; skip 9 to 12. |
| 17 | A cursed room's kill with no undead of its level: the nearest level, or nothing? | **Nearest**, so low floors' curses are not empty; add low-level undead (a rat skeleton) with the campaign's barrow wave if the jump from 1 to 3 feels wrong. |
