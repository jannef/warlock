# Warlock — Generic vaults, way more of them

**Built; this file is history.** Current behaviour is in spec §7 "Vaults" and `docs/dev/mapgen.md`.

Design document for the owner's review (#961, part of #957). First pass, 2026-11-05, written against `origin/main` at d562ceb5. Nothing here is implemented; §8 cuts it into issues. Every rule quoted below was read from the files named; anything not checked is marked *(unchecked)*. Second pass, 2026-11-05, after the owner's approval, checked against `origin/main` at 5b3326f9: the local areas' ruins come by a chance per landscape, not always (§1, §4), random orientation has landed with its real syntax (#959, §2), and what landed since (deep water #1019/#1020, zones #1000, stair followers #985, the endless branch #1010) is placed in §1 and §2; §9 records the decisions.

**The owner's request** (#957, verbatim): the vaults are *"way too monotonous and appear way too often"*, and *"we need way way more of generic vaults"*; random contents rather than static piles, along the lines of *"20 monsters from dungeon monster pool"* or *"10 monsters from goblin tribe"*. And on proposals: dull and samey ones are rejected. So the standard this document holds every vault to: **a vault is a small situation with a decision in it**, not a room with loot and monsters in another shape. Each entry in the catalogue names its decision in one line; a vault with none was cut.

**What was checked.** Every catalogue map below (31 files) was extracted, uncommitted, into a scratch copy of `assets/` beside the six that exist and loaded through `warlock-cli gen --content-dir` (the loader runs `Vault::new` on every vault file, so the edge, entrance, reachability, key, lock, chest, pool, and loot checks of `content/vault.rs` and `content/level.rs` ran on each), until it loaded clean; then Depths, Cealdhlāw, Fyrdelf, and Wulfeshol floors were previewed with them in the draw. The loader caught five things the first draft had wrong, each now in §2's toolbox: `household` and `food` are no loot classes; `scrolls`, `wands`, and `money` take no `magic` grade; a `fey` pool holds nothing (§6, F9); and a ring's south stone stood on its door. Nothing was committed but this file.

## Contents

1. How many, and why
2. The author's toolbox: what the format gives today, and what it refuses
3. The catalogue: dungeon vaults
4. The catalogue: outdoor vaults (the local areas)
5. The six that exist
6. Vaults that need a feature first
7. The vault rules for authors
8. Implementation issues
9. Decisions

## 1. How many, and why

### How often a vault comes now (#991)

| Floor | Chance of a random vault | Source |
|---|---|---|
| rooms floor (80×21) | 20 % | `mapgen/rooms.rs::VAULT_CHANCE` |
| cave floor (100×40, 120×60) | 40 %, then 15 % for a second | `mapgen/caves.rs` |
| a local area of the overworld | by its landscape: hills 30 %; grassland, forest, and plains 15 %; dense forest and swamp 10 %; then one outdoor vault if one fits | `mapgen/local/mod.rs::features` (`ruins`), `build`: `if rng.chance(features.ruins) { ruins(..) }` |

A road's area is generated as its neighbours' most common land (`Setting::read`), so it takes that land's chance; river and lake cells have no local area. When the chance comes up, `ruins` shuffles every outdoor vault, turns each (#959), and tries each 30 times for a spot whose centre is at least 24 cells from the area's centre (rows counted double), with a ring of free ground round it and no water under it or its ring; the first that fits is stamped and the rest are not tried. Outdoor vaults are not dealt into phases: each area shuffles all of them anew.

A branch deals its random vaults round four phases by the run's chance (`mapgen::spaced_vaults`, `VAULT_PHASES` 4): floor `depth` may only draw the vaults of phase `depth % 4`, so no vault comes back within four floors of itself in a branch. **Adding vaults raises what a floor may choose from, not the chance of one** (docs/dev/mapgen.md). Pinned vaults (a branch level's `vaults`) are outside all of this and untouched by this design.

### What a player sees over a run

The Ascmearc's branches (`campaigns/ascmearc/data/dungeons/`): Lytelscræf 3 cave floors; Cealdhlāw 2 rooms + 4 caves; Fennmynster 8 rooms; Fyrdelf 4 rooms + 12 caves; the four lairs Bānfeld and Brocenburg 4 rooms each, Wulfeshol and Wyrtholm 4 caves each. 22 rooms floors and 27 cave floors in all.

| | Rooms floors | Cave floors | Random vaults expected |
|---|---|---|---|
| a full run, every branch to its bottom | 22 × 0.20 = 4.4 | 27 × 0.46 = 12.4 | **about 17** (plus the 9 pinned ones) |
| a run that skips the four lairs | 2.4 | 8.7 | about 11 |
| Grundlēas, the endless branch (#1010) | 85 % of its floors (`[dungeon.endless] caves = 15`) | 15 % | about **one in four floors** (0.85 × 0.20 + 0.15 × 0.46 ≈ 0.24), without end |

Grundlēas deals its vaults anew on every floor under that floor's own seed (docs/dev/mapgen.md), so there the phase deal does not space repeats: one vault may come on two floors in a row. Its floors are never remembered, so no one can compare them, but a deep dive is also where a small catalogue shows most. It is one more reason for the count, not a reason for a different one.

**Local areas.** A local area is entered by choice (`>` on an overworld cell) or by an encounter fight that opens in the cell's area (docs/dev/encounters.md); nothing makes a player enter any. Of the Ascmearc's 5,010 overworld cells that have a local area, 1,315 are grassland, 1,297 plains, 793 forest, 592 hills, 534 dense forest, 335 swamp, and 144 road, so an area picked at random has a ruin about **one time in six** (16 %, weighted by those counts; the hills, Ascdūn's among them, twice that, the swamp of Micelfenn and the dense woods two thirds of it). A player who enters 30 areas over a run meets about 5 ruins; one who enters 60, about 10.

Today three shared vaults stamp in dungeons (Flooded cell, Guard post, Shrine; the other three are outdoor), so those 17 draws are 17 visits to three rooms: with the phase deal the same one comes every four floors of a branch, and every branch starts over. The owner's "appear way too often" is this. The outdoor side has three ruins (Ruined chapel, Ruined tower, Stone circle) for those 5 to 10 meetings: fewer ruins than the dungeon side has draws, but from the same three every time.

### The target

| Group | Now | Target | Why that number |
|---|---|---|---|
| shared dungeon vaults | 3 | **48** | 17 draws from 48 (12 a phase): the chance that any two draws of a run are the same vault is about 136 pairs × 1⁄48 ≈ 2.8 repeated pairs, so a full run sees about 14 *different* vaults of its 17, and a repeat has turned (#959) and rolled its pools, so it is not the same room. At 24 it would be 5.7 repeated pairs; at 72, 1.9: the curve flattens past 48, and 48 is three content batches of sixteen |
| shared outdoor vaults | 3 | **9** | about 5 ruins a run (30 areas entered), 10 for a player who explores (60). The number of *different* ruins met, N × (1 − (1 − 1⁄N)^k) for k ruins from N vaults: at 5 ruins, 2.6 of 3 today, 4.0 of 9, 4.2 of 12; at 10 ruins, 3.0 today, 6.2 of 9, 7.0 of 12. Nine gives a 5-ruin run four different ruins of five (80 %, the dungeon side's 85 % at 48), and twelve adds under one different ruin a run for three more files. Outdoor vaults have no pools (§2), so their variety is in shape and turn alone (#959 turns them too), which is why the count is not lower |

The catalogue below has **25 dungeon vaults drawn in full** (§3), **9 outdoor** (§4: six for the target, three marked optional, kept and not cut), **the 6 existing** (§5), and **9 that wait on a feature** (§6), with a dozen sketched for the third batch (§3, at the end): 49 dungeon + 9 outdoor when every batch is in (12 outdoor with the optional three). No cap per run and no weights are proposed: the phase deal already spaces repeats, and a weight would be a hidden number to tune for no visible gain. If the count ever makes repeats rare enough that the deal is pointless, that is a later owner call.

### Depth

Where a vault suits is in its row. **A shared vault cannot pick its depth today**: `dungeons` on a shared vault may name no campaign's branch (`Vault::check_dungeons`: "a vault stamps only in its own campaign's branches"), so a shared vault stamps on every floor of every branch. The only depth control is the pool itself: a tag pool draws at the floor's danger and brings nothing when the tag's shallowest type is above it (`danger::draw` weighs a type above the danger 0; the brief: "a tribe with nothing at the floor's danger leaves its cells empty"). So the catalogue's deep vaults use pools with a shallow member (a bridge kept by `["orc", "ogre", "troll", "giant"]` is an orc on floor 3 and a giant on floor 20), and the few that are empty rooms above a depth (Old Enemies, The Conventicle) are the reason for the one small feature this design asks for first: a vault file's `danger = { from, to }` (§6, F1). Until it lands those two ship anyway: an empty hall of a vault is a quiet room with a find in it, and the player learns nothing from it.

## 2. The author's toolbox

What a vault file can say today, read from `content/level.rs` (`LegendDetail`), `content/vault.rs`, `content/monster_pool.rs`, and `magic/loot.rs`. The catalogue uses nothing outside this table.

| Thing | Syntax | Notes |
|---|---|---|
| terrain | `"x" = "floor"` or `{ terrain = "...", ... }` | the shared legend (`data/legend.toml`) gives `#` wall, `.` floor, `+` closed door, `'` open door, `~` still water, `_` altar (walkable, glows), ` ` void |
| walkable ground with a rule | `shallow_water` (1.5× time, loud), `water` (2× time, loud, cold if frozen), `rubble` (1.5× time, clatters), `dusty_floor` (steps deadened: 70 % noise), `fungus` and `uv_rune` (glow), `cracked_floor`, `flagstones`, `plank_floor` (roofed) | `data/terrain.toml` |
| fixtures (not walkable) | `standing_stone`, `statue`, `shelves`, `weapon_rack`, `workbench`, `anvil`, `brazier` and `sconce` (lit), `crystal` (lit), `lava` (lit, never walkable), `packed_earth` (dug through in a minute), `granite` (half an hour, no blast), `timber_wall`, `dead_tree`, `boulder`, `spent_altar`, `forgotten_altar`, `spring`, `fountain` | an anvil, spring, or forgotten altar in a vault **offers nothing**: Pray, Drink, and Temper are offered only beside a terrain a *room effect's* `feature` names (`world/pray.rs`, `drink.rs`, `temper.rs`), and a vault cannot mark special rooms. They are decoration in a vault |
| a placed light | `light = "lamp"` (`torch`, `lantern`, `brazier`, `ember`, `wisp`, `crystal_red` …) | `data/lights.toml`; a lit vault is seen from its door, and its people see the player come |
| a find | `loot = "any"` / `"magic"` / `"rare"`, or `loot = { pool = ["armour", "rings"], grade = "magic" }` | one find a cell, rolled at the floor's item level; pool names are item class ids or the groups `weapons`, `armour`, `jewellery`, `money`; on walkable terrain only. **Only classes with base items roll**: weapons and armour (every class of both groups), `rings`, `amulets`, `coins` and `gems` (`money`), `potions`, `scrolls`, `wands`, `rods`, `missiles`, and `tools` (the pick-axe is its one base, docs/dev/magic-items.md); `food`, `household`, and `light_sources` are refused at load ("can hold nothing of grade Any at any item level"), so bread, pots, and candles are placed as `objects`, flavour and not a find. A `grade` above `any` is refused on a class no affix rolls on (`scrolls`, `potions`, `wands`: "can hold nothing of grade Magic at any item level"); `magic` and `rare` are for weapons, armour, and jewellery |
| a crowd | `monsters = { pool = "level", count = 7 }` or `pool = "goblin"` / `["orc", "troll"]` | drawn at the floor's danger, outside the population cap; `count` over the entry's marked cells (default one on each); on walkable non-door terrain; a cell of `monsters` takes no `creature` or `trap` too; **no `status`, `faction`, `carry`, or `route`** on a pool cell (those are a placed `creature`'s) |
| a placed creature | `creature = "kobold"`, with `route`, `carry`, `faction`, `alignment`, `status = ["asleep"]` | fixed, so it never scales: use one as a key-bearer or for flavour, never as the guard of a find worth having; no unique `npc`, no mimic |
| a trap | `trap = "spear"`, `known = true`, `dir = "S"` (a bolt) | `arrow`, `spear`, `fireball`, `frost`, `lightning_bolt`, `pit`, `alarm`, `teleport`; hidden by default; on ground, a door, or a shut chest (no pit on a chest) |
| a chest | `terrain = "chest"`, `locked = true`, `trap = "arrow"`, `objects = [...]` | of the floor's band, filled by its band's table unless `objects` says; a chest's lock wants no key (pick, Knock, kick) |
| a lock | `lock = "guard_post"` on a closed door | its key must lie, or be carried, within reach of an unlocked entrance without unlocking a door; keys are objects with `key = "id"` (`brass_key` → `guard_post`, `iron_key` → `pantry`, `rusty_key` → `depths`, `bronze_key` → `barrow_chamber`, `skeleton_key` → `*`), so a new lock id ships its key file; `locked = true` without an id wants a skeleton key in reach: avoid it |
| a secret door | `secret = true` on a closed door | dungeon vaults only |
| a named place | `name = "..."`, `note = "..."` | stamped with the vault; look mode says them |
| a pile | `objects = ["bones"]` on a cell, or `[[pile]] at = [x, y]` | prefer the legend form in a generic vault: the map is then the one place to read where things lie. Both turn with the vault (`content/vault_turn.rs` maps `objects` with the rest), so this is for the author, not for #959 |
| outdoor | `kind = "outdoor"` | void is the area's ground; every open edge cell is a way in; **no traps, chests, secret doors, or monster pools** (a local area has no danger) |
| shape | at least 3×3; a solid edge but entrance doors, never on a corner; everything walkable reachable from an entrance | a rooms floor is 80×21 with a one-cell ring of rock around the vault inside the inner 78×19: **at most 17 tall**; keep 9 or less to fit beside the rooms. Caves take 15×15 easily |
| orientation (#959, landed in #1035) | `turns = false` at the top of the vault file | a vault drawn at random lies any of eight ways (four turns, each mirrored or not) unless its file says `turns = false`, "for text that depends on direction or a layout that must not turn" (docs/dev/content.md); refused on a level; a pinned vault (a branch level's `vaults`) never turns. `ruins` turns outdoor vaults too. Terrain, creatures and their `route`, lights, `flow`, piles, traps and a bolt's `dir`, locks, chests, secrets, places, loot, and `monsters` all turn with the vault (`content/vault_turn.rs`). The catalogue has nothing that cannot turn, and nothing written that names a direction, so no file in it says `turns = false` |
| deep water (#1019, #1020) | `"D" = "deep_water"` in the vault's own legend (it has no shared glyph) | not walkable (`swim = true`): a trained swimmer may dive in, nothing else enters it. The vault's reachability check counts walkable cells and doors only (`Vault::new`), so nothing in a vault may be reachable only by swimming, and a find cannot lie in it (`loot` is for walkable terrain). The generators' deep pools never touch a vault (`mapgen/deep.rs`, docs/dev/mapgen.md). The catalogue uses none: its water is `water` (the shared `~`) and `shallow_water`, both wadeable, so the Mere, the Archers' Walk, the Well-house, and the Drowned Mill are wades, not swims |

Not in the toolbox: **zones** (#1000: Balefire, Reek, Mirk, Slick) are laid in play by spells and their scrolls (`LevelState::zones`); a level or vault file cannot lay one. A vault's finds may roll the four scrolls like any other (they joined the loot pools).

Who can leave a room: a creature without `opens_doors` cannot pass a closed door (`CreatureDef::opens_doors`, docs/dev/creatures.md); of the shipped types only the big and the clever have it (vampires, trolls, wight lords, the wild huntsman …). So **a closed door holds a crowd**, and an open doorway (`'`) does not: the catalogue leans on this. Once stamped, a pool's creatures are ordinary creatures of the floor: a hostile one beside the player when they take the stairs follows them (#985, `world/stair_followers.rs`; a den's sleepers only once woken), so a crowd fought at a vault's door can come down with a player who breaks off at the stairs. No vault has stairs, so this changes no map.

Tag pools and their shallowest member (from the creature files): `undead` 1, `skeleton` 1, `vermin` 1, `elemental` 1, `goblin` 2, `kobold` 2, `spirit` 2, `fey` 2, `orc` 3, `archer` 3, `caster` 4, `marsh-thing` 4, `fiend` and `devil` 5, `cultist` 6, `outlaw` 6 (the Ascmearc's), `demon` and `duergar` and `summoner` 8, `troll` 9 (the marsh troll), `hag` 11, `giant` and `ogre` 12, `drow` 14. Beasts never come from a pool: `wildlife` is neutral, and a pool holds only what `Content::may_stray` (hostile to the player). **Nor do the fey**: a type that rolls its court per creature (`factions = { seelie = 50, unseelie = 50 }`) has no `faction`, so `Content::stance` reads it as neutral and `may_stray` leaves it out; the loader refuses `pool = "fey"` ("no creature that may be drawn has that tag"). The fey vault waits on §6, F9.

## 3. The catalogue: dungeon vaults

Grouped by the kind of decision. Each entry: the name (its register: Old, Middle, or Victorian English by the age of the thing named, CLAUDE memory "World language register"), the situation, **the decision**, the file, where it suits, and what it borrows. Pool counts are below the cell count on purpose: the empty cells move from visit to visit, so the room is never the same twice even before it turns.

The finds are weighed by §7's table: the more the vault asks, the more it pays.

### A. Shut doors: what is behind?

#### A1. The Kennels *(Middle English kenel)*

Ten pens off a passage, every door shut. Seven of the ten hold one of the floor's own; every pen holds a find. Nothing shows through a shut door, and nothing in a pen can open one, so each door is a draw from the floor's population with a prize behind it, one at a time, in a doorway. **The decision**: which doors, and when to stop. Two entrances at the passage's ends, so it is also a way through for those who want none of it.

Borrows: the cages of Brogue's captive-monster vaults, without the captives.

```toml
# assets/vaults/kennels.toml
# Vault: ten pens off a passage, every door shut. Seven hold one of the floor's own, every one a
# find. The decision: which doors, and when to stop.
name = "The Kennels"
map = '''
#################
#g$#g$#g$#g$#g$##
##+##+##+##+##+##
+...............+
##+##+##+##+##+##
#g$#g$#g$#g$#g$##
#################
'''

[legend]
"g" = { terrain = "floor", monsters = { pool = "level", count = 7 } }
"$" = { terrain = "floor", loot = "any" }
```

Suits every depth (`level`). 17×7.

#### A2. The Reeve's Strong-room *(Middle English reve; strong-room is Victorian)*

Four chests in four alcoves, every one locked, one of them trapped, and two of the floor's own in the room. No key anywhere: a chest's lock is picked, Knocked, or kicked. Kicking is loud, picking takes time, and Chest-wise shows which coffer bites. **The decision**: how to open four locks with the floor listening, and whether the trapped one is worth it (a fireball blows a chest up: coin and gems survive, the rest does not).

```toml
# assets/vaults/reeves_strongroom.toml
# Vault: four locked chests in alcoves, one trapped, two guards. No key: pick, Knock, or kick.
# The decision: how to open four locks with the floor listening.
name = "The Reeve's Strong-room"
map = '''
#########
#C#T#C#C#
#.......#
#.g.g.g.#
#.......#
####+####
'''

[legend]
"C" = { terrain = "chest", locked = true }
"T" = { terrain = "chest", locked = true, trap = "arrow" }
"g" = { terrain = "floor", monsters = { pool = "level", count = 2 } }
```

Suits every depth; the chests are of the floor's band. 9×6, so it fits any rooms floor.

#### A3. The Spiteful Coffer *(Middle English cofre)*

One chest in a small room, and the player is told what it is: its fireball trap is *known* from the door ("a coffer, trapped: a fireball trap"). A burst on a shut chest blows it up, sparing the coin and the gems; a disarm failed badly springs it. **The decision**: Disable device, or Knock and stand well back (the lid lifts, the trap goes, the chest and most of what is in it with it), or leave it. One of the floor's own keeps the room.

```toml
# assets/vaults/spiteful_coffer.toml
# Vault: one chest whose fireball trap is known from the door. The decision: disarm, blow it
# open and keep the coin, or leave it.
name = "The Spiteful Coffer"
map = '''
#######
#..C..#
#.....#
#.g.g.#
###+###
'''

[legend]
"C" = { terrain = "chest", locked = true, trap = "fireball", known = true }
"g" = { terrain = "floor", monsters = { pool = "level", count = 1 } }
```

Suits every depth. 7×5. Whether `known` is honoured on a chest's trap as on a floor's is *(unchecked)*; if not, Chest-wise sees it and everyone else learns from the first spring, which is still a situation.

#### A4. The Sealed Barrow *(Old English beorg)*

The dead are locked in, not the player out: the barrow's door is locked, its key lies in the antechamber before it, and behind it stand six of the dead among their grave goods. Nothing undead opens a door. **The decision**: turn the key. Open it and they come through one doorway; or open, step back, and hold the door; or leave the key on the floor and the dead in the dark.

```toml
# assets/vaults/sealed_barrow.toml
# Vault: a locked barrow, the key lying before its door, the dead and their grave goods behind
# it. The decision: turn the key.
name = "The Sealed Barrow"
map = '''
###########
#u.u.u.u.u#
#.$.....$.#
#u.u.u.u.u#
#####L#####
#...k.....#
#####+#####
'''

[legend]
"u" = { terrain = "floor", monsters = { pool = "undead", count = 6 } }
"$" = { terrain = "floor", loot = { pool = "jewellery", grade = "magic" } }
"L" = { terrain = "door_closed", lock = "barrow_seal" }
"k" = { terrain = "floor", objects = ["lead_key"] }
```

Ships `assets/data/objects/lead_key.toml` (`key = "barrow_seal"`). Suits every depth (`undead` runs 1 to 70). 11×7.

### B. Two ways in: the way round

#### B1. The Gauntlet *(Victorian phrase, "to run the gauntlet"; the thing itself is older)*

The prize at the far end, two ways to it: a corridor with three arrow traps the player can see, or the guard-room with the floor's own in it, a door at each end. **The decision**: traps or the fight. Disable device makes the corridor free; a sidestep is a gamble three times over; the guard-room is a corridor-shaped fight through a doorway. One entrance, so the choice is made inside.

```toml
# assets/vaults/gauntlet.toml
# Vault: a trapped corridor or a guard-room, both to the same prize. The decision: three known
# arrow traps, or the floor's own one doorway at a time.
name = "The Gauntlet"
map = '''
###############
#.^..^..^.....#
#.###########$#
+.+ggg..ggg.+.#
#.###########$#
###############
'''

[legend]
"^" = { terrain = "floor", trap = "arrow", known = true }
"g" = { terrain = "floor", monsters = { pool = "level", count = 4 } }
"$" = { terrain = "floor", loot = "magic" }
```

Suits every depth; the arrows do not scale, the finds do (§7). 15×6.

#### B2. The Troll-bridge *(Old English brycg; the troll under it is the folk tale's)*

A lava moat, a causeway one cell wide across it, and the bridge-keeper at its far end before the finds. Lava lights the hall, so the keeper sees the player come. **The decision**: a single fight on a one-wide bridge with no stepping aside and no flanking, or arrows and spells from the near shore while the keeper has no way to come at the player but the bridge. The keeper is drawn from a pool that deepens with the floor: an orc at danger 3, an ogre at 12, a troll, a giant.

```toml
# assets/vaults/trolls_bridge.toml
# Vault: a causeway over lava and the bridge-keeper at its end. The decision: fight on the
# bridge, or shoot across the lava.
name = "The Troll-bridge"
map = '''
#############
#..LLLLLLL..#
#..LLLLLLL.$#
+.........T$#
#..LLLLLLL.$#
#..LLLLLLL..#
#############
'''

[legend]
"L" = "lava"
"T" = { terrain = "floor", monsters = { pool = ["orc", "ogre", "troll", "giant"] } }
"$" = { terrain = "floor", loot = { pool = ["weapons", "armour"], grade = "magic" } }
```

Suits danger 3 and deeper (an orc runt is the shallowest keeper); above that the bridge is unkept and the hall is three free finds, which is the price of no depth field (§1). 13×7.

#### B3. The Mere *(Old English mere, a pool)*

A hall of shallow water with a walk of dry stones through it, broken by three short wades, and marsh-things standing in the water either side. Wading is slow and loud, and every wade is beside them. **The decision**: cross by the stones and wade where they end, with what is in the water coming, or stay dry at the door and shoot what rises, or go round the hall entirely. The finds on the island at the far end.

```toml
# assets/vaults/the_mere.toml
# Vault: dry stones across a shallow mere, marsh-things in the water. The decision: wade past
# them, slow and loud, or shoot from the door.
name = "The Mere"
map = '''
###############
#~~~~~~~~~~~~~#
#~~m~~~m~~~m~$#
+..~~..~~..~.$#
#~~m~~~m~~~m~$#
#~~~~~~~~~~~~~#
###############
'''

[legend]
"~" = "shallow_water"
"m" = { terrain = "shallow_water", monsters = { pool = "marsh-thing", count = 3 } }
"$" = { terrain = "floor", loot = "any" }
```

Suits danger 4 and deeper (frogs and leeches first, crocodiles, a marsh troll, a bog hag by 11); shallower, an empty mere. 15×7.

#### B4. The Archers' Walk *(Middle English archer)*

A dais in a ring of water, archers on it, the finds at their feet: two missile weapons, one of them magic at least. Every step toward them is through water, at twice the time, under fire. **The decision**: cross the water taking the volleys, or trade shots from the door where only the player's missiles decide it, or walk on.

```toml
# assets/vaults/archers_walk.toml
# Vault: archers on a dais in a ring of water, their arms at their feet. The decision: wade
# under fire, or trade shots from the door.
name = "The Archers' Walk"
map = '''
###############
#.............#
#..~~~~~~~~~..#
+..~$aa.aa$~..#
#..~~~~~~~~~..#
#.............#
###############
'''

[legend]
"a" = { terrain = "floor", monsters = { pool = "archer", count = 3 } }
"$" = { terrain = "floor", loot = { pool = "missile_weapons", grade = "magic" } }
```

Suits danger 3 and deeper (goblin and kobold archers, then skeleton archers, orc, duergar, drow). 15×7.

### C. Crowds, and what holds them

#### C1. The Mead-hall *(Old English medu-heall)*

A long hall with a fire, a tribe at the board: goblins on the upper floors, hobgoblins and orc veterans below. Doors at both ends of the hall, and behind the hall the buttery, with its own door to the outside and a door to the hall. The buttery holds the bread and meat of the feast (placed, not rolled: food is no loot class), a drink, and something good. **The decision**: the quiet way in by the buttery's outer door (the hall is a wall away and hears a kicked door or a fight) and out again; or the hall itself, where eight pour out of two doors. The hall is a through-route, and that is a decision too: a crowd between the player and the stairs.

Borrows: the shape of NetHack's barracks and throne rooms.

```toml
# assets/vaults/mead_hall.toml
# Vault: a tribe at the board in a long hall, and the buttery behind it with a door of its own.
# The decision: the buttery quietly from outside, or the hall.
name = "The Mead-hall"
map = '''
#################
#.g.g.g.g.g.g...#
+.g.g.g.g.g.g.*.+
#.g.g.g.g.g.g...#
########+########
#f.p..f.m.f..p.f#
########+########
'''

[legend]
"g" = { terrain = "floor", monsters = { pool = ["goblin", "hobgoblin", "orc"], count = 8 } }
"*" = { terrain = "floor", light = "lamp" }
"f" = { terrain = "floor", objects = ["bread", "dried_meat*2"] }
"p" = { terrain = "floor", loot = { pool = "potions" } }
"m" = { terrain = "floor", loot = "magic" }
```

Suits danger 2 and deeper. 17×7.

#### C2. The Bell and the Barracks *(Victorian barracks)*

A strong-room with a rare thing on a dais, and a tripwire on each of the four cells about it. Through an open doorway, the barracks, with eight of the floor's own in it and no door to hold them. Ring the bell and they come. **The decision**: search the floor first (time: Spot, then Disable device), or take the thing and run with the barracks behind, or take it and hold the doorway. One entrance, into the strong-room, so the barracks is seen through its doorway before anything is decided.

```toml
# assets/vaults/bell_and_barracks.toml
# Vault: a rare find ringed by tripwires, and the barracks through an open doorway. The
# decision: search and disarm, or ring the bell and run.
name = "The Bell and the Barracks"
map = '''
###############
#g.g.g.g.#....#
#.g.g.g.g#.^..#
#g.g.g.g.'^r^.+
#.g.g.g.g#.^..#
#g.g.g.g.#....#
###############
'''

[legend]
"g" = { terrain = "floor", monsters = { pool = "level", count = 8 } }
"^" = { terrain = "floor", trap = "alarm" }
"r" = { terrain = "floor", loot = "rare" }
```

Suits every depth. 15×7. A branch's own hidden traps may land on the barracks' cells too; nothing in the vault minds.

#### C3. The Catacomb *(Middle English, from the Latin)*

A passage one cell wide with the dead in niches down both sides, eight of eighteen, and three finds along the way. It is a fight in a line: one at a time, no stepping aside, and whoever is behind is behind. Both ends are entrances. **The decision**: which end to start from, and whether to be in a one-wide passage with the dead on both sides at all. A fine place to be caught between two of them.

```toml
# assets/vaults/catacomb.toml
# Vault: a one-wide passage with the dead in niches down both sides. The decision: which end,
# and whether to be caught in a line at all.
name = "The Catacomb"
map = '''
###################
#u#u#u#u#u#u#u#u#u#
+..$.....$.....$..+
#u#u#u#u#u#u#u#u#u#
###################
'''

[legend]
"u" = { terrain = "floor", monsters = { pool = "undead", count = 8 } }
"$" = { terrain = "floor", loot = { pool = ["jewellery", "money"] } }
```

Suits every depth. 19×5: long and low, the shape rooms floors have room for.

#### C4. The Wights' Niches *(Old English wiht)*

Six niches along a passage, a find at the back of each, and in three of them a wight standing before its goods, in plain sight from the passage. **The decision**: take the three unguarded finds and go, or fight for the other three. Everything is visible before anything is decided, which is the opposite of the Kennels.

```toml
# assets/vaults/wights_niches.toml
# Vault: six niches with grave goods, three of them with a wight standing before its own. The
# decision: take the free three and go, or fight for the rest.
name = "The Wights' Niches"
map = '''
#############
#$#$#$#$#$#$#
#u#u#u#u#u#u#
+...........+
#############
'''

[legend]
"$" = { terrain = "floor", loot = { pool = ["jewellery", "money"] } }
"u" = { terrain = "floor", monsters = { pool = "undead", count = 3 } }
```

Suits every depth. 13×5.

#### C5. The Ossuary *(Victorian word; the bone-house it names is old)*

Bones heaped on a dusty floor, and the dead standing still among the heaps: six of twelve. Dust deadens steps (`dusty_floor`, 70 % noise), so the room favours whoever is quiet. **The decision**: creep between the still dead to the grave goods along the far wall, or wake the room.

```toml
# assets/vaults/ossuary.toml
# Vault: bones on a dusty floor, the dead standing among them, grave goods along the far wall.
# The decision: creep past the still dead, or wake them.
name = "The Ossuary"
map = '''
#############
#b.s.b.s.b.o#
#.s.b.s.b.s.#
+b.s.b.s.b.$#
#.s.b.s.b.s.#
#b.s.b.s.b.o#
#############
'''

[legend]
"." = "dusty_floor"
"b" = { terrain = "dusty_floor", objects = ["bones"] }
"s" = { terrain = "dusty_floor", monsters = { pool = "skeleton", count = 6 } }
"$" = { terrain = "dusty_floor", loot = { pool = "jewellery", grade = "magic" } }
"o" = { terrain = "dusty_floor", loot = "any" }
```

Suits every depth (skeletal rats at 1, bone dragons at 50). 13×7.

### D. Hoards with a catch

#### D1. The Delving *(Old English delfan)*

A warren of packed earth, the glitter at the ends of its winding tunnels, and its diggers in them: kobolds above, duergar below. A pick-axe lies at the way in. Packed earth is a minute's digging a cell, and digging is heard. **The decision**: walk the tunnels, or dig straight to the glitter in a minute with the warren listening.

```toml
# assets/vaults/delving.toml
# Vault: a warren of packed earth with the glitter at the ends of its tunnels and its diggers
# in them. The decision: walk the tunnels, or dig straight through, loud.
name = "The Delving"
map = '''
#############
#$.k...k.k.$#
#.EEEE.EEEE.#
#.E$.k.k.$E.#
#.EEEEEEEEE.#
#.....P.....#
######+######
'''

[legend]
"E" = "packed_earth"
"k" = { terrain = "floor", monsters = { pool = ["kobold", "duergar"], count = 4 } }
"$" = { terrain = "floor", loot = { pool = ["gems", "money"] } }
"P" = { terrain = "floor", objects = ["pick_axe"] }
```

Suits danger 2 to about 40 (kobolds 2 to 4, duergar 8 to 40). 13×7.

#### D2. Dead Men's Gear *(Middle English gere)*

Five dead on the floor of a dead-end chamber, their arms and armour where they fell, and what killed them still in the room: two of the floor's own. The scene says what the room is before the player steps in. **The decision**: five finds of weapons and armour for a fight in a room with one door, which is the whole of it; its place in the catalogue is that it is read, not rolled.

```toml
# assets/vaults/dead_mens_gear.toml
# Vault: five dead with their gear where they fell, and what killed them still there. The
# decision: five finds for what is in the room.
name = "Dead Men's Gear"
map = '''
#########
#b.b...b#
#..w.a..#
#.g.g.g.#
#..a.w..#
#b.b...b#
####+####
'''

[legend]
"b" = { terrain = "floor", objects = ["bones"] }
"w" = { terrain = "floor", loot = { pool = "weapons" } }
"a" = { terrain = "floor", loot = { pool = "armour" } }
"g" = { terrain = "floor", monsters = { pool = "level", count = 2 } }
```

Suits every depth. 9×7.

#### D3. The Reliquary *(Middle English)*

A lit altar, and before it a rare thing lying on a lightning rune the player can see, its bolt aimed down the aisle at whoever comes up it. **The decision**: Disable device; or the sidestep's gamble; or take 4d6 and the thing; or leave it on its rune. Nothing else is in the room: the trap is the guard.

```toml
# assets/vaults/reliquary.toml
# Vault: a rare find lying on a known lightning rune before an altar. The decision: disarm, eat
# the bolt, or leave it.
name = "The Reliquary"
map = '''
#########
#B.....B#
#..._...#
#...R...#
#.......#
#.......#
####+####
'''

[legend]
"B" = "brazier"
"R" = { terrain = "floor", loot = "rare", trap = "lightning_bolt", known = true, dir = "S" }
```

Suits every depth; 4d6 is deadly on floor 1 and a sting on floor 30 while the rare scales with the floor, which is the trade §7 allows a trap vault. 9×7. `dir` turns with the vault (#959).

#### D4. The Lectern *(Middle English)*

A reading-room: shelves along one wall, three scrolls and a wand on the desks, and two casters at their study. Casters in a small room are fireballs in a box, and they are soft. **The decision**: rush them before they finish a casting, or shoot from the door, or not: the scrolls are not going anywhere and neither are they.

```toml
# assets/vaults/lectern.toml
# Vault: a reading-room with scrolls and a wand on the desks and two casters at their study. The
# decision: rush them, or shoot from the door.
name = "The Lectern"
map = '''
###########
#SSS...SSS#
#.s.....s.#
#..c.w.c..#
#.s.....c.#
#.........#
#####+#####
'''

[legend]
"S" = "shelves"
"s" = { terrain = "floor", loot = { pool = "scrolls" } }
"w" = { terrain = "floor", loot = { pool = "wands" } }
"c" = { terrain = "floor", monsters = { pool = "caster", count = 2 } }
```

Suits danger 4 and deeper (a kobold shaman is the first caster). 11×7.

### E. Rooms where the player is not the only side

#### E1. Old Enemies

Two halls facing each other across a nave, demons in one and devils in the other, the nave their field and a rare thing in the middle of it. Demons hate devils (`factions.toml`: `demon` is hostile to `devil`). **The decision**: stand at the nave's door and let them settle it, then take the field from what is left; or go in while it is on. The vault's point is that the player's arrival is not the start of the fight.

Borrows: the Blood War of D&D's planes, by shape only.

```toml
# assets/vaults/old_enemies.toml
# Vault: demons in one hall, devils in the other, a nave between them and a rare find in it.
# The decision: let them fight it out, or go in while it is on.
name = "Old Enemies"
map = '''
###############
#d.d.d#.#v.v.v#
#.d.d.'.'.v.v.#
#d.d.d#$#v.v.v#
#.d.d.'.'.v.v.#
#d.d.d#.#v.v.v#
#######+#######
'''

[legend]
"d" = { terrain = "floor", monsters = { pool = "demon", count = 4 } }
"v" = { terrain = "floor", monsters = { pool = "devil", count = 4 } }
"$" = { terrain = "floor", loot = "rare" }
```

Suits danger 8 and deeper (dretches and imps first); shallower, a quiet nave with a rare in it, until the `danger` field (§6, F1) keeps it deep. *(unchecked)*: that a devil struck by a demon strikes back; if the AI only answers its own hostiles, `devil` gets `demon` in its `hostile` list (data, one line). 15×7.

#### E2. The Conventicle *(Middle English, a secret meeting)*

A summoner at the altar inside a ring of runes, the congregation about it, and the scrolls of the rite on the altar. A summoner keeps summoning while it stands. **The decision**: the order of the killing. Through the crowd to the summoner first, or the crowd first while the room fills.

```toml
# assets/vaults/conventicle.toml
# Vault: a summoner at the altar and the congregation about it. The decision: the summoner
# first through the crowd, or the crowd while the room fills.
name = "The Conventicle"
map = '''
###########
#c...c...c#
#.........#
#..r.r.r..#
#c.r$S$r.c#
#..r.r.r..#
#.........#
#c...c...c#
#####+#####
'''

[legend]
"r" = "uv_rune"
"S" = { terrain = "altar", monsters = { pool = "summoner" } }
"$" = { terrain = "altar", loot = { pool = ["scrolls", "wands"] } }
"c" = { terrain = "floor", monsters = { pool = "cultist", count = 5 } }
```

Suits danger 8 and deeper (a goblin wizard is the first summoner, a cultist the first of the crowd at 6). 11×9.

#### E3. The Champion's Ring *(Middle English champioun)*

A ring of standing stones lit by braziers, one great foe in its middle, two finds beside it. The pool deepens with the floor: an orc, then an ogre, a troll, a giant. The stones break lines of sight. **The decision**: single combat in the open, or stone to stone with missiles, keeping a pillar between; and a floor's danger in one body is a harder thing than the same in eight.

```toml
# assets/vaults/champions_ring.toml
# Vault: one great foe in a ring of standing stones, two finds beside it. The decision: single
# combat in the open, or stone to stone with missiles.
name = "The Champion's Ring"
map = '''
###########
#B...O...B#
#..O...O..#
#....$....#
#O...T...O#
#....$....#
#..O...O..#
#B..O.O..B#
#####+#####
'''

[legend]
"O" = "standing_stone"
"B" = "brazier"
"T" = { terrain = "floor", monsters = { pool = ["orc", "ogre", "troll", "giant"] } }
"$" = { terrain = "floor", loot = { pool = ["weapons", "armour"], grade = "rare" } }
```

Suits danger 3 and deeper. 11×9.

### F. Odd rooms

#### F1. The Rune-stone *(Old English rūn-stān)*

A chamber of faint runes, four of the floor's own among them, two finds, and at the centre a rune the player can read: a teleport trap, known. Step on it and the level has the player somewhere else. **The decision**: a door out that is not a door, for when the room goes wrong; or a way to skip the fight to the finds and lose the fight anyway.

```toml
# assets/vaults/rune_stone.toml
# Vault: a chamber of runes with a known teleport rune at its centre. The decision: a way out
# that is not a door, when the room goes wrong.
name = "The Rune-stone"
map = '''
#########
#g.r.r.g#
#.r...r.#
#r.$T$.r#
#.r...r.#
#g.r.r.g#
####+####
'''

[legend]
"r" = "uv_rune"
"T" = { terrain = "uv_rune", trap = "teleport", known = true }
"g" = { terrain = "floor", monsters = { pool = "level", count = 3 } }
"$" = { terrain = "floor", loot = { pool = "scrolls" } }
```

Suits every depth. 9×7.

#### F2. The Dark Chapel

A chapel with no light in it at all and a cold altar, spirits standing in the dark, rings and amulets beside the altar. Whoever carries a light is seen by everything in the room; whoever does not sees nothing of what stands there. **The decision**: light or no light, which is the vault's only question, and the one the spirits are chosen for.

```toml
# assets/vaults/dark_chapel.toml
# Vault: an unlit chapel, spirits in the dark, jewellery by a cold altar. The decision: carry a
# light and be seen, or go dark and see nothing.
name = "The Dark Chapel"
map = '''
###########
#p.......p#
#.........#
#.p.$A$.p.#
#.........#
#p.......p#
#####+#####
'''

[legend]
"A" = "spent_altar"
"p" = { terrain = "floor", monsters = { pool = "spirit", count = 3 } }
"$" = { terrain = "floor", loot = { pool = ["rings", "amulets"], grade = "magic" } }
```

Suits danger 2 and deeper (corpse candles, then stone haunts, poltergeists, shadows, wraiths). 11×7. It lays no light on purpose: the altar is the spent kind, which does not glow.

### The third batch, sketched

Twelve more for batch 5 (§8), each a line: the decision is the entry, the map is the batch's work. They bring the dungeon count to 49 with §5's three and §6's nine.

| Vault | Register | The situation and its decision | Pool | Suits |
|---|---|---|---|---|
| The Tithe-barn | Middle | the harvest heaped in a barn full of vermin: cheap fights, many of them, in a door, and a sack of coin under the grain | `vermin` 6 of 10; bread and turnips as `objects`; `money` ×1 | 1–6 |
| The Rat-run | Middle | a winding one-wide tunnel of vermin: the swarm comes one at a time, which is the only way to meet a swarm | `vermin` 8 of 12 | 1–8 |
| The Well-house | Middle | coins on the bottom of a deep pool, water-things in it: every coin is two slow loud steps | `water` 2 of 4; `money` ×4 in the water | 4+ |
| The Watch-fire | Old | a brazier-lit crossroads chamber with doors on all four sides and the floor's own at the fire: a through-route that cannot be passed quietly | `level` 3 of 4; `any` ×2 | any |
| The Smithy | Middle | a cold forge (anvil, rack, bench, decoration only) with magic arms on the rack and its smiths at it | `["kobold", "duergar"]` 3 of 4; `weapons` and `armour` magic | 2–40 |
| The Gallows-yard | Middle | the hanged still hanging (bones in a row) and two of the dead among them; the hangman's purse | `undead` 2 of 4; `money` ×2 | any |
| The Counting-house | Victorian | coin in six piles and a tripwire under one of them, the clerks' barracks through a doorway: the Bell's small cousin | `level` 5 of 8; `coins` ×6; one `alarm` | any |
| The Drowned Stair | Old | a shallow-water hall where the drowned stand: undead that are also marsh-things, in water that slows the living only | `["undead", "marsh-thing"]` 5 of 8 | 4+ |
| The Burning Hall | Old | lava channels cut the hall into islands joined by necks one cell wide, fire-things on the islands, the find on the last: three bottlenecks in a row | `["fire-thing", "fire"]` 3 of 5; `weapons` rare | 2+ |
| The Pillared Hall | Middle | a hall of pillars with archers behind them: every line of sight is short, so the player's missiles and theirs are even | `archer` 4 of 6; `missiles` ×2 | 3+ |
| The Broken Bridge | Middle | known pits where a bridge's planks were, a one-wide path between them, and the floor's own on the far side: a kick lands someone in a pit | `level` 3 of 4; `any` ×2; `pit` known ×6 | any |
| The Four Fonts | Old | four alcoves, one element each, its elementals in it: the player picks which element to face for which find | `fire`, `water`, `earth`, `air` 1 of 2 each; four finds | 1+ |

## 4. The catalogue: outdoor vaults

For the local areas of the overworld. No pools, no traps, no chests, no secret doors (§2): an outdoor vault is a shape, its finds, and at most a creature or two for flavour at a fixed level, never as a guard of anything worth having. Their decisions are small ones. A ruin comes in about one local area in six (§1), so the target is nine outdoor vaults: the three of §5 and six of these. The six are O3, O4, O6, O7, O8, and O9 (the Hoar-stone stays: the owner approved one landmark stone, §9). **O1, O2, and O5 are optional**, kept here and not cut, each the nearest to a ruin the game already has or this list keeps: the Cairn's ring of stones is the Stone circle's, the Hanging Oak's dead among the rubble is the Old Battle's on a smaller scale, and the Wayside Shrine's offering before a stone is the Ruined chapel's scroll before its altar. They are built when the owner asks for them, or when a later count calls for more.

#### O1. The Cairn *(Old; Gaelic through Scots)* — optional

A ring of standing stones about a heap of rubble, bones under it, and a piece of grave goods among them. Nothing in the way but the walk over loose stones.

```toml
# assets/vaults/cairn.toml
# Outdoor vault: a ring of stones about a cairn, bones and grave goods under it.
name = "The Cairn"
kind = "outdoor"
map = '''
   O   
 O,,,O 
 ,,b,, 
O,,$,,O
 ,,,,, 
 O,,,O 
   O   
'''

[legend]
"O" = "standing_stone"
"," = "rubble"
"b" = { terrain = "rubble", objects = ["bones"] }
"$" = { terrain = "rubble", loot = { pool = "jewellery" } }
```

7×7.

#### O2. The Hanging Oak *(Middle; the hanging tree of every crossroads)* — optional

A dead tree, bones at its foot, a rope among them, and the hanged man's purse where it fell.

```toml
# assets/vaults/hanging_oak.toml
# Outdoor vault: a dead tree, the hanged at its foot, his purse where it fell.
name = "The Hanging Oak"
kind = "outdoor"
map = '''
 ,,, 
,,T,,
,,b,,
,,$,,
 ,,, 
'''

[legend]
"," = "dry_grass"
"T" = "dead_tree"
"b" = { terrain = "dry_grass", objects = ["bones", "rope"] }
"$" = { terrain = "dry_grass", loot = { pool = "money" } }
```

5×5.

#### O3. The Burnt Steading *(Old English stede)*

The timber walls of a farmhouse, its roof gone to rubble in a line through the middle, the planks of its floor still roofed either side: dark by day, a place to be out of the rain. The pots of the kitchen, food, a tool. The decision is only whether to shelter here.

```toml
# assets/vaults/burnt_steading.toml
# Outdoor vault: a burnt farmhouse, half its roof still on, its goods inside.
name = "The Burnt Steading"
kind = "outdoor"
map = '''
 WWWWW,WWW 
 W=h=,=f=W 
 W===,===, 
 W=t=,=,=W 
 WW,WW,WWW 
'''

[legend]
"W" = "timber_wall"
"=" = "plank_floor"
"," = "rubble"
"h" = { terrain = "plank_floor", objects = ["cooking_pot", "kettle"] }
"f" = { terrain = "plank_floor", objects = ["bread", "apple*2"] }
"t" = { terrain = "plank_floor", loot = { pool = "tools" } }
```

11×5.

#### O4. The Old Battle

A field of the dead among rubble and a dead tree: six of them, two with a weapon still by them, one in his armour.

```toml
# assets/vaults/old_battle.toml
# Outdoor vault: the dead of an old fight, their arms among them.
name = "The Old Battle"
kind = "outdoor"
map = '''
.b...w..b
..,T,..b.
b.a,.,b..
.,.b.w...
b..,...b.
'''

[legend]
"." = "dry_grass"
"," = "rubble"
"T" = "dead_tree"
"b" = { terrain = "dry_grass", objects = ["bones"] }
"w" = { terrain = "dry_grass", loot = { pool = "weapons" } }
"a" = { terrain = "dry_grass", loot = { pool = "armour" } }
```

9×5.

#### O5. The Wayside Shrine *(Middle)* — optional

A shrine stone in a bed of flowers, and an offering left before it: a potion.

```toml
# assets/vaults/wayside_shrine.toml
# Outdoor vault: a shrine stone in flowers, an offering before it.
name = "The Wayside Shrine"
kind = "outdoor"
map = '''
 *** 
*,S,*
*,p,*
 *** 
'''

[legend]
"*" = "flowers"
"," = "flagstones"
"S" = "shrine_stone"
"p" = { terrain = "flagstones", loot = { pool = "potions" } }
```

5×4.

#### O6. The Fairy Ring

A ring of bluebells, three sprites in it, a jewel in the grass. Each sprite rolls its court, so some of the three sting and some do not, and striking the wrong one is remembered.

```toml
# assets/vaults/fairy_ring.toml
# Outdoor vault: a ring of bluebells, three sprites of either court in it, a jewel in the grass.
name = "The Fairy Ring"
kind = "outdoor"
map = '''
  ,,,  
 ,s.s, 
,.....,
,..$..,
,..s..,
 ,..., 
  ,,,  
'''

[legend]
"," = "bluebells"
"." = "grass"
"s" = { terrain = "grass", creature = "sprite" }
"$" = { terrain = "grass", loot = { pool = "jewellery" } }
```

7×7. Sprites are level 2 and shared; three of them on a level-30 traveller's road are a nuisance, which is what a fairy is.

#### O7. The Hermit's Hut *(Middle)*

A timber hut with a door, the hermit's bones on his floor, his book, and a scroll.

```toml
# assets/vaults/hermits_hut.toml
# Outdoor vault: a hut, the hermit's bones in it, his book and a scroll.
name = "The Hermit's Hut"
kind = "outdoor"
map = '''
 WWWWW 
 W=b=W 
 W=s=W 
 WW+WW 
       
'''

[legend]
"W" = "timber_wall"
"=" = "plank_floor"
"b" = { terrain = "plank_floor", objects = ["bones", "book"] }
"s" = { terrain = "plank_floor", loot = { pool = "scrolls" } }
```

7×5. The bottom row is void, so the door is inside the vault and opens onto the area's ground.

#### O8. The Drowned Mill *(Middle)*

A mill-pond with reeds, the timber stump of the mill in it, and coin on the pond's bottom: two slow loud wades for each.

```toml
# assets/vaults/drowned_mill.toml
# Outdoor vault: a mill-pond, the mill's stump in it, coin on the bottom.
name = "The Drowned Mill"
kind = "outdoor"
map = '''
  ~~~~~  
 ~~r~~r~ 
 ~~$~W~~ 
~r~~W=W~~
 ~~$~=~~ 
 ~~r~~r~ 
  ~~~~~  
'''

[legend]
"r" = "reeds"
"W" = "timber_wall"
"=" = "plank_floor"
"$" = { terrain = "water", loot = { pool = "money" } }
```

9×7.

#### O9. The Hoar-stone *(Old English hār-stān, a boundary stone)*

One standing stone with a name: a landmark, nothing more. Look mode names it ("the Hoar-stone"), so it is a place to steer by.

```toml
# assets/vaults/hoar_stone.toml
# Outdoor vault: one old boundary stone, named, to steer by.
name = "The Hoar-stone"
kind = "outdoor"
map = '''
...
.O.
...
'''

[legend]
"." = "grass"
"O" = { terrain = "standing_stone", name = "the Hoar-stone", note = "A boundary stone older than any boundary it marked." }
```

3×3. Nine such stones with nine names would be nine landmarks for the cost of nine small files; one is built now (§9, decision 5); more stones are a later line if the owner likes steering by them.

The three existing outdoor vaults (§5) and the six make nine, the target; with the optional three, twelve.

## 5. The six that exist

| Vault | Kind | Stamps in dungeons | What is static now | What this design does |
|---|---|---|---|---|
| Flooded cell | dungeon | yes | nothing: its one find is already `loot = { pool = ["money", "jewellery"] }` (#967) | **kept as is.** The owner's example ("always 2 rubies") is already rolled. It is the shortest vault in the game and stays the easy one |
| Guard post | dungeon | yes | three piles: `spear, leather_jerkin`, `dried_meat*4`, `skeleton_key`; the kobold's `route = "E2 W2"`. Its company is already drawn: `"g" = { terrain = "floor", monsters = { pool = "level", count = 2 } }` on four cells (#958) | **converted**: the arms pile → `loot = { pool = ["weapons", "armour"] }`; the meat stays placed, as `objects = ["dried_meat*4"]` on a legend cell (food is no loot class, §2: flavour, not a find); the kobold stays (a fixed key-bearer is the pattern §2 allows); the skeleton key stays (§9, decision 3); the route turns with the vault (#959, landed) |
| Shrine | dungeon | yes | the `jade_amulet` offering on the spear trap | **converted**: the offering → `loot = { pool = "jewellery" }` on the trap cell (loot on a trap is allowed: `content/level.rs` refuses neither); the `m` find stays `any` |
| Ruined chapel | outdoor | no | the `scroll_of_light` on the altar's steps | **converted**: → `loot = { pool = "scrolls" }` |
| Ruined tower | outdoor | no | `copper_coin*3` in the rubble | **converted**: → `loot = { pool = "money" }` |
| Stone circle | outdoor | no | nothing (it holds no find) | **kept**: a landmark; it gains a `name` on its altar ("the Stone Circle") so look mode names it, as the Hoar-stone |

None of the six needs a pool it lacks. The guard post's `[[pile]]` entries become legend cells (`"w" = { terrain = "floor", loot = ... }`), so the map is the one place that says where everything lies (#959 would turn the piles either way, §2).

## 6. Vaults that need a feature first

Each with the feature, its size, and the vault it unlocks. The features are small on purpose: each is one field read in one place.

| # | Feature | Size | Where | Vaults it unlocks |
|---|---|---|---|---|
| F1 | **A vault's depth**: `danger = { from = 8 }` / `{ to = 6 }` in a vault file; `Vault::stamps_on(def, depth)` also asks the branch's danger at that depth (`DungeonDef` already resolves a depth's danger for the draw) | small | `content/vault.rs`, `mapgen/mod.rs::generate_for` (a scaled danger needs the `Visit`'s player level, which `generate_for` has) | Old Enemies, The Conventicle, every §3 vault with a "suits N and deeper" row stops being an empty room above N; shallow-only vaults (The Tithe-barn) stop stamping at danger 30 |
| F2 | **Sleeping pools**: `status = ["asleep"]` (any status) on a `monsters` cell, given to each draw as a placed creature's is (`population.rs` builds the spawn with `status: Vec::new()` today) | small | `content/level.rs`, `mapgen/population.rs` | **The Wyrm's Bed** (coin heaped round one great sleeper from `["ogre", "troll", "giant"]`: fill a sack in silence or wake it); **The Dormitory** (a tribe asleep in rows, a known tripwire at the door: the stealth vault); **The Drunken Hall** (the Mead-hall after the feast: `asleep` and `drowsy`) |
| F3 | **The key-bearer**: `carry` on a `monsters` cell, given to one of the draws (the first placed) | small | the same two files; `check_keys` already counts a carried key, and must count a pool cell's `carry` as within reach when the cell is | **One of Them Has It** (a locked treasury and a crowd in the hall before it, the key on one of them: fight the whole crowd, or pick the lock, or kick it) |
| F4 | **Patrols from a pool**: `route` on a `monsters` cell | small | the same; `out_and_back` validation as for a placed creature | **The Watchmen** (a courtyard with two patrols round a hoard: timing the gaps) |
| F5 | **Hollow pockets**: a vault's reachability check treats terrain with a `hardness` of 100 or less as passable, and `search` already finds hollows (`known_hollows`) | medium: the generator must not strand anything, and the Search rule must find a vault's pocket as it finds a level's | `content/vault.rs`, `world/search.rs` | **The Hollow Wall** (gem pockets in a warren's earth, found by Search, dug in a minute: the Delving's secret cousin) |
| F6 | **A pool's side**: `faction = "peaceful"` on a `monsters` cell | small | the same two files | **The Pens** (a tribe's captives, peaceful, in locked cells beside their captors: let them out and the captors turn on them; a fight with a third side) |
| F7 | **A generic word**: a shared dialogue entry a vault's placed creature may name (`dialogue = "captive"`) with one line and one topic | medium: dialogue is a campaign table today | `content/dialogue`, `docs/dev/npcs.md` | **The Prisoner** (a captive in a cell with a line of thanks and a rumour of the floor: where its stairs are) — the one vault here that gives information |
| F8 | **Beasts in a pool**: a pool may name a faction (`pool = { faction = "wildlife" }`) and bypass the hostile-only rule, the beasts placed hostile | small | `content/monster_pool.rs` | **The Menagerie** (a collector's beasts in pens, each pen a door: the Kennels with teeth and no loot but the beasts' owner's) |
| F9 | **The fey in a pool**: `tagged` admits a type with split `factions` when any of its courts is hostile to the player, and each draw rolls its court as a placed one does | small | `content/monster_pool.rs` (`may_stray` is a `stance` of the type's one `faction`, which a split type lacks: §2) | **The Hollow Court** (a ring of glowing fungus, the fair folk of both courts about it, a jewel in the middle; the seelie leave the player be, the unseelie do not, the courts fight each other, and striking a seelie is remembered by the wrathful fey: whom to fight in a crowd that is not all against you). Its map is drawn: 11×9, `fungus` ring, eight `f` cells with `count = 5`, the jewel `{ pool = "jewellery", grade = "magic" }` at the centre; it loads the day F9 does |

None of these is asked for before batch 2 (§8); F1 is asked for first because it fixes the "empty hall" cost that every deep vault pays today.

## 7. The vault rules for authors

The rules the catalogue was written to, for `assets/vaults/README.toml` and docs/dev/mapgen.md when the first batch lands.

| Rule | Why |
|---|---|
| **Every vault names its decision** in its file's first comment, one line. A vault whose comment is "a room with X and Y in it" is sent back | the owner's standard (#957): monotony is a room with loot and monsters in another shape |
| **The finds pay for what the vault asks.** No crowd: one find, `any`. A crowd of 1 to 3 of the floor's own, or a known trap: two finds, one of them `magic`. A crowd of 4 or more, a hidden trap, or a lock: three or four finds, one `magic`, and a `rare` where the crowd is 6 or more. A deliberate "cheap" room (Flooded cell) breaks this downward only | loot worth the danger (the brief); the player learns to read a vault's price from its door |
| **A crowd is behind something.** Four or more of a pool stand behind a closed door, across water or lava, or in a dead end with one entrance; never loose in a vault with two entrances, which the generator may make the floor's route | an avoidable door or way round for big crowds (the brief); a through-route vault is a corridor the level may need |
| **Two entrances means a way through.** Give two only to vaults that are a route by design (the Kennels, the Catacomb, the Mead-hall), one to every other | the same |
| **Counts below cells.** `count` is at most two thirds of the cells it marks, so the empty cells move | a pool with `count` = cells is a fixed layout in different faces |
| **Pools with a shallow member.** A tag pool whose shallowest type is above danger 1 stamps an empty room above it until F1 lands; so either use `level`, or a list with a shallow tribe in it (`["orc", "ogre", "troll", "giant"]`), or accept the empty room and say so in the file | §1, "Depth" |
| **Nothing names a direction.** No `note`, `name`, or comment that says north, left, or far end; routes, bolt `dir`s, `flow`, and `[[pile]] at` are allowed because #959 transforms them (prefer a legend cell to a `[[pile]]` all the same, §2). A vault whose text must name a direction says `turns = false` at the top of its file, and says why in its comment | no text that breaks under rotation (the brief) |
| **Symmetry is cheap variety.** A vault that is the same mirrored gains nothing from #959's mirror; one that is not gains two looks. The Gauntlet's corridor on one side and guard-room on the other is the shape to prefer over the Shrine's four-way symmetry | #959 |
| **Size by floor.** 9 tall or less and 19 wide or less fits beside the rooms of an 80×21 floor; bigger vaults are cave vaults and are skipped on rooms floors (`vault::shuffled` tries the next), which is allowed but not free: a vault that never fits rooms floors is dead weight there. A quarter turn (#959) swaps width and height, so a vault more than 17 wide fits a rooms floor only in the four ways that keep it lying across, and the others are failed tries (docs/dev/mapgen.md) | §2 "shape" |
| **Finds away from the door, and off the pool's cells.** A branch's hidden trap may take a loot or monster cell and leave it empty (`hide_traps` avoids piles, not these); it is a known cost and no vault should fall apart over one empty cell | docs/dev/mapgen.md |
| **Fixed creatures carry keys and colour; pools carry the danger.** A placed `creature` is level 2 on floor 20 | §2 |
| **Locks ship their key**, one key object per lock id, in `assets/data/objects/`; a chest is the keyless lock; `locked = true` on a door is not used | §2 |
| **A vault offers nothing to pray at, drink from, or temper on**: anvils, springs, and altars are decoration until a room effect names them | §2 |
| **Campaign lore stays in campaign vaults.** A shared vault names no place, people, god, or event; its names are kinds of place (a mead-hall, a cairn). The Ascmearc's ten are untouched | the brief |
| **Borrowings are named** in the file's comment, as the catalogue names them | the house rule |

## 8. Implementation issues

In order; each a PR; content only where it can be.

| # | Issue | Kind | Contents | Depends on |
|---|---|---|---|---|
| 1 | Convert the six (§5) | content | Guard post, Shrine, Ruined chapel, Ruined tower to pools; a `name` on the Stone circle's altar; the `README.toml` with §7's rules | — (#959 has landed, #1035) |
| 2 | Batch 1: eight any-depth dungeon vaults | content | The Kennels, The Reeve's Strong-room, The Spiteful Coffer, The Sealed Barrow (+ `lead_key`), The Gauntlet, The Catacomb, The Wights' Niches, The Ossuary | 1; a test that each stamps at least once in 100 seeds of a rooms branch and a caves branch (the outdoor vaults have this in `mapgen/local/tests.rs`; the dungeon ones do not) |
| 3 | F1: a vault's `danger` | core, small | §6 F1; `spec/07c-generation.md`, docs/dev/mapgen.md, docs/dev/content.md | — |
| 4 | Batch 2: the depth vaults | content | The Troll-bridge, The Mere, The Archers' Walk, The Mead-hall, The Bell and the Barracks, The Delving, Dead Men's Gear, The Reliquary, The Lectern, The Champion's Ring, The Rune-stone, The Dark Chapel, Old Enemies, The Conventicle, each with its `danger` | 3 |
| 5 | Batch 3: six outdoor vaults (§4) | content | O3 The Burnt Steading, O4 The Old Battle, O6 The Fairy Ring, O7 The Hermit's Hut, O8 The Drowned Mill, O9 The Hoar-stone; O1, O2, and O5 are optional and not in it | 1 |
| 6 | F2 + F3 + F4: statuses, a carried key, and a route on a pool cell | core, small ×3 | one PR: the three fields are read in the same two places | — |
| 7 | Batch 4: the sleeping and keyed vaults | content | The Wyrm's Bed, The Dormitory, The Drunken Hall, One of Them Has It, The Watchmen | 6 |
| 8 | Batch 5: the twelve sketched (§3, end) | content | drawn and checked as §3's were | 3 |
| 9 | F5–F9, each its own issue, when wanted | core | hollow pockets, a pool's side, a generic word, beasts, the fey (and The Hollow Court with it) | — |

Tests each content batch runs: the loader's checks (every vault file is validated at load, so a bad map fails `cargo test`), `gen` previews looked at with the new vaults stamped, the insta snapshots of the generators (a new vault changes no snapshot by itself: `vault::shuffled` draws from the same stream, but a longer list shifts which is drawn, so the Depths' snapshots will move once and are accepted deliberately), and the per-vault stamp test of issue 2.

Wiki: vaults are not described on the wiki (no locations or level walkthroughs yet, CLAUDE.md), and this design changes no rule the wiki states.

## 9. Decisions

The owner approved the design (PR #1034) with the recommendations of its five open questions, and one correction.

| # | Decision |
|---|---|
| 1 | **The target**: 48 shared dungeon vaults, in the batches of §8. The repeat curve flattens past 48 (§1); beyond it, new vaults come with new mechanics, not more shapes. The outdoor target the first pass gave (12) rested on the wrong rate and is now nine: see the correction |
| 2 | **Depth by a field**: F1, a vault file's `danger`, lands before batch 2; the catalogue's "suits" rows are written to it |
| 3 | **The Guard post's skeleton key stays**: a reward for a decision (take the store), not a pile, and the only static find left in the shared vaults |
| 4 | **Fixed creatures outdoors** are allowed for flavour only (O6's sprites); no outdoor vault guards a find with a fixed creature |
| 5 | **Landmarks**: one now, the Hoar-stone (O9); more stones only if the owner asks |
| Correction | **Ruins come by chance, not in every area.** `mapgen/local/mod.rs` rolls each landscape's ruins chance (hills 30 %; grassland, forest, plains 15 %; dense forest, swamp 10 %) before placing one, at least 24 cells from the centre on free ground out of water. A ruin is met in about one local area in six, 5 to 10 a run (§1). The outdoor target is nine: the three existing and six of §4's (O3, O4, O6, O7, O8, O9); O1, O2, and O5 are kept as optional, not cut |
