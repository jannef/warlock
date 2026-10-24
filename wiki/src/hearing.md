# Hearing

You hear things you cannot see: footsteps in the dark, a door closing behind a wall. The creatures
around you hear too.

## What makes noise

- **Footsteps.** Every step makes noise, yours and the creatures'. Some creatures are heavier on
  their feet than others, and some (bats, which fly) make no sound at all. A heavy load makes
  steps louder: a quarter louder when burdened, half again as loud when strained (see
  [Burden](burden.md)).
- **Doors.** Opening a door makes noise, and closing one is louder still. Kicking a door is
  louder than anything: it bangs on the door itself, heard on both sides as banging, whether the
  door holds or breaks (see [Doors](doors.md)).
- **Stealth.** The [stealth](skills.md#what-skills-give) skill makes your footsteps and the doors
  you open and close quieter, more the higher it is; your attacks are as loud as ever.
- **Fighting.** Attacks are noisy: you may hear fighting, a bowstring, or chanting (a spell)
  where you cannot see it. A throw sounds like fighting, but quieter. See
  [Creatures](creatures.md).
- **Digging.** Digging into a wall is loud, louder than fighting (see
  [Digging and walls](digging.md)).
- **Traps.** Some traps are loud when they go off: an alarm's bell carries farther than anything,
  a fireball explodes, lightning cracks like thunder, and an arrow trap twangs like a bowstring
  ("You hear an explosion to the west."). See [Traps](traps.md).
- **Voices.** People muttering to themselves can be heard a few cells away. If you hear one you
  cannot see, you hear a voice, and its words unless it was faint: "You hear a voice to the
  north: "Weeds. Always the weeds."" See [People and talk](npcs.md#muttering).

How loud a step is also depends on what you step onto. Wading through water splashes, and
crashing through bushes or over loose rubble is loud. Grass, flowers, the forest floor, a dirt
road, and dust soften your steps. Steps on anything not listed here, such as stone floor,
flagstones, and stairs, sound normal (100%). A step's loudness is the walker's footsteps times
the terrain's share, rounded down:

{{#include generated/footsteps.md}}

Where a noise is made matters too: everything is twice as loud in an echoing hall, and much
quieter in a hushed chamber (see [Special rooms](special-rooms.md)).

The overworld is silent: its steps are journeys, not footsteps.

### How loud

The game only ever tells you a sound is faint, clear, or loud. Exactly, every noise has a
**loudness** where it is made; the table says how far you hear each in the open, with the hearing
you start with (the [listen](skills.md#what-skills-give) skill makes it keener). A load makes
your footsteps louder (above, and [Burden](burden.md)), [stealth](skills.md#what-skills-give)
takes its share off your steps and doors, and a special room multiplies everything made in it.
Creatures' footsteps and hearing are on [Creatures in numbers](creature-numbers.md#pace-senses-and-size).

{{#include generated/noises.md}}

## How sound travels

A sound spreads out from where it was made and grows fainter with every cell it travels. It goes
around corners through open space, but never through walls or solid rock. A **closed door
muffles** it heavily, so through a closed door you only hear what happens close behind it. An
open door lets sound through freely.

Sound takes the easiest way, so a noise behind a long wall may still reach you around its end.
Exactly:

{{#include generated/sound-travel.md}}

What muffles sound:

{{#include generated/muffling.md}}

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

[Look mode](controls.md#look-mode) (`x`) describes a marker: "You hear footsteps here.", "You faintly hear
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

Your footsteps, the doors you open and close, and your attacks carry to the creatures around
you, through the same rules. The [listen](skills.md#what-skills-give) skill lets you, or a
creature, hear fainter sounds. A creature with enemies that hears something it cannot see goes to
have a look: walk softly, and mind the doors. See [Creatures](creatures.md).
