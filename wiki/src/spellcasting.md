# How spells work

Every spell, whether you cast it or a creature does, follows the same rules. Its **shape** decides
what it covers. Its **resolution** decides whether it rolls to hit. Its **damage type** adds a rule
of its own: fire burns, cold slows, lightning jumps (see [Spell types](spell-types.md)). And its
**ladders** make it stronger as your stats grow. A few spells follow a rule no other spell does
(see [Spells with a rule of their own](spell-rules.md), where zones are too). For the spells
themselves, their levels, and their costs, see [Magic](magic.md).

## Casting a spell

Some classes give spells: the [priest](class-priest.md), [druid](class-druid.md),
[wizard](class-wizard.md), and [witch](class-witch.md) a few, a [bard](class-bard.md), a
[paladin](class-paladin.md), or a [ranger](class-ranger.md) one or two (see
[What your class gives](magic.md#what-your-class-gives)). You cast them from the
[abilities menu](abilities.md#the-abilities-menu) as **spells**. Each costs
[mana](mana.md#spending-it) by its [spell level](magic.md#spell-levels) and your armour. It is
greyed out ("too little") while you have less mana; there is no other wait.

- A [burst](#shapes) flies and goes off as a [scroll](scrolls.md)'s does.
- A [line](#shapes) runs through everyone on it; its blows and kills are yours.
- A [hex](#shapes) falls on a [hostile](creatures.md#factions) you see within its reach.

## Shapes

A spell strikes one target (a ray, a bolt, a touch, or yourself) or an area (a cone, a line, or a
burst). Areas cost more [mana](mana.md#spending-it): the base cost is multiplied by the shape's
share.

{{#include generated/spell-shapes.md}}

- A **ray** or a **bolt** flies along the line toward the place you aim at, and strikes the first
  creature in its way. A [hex](magic.md#spell-hex-of-weakness) is cast the same way at a hostile
  creature you see, and falls on it.
- A **line** runs through everyone on it, as far as its length, until a wall stops it.
- A **cone** fans out from you toward where you aim, a quarter turn wide, as far as its length.
- A **burst** flies toward where you aim and goes off on the first creature in its way (or at the
  spot you aimed at). It strikes everyone it reaches within its radius, half as hard at its edge. It
  can catch you too.
- A **touch** reaches only a creature next to you.

How far a spell reaches is in its row, and so is how much further your
**[Willpower](stats.md#stat-willpower)** takes it. A [hex](magic.md#spell-hex-of-weakness)'s or a
[blessing](magic.md#spell-blessing)'s row says how long your **[Presence](stats.md#stat-presence)**
makes it last. Each degree lasts as long again, up to the maximum its
[status](status-effects.md#how-they-work) allows. A burst's cost depends on its base radius.

## How a spell finds its target

Each spell has one of four resolutions:

- An **attack** spell rolls [to hit](combat.md#hitting) like a
  [missile](ranged.md#shooting-and-throwing): 2d12 plus your [Dexterity](stats.md#stat-dexterity)'s
  modifier, against the target's [Defence](combat.md#defence). It can miss, it can be a
  [critical hit](combat.md#criticals), and it does full damage. (A spell may name another stat to
  aim with, but none does yet. A spell scroll or a wand always aims with Dexterity.)
- An **auto** spell always hits, for less damage.
- An **area** spell hits everyone in its shape with no roll, for less again per target.
  *[Evasion](talents.md#talent-evasion)* halves what it does.
- A **touch** spell rolls to hit at arm's length. It does the most damage, for less mana.

Only attack and touch spells can be critical hits. Like a blow, they threaten a
[critical](combat.md#criticals) on a natural 22 or more, and a critical rolls the damage again.

- [Magic items](magic-items.md) *of the Magus* add their number to your own spells' roll to hit.
- *Baleful* items widen the threat range by their number (a Baleful +2 threatens from 20), never
  below 2.
- Neither helps a spell scroll you read or a wand you use.
- Your [caster style](schools.md#caster-styles) adds to the roll when you cast the spell yourself.

[Resistances](damage-types.md#armour-and-resistance) take their share of any spell's damage.
[Armour](objects.md#class-body-armour) takes none.

What each resolution does to a spell's cost:

{{#include generated/spell-resolutions.md}}

## How a spell grows

Each spell's row lists its ladders: a stat, and the scores at which the spell changes. When your
score reaches a number, you cast at that rung. The score is the one your character screen shows,
with what your [status effects](status-effects.md), [stat damage](status-effects.md), and
[magic items](magic-items.md) do to it. A low score never shrinks a spell below its base.

Spell damage is rolled dice, as written in the spell's row. What makes it grow is its ladder, plus
your talents and items. Your [talents](talents.md) and [magic items](magic-items.md) each add in
their own way, listed in their own row:

- **A rung.** *[Empower](class-wizard.md#talent-empower)* (a [wizard](class-wizard.md)'s, at level 32)
  takes your spells a rung up their damage and healing ladders, and
  *[Archmage](class-wizard.md#talent-archmage)* (at 60) two. Spells already on the last rung stay
  there.
- **Bigger dice.** *[Keen mind](talents.md#talent-keen-mind)*, taken for a school of magic or an
  element, makes those spells roll dice a size bigger for you: a d6 becomes a d7, a d8 a d9.
- **A die more.** *[Brimming](talents.md#talent-brimming)* rolls a die more on your damage and
  healing spells, and they cost a tenth more [mana](mana.md#spending-it).
- **A bonus.** *Potent* [magic items](affixes.md#prefixes) add 1 to 3 to every spell's roll,
  harm or healing (a Potent +2 ring makes 4d6 into 4d6+2).
- **A die rolled again.** *[Sure hand](talents.md#talent-sure-hand)* rolls a spell die that shows a
  1 once more, and that roll stands.
- **Reach and degrees.** *[School focus](talents.md#talent-school-focus)*, taken for a school or an
  element, makes those spells reach a cell farther (range or length, not a burst's radius, and
  never a touch's) and cost 10 % less. [Magic items](affixes.md#suffixes) *of Farcasting* add their cells to every spell's
  reach but a touch's, and *of Lingering* their degrees to every blessing and hex.
- **The spell's rank.** Reading a spell's grimoire again deepens it, and each rank adds what the
  spell's own table says, in any of these ways and a few more (the aim, the critical range, the
  casting time, the mana): see [Spell ranks](learning-spells.md#spell-ranks).

They apply in this order: the rung, then the dice, then the roll (with its rerolls), then the harm,
then the extra effects. A [critical](combat.md#criticals) rolls the dice twice, as a blow's does.

A **[spell scroll](scrolls.md#spell-scrolls)** you read casts at the rung its row gives, whatever
your stats. The [scroll of fireball](scrolls.md#scroll-of-fireball)'s is 1, so 6d6.
*[Scroll adept](class-wizard.md#talent-scroll-adept)* makes it a rung higher. A
**[wand](wands-and-rods.md#wands)** casts at a rung of its own, a rung higher with
*[Wand mastery](talents.md#talent-wand-mastery)*, and higher again with items *of Wands*. Your
other talents and items work on scrolls and wands as on your own casts, and their reach and degrees
are yours.

Most of these talents belong to Spellcraft, the tree a [wizard](class-wizard.md) takes bonus
talents from, alongside Scholar. See [Talents](talents.md) for every talent that affects spells.

Two kinds of damage only magic does, **[force](damage-types.md#damage-force)** and
**[necrotic](damage-types.md#damage-necrotic)**, are among the [damage types](damage-types.md).

The spells you cast are yours. What they kill earns you its [experience](experience.md), and a spell
of your own that catches you can kill you.
