# What's in the game

An overview of what the game does today, feature by feature; the other pages explain the
rules behind each.

This is the engine and framework phase: creatures hunt you, fight, shoot, and die (spells are only
announced so far), death is permanent, and you grow by experience levels and by skills that
improve with use. You come into **the Ascmearc** through a mountain pass that closes behind you
(see [The Ascmearc](ascmearc.md)): an ADOM-like **overworld** with day and night and weather,
outdoor areas generated from it, the market town of Cēaptūn, and procedurally generated
dungeons under the barrows, the fen, and the hills.

- **Light and vision** — every cell carries RGB + UV light from torches, lamps, crystals, braziers,
  and your lantern, with shadows from walls and pillars, soft shadows from creatures, and a weak
  bounce of light off nearby surfaces. Firelight drifts gently in hue and strength; magical light
  holds steady. What you see depends on your eyes: normal
  vision needs light, low-light eyes see by faint fungus glow, darkvision sees in the dark in
  fading monochrome, and UV sight reveals runes no one else can see.

  ![The darkness chamber under four kinds of vision](images/m3-darkness-chamber-visions.png)

- **Creatures** — every creature acts at its own pace (some twice for each of your turns, some
  less; the game never tells you) and belongs to a faction that is hostile or neutral to you and to
  the others. Hostile creatures see by the same rules of light and eyes as you do, then approach
  and attack, keep their distance and shoot, or cast from afar, entirely deterministically; when
  they lose you, they search where they last saw you, follow noises, and return to their rounds.

- **Hearing** — footsteps, doors, and fights make noise that travels around corners, is muffled by
  closed doors, and is louder in water and rubble than on grass. You hear what you cannot see as
  faint marks on the map and short messages ("You hear footsteps to the north-east."), without
  learning who made them; creatures hear you too and come to look.
- **Perception and memory** — symmetric line of sight; explored terrain and the objects seen on
  it stay as dim memory, never showing creatures or current light.
- **Objects** — weapons, armor, tools, potions, scrolls, food, gems, coins, rings, and amulets,
  each with its own glyph, color, material, and weight, lie in piles to pick up and drop; an
  ADOM-style inventory lettered a–z shows what you carry and what it weighs, and an ADOM-style
  equipment screen puts it on: helm, amulet, armor, cloak, weapons and shield (or a second, light
  weapon, or a two-handed staff or great sword), gloves, rings, boots, a bow and arrows, and a
  tool. Creatures wield and wear what they carry, and you see it.
- **Digging** — walls give way: ready a pick-axe or a mattock in the Tool slot and dig, turn after
  turn, through packed earth in a minute, a stone wall in a few, granite in half an hour or so
  (Strength helps, a heavy load slows you), until a creature comes into view or the wall crumbles
  to rubble; light and sight spill through at once. Digging is loud. A fireball blows the softer
  walls apart.
- **Combat** — D&D-style with two twelve-sided dice: every blow and shot is rolled against
  Defence (Dexterity, armour, shield, parry, a beetle's shell), misses say why ("You block the
  kobold's spear with your shield."), criticals threaten and confirm ("The kobold hits you
  hard!"), armour and hides soak damage like ADOM's PV ("Your sword glances off the cave beetle's
  shell."), and weapons and armour come from data. ADOM-like hit points from Toughness, race, and
  class heal with time; creatures die and drop what they carried, look as hurt as they are, and
  your own death ends the run for good: the save is gone.
- **Experience** — every creature has a level and an XP value; kills you make yourself (melee,
  shots, throws) earn XP, less and less the further the monster is below your level, as in World
  of Warcraft (nothing from grey ones, a little extra from tougher ones). Levels run to 50 and, up
  to level 20, make you hardier. Your level and XP show on the status bar, the character screen,
  and the death screen.
- **Magic items** — after Path of Exile: sixty base items (daggers to great axes, bows and
  crossbows, padded coats to plate, helms, cloaks, gloves, boots, shields, rings, amulets) roll
  prefixes and suffixes from 47 affixes in tiers that deepen with the dungeon. A magic item has one
  or two ("Sharp long sword of the Bear", in blue); a rare one up to six and a name ("Grim Bite,
  long sword", in yellow). Found unidentified, they work all the same; each remembers what it
  rolled. They lie in the dungeons, in vaults, and fall from slain creatures, and an item's
  description says what it does in plain words. A glowing helm lights your way.
- **Gems** — ten gems from smokestone to diamond, each of a size and a cut (uncut included), are
  rare finds, larger and finer deeper down, and what rings, amulets, and now and then weapons,
  shields, and helms are set with: "an emerald-pommelled long sword", "a gold and ruby ring".
  General stores buy them at what they are worth.
- **Materials** — seventeen metals, from tin to nicrosil, and woods, bones, stones, hides, and
  cloths, each with its weight, what it does as a blade or as armour, its worth, and its colour.
  Items are found made of other materials than their usual one, by what each may be made of: "a
  bronze long sword" (heavier, a little weaker, cheaper), "a flint spear", "a dragonbone
  quarterstaff". Copper and bronze are common near the surface, steel deeper; kobolds carry
  bone, flint, and bronze.
- **Scrolls** — read with `r`: scrolls of fireball, lightning, and frost cast the traps' spells
  where you aim them (a fireball read too close asks first), and what they kill is yours; a
  scroll of identify tells you what an unidentified item is. They lie in the dungeons and sell in
  general stores.
- **People and talk** — Cēaptūn has folk to talk to: townsfolk on the square, Sister Hild in the
  Mynster, Godgifu at the inn, the reeve and the mine-factor. Chat with `C`: most say a
  one-liner, a different one each time; some have topics to ask about on a small dialogue
  screen (Godgifu has news of the barrows, the marsh, the mines, the volcano, and the pass), and
  Sister Hild sells a cure and a mending for gold. They mutter to themselves
  now and then, which you may overhear through a wall. Walking into anyone who is not hostile
  swaps places with them, so nobody blocks a corridor (a few will not budge).
- **Quests** — simple, ADOM-like errands from the people you meet, offered through their topics:
  kill something, bring something, go somewhere, or carry something to someone else. Asking
  gives you the quest; follow it in the quest log (`q`), and come back for gold, things, and
  experience.
  Some quests tell you where a hidden place lies and mark it on your map: in Cēaptūn the reeve,
  Sister Hild, and the mine-factor each have a chain of three.
- **Ranged combat** — bows (drawn by Strength), a crossbow, a sling, and throwing weapons; aim
  ADOM-style with a cursor and a faint line of fire, watch the arrow fly, and pick it up where it
  fell. A flaming arrow flies burning and lights the dark where it lands until it burns out:
  shoot one into the dark to see what waits there (and be seen).
- **Blood** — big hits and kills stain the floor and the walls beside the victim, by its size and
  how the blow landed; subtle for one kill, a slaughterhouse where many fell (a setting hides it).
- **Weapon skills** — ADOM-style: every blow, shot, and throw earns marks in its weapon's skill
  (daggers, long blades, axes, bows, …, sixteen in all, shield blocking and shield bashing among
  them), and ranks 0 to 10 come slower and slower; each rank makes you hit, hurt, parry, or
  block a little better ("You feel more skilled with long blades."). Skill carries over to
  related weapons, as in *The Riddle of Steel*: long blades 8 makes short blades 6. Your race and
  class start you off, a weapon skills screen shows your ranks, and a shield can bash too.
- **Fighting styles** — after AD&D's *Complete Fighter's Handbook*, how you hold your weapons is a
  skill of its own: Duelist, Twin Blades, Great Weapon, Sword and Board, Marksman, Skirmisher,
  Brawler, and Staff Fighter, each with ranks 0 to 10 trained by fighting that way and benefits
  that add to your weapon's (a duelist parries, a great weapon hits harder, sword and board may
  follow a hit with a free shield bash). A light weapon in your off hand strikes too, at a
  penalty that Twin Blades trains away. Your class starts you off; the weapon skills screen
  marks the styles you fight in.
- **Status effects and healing** — big hits and criticals open bleeding wounds, a giant rat's
  bite may leave poison behind; they cost hit points as time passes, stack, wear off, slow or
  stop healing, and can kill (creatures bleed to death too); strong poison saps Strength and
  Dexterity. The status bar names what ails you, look mode shows a creature bleeding; never a
  number. Hit points come back twice as fast while you rest.
- **Mana** — a second pool beside your hit points, for the magic to come: from your race and
  class, your Intelligence, Willpower, and Presence, your levels, and magic items. It comes back
  with time by your Willpower, faster while you rest or in a sanctuary, slower while poisoned or
  diseased, not at all while starving. The status bar shows it (`MP 5/12`).
- **Resources** — pools of your own beside hit points and mana, such as a barbarian's rage or a
  ranger's focus: some come back with time, some wear off, some build up from kills and blows,
  abilities spend them, and some end when they run out. You may hold several; the status bar
  shows each one you hold (`Rage 7`, `Focus 7/10`).
- **Diseases** — filth fever from a rat's bite, marsh ague from the swamps, grave rot from
  rotten food, and more, in the spirit of D&D 3.5: caught unseen when a hidden check fails, they
  incubate for days, then you are Diseased. Each day your body fights back with another check;
  a loss takes points off your stats (Toughness damage lowers your hit points), two wins in a
  row shake it off, and grave rot never goes by itself. The character screen names the disease
  once it has run a day; lost stat points mend a point a day once you are well.
- **Traps** — arrow and spear traps, fireballs, lightning bolts, pits, alarms, and teleports lie
  hidden until someone steps on them: you, or a creature you lure there; a trapped door goes off
  when someone opens or kicks it. One you saw go off is known from then on, shown as `^` and
  remembered, and stepping onto it asks first. The dungeons hide a few on every level.
- **Searching** — secret doors look like the wall and hidden traps like the floor until you find
  them: search (`s`, or `S` for a while) with your Intelligence and Search skill, or notice them in
  passing with your Willpower, if you are an elf, a dwarf (in stone), or trained in Spot. Every
  turn is a fresh try. The dungeons hide a door or two on their levels of rooms, never one you
  need.
- **Burden** — everything you carry weighs on you: past a third of what your Strength and
  Toughness can bear you are burdened, then strained, then overloaded. Every action takes longer,
  armour-like caps keep your Dexterity from helping your Defence, and your footsteps get louder.
  Dwarves shrug off the first level. The inventory shows where the thresholds lie.
- **Doors** — some are locked, and look like any other until you try them: a key made for the
  lock, or a skeleton key, opens it. Or kick it in (`k`): the stronger you are, the sooner it
  breaks, and every kick bangs through the halls. Kobolds open doors, but a locked one only with
  its key; the dungeon's locked rooms always leave their key within reach.
- **Hunger** — ADOM-style: time empties your stomach (resting and long journeys too), faster for
  a halfling than an elf. Hungry, then weak, fainting, and starving to death; eat too much and
  you are bloated and slow, or vomit. Eat rations, bread, and apples before they spoil, or the
  corpses creatures leave behind: they weigh what the creature did, rot within a day or two, and
  some do more than feed you (a kobold tastes terrible; a beetle might toughen you). Rotten food
  makes you sick. Cēaptūn's provisioner sells food.
- **Stats** — Strength, Dexterity, Toughness, Intelligence, Willpower, and Presence, inspired by
  D&D, for you and every creature; a character screen shows your own. Strength, Dexterity, and
  Toughness count in a fight, and Strength kicks doors in; the rest do nothing yet.
- **Character creation** — ADOM-style, at the start of every run: thirteen races (elf, half-elf,
  human, dwarf, halfling, gnome, half-orc, and the evil goblin, drow, kobold, lizardfolk,
  duergar, and ratfolk), each with its own eyes, three genders, and fifteen
  classes from barbarian and wizard to merchant and commoner shape your starting stats and
  skills, with dice rolled in the background (a warrior's Strength is 1d3+1). Your age and birthday are
  random, and the year you were born gives you a horoscope sign: one of twelve mythological
  animals and five elements, sixty signs in a great cycle (the Fire Manticore, the Water
  Basilisk). Not balanced, on purpose: some characters are a challenge.
- **Skills** — 25 ADOM-like skills, from stealth and tumble to open lock and spellcraft. Your race
  and class train you in a few; the rest you simply do not have. Skills grow with use (each
  meaningful use may earn a training mark; past 100 they grow slowly), and the passive ones help
  already: tumbling adds to your Defence, stealth quiets your steps and doors, listening lets you
  hear fainter sounds, balance eases a heavy load, and survival speeds your journeys. A Skills
  screen (`A`) shows their values, never the numbers behind them.
- **Abilities** — unlike ADOM's key for each, one menu (`a`) holds everything you can do
  because of who you are: a shield bash, the skills you use as actions, and later racial, class,
  and item abilities. What you cannot use right now is greyed out with why, and `X` repeats the
  last one.
- **Living glyphs and effects** — still ponds that glint in the torchlight, streams and a river
  that visibly flow and foam against rocks, a pulsing altar, flickering torches, sparks and
  embers, a soft glow, heat haze over braziers, short fades. All of it is tunable per effect, and
  a reduced-effects preset turns it off.

  ![The effects room](images/m4-effects-room.png)

- **Sunlight** — a game clock moves the sun across the sky of outdoor levels: warm and low at dawn
  and dusk, white at noon, with a dim blue sky light in the shade and shadows from trees, walls,
  and stones that turn and stretch with it.

  ![A walled garden at 07:00, 12:00, 17:30, and 22:00](images/m9-courtyard-day.png)

- **Moonlight** — by night a faint, slightly purple moon lights the land, waxing and waning with
  the 30-day month and rising about 50 minutes later each day: under a full moon you dimly make out
  open ground (not what lies in its shadows), low-light eyes see by a half moon, and a new moon
  leaves you to your lantern. Moons cast shadows like the sun, widen your sight on the overworld,
  and are data: add more, bigger or faster ones. The status bar shows the date and the phase.

  ![A walled garden at midnight and 04:30 under the full moon, by a first-quarter moon with low-light eyes, and at a new moon](images/m12-courtyard-moon.png)

- **Weather** — clear skies, overcast, rain, and heavy rain come and go with the season (Rainmoon is
  wet, Highsun dry). Clouds dim the sun and the moon and flatten shadows, for you and every
  creature alike; rain shortens your sight on the overworld, falls as faint drops, and rings the
  ponds. The status bar shows the weather when you are outside.

  ![A walled garden in rain at noon](images/m12-courtyard-rain.png)

- **Temperature and ice** — every place has a temperature, shown in the status bar (°C or °F):
  cool and steady underground, outdoors following the month, the hour, and the weather (clear
  nights are cold). Water freezes slowly in the cold and thaws in the warmth, even while you are
  away: thin ice with hairline cracks over dark water, then a grey-blue sheet, then thick white
  ice. Ice holds by its thickness against your weight and everything you carry; a laden dwarf
  breaks through where a light elf crosses, plunges into the freezing water, and climbs out in a
  few turns, while what lay on the ice sinks to the bottom.

  ![A frozen pool: thick ice, ice, and thin ice with a fresh hole, at -2 °C](images/m26-frost-grotto.png)

- **Overworld and time** — a 12-month, 30-day calendar and a clock that every action advances.
  The Ascmearc's overworld takes real travel time per step (a road is quick, a swamp slow,
  mountains impassable); how far you see follows the daylight and the moon. Its locations lead to
  the town and the dungeons (two of them hidden until a quest tells you where), and any other
  cell opens a generated outdoor area — meadows, forests, hills, swamps, riverbanks — that you
  leave by walking off its edge.

  ![An overworld at noon](images/m9-overworld-noon.png)

  ![A forest clearing at noon](images/m9-forest-clearing.png)

- **Levels** — the overworld and the market town, made by hand, and the dungeons:
  rooms-and-corridors and cave levels generated for each run, with hand-made vaults stamped in,
  where creatures of the floor's danger keep coming (see [Dungeons and danger](dungeons.md)).
- **Special rooms** — ADOM-like, some dungeon levels have a room that changes
  the rules inside it, announced the first time you step in: a shadowed room that drains every
  light, an echoing hall that makes every sound loud, a hushed chamber for sneaking, a softly
  glowing sanctuary where wounds heal fast and nothing hostile follows you in (Cēaptūn's
  Mynster is one), and a flooded room of splashing shallow water.
- **Shops** — ADOM-like general stores and specialty shops (a weaponsmith, an armourer, a
  provisioner, a fletcher), each kept by a named shopkeeper by the door: goods show their price
  in gold, what you pick up you owe until you pay (`p`, or when you leave), the keeper makes an
  offer for what you drop, and your Presence and the Appraise skill get you better prices.
  The keeper won't let you out of the door owing money; leave another way (dig through the wall)
  and the shopkeeper, tough and well armed, comes after you. Cēaptūn has four shops on its
  square, and now and then a dungeon level has one.

  ![A generated dungeon level](images/m6-depths-rooms.png)

- **Saves** — one save per run, written when you quit and autosaved as you play (in the browser
  too); when you die, it is deleted.
- **Settings** — font size, color palettes (including high contrast and a colorblind-friendly
  variant), per-effect switches, °C or °F, and fully rebindable keys.
- **Everywhere** — Windows, macOS, Linux, and the browser (WebGPU).
