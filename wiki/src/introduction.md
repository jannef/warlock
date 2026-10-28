# Warlock

Warlock is a turn-based, grid-based roguelike in the spirit of classic ASCII ADOM. Everything on
screen, from creatures and terrain to the menus, is a glyph, drawn on the GPU with colored light,
real shadows, and gently animated terrain.

This wiki explains how the game works: the rules you play by and the facts about its world. It is
for players. It describes the game as it is today, which is an early build. You can walk and
explore, [travel the overworld](overworld.md#travel), watch the [days](time.md#days), the [moon](sun.md#moonlight), and the [weather](weather.md#how-the-weather-changes) change, [hear](hearing.md) what you
cannot see, [talk to the people](npcs.md#chatting) you meet, and pick up, carry, [wear](equipment.md#what-fits-where), and [wield](equipment.md#what-fits-where) [objects](objects.md). [Creatures](creatures.md) go about their business,
notice you, and fight each other and you: blows and arrows are rolled against your [Defence](combat.md#defence), hurt,
and kill, and [death](combat.md#death) is final (a creature's [spells](magic.md) are only announced so far; yours are cast
for [mana](mana.md#how-much), see [Magic](magic.md)). Your [stats](stats.md), your weapon, and your [armour](equipment.md#what-fits-where) decide how well you
fight.

**The numbers here are exact.** The game keeps many of them to itself and tells you in words: a
creature looks "badly wounded", you "feel more skilled", boots make you "move faster". The wiki
gives the rules' own numbers, generated from the game's data, so they cannot drift: your bonus
[to hit](combat.md#hitting) and its parts, the stat [modifiers](stats.md#modifiers), [speeds](turns.md#speed), [how loud](hearing.md#how-loud) things are, what a [rank](weapon-skills.md#marks-and-ranks) gives, and
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

The game opens on the main menu. **New run** starts a new character in [the Ascmearc](ascmearc.md), the
world you play in (see [Saving](saving.md) for the characters you keep). You create your character: your [race](races.md), your [gender](character.md#gender),
and your [class](classes.md), one step at a time, and a summary of your [age, birth sign](character.md#age-birthday-and-sign), and the [stats](stats.md) you
will start with (see [Your character](character.md)). You set out with your
[starting kit](starting-kit.md):

{{#include generated/scenarios.md}}

Tables in this wiki that list names, times, or numbers are generated from the game's own data, so
they match the version of the game this wiki was published with.

**The Ascmearc**, "the ash march", is the first world. You come down into it through
**[Beorggeat](ascmearc.md#the-places)**, a pass in the mountains of its western edge; snow closes the pass behind you, so
there is no way back. The road runs east to **[Cēaptūn](ascmearc.md#the-places)**, a market town an easy morning's walk
away (see [The Ascmearc](ascmearc.md)).

Apart from the land itself and its places, this wiki does not describe levels or walk you
through them: the game's world is still being built.
