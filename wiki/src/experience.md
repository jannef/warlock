# Experience

Killing monsters earns **experience points** (XP), and enough of them make you a higher **level**.
Your level (`Lv`) is in the status bar, and the [character screen](character.md) (`@`) shows it
with your XP and the XP at which the next level comes, like `XP  85 / 767`. Dying shows the level
you had reached.

## What counts

You get XP for the kills you make yourself: with your weapon up close, with a shot, or with
something you throw (see [Combat](combat.md) and [Ranged combat](ranged.md)), for a creature
that bleeds to death from a wound you gave it (see [Status effects](status-effects.md)), and for
[quests](quests.md) you complete. A creature that dies to a trap or to another creature (or to a
wound another creature gave it) gives you nothing, and neither does anything else, such as a kill
you merely watched.

Every kind of creature has a **level** and an **XP value**. A creature is worth about 5 XP per
level plus 10 (the guideline the level curve is written in; the vermin are worth less, and some
are not meant to be fought):

{{#include generated/experience-creatures.md}}

## Discovery

Meeting something new teaches you a little. The first time you kill a kind of creature
yourself, it is worth its XP once more, however far below you it is ("You learn something of
kobolds."), and the first time you arrive on a level you gain a little for finding your way
there ("You gain 10 experience for finding your way here."). Only the first time counts: a
second kobold, or a level you have been on before, is worth nothing more.

{{#include generated/experience-discovery.md}}

## Fewer points for weak monsters

As in World of Warcraft, the further a monster is below you in level, the less its death is worth.
A monster of your own level gives its whole value.

- **Below you**: it gives its XP times one minus the difference in levels over a number that grows
  with your level (the table below); once the monster is that many levels below you, it is
  **grey** and gives nothing. Any kill that is not grey gives at least 1 XP.
- **Above you**: each level it has over you adds 5% to its value, up to four levels (+20%).

{{#include generated/experience-zd.md}}

For a level 7 character (grey five levels down), a monster worth 100 XP at its own level gives:

{{#include generated/experience-scaling.md}}

So a kobold shaman is worth less every level you gain:

{{#include generated/experience-example.md}}

## Levels

The highest level is 60, and the road there is long. Each level costs a number of kills of
creatures of your own level: 38 for level 2, about 160 a level by level 10, and more for every
level after that, up to 365 for level 60. That is about 3,000 such kills to reach level 20 and
15,000 to reach level 60 (real counts run higher: weaker monsters are worth less, and grey ones
nothing). Gaining a level says "You feel more experienced." and "Welcome to level N."; one kill
can bring several levels at once.

Your **race and class** change how much XP every level needs: long-lived races and the patient
callings need more, short-lived races and everyday folk less. The two multiply, so an elf wizard
needs 175 % of the XP below and a half-orc commoner 72 %. Character creation says "Levels come
slowly." or "Levels come quickly." where it matters:

{{#include generated/experience-factors.md}}

The XP needed for each level, for a character of 100 % (a human warrior, say) and for the
quickest and the slowest race and class:

{{#include generated/experience-levels.md}}

## What a level gives

Each level up to 20 raises your maximum **hit points** (and the hit points you have now) by half
your class's hit points plus your Toughness's share, at least 1 (see
[Combat](combat.md#hit-points)). From level 21 on, levels grow no hit points: hit point pools do
not run away. Each level also gives every stat a chance to rise by what you have done with it
since (see [Stats](stats.md#training)). Up to level 30 each level raises your **mana** too, by
half your class's mana plus what the better of your Intelligence and Willpower adds, if that
comes to anything (see [Mana](mana.md)).

Every level also gives **skill rolls**: two for most classes, three for a thief or a ranger, four
for a bard, one more for a human, and more for a high Intelligence. You spend them on your trained
skills before you do anything else; how much each roll adds depends on how much you have used the
skill since its last roll (see [Skills](skills.md#getting-better)).
