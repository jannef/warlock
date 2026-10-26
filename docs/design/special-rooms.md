# Warlock — Special rooms, second wave: teeming dens, cursed rooms, draining rooms, forgotten altars, hot and cold rooms, and thirteen more

Design document for the owner's review (2026-10-26). Written against `origin/main` at 1c0ce120 and revised with the owner's decisions (§13). Nothing here is implemented; §12 cuts it into issues. Everything builds on the existing room-effect system (`assets/data/room_effects.toml`, spec §7 "Special rooms"): an effect is data, a room carries it, and the rules ask one question about a cell. The developer notes move to a new topic file, `docs/dev/special-rooms.md` (R1, §12).

## Contents

1. Summary
2. What exists today
3. Shared mechanisms (each specified once)
4. Teeming dens (the owner's "swarms")
5. Cursed rooms
6. Draining rooms
7. Forgotten altars and prayer
8. Hot and cold rooms
9. More rooms
10. What the player sees, and the wiki
11. Talents and class abilities
12. Implementation plan
13. Decisions (owner, 2026-10-26)

## 1. Summary

| Piece | Proposal |
|---|---|
| Which room is drawn | every kind of special room (a **feature**) carries its own default `weight` in its own data. A dungeon (a branch, or a range of its depths) may name `favoured` features, each with a fixed share of the draws, and a `banned` list. Everything else is drawn from all the features that are not banned. A new feature therefore appears in every dungeon without touching any dungeon file. The old `effects` list is retired. §3.2 |
| Rare whole-level versions | one field, `level_chance` (percent): when a generated level draws an effect that has one, the effect covers the **whole level** instead of the room, with that chance (5 for every effect that uses it). Hand-made levels may name one: `[level] effect = "cursed"`. §3.1 |
| A status while inside | `status = "ebbing"`: everyone inside holds that status at least at one degree, and it eases once they leave. Statuses already carry pace, ticks, mana drain, damage types, and resistances, so the draining room and the silent vault need no new rule of their own. §3.3 |
| Terrain in the room | `feature = "forgotten_altar"`: a generator places one cell of that terrain at the room's middle. §3.4 |
| A hint from outside | `near` (a message), said once when a cell of the room first comes into the player's view. §3.5 |
| Abilities a place offers | a new abilities-menu source, *Here*, and outcome tables in data (`data/outcomes/`), shared by the altar's Pray, the spring's Drink, and the forge's Temper. §3.8 |
| Teeming den (swarm) | a room filled with **sleeping** hostile creatures of the floor's danger, and their hoard. Sleepers always come from the branch population's own draw (no den is hand-made per monster); some dens are themed, a warren of one type or a tribe of one faction. They wake to noise, a blow, or a sleeper's cry, and never on their own, so one loud step can wake the whole room. Stealth and sneak attacks matter here. §4 |
| Cursed room | a non-undead creature killed inside rises from its corpse two minutes later, as a random undead of its level or the nearest level. Taking or eating the corpse first stops it. Whole level at 5 %. §5 |
| Draining room | the status *Ebbing* drains mana from everyone inside, creatures too. Whole level at 5 %. §6 |
| Forgotten altar | ADOM's pool mechanic with its own flavour (no gods, no sacrifices, no conversion). **Pray** (abilities menu, *Here*; no key of its own) takes **one minute** and draws a boon, nothing, or a bane. The odds lean on the altar's hidden side against the player's alignment, and the side shows in the altar's light colour. Forty outcomes, most of them small (a few coins, a little healing, a brief hex, a pang of hunger, a wandering creature) and six "nothing happens" lines: about 30 % of prayers do nothing, 44 % something minor, 22 % something moderate, 4 % something great (a level gained or lost, a stat raised or lowered for good, a talent or a flaw for life, a relic, a horde). Each prayer has a **50 % chance to exhaust** the altar (a spent altar stays on the map), so an altar answers about twice on average and rarely swings a game. One rare outcome makes the **altar explode**: a fireball on its cell, and the altar is gone. No wish. §7 |
| Hot and cold rooms | the room sets its cells' **temperature** (+25 °C, −25 °C) and nothing else. What the temperature does (ice today; creatures, items, and food later) is the environment simulation's job. §8 |
| More rooms | all thirteen are built: silent vault, tempting hoard, haunted room, shrine of a side, black chamber, scriptorium, beast den, unstable room, heavy hall, healing spring, forge, hall of mirrors, collapsed floor. §9 |
| Talents | a new system `special_rooms` with eight talents, and a new system `prayer` with two. One class ability, *Consecrate* (priest, paladin). §11 |
| Slices | nineteen issues in six waves, never more than three at once that add actions or save state. §12 |

## 2. What exists today

| Part | Today (`docs/dev/mapgen.md`, spec §7) |
|---|---|
| Fields | `id`, `name`, `mark`, `enter`, `light`, `glow`, `noise`, `regen`, `mana_regen`, `shunned`, `floor`; at most 32 effects; ids are indices (in the save fingerprint) |
| Effects | shadowed room, echoing hall, hushed chamber, sanctuary, flooded room |
| Generation | `mapgen/special.rs`, last, from the level's map stream: each room except the arrival room and vaults (each of eight cave chambers) is special with `chance` 7 % until `per_level` 1; the effect is a **uniform** pick; about 40 % of generated levels have one. Branches override `chance`, `per_level`, and `effects`, an exclusive list (`RoomOdds`, `content/dungeon.rs`); only Fennmynster does: `{ chance = 25, per_level = 2, effects = ["flooded", "hushed", "shadowed"] }` |
| State | `LevelState::rooms` (effect, box, a cave blob's cells, entered), saved and hashed |
| The one question | `LevelState::room_effect_at(pos)` / `World::room_effect_at`, read by perception light and creature sight (together), noise, `World::healing_rate`, `World::mana_rate`, and `World::shuns` |
| Player | the `enter` line once per room; look mode names a room the player has been in; numbers never shown |
| Nearby systems this design reuses | statuses (`data/status.toml`: `tick`, `damage`, `damage_type`, `mana_damage`, `regen`, `mana_regen`, `cost`, `blocks`, `seen`); temperature per level (`WorldState::temperature_on`) and ice (`LevelState::advance_ice`, 10-minute ticks); spawning (`world/spawn.rs`: `make_creature`, the danger draw `content/danger.rs`); `World::unaware` (sneak attacks); alignment `Deed`s and `shift_alignment`; the `altar` terrain (`_`, steady glow); the cure id `prayer` already on every disease; the teleport and lightning traps (`world/trap.rs`); flaw talents (`flaw = true`: Frail, Thin-skinned, Slight, …); affixes and rarities (`data/affixes.toml`); the Metalcraft and Decipher script skills |
| What nothing does yet | nothing feels the cold or heat (`docs/dev/temperature.md`); creatures never sleep; there is no "rise from a corpse"; nobody falls between levels |

## 3. Shared mechanisms (each specified once)

### 3.1 `level_chance`: the rare whole-level version

| | |
|---|---|
| Field | `level_chance` (percent, 0 to 100, default 0) on an `[[effect]]` |
| Rule | when the generator has drawn this effect for a room, it draws once more from the **map stream**: with `level_chance` the effect **pervades the level** and the room stays plain. The extra draw happens only for effects that have a `level_chance` above 0. |
| State | `LevelState::pervading: Option<RoomEffectId>` (saved and hashed, left out while `None`, so other levels hash as before) |
| The one question | `room_effect_at` becomes `effects_at(pos) -> Here` (the room's effect, then the pervading one). Every reader folds both the same way: percents multiply, flags OR, statuses both apply. A level has at most two effects at a cell, so the cost stays the same. The light invariant test (`special_rooms_keep_the_light_invariants`) gains a pervading case. |
| Hand-made levels | `effect = "cursed"` in a level file's header (refused on the overworld and in local areas) |
| Player | the effect's `pervade` message on arrival, said each time the player arrives (as a level feeling, not once per run); look mode does not name the whole level, since the arrival line already says it |
| Uses it (5 %) | cursed (§5), draining (§6). Not now (decision 2): shadowed, hushed, echoing; cheap to add later as data. Never: sanctuary (a level hostiles shun is broken), flooded and every `floor`/`feature` effect, teeming dens, altars, hot and cold (a hot level is just its `temperature`), and every §9 room |
| Odds | 5 % of the floors that drew the effect: about 0.15 % of generated floors are cursed throughout and about as many drained (§3.2's shares). A story a player tells, not a regular feature. 5 % stays; we adjust later if need be (decision 1). The wiki states it. |

### 3.2 The draw: features, default weights, favoured and banned

Each kind of special room is a **feature**. Adding a feature means adding one effect file entry with its own weight; no dungeon file has to change, and the new feature turns up in every dungeon that has special rooms and has not banned it.

**Data.**

| Where | Field | Meaning |
|---|---|---|
| each `[[effect]]` in `room_effects.toml` | `weight` (1 to 100, default 10) | the feature's default weight: how often it is drawn relative to the others where no dungeon says otherwise |
| `room_effects.toml`, top | `chance`, `per_level` | as today (shared `chance` rises from 7 to 12, below) |
| a dungeon's `room_effects` (the branch's, or a `[[dungeon.levels]]` range's) | `chance`, `per_level` | as today |
| the same | `favoured = { flooded = 40, cursed = 15 }` | features with a **fixed share** of this dungeon's draws, in percent (1 to 100 each, together at most 100). A favoured feature keeps its share however many features are added later. |
| the same | `banned = ["hot"]` | features this dungeon never draws |
| the same | `effects` | **retired**: refused at load with a message pointing to `favoured` and `banned` |

Fields left out fall back as today: a range's table, then the branch's, then the shared one (which has no `favoured` or `banned`). Validation at load: unknown ids, an id both favoured and banned, shares summing over 100, and a dungeon that bans every feature while its `chance` is above 0 are content errors.

**The draw** (for one room, from the level's map stream):

1. **Eligible** = every feature, less the dungeon's `banned`, less those this room or level does not qualify for (§3.6).
2. If any eligible feature is favoured: draw `range(0, 100)`. Walking the favoured eligible features in effect-file order and adding up their shares, the first whose running sum is above the draw is chosen.
3. Otherwise (no favoured feature, or the draw landed past the favoured shares): a weighted pick, `range(0, total)`, over the eligible features that are **not** favoured, by their default `weight`. If that set is empty, the pick runs over the eligible favoured ones by their shares.

A favoured feature that this room is not eligible for gives its share to the rest (step 3). With no `favoured` the first draw is skipped, so the shared table costs one draw per room as today. Adding features changes which effect existing seeds draw: the Depths' snapshots change once, in R1 (reviewed, accepted).

**Code.** `RoomOdds { chance, per_level, effects: Vec<RoomEffectId> }` becomes `RoomOdds { chance, per_level, favoured: Vec<(RoomEffectId, u8)>, banned: Vec<RoomEffectId> }`; `RoomOdds::shared` has both empty; the weights are read from `RoomEffects` at the draw. `RoomOddsSpec::layer` and `resolve` (`content/dungeon.rs`) change field for field. The same shape could later serve shops' `kinds`; not in this design.

**Default weights**, each in its effect's own entry:

| Feature | Weight | Share of draws | Floors with one |
|---|---|---|---|
| shadowed | 10 | 7.5 % | 4.5 % |
| echoing, hushed, flooded, teeming den, forgotten altar | 8 each | 6.0 % each | 3.6 % each |
| sanctuary, cursed, draining | 6 each | 4.5 % each | 2.7 % each |
| hot, cold, silent vault, tempting hoard, haunted, black chamber, scriptorium, beast den | 5 each | 3.8 % each | 2.3 % each |
| shrine of a side, unstable, heavy hall, healing spring | 4 each | 3.0 % each | 1.8 % each |
| forge, hall of mirrors, collapsed floor | 3 each | 2.3 % each | 1.4 % each |
| *(total 133)* | | | |

"Floors with one" assumes the shared `chance` rises from 7 % to **12 %** in R1, so about 60 % of generated floors have a special room (about 40 % today). With twenty-four kinds instead of five, keeping 7 % would make each one rarer than the owner accepted for altars and dens (about 4 % of floors). The Ascmearc's 26 floors would hold about one altar and one den per game. Eligibility (§3.6) moves the shares a little: dens and cursed rooms are not drawn on a level without a danger. All numbers are data, tuned after a play-through.

**Existing dungeons.** Only Fennmynster names `effects` today. It becomes:

```toml
# The fen seeps in: water, hush, and shadow most of the time, the dead often, never heat.
room_effects = { chance = 25, per_level = 2, favoured = { flooded = 35, cursed = 15, hushed = 15, shadowed = 15 }, banned = ["hot", "forge"] }
```

80 % of its special rooms are its own four; the other 20 % come from every feature not banned, including those added later. The shared table and every other branch need no change.

### 3.3 `status`: a status while inside

| | |
|---|---|
| Field | `status` (a status id) on an effect |
| Rule | after every action of an actor whose cell has the effect (the player's in `World::apply`, a creature's at the end of `creature_turn`, and each step of a rest or a long action), `World::dwell(id)` gives it the status at one degree with `refresh` stacking. Once the actor leaves, the status eases by its own `eases`. Its `eases` must be at least the longest single action (10 s) so it never lapses mid-rest; validated. |
| Resistance | a status with a `damage_type` is never given to one immune, and is shrunk for one resistant (existing rule) |
| Who | **everyone inside**, creatures too (decision 11): a kobold shaman's mana ebbs as the wizard's does |
| Used by | draining room (*Ebbing*), silent vault (*Silenced*), shrine of a side (*Unwelcome*). Never for temperature (§8). |

### 3.4 `feature`: one cell of terrain

`feature` (a terrain id; not walkable) is placed by the generator at the free floor cell nearest the room's centre (a cave: the blob's seed cell), and never in a doorway or on a path between two doors (the generators' connectivity check runs after). A hand-made level draws its own. Each special room's feature is its **own terrain** with its own name and description, so look mode tells it from scenery: `forgotten_altar` (§7), `shelves` (scriptorium), `spring` (healing spring), `anvil` (forge).

### 3.5 `near`: a hint from outside

`near` (a message) is said once per room, the first time any of its cells comes into the player's view, before they set foot in it. `LevelState::rooms` gains `glimpsed: bool` beside `entered` (saved, hashed). Perception then knows the room as "glimpsed" but does not name it; look mode names it only once entered, as today (Sense of place changes that, §11). Optional on every effect. The five existing effects get none.

### 3.6 Eligibility

| Field | Meaning |
|---|---|
| `needs_danger` (bool) | drawn only on a level with a danger (`World::danger` > 0); hand-made levels without one never get it. Teeming den, beast den, cursed, haunted, tempting hoard |
| `min_cells` (default 0) | eligible only for a room (blob) of at least this many floor cells |
| `max_cells` (default none) | eligible only for a room of at most this many floor cells. Unstable room, hall of mirrors |
| `needs_below` (bool) | drawn only on a branch level with another level below it. Collapsed floor |

The draw runs over the eligible features only (§3.2 step 1).

### 3.7 Determinism

| Draw | Stream |
|---|---|
| room chance, the feature draw, `level_chance`, a room's side, `feature` placement | the level's **map** stream (as today, at the end) |
| a den's sleepers and hoard, a tempting hoard's items, a scriptorium's scrolls | new `swarm/<level id>` (`Rng::swarm`), so the population, loot, and gear streams do not move |
| a sleeper's gear | `gear/<level id>`, after the population's, as `make_creature` does |
| what a corpse rises as, an outcome (prayer, spring, forge), a warp, a collapse, a guardian or a haunting | the gameplay stream (they happen in play) |
| a hall of mirrors' reflections | none: a hash of the level, the turn, and the actor (§9.11), so perception draws nothing |
| waking, dwelling, rising times | none: thresholds and game time |

### 3.8 Abilities a place offers, and outcome tables

| | |
|---|---|
| Source | a new abilities-menu source **Here** (`Source::Place`): abilities a place offers, listed while the player stands on or next to the feature cell of a room that offers them, absent (not greyed) elsewhere. No key of their own (`KeyCommand` unbound by default), bindable like every ability. Pray (§7), Drink (§9.10), Temper (§9.12). |
| Only here | these abilities exist **only** in these rooms (decision 14). The showcase's water-garden altar and the temple of Cēaptūn are scenery and services; a hand-made level that wants an altar room marks it (`[[room]] effect = "altar"`). |
| Outcome tables | `data/outcomes/<id>.toml`, one table a file (churning content is a directory): `kinds` with base weights (`boon`, `nothing`, `bane`; the forge's `improved`, `nothing`, `flawed`, `ruined`), a `lean` that moves weight between the best and the worst kind, `exhaust` (below), and `[[outcome]]` entries (`kind`, `id`, `weight`, `tier` = `minor`, `moderate`, or `great`, `effect`, and `message` or `messages`, a list one line of which is drawn). Each `effect` is one of a closed set of kinds, each calling one hook; `effect` may be empty (a "nothing happens" line). Drawn from the gameplay stream. `content/outcome.rs` parses and validates; `world/outcome.rs` applies. |
| Time | each ability is a long action through `World::keep_at`, the rest's machinery: anything that disturbs a rest interrupts it (a creature comes into view, a hit), and an interrupted one gives **nothing**, but the time is spent |
| Repeats and exhaustion | `exhaust` (percent, 0 to 100, **default 50**) on a table: after each completed use, one more gameplay draw decides whether the place is **spent** (its feature cell becomes its spent terrain, `spent = "spent_altar"` on the table, and the ability is no longer offered there). The forgotten altar keeps the default, 50 (the owner's rule, decision 19, §7.3). The spring and the forge set `exhaust = 0`: no limit, the owner's no-anti-farming rule, and the risk of their bad outcomes is the price. An interrupted use draws nothing and never exhausts. |

## 4. Teeming dens (the owner's "swarms")

**Name**: id `teeming`, name **teeming den**. The word swarm reads as insects, and the room is mostly sleepers of every kind.

**Messages** (no ADOM wording, and no sense of dread before the room is seen): `near` *"A warm, rank smell drifts out of the dark ahead."* `enter` *"You hear slow breathing all around you."* The room is often dark, so the player may hear it before seeing it. Sixth sense (an existing talent) adds *"Something sleeps on this level."* on arrival.

**What is data and what is procedural** (decision 3). A den is **not** a hand-drawn vault. The data defines what a den *is*: its effect entry in `room_effects.toml` with the fields below (its weight, its messages, how many sleepers per cell and the least and most, how loud a noise must be to wake them, how loud a waking sleeper cries, how good the hoard is). The **generator** decides everything about a particular den when it makes the level: which room becomes the den (any eligible room, by the draw of §3.2), how many sleepers it holds (from the room's size), which creatures they are (drawn from the floor's danger and the branch's population), where each lies, what gear each carries, and what the hoard holds. Two dens are never the same, and no level file has to mention one.

**The principle: no hand-made den definitions per monster.** A den's sleepers are always drawn from the branch's population by the same draw that lays down a floor's creatures (`content/danger.rs::draw`): the floor's danger, each type's level weight, its rarity tier (`data/creature_rarities.toml`: common 100, uncommon 50, rare 25, very rare 10), the strays (weaker types the draw still allows, weight 1 down to eight levels below the danger), and the out-of-depth chance (5 % of draws made 1 to 3 dangers deeper), and later tribes data when it exists. So adding a monster type to a branch's population puts it in that branch's dens at once, and den data never names a monster. The only filter a den kind may give is by faction (§9.5's beast den), which is also data about the den, not about any monster.

**Themed dens** (optional, procedural; **recommended**). With `swarm.themes = { kind = 10, faction = 20 }`, a den is themed with that chance: first one creature type (a *warren*, 10 %) or one faction (a *tribe*, 20 %) is drawn, by the same weighted draw, and then every sleeper is drawn from the population restricted to it. Nothing is hand-picked: a warren of a common type is far likelier than one of a rare type, and out-of-depth and stray draws still apply within the theme. Themed dens make a few dens read as a place (a rat warren, a goblin camp) at the cost of one extra draw. When tribes data exists, the faction theme becomes a tribe theme with no change to den data.

**Examples** (illustrative draws from today's creatures; the numbers are what the rules give):

| Den | How it comes out |
|---|---|
| Floor 3 of a branch whose population holds kobolds, goblins, and vermin; a room of 36 floor cells | 36 / 4 = **9 sleepers**, mixed: 3 kobolds and 2 goblins (level 2, weight 4, common), a goblin archer and a giant centipede (level 3, weight 4, uncommon and common), a rat and a grave worm (level 1, weight 2: the strays), and the hoard, 2 items at item level + 1 |
| A **warren** on floor 2; a cave chamber of 22 cells | the theme draws one type, the rat (common, so warrens of rats are frequent); **5 sleepers**, all rats; a hoard of 1 item. A warren of goblin chiefs could happen in principle, but the chief's rarity (rare, 25) and level (5) make it a very unlikely draw at danger 2 |
| A **tribe** den on floor 5; a hall of 64 cells | the theme draws the faction `goblins`; **16 sleepers** (the most), all goblins: plain goblins and goblin archers mostly, a goblin chief now and then (level 5, rare), and an ogre only by an out-of-depth draw. A goblin camp, and a loud one if it wakes |
| Floor 3 again, a room of 32 cells, no theme | **8 sleepers**; each draw has the 5 % out-of-depth chance, so about one den in three like this holds **an out-of-depth sleeper** (1 − 0.95⁸ ≈ 34 %): here a kobold shaman (level 4) drawn at danger 4 to 6, among kobolds and rats. Waking the shaman first is the mistake |

| Rule | Proposal |
|---|---|
| Sleepers | a new status **`asleep`** (`max = 1`, no `eases`, `seen = "asleep"`, `sleeps = true`): an actor with a `sleeps` status takes no turn (one check at the top of `creature_turn`, beside the coarse-turn check) and is `World::unaware` of everyone. Look mode says "a goblin (hostile, asleep)". A status keeps it saved, hashed, and shown with no new state, and later sleep spells can use the same status. |
| How many | one sleeper per 4 floor cells of the room, 4 to 16 (`swarm.per_cells`, `swarm.min`, `swarm.max`), drawn by the floor's danger from the branch's `population` (`content/danger.rs::draw`: level weights, rarity tiers, strays, and the out-of-depth chance), hostile types only (or `swarm.factions`), within a theme if one was drawn (`swarm.themes`), never uniques, from `swarm/<level id>`; placed on free floor in the room, made by `make_creature` with gear rolled, asleep |
| Waking | **noise**: in `World::hear` (where a creature's `Heard` is pushed), a sleeper wakes when the loudness at its cell is at least its hearing threshold × `swarm.wake` (200 %): a sneaking step does not wake it, a fight or a door kicked open beside it does. **A blow** of any kind wakes the one struck. **A cry**: a sleeper that wakes makes an `Act::Voice` noise of `swarm.cry` (60) at once, which may wake the next: a cascade, deterministic, through the existing propagation. **Light and sight do not wake** (a lantern carried in is fine). |
| Never on their own | no timer and no spawn tick wakes them (decision 5): the player chooses when the den wakes |
| The hoard | one item per 4 sleepers (at least 1) at the floor's item level + `swarm.loot_level`, `loot::make`, on the room's floor, from `swarm/<level id>` after the sleepers |
| Versus the normal population | sleepers come on top of the 35 per 1 000 cells. They **count** toward the spawning cap (`attempt` counts hostiles on the level), so a floor with a den spawns nothing until enough of it is dead. Nothing else changes. |
| Stealth | step noise already falls with Stealth (`noise.rs`), so a stealthy player crosses a den that a clanking one wakes. Sleepers are `unaware`, so **sneak attacks** land on each (Sneak attack, Ambusher). A kill is a noise too (`Act::Attack`): killing quietly in a den is the thief's game. |
| Alignment | **none** (decision 4). Killing a sleeping hostile is a hostile kill (`kill_hostile`), as now. |
| Patrols | a den is never where a creature patrols (as for a shunned room); sleepers have no post to return to until woken, after which the normal AI applies |
| Fields | `swarm = { per_cells = 4, min = 4, max = 16, wake = 200, cry = 60, loot_level = 1, themes = { kind = 10, faction = 20 } }`, `needs_danger = true`, `min_cells = 16` |
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
swarm = { per_cells = 4, min = 4, max = 16, wake = 200, cry = 60, loot_level = 1, themes = { kind = 10, faction = 20 } }
```

## 5. Cursed rooms

**Messages**: `near` *"A cold draught smells of the grave."* `enter` *"The air is cold and the dead are near."* `pervade` *"The dead do not rest on this level."*

| Rule | Proposal |
|---|---|
| Who rises | a creature that dies on a cursed cell, whose faction is not `rise.faction` (`undead`), and that leaves a corpse (`corpse = true`). Anyone's kill counts, a creature's included. Never the player. |
| As what | an undead type (faction `undead`, not unique, no npc) of the **same level** as the dead, drawn uniformly from the gameplay stream at death; if none has that level, the **nearest** level (a tie goes to the lower; decision 17). Today's undead are skeleton and zombie (3), barrow ghoul (4), drowned (5), ghost (8), and ash wraith (15), so a rat (1) rises as a skeleton or a zombie and a troll as a ghost; more undead make the match closer. |
| When | `rise.after` (2 minutes of game time, 20 normal turns) later. `LevelState::risings: Vec<Rising { at, pos, from: CreatureId, into: CreatureId }>` (saved, hashed, left out while empty); `rise_tick` runs once at the end of `run_turn`, beside `spawn_tick`. A frozen level (the player away) rises when the player returns, as spawning catches up. |
| From its corpse | at `at`, if a pile at `pos` still holds a corpse of `from`, one is taken and the undead stands on `pos` (or the nearest free cell, the spawner's helper). **A corpse picked up, eaten, or destroyed does not rise**: that is the counterplay. A corpse carried out of the room and dropped is not cursed again. |
| Warnings | at death, if seen: *"The goblin's corpse twitches."* At rising: *"The goblin's corpse rises as a zombie!"*, or a noise (`Act::Voice`, a groan) if unseen |
| XP, loot | a risen undead gives its own kill XP (no anti-farming limit) and carries nothing; what the dead carried already lay on the floor |
| Whole level (5 %) | every cell is cursed: a whole floor of risings. With a teeming den elsewhere on the level, that is the floor players remember. |
| Fields | `rise = { faction = "undead", after = "2m" }`, `needs_danger = true`, `level_chance = 5`, `weight = 6` |
| Cost | M. New: `rise` parse, `risings` state, one line in `fight.rs::dies`, `rise_tick`. Risk: a rising during a rest, since it is a creature arriving in view and the rest stops as usual. |

## 6. Draining rooms

**Messages**: `enter` *"You feel your power seeping out of you."* `pervade` *"Something here drinks at your power."* Glow: a faint cold violet (`[10, 6, 24, 8]`), a visual cue.

| Rule | Proposal |
|---|---|
| Status | **`ebbing`** (name *Ebbing*, `seen = "ebbing"`, `max = 1`, `stacking = "refresh"`, `tick = 20`, `mana_damage = 1`, `mana_regen = 0`, `eases = 30`, start *"Your power ebbs away."*, end *"Your power stops ebbing."*), given while inside (§3.3). It drains 1 mana per 20 s (180 an hour) and stops regeneration, never below 0 and never a death (existing `mana::drain`). |
| Who | everyone with mana: a wizard and a kobold shaman alike. A casters' fight in a draining room is a melee. |
| Whole level (5 %) | `level_chance = 5`: a floor where casters must ration and leave quickly |
| Fields | `status = "ebbing"`, `level_chance = 5`, `glow`, `weight = 6` |
| Cost | S once §3.3 exists (no new rule: the status framework already drains). |

## 7. Forgotten altars and prayer

**What this is, and is not.** A forgotten altar is ADOM's **pool** mechanic with a different flavour: a place where the player may gamble on a draw of boons and banes until the altar is spent. It is **not** ADOM's altar and god system: there are no gods to name or serve, no sacrifices, no conversion of altars, and no relationship that builds over time. The altar's hidden side (§7.2) only tilts the gamble's odds; it is flavour on the pool, not a pantheon.

**The shape of the gamble** (decision 18). Most prayers do little: a few coins, a little healing, a brief hex, a pang of hunger, a wandering creature, or nothing at all with a line of flavour. The outcomes that make or ruin a character are rare, and an altar is usually spent after one or two prayers (§7.3), so **no single altar swings a game for sure**. The range is wide on purpose: kneeling should be a small story every time, and only now and then a large one.

**Messages**: `near` *"A faint glow rises from an altar ahead."* `enter` *"An old altar stands here, its god long forgotten."*

### 7.1 The altar

| | |
|---|---|
| Room | `feature = "forgotten_altar"`, offers Pray with the outcome table `forgotten_altar` (§3.8, §7.4), `weight = 8`, `min_cells = 9` |
| Its terrain | a **new terrain**, `forgotten_altar` (glyph `_`, stone, not walkable), with its own name and description (decision 14), so it never reads as the showcase's or a temple's altar. Look mode: "a forgotten altar". Description: *"An altar of worn stone. Its carvings have been rubbed smooth by hands or by time, and whatever god it served has no name left. Something here still listens."* |
| Spent | a second **new terrain**, `spent_altar` (glyph `_`, dark grey, stone, not walkable, no light, no glow). Look mode: "a spent altar". Description: *"An altar of worn stone, cold and dark. Whatever listened here has stopped."* Pray is not offered beside it. §7.3 |
| Destroyed | the explosion (§7.5) turns the altar's cell into `rubble` (today's terrain: walkable, loose stones that clatter) |
| Its side | drawn at generation from the map stream (§7.2), kept in the room's state (`Room::side: Option<Alignment>`, saved and hashed only when set). A hand-made level gives it: `[[room]] effect = "altar" side = "evil"`. |
| Its light | by side: `altar_good` (warm gold, today's `altar_glow`), `altar_neutral` (pale grey), `altar_evil` (dull red). The terrain is the same; its light is chosen per cell (a terrain light override in `LevelState`, the same mechanism a placed light uses). The override is dropped when the altar is spent or destroyed. |

### 7.2 The altar's hidden side, plainly (decision 8)

Every forgotten altar is **good, neutral, or evil** in nature (the flavour says a god once owned it; the rules know only the side). The game decides which when it makes the level (one good to two neutral to one evil) and never says it in words.

The side matters because it tilts the prayer's odds against the player's own alignment (their band: good, neutral, or evil). The base weights are boon 35, nothing 30, bane 35 (§7.4); the lean moves 15 points:

| The altar is | The player is | Effect on the odds |
|---|---|---|
| good or evil | on the **same** side | **agreement**: 15 points move from bane to boon (boon 50, nothing 30, bane 20) |
| good or evil | on the **other** side (good at an evil altar, evil at a good one) | **opposition**: 15 points move from boon to bane (boon 20, nothing 30, bane 50) |
| good or evil | neutral | no lean (boon 35, nothing 30, bane 35) |
| neutral | anyone | no lean |

Praying also moves the player's alignment a little: +5 at a good altar, −10 at an evil one, nothing at a neutral one (`Deed::Prayed(side)`, `[shift] prayed_good`, `prayed_evil` in `data/alignment.toml`). A paladin who prays at red altars strays.

**How the player learns the side**: by the colour of the altar's glow. Gold is good, grey is neutral, red is evil. The game never explains the colours; a player learns them by praying and noticing what happens, or by reading the wiki, which states them. Recommendation kept: the light colour shows the side, the words never do.

### 7.3 Praying

| | |
|---|---|
| How | **Pray** under *Here* (§3.8), listed while the player stands next to a forgotten altar (never a spent one) |
| Action | `Action::Pray` (`pray` in action text). A long action of **one minute** (decision 6; six 10-second steps through `World::keep_at`); an interrupted prayer gives nothing (*"Your prayer is broken."*), draws nothing, and never exhausts the altar. |
| Outcome | at the end, one draw from the gameplay stream on the table's weights, leaned by side (§7.2), then the outcome's hook and its message |
| Exhaustion | then, unless the altar exploded, one more gameplay draw: with `exhaust` (**50 %**) the altar is **spent**: its cell becomes `spent_altar` through `World::set_terrain` (which drops the cell's light masks, as digging does) and its light override is removed. Seen: *"The altar's glow gutters and dies."* Pray is offered at it no more. The chance does not depend on the outcome: a great boon spends the altar no more often than nothing does. |
| How many prayers | one always; a second with 50 %, a third with 25 %, and so on: **about two on average** (1.97 with the explosion's chance), five or more at one altar in sixteen |
| Saved state | **none new**: the level's terrain grid is already saved and hashed (digging's rubble is), so the swap to `spent_altar` or `rubble` is the whole record. *Here* asks the terrain, so a spent or destroyed altar offers nothing. `Room::side` stays (dumps show it); the `rooms` dump marks the room `spent` or `destroyed` from its feature cell's terrain. The two new terrains change the content fingerprint as any new terrain does. |
| Versus the no-limits rule | the first draft had no limit on prayers (the owner's no-anti-farming rule: time and risk are the price, never a lockout by one failed roll). The owner now sets exhaustion for altars (decision 19). It is not a lockout by a failed roll: the chance is the same whatever the prayer gave, a prayer still costs only a minute and its risk, and the player may seek out every other altar in the game (one in about 28 generated floors has one, §3.2). It replaces farming one altar without end, so what one altar can do to a game stays bounded. The spring and the forge keep no limit (§3.8). |
| No wish | no outcome grants a wish; wishes stay on #196 |

### 7.4 The table: `data/outcomes/forgotten_altar.toml`

```toml
seconds = 60
# Weights of the kinds, before the side's lean.
kinds = { boon = 35, nothing = 30, bane = 35 }
# Moved between boon and bane on agreement or opposition (§7.2).
lean = 15
# Percent: the altar is spent after a completed prayer (the owner's rule; default 50).
exhaust = 50
spent = "spent_altar"

[[outcome]]
kind = "boon"
tier = "minor"
id = "soothed"
weight = 13
effect = { heal = "2d6", cure = ["bleeding"] }
message = "A little warmth runs through you."

[[outcome]]
kind = "nothing"
tier = "minor"
id = "omen"
weight = 20
messages = ["The smoke of your breath bends toward the stairs.", "For a moment you smell rain.", "Somewhere far below, a bell rings once.", "The carvings seem to shift under your eyes, and are still."]

[[outcome]]
kind = "bane"
tier = "moderate"
id = "burst"
weight = 5
effect = { explode = { spell = "fireball", radius = 2, damage = "3d6", damage_type = "fire", heat = true, noise = { sound = "explosion", loudness = 150 } }, becomes = "rubble" }
message = "The altar bursts apart in fire!"
```

Weights are **within each kind** (each kind's weights sum to 100, so a weight is also its percent of that kind). Items left "at the altar's foot" go on the free cell beside the altar nearest the player (the altar is not walkable, so nothing lies on it).

**Nothing happens** (kind `nothing`, 30 of the base weights; no hook, flavour only, never a hint of anything true):

| Outcome | Weight | Message |
|---|---|---|
| Silence | 25 | "Silence answers you." |
| Echo | 15 | "Your words echo, and fade." |
| Dust | 15 | "Dust sifts down from the ceiling." |
| Flicker | 15 | "The altar's glow flickers, then steadies." |
| Breath | 10 | "A cold breath brushes your neck, and is gone." |
| Omen | 20 | one of four lines (above): the smoke, the rain, the bell, the carvings |

**Boons** (kind `boon`; within it: minor 65, moderate 29, great 6):

| Tier | Outcome | Weight | What it does | Hook | Message (vague) |
|---|---|---|---|---|---|
| minor | Soothed | 13 | 2d6 hit points back, bleeding stopped | `heal`, status cure | "A little warmth runs through you." |
| minor | Coins | 10 | 2d10 copper coins at the altar's foot; one time in four 1d6 gold coins instead | object placement (`copper_coin`, `gold_coin`) | "Coins lie at the foot of the altar." |
| minor | Sated | 8 | nutrition one stage better (never past Satiated) | hunger | "You feel fed." |
| minor | Trinket | 7 | an item at the floor's item level at the altar's foot | `loot::make` | "Something lies at the foot of the altar." |
| minor | Trickle | 9 | 2d4 mana back | mana | "Your mind stirs." |
| minor | Uplifted | 7 | the `inspired` status (five minutes) | statuses | "Your heart lifts." |
| minor | Wary | 5 | every trap within 10 cells becomes known | `World::know_trap` | "You sense where the floor lies." |
| minor | Glimpse | 6 | one random unidentified carried item identified (none: "You feel you understand a little.", nothing) | `loot::identify` | "Something you carry becomes clear to you." |
| moderate | Mended | 6 | full hit points, bleeding and poison cured | `heal`, status cure | "Warmth runs through you, and your wounds close." |
| moderate | Replenished | 5 | mana to full | mana | "Your mind feels clear and full." |
| moderate | Cleansed | 4 | every disease cured (cure id `prayer`) | `Diseases::cure` | "A sickness leaves you." (else "You feel clean.") |
| moderate | Restored | 4 | all stat damage mended | the temple's mending service code | "You feel whole again." |
| moderate | Blessed | 5 | the `blessed` status, 3 degrees | statuses | "You feel watched over." |
| moderate | Offering | 3 | an item at item level + 3 at the altar's foot | `loot::make` | "Something lies at the foot of the altar that was not there before." |
| moderate | Favour | 2 | a fifth of the XP to the next level | `earn` | "You feel wiser." |
| **great** | **Gift of the forgotten** | 2 | +1d3 to a random stat's score and potential, for good | `Stats::train` (potential first) | "You feel changed." |
| **great** | **Exalted** | 1 | a whole experience level at once (XP to the next level's threshold, with its picks due) | `earn` | "Power floods into you." |
| **great** | **Chosen** | 1 | a talent the player qualifies for, given free and kept for life (never counted against picks) | `Actor::boons` (new, saved), read by `Talents::regrant` beside race and sign | "You have been chosen." |
| **great** | **Relic** | 2 | an item at item level + 10, of the top rarity its base allows, at the altar's foot | `loot::make` with a rarity floor | "Something gleams at the foot of the altar." |

**Banes** (kind `bane`; within it: minor 60, moderate 34 with the explosion, great 6):

| Tier | Outcome | Weight | What it does | Hook | Message (vague) |
|---|---|---|---|---|---|
| minor | Scorched | 11 | 2d6 fire damage through `hurt` with `Harm::Spared` (never the last hit point; fire resistance counts) | `hurt` | "The altar's glow flares and burns you." |
| minor | Sapped | 7 | 2d4 mana lost (never below 0) | `mana::drain` | "Your mind dims a little." |
| minor | Lightened | 7 | a third of the coins carried vanish (none: nothing lost) | inventory | "Your purse feels lighter." |
| minor | Hungry | 8 | nutrition one stage worse (never into Starving) | hunger | "Your stomach growls." |
| minor | Sluggish | 8 | the `sluggish` status, 1 degree | statuses | "You feel sluggish." |
| minor | Ill-wished | 6 | `ill_wished`, 1 degree | statuses | "You feel a stranger's ill will." |
| minor | Snuffed | 5 | the player's lantern goes out, if lit (none lit: only the line) | `World::toggle_lantern` | "A cold wind blows past you." |
| minor | Drawn | 4 | one hostile of the floor's danger (the normal draw, `content/danger.rs::draw`), awake, at the room's edge, out of view if it can be (danger 0: nothing) | `make_creature`, the spawner's placement | "You hear something stir." |
| minor | Muddled | 4 | one identified magic item carried becomes unidentified again (none: nothing) | **the one new hook**: `loot::forget` clears the item's `identified` flag | "Something you carry seems strange to you." |
| moderate | Sickened | 5 | a random disease, the deadly ones included (grave rot) | `infect` | "A chill settles in your bones." |
| moderate | Withered | 5 | 1d3 stat damage to a random stat (mended as any stat damage is) | stat damage | "Something is taken from you." |
| moderate | Smitten | 3 | a bolt from above, 4d10 lightning through `hurt` (resistances count); **it can kill** | the lightning trap's spell code | "The altar answers with fire from above!" |
| moderate | Answered | 4 | a hostile creature of danger + 4, awake, beside the altar (danger 0: the deepest type of the branch's population) | `make_creature` | "Something answers your prayer." |
| moderate | Hexed | 4 | `ill_wished`, 3 degrees | statuses | "You feel watched with malice." |
| moderate | Tithe | 3 | the most valuable stack carried, worn and wielded things included, vanishes | inventory | "Something precious is gone." |
| moderate | Displaced | 5 | teleported as the teleport trap does, to a random free cell of the level | the trap's teleport (`world/trap.rs`) | "The world lurches around you." |
| moderate | **The altar explodes** | 5 | a fireball on the altar's cell, and the altar is gone (§7.5) | the burst spell code (`World::cast_burst`), `World::set_terrain` | "The altar bursts apart in fire!" |
| **great** | **Wrath of the forgotten** | 1 | −1d3 to a random stat's potential and score, for good (never below 3) | `Stats` | "You feel lessened, forever." |
| **great** | **Unmade** | 1 | one experience level lost (XP to the start of the level below; its hit points and picks taken back) | new level-loss rule (`World::lose_level`) | "Your past slips away from you." |
| **great** | **Forsaken** | 2 | a random flaw talent (Frail, Thin-skinned, …) the player lacks, kept for life | `Actor::boons` | "You have been marked." |
| **great** | **Damned** | 2 | a horde, 2d4 hostiles of danger + 3, awake around the altar | `make_creature` | "The dark fills with answering voices." |

That is **40 outcomes** (19 boons, 21 banes) and **6 "nothing happens"** lines. All but Muddled run on existing systems; the great ones need R6's `Actor::boons`, `World::lose_level`, and the rarity floor.

**Shares per prayer** (percent of prayers: the kind's weight × the tier's share within it):

| | Nothing | Minor | Moderate | Great | Great boon / great bane | The altar explodes |
|---|---|---|---|---|---|---|
| No lean (boon 35, nothing 30, bane 35) | **30.0** | **43.75** (boon 22.75, bane 21.0) | **22.05** (boon 10.15, bane 11.9) | **4.2** | 2.1 / 2.1 | 1.75 |
| Agreement (50 / 30 / 20) | 30.0 | 44.5 (boon 32.5, bane 12.0) | 21.3 (boon 14.5, bane 6.8) | 4.2 | 3.0 / 1.2 | 1.0 |
| Opposition (20 / 30 / 50) | 30.0 | 43.0 (boon 13.0, bane 30.0) | 22.8 (boon 5.8, bane 17.0) | 4.2 | 1.2 / 3.0 | 2.5 |

**Per altar** (about two prayers, §7.3): with no lean, a great boon comes at about **4 %** of altars and a great bane at about 4 %, so some great outcome at about 8 %; the explosion at about 3.4 %. At an agreeing altar a great boon is about 6 %; at an opposed one a great bane is about 6 % and the explosion about 5 %. A player who finds one altar in a game most often takes away a few coins, a heal, or a hex; about one altar in twelve marks the character for good.

Smitten, Answered, the explosion, and Damned can each end a weak character's run; the minor banes never kill (Scorched is spared). The wiki lists the table exactly, with the lean, the tiers, the exhaustion chance, and the colours. The table is data: the owner tunes it in review. Making the great outcomes still bigger (ADOM's crowning, artifacts, wishes) stays a deferred follow-up on #196 (the "Jackpots at ADOM level" line).

### 7.5 The altar explodes (decision 20)

| | |
|---|---|
| What | a **fireball** centred on the altar's cell, through the spell code the fireball trap and the scroll of fireball share (`World::cast_burst`, `world/spell.rs`): the burst's cells by `burst_cells` (radius 2, stopped by walls), then everyone in them hurt |
| Damage | 3d6 fire (the fireball trap's numbers) to **everyone in the burst, the player included**: the player prays from beside the altar, so they are always in it, and so is anything Drawn or Answered that stands close. Fire resistance and immunity count, and Evasion's dodge (`World::evaded`) applies as to any burst. **It can kill** (no `Harm::Spared`). It is cast as a trap's spell is: its kills are no one's and earn no experience, and its lines name it "the altar's fire". |
| Light, sound, heat | all from the existing effect code: the `fireball` spell effect is shown (its flash and particles; a still flash under reduced effects; never read by the rules), the noise is the trap's explosion (`sound_at`, `explosion`, loudness 150: much of the level hears it), and `heat = true` melts ice in the burst as the trap's does. No `blast`: the walls around stand. |
| The altar | gone: its cell becomes `rubble` (`World::set_terrain`; walkable, it clatters underfoot), its light override is removed, and *Here* no longer lists Pray. No exhaustion draw follows (nothing is left to spend). |
| Weight | 5 within the banes, moderate tier: **1.75 %** of prayers with no lean (1.0 % agreeing, 2.5 % opposed), about 3.4 % of altars |
| Order | the line *"The altar bursts apart in fire!"*, then the burst (its hurt lines and any deaths), then the terrain change, all in the prayer's last step |
| Penitent | does not stop it (it is moderate, not great) |

## 8. Hot and cold rooms

A hot or cold room changes **the temperature of its cells and nothing else** (decisions 9 and 10). It gives no status, deals no damage, and touches no items or food. Whatever the temperature does is the **environment simulation's** job, so a hot room feels exactly like any other hot place, and every improvement to the environment reaches these rooms with no room-side code.

| | Hot room | Cold room |
|---|---|---|
| id, name | `hot`, "sweltering chamber" | `cold`, "frigid hall" |
| enter | "A wave of heat meets you." | "Your breath mists in the bitter cold." |
| glow | faint red `[30, 10, 0, 0]` | faint blue `[0, 8, 22, 0]` |
| `temperature` | +25 °C over the level's | −25 °C under the level's |
| weight | 5 | 5 |

| Rule | Proposal |
|---|---|
| The field | `temperature` (−40 to 40 °C, default 0) on an effect, added to the level's |
| The one place | `WorldState::temperature_on(level)` gains `temperature_at(level, pos)` (the level's plus `effects_at(pos)`'s), the one place a cell's temperature is decided. Every reader that wants the temperature at a place asks it. |
| What reads it today | **perception**: the status bar shows the temperature of the player's cell, so stepping in shows the change (the effect's only number, which the game already shows). **Ice**: `advance_ice` groups cells by (water, thickness, temperature), so a cold room's water freezes on a 10 °C level and a hot room's ice melts. A generated cold room in a cave with water is a frozen pool; Winter-born's footing applies. |
| Not whole-level | a hot or cold level is a `temperature` in its file and needs no `level_chance` |
| Cost | S–M: the per-cell temperature and the ice grouping. Risk: the ice tick's per-group cost; a test checks a year's catch-up still runs in its budget. |

**What the environment does today, and what is missing.** Today temperature is shown and freezes or melts water; nothing else feels it (`docs/dev/temperature.md`: "Nothing feels the cold yet"). Missing, as **separate future work outside this design** (its own design and issues, under the temperature topic), each of which hot and cold rooms would then get for free through `temperature_at`:

| Missing | What it would do |
|---|---|
| Creatures and the player feel heat and cold | statuses by temperature band (chilled, sweltering, and worse at the extremes, with damage), resisted by fire and cold resistance and by clothing; slower healing, hunger in the cold; a talent like the dropped *Hardy traveller* belongs there |
| Items | potions and water freezing, scrolls and cloth scorching at furnace heat |
| Food | rot by temperature (a cold larder, a hot rot-room); `crate::rot` is pure in age today |

## 9. More rooms

All thirteen are built (decision 16), each as its own issue (§12). They are listed in the order they are planned, by fun per cost.

### 9.1 Silent vault

| | |
|---|---|
| id, name, mark | `silent`, "silent vault", `v` |
| Rule | `noise = 1` (every noise inside at 1 %) and `status = "silenced"`: a new status (*Silenced*, `seen = "silenced"`, `max = 1`, `eases = 10`, `blocks = ["cast", "read", "throw_voice"]`). Casters' spells, scrolls, and the bard's voice are refused inside, creatures' too. |
| Messages | `enter` "Your footsteps make no sound at all." Status start "The silence closes around you.", end "Sound returns." |
| Why | turns a fight upside down: the wizard and the kobold shaman fight with sticks; a thief's paradise |
| Fields | `noise = 1`, `status = "silenced"`, `weight = 5` |
| Cost | S on §3.3. Risk: a caster cornered inside; they can walk out. |

### 9.2 Black chamber

| | |
|---|---|
| id, name, mark | `black`, "black chamber", `b` |
| Rule | `light = 0`: every light inside is kept at 0 %, carried ones too; no glow. Only darkvision sees here. |
| Messages | `enter` "The dark here swallows your light." |
| Fields | `light = 0`, `weight = 5` (data only, today's fields) |
| Cost | none. Ships with R1. |

### 9.3 Shrine of a side

| | |
|---|---|
| id, name, mark | `shrine`, "shrine", `h` |
| Rule | a side drawn at generation, good or evil (1 : 1), as an altar's (§7.1, `Room::side`), with the same light by side. For one whose band agrees: healing quickened (`regen = 200`). For one opposed: the status *Unwelcome* (`regen = 50`, `combat = { to_hit = -1 }`, `eases = 30`). For the neutral: nothing. Everyone inside, creatures by their own alignment. |
| Messages | `enter` (agreeing) "You feel welcome here."; (opposed) "Something here wants you gone."; (neutral) "An old shrine. It ignores you." |
| Fields | `shrine = true`, `weight = 4` |
| Cost | S: one fold in `effects_at` by the dweller's band (`alignment::Requirement`). Depends on R5 for the side and its light. |

### 9.4 Scriptorium

| | |
|---|---|
| id, name, mark | `scriptorium`, "scriptorium", `r` |
| Rule | `feature = "shelves"` (a new terrain, stone, "shelves of rotting scrolls"); 2 to 4 **scrolls** on the floor (`loot = { class = "scroll", count = [2, 4] }`, from `swarm/<level id>`); reading inside trains Decipher script twice as often (one factor at the training hook) |
| Messages | `near` "You smell dust and old vellum."; `enter` "Shelves of rotting scrolls line the walls." |
| Fields | `feature`, `loot`, `train = { decipher_script = 200 }`, `weight = 5`, `min_cells = 9` |
| Cost | S on R5's `feature`. |

### 9.5 Beast den

| | |
|---|---|
| id, name, mark | `beast_den`, "beast den", `d` |
| Rule | a teeming den (§4) whose sleepers are beasts only: the factions `wildlife`, `vermin`, and `marsh` (`swarm.factions`; still drawn from the branch's population, so a branch without beasts never gets one: the den kind is then not eligible); its hoard is bones and fresh corpses (food) instead of items (`swarm.hoard = "corpses"`) |
| Messages | `near` "You smell musk and old blood."; `enter` "Heavy shapes breathe in the dark around you." |
| Fields | `swarm = { …, factions = ["wildlife", "vermin", "marsh"], hoard = "corpses" }`, `needs_danger`, `min_cells = 16`, `weight = 5` |
| Cost | S on §4. The ranger's and Beast friend's room. |

### 9.6 Unstable room

| | |
|---|---|
| id, name, mark | `unstable`, "unstable room", `u` |
| Rule | every step inside, the stepper's (player or creature) has `warp` (10 %) to be teleported as the teleport trap does (`world/trap.rs`, a gameplay draw) |
| Messages | `enter` "The floor shimmers under your feet."; on a warp, the trap's own line |
| Fields | `warp = 10`, `max_cells = 60`, `weight = 4` |
| Cost | S: the trap's teleport reused. Fleeing through it is a gamble. |

### 9.7 Tempting hoard

| | |
|---|---|
| id, name, mark | `hoard`, "treasure room", `$` |
| Rule | `hoard.items` (3) items at item level + `hoard.loot_level` (3) on the floor, from `swarm/<level id>`. The first item lifted (`pick_up`, one line) **seals the room**: every door of it locks (a lock of the floor's level, pickable and kickable, `doors.rs`) and `hoard.guardians` (2 to 4) hostiles of danger + 1 appear at the room's edges, out of view if they can be. `Room::sealed` (saved, hashed). A cave blob has no doors: only the guardians. |
| Messages | `near` "Something glitters ahead."; `enter` "Treasure lies here, unguarded."; sealing "The doors slam shut!" |
| Fields | `hoard = { items = 3, loot_level = 3, guardians = [2, 4], danger = 1 }`, `needs_danger`, `weight = 5` |
| Cost | M. The classic greed check; doors, locks, kicking, and Pick lock all come into play. Hoard-wise (§11). |

### 9.8 Haunted room

| | |
|---|---|
| id, name, mark | `haunted`, "haunted room", `g` |
| Rule | while the player **rests** inside, at every full hour of rest an undead of the floor's danger (the cursed room's undead pick, §5) appears inside, out of view if it can be; the rest stops as for any arrival |
| Messages | `near` "You hear a whisper, and then nothing."; `enter` "It is very quiet here. Too quiet to sleep." |
| Fields | `haunt = { every = "1h" }`, `needs_danger`, `weight = 5` |
| Cost | S–M on R7's undead pick. A fine camp spot that is not. |

### 9.9 Heavy hall

| | |
|---|---|
| id, name, mark | `heavy`, "heavy hall", `w` |
| Rule | `burden = 50`: carrying limits are halved inside, for everyone, through the one burden hook (`docs/dev/burden.md`); heavy armour turns Burdened |
| Messages | `enter` "Your pack drags at you, as if the stones pulled it down." |
| Fields | `burden = 50`, `weight = 4` |
| Cost | S. |

### 9.10 Healing spring

| | |
|---|---|
| id, name, mark | `spring`, "spring", `~` |
| Rule | `feature = "spring"` (a new terrain, a stone basin of water, not walkable). **Drink** (*Here*, §3.8; `Action::Drink`, `drink` in action text) takes 10 s and draws from `data/outcomes/spring.toml`: Refreshed (2d8 hit points, weight 55), nothing (25), Fouled (a disease caught, one `prayer` cures: marsh ague, 15), Clear-headed (half the mana back, 5). No limit (`exhaust = 0`). |
| Messages | `enter` "Clear water wells up in an old stone basin."; outcomes "The water is cold and good." / "It tastes of stone." / "The water tastes foul." / "Your head clears." |
| Fields | `feature = "spring"`, `outcomes = "spring"`, `weight = 4`, `min_cells = 9` |
| Cost | S on R5's outcome tables and *Here*. |

### 9.11 Hall of mirrors

| | |
|---|---|
| id, name, mark | `mirrors`, "hall of mirrors", `m` |
| Rule | every actor inside that the player sees is shown with `mirrors.images` (2) **reflections** on free floor of the room. Their cells come from a hash of (level, turn, actor), not a stream, so the rules and perception compute the same images without drawing (`World::images`). Perception shows a reflection as the creature it reflects, under a synthetic id: nothing tells it from the real one. A melee blow or a step into a reflection's cell: "Your blow meets only glass." (the blow's time). A missile or bolt aimed at a reflection flies on through its cell. Creatures shooting at the player from outside the room pick the player or one of their images (a gameplay draw); creatures inside know the hall and are not fooled. The images vanish once the actor leaves. |
| Messages | `enter` "Your reflection stares back at you from every side." |
| Fields | `mirrors = { images = 2 }`, `max_cells = 80`, `weight = 3` |
| Dumps | `actors` (dev) marks images; `view` shows them as the player does |
| Cost | L: the images in perception without leaking which is real, targeting and bumping images, creatures' ranged pick. Mirror-wise (§11). |

### 9.12 Forge

| | |
|---|---|
| id, name, mark | `forge`, "forge", `f` |
| Rule | `feature = "anvil"` (a new terrain, "an old anvil, its hammer chained beside it"). **Temper** (*Here*, §3.8; `Action::Temper { item }`, `temper:LETTER` in action text) works one carried weapon or armour for 10 minutes, then draws from `data/outcomes/forge.toml`: `improved` (an affix of the item's class added, up to its rarity's count, else its strongest affix a step up), `nothing`, `flawed` (one affix lost), `ruined` (the item breaks and is gone). Base weights 35 / 30 / 25 / 10; a **Metalcraft** check leans them (its margin moves up to 20 points from `ruined` and `flawed` to `improved`, or the other way). Tempering identifies the item and trains Metalcraft. No limit (`exhaust = 0`): the rarity's affix count is the natural cap. |
| Messages | `enter` "A cold forge, and an anvil waiting."; outcomes "The metal rings true." / "Nothing changes." / "Something in the metal is lost." / "It shatters under the hammer!" |
| Fields | `feature = "anvil"`, `outcomes = "forge"`, `weight = 3`, `min_cells = 9` |
| Cost | M on R5: the item list (the identify list's), affix add/raise/remove (`magic-items.md`). |

### 9.13 Collapsed floor

| | |
|---|---|
| id, name, mark | `collapsed`, "collapsed floor", `c` |
| Rule | `floor = "cracked_floor"` (a new walkable terrain, "cracked floor", shown as such). Each step of the **player** onto a cracked cell has `collapse.chance` (15 %, doubled when Burdened or worse; a gameplay draw) to give way: the player falls to the next level of the branch, landing on a random free floor cell (never a vault or a shop), takes `collapse.damage` (2d6, `hurt` with `Harm::Spared`, never the last hit point), and is Staggered. The level below is generated as usual; the fall is a new arrival kind beside the stairs (`Arrival::Fell`). Creatures do not fall. |
| Messages | `near` "The floor ahead is cracked and sagging."; `enter` "The floor creaks under your weight."; falling "The floor gives way beneath you!" |
| Fields | `floor = "cracked_floor"`, `collapse = { chance = 15, damage = "2d6" }`, `needs_below = true`, `weight = 3` |
| Cost | L: falling between levels (a new arrival, the level travel code). Light tread (§11). |

## 10. What the player sees, and the wiki

| Where | What |
|---|---|
| Messages | `near` once per room when first seen; `enter` once per room when first entered; `pervade` on every arrival on a pervaded level; the rooms' own lines in §4 to §9; the statuses' start and end |
| Status bar | the statuses (*Ebbing*, *Silenced*, *Unwelcome*) and the temperature of the player's cell |
| Look mode | "in a teeming den" (once entered); "a goblin (hostile, asleep)"; "a forgotten altar", "a spent altar", "a spring", "an anvil", "shelves", "cracked floor", each with its own description |
| The abilities menu | *Here*: Pray at a forgotten altar, Drink at a spring, Temper at an anvil, listed only there |
| Never shown | weights, chances, wake thresholds, the altar's side in words, an altar's exhaustion chance (only that it went dark), rise timers, which undead will rise, which image is real |
| Dumps | `rooms` (dev): each room's effect, its side, `glimpsed`/`entered`/`sealed`, and `pervading <id>` in the header; `actors` (dev): `asleep`, the pending risings with their times, mirror images; `abilities` lists the *Here* abilities; JSON: `room` as today, `pervading` (only once arrived, as the player knows it) |
| Action text | `pray`, `drink`, `temper:LETTER` |
| Wiki | a **Special rooms** page (`wiki/src/`, by topic beside traps) with generated tables (`xtask/src/wiki/rooms.rs`): every feature with what it does in exact numbers, its default weight, its share, its whole-level chance, and the per-floor odds; each dungeon's favoured and banned features; the outcome tables (altar, spring, forge) with weights, tiers, the lean, the side colours, the exhaustion chance, and the shares per prayer and per altar (nothing, minor, moderate, great, the explosion), computed from the data; the den numbers; the risings table (which undead each level becomes) |

## 11. Talents and class abilities

Two new systems join `data/talents.toml`, placed by topic (after `danger`, before `traps`).

| System | Talent (tree) | Effect | Hook (one read) | Issue |
|---|---|---|---|---|
| `special_rooms` | **Sense of place** (wayfarer, free) | a room's nature is known when glimpsed (look names it before entering); a pervading effect's line says what it is | `PerceptionView::room` (glimpsed counts as entered) | R1 |
| `special_rooms` | **Hallowed rest** (grit) | where a room quickens healing or mana, it quickens them half again more for you | `World::healing_rate`, `World::mana_rate`: the room factor | R1 |
| `special_rooms`, `mana` | **Anchored mind** (scholar) | Ebbing never takes you | `World::dwell` (immune to that status) | R2 |
| `special_rooms`, `stealth` | **Hush-foot** (shadow) | sleepers need twice the noise to wake from what you do | `World::hear`'s wake check (`swarm.wake` × 2 for the player's noises) | R4 |
| `special_rooms` | **Grave warden** (guard) | what you kill never rises | `fight.rs::dies`' rise line | R7 |
| `special_rooms` | **Hoard-wise** (delver) | a tempting hoard seals only after the second item lifted | the hoard trigger | R14 |
| `special_rooms` | **Mirror-wise** (scholar) | reflections within 2 cells are seen for what they are | `World::images` for the player's perception | R18 |
| `special_rooms` | **Light tread** (delver) | cracked floors never give way under you | the collapse draw | R19 |
| `prayer` | **Devout** (wayfarer) | the lean counts double at agreeing altars, and an opposed one leans as neutral | the prayer draw's lean | R5 |
| `prayer` | **Penitent** (grit) | a great bane becomes nothing | the prayer draw's outcome | R6 |
| class ability (`special_rooms`) | **Consecrate** (priest 9, paladin 12; `requires_alignment = { min = "good" }`; 15 mana; one minute) | see below | `LevelState::rooms`/`pervading`; `Deed::Consecrated` +20 | R7 |

**Consecrate, plainly** (decision 12). A good priest or paladin spends 15 mana and a minute of prayer standing inside a cursed, draining, or haunted room. The room's curse is lifted **for good**: it becomes a plain room, so corpses no longer rise there, mana no longer drains, and nothing comes while one rests. On a level that a curse or a drain covers entirely, it lifts the whole level's effect instead. Doing it moves the character's alignment toward good (+20). Anywhere else it is refused with no time spent ("There is nothing here to consecrate."). It gives the priest a power tied to places that no other class has.

`special_rooms` ships in R1 with its first two (Sense of place, Hallowed rest: both on today's hooks), so the coverage bar holds from the start; the rest come with their issues. `prayer` ships with R5 and Devout; Penitent comes with R6's great banes. `min_class` is 0, so Consecrate is not required by the bar. *Hardy traveller* (immunity to room heat and cold) is dropped: rooms no longer give temperature statuses (§8), and such a talent belongs to the future environment work.

## 12. Implementation plan

Each item is one issue, one branch, one PR, updating the spec (§7 "Special rooms", the statuses), **`docs/dev/special-rooms.md`** (decision 13: a new topic file created in R1 and listed in CLAUDE.md's topic index there; every later issue edits only it, not `docs/dev/mapgen.md`, so special-room PRs do not conflict with generator PRs), and the wiki in the same PR. "Save" means it adds saved state; "act" means it adds an action. Sizes: S about a day, M two to three, L a week.

| # | Issue title | Sections | Adds | Depends on | Size |
|---|---|---|---|---|---|
| **R1** | Special rooms: features with default weights, `favoured`/`banned` per dungeon (retire `effects`, move Fennmynster), shared chance 12 %, the whole-level version (`level_chance`, `pervading`, `effect` in level files), `effects_at`, `near` hints, eligibility; `docs/dev/special-rooms.md`; the `special_rooms` system with Sense of place and Hallowed rest; the black chamber | §3.1, §3.2, §3.5, §3.6, §9.2, §11 | save | — | M |
| **R2** | Special rooms: statuses while inside (`status`, `World::dwell`); the draining room and Anchored mind | §3.3, §6 | — | R1 | S |
| **R3** | Hot and cold rooms: per-cell temperature (`temperature_at`), ice by temperature | §8 | — | R1 | S–M |
| **R4** | Teeming dens: the `asleep` status and `sleeps`, waking by noise, blow, and cry, the den filler and hoard (`Rng::swarm`), Hush-foot | §4 | — | R1 | M |
| **R5** | Forgotten altars and prayer: `feature`, the `forgotten_altar` and `spent_altar` terrains, the side and its light, `Source::Place` (*Here*), outcome tables (`data/outcomes/`: tiers, `messages`, `exhaust`, `spent`), `Action::Pray`, exhaustion, the "nothing happens" lines, the minor and moderate boons and banes (`loot::forget` for Muddled), the altar's explosion, `Deed::Prayed`; the `prayer` system with Devout | §3.4, §3.8, §7 | save, act | R1 | M–L |
| **R6** | Prayer's great boons and banes: Gift, Exalted, Chosen, Relic, Wrath, Unmade, Forsaken, Damned (`Actor::boons`, `World::lose_level`, the rarity floor); Penitent | §7.4, §11 | save | R5 | M |
| **R7** | Cursed rooms: risings (`LevelState::risings`, `rise_tick`), Grave warden; Consecrate | §5, §11 | save, act | R1; class tables for Consecrate | M |
| **R8** | Silent vault (the `silenced` status) | §9.1 | — | R2 | S |
| **R9** | Shrine of a side (*Unwelcome*) | §9.3 | — | R2, R5 | S |
| **R10** | Scriptorium (`shelves`, scroll loot, Decipher script training) | §9.4 | — | R5 | S |
| **R11** | Beast den (faction filter, corpse hoard) | §9.5 | — | R4 | S |
| **R12** | Unstable room (`warp`, `max_cells`) | §9.6 | — | R1 | S |
| **R13** | Heavy hall (`burden`) | §9.9 | — | R1 | S |
| **R14** | Tempting hoard (sealing, guardians), Hoard-wise | §9.7 | save | R1 | M |
| **R15** | Haunted room (hourly haunting during rest) | §9.8 | — | R7 | S–M |
| **R16** | Healing spring (`spring`, Drink) | §9.10 | act | R5 | S |
| **R17** | Forge (`anvil`, Temper, Metalcraft lean) | §9.12 | act | R5 | M |
| **R18** | Hall of mirrors (images, Mirror-wise) | §9.11 | — | R1 | L |
| **R19** | Collapsed floor (`cracked_floor`, falling to the level below, Light tread) | §9.13 | save | R1 | L |

**Waves** (at most about three PRs at once that add actions or save state):

1. **R1** alone (it changes the special-room draw and the Depths' snapshots; nothing should race it).
2. **R2, R3, R4, R5** together. R5 is the only one with save state and an action; R2 and R4 add statuses (`data/status.toml` is a hot file: each adds its entries by group).
3. **R6, R7** (save, act) with **R8, R11, R12, R13** (data and small rules, no save state).
4. **R9, R10, R14** (save), **R16** (act).
5. **R15, R17** (act).
6. **R18, R19** (save), the two large ones, last.

**Tests each item carries**: `content/room_effect.rs` (new fields parse and validate); `content/dungeon.rs` (`favoured` and `banned` resolve and layer; `effects` is refused; the share sum and the all-banned dungeon are errors); `mapgen/special_tests.rs` (the shares over 1 200 levels within bounds, with and without favoured; a new feature added in a test table is drawn in a dungeon that never names it; `level_chance` near 5 % over many draws; eligibility; a den's count and hoard; an altar's feature placed and the level still connected); `world/rooms_tests.rs` (`effects_at` folding; `dwell` gives and eases; Ebbing drains a creature; Silenced refuses a cast; ice freezes in a cold room and not beside it; temperature on the status bar); `world/den_tests.rs` (a sneaking step wakes nobody, a kicked door wakes the near ones and the cascade the rest, sleepers never wake by time, sneak attacks land on sleepers, sleepers count toward the spawn cap); `world/curse_tests.rs` (rises after two minutes, not if the corpse is eaten or carried, the level pick and its nearest-level fallback, Grave warden, a pervaded level, frozen floors catch up, Consecrate lifts it); `world/outcome_tests.rs` (*Here* lists Pray, Drink, Temper only at their features; an interrupted prayer gives nothing and never exhausts; the lean by side and band; each outcome's hook, the great ones included, and each "nothing happens" line changes nothing but the message log; the tier shares over many draws within bounds of §7.4's table; exhaustion near 50 % over many prayers, the spent altar offers no Pray and survives a save and load as terrain alone; the explosion hurts everyone in radius 2, the player too, can kill, makes the explosion noise, melts ice, leaves rubble, and skips the exhaustion draw; Penitent turns a great bane into nothing but not the explosion; the alignment deed; the spring and the forge, `exhaust = 0`, are never spent); one test file per §9 room; screen snapshots for the abilities menu's *Here* group and look mode's "asleep"; a replay per action (`replay --create`, `--verify`); `cargo xtask wiki`. No golden: the glows are light, and the light tests and a `--capture` checked by eye cover them; mirror images are checked through perception and dumps.

## 13. Decisions (owner, 2026-10-26)

| # | Question | Decision | Sections changed |
|---|---|---|---|
| 1 | Is 5 % the right `level_chance`? | Fine; adjust later if need be. | §3.1 |
| 2 | Whole-level versions of shadowed, hushed, echoing? | Not now. | §3.1 |
| 3 | The den: data or procedural? | Keep the design: the effect is defined in data, the rooms are placed and filled by the generator. A plain explanation of which is which added; the principle stated (no hand-made den definitions per monster: sleepers always come from the branch population's draw, rarity, strays, out-of-depth, danger, later tribes); four example dens; optional themed dens (a warren of one type, a tribe of one faction) recommended. | §4, §9.5 |
| 4 | Alignment for killing sleepers? | None. | §4 |
| 5 | Sleepers waking on their own? | Never. | §4 |
| 6 | Prayer length (was 30 minutes)? | Short: one minute. The tension is the chance of a very bad outcome, so the banes are made genuinely nasty (lightning that can kill, an out-of-depth creature, deadly disease, the most valuable item lost). | §3.8, §7.3, §7.4 |
| 7 | The jackpots (+1 stat, −1 potential)? | Keep, but far stronger now, toward ADOM's significance: four great boons (up to +3 to a stat, a level, a free talent for life, a relic) and four great banes (up to −3, a level lost, a flaw for life, a horde). Making them bigger still stays deferred on #196. | §7.4, §11, §12 (R6) |
| 8 | The altar's side by light colour? | Explained plainly (what the side is, how it leans the odds, how the colour shows it); recommendation kept. Framing: the altar is ADOM's pool mechanic with a different flavour, not its altar and god system: no gods, no sacrifices, no conversion. | §7, §7.2 |
| 9 | Should hot and cold rooms deal damage? | No room-side damage or statuses: the room sets its cells' temperature only; effects come from the environment simulation. What it does today and what is missing listed as separate future work. | §1, §3.3, §8, §10, §11 |
| 10 | Rot in cold and hot rooms? | Up to the environment simulation, not the room. | §8 |
| 11 | Who suffers room statuses? | Everyone. | §3.3 |
| 12 | Consecrate? | Yes, accepted as written; what it does explained plainly. | §11 |
| 13 | Where the developer notes go | A new topic file, `docs/dev/special-rooms.md`, listed in CLAUDE.md's index in R1, to reduce merge conflicts. | header, §12 |
| 14 | Pray outside these rooms? | No. The feature's altars get their own terrain and description. | §3.4, §3.8, §7.1 |
| 15 | The weight table | No global table every dungeon restates: each feature has a default weight in its own data; a dungeon may name `favoured` features with fixed shares and a `banned` list; a new feature appears everywhere automatically. Fennmynster's `effects` mapped to it. | §1, §3.2, §3.6, §12 (R1) |
| 16 | Which §9 rooms to build | All of them, each fully specified and planned. | §1, §9, §11, §12 |
| 17 | A curse with no undead of the dead's level | The nearest level. | §5 |
| 18 | The range of prayer outcomes (2026-10-26, later) | Many "whatever" level boons and banes, some "nothing happens" outcomes; a big range of things that can happen, but no altar swings the game for sure. The table becomes 40 outcomes in three tiers (minor, moderate, great) plus six "nothing happens" lines: per prayer 30 % nothing, about 44 % minor, 22 % moderate, 4 % great. | §1, §3.8, §7, §7.4, §10, §12 (R5) |
| 19 | Limits on praying (2026-10-26, later) | Each prayer has a chance, 50 %, to exhaust the altar (`exhaust`, data, default 50). The spent altar stays on the map as its own terrain; Pray is no longer offered. This replaces the "no limit on prayers" of decision 6's draft for altars only; the spring and the forge keep no limit (`exhaust = 0`). Saved state: the terrain swap alone. | §1, §3.8, §7.1, §7.3, §10, §12 |
| 20 | A special outcome (2026-10-26, later) | The altar explodes: a fireball at its cell (the existing burst code: 3d6 fire, radius 2, everyone in it including the player, can kill, the explosion noise, melts ice), and the altar is removed from the map (it becomes rubble). A moderate bane, 1.75 % of prayers with no lean. | §7.4, §7.5, §12 (R5) |
