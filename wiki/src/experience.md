# Experience

Killing monsters earns **experience points** (XP), and enough of them make you a higher **level**.
Your level (`Lv`) is in the status bar, and the [character screen](character.md) (`@`) shows it
with your XP and the XP at which the next level comes, like `XP  85 / 120`. Dying shows the level
you had reached.

## What counts

You get XP for the kills you make yourself: with your weapon up close, with a shot, or with
something you throw (see [Combat](combat.md) and [Ranged combat](ranged.md)), for a creature
that bleeds to death from a wound you gave it (see [Status effects](status-effects.md)), and for
[quests](quests.md) you complete. A creature that dies to a trap or to another creature (or to a
wound another creature gave it) gives you nothing, and neither does anything else, such as a kill
you merely watched.

Every kind of creature has a **level** and an **XP value**:

{{#include generated/experience-creatures.md}}

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

The XP you need grows roughly with the square of the level, and the highest level is 50. Gaining
a level says "You feel more experienced." and "Welcome to level N."; one kill can bring several
levels at once.

{{#include generated/experience-levels.md}}

## What a level gives

Each level up to 20 raises your maximum **hit points** (and the hit points you have now) by half
your class's hit points plus your Toughness's share, at least 1 (see
[Combat](combat.md#hit-points)). From level 21 on, levels grow no hit points: hit point pools do
not run away. Up to level 30 each level raises your **mana** too, by half your class's mana plus
what the better of your Intelligence and Willpower adds, if that comes to anything (see
[Mana](mana.md)). Skills and the rest do not depend on your level.
