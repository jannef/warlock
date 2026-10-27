# Dungeons and danger

A dungeon is a stack of floors made the first time you set foot on each, then kept as you left
them. Every place has a **danger level**, a number the game never shows you. On a dungeon floor
it is the dungeon's base danger plus a step for every floor down; a hand-made place has one of its
own; the overworld and the land you explore from it have none. A place could also grow more
dangerous as you do (a puppy cave of the old games), but no place does yet.

## Who lives there, and how many

Each dungeon names the creatures that live in it. Every creature's level is its danger: on a
floor of danger 6, the creatures you meet are mostly of level 5 and 6, some a little weaker, a
few weaker still, and now and then one from deeper down. Each time a creature is chosen, every
kind that lives there gets a weight by how far its level is below the floor's danger, and the
choice goes by weight (kinds of the same weight are equally likely):

{{#include generated/danger-weights.md}}

Some kinds are rarer than others: the more dangerous and the more rewarding (casters,
chieftains, the strongest of a place), the fewer. A kind's weight above is multiplied by its
rarity's share, so a rare kind at the floor's danger is chosen a quarter as often as a common
one of the same level. Creatures placed by hand (people, shopkeepers, a place's own guardians)
are never chosen this way.

{{#include generated/creature-rarities.md}}

A dungeon is mostly its own folk, but now and then something wanders in from elsewhere: one
draw in twenty (unless a place says otherwise) is a **stray**, chosen the same way but from the
**generic list**, every kind that lives in any dungeon of the land (a creature marked "once a
run" is never on it, nor are people, shopkeepers, and the guardians placed by hand). A stray may
be one of the place's own kinds too.

Another draw in twenty (again unless a place says otherwise) is made **out of depth**, as if the
floor were one danger deeper; and then, again and again, one draw in twenty of those goes one
deeper still (up to the limit the table shows), so a careless step on a quiet floor can still
meet something it should not, and very rarely something far worse. If nothing that lives in the place is strong
enough to be chosen that deep (every kind more than 8 below it), the deeper creature comes from
the generic list instead. If nothing anywhere is, the draw is made at the floor's own danger.
Each draw is a stray or not and out of depth or not by its own chance; as a share of all draws:

{{#include generated/draw-odds.md}}

A creature marked "once a run" comes at most once in your life, however many floors it could
come to.

## A floor starts full

A dungeon floor with a danger is full when you first come down to it: its population (the
number in the tables below; by default 35 creatures for every 1000 cells you could walk on, at
least 25 and at most 80) is chosen as above and spread over its plain floor, never within 8
steps of a staircase, never in a shop, a sanctuary, or a teeming den. Most walk a short beat, back and forth;
where there is no room for one, they wait where they stand. They come armed as their kind is
(see the creatures' gear), on top of the few the floor's own rooms and vaults hold. A creature
marked "once a run" that you have met already is not among them. A hand-made place with a
danger starts as it was made; only the arrivals below fill it up. A [teeming den](special-rooms.md#teeming-dens)
holds its sleepers on top of the population, and they count toward it below.

## They keep coming

While you are on a floor with a danger, the creatures that live there keep coming: every so
often (a quarter of an hour of game time, unless a place says otherwise) one may come, with a
chance, as long as the floor holds fewer creatures hostile to you than its population. It comes
somewhere you cannot see, never within 8 steps of you, never on stairs, in a doorway, on a shop
floor, or in a sanctuary. A cleared floor fills up again while you stay; a long rest below lets
it refill. A floor you are not on is frozen: nothing comes while you are away, and the clock
starts again when you come back.

The talents of the Delver tree help: **Quiet delver** puts each next arrival off twice as long,
and **Sixth sense** tells you how a place feels against your own level as you arrive (quiet,
uneasy, dangerous, or deadly), and at its second rank notes it when something stirs on your
floor.

## The dungeons

**Cealdhlāw**, the cold barrow, is the Ascmearc's first dungeon, its way in on the downs of
Hlāwdūn east of Cēaptūn. Its first two floors are the barrow's passages and chambers; the four
below are the hollows of the hill, wide caves. It is cold (8 °C), its traps are arrows, spears,
pits, and alarms, and its dead walk: crawling hands and the bones of rats and wolves, skeletons
(some with bows, some armed and shielded warriors), zombies (kobolds among them), ghouls, and the
barrow wight, with goblins, kobolds, and vermin among them, and in the hollows an orc war band
with its wargs. On its second floor, always, lies the
**barrow chamber**: a skeleton paces a burial hall before a locked bronze door, and behind it
the silver chalice Sister Hild wants back lies on a plinth. The skeleton carries the bronze key.
On the first floor you feel cold air breathing up from the mound.

{{#include generated/dungeons.md}}

### Floor by floor

{{#include generated/dungeon-floors.md}}

### Fennmynster

**Fennmynster**, the fen minster, is what is left of a minster that sank into the marsh of
Micelfenn: eight floors of its cellars and crypts at 6 °C. No road leads there and its way in
cannot be found by looking: Sister Hild marks it on your map as you take up her quest **The fen
minster**, the one after she has her chalice back. The fen seeps into its rooms: one
room in four is out of the ordinary (most floors have one or two), flooded ankle-deep (slow and
loud to wade), hushed, or shadowed. Its traps are pits, frost, alarms, and teleports, and the
monks hid their stores well: more of its doors are secret than in the other dungeons. The
marsh's leeches, frogs, crocodiles, wisps, mud golems, and trolls, and the lesser water
elementals the rain and the fen water make, live here among the minster's
drowned dead, its peat-dark bog bodies, and its ghasts and fen ghouls, whose claws hold fast,
and the restless spirits that pass through its walls: corpse candles, poltergeists, ghosts, the
shadows that drink your strength, and the wraiths that drink your life. With
them is the cult that came to the ruin, cultists and their priests; somewhere in the dark is
the bog hag, whom you meet once in your life. No one trades down there: Fennmynster has no shops.

At the very bottom lies the **sunken crypt**: a nave of shallow water between the monks' tombs,
where two of the drowned still stand guard, and at its far end a dry dais with an altar and the
minster reliquary on it, which Sister Hild's last quest asks for. The crypt is on the eighth
floor and nowhere else.

### Fyrdelf

**Fyrdelf**, the fire delving, is the duergar's mine under the hills of Ascdūn, below the
volcano. No road leads there and its way in cannot be found by looking: Factor Wulfrun marks it
on your map as you take up her quest **The lost shaft**, which she offers once you have been four
floors down into Cealdhlāw for the reeve, and gives you only once you have reached level 5. Its
first four floors are the galleries, rooms and passages at 10 °C; the eight
below are the deep workings, wide caves that grow hotter toward the volcano's heart (16 °C,
then 20 °C, and 24 °C at the bottom). Its traps are arrows, spears, fireballs, lightning, and
alarms, few of its doors are hidden, and its locks are the hardest to pick of the three
dungeons. The duergar dig here, miners, warriors, and smiths, among ember beetles, salamanders,
magma crawlers, lesser fire elementals, cinder elementals, earth elementals that move through
the rock, ash wraiths, basilisks, ogres, trolls, and hill trolls down from the hills above, and the dead the
delf's fires left: ogre zombies and grave hounds, burning skulls, the charred dead, duergar
skeletons still in their helms, rotting trolls, and the spirits of the miners, knockers and
spectres, that pass through the rock; hell hounds, fiends at home in the fires, run its
deep workings, with kobolds, centipedes, and bats in the upper galleries. Hobgoblin war bands come
down to raid the duergar: soldiers, archers, legionnaires with spear and shield, warcasters,
sergeants, and captains. Orc war bands raid the mine too: raiders, shamans, berserkers, veterans,
bowmen, and their wargs, and now and then a war chief.

Every gallery floor holds a **foreman's cabin**: a timber-floored lodging where a duergar miner
keeps watch, with a spare pick-axe, his lamp, and a brass key to the delf's locked rooms behind a
plank wall. Duergar traders keep general stores in the galleries, where a room is fit for one
(about one gallery floor in four). At the very bottom lies the **drake's hoard**: a hall ringed
with lava where the fire drake sleeps on its gold, with a rare treasure and a magic one. The
drake lives nowhere else.

### The lairs of the wanted

Four more dungeons, each four floors deep, of danger 6 on the first floor and 9 on the fourth, are
the lairs of the criminals the [Weyfarers' Gilde](guild.md#the-wanted) wants dead. Each is hidden
until Wat One-Eye names its criminal and marks it on your map; the other three stay hidden for
the rest of your life. No one trades in any of them. At the bottom of each, and nowhere else,
the criminal waits in a room of their own:

- **Wyrtholm**, the herb islet, is the caves under an islet out in Micelfenn (9 °C), where
  **Mother Sallow** brews among the fen's leeches, frogs, crocodiles, wisps, mud golems, and
  marsh trolls, with footpads in her pay. Its traps are pits and alarms. Her **still-room** at
  the bottom holds her pots, her jars of potions, and angelica root.
- **Wulfeshol**, the wolf's hole, is a cave under Deorcholt where **Black Wystan**'s footpads and
  their archers keep watch, with goblin bowmen he hires, and bats, centipedes, and barrow spiders
  in the dark. Its traps are pits, arrows, and alarms. At the bottom, the **robbers' hoard**:
  the coin of the Fennhām road, a rare thing among it, two footpads at the door, and him.
- **Bānfeld**, the bone field, is the grave passages under a barrow field on Hlāwdūn (8 °C),
  where **Brother Osrick** keeps the dead he raised: skeletons, zombies, ghouls, the drowned,
  ghosts, and the worms and spiders that feed where the dead are. One room in four is shadowed or
  hushed; its traps are arrows, spears, pits, and alarms, and rusty keys open its locked rooms.
  At the bottom, the **unhallowed chapel**: bones heaped along the walls, two zombies in the
  aisle, and Osrick at an altar he defiled.
- **Brocenburg**, the broken fort, is the cellars under the ruined tower in Holegnwudu, where
  **Sergeant Herrick** sells his axe to the goblins: goblins, their archers and chiefs, footpads
  who ran with him, bats in the broken roof, and now and then an ogre he hired, far stronger
  than the floor. Its traps are arrows, spears, and alarms, and iron keys open its locked rooms.
  At the bottom, the **captain's hall**: the old table broken up, the captain he killed under a
  cairn in the corner, and Herrick in the captain's place, with two goblins at the door.
