# The overworld

The overworld is the map of the land at large. It is drawn at map scale: one cell is a journey,
not a pace, and each step takes real time.

## Travel

A step on the overworld takes as long as the [terrain](terrain.md) you step onto: minutes or
hours. Every other action takes its usual time, as anywhere else. [Waiting](turns.md#what-actions-cost)
takes 6 seconds, and so does entering a [location](overworld.md#locations) or a
[local area](#local-areas) with `>` (see [Turns and movement](turns.md)).
[Resting](resting.md#how-resting-works) passes the hours quickly all the same (see
[Resting](resting.md)). Mountains, volcanoes, and open water cannot be crossed. A river can only be
crossed by a bridge or at a ford.

{{#include generated/terrain-overworld.md}}

Roads and bridges are the fast way to travel; swamp and dense forest are the slow way. The
[survival](skill-benefits.md) skill makes every journey quicker, by up to a fifth. The
[status bar](the-screen.md#the-screen) shows the name of the region you are in, here and in its
[local areas](#local-areas).

A long journey takes days, and you get [hungry](hunger.md#how-hungry-you-are) on the way, so take
[food](food.md#eating) (see [Hunger](hunger.md)). On the way you may
[meet someone](encounters.md): footpads, raiders, travellers, or animals. The longer you are out in
the wild, the more you meet.

## Sight by daylight and moonlight

Sight on the overworld is plain [line of sight](sight.md#line-of-sight) within a radius. The radius
depends on the time of day, at night on the moon, and on the [weather](weather.md): rain shortens
it, and clouds dim the moon. [Light](light.md#what-light-is), your eyes' threshold, and
[darkvision](vision.md#the-four-types) play no part. Mountain ranges and dense forest block sight;
ordinary forest does not.

{{#include generated/overworld-sight.md}}

At night the overworld is drawn in a faint cold light, so the cells in sight stand out from the dim
[remembered](sight.md#memory) map, with the [moonlight](sun.md#moonlight) on top. See
[Sun, moon, and sky](sun.md).

## The Ascmearc

The overworld you play on is the Ascmearc's, 120 by 60 cells. Its places, regions, names, and the
whole map are on [The Ascmearc](ascmearc.md).

## Locations

Some cells are **locations**: ways into other levels, such as a hall, a cave, or a
[dungeon](dungeons.md). Walking onto one tells you where it leads, and `>` enters it. You arrive on
the [stairs](terrain.md#stairs) that lead back, and `<` there takes you out to the overworld at the
location. A town has no [stairs](terrain.md#stairs). You arrive in it, and leave it as you leave a
local area ([Leaving an area](#leaving-an-area)).

### Hidden locations

Some locations are **hidden**. Until someone tells you where one is, its cell is in every way just
the land it looks like, a stretch of swamp or forest. Walking onto it is a journey through that
land, and `>` explores it like any other cell. [Searching](searching.md#how-to-search) does not find
one.

**Only a [quest](quests.md) reveals a hidden location**, or coming out of it from below. A few
dungeons have a second way in, deep down. Climbing out through it marks its hidden mouth on your
map as you step out onto it ("You mark … on your map.").

A [quest](quests.md) that reveals one has its giver mark the place on your map, either as the quest
is given to you or as you hand it in ("… marks the old barrow on your map."). From then on the
overworld shows the way in at that cell, wherever you were when you heard of it, and it leads in
like any other location. The [quest log](quests.md#the-quest-log) may say in words where the place
lies. Hearing of it a second time changes nothing.

### Rumoured ways

Some people tell you the way to a place when you ask them about it. Godgifu at the inn in Cēaptūn
knows the way to the barrows, the marsh, the mines, the volcano, and the pass. You mark the way on
your map at once, wherever you are, and it takes no time.

Every cell you would have seen walking that way from the town [by day](time.md#days) goes on your
map as you would remember it. That is every cell within 9 cells of the way and in
[line of sight](sight.md#line-of-sight) from it, so hills and mountains still hide what lies behind
them. Asking again adds nothing. Each way is a line through its waypoints, starting at the town's
cell, (18, 28):

{{#include generated/rumoured-ways.md}}

A rumoured way shows only the land itself. A [hidden location](#hidden-locations) along it is a
stretch of swamp or hills like any other, as if you had walked past it. Only a [quest](quests.md)
reveals one.

## Local areas

On any other cell whose [terrain](terrain.md) has a local area (the "Local area" column above),
press `>` to explore that cell on foot. You enter an outdoor area of 80 by 40 cells, made for that
cell:

- It is made the first time you go there, and stays as you left it afterwards.
- What it looks like follows the terrain of the cell and its neighbours: meadows, woods with
  clearings, rocky hills with crags, [marsh](terrain.md#terrain-marsh) with pools and
  [reeds](terrain.md#terrain-reeds). There are roads, riverbanks, or lake shores along the sides
  where the overworld has them, and now and then ponds, brooks, ruins, or
  [deer](creatures.md#creature-deer). Woods blend towards the neighbouring cells, so areas side by
  side fit together.
- A wood may hide a [boar](creatures.md#creature-boar), a deep wood a
  [lynx](creatures.md#creature-lynx), and the hills a [badger](creatures.md#creature-badger). These
  wild beasts leave you be unless you fall on them, and they are the kinds a ranger may keep as a
  [companion](companions.md#companions).
- You arrive on the side you walked onto the cell from (on the road if you came along one), or in a
  clearing in the middle if you did not walk there.
- Walk off any edge to return to the overworld cell. This takes 6 seconds, like taking
  [stairs](terrain.md#stairs). See [Leaving an area](#leaving-an-area) for the quicker way.
- Inside, a step is a pace again, not a journey: 6 seconds on open ground. By day the
  [sun and sky](sun.md#sky-light) light the area, under the same [weather](weather.md) as the
  overworld. At night only the [moon](sun.md#moonlight) lights it. Under a bright moon and a clear
  sky you can dimly make out the open ground; otherwise it is dark, so bring your
  [lantern](lantern.md).

## Leaving an area

Every outdoor place you reach from the overworld is left the same way: a
[local area](#local-areas), and a town such as Cēaptūn, which has no stairs. **Walk off any edge**
and you are back on its cell of the overworld, in 6 seconds, like taking
[stairs](terrain.md#stairs).

But not with an enemy at your heels. While any creature [hostile](creatures.md#factions) to you is
within **3 cells** of you (a diagonal step counts as one cell; seen or not, awake or asleep), you
are told "They are too close to slip away." and no time passes. Fight, or get farther from them
first.

There is a quicker way: press `<` anywhere in the area (except on [stairs](terrain.md#stairs) up,
which you take as usual). You leave at once, exactly as if you had walked off the edge, **unless an
enemy is about**. An enemy is about if there is:

- any creature [hostile](creatures.md#factions) to you that you can see, however far away, or
- any [hostile](creatures.md#factions) creature you cannot see within **10 cells** of you, a
  diagonal step counting as one cell. That is a square 21 cells across with you in the middle. It
  counts awake or asleep, behind walls or not.

Creatures that are not [hostile](creatures.md#factions), such as townsfolk, merchants, and
[deer](creatures.md#creature-deer), never stop you. If an enemy is about, you are told "You cannot
slip away with enemies about." and no time passes. Walk off the edge instead, where none is within
3 cells of you.

Underground and on the overworld itself, `<` still needs [stairs](terrain.md#stairs). An outdoor
place that no overworld cell leads into is left by its stairs.

See also: [Encounters on the way](encounters.md), [The Ascmearc](ascmearc.md), and
[Weather](weather.md).
