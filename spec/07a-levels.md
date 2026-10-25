# Warlock specification — 7. Levels and data formats: level files

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

## 7. Levels and data formats

Rule: **hand-authored data is TOML; machine-written data is JSON.**

- **Level files** (TOML): a header (name, ambient light, legend, lights, patrols, stair links,
  and how time passes: `kind = "dungeon"` (default), `"outdoor"` (under the open sky, lit by the
  sun and the sky; §5 "Sunlight"), or `"overworld"` (also under the sky, where a step's base time
  is the terrain's travel time; §8 "Time"; a level has no turn length of its own); optional
  named regions, `[[region]] name = "Thornwood" area = [x0, y0, x1, y1]`, inclusive, first match
  wins) plus a multiline ASCII map. Level size comes from the map, with no fixed upper limit: most dungeon levels
  are about 80×21, some are vast. The camera scrolls when a level is larger than the viewport. The
  same format defines vaults for procedural generation (below). Illustrative only — exact syntax
  is designed in M1:

  ```toml
  name = "Hub"
  ambient = [8, 8, 12, 0]  # r, g, b, uv
  map = '''
  ##########
  #@..T...+.
  #..~~~..#>
  ##########
  '''

  [legend]
  "#" = "wall"
  "." = "floor"
  "+" = "door_closed"
  "~" = "water"
  "T" = { terrain = "floor", light = "torch" }
  ">" = { terrain = "stairs_down", to = "vast" }
  "@" = { terrain = "floor", spawn = "player" }

  [[patrol]]
  creature = "rat"
  path = [[12, 3], [20, 3], [20, 8]]
  ```
- **Objects in level files** (M11): a legend entry places a pile with `objects = ["dagger",
  "arrow*12"]` (top first; `*N` places N, as one stack if the object is stackable, else as N
  stacks of one; a loose gem names its gem, `"gem:ruby*3"`, of the plain size and cut, §7
  "Gems"; anything else may name another material its construction list allows,
  `"great_sword:bronze"`, §7 "Materials"), and a creature's entry gives what it carries with `carry = ["spear",
  "copper_coin*7"]` (any number of stacks), and what of that it wields and wears with `wield =
  "spear"` and `wear = ["leather_jerkin"]` (an object id or a list; M16): each the first carried
  stack of that object not yet equipped, in the first free slot that fits it (a second ring on
  the other hand). `wield` takes what goes in a slot of action `wield` or `ready` (weapons,
  shields, missile weapons and missiles), `wear` the rest; an object it does not carry, one with
  no free slot, or the wrong one of the two is a content error. `[[pile]] at = [x, y] objects =
  [...]` places a pile without a legend character (on top of the legend's there, if any).
  Objects lie only on walkable terrain. Vaults place objects the same way, and generators stamp
  them with the vault.
  Magic items (M29, #149; §8 "Magic items") are placed by `[[item]]` tables, on top of the pile
  at `at = [x, y]` after the `[[pile]]`s: `object` (a base item), `affixes` (affix ids, each with
  an optional tier, `"sharp:2"`, default 1; its value the top of the tier's range), `name` (a
  rare item's: with a name it is rare, without magic), `identified` (default false), `level`
  (the item level, default that of its highest tier), and `count` (stackable ones, default 1).
  The item must be one its affixes may roll on (§8 "Magic items"), else a content error. An
  `[[item]]` may give `parts` (§7 "Gems"), with affixes or without: gems, each `{ part, of, size,
  cut }` (`part` a part id, none for a loose gem's gem, its main part; `of` a gem; `size` and
  `cut` default to the plain ones), in place of the base item's part of that name or besides its
  parts: `parts = [{ part = "pommel", of = "emerald", size = "small", cut = "fine" }]`. A
  level's `item_level` (1 to 99, default 1) is the item level of what creatures drop there (the
  arena's is 2); a branch level's is its depth. A vault's legend entry may give `loot = "any"`,
  `"magic"`, or `"rare"`: a cell where an item of loot at least that good is rolled when the
  vault is stamped into a generated level (only vaults: in a level file it is a content error);
  the guard post's store room holds a magic one, the shrine and the roofless chapel any.
- **Locks in level files** (M19, §8 "Doors"): a legend entry on a closed door locks it with
  `lock = "pantry"` (a lock id: keys with that id open it, and skeleton keys) or `locked = true`
  (a lock without an id: only a skeleton key opens it), and `lock_difficulty = N` with either
  sets how hard it is to pick (1 to 50; default 14, `door::LOCK_DIFFICULTY`; §8 "Doors"). A
  lock on anything but a closed door, an empty lock id or `*`, a `lock` with `locked = false`,
  or a `lock_difficulty` without a lock or out of range is a content error. Hand-made levels
  need not give a lock's key (a door may be meant to be kicked in, or picked); vaults must
  (below).
- **Secret doors in level files** (#140, §8 "Searching"): a legend entry on a closed door makes
  it secret with `secret = true` (difficulty `data/search.toml`'s `secret_door`) or `secret = N`
  (difficulty N, 1 to 50). The cell then shows the wall around it (of its solid neighbors north,
  east, south, and west that are no door, the most common) until the player finds it. A secret
  on anything but an unlocked closed door, or one with no wall beside it, is a content error.
  Indoor vaults place secret doors the same way and generators stamp them with the vault (a key
  must not lie behind one); outdoor vaults have none.
- **Hidden locations** (#327, the campaign design §8): a legend entry on the overworld may hide a
  location on its cell, `"2" = { terrain = "swamp", hidden = { id = "fennmynster", terrain =
  "dungeon_entrance", to = "fennmynster:1", name = "Fennmynster" } }`. Until a quest reveals it
  (§8 "Quests", `reveals`) the cell is the plain terrain it shows in every way: a swamp cell, its
  travel time, its local area, no stairs link; the view, memory, look mode, and every
  perception-built dump show the swamp, as a secret door is the wall it looks like. Revealed
  (`World::reveal_location`, `world/reveal.rs`), the cell becomes `terrain` (stairs down) leading
  `to`, the location goes by `name` (look mode, "The way into Fennmynster is here.", "You enter
  Fennmynster."), and it is **marked on the player's map** (their memory of the overworld holds
  the entrance) wherever they are; revealed before the overworld is made, it is applied as it is
  made. The id (lower-case letters, digits, and underscores) is unique on the overworld; a hidden
  location on any other level, on a cell that is not walkable or is a location already, with an
  unknown `terrain` or one that is no way down, an unknown `to`, or no `name` is a content error.
  Without `to` it leads nowhere yet (its level is not built), like stairs without one.
- **Named places** (#332): a legend entry may name the place on its cell, `name = "Beorggeat"`,
  and give look mode a sentence about it, `note = "The way you came; snow has closed it behind
  you."` (only with a `name`). Look mode names it on a cell the player sees or remembers ("A
  mountain pass, Beorggeat." and the note after); a way in on that cell goes by the name (look
  mode's "leading to Cēaptūn", "The way into Cēaptūn is here.", "You enter …"), even while it
  leads nowhere yet. A `name` beside a `hidden` (it would give the place away), an empty one,
  or a `note` without a `name` is a content error.
  State: `LevelState::hidden` (ground truth: the unrevealed ones, by cell) and
  `WorldState::revealed` (the ids revealed so far), both saved and hashed and left out while
  empty; a save naming a location the content does not have is refused. Only the reveal and the
  dev views read `hidden`: the `truth` layer and F2 draw a hidden entrance by its glyph. Quests
  are the only key: there is no search for one on the overworld. The dev action `reveal:ID`
  reveals one at once (no time).
- **Traps in level files** (M18; §8 "Traps"): a legend entry places a trap with `trap = "id"` (an
  id in `data/traps.toml`), hidden unless `known = true`, and for a bolt trap optionally the way
  its bolt runs, `dir = "E"` (default: along the longest clear line from it). Traps lie only on
  walkable terrain that is not stairs nor water that freezes, or on a door, closed or open (§8
  "Trapped doors"), where any trap but a pit may lie, and never on a secret door; `known` or `dir`
  without a `trap`, `dir` on any other trap, a pit on a door, a trapped secret door, or an unknown
  id is a content error. Dungeon vaults place traps the same way and
  generators stamp them with the vault; outdoor vaults have none.
- **Traps** (`data/traps.toml`, optional; without it there are no traps; M18): `[[trap]]`s with
  `id`, `name` ("fireball trap", said in "Really step onto the fireball trap?"), `glyph` (default
  `^`) and `fg` (how it shows once known), an optional `message` said when the player perceives
  it go off ("A fireball explodes!"), `difficulty` (how hard it is to find, 1 to 50, default 15;
  §8 "Searching") and `sign` (what gives it away when found, default "a pressure plate": "You
  notice a tripwire: an alarm trap."), optional `charges` (1 to 1000; how often it goes off before
  it is spent and gone; none: every time), an optional `noise` (`{ sound, loudness }`: a sound
  kind of §5 "Hearing" and its loudness on the trap's cell, 1 to 300), and its `kind`, with the
  fields each kind needs (any other is a content error): `shot` (`object`, the missile, which
  lands as that object; `damage`, `to_hit`), `strike` (`noun`, `damage`, `to_hit`), `burst`
  (`radius` 1 to 5, `damage`, `spell`, `noun`, an optional `blast` 0 to 10000: the walls it
  reaches whose hardness is below it break, §8 "Digging"; and either `heat = true`, it melts the
  ice it reaches, or `frost` 1 to 1000, the water it reaches freezes to at least that many mm of
  ice, §5 "Ice"), `bolt` (`length` 1 to 40, `damage`,
  `spell`, `noun`), `pit` (`damage`), `alarm` (needs a `noise`), `teleport`. `noun` and `verb`
  (default "hits") make the messages ("The fireball burns you."); `spell` names the effect
  presentation shows (`data/effects.toml`; §4.8). `to_hit` is within ±20 and never shown;
  `damage` is dice that can do harm. The built-in traps: arrow (`shot`, an arrow, 1d6, a
  bowstring 40), spear (`strike`, 1d8), fireball (`burst` radius 2, 3d6, a blast of 1000, heat,
  once, an explosion 150), frost (`burst` radius 2, 2d6, frost 100 mm, once, silent), lightning
  bolt
  (`bolt` 12 cells, 4d6, twice, a thunderclap 120), pit (1d6), alarm (a bell 300), and teleport.
  The wiki shows them in a generated table.
- **Water**: `~` is still water (`water`); `shallow_water` (ankle-deep over a stone floor, paler,
  a flooded room's floor, §7 "Special rooms") steps at 150 and splashes like water. Running
  water (`running_water`, terrain with a `flow`
  animation) needs a flow on every cell, given by its legend entry: `flow` (the direction it runs,
  `N` … `NW`) and an optional `flow_speed` (percent of the terrain's animation speed, 10–300,
  default 100). The shared legend (`data/legend.toml`) draws running water as arrows pointing the
  way it flows: `↑ ↗ → ↘ ↓ ↙ ← ↖` at normal speed and `⇑ ⇗ ⇒ ⇘ ⇓ ⇙ ⇐ ⇖` at double speed, so a
  stream reads in the map file as it flows, e.g. `~~~←←←~~~`. A `flow` on any other terrain, or
  running water without one, is a content error. Levels carry their flow into the game (saved with
  the level; generated levels get theirs from vaults); like animation it is presentation data that
  no rule reads. The vast level's generator derives the river's flow from its channel: each row
  runs toward where the channel continues in the next row, edge cells turn toward water below them
  where the channel narrows, and a few boulders (from their own random stream, so the rest of the
  level is unchanged) stand in the wide stretches for the water to foam against.
- **Temperature and ice in level files** (M26; §5 "Temperature", "Ice"): a level under no sky
  may set its `temperature` (°C, −40 to 40; default its branch's, else 10); under the open sky
  it follows the weather, so an outdoor or overworld level giving one is a content error. A
  legend entry on water that freezes may lay ice on it, `{ terrain = "water", ice = 25 }` (mm, 1
  to 1000): the cell shows the ice terrain that thickness reaches. Ice terrain is never placed by
  itself (a content error): it lies on water, which it thaws back into. Vaults lay no ice and
  have no temperature (the level they are stamped into decides).
