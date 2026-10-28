# Warlock — Chests, mimics, and treasure rooms

Design document for the owner's review (2026-10-28). Written against `origin/main` at b9f41e21. Nothing here is implemented; §11 cuts it into issues. It builds on what is there: locks and keys (`docs/dev/doors.md`), traps (`docs/dev/traps.md`), searching (`docs/dev/searching.md`), digging (`docs/dev/digging.md`), the loot roll (`docs/dev/magic-items.md`), the creature rosters (`docs/dev/creatures.md`), and the special-rooms system (`docs/dev/special-rooms.md`, `docs/design/special-rooms.md`). Every number is data and hidden in the game; the wiki states it (the owner's rule: the game is vague, the wiki is accurate).

## Contents

1. Summary
2. The owner's request, and what exists today
3. Chests
4. The chest loot table (shared with mimics)
5. Mimics
6. Treasure rooms and greater treasure rooms
7. Determinism
8. Talents and class abilities
9. What the player sees; the wiki, the spec, and the docs
10. Choices made along the way
11. Implementation plan
12. Open questions (each with a recommended default)

## 1. Summary

| Piece | Proposal |
|---|---|
| What a chest is | a **fixture of a cell**, like a door: a terrain (`chest`, shut, not walkable; `chest_open` and `chest_smashed`, walkable) plus per-cell state in `LevelState::chests` (its tier and what it holds). Its lock lies in `LevelState::locks` and its trap in `LevelState::traps`, the very maps a door's use, so keys, Pick lock, Disarm, searching, and the kick all work on a chest with one extra arm each and **no new action and no new key**: `open:DIR`, `pick:DIR`, `disarm:DIR`, `kick:DIR`, and a bump. §3 |
| Tiers | six, by the floor's item level, in `data/chests.toml`: coffer (1), chest (4), iron-banded chest (8), strongbox (13), ark (20), great ark (30). A tier says how sturdy the chest is, how often it is locked and trapped, what it holds, and how often it is a mimic. The Ascmearc (item levels 1 to 12) meets the first three; the deeper tiers wait for deeper places, as the deep rosters do. §3.2 |
| Locked, trapped | "they come locked, and often trapped": a lock of the floor's level (only a skeleton key fits a chest's lock, as a tempting hoard's; Pick lock as on a door), and a trap drawn from the branch's own trap list, those that fit (as a door's). §3.4, §3.5 |
| Smashing | `kick:DIR` at a shut chest: a Strength roll against its sturdiness, as a door's (Lockbreaker counts). Broken, **the trap goes off at the kicker**, and then the chest is **smashed open**: each stack inside is destroyed with 50 % (coins and gems never: they do not break), the rest spills on the cell. A burst trap (fireball, frost) going off in a chest smashes it too, however it was set off; so does a blast (a fireball's) reaching a shut chest. Opened with a key, by Pick lock, or by Open on an unlocked chest, nothing breaks, though the trap still goes off at the opener. §3.7, §3.8 |
| The loot | one table per tier, dice-driven: items at the floor's item level plus the tier's bonus through the one loot roll (`loot::roll_with`), with the tier's chance of at least magic and of rare; a purse of gold; loose gems by the gems' roll. One table serves chests and the mimics that stand in for them. §4 |
| Mimics | six creature types on the roster formulas (brute role): young mimic (3), mimic (7), old mimic (12), greater mimic (18), elder mimic (26), ancient mimic (36), each the stand-in for one chest tier. A chest placed on a floor with a danger is a mimic with the tier's chance; the mimic **carries the loot the chest would have held** (the same draws) and drops it when it dies. It is disguised through a status (`disguised`, as sleepers are `asleep`): it takes no turn, and the perception snapshot shows its cell as a shut chest of its tier and no actor, so nothing leaks. It is revealed by touch (open, pick, disarm, kick, a bump), by any blow or spell that reaches it, by a search or a Spot check, and by Mimic-wise within two cells. Revealed, it is slow, tough, and sticky: its slam can hold the victim. §5 |
| Treasure room | a special room (`treasury`, "treasure room") on the special-rooms system: 1d2 chests, gold in heaps, and loose gems; **every door of it is a secret door**. On a cave floor it is a walled box stamped into the rock with one secret door onto the cave. Weight 4. §6.1 |
| Greater treasure room | `great_treasury`, "greater treasure room": 2d2 chests a tier up, more gold, more gems; **no door at all, ever**: on a rooms floor the one door of a dead-end room is walled up and its corridor stub stays, ending in a blank wall; on a cave floor a sealed box in the rock, its wall touching the cave. The generator guarantees a diggable wall between it and the rest of the level, and `connect::validate` learns that a sealed room is reached by digging. Weight 2. §6.2, §6.3 |
| The hint | a new effect field, `arrive`: said on every arrival on the level while the room is unentered: *"Something tells you there is wealth hidden on this level."* / *"Something tells you a fortune lies walled up on this level."* Treasure sense adds the direction. Nothing new is saved. §6.5 |
| Talents | two new systems: `chests` (Deft breaker, Chest-wise; Lockbreaker's kick reaches chests) and `mimics` (Mimic-wise, Slick). Treasure rooms extend `special_rooms` (Treasure sense). No class ability (`min_class` is 0; Pick lock and Disarm already cover chests). §8 |
| Slices | seven issues in four waves; never more than one at a time that adds save state, and none adds an action. §11 |

## 2. The owner's request, and what exists today

The request (2026-10-28), verbatim:

> chests. they come locked, and often trapped. they can be smashed. traps can also break the chest if they go off. traps go off when chest is smashed. if chest was smashed open, roll 50% chance for each item inside to be destroyed. chests should contain good number of randomly generated loot roll with dice or something. also implement mimic creatures (least, lesser, regular, greater, elder, ancient variants? you decide what makes sense naming wise). sometimes a chest is a mimic instead. mimics should have same loot table as chest they mimic. also implement treasure rooms and greater treasure rooms dungeon features (1 or more chests, and piles of gold + gems). these rooms always have secret door entrances, OR they need to be accessed by digging (perhaps greater ones never connect by ready made tunnels?). there should be hint when entering level with one of these treasure rooms (you sense riches, you sense vast fortune - idk, you figure out the language)

| Part | Today |
|---|---|
| Containers | none. Objects are stacks in piles (`LevelState::objects`) and packs; a `Stack` holds an object, a count, its magic, its own parts, a food's age, and a wand's charges (`object.rs`). Nothing holds other stacks. |
| Locks and keys | per-cell state on closed doors (`LevelState::locks`, `door::Lock { id, difficulty }`), dropped by `set_terrain` when the cell stops being a closed door; keys are objects with `key = "id"` or `"*"`; `open_door` unlocks with a fitting key; Pick lock (`world/lockpick.rs`, the Open lock skill or the thief's talent) checks Dexterity against `Lock::difficulty` + the branch's `locks_per_depth`; a kick breaks a door with `1d(sturdiness) ≤ Strength` (`door::kick_breaks`, Lockbreaker's `door_kick` added) |
| Traps | data in `data/traps.toml` (shot, strike, burst, bolt, pit, alarm, teleport); instances in `LevelState::traps`; `World::spring_trap(at, victim)` the one way one goes off; a trap on a door goes off as the door is opened or kicked (`swing_open`, `kick_door`), aimed from the door; `TrapEffect::fits_door` (everything but a pit); Disarm on a known trap; searching finds them (`Hidden::Trap`) |
| Searching and secret doors | `LevelState::secrets` beside the wall each looks like; the rooms generator hides `secret_doors` of a branch's doors last, keeping each only if `connect::validate` still reaches the stairs and every key without it; one check for all hidden features, active and passive (`world/search.rs`) |
| Digging | `wall` hardness 600, `packed_earth` 100, `granite` 3000, into `rubble`; a pick-axe at Strength 10 does 10 a turn; a blast (the fireball trap's 1000) breaks walls under it; edge cells never break; creatures do not dig |
| Loot | `loot::roll_with(content, rng, level, grade, source, luck)` draws a gem or a base item, its rarity, affixes, name, ornament, and material from the stream it is given; `gem::roll` a loose gem at an item level; coins are objects of class `coins` (`gold_coin`, value 1, stackable); shops pay in gold |
| Special rooms | features in `data/room_effects.toml` with weights, `favoured`/`banned` per dungeon, eligibility, `near`/`enter`/`pervade` lines, `feature` and `dressing` terrain, dens, the tempting hoard (`hoard`, named "treasure room", which seals when something is lifted), scriptoria's `loot`; generated last from the map stream, filled from `swarm/<level id>` |
| Creatures | types in `data/creatures/`, numbers by the roster formulas (`content/animated.rs::Numbers::at`, `undead_roster_tests.rs`'s roles), holds (`held`, `world/hold.rs`), sleepers (`asleep`, `sleeps = true`: no turn, `unaware`), the disguise of animated things by `Shape` in `SeenActor` (what they look like, which anyone sees), `stray = false` for types placed only by their own rules |
| Mimics | deferred from the animated-objects design (its §16 D11): "an item that turns out to be a creature; the surprise is its point" |

## 3. Chests

### 3.1 What a chest is, and why not an object

A chest is a **fixture of a cell**: a terrain plus per-cell state, exactly as a locked door is a terrain plus a lock. Not a carried object (decision Q1). The reasons:

- Everything a chest does is done to a **cell from beside it**: open, unlock, pick, disarm, kick. Those are the door's verbs, each already an action with a direction prompt, a price, a noise, and a replay form. Locks and traps are per-cell maps; a chest's lock and trap go in the same maps, so `key_for`, `pick_lock`, `disarm`, `hidden_near`, `spring_trap`, and `lock_difficulty` need one more arm each ("a closed door **or a shut chest**"), not a second system.
- A mimic must look like a chest. An actor's cell showing a terrain is one override in `Perception::compute` (§5.4); an actor's cell showing a pile would need the pile machinery to lie.
- Nothing in the game carries other things. Giving `Stack` a contents field would touch every stacking, saving, hashing, shop, and description path for one object.

What is given up: carrying a chest away (ADOM's heavy chests). Nothing in the Ascmearc wants that, and a portable strongbox could come later as an object of its own without touching this.

| | |
|---|---|
| Terrain | three new terrains in `data/terrain.toml`: `chest` (shut: not walkable, transparent, a `description`), `chest_open` (walkable, transparent; "an open chest": one stands in it as in a doorway), `chest_smashed` (walkable; "a smashed chest", splinters; its `toggle` particles a puff of splinters). A chest's **tier** is not a terrain: the three terrains serve every tier, and the tier is in the cell's state, read for the name and the glyph colour. Glyphs: shut `⊟`, open `⊔`, smashed `⊠`, the tier's colour (wood browns to iron grey to the ark's dark gold); the font's coverage is checked in C1, with `(` the fallback. |
| State | `LevelState::chests: BTreeMap<Pos, Chest>` (saved, hashed, left out while empty, so other levels hash as before): `Chest { tier: ChestTierId, holds: Vec<Stack> }`. `holds` is ground truth until the chest is open: nothing in perception reads it. An open or smashed chest has no entry: its contents lie in the pile at its cell, which is walkable, and are picked up as any pile (`pickup`). |
| The one way it changes | `World::set_terrain`: a chest's lock stays only on a shut chest (the lock arm becomes "closed door or shut chest"), its trap goes with it as a door's does (open or smashed: the trap is gone), and its `chests` entry is **spilled** (§3.7) by the opening code before the terrain changes. A chest never becomes anything but its open or smashed self; digging and blasts treat the shut chest as terrain with a `hardness` (§3.8). |
| Tier ids | `ChestTierId`, an index into `data/chests.toml` in file order (in the content fingerprint, as effects are). |

### 3.2 The data: `data/chests.toml`

One `[[tier]]` each, least first. A tier is chosen by the **floor's item level** (`Content::item_level`, the depth: what the floor's loot rolls at): the last tier whose `from` the item level reaches. Level files name a tier outright.

```toml
# Chests (spec §8 "Chests"): what a chest of each tier is. Chosen by the floor's item level.
# Fields:
#   id, name      the tier ("iron-banded chest"), as look mode and messages say it
#   description   what look mode adds after the name
#   fg            its colour on the map
#   from          the least item level it is placed at (1 up; tiers in rising order)
#   sturdiness    a kick breaks it with a chance of the kicker's Strength in this, as a door's
#                 (terrain.toml), 1 to 100
#   hardness      what a blast must beat to smash it, as a wall's (1 to 10000); digging never
#                 touches a chest
#   locked        percent of chests of the tier that come locked (0 to 100)
#   trapped       percent that come trapped, from the branch's traps that fit (0 to 100)
#   mimic         percent that are a mimic instead, on a floor with a danger (0 to 100)
#   loot          the table (§4): items = dice, loot_level = item levels above the floor's,
#                 magic = percent of items at least magic, rare = percent rare,
#                 gold = dice × each, gems = dice (0 allowed: "0")

[[tier]]
id = "coffer"
name = "coffer"
description = "A small wooden box with a hasp."
fg = "#9a7a48"
from = 1
sturdiness = 20
hardness = 200
locked = 60
trapped = 25
mimic = 5
loot = { items = "1d3", loot_level = 0, magic = 10, rare = 1, gold = { dice = "2d6", each = 1 }, gems = "1d4-3" }
```

| Tier | Name | `from` | Sturdiness | Hardness | Locked | Trapped | Mimic | Where met |
|---|---|---|---|---|---|---|---|---|
| 1 | coffer | 1 | 20 | 200 | 60 % | 25 % | 5 % | Cealdhlāw 1–3, Fennmynster 1–3, Fyrdelf 1–3, the lairs 1–3 |
| 2 | chest | 4 | 30 | 400 | 75 % | 40 % | 8 % | Cealdhlāw 4–6, Fennmynster 4–7, Fyrdelf 4–7, the lairs' 4 |
| 3 | iron-banded chest | 8 | 40 | 800 | 85 % | 50 % | 10 % | Fennmynster 8, Fyrdelf 8–12 |
| 4 | strongbox | 13 | 50 | 1500 | 95 % | 60 % | 12 % | deeper places |
| 5 | ark | 20 | 60 | 2500 | 100 % | 70 % | 15 % | deeper places |
| 6 | great ark | 30 | 70 | 4000 | 100 % | 80 % | 15 % | deeper places |

"Ark" is the Middle English word for a great chest (Chaucer's *arke*): an old thing in the register the owner set for old things, and short. The descriptions explain it ("A great chest of oak, bound and studded with iron."), so a player who does not know the word learns it from look mode. The other names are plain. A tier's `locked`, `trapped`, and `mimic` draws are each one `chance` draw from the chest stream (§7).

Sturdiness is on the door's scale (`door_closed` 30): a Strength 12 character breaks a coffer with 60 % a kick, a chest with 40 %, a strongbox with 24 %; Lockbreaker's +6 counts (§8). Hardness is on the wall's scale (600): a fireball's blast (1000) smashes anything up to an iron-banded chest; the deep tiers want a bigger blast than the game has yet.

### 3.3 Placement

| Where | How |
|---|---|
| Generated floors | `chest::fill` (`mapgen/chest.rs`), after `room_loot::fill` and before `gear::arm`, from the new stream `chests/<level id>` (§7): the branch's `chests_per_level = [least, most]` (`data/dungeons.toml`, default `[0, 1]`) decides how many; each goes on a cell of plain floor of a room (a rooms floor: not the arrival room, not a shop, not a vault, not a special room with a feature or sleepers; a cave floor: any plain floor cell at least 6 cells from the stairs) where nothing lies, stands, shines, or leads away, and which the room's walkable cells stay 4-connected without (`special::joined`, as a feature's cell); then its tier (the item level), lock, trap (§3.5), contents (§4), and, on a floor with a danger, whether it is a mimic (§5.3). A chest in a doorway or on a patrol never happens (the same filters as a feature's). |
| Treasure rooms | §6.4, from the same stream, after the floor's own chests |
| Level files and vaults | a legend entry `chest = "coffer"`, with the door's `locked = true` / `lock = "id"` / `lock_difficulty`, the trap fields `trap = "arrow"` (`known`, `dir`), and `objects = ["gold_coin*30", "gem:ruby"]` for what it holds (the `[[pile]]` syntax, `parse_stacks`); `mimic = true` places a mimic of the tier instead (§5.3). `content/level.rs` checks: a chest cell is `chest` terrain, a trap on it fits, a pit never, a lock only with the terrain shut, and a key to its lock within reach as for a door's (`check_keys`: a chest's key may be inside another chest, but never inside itself). Vaults may hold chests (the drake's hoard, the robbers' hoard, and the captain's hall are the obvious takers; content, not this design). |
| Dumps | `truth` marks a shut chest's cell with its glyph and, in the `rooms`-style legend line, `chest coffer at (x, y): locked, trapped (arrow), holds 3 stacks` per chest (dev-only); `view` shows what the player sees (§3.9) |

### 3.4 Locks and keys

| Rule | Proposal |
|---|---|
| The lock | `Lock::new(None)` in `LevelState::locks` at the chest's cell: only a skeleton key fits a generated chest (as a tempting hoard's doors), its difficulty `LOCK_DIFFICULTY` (14) plus the branch's `locks_per_depth` (`World::lock_difficulty`, as a door's). A level file's `lock = "id"` makes one its key opens, as a door's. |
| Opening with a key | `open:DIR` at a locked chest with a fitting key carried: `unlock` + `open_chest` (a new action cost, 1000; `data/actions.toml` beside `open_door`), "You unlock the coffer with your skeleton key and open it." Without one: "The coffer is locked." and no time (`open_door`'s own flow, one arm). |
| Pick lock | `pick:DIR` works at a chest as at a door (`World::pick_lock`: the cell check becomes "a locked door or a locked chest"); the Open lock marks and the ability's availability are the door's. A picked lock is removed and the chest opened as `open_chest` does (so its trap goes off at the picker, as a trapped door's does). |
| Kicking | §3.7: a kick never unlocks, it breaks |
| Creatures | never open chests (they open doors only with `opens_doors`, and a chest is not in their way) |
| Hidden | whether a chest is locked is **not shown** until tried (as a door's lock is not); Chest-wise (§8) shows it |

### 3.5 Traps on chests

| Rule | Proposal |
|---|---|
| Which traps | drawn from the **branch's `traps` list** (`data/dungeons.toml`), those that fit: `TrapEffect::fits_fixture` is `fits_door` renamed (everything but a pit). A branch without traps places no trapped chests (its `trapped` draw is still made, so streams do not move). So the Cealdhlāw's coffers hold arrows, spears, and alarm bells; Fyrdelf's chests fireballs and lightning; Fennmynster's frost and teleports. No chest-only trap kinds now (Q8). |
| Where it lies | `LevelState::traps` at the chest's cell, `known = false`. Searching finds it as a door's trap (`hidden_near` reads the traps map whatever the terrain; the sign is the chest's: "You notice that the coffer is trapped: an arrow trap."); the passive check and Spot apply; dwarves' `stone` sense does not (a chest is no stone). |
| When it goes off | **on opening** (`open_chest`, with a key or without, and after Pick lock) at the opener, and **on smashing** at the smasher (§3.7), through `spring_trap(at, who)`, aimed from the chest as from a door: a shot and a strike at the victim, a bolt from the chest through the victim, a burst centred on the chest (its own cell never stops it). A step never sets it off (`step_onto_trap` skips chest cells as it skips doors; a shut chest is not walkable anyway). |
| Disarm | `disarm:DIR` at a chest with a **known** trap: the door's rule, nothing special (`World::disarm`'s cell check reads the traps map). Failed by much, it goes off at the disarmer, and if it is a burst it smashes the chest (§3.7): a botched disarm can cost the loot. |
| A trap that breaks the chest | a **burst** trap (the fireball, the frost) goes off *inside* the chest, so a sprung burst **smashes it**, however it was sprung (opened, kicked, disarmed badly). A shot, strike, bolt, alarm, or teleport leaves the chest as it was (shut if the kick failed; open if it was opened). A spent trap (`charges`) is gone as on a door. Rule: `TrapEffect::smashes_fixture()` is true for `Burst`. |
| Known or not | the trap's knowledge is the door's: set off, seen going off, or found; `trap_memory` shows it on the chest's cell as on a door's (the glyph stays the chest's; look mode adds "trapped: an arrow trap") |
| Trap sense | its `trap_damage_percent` halves a chest trap's blow as any trap's (no hook of its own) |

### 3.6 Opening, picking, disarming, and bumping: the flow

`World::open_chest(p)` (`world/chest.rs`), the one way a shut chest opens without breaking, reached from `open_door`'s arm for a `chest` terrain, from `pick_lock`, and from the bump (`player_moves` opens a shut chest the player walks into, as it opens a closed door):

1. Locked and no key: "The coffer is locked.", no time (a kick or Pick lock is the way).
2. Price: `unlock` + `open_chest`, or `open_chest` alone; taken before the trap (it may kill).
3. The lock is removed; the trap, if any, goes off at the opener (`spring_trap`); if it was a burst, go to the smash (§3.7, step 3) instead of 4.
4. Spill: the contents go to the pile at the cell (`add_to_pile`, top first in the order rolled), the entry leaves `chests`, the terrain becomes `chest_open` (`set_terrain`, which drops the trap's memory with the trap). Noise: `Act::OpenDoor`'s loudness with the chest's own `toggle_noise` (a creak, 20).
5. "You open the coffer." then the pile line ("You see here ...") as a step would say it, so the player reads what they found. Nothing is identified.

A chest that is open is a pile on a walkable cell from then on; nothing closes it again (no `close:DIR` arm: a closed chest would be an obstacle the player made, with nothing in it).

### 3.7 Smashing

`kick:DIR` at a shut chest (`World::kick`'s terrain match gains a chest arm beside the door's):

1. Price: the plain `kick` (not an attack), as a door's. Noise: `Act::Kick(p)` as a door kick (Lockbreaker's `kick_noise_percent` counts).
2. The break draw: `door::kick_breaks(rng, strength + athletics + door_kick, sturdiness)`, the door's one gameplay draw, against the tier's `sturdiness`. Not broken: "You kick the coffer. It holds.", and the turn is spent (no limit on tries: time is the price).
3. Broken: the lock is gone; **the trap goes off at the kicker** (the owner's rule), `spring_trap(p, player)`, aimed as from a door. The kicker may die here; the chest still breaks.
4. **Smashed open**: for each stack in `holds`, in order, one gameplay draw at `[smash] destroyed` (`data/chests.toml`, 50): destroyed stacks vanish; **coins and loose gems are never rolled** (they do not break; recommended default, Q4). The rest spills on the cell; the terrain becomes `chest_smashed`. The lines: "The coffer bursts apart!" then, for what broke, one line naming the stacks, "The potion of healing and the scroll of fireball are destroyed.", then the pile line.
5. Athletics marks as a door kick gives them (`Use::Kick`).

**What counts as smashed** (the 50 % roll runs) and what does not:

| How the chest opened | Smashed? |
|---|---|
| a kick that broke it | yes |
| a burst trap going off in it (opened, kicked, or disarmed badly) | yes: the trap's blast is inside; the roll runs even if the kicker or opener died |
| a blast reaching it from outside (a fireball trap elsewhere, a scroll of fireball, the altar's explosion) with a `blast` above its `hardness` | yes, through `World::blast` (§3.8) |
| Open on an unlocked chest, a key, Pick lock | no: everything inside is intact, whatever trap went off (a fireball trap's victim still loses the contents: it is a burst) |
| a shot, strike, bolt, alarm, or teleport trap | no: these do not touch the chest |

A smashed chest is a walkable splinter heap with a pile; the state entry is gone, nothing more to do with it.

### 3.8 Blasts, fire, and digging

| | |
|---|---|
| Blasts | `World::blast(center, radius, blast)` breaks every breakable terrain in the burst whose hardness is below the blast; a shut chest is breakable terrain with the tier's `hardness` (`TerrainDef::breaks` is read for walls; for a chest the tier's number is read in its place): it **smashes** (§3.7 step 4, no kicker; its own trap goes off at no one: a trap needs a victim, so a chest blown open from afar springs nothing, and its trap is dropped with the terrain). "The coffer bursts apart!" if seen. |
| Fire | no burning of chests or their contents beyond the above (no fire-spreads rule exists). |
| Digging | `dig:DIR` at a chest: refused, "You cannot dig into that." (a chest has no `breaks`: the hardness is read by blasts only). |
| Water and ice | a chest is never placed on water or ice (plain floor only), and never freezes. |
| Creatures | walk around a shut chest as around a rack; never open or break one; a creature's burst spell reaching one smashes it as a trap's does. |

### 3.9 What the player sees

| Where | What |
|---|---|
| The map | the chest's glyph in its tier's colour (shut, open, smashed); a remembered chest as last seen |
| Look mode | "a coffer" / "an open coffer" / "a smashed coffer", the tier's description, "trapped: an arrow trap" when the trap is known; **never** whether it is locked (Chest-wise: "a locked coffer"), never what is inside until open (then the pile as any pile) |
| Messages | the open, unlock, locked, kick, burst, and destroyed lines above; the trap's own lines; the pile line |
| Perception | `PerceptionView::chest_at(pos) -> Option<ChestLook { tier, state }>` on cells seen or remembered (`Perception::remember` keeps it as terrain memory does); the contents are not in it |
| Dumps | `view` the glyph; `truth` (dev) the chests' lines (§3.3); JSON `chests` (seen: `pos`, `tier`, `state`) |
| Action text | nothing new: `open:E`, `pick:E`, `disarm:E`, `kick:E`, `move:E` |

## 4. The chest loot table (shared with mimics)

"Chests should contain a good number of randomly generated loot, rolled with dice." One table per tier in `data/chests.toml`, applied by `chest::roll_contents(content, rng, tier, item_level, luck) -> Vec<Stack>` from the chest stream, the **same function and the same draws** for a chest and for the mimic that stands in for it (§5.3):

1. **Items**: `items` dice (one roll); for each, one `below(100)` for the grade (under `rare` rare, under `magic` at least magic, else any: `magic` counts the rare ones, the animated families' grade draw), then `loot::roll_with(content, rng, item_level + loot_level, grade, Source::Loot, luck)` (its own draws: a gem by chance for grade any, the base, rarity, affixes, name, ornament, material). `None` (nothing found at that level) rolls nothing more for that item.
2. **Gold**: `gold.dice` × `gold.each` coins of the shops' currency (`gold_coin`), one stack (one roll).
3. **Gems**: `gems` dice (one roll; a result of 0 or less, none), each `gem::roll` at the item level + `loot_level` (the loot's gem draws: gem by rarity, size and cut by their weights).

| Tier | Items | `loot_level` | Magic / rare | Gold | Gems | A typical coffer / ark |
|---|---|---|---|---|---|---|
| coffer | 1d3 | 0 | 10 % / 1 % | 2d6 × 1 | 1d4 − 3 (one in four) | 2 items, 7 gold, no gem |
| chest | 2d3 | 1 | 20 % / 2 % | 3d6 × 2 | 1d3 − 1 | 4 items, 21 gold, a gem |
| iron-banded chest | 2d4 | 2 | 30 % / 4 % | 4d6 × 3 | 1d2 | 5 items, 42 gold, a gem or two |
| strongbox | 3d4 | 3 | 40 % / 6 % | 5d6 × 5 | 1d3 | 7 items, 87 gold, 2 gems |
| ark | 3d4 + 2 | 4 | 50 % / 10 % | 6d6 × 10 | 1d3 + 1 | 9 items, 210 gold, 3 gems |
| great ark | 4d4 + 2 | 5 | 60 % / 15 % | 8d6 × 20 | 2d3 + 1 | 12 items, 560 gold, 5 gems |

For scale: a generated floor lays 2 to 4 items of loot (`loot = [2, 4]`), a tempting hoard 3 at +3, a den's hoard one per four sleepers, and Cēaptūn's start gives 60 gold. A coffer is a floor's worth of loot and a few coins; an iron-banded chest on Fyrdelf 10 is two floors' worth with a third of it magic. The finder's luck (Treasure hunter, Hoard sense) applies through `roll_with` as to any find. The whole table is data, tuned after a play-through, and the wiki prints it with the means.

Coins and gems in a chest are what makes smashing a real choice: a Strength character without a key or Pick lock loses half the items on average but every coin and gem (§3.7, Q4).

## 5. Mimics

### 5.1 The ladder

Six types, one file each in `data/creatures/` (shared: a mimic is a thing of every dungeon), on the roster formulas (`Numbers::at` with the **brute** role: Toughness +5, base HP 2.5 × L × 1.6 = 4L, dice one step above the level's rung; `content/mimic_roster_tests.rs` holds them to ±35 % as the undead roster is held). Levels are spaced so each is a little **over** the floor its chest tier is met on: a mimic is a nasty surprise, not a fair fight, and the player chose to touch it.

| Level | Type | Stands in for | St | To | Dx | Wp | Base HP | Dice | Hold |
|---|---|---|---|---|---|---|---|---|---|
| 3 | young mimic | coffer | 12 | 18 | 10 | 17 | 12 | 1d8 | 10 % at 12 |
| 7 | mimic | chest | 13 | 19 | 11 | 18 | 28 | 2d6 | 15 % at 13 |
| 12 | old mimic | iron-banded chest | 15 | 21 | 13 | 20 | 48 | 3d6 | 15 % at 14 |
| 18 | greater mimic | strongbox | 18 | 24 | 14 | 22 | 72 | 3d8 | 20 % at 15 |
| 26 | elder mimic | ark | 21 | 27 | 16 | 25 | 104 | 3d10 | 20 % at 17 |
| 36 | ancient mimic | great ark | 25 | 31 | 19 | 28 | 144 | 4d8 | 25 % at 19 |

(Intelligence and Presence 2, as the mindless have; the hold's difficulty is a drain's, 12 + L ÷ 5.) Names: the owner's ladder was least, lesser, regular, greater, elder, ancient. "Least" and "lesser" read as the demons' rungs, and a plain "mimic" wants to be the common one; so young, (plain), old, greater, elder, ancient: an age ladder, which fits a thing that sits in the dark getting bigger. `xp = 5 × level + 10`, `xp_percent` 100.

```toml
# The mimics (spec §8 "Mimics", docs/design/chests-and-treasure.md §5): a thing that sits
# shaped as a chest until something touches it. Its numbers are #383's for a brute.
[[creature]]
id = "mimic"
name = "mimic"
glyph = "m"
fg = "#b08a50"
speed = 800
weight = 90000
faction = "mimics"
tags = ["mimic"]
alignment = "neutral"
vision = "darkvision"
sight = 6
hearing = 60
budge = "never"
stray = false
level = 7
xp = 45
rarity = "rare"
stats = { strength = 13, toughness = 19, dexterity = 11, willpower = 18, intelligence = 2, presence = 2 }
base_hp = 28
mimic = { chest = "chest" }
immune = ["poisoned", "asleep"]
[creature.attack]
damage = "2d6"
damage_type = "bludgeoning"
name = "pseudopod"
verb = "slams"
on_hit = [{ status = "held", chance = 15, difficulty = 13 }]
[creature.natural]
defence = 0
protection = 1
name = "woody hide"
```

### 5.2 Faction, body, and what it is

| | |
|---|---|
| Faction | a new faction `mimics` in `data/factions.toml` (placed by group, beside the vermin), hostile to the player and to every other faction, neutral to itself: a mimic eats whatever touches it, and a kobold that blunders into one is a kobold's problem. `stray = false` and no `population` names it: a mimic comes **only** through a chest (§5.3) or a legend's `mimic = true`. |
| Tags | `mimic` (a new tag; a kill quest's `tag:mimic` works at once) |
| Body | hide `woody hide` (the body rule's "hide": Protection (L − 10) ÷ 8, so 0 until the old mimic, then 1, 2, 3; `creature_body_tests` names it, decision Q11); medium size (the elder and ancient large weights, 200 and 400 kg: no large size yet, as the giants); `footsteps` 0 (it oozes); `hearing` 60; darkvision, sight 6; `speed` 800 (slow: it is a box with a mouth; fleeing works, which is the counterplay for a weak character); `budge = "never"`; `opens_doors` false |
| Corpse | yes, a "mimic corpse", `on_eat = { vomit = 60, harm = "1d3" }` (glue and bile); blood a dark amber `#5a4010` |
| Immune | `poisoned`, `asleep` (it never sleeps; a den's filler never draws it anyway) |
| Natural attack | a pseudopod slam, bludgeoning, the roster's dice, with a **hold** (`held`, `world/hold.rs`: the existing status; Strength struggles against the hit's `difficulty`): "sticky". No drain, no disease. |
| What it drops | what it carries (§5.3): the chest's loot, falling where it dies as anyone's pack does; its `loot` percent is 0 (no extra drop; the chest is the drop) |
| Stealing | `World::steal` refuses a mimic ("It has no pockets."), no time; stealing from a chest is opening it |
| Alignment | neutral (an animal); killing one is a hostile kill, as any |

### 5.3 How a chest becomes a mimic

In `chest::fill` (§3.3), after a chest's tier, lock, trap, and contents are rolled, on a floor **with a danger** (`Canvas::danger`): one `chance(tier.mimic)` draw of the chest stream. True: no chest is laid; instead a `CreatureSpawn` of the mimic type whose `mimic.chest` names the tier (exactly one type per tier: a load check, `Content::mimic_of(tier)`), `disguised: true`, `rolls_gear: false`, with **the rolled contents in its pack** (`carry`), is pushed on the template's spawns after the population's. The lock and trap draws were made but are dropped: a mimic is neither locked nor trapped (nothing says "The chest is locked." of a mimic: touching it reveals it first, §5.5). The streams are the same with or without the swap: the draws are the chest's, and the mimic's `make_creature` draws nothing (no gear, no body).

So the owner's "same loot table as the chest they mimic" is exact: the mimic carries precisely the stacks the chest would have held, rolled by the same draws, and the player gets them by killing it instead of opening it (and gets them whole: a mimic is never smashed).

A level file's `mimic = true` on a `chest = "..."` entry places the tier's mimic with the entry's `objects` in its pack (and refuses `locked`, `lock`, and `trap` beside it). The dev `spawn` action can bring one as any type (then it carries nothing and is disguised on arrival if the cell is free plain floor, else revealed).

Treasure rooms (§6.4) draw their chests through the same `fill`, so a treasure room's chest can be a mimic (the tier's chance, no more: a treasure room is not a mimic nest).

### 5.4 The disguise in the perception snapshot

The mimic is an ordinary actor in `WorldState::actors` with the status **`disguised`** (`data/status.toml`, beside Asleep: `max = 1`, no `eases`, `seen` none, a new flag `hides = true`). The status is the whole record: saved and hashed with the actor, nothing new in the save format.

| What | Rule |
|---|---|
| Turns | `creature_turn`'s `slumbers` check becomes "sleeps or hides": a disguised mimic's turn is a wait (strided in a rest), its `heard` cleared, it never reaches `decide`. It does not move, so the chest the player remembers is where it was. |
| Perception | `Perception::compute` (`perception.rs`) treats a disguised actor as **not an actor**: it is not in `actors`, `sees_actors_at` is false for its cell, and the cell's **terrain look** is `chest` of its tier (`ChestLook { tier, state: Shut }` in `chest_at`, the same struct a real chest gives, §3.9). Terrain memory remembers "a chest" there. Nothing else in the snapshot (sounds: it makes none; health; stance; `shape`) exists for it. Creature sight (`sight.rs`) reads ground truth, and other creatures see it as the actor it is, which changes nothing: they are not hostile to it (they path around it as around a rack: `budge = "never"`, and the AI never attacks a non-hostile). |
| The rules that read perception | `creatures_in_view` (rest interruption) leaves it out, as it leaves images out; `hostiles_here` (the spawn cap) **counts** it (it is a hostile on the level; a floor with a mimic spawns one fewer, which nobody notices); Sixth sense's "Something sleeps on this level." is not said for it (it does not sleep); `sense_danger` ignores it. |
| Light | it carries none; a disguised mimic's cell is lit as its terrain would be (a shut chest is transparent and not walkable, like a rack) |
| Blocking | an actor blocks a cell as a shut chest does; a step into it is a bump (§5.5). It is not transparent to *sight* as an actor would not be: actors never block sight in this game, and a shut chest's terrain is transparent, so nothing differs. |
| Dumps | `view` shows the chest; `truth` and `actors` (dev) show the mimic, `actors` adding `disguised`; JSON `actors` leaves it out until revealed, `chests` lists its cell as a shut chest |
| Look mode | the chest's words, the tier's description: nothing a real chest would not say. Mimic-wise (§8): "a young mimic (hostile, unhurt), shaped as a coffer" within its reach. |
| Leaks closed | the disguise is decided in the one place the snapshot is built; `PerceptionView::chest_at` is the only reader; the rules never ask perception. The test `mimic_tests::a_disguised_mimic_is_a_chest_in_every_snapshot` compares the snapshot of a level with a mimic against the same level with a real chest in its place (JSON equal). |

### 5.5 Revealing

`World::reveal_mimic(id, how)` (`world/mimic.rs`) removes `disguised` (not through `ended`: nothing is said of a status the player never saw), says the line, and gives it its turn as usual from then on. It is called from one line in each place a chest is touched or a creature is hurt:

| How | Where | Line (if the cell is seen) | Then |
|---|---|---|---|
| **Touch**: open, unlock, pick, disarm, kick at its cell, or a bump into it | `open_chest`, `pick_lock`, `disarm`, `kick`, `player_moves`: each first asks `World::mimic_at(p)` (a disguised actor there) | "The coffer lurches: it is a young mimic!" | the touch's action is **refused with no time** (the player meant to open a chest; nothing was done), and the mimic gets a **free blow** at the toucher at once (`World::free_blow`, the existing free-blow code, as a sneak attack's surprise: the owner's "surprise is its point"), with its hold. Trap sense and Light step do nothing: this is no trap. |
| **A blow**: any melee hit or miss at it (a player who attacks "the chest" is not a case: `move` bumps, and `fire`/`throw` aim at a cell), a missile landing on its cell, a bolt or burst reaching it, a trap's burst | `World::hurt` (as a sleeper wakes on harm) and `shoot.rs::fly` (a missile stopping at an actor's cell stops at it: the disguise does not make missiles pass) | "The coffer shudders: it is a young mimic!" | revealed, no free blow (the player struck first) |
| **Searching**: the active search (`search`, `search:N`) and the passive check each turn, with Spot or the race's `passive_search = "all"` | `hidden_near` lists a disguised mimic within reach as `Hidden::Mimic(id)` with difficulty 12 + L ÷ 5 + the branch's `hidden_at(depth)`; `notices`/`search_turn` make the one check; `reveal` reveals it | "You notice the coffer breathing: a young mimic!" | revealed; it is **not** `unaware` from then on (it was never asleep), but the player now chooses the first move: shoot it, or walk away. A Search mark as for any find. |
| **Mimic-wise** | `Perception::compute` with the talent's reach (§8): within it the disguise is not applied | look mode says what it is; nothing is said | not revealed in the rules (it still waits); the player just sees it |
| **Being disturbed** | nothing: noise, light, time, and a creature walking past never reveal it (as sleepers never wake on their own). The player decides when the chest is touched. |

A revealed mimic is a plain hostile creature with the `mimics` faction: it closes in and slams; the AI needs nothing new. At speed 800 it is left behind by anyone who walks away, so the fight is always chosen.

**Sneak attacks**: a disguised mimic is `unaware` (as a sleeper is), so a thief who sees it for what it is (Mimic-wise, or found it by searching and it has not yet acted) lands a sneak attack: the reward for the Spot. After its first turn it is aware as any creature.

### 5.6 Dumps and the roster test

`content/mimic_roster_tests.rs`: six types, one per tier, each tier named once (`mimic.chest`), on the guideline and within ±35 % of the brute formulas, `stray = false`, in no population, the `mimic` tag, hostile to everyone, `budge = "never"`. `world/bestiary_tests.rs`'s `WAVES` gains the six (shared). The `actors` dev layer adds `disguised` after the statuses; `truth` marks its cell `m`.

## 6. Treasure rooms and greater treasure rooms

Both are **features** on the special-rooms system (`docs/dev/special-rooms.md`): an `[[effect]]` each in `data/room_effects.toml` with a weight, drawn by `special::pick` like any feature, placed by `special::place`, and filled from the chest stream. Each is **sealed** in its own way; the rules of the chests inside are §3's. Neither pervades, neither has a `status`; both carry the new `arrive` line (§6.5) and `treasure` (their contents, §6.4), and the greater one `sealed = "dig"`.

### 6.1 Treasure room

| | |
|---|---|
| id, name, mark | `treasury`, "treasure room", `$` (the tempting hoard's mark moves to `T`; its name becomes "tempting hoard", §6.6) |
| Access | **every door of the room is a secret door** (`secret_doors = "all"` on the effect, read by the generator, §6.3). The room is found by searching (the branch's secret-door difficulty, `search.toml`'s `secret_door` plus `hidden_at(depth)`), by the arrival hint's prompting, or by digging in from a neighbouring room or corridor. |
| Contents | `treasure = { chests = "1d2", tier_up = 0, gold = { heaps = "1d4+1", dice = "3d6", each = 2 }, gems = "1d3" }`: one or two chests of the floor's tier, two to five heaps of 6 to 36 gold each, one to three loose gems on the floor (§6.4) |
| Messages | `arrive` "Something tells you there is wealth hidden on this level." `near` none (its doors are secret: nothing is glimpsed from outside until a door is found; once found, the first look in is the enter line's job). `enter` "Gold glints in the dust. No one has been here in a long time." |
| Eligibility | a room with at least one door and no stairs, not the arrival room, not a vault, not locked, not a shop's room, not on a patrol (a rooms floor); any cave floor with rock enough for the box (§6.3); `needs_danger` **not** required (a treasure room is fine on the showcase's Depths, which has no danger; its chests are then never mimics) |
| Weight | 4 (3.0 % of special rooms, about 1.8 % of generated floors: one in about 55; the Ascmearc's 26 branch floors hold about one in two games, more in Bānfeld and Fennmynster which favour special rooms). Tuned after play. |
| Fields | `secret_doors = "all"`, `treasure`, `arrive`, `enter`, `weight = 4`, `min_cells = 9` |

### 6.2 Greater treasure room

| | |
|---|---|
| id, name, mark | `great_treasury`, "greater treasure room", `G` |
| Access | **no door, ever** (`sealed = "dig"`): the room is reached only by breaking a wall, with a digging tool (`dig:DIR`), or with a blast (a scroll of fireball at the wall: its 1000 beats `wall`'s 600). The arrival hint says it is walled up. |
| Contents | `treasure = { chests = "2d2", tier_up = 1, gold = { heaps = "2d4+2", dice = "4d6", each = 5 }, gems = "2d3" }`: two to four chests of the tier **above the floor's** (`tier_up`: that many tiers up the ladder, the last tier staying last; their loot by that tier's table at the floor's item level), four to ten heaps of 20 to 120 gold, two to six gems |
| Messages | `arrive` "Something tells you a fortune lies walled up on this level." `enter` "The air is stale and heavy with the smell of old gold." |
| Eligibility | a rooms floor: a room with **exactly one door** and no stairs, not the arrival room, not a vault, not locked, not a shop, not on a patrol, **whose door cell is not on the level's edge** (edge cells never break); a cave floor: rock for the box (§6.3). Never on a level whose palette wall has no `hardness` (none exists; a check at load keeps it so for generators' walls). `needs_danger` not required. |
| Weight | 2 (1.5 % of special rooms, about 0.9 % of floors: one in about 110; a game in two finds one). The owner's "vast fortune" is a story, not a feature of every run. |
| Fields | `sealed = "dig"`, `treasure`, `arrive`, `enter`, `weight = 2`, `min_cells = 9` |

### 6.3 How mapgen seals a room and guarantees a way in

**Rooms floors** (`mapgen/rooms.rs`, after corridors, doors, stairs, keys, and `special::place`, before `hide_secrets`). The feature's effect decides what `special::seal` does to the chosen room:

| Feature | What happens to its doors | Guarantee |
|---|---|---|
| treasure room | every door of the room (closed, open, or a doorway: each made a closed door first, as `lock_rooms` does) becomes a **secret door** by the generator's own rule (`secret_look`, `template.secrets`), **not** counted against the branch's `secret_doors` (it is the room's nature, not the level's quota; a branch with `secret_doors = [0, 0]` still has them). `connect::validate` passes as before: it reaches every cell treating secret cells as doors, and the stairs and keys without them, and a treasure room holds no stairs and no key (eligibility; `lock_rooms` drops keys in open rooms only, and `chest::fill` never puts a branch key in a chest). `hide_secrets` leaves them alone (they are secret already). | a secret door is always findable (no limit on searching) and always diggable around (its wall is `wall`) |
| greater treasure room | its one door cell becomes **`wall`** (the palette's wall, hardness 600; a door that was a doorway likewise); the corridor that led to it stays, a dead end in blank rock (a tell, as in every roguelike with walled-up rooms: a player who maps the level sees a corridor that goes nowhere). The room is marked `SpecialRoom::sealed_by = Some(p)` (the breach cell; saved, hashed, left out while `None`). | **`connect::validate`** gains one rule: the walkable cells of a room with `sealed_by` need not be reachable, **provided** `p` is `wall` with a `hardness`, not on the edge, and 4-adjacent to a cell reachable from the up staircase (the corridor's end) and to a cell of the room. A level that fails it is regenerated as any invalid level is (the generator never produces it on purpose: the chosen room's door satisfied it by construction). The player's way in: dig `p` (a pick-axe at Strength 10: 60 turns, six minutes) or blast it. |

Why wall the door and keep the corridor rather than place a room with no corridor: the rooms generator joins rooms along a spanning tree and corridors reuse one another, so a room with no corridor would have to be scattered *after* the tree with its own exclusion, a new placement mode; walling a dead-end's door is three lines and gives the tell for free. A dead-end room is one of up to two the generator already picks for locks; a greater treasury takes one of them first, and `lock_rooms` sees it as not available (as a vault is).

**Cave floors** (`mapgen/caves.rs`): a cave chamber is a blob with no doors, so neither feature can take one. Instead both are **boxes stamped into the rock** at `special::place`, through the vault stamper's spot search (`vault::spots` on a generated template: a rectangle of `min_cells` + walls, at a random spot among those where it fits in rock; the cave's `make_room` rule fills cave if no rock is big enough, as for pinned vaults), then:

| Feature | The box |
|---|---|
| treasure room | the box's interior is floor, its ring `wall`; one cell of the ring, facing the cave (4-adjacent to cave floor; else the end of a tunnel dug from the ring to the cave as a vault's entrance tunnel is, `MAX_TUNNEL`), is a closed door made secret |
| greater treasure room | the box as above with **no door**; the spot is chosen so that at least one ring cell is 4-adjacent to cave floor reachable from the stairs (`distances` from the up staircase decides reachability; the stamper prefers such spots and fails the attempt if there is none: the next attempt's stream tries again, as a vault that fits nowhere does); that ring cell is `sealed_by` |

Both generators then run `connect::validate` with the new rule, and `hide_traps` never puts a trap on a cell of a sealed room (a trap no one will walk on for a long while is wasted; also simpler: the room's cells are left out as a shop's are).

**Levels where digging is impossible.** There are none by terrain: every generator wall is `wall` (hardness 600) or `packed_earth`, and the only harder shipped stone is `granite` (3000, still diggable). What can be missing is the **tool**: a character without a pick-axe or a blast cannot enter a greater treasure room, and that is the design (the owner: "they need to be accessed by digging"). The arrival hint tells them something is there; the dead-end corridor shows them where; Tunneler, a pick-axe from the quarry or a shop, a dwarf's digging, or a scroll of fireball is the answer, in this visit or the next. A greater treasure room is never required for anything (no stairs, no key, no quest target may be placed in one: `connect::validate`'s key rule and the quest roller's placement both exclude sealed rooms). On a hand-made level `[[room]] effect = "great_treasury"` is accepted only if the room's cells are unreachable without digging and a `breach` cell is given (`content/level.rs` runs the same check).

### 6.4 Contents

`treasury::fill` (`mapgen/treasury.rs`), after `chest::fill` on the chest stream, for each special room whose effect has `treasure`:

1. **Chests**: `chests` dice (one roll); each a chest of the floor's tier raised by `tier_up` (§6.2), placed by `chest::place` (the same cell filters, the same lock, trap, contents, and mimic draws as a floor's chest: §3.3, §5.3) on the room's plain floor, drawn anew; a room too small for them all holds what fits.
2. **Gold**: `gold.heaps` dice (one roll); each a stack of `gold.dice` × `gold.each` gold coins on a plain floor cell drawn anew (two heaps on one cell join: coins stack).
3. **Gems**: `gems` dice (one roll); each a `gem::roll` at the room's item level on a cell drawn anew.

Nothing else: no guardians, no sealing-on-lift (that is the tempting hoard's game), no sleepers. A treasure room's danger is its mimics and its traps, and the time spent finding it. The tempting hoard stays the greed test; the treasure room is the explorer's reward.

The `rooms` dev layer lists the room's chests, heaps, and gems as placed; `truth` shows them (and a mimic as `m`).

### 6.5 The hint on arrival

| | |
|---|---|
| Field | `arrive` (a message) on an effect, beside `near`, `enter`, and `pervade` |
| Rule | said in `World::travel` (and at the start of a run), after the arrival lines and the `pervade` line, **once per arrival**, for each effect with an `arrive` that has a room on the level the player has not `entered` (one line per effect, not per room: two treasure rooms say it once). Left once the room is entered. |
| Stored | nothing new: `SpecialRoom::entered` is the record. The line is said again on every return to the level while the room waits, which is the point: it nags. |
| Words | treasure room: *"Something tells you there is wealth hidden on this level."* greater: *"Something tells you a fortune lies walled up on this level."* Plain present-day English for the player's own inner sense (as the `pervade` lines are: "The dead do not rest on this level."); "walled up" is the one word of mechanics, and it is the clue the owner asked for. No "you sense": the game already says "Something sleeps on this level." for Sixth sense, and the shape matches. |
| Treasure sense | the talent (§8) adds the bearing: *"..., to the north-east."* (the room's middle from the arrival cell, eight winds) |
| Also | Sense of place (the existing talent) names a glimpsed room; a treasure room is never glimpsed before it is found, so it adds nothing here |

### 6.6 The tempting hoard's name

`data/room_effects.toml`'s `hoard` is named "treasure room" today. With a feature of that name arriving, the hoard becomes **"tempting hoard"** (its id `hoard`, its mark `T`, its rules, Hoard-wise's text "You know how a treasure room is baited" becomes "how a tempting hoard is baited"), so look mode and the wiki tell the two apart. Data and two strings; no save changes (effects are saved by index).

## 7. Determinism

| Draw | Stream |
|---|---|
| which room is a treasury, the sealed box's spot and breach cell | the level's **map** stream (as every feature: `special::pick` and `place`) |
| a floor's chests: how many, their cells, tiers (none: the item level decides), locked, trapped and which trap, contents (§4), mimic or not | new **`chests/<level id>`** (`Rng::chests`), after `room_loot`'s swarm stream and before gear, so loot, population, dens, hoards, scriptoria, and gear streams do not move; a level is the same with or without chests |
| a treasure room's chests, heaps, and gems | the same chest stream, after the floor's chests |
| a mimic's loot | the chest's draws (the swap draws nothing more) |
| the kick's break, the 50 % destruction per stack, the trap's rolls, the free blow, the hold, the search checks | the **gameplay** stream (they happen in play) |
| a blast smashing a chest | none: a comparison |
| the arrival hint, Treasure sense's bearing | none: state and geometry |

`Rng::chests` joins `rng.rs`'s table and `spec/08j-determinism.md`'s. Adding chests moves no existing stream and changes no existing snapshot but the Depths' terrain where a chest or a sealed room lands (reviewed once, in C1 and C5: the `special` draw changes which effect existing seeds get, as every added feature does).

## 8. Talents and class abilities

Two new systems join `data/talents.toml`'s list (after `doors`, before `traps`, by topic): `chests` and `mimics`. Treasure rooms extend `special_rooms` (no new tag: "a new trap kind needs no new tag"). Each talent is one file under `data/talents/`, one hook each.

| System | Talent (tree) | Effect | Hook (one read) | Issue |
|---|---|---|---|---|
| `chests` | **Deft breaker** (might; Strength 12) | when you smash a chest open, what is inside breaks half as often | the destruction draw's percent (`TalentMods::smash_destroy_percent`, −50) | C1 |
| `chests` | **Chest-wise** (delver; Intelligence 11 or Search 15, `either`) | you tell a locked chest from an unlocked one at a glance, and a chest's trap gives itself away to you as you look at it (a chest's trap within sight becomes known) | `PerceptionView::chest_at`'s `locked` word; `learn_traps` on chest cells in view | C1 (lock), C2 (trap) |
| `chests`, `doors` | **Lockbreaker** (existing) | its door kicks reach chests: `door_kick` adds to the smash roll, `kick_noise_percent` to its noise | the smash roll reads the same mods the door kick does; the talent's `systems` gains `chests` | C1 |
| `mimics` | **Mimic-wise** (scholar; Spot 10 or Intelligence 12) | a disguised mimic within two cells is seen for what it is | `Perception::compute`'s disguise (`TalentMods::sees_mimics`, a reach, as Mirror-wise's) | C4 |
| `mimics` | **Slick** (finesse; Dexterity 13) | a mimic's grip never holds you | `World::afflict` of `held` by an actor with the `mimic` tag (`TalentMods::slick`; the one check beside immunities) | C4 |
| `special_rooms` | **Treasure sense** (wayfarer; Spot 10) | the hint on arriving on a level with a treasure room tells you which way it lies | the `arrive` line's bearing (`TalentMods::treasure_bearing`) | C5 |

`chests` ships in C1 with Deft breaker and Chest-wise's lock half (the bar is two talents; Chest-wise counts once it exists, its trap half following in C2 on the same file); `mimics` ships in C4 with both. **No class ability**: `min_class` is 0, and the thief's Pick lock and Disarm (class and skill abilities already in the one menu) are the class answer to chests; a priest's or wizard's *Knock* is a spell for the magic list, not a room feature (deferred to #196 as a line). No new `KeyCommand`: every verb here is an existing one.

## 9. What the player sees; the wiki, the spec, and the docs

| Where | What |
|---|---|
| Messages | §3.6, §3.7, §5.5, §6.1, §6.2, §6.5 |
| Look mode | a chest by its tier and state, its known trap, "a locked coffer" with Chest-wise, a mimic as a chest (or as itself with Mimic-wise), "in a treasure room" once entered |
| Never shown | a tier's chances and dice, what a shut chest holds, whether it is locked (without the talent), whether it is a mimic, a mimic's level or numbers, a treasure room's existence beyond the hint, where the breach is (the dead-end corridor is the player's to notice) |
| Abilities menu | nothing new (Pick lock and Disarm list as they do) |
| Action text | nothing new |
| Dumps | `truth` (chests, mimics, sealed rooms' breach), `actors` (`disguised`), `rooms` (`sealed_by (x, y)`, the contents placed), `view` and JSON `chests` as the player sees them |
| Wiki | a **Chests** page (`wiki/src/chests.md`, under "Objects and your pack" beside Gems, in `SUMMARY.md` by topic) with generated tables (`xtask/src/wiki/chests.rs`): the tiers (level, sturdiness, kick odds by Strength, hardness, locked, trapped, mimic), the loot table with its means, the smash rule and its exemptions; the **Special rooms** page's generated feature table gains the two rooms (weights, shares, the hint lines); the creatures tables gain the mimics (numbers, holds, the tag) by themselves; the Traps page says which traps a chest may hold and that a burst smashes it; the Talents page's rows |
| Spec | `spec/07b-data.md` "Chests" (the data file); `spec/08d-traps-doors-digging.md` "Chests" after "Trapped doors" (locks, traps, opening, smashing, the 50 % rule, blasts); `spec/08b-creatures.md` "Mimics"; `spec/07d-shops-and-rooms.md` "Special rooms" (the two features, `arrive`, `secret_doors`, `sealed`, the hoard's new name); `spec/08g-magic-items.md` "Chest loot"; `spec/08j-determinism.md` the stream |
| docs/dev | a new topic file **`docs/dev/chests.md`** (chests, their loot, the streams, the mimic swap; listed in CLAUDE.md's index in C1), **`docs/dev/creatures/mimics.md`** (the roster file, per the creatures rule), and a "Treasure rooms" section in `docs/dev/special-rooms.md`; `docs/dev/doors.md`, `traps.md`, `searching.md`, `digging.md`, and `mapgen.md` each get one sentence pointing at them |

## 10. Choices made along the way

| Choice | Why |
|---|---|
| A chest is a fixture, not an object | §3.1 |
| Six chest tiers, six mimics, one to one | the owner's "same loot table as the chest they mimic" is exact only with a mimic per tier; the deep tiers wait as the deep rosters do |
| The tier by item level, not danger | the chest's worth is loot, and loot is rolled at the item level; mimics then come a little over the danger, which is right for a surprise |
| Only a skeleton key fits a generated chest | as a tempting hoard's doors; a branch key that opened every chest would make the branch's two locked rooms the chests' key too, and keys are for rooms. Pick lock, a kick, or the quarry's pick-axe are the ways; a thief's game |
| Coins and gems never break | the owner's rule says each item; a smashed coffer that destroys its gold would make the kick pointless for the very character it is for (Q4) |
| A burst smashes the chest, nothing else does | the owner: "traps can also break the chest if they go off"; a burst is the trap that goes off *inside*; an arrow or a bolt does not touch the box |
| A mimic takes no turn while disguised and never reveals itself | the player decides when; the same rule as sleepers; a mimic that walked about would be a monster, not a chest |
| A touch reveals and gives the mimic a free blow, the action refused | the surprise lands once and costs the player a hit, not a turn and a hit; the next move is theirs |
| Greater treasure rooms keep their dead-end corridor | a tell the player can read, and three lines in the generator (§6.3) |
| Treasure rooms never need a danger | the Depths may have them; mimics need the danger, and a floor without one just has no mimics |
| The hint is said on every arrival | the owner asked for a hint when entering; nagging until found is what makes a player search |

## 11. Implementation plan

Each item is one issue, one branch, one PR, updating the spec, `docs/dev/chests.md` (created in C1), `docs/dev/creatures/mimics.md` (C4), `docs/dev/special-rooms.md` (C5, C6), and the wiki in the same PR. "Save" means it adds saved state; none adds an action. Sizes: S about a day, M two to three, L a week.

| # | Issue title | Sections | Adds | Depends on | Size |
|---|---|---|---|---|---|
| **C1** | Chests: `data/chests.toml` (tiers; the loot fields parsed, unused), the three terrains, `LevelState::chests`, legend placement (`chest`, `locked`, `lock`, `objects`), open with a key and without, Pick lock on chests, the bump, the kick (smash, the 50 % rule, coins and gems exempt), blasts, perception's `chest_at`, look mode, the `truth`/`view`/JSON dumps, `docs/dev/chests.md`; the `chests` system with Deft breaker, Chest-wise (locks), Lockbreaker's reach | §3.1–3.4, §3.6–3.9, §8 | save | — | L |
| **C2** | Trapped chests: `fits_fixture`, traps on chest cells (legend `trap`), springing on open and on smash, a burst smashes, Disarm on chests, searching finds them, Chest-wise's trap half, Trap sense's cut | §3.5, §8 | — | C1 | M |
| **C3** | Chest loot and generated chests: `chest::roll_contents` (items, gold, gems), `chests_per_level`, `chest::fill` on the chest stream (`Rng::chests`), the wiki's Chests page (`xtask/src/wiki/chests.rs`) | §4, §3.3, §7 | — | C1 | M |
| **C4** | Mimics: the six types, the `mimics` faction and `mimic` tag, `mimic = { chest }`, the `disguised` status (`hides`), the disguise in perception, revealing (touch with the free blow, a blow, searching and Spot, Mimic-wise), the swap in `chest::fill`, the legend's `mimic = true`, the dumps, `docs/dev/creatures/mimics.md`, `mimic_roster_tests.rs`; the `mimics` system with Mimic-wise and Slick | §5, §8 | — (the status is saved as any) | C2, C3 | L |
| **C5** | Treasure rooms: `secret_doors = "all"`, `treasure`, `arrive`, `treasury::fill`, the rooms generator's sealing of doors, the caves' stamped box with a secret door, the hint, the hoard renamed "tempting hoard" (mark `T`, Hoard-wise's text); Treasure sense | §6.1, §6.3 (secret), §6.4–6.6, §8 | — | C3 | M |
| **C6** | Greater treasure rooms: `sealed = "dig"`, `SpecialRoom::sealed_by`, the walled door and the box without one, `connect::validate`'s breach rule, `hide_traps` and the key and quest placements leaving sealed rooms alone, `[[room]]` with `breach`, the hint | §6.2, §6.3 (dig) | save | C5 | M |
| **C7** | Content: chests in the Ascmearc's vaults (the drake's hoard, the robbers' hoard, the captain's hall, the still room), `chests_per_level` per branch, a `chests` showcase scenario (a coffer, a locked chest, a trapped one, a mimic, a treasure room behind a secret door, a greater one with a pick-axe beside the wall) for captures and tests | §3.3, §6 | — | C4, C6 | S |

**Waves** (at most about three PRs at once that add actions or save state; here never more than one):

1. **C1** alone (it adds save state and changes the Depths' snapshots where a chest lands).
2. **C2, C3** together (no save state; C2 touches `trap.rs`, C3 `mapgen/mod.rs` and `rng.rs`: no overlap).
3. **C4, C5** together (C4 touches perception, `creature_turn`, and the creature files; C5 the generators and `room_effects.toml`: no overlap but `chest::fill`, where C4 adds the swap after C3's cells; C5 calls it).
4. **C6** (save), then **C7**.

**Tests each item carries**: `content/chest.rs` (the tiers parse and validate: rising `from`, ranges, the dice, one mimic per tier once C4 lands); `world/chest_tests.rs` (a coffer opens on a bump and on `open`, a locked one refuses without a key and opens with a skeleton key or Pick lock, the kick's odds over many draws, a smash destroys about half the stacks and never coins or gems, Deft breaker's quarter, a blast smashes a coffer and not a strongbox, the contents are not in the snapshot while shut and are the pile once open, look mode's words with and without Chest-wise, a legend's chest with objects, saving and a replay, the dumps); `world/chest_trap_tests.rs` (a trap springs on open and on smash at the right victim, aimed from the chest; a burst smashes it; Disarm made and failed; searching finds it with the chest's sign; a pit refused; a branch without traps traps nothing but draws the same); `mapgen/chest_tests.rs` (the count within `chests_per_level`, cells plain and joined, the level the same without chests, the loot's means over many floors within the table's, a fixture branch at item level 8 placing iron-banded chests); `world/mimic_tests.rs` (the disguised snapshot equals a chest's, a touch reveals with a free blow and no time, a blow reveals, a missile stops at it, a search and the passive Spot reveal it, Mimic-wise sees it, Slick, it takes no turn and rests are not disturbed, it counts toward the cap, it carries the chest's loot and drops it, a sneak attack on an unaware one, saving); `content/mimic_roster_tests.rs`; `mapgen/treasury_tests.rs` (every door secret and the level valid, the box on a cave floor with its secret door, contents as the dice say, a greater one's breach cell `wall` beside a reachable cell, the room unreachable without it, `validate` refusing a sealed room with no breach, no trap or key or quest target inside, regeneration when no spot fits); `world/treasury_tests.rs` (the hint on each arrival until entered, with Treasure sense's bearing, the enter line, digging the breach opens the room, a scroll of fireball does too, the `rooms` dump); screen snapshots for look mode's chest lines (text snapshots, `screen_snapshots.rs`); a replay per flow (`replay --create`, `--verify`); `cargo xtask wiki`. **No golden**: nothing here is produced by the GPU alone; the glyphs are checked by a `--capture` looked at in C1 and noted in the PR.

## 12. Open questions (each with a recommended default)

| # | Question | Recommended default |
|---|---|---|
| Q1 | A chest as a cell fixture (terrain + state) or as a carried object? | **Fixture** (§3.1). A portable strongbox can come later as an object without touching this. |
| Q2 | The tier names: coffer, chest, iron-banded chest, strongbox, ark, great ark? | **As listed.** "Ark" is the one period word; its description explains it. If the owner prefers plain words throughout: "great chest" and "vault chest". |
| Q3 | The mimic names: young, (plain), old, greater, elder, ancient? | **As listed** (§5.1). The owner's "least/lesser" are free if preferred; the levels and numbers do not change. |
| Q4 | The 50 % destruction: per item (the owner's words) or per stack, and are coins and gems exempt? | **Per stack, coins and loose gems exempt.** A stack of 20 arrows is one thing to break; gold and gems do not break, and they are why a Strength character kicks. If the owner wants the letter of the rule: per stack, nothing exempt, and the smash becomes a loss for everyone. |
| Q5 | Should a trap go off at the kicker even when the kick fails? | **No**: only a broken chest springs its trap ("traps go off when chest is smashed"). A failed kick is noise and a spent turn. |
| Q6 | A greater treasure room on a rooms floor: wall up a dead-end's door and keep the corridor (a tell), or place a room with no corridor at all? | **Wall up the door, keep the corridor** (§6.3). Cheaper, and the dead end is a fair clue. |
| Q7 | The hint on every arrival until the room is entered, or once per level? | **Every arrival** until entered; it costs one line and keeps the room in mind. |
| Q8 | Chest-only trap kinds (a poison needle, a gas cloud)? | **Not now**: the branch's own traps that fit (§3.5). New kinds are data later (a `strike` with `on_hit` once strikes carry riders). Recorded on #196. |
| Q9 | Mimics' faction: a new `mimics` faction hostile to everyone, or `vermin`? | **New faction, hostile to all** (§5.2): a mimic eats kobolds too, and a den's folk never fight beside one. |
| Q10 | Do mimics ever stand in for anything but chests (a door, an item on the floor)? | **No** (chests only). A door mimic is one `mimic = { door = true }` away later; a floor-item mimic needs the pile machinery to lie, which §3.1 avoided. |
| Q11 | A mimic's natural Protection: the body rule's hide, or armoured? | **Hide** ("woody hide": 0 until level 12, then 1 to 3). Its toughness is in its hit points, as a brute's is. |
| Q12 | `chests_per_level` default `[0, 1]`: about one chest every two floors outside treasure rooms? | **Yes**, with the Ascmearc's branches set by hand in C7 (the lairs `[1, 2]`: they are robbers' holds; Fennmynster `[0, 1]`). |
| Q13 | The treasure room's `needs_danger`? | **Not required** (§6.1): the Depths may have one; mimics simply do not come without a danger. |
| Q14 | Should a revealed mimic that is left alone go back to sleep (disguise again)? | **No.** Once seen, a monster; the simplicity is worth more than the trick, and a player who leaves it knows where it is. |
| Q15 | A class ability (a wizard's *Knock*, a priest's *Sanctify the vault*)? | **None now** (§8): `min_class` is 0; *Knock* is a spell for the magic list and goes on #196. |
