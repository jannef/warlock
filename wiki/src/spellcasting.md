# How spells work

INTRO

## Shapes

A spell strikes one target (a ray, a bolt, a touch, or yourself) or an area (a cone, a line, or a
burst). Areas cost more [mana](mana.md#spending-it): the base cost is multiplied by the shape's.

{{#include generated/spell-shapes.md}}

- A **ray** or a **bolt** flies along the line toward the place you aim at and strikes the first
  creature in its way. A [hex](magic.md#spell-hex-of-weakness) is cast the same way at a hostile you see, and falls on it.
- A **line** runs through everyone on it, as far as its length, until a wall stops it.
- A **cone** fans out from you toward where you aim, a quarter turn wide, as far as its length.
- A **burst** flies toward where you aim, goes off on the first creature in its way (or there),
  and strikes everyone within its radius that it reaches, half as hard at its edge. It can catch
  you too.
- A **touch** reaches only a creature next to you.

How far a spell reaches is in its row, and so is how far your **[Willpower](stats.md#stat-willpower)** takes it. A
[hex](magic.md#spell-hex-of-weakness)'s or a [blessing](magic.md#spell-blessing)'s row says how long your **[Presence](stats.md#stat-presence)** makes it last (each degree lasts
as long again, up to the most its [status](status-effects.md#how-they-work) has). A burst costs by its base radius.

## How a spell finds its target

Each spell resolves one way. An **attack** rolls [to hit](combat.md#hitting) like a [missile](ranged.md#shooting-and-throwing) (2d12 plus your [Dexterity](stats.md#stat-dexterity)'s
modifier against the target's [Defence](combat.md#defence); a spell may name another stat to aim with, none
does yet; a spell scroll or a wand always aims with Dexterity), can miss, and can be a [critical hit](combat.md#criticals), and does full damage.
Like a blow, it threatens a [critical](combat.md#criticals) on a natural 22 or more; [magic items](magic-items.md) *of the Magus* add
their number to your own spells' roll to hit, and *Baleful* ones widen their threat by theirs
(a Baleful +2 threatens from 20), never below 2. Neither helps a spell scroll you read or a
wand you use.
Your rank in the spell's [school](schools.md) and your
[caster style](schools.md#caster-styles) add to its roll when you cast it yourself.
An **auto** spell always hits, for less. An **area** spell hits everyone in its shape with no
roll, for less again per target, and *[Evasion](talents.md#talent-evasion)* halves what it does. A **touch** spell rolls to hit
at arm's length and does the most, for less mana. Only attack and touch spells can be [critical
hits](combat.md#criticals); a [critical](combat.md#criticals) rolls the damage again. [Resistances](damage-types.md#armour-and-resistance) take their
share of any spell's damage, and [armour](objects.md#class-body-armour) none.

What each resolution does to a spell's cost:

{{#include generated/spell-resolutions.md}}

## Spell types

Every spell of a [damage type](damage-types.md) carries that type's rule, whether cast from [mana](mana.md#spending-it), a [scroll](magic.md#learning-spells-from-scrolls), or a [wand](wands-and-rods.md#wands). It
is magic's alone: a [Flaming](magic-items.md#prefixes) sword's blow reads no rule. It spares no one: you and your [allies](allies.md) are struck like anyone it reaches.

{{#include generated/spell-types.md}}

**Fire catches.** One creature in four that a fire spell harms is left [burning](status-effects.md#burning), whatever the
spell: a [Fireball](magic.md#the-spells) on five kobolds leaves one or two alight. A creature that takes no harm from it
never burns. A fire burst melts the ice on the cells it reaches, and whoever stands on that ice
[falls in](temperature.md#water-freezes); it leaves a fire on the cell at its centre, which gives light for a minute. No rule reads that
fire, and it never spreads.

**Cold slows the blood.** Every creature a cold spell harms is [Sluggish](status-effects.md#status-sluggish) besides, and whatever burned on it goes
out; nothing on a miss. A cold burst freezes the [water](terrain.md#water) on the cells it reaches, into ice that holds a
laden person, and puts out the fires there.

**Lightning seeks the nearest path.** A ray, a bolt, a touch, or a line of electricity that harms a creature
strikes once more, one time in four, at the nearest creature beside it not yet struck (ties go to the one that
came into the world first), at the same roll, and no roll to hit nor [Evasion](talents.md#talent-evasion) saves anyone from it. That one may arc again, and
so on. It is not choosy: [Spark](magic.md#the-spells) at a rat beside you may jump back into you, or into your [ally](allies.md).

**Water carries it.** A creature harmed while standing in [water](terrain.md#water) is one of its whole body, every
cell of open water connected to it, diagonals too: everyone standing in that body not yet struck is struck at the same
roll, you too if you wade, one after another in the order they came into the world, and each of them may arc. Every
electricity spell does this, whatever its shape. [Ice](temperature.md#water-freezes) is not water: it carries nothing, and a frozen band parts a
pond in two. Each creature only ever takes a cast once. Aiming shows you none of this.

**Chain lightning** takes its first jumps for certain, three at first and up to six as your
[Willpower](stats.md#stat-willpower) grows, and any jump of it reaches two cells instead of one, so long as nothing
that stops sight stands between. Past those, a jump is one time in four as usual. **[Conductor](talents.md#talent-conductor)** makes
every arc of your electricity spells happen, and you resist electricity 25 %: it spares you nothing, it only bites less. A chain ends
when no creature not yet struck is within reach of the last, or the last took no harm.

**A ray, a bolt, or a touch** does to one cell what a burst does to all of its own: the cell of the creature it
strikes, else the cell you aimed at. So [Ray of frost](magic.md#the-spells) makes a bridge across a lake, a cell a cast.
A line does it to the last cell it reaches, a cone not at all.

**Force pushes.** A force spell that takes at least a third of a creature's [hit points](hit-points.md#hit-points), or lands a
[critical](combat.md#criticals), knocks it a cell away: from the centre of a burst, from you otherwise, and from you too for
the one at the burst's centre. A cell that is not free stops it, and so does [being held or webbed](status-effects.md#status-held); you do not
move when you are at the centre of your own burst, though you take the harm. A force burst with a blast
breaks the [shut doors](doors.md) it reaches, however sturdy, locked or trapped, the trap going with the door and
striking no one; [chests](chests.md) break under any blast below their hardness. It also shatters thin [ice](temperature.md#water-freezes), and whoever
stands on that ice falls in. A fire blast leaves doors standing.

**Poison lingers.** Every creature a poison spell harms is [Poisoned](status-effects.md#poisoned) a degree, besides what the spell's own
line says.

**Death takes the dying first.** A necrotic spell does half again as much, rounded down, to a creature below
half its hit points, before what it resists. The undead are immune to necrotic harm.

**Acid keeps eating.** Every creature an acid spell harms is [Corroded](status-effects.md#corroded) a degree: two more hit points in the
next twelve seconds. Acid does not eat armour.

### What lies in its way

An area spell (a burst, a cone, or a line; [shapes](#shapes)) with a [damage type](damage-types.md) may destroy the
[items lying on the floor](objects.md#piles) it reaches, whether cast from [mana](mana.md#spending-it), a
[scroll](magic.md#learning-spells-from-scrolls), or a [wand](wands-and-rods.md#wands), and so may a dragon's breath: a
[Fireball](magic.md#the-spells) can burn the scrolls it lands on, a blast of [Frost](magic.md#the-spells) shatter potions. Every
item of every stack there is lost at the chance of the first row below of the spell's type that fits it, by its
[item class](objects.md#item-classes) or the [material](materials.md#materials) of its main part, so a stack may lose
some of its items or all of them. A stack no row fits is never touched. What you carry or wear, what lies in a
[chest](chests.md), a [unique item](magic-items.md), and anything a [quest](quests.md) asks you to bring or deliver
(until that quest is done for good) are never touched, and [traps](traps.md) ruin nothing. A [corpse](food.md) that
burns away never rises. You are told what you see go: "The scroll burns up.", "3 potions freeze and shatter."

{{#include generated/spell-ruin.md}}

## Spells with a rule of their own

Some [spells](magic.md#the-spells) do one thing no other does. None of them spares you or your [allies](allies.md).

- **Thunderclap** and **Benison** go off about you: they ask for no aim, and you are at their
  centre. Thunderclap strikes you with the full roll, as everyone at a burst's centre, and leaves
  you [Stunned](status-effects.md#status-stunned) with the rest; the whole level hears it. Benison's one roll heals you and every ally of yours within it who is hurt; it does nothing to anyone
  else, and it is not cast when no one there is hurt.
- **Leech** gives you back half the [hit points](hit-points.md#hit-points) it takes, rounded down, never past your maximum. The undead, immune to [necrotic](#spell-types) harm, give nothing.
- **Rebuke** reaches only the undead, yours and others': the living in it are not touched. It is
  fire, so it [burns](status-effects.md#burning) as fire does and leaves a fire at its centre.
- **Bramble** catches only those standing where things grow: grass, marsh, reeds, flowers, bushes,
  forest floor, fungus (the [terrain](terrain.md) table says which). On stone it does nothing.
- **Gale** drives everyone on its line a cell straight away from you, the nearest first: one with
  someone behind it stays put, and so does one [held or webbed](status-effects.md#status-held). It harms no one, and it puts out the fires on its line and tears away the [zones](#zones) of fire, foul cloud, and darkness on it; slick ground stays. A shut door stops it.
- **Bind** holds one hostile you see where it stands: it is [Pinned](status-effects.md#status-pinned) two degrees (more
  as your Presence grows), so it cannot step, though it may still strike. A pin eases a degree with each action it takes.
  It never misses. A [wand of bind](wands-and-rods.md#wands) casts it; **Shatter**'s [wand](wands-and-rods.md#wands) is a [force](#spell-types) burst like the spell.
- **Blink** takes you to a free cell within 5, any of them as likely, through walls or not. It
  works while you are [held](status-effects.md#status-held), [webbed](status-effects.md#status-webbed), or [pinned](status-effects.md#status-pinned), and ends all three.
- **Sense the quick** lets you feel every living creature within 8 cells that you cannot see as a
  heartbeat, [heard](hearing.md) up to 2 cells from where it is, every turn while it lasts. The undead, constructs, and
  animated things have none.
- **Cloudburst**, cast under the open sky, brings an hour of [rain](weather.md) and at once puts out every fire and
  everyone [burning](status-effects.md#burning) under the open sky on the level.

## Zones

A **zone** is an area that lies on the floor for a while: [Balefire](magic.md#the-spells) burns, [Reek](magic.md#the-spells) poisons, [Mirk](magic.md#the-spells) darkens, and [Slick](magic.md#the-spells)
trips. All four come only from [scrolls](magic.md#learning-spells-from-scrolls) you find. A zone's cells stay as they were laid, and it never spreads. It
takes you and your [allies](allies.md) like anyone: the caster has no protection against their own, though a
kill it makes is theirs. A zone laid over another replaces it on the cells they share. It stays when you
leave the level, its time running while you are away. No zone can be laid on the [overworld](overworld.md).

| Kind | Who it takes | What it does |
|---|---|---|
| Fire | whoever starts a turn on it or steps onto it | the spell's [fire](#spell-types) harm, one number rolled as it is laid and the same for all its time, halved by [Evasion](talents.md#talent-evasion); then fire's one chance in four of [burning](status-effects.md#burning). Each cell sheds a fire's light, and the [ice](temperature.md#water-freezes) under it melts as it is laid. It lies only on walkable ground out of the water |
| Foul cloud | whoever starts a turn in it | [Poisoned](status-effects.md#poisoned) one degree more, no other harm; nothing to the [poison-immune](damage-types.md). It does not block sight |
| Slick ground | whoever steps onto it or off it | a [Dexterity](stats.md#stat-dexterity) check against 12: failed, [Staggered](status-effects.md#status-staggered), the next action half again as slow. The step is still taken. Nothing incorporeal slips. It lies only on walkable ground out of the water |
| Darkness | no one directly | no light reaches its cells, the sun's included, and a light inside sheds none, a burning creature's too. Eyes that need light see nothing inside; [darkvision](vision.md#the-four-types) sees in as far as it reaches. The player's eyes follow the same rule, and from inside it they see only the cell they stand on and what is lit beyond |

A [cold](#spell-types) spell puts out the fire on each cell it reaches, and the rest of the zone burns on. A zone shows as a still
tint of its cells, and look mode names it.

## How a spell grows

Each spell's row lists its ladders: a stat and the scores at which the spell changes. Your score
reaches the number, you cast that rung. The score is the one your character screen shows, with
what your [status effects](status-effects.md), [stat damage](status-effects.md), and [magic items](magic-items.md) do to it; a low score never shrinks a
spell below its base. Spell damage is rolled dice, as written in the spell's row; what grows it is
its ladder, and your talents and items.

Your [talents](talents.md) and [magic items](magic-items.md) may add to that, each in its own way, said in its own row:

- **A rung**: *[Empower](talents.md#talent-empower)* (a [wizard](classes.md#wizard)'s at 32) takes your spells a rung up their damage and
  healing ladders, *[Archmage](talents.md#talent-archmage)* (at 60) two; on the last rung they stay there.
- **Bigger dice**: *[Keen mind](talents.md#talent-keen-mind)*, taken for a school of magic or an element, makes those spells roll
  dice a size bigger for you: a d6 is a d7, a d8 a d9.
- **A die more**: *[Brimming](talents.md#talent-brimming)* rolls a die more on your damage and healing spells, and they cost a
  tenth more [mana](mana.md#spending-it).
- **A bonus**: *Potent* [magic items](magic-items.md#prefixes) add 1 to 3 to every spell's roll, harm or healing (a Potent +2 ring
  makes 4d6 into 4d6+2).
- **A die rolled again**: *[Sure hand](talents.md#talent-sure-hand)* rolls a spell die that shows a 1 once more, and that roll stands.
- **Reach and degrees**: *[School focus](talents.md#talent-school-focus)*, taken for a school or an element, makes those spells reach
  a cell farther (range or length, not a burst's radius) and cost 10 % less; [magic items](magic-items.md#suffixes) *of
  Farcasting* add their cells to every spell's reach, *of Lingering* their degrees to every
  blessing and hex.

They apply in that order: the rung, then the dice, then the roll (with its rerolls), then the
harm, then the riders. A [critical](combat.md#criticals) rolls the dice twice, as a blow's does. A **[spell scroll](scrolls.md#spell-scrolls)**
you read casts at the rung the scroll's row says (the [scroll of fireball](scrolls.md#scroll-of-fireball)'s is 1: 6d6), a rung higher
with *[Scroll adept](talents.md#talent-scroll-adept)*, whatever your stats; a **[wand](wands-and-rods.md#wands)** at a rung of its own, a rung higher with
*[Wand mastery](talents.md#talent-wand-mastery)* and higher again by *of Wands*. Your other talents and items work on them as on
your own casts, and their reach and degrees are yours. Most of these talents are Spellcraft's,
the tree a [wizard](classes.md#wizard) takes bonus talents from beside Scholar; see [Talents](talents.md) for every talent that
touches spells.

Two kinds of damage only magic does, **[force](damage-types.md#damage-force)** and **[necrotic](damage-types.md#damage-necrotic)**, join the
[damage types](damage-types.md).

Spells you cast are yours: what they kill earns you its [experience](experience.md), and a spell of your own that
catches you can kill you.
