# Dungeons and danger

A dungeon is a stack of floors. Each floor is made the first time you set foot on it, then kept as
you left it. This page explains who lives on a floor and how many; the dungeons with rules of
their own, from the endless Grundlēas to the lairs of the wanted, are on
[The dungeons one by one](dungeon-guide.md).

Every place has a **danger level**, a number the game never shows you. On a dungeon floor it is
the dungeon's base danger plus a step for every floor down. A hand-made place has one of its own.
The overworld and the land you explore from it have none. Only the
[Pits](pits.md#who-lives-there) grow more dangerous as you do.

## Who lives there, and how many

Each dungeon names the [creatures](creatures.md) that live in it. A creature's level is its
danger. On a floor of danger 6, the creatures you meet are mostly of level 5 and 6, some a little
weaker, a few weaker still, and now and then one from deeper down. Each time a creature is chosen,
every kind that lives there gets a weight by how far its level is below the floor's danger, and
the choice goes by weight. Kinds of the same weight are equally likely:

{{#include generated/danger-weights.md}}

Some kinds are rarer than others. The more dangerous and rewarding a kind is (casters, chieftains,
the strongest of a place), the rarer. A kind's weight above is multiplied by its rarity's share, so
a rare kind at the floor's danger is chosen a quarter as often as a common one of the same level.
Creatures placed by hand ([people, shopkeepers](npcs.md), a place's own guardians) are never chosen
this way.

{{#include generated/creature-rarities.md}}

A dungeon is for fighting. Kinds that are not [hostile](creatures.md#factions) to you, the deer,
bats, beetles, and other wild animals that leave you be, are chosen far less often when a floor is
filled or a creature comes later: their weight is multiplied once more, by the share below, unless
the [table of dungeons](#the-dungeons) says otherwise (the fey of Ælfhām are its folk and keep
theirs). The draws they lose go to the hostile kinds, so a floor holds as many creatures as
before. A [teeming den](teeming-dens.md) of wild beasts is still full of them, and on the land
you explore from the overworld the wild animals are as common as ever.

{{#include generated/neutral-weight.md}}

### Strays

A dungeon holds mostly its own folk, but now and then something wanders in from elsewhere. One
draw in twenty, unless a place says otherwise, is a **stray**. It is chosen the same way, but from
the **generic list**:

- every kind of the land that is [hostile](creatures.md#factions) to you, whether any dungeon is
  its home or not;
- every kind that lives in some dungeon of the land.

A creature marked "once a run" is never on the generic list, and neither are
[people, shopkeepers](npcs.md), or the guardians placed by hand. [Animated objects](animated.md)
come only where a place's folk include them. A stray may also be one of the place's own kinds.
Some places, the outlaws' four lairs, take their strays only from the kinds that live in the land's
dungeons (the "Strays" column of the table below). Together with the deeper draws below, this means
anything the land knows can come to any floor, however rarely. A kind no dungeon is home to is met
only as a stray.

### Out of depth

Another draw in twenty, again unless a place says otherwise, is made **out of depth**: as if the
floor were one danger deeper. Then one draw in twenty of those goes one deeper still, and so on,
up to the limit the table shows. So a careless step on a quiet floor can still meet something it
should not, and very rarely something far worse.

If nothing that lives in the place is strong enough to be chosen that deep (every kind more than 8
below it), the deeper creature comes from the generic list instead. If nothing anywhere is, the
draw is made at the floor's own danger. Whether a draw is a stray and whether it is out of depth
are each decided by their own chance. As a share of all draws:

{{#include generated/draw-odds.md}}

### Dread

Some kinds are **dread**: the monsters to fear. Each has something of its own that a plain fight
does not prepare you for: the basilisk, the shadows, the vampires, the banshee, the nymph. The
[creature tables](creatures.md) mark them "dread".

Before anything else, one draw in a hundred, unless a place says otherwise, is a **dread draw**. It
is made among the dread kinds alone, the place's own and the
[strays](#who-lives-there-and-how-many) alike, each weighted by its level and its
[rarity](#who-lives-there-and-how-many) as above. A common dread kind therefore comes far more often
than a very rare one. Every other draw is made among the kinds that are not dread. If a dread draw
finds no dread kind strong enough to be chosen there, it is made the ordinary way instead.

There is no limit. A floor of forty creatures has about one chance in three of holding a dread
creature, before counting those that come later. Creatures placed by hand, a
[vault](#a-floor-starts-full)'s own folk, and the first kind a
[teeming den](teeming-dens.md#teeming-dens) is drawn around are never dread draws.

The deeper and more dangerous a place, the more often a draw is dread. In Grundlēas, past the
floor where the draws stop rising, the dread keep coming more often floor by floor:

{{#include generated/dread-draws.md}}

A creature marked "once a run" comes at most once in your life, however many floors it could come
to.

## A floor starts full

A dungeon floor with a danger is full when you first come down to it. Its population is the number
in the tables below; by default it is 35 creatures for every 1000 cells you could walk on, at least
25 and at most 80. They are chosen as above and spread over its plain floor. None is placed within
8 steps of a [staircase](terrain.md#stairs), in a [shop](shops.md), in a
[sanctuary](special-rooms.md#room-sanctuary), or in a [teeming den](teeming-dens.md#teeming-dens).
Most walk a short beat, back and forth; where there is no room for one, they wait where they stand.
They come armed as their kind is (see the creatures' gear), on top of the few the floor's own rooms
and vaults hold. A creature marked "once a run" that you have met already is not among them.

Some vaults hold a crowd of their own, on top of the population. It is chosen as above when the
floor is made (a guard post keeps two of the floor's creatures beside its guard), so who is inside
differs from one visit to the next. A hand-made place with a danger starts as it was made; only the
arrivals below fill it up. A [teeming den](teeming-dens.md#teeming-dens) holds its sleepers on top
of the population, and they count toward it below.

## They keep coming

While you are on a floor with a danger, the creatures that live there keep coming. Every so often,
two minutes of game time unless a place says otherwise, one may come, with a 60 percent chance, as
long as the floor holds fewer creatures [hostile](creatures.md#factions) to you than its
population. A vault's own crowd is not counted, so leaving a vault shut does not stop the arrivals.

A newcomer appears somewhere you cannot see, never within 8 steps of you. It never appears on
[stairs](terrain.md#stairs), in a doorway, on a shop floor, or in a
[sanctuary](special-rooms.md#room-sanctuary). A cleared floor fills up again while you stay, and a
[long rest](resting.md#how-resting-works) below lets it refill. A floor you are not on is frozen:
nothing comes while you are away, and the clock starts again when you come back.

The [Delver](talents.md#delver) tree's [Quiet delver](talents.md#talent-quiet-delver) talent puts
off each next arrival twice as long.

## The dungeons

**Lytelscræf**, the little cave, is the easiest place to start. Its mouth opens in the foot of the
western wall two cells from Beorggeat, where you come into the Ascmearc, and you see it from your
first step. It has three short floors of cave at [danger](#who-lives-there-and-how-many) 1, 2, and
3, thinly peopled by the weakest things in the land and nothing else:
[giant rats](creatures.md#creature-rat), [cave bats](creatures.md#creature-bat),
[cave beetles](creatures.md#creature-beetle), [grave worms](creatures.md#creature-grave-worm), and
[kobolds](creatures.md#creature-kobold). No [stray](#who-lives-there-and-how-many) from elsewhere
comes in, and none from [deeper down](#who-lives-there-and-how-many). Its [traps](traps.md) are a few
[pits](traps.md#trap-pit) and [arrows](traps.md#trap-arrow). It has no [shop](shops.md) and no
[special rooms](special-rooms.md).

On its third floor there is always the **kobolds' cache**: two
[kobolds](creatures.md#creature-kobold) keeping a [chest](chests.md) that is neither
[locked](chests.md#locked-chests) nor [trapped](chests.md#trapped-chests), filled as any chest of
that floor would be. On the first floor the cave smells of wet stone and rats.

**Cealdhlāw**, the cold barrow, is the first of the Ascmearc's deeper dungeons. Its way in is on the
downs of Hlāwdūn, east of Cēaptūn. Its first two floors are the barrow's passages and chambers; the
four below are the hollows of the hill, wide caves. It is [cold](temperature.md) (8 °C). Its traps
are [arrows](traps.md#trap-arrow), [spears](traps.md#trap-spear), [pits](traps.md#trap-pit), and
[alarms](traps.md#trap-alarm).

Its dead walk: crawling hands and the bones of rats and wolves,
[skeletons](creatures.md#creature-skeleton) (some with bows, some armed and shielded warriors),
[zombies](creatures.md#creature-zombie) ([kobolds](creatures.md#creature-kobold) among them),
[ghouls](creatures.md#creature-barrow-ghoul), and the
[barrow wight](creatures.md#creature-barrow-wight). Among them live
[goblins](creatures.md#creature-goblin), [kobolds](creatures.md#creature-kobold), and vermin, and in
the hollows an orc war band with its [wargs](creatures.md#creature-warg).

On its second floor there is always the **barrow chamber**. A
[skeleton](creatures.md#creature-skeleton) paces a burial hall before a
[locked bronze door](doors.md#locked-doors). Behind it, the silver chalice Sister Hild wants back
lies on a plinth. The [skeleton](creatures.md#creature-skeleton) carries the
[bronze key](doors.md#keys). On the first floor you feel cold air breathing up from the mound.

{{#include generated/dungeons.md}}

### Floor by floor

{{#include generated/dungeon-floors.md}}

See also: [The dungeons one by one](dungeon-guide.md), [Teeming dens](teeming-dens.md), and
[Quests](quests.md).
