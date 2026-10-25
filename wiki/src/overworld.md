# The overworld

The overworld is the map of the land at large. One cell is a journey, not a pace: it is drawn at
map scale, and each step takes real time.

## Travel

A step on the overworld takes as long as the terrain you step onto: minutes or hours. Every other
action takes its usual time, as anywhere else: waiting takes 6 seconds, and so does entering a
location or a local area with `>` (see [Turns and movement](turns.md)). Resting passes the hours
quickly all the same (see [Resting](resting.md)).
Mountains, volcanoes, and open water cannot be crossed; a river can only be crossed by a bridge
or at a ford.

{{#include generated/terrain-overworld.md}}

Roads and bridges are the fast way to travel, and swamp and dense forest the slow way. The
[survival](skills.md#what-skills-give) skill makes every journey quicker, by up to a fifth. The
status bar shows the name of the region you are in, here and in its local areas. A long journey
takes days, and you get hungry on the way: take food (see [Hunger](hunger.md)).

## Sight by daylight and moonlight

Sight on the overworld is plain line of sight within a radius that depends on the time of day, at
night on the moon, and on the [weather](weather.md): rain shortens it, and clouds dim the moon.
Light, your eyes' threshold, and darkvision play no part. Mountain ranges and dense forest block
sight; ordinary forest does not.

{{#include generated/overworld-sight.md}}

At night the overworld is drawn by a faint cold light so the cells in sight stand out from the dim
remembered map, with the moonlight on top of it. See [Sun, moon, and sky](sun.md).

## The Ascmearc

The Ascmearc, "the ash march", is 120 by 60 cells, several days of travel across. You come
into it by **Beorggeat**, a pass in Grægweall, the mountain wall on its western edge. Snow
closes the pass behind you: it is no way back, and there is nothing to explore there. The road
runs five cells east to **Cēaptūn**, the market town, and on from there: east over the barrow
downs of Hlāwdūn to the ford over Blæcwæter, south over the bridge on Beorhtburna and along its
bank, and north to the edge of the holly wood. The volcano **Fyrbeorg** smokes on the south face
of Rēcbeorgas, the range in the north-east. The names are Old English: plain words for what each
place is.

{{#include generated/overworld-places.md}}

**Cēaptūn** is the first place to go: a walled market town, safe, with four shops on its
square, the Mynster (the temple, where you may rest and Sister Hild cures and mends for gold),
and the Gæsthūs, the inn, where Godgifu has news of the march (see [Shops](shops.md) and
[People and talk](npcs.md)). Its stairs up lead back out to its cell. The places that lead
"nowhere yet" are still being built: for now `>` there says so. The two
hidden ones are where they will be once a quest tells you of them (see "Hidden locations"
below); until then each is the land it looks like.

## Locations

Some cells are **locations**: ways into other levels such as a hall, a cave, or a dungeon. Walking
onto one tells you where it leads, and `>` enters it. You arrive on the stairs that lead back, and
`<` there takes you out to the overworld at the location.

### Hidden locations

Some locations are **hidden**: until someone tells you where one is, its cell is just the land
it looks like, a stretch of swamp or forest, in every way. Walking onto it is a journey through
that land, and `>` explores it like any other cell. Searching does not find one: **only a
quest reveals it**. A quest that does, as it is offered, as you take it up, or as you hand it in,
has its giver mark the place on your map ("… marks the old barrow on your map."):
from then on the overworld shows the way in at that cell, wherever you were when you heard of
it, and it leads in like any other location. The quest log may say in words where the place
lies. Hearing of it a second time changes nothing.

## Local areas

On any other cell whose terrain has a local area (the "Local area" column above), press `>` to
explore that cell on foot. You enter an outdoor area, 80 by 40 cells, made for that cell:

- It is generated the first time you go there, and stays as you left it afterwards.
- What it looks like follows the terrain of the cell and its neighbors: meadows, woods with
  clearings, rocky hills with crags, marsh with pools and reeds, roads, riverbanks or lake shores
  along the sides where the overworld has them, and now and then ponds, brooks, ruins, or deer.
  Woods blend towards the neighboring cells so areas side by side fit together.
- You arrive on the side you walked onto the cell from (on the road if you came along one), or in a
  clearing in the middle if you did not walk there.
- Walk off any edge to return to the overworld cell. This takes 6 seconds, like taking stairs.
- Inside, a step is a pace again, not a journey (6 seconds on open ground), and the sun and sky
  light the area by day, under the same
  [weather](weather.md) as the overworld. At night only the moon lights it: under a bright moon
  and a clear sky you dimly make out the open ground, otherwise it is dark, so bring your
  [lantern](lantern.md).
