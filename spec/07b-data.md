# Warlock specification — 7. Levels and data formats (continued): data definitions

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

- **Definitions** (TOML): terrain types, creature types, light presets, vision profiles, glyph
  animations, and effects (glow, heat haze, particle emitters; §4.8). A table that grows entry by
  entry (creature types, objects, materials, affixes) may be a directory read in path order,
  one file per entry (`data/creatures/kobold.toml`), after the single file of the same name if
  there is one; ids are numbered in that order, and an id in two files is a load error naming
  both. The small ordered registries (slots, stats, weapon skills, factions, actions, ...) stay
  single files. Creature types have a
  `name` and a `plural` (default the name plus `s`; the deer's is "deer"; quest messages count
  them), a
  `speed` (100 to 10 000, default 1000, the player's; never shown to the player; §8 "Speed"): the
  giant rat 1200, the cave bat 2000, the kobold 900, the deer 1300; and for hearing (§5)
  `footsteps` (at most 300; default 0, silent) and `hearing` (the faintest loudness heard, 1 to
  300; default none, deaf). For behavior (§8 "Creature behavior") a creature type names its
  `faction` (below; required when factions exist, the player's is `player`), its eyes `vision`
  (a profile; default `normal`) and `sight` (radius, 1 to 40, default 10), how it fights:
  `combat = "melee"` (default), `"ranged"`, or `"caster"`, the latter two with a `range` (2 to its
  sight), `opens_doors` (default false), `budge` (whether the player may swap places with it
  when it is not hostile, §8 "Displacement": `always`, the default; `off_post`, not while it keeps
  its post, going about its routine, as a shopkeeper at its counter will; `never`), and
  optionally `attack_cost`, what its attacks cost
  instead of the attack's cost in `data/actions.toml` (100 to 10 000; §8 "Time"; none of the
  built-in creatures has one yet). The kobold: darkvision, sight 10, melee, opens doors;
  the kobold archer: darkvision, sight 12, ranged 7; the kobold shaman (speed 800): darkvision,
  caster 5; the giant rat: low-light, sight 8; the cave bat: darkvision, sight 8; the cave
  beetle (1200): darkvision, sight 6; the deer: normal, sight 12. Its stats (§8 "Stats"; never
  shown): `stats`, its scores by stat id (`stats = { strength = 9, dexterity = 13 }`; ids
  `strength`, `dexterity`, `toughness`, `intelligence`, `willpower`, `presence`; 1 to 40, a
  missing one 10), and `potential`, the same for its potentials (1 to 40, not below the score;
  a missing one equals the score). The player creature has none: the scenario gives the
  player's. The test creatures have D&D-like scores (the kobold St 9 Dx 13 To 10 In 10 Wi 9
  Pr 8, the giant rat Dx 17 In 1, the cave bat St 1, the shaman Wi 14). For combat (§8
  "Combat"; never shown but as words): `base_hp`, its hit points beyond its Toughness (0 to 1000,
  default 0; the player creature has none: race and class give the player's); `attack`, its
  natural attack, used when it wields no weapon (`{ damage = "1d4", name = "bite", verb = "bites"
  }`, with optional `threat`, `crit`, and `to_hit` as for weapons, and `on_hit`, the status
  effects and diseases a hit with it may cause if the target lives: `[{ status = "poisoned",
  chance = 10, severity = 2 }]`, the chance in percent (1 to 100) and the severity in degrees (1
  to the status's `max`; §8 "Status effects"), or `{ disease = "filth_fever", chance = 10 }`
  (exposure to the disease, which the target then resists with a check; §8 "Diseases"; never the
  status diseases show as); without one, bare hands: 1d3, "fist", "hits"); and
  `natural`, its natural armour (`{ defence = 3, name = "shell" }`, 1 to 20, the Natural
  category of Defence; the name is said when it turns a blow). For experience (§8
  "Experience"): `level` (1 to 99, default 1) and `xp` (0 to 100000, default 0; the player
  creature has neither): the kobold 2/15, the kobold archer 3/25, the kobold shaman 4/40, the
  giant rat 1/5, the cave bat 1/3, the cave beetle 2/8, the deer 1/2. For loot (§8 "Magic
  items"): `loot`, the chance in percent (0 to 100, default 0) that it drops a magic or rare item
  where it dies: the kobold 25, the kobold archer 30, the kobold shaman 50. The giant rat and the
  cave bat bite (1d4, 1d3; the rat's bite poisons one hit in ten, 2 degrees, and exposes to
  filth fever one hit in ten), the deer kicks
  (1d4, "hoof"; `base_hp` 4), the cave beetle
  bites (1d3) and has a shell (3); kobolds fight with what they wield. Its weapon skills (§8
  "Weapon skills"): `weapon_skills`, its ranks by skill id (0 to 10; each raises the skills it
  defaults to; the player creature has none: race and class give the player's): the kobold
  spears 1, the kobold archer bows 1. For burden (§8
  "Burden"): `size`, a size in `data/sizes.toml` (default `medium`); the kobolds, the giant rat,
  the cave bat, and the cave beetle are small, the deer and the player creature medium (the
  player's size comes from their race when the run has a character). Skills (§8 "Skills"; never
  shown): `skills`, the skills it is trained in by skill id with fixed values (0 to 999); the
  rest untrained; the player creature lists none. The deer listens (30), the kobold shaman knows
  concentration (20) and spellcraft (25). The showcase's people (M28): the gardener, the
  wandering bard, and the old hermit (speed 800), glyph `@`, of the peaceful folk by type, level
  1 and worth no experience. For blood (§8 "Blood"; presentation only):
  `blood`, the colour of its stains (`"#rrggbb"`; default dark red, `#6e0e0c`) or `"none"` for a
  bloodless creature; the cave beetle bleeds greenish-black ichor (`#38461a`). For hunger (§8
  "Hunger"): `corpse` (default true: it leaves a corpse when it dies; with `data/hunger.toml`),
  and `on_eat`, what eating its fresh corpse does besides feeding, never shown before the player
  tries (`{ message = "...", vomit = 25, stat = { toughness = 5 }, heal = "1d4", harm = "1d2",
  satiation = 500 }`: the message said once it is eaten, a percent chance it comes back up, a
  percent chance per stat to raise it by 1 up to its potential, dice of hit points healed, dice
  of degrees of sickness given (the status `sickness` names, poisoned; §8 "Status
  effects"), nutrition beyond its size's; chances 0 to 100, dice rolling 0 to 1000; only on a
  creature that leaves a corpse). The built-in ones: the giant rat "Gamey and stringy, but it
  stays down."; the cave bat leathery and sickening (1d2); the kobold and the kobold archer
  "This kobold meat tastes terrible!" (vomit 40); the kobold shaman the same (vomit 40) and
  Willpower 3 %; the deer "Venison! It does you good." (satiation 1000, heal 1d6); the cave
  beetle "Crunchy." and Toughness 10 %. Creature particle emitters may
  fire `on = "attack"`: a burst at the target of each attack the player sees (the shaman's
  violet sparkle). Terrain has a `move_cost`: what a step onto it costs in percent of a normal
  step (the `step` action; default 100; still and running water 200, shallow water 150; local
  levels only), a
  `bounce` tint for the light it reflects (§5), a `height` in cells for sun shadows (default 1 if opaque, else 0; at most 4),
  `roofed` (never lit by the sun or the sky), `stone` (stone, worked or not: dwarves notice what
  is hidden on it in passing, §8 "Searching"; default false; floors, walls, granite, stairs,
  flagstones, rubble, rocky ground, boulders, and the like), `stains` (whether blood stains it,
  §8 "Blood"; default true; false for water, running water, lava, fountains, braziers, wall
  torches, solid rock, and the overworld's river and lake), and for hearing a `step_noise` (how loud steps onto
  it are, in percent of the walker's footsteps, 0 to 1000; default 100), a door's `toggle_noise`
  (how loud opening the closed door or closing the open one is, at most 300; default 0, silent;
  only doors), and a `muffle` (the loudness a sound loses entering the cell, at most 300; default
  0, but solid terrain blocks sound unless it has one). A closed door may break when kicked
  (§8 "Doors"): `breaks_to`, the terrain it becomes (walkable and no door: the broken door), and
  `sturdiness` (1 to 100: a kick breaks it with a chance of the kicker's Strength score in it;
  the closed door's is 30), both or neither, and only on closed doors. For digging (§8
  "Digging"), terrain that breaks has a `hardness` (1 to 10 000, the work it takes: a pick-axe at
  Strength 10 does 10 a turn) and what it `becomes` (walkable, transparent open ground that does
  not break, no door or stairs: rubble), together; only solid terrain (not walkable, no door,
  stairs, or light) breaks, so digging and blasts never break a door. Stone walls 600, packed
  earth 100, granite 3000. Solid rock may have a `face`, the solid terrain it shows as once a cell
  next to it is broken open (`void`: `wall`). Overworld terrain has
  `travel_minutes` (1 to 1440; required for every walkable terrain on the overworld),
  optionally `local`, the ground terrain of the local area entered from a cell of it (none:
  nothing to explore there; locations never have one), and `landscape`, what its land is like in
  local areas (required with `local`): `plains`, `grassland`, `forest`, `dense_forest`, `hills`,
  or `swamp` pick the generator of its own areas; `road` runs a road through them amid the
  neighbors' land; `river` and `lake` put running or still water along that side of the
  neighboring areas (§3). Water that freezes (§5 "Ice") has a `freeze` table: `below` (ice grows
  while the air is colder than this, °C, 0 down to −30; it melts above 0), `rate` (percent of
  still water's pace, 1 to 100, default 100), and `plunge` (dice: the hit points a fall through
  the ice into it takes, never the last one); still water `{ below = 0, plunge = "1d3" }`,
  running water `{ below = -6, rate = 40, plunge = "1d3" }`, a flooded room's shallow water
  `{ below = 0, plunge = 1 }`. Ice is terrain with `ice`, the
  thinnest ice in mm that looks like it (1 to 1000, distinct; the next one's takes over from
  its thickness): thin ice (10, hairline cracks `⁄ ∖ ·` over dark water), ice (50, a grey-blue
  sheet, a dot here and there), thick ice (150, pale and frosted all over), each `move_cost`
  120. Water that freezes and ice are walkable and transparent, give no light, and are neither
  doors, stairs, nor overworld terrain; ice does not flow; if any water freezes, some terrain
  must be ice. Presentation-only, any terrain may have `variants`: glyphs the map draws in place
  of its `glyph`, one per cell picked by a hash of its position (a space for none; look mode
  and dumps use `glyph`). The calendar's month names are
  `data/calendar.toml` (exactly 12; without the file months are numbered).
- **Searching** (`data/search.toml`, optional; without it the numbers below; §8 "Searching"):
  `passive_penalty` (what noticing in passing loses against searching, 0 to 20; 4), `turns` (how
  long searching for a while lasts at most, 1 to 100; 10), and `secret_door` (a secret door's
  difficulty where its level gives none, 1 to 50; 14).
- **Action costs** (`data/actions.toml`, optional): action ids mapped to their cost in energy,
  e.g. `wait = 1000`, 1000 being a normal action (6 s at normal speed; §8 "Time"), 100 to
  10 000. The ids are `step` (onto terrain of `move_cost` 100), `wait`, `open_door`,
  `close_door`, `unlock` (unlocking a door with a key, on top of opening it), `kick`, `lantern`,
  `stairs` (also entering a location or local area and walking off an
  area's edge), `pick_up`, `drop`, `melee` (a shield bash too), `shoot`, `throw`, `reload` (a
  missile weapon that `reloads`: a crossbow shot costs `shoot` plus `reload`, 2000), `cast`,
  `dig` (a turn of digging), and for equipment (each slot names
  one, below) `wield` (1000), `wear` (1000: helmets, cloaks, gloves, boots, rings, amulets),
  `body_armor` (3000: it takes a while to strap on), `ready` (1000: a missile weapon, missiles,
  a tool), `eat` (1000: a turn of eating; a meal takes the food's `turns`, §8 "Hunger"), `chat`
  (500: a few words, §8 "Dialogue"), and `climb` (2000: climbing out of the water onto ice, on
  top of the step onto it; §5 "Ice"). An action left out, or every action without the file,
  costs 1000; an unknown id or a cost out of range is a content error. The wiki shows them in a
  generated table.
- **The sky** (`data/sky.toml`, optional; without it there are no moons): up to 8 `[[moon]]`s,
  each with `name` (shown in the status bar when there are several), `color` (`[r, g, b, uv]`,
  the direct light of the moon when full and high in the night sky), `sky` (the same for the
  light of the sky around it; default none), `size` (apparent size in percent of a standard moon,
  1 to 400, default 100; its light scales with the square), `period_days` (new moon to new moon,
  1 to 3600), and `phase_offset_days` (its age at the calendar epoch, less than the period,
  default 0). §5 "Moonlight".
- **Objects and materials** (M11): every object (`[[object]]` in `data/objects.toml`, a
  directory read in path order, one object a file; see Definitions) defines the
  symbol it renders with and its color, item class, material, and weight: `id`, `name` (singular,
  without an article; `plural` defaults to the name plus `s`), `glyph`, `fg`, `class`, `material` (an id in
  `data/materials.toml`, likewise a directory, whose `[[material]]` entries have an `id` and a
  `name`; properties come later), `weight` (integer grams, at least 1), `value` (what one is worth in gold coins,
  required, 0 to 1 000 000; shops price their goods from it, §7 "Shops"), and `stackable`
  (default false). Glyphs follow ADOM loosely: `)` weapons, `}` missile weapons, `/` missiles, `[`
  armor, `(` tools, `!` potions, `?` scrolls, `%` food, `*` stones and gems, `$` coins, `=` rings,
  `"` amulets. **Item classes** (#189) say what kind of thing an object is, ADOM-like:
  `data/item_classes.toml` is their ordered list (`[[class]]` with `id`, `name`, the header item
  lists show, `kinds`, and `corpses`): melee weapons, missile weapons, missiles, shields, body
  armour, helmets, cloaks, gloves, boots, rings, amulets, light sources, tools, food (corpses
  included, `corpses = true`), potions, scrolls, gems, coins, miscellaneous. Every object names
  its class (`class`, required); a class lists the magic item kinds (`kind`, §8 "Magic items")
  of its objects, each kind in one class only, so an object of a kind is of that kind's class
  and data cannot contradict itself. Every item list groups by class in this order (§6 "Item
  lists"), and shops sell and buy by class (§7 "Shops"). The player knows the class of anything
  they see (an unidentified item's is its base item's).
  Every creature in `data/creatures.toml` has a base `weight` in grams (required, at least 1).
  Things that can be worn or wielded (M16) say where they go: `slot`, a slot's id or what it
  `fits` (`slot = "body"`, `slot = "ring"` for either ring slot; an unknown one is a content
  error), `two_handed` (default false; only with a hand slot's `slot`, and there must be two
  hand slots: it takes both), `light_weapon` (default false; only for a one-handed weapon with a
  hand slot's `slot`, not two-handed: it fits every hand slot, the off hand too, for fighting
  with two weapons, §8 "Fighting styles"; `Content::fits`; the built-in light weapons are the
  dagger, the stiletto, the short sword, and the hand axe, as in D&D: one-handers like the
  scimitar, the rapier, and the club are not), `ammo` (the kind of ammunition it is, `ammo = "arrow"`), and
  `launches` (the kind a missile weapon shoots, `launches = "arrow"`; some object must be that
  ammunition). Launchers and ammunition pair by that kind (`Content::fires`). Food (§8
  "Hunger") has `food = { nutrition = 1500, turns = 4, keeps = "30d", lasts = "60d" }`: the
  nutrition points it gives (1 to 100 000), the turns of eating it takes (1 to 100; never on
  things worn, wielded, shot, thrown, or burned), and its shelf life in whole hours or days
  (`"8h"`, `"30d"`): how long after it was made it `keeps` (1 hour or more; old past half of it,
  rotten after it) and `lasts` (longer than it keeps, at most 3600 days; then it has rotted
  away): strips of dried meat (600, 2 turns, 60 and 120 days), apples (150, 1, 14 and 28), food
  rations (1500, 4, 30 and 60), loaves of bread (800, 3, 5 and 10), angelica roots (20, 1, 30
  and 60). A remedy also has `cures`, the means it
  applies as it is eaten (names some status or disease lists in its `cured_by`, else a content
  error; food only): the angelica root `cures = ["remove_disease"]` (§8 "Diseases"). A
  **scroll** (§8 "Scrolls") has `read`, what reading it does (never on things worn, wielded,
  eaten, shot, or thrown): `kind = "burst"` or `"bolt"`, a trap's spell with a trap's fields
  (`spell`, an effect of its shape in `data/effects.toml`; `noun`, `verb`, `damage`; a burst's
  `radius`, `blast`, `heat` or `frost`, and `range`, how far it flies, 1 to the view radius; a
  bolt's `length`; an optional `noise` and `message`), or `kind = "identify"`; with `read` an
  object may have `level` and `frequency`, as base items do, and is loot (never magic): the
  scrolls of fireball (item level 3), lightning (2), frost, and identify. **Corpses** are objects too, one made for every creature type
  that leaves one (not the player) when content loads, after the objects of the file in creature
  order, with the id `<creature>_corpse`: "kobold corpse", `%` in the creature's color, made of
  the corpses' material (`data/hunger.toml`), weighing the creature's weight, not stackable, and
  feeding and keeping by the creature's size. Level files and `carry` cannot place them (a
  content error): only creatures that die leave them, each knowing when it died. What they do in a
  fight (§8 "Combat"; only for things with a `slot`): a weapon's `damage` dice (`"1d8"`; a
  launcher's, required for one, are its shots' and carry its draw weight, `"1d8+2"`), `threat`
  (the lowest natural roll that threatens a critical, 2 to 24, default 22), `crit` (the damage
  multiplier, 2 to 5, default 2), `to_hit` (default 0, within ±20), and for bows `strength` (the
  Strength they need, 1 to 40), all four only with `damage`; `parry` (Defence while wielded),
  `armour` (while worn) and `max_dex` (the most Dexterity may add to Defence while it is worn),
  and `shield` (while held), each 0 to 20; `bash` (a shield's damage dice when bashing with it,
  only with `shield`; §8 "Weapon skills"); and `category`, the weapon skill a weapon draws on and
  trains (an id in `data/weapon_skills.toml`; required for every weapon when there are weapon
  skills, refused on anything else): a `melee` skill for what goes in a hand, a `launcher` one
  for a missile weapon, the `thrown` one for what is only thrown (a rock). Ranged combat (§8 "Ranged combat"): `range` (1 to
  40 cells; required for a missile weapon: how far it shoots; on anything else it makes a
  throwing weapon, thrown that far with its own `damage`, which then needs no `slot`: a rock;
  ammunition has none), `reloads` (missile weapons: each shot also takes the `reload` action),
  `breaks` (ammunition and throwing weapons: the chance in percent, 0 to 100, that it breaks on a
  hit), `lit` (ammunition and throwing weapons: the object it flies and lands as, lit as it is
  loosed, which must burn), `light` and `burns` (together: the light preset it gives where it
  lies and for how many seconds, 1 to a day, it burns there before it is gone); and
  for digging (§8 "Digging"): `dig`, a digging tool's power (1 to 100; it needs a `slot`); and
  presentation only: `missile` (the glyph it flies as, drawn pointing right; default its own,
  upright), `spins` (it tumbles in flight), `glow` (0 to 1, where it lies), and particle
  `particles` (`ambient`: while it lies in view on top of its pile; `flight`: while it flies).
  Keys (M19, §8 "Doors"): `key`, the lock id it opens, or `*` for a skeleton key, which opens
  every lock. The keys: an iron key (`pantry`, the hub's pantry), a brass key (`guard_post`, the
  guard post vault's store room), a rusty key (`depths`, the rooms locked in the Depths), and a
  skeleton key (in the guard post's store room).
  Test objects: weapons for the main hand (dagger 1d4 and short sword 1d6, threat 20, both
  light, so either hand; spear 1d8 ×3; club 1d6; a two-handed quarterstaff 1d6 with parry 1; the
  dagger thrown 5 cells, the spear 6; of the daggers, short blades, spears, maces, and staves
  skills), a small shield (off hand,
  shield 1, bash 1d3), an iron helm, a wool cloak, leather gloves and boots,
  body armor (leather jerkin armour 2, max Dexterity 6; chain mail shirt armour 4, max Dexterity
  4), a silver ring, amulets; missile weapons: a short bow (1d6 ×3, Strength 10, range 10), a long
  war bow (1d8+2 ×3, Strength 14, range 15), a crossbow (1d10+1, threat 20, no Strength, range 13,
  reloads), a sling (1d4, range 9) (of the bows, crossbows, and slings skills); ammunition: arrows (break 30 %), flaming arrows (lit as burning
  arrows: light `arrow_fire`, burning 60 s), crossbow bolts (break 20 %), sling stones; rocks
  (thrown 1d3, 6 cells, readied in the missiles slot; the thrown skill); digging tools for the
  Tool slot: a pick-axe (power 10, 3 kg) and a mattock (power 15, 4.5 kg), never magic (no
  `kind`).
  **Base items** (M29, #149; §8 "Magic items"): `kind`, what kind of item it is for magic
  (`weapon`: has `damage` and a hand `slot`; `launcher`: `launches`; `ammo`: `ammo` and a `slot`;
  `shield`: `shield`; `body`, `helmet`, `cloak`, `gloves`, `boots`, `ring`, `amulet`: worn, no
  `damage`; anything else is a content error): only things with a kind are ever magic or loot;
  `level`, the lowest item level loot holds it at (1 to 99, default 1), and `frequency`, how often
  (default 100, 0 never), both only with a kind. Weapons (things with `damage`) may have an
  `attack_cost` (100 to 10 000; worn or wielded weapons only), what a blow with it (a missile
  weapon: a shot) costs instead of the `melee` (`shoot`) action, before its weapon skill's
  `speed` (a shield bash costs what a blow with the shield does: `melee`, never the wielded
  weapon's `attack_cost`). The built-in base items, folding in the test
  objects (every weapon names its weapon skill `category`, which also names it in item
  descriptions: "Long blade, for the main hand"; rocks are `thrown`): daggers and stilettos (threat
  19, quick, 900), short swords and scimitars (threat 19), long swords (1d8, threat 20), rapiers
  (1d6, threat 19, 900), bastard swords (1d10), great swords (2d6, both hands, 1200), hand axes
  (thrown 4), battle axes, great axes (1d12 ×3, both hands, 1200), spears, javelins (thrown 8),
  tridents (parry 1), clubs, maces, morning stars (2d4), war hammers (×3), mauls (2d6 ×3, both
  hands, 1200), halberds and glaives (1d10, both hands, 1100), quarterstaves and ironwood staves;
  short, composite, and long war bows, crossbows and heavy crossbows (1d12+2), slings; arrows,
  crossbow bolts, sling stones (flaming arrows are never magic); body armour from a padded coat
  (Armour 1, Dexterity 8) through a leather jerkin, studded leather (3/5), chain mail (4/4), scale
  mail (5/3), and banded mail (6/1) to plate armour (7/1); leather caps, iron helms, great helms;
  wool, travelling, and fur cloaks; leather gloves and gauntlets; leather and iron-shod boots;
  bucklers and small shields (1), large shields (2), tower shields (3, Dexterity 2); iron,
  silver, and gold rings; bone, jade, amber, and gold amulets. Helmets, cloaks, gloves, boots,
  rings, and amulets have no stats of their own (no implicits yet): they are worn for their magic.
  The wiki lists them all in a generated table.
- **Magic items** (`data/affixes.toml`, optional; without it loot is always normal; M29, #149;
  §8 "Magic items"; the settings below live in that file and the affixes themselves in the
  `data/affixes/` directory, one a file, read in path order; see Definitions): `[rarity]` the
  weights of normal, magic, and rare loot (70, 24, 6, not all
  0); `[magic] both`, the chance in percent that a magic item has a prefix and a suffix (25; else
  one of either, evenly); `[rare] affixes`, `[count, weight]` pairs for how many affixes a rare
  item rolls (1 to 6; built in: 3 40, 4 35, 5 17, 6 8); `[names] first` (words, none empty) and
  `[names.second]`, words for every kind of item (rare names: one of each, "Grim Bite"). Then
  `[[prefix]]`s and `[[suffix]]`es (ids unique across both): `id`, `name` ("Sharp", "of the
  Bear"), `effect` (one of `to_hit`, `damage`, `extra_damage` (adds 1dN), `threat`, `crit`,
  `attack_speed`, `range`, `draw`, `durability`, `parry`, `armour`, `shield`, `max_dex`,
  `weight`, `dodge`, `deflection`, `natural`, `max_hp`, `regeneration`, a stat id, `move_speed`,
  `carry`, `stealth`, `light`, `skill` with a `skill`, an id in `data/skills.toml`, whose value
  it raises; without skills content, skill affixes are left out), `bases` (kinds of item it
  rolls on, at least one; an effect only
  attacks can use, `extra_damage`, `range`, `draw`, `durability`, needs weapons, missile weapons,
  or ammunition, `parry` weapons, `armour` body armour, `shield` shields, `max_dex` body armour or
  shields; else a content error), `weight` (default 100), and `tiers`, at least one, item levels
  rising: `{ level, min, max }` (values within the effect's range: 1 to 10 for most, dice sides 2
  to 20 for `extra_damage`, 1 to 3 for `threat` and `crit`, percent 1 to 50 for speeds, 1 to 90
  for `durability`, `weight`, `stealth`, 1 to 100 for `max_hp`, `carry` (kg), and `skill`, 1 to
  300 for `regeneration`), or for `light` `{ level, light }` naming a light preset. The built-in
  affixes: 21 prefixes and 26 suffixes in up to four tiers (item levels 1, 3, 5, 7 for most); the wiki
  lists them in generated tables. The light presets `item_glow_faint`, `item_glow`, and
  `item_glow_bright` (radius 2, 3, 4; steady, cool) are the glowing tiers'.
- **Weapon skills** (`data/weapon_skills.toml`, optional; without it nobody has weapon skills and
  weapons name none; M22, §8 "Weapon skills"): `[marks]` (`attempt`, `hit`, `per_level`, at most
  1000, and `most`, a percent 100 to 1000), `[curves]` (named lists of the marks each rank asks
  for, ranks 1 to 10: ten numbers, each more than the last, at most 1 000 000), `[[skill]]`s in
  the order the weapon skills screen lists them (at most 255): `id`, `name` (`Long blades`),
  `phrase` (the message's `with long blades`), `role` (`melee`, `launcher`, `thrown`, `unarmed`,
  `shield_block`, `shield_bash`, `style`; `unarmed` to `shield_bash` at most one skill each),
  `curve` (a key of `curves`), and its benefits as eleven numbers, ranks 0 to 10 (0 at rank 0,
  never less at a higher rank; a list left out is all 0): `to_hit` and `damage` (0 to 20),
  `threat` (0 to 10), `crit` (added to a confirmed critical's multiplier, 0 to 3, never past ×5),
  and `speed` (percent off the attack's cost, 0 to 50) for the roles that attack; `parry` (0 to
  20) for `melee`, `reload` (0 to 50) for `launcher`, `shield` (0 to 20) for `shield_block` (a
  benefit the role does not use is an error). A fighting style (`role = "style"`, §8 "Fighting
  styles") also has its `grip` (`one_weapon`, `two_weapons`, `two_handed`, `weapon_and_shield`,
  `unarmed`, `launcher`, `thrown`) and may name `weapons` (weapon skill ids of the roles its
  grip fights with: `melee` for the hand grips, `launcher` for `launcher`, `melee` or `thrown`
  for `thrown`, none for `unarmed`; each once; left out: every weapon of its grip that no other
  style of the grip names; within a grip at most one style leaves it out and no weapon skill is
  named twice: one configuration, one style); its benefits may also be `parry` (grips that
  wield a weapon), `shield` and `bash` (the chance in percent, 0 to 100, of a shield bash after
  a blow that hits; `weapon_and_shield`), `reload` (`launcher`), `ease_main` and `ease_off`
  (points off the two-weapon penalties, 0 to 20; `two_weapons`); `grip` and `weapons` on a
  skill that is no style are errors. `[two_weapons]` (optional; default −4 and −8): `main` and
  `off`, the to-hit penalties of the main hand's and the off hand's blows with a weapon in each
  hand (−20 to 0). `[[default]]`s, `between = [a, b]` and `penalty` (1 to 10), each pair once,
  symmetric; styles default only to styles, weapon skills only to weapon skills. The built-in
  file: marks 1 an attempt, 2 more
  a hit, 20 % a level, at most 200 %; curves melee 15, 30, 60, 120, 240, 480, 960, 1920, 3840,
  7680 and missile 10, 20, … 5120; sixteen skills: daggers and knives, short blades, long
  blades, great blades, axes, maces and clubs, spears, polearms, staves (`melee`), unarmed;
  bows, crossbows, slings (`launcher`, missile curve), thrown weapons (missile curve); shield
  blocking, shield bashing; defaults: long ↔ short blades −2, long ↔ great blades −2, short
  blades ↔ daggers −2, long blades ↔ daggers −4, short blades ↔ thrown −4, daggers ↔ thrown −2,
  axes ↔ maces −2, axes ↔ polearms −3, maces ↔ staves −3, spears ↔ polearms −2, spears ↔
  staves −3, spears ↔ thrown −4, polearms ↔ great blades −4, bows ↔ crossbows −4, slings ↔
  thrown −3, shield blocking ↔ shield bashing −3; two weapons −4 and −8; eight styles after the
  sixteen skills (§8 "Fighting styles": Duelist, Twin Blades, Great Weapon, Sword and Board,
  Marksman (bows, crossbows, slings; missile curve), Skirmisher (missile curve), Brawler, Staff
  Fighter (staves)), with the defaults duelist ↔ sword and board −3, duelist ↔ twin blades −3,
  great weapon ↔ staff fighter −3, marksman ↔ skirmisher −4. The wiki shows the skills, the
  styles, the defaults, the marks and curves, and the benefits per rank in generated tables.
- **Equipment slots** (`data/slots.toml`, optional; without it nothing can be worn or wielded;
  M16): up to 26 `[[slot]]`s in the order the equipment screen lists them (lettered `a` on):
  `id` (objects name it in `slot`, replays in `equip:SLOT:LETTER`), `name`, `fits` (what objects
  give as their `slot` to go here; default the id, so slots may share it: `left_ring` and
  `right_ring` both fit `ring`), `action` (the action that puts a thing on here or takes it off,
  for its time: `wield`, `wear`, `body_armor`, or `ready` in `data/actions.toml`), `mark` (what
  the inventory says of a thing here: "being worn"), and `hand` (default false; a two-handed
  object takes every hand slot, and a light weapon fits every one). The built-in slots, ADOM's: Head, Neck, Body (`body_armor`),
  Cloak, Main hand and Off hand (`wield`, hands; "wielded", "in off hand"), Hands, Left ring
  and Right ring ("on left hand", "on right hand"), Feet, Missile weapon and Missiles (`ready`;
  "readied", "quivered"), Tool (`ready`, "in use"; M19); the rest `wear`, "being worn". The wiki
  shows them in a generated table with what fits each.
- **Factions** (`data/factions.toml`, optional; without it everyone is neutral): `[[faction]]`
  with `id`, `name`, and `hostile` (faction ids). Relations are pairwise and symmetric: if A lists
  B, A and B are hostile to each other; every pair not listed (a faction and itself too) is
  neutral. Validation: unique ids, known ids, no faction hostile to itself, a `player` faction
  (the player's), and every creature type but the player naming a known faction. The core API
  gives the stance (`Hostile` or `Neutral`) of two factions, creature types, or actors. Test
  factions: `vermin` (giant rats; hostile to the player), `kobolds` (the war band; hostile to the
  player and the vermin), `wildlife` (deer, cave bats, the cave beetle; neutral to everyone), and
  `peaceful` ("peaceful folk", hostile to no one: the showcase's people by type, and creatures
  levels place in it); `merchants` (shopkeepers; hostile to no one) and `outraged` ("outraged
  shopkeepers", hostile to the player), which a robbed or attacked shopkeeper joins (§7
  "Shops"). A level
  legend entry with a `creature` may name a `faction` (a known faction id; an unknown one is a
  content error) that replaces its type's for that creature: it is stored on the actor
  (`faction_override`, saved and hashed only when set), used wherever stance is computed (AI
  targeting and noise, look mode "(hostile)", bump-to-attack vs "in the way"), and so can make a
  hostile type harmless in the showcase. The showcase's people belong to `peaceful` by type. A
  legend entry with a `creature` may also name an `npc` (an npc id of `data/dialogue.toml`; one
  without a dialogue there is a content error): the unique character that creature is, which
  speaks the npc's dialogue rather than its type's (stored on the actor, saved and hashed only
  when set). Level files may also have an `arrival` line, said each time the player arrives by
  stairs.
- **Dialogue** (`data/dialogue.toml`, optional; without it nobody talks; §8 "Dialogue"): each
  `[[dialogue]]` names exactly one of `creature` (a creature type, not the player) or `npc` (an
  npc id: lower-case letters, digits, and underscores), at most one dialogue each, and has `lines`
  (at least one one-liner), an optional `greet`, optional `[[dialogue.topic]]`s (at most 9, each
  with a unique `id` in lower-case letters, digits, and underscores, a `name`, and a `reply`), and
  an optional `mutter = { chance, verb, lines }` (`chance` in percent per action, 1 to 100; `verb`
  one lower-case word, default "mutters"; at least one line). Lines, greetings, topic names, and
  muttered lines are one line of at most 72 characters (they fit the message line with the
  speaker's name), replies at most 280 (the dialogue screen wraps them); none is empty or padded.
  Built in: the gardener (one-liners; grumbles), the wandering bard (one-liners and topics: the
  moon, the Great Race, this year, the Wildlands; sings), and the old hermit (a greeting,
  one-liners, and topics: this hall, the dark chamber, the gallery, the water garden, these
  stairs; mutters).
- **Quests** (`data/quests.toml`, optional; without it nobody gives quests; §8 "Quests"): each
  `[[quest]]` has a unique `id` (lower-case letters, digits, and underscores), a `name` (the log's
  title and the topic, at most 32 characters), a `giver` (a creature type with a dialogue, or an
  npc id with one), a `type` and a `target`: `kill` (a creature type, not the player; `count` 1 to
  99, default 1; optional `level`, a level id or a dungeon branch id for any of its levels),
  `bring` (an object type; `count`; optional `to`, someone else with a dialogue who takes it),
  `reach` (a level id, a branch level such as `depths:3` included; never `repeatable`), or
  `deliver` (an object type the giver hands over; `to` required, not the giver); a field another
  type has is a content error. `reward = { gold, items, xp }` (each optional: gold coins, at most
  9999, which needs the `gold_coin` object; objects as levels place them; experience, at most
  1 000 000). Texts, one line each of at most 280 characters, neither empty nor padded: `offer`,
  `accept`, optional `decline` (default "Another time, then."), `reminder` (the giver's), and
  `done` (said by whoever it is handed in to). Optional `requires` (a quest earlier in the file,
  done before this one is offered) and `repeatable` (default false). Everyone's topics must fit
  the dialogue screen's nine: their dialogue's own, one per quest they give or take in, and one
  more for an offer's accept and decline. Built in: the old hermit's "Rats in the arena" (kill 3
  giant rats in the arena; 20 gold, 30 XP) and, after it, "Three levels down" (reach the Depths 3;
  50 gold, a jade amulet, 100 XP), the gardener's "Three apples" (bring 3 apples; 10 gold, 15
  XP), and the wandering bard's "The bard's letter" (deliver a sealed letter, a new object, to the
  old hermit; 5 gold, 15 XP).
- **The weather** (`data/weather.toml`, optional; without it the sky is always clear and every
  month a mild 10 °C with a swing of 8): `swing` at the top (a clear day's swing of temperature
  from the coldest hour to the warmest, °C, 0 to 30), exactly 12
  `[[month]]`s in calendar order, each with the chance in percent of `clear`, `overcast`, `rain`,
  and `heavy_rain` (adding up to 100) and its mean `temperature` (°C, −40 to 40; §5
  "Temperature"), and a table for each of `overcast`, `rain`, and
  `heavy_rain` (clear is the sky as it is): `direct` and `sky` (percent of the direct light of the
  sun and the moons, and of the sky light, that gets through; 1 to 100), `scatter` (percent of the
  direct light the clouds spread over the sky as sky light, 0 to 100, default 0), `sight` (cells
  of overworld sight lost, 0 to 8, default 0), `swing` (percent of a clear day's swing of
  temperature left under it, 0 to 100, default 100), `chill` (how much cooler it is, °C, 0 to
  15, default 0), and presentation-only `ambient` particle emitters
  (§4.8; `rate` per cell and second): `drops` (on every visible cell under the sky), `splash` (on
  open ground: walkable, transparent, not running water), and `ripples` (on still water, terrain
  with a `wave` animation). §5 "Weather".
- **Spell effects** (`data/effects.toml`, optional; presentation only, no rule reads it, and it is
  not in the content fingerprint): `[[effect]]` with a unique `id` (no `:`, `@`, `>`, or spaces),
  `shape` (`burst` or `bolt`), `radius` (bursts: the dev action's radius, 0 to 8, default 1),
  `speed` (cells a second: a burst's projectile, a bolt drawing out), `grow` (bursts: seconds to
  full size, at most `impact`), `impact` and `linger` (seconds, up to 2; `linger` may be 0),
  `flashes` (bolts: 1 to 6), `flicker` (seconds a glyph shape lasts), `glyphs` (bursts: by heat,
  hottest first; bolts: the line glyph, then the forks'), `projectile` (bursts), `colors`
  (hottest first), `glow` and `fill` (bursts: the opacity of each cell's glowing body) and
  `branches` (bolts: the chance of a fork at each corner) from 0 to 1, optional
  `light = { color = [r, g, b], radius }` (255 = fully lit, up to 4000), `darken = { amount,
  seconds }` (amount 0 to 0.9), `shake = { pixels, radius, seconds }` (pixels 0 to 12, radius up
  to the view radius), and `[[effect.particles]]` emitters on `impact`, `linger`, or (with a
  `projectile`) `flight`. Every glyph joins the atlas. §4.8 "Spell effects".
