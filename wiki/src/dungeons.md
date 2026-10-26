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

One draw in twenty (unless a place says otherwise) is made one to three dangers deeper, so a
careless step on a quiet floor can still meet something it should not. A creature marked "once a
run" comes at most once in your life, however many floors it could come to.

## A floor starts full

A dungeon floor with a danger is full when you first come down to it: its population (the
number in the tables below; by default 35 creatures for every 1000 cells you could walk on, at
least 25 and at most 80) is chosen as above and spread over its plain floor, never within 8
steps of a staircase, never in a shop or a sanctuary. Most walk a short beat, back and forth;
where there is no room for one, they wait where they stand. They come armed as their kind is
(see the creatures' gear), on top of the few the floor's own rooms and vaults hold. A creature
marked "once a run" that you have met already is not among them. A hand-made place with a
danger starts as it was made; only the arrivals below fill it up.

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
pits, and alarms, and its dead walk: grave worms, skeletons, zombies, ghouls, and the barrow
wight, with goblins, kobolds, and vermin among them. On either of its first two floors you may
find the **barrow chamber**: a skeleton paces a burial hall before a locked bronze door, and
behind it something of worth lies on a plinth. The skeleton carries the bronze key.

{{#include generated/dungeons.md}}

### Floor by floor

{{#include generated/dungeon-floors.md}}

### Fennmynster

**Fennmynster**, the fen minster, is what is left of a minster that sank into the marsh of
Micelfenn: eight floors of its cellars and crypts at 6 °C. No road leads there and its way in
cannot be found by looking: a quest must show you where it is. The fen seeps into its rooms: one
room in four is out of the ordinary (most floors have one or two), flooded ankle-deep (slow and
loud to wade), hushed, or shadowed. Its traps are pits, frost, alarms, and teleports, and the
monks hid their stores well: more of its doors are secret than in the other dungeons. The
marsh's leeches, frogs, crocodiles, wisps, mud golems, and trolls live here among the minster's
drowned dead and its ghosts, and with them the cult that came to the ruin, cultists and their
priests; somewhere in the dark is the bog hag, whom you meet once in your life. No one trades
down there: Fennmynster has no shops.

At the very bottom lies the **sunken crypt**: a nave of shallow water between the monks' tombs,
where two of the drowned still stand guard, and at its far end a dry dais with an altar and a
magic treasure on it. The crypt is on the eighth floor and nowhere else.

### Fyrdelf

**Fyrdelf**, the fire delving, is the duergar's mine under the hills of Ascdūn, below the
volcano. No road leads there and its way in cannot be found by looking: a quest must show you
where it is. Its first four floors are the galleries, rooms and passages at 10 °C; the eight
below are the deep workings, wide caves that grow hotter toward the volcano's heart (16 °C,
then 20 °C, and 24 °C at the bottom). Its traps are arrows, spears, fireballs, lightning, and
alarms, few of its doors are hidden, and its locks are the hardest to pick of the three
dungeons. The duergar dig here, miners, warriors, and smiths, among ember beetles, salamanders,
magma crawlers, cinder elementals, ash wraiths, basilisks, ogres, and trolls, with kobolds,
centipedes, and bats in the upper galleries.

Every gallery floor holds a **foreman's cabin**: a timber-floored lodging where a duergar miner
keeps watch, with a spare pick-axe, his lamp, and a brass key to the delf's locked rooms behind a
plank wall. Duergar traders keep general stores in the galleries, where a room is fit for one
(about one gallery floor in four). At the very bottom lies the **drake's hoard**: a hall ringed
with lava where the fire drake sleeps on its gold, with a rare treasure and a magic one. The
drake lives nowhere else.
