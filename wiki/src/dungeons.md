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

### Dread

Some kinds are **dread**: the monsters to fear, each with something of its own that a plain
fight does not prepare you for (the basilisk, the shadows, the vampires, the banshee, the nymph;
the [creature tables](creatures.md) mark them "dread"). Before anything else, one draw in a
hundred (unless a place says otherwise) is a **dread draw**: it is made among the dread kinds
alone, the place's own and the [strays](#who-lives-there-and-how-many) alike, each by its level
and its [rarity](#who-lives-there-and-how-many) as above, so a common dread kind comes far more
often than a very rare one. Every other draw is made among the kinds that are not dread. If a
dread draw finds no dread kind strong enough to be chosen there, it is made the ordinary way
instead. There is no limit: a floor of forty creatures has about one chance in three of holding
one, before the ones that come later. Creatures placed by hand, a [vault](#a-floor-starts-full)'s
own folk, and the first kind a [teeming den](teeming-dens.md#teeming-dens) is drawn around are never dread draws.

The deeper and the more dangerous a place, the oftener a draw is dread; in Grundlēas, past the
floor where the draws stop rising, the dread keep coming oftener floor by floor:

{{#include generated/dread-draws.md}}

A creature marked "once a run" comes at most once in your life, however many floors it could
come to.

## A floor starts full

A dungeon floor with a danger is full when you first come down to it: its population (the
number in the tables below; by default 35 creatures for every 1000 cells you could walk on, at
least 25 and at most 80) is chosen as above and spread over its plain floor, never within 8
steps of a [staircase](terrain.md#stairs), never in a [shop](shops.md), a [sanctuary](special-rooms.md#room-sanctuary), or a [teeming den](teeming-dens.md#teeming-dens). Most walk a short beat, back and forth;
where there is no room for one, they wait where they stand. They come armed as their kind is
(see the creatures' gear), on top of the few the floor's own rooms and vaults hold. A creature
marked "once a run" that you have met already is not among them. Some vaults hold a
crowd of their own, chosen as above when the floor is made, on top of the population (a guard
post keeps two of the floor's creatures beside its guard), so who is inside differs from one
visit to the next. A hand-made place with a
danger starts as it was made; only the arrivals below fill it up. A [teeming den](teeming-dens.md#teeming-dens)
holds its sleepers on top of the population, and they count toward it below.

## They keep coming

While you are on a floor with a danger, the creatures that live there keep coming: every so
often (two minutes of game time, unless a place says otherwise) one may come, with a 60 percent
chance, as long as the floor holds fewer creatures [hostile](creatures.md#factions) to you than its population; a vault's
own crowd is not counted, so leaving a vault shut does not stop the arrivals. It comes
somewhere you cannot see, never within 8 steps of you, never on [stairs](terrain.md#stairs), in a doorway, on a shop
floor, or in a [sanctuary](special-rooms.md#room-sanctuary). A cleared floor fills up again while you stay; a [long rest](resting.md#how-resting-works) below lets
it refill. A floor you are not on is frozen: nothing comes while you are away, and the clock
starts again when you come back.

The [Delver](talents.md#delver) tree's **[Quiet delver](talents.md#talent-quiet-delver)** puts each next arrival off twice as long.

## The dungeons

**Lytelscræf**, the little cave, is the easiest place to start: its mouth opens in the foot of
the western wall two cells from Beorggeat, where you come into the Ascmearc, and you see it
from your first step. It is three short floors of cave at [danger](#who-lives-there-and-how-many) 1, 2, and 3,
thinly peopled by the weakest things in the land and nothing else: [giant rats](creatures.md#creature-rat),
[cave bats](creatures.md#creature-bat), [cave beetles](creatures.md#creature-beetle),
[grave worms](creatures.md#creature-grave-worm), and [kobolds](creatures.md#creature-kobold). No
[stray](#who-lives-there-and-how-many) from elsewhere comes in, and none from
[deeper down](#who-lives-there-and-how-many). Its [traps](traps.md) are a few
[pits](traps.md#trap-pit) and [arrows](traps.md#trap-arrow); it has no [shop](shops.md) and no
[special rooms](special-rooms.md). On its third floor, always, is the **kobolds' cache**: two
[kobolds](creatures.md#creature-kobold) keeping a [chest](chests.md) that is neither
[locked](chests.md#locked-chests) nor [trapped](chests.md#trapped-chests), filled as any chest
of that floor would be. On the first floor the cave smells of wet stone and rats.

**Cealdhlāw**, the cold barrow, is the first of the Ascmearc's deeper dungeons, its way in on the downs of
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
