# Hearing

You hear things you cannot see: footsteps in the dark, a door closing behind a wall. The creatures
around you hear too.

## What makes noise

- **Footsteps.** Every step makes noise, yours and the creatures'. Some creatures are heavier on
  their feet than others, and some (bats, which fly) make no sound at all.
- **Doors.** Opening a door makes noise, and closing one is louder still.

How loud a step is also depends on what you step onto. Wading through water splashes, and
crashing through bushes or over loose rubble is loud. Grass, flowers, moss, and dust soften your
steps. Steps on anything not listed here, such as stone floor, flagstones, and stairs, sound normal.

{{#include generated/footsteps.md}}

The overworld is silent: its steps are journeys, not footsteps.

## How sound travels

A sound spreads out from where it was made and grows fainter with every cell it travels. It goes
around corners through open space, but never through walls or solid rock. A **closed door
muffles** it heavily, about as much as three more cells of distance, so through a closed door you
only hear what happens close behind it. An open door lets sound through freely. Nothing carries
more than 30 cells.

Sound takes the easiest way, so a noise behind a long wall may still reach you around its end.

## What you hear

You only notice sounds whose source you cannot see. If you see the creature walking, you see
what happens, and there is nothing more to hear. So hearing tells you about creatures in the dark,
around corners, and behind doors.

A sound you hear shows on the map as a faint dotted ring, `◌`, until your next turn:

- It marks roughly where the sound came from, not exactly. A loud sound is placed where it was
  made; a clear one up to a cell off; a faint one up to two cells off. A marker is never put on a
  cell you can see.
- Fainter sounds have fainter markers.
- It never tells you what made the sound.
- With glyph animation on, a ring ripples out of the marker when you hear it. With reduced
  effects the marker is simply still; it never disappears, because it is information.

[Look mode](controls.md) (`x`) describes a marker: "You hear footsteps here.", "You faintly hear
footsteps here.", or "You hear footsteps here, where you remember a stone floor."

## Messages

New sounds are written to the message line, with the direction they came from:

```
You hear footsteps to the north-east.
You faintly hear a door close to the west.
```

Only news gets a message. A sound is not news if you heard the same kind of sound near the same
place within the last 10 turns (allowing for whatever made it to have moved a couple of cells each
turn). So a creature walking back and forth behind a door is reported once each time it comes
back into earshot, not on every step.

## Resting

A message stops a [rest](resting.md), so a new sound wakes you. A sound you already know about,
such as footsteps you have been hearing for a few turns, does not.

## Creatures hear you

Your footsteps and the doors you open and close carry to the creatures around you, through the
same rules. Today nothing comes of it: creatures still only walk their patrols.
