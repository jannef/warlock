# Sight and memory

## Line of sight

You see in straight lines, and line of sight works both ways: if you have a clear line to a cell,
anything standing there has a clear line to you. Walls and closed doors block sight; open doors and most floor do not. Some terrain
blocks sight too, such as trees, boulders, and crags (see [Terrain](terrain.md)).

The farthest you can see is 40 cells, and the edge of any radius is slightly rounded, so a cell
counts as within radius `r` when its squared distance is at most `r² + r`.

## When you see a cell

A cell is visible when it is in line of sight, within 40 cells, **and** one of these is true:

- it is lit brightly enough for your eyes (your [vision type](vision.md) sets how bright), or
- it is close enough to be seen with darkvision.

The cell you stand on is always visible.

You see a creature when you see its cell. So a creature in the dark is hidden from you unless it is
within your darkvision, and a creature carrying a light shows up. You may still hear one you cannot
see: see [Hearing](hearing.md). Objects lying on a cell are seen the same way as creatures (see
[Objects and your pack](objects.md)).

If you see floor through a narrow gap, you also see the walls around that floor, so a lit passage
never appears without its walls. Such walls show only the terrain itself: no creatures or objects,
and nothing beyond them. In total darkness this does not apply to your own cell, so you then see only
yourself.

## Memory

The game remembers the terrain you have seen, per level, as you last saw it, and the piles of
objects lying on it. Remembered cells are drawn dim and desaturated. A door you saw open is
remembered open until you see it closed. A pile is remembered as it lay when you last saw it, even
if it changed since, and forgotten once you see it gone.

Memory holds terrain and objects only. You never remember creatures, effects, or the light that was there.
When you look away, a creature you saw drops off the map, and the light on remembered cells is
gone: they are simply dimmed. The faint `◌` markers of [sounds you heard](hearing.md) last only
until your next turn.

## The overworld is different

On the overworld sight follows the time of day, not light. See [The overworld](overworld.md).
