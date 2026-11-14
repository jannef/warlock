# Companions

A **companion** is a beast that is yours for good. It stays until it is slain, and there is no
sending it away. You have at most one at a time. A [ranger](classes.md#ranger) starts with one;
anyone trained in [Handle animal](skills.md#skill-handle-animal) can [tame](#taming) one.

A companion is an [ally](allies.md) like any other, with the same [orders](allies.md#orders), the
same side, and the same share of its kills. The messages call it by its **name**, drawn when it
becomes yours ("Snel bites the kobold.", "Snel comes to heel.", "Snel dies."). The
[status bar](the-screen.md#the-screen) says how it fares ("Snel: unhurt", or "Snel: with you" while
it travels with you on the overworld). [Look mode](controls.md#look-mode) names it ("Snel, your
hound"), and the Character screen lists it under your class.

A companion **always comes along**. Wherever it stands when you take the stairs or leave an area,
held fast, holding its place, or not, it arrives beside you ("Snel comes with you."). It rides with
you unseen across the overworld, and steps out beside you at the next place ("Snel follows you
in."). Nothing but death keeps it from you. When it dies it leaves a corpse like any creature, and
nothing else happens to you: find another to tame.

Beasts that hunt by scent have a **nose**. Within so many cells (see the Nose column of
[Creatures in numbers](creature-numbers.md#pace-senses-and-size)), your companion smells a foe of
yours however dark it is, even **through a closed door**, though never through a wall. It goes for
the foe when its eyes find nothing. When it scents something you cannot see, it growls once ("Snel
growls at the dark."). A door it cannot open keeps it from what it smells, so open it.

## The collar

A beast you keep, a companion or a [familiar](familiars.md#familiars), wears one thing, on its
neck. [Give](abilities.md#ability-give), in the Allies group, fastens something of yours that goes
on a neck, such as an amulet, on the beast beside you. It takes the time wearing it would take you
([What takes time](time.md#what-takes-time)). You give one of a stack, and it is taken off you first
if you wore it. With two beasts beside you, you are asked which way.

Whatever the beast wore before falls to the floor beneath it. The magic works on the beast as it
would on you: its [resistances](damage-types.md#armour-and-resistance), its armour, its eyes. You
learn nothing about what you give by the beast's wearing it.

You cannot take a collar back. Give another and pick up the old one, or find it among the beast's
remains. Goods you have not paid for stay with you.

## Starting companions

The [ranger](classes.md#ranger)'s **Animal companion** (level 1) puts a beast at their side from the
first turn. It is one of their race's kinds, drawn by chance as the run begins, or rarely one of the
rares that any race may begin with. The [story](background.md#the-companion) of how it came to be
yours is told with the rest of your background.

{{#include generated/companion-starts.md}}

## Taming

Anyone trained in [Handle animal](skills.md#skill-handle-animal) may **tame** a beast, at any
level. Choose **Tame** in the [abilities menu](abilities.md) and the direction of a beast next to
you.

Only some kinds can be tamed: see the Tameable column of
[Creatures in numbers](creature-numbers.md#pace-senses-and-size). The beast must also not be your
foe (wildlife, or an animal a [Beast-friend](talents.md#talent-beast-friend) has calmed), or must
not have seen you (asleep, say).

You crouch beside it for a while, holding out food if you carry any. If you are hurt, or it leaves
your side, or someone comes into view, you stop, and nothing is lost but the time. Then one check
decides:

- If you make it, the beast comes to your hand and is your companion, named, at your level.
- If you fail, it snaps at you and backs away. A beast that was your foe has seen you now.

Try again as often as you like: time is the only price. While you have a companion you cannot tame
another. The ranger's Animal companion makes them a little better at it. The exact check and the
chances:

{{#include generated/companion-taming.md}}

## Growth

A companion's level is **yours**. Whatever its kind's own level, it is raised to your level when it
becomes yours and every time you gain a level ("Snel grows stronger."), and it never falls.

Above its kind's level it gains hit points every level. Now and then it also gains a surer bite,
more damage and another die of it, natural armour, and Strength, Dexterity, and Toughness. Its
tougher hide is its body's Protection at its level, as the
[hide rule](creature-bodies.md#hides-and-bodies) gives every creature. Its speed, senses, size, and
resistances stay its kind's. Summons do not grow: their power is their spell's. The exact table, as
the game computes it:

{{#include generated/companion-growth.md}}

From level 18 the ranger's **[Pack tactics](classes.md#talent-pack-tactics)** makes the two of you
hunt as one. A creature beside you both is flanked: your melee attacks and your companion's at it
get +2 [to hit](combat.md#hitting), and your companion shares your
[favoured prey](classes.md#ranger) bonus.

## Hunter's bond

From level 25 the ranger's **[Hunter's bond](classes.md#talent-hunters-bond)** lets them share what
their companion has. It works while the companion stands on the same level, not while it rides with
you across the overworld.

- **Shared sight.** You see every cell it sees, by its own eyes: as far as its kind sees, in the
  dark as its [vision](vision.md) allows, and around the corner you cannot see past. You also see
  the cell of every foe its [nose](#companions) finds, however dark, even **through a closed door**,
  but never through a wall. These are seen cells like any other: you remember them, can look at
  them, and can aim at them. It finds no hidden [trap](traps.md) or [secret door](searching.md)
  for you, and smells nothing of a disguised [mimic](chests.md). Being dazzled leaves its view
  untouched.
- **Shared buffs.** A [buff](status-effects.md#buffs-and-cooldowns) that lands on either of you
  lands on the other too, at the same strength, and each of you wears it off alone. This is any
  [status](status-effects.md) that a buff [spell](magic.md) lays or a [potion](potions.md) lends,
  whatever laid it ("Snel is bark-skinned too."). It reaches your companion on the overworld too. A
  hex, a harm, or a cooldown never crosses.

From level 40, **[Sic 'em](abilities.md#ability-sic)** sets your companion on a
[hostile](creatures.md) creature you see within 10 cells. It costs 5
[focus](resources.md#resource-focus) and the [time](turns.md#what-actions-cost) of a wait. Your
companion goes for that creature before any other, and follows it as long as it sees or smells it.

It is also [Savage](status-effects.md#status-savage): half again as fast, and each of its next three
[hits](combat.md#hitting) is a critical with no confirmation roll. Each hit spends a degree;
otherwise a degree wears off each minute. Give it another order and the chase is over. Savage is its
own: it never crosses to you.

## The companion's rules

{{#include generated/companion-rules.md}}
