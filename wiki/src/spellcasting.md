# How spells work

Every spell, whether you cast it or a creature does, follows the same rules. Its **shape** decides
what it covers. Its **resolution** decides whether it rolls to hit. Its **damage type** adds a rule
of its own: fire burns, cold slows, lightning jumps. And its **ladders** make it stronger as your
stats grow. This page explains each in turn. For the spells themselves, their levels, and their
costs, see [Magic](magic.md).

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
- Your rank in the spell's [school](schools.md) and your [caster style](schools.md#caster-styles)
  add to the roll when you cast the spell yourself.

[Resistances](damage-types.md#armour-and-resistance) take their share of any spell's damage.
[Armour](objects.md#class-body-armour) takes none.

What each resolution does to a spell's cost:

{{#include generated/spell-resolutions.md}}

## Spell types

Every spell of a [damage type](damage-types.md) carries that type's rule, whether it is cast from
[mana](mana.md#spending-it), a [scroll](magic.md#learning-spells-from-scrolls), or a
[wand](wands-and-rods.md#wands). The rules belong to magic alone: a
[Flaming](affixes.md#prefixes) sword's blow follows none of them. They spare no one: you and
your [allies](allies.md) are struck like anyone else the spell reaches.

{{#include generated/spell-types.md}}

**Fire catches.** One creature in four that a fire spell harms is left
[burning](status-effects.md#burning), whatever the spell. A [Fireball](magic.md#the-spells) on five
kobolds leaves one or two alight. A creature that takes no harm from it never burns.

A fire burst melts the ice on the cells it reaches, and whoever stands on that ice
[falls in](temperature.md#water-freezes). It leaves a fire on the cell at its centre, which gives
light for a minute. That fire does nothing else, and it never spreads.

**Cold slows the blood.** Every creature a cold spell harms is also
[Sluggish](status-effects.md#status-sluggish), and anything burning on it goes out. A miss does
nothing. A cold burst freezes the [water](terrain.md#water) on the cells it reaches into ice that
holds a laden person, and puts out the fires there.

**Lightning seeks the nearest path.** When a ray, a bolt, a touch, or a line of electricity harms a
creature, one time in four it strikes once more. It hits the nearest creature beside that one that
has not been struck yet (ties go to the one that came into the world first), at the same roll.
Neither a roll to hit nor [Evasion](talents.md#talent-evasion) saves anyone from it. That creature
may arc on again, and so on. It is not choosy: [Spark](magic.md#the-spells) at a rat beside you may
jump back into you, or into your [ally](allies.md).

**Water carries it.** A creature harmed while standing in [water](terrain.md#water) is part of that
whole body of water: every cell of open water connected to it, diagonals too. Everyone standing in
that body who has not been struck yet is struck at the same roll, you too if you are wading. They
are struck one after another, in the order they came into the world, and each of them may arc.
Every electricity spell does this, whatever its shape.

[Ice](temperature.md#water-freezes) is not water. It carries nothing, and a frozen band splits a
pond in two. Each creature only ever takes one hit from a cast. Aiming shows you none of this.

**Chain lightning** takes its first jumps for certain: three at first, and up to six as your
[Willpower](stats.md#stat-willpower) grows. Any of its jumps reaches two cells instead of one, as
long as nothing that blocks sight stands between. After those, a jump happens one time in four as
usual. A chain ends when no creature that has not been struck is within reach of the last one, or
when the last one took no harm.

**[Conductor](talents.md#talent-conductor)** makes every arc of your electricity spells happen, and
you resist electricity 25 %. It spares you nothing; it only makes your own lightning bite less.

**A ray, a bolt, or a touch** does to one cell what a burst does to all of its cells: to the cell
of the creature it strikes, or else the cell you aimed at. So [Ray of frost](magic.md#the-spells)
builds a bridge across a lake, one cell per cast. A line does it to the last cell it reaches; a
cone does not do it at all.

**Force pushes.** A force spell that takes at least a third of a creature's
[hit points](hit-points.md#hit-points), or lands a [critical](combat.md#criticals), knocks it a cell
away. It is pushed away from the centre of a burst, or otherwise away from you (and away from you
too if it is the one at the burst's centre). A cell that is not free stops the push, and so does
[being held or webbed](status-effects.md#status-held). You do not move when you are at the centre of
your own burst, though you take the harm.

A force burst with a blast breaks the [shut doors](doors.md) it reaches, however sturdy, locked or
trapped. A trap on the door goes with it and strikes no one. [Chests](chests.md) break under any
blast below their hardness. A force blast also shatters thin [ice](temperature.md#water-freezes),
and whoever stands on that ice falls in. A fire blast leaves doors standing.

**Poison lingers.** Every creature a poison spell harms is [Poisoned](status-effects.md#poisoned)
one degree, on top of whatever else the spell does.

**Death takes the dying first.** A necrotic spell does half again as much, rounded down, to a
creature below half its hit points, before what it resists. The undead are immune to necrotic harm.

**Acid keeps eating.** Every creature an acid spell harms is
[Corroded](status-effects.md#corroded) one degree: two more hit points over the next twelve seconds.
Acid that harms you also eats at what you wear: the first piece (your body armour first) is
[damaged](materials.md#item-damage) unless its material withstands acid by its chance, and a piece already
damaged is eaten away. Acid eats no creature's gear.

### What lies in its way

An area spell (a burst, a cone, or a line; see [shapes](#shapes)) with a
[damage type](damage-types.md) may destroy [items lying on the floor](objects.md#piles) it reaches,
whether it is cast from [mana](mana.md#spending-it), a
[scroll](magic.md#learning-spells-from-scrolls), or a [wand](wands-and-rods.md#wands). So may a
dragon's breath. A [Fireball](magic.md#the-spells) can burn the scrolls it lands on, and a blast of
[Frost](magic.md#the-spells) can shatter potions.

Each item in each stack there is lost at the chance given by the first row below that fits it, for
that spell type. A row fits by the item's [item class](objects.md#item-classes) or the
[material](materials.md#materials) of its main part. So a stack may lose some of its items or all
of them. A stack that no row fits is never touched.

Some things are never touched:

- what you carry or wear;
- what lies in a [chest](chests.md);
- what is [protected from item damage](materials.md#protected-items).

[Traps](traps.md) ruin nothing. A [corpse](food.md) that burns away never rises. You are told what
you see go: "The scroll burns up.", "3 potions freeze and shatter."

{{#include generated/spell-ruin.md}}

## Spells with a rule of their own

Some [spells](magic.md#the-spells) do something no other spell does. None of them spares you or your
[allies](allies.md).

- **Thunderclap** and **Benison** go off around you. They need no aim, and you are at their centre.
  - Thunderclap strikes you with the full roll, as it does everyone at a burst's centre, and leaves
    you [Stunned](status-effects.md#status-stunned) along with the rest. The whole level hears it.
  - Benison's one roll heals you and every ally of yours within it who is hurt. It does nothing to
    anyone else, and it is not cast when no one there is hurt.
- **Leech** gives you back half the [hit points](hit-points.md#hit-points) it takes, rounded down,
  never past your maximum. The undead are immune to [necrotic](#spell-types) harm, so they give
  nothing.
- **Rebuke** reaches only the undead, yours and others'. The living in it are not touched. It is
  fire, so it [burns](status-effects.md#burning) as fire does and leaves a fire at its centre.
- **Bramble** only catches those standing where things grow: grass, marsh, reeds, flowers, bushes,
  forest floor, fungus (the [terrain](terrain.md) table says which). On stone it does nothing.
- **Gale** drives everyone on its line one cell straight away from you, the nearest first. Anyone
  with someone behind them stays put, and so does anyone [held or webbed](status-effects.md#status-held).
  It harms no one. It puts out the fires on its line and tears away the [zones](#zones) of fire,
  foul cloud, darkness, and smoke on it; slick ground stays.
- **Flare** [stuns](status-effects.md#status-stunned) everyone it reaches who has eyes. The eyeless
  (worms, the black pudding, the canoloth) take nothing. A shut door stops it.
- **Bind** holds one hostile creature you see where it stands. It is
  [Pinned](status-effects.md#status-pinned) two degrees (more as your Presence grows), so it cannot
  step, though it may still strike. A pin eases a degree with each action it takes. Bind never
  misses. A [wand of bind](wands-and-rods.md#wands) casts it. (**Shatter**'s
  [wand](wands-and-rods.md#wands) is a [force](#spell-types) burst, like the spell.)
- **Blink** takes you to a free cell within 5, each equally likely, through walls or not. It works
  while you are [held](status-effects.md#status-held), [webbed](status-effects.md#status-webbed), or
  [pinned](status-effects.md#status-pinned), and ends all three.
- **Sense the quick** lets you feel every living creature within 8 cells that you cannot see, as a
  heartbeat. You [hear](hearing.md) it within 2 cells of where the creature is, every turn while the
  spell lasts. The undead, constructs, and animated things have no heartbeat.
- **Cloudburst**, cast under the open sky, brings an hour of [rain](weather.md). It at once puts
  out every fire, and everyone [burning](status-effects.md#burning), under the open sky on the
  level.

## Zones

A **zone** is an area that lies on the floor for a while. [Balefire](magic.md#the-spells) burns,
[Reek](magic.md#the-spells) poisons, [Mirk](magic.md#the-spells) darkens,
[Slick](magic.md#the-spells) trips, and [Fume](magic.md#the-spells) smokes. All five come only
from [scrolls](magic.md#learning-spells-from-scrolls) you find (and a flask of smoke lays Fume's).

- A zone's cells stay as they were laid, and it never spreads.
- It takes you and your [allies](allies.md) like anyone else. The caster has no protection from
  their own zone, though a kill it makes is theirs.
- A zone laid over another replaces it on the cells they share.
- It stays when you leave the level, and its time runs while you are away.
- No zone can be laid on the [overworld](overworld.md).

| Kind | Who it takes | What it does |
|---|---|---|
| Fire | whoever starts a turn on it or steps onto it | the spell's [fire](#spell-types) harm, one number rolled when it is laid and the same for all its time, halved by [Evasion](talents.md#talent-evasion); then fire's one chance in four of [burning](status-effects.md#burning). Each cell sheds a fire's light, and the [ice](temperature.md#water-freezes) under it melts as it is laid. It lies only on walkable ground out of the water |
| Foul cloud | whoever starts a turn in it | [Poisoned](status-effects.md#poisoned) one degree more, no other harm; nothing to the [poison-immune](damage-types.md). It does not block sight |
| Slick ground | whoever steps onto it or off it | a [Dexterity](stats.md#stat-dexterity) check against 12. Failed: [Staggered](status-effects.md#status-staggered), the next action half again as slow. The step is still taken. Nothing incorporeal slips. It lies only on walkable ground out of the water |
| Smoke | no one directly | no eye sees into it or through it, yours and every creature's: your sight stops at its edge as at a wall, and whoever stands in it is seen only from a cell beside them. From inside, you see only the cells beside you. Light, missiles, and spells pass through it. [Gale](#spells-with-a-rule-of-their-own) tears it away; rain does not |
| Darkness | no one directly | no light reaches its cells, the sun's included, and a light inside sheds none, a burning creature's too. Eyes that need light see nothing inside; [darkvision](vision.md#the-four-types) sees in as far as it reaches. Your own eyes follow the same rule: from inside, you see only the cell you stand on and what is lit beyond |

A [cold](#spell-types) spell puts out the fire on each cell it reaches, and the rest of the zone
burns on. A zone shows as a still tint over its cells, and look mode names it.

## How a spell grows

Each spell's row lists its ladders: a stat, and the scores at which the spell changes. When your
score reaches a number, you cast at that rung. The score is the one your character screen shows,
with what your [status effects](status-effects.md), [stat damage](status-effects.md), and
[magic items](magic-items.md) do to it. A low score never shrinks a spell below its base.

Spell damage is rolled dice, as written in the spell's row. What makes it grow is its ladder, plus
your talents and items. Your [talents](talents.md) and [magic items](magic-items.md) each add in
their own way, listed in their own row:

- **A rung.** *[Empower](talents.md#talent-empower)* (a [wizard](classes.md#wizard)'s, at level 32)
  takes your spells a rung up their damage and healing ladders, and
  *[Archmage](talents.md#talent-archmage)* (at 60) two. Spells already on the last rung stay
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
  element, makes those spells reach a cell farther (range or length, not a burst's radius) and cost
  10 % less. [Magic items](affixes.md#suffixes) *of Farcasting* add their cells to every spell's
  reach, and *of Lingering* their degrees to every blessing and hex.

They apply in this order: the rung, then the dice, then the roll (with its rerolls), then the harm,
then the extra effects. A [critical](combat.md#criticals) rolls the dice twice, as a blow's does.

A **[spell scroll](scrolls.md#spell-scrolls)** you read casts at the rung its row gives, whatever
your stats. The [scroll of fireball](scrolls.md#scroll-of-fireball)'s is 1, so 6d6.
*[Scroll adept](talents.md#talent-scroll-adept)* makes it a rung higher. A
**[wand](wands-and-rods.md#wands)** casts at a rung of its own, a rung higher with
*[Wand mastery](talents.md#talent-wand-mastery)*, and higher again with items *of Wands*. Your
other talents and items work on scrolls and wands as on your own casts, and their reach and degrees
are yours.

Most of these talents belong to Spellcraft, the tree a [wizard](classes.md#wizard) takes bonus
talents from, alongside Scholar. See [Talents](talents.md) for every talent that affects spells.

Two kinds of damage only magic does, **[force](damage-types.md#damage-force)** and
**[necrotic](damage-types.md#damage-necrotic)**, are among the [damage types](damage-types.md).

The spells you cast are yours. What they kill earns you its [experience](experience.md), and a spell
of your own that catches you can kill you.
