# Warlock specification — 7. Levels and data formats (continued): generation

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

- **Dungeon branches** (`data/dungeons.toml`): stacks of generated levels, e.g.
  `{ id = "depths", generator = "rooms", depth = 8, size = [80, 21] }`, plus optional name,
  an `arrival` line (said each time the player arrives on its first level by stairs, as a
  hand-made level's is),
  patrolling creature kinds, the traps its levels hide (`traps`, trap ids; one named twice comes
  up twice as often) and how many a level gets (`traps_per_level = [least, most]`, default
  `[2, 4]`, at most 50), how many doors its rooms levels hide (`secret_doors = [least, most]`,
  at most 20, default none; the Depths `[1, 2]`; §8 "Searching") and how much harder its hidden
  features are for each level below the first (`hidden_per_depth`, 0 to 5, default 0; the Depths
  1), how much harder its locks are to pick for each level below the first, on top of each lock's
  own difficulty (`locks_per_depth`, 0 to 5, default 0; the Depths 1; §8 "Doors"), ambient light, `loot = [fewest, most]` (items of loot on each
  level's floor, at most 50; the Depths 2 to 4; §8 "Magic items"), `temperature` (°C, −40 to 40;
  default 10; §5 "Temperature"), a `key` (a key object: its rooms levels lock a few rooms with
  the lock it opens, §8 "Doors"; the Depths' is the rusty key), its own special-room and shop
  odds (`room_effects`, `shops`; §7 "Special rooms", "Shops"; default the shared ones), and
  per-depth overrides of generator, size, temperature, `room_effects`, and `shops` (each default
  the branch's), and the vaults a range of depths stamps for certain (`vaults`, see "Vaults"). Stairs
  (and scenarios) may target `depths:N`; content validation accepts such ids when the branch has
  that depth. A branch level is generated on its first visit from its own random stream (§8) and
  then persists like a hand-made level. Its `<` leads to `depths:N-1` (on depth 1, back to the
  level it was first entered from), its `>` to `depths:N+1` (the deepest level has none).
  The Ascmearc's first branch is **Cealdhlāw** (`cealdhlaw`, #337; the campaign design §7), the
  cold barrow under Hlāwdūn, its way in on the overworld from the start: six floors, the barrow's
  passages (rooms, 80×21) on 1–2 and the hill's hollows (caves, 100×40) on 3–6, danger 1 to 6
  (`danger = 1`), its population the six old hostiles and the barrow wave (§7 "Data"),
  `population_count = { per_1000 = 30 }` (25 on a rooms floor, about 70 in the caves), the
  arrow, spear, pit, and alarm traps, secret doors, locks, and loot as the Depths', the rusty
  key, 8 °C, and the shared special-room and shop odds. Its vault is the barrow chamber, named
  by its second floor (`[[dungeon.levels]] vaults`), so it is there for certain and nowhere else,
  with the silver chalice of Sister Hild's quest on its plinth (§8 "Quests"). Its `arrival`:
  "Cold air breathes up from the mound." Its dev scenario `cealdhlaw` starts on the first floor.
- **Danger levels** (#326; `content/danger.rs`, `world/spawn.rs`; the design is
  `docs/design/campaign.md` §9): every level has a danger, computed when asked
  (`World::danger`), never stored. A branch's `danger = 4` (or `{ base, per_depth, player }`;
  `base` 0 to 99, `per_depth` 0 to 5, default 1) gives its floor at depth *d*
  `base + (d − 1) × per_depth`, and a `[[dungeon.levels]]` range may give its own `danger`; a
  hand-made level's `danger = N` (0 to 99, or `{ base, player }`) is its own; the overworld,
  local areas, and every level without one have 0, and nothing comes there (every showcase
  level and the Depths: unchanged). `player = { percent, from, max }` scales an area with the
  player (ADOM's puppy cave): `+ max(0, level − from) × percent / 100`, at most `max`; no area
  uses it yet. With a danger come `population` (creature ids; a branch's default its
  `creatures`), `population_count` (a number, or `{ per_1000, least, most }`, default 35 per
  1000 walkable cells, 25 to 80; at most 200), the draw's odds `stray`, `out_of_depth`,
  `deeper` (percents) and `deeper_cap` (0 to 50), each defaulting to `data/dungeons.toml`'s
  `[defaults]` (5, 5, 5, 10; required once any area has a danger), and
  `spawn = { every, chance, cap }` (default `15m`, 60 %, cap the population). A creature type's
  `level` is its danger; a draw at danger *D* weighs a type of level *L* 4 at `D` and `D − 1`, 2
  at `D − 2` and `D − 3`, 1 down to `D − 8`, else 0 (above *D* too), uniform among equal weights
  over the types sorted by id. **Strays and out-of-depth draws** (#375; both spawns and initial
  populations): with `stray` % a draw is made from the area's **generic creature list** instead
  of its population: every type that lives in some area with a danger (a branch's or a
  hand-made level's `population`) of the area's campaign or of none, uniques left out
  (`Spawning::generic`, derived at load, `Content::fill_generic`; hand-placed people,
  shopkeepers, guardians, quest creatures, and test types live in no campaign population, so
  none strays; the list may hold the area's own types); `stray = 0` keeps an area to its own.
  With `out_of_depth` % the draw is made at `D + 1`, then with `deeper` % again and again one
  deeper still, at most `deeper_cap` times (an exploding tail: at the defaults `D + 1` 4.75 % of
  all draws, `D + 2` 0.24 %, `D + 3` 0.012 %); where the drawn pool weighs nothing that deep (its
  types all more than 8 below) the generic list is drawn there instead, and where nothing
  weighs anything there the draw is made at `D` (a stray finding nothing at `D` takes the
  population). The rolls come in a fixed order from the same streams: stray, out of depth, each
  step deeper, the pick.
  **Rarity** (#373): a creature type's `rarity` is a tier of `data/creature_rarities.toml`
  (`common` 100, `uncommon` 50, `rare` 25, `very_rare` 10, in percent; default `common`), and its
  weight in every draw, strays and out-of-depth ones too, is the level weight above
  times that factor: the more dangerous and rewarding a type (casters, chieftains, the
  strongest of a place), the rarer. Hand-placed creatures are never drawn and so unaffected.
  Hidden in the game; the wiki's dungeons page gives the tiers and every drawn type's, and the
  draws' odds exactly. A type with `unique = true` is drawn once a run (`Progress::uniques`).
  **Spawning**: a level with a danger keeps a spawner in its state (`LevelState::spawner`: the next
  attempt's time and the stream `spawn/<level id>`; none elsewhere, so other levels' states are as they
  were). At the end of every turn (`run_turn` → `spawn_tick`) each attempt due on the current
  level is made: skipped while the level holds as many creatures hostile to the player as its
  cap, else with its chance a type is drawn and placed on a cell out of the player's view by the
  perception rules and at least 8 cells (Chebyshev) from them, walkable, free, and no stairs,
  door, shop floor, or sanctuary (up to 32 tries, else nothing comes), with a short out-and-back
  route where one fits, else a post. Floors the player is not on stay frozen: on arrival the next
  attempt is put off to `every` from then. A rest perceives after any turn in which one came. The
  dev action `spawn` makes one attempt now, whatever the cap and chance. Talents (system
  `danger`): Quiet delver (`spawn_every_percent` 100: attempts half as often), Sixth sense
  (`danger_sense`, 2 ranks: on arrival "This place feels quiet / uneasy / dangerous / deadly."
  at danger up to level − 4 / level / level + 3 / beyond; at 2, the note "Something stirs in the
  dark." when one comes). Hidden in the game; the wiki's dungeons page states it all.
  **The initial population**: a generated floor of a branch with a danger is laid down full:
  its `population_count` for the floor's walkable cells (by default 35 per 1000, 25 to 80; on
  top of the generator's own patrols), each creature drawn by the weights above, a unique type
  not if the run has met it, then placed on plain floor free of creatures, lights, traps, and
  shops, outside a sanctuary or a teeming den, and at least 8 cells (Chebyshev) from every
  staircase, with a short out-and-back route over cells as free (no door, no trap) where one
  fits, else a post. The draws come from the level's own stream `population/<level id>` after
  its loot and before its teeming dens' sleepers (`swarm/<level id>`, §7 "Special rooms") and its
  creatures' gear (which they roll like every creature without a fixed kit), so its map, traps,
  secrets, and loot are the same with any population. A hand-made level with a danger starts as
  its file lays it out; its `population_count` is only the cap its spawns refill toward. A unique
  type placed on any level is met. A generated 120×60 cave floor at full population is a
  `bench-turns` entry (16 ms a turn).
- **The overworld**: at most one level of kind `overworld`. Its stairs down are locations; every
  other cell whose terrain has a `local` ground names a local area, level id `local:x,y` (canonical
  decimal, no signs or leading zeros), which stairs and scenarios may also target. A local area is
  generated on its first visit from the content, the seed, and its id (its own random stream,
  `map/local:x,y`), then persists; it records its edge exit (the overworld and the cell), and
  walking off its edge leads there. `>` arrives on the edge cell of the side the player walked
  onto the cell from (the run remembers the direction of the last overworld step until the next
  level change): on the road if one leaves the area there, else on dry ground if any, near the
  middle of the side (or the corner) but trading a few cells along it for open ground around
  it; with no such step, at the area's start in the middle.
- **Leaving an outdoor area** (#474): a hand-made level of kind `outdoor` that an overworld
  location leads into (its stairs down, or a hidden location's `to`; the first in the
  overworld's order) records that location as its edge exit, like a local area: walking off its
  edge returns to the location's cell (Cēaptūn has no stairs and is left only so, or as below).
  An outdoor level no location leads into (the showcase's courtyard) has no edge exit and keeps
  its stairs. **The quick leave**: `<` (`ascend`) on an outdoor level with an edge exit, when the
  player is not on stairs up, leaves it exactly as walking off the edge does (the same arrival,
  message, and the time of taking stairs) if and only if no creature hostile to the player
  (by faction stance, awake or asleep) is on the level where the player sees it, at any distance,
  or unseen within 10 cells of the player (Chebyshev distance: diagonal steps count one). The
  check reads the true positions (a rule, like a creature noticing the player). Otherwise it is
  refused with one line that names no creature and does not say whether it was seen ("You
  cannot slip away with enemies about."), taking no time. Stairs up under the player are taken
  as ever; on dungeon levels and on the overworld `<` needs stairs.
- **Generators** (`warlock-core`, integer-only): *rooms* — walled rooms joined along a minimum
  spanning tree plus a few loops by corridors dug as cheapest paths (bends cost extra), with closed
  doors, open doors, or doorways where corridors enter, and wall torches in most rooms; with the
  branch's `key`, last, up to two rooms with a single door (not the arrival room, no room with
  stairs, no vault) get that door locked with the key's lock (a doorway or open door there
  becomes a closed door), and the key lies on a free spot in a room that is not locked (what
  lies or walks in a locked room stays there); *caves* —
  cellular automata (45 % rock, five 4-5 smoothing passes, largest cave kept), pools of still water,
  lamps and glowing fungus. Both place the up staircase (the arrival point) and the down staircase
  far from it, a few creatures on looping patrols, and last, now and then, a special room
  (below). Every level is validated by flood fill —
  every stair, door, and walkable cell reachable from the up staircase; locks only on closed
  doors, and the stairs and a key to every lock (lying, or carried by a creature) reachable from
  it without unlocking a door, so no level needs a key it does not give; patrols valid, some
  light — and a level that fails is regenerated from the next derived stream. A valid level then
  hides its branch's traps (§8 "Traps"), all hidden, drawn from a stream of their own
  (`traps/<level id>`, so the level is the same with or without them): a count within
  `traps_per_level`, each of a kind from `traps`, on plain floor nothing else claims (no object,
  light, or trap), never on or next to a staircase (the player arrives there), and never on a
  creature's patrol (creatures know nothing of traps and would spring one every round); then it
  hides doors (secret doors, §8 "Searching"), also from a stream of their own (`secrets/<level
  id>`): a count within `secret_doors`, each a closed door nothing else claims (no lock, no
  patrol through it, no shop it leads into, no vault it came with), made the wall around it and
  kept only if the level still validates with secret doors counted as walls for the stairs and
  the keys (every cell is still reached with them found), so a secret door only ever hides a way
  the player can do without; its loot is rolled after that (§8 "Magic items"). *outdoor* (the local
  areas of the overworld, §3): smooth integer value noise (random values on a coarse lattice,
  eased between) sized as a share of the area lays everything in natural blobs — woods, clearings,
  meadows of flowers, pools, crags; the share of woods blends towards each neighboring cell's
  within the third of the area nearest it. Each attempt lays water along river and lake sides
  (bands whose banks wander with the noise), a brook (a winding course, two cells wide, one flow
  direction per straight stretch), and the road first; keeps a glade clear around the middle;
  adds a pond and ruins (outdoor vaults) where they fit on open land; grows the landscape; then
  joins every walkable pocket to the start (small ones are grown over with what surrounds them,
  larger ones get a way cleared through the cheapest obstacles) and opens every side. A few deer
  graze on looping patrols in meadows and woods. Validation: the start is walkable, every walkable
  cell is reachable from it (4-connected), every side has at least 3 walkable cells, running water
  and only running water has a flow, and patrols are valid.
- **Vaults** (`assets/vaults/*.toml`): hand-made pieces in the level-file format, stamped verbatim
  (terrain, lights, objects, traps, locks, creatures) where they fit — into rock with room for corridors or tunnels to
  every entrance. A vault's edge is solid except for its entrances (doors, not on a corner); it has
  no stairs, player start, or unique character (`npc`), and all its walkable cells are reachable
  from an entrance. Its locks
  come with their keys: a key to every lock lies, or is carried by a creature, within reach of an
  unlocked entrance without unlocking a door (the guard post: its store room locked, the brass
  key carried by its kobold). Vaults of
  `kind = "outdoor"` are ruins for local areas, stamped on open land with a ring of open ground
  around them: their spaces (`void`) keep the area's own ground, every open cell on their edge is
  a way in, and their walkable cells are reachable from the edge; they have no traps. Dungeon
  generators use only dungeon vaults, local areas only outdoor ones. A shared vault
  (`assets/vaults/`) stamps in every branch; a campaign's (`campaigns/<id>/vaults/`) only in that
  campaign's branches; and a vault whose file gives `dungeons` (branch ids, or branch levels such
  as `"cealdhlaw:2"`, of its own campaign) only there. A `[[dungeon.levels]]` range
  may name vaults (`vaults = ["drakes_hoard"]`, #339): each is stamped on every level of the range
  for certain (a generation attempt that cannot fit one is made again), before any vault drawn at
  random, and a vault a branch level names is never drawn at random anywhere (a quest's target
  is where the branch says, once). A named vault is a dungeon vault, shared or of the branch's
  own campaign, and named once a level.
- **Fennmynster** (#338; `campaigns/ascmearc/data/dungeons/fennmynster.toml`; the design is
  `docs/design/campaign.md` §7): the cellars and crypts of the drowned minster in Micelfenn, its
  way in a hidden location of the overworld (70, 48) leading to `fennmynster:1`. Eight levels of
  rooms, 80×21, at 6 °C. Danger 4 to 11, nothing scaled by the player; its population is the fen
  wave (§7b), the bog hag among them (unique: once a run), at the default count (25 a floor). Pit,
  frost, alarm, and teleport traps; one to three secret doors a level, hidden things one harder
  each level down, locks one; 2 to 4 items of loot; the iron key. Its own room odds: a room is
  special with 25 % (at most two a level), flooded, hushed, or shadowed, so most levels have one
  and a third of them are flooded; no shops (`shops = { chance = 0 }`). Its vault
  (`campaigns/ascmearc/vaults/`): the **sunken crypt** on level 8 alone, a nave of shallow water
  between tombs, two drowned on guard, and at its far end a dry dais with an altar and the minster
  reliquary of Sister Hild's last quest on it. Its `arrival`: "The water is up to the third step."
  The `fennmynster` dev scenario starts on level 1.
- **Fyrdelf** (#339; `campaigns/ascmearc/data/dungeons/fyrdelf.toml`; the design is
  `docs/design/campaign.md` §7): the Ascmearc's deepest first dungeon, the duergar's mine under
  Ascdūn, its way in a hidden location of the overworld (97, 21) leading to `fyrdelf:1`. Twelve
  levels: galleries (rooms, 80×21) on 1 to 4, the deep workings (caves, 120×60) on 5 to 12;
  10 °C on the galleries, 16 °C on 5 to 8, 20 °C on 9 to 11, 24 °C at the bottom. Danger 8 to
  19, nothing scaled by the player; its population is the delf wave (§7b) with the bats,
  kobolds, and giant centipedes of the upper galleries, at the default count (25 on a gallery
  floor, 80 in the workings). Arrow, spear, fireball, lightning, and alarm traps; no to two secret
  doors a gallery level, hidden things one harder each level down, locks two; 3 to 5 items of
  loot; the brass key. Every gallery level with a fitting room has a general store
  (`shops = { chance = 100 }`: about one gallery floor in four) kept by a **duergar trader** (the
  campaign's own keeper type, with the shared keeper's kit). Its vaults (`campaigns/ascmearc/vaults/`):
  the **foreman's cabin** on each gallery level (a duergar miner at his post, a pick-axe, a lamp,
  a brass key) and the **drake's hoard** on level 12 alone (the fire drake on its gold, a rare
  and a magic find, ringed with lava). The drake is not in the population, so it is met there
  only (Factor Wulfrun's last quest). Its `arrival`: "The galleries smell of sulphur." The
  `fyrdelf` dev scenario starts on level 1.
