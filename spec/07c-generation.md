# Warlock specification — 7. Levels and data formats (continued): generation

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

- **Dungeon branches** (`data/dungeons.toml`): stacks of generated levels, e.g.
  `{ id = "depths", generator = "rooms", depth = 8, size = [80, 21] }`, plus optional name,
  patrolling creature kinds, the traps its levels hide (`traps`, trap ids; one named twice comes
  up twice as often) and how many a level gets (`traps_per_level = [least, most]`, default
  `[2, 4]`, at most 50), how many doors its rooms levels hide (`secret_doors = [least, most]`,
  at most 20, default none; the Depths `[1, 2]`; §8 "Searching") and how much harder its hidden
  features are for each level below the first (`hidden_per_depth`, 0 to 5, default 0; the Depths
  1), how much harder its locks are to pick for each level below the first, on top of each lock's
  own difficulty (`locks_per_depth`, 0 to 5, default 0; the Depths 1; §8 "Doors"), ambient light, `loot = [fewest, most]` (items of loot on each
  level's floor, at most 50; the Depths 2 to 4; §8 "Magic items"), `temperature` (°C, −40 to 40;
  default 10; §5 "Temperature"), a `key` (a key object: its rooms levels lock a few rooms with
  the lock it opens, §8 "Doors"; the Depths' is the rusty key), and per-depth overrides of
  generator, size, and temperature (each default the branch's). Stairs
  (and scenarios) may target `depths:N`; content validation accepts such ids when the branch has
  that depth. A branch level is generated on its first visit from its own random stream (§8) and
  then persists like a hand-made level. Its `<` leads to `depths:N-1` (on depth 1, back to the
  level it was first entered from), its `>` to `depths:N+1` (the deepest level has none).
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
  generators use only dungeon vaults, local areas only outdoor ones.
