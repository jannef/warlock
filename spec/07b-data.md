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
  `name` and a `plural` (default by the plural dictionary, `data/plurals.toml` below: mostly the
  name plus `s`; the deer's is "deer"; quest messages count them), optional `tags` (plain
  singular words, lower-case words joined by hyphens, 1 to 24 characters, each once: `["goblin",
  "archer"]`; nothing declares a tag, a creature carrying it is all there is; a kill quest may
  ask for a tag, §8 "Quests"; never shown to the player), a
  `speed` (100 to 10 000, default 1000, the player's; never shown to the player; §8 "Speed"): the
  giant rat 1200, the cave bat 2000, the kobold 900, the deer 1300; and for hearing (§5)
  `footsteps` (at most 300; default 0, silent) and `hearing` (the faintest loudness heard, 1 to
  300; default none, deaf). For behavior (§8 "Creature behavior") a creature type names its
  `faction` (below; required when factions exist, the player's is `player`), its eyes `vision`
  (a profile; default `normal`) and `sight` (radius, 1 to 40, default 10), how it fights:
  `combat = "melee"` (default), `"ranged"`, or `"caster"`, the latter two with a `range` (2 to its
  sight), `opens_doors` (default false), `budge` (whether the player may swap places with it
  when it is not hostile, §8 "Displacement": `always`, the default; `off_post`, not while it keeps
  its post, going about its routine, as a shopkeeper at its counter will; `never`), its
  `alignment` (`good`, `neutral`, the default, or `evil`; §8 "Alignment": the kobolds evil, the
  beasts neutral, the showcase's people good; never shown), an optional `race` (a race id; §8
  "Alignment": its corpse is kin to a character of that race; the showcase's people are human;
  not checked in content without races), and
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
  natural attack, used when it wields no weapon (`{ damage = "1d4", damage_type = "piercing",
  name = "bite", verb = "bites" }`, `damage_type` the kind of damage it does (§8 "Damage
  types": the bites piercing, the deer's hoof bludgeoning),
  with optional `threat`, `crit`, and `to_hit` as for weapons, and `on_hit`, the status
  effects and diseases a hit with it may cause if the target lives: `[{ status = "poisoned",
  chance = 10, severity = 2 }]`, the chance in percent (1 to 100) and the severity in degrees (1
  to the status's `max`; §8 "Status effects"), or `{ disease = "filth_fever", chance = 10 }`
  (exposure to the disease, which the target then resists with a check; §8 "Diseases"; never the
  status diseases show as), or `{ drain = "strength", points = 1, chance = 100, check =
  "willpower", difficulty = 14 }` (a drain: unless the target makes a check of `check`, Willpower
  by default, against `difficulty`, 1 to 40, `points`, 1 to 5 (default 1), come off its score in
  the stat for good; `drain = "random"` draws one of the six each time; §8 "Stats": "Drain");
  without one, bare hands: 1d3, "fist", "hits"); and
  `natural`, its natural armour (`{ defence = 1, protection = 3, name = "shell" }`: `defence`
  in the Natural category of Defence, `protection` soaking damage off every hit with what it
  wears, each 0 to 20, not both 0; the name is said when it turns or soaks a blow); `resist`,
  what it resists in percent per damage type (`{ acid = 50, poison = 25 }`, −100 to 100, 100
  immune; §8 "Damage types"; the cave beetle acid 50 and poison 25, the kobold shaman fire 25;
  never shown). For experience (§8
  "Experience"): `level` (1 to 99, default 1) and `xp` (0 to 100000, default 0; the player
  creature has neither): the kobold 2/15, the kobold archer 3/25, the kobold shaman 4/40, the
  giant rat 1/5, the cave bat 1/3, the cave beetle 2/8, the deer 1/2; new creatures follow the
  guideline of `data/experience.toml`, about `5 × level + 10` (§8 "Experience"). For loot (§8 "Magic
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
  player's size comes from their race when the run has a character). Its `rarity` (§7 "Danger
  levels"; never shown), a tier in `data/creature_rarities.toml` (`common`, the default,
  `uncommon`, `rare`, `very_rare`): how often a population or a spawn draws it. Skills (§8 "Skills"; never
  shown): `skills`, the skills it is trained in by skill id with fixed values (0 to 999); the
  rest untrained; the player creature lists none. The deer listens (30), the kobold shaman knows
  concentration (20) and spellcraft (25). The showcase's people (M28): the gardener, the
  wandering bard, and the old hermit (speed 800), glyph `@`, of the peaceful folk by type, level
  1 and worth no experience. The campaign's creatures come in waves, one per dungeon
  (docs/design/campaign.md §10.1), each type on the `5 × level + 10` guideline and on the fields
  above, with a gear table where it is a humanoid (§8 "Creature gear"); the wiki's creature
  tables give every number. The barrow wave (the barrows of Cealdhlāw, danger 1 to 6): the grave
  worm 1 (slow, a mildly poisonous bite), the barrow spider 2 (quick, silent, poisonous), the
  goblin 2 (racial club, loud, opens doors), the goblin archer 3 (racial short bow and arrows, a
  club in reserve), the skeleton 3 (resists piercing 50, takes bludgeoning double, no corpse, no
  blood; an old sword or spear now and then), the zombie 3 (slow, many hit points, grave rot on
  a hit), the giant centipede 3 (plates, poisonous), the barrow ghoul 4 (quick, keen slashing
  claws whose criticals bleed), the goblin chief 5 (club, short sword, jerkin, coins), and the
  barrow wight 6 (a caster whose touch is cold, resists cold, no corpse; an ancient long sword,
  sometimes magic). The undead are immune to poison. The barrow wight is the Ascmearc's own type
  (`docs/dev/campaign.md`). The undead roster's first batch (#383, #384) joins it: the crawling
  hand 1 and the skeletal rat 1 (small, quick, bone that a club breaks), the kobold zombie 2
  (slow, grave rot now and then), the skeleton archer 4 (a short bow and arrows, keeps its
  distance), the skeletal wolf 5 (quick bone), and the skeleton warrior 7 (a long sword or spear,
  a small shield, chain mail now and then). The fen wave (the drowned minster of Fennmynster, danger 4 to 11):
  the giant leech 4 (its bite leaves a bleeding wound), the giant frog 4 (quick), the drowned 5
  (undead, slow, filth fever on a hit), the crocodile 6 (a strong bite, scales 2/3), the
  cultist 6 (racial dagger, a scroll now and then), the will-o'-wisp 7 (a caster that glows,
  silent, hard to hit, no corpse), the ghost 8 (undead, resists slashing, piercing, and
  bludgeoning 50, a cold touch, no corpse; incorporeal, §8 "Incorporeal creatures"), the mud
  golem 8 (slow, protection 5, no corpse), the
  cult priest 9 (a caster with the cult's dagger, sometimes magic, and often a scroll), the marsh
  troll 9 (huge, regenerating by its Toughness), and the bog hag 11 (a caster with a staff, often
  magic; unique: a population draws her once a run). The cultist, the cult priest, and the bog
  hag are the Ascmearc's own types. Its undead (#384): the corpse candle 2 (small, incorporeal,
  glows, a cold touch), the poltergeist 6 (quick, incorporeal), the ghast 7 (quick, opens doors,
  filth fever on a hit; its claws may hold), the shadow 8 (incorporeal, a cold touch that drains
  Strength, §8 "Stats": "Drain"), and the bog body 9 (slow, resists cold, marsh ague on a hit);
  and from the roster's second batch (#385) the wraith 11 (incorporeal, a cold touch that drains
  Toughness) and the fen ghoul 11 (quick, opens doors, claws that may hold and carry marsh
  ague). The delf wave (the duergar delf of Fyrdelf, danger 8 to 19): the ember beetle 8 (a
  burning bite, shell, resists fire 50, a faint glow), the duergar miner 9 (a pick-axe
  held as a tool, the racial hand axe half the time, a leather cap), the duergar warrior 11
  (racial hand axe, small shield, chain mail, sometimes magic), the ogre 12 (slow, racial maul),
  the fire salamander 12 (immune to fire, cold double, lights its way), the troll 13 (so tough its
  wounds close fast, fire hurts it more), the duergar smith 13 (racial war hammer, sometimes magic,
  chain mail), the magma crawler 14 (slow, a searing touch, slag hide, cold hurts it, no corpse),
  the ash wraith 15 (an undead caster with a cold touch, no corpse), the basilisk 16 (scales, a
  bite that leaves one Sluggish), the cinder elemental 17 (immune to fire, bright, no corpse), and
  the fire drake 19 (the Ascmearc's own, unique: a caster whose breath is announced, scales,
  immune to fire). Its undead (#384): the ogre zombie 9 (slow, heavy fists, grave rot) and the
  grave hound 10 (quick, grave rot), and from the second batch (#385) the burning skull 12 (small,
  lit by its own fire, a burning bite, immune to fire, hurt more by cold), the knocker 13 (a
  miner's spirit, incorporeal), the spectre 14 (incorporeal, a cold touch that drains
  Willpower), the charred dead 14 (slow, resists fire, hurt more by cold), the duergar skeleton 15
  (a war hammer and an iron helm, bone that a club breaks), and the rotting troll 17 (slow, huge
  claws, grave rot, fire hurts it more). In no population yet, waiting for deep places (#383): the
  mummy 10 (slow, mummy rot on a hit, fire hurts it more), the draugr 12 (huge, resists cold, a
  great axe half the time), the barghest 13 (a quick black dog whose bite frightens), the vampire
  spawn 16 (quick, opens doors, a bite that drains Toughness, fire hurts it more), the hand of
  glory 16 (small, lit by its fingers, a cold touch that transfixes), the gallows dead 17 (a
  tongue that holds fast), the gibbering shade 18 (incorporeal, a cold touch that drains
  Intelligence), the bone wyvern 18 (a sting, bone with protection 2), the revenant 19 (very
  tough, a long sword half the time), the fetch 19 (incorporeal, a cold touch that frightens), and
  the mummy priest 20 (a caster whose touch carries mummy rot, sometimes something worth taking). The duergar types are of the duergar race. For blood (§8 "Blood";
  presentation only):
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
  the ice into it takes, never the last one), and `damage_type` (the kind of damage the plunge
  does, cold; §8 "Damage types"); still water `{ below = 0, plunge = "1d3", damage_type = "cold"
  }`,
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
- **Stealing** (`data/steal.toml`, optional; without it nobody steals; §8 "Stealing"): `caught`,
  the faction a creature that catches the player stealing joins (an id in `data/factions.toml`,
  hostile to the player: `outraged`). Who can steal is the skills' business (the `steal`
  benefit, `data/skills.toml`); a shopkeeper that catches the player is outraged by its shop's
  rules (`data/shops.toml`'s `outraged`).
- **Stat training** (`data/stats.toml`, optional; without it nothing trains and no level checks
  a stat; §8 "Stats"): `[training]` `need_base` (20) and `need_per_point` (3), each 0 to 200
  and not both 0, `over_potential_divisor` (4; 1 to 20), and `[[training.use]]` rules: `stat`
  (a stat id), `on` (a use: `burdened`, `blow`, `dig`, `kick`, `dodge`, `shot`, `sneak`,
  `climb`, `hurt`, `big_hit`, `resisted`, `plunge`, `found`, `read`, `identified`,
  `discovered`, `spot`, `endured`, `resisted_will`, `hungry_fight`, `cast`, `chat`, `trade`, `quest`,
  `swapped`), and `chance` (1 to 100); a stat and use at most once.
- **Action costs** (`data/actions.toml`, optional): action ids mapped to their cost in energy,
  e.g. `wait = 1000`, 1000 being a normal action (6 s at normal speed; §8 "Time"), 100 to
  10 000. The ids are `step` (onto terrain of `move_cost` 100), `wait`, `open_door`,
  `close_door`, `unlock` (unlocking a door with a key, on top of opening it), `pick_lock` (2000:
  a try at picking a lock, whatever comes of it; a lock that gives opens its door on top of it,
  `open_door`; §8 "Doors"), `kick`, `lantern`,
  `hood` (500: lowering or raising the lantern's hood; lighting it hooded costs `lantern` too),
  `stairs` (also entering a location or local area and walking off an
  area's edge), `pick_up`, `drop`, `melee` (a shield bash too), `shoot`, `throw`, `reload` (a
  missile weapon that `reloads`: a crossbow shot costs `shoot` plus `reload`, 2000), `cast`,
  `dig` (a turn of digging), and for equipment (each slot names
  one, below) `wield` (1000), `wear` (1000: helmets, cloaks, gloves, boots, rings, amulets),
  `body_armor` (3000: it takes a while to strap on), `ready` (1000: a missile weapon, missiles,
  a tool), `eat` (1000: a turn of eating; a meal takes the food's `turns`, §8 "Hunger"), `chat`
  (500: a few words, §8 "Dialogue"), `steal` (1000: trying to steal, whatever comes of it, §8
  "Stealing"), `throw_voice` (1000: throwing one's voice, §5 "Hearing"), and `climb` (2000:
  climbing out of the water onto ice, on
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
  symbol it renders with and its color, item class, materials, and weight: `id`, `name` (singular,
  without an article; `plural` defaults by the plural dictionary, mostly the name plus `s`), `glyph`, `fg`, `class`,
  `materials`, `weight` (integer grams, at least 1), `value` (what one is worth in gold coins,
  required, 0 to 1 000 000; shops price their goods from it, §7 "Shops"), and `stackable`
  (default false).
  **Parts** (#199): `materials` is what an object is made of, an ordered list of parts, each of
  one material (an id in `data/materials.toml`, likewise a directory, whose `[[material]]`
  entries have an `id`, a `name`, `precious` (a gem or a precious metal, default false), and
  `value` (percent of steel's, 1 to 10 000, default 100); the properties below, under
  "Materials"). A part is
  a table, `{ part = "setting", of = "ruby", share = 10 }`: `part` an id in `data/parts.toml` (a
  small vocabulary, `[[part]]` with `id`, `name`, and an optional `adjective`: band, blade, chain,
  cord, grip, haft, head, inlay, lining, pendant, pommel, setting, studs, tip, trim), `of` its material,
  `share` its percent of the weight. A plain material id is one unnamed part, the shorthand:
  `materials = ["steel"]`, what most objects are. **The first part is the main one**: it takes
  the share the others leave (at least 1%; it gives none), may be unnamed, and decides what a
  single material decides (`ObjectDef::material`: the material named first, the material word a
  name begins with, and, for an item of another material, its colour and its weapon and armour
  modifiers, §7 "Materials"). Every other part names its part and share; a part
  name appears once; the old `material` key is a content error that says to write `materials`.
  **Value by shares**: an object made of other parts than its own is worth its `value` × Σ(share
  × material value) of those parts without a gem / the same sum of its own (`ObjectDef::value_with`,
  rounded to the nearest coin), plus what each gem part is worth by its size and cut (below), so
  made of its own parts it is worth its `value`. An item may carry **parts of its own** (gems,
  below; other materials, "Materials"): `Stack::parts` is its base item's parts with its own in
  their place, each an instance part (a part, a material, and a gem's size and cut) saved by id
  with the stack beside its magic. **Names** show the notable parts, gems and
  precious metals (`precious` materials), that the written name does not mention as a word: a
  notable part whose part has an `adjective` goes before the name ("ruby-pommelled long sword"),
  any other joins the main part's material if the name begins with it ("gold amulet" with a ruby
  setting is the "gold and ruby amulet"; "iron and ruby ring"), else goes before the name; the
  name is composed when content loads (`part::names`), so every message, list, and screen says
  it; an item with parts of its own is named from its written name and those parts
  (`Stack::names`). The plain parts (a leather grip) show in the item's description, which lists every part
  ("steel blade, leather grip"). Materials are visible, not hidden: an unidentified amulet shows
  that it is gold and ruby (its magic stays hidden). The showcase's examples: the gold amulet
  (gold chain, ruby setting), the amber amulet (amber pendant, leather cord), and the long sword
  (steel blade, leather grip). Glyphs follow ADOM loosely: `)` weapons, `}` missile weapons, `/` missiles, `[`
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
- **Gems** (#191): ten gems are materials of their own (`data/materials/diamond.toml`, ...: a
  precious material with a `gem` table: `color`, `value` in gold for a gem of the plain size and
  cut, `rarity`, a weight among the gems in loot, and an optional `plural`): diamond (400, very
  rare), ruby and emerald (300, rare), sapphire (280, rare), topaz (120), heliodor (110), and
  amethyst (80, uncommon), zircon (70), garnet (60), and smokestone (30, common); amber stays a
  precious material, no gem. `data/gems.toml` holds what they share: `object` (the loose gem,
  `objects/gem.toml`, `*`, item class gems), `chance` (percent of finds that are a loose gem),
  the ordered `[[size]]`s (`value` multiplier in percent, `weight` in grams, loot weights `find`
  at item level 1 plus `deeper` per item level): tiny ×0.3, small ×1, medium ×4, large ×15, huge
  ×50, weighing 1 to 4 g; the ordered `[[cut]]`s (`value`, `find`, `deeper`): uncut ×0.25,
  crude ×0.6, fair ×1, fine ×1.6, exquisite ×2.5, flawless ×4; exactly one size and one cut is
  `plain` (small and fair, ×1 both: what names leave unsaid and what a placed gem is unless it
  says otherwise), and a cut may be `rough` (uncut: named before the size, drawn at half its
  colour's saturation, never set in an item); and the `[[ornament]]`s, one per item class: the
  `part` a gem becomes in its items (melee weapons a pommel, missile weapons and shields an
  inlay, helmets studs, rings and amulets a setting), its `chance` in percent, and `colour` (the
  item takes its gem's colour: rings). An object may name its own `ornament` part (staves a tip;
  clubs and polearms an inlay); a gem never becomes an item's main part. A gem is worth its
  value × its size's × its cut's multiplier (`Gems::value`, rounded, at least 1). **A loose
  gem** is the gems' object made of its gem as its main part, with a size and cut: it is named
  "[uncut] [size] [cut] gem", the plain size and cut unsaid ("a ruby", "a large fine emerald",
  "an uncut tiny garnet", "3 medium rubies"), drawn in its gem's colour, weighs its size's
  weight, and is worth its gem's worth; it stacks only with identical gems. **An ornament** is a
  gem part of any other item, in place of its part of that name or besides its parts (weighing
  nothing beside them): the item's name shows it as its part does ("an emerald-pommelled long
  sword", "a gold and sapphire amulet", "a topaz-tipped quarterstaff", "a garnet-studded iron
  helm"), the item's description lists it with its size and cut ("Pommel: small emerald, fine
  cut"), it adds its gem's worth to the item's, and it has no rules effect yet. Like every
  material, gems show at once, identified or not. Loot rolls them (§8 "Magic items"), general
  stores buy them (§7 "Shops"), and level files place them (§7 "Levels").
- **Materials** (#193, #194): a material's properties are on one scale, steel's (all optional,
  so a material without them is as it was): `group` (one word, not `any`: metal, wood, bone,
  stone, hide, cloth, glass, resin; none for paper, wax, food, and the gems, which are set in
  items but never made into them), `color` (an item of it drawn so where its base item is of
  another material), `weight` (percent of steel's for an item of it, 1 to 1000; metals by real
  density, the rest an effective weight), `to_hit` and `damage` (as a weapon's working part, ±10;
  neither: it cannot be one), `protection` (percent of an armour's Protection kept, 0 to 500;
  none: it cannot be armour), `value` (above), `rarity` and `gear` (how often loot, by item
  level, and creatures' gear, by creature level, make an item of it: `{ weight, deeper, level }`,
  the weight at level 1, `deeper` per level past the first, never below 0, none below `level`).
  The seventeen metals (#193; no lerasium, atium, or malatium), weight / to hit / damage / armour
  / value: steel 100 / 0 / 0 / 100% / ×1, iron 100 / −1 / 0 / 90% / ×0.6, tin 93 / 0 / −2 / 40%
  / ×0.4, pewter 93 / 0 / −2 / 40% / ×0.5, bronze 112 / 0 / −1 / 80% / ×0.8, copper 114 / 0 / −1
  / 60% / ×0.7, zinc 91 / 0 / −2 / 50% / ×0.6, brass 108 / 0 / −1 / 70% / ×0.9, gold 246 / −2 /
  +1 / 50% / ×20, electrum 191 / −1 / 0 / 60% / ×12, cadmium 110 / 0 / −2 / 50% / ×4, bendalloy
  118 / 0 / −1 / 70% / ×6, aluminium 34 / +1 / −2 / 70% / ×8, duralumin 36 / +1 / −1 / 90% /
  ×10, chromium 92 / 0 / +1 / 110% / ×9, nicrosil 108 / 0 / +1 / 110% / ×10, silver 134 / −1 /
  −1 / 50% / ×15; the organic ones (#194): wood 40 / −1 / −2 / 30% / ×0.2, hardwood 50 / 0 / −1 /
  40% / ×0.4, ironwood 70 / 0 / 0 / 60% / ×3, bone 60 / 0 / −1 / 50% / ×0.3, horn 50 / 0 / −1 /
  45% / ×0.5, ivory 65 / −1 / −1 / 40% / ×5, chitin 45 / 0 / −1 / 60% / ×0.6, dragonbone 55 / +1
  / +1 / 110% / ×25 (bone group), flint 80 / 0 / −1 / − / ×0.1, obsidian 75 / 0 / 0 / − / ×0.3
  (stone), leather 35 / − / 30% / ×0.5, hardened leather 45 / − / 45% / ×0.8, fur 40 / − / 25% /
  ×0.8, dragonscale 40 / − / 130% / ×30 (hide), cloth 25 / − / 15% / ×0.3, silk 18 / − / 20% /
  ×4, and wool 30 / − / 20% / ×0.4 (cloth; the wool cloak's, so its variants name well); jade
  (stone) and amber (resin) are jewellery materials only (×4, ×3); glass, stone, paper, wax,
  flesh, and plant matter have colours only. Found (loot at item level 1, weights): copper and
  bronze very common (800, fewer deeper), iron and tin common (300), steel common (300) and +60 a
  level, the usual metal deeper; pewter, zinc, brass, horn, chitin, obsidian uncommon (100);
  ironwood, ivory, silk, jade, amber rare (40); gold, silver, electrum, cadmium, bendalloy,
  aluminium, duralumin very rare (10); chromium and nicrosil very rare from item level 8,
  dragonbone and dragonscale from 10. Creatures' gear leans primitive: steel 20 at creature
  level 1 (+15 a level), bone and flint 600, copper and bronze 600 (fewer with level).
  **Construction lists**: each part may say what it may be made of, `made_of` in its table (the
  main part's also as the object's own `made_of`, the shorthand), naming groups, materials, or
  `any` (every material of a group: jewellery, so never paper or wax); it must allow the part's
  own material; the main part of a weapon or ammunition keeps only materials that can be a
  weapon's working part, of armour or a shield only those that can be armour (a material named
  by itself that cannot is an error, as is a gem). Without a list a part is only its own
  material. The built-in lists: swords metal and dragonbone; daggers metal, bone, stone; axes
  metal, stone; spears, javelins, and arrow and bolt heads (their main part, an iron head on a
  wooden haft or shaft) metal, bone, stone; polearms, the trident, maces, and hammers metal;
  clubs wood, bone; staves wood, dragonbone; bows wood, horn, dragonbone; crossbows wood; padded
  armour cloth; leather armour hide; scale mail metal, bone, dragonscale; chain and banded mail
  metal; plate metal, chitin, dragonbone; shields wood, metal, hide, chitin, dragonscale; helmets
  metal, hide, bone; gloves and boots hide (gauntlets and iron-shod boots metal); cloaks cloth,
  fur, leather; pick-axe and mattock metal, stone; rings and amulets any. Rocks, food, potions,
  scrolls, coins, gems, keys, the sling, and flaming arrows have none. **Variants**: an item made
  of another material (an instance part, `ItemPart` with no gem in place of a base part, by id;
  only from its part's list and never what that part is made of anyway, checked for saves, level
  files, and placements) weighs its base weight with each part's share scaled by its material's
  weight over the base part's (`variant::weight`, rounded, at least 1); strikes with to hit and
  damage + the difference of the main parts' weapon modifiers (both must have them; through
  `Mods::local`, so melee, the off hand, launchers, ammunition, and throws); keeps Protection ×
  its main material's protection / the base's, rounded to the nearest (`Stack::base_protection`);
  is worth what its parts are, by shares (above); and is drawn in its main material's colour (a
  ring's gem colour first). Jewellery changes only in worth and colour. Its name gives the
  material: in place of the base material's word if the written name says it ("iron helm":
  "bone helm", "pair of iron-shod boots": "pair of bronze-shod boots", "gold ring": "silver
  ring"), else before the name ("bronze long sword"), after any ornament ("emerald-pommelled
  bronze long sword"). The description shows its parts, weight, damage dice with the material's
  bonus ("2d6-1"), and Protection; the to-hit stays hidden. **Where variants come from**: loot
  draws the material last (§8 "Magic items"; the item's main part from its list by `rarity` at
  the item level), creature drops by `gear` at the creature's level; a material worse in a fight
  than the base item's own (to hit or damage lower on what strikes or is shot, Protection lower
  on armour with some) but dearer weighs at most 10 (`VERY_RARE`) for that item, never for
  jewellery. Hand-made levels keep their legends' gear unless a placement names a material
  (`"great_sword:bronze"`, or `[[item]]` `parts = [{ of = "bronze" }]`); shop stock is plain. The
  showcase's storeroom has a bronze great sword. Not yet: rust and wear.
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
  (`spell`, an effect of its shape in `data/effects.toml`; `noun`, `verb` (default: the kind's),
  `damage`, `damage_type` (one kind, §8 "Damage types": fire, electricity, cold); a burst's
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
  and `shield` (while held), each 0 to 20; `protection` (damage soaked off every hit while worn,
  0 to 20, adding up across what is worn; only on armour worn, never in a hand or on a shield);
  `penetration` (how much of a target's Protection it ignores, 0 to 20; weapons and
  ammunition); `bash` (a shield's damage dice when bashing with it,
  only with `shield`; §8 "Weapon skills"); and `category`, the weapon skill a weapon draws on and
  trains (an id in `data/weapon_skills.toml`; required for every weapon when there are weapon
  skills, refused on anything else): a `melee` skill for what goes in a hand, a `launcher` one
  for a missile weapon, the `thrown` one for what is only thrown (a rock). Ranged combat (§8 "Ranged combat"): `range` (1 to
  40 cells; required for a missile weapon: how far it shoots; on anything else it makes a
  throwing weapon, thrown that far with its own `damage`, which then needs no `slot`: a rock;
  ammunition has none), `reloads` (missile weapons: each shot also takes the `reload` action);
  damage types (§8 "Damage types"): `damage_type`, the kind a weapon does (an id in
  `data/damage_types.toml`, or two of which a blow does the better: `["slashing", "piercing"]`;
  a launcher's is its shots', a shield's its bash's, which is the unarmed kind without one;
  only on weapons and shields that bash), `verb`, what a blow with a weapon does instead of
  the kind's verb ("cleaves"), `added`, damage of another, non-physical kind every hit with a
  weapon, a missile weapon, or ammunition adds (`{ type = "fire", damage = "1d4" }`: the
  flaming arrow), `resist`, what a worn or held thing resists in percent per kind (`{ cold = 20
  }`, −100 to 75; the fur cloak cold 20, the wool cloak and padded coat cold 10, the leather
  jerkins acid 10), and `protects`, how armour's Protection stands against each physical kind,
  more or less than its `protection` (`{ slashing = 1, piercing = -1 }`, within ±20; chain
  mail, scale mail slashing +1 and bludgeoning −1, banded mail slashing +1; only with
  `protection`);
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
  `attack_speed`, `range`, `draw`, `durability`, `parry`, `armour`, `protection`, `shield`,
  `max_dex`,
  `weight`, `dodge`, `deflection`, `natural`, `max_hp`, `regeneration`, `max_mana`,
  `mana_regeneration`, a stat id, `move_speed`,
  `carry`, `stealth`, `light`, `skill` with a `skill`, an id in `data/skills.toml`, whose value
  it raises; without skills content, skill affixes are left out; `elemental` (adds 1dN damage of
  the affix's kind to every roll) and `resistance` (+N % resistance to it) with a
  `damage_type`, an id in `data/damage_types.toml`; without damage types they are left out; §8
  "Damage types"), `bases` (kinds of item it
  rolls on, at least one; an effect only
  attacks can use, `extra_damage`, `elemental`, `range`, `draw`, `durability`, needs weapons,
  missile weapons, or ammunition, `parry` weapons, `armour` body armour, `protection` armour (body armour,
  helmets, cloaks, gloves, boots), `shield` shields, `max_dex` body armour or shields; else a
  content error), `weight` (default 100), and `tiers`, at least one, item levels
  rising: `{ level, min, max }` (values within the effect's range: 1 to 10 for most, dice sides 2
  to 20 for `extra_damage` and `elemental`, 1 to 3 for `threat` and `crit`, percent 1 to 50 for
  speeds, 1 to 90
  for `durability`, `weight`, `stealth`, 1 to 100 for `max_hp`, `max_mana`, `carry` (kg), and `skill`, 1 to
  300 for `regeneration` and `mana_regeneration`, 1 to 75 for `resistance`), or for `light` `{ level, light }` naming a
  light preset. The built-in
  affixes: 25 prefixes and 29 suffixes in up to four tiers (item levels 1, 3, 5, 7 for most); the wiki
  lists them in generated tables. The light presets `item_glow_faint`, `item_glow`, and
  `item_glow_bright` (radius 2, 3, 4; steady, cool) are the glowing tiers'.
- **Talents** (`data/talents.toml`, the settings; `data/talents/`, one talent a file, read in
  path order; optional, without them no level asks for a pick; M30, #229; §8 "Talents"):
  `max_ranks` (3), `[coverage]` (`min_talents`, `min_class`, `free_per_tree`), `[[tree]]`s
  (`id`, `name`, `description`, `pending`, `no_free`) in display order, and `[[system]]`s (`id`,
  `name`, `pending`), the closed list a talent's `systems` may name. A `[[talent]]`: `id`,
  `name`, `tree`, `description`, `ranks`, `choice` (`weapon_skill`, `style`, `element`), `needs`
  (`talents = { dodge = 1 }`, `stats = { toughness = 14 }`, `skills = { stealth = 25 }`,
  `weapon_skills = { bows = 3 }`, `chosen`, `level`, `race`, `class`, `eyes`,
  `requires_alignment = { min = "good" }`), `effects`
  (`{ kind = "dodge", value = 1 }`: `dodge`, `natural`, `melee_damage`, `max_hp_per_levels
  { per_levels, at_least }`, `healing_percent`, `footsteps_percent`, `silent_doors`, `hearing`,
  `carry_percent`, `travel_percent`, `hunger_percent`, `no_exertion`, `xp_percent`, `notice`,
  `free_blow { trigger }`; bonuses −20 to 20, percents −100 to 100, never 0), `[[talent.rank]]`
  (a rank's own `needs` and `effects`, as many as `ranks` or none), `systems` (at least one),
  `granted`. Talents are saved by id (not in the content fingerprint); without skills content a
  skill need is left out, as skill affixes are. Load fails on a coverage gap (§8 "Talents").
- **Damage types** (`data/damage_types.toml`, optional; without it nothing is typed, nobody
  resists anything, and no data may name a kind; #151, §8 "Damage types"): an ordered list of
  `[[type]]`s, each with `id`, `name`, `physical` (Protection soaks it; default false), `verb`
  and `you` (what a blow of it does, in the third and the second person: "slashes", "slash"),
  and `unarmed` (bare hands, kicks, and shield bashes do this kind; one at most). Ids are
  indices (in the content fingerprint), so a new type is a new entry at the end of the list, at
  most 32. Built in: slashing, piercing, bludgeoning (`physical`, `unarmed`), fire, cold,
  electricity, acid, poison, force, necrotic (the last two the spells', §8 "Spells"). The status effects (`data/status.toml`, §8 "Status effects") name
  the kinds that wound (`wound.from = ["slashing", "piercing"]`: bleeding; none named: any) and
  the kind a status is (`damage_type = "poison"`: poisoned).
- **Resources** (`data/resources.toml`, optional; without it there are none; #225, §8
  "Resources"): an ordered list of `[[resource]]`s, each with `id`, `name` (the character
  screen's), `short` (the status bar's label, at most 12 characters), `max` (1 to 1000), `start`
  (what it holds when granted; default `max`; at most `max`, above 0 for one that ends at zero),
  `show_max` (the status bar shows `Focus 7/10` rather than `Rage 7`; default true),
  `regenerates` or `decays` (a point back, or lost, every that many seconds of game time, 1 to
  86400; at most one of the two), `resting` (it changes over time while the player rests;
  default true), `ends_at_zero` (default false), `ended` (what the player is told when it ends;
  needs `ends_at_zero`), `builds` (`[{ on = "kill", amount = 5 }]`, each event once, amounts
  1 to 1000; events `kill`, `hit`, `critical`, `struck`, `sneak_attack`), and `status` (a status
  id: its holder suffers it while they hold the resource; it must not `eases`; Rage brings
  Raging). A status that something `follows` into and that neither eases nor is cured must be
  some resource's status. Ids are indices (in
  the content fingerprint), so a new resource is a new entry. Built in, the progression design's
  first guesses (§4.18): Rage, Focus, Openings, Ki, Inspiration. A class active (a talent of the
  class tree with an active effect) may cost one: `resource = { id = "focus", cost = 6 }` (0
  asks only that it is held; at most its `max`).
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
  with `id`, `name`, `hostile` (faction ids), and `animal` (default false; the vermin and the
  wildlife are animals: a Beast-friend player, §8 "Talents", stands neutral with their creatures
  until they attack one of a type, `World::stance` and the AI alike). Relations are pairwise and
  symmetric: if A lists B, A and B are hostile to each other; every pair not listed (a faction
  and itself too) is neutral. Validation: unique ids, known ids, no faction hostile to itself, a `player` faction
  (the player's), and every creature type but the player naming a known faction. The core API
  gives the stance (`Hostile` or `Neutral`) of two factions, creature types, or actors. Test
  factions: `vermin` (giant rats; hostile to the player), `kobolds` (the war band; hostile to the
  player and the vermin), `goblins` (the barrows' goblin tribes; hostile to the player, the
  vermin, and the kobolds), `undead` (the barrows' and the fen's dead; hostile to the player,
  the kobolds, and the goblins, and leaving the vermin alone), `marsh` ("marsh things", the
  fen's living things and the bog hag; hostile to the player; not animals as a faction), `cult`
  ("cultists", the drowned minster's cult; hostile to the player and the marsh), `duergar` (the
  delf's grey dwarves; hostile to the player, the goblins (the ogres and trolls among them), and
  the undead), `fire` ("fire things", the delf's burning creatures and the fire drake; hostile to
  the player only), `wildlife` (deer, cave bats, the cave beetle;
  neutral to everyone), and
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
  when set). A legend entry with a `creature` may also name an `alignment` (`good`, `neutral`,
  `evil`) that replaces its type's for that creature (`Actor::alignment_override`, saved and
  hashed only when set; §8 "Alignment"). Level files may also have an `arrival` line, said each
  time the player arrives by stairs.
- **Dialogue** (`data/dialogue.toml`, optional; without it nobody talks; §8 "Dialogue"): each
  `[[dialogue]]` names exactly one of `creature` (a creature type, not the player) or `npc` (an
  npc id: lower-case letters, digits, and underscores; an npc's may give its proper `name`, 1 to
  24 characters, "Sister Hild": messages, look mode, and the dialogue screen call it so, without
  an article, and quest texts name it so as a giver), at most one dialogue each, and has `lines`
  (at least one one-liner), an optional `greet`, optional cold greetings (`greet_evil`, lines a
  good speaker says to an Evil or Wicked character instead of its greeting and one-liners, in
  turn; `greet_good`, an evil speaker's to a Good or Saintly character; §8 "Alignment"), optional
  `[[dialogue.topic]]`s (at most 9, each
  with a unique `id` in lower-case letters, digits, and underscores, a `name`, and a `reply`), and
  an optional `mutter = { chance, verb, lines }` (`chance` in percent per action, 1 to 100; `verb`
  one lower-case word, default "mutters"; at least one line). Lines, greetings, topic names, and
  muttered lines are one line of at most 72 characters (they fit the message line with the
  speaker's name), replies at most 280 (the dialogue screen wraps them); none is empty or padded.
  Built in: the gardener (one-liners; grumbles), the wandering bard (one-liners and topics: the
  moon, the Great Race, this year, the Wildlands; sings), and the old hermit (a greeting,
  one-liners, and topics: this hall, the dark chamber, the gallery, the water garden, these
  stairs; mutters). The gardener, the bard, the hermit, and the shopkeepers have two cold
  lines each for an evil character ("Prices are higher for your sort. Take it or leave it.").
- **Quests** (`data/quests.toml`, optional; without it nobody gives quests; §8 "Quests"): each
  `[[quest]]` has a unique `id` (lower-case letters, digits, and underscores), a `name` (the log's
  title and the topic, at most 32 characters), a `giver` (a creature type with a dialogue, or an
  npc id with one), a `type` and a `target`: `kill` (a creature type, not the player, `count` 1 to
  99, default 1; or a creature tag, `tag:goblin`, `count` 1 to 999, refused when no creature
  type a run of the quest's campaign can meet carries it; optional `level`, a level id or a
  dungeon branch id for any of its levels; optional `what`, 1 to 24 characters, what the quest
  calls them instead of the type's name or the tag's plural),
  `bring` (an object type; `count`; optional `to`, someone else with a dialogue who takes it),
  `reach` (a level id, a branch level such as `depths:3` included; never `repeatable`), or
  `deliver` (an object type the giver hands over; `to` required, not the giver); a field another
  type has is a content error. `reward = { gold, items, xp }` (each optional: gold coins, at most
  9999, which needs the `gold_coin` object; objects as levels place them; experience, at most
  1 000 000). Texts, one line each of at most 280 characters, neither empty nor padded: `offer`,
  `accept`, optional `decline` (default "Another time, then."), `reminder` (the giver's), and
  `done` (said by whoever it is handed in to). Optional `requires` (a quest loaded before this
  one, or a list of them, `["foremans_ledger", "barrow_heart"]`, all done before this one is
  offered), `requires_alignment` (the bands it is offered to, §8
  "Alignment": `{ min = "kind" }` Kind or better, `{ max = "callous" }` Callous or worse, or
  both; never neither, nor a `min` above the `max`), `alignment` (how far completing it moves the
  player's alignment, −1000 to 1000, default 0), `repeatable` (default false), `reveals = {
  location = "ID", when = "accepted" }` (a hidden location of the overworld it reveals, §7
  "Hidden locations", as it is `offered`, `accepted` (the default), or `done`; several quests
  may reveal the same one, the first to get there does), and `where` (where its goal lies, in
  words, after the objective in the quest log; one line of at most 280 characters). Everyone's topics must fit
  the dialogue screen's nine: their dialogue's own, one per quest they give or take in, and one
  more for an offer's accept and decline. Built in: the old hermit's "Rats in the arena" (kill 3
  giant rats in the arena; 20 gold, 30 XP) and, after it, "Three levels down" (reach the Depths 3;
  50 gold, a jade amulet, 100 XP), the gardener's "Three apples" (bring 3 apples; 10 gold, 15
  XP), and the wandering bard's "The bard's letter" (deliver a sealed letter, a new object, to the
  old hermit; 5 gold, 15 XP). All four help someone, and are a little good (`alignment` 20, 15,
  10, and 10). The Ascmearc's quests are §3b "Quests".
- **Plurals** (`data/plurals.toml`, and a campaign's `data/plurals.toml`; optional; without it
  every plural is the word and an `s`): the plural dictionary, one `singular = "plural"` line
  per word that does not take a plain `s` (`wolf = "wolves"`, `undead = "undead"`,
  `duergar = "duergar"`), kept by hand: a new tag or name with an irregular plural adds its
  line. A word is looked up whole, then by its last word (after a space or a hyphen: "skeletal
  wolf" is "skeletal wolves"), else it takes an `s`. It makes every plural content leaves out: a
  creature's, an object's, or a gem's name without a `plural`, and a creature tag's (§8
  "Quests"). A word given two different plurals (in two files) is a content error.
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
