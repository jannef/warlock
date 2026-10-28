# Dungeons and danger

A dungeon is a stack of floors made the first time you set foot on each, then kept as you left
them. Every place has a **danger level**, a number the game never shows you. On a dungeon floor
it is the dungeon's base danger plus a step for every floor down; a hand-made place has one of its
own; the overworld and the land you explore from it have none. A place could also grow more
dangerous as you do (a puppy cave of the old games), but no place does yet.

## Who lives there, and how many

Each dungeon names the [creatures](creatures.md) that live in it. Every creature's level is its danger: on a
floor of danger 6, the creatures you meet are mostly of level 5 and 6, some a little weaker, a
few weaker still, and now and then one from deeper down. Each time a creature is chosen, every
kind that lives there gets a weight by how far its level is below the floor's danger, and the
choice goes by weight (kinds of the same weight are equally likely):

{{#include generated/danger-weights.md}}

Some kinds are rarer than others: the more dangerous and the more rewarding (casters,
chieftains, the strongest of a place), the fewer. A kind's weight above is multiplied by its
rarity's share, so a rare kind at the floor's danger is chosen a quarter as often as a common
one of the same level. Creatures placed by hand ([people, shopkeepers](npcs.md), a place's own guardians)
are never chosen this way.

{{#include generated/creature-rarities.md}}

A dungeon is mostly its own folk, but now and then something wanders in from elsewhere: one
draw in twenty (unless a place says otherwise) is a **stray**, chosen the same way but from the
**generic list**: every kind of the land that is [hostile](creatures.md#factions) to you, whether any dungeon is its home
or not, and every kind that lives in some dungeon of the land (a creature marked "once a run" is
never on it, nor are [people, shopkeepers](npcs.md), and the guardians placed by hand, and [animated objects](animated.md)
come only where a place's folk include them). A stray may be one of the place's own kinds too.
Some places, the outlaws' four lairs, take their strays only from the kinds that live in the
land's dungeons (the "Strays" column of the table below). With the deeper draws below, anything the land knows
can come to any floor, however rarely: a kind no dungeon is home to is met only as a stray.

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
steps of a [staircase](terrain.md#stairs), never in a [shop](shops.md), a [sanctuary](special-rooms.md#room-sanctuary), or a [teeming den](special-rooms.md#teeming-dens). Most walk a short beat, back and forth;
where there is no room for one, they wait where they stand. They come armed as their kind is
(see the creatures' gear), on top of the few the floor's own rooms and vaults hold. A creature
marked "once a run" that you have met already is not among them. A hand-made place with a
danger starts as it was made; only the arrivals below fill it up. A [teeming den](special-rooms.md#teeming-dens)
holds its sleepers on top of the population, and they count toward it below.

## They keep coming

While you are on a floor with a danger, the creatures that live there keep coming: every so
often (a quarter of an hour of game time, unless a place says otherwise) one may come, with a
chance, as long as the floor holds fewer creatures [hostile](creatures.md#factions) to you than its population. It comes
somewhere you cannot see, never within 8 steps of you, never on [stairs](terrain.md#stairs), in a doorway, on a shop
floor, or in a [sanctuary](special-rooms.md#room-sanctuary). A cleared floor fills up again while you stay; a [long rest](resting.md#how-resting-works) below lets
it refill. A floor you are not on is frozen: nothing comes while you are away, and the clock
starts again when you come back.

The talents of the [Delver](talents.md#delver) tree help: **[Quiet delver](talents.md#talent-quiet-delver)** puts each next arrival off twice as long,
and **[Sixth sense](talents.md#talent-sixth-sense)** tells you how a place feels against your own [level](experience.md#levels) as you arrive (quiet,
uneasy, dangerous, or deadly), and at its second rank notes it when something stirs on your
floor.

## The dungeons

**Cealdhlāw**, the cold barrow, is the Ascmearc's first dungeon, its way in on the downs of
Hlāwdūn east of Cēaptūn. Its first two floors are the barrow's passages and chambers; the four
below are the hollows of the hill, wide caves. It is [cold](temperature.md) (8 °C), its traps are [arrows](traps.md#trap-arrow), [spears](traps.md#trap-spear),
[pits](traps.md#trap-pit), and [alarms](traps.md#trap-alarm), and its dead walk: crawling hands and the bones of rats and wolves, [skeletons](creatures.md#creature-skeleton)
(some with bows, some armed and shielded warriors), [zombies](creatures.md#creature-zombie) ([kobolds](creatures.md#creature-kobold) among them), [ghouls](creatures.md#creature-barrow-ghoul), and the
[barrow wight](creatures.md#creature-barrow-wight), with [goblins](creatures.md#creature-goblin), [kobolds](creatures.md#creature-kobold), and vermin among them, and in the hollows an orc war band
with its [wargs](creatures.md#creature-warg). On its second floor, always, lies the
**barrow chamber**: a [skeleton](creatures.md#creature-skeleton) paces a burial hall before a [locked bronze door](doors.md#locked-doors), and behind it
the silver chalice Sister Hild wants back lies on a plinth. The [skeleton](creatures.md#creature-skeleton) carries the [bronze key](doors.md#keys).
On the first floor you feel cold air breathing up from the mound.

{{#include generated/dungeons.md}}

### Floor by floor

{{#include generated/dungeon-floors.md}}

### Fennmynster

**Fennmynster**, the fen minster, is what is left of a minster that sank into the marsh of
Micelfenn: eight floors of its cellars and crypts at 6 °C. No road leads there and its way in
cannot be found by looking: Sister Hild marks it on your map as you take up her [quest](quests.md) **The fen
minster**, the one after she has her chalice back. The fen seeps into its rooms: one
room in four is out of the ordinary (most floors have one or two), [flooded](special-rooms.md#room-flooded) ankle-deep (slow and
loud to wade), [hushed](special-rooms.md#room-hushed), or [shadowed](special-rooms.md#room-shadowed). Its traps are [pits](traps.md#trap-pit), [frost](traps.md#trap-frost), [alarms](traps.md#trap-alarm), and [teleports](traps.md#trap-teleport), and the
monks hid their stores well: more of its [doors are secret](doors.md#secret-doors) than in the other dungeons. The
marsh's [leeches](creatures.md#creature-giant-leech), [frogs](creatures.md#creature-giant-frog), [crocodiles](creatures.md#creature-crocodile), [wisps](creatures.md#creature-will-o-wisp), [mud golems](creatures.md#creature-mud-golem), and [trolls](creatures.md#creature-troll), and the lesser water
elementals the rain and the fen water make, live here among the minster's
[drowned](creatures.md#creature-drowned) dead, its peat-dark [bog bodies](creatures.md#creature-bog-body), and its [ghasts](creatures.md#creature-ghast) and [fen ghouls](creatures.md#creature-fen-ghoul), whose claws hold fast,
and the restless spirits that pass through its walls: [corpse candles](creatures.md#creature-corpse-candle), [poltergeists](creatures.md#creature-poltergeist), [ghosts](creatures.md#creature-ghost), the
[shadows](creatures.md#creature-shadow) that drink your strength, and the [wraiths](creatures.md#creature-wraith) that drink your life. With
them is the cult that came to the ruin, [cultists](creatures.md#creature-cultist) and their priests; somewhere in the dark is
the [bog hag](creatures.md#creature-bog-hag), whom you meet once in your life. Fey of the damp haunt the cellars, [pixies](creatures.md#creature-pixie),
[sprites](creatures.md#creature-sprite), [boggles](creatures.md#creature-boggle), [quicklings](creatures.md#creature-quickling), [satyrs](creatures.md#creature-satyr), and [darklings](creatures.md#creature-darkling);
water and [frost motes](creatures.md#creature-frost-mote) and [lesser ice elementals](creatures.md#creature-lesser-ice-elemental) stir in the cold water, [stone haunts](creatures.md#creature-stone-haunt) drift
through the minster's walls, and [pale drinkers](creatures.md#creature-pale-drinker) and [mummies](creatures.md#creature-mummy) keep its crypts. No one trades down there: Fennmynster has no [shops](shops.md).

At the very bottom lies the **sunken crypt**: a nave of [shallow water](terrain.md#terrain-shallow-water) between the monks' tombs,
where two of the [drowned](creatures.md#creature-drowned) still stand guard, and at its far end a dry dais with an altar and the
minster reliquary on it, which Sister Hild's last quest asks for. The crypt is on the eighth
floor and nowhere else.

### Fyrdelf

**Fyrdelf**, the fire delving, is the [duergar](races.md#race-duergar)'s mine under the hills of Ascdūn, below the
volcano. No road leads there and its way in cannot be found by looking: Factor Wulfrun marks it
on your map as you take up her [quest](quests.md) **The lost shaft**, which she offers once you have been four
floors down into Cealdhlāw for the reeve, and gives you only once you have reached level 5. Its
first four floors are the galleries, rooms and passages at 10 °C; the eight
below are the deep workings, wide caves that grow hotter toward the volcano's heart (16 °C,
then 20 °C, and 24 °C at the bottom). Its traps are [arrows](traps.md#trap-arrow), [spears](traps.md#trap-spear), [fireballs](traps.md#trap-fireball), [lightning](traps.md#trap-lightning-bolt), and
[alarms](traps.md#trap-alarm), few of its [doors are hidden](doors.md#secret-doors), and its locks are the hardest to pick of the three
dungeons. The [duergar](races.md#race-duergar) dig here, [miners](creatures.md#creature-duergar-miner), [warriors](creatures.md#creature-duergar-warrior), and [smiths](creatures.md#creature-duergar-smith), among [ember beetles](creatures.md#creature-ember-beetle), [salamanders](creatures.md#creature-fire-salamander),
[magma crawlers](creatures.md#creature-magma-crawler), [lesser fire elementals](creatures.md#creature-lesser-fire-elemental), [cinder elementals](creatures.md#creature-cinder-elemental), [earth elementals](creatures.md#creature-earth-elemental) that move through
the rock, [ash wraiths](creatures.md#creature-ash-wraith), [basilisks](creatures.md#creature-basilisk), [ogres](creatures.md#creature-ogre), [trolls](creatures.md#creature-troll), and [hill trolls](creatures.md#creature-hill-troll) down from the hills above, and the dead the
delf's fires left: [ogre zombies](creatures.md#creature-ogre-zombie) and [grave hounds](creatures.md#creature-grave-hound), [burning skulls](creatures.md#creature-burning-skull), the [charred dead](creatures.md#creature-charred-dead), [duergar](races.md#race-duergar)
skeletons still in their helms, [rotting trolls](creatures.md#creature-rotting-troll), and the spirits of the miners, [knockers](creatures.md#creature-knocker) and
[spectres](creatures.md#creature-spectre), that pass through the rock; [hell hounds](creatures.md#creature-hell-hound), fiends at home in the fires, run its
deep workings, and the upper galleries belong mostly to [kobolds](creatures.md#creature-kobold), [centipedes](creatures.md#creature-giant-centipede), and bats. [Hobgoblin](creatures.md#creature-hobgoblin) war bands come
down to raid the [duergar](races.md#race-duergar): [soldiers](creatures.md#creature-hobgoblin), [archers](creatures.md#creature-hobgoblin-archer), [legionnaires](creatures.md#creature-hobgoblin-legionnaire) with spear and shield, [warcasters](creatures.md#creature-hobgoblin-warcaster),
and [sergeants](creatures.md#creature-hobgoblin-sergeant). Orc war bands raid the mine too: [raiders](creatures.md#creature-orc-raider), [shamans](creatures.md#creature-orc-shaman), [berserkers](creatures.md#creature-orc-berserker), [veterans](creatures.md#creature-orc-veteran), [bowmen](creatures.md#creature-orc-bowman),
and their [wargs](creatures.md#creature-warg), with [goblin wizards](creatures.md#creature-goblin-wizard) who call up more of their kin. Their [captains](creatures.md#creature-hobgoblin-captain) and [war chiefs](creatures.md#creature-orc-war-chief),
like the duergar's [priests](creatures.md#creature-duergar-priest), stay deeper than the delf goes. The motes of stone, fire, and air
drift through the upper galleries, and [spark motes](creatures.md#creature-spark-mote), lesser earth, air, and lightning elementals,
and [water elementals](creatures.md#creature-water-elemental) through the workings. The demons' fires draw up [lemures](creatures.md#creature-lemure) and [imps](creatures.md#creature-imp); [meenlocks](creatures.md#creature-meenlock)
lurk in the dark. The deep workings' dead are many: [draugr](creatures.md#creature-draugr), [barghests](creatures.md#creature-barghest), [hands of glory](creatures.md#creature-hand-of-glory), vampire
spawn, and the [gallows dead](creatures.md#creature-gallows-dead).

Every gallery floor holds a **foreman's cabin**: a timber-floored lodging where a duergar miner
keeps watch, with a spare [pick-axe](digging.md#digging), his lamp, and a [brass key](doors.md#keys) to the delf's locked rooms behind a
plank wall. [Duergar traders](creatures.md#creature-duergar-trader) keep [general stores](shops.md) in the galleries, where a room is fit for one
(about one gallery floor in four). At the very bottom lies the **drake's hoard**: a hall ringed
with lava where the [fire drake](creatures.md#creature-fire-drake) sleeps on its gold, with a rare treasure and a magic one. The
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
