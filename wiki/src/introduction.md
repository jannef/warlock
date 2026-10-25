# Warlock

Warlock is a turn-based, grid-based roguelike in the spirit of classic ASCII ADOM. Everything on
screen, from creatures and terrain to the menus, is a glyph, drawn on the GPU with colored light,
real shadows, and gently animated terrain.

This wiki explains how the game works: the rules you play by and the facts about its world. It is
for players. It describes the game as it is today, which is an early build. You can walk and
explore, travel the overworld, watch the days, the moon, and the weather change, hear what you
cannot see, talk to the people you meet, and pick up, carry, wear, and wield objects. Creatures go about their business,
notice you, and fight each other and you: blows and arrows are rolled against your Defence, hurt,
and kill, and death is final (spells are only announced so far). Your stats, your weapon, and
your armour decide how well you fight.

**The numbers here are exact.** The game keeps many of them to itself and tells you in words: a
creature looks "badly wounded", you "feel more skilled", boots make you "move faster". The wiki
gives the rules' own numbers, generated from the game's data, so they cannot drift: your bonus
to hit and its parts, the stat modifiers, speeds, how loud things are, what a rank gives, and
every creature's stats, attacks, and senses ([Creatures in numbers](creature-numbers.md)).

- **Play in your browser:** <https://jannef.github.io/warlock/> (needs a browser with WebGPU)
- **Download for Windows, macOS, and Linux:** <https://github.com/jannef/warlock/releases>
- **Source and issues:** <https://github.com/jannef/warlock>

## Where to start

- [What's in the game](features.md) is the overview: every feature in a paragraph, with
  screenshots.
- [Controls](controls.md) lists the keys and explains the screen. In the game, `?` shows the
  up-to-date list. [Settings](settings.md) covers font size, colors, and effects.
- [Time and the calendar](time.md), [Turns and movement](turns.md), and [Resting](resting.md)
  explain how time passes.
- [Light](light.md), [Sight and memory](sight.md), [Vision types](vision.md), and
  [Hearing](hearing.md) explain what you can see and hear, and why.
- [The Ascmearc](ascmearc.md) is the world you play in. [Sun, moon, and sky](sun.md),
  [Weather](weather.md), and [The overworld](overworld.md) explain the outdoors.
- [Your character](character.md), with its [Races](races.md), [Classes](classes.md), and
  [Horoscope](horoscope.md), and [Stats](stats.md) explain who you are; [Combat](combat.md) how
  fights go, and how you die.
- [Terrain](terrain.md), [Special rooms](special-rooms.md), [Objects and your
  pack](objects.md), [Creatures](creatures.md), [People and talk](npcs.md), [Your
  lantern](lantern.md), and [Saving](saving.md) cover the rest.

## Starting a run

The game opens on the main menu. **New run** starts a new character in the Ascmearc, the
world you play in (see [Saving](saving.md) for the characters you keep). You create your character: your race, your gender,
and your class, one step at a time, and a summary of your age, birth sign, and the stats you
will start with (see [Your character](character.md)). You set out with your
[starting kit](starting-kit.md):

{{#include generated/scenarios.md}}

Tables in this wiki that list names, times, or numbers are generated from the game's own data, so
they match the version of the game this wiki was published with.

**The Ascmearc**, "the ash march", is the first world. You come down into it through
**Beorggeat**, a pass in the mountains of its western edge; snow closes the pass behind you, so
there is no way back. The road runs east to **Cēaptūn**, a market town an easy morning's walk
away (see [The Ascmearc](ascmearc.md)).

Apart from the land itself and its places, this wiki does not describe levels or walk you
through them: the game's world is still being built.
