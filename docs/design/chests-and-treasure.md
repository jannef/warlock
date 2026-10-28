# Warlock — Chests, mimics, and treasure rooms

Design document for the owner's review (2026-10-28), revised with the owner's decisions the same day (§13). Written against `origin/main` at b9f41e21. Nothing here is implemented; §12 cuts it into issues. It builds on what is there: locks and keys (`docs/dev/doors.md`), traps (`docs/dev/traps.md`), searching (`docs/dev/searching.md`), digging (`docs/dev/digging.md`), the loot roll (`docs/dev/magic-items.md`), the creature rosters and the generated families (`docs/dev/creatures.md`, `docs/dev/animated.md`), and the special-rooms system (`docs/dev/special-rooms.md`, `docs/design/special-rooms.md`). Every number is data and hidden in the game; the wiki states it (the owner's rule: the game is vague, the wiki is accurate).

## Contents

1. Summary
2. The owner's request, and what exists today
3. Chests
4. The chest loot table (shared with mimics)
5. Mimics
6. Treasure rooms and greater treasure rooms
7. Who fights whom: inter-enemy hostility made rare
8. Determinism
9. Talents and class abilities
10. What the player sees; the wiki, the spec, and the docs
11. Choices made along the way
12. Implementation plan
13. Decisions (owner, 2026-10-28)

## 1. Summary

| Piece | Proposal |
|---|---|
| What a chest is | a **fixture of a cell**, like a door: a terrain (`chest`, shut, not walkable; `chest_open` and `chest_smashed`, walkable) plus per-cell state in `LevelState::chests` (its band and what it holds). Its lock lies in `LevelState::locks` and its trap in `LevelState::traps`, the very maps a door's use, so keys, Pick lock, Disarm, searching, and the kick all work on a chest with one extra arm each and **no new action and no new key**: `open:DIR`, `pick:DIR`, `disarm:DIR`, `kick:DIR`, and a bump. §3 |
| Its level | **always the dungeon level** (the floor's item level: a branch floor's depth, a hand-made level's `item_level`, no danger needed). Ten **name bands** in `data/chests.toml` (coffer, chest, hutch, banded chest, strongbox, kist, ark, forcer, silvered ark, gilded ark) say what a chest of that level is called, how sturdy it is, how often it is locked, trapped, or a mimic, and what it holds. The Ascmearc (levels 1 to 12) meets the first five; the rest wait for deeper places. §3.2 |
| Locked, trapped | "they come locked, and often trapped": a lock of the floor's level (only a skeleton key fits a chest's lock, as a tempting hoard's; Pick lock as on a door), and a trap drawn from the same list a door's trap comes from (the branch's `traps`, those that fit). §3.4, §3.5 |
| Smashing | `kick:DIR` at a shut chest: a Strength roll against its sturdiness, as a door's (Lockbreaker counts). Broken, **the trap goes off at the kicker**, and then the chest is **smashed open**: each stack inside is destroyed with 50 %, **coins and gems spared** on a kick. A burst trap (fireball, frost) going off in a chest **blows it up**: then **nothing is spared**, the gold melts with the rest. A blast from outside does the same. Opened with a key, by Pick lock, or by Open on an unlocked chest, nothing breaks, though the trap still goes off at the opener. §3.7, §3.8 |
| The loot | one table per band, dice-driven: items at the floor's level plus the band's bonus through the one loot roll (`loot::roll_with`), with the band's chance of at least magic and of rare; a purse of gold; loose gems by the gems' roll. One table serves chests and the mimics that stand in for them. §4 |
| Mimics | a **generated family** of creature types, `mimic_<L>`, one per level 1 to 60 on the roster formulas (brute role), named by ten bands matching the chests' (mimicling, young mimic, mimic, old mimic, greater mimic, elder mimic, hoary mimic, ancient mimic, primeval mimic, eldest mimic). A chest placed on a floor is a mimic **of that floor's level** with the band's chance; it **carries the loot the chest would have held** (the same draws) and drops it when it dies. It is disguised through a status (`disguised`, as sleepers are `asleep`): it takes no turn, nobody attacks it, and the perception snapshot shows its cell as a shut chest of its band and no actor, so nothing leaks. Revealed by touch (open, pick, disarm, kick, Knock, a bump), by any blow or spell that reaches it, by a search or a Spot check, and by Mimic-wise within two cells. Revealed, it is slow, tough, and sticky: its slam can hold the victim. Faction **vermin**. §5 |
| Treasure room | a special room (`treasury`, "treasure room") on the special-rooms system: 1d2 chests, gold in heaps, and loose gems; **every door of it is a secret door**. On a cave floor it is a walled box stamped into the rock with one secret door onto the cave. Weight 4. §6.1 |
| Greater treasure room | `great_treasury`, "greater treasure room": 2d2 chests of the band above, more gold, more gems; **no door and no corridor, ever**: a sealed box stamped into the rock of any floor, its wall touching a passage. The generator guarantees one diggable wall cell between it and the rest of the level (the **breach**), searching can find that cell ("the wall sounds hollow"), and `connect::validate` learns that a sealed room is reached by digging. Weight 2. §6.2, §6.3 |
| The hint | a new effect field, `arrive`: said on every arrival on the level while the room is unentered: *"Something tells you there is wealth hidden on this level."* / *"Something tells you a fortune lies walled up on this level."* Treasure sense adds the direction. Nothing new is saved. §6.5 |
| Inter-enemy hostility | the owner (q9): dungeons are not a battle royale. Its own slice, H1, independent of chests: the shipped `factions.toml` keeps only the two story feuds (the fey courts, demons against devils) and makes every other pair of monster factions neutral. §7 |
| Talents | two new systems: `chests` (Deft breaker, Chest-wise; Lockbreaker's kick reaches chests) and `mimics` (Mimic-wise, Slick). Treasure rooms extend `special_rooms` (Treasure sense). One class ability: the wizard's **Knock** (a lock springs open). §9 |
| Slices | eight issues in four waves; never more than one at a time that adds save state, and none adds an action. §12 |

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
| The level of a floor | `Content::item_level(id)`: a branch floor's depth, a hand-made level's `item_level` (default 1), else 1. It needs no danger. |
| Loot | `loot::roll_with(content, rng, level, grade, source, luck)` draws a gem or a base item, its rarity, affixes, name, ornament, and material from the stream it is given; `gem::roll` a loose gem at an item level; coins are objects of class `coins` (`gold_coin`, value 1, stackable); shops pay in gold |
| Special rooms | features in `data/room_effects.toml` with weights, `favoured`/`banned` per dungeon, eligibility, `near`/`enter`/`pervade` lines, `feature` and `dressing` terrain, dens, the tempting hoard (`hoard`, named "treasure room", which seals when something is lifted), scriptoria's `loot`; generated last from the map stream, filled from `swarm/<level id>` |
| Creatures | types in `data/creatures/`, numbers by the roster formulas (`content/animated.rs::Numbers::at`, `undead_roster_tests.rs`'s roles), generated families with one type per level (`Animated::member`: `animated_blade`, the four reads of `docs/dev/animated.md`), holds (`held`, `world/hold.rs`), sleepers (`asleep`, `sleeps = true`: no turn, `unaware`), `stray = false` for types placed only by their own rules |
| Factions | `data/factions.toml`: symmetric hostility lists; today most monster factions hunt several others (kobolds the vermin, goblins the kobolds, orcs four, undead three, duergar three, drow four, trolls four, demons five, ...), so a level's folk fight each other on sight |
| Mimics | deferred from the animated-objects design (its §16 D11): "an item that turns out to be a creature; the surprise is its point" |

## 3. Chests

### 3.1 What a chest is, and why not an object

A chest is a **fixture of a cell**: a terrain plus per-cell state, exactly as a locked door is a terrain plus a lock. Not a carried object (decision 1). The reasons:

- Everything a chest does is done to a **cell from beside it**: open, unlock, pick, disarm, kick. Those are the door's verbs, each already an action with a direction prompt, a price, a noise, and a replay form. Locks and traps are per-cell maps; a chest's lock and trap go in the same maps, so `key_for`, `pick_lock`, `disarm`, `hidden_near`, `spring_trap`, and `lock_difficulty` need one more arm each ("a closed door **or a shut chest**"), not a second system.
- A mimic must look like a chest. An actor's cell showing a terrain is one override in `Perception::compute` (§5.4); an actor's cell showing a pile would need the pile machinery to lie.
- Nothing in the game carries other things. Giving `Stack` a contents field would touch every stacking, saving, hashing, shop, and description path for one object.

What is given up: carrying a chest away (ADOM's heavy chests). Nothing in the Ascmearc wants that, and a portable strongbox could come later as an object of its own without touching this.

| | |
|---|---|
| Terrain | three new terrains in `data/terrain.toml`: `chest` (shut: not walkable, transparent, a `description`), `chest_open` (walkable, transparent; "an open chest": one stands in it as in a doorway), `chest_smashed` (walkable; "a smashed chest", splinters; its `toggle` particles a puff of splinters). A chest's **band** is not a terrain: the three terrains serve every band, and the band is in the cell's state, read for the name and the glyph colour. Glyphs: shut `⊟`, open `⊔`, smashed `⊠`, the band's colour (wood browns to iron grey to the arks' silver and gold); the font's coverage is checked in C1, with `(` the fallback. |
| State | `LevelState::chests: BTreeMap<Pos, Chest>` (saved, hashed, left out while empty, so other levels hash as before): `Chest { level: u8, holds: Vec<Stack> }`. The band is **not** stored: it follows from `level` (`ChestBands::band_of(level)`), so a band renamed or re-cut in data never breaks a save. `holds` is ground truth until the chest is open: nothing in perception reads it. An open or smashed chest has no entry: its contents lie in the pile at its cell, which is walkable, and are picked up as any pile (`pickup`). |
| The one way it changes | `World::set_terrain`: a chest's lock stays only on a shut chest (the lock arm becomes "closed door or shut chest"), its trap goes with it as a door's does (open or smashed: the trap is gone), and its `chests` entry is **spilled** (§3.7) by the opening code before the terrain changes. A chest never becomes anything but its open or smashed self; blasts treat the shut chest as terrain with a `hardness` (§3.8). |

### 3.2 The data: `data/chests.toml`

Ten `[[band]]`s, least first. **A chest's level is always the dungeon level** (decision 2): the floor's item level (§2), which needs no danger (decision 13). The band is the last whose `from` the level reaches; it gives the chest its name and its numbers. Level files name a band outright (`chest = "kist"`), which fixes the level at that band's `from`, or a level (`chest = 7`).

```toml
# Chests (spec §8 "Chests"): what a chest of each level is called and what it is like. A
# chest's level is its floor's (a branch floor's depth, a hand-made level's item_level); the
# band is the last whose `from` it reaches.
# Fields:
#   id, name      the band ("banded chest"), as look mode and messages say it
#   description   what look mode adds after the name
#   fg            its colour on the map
#   from          the least level of the band (1 up; bands in rising order)
#   sturdiness    a kick breaks it with a chance of the kicker's Strength in this, as a door's
#                 (terrain.toml), 1 to 100
#   hardness      what a blast must beat to blow it open, as a wall's (1 to 10000)
#   locked        percent of chests of the band that come locked (0 to 100)
#   trapped       percent that come trapped, from the branch's traps that fit (0 to 100)
#   mimic         percent that are a mimic instead (0 to 100)
#   hold          a mimic of the band holds with this chance on a hit, in percent (0 to 100)
#   loot          the table (§4): items = dice, loot_level = levels above the floor's,
#                 magic = percent of items at least magic, rare = percent rare,
#                 gold = dice × each, gems = dice (a result of 0 or less: none)
# [smash]
#   destroyed     percent of each stack destroyed when a chest is kicked open (50)

[[band]]
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
hold = 10
loot = { items = "1d3", loot_level = 0, magic = 10, rare = 1, gold = { dice = "2d6", each = 1 }, gems = "1d4-3" }

[smash]
destroyed = 50
```

| Band | Name | `from` | Sturdiness | Hardness | Locked | Trapped | Mimic | Where met |
|---|---|---|---|---|---|---|---|---|
| 1 | coffer | 1 | 20 | 200 | 60 % | 25 % | 5 % | the first floors of every branch |
| 2 | chest | 3 | 25 | 300 | 65 % | 30 % | 6 % | Cealdhlāw 3–4, Fennmynster 3–4, Fyrdelf 3–4, the lairs 3–4 |
| 3 | hutch | 5 | 30 | 400 | 70 % | 35 % | 7 % | Cealdhlāw 5–6, Fennmynster 5–7, Fyrdelf 5–7 |
| 4 | banded chest | 8 | 35 | 600 | 75 % | 40 % | 8 % | Fennmynster 8, Fyrdelf 8–10 |
| 5 | strongbox | 11 | 40 | 900 | 80 % | 45 % | 9 % | Fyrdelf 11–12 |
| 6 | kist | 15 | 45 | 1200 | 85 % | 50 % | 10 % | deeper places |
| 7 | ark | 20 | 50 | 1600 | 90 % | 55 % | 11 % | deeper places |
| 8 | forcer | 26 | 55 | 2000 | 95 % | 60 % | 12 % | deeper places |
| 9 | silvered ark | 33 | 60 | 2500 | 100 % | 65 % | 13 % | deeper places |
| 10 | gilded ark | 40 | 70 | 3000 | 100 % | 70 % | 15 % | deeper places |

The names are the register the owner set for old things: plain words for the common bands and the Middle English words for the great ones, each explained by its description so a player learns it from look mode. *Hutch* (ME *huche*) is a locking chest of carved wood; *kist* the northern word for a heavy iron-cornered chest; *ark* (Chaucer's *arke*) a great chest of oak and iron; *forcer* (ME, from the Old French *forcier*) a strongbox of iron plates, locked thrice. The silvered and gilded arks are plated. The numbers rise a step a band; a band's `locked`, `trapped`, and `mimic` draws are each one `chance` draw from the chest stream (§8). All of it is percent integers in data: any chance from 1 % up is one line, and a per-mille field is one more line if ever a chance under 1 % is wanted (decision 12).

Sturdiness is on the door's scale (`door_closed` 30): a Strength 12 character breaks a coffer with 60 % a kick, a hutch with 40 %, a strongbox with 30 %; Lockbreaker's +6 counts (§9). Hardness is on the wall's scale (600): a fireball's blast (1000) blows open anything up to a strongbox; the deep bands want a bigger blast than the game has yet.

### 3.3 Placement

| Where | How |
|---|---|
| Generated floors | `chest::fill` (`mapgen/chest.rs`), after `room_loot::fill` and before `gear::arm`, from the new stream `chests/<level id>` (§8): the branch's `chests = { chance = 20, count = [1, 1] }` (`data/dungeons.toml`; the default: **one floor in five** has a chest, decision 12; `count` = [least, most] when it does) decides; each goes on a cell of plain floor of a room (a rooms floor: not the arrival room, not a shop, not a vault, not a special room with a feature or sleepers; a cave floor: any plain floor cell at least 6 cells from the stairs) where nothing lies, stands, shines, or leads away, and which the room's walkable cells stay 4-connected without (`special::joined`, as a feature's cell); then its level (the floor's), lock, trap (§3.5), contents (§4), and whether it is a mimic (§5.3). A chest in a doorway or on a patrol never happens (the same filters as a feature's). |
| Treasure rooms | §6.4, from the same stream, after the floor's own chests |
| Level files and vaults | a legend entry `chest = "coffer"` (a band) or `chest = 7` (a level), with the door's `locked = true` / `lock = "id"` / `lock_difficulty`, the trap fields `trap = "arrow"` (`known`, `dir`), and `objects = ["gold_coin*30", "gem:ruby"]` for what it holds (the `[[pile]]` syntax, `parse_stacks`); `mimic = true` places a mimic of that level instead (§5.3). `content/level.rs` checks: a chest cell is `chest` terrain, a trap on it fits, a pit never, a lock only with the terrain shut, and a key to its lock within reach as for a door's (`check_keys`: a chest's key may be inside another chest, but never inside itself). Vaults may hold chests (the drake's hoard, the robbers' hoard, and the captain's hall are the obvious takers; content, not this design). |
| Dumps | `truth` marks a shut chest's cell with its glyph and lists, dev-only, `chest 7 (hutch) at (x, y): locked, trapped (arrow), holds 3 stacks` per chest; `view` shows what the player sees (§3.9) |

### 3.4 Locks and keys

| Rule | Proposal |
|---|---|
| The lock | `Lock::new(None)` in `LevelState::locks` at the chest's cell: only a skeleton key fits a generated chest (as a tempting hoard's doors), its difficulty `LOCK_DIFFICULTY` (14) plus the branch's `locks_per_depth` (`World::lock_difficulty`, as a door's). A level file's `lock = "id"` makes one its key opens, as a door's. |
| Opening with a key | `open:DIR` at a locked chest with a fitting key carried: `unlock` + `open_chest` (a new action cost, 1000; `data/actions.toml` beside `open_door`), "You unlock the coffer with your skeleton key and open it." Without one: "The coffer is locked." and no time (`open_door`'s own flow, one arm). |
| Pick lock | `pick:DIR` works at a chest as at a door (`World::pick_lock`: the cell check becomes "a locked door or a locked chest"); the Open lock marks and the ability's availability are the door's. A picked lock is removed and the chest opened as `open_chest` does (so its trap goes off at the picker, as a trapped door's does). |
| Knock | the wizard's class ability (§9): the lock springs open, the chest stays shut and its trap untouched; then `open:DIR` as for an unlocked chest |
| Kicking | §3.7: a kick never unlocks, it breaks |
| Creatures | never open chests (they open doors only with `opens_doors`, and a chest is not in their way) |
| Hidden | whether a chest is locked is **not shown** until tried (as a door's lock is not); Chest-wise (§9) shows it |

### 3.5 Traps on chests

| Rule | Proposal |
|---|---|
| Which traps | **the same list a door's trap comes from** (decision 8): the branch's `traps` (`data/dungeons.toml`), those that fit. `TrapEffect::fits_fixture` is `fits_door` renamed (everything but a pit). A branch without traps places no trapped chests (its `trapped` draw is still made, so streams do not move). So the Cealdhlāw's coffers hold arrows, spears, and alarm bells; Fyrdelf's chests fireballs and lightning; Fennmynster's frost and teleports. No chest-only trap kinds. |
| Where it lies | `LevelState::traps` at the chest's cell, `known = false`. Searching finds it as a door's trap (`hidden_near` reads the traps map whatever the terrain; the sign is the chest's: "You notice that the coffer is trapped: an arrow trap."); the passive check and Spot apply; dwarves' `stone` sense does not (a chest is no stone). |
| When it goes off | **on opening** (`open_chest`, with a key or without, and after Pick lock) at the opener, and **on smashing** at the smasher (§3.7), through `spring_trap(at, who)`, aimed from the chest as from a door: a shot and a strike at the victim, a bolt from the chest through the victim, a burst centred on the chest (its own cell never stops it). A step never sets it off (`step_onto_trap` skips chest cells as it skips doors; a shut chest is not walkable anyway). A kick that fails to break the chest springs nothing (decision 5). |
| Disarm | `disarm:DIR` at a chest with a **known** trap: the door's rule, nothing special (`World::disarm`'s cell check reads the traps map). Failed by much, it goes off at the disarmer, and if it is a burst it blows the chest up (§3.7): a botched disarm can cost the loot. |
| A trap that breaks the chest | a **burst** trap (the fireball, the frost) goes off *inside* the chest, so a sprung burst **blows it up**, however it was sprung (opened, kicked, disarmed badly). A shot, strike, bolt, alarm, or teleport leaves the chest as it was (shut if the kick failed; open if it was opened). A spent trap (`charges`) is gone as on a door. Rule: `TrapEffect::blows_fixture()` is true for `Burst`. |
| Known or not | the trap's knowledge is the door's: set off, seen going off, or found; `trap_memory` shows it on the chest's cell as on a door's (the glyph stays the chest's; look mode adds "trapped: an arrow trap") |
| Trap sense | its `trap_damage_percent` halves a chest trap's blow as any trap's (no hook of its own) |

### 3.6 Opening, picking, disarming, and bumping: the flow

`World::open_chest(p)` (`world/chest.rs`), the one way a shut chest opens without breaking, reached from `open_door`'s arm for a `chest` terrain, from `pick_lock`, and from the bump (`player_moves` opens a shut chest the player walks into, as it opens a closed door):

1. Locked and no key: "The coffer is locked.", no time (a kick, Pick lock, or Knock is the way).
2. Price: `unlock` + `open_chest`, or `open_chest` alone; taken before the trap (it may kill).
3. The lock is removed; the trap, if any, goes off at the opener (`spring_trap`); if it was a burst, go to the blow-up (§3.7, step 4b) instead of 4.
4. Spill: the contents go to the pile at the cell (`add_to_pile`, top first in the order rolled), the entry leaves `chests`, the terrain becomes `chest_open` (`set_terrain`, which drops the trap's memory with the trap). Noise: `Act::OpenDoor`'s loudness with the chest's own `toggle_noise` (a creak, 20).
5. "You open the coffer." then the pile line ("You see here ...") as a step would say it, so the player reads what they found. Nothing is identified.

A chest that is open is a pile on a walkable cell from then on; nothing closes it again (no `close:DIR` arm: a closed chest would be an obstacle the player made, with nothing in it).

### 3.7 Smashing, and blowing up

`kick:DIR` at a shut chest (`World::kick`'s terrain match gains a chest arm beside the door's):

1. Price: the plain `kick` (not an attack), as a door's. Noise: `Act::Kick(p)` as a door kick (Lockbreaker's `kick_noise_percent` counts).
2. The break draw: `door::kick_breaks(rng, strength + athletics + door_kick, sturdiness)`, the door's one gameplay draw, against the band's `sturdiness`. Not broken: "You kick the coffer. It holds.", the turn is spent, nothing springs (no limit on tries: time is the price).
3. Broken: the lock is gone; **the trap goes off at the kicker** (the owner's rule), `spring_trap(p, player)`, aimed as from a door. The kicker may die here; the chest still breaks.
4. Then one of two (decision 4):
   - **4a. Smashed open** (no trap, or a trap that is no burst): for each stack in `holds`, in order, one gameplay draw at `[smash] destroyed` (50): destroyed stacks vanish; **coins and loose gems are spared**, never rolled (they do not break under a boot). The rest spills on the cell; the terrain becomes `chest_smashed`. Lines: "The coffer bursts apart!", then, for what broke, one line naming the stacks ("The potion of healing and the scroll of fireball are destroyed."), then the pile line.
   - **4b. Blown up** (the trap was a burst, whoever sprang it: the kicker, the opener, the disarmer): every stack is rolled at the same 50 %, **nothing spared**: a destroyed purse is "The gold runs molten into the cracks." (fire) or is simply gone (frost); gems shatter. Line: "The coffer is blown apart!" then the destroyed line, then the pile line. The 50 % is the same `[smash] destroyed`; only the exemption differs.
5. Athletics marks as a door kick gives them (`Use::Kick`).

**What counts as what**:

| How the chest opened | What happens to the contents |
|---|---|
| a kick that broke it, no burst | smashed: 50 % per stack, coins and gems spared |
| a burst trap going off in it (opened, kicked, or disarmed badly) | blown up: 50 % per stack, nothing spared; the roll runs even if the victim died |
| a blast reaching it from outside (a fireball trap elsewhere, a scroll of fireball, the altar's explosion) with a `blast` above its `hardness` | blown up (§3.8): nothing spared |
| Open on an unlocked chest, a key, Pick lock, Knock then Open | nothing breaks, whatever trap went off (a burst still blows it up: it is a burst) |
| a shot, strike, bolt, alarm, or teleport trap | nothing: these do not touch the box |

A smashed or blown-up chest is a walkable splinter heap with a pile; the state entry is gone, nothing more to do with it.

### 3.8 Blasts, fire, and digging

| | |
|---|---|
| Blasts | `World::blast(center, radius, blast)` breaks every breakable terrain in the burst whose hardness is below the blast; a shut chest is breakable terrain with the band's `hardness` (`TerrainDef::breaks` is read for walls; for a chest the band's number is read in its place): it is **blown up** (§3.7 step 4b, no kicker; its own trap springs at no one: a trap needs a victim, so a chest blown open from afar springs nothing, and its trap is dropped with the terrain). "The coffer is blown apart!" if seen. |
| Fire | no burning of chests or their contents beyond the above (no fire-spreads rule exists). |
| Digging | `dig:DIR` at a chest: refused, "You cannot dig into that." (a chest has no `breaks`: the hardness is read by blasts only). |
| Water and ice | a chest is never placed on water or ice (plain floor only), and never freezes. |
| Creatures | walk around a shut chest as around a rack; never open or break one; a creature's burst spell reaching one blows it up as a trap's does. |

### 3.9 What the player sees

| Where | What |
|---|---|
| The map | the chest's glyph in its band's colour (shut, open, smashed); a remembered chest as last seen |
| Look mode | "a coffer" / "an open coffer" / "a smashed coffer", the band's description, "trapped: an arrow trap" when the trap is known; **never** whether it is locked (Chest-wise: "a locked coffer"), never what is inside until open (then the pile as any pile) |
| Messages | the open, unlock, locked, kick, burst, and destroyed lines above; the trap's own lines; the pile line |
| Perception | `PerceptionView::chest_at(pos) -> Option<ChestLook { band, state }>` on cells seen or remembered (`Perception::remember` keeps it as terrain memory does); the contents and the level are not in it |
| Dumps | `view` the glyph; `truth` (dev) the chests' lines (§3.3); JSON `chests` (seen: `pos`, `band`, `state`) |
| Action text | nothing new: `open:E`, `pick:E`, `disarm:E`, `kick:E`, `move:E`, `use:knock:E` |

## 4. The chest loot table (shared with mimics)

"Chests should contain a good number of randomly generated loot, rolled with dice." One table per band in `data/chests.toml`, applied by `chest::roll_contents(content, rng, level, luck) -> Vec<Stack>` from the chest stream, the **same function and the same draws** for a chest and for the mimic that stands in for it (§5.3):

1. **Items**: `items` dice (one roll); for each, one `below(100)` for the grade (under `rare` rare, under `magic` at least magic, else any: `magic` counts the rare ones, the animated families' grade draw), then `loot::roll_with(content, rng, level + loot_level, grade, Source::Loot, luck)` (its own draws: a gem by chance for grade any, the base, rarity, affixes, name, ornament, material). `None` (nothing found at that level) rolls nothing more for that item.
2. **Gold**: `gold.dice` × `gold.each` coins of the shops' currency (`gold_coin`), one stack (one roll).
3. **Gems**: `gems` dice (one roll; a result of 0 or less, none), each `gem::roll` at `level + loot_level` (the loot's gem draws: gem by rarity, size and cut by their weights).

| Band | Items | `loot_level` | Magic / rare | Gold | Gems | A typical one |
|---|---|---|---|---|---|---|
| coffer | 1d3 | 0 | 10 % / 1 % | 2d6 × 1 | 1d4 − 3 | 2 items, 7 gold, a gem one time in four |
| chest | 1d3 + 1 | 0 | 15 % / 1 % | 2d6 × 2 | 1d4 − 2 | 3 items, 14 gold |
| hutch | 2d3 | 1 | 20 % / 2 % | 3d6 × 2 | 1d3 − 1 | 4 items, 21 gold, a gem |
| banded chest | 2d3 + 1 | 1 | 25 % / 3 % | 3d6 × 3 | 1d2 | 5 items, 31 gold, a gem or two |
| strongbox | 2d4 | 2 | 30 % / 4 % | 4d6 × 4 | 1d2 | 5 items, 56 gold |
| kist | 3d4 | 2 | 35 % / 5 % | 4d6 × 6 | 1d3 | 7 items, 84 gold, 2 gems |
| ark | 3d4 + 1 | 3 | 40 % / 7 % | 5d6 × 8 | 1d3 + 1 | 8 items, 140 gold, 3 gems |
| forcer | 3d4 + 2 | 3 | 45 % / 9 % | 6d6 × 10 | 1d3 + 1 | 9 items, 210 gold, 3 gems |
| silvered ark | 4d4 + 1 | 4 | 50 % / 12 % | 6d6 × 15 | 2d3 | 11 items, 315 gold, 4 gems |
| gilded ark | 4d4 + 2 | 5 | 60 % / 15 % | 8d6 × 20 | 2d3 + 1 | 12 items, 560 gold, 5 gems |

For scale: a generated floor lays 2 to 4 items of loot (`loot = [2, 4]`), a tempting hoard 3 at +3, a den's hoard one per four sleepers, and Cēaptūn's start gives 60 gold. A coffer is a floor's worth of loot and a few coins; a banded chest on Fyrdelf 10 is two floors' worth with a quarter of it magic. The finder's luck (Treasure hunter, Hoard sense) applies through `roll_with` as to any find. The whole table is data, tuned after a play-through, and the wiki prints it with the means.

Coins and gems in a chest are what makes smashing a real choice: a Strength character without a key or Pick lock loses half the items on average but keeps every coin and gem (§3.7), unless the chest was trapped with a fireball, in which case the kick costs them everything.

## 5. Mimics

### 5.1 The family and its bands

Mimics are a **generated family** of creature types, as the animated blades are (`docs/dev/animated.md` "Families"): `data/mimics.toml` names ten `[[band]]`s, and content loading makes one type per level, `mimic_<L>` for L = 1 to `levels = [1, 60]`, numbered after the animated families and before the campaigns' types (the same splice; adding a band or a level shifts ids as adding a body does: the integrator's format bump). **A mimic's level is always the dungeon level** (decision 2): a chest on depth 7 that is a mimic is `mimic_7`, and its band, the last whose `from` it reaches, gives its name. The bands' `from`s are the chests' (§3.2): a mimic's naming band is the band of the chest it stands in for.

Numbers by level from the roster formulas (`Numbers::at(L, brute)`: Toughness +5, base HP 2.5 × L × 1.6 = 4L, the dice one rung above the level's target; to-hit and natural Defence as #383's conventions give a brute; `content/mimic_roster_tests.rs` holds every level to ±35 % as the undead roster is held). Levels are the floor's, so a mimic is never out of depth; its brute numbers and its hold make it a hard fight for the floor, which is right for a thing the player chose to touch. `xp = 5 × level + 10`.

| Band | Name | `from` | St | To | Dx | Wp | Base HP | Dice | Hold |
|---|---|---|---|---|---|---|---|---|---|
| 1 | mimicling | 1 | 11 | 17 | 10 | 16 | 4 | 1d6 | 10 % at 12 |
| 2 | young mimic | 3 | 12 | 18 | 10 | 17 | 12 | 1d8 | 10 % at 12 |
| 3 | mimic | 5 | 13 | 19 | 11 | 17 | 20 | 1d10 | 15 % at 13 |
| 4 | old mimic | 8 | 14 | 20 | 12 | 18 | 32 | 2d8 | 15 % at 13 |
| 5 | greater mimic | 11 | 15 | 21 | 12 | 19 | 44 | 2d8 | 15 % at 14 |
| 6 | elder mimic | 15 | 17 | 23 | 13 | 21 | 60 | 3d8 | 20 % at 15 |
| 7 | hoary mimic | 20 | 19 | 25 | 15 | 23 | 80 | 4d6 | 20 % at 16 |
| 8 | ancient mimic | 26 | 21 | 27 | 16 | 25 | 104 | 3d10 | 20 % at 17 |
| 9 | primeval mimic | 33 | 24 | 30 | 18 | 27 | 132 | 4d8 | 25 % at 18 |
| 10 | eldest mimic | 40 | 27 | 33 | 20 | 30 | 160 | 3d12 | 25 % at 20 |

(The row shows the band's first level; every level between has its own type on the same formulas. Intelligence and Presence 2, as the mindless have; the hold's chance is the band's `hold` in `data/chests.toml` and its difficulty a drain's, 12 + L ÷ 5.) The names are an age ladder (decision 3; the owner's "least ... ancient" ladder with ten rungs): a thing that sits in the dark getting bigger. The one made-up word, *mimicling*, is the hatchling; the rest are plain English words of age.

```toml
# Mimics (spec §8 "Mimics", docs/design/chests-and-treasure.md §5): a thing that sits shaped as a
# chest until something touches it. One type a level, `mimic_<L>`, on #383's formulas for a brute;
# the bands name them, and match the chests' bands (chests.toml).
[creature]
glyph = "m"
faction = "vermin"
tags = ["mimic"]
alignment = "neutral"
vision = "darkvision"
sight = 6
hearing = 60
speed = 800
size = "medium"
budge = "never"
stray = false
rarity = "rare"
immune = ["poisoned", "asleep"]
corpse = true
on_eat = { vomit = 60, harm = "1d3" }
blood = "#5a4010"
attack = { damage_type = "bludgeoning", name = "pseudopod", verb = "slams" }
natural = { name = "woody hide" }
levels = [1, 60]

[[band]]
id = "mimicling"
name = "mimicling"
from = 1
fg = "#9a7a48"
weight = 20000
```

### 5.2 Body, and what it is

| | |
|---|---|
| Faction | **`vermin`** (decision 9): hostile to the player, an animal faction. Today the kobolds, goblins, orcs, giants, and trolls list the vermin as prey, so a *revealed* mimic would be set upon by a kobold band; after H1 (§7) no monster faction hunts the vermin, and a revealed mimic and a kobold leave each other be. While disguised, nobody attacks it in any case (§5.4). `animal = true` is the faction's: a Beast-friend's mimic is calm toward them until struck, an animal that lies still; a Beast-friend who opens it gets the reveal and no free blow. `stray = false` and no `population` names the family: a mimic comes **only** through a chest (§5.3) or a legend's `mimic = true`. |
| Tags | `mimic` (a new tag; a kill quest's `tag:mimic` counts the family once, as `tag:animated` does) |
| Body | hide `woody hide` (the body rule's "hide": Protection (L − 10) ÷ 8, so 0 until level 10, then 1 up to 6 at level 60; `creature_body_tests` names it, decision 11); medium size (weights rise by band, 20 to 600 kg: no large size yet, as the giants); `footsteps` 0 (it oozes); `hearing` 60; darkvision, sight 6; `speed` 800 (slow: it is a box with a mouth; fleeing works, which is the counterplay for a weak character); `budge = "never"`; `opens_doors` false |
| Corpse | yes, a "mimic corpse", `on_eat = { vomit = 60, harm = "1d3" }` (glue and bile); blood a dark amber |
| Immune | `poisoned`, `asleep` (it never sleeps; a den's filler never draws it anyway) |
| Natural attack | a pseudopod slam, bludgeoning, the level's dice, with a **hold** (`held`, `world/hold.rs`: the existing status; Strength struggles against the hit's `difficulty`): "sticky". No drain, no disease. |
| What it drops | what it carries (§5.3): the chest's loot, falling where it dies as anyone's pack does; its `loot` percent is 0 (no extra drop; the chest is the drop) |
| Stealing | `World::steal` refuses a mimic ("It has no pockets."), no time; stealing from a chest is opening it |
| Alignment | neutral (an animal); killing one is a hostile kill, as any |

### 5.3 How a chest becomes a mimic

In `chest::fill` (§3.3), after a chest's level, lock, trap, and contents are rolled, on **any** floor (no danger needed, decision 13: the level is the floor's item level): one `chance(band.mimic)` draw of the chest stream. True: no chest is laid; instead a `CreatureSpawn` of `mimic_<L>` (`Content::mimic_at(level)`: the family's member for that level, the top member for a level beyond `levels`), `disguised: true`, `rolls_gear: false`, with **the rolled contents in its pack** (`carry`), is pushed on the template's spawns after the population's. The lock and trap draws were made but are dropped: a mimic is neither locked nor trapped (nothing says "The coffer is locked." of a mimic: touching it reveals it first, §5.5). The streams are the same with or without the swap: the draws are the chest's, and the mimic's `make_creature` draws nothing (no gear, no body).

So the owner's "same loot table as the chest they mimic" is exact: the mimic carries precisely the stacks the chest would have held, rolled by the same draws, and the player gets them by killing it instead of opening it (and gets them whole: a mimic is never smashed or blown up).

A level file's `mimic = true` on a `chest = ...` entry places that level's mimic with the entry's `objects` in its pack (and refuses `locked`, `lock`, and `trap` beside it). The dev `spawn` action can bring one as any type (then it carries nothing and is disguised on arrival if the cell is free plain floor, else revealed).

Treasure rooms (§6.4) draw their chests through the same `fill`, so a treasure room's chest can be a mimic (the band's chance, no more: a treasure room is not a mimic nest).

### 5.4 The disguise in the perception snapshot

The mimic is an ordinary actor in `WorldState::actors` with the status **`disguised`** (`data/status.toml`, beside Asleep: `max = 1`, no `eases`, `seen` none, a new flag `hides = true`). The status is the whole record: saved and hashed with the actor, nothing new in the save format.

| What | Rule |
|---|---|
| Turns | `creature_turn`'s `slumbers` check becomes "sleeps or hides": a disguised mimic's turn is a wait (strided in a rest), its `heard` cleared, it never reaches `decide`. It does not move, so the chest the player remembers is where it was. |
| Other creatures | the AI's target search (`nearest_hostile_in_sight`, `fight`) **leaves out** an actor with a `hides` status, one line, so no kobold ever sets on a "chest" in front of the player (a leak and a nonsense both). Creatures path around it as around a rack (`budge = "never"`). After H1 the line matters only for the player's own allies, should any come. |
| Perception | `Perception::compute` (`perception.rs`) treats a disguised actor as **not an actor**: it is not in `actors`, `sees_actors_at` is false for its cell, and the cell's **terrain look** is `chest` of its band (`ChestLook { band, state: Shut }` in `chest_at`, the same struct a real chest gives, §3.9). Terrain memory remembers "a chest" there. Nothing else in the snapshot (sounds: it makes none; health; stance; `shape`) exists for it. |
| The rules that read perception | `creatures_in_view` (rest interruption) leaves it out, as it leaves images out; `hostiles_here` (the spawn cap) **counts** it (it is a hostile on the level; a floor with a mimic spawns one fewer, which nobody notices); Sixth sense's "Something sleeps on this level." is not said for it (it does not sleep); `sense_danger` ignores it. |
| Light | it carries none; a disguised mimic's cell is lit as its terrain would be (a shut chest is transparent and not walkable, like a rack) |
| Blocking | an actor blocks a cell as a shut chest does; a step into it is a bump (§5.5). Actors never block sight, and a shut chest's terrain is transparent, so nothing differs. |
| Dumps | `view` shows the chest; `truth` and `actors` (dev) show the mimic, `actors` adding `disguised`; JSON `actors` leaves it out until revealed, `chests` lists its cell as a shut chest |
| Look mode | the chest's words, the band's description: nothing a real chest would not say. Mimic-wise (§9): "a young mimic (hostile, unhurt), shaped as a coffer" within its reach. |
| Leaks closed | the disguise is decided in the one place the snapshot is built; `PerceptionView::chest_at` is the only reader; the rules never ask perception. The test `mimic_tests::a_disguised_mimic_is_a_chest_in_every_snapshot` compares the snapshot of a level with a mimic against the same level with a real chest in its place (JSON equal). |

### 5.5 Revealing

`World::reveal_mimic(id, how)` (`world/mimic.rs`) removes `disguised` (not through `ended`: nothing is said of a status the player never saw), says the line, and gives it its turn as usual from then on. It is called from one line in each place a chest is touched or a creature is hurt:

| How | Where | Line (if the cell is seen) | Then |
|---|---|---|---|
| **Touch**: open, unlock, pick, disarm, kick at its cell, or a bump into it | `open_chest`, `pick_lock`, `disarm`, `kick`, `player_moves`: each first asks `World::mimic_at(p)` (a disguised actor there) | "The coffer lurches: it is a young mimic!" | the touch's action is **refused with no time** (the player meant to open a chest; nothing was done), and the mimic gets a **free blow** at the toucher at once (`World::free_blow`, the existing free-blow code, as a sneak attack's surprise: the owner's "surprise is its point"), with its hold. Trap sense and Light step do nothing: this is no trap. |
| **Knock** at its cell (§9) | `knock` | the same line | revealed, the mana spent, **no** free blow (the caster's touch was a spell, not a hand) |
| **A blow**: any melee hit or miss at it, a missile landing on its cell, a bolt or burst reaching it, a trap's burst | `World::hurt` (as a sleeper wakes on harm) and `shoot.rs::fly` (a missile stopping at an actor's cell stops at it: the disguise does not make missiles pass) | "The coffer shudders: it is a young mimic!" | revealed, no free blow (the player struck first) |
| **Searching**: the active search (`search`, `search:N`) and the passive check each turn, with Spot or the race's `passive_search = "all"` | `hidden_near` lists a disguised mimic within reach as `Hidden::Mimic(id)` with difficulty 12 + L ÷ 5 + the branch's `hidden_at(depth)`; `notices`/`search_turn` make the one check; `reveal` reveals it | "You notice the coffer breathing: a young mimic!" | revealed; the player now chooses the first move: shoot it, or walk away. A Search mark as for any find. |
| **Mimic-wise** | `Perception::compute` with the talent's reach (§9): within it the disguise is not applied | look mode says what it is; nothing is said | not revealed in the rules (it still waits); the player just sees it |
| **Being disturbed** | nothing: noise, light, time, and a creature walking past never reveal it (as sleepers never wake on their own). The player decides when the chest is touched. A revealed mimic never disguises itself again (decision 14). |

A revealed mimic is a plain hostile creature of the vermin: it closes in and slams; the AI needs nothing new. At speed 800 it is left behind by anyone who walks away, so the fight is always chosen.

**Sneak attacks**: a disguised mimic is `unaware` (as a sleeper is), so a thief who sees it for what it is (Mimic-wise, or found it by searching and it has not yet acted) lands a sneak attack: the reward for the Spot. After its first turn it is aware as any creature.

### 5.6 Dumps and the roster test

`content/mimic_roster_tests.rs`: one type a level over `levels`, the band names in order with rising `from`s equal to the chests' bands', every level on the guideline and within ±35 % of the brute formulas, `stray = false`, in no population, the `mimic` tag, the vermin faction, `budge = "never"`. `world/bestiary_tests.rs`'s `WAVES` gains the family (shared). The `actors` dev layer adds `disguised` after the statuses; `truth` marks its cell `m`. The wiki's creature tables show the family once (its first member's row, as the blades) and `xtask/src/wiki/mimics.rs` prints the per-band numbers.

## 6. Treasure rooms and greater treasure rooms

Both are **features** on the special-rooms system (`docs/dev/special-rooms.md`): an `[[effect]]` each in `data/room_effects.toml` with a weight, drawn by `special::pick` like any feature, placed by `special::place`, and filled from the chest stream. Each is **sealed** in its own way; the rules of the chests inside are §3's. Neither pervades, neither has a `status`, neither needs a danger (decision 13: the chests' level is the floor's item level, and the mimics' with it); both carry the new `arrive` line (§6.5) and `treasure` (their contents, §6.4), and the greater one `sealed = "dig"`.

### 6.1 Treasure room

| | |
|---|---|
| id, name, mark | `treasury`, "treasure room", `$` (the tempting hoard's mark moves to `T`; its name becomes "tempting hoard", §6.6) |
| Access | **every door of the room is a secret door** (`secret_doors = "all"` on the effect, read by the generator, §6.3). The room is found by searching (the branch's secret-door difficulty, `search.toml`'s `secret_door` plus `hidden_at(depth)`), by the arrival hint's prompting, or by digging in from a neighbouring room or corridor. |
| Contents | `treasure = { chests = "1d2", band_up = 0, gold = { heaps = "1d4+1", dice = "3d6", each = 2 }, gems = "1d3" }`: one or two chests of the floor's level, two to five heaps of 6 to 36 gold each, one to three loose gems on the floor (§6.4) |
| Messages | `arrive` "Something tells you there is wealth hidden on this level." `near` none (its doors are secret: nothing is glimpsed from outside until a door is found; once found, the first look in is the enter line's job). `enter` "Gold glints in the dust. No one has been here in a long time." |
| Eligibility | a room with at least one door and no stairs, not the arrival room, not a vault, not locked, not a shop's room, not on a patrol (a rooms floor); any cave floor with rock enough for the box (§6.3) |
| Weight | 4 (3.0 % of special rooms, about 1.8 % of generated floors: one in about 55; the Ascmearc's 26 dungeon floors hold about one in two games, more in Bānfeld and Fennmynster, which favour special rooms). Tuned after play. |
| Fields | `secret_doors = "all"`, `treasure`, `arrive`, `enter`, `weight = 4`, `min_cells = 9` |

### 6.2 Greater treasure room

| | |
|---|---|
| id, name, mark | `great_treasury`, "greater treasure room", `G` |
| Access | **no door and no corridor, ever** (`sealed = "dig"`, decision 6): a sealed box in the rock, reached only by breaking a wall, with a digging tool (`dig:DIR`), or with a blast (a scroll of fireball at the wall: its 1000 beats `wall`'s 600). The arrival hint says it is walled up; searching beside its breach cell finds it (§6.3). |
| Contents | `treasure = { chests = "2d2", band_up = 1, gold = { heaps = "2d4+2", dice = "4d6", each = 5 }, gems = "2d3" }`: two to four chests of the **band above** the floor's (`band_up`: that many bands up the ladder, the last staying last; a chest's level is still the floor's, its band's numbers and table the higher band's), four to ten heaps of 20 to 120 gold, two to six gems |
| Messages | `arrive` "Something tells you a fortune lies walled up on this level." `enter` "The air is stale and heavy with the smell of old gold." |
| Eligibility | the level has a spot for the box (§6.3), decided before the draw (as a beast den's `peopled` is), so no draw is wasted |
| Weight | 2 (1.5 % of special rooms, about 0.9 % of floors: one in about 110; a game in two finds one). The owner's "vast fortune" is a story, not a feature of every run. |
| Fields | `sealed = "dig"`, `treasure`, `arrive`, `enter`, `weight = 2`, `min_cells = 9` |

### 6.3 How mapgen seals a room and guarantees a way in

**Treasure rooms on rooms floors** (`mapgen/rooms.rs`, after corridors, doors, stairs, keys, and `special::place`, before `hide_secrets`): every door of the chosen room (closed, open, or a doorway: each made a closed door first, as `lock_rooms` does) becomes a **secret door** by the generator's own rule (`secret_look`, `template.secrets`), **not** counted against the branch's `secret_doors` (it is the room's nature, not the level's quota; a branch with `secret_doors = [0, 0]` still has them). `connect::validate` passes as before: it reaches every cell treating secret cells as doors, and the stairs and keys without them, and a treasure room holds no stairs and no key (eligibility; `lock_rooms` drops keys in open rooms only, and `chest::fill` never puts a branch key in a chest). `hide_secrets` leaves them alone (they are secret already). A secret door is always findable (no limit on searching) and always diggable around (its wall is `wall`).

**Greater treasure rooms, and treasure rooms on cave floors**, are **boxes stamped into the rock** by `special::stamp` (a new placement mode beside `make`), through the vault stamper's spot search (`vault::spots` on the generated template: a rectangle of `min_cells` plus its walls, among every spot where it fits in rock; a cave fills cave with rock first where no rock is big enough, as for pinned vaults, `caves::make_room`). The box's interior is the palette's floor, its ring the palette's `wall` (hardness 600, never `void`: `void` has no hardness). On a rooms floor the box is stamped **after** the corridors are dug, in the rock they left, so no corridor leads to it and none is dug to it later. Then:

| Feature | The box |
|---|---|
| treasure room (caves only) | one cell of the ring facing the cave (4-adjacent to cave floor; else the end of a tunnel dug from the ring to the cave as a vault's entrance tunnel is, `MAX_TUNNEL`) is a closed door made secret |
| greater treasure room (both generators) | **no door**. The spot is chosen among those where at least one ring cell is 4-adjacent to a walkable cell **reachable from the up staircase** (`connect::distances`; a corridor cell, a doorway, or cave floor) and not on the level's edge; that cell is the **breach**, `SpecialRoom::breach: Option<Pos>` (saved, hashed, left out while `None`). No such spot: the feature is not eligible on this level (§6.2), and the draw goes to another feature. Where a reachable wall of another room is the only neighbour, the spot is still refused: one wall to dig, never two. |

Both generators then run `connect::validate` with one new rule: the walkable cells of a room with a `breach` need not be reachable, **provided** the breach is `wall` with a `hardness`, not on the edge, and 4-adjacent both to a cell reachable from the up staircase and to a cell of the room. A level that fails it is regenerated as any invalid level is (the generator never produces it on purpose: the spot satisfied it by construction). `hide_traps` never puts a trap on a cell of a sealed room; the key rule and the quest roller's placement both exclude sealed rooms (no stairs, key, or quest target is ever walled up: a greater treasure room is never required for anything).

**Finding the breach.** With no corridor there is no tell on the map (decision 6), so the breach is a **hidden feature**: `Hidden::Hollow(pos)`, listed by `hidden_near` for a breach within reach, found by the one search check (active or passive, Spot and the dwarves' `stone` sense included: a wall is stone) at the branch's secret-door difficulty; found, "The wall here sounds hollow.", and the cell is remembered as a **hollow wall** (a terrain look in memory, as a known trap is: `LevelState::hollows` holds the found breaches, saved, left out while empty; look mode "a hollow wall"). The player then digs it or blasts it. Treasure sense's bearing (§6.5) narrows where to search; a thorough player searches the walls along the bearing; a lucky one digs there without searching, which also works (any wall of the box is `wall`: the breach is the one wall *guaranteed* to be reachable and thin, not the only one that can be dug).

**Levels where digging is impossible.** There are none by terrain: every generator wall is `wall` (hardness 600) or `packed_earth`, and the only harder shipped stone is `granite` (3000, still diggable). What can be missing is the **tool**: a character without a pick-axe or a blast cannot enter a greater treasure room, and that is the design (the owner: "they need to be accessed by digging"). The arrival hint tells them something is there; the hollow wall shows them where; Tunneler, a pick-axe from the quarry or a shop, a dwarf's digging, or a scroll of fireball is the answer, in this visit or the next. On a hand-made level `[[room]] effect = "great_treasury"` is accepted only if the room's cells are unreachable without digging and a `breach` cell is given (`content/level.rs` runs the same check).

### 6.4 Contents

`treasury::fill` (`mapgen/treasury.rs`), after `chest::fill` on the chest stream, for each special room whose effect has `treasure`:

1. **Chests**: `chests` dice (one roll); each a chest of the floor's level, its band raised by `band_up` (§6.2), placed by `chest::place` (the same cell filters, the same lock, trap, contents, and mimic draws as a floor's chest: §3.3, §5.3) on the room's plain floor, drawn anew; a room too small for them all holds what fits.
2. **Gold**: `gold.heaps` dice (one roll); each a stack of `gold.dice` × `gold.each` gold coins on a plain floor cell drawn anew (two heaps on one cell join: coins stack).
3. **Gems**: `gems` dice (one roll); each a `gem::roll` at the room's level on a cell drawn anew.

Nothing else: no guardians, no sealing-on-lift (that is the tempting hoard's game), no sleepers. A treasure room's danger is its mimics and its traps, and the time spent finding it. The tempting hoard stays the greed test; the treasure room is the explorer's reward.

The `rooms` dev layer lists the room's chests, heaps, and gems as placed, and a sealed room's `breach (x, y)`; `truth` shows them (and a mimic as `m`).

### 6.5 The hint on arrival

| | |
|---|---|
| Field | `arrive` (a message) on an effect, beside `near`, `enter`, and `pervade` |
| Rule | said in `World::travel` (and at the start of a run), after the arrival lines and the `pervade` line, **once per arrival**, for each effect with an `arrive` that has a room on the level the player has not `entered` (one line per effect, not per room: two treasure rooms say it once). Left once the room is entered. Every arrival until then (decision 7). |
| Stored | nothing new: `SpecialRoom::entered` is the record. The line is said again on every return to the level while the room waits, which is the point: it nags. |
| Words | treasure room: *"Something tells you there is wealth hidden on this level."* greater: *"Something tells you a fortune lies walled up on this level."* Plain present-day English for the player's own inner sense (as the `pervade` lines are: "The dead do not rest on this level."); "walled up" is the one word of mechanics, and it is the clue the owner asked for. No "you sense": the game already says "Something sleeps on this level." for Sixth sense, and the shape matches. |
| Treasure sense | the talent (§9) adds the bearing: *"..., to the north-east."* (the room's middle from the arrival cell, eight winds) |
| Also | Sense of place (the existing talent) names a glimpsed room; a treasure room is never glimpsed before it is found, so it adds nothing here |

### 6.6 The tempting hoard's name

`data/room_effects.toml`'s `hoard` is named "treasure room" today. With a feature of that name arriving, the hoard becomes **"tempting hoard"** (its id `hoard`, its mark `T`, its rules, Hoard-wise's text "You know how a treasure room is baited" becomes "how a tempting hoard is baited"), so look mode and the wiki tell the two apart. Data and two strings; no save changes (effects are saved by index).

## 7. Who fights whom: inter-enemy hostility made rare

The owner (decision 9): a mimic hostile to everyone is realistic but dull, because a dungeon where every faction fights every other is a battle royale in which the player picks off the stragglers; **inter-enemy hostility should be much rarer in general**. This is its own slice, **H1**, independent of chests and ahead of them in the queue if the integrator likes (data only).

**Today** (`data/factions.toml`): 19 factions; besides the player, the kobolds list the vermin; the goblins the vermin and kobolds; the orcs the vermin, kobolds, undead, and duergar; the undead the kobolds, goblins, and giants; the cult the marsh; the duergar the goblins, undead, and giants; the drow the kobolds, goblins, orcs, and duergar; the giants the vermin and kobolds; the trolls the vermin, kobolds, undead, and duergar; the demons the kobolds, goblins, duergar, marsh, and devils; the Seelie the Unseelie (and the wrathful fey them). About thirty hostile pairs among monsters, so on Fyrdelf (kobolds, duergar, ogres, trolls, demons, undead, orcs, hobgoblins) much of a floor's population is at war with the rest of it.

**Proposal.** Keep only the **two story feuds**, and make every other pair of monster factions neutral:

| Kept | Why |
|---|---|
| Seelie ↔ Unseelie, wrathful fey ↔ Unseelie | the courts' war is what the fey are (#452); a Seelie fey is the player's only friend down there |
| demons ↔ devils | the Blood War, the one feud the lower planes are known for (#448, #449) |

Everything else (the tribes' raids, the dead against the living, the duergar's grudges, the drow's rivalries, the cult against the fen) becomes **lore in the file's comments**, not a stance: the kobolds still *say* they hunt the vermin, but a kobold and a rat on the same floor leave each other be, and so do an orc band and the dead. The player is every monster's enemy, as now; the monsters are not each other's. The comments stay as written (they are good world text) with one line each saying the feud is not fought on the floor.

| | |
|---|---|
| Data | `hostile` lists cut to `["player"]` on every faction but the four above; the vermin stay `animal`, the Seelie keep `provoked` |
| Code | none: `Factions` and `Content::stance` read the lists |
| Tests | `world/bestiary_tests.rs`'s stance rows (each tribe's "hostile to X" assertions become "neutral to X" for the dropped pairs); each roster test's faction assertions (`orc_roster_tests`, `troll_roster_tests`, ...); the fixture factions files (`world/fixture_factions.rs` and the four tests that write their own) are untouched (they list ids, not stances) |
| What changes in play | dens of mixed folk stay quiet until the player wakes them; a floor's population walks its routes without thinning itself; the arena scenario's fight (kobolds against giant rats, `arena`) **stops**: that scenario's level gives its rats a `faction` of their own that lists the kobolds (a level legend's `faction` override, `docs/dev/content.md`), so the showcase keeps its fight and the bench-turns row its budget |
| The wiki | the factions table (`xtask/src/wiki`'s creatures module) prints the new lists by itself; the Creatures page says in one sentence that monsters do not fight each other but for the fey courts and the fiends |
| Later | a `rivalry` percent (a chance a creature of one faction attacks one of another on sight, instead of always) would bring back the occasional skirmish without the royale; not now (a line on #196) |

With H1 in, a revealed mimic in the vermin is hostile to the player alone, which is what the owner wants of it.

## 8. Determinism

| Draw | Stream |
|---|---|
| which room is a treasury, a box's spot and its breach | the level's **map** stream (as every feature: `special::pick` and `place`) |
| a floor's chests: whether one comes and how many, their cells, locked, trapped and which trap, contents (§4), mimic or not | new **`chests/<level id>`** (`Rng::chests`), after `room_loot`'s swarm stream and before gear, so loot, population, dens, hoards, scriptoria, and gear streams do not move; a level is the same with or without chests |
| a treasure room's chests, heaps, and gems | the same chest stream, after the floor's chests |
| a mimic's loot | the chest's draws (the swap draws nothing more) |
| the kick's break, the 50 % destruction per stack, the trap's rolls, the free blow, the hold, the search checks (a mimic's, a breach's) | the **gameplay** stream (they happen in play) |
| a blast blowing a chest up | none: a comparison |
| the arrival hint, Treasure sense's bearing | none: state and geometry |

`Rng::chests` joins `rng.rs`'s table and `spec/08j-determinism.md`'s. Adding chests moves no existing stream and changes no existing snapshot but the Depths' terrain where a chest or a stamped box lands (reviewed once, in C1 and C5: the `special` draw changes which effect existing seeds get, as every added feature does). H1 moves no stream at all.

## 9. Talents and class abilities

Two new systems join `data/talents.toml`'s list (after `doors`, before `traps`, by topic): `chests` and `mimics`. Treasure rooms extend `special_rooms` (no new tag: "a new trap kind needs no new tag"). Each talent is one file under `data/talents/`, one hook each.

| System | Talent (tree) | Effect | Hook (one read) | Issue |
|---|---|---|---|---|
| `chests` | **Deft breaker** (might; Strength 12) | when you kick a chest open, what is inside breaks half as often (a blown-up chest is not a kick: no help) | the smash draw's percent (`TalentMods::smash_destroy_percent`, −50), read in step 4a only | C1 |
| `chests` | **Chest-wise** (delver; Intelligence 11 or Search 15, `either`) | you tell a locked chest from an unlocked one at a glance, and a chest's trap gives itself away to you as you look at it (a chest's trap within sight becomes known) | `PerceptionView::chest_at`'s `locked` word; `learn_traps` on chest cells in view | C1 (lock), C2 (trap) |
| `chests`, `doors` | **Lockbreaker** (existing) | its door kicks reach chests: `door_kick` adds to the smash roll, `kick_noise_percent` to its noise | the smash roll reads the same mods the door kick does; the talent's `systems` gains `chests` | C1 |
| `chests`, `doors` | class ability **Knock** (wizard; `class` tree, given by the wizard's table at level 4, `TalentEffect::Knock { mana = 5 }`; `Aim::Direction`; `use:knock:DIR`) | a lock springs open: the locked door or chest that way is unlocked (the lock removed, "The lock springs open with a click."), nothing opens and no trap springs; refused with no time where nothing locked stands (the same line for an unlocked chest, so a try leaks nothing); at a disguised mimic, the reveal (§5.5) and the mana spent. Greyed out in the menu without the mana. | `world/knock.rs`, one arm in `use_class`; the lock maps | C1 |
| `mimics` | **Mimic-wise** (scholar; Spot 10 or Intelligence 12) | a disguised mimic within two cells is seen for what it is | `Perception::compute`'s disguise (`TalentMods::sees_mimics`, a reach, as Mirror-wise's) | C4 |
| `mimics` | **Slick** (finesse; Dexterity 13) | a mimic's grip never holds you | `World::afflict` of `held` by an actor with the `mimic` tag (`TalentMods::slick`; the one check beside immunities) | C4 |
| `special_rooms` | **Treasure sense** (wayfarer; Spot 10) | the hint on arriving on a level with a treasure room tells you which way it lies | the `arrive` line's bearing (`TalentMods::treasure_bearing`) | C5 |

`chests` ships in C1 with Deft breaker, Chest-wise's lock half (its trap half following in C2 on the same file), and Knock (decision 15: "knock works"; a class ability as Consecrate is, paid in mana, in the one abilities menu, no key of its own; a scroll of knock for the magic list is one data file later and not part of this). Nothing is sanctified: a treasure room is just treasure (decision 15). `mimics` ships in C4 with both its talents. No new `KeyCommand`: every verb here is an existing one or an ability.

## 10. What the player sees; the wiki, the spec, and the docs

| Where | What |
|---|---|
| Messages | §3.6, §3.7, §5.5, §6.1, §6.2, §6.3, §6.5 |
| Look mode | a chest by its band and state, its known trap, "a locked coffer" with Chest-wise, a mimic as a chest (or as itself with Mimic-wise), "a hollow wall" once found, "in a treasure room" once entered |
| Never shown | a band's chances and dice, a chest's level, what a shut chest holds, whether it is locked (without the talent), whether it is a mimic, a mimic's level or numbers, a treasure room's existence beyond the hint, where the breach is until searched out |
| Abilities menu | Knock under the wizard's class abilities; Pick lock and Disarm as they are |
| Action text | `use:knock:DIR`; nothing else new |
| Dumps | `truth` (chests, mimics, breaches), `actors` (`disguised`), `rooms` (`breach (x, y)`, the contents placed), `view` and JSON `chests` as the player sees them, JSON `known_hollows` |
| Wiki | a **Chests** page (`wiki/src/chests.md`, under "Objects and your pack" beside Gems, in `SUMMARY.md` by topic) with generated tables (`xtask/src/wiki/chests.rs`): the bands (level, sturdiness, kick odds by Strength, hardness, locked, trapped, mimic), the loot table with its means, the smash and blow-up rules and what each spares; the **Special rooms** page's generated feature table gains the two rooms (weights, shares, the hint lines); the creatures tables gain the mimic family (the first member's row) and `xtask/src/wiki/mimics.rs` the per-band numbers; the Traps page says a chest's traps are a door's and that a burst blows it up; the Talents and Classes pages' rows (Knock); the Creatures page's factions sentence (H1) |
| Spec | `spec/07b-data.md` "Chests" (the data file) and "Mimics" (the family file); `spec/08d-traps-doors-digging.md` "Chests" after "Trapped doors" (locks, traps, opening, smashing, blowing up, the 50 % rule and what it spares, blasts, the hollow wall); `spec/08b-creatures.md` "Mimics" and the factions paragraph (H1); `spec/07d-shops-and-rooms.md` "Special rooms" (the two features, `arrive`, `secret_doors`, `sealed`, the breach, the hoard's new name); `spec/08g-magic-items.md` "Chest loot"; `spec/08i-character.md` the wizard's Knock; `spec/08j-determinism.md` the stream |
| docs/dev | a new topic file **`docs/dev/chests.md`** (chests, their loot, the streams, the mimic swap, Knock; listed in CLAUDE.md's index in C1), **`docs/dev/creatures/mimics.md`** (the family file, per the creatures rule), and a "Treasure rooms" section in `docs/dev/special-rooms.md`; `docs/dev/creatures.md`'s faction paragraph (H1); `docs/dev/doors.md`, `traps.md`, `searching.md`, `digging.md`, and `mapgen.md` each get one sentence pointing at them |

## 11. Choices made along the way

| Choice | Why |
|---|---|
| A chest is a fixture, not an object | §3.1 (decision 1) |
| The level is the floor's, the band only names it | the owner's rule (decision 2); the item level exists on every level with no danger (decision 13), so nothing new decides it |
| The band is not saved | `band_of(level)` recovers it, so the bands can be renamed or re-cut without a format bump |
| Mimics as a generated family, one type a level | the only way a mimic's level always equals the dungeon level with ten names; the animated families built the mechanism, and its four reads already handle a family in kills, quests, the danger draw, and the wiki |
| Only a skeleton key fits a generated chest | as a tempting hoard's doors; a branch key that opened every chest would make the branch's two locked rooms the chests' key too, and keys are for rooms. Pick lock, Knock, a kick, or a blast are the ways |
| Coins and gems spared under a boot, nothing under a fireball | the owner's rule (decision 4): a kick is a boot on wood; a burst is fire inside the box |
| A blast from outside blows the chest up with nothing spared | my reading of decision 4: the owner spoke of a trap; an outside fireball is the same fire inside the same box, and one rule for bursts is simpler than two |
| A burst blows the chest up, nothing else does | the owner: "traps can also break the chest if they go off"; a burst is the trap that goes off *inside*; an arrow or a bolt does not touch the box |
| A mimic takes no turn while disguised and never reveals itself | the player decides when; the same rule as sleepers; a mimic that walked about would be a monster, not a chest |
| A touch reveals and gives the mimic a free blow, the action refused | the surprise lands once and costs the player a hit, not a turn and a hit; the next move is theirs |
| The AI ignores disguised actors | with the vermin faction some tribes hunt the vermin until H1 lands; a kobold attacking a "chest" would both leak and look absurd |
| Greater treasure rooms have no corridor, and the breach is found by searching | the owner (decision 6); without a tell on the map, searching is the one fair way to find a wall worth digging, and the search machinery already finds hidden things in walls |
| Treasure rooms never need a danger | the Depths may have them; the chests' and mimics' level is the item level, which every level has (decision 13) |
| The hint is said on every arrival | the owner (decision 7) |
| One chest a floor in five | the owner (decision 12): the first draft's one in two was too many |
| Knock, and nothing sanctified | the owner (decision 15) |

## 12. Implementation plan

Each item is one issue, one branch, one PR, updating the spec, `docs/dev/chests.md` (created in C1), `docs/dev/creatures/mimics.md` (C4), `docs/dev/special-rooms.md` (C5, C6), and the wiki in the same PR. "Save" means it adds saved state; none adds an action (Knock is a class ability through `use:`). Sizes: S about a day, M two to three, L a week.

| # | Issue title | Sections | Adds | Depends on | Size |
|---|---|---|---|---|---|
| **H1** | Factions: inter-enemy hostility made rare (the two story feuds kept, every other monster pair neutral; the arena's rats given their own faction; the stance tests, the wiki sentence) | §7 | — | — | S |
| **C1** | Chests: `data/chests.toml` (bands; the loot fields parsed, unused), the three terrains, `LevelState::chests`, legend placement (`chest`, `locked`, `lock`, `objects`), open with a key and without, Pick lock on chests, the bump, the kick (smash, the 50 % rule, coins and gems spared), blasts (blown up, nothing spared), perception's `chest_at`, look mode, the `truth`/`view`/JSON dumps, `docs/dev/chests.md`; the `chests` system with Deft breaker, Chest-wise (locks), Lockbreaker's reach, and Knock | §3.1–3.4, §3.6–3.9, §9 | save | — | L |
| **C2** | Trapped chests: `fits_fixture`, traps on chest cells (legend `trap`), springing on open and on smash, a burst blows it up, Disarm on chests, searching finds them, Chest-wise's trap half, Trap sense's cut | §3.5, §9 | — | C1 | M |
| **C3** | Chest loot and generated chests: `chest::roll_contents` (items, gold, gems), `chests = { chance, count }` per branch, `chest::fill` on the chest stream (`Rng::chests`), the wiki's Chests page (`xtask/src/wiki/chests.rs`) | §4, §3.3, §8 | — | C1 | M |
| **C4** | Mimics: the generated family (`data/mimics.toml`, `mimic_<L>`), the `mimic` tag, the `disguised` status (`hides`), the AI's blind eye, the disguise in perception, revealing (touch with the free blow, Knock, a blow, searching and Spot, Mimic-wise), the swap in `chest::fill`, the legend's `mimic = true`, the dumps, `docs/dev/creatures/mimics.md`, `mimic_roster_tests.rs`, the wiki's mimics table; the `mimics` system with Mimic-wise and Slick | §5, §9 | — (the status is saved as any) | C2, C3 | L |
| **C5** | Treasure rooms: `secret_doors = "all"`, `treasure`, `arrive`, `treasury::fill`, the rooms generator's sealing of doors, `special::stamp` and the caves' box with a secret door, the hint, the hoard renamed "tempting hoard" (mark `T`, Hoard-wise's text); Treasure sense | §6.1, §6.3 (secret), §6.4–6.6, §9 | — | C3 | M |
| **C6** | Greater treasure rooms: `sealed = "dig"`, the box on both generators with its `breach` (saved), eligibility by spot, `connect::validate`'s breach rule, `hide_traps` and the key and quest placements leaving sealed rooms alone, `Hidden::Hollow` and `LevelState::hollows` (saved), the hollow wall in memory and look mode, `[[room]]` with `breach`, the hint | §6.2, §6.3 (dig) | save | C5 | M–L |
| **C7** | Content: chests in the Ascmearc's vaults (the drake's hoard, the robbers' hoard, the captain's hall, the still room), `chests` per branch, a `chests` showcase scenario (a coffer, a locked chest, a trapped one, a mimic, a treasure room behind a secret door, a greater one with a pick-axe beside the wall) for captures and tests | §3.3, §6 | — | C4, C6 | S |

**Waves** (at most about three PRs at once that add actions or save state; here never more than one):

1. **H1** and **C1** together (H1 is data and tests; C1 adds save state and changes the Depths' snapshots where a chest lands; no file in common).
2. **C2, C3** together (no save state; C2 touches `trap.rs`, C3 `mapgen/mod.rs` and `rng.rs`: no overlap).
3. **C4, C5** together (C4 touches perception, `creature_turn`, the AI's target line, and `content/animated.rs`'s family code; C5 the generators and `room_effects.toml`: no overlap but `chest::fill`, where C4 adds the swap after C3's cells; C5 calls it).
4. **C6** (save), then **C7**.

**Tests each item carries**: `content/chest.rs` (the bands parse and validate: rising `from`, ranges, the dice, `band_of`); `content/mimic_roster_tests.rs` (above); `world/chest_tests.rs` (a coffer opens on a bump and on `open`, a locked one refuses without a key and opens with a skeleton key, Pick lock, or Knock then Open, Knock refused with no time at an unlocked chest and at nothing, the kick's odds over many draws, a smash destroys about half the stacks and never coins or gems, Deft breaker's quarter, a blast blows a coffer up with coins destroyed too and not a kist, the contents are not in the snapshot while shut and are the pile once open, look mode's words with and without Chest-wise, a legend's chest with objects, saving and a replay, the dumps); `world/chest_trap_tests.rs` (a trap springs on open and on smash at the right victim, aimed from the chest, never on a failed kick; a burst blows it up with nothing spared; Disarm made and failed; searching finds it with the chest's sign; a pit refused; a branch without traps traps nothing but draws the same); `mapgen/chest_tests.rs` (one floor in five, cells plain and joined, the level the same without chests, the loot's means over many floors within the table's, a fixture branch at depth 8 placing banded chests); `world/mimic_tests.rs` (the disguised snapshot equals a chest's, a touch reveals with a free blow and no time, Knock reveals without one, a blow reveals, a missile stops at it, a search and the passive Spot reveal it, Mimic-wise sees it, Slick, it takes no turn and rests are not disturbed, no creature attacks it, it counts toward the cap, it carries the chest's loot and drops it, a sneak attack on an unaware one, its level is the floor's on a floor without a danger, saving); `mapgen/treasury_tests.rs` (every door secret and the level valid, the box on a cave floor with its secret door, contents as the dice say, a greater one's breach `wall` beside a reachable cell with no corridor to the box, the room unreachable without it, `validate` refusing a sealed room with no breach, no trap or key or quest target inside, not eligible when no spot fits); `world/treasury_tests.rs` (the hint on each arrival until entered, with Treasure sense's bearing, the enter line, the breach found by searching and remembered, digging it opens the room, a scroll of fireball does too, the `rooms` dump); `world/bestiary_tests.rs` and the roster tests' stance rows (H1); screen snapshots for look mode's chest lines and the abilities menu with Knock (text snapshots, `screen_snapshots.rs`); a replay per flow (`replay --create`, `--verify`); `cargo xtask wiki`. **No golden**: nothing here is produced by the GPU alone; the glyphs are checked by a `--capture` looked at in C1 and noted in the PR.

## 13. Decisions (owner, 2026-10-28)

The owner answered the first draft's open questions q1 to q15, which were numbered as its §12's Q1 to Q15; the numbering lines up one to one. Each answer is recorded with how it was read where reading was needed.

| # | Question | Decision | Sections changed |
|---|---|---|---|
| 1 | A chest as a cell fixture or a carried object? | Fixture. | §3.1 |
| 2 | The chest tier names? | Tiered names are fine, but **the actual level always matches the dungeon level**; the potential mimic matches the naming band; ten name variants. Read as: a chest's level is the floor's item level, the name comes from ten bands by level, and a mimic is of the floor's level and named by the same bands. The six fixed tiers became ten bands; the mimics' six fixed types became a generated family with one type a level and ten band names. | §1, §3.1, §3.2, §4, §5.1, §5.3, §6.2 |
| 3 | The mimic names? | Ten names: mimicling, young mimic, mimic, old mimic, greater mimic, elder mimic, hoary mimic, ancient mimic, primeval mimic, eldest mimic. | §5.1 |
| 4 | The 50 % destruction: per item or per stack; coins and gems exempt? | Per stack; gold and gems spared on a kick; **nothing spared when the chest blows up because of a trap**. Read also onto a blast from outside (the same fire inside the same box). | §3.7, §3.8, §4, §9 (Deft breaker's reach) |
| 5 | A trap at the kicker when the kick fails? | No. | §3.5, §3.7 |
| 6 | A greater treasure room: wall up a door and keep the corridor, or no corridor at all? | **No corridor at all.** The room is a box stamped into the rock on both generators, no door, no corridor; its breach is a hidden feature found by searching ("the wall sounds hollow"). | §1, §6.2, §6.3, §11, §12 (C6) |
| 7 | The hint on every arrival or once? | Every arrival until the room is entered. | §6.5 |
| 8 | Chest-only trap kinds? | No new kinds; the same list as the doors'. | §3.5 |
| 9 | The mimics' faction? | **Vermin**, not a faction hostile to everyone: a dungeon where everyone fights everyone is a battle royale in which the player picks up stragglers. And **inter-enemy hostility is to be made much rarer** in general: its own slice, H1 (§7), independent of chests. | §1, §5.2, §5.4, §7, §12 (H1) |
| 10 | Mimics as anything but chests? | No: chests only. | §5 |
| 11 | A mimic's natural Protection: hide? | Sounds good. | §5.2 |
| 12 | The chests' frequency on floors (`chests_per_level = [0, 1]`, about one floor in two) | "What sounds a little much? Is our config that inflexible that we cannot express smaller chances? Perhaps one in five." Read as the frequency of chests outside treasure rooms: it becomes `chests = { chance = 20, count = [1, 1] }`, **one floor in five**. The data is percent integers everywhere (any chance from 1 % up is one line; a per-mille field would be one more if ever wanted); the one chance changed is this one. | §3.2, §3.3, §11, §12 (C3) |
| 13 | The treasure room's `needs_danger`? | Fine without a danger if the level can be found simply. It can: the floor's item level (a branch floor's depth, a hand-made level's `item_level`) is what loot already rolls at and needs no danger; chests, mimics, and treasure rooms all read it, and the mimic swap no longer asks for a danger either. | §2, §3.2, §5.3, §6 |
| 14 | A revealed mimic disguising itself again? | No. | §5.5 |
| 15 | A class ability: Knock, Sanctify? | **Knock works**; Sanctify would do nothing (it is just a treasure room). Knock is the wizard's class ability: a lock springs open. Nothing sanctified anywhere. | §3.4, §5.5, §9, §12 (C1) |
