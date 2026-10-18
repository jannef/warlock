# The overworld

The overworld is the map of the land at large. One cell is a journey, not a pace: it is drawn at
map scale, and each step takes real time.

## Travel

Time on the overworld passes in minutes. A step takes as long as the terrain you step onto, and
waiting or any other action takes one 10-minute turn (see [Turns and movement](turns.md)).
Mountains and open water cannot be crossed; a river can only be crossed by a bridge.

{{#include generated/terrain-overworld.md}}

Roads and bridges are the fast way to travel, and swamp and dense forest the slow way. The status
bar shows the name of the region you are in.

## Sight by daylight and moonlight

Sight on the overworld is plain line of sight within a radius that depends on the time of day, at
night on the moon, and on the [weather](weather.md): rain shortens it. Light, your eyes' threshold,
and darkvision play no part. Mountain ranges and
dense forest block sight; ordinary forest does not.

{{#include generated/overworld-sight.md}}

At night the overworld is drawn by a faint cold light so the cells in sight stand out from the dim
remembered map, with the moonlight on top of it. See [Sun, moon, and sky](sun.md).

## Locations

Some cells are **locations**: ways into other levels such as a hall, a cave, or a dungeon. Walking
onto one tells you where it leads, and `>` enters it. You arrive on the stairs that lead back, and
`<` there takes you out to the overworld at the location.

## Local areas

On any other cell whose terrain has a local area (the "Local area" column above), press `>` to
explore that cell on foot. You enter an outdoor area, 80 by 40 cells, made for that cell:

- It is generated the first time you go there from the run's seed and the cell, and stays as you
  left it afterwards.
- What it looks like follows the terrain of the cell and its neighbors: meadows, woods with
  clearings, rocky hills with crags, marsh with pools and reeds, roads, riverbanks or lake shores
  along the sides where the overworld has them, and now and then ponds, brooks, ruins, or deer.
  Woods blend towards the neighboring cells so areas side by side fit together.
- You arrive on the side you walked onto the cell from (on the road if you came along one), or in a
  clearing in the middle if you did not walk there.
- Walk off any edge to return to the overworld cell. This takes one 6-second turn.
- Inside, turns are 6 seconds and the sun and sky light the area by day. At night only the moon
  lights it: under a bright moon you dimly make out the open ground, otherwise it is dark, so
  bring your [lantern](lantern.md).
