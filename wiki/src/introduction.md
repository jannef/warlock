# Warlock

Warlock is a turn-based, grid-based roguelike in the spirit of classic ASCII ADOM. Everything on
screen, from creatures and terrain to the menus, is a glyph, drawn on the GPU with colored light,
real shadows, and gently animated terrain.

This wiki explains how the game works: the rules you play by and the facts about its world. It is
for players. It describes the game as it is today, which is an early build. You can walk and
explore, travel the overworld, watch the days, the moon, and the weather change, hear what you
cannot see, and pick up, carry, wear, and wield objects. Creatures go about their business,
notice you, and fight each other and you: blows and arrows are rolled against your Defence, hurt,
and kill, and death is final (spells are only announced so far). Your stats, your weapon, and
your armour decide how well you fight.

- **Play in your browser:** <https://jannef.github.io/warlock/> (needs a browser with WebGPU)
- **Download for Windows, macOS, and Linux:** <https://github.com/jannef/warlock/releases>
- **Source and issues:** <https://github.com/jannef/warlock>

## Where to start

- [Controls](controls.md) lists the keys and explains the screen. In the game, `?` shows the
  up-to-date list. [Settings](settings.md) covers font size, colors, and effects.
- [Time and the calendar](time.md), [Turns and movement](turns.md), and [Resting](resting.md)
  explain how time passes.
- [Light](light.md), [Sight and memory](sight.md), [Vision types](vision.md), and
  [Hearing](hearing.md) explain what you can see and hear, and why.
- [Sun, moon, and sky](sun.md), [Weather](weather.md), and [The overworld](overworld.md) explain
  the outdoors.
- [Your character](character.md), with its [Races](races.md), [Classes](classes.md), and
  [Horoscope](horoscope.md), and [Stats](stats.md) explain who you are; [Combat](combat.md) how
  fights go, and how you die.
- [Terrain](terrain.md), [Special rooms](special-rooms.md), [Objects and your
  pack](objects.md), [Creatures](creatures.md), [Your lantern](lantern.md), and
  [Saving](saving.md) cover the rest.

## Starting a run

The game opens on the main menu. **New run** lists the scenarios you can start, and takes an
optional seed (see [Saving](saving.md)). Then you create your character: your race, your gender,
and your class, one step at a time, and a summary of your age, birth sign, and the stats you
will start with (see [Your character](character.md)). Each scenario starts you somewhere else in the same world, at
its own time of day:

{{#include generated/scenarios.md}}

Tables in this wiki that list names, times, or numbers are generated from the game's own data, so
they match the version of the game this wiki was published with.

The Showcase is a safe place: its creatures keep to their rounds and leave you alone. The
**Arena**, reached by the stairs down from the showcase hub's stairs nook, is the place to watch
and join fights, the **Quarry**, down another staircase there, the place to
[dig](digging.md), and the generated **Depths** are the real dungeon, where creatures hunt you.

Apart from where the scenarios start, this wiki does not describe places or walk you through
levels: the game's world is still being built.
