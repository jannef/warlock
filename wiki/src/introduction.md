# Warlock

Warlock is a turn-based roguelike played on a grid, in the spirit of the classic ASCII game
ADOM. Everything on screen is a glyph: creatures, terrain, and the menus too. The glyphs are drawn
on the GPU with colored light, real shadows, and gently animated terrain.

This wiki explains how the game works: the rules you play by and the facts of its world. It is
written for players, and it describes the game as it is today, which is an early build.

In this build you can walk and explore, [travel the overworld](overworld.md#travel), and watch
the [days](time.md#days), the [moon](sun.md#moonlight), and the
[weather](weather.md#how-the-weather-changes) change. You [hear](hearing.md) what you cannot
see. You [talk to the people](npcs.md#chatting) you meet. You pick up and carry
[objects](objects.md), [wear](equipment.md#what-fits-where) them, and
[wield](equipment.md#what-fits-where) them.

[Creatures](creatures.md) go about their business, notice you, and fight each other and you.
Blows and arrows are rolled against your [Defence](combat.md#defence). They hurt, they kill, and
[death](hit-points.md#death) is final. Spells work the same way, whether a creature casts them or
you do; they cost [mana](mana.md#how-much) (see [Magic](magic.md)). How well you fight depends on
your [stats](stats.md), your weapon, and your [armour](equipment.md#what-fits-where).

**The numbers here are exact.** The game keeps many numbers to itself and tells you in words
instead: a creature looks "badly wounded", you "feel more skilled", boots make you "move
faster". The wiki gives the real numbers, generated from the game's data so they cannot drift.
That includes your bonus [to hit](combat.md#hitting) and where it comes from, the stat
[modifiers](stats.md#modifiers), [speeds](turns.md#speed), [how loud](hearing.md#how-loud)
things are, what a [rank](weapon-skills.md#marks-and-ranks) gives, and every creature's stats,
attacks, and senses ([Creatures in numbers](creature-numbers.md)).

- **Play in your browser:** <https://jannef.github.io/warlock/> (needs a browser with WebGPU)
- **Download for Windows, macOS, and Linux:** <https://github.com/jannef/warlock/releases>
- **Source and issues:** <https://github.com/jannef/warlock>

## Where to start

- [What's in the game](features.md) is the overview, with screenshots.
- [Controls](controls.md) lists the keys and explains the screen. In the game, `?` shows the
  up-to-date list. [Settings](settings.md) covers font size, colors, and effects.
- [Time and the calendar](time.md), [Turns and movement](turns.md), and [Resting](resting.md)
  explain how time passes.
- [Light](light.md), [Sight and memory](sight.md), [Vision types](vision.md), and
  [Hearing](hearing.md) explain what you can see and hear, and why.
- [The Ascmearc](ascmearc.md) is the world you play in. [Sun, moon, and sky](sun.md),
  [Weather](weather.md), and [The overworld](overworld.md) explain the outdoors.
- [Your character](character.md), [Races](races.md), [Classes](classes.md),
  [Horoscope](horoscope.md), and [Stats](stats.md) explain who you are. [Combat](combat.md)
  explains how fights go, and how you die.
- [Terrain](terrain.md), [Special rooms](special-rooms.md), [Objects and your
  pack](objects.md), [Creatures](creatures.md), [People and talk](npcs.md), [Your
  lantern](lantern.md), and [Saving](saving.md) cover the rest.

## Starting a run

The game opens on the main menu. **New run** starts a new character in [the Ascmearc](ascmearc.md),
the world you play in. (See [Saving](saving.md) for the characters you keep.) You make your
character one step at a time: your [race](races.md), your [gender](character.md#gender), and your
[class](classes.md). A summary then shows your [age and birth sign](character.md#age-birthday-and-sign)
and the [stats](stats.md) you will start with (see [Your character](character.md)). You set out
with your [starting kit](starting-kit.md):

{{#include generated/scenarios.md}}

Tables in this wiki that list names, times, or numbers are generated from the game's own data, so
they match the version of the game this wiki was published with.

**The Ascmearc**, "the ash march", is the first world. You come down into it through
**[Beorggeat](ascmearc.md#the-places)**, a pass in the mountains on its western edge. Snow closes
the pass behind you, so there is no way back. The road runs east to
**[Cēaptūn](ascmearc.md#the-places)**, a market town an easy morning's walk away (see [The
Ascmearc](ascmearc.md)).

Apart from the land and its places, this wiki does not describe levels or walk you through them.
The game's world is still being built.
